---
document_type: story
story_id: "STORY-032"
epic_id: "E-3"
version: "1.4"
status: completed
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.004.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.005.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.006.md
input-hash: "223ffdd"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-031]
blocks: [STORY-033]
behavioral_contracts:
  - BC-2.05.004
  - BC-2.05.005
  - BC-2.05.006
verification_properties: [VP-004]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 13
target_module: src/dispatcher.rs
subsystems: [SS-05]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-032`

# STORY-032: Classification Caching and DispatchTarget::None Retry Budget

## Narrative
- **As a** forensic analyst
- **I want to** have the dispatcher cache flow classifications after the first successful match — so subsequent data chunks for the same flow skip re-classification — and retry unclassified flows up to `max_classification_attempts` times before permanently marking them as None
- **So that** analysis is efficient for long-lived flows and late-arriving content (e.g., mid-stream join) can eventually be classified

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.05.004 | Unknown Content + Unknown Port Returns DispatchTarget::None |
| BC-2.05.005 | Classification Cached Per FlowKey After First Non-None Result |
| BC-2.05.006 | DispatchTarget::None NOT Cached Until Retry Cap; Reclassification Retried Until Cap Then Cached Permanently |

## Acceptance Criteria

### AC-001 (traces to BC-2.05.004 postcondition 1-4)
When all three classification checks fail (TLS content, HTTP method prefix, and port fallback), `classify` returns `DispatchTarget::None`. Data is not forwarded to any analyzer. `classification_attempts[flow_key]` is incremented by 1.
- **Test:** `test_unclassified_flows_counter`

### AC-002 (traces to BC-2.05.004 postcondition 4)
When `classification_attempts[flow_key] >= max_classification_attempts` after increment, `DispatchTarget::None` is inserted into `routes` permanently, `classification_attempts` entry is removed, and future `on_data` calls short-circuit via the cached None route without calling `classify` again.
- **Test:** `test_BC_2_05_006_none_cached_permanently_after_retry_cap`

### AC-003 (traces to BC-2.05.004 invariant 1-2)
`DispatchTarget::None` is never cached in `routes` before the retry cap is hit (pre-cap calls re-run `classify` on every `on_data`). After the cap is hit, `DispatchTarget::None` IS cached and `classify` never runs again for that flow.
- **Test:** `test_BC_2_05_006_none_not_cached_before_retry_cap`

### AC-004 (traces to BC-2.05.005 postcondition 1-4)
After a flow is classified as Http or Tls, the result is stored in `routes: HashMap<FlowKey, DispatchTarget>`. Subsequent `on_data` calls for the same FlowKey use the cached target without re-running `classify`. The classification result is immutable — a cached Http flow cannot be reclassified as Tls even if later data starts with TLS bytes.
- **Test:** `test_BC_2_05_005_classification_cached_after_first_match`

### AC-005 (traces to BC-2.05.005 invariant 1)
Http and Tls are inserted into `routes` on first non-None classification. `DispatchTarget::None` is NOT inserted during the retry phase. Cached routes are removed only on `on_flow_close`.
- **Test:** `test_BC_2_05_005_classification_cached_after_first_match`

### AC-006 (traces to BC-2.05.006 postcondition Phase A)
Before the retry cap is reached, when `classify` returns None: `routes` does NOT contain an entry for this FlowKey, `classification_attempts[flow_key]` is incremented by 1, and on the next `on_data`, `routes.get(flow_key)` returns None triggering another `classify` call.
- **Test:** `test_BC_2_05_006_none_not_cached_before_retry_cap`

### AC-007 (traces to BC-2.05.006 postcondition Phase B)
When the retry cap is reached (`classification_attempts[flow_key]` reaches `max_classification_attempts`): `routes[flow_key] = DispatchTarget::None` is inserted permanently (dispatcher.rs:146), `classification_attempts[flow_key]` is removed (dispatcher.rs:147), and all subsequent `on_data` calls for this FlowKey short-circuit via the cached None route.
- **Test:** `test_BC_2_05_006_none_cached_permanently_after_retry_cap`

### AC-008 (traces to BC-2.05.006 invariant 3-4)
`max_classification_attempts` defaults to `DEFAULT_MAX_CLASSIFICATION_ATTEMPTS = 8` (dispatcher.rs:40). None is never inserted into `routes` BEFORE the cap; it IS inserted once the cap is hit. The cap is configurable (not hardcoded in the dispatch logic).
- **Test:** `test_BC_2_05_006_none_cached_permanently_after_retry_cap`

### AC-009 (traces to BC-2.05.006 edge case EC-001)
If the first N `on_data` calls return None and the (N+1)th returns TLS, on the (N+1)th call `routes[FlowKey]=Tls` is cached (dispatcher.rs:150) and `classification_attempts[FlowKey]` is removed (dispatcher.rs:151) — as long as N+1 <= max_classification_attempts.
- **Test:** `test_BC_2_05_006_late_classification_after_nones`

### AC-010 (traces to BC-2.05.005 invariant 2 + BC-2.05.006 Phase B postconditions + EC-005 + on_flow_close cleanup (dispatcher.rs:175-176) + STORY-032 EC-008)
After a flow is permanently cached as `DispatchTarget::None` (retry cap reached), calling `on_flow_close(&flow_key)` removes the entry from both `routes` and `classification_attempts`. Re-opening the same `FlowKey` with new content (e.g., TLS bytes) results in fresh classification — the previous None-cache does NOT persist across flow lifecycle boundaries. After the re-opened flow is classified (e.g., as Tls), subsequent `on_data` calls use the new cached target and do not re-run `classify`. Specifically: (1) during Phase B (cap=3, unknown bytes ×3), `on_data` with TLS bytes is short-circuited by the cached None route and `TlsAnalyzer` receives no data; (2) after `on_flow_close`, re-opening with TLS bytes causes `classify` to run and route data to `TlsAnalyzer` (producing at least one parse or truncation event); (3) a subsequent `on_data` with HTTP GET bytes does not reach `HttpAnalyzer` because the flow is now Tls-cached; (4) the second `on_flow_close` does not increment `unclassified_flows` beyond the count already set by the first close. (Test uses cap=3 rather than EC-008's illustrative cap=8 for runtime brevity; both are equivalent because the cap-comparison logic at dispatcher.rs:143 uses `>=` and is parametric in cap value.)
- **Test:** `test_BC_2_05_005_cache_evicted_on_flow_close_then_reclassified`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| routes HashMap | src/dispatcher.rs:133-154 | effectful-shell (stores FlowKey->DispatchTarget) |
| classification_attempts | src/dispatcher.rs:137-148 | effectful-shell (stores FlowKey->attempt count) |
| DEFAULT_MAX_CLASSIFICATION_ATTEMPTS const | src/dispatcher.rs:40 | pure-core (named constant) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | SSH bytes on port 22 (no match) | DispatchTarget::None; no analyzer call |
| EC-002 | 8 consecutive None results (default cap=8) | After 8th: None cached permanently |
| EC-003 | 9th call after cap hit | Cached None used; classify not called |
| EC-004 | max_classification_attempts=0 | Every flow immediately gets None cached on first chunk |
| EC-005 | Flow cached as Http; later TLS data on same flow | Stays Http (immutable cache) |
| EC-006 | 3 None then 1 TLS (cap=8) | Tls cached on 4th call; attempts removed |
| EC-007 | 7 None then 1 TLS (cap=8) | Tls cached on 8th call (cap not yet hit when Tls arrives) |
| EC-008 | 8 None (cap=8), then flow closed, same key reopened | New flow starts with no cache; re-classified from scratch |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/dispatcher.rs (routes/classification_attempts management) | effectful-shell | Mutates HashMaps on every on_data call |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| Referenced code (dispatcher.rs:133-154) | ~2,500 |
| Test files (dispatcher_tests.rs) | ~3,000 |
| BC files (3 BCs) | ~4,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~14,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement cache-hit path per BC-2.05.005 (routes.get on every on_data; use cached target; skip classify)
4. [ ] Implement classification caching for Http/Tls per BC-2.05.005 (routes.insert when target != None; attempts.remove)
5. [ ] Implement None non-caching + retry budget per BC-2.05.006 Phase A (increment attempts; do NOT insert routes)
6. [ ] Implement None permanent caching per BC-2.05.006 Phase B (at cap: routes.insert(None); attempts.remove)
7. [ ] Implement None return (unknown content + unknown port) per BC-2.05.004
8. [ ] Test late-classification scenario (N Nones then valid match per AC-009)
9. [ ] Run all tests; verify all pass
10. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-031 | `classify` is a pure function that takes `data` and `flow_key` and returns `DispatchTarget` without mutating state | Route caching happens in `on_data`, NOT in `classify` — classify is stateless | `DispatchTarget::None` has two distinct states: "not-yet-cached" (absent from routes) and "permanently-cached-None" (Some(None) in routes) |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `DispatchTarget::None` is NEVER inserted into routes before the retry cap is reached | BC-2.05.006 invariant 4 | Unit test: AC-006 |
| `DispatchTarget::None` IS inserted permanently once cap is hit | BC-2.05.006 postcondition Phase B | Unit test: AC-007 |
| `DEFAULT_MAX_CLASSIFICATION_ATTEMPTS = 8` is a named constant at dispatcher.rs:40 | BC-2.05.006 invariant 3 | Code review |
| Cached classifications are immutable — Http flows stay Http even if later TLS data arrives | BC-2.05.005 postcondition 4 | Unit test: AC-004 |
| `classification_attempts` entry is removed when a non-None target is cached (both Http/Tls AND permanent-None paths) | BC-2.05.006 (line 147 and 151) | Code review |
| Cache is fully evicted on `on_flow_close`; re-opening the same `FlowKey` restarts classification from scratch — the prior None-cache (or any cached target) is NOT reused | BC-2.05.005 invariant 2 | Unit test: AC-010 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | HashMap::get, HashMap::insert, HashMap::remove, entry API |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/dispatcher.rs | modify | on_data classification cache block (133-154): cache hit, None branch (137-148), non-None branch (149-151) |
| tests/dispatcher_tests.rs | modify | Add: test_BC_2_05_005_classification_cached_after_first_match, test_BC_2_05_006_none_not_cached_before_retry_cap, test_BC_2_05_006_none_cached_permanently_after_retry_cap, test_BC_2_05_006_late_classification_after_nones, test_BC_2_05_005_cache_evicted_on_flow_close_then_reclassified |

## Changelog

| Version | Date | Description | Author |
|---------|------|-------------|--------|
| v1.0 | 2026-05-21 | Initial story decomposition | story-writer |
| v1.1 | 2026-05-27 | W13 Pass 1 remediation: test-name refresh to match BC-prefixed actual names; status → in-progress | story-writer |
| v1.2 | 2026-05-27 | W13 Pass 2 remediation: add AC-010 for EC-008 traceability; propagate dispatcher.rs:137-148 line range to Architecture Mapping + File Structure Requirements; add 5th test to File Structure Requirements; update Task 1 AC range to 001-010; add ACR row for cache-eviction-on-close invariant | story-writer |
| v1.3 | 2026-05-27 | W13 Pass 3 polish: refine AC-010 EC trace precision (drop imprecise EC-003 cite; add Phase B postconditions + on_flow_close cleanup (dispatcher.rs:175-176)); add cap=3 vs cap=8 reconciliation note | story-writer |
| v1.4 | 2026-05-29 | status reconciled to completed per sprint-state.yaml (merge_commit 0d9b16d wave 13); F-DRIFT3B-001/PG-W16-002. | state-manager |
