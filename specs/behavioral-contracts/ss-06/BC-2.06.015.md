---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: Wave 16 Pass-2 (F-W16-S044-P2-002) — align response poison anchor :467 → :467-468 (guard+assignment) in Architecture Anchors and Source Evidence; align with VP-006 [pre-F2 lines; now :488-489 post-F2] — 2026-05-28"
  - "v1.4 (2026-06-13): P19-B-08 ss-06 line-anchor re-sync — POISON_THRESHOLD :80→:82; req poison :408-409→:427-428; resp poison :467-468→:488-489. Verified against current src/analyzer/http.rs (1044 lines)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.015: After 3 Consecutive Parse Errors a Direction is Poisoned; Subsequent Bytes Skipped

## Description

When a flow direction (request or response) accumulates `POISON_THRESHOLD = 3` CONSECUTIVE
parse errors (errors from httparse that are not `Incomplete`), the direction is marked
"poisoned" (`request_poisoned = true` or `response_poisoned = true`). Once poisoned, all
subsequent bytes for that direction are silently absorbed without parsing, and the count
accumulated in `poisoned_bytes_skipped`. The poisoned state is monotonic: once true, it never
resets to false within the flow's lifetime (INV-8).

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. The request direction has accumulated exactly 3 consecutive parse errors:
   `request_error_count == POISON_THRESHOLD (3)`.
3. The THIRD consecutive parse error triggers the poisoning transition.

## Postconditions

1. `HttpFlowState.request_poisoned = true` (never resets to false for this flow).
2. `non_http_flows` is incremented by 1 (first direction to be poisoned; per BC-2.06.018,
   the second direction's poisoning does NOT increment again).
3. The buffer for the poisoned direction is cleared at poison time.
4. All subsequent `on_data` calls for this direction early-exit: bytes are counted in
   `poisoned_bytes_skipped` but not parsed.

## Invariants

1. Poisoning is per-direction: `request_poisoned` and `response_poisoned` are independent.
2. The error counter is CONSECUTIVE, not cumulative. One successful parse resets the counter
   to 0. POISON_THRESHOLD therefore measures consecutive failures (INV-8).
3. Poisoning is irreversible within a flow lifetime. The ONLY reset is via `on_flow_close`
   which drops the entire `HttpFlowState`.
4. `poisoned_bytes_skipped` is observable via `summarize()`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 2 errors, then 1 successful parse, then 2 more errors | Counter resets; not poisoned; need 3 more consecutive errors |
| EC-002 | 3 errors in a row | Poisoned on 3rd error; subsequent bytes skipped |
| EC-003 | Request poisoned; response gets 3 errors too | Response also poisoned; non_http_flows NOT incremented again |
| EC-004 | Poisoned flow receives 1000 bytes | All 1000 counted in poisoned_bytes_skipped; no parse attempted |
| EC-005 | on_flow_close on poisoned flow | HttpFlowState removed; same FlowKey can reopen fresh |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 consecutive non-HTTP bytes chunks | request_poisoned=true; non_http_flows=1 | happy-path |
| 2 bad + 1 good + 2 bad | request_poisoned=false; error_count=2 (not 3) | edge-case |
| 3 bad, then 100 more bytes | poisoned_bytes_skipped=100+; no parse on the 100 bytes | happy-path |
| Flow poisoned, flow closed, same flow reopened | New HttpFlowState with poison=false | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-006 | 3 consecutive errors trigger poisoning | unit: test_parse_error_poisons_direction_after_threshold |
| VP-006 | Poisoning is monotonic false->true (never resets within flow) | unit: test absence of = false assignments |
| VP-006 | Bytes after poisoning counted in poisoned_bytes_skipped | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP traffic analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP traffic analysis") per domain/capabilities/cap-06-http-analysis.md -- HTTP poisoning state machine is the resilience mechanism for non-HTTP traffic in HTTP-dispatched flows |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true) |
| Architecture Module | SS-06 (analyzer/http.rs:427-428, C-12) |
| Stories | STORY-044 |
| Origin BC | BC-HTTP-015 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.016 -- related to (single error does NOT poison; contrast case)
- BC-2.06.017 -- composes with (poisoning is per-direction)
- BC-2.06.018 -- composes with (non_http_flows counting semantics)
- BC-2.06.019 -- composes with (flow close is the only reset mechanism)

## Architecture Anchors

- `src/analyzer/http.rs:82` -- `const POISON_THRESHOLD: u8 = 3`
- `src/analyzer/http.rs:427-428` -- request direction poison transition: `if state.request_error_count >= POISON_THRESHOLD { state.request_poisoned = true; }`
- `src/analyzer/http.rs:488-489` -- response direction poison transition: `if state.response_error_count >= POISON_THRESHOLD { state.response_poisoned = true; }`
- `tests/http_analyzer_tests.rs` -- test_parse_error_poisons_direction_after_threshold

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:427-428` (request poison), `:488-489` (response poison), `:82` (threshold const) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: `if error_count >= POISON_THRESHOLD { poisoned = true }`
- **assertion**: test_parse_error_poisons_direction_after_threshold

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates HttpFlowState per flow |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
