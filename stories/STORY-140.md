---
document_type: story
story_id: STORY-140
title: "DNP3 Per-Direction Carry Buffer + Saturating Window Monotonicity + Operator Pin (DRIFT-DNP3-DIRECTION-001 / DRIFT-DNP3-CLOCK-001 / DRIFT-DNP3-OP-001)"
epic_id: E-15
wave: 63
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-v0.11.0
github_issue: 316
subsystems: [SS-15]
target_module: analyzer/dnp3
depends_on: [STORY-139]
blocks: []
behavioral_contracts:
  - BC-2.15.016
  - BC-2.15.010
  - BC-2.15.014
  - BC-2.15.015
verification_properties:
  - VP-035
  - VP-036
assumption_validations: []
risk_mitigations: []
ruling: RULING-DNP3-SIBLING-001
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.016.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.010.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.014.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.015.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/cycles/feature-enip-v0.11.0/RULING-DNP3-SIBLING-001-direction-and-clock.md
input-hash: "b3a4fd0"
---

# STORY-140: DNP3 Per-Direction Carry Buffer + Saturating Window Monotonicity + Operator Pin (DRIFT-DNP3-DIRECTION-001 / DRIFT-DNP3-CLOCK-001 / DRIFT-DNP3-OP-001)

## Narrative

**As a** security analyst relying on wirerust DNP3/ICS detections for OT threat detection,
**I want** the DNP3 analyzer to correctly isolate carry-buffer state per TCP direction and
apply backwards-clock-safe window expiry arithmetic with a consistent strict-greater-than
operator,
**so that** bidirectional DNP3 flows do not produce phantom findings or suppress legitimate
detections (DRIFT-DNP3-DIRECTION-001), adversarially injected out-of-order timestamps cannot
abort in-progress burst windows (DRIFT-DNP3-CLOCK-001), and the 300s correlation window
expiry is consistent with all other DNP3 window checks (DRIFT-DNP3-OP-001) — unblocking the
v0.11.0 release.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.15.016 | v2.0 | Per-Flow State Bounds — Carry Buffers ≤292 B per Direction, master_addrs ≤64, pending_requests ≤256 | Per-direction carry split (`carry_c2s`/`carry_s2c`); `on_data` direction parameter; Invariant 6 direction isolation; EC-010 direction non-contamination |
| BC-2.15.010 | v1.8 | Unauthorized Control Command — Unexpected Source (count=1) or Control-Class FC Exceeding Threshold Emits T1692.001 | Postcondition 4 window-expiry: `saturating_sub > 60`; EC-012 backwards-ts no-reset |
| BC-2.15.014 | v2.1 | Inferred Block-Command — Control Request Without Response Within Window Emits T1691.001 | Precondition 3 timeout check: `saturating_sub > 10`; EC-009 backwards-clock on pending-request timeout |
| BC-2.15.015 | v2.0 | Derived Loss-of-Control — N Restart/Block Events in Window Emits T0827 as Correlated Finding | Window-expiry: `saturating_sub > 300` (strict `>`, was `>=`); EC-010 backwards-clock on 300s window; single reset owner |

## Acceptance Criteria

### AC-140-001: `Dnp3FlowState.carry` removed; `carry_c2s` and `carry_s2c` added; `on_data` gains `direction: Direction`
**Traces to:** BC-2.15.016 v2.0 Precondition 2, Postcondition 1, Postcondition 2, Postcondition 3, Postcondition 4, Invariant 1, Invariant 6, EC-010

The single `carry: Vec<u8>` field is removed from `Dnp3FlowState`. Two separate fields are added:
- `carry_c2s: Vec<u8>` — partial frames from the TCP master (master-to-outstation, ClientToServer direction)
- `carry_s2c: Vec<u8>` — partial frames from the TCP outstation (outstation-to-master, ServerToClient direction)

`Dnp3Analyzer::on_data` gains a new `direction: Direction` parameter (type:
`crate::reassembly::handler::Direction`), mirroring the existing Modbus `StreamHandler`
signature:

```rust
// AFTER (BC-2.15.016 v2.0 Precondition 2):
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction)
```

Within the frame-walk loop:
- The directional carry buffer is selected at call entry:
  `let active_carry = if direction == Direction::ClientToServer { &mut flow.carry_c2s } else { &mut flow.carry_s2c };`
- `buf = active_carry ++ data` is formed from the SAME directional carry (BC-2.15.016 v2.0 Postcondition 1)
- After the loop, remaining partial-frame bytes are stashed back into the SAME directional carry
  (BC-2.15.016 v2.0 Postcondition 4)
- The carry-cap check (`active_carry.len() + new_bytes.len() > MAX_DNP3_FRAME_LEN = 292`) applies
  to the active directional carry only; the other direction's carry is unaffected (BC-2.15.016 v2.0 Postcondition 2)
- Invariant 6: `carry_c2s` and `carry_s2c` are NEVER mixed. A partial c2s frame stashed in
  `carry_c2s` is NEVER prepended to an s2c delivery, and vice versa. (RULING-DNP3-SIBLING-001 §1.2)
- **DNP3 carry-cap IS reachable** (RULING-DNP3-SIBLING-001 §4): unlike the ENIP situation,
  there is no "exactly-cap is unreachable" ambiguity. The overflow arm can be reached via
  repeated sub-frame deliveries accumulating carry up to 291 bytes, then one more delivery.
  Each directional carry is independently bounded; the overflow arm is kept live (BC-2.15.016 EC-004).

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_001_carry_direction_isolation_no_splice`
— Deliver partial master-to-outstation frame (5 bytes of a 10-byte DNP3 link header, stashed in
`carry_c2s`), then deliver a complete outstation-to-master frame with valid sync `[0x05, 0x64]`,
FIR=1 Response (FC=0x81). Assert: `frame_count == 1` (the s2c frame processed), `parse_errors == 0`,
`carry_c2s.len() > 0` (partial c2s retains its bytes), `carry_s2c.len() == 0` (s2c carry drained).
This is the EC-X1 analog for DNP3 (RULING-DNP3-SIBLING-001 §8 AC-8). (traces to BC-2.15.016 v2.0 Invariant 6, EC-010)

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_001_carry_c2s_and_carry_s2c_are_independent`
— Deliver a partial c2s frame (carry_c2s stashed), then deliver a partial s2c frame (carry_s2c
stashed). Assert `carry_c2s.len() > 0 && carry_s2c.len() > 0` after both deliveries, and
that neither carry contains bytes from the other direction. (traces to BC-2.15.016 v2.0 Postcondition 1, Invariant 6)

### AC-140-002: Direction-aware source IP resolution replaces port-20000 heuristic
**Traces to:** RULING-DNP3-SIBLING-001 §1.4 (fix-along); BC-2.15.016 v2.0 Precondition 2

The `resolve_master_ip` port-20000 heuristic is replaced with direction-aware source IP
resolution (Modbus pattern at `src/analyzer/modbus.rs` ~355-382). With `Direction` now
available in `on_data`, findings emit:

```rust
let master_ip = match direction {
    Direction::ClientToServer => flow_key.src_ip_of(direction),  // master initiates to port 20000
    Direction::ServerToClient => flow_key.dst_ip_of(direction),  // outstation listens on port 20000
};
```

This resolves DRIFT-DNP3-DIRECTION-001 as a no-cost fix-along since `Direction` is now threaded.
The `direction` field on emitted Findings is NOT populated by this story — DNP3 detections are
per-flow aggregates; finding-direction tagging is deferred to the v0.12.0 per-direction
sub-state rearchitecture. The old `resolve_master_ip` function must be removed (not retained as dead code).

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_002_direction_based_source_ip`
— Feed a c2s Control-class FC (DIRECT_OPERATE, FC=0x05, FIR=1) on a flow where `src=10.0.0.1:54321`,
`dst=10.0.0.2:20000`, `direction=ClientToServer`. Push enough Control FCs to cross the threshold.
Assert the emitted T1692.001 finding has `source_ip == Some(IpAddr::from([10, 0, 0, 1]))` (the
initiator IP, not the port-20000 heuristic fallback). (traces to RULING-DNP3-SIBLING-001 §1.4)

### AC-140-003: All dispatcher call sites for `Dnp3Analyzer::on_data` updated to pass `direction`
**Traces to:** BC-2.15.016 v2.0 Precondition 2; RULING-DNP3-SIBLING-001 §1.5

All `Dnp3Analyzer::on_data` call sites in `src/dispatcher.rs` (DNP3 arm of the stream
dispatcher) are updated to pass `direction` from the `StreamHandler` context — matching exactly
the Modbus arm pattern already established by STORY-139 for ENIP. No changes are made to the
ENIP dispatcher arm (`src/analyzer/enip.rs` and its dispatcher call sites are out of scope —
ENIP was fixed in STORY-139).

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_003_dispatcher_passes_direction`
— Integration-style test: construct a `Dnp3Analyzer` and call `on_data` directly with both
`Direction::ClientToServer` and `Direction::ServerToClient` values; assert no compilation error
and that the 4-argument signature is accepted. (traces to BC-2.15.016 v2.0 Precondition 2)

### AC-140-004: All `wrapping_sub` sites replaced with `saturating_sub` (8 computation sites)
**Traces to:** BC-2.15.010 v1.8 Postcondition 4, BC-2.15.014 v2.1 Precondition 3, BC-2.15.015 v2.0 Postcondition 3, Invariant 6; RULING-DNP3-SIBLING-001 §2.2

Replace every `now_ts.wrapping_sub(...)` in windowed timestamp comparisons within `dnp3.rs`
with `now_ts.saturating_sub(...)`. All 8 code sites in the ruling:

| dnp3.rs approx. line | Window | Before | After |
|---------------------|--------|--------|-------|
| ~745 | 60s detect | `now_ts.wrapping_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS` | `now_ts.saturating_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS` |
| ~765 | 60s detect | `now_ts.wrapping_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS` | `now_ts.saturating_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS` |
| ~769 | 60s display | `now_ts.wrapping_sub(flow.window_start_ts)` (elapsed) | `now_ts.saturating_sub(flow.window_start_ts)` |
| ~895 | 10s block timeout | `now_ts.wrapping_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS` | `now_ts.saturating_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS` |
| ~984 | 300s correlation | `now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS` | `now_ts.saturating_sub(flow.correlation_window_start_ts) > CORRELATION_WINDOW_SECS` |
| ~1025 | 300s display | `now_ts.wrapping_sub(flow.correlation_window_start_ts)` (elapsed) | `now_ts.saturating_sub(flow.correlation_window_start_ts)` |
| ~1335 | 300s T0814 guard | `now_ts.wrapping_sub(flow.correlation_window_start_ts) < CORRELATION_WINDOW_SECS` | `now_ts.saturating_sub(flow.correlation_window_start_ts) < CORRELATION_WINDOW_SECS` |
| ~1341 | 300s T0814 display | `now_ts.wrapping_sub(flow.correlation_window_start_ts)` (elapsed) | `now_ts.saturating_sub(flow.correlation_window_start_ts)` |

Semantics: when `now_ts < window_start_ts` (backwards/out-of-order timestamp),
`saturating_sub` returns 0, which is NOT > any positive threshold. The window is
preserved. An adversary injecting a single old-timestamp packet cannot abort a
burst-in-progress. (RULING-DNP3-SIBLING-001 §2.2)

**No test dedicated to this AC** — validated through AC-140-005, AC-140-006, AC-140-007,
VP-036 proptest harnesses, and the `cargo clippy -- -D warnings` check that `wrapping_sub`
does not appear in window comparison paths. (traces to BC-2.15.010 v1.8 PC4, BC-2.15.014 v2.1 PC3, BC-2.15.015 v2.0 PC3)

### AC-140-005: 300s correlation-window expiry uses strict `>` (not `>=`)
**Traces to:** BC-2.15.015 v2.0 Postcondition 3, Invariant 6; RULING-DNP3-SIBLING-001 §2.3

The 300s correlation-window expiry condition at the relevant line in `dnp3.rs` is changed
from `>= CORRELATION_WINDOW_SECS` to `> CORRELATION_WINDOW_SECS`. This is consistent with:
- `dnp3.rs` T1692.001 window: `> DETECTION_WINDOW_SECS`
- `dnp3.rs` T1691.001 timeout: `> BLOCK_CMD_TIMEOUT_SECS`
- All three ENIP windows after STORY-139

Semantic: `> CORRELATION_WINDOW_SECS` means a packet at exactly elapsed=300s is the last
packet of the current window, not the first of the new one.

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_005_correlation_window_operator_pin_boundary`
— Deliver 2 COLD_RESTART events at ts=0 and ts=150 (`restart_event_count=2`,
`correlation_window_start_ts=0`). Advance to exactly elapsed=300 seconds (ts=300); deliver
one more COLD_RESTART. Assert: window is NOT reset at elapsed=300 (strict `>` means 300 > 300
is false); `restart_event_count == 3`; T0827 fires. Advance to elapsed=301 (ts=301); assert
the window IS reset on the next call. (traces to BC-2.15.015 v2.0 Postcondition 3, Invariant 6)

### AC-140-006: All existing DNP3 tests pass with the new `on_data` signature
**Traces to:** BC-2.15.016 v2.0 Precondition 2; RULING-DNP3-SIBLING-001 §8 AC-7

All existing tests in all DNP3 test files (`tests/dnp3_detection_tests.rs`,
`tests/dnp3_flow_state_tests.rs`, `tests/dnp3_parse_core_tests.rs`,
`tests/dnp3_correlation_tests.rs`, `tests/dnp3_determinism_tests.rs`,
`tests/dnp3_f5_remediation_tests.rs`, `tests/bc_2_15_110_dnp3_dispatcher_tests.rs`)
pass with the new `on_data(flow_key, data, ts, direction)` four-argument signature.
Existing call sites are updated to pass `Direction::ClientToServer` for c2s traffic and
`Direction::ServerToClient` for s2c traffic; call sites that do not test direction
semantics may use either value consistently.

**Test:** `cargo test --all-targets` — zero regressions. The gate is the full test suite
passing green. (traces to BC-2.15.016 v2.0 Precondition 2)

### AC-140-007: Regression — partial c2s frame in `carry_c2s` + complete s2c frame → direction isolation confirmed
**Traces to:** BC-2.15.016 v2.0 Invariant 6, EC-010; RULING-DNP3-SIBLING-001 §8 AC-8

Deliver a partial master-to-outstation frame (bytes that do NOT form a complete DNP3 link
frame — fewer than 10 bytes, not a complete header), then deliver a complete
outstation-to-master frame with valid FIR=1 Response (FC=0x81). Assert:
- `frame_count == 1` (only the s2c frame was fully processed)
- `parse_errors == 0` (no cross-direction splice producing a malformed frame)
- `carry_c2s.len() > 0` (the partial c2s bytes are still stashed in `carry_c2s`)
- `carry_s2c.len() == 0` (the s2c carry was consumed by the complete s2c frame)

This is the DNP3 EC-X1 analog from RULING-DNP3-SIBLING-001 §1.1. Before the fix, the
spliced buffer `carry(master_partial_header) ++ outstation_bytes` could pass the sync gate
(`[0x05, 0x64]` already in carry head) and produce spurious `parse_errors`.

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_007_regression_carry_direction_no_splice`
(may be combined with or aliased to `test_ac140_001_carry_direction_isolation_no_splice` if
the assertion set is identical; must be a uniquely named test). (traces to BC-2.15.016 v2.0 Invariant 6, EC-010)

### AC-140-008: Regression — backwards-clock packet does not reset 60s detect window; T1692.001 fires at count=11
**Traces to:** BC-2.15.010 v1.8 Postcondition 4, EC-012; RULING-DNP3-SIBLING-001 §8 AC-9

Deliver 9 Control-class FCs (DIRECT_OPERATE, FC=0x05, FIR=1) at ts=100
(`window_start_ts=100`, `direct_operate_count=9`). Then deliver 1 Control-class FC at
ts=50 (backwards clock). Assert: `saturating_sub(50, 100) = 0`; elapsed=0, NOT > 60 →
window NOT reset; `direct_operate_count` increments to 10. Threshold is strict `>` not
`>=`: count=10 is NOT > 10 (`direct_operate_threshold` default=10), so no finding yet.
Deliver 2 more Control-class FCs at ts=100. Assert: `direct_operate_count` reaches 11,
T1692.001 fires (11 > 10). No spurious window reset from the backwards-ts packet.

This is the DNP3 EC-X2 analog (RULING-DNP3-SIBLING-001 §8 AC-9).

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_008_regression_backwards_ts_t1692_no_reset`
(traces to BC-2.15.010 v1.8 Postcondition 4, EC-012)

### AC-140-009: Regression — backwards-clock event does not reset 300s correlation window; T0827 fires
**Traces to:** BC-2.15.015 v2.0 Postcondition 3, EC-010; RULING-DNP3-SIBLING-001 §8 AC-10

Deliver 2 COLD_RESTART events (FC=0x0D, triggers `restart_event_count` increment) at
ts=0 and ts=150 (`restart_event_count=2`, `correlation_window_start_ts=0`). Then deliver
one event at ts=50 (backwards clock). Assert: `saturating_sub(50, 0) = 50`... actually
`correlation_window_start_ts=0` and `backwards_ts=50` — but backwards relative to another
call. Use the exact scenario from BC-2.15.015 EC-010: 2 restarts at ts=100
(`correlation_window_start_ts=100`), then one event at ts=50 (backwards).
Assert: `saturating_sub(50, 100) = 0`; NOT > 300 → window NOT reset; `restart_event_count`
still 2 on the backwards-ts call. Deliver one more COLD_RESTART at ts=200: `restart_event_count=3`
→ T0827 fires (3 ≥ T0827_THRESHOLD). No spurious window reset from the backwards-ts packet.

**Test:** `tests/dnp3_detection_tests.rs::direction_and_clock::test_ac140_009_regression_backwards_ts_t0827_no_reset`
(traces to BC-2.15.015 v2.0 Postcondition 3, Invariant 6, EC-010)

### AC-140-010: VP-035 proptest (carry direction isolation) — GENUINE proptest with generated strategies
**Traces to:** VP-035; BC-2.15.016 v2.0 Invariant 6; RULING-DNP3-SIBLING-001 §7

The VP-035 proptest harness skeleton from `vp-035-dnp3-carry-direction-isolation.md` is
implemented as a passing proptest suite. These tests serve as regression guards preventing
future refactors from breaking direction isolation.

**CRITICAL: these MUST be genuine proptests using `proptest::prelude::*` with generated
strategies** — NOT deterministic point tests masquerading as proptests. The `proptest!` macro
must drive the strategy generation; test failure must be reproducible via the seed. (This
lesson is explicit from STORY-139 F-139-002.)

**VP-035 tests** (mod `vp035_dnp3_carry_direction_isolation` in `tests/dnp3_detection_tests.rs`):

- `proptest_vp035_direction_isolation_frame_count` — generates `split_offset in 1usize..9`
  and `_s2c_ctrl in 0x00u8..=0xFFu8`; builds minimal DNP3 frames (`build_minimal_dnp3_frame`
  helper); verifies that delivering partial-c2s + full-s2c + completing-c2s produces
  `frame_count == 2` and `parse_errors == 0`; verifies both carries are drained after
  completion. Strategy drives the partial-delivery split point over the range of possible
  incomplete header prefixes.
- `proptest_vp035_independent_run_equivalence` — generates `split_offset in 1usize..9`;
  verifies that the interleaved run `frame_count` and `parse_errors` equal the sum of
  independent same-direction control runs. Establishes carry-isolation as an observable
  behavioral invariant independent of FIR gating.

**Test:** `tests/dnp3_detection_tests.rs::vp035_dnp3_carry_direction_isolation::proptest_vp035_direction_isolation_frame_count`
**Test:** `tests/dnp3_detection_tests.rs::vp035_dnp3_carry_direction_isolation::proptest_vp035_independent_run_equivalence`
(traces to VP-035; BC-2.15.016 v2.0 Invariant 6)

### AC-140-011: VP-036 proptest (window monotonic) — GENUINE proptests for all three windows + operator pin + rollover
**Traces to:** VP-036; BC-2.15.010 v1.8 PC4, BC-2.15.014 v2.1 PC3, BC-2.15.015 v2.0 PC3, Invariant 6; RULING-DNP3-SIBLING-001 §7

The VP-036 proptest harness skeleton from `vp-036-dnp3-window-monotonic-no-spurious-reset.md`
is implemented as passing proptest suites covering all four sub-properties.

**CRITICAL: genuine proptests only.** The `prop_assume!(backwards_ts <= window_start)`
filter must be used to constrain the strategy domain — do NOT replace with a deterministic
fixed scenario and call it a proptest.

**VP-036 tests** (mod `vp036_dnp3_window_monotonic_no_spurious_reset` in `tests/dnp3_detection_tests.rs`):

- `proptest_vp036_sub_a_direct_operate_60s_backwards_ts_no_reset` — Sub-A: T1692.001 60s
  window; strategy generates `(window_start in 1u32..u32::MAX, threshold in 1u64..200,
  backwards_ts in 0u32..=u32::MAX)` with `prop_assume!(backwards_ts <= window_start)`;
  asserts `saturating_sub(backwards_ts, window_start) == 0` → NOT > 60 → no reset;
  `threshold + 1 > threshold` → T1692.001 fires on next forward event.
- `proptest_vp036_sub_a_ec_x2_repro_t1692` — Direct EC-X2 T1692.001 repro: generates
  `burst_count in 2u64..200`; asserts `50u32.saturating_sub(100) == 0` (not the wrapping
  ~4.29e9 that caused the bug). Deterministic in inputs but driven by proptest runner.
- `proptest_vp036_sub_b_block_timeout_backwards_ts_no_fire` — Sub-B: T1691.001 10s
  block-command timeout; generates `(request_ts in 1u32..u32::MAX, backwards_ts in
  0u32..=u32::MAX)` with `prop_assume!(backwards_ts <= request_ts)`; asserts
  `saturating_sub(backwards_ts, request_ts) == 0` → NOT > 10 → timeout NOT spuriously
  fired.
- `proptest_vp036_sub_c_300s_window_backwards_ts_no_reset` — Sub-C: T0827/T0814 300s
  window no-reset; `prop_assume!(backwards_ts <= window_start)`; asserts `saturating_sub
  == 0` → NOT > 300 → window NOT reset; includes `prop_assume!(forward_ts.saturating_sub(window_start) <= 300)` for the still-in-window forward event.
- `proptest_vp036_sub_c_operator_pin_elapsed_300_not_expired` — Operator pin boundary:
  `elapsed == 300`: `300u32.saturating_sub(0) > 300` is FALSE → window NOT expired (packet
  is last of window). `elapsed == 301`: `301u32.saturating_sub(0) > 300` is TRUE → window
  expires. (DRIFT-DNP3-OP-001 operator pin)
- `test_vp036_sub_d_genuine_rollover_no_spurious_reset` — Deterministic unit test:
  `window_start = u32::MAX - 5`, `now_ts = 4`; `wrapping_sub` would give 10 (old spurious
  reset on the 10s block-timeout window); `saturating_sub` gives 0 (no spurious reset).
  Documents old vs. new behavior for all three windows.

**Test:** `tests/dnp3_detection_tests.rs::vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_a_direct_operate_60s_backwards_ts_no_reset`
**Test:** `tests/dnp3_detection_tests.rs::vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_a_ec_x2_repro_t1692`
**Test:** `tests/dnp3_detection_tests.rs::vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_b_block_timeout_backwards_ts_no_fire`
**Test:** `tests/dnp3_detection_tests.rs::vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_c_300s_window_backwards_ts_no_reset`
**Test:** `tests/dnp3_detection_tests.rs::vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_c_operator_pin_elapsed_300_not_expired`
**Test:** `tests/dnp3_detection_tests.rs::vp036_dnp3_window_monotonic_no_spurious_reset::test_vp036_sub_d_genuine_rollover_no_spurious_reset`
(traces to VP-036; BC-2.15.010 v1.8 PC4, BC-2.15.014 v2.1 PC3, BC-2.15.015 v2.0 PC3)

### AC-140-012: `cargo clippy`, `cargo fmt`, `cargo test --all-targets` all green
**Traces to:** BC-2.15.016 v2.0 (no `wrapping_sub` in window comparisons; `carry` singular absent); BC-2.15.015 v2.0 (operator `>=` removed)

After all changes:
- `cargo clippy --all-targets -- -D warnings` — zero warnings. In particular, no `wrapping_sub`
  call appears in windowed comparison paths in `dnp3.rs` (would indicate a missed replacement).
- `cargo fmt --check` — zero format drift.
- `cargo test --all-targets` — full test suite green (no regressions in any existing test module).
- `grep -n 'wrapping_sub' src/analyzer/dnp3.rs` — returns no results (all 8 sites replaced).
- `grep -n '\.carry[^_]' src/analyzer/dnp3.rs` — returns no results (singular `carry` field absent; only `carry_c2s` and `carry_s2c` remain).

**Test:** implicit — CI gate. (traces to BC-2.15.016 v2.0 Invariant 6, BC-2.15.015 v2.0 Invariant 6)

## Architecture Mapping

| Component | Location | Role | Pure/Effectful |
|-----------|----------|------|----------------|
| `Dnp3FlowState.carry_c2s: Vec<u8>` | `src/analyzer/dnp3.rs` | Per-direction carry buffer (c2s, master-to-outstation); replaces `carry: Vec<u8>` | Effectful (mutated by on_data) |
| `Dnp3FlowState.carry_s2c: Vec<u8>` | `src/analyzer/dnp3.rs` | Per-direction carry buffer (s2c, outstation-to-master); replaces `carry: Vec<u8>` | Effectful (mutated by on_data) |
| `Dnp3Analyzer::on_data(flow_key, data, ts, direction)` | `src/analyzer/dnp3.rs` | Frame-walk entry point; `direction: Direction` added; active carry selected by direction | Effectful shell |
| `crate::reassembly::handler::Direction` | `src/reassembly/handler.rs` | Direction enum (already used by Modbus and, after STORY-139, by ENIP); imported by DNP3 | Pure enum |
| Stream dispatcher DNP3 arm | `src/dispatcher.rs` | Call site update: pass `Direction` to `Dnp3Analyzer::on_data` (Modbus/ENIP pattern) | Effectful |
| `resolve_master_ip` (removed) | `src/analyzer/dnp3.rs` | Port-20000 heuristic replaced by direction-based IP selection inline | — (deleted) |
| `tests/dnp3_detection_tests.rs` | `tests/dnp3_detection_tests.rs` | `mod direction_and_clock { ... }` + `mod vp035_dnp3_carry_direction_isolation { ... }` + `mod vp036_dnp3_window_monotonic_no_spurious_reset { ... }` | Test |

**Subsystem anchor:** SS-15 owns this story's scope because the carry-buffer split, direction
threading, saturating_sub window expiry fixes, and operator pin are all localized to
`src/analyzer/dnp3.rs` and its dispatch call site in SS-05 (src/dispatcher.rs). Per
ARCH-INDEX.md §SS-15.

**Dependency anchor:** STORY-140 depends on STORY-139 because STORY-139 established the
`Direction` threading pattern in the dispatcher for ENIP, confirming the `StreamHandler`
context's `Direction` field is available and the Modbus-mirroring pattern works. The
dispatcher infrastructure changes in STORY-139 are the prerequisite proof that the same
carry-split + Direction-threading dispatcher pattern is applicable to the DNP3 arm. The
ADR-007 Decision 2 struct layout change (`carry_c2s`/`carry_s2c`) in STORY-140 requires
STORY-139 to be merged so that `enip.rs` is already consistent.

**Forbidden dependencies:** `src/analyzer/dnp3.rs` MUST NOT depend on any other analyzer
module (`enip`, `modbus`, `arp`, `http`, `tls`, etc.). If this module gains a cross-analyzer
dependency (other than shared types from `src/reassembly/`), the build MUST fail. The
`Direction` type is shared infrastructure from `src/reassembly/handler.rs`, NOT an analyzer
dependency.

**NOT in scope:**
- `src/analyzer/enip.rs` — ENIP was fixed in STORY-139 (RULING-EDGECASE-001). Do NOT touch
  `enip.rs`, its tests, or its dispatcher arm in this story.
- Per-direction sub-state for detection counters (deferred to v0.12.0 per RULING-DNP3-SIBLING-001 §1.2).
- ADR-007 prose update — the ruling specifies this be done at F4 entry on a worktree; the
  implementer adds a file-structure entry for `docs/adr/0007` as an F4 action, not an F3 code action.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Partial c2s frame stashed in `carry_c2s`; next call is s2c with full response | `carry_s2c` (empty) prepended to s2c data; s2c frame processes cleanly; `carry_c2s` retains partial bytes unchanged — no splice (BC-2.15.016 v2.0 EC-010) |
| EC-002 | Both directions have partial frames stashed simultaneously | `carry_c2s.len() > 0` AND `carry_s2c.len() > 0` concurrently; each direction's next delivery completes its own frame independently |
| EC-003 | Carry-cap overflow in c2s direction | `parse_errors++`; inline resync on `carry_c2s`; `carry_s2c` unaffected (BC-2.15.016 v2.0 Postcondition 2, EC-004) |
| EC-004 | DNP3 carry-cap IS reachable (unlike ENIP) | Overflow arm fires when `carry_c2s.len() + new_bytes.len() > 292`; no "Path B unreachability" — this path can be reached via adversarial partial-frame flood. Carry-cap code path must be kept live. (RULING-DNP3-SIBLING-001 §4) |
| EC-005 | 9 Control FCs at ts=100, 1 FC at ts=50 (backwards), 2 FCs at ts=100 | `saturating_sub(50, 100) = 0`; window NOT reset; count reaches 11; T1692.001 fires (BC-2.15.010 v1.8 EC-012) |
| EC-006 | COLD_RESTART events at ts=100, backwards event at ts=50, COLD_RESTART at ts=200 | Window NOT reset at ts=50; `restart_event_count` preserved; T0827 fires when count reaches threshold (BC-2.15.015 v2.0 EC-010) |
| EC-007 | Correlation window elapsed == 300 exactly (strict operator pin) | Window NOT expired under `> 300` (EC-X analog for DNP3 DRIFT-DNP3-OP-001) |
| EC-008 | Correlation window elapsed == 301 | Window IS expired; all six fields reset (restart_event_count, block_event_count, block_finding_emitted_this_window, loss_of_control_emitted, malformed_in_window, malformed_anomaly_emitted); new window seeded (BC-2.15.015 v2.0 Postcondition 3) |
| EC-009 | Control request at ts=100 in pending_requests; backwards on_data at ts=50 | `saturating_sub(50, 100) = 0`; NOT > 10 → timeout NOT fired; request remains pending; no spurious `block_event_count` increment (BC-2.15.014 v2.1 EC-009) |
| EC-010 | Genuine u32 rollover: `window_start = u32::MAX - 5` (0xFFFFFFFA), `now_ts = 500` | `saturating_sub(500, u32::MAX-5) = 0`; no spurious reset on any of the three windows (vs. `wrapping_sub` which gives 506 — spuriously fires all three: 506 > 300 T0827/T0814, 506 > 60 T1692.001, 506 > 10 T1691.001) |
| EC-011 | Existing DNP3 tests with the old 3-argument `on_data` signature | All existing call sites updated to 4-argument form; no compilation errors; no behavioral regressions |

## Tasks

- [ ] In `Dnp3FlowState`: remove `carry: Vec<u8>`; add `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`
- [ ] Add `direction: Direction` parameter to `Dnp3Analyzer::on_data`; update signature throughout
- [ ] Import `crate::reassembly::handler::Direction` in `dnp3.rs` (already imported in `modbus.rs` and `enip.rs` after STORY-139)
- [ ] In the frame-walk loop:
  - [ ] Select directional carry at entry: `let active_carry = if direction == Direction::ClientToServer { &mut flow.carry_c2s } else { &mut flow.carry_s2c };`
  - [ ] Build `buf = active_carry.drain(..).chain(data.iter().copied()).collect()` (or equivalent concatenation)
  - [ ] After loop, stash remaining bytes into the SAME directional carry
  - [ ] Cap check: `active_carry.len() + data.len() > MAX_DNP3_FRAME_LEN` → `parse_errors += 1`, `malformed_in_window += 1`, inline resync, keep carry cap live (RULING-DNP3-SIBLING-001 §4)
- [ ] Replace `resolve_master_ip` port-20000 heuristic with direction-based IP selection:
  `let master_ip = match direction { ClientToServer => flow_key.src_ip_of(direction), ServerToClient => flow_key.dst_ip_of(direction) };`
  Update all finding emission paths that currently call `resolve_master_ip`. Remove `resolve_master_ip` as dead code.
- [ ] Replace ALL `wrapping_sub` with `saturating_sub` in window expiry comparisons (8 sites):
  - [ ] T1692.001 60s window reset: `now_ts.saturating_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS`
  - [ ] T1692.001 60s window emit guard: `now_ts.saturating_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS`
  - [ ] T1692.001 60s elapsed display: `now_ts.saturating_sub(flow.window_start_ts)`
  - [ ] T1691.001 10s block timeout: `now_ts.saturating_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS`
  - [ ] 300s correlation window expiry: `now_ts.saturating_sub(flow.correlation_window_start_ts) > CORRELATION_WINDOW_SECS` (**also change `>=` → `>`**)
  - [ ] 300s correlation window elapsed display: `now_ts.saturating_sub(flow.correlation_window_start_ts)`
  - [ ] T0814 300s in-window guard: `now_ts.saturating_sub(flow.correlation_window_start_ts) < CORRELATION_WINDOW_SECS`
  - [ ] T0814 300s elapsed display: `now_ts.saturating_sub(flow.correlation_window_start_ts)`
- [ ] Update `src/dispatcher.rs` DNP3 arm call site to pass `Direction` (mirror Modbus and ENIP arms — do NOT touch the ENIP arm)
- [ ] Update ALL existing tests in all DNP3 test files that call `on_data` to pass a `Direction` argument
  - [ ] `tests/dnp3_detection_tests.rs` — all `on_data` call sites
  - [ ] `tests/dnp3_flow_state_tests.rs` — all `on_data` call sites
  - [ ] `tests/dnp3_parse_core_tests.rs` — all `on_data` call sites
  - [ ] `tests/dnp3_correlation_tests.rs` — all `on_data` call sites
  - [ ] `tests/dnp3_determinism_tests.rs` — all `on_data` call sites
  - [ ] `tests/dnp3_f5_remediation_tests.rs` — all `on_data` call sites
  - [ ] `tests/bc_2_15_110_dnp3_dispatcher_tests.rs` — all `on_data` call sites
- [ ] Add `mod direction_and_clock` to `tests/dnp3_detection_tests.rs` with AC-140-001..009 named tests
- [ ] Add `mod vp035_dnp3_carry_direction_isolation` with genuine VP-035 proptest harnesses (2 tests)
- [ ] Add `mod vp036_dnp3_window_monotonic_no_spurious_reset` with genuine VP-036 proptest + deterministic harnesses (6 tests)
- [ ] Run `cargo test dnp3` — all direction_and_clock + VP proptest tests pass
- [ ] Run `cargo test --all-targets` — full test suite green (no regressions)
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings
- [ ] Run `cargo fmt --check` — zero format drift
- [ ] Run `grep -n 'wrapping_sub' src/analyzer/dnp3.rs` — no results
- [ ] Run `grep -n '\.carry[^_]' src/analyzer/dnp3.rs` — no results (singular `carry` gone)
- [ ] Add `docs/adr/0007` to file structure as an F4-action note (do NOT modify `docs/adr/0007` during F3 — update is scheduled at F4 per RULING-DNP3-SIBLING-001 §6)
- [ ] Run `bin/compute-input-hash --write .factory/stories/STORY-140.md` to populate `input-hash`

## Test Plan

**Test file:** `tests/dnp3_detection_tests.rs`

**New test modules added by this story:**

```
mod direction_and_clock {
    test_ac140_001_carry_direction_isolation_no_splice
    test_ac140_001_carry_c2s_and_carry_s2c_are_independent
    test_ac140_002_direction_based_source_ip
    test_ac140_003_dispatcher_passes_direction
    test_ac140_005_correlation_window_operator_pin_boundary
    test_ac140_007_regression_carry_direction_no_splice
    test_ac140_008_regression_backwards_ts_t1692_no_reset
    test_ac140_009_regression_backwards_ts_t0827_no_reset
}

mod vp035_dnp3_carry_direction_isolation {
    proptest_vp035_direction_isolation_frame_count
    proptest_vp035_independent_run_equivalence
}

mod vp036_dnp3_window_monotonic_no_spurious_reset {
    proptest_vp036_sub_a_direct_operate_60s_backwards_ts_no_reset
    proptest_vp036_sub_a_ec_x2_repro_t1692
    proptest_vp036_sub_b_block_timeout_backwards_ts_no_fire
    proptest_vp036_sub_c_300s_window_backwards_ts_no_reset
    proptest_vp036_sub_c_operator_pin_elapsed_300_not_expired
    test_vp036_sub_d_genuine_rollover_no_spurious_reset
}
```

**GENUINE proptest discipline (lessons from STORY-139 F-139-002):** All `proptest!` blocks
MUST use generated strategies via `proptest::prelude::*`. The `prop_assume!(backwards_ts <=
window_start)` filter is the correct technique for constraining the strategy domain, NOT
replacing the proptest with a deterministic point test. A proptest that calls
`assert_eq!(50u32.saturating_sub(100), 0)` as its only assertion is NOT a genuine proptest.

**Updated existing tests:** All existing `on_data(key, data, ts)` call sites in all DNP3 test
files updated to `on_data(key, data, ts, Direction::ClientToServer)` or
`Direction::ServerToClient` as appropriate. Call sites that do not test direction semantics
may use `Direction::ClientToServer` as a neutral default.

**TDD discipline (strict mode):** Implementer writes `todo!()` stubs for all new field
additions and `on_data` parameter changes first. Existing tests fail due to signature change
(RED gate). Then implements the direction split and saturating_sub fixes to turn them GREEN
one-by-one. No implementation code is written before a failing test exists.

## Previous Story Intelligence

- STORY-107 introduced `Dnp3FlowState.carry: Vec<u8>` and the frame-walk loop. This story
  replaces `carry` with `carry_c2s`/`carry_s2c`.
- STORY-108 introduced `direct_operate_count`, `window_start_ts`, `direct_operate_emitted`,
  and the `wrapping_sub` detect-window expiry that is fixed here (T1692.001).
- STORY-109 introduced `pending_requests`, `block_event_count`, `correlation_window_start_ts`,
  and the `wrapping_sub` correlation-window expiry and block-timeout that are fixed here
  (T1691.001, T0827/T0814). Also introduced `malformed_in_window`, `malformed_anomaly_emitted`
  (BC-2.15.024 windowed fields), and `T1691.001` arm in `src/mitre.rs:174`.
- STORY-110 delivered the dispatcher integration + CLI flag for DNP3 (BC-2.15.017).
  The `Dnp3Analyzer::on_data` call site in `src/dispatcher.rs` was established in STORY-110;
  this story adds the `Direction` argument to it.
- STORY-139 (wave 62) fixed the identical bug pattern in `src/analyzer/enip.rs`:
  - Established the `carry_c2s`/`carry_s2c` split pattern
  - Confirmed the `Direction` threading dispatcher pattern works (Modbus mirror)
  - Replaced `wrapping_sub` with `saturating_sub` in all ENIP window expiry paths
  - The implementation pattern for STORY-140 MIRRORS STORY-139 exactly — read STORY-139
    carefully before implementing. The DNP3 differences are: DNP3 sync word `[0x05, 0x64]`,
    DNP3 frame min 10 bytes (vs. ENIP 24 bytes), DNP3 carry-cap IS reachable (vs. ENIP
    Path-B unreachability question), and DNP3 has three windows vs. ENIP's three windows
    with different thresholds (60s/10s/300s vs. 1s/10s/300s).
- The `Direction` type from `crate::reassembly::handler::Direction` is already imported in
  `src/analyzer/modbus.rs` and, after STORY-139, in `src/analyzer/enip.rs`. Import the same
  type in `dnp3.rs`.
- **`resolve_master_ip` port-20000 heuristic** (doc comment at line 1456 was already marked
  as a direction-deferral): now replaced. Use the same `match direction` pattern as ENIP's
  `resolve_enip_client_ip` replacement in STORY-139 AC-139-002.

## Architecture Compliance Rules

From ADR-007 Decision 2 (amended by RULING-DNP3-SIBLING-001), BC-2.15.016 v2.0,
BC-2.15.010 v1.8, BC-2.15.014 v2.1, BC-2.15.015 v2.0:

1. **`carry: Vec<u8>` is REMOVED (BC-2.15.016 v2.0 §1.3):** Any code referencing
   `flow.carry` (singular) after this story is a regression. The fields are `carry_c2s`
   and `carry_s2c`. The `grep -n '\.carry[^_]'` check in Tasks enforces this.
2. **Direction isolation is structurally enforced (BC-2.15.016 v2.0 Invariant 6):** The
   `if direction == Direction::ClientToServer { ... } else { ... }` arm selects exactly one
   of the two carry buffers. No conditional path may read `carry_c2s` when processing
   `ServerToClient` or read `carry_s2c` when processing `ClientToServer`.
3. **`saturating_sub` is the ONLY permitted window arithmetic (RULING-DNP3-SIBLING-001 §2.2):**
   `wrapping_sub` MUST NOT appear in any window-expiry comparison in `dnp3.rs`. Use
   `now_ts.saturating_sub(window_start_ts)` in all 8 window paths.
4. **Strict `>` for ALL windows (RULING-DNP3-SIBLING-001 §2.3):** The 300s correlation
   window is pinned to strict `> CORRELATION_WINDOW_SECS` (was `>= CORRELATION_WINDOW_SECS`).
   All three windows now use strict `>`. No window uses `>=`.
5. **Detection counters remain PER-FLOW (RULING-DNP3-SIBLING-001 §1.3):** `direct_operate_count`,
   `window_start_ts`, `direct_operate_emitted`, `pending_requests`, `block_event_count`,
   `restart_event_count`, `frame_count`, `parse_errors`, `master_addrs_seen`, and all other
   non-carry fields remain shared per flow (not split by direction). ONLY `carry_c2s`/`carry_s2c`
   are per-direction.
6. **DNP3 carry-cap IS reachable — keep the overflow arm live (RULING-DNP3-SIBLING-001 §4):**
   Unlike the ENIP RULING-137-002 unreachability assessment, the DNP3 carry-cap overflow arm
   at `carry.len() + new_bytes.len() > MAX_DNP3_FRAME_LEN` is reachable via adversarial
   partial-frame floods. Do NOT remove or dead-code the overflow arm. The cap must be applied
   independently to `carry_c2s` and `carry_s2c`.
7. **ENIP code is OUT OF SCOPE:** Do NOT touch `src/analyzer/enip.rs`, `tests/enip_*`, or
   the ENIP dispatcher arm. ENIP was fixed in STORY-139.
8. **`resolve_master_ip` is REMOVED (RULING-DNP3-SIBLING-001 §1.4):** The port-20000
   heuristic function is replaced by direct direction-based IP selection inline in each
   finding emission site. Do NOT retain `resolve_master_ip` as dead code.
9. **ADR-007 update is an F4 action, NOT F3:** The prose amendment to `docs/adr/0007` per
   RULING-DNP3-SIBLING-001 §6 is scheduled at F4 entry (factory-artifacts worktree). The
   implementer MUST NOT modify `docs/adr/0007` during F3 implementation.

## Library & Framework Requirements

- `crate::reassembly::handler::Direction` — existing type; no new external dependency
- `proptest` crate — already used in the test suite (VP-035/VP-036 harnesses use `proptest::prelude::*`)
- No new `Cargo.toml` dependencies

## File Structure Requirements

**Files to modify:**

- `src/analyzer/dnp3.rs`
  - Remove `carry: Vec<u8>` from `Dnp3FlowState`
  - Add `carry_c2s: Vec<u8>`, `carry_s2c: Vec<u8>` to `Dnp3FlowState`
  - Update `on_data` signature: add `direction: Direction` parameter
  - Import `crate::reassembly::handler::Direction`
  - Replace `resolve_master_ip` with inline direction-based IP selection (Modbus pattern)
  - Replace all `wrapping_sub` with `saturating_sub` in window-expiry paths (8 sites)
  - Pin 300s correlation window operator: `>= CORRELATION_WINDOW_SECS` → `> CORRELATION_WINDOW_SECS`
- `src/dispatcher.rs`
  - Update DNP3 arm call site: `dnp3_analyzer.on_data(flow_key, data, ts)` →
    `dnp3_analyzer.on_data(flow_key, data, ts, direction)` (Modbus/ENIP pattern)
  - Do NOT modify the ENIP arm
- `tests/dnp3_detection_tests.rs`
  - Update all existing `on_data(key, data, ts)` call sites to pass `Direction`
  - Add `mod direction_and_clock { ... }` with AC-140-001..009 named tests
  - Add `mod vp035_dnp3_carry_direction_isolation { ... }` with VP-035 proptest harnesses
  - Add `mod vp036_dnp3_window_monotonic_no_spurious_reset { ... }` with VP-036 proptest harnesses
- `tests/dnp3_flow_state_tests.rs` — update `on_data` call sites to pass `Direction`
- `tests/dnp3_parse_core_tests.rs` — update `on_data` call sites to pass `Direction`
- `tests/dnp3_correlation_tests.rs` — update `on_data` call sites to pass `Direction`
- `tests/dnp3_determinism_tests.rs` — update `on_data` call sites to pass `Direction`
- `tests/dnp3_f5_remediation_tests.rs` — update `on_data` call sites to pass `Direction`
- `tests/bc_2_15_110_dnp3_dispatcher_tests.rs` — update `on_data` call sites to pass `Direction`

**Files with F4-only amendments (DO NOT MODIFY in F3):**

- `docs/adr/0007` — ADR-007 amendment per RULING-DNP3-SIBLING-001 §6: carry split,
  `saturating_sub`, operator pin `>=` → `>`. Scheduled at F4 entry on factory-artifacts worktree.

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/dnp3.rs` changes (carry split, `on_data` sig, direction IP, saturating_sub, operator pin) | ~1,000 |
| `src/dispatcher.rs` DNP3 arm call-site update | ~50 |
| Existing test updates (add `Direction` arg to ~40 call sites across 7 test files) | ~300 |
| New `mod direction_and_clock` tests (8 tests) | ~600 |
| `mod vp035_dnp3_carry_direction_isolation` proptest harnesses (2 tests + `build_minimal_dnp3_frame` helper) | ~400 |
| `mod vp036_dnp3_window_monotonic_no_spurious_reset` proptest + deterministic tests (6 tests) | ~450 |
| BC files (4 BCs) | ~800 |
| **Total** | **~3,600** |

Context utilization: ~3,600 tokens / ~200,000 token window = ~1.8%. Well within the 20-30%
per-story budget.

## Dependency Rationale

Wave 63 (new wave added for this post-STORY-139 fix story).

**STORY-140 depends on STORY-139 because:** STORY-139 established and merged the
carry-split + Direction-threading dispatcher pattern for ENIP into `develop`. This confirms
the `StreamHandler` context provides `Direction` and the Modbus-mirror approach is correct.
STORY-140 applies the identical pattern to `dnp3.rs` and the DNP3 dispatcher arm. Requiring
STORY-139 to be merged first ensures the `develop` branch already has the precedent code that
implementers can mirror exactly. No conceptual dependency — pure build-order confirmation.
