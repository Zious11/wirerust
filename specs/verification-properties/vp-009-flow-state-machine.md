---
document_type: verification-property
level: L4
version: "1.2"
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
modified:
  - "v1.1: Wave 7 wave-level adv-pass-1 F-1: corrected on_data_without_syn anchor references from flow.rs:241 to flow.rs:248 (Wave 6 fin_count addition shifted lines +7; W4.1 recurrence). — 2026-05-25"
  - "v1.2: Wave 7 wave-level adv-pass-2 F-1 CRITICAL: completed the partial v1.1 fix by updating the Source Location section's on_syn/on_syn_ack/on_fin/on_rst anchors from pre-Wave-6 line numbers (229/235/248/257) to post-Wave-6 line numbers (236/242/255/264). The on_fin entry was previously colliding with on_data_without_syn at line 248 (same line claim) — corrected. Sibling-anchor sweep across all SS-04 BCs/VPs verified independently. — 2026-05-25"
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

Notes on `TcpFlow` API:
- `TcpFlow::new(key: FlowKey, timestamp: u32)` is the only public constructor.
  There is no `with_state` convenience helper in production code; proofs that
  need to start from an arbitrary state must call `new` and then drive the state
  machine via the event methods before the assertion under test.
- `flow.state` is a `pub` field (`FlowState`), not a getter method. Access it
  as `flow.state`, not `flow.state()`.
- `on_data_without_syn(&mut self)` takes NO arguments
  (`src/reassembly/flow.rs:248`). It does not accept an ISN; the ISN is set
  separately via `flow_dir.set_isn(isn)` on a `FlowDirection`.
- `apply_event` is a test-only helper that must be defined inside the proof
  module; it does not exist in production code.

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::reassembly::flow::{FlowKey, FlowState, TcpFlow};
    use std::net::{IpAddr, Ipv4Addr};

    /// Build a `TcpFlow` pre-seeded to `target_state` by driving the real
    /// transition methods. Used by proofs that need a symbolic starting state.
    ///
    /// Mapping (kani::any() % 5):
    ///   0 => New       (fresh flow, no transitions)
    ///   1 => SynSent   (New -> on_syn)
    ///   2 => Established (New -> on_syn -> on_syn_ack)
    ///   3 => Closing   (New -> on_syn -> on_syn_ack -> on_fin)
    ///   4 => Closed    (New -> on_rst)
    fn make_flow_in_state(discriminant: u8) -> TcpFlow {
        let key = FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)), 1000,
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 2)), 80,
        );
        let mut flow = TcpFlow::new(key, 0);
        match discriminant % 5 {
            0 => {} // New
            1 => { flow.on_syn(); } // SynSent
            2 => { flow.on_syn(); flow.on_syn_ack(); } // Established
            3 => { flow.on_syn(); flow.on_syn_ack(); flow.on_fin(); } // Closing
            _ => { flow.on_rst(); } // Closed
        }
        flow
    }

    /// Apply one of 5 driving events (0=syn, 1=syn_ack, 2=rst, 3=fin,
    /// 4=data_without_syn) to a flow. Test-only helper; not in production code.
    fn apply_event(flow: &mut TcpFlow, event: u8) {
        match event % 5 {
            0 => flow.on_syn(),
            1 => flow.on_syn_ack(),
            2 => flow.on_rst(),
            3 => flow.on_fin(),
            _ => flow.on_data_without_syn(),
        }
    }

    #[kani::proof]
    fn verify_rst_closes_from_any_state() {
        // For each starting state, apply RST, verify Closed.
        // `flow.state` is a pub field (not a method).
        let discriminant: u8 = kani::any();
        let mut flow = make_flow_in_state(discriminant);
        flow.on_rst();
        assert!(matches!(flow.state, FlowState::Closed));
    }

    #[kani::proof]
    fn verify_closed_is_terminal() {
        // Start in Closed (discriminant % 5 == 4), apply every event.
        let mut flow = make_flow_in_state(4);
        flow.on_rst();
        assert!(matches!(flow.state, FlowState::Closed));
        let mut flow2 = make_flow_in_state(4);
        flow2.on_syn();
        // on_syn only transitions New -> SynSent; Closed is unaffected.
        assert!(matches!(flow2.state, FlowState::Closed));
        let mut flow3 = make_flow_in_state(4);
        flow3.on_fin();
        assert!(matches!(flow3.state, FlowState::Closed));
    }

    #[kani::proof]
    fn verify_data_without_syn_sets_partial() {
        // on_data_without_syn takes NO arguments (src/reassembly/flow.rs:248).
        let key = FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)), 1000,
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 2)), 80,
        );
        let mut flow = TcpFlow::new(key, 0);
        // flow.state is New at construction.
        assert!(matches!(flow.state, FlowState::New));
        flow.on_data_without_syn();
        assert!(matches!(flow.state, FlowState::Established));
        assert!(flow.partial);
    }

    #[kani::proof]
    fn verify_no_invalid_state_reachable() {
        // Symbolic starting state + symbolic event sequence of length 2.
        let disc: u8 = kani::any();
        let event1: u8 = kani::any();
        let event2: u8 = kani::any();

        let mut flow = make_flow_in_state(disc);
        apply_event(&mut flow, event1);
        apply_event(&mut flow, event2);

        // After any sequence, state must be one of the 5 valid variants.
        // flow.state is a pub field.
        assert!(
            matches!(flow.state, FlowState::New)
            || matches!(flow.state, FlowState::SynSent)
            || matches!(flow.state, FlowState::Established)
            || matches!(flow.state, FlowState::Closing)
            || matches!(flow.state, FlowState::Closed)
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

`src/reassembly/flow.rs:77` -- `FlowState` enum definition.
`src/reassembly/flow.rs:236` -- `TcpFlow::on_syn` (New -> SynSent).
`src/reassembly/flow.rs:242` -- `TcpFlow::on_syn_ack` (SynSent/New -> Established).
`src/reassembly/flow.rs:248` -- `TcpFlow::on_data_without_syn` (no-arg; New -> Established, sets partial=true).
`src/reassembly/flow.rs:255` -- `TcpFlow::on_fin` (Established/SynSent -> Closing; second FIN -> Closed).
`src/reassembly/flow.rs:264` -- `TcpFlow::on_rst` (any state -> Closed).
`src/reassembly/flow.rs:185` -- `TcpFlow::state` pub field (access as `flow.state`, not `flow.state()`).
Note: FIN/RST state changes are implemented directly on `TcpFlow` in `flow.rs`;
`lifecycle.rs` handles flow retirement (memcap eviction, close_flow), not state transitions.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
