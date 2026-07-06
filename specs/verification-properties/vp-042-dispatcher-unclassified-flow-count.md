---
artifact: verification-property
vp_id: VP-042
title: "Dispatcher (TransportProto, u16) Key Unclassified-Flow Count Accumulation"
status: draft
phase: P1
tool: proptest
subsystem: SS-05
module: "dispatcher.rs"
producer: architect
timestamp: 2026-07-01T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
feature_cycle: feature-protocol-coverage
source_bc: BC-2.05.010
bcs:
  - BC-2.05.010
  - BC-2.05.011
verification_lock: false
---

# VP-042: Dispatcher (TransportProto, u16) Key Unclassified-Flow Count Accumulation

## Property Statement

`StreamDispatcher::on_flow_close` correctly accumulates per-`(TransportProto, u16)` key
counts in `unclassified_port_counts` for `DispatchTarget::None` flows. The following
properties hold for all N ∈ [1, 256] and all `(TransportProto, u16)` key sequences:

1. **Total count:** After N `on_flow_close` calls for `DispatchTarget::None` flows,
   `unclassified_port_counts.values().sum() == N`.
2. **Per-key accuracy:** For each unique `(TransportProto, u16)` key K,
   `unclassified_port_counts[K]` equals the number of times K was presented in the
   input sequence.
3. **None-target gate:** Classified flows (`DispatchTarget != None`) do NOT increment
   any entry in `unclassified_port_counts`.

**Key construction for TCP flows:** TCP flows use
`(TransportProto::Tcp, lower_port().min(upper_port()))` — the FlowKey lower/upper
accessors provide direction-normalized port values symmetric with the dispatch path.

**Harness precondition (all 3 harnesses):** ≥1 analyzer `is_some()` AND
`coverage_gaps_enabled = true`. The `unclassified_port_counts` increment is placed INSIDE
the same analyzer-present guard as `unclassified_flows += 1`; both counters increment
together as an inherent dual-gate consequence (VP042D-FROZEN-RESIDUAL-001 / F-F2P9-003 /
ADR-012 Decision 6 Clarification).

**Scope note:** VP-042 covers the `dispatcher.rs on_flow_close` path only. The UDP
decode-loop unclassified-packet counter in `main.rs` is a separate code path unreachable
from `dispatcher.rs` and is covered by VP-043 (F-F2P1-011).

**NOTE:** VP-004 (Kani, dispatcher `classify()`) must be re-validated at F6 to confirm
the new `HashMap<(TransportProto, u16), u64>` field introduced no regression in the
`classify()` proof — the oracle model is unchanged; re-validation is regression-confirmation
only.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.05.010 | `unclassified_port_counts` accumulates correctly; None-target gate | Sub-1 (total) + Sub-3 (gate) |
| BC-2.05.011 | Per-`(TransportProto, u16)` key frequency accuracy | Sub-2 (per-key) |

## Purity Classification

**Pure state-machine with controlled injection.** The proptest strategy drives
`StreamDispatcher` directly with synthetic `FlowKey` inputs and `DispatchTarget::None`
close events. No I/O; no pcap files; no global state beyond the dispatcher under test.

## Proof Harnesses

```rust
proptest! {
    /// VP-042 Sub-1: total count equals N.
    /// After N on_flow_close None-target calls, unclassified_port_counts.values().sum() == N.
    #[test]
    fn proptest_vp042_total_count_equals_n(
        n in 1usize..=256,
        keys in prop::collection::vec(
            (any::<u8>().prop_map(|b| if b < 128 { TransportProto::Tcp } else { TransportProto::Udp }),
             any::<u16>()),
            1..=256,
        ),
    ) { ... }

    /// VP-042 Sub-2: per-(TransportProto, u16) key count equals frequency.
    #[test]
    fn proptest_vp042_per_port_count_equals_frequency(
        key_sequence in prop::collection::vec(
            (any::<u8>().prop_map(|b| if b < 128 { TransportProto::Tcp } else { TransportProto::Udp }),
             any::<u16>()),
            1..=256,
        ),
    ) { ... }

    /// VP-042 Sub-3: classified flows do NOT increment unclassified_port_counts.
    #[test]
    fn proptest_vp042_no_count_spurious_on_classified_flows(
        n in 1usize..=64,
    ) { ... }
}
```

## Feasibility Assessment

**Assessment: FEASIBLE.**

`on_flow_close` is a map-accumulation operation (increment counter for key). proptest
is the natural tool for accumulation-over-sequences properties. The three harnesses
cover total count, per-key accuracy, and the None-target gate — together they fully
specify the accumulation behavior. Default proptest case count is sufficient.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-042 added (dispatcher port-count accumulation, D-322 key-type clarification, F-F2P1-006 key fix) | draft |
| F4 (TDD implementation) | 3 proptest harnesses authored in tests/ for dispatcher | draft → active |
| F6 (formal hardening) | proptest suite confirmed in CI; VP-004 re-validation completed | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.
