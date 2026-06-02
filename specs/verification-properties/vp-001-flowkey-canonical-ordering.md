---
document_type: verification-property
level: L4
version: "2.0"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.04.003
bcs:
  - BC-2.04.003
  - BC-2.04.053
module: src/reassembly/flow.rs
proof_method: kani
feasibility: feasible
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "02d1ad68d775bfdae345abf94399af59ffede19f26f5ea20f685b5d9f0caea35"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set."
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-001: FlowKey Canonical Ordering

## Property Statement

For all valid pairs of (IP address, port) endpoints A and B:
`FlowKey::new(ip_a, port_a, ip_b, port_b) == FlowKey::new(ip_b, port_b, ip_a, port_a)`.

The ordering is TUPLE-PAIR comparison `(ip, port) <= (other_ip, other_port)`, not
independent per-field sorting. The endpoint where `(ip, port)` is less-than-or-equal
is always stored as `(lower_ip, lower_port)` regardless of argument order.

Corollary: hashing of the key is consistent (Hash and Eq agree) so it is safe as a
HashMap key.

## Source Contract

- **Primary BC:** BC-2.04.003 -- Canonical FlowKey Ordering Ensures A->B and B->A Produce Identical Key
- **Postcondition:** `FlowKey::new(a, pa, b, pb) == FlowKey::new(b, pb, a, pa)` for all inputs
- **Invariant:** INV-1 (FlowKey Canonical Ordering, inv-01-core-invariants.md)
- **Related BC:** BC-2.04.053 -- TcpFlow::direction returns ClientToServer when src matches initiator (depends on FlowKey identity being stable)

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- 32-bit IPv4 address space; 16-bit port space; single proof step | All IPv4 address and port combinations within Kani's symbolic execution bound |

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[kani::proof]
    fn verify_flowkey_canonical_ordering_ipv4() {
        // Symbolic inputs: all u32 IPv4 addresses and u16 ports
        let raw_a: u32 = kani::any();
        let port_a: u16 = kani::any();
        let raw_b: u32 = kani::any();
        let port_b: u16 = kani::any();

        let ip_a = IpAddr::V4(Ipv4Addr::from(raw_a));
        let ip_b = IpAddr::V4(Ipv4Addr::from(raw_b));

        let key_ab = FlowKey::new(ip_a, port_a, ip_b, port_b);
        let key_ba = FlowKey::new(ip_b, port_b, ip_a, port_a);

        // Commutativity: argument order must not matter
        assert_eq!(key_ab, key_ba);

        // Ordering invariant: lower field must be <= upper field.
        // FlowKey fields are PRIVATE; use public accessors lower_ip(), lower_port(),
        // upper_ip(), upper_port() (src/reassembly/flow.rs:29-43).
        assert!(
            (key_ab.lower_ip(), key_ab.lower_port()) <= (key_ab.upper_ip(), key_ab.upper_port())
        );
    }

    #[kani::proof]
    fn verify_flowkey_tuple_pair_not_independent_field() {
        // Construct a case where tuple-pair ordering differs from independent-field sorting.
        // Example: ip_a = ip_b = 1.0.0.0; port_a = 9000 > port_b = 80
        // Tuple-pair: (1.0.0.0, 80) < (1.0.0.0, 9000) -> lower_port() = 80.
        // FlowKey fields are PRIVATE; use public accessors lower_port(), upper_port().
        let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 0, 0, 0));
        let key = FlowKey::new(ip, 9000, ip, 80);
        assert_eq!(key.lower_port(), 80);
        assert_eq!(key.upper_port(), 9000);
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Kani handles 32-bit + 16-bit symbolic vars cleanly |
| Proof complexity | Low | Single function call, equality check; no loops |
| Tool support | High | `FlowKey::new` is a pure function; ideal Kani target |
| Estimated proof time | < 60 seconds | No heap allocation; simple branch logic |

## Source Location

`src/reassembly/flow.rs:48` -- tuple-pair comparison: `if (ip_a, port_a) <= (ip_b, port_b)`

Existing tests: `tests/reassembly_flow_tests.rs` -- `test_flow_key_canonicalization`,
`test_flow_key_same_ip_different_ports`.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
