---
document_type: story
story_id: STORY-158
epic_id: E-11
version: "1.13"
status: delivered
producer: story-writer
timestamp: 2026-07-08T00:00:00Z
phase: f7
level: feature
cycle: wave-71
points: 3
priority: P3
depends_on: []
blocks: [STORY-159, STORY-160]
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
input-hash: "b6ee893"
inputs:
  - .factory/cycles/wave-71/STORY-157/FINDINGS.md
  - .github/workflows/ci.yml
  - .factory/maintenance/backlog-triage-maint-2026-07-08.md
---

# STORY-158: Wave-71 process-gap codifications: changelog gate, cycle-artifact identity lint, CI scan-guard hardening

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** delivered
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
VIOLATIONS=$(grep -rn "_for_testing(" src/ | grep -v "fn [a-zA-Z_]*_for_testing(") || true
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

**Cargo.lock exclusion (by design):** `Cargo.lock` is also excluded from the trigger
set. Transitive-dependency version bumps route through maintenance-sweep CHANGELOG
discipline, not through this per-PR gate; including `Cargo.lock` in the trigger set
would fire the gate on every routine dependency update that carries no product
behavior change.

### AC-158-002 (traces to PG-W71-CHANGELOG — pr-manager guidance)
`CLAUDE.md` is updated to add a standing obligation in the pr-manager or delivery
guidance: "PRs that modify files under `src/`, `Cargo.toml`, or `bin/` MUST include
an `[Unreleased]` CHANGELOG entry (enforced by CI; AC-158-001)." The note must
reference PG-W71-CHANGELOG and the AC-158-001 CI gate.

### AC-158-003 (traces to PG-W71-CYCLE-ARTIFACT-IDENTITY — identity lint)
A new `bin/lint-cycle-artifact` script (Python 3, stdlib only) exists that accepts
`--story <path>` and `--artifact <path>` and enforces the following HARD FAIL contract.
**Evaluation order:** rule (1) frontmatter presence → rule (6) path/story_id identity →
rule (2) empty-bcs short-circuit [exit 0] → rule (3) BC existence on disk → rule (7) story
ownership. An empty-bcs artifact at a wrong path still FAILS rule (6) before reaching
rule (2).
(1) If the artifact is missing a YAML frontmatter block entirely, OR if the frontmatter
    block is present but missing either the `story_id:` or `bcs:` key, the tool MUST exit
    non-zero immediately with the exact message:
    `ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) -- see current cycle-artifact template (STORY-158)`
    No legacy mode. No SKIP-with-warning. No fallback to body-prose or H1 heading.
(2) If `bcs:` is present and explicitly empty (`bcs: []` or an empty list block), the tool
    MUST exit 0 — this is a valid well-formed artifact with no BC citations. This rule
    short-circuits before rule (7)'s parent-story lookup runs — parent-story existence is NOT
    required for empty-bcs artifacts (empty bcs = no BC claims to validate).
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
(6) The `story_id:` value is validated against the artifact's path. The tool searches
    **upward** from the artifact path for the nearest `STORY-[0-9]+` ancestor directory
    component under `.factory/cycles/` (so `.factory/cycles/wave-72/STORY-158/subdir/finding.md`
    resolves to `STORY-158`). The matched `STORY-[0-9]+` component MUST have an immediate
    parent directory matching the wave-directory naming convention (`wave-[0-9]+`);
    a path like `.factory/cycles/STORY-158/x.md` (no wave-NNN intermediate) →
    invalid-path HARD FAIL. Cycle directories with other naming (maint-*, triage-*,
    feature-*) are outside lint scope by design — the lint targets wave-cycle artifacts
    only. If no valid `STORY-[0-9]+` ancestor exists (or the wave-NNN intermediate is
    absent), the tool MUST exit non-zero with:
    `ERROR: artifact path does not match expected .factory/cycles/<wave>/STORY-NNN/<artifact> pattern -- cannot derive expected story_id`
    If the declared `story_id:` does not match the directory-derived value, the tool MUST
    exit non-zero with:
    `ERROR: story_id: <declared> does not match directory-derived <expected>`
(7) After rules (1), (2), and (6) pass (reached only when `bcs:` is non-empty — rule (2)
    short-circuits before this point for empty-bcs artifacts), the tool resolves the parent
    story at `.factory/stories/<story_id>.md`, using the same factory-root upward search as
    `bin/compute-input-hash` (supports `WIRERUST_REPO_ROOT` override for parity). If the
    parent story file is missing, the tool MUST exit non-zero (HARD FAIL). Then:
    - If the story frontmatter lacks the `behavioral_contracts:` key → HARD FAIL.
    - Any `bcs:` ID absent from the story's `behavioral_contracts:` list → HARD FAIL listing
      ALL offenders.
    **Asymmetry note:** Rules (3) and (7) are complementary — rule (3) guards against
    fabricated IDs (ID exists in the artifact but has no on-disk BC file); rule (7) guards
    against borrowed IDs (ID exists on disk but is not owned by this story). The asymmetry
    is intentional: a story may declare BCs that the artifact does not evidence; an artifact
    may NOT claim BCs that the story does not own.

### AC-158-003(a) (story_id extraction — see Rule 6)
The `story_id:` extraction and directory-match requirement is fully specified in rule (6)
above. `story_id:` is parsed from YAML frontmatter ONLY — no fallback to H1 headings or
bolded header text.

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

This story's own PR modifies nine files (see FSR table) — of which the `bin/` tools
(`bin/lint-cycle-artifact`, `bin/check-green-doc-tense`, `bin/test_lint_cycle_artifact.py`,
`bin/test_check_green_doc_tense.py`) are in the CHANGELOG-gate trigger set (`src/`,
`Cargo.toml`, `bin/`); `.github/` and `CLAUDE.md` are excluded by AC-158-001, but the
`bin/` modifications alone fire the gate (bootstrap self-consistency intact). The same PR
introduces the gate that enforces this requirement.
Therefore the PR MUST include a `CHANGELOG.md` `[Unreleased]` entry covering: the new
changelog-gate CI step, the new `bin/lint-cycle-artifact` tool, and the
`bin/check-green-doc-tense` zero-file-guard hardening — satisfying the gate this PR
introduces (bootstrap self-consistency). The CHANGELOG entry MUST include a
`[process-gap]` provenance note per VSDD convention.

### AC-158-008 (PR type)

The pull request title uses the `ci:` semantic prefix (e.g.,
`ci: CHANGELOG gate + cycle-artifact identity lint + scan-guard hardening`), consistent
with the primary deliverable being a CI gate (AC-158-001). The `bin/` lint tool and
`CLAUDE.md` documentation amendments are supporting surfaces; `ci:` is the correct
semantic type when the principal change is a new CI job or step.

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
| EC-007 | Artifact lacks YAML frontmatter entirely, OR frontmatter present but missing `story_id:` or `bcs:` key | lint-cycle-artifact: HARD FAIL with exact message `ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) -- see current cycle-artifact template (STORY-158)` |
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
   The job definition MUST specify `fetch-depth: 0` (or include an explicit
   `git fetch origin develop` step before the diff) — the `actions/checkout` default of
   `fetch-depth: 1` does not fetch `origin/develop` and will cause the base-SHA diff to fail.
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
   the full HARD FAIL contract in AC-158-003: (1) missing frontmatter or missing
   `story_id:`/`bcs:` keys → exit non-zero with the exact error message; (2) `bcs: []` →
   exit 0; (3) any unresolvable ID in `bcs:` → exit non-zero listing ALL unresolvable IDs;
   (4) body prose BC IDs not checked; (6) `story_id:` derived from the `STORY-[0-9]+` path
   component and validated against the declared value — path-pattern mismatch or
   declared/derived mismatch → exit non-zero with the exact error message; (7) after rules
   (1)+(6) pass, resolve parent story `.factory/stories/<story_id>.md` via upward
   factory-root search (supports `WIRERUST_REPO_ROOT` override) and validate `bcs:` IDs
   against the story's `behavioral_contracts:` list — any unowned ID → exit non-zero listing
   ALL offenders; `bcs: []` → PASS regardless. `story_id:` is parsed from frontmatter only —
   no H1 or bolded-header fallback.
   Create `bin/test_lint_cycle_artifact.py` with eight test cases:
   - **TC1 (missing frontmatter):** artifact with no YAML frontmatter block → expect exit 1
     with the exact `ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) --
     see current cycle-artifact template (STORY-158)` message. The tool uses ASCII `--`
     (not U+2014) in all error strings; TC1 asserts this ASCII form.
   - **TC2 (`bcs: []`):** artifact placed at a CORRECT path (e.g.,
     `.factory/cycles/wave-72/STORY-158/artifact.md`) with matching `story_id: STORY-158`
     and `bcs: []` → expect exit 0. The fixture does NOT need a parent story file — rule (2)
     short-circuits before rule (7)'s parent-story lookup runs. (Rule (6) passes first because
     the path is correct; only then does rule (2) short-circuit.)
   - **TC3 (unresolvable ID):** artifact with a fabricated ID in `bcs:` (no on-disk BC file)
     → expect exit 1 listing ALL unresolvable IDs.
   - **TC4 (prose BC ID only):** artifact with a BC ID referenced only in body prose (not
     in `bcs:` frontmatter) → expect exit 0 (prose not checked).
   - **TC5 (missing `bcs:` key):** artifact with YAML frontmatter that has `story_id:` but
     is missing the `bcs:` key entirely → expect exit 1 with the exact error message
     (rule 1 applies to missing keys as well as missing block).
   - **TC6 (directory/story_id mismatch):** artifact at
     `.factory/cycles/wave-72/STORY-158/FINDINGS.md` with `story_id: STORY-999` → expect
     exit 1 naming both the declared value (`STORY-999`) and the directory-derived expected
     value (`STORY-158`).
   - **TC7 (borrowed BC ID):** artifact with `story_id: STORY-158` and
     `bcs: [BC-2.11.036]` where STORY-158 has `behavioral_contracts: []` → expect exit 1
     listing `BC-2.11.036` as an unowned ID (borrowed from a different story).
   - **TC8 (no wave-NNN intermediate):** artifact at
     `.factory/cycles/STORY-158/x.md` (no `wave-NNN` directory between `.factory/cycles/`
     and `STORY-158`) → expect exit 1 with the exact
     `ERROR: artifact path does not match expected .factory/cycles/<wave>/STORY-NNN/<artifact> pattern -- cannot derive expected story_id`
     message (rule (6) branch (a) — invalid-path HARD FAIL).

   **Fixture construction (hermetic; CI-safe):** All TCs MUST construct isolated
   parent-story and BC-file fixtures under `tempfile.TemporaryDirectory()` and pass the
   temporary root via `WIRERUST_REPO_ROOT`, mirroring the pattern in
   `bin/test_compute_input_hash.py`. Tests MUST NOT reference live `.factory/` files —
   `.factory/` is absent on `develop` checkouts and any test touching it will fail in CI.

4. **Cycle-artifact template and wave-gate checklist — CREATE new files (AC-158-003 legacy scope):**
   Create two new files (`.factory/templates/` directory does not yet exist and must be
   created):
   - **`.factory/templates/cycle-artifact.md`** — new canonical cycle-artifact template.
     Include `story_id:` and `bcs:` YAML frontmatter fields with placeholder values
     (e.g., `story_id: STORY-NNN` and `bcs: []`). This is the template referenced by the
     error message in AC-158-003 rule (1).
   - **`.factory/templates/wave-gate-checklist.md`** — new standalone wave-gate checklist
     (no existing standalone checklist file exists in `.factory/`; wave-gate data currently
     lives in per-cycle `cycles/wave-NNN/wave-gate/` directories, not as a reusable
     template). The checklist MUST require that all cycle artifacts for **this wave and
     forward** carry `story_id:` and `bcs:` frontmatter fields before the gate closes.
     Document explicitly that wave-71-and-earlier artifacts are outside lint scope —
     running `bin/lint-cycle-artifact` against them will fail rule (1) by design, but they
     are not required to be retroactively updated.

> **Note for implementer:** `.factory/` lives on the orphan `factory-artifacts` branch and
> cannot be included in a `develop`-targeted PR. Commit `.factory/templates/cycle-artifact.md`
> and `.factory/templates/wave-gate-checklist.md` to `factory-artifacts` in the same delivery
> burst as the develop PR. Do NOT include `.factory/` paths in the develop PR diff.

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
  obligation + AC-158-006 gate code-review artifact protocol), `CHANGELOG.md` (AC-158-007
  bootstrap self-entry), new files `bin/lint-cycle-artifact` and
  `bin/test_lint_cycle_artifact.py`, and (on `factory-artifacts`)
  `.factory/templates/cycle-artifact.md` and `.factory/templates/wave-gate-checklist.md`.
  No production Rust is touched.
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
| `CHANGELOG.md` | Modify | Add [Unreleased] entry (AC-158-007 bootstrap self-consistency) |
| `bin/lint-cycle-artifact` | Create | New Python 3 identity validator for cycle artifacts |
| `bin/test_lint_cycle_artifact.py` | Create | Self-test covering clean + mismatch cases |
| `bin/check-green-doc-tense` | Modify | Change WARNING→ERROR + sys.exit(1) on zero files |
| `bin/test_check_green_doc_tense.py` | Modify | Add zero-file exit-non-zero assertion |
| `.factory/templates/cycle-artifact.md` | Create | Canonical cycle-artifact template; factory-artifacts branch |
| `.factory/templates/wave-gate-checklist.md` | Create | Standalone wave-gate checklist; factory-artifacts branch |

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
- input-hash note: declares three real spec inputs (fourth PG added in the v1.1 amendment)
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
| 1.13 | 2026-07-08 | story-writer | Adversary P12 fixes + class-level sweep: F-W72-P12-001 (MEDIUM) — EC-007 row exact error message changed from U+2014 em-dash to ASCII `--` (consistent with rule (1), rule (6), and TC1). F-W72-P12-003 (LOW) — frontmatter `blocks:` updated `[]` → `[STORY-159, STORY-160]` (sequencing chain: STORY-158 blocks both downstream stories that depend on it). Class-level sweep: Sweep 1 (non-ASCII in exact-match strings) — EC-007 was the only actionable hit; all other em-dashes in live content are prose punctuation; historical changelog entries exempt. Sweep 2 (BRE alternation) — clean. Sweep 3 (GNU-only grep flags) — all hits in historical changelog rows, exempt. Sweep 4 (boundary guard) — both Decision $n loops retain `(\.|:|,| |\)|\*|\`|$)` class; PASS. Sweep 5 (stale version self-refs) — single hit in historical changelog entry; exempt. Sweep 6 (VP/BC version arithmetic) — N/A for STORY-158; PASS. Sweep 7 (line-number anchors) — no file:line citations in STORY-158 live content; PASS. Sweep 8 (cross-story consistency) — error-string ASCII `--` form now consistent across EC-007, rule (1), rule (6), and TC1. |
| 1.12 | 2026-07-08 | story-writer | Adversary P11 fixes: F-W72-P11-M01 (MEDIUM) — Rule (6) fallback "(or the documented wave-directory form)" deleted; constraint is `wave-[0-9]+` ONLY; added out-of-scope sentence: "Cycle directories with other naming (maint-*, triage-*, feature-*) are outside lint scope by design — the lint targets wave-cycle artifacts only." F-W72-P11-M02 (MEDIUM) — TC count bumped seven→eight; TC8 added: artifact at `.factory/cycles/STORY-158/x.md` (no wave-NNN intermediate) → HARD FAIL with branch-(a) invalid-path error message (tests rule (6) branch (a); TC6 covers branch (b)). F-W72-P11-L01 (LOW) — Rule (1) and rule (6) error strings changed from U+2014 em-dash to ASCII `--` in all error strings; TC1 em-dash disjunction collapsed: tool emits ASCII `--`; TC1 asserts the ASCII form. F-W72-P11-L02 (LOW) — AC-158-001 gains explicit Cargo.lock exclusion note: excluded from trigger set by design (transitive-dep bumps route through maintenance-sweep CHANGELOG discipline, not the per-PR gate). F-W72-P11-L03 (LOW) — AC-158-007 reworded to "nine files (see FSR table)"; three-file undercounting sentence replaced with enumeration of all four `bin/` tools as trigger-set members. |
| 1.11 | 2026-07-08 | story-writer | Adversary P10 fixes: F-W72-P10-L04 (LOW) — rule (6) extended with wave-NNN intermediate requirement: matched `STORY-[0-9]+` component MUST have an immediate parent matching wave-directory naming (`wave-[0-9]+`); path like `.factory/cycles/STORY-158/x.md` (no wave-NNN intermediate) → invalid-path HARD FAIL; error message unchanged. F-W72-P10-L05 (LOW) — TC1 em-dash note added: error string contains U+2014 em-dash; TC1 must copy byte-for-byte from AC, OR tool uses ASCII `--` consistently in both tool and TC — implementer picks ONE form. F-W72-P10-L02 (LOW) — Notes input-hash note stale wording fixed: "v1.1 declares three real spec inputs" → "declares three real spec inputs (fourth PG added in the v1.1 amendment)". |
| 1.10 | 2026-07-08 | story-writer | Adversary P9 fixes: F-W72-P9-M01 (MEDIUM) — AC-158-007 trigger-set claim corrected: sentence previously implied `.github/workflows/ci.yml` was in the CHANGELOG-gate trigger set (`src/`, `Cargo.toml`, `bin/`), contradicting AC-158-001; replaced with explicit wording distinguishing the two `bin/` tools (in trigger set; alone fire the gate) from `.github/` (excluded by AC-158-001). F-W72-P9-L01 (LOW) — rule-ordering DAG added at top of AC-158-003 contract: "Evaluation order: rule (1) frontmatter presence → rule (6) path/story_id identity → rule (2) empty-bcs short-circuit [exit 0] → rule (3) BC existence on disk → rule (7) story ownership"; TC2 fixture instructions clarified to require correct path with matching story_id (rule (6) passes before rule (2) short-circuits; wrong-path empty-bcs artifact still FAILS). |
| 1.9 | 2026-07-08 | story-writer | Adversary P8 fixes: F-W72-P8-L02 (LOW) — rule (2) gains explicit short-circuit note (exits before rule (7)'s parent-story lookup; parent-story existence NOT required for empty-bcs artifacts); rule (7) opening updated to "After rules (1), (2), and (6) pass (reached only when bcs: is non-empty)"; unreachable bcs: [] sub-bullet removed from rule (7); Task 3 TC2 gains note that fixture does NOT need a parent story file. F-W72-P8-L03 (LOW) — rule (6) rewritten to specify upward-search behavior: tool searches upward from artifact path for nearest STORY-[0-9]+ ancestor directory component under .factory/cycles/ (e.g., .factory/cycles/wave-72/STORY-158/subdir/finding.md resolves to STORY-158); no such ancestor → existing HARD FAIL applies. |
| 1.8 | 2026-07-08 | story-writer | Adversary P7 fixes: F-W72-P7-003 (MEDIUM) — ACR "modifies ONLY" list extended with CHANGELOG.md (AC-158-007) and .factory/templates/cycle-artifact.md + .factory/templates/wave-gate-checklist.md (factory-artifacts branch); FSR gains corresponding rows for CHANGELOG.md (Modify) and both template files (Create; factory-artifacts branch noted). F-W72-P7-004 (MEDIUM) — Task 4 gains cross-branch implementer note: .factory/templates/ files committed to factory-artifacts in same delivery burst; do NOT include .factory/ paths in develop PR. F-W72-P7-005 (MEDIUM) — Task 3 gains hermetic fixture-construction note: TCs MUST use tempfile.TemporaryDirectory() + WIRERUST_REPO_ROOT, mirroring bin/test_compute_input_hash.py; no live .factory/ references permitted (CI-safe on develop). F-W72-P7-007 (LOW) — Background bash block: two-line backslash continuation replaced with the one-line verbatim mirror of ci.yml:196. |
| 1.7 | 2026-07-08 | story-writer | Adversary P6 fixes: F-W72-P6-001 (HIGH) — AC-158-003 extended with Rules 6-7: Rule 6 derives expected story_id from .factory/cycles/<wave>/STORY-NNN/ path component (path mismatch or declared≠derived → HARD FAIL); Rule 7 checks every bcs: ID against parent story's behavioral_contracts: (borrowed ID not owned → HARD FAIL listing all offenders); TC6/TC7 added to Task 3; AC-158-003(a) replaced with pointer to Rule 6; Task 3 tool description updated with rules 6-7 and WIRERUST_REPO_ROOT override. F-W72-P6-004 (MEDIUM) — AC-158-008 added: PR title uses ci: semantic prefix (primary deliverable is CI gate); ordering corrected to AC-158-007 then AC-158-008. F-W72-P6-008 (LOW) — Task 1: fetch-depth: 0 required for changelog-gate job (default fetch-depth: 1 does not fetch origin/develop; base-SHA diff fails). F-W72-P6-009 (LOW) — Task 4: exact paths named (CREATE .factory/templates/cycle-artifact.md; CREATE .factory/templates/wave-gate-checklist.md as standalone file; .factory/templates/ directory must be created). |
| 1.6 | 2026-07-08 | story-writer | Adversary P5 fixes: F-W72-P5-002 (MEDIUM) — AC-158-003 rewritten to HARD FAIL contract: (1) missing frontmatter or missing story_id:/bcs: keys → exit non-zero with exact error message; no legacy mode, no SKIP-with-warning; (2) bcs: [] → PASS; (3) unresolvable IDs in bcs: → HARD FAIL listing ALL; (4) body prose not checked; (5) legacy scope procedural (Task 4). AC-158-003(a) added: story_id: extracted from frontmatter only, must match STORY-NNN directory, no H1/bolded-header fallback. EC-005/006/007 updated to match new contract. Task 3 rewritten with five TCs. New Task 4 added (cycle-artifact template + wave-gate checklist update; wave-71-and-earlier outside lint scope); old Tasks 4-6 shifted to 5-7. F-W72-P5-005 (LOW) — AC-158-003(a) story_id extraction convention added (same commit). |
| 1.5 | 2026-07-08 | story-writer | Adversary P4 fixes: F-W72-P4-001 (HIGH) — add AC-158-007 (bootstrap self-consistency): this PR modifies bin/ and .github/ (CHANGELOG-gate trigger set), so it MUST include a CHANGELOG.md [Unreleased] entry with [process-gap] provenance note; AC documents the requirement and names the three items covered. Task 1 extended with explicit self-CHANGELOG bullet. |
| 1.4 | 2026-07-08 | story-writer | Adversary P3 fixes: F-W72-P3-002 (MEDIUM) — EC-011 narrowed: no automation detects empty code-review.md; caught by adversarial reviewer per CLAUDE.md rule only (documentation-only control). F-W72-P3-004 (LOW) — AC-158-001(a) restricted to pull_request trigger only; push-to-develop events are inherently no-op (origin/develop == HEAD) and must not be included. F-W72-P3-006 (LOW) — AC-158-003 and Task 3: scope-asserted BC IDs are ONLY those in artifact bcs: frontmatter field; open-ended scope-assertion header path dropped throughout. F-W72-P3-009 (LOW) — EC-004 Expected Behavior parenthetical trimmed to match the three described surfaces; .factory/ mention removed entirely. |
| 1.3 | 2026-07-08 | story-writer | Adversary P2 fixes: F-W72-P2-005 (MEDIUM) — AC-158-002 and Task 2 updated to three-path trigger set (src/, Cargo.toml, bin/) matching AC-158-001. F-W72-P2-006 (MEDIUM) — body header Wave: TBD → Wave: 72. F-W72-P2-008 (LOW) — ci.yml line-range citation corrected 290–296 → 290–295. F-W72-P2-009 (LOW) — EC-011 rewritten from tautological gate-violation restatement to discriminating edge case: code-review.md written but EMPTY → lint fails. F-W72-P2-010 (LOW) — EC-004 drops .factory/ from excluded surfaces (lives on orphan branch, never appears in develop PR diff). |
| 1.2 | 2026-07-08 | story-writer | Adversary P1 fixes: F-W72-P1-005 (MEDIUM) — AC-158-003 BC-citation lint tightened: lint flags only BC IDs asserted as scope (artifact bcs: frontmatter or scope-assertion headers), NOT BC IDs in body prose; explicit "narrative context permitted" note added; Tasks item 3 updated with scoped-assertion semantics and third test case (prose-only BC does not trigger). F-W72-P1-007 (MEDIUM) — CHANGELOG-gate trigger broadened from src/-only to src/, Cargo.toml, bin/; explicit exclusion rationale for tests/, .github/, docs/ added to AC-158-001 and CI job comment requirement; EC-004 updated; Tasks item 1 updated. F-W72-P1-008 (LOW) — bash block in Background fixed: || true moved outside $() substitution to mirror .github/workflows/ci.yml:196 verbatim. |
| 1.1 | 2026-07-08 | story-writer | Amendment (maint-2026-07-08, S-7.02 cycle-close codification) — add PG-W71-CODEREVIEW-ARTIFACT as fourth process gap: gate-level code-review output not persisted at wave-71 wave gate; MINOR finding text unrecoverable, finding re-keyed CR-W71-001 (canonical-ID collision resolution); adds AC-158-006 (CLAUDE.md gate-close code-review protocol); adds backlog-triage-maint-2026-07-08.md to inputs; input-hash updated; count updated three→four gaps throughout. Evidence: backlog-triage-maint-2026-07-08.md item 7 + pattern-findings.md PF-008. |
| 1.0 | 2026-07-08 | story-writer | Initial authorship — wave-71 process-gap codifications: PG-W71-CHANGELOG (changelog gate AC-158-001/002), PG-W71-CYCLE-ARTIFACT-IDENTITY (lint tool AC-158-003), PG-W71-CI-SCAN-GUARDS (trust-boundary guard AC-158-004, check-green-doc-tense fix AC-158-005); S-7.02 wave-71 cycle-close. |
