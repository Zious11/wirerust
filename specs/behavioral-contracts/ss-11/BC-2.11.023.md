---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/csv.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
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

# BC-2.11.023: CsvReporter Implements Reporter Trait and Emits One Row per Finding; Summary and AnalysisSummary Are Ignored

## Description

`CsvReporter` is a unit struct that satisfies the `Reporter` trait by implementing
`render(&self, summary: &Summary, findings: &[Finding], analyzer_summaries: &[AnalysisSummary]) -> String`.
The `summary` and `analyzer_summaries` parameters are accepted but intentionally discarded;
only the `findings` slice drives output. The returned `String` contains one data row per
`Finding` element in the slice, preceded by a fixed header row, with no trailing metadata
or summary section appended.

## Preconditions

1. `CsvReporter` is constructed (unit struct; no fields or configuration).
2. `Reporter::render` is called with a valid `&Summary`, `&[Finding]`, and
   `&[AnalysisSummary]`.
3. The `findings` slice may be empty or arbitrarily large.

## Postconditions

1. The returned `String` contains exactly one header row followed by exactly
   `findings.len()` data rows.
2. The `summary` parameter is not represented anywhere in the output.
3. The `analyzer_summaries` parameter is not represented anywhere in the output.
4. The output is a complete, self-contained RFC 4180 CSV document (header + data rows).
5. Row order is identical to the iteration order of the `findings` slice (no sorting,
   no deduplication).

## Invariants

1. Total number of rows in the output is `1 + findings.len()` (header row + one data row
   per finding).
2. The `_summary` and `_analyzer_summaries` parameters are underscore-prefixed in the
   implementation (`csv.rs:53-56`), confirming intentional non-use.
3. Row ordering is deterministic and preserves input slice order.
4. No summary section, footer, or metadata appears after the data rows.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | findings slice is empty | Output is the header row only; no data rows; valid 1-line CSV |
| EC-002 | findings slice has exactly 1 element | Output has 2 rows: header + 1 data row |
| EC-003 | summary contains non-zero total_packets | Not reflected in output; summary is silently ignored |
| EC-004 | analyzer_summaries is non-empty (e.g., TLS stats present) | Not reflected in output; analyzer_summaries silently ignored |
| EC-005 | findings slice has duplicate elements | Both are emitted as separate rows; no deduplication |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| findings=[] | Single header line only | happy-path (empty) |
| findings=[f1, f2, f3] | Header + 3 data rows = 4 total rows | happy-path |
| summary.total_packets=9999, findings=[] | Header line only; 9999 not in output | edge-case (summary ignored) |
| analyzer_summaries=[tls_summary], findings=[f1] | Header + 1 data row; tls_summary not in output | edge-case (analyzers ignored) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Row count equals 1 + findings.len() for all input sizes | unit / proptest |
| — | Summary fields (total_packets, etc.) do not appear anywhere in CSV output | unit |
| — | Row order matches findings slice order | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- this BC documents the Reporter trait compliance and the findings-only scope of CsvReporter, which is the core behavioral contract of the CSV output channel |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation -- CsvReporter is the display layer for the CSV channel; it applies injection neutralization at render time) |
| Architecture Module | SS-11 (reporter/csv.rs:51-106, Reporter trait impl) |
| Stories | S-TBD |
| Origin BC | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |

## Related BCs

- BC-2.11.020 -- depends on (column schema is part of this render contract)
- BC-2.11.021 -- depends on (injection neutralization is part of this render contract)
- BC-2.11.022 -- depends on (evidence encoding is part of this render contract)
- BC-2.11.024 -- composes with (None-field encoding produces the empty cells in data rows)
- BC-2.11.001 -- related to (JsonReporter implements the same Reporter trait but includes summary and analyzers)

## Architecture Anchors

- `src/reporter/csv.rs:51-106` -- full `impl Reporter for CsvReporter` block
- `src/reporter/csv.rs:53-56` -- `_summary` and `_analyzer_summaries` underscore-prefixed parameters confirming intentional non-use
- `src/reporter/mod.rs:26-33` -- `Reporter` trait definition
- `src/reporter/csv.rs:1-16` -- module doc scope rationale (findings-only, intentional limitation)

## Story Anchor

S-TBD -- CsvReporter implementation (LESSON-P2.03)

## VP Anchors

- — -- Reporter trait impl unit tests

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/csv.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: `impl Reporter for CsvReporter` enforces the exact `render` signature via the trait
- **documentation**: module doc comment at csv.rs:8-16 explicitly documents the findings-only scope and states it is "an intentional, documented limitation, not an oversight"
- **guard clause**: `_summary` and `_analyzer_summaries` underscore prefix at csv.rs:53-56 encodes the non-use decision structurally (compiler warns on unused parameters without the prefix)
- **inferred**: the for-loop at csv.rs:76-100 iterates only `findings`; no other data source is touched

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (in-memory Vec<u8> buffer only; returns owned String) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (CsvReporter is a unit struct) |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed -- pure transformation over a slice. The findings-only scope is a deliberate design choice documented in the module comment; callers needing summary data should use JsonReporter.
