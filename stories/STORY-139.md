---
document_type: story
story_id: STORY-139
title: "ENIP Per-Direction Carry Buffer + Saturating Window Monotonicity (EC-X1/EC-X2 Detection-Correctness Fixes)"
epic_id: E-20
wave: 62
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-17]
target_module: analyzer/enip
depends_on: [STORY-138]
blocks: []
behavioral_contracts:
  - BC-2.17.016
  - BC-2.17.008
  - BC-2.17.012
  - BC-2.17.018
verification_properties:
  - VP-033
  - VP-034
assumption_validations: []
risk_mitigations: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.016.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.008.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.012.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.018.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
ruling: RULING-EDGECASE-001
input-hash: "759464a"
---

# STORY-139: ENIP Per-Direction Carry Buffer + Saturating Window Monotonicity (EC-X1/EC-X2 Detection-Correctness Fixes)

## Narrative

**As a** security analyst relying on wirerust ENIP/CIP detections for ICS threat detection,
**I want** the EtherNet/IP analyzer to correctly isolate carry-buffer state per TCP direction and
apply backwards-clock-safe window expiry arithmetic,
**so that** bidirectional ENIP flows do not produce phantom findings or suppress legitimate
detections (EC-X1), and adversarially injected out-of-order timestamps cannot abort
in-progress burst windows (EC-X2), unblocking v0.11.0 release.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.17.016 | v2.0 | Carry-Buffer Frame-Walk Loop — Partial Frame Stash and MAX_ENIP_CARRY_BYTES Cap | Per-direction carry split (`carry_c2s`/`carry_s2c`); `on_data` direction parameter; Invariant 7 direction isolation; EC-010 direction non-contamination |
| BC-2.17.008 | v1.3 | CIP Error Response Detection — general_status Extraction | Postcondition 4 window-expiry: `saturating_sub > 10`; EC-009 backwards-ts no-reset |
| BC-2.17.012 | v1.2 | CIP Write-Class Service Burst Exceeding Threshold Emits T0836 | Postcondition 4 window-expiry: `saturating_sub > 1`; EC-009 backwards-ts no-reset |
| BC-2.17.018 | v1.1 | Malformed ENIP Frame Threshold Emits T0814 Structural Anomaly | Postcondition 5 window-expiry: `saturating_sub > 300` (strict `>`; was `>= 300`); `malformed_window_start_ts` field name; EC-008 backwards-ts no-reset |

## Acceptance Criteria

### AC-139-001: `EnipFlowState.carry` split into per-direction buffers; `on_data` gains `direction: Direction`
**Traces to:** BC-2.17.016 v2.0 Precondition 1, Precondition 2, Postcondition 1 inner body, Postcondition 3, Postcondition 4, Invariant 7, EC-010

The single `carry: Vec<u8>` field is removed from `EnipFlowState`. Two separate fields are added:
- `carry_c2s: Vec<u8>` — partial frames from the TCP initiator (client→server direction)
- `carry_s2c: Vec<u8>` — partial frames from the TCP responder (server→client direction)

`EnipAnalyzer::on_data` gains a new `direction: Direction` parameter (type:
`crate::reassembly::handler::Direction`), mirroring the existing Modbus `StreamHandler`
signature:

```rust
// AFTER (BC-2.17.016 v2.0 Precondition 1):
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], timestamp: u32, direction: Direction)
```

Within the frame-walk loop:
- The directional carry buffer is selected at call entry:
  `let carry = match direction { ClientToServer => &mut flow.carry_c2s, ServerToClient => &mut flow.carry_s2c };`
- `buf = carry ++ data` is formed from the SAME directional carry (BC-2.17.016 v2.0 Precondition 2)
- After the loop, remaining partial-frame bytes are stashed back into the SAME directional carry
  (BC-2.17.016 v2.0 Postcondition 3)
- The carry-cap check (> MAX_ENIP_CARRY_BYTES = 600) applies to the active directional carry only;
  the other direction's carry is unaffected (BC-2.17.016 v2.0 Postcondition 4)
- Invariant 7: `carry_c2s` and `carry_s2c` are NEVER mixed. A partial c2s frame stashed in
  `carry_c2s` is NEVER prepended to an s2c delivery, and vice versa. (RULING-EDGECASE-001 §1.2)

All call sites for `on_data` in the stream dispatcher (ENIP arm) are updated to pass
`Direction` from the `StreamHandler` context — matching exactly the Modbus pattern.

**Test:** `tests/enip_analyzer_tests.rs::direction_and_clock::test_ec_x1_cross_direction_no_splice`
— Deliver partial c2s frame (24 bytes of a SendRRData declaring > 0 payload), then deliver a
complete s2c CIP error response frame. Assert `error_count == 1` (the s2c response was processed
correctly), `pdu_count == 1` (only the s2c PDU; the c2s partial is still pending in `carry_c2s`),
`parse_errors == 0`, `findings == 0` (threshold not crossed). This is the EC-X1 repro from
`tests/scratch_ecx1_ecx2_repro.rs::scratch_ecx1_direct_partial_request_then_response_pdu_count`
promoted to a permanent named regression test. (traces to BC-2.17.016 v2.0 Invariant 7, EC-010)

**Test:** `tests/enip_analyzer_tests.rs::direction_and_clock::test_carry_c2s_and_carry_s2c_are_independent`
— Deliver a partial c2s frame (carry_c2s stashed), then deliver a partial s2c frame (carry_s2c
stashed). Assert `carry_c2s.len() > 0 && carry_s2c.len() > 0` after both deliveries, and
that neither carry contains bytes from the other direction.

### AC-139-002: DRIFT-ENIP-DIRECTION-001 fix-along — direction-based source IP resolution
**Traces to:** RULING-EDGECASE-001 §1.4 (fix-along); BC-2.17.016 v2.0 Precondition 1

The `resolve_enip_client_ip` port-44818 heuristic is replaced with direction-aware source IP
resolution (Modbus pattern). When `Direction` is available in `on_data`, findings emit:
```rust
let src_ip = match direction {
    Direction::ClientToServer => flow_key.src_ip(),   // initiator (non-44818 side)
    Direction::ServerToClient => flow_key.dst_ip(),   // responder (44818 side)
};
```
This resolves DRIFT-ENIP-DIRECTION-001 as a no-cost fix-along since `Direction` is now
threaded. Findings emit `direction: Some(direction)` rather than `None` where applicable.

**Test:** `tests/enip_analyzer_tests.rs::direction_and_clock::test_direction_based_source_ip`
— Feed a c2s CIP Stop request on a flow where `src=10.0.0.1:54321`, `dst=10.0.0.2:44818`.
Assert the emitted T0858 finding has `source_ip == Some(10.0.0.1)` (the initiator IP),
not the 44818-port heuristic fallback.

### AC-139-003: All three window-expiry checks use `saturating_sub` (not `wrapping_sub`)
**Traces to:** BC-2.17.008 v1.3 Postcondition 4, BC-2.17.012 v1.2 Postcondition 4, BC-2.17.018 v1.1 Postcondition 5; RULING-EDGECASE-001 §2.2

Replace every `wrapping_sub` in the windowed expiry comparisons within `on_data` and
`process_pdu` with `saturating_sub`:

| Window | Before (broken) | After (fixed) |
|--------|-----------------|---------------|
| T0836 write-burst (BC-2.17.012) | `now_ts.wrapping_sub(flow.write_window_start_ts) > 1` | `now_ts.saturating_sub(flow.write_window_start_ts) > 1` |
| T0888 error-rate (BC-2.17.008) | `now_ts.wrapping_sub(flow.error_window_start_ts) > 10` | `now_ts.saturating_sub(flow.error_window_start_ts) > 10` |
| T0814 malformed (BC-2.17.018) | `timestamp.wrapping_sub(flow.malformed_window_start) >= 300` | `timestamp.saturating_sub(flow.malformed_window_start_ts) > 300` |

Semantics: when `now_ts < window_start_ts` (backwards/out-of-order timestamp),
`saturating_sub` returns 0, which is NOT > any positive threshold. The window is
preserved. An adversary injecting a single old-timestamp packet cannot abort a
burst-in-progress. (RULING-EDGECASE-001 §2.2)

The `malformed_window_start_ts` field name (note `_ts` suffix) is used throughout,
matching BC-2.17.018 v1.1 Postcondition 5 and the F2-correction F-006 rename. The
`EnipFlowState` field MUST be `malformed_window_start_ts: u32`, NOT `malformed_window_start`.

**Test:** `tests/enip_analyzer_tests.rs::direction_and_clock::test_ec_x2_backwards_ts_t0836_no_reset`
— 50 writes at ts=100 (window_start=100, count=50), then 1 write at ts=50 (backwards), then
1 write at ts=100. Assert T0836 fires (write_count_in_window >= 51 > threshold=50). This is
the EC-X2 repro from `tests/scratch_ecx1_ecx2_repro.rs::scratch_ecx2_write_burst_backwards_timestamp_suppresses_detection`
promoted to a permanent named regression test. (traces to BC-2.17.012 v1.2 Postcondition 4, EC-009)

**Test:** `tests/enip_analyzer_tests.rs::direction_and_clock::test_ec_x2_backwards_ts_t0888_no_reset`
— 5 errors at ts=100 (window_start=100), then 1 error at ts=50 (backwards), then 1 error at
ts=100. Assert T0888 fires. Promoted from
`tests/scratch_ecx1_ecx2_repro.rs::scratch_ecx2_error_burst_backwards_timestamp_suppresses_detection`.
(traces to BC-2.17.008 v1.3 Postcondition 4, EC-009)

**Test:** `tests/enip_analyzer_tests.rs::direction_and_clock::test_ec_x2_backwards_ts_t0814_no_reset`
— 2 malformed frames at ts=100 (malformed_window_start_ts=100, malformed_in_window=2), then
1 malformed frame at ts=50 (backwards). Assert T0814 fires (malformed_in_window=3 ≥ threshold=3).
(traces to BC-2.17.018 v1.1 Postcondition 5, EC-008)

### AC-139-004: Malformed window operator pinned to strict `> 300` (EC-X4)
**Traces to:** BC-2.17.018 v1.1 Postcondition 5, Invariant 2; RULING-EDGECASE-001 §2.4

The malformed-window expiry condition is `timestamp.saturating_sub(flow.malformed_window_start_ts) > 300`
(strict greater-than), NOT `>= 300`. This is consistent with the T0836 (`> 1`) and T0888 (`> 10`)
window semantics: the packet arriving at exactly `elapsed == 300` seconds is the LAST packet
of the current window, not the first of a new one. (EC-X4 operator pinning per RULING-EDGECASE-001 §2.4)

**Test:** `tests/enip_analyzer_tests.rs::direction_and_clock::test_malformed_window_operator_pin_boundary`
— Deliver 3 malformed frames to arm the T0814 guard (guard set, malformed_anomaly_emitted=true).
Advance time to exactly `elapsed = 300` seconds after window_start. Deliver 1 more malformed frame.
Assert the window is NOT reset at elapsed=300 (the packet is still in-window); assert
`malformed_anomaly_emitted` is still true (no new T0814 — guard was set, window not reset).
Advance to elapsed=301. Assert the window IS reset (new window, `malformed_anomaly_emitted=false`).
(traces to BC-2.17.018 v1.1 Invariant 2)

### AC-139-005: VP-033 proptest (carry direction isolation) and VP-034 proptest (window monotonic) implemented
**Traces to:** VP-033 (BC-2.17.016 v2.0 Invariant 7); VP-034 (BC-2.17.008 v1.3 PC-4, BC-2.17.012 v1.2 PC-4, BC-2.17.018 v1.1 PC-5)

The VP-033 and VP-034 proof harness skeletons from the VP files are implemented as
passing proptest suites. These tests serve as regression guards preventing future
refactors from breaking direction isolation or window monotonicity.

**VP-033 tests** (mod `vp033_carry_direction_isolation` in `tests/enip_analyzer_tests.rs`):
- `proptest_vp033_direction_isolation_pdu_count` — generates `split_offset in 1..23` and
  `s2c_cmd in [0x0065, 0x0066, 0x0063]`; verifies interleaved partial-c2s + full-s2c deliveries
  produce `pdu_count == 2` and `parse_errors == 0`; verifies both carries drained after completion.
- `proptest_vp033_independent_run_equivalence` — verifies interleaved run `pdu_count` and
  `parse_errors` equal the sum of independent same-direction control runs.

**VP-034 tests** (mod `vp034_window_monotonic_no_spurious_reset` in `tests/enip_analyzer_tests.rs`):
- `proptest_vp034_sub_a_write_burst_backwards_ts_no_reset` — T0836 window; proptest over
  `(window_start, threshold, backwards_ts)` with `prop_assume!(backwards_ts <= window_start)`;
  verifies `saturating_sub(backwards_ts, window_start) == 0` → elapsed NOT > 1 → no reset.
- `proptest_vp034_sub_a_ec_x2_repro_t0836` — Direct EC-X2 T0836 repro: asserts
  `50u32.saturating_sub(100) == 0` (not the wrapping ~4.29e9 that caused the bug).
- `proptest_vp034_sub_b_error_rate_backwards_ts_no_reset` — T0888 10s window; same pattern.
- `proptest_vp034_sub_c_malformed_window_backwards_ts_no_reset` — T0814 300s window; includes
  `prop_assume!(backwards_ts <= window_start)`.
- `proptest_vp034_sub_c_malformed_window_operator_pin` — Boundary test: `elapsed==300` does NOT
  expire under strict `>`; `elapsed==301` DOES expire. (EC-X4 operator pin)
- `test_vp034_sub_d_genuine_rollover_no_spurious_reset` — Deterministic unit test: `window_start =
  u32::MAX - 5`, `now_ts = 4`; `wrapping_sub` would give 10 (old spurious reset); `saturating_sub`
  gives 0 (no spurious reset). Documents the old vs. new behavior.

(traces to VP-033 harness spec; VP-034 Sub-A/B/C/D harness spec)

## Architecture Mapping

| Component | Location | Role | Pure/Effectful |
|-----------|----------|------|----------------|
| `EnipFlowState.carry_c2s: Vec<u8>` | `src/analyzer/enip.rs` | Per-direction carry buffer (c2s); replaces `carry: Vec<u8>` | Effectful (mutated by on_data) |
| `EnipFlowState.carry_s2c: Vec<u8>` | `src/analyzer/enip.rs` | Per-direction carry buffer (s2c); replaces `carry: Vec<u8>` | Effectful (mutated by on_data) |
| `EnipFlowState.malformed_window_start_ts: u32` | `src/analyzer/enip.rs` | Malformed-window start timestamp (renamed from `malformed_window_start` — BC-2.17.018 v1.1 F-006) | Effectful |
| `EnipAnalyzer::on_data(flow_key, data, timestamp, direction)` | `src/analyzer/enip.rs` | Frame-walk entry point; `direction: Direction` added | Effectful shell |
| `crate::reassembly::handler::Direction` | `src/reassembly/handler.rs` | Direction enum (already used by Modbus); imported by ENIP | Pure enum |
| Stream dispatcher ENIP arm | `src/reassembly/stream_dispatcher.rs` | Call site update: pass `Direction` to `on_data` (Modbus pattern) | Effectful |
| `tests/enip_analyzer_tests.rs` | `tests/enip_analyzer_tests.rs` | Test mod `mod direction_and_clock { ... }` + `mod vp033_carry_direction_isolation { ... }` + `mod vp034_window_monotonic_no_spurious_reset { ... }` | Test |

**Subsystem anchor:** SS-17 owns this story's scope because the carry-buffer split, direction
threading, and window expiry fixes are all localized to `src/analyzer/enip.rs` and its dispatch
call site in SS-05 (stream_dispatcher.rs). Per ARCH-INDEX.md §SS-17.

**Forbidden dependencies:** `src/analyzer/enip.rs` MUST NOT depend on any other analyzer module
(`modbus`, `dnp3`, `arp`, etc.). If this module gains a cross-analyzer dependency (other than
shared types from `src/reassembly/`), the build MUST fail. The `Direction` type is shared
infrastructure from `src/reassembly/handler.rs`, NOT an analyzer dependency.

**NOT in scope (DNP3 sibling deferral per RULING-EDGECASE-001 §1.6 and §2.5):**
- `Dnp3FlowState.carry` per-direction split is deferred to v0.12.0 (DRIFT-DNP3-DIRECTION-001).
- DNP3 window `wrapping_sub` → `saturating_sub` is deferred to v0.12.0 (DRIFT-DNP3-CLOCK-001).
- Human confirmation of this scope call required before v0.11.0 release notes are finalized.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Partial c2s frame stashed in `carry_c2s`; next call is s2c with full response | `carry_s2c` (empty) prepended to s2c data; s2c frame processes cleanly; `carry_c2s` retains partial bytes unchanged — no splice (BC-2.17.016 v2.0 EC-010) |
| EC-002 | Both directions have partial frames stashed simultaneously | `carry_c2s.len() > 0` AND `carry_s2c.len() > 0` concurrently; each direction's next delivery completes its own frame independently |
| EC-003 | Carry-cap overflow in c2s direction | `is_non_enip = true`; `carry_c2s` cleared or bounded; `carry_s2c` unaffected (BC-2.17.016 v2.0 Postcondition 4) |
| EC-004 | 50 writes at ts=100, then 1 write at ts=50 (backwards), then 1 write at ts=100 | `saturating_sub(50, 100) = 0`; window NOT reset; `write_count_in_window = 51`; T0836 fires (BC-2.17.012 v1.2 EC-009) |
| EC-005 | 5 errors at ts=100, then 1 error at ts=50 (backwards), then 1 error at ts=100 | `saturating_sub(50, 100) = 0`; window NOT reset; error_count_in_window accumulated; T0888 fires (BC-2.17.008 v1.3 EC-009) |
| EC-006 | 2 malformed frames at ts=100, then 1 malformed at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; NOT > 300; window NOT reset; `malformed_in_window = 3`; T0814 fires (BC-2.17.018 v1.1 EC-008) |
| EC-007 | Genuine u32 rollover: `window_start = u32::MAX - 5`, `now_ts = 4` | `saturating_sub(4, u32::MAX-5) = 0`; no spurious reset (vs. old `wrapping_sub` which gave 10 and triggered false reset) |
| EC-008 | T0814 malformed window: `elapsed == 300` (exactly at threshold) | Window NOT expired under strict `> 300` (EC-X4 pin); packet is last of current window (BC-2.17.018 v1.1 Invariant 2) |
| EC-009 | T0814 malformed window: `elapsed == 301` | Window IS expired; `malformed_in_window` reset; new window seeded |
| EC-010 | Existing ENIP tests (all `mod` blocks in `tests/enip_analyzer_tests.rs`) | All existing tests pass with the new `on_data(flow_key, data, timestamp, direction)` signature — no regressions |

## Tasks

- [ ] In `EnipFlowState`: remove `carry: Vec<u8>`; add `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`
- [ ] In `EnipFlowState`: rename `malformed_window_start: u32` → `malformed_window_start_ts: u32`
  (BC-2.17.018 v1.1 F-006 rename; update all references in on_data and process_pdu)
- [ ] Add `direction: Direction` parameter to `EnipAnalyzer::on_data`; update signature throughout
- [ ] Import `crate::reassembly::handler::Direction` in `enip.rs` (already imported in `modbus.rs`)
- [ ] In the frame-walk loop:
  - [ ] Select directional carry at entry: `let carry = if direction == Direction::ClientToServer { &mut flow.carry_c2s } else { &mut flow.carry_s2c };`
  - [ ] Build `buf = carry.drain(..).chain(data.iter().copied()).collect()` (or equivalent)
  - [ ] After loop, stash remaining bytes into the SAME directional carry
  - [ ] Cap check: `active_carry.len() > MAX_ENIP_CARRY_BYTES` → `is_non_enip = true`, `parse_errors += 1`, clear active carry
- [ ] Replace `resolve_enip_client_ip` port-44818 heuristic with direction-based IP selection:
  `let src_ip = match direction { ClientToServer => flow_key.src_ip(), ServerToClient => flow_key.dst_ip() };`
  Update all finding emission paths to use this `src_ip`.
- [ ] Replace ALL `wrapping_sub` with `saturating_sub` in window expiry comparisons (3 sites):
  - [ ] T0836 write-burst: `now_ts.saturating_sub(flow.write_window_start_ts) > 1`
  - [ ] T0888 error-rate: `now_ts.saturating_sub(flow.error_window_start_ts) > 10`
  - [ ] T0814 malformed: `timestamp.saturating_sub(flow.malformed_window_start_ts) > 300` (strict `>`, was `>=`)
- [ ] Update stream dispatcher ENIP arm call site to pass `Direction` (mirror Modbus arm)
- [ ] Update all existing tests in `tests/enip_analyzer_tests.rs` that call `on_data` to pass
  a `Direction` argument — use `Direction::ClientToServer` for c2s traffic, `Direction::ServerToClient`
  for s2c traffic (existing tests that don't test direction semantics can pick either)
- [ ] Add `mod direction_and_clock` to `tests/enip_analyzer_tests.rs` with AC-139-001..004 tests
- [ ] Add `mod vp033_carry_direction_isolation` and `mod vp034_window_monotonic_no_spurious_reset`
  (VP-033 and VP-034 harnesses from the VP spec files)
- [ ] Run `cargo test enip` — all direction_and_clock + VP proptest tests pass
- [ ] Run `cargo test --all-targets` — full test suite green (no regressions)
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings
- [ ] Run `cargo fmt --check` — zero format drift
- [ ] Run `bin/compute-input-hash --write .factory/stories/STORY-139.md` to populate `input-hash`

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`

**New test modules added by this story:**

```
mod direction_and_clock {
    test_ec_x1_cross_direction_no_splice
    test_carry_c2s_and_carry_s2c_are_independent
    test_direction_based_source_ip
    test_ec_x2_backwards_ts_t0836_no_reset
    test_ec_x2_backwards_ts_t0888_no_reset
    test_ec_x2_backwards_ts_t0814_no_reset
    test_malformed_window_operator_pin_boundary
}

mod vp033_carry_direction_isolation {
    proptest_vp033_direction_isolation_pdu_count
    proptest_vp033_independent_run_equivalence
}

mod vp034_window_monotonic_no_spurious_reset {
    proptest_vp034_sub_a_write_burst_backwards_ts_no_reset
    proptest_vp034_sub_a_ec_x2_repro_t0836
    proptest_vp034_sub_b_error_rate_backwards_ts_no_reset
    proptest_vp034_sub_c_malformed_window_backwards_ts_no_reset
    proptest_vp034_sub_c_malformed_window_operator_pin
    test_vp034_sub_d_genuine_rollover_no_spurious_reset
}
```

**Scratch repro basis:** The EC-X1 and EC-X2 scratch tests in
`tests/scratch_ecx1_ecx2_repro.rs` (in `.worktrees/enip-edgecase-verify/`) are the
empirical evidence for the bugs. The promoted regression tests above encode their
core assertions as permanent named tests in the main test suite:
- `test_ec_x1_cross_direction_no_splice` ← `scratch_ecx1_direct_partial_request_then_response_pdu_count`
- `test_ec_x2_backwards_ts_t0836_no_reset` ← `scratch_ecx2_write_burst_backwards_timestamp_suppresses_detection`
- `test_ec_x2_backwards_ts_t0888_no_reset` ← `scratch_ecx2_error_burst_backwards_timestamp_suppresses_detection`

**Updated existing tests:** All existing `on_data(key, data, ts)` call sites in
`tests/enip_analyzer_tests.rs` updated to `on_data(key, data, ts, Direction::ClientToServer)`
or `Direction::ServerToClient` as appropriate.

**TDD discipline (strict mode):** Implementer writes `todo!()` stubs for all new
`on_data` parameter changes and field renames first, then drives tests RED (existing tests
fail due to signature change), then implements the direction split and saturating_sub fixes
to turn them GREEN one-by-one.

## Previous Story Intelligence

- STORY-137 introduced the frame-walk loop, `carry: Vec<u8>`, and the `malformed_window_start`
  field (now renamed to `malformed_window_start_ts` by BC-2.17.018 v1.1 F-006). STORY-139
  replaces `carry` with `carry_c2s`/`carry_s2c`.
- STORY-134 introduced `error_window_start_ts`, `error_window_active`, and the `wrapping_sub`
  error-rate window expiry that is fixed here.
- STORY-135 introduced `write_window_start_ts` and the `wrapping_sub` write-burst window expiry
  that is fixed here.
- STORY-138 completed the full ENIP analyzer (summarize, on_flow_close). All Wave 60+61 stories
  are now merged. STORY-139 is the post-convergence bug-fix story applying RULING-EDGECASE-001.
- The `on_data` call sites in `stream_dispatcher.rs` currently pass 3 arguments; this story
  adds a 4th (`Direction`). Update the dispatcher ENIP arm to match the Modbus pattern.
- The field `malformed_window_start` appears in STORY-137; BC-2.17.018 v1.1 (F-006 correction)
  renames it to `malformed_window_start_ts`. Implementer must locate every reference to
  `malformed_window_start` in `enip.rs` and rename to `malformed_window_start_ts`.

## Architecture Compliance Rules

From ADR-010 Decision 4 (amended by RULING-EDGECASE-001), BC-2.17.016 v2.0, BC-2.17.008 v1.3,
BC-2.17.012 v1.2, BC-2.17.018 v1.1:

1. **`carry: Vec<u8>` is REMOVED (BC-2.17.016 v2.0 §1.3):** Any code referencing `flow.carry`
   (singular) after this story is a regression. The fields are `carry_c2s` and `carry_s2c`.
2. **Direction isolation is structurally enforced (BC-2.17.016 v2.0 Invariant 7):** The match arm
   `match direction { ClientToServer => ..., ServerToClient => ... }` selects exactly one of the
   two carry buffers. No conditional path may read `carry_c2s` when processing `ServerToClient`
   or read `carry_s2c` when processing `ClientToServer`.
3. **`saturating_sub` is the ONLY permitted window arithmetic (RULING-EDGECASE-001 §2.2):**
   `wrapping_sub` MUST NOT appear in any window-expiry comparison in `enip.rs`. Use
   `now_ts.saturating_sub(window_start_ts)` in all three window paths.
4. **Strict `>` for ALL windows (RULING-EDGECASE-001 §2.4):** The T0814 window is pinned to
   strict `> 300` (was `>= 300`). All three windows use strict `>`. No window uses `>=`.
5. **`malformed_window_start_ts` spelling (BC-2.17.018 v1.1 F-006):** The field MUST be
   `malformed_window_start_ts`, not `malformed_window_start`. This matches the existing
   `write_window_start_ts` and `error_window_start_ts` naming convention.
6. **Detection counters remain PER-FLOW (RULING-EDGECASE-001 §1.3):** `command_counts`,
   `parse_errors`, `write_count_in_window`, `error_counts_in_window`, `malformed_in_window`,
   `pdu_count` and all other non-carry fields remain shared per flow (not split by direction).
   ONLY `carry_c2s`/`carry_s2c` are per-direction.
7. **DNP3 sibling fix is OUT OF SCOPE (RULING-EDGECASE-001 §1.6 / §2.5):** Do NOT touch
   `dnp3.rs` carry or window arithmetic. DNP3 direction threading is a v0.12.0 item
   (DRIFT-DNP3-DIRECTION-001 / DRIFT-DNP3-CLOCK-001).
8. **`resolve_enip_client_ip` is REMOVED (RULING-EDGECASE-001 §1.4):** The port-44818 heuristic
   function is replaced by direct direction-based IP selection inline in each finding emission.
   Do NOT retain `resolve_enip_client_ip` as dead code.

## Library & Framework Requirements

- `crate::reassembly::handler::Direction` — existing type; no new external dependency
- `proptest` crate — already used in the test suite (VP-033/VP-034 harnesses use `proptest::prelude::*`)
- No new `Cargo.toml` dependencies

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs`
  - Remove `carry: Vec<u8>` from `EnipFlowState`
  - Add `carry_c2s: Vec<u8>`, `carry_s2c: Vec<u8>` to `EnipFlowState`
  - Rename `malformed_window_start` → `malformed_window_start_ts` throughout
  - Update `on_data` signature: add `direction: Direction` parameter
  - Import `crate::reassembly::handler::Direction`
  - Replace `resolve_enip_client_ip` with inline direction-based IP selection
  - Replace all `wrapping_sub` with `saturating_sub` in window-expiry paths
  - Pin T0814 malformed window operator: `>= 300` → `> 300`
- `src/reassembly/stream_dispatcher.rs`
  - Update ENIP arm call site: `enip_analyzer.on_data(flow_key, data, ts)` →
    `enip_analyzer.on_data(flow_key, data, ts, direction)` (Modbus pattern)
- `tests/enip_analyzer_tests.rs`
  - Update all existing `on_data(key, data, ts)` call sites to pass `Direction`
  - Add `mod direction_and_clock { ... }` with AC-139-001..004 named tests
  - Add `mod vp033_carry_direction_isolation { ... }` with VP-033 proptest harnesses
  - Add `mod vp034_window_monotonic_no_spurious_reset { ... }` with VP-034 proptest harnesses

**Files that may need minor touches:**
- `tests/scratch_ecx1_ecx2_repro.rs` (in worktree) — can be superseded by the permanent
  regression tests in `enip_analyzer_tests.rs`; scratch file may be removed from the
  production test suite (or gated with `#[cfg(test_scratch)]`)

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` changes (carry split, field rename, `on_data` sig, direction IP, saturating_sub, operator pin) | ~800 |
| `src/reassembly/stream_dispatcher.rs` ENIP arm call-site update | ~50 |
| Existing test updates (add `Direction` arg to ~30 call sites across test modules) | ~200 |
| New `mod direction_and_clock` tests (7 tests) | ~500 |
| `mod vp033_carry_direction_isolation` proptest harnesses (2 proptests + helper) | ~350 |
| `mod vp034_window_monotonic_no_spurious_reset` proptest + deterministic tests (6 tests) | ~400 |
| BC files (4 BCs) | ~800 |
| **Total** | **~3,100** |

Context utilization: ~3,100 tokens / ~200,000 token window = ~1.5%. Well within the 20-30%
per-story budget.

## Dependency Rationale

Wave 62 (new wave added for this post-convergence fix story). Depends on STORY-138 (all
Wave 61 ENIP analyzer work complete). This story is the sole fix story for RULING-EDGECASE-001
and unblocks v0.11.0 release. No story depends on STORY-139 yet (end of the E-20 chain).

**STORY-139 depends on STORY-138 because:** STORY-138 completed the full `EnipFlowState`
and `EnipAnalyzer` struct definitions and `on_data` frame-walk loop that this story
modifies. The `carry: Vec<u8>` field being replaced was added in STORY-137; STORY-138
is the last story in the dependency chain and must be merged before STORY-139 touches
any of these fields.
