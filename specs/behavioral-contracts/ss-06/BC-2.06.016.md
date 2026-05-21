---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.016: Single Parse Error Does NOT Poison

## Description

A single parse error in the request or response direction increments `request_error_count`
to 1 and `parse_errors` to 1, but does NOT trigger poisoning (threshold requires 3 consecutive
errors). After the error, subsequent valid HTTP data can still be parsed normally.
`request_error_count` resets to 0 on the next successful parse.

## Preconditions

1. An HttpFlowState exists for the FlowKey with both poison flags false.
2. Exactly one parse error occurs (not preceded by 2+ prior consecutive errors).

## Postconditions

1. `request_error_count == 1` (not >= POISON_THRESHOLD=3).
2. `request_poisoned == false`.
3. `parse_errors` incremented by 1.
4. Buffer cleared.
5. A subsequent valid HTTP request can be parsed normally.

## Invariants

1. `POISON_THRESHOLD = 3`; a single error never reaches the threshold (INV-8).
2. Resetting the counter on success means the threshold measures CONSECUTIVE errors.
3. `request_poisoned` can only transition false->true when `request_error_count >= 3`
   on the same direction.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 1 error, then valid request | error_count=0 after success; poisoned=false |
| EC-002 | 2 errors, then valid request | error_count=0 after success; poisoned=false |
| EC-003 | 2 errors, then 1 error (not consecutive reset) | NOT poisoned; count=1 after third call |
| EC-004 | 3 errors (all consecutive) | Poisoned on third (BC-2.06.015) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| on_data(garbage) x1; then on_data(valid GET) | request_poisoned=false; method counted | happy-path |
| on_data(garbage) x2; then on_data(valid GET) | request_poisoned=false; error_count=0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-006 | Single error does not poison direction | unit: test_single_error_does_not_poison |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- non-poisoning of a single error is the tolerance policy for mid-stream joins |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true -- single error does not cross threshold) |
| Architecture Module | SS-06 (analyzer/http.rs:406-414, C-12) |
| Stories | STORY-044 |
| Origin BC | BC-HTTP-016 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.015 -- related to (3 consecutive errors is the poisoning case)
- BC-2.06.013 -- composes with (error counting mechanism)

## Architecture Anchors

- `src/analyzer/http.rs:406-414` -- error_count increment and threshold check
- `tests/http_analyzer_tests.rs` -- test_single_error_does_not_poison

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:406-414` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if state.request_error_count >= POISON_THRESHOLD { ... }`
- **assertion**: test_single_error_does_not_poison

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads/mutates HttpFlowState.request_error_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
