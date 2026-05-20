---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/dns.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-08
capability: CAP-08
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

# BC-2.08.004: DnsAnalyzer NEVER Emits Findings (Statistics-Only by Design)

## Description

`DnsAnalyzer::analyze(packet)` unconditionally returns `vec![]`. The DNS subsystem is a
pure statistics collector (query count, response count) with no anomaly detection. It does not
emit `Finding` objects for any DNS traffic condition. This is Smell #5 in the domain debt --
low severity, by design, not a bug.

## Preconditions

1. DnsAnalyzer receives a packet matching can_decode() (port 53, TCP or UDP).
2. The packet is passed to analyze().

## Postconditions

1. `analyze(packet)` returns `Vec<Finding>` that is always empty.
2. Internal counters (`query_count` or `response_count`) are incremented based on QR bit.
3. No Finding is constructed or allocated.

## Invariants

1. The empty return is unconditional. No DNS condition (NXDOMAIN, volume, TTL, etc.) will
   cause a Finding to be emitted by the current implementation.
2. DNS detection findings are explicitly out of scope (see Section 1.5 Out of Scope).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Malformed DNS payload (truncated, < 12 bytes) | `is_query` returns `false` (length guard fires before bit test); `response_count` is unconditionally incremented; empty findings returned |
| EC-002 | DNS-over-TCP (port 53, TCP) | Counted; empty findings |
| EC-003 | DNS-over-UDP (port 53, UDP) | Counted; empty findings |
| EC-004 | Very high DNS query volume | Counted; empty findings (no flood detection) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Any DNS packet | analyze() returns vec![] | happy-path |
| 1000 DNS query packets | findings.len() == 0; query_count == 1000 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | analyze() always returns empty Vec | unit: test_dns_analyzer_counts_queries asserts findings.is_empty() |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-08 ("DNS traffic analysis") per capabilities.md §CAP-08 |
| Capability Anchor Justification | CAP-08 ("DNS traffic analysis") per capabilities.md §CAP-08 -- the never-emit contract is the defining behavioral boundary of the current DNS implementation |
| L2 Domain Invariants | None |
| Architecture Module | SS-08 (analyzer/dns.rs, C-13) |
| Stories | S-TBD |
| Origin BC | BC-DNS-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.08.002 -- composes with (QR-bit dispatch is the only action in analyze())
- BC-2.08.003 -- composes with (summarize is the primary output mechanism for DNS stats)

## Architecture Anchors

- `src/analyzer/dns.rs` -- analyze() returning vec![]

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/dns.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **assertion**: test_dns_analyzer_counts_queries asserts findings.is_empty()

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates query_count/response_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed (pure logic; counter mutation is trivial) |
