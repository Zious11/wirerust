---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.06.015
bcs:
  - BC-2.06.015
  - BC-2.06.016
  - BC-2.06.017
module: src/analyzer/http.rs
proof_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-006: HTTP Poison Monotonicity

## Property Statement

Within a single flow's lifetime, the `request_poisoned` and `response_poisoned`
boolean fields on `HttpFlowState` are monotonically false-to-true:

1. Both fields start as `false` when a flow is created.
2. Each field may transition from `false` to `true` exactly once: when the
   respective direction's consecutive error count reaches `POISON_THRESHOLD` (3).
3. Neither field ever transitions from `true` back to `false` within the flow's
   lifetime. There are zero assignments of `false` to either field in http.rs.
4. Poisoning is per-direction: `request_poisoned` transitioning to `true` has no
   effect on `response_poisoned` and vice versa.

The `error_count` fields ARE non-monotonic (they reset to 0 on a successful parse),
so the threshold measures consecutive errors, not cumulative.

## Source Contract

- **Primary BC:** BC-2.06.015 -- After 3 consecutive parse errors a direction is poisoned; subsequent bytes skipped
- **Postcondition:** `*_poisoned` is true and never resets to false within flow lifetime
- **Invariant:** INV-8 (HTTP Poisoning is Monotonic False-to-True, inv-01-core-invariants.md)
- **Related BC:** BC-2.06.016 -- Single parse error does NOT poison; next valid request parses normally
- **Related BC:** BC-2.06.017 -- Poisoning is per-direction: poisoned request does not affect response

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Property testing | proptest | No -- arbitrary sequences of valid/invalid HTTP data chunks | All orderings of error and success chunks within a flow |

Rationale for proptest over Kani: the HTTP parser (httparse) has complex internal
state that makes Kani's bounded model checking impractical. proptest generates random
sequences of byte chunks and verifies the monotonicity invariant across all orderings.

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::*;

    // Strategy: sequence of "parse events" (success or error) applied to one flow
    #[derive(Clone, Debug)]
    enum ParseEvent {
        ValidRequest,     // bytes that parse as a complete HTTP request
        InvalidBytes,     // bytes that cause a parse error
        ValidResponse,    // bytes that parse as a complete HTTP response
    }

    proptest! {
        #[test]
        fn prop_poison_monotonic_false_to_true(
            events in prop::collection::vec(
                prop_oneof![
                    Just(ParseEvent::ValidRequest),
                    Just(ParseEvent::InvalidBytes),
                    Just(ParseEvent::ValidResponse),
                ],
                1..50
            )
        ) {
            let mut analyzer = HttpAnalyzer::new();
            let key = test_flow_key();
            let mut req_ever_poisoned = false;
            let mut resp_ever_poisoned = false;

            for event in &events {
                let data = match event {
                    ParseEvent::ValidRequest => b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".as_ref(),
                    ParseEvent::InvalidBytes => b"\xFF\xFE garbage".as_ref(),
                    ParseEvent::ValidResponse => b"HTTP/1.1 200 OK\r\n\r\n".as_ref(),
                };
                let dir = match event {
                    ParseEvent::ValidResponse => Direction::ServerToClient,
                    _ => Direction::ClientToServer,
                };
                analyzer.on_data(&key, dir, data, 0);

                // Once poisoned, must stay poisoned
                let state = analyzer.flow_state(&key);
                if let Some(s) = state {
                    if req_ever_poisoned {
                        prop_assert!(s.request_poisoned,
                            "request_poisoned reverted to false");
                    }
                    if resp_ever_poisoned {
                        prop_assert!(s.response_poisoned,
                            "response_poisoned reverted to false");
                    }
                    if s.request_poisoned { req_ever_poisoned = true; }
                    if s.response_poisoned { resp_ever_poisoned = true; }
                }
            }
        }

        #[test]
        fn prop_poison_per_direction_isolated(
            req_errors in 1usize..=10,
            resp_errors in 0usize..=2  // below threshold
        ) {
            let mut analyzer = HttpAnalyzer::new();
            let key = test_flow_key();

            // Drive request direction to poisoning
            for _ in 0..(req_errors.max(3)) {
                analyzer.on_data(&key, Direction::ClientToServer, b"\xFF garbage", 0);
            }

            // Response direction should not be poisoned
            let state = analyzer.flow_state(&key);
            if let Some(s) = state {
                prop_assert!(!s.response_poisoned,
                    "response direction poisoned by request-side errors");
            }
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Unbounded -- proptest generates random sequences | Shrinking on failure pinpoints minimal poison sequence |
| Proof complexity | Low | The invariant is a simple boolean monotonicity check |
| Tool support | High | HttpAnalyzer is pure; per-instance state; no global side effects |
| Estimated proof time | < 30 seconds for 1000 cases | proptest default case count is sufficient |

## Source Location

`src/analyzer/http.rs:341-345` -- poison transition logic.
Confirmed zero `= false` assignments to `*_poisoned` fields (pass-2 R3 Target 3 audit).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
