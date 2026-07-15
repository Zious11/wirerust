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
version: "2.0"
modified:
  - "v2.0 BC-realignment per SR-172-01/02/03 (pre-delivery fidelity check, 3rd F3-DECOMPOSITION-BC-FIDELITY occurrence, 2026-07-15): FlowId->FlowKey; carry-overflow discard-all-new canonical vectors; malformed-LEN EMIT-WITH-DEDUP per BC-2.19.026 v1.6 + research validation; sibling on_data signature adopted; inputs completed."
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.001.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.002.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.003.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.004.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.005.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.006.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.025.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.026.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.027.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "af0f732"
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

### AC-172-002: Carry overflow emits T0814 and discards the entire incoming delivery
**Traces to:** BC-2.19.025 postconditions 1–3 and invariant 3
- Given a carry buffer holding N bytes and new bytes arriving such that N + new > 255
- When the frame-walk loop would stash the remaining bytes into carry
- Then a T0814 "Denial of Service" finding is emitted with
  `ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium`
- The carry buffer retains its PRIOR content unchanged — it is NOT extended, and excess bytes
  are NOT appended then truncated; the entire incoming delivery (the remainder bytes that
  would have been stashed) is discarded atomically
- EC-001 (exact-255, no overflow): carry is extended to exactly 255 bytes; no T0814 emitted
- Canonical overflow vectors (from BC-2.19.025):
  - C2S carry=1, new=255 (total 256): T0814 emitted; carry retains 1 byte; all 255 new bytes discarded
  - S2C carry=200, new=100 (total 300): T0814 emitted; carry retains 200 bytes; all 100 new bytes discarded

### AC-172-003: Frame-walk loop processes all complete APCI frames per on_data call
**Traces to:** BC-2.19.026 postconditions 1–3
- Given data containing multiple complete APCI frames (e.g., two back-to-back STARTDT frames)
- When `on_data(flow_key, data, ts, direction)` is called
- Then all complete frames are parsed and dispatched sequentially
- Remaining incomplete bytes are stashed into the directional carry buffer

### AC-172-004: Frame-walk loop terminates and handles advance modes including malformed-LEN EMIT-WITH-DEDUP
**Traces to:** BC-2.19.026 postcondition 4, invariants 1 and 5
- The loop always advances the cursor per iteration per ADR-013 Decision 3:
  - Bad start byte (data[pos] != 0x68): advance 1 byte; carry NOT cleared (resync scan);
    NO finding emitted
  - Malformed LEN (valid 0x68 start byte, LEN outside [4, 253]): advance 2 bytes (skip APCI
    stub); on the FIRST occurrence in a given flow direction, emit ONE T0814 finding with
    `ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium` and set the
    per-direction dedup flag (`malformed_len_reported_c2s` or `malformed_len_reported_s2c`)
    in `Iec104FlowState`; subsequent malformed-LEN in the same direction: advance-only, NO
    finding (silent resync); the dedup flag is never reset within a flow lifetime
  - Valid frame: advance LEN+2 bytes
  - Insufficient data (fewer than LEN+2 bytes remain after start): stash remaining to carry
    (subject to carry-overflow check per AC-172-002); return
- No infinite loop is possible for any finite input (VP-047 fuzz verifies this)

### AC-172-005: on_data does not panic for any byte sequence
**Traces to:** BC-2.19.026 postcondition 5 (VP-047 no-panic obligation)
- Given any arbitrary byte sequence delivered to `on_data`
- When `on_data(flow_key, data, ts, direction)` is called
- Then the function returns without panicking, unwinding, or accessing out-of-bounds memory
- This is the top-level VP-047 fuzz harness target: `fuzz_iec104_parser` calls `on_data` directly

### AC-172-006: on_flow_close removes state and discards carry bytes
**Traces to:** BC-2.19.027 postconditions 1–4 and invariants 1–2
- Given a flow with active `Iec104FlowState` (possibly with non-empty carry buffers)
- When `on_flow_close(flow_key)` is called
- Then `Iec104FlowState` for that flow is removed from the state map
- `carry_c2s` and `carry_s2c` are dropped (memory freed)
- No finding is emitted for normal flow close
- Calling `on_flow_close` for an already-removed flow_key is a no-op (no panic)

### AC-172-007: VP-045 proptest skeleton compiles — carry direction isolation
**Traces to:** BC-2.19.025 invariant 1 (VP-045 proptest obligation)
- Given the `proptest_vp045_direction_isolation` and `proptest_vp045_independent_run_equivalence`
  harnesses anchored in this story
- When the proptest skeletons are compiled
- Then they compile without error
- Full proptest runs are executed in STORY-174
- Mirrors VP-033 (ENIP), VP-035 (DNP3), VP-037 (Modbus) pattern

### AC-172-008: Malformed-LEN dedup per direction — concrete test expectations
**Traces to:** BC-2.19.026 invariant 5 and EC-006/007/008
- Given a flow where C2S direction receives a first malformed-LEN frame (valid 0x68, LEN=3):
  - Test `test_BC_2_19_026_malformed_len_first_c2s`: cursor advances 2 bytes; exactly ONE
    T0814 finding is emitted with `ThreatCategory::Anomaly / Verdict::Possible /
    Confidence::Medium`; `malformed_len_reported_c2s` flag is set to true
- Given the same flow subsequently receives a second C2S malformed-LEN frame (flag already set):
  - Test `test_BC_2_19_026_malformed_len_second_c2s`: cursor advances 2 bytes; NO finding
    emitted; `malformed_len_reported_c2s` remains true
- Given the same flow then receives a FIRST S2C malformed-LEN frame:
  - Test `test_BC_2_19_026_malformed_len_first_s2c_after_c2s`: cursor advances 2 bytes;
    exactly ONE T0814 finding emitted independently for S2C;
    `malformed_len_reported_s2c` flag is set to true; `malformed_len_reported_c2s` unchanged

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `carry_c2s: Vec<u8>` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
| `carry_s2c: Vec<u8>` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
| `malformed_len_reported_c2s: bool` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
| `malformed_len_reported_s2c: bool` | SS-19 per-flow state | `src/analyzer/iec104.rs` | Mutable state |
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
        let flow_key = FlowKey::new(
            "127.0.0.1".parse().unwrap(), 1234,
            "127.0.0.2".parse().unwrap(), 2404,
        );
        // Interleaved C2S and S2C deliveries must not mix carries
        analyzer.on_data(flow_key.clone(), &c2s_data, 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &s2c_data, 0, Direction::ServerToClient);
        // carry_c2s must only contain bytes from c2s_data path
        // carry_s2c must only contain bytes from s2c_data path
        // (proptest verifies isolation invariant)
    }
}
```

Full execution in STORY-174. Mirrors VP-033 (ENIP carry isolation) pattern.

## Tasks

- [ ] Add `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>` to `Iec104FlowState` (initialized empty)
- [ ] Add `malformed_len_reported_c2s: bool` and `malformed_len_reported_s2c: bool` to
      `Iec104FlowState` (initialized false); these per-direction dedup flags prevent T0814
      re-emission for malformed-LEN after the first occurrence per direction (BC-2.19.026
      invariant 5 / SR-172-03 EMIT-WITH-DEDUP)
- [ ] Define `const MAX_IEC104_CARRY_BYTES: usize = 255;`
- [ ] Implement `Iec104Analyzer` struct with `flows: HashMap<FlowKey, Iec104FlowState>`
- [ ] Implement `Iec104Analyzer::on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction)`:
  - Prepend directional carry to data
  - Frame-walk loop per ADR-013 Decision 3:
    - Bad start byte → advance 1 (carry NOT cleared; NO finding)
    - Malformed LEN (valid 0x68, LEN outside [4, 253]) → advance 2; emit ONE T0814
      (`ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium`) on first occurrence
      per direction and set `malformed_len_reported_{c2s,s2c}`; subsequent: advance-only, no finding
    - Valid frame → parse + dispatch (calling STORY-167/168/169/170/171 fns) → advance LEN+2
    - Insufficient → check carry overflow: if carry.len() + rem.len() > 255, retain prior
      carry, emit T0814, discard rem; else extend carry; return
  - Carry overflow semantics: retain prior carry unchanged + emit T0814 + discard all new bytes
    (BC-2.19.025 canonical vectors)
- [ ] Implement `Iec104Analyzer::on_flow_close(&mut self, flow_key: FlowKey)`:
  - `self.flows.remove(&flow_key)` (state dropped; carry freed) (BC-2.19.027)
- [ ] Write VP-045 proptest skeletons in `tests/iec104_analyzer_tests.rs`
- [ ] Write unit tests: one per AC, named `test_BC_2_19_025_*`, `test_BC_2_19_026_*`,
      `test_BC_2_19_027_*`; include `test_BC_2_19_026_malformed_len_first_c2s`,
      `test_BC_2_19_026_malformed_len_second_c2s`,
      `test_BC_2_19_026_malformed_len_first_s2c_after_c2s` (AC-172-008 dedup tests)
- [ ] Verify `cargo test` passes

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.025 | Carry + new = 255 (boundary) | Carry extended to exactly 255; no T0814 |
| EC-002 | BC-2.19.025 | C2S carry=1, new=255 (total 256, 1 over cap) | T0814 emitted; carry retains prior 1 byte unchanged; all 255 new bytes discarded |
| EC-003 | BC-2.19.025 | S2C carry=200, new=100 (total 300 over cap) | T0814 emitted; carry_s2c retains prior 200 bytes; all 100 new bytes discarded; carry_c2s unaffected |
| EC-004 | BC-2.19.026 | Empty data slice | No processing; carry unchanged; no panic |
| EC-005 | BC-2.19.026 | Bad start byte mid-stream | Advance 1 byte; carry NOT cleared; continue scanning; NO finding |
| EC-006 | BC-2.19.026 | First malformed-LEN frame (valid 0x68, LEN=3) in C2S direction | 2-byte advance; ONE T0814 (`ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium`) emitted; `malformed_len_reported_c2s` flag set |
| EC-007 | BC-2.19.026 | Second malformed-LEN frame in same C2S direction (flag already set) | 2-byte advance; NO finding (silent resync) |
| EC-008 | BC-2.19.026 | First malformed-LEN frame in S2C direction after C2S flag already set | 2-byte advance; ONE T0814 emitted independently for S2C; `malformed_len_reported_s2c` flag set; C2S flag unchanged |
| EC-009 | BC-2.19.026 | 3 complete frames back-to-back | All 3 processed sequentially |
| EC-010 | BC-2.19.027 | on_flow_close with non-empty carry | Carry silently discarded; no finding |
| EC-011 | BC-2.19.027 | on_flow_close for unknown flow_key | No-op; no panic |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| BC-2.19.025–027 (3 BCs × ~800 each) | ~2,400 |
| BC-2.19.001–006 (6 BCs × ~500 each) | ~3,000 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (from STORY-171) | ~10,000 |
| Test file delta (unit + proptest) | ~2,500 |
| TOTAL | ~41,400 |

Agent context window ~200k tokens. This story uses ~21% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-170 (control command detection) + STORY-171 (sequence tracking)
- STORY-170 completed the TypeID detection layer; STORY-171 added Option<u16> sequence state
- This story adds the outer infrastructure that CALLS all prior fns: `on_data` frame-walk loop
- `Iec104FlowState` now has all 7 fields: `carry_c2s`, `carry_s2c`, `session_started`,
  `last_ns_c2s: Option<u16>`, `last_ns_s2c: Option<u16>`, `malformed_len_reported_c2s: bool`,
  `malformed_len_reported_s2c: bool`
- ADR-013 Decision 3: bad-start-byte does NOT clear carry — this is a recovery scan
  (advancing 1 byte to find the next 0x68 candidate); the carry stays intact
- Malformed-LEN EMIT-WITH-DEDUP (SR-172-03): per-direction dedup flags prevent T0814 spam
  on persistent malformed-LEN injection; research-validated against CVE-2023-5768, Snort3
  IEC104_BAD_LENGTH, Wireshark iec104.apdu_invalid_len, and Zeek weird+sampling
  (see `.factory/cycles/feature-iec104/research/sr-172-03-malformed-len-validation.md`)

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 2**: `MAX_IEC104_CARRY_BYTES = 255` (max APCI frame = 255 bytes; carry ≥ 255
  is impossible without malformed input or attack).
- **ADR-013 Decision 3**: Frame-walk advance modes: bad-start-byte → +1/no-carry-clear/no-finding;
  malformed-LEN → +2-stub + EMIT-WITH-DEDUP (T0814 on first occurrence per direction only,
  `ThreatCategory::Anomaly / Verdict::Possible / Confidence::Medium`); valid → LEN+2;
  insufficient → stash+return. This exact enumeration MUST be implemented; no other advance
  mode is valid.
- **ADR-013 Decision 8**: `on_data` is the effectful shell — VP-047 fuzz target. `parse_apci_header`
  is VP-044 Kani target. These scopes MUST NOT be conflated.
- **RULING-DNP3-SIBLING-001**: Carry buffers are directionally isolated — matches DNP3/ENIP/TLS
  pattern. VP-045 proptest verifies.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ | `Vec<u8>`, `HashMap`, `Option<u16>`, `bool` |
| proptest | latest (from Cargo.toml) | VP-045 direction isolation skeletons |
| cargo-fuzz | latest | VP-047 fuzz harness (full run in STORY-174) |

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add `Iec104Analyzer` struct + `flows: HashMap`; add carry fields and dedup-flag fields to `Iec104FlowState`; implement `on_data` frame-walk loop with EMIT-WITH-DEDUP; implement `on_flow_close`; define `MAX_IEC104_CARRY_BYTES` |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.025–027 unit tests + AC-172-008 dedup tests + VP-045 proptest skeletons |
| `fuzz/fuzz_targets/fuzz_iec104_parser.rs` | CREATE | VP-047 fuzz harness skeleton calling `on_data` |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- Carry buffers MUST NOT be merged: `carry_c2s` and `carry_s2c` must remain separate `Vec<u8>`
  fields in `Iec104FlowState` — a single shared carry Vec would violate RULING-DNP3-SIBLING-001
