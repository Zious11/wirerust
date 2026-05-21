---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.003: summarize Emits AnalysisSummary with dns_queries and dns_responses

## Description

`DnsAnalyzer::summarize()` returns an `AnalysisSummary` with `analyzer_name == "DNS"`,
`packets_analyzed == query_count + response_count`, and a `detail` BTreeMap containing two
numeric JSON values: `"dns_queries"` and `"dns_responses"`. This summary is consumed by
the reporter layer to display DNS statistics.

## Preconditions

1. `DnsAnalyzer` has processed zero or more packets via `analyze`.
2. `summarize` is called.

## Postconditions

1. Returns `AnalysisSummary { analyzer_name: "DNS", packets_analyzed: query_count + response_count, detail: BTreeMap { "dns_queries": query_count, "dns_responses": response_count } }`.
2. `detail["dns_queries"]` is a `serde_json::Value::Number` equal to `query_count`.
3. `detail["dns_responses"]` is a `serde_json::Value::Number` equal to `response_count`.
4. The BTreeMap has exactly two keys: "dns_queries" and "dns_responses".
5. `packets_analyzed` equals `query_count + response_count` (total packets seen by analyze).

## Invariants

1. The detail map uses `BTreeMap<String, serde_json::Value>` so key order is deterministic:
   "dns_queries" comes before "dns_responses" in lexicographic order.
2. Both values are JSON numbers (u64-compatible); no null, string, or array values.
3. `packets_analyzed` is always the sum of both counters.
4. `summarize` does NOT reset counters; it is a read-only snapshot.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No packets analyzed | packets_analyzed=0, dns_queries=0, dns_responses=0 |
| EC-002 | 1000 queries, 0 responses | dns_queries=1000, dns_responses=0, packets_analyzed=1000 |
| EC-003 | summarize called twice | Same values both times (no counter reset) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 queries analyzed | dns_queries=3, dns_responses=0, packets_analyzed=3 | happy-path |
| 2 queries + 1 response | dns_queries=2, dns_responses=1, packets_analyzed=3 | happy-path |
| No packets | dns_queries=0, dns_responses=0, packets_analyzed=0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-019 | detail map contains exactly dns_queries and dns_responses keys | unit: assert detail keys == {"dns_queries", "dns_responses"} |
| VP-019 | packets_analyzed == query_count + response_count | unit: test_dns_summarize |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-08 ("DNS traffic analysis") per capabilities.md §CAP-08 |
| Capability Anchor Justification | CAP-08 ("DNS traffic analysis") per capabilities.md §CAP-08 -- summarize is the primary output mechanism for the DNS statistics-only analysis |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-08 (analyzer/dns.rs, C-11) |
| Stories | S-TBD |
| Origin BC | BC-DNS-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.08.002 -- composes with (query/response counts are input to summarize)
- BC-2.08.004 -- related to (DNS has no findings; summary is the only output)

## Architecture Anchors

- `src/analyzer/dns.rs:72-88` -- summarize implementation

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/dns.rs:72-88` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: BTreeMap usage guarantees deterministic key ordering

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads query_count/response_count (immutable read of self) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (&self) |
| **Overall classification** | pure (snapshot; no mutation) |

## Refactoring Notes

No refactoring needed.
