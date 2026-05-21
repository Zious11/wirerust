---
document_type: behavioral-contract
level: L3
version: "1.2"
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
`DispatchTarget::None` IS permanently inserted into `routes` (dispatcher.rs:146), and the
`classification_attempts` counter for that flow is removed (dispatcher.rs:147). Subsequent
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
1. `routes[flow_key] = DispatchTarget::None` is inserted permanently (dispatcher.rs:146).
2. `classification_attempts[flow_key]` is removed (dispatcher.rs:147).
3. All subsequent `on_data` calls for this FlowKey short-circuit via the cached None route.
4. `classify` is never called again for this FlowKey for the rest of the flow's lifetime.

## Invariants

1. Phase A: `if target == DispatchTarget::None { increment count; if count >= cap { cache None permanently } }` (dispatcher.rs:137-148).
2. Phase B: Once cached as None, the flow stays None until `on_flow_close` evicts it.
3. Reclassification is bounded by `max_classification_attempts` (default 8) -- DEFAULT_MAX_CLASSIFICATION_ATTEMPTS at dispatcher.rs:40.
4. None is never inserted into `routes` BEFORE the cap is reached; it IS inserted once the cap is hit.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First 3 on_data calls return None; 4th returns TLS | On 4th: Tls cached via dispatcher.rs:150; attempts counter cleared via :151 |
| EC-002 | max_classification_attempts=8; 8 consecutive Nones | On 8th: count reaches 8 >= 8; None cached permanently via dispatcher.rs:146; attempts removed via :147 |
| EC-003 | Flow closed before cap reached | on_flow_close removes both routes and classification_attempts for the key (dispatcher.rs:175-176) |
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
| VP-004 | None result is not cached; reclassification retried | MEDIUM -- inferred from code; only insert path tested |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 -- None-not-cached design allows late content classification for mid-stream-join scenarios |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- None is a temporary state, not a permanent classification) |
| Architecture Module | SS-05 (dispatcher.rs:133-154, C-21) |
| Stories | STORY-032 |
| Origin BC | BC-DSP-006 (pass-3 ingestion corpus, MEDIUM confidence -- inferred from code) |

## Related BCs

- BC-2.05.004 -- composes with (cap-triggered None caching is the eventual terminus)
- BC-2.05.005 -- related to (Http/Tls ARE cached; None is the asymmetric exception)

## Architecture Anchors

- `src/dispatcher.rs:133-154` -- classification cache + retry-budget block (on_data)
- `src/dispatcher.rs:136-148` -- None branch: increment count; if count >= cap, cache None at :146, remove attempts at :147
- `src/dispatcher.rs:149-151` -- non-None branch: routes.insert target; classification_attempts.remove
- `src/dispatcher.rs:40` -- DEFAULT_MAX_CLASSIFICATION_ATTEMPTS = 8

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:133-154` (full cache+retry block), `:136-148` (None branch), `:146` (permanent None insert) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if target == DispatchTarget::None { ... } else { routes.insert(...) }`
- **inferred**: no independent test of the cache-not-inserted path

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates classification_attempts (does NOT mutate routes) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
