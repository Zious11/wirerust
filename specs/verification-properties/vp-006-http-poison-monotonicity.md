---
document_type: verification-property
level: L4
version: "2.1"
status: verified
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
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "5ba85a7a1bdec2838e66b5026ae207222bce7a145e492b7f0e9e7d72694a5334"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.1: DF-SIBLING-SWEEP-001 — fix stale http.rs line anchors in proof harness comment: request_poisoned block :509-511 → :509-512, response_poisoned block :521-522 → :521-524; verified against HEAD cfe0112a — 2026-06-01"
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set."
  - "v2.1 (2026-06-13, PG-ARP-F2-007 anchor-drift sweep): Source Location and harness-comment line anchors corrected for F2 http.rs shifts. POISON_THRESHOLD: :80→:82. Request poison transition: :408-409→:427-429. Response poison transition: :467-468→:489-490. Harness comment skip-block anchors: :509-512/:521-524→:542-544/:554-556. Lock fields unchanged."
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

// API notes (src/analyzer/http.rs):
//   - HttpFlowState (including request_poisoned / response_poisoned fields) is
//     a PRIVATE struct. There is no flow_state() public method on HttpAnalyzer.
//   - The only public observable for poisoning is poisoned_bytes_skipped() -> u64,
//     which increments for every byte fed to a direction after it is poisoned
//     (src/analyzer/http.rs:542-544, 554-556).
//   - Public methods: new(), transaction_count(), parse_error_count(),
//     poisoned_bytes_skipped(), method_counts(), host_counts(), uri_list(),
//     status_code_counts(), user_agent_counts().
//   - The harness must expose poison monotonicity via the public observable
//     (poisoned_bytes_skipped monotonically non-decreasing once poison sets in)
//     OR via #[cfg(test)] with a test-only accessor added to HttpAnalyzer.
//   - The formal-verifier MUST add a test-only accessor:
//       #[cfg(test)]
//       pub fn flow_state_for_test(&self, key: &FlowKey)
//           -> Option<(bool, bool)>  // (request_poisoned, response_poisoned)
//     before locking this VP, or restructure the harness to use only
//     poisoned_bytes_skipped() as the observable.

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

    // Helper: construct a deterministic flow key for tests.
    fn test_flow_key() -> FlowKey {
        use std::net::{IpAddr, Ipv4Addr};
        let c = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let s = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        FlowKey::new(c, 54321, s, 80)
    }

    proptest! {
        // Monotonicity observable via poisoned_bytes_skipped:
        // Once a direction is poisoned, every byte sent to it adds to
        // poisoned_bytes_skipped. The counter must never decrease.
        #[test]
        fn prop_poison_bytes_skipped_monotonic(
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
            let mut prev_skipped: u64 = 0;

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

                let now_skipped = analyzer.poisoned_bytes_skipped();
                prop_assert!(
                    now_skipped >= prev_skipped,
                    "poisoned_bytes_skipped decreased: was {} now {}",
                    prev_skipped, now_skipped
                );
                prev_skipped = now_skipped;
            }
        }

        // Per-direction isolation: request-side errors >= POISON_THRESHOLD (3)
        // must not cause response-direction bytes to be skipped when response
        // direction receives a single valid response before any response errors.
        #[test]
        fn prop_poison_per_direction_isolated(
            req_errors in 3usize..=10
        ) {
            let mut analyzer = HttpAnalyzer::new();
            let key = test_flow_key();

            // Drive request direction past POISON_THRESHOLD
            for _ in 0..req_errors {
                analyzer.on_data(&key, Direction::ClientToServer, b"\xFF\xFE garbage", 0);
            }
            let skipped_after_req_poison = analyzer.poisoned_bytes_skipped();

            // Feed a valid response -- response direction is NOT poisoned;
            // its bytes must NOT be counted as skipped.
            let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
            analyzer.on_data(&key, Direction::ServerToClient, resp, 0);
            prop_assert_eq!(
                analyzer.poisoned_bytes_skipped(),
                skipped_after_req_poison,
                "response-direction bytes were skipped due to request-side poison"
            );
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

`src/analyzer/http.rs:427-429` -- request direction poison transition:
  `if state.request_error_count >= POISON_THRESHOLD { state.request_poisoned = true; }`

`src/analyzer/http.rs:489-490` -- response direction poison transition:
  `if state.response_error_count >= POISON_THRESHOLD { state.response_poisoned = true; }`

`src/analyzer/http.rs:82` -- `const POISON_THRESHOLD: u8 = 3;`

Confirmed zero `= false` assignments to `*_poisoned` fields (pass-2 R3 Target 3 audit).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
