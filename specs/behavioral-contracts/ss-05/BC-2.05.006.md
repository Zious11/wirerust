---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/dispatcher.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-05
capability: CAP-05
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
  - v1.3: W13 Pass 1 remediation: add test anchors for Phase A/B paths (F-W13P1-002); update stale coverage prose and VP-004 confidence to reflect STORY-032 closure (F-W13P1-003); tighten dispatcher.rs:136-148 to :137-148 (F-W13P1-007) — 2026-05-27
  - v1.4: W13 Pass 2 remediation: append test_BC_2_05_005_cache_evicted_on_flow_close_then_reclassified (F-W13P2-001, exercises Phase B postconditions + EC-005 cached-None short-circuit) and test_late_classification_within_attempt_budget_still_routes (sibling-sweep gap: Phase A late-classification path unanchored) to Architecture Anchors — 2026-05-27
  - v1.5: Pass-18 B-01/B-02 — re-anchored all dispatcher.rs line citations to current post-ICS-insertion positions. Most heavily stale BC: stale :40/:133-154/:137-148/:146/:147/:149-151/:175-176 → current :58/:269-290/:273-284/:282/:283/:286-287/:326-327. — 2026-06-13
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.006: DispatchTarget::None NOT Cached Until Retry Cap; Reclassification Retried Until Cap Then Cached Permanently

## Description

When `classify` returns `DispatchTarget::None`, the result is NOT inserted into the `routes`
HashMap (unlike Http or Tls results). Instead, `classification_attempts` is incremented and
`classify` will be re-run on the next `on_data` call for the same FlowKey. This allows late-
arriving data (e.g., a long-running connection whose first few chunks were small) to eventually
be classified when the buffer grows large enough to match a content signature.

Once `classification_attempts[flow_key]` reaches `max_classification_attempts` (default 8),
`DispatchTarget::None` IS permanently inserted into `routes` (dispatcher.rs:282), and the
`classification_attempts` counter for that flow is removed (dispatcher.rs:283). Subsequent
`on_data` calls for that flow short-circuit via the cached `None` route and never call
`classify` again. This two-phase behavior is the full contract: retry until the cap, then
cache permanently.

## Preconditions

### Phase A (before cap)
1. `classify` has returned `DispatchTarget::None` for this FlowKey.
2. `classification_attempts[flow_key] < max_classification_attempts`.

### Phase B (at cap)
1. `classify` has returned `DispatchTarget::None` again.
2. `classification_attempts[flow_key]` equals or exceeds `max_classification_attempts` after increment.

## Postconditions

### Phase A postconditions (count < cap after increment)
1. `routes` HashMap does NOT contain an entry for this FlowKey.
2. `classification_attempts[flow_key]` has been incremented by 1.
3. On the next `on_data`, `routes.get(flow_key)` returns None -> classify is re-run.

### Phase B postconditions (count >= cap after increment)
1. `routes[flow_key] = DispatchTarget::None` is inserted permanently (dispatcher.rs:282).
2. `classification_attempts[flow_key]` is removed (dispatcher.rs:283).
3. All subsequent `on_data` calls for this FlowKey short-circuit via the cached None route.
4. `classify` is never called again for this FlowKey for the rest of the flow's lifetime.

## Invariants

1. Phase A: `if target == DispatchTarget::None { increment count; if count >= cap { cache None permanently } }` (dispatcher.rs:273-284).
2. Phase B: Once cached as None, the flow stays None until `on_flow_close` evicts it.
3. Reclassification is bounded by `max_classification_attempts` (default 8) -- DEFAULT_MAX_CLASSIFICATION_ATTEMPTS at dispatcher.rs:58.
4. None is never inserted into `routes` BEFORE the cap is reached; it IS inserted once the cap is hit.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First 3 on_data calls return None; 4th returns TLS | On 4th: Tls cached via dispatcher.rs:286; attempts counter cleared via :287 |
| EC-002 | max_classification_attempts=8; 8 consecutive Nones | On 8th: count reaches 8 >= 8; None cached permanently via dispatcher.rs:282; attempts removed via :283 |
| EC-003 | Flow closed before cap reached | on_flow_close removes both routes and classification_attempts for the key (dispatcher.rs:326-327) |
| EC-004 | max_classification_attempts=0 | Every first chunk immediately hits cap; every flow is permanently None-cached on its first chunk |
| EC-005 | Flow cached as None; on_data called again | Short-circuits via routes.get -> cached; classify NOT called; DispatchTarget::None arm falls through silently |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SSH bytes (None) x3, then TLS bytes on 4th | routes[FlowKey]=Tls after 4th call | happy-path |
| Unknown content x2 then valid GET | routes[FlowKey]=Http after 3rd call | happy-path |
| Unknown content x8 (cap=8) | After 8th: routes[FlowKey]=None permanently; no classify on 9th | edge-case (Phase B) |
| Unknown content x7 (cap=8), then TLS bytes | On 8th call: classify returns Tls; routes[FlowKey]=Tls | edge-case (cap not yet hit) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | None result is not cached; reclassification retried | HIGH -- formalized by STORY-032 tests (test_BC_2_05_006_none_not_cached_before_retry_cap, test_BC_2_05_006_none_cached_permanently_after_retry_cap, test_BC_2_05_006_late_classification_after_nones) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md -- None-not-cached design allows late content classification for mid-stream-join scenarios |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- None is a temporary state, not a permanent classification) |
| Architecture Module | SS-05 (dispatcher.rs:269-290, C-21) |
| Stories | STORY-032 |
| Origin BC | BC-DSP-006 (pass-3 ingestion corpus, HIGH confidence post-STORY-032 (3 dedicated tests cover Phase A non-caching, Phase B permanent caching, and late-classification cache-clear)) |

## Related BCs

- BC-2.05.004 -- composes with (cap-triggered None caching is the eventual terminus)
- BC-2.05.005 -- related to (Http/Tls ARE cached; None is the asymmetric exception)

## Architecture Anchors

- `src/dispatcher.rs:269-290` -- classification cache + retry-budget block (on_data)
- `src/dispatcher.rs:273-284` -- None branch only (excludes line 272 classify() call): increment count; if count >= cap, cache None at :282, remove attempts at :283
- `src/dispatcher.rs:286-287` -- non-None branch: routes.insert target; classification_attempts.remove
- `src/dispatcher.rs:58` -- DEFAULT_MAX_CLASSIFICATION_ATTEMPTS = 8
- `tests/dispatcher_tests.rs` -- test_BC_2_05_006_none_not_cached_before_retry_cap, test_BC_2_05_006_none_cached_permanently_after_retry_cap, test_BC_2_05_006_late_classification_after_nones, test_late_classification_within_attempt_budget_still_routes, test_BC_2_05_005_cache_evicted_on_flow_close_then_reclassified

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:269-290` (full cache+retry block), `:273-284` (None branch only, excluding :272 classify() call), `:282` (permanent None insert) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if target == DispatchTarget::None { ... } else { routes.insert(...) }`
- **test-vector**: cache-not-inserted (Phase A) and cache-inserted-at-cap (Phase B) paths both independently verified by STORY-032 tests

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates classification_attempts (does NOT mutate routes) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
