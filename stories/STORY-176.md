---
document_type: story
story_id: STORY-176
epic_id: E-11
version: "2.7"
status: delivered
producer: story-writer
timestamp: 2026-07-20T00:00:00Z
phase: f7
level: feature
cycle: feature-iec104
points: 2
priority: P3
depends_on: []
blocks: []
# BC status: E-11 convention — governance-only story; no BCs authored
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: bin/
subsystems: []
estimated_days: 1
wave: "84"
traces_to:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
  - .factory/maintenance/delivery-doc-currency-protocol.md
  - bin/check-green-doc-tense
  - bin/test_check_green_doc_tense.py
  - bin/test_gitignore_mutants_glob.py
  - .gitignore
  - CHANGELOG.md
  - .github/workflows/ci.yml
inputs:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
input-hash: "119f591"
---

# STORY-176: Feature-IEC104 Cycle-Close: Local Gate + Tooling Hygiene Sweeps

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** delivered
**Wave:** 84
**Points:** 2
**Priority:** P3

## Narrative

- **As a** spec-steward, adversary reviewer, and future contributor on the wirerust project
- **I want** two product-local tooling improvements codified: (1) the green-doc-tense CI gate
  extended with phrase-level patterns for stub-era vocabulary (`skeleton`/`seam` compile-only
  assertions), and (2) minor housekeeping items applied to the repo — an input-hash
  post-delivery re-baseline reminder in the delivery protocol and a `mutants.out*/` glob
  added to `.gitignore`
- **So that** stub-era wording is caught by CI before adversarial review, the delivery
  checklist includes the hash re-baseline step, and mutation-testing residue no longer lands
  untracked in the repo root

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

### PG-GATE-VOCAB-BLINDSPOT — green-doc-tense gate misses stub-era vocabulary

`AC-174-008` (STORY-174) extended the green-doc-tense gate (`bin/check-green-doc-tense`)
with phrase-level patterns that should not appear in delivered ("green") code. The existing
pattern list catches common stale-TODO and red-gate phrasing but misses stub-era vocabulary:

- `"skeleton"` — used during VP-044 Kani harness scaffolding ("harness skeleton compiles")
  and in test scaffolding comments; stub-era wording that should not survive into a fully
  implemented story. Bare-word `skeleton` is also used for legitimate past-tense provenance
  (`"error type skeleton (extended in STORY-168)"`, `"proof skeleton"`, `"VP-024 Sub-D
  skeleton"`) — these must NOT be gated; only present-tense compile-only assertions are in
  scope.
- `"seam"` — used in compile-only seam discussions during red-gate phase (`"compile-only
  seam"`, `"are currently compile-only seams"`); likewise should not appear as present-tense
  claims in green-state deliveries. Bare-word `seam` is a first-class codebase idiom for
  test-seam / verification-seam patterns (`VP-047 seam`, `Test seam accessors`, `UDP gap-key
  seam`) — these must NOT be gated; only phrase-level stub-state assertions are in scope.

Two independent adversary observations on STORY-174: P2 Obs-1 (stale skeleton prose in
the story spec itself) and a seam commentary observation (see convergence-report.md F-174-002
Pass-2 / PG-GATE-VOCAB-BLINDSPOT in lessons.md D-462). Both were remediated in STORY-174
delivery, but the gate did not catch them automatically because the pattern list was not
extended.

This is a feature-iec104 cycle-execution finding — DF-VALIDATION-001-exempt per the
in-process exemption.

**v2.3 correction note:** The original AC-176-001 (v2.2) described the gate locus as
"`.github/workflows/ci.yml`", bare-word token patterns (`\bskeleton\b` / `\bseam\b`),
and an inline `# green-doc-tense-gate: allow` allowlist mechanism. All three claims were
invalidated by the DF-VALIDATION-001 research pass
(`planning/story-176-ac001-validation.md`, 2026-07-20): the gate lives in
`bin/check-green-doc-tense` (ci.yml only invokes the tool), bare tokens produce ~91
legitimate matches, and no inline allowlist mechanism exists. This v2.3 corrects the
locus, replaces bare-word tokens with four phrase-level zero-FP patterns, removes the
fabricated allowlist claim, and corrects the CHANGELOG obligation (`bin/` IS in the
AC-158-001 changelog-gate trigger set — entry required).

**P4-seam provenance attribution note:** STORY-176 §Background (v2.2) described the two
STORY-174 adversary findings as "P2 Obs-1 (stale skeleton prose in the story spec itself)
and P4 finding (stale seam commentary in test headers)." The validation report found this
attribution **INCONCLUSIVE** (MEDIUM confidence): the convergence-report's actual P4
finding (F-174-P4-001) is a BC-2.19.025 invariant-2 mis-anchor, not seam commentary. The
seam observation is corroborated in lessons.md (D-462) and the convergence-report Pass-2
finding (F-174-002), but the exact per-pass label is ambiguous across artifacts. Verbatim
flagged lines were scrubbed by commit 038286a and are not recoverable.
PG-GATE-VOCAB-BLINDSPOT remains the motivating pattern gap regardless of the precise
per-pass attribution.

### Input-hash self-referential drift (absorbed from STORY-178 AC-178-003)

Stories whose `inputs:` list includes the same spec files that get modified during their
delivery will have a stale `input-hash` immediately after delivery. This is expected and
correct behavior (the hash detects drift), but the post-delivery re-baseline step is not
documented as a standard checklist item. Observed on STORY-164/165 (re-baselined 2026-07-18).

### .gitignore lacks `mutants.out*/` glob (absorbed from STORY-178 AC-178-004)

`.gitignore` covers `mutants-f6*/` (F6 targeted hardening runs) but not the default
`mutants.out/` and `mutants.out.j4-invalid/` directories produced by standard `cargo
mutants` runs. Both directories land as untracked files in the repo root after mutation
testing sessions (confirmed in current `git status`). The pattern `mutants.out*/` covers
both.

## Disposition (engine ACs from v1.0 routed upstream)

The following ACs from STORY-176 v1.0 were engine-level and routed to the vsdd-factory
engine repo 2026-07-19:

| AC (v1.0) | Description | Upstream |
|-----------|-------------|----------|
| AC-176-002 (pre-adversarial doc sweep step) | Engine: delivery-protocol prompt behavior | drbothen/vsdd-factory#682 |
| AC-176-003 (adversary severity calibration guidance) | Engine: adversary agent prompt calibration | drbothen/vsdd-factory#686 |

> **AC-ID renumbering note (v2.0):** The "AC (v1.0)" IDs in the table above refer to the
> **retired v1.0 numbering**. At v2.0 the story was re-scoped and the AC IDs were reassigned.
> The live `AC-176-002` (input-hash post-delivery re-baseline reminder, absorbed from STORY-178
> AC-178-003) and `AC-176-003` (`.gitignore mutants.out*/` glob, absorbed from STORY-178
> AC-178-004) are **unrelated absorbed ACs** — they are entirely different acceptance criteria,
> not updated versions of the engine-routed items above.

## Acceptance Criteria

### AC-176-001 (traces to PG-GATE-VOCAB-BLINDSPOT — extend green-doc-tense gate pattern list)

The green-doc-tense gate (`bin/check-green-doc-tense`, extended per AC-174-008) is
extended with phrase-level patterns for stub-era vocabulary (`skeleton`/`seam` as
compile-only or wiring-incomplete present-tense assertions). `ci.yml` is NOT modified —
it only invokes the tool.

- Given the existing `green-doc-tense-gate` missed stub-era vocabulary during STORY-174
  delivery (skeleton/seam adversary findings; see Background §PG-GATE-VOCAB-BLINDSPOT)
- When `bin/check-green-doc-tense` `_VIOLATION_PATTERNS` list is extended with four
  phrase-level, case-insensitive, comment-line-matched patterns:
  (a) `skeleton\s+compiles?` — catches "harness skeleton compiles" (stub-era: it only
      compiles, no real proof/assertions); past-tense forms (`skeleton originated`,
      `proof skeleton`) are not matched by specificity
  (b) `compile-only\s+seams?` — catches "compile-only seam(s)" present-tense assertions
      (stale once upgraded per AC-174-002); `test seam`, `Test seam accessors`,
      `VP-047 seam` are not matched by specificity
  (c) `(?:are|is)\s+(?:currently\s+)?compile-only` — catches "are currently compile-only
      seams with no assertions" style present-tense claims
  (d) `\buntil\b[^\n]*\bwired\b` — catches "fails until wired" CI-wiring prose

  The implementer MAY refine the regex spelling (e.g., case flags, anchoring), but MUST
  maintain: (i) phrase-level patterns, not bare word tokens; (ii) comment-line-only
  matching (consistent with existing engine); (iii) zero false positives on the current
  tree confirmed before merge; (iv) known-bad + known-good fixture pairs added for each
  new pattern.

- And the **allowlist** is pattern specificity, NOT inline annotations. There is no
  `# green-doc-tense-gate: allow` mechanism in this gate. If any pre-existing use matches
  the chosen regex spelling, the implementer MUST narrow the pattern, not add an annotation.

- And corresponding known-bad + known-good fixture pairs are added to
  `bin/test_check_green_doc_tense.py` for each of the four new patterns (fixture structure
  at `bin/test_check_green_doc_tense.py:51+`); the self-test MUST pass, proving no
  regression against existing patterns

- And a CHANGELOG `[Unreleased]` entry records the gate extension (`bin/` is in the
  AC-158-001 changelog-gate trigger set — entry REQUIRED, same precedent as AC-174-008)

- Then `python3 bin/check-green-doc-tense` and `python3 bin/test_check_green_doc_tense.py`
  both exit 0

Verification:
```bash
# Confirm new patterns are in bin/check-green-doc-tense _VIOLATION_PATTERNS
grep -n "skeleton.*compiles\|compile.only.*seam\|currently.*compile.only\|until.*wired" \
  bin/check-green-doc-tense
# Must emit non-empty output

# Confirm gate and self-test both pass
python3 bin/check-green-doc-tense          # must exit 0 (zero false positives tree-wide)
python3 bin/test_check_green_doc_tense.py  # must exit 0 (including new fixtures)

# Confirm zero false positives for new phrase patterns against current scan set
# (run before merge to satisfy zero-FP requirement)
git ls-files -- 'tests/*.rs' 'src/**/*.rs' | xargs grep -lE \
  "skeleton[[:space:]]+compiles?|compile-only[[:space:]]+seams?|(are|is)[[:space:]]+(currently[[:space:]]+)?compile-only|until.*wired"
# Must emit empty output
```

### AC-176-002 (minor — input-hash post-delivery re-baseline checklist step)

`.factory/maintenance/delivery-doc-currency-protocol.md` is extended with a post-delivery
input-hash re-baseline reminder. The reminder MUST state:

- Stories whose `inputs:` list includes spec files that were amended during the delivery
  wave will have a stale `input-hash` immediately after delivery. This is expected behavior
  (the hash correctly detects drift).
- Post-delivery re-baseline step: run `bin/compute-input-hash --write
  .factory/stories/STORY-NNN.md` for the delivered story immediately after wave close.
- The re-baseline is NOT optional — a STALE hash on a just-delivered story that modified
  its own spec inputs must be resolved before the next wave's plan gate.
- Reference: observed on STORY-164/165 (re-baselined 2026-07-18); also applicable to any
  story in the E-22 or E-11 series that traces to actively-revised spec documents.

Verification:
```bash
grep -n "re-baseline\|input-hash.*post\|post-delivery.*hash" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output
```

### AC-176-003 (minor — .gitignore mutants.out* glob)

`.gitignore` is updated with a `mutants.out*/` glob pattern. The change MUST:

(a) **Pattern:** Add `mutants.out*/` to `.gitignore` under the existing cargo-mutants
    section (near `mutants-f6*/`). The `*` wildcard covers `mutants.out/`,
    `mutants.out.j4-invalid/`, and any future variant of the default output directory.

(b) **Develop PR:** This change touches `.gitignore` in the project root. It is committed
    on the develop branch (no CHANGELOG entry required — `.gitignore` is not in the
    AC-158-001 trigger set). AC-176-003 SHOULD be batched with AC-176-001 in a single
    develop PR.

(c) **Effect:** After this change, `git status` shows no untracked `mutants.out*/`
    entries after a standard `cargo mutants` run.

This AC is regression-guarded by `bin/test_gitignore_mutants_glob.py` (new file, develop
branch), which contains 2 `git check-ignore` assertions verifying the glob covers both
`mutants.out/` and `mutants.out.j4-invalid/`. This test file is added test-first per
`tdd_mode: strict`. The regression guard is CI-enforced: `bin/test_gitignore_mutants_glob.py`
is wired into the existing `bin-selftest` job in `.github/workflows/ci.yml` (one new `run:`
step added; no SHA-pin changes, no `green-doc-tense-gate` job changes).

Verification:
```bash
grep "mutants.out" .gitignore
# Must emit: mutants.out*/
# (in addition to existing mutants-f6*/)
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Green-doc-tense gate phrase-pattern extension | `bin/check-green-doc-tense` (amend) | develop |
| Green-doc-tense gate self-test fixtures | `bin/test_check_green_doc_tense.py` (amend) | develop |
| CHANGELOG entry for gate extension | `CHANGELOG.md` (amend) | develop |
| Input-hash post-delivery re-baseline reminder | `.factory/maintenance/delivery-doc-currency-protocol.md` (amend) | factory-artifacts |
| mutants.out* glob | `.gitignore` (amend) | develop |
| mutants.out* glob regression guard | `bin/test_gitignore_mutants_glob.py` (new) | develop |
| AC-176-003 regression-guard CI wiring | `.github/workflows/ci.yml` (bin-selftest job, amend) | develop |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes. `bin/` changes are
CI tooling only (no production code). `ci.yml` SHA pins are NOT modified (18/18 verified
identical). Commit ea4bcd8e delivered three `ci.yml` edits: (1) one new `run:` step in the
`bin-selftest` job to wire `bin/test_gitignore_mutants_glob.py` (AC-176-003 regression guard);
(2) `bin-selftest` job name de-enumerated to count-free "Bin selftest suites" (W75 NIT-1
count-free discipline); (3) the COMMENT block above the `green-doc-tense-gate` job reworded
count-free (stale "10 known-bad/14 known-good" claim removed). The `green-doc-tense-gate`
job's steps and name are untouched; only its leading comment was changed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A `skeleton` or `seam` use in a comment is legitimate (e.g., `"proof skeleton"`, `"Test seam accessors"`, `"VP-047 seam"`) | The allowlist mechanism is pattern specificity — phrase patterns are designed to exclude past-tense provenance and architectural uses. The implementer MUST verify the chosen regex produces zero false positives against the current scan set before merging, NOT add an inline annotation |
| EC-002 | Story spec (`.factory/`) files contain "skeleton" / "seam" | Gate scan set is `src/**/*.rs` and `tests/*.rs` only (per `bin/check-green-doc-tense` scan-set logic); spec files are outside the scan set by construction |
| EC-003 | Story inputs do not include any spec file revised during delivery | No stale hash after delivery; AC-176-002 note applies but no action required |
| EC-004 | mutants.out* directory exists with important results the implementer wants to keep | .gitignore suppresses tracking but does not delete files; results remain on disk and are simply not staged |

## Tasks

1. **Extend green-doc-tense gate (AC-176-001):** Run zero-false-positive check on
   the current tree for each phrase pattern (verify empty output from `git ls-files` pipe).
   Extend `bin/check-green-doc-tense` `_VIOLATION_PATTERNS` with the four phrase-level
   patterns (a)-(d). Add known-bad + known-good fixture pairs to
   `bin/test_check_green_doc_tense.py` for each new pattern. Add `[Unreleased]`
   CHANGELOG entry. Run `python3 bin/check-green-doc-tense` and
   `python3 bin/test_check_green_doc_tense.py` to confirm both exit 0.
   Batch with AC-176-003 in one develop PR (CHANGELOG entry REQUIRED — `bin/` changes
   trigger changelog-gate per AC-158-001).

2. **Update .gitignore (AC-176-003):** Add `mutants.out*/` glob under cargo-mutants
   section. Batch with AC-176-001 in the same develop PR. Add
   `bin/test_gitignore_mutants_glob.py` (new file) with 2 `git check-ignore` assertions
   confirming the glob covers `mutants.out/` and `mutants.out.j4-invalid/`; this test
   file is added test-first per `tdd_mode: strict` before the `.gitignore` change.

3. **Extend delivery-doc-currency-protocol.md (AC-176-002):** Add input-hash
   post-delivery re-baseline reminder. Factory-artifacts branch commit.

4. **Register in STORY-INDEX.md:** Update STORY-176 row (ready, E-11, wave-84,
   2 pts). Factory-artifacts branch commit.

> **Note for implementer:** Tasks 1 and 2 (`bin/check-green-doc-tense` /
> `bin/test_check_green_doc_tense.py` and `.gitignore`) are develop-branch changes that
> can be batched in one PR. Task 1 (AC-176-001) REQUIRES a CHANGELOG entry (`bin/`
> changes trigger the changelog-gate per AC-158-001). Task 2 (AC-176-003, `.gitignore`)
> does NOT require a CHANGELOG entry. Task 3 is a factory-artifacts branch commit.

## Previous Story Intelligence

- **AC-174-008 (STORY-174, wave-83):** Established `bin/check-green-doc-tense` and its
  phrase-level pattern engine. AC-176-001 extends the same `_VIOLATION_PATTERNS` list
  with four new phrase patterns — read AC-174-008 before writing the
  `bin/check-green-doc-tense` change to match the existing pattern style, comment-line
  anchoring, and fixture structure in `bin/test_check_green_doc_tense.py`.
- **STORY-165 AC-165-003 (wave-75):** Established `delivery-doc-currency-protocol.md`
  as the canonical delivery sweep document. STORY-176 amends the same document with
  the input-hash re-baseline reminder (AC-176-002).

## Architecture Compliance Rules

- **ci.yml scoped edits (three total, commit ea4bcd8e):** (1) one new `run:` step in the
  `bin-selftest` job to wire `bin/test_gitignore_mutants_glob.py` (AC-176-003 regression
  guard); (2) `bin-selftest` job name de-enumerated to count-free "Bin selftest suites"
  (W75 NIT-1 count-free discipline); (3) the COMMENT block above the `green-doc-tense-gate`
  job reworded count-free (stale "10 known-bad/14 known-good" claim removed). The
  `green-doc-tense-gate` job's steps and name are untouched — only its leading comment was
  changed. `ci.yml` SHA pins are NOT modified (18/18 verified identical). No new grep
  commands or token lists belong in `ci.yml`. The action SHA-pin policy
  (CLAUDE.md §CI / Supply Chain) applies to any `ci.yml` additions — the bin-selftest step
  uses only a `run:` step (no new `uses:` action refs), so no new SHA pin is required.
- **Branch-protection rename check (F-S176P5-002 RESOLVED-CLEAN, execution-verified
  2026-07-20):** Neither the classic develop branch protection (11 required-status contexts)
  nor the develop ruleset (Test/Clippy/Format) references the `bin-selftest` job name.
  Renaming the job to "Bin selftest suites" orphans no protected branch requirement.
- **Zero false positives required:** The phrase-level patterns MUST be verified against
  the current tree (run `python3 bin/check-green-doc-tense` → exit 0) before the develop
  PR is merged. Bare-word `skeleton`/`seam` tokens are explicitly rejected: ~91 legitimate
  matches in the current tree (see `planning/story-176-ac001-validation.md` §Task 3).

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~2.5 k |
| `bin/check-green-doc-tense` (amendment target, _VIOLATION_PATTERNS extension) | ~3.0 k |
| `bin/test_check_green_doc_tense.py` (amendment target, new fixture pairs) | ~2.0 k |
| `CHANGELOG.md` (amendment target, Unreleased entry) | ~0.2 k |
| `delivery-doc-currency-protocol.md` (amendment target) | ~1.0 k |
| `.gitignore` (full file, <=15 lines) | ~0.1 k |
| `bin/test_gitignore_mutants_glob.py` (new file, git check-ignore assertions) | ~0.3 k |
| `.github/workflows/ci.yml` (bin-selftest job section only) | ~0.5 k |
| **Total** | **~9.6 k** |

Well within context window. No story split required.

## Notes

- **S-7.02 disposition:** STORY-176 v2.0 (re-scoped 2026-07-19) is the wirerust-local
  survivor of the feature-iec104 cycle-close E-11 burst. The engine-routed ACs from
  v1.0 (pre-adversarial doc sweep → drbothen/vsdd-factory#682; severity calibration →
  drbothen/vsdd-factory#686) are tracked upstream. The two absorbed ACs (AC-176-002
  from STORY-178 AC-178-003; AC-176-003 from STORY-178 AC-178-004) are product-local
  housekeeping items with zero Rust source impact.
- **DF-VALIDATION-001 gate:** PG-GATE-VOCAB-BLINDSPOT is a feature-iec104 in-process
  execution finding. DF-VALIDATION-001-exempt per the in-process exemption.
- **No behavioral contract required:** E-11 convention.
- **Develop PR:** `bin/check-green-doc-tense`/`bin/test_check_green_doc_tense.py` and
  `.gitignore` changes (Tasks 1 and 2) can be batched in a single develop PR. Task 1
  (AC-176-001) requires a CHANGELOG entry (`bin/` is in the AC-158-001 trigger set).
  Task 2 (AC-176-003, `.gitignore`) does not require a CHANGELOG entry.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.7 | 2026-07-20 | story-writer | F-S176P5-001: ci.yml scoping statements corrected to enumerate all three delivered edits (step add, job-name de-enumeration, gate-comment count-free reword); F-S176P5-002 branch-protection rename check recorded RESOLVED-CLEAN. No AC semantic change. |
| 2.6 | 2026-07-20 | story-writer | F-S176P4-001/-002/-003: AC-176-003 regression guard CI-wired (bin-selftest step; scoped ci.yml edit documented); traces_to completed (CHANGELOG.md + ci.yml); stale v2.3 Task-4 token dropped. No AC semantic change beyond guard-enforcement locus. |
| 2.5 | 2026-07-20 | story-writer | F-S176P3-001: added bin/test_gitignore_mutants_glob.py to Architecture Mapping + traces_to + AC-176-003 regression-guard note; deliverable-map completeness sweep (5/5 develop files + factory doc verified). No AC semantic change. |
| 2.4 | 2026-07-20 | story-writer | F-S176P1-008: Disposition-table AC-ID renumbering footnote; clarification only, no AC content change. |
| 2.3 | 2026-07-20 | story-writer | Spec-route remediation per planning/story-176-ac001-validation.md: corrected AC-176-001 locus to bin/check-green-doc-tense + bin/test_check_green_doc_tense.py; replaced bare-word tokens with four phrase-level zero-FP patterns; deleted fabricated # green-doc-tense-gate: allow allowlist claim; corrected CHANGELOG obligation (bin/ IS in trigger set, entry required); fixed verification commands; updated Architecture Mapping, Edge Cases, Tasks, Token Budget, Notes, Previous Story Intelligence, Architecture Compliance Rules, target_module frontmatter, and Background (v2.3 correction note + P4-seam provenance attribution note). No points/status/wave change (2 pts, ready, wave 84). |
| 2.2 | 2026-07-19 | story-writer | Remediation: added missing "Token Budget Estimate" section (per-story-delivery.md Token Budget Check). No AC or scope content change. |
| 2.1 | 2026-07-19 | story-writer | Wave-84 opening: wave TBD→84, status draft→ready (plan gate approved by human, 2026-07-19; mini-wave composition 166+176+147v2 = 7 pts, all product-local). No AC content change. |
| 2.0 | 2026-07-19 | story-writer | Re-scoped upstream 2026-07-19: ACs-176-002/003 (pre-adversarial doc sweep / severity calibration) routed to drbothen/vsdd-factory#682/#686. Absorbed STORY-178 AC-178-003 (input-hash re-baseline) → AC-176-002 and STORY-178 AC-178-004 (.gitignore mutants.out*) → AC-176-003. Points 3→2. |
| 1.0 | 2026-07-18 | story-writer | Initial authorship — feature-iec104 cycle-close S-7.02: PG-GATE-VOCAB-BLINDSPOT (AC-176-001 green-doc-tense gate skeleton/seam extension) + PG-DOC-CURRENCY-SWEEP (AC-176-002 pre-adversarial doc sweep step) + PG-ADVERSARY-SEVERITY-CALIBRATION (AC-176-003 frozen-code severity ceiling guidance). |
