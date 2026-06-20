---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: F2
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-01
capability: CAP-01
lifecycle_status: active
introduced: v0.10.0-pcapng
modified:
  - "v1.1: F-07 completeness delta — explicitly enumerate all pcap-file Block variants that fall through to the skip path (NRB, ISB, DSB, SystemdJournalExport, obsolete Packet Block 0x2, Unknown); note that obsolete Packet Block 0x2 carries packet data but is treated as out-of-scope/skipped; add AC to prevent omitted match arm at implementation. — 2026-06-19"
  - "v1.2: F2 Burst-A remediation per ADR-009 rev 4 PO dispatch — (1) VP-029 added to Verification Properties. (2) EC-004 CORRECTED: `block_total_length = 8` is REJECTED by the crate (`block_common.rs:101`: `< 12` threshold), NOT '0 bytes consumed; no error'. Removed the false 'no error' claim; replaced with crate-accurate reject behavior. (3) EC-005 updated accordingly (threshold is now < 12, not < 8). (4) Added forward-progress invariant: block-walk loop MUST break on Err(_); the crate cursor does NOT advance on error (`read_buffer.rs:65`). (5) On the raw-block path, skip means ignoring the RawBlock body bytes (already in the body slice); the loop-break-on-error invariant covers all block types. (6) DSB log-guard note (SEC-007) added to AC-002: block body bytes MUST NOT be logged at any level. (7) Corrected AC-001 entry for DSB: DSB (type 0x0A) is NOT a named pcap-file Block variant — it arrives as Block::Unknown on the high-level API; on the raw-block path, block-type dispatch reads bytes 0-3 of each RawBlock body — no named DSB arm exists. (8) Added no-panic AC (SEC-005). — 2026-06-19"
  - "v1.5: Pass-4 remediation R3a (ADR-009 rev 7) — (Decision 19 / M-4) Fixed PC9 citation: 'Decision 17' corrected to 'Decision 19' in Postcondition 9 cross-reference. The skipped_blocks counter and its pass-to-caller semantics are unchanged. BC-2.01.009 owns the emission gate ('valid file + zero packets', Decision 19); BC-2.01.015 owns the counter. — 2026-06-20"
  - "v1.4: Pass-3 remediation Burst Q3 (ADR-009 rev 6) — (M-3) PC9 clarified: the emission trigger now lives in BC-2.01.009 as 'valid file + zero packets' (not 'zero packets AND skipped_blocks>0'). BC-2.01.015 retains the skipped_blocks counter and passes it to BC-2.01.009; BC-2.01.009 decides whether to include the count in the notice message. AC-006 updated to reflect that the counter is reported in the notice message when >0 but is NOT the gating condition. — 2026-06-19"
  - "v1.3: Pass-2 remediation per ADR-009 rev 5 (C-3, I-3, I-11) — (C-3) Canonical Test Vector body-byte count corrected: block_total_length=20 has 20-12=8 body bytes (not 12; pcapng frame overhead is 12 bytes: type:4 + total_len:4 + trailing_total_len:4). Test vector updated accordingly. (I-3) Added skipped_blocks counter concept and cross-reference to BC-2.01.009 for the one-shot stderr notice: when packet-bearing blocks are skipped and packet count reaches zero on a non-empty file, BC-2.01.009 emits a one-shot E-INP-007-style notice with the count of skipped blocks (no body bytes logged — SEC-007). BC-2.01.015 owns the counter; emission is BC-2.01.009's responsibility. Added Postcondition 9 and AC-006. (I-11) Added Test: citations to all ACs. — 2026-06-19"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.015: Unknown pcapng Block Types Are Silently Skipped via block-total-length

## Description

The pcapng specification allows any block type, with the guarantee that every block begins
with a 4-byte block-type field followed by a 4-byte block-total-length field. On the raw-block
path (ADR-009 Decision 1 rev 4), wirerust dispatches on the block-type bytes of each
`RawBlock`. Any block type not identified as SHB (`0x0A0D0D0A`), IDB (`0x00000001`),
EPB (`0x00000006`), or SPB (`0x00000003`) is silently skipped: the RawBlock body bytes
are discarded without logging. Neither a warning nor an error is emitted. Parse state —
interface table, byte order, packet list — is unchanged. The block-walk loop MUST break
on any `Err(_)` from the crate's block reader; the crate does NOT advance its cursor on
error, so retrying the same source after an error would spin (CWE-835).

## Preconditions

1. The SHB has been parsed; byte order is established.
2. The current stream position is at the start of a block (crate handles framing).
3. The block-type value is not one of the four known types (SHB/IDB/EPB/SPB).
4. `block_total_length` is parseable by the crate framing layer (crate rejects
   `block_total_length < 12` with `Err` before returning any block to the caller).

## Postconditions

1. On the raw-block path: the `RawBlock` body bytes are discarded (already available in the
   slice; no additional read needed). The crate's forward-progress guarantee ensures the
   cursor has advanced past the full block extent (`block_total_length` bytes) before the
   block is returned to the caller.
2. No error is returned.
3. No warning is emitted to stderr.
4. No warning or log entry is emitted at ANY log level. Block body bytes MUST NOT appear in
   any diagnostic output regardless of severity (SEC-007 — DSB carries TLS key material).
5. No packet is added to `PcapSource.packets`.
6. The interface table is unchanged.
7. Parsing continues with the next block.
8. If the crate returns `Err(_)` for any block (including a malformed unknown block), the
   block-walk loop MUST break immediately. The loop MUST NOT retry the same source after
   an error.
9. BC-2.01.015 maintains a `skipped_blocks: u64` counter incremented once per skipped block
   (any block falling through to the skip arm). This counter is passed to the caller context
   at end-of-file. **The emission trigger is owned by BC-2.01.009 and is "valid file + zero
   packets"** (M-3 / ADR-009 rev 7 Decision 19): BC-2.01.009 emits the one-shot stderr
   notice whenever the pcapng file is structurally valid and the packet list is empty —
   regardless of whether `skipped_blocks > 0`. The `skipped_blocks` count from this counter
   is included in the notice message when it is greater than zero; it is NOT the gating
   condition for notice emission. Block body bytes MUST NOT appear in this notice (SEC-007).
   This cross-reference is bidirectional: BC-2.01.015 owns the counter; BC-2.01.009 owns
   the emission decision and message format.

## Acceptance Criteria

- **AC-001 (raw-block path dispatch):** On the raw-block path, block-type identification is
  done by reading the first 4 bytes of each `RawBlock` body. The dispatch MUST cover:
  - SHB (`0x0A0D0D0A`) — handled by SHB parse arm.
  - IDB (`0x00000001`) — handled by IDB parse arm.
  - EPB (`0x00000006`) — handled by EPB parse arm.
  - SPB (`0x00000003`) — handled by SPB parse arm.
  - **ALL other block-type values** — silently skipped (RawBlock body discarded). This
    includes: NRB (`0x00000004`), ISB (`0x00000005`), OPB (`0x00000002`), SJE (`0x00000009`),
    DSB (`0x0000000A`), and any future/unknown block types. **IMPORTANT:** DSB is NOT a
    named variant in `pcap_file::pcapng::Block` — there is no `DecryptionSecrets` enum arm.
    On the raw-block path, DSB arrives as a `RawBlock` with block-type bytes `0x0000000A` and
    is handled by the skip arm. Do NOT attempt to name-match on a DSB enum variant.
  **Test:** `test_BC_2_01_015_dispatch_known_and_skip_unknown`
- **AC-002 (no output at any log level, SEC-007):** For each block type on the skip path, the
  skip MUST NOT emit any warning, error, finding, or log entry at any log level (trace, debug,
  info, warn, error). Block body bytes MUST NOT be logged, printed, or included in any
  diagnostic output. DSB (type `0x0000000A`) carries TLS key material; logging its bytes
  would be a security violation.
  **Test:** `test_BC_2_01_015_no_output_on_skip`
- **AC-003 (OPB is skipped, not parsed):** OPB (type `0x00000002`) carries packet data but is
  an obsolete/deprecated block superseded by EPB. wirerust treats it as out-of-scope and skips
  it silently. OPB packet data is intentionally NOT ingested. Captures relying solely on OPB
  will yield zero packets from those blocks.
  **Test:** `test_BC_2_01_015_opb_skipped_not_parsed`
- **AC-004 (forward-progress / loop-break-on-error):** The block-walk loop MUST `break` (or
  return) on any `Err(_)` from the crate's block reader. The documented rustdoc example
  with an empty `Err(_) => {}` arm is WRONG and MUST NOT be copied — it would spin on the
  same error indefinitely because the crate does NOT advance the cursor on error
  (`read_buffer.rs:65`).
  **Test:** `test_BC_2_01_015_loop_break_on_error`
- **AC-005 (no-panic, SEC-005):** The skip path and the block-walk loop MUST NOT contain
  `unwrap()`, `expect()`, or `panic!()` reachable from the skip arm.
  **Test:** `test_BC_2_01_015_no_panic_skip_path`
- **AC-006 (skipped_blocks counter, cross-ref BC-2.01.009):**
  The block-walk loop MUST maintain a `skipped_blocks: u64` counter, incrementing it once per
  block entering the skip arm. At end-of-file, the counter value is passed to BC-2.01.009's
  notice-emission logic. **The gating condition for emission is "valid file + zero packets"
  (BC-2.01.009 PC6), not "skipped_blocks > 0"**: BC-2.01.009 will emit the one-shot notice
  even when `skipped_blocks == 0` (e.g., an IDB-only file with no packet blocks and no
  skipped blocks). When `skipped_blocks > 0`, the count is included in the notice message;
  when `skipped_blocks == 0`, the notice is emitted without a skip-count segment.
  The notice MUST NOT include block body bytes (SEC-007). The counter itself MUST NOT overflow
  (use saturating_add or u64 — realistic file sizes cannot approach u64::MAX blocks).
  **Test:** `test_BC_2_01_015_skipped_blocks_counter_and_notice`

## Invariants

1. The skip is performed using the crate's block framing layer: the crate returns the
   `RawBlock` with body bytes already bounded by `block_total_length`; discarding the body
   is the skip. No hand-rolled length arithmetic is needed for skip.
2. The block-walk loop MUST break on `Err(_)`. The crate's cursor does NOT advance on
   error; breaking is the caller's only obligation.
3. The skip MUST NOT emit any diagnostic to stderr, stdout, or any log sink at any level.
4. All four known block types (SHB, IDB, EPB, SPB) MUST be handled by their own parsing
   branches; they MUST NOT fall through to the skip path.
5. The crate OWNS forward progress: `block_total_length < 12` is rejected by the crate
   before the block is returned (`block_common.rs:101-103`). wirerust does not need to
   hand-check a minimum block size for the skip path — the crate enforces it.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Interface Statistics Block (ISB, type `0x00000005`) | Silently skipped; no warning; no packet added |
| EC-002 | Obsolete Packet Block (OPB, type `0x00000002`) | Silently skipped; packet data in OPB NOT ingested; EPB parsing unaffected |
| EC-003 | Block with future type value (e.g. `0x00000007`) | Silently skipped |
| EC-004 | `block_total_length = 8` | REJECTED by the crate (`block_common.rs:101`: threshold is `< 12`). The crate returns `Err(InvalidField("Block: initial_len < 12"))` before handing any block to the caller. The block-walk loop receives `Err(_)` and MUST break. The prior characterization "0 bytes consumed; no error" was INCORRECT — removed. |
| EC-005 | `block_total_length < 12` (any value below crate threshold) | REJECTED by crate with `Err`; caller breaks on Err. |
| EC-006 | Stream truncated mid-skip | Crate returns `Err` (trailer-length mismatch or EOF); caller breaks on Err |
| EC-007 | Multiple consecutive unknown blocks | Each handled as a separate `Ok(RawBlock)` → skip; loop continues |
| EC-008 | Name Resolution Block (NRB, type `0x00000004`) | Silently skipped; name resolution data NOT ingested; no warning |
| EC-009 | Decryption Secrets Block (DSB, type `0x0000000A`) | Silently skipped; TLS key material in body NOT logged, printed, or included in any output at any level (SEC-007) |
| EC-010 | Systemd Journal Export Block (type `0x00000009`) | Silently skipped; journal data NOT ingested; no warning |
| EC-011 | pcapng file containing OPB blocks before and after EPBs | OPBs silently skipped; only EPBs produce packets; packet list is EPB-derived only |
| EC-012 | DSB immediately followed by EPB | DSB skipped silently (body not logged); EPB parsed normally; packet produced |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcapng file with ISB before final EPB | ISB skipped silently; final EPB produces `RawPacket` | happy-path |
| Block with type `0xDEADBEEF`, `block_total_length=20` | 8 body bytes discarded (20 - 12 frame overhead = 8; overhead: type:4 + total_len:4 + trailing_total_len:4), no error, no packet | edge-case |
| Block with `block_total_length=8` | `Err` returned by crate (rejected at `< 12` threshold); loop breaks | error (EC-004) |
| Unknown block followed by EPB | Unknown block skipped; EPB parsed normally | happy-path |
| DSB (type `0x0000000A`) followed by EPB | DSB body discarded without logging; EPB produces RawPacket | edge-case (EC-012) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-029 | Block-walk skip correctness: unknown-block skip always advances past `block_total_length` bytes (crate guarantee); no infinite loop; loop terminates for any valid/malformed block sequence (Err arm breaks) | proptest: generate arbitrary block sequences including malformed lengths; assert loop terminates and result is Ok or Err (not spin) |
| — | ISB does not produce a packet or an error | unit: pcapng file with ISB; assert no error, packets unchanged |
| — | No stderr/stdout output on unknown block | unit: capture stderr + stdout during parse of unknown block; assert both empty |
| — | Truncated unknown block returns Err, not Ok | unit: craft block with `block_total_length < 12`; assert Err; assert loop breaks |
| — | DSB body bytes absent from all output | unit: DSB with synthetic key material; assert no log/stderr/stdout contains those bytes |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- block-type skip is a robustness property of the ingestion pipeline; the ability to traverse unknown blocks is required to successfully read all packets from a well-formed pcapng file that contains optional blocks |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-126 |
| ADR Reference | ADR-009 rev 4 Decision 2 (unknown block types silently skipped; DSB has no named variant — arrives as Unknown), Decision 8 (forward-progress: crate rejects `< 12`; caller MUST break on Err), Decision 10 (panic surface), Decision 14 (VP-029 proptest) |

## Related BCs

- BC-2.01.012 -- related (EPB is a known block; must NOT fall to skip path)
- BC-2.01.013 -- related (SPB is a known block; must NOT fall to skip path)
- BC-2.01.017 -- related (block-level errors map to E-INP-008/010; skip path never errors for crate-valid unknown blocks)

## Architecture Anchors

- ADR-009 rev 4 Decision 8: `block_common.rs:101-103` rejects `block_total_length < 12` before returning any block; `read_buffer.rs:65` confirms cursor does not advance on Err; caller MUST break on Err
- ADR-009 rev 4 Decision 2: DSB (`0x0000000A`) is NOT a named `Block` enum variant — it arrives as `Block::Unknown` on high-level API (`block_common.rs:217-251`); on raw-block path, wirerust reads block-type bytes directly
- `block_common.rs:146-166` (pcap-file 2.0.0 source): 9-variant Block enum; NO `DecryptionSecrets` variant; DSB → Unknown
- pcapng spec IETF draft §General-Block-Structure: every block has 4-byte type + 4-byte total-length; minimum = 12 bytes (8-byte header + 4-byte trailer)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads (and discards) RawBlock body bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O only) |
