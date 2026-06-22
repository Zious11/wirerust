---
document_type: story
story_id: STORY-123
epic_id: E-19
version: "1.0"
status: completed
# BC status: BCs authored and anchored below; all traces complete.
producer: story-writer
timestamp: 2026-06-20T00:00:00Z
phase: f3
points: 5
priority: P0
depends_on: []
blocks: [STORY-124, STORY-125, STORY-126, STORY-127]
behavioral_contracts:
  - BC-2.01.009
  - BC-2.01.010
verification_properties: [VP-026]
tdd_mode: strict
target_module: reader
subsystems: [SS-01]
estimated_days: 2
feature_id: f3-pcapng-reader-support
wave: 51
inputs:
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md
# Dependency anchor: STORY-123 has no depends_on because it is the foundation
#   of the entire pcapng reader stack. All other pcapng stories depend on this
#   story's magic-byte probe and SHB parsing infrastructure.
# Subsystem anchor: SS-01 owns this story's scope because BC-2.01.009 and
#   BC-2.01.010 are both SS-01 behavioral contracts per their traceability
#   tables, anchored to src/reader.rs (C-4) per ARCH-INDEX Subsystem Registry.
input-hash: "dc88884"
---

# STORY-123: pcapng Format Detection (Magic-Byte Probe) and SHB Parse

## Narrative

- **As a** security analyst running wirerust against a corpus that includes both classic pcap
  and pcapng captures (including pcapng files with `.cap` extensions)
- **I want** `PcapSource::from_pcap_reader` to transparently detect pcapng files by
  peeking the first 4 bytes without consuming them, then parse the Section Header Block (SHB)
  to establish byte-order and version
- **So that** all downstream block parsing (IDB, EPB, SPB) receives the correctly routed
  reader with the stream still positioned at byte 0 for the appropriate parser

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.01.009 | Accept pcapng Format: Transparent Detection via Magic-Byte Probe |
| BC-2.01.010 | Parse pcapng Section Header Block (SHB): Byte-Order Detection and Version |

## Acceptance Criteria

### AC-001 (traces to BC-2.01.009 postcondition 3 — probe consumes no bytes)
`from_pcap_reader<R: Read>` MUST internally wrap its `R` argument in `std::io::BufReader`
before performing the magic-byte probe. The probe uses `BufReader::fill_buf()` to peek 4
bytes and reads from the filled buffer WITHOUT calling `consume()`. After the probe, the
byte at offset 0 remains the next readable byte. Both the pcapng branch (`next_raw_block`)
and the classic-pcap branch (`PcapReader::new`) receive the SAME BufReader with byte 0
unconsumed.

**Test:** `test_BC_2_01_009_unbuffered_read_routes_correctly` (pass unbuffered `Cursor<&[u8]>`
with valid pcapng SHB; assert `Ok(PcapSource)` with correct routing; assert BufReader
wrapping is present), `test_BC_2_01_009_pipe_stream_probe_observable` (assert next-byte
after probe equals original byte-0)

### AC-002 (traces to BC-2.01.009 postcondition 1 — pcapng branch routes correctly)
When the first 4 bytes are `[0x0A, 0x0D, 0x0D, 0x0A]`, the reader selects the pcapng parse
path and returns `Ok(PcapSource)` for a structurally valid pcapng file. The `smb3.pcapng`
fixture (formerly the negative-assertion fixture for BC-2.01.004) MUST now return
`Ok(PcapSource)` with the correct packet count and link type.

**Test:** `test_BC_2_01_009_smb3_pcapng_accepted`, `test_BC_2_01_009_pcapng_magic_routes_to_pcapng_path`

### AC-003 (traces to BC-2.01.009 postcondition 2 — classic-pcap path unchanged)
When the first 4 bytes are a valid classic-pcap magic (`0xA1B2C3D4`, `0xD4C3B2A1`,
`0xA1B23C4D`, `0x4D3CB2A1`), the existing classic-pcap path is taken unchanged. All prior
reader tests MUST remain green.

**Test:** `test_BC_2_01_009_classic_pcap_routing_unchanged` (regression: all existing classic-pcap reader tests pass after probe insertion), `test_BC_2_01_009_nanosecond_pcap_routing`

### AC-004 (traces to BC-2.01.009 postcondition 4 — unrecognized magic returns Err)
When the first 4 bytes match neither format, returns `Err` with context indicating the
unrecognized magic. A 4-byte stream of `[0xDE, 0xAD, 0xBE, 0xEF]` returns `Err` containing
"unrecognized pcap magic" or equivalent.

**Test:** `test_BC_2_01_009_unrecognized_magic`, `test_BC_2_01_009_stream_under_4_bytes`

### AC-005 (traces to BC-2.01.010 postcondition 1 — BOM detection, canonical table)
After routing to the pcapng path, the SHB Byte-Order Magic field (body bytes 0-3) is read
as four raw on-disk bytes and compared against the canonical BOM table in BC-2.01.010 PC1:
- On-disk `1A 2B 3C 4D` → big-endian section
- On-disk `4D 3C 2B 1A` → little-endian section
- Any other pattern → `Err` mapping to E-INP-008

The established section endianness MUST be stored and propagated to ALL downstream block
decoders (IDB, EPB, SPB) — they MUST NOT re-detect endianness per-block (BC-2.01.010 Invariant 4).

**Test:** `test_BC_2_01_010_bom_little_endian`, `test_BC_2_01_010_bom_big_endian`

### AC-006 (traces to BC-2.01.010 postcondition 2 — version validation)
The pcapng major version MUST be 1; any other value returns `Err` immediately.
The minor version MAY be any value >= 0.

**Test:** `test_BC_2_01_010_major_version_not_1_rejected`

### AC-007 (traces to BC-2.01.010 postcondition 5 — SHB error routing 4-way split)
SHB errors are routed according to the uniform 4-way split from ADR-009 Decision 20:
- (a) btl < 12 / btl % 4 != 0 / EOF before trailer → crate rejects → E-INP-010
- (b) 12 ≤ btl < 28 (body 0-15 bytes) → crate returns block; wirerust body-decode finds
  body < 16 SHB fixed-field bytes → E-INP-008 (constructible fixture: btl=16 → body=4 bytes)
- (c) btl ≥ 28, body ≥ 16, invalid BOM or major_version ≠ 1 → E-INP-008
- (d) Well-formed → parse proceeds

**Test:** `test_BC_2_01_010_shb_body_truncated_e_inp_008` (btl=16 → body=4 < 16 → E-INP-008),
`test_BC_2_01_010_shb_btl8_maps_to_e_inp_008` (btl=8 → PcapNgParser::new InvalidField → E-INP-008 per ADR-009 Decision 23),
`test_BC_2_01_010_invalid_bom_e_inp_008`

### AC-008 (traces to BC-2.01.010 AC-002 — multi-section rejection)
A second Section Header Block encountered anywhere after the first is REJECTED with `Err`
mapping to E-INP-012. The rejection fires before any byte-order reset. The error message
includes a remediation hint directing to `mergecap -w out.pcapng <file>` or `editcap`.

**Test:** `test_BC_2_01_010_second_shb_rejected_e_inp_012`

### AC-009 (traces to BC-2.01.010 AC-005 — no-panic, SEC-005)
The SHB parse path MUST return `Err` for any malformed or truncated SHB byte sequence.
`unwrap()`, `expect()`, `panic!()`, and `unreachable!()` are prohibited in the SHB parse path.

**Test:** `test_BC_2_01_010_no_panic_fuzz` (property test over arbitrary SHB bytes)

### AC-010 (traces to BC-2.01.009 postcondition 6 — zero-packet notice on SHB-only file)
An SHB-only pcapng file (no IDB, no subsequent blocks) is structurally valid and yields
`Ok(PcapSource)` with `packets.len() == 0`, `skipped_blocks == 0`, `opb_skipped == 0`.
`from_pcap_reader` does NOT emit to stderr; `main.rs` emits the zero-packet notice.

**Test:** `test_BC_2_01_009_shb_only_zero_packet_notice`

### AC-011 (traces to BC-2.01.009 invariant 4 — pcapng SHB magic is endian-independent)
The pcapng SHB magic `0x0A0D0D0A` is endian-independent as a 4-byte literal; detection
does not depend on byte order. Both `from_file` and `from_pcap_reader` route through the
same probe; it is not duplicated.

**Test:** covered by `test_BC_2_01_009_smb3_pcapng_accepted` (LE pcapng) and `test_BC_2_01_010_bom_big_endian` (BE SHB)

### AC-012 (traces to BC-2.01.009 postcondition 5 — arp-baseline-16pkt.cap accepted)
The `arp-baseline-16pkt.cap` fixture (pcapng with `.cap` extension) MUST return
`Ok(PcapSource)` with 16 packets via the pcapng parse path.

**Test:** `test_BC_2_01_009_arp_baseline_cap_accepted` — assert `packets.len() == 16`

## Behavioral Contracts Table

| BC | Version | Clauses Covered |
|----|---------|-----------------|
| BC-2.01.009 | v1.7 | PC3 (probe consumes no bytes), PC1 (pcapng routing), PC2 (classic routing), PC4 (unrecognized magic), PC6 (zero-packet notice emission via main.rs), PC5 (smb3/arp-baseline.cap fixtures), Inv1 (4-byte peek), Inv3 (probe not duplicated), Inv4 (SHB magic endian-independent), AC-007 (BufReader wrap-site) |
| BC-2.01.010 | v2.2 | PC1 (canonical BOM table + section-wide endianness authority), PC2 (major version), PC3 (section_length ignored), AC-002 (multi-section rejection → E-INP-012), PC5 (4-way error routing, Decision 23 first-SHB btl-degenerate → E-INP-008), Inv4 (section-wide endianness scope) |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `PcapSource::from_pcap_reader` magic-byte probe | `src/reader.rs` | Effectful shell (I/O: BufReader peek) |
| `PcapSource::from_pcap_reader` classic-pcap branch | `src/reader.rs` | Effectful shell (unchanged path) |
| SHB body decode (BOM, version, section_length) | `src/reader.rs` (pcapng_pure_core fns) | Pure core (byte slice decode) |
| Section endianness stored for downstream decoders | `src/reader.rs` (internal state) | Pure state (no I/O) |

Architecture section references: `architecture/module-decomposition.md` (SS-01 C-4,
`src/reader.rs`); ADR-009 Decision 1 (raw-block path), Decision 5 (magic-byte probe discipline),
Decision 8 (forward-progress contract).

## Forbidden Dependencies

- The pcapng probe insertion MUST NOT import any pcap-file high-level typed API
  (`EnhancedPacketBlock`, `Block`, etc.) in this story — the raw-block path is the ONLY
  correct path (ADR-009 Decision 1). If any import of `EnhancedPacketBlock` appears in
  the diff, the build MUST fail review.
- `src/reader.rs` MUST NOT gain a dependency on any new crate. +0 new crates is the
  explicit contract per ADR-009 Decision 1 (Option A).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `smb3.pcapng` (previously a rejection fixture) | `Ok(PcapSource)` with correct packet count |
| EC-002 | `arp-baseline-16pkt.cap` (pcapng with `.cap` extension) | `Ok(PcapSource)` with 16 packets |
| EC-003 | Stream under 4 bytes | `Err` (short-read) |
| EC-004 | 4-byte `[0xDE, 0xAD, 0xBE, 0xEF]` | `Err` with unrecognized magic |
| EC-005 | Non-seekable pipe stream | Probe uses `BufReader::fill_buf()` (peek, no seek); works correctly |
| EC-006 | Classic ns-resolution pcap (`0xA1B23C4D`) | Routed to classic-pcap path |
| EC-007 | Unbuffered `Cursor<&[u8]>` with valid pcapng SHB | `Ok(PcapSource)` with correct routing (AC-007) |
| EC-008 | SHB with btl=16 (body=4 bytes, < 16 SHB fixed-field bytes) | `Err` mapping to E-INP-008 |
| EC-009 | SHB with btl=8 (first-SHB btl-degenerate; PcapNgParser::new → InvalidField("invalid magic number")) | `Err` mapping to E-INP-008 per ADR-009 Decision 23 |
| EC-010 | SHB with invalid BOM on-disk bytes | `Err` mapping to E-INP-008 |
| EC-011 | Crafted 2-section pcapng (SHB1 + IDB + EPB + SHB2) | `Err` mapping to E-INP-012 |
| EC-012 | SHB-only pcapng (no IDB, no subsequent blocks) | `Ok(PcapSource)` with `packets.len()==0`, `skipped_blocks==0`, `opb_skipped==0` |

## Tasks

1. Insert magic-byte probe at the top of `PcapSource::from_pcap_reader`:
   wrap `R: Read` in `BufReader`, call `fill_buf()`, read 4 bytes from buffer
   without consuming, branch on magic bytes.
2. Implement SHB body decode (pure-core helper): read BOM bytes, compare to canonical
   BOM table, determine endianness; parse major/minor version; parse section_length
   (accept any value); store endianness for downstream block decoders.
3. Implement second-SHB rejection (E-INP-012) with mergecap hint.
4. Update `src/reader.rs:5` module doc to reflect that pcapng is now supported.
5. Write unit tests covering all ACs above. Run `cargo test --all-targets` to verify
   all existing classic-pcap tests remain green.
6. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.

## Test Plan

| AC | Test | Type |
|----|------|------|
| AC-001 | `test_BC_2_01_009_unbuffered_read_routes_correctly`, `test_BC_2_01_009_pipe_stream_probe_observable` | Unit |
| AC-002 | `test_BC_2_01_009_smb3_pcapng_accepted` | Integration |
| AC-003 | Existing classic-pcap test suite (regression) | Regression |
| AC-004 | `test_BC_2_01_009_unrecognized_magic`, `test_BC_2_01_009_stream_under_4_bytes` | Unit |
| AC-005 | `test_BC_2_01_010_bom_little_endian`, `test_BC_2_01_010_bom_big_endian` | Unit |
| AC-006 | `test_BC_2_01_010_major_version_not_1_rejected` | Unit |
| AC-007 | `test_BC_2_01_010_shb_body_truncated_e_inp_008`, `test_BC_2_01_010_shb_btl8_maps_to_e_inp_008`, `test_BC_2_01_010_invalid_bom_e_inp_008` | Unit |
| AC-008 | `test_BC_2_01_010_second_shb_rejected_e_inp_012` | Unit |
| AC-009 | `test_BC_2_01_010_no_panic_fuzz` | Property |
| AC-010 | `test_BC_2_01_009_shb_only_zero_packet_notice` | Integration |
| AC-012 | `test_BC_2_01_009_arp_baseline_cap_accepted` | Integration |

## Previous Story Intelligence

N/A — first story in E-19 (pcapng reader support epic). Prior art: BC-2.01.004 was the
normative statement of pcapng rejection. It was RETIRED during F2 spec evolution and
superseded by BC-2.01.009. The test `test_BC_2_01_004_rejects_pcapng` will be rewritten
from a negative-assertion test into a positive-acceptance test. Ensure that rename is
part of this story's PR.

## Architecture Compliance Rules

Derived from ADR-009 rev 9 and BC-2.01.009/010:

1. **Probe is PEEK-ONLY** — `BufReader::fill_buf()` then read 4 bytes from the returned
   `&[u8]` slice WITHOUT calling `consume()`. The stream is not advanced after the peek.
   Implementing `consume(4)` would break every file; this is a HIGH-severity correctness rule.
2. **Raw-block path only** — wirerust MUST use `RawBlock` / `next_raw_block`, NOT
   `Block::EnhancedPacket` / `EnhancedPacketBlock`. The crate's high-level API hard-codes
   nanosecond resolution and is WRONG for the common case (ADR-009 Decision 1 rev 4).
3. **+0 new crates** — `pcap-file` 2.0.0 is already in `Cargo.toml`. No new dependency.
4. **Section-wide endianness** — endianness determined once by SHB BOM, stored, propagated
   to all downstream decoders. Downstream decoders MUST NOT re-detect per-block (BC-2.01.010 Inv 4).
5. **SHB fixed-field minimum = 16 body bytes** — check `body.len() >= 16` before decoding
   BOM, major, minor, section_length. btl=16 → body=4 < 16 → E-INP-008 (NOT E-INP-010).
6. **Break on crate Err** — block-walk loop MUST break on `Err(_)` from `next_raw_block`.
   The crate does NOT advance its cursor on error; retrying after error spins (CWE-835).

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `pcap_file` | 2.0.0 | Already in Cargo.toml. Use `RawBlock` / `next_raw_block` API only. |
| `anyhow` | existing | Use `.context(...)` chaining for all pcapng errors |
| `std::io::BufReader` | stdlib | Wrap `R: Read` internally; `fill_buf()` for peek |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `src/reader.rs` | Modify | Insert magic-byte probe in `from_pcap_reader`; add SHB parse helpers |
| `src/reader.rs:5` | Modify | Update module doc: pcapng is now supported; remove "LESSON-P0.02" reference |
| `tests/reader_tests.rs` (or equivalent) | Modify | Add pcapng probe tests; rewrite `test_BC_2_01_004_rejects_pcapng` |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file) | ~5,000 |
| BC files (2 BCs: BC-2.01.009 v1.7, BC-2.01.010 v2.1) | ~10,000 |
| ADR-009 rev 9 (canonical constants + relevant decisions) | ~4,000 |
| `src/reader.rs` (current implementation) | ~4,000 |
| Test files (new + existing) | ~3,000 |
| Tool outputs (cargo test, clippy) | ~1,000 |
| **Total estimated** | **~27,000** |

Within 20-30% of agent context window.

## Dependency Rationale

- `depends_on: []` — STORY-123 has no predecessors; it is the foundation of the pcapng
  reader stack. The magic-byte probe and SHB parser must exist before any downstream block
  (IDB, EPB, SPB) can be parsed.
- `blocks: [STORY-124, STORY-125, STORY-126, STORY-127]` — All four downstream stories
  depend on the pcapng routing infrastructure and SHB endianness established here. None of
  them can be implemented without the probe and SHB in place.

## Cross-Story Deferred Findings (from STORY-123 adversarial review)

The following findings surfaced during the STORY-123 Wave-51 adversarial pass-1 review
(D-169) but are out of scope for STORY-123. They are recorded here for traceability and
mirrored in STATE.md Drift Items.

### F-2 [cross-story → STORY-125 / BC-2.01.012]

EPB padding-overrun check (ADR-009 Decision 20/22: verify
`20 + captured_len + pad_len(captured_len) <= body.len()` → E-INP-008) is NOT implemented
or tested in STORY-123. STORY-123 only checks `captured_len > available`. STORY-125 MUST
add a `captured_len % 4 != 0` fixture that exercises the padding-overrun path and confirms
E-INP-008 is returned.

**Owner:** STORY-125 (BC-2.01.012). Status: DEFERRED.

### F-3 [cross-story → STORY-125 / BC-2.01.014]

Timestamp decode in `read_pcapng_crate` hardcodes `DEFAULT_TSRESOL=6` (microsecond), does
NOT walk IDB `if_tsresol` options, and inlines the math instead of calling the BC-2.01.014
pure-core helper. This produces timestamps that are wrong by 1000x for `if_tsresol=9`
(nanosecond) captures. STORY-125 MUST implement the `if_tsresol` option-walk, call the
BC-2.01.014 pure-core helper for the actual scaling, and deliver the VP-025 Kani proof
target covering the full `u8` `if_tsresol` space including the base-2 branch.

**Owner:** STORY-125 (BC-2.01.014). Status: DEFERRED.

### F-5 [system-level → Phase-4]

AC-012 (`test_BC_2_01_009_arp_baseline_cap_accepted`) uses a SYNTHETIC `arp-baseline-16pkt.cap`
fixture created via `temp_dir`. The authentic PacketLife-sourced capture MUST replace this
synthetic fixture before the Phase-4 holdout evaluation. Tracked under DF-VALIDATION-001
policy.

**Owner:** Phase-4 entry gate. Status: DEFERRED — pre-Phase-4 obligation.

### F-7 [observation → Phase-6]

`parse_shb_body` (the VP-026 Kani proof target) is NOT on the live integration path — the
`pcap-file` crate itself parses the SHB in the `PcapNgParser::new` call; wirerust's
`parse_shb_body` is a pure helper that is NOT invoked in the actual execution path. VP-026's
Kani target function may need re-scoping in Phase-6 to cover the actual reachable code path
rather than the dead pure helper.

**Owner:** Phase-6 (formal hardening). Status: OBSERVATION — evaluate at Phase-6 entry.
