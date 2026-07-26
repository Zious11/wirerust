---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-26T22:00:00Z
cycle: "wave-086"
pass: 11
verdict: NOT_CONVERGED
novelty: "substantive"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 11

**Date:** 2026-07-26
**Pass:** 11 of N
**Verdict:** NOT CONVERGED
**Novelty:** substantive — 4th recurrence of inert/self-referential-predicate class (now both directions: vacuous-pass AND false-FAIL from prose needle); F-W86S-P11-001 HIGH is pass-10-induced
**Tally:** 14 findings — 0 CRIT / 1 HIGH / 6 MED / 7 LOW + 3 NITs (actioned) + 2 [process-gap] observations
**Status:** REMEDIATED — D-528 state burst; STORY-182 v2.0→v2.1 + STORY-183 v2.0→v2.1
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Positives from adversary:** All counts/hashes/line-refs/regex tracing verified correct; STORY-183 pattern core assessed as "unusually careful work and I could not break it"; deferred TIER-2 sites compatible with zero-FP; DF-INPUT-HASH-CANONICAL-001 compliance confirmed (9a0f34c / 9c9b12f unchanged); DF-CANONICAL-FRAME-HOLDOUT-001 n/a (E-11 template stories)

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P11-001 | HIGH | STORY-182 | FIXED v2.1 | Pass-10 `fixture_present("` call-site count assertion guaranteed false-FAIL — needle literal present in its own comments (:688/:692-693); count 6 ≠ len 4; justification comment had the hazard inverted (claimed vacuous-pass risk; real risk was false-FAIL from prose). 4th recurrence of inert/self-referential-predicate class. |
| F-W86S-P11-002 | MED | STORY-182 | FIXED v2.1 | `.factory/maintenance/fixture-count-gate-entry.md` assigned Branch=develop in Arch Mapping though .factory/ is gitignored on develop (CLAUDE.md branch topology); Task 11 vs Notes §Develop PR contradictory. |
| F-W86S-P11-003 | MED | STORY-182 | FIXED v2.1 | tdd_mode note cited :669-674 (now the forbidden-committed guard after pass-10 insertions) instead of the move-capture-aside procedure. |
| F-W86S-P11-004 | MED | STORY-182 | FIXED v2.1 | ACR §Constants placement omitted FIXTURE_GATED_TESTS (DF-SIBLING-SWEEP-001 violation — pass-9 F-008 fix not swept to ACR). |
| F-W86S-P11-005 | MED | STORY-183 | FIXED v2.1 | AC-183-001 placed src/*.rs in BOTH ls-files invocations (collector has no dedup → double-count of 10 top-level src files) contradicting Task 2. |
| F-W86S-P11-006 | MED | STORY-183 | FIXED v2.1 | AC-183-009 mis-attributed CI wiring to PG-W84-012's registration-only scope; bin-selftest job (ci.yml:479-486) lacks the test_lint_cycle_artifact.py step, so registration alone never runs it. |
| F-W86S-P11-007 | MED | STORY-183 | FIXED v2.1 | tdd_mode note prescribed inverted-pattern-order RED that ACR :1015-1016 (append-only/no-reorder) forbids; also contained false "no automated RED reachable" premise. |
| F-W86S-P11-008 | LOW | STORY-182 | FIXED v2.1 | Duplicate Arch Mapping rows for FIXTURE_GATED_TESTS. |
| F-W86S-P11-009 | LOW | STORY-182 | FIXED v2.1 | Harness self-read (CARGO_MANIFEST_DIR source-tree assumption) uncovered by ECs — EC-008 missing. |
| F-W86S-P11-010 | LOW | STORY-182 | FIXED v2.1 | dissect-download sha256 gate embedded in AC-182-003 (committed-fixtures AC); belongs in AC-182-002 (download-gate AC). |
| F-W86S-P11-011 | LOW | STORY-182 | FIXED v2.1 | CLAUDE.md Project-References row omitted the BLOCKS-gate-entry obligation for fixture-count-gate-entry.md. |
| F-W86S-P11-012 | LOW | STORY-183 | FIXED v2.1 | Zero-FP budget did not account for 10 newly-scanned src/*.rs files (adversary verified currently clean; need audit statement + re-run-at-delivery note). |
| F-W86S-P11-013 | LOW | STORY-183 | FIXED v2.1 | Task 9 hermetic `git add <tmp>/...` from repo-root cwd fails "outside repository" — should be `git -C <tmp> add ...`. |
| F-W86S-P11-014 | LOW | STORY-183 | FIXED v2.1 | "N > previous Rust-only count" not mechanically verifiable; needs baseline derivation command + class assertion. |

---

## Findings Detail

### F-W86S-P11-001 (HIGH) — Call-site count assertion guaranteed false-FAIL from prose needle
**Story:** STORY-182
**Status:** FIXED v2.1
**4th recurrence:** inert/self-referential-predicate class. Prior instances: P3-F005 (vacuous-pass), P5-F001 (vacuous-pass), P9-F003 (vacuous-pass). P11 is the INVERSE direction: false-FAIL from prose.
**Description:** The pass-10 remediation added a non-vacuous call-site count assertion at AC-182-005
that counted occurrences of the literal string `fixture_present("` in the harness source file and
asserted this count equals `FIXTURE_MANIFEST.len()` (4). However, the assertion's own prescribed
justification comments at lines :688, :692-693 contain the contiguous token `fixture_present("`
as part of their inline explanation. The harness self-read (via `include_str!`) therefore sees
count 6 (4 real call-sites + 2 comment occurrences), not 4. The assertion would fail immediately
on delivery with `assert_eq!(6, 4)`.

The pass-10 justification comment incorrectly claimed the risk was vacuous-pass (the prior class).
The actual risk was the opposite: guaranteed false-FAIL from the prose needle. The assertion
"verified" something it structurally could never pass.

**Fix (v2.1):** Needle built via `concat!("fixture_present", "(\"")` so the literal token does not
appear contiguous anywhere in the harness source; all justification comments rewritten to avoid
the contiguous token; justification states the real hazard (false-FAIL from prose occurrence);
concrete failure conditions enumerated.

---

### F-W86S-P11-002 (MED) — fixture-count-gate-entry.md assigned develop branch in Arch Mapping
**Story:** STORY-182
**Status:** FIXED v2.1
**Description:** The Arch Mapping table and Notes §Develop PR list both assigned
`.factory/maintenance/fixture-count-gate-entry.md` to Branch=develop. Per CLAUDE.md (Git
Workflow), `.factory/` is gitignored on develop — this file lives on factory-artifacts branch,
committed by state-manager. Task 11 listed the file in the develop PR file-changed list,
contradicting Notes §Develop PR which stated it as a state-manager commit.
**Fix (v2.1):** Branch cell updated to factory-artifacts; file removed from develop PR list;
"state-manager commits" note added; traces_to annotated.

---

### F-W86S-P11-003 (MED) — tdd_mode note line-citation stale after pass-10 insertions
**Story:** STORY-182
**Status:** FIXED v2.1
**Description:** The tdd_mode RED note cited lines :669-674 as the move-capture-aside procedure.
After pass-10 insertions added ~45 lines to the AC body, lines :669-674 now point to the
forbidden-committed guard, not the move-capture-aside procedure. The cited lines no longer
correspond to the claimed content.
**Fix (v2.1):** Re-anchored to :740-745 (verified post-edit). PG-W86-014 codified: this is the
intra-document line-citation drift class; a mandatory re-anchor sweep is now required after
every remediation burst.

---

### F-W86S-P11-004 (MED) — ACR §Constants omits FIXTURE_GATED_TESTS (DF-SIBLING-SWEEP-001 gap)
**Story:** STORY-182
**Status:** FIXED v2.1
**Description:** The ACR §Constants section named FIXTURE_MANIFEST and COMMITTED_FIXTURE_NAMES
but omitted FIXTURE_GATED_TESTS. The pass-9 F-008 fix added FIXTURE_GATED_TESTS to the story
body but the sweep did not propagate to the ACR. DF-SIBLING-SWEEP-001 violation.
**Fix (v2.1):** All three constants named in ACR §Constants.

---

### F-W86S-P11-005 (MED) — src/*.rs double-listed in both ls-files invocations (double-count)
**Story:** STORY-183
**Status:** FIXED v2.1
**Description:** AC-183-001 prescribed two separate `git ls-files` invocations — one for `tests/*.rs`
and `src/**/*.rs`, another for `src/*.rs` and `bin/*.py`. Because `src/*.rs` appears in both,
the collector processes top-level src files twice, and since there is no dedup step, the count
inflates by 10 (the 10 top-level src files). This contradicts Task 2 which prescribes a single
merged invocation.
**Fix (v2.1):** Single merged invocation `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py`
at all 3 loci; no-dedup rationale documented.

---

### F-W86S-P11-006 (MED) — AC-183-009 CI wiring mis-attributed to PG-W84-012 registration-only scope
**Story:** STORY-183
**Status:** FIXED v2.1 per orchestrator ruling
**Description:** AC-183-009 attributed the CI wiring to PG-W84-012's scope, but PG-W84-012 as
originally written covers only registration of bin-selftest as a required-status-check. The
bin-selftest CI job (ci.yml:479-486) does not include the test_lint_cycle_artifact.py step, so
registration alone (PG-W84-012 original scope) never causes the test to run in CI.
**Orchestrator ruling (D-528):** Attribution corrected to "PG-W84-012 AS EXTENDED at D-525 per
F-W86S-P9-012" which covers (a) registration AND (b) adding both missing steps. Local-only
until (b) is completed; no new CI-wiring task added (consistent with F-P9-012 ruling).
**Fix (v2.1):** Attribution corrected per orchestrator ruling.

---

### F-W86S-P11-007 (MED) — tdd_mode note prescribed inverted-pattern-order RED (ACR :1015-1016 violation) + false premise
**Story:** STORY-183
**Status:** FIXED v2.1 per orchestrator ruling
**Description:** The tdd_mode RED note described RED as adding test cases in inverted order (BAD
cases AFTER good cases). ACR :1015-1016 states append-only/no-reorder — inverted ordering
would violate this constraint. Additionally, the note contained the false premise "no automated
RED reachable" which was a generic boilerplate claim, not a demonstrated assertion for this
specific story.
**Orchestrator ruling (D-528):** Real automated RED prescribed: add BAD_CASES (Tasks 7/8) before
pattern tuples (Task 6) → selftest exits 1 → add patterns → GREEN. Explicit task ordering
prescribed. Inverted-order text removed. "No automated RED reachable" text replaced with
per-story demonstrated claim.
**Fix (v2.1):** Per orchestrator ruling applied.

---

### F-W86S-P11-008 (LOW) — Duplicate Arch Mapping rows for FIXTURE_GATED_TESTS
**Story:** STORY-182
**Status:** FIXED v2.1
**Description:** FIXTURE_GATED_TESTS appeared in two separate Arch Mapping rows.
**Fix (v2.1):** Merged into single row.

---

### F-W86S-P11-009 (LOW) — Harness self-read CARGO_MANIFEST_DIR assumption missing EC
**Story:** STORY-182
**Status:** FIXED v2.1
**Description:** The harness reads itself via a path derived from CARGO_MANIFEST_DIR, assuming it
is in the source tree. This assumption is not covered by an EC documenting what happens when
CARGO_MANIFEST_DIR points to a different location (e.g., cargo-vendor or out-of-source build).
**Fix (v2.1):** EC-008 added documenting the source-tree assumption.

---

### F-W86S-P11-010 (LOW) — sha256 gate embedded in wrong AC (download-gate vs committed-fixtures AC)
**Story:** STORY-182
**Status:** FIXED v2.1
**Description:** The dissect-download sha256 integrity gate was embedded inside AC-182-003
(committed-fixtures AC) but belongs in AC-182-002 (download-gate AC) since it gates the
download step, not the committed-fixture assertion.
**Fix (v2.1):** Relocated to AC-182-002; cross-ref left in AC-182-003.

---

### F-W86S-P11-011 (LOW) — CLAUDE.md Project-References row missing BLOCKS-gate-entry obligation
**Story:** STORY-182
**Status:** FIXED v2.1
**Description:** The CLAUDE.md Project-References row for fixture-count-gate-entry.md was present
but did not include the BLOCKS-gate-entry obligation note (i.e., this file must be consulted
before running the gate).
**Fix (v2.1):** Obligation added in enforcement-trigger style.

---

### F-W86S-P11-012 (LOW) — Zero-FP budget does not account for 10 newly-scanned src/*.rs files
**Story:** STORY-183
**Status:** FIXED v2.1
**Description:** The pass-9 fix added src/*.rs to the glob (F-W86S-P9-009), adding 10 previously-
unscanned top-level src files. The zero-FP budget did not update to acknowledge these 10 new
files. The adversary verified they are currently clean but the story provided no audit trail.
**Fix (v2.1):** Audit statement added documenting the grep + result; re-run-at-delivery note added.

---

### F-W86S-P11-013 (LOW) — Task 9 hermetic git-add from repo-root cwd fails "outside repository"
**Story:** STORY-183
**Status:** FIXED v2.1
**Description:** Task 9 prescribed `git add <tmp>/violating.py` run from the repo root cwd. Git
treats `<tmp>/...` as outside the repository, producing "fatal: outside repository" error.
**Fix (v2.1):** Both occurrences corrected to `git -C <tmp> add violating.py`.

---

### F-W86S-P11-014 (LOW) — "N > previous Rust-only count" not mechanically verifiable
**Story:** STORY-183
**Status:** FIXED v2.1
**Description:** The assertion "N > previous Rust-only count" was not mechanically verifiable
because the baseline count was not specified and no derivation command was provided.
**Fix (v2.1):** Baseline derivation command added; class assertion `any(p.suffix == ".py" ...)`
added to confirm the delta is attributable to Python files.

---

## NIT-Observations (3 items — actioned; pass-10 NITs untouched)

1. **NIT-01 (ACTIONED):** ACR "comment-only/step-name" wording ambiguous between no-behavior-change
   and structural constraints. Fixed: reworded to "non-functional edits only" in both stories.

2. **NIT-02 (ACTIONED):** STORY-183 pass-message "reflect both file types" was imprecise — the
   pass is a variable rename only (not "both file types" semantics). Fixed: reworded to
   "variable rename only".

3. **NIT-03 (ACTIONED):** traces_to created-artifact gap (folded into F-002 fix — resolved
   when factory-artifacts branch assignment was corrected in Arch Mapping).

**Pass-10 NITs (NIT-01..NIT-05):** Deliberately untouched per D-527 churn-avoidance ruling.

---

## Process-Gap Observations

### [PG-W86-013 EXTENSION] — E-11 tdd_mode boilerplate itself defective in v2.0

The v2.0 E-11 tdd_mode note added per F-W86S-P10-010 contained generic boilerplate asserting
"no automated RED reachable" at the epic level. This was defective:

- **STORY-183 (F-P11-007):** The "no automated RED reachable" claim was false — a real automated
  RED exists (add BAD_CASES before pattern tuples; selftest exits 1).
- **STORY-182 (F-P11-003):** The claim was chosen-ordering-dependent, not epic-level invariant.

v2.1 notes are now per-story demonstrated claims replacing the generic assertion. Codification
requirement sharpened: the E-11 template must require stories to **DEMONSTRATE** RED
unreachability (or specify a concrete RED path), not assert it as a template-level boilerplate.

This extends PG-W86-013 evidence: the boilerplate fix itself introduced a defective template.

**Codification flag:** PG-W86-013 extended. Codification at S-7.02 must require demonstrated
per-story RED claim, not generic template text.

---

### [PG-W86-014 NEW] — Intra-story `:NNN` self-citation drift after mid-burst insertions

Pass-10 remediations inserted ~45 lines into STORY-182's AC bodies without re-anchoring
intra-document line citations. F-W86S-P11-003 (tdd_mode note cited wrong lines after insertion)
is a direct instance.

STORY-182 carries 17+ intra-document `:NNN` self-citations. No mechanical guard exists:
`bin/validate-citations` is a docs-writer preflight for external doc citations, not intra-story
line references.

**Mitigation imposed D-528:** Story-writer dispatches now include a mandatory post-edit
intra-document line-citation re-anchor sweep. This burst executed the sweep — table clean.

**Codification candidate at S-7.02:** Extend DF-SIBLING-SWEEP-001 or story-writer skill with
this sweep. PG-W86-014 added to process-gap-ledger.md.

---

## Adversary Positive Observations

- All counts/hashes/line-refs/regex tracing verified correct across both stories.
- STORY-183 pattern core assessed as "unusually careful work and I could not break it"
  (adversary verbatim observation).
- Deferred TIER-2 sites compatible with zero-FP requirement.
- No collateral test breakage from v2.0 edits.
- DF-GREEN-DOC-TENSE-SWEEP v6 compliance confirmed (no regressions from v2.0 edits).
- DF-TEST-NAMESPACE-001 compliance confirmed.
- DF-AC-TEST-NAME-SYNC-001 compliance confirmed.
- DF-INPUT-HASH-CANONICAL-001 compliance confirmed (9a0f34c / 9c9b12f canonical values).
- DF-CANONICAL-FRAME-HOLDOUT-001: n/a (E-11 template stories).

---

## Pass-11 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: substantive — 4th recurrence of inert/self-referential-predicate class (now both
directions); HIGH = pass-10-induced (false-FAIL from prose needle).
Pass-12 next after D-528 remediation burst.

Pass tallies (P1–P11): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14.
Total across all passes: 203 findings. Canonical hashes: 9a0f34c / 9c9b12f.

---

## Remediation (D-528)

**Date:** 2026-07-26
**Burst:** D-528 STATE BURST — WAVE-86 ADVERSARIAL PASS 11 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 14 findings FIXED at STORY-182 v2.1 / STORY-183 v2.1 with per-fix grep evidence
per PG-W86-010 mandate. Full DF-SIBLING-SWEEP-001 sweep performed. NEW: intra-document
line-citation re-anchor sweep executed (table clean — PG-W86-014).

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P11-001 (HIGH) | FIXED v2.1 | STORY-182 | concat!-needle fix: needle built via `concat!("fixture_present", "(\"")` so literal not contiguous in harness source; comments rewritten; justification states real hazard (false-FAIL) + concrete failure conditions |
| F-W86S-P11-002 (MED) | FIXED v2.1 | STORY-182 | Branch cell→factory-artifacts; removed from develop PR list; state-manager-commits note; traces_to annotated |
| F-W86S-P11-003 (MED) | FIXED v2.1 | STORY-182 | tdd_mode note re-anchored to :740-745 (verified post-edit) |
| F-W86S-P11-004 (MED) | FIXED v2.1 | STORY-182 | All three constants (FIXTURE_MANIFEST, COMMITTED_FIXTURE_NAMES, FIXTURE_GATED_TESTS) named in ACR §Constants |
| F-W86S-P11-005 (MED) | FIXED v2.1 | STORY-183 | Single merged invocation at all 3 loci; no-dedup rationale documented |
| F-W86S-P11-006 (MED) | FIXED v2.1 per orch. ruling | STORY-183 | Attribution corrected to "PG-W84-012 AS EXTENDED at D-525 per F-W86S-P9-012"; local-only-until-(b); no new CI-wiring task |
| F-W86S-P11-007 (MED) | FIXED v2.1 per orch. ruling | STORY-183 | Real automated RED prescribed (BAD_CASES Tasks 7/8 before pattern tuples Task 6); inverted-order text removed; false "no automated RED" text replaced with per-story demonstrated claim |
| F-W86S-P11-008 (LOW) | FIXED v2.1 | STORY-182 | Duplicate FIXTURE_GATED_TESTS rows merged |
| F-W86S-P11-009 (LOW) | FIXED v2.1 | STORY-182 | EC-008 added (CARGO_MANIFEST_DIR source-tree assumption) |
| F-W86S-P11-010 (LOW) | FIXED v2.1 | STORY-182 | sha256 gate relocated to AC-182-002; cross-ref left in AC-182-003 |
| F-W86S-P11-011 (LOW) | FIXED v2.1 | STORY-182 | CLAUDE.md row updated with BLOCKS-gate-entry obligation in enforcement-trigger style |
| F-W86S-P11-012 (LOW) | FIXED v2.1 | STORY-183 | Audit statement + grep added; re-run-at-delivery note added |
| F-W86S-P11-013 (LOW) | FIXED v2.1 | STORY-183 | Both `git add <tmp>/...` occurrences corrected to `git -C <tmp> add ...` |
| F-W86S-P11-014 (LOW) | FIXED v2.1 | STORY-183 | Baseline derivation command added; class assertion `any(p.suffix == ".py" ...)` added |

**3 NITs actioned:** ACR wording → "non-functional edits only"; pass-message → "variable rename only";
traces_to gap folded into F-002 fix. Pass-10 NITs deliberately untouched.

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**NEW — Intra-document line-citation re-anchor sweep (PG-W86-014 mitigation D-528):**
Post-edit sweep of all :NNN citations in STORY-182 body executed. 17+ self-citations verified
against post-edit line numbers. Table clean. This sweep is now a mandatory dispatch step.

**Orchestrator rulings recorded:**
- F-W86S-P11-002: fixture-count-gate-entry.md lives on factory-artifacts branch (state-manager
  commits), NOT develop.
- F-W86S-P11-005: Single merged glob `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py`
  is the authoritative form at all 3 loci.
- F-W86S-P11-006: PG-W84-012-as-extended attribution covering (a) registration + (b) missing
  steps; consistent with F-P9-012 ruling at D-525.
- F-W86S-P11-007: Real automated RED for STORY-183 prescribed; BAD_CASES before pattern tuples.

**STORY-INDEX:** v4.06→v4.07 (wave-86 row v2.0→v2.1 both stories; no numeric totals changed).

**PG-W86-013 EXTENDED:** Evidence from P11 added — v2.0 boilerplate was itself defective; v2.1
notes are per-story demonstrated claims. Codification requirement sharpened.

**PG-W86-014 NEW:** Intra-story :NNN self-citation drift after mid-burst insertions. Added to
process-gap-ledger.md.

**Streak:** 0/3. Pass 12 pending adversary dispatch. Trajectory-tail: →12→11→14.
