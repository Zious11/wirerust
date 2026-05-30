---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - v1.3: W13 Pass 1 remediation: split broad dispatcher.rs:136-153 anchor into three precise line-range anchors (F-W13P1-007) — 2026-05-27
  - v1.4: W13 Pass 2 remediation: correct :149-152 to :149-151 (exclusive-of-closing-brace convention, F-W13P2-005) — 2026-05-27
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.004: Unknown Content + Unknown Port Returns DispatchTarget::None

## Description

When all three classification checks fail -- TLS content signature, HTTP method prefix, and
port fallback -- the `classify` function returns `DispatchTarget::None`. The data is not
forwarded to any analyzer. `DispatchTarget::None` is NOT cached (BC-2.05.006); classify will
be retried on the next `on_data` call, up to `max_classification_attempts` times (default 8).
If the flow reaches the retry cap, `DispatchTarget::None` is then cached permanently for
the flow and the classification_attempts counter entry is removed.

## Preconditions

1. TLS content signature check failed.
2. HTTP method prefix check failed.
3. Neither port is in {80, 443, 8080, 8443}.

## Postconditions

1. Returns `DispatchTarget::None`.
2. `classification_attempts[flow_key]` is incremented by 1.
3. Data is NOT forwarded to any analyzer.
4. If `classification_attempts[flow_key] >= max_classification_attempts`:
   - `DispatchTarget::None` is inserted into `routes` (permanently).
   - `classification_attempts` entry is removed.
   - Future `on_data` calls short-circuit via the route cache.

## Invariants

1. `DispatchTarget::None` is never cached before the retry cap is hit (pre-cap calls
   re-run classify on every `on_data`).
2. After the cap is hit, `DispatchTarget::None` IS cached and classify never runs again
   for that flow.
3. Data bytes for unclassified flows are silently discarded (BC-INV-2 implicit).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Unknown content on port 9999, 1 call | Returns None; attempt_count=1; not cached |
| EC-002 | 8 consecutive None calls (default cap) | On 8th: None cached; attempt counter removed |
| EC-003 | 9th call after cap hit | Cached None used; classify not called |
| EC-004 | max_classification_attempts=0 | Every flow immediately routes to cached None |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SSH bytes on port 22 (unknown to dispatcher) | DispatchTarget::None; no analyzer call | happy-path |
| 8 consecutive SSH-bytes on port 22 | After 8: None cached | edge-case |
| on_data with max_attempts=1, unknown content | Cached None after first attempt | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | Unknown content + unknown port returns None | unit: test_unclassified_flows_counter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md -- DispatchTarget::None is the defined fallthrough for flows that match no classification rule |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- None is the explicit non-match result) |
| Architecture Module | SS-05 (dispatcher.rs:116, 136, 137-148, 149-151, C-21) |
| Stories | STORY-032 |
| Origin BC | BC-DSP-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.003 -- related to (port fallback is the last check before None)
- BC-2.05.006 -- composes with (None is not cached pre-cap; complements this BC)
- BC-2.05.007 -- composes with (unclassified_flows counter increments on close for None flows)

## Architecture Anchors

- `src/dispatcher.rs:116` -- `DispatchTarget::None` return in classify
- `src/dispatcher.rs:136` -- classify() call dispatch
- `src/dispatcher.rs:137-148` -- None handling: attempt counter increment + cap-triggered caching
- `src/dispatcher.rs:149-151` -- non-None branch: cache target + remove attempts entry
- `tests/dispatcher_tests.rs` -- test_unclassified_flows_counter

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:116, 136, 137-148, 149-151` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if target == DispatchTarget::None { count += 1; if count >= max_classification_attempts { routes.insert(None); } }`
- **assertion**: test_unclassified_flows_counter

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates classification_attempts, optionally routes |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
