---
document_type: story
story_id: STORY-164
epic_id: E-11
version: "1.16"
status: delivered
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
input-hash: "3929496"
---

# STORY-164: Wave-73 cycle-closing: status-vocabulary legend, citation preflight validator, changelog-gate content assertion, guidance-doc reference row, BREAKING-change holdout-sweep obligation

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** delivered
**Wave:** 74
**Points:** 4
**Priority:** P3

## Narrative

- **As a** spec-steward, orchestrator operator, and future contributor on the wirerust project
- **I want** five process improvements codified into durable project artifacts: a canonical
  status-vocabulary legend in STORY-INDEX, a mechanical citation preflight validator in
  `bin/`, a content assertion added to the changelog-gate CI job, a CLAUDE.md Project
  References row for the docs-writer dispatch guidance, and a BREAKING-change holdout-sweep
  obligation protocol in `.factory/maintenance/`
- **So that** future contributors have an authoritative definition of story-status vocabulary,
  phantom-anchor fabrication (citations to nonexistent files or out-of-range lines — the
  F-S163P1-001 class) is caught mechanically before dispatch; content-mismatch fabrication
  remains an adversarial-review responsibility, the changelog-gate cannot be silently satisfied by a whitespace-
  only touch to CHANGELOG.md, the docs-writer dispatch guidance is discoverable from
  CLAUDE.md alongside the existing pr-manager guidance peer, and BREAKING-change stories
  are required to sweep and repair stale holdout expectations before opening a PR

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
- Pre-STORY-164 gate implementation (as of develop b5e1e15 / v0.12.0): the changelog-gate
  was presence-only with no content quality assertion — resolved by this story's AC-164-003
  (ci.yml now delegates to `bin/changelog-gate-check` at line 509).

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

(a) Define all seven recognized status values with precise semantics:

| Status | Definition | Loci |
|--------|------------|------|
| `draft` | Story created; not yet dispatched for implementation | Frontmatter `status:`, body header, index cell |
| `ready` | Spec-first gate (S-7.01) passed; story may be dispatched | Frontmatter `status:`, body header, index cell |
| `pending` | Story dispatched; implementation in progress or blocked on predecessor | Frontmatter `status:`, body header, index cell |
| `delivered` | PR merged to develop; story on develop but wave not yet closed | Frontmatter `status:`, body header, index cell |
| `merged` | Story squash-merged to develop via tagged PR; semantically equivalent to `delivered` | Frontmatter `status:`, body header, index cell |
| `completed` | Equivalent to `delivered`/`merged`; valid wherever used | Frontmatter `status:`, body header, index cell (dominant in early-wave stories); new stories prefer `delivered` |
| `superseded` | Story scope fully delivered by a later story/PR; retained for traceability — no further delivery expected | Frontmatter `status:`, body header, index cell |

(b) Add a **Synonym note**: `delivered`, `merged`, and `completed` are delivery-class synonyms
    — they all mean "PR merged to develop." The canonical term for new stories is `delivered`
    (frontmatter); `completed` is historically dominant in early waves and remains valid
    wherever used. Stories already using `merged` need not be updated. Tooling that reads
    story status MUST treat all three as equivalent delivery-class values.

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
    Non-blank, non-comment lines that do not match the citation regex are MALFORMED
    citations — reported as `MALFORMED: {line}` and counted in the failure denominator
    so that `FAIL: K of N` always reflects the true total input size (F-S164P2-002).

(b) **Validation:** For each entry:
    - The path must not escape the repo root via absolute reference or parent-directory
      traversal (`../`); checked via `resolve()+is_relative_to()` (FAIL otherwise with
      "OUTSIDE REPO: path" — CWE-22 containment, parity with issue #392, F-S164P2-003).
    - The cited file must exist (FAIL otherwise with "FILE NOT FOUND: path").
    - The cited path must be a regular file, not a directory or symlink-to-dir
      (FAIL otherwise with "NOT A FILE: path" — F-S164P8-001).
    - The cited line number (or both endpoints of a range) must be ≥ 1 (FAIL otherwise
      with "INVALID LINE: path:N (line numbers start at 1)").
    - The cited file must be readable by the process (FAIL otherwise with
      "UNREADABLE: path" — catches PermissionError/OSError on the target; F-S164P8-001).
    - For a range citation, the start line must be ≤ the end line (FAIL otherwise with
      "INVALID RANGE: path:N-M (start > end)"; EC-002).
    - The cited line number (or both endpoints of a range) must be ≤ the file's actual
      line count (FAIL otherwise with "LINE OUT OF RANGE: path:N (file has M lines)").
    - Non-parseable lines are reported as "MALFORMED: {line}" (see (a)).

(c) **Output:** On success (all citations valid), print `PASS: N citations verified` and
    exit 0. On failure, print each failing entry with its failure reason, a summary
    `FAIL: K of N citations invalid`, and exit 1. MALFORMED lines count toward both K
    (failures) and N (total), so a malformed-only input produces `FAIL: 1 of 1` not
    `FAIL: 1 of 0` (F-S164P2-002). A non-UTF-8 or unreadable citations file, or non-UTF-8
    bytes on stdin, prints an error message to stderr and exits 2 (usage error — exit 2 is
    reserved for input/argument errors; citation validation failures use exit 1)
    (F-S164P2-004, F-S164P6-001).

(d) **Self-test:** A corresponding `bin/test_validate_citations.py` (Python 3 stdlib,
    `subprocess`+`tempfile`) is created covering 22 test cases (T01–T22):
    - T01: Valid file:line citation passes
    - T02: Valid file:line-range citation passes
    - T03: Nonexistent file is rejected with FILE NOT FOUND
    - T04: Out-of-range single line is rejected with LINE OUT OF RANGE
    - T05: Out-of-range range endpoint is rejected
    - T06: Comment lines and blank lines are ignored
    - T07: Empty input (no citations) produces PASS: 0 citations verified
    - T08 (EC-002): start > end range → INVALID RANGE, exit 1
    - T09: Citations file not found → exit 2 (usage error)
    - T10: Multiple valid citations pass with correct count
    - T11: Mixed valid + invalid → correct failure count, exit 1
    - T12: Non-parseable line (space instead of colon) → MALFORMED, exit 1
    - T13: Line number 0 → INVALID LINE, exit 1
    - T14: Range start 0 → INVALID LINE, exit 1
    - T15 (F-S164P2-002): Malformed-only input → FAIL: 1 of 1 (denominator includes MALFORMED)
    - T16 (F-S164P2-003): Absolute path → OUTSIDE REPO, exit 1 (CWE-22)
    - T17 (F-S164P2-003): Parent-escape path (`../../`) → OUTSIDE REPO, exit 1 (CWE-22)
    - T18 (F-S164P2-004): Non-UTF-8 citations file → exit 2, no UnicodeDecodeError traceback
    - T19 (F-S164P3-003): Unreadable citations file (chmod 000) → exit 2, no PermissionError
      traceback; root-environment skip guard prevents false pass/fail when running as root
    - T20 (F-S164P6-001): Non-UTF-8 bytes on stdin → exit 2, no UnicodeDecodeError traceback
      (stdin parity with file-argument exit-2 path)
    - T21 (F-S164P8-001): Citation to an existing directory → NOT A FILE, exit 1, no
      IsADirectoryError traceback
    - T22 (F-S164P8-001): Unreadable cited target (chmod 000) → UNREADABLE, exit 1, no
      PermissionError traceback; root-environment skip guard (same pattern as T19)

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

(a) The content assertion is implemented in a new `bin/changelog-gate-check` bash script
    that reads the CHANGELOG diff from stdin and exits 0 (PASS) or 1 (FAIL). The
    `changelog-gate` ci.yml job delegates to it immediately after the presence check:
    ```bash
    git diff origin/develop...HEAD -- CHANGELOG.md | bin/changelog-gate-check
    ```
    `bin/changelog-gate-check` captures the diff via `$(cat)`, then counts non-blank,
    non-section-header added lines. A `{ ... || true; }` brace group prevents
    `set -euo pipefail` from aborting on empty selection (whitespace-only / header-only /
    deletions-only diffs):
    ```bash
    #!/usr/bin/env bash
    set -euo pipefail
    CHANGELOG_DIFF=$(cat)
    CONTENT_LINES=$(echo "${CHANGELOG_DIFF}" | \
      { grep '^+' | \
        grep -v '^+++' | \
        grep -v '^+[[:space:]]*$' | \
        grep -v '^+##' || true; } | \
      wc -l | tr -d ' ')
    if [ "${CONTENT_LINES}" -eq 0 ]; then
      echo "FAIL: CHANGELOG.md touched but no content added to [Unreleased] section."
      echo "(A whitespace-only touch does not satisfy AC-158-001 / PG-W71-CHANGELOG.)"
      exit 1
    fi
    echo "PASS: CHANGELOG.md updated with ${CONTENT_LINES} content line(s)."
    exit 0
    ```

(b) The check MUST be **reliably green** per the no-flaky-stub policy: it passes if and
    only if at least one non-blank, non-section-header (`##`) added line exists in the
    CHANGELOG.md diff. A real entry (bullet point or prose sentence) will always satisfy
    this; a whitespace-only touch, or a `##`-prefixed section or version heading alone (e.g.,
    `## [x.y.z]`), will always fail — `##`-prefixed lines are filtered by `grep -v '^+##'`
    in `bin/changelog-gate-check`. No external state is required; the check is deterministic
    given the diff content.

(c) The existing CHANGELOG obligation comment and `echo` messages in ci.yml are preserved.
    The ci.yml change replaces the inline content-counting bash block with a single pipe
    delegation to `bin/changelog-gate-check`. The extracted script carries `set -euo pipefail`
    independently and is exercised in isolation by `bin/test_changelog_gate_content.py`
    behavioral tests B01–B05.

Verification:
```bash
test -f bin/changelog-gate-check
grep -n "changelog-gate-check" .github/workflows/ci.yml
python3 bin/test_changelog_gate_content.py
```
All three must succeed: script exists, ci.yml has the delegation line, and behavioral tests
(B01–B05) pass against crafted diffs.

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
| Changelog-gate content assertion script | `bin/changelog-gate-check` (new) | Effectful (stdin reads, stdout, exit codes) |
| Changelog-gate content assertion CI wiring | `.github/workflows/ci.yml` (amend) | CI configuration |
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
| `bin/changelog-gate-check` | Effectful bash script | Reads stdin (CHANGELOG diff); writes stdout; exits 0/1 |
| `ci.yml` | CI configuration | Runs in CI sandbox; delegates to `bin/changelog-gate-check` via pipe |
| `CLAUDE.md` | Documentation artifact | Navigation index |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | validate-citations input has zero citations (empty file or all comments) | PASS: 0 citations verified; exit 0 |
| EC-002 | Citation with a range where start > end (e.g., `file.md:20-10`) | FAIL with "INVALID RANGE: path:20-10 (start > end)"; exit 1 |
| EC-003 | Changelog-gate: PR adds only `## [Unreleased]` header to CHANGELOG.md (no bullet/content) | `grep -v '^+##'` filters the header line; `{ ... \|\| true; }` brace group in `bin/changelog-gate-check` prevents pipefail abort on empty selection; `wc -l` returns 0; `CONTENT_LINES=0`; gate reaches the explicit FAIL branch with diagnostic message |
| EC-004 | Changelog-gate: PR removes lines from CHANGELOG.md with net zero additions | `grep '^+'` finds no added lines; `{ ... \|\| true; }` brace group in `bin/changelog-gate-check` prevents pipefail abort on empty selection; `CONTENT_LINES=0`; gate correctly reaches the explicit FAIL branch with diagnostic message |
| EC-005 | validate-citations run from directory that is not the repo root | Repo-root resolution follows `compute-input-hash` pattern (walk upward for `.factory/`; `WIRERUST_REPO_ROOT` env override) |
| EC-008 | validate-citations: MALFORMED citation (e.g., space instead of colon: `src/file.rs 10-20`) | Counted in both K (failures) and N (total); single MALFORMED input → `FAIL: 1 of 1` not `FAIL: 1 of 0` (F-S164P2-002) |
| EC-009 | validate-citations: absolute path citation (e.g., `/etc/passwd:1`) | Rejected before file-existence check via `resolve()+is_relative_to()`; `OUTSIDE REPO` failure, exit 1 (F-S164P2-003, CWE-22) |
| EC-010 | validate-citations: parent-directory escape path (`../../etc/passwd:1`) | Rejected via `resolve()+is_relative_to()`; `OUTSIDE REPO` failure, exit 1 regardless of whether target file exists (F-S164P2-003, CWE-22) |
| EC-011 | validate-citations: non-UTF-8 bytes in citations file passed as argument | Exit 2 with `"Error: citations file is not valid UTF-8: ..."` on stderr; no UnicodeDecodeError traceback; exit 1 reserved for citation validation failures (F-S164P2-004) |
| EC-012 | validate-citations: unreadable citations file (e.g., chmod 000 / PermissionError) | Exit 2 with `"Error: cannot read citations file: ..."` on stderr via `except OSError` branch; no PermissionError traceback; root-environment skip guard in T19 prevents false pass/fail when process runs as root (F-S164P3-003) |
| EC-013 | validate-citations: non-UTF-8 bytes on stdin (no file-argument path) | Exit 2 with `"Error: stdin is not valid UTF-8: ..."` on stderr; reads `sys.stdin.buffer` and decodes explicitly; parity with file-argument exit-2 path (F-S164P6-001) |
| EC-014 | validate-citations: citation to an existing directory (e.g., `docs:5`) | Passes `exists()` but fails `is_file()`; reported as `NOT A FILE: path`, exit 1, no IsADirectoryError traceback (F-S164P8-001) |
| EC-015 | validate-citations: citation to an unreadable target file (chmod 000 / PermissionError) | `count_lines()` wrapped in `try/except OSError`; reported as `UNREADABLE: path`, exit 1, no PermissionError traceback; root-environment skip guard in T22 (same pattern as T19) (F-S164P8-001) |
| EC-006 | STORY-INDEX legend update: existing stories with `status: completed` vs. `status: delivered` | Legend clarifies these are delivery-class synonyms; no mass rename required |
| EC-007 | docs-writer-dispatch-guidance.md §4 already has a manual preflight step | The new step is added alongside (not replacing) the existing verification template; both steps remain in the dispatch workflow |

## Tasks

1. **Create bin/validate-citations (AC-164-002):** Write `bin/validate-citations` as a
   Python 3.10+ stdlib-only script following the `bin/compute-input-hash` structural
   pattern (shebang, module-level docstring with ALGORITHM section, repo-root resolution,
   argparse). Input: filename argument (or stdin). Parse each non-blank non-comment line
   as `path:LINE` or `path:LINE-LINE`. Validate file existence and line-range bounds.
   Additional failure classes per Pass-2/8 findings: MALFORMED (non-parseable citations
   counted in failure denominator — F-S164P2-002); OUTSIDE REPO (absolute paths and
   `../` escapes rejected via `resolve()+is_relative_to()` — F-S164P2-003, CWE-22);
   non-UTF-8 citations file → exit 2 usage error (F-S164P2-004); NOT A FILE (cited path
   is a directory or non-regular-file — F-S164P8-001); UNREADABLE (cited target is
   unreadable/PermissionError — F-S164P8-001). Exit 0 on success, exit 1 on citation
   failures, exit 2 on usage error.

2. **Create bin/test_validate_citations.py (AC-164-002):** Write the self-test (Python 3
   stdlib, `subprocess`+`tempfile`; T01–T22). Create temporary files with known line
   counts. Cover all 22 cases listed in AC-164-002(d) including Pass-2 additions
   (T15 MALFORMED denominator, T16/T17 OUTSIDE REPO, T18 non-UTF-8 exit 2), Pass-3
   addition (T19 unreadable/OSError exit 2, root-skip guard), Pass-6 addition
   (T20 stdin non-UTF-8 exit 2), and Pass-8 additions (T21 directory target NOT A FILE,
   T22 unreadable target UNREADABLE exit 1, root-skip guard). Verify:
   `python3 bin/test_validate_citations.py` runs green.

3. **Amend docs-writer-dispatch-guidance.md §4 (AC-164-002):** Append the
   `bin/validate-citations` preflight step to Section 4 of
   `.factory/maintenance/docs-writer-dispatch-guidance.md`. Place it after the existing
   verification template block and before the section end. Cite PG-W73-CITATION-VALIDATOR
   and STORY-164 AC-164-002.

4. **Create bin/changelog-gate-check and amend .github/workflows/ci.yml (AC-164-003):**
   Create `bin/changelog-gate-check` (bash, `set -euo pipefail`, reads CHANGELOG diff
   from stdin via `$(cat)`, uses `{ grep ... || true; }` brace group to prevent pipefail
   abort on empty selection, exits 0/1 with diagnostic `echo` messages). Wire ci.yml to
   delegate via `git diff origin/develop...HEAD -- CHANGELOG.md | bin/changelog-gate-check`
   immediately after the presence check. Preserve `set -euo pipefail` in ci.yml and all
   existing comments. Verify the action-pin-gate exemption list is unchanged.

5. **Amend CLAUDE.md (AC-164-004):** Add the `docs-writer-dispatch-guidance.md` row to
   the Project References table, immediately after the `pr-manager-merge-auth-guidance.md`
   row. Use the exact wording from AC-164-004.

6. **Amend STORY-INDEX (AC-164-001):** Add the status-vocabulary legend immediately after
   the `## Index Table` heading. Include all seven statuses, synonym note, and loci
   agreement rule per AC-164-001. No row-level status changes are required for this story.

7. **Open develop PR:** Create a PR targeting `develop` for the develop-tree changes
   (bin/validate-citations, bin/test_validate_citations.py, bin/changelog-gate-check,
   .github/workflows/ci.yml, CLAUDE.md). Add a CHANGELOG.md `[Unreleased]` entry (required by AC-158-001 /
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
  `set -euo pipefail`, explicit `echo` messages for both PASS and FAIL paths, exit codes
  0/1. The delivered AC-164-003 implementation extracts the content-counting logic to
  `bin/changelog-gate-check` rather than inlining it in ci.yml, making the script
  independently testable via `bin/test_changelog_gate_content.py` (behavioral tests B01–B05).

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
- The `set -euo pipefail` header in the changelog-gate `run:` block MUST be preserved in ci.yml; the extracted `bin/changelog-gate-check` script carries its own `set -euo pipefail` header.
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
| `bin/changelog-gate-check` | Create | develop | Bash content assertion script for changelog-gate (AC-164-003); reads CHANGELOG diff from stdin |
| `.github/workflows/ci.yml` | Modify | develop | Wire changelog-gate to delegate to `bin/changelog-gate-check` via pipe (AC-164-003) |
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
  GATE-CONTENT, wave-73 consistency audit advisory for CLAUDE.md) plus one wave-72
  lesson (PG-W72-BREAKING-HOLDOUT-SWEEP, AC-164-005, added v1.1) for the S-7.02
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
- **F-S164P6-002 disposition (accepted-by-design, 2026-07-11):** Pass-6 adversary raised
  that AC-164-003 title says "changelog-gate content assertion" but the gate operates on the
  whole diff (not just the `[Unreleased]` section). Dispositioned accepted-by-design: the
  gate intentionally uses `git diff origin/develop...HEAD -- CHANGELOG.md` (whole-file diff)
  so that any content addition to CHANGELOG.md — not just the `[Unreleased]` section —
  satisfies the obligation; the title refers to the assertion's purpose (ensuring content is
  present), not its implementation scope.
- **Precedent:** STORY-164 follows the same E-11 pattern: cycle process-gap follow-up
  encoding lessons into project governance and tooling (STORY-157 → wave-70; STORY-158 →
  wave-71; STORY-162 → wave-72; STORY-163 → maint-2026-07-09; STORY-164 → wave-73).

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.16 | 2026-07-11 | spec-amend | F-W74P13-001: Background PG-W73-CHANGELOG-GATE-CONTENT evidence bullet reframed historically — "Current gate implementation" → "Pre-STORY-164 gate implementation (as of develop b5e1e15 / v0.12.0)"; stale line-range citation and bash snippet dropped; AC-164-003 resolution noted. Sibling sweep: section heading + quoted adversary note exempt (historical context); line 326 "the check is deterministic" refers to new behavior — clean. |
| 1.15 | 2026-07-11 | spec-amend | F-W74P12-001/002 (spec-precision, no behavior change): narrative "citation fabrication is caught mechanically" narrowed to phantom-anchor subclass (F-W74P12-001); AC-164-002(b) bullet order corrected to match delivered validate() precedence — INVALID LINE before UNREADABLE, INVALID RANGE bullet added between UNREADABLE and LINE OUT OF RANGE (F-W74P12-002). Sibling sweep: single overclaim at narrative line 60 only. |
| 1.14 | 2026-07-11 | spec-amend | F-W74P6-001 + full-legend ground-truth audit: AC-164-001(a) completed row — Loci "Index cell only" → all-loci with dominant note (83 frontmatters ground-truth); Definition "early-wave entries" qualifier removed (STORY-162 wave-73 counter-evidence). 6 other rows audited: all pass. Sync with STORY-INDEX v3.50. |
| 1.13 | 2026-07-11 | spec-amend | F-W74P4-001: AC-164-001(b) synonym-note "pre-v3.00" characterization corrected to descriptive-accurate form ("historically dominant in early waves and remains valid wherever used") — STORY-162 wave-73 counter-evidence. Sync with STORY-INDEX v3.49 Synonym note. |
| 1.12 | 2026-07-11 | spec-amend | F-W74P3-001: AC-164-001(a) superseded row added (seventh status value; STORY-148 ground-truth; loci-rule categories complete); "six recognized" → "seven recognized" in AC text and Task 6; sibling sweep: demo-evidence/story-164/AC-164-001.md "six" count noted stale (currency note added). |
| 1.11 | 2026-07-11 | spec-amend | F-W74P1-001: status ready → delivered at all loci (delivery flip PR #397 d6e3be8; loci-agreement per AC-164-001(c)). Sibling sweep: v1.3 changelog entry "promoted to ready" is historical archive — exempt. |
| 1.10 | 2026-07-11 | spec-amend | F-S164P8-001: AC-164-002(b) extended with NOT A FILE (directory cited, F-S164P8-001) and UNREADABLE (PermissionError on target, F-S164P8-001) — now eight validation checks; AC-164-002(d) T01–T20 → T01–T22 (T21: directory → NOT A FILE exit 1; T22: unreadable target → UNREADABLE exit 1, root-skip guard); EC-014/EC-015 added; Task 1 failure-class list updated; Task 2 count synced to 22 cases. Delivered code verified at 59a70ea. |
| 1.9 | 2026-07-11 | spec-amend | F-S164P7-002: AC-164-003(b) wording precision — removed "version line" from always-satisfy list; added clarification that `##`-prefixed section/version headings are filtered by `grep -v '^+##'` and do not count as content (a version-header-only addition FAILS). Sibling sweep: no other "version line" content claims found. |
| 1.8 | 2026-07-11 | spec-amend | F-S164P6-001 T20 sync: AC-164-002(c) extended to cover non-UTF-8 stdin (F-S164P6-001); AC-164-002(d) T01–T19 → T01–T20 (T20: non-UTF-8 stdin → exit 2, no traceback, parity with file-argument path); EC-013 added (stdin non-UTF-8 edge case); Task 2 count synced to 20 cases; F-S164P6-002 accepted-by-design disposition recorded in Notes. Delivered code verified at f66dd12. |
| 1.7 | 2026-07-11 | spec-amend | F-S164P3-003: AC-164-002(d) test enumeration T01–T18 → T01–T19 (T19: unreadable citations file/chmod 000 → exit 2 via `except OSError` branch, no PermissionError traceback, root-skip guard); EC-012 added for OSError/PermissionError edge case; Task 2 count synced to 19 cases. AC-164-002(c) verified unchanged ("non-UTF-8 or unreadable" already covers OSError broadening). |
| 1.6 | 2026-07-11 | spec-amend | F-S164P2-001..004: AC-164-002(a)/(b)/(c)/(d) extended with MALFORMED denominator (F-S164P2-002), OUTSIDE REPO containment (F-S164P2-003, CWE-22), exit-2 on non-UTF-8 citations file (F-S164P2-004); self-test expanded T01-T18; B05 exec-bit test noted (F-S164P2-001, in test_changelog_gate_content.py); sibling sweep on Tasks and EC rows. AC-164-003(c) B-test range updated B01–B04 → B01–B05 (10-test suite at ed5c90d). Delivered files verified at worktree HEAD ed5c90d. |
| 1.5 | 2026-07-11 | spec-amend | F-S164P1-001(spec) mechanism reconciliation: AC-164-003 amended to describe the delivered extraction mechanism — logic extracted to `bin/changelog-gate-check` (bash, `{ ... \|\| true; }` brace group); ci.yml delegates via pipe; EC-003/EC-004 updated; FSR row added for `bin/changelog-gate-check`; sibling sweep applied to Tasks, Architecture Mapping, Purity Classification, Architecture Compliance Rules, and Previous Story Intelligence. |
| 1.4 | 2026-07-11 | spec-amend | F-S164P1-003: H1 rewritten as single line listing all five deliverables (matches STORY-INDEX:208). F-S164P1-001(spec): AC-164-003 code block amended with terminal `\|\| true` guard so CONTENT_LINES resolves to 0 under `set -euo pipefail` on empty selection; EC-003/EC-004 corrected to reflect explicit FAIL branch reachability. Sibling sweep: Narrative "I want" updated four to five deliverables; S-7.02 disposition note extended with wave-72 fifth finding. |
| 1.3 | 2026-07-11 | story-writer | Assigned to wave-74; promoted to ready (plan gate approved, human, 2026-07-11). |
| 1.2 | 2026-07-11 | story-writer | Citation-precision fix (source: story-164-citation-validation-2026-07-11.md) — both loci citing the fabricated STORY-163 anchor corrected from `pr-manager-merge-auth-guidance.md:332-333` to `.factory/code-delivery/maint-2026-07-09/pr-review.md:332-333`; "the file is only 111 lines" now correctly describes pr-review.md (pr-manager-merge-auth-guidance.md is 210 lines). Verified via authoring-evidence.md:113-114 and wc -l pr-review.md. |
| 1.1 | 2026-07-11 | story-writer | maint-2026-07-11 amendment — AC-164-005 added: BREAKING-change holdout-expectation sweep obligation (PG-W72-BREAKING-HOLDOUT-SWEEP); wave-72 Lesson-2 evidence cited (13 stale holdout scenarios at wave-72 gate after STORY-160 casing change); creates `.factory/maintenance/breaking-change-delivery-protocol.md` (factory-artifacts) + CLAUDE.md reference row; delivery checklist gate item `holdout-expectations-sweep: COMPLETE` codified. Points 3→4: AC-164-005 adds a new maintenance protocol document + CLAUDE.md row; 5 ACs total justifies +1 pt over original 3-AC estimate. |
| 1.0 | 2026-07-11 | story-writer | Initial authorship — wave-73 process-gap codifications: PG-W73-STATUS-VOCAB (AC-164-001 STORY-INDEX legend), PG-W73-CITATION-VALIDATOR (AC-164-002 bin/validate-citations), PG-W73-CHANGELOG-GATE-CONTENT (AC-164-003 CI content assertion), wave-73 consistency audit CLAUDE.md row (AC-164-004). S-7.02 wave-73 cycle-close. |
