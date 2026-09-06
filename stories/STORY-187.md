---
document_type: story
level: ops
story_id: STORY-187
title: "S7comm Flow State Completion, Four-Way protocol_id Dispatch Skeleton, and parse_s7comm_header Pure-Core Parser"
epic_id: E-23
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 8
priority: P1
cycle: feature-s7comm
wave: 90
target_module: analyzer/s7comm
subsystems: [SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-186]
blocks: [STORY-188]
behavioral_contracts: [BC-2.21.001, BC-2.21.002, BC-2.21.004, BC-2.21.005, BC-2.21.006, BC-2.21.007, BC-2.21.008, BC-2.21.009]
verification_properties: [VP-051, VP-053]
inputs:
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.001.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.002.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.004.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.005.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.006.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.007.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.008.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.009.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
input-hash: "ffc51b4"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-187: S7comm Flow State Completion, Four-Way protocol_id Dispatch Skeleton, and parse_s7comm_header Pure-Core Parser

## Narrative

**As a** security analyst using wirerust to inspect classic S7comm traffic,
**I want** `S7commFlowState` completed with classification state, `S7commAnalyzer` to
branch on the extracted COTP `protocol_id` byte, and a bounds-safe pure-core parser for
the classic S7comm (`0x32`) common header,
**so that** classic S7comm PDUs (ROSCTR, PDU reference, parameter/data length) are
correctly and safely parsed as the foundation for function-code classification
(STORY-188/189) and MITRE technique emission (STORY-191/192).

This story completes the `S7commFlowState` struct definition (fields not yet needed by
STORY-186's frame-extraction-only scope) and extends `S7commAnalyzer::on_data` with the
four-way dispatch on `CotpHeader::protocol_id`. The `Some(0x32)` (classic) branch is
fully wired to `parse_s7comm_header`; the `Some(0x72)` (S7comm-plus) and
unclassified/unrecognized branches are completed structurally in STORY-190 — this story
routes them to a placeholder no-op so the four-way match is total and compiles, without
yet implementing their observable behavior.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.21.001 | `S7commFlowState` Owns TPKT/COTP Carry Buffers, S7comm Classification State, and Per-Direction Dedup Flags | Completes the flow-state struct started in STORY-186 |
| BC-2.21.002 | `S7commAnalyzer::on_data` Four-Way Dispatch on `CotpHeader::protocol_id` | Dispatch skeleton (classic branch fully wired; plus/unclassified branches placeholder) |
| BC-2.21.004 | `parse_s7comm_header` Returns None for Input Shorter Than 10 Bytes | Reject path: length < 10 |
| BC-2.21.005 | `parse_s7comm_header` Defensively Rejects `data[0] != 0x32` | Defense-in-depth reject path |
| BC-2.21.006 | `parse_s7comm_header` Extracts Common Header Fields (Happy Path) | Accept path for Job/AckData/Userdata ROSCTR |
| BC-2.21.007 | `parse_s7comm_header` Returns None for an Unrecognized ROSCTR Byte | Safe-reject, no force-fit |
| BC-2.21.008 | `parse_s7comm_header` for ROSCTR=Ack Requires 12 Bytes | Ack-specific header extension |
| BC-2.21.009 | Declared param_length/data_length Bounds-Checked Before Slice Access | Safe-reject on inconsistency |

## Acceptance Criteria

### AC-187-001: `S7commFlowState` carries the full field set required by this story's scope
(traces to BC-2.21.001 postcondition 1)
- Given `S7commFlowState` after this story
- When its fields are inspected
- Then, in addition to the carry fields from STORY-186, it now also carries:
  `session_established: bool`, `classified_protocol: Option<S7Protocol>`,
  `malformed_header_reported_c2s: bool`, `malformed_header_reported_s2c: bool`
- No field on `S7commFlowState` duplicates a field SS-20 owns (traces to BC-2.21.001
  postcondition 2)
- **Test:** `test_BC_2_21_001_flow_state_field_set`

### AC-187-002: `S7commFlowState` is created lazily on first `on_data` call
(traces to BC-2.21.001 postcondition 3)
- Given a newly classified flow with no prior `on_data` call
- When `on_data` is called for the first time
- Then `S7commFlowState` is created and stored in the analyzer's per-flow map, keyed by
  `FlowKey`
- **Test:** `test_BC_2_21_001_lazy_flow_state_creation`

### AC-187-003: CR/CC frames update session_established and defer classification
(traces to BC-2.21.002 postcondition 2)
- Given `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: ConnectRequest |
  ConnectConfirm, .. })`
- When `on_data` dispatches this frame
- Then `S7commFlowState.session_established` is updated per the CR/CC observation; no
  protocol classification occurs — classification is deferred to the first DT frame
- **Test:** `test_BC_2_21_002_cr_cc_updates_session_no_classification`

### AC-187-004: `Some(0x32)` DT frames dispatch to classic S7comm dissection
(traces to BC-2.21.002 postcondition 3)
- Given `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer,
  protocol_id: Some(0x32), .. })`
- When `on_data` dispatches this frame
- Then `parse_s7comm_header` is called on the slice beginning at `payload_offset`
- **Test:** `test_BC_2_21_002_classic_s7comm_dispatch`

### AC-187-005: First DT frame sets classified_protocol exactly once (sticky first-classification-wins)
(traces to BC-2.21.002 postcondition 6)
- Given a flow's first DT frame (any `protocol_id` value, including `None`)
- When `on_data` processes it
- Then `S7commFlowState.classified_protocol` is set exactly once; subsequent DT frames on
  the same flow do not overwrite it even if their `protocol_id` differs
- **Test:** `test_BC_2_21_002_sticky_first_classification`

### AC-187-006: `parse_s7comm_header` returns None for input shorter than 10 bytes
(traces to BC-2.21.004 postcondition 1)
- Given `data.len() < 10`
- When `parse_s7comm_header(data)` is called
- Then returns `None`; no bytes beyond the length check are accessed; no panic for any
  `data.len()` in `[0, 9]` (traces to BC-2.21.004 postcondition 2)
- The first occurrence per flow direction emits one T0814
  (`ThreatCategory::Anomaly`/`Verdict::Possible`/`Confidence::Medium`) via
  `malformed_header_reported_c2s`/`_s2c` (traces to BC-2.21.004 postcondition 4)
- **Test:** `test_BC_2_21_004_len_shorter_than_10_returns_none_and_emits_t0814_once`

### AC-187-007: `parse_s7comm_header` defensively rejects `data[0] != 0x32`
(traces to BC-2.21.005 postcondition 1)
- Given `data.len() >= 10` and `data[0] != 0x32`
- When `parse_s7comm_header(data)` is called
- Then returns `None`; no other bytes are accessed once `data[0]` fails the equality
  check; no `Finding` is emitted — this is a defense-in-depth caller-hygiene contract,
  not a wire-observable anomaly (traces to BC-2.21.005 postcondition 3)
- **Test:** `test_BC_2_21_005_defensive_reject_wrong_protocol_id_byte`

### AC-187-008: `parse_s7comm_header` extracts common header fields for Job/AckData/Userdata
(traces to BC-2.21.006 postcondition 1)
- Given `data.len() >= 10`, `data[0] == 0x32`, `data[1] ∈ {0x01, 0x03, 0x07}`
- When `parse_s7comm_header(data)` is called
- Then returns `Some(S7commHeader { rosctr, pdu_reference, param_length, data_length,
  error_class: None, error_code: None, header_len: 10 })` with fields extracted exactly
  per the byte offsets: `pdu_reference = u16::from_be_bytes([data[4], data[5]])`
  (traces to BC-2.21.006 postcondition 2), `param_length = u16::from_be_bytes([data[6],
  data[7]])` (postcondition 3), `data_length = u16::from_be_bytes([data[8], data[9]])`
  (postcondition 4)
- `data[2..4]` (Reserved) is read for header-length bookkeeping only, never compared or
  branched on (traces to BC-2.21.006 postcondition 5)
- **Test:** `test_BC_2_21_006_common_header_field_extraction`

### AC-187-009: `parse_s7comm_header` returns None for an unrecognized ROSCTR byte
(traces to BC-2.21.007 postcondition 1)
- Given `data.len() >= 10`, `data[0] == 0x32`, `data[1] ∉ {0x01, 0x02, 0x03, 0x07}`
- When `parse_s7comm_header(data)` is called
- Then returns `None` for all 252 remaining `u8` values; no panic (traces to BC-2.21.007
  postcondition 2)
- Emits one T0814 via the same `malformed_header_reported_c2s`/`_s2c` dedup flag as
  AC-187-006 (traces to BC-2.21.007 postcondition 3 — this is the same dedup flag, not a
  second, distinct anomaly class)
- **Test:** `test_BC_2_21_007_unrecognized_rosctr_returns_none`

### AC-187-010: ROSCTR=Ack requires 12 bytes and extracts error_class/error_code
(traces to BC-2.21.008 postcondition 1)
- Given `data[1] == 0x02` (Ack) and `data.len() < 12`
- When `parse_s7comm_header(data)` is called
- Then returns `None` (malformed-header, shares the dedup flag with AC-187-006/009)
- Given `data.len() >= 12`
- Then returns `Some(S7commHeader { rosctr: Ack, error_class: Some(data[10]),
  error_code: Some(data[11]), header_len: 12, .. })` (traces to BC-2.21.008
  postcondition 2)
- `error_class`/`error_code` are `Some` only when `rosctr == Ack`; every other
  `S7commHeader` has both fields `None` (traces to BC-2.21.008 postcondition 3)
- **Test:** `test_BC_2_21_008_ack_rosctr_12_byte_minimum_and_error_fields`

### AC-187-011: Declared param_length/data_length are bounds-checked before any slice access
(traces to BC-2.21.009 postcondition 1)
- Given `parse_s7comm_header(data)` returned `Some(header)` and
  `data.len() < header.header_len + header.param_length as usize + header.data_length
  as usize`
- When `S7commAnalyzer` attempts to slice out the parameter/data blocks
- Then no slice into `data` beyond `data.len()` is ever attempted; the frame is treated
  as malformed (one T0814 per flow direction, sharing the
  `malformed_header_reported_c2s`/`_s2c` dedup flag — traces to BC-2.21.009
  postcondition 2)
- No function-code or Userdata classification is attempted for a frame that fails this
  check (traces to BC-2.21.009 postcondition 3)
- **Test:** `test_BC_2_21_009_bounds_check_before_parameter_data_slice`

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `S7commFlowState` (completed) | SS-21 per-flow state | `src/analyzer/s7comm.rs` | Mutable state |
| `S7Protocol` enum | SS-21 data model | `src/analyzer/s7comm.rs` | N/A (`Classic`, `Plus`, `Unclassified`; full population in STORY-190) |
| `S7commHeader` struct | SS-21 data model | `src/analyzer/s7comm.rs` | N/A (frozen per ADR-014 Decision 9 item 3) |
| `Rosctr` enum | SS-21 data model | `src/analyzer/s7comm.rs` | N/A (`Job`, `Ack`, `AckData`, `Userdata`) |
| `parse_s7comm_header` | SS-21 S7comm header parser | `src/analyzer/s7comm.rs` | Pure (free fn, VP-051 target) |
| `S7commAnalyzer::on_data` (dispatch extension) | SS-21 effectful shell | `src/analyzer/s7comm.rs` | Effectful |

Subsystem anchor: SS-21 owns this story's scope because `S7commFlowState`,
`parse_s7comm_header`, and the `protocol_id` dispatch skeleton are the core data model
and entry point of the S7comm analyzer per ARCH-INDEX.md §SS-21.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `parse_s7comm_header` | pure-core | Returns `Option<S7commHeader>` by value; no mutation, no I/O; defensively re-checks `data[0]` but performs no side effects |
| `S7commHeader`, `Rosctr`, `S7Protocol` | pure-core | Plain data types |
| `S7commAnalyzer::on_data` (dispatch extension) | effectful-shell | Mutates `S7commFlowState`, calls `parse_s7comm_header`, emits T0814 on malformed-header conditions |

## VP-051 Kani Obligation

**Harness:** `verify_parse_s7comm_header_bounds_safety` (anchored in this story)
**Method:** Kani symbolic execution
**Priority:** P0

Covers BC-2.21.004 (10-byte minimum) and BC-2.21.009 (the caller-side bounds obligation
`header_len + param_length + data_length` cannot overflow `usize`, and no slice beyond
`data.len()` is ever constructed). Skeleton written here; full proof in STORY-194.

## VP-053 Proptest Obligation (partial — completed in STORY-190)

**Harness:** `proptest_vp053_protocol_id_dispatch_totality` (skeleton started here)
**Method:** proptest
**Priority:** P0

This story wires the CR/CC and `Some(0x32)` branches of the four-way dispatch
(BC-2.21.002). The `Some(0x72)` and unclassified/unrecognized branches — required for
VP-053's full totality proof — are completed in STORY-190. The skeleton compiles here
against a `todo!()`/placeholder stub for the not-yet-implemented branches; the full
non-vacuous run is deferred to STORY-194.

## Tasks

- [ ] Extend `S7commFlowState` (from STORY-186) with: `session_established: bool`,
      `classified_protocol: Option<S7Protocol>`, `malformed_header_reported_c2s: bool`,
      `malformed_header_reported_s2c: bool`
- [ ] Define `pub enum S7Protocol { Classic, Plus, Unclassified }` (skeleton; `Plus` and
      `Unclassified` variants are fully driven starting in STORY-190)
- [ ] Define `pub enum Rosctr { Job, Ack, AckData, Userdata }`
- [ ] Define `pub struct S7commHeader { pub rosctr: Rosctr, pub pdu_reference: u16,
      pub param_length: u16, pub data_length: u16, pub error_class: Option<u8>,
      pub error_code: Option<u8>, pub header_len: usize }`
- [ ] Implement `pub fn parse_s7comm_header(data: &[u8]) -> Option<S7commHeader>`:
  - `data.len() < 10` guard -> `None` (BC-2.21.004)
  - `data[0] != 0x32` defensive guard -> `None` (BC-2.21.005)
  - ROSCTR match: `0x01`/`0x03`/`0x07` -> common-header extraction, `header_len: 10`
    (BC-2.21.006); `0x02` -> Ack extension requiring 12 bytes (BC-2.21.008); any other
    byte -> `None` (BC-2.21.007)
- [ ] Extend `S7commAnalyzer::on_data`'s frame dispatch (from STORY-186) with the
      four-way `CotpHeader::protocol_id` branch (BC-2.21.002): `None` (session
      TPDU/unparseable) routes to session tracking or a STORY-190 placeholder;
      `Some(0x32)` DT calls `parse_s7comm_header` and applies the bounds check
      (BC-2.21.009); `Some(0x72)` and `Some(other)` route to a `todo!()`-free
      placeholder no-op completed in STORY-190
- [ ] Implement the BC-2.21.009 caller-side bounds check before any parameter/data-block
      slice is constructed
- [ ] Write `#[cfg(kani)]` VP-051 skeleton
- [ ] Write `proptest_vp053_protocol_id_dispatch_totality` skeleton (partial, per the VP
      Obligation section above)
- [ ] Write unit tests: one per AC, named `test_BC_2_21_001_*` .. `test_BC_2_21_009_*`
- [ ] Verify `cargo test` passes for this story's tests
- [ ] Extend `tests/fixtures/mk_s7comm_pcap.py` (CREATE, first use in this story) with
      Setup Communication (`0xF0`) and a minimal classic-S7comm Job/Ack_Data frame pair,
      per ADR-014 Decision 7 — synthetic, CC0/MIT, mirrors `mk_modbus_pcap.py`
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the S7comm header
      parser and dispatch skeleton, before creating the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.21.001 | Flow receives zero bytes before close | `S7commFlowState` never created |
| EC-002 | BC-2.21.002 | First DT frame classifies `Unclassified`, later frame on same flow carries `Some(0x32)` | `classified_protocol` remains `Unclassified` — sticky-first-classification applies uniformly |
| EC-003 | BC-2.21.004 | `data.len() == 9` (one byte short) | `None`; T0814 on first occurrence per direction |
| EC-004 | BC-2.21.005 | `data[0] == 0x72` reaching this function (caller-drift simulation) | `None`; no `Finding` — pure defensive hygiene, not a wire anomaly |
| EC-005 | BC-2.21.007 | `data[1] == 0x00` | `None`; shares dedup flag with EC-003 |
| EC-006 | BC-2.21.008 | `data[1] == 0x02`, `data.len() == 11` (one short of the 12-byte Ack minimum) | `None`; malformed-header T0814 |
| EC-007 | BC-2.21.009 | `header.param_length == 0xFFFF`, `header.data_length == 0xFFFF`, `data.len() == 10` | Bounds check fails cleanly; no overflow in the sum; no slice attempted |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~6,200 |
| BC-2.21.001-009 excl. 003 (8 BCs) | ~9,000 |
| ADR-014 (Decisions 1, 2, 4, 8, 9) | ~10,000 |
| src/analyzer/s7comm.rs (from STORY-186) | ~4,000 |
| Test file delta + new fixture generator | ~4,500 |
| **Total** | **~33,700** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~17%** |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-186 | `S7commAnalyzer`/minimal `S7commFlowState` created; frame-walk loop proven; `on_flow_close` implemented | `S7commFlowState` grows incrementally across stories — do not assume the full field set exists until each field's owning story lands | The frame-walk loop's dispatch point (post `parse_cotp_header`) was a placeholder in STORY-186; this story is the first to give it real branching logic — do not regress the carry-buffer/resync behavior STORY-186 already proved |

The `on_flow_close` behavioral contract was deliberately packaged into STORY-186, not
this story, because flow-map lifecycle cohered better with the flow-map's creation than
with classification dispatch — see STORY-186's own BC table for the justification.

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 2**: the four-row disambiguation table (classic/plus/session/
  unclassified) lives entirely in `S7commAnalyzer`, never in SS-20. This story wires two
  of the four rows fully (session TPDUs, classic `0x32`); STORY-190 completes the
  remaining two (S7comm-plus, unclassified).
- **ADR-014 Decision 4**: S7comm classic field layout is derived from free-to-read prose/
  behavioral sources only (Wireshark wiki prose, Kleinmann & Wool 2014,
  Orange-Cyberdefense catalog) — never from Wireshark's dissector source, Snap7, or
  libnodave (all GPL/LGPL-tainted).
- **ADR-014 Decision 9 item 3**: `parse_s7comm_header` is a pure-core free `fn` — the
  VP-051 Kani target, and part of the combined VP-055 fuzz chain.
- Pure/effectful boundary: `parse_s7comm_header` is pure; `on_data`'s dispatch extension
  is the effectful shell.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | `u16::from_be_bytes`, `Option`, match patterns |
| kani | Latest via `cargo kani` | VP-051 formal verification harness |
| proptest | 1 (pinned in `Cargo.toml`) | VP-053 dispatch-totality skeleton (partial) |
| Python 3.10+ | — | `tests/fixtures/mk_s7comm_pcap.py` generator (mirrors `mk_modbus_pcap.py`) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/s7comm.rs` | MODIFY | Complete `S7commFlowState`; add `S7Protocol`, `Rosctr`, `S7commHeader`, `parse_s7comm_header`; extend `on_data`'s dispatch |
| `tests/s7comm_analyzer_tests.rs` | MODIFY | Add BC-2.21.001/002/004-009 unit tests + VP-051 Kani skeleton + VP-053 proptest skeleton |
| `tests/fixtures/mk_s7comm_pcap.py` | CREATE | Synthetic fixture generator per ADR-014 Decision 7 — Setup Communication + minimal classic-S7comm frames |

## Forbidden Dependencies

- Wireshark, Snap7, libnodave source, and any `s7`/`s7-comm`/`s7-client` crate — banned/
  avoid per ADR-014 Decision 4
- `parse_s7comm_header` MUST NOT read the `S7commAnalyzer`'s flow state or perform I/O —
  it remains a pure free fn per ADR-014 Decision 9

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — flow-state completion, four-way dispatch skeleton (classic branch fully wired), `parse_s7comm_header`, VP-051 Kani skeleton, VP-053 proptest skeleton (partial), AC-187-001..011. |
