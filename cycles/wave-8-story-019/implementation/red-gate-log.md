# Red Gate Log — STORY-019 (Wave 8)

**Date:** 2026-05-25
**Agent:** Test Writer (Part B)
**Story:** STORY-019 v1.2 — Flow Lifecycle: RST Close, FIN Close, Timeout Expiry, Missing-Key Warning

---

## Summary

STORY-019 Part B replaced all 28 Part A panic stubs with real BC-anchored assertions across two integration test files.

**This story targets brownfield-formalization**: the implementation already existed; tests verify it matches the behavioral contracts. All 24 non-gated tests passed on first run (PASS = existing implementation satisfies the BC).

---

## Test Files Modified

| File | Tests Added (STORY-019) |
|------|------------------------|
| `tests/reassembly_engine_tests.rs` | 23 real test bodies (AC-001..AC-012, EC-001..EC-008) + 4 cfg-gated (AC-013..AC-015, EC-009, EC-010) |
| `tests/reassembly_flow_tests.rs` | 4 real test bodies (flow-level: BC-2.04.010 + BC-2.04.011 state machine) |

---

## Red Gate Results: BC-2.04.010, BC-2.04.011, BC-2.04.013

All 23 engine tests + 4 flow tests that do NOT require the W8.3 test seam:

| Test Name | Result | Note |
|-----------|--------|------|
| `test_BC_2_04_010_rst_increments_flows_rst` | PASS | Brownfield: impl satisfies BC |
| `test_BC_2_04_010_rst_flushes_then_closes` | PASS | Brownfield |
| `test_BC_2_04_010_rst_payload_not_processed` | PASS | Brownfield |
| `test_BC_2_04_010_rst_closes_from_any_state` | PASS | Brownfield (4 distinct flow keys) |
| `test_BC_2_04_011_first_fin_transitions_to_closing` | PASS | Brownfield |
| `test_BC_2_04_011_second_fin_closes_flow` | PASS | Brownfield |
| `test_BC_2_04_011_fin_payload_processed_before_close` | PASS | Brownfield |
| `test_BC_2_04_011_same_direction_fin_retransmit_closes_flow` | PASS | Brownfield |
| `test_BC_2_04_013_expire_flows_closes_idle_flows` | PASS | Brownfield |
| `test_BC_2_04_013_expire_flows_does_not_close_active_flows` | PASS | Brownfield |
| `test_BC_2_04_013_expire_flows_does_not_underflow_when_time_travels_backwards` | PASS | Brownfield |
| `test_BC_2_04_013_already_closed_state_is_expired` | PASS | Brownfield |
| `test_BC_2_04_010_ec001_rst_on_new_flow_no_data` | PASS | Brownfield |
| `test_BC_2_04_010_ec002_rst_on_flow_with_buffered_data_flushes_first` | PASS | Brownfield |
| `test_BC_2_04_010_ec003_rst_packet_payload_is_discarded` | PASS | Brownfield |
| `test_BC_2_04_010_ec006_rst_and_fin_same_packet_rst_wins` | PASS | Brownfield |
| `test_BC_2_04_013_ec007_flow_idle_exactly_timeout_is_not_expired` | PASS | Brownfield (boundary: `>` not `>=`) |
| `test_BC_2_04_013_ec008_current_time_less_than_last_seen_no_expiry` | PASS | Brownfield (underflow guard) |
| `test_BC_2_04_010_flow_rst_on_closing_state_becomes_closed` | PASS | Brownfield |
| `test_BC_2_04_011_flow_first_fin_state_becomes_closing` | PASS | Brownfield |
| `test_BC_2_04_011_flow_second_fin_same_direction_reaches_closed` | PASS | Brownfield |
| `test_BC_2_04_011_flow_fin_on_new_state_transitions` | PASS | Brownfield (EC-004 edge) |

---

## Red Gate Results: BC-2.04.029 (BLOCKED — compile errors)

The following 4 tests require test seam functions not yet present in `src/reassembly/lifecycle.rs`.
They are `#[cfg(any())]`-gated (excluded from compilation) until W8.3 implements the seam.

**Missing functions (all in `wirerust::reassembly::lifecycle`):**

1. `reset_close_flow_missing_warned_for_testing() -> ()`
2. `close_flow_missing_warned_for_testing() -> bool`
3. `trigger_close_flow_missing_key_for_testing(reassembler: &mut TcpReassembler, key: &FlowKey, reason: CloseReason, handler: &mut dyn StreamHandler) -> ()`

**Additional blocker:** `mod lifecycle;` in `src/reassembly/mod.rs` is private (`mod lifecycle`, not `pub mod lifecycle`). Integration tests cannot access it as `wirerust::reassembly::lifecycle::`. The implementer must also change this to `pub mod lifecycle;` (or use a `pub use` re-export) OR use a different exposure mechanism.

| Test Name | Status | Blocked By |
|-----------|--------|-----------|
| `test_BC_2_04_029_close_flow_missing_key_warns_once` (AC-013+014) | cfg-gated | W8.3 seam missing |
| `test_BC_2_04_029_close_flow_missing_key_does_not_modify_state` (AC-015) | cfg-gated | W8.3 seam missing |
| `test_BC_2_04_029_ec009_already_warned_is_silent` | cfg-gated | W8.3 seam missing |
| `test_BC_2_04_029_ec010_close_flow_for_existing_key_is_normal` | cfg-gated | W8.3 seam missing |

---

## W8.3 Implementer Instructions

To unblock the 4 BC-2.04.029 tests, add to `src/reassembly/lifecycle.rs`:

```rust
/// Test-only accessor for the process-global CLOSE_FLOW_MISSING_WARNED flag.
#[doc(hidden)]
pub fn close_flow_missing_warned_for_testing() -> bool {
    CLOSE_FLOW_MISSING_WARNED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Test-only reset of the process-global CLOSE_FLOW_MISSING_WARNED flag.
#[doc(hidden)]
pub fn reset_close_flow_missing_warned_for_testing() {
    CLOSE_FLOW_MISSING_WARNED.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// Test-only trigger: calls close_flow with a key NOT in self.flows to exercise
/// the missing-key warning path. Requires TcpReassembler to expose this seam.
#[doc(hidden)]
pub fn trigger_close_flow_missing_key_for_testing(
    reassembler: &mut crate::reassembly::TcpReassembler,
    key: &crate::reassembly::flow::FlowKey,
    reason: crate::reassembly::handler::CloseReason,
    handler: &mut dyn crate::reassembly::handler::StreamHandler,
) {
    reassembler.close_flow(key, reason, handler);
}
```

Also change `mod lifecycle;` to `pub mod lifecycle;` in `src/reassembly/mod.rs` (line ~23).

After W8.3: remove the `#[cfg(any())]` attributes from the 4 tests and remove the `#[allow(dead_code)]` from `CLOSE_FLOW_MISSING_WARNED_LOCK`.

---

## Clippy

`cargo clippy --all-targets -- -D warnings` — CLEAN (no warnings or errors).

---

## Final State

- `cargo test --all-targets`: **529 passed, 0 failed** across all test binaries
- 4 BC-2.04.029 tests cfg-gated pending W8.3 seam implementation
- No commits made (as instructed)
