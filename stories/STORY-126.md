---
document_type: story
story_id: STORY-126
epic_id: E-19
version: "1.0"
status: completed
# BC status: BCs authored and anchored below; all traces complete.
producer: story-writer
timestamp: 2026-06-20T00:00:00Z
phase: f3
points: 8
priority: P0
depends_on: [STORY-123, STORY-124]
blocks: [STORY-127]
behavioral_contracts:
  - BC-2.01.013
  - BC-2.01.015
  - BC-2.01.017
verification_properties: [VP-029, VP-031]
tdd_mode: strict
target_module: reader
subsystems: [SS-01]
estimated_days: 3
feature_id: f3-pcapng-reader-support
wave: 54
inputs:
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.013.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.015.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.017.md
# Dependency anchor: STORY-126 depends on STORY-123 because SHB parse and
#   byte-order state are prerequisites for SPB and block-walk dispatch. STORY-126
#   depends on STORY-124 because the interface table existence check (empty-table
#   guard for SPB → E-INP-009) requires the Vec<InterfaceInfo> structure from
#   STORY-124. STORY-126 does NOT depend on STORY-125 (SPB parsing and block-skip
#   are independent of EPB parsing; they share the block-walk loop but EPB decode
#   is a separate dispatch arm).
# Subsystem anchor: SS-01 owns this story's scope because BC-2.01.013,
#   BC-2.01.015, and BC-2.01.017 are all SS-01 behavioral contracts per their
#   traceability tables, anchored to src/reader.rs (C-4) per ARCH-INDEX
#   Subsystem Registry.
input-hash: "a59f35b"
---

# STORY-126: SPB Parse, Explicit Block-Skip Dispatch (F-07), and Error-Surface Contract

## Narrative

- **As a** security analyst running wirerust against pcapng captures that may include Simple
  Packet Blocks, Interface Statistics Blocks, obsolete Packet Blocks, or Decryption Secrets
  Blocks
- **I want** the reader to correctly parse SPBs (producing zero-timestamp RawPackets), skip
  all non-EPB/IDB/SHB blocks with explicit dispatch arms and accurate counters, and surface
  all pcapng errors via anyhow context chains
- **So that** SPB-only captures yield packets, unknown blocks are safely skipped (including
  DSB without leaking key material to logs), and the error-surface contract enables useful
  diagnostics for users

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.01.013 | Parse pcapng Simple Packet Block (SPB): Packet Data Without Timestamp |
| BC-2.01.015 | Unknown pcapng Block Types Are Silently Skipped via block-total-length |
| BC-2.01.017 | pcapng Block-Level Parse Errors Surface via anyhow Context Chain |

## Acceptance Criteria

### AC-001 (traces to BC-2.01.013 AC-001 — SPB empty-table guard)
wirerust MUST check that the interface table is non-empty before processing an SPB. If
empty → `Err` mapping to **E-INP-009** with EXACT message:
`"SPB encountered but interface table is empty — no IDB has been parsed"`.
Snaplen from `idb[0]` is NOT used in SPB `captured_len` computation (ADR-009 rev 8
Decision 9 amendment); this guard solely prevents an unchecked index on an empty Vec.

**Test:** `test_BC_2_01_013_empty_interface_table_guarded`

### AC-002 (traces to BC-2.01.013 AC-002 — padding strip and spb_data_available formula)
On the raw-block path `RawBlock.body = [original_len:4][padded data]`. The canonical
symbol `spb_data_available = body.len() - 4` (NOT `body.len()` alone — that is 4 bytes
too large because it counts the `original_len` field). wirerust MUST compute
`captured_len = min(original_len, body.len() - 4)` and slice to exactly `captured_len`
bytes before populating `RawPacket.data`. The bare `body.len()` bound MUST NOT be used.
Snaplen is NOT applied (ADR-009 rev 8 Decision 9 amendment). Handing the padded or
unbounded slice to downstream decoders verbatim is prohibited.

**Test:** `test_BC_2_01_013_padding_strip`

### AC-003 (traces to BC-2.01.013 postcondition 3 — zero timestamps)
Every `RawPacket` produced from an SPB MUST have `timestamp_secs = 0` and
`timestamp_usecs = 0`. There is no per-packet timestamp in the SPB format. Downstream
consumers (reassembly, findings timestamp) receive zero-timestamps for SPBs.

**Test:** `test_BC_2_01_013_zero_timestamps`

### AC-004a (traces to BC-2.01.013 AC-004a — body-too-short E-INP-008)
An SPB where the crate returns a valid-framed `RawBlock` (btl ≥ 12, btl % 4 == 0) but
the body is fewer than 4 bytes (insufficient for `original_len: u32`) causes wirerust
body-decode to return `Err` mapped to **E-INP-008**. wirerust checks `body.len() >= 4`
itself on the raw path before decoding SPB fixed fields — the crate does NOT enforce this
minimum for the caller. Constructible fixture: btl=12 → body=0 bytes < 4 → E-INP-008.
Covered by HS-107 Case F.

**Test:** `test_BC_2_01_013_spb_body_truncated_e_inp_008`

### AC-004b (traces to BC-2.01.013 AC-004b — SPB_FIXED_OVERHEAD_BYTES = 4)
The named constant `SPB_FIXED_OVERHEAD_BYTES` MUST equal 4 (body-relative;
`original_len: u32` only). This constant MUST NOT be confused with
`EPB_FIXED_OVERHEAD_BYTES = 20`. The minimum valid SPB `block_total_length` is 16 bytes
(12 outer + 4 body-fixed for `original_len` + 0 padded data).

**Test:** `test_BC_2_01_013_fixed_overhead_constant`

### AC-005 (traces to BC-2.01.013 AC-003 — no-panic SEC-005)
The SPB parse path MUST return `Err` for any malformed or truncated input.
`unwrap()`, `expect()`, and `panic!()` are prohibited in the SPB parse path.

**Test:** `test_BC_2_01_013_no_panic_malformed`

### AC-006 (traces to BC-2.01.015 AC-001 — explicit match arms for ALL block types — F-07 MANDATORY)
The block-walk dispatch MUST have EXPLICIT named arms for EVERY identified skip-block type.
**The implementation MUST NOT use a single wildcard arm that implicitly handles both
known-skip and unknown-skip types without distinct counter semantics.** Specifically, the
dispatch table MUST include ALL of the following arms (in the match on raw block-type bytes):

- **SHB arm** (`0x0A0D0D0A`) — SHB parse (second SHB → E-INP-012)
- **IDB arm** (`0x00000001`) — IDB parse (three-level precedence per BC-2.01.011 AC-006)
- **EPB arm** (`0x00000006`) — EPB parse
- **SPB arm** (`0x00000003`) — SPB parse (this story)
- **OPB arm** (`0x00000002`) — EXPLICITLY named skip arm; increments BOTH `skipped_blocks`
  AND `opb_skipped` via `saturating_add(1)` on each counter
- **NRB arm** (`0x00000004`) — EXPLICITLY named skip arm; increments `skipped_blocks` only
- **ISB arm** (`0x00000005`) — EXPLICITLY named skip arm; increments `skipped_blocks` only
- **SJE arm** (`0x00000009`) — EXPLICITLY named skip arm; increments `skipped_blocks` only
- **DSB arm** (`0x0000000A`) — EXPLICITLY named skip arm; body bytes MUST NOT be logged
  (SEC-007: DSB carries TLS key material); increments `skipped_blocks` only.
  **CRITICAL:** DSB is NOT a named `Block` enum variant in `pcap_file::pcapng::Block`.
  On the raw-block path, match the type bytes `0x0000000A` directly.
  Do NOT attempt to match a `DecryptionSecrets` enum arm that does not exist.
- **Default catch-all arm** (`_`) — any remaining unknown type; increments `skipped_blocks`
  only; no diagnostic output at any level

**Test:** `test_BC_2_01_015_dispatch_known_and_skip_unknown`

### AC-007 (traces to BC-2.01.015 AC-003 — OPB increments BOTH counters)
OPB (type `0x00000002`) carries packet data but is obsolete/deprecated superseded by EPB.
wirerust MUST skip it (NOT parse it); OPB packet data is intentionally NOT ingested.
When an OPB is skipped, BOTH `skipped_blocks` AND `opb_skipped` are incremented
(saturating). `opb_skipped <= skipped_blocks` always holds. This sub-counter drives the
OPB mergecap clause in the zero-packet notice (BC-2.01.009 PC6).

**Test:** `test_BC_2_01_015_opb_skipped_not_parsed` — assert OPB-only pcapng yields
`packets.len()==0`; `source.skipped_blocks == N`; `source.opb_skipped == N` (same N
since all skips are OPB); a non-OPB unknown block increments only `skipped_blocks`.

### AC-008 (traces to BC-2.01.015 AC-002 — no output at any log level, SEC-007)
For each block type on the skip path, the skip MUST NOT emit any warning, error, finding,
or log entry at any level (trace, debug, info, warn, error). Block body bytes MUST NOT be
logged, printed, or included in any diagnostic output regardless of severity. DSB (type
`0x0000000A`) carries TLS key material; logging its bytes at any level is a security
violation (SEC-007).

**Test:** `test_BC_2_01_015_no_output_on_skip`

### AC-009 (traces to BC-2.01.015 AC-004 — loop-break on Err, forward-progress)
The block-walk loop MUST `break` (or return `Err`) on any `Err(_)` from
`next_raw_block`. The crate does NOT advance its cursor on error (`read_buffer.rs:65`);
retrying after error spins indefinitely (CWE-835). The documented rustdoc example with
an empty `Err(_) => {}` arm is WRONG and MUST NOT be copied — it causes an infinite loop
on the same error position.

**Test:** `test_BC_2_01_015_loop_break_on_error`

### AC-010 (traces to BC-2.01.015 postcondition 9 — counter surfacing on PcapSource)
Both `skipped_blocks: u32` and `opb_skipped: u32` MUST be public fields on the
`PcapSource` struct returned by `from_pcap_reader`. `from_pcap_reader` MUST NOT emit any
stderr output — it surfaces the counters and returns. `main.rs` reads
`source.skipped_blocks` and `source.opb_skipped` AFTER `Ok(source)` is returned and
constructs the canonical zero-packet notice (BC-2.01.009 PC6) if `packets.is_empty()`.
Counters use `saturating_add` — `u32` is sufficient; realistic pcapng files cannot
approach 4 billion blocks.

**Test:** `test_BC_2_01_015_skipped_blocks_counter_and_notice` — verify both counter
fields on returned `PcapSource`; verify OPB increments both counters; verify non-OPB
skip increments only `skipped_blocks`.

### AC-011 (traces to BC-2.01.017 postcondition 1 — anyhow context chain for all pcapng errors)
All pcapng parse failures MUST surface as `Err(anyhow::Error)` via `?` propagation with
`.context(...)` identifying the block type. Bare `?` without context is prohibited for
pcapng paths. Context strings (canonical per BC-2.01.017 PC1):
- `"Failed to parse pcapng Section Header Block"` → E-INP-008 (SHB body-decode)
- `"Failed to parse pcapng Interface Description Block at interface index <N>"` → E-INP-008
- `"Failed to parse pcapng Enhanced Packet Block (packet <seq>)"` → E-INP-008 (EPB body-decode)
- `"Failed to read pcapng Simple Packet Block"` → E-INP-008 (SPB body-decode)
- `"Failed to skip pcapng block (type=0x{block_type:08X})"` → E-INP-010 (crate framing rejection)
- `"pcapng Enhanced Packet Block encountered before any Interface Description Block"` → E-INP-009
- `"pcapng Simple Packet Block encountered before any Interface Description Block"` → E-INP-009
- `"pcapng Enhanced Packet Block references interface <id> but only <n> interfaces defined"` → E-INP-010
- `"pcapng Interface Description Block link type rejected"` → E-INP-001

**Test:** `test_BC_2_01_017_all_error_paths_have_context`,
`test_BC_2_01_017_epb_before_idb_emits_einp009_context`

### AC-012 (traces to BC-2.01.017 postcondition 3 — cross-cutting no-panic parent)
For ANY byte sequence fed to `PcapSource::from_pcap_reader`, the function returns
`Ok(_)` or `Err(_)` — it MUST NOT panic and MUST NOT loop indefinitely. This
cross-cutting contract is the top-level statement of SEC-005; per-BC no-panic ACs in
STORY-123..126 are specializations. VP-028 (cargo-fuzz `fuzz_pcapng_reader`) is the
primary verification vehicle — it is an **F6 hardening deliverable, NOT an F3
obligation**. The F3 obligation is ensuring no `unwrap()`/`expect()` appear in any
pcapng code path.

**Test:** `test_BC_2_01_017_no_panic_truncated_pcapng`

## Behavioral Contracts Table

| BC | Version | Clauses Covered |
|----|---------|-----------------|
| BC-2.01.013 | v1.9 | AC-001 (empty-table guard → E-INP-009 exact message), AC-002 (padding strip; spb_data_available=body.len()-4; captured_len=min(original_len, body.len()-4); snaplen not applied), PC3 (zero timestamps), AC-004a (body-too-short → E-INP-008; btl=12 window), AC-004b (SPB_FIXED_OVERHEAD_BYTES=4), AC-003/SEC-005 (no-panic), PC5 (empty-table E-INP-009 exact message string), Inv2 (spb_data_available canonical formula), EC-001..008 (all edge cases) |
| BC-2.01.015 | v1.8 | AC-001 (F-07: explicit match arms for ALL named block types; no bare wildcard), AC-003 (OPB increments BOTH skipped_blocks + opb_skipped), AC-002/SEC-007 (no output at any level; DSB body bytes not logged), AC-004 (loop-break on Err; no CWE-835 spin), AC-005/SEC-005 (no-panic in skip path), AC-006 (counter surfacing on PcapSource), PC9 (skipped_blocks/opb_skipped fields; main.rs reads post-Ok; emission gated by "valid file + zero packets"), Inv1-5 (crate forward-progress; skip via RawBlock body discard; no diagnostic output; SHB/IDB/EPB/SPB NOT in skip path), EC-001..013 |
| BC-2.01.017 | v1.6 | PC1 (anyhow context strings; bare ? forbidden), PC2 (no partial PcapSource on error), PC3 (cross-cutting no-panic; block-walk breaks on Err), PC4 (no unwrap/expect), PC5 (E-INP-005 wrapping), Inv1-3 (? + context style; every error path has context; taxonomy codes are categorization not runtime codes), EC-001..006 |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| SPB body decode (`original_len`, `spb_data_available`, `captured_len`, padding strip) | `src/reader.rs` (block-walk dispatch arm) | Effectful shell (I/O: block reading) |
| Block-walk dispatch (explicit match arms for all 9 named block types + catch-all) | `src/reader.rs` | Effectful shell (block-type routing) |
| Skip counters (`skipped_blocks`, `opb_skipped`) on `PcapSource` | `src/reader.rs` (PcapSource struct fields) | Pure state (counter increment, no I/O) |
| Error context chain (`anyhow::Context`) | `src/reader.rs` (all pcapng error paths) | Cross-cutting (error propagation) |

Architecture section references: `architecture/module-decomposition.md` (SS-01 C-4,
`src/reader.rs`); ADR-009 Decision 2 (SPB coverage, unknown block types silently skipped),
Decision 8 (forward-progress contract; crate rejects btl<12; caller breaks on Err),
Decision 9 amendment (snaplen NOT enforced for SPB — same policy as EPB),
Decision 10 (panic surface), Decision 19 (skip counter surfacing via PcapSource fields),
Decision 20 (uniform error-code rule: btl<12/misaligned→E-INP-010; btl=12→body=0<4→
E-INP-008 via wirerust body-decode; SPB fixed-field minimum=4),
Decision 22 (SPB canonical formula: spb_data_available=body.len()-4; captured_len=
min(original_len, body.len()-4); bare body.len() is WRONG — 4 bytes too large).

## Forbidden Dependencies

- `src/reader.rs` MUST NOT gain any new crate dependency. +0 new crates per ADR-009 Decision 1.
- The block-walk dispatch MUST NOT match `Block::DecryptionSecrets` or any DSB named variant
  — no such variant exists in `pcap_file::pcapng::Block` (9-variant enum per
  `block_common.rs:146-166`). Attempting to match a non-existent variant is a compile error.
- `SPB_FIXED_OVERHEAD_BYTES` MUST be defined as a named constant with value `4`.
  Using the value `20` (EPB overhead) is a HIGH-severity correctness defect.
- The bare `body.len()` (WITHOUT subtracting 4) MUST NOT be used as the SPB `captured_len`
  bound — it is 4 bytes too large because it counts the `original_len` field itself
  (ADR-009 Decision 22 / BC-2.01.013 Inv2).
- `from_pcap_reader` MUST NOT emit anything to stderr — it is a library function.
  Counter surfacing (PC9) is the ONLY mechanism; emission is `main.rs` responsibility.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SPB with `original_len > spb_data_available` (on-disk truncation) | `captured_len = min(original_len, body.len()-4) = body.len()-4`; data sliced to `spb_data_available` bytes; no OOB |
| EC-002 | SPB where `original_len == spb_data_available` (= `body.len()-4`) | `captured_len = original_len`; no truncation; data copied in full |
| EC-003 | SPB with `original_len = 0` | `RawPacket { timestamp_secs: 0, timestamp_usecs: 0, data: vec![] }` |
| EC-004 | SPB with btl=12 (crate frames block successfully; body=0 bytes < 4 SPB fixed fields for `original_len:u32`) | `Err` mapping to E-INP-008 (body-too-short; wirerust body-decode checks body.len()>=4 itself). No panic. Covered by HS-107 Case F. |
| EC-005a | SPB with btl=8 (< 12) | Crate rejects before returning block → `Err` mapping to E-INP-010 |
| EC-005b | SPB with btl=14 (≥12 but 14%4≠0 — misaligned) | Crate rejects for 4-byte alignment → `Err` mapping to E-INP-010. Covered by HS-107 Case E. |
| EC-006 | SPB before any IDB (empty interface table) | `Err` mapping to E-INP-009 with EXACT message: `"SPB encountered but interface table is empty — no IDB has been parsed"` |
| EC-007 | OPB (type `0x00000002`) encountered in block stream | Skipped via explicit OPB arm; `skipped_blocks.saturating_add(1)`; `opb_skipped.saturating_add(1)`; no packet produced; no log output |
| EC-008 | ISB (type `0x00000005`) encountered | Skipped via explicit ISB arm; `skipped_blocks.saturating_add(1)` only; no output |
| EC-009 | DSB (type `0x0000000A`) followed by EPB | DSB body discarded WITHOUT logging any body bytes (SEC-007); `skipped_blocks++`; EPB parsed normally; packet produced |
| EC-010 | SHB-only pcapng (no IDB, no blocks of any kind after the SHB) | No blocks reach the skip arm; `skipped_blocks==0`, `opb_skipped==0` |
| EC-011 | Unknown block with btl < 12 | Crate returns `Err(_)` → loop breaks immediately; no spin (CWE-835); `Err` propagated |
| EC-012 | Multiple consecutive OPBs (N OPBs, no EPBs) | `packets.len()==0`; `skipped_blocks==N`; `opb_skipped==N` |
| EC-013 | DSB then EPB; assert DSB body bytes absent from all stderr/stdout | DSB bytes never logged at any level; EPB produces `RawPacket`; overall `Ok(PcapSource)` |

## Tasks

1. **SPB body decode:** In the block-walk dispatch, add the SPB arm (`0x00000003`):
   - Check `body.len() >= 4` (SPB_FIXED_OVERHEAD_BYTES); if false → `Err` with context
     `"Failed to read pcapng Simple Packet Block"` → E-INP-008.
   - Read `original_len: u32` from the first 4 bytes (endian-corrected per SHB BOM).
   - Compute `spb_data_available = body.len() - 4` (canonical symbol per Decision 22).
   - Compute `captured_len = min(original_len, spb_data_available as u32)`.
   - Slice body to exactly `captured_len` bytes (stripping padding).
   - Produce `RawPacket { timestamp_secs: 0, timestamp_usecs: 0, data: ..., original_len }`.
   - Append to `PcapSource.packets`.
2. **Add counter fields:** Add `pub skipped_blocks: u32` and `pub opb_skipped: u32` as
   public fields on `PcapSource`. Initialize to 0.
3. **Implement F-07 explicit block-walk dispatch:** Replace any wildcard-only skip logic
   with named arms for all 9 skip-block types as specified in AC-006. Each arm:
   - OPB (`0x00000002`): `skipped_blocks = skipped_blocks.saturating_add(1);`
     `opb_skipped = opb_skipped.saturating_add(1);`
   - NRB/ISB/SJE/DSB (`0x00000004`/`0x00000005`/`0x00000009`/`0x0000000A`):
     `skipped_blocks = skipped_blocks.saturating_add(1);`
   - Default catch-all (`_`): `skipped_blocks = skipped_blocks.saturating_add(1);`
   - DSB: ensure body bytes are NOT logged, not even at trace level.
4. **Wrap all pcapng errors with `.context(...)`** strings per BC-2.01.017 PC1 (AC-011).
   Audit existing SHB/IDB/EPB error paths and add missing context strings in this story.
5. **Ensure loop breaks on `Err(_)`** from `next_raw_block`. The break must be a hard
   `break` or `return Err(...)`, not `continue`.
6. **Write VP-029 proptest** (block-walk termination): generate arbitrary block sequences
   including malformed lengths; assert loop terminates and result is Ok or Err (not spin).
7. **Write VP-031 proptest** (SPB framing arithmetic): for all (original_len: u32, body: &[u8])
   where `body.len() >= 4`; assert `captured_len == min(original_len, body.len()-4)` and
   the returned data slice has exactly `captured_len` bytes with no OOB access.
8. Run `cargo test --all-targets` (regression: all prior reader tests must remain green).
9. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_01_013_empty_interface_table_guarded` | Unit |
| AC-002 | `test_BC_2_01_013_padding_strip` | Unit |
| AC-003 | `test_BC_2_01_013_zero_timestamps` | Unit |
| AC-004a | `test_BC_2_01_013_spb_body_truncated_e_inp_008` | Unit |
| AC-004b | `test_BC_2_01_013_fixed_overhead_constant` | Unit |
| AC-005 | `test_BC_2_01_013_no_panic_malformed` | Unit |
| AC-006 | `test_BC_2_01_015_dispatch_known_and_skip_unknown` | Unit |
| AC-007 | `test_BC_2_01_015_opb_skipped_not_parsed` | Unit |
| AC-008 | `test_BC_2_01_015_no_output_on_skip` | Unit |
| AC-009 | `test_BC_2_01_015_loop_break_on_error` | Unit |
| AC-010 | `test_BC_2_01_015_skipped_blocks_counter_and_notice` | Unit |
| AC-011 | `test_BC_2_01_017_all_error_paths_have_context`, `test_BC_2_01_017_epb_before_idb_emits_einp009_context` | Unit |
| AC-012 | `test_BC_2_01_017_no_panic_truncated_pcapng` | Unit |
| VP-029 | proptest: block-walk termination over arbitrary block sequences | Property |
| VP-031 | proptest: SPB framing arithmetic `min(original_len, body.len()-4)` for body.len()>=4 | Property |

## Previous Story Intelligence

- STORY-124 established the `Vec<InterfaceInfo>` structure; `InterfaceInfo` carries only
  `linktype: DataLink` and `if_tsresol: u8` — there is NO `snaplen` field (ADR-009 Decision
  9 amendment / F-M3). STORY-126's SPB empty-table guard (`idb.is_empty()`) is the
  structural precondition check; it does NOT attempt to access `idb[0].snaplen`.
- STORY-123 set up the block-walk loop skeleton. STORY-126 completes the dispatch table by
  adding the SPB arm and ALL named skip arms. The F-07 constraint (explicit named arms) was
  identified during adversarial review; retrofitting after a wildcard arm is already merged
  is harder than building correctly from the start.
- The `skipped_blocks` and `opb_skipped` fields on `PcapSource` are introduced in
  STORY-126. If STORY-123..125 have stubs for these fields already, use them; otherwise add
  them fresh. Do NOT add a `snaplen` field — it is read-and-discarded from IDB fixed fields
  and is explicitly absent from `InterfaceInfo`.
- STORY-123 reverted the negative test `test_BC_2_01_004_rejects_pcapng` into a positive
  acceptance test. STORY-126 adds the SPB parse path and skip dispatch without revisiting
  the pcapng routing (that is already in place).

## Architecture Compliance Rules

Derived from ADR-009 rev 9 and BC-2.01.013/015/017:

1. **F-07: explicit match arms are MANDATORY** — no single wildcard arm that handles
   both OPB (with its dual-counter semantics) and generic unknowns (single-counter).
   OPB explicitly increments BOTH `skipped_blocks` AND `opb_skipped`. This is the
   normative enforcement of BC-2.01.015 AC-001 + AC-003.
2. **`spb_data_available = body.len() - 4`** — the canonical symbol (ADR-009 Decision 22).
   The bare `body.len()` is 4 bytes too large (counts the `original_len` field) and MUST
   NOT be used as the SPB `captured_len` bound.
3. **`SPB_FIXED_OVERHEAD_BYTES = 4`** (body-relative). Combined minimum btl = 16 bytes
   (12 outer + 4 body-fixed). btl=12 → body=0 < 4 → E-INP-008 window for SPB is btl=12
   ONLY. `btl=16` (body=4, spb_data_available=0) is the minimum legal SPB — parse succeeds
   with empty data.
4. **DSB has no named enum variant** on the raw-block path — `pcap_file::pcapng::Block`
   has a 9-variant enum with no `DecryptionSecrets` arm (`block_common.rs:146-166`).
   Match the type bytes `0x0000000A` directly on the raw-block path.
5. **Loop break on Err is the ONLY correct response** — the crate cursor does not advance
   on error (`read_buffer.rs:65`). An empty `Err(_) => {}` arm or `continue` after error
   is CWE-835 (infinite loop).
6. **No diagnostic output for skip or DSB body bytes** — SEC-007 applies to all skip arms,
   not just DSB. Any logging in the skip path is a violation; DSB body-byte logging is a
   security violation.
7. **Error context strings are canonical** — use EXACT strings from BC-2.01.017 PC1.
   Diverging strings break error-taxonomy classification (E-INP-008..010 mapping depends
   on context string content).

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `pcap_file` | 2.0.0 | `RawBlock` / `next_raw_block` API only; `SPB_FIXED_OVERHEAD_BYTES = 4` |
| `anyhow` | existing | `.context(...)` for all pcapng errors; bare `?` without context is prohibited |
| `proptest` | existing | VP-029 and VP-031 property tests |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reader.rs` | Modify | Add SPB decode arm; add explicit skip arms for all 9 block types; add `skipped_blocks`/`opb_skipped` fields to `PcapSource`; add `.context(...)` to all pcapng error paths |
| `tests/reader_tests.rs` (or equivalent) | Modify | Add SPB unit tests, skip-path tests, error-context tests, loop-break test |
| `tests/proptest_reader.rs` (or equivalent) | Create/Modify | VP-029 (block-walk termination) and VP-031 (SPB framing arithmetic) proptests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~6,000 |
| BC files (3 BCs: BC-2.01.013 v1.9, BC-2.01.015 v1.8, BC-2.01.017 v1.6) | ~14,000 |
| ADR-009 rev 9 (canonical constants + skip-block arm list + Decision 22) | ~4,000 |
| `src/reader.rs` (post-STORY-124/125) | ~7,000 |
| Test files + proptests | ~4,000 |
| Tool outputs (cargo test, clippy) | ~1,000 |
| **Total estimated** | **~36,000** |

Within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: [STORY-123, STORY-124]` — STORY-126 requires SHB parse infrastructure and
  byte-order state (STORY-123) and the `Vec<InterfaceInfo>` with non-empty existence check
  (STORY-124). STORY-126 does NOT require STORY-125 — EPB parsing is a sibling arm in the
  block-walk dispatch; SPB parsing and block-skip are independent of EPB and can be
  developed in parallel.
- `blocks: [STORY-127]` — STORY-127 requires the full reader stack (SHB + IDB + EPB +
  SPB + skip dispatch) to be in place before E2E corpus tests can run against the complete
  pcapng reader. STORY-127 must not be dispatched until STORY-123..126 are all merged.
