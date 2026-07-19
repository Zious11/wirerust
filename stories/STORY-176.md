---
document_type: story
story_id: STORY-176
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-18T00:00:00Z
phase: f7
level: feature
cycle: feature-iec104
points: 3
priority: P3
depends_on: []
blocks: []
# BC status: E-11 convention — governance-only story; no BCs authored
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: .github/workflows/
subsystems: []
estimated_days: 1
wave: "TBD"
traces_to:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
  - .factory/maintenance/delivery-doc-currency-protocol.md
  - .github/workflows/ci.yml
inputs:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
input-hash: "a30d524"
---

# STORY-176: Feature-IEC104 Cycle-Close: Gate Vocabulary + Pre-Adversarial Doc Accuracy

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 3
**Priority:** P3

## Narrative

- **As a** spec-steward, adversary reviewer, and future contributor on the wirerust project
- **I want** three complementary process improvements codified: (1) the green-doc-tense gate
  extended to catch stub-era vocabulary ("skeleton", "seam") that survives into green
  deliveries, (2) a pre-adversarial code-comment and test-header doc sweep added to the
  delivery protocol to prevent doc-accuracy drift from consuming adversarial pass budget,
  and (3) severity calibration guidance added for adversary findings against code that has
  been frozen since earlier passes
- **So that** the residual adversarial pass count is driven by genuine behavioral findings
  rather than doc-drift findings that could have been caught in a pre-adversarial sweep,
  and so that adversary instances converge on consistent severity ratings for frozen code

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

### PG-GATE-VOCAB-BLINDSPOT — green-doc-tense gate misses stub-era vocabulary

`AC-174-008` (STORY-174) extended the green-doc-tense gate (`.github/workflows/ci.yml`)
with token patterns that should not appear in delivered ("green") code. The existing token
list catches common stale-TODO and red-gate phrasing but misses stub-era vocabulary:

- `"skeleton"` — used during VP-044 Kani harness scaffolding ("harness skeleton compiles")
  and in test scaffolding comments; stub-era wording that should not survive into a fully
  implemented story
- `"seam"` — used in test-seam discussions ("write seam accessor", "test seam") during
  red-gate phase; likewise should not appear in green-state deliveries

Two independent adversary observations on STORY-174: P2 Obs-1 (stale skeleton prose in
the story spec itself) and P4 finding (stale seam commentary in test headers). Both were
remediated in STORY-174 delivery, but the gate did not catch them automatically because
the token list was not extended.

### PG-DOC-CURRENCY-SWEEP — missing pre-adversarial doc sweep step

STORY-173 required 17 adversarial passes to converge (14 passes over two adversary
instances + 3 convergence-check passes). Post-analysis showed that 12 of the 17 passes
were driven by doc-accuracy findings (stale comments, test header prose referring to
earlier spec versions, outdated inline documentation) rather than behavioral findings.
The feature code was CONVERGED by Pass 2; the remaining 15 passes corrected documentation.

A mandatory pre-adversarial doc sweep step — checking code comments and test headers for
currency with the current BC and story spec — would have caught these before the adversary
session, reducing the adversarial pass budget to the behavioral-finding tail only.

### PG-ADVERSARY-SEVERITY-CALIBRATION — inconsistent severity on frozen code

During late adversarial passes on STORY-173 (passes P9–P14), two adversary instances
disagreed on the severity of findings against production code that had been frozen since
Pass 2 (i.e., unchanged through all subsequent passes). One instance rated unchanged
code findings as MEDIUM; another rated equivalent findings as LOW or advisory. This
divergence consumed reconciliation overhead and introduced ambiguity about whether the
pass was genuinely CLEAN or had been under-reported.

Guidance is needed: findings against code frozen since a named earlier pass should be
rated at most LOW (advisory) unless the code change would affect observable behavior
of the current HEAD. Prior pass convergence on that code establishes a baseline; a new
instance finding the same code "wrong" is re-assessing prior accepted findings, which
should be surfaced as LOW or advisory, not as MEDIUM blockers.

These are feature-iec104 cycle-execution findings — DF-VALIDATION-001-exempt per the
in-process exemption.

## Acceptance Criteria

### AC-176-001 (traces to PG-GATE-VOCAB-BLINDSPOT — extend green-doc-tense gate token list)

The green-doc-tense gate (`ci.yml` job `green-doc-tense-gate`, implemented per AC-174-008)
is extended with two additional token patterns: `skeleton` and `seam`.

(a) **New grep patterns:** The gate's grep command is extended to include `\bskeleton\b`
    and `\bseam\b` (or equivalent patterns matching these as word tokens) in the token
    list applied to `src/` and `tests/` Rust source files.

(b) **Zero false positives:** Before extending the pattern list, the implementer MUST
    run the new patterns against the current `src/` and `tests/` tree and confirm zero
    matches. If any pre-existing legitimate uses exist, they MUST be handled with the
    same `# green-doc-tense-gate: allow` allowlist mechanism established by AC-174-008,
    not by weakening the pattern.

(c) **CHANGELOG obligation:** The AC-176-001 develop PR modifies `.github/workflows/ci.yml`
    (`ci.yml` is excluded from the CHANGELOG trigger set per AC-158-001; no CHANGELOG entry
    required unless `src/` or `bin/` changes are included in the same PR).

(d) **Develop PR:** This AC requires a develop-branch PR touching `ci.yml`. The PR MUST
    pass CI including the green-doc-tense gate itself (confirming the extended patterns
    produce zero false positives on the current tree).

Verification:
```bash
# Confirm new patterns are in ci.yml
grep -n "skeleton\|seam" .github/workflows/ci.yml
# Must emit non-empty output referencing the green-doc-tense-gate job

# Confirm zero false positives in current tree
grep -rn --include="*.rs" "\bskeleton\b\|\bseam\b" src/ tests/
# Must emit empty output (or only lines with # green-doc-tense-gate: allow)
```

### AC-176-002 (traces to PG-DOC-CURRENCY-SWEEP — pre-adversarial doc sweep in delivery protocol)

`.factory/maintenance/delivery-doc-currency-protocol.md` is extended with a mandatory
pre-adversarial code-comment and test-header doc sweep step. The new step MUST:

(a) **Placement:** Appear as a named step before the "dispatch adversary" instruction,
    after implementation is complete and all CI is green.

(b) **Scope:** Require a sweep of `src/` inline comments and `tests/` test-function
    docstrings/headers for references to:
    - Prior story spec versions (e.g., "see BC-2.19.006 v1.1" when v1.2 is current)
    - Stale function names or field names from earlier BC versions
    - "todo" / "fixme" / "TODO" comments left from the implementation pass (these are
      red-gate artifacts and must be resolved before adversarial dispatch)
    - Any comment or docstring that references an AC, BC, or implementation detail
      that changed during the BC-realignment phase

(c) **Verification:** The sweep is complete when no stale cross-references are found.
    The implementer records "doc sweep: PASS" (or lists the items corrected) in the
    session checkpoint before dispatching the adversary.

(d) **Evidence rationale (PG-DOC-CURRENCY-SWEEP):** 12 of 17 STORY-173 adversarial
    passes were consumed on doc-drift corrections that a pre-adversarial sweep would have
    caught. A 1–2 hour manual sweep before adversarial dispatch saves 10+ adversarial
    passes.

Verification:
```bash
grep -n "pre-adversarial\|doc sweep\|PG-DOC-CURRENCY" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output containing the new step
```

### AC-176-003 (traces to PG-ADVERSARY-SEVERITY-CALIBRATION — severity calibration guidance)

A severity calibration note is added to `.factory/maintenance/delivery-doc-currency-protocol.md`
(or a new maintenance doc if more appropriate) covering adversary severity ratings for
findings against code frozen since a named earlier pass. The note MUST state:

(a) **Frozen-code baseline:** If production code at a given HEAD has been UNCHANGED since
    a named earlier pass (e.g., "code frozen since Pass 2"), findings against that code
    from a fresh adversary instance must acknowledge the prior pass baseline. A fresh
    instance re-assessing the same frozen code is performing a retrospective review, not
    a forward scan for new regressions.

(b) **Severity ceiling:** Findings against frozen code MUST be rated at most LOW unless
    the finding identifies an observable behavioral regression at current HEAD. If the
    behavior has been unchanged since the freeze point, MEDIUM+ severity is not
    appropriate — the finding would have been equally valid to report in the prior pass
    (when it was accepted or deferred by the earlier adversary instance).

(c) **Wording:** The adversary MUST note when a finding targets frozen-since-pass-N code
    (e.g., "code frozen since Pass 2; LOW advisory"). The reviewer can accept or defer
    LOW advisories without blocking convergence.

Verification:
```bash
grep -n "frozen.*pass\|severity.*calibration\|PG-ADVERSARY-SEVERITY" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output containing the calibration guidance
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Green-doc-tense gate token extension | `.github/workflows/ci.yml` (amend) | develop |
| Pre-adversarial doc sweep step | `.factory/maintenance/delivery-doc-currency-protocol.md` (amend) | factory-artifacts |
| Adversary severity calibration guidance | `.factory/maintenance/delivery-doc-currency-protocol.md` (amend) | factory-artifacts |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes. No `bin/` changes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `skeleton` appears in a comment legitimately (e.g., describing architecture) | The `# green-doc-tense-gate: allow` allowlist mechanism handles this; implementer must verify zero false positives before merging the ci.yml PR |
| EC-002 | Story spec (`.factory/`) files contain "skeleton" / "seam" | Gate applies to `src/` and `tests/` only; spec files are not gated |
| EC-003 | Code frozen since Pass N has a genuine behavioral bug not caught in Pass N | Severity may be MEDIUM+ if the adversary can demonstrate observable behavioral regression at current HEAD; the frozen-code ceiling does not apply to genuinely new findings |

## Tasks

1. **Extend green-doc-tense gate (AC-176-001):** Run zero-false-positive check on
   current tree, then extend ci.yml token list with `skeleton` and `seam` patterns.
   Open develop PR.

2. **Extend delivery-doc-currency-protocol.md (AC-176-002):** Add pre-adversarial doc
   sweep step before adversary dispatch instruction. Factory-artifacts branch commit.

3. **Add severity calibration guidance (AC-176-003):** Add frozen-code severity ceiling
   note to delivery-doc-currency-protocol.md. Factory-artifacts branch commit.

4. **Register in STORY-INDEX.md:** Add STORY-176 row (draft, E-11, wave-TBD).
   Factory-artifacts branch commit.

> **Note for implementer:** AC-176-001 (ci.yml change) requires a develop PR. AC-176-002
> and AC-176-003 are factory-artifacts branch commits. The ci.yml PR does NOT require a
> CHANGELOG entry (ci.yml is excluded from the AC-158-001 trigger set).

## Previous Story Intelligence

- **AC-174-008 (STORY-174, wave-83):** Established the green-doc-tense gate and its
  allowlist mechanism. This story extends the token list — the same ci.yml job and
  the same allowlist pattern apply. Implementer should read AC-174-008 before writing
  the ci.yml change to ensure the extension follows the same pattern.
- **STORY-165 AC-165-003 (wave-75):** Established `delivery-doc-currency-protocol.md`
  as the canonical delivery sweep document. STORY-176 amends the same document with
  two additional steps.

## Architecture Compliance Rules

- **ci.yml action SHA pins:** The ci.yml amendment MUST NOT change any `uses:` action
  SHA pins. Only the token list in the green-doc-tense-gate step changes.
- **Zero false positives required:** The `skeleton`/`seam` patterns must be verified
  against the current tree before the ci.yml PR is merged.

## Notes

- **S-7.02 disposition:** Creating this story at draft status codifies three
  feature-iec104 cycle-execution process gaps: PG-GATE-VOCAB-BLINDSPOT (2 independent
  adversary observations, STORY-174 P2+P4), PG-DOC-CURRENCY-SWEEP (12 of 17 STORY-173
  passes doc-accuracy), PG-ADVERSARY-SEVERITY-CALIBRATION (STORY-173 late-pass severity
  divergence).
- **DF-VALIDATION-001 gate:** All three gaps are feature-iec104 in-process execution
  findings. DF-VALIDATION-001-exempt per the in-process exemption (same pattern as
  STORY-165/166 Notes).
- **No behavioral contract required:** E-11 convention.
- **Develop/factory split:** AC-176-001 (ci.yml) → develop PR; AC-176-002/003
  (delivery-doc-currency-protocol.md amendments) → factory-artifacts branch commits.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-18 | story-writer | Initial authorship — feature-iec104 cycle-close S-7.02: PG-GATE-VOCAB-BLINDSPOT (AC-176-001 green-doc-tense gate skeleton/seam extension) + PG-DOC-CURRENCY-SWEEP (AC-176-002 pre-adversarial doc sweep step) + PG-ADVERSARY-SEVERITY-CALIBRATION (AC-176-003 frozen-code severity ceiling guidance). |
