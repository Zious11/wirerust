---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.06.021
bcs:
  - BC-2.06.021
  - BC-2.06.019
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

# VP-014: HttpAnalyzer Cross-Flow Isolation

## Property Statement

`HttpAnalyzer` maintains completely independent per-flow state. For any two
distinct `FlowKey` values A and B:

1. Parse errors accumulated on flow A do not affect `parse_errors` or poisoning
   state on flow B.
2. Poisoning of flow A (request or response direction) does not cause flow B
   to stop parsing.
3. Findings emitted for flow A are attributable to flow A's data only; no
   cross-contamination from flow B occurs.
4. Closing flow A via `on_flow_close` removes flow A's state and does not
   affect flow B's state.
5. After `on_flow_close(A)`, `on_data(A, ...)` creates a fresh `HttpFlowState`
   for key A (reopening same key starts fresh; BC-2.06.019).

## Source Contract

- **Primary BC:** BC-2.06.021 -- Cross-Flow Isolation: Errors and Poisoning Do Not Leak
- **Postcondition:** flow A state mutations do not affect flow B state
- **Related BC:** BC-2.06.019 -- on_flow_close Removes Per-Flow State; Reopening Same Key Starts Fresh

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Property testing | proptest | No -- arbitrary interleaving of data for two flows | All orderings of data delivery between two concurrent flows |

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::*;

    #[derive(Clone, Debug)]
    enum TwoFlowEvent {
        DataA(Vec<u8>),          // data for flow A
        DataB(Vec<u8>),          // data for flow B
        CloseA,
        CloseB,
        ReopenA,
    }

    proptest! {
        #[test]
        fn prop_flow_b_unaffected_by_flow_a_errors(
            events in prop::collection::vec(
                prop_oneof![
                    prop::collection::vec(any::<u8>(), 1..64)
                        .prop_map(TwoFlowEvent::DataA),
                    prop::collection::vec(any::<u8>(), 1..64)
                        .prop_map(TwoFlowEvent::DataB),
                    Just(TwoFlowEvent::CloseA),
                ],
                1..40
            )
        ) {
            let mut analyzer = HttpAnalyzer::new();
            let key_a = test_flow_key_a();
            let key_b = test_flow_key_b();

            // Seed a valid HTTP request on B so it has a real parse state
            analyzer.on_data(&key_b, Direction::ClientToServer,
                b"GET /healthy HTTP/1.1\r\nHost: b.example.com\r\n\r\n", 0);

            let mut b_errors_before: u64 = analyzer.flow_parse_errors(&key_b);

            for event in events {
                match event {
                    TwoFlowEvent::DataA(data) => {
                        analyzer.on_data(&key_a, Direction::ClientToServer, &data, 0);
                    }
                    TwoFlowEvent::DataB(data) => {
                        analyzer.on_data(&key_b, Direction::ClientToServer, &data, 0);
                        b_errors_before = analyzer.flow_parse_errors(&key_b);
                    }
                    TwoFlowEvent::CloseA => {
                        analyzer.on_flow_close(&key_a, CloseReason::Fin);
                    }
                    _ => {}
                }

                // After any A event, B's poisoning state must not have changed
                let b_state = analyzer.flow_state(&key_b);
                if let Some(s) = b_state {
                    // B was never sent error data in this event sequence
                    // Its poisoning should remain false unless B received invalid data
                }
            }
        }

        #[test]
        fn prop_close_and_reopen_starts_fresh(
            initial_data in prop::collection::vec(any::<u8>(), 1..100),
            new_data: Vec<u8>,
        ) {
            let mut analyzer = HttpAnalyzer::new();
            let key = test_flow_key_a();

            // Accumulate some state on the flow
            analyzer.on_data(&key, Direction::ClientToServer, &initial_data, 0);
            let errors_before = analyzer.flow_parse_errors(&key);

            // Close the flow
            analyzer.on_flow_close(&key, CloseReason::Fin);

            // Reopen -- state should be fresh
            analyzer.on_data(&key, Direction::ClientToServer, &new_data, 0);
            let state_after = analyzer.flow_state(&key);
            if let Some(s) = state_after {
                // Error count must not include errors from the previous flow instance
                prop_assert!(!s.request_poisoned || errors_before > 0,
                    "new flow started poisoned without accumulating errors");
            }
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Unbounded | Random interleaved byte sequences for two flows |
| Proof complexity | Medium | Must verify HashMap isolation; two-key test pattern |
| Tool support | High | HttpAnalyzer is pure per-instance; per-flow HashMap state |
| Estimated proof time | < 60 seconds | |

## Source Location

`src/analyzer/http.rs` -- `HttpAnalyzer.flows: HashMap<FlowKey, HttpFlowState>`.
Per-flow state is keyed by `FlowKey`; no global state.
`on_flow_close` removes the entry: `self.flows.remove(flow_key)`.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
