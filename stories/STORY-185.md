---
document_type: story
level: ops
story_id: STORY-185
title: "S7comm COTP TPDU-Type Parser: parse_cotp_header, Protocol-ID Extraction, VP-049 Kani Skeleton"
epic_id: E-23
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 5
priority: P1
cycle: feature-s7comm
wave: 88
target_module: analyzer/iso_on_tcp
subsystems: [SS-20]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-184]
blocks: [STORY-186]
behavioral_contracts: [BC-2.20.005, BC-2.20.006, BC-2.20.007, BC-2.20.008, BC-2.20.009, BC-2.20.010, BC-2.20.011, BC-2.20.012]
verification_properties: [VP-049]
inputs:
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.005.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.006.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.007.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.008.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.009.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.010.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.011.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.012.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
input-hash: "54ebb24"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-185: S7comm COTP TPDU-Type Parser: parse_cotp_header, Protocol-ID Extraction, VP-049 Kani Skeleton

## Narrative

**As a** security analyst using wirerust to inspect Siemens S7comm traffic,
**I want** the ISO-on-TCP framing layer to correctly discriminate COTP (ISO 8073) TPDU
types (CR/CC/DT) and extract the DT-TPDU's protocol-ID byte without interpreting it,
**so that** `S7commAnalyzer` (built in STORY-186 onward) can dispatch classic S7comm
(`0x32`), S7comm-plus (`0x72`), and any other port-102 traffic (MMS, ICCP, unrecognized)
without SS-20 ever having S7comm-specific knowledge baked into its parsing logic.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.20.005 | `parse_cotp_header` Returns None for Input Shorter Than 2 Bytes | Reject path: length < 2 |
| BC-2.20.006 | `parse_cotp_header` Returns None When LI Declares More Bytes Than Present | Reject path: LI-truncation |
| BC-2.20.007 | `parse_cotp_header` Recognizes Connect Request (CR) TPDU | Accept path: CR, `protocol_id: None` |
| BC-2.20.008 | `parse_cotp_header` Recognizes Connect Confirm (CC) TPDU | Accept path: CC, `protocol_id: None` |
| BC-2.20.009 | `parse_cotp_header` Recognizes DT TPDU With Non-Empty Payload | Accept path: DT, `protocol_id: Some(byte)` |
| BC-2.20.010 | `parse_cotp_header` Recognizes DT TPDU With Empty Payload | Accept path: DT, `protocol_id: None` |
| BC-2.20.011 | `parse_cotp_header` Returns None for an Unrecognized TPDU-Type Code | Reject path: not CR/CC/DT |
| BC-2.20.012 | `protocol_id` Extracted Verbatim, Never Interpreted (Frozen SS-20 to SS-21 Boundary) | Architectural correctness property |

## Acceptance Criteria

### AC-185-001: `parse_cotp_header` returns None for input shorter than 2 bytes
(traces to BC-2.20.005 postcondition 1)
- Given `tpkt_payload.len() < 2` (including the empty-payload case from a TPKT
  `length == 4` header-only frame)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `None`; no bytes accessed beyond the length check, no panic even for
  `len() == 0` (traces to BC-2.20.005 postcondition 2)
- **Test:** `test_BC_2_20_005_len_shorter_than_2_returns_none`

### AC-185-002: `parse_cotp_header` returns None when the Length Indicator declares more bytes than are present
(traces to BC-2.20.006 postcondition 1)
- Given `tpkt_payload.len() >= 2` and `tpkt_payload.len() < 1 + tpkt_payload[0] as usize`
  (LI truncation)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `None`; no out-of-bounds index for any `u8` LI value, including `0`
  (traces to BC-2.20.006 postcondition 2, invariant 2)
- **Test:** `test_BC_2_20_006_li_truncation_returns_none`

### AC-185-003: `parse_cotp_header` recognizes Connect Request (CR)
(traces to BC-2.20.007 postcondition 1)
- Given `tpkt_payload[1] & 0xF0 == 0xE0` and the LI-truncation check has passed
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: ConnectRequest, protocol_id: None,
  payload_offset })` where `payload_offset == 1 + LI` (traces to BC-2.20.007
  postcondition 2)
- `protocol_id` is unconditionally `None` for CR, regardless of any bytes present beyond
  the fixed CR header (traces to BC-2.20.007 postcondition 3)
- **Test:** `test_BC_2_20_007_connect_request_recognized`

### AC-185-004: `parse_cotp_header` recognizes Connect Confirm (CC)
(traces to BC-2.20.008 postcondition 1)
- Given `tpkt_payload[1] & 0xF0 == 0xD0` and the LI-truncation check has passed
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: ConnectConfirm, protocol_id: None,
  payload_offset })` with `payload_offset == 1 + LI` (traces to BC-2.20.008
  postcondition 2)
- **Test:** `test_BC_2_20_008_connect_confirm_recognized`

### AC-185-005: `parse_cotp_header` recognizes DT with non-empty payload and extracts protocol_id
(traces to BC-2.20.009 postcondition 1)
- Given `tpkt_payload[1] & 0xF0 == 0xF0` and `tpkt_payload.len() > payload_offset` where
  `payload_offset = 1 + LI`
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id:
  Some(tpkt_payload[payload_offset]), payload_offset })` (traces to BC-2.20.009
  postcondition 1)
- `protocol_id` is the trailing byte verbatim for every `u8` value (`0x32`, `0x72`, or
  any other byte) — never coerced or force-fit (traces to BC-2.20.009 edge case EC-004)
- **Test:** `test_BC_2_20_009_dt_nonempty_payload_extracts_protocol_id`

### AC-185-006: `parse_cotp_header` recognizes DT with empty payload — protocol_id is None
(traces to BC-2.20.010 postcondition 1)
- Given `tpkt_payload[1] & 0xF0 == 0xF0` and `tpkt_payload.len() == payload_offset`
  exactly (no trailing byte)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: None,
  payload_offset })`; no out-of-bounds index at `tpkt_payload[payload_offset]` (traces
  to BC-2.20.010 postcondition 2)
- **Test:** `test_BC_2_20_010_dt_empty_payload_protocol_id_none`

### AC-185-007: `parse_cotp_header` returns None for an unrecognized TPDU-type code
(traces to BC-2.20.011 postcondition 1)
- Given `tpkt_payload[1] & 0xF0` is none of `0xE0` (CR), `0xD0` (CC), `0xF0` (DT) — i.e.
  one of the 13 remaining nibble values (DR, DC, ED, AK, EA, RJ, ER, and others)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `None`; no panic for any of the 13 remaining nibble values; the frame is
  never force-fit into CR, CC, or DT (traces to BC-2.20.011 postcondition 2, invariant 2)
- **Test:** `test_BC_2_20_011_unrecognized_tpdu_type_returns_none`

### AC-185-008: The four-way TPDU-type match is exhaustive and non-overlapping over all 16 nibble values
(traces to BC-2.20.011 invariant 3)
- Given any `u8` value at `tpkt_payload[1] & 0xF0`
- When `parse_cotp_header` classifies it
- Then exactly one of CR (`0xE`), CC (`0xD`), DT-with-payload/DT-empty-payload (`0xF`),
  or the unrecognized-reject arm (the 13 remaining values) applies
- **Test:** `test_BC_2_20_011_tpdu_type_match_is_exhaustive` (unit-level spot check; full
  exhaustiveness is the VP-049 Kani obligation)

### AC-185-009: `protocol_id` extraction is a total, uninterpreted identity mapping
(traces to BC-2.20.012 postcondition 1)
- Given the DT-with-non-empty-payload branch (BC-2.20.009 preconditions hold)
- When `parse_cotp_header` extracts the protocol-ID byte
- Then for any `u8` value `b`, the result is `protocol_id: Some(b)` — no branch, match
  arm, or conditional inside `parse_cotp_header` ever compares `b` against `0x32`,
  `0x72`, or any other specific value (traces to BC-2.20.012 postcondition 2)
- `src/analyzer/iso_on_tcp.rs` contains no reference to the literals `0x32`/`0x72` nor
  the strings "S7comm"/"S7comm-plus" anywhere in its parsing logic (traces to
  BC-2.20.012 postcondition 3)
- **Test:** `test_BC_2_20_012_protocol_id_extraction_totality` (proptest sweep over all
  256 `u8` values) and a static regression-guard test asserting zero occurrences of
  `0x32`/`0x72` literals in `src/analyzer/iso_on_tcp.rs`'s parsing logic

### AC-185-010: VP-049 Kani harness skeleton compiles
(traces to BC-2.20.005 postcondition 2) (traces to BC-2.20.006 invariant 1)
(traces to BC-2.20.011 invariant 3)
- Given the `#[cfg(kani)]` module in `src/analyzer/iso_on_tcp.rs`
- When `cargo kani --harness verify_parse_cotp_header_safety` is run
- Then the harness skeleton compiles without errors
- The full Kani proof run (STORY-194) verifies: no panics or out-of-bounds reads for any
  symbolic input (including the LI-truncation bounds check), the TPDU-type
  classification is exhaustive and non-overlapping over all 16 nibble values, and the
  protocol-ID extraction is a total identity mapping over all 256 `u8` values
- **Test:** `verify_parse_cotp_header_safety` (Kani harness, full run deferred to STORY-194)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `CotpTpduType` enum | SS-20 data model | `src/analyzer/iso_on_tcp.rs` | N/A (frozen, exactly 3 variants per ADR-014 Decision 1) |
| `CotpHeader` struct | SS-20 data model | `src/analyzer/iso_on_tcp.rs` | N/A (frozen: `tpdu_type`, `protocol_id: Option<u8>`, `payload_offset: usize`) |
| `parse_cotp_header` | SS-20 COTP parser | `src/analyzer/iso_on_tcp.rs` | Pure (free fn) |
| VP-049 harness | Kani verification | `src/analyzer/iso_on_tcp.rs` | `#[cfg(kani)]` block |

Subsystem anchor: SS-20 owns this story's scope because `parse_cotp_header` is the
second pure-core parsing layer of the ISO-on-TCP framing subsystem per ARCH-INDEX.md
§SS-20, consuming the already-accepted `TpktHeader` payload from STORY-184's
`parse_tpkt_header`.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `CotpTpduType`, `CotpHeader` | pure-core | Plain data types; no I/O |
| `parse_cotp_header` | pure-core | Returns `Option<CotpHeader>` by value; no mutation, no I/O, no S7comm-specific interpretation of the extracted `protocol_id` byte |

## VP-049 Kani Obligation

**Harness:** `verify_parse_cotp_header_safety` (new; anchored in STORY-185)
**Method:** Kani symbolic execution
**Priority:** P0

The skeleton is written here; the full proof (bounds safety across all LI values,
TPDU-type exhaustiveness over all 16 nibble values, protocol-ID-extraction totality over
all 256 `u8` values) runs in STORY-194. Skeleton structure:

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_parse_cotp_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 300);
        let mut data = vec![0u8; len];
        for b in data.iter_mut() {
            *b = kani::any();
        }
        let _ = parse_cotp_header(&data);
    }
}
```

## Tasks

- [ ] Extend `src/analyzer/iso_on_tcp.rs` with:
  - `pub enum CotpTpduType { ConnectRequest, ConnectConfirm, DataTransfer }` (frozen,
    exactly 3 variants — ADR-014 Decision 1)
  - `pub struct CotpHeader { pub tpdu_type: CotpTpduType, pub protocol_id: Option<u8>,
    pub payload_offset: usize }` (frozen)
- [ ] Implement `pub fn parse_cotp_header(tpkt_payload: &[u8]) -> Option<CotpHeader>`:
  - `tpkt_payload.len() < 2` guard -> `None` (BC-2.20.005)
  - LI-truncation guard `tpkt_payload.len() < 1 + LI` -> `None` (BC-2.20.006)
  - `tpkt_payload[1] & 0xF0 == 0xE0` -> CR, `protocol_id: None` (BC-2.20.007)
  - `tpkt_payload[1] & 0xF0 == 0xD0` -> CC, `protocol_id: None` (BC-2.20.008)
  - `tpkt_payload[1] & 0xF0 == 0xF0` and `len() > payload_offset` -> DT,
    `protocol_id: Some(tpkt_payload[payload_offset])` (BC-2.20.009)
  - `tpkt_payload[1] & 0xF0 == 0xF0` and `len() == payload_offset` -> DT,
    `protocol_id: None` (BC-2.20.010)
  - any other high-nibble value -> `None` (BC-2.20.011)
- [ ] Write `#[cfg(kani)]` block with `verify_parse_cotp_header_safety` skeleton
- [ ] Write unit tests: one per AC; named `test_BC_2_20_005_*` .. `test_BC_2_20_012_*`
- [ ] Write the proptest sweep over all 256 `u8` protocol-ID values (AC-185-009) and the
      static regression-guard test (grep-equivalent assertion) for zero `0x32`/`0x72`
      literals in the parsing logic
- [ ] Verify `cargo test` passes for this story's tests
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the new COTP TPDU
      parser, before creating the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.20.005 | `tpkt_payload.len() == 0` (TPKT `length == 4` header-only frame) | `None` — legitimately empty payload |
| EC-002 | BC-2.20.006 | `LI == 6`, only 3 bytes present after LI | `None` — truncated CR/CC header |
| EC-003 | BC-2.20.006 | `LI == 0` (degenerate but `1 + 0 <= len`) | Not truncated; proceeds to TPDU-type recognition |
| EC-004 | BC-2.20.007 | `tpkt_payload[1] == 0xE1` (CR with non-zero low nibble) | Still recognized as CR — high-nibble-only discrimination |
| EC-005 | BC-2.20.009 | protocol-ID byte `== 0x32` (classic S7comm) | `Some(0x32)` extracted verbatim; SS-21 (STORY-187+) disambiguates, not this function |
| EC-006 | BC-2.20.009 | protocol-ID byte `== 0x72` (S7comm-plus) | `Some(0x72)` extracted verbatim |
| EC-007 | BC-2.20.009 | protocol-ID byte `== 0x01` (simulating MMS/ICCP) | `Some(0x01)` extracted verbatim — never coerced to `None` or misattributed |
| EC-008 | BC-2.20.011 | `tpkt_payload[1] & 0xF0 == 0x80` (DR, Disconnect Request) | `None` — not modeled |
| EC-009 | BC-2.20.011 | `tpkt_payload[1] == 0x00` (all-zero, no valid high nibble) | `None` |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~5,000 |
| BC-2.20.005-012 (8 BCs) | ~7,200 |
| ADR-014 (Decisions 1, 2, 4, 9) | ~10,000 |
| src/analyzer/iso_on_tcp.rs (from STORY-184) | ~2,000 |
| Test file delta | ~2,500 |
| **Total** | **~26,700** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~13%** |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-184 | `TpktHeader { version, length }` frozen struct; `parse_tpkt_header` pure free fn | Pure-core free-fn design for Kani amenability | The reserved byte (`data[1]`) is never validated — carry that same "don't-care" discipline forward: `parse_cotp_header` similarly never validates COTP variable parameters beyond the fixed prefix |

This story extends `src/analyzer/iso_on_tcp.rs` (created in STORY-184) with the second
pure-core parsing layer. `s7comm.rs` (SS-21) does not exist yet — it is created in
STORY-186. This story's tests operate directly on byte slices (`tpkt_payload: &[u8]`),
not on any flow/analyzer state.

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 1**: `CotpTpduType` has exactly 3 variants (`ConnectRequest`,
  `ConnectConfirm`, `DataTransfer`) — deliberately not all ISO 8073 TPDU types. `CotpHeader`
  has exactly `tpdu_type`, `protocol_id: Option<u8>`, `payload_offset: usize` — no
  additional fields.
- **ADR-014 Decision 2**: `parse_cotp_header` performs zero interpretation of the
  extracted `protocol_id` byte — the four-row disambiguation table (classic S7comm,
  S7comm-plus, session-establishment, unrecognized/unclassified) lives entirely in
  `S7commAnalyzer` (SS-21), built starting in STORY-186/STORY-187.
- **ADR-014 Decision 4**: COTP field layout implemented directly from ITU-T X.224 (=
  ISO/IEC 8073:1997) — a freely downloadable open specification, not from Wireshark's
  `packet-s7comm.c` (GPLv2, banned) or any other copyleft source.
- **ADR-014 Decision 9**: `parse_cotp_header` is a pure-core free `fn` — the VP-049 Kani
  target. It must never call `emit_finding`, access flow state, or perform I/O.
- Pure/effectful boundary: `parse_cotp_header` is pure; the frame-walk loop that calls it
  is the effectful shell, built in STORY-186.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Language; bitwise `&`, match patterns, enum dispatch |
| kani | Latest via `cargo kani` | VP-049 formal verification harness |
| proptest | 1 (pinned in `Cargo.toml`) | AC-185-009 protocol-ID totality sweep |

No new external crate dependencies beyond what STORY-184 introduced.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/iso_on_tcp.rs` | MODIFY | Add `CotpTpduType`, `CotpHeader`, `parse_cotp_header`, `#[cfg(kani)]` VP-049 skeleton |
| `tests/iso_on_tcp_tests.rs` | MODIFY | Add BC-2.20.005-012 unit tests + protocol-ID totality proptest + regression-guard static check |

## Forbidden Dependencies

- `rusty-cotp`, `rusty-tpkt`, `tpkt`, `copt`, `s7`, `s7-comm`, `s7-client` — banned/avoid
  per ADR-014 Decision 4
- Wireshark, Snap7, libnodave source of any kind — banned (GPL/LGPL)
- `src/analyzer/iso_on_tcp.rs` MUST NOT contain the literals `0x32` or `0x72`, nor the
  strings `"S7comm"`/`"S7comm-plus"`, anywhere in its parsing logic (BC-2.20.012
  postcondition 3) — this is the load-bearing correctness property that keeps SS-20
  reusable by a future IEC 61850 MMS or ICCP/TASE.2 cycle without modification.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — COTP TPDU-type parser, protocol-ID extraction, VP-049 Kani skeleton, AC-185-001..010. |
