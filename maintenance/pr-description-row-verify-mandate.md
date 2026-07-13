# PR Description Per-Test Table Row-Verify Mandate

**Policy reference:** PG-W74-PRDESC-ROW-VERIFY  
**Finding reference:** F-W74G-P3-001 (wave-74 gate adversarial convergence Pass 3, W3)  
**Codification story:** STORY-165 AC-165-002  
**Added:** 2026-07-13 (STORY-165 AC-165-002)

---

## Background

PR #397 (STORY-164, wave-74) included a test-evidence table in the delivery doc
(`code-delivery/STORY-164/pr-description.md`). Wave-74 gate adversarial convergence
Pass 3 (W3) (F-W74G-P3-001, HIGH) found that the table's aggregate count row claimed
"python 101/101" across both bin/ test suites, but that count was computed before the
final 10-test suite (`bin/test_changelog_gate_content.py`) was complete; the row also
cited a pytest run output that did not match the actual `bin/test_changelog_gate_content.py`
output format. The delivery doc also carried a per-test results table listing T01–T22
per-test results for `bin/test_validate_citations.py`; neither the pr-reviewer nor the
pr-manager agent cross-checked any entry in that table against the actual test function
names in the source file.

The wave-74 gate code review disposition table
(`.factory/cycles/wave-74/wave-gate/code-review.md:104`) records MINOR and NIT
findings for the wave — none address a PR-description row-verify requirement. The gap
was not caught at the code-review level; it surfaced at the adversarial-convergence
level. A PR description could carry a stale aggregate count or a fabricated per-test
table and the review pipeline would pass it unchallenged.

Root cause: no mandate exists requiring agents to cross-check aggregate test counts or
spot-verify per-test results tables when they appear in PR descriptions. This document
codifies that mandate.

---

## Scope

This mandate applies whenever a PR description carries:
- A **per-test results table** — a markdown table or bulleted list enumerating individual
  test identifiers (e.g., T01–T22, B01–B10) with pass/fail or similar status annotations.
- Any **claimed aggregate test count or aggregate result** (e.g., "22 passed", "101/101",
  "22 + 10 = 32 tests pass") in a test-evidence section of the PR description.

Such tables and counts are common in E-11 governance and tooling stories.

The mandate applies to:
- **pr-reviewer** agents performing the gate-level or story-level PR review pass
- **pr-manager** agents coordinating the PR lifecycle

Per-test row-verification (Mandate item 1) does NOT apply to sections that contain only
aggregate counts with no individually-named test rows. Aggregate-count cross-check
(Mandate item 2) applies to any claimed count regardless of whether per-test rows are
present.

---

## Mandate

The pr-reviewer and pr-manager agents MUST perform BOTH of the following checks where
applicable:

### 1. Per-Test Row-Verify (when per-test rows are present)

Row-verify **at least three randomly-selected entries** from any per-test results table
in the PR description by:

1. Locating the test file named in the PR description.
2. Reading that file to confirm the test function name for each selected row exists at
   the line or location implied by the table entry.
3. Recording in the review that row-verification was performed, naming the verified rows
   and their source locations. Example recording:

   > Row-verified T01 (`test_T01_valid_line_citation_passes`,
   > `bin/test_validate_citations.py:120`), T12
   > (`test_T12_malformed_line_reported`, line 278), T22
   > (`test_T22_unreadable_target_file`, line 553).

A table with **fewer than three rows** requires verification of all rows — the floor
is the actual row count when the table has fewer than three entries.

### 2. Aggregate-Count Cross-Check (when aggregate counts are claimed)

Cross-check every claimed aggregate count or aggregate result in the PR description's
test-evidence section (e.g., "22 passed", "101/101", "22 + 10 = 32") against the actual
test-run or CI output for the PR HEAD commit, and record the cross-check.

A claimed aggregate count that **cannot be matched to an actual run output** is a
**blocking review finding**.

---

## Fabrication Risk Rationale

A per-test results table claiming "22 tests PASS" is unverifiable without reading the
source file. Row-verification prevents:

- **Copy-paste errors:** test IDs and function names mis-keyed from a prior story's table
- **Auto-generation hallucinations:** fabricated function names that do not exist in the
  file
- **Count drift:** function names that existed at the time of PR authorship but have since
  been renamed, split, or removed by a prior commit
- **Label drift:** test identifiers (T01, T22) that map to different test functions than
  the table implies
- **Stale/pre-completion aggregate counts:** a count claimed in the PR description
  (e.g., "101/101", "22 + 10 = 32") computed before the full test suite was complete, or
  against an earlier commit, producing a number that does not match the actual CI run for
  the PR HEAD commit (wave-74 precedent: F-W74G-P3-001)

Without row-verification, a review that approves a table with fabricated entries provides
no assurance that the claimed tests exist or pass.

---

## Non-Conformance Consequence

A PR review that does not record row-verification for an in-scope per-test results table:

- **Is incomplete** — the review cannot attest to the accuracy of the test evidence
  presented in the PR description.
- **Leaves fabrication risk undetected** — the wave-74 precedent (F-W74G-P3-001) shows
  this gap surfaces at the adversarial-convergence level, consuming a gate pass to
  correct what a row-verify or count-cross-check step would have caught during review.
- **Is flagged as a process violation** in the wave gate adversarial record if caught
  post-review.

---

## Wave-74 Evidence

| Item | Detail |
|------|--------|
| PR | #397 (STORY-164, wave-74, squash d6e3be8) |
| Table | Aggregate count row ("python 101/101" / "22+10=32 tests pass") + T01–T22 per-test results for `bin/test_validate_citations.py` in `code-delivery/STORY-164/pr-description.md` |
| Gap | Aggregate count claimed before final test suite complete and not matching actual CI run; no per-test row verified against source file |
| Detection | F-W74G-P3-001 (HIGH), wave-74 gate adversarial convergence Pass 3 (W3) |
| Gate-summary | `.factory/cycles/wave-74/wave-gate/gate-summary.md:40` — W3 row describes the aggregate-count mismatch |
| Code-review record | Disposition table at `.factory/cycles/wave-74/wave-gate/code-review.md:104` — no row-verify finding recorded, confirming the gap was not caught at code-review level |

---

## Reference

- **PG-W74-PRDESC-ROW-VERIFY:** Root process-gap (wave-74 gate adversarial convergence
  Pass 3, W3, 2026-07-11). Direct cause of this mandate.
- **F-W74G-P3-001:** Finding (HIGH) — aggregate count row stale/pre-completion and not
  matching actual CI output, wave-74 gate adversarial convergence Pass 3 (W3). Gate-summary:
  `.factory/cycles/wave-74/wave-gate/gate-summary.md:40`.
- **STORY-165 AC-165-002:** Codification story for this mandate.

---

## Correction Record

| Finding | Date | Change |
|---------|------|--------|
| F-S165P1-001 | 2026-07-13 | Mandate section example row: fabricated test name `test_T12_malformed_line_counted_in_denominator` at line 342 replaced with ground-truth name `test_T12_malformed_line_reported` at line 278. Sibling locus in STORY-165.md AC-165-002(b) fixed in same burst per DF-SIBLING-SWEEP-001. |
| F-S165P4-001 | 2026-07-13 | Fabricated finding-ID F-W74P8-001 / "Pass 8" corrected to F-W74G-P3-001 / "gate adversarial convergence Pass 3 (W3)" at all loci (header, Background, Non-Conformance, Wave-74 Evidence table, Reference section). Sibling loci in STORY-165.md fixed in same burst per DF-SIBLING-SWEEP-001. |
| F-S165P4-002 | 2026-07-13 | Wave-74 precedent evidence recharacterized: actual defect in F-W74G-P3-001 was aggregate-count row ("python 101/101" / "22+10=32") computed before final suite completed and not matching actual CI output — not per-test name rows unverified (which is a secondary observation retained). Mandate broadened: Mandate section restructured into two explicit items (per-test row-verify + aggregate-count cross-check). Scope updated to include aggregate counts. Fabrication Risk section amended to name both defect classes. Wave-74 Evidence table updated. |
