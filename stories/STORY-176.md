---
document_type: story
story_id: STORY-176
epic_id: E-11
version: "2.2"
status: ready
producer: story-writer
timestamp: 2026-07-19T00:00:00Z
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
target_module: .github/workflows/
subsystems: []
estimated_days: 1
wave: "84"
traces_to:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
  - .factory/maintenance/delivery-doc-currency-protocol.md
  - .github/workflows/ci.yml
  - .gitignore
inputs:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
input-hash: "4aef9c8"
---

# STORY-176: Feature-IEC104 Cycle-Close: Local Gate + Tooling Hygiene Sweeps

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** ready
**Wave:** 84
**Points:** 2
**Priority:** P3

## Narrative

- **As a** spec-steward, adversary reviewer, and future contributor on the wirerust project
- **I want** two product-local tooling improvements codified: (1) the green-doc-tense CI gate
  extended with stub-era vocabulary tokens `skeleton` and `seam`, and (2) minor housekeeping
  items applied to the repo — an input-hash post-delivery re-baseline reminder in the delivery
  protocol and a `mutants.out*/` glob added to `.gitignore`
- **So that** stub-era wording is caught by CI before adversarial review, the delivery
  checklist includes the hash re-baseline step, and mutation-testing residue no longer lands
  untracked in the repo root

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

This is a feature-iec104 cycle-execution finding — DF-VALIDATION-001-exempt per the
in-process exemption.

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
    (`ci.yml` is excluded from the CHANGELOG trigger set per AC-158-001; no CHANGELOG
    entry required unless `src/` or `bin/` changes are included in the same PR).

(d) **Develop PR:** This AC requires a develop-branch PR touching `ci.yml`. The PR MUST
    pass CI including the green-doc-tense gate itself (confirming the extended patterns
    produce zero false positives on the current tree). AC-176-001 SHOULD be batched with
    AC-176-003 in a single develop PR.

Verification:
```bash
# Confirm new patterns are in ci.yml
grep -n "skeleton\|seam" .github/workflows/ci.yml
# Must emit non-empty output referencing the green-doc-tense-gate job

# Confirm zero false positives in current tree
grep -rn --include="*.rs" "\bskeleton\b\|\bseam\b" src/ tests/
# Must emit empty output (or only lines with # green-doc-tense-gate: allow)
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

Verification:
```bash
grep "mutants.out" .gitignore
# Must emit: mutants.out*/
# (in addition to existing mutants-f6*/)
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Green-doc-tense gate skeleton/seam token extension | `.github/workflows/ci.yml` (amend) | develop |
| Input-hash post-delivery re-baseline reminder | `.factory/maintenance/delivery-doc-currency-protocol.md` (amend) | factory-artifacts |
| mutants.out* glob | `.gitignore` (amend) | develop |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes. No `bin/` changes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `skeleton` appears in a comment legitimately (e.g., describing architecture) | The `# green-doc-tense-gate: allow` allowlist mechanism handles this; implementer must verify zero false positives before merging the ci.yml PR |
| EC-002 | Story spec (`.factory/`) files contain "skeleton" / "seam" | Gate applies to `src/` and `tests/` only; spec files are not gated |
| EC-003 | Story inputs do not include any spec file revised during delivery | No stale hash after delivery; AC-176-002 note applies but no action required |
| EC-004 | mutants.out* directory exists with important results the implementer wants to keep | .gitignore suppresses tracking but does not delete files; results remain on disk and are simply not staged |

## Tasks

1. **Extend green-doc-tense gate (AC-176-001):** Run zero-false-positive check on
   current tree, then extend ci.yml token list with `skeleton` and `seam` patterns.
   Batch with AC-176-003 in one develop PR (no CHANGELOG entry required).

2. **Update .gitignore (AC-176-003):** Add `mutants.out*/` glob under cargo-mutants
   section. Batch with AC-176-001 in the same develop PR.

3. **Extend delivery-doc-currency-protocol.md (AC-176-002):** Add input-hash
   post-delivery re-baseline reminder. Factory-artifacts branch commit.

4. **Register in STORY-INDEX.md:** Update STORY-176 row (v2.0, draft, E-11, wave-TBD,
   2 pts). Factory-artifacts branch commit.

> **Note for implementer:** Tasks 1 and 2 (ci.yml and .gitignore) are develop-branch
> changes that can be batched in one PR (no CHANGELOG entry required for either).
> Task 3 is a factory-artifacts branch commit.

## Previous Story Intelligence

- **AC-174-008 (STORY-174, wave-83):** Established the green-doc-tense gate and its
  allowlist mechanism. AC-176-001 extends the same ci.yml job with two more tokens —
  read AC-174-008 before writing the ci.yml change to match the existing pattern.
- **STORY-165 AC-165-003 (wave-75):** Established `delivery-doc-currency-protocol.md`
  as the canonical delivery sweep document. STORY-176 amends the same document with
  the input-hash re-baseline reminder (AC-176-002).

## Architecture Compliance Rules

- **ci.yml action SHA pins:** The ci.yml amendment MUST NOT change any `uses:` action
  SHA pins. Only the token list in the green-doc-tense-gate step changes.
- **Zero false positives required:** The `skeleton`/`seam` patterns must be verified
  against the current tree before the ci.yml PR is merged.

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~2.0 k |
| `.github/workflows/ci.yml` (green-doc-tense-gate job section, amendment target) | ~1.0 k |
| `delivery-doc-currency-protocol.md` (amendment target) | ~1.0 k |
| `.gitignore` (full file, <=15 lines) | ~0.1 k |
| **Total** | **~4.1 k** |

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
- **Develop PR:** Both ci.yml and .gitignore changes (Tasks 1 and 2) can be batched in
  a single develop PR. Neither requires a CHANGELOG entry.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.2 | 2026-07-19 | story-writer | Remediation: added missing "Token Budget Estimate" section (per-story-delivery.md Token Budget Check). No AC or scope content change. |
| 2.1 | 2026-07-19 | story-writer | Wave-84 opening: wave TBD→84, status draft→ready (plan gate approved by human, 2026-07-19; mini-wave composition 166+176+147v2 = 7 pts, all product-local). No AC content change. |
| 2.0 | 2026-07-19 | story-writer | Re-scoped upstream 2026-07-19: ACs-176-002/003 (pre-adversarial doc sweep / severity calibration) routed to drbothen/vsdd-factory#682/#686. Absorbed STORY-178 AC-178-003 (input-hash re-baseline) → AC-176-002 and STORY-178 AC-178-004 (.gitignore mutants.out*) → AC-176-003. Points 3→2. |
| 1.0 | 2026-07-18 | story-writer | Initial authorship — feature-iec104 cycle-close S-7.02: PG-GATE-VOCAB-BLINDSPOT (AC-176-001 green-doc-tense gate skeleton/seam extension) + PG-DOC-CURRENCY-SWEEP (AC-176-002 pre-adversarial doc sweep step) + PG-ADVERSARY-SEVERITY-CALIBRATION (AC-176-003 frozen-code severity ceiling guidance). |
