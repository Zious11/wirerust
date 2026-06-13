---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.3: anchor-completeness — add 5 missing test back-references (STORY-046 Wave 18, F-S046-P3-001) — 2026-05-29"
  - "v1.4: FIX-P5-003 / ADV-IMPL-P06-HIGH-001 — tighten top_hosts tiebreaker: count desc then host name ASC; determinism claim now covers sort key, not just BTreeMap; add EC-004; add VP/anchor for test_summarize_top_hosts_ties_broken_alphabetically — 2026-06-01"
  - "v1.5 (2026-06-13): P19-B-08 ss-06 line-anchor re-sync — summarize() :550-601→:583-634; top_hosts sort :571-573→:604-606. Verified against current src/analyzer/http.rs (1044 lines)."
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
     - `"top_hosts"`: top 20 hosts sorted by count descending, ties broken by host name ascending
     - `"transactions"`: u64 = self.transactions
     - `"user_agents"`: map of UA string -> count
2. `top_hosts` is sorted by count descending; ties are broken by host name ascending
   (lexicographic); the result is deterministic across all runs regardless of
   HashMap/insertion order. The sort key is:
   `b.count.cmp(a.count).then_with(|| a.host.cmp(b.host))`.
   The sorted list is then truncated to at most 20 entries.
3. `recent_uris` is the first 20 entries from `self.uris` (not sorted -- insertion order).

## Invariants

1. BTreeMap key order is alphabetical and deterministic.
2. `packets_analyzed` equals `transactions`, NOT the count of parsed requests.
3. `status_codes` keys are stringified u16 values (e.g., "200", "404").
4. `summarize()` does not modify any state (read-only).
5. The `top_hosts` array in JSON output is fully deterministic: given the same set of
   (host, count) pairs, every invocation produces the same ordered array, regardless of
   HashMap internal ordering or insertion sequence. This is achieved by sorting with a
   composite key (count desc, name asc) before `.take(20)`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No flows processed | All maps empty; transactions=0 |
| EC-002 | > 20 hosts seen | top_hosts truncated to 20 (most frequent) |
| EC-003 | > 20 URIs seen | recent_uris shows first 20 (not last 20) |
| EC-004 | Multiple hosts with equal counts | top_hosts entries within the tied group appear in ascending alphabetical order by host name; order is deterministic regardless of HashMap/insertion ordering |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| After parsing 5 GET requests and 3 responses | transactions=3, methods={"GET":5}, recent_uris has 5 entries | happy-path |
| Zero traffic | transactions=0; all maps empty | edge-case |
| 25 hosts all with count=5, inserted in reverse alphabetical order | top_hosts[0..20] appear in strictly ascending alphabetical order within the tied group; result identical regardless of insertion order | tiebreaker / EC-004 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | summarize produces complete output with all required keys | unit: test_summarize_produces_complete_output (postcondition 1) |
| — | summarize includes parse_errors correctly | unit: test_parse_error_in_summarize (postcondition 1) |
| — | top_hosts is sorted by count descending and truncated to ≤ 20 entries | unit: test_summarize_top_hosts_sorted_and_truncated (postcondition 2 / EC-002) |
| — | top_hosts ties broken by host name ascending; result is deterministic regardless of insertion order | unit: test_summarize_top_hosts_ties_broken_alphabetically (postcondition 2 / invariant 5 / EC-004) |
| — | recent_uris returns first 20 URIs in insertion order, not sorted | unit: test_summarize_recent_uris_first_20 (postcondition 3 / EC-003) |
| — | BTreeMap key order is alphabetical and deterministic across runs | unit: test_summarize_btreemap_key_order_is_deterministic (invariant 1) |
| — | packets_analyzed equals transactions (response count), not request count | unit: test_summarize_packets_analyzed_equals_transactions (invariant 2) |
| — | summarize() is read-only and does not mutate any analyzer state | unit: test_summarize_does_not_mutate_state (invariant 4) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- summarize() is the primary output channel for HTTP analysis statistics |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:583-634, C-12) |
| Stories | STORY-046 |
| Origin BC | BC-HTTP-023 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.004 -- composes with (packets_analyzed = transactions = response count)
- BC-2.06.018 -- composes with (non_http_flows appears in detail map)

## Architecture Anchors

- `src/analyzer/http.rs:583-634` -- summarize() implementation
- `src/analyzer/http.rs:604-606` -- top_hosts sort: `sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)))` then `.take(20)` (FIX-P5-003)
- `tests/http_analyzer_tests.rs::test_summarize_produces_complete_output` -- covers postcondition 1 (all required keys present)
- `tests/http_analyzer_tests.rs::test_parse_error_in_summarize` -- covers postcondition 1 (parse_errors key)
- `tests/http_analyzer_tests.rs::test_summarize_top_hosts_sorted_and_truncated` -- covers postcondition 2 / EC-002 (top_hosts sort + truncation)
- `tests/http_analyzer_tests.rs::test_summarize_top_hosts_ties_broken_alphabetically` -- covers postcondition 2 / invariant 5 / EC-004 (tiebreaker: host name ASC; determinism under reverse-insertion) (FIX-P5-003)
- `tests/http_analyzer_tests.rs::test_summarize_recent_uris_first_20` -- covers postcondition 3 / EC-003 (recent_uris insertion-order slice)
- `tests/http_analyzer_tests.rs::test_summarize_btreemap_key_order_is_deterministic` -- covers invariant 1 (deterministic key ordering)
- `tests/http_analyzer_tests.rs::test_summarize_packets_analyzed_equals_transactions` -- covers invariant 2 (packets_analyzed = transactions)
- `tests/http_analyzer_tests.rs::test_summarize_does_not_mutate_state` -- covers invariant 4 (read-only, no state mutation)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:583-634` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_summarize_produces_complete_output
- **assertion**: test_summarize_top_hosts_ties_broken_alphabetically (FIX-P5-003)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads only (no mutation) |
| **Deterministic** | yes — BTreeMap ensures key order; composite sort key (count desc, host name asc) ensures top_hosts array order is fully deterministic even when multiple hosts share the same count (FIX-P5-003) |
| **Thread safety** | requires &self (shared ref) |
| **Overall classification** | pure (read-only view computation) |
