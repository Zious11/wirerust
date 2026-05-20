---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/http.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-06
capability: CAP-06
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

# BC-2.06.021: Cross-Flow Isolation: Errors and Poisoning Do Not Leak

## Description

`HttpFlowState` is per-FlowKey; all per-direction state (buffers, error counts, poison flags,
`counted_as_non_http`) is scoped to a single `HashMap<FlowKey, HttpFlowState>` entry. Parse
errors or poisoning in flow A cannot affect flow B. Concurrent observation of two different
flows proceeds independently. Aggregate counters (`parse_errors`, `non_http_flows`, etc.) are
global, but per-flow decision gates (poisoning, error counts) are fully isolated.

## Preconditions

1. At least two distinct FlowKeys are active simultaneously in `self.flows`.
2. Flow A has experienced parse errors or has been poisoned.

## Postconditions

1. Flow B's `HttpFlowState` is unaffected by flow A's errors.
2. Flow B's `request_error_count` and `response_error_count` remain at their own values.
3. Flow B's poison flags remain at their own values.

## Invariants

1. `flows: HashMap<FlowKey, HttpFlowState>` -- each entry is completely independent.
2. Only `on_flow_close` removes entries; entries do not affect each other.
3. Aggregate stats (`parse_errors`, `non_http_flows`) are global sums and do NOT gate
   per-flow behavior.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow A poisoned; Flow B receives first on_data | Flow B parses normally |
| EC-002 | Flow A has 2 errors; Flow B has 0 errors | Flow B's error_count remains 0 |
| EC-003 | 100 flows all poisoned simultaneously | Each flow's poison state independent; aggregate non_http_flows=100 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Flow A: 3 errors; Flow B: valid GET | Flow B method counted; parse_errors=3 (from A), not affecting B's parse | happy-path |
| Flow A: poisoned; Flow B: same request | Flow B result identical to standalone execution | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Parse errors in flow A do not affect flow B | unit: test_cross_flow_isolation_parse_errors |
| VP-TBD | Poisoning in flow A does not affect flow B | unit: test_cross_flow_isolation_poisoning |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- cross-flow isolation is required for forensic correctness in multi-flow captures |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation), INV-8 (HTTP poisoning is monotonic -- per flow, not global) |
| Architecture Module | SS-06 (analyzer/http.rs:114-126, C-12) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-021 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.015 -- composes with (poisoning is per-flow, isolated)
- BC-2.06.019 -- composes with (flow removal on close is the clean isolation boundary)

## Architecture Anchors

- `src/analyzer/http.rs:114-126` -- flows HashMap field declaration
- `tests/http_analyzer_tests.rs` -- test_cross_flow_isolation_parse_errors, test_cross_flow_isolation_poisoning

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:114-126` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: HashMap<FlowKey, HttpFlowState> provides per-key isolation by construction
- **assertion**: test_cross_flow_isolation_parse_errors, test_cross_flow_isolation_poisoning

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | per-flow HashMap entries are isolated; global counters are sums only |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
