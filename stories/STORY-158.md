---
document_type: story
story_id: STORY-158
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-08T00:00:00Z
phase: f7
level: feature
cycle: wave-71
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
target_module: bin/lint-cycle-artifact
subsystems: []
estimated_days: 1
wave: "~"
traces_to:
  - .factory/policies.yaml
  - .github/workflows/ci.yml
  - bin/check-green-doc-tense
  - bin/lint-cycle-artifact
input-hash: "595de8c"
inputs:
  - .factory/cycles/wave-71/STORY-157/FINDINGS.md
  - .github/workflows/ci.yml
---

# STORY-158: Wave-71 process-gap codifications: changelog gate, cycle-artifact identity lint, CI scan-guard hardening

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 3
**Priority:** P3

## Narrative

- **As a** factory orchestrator and developer on the wirerust project
- **I want** three wave-71 process gaps codified into durable project artifacts (a CI
  gate, a bin/ lint tool, and two CI scan-guard amendments)
- **So that** PRs with production-code changes but missing CHANGELOG entries are caught
  at CI time, cycle-artifact identity drift (wrong story title, fabricated BC IDs) is
  caught by a lint step rather than relying solely on adversarial review, and two CI
  scan-guard weaknesses (trust-boundary no-src-directory guard, check-green-doc-tense
  silent zero-file scan) are hardened to fail loudly on misconfiguration

## Behavioral Contracts

_(none — E-11 convention: no BCs authored yet; status: draft, pending PO authorship)_

## Background

Wave-71 (STORY-150/156/157, delivered 2026-07-08) and the wave-71 wave-gate
integration review surfaced three process gaps. S-7.02 (cycle-close requirement)
mandates codification of recurring process gaps as follow-up stories.

### PG-W71-CHANGELOG — Unreleased CHANGELOG entries not gated at CI

Wave-71 PRs containing production-code changes (src/ deltas) were merged without
a corresponding Unreleased CHANGELOG entry. No CI step checks this requirement: the
existing workflow validates format only (semantic-PR title type), not content
presence when src/ files change.

Source: F-W71-P1-001 (wave-71 wave-gate pass 1, MEDIUM, process-gap).

Root cause: the per-story delivery flow (pr-manager step-3 PR creation) has no
standing obligation to add a CHANGELOG entry for production-code PRs, and no CI
job enforces the obligation.

### PG-W71-CYCLE-ARTIFACT-IDENTITY — Cycle artifact identity fields unvalidated

During wave-71 adversarial review, a cycle implementation evidence artifact was
found to carry a wrong story-title reference and fabricated BC IDs not present in
the story's `behavioral_contracts` frontmatter. No validation step cross-checks
cycle artifact identity fields against the story file they evidence.

Source: O-W71-P4-002 (wave-71 wave-gate pass 4 observation, process-gap). Related
primary evidence: F-W71-P4-001 (fabricated BC ID in cycle evidence artifact).

Root cause: `validate-template-compliance` covers story frontmatter fields but does
not cross-check cycle artifact header text (story ID, story title, cited BC IDs)
against the story they evidence. Identity drift is invisible until adversarial review
catches it.

### PG-W71-CI-SCAN-GUARDS — Two CI scan-guard weaknesses

Two guard weaknesses in CI scan steps were identified during wave-71 CI review:

**(a) Trust-boundary gate lacks `src/` directory existence check.**
The `trust-boundary` CI job (`.github/workflows/ci.yml`) scans for test-seam
violations:

```bash
VIOLATIONS=$(grep -rn "_for_testing(" src/ \
  | grep -v "fn [a-zA-Z_]*_for_testing(" || true)
```

There is no existence guard on `src/` before this grep. If `src/` is renamed or
deleted, `grep` exits 2, `|| true` suppresses the error, `$VIOLATIONS` is empty,
and the job falsely PASSes. The `help-provenance-gate` job (`.github/workflows/ci.yml`
lines 290–296) has the correct SEC-001 pattern:

```bash
if ! test -f src/cli.rs; then
  echo "FAIL: …"; exit 1
fi
```

The trust-boundary job predates this pattern and has no equivalent guard.

Source: F-W71-P3-001 (wave-71 wave-gate pass 3, LOW, process-gap).
Concrete evidence: `.github/workflows/ci.yml` trust-boundary job, grep line; no
`test -d src/` guard present.

**(b) `bin/check-green-doc-tense` emits WARNING on zero files scanned (exits 0).**
When `_collect_rust_files` returns an empty list (e.g., if the repo structure
changes so that no `.rs` files are tracked), `bin/check-green-doc-tense` prints:

```
WARNING: no tracked Rust files found; scan target may be wrong.
```

and exits 0 — a false CI PASS. The CI step at `.github/workflows/ci.yml` line 423
(`run: python3 bin/check-green-doc-tense`) would mark the job green even if the
entire scan target has silently moved.

Source: F-W71-P3-002 (wave-71 wave-gate pass 3, LOW, process-gap).
Concrete evidence: `bin/check-green-doc-tense` line 367 — `print("WARNING: …",
file=sys.stderr)` followed by continued execution and eventual exit 0.

## Acceptance Criteria

### AC-158-001 (traces to PG-W71-CHANGELOG — CI gate)
A CI job or step exists in `.github/workflows/ci.yml` that detects when a PR
modifies at least one file under `src/` without also modifying `CHANGELOG.md`,
and fails with a human-readable message. The check MUST:
(a) run on every push/PR against `develop`,
(b) emit a message naming the CHANGELOG obligation (reference PG-W71-CHANGELOG and
    this story's AC-158-001),
(c) exit non-zero so the CI job is marked FAILED — not a warning.

### AC-158-002 (traces to PG-W71-CHANGELOG — pr-manager guidance)
`CLAUDE.md` is updated to add a standing obligation in the pr-manager or delivery
guidance: "PRs that modify files under `src/` MUST include an `[Unreleased]`
CHANGELOG entry (enforced by CI; AC-158-001)." The note must reference
PG-W71-CHANGELOG and the AC-158-001 CI gate.

### AC-158-003 (traces to PG-W71-CYCLE-ARTIFACT-IDENTITY — identity lint)
A new `bin/lint-cycle-artifact` script (Python 3, stdlib only) or a validated
extension to `validate-template-compliance` exists that accepts `--story <path>`
and `--artifact <path>` and verifies:
(a) the artifact's story ID reference matches the story's `story_id` frontmatter
    field,
(b) any BC IDs cited in the artifact appear in the story's `behavioral_contracts`
    frontmatter list (for stories with `behavioral_contracts: []` the check passes
    only if the artifact cites no BC IDs).
The tool MUST exit non-zero with a human-readable diff on any mismatch, and exit 0
on a clean identity match. A self-test (`bin/test_lint_cycle_artifact.py`) MUST
cover the mismatch case and the clean case.

### AC-158-004 (traces to PG-W71-CI-SCAN-GUARDS (a) — trust-boundary src/ guard)
The `trust-boundary` CI job in `.github/workflows/ci.yml` includes an explicit
existence guard for the `src/` directory before the grep scan, mirroring the
SEC-001 pattern used by `help-provenance-gate`:

```bash
if ! test -d src/; then
  echo "FAIL: trust-boundary: src/ directory not found — seam scan target moved?"
  echo "Update the scan target in .github/workflows/ci.yml before merging."
  exit 1
fi
```

After the fix, the trust-boundary job cannot silently PASS when `src/` is absent.

### AC-158-005 (traces to PG-W71-CI-SCAN-GUARDS (b) — check-green-doc-tense zero-file guard)
`bin/check-green-doc-tense` exits with a non-zero status code (exit 1 or exit 2)
when `_collect_rust_files` returns an empty list, instead of printing `WARNING` and
continuing to exit 0. The updated error message MUST direct the maintainer to
verify the scan target. `bin/test_check_green_doc_tense.py` is updated with a test
that asserts exit non-zero when `_collect_rust_files` returns `[]`.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| CHANGELOG CI gate | `.github/workflows/ci.yml` (new job or step) | Configuration artifact |
| CHANGELOG pr-manager guidance | `CLAUDE.md` | Documentation artifact |
| Cycle-artifact identity lint | `bin/lint-cycle-artifact` (new) | Effectful (I/O) |
| Cycle-artifact identity self-test | `bin/test_lint_cycle_artifact.py` (new) | Pure (test-only) |
| Trust-boundary src/ guard | `.github/workflows/ci.yml` (amendment) | Configuration artifact |
| check-green-doc-tense zero-file guard | `bin/check-green-doc-tense` (amendment) | Effectful (I/O) |
| check-green-doc-tense self-test update | `bin/test_check_green_doc_tense.py` (amendment) | Pure (test-only) |

No production Rust modules are modified. The `tdd_mode: strict` requirement applies
to `bin/lint-cycle-artifact` — the self-test in `bin/test_lint_cycle_artifact.py`
serves as the Red Gate.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `.github/workflows/ci.yml` | Configuration artifact | No code, no runtime side effects |
| `CLAUDE.md` | Documentation artifact | No code, no side effects |
| `bin/lint-cycle-artifact` | Effectful (I/O) | Reads filesystem (story + artifact files) |
| `bin/test_lint_cycle_artifact.py` | Pure (test-only) | In-memory assertions against fixture text |
| `bin/check-green-doc-tense` | Effectful (I/O) | Reads filesystem |
| `bin/test_check_green_doc_tense.py` | Pure (test-only) | In-memory test assertions |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PR modifies `src/` AND `CHANGELOG.md` | CHANGELOG gate: PASS |
| EC-002 | PR modifies only `CHANGELOG.md` (no src/ change) | CHANGELOG gate: PASS (no src/ delta) |
| EC-003 | PR modifies `src/` without `CHANGELOG.md` | CHANGELOG gate: FAIL with clear message |
| EC-004 | PR modifies only docs/ or .factory/ (no src/ or CHANGELOG change) | CHANGELOG gate: PASS |
| EC-005 | Cycle artifact matches story ID + zero BC IDs cited (empty behavioral_contracts) | lint-cycle-artifact: PASS |
| EC-006 | Cycle artifact cites BC ID not in story's behavioral_contracts | lint-cycle-artifact: FAIL with unmatched BC ID listed |
| EC-007 | Cycle artifact references wrong story ID | lint-cycle-artifact: FAIL with expected vs. actual |
| EC-008 | `src/` directory renamed or removed | trust-boundary: FAIL loudly (existence guard fires, exit 1) |
| EC-009 | `_collect_rust_files` returns empty list | check-green-doc-tense: FAIL, exit non-zero, message directs to scan target |
| EC-010 | `_collect_rust_files` returns non-empty list (normal operation) | check-green-doc-tense: behavior unchanged from pre-fix |

## Tasks

1. **CHANGELOG CI gate (AC-158-001):** Add a new CI job (or step in an existing job)
   to `.github/workflows/ci.yml` that runs `git diff --name-only origin/develop HEAD`
   (or `${{ github.event.pull_request.base.sha }}` on PR events), checks for any
   `src/` path in the diff, and fails if `CHANGELOG.md` is not also in the diff. SHA-pin
   any new action refs per the Action pin gate policy.

2. **CHANGELOG pr-manager guidance (AC-158-002):** Add a sentence to `CLAUDE.md` under
   the delivery or pr-manager section: "PRs that modify files under `src/` MUST include
   an `[Unreleased]` CHANGELOG entry (enforced by CI; AC-158-001, PG-W71-CHANGELOG)."

3. **Cycle-artifact identity lint (AC-158-003):** Create `bin/lint-cycle-artifact`
   (Python 3, stdlib only). Accepts `--story <path>` and `--artifact <path>`. Reads
   `story_id` and `behavioral_contracts` from story YAML frontmatter. Scans artifact
   text for story ID reference and BC-S.SS.NNN patterns. Exits 1 on any mismatch with
   a diff. Create `bin/test_lint_cycle_artifact.py` with at least two test cases: clean
   identity (exit 0) and fabricated BC ID (exit 1).

4. **Trust-boundary src/ guard (AC-158-004):** In `.github/workflows/ci.yml` under the
   `trust-boundary` job's `run:` block, prepend the SEC-001-style existence guard:
   `if ! test -d src/; then echo "FAIL: trust-boundary: src/ directory not found...";
   exit 1; fi`. The guard must appear before the `grep` invocation.

5. **check-green-doc-tense zero-file guard (AC-158-005):** In `bin/check-green-doc-tense`
   at line ~367, change the `print("WARNING: no tracked Rust files found...")` branch to
   print an `ERROR:` message and call `sys.exit(1)` (or equivalent). Update
   `bin/test_check_green_doc_tense.py` to add a test asserting exit non-zero when
   `_collect_rust_files` returns `[]`.

## Previous Story Intelligence

Lessons from closest analogues:
- **STORY-157 (wave-70 process-gap codifications, 5 pts):** Multi-item codification burst;
  input-hash workflow established; pattern for declaring real spec evidence files as
  `inputs:`. Follow the same pattern.
- **STORY-143 (RELEASE-CHANGELOG-FULL-RANGE-001, 3 pts):** CHANGELOG discipline
  codification. AC-158-001 is complementary: STORY-143 ensures completeness of each
  entry; STORY-158 ensures entries exist at all for src/ changes.
- **STORY-147 (PG-MUTANTS-JOBS-001, 3 pts):** Config + documentation deliverable;
  each deliverable was ≤15 lines. Target the same tight scope.
- **STORY-155 (PG-INDEX-DRIFT-001, 3 pts):** Workflow change + policy note. Each codification
  in STORY-158 follows the same pattern: identify root cause, add one structural check.

## Architecture Compliance Rules

- This story modifies ONLY: `.github/workflows/ci.yml`, `bin/check-green-doc-tense`,
  `bin/test_check_green_doc_tense.py`, `CLAUDE.md`, and new files `bin/lint-cycle-artifact`
  and `bin/test_lint_cycle_artifact.py`. No production Rust is touched.
- The CHANGELOG CI gate MUST NOT break CI on the current develop branch (no false
  positives on merged commits).
- The trust-boundary src/ guard MUST use the same pattern as `help-provenance-gate`
  (test -d, not test -f, since src/ is a directory).
- `bin/lint-cycle-artifact` MUST use Python 3 stdlib only (no third-party deps),
  consistent with `bin/compute-input-hash` and other factory bin/ tools.
- The check-green-doc-tense fix MUST NOT alter behavior for non-empty file lists.

## Library & Framework Requirements

- Python 3 standard library only — no third-party deps.
- GitHub Actions CI YAML — any new action `uses:` must be SHA-pinned per the Action pin
  gate policy (40-char commit SHA + `# vX.Y.Z` comment).

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/ci.yml` | Modify | Add changelog-gate job/step; add trust-boundary src/ existence guard |
| `CLAUDE.md` | Modify | Add CHANGELOG obligation to delivery/pr-manager guidance |
| `bin/lint-cycle-artifact` | Create | New Python 3 identity validator for cycle artifacts |
| `bin/test_lint_cycle_artifact.py` | Create | Self-test covering clean + mismatch cases |
| `bin/check-green-doc-tense` | Modify | Change WARNING→ERROR + sys.exit(1) on zero files |
| `bin/test_check_green_doc_tense.py` | Modify | Add zero-file exit-non-zero assertion |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~4 k |
| `.github/workflows/ci.yml` (new gate + amendment) | ~1 k |
| `CLAUDE.md` (1 sentence addition) | ~0.2 k |
| `bin/lint-cycle-artifact` (~70 lines) | ~1 k |
| `bin/test_lint_cycle_artifact.py` (~40 lines) | ~0.5 k |
| `bin/check-green-doc-tense` (one-line fix) | ~0.3 k |
| `bin/test_check_green_doc_tense.py` (one test case) | ~0.3 k |
| **Total** | **~7.3 k** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** All three process gaps originate from wave-71 adversarial
  review and wave-gate pass observations — they are validated in-process and do not
  require separate research-agent validation before issue filing per DF-VALIDATION-001,
  which applies to deferred open findings (not directly-observed adversarial findings).
- Source process-gaps: PG-W71-CHANGELOG (F-W71-P1-001, wave-71 wave-gate pass 1, MEDIUM);
  PG-W71-CYCLE-ARTIFACT-IDENTITY (O-W71-P4-002, wave-71 wave-gate pass 4, observation;
  related: F-W71-P4-001 fabricated BC ID in cycle evidence);
  PG-W71-CI-SCAN-GUARDS (F-W71-P3-001/002, wave-71 wave-gate pass 3, LOW).
- Concrete evidence for PG-W71-CI-SCAN-GUARDS: `.github/workflows/ci.yml` trust-boundary
  job grep scan has no `test -d src/` guard; `bin/check-green-doc-tense` line 367 emits
  `WARNING` and exits 0 on empty file list. The `help-provenance-gate` job has the
  correct SEC-001 pattern as the reference implementation.
- S-7.02 disposition: creating this story at draft status codifies three wave-71 PG-*
  open items for S-7.02 wave-71 cycle-close purposes.
- No behavioral contract required: E-11 convention (epics.md E-11: "BCs: none authored
  yet — status: draft; pending PO authorship").
- input-hash note: v1.0 declares two real spec inputs
  (`.factory/cycles/wave-71/STORY-157/FINDINGS.md` — primary evidence source for all
  three PG wave-71 process observations; `.github/workflows/ci.yml` — source artifact
  for the trust-boundary and check-green-doc-tense CI gaps). The frontmatter input-hash
  field is always the authoritative current value.
- Precedent: STORY-157 (PG-W70-*, wave-71 delivery, 2026-07-08) — same E-11 pattern:
  cycle process-gap follow-up encoding lessons into project workflow, tooling, and docs.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-08 | story-writer | Initial authorship — wave-71 process-gap codifications: PG-W71-CHANGELOG (changelog gate AC-158-001/002), PG-W71-CYCLE-ARTIFACT-IDENTITY (lint tool AC-158-003), PG-W71-CI-SCAN-GUARDS (trust-boundary guard AC-158-004, check-green-doc-tense fix AC-158-005); S-7.02 wave-71 cycle-close. |
