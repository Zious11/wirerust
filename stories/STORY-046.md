---
document_type: story
story_id: "STORY-046"
epic_id: "E-4"
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.023.md
input-hash: "c7fcb8b"
traces_to: .factory/specs/prd.md
points: 3
depends_on: [STORY-041, STORY-042, STORY-043, STORY-044, STORY-045]
blocks: [STORY-076]
behavioral_contracts:
  - BC-2.06.023
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 18
target_module: src/analyzer/http.rs
subsystems: [SS-06]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-046`

# STORY-046: HTTP Analyzer Summary Output

## Narrative
- **As a** forensic analyst or SOC operator
- **I want to** receive a complete, deterministic `AnalysisSummary` from the HTTP analyzer after processing a pcap — including transaction count, method/host/UA/status-code maps, URI list, parse-error count, non-HTTP flow count, and poisoned-bytes-skipped count
- **So that** I can understand the full scope of HTTP activity in a capture and identify broad traffic patterns

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.06.023 | summarize Emits AnalysisSummary with HTTP Stats Detail Map |

## Acceptance Criteria

### AC-001 (traces to BC-2.06.023 postcondition 1)
`HttpAnalyzer::summarize()` returns an `AnalysisSummary` with `analyzer_name = "HTTP"` and `packets_analyzed = self.transactions` (the parsed response count, not request count).
- **Test:** `test_summarize_produces_complete_output`

### AC-002 (traces to BC-2.06.023 postcondition 1 detail map keys)
The `detail` BTreeMap contains exactly these keys: `"methods"`, `"non_http_flows"`, `"parse_errors"`, `"poisoned_bytes_skipped"`, `"recent_uris"`, `"status_codes"`, `"top_hosts"`, `"transactions"`, `"user_agents"`. No extra or missing keys.
- **Test:** `test_summarize_produces_complete_output`

### AC-003 (traces to BC-2.06.023 postcondition 2)
`top_hosts` is sorted by count descending and truncated to at most 20 entries when more than 20 distinct hosts are observed; ties broken by host name ascending (lexicographic); deterministic across runs regardless of HashMap/insertion order.
- **Test:** `test_summarize_top_hosts_sorted_and_truncated`

### AC-004 (traces to BC-2.06.023 postcondition 3)
`recent_uris` is the first 20 entries from `self.uris` (insertion order, not sorted). When fewer than 20 URIs exist, all are included.
- **Test:** `test_summarize_recent_uris_first_20`

### AC-005 (traces to BC-2.06.023 invariant 1)
The `detail` BTreeMap uses alphabetical key order, making output deterministic across runs (per LESSON-P2.09). Running `summarize()` twice on the same analyzer produces identical output.
- **Test:** `test_summarize_btreemap_key_order_is_deterministic`

### AC-006 (traces to BC-2.06.023 invariant 2)
`packets_analyzed` equals `transactions` (response count), NOT the count of parsed requests. This is confirmed by parsing 5 requests and 3 responses: `packets_analyzed = 3`.
- **Test:** `test_summarize_packets_analyzed_equals_transactions`

### AC-007 (traces to BC-2.06.023 invariant 4)
`summarize()` does not modify any analyzer state — it is a read-only operation. Calling `summarize()` between two `on_data` calls does not affect subsequent parsing.
- **Test:** `test_summarize_does_not_mutate_state`

### AC-008 (traces to BC-2.06.023 edge case EC-001)
When no flows have been processed (zero traffic), `summarize()` returns all maps empty, `transactions=0`, `parse_errors=0`, `non_http_flows=0`, `poisoned_bytes_skipped=0`, and `recent_uris=[]`.
- **Test:** `test_parse_error_in_summarize`

### AC-009 (traces to BC-2.06.023 postcondition 2 / invariant 5 / EC-004)
When multiple hosts share the same count, the tied group is ordered by host name ascending (lexicographic); the full `top_hosts` array is deterministic across all runs regardless of HashMap internal ordering or insertion sequence. Sort key: `b.count.cmp(a.count).then_with(|| a.host.cmp(b.host))`.
- **Test:** `test_summarize_top_hosts_ties_broken_alphabetically`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| summarize | src/analyzer/http.rs:550-601 | pure-core (read-only; returns owned value) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No flows processed | All maps empty; transactions=0 |
| EC-002 | > 20 hosts seen | top_hosts truncated to 20 most frequent |
| EC-003 | > 20 URIs seen | recent_uris shows first 20 (not last 20) |
| EC-004 | status_codes keys are stringified u16 values | "200", "404" etc. (not integers) |
| EC-005 | parse_errors > 0 from prior parse errors | parse_errors appears correctly in detail map |
| EC-006 | Multiple hosts with equal counts, inserted in reverse alphabetical order | top_hosts tied group appears in ascending alphabetical order; result identical regardless of insertion order (BC-2.06.023 EC-004) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/http.rs (summarize) | pure-core | Takes &self; returns owned AnalysisSummary; no mutation |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| Referenced code (http.rs:550-601) | ~2,000 |
| Test files (http_analyzer_tests.rs, summary_tests.rs) | ~2,000 |
| BC files (1 BC) | ~1,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~9,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-009 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement `summarize()` per BC-2.06.023 (BTreeMap; exact key set; packets_analyzed=transactions; top_hosts sorted by count desc then host name asc, truncated to 20; recent_uris first 20; status_codes keys as stringified u16)
4. [ ] Confirm `summarize()` is read-only (no `&mut self`)
5. [ ] Run all tests; verify all pass
6. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-041 | `transactions` counts responses only; uris is a Vec<String> (insertion order) | Use BTreeMap for detail map to guarantee alphabetical key order per LESSON-P2.09 | `status_codes` keys must be stringified u16 (e.g., "200") — not integer keys |
| STORY-044 | `parse_errors`, `non_http_flows`, `poisoned_bytes_skipped` are aggregate counters on HttpAnalyzer | All aggregate counters are preserved across `on_flow_close` calls | Ensure `summarize()` signature is `&self` not `&mut self` |
| STORY-045 | `top_hosts` requires sorting by count descending; `MAX_URIS` limits uris Vec to 10000 | Return first 20 URIs (not last 20, not sorted) | `summarize()` must not mutate state — no sort-in-place on the live data structures |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| BTreeMap (not HashMap) for `detail` to guarantee alphabetical key order | BC-2.06.023 invariant 1 / LESSON-P2.09 | Code review: confirm `BTreeMap::new()` not `HashMap::new()` |
| `packets_analyzed` must equal `transactions` (response count) | BC-2.06.023 invariant 2 | Unit test: AC-006 |
| `summarize()` must be a read-only operation | BC-2.06.023 invariant 4 | Code review: confirm `&self` not `&mut self` |
| `top_hosts` sorted by count descending; ties broken by host name ascending; truncated to 20 entries; deterministic across runs | BC-2.06.023 postcondition 2 / invariant 5 | Unit tests: AC-003, AC-009 |
| `recent_uris` is first 20 entries in insertion order | BC-2.06.023 postcondition 3 | Unit test: AC-004 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | BTreeMap, Vec::iter, sort_by, take |
| (AnalysisSummary type from summary.rs) | internal | Return type for summarize() |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/http.rs | modify | summarize() implementation (550-601): BTreeMap detail map, top_hosts sort/truncate, recent_uris first 20 |
| tests/http_analyzer_tests.rs | modify | Add: test_summarize_produces_complete_output (AC-001/AC-002), test_parse_error_in_summarize (AC-008), test_summarize_top_hosts_sorted_and_truncated (AC-003), test_summarize_recent_uris_first_20 (AC-004), test_summarize_btreemap_key_order_is_deterministic (AC-005), test_summarize_packets_analyzed_equals_transactions (AC-006), test_summarize_does_not_mutate_state (AC-007), test_summarize_top_hosts_ties_broken_alphabetically (AC-009) |

## Changelog

| Version | Date | Author | Note |
|---------|------|--------|------|
| 1.0 | 2026-05-21 | story-writer | Initial story |
| 1.2 | 2026-06-01 | story-writer | FIX-P5-003 — add AC-009 (top_hosts tiebreaker: host name ASC, deterministic); expand AC-003 description with tiebreaker + determinism; add EC-006; update Architecture Compliance Rules and FSR for top_hosts determinism (BC-2.06.023 v1.4 postcondition 2 / invariant 5 / EC-004) |
| 1.1 | 2026-05-29 | story-writer | FSR-table completeness — enumerate all 7 BC-2.06.023 formalization tests (F-S046-P4-001) |
