---
document_type: story
level: ops
story_id: STORY-184
title: "S7comm TPKT Core Parser: parse_tpkt_header Pure-Core Free Function + VP-048 Kani Skeleton"
epic_id: E-23
version: "1.0"
status: ready
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 3
priority: P1
cycle: feature-s7comm
wave: 87
target_module: analyzer/iso_on_tcp
subsystems: [SS-20]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: []
blocks: [STORY-185]
behavioral_contracts: [BC-2.20.001, BC-2.20.002, BC-2.20.003, BC-2.20.004]
verification_properties: [VP-048]
inputs:
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.001.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.002.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.003.md
  - .factory/specs/behavioral-contracts/ss-20/BC-2.20.004.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/cycles/feature-s7comm/f1-delta-analysis.md
input-hash: "24c7b1e"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced (`todo!()` bodies + Red Gate density
> check >= 0.5 required before Step 4 dispatch).

# STORY-184: S7comm TPKT Core Parser: parse_tpkt_header Pure-Core Free Function + VP-048 Kani Skeleton

## Narrative

**As a** security analyst using wirerust to inspect Siemens S7comm traffic on TCP port
102,
**I want** a bounds-safe, pure-core TPKT (RFC 1006) header parser with formal Kani
verification,
**so that** every downstream COTP and S7comm dissection step (SS-20 COTP parse, SS-21
S7comm PDU parse) has a proven foundation: `parse_tpkt_header` never panics, correctly
rejects malformed input, and correctly extracts the `version`/`length` fields from a
valid 4-byte TPKT header.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.20.001 | `parse_tpkt_header` Returns None for Input Shorter Than 4 Bytes | Reject path: length < 4 |
| BC-2.20.002 | `parse_tpkt_header` Returns None for Version Byte != 0x03 | Reject path: bad version byte (also the SS-20 resync anchor) |
| BC-2.20.003 | `parse_tpkt_header` Returns None for Length Field < 7 (Malformed, Includes Zero-Length) | Reject path: declared length below the RFC 1006 §6 minimum of 7 |
| BC-2.20.004 | `parse_tpkt_header` Returns Some(TpktHeader) for Valid Input (Happy Path) | Accept path: `TpktHeader { version: 3, length }`, `length` in `[7, 65535]` |

## Acceptance Criteria

### AC-184-001: `parse_tpkt_header` returns None for input shorter than 4 bytes
(traces to BC-2.20.001 postcondition 1)
- Given a `&[u8]` slice with `data.len() < 4` (including empty, 1-byte, 3-byte slices)
- When `parse_tpkt_header(data)` is called
- Then returns `None` without accessing any byte in `data`; no panics
  (traces to BC-2.20.001 postcondition 2)
- **Test:** `test_BC_2_20_001_returns_none_for_three_bytes_canonical_vector`

### AC-184-002: `parse_tpkt_header` returns None for version byte != 0x03
(traces to BC-2.20.002 postcondition 1)
- Given `data.len() >= 4` and `data[0] != 0x03`
- When `parse_tpkt_header(data)` is called
- Then returns `None`; the length field (`data[2..4]`) is never decoded
  (traces to BC-2.20.002 postcondition 2)
- No panic for any `u8` value of `data[0]` (traces to BC-2.20.002 invariant 2)
- **Test:** `test_BC_2_20_002_returns_none_for_version_0x04_off_by_one_canonical_vector`

### AC-184-003: `parse_tpkt_header` returns None for decoded length < 7
(traces to BC-2.20.003 postcondition 1)
- Given `data.len() >= 4`, `data[0] == 0x03`, and `u16::from_be_bytes([data[2], data[3]]) < 7`
  (RFC 1006 §6 minimum)
- When `parse_tpkt_header(data)` is called
- Then returns `None`; no panic or overflow for any `u16` length value, including `0`
  (traces to BC-2.20.003 invariant 2)
- **Test:** `test_BC_2_20_003_returns_none_for_length_three_off_by_one_canonical_vector`

### AC-184-004: `parse_tpkt_header` returns Some(TpktHeader) for valid input
(traces to BC-2.20.004 postcondition 1)
- Given `data.len() >= 4`, `data[0] == 0x03`, and the decoded `length` in `[7, 65535]`
  (`7` is the RFC 1006 §6 minimum accept floor — the 4-byte TPKT header plus the 3-byte
  minimum COTP unit that must follow it)
- When `parse_tpkt_header(data)` is called
- Then returns `Some(TpktHeader { version: 3, length })` where `length` is exactly the
  big-endian `u16` decoded from `data[2..4]`
- `data[1]` (reserved byte) is never inspected; any value is accepted
  (traces to BC-2.20.004 invariant 1)
- `length == 65535` (the maximum representable `u16`, the "oversized-length-field" edge
  case) is a legal accept (traces to BC-2.20.004 invariant 2)
- **Test:** `test_BC_2_20_004_valid_input_returns_some_header_length_7_canonical_vector`

### AC-184-005: The four `parse_tpkt_header` outcomes are jointly exhaustive and mutually exclusive
(traces to BC-2.20.004 invariant 3)
- Given any `&[u8]` input
- When `parse_tpkt_header(data)` is called
- Then exactly one of BC-2.20.001/002/003's `None` paths or BC-2.20.004's `Some` path
  applies — no input falls outside all four, and no input satisfies more than one
- **Test:** `test_BC_2_20_004_four_way_partition_is_exhaustive` (unit-level spot check;
  full exhaustiveness is the VP-048 Kani obligation, see below)

### AC-184-006: VP-048 Kani harness skeleton compiles
(traces to BC-2.20.001 invariant 2) (traces to BC-2.20.002 invariant 2)
(traces to BC-2.20.003 invariant 2) (traces to BC-2.20.004 postcondition 3)
- Given the `#[cfg(kani)]` module in `src/analyzer/iso_on_tcp.rs`
- When `cargo kani --harness verify_parse_tpkt_header_safety` is run (against the
  `todo!()`-free implementation from this story)
- Then the harness skeleton compiles without errors
- The full Kani proof run (STORY-194) verifies: no panics for any symbolic `[u8; N]`
  input, and the four-way partition (AC-184-005) is exhaustive and non-overlapping over
  all possible `data` inputs, with no overflow in `h.length` decoding
- ADR-014 Decision 9 scope: VP-048 covers `parse_tpkt_header` only; `parse_cotp_header`
  is VP-049 (STORY-185); the combined no-panic frame-walk loop is VP-050/VP-055
- **Test:** `verify_parse_tpkt_header_safety` (Kani harness, full run deferred to STORY-194)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `TpktHeader` struct | SS-20 data model | `src/analyzer/iso_on_tcp.rs` | N/A (data type, frozen per ADR-014 Decision 1) |
| `parse_tpkt_header` | SS-20 TPKT parser | `src/analyzer/iso_on_tcp.rs` | Pure (free fn) |
| VP-048 harness | Kani verification | `src/analyzer/iso_on_tcp.rs` | `#[cfg(kani)]` block |

Subsystem anchor: SS-20 owns this story's scope because `parse_tpkt_header` is the
pure-core entry point of the ISO-on-TCP (TPKT/COTP) framing layer per ARCH-INDEX.md
§SS-20 and ADR-014 Decision 1's frozen module split (SS-20 `iso_on_tcp.rs` vs. SS-21
`s7comm.rs`).

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `TpktHeader` struct | pure-core | Plain data (`version: u8`, `length: u16`); no I/O |
| `parse_tpkt_header` | pure-core | Returns `Option<TpktHeader>` by value; no mutation, no I/O, no side effects, deterministic |

## VP-048 Kani Obligation

**Harness:** `verify_parse_tpkt_header_safety` (new; anchored in STORY-184)
**Method:** Kani symbolic execution
**Priority:** P0 (safety-critical parse function)

The Kani harness skeleton is written in this story. The full proof run targeting all
four outcomes (BC-2.20.001/002/003 reject paths, BC-2.20.004 accept path) is executed in
STORY-194. Skeleton structure (inside `#[cfg(kani)]` block):

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_parse_tpkt_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 300);
        let mut data = vec![0u8; len];
        for b in data.iter_mut() {
            *b = kani::any();
        }
        // Must not panic for any input:
        let _ = parse_tpkt_header(&data);
    }
}
```

ADR-014 Decision 9 scope note: this harness covers `parse_tpkt_header` only.
`parse_cotp_header`'s Kani obligation is VP-049 (STORY-185).

## Tasks

- [ ] Create `src/analyzer/iso_on_tcp.rs` with a module-level doc comment citing ADR-014
      Decision 1 (frozen SS-20 interface: pure free functions only, no `StreamAnalyzer`
      impl, no per-flow state of its own — the frozen-module-boundary contract that
      STORY-186 verifies structurally)
- [ ] Define `pub struct TpktHeader { pub version: u8, pub length: u16 }` (frozen struct
      per ADR-014 Decision 1 — exactly these two fields, no `reserved` field)
- [ ] Implement `pub fn parse_tpkt_header(data: &[u8]) -> Option<TpktHeader>` as a
      pure-core free fn:
  - `data.len() < 4` guard -> `None` (BC-2.20.001)
  - version check `data[0] != 0x03` -> `None` (BC-2.20.002)
  - decoded length `< 7` -> `None` (BC-2.20.003; RFC 1006 §6 minimum)
  - accept path -> `Some(TpktHeader { version: 3, length })` (BC-2.20.004)
- [ ] Write `#[cfg(kani)]` block with `verify_parse_tpkt_header_safety` skeleton
- [ ] Write unit tests: one per AC; named `test_BC_2_20_001_*` .. `test_BC_2_20_004_*`
- [ ] Verify `cargo check` passes with the new module
- [ ] Verify `cargo test` passes for this story's tests (all four BC paths + the
      four-way-partition spot check)
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the new TPKT header
      parser, before creating the PR (touches `src/` — CHANGELOG gate applies)

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.20.001 | `data.len() == 0` (empty slice) | `None` — no bytes accessed, no panic |
| EC-002 | BC-2.20.001 | `data.len() == 3` (one byte short of minimum) | `None` |
| EC-003 | BC-2.20.002 | `data[0] == 0x00` | `None` |
| EC-004 | BC-2.20.002 | `data[0] == 0x04` (off-by-one from the valid version) | `None` — no leniency |
| EC-005 | BC-2.20.003 | decoded `length == 0` (all-zero length field, most degenerate case) | `None` |
| EC-006 | BC-2.20.003 | decoded `length == 6` (one below the RFC 1006 §6 minimum of 7) | `None` |
| EC-007 | BC-2.20.004 | decoded `length == 7` (exactly the RFC 1006 §6 minimum — the smallest frame with room for a minimal COTP unit) | `Some(TpktHeader{version:3, length:7})` |
| EC-008 | BC-2.20.004 | decoded `length == 65535` (maximum representable `u16`) | `Some(TpktHeader{version:3, length:65535})` — accepted; stresses the carry-buffer ceiling introduced in STORY-186 |
| EC-009 | BC-2.20.004 | `data[1]` (reserved byte) is non-zero, e.g. `0xFF` | Accepted identically to `data[1] == 0x00` — reserved byte never validated |
| EC-010 | BC-2.20.004 | `data.len() > length as usize` (a second frame follows immediately) | `parse_tpkt_header` still returns `Some` for the first `length` bytes; frame-walk advance is a STORY-186 concern, not this function's |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,600 |
| BC-2.20.001-004 (4 BCs) | ~4,200 |
| ADR-014 (Decisions 1, 4, 9) | ~10,000 |
| src/analyzer/iso_on_tcp.rs (new file, stub) | ~1,200 |
| Test file (new unit tests) | ~1,800 |
| TOTAL | ~20,800 |

Agent context window ~200k tokens. This story uses ~10% — well within budget.

## Previous Story Intelligence

N/A — this is the first story in Epic E-23 (feature-s7comm). No predecessor story
exists in this epic. Closest cross-epic precedent: STORY-167 (IEC-104's
`parse_apci_header` + VP-044 Kani skeleton), which this story's shape mirrors closely
(single pure-core header-parse function, 4-6 BCs, Kani P0 skeleton).

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 1**: `iso_on_tcp.rs` (SS-20) is a standalone module, separate from
  `s7comm.rs` (SS-21). `TpktHeader { version: u8, length: u16 }` is the frozen struct —
  exactly these two fields, no `reserved` field surfaced.
- **ADR-014 Decision 4**: no external S7/COTP/TPKT crate is consulted or copied.
  `parse_tpkt_header` is implemented directly from RFC 1006 (freely implementable open
  spec) — original Rust only, zero lines borrowed.
- **ADR-014 Decision 9**: `parse_tpkt_header` is a pure-core free `fn` (module scope, not
  an `impl` method) — the VP-048 Kani target. It must not mutate state, perform I/O, or
  emit findings.
- Pure/effectful boundary: `parse_tpkt_header` is pure; the frame-walk loop that calls it
  (`S7commAnalyzer::on_data`, SS-21) is the effectful shell, built in STORY-186.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Language; `u8`, `u16::from_be_bytes`, `Option`, slice indexing |
| kani | Latest via `cargo kani` | VP-048 formal verification harness |

No new external crate dependencies. TPKT parser is original Rust — no `rusty-tpkt`/`tpkt`
crate (unclear/non-standard license per ADR-014 Decision 4) — implemented directly from
RFC 1006.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iso_on_tcp.rs` | CREATE | `TpktHeader`, `parse_tpkt_header`, `#[cfg(kani)]` VP-048 skeleton |
| `src/analyzer/mod.rs` | MODIFY | Add `pub mod iso_on_tcp;` (module created but not yet exported for external consumption beyond `s7comm.rs`, which does not exist until STORY-186) |
| `tests/iso_on_tcp_tests.rs` | CREATE | Unit tests for BC-2.20.001-004 |

## Forbidden Dependencies

The following crates MUST NOT appear in `src/analyzer/iso_on_tcp.rs` or any file it
imports:
- `rusty-cotp`, `rusty-tpkt`, `tpkt`, `copt` (unclear/non-standard license per ADR-014
  Decision 4 — "AVOID, implement from the open specs instead")
- `s7`, `s7-comm`, `s7-client` (non-standard custom license grant)
- Wireshark, Snap7, or libnodave source of any kind (GPL/LGPL — banned per ADR-014
  Decision 4)

`src/analyzer/iso_on_tcp.rs` MUST NOT depend on `dispatcher.rs` (mirrors `protocols.rs`'s
documented pure-core-leaf discipline) and MUST NOT contain an `impl StreamAnalyzer`
block (the frozen module-boundary contract, verified structurally starting in STORY-186
once `s7comm.rs` exists to be the sole consumer).

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — TPKT core parser, `parse_tpkt_header`, VP-048 Kani skeleton, AC-184-001..006. |
