---
document_type: story
story_id: STORY-164
epic_id: E-11
version: "1.3"
status: ready
producer: story-writer
timestamp: 2026-07-11T00:00:00Z
phase: f7
level: feature
cycle: wave-73
points: 4
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
wave: "74"
traces_to:
  - .factory/stories/STORY-INDEX.md
  - .factory/cycles/wave-73/STORY-163/adversary-convergence-state.json
  - .factory/cycles/wave-73/STORY-162/adversary-convergence-state.json
  - .github/workflows/ci.yml
  - CLAUDE.md
  - .factory/maintenance/docs-writer-dispatch-guidance.md
inputs:
  - .factory/stories/STORY-INDEX.md
  - .factory/cycles/wave-73/STORY-163/adversary-convergence-state.json
  - .factory/cycles/wave-73/STORY-162/adversary-convergence-state.json
  - .github/workflows/ci.yml
  - CLAUDE.md
  - .factory/maintenance/docs-writer-dispatch-guidance.md
input-hash: "8bfa01d"
---

# STORY-164: Wave-73 cycle-closing: status-vocabulary legend, citation preflight validator,
changelog-gate content assertion, guidance-doc reference row

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** ready
**Wave:** 74
**Points:** 4
**Priority:** P3

## Narrative

- **As a** spec-steward, orchestrator operator, and future contributor on the wirerust project
- **I want** four wave-73 process gaps codified into durable project artifacts: a canonical
  status-vocabulary legend in STORY-INDEX, a mechanical citation preflight validator in
  `bin/`, a content assertion added to the changelog-gate CI job, and a CLAUDE.md Project
  References row for the docs-writer dispatch guidance
- **So that** future contributors have an authoritative definition of story-status vocabulary,
  citation fabrication is caught mechanically before dispatch rather than by the adversary
  at CRITICAL severity, the changelog-gate cannot be silently satisfied by a whitespace-
  only touch to CHANGELOG.md, and the docs-writer dispatch guidance is discoverable from
  CLAUDE.md alongside the existing pr-manager guidance peer

## Behavioral Contracts

_(none -- E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

Wave-73 gate adversarial convergence surfaced four process gaps via findings F-W73G-P3-001
(status-vocabulary ambiguity), F-S163P1-001 CRITICAL (fabricated citations in the citation-
mandate story's own evidence), and PG-W73-CHANGELOG-GATE-CONTENT / PG-W73-CITATION-VALIDATOR
(carried findings from the STORY-162 and STORY-163 per-story convergences). A wave-73
consistency audit advisory additionally identified that `docs-writer-dispatch-guidance.md`
(created in STORY-163 AC-163-001) was not yet registered in CLAUDE.md's Project References
table, making it invisible to agents and contributors who consult that table as a navigation
index. S-7.02 (cycle-close requirement) mandates codification of wave-execution process gaps
as follow-up stories.

### PG-W73-STATUS-VOCAB — status-vocabulary ambiguity in STORY-INDEX

During the wave-73 gate adversarial convergence Pass 3 (finding F-W73G-P3-001), a full-corpus
sweep of all 116 story files revealed 38 stories with statuses in their story files that
did not match the STORY-INDEX index cell. Root cause: STORY-INDEX has no canonical
status-vocabulary legend defining what each status term means, which loci each appears in
(frontmatter, body header, index cell), and which loci must agree. Delivery-class synonym
rows (e.g., `delivered` vs `completed` vs `merged`) were left unresolved for S-7.02
adjudication.

Evidence:
- STORY-INDEX v3.42 comment: "F-W73G-P3-001 — STORY-158/159 status sync draft→delivered;
  FULL-corpus status-coherence sweep (116 stories) executed mechanically; 38 stale story-
  file statuses synced to index record; delivery-class synonym rows (0) left for
  PG-W73-STATUS-VOCAB adjudication (S-7.02)." (`.factory/stories/STORY-INDEX.md`, v3.42
  header comment)

### PG-W73-CITATION-VALIDATOR — fabricated citations in citation-mandate story's own evidence

During STORY-163 adversarial convergence Pass 1 (finding F-S163P1-001, severity CRITICAL),
the adversary found that `authoring-evidence.md` — the evidence artifact for the citation-
mandate story (AC-163-001 ground-truth citation mandate) — cited `.factory/code-delivery/
maint-2026-07-09/pr-review.md:332-333` as anchor locations, but the file is only 111 lines.
The citations were fabricated. This is a meta-failure: the story whose purpose is to prevent citation
fabrication itself contained fabricated citations.

Root cause: there was no mechanical preflight tool to validate that cited file:line
references exist before the evidence artifact was committed. The adversary caught the
fabrication at CRITICAL severity; without adversarial review, these phantom anchors would
have been shipped as authoritative evidence. The adversary P1 process-gap observation
explicitly recommended `bin/validate-citations` for the S-7.02 wave-close.

Evidence:
- Adversary convergence state: `.factory/cycles/wave-73/STORY-163/adversary-convergence-state.json`
  P1 notes: "F-S163P1-001 CRITICAL: fabricated citations — authoring-evidence.md anchor
  table cited non-existent line numbers in pr-manager-merge-auth-guidance.md; all anchor
  refs corrected to actual file:line values... A mechanical preflight validator
  (bin/validate-citations) would have caught this before dispatch. Recommended for S-7.02."
- Carried finding PG-W73-CITATION-VALIDATOR: same file, `carried_findings` array.

### PG-W73-CHANGELOG-GATE-CONTENT — changelog-gate is presence-only

During STORY-162 adversarial convergence Pass 5 (carried finding PG-W73-CHANGELOG-GATE-
CONTENT), the adversary noted that the changelog-gate CI job (AC-158-001) only checks
whether `CHANGELOG.md` appears anywhere in the PR diff — not whether the `[Unreleased]`
section actually gained at least one non-blank content line. A PR author could satisfy the
gate by touching a whitespace character anywhere in CHANGELOG.md, or adding only a blank
line, and the gate would pass identically to a PR with a real entry.

Evidence:
- Adversary convergence state: `.factory/cycles/wave-73/STORY-162/adversary-convergence-state.json`
  P5 notes: "process-gap noted (PG-W73-CHANGELOG-GATE-CONTENT: changelog-gate is presence-only,
  no content assertion — pre-existing gate weakness, not introduced by this story)."
- Carried finding PG-W73-CHANGELOG-GATE-CONTENT: same file, `carried_findings` array.
- Current gate implementation: `.github/workflows/ci.yml` lines 506-509: the check is
  `if echo "${CHANGED}" | grep -q '^CHANGELOG\.md$'; then ... exit 0` — presence-only with
  no content quality assertion.

### Wave-73 Consistency Audit — docs-writer dispatch guidance missing from CLAUDE.md

A wave-73 consistency audit advisory identified that `.factory/maintenance/docs-writer-
dispatch-guidance.md` (created in STORY-163 AC-163-001, codifying PG-RA-P3-ARP-REC006-
INVERSION-001) was not registered in CLAUDE.md's Project References table. CLAUDE.md
already has a peer row for `.factory/maintenance/pr-manager-merge-auth-guidance.md`
(DF-MERGE-AUTH-CLASSIFIER-001). The guidance file is invisible to agents and contributors
who use the Project References table as a navigation index, defeating the discoverability
purpose of the table.

Evidence:
- `CLAUDE.md` Project References table: lists `pr-manager-merge-auth-guidance.md` but
  not `docs-writer-dispatch-guidance.md`.

## Acceptance Criteria

### AC-164-001 (traces to PG-W73-STATUS-VOCAB — STORY-INDEX status-vocabulary legend)

STORY-INDEX gains a **status-vocabulary legend** — a canonical table defining each story
status term with precise semantics and loci. The legend MUST:

(a) Define all six recognized status values with precise semantics:

| Status | Definition | Loci |
|--------|------------|------|
| `draft` | Story created; not yet dispatched for implementation | Frontmatter `status:`, body header, index cell |
| `ready` | Spec-first gate (S-7.01) passed; story may be dispatched | Frontmatter `status:`, body header, index cell |
| `pending` | Story dispatched; implementation in progress or blocked on predecessor | Frontmatter `status:`, body header, index cell |
| `delivered` | PR merged to develop; story on develop but wave not yet closed | Frontmatter `status:`, body header, index cell |
| `merged` | Story squash-merged to develop via tagged PR; semantically equivalent to `delivered` | Frontmatter `status:`, body header, index cell |
| `completed` | Equivalent to `delivered`/`merged`; used in early-wave entries for delivery + closed wave | Index cell only (legacy phrasing); frontmatter prefers `delivered` or `merged` |

(b) Add a **Synonym note**: `delivered`, `merged`, and `completed` are delivery-class synonyms
    — they all mean "PR merged to develop." The canonical term for new stories is `delivered`
    (frontmatter) or `completed` (index cell for early waves pre-v3.00); stories already
    using `merged` need not be updated. Tooling that reads story status MUST treat all three
    as equivalent delivery-class values.

(c) State the **loci agreement rule**: frontmatter `status:`, the body status line (e.g.,
    `**Status:** delivered`), and the STORY-INDEX index cell MUST agree on the delivery-class
    category (draft/ready/pending/delivered-class/superseded). Wave-gate adversarial passes
    are authorized to correct loci mismatches as administrative fixes.

(d) Be placed immediately after the `## Index Table` heading and before the table itself,
    so it is encountered before any status cell is read.

Verification:
```bash
grep -n "Status.*Vocabulary\|status-vocabulary\|delivered.*merged.*completed\|Synonym note\|loci agreement" \
  .factory/stories/STORY-INDEX.md
```
must emit non-empty output containing the legend text.

### AC-164-002 (traces to PG-W73-CITATION-VALIDATOR — mechanical citation preflight validator)

A new tool `bin/validate-citations` (Python 3, stdlib only, Python 3.10+ type syntax,
following the `bin/compute-input-hash` structural pattern) is created that mechanically
validates a citations table: for each `file:line-range` anchor, the tool verifies the
cited file exists and the cited line numbers are within the file's actual line count.

(a) **Input format:** the tool reads a citations table from stdin or a file argument.
    Each non-blank, non-comment line has the form:
    ```
    path/to/file.md:LINE          # optional comment
    path/to/file.md:LINE-LINE     # line range
    ```
    Lines beginning with `#` are comments and are ignored. Blank lines are ignored.
    Paths are relative to the repo root (resolved the same way as `compute-input-hash`).

(b) **Validation:** For each entry:
    - The cited file must exist (FAIL otherwise with "FILE NOT FOUND: path").
    - The cited line number (or both endpoints of a range) must be ≤ the file's actual
      line count (FAIL otherwise with "LINE OUT OF RANGE: path:N (file has M lines)").

(c) **Output:** On success (all citations valid), print `PASS: N citations verified` and
    exit 0. On failure, print each failing entry with its failure reason, a summary
    `FAIL: K of N citations invalid`, and exit 1.

(d) **Self-test:** A corresponding `bin/test_validate_citations.py` (Python 3 stdlib
    unittest) is created covering:
    - Valid file:line citation passes
    - Valid file:line-range citation passes
    - Nonexistent file is rejected with FILE NOT FOUND
    - Out-of-range single line is rejected with LINE OUT OF RANGE
    - Out-of-range range endpoint is rejected
    - Comment lines and blank lines are ignored
    - Empty input (no citations) produces PASS: 0 citations verified

(e) **Wired into docs-writer-dispatch-guidance.md §4:** Section 4 of
    `.factory/maintenance/docs-writer-dispatch-guidance.md` ("Verification Template for
    Orchestrator Dispatches") gains a step: "Before submitting the claims-citation table
    for review, run `bin/validate-citations` on the anchor list. Any FAIL result means a
    cited file or line range does not exist — the anchor MUST be corrected before
    proceeding." This makes the validator a mandatory preflight step in the dispatch
    workflow.

Verification:
```bash
test -f bin/validate-citations && test -f bin/test_validate_citations.py
python3 bin/test_validate_citations.py
grep -n "validate-citations\|preflight" .factory/maintenance/docs-writer-dispatch-guidance.md
```
All three must succeed (file existence, tests passing, guidance doc updated).

### AC-164-003 (traces to PG-W73-CHANGELOG-GATE-CONTENT — changelog-gate content assertion)

The `changelog-gate` CI job in `.github/workflows/ci.yml` gains a **content assertion**
step that verifies the `[Unreleased]` section gained at least one non-blank, non-header
content line between `origin/develop` and `HEAD`, not merely that `CHANGELOG.md` appears
in the diff.

(a) The content assertion is added immediately after the existing presence check (line 506)
    within the same `run:` block: when `CHANGELOG.md` is in the diff and the trigger set was
    hit, the gate also runs:
    ```bash
    CHANGELOG_DIFF=$(git diff origin/develop...HEAD -- CHANGELOG.md)
    CONTENT_LINES=$(echo "${CHANGELOG_DIFF}" | \
      grep '^+' | \
      grep -v '^+++' | \
      grep -v '^+[[:space:]]*$' | \
      grep -v '^+##' | \
      wc -l | tr -d ' ')
    if [ "${CONTENT_LINES}" -eq 0 ]; then
      echo "FAIL: CHANGELOG.md touched but no content added to [Unreleased] section."
      echo "(A whitespace-only touch does not satisfy AC-158-001 / PG-W71-CHANGELOG.)"
      exit 1
    fi
    echo "PASS: CHANGELOG.md updated with ${CONTENT_LINES} content line(s)."
    ```

(b) The check MUST be **reliably green** per the no-flaky-stub policy: it passes if and
    only if at least one non-blank, non-section-header (`##`) added line exists in the
    CHANGELOG.md diff. A real entry (bullet point, prose sentence, or version line) will
    always satisfy this; a whitespace-only touch will always fail. No external state is
    required; the check is deterministic given the diff content.

(c) The existing CHANGELOG obligation comment and `echo` messages are preserved. The only
    change is the addition of the content assertion block after the presence check.

Verification:
```bash
grep -n "CONTENT_LINES\|CHANGELOG_DIFF\|whitespace-only\|content line" .github/workflows/ci.yml
```
must emit non-empty output containing the content assertion.

### AC-164-004 (CLAUDE.md reference row — docs-writer dispatch guidance discoverability)

CLAUDE.md's **Project References** table gains a new row for
`.factory/maintenance/docs-writer-dispatch-guidance.md`, placed immediately after the
existing `pr-manager-merge-auth-guidance.md` row (the two are peer governance docs and
should be adjacent):

```
| `.factory/maintenance/docs-writer-dispatch-guidance.md` | Docs-writer dispatch citation mandate (PG-RA-P3-ARP-REC006-INVERSION-001; `bin/validate-citations` preflight required) |
```

Verification:
```bash
grep -n "docs-writer-dispatch-guidance" CLAUDE.md
```
must emit non-empty output with the new row.

### AC-164-005 (traces to PG-W72-BREAKING-HOLDOUT-SWEEP — holdout-expectation sweep obligation for BREAKING-change stories)

A new maintenance artifact `.factory/maintenance/breaking-change-delivery-protocol.md`
(factory-artifacts branch) is created that codifies the **holdout-expectation sweep
obligation** as a mandatory delivery gate for any story that changes observable output
format.

(a) **Scope trigger:** The sweep obligation applies to any story satisfying at least one:
    - Tagged `BREAKING` in its frontmatter, title, or CHANGELOG entry
    - Changes observable JSON output schema (field names, types, enum values, or enum
      casing — e.g., PascalCase → lowercase or snake_case)
    - Changes observable text output layout (column ordering, header format, separator
      characters, or field labels)

(b) **Mandatory delivery gate:** Before the PR is opened, the implementer MUST:
    1. Run the holdout evaluator against the story's output changes.
    2. Identify all stale holdout-scenario expectations in `.factory/holdout-scenarios/`
       (any scenario whose `expected_output` references the old enum names, JSON schema,
       or text layout).
    3. Repair all stale expectations to match the new output format.
    4. Record `holdout-expectations-sweep: COMPLETE` in the story's delivery checklist
       (Tasks section). A PR opened for an in-scope story without this item completed
       is **non-conforming** per this protocol (PG-W72-BREAKING-HOLDOUT-SWEEP).

(c) **Wave-72 evidence:** STORY-160 introduced a BREAKING JSON change (enum casing:
    PascalCase → lowercase/snake_case + `schema_version` envelope). Thirteen holdout
    scenarios were found stale at the wave-72 integration gate:
    HS-021/024/032/033/034/035/050/054/059/064/065/074/075. These were repaired by the
    product-owner at gate time rather than during story delivery — a significant
    unplanned gate-time work item that this protocol prevents in future waves.
    Source: `.factory/cycles/wave-72/lessons.md` Lesson 2
    (tag PG-W72-BREAKING-HOLDOUT-SWEEP).

(d) **CLAUDE.md reference row:** CLAUDE.md's Project References table gains a new row
    for `.factory/maintenance/breaking-change-delivery-protocol.md`, placed after the
    `docs-writer-dispatch-guidance.md` row added by AC-164-004:

    ```
    | `.factory/maintenance/breaking-change-delivery-protocol.md` | BREAKING-change holdout-sweep obligation (PG-W72-BREAKING-HOLDOUT-SWEEP; `holdout-expectations-sweep: COMPLETE` required before PR for BREAKING or output-format-change stories) |
    ```

Verification:
```bash
test -f .factory/maintenance/breaking-change-delivery-protocol.md
grep -n "holdout-expectations-sweep\|PG-W72-BREAKING-HOLDOUT-SWEEP" \
  .factory/maintenance/breaking-change-delivery-protocol.md
grep -n "breaking-change-delivery-protocol" CLAUDE.md
```
All three must succeed: file exists, grep emits non-empty output from the protocol
document, and the CLAUDE.md row is present.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| STORY-INDEX status-vocabulary legend | `.factory/stories/STORY-INDEX.md` (amend) | Documentation |
| Citation preflight validator | `bin/validate-citations` (new) | Effectful (filesystem reads, stdout) |
| Citation preflight validator self-test | `bin/test_validate_citations.py` (new) | Pure (test harness) |
| Docs-writer dispatch guidance §4 step | `.factory/maintenance/docs-writer-dispatch-guidance.md` (amend) | Documentation |
| Changelog-gate content assertion | `.github/workflows/ci.yml` (amend) | CI configuration |
| CLAUDE.md Project References rows (AC-164-004 + AC-164-005) | `CLAUDE.md` (amend) | Documentation |
| BREAKING-change holdout-sweep protocol | `.factory/maintenance/breaking-change-delivery-protocol.md` (new) | Documentation |

No Rust source files in `src/`, no test files in `tests/`, no `Cargo.toml` changes.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `bin/validate-citations` | Effectful Python tool | Reads filesystem (file existence, line count); writes stdout |
| `bin/test_validate_citations.py` | Pure test harness | Uses stdlib `unittest`, `tempfile`; no external I/O |
| `STORY-INDEX.md` | Documentation artifact | Governance prose + index table |
| `docs-writer-dispatch-guidance.md` | Documentation artifact | Governance prose |
| `ci.yml` | CI configuration | Runs in CI sandbox; deterministic bash logic |
| `CLAUDE.md` | Documentation artifact | Navigation index |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | validate-citations input has zero citations (empty file or all comments) | PASS: 0 citations verified; exit 0 |
| EC-002 | Citation with a range where start > end (e.g., `file.md:20-10`) | FAIL with "INVALID RANGE: path:20-10 (start > end)"; exit 1 |
| EC-003 | Changelog-gate: PR adds only `## [Unreleased]` header to CHANGELOG.md (no bullet/content) | Content assertion fires: CONTENT_LINES=0; gate FAILS as intended |
| EC-004 | Changelog-gate: PR removes lines from CHANGELOG.md with net zero additions | `grep '^+'` only captures added lines; CONTENT_LINES counts only additions; gate correctly checks additions only |
| EC-005 | validate-citations run from directory that is not the repo root | Repo-root resolution follows `compute-input-hash` pattern (walk upward for `.factory/`; `WIRERUST_REPO_ROOT` env override) |
| EC-006 | STORY-INDEX legend update: existing stories with `status: completed` vs. `status: delivered` | Legend clarifies these are delivery-class synonyms; no mass rename required |
| EC-007 | docs-writer-dispatch-guidance.md §4 already has a manual preflight step | The new step is added alongside (not replacing) the existing verification template; both steps remain in the dispatch workflow |

## Tasks

1. **Create bin/validate-citations (AC-164-002):** Write `bin/validate-citations` as a
   Python 3.10+ stdlib-only script following the `bin/compute-input-hash` structural
   pattern (shebang, module-level docstring with ALGORITHM section, repo-root resolution,
   argparse). Input: filename argument (or stdin). Parse each non-blank non-comment line
   as `path:LINE` or `path:LINE-LINE`. Validate file existence and line-range bounds.
   Exit 0 on success, exit 1 on any failure.

2. **Create bin/test_validate_citations.py (AC-164-002):** Write the self-test using
   `unittest` + `tempfile`. Create temporary files with known line counts. Test all seven
   cases listed in AC-164-002(d). Verify: `python3 bin/test_validate_citations.py` runs
   green.

3. **Amend docs-writer-dispatch-guidance.md §4 (AC-164-002):** Append the
   `bin/validate-citations` preflight step to Section 4 of
   `.factory/maintenance/docs-writer-dispatch-guidance.md`. Place it after the existing
   verification template block and before the section end. Cite PG-W73-CITATION-VALIDATOR
   and STORY-164 AC-164-002.

4. **Amend .github/workflows/ci.yml (AC-164-003):** Add the content-assertion bash block
   immediately after the existing `grep -q '^CHANGELOG\.md$'` presence check in the
   `changelog-gate` job. Preserve the `set -euo pipefail` header and all existing
   comments. Verify the action-pin-gate exemption list is unchanged.

5. **Amend CLAUDE.md (AC-164-004):** Add the `docs-writer-dispatch-guidance.md` row to
   the Project References table, immediately after the `pr-manager-merge-auth-guidance.md`
   row. Use the exact wording from AC-164-004.

6. **Amend STORY-INDEX (AC-164-001):** Add the status-vocabulary legend immediately after
   the `## Index Table` heading. Include all six statuses, synonym note, and loci
   agreement rule per AC-164-001. No row-level status changes are required for this story.

7. **Open develop PR:** Create a PR targeting `develop` for the develop-tree changes
   (bin/validate-citations, bin/test_validate_citations.py, .github/workflows/ci.yml,
   CLAUDE.md). Add a CHANGELOG.md `[Unreleased]` entry (required by AC-158-001 /
   PG-W71-CHANGELOG, since bin/ is in the trigger set). The STORY-INDEX amendment and
   docs-writer-dispatch-guidance.md §4 update are factory-artifacts branch commits only.

8. **Create breaking-change-delivery-protocol.md (AC-164-005):** Write
   `.factory/maintenance/breaking-change-delivery-protocol.md` on the factory-artifacts
   branch. The document MUST cover: scope trigger conditions (BREAKING tag / JSON enum
   casing / observable text layout), the mandatory delivery gate steps (run holdout
   evaluator, identify stale scenarios, repair, record `holdout-expectations-sweep:
   COMPLETE`), and the wave-72 evidence (13 stale scenarios HS-021/024/032/033/034/035/
   050/054/059/064/065/074/075 repaired at gate after STORY-160; source:
   `.factory/cycles/wave-72/lessons.md` Lesson 2, tag PG-W72-BREAKING-HOLDOUT-SWEEP).
   The CLAUDE.md row for this file is added to the same develop PR as AC-164-004 (Task 5).

> **Note for implementer:** The develop PR (Task 7) is required for AC-164-002 (bin/),
> AC-164-003 (.github/), and AC-164-004 (CLAUDE.md). AC-164-001 (STORY-INDEX legend)
> and the docs-writer-dispatch-guidance.md §4 update (AC-164-002(e)) are factory-
> artifacts branch commits only. Both tracks must complete for the story to be declared
> delivered.

## Previous Story Intelligence

Lessons from analogous governance/tooling stories in E-11 — especially STORY-162 and
STORY-163, which immediately precede this story in wave-73:

- **STORY-163 (wave-73, E-11, 2 pts) — meta-irony of F-S163P1-001:** STORY-163
  codified the citation mandate (AC-163-001) requiring docs-writers to provide file:line
  anchors for every behavioral claim. The adversary's Pass 1 then found CRITICAL-severity
  fabricated citations in STORY-163's own evidence artifact. Specifically, three
  `.factory/code-delivery/maint-2026-07-09/pr-review.md:332-333` anchors were cited
  when the file is only 111 lines. This meta-failure is the direct evidence motivating AC-164-002: a mechanical
  preflight tool would have caught the phantom anchors before the adversary saw them. When
  implementing `bin/validate-citations`, note that the validate-citations tool should
  itself be authored with precision — do not let the implement of the citation validator
  contain unchecked anchor references in its own doc comments.

- **STORY-162 (wave-73, E-11, 3 pts):** The LMR-003 template-conformance exemption and
  check-green-doc-tense guard tests. Key lesson: when a CI check has hermetic coverage
  gaps (CR-001 from wave-73 gate: AC-158-005 test could pass for the wrong exit code),
  the gap accumulates risk silently. The changelog-gate content assertion (AC-164-003)
  is the analogous fix for the presence-only weakness in AC-158-001.

- **STORY-158 (wave-72, E-11, 3 pts):** Introduced the changelog-gate itself
  (AC-158-001, PG-W71-CHANGELOG). AC-164-003 extends that gate. Follow the same style:
  inline `bash` in the `run:` block, `set -euo pipefail`, explicit `echo` messages for
  both PASS and FAIL paths, exit codes 0/1.

- **STORY-157 (wave-71, E-11, 5 pts):** Created `pr-manager-merge-auth-guidance.md` and
  registered it in CLAUDE.md Project References. AC-164-004 adds the peer entry for
  `docs-writer-dispatch-guidance.md`. Use the exact same row format as the wave-71
  precedent.

## Architecture Compliance Rules

- This story modifies ONLY the files listed in File Structure Requirements below.
- `bin/validate-citations` and `bin/test_validate_citations.py` MUST be Python 3.10+,
  stdlib-only (no `pip install` required — `pathlib`, `re`, `sys`, `argparse`, `hashlib`,
  `textwrap`, `unittest`, `tempfile` are all stdlib). No third-party dependencies.
- All SHA-pinned action refs in `.github/workflows/ci.yml` MUST remain SHA-pinned after
  the amendment. The action-pin-gate (`action-pin-gate` CI job) must continue to pass.
  The `dtolnay/rust-toolchain@stable` exemption must not be disturbed.
- The `set -euo pipefail` header in the changelog-gate `run:` block MUST be preserved.
- No production Rust source (`src/`), no test files (`tests/`), no `Cargo.toml`, no
  story files other than STORY-164.md itself and STORY-INDEX.md.
- STORY-INDEX.md is a factory-artifacts file; the develop PR must NOT include it.
- `docs-writer-dispatch-guidance.md` is a factory-artifacts file; the develop PR must NOT
  include it.

## Library & Framework Requirements

- No code dependencies. Python 3.10+ (stdlib). Bash. No new Rust crates.
- No Rust toolchain changes. No Cargo.toml changes.
- Existing `bin/compute-input-hash` (Python 3.10+) is the structural pattern reference;
  use the same repo-root resolution logic (walk upward, check `.factory/` or `.git`).

## File Structure Requirements

| File | Action | Branch | Notes |
|------|--------|--------|-------|
| `bin/validate-citations` | Create | develop | Python 3.10+ stdlib citation preflight validator |
| `bin/test_validate_citations.py` | Create | develop | Stdlib unittest self-test suite |
| `.github/workflows/ci.yml` | Modify | develop | Add content assertion to changelog-gate |
| `CLAUDE.md` | Modify | develop | Add docs-writer-dispatch-guidance.md row (AC-164-004) + breaking-change-delivery-protocol.md row (AC-164-005) |
| `CHANGELOG.md` | Modify | develop | Add [Unreleased] entry (AC-158-001 obligation for bin/) |
| `.factory/stories/STORY-INDEX.md` | Modify | factory-artifacts | Status-vocabulary legend after `## Index Table` |
| `.factory/maintenance/docs-writer-dispatch-guidance.md` | Modify | factory-artifacts | Add §4 validate-citations preflight step |
| `.factory/maintenance/breaking-change-delivery-protocol.md` | Create | factory-artifacts | BREAKING-change holdout-sweep obligation (PG-W72-BREAKING-HOLDOUT-SWEEP) |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~3.0 k |
| `bin/compute-input-hash` (pattern reference, structural model) | ~0.5 k |
| `.github/workflows/ci.yml` changelog-gate section (~40 lines) | ~0.3 k |
| `docs-writer-dispatch-guidance.md` Section 4 (§4 wiring target) | ~0.4 k |
| `CLAUDE.md` Project References section | ~0.2 k |
| STORY-INDEX v3.42 comment (AC-164-001 evidence) | ~0.3 k |
| **Total** | **~4.7 k** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** All five process gaps are DF-VALIDATION-001-exempt. The
  original four (AC-164-001..004) originate from wave-73 in-process execution findings:
  F-W73G-P3-001 from the wave-73 gate adversarial sweep (in-process), F-S163P1-001 from
  STORY-163 per-story convergence Pass 1 (in-process), PG-W73-CHANGELOG-GATE-CONTENT from
  STORY-162 per-story convergence Pass 5 (in-process), and the wave-73 consistency audit
  advisory (in-process). AC-164-005 (PG-W72-BREAKING-HOLDOUT-SWEEP) originates from the
  wave-72 gate lessons (cycles/wave-72/lessons.md Lesson 2, human-approved 2026-07-11),
  codified via maint-2026-07-11 maintenance run — in-process execution finding carried
  through S-7.02. All five are in-process execution findings — DF-VALIDATION-001-exempt
  per the in-process exemption (same pattern as STORY-162 Notes, STORY-163 Notes,
  STORY-159 Notes, STORY-158 Notes).
- **S-7.02 disposition:** Creating this story at draft status codifies four wave-73
  process-gap findings (PG-W73-STATUS-VOCAB, PG-W73-CITATION-VALIDATOR, PG-W73-CHANGELOG-
  GATE-CONTENT, wave-73 consistency audit advisory for CLAUDE.md) for the S-7.02 wave-73
  cycle-close obligation.
- **No behavioral contract required:** E-11 convention (epics.md E-11: "BCs: none
  authored yet -- status: draft; pending PO authorship").
- **Develop/factory split:** AC-164-002 (bin/validate-citations, bin/test_validate_citations.py),
  AC-164-003 (.github/workflows/ci.yml), and AC-164-004 (CLAUDE.md) touch the develop
  tree and require a PR. AC-164-001 (STORY-INDEX legend) and the docs-writer-dispatch-
  guidance.md §4 update (AC-164-002(e)) are factory-artifacts branch commits only.
- **bin/ CHANGELOG obligation (AC-158-001, PG-W71-CHANGELOG):** The develop PR adds files
  to `bin/` (changelog-gate trigger set); a CHANGELOG.md `[Unreleased]` entry is required
  before the PR can pass the gate. The content assertion added by AC-164-003 will itself
  validate this entry when the story is delivered — a self-referential correctness check.
- **Precedent:** STORY-164 follows the same E-11 pattern: cycle process-gap follow-up
  encoding lessons into project governance and tooling (STORY-157 → wave-70; STORY-158 →
  wave-71; STORY-162 → wave-72; STORY-163 → maint-2026-07-09; STORY-164 → wave-73).

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-07-11 | story-writer | Assigned to wave-74; promoted to ready (plan gate approved, human, 2026-07-11). |
| 1.2 | 2026-07-11 | story-writer | Citation-precision fix (source: story-164-citation-validation-2026-07-11.md) — both loci citing the fabricated STORY-163 anchor corrected from `pr-manager-merge-auth-guidance.md:332-333` to `.factory/code-delivery/maint-2026-07-09/pr-review.md:332-333`; "the file is only 111 lines" now correctly describes pr-review.md (pr-manager-merge-auth-guidance.md is 210 lines). Verified via authoring-evidence.md:113-114 and wc -l pr-review.md. |
| 1.1 | 2026-07-11 | story-writer | maint-2026-07-11 amendment — AC-164-005 added: BREAKING-change holdout-expectation sweep obligation (PG-W72-BREAKING-HOLDOUT-SWEEP); wave-72 Lesson-2 evidence cited (13 stale holdout scenarios at wave-72 gate after STORY-160 casing change); creates `.factory/maintenance/breaking-change-delivery-protocol.md` (factory-artifacts) + CLAUDE.md reference row; delivery checklist gate item `holdout-expectations-sweep: COMPLETE` codified. Points 3→4: AC-164-005 adds a new maintenance protocol document + CLAUDE.md row; 5 ACs total justifies +1 pt over original 3-AC estimate. |
| 1.0 | 2026-07-11 | story-writer | Initial authorship — wave-73 process-gap codifications: PG-W73-STATUS-VOCAB (AC-164-001 STORY-INDEX legend), PG-W73-CITATION-VALIDATOR (AC-164-002 bin/validate-citations), PG-W73-CHANGELOG-GATE-CONTENT (AC-164-003 CI content assertion), wave-73 consistency audit CLAUDE.md row (AC-164-004). S-7.02 wave-73 cycle-close. |
