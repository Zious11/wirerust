---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/flow.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.003: Canonical FlowKey Ordering Ensures A->B and B->A Produce Identical Key

## Description

`FlowKey::new(ip_a, port_a, ip_b, port_b)` canonically orders the two endpoints so that the
endpoint with the SMALLER (ip, port) tuple-pair is stored as (lower_ip, lower_port). This
means that a packet from A to B and a packet from B to A produce the identical FlowKey and
therefore map to the same flow entry. This is INV-1 and is load-bearing for the flow table.

## Preconditions

1. Two IP addresses and two port numbers are provided.
2. The IP addresses may be IPv4 or IPv6; comparison is lexicographic on IpAddr (IPv4 < IPv6
   in Rust's PartialOrd).

## Postconditions

1. The returned FlowKey stores the endpoint where `(ip, port) <= (other_ip, other_port)` as
   (lower_ip, lower_port).
2. `FlowKey::new(ip_a, port_a, ip_b, port_b) == FlowKey::new(ip_b, port_b, ip_a, port_a)` for
   all valid inputs.
3. Equality and hashing are consistent (used as HashMap key).

## Invariants

1. The ordering is TUPLE-PAIR comparison: `(ip_a, port_a) <= (ip_b, port_b)`. It is NOT
   independent per-field (sort-ip-then-sort-port separately). This is critical: two connections
   sharing one field but differing in the other would incorrectly merge under independent sorting.
2. The FlowKey is immutable once constructed; flow direction is determined separately by
   TcpFlow::direction().

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A->B and B->A with same IP, different ports | lower_port wins: FlowKey same in both directions |
| EC-002 | Same IP:port on both sides (loopback self-connection) | lower_ip = upper_ip = same; lower_port = upper_port = same |
| EC-003 | IPv4 vs IPv6 addresses | IPv4 < IPv6 in IpAddr PartialOrd; IPv4 endpoint always becomes lower |
| EC-004 | IP addresses equal, port_a < port_b | port_a side is lower; port_b side is upper |
| EC-005 | IP addresses equal, ports equal | Degenerate; lower == upper; still valid FlowKey |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| new(1.1.1.1, 5000, 2.2.2.2, 80) | FlowKey { lower: (1.1.1.1,5000), upper: (2.2.2.2,80) } | happy-path |
| new(2.2.2.2, 80, 1.1.1.1, 5000) | FlowKey { lower: (1.1.1.1,5000), upper: (2.2.2.2,80) } (same!) | happy-path |
| new(1.1.1.1, 443, 1.1.1.1, 55000) | FlowKey { lower: (1.1.1.1,443), upper: (1.1.1.1,55000) } | edge-case |
| new(1.1.1.1, 55000, 1.1.1.1, 443) | Same as above (443 < 55000) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | FlowKey::new is commutative: new(a,pa,b,pb) == new(b,pb,a,pa) for all (a,pa,b,pb) | proptest: generate random IP+port pairs |
| VP-TBD | Ordering uses tuple-pair comparison not independent field ordering | unit: construct case where tuple-pair differs from independent-field ordering |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- canonical FlowKey ordering is the identity contract underpinning the entire reassembly engine's flow table |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering) |
| Architecture Module | SS-04 (reassembly/flow.rs, C-7) |
| Stories | S-TBD |
| Origin BC | BC-RAS-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.004 -- composes with (FlowKey is used to look up flow on SYN)
- BC-2.04.053 -- composes with (TcpFlow::direction uses the key to determine client/server)

## Architecture Anchors

- `src/reassembly/flow.rs:34` -- tuple-pair comparison: `if (ip_a, port_a) <= (ip_b, port_b)`
- `tests/reassembly_flow_tests.rs` -- test_flow_key_canonicalization, test_flow_key_same_ip_different_ports

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:34` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: explicit if-else on tuple-pair comparison
- **assertion**: test_flow_key_canonicalization, test_flow_key_same_ip_different_ports

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed -- pure function suitable for formal verification (Kani).
