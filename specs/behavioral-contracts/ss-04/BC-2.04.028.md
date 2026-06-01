---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/mod.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:620-658 → mod.rs:706-744 (summarize fn); LESSON-P2.09 doc comment at mod.rs:618 → mod.rs:703. — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.028: summarize Returns AnalysisSummary with Reassembly Stats Detail Map

## Description

`TcpReassembler::summarize()` produces an `AnalysisSummary` whose `analyzer_name` is
`"TCP Reassembly"`, `packets_analyzed` equals `stats.packets_tcp`, and `detail` is a
`BTreeMap<String, serde_json::Value>` containing every `ReassemblyStats` field plus the
derived `flows_completed` key. The `BTreeMap` ordering guarantees alphabetical, deterministic
JSON output across runs.

## Preconditions

1. `TcpReassembler` has been constructed with a valid `ReassemblyConfig`.
2. Zero or more packets have been processed; stats fields reflect cumulative totals.
3. `summarize()` may be called at any time (before or after `finalize()`).

## Postconditions

1. Returns `AnalysisSummary` with `analyzer_name == "TCP Reassembly"`.
2. `packets_analyzed == stats.packets_tcp` (TCP packets only, not total packets_processed).
3. `detail` contains exactly these keys (BTreeMap, alphabetical order in JSON):
   - `bytes_reassembled`
   - `dropped_findings`
   - `evictions`
   - `flows_completed` (derived: `flows_fin + flows_rst`)
   - `flows_expired`
   - `flows_fin`
   - `flows_partial`
   - `flows_rst`
   - `flows_total`
   - `packets_processed`
   - `packets_skipped_non_tcp`
   - `segments_depth_exceeded`
   - `segments_duplicates`
   - `segments_inserted`
   - `segments_out_of_window`
   - `segments_overlaps`
   - `segments_segment_limit`
4. Each value is a `serde_json::Value::Number` holding the current counter value.
5. The `detail` BTreeMap is freshly constructed on each call; it does not alias internal state.

## Invariants

1. `flows_completed == flows_fin + flows_rst` always; this derived key provides a summary
   convenience value for callers that do not want to sum two fields themselves.
2. `dropped_findings` is included so callers can detect when the MAX_FINDINGS cap was hit
   (INV-6 observability requirement, LESSON-P1.01 / #73).
3. The BTreeMap ordering guarantees deterministic key ordering in the JSON analyzer summary
   section across runs on any platform (LESSON-P2.09).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | summarize() called on freshly-constructed reassembler before any packets | All counters are 0; AnalysisSummary returned with all-zero detail |
| EC-002 | summarize() called after finalize() | Returns accurate snapshot; finalize does not reset stats |
| EC-003 | packets_processed > packets_tcp (non-TCP packets were processed) | packets_analyzed == packets_tcp (not packets_processed) |
| EC-004 | dropped_findings > 0 after MAX_FINDINGS cap hit | dropped_findings key reflects the count accurately |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Fresh reassembler, no packets | AnalysisSummary{analyzer_name="TCP Reassembly", packets_analyzed=0, detail: all zeros} | happy-path |
| 3 TCP packets processed, 1 flow, 10 bytes reassembled | packets_analyzed=3, detail["flows_total"]=1, detail["bytes_reassembled"]=10 | happy-path |
| 5 non-TCP + 3 TCP packets processed | packets_analyzed=3, detail["packets_processed"]=8, detail["packets_skipped_non_tcp"]=5 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | summarize() contains exactly the documented key set (no missing, no extra keys) | unit: test_summarize_returns_reassembly_stats |
| — | flows_completed == flows_fin + flows_rst in every summary | unit: property test |
| — | packets_analyzed == packets_tcp, not packets_processed | unit: inject non-TCP packets, assert |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- summarize() is the observability surface for the TCP reassembly engine's operational statistics |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap -- dropped_findings key makes cap observable) |
| Architecture Module | SS-04 (reassembly/mod.rs:706-744, C-6) |
| Stories | STORY-012 |
| Origin BC | BC-RAS-028 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.014 -- related to (total_memory accounting feeds evictions stat)
- BC-2.04.024 -- related to (dropped_findings key reports MAX_FINDINGS cap hits)
- BC-2.04.027 -- related to (segments_depth_exceeded key reflects depth counter)

## Architecture Anchors

- `src/reassembly/mod.rs:706-744` -- summarize() implementation
- `src/reassembly/stats.rs` -- ReassemblyStats fields (source of all counters)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:706-744` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_summarize_returns_reassembly_stats (reassembly_engine_tests) verifies presence of stat fields
- **type constraint**: BTreeMap<String, serde_json::Value> return type enforces key-value contract
- **documentation**: LESSON-P2.09 doc comment at mod.rs:703 explains BTreeMap ordering rationale

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads self.stats (immutable borrow) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (immutable borrow, no mutation) |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed -- pure read of internal counters. Suitable for formal verification of the `flows_completed` derivation invariant.
