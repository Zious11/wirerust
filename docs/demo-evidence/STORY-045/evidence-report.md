# STORY-045 Demo Evidence Report

**Story:** STORY-045 — Flow Lifecycle, Cross-Flow Isolation, and Buffer/Map Caps
**Wave:** 17
**Strategy:** brownfield-formalization (zero production behavior change; tests formalize
existing behavior that was already present in `src/analyzer/http.rs`)
**Test module:** `mod bc_2_06_045_formalization` in `tests/http_analyzer_tests.rs`
**Date:** 2026-05-29
**Suite result:** 17/17 PASS — `cargo test --all-targets` fully green (no failures across all modules)

---

## Per-AC Evidence Table

| AC | BC | Test Function(s) | Result | What It Proves |
|----|----|-----------------|--------|----------------|
| AC-001 | BC-2.06.019 | `test_BC_2_06_019_flow_close_removes_entry_and_preserves_aggregates` | PASS | `on_flow_close` removes the flow entry via `flows.remove(flow_key)`; aggregate counters (`transactions`, `parse_errors`, `non_http_flows`) are unchanged after close |
| AC-002 | BC-2.06.019 | `test_BC_2_06_019_reopen_same_key_starts_fresh_state` | PASS | After `on_flow_close`, `on_data` for the same FlowKey creates a brand-new `HttpFlowState::new()` with `request_poisoned=false`, `response_poisoned=false`, `error_count=0`, `counted_as_non_http=false`, and empty buffers |
| AC-003 | BC-2.06.019 | `test_BC_2_06_019_flow_close_removes_entry_and_preserves_aggregates` (CloseReason::Rst path); `test_BC_2_06_019_flow_close_on_unknown_key_is_noop` | PASS | `on_flow_close` ignores the `CloseReason` parameter (`_reason`); close with any reason (including Rst and close of unknown key) produces the same removal result |
| AC-004 | BC-2.06.021 | `test_BC_2_06_021_flow_a_parse_errors_do_not_affect_flow_b` | PASS | When flow A accumulates parse errors, flow B's `HttpFlowState` (`error_count`, `poison` flags) is completely unaffected; global aggregate counters are sums but do not gate per-flow behavior |
| AC-005 | BC-2.06.021 | `test_BC_2_06_021_flow_a_poisoning_does_not_affect_flow_b` | PASS | `flows: HashMap<FlowKey, HttpFlowState>` provides per-key isolation by construction; a poisoned flow A leaves flow B's state identical to a standalone-executed flow B |
| AC-006 | BC-2.06.022 | `test_BC_2_06_022_buffer_cap_exact_65536_no_more_bytes_accepted`; `test_BC_2_06_022_response_buffer_cap_exact_65536_no_more_bytes_accepted` | PASS | Per-direction buffer never exceeds `MAX_HEADER_BUF = 65,536` bytes; bytes past the cap are silently dropped without error or counter increment |
| AC-007 | BC-2.06.022 | `test_BC_2_06_022_buffer_cap_partial_fill_one_byte_appended`; `test_BC_2_06_022_response_buffer_cap_partial_fill_one_byte_appended` | PASS | `MAX_HEADER_BUF = 65,536` constant enforced; cap applies per-direction independently (request_buf and response_buf have separate limits); no finding emitted at cap |
| AC-008 | BC-2.06.024 | `test_BC_2_06_024_map_cardinality_cap_drops_new_keys`; `test_BC_2_06_024_hosts_map_cardinality_cap_independent_of_methods`; `test_BC_2_06_024_user_agents_map_cardinality_cap_independent_of_methods`; `test_BC_2_06_024_map_cardinality_cap_nth_entry_succeeds` | PASS | When a map (`methods`, `hosts`, or `user_agents`) reaches `MAX_MAP_ENTRIES = 50,000` distinct keys, new unique keys are silently not inserted; existing keys increment normally past cap; each map is capped independently |
| AC-009 | BC-2.06.024 | `test_BC_2_06_024_existing_keys_increment_at_cap` | PASS | `contains_key` short-circuit allows existing keys to increment even when map is at capacity (`if self.methods.len() < MAX_MAP_ENTRIES \|\| self.methods.contains_key(&parsed.method)`); `status_codes` uses u16 keys with no explicit cap guard |
| AC-010 | BC-2.06.025 | `test_BC_2_06_025_uris_capped_at_max_uris` | PASS | When `self.uris.len() == MAX_URIS (10,000)`, new request URIs are not appended; other counters (`methods`, `hosts`) are still updated for the request; no error or counter increment for dropped URI |
| AC-011 | BC-2.06.025 | `test_BC_2_06_025_uris_capped_at_max_uris`; `test_BC_2_06_025_uris_no_deduplication` | PASS | `MAX_URIS = 10,000` constant enforced; guard `if self.uris.len() < MAX_URIS { self.uris.push(...) }`; same URI can appear multiple times (no deduplication confirmed); URIs dropped at cap are permanently lost |

---

## Test Run Output

```
running 17 tests
test bc_2_06_045_formalization::test_BC_2_06_019_flow_close_on_unknown_key_is_noop ... ok
test bc_2_06_045_formalization::test_BC_2_06_019_partial_buf_discarded_on_close ... ok
test bc_2_06_045_formalization::test_BC_2_06_019_flow_close_removes_entry_and_preserves_aggregates ... ok
test bc_2_06_045_formalization::test_BC_2_06_021_flow_a_parse_errors_do_not_affect_flow_b ... ok
test bc_2_06_045_formalization::test_BC_2_06_019_reopen_same_key_starts_fresh_state ... ok
test bc_2_06_045_formalization::test_BC_2_06_022_buffer_cap_partial_fill_one_byte_appended ... ok
test bc_2_06_045_formalization::test_BC_2_06_022_response_buffer_cap_partial_fill_one_byte_appended ... ok
test bc_2_06_045_formalization::test_BC_2_06_022_response_buffer_cap_exact_65536_no_more_bytes_accepted ... ok
test bc_2_06_045_formalization::test_BC_2_06_021_flow_a_poisoning_does_not_affect_flow_b ... ok
test bc_2_06_045_formalization::test_BC_2_06_022_buffer_cap_exact_65536_no_more_bytes_accepted ... ok
test bc_2_06_045_formalization::test_BC_2_06_025_uris_no_deduplication ... ok
test bc_2_06_045_formalization::test_BC_2_06_025_uris_capped_at_max_uris ... ok
test bc_2_06_045_formalization::test_BC_2_06_024_map_cardinality_cap_nth_entry_succeeds ... ok
test bc_2_06_045_formalization::test_BC_2_06_024_map_cardinality_cap_drops_new_keys ... ok
test bc_2_06_045_formalization::test_BC_2_06_024_existing_keys_increment_at_cap ... ok
test bc_2_06_045_formalization::test_BC_2_06_024_hosts_map_cardinality_cap_independent_of_methods ... ok
test bc_2_06_045_formalization::test_BC_2_06_024_user_agents_map_cardinality_cap_independent_of_methods ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 115 filtered out; finished in 0.23s
```

---

## Additional Tests in Module (Beyond Story AC Scope)

The formalization module contains 2 extra tests that cover edge cases from the story's
EC list but are not directly cited in the AC table above:

| Test Function | Edge Case | Result |
|---|---|---|
| `test_BC_2_06_019_flow_close_on_unknown_key_is_noop` | EC-002: `on_flow_close` on unknown key is a no-op, no panic | PASS |
| `test_BC_2_06_019_partial_buf_discarded_on_close` | EC-003: partial request buffer discarded with flow state on close | PASS |

---

## Coverage Summary

- **ACs covered:** 11 / 11 (100%)
- **Tests exercised:** 17 (15 directly cited in ACs + 2 edge-case extras)
- **BCs traced:** BC-2.06.019, BC-2.06.021, BC-2.06.022, BC-2.06.024, BC-2.06.025
- **Full suite:** `cargo test --all-targets` — 0 failures across all modules
