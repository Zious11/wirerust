---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/summary.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-11
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

# BC-2.12.020: Summary::unique_hosts Returns Sorted Deduplicated Vec<IpAddr>

## Description

`Summary::unique_hosts()` returns all unique source and destination IP addresses observed
across all ingested packets as a sorted `Vec<IpAddr>`. The IPs are collected from the
`hosts: HashSet<IpAddr>` and sorted before returning. IPv4 addresses sort before IPv6
addresses (the IpAddr Ord implementation in Rust's std sorts V4 before V6).

## Preconditions

1. `Summary::unique_hosts()` is called on a `Summary` instance.
2. `ingest` may have been called zero or more times.

## Postconditions

1. Returns a sorted Vec of all unique IpAddr values seen in `ingest` calls.
2. No duplicates in the returned Vec.
3. Empty Vec when no packets were ingested.
4. Sort order is deterministic: the same set of IPs produces the same order.

## Invariants

1. The method clones host IPs from the HashSet: `self.hosts.iter().copied().collect()`.
2. `hosts.sort()` is called before returning.
3. The underlying HashSet deduplicates automatically at insertion time; `unique_hosts`
   just sorts for deterministic output.
4. `unique_hosts` does NOT mutate the Summary; it takes `&self`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No packets ingested | Returns empty Vec |
| EC-002 | Same IP as both src and dst | Appears once (HashSet dedup) |
| EC-003 | Mix of IPv4 and IPv6 | Both returned; IPv4 sorts before IPv6 |
| EC-004 | Three packets, 3 unique hosts | Returns sorted Vec of 3 IPs |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 packets with distinct IPs | sorted Vec of 3 unique IPs | happy-path |
| Packet where src=dst | Vec length 1 (dedup) | edge-case |
| No packets | Vec::new() | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Returns sorted deduplicated Vec<IpAddr> | unit: test_summary_host_counting (asserts len==3 from 3 packets) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- unique_hosts is consumed by both the terminal reporter's host breakdown and the JSON reporter's unique_hosts array |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (summary.rs, C-17) |
| Stories | S-TBD |
| Origin BC | BC-SUM-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.018 -- depends on (hosts HashSet is populated by ingest; this method reads it)
- BC-2.11.019 -- depends on (HOSTS breakdown in terminal output calls unique_hosts)

## Architecture Anchors

- `src/summary.rs:70-74` -- unique_hosts implementation

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/summary.rs:70-74` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_summary_host_counting
- **type constraint**: IpAddr Ord implementation from std

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (reads from &self) |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
