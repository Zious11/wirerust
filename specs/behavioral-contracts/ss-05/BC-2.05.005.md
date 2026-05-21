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

# BC-2.05.005: Classification Cached Per FlowKey After First Non-None Result

## Description

After a flow is classified as Http or Tls, the result is stored in `routes: HashMap<FlowKey,
DispatchTarget>`. Subsequent `on_data` calls for the same FlowKey use the cached target without
re-running the classify function. Once a flow is classified, it stays classified for its
lifetime. `DispatchTarget::None` is excluded from caching during the retry phase (before `max_classification_attempts` is reached), but IS inserted permanently into `routes` once the cap is hit. See BC-2.05.006 for the full two-phase None-caching contract.

This contract's test coverage (R4 finding): the INSERT path (classify + cache) is exercised
by tests, but the cache-HIT path (subsequent calls using cached result) is not independently
verified. A refactor that broke cache lookup would pass CI if the INSERT path tests still pass.

## Preconditions

1. A FlowKey has been classified as Http or Tls on a prior on_data call.
2. The classification result is stored in the routes HashMap.

## Postconditions

1. Subsequent on_data calls for the same FlowKey use the cached DispatchTarget.
2. The classify function is NOT re-run.
3. The data is forwarded directly to the cached analyzer.
4. Cache result is immutable: a cached Http flow cannot be reclassified as Tls, even if later
   data starts with TLS bytes.

## Invariants

1. Http and Tls are inserted into routes on first non-None classification. None is NOT inserted during the retry phase (before the cap); None IS inserted permanently once `classification_attempts[flow_key]` reaches `max_classification_attempts` (dispatcher.rs:146). See BC-2.05.006 for the full two-phase contract.
2. Cached routes are removed only on on_flow_close (BC-2.05.009).
3. Cache entries are per FlowKey (not per connection direction).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow classified as Http; later TLS data arrives on same flow | Stays Http (cache is immutable); TLS data sent to HttpAnalyzer |
| EC-002 | Flow classified on first on_data; 1000 subsequent on_data calls | Each uses cache; classify not called 1000 times |
| EC-003 | on_flow_close removes cache; same FlowKey reopens | New flow starts with no cache; re-classified from scratch |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| First call: TLS bytes -> classify -> Tls cached | routes[FlowKey] == Tls | happy-path |
| Second call: same FlowKey, different bytes | Forwarded to Tls without re-classify | happy-path |
| on_flow_close; then on_data again | Re-classify runs (cache was evicted) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | Cache prevents repeated classification for same flow | unit: call on_data N times, verify classify called once |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-first protocol dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-first protocol dispatch") per capabilities.md §CAP-05 -- caching is the efficiency mechanism for per-flow classification state |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- once classified, the decision is sticky) |
| Architecture Module | SS-05 (dispatcher.rs:133-154, C-21) |
| Stories | STORY-032 |
| Origin BC | BC-DSP-005 (pass-3 ingestion corpus, MEDIUM confidence -- cache miss path covered; cache hit path not independently tested; R4 finding) |

## Related BCs

- BC-2.05.006 -- related to (None is excluded from caching, creating asymmetry)
- BC-2.05.009 -- composes with (cache removal on flow close)

## Architecture Anchors

- `src/dispatcher.rs:133-154` -- classification cache block in on_data
- `src/dispatcher.rs:149-151` -- routes.insert when target is Http or Tls (not None); classification_attempts.remove

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:149-151` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: `if target != DispatchTarget::None { routes.insert(...) }`
- **inferred**: cache-hit path is exercised by tests but not independently asserted

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates routes HashMap |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
