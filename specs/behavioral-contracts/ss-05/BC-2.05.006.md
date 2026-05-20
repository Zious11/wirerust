---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.006: DispatchTarget::None NOT Cached; Reclassification Retried

## Description

When `classify` returns `DispatchTarget::None`, the result is NOT inserted into the `routes`
HashMap (unlike Http or Tls results). Instead, `classification_attempts` is incremented and
`classify` will be re-run on the next `on_data` call for the same FlowKey. This allows late-
arriving data (e.g., a long-running connection whose first few chunks were small) to eventually
be classified when the buffer grows large enough to match a content signature. The only
exception is after the retry cap is reached (BC-2.05.004), at which point None IS cached.

## Preconditions

1. `classify` has returned `DispatchTarget::None` for this FlowKey.
2. `classification_attempts[flow_key] < max_classification_attempts`.

## Postconditions

1. `routes` HashMap does NOT contain an entry for this FlowKey.
2. `classification_attempts[flow_key]` has been incremented by 1.
3. On the next `on_data`, `routes.get(flow_key)` returns None -> classify is re-run.

## Invariants

1. The insert-only-when-not-None invariant: `if target != DispatchTarget::None { routes.insert(...) }` (dispatcher.rs:149-151).
2. Reclassification is bounded by `max_classification_attempts` (default 8).
3. None is never inserted into `routes` BEFORE the cap is reached.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First 3 on_data calls return None; 4th returns TLS | On 4th: Tls cached; attempts counter cleared |
| EC-002 | max_classification_attempts=8; 8 Nones | On 8th: None cached permanently (BC-2.05.004) |
| EC-003 | Flow closed before cap reached | attempts counter cleared; no route cached |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SSH bytes (None) x3, then TLS bytes on 4th | routes[FlowKey]=Tls after 4th call | happy-path |
| Unknown content x2 then valid GET | routes[FlowKey]=Http after 3rd call | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | None result is not cached; reclassification retried | MEDIUM -- inferred from code; only insert path tested |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 -- None-not-cached design allows late content classification for mid-stream-join scenarios |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- None is a temporary state, not a permanent classification) |
| Architecture Module | SS-05 (dispatcher.rs:137-153, C-15) |
| Stories | S-TBD |
| Origin BC | BC-DSP-006 (pass-3 ingestion corpus, MEDIUM confidence -- inferred from code) |

## Related BCs

- BC-2.05.004 -- composes with (cap-triggered None caching is the eventual terminus)
- BC-2.05.005 -- related to (Http/Tls ARE cached; None is the asymmetric exception)

## Architecture Anchors

- `src/dispatcher.rs:137-153` -- None handling block with attempt counting
- `src/dispatcher.rs:149-151` -- `if target != None { routes.insert(...) }` guard

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:137-153` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if target != DispatchTarget::None { routes.insert(...) }`
- **inferred**: no independent test of the cache-not-inserted path

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates classification_attempts (does NOT mutate routes) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
