---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-26T21:00:00Z
cycle: "wave-086"
pass: 10
verdict: NOT_CONVERGED
novelty: "substantive-narrow"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 10

**Date:** 2026-07-26
**Pass:** 10 of N
**Verdict:** NOT CONVERGED
**Novelty:** substantive-narrow — 3 of 5 MEDs are pass-9-induced propagation gaps; adversary assessed both story designs sound, no redesign needed, ~1 more burst expected
**Tally:** 11 findings — 0 CRIT / 0 HIGH / 5 MED / 6 LOW + 5 NITs
**Status:** REMEDIATED — D-527 state burst; STORY-182 v1.9→v2.0 + STORY-183 v1.9→v2.0. FIRST ZERO-HIGH PASS of wave-86.
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS.
**Positives from adversary:** All line/hash/count/test-name citations verified correct including 42-violation arithmetic and 13 rename sites; deferred TIER-2 sites compatible with zero-FP requirement; additive ci.yml step a genuine gate; no collateral test breakage from new committed capture; DF-GREEN-DOC-TENSE-SWEEP v6 / DF-TEST-NAMESPACE-001 / DF-AC-TEST-NAME-SYNC-001 / DF-INPUT-HASH-CANONICAL-001 compliance confirmed; DF-CANONICAL-FRAME-HOLDOUT-001 n/a.

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P10-001 | MED | STORY-182 | FIXED v2.0 | .gitignore modification (coverage-out.txt) not propagated to Arch Mapping/FSR/traces_to/Task-11 PR list/Notes §Develop PR; contradicted by Background :153-157 "no .gitignore change is required" |
| F-W86S-P10-002 | MED | STORY-183 | FIXED v2.0 | src/*.rs glob widening (pass-9 F-009) not propagated to 3 scope-prose sweep instructions (Task 2 :808-812; Task 10 :931-934 and :939-943) |
| F-W86S-P10-003 | MED | STORY-182 | FIXED v2.0 | Non-vacuous verification blocks at :441-445 and :933-937 use `grep -c ... \|\| true` — structurally unfailable; doubly inert in AC-182-003 form |
| F-W86S-P10-004 | MED | STORY-182 | FIXED v2.0 | Prescribed fixture_present() (display-path construction) contradicts ACR :1009-1012 "No independent path construction" |
| F-W86S-P10-005 | MED | STORY-182 | FIXED v2.0 | No negative guard asserting non-committed manifest entries absent from tests/fixtures/ (licensing/redistribution exposure) |
| F-W86S-P10-006 | LOW | STORY-182 | FIXED v2.0 | size-gate truncate/re-derive branch contradicts pinned sha256 + "content unmodified" attribution |
| F-W86S-P10-007 | LOW | STORY-182 | FIXED v2.0 | ci.yml grep `[1-9]/4` hardcoded denominator = silent third locus coupled to manifest size |
| F-W86S-P10-008 | LOW | STORY-182 | FIXED v2.0 | Background "ALL committed fixtures must have a row" overclaims 25-row backfill |
| F-W86S-P10-009 | LOW | STORY-182 | FIXED v2.0 | "genuine per-test coupling" overclaim (registry catches renames, not unregistered additions) |
| F-W86S-P10-010 | LOW [process-gap] | BOTH | FIXED v2.0 | tdd_mode: strict with task ordering that never produces an automated RED observation (E-11 template pattern) |
| F-W86S-P10-011 | LOW | STORY-183 | FIXED v2.0 | collector self-test assertions underspecified (repo_root derivation implicit + mitre.rs-literal fragility) |

---

## Findings Detail

### F-W86S-P10-001 (MED) — .gitignore modification propagation gap
**Story:** STORY-182
**Status:** FIXED v2.0
**Description:** Task 10b adds `coverage-out.txt` to `.gitignore`. This change was not propagated
to: (1) Arch Mapping table (no row for `.gitignore`); (2) FSR table (same omission);
(3) `traces_to:` frontmatter (`.gitignore` not listed); (4) Task-11 PR list (`.gitignore`
missing from files-changed list); (5) Notes §Develop PR (same omission). Additionally, the
Background section at :153-157 contained the contradicting text "no .gitignore change is
required", which was an unpropagated residue of the pass-9 F-011 fix that added Task 10b.
This is a classic propagation gap: the task was added but the 5 structural loci that reference
file-change scope were not swept.

---

### F-W86S-P10-002 (MED) — src/*.rs glob widening not propagated to scope-prose
**Story:** STORY-183
**Status:** FIXED v2.0
**Description:** Pass-9 F-009 (DRIFT-src-glob-blindspot resolution) added `src/*.rs` alongside
`src/**/*.rs` to the tool's glob list. This change was not propagated to 3 scope-prose sweep
instructions that enumerate the globs used by the tool:
- Task 2 :808-812 (rename-sweep prose)
- Task 10 :931-934 (scope description)
- Task 10 :939-943 (equivalent second locus)
These 3 loci would cause the delivered docstrings and ci.yml comments to misdocument the
actual shipped pathspec (listing `src/**/*.rs` only, omitting `src/*.rs`, `src/**/*.rs`,
`bin/*.py`, and `tests/**/*.rs` four-glob enumeration).

---

### F-W86S-P10-003 (MED) — verification blocks structurally unfailable
**Story:** STORY-182
**Status:** FIXED v2.0
**Location:** :441-445 and :933-937
**Description:** Two verification blocks labelled "Non-vacuous" use `grep -c ... || true`. The
`|| true` suffix suppresses the non-zero exit from `grep -c` when the count is zero — making
the block exit 0 regardless of whether zero or N matches were found. This is structurally
unfailable: the "verification" passes unconditionally.
The defect is doubly inert in AC-182-003's form because the pattern was scoped such that the
`dissect SKIP` event it checks is unreachable under the story's stated conditions. Both blocks
were prescribed as genuine gates but cannot gate anything.

---

### F-W86S-P10-004 (MED) — display-path construction contradicts ACR no-independent-path-construction
**Story:** STORY-182
**Status:** FIXED v2.0 per orchestrator ruling
**Location:** ACR :1009-1012
**Description:** The prescribed `fixture_present()` helper constructs a display path by
concatenating `tests/fixtures/` + filename. This path construction contradicts the ACR at
:1009-1012 which reads "No independent path construction" — a constraint introduced to prevent
the harness from hard-coding fixture locations that may change.
**Orchestrator ruling:** ACR constraint scoped to RESOLVING OR OPENING fixtures (functional
path construction). Display-only path for SKIP diagnostic is explicitly permitted — the
display string is not used to open or resolve any file.

---

### F-W86S-P10-005 (MED) — no negative guard for non-committed manifest entries
**Story:** STORY-182
**Status:** FIXED v2.0
**Description:** STORY-182 prescribes that the fixture manifest lists the committed captures.
Nothing in the AC set asserts that non-committed manifest entries are ABSENT from
`tests/fixtures/`. This matters because the SKIP diagnostic names `tests/fixtures/` as the
expected path for the forbidden file — if a developer commits a forbidden file there by mistake,
the registry would silently succeed. The story has no negative-guard AC that fires when a
listed-but-forbidden file is found committed. This creates a licensing/redistribution exposure:
if a file that should NOT be committed ends up in `tests/fixtures/`, neither the harness nor CI
detects it.

---

### F-W86S-P10-006 (LOW) — size-gate truncate/re-derive branch contradicts pinned sha256
**Story:** STORY-182
**Status:** FIXED v2.0
**Description:** The story's size-gate specifies a truncate-and-re-derive fallback path for
files exceeding the size limit. This contradicts the pinned sha256 hash and the "content
unmodified" attribution claim: if the file is truncated, its sha256 diverges from the pinned
value and the attribution is no longer accurate. The story does not acknowledge this
contradiction or specify what happens to the sha256 record on the truncation path.

---

### F-W86S-P10-007 (LOW) — ci.yml grep denominator hardcoded
**Story:** STORY-182
**Status:** FIXED v2.0
**Description:** The ci.yml grep pattern `[1-9]/4` uses a hardcoded denominator of `4`
(matching "N/4" for any nonzero N). This pattern is a third locus coupled to the manifest
size: if the manifest grows to 5 entries, the grep silently stops matching. The denominator
should be loosened to `[1-9][0-9]*/[0-9]+` to be manifest-size-agnostic. Additionally, the
assertion message and ACR name reference the denominator implicitly, creating co-update
obligations if the manifest ever changes.

---

### F-W86S-P10-008 (LOW) — Background overclaims 25-row backfill scope
**Story:** STORY-182
**Status:** FIXED v2.0
**Description:** The Background section's statement "ALL committed fixtures must have a row"
overclaims the scope of the backfill obligation. The story's actual obligation is to add rows
for the fixtures committed BY THIS STORY (the IEC-104 captures). The 25-row figure cited as
the "complete" backfill count includes historical fixtures predating this story whose coverage
the story does not actually deliver. The overclaim would set a false expectation that the story
retroactively covers all 25 historical fixtures.

---

### F-W86S-P10-009 (LOW) — "genuine per-test coupling" overclaim
**Story:** STORY-182
**Status:** FIXED v2.0
**Description:** The story claims the `fixture_present()` registry provides "genuine per-test
coupling" — the notion that every test is coupled to the fixture it needs. This overclaims
the registry's enforcement power: the registry catches RENAMES (a renamed fixture file no
longer satisfies a `fixture_present("old_name")` call), but does NOT catch UNREGISTERED
ADDITIONS (a new test that opens a fixture without calling `fixture_present()` is invisible
to the registry). The registry is an incomplete coupling mechanism, and calling it "genuine
per-test coupling" overstates its guarantees.

---

### F-W86S-P10-010 (LOW) [process-gap] — tdd_mode: strict with task ordering precluding automated RED
**Stories:** STORY-182, STORY-183
**Status:** FIXED v2.0 per orchestrator ruling
**Description:** Both stories carry `tdd_mode: strict` but their task orderings never produce
an automated RED observation. In E-11 template stories, the ACs assert against already-green
artifacts (fixtures, registry entries, ci.yml configurations) — the test harness cannot produce
a RED state mechanically because the subject of the test already exists. This is the E-11
template pattern also observed in STORY-176.
**Orchestrator ruling:** The E-11 tdd_mode convention is accepted: manual RED demonstration
(developer removes/corrupts the artifact, observes test failure, restores) is the accepted
substitute for automated RED. No task reorder required. An explicit E-11 template note added
to both stories (STORY-182 v2.0 + STORY-183 v2.0) documenting the convention.
**Process-gap note:** PG-W86-013 added — codification candidate at wave-86 cycle-close (S-7.02).

---

### F-W86S-P10-011 (LOW) — collector self-test assertions underspecified
**Story:** STORY-183
**Status:** FIXED v2.0
**Description:** The collector self-test asserts that `mod._find_repo_root()` returns a valid
path, but the invocation form is not specified — the repo_root derivation is left implicit.
Additionally, the positive-coverage assertion `any(Path(f).suffix == ".rs" for f in files)`
is fragile: it would pass even if `mitre.rs` were the only `.rs` file returned, or if a
non-`src/` `.rs` file were included. The assertion needs an explicit derivation form and a
structural assertion that checks `any(p.parent.name == "src" and p.suffix == ".rs" ...)`.

---

## NIT-Observations (5 items — deliberately not actioned, churn avoidance)

1. **NIT-01:** Token-budget estimates in story header comments may not reflect v2.0 body size.
2. **NIT-02:** STORY-183 AC-183-003 partition/ratio wording at :917 is ambiguous between
   "fraction of tests" and "fraction of patterns".
3. **NIT-03:** STORY-182 `.gitignore` entry style is unanchored (bare `coverage-out.txt`
   rather than `/coverage-out.txt` repo-root-scoped). Minor style only; both forms work.
4. **NIT-04:** STORY-182 exit-0 wording at :224 could be clarified ("exits 0" vs
   "does not exit non-zero") — same semantics, different framing.
5. **NIT-05:** STORY-183 falls-through subset labelling in the GOOD_CASES section does
   not enumerate which GOOD_CASEs cover the "falls through" vs "Expected RED:" class.

All 5 NITs deliberately unfixed to avoid churn. Recorded for completeness only.

---

## Adversary Positive Observations

- All line/hash/count/test-name citations verified correct including 42-violation arithmetic
  and 13 rename sites in STORY-183.
- Deferred TIER-2 sites are compatible with the zero-FP requirement.
- Additive ci.yml step is a genuine gate (not redundant with existing coverage).
- No collateral test breakage from the new committed capture (iec104-iti-diverse.pcap).
- DF-GREEN-DOC-TENSE-SWEEP v6 compliance confirmed (no regressions from v2.0 edits).
- DF-TEST-NAMESPACE-001 compliance confirmed.
- DF-AC-TEST-NAME-SYNC-001 compliance confirmed.
- DF-INPUT-HASH-CANONICAL-001 compliance confirmed (9a0f34c / 9c9b12f canonical values).
- DF-CANONICAL-FRAME-HOLDOUT-001: n/a (E-11 template stories).
- Both story designs assessed sound by adversary; no redesign needed.
- Adversary assessment: ~1 more burst expected to achieve 3-clean streak.

---

## Pass-10 Verdict

**NOT CONVERGED.** Streak: 0/3. FIRST ZERO-HIGH PASS of wave-86 (P1–P10).
Novelty: substantive but narrow — 3 of 5 MEDs are pass-9-induced propagation gaps.
Pass-11 next after D-527 remediation burst.

Pass tallies (P1–P10): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11.
Total across all passes: 189 findings. Canonical hashes: 9a0f34c / 9c9b12f.

---

## Remediation (D-527)

**Date:** 2026-07-26
**Burst:** D-527 STATE BURST — WAVE-86 ADVERSARIAL PASS 10 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 11 findings FIXED at STORY-182 v2.0 / STORY-183 v2.0 with per-fix grep evidence
per PG-W86-010 mandate. Full DF-SIBLING-SWEEP-001 sweep performed.

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P10-001 (MED) | FIXED v2.0 | STORY-182 | Background :153-157 rewritten; .gitignore added to Arch Mapping + FSR + traces_to + Task-11 PR list + Notes §Develop PR (all 5 structural loci) |
| F-W86S-P10-002 (MED) | FIXED v2.0 | STORY-183 | All 3 scope-prose sweep loci (Task 2 :808-812; Task 10 :931-934 and :939-943) updated to enumerate all four globs: src/*.rs src/**/*.rs bin/*.py tests/**/*.rs |
| F-W86S-P10-003 (MED) | FIXED v2.0 | STORY-182 | Both verification blocks at :441-445 and :933-937 replaced with gating `test "$(grep -c ...)" -eq 0` tee-to-file form; pattern widened to any `[iec104-e2e] SKIP:` |
| F-W86S-P10-004 (MED) | FIXED v2.0 per orch. ruling | STORY-182 | ACR :1009-1012 scoped: constraint applies to RESOLVING OR OPENING fixtures only; display-only path for SKIP diagnostic explicitly permitted per D-527 ruling |
| F-W86S-P10-005 (MED) | FIXED v2.0 | STORY-182 | Forbidden-committed negative-guard loop added to AC-182-005: iterates non-committed manifest entries, asserts each absent from tests/fixtures/ with LICENSING/REDISTRIBUTION VIOLATION panic message + diagnostic prose note |
| F-W86S-P10-006 (LOW) | FIXED v2.0 | STORY-182 | size-gate truncate/re-derive branch marked UNREACHABLE IN PRACTICE (14 KB recorded, E2E-PCAPS.md:358); co-update obligations documented if ever exercised |
| F-W86S-P10-007 (LOW) | FIXED v2.0 | STORY-182 | ci.yml grep denominator loosened to `[1-9][0-9]*/[0-9]+` at 3 loci; assertion message + ACR name co-update loci noted |
| F-W86S-P10-008 (LOW) | FIXED v2.0 | STORY-182 | Background "ALL committed fixtures" claim scoped to "committed BY THIS STORY" |
| F-W86S-P10-009 (LOW) | FIXED v2.0 | STORY-182 | "genuine per-test coupling" claim tempered; non-self-referential `fixture_present("` call-site count assertion added to document registry coverage boundary |
| F-W86S-P10-010 (LOW) [process-gap] | FIXED v2.0 per orch. ruling | BOTH | Explicit E-11 template note added to STORY-182 v2.0 and STORY-183 v2.0: manual RED demonstration is accepted substitute; no task reorder. PG-W86-013 added. |
| F-W86S-P10-011 (LOW) | FIXED v2.0 | STORY-183 | Explicit `mod._find_repo_root(Path(mod.__file__).resolve().parent)` derivation form added; structural assertion updated to `any(p.parent.name == "src" and p.suffix == ".rs" for p in files)` |

**5 NITs:** Deliberately unfixed per churn-avoidance ruling. Recorded above.

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed; no sibling stories share the affected loci.

**Orchestrator rulings recorded:**
- F-W86S-P10-004: ACR "No independent path construction" scoped to RESOLVING OR OPENING;
  display-only path for SKIP diagnostic explicitly permitted.
- F-W86S-P10-010: E-11 tdd_mode:strict manual-RED convention documented in both stories;
  no task reorder required.

**STORY-INDEX:** v4.05→v4.06 (wave-86 row v1.9→v2.0 both stories; no numeric totals changed).

**Streak:** 0/3. Pass 11 pending adversary dispatch. Trajectory-tail: →12→12→11.
