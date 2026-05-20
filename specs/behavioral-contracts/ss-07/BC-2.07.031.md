---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
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

# BC-2.07.031: summarize Emits AnalysisSummary with TLS Stats Detail Map

## Description

`TlsAnalyzer::summarize` returns an `AnalysisSummary` with `analyzer_name = "TLS"`,
`packets_analyzed = handshakes_seen`, and a detail BTreeMap with the following keys:
`top_snis` (top 20 SNIs by count), `ja3_hashes`, `ja3s_hashes`, `tls_versions`,
`cipher_suites`, `parse_errors`, and `truncated_records`. The BTreeMap ensures
deterministic alphabetical key ordering in JSON output.

## Preconditions

1. `TlsAnalyzer::summarize` is called (typically once, after processing is complete).
2. No minimum data requirement; can be called on a fresh analyzer with no data.

## Postconditions

1. `AnalysisSummary.analyzer_name == "TLS"`.
2. `AnalysisSummary.packets_analyzed == self.handshakes_seen`.
3. `detail["top_snis"]` is a JSON array of up to 20 SNI strings sorted by count
   descending.
4. `detail["ja3_hashes"]` is a JSON object mapping JA3 hash -> count.
5. `detail["ja3s_hashes"]` is a JSON object mapping JA3S hash -> count.
6. `detail["tls_versions"]` is a JSON object mapping version string -> count
   (keys are decimal version strings, e.g., "771" for 0x0303).
7. `detail["cipher_suites"]` is a JSON object mapping cipher name -> count.
8. `detail["parse_errors"]` is a JSON number.
9. `detail["truncated_records"]` is a JSON number.

## Invariants

1. `detail` is a BTreeMap, so JSON output keys are alphabetically ordered
   (per LESSON-P2.09).
2. `top_snis` contains at most 20 entries; it uses sort-by-count-descending then
   `.take(20)`.
3. `version_counts` values are u16 keys; they are converted to String via
   `k.to_string()` (decimal) for the JSON map.
4. The `truncated_records` key was added in P1.05 for CNV-PAT-002 compliance.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Analyzer with no data (fresh instance) | packets_analyzed=0; all maps empty; parse_errors=0 |
| EC-002 | More than 20 distinct SNIs seen | top_snis has exactly 20 entries |
| EC-003 | Version counts have multiple entries | tls_versions map has multiple entries |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Analyzer after one clean handshake | packets_analyzed=1; top_snis has 1 entry; parse_errors=0; truncated_records=0 | happy-path |
| Fresh analyzer, no data | packets_analyzed=0; all maps/arrays empty | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | summarize contains all required detail keys | unit: test_summarize_output; integration: test_summarize_has_all_required_fields |
| VP-TBD | truncated_records is present in detail | unit: assert detail["truncated_records"] exists |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- summarize is the statistics output method of TLS analysis |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation -- SNI strings in summary are raw) |
| Architecture Module | SS-07 (analyzer/tls.rs:763-808, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-031 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.001 -- depends on (handshakes_seen drives packets_analyzed)
- BC-2.07.004 -- composes with (truncated_records is surfaced here)
- BC-2.07.029 -- composes with (parse_errors is surfaced here)

## Architecture Anchors

- `src/analyzer/tls.rs:763-808` -- `summarize` implementation
- `tests/tls_analyzer_tests.rs` -- test_summarize_output

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:763-808` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_summarize_output; integration test_summarize_has_all_required_fields

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads all count maps and counters |
| **Deterministic** | yes (BTreeMap ensures ordering) |
| **Thread safety** | not thread-safe (&self, but mutable borrows of TlsAnalyzer blocked) |
| **Overall classification** | pure (read-only) |
