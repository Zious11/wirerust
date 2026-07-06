---
artifact: verification-property
vp_id: VP-043
title: "UDP Decode-Loop Unclassified-Packet Count Accumulation"
status: draft
phase: P1
tool: proptest
subsystem: SS-05
module: "main.rs"
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

# VP-043: UDP Decode-Loop Unclassified-Packet Count Accumulation

## Property Statement

In the `main.rs` decode loop, UDP packets processed as `DecodedFrame::Ip(parsed)` that
are NOT classified by `dns_analyzer.can_decode()` increment the unclassified UDP gap
counter keyed on `(TransportProto::Udp, min(src_port, dst_port))`. The following
properties hold for all N ∈ [1, 256] and all UDP port sequences:

1. **Accumulation totality:** After N unclassifiable UDP packets, the counter total == N.
2. **Gate invariant:** UDP packets classified by `dns_analyzer.can_decode()` do NOT
   increment the unclassified counter.
3. **Key symmetry:** The key uses `min(src_port, dst_port)` — symmetric with the TCP
   path in VP-042 — to eliminate ephemeral-port noise and ensure bidirectional flows
   map to the same counter bucket.

**Rationale for separate VP from VP-042:** VP-042 is anchored to `dispatcher.rs
on_flow_close`. The UDP packet path in `main.rs` decode loop is not routed through
`on_flow_close` and is therefore unreachable by VP-042 (F-F2P1-011). VP-043 + VP-042
together cover OQ-5 UDP exactness/monotonicity jointly (dispatcher TCP path + decode-loop
UDP path).

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.05.010 | UDP unclassified counter accumulates at N-per-N rate; gate: classified UDP excluded | Sub-1 (total) + Sub-2 (gate) |
| BC-2.05.011 | Per-`(TransportProto::Udp, min(src_port, dst_port))` key frequency accuracy | Sub-1 (via key structure) |

## Purity Classification

**Integration test over a pure decode-loop abstraction.** The proptest strategy drives
the decode-loop UDP classification path with synthetic `ParsedPacket` inputs representing
UDP packets. The DNS classifier oracle is a test double. No I/O; no pcap files.

## Proof Harnesses

```rust
proptest! {
    /// VP-043 Sub-1: after N unclassifiable UDP packets, counter total == N.
    #[test]
    fn proptest_vp043_total_count_equals_n(
        n in 1usize..=256,
        ports in prop::collection::vec((any::<u16>(), any::<u16>()), 1..=256),
    ) { ... }

    /// VP-043 Sub-2: UDP packets classified by dns_analyzer do NOT increment the counter.
    #[test]
    fn proptest_vp043_no_increment_on_classified_udp(
        n in 1usize..=64,
    ) { ... }
}
```

## Feasibility Assessment

**Assessment: FEASIBLE.**

The UDP decode-loop counter is a simple per-packet accumulation. The proptest strategy
drives the accumulation path with synthetic UDP packets; the DNS classifier oracle is
a deterministic test double. Two harnesses (total count + gate invariant) fully specify
the behavior. Default proptest case count (100) is sufficient.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-043 added to cover the UDP decode-loop path (F-F2P1-011; fills OQ-5 UDP exactness gap left by VP-042's dispatcher-only scope) | draft |
| F4 (TDD implementation) | 2 proptest harnesses authored for main.rs UDP path | draft → active |
| F6 (formal hardening) | proptest suite confirmed in CI | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation.
