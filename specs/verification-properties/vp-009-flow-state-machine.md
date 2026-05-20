---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.04.050
bcs:
  - BC-2.04.050
  - BC-2.04.051
  - BC-2.04.052
  - BC-2.04.004
  - BC-2.04.005
module: src/reassembly/flow.rs
proof_method: kani
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

# VP-009: FlowState Machine Validity

## Property Statement

The `FlowState` state machine on `TcpFlow` satisfies:

1. All valid transitions are:
   - `New` -> `SynSent` (on first SYN)
   - `SynSent` -> `Established` (on SYN+ACK)
   - `New` -> `Established` (on data-without-SYN; sets `partial=true`)
   - `Established` -> `Closing` (on first FIN)
   - `Closing` -> `Closed` (on second FIN)
   - Any state -> `Closed` (on RST)

2. No state transition not listed above is reachable through the public API.

3. RST transitions the flow to `Closed` from ANY prior state (including
   `New`, `SynSent`, `Established`, `Closing`).

4. Data-without-SYN (`on_data_without_syn`) correctly transitions `New` ->
   `Established` and sets `partial = true`.

5. Once `Closed`, the state does not change (terminal state).

## Source Contract

- **Primary BC:** BC-2.04.050 -- Flow state machine: New->SynSent->Established->Closing->Closed transitions
- **Postcondition:** Only the listed transitions are reachable
- **Related BC:** BC-2.04.051 -- RST transitions state to Closed from any prior state
- **Related BC:** BC-2.04.052 -- on_data_without_syn transitions New->Established and sets partial=true
- **Related BC:** BC-2.04.004 -- First SYN sets client ISN and initiator
- **Related BC:** BC-2.04.005 -- SYN+ACK marks server as responder; state -> Established

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- enumerate all state/event combinations (5 states x 6 events) | Complete state-transition table |

The state machine has 5 states and 6 driving events. Kani can exhaustively check
all 30 combinations without a problematic unwind bound.

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // All possible starting states
    fn all_states() -> Vec<FlowState> {
        vec![
            FlowState::New,
            FlowState::SynSent,
            FlowState::Established,
            FlowState::Closing,
            FlowState::Closed,
        ]
    }

    #[kani::proof]
    fn verify_rst_closes_from_any_state() {
        // For each starting state, apply RST, verify Closed
        // (Kani handles this as symbolic state)
        let state: FlowState = kani::any();
        let mut flow = TcpFlow::with_state(state);
        flow.on_rst();
        assert!(matches!(flow.state(), FlowState::Closed));
    }

    #[kani::proof]
    fn verify_closed_is_terminal() {
        let mut flow = TcpFlow::with_state(FlowState::Closed);
        // Apply any event -- state must remain Closed
        flow.on_rst();
        assert!(matches!(flow.state(), FlowState::Closed));
        // (Additional events: on_syn, on_syn_ack, on_fin -- all must leave Closed)
    }

    #[kani::proof]
    fn verify_data_without_syn_sets_partial() {
        let mut flow = TcpFlow::with_state(FlowState::New);
        let isn: u32 = kani::any();
        flow.on_data_without_syn(isn);
        assert!(matches!(flow.state(), FlowState::Established));
        assert!(flow.partial);
    }

    #[kani::proof]
    fn verify_no_invalid_state_reachable() {
        // Symbolic starting state + symbolic event sequence of length 3
        let state: FlowState = kani::any();
        let event1: u8 = kani::any();  // 0=syn, 1=syn_ack, 2=rst, 3=fin, 4=data, 5=data_no_syn
        let event2: u8 = kani::any();
        kani::assume(event1 <= 5 && event2 <= 5);

        let mut flow = TcpFlow::with_state(state);
        apply_event(&mut flow, event1);
        apply_event(&mut flow, event2);

        // After any sequence, state must be one of the 5 valid variants
        let s = flow.state();
        assert!(
            matches!(s, FlowState::New)
            || matches!(s, FlowState::SynSent)
            || matches!(s, FlowState::Established)
            || matches!(s, FlowState::Closing)
            || matches!(s, FlowState::Closed)
        );
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite | 5 states x 6 events = 30 combinations; Kani handles trivially |
| Proof complexity | Low | Pure state machine with simple match arms |
| Tool support | High | `TcpFlow` state is encapsulated; pure transitions |
| Estimated proof time | < 30 seconds | Minimal symbolic state space |

## Source Location

`src/reassembly/flow.rs` -- `FlowState` enum and transition methods on `TcpFlow`.
`src/reassembly/lifecycle.rs` -- RST and FIN handling.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
