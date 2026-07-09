---
document_type: story
story_id: STORY-158
epic_id: E-11
version: "1.6"
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
wave: "72"
traces_to:
  - .factory/policies.yaml
  - .github/workflows/ci.yml
  - bin/check-green-doc-tense
  - bin/lint-cycle-artifact
input-hash: "d3cf551"
inputs:
  - .factory/cycles/wave-71/STORY-157/FINDINGS.md
  - .github/workflows/ci.yml
  - .factory/maintenance/backlog-triage-maint-2026-07-08.md
---

# STORY-158: Wave-71 process-gap codifications: changelog gate, cycle-artifact identity lint, CI scan-guard hardening

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** 72
**Points:** 3
**Priority:** P3

## Narrative

- **As a** factory orchestrator and developer on the wirerust project
- **I want** four wave-71 process gaps codified into durable project artifacts (a CI
  gate, a bin/ lint tool, two CI scan-guard amendments, and a wave-gate code-review
  artifact protocol requirement)
- **So that** PRs with production-code changes but missing CHANGELOG entries are caught
  at CI time, cycle-artifact identity drift (wrong story title, fabricated BC IDs) is
  caught by a lint step rather than relying solely on adversarial review, two CI
  scan-guard weaknesses (trust-boundary no-src-directory guard, check-green-doc-tense
  silent zero-file scan) are hardened to fail loudly on misconfiguration, and future
  wave gates cannot close without persisting a code-review artifact enumerating every
  MINOR and NIT finding

## Behavioral Contracts

_(none — E-11 convention: no BCs authored yet; status: draft, pending PO authorship)_

## Background

Wave-71 (STORY-150/156/157, delivered 2026-07-08) and the wave-71 wave-gate
integration review surfaced three process gaps directly; a fourth was identified
during the maint-2026-07-08 backlog triage. S-7.02 (cycle-close requirement)
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
  | grep -v "fn [a-zA-Z_]*_for_testing(") || true
```

There is no existence guard on `src/` before this grep. If `src/` is renamed or
deleted, `grep` exits 2, `|| true` suppresses the error, `$VIOLATIONS` is empty,
and the job falsely PASSes. The `help-provenance-gate` job (`.github/workflows/ci.yml`
lines 290–295) has the correct SEC-001 pattern:

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

### PG-W71-CODEREVIEW-ARTIFACT — Gate-level code-review output not persisted

Wave-71's wave-gate integration review ran a code-review pass whose output was summarized
as "CR-001 MINOR + 3 NITs; all routed to maintenance/debt; 0 BLOCKING" in
`cycles/wave-71/wave-gate/gate-summary.md`, but no standalone code-review artifact was
written to `cycles/wave-71/wave-gate/`. The MINOR finding text is unrecoverable;
maint-2026-07-08 triage verdict: UNVERIFIABLE. The finding was re-keyed to CR-W71-001 to
resolve a canonical-ID collision with the closed CR-001/PR #177 register row.

Source: maint-2026-07-08 backlog-triage item 7 + pattern-findings.md PF-008.

Root cause: the factory artifact protocol has no standing requirement that gate-level
code-review output be persisted to `cycles/wave-NNN/wave-gate/code-review.md` before
the gate is declared closed. Gate-level reviews exist only as one-line summaries in
gate-summary.md, making individual findings unrecoverable after the review session ends.

## Acceptance Criteria

### AC-158-001 (traces to PG-W71-CHANGELOG — CI gate)
A CI job or step exists in `.github/workflows/ci.yml` that detects when a PR
modifies at least one file under `src/`, `Cargo.toml`, or `bin/` without also
modifying `CHANGELOG.md`, and fails with a human-readable message. The check MUST:
(a) run on `pull_request` events against `develop`; push-to-develop events are
    inherently no-op (origin/develop == HEAD on direct pushes to develop) and the
    trigger MUST be restricted to `pull_request` only to avoid false signals,
(b) emit a message naming the CHANGELOG obligation (reference PG-W71-CHANGELOG and
    this story's AC-158-001),
(c) exit non-zero so the CI job is marked FAILED — not a warning.

**Trigger set rationale:** `src/` (production Rust), `Cargo.toml` (dependency and
version changes), and `bin/` (factory tooling shipped with the repo) are all
user-visible surfaces that warrant a CHANGELOG entry. `tests/` and `.github/` are
process-internal (not user-visible behavior changes). `docs/` is self-documenting
(ADR authoring, README updates do not describe product behavior changes). These
exclusions are explicit and must be documented in the CI job comment.

### AC-158-002 (traces to PG-W71-CHANGELOG — pr-manager guidance)
`CLAUDE.md` is updated to add a standing obligation in the pr-manager or delivery
guidance: "PRs that modify files under `src/`, `Cargo.toml`, or `bin/` MUST include
an `[Unreleased]` CHANGELOG entry (enforced by CI; AC-158-001)." The note must
reference PG-W71-CHANGELOG and the AC-158-001 CI gate.

### AC-158-003 (traces to PG-W71-CYCLE-ARTIFACT-IDENTITY — identity lint)
A new `bin/lint-cycle-artifact` script (Python 3, stdlib only) exists that accepts
`--story <path>` and `--artifact <path>` and enforces the following HARD FAIL contract:
(1) If the artifact is missing a YAML frontmatter block entirely, OR if the frontmatter
    block is present but missing either the `story_id:` or `bcs:` key, the tool MUST exit
    non-zero immediately with the exact message:
    `ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) — see current cycle-artifact template (STORY-158)`
    No legacy mode. No SKIP-with-warning. No fallback to body-prose or H1 heading.
(2) If `bcs:` is present and explicitly empty (`bcs: []` or an empty list block), the tool
    MUST exit 0 — this is a valid well-formed artifact with no BC citations.
(3) If `bcs:` lists one or more BC IDs, every ID MUST resolve on disk at
    `.factory/specs/behavioral-contracts/ss-NN/BC-S.SS.NNN.md` (where `ss-NN` is derived
    from the subsection digits in the ID). Any ID that does not resolve is fabricated. The
    tool MUST exit non-zero and list ALL unresolvable IDs in the error output.
(4) BC IDs in the artifact's body prose or section headers are **NOT checked** — only the
    `bcs:` frontmatter field is linted (prose false-positive protection preserved).
(5) Legacy artifacts (wave-71 and earlier) are outside lint scope **procedurally** — they
    will fail rule (1) if run through the tool, but the tool is not required to have a
    `--skip-legacy` flag or special-case these artifacts. The procedural boundary is
    documented in Task 4.

### AC-158-003(a) (story_id extraction convention)
Under the frontmatter contract above, the `story_id:` value in the artifact's frontmatter
MUST match the `STORY-NNN` directory the artifact lives in (e.g., an artifact at
`.factory/cycles/<wave>/STORY-158/impl-evidence.md` must carry `story_id: STORY-158`).
The tool MUST parse `story_id:` from YAML frontmatter ONLY — no fallback to H1 headings
or bolded header text. Frontmatter-only parsing is consistent with the HARD FAIL contract
in rule (1) above.

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

### AC-158-006 (traces to PG-W71-CODEREVIEW-ARTIFACT — gate code-review artifact protocol)
`CLAUDE.md` is updated to add a standing gate-close requirement: "Before a wave gate is
declared closed, a `cycles/wave-NNN/wave-gate/code-review.md` artifact MUST be written
enumerating every MINOR and NIT finding from the gate-level code review together with
its disposition (accepted/deferred/fixed). A gate with zero findings MUST still create
the file with a 'No findings' note." The requirement MUST reference
PG-W71-CODEREVIEW-ARTIFACT and AC-158-006.

### AC-158-007 (bootstrap self-consistency — this PR's own CHANGELOG entry)

This story's own PR modifies `bin/lint-cycle-artifact`, `bin/check-green-doc-tense`, and
`.github/workflows/ci.yml` — all files in the CHANGELOG-gate trigger set (`src/`,
`Cargo.toml`, `bin/`). The same PR introduces the gate that enforces this requirement.
Therefore the PR MUST include a `CHANGELOG.md` `[Unreleased]` entry covering: the new
changelog-gate CI step, the new `bin/lint-cycle-artifact` tool, and the
`bin/check-green-doc-tense` zero-file-guard hardening — satisfying the gate this PR
introduces (bootstrap self-consistency). The CHANGELOG entry MUST include a
`[process-gap]` provenance note per VSDD convention.

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
| Gate code-review artifact protocol | `CLAUDE.md` (amendment) | Documentation artifact |

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
| EC-004 | PR modifies only docs/, tests/, or .github/ (no src/, Cargo.toml, or bin/ change) | CHANGELOG gate: PASS (excluded surfaces: tests/ and .github/ are process-internal; docs/ is self-documenting) |
| EC-005 | Artifact has valid `story_id:` and `bcs: []` (explicit empty) | lint-cycle-artifact: PASS |
| EC-006 | Artifact `bcs:` contains an unresolvable ID (fabricated — no on-disk BC file) | lint-cycle-artifact: HARD FAIL listing ALL unresolvable IDs |
| EC-007 | Artifact lacks YAML frontmatter entirely, OR frontmatter present but missing `story_id:` or `bcs:` key | lint-cycle-artifact: HARD FAIL with exact message `ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) — see current cycle-artifact template (STORY-158)` |
| EC-008 | `src/` directory renamed or removed | trust-boundary: FAIL loudly (existence guard fires, exit 1) |
| EC-009 | `_collect_rust_files` returns empty list | check-green-doc-tense: FAIL, exit non-zero, message directs to scan target |
| EC-010 | `_collect_rust_files` returns non-empty list (normal operation) | check-green-doc-tense: behavior unchanged from pre-fix |
| EC-011 | `code-review.md` written but EMPTY (no findings content) | Caught by adversarial reviewer per the CLAUDE.md rule (AC-158-006); NO automation exists to detect this — documentation-only control |

## Tasks

1. **CHANGELOG CI gate (AC-158-001):** Add a new CI job (or step in an existing job)
   to `.github/workflows/ci.yml` that runs `git diff --name-only origin/develop HEAD`
   (or `${{ github.event.pull_request.base.sha }}` on PR events), checks for any path
   under `src/`, `Cargo.toml`, or `bin/` in the diff, and fails if `CHANGELOG.md` is
   not also in the diff. Add a comment in the job body documenting the exclusion
   rationale for `tests/`, `.github/`, and `docs/` (process-internal or self-documenting).
   SHA-pin any new action refs per the Action pin gate policy.
   Add a `CHANGELOG.md` `[Unreleased]` entry (with `[process-gap]` provenance note) for
   this PR covering the new changelog-gate CI step, `bin/lint-cycle-artifact`, and
   `bin/check-green-doc-tense` zero-file guard — satisfying the bootstrap self-consistency
   requirement of AC-158-007.

2. **CHANGELOG pr-manager guidance (AC-158-002):** Add a sentence to `CLAUDE.md` under
   the delivery or pr-manager section: "PRs that modify files under `src/`, `Cargo.toml`,
   or `bin/` MUST include an `[Unreleased]` CHANGELOG entry (enforced by CI; AC-158-001,
   PG-W71-CHANGELOG)."

3. **Cycle-artifact identity lint (AC-158-003):** Create `bin/lint-cycle-artifact`
   (Python 3, stdlib only). Accepts `--story <path>` and `--artifact <path>`. Implements
   the HARD FAIL contract in AC-158-003: (1) missing frontmatter or missing `story_id:`/`bcs:`
   keys → exit non-zero with the exact error message; (2) `bcs: []` → exit 0; (3) any
   unresolvable ID in `bcs:` → exit non-zero listing ALL unresolvable IDs; (4) body prose
   BC IDs not checked. `story_id:` is parsed from frontmatter only — no H1 or
   bolded-header fallback (AC-158-003(a)).
   Create `bin/test_lint_cycle_artifact.py` with five test cases:
   - **TC1 (missing frontmatter):** artifact with no YAML frontmatter block → expect exit 1
     with the exact `ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) —
     see current cycle-artifact template (STORY-158)` message.
   - **TC2 (`bcs: []`):** artifact with valid `story_id:` and `bcs: []` → expect exit 0.
   - **TC3 (unresolvable ID):** artifact with a fabricated ID in `bcs:` (no on-disk BC file)
     → expect exit 1 listing ALL unresolvable IDs.
   - **TC4 (prose BC ID only):** artifact with a BC ID referenced only in body prose (not
     in `bcs:` frontmatter) → expect exit 0 (prose not checked).
   - **TC5 (missing `bcs:` key):** artifact with YAML frontmatter that has `story_id:` but
     is missing the `bcs:` key entirely → expect exit 1 with the exact error message
     (rule 1 applies to missing keys as well as missing block).

4. **Cycle-artifact template and wave-gate checklist update (AC-158-003 legacy scope):**
   Update the cycle-artifact template (the canonical template STORY-158 defines as the
   reference) to include `story_id:` and `bcs:` YAML frontmatter fields with placeholder
   values. Update the wave-gate checklist to require that all cycle artifacts for **this
   wave and forward** carry these frontmatter fields before the gate closes. Document
   explicitly that wave-71-and-earlier artifacts are outside lint scope — running
   `bin/lint-cycle-artifact` against them will fail rule (1) by design, but they are not
   required to be retroactively updated.

5. **Trust-boundary src/ guard (AC-158-004):** In `.github/workflows/ci.yml` under the
   `trust-boundary` job's `run:` block, prepend the SEC-001-style existence guard:
   `if ! test -d src/; then echo "FAIL: trust-boundary: src/ directory not found...";
   exit 1; fi`. The guard must appear before the `grep` invocation.

6. **check-green-doc-tense zero-file guard (AC-158-005):** In `bin/check-green-doc-tense`
   at line ~367, change the `print("WARNING: no tracked Rust files found...")` branch to
   print an `ERROR:` message and call `sys.exit(1)` (or equivalent). Update
   `bin/test_check_green_doc_tense.py` to add a test asserting exit non-zero when
   `_collect_rust_files` returns `[]`.

7. **Gate code-review artifact protocol (AC-158-006):** Add a standing requirement to
   `CLAUDE.md` (in the wave-gate or delivery guidance section) that before a wave gate is
   declared closed, a `cycles/wave-NNN/wave-gate/code-review.md` artifact MUST be written
   enumerating every MINOR and NIT finding from the gate-level code review together with
   its disposition (accepted/deferred/fixed). A gate with zero findings still creates the
   file with a "No findings" note. Reference PG-W71-CODEREVIEW-ARTIFACT and AC-158-006.

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
  `bin/test_check_green_doc_tense.py`, `CLAUDE.md` (two amendments: AC-158-002 CHANGELOG
  obligation + AC-158-006 gate code-review artifact protocol), and new files
  `bin/lint-cycle-artifact` and `bin/test_lint_cycle_artifact.py`. No production Rust is
  touched.
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
| `CLAUDE.md` | Modify | Add CHANGELOG obligation (AC-158-002) + gate code-review artifact protocol (AC-158-006) |
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

- **DF-VALIDATION-001 gate:** Three of the four process gaps (PG-W71-CHANGELOG,
  PG-W71-CYCLE-ARTIFACT-IDENTITY, PG-W71-CI-SCAN-GUARDS) originate from wave-71 adversarial
  review and wave-gate pass observations — validated in-process. PG-W71-CODEREVIEW-ARTIFACT
  originates from the maint-2026-07-08 research-agent triage (backlog-triage item 7 +
  PF-008) and is therefore DF-VALIDATION-001-validated by that triage run. None of the four
  require separate research-agent validation before issue filing.
- Source process-gaps: PG-W71-CHANGELOG (F-W71-P1-001, wave-71 wave-gate pass 1, MEDIUM);
  PG-W71-CYCLE-ARTIFACT-IDENTITY (O-W71-P4-002, wave-71 wave-gate pass 4, observation;
  related: F-W71-P4-001 fabricated BC ID in cycle evidence);
  PG-W71-CI-SCAN-GUARDS (F-W71-P3-001/002, wave-71 wave-gate pass 3, LOW);
  PG-W71-CODEREVIEW-ARTIFACT (maint-2026-07-08 item 7 + PF-008; UNVERIFIABLE finding
  re-keyed CR-W71-001 to resolve canonical-ID collision with closed CR-001/PR #177).
- Concrete evidence for PG-W71-CI-SCAN-GUARDS: `.github/workflows/ci.yml` trust-boundary
  job grep scan has no `test -d src/` guard; `bin/check-green-doc-tense` line 367 emits
  `WARNING` and exits 0 on empty file list. The `help-provenance-gate` job has the
  correct SEC-001 pattern as the reference implementation.
- S-7.02 disposition: creating this story at draft status codifies four wave-71 PG-*
  open items for S-7.02 wave-71 cycle-close purposes.
- No behavioral contract required: E-11 convention (epics.md E-11: "BCs: none authored
  yet — status: draft; pending PO authorship").
- input-hash note: v1.1 declares three real spec inputs
  (`.factory/cycles/wave-71/STORY-157/FINDINGS.md` — primary evidence source for the
  three PG wave-71 CI/tooling process observations; `.github/workflows/ci.yml` — source
  artifact for the trust-boundary and check-green-doc-tense CI gaps;
  `.factory/maintenance/backlog-triage-maint-2026-07-08.md` — triage evidence for
  PG-W71-CODEREVIEW-ARTIFACT, item 7 + PF-008). The frontmatter input-hash field is
  always the authoritative current value.
- Precedent: STORY-157 (PG-W70-*, wave-71 delivery, 2026-07-08) — same E-11 pattern:
  cycle process-gap follow-up encoding lessons into project workflow, tooling, and docs.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.6 | 2026-07-08 | story-writer | Adversary P5 fixes: F-W72-P5-002 (MEDIUM) — AC-158-003 rewritten to HARD FAIL contract: (1) missing frontmatter or missing story_id:/bcs: keys → exit non-zero with exact error message; no legacy mode, no SKIP-with-warning; (2) bcs: [] → PASS; (3) unresolvable IDs in bcs: → HARD FAIL listing ALL; (4) body prose not checked; (5) legacy scope procedural (Task 4). AC-158-003(a) added: story_id: extracted from frontmatter only, must match STORY-NNN directory, no H1/bolded-header fallback. EC-005/006/007 updated to match new contract. Task 3 rewritten with five TCs. New Task 4 added (cycle-artifact template + wave-gate checklist update; wave-71-and-earlier outside lint scope); old Tasks 4-6 shifted to 5-7. F-W72-P5-005 (LOW) — AC-158-003(a) story_id extraction convention added (same commit). |
| 1.5 | 2026-07-08 | story-writer | Adversary P4 fixes: F-W72-P4-001 (HIGH) — add AC-158-007 (bootstrap self-consistency): this PR modifies bin/ and .github/ (CHANGELOG-gate trigger set), so it MUST include a CHANGELOG.md [Unreleased] entry with [process-gap] provenance note; AC documents the requirement and names the three items covered. Task 1 extended with explicit self-CHANGELOG bullet. |
| 1.4 | 2026-07-08 | story-writer | Adversary P3 fixes: F-W72-P3-002 (MEDIUM) — EC-011 narrowed: no automation detects empty code-review.md; caught by adversarial reviewer per CLAUDE.md rule only (documentation-only control). F-W72-P3-004 (LOW) — AC-158-001(a) restricted to pull_request trigger only; push-to-develop events are inherently no-op (origin/develop == HEAD) and must not be included. F-W72-P3-006 (LOW) — AC-158-003 and Task 3: scope-asserted BC IDs are ONLY those in artifact bcs: frontmatter field; open-ended scope-assertion header path dropped throughout. F-W72-P3-009 (LOW) — EC-004 Expected Behavior parenthetical trimmed to match the three described surfaces; .factory/ mention removed entirely. |
| 1.3 | 2026-07-08 | story-writer | Adversary P2 fixes: F-W72-P2-005 (MEDIUM) — AC-158-002 and Task 2 updated to three-path trigger set (src/, Cargo.toml, bin/) matching AC-158-001. F-W72-P2-006 (MEDIUM) — body header Wave: TBD → Wave: 72. F-W72-P2-008 (LOW) — ci.yml line-range citation corrected 290–296 → 290–295. F-W72-P2-009 (LOW) — EC-011 rewritten from tautological gate-violation restatement to discriminating edge case: code-review.md written but EMPTY → lint fails. F-W72-P2-010 (LOW) — EC-004 drops .factory/ from excluded surfaces (lives on orphan branch, never appears in develop PR diff). |
| 1.2 | 2026-07-08 | story-writer | Adversary P1 fixes: F-W72-P1-005 (MEDIUM) — AC-158-003 BC-citation lint tightened: lint flags only BC IDs asserted as scope (artifact bcs: frontmatter or scope-assertion headers), NOT BC IDs in body prose; explicit "narrative context permitted" note added; Tasks item 3 updated with scoped-assertion semantics and third test case (prose-only BC does not trigger). F-W72-P1-007 (MEDIUM) — CHANGELOG-gate trigger broadened from src/-only to src/, Cargo.toml, bin/; explicit exclusion rationale for tests/, .github/, docs/ added to AC-158-001 and CI job comment requirement; EC-004 updated; Tasks item 1 updated. F-W72-P1-008 (LOW) — bash block in Background fixed: || true moved outside $() substitution to mirror .github/workflows/ci.yml:196 verbatim. |
| 1.1 | 2026-07-08 | story-writer | Amendment (maint-2026-07-08, S-7.02 cycle-close codification) — add PG-W71-CODEREVIEW-ARTIFACT as fourth process gap: gate-level code-review output not persisted at wave-71 wave gate; MINOR finding text unrecoverable, finding re-keyed CR-W71-001 (canonical-ID collision resolution); adds AC-158-006 (CLAUDE.md gate-close code-review protocol); adds backlog-triage-maint-2026-07-08.md to inputs; input-hash updated; count updated three→four gaps throughout. Evidence: backlog-triage-maint-2026-07-08.md item 7 + pattern-findings.md PF-008. |
| 1.0 | 2026-07-08 | story-writer | Initial authorship — wave-71 process-gap codifications: PG-W71-CHANGELOG (changelog gate AC-158-001/002), PG-W71-CYCLE-ARTIFACT-IDENTITY (lint tool AC-158-003), PG-W71-CI-SCAN-GUARDS (trust-boundary guard AC-158-004, check-green-doc-tense fix AC-158-005); S-7.02 wave-71 cycle-close. |
