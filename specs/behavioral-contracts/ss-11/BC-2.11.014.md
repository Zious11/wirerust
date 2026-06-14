---
document_type: behavioral-contract
level: L3
version: "1.6"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: VP-016 proof-method cell unit→integration to match VP-016 frontmatter + VP-INDEX (Wave-21 wave-level consistency lens; SS-11 reporter VP family harmonization — sibling of VP-017 fix in 86113c2; DF-SIBLING-SWEEP-001)"
  - "v1.4: re-anchor Architecture-Anchor from legacy reporter_tests.rs to authoritative reporter_terminal_tests.rs mod story_078 formalization (F-W22-BC-ANCHOR) — 2026-05-31"
  - "v1.5: DF-SIBLING-SWEEP-001 — fix stale terminal.rs sort-closure anchor: 262-280 → 269-288 (verdict_rank/confidence_rank helpers at 269-282, sort_by_key call at 284-288); verified against HEAD cfe0112a — 2026-06-01"
  - "v1.6: PG-ARP-F2-007 — fix stale terminal.rs line anchors shifted by F2 multi-tag additions (STORY-100): verdict_rank :269-275 → :287-293; confidence_rank :276-282 → :295-301; sort_by_key range :284-288 → :303-307; bucket push line 266 → 284; bounding range :269-288 → :287-307; verified against current HEAD — 2026-06-13"
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
   function in terminal.rs:287-293).
2. Confidence ranks: High=0, Medium=1, Low=2 (defined by local `confidence_rank` function
   in terminal.rs:295-301).
3. The sort key is `(verdict_rank, confidence_rank, original_index)` -- a 3-tuple
   sort that is stable (Rust's sort_by_key is stable).
4. Original index is attached at bucket insertion time (line 284) as `(i, f)` via `buckets.entry(tactic).or_default().push((i, f))`.

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
| VP-016 | Sort by verdict then confidence within bucket | integration: mitre_grouping_sorts_within_tactic_by_verdict_then_confidence |
| VP-016 | Stable emission order when verdict and confidence tie | integration: mitre_grouping_preserves_emission_order_when_verdict_and_confidence_tie |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- sorting findings by severity within each tactic bucket is part of the MITRE-grouped reporting output contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078 |
| Origin BC | BC-RPT-014 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.013 -- composes with (bucket assignment is the prerequisite; this BC governs within-bucket order)

## Architecture Anchors

- `src/reporter/terminal.rs:287-307` -- sort closure and sort_by_key call (verdict_rank helper at 287-293; confidence_rank helper at 295-301; sort_by_key call at 303-307)
- `tests/reporter_terminal_tests.rs` -- mod story_078 :: test_BC_2_11_014_sort_by_verdict_within_bucket, test_BC_2_11_014_stable_emission_order_on_tie

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:287-307` |
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
