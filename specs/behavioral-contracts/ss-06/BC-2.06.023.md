---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/http.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-06
capability: CAP-06
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

# BC-2.06.023: summarize Emits AnalysisSummary with HTTP Stats Detail Map

## Description

`HttpAnalyzer::summarize()` returns an `AnalysisSummary` with `analyzer_name = "HTTP"`,
`packets_analyzed = self.transactions` (response count), and a `BTreeMap` detail map
containing all HTTP statistics. The BTreeMap ensures keys are alphabetically ordered and
the output is deterministic across runs (per LESSON-P2.09). The detail map includes exactly
the keys listed in the postconditions.

## Preconditions

1. `HttpAnalyzer::summarize()` is called.

## Postconditions

1. Returns AnalysisSummary with:
   - `analyzer_name = "HTTP"`
   - `packets_analyzed = self.transactions` (parsed response count)
   - `detail` BTreeMap with keys (alphabetical order):
     - `"methods"`: map of method -> count
     - `"non_http_flows"`: u64 count
     - `"parse_errors"`: u64 count
     - `"poisoned_bytes_skipped"`: u64 count
     - `"recent_uris"`: first 20 URIs from self.uris
     - `"status_codes"`: map of status_code_str -> count
     - `"top_hosts"`: top 20 hosts sorted by frequency (desc)
     - `"transactions"`: u64 = self.transactions
     - `"user_agents"`: map of UA string -> count
2. `top_hosts` is sorted by count descending and truncated to 20 entries.
3. `recent_uris` is the first 20 entries from `self.uris` (not sorted -- insertion order).

## Invariants

1. BTreeMap key order is alphabetical and deterministic.
2. `packets_analyzed` equals `transactions`, NOT the count of parsed requests.
3. `status_codes` keys are stringified u16 values (e.g., "200", "404").
4. `summarize()` does not modify any state (read-only).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No flows processed | All maps empty; transactions=0 |
| EC-002 | > 20 hosts seen | top_hosts truncated to 20 (most frequent) |
| EC-003 | > 20 URIs seen | recent_uris shows first 20 (not last 20) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| After parsing 5 GET requests and 3 responses | transactions=3, methods={"GET":5}, recent_uris has 5 entries | happy-path |
| Zero traffic | transactions=0; all maps empty | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | summarize produces complete output with all required keys | unit: test_summarize_produces_complete_output |
| — | summarize includes parse_errors correctly | unit: test_parse_error_in_summarize |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- summarize() is the primary output channel for HTTP analysis statistics |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:550-601, C-12) |
| Stories | STORY-046 |
| Origin BC | BC-HTTP-023 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.004 -- composes with (packets_analyzed = transactions = response count)
- BC-2.06.018 -- composes with (non_http_flows appears in detail map)

## Architecture Anchors

- `src/analyzer/http.rs:550-601` -- summarize() implementation
- `tests/http_analyzer_tests.rs` -- test_summarize_produces_complete_output, test_parse_error_in_summarize

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:550-601` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_summarize_produces_complete_output

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads only (no mutation) |
| **Deterministic** | yes (BTreeMap ensures key order) |
| **Thread safety** | requires &self (shared ref) |
| **Overall classification** | pure (read-only view computation) |
