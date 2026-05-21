---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/summary.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.018: Summary::ingest Increments total_packets, total_bytes, hosts, protocols

## Description

`Summary::ingest` is called once per successfully decoded packet. It increments
`total_packets` by 1, increments `total_bytes` by `packet.packet_len as u64`, inserts both
`packet.src_ip` and `packet.dst_ip` into the `hosts: HashSet<IpAddr>`, and increments the
per-protocol counter in `protocols: HashMap<Protocol, u64>`.

## Preconditions

1. `Summary::ingest` is called with a valid `ParsedPacket`.
2. The `ParsedPacket` has `src_ip`, `dst_ip`, `packet_len`, and `protocol` fields set.

## Postconditions

1. `total_packets` incremented by 1.
2. `total_bytes` incremented by `packet_len as u64`.
3. `hosts` set contains both `src_ip` and `dst_ip` (deduplication is the HashSet's property).
4. `protocols[packet.protocol]` incremented by 1 (entry created if absent).
5. `app_protocol_hint()` optionally increments a service counter (BC-2.12.019).

## Invariants

1. `ingest` is the ONLY method that mutates `total_packets`, `total_bytes`, `hosts`,
   and `protocols` on a `Summary`.
2. `skipped_packets` is NOT set by `ingest`; it is set by the caller in main.rs after
   the packet loop (main.rs:183, 278).
3. Both endpoints (src and dst) are inserted into the host set on every call.
   A packet between A and B inserts both A and B.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Same src_ip/dst_ip pair in multiple packets | Hosts set deduplicates; count stays at 2 |
| EC-002 | Same src appears as dst in another packet | Correctly deduplicated in host set |
| EC-003 | Packet with protocol=Protocol::Icmp | Icmp counter incremented |
| EC-004 | packet_len=0 | total_bytes incremented by 0 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 packets from different hosts | hosts set has 3+ IPs; total_packets=3 | happy-path |
| 2 packets same protocol | protocols[protocol]=2 | happy-path |
| Packet with large packet_len | total_bytes += packet_len | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | total_packets and hosts incremented correctly | unit: test_summary_host_counting |
| — | protocol counters incremented | unit: test_summary_protocol_breakdown |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- Summary::ingest is called inside CAP-12's per-target packet loop (main.rs: summary.ingest(&parsed)) as part of the Summary accumulation described in CAP-12; summary.rs (C-17) is listed in CAP-12's source sources |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (summary.rs, C-17) |
| Stories | STORY-090 |
| Origin BC | BC-SUM-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.019 -- composes with (service counter is also incremented in ingest)
- BC-2.12.020 -- composes with (unique_hosts reads from the hosts set populated here)
- BC-2.12.021 -- composes with (serialized Summary fields are populated by this function)

## Architecture Anchors

- `src/summary.rs:58-68` -- Summary::ingest implementation
- `tests/summary_tests.rs` -- test_summary_host_counting, test_summary_protocol_breakdown

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/summary.rs:58-68` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: HashSet deduplication and HashMap entry API
- **assertion**: test_summary_host_counting, test_summary_protocol_breakdown

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (mutates &mut self) |
| **Overall classification** | pure (in-memory state mutation) |

#### Refactoring Notes

No refactoring needed.
