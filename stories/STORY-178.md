---
document_type: story
story_id: STORY-178
epic_id: E-11
version: "1.0"
status: superseded
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
target_module: .factory/maintenance/
subsystems: []
estimated_days: 1
wave: "TBD"
traces_to:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
  - .factory/maintenance/delivery-doc-currency-protocol.md
  - .gitignore
inputs:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
input-hash: "62d13e0"
---

# STORY-178: Feature-IEC104 Cycle-Close: Pre-Delivery Spec Fidelity Gate

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** superseded
**Wave:** TBD
**Points:** 3
**Priority:** P3

## Narrative

- **As a** implementer, spec-steward, and story-writer on the wirerust project
- **I want** four pre-delivery discipline improvements codified: (1) a mandatory AC↔BC
  fidelity check before coding any F3/F4 story, (2) spec-version citation currency
  included in the delivery sweep set, (3) a post-delivery input-hash re-baseline step
  for stories whose delivery touches their own tracked inputs, and (4) a
  `mutants.out*/` glob added to `.gitignore` so mutation-run residue no longer lands
  untracked in the repo root
- **So that** the class of pre-delivery spec drift (4 confirmed STORY-169/170/172/173
  occurrences) is caught before coding rather than during adversarial passes, version
  references in source comments stay current with spec bumps, and the working tree
  remains clean after mutation testing runs

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored)_

## Background

### F3-DECOMPOSITION-BC-FIDELITY — mandatory pre-delivery AC↔BC fidelity check (4 occurrences, CODIFY-NOW)

Four confirmed occurrences across feature-iec104 F4 delivery where the story's ACs
drifted from the corresponding BCs between F3 decomposition and delivery time:

- **STORY-169** (wave-78): `AsduHeader` renamed to `Asdu` in BC; story ACs still
  referenced the old flat field layout and wrong minimum-length guard.
- **STORY-170** (wave-79): False-positive T0827 emission specced for C_IC/C_CI/C_CS
  interrogation TypeIDs (BC says no finding); confidence level Possible vs Likely
  mismatch; reserved-TypeID scope incorrect; naming drift.
- **STORY-172** (wave-81): `FlowId` renamed to `FlowKey` (non-existent field);
  carry-overflow discard-all-new semantics contradict BC; malformed-LEN PC4
  contradiction with BC-2.19.026.
- **STORY-173** (wave-82): T0881 tactic string `"impact"` — `MitreTactic` variant
  naming error; caused a compilation blocker on first implementation attempt.

All four were corrected via BC-realignment before coding, adding a pre-coding alignment
step to the delivery cycle. CODIFY-NOW: this pre-delivery check should be a documented
mandatory gate step, not an ad-hoc rescue operation.

### PG-SPEC-VERSION-CITATION-CURRENCY — spec-version bumps must include src/ and CHANGELOG sweep

Surfaced by F-172-301 NIT (D-454): when a BC or spec document is bumped to a new version
(e.g., BC-2.19.006 v1.1 → v1.2), `src/` inline comments and `CHANGELOG.md` entries that
cite the old version number become stale. The citation-currency sweep protocol
(`delivery-doc-currency-protocol.md`) covers `docs/` and `.factory/` artifacts but does
not explicitly include `src/` comments or `CHANGELOG` entries as citation targets.

### Input-hash self-referential drift (minor — observed STORY-164/165, re-baselined 2026-07-18)

Stories whose `inputs:` list includes the same spec files that get modified during their
delivery (e.g., a story that traces to BC-2.19.006 and whose delivery causes BC-2.19.006
to be amended) will always have a stale input-hash immediately after delivery. This is
expected and correct behavior (the hash detects drift), but the post-delivery re-baseline
step is not documented as a standard checklist item.

### .gitignore lacks `mutants.out*/` glob (minor — mutation residue lands untracked)

`.gitignore` covers `mutants-f6*/` (F6 targeted hardening runs) but not the default
`mutants.out/` and `mutants.out.j4-invalid/` directories produced by standard
`cargo mutants` runs. After any mutation testing session, these directories land as
untracked files in the repo root (confirmed in current `git status`: `mutants.out.j4-invalid/`
and `mutants.out/` both untracked). The pattern `mutants.out*/` covers both.

These are feature-iec104 cycle-execution findings — DF-VALIDATION-001-exempt per the
in-process exemption.

## Acceptance Criteria

### AC-178-001 (traces to F3-DECOMPOSITION-BC-FIDELITY — mandatory pre-delivery fidelity check)

`CLAUDE.md` and `.factory/maintenance/delivery-doc-currency-protocol.md` are updated to
codify a mandatory pre-delivery AC↔BC fidelity check as a named gate step before coding
any F3/F4 story. The documentation MUST:

(a) **Named gate step:** Add a named step "Pre-Coding AC↔BC Fidelity Check" to the
    delivery protocol. This step occurs AFTER the story is assigned to a wave and BEFORE
    any Rust source code is written.

(b) **Check scope:** For each AC in the story, the implementer MUST verify against the
    current version of the traced BC:
    - Field names match (e.g., `Asdu` not `AsduHeader`; `FlowKey` not `FlowId`)
    - Guards and conditions match (e.g., minimum length, enum variants)
    - Confidence/verdict levels match (e.g., `Possible` vs `Likely`)
    - Emit-or-no-emit decisions match (no false findings per BC invariants)

(c) **Written table:** The implementer MUST produce a written fidelity check table
    (or equivalent checklist) before coding. If discrepancies are found, the story ACs
    MUST be updated (BC-realignment) before coding begins.

(d) **CLAUDE.md reference:** A brief note is added to CLAUDE.md pointing to the
    delivery protocol for the pre-coding fidelity check obligation.

(e) **Evidence rationale:** Cite F3-DECOMPOSITION-BC-FIDELITY (4 confirmed occurrences
    STORY-169/170/172/173; corrected pre-delivery via BC-realignment; CODIFY-NOW flag
    set in STATE.md Active Carry-Forwards).

Verification:
```bash
grep -n "fidelity\|AC.*BC\|BC-realignment\|F3-DECOMPOSITION" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output containing the new gate step
```

### AC-178-002 (traces to PG-SPEC-VERSION-CITATION-CURRENCY — spec-version citation sweep)

`.factory/maintenance/delivery-doc-currency-protocol.md` is extended to include `src/`
inline comments and `CHANGELOG.md` entries in the citation-currency sweep set. The
extension MUST:

(a) **Explicit sweep targets:** Add `src/` comments and `CHANGELOG` entries to the
    list of citation targets that must be checked when a spec document version is bumped.
    The sweep trigger is any BC, PRD, ADR, or ARCH-INDEX version bump that modifies a
    document cited by existing `src/` comments.

(b) **Check command:** Provide a representative check command:
    ```bash
    grep -rn "BC-2\.19\.006 v1\." src/ CHANGELOG.md
    # Replace with the actual BC ID and old version being superseded
    ```

(c) **Scope note:** The sweep covers `src/` (inline Rust doc comments referencing spec
    versions) and `CHANGELOG.md` (entries that cite spec versions by number). It does not
    require re-writing all mentions — only updating stale version numbers.

Verification:
```bash
grep -n "src/.*comment\|CHANGELOG.*citation\|PG-SPEC-VERSION" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output containing the spec-version sweep addition
```

### AC-178-003 (minor — input-hash post-delivery re-baseline checklist step)

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

### AC-178-004 (minor — .gitignore mutants.out* glob)

`.gitignore` is updated with a `mutants.out*/` glob pattern. The change MUST:

(a) **Pattern:** Add `mutants.out*/` to `.gitignore` under the existing cargo-mutants
    section (near `mutants-f6*/`). The `*` wildcard covers `mutants.out/`,
    `mutants.out.j4-invalid/`, and any future variant of the default output directory.

(b) **Develop PR:** This change touches `.gitignore` in the project root. It is committed
    on the develop branch (no CHANGELOG entry required — `.gitignore` is not in the
    AC-158-001 trigger set).

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
| Pre-coding fidelity check step + CLAUDE.md note | `delivery-doc-currency-protocol.md` (amend) + `CLAUDE.md` (amend) | factory-artifacts / develop |
| Spec-version citation sweep addition | `delivery-doc-currency-protocol.md` (amend) | factory-artifacts |
| Input-hash post-delivery re-baseline reminder | `delivery-doc-currency-protocol.md` (amend) | factory-artifacts |
| mutants.out* glob | `.gitignore` (amend) | develop |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes. No new `bin/` tools.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Story ACs trace to multiple BCs, some of which have been revised and some have not | Fidelity check covers ALL traced BCs; implementer checks current version of each |
| EC-002 | A BC is amended after the fidelity check but before delivery | The pre-coding check is a snapshot gate; if BC changes mid-delivery, a re-check is warranted but not mandated by this story (handled by the adversarial pass) |
| EC-003 | mutants.out* directory exists with important results the implementer wants to keep | .gitignore suppresses tracking but does not delete files; results remain on disk and are simply not staged |
| EC-004 | Story inputs do not include any spec file revised during delivery | No stale hash after delivery; AC-178-003 note applies but no action required |

## Tasks

1. **Extend delivery-doc-currency-protocol.md (AC-178-001):** Add pre-coding AC↔BC
   fidelity check gate step with scope, written-table requirement, evidence rationale.
   Factory-artifacts branch commit.

2. **Update CLAUDE.md (AC-178-001d):** Add brief note pointing to delivery protocol for
   the pre-coding fidelity check. Develop branch commit (CLAUDE.md is develop-tree).

3. **Extend delivery-doc-currency-protocol.md (AC-178-002):** Add spec-version citation
   sweep extension covering src/ comments and CHANGELOG entries. Factory-artifacts commit.

4. **Extend delivery-doc-currency-protocol.md (AC-178-003):** Add input-hash
   post-delivery re-baseline reminder. Factory-artifacts commit.

5. **Update .gitignore (AC-178-004):** Add `mutants.out*/` glob under cargo-mutants
   section. Develop branch commit.

6. **Register in STORY-INDEX.md:** Add STORY-178 row (draft, E-11, wave-TBD).
   Factory-artifacts branch commit.

> **Note for implementer:** Tasks 2 and 5 (CLAUDE.md and .gitignore) are develop-branch
> changes. Tasks 1, 3, 4 are factory-artifacts branch changes. These can be batched: one
> develop PR covering AC-178-001d (CLAUDE.md) + AC-178-004 (.gitignore), and one
> factory-artifacts commit covering AC-178-001/002/003 (delivery-doc-currency-protocol.md
> amendments). Neither develop change requires a CHANGELOG entry (CLAUDE.md and .gitignore
> are not in the AC-158-001 trigger set).

## Notes

- **S-7.02 disposition:** Creating this story at draft status codifies
  F3-DECOMPOSITION-BC-FIDELITY (4 confirmed occurrences; CODIFY-NOW flag from STATE.md),
  PG-SPEC-VERSION-CITATION-CURRENCY (F-172-301 NIT D-454), and two minor housekeeping
  items (input-hash self-referential drift + .gitignore mutants.out*).
- **DF-VALIDATION-001 gate:** All gaps are feature-iec104 in-process execution findings.
  DF-VALIDATION-001-exempt per the in-process exemption.
- **No behavioral contract required:** E-11 convention.

## Disposition

**Status:** superseded — partially routed upstream 2026-07-19; product-local ACs absorbed by STORY-176

AC-178-001 and AC-178-002 are engine-level (the pre-delivery fidelity check and
citation-currency sweep are orchestrator/agent-prompt behaviors, not wirerust file changes).
AC-178-003 and AC-178-004 are product-local and have been absorbed into STORY-176
(re-scoped as the feature-iec104 cycle-close consolidation survivor).

| AC | Disposition |
|----|-------------|
| AC-178-001 (pre-delivery AC↔BC fidelity check gate) | Engine → drbothen/vsdd-factory#305 evidence comment, 2026-07-19 |
| AC-178-002 (spec-version citation currency sweep) | Engine → drbothen/vsdd-factory#396 evidence comment, 2026-07-19 |
| AC-178-003 (input-hash post-delivery re-baseline checklist) | Product-local → absorbed by STORY-176 AC-176-002 |
| AC-178-004 (.gitignore mutants.out* glob) | Product-local → absorbed by STORY-176 AC-176-003 |

Note: drbothen/vsdd-factory#690 (validate-count-propagation E-11→11 tokenizer
false-positive) was filed upstream 2026-07-19 as a new separate issue; it relates to
tooling hygiene observations in this cycle but does not map to a specific STORY-178 AC.

This story file is retained on disk for traceability. No further wirerust delivery expected.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-18 | story-writer | Initial authorship — feature-iec104 cycle-close S-7.02: F3-DECOMPOSITION-BC-FIDELITY (AC-178-001 mandatory pre-delivery fidelity check gate) + PG-SPEC-VERSION-CITATION-CURRENCY (AC-178-002 src/CHANGELOG citation sweep) + input-hash post-delivery re-baseline minor (AC-178-003) + .gitignore mutants.out* glob minor (AC-178-004). |
