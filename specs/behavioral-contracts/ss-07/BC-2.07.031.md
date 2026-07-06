---
document_type: behavioral-contract
level: L3
version: "1.5"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: FIX-P5-003 / ADV-IMPL-P06-HIGH-001 — tighten top_snis tiebreaker: count desc then SNI name ASC; determinism claim now covers sort key; add EC-004; add VP/anchor for test_summarize_top_snis_ties_broken_alphabetically — 2026-06-01"
  - "v1.4: PG-ARP-F2-007 ss-07 full re-anchor — summarize 763-808→853-897; top_snis sort 771-773→861-862 — 2026-06-13"
  - "v1.5 (2026-07-06): silent-limit audit — add dropped_map_entries key (PC-10, u64); incremented when any distribution map (sni_counts, ja3_counts, ja3s_counts, version_counts, cipher_counts) hits MAX_MAP_ENTRIES=50,000 and a new key is dropped; counter only, no Finding; update EC, Architecture Anchors, Related BCs"
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
`cipher_suites`, `parse_errors`, `truncated_records`, `handshake_reassembly_overflows`
(BC-2.07.039), `buffer_saturation_drops` (BC-2.07.043), and `dropped_map_entries`
(this BC, v1.5). The BTreeMap ensures deterministic alphabetical key ordering in JSON output.

## Preconditions

1. `TlsAnalyzer::summarize` is called (typically once, after processing is complete).
2. No minimum data requirement; can be called on a fresh analyzer with no data.

## Postconditions

1. `AnalysisSummary.analyzer_name == "TLS"`.
2. `AnalysisSummary.packets_analyzed == self.handshakes_seen`.
3. `detail["top_snis"]` is a JSON array of up to 20 SNI strings sorted by count
   descending; ties are broken by SNI name ascending (lexicographic). The array is
   fully deterministic across runs regardless of HashMap/insertion order. Sort key:
   `b.count.cmp(a.count).then_with(|| a.sni.cmp(b.sni))`, then `.take(20)`.
4. `detail["ja3_hashes"]` is a JSON object mapping JA3 hash -> count.
5. `detail["ja3s_hashes"]` is a JSON object mapping JA3S hash -> count.
6. `detail["tls_versions"]` is a JSON object mapping version string -> count
   (keys are decimal version strings, e.g., "771" for 0x0303).
7. `detail["cipher_suites"]` is a JSON object mapping cipher name -> count.
8. `detail["parse_errors"]` is a JSON number.
9. `detail["truncated_records"]` is a JSON number.
10. `detail["dropped_map_entries"]` is a JSON number (u64). This counter accumulates the
    total count of new-key drops across ALL five distribution maps (`sni_counts`,
    `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`) caused by the
    `MAX_MAP_ENTRIES = 50,000` cap in `TlsAnalyzer::increment`. Each time `increment`
    silently drops a new key (i.e., `map.len() >= MAX_MAP_ENTRIES && !map.contains_key(&key)`),
    `TlsAnalyzer.dropped_map_entries: u64` is incremented by 1. The counter is ALWAYS
    present in the detail map, even when 0. No Finding is emitted for any map drop
    (BC-2.07.028 Invariant 1 preserved: Finding emission is decoupled from count insertion
    and still fires independently when applicable).

## Invariants

1. `detail` is a BTreeMap, so JSON output keys are alphabetically ordered
   (per LESSON-P2.09).
2. `top_snis` contains at most 20 entries; it uses sort-by-count-descending
   with tie-breaking by SNI name ascending, then `.take(20)`. The resulting
   array is fully deterministic: given the same (sni, count) pairs, every
   invocation produces the same ordered array regardless of sni_counts HashMap
   internal ordering or insertion sequence.
3. `version_counts` values are u16 keys; they are converted to String via
   `k.to_string()` (decimal) for the JSON map.
4. The `truncated_records` key was added in P1.05 for CNV-PAT-002 compliance.
5. `dropped_map_entries` is monotonically non-decreasing across the analyzer lifetime.
   It counts drops across ALL maps in aggregate (not per-map). When implementation is
   added, the `TlsAnalyzer::increment` helper must return a bool (or increment a field)
   to allow the caller to accumulate this counter.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Analyzer with no data (fresh instance) | packets_analyzed=0; all maps empty; parse_errors=0; dropped_map_entries=0 |
| EC-002 | More than 20 distinct SNIs seen | top_snis has exactly 20 entries |
| EC-003 | Version counts have multiple entries | tls_versions map has multiple entries |
| EC-004 | Multiple SNIs with equal counts | SNIs within the tied group appear in ascending alphabetical order; result is deterministic regardless of sni_counts HashMap/insertion ordering |
| EC-005 | Any distribution map hits MAX_MAP_ENTRIES=50,000; additional new keys arrive | dropped_map_entries > 0; no Finding emitted for the drops; existing-key counts still increment normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Analyzer after one clean handshake | packets_analyzed=1; top_snis has 1 entry; parse_errors=0; truncated_records=0; dropped_map_entries=0 | happy-path |
| Fresh analyzer, no data | packets_analyzed=0; all maps/arrays empty; dropped_map_entries=0 | edge-case |
| 25 SNIs all with count=1, inserted in reverse alphabetical order | top_snis[0..20] appear in strictly ascending alphabetical order within the tied group; result identical regardless of insertion order; dropped_map_entries=0 | tiebreaker / EC-004 |
| sni_counts filled to 50,000; 5 new unique SNIs arrive | dropped_map_entries=5; sni_counts.len()=50,000; no Finding emitted for drops | edge-case / EC-005 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | summarize contains all required detail keys | unit: test_summarize_output; integration: test_summarize_has_all_required_fields |
| — | truncated_records is present in detail | unit: assert detail["truncated_records"] exists |
| — | top_snis ties broken by SNI name ascending; result is deterministic regardless of insertion order | unit: test_summarize_top_snis_ties_broken_alphabetically (postcondition 3 / invariant 2 / EC-004) (FIX-P5-003) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- summarize is the statistics output method of TLS analysis |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation -- SNI strings in summary are raw) |
| Architecture Module | SS-07 (analyzer/tls.rs:853-897, C-13) |
| Stories | STORY-058 |
| Origin BC | BC-TLS-031 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.001 -- depends on (handshakes_seen drives packets_analyzed)
- BC-2.07.004 -- composes with (truncated_records is surfaced here)
- BC-2.07.028 -- composes with (dropped_map_entries counter is additive; BC-2.07.028 defines finding-still-fires-when-map-full behavior which is preserved)
- BC-2.07.029 -- composes with (parse_errors is surfaced here)

## Architecture Anchors

- `src/analyzer/tls.rs:853-897` -- `summarize` implementation
- `src/analyzer/tls.rs:861-862` -- top_snis sort: `sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)))` then `.take(20)` (FIX-P5-003)
- `src/analyzer/tls.rs` -- `TlsAnalyzer.dropped_map_entries: u64` field (to be added by implementer)
- `src/analyzer/tls.rs:379-383` -- `TlsAnalyzer::increment` helper (cap logic; needs return value or field increment to surface drops)
- `tests/tls_analyzer_tests.rs::test_summarize_output` -- covers postcondition 1-9 (all required detail keys; needs update to assert dropped_map_entries=0 for happy-path)
- `tests/tls_analyzer_tests.rs::test_summarize_top_snis_ties_broken_alphabetically` -- covers postcondition 3 / invariant 2 / EC-004 (tiebreaker: SNI name ASC; determinism under reverse-insertion) (FIX-P5-003)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:853-897` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_summarize_output; integration test_summarize_has_all_required_fields
- **assertion**: test_summarize_top_snis_ties_broken_alphabetically (FIX-P5-003)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads all count maps and counters |
| **Deterministic** | yes — BTreeMap ensures key order; composite sort key (count desc, SNI name asc) ensures top_snis array order is fully deterministic even when multiple SNIs share the same count (FIX-P5-003) |
| **Thread safety** | not thread-safe (&self, but mutable borrows of TlsAnalyzer blocked) |
| **Overall classification** | pure (read-only) |
