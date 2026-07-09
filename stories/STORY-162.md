---
document_type: story
story_id: STORY-162
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-09T22:45:00Z
phase: f7
level: feature
cycle: wave-72
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
target_module: .factory/specs/verification-properties/
subsystems: []
estimated_days: 1
wave: "~"
traces_to:
  - .factory/cycles/wave-72/STORY-161/adversary-convergence-state.json
  - .factory/cycles/wave-72/wave-gate/code-review.md
  - .factory/specs/verification-properties/VP-INDEX.md
input-hash: "0e03ea4"
inputs:
  - .factory/cycles/wave-72/STORY-161/adversary-convergence-state.json
  - .factory/cycles/wave-72/wave-gate/code-review.md
  - .factory/specs/verification-properties/VP-INDEX.md
---

# STORY-162: Wave-72 cycle-closing: LMR-003 template-conformance exemption + check-green-doc-tense main() guard self-tests

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** ~
**Points:** 3
**Priority:** P3

## Narrative

- **As a** spec-steward, future implementer, and test-infrastructure maintainer on the
  wirerust project
- **I want** two wave-72 process gaps codified into durable project artifacts: an
  LMR-003 clarification in VP-INDEX covering hook-mandated template-conformance fields
  on locked VP documents, and additional self-test coverage in
  `bin/test_check_green_doc_tense.py` for the wave-72 runtime guards
- **So that** the LMR-003 allowlist is no longer silently silent on `inputs:` and
  `input-hash:` fields added by validation hooks to locked documents, and so that the
  zero-file exit-1 guard (line 370-376) and the `.factory/` OR-sentinel (line 361) in
  `bin/check-green-doc-tense` are covered by main()-level tests that verify exact exit
  code semantics rather than just non-zero

## Behavioral Contracts

_(none -- E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

Wave-72 (STORY-158/159/160/161, delivered 2026-07-09) surfaced two process gaps during
the per-story adversary pass for STORY-161 (F-S161P1-001) and the wave-72 integration
gate adversary Pass 2 (F-W72G-P2-OBS-001). S-7.02 (cycle-close requirement) mandates
codification of recurring process gaps as follow-up stories.

### PG-W72-LMR003-TEMPLATE-CONFORMANCE (F-S161P1-001) -- LMR-003 silent on hook-mandated template fields

During STORY-161 per-story adversary Pass 1, finding F-S161P1-001 (LOW, process-gap)
was surfaced: the LMR-003 Locked-Doc-Appendable Provenance Field Allowlist is
**closed** (currently `kani_version:` only), yet validation hooks forced two
template-conformance fields (`inputs:`, `input-hash:`) onto VP-024 v2.5 --
a locked L4 document -- when STORY-161 was implemented.

These fields are non-value-bearing and non-integrity: `inputs:` declares the list of
spec-input files driving the document's content (empty for VP-024, which has no
separate spec-input files), and `input-hash:` is the MD5-first-7 advisory drift
sentinel. Neither field affects proof correctness, harness code, postconditions, or
any BC anchor. They were added solely to satisfy the template-conformance validator
hook (per the `PG-HASH-EMPTY-INPUTS` pattern established in STORY-157).

The VP-024 v2.5 modified-log entry (item 4 of the v2.5 record) acknowledges this
as "bundled template-conformance hygiene" with a note that the empty `inputs:`
reflects that VP-024 is a governance-only doc with no separate spec-input files.
However, LMR-003 as currently written does not cover this class of field: it only
allowlists `kani_version:`, and its condition 1 states the allowlist is closed.

Source: F-S161P1-001 (wave-72 STORY-161 per-story adversary Pass 1, LOW, process-gap).
Evidence: `.factory/cycles/wave-72/STORY-161/adversary-convergence-state.json`
(F-S161P1-001 entry); VP-024 v2.5 modified-log item (4) in
`.factory/specs/verification-properties/vp-024-arp-parse-safety.md`.

Root cause: LMR-003 was authored specifically to govern the `kani_version:` case and
its allowlist was intentionally left minimal. The hook-enforcement system for
template-conformance fields (`inputs:`, `input-hash:`) was added in separate prior
stories (STORY-155/157) without a corresponding LMR-003 amendment covering the
interaction with locked documents.

### PG-W72-CGDT-MAIN-GUARDS (F-W72G-P2-OBS-001) -- check-green-doc-tense main() guards not hermetically tested

During the wave-72 integration gate adversary Pass 2, observation F-W72G-P2-OBS-001
(process-gap) was surfaced: `bin/test_check_green_doc_tense.py` does not provide
hermetic, exit-code-precise coverage of the two `main()`-level guards added to
`bin/check-green-doc-tense` by STORY-158 (AC-158-005):

**(a) Zero-file guard (lines 370-376):** `main()` returns exit code 1 when
`_collect_rust_files` returns an empty list. The existing AC-158-005 test patches
`_collect_rust_files` to return `[]` and asserts `exit_code != 0`. This passes for
exit code 1 (zero-file guard fired) AND exit code 2 (repo-root-not-found) -- the
two guards are not distinguished. In a CI environment where `.factory/` is absent
(develop checkout without factory-artifacts worktree), the repo root may not be
found, causing `main()` to return 2 and the test to pass without ever exercising the
zero-file guard at lines 370-376.

**(b) `.factory/` OR-sentinel (line 361):** `main()` accepts either a `.git` entry
or a `.factory/` directory as a valid repo-root sentinel. No test verifies this
second arm. The tool is tested in practice only in environments where `.git` is
present, and the `.factory/` sentinel is untested.

Source: F-W72G-P2-OBS-001 (wave-72 integration gate adversary Pass 2, process-gap).
Evidence: `bin/check-green-doc-tense` lines 361 and 370-376 (both new guards shipped
by STORY-158 PR #387); `bin/test_check_green_doc_tense.py` AC-158-005 test block
(lines 436-469 -- assertion `exit_code != 0`, not `exit_code == 1`).

Root cause: The AC-158-005 test block was authored during STORY-158 story-writer
convergence to assert the correct direction of behavior (non-zero exit) but was not
tightened to distinguish exit 1 from exit 2 nor to provide a hermetic fixture for
the `.factory/` sentinel arm.

## Acceptance Criteria

### AC-162-001 (traces to PG-W72-LMR003-TEMPLATE-CONFORMANCE -- VP-INDEX LMR-003 amendment)

VP-INDEX (`.factory/specs/verification-properties/VP-INDEX.md`) gains a prose
amendment to the **LMR-003** section that explicitly addresses
template-conformance provenance fields. The amendment MUST:

(a) Define the term **"template-conformance provenance fields"** as fields appended to
    a locked VP document solely to satisfy hook-mandated template conformance
    validation -- specifically `inputs:` and `input-hash:` -- that are
    non-value-bearing (empty-list or empty-inputs sentinel value only) and
    non-integrity (they do not anchor proof correctness, harness code, postconditions,
    property statements, or BC anchors; the `input-hash:` value for a VP document with
    `inputs: []` is always `d41d8cd`, the MD5 of empty bytes).

(b) Either:
    - **Option A (extend allowlist):** Add `inputs:` and `input-hash:` to the
      Locked-Doc-Appendable Provenance Field Allowlist with the permitted meaning
      "template-conformance provenance (hook-mandated; non-value-bearing; `inputs: []`
      and `input-hash: d41d8cd` only; must cite this exemption in modified-log)"; or
    - **Option B (exemption clause):** Add an explicit exemption paragraph to LMR-003
      stating that template-conformance provenance fields may be added to locked VP
      documents without appearing on the allowlist, provided: (1) both fields are
      present together (`inputs:` and `input-hash:` are co-required); (2) `inputs:`
      MUST be `[]` (empty list) for a locked VP document (no new spec-input
      dependencies may be introduced via a locked-doc amendment); (3) `input-hash:`
      MUST be `d41d8cd` (the canonical empty-inputs hash); (4) a modified-log entry
      cites this exemption.

    The implementer picks exactly ONE option and applies it consistently. If Option A
    is chosen, the allowlist table gains two new rows (one for `inputs:`, one for
    `input-hash:`). If Option B is chosen, the exemption paragraph is placed
    immediately after LMR-003 condition 1.

(c) Cite VP-024 v2.5 as the confirming precedent: "First application: VP-024 v2.5
    (STORY-161/162, wave-72) -- `inputs: []` and `input-hash: d41d8cd` added as
    bundled template-conformance hygiene per STORY-162 AC-162-001."

Verification:
```bash
grep -n "template-conformance\|inputs:\|input-hash:" \
  .factory/specs/verification-properties/VP-INDEX.md
```
must emit non-empty output containing the amendment text.

### AC-162-002 (traces to PG-W72-LMR003-TEMPLATE-CONFORMANCE -- VP-INDEX version bump)

VP-INDEX version is bumped from `"2.39"` to `"2.40"` and the `modified:` field
updated. The modification record MUST cite STORY-162 and F-S161P1-001.

Verification:
```bash
grep "^version:" .factory/specs/verification-properties/VP-INDEX.md
```
must produce `version: "2.40"`.

### AC-162-003 (traces to PG-W72-CGDT-MAIN-GUARDS -- zero-file guard exit-code precision)

`bin/test_check_green_doc_tense.py` gains a new test that exercises `main()` with
`_collect_rust_files` patched to return `[]` in a hermetic environment where the
repo root IS reliably found, and asserts `exit_code == 1` (not merely `!= 0`). This
distinguishes the zero-file guard (exit 1, lines 370-376) from the repo-root-not-found
guard (exit 2, lines 365-367).

The hermetic fixture MUST use `tempfile.TemporaryDirectory()` to create a directory
tree where the tool can find a repo root, following the pattern established in
`bin/test_compute_input_hash.py` (use `WIRERUST_REPO_ROOT` override or equivalent
technique to make the tool's repo-root detection deterministic). Tests MUST NOT rely
on the live `.factory/` or `.git` of the develop checkout (these may be absent in CI).

The test label MUST reference F-W72G-P2-OBS-001 in its description string.

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
```
must pass, with the new test present in the output and labeled with F-W72G-P2-OBS-001.

### AC-162-004 (traces to PG-W72-CGDT-MAIN-GUARDS -- .factory/ OR-sentinel hermetic test)

`bin/test_check_green_doc_tense.py` gains a test that specifically exercises the
`.factory/` OR-sentinel arm at `bin/check-green-doc-tense` line 361. The test MUST:

(a) Use `tempfile.TemporaryDirectory()` to create a temporary tree containing a
    `.factory/` subdirectory but NO `.git` directory at the same level.
(b) Verify that the repo-root detection in `main()` (or an exposed `_find_repo_root`
    helper, if the implementer chooses to extract it) resolves the repo root via the
    `.factory/` sentinel rather than `.git`.
(c) NOT rely on the tool's own hardcoded `Path(__file__).resolve()` without override,
    since that path is not under the temp directory. One of the following approaches
    is acceptable:
    - Extract the repo-root-walking logic into a helper function
      `_find_repo_root(start: Path) -> Path | None` and test it directly;
    - Monkey-patch `Path(__file__).resolve` in `main()` to return a path inside the
      temp tree; or
    - Test the guard by calling `main()` with `WIRERUST_REPO_ROOT` set to a
      temporary directory, and verify that `.factory/` is accepted while a path with
      no sentinel returns exit 2.

The test label MUST reference F-W72G-P2-OBS-001.

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
```
must pass, with the new test present in the output and labeled with F-W72G-P2-OBS-001.

### AC-162-005 (PR type)

The pull request title uses the `docs:` semantic prefix (e.g.,
`docs: LMR-003 template-conformance exemption + check-green-doc-tense guard tests`),
consistent with the primary deliverable being a VP-INDEX governance amendment.
The `bin/test_check_green_doc_tense.py` additions are supporting test changes; `docs:`
is correct when the principal change is governance documentation (no production Rust
changed, no new CI gate added).

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| LMR-003 amendment | `.factory/specs/verification-properties/VP-INDEX.md` (amend) | Documentation |
| Main-guard tests | `bin/test_check_green_doc_tense.py` (amend) | Pure (test-only) |

No Rust source files, no tests in `tests/`, no CI configuration.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `VP-INDEX.md` | Documentation artifact | Governance prose; no code |
| `bin/test_check_green_doc_tense.py` | Pure (test-only) | In-memory test assertions; no I/O side effects |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `inputs:` on locked VP is `[]` (empty) | Explicitly allowed by the amendment -- non-value-bearing; must cite exemption in modified-log |
| EC-002 | `inputs:` on locked VP is non-empty (e.g., has spec-input file paths) | NOT covered by AC-162-001 exemption -- a non-empty `inputs:` introduces new spec-input dependencies onto a locked doc; requires full re-lock or a separate governance ruling |
| EC-003 | `input-hash:` on locked VP is NOT `d41d8cd` (i.e., hashes a non-empty input list) | NOT covered by AC-162-001 exemption -- same rationale as EC-002 |
| EC-004 | VP-INDEX amended while VP-024 is locked (`verification_lock: true`) | No VP-024 change needed -- VP-024 v2.5 already records the precedent; VP-INDEX amendment is an addition to the governance rules, not a change to VP-024 |
| EC-005 | `.factory/` sentinel test run in CI where `.factory/` is absent | Test MUST be hermetic -- uses `tempfile.TemporaryDirectory()` with controlled tree; must not rely on live `.factory/` directory; CI-safe on develop checkouts |
| EC-006 | Future locked VP receives `inputs:` with a non-empty list | This is an expansion of the exemption's scope, which AC-162-001 option B(2) explicitly forbids. Any non-empty `inputs:` on a locked VP requires a new governance ruling beyond this story |

## Tasks

1. **Amend VP-INDEX LMR-003 section (AC-162-001/002):** Read the LMR-003 section of
   VP-INDEX.md in full. Choose Option A or Option B for the template-conformance
   exemption. Draft the amendment, cite VP-024 v2.5 as precedent. Bump VP-INDEX version
   from `"2.39"` to `"2.40"`. Update the `modified:` field with a new entry citing
   STORY-162 and F-S161P1-001.

2. **Add zero-file exit-1 precision test (AC-162-003):** In
   `bin/test_check_green_doc_tense.py`, add a new test block after the existing
   AC-158-005 block. The test must create a hermetic fixture (using
   `tempfile.TemporaryDirectory()`), control the repo-root environment so the tool
   reliably finds a repo root, patch `_collect_rust_files` to return `[]`, call
   `mod.main()`, and assert `exit_code == 1` (exact value, not just `!= 0`).
   Label the test block clearly with `F-W72G-P2-OBS-001`.

3. **Add `.factory/` OR-sentinel hermetic test (AC-162-004):** In
   `bin/test_check_green_doc_tense.py`, add a test that creates a temp tree with only
   `.factory/` as the repo-root sentinel (no `.git`). Use one of the three approaches
   listed in AC-162-004(c) to make the tool's repo-root detection deterministic against
   the temp tree. Verify the `.factory/` arm resolves the repo root correctly. Label
   the test block with `F-W72G-P2-OBS-001`.

4. **Verify no product code changes.** Confirm zero changes to `src/`, `tests/`,
   `.github/`, or `Cargo.toml`. The diff must touch only
   `.factory/specs/verification-properties/VP-INDEX.md` (factory-artifacts branch) and
   `bin/test_check_green_doc_tense.py` (develop branch).

> **Note for implementer:** VP-INDEX.md lives on the `factory-artifacts` branch and
> cannot appear in the `develop`-targeted PR. Commit the VP-INDEX amendment to
> `factory-artifacts` in the same delivery burst as the develop PR (which carries only
> `bin/test_check_green_doc_tense.py`). Do NOT include `.factory/` paths in the develop
> PR diff.

## Previous Story Intelligence

Lessons from analogous governance/tooling stories in E-11:

- **STORY-161 (wave-72, E-11, 3 pts):** VP governance amendment (VP-INDEX + VP-024 +
  CLAUDE.md). Pattern: read the existing VP-INDEX section in full before amending; bump
  version and update modified: field; document the primary evidence (VP-024 v2.5) as
  the confirming precedent.
- **STORY-158 (wave-72, E-11, 3 pts):** Added the zero-file guard to check-green-doc-tense
  and the AC-158-005 test. The STORY-162 tests build on this; do not re-implement what
  AC-158-005 already covers -- add precision and the missing `.factory/` sentinel arm.
- **bin/test_compute_input_hash.py:** The canonical pattern for hermetic tests in
  `bin/` tools using `tempfile.TemporaryDirectory()` + `WIRERUST_REPO_ROOT` override.
  Follow this pattern for AC-162-003 and AC-162-004.

## Architecture Compliance Rules

- This story modifies ONLY: `VP-INDEX.md` (factory-artifacts branch) and
  `bin/test_check_green_doc_tense.py` (develop branch).
  No production Rust, no CI YAML, no CLAUDE.md, no Cargo.toml.
- The VP-INDEX amendment is an additive governance change to LMR-003. No existing LMR
  rules, no VP content, and no BC anchors are modified.
- `bin/test_check_green_doc_tense.py` tests MUST be hermetic per AC-162-003/004 -- no
  live `.factory/` or `.git` references; CI-safe on develop checkouts.
- Python 3 standard library only -- no third-party dependencies in new test code.

## Library & Framework Requirements

- Python 3 standard library only (`tempfile`, `pathlib`, `importlib`, `subprocess`)
  for test additions in `bin/test_check_green_doc_tense.py`.
- No new library versions required. No Rust toolchain changes.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/specs/verification-properties/VP-INDEX.md` | Modify | Add template-conformance exemption to LMR-003; bump version 2.39→2.40; factory-artifacts branch |
| `bin/test_check_green_doc_tense.py` | Modify | Add AC-162-003 and AC-162-004 hermetic main()-guard tests; develop branch |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~3 k |
| VP-INDEX LMR-003 section in context (~60 lines relevant) | ~0.5 k |
| `bin/test_check_green_doc_tense.py` (existing + new tests) | ~1 k |
| `bin/check-green-doc-tense` lines 354-380 for context | ~0.3 k |
| **Total** | **~4.8 k** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** Both process gaps originate from wave-72 in-process
  findings: F-S161P1-001 from the wave-72 STORY-161 per-story adversary Pass 1 (sourced
  from `adversary-convergence-state.json`), and F-W72G-P2-OBS-001 from the wave-72
  integration gate adversary Pass 2 (wave-gate in-process finding). Both are
  sweep-validated in-process wave-gate findings -- DF-VALIDATION-001-exempt per the
  in-process exemption (same pattern as STORY-159 Notes, same pattern as STORY-158 Notes
  for wave-71 adversarial review findings).
- **S-7.02 disposition:** Creating this story at draft status codifies two wave-72
  process-gap findings (F-S161P1-001, F-W72G-P2-OBS-001) for the S-7.02 wave-72
  cycle-close obligation.
- **No behavioral contract required:** E-11 convention (epics.md E-11: "BCs: none
  authored yet -- status: draft; pending PO authorship").
- **VP-024 v2.5 is the key evidence.** The v2.5 modified-log item (4) ("bundled
  template-conformance hygiene") is the first instance of `inputs:` and `input-hash:`
  being added to a locked VP document. AC-162-001 codifies the governance rule that
  makes this a sanctioned pattern (rather than a one-off undocumented action).
- **LMR-003 allowlist is closed by design.** AC-162-001 does NOT change this: Option A
  adds new rows to an explicitly limited table with well-defined conditions; Option B
  adds a bounded exemption clause. In either case, non-empty `inputs:` values on locked
  VPs remain out-of-scope (EC-002/003).
- **Precedent:** STORY-162 follows the same E-11 pattern: cycle process-gap follow-up
  encoding lessons into project governance and tooling (STORY-157 → wave-70; STORY-158
  → wave-71; STORY-162 → wave-72).

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-09 | story-writer | Initial authorship -- wave-72 process-gap codifications: PG-W72-LMR003-TEMPLATE-CONFORMANCE (F-S161P1-001, VP-INDEX LMR-003 amendment) + PG-W72-CGDT-MAIN-GUARDS (F-W72G-P2-OBS-001, check-green-doc-tense main() guard tests). S-7.02 wave-72 cycle-close. |
