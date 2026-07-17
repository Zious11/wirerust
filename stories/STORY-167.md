---
document_type: story
story_id: STORY-167
title: "IEC-104 APCI Core Parser: parse_apci_header Pure-Core Free Function + VP-044 Kani Skeleton"
epic_id: E-22
wave: 76
points: 5
phase: f3
tdd_mode: strict
status: delivered
feature_id: feature-iec104
subsystems: [SS-19]
target_module: analyzer/iec104
depends_on: []
blocks: [STORY-168, STORY-172]
behavioral_contracts:
  - BC-2.19.001
  - BC-2.19.002
  - BC-2.19.003
  - BC-2.19.004
  - BC-2.19.005
  - BC-2.19.006
verification_properties:
  - VP-044
  - VP-047
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.001.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.002.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.003.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.004.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.005.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.006.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "32f0ab7"
---

# STORY-167: IEC-104 APCI Core Parser: parse_apci_header Pure-Core Free Function + VP-044 Kani Skeleton

## Narrative

**As a** security analyst using wirerust to inspect ICS/SCADA network traffic on port 2404,
**I want** a bounds-safe, pure-core APCI header parser with formal Kani verification,
**so that** all downstream frame discrimination and threat-detection logic has a proven
foundation: `parse_apci_header` never panics, correctly rejects malformed input, and
correctly extracts CF1–CF4 from valid 6-byte APCI headers.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.001 | `parse_apci_header` Returns None for Input Shorter Than 6 Bytes | Core reject path (len<6) |
| BC-2.19.002 | `parse_apci_header` Returns None for Start Byte ≠ 0x68 | Core reject path (bad start) |
| BC-2.19.003 | `parse_apci_header` Returns None for LEN < 4 | Core reject path (lower bound) |
| BC-2.19.004 | `parse_apci_header` Returns None for LEN > 253 | Core reject path (upper bound) |
| BC-2.19.005 | `parse_apci_header` Returns Some(ApciHeader) for Valid Input | Accept path + CF1–CF4 extraction |
| BC-2.19.006 | `is_valid_iec104_frame` Post-Classification Validity Gate | Lightweight validity gate (2-byte check) |

## Acceptance Criteria

### AC-167-001: `parse_apci_header` returns None for len < 6
**Traces to:** BC-2.19.001 postconditions 1–3
- Given a `&[u8]` slice with `len < 6` (including empty slice, 1-byte, 5-byte)
- When `parse_apci_header(data)` is called
- Then returns `None` without accessing any bytes; no panics; no partial decode
- The function is pure: no side effects, no global state mutation

### AC-167-002: `parse_apci_header` returns None for start byte ≠ 0x68
**Traces to:** BC-2.19.002 postcondition 1 and invariant 1
- Given `data.len() >= 6` and `data[0] != 0x68`
- When `parse_apci_header(data)` is called
- Then returns `None`; the IEC-104 start byte (0x68) is fixed by protocol specification

### AC-167-003: `parse_apci_header` returns None for LEN < 4
**Traces to:** BC-2.19.003 postcondition 1 and invariant 1
- Given `data.len() >= 6`, `data[0] == 0x68`, and `data[1] < 4` (LEN byte < 4)
- When `parse_apci_header(data)` is called
- Then returns `None`; LEN=4 is the minimum (U-frame CF1–CF4, no ASDU)

### AC-167-004: `parse_apci_header` returns None for LEN > 253
**Traces to:** BC-2.19.004 postcondition 1 and invariant 1
- Given `data.len() >= 6`, `data[0] == 0x68`, and `data[1] > 253` (LEN byte > 253)
- When `parse_apci_header(data)` is called
- Then returns `None`; LEN=253 is the maximum (LEN+2=255 total, fitting in one u8)

### AC-167-005: `parse_apci_header` returns Some(ApciHeader) for valid input
**Traces to:** BC-2.19.005 postconditions 1–6
- Given `data.len() >= 6`, `data[0] == 0x68`, `4 <= data[1] <= 253`
- When `parse_apci_header(data)` is called
- Then returns `Some(ApciHeader { len: data[1], cf1: data[2], cf2: data[3], cf3: data[4], cf4: data[5] })`
- `len` (LEN field) is in `[4, 253]`; `len + 2` (total frame bytes) is in `[6, 255]` — no overflow
- CF1–CF4 are copied verbatim; bytes beyond index 5 are not accessed by this function

### AC-167-006: `is_valid_iec104_frame` post-classification validity gate
**Traces to:** BC-2.19.006 postconditions 1–3 and invariant 3
- Given a `&[u8]` slice from a port-2404-dispatched flow
- When `is_valid_iec104_frame(data)` is called
- Then returns `true` iff `data.len() >= 2 && data[0] == 0x68 && 4 <= data[1] <= 253`
- Returns `false` for empty slice, wrong start byte, or out-of-range LEN
- Any input where `is_valid_iec104_frame` returns `true` and `data.len() >= 6` will yield `Some` from `parse_apci_header`

### AC-167-007: VP-044 Kani harness skeleton compiles and passes with `todo!()` bodies
**Traces to:** BC-2.19.006 invariant 2 (purity) and BC-2.19.005 postcondition 5
- Given the `#[cfg(kani)]` module in `src/analyzer/iec104.rs`
- When `cargo kani --harness verify_parse_apci_header_safety` is run (with stub implementation)
- Then the harness skeleton compiles without errors
- The full Kani proof run (STORY-174) verifies: no panics for any symbolic `[u8; N]` input, and all
  five facets (len<6→None, start≠0x68→None, LEN<4→None, LEN>253→None, valid→Some) are correct
- ADR-013 Decision 8: VP-044 scope is `parse_apci_header` only; `on_data` loop no-panic belongs to VP-047

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `parse_apci_header` | SS-19 APCI parser | `src/analyzer/iec104.rs` | Pure (free fn) |
| `is_valid_iec104_frame` | SS-19 validity gate | `src/analyzer/iec104.rs` | Pure (free fn) |
| `ApciHeader` struct | SS-19 data model | `src/analyzer/iec104.rs` | N/A (data type) |
| `Iec104ParseError` enum | SS-19 error type | `src/analyzer/iec104.rs` | N/A (data type) |
| VP-044 harness | Kani verification | `src/analyzer/iec104.rs` | `#[cfg(kani)]` block |

Subsystem anchor: SS-19 owns this story's scope because `parse_apci_header` is the pure-core
entry point of the IEC-104 passive analyzer per ARCH-INDEX.md §SS-19.

## VP-044 Kani Obligation

**Harness:** `verify_parse_apci_header_safety` (new; anchored in STORY-167)
**Method:** Kani symbolic execution
**Priority:** P0 (safety-critical parse function)

The Kani harness skeleton is written in this story. The full proof run targeting all five facets
is executed in STORY-174. Skeleton structure (inside `#[cfg(kani)]` block):

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_parse_apci_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 300);
        let mut data = vec![0u8; len];
        for b in data.iter_mut() {
            *b = kani::any();
        }
        // Must not panic for any input:
        let _ = parse_apci_header(&data);
    }
}
```

ADR-013 Decision 8 scope note: this harness covers `parse_apci_header` only. The `on_data`
frame-walk loop no-panic proof is VP-047 (cargo-fuzz `fuzz_iec104_parser`).

## Tasks

- [ ] Create `src/analyzer/iec104.rs` with module-level doc comment citing ADR-013
- [ ] Define `ApciHeader` struct: `len: u8, cf1: u8, cf2: u8, cf3: u8, cf4: u8`
- [ ] Define `Iec104ParseError` error enum (skeleton; extended in STORY-168)
- [ ] Implement `parse_apci_header(data: &[u8]) -> Option<ApciHeader>` as pure-core free fn
  - len<6 guard → None (BC-2.19.001)
  - start byte check `data[0] != 0x68` → None (BC-2.19.002)
  - LEN lower bound `data[1] < 4` → None (BC-2.19.003)
  - LEN upper bound `data[1] > 253` → None (BC-2.19.004)
  - Accept path: return Some(ApciHeader) (BC-2.19.005)
- [ ] Implement `is_valid_iec104_frame(data: &[u8]) -> bool` (BC-2.19.006)
- [ ] Write `#[cfg(kani)]` block with `verify_parse_apci_header_safety` skeleton
- [ ] Write unit tests: one per AC; named `test_BC_2_19_001_*`, `test_BC_2_19_002_*`, etc.
- [ ] Verify `cargo check` passes with new module
- [ ] Verify `cargo test` passes for this story's tests (all 5 AC paths + is_valid)

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.001 | Empty slice `&[]` | `parse_apci_header` → None |
| EC-002 | BC-2.19.001 | 5-byte slice (one short of minimum) | `parse_apci_header` → None |
| EC-003 | BC-2.19.002 | `data[0] = 0x69` (off-by-one from 0x68) | None |
| EC-004 | BC-2.19.003 | LEN=3 (below minimum) | None |
| EC-005 | BC-2.19.003 | LEN=4 (boundary, minimum) | Some(ApciHeader) if other conditions met |
| EC-006 | BC-2.19.004 | LEN=253 (boundary, maximum) | Some(ApciHeader) if other conditions met |
| EC-007 | BC-2.19.004 | LEN=254 (one above maximum) | None |
| EC-008 | BC-2.19.006 | is_valid on 1-byte slice | false (can't read LEN) |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| BC-2.19.001–006 (6 BCs × ~600 each) | ~3,600 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (new file, stub) | ~1,500 |
| Test file (new unit tests) | ~2,000 |
| TOTAL | ~30,600 |

Agent context window ~200k tokens. This story uses ~15% — within budget.

## Previous Story Intelligence

N/A — this is the first story in Epic E-22 (IEC-104 feature). No predecessor story exists.

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 1**: IEC-104 dispatched via Rule 8 (port 2404); `is_valid_iec104_frame`
  used as lightweight content-signature gate (NOT added to `classify()` rule table).
- **ADR-013 Decision 3**: Frame-walk loop is in `on_data` (effectful shell), not in
  `parse_apci_header`. `parse_apci_header` is a pure function.
- **ADR-013 Decision 8**: `parse_apci_header` is the VP-044 Kani target. The `on_data`
  loop is the VP-047 fuzz target. These scopes must not be conflated.
- **RULING-DNP3-SIBLING-001**: Carry buffers are directionally isolated (enforced in STORY-172).
- Pure/effectful boundary: `parse_apci_header` and `is_valid_iec104_frame` are pure free
  functions; `Iec104Analyzer::on_data` is the effectful shell. Pure fns must not call
  `emit_finding`, access flow state, or perform I/O.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Language; `u8`, `Option`, slice indexing |
| kani | Latest via `cargo kani` | VP-044 formal verification harness |
| cargo-fuzz | Latest | VP-047 fuzz harness (scaffolded, run in STORY-174) |

No new external crate dependencies. IEC-104 parser is original Rust — no `iec60870-5` crate
(proprietary), no Wireshark dissector (GPLv2), no lib60870 (GPLv3) per ADR-013 Decision 7.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | CREATE | `ApciHeader`, `Iec104ParseError`, `parse_apci_header`, `is_valid_iec104_frame`, `#[cfg(kani)]` VP-044 skeleton |
| `src/analyzer/mod.rs` | MODIFY | Add `pub mod iec104;` (deferred to STORY-173 registration; create module but do not export publicly until dispatch integration) |
| `tests/iec104_analyzer_tests.rs` | CREATE | Unit tests for BC-2.19.001–006 |

## Forbidden Dependencies

The following crates MUST NOT appear in `src/analyzer/iec104.rs` or any file it imports:
- `iec60870-5` (proprietary — licensing violation per ADR-013 Decision 7)
- `wireshark` (GPLv2 — licensing violation)
- `lib60870` (GPLv3 — licensing violation)

Build enforcement: if any of these crate names appear in `Cargo.toml` after this story,
the build MUST fail at the license-check step. Original implementation only.
