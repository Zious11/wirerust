---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/terminal.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
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

# BC-2.11.014: Within Tactic Bucket: Sort by Verdict, Confidence, Emission Order

## Description

Within each MITRE tactic bucket in the grouped rendering mode, findings are sorted by a
three-key sort: (1) verdict rank ascending (Likely < Inconclusive < Unlikely), (2) confidence
rank ascending (High < Medium < Low), (3) original emission index ascending (stable tertiary
key). This ordering surfaces the highest-severity findings at the top of each tactic section.

## Preconditions

1. `TerminalReporter.show_mitre_grouping = true`.
2. At least two findings share the same tactic bucket.

## Postconditions

1. Within a bucket, findings appear sorted by verdict rank (Likely first).
2. Among findings with the same verdict, they are sorted by confidence rank (High first).
3. Among findings with the same verdict and confidence, original emission order is preserved
   (stable sort by original index).
4. The sort does NOT affect findings in different buckets.

## Invariants

1. Verdict ranks: Likely=0, Inconclusive=1, Unlikely=2 (defined by local `verdict_rank`
   closure in terminal.rs:262-267).
2. Confidence ranks: High=0, Medium=1, Low=2 (defined by local `confidence_rank` closure
   in terminal.rs:268-273).
3. The sort key is `(verdict_rank, confidence_rank, original_index)` -- a 3-tuple
   sort that is stable (Rust's sort_by_key is stable).
4. Original index is attached at bucket insertion time (line 258) as `(i, f)`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All same verdict and confidence | Original emission order preserved |
| EC-002 | Mixed verdicts, same confidence | Likely before Inconclusive before Unlikely |
| EC-003 | Mixed confidence, same verdict | High before Medium before Low |
| EC-004 | Single finding in bucket | Trivially sorted; rendered as-is |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Bucket: [Inconclusive/Low at i=0, Likely/High at i=1] | Likely/High rendered first | happy-path |
| Bucket: [Likely/Low at i=0, Likely/High at i=1] | Likely/High rendered first | happy-path |
| Bucket: [Likely/High at i=0, Likely/High at i=1] | i=0 rendered first (stable) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Sort by verdict then confidence within bucket | unit: mitre_grouping_sorts_within_tactic_by_verdict_then_confidence |
| VP-TBD | Stable emission order when verdict and confidence tie | unit: mitre_grouping_preserves_emission_order_when_verdict_and_confidence_tie |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- sorting findings by severity within each tactic bucket is part of the MITRE-grouped reporting output contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | S-TBD |
| Origin BC | BC-RPT-014 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.013 -- composes with (bucket assignment is the prerequisite; this BC governs within-bucket order)

## Architecture Anchors

- `src/reporter/terminal.rs:262-280` -- sort closure and sort_by_key call
- `tests/reporter_tests.rs` -- mitre_grouping_sorts_within_tactic_by_verdict_then_confidence, mitre_grouping_preserves_emission_order_when_verdict_and_confidence_tie

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:262-280` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: mitre_grouping_sorts_within_tactic_by_verdict_then_confidence
- **type constraint**: Rust sort_by_key is stable (std guarantee)

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
