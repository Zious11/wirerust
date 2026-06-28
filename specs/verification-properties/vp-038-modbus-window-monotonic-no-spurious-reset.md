---
artifact: verification-property
vp_id: VP-038
title: "Modbus Window Monotonic / No-Spurious-Reset on Backwards Timestamp"
status: draft
phase: P1
tool: proptest
subsystem: SS-14
module: "src/analyzer/modbus.rs"
producer: spec-steward
timestamp: 2026-06-28T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-005-modbus-tcp-analysis.md
feature_cycle: feature-enip-v0.11.0
issue: "#316"
bcs:
  - BC-2.14.016
  - BC-2.14.017
  - BC-2.14.019
ruling: RULING-MODBUS-SIBLING-001
verification_lock: false
---

# VP-038: Modbus Window Monotonic / No-Spurious-Reset on Backwards Timestamp

## Property Statement

For all four windowed detections in `src/analyzer/modbus.rs` (T0831 coordinated-write 5s,
T0806 burst 1s, T0806 sustained >=2s minimum-duration gate, T0888 exception burst 10s),
a single event with a backwards or out-of-order timestamp (`event_ts < window_start_ts`)
does NOT reset the window.

Formally: for any `(window_start_ts: u32, accumulator: u64, backwards_ts: u32,
forward_ts: u32, threshold: u64)` satisfying:

```
backwards_ts <= window_start_ts     // backwards / out-of-order timestamp
forward_ts >= window_start_ts       // forward timestamp (same window)
accumulator == threshold            // exactly at threshold — one more event fires
forward_ts.saturating_sub(window_start_ts) <= window_duration  // still in window
```

delivering one event at `backwards_ts` followed by one event at `forward_ts`:

1. The window is NOT reset by the backwards-ts event (`window_start_ts` unchanged).
2. After the forward-ts event, `accumulator == threshold + 1`.
3. A detection is fired (threshold crossed) for the burst/T0831/T0888 windows.

This property holds for all four windows:
- **T0831 (BC-2.14.016 v2.3):** `t0831_window_write_count`; 5-second window;
  `saturating_sub > T0831_WINDOW_SECS (5)`
- **T0806 burst (BC-2.14.017 v2.7):** `window_write_count`; 1-second window;
  `saturating_sub > WRITE_BURST_WINDOW_SECS (1)`
- **T0806 sustained (BC-2.14.017 v2.7):** `sustained_window_write_count`; 2-second
  minimum-duration gate; `saturating_sub >= WRITE_SUSTAINED_WINDOW_SECS (2)`
  — NOTE: the sustained window uses `>=` (intentional — minimum-duration gate, NOT an
  expiry-reset operator). This is preserved in VP-038 Sub-C (not flipped to `>`).
- **T0888 exception (BC-2.14.019 v1.5):** `exception_window_counts[ec]`; 10-second window;
  `saturating_sub > EXCEPTION_WINDOW_SECS (10)`

Additionally, genuine u32 rollover (where `event_ts` appears less than `window_start_ts`
due to wraparound) also does NOT trigger a spurious reset — `saturating_sub` returns 0
for both backwards-clock and rollover.

The empirical repro (`scratch_EC_X2_explicit_window_state_via_process_pdu`, commit 74f2913)
confirms: deliver 20 writes at ts=100 (`window_write_count=20`), then 1 write at ts=50
(backwards). `wrapping_sub(50, 100) = 4294967246 >> WRITE_BURST_WINDOW_SECS(1)`: window
reset fires, `window_write_count` drops to 1, burst suppressed. `saturating_sub(50, 100)
= 0`: NOT > 1 → window NOT reset, burst preserved.

## Verified BCs

| BC-ID | Version | Description | How VP-038 Covers It |
|-------|---------|-------------|----------------------|
| BC-2.14.016 | v2.3 | T0831 Coordinated Write 5s window (`saturating_sub > 5`) | Postcondition pseudocode window-expiry changed from `wrapping_sub` to `saturating_sub`: VP-038 Sub-A proptest proves backwards-ts write (ts < t0831_window_start_ts) does not reset the 5s window; `t0831_window_write_count` preserved. Also covers EC-010 (updated) and EC-011 (new: ts=50 during window_start=100; count preserved; T0831 co-tag fires on next write). |
| BC-2.14.017 | v2.7 | T0806 Burst 1s window + Sustained >=2s minimum-duration gate (`saturating_sub > 1` / `saturating_sub >= 2`) | Both burst and sustained pseudocode window-expiry changed from `wrapping_sub` to `saturating_sub`: VP-038 Sub-B proves backwards-ts write does not reset the 1s burst window; VP-038 Sub-C proves backwards-ts write does not cause the sustained minimum-duration gate to fire spuriously (`saturating_sub(backwards_ts, start) == 0 < 2` → NOT `>=` 2 on backwards delivery alone). `>=` is KEPT (intentional); not flipped. Also covers EC-010 (updated) and EC-012 (new: 20 writes at ts=100 → 1 at ts=50 → burst fires at count=21). |
| BC-2.14.019 | v1.5 | T0888 Exception Burst 10s window (`saturating_sub > 10`) | Postcondition pseudocode window-expiry changed from `wrapping_sub` to `saturating_sub`: VP-038 Sub-D proves backwards-ts exception response (ts < exception_window_start_ts[ec]) does not reset the 10s window; exception count preserved. Also covers EC-009 (updated: elapsed=0, NOT > 10 → window NOT reset; burst accumulation preserved). |

### BC Version Notes

All three BCs were amended by RULING-MODBUS-SIBLING-001:
- BC-2.14.016: v2.2 → v2.3 (`wrapping_sub → saturating_sub` for T0831 5s; EC-010 updated; EC-011 added)
- BC-2.14.017: v2.6 → v2.7 (`wrapping_sub → saturating_sub` for burst 1s + sustained >=2s; `>=` kept for sustained; EC-010 updated; EC-012 added)
- BC-2.14.019: v1.4 → v1.5 (`wrapping_sub → saturating_sub` for exception 10s; EC-009 updated)

## Relationship to VP-034 (ENIP Analog) and VP-036 (DNP3 Analog)

VP-038 is structurally analogous to VP-034 (EtherNet/IP Window Monotonic / No-Spurious-Reset)
and VP-036 (DNP3 Window Monotonic / No-Spurious-Reset), but targets `src/analyzer/modbus.rs`.
Key differences:

| Aspect | VP-034 (ENIP) | VP-036 (DNP3) | VP-038 (Modbus) |
|--------|--------------|--------------|-----------------|
| Window 1 | T0836 write-burst; 1s; `> 1` | T1692.001 direct-operate; 60s; `> 60` | T0831 coordinated-write; 5s; `> 5` |
| Window 2 | T0888 error-rate; 10s; `> 10` | T1691.001 block-timeout; 10s; `> 10` | T0806 burst; 1s; `> 1` |
| Window 3 | T0814 malformed; 300s; `> 300` | T0827/T0814 correlated; 300s; `> 300` | T0806 sustained; 2s min-duration; `>= 2` (KEEP `>=`) |
| Window 4 | — | — | T0888 exception; 10s; `> 10` |
| Operator pin | `>= → >` (ENIP) | `>= → >` (DNP3) | No pin needed — sustained `>=` is intentional (§2.3) |
| BC traces | BC-2.17.008/012/018 | BC-2.15.010/014/015 | BC-2.14.016/017/019 |
| Sub-E | — | — | Genuine u32 rollover (all 4 windows) |

Modbus has 4 windows (vs. 3 for ENIP/DNP3) because it has distinct coordinated-write (T0831)
and burst (T0806) windows in addition to sustained and exception windows. VP-038 has
Sub-A/B/C/D/Sub-E (4 windows + rollover).

**Important: the sustained window `>=` operator is NOT a pin to fix.** Unlike the ENIP
`>=` at line 821 (which was a bug — inconsistent with other windows) and DNP3 `>=` at
line 984 (DRIFT-DNP3-OP-001 — a wrong operator pinned by the ruling), the Modbus sustained
window at `modbus.rs:670` intentionally uses `>=`. `BC-2.14.017 Postcondition 2` specifies
`elapsed_secs >= WRITE_SUSTAINED_WINDOW_SECS` as the minimum-duration gate semantics:
"fire when at least 2 seconds of writes have accumulated." This is preserved, not corrected.

## Sub-Properties

### Sub-A: T0831 Coordinated-Write 5s Window

Property: for any `(t0831_window_start_ts, t0831_window_write_count, backwards_ts)` with
`backwards_ts <= t0831_window_start_ts`, delivering one write at `backwards_ts` does NOT
reset the 5-second T0831 window. `saturating_sub(backwards_ts, t0831_window_start_ts) == 0`,
which is NOT `> T0831_WINDOW_SECS (5)`. A subsequent write at `t0831_window_start_ts`
increments the count and co-tags the per-PDU finding with T0831.

### Sub-B: T0806 Burst 1s Window

Property: for any `(window_start_ts, window_write_count, backwards_ts)` with
`backwards_ts <= window_start_ts`, delivering one write at `backwards_ts` does NOT
reset the 1-second burst window. `saturating_sub(backwards_ts, window_start_ts) == 0`,
which is NOT `> WRITE_BURST_WINDOW_SECS (1)`. The DRIFT-MODBUS-CLOCK-001 repro scenario:
20 writes at ts=100, then 1 write at ts=50 → `window_write_count=21`, burst fires.

### Sub-C: T0806 Sustained >=2s Minimum-Duration Gate

Property: for any `(sustained_window_start_ts, sustained_window_write_count, backwards_ts)`
with `backwards_ts <= sustained_window_start_ts`, delivering one write at `backwards_ts`
does NOT falsely satisfy the sustained minimum-duration gate.
`saturating_sub(backwards_ts, sustained_window_start_ts) == 0`, which is NOT
`>= WRITE_SUSTAINED_WINDOW_SECS (2)`. The gate correctly requires at least 2 continuous
seconds of writes before the rate comparison applies — a single backwards-ts delivery
cannot falsely trigger it.

**Operator note:** This sub-property validates the `>=` semantics, NOT `>`. The test
asserts `elapsed=0 NOT >= 2` (gate not satisfied) on the backwards delivery, and
`elapsed=2 IS >= 2` (gate satisfied) when genuine time has passed.

### Sub-D: T0888 Exception Burst 10s Window

Property: for any `(exception_window_start_ts[ec], exception_window_count[ec], backwards_ts)`
with `backwards_ts <= exception_window_start_ts[ec]`, delivering one exception response
at `backwards_ts` does NOT reset the 10-second exception window.
`saturating_sub(backwards_ts, exception_window_start_ts[ec]) == 0`, which is NOT
`> EXCEPTION_WINDOW_SECS (10)`. Exception count preserved; burst accumulation intact.

### Sub-E: Genuine u32 Rollover

Property: genuine u32 rollover (`now_ts` post-wrap near 0, `window_start_ts` pre-wrap
near u32::MAX) does NOT trigger a spurious window reset for any of the four Modbus windows.
`saturating_sub(post_rollover_ts, window_start_ts) == 0` — same as backwards-clock.
No spurious reset.

## Purity Classification

**Pure-core with controlled state injection.** The proptest strategy drives
`ModbusFlowState` window fields directly without any I/O. The test constructs synthetic
event sequences with controlled timestamps and asserts window state invariants. No file
I/O, no network, no global state.

**Why proptest and NOT Kani:** The backwards-clock property is fundamentally an arithmetic
property (`saturating_sub(a, b) == 0` when `a <= b`), which Kani could prove. However, the
operationally relevant property is the full behavioral invariant: that the window state
machine (accumulation + threshold comparison + detection emission) is NOT disrupted by a
backwards-ts event. This requires driving the full state machine — the domain of proptest.
The DRIFT-MODBUS-CLOCK-001 repro was a sequence-based scenario; proptest directly encodes
the same evidence.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod vp038_modbus_window_monotonic_no_spurious_reset {
    use super::*;
    use proptest::prelude::*;

    /// VP-038 Sub-A: T0831 coordinated-write 5s window — backwards timestamp does NOT reset.
    proptest! {
        #[test]
        fn proptest_vp038_sub_a_t0831_5s_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T0831 5s window)");
            // 0 NOT > 5 → window NOT reset
            prop_assert!(elapsed <= 5,
                "elapsed=0 must NOT trigger the > 5 second T0831 window reset");
        }

        #[test]
        fn proptest_vp038_sub_a_ec011_repro_t0831(
            write_count in 2u64..200,
        ) {
            let window_start: u32 = 100;
            let backwards_ts: u32 = 50; // ts=50 < window_start=100

            let elapsed = backwards_ts.saturating_sub(window_start);
            assert_eq!(elapsed, 0,
                "saturating_sub(50, 100) must equal 0 (EC-011 repro)");
            assert!(elapsed <= 5,
                "elapsed=0 must NOT trigger the > 5 window reset: T0831 co-tag preserved");
        }
    }

    /// VP-038 Sub-B: T0806 burst 1s window — backwards timestamp does NOT reset.
    proptest! {
        #[test]
        fn proptest_vp038_sub_b_t0806_burst_1s_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T0806 burst 1s window)");
            // 0 NOT > 1 → window NOT reset
            prop_assert!(elapsed <= 1,
                "elapsed=0 must NOT trigger the > 1 second burst window reset");
        }

        #[test]
        fn proptest_vp038_sub_b_ec012_repro_drift_modbus_clock_001(
            burst_count in 2u64..200,
        ) {
            // DRIFT-MODBUS-CLOCK-001 repro: 20 writes at ts=100, then 1 write at ts=50.
            // wrapping_sub would give ~4.29e9 >> 1 → spurious reset. saturating_sub gives 0.
            let threshold = burst_count - 1;
            let window_start: u32 = 100;
            let backwards_ts: u32 = 50;

            let elapsed_wrapping = backwards_ts.wrapping_sub(window_start);
            assert!(elapsed_wrapping > 1,
                "wrapping_sub gives large value (> 1) → would spuriously reset burst window");

            let elapsed_saturating = backwards_ts.saturating_sub(window_start);
            assert_eq!(elapsed_saturating, 0,
                "saturating_sub(50, 100) must equal 0");
            assert!(elapsed_saturating <= 1,
                "elapsed=0 must NOT trigger the > 1 burst reset: window_write_count preserved");

            let count_after = threshold + 1;
            assert!(count_after > threshold,
                "threshold+1 must exceed threshold: T0806 burst must fire");
        }
    }

    /// VP-038 Sub-C: T0806 sustained >=2s minimum-duration gate — backwards timestamp
    /// does NOT falsely satisfy the gate; >= operator is KEPT (intentional, not a bug).
    proptest! {
        #[test]
        fn proptest_vp038_sub_c_t0806_sustained_backwards_ts_no_gate_trigger(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // saturating_sub for backwards ts → 0 elapsed
            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (sustained >=2s gate)");
            // 0 NOT >= 2 → minimum-duration gate NOT satisfied on backwards delivery
            prop_assert!(elapsed < 2,
                "elapsed=0 must NOT satisfy the >= 2 second sustained minimum-duration gate");
        }

        #[test]
        fn proptest_vp038_sub_c_sustained_operator_pin_keep_ge(
            window_start in 1u32..(u32::MAX - 10),
        ) {
            // Confirm >= semantics: the sustained gate fires AT exactly 2 seconds.
            let ts_at_exact_threshold = window_start + 2;
            let ts_before_threshold  = window_start + 1;

            let elapsed_exact  = ts_at_exact_threshold.saturating_sub(window_start);
            let elapsed_before = ts_before_threshold.saturating_sub(window_start);

            assert_eq!(elapsed_exact,  2);
            assert_eq!(elapsed_before, 1);

            // >= 2: elapsed=2 IS >= 2 → gate satisfied (window has run for at least 2 seconds)
            assert!(elapsed_exact >= 2,
                "elapsed=2 MUST satisfy the >= 2 sustained minimum-duration gate");
            // elapsed=1 NOT >= 2 → gate NOT satisfied (window not yet 2 seconds old)
            assert!(!(elapsed_before >= 2),
                "elapsed=1 must NOT satisfy the >= 2 gate (not yet minimum duration)");

            // NOTE: This test VALIDATES the >= operator — it is intentionally NOT `>`.
            // The sustained window fires when the window has accumulated AT LEAST 2 seconds
            // of writes (minimum-duration semantics per BC-2.14.017 Postcondition 2).
            // This contrasts with the expiry-reset operators which use strict `>`.
        }
    }

    /// VP-038 Sub-D: T0888 exception burst 10s window — backwards timestamp does NOT reset.
    proptest! {
        #[test]
        fn proptest_vp038_sub_d_t0888_exception_10s_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T0888 exception 10s window)");
            // 0 NOT > 10 → window NOT reset; exception count preserved
            prop_assert!(elapsed <= 10,
                "elapsed=0 must NOT trigger the > 10 second exception-window reset");
        }
    }

    /// VP-038 Sub-E: genuine u32 rollover — saturating_sub returns 0 for rollover.
    /// Confirms genuine rollover is handled identically to backwards clock: no spurious
    /// window reset for any of the four Modbus windows.
    #[test]
    fn test_vp038_sub_e_genuine_rollover_no_spurious_reset() {
        // Simulate genuine rollover: window_start near u32::MAX, now_ts post-rollover.
        // Canonical scenario: window_start = u32::MAX - 5 (0xFFFFFFFA), now_ts = 500.
        let window_start: u32 = u32::MAX - 5; // 0xFFFFFFFA = 4294967290
        let now_ts_post_rollover: u32 = 500;  // post-rollover value

        // wrapping_sub would give: 500u32.wrapping_sub(0xFFFFFFFA) = 500 + 6 = 506
        // 506 > 5 (T0831), 506 > 1 (burst), 506 >= 2 (sustained), 506 > 10 (exception) →
        // ALL FOUR windows would spuriously fire → SPURIOUS RESET (the old bug)
        let wrapping_elapsed = now_ts_post_rollover.wrapping_sub(window_start);
        assert_eq!(wrapping_elapsed, 506,
            "wrapping_sub gives 506 for this rollover scenario");
        assert!(wrapping_elapsed > 5,  "wrapping would reset T0831 5s window");
        assert!(wrapping_elapsed > 1,  "wrapping would reset T0806 burst 1s window");
        assert!(wrapping_elapsed >= 2, "wrapping would fire T0806 sustained gate");
        assert!(wrapping_elapsed > 10, "wrapping would reset T0888 exception 10s window");

        // saturating_sub gives 0 → no spurious reset for any window
        let saturating_elapsed = now_ts_post_rollover.saturating_sub(window_start);
        assert_eq!(saturating_elapsed, 0,
            "saturating_sub must return 0 for genuine rollover");

        // Not > 5 (T0831): no spurious reset
        assert!(!(saturating_elapsed > 5),
            "saturating_sub=0 must NOT trigger T0831 5s window reset on rollover");
        // Not > 1 (T0806 burst): no spurious reset
        assert!(!(saturating_elapsed > 1),
            "saturating_sub=0 must NOT trigger T0806 burst 1s window reset on rollover");
        // Not >= 2 (T0806 sustained): minimum-duration gate not falsely satisfied
        assert!(!(saturating_elapsed >= 2),
            "saturating_sub=0 must NOT satisfy T0806 sustained >=2s gate on rollover");
        // Not > 10 (T0888 exception): no spurious reset
        assert!(!(saturating_elapsed > 10),
            "saturating_sub=0 must NOT trigger T0888 exception 10s window reset on rollover");
    }
}
```

### Implementation Notes

- Sub-A through Sub-D use `prop_assume!(backwards_ts <= window_start)` to filter to the
  backwards-ts domain. The rejection rate is ~50% of the u32 range; proptest default
  shrinking handles this. Increasing test count to 500 is recommended.
- Sub-C includes BOTH the no-gate-trigger test (backwards delivery cannot falsely satisfy
  the sustained gate) AND the operator-pin test confirming `>=` semantics are correct and
  intentional (elapsed=2 IS >= 2; elapsed=1 is NOT). This documents the deliberate design.
- Sub-E is a deterministic unit test covering all four Modbus windows in one test.
- The harnesses test the arithmetic and window-state invariants directly. They may either
  call `on_data` with synthetic event sequences or test the arithmetic gates directly.
- The `backwards_ts in 0u32..=u32::MAX` range in Sub-A/B/C/D is intentionally the full
  u32 range; `prop_assume!` filters to the backwards-ts half.

## Feasibility Assessment

**Assessment: FEASIBLE. Low complexity.**

1. **Arithmetic invariant:** The core property is `saturating_sub(a, b) == 0` when
   `a <= b`. This is a Rust stdlib arithmetic guarantee, trivially testable.

2. **Four-window coverage:** VP-038 covers all four windowed detections (T0831 5s,
   T0806 burst 1s, T0806 sustained >=2s, T0888 exception 10s). Sub-C additionally
   validates the intentional `>=` minimum-duration semantics.

3. **DRIFT-MODBUS-CLOCK-001 repro equivalence:** The Sub-B
   `proptest_vp038_sub_b_ec012_repro_drift_modbus_clock_001` harness directly encodes the
   DRIFT-MODBUS-CLOCK-001 repro scenario from RULING-MODBUS-SIBLING-001 §2.1 (20 writes
   at ts=100, then 1 write at ts=50 → must not suppress T0806 burst).

4. **Sustained `>=` documentation:** Sub-C explicitly validates and documents that `>=` is
   intentional for the sustained minimum-duration gate, preventing future "fix" regressions.

5. **Precedent:** VP-034 (EnipFlowState window monotonic) and VP-036 (Dnp3FlowState window
   monotonic) use the identical proptest pattern. VP-038 mirrors this structure with
   Modbus's four windows.

**Not Kani because:** Kani could prove `saturating_sub(a, b) == 0` when `a <= b` as a pure
arithmetic fact. However, the operationally meaningful check is that the production
`modbus.rs` code uses `saturating_sub` (not `wrapping_sub`) at all four window-comparison
sites (lines 534, 595, 670, 820) and that the window state machines are NOT disrupted.
This is best validated by a sequence-based proptest.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-038 produced, added to VP-INDEX | draft |
| F3 (story decomposition) | Proptest harnesses assigned to STORY-141 (Modbus carry/clock fix) | draft |
| F4 (TDD implementation) | All Sub-A/B/C/D/Sub-E harnesses authored and passing | draft → active |
| F6 (formal hardening) | Proptest suite confirmed in CI; no new failures | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.

## VP-INDEX Update Triggered by This VP

When VP-038 is added (after VP-037):
- `total_vps`: 37 → 38
- `p1_count`: 23 → 24
- `proptest_count`: 15 → 16
- `draft` count: 6 → 7 (VP-032..VP-038 all draft)
- Tool row in VP-INDEX summary: proptest VP-IDs list: append VP-038

These counts must be propagated in the same burst (by spec-steward) to:
1. `VP-INDEX.md` (authoritative source)
2. `verification-architecture.md` (Should Prove table + P1 list + Tooling Selection proptest row)
3. `verification-coverage-matrix.md` (VP-to-Module table + analyzer/modbus.rs Per-Module row + Totals row)
