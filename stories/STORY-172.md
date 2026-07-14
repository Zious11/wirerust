---
document_type: story
story_id: STORY-172
title: "IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle (on_data / on_flow_close)"
epic_id: E-22
wave: 81
points: 5
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-iec104
subsystems: [SS-19]
target_module: analyzer/iec104
depends_on: [STORY-170, STORY-171]
blocks: [STORY-173]
behavioral_contracts:
  - BC-2.19.025
  - BC-2.19.026
  - BC-2.19.027
verification_properties:
  - VP-045
  - VP-047
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.025.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.026.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.027.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "27f3cf4"
---

# STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle

## Narrative

**As a** security analyst using wirerust to inspect IEC-104 traffic spanning TCP segment
boundaries,
**I want** the analyzer to correctly manage directional carry buffers (bounded at 255 bytes),
implement a terminating frame-walk loop, and cleanly tear down per-flow state on connection close,
**so that** multi-frame on_data calls are processed correctly, carry overflow attacks are
detected (T0814), and no state leaks occur across flows.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.025 | Directional Carry Buffers Bounded at MAX_IEC104_CARRY_BYTES=255 (VP-045) | Carry buffer management — bounded, directionally isolated |
| BC-2.19.026 | Frame-Walk Loop Processes Multiple APDUs per on_data Call | Main processing loop — on_data implementation |
| BC-2.19.027 | on_flow_close Removes Iec104FlowState and Discards Carry Bytes | Flow lifecycle teardown |

## Acceptance Criteria

### AC-172-001: Directional carry buffers are independent and bounded at 255 bytes
**Traces to:** BC-2.19.025 postconditions 1–4 and invariants 1–2
- Given independent C2S and S2C byte streams for a flow
- When carry bytes accumulate across on_data calls
- Then `carry_c2s` and `carry_s2c` are always strictly separate — bytes from one direction
  are never appended to the other's carry buffer (VP-045 proptest verifies this)
- Each carry buffer is bounded at `MAX_IEC104_CARRY_BYTES = 255` bytes

### AC-172-002: Carry overflow emits T0814 and discards excess bytes
**Traces to:** BC-2.19.025 postconditions 1–3 and invariant 3
- Given a carry buffer at N bytes and new bytes arriving such that N + new > 255
- When the frame-walk loop would extend the carry beyond the cap
- Then T0814 "Denial of Service" finding is emitted (confidence Possible)
- Excess bytes are discarded; the carry buffer is NOT extended beyond 255

### AC-172-003: Frame-walk loop processes all complete APCI frames per on_data call
**Traces to:** BC-2.19.026 postconditions 1–3
- Given data containing multiple complete APCI frames (e.g., two back-to-back STARTDT frames)
- When `on_data(data, direction, state)` is called
- Then all complete frames are parsed and dispatched sequentially
- Remaining incomplete bytes are stashed into the directional carry buffer

### AC-172-004: Frame-walk loop terminates — cursor advances ≥ 1 byte per iteration
**Traces to:** BC-2.19.026 postcondition 4 and invariant 1
- The loop always advances the cursor per iteration per ADR-013 Decision 3:
  - Bad start byte (data[pos] != 0x68): advance 1 byte; carry NOT cleared (resync scan)
  - Malformed LEN (LEN < 4 or LEN > 253): advance 2 bytes (skip APCI stub)
  - Valid frame: advance LEN+2 bytes
  - Insufficient data (< LEN+2 bytes remain after start): stash remaining to carry; return
- No infinite loop is possible for any finite input (VP-047 fuzz verifies this)

### AC-172-005: on_data does not panic for any byte sequence
**Traces to:** BC-2.19.026 postcondition 5 (VP-047 no-panic obligation)
- Given any arbitrary byte sequence delivered to `on_data`
- When `on_data(data, direction, state)` is called
- Then the function returns without panicking, unwinding, or accessing out-of-bounds memory
- This is the top-level VP-047 fuzz harness target: `fuzz_iec104_parser` calls `on_data` directly

### AC-172-006: on_flow_close removes state and discards carry bytes
**Traces to:** BC-2.19.027 postconditions 1–4 and invariants 1–2
- Given a flow with active `Iec104FlowState` (possibly with non-empty carry buffers)
- When `on_flow_close(flow_id)` is called
- Then `Iec104FlowState` for that flow is removed from the state map
- `carry_c2s` and `carry_s2c` are dropped (memory freed)
- No finding is emitted for normal flow close
- Calling `on_flow_close` for an already-removed flow_id is a no-op (no panic)

### AC-172-007: VP-045 proptest skeleton compiles — carry direction isolation
**Traces to:** BC-2.19.025 invariant 1 (VP-045 proptest obligation)
- Given the `proptest_vp045_direction_isolation` and `proptest_vp045_independent_run_equivalence`
  harnesses anchored in this story
- When the proptest skeletons are compiled
- Then they compile without error
- Full proptest runs are executed in STORY-174
- Mirrors VP-033 (ENIP), VP-035 (DNP3), VP-037 (Modbus) pattern

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `carry_c2s: Vec<u8>` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
| `carry_s2c: Vec<u8>` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
| `MAX_IEC104_CARRY_BYTES` const | SS-19 constants | `src/analyzer/iec104.rs` | N/A |
| `Iec104Analyzer::on_data` | SS-19 effectful shell | `src/analyzer/iec104.rs` | Effectful |
| `Iec104Analyzer::on_flow_close` | SS-19 lifecycle | `src/analyzer/iec104.rs` | Effectful |
| `Iec104Analyzer::flows` | SS-19 state map | `src/analyzer/iec104.rs` | Mutable HashMap |
| VP-045 proptest | proptest harnesses | `tests/iec104_analyzer_tests.rs` | Test-only |

Subsystem anchor: SS-19 owns this story's scope because carry buffer management, the
frame-walk loop, and flow lifecycle are the central processing infrastructure of the
IEC-104 passive analyzer per ARCH-INDEX.md §SS-19.

## VP-045 Proptest Obligation

**Harnesses:** `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence`
**Anchored in:** STORY-172 (carry buffer management is the primary VP-045 target per BC-2.19.025)
**Method:** proptest
**Priority:** P1

Skeleton structure (in `tests/iec104_analyzer_tests.rs`):

```rust
proptest! {
    #[test]
    fn proptest_vp045_direction_isolation(
        c2s_data in prop::collection::vec(any::<u8>(), 0..256),
        s2c_data in prop::collection::vec(any::<u8>(), 0..256),
    ) {
        let mut analyzer = Iec104Analyzer::new();
        let flow_id = FlowId::test_default();
        // Interleaved C2S and S2C deliveries must not mix carries
        analyzer.on_data(&c2s_data, Direction::ClientToServer, flow_id);
        analyzer.on_data(&s2c_data, Direction::ServerToClient, flow_id);
        // carry_c2s must only contain bytes from c2s_data path
        // carry_s2c must only contain bytes from s2c_data path
        // (proptest verifies isolation invariant)
    }
}
```

Full execution in STORY-174. Mirrors VP-033 (ENIP carry isolation) pattern.

## Tasks

- [ ] Add `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>` to `Iec104FlowState` (initialized empty)
- [ ] Define `const MAX_IEC104_CARRY_BYTES: usize = 255;`
- [ ] Implement `Iec104Analyzer` struct with `flows: HashMap<FlowId, Iec104FlowState>`
- [ ] Implement `Iec104Analyzer::on_data(&mut self, data: &[u8], dir: Direction, flow_id: FlowId)`:
  - Prepend directional carry to data
  - Frame-walk loop per ADR-013 Decision 3:
    - Bad start byte → advance 1 (carry NOT cleared)
    - Malformed LEN → advance 2
    - Valid frame → parse + dispatch (calling STORY-167/168/169/170/171 fns) → advance LEN+2
    - Insufficient → stash to carry (capped at 255); return
  - Carry overflow → emit T0814 + discard excess (BC-2.19.025)
- [ ] Implement `Iec104Analyzer::on_flow_close(&mut self, flow_id: FlowId)`:
  - `self.flows.remove(&flow_id)` (state dropped; carry freed) (BC-2.19.027)
- [ ] Write VP-045 proptest skeletons in `tests/iec104_analyzer_tests.rs`
- [ ] Write unit tests: one per AC, named `test_BC_2_19_025_*`, `test_BC_2_19_026_*`, `test_BC_2_19_027_*`
- [ ] Verify `cargo test` passes

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.025 | Carry + new = 255 (boundary) | Carry extended to exactly 255; no T0814 |
| EC-002 | BC-2.19.025 | Carry + new = 256 (1 over cap) | T0814 emitted; bytes discarded |
| EC-003 | BC-2.19.026 | Empty data slice | No processing; carry unchanged; no panic |
| EC-004 | BC-2.19.026 | Bad start byte mid-stream | Advance 1 byte; carry NOT cleared; continue scanning |
| EC-005 | BC-2.19.026 | Malformed LEN byte | Advance 2-byte APCI stub; continue |
| EC-006 | BC-2.19.026 | 3 complete frames back-to-back | All 3 processed sequentially |
| EC-007 | BC-2.19.027 | on_flow_close with non-empty carry | Carry silently discarded; no finding |
| EC-008 | BC-2.19.027 | on_flow_close for unknown flow_id | No-op; no panic |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| BC-2.19.025–027 (3 BCs × ~800 each) | ~2,400 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (from STORY-171) | ~10,000 |
| Test file delta (unit + proptest) | ~2,500 |
| TOTAL | ~38,400 |

Agent context window ~200k tokens. This story uses ~19% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-170 (control command detection) + STORY-171 (sequence tracking)
- STORY-170 completed the TypeID detection layer; STORY-171 added Option<u16> sequence state
- This story adds the outer infrastructure that CALLS all prior fns: `on_data` frame-walk loop
- `Iec104FlowState` now has all 5 fields: `carry_c2s`, `carry_s2c`, `session_started`,
  `last_ns_c2s: Option<u16>`, `last_ns_s2c: Option<u16>`
- ADR-013 Decision 3: bad-start-byte does NOT clear carry — this is a recovery scan
  (advancing 1 byte to find the next 0x68 candidate); the carry stays intact

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 2**: `MAX_IEC104_CARRY_BYTES = 255` (max APCI frame = 255 bytes; carry ≥ 255
  is impossible without malformed input or attack).
- **ADR-013 Decision 3**: Frame-walk advance modes: bad-start-byte → +1/no-carry-clear;
  malformed-LEN → +2-stub; valid → LEN+2; insufficient → stash+return. This exact enumeration
  MUST be implemented; no other advance mode is valid.
- **ADR-013 Decision 8**: `on_data` is the effectful shell — VP-047 fuzz target. `parse_apci_header`
  is VP-044 Kani target. These scopes MUST NOT be conflated.
- **RULING-DNP3-SIBLING-001**: Carry buffers are directionally isolated — matches DNP3/ENIP/TLS
  pattern. VP-045 proptest verifies.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ | `Vec<u8>`, `HashMap`, `Option<u16>` |
| proptest | latest (from Cargo.toml) | VP-045 direction isolation skeletons |
| cargo-fuzz | latest | VP-047 fuzz harness (full run in STORY-174) |

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add `Iec104Analyzer` struct + `flows: HashMap`; add carry fields to `Iec104FlowState`; implement `on_data` frame-walk loop; implement `on_flow_close`; define `MAX_IEC104_CARRY_BYTES` |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.025–027 unit tests + VP-045 proptest skeletons |
| `fuzz/fuzz_targets/fuzz_iec104_parser.rs` | CREATE | VP-047 fuzz harness skeleton calling `on_data` |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- Carry buffers MUST NOT be merged: `carry_c2s` and `carry_s2c` must remain separate `Vec<u8>`
  fields in `Iec104FlowState` — a single shared carry Vec would violate RULING-DNP3-SIBLING-001
