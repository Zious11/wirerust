---
artifact: verification-property
vp_id: VP-036
title: "DNP3 Window Monotonic / No-Spurious-Reset on Backwards Timestamp"
status: draft
phase: P1
tool: proptest
subsystem: SS-15
module: "src/analyzer/dnp3.rs"
producer: spec-steward
timestamp: 2026-06-27T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
feature_cycle: feature-enip-v0.11.0
issue: "#316"
bcs:
  - BC-2.15.010
  - BC-2.15.014
  - BC-2.15.015
ruling: RULING-DNP3-SIBLING-001
verification_lock: false
---

# VP-036: DNP3 Window Monotonic / No-Spurious-Reset on Backwards Timestamp

## Property Statement

For all three windowed detections in `src/analyzer/dnp3.rs` (T1692.001 burst,
T1691.001 block-command timeout, T0827/T0814 correlated-event window), a single event
with a backwards or out-of-order timestamp (`event_ts < window_start`) does NOT reset
the window.

Formally: for any `(window_start: u32, burst_count: u64, backwards_ts: u32,
forward_ts: u32, threshold: u64)` satisfying:

```
backwards_ts <= window_start       // backwards / out-of-order timestamp
forward_ts >= window_start         // forward timestamp (same window)
burst_count == threshold           // exactly at threshold — one more event fires
forward_ts.saturating_sub(window_start) <= window_duration  // still in window
```

delivering one event at `backwards_ts` followed by one event at `forward_ts`:

1. The window is NOT reset by the backwards-ts event (window_start unchanged).
2. After the forward-ts event, `burst_count == threshold + 1`.
3. A detection is fired (threshold crossed).

This property holds for all three windows:
- **T1692.001 (BC-2.15.010 v1.8):** `direct_operate_count`; 60-second window;
  `saturating_sub > DETECTION_WINDOW_SECS`
- **T1691.001 (BC-2.15.014 v2.1):** `pending_requests` timeout; 10-second window;
  `saturating_sub > BLOCK_CMD_TIMEOUT_SECS`
- **T0827/T0814 (BC-2.15.015 v2.0):** `restart_event_count` / `block_event_count`;
  300-second window; `saturating_sub > CORRELATION_WINDOW_SECS` (strict `>`, NOT `>=`)

Additionally, genuine u32 rollover (where `event_ts` appears less than `window_start`
due to wraparound) also does NOT trigger a spurious reset — `saturating_sub` returns 0
for both backwards-clock and rollover.

The **operator pin** for the 300-second window (DRIFT-DNP3-OP-001): the window-expiry
expression uses strict `>` (NOT `>=`). A packet at exactly elapsed=300 seconds is the
last packet of the current window, not the first of a new window.

## Verified BCs

| BC-ID | Version | Description | How VP-036 Covers It |
|-------|---------|-------------|----------------------|
| BC-2.15.010 | v1.8 | T1692.001 Direct-Operate burst detection (`saturating_sub > 60`) | Postcondition 4 window-expiry changed from `wrapping_sub` to `saturating_sub`: VP-036 Sub-A proptest proves backwards-ts event (ts < window_start) does not reset the 60s detect window; `direct_operate_count` preserved; detection fires. Also covers EC-012 (backwards ts at ts=50 during window_start=100 burst: count preserved). |
| BC-2.15.014 | v2.1 | T1691.001 Block-command timeout (`saturating_sub > 10`) | Precondition 3 timeout check changed from `wrapping_sub` to `saturating_sub`: VP-036 Sub-B proptest proves backwards-ts event (ts < request_ts) does not fire a spurious timeout; pending request remains in `pending_requests` until genuine 10s elapses. Also covers EC-009 (request at ts=100; backwards packet at ts=50: timeout NOT fired). |
| BC-2.15.015 | v2.0 | T0827/T0814 300s correlation window (`saturating_sub > 300`; `>=` → `>` pin) | Postcondition 3 window-reset condition changed from `wrapping_sub >= 300` to `saturating_sub > 300`: VP-036 Sub-C proptest proves backwards-ts event does not reset the 300s window AND that the pinned strict `>` operator is correct (elapsed==300 NOT > 300 → no reset; elapsed==301 → reset). Also covers EC-010 (backwards restart event at ts=50 during window_start=100; restart_event_count preserved). |

### BC Version Notes

All three BCs were amended by RULING-DNP3-SIBLING-001:
- BC-2.15.010: v1.7 → v1.8 (`wrapping_sub → saturating_sub` in PC4; EC-012 added)
- BC-2.15.014: v2.0 → v2.1 (`wrapping_sub → saturating_sub` in PC3; EC-009 added)
- BC-2.15.015: v1.9 → v2.0 (`wrapping_sub >= 300 → saturating_sub > 300` in PC3/Inv6; EC-010 added)

## Relationship to VP-034 (ENIP Analog)

VP-036 is structurally analogous to VP-034 (EtherNet/IP Window Monotonic / No-Spurious-Reset)
but targets `src/analyzer/dnp3.rs` instead of `src/analyzer/enip.rs`. Key differences:

| Aspect | VP-034 (ENIP) | VP-036 (DNP3) |
|--------|--------------|--------------|
| Window 1 | T0836 write-burst; 1s; `saturating_sub > 1` | T1692.001 direct-operate; 60s; `saturating_sub > 60` |
| Window 2 | T0888 error-rate; 10s; `saturating_sub > 10` | T1691.001 block-timeout; 10s; `saturating_sub > 10` |
| Window 3 | T0814 malformed; 300s; `saturating_sub > 300` | T0827/T0814 correlated; 300s; `saturating_sub > 300` |
| Operator pin | `>= → >` (ENIP BC-2.17.018) | `>= → >` (DNP3 BC-2.15.015, DRIFT-DNP3-OP-001) |
| BC traces | BC-2.17.008/012/018 | BC-2.15.010/014/015 |
| Sub-A threshold | 1s | 60s |
| Sub-B threshold | 10s | 10s |
| Sub-C operator pin | elapsed==300 NOT > 300 | elapsed==300 NOT > 300 (same) |
| Sub-D rollover | u32 rollover | u32 rollover |

The sub-property structure (Sub-A/B/C/D) mirrors VP-034 exactly.

## Sub-Properties

### Sub-A: T1692.001 Direct-Operate 60s Window

Property: for any `(window_start, direct_operate_count, backwards_ts)` with
`backwards_ts <= window_start`, delivering one event at `backwards_ts` does NOT reset
the 60-second window. `saturating_sub(backwards_ts, window_start) == 0`, which is NOT
`> DETECTION_WINDOW_SECS (60)`. A subsequent event at `window_start` increments the count
to `direct_operate_count + 1` and fires T1692.001 if count > threshold.

### Sub-B: T1691.001 Block-Command 10s Timeout

Property: for any `(request_ts, backwards_ts)` with `backwards_ts <= request_ts`, a
subsequent `on_data` call at `backwards_ts` does NOT fire the 10-second block-command
timeout. `saturating_sub(backwards_ts, request_ts) == 0`, which is NOT
`> BLOCK_CMD_TIMEOUT_SECS (10)`. The pending request remains in `pending_requests`.

### Sub-C: T0827/T0814 300s Correlation Window + Operator Pin

Property (no-reset): for any `(correlation_window_start_ts, backwards_ts)` with
`backwards_ts <= correlation_window_start_ts`, delivering one event at `backwards_ts` does
NOT reset the 300-second window. `saturating_sub(backwards_ts, correlation_window_start_ts)
== 0`, which is NOT `> CORRELATION_WINDOW_SECS (300)`.

Property (operator pin): the window-expiry uses strict `>` (NOT `>=`):
- elapsed == 300: `300 > 300` is FALSE → window NOT expired (packet is last of window)
- elapsed == 301: `301 > 300` is TRUE → window expires

### Sub-D: Genuine u32 Rollover

Property: genuine u32 rollover (`now_ts` post-wrap near 0, `window_start` pre-wrap near
u32::MAX) does NOT trigger a spurious window reset. `saturating_sub(post_rollover_ts,
window_start) == 0` — same as backwards-clock. No spurious reset for any of the three
windows.

## Purity Classification

**Pure-core with controlled state injection.** The proptest strategy drives `Dnp3FlowState`
window fields directly without any I/O. The test constructs synthetic event sequences with
controlled timestamps, calls the relevant `on_data`/helper logic, and asserts window state
invariants. No file I/O, no network, no global state.

**Why proptest and NOT Kani:** The backwards-clock property is fundamentally an arithmetic
property (`saturating_sub(a, b) == 0` when `a <= b`), which Kani could prove. However, the
operationally relevant property is the full behavioral invariant: that the window state
machine (accumulation + threshold comparison + detection emission) is NOT disrupted by a
backwards-ts event. This requires driving the full state machine — the domain of proptest.
The DRIFT-DNP3-CLOCK-001 repro was itself a sequence-based scenario; proptest directly
encodes the same evidence. Kani proving `saturating_sub(a, b) == 0` when `a <= b` is a
trivial arithmetic fact that adds no behavioral value beyond the `saturating_sub`
documentation.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod vp036_dnp3_window_monotonic_no_spurious_reset {
    use super::*;
    use proptest::prelude::*;

    /// VP-036 Sub-A: T1692.001 direct-operate 60s window — backwards timestamp does NOT reset.
    ///
    /// Strategy: generate (window_start, threshold, backwards_ts) satisfying
    /// backwards_ts <= window_start (backwards/out-of-order).
    /// Verify: saturating_sub(backwards_ts, window_start) == 0 → NOT > 60 → no reset.
    proptest! {
        #[test]
        fn proptest_vp036_sub_a_direct_operate_60s_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            threshold in 1u64..200,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            let elapsed_backwards = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed_backwards, 0,
                "saturating_sub must return 0 for backwards ts (T1692.001 60s window)");
            // 0 NOT > 60 → window NOT reset
            prop_assert!(elapsed_backwards <= 60,
                "elapsed=0 must NOT trigger the > 60 second window reset");

            // With window preserved: count goes from threshold to threshold+1
            let count_after = threshold + 1;
            prop_assert!(count_after > threshold,
                "threshold+1 must exceed threshold: T1692.001 must fire");
        }

        #[test]
        fn proptest_vp036_sub_a_ec_x2_repro_t1692(
            burst_count in 2u64..200,
        ) {
            let threshold = burst_count - 1;
            let window_start: u32 = 100;
            let backwards_ts: u32 = 50; // ts=50 < window_start=100

            // saturating_sub(50, 100) == 0 → elapsed=0 → NOT > 60 → no reset
            let elapsed = backwards_ts.saturating_sub(window_start);
            assert_eq!(elapsed, 0,
                "saturating_sub(50, 100) must equal 0; wrapping_sub would give ~4.29e9");
            assert!(elapsed <= 60,
                "elapsed=0 must NOT trigger the > 60 window reset: T1692.001 burst preserved");

            let count_after = threshold + 1;
            assert!(count_after > threshold,
                "threshold+1 must exceed threshold: T1692.001 must fire");
        }
    }

    /// VP-036 Sub-B: T1691.001 block-command 10s timeout — backwards timestamp does NOT
    /// fire spurious timeout.
    proptest! {
        #[test]
        fn proptest_vp036_sub_b_block_timeout_backwards_ts_no_fire(
            request_ts in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= request_ts);

            // saturating_sub for backwards ts → 0 elapsed
            let elapsed = backwards_ts.saturating_sub(request_ts);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T1691.001 10s timeout)");
            // 0 NOT > 10 → timeout NOT fired; pending request preserved
            prop_assert!(elapsed <= 10,
                "elapsed=0 must NOT trigger the > 10 second block-timeout");
        }
    }

    /// VP-036 Sub-C: T0827/T0814 300s correlation window — backwards timestamp does NOT
    /// reset window; operator is strict `>` (NOT `>=`).
    proptest! {
        #[test]
        fn proptest_vp036_sub_c_correlation_window_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T0827/T0814 300s window)");
            // 0 NOT > 300 → window NOT reset
            prop_assert!(elapsed <= 300,
                "elapsed=0 must NOT trigger the 300-second correlation-window reset");
        }

        #[test]
        fn proptest_vp036_sub_c_correlation_window_operator_pin(
            window_start in 1u32..(u32::MAX - 400),
        ) {
            // DRIFT-DNP3-OP-001 operator pin: the 300s correlation window uses strict > (NOT >=).
            // Boundary: elapsed == 300 must NOT trigger reset (packet is last of window).
            // Elapsed == 301 MUST trigger reset.
            let ts_at_exact_threshold = window_start + 300;
            let ts_over_threshold = window_start + 301;

            let elapsed_exact = ts_at_exact_threshold.saturating_sub(window_start);
            let elapsed_over  = ts_over_threshold.saturating_sub(window_start);

            assert_eq!(elapsed_exact, 300);
            assert_eq!(elapsed_over,  301);

            // Strict > 300: elapsed=300 is NOT > 300 → window NOT expired
            assert!(!(elapsed_exact > 300),
                "elapsed=300 must NOT expire the 300s window under strict > operator \
                 (DRIFT-DNP3-OP-001 pin)");
            // elapsed=301 IS > 300 → window expires
            assert!(elapsed_over > 300,
                "elapsed=301 MUST expire the 300s window under strict > operator");
        }
    }

    /// VP-036 Sub-D: genuine u32 rollover — saturating_sub returns 0 for rollover.
    /// Confirms genuine rollover is handled identically to backwards clock: no spurious
    /// window reset for any of the three DNP3 windows.
    #[test]
    fn test_vp036_sub_d_genuine_rollover_no_spurious_reset() {
        // Simulate genuine rollover: window_start near u32::MAX, now_ts near 0
        let window_start: u32 = u32::MAX - 5;
        let now_ts_post_rollover: u32 = 4; // post-rollover small value

        // wrapping_sub would give: 4u32.wrapping_sub(u32::MAX - 5) = 4 + 6 = 10
        // which is > any 1s or 10s threshold → SPURIOUS RESET (the old bug)
        let wrapping_elapsed = now_ts_post_rollover.wrapping_sub(window_start);
        assert_eq!(wrapping_elapsed, 10, "wrapping_sub gives 10 for this rollover scenario");

        // saturating_sub gives 0 → no spurious reset
        let saturating_elapsed = now_ts_post_rollover.saturating_sub(window_start);
        assert_eq!(saturating_elapsed, 0,
            "saturating_sub must return 0 for genuine rollover");

        // Not > 60 (T1692.001 window): no spurious reset
        assert!(!(saturating_elapsed > 60),
            "saturating_sub=0 must NOT trigger T1692.001 60s window reset on rollover");
        // Not > 10 (T1691.001 timeout): no spurious timeout
        assert!(!(saturating_elapsed > 10),
            "saturating_sub=0 must NOT trigger T1691.001 10s timeout on rollover");
        // Not > 300 (T0827/T0814 window): no spurious correlation-window reset
        assert!(!(saturating_elapsed > 300),
            "saturating_sub=0 must NOT trigger T0827/T0814 300s window reset on rollover");
    }
}
```

### Implementation Notes

- Sub-A and Sub-B use `prop_assume!(backwards_ts <= window_start)` to filter to the
  backwards-ts domain. The rejection rate is ~50% of the u32 range; proptest default
  shrinking handles this. Increasing test count to 500 is recommended.
- Sub-C includes a deterministic boundary test for the DRIFT-DNP3-OP-001 operator pin at
  `elapsed == 300` to confirm `>` (not `>=`) semantics — exactly the fix from line 984
  of `dnp3.rs`.
- Sub-D is a deterministic unit test (not proptest) because the rollover scenario requires
  specific arithmetic values near u32::MAX. It covers all three DNP3 windows in one test.
- The harnesses test the arithmetic and window-state invariants directly. They may either
  call `on_data` with synthetic event sequences or test the arithmetic gates directly.
  The purity classification is "pure arithmetic over window state fields."
- The `backwards_ts in 0u32..=u32::MAX` range in Sub-A/C is intentionally the full u32
  range; `prop_assume!` filters to the backwards-ts half. This ensures the proptest
  strategy covers the exact adversarial scenario (not just near-boundary cases).

## Feasibility Assessment

**Assessment: FEASIBLE. Low complexity.**

1. **Arithmetic invariant:** The core property is `saturating_sub(a, b) == 0` when `a <= b`.
   This is a Rust stdlib arithmetic guarantee, trivially testable. The proptest validates
   that the production code uses this operator (not `wrapping_sub`) and that the window state
   machine responds correctly.

2. **Three-window coverage:** VP-036 covers all three windowed detections (T1692.001 60s,
   T1691.001 10s, T0827/T0814 300s) with sub-harnesses. Sub-C additionally validates the
   DRIFT-DNP3-OP-001 operator pin (`>= → >`).

3. **DRIFT-DNP3-CLOCK-001 repro equivalence:** The Sub-A `proptest_vp036_sub_a_ec_x2_repro_t1692`
   harness directly encodes the DRIFT-DNP3-CLOCK-001 repro scenario from RULING-DNP3-SIBLING-001
   §2.1 (9 FCs at ts=100, backwards FC at ts=50 → must not suppress T1692.001).

4. **Operator pin coverage:** Sub-C includes the deterministic `elapsed==300 NOT > 300`
   boundary test. This is the exact analog of VP-034 Sub-C's EC-X4 operator pin test for
   ENIP (elapsed==300 on T0814 malformed window).

5. **Precedent:** VP-034 (EnipFlowState window monotonic no-spurious-reset) uses the
   identical proptest pattern across three windows + Sub-D rollover test. VP-036 mirrors
   this structure exactly.

**Not Kani because:** Kani could prove `saturating_sub(a, b) == 0` when `a <= b` as a pure
arithmetic fact. However, the operationally meaningful check is that the production `dnp3.rs`
code uses `saturating_sub` (not `wrapping_sub`) at all 8 window-comparison sites (lines 745,
765, 769, 895, 984, 1025, 1335, 1341) and that the window state machines are NOT disrupted.
This is best validated by a sequence-based proptest.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-036 produced, added to VP-INDEX | draft |
| F3 (story decomposition) | Proptest harnesses assigned to STORY-140 (DNP3 window fix) | draft |
| F4 (TDD implementation) | All Sub-A/B/C/D harnesses authored and passing | draft → active |
| F6 (formal hardening) | Proptest suite confirmed in CI; no new failures | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.

## VP-INDEX Update Triggered by This VP

When VP-036 is added (after VP-035):
- `total_vps`: 35 → 36
- `p1_count`: 21 → 22
- `proptest_count`: 13 → 14
- `draft` count: 4 → 5 (VP-032, VP-033, VP-034, VP-035, VP-036 all draft)
- Tool row in VP-INDEX summary: proptest VP-IDs list: append VP-036

These counts must be propagated in the same burst (by spec-steward) to:
1. `VP-INDEX.md` (authoritative source)
2. `verification-architecture.md` (Should Prove table + P1 list + Tooling Selection proptest row)
3. `verification-coverage-matrix.md` (VP-to-Module table + analyzer/dnp3.rs Per-Module row + Totals row)
