---
document_type: story
story_id: STORY-165
epic_id: E-11
version: "1.6"
status: delivered
producer: story-writer
timestamp: 2026-07-13T00:00:00Z
phase: f7
level: feature
cycle: wave-74
points: 3
priority: P3
depends_on: []
blocks: []
# BC status: pending PO authorship
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: bin/
subsystems: []
estimated_days: 1
wave: "75"
traces_to:
  - .factory/stories/STORY-INDEX.md
  - .factory/cycles/wave-74/wave-gate/code-review.md
  - .github/workflows/ci.yml
  - CLAUDE.md
inputs:
  - .factory/stories/STORY-INDEX.md
  - .factory/cycles/wave-74/wave-gate/code-review.md
  - .github/workflows/ci.yml
  - CLAUDE.md
input-hash: "24ff099"
---

# STORY-165: Wave-74 cycle-closing: bin-selftest CI wiring, PR-description row-verify mandate, delivery-doc currency sweep, governance-table audit-first rule

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** delivered
**Wave:** 75
**Points:** 3
**Priority:** P3

## Narrative

- **As a** spec-steward, orchestrator operator, and future contributor on the wirerust project
- **I want** four process improvements codified into durable project artifacts: CI jobs that run
  the delivered `bin/test_validate_citations.py` and `bin/test_changelog_gate_content.py` test
  suites automatically, a pr-reviewer/pr-manager mandate to row-verify per-test results tables
  in PR descriptions, a wave-gate-entry step requiring a full currency sweep of delivery-narrative
  artifacts, and a governance-table audit-first rule requiring a complete cell audit before any
  correction is written
- **So that** wave-74 test files shipped without CI wiring do not remain silently uncovered,
  PR description tables containing fabricated or stale test-function names are caught during
  review, adversarial passes are not burned on documentation staleness that could have been
  caught by a pre-gate sweep, and governance-table remediation does not require multiple passes
  to fix cells that a single audit burst would have caught together

## Behavioral Contracts

_(none -- E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

Wave-74 gate adversarial convergence and code review surfaced four process gaps: a CI coverage
gap for newly delivered test scripts (PG-W74-CI-BIN-SELFTEST), a PR description row-verify gap
for per-test results tables (PG-W74-PRDESC-ROW-VERIFY), a delivery-doc staleness pattern that
burned late adversarial passes (PG-W74-DELIVERY-DOC-CURRENCY), and a multi-pass legend
remediation that a single audit burst would have avoided (PG-W74-GROUND-TRUTH-AUDIT-FIRST).
S-7.02 (cycle-close requirement) mandates codification of wave-execution process gaps as
follow-up stories.

### PG-W74-CI-BIN-SELFTEST — test scripts delivered without CI wiring

STORY-164 (wave-74) delivered two Python test scripts — `bin/test_validate_citations.py` (22
tests) and `bin/test_changelog_gate_content.py` (10 tests) — as part of AC-164-002(d) and
AC-164-003(c). These files passed locally at wave-74 close but were not wired into the CI
pipeline. CI would not have caught regressions in these test scripts (gap closed: PR #398
fa646ed, 2026-07-13).

The wave-73 precedent (STORY-162 AC-162-002) established the correct pattern: when a gate
script has a self-test file in `bin/`, a dedicated CI job runs that self-test on every PR. The
`green-doc-tense-gate` job at `.github/workflows/ci.yml:451` executes
`python3 bin/test_check_green_doc_tense.py` as its first step (`.github/workflows/ci.yml:459-460`),
then runs the gate script itself. The identical pattern must be applied to
`bin/test_validate_citations.py` and `bin/test_changelog_gate_content.py`.

Evidence:
- Code review scope confirmation: `.factory/cycles/wave-74/wave-gate/code-review.md:20` lists
  `bin/test_validate_citations.py` (new, 655 lines, 22 tests) and
  `.factory/cycles/wave-74/wave-gate/code-review.md:21` lists
  `bin/test_changelog_gate_content.py` (new, 279 lines, 10 tests) as files reviewed in PR #397.
  Neither file has a corresponding CI job at wave-74 close.
- The `green-doc-tense-gate` pattern (ci.yml:451): the structural template for wiring a
  `bin/test_*.py` file into CI — one job, one checkout step, one run step.

### PG-W74-PRDESC-ROW-VERIFY — test-evidence tables unverified during review

PR #397 (STORY-164) included a test-evidence table in the delivery doc (`code-delivery/
STORY-164/pr-description.md`). Wave-74 gate adversarial convergence Pass 3 (W3)
(F-W74G-P3-001, HIGH) found that the table's aggregate count row claimed "python 101/101"
across both bin/ test suites, but that count was computed before the final 10-test suite
(`bin/test_changelog_gate_content.py`) was complete; the row also cited a pytest run output
that did not match the actual `bin/test_changelog_gate_content.py` output format. The PR
description also carried a per-test results table listing T01–T22 per-test results for
`bin/test_validate_citations.py`; neither the pr-reviewer nor pr-manager agent cross-checked
any entry in that table against the actual test function names in the source file.

Root cause: no mandate exists requiring agents to cross-check aggregate test counts or
spot-verify per-test results tables in PR descriptions. Code review artifacts
(`.factory/cycles/wave-74/wave-gate/code-review.md:104`) record wave-74 gate findings, but
none of the finding dispositions addresses the PR description row-verify gap — it was surfaced
at the adversarial-convergence level, not the code-review level. The gap must be codified as
an explicit pr-reviewer and pr-manager process obligation.

Evidence:
- Wave-74 PR #397 delivery doc claimed an aggregate count ("python 101/101" / "22+10=32")
  before the final test suite was complete (F-W74G-P3-001). Disposition table at
  `.factory/cycles/wave-74/wave-gate/code-review.md:104` does not include a PR-description
  row-verify requirement — confirming the gap was not caught at code-review level.

### PG-W74-DELIVERY-DOC-CURRENCY — delivery-narrative staleness burned late adversarial passes

Multiple late adversarial passes at the wave-74 gate were consumed fixing delivery-narrative
artifacts that described pre-delivery state as "current" after STORY-164 had already been
merged:

- Pass 1 (F-W74P1-001): STORY-164 status field still read `ready` after PR #397 merged
  (`.factory/stories/STORY-164.md:660`). Corrected status to `delivered` at all loci. Loci
  agreement is a mechanical property verifiable before any adversarial pass begins.
- Pass 13 (F-W74P13-001): Background section described "Current gate implementation" using
  present tense for the pre-STORY-164 changelog-gate behavior (`.factory/stories/STORY-164.md:655`).
  Reframed as historical once STORY-164 was delivered. A pre-gate currency sweep of tense
  and status across delivery-narrative artifacts would have caught both of these without
  consuming adversarial convergence capacity.

The pattern is consistent with the scheduler boundary observation from the wave-74 gate (W5
advisory): present-tense references to implementation state in specs and evidence artifacts
should be reviewed against delivered state before adversarial passes begin, because each
staleness correction reopens the changed section to re-review.

Root cause: no explicit wave-gate-entry step requires a full currency sweep of delivery
documents (story specs, demo evidence, maintenance docs) before the adversarial pipeline
starts.

Evidence:
- STORY-164.md:655 (v1.16 entry, F-W74P13-001): "Current gate implementation" reframed
  historically — a correction that should have been pre-gate.
- STORY-164.md:660 (v1.11 entry, F-W74P1-001): status `ready` → `delivered` — a mechanical
  loci-agreement correction.

### PG-W74-GROUND-TRUTH-AUDIT-FIRST — legend remediated cell-by-cell over three passes

The STORY-INDEX status-vocabulary legend was corrected in three separate gate passes:
- v3.48 (F-W74P3-001, STORY-INDEX.md:17): `superseded` row added — gap in the legend not
  caught by the v3.46 authorship pass.
- v3.49 (F-W74P4-001, STORY-INDEX.md:16): `pending` synonym-note "pre-v3.00" corrected —
  a sibling of the superseded-row change not swept at v3.48.
- v3.50 (F-W74P6-001, STORY-INDEX.md:15): `completed` row Loci and Definition corrected —
  another sibling not swept at v3.48 or v3.49.

Each pass fixed the found cell but failed to audit sibling cells in the same burst, forcing
the next adversarial pass to re-open a table that had already been "fixed." A single
ground-truth audit pass enumerating all seven rows before writing any correction would have
caught all three gaps in one burst.

Root cause: no governance-table amendment rule requires auditing ALL cells before writing.
Authors and agents default to fixing the found cell and moving on, leaving siblings for
subsequent passes.

Evidence:
- STORY-INDEX.md:15 (v3.50 header): F-W74P6-001 `completed` row correction.
- STORY-INDEX.md:16 (v3.49 header): F-W74P4-001 `pending` synonym note correction.
- STORY-INDEX.md:17 (v3.48 header): F-W74P3-001 `superseded` row addition.

## Acceptance Criteria

### AC-165-001 (traces to PG-W74-CI-BIN-SELFTEST — bin selftest CI wiring)

`.github/workflows/ci.yml` gains a new CI job `bin-selftest` that runs
`bin/test_validate_citations.py` (22 tests) and `bin/test_changelog_gate_content.py` (10 tests)
following the structural pattern of the `green-doc-tense-gate` job
(`.github/workflows/ci.yml:451-462`).

(a) **CI job structure:** The new job MUST:
    - Be named `bin-selftest` with a descriptive `name:` label (e.g.,
      "Bin selftest suite (test_validate_citations + test_changelog_gate_content)")
    - Run on `ubuntu-latest`, `timeout-minutes: 5`, `permissions: contents: read`
    - Include a single `actions/checkout` step (SHA-pinned, same pin as other jobs)
    - Include one run step per test file:
      ```yaml
      - name: Run bin/test_validate_citations.py (22 tests)
        run: python3 bin/test_validate_citations.py
      - name: Run bin/test_changelog_gate_content.py (10 tests)
        run: python3 bin/test_changelog_gate_content.py
      ```
    - Preserve all existing SHA-pinned action refs; the action-pin-gate MUST continue to pass.

(b) **Changelog-gate trigger-set adjudication:** `bin/test_validate_citations.py` and
    `bin/test_changelog_gate_content.py` reside in `bin/`, which IS in the changelog-gate
    trigger set (AC-158-001: trigger set = `src/`, `Cargo.toml`, `bin/`). PRs that add or
    modify `bin/test_*.py` files therefore require a CHANGELOG entry — a requirement already
    satisfied by PR #397 (STORY-164 wave-74). The AC-165-001 develop PR modifies ONLY
    `.github/workflows/ci.yml`; `.github/` is excluded from the trigger set, so this PR
    does NOT require a CHANGELOG entry.

Verification:
```bash
grep -n "bin-selftest\|test_validate_citations\|test_changelog_gate_content" \
  .github/workflows/ci.yml
```
must emit non-empty output containing the new job name and both run commands.

### AC-165-002 (traces to PG-W74-PRDESC-ROW-VERIFY — PR description per-test table row-verify mandate)

A new maintenance artifact `.factory/maintenance/pr-description-row-verify-mandate.md`
(factory-artifacts branch) is created that codifies the **PR description per-test table
row-verify mandate** for pr-reviewer and pr-manager agents.

(a) **Scope:** The mandate applies whenever a PR description carries:
    - A **per-test results table** — a markdown table or bulleted list enumerating individual
      test identifiers (e.g., T01–T22, B01–B10) with pass/fail or similar status annotations.
    - Any **claimed aggregate test count or aggregate result** (e.g., "22 passed", "101/101",
      "22 + 10 = 32 tests pass") in a test-evidence section of the PR description.
    Such tables and counts are common in E-11 governance and tooling stories.

(b) **Mandate:** The pr-reviewer and pr-manager agents MUST perform BOTH of the following
    checks where applicable:

    **1. Per-test row-verify (when per-test rows are present):** Row-verify at least three
    randomly-selected entries from any per-test results table in the PR description by:
    1. Locating the test file named in the PR description.
    2. Reading that file to confirm the test function name for each selected row exists at
       the line or location implied by the table.
    3. Recording in the review that row-verification was performed (e.g., "Row-verified T01
       (`test_T01_valid_line_citation_passes`, `bin/test_validate_citations.py:120`), T12
       (`test_T12_malformed_line_reported`, line 278), T22 (`test_T22_unreadable_target_file`,
       line 553)."). A table with fewer than three rows requires verification of all rows.

    **2. Aggregate-count cross-check (when aggregate counts are claimed):** Cross-check every
    claimed aggregate count or aggregate result in the PR description's test-evidence section
    (e.g., "22 passed", "101/101", "22 + 10 = 32") against the actual test-run or CI output
    for the PR HEAD commit, and record the cross-check. A claimed aggregate count that cannot
    be matched to an actual run output is a **blocking review finding**.

(c) **Fabrication risk:** A per-test results table claiming "22 tests PASS" is unverifiable
    without reading the source. Row-verification prevents copy-paste errors,
    auto-generation hallucinations, and count drift from subsequent test additions from
    passing through PR review undetected.

(d) **Wave-74 evidence:** PR #397 (STORY-164) delivery doc (`code-delivery/STORY-164/
    pr-description.md`) claimed "python 101/101" as an aggregate count across both bin/ test
    suites, but that count was computed before the final 10-test suite
    (`bin/test_changelog_gate_content.py`) was complete; the cited pytest run output also did
    not match the actual CI output. The delivery doc also carried a per-test table for T01–T22
    of `bin/test_validate_citations.py`; neither the pr-reviewer nor pr-manager agent
    cross-checked any row against actual test function names in the source file. The gap was
    identified at wave-74 gate adversarial convergence Pass 3 (W3) (F-W74G-P3-001, HIGH).

Verification:
```bash
test -f .factory/maintenance/pr-description-row-verify-mandate.md
grep -n "row-verify\|PG-W74-PRDESC-ROW-VERIFY" \
  .factory/maintenance/pr-description-row-verify-mandate.md
```
Both must succeed: file exists, grep emits non-empty output.

### AC-165-003 (traces to PG-W74-DELIVERY-DOC-CURRENCY — delivery-doc currency sweep)

A new maintenance artifact `.factory/maintenance/delivery-doc-currency-protocol.md`
(factory-artifacts branch) is created that codifies the **delivery-doc currency sweep** as a
mandatory wave-gate-entry step.

(a) **Scope trigger:** This sweep is performed once per wave, before the first adversarial
    pass of the wave gate begins (per-story Step-4.5 passes out of scope). It applies to all delivery-narrative artifacts associated with
    the wave's stories: story spec files (`.factory/stories/STORY-NNN.md`), demo-evidence
    artifacts (`.factory/demo-evidence/`), and maintenance docs created or amended by the
    wave's stories.

(b) **Mandatory sweep steps:** Before the first adversarial pass of a wave gate, the operator
    MUST:
    1. **Status loci check:** verify that frontmatter `status:`, body header `**Status:**`, and
       STORY-INDEX index cell agree and reflect the current delivery state (draft/ready/pending/
       delivered/superseded) for every story assigned to the wave (AC-164-001(c) loci agreement
       rule).
    2. **Tense audit:** scan story Background and Acceptance Criteria sections for present-tense
       references to implementation behavior that describe the pre-delivery state as "current"
       after delivery — phrases like "Current gate implementation", "The gate currently", or
       inline bash/code blocks copied from pre-delivery state that AC changes have superseded.
       Reframe any such references as historical (e.g., "Pre-STORY-NNN implementation
       (as of develop COMMITSHA / vX.Y.Z): ...").
    3. **Demo-evidence currency notes:** review demo-evidence artifacts for any counts, code
       excerpts, or behavioral claims that have been superseded by the wave's delivery.
       Add currency notes per the pattern established in
       `.factory/demo-evidence/story-164/AC-164-001.md`.

(c) **Currency sweep record:** The sweep completion MUST be recorded before the first pass.
    A one-line `**Currency sweep: COMPLETE (YYYY-MM-DD)**` note is sufficient. Omitting the
    sweep record is non-conforming; the first adversarial pass of the wave gate MUST verify that the record
    exists.

(d) **Why pre-gate:** Stale delivery-narrative text forces late adversarial passes to re-open
    sections already reviewed once the staleness is found, consuming convergence capacity on
    mechanical corrections rather than behavioral review. The wave-74 gate burned Pass 1
    (F-W74P1-001, status mismatch) and Pass 13 (F-W74P13-001, stale "Current" tense) on
    corrections that a pre-gate sweep would have caught.

Verification:
```bash
test -f .factory/maintenance/delivery-doc-currency-protocol.md
grep -n "currency sweep\|PG-W74-DELIVERY-DOC-CURRENCY\|tense audit" \
  .factory/maintenance/delivery-doc-currency-protocol.md
```
Both must succeed: file exists, grep emits non-empty output.

### AC-165-004 (traces to PG-W74-GROUND-TRUTH-AUDIT-FIRST — governance-table audit-first rule)

STORY-INDEX gains a **Governance-Table Amendment Protocol** note placed immediately after the
Loci Agreement Rule within the Status Vocabulary section. The note MUST:

(a) State the **audit-first rule**: when remediating any governance table (including but not
    limited to the Status Vocabulary legend, the Wave Delivery Progress table, and the Stories
    by Epic table), the remediation agent MUST:
    1. Read the full table in one burst.
    2. Identify ALL cells with potential errors against ground truth.
    3. Record every suspected error before writing any correction.
    4. Apply all corrections in a single edit pass.

    Correcting one cell and submitting before auditing siblings is non-conforming. A single
    audit burst followed by a single correction burst is the mandatory sequence.

(b) State the **evidence rationale**: the wave-74 gate fixed the Status Vocabulary legend
    in three separate passes (v3.48, v3.49, v3.50 — F-W74P3-001, F-W74P4-001, F-W74P6-001)
    because each pass fixed one cell without auditing siblings. Three adversarial passes were
    consumed; a single audit burst would have required one.

Verification:
```bash
grep -n "Governance-Table Amendment Protocol\|audit-first\|audit ALL cells" \
  .factory/stories/STORY-INDEX.md
```
must emit non-empty output containing the new protocol note.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| bin-selftest CI job | `.github/workflows/ci.yml` (amend) | CI configuration |
| PR description row-verify mandate | `.factory/maintenance/pr-description-row-verify-mandate.md` (new) | Documentation |
| Delivery-doc currency protocol | `.factory/maintenance/delivery-doc-currency-protocol.md` (new) | Documentation |
| Governance-table audit-first note | `.factory/stories/STORY-INDEX.md` (amend) | Documentation |

No Rust source files in `src/`, no test files in `tests/`, no `Cargo.toml` changes.
No new Python or bash files in `bin/`.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `.github/workflows/ci.yml` | CI configuration | Runs in CI sandbox; delegates to existing Python test scripts |
| `pr-description-row-verify-mandate.md` | Documentation artifact | Governance prose |
| `delivery-doc-currency-protocol.md` | Documentation artifact | Governance prose |
| `STORY-INDEX.md` | Documentation artifact | Governance prose + index table |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PR description table has fewer than three rows | Row-verify ALL rows (AC-165-002(b) minimum floors to the actual row count when < 3) |
| EC-002 | A new `bin/test_*.py` file is added in a future PR | That PR MUST include a CHANGELOG entry (bin/ is in the trigger set per AC-165-001(b) adjudication) AND a CI wiring amendment to add the new test to the `bin-selftest` job |
| EC-003 | `bin-selftest` CI job runs on a PR that does not touch `bin/test_*.py` | The job still runs (it runs unconditionally on all PRs, like `green-doc-tense-gate`), exercising the tests on every change to ensure the suite remains green |
| EC-004 | Delivery-doc currency sweep finds zero stale items | Still MUST record `**Currency sweep: COMPLETE**` before the first adversarial pass of the wave gate (AC-165-003(c)) |
| EC-005 | Governance table has only one row (or all rows have been freshly audited) | The audit-first rule still applies; the audit concludes with no corrections needed, and a single no-op edit pass is acceptable |

## Tasks

1. **Amend .github/workflows/ci.yml (AC-165-001):** Add the `bin-selftest` CI job following
   the `green-doc-tense-gate` structural pattern (ci.yml:451-462). The job runs
   `python3 bin/test_validate_citations.py` and `python3 bin/test_changelog_gate_content.py`
   in sequence. Use the same SHA-pinned `actions/checkout` ref as adjacent jobs; do NOT
   introduce a new unpinned action ref. Verify the action-pin-gate passes after the amendment.

2. **Open develop PR for .github/workflows/ci.yml and CLAUDE.md (AC-165-001, F-S165P4-003):**
   Create a PR targeting `develop` that includes both:
   - `.github/workflows/ci.yml` — `bin-selftest` CI job (AC-165-001).
   - `CLAUDE.md` — add Project References table rows for the two new maintenance docs:
     `.factory/maintenance/pr-description-row-verify-mandate.md` and
     `.factory/maintenance/delivery-doc-currency-protocol.md`, following the pattern of
     existing entries in the Project References table (path | purpose).
   No CHANGELOG.md entry is required: `.github/` is excluded from the trigger set per
   AC-158-001; `CLAUDE.md` is also not in the trigger set (trigger set = `src/`, `Cargo.toml`,
   `bin/`); adjudication per AC-165-001(b).

3. **Create pr-description-row-verify-mandate.md (AC-165-002):** Write
   `.factory/maintenance/pr-description-row-verify-mandate.md` on the factory-artifacts
   branch. The document MUST cover: scope (per-test results tables in PR descriptions),
   the mandate (row-verify ≥3 entries against actual test function names), fabrication risk,
   and wave-74 evidence (F-W74G-P3-001). Follow the structural pattern of
   `.factory/maintenance/breaking-change-delivery-protocol.md` (Policy reference header +
   Finding reference + Background + Scope + Mandate + Non-Conformance Consequence + Evidence).

4. **Create delivery-doc-currency-protocol.md (AC-165-003):** Write
   `.factory/maintenance/delivery-doc-currency-protocol.md` on the factory-artifacts
   branch. The document MUST cover: scope trigger (once per wave, before first adversarial
   pass of the wave gate), mandatory steps (status loci check, tense audit, demo-evidence currency notes),
   currency sweep record requirement, and wave-74 evidence (F-W74P1-001, F-W74P13-001).

5. **Amend STORY-INDEX (AC-165-004):** Add the Governance-Table Amendment Protocol note
   immediately after the Loci Agreement Rule (`.factory/stories/STORY-INDEX.md:149`),
   before the horizontal rule that follows. Include the audit-first rule and wave-74
   evidence rationale per AC-165-004.

> **Note for implementer:** The develop PR (Task 2) is required for AC-165-001 (ci.yml).
> AC-165-002 (pr-description-row-verify-mandate.md), AC-165-003
> (delivery-doc-currency-protocol.md), and AC-165-004 (STORY-INDEX amendment) are
> factory-artifacts branch commits only. Both tracks must complete for the story to be
> declared delivered.

## Previous Story Intelligence

Lessons from analogous governance/tooling stories in E-11 — especially STORY-162, STORY-163,
and STORY-164, which immediately precede this story:

- **STORY-164 (wave-74, E-11, 4 pts) — self-referential quality discipline:** STORY-164
  delivered `bin/validate-citations` and its test suite, but did not wire the test suite
  into CI. AC-165-001 closes this gap using the same green-doc-tense-gate pattern that
  STORY-162 established (AC-162-002). The lesson: every gate script or tool delivered in
  `bin/` should have a corresponding CI self-test job wired at delivery time, not at the
  next wave's cycle-close.

- **STORY-163 (wave-73, E-11, 2 pts) — meta-irony precedent:** STORY-163 codified the
  citation mandate, yet its own evidence contained fabricated citations — caught by the
  adversary at CRITICAL severity. The wave-74 analog (F-W74G-P3-001 for STORY-165) is
  softer: a PR description delivery doc claimed a stale aggregate test count ("python 101/101")
  that was pre-completion and did not match actual CI output; per-test row verification was
  also absent. In both cases the gap is that a self-referential quality property was not
  applied to the story's own artifacts.

- **STORY-162 (wave-72, E-11, 3 pts):** Introduced the `green-doc-tense-gate` CI job
  pattern (AC-162-002). AC-165-001 follows the exact same structural template. This is the
  third iteration of the pattern (check-green-doc-tense → validate-citations +
  changelog-gate-content). The pattern is now established; future test scripts in `bin/`
  should be wired in their delivery PR, not deferred to the next S-7.02.

- **STORY-164 (wave-74, E-11, 4 pts) — delivery-narrative staleness pattern:** The
  changelog records F-W74P1-001 (Pass 1: status mismatch) and F-W74P13-001 (Pass 13:
  stale tense). This 12-pass gap between a mechanical status correction and a tense
  correction illustrates that staleness comes in layers; a pre-gate currency sweep catches
  all layers at once rather than requiring a separate adversarial pass for each.

## Architecture Compliance Rules

- This story modifies ONLY the files listed in File Structure Requirements below.
- The `bin-selftest` CI job MUST NOT introduce any new action pin refs. The
  `actions/checkout` step MUST use the same SHA already present in adjacent jobs.
- All SHA-pinned action refs in `.github/workflows/ci.yml` MUST remain SHA-pinned after
  the amendment. The action-pin-gate (`action-pin-gate` CI job) must continue to pass.
  The `dtolnay/rust-toolchain@stable` exemption must not be disturbed.
- No production Rust source (`src/`), no test files (`tests/`), no `Cargo.toml`, no
  new Python or bash files in `bin/`.
- STORY-INDEX.md, `pr-description-row-verify-mandate.md`, and `delivery-doc-currency-
  protocol.md` are factory-artifacts files; the develop PR must NOT include them.

## Library & Framework Requirements

- No code dependencies. No Python changes. No Bash changes. No new Rust crates.
- No Rust toolchain changes. No Cargo.toml changes.
- CI uses existing Python 3 interpreter already available on `ubuntu-latest`.

## File Structure Requirements

| File | Action | Branch | Notes |
|------|--------|--------|-------|
| `.github/workflows/ci.yml` | Modify | develop | Add `bin-selftest` CI job (AC-165-001) |
| `CLAUDE.md` | Modify | develop | Project References rows for the two new maintenance docs (F-S165P4-003, sibling-registration pattern) |
| `.factory/maintenance/pr-description-row-verify-mandate.md` | Create | factory-artifacts | PR description per-test table row-verify mandate (AC-165-002; PG-W74-PRDESC-ROW-VERIFY) |
| `.factory/maintenance/delivery-doc-currency-protocol.md` | Create | factory-artifacts | Delivery-doc currency sweep protocol (AC-165-003; PG-W74-DELIVERY-DOC-CURRENCY) |
| `.factory/stories/STORY-INDEX.md` | Modify | factory-artifacts | Governance-Table Amendment Protocol note after Loci Agreement Rule (AC-165-004) |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~2.5 k |
| `.github/workflows/ci.yml` green-doc-tense-gate block (~12 lines) | ~0.1 k |
| `breaking-change-delivery-protocol.md` (structural pattern reference) | ~0.5 k |
| STORY-INDEX Loci Agreement Rule section (insert point) | ~0.1 k |
| **Total** | **~3.2 k** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** All four process gaps are DF-VALIDATION-001-exempt. All
  originate from wave-74 in-process execution findings: F-W74G-P3-001 (gate adversarial
  convergence Pass 3, W3), F-W74P1-001 (gate Pass 1), F-W74P13-001 (gate Pass 13), and
  F-W74P3-001/P4-001/P6-001 (gate Passes 3, 4, 6 — all legend remediation). All are
  in-process execution findings —
  DF-VALIDATION-001-exempt per the in-process exemption (same pattern as STORY-164 Notes,
  STORY-163 Notes, STORY-162 Notes).
- **S-7.02 disposition:** Creating this story at draft status codifies four wave-74 process-gap
  findings (PG-W74-CI-BIN-SELFTEST, PG-W74-PRDESC-ROW-VERIFY, PG-W74-DELIVERY-DOC-CURRENCY,
  PG-W74-GROUND-TRUTH-AUDIT-FIRST) for the S-7.02 cycle-close obligation.
- **No behavioral contract required:** E-11 convention (epics.md E-11: "BCs: none
  authored yet -- status: draft; pending PO authorship").
- **Develop/factory split:** AC-165-001 (.github/workflows/ci.yml) touches the develop tree
  and requires a PR. The develop PR also includes CLAUDE.md Project References rows for the
  two new maintenance docs (F-S165P4-003, sibling-registration pattern). CLAUDE.md is not in
  the changelog trigger set (AC-158-001: trigger set = `src/`, `Cargo.toml`, `bin/`), so no
  CHANGELOG entry is required for the CLAUDE.md change. AC-165-002 and AC-165-003 (new
  maintenance docs) and AC-165-004 (STORY-INDEX amendment) are factory-artifacts branch
  commits only.
- **No CHANGELOG obligation for AC-165-001 develop PR:** The PR touches ONLY
  `.github/workflows/ci.yml`; `.github/` is explicitly excluded from the changelog-gate
  trigger set (AC-158-001). No CHANGELOG.md entry is required. This is distinct from
  PRs adding `bin/test_*.py` files (which ARE in the trigger set) — adjudication per
  AC-165-001(b).
- **W5 scheduler vocabulary advisory (folded):** Wave-74 gate Pass 5 noted a scheduler
  vocabulary boundary observation (the distinction between "scheduler" and "orchestrator"
  terminology in wave-gate entry docs). This is advisory context for AC-165-003: the
  delivery-doc currency sweep protocol should cover wave-gate-entry documentation alongside
  story spec tense, as these documents are similarly subject to stale terminology.
- **bin-selftest dogfood:** This story's first mandated use of `bin/validate-citations`
  (AC-164-002, STORY-164) was to validate the anchor list used in this story file's
  Background and Acceptance Criteria sections before writing. The anchor list consisted of
  11 entries; `bin/validate-citations` reported `PASS: 11 citations verified`.
- **Precedent chain:** STORY-165 follows the E-11 S-7.02 pattern: STORY-157 → wave-70;
  STORY-158 → wave-71; STORY-162 → wave-72; STORY-163 → maint-2026-07-09;
  STORY-164 → wave-73; STORY-165 → wave-74.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.6 | 2026-07-13 | story-writer | STORY-165 DELIVERED — develop track PR #398 squash-merged (fa646ed, 2026-07-13; 13/13 CI green incl. bin-selftest first run; pr-reviewer APPROVE 0 findings, 9 rows row-verified + counts cross-checked per PG-W74-PRDESC-ROW-VERIFY first compliant execution); factory track AC-165-002/003/004 complete. Status ready→delivered at all loci. |
| 1.5 | 2026-07-13 | story-writer | F-S165P6-001 (wave-75 Pass 6, MEDIUM): currency-sweep trigger disambiguated — wave-gate-entry only; per-story Step-4.5 passes explicitly out of scope. AC-165-003(a) + protocol doc fixed at all loci in one burst. |
| 1.4 | 2026-07-13 | story-writer | Wave-75 Pass-4 remediation (human-ratified): F-S165P4-001 fabricated finding-ID F-W74P8-001/Pass-8 corrected to F-W74G-P3-001/Pass-3(W3) at all loci; F-S165P4-002 wave-74 evidence recharacterized (aggregate count vs CI output) + AC-165-002(b) mandate broadened with aggregate-count cross-check clause; F-S165P4-003 CLAUDE.md registration rows added to develop track (File Structure + Tasks). |
| 1.3 | 2026-07-13 | story-writer | F-S165P1-001 (wave-75 Pass 1, HIGH): fabricated test name test_T12_malformed_line_counted_in_denominator corrected to test_T12_malformed_line_reported (line 278) in AC-165-002(b) example; sibling locus in pr-description-row-verify-mandate.md fixed in same burst per DF-SIBLING-SWEEP-001. |
| 1.2 | 2026-07-13 | story-writer | Line-citation refresh after STORY-INDEX v3.53 prepend (AC-165-004 delivery); no content change. |
| 1.1 | 2026-07-13 | story-writer | Wave-75 assignment (plan gate approved, human, 2026-07-13): status draft→ready, wave TBD→75; fixed 2 consistency-audit MINORs (stale STORY-INDEX line citations in Background evidence and Task 5 insert-point hint) per wave-75 opening audit. |
| 1.0 | 2026-07-11 | story-writer | Initial authorship — wave-74 process-gap codifications: PG-W74-CI-BIN-SELFTEST (AC-165-001 bin-selftest CI wiring), PG-W74-PRDESC-ROW-VERIFY (AC-165-002 PR description row-verify mandate), PG-W74-DELIVERY-DOC-CURRENCY (AC-165-003 delivery-doc currency sweep protocol), PG-W74-GROUND-TRUTH-AUDIT-FIRST (AC-165-004 governance-table audit-first rule). S-7.02 wave-74 cycle-close. bin/validate-citations dogfood: PASS on 11-entry anchor list. |
