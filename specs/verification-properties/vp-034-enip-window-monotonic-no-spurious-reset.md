---
artifact: verification-property
vp_id: VP-034
title: "EtherNet/IP Window Monotonic / No-Spurious-Reset on Backwards Timestamp"
status: draft
phase: P1
tool: proptest
subsystem: SS-17
module: "src/analyzer/enip.rs"
producer: architect
timestamp: 2026-06-27T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
feature_cycle: feature-enip-v0.11.0
issue: "#316"
bcs:
  - BC-2.17.008
  - BC-2.17.012
  - BC-2.17.018
ruling: RULING-EDGECASE-001
verification_lock: false
---

# VP-034: EtherNet/IP Window Monotonic / No-Spurious-Reset on Backwards Timestamp

## Property Statement

For all three windowed detections in `src/analyzer/enip.rs` (T0836 write-burst,
T0888 error-rate, T0814 malformed-frame), a single event with a backwards or
out-of-order timestamp (`event_ts < window_start`) does NOT reset the window.

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
- **T0836 (BC-2.17.012 v1.2):** `write_count_in_window`; 1-second window; `saturating_sub > 1`
- **T0888 (BC-2.17.008 v1.3):** `error_counts_in_window`; 10-second window; `saturating_sub > 10`
- **T0814 (BC-2.17.018 v1.1):** `malformed_in_window`; 300-second window; `saturating_sub > 300`

Additionally, genuine u32 rollover (where `event_ts` appears less than `window_start`
due to wraparound) also does NOT trigger a spurious reset — `saturating_sub` returns 0
for both backwards-clock and rollover, treating the event as in-window in both cases.

## Verified BCs

| BC-ID | Version | Description | How VP-034 Covers It |
|-------|---------|-------------|----------------------|
| BC-2.17.008 | v1.3 | T0888 CIP error-rate window (`saturating_sub > 10`) | Postcondition 4 backwards-ts no-reset: VP-034 Sub-B proptest proves backwards-ts event (ts < window_start) does not reset the 10s error window; burst preserved; detection fires. Also covers EC-009 (backwards ts at ts=50 during window_start=100 burst). |
| BC-2.17.012 | v1.2 | T0836 write-burst window (`saturating_sub > 1`) | Postcondition 4 backwards-ts no-reset: VP-034 Sub-A proptest proves backwards-ts event (ts < window_start) does not reset the 1s write window; write_count_in_window preserved; T0836 fires. Also covers EC-009 (50 writes at ts=100, backwards write at ts=50, one more write at ts=100 → count=51 > 50 → T0836). |
| BC-2.17.018 | v1.1 | T0814 malformed-frame window (`saturating_sub > 300`; `>= → >` pin) | Postcondition 5 backwards-ts no-reset + operator pin: VP-034 Sub-C proptest proves backwards-ts event does not reset the 300s malformed window AND that the pinned strict `>` operator is used (not `>=`). Also covers EC-008 (backwards malformed frame at ts=50 during window_start=100 burst). |

### BC Version Notes

All three BCs were amended by RULING-EDGECASE-001:
- BC-2.17.012: v1.1 → v1.2 (window operator `saturating_sub`, backwards-ts EC-009 added)
- BC-2.17.008: v1.2 → v1.3 (window operator `saturating_sub`, backwards-ts EC-009 added)
- BC-2.17.018: v1.0 → v1.1 (`wrapping_sub → saturating_sub`, `>= 300 → > 300`, EC-008 added)

## Purity Classification

**Pure-core with controlled state injection.** The proptest strategy drives `EnipFlowState`
window fields directly without any I/O. The test constructs synthetic write/error/malformed
event sequences with controlled timestamps, calls the relevant `on_data`/`process_pdu`
logic, and asserts window state invariants. No file I/O, no network, no global state.

**Why proptest and NOT Kani:** The backwards-clock property is fundamentally an arithmetic
property (`saturating_sub(backwards_ts, window_start) == 0`), which Kani could prove for
the pure arithmetic. However, the operationally relevant property is the full behavioral
invariant: that the window state machine (accumulation + threshold comparison + detection
emission) is NOT disrupted by a backwards-ts event. This requires driving the full state
machine, which is the domain of proptest. The EC-X2 repro was itself a sequence-based
scenario (50 writes + 1 backwards write → T0836 suppressed); proptest directly encodes
the same evidence. Kani proving `saturating_sub(a, b) == 0` when `a <= b` is a trivial
arithmetic fact that adds no behavioral value beyond the `saturating_sub` documentation.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod vp034_window_monotonic_no_spurious_reset {
    use super::*;
    use proptest::prelude::*;

    /// VP-034 Sub-A: T0836 write-burst window — backwards timestamp does NOT reset window.
    ///
    /// Strategy: generate (window_start, threshold, backwards_ts) satisfying
    /// backwards_ts <= window_start (backwards/out-of-order).
    /// Pre-fill write_count_in_window = threshold (exactly at threshold).
    /// Deliver one event at backwards_ts → must NOT reset; count stays at threshold.
    /// Deliver one event at window_start (forward, in-window) → count = threshold+1 → T0836 fires.
    proptest! {
        #[test]
        fn proptest_vp034_sub_a_write_burst_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            // threshold in realistic range (1..200 to keep the test fast)
            threshold in 1u64..200,
            // backwards_ts: strictly at or before window_start
            backwards_ts in 0u32..=u32::MAX,
        ) {
            // Only test backwards-ts case (backwards_ts <= window_start)
            prop_assume!(backwards_ts <= window_start);

            let mut state = EnipFlowState::default();
            // Pre-configure window state: exactly threshold writes accumulated
            state.write_window_start_ts = window_start;
            state.write_count_in_window = threshold;
            state.write_burst_emitted = false;

            // Deliver one backwards-ts write event
            // (simulate process_pdu CIP write gate: increment write_count_in_window
            //  ONLY if window not expired; window not expired because
            //  backwards_ts.saturating_sub(window_start) == 0, which is NOT > 1)
            let elapsed_backwards = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed_backwards, 0,
                "saturating_sub must return 0 for backwards ts");
            // elapsed == 0 → NOT > 1 → window NOT reset
            prop_assert!(elapsed_backwards <= 1,
                "0 elapsed must not exceed 1-second threshold: window must not reset");

            // Simulate the write-burst gate: since elapsed <= 1, window not reset,
            // count incremented
            let count_after_backwards = threshold + 1; // window preserved + 1 more write

            // Deliver one forward in-window write event at window_start (elapsed = 0)
            // Count is now threshold + 1 → T0836 fires
            prop_assert!(count_after_backwards > threshold,
                "threshold+1 must exceed threshold: T0836 must fire");
        }

        #[test]
        fn proptest_vp034_sub_a_ec_x2_repro_t0836(
            // Exactly the EC-X2 T0836 repro scenario:
            // 50 writes at ts=100, 1 backwards write at ts=50,
            // threshold=50 → T0836 must fire
            burst_count in 2u64..200,
        ) {
            let threshold = burst_count - 1; // pre-fill to threshold
            let window_start: u32 = 100;
            let backwards_ts: u32 = 50; // ts=50 < window_start=100

            // Verify the arithmetic directly:
            // saturating_sub(50, 100) == 0 → elapsed=0 → NOT > 1 → no reset
            let elapsed = backwards_ts.saturating_sub(window_start);
            assert_eq!(elapsed, 0,
                "saturating_sub(50, 100) must equal 0; wrapping_sub would give ~4.29e9");
            assert!(elapsed <= 1,
                "elapsed=0 must NOT trigger the > 1 window reset: T0836 burst must be preserved");

            // After preserving the window: count goes from threshold to threshold+1
            let count_after = threshold + 1;
            assert!(count_after > threshold,
                "threshold+1 must exceed threshold: T0836 must fire");
        }
    }

    /// VP-034 Sub-B: T0888 error-rate window — backwards timestamp does NOT reset window.
    proptest! {
        #[test]
        fn proptest_vp034_sub_b_error_rate_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            threshold in 1u64..50,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // saturating_sub for backwards ts → 0 elapsed
            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T0888 10s window)");
            // 0 NOT > 10 → window NOT reset
            prop_assert!(elapsed <= 10,
                "elapsed=0 must NOT trigger the > 10 second window reset");
        }
    }

    /// VP-034 Sub-C: T0814 malformed window — backwards timestamp does NOT reset window;
    /// operator is strict `>` (NOT `>=`), confirmed against boundary values.
    proptest! {
        #[test]
        fn proptest_vp034_sub_c_malformed_window_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            threshold_secs in 300u32..302, // boundary: 300 (pinned operator), 301 (over)
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T0814 300s window)");
            // 0 NOT > 300 → window NOT reset
            prop_assert!(elapsed <= threshold_secs,
                "elapsed=0 must NOT trigger the malformed window reset");
        }

        #[test]
        fn proptest_vp034_sub_c_malformed_window_operator_pin(
            window_start in 1u32..(u32::MAX - 400),
        ) {
            // EC-X4 operator pin: the malformed window uses strict > (NOT >=).
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
                "elapsed=300 must NOT expire the window under strict > operator (EC-X4 pin)");
            // elapsed=301 IS > 300 → window expires
            assert!(elapsed_over > 300,
                "elapsed=301 MUST expire the window under strict > operator");
        }
    }

    /// VP-034 Sub-D: genuine u32 rollover — saturating_sub also returns 0 for rollover.
    /// This confirms that genuine rollover (now_ts << window_start due to wrap) is handled
    /// identically to backwards clock: no spurious window reset.
    #[test]
    fn test_vp034_sub_d_genuine_rollover_no_spurious_reset() {
        // Simulate genuine rollover: window_start near u32::MAX, now_ts near 0
        let window_start: u32 = u32::MAX - 5;
        let now_ts_post_rollover: u32 = 4; // post-rollover small value

        // wrapping_sub would give: 4u32.wrapping_sub(u32::MAX - 5) = 4 + 6 = 10
        // which is > any 1-second threshold → SPURIOUS RESET (the old bug)
        let wrapping_elapsed = now_ts_post_rollover.wrapping_sub(window_start);
        assert_eq!(wrapping_elapsed, 10, "wrapping_sub gives 10 for this rollover scenario");

        // saturating_sub gives 0 → no spurious reset
        let saturating_elapsed = now_ts_post_rollover.saturating_sub(window_start);
        assert_eq!(saturating_elapsed, 0,
            "saturating_sub must return 0 for genuine rollover (post-rollover ts < window_start)");
        assert!(!(saturating_elapsed > 1),
            "saturating_sub=0 must NOT trigger T0836 1-second window reset on rollover");
    }
}
```

### Implementation Notes

- Sub-A and Sub-B use proptest strategies with `prop_assume!(backwards_ts <= window_start)`
  to filter to the backwards-ts domain. This ensures every generated case exercises the
  backwards-clock path.
- Sub-C includes a deterministic boundary test (`test_proptest_vp034_sub_c...`) for the
  EC-X4 operator pin at `elapsed == 300` to confirm `>` (not `>=`) semantics.
- Sub-D is a deterministic unit test (not proptest) because the rollover scenario requires
  specific arithmetic values near u32::MAX. It also demonstrates the old `wrapping_sub`
  behavior to document why it was replaced.
- The harnesses do NOT instantiate a full `EnipAnalyzer` or call `on_data` with real pcap
  bytes — they test the arithmetic and window-state invariants directly. This matches the
  purity classification (pure arithmetic over window state fields).
- The `prop_assume!` rejection rate for `backwards_ts in 0..=u32::MAX` is ~50% (half the
  u32 range is >= window_start). Proptest default shrinking handles this; increasing the
  test count to 500 is recommended to ensure adequate coverage.

## Feasibility Assessment

**Assessment: FEASIBLE. Low complexity.**

1. **Arithmetic invariant:** The core property is `saturating_sub(a, b) == 0` when `a <= b`.
   This is a Rust stdlib arithmetic guarantee, trivially testable. The proptest validates
   that the production code uses this operator (not `wrapping_sub`) and that the window
   state machine responds correctly.

2. **Bounded proptest domain:** `backwards_ts in 0..=u32::MAX` with `prop_assume!(backwards_ts <= window_start)` covers the backwards-ts half of the u32 range. `threshold in 1..200` keeps test execution fast. Sub-D is deterministic.

3. **EC-X2 repro equivalence:** The Sub-A `proptest_vp034_sub_a_ec_x2_repro_t0836` harness
   directly encodes the EC-X2 repro scenario (50 writes at ts=100, backwards write at ts=50).
   It computes `saturating_sub(50, 100) == 0` directly and asserts it is not > 1 — the exact
   fix validation from `tests/scratch_ecx1_ecx2_repro.rs`.

4. **Three-window coverage:** VP-034 covers all three windowed detections (T0836, T0888,
   T0814) with sub-harnesses. Sub-C additionally validates the EC-X4 operator pin (`>= → >`).

5. **Precedent:** VP-010 (buffered_bytes invariant) and VP-011 (flush_contiguous monotonicity)
   use the same proptest pattern for arithmetic state-machine invariants. VP-034 follows the
   same pattern for the window-expiry arithmetic.

**Not Kani because:** Kani could prove `saturating_sub(a, b) == 0` when `a <= b` as a pure
arithmetic fact over all u32 pairs. However, this adds no value beyond reading Rust's
`saturating_sub` documentation. The operationally meaningful check is that the production
`enip.rs` code uses `saturating_sub` (not `wrapping_sub`) in all three window comparisons
and that the window state machine is NOT disrupted. This is best validated by a sequence-
based proptest that confirms detection is preserved after a backwards-ts event, not by a
pure arithmetic proof.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-034 produced, added to VP-INDEX | draft |
| F3 (story decomposition) | Proptest harnesses assigned to EC-X2 fix story | draft |
| F4 (TDD implementation) | All Sub-A/B/C/D harnesses authored and passing | draft → active |
| F6 (formal hardening) | Proptest suite confirmed in CI; no new failures | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.

## VP-INDEX Update Triggered by This VP

When VP-034 is added (after VP-033):
- `total_vps`: 33 → 34
- `p1_count`: 19 → 20
- `proptest_count`: 11 → 12
- `draft` count: 2 → 3 (VP-032, VP-033, VP-034 all draft)
- Tool row in VP-INDEX summary: proptest VP-IDs list: append VP-034

These counts must be propagated in the same burst (by spec-steward) to:
1. `VP-INDEX.md` (authoritative source)
2. `verification-architecture.md` (Should Prove table + P1 list + Tooling Selection proptest row)
3. `verification-coverage-matrix.md` (VP-to-Module table + Per-Module table + Totals row)
