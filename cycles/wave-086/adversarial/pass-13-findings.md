---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-26T23:45:00Z
cycle: "wave-086"
pass: 13
verdict: NOT_CONVERGED
novelty: "substantive"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 13

**Date:** 2026-07-26
**Pass:** 13 of N
**Verdict:** NOT CONVERGED
**Novelty:** substantive — both HIGHs are remediation-introduced regressions: F-P13-001 is a truth-inversion introduced by the pass-12 pathspec "correction"; F-P13-002 is 3 stale intra-document self-anchors that survived the D-528 re-anchor sweep because the sweep only covered changed citations, not the full citation corpus
**Tally:** 15 findings — 0 CRIT / 2 HIGH / 4 MED / 9 LOW + 5 NITs (all fixed)
**Status:** REMEDIATED — D-530 state burst; STORY-182 v2.2→v2.3 + STORY-183 v2.2→v2.3
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Positives from adversary:** AC-183-009 grep-count==0 predicates non-vacuous; hermetic Task-9 harness (git -C <tmp>) sound; no-literal-phrase sweep obligation (D-529 standing discipline) applied correctly to pass-12 annotations; DF-GREEN-DOC-TENSE-SWEEP v6 DIRECT-SCRUB OBLIGATION verified; canonical hashes 9a0f34c/9c9b12f unchanged; DF-SIBLING-SWEEP-001 compliance confirmed; all 5 NITs were minor wording improvements.

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P13-001 | HIGH | STORY-183 | FIXED v2.3 | Pathspec truth inversion: pass-12 F-P12-006 "correction" changed :228-230 to "both globs cover the same files" — this inverts the load-bearing v1.9 claim that `src/*.rs` STRICTLY SUBSUMES `src/**/*.rs` in git wildmatch. The v2.2 prose was technically true (dedup makes file lists equal) but false w.r.t. the intended semantics claim (src/*.rs is the broader pattern). All 7 loci now agree: "src/*.rs strictly subsumes src/**/*.rs; both explicit for clarity". |
| F-W86S-P13-002 | HIGH | STORY-182 | FIXED v2.3 | 3 stale intra-document self-anchors survived the D-528 re-anchor sweep: (a) tdd_mode RED-procedure :740-745 stale (v2.2 edits shifted lines); (b) move-aside cross-reference :698 stale; (c) AC-182-003 intra-AC cite :612 stale. D-528 sweep only checked citations in changed sections, not unchanged sections. Structural fix D-530: all intra-document :NNN self-citations eliminated in favor of content-based locators (phrase anchors, task names, AC identifiers). Zero :NNN self-citations remain in either story. |
| F-W86S-P13-003 | MED | STORY-183 | FIXED v2.3 | Pattern 33 discriminator wrong: story stated "is" as the discriminator but the actual TIER-1 pattern is "falls through to" — the word "through" is the discriminator that makes the phrase TIER-1 (versus "falls to" which is TIER-2 or benign). Fix: discriminator corrected to "through"; Pattern 33 description updated at 2 loci. |
| F-W86S-P13-004 | MED | STORY-183 | FIXED v2.3 | AC-183-009 grep-count predicates used inverted gating form (`grep -c ... \| grep -v "^0$"` — fails when count IS zero) rather than the canonical `test "$(grep -c ...)" -eq 0` form (fails when count is NOT zero). Sibling propagation from STORY-182 F-P10-003 applied; all 3 predicates corrected. |
| F-W86S-P13-005 | MED | BOTH | FIXED v2.3 | ci.yml merge-order/line-anchor order-dependence unacknowledged: both stories modify ci.yml and cite baseline e8841d76 line numbers, but both stories' ci.yml tasks can shift those line numbers on delivery order. Labels added: "baseline-e8841d76; order-dependent — rebase if sibling story's CI changes landed first"; 3 loci in STORY-182, 2 loci in STORY-183. |
| F-W86S-P13-006 | MED | STORY-183 | FIXED v2.3 | Primary citation path-elided: story cited "convergence-report.md" without path qualification; 6 files named convergence-report.md exist in the repo across different cycles. Path-qualified at 7 loci to `.factory/cycles/wave-085/STORY-180/convergence-report.md`; added to traces_to frontmatter. |
| F-W86S-P13-007 | LOW | STORY-183 | FIXED v2.3 | TypeID overclaim: story stated TypeIDs 58-64 (7 TypeIDs) but STORY-180/BC-2.19.029 establishes only 58/59/61/63 (4 TypeIDs: S_SP_NA_1, S_DP_NA_1, S_ME_NB_1, S_IT_NA_1). Corrected at 3 loci including README row reference. |
| F-W86S-P13-008 | LOW | STORY-182 | FIXED v2.3 | Non-restoring `mv` in move-aside procedures: all 3 templates used bare `mv` without `trap EXIT` restore or `\|\|true` guard. Fix: `trap 'mv -f "$ASIDE" "$ORIG" 2>/dev/null \|\|true' EXIT` added to all 3 move-aside procedures. |
| F-W86S-P13-009 | LOW | STORY-183 | FIXED v2.3 | Task 9 hermetic harness: `import` statements and function placement described without specifying that the import block must precede the function block (Python ordering rule). Placement order made explicit in Task 9 step description. |
| F-W86S-P13-010 | LOW | STORY-183 | FIXED v2.3 | Pattern 34 apostrophe: pattern written as `doesn't` but the regex target must use `doesn['"]?t` to match both the straight apostrophe and curly apostrophe variants that can appear in comment text. Regex form added at 2 loci. |
| F-W86S-P13-011 | LOW | STORY-183 | FIXED v2.3 | Sweep locus :97 in bin/check-green-doc-tense.py omitted from Task 13 scrub list; :97 carries a stale tense reference. Added to Task 13 target list. |
| F-W86S-P13-012 | LOW | BOTH | FIXED v2.3 | Baseline anchor labels absent from ci.yml task descriptions in both stories: implementers need to know which baseline HEAD these line numbers were counted against to validate them post-rebase. Labels "baseline-e8841d76" added adjacent to all absolute ci.yml line citations. (Distinct from F-P13-005 which covers the order-dependence acknowledgement note; this finding covers the baseline label addition only.) |
| F-W86S-P13-013 | LOW | STORY-183 | FIXED v2.3 | Pattern 34 GOOD annotation used describe-not-quote inconsistently: some GOOD_CASE annotations for Pattern 34 quoted the literal phrase inline (violating no-literal-phrase sweep obligation per D-529); reworded to describe-without-quoting at 2 annotation loci per standing D-529 discipline. |
| F-W86S-P13-014 | LOW | STORY-183 | FIXED v2.3 | "falls through to" site enumeration incomplete: story claimed "all 10 sites" but only 9 were enumerated in the task sweep list. Tenth site (tests/iec104_analyzer_tests.rs line range) added to complete the enumeration; count reconciled at 2 loci. |
| F-W86S-P13-015 | LOW | STORY-182 | FIXED v2.3 | `if: !cancelled()` + compile-failure visibility: AC-182-006 ci.yml step used `if: always()` without noting that `if: !cancelled()` is the preferred form (runs after failures but not after cancellation); additionally, the step placement after the main test step meant compile failures before the step would suppress the fixture coverage report entirely. Corrected: `if: !cancelled()` + placement note added. |

---

## Findings Detail

### F-W86S-P13-001 (HIGH) — Pathspec truth inversion by pass-12 correction

**Story:** STORY-183
**Status:** FIXED v2.3
**Root cause class:** Truth-inversion-during-reword — a factually accurate wording of the
deduplication behavior ("both globs cover the same files after dedup") conflated with the
semantics claim, overwriting the true v1.9 assertion about which glob is the broader pattern.

**Description:** STORY-183 v1.9 (F-W86S-P9-009 remediation) established the pathspec truth:
`src/*.rs` in git wildmatch **strictly subsumes** `src/**/*.rs` — the former pattern matches
everything the latter does plus top-level `src/*.rs` files. The v1.9 claim was load-bearing:
it justified including both globs in the 4-glob `git ls-files` invocation (for explicitness)
while noting that `src/*.rs` is the operative wider-coverage pattern.

Pass-12 F-P12-006 corrected 5 documentation loci that described "src/*.rs covers top-level
only" — the correction stated "both globs cover the same files" (after git deduplication).
This is technically true in terms of file-set result (dedup produces equal sets) but it
INVERTS the semantic claim: the reason both globs are present is precisely because src/*.rs
is BROADER, not equal. A reader of v2.2 could conclude that src/**/*.rs provides equivalent
coverage and drop src/*.rs from the 4-glob invocation — which would leave top-level `src/*.rs`
files unscanned. The corrected semantics in v2.3 are: "src/*.rs strictly subsumes
src/**/*.rs; git deduplicates so no file is double-counted; both globs kept for explicitness
and to make the src/*.rs breadth visible to implementers."

**7-loci agreement check (D-530 verification):** Pass-13 adversary verified that all 7
loci in v2.3 STORY-183 that describe the glob relationship state the subsumption direction
correctly. Zero loci remain with the inverted "both cover the same files" framing.

**Fix (v2.3):** :228-230 and 6 additional loci updated to "src/*.rs strictly subsumes
src/**/*.rs; both explicit for clarity; dedup ensures no double-count". All-loci agreement
confirmed before commit.

---

### F-W86S-P13-002 (HIGH) — 3 stale intra-document self-anchors survived D-528 sweep

**Story:** STORY-182
**Status:** FIXED v2.3 — structural fix: all :NNN self-citations eliminated
**Root cause class:** Self-anchor drift class (extends PG-W86-014) — D-528 re-anchor sweep
only checked citations in sections that were CHANGED in the D-528 burst; it did not check
sections that were unchanged but whose line numbers were shifted by prior-section insertions.

**Description:** Three intra-document :NNN self-citations in STORY-182 were stale after the
v2.2 burst:

1. **:740-745 (tdd_mode RED-procedure):** The D-527 pass-10 burst added ~45 lines; D-528
   re-anchored this to :740-745. The D-529 pass-12 burst added additional lines in earlier
   sections, shifting this block to a different range. The D-528 sweep did not re-check
   previously-swept citations for secondary drift from the D-529 edits.

2. **:698 (move-aside cross-reference):** Added in v2.1 D-528 burst; not in a changed section
   in D-529, so the D-529 sweep missed it. Line shifted by ~15 lines from D-529 insertions.

3. **:612 (AC-182-003 intra-AC cite):** Original citation from v2.0 that had survived
   multiple sweeps; shifted by cumulative insertions across D-527/D-528/D-529 bursts.

**Structural fix (D-530):** Rather than applying a third re-anchor sweep (which would
generate further drift on every subsequent burst), all intra-document :NNN self-citations
in both STORY-182 and STORY-183 were ELIMINATED and replaced with content-based locators:
- Task citations: "the move-aside procedure in Task 8" (not ":698")
- Section citations: "see AC-182-003 Resolver Coupling clause" (not ":612")
- Procedure citations: "the tdd_mode RED-procedure section" (not ":740-745")

Post-fix sweep: zero :NNN self-citations remain in STORY-182 or STORY-183. This is the
structural fix recommended by PG-W86-014 — content-based locators are immune to line-shift
drift. Recommend as E-11 template convention at S-7.02.

---

### F-W86S-P13-003 (MED) — Pattern 33 wrong discriminator

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** Pattern 33 ("falls through to") was described with "is" as the discriminator
that distinguishes it from TIER-2 patterns. The actual discriminator is "through" — it is the
word "through" in "falls through to" that is the operative TIER-1 token. "Falls to" (without
"through") is benign. Story stated: "discriminator: 'is'" at 2 loci. Fix: corrected to
"discriminator: 'through'" with the rational "falls-through-to vs. falls-to distinction".

---

### F-W86S-P13-004 (MED) — Inverted grep -c gates in AC-183-009

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** AC-183-009's three grep-count==0 predicates were expressed as:
```
grep -c "phrase" file | grep -v "^0$"   # WRONG: exits 0 when count != 0
```
The canonical gating form (per STORY-182 F-P10-003 discipline) is:
```
test "$(grep -c "phrase" file)" -eq 0   # CORRECT: exits 1 when count != 0
```
The inverted form exits 0 (success) precisely when the phrase IS present — the opposite of
the intended gate behavior. All 3 predicates corrected to `test "$(grep -c ...)" -eq 0`.
Cross-story sibling propagation discipline (DF-SIBLING-SWEEP-001) applied.

---

### F-W86S-P13-005 (MED) — ci.yml merge-order/line-anchor order-dependence unacknowledged

**Story:** BOTH
**Status:** FIXED v2.3
**Description:** Both stories modify ci.yml and cite absolute line numbers computed against
baseline develop=e8841d76. If either story is delivered first and the other story's
implementer does not rebase, the ci.yml line numbers in the second story's tasks will be
wrong. Neither story acknowledged this order-dependence. Fix: both stories' ci.yml task
sections annotated with "baseline-e8841d76; order-dependent — rebase against develop before
verifying line numbers"; 3 loci in STORY-182, 2 loci in STORY-183.

---

### F-W86S-P13-006 (MED) — Path-elided primary citation; 6 files share the name

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** STORY-183 cited "convergence-report.md" as the primary source for the
efficacy AC grounding. The `.factory/` tree contains 6 files named `convergence-report.md`
across different cycle directories:
- cycles/wave-084/STORY-147/convergence-report.md
- cycles/wave-084/STORY-166/convergence-report.md
- cycles/wave-084/STORY-176/convergence-report.md
- cycles/wave-085/STORY-180/convergence-report.md (the intended file)
- cycles/wave-085/STORY-181/convergence-report.md
- cycles/feature-iec104/convergence-trajectory.md (different name but similar)

Path-elided citation leaves the intended file ambiguous. Fix: all 7 citation loci updated to
the full path `.factory/cycles/wave-085/STORY-180/convergence-report.md`; path added to
traces_to frontmatter.

---

### F-W86S-P13-007 (LOW) — TypeID overclaim: should be 58/59/61/63, not 58-64

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** STORY-183 referenced "TypeIDs 58-64" as the timed control-command set.
STORY-180 and BC-2.19.029 establish the correct set: {58 S_SP_NA_1, 59 S_DP_NA_1,
61 S_ME_NB_1, 63 S_IT_NA_1} — 4 TypeIDs, not 7. TypeIDs 60, 62, 64 are NOT in the
timed-command subset. Corrected at 3 loci including the README cross-reference row.

---

### F-W86S-P13-008 (LOW) — Non-restoring `mv` in move-aside procedures

**Story:** STORY-182
**Status:** FIXED v2.3
**Description:** All 3 move-aside procedure templates in STORY-182 used bare `mv "$ORIG" "$ASIDE"`.
If any subsequent command in the procedure fails (e.g., the test being run exits non-zero
unexpectedly), the moved file is left in the aside location with no restore path. This is
a test-harness resource leak.
Fix: `trap 'mv -f "$ASIDE" "$ORIG" 2>/dev/null ||true' EXIT` added as the first line of each
move-aside procedure, before the mv command. The `||true` guard ensures the restore itself
cannot cause a double-failure.

---

### F-W86S-P13-009 (LOW) — Task 9 imports + placement ordering underspecified

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** Task 9's hermetic harness setup described the import statements and helper
function to add to the test file without specifying that Python requires the import block to
precede the function definition block. Without the placement note, an implementer unfamiliar
with Python module structure might place the function before the imports. Task 9 step
description updated to specify: "add imports first (top of file, after existing imports),
then function definition."

---

### F-W86S-P13-010 (LOW) — Pattern 34 apostrophe regex missing curly-quote variant

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** Pattern 34 was written as matching the literal `doesn't`. Source code
comments can contain curly apostrophes (U+2019) as well as straight apostrophes (U+0027)
depending on editor/paste behavior. The pattern should match `doesn['"]?t` (or equivalently
a character class including the two common apostrophe variants). Fix: Pattern 34 regex
updated to `doesn['"]?t` at 2 loci; rationale note added.

---

### F-W86S-P13-011 (LOW) — :97 sweep locus absent from Task 13 scrub list

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** bin/check-green-doc-tense.py:97 contains a stale tense reference that falls
within the DIRECT-SCRUB OBLIGATION scope per DF-GREEN-DOC-TENSE-SWEEP v6. This locus was
not included in the Task 13 scrub target list. Added.

---

### F-W86S-P13-012 (LOW) — Baseline anchor labels absent from ci.yml citations

**Story:** BOTH
**Status:** FIXED v2.3
**Description:** ci.yml absolute line number citations throughout both stories (Task descriptions,
AC notes) lacked "baseline-e8841d76" labels. Without the baseline label, an implementer who
has rebased cannot validate whether the cited line numbers still correspond to the intended
code blocks. Labels added at all absolute ci.yml citation sites. (Distinct from F-P13-005
which covers the order-dependence warning note; this finding covers the baseline label for
line-number validation.)

---

### F-W86S-P13-013 (LOW) — Pattern 34 GOOD annotation quoted literal phrase

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** Two GOOD_CASE annotations for Pattern 34 included the literal flagged phrase
inline in the annotation text (e.g., `# GOOD: "doesn't currently assert" is allowed because...`).
This violates the no-literal-phrase sweep obligation imposed at D-529: story text must
describe patterns without quoting their literal forms. Fixed per D-529 standing discipline:
both annotations reworded to describe-without-quoting form.

---

### F-W86S-P13-014 (LOW) — "falls through to" enumeration: 9 sites listed, 10 claimed

**Story:** STORY-183
**Status:** FIXED v2.3
**Description:** The Task 13 sweep enumeration claimed "all 10 falls-through-to sites" but
the explicit site list contained only 9 entries. The tenth site
(tests/iec104_analyzer_tests.rs at the TypeID dispatch comment block) was missing from the
enumeration. Added; count reconciled at 2 loci.

---

### F-W86S-P13-015 (LOW) — `if: always()` + compile-failure visibility scope

**Story:** STORY-182
**Status:** FIXED v2.3
**Description:** AC-182-006 prescribed `if: always()` for the ci.yml fixture coverage
reporting step. Two issues: (a) `if: !cancelled()` is the preferred form — it runs after
job failures but skips on user cancellation, preventing spurious uploads on cancelled runs;
(b) the step was placed after the main test step without noting that if compilation fails
(before the test step runs), the coverage report step is still executed by `always()` but
produces an empty/meaningless report. Fix: changed to `if: !cancelled()`; placement note
added: "placed after cargo test; if compilation fails, step runs but reports 0/0 fixtures
scanned — expected behavior for compile-failure CI runs."

---

## NIT-Observations (5 items — all actioned)

1. **NIT-01 (ACTIONED):** `///` doc-comment vs `//!` inner-doc-comment distinction in
   STORY-182 Task 6 description — "module-level doc comment" is `//!` not `///`; corrected.

2. **NIT-02 (ACTIONED):** Token budget estimate revised from ~680 to ~690-700 in STORY-183
   Task 1 notes; the v2.2 additions added ~10-20 tokens beyond the v2.1 estimate.

3. **NIT-03 (ACTIONED):** "fall" vs "falls" inflection inconsistency: AC-183-007 body used
   "fall through" (bare infinitive) in 2 locations where "falls through" (third-person singular)
   was intended; corrected.

4. **NIT-04 (ACTIONED):** `stat` vs `shasum` gating forms mixed within same task in
   STORY-182 Task 2; standardized to `shasum -c` form per prior PG-W86-010 mandate precedent.

5. **NIT-05 (ACTIONED):** EC-003 stderr note missing from STORY-182 Env B discriminator
   block — the note "SKIP messages land on stderr; this grep targets stdout" should appear
   adjacent to all stdout-only grep predicates; added.

---

## Process-Gap and Knowingly-Inert Observations

### [process-gap] — .py surface outside bin/ out of scope for STORY-183

Files `tests/fixtures/mk_modbus_large_pcap.py`, `tests/fixtures/mk_modbus_pcap.py`, and
`fuzz/seed_corpus.py` are tracked Python files in the repo outside the `bin/*.py` glob
covered by STORY-183. These files are candidates for the green-doc-tense scan surface but
are currently excluded by the STORY-183 scope definition (bin/*.py per PG-W84-010).

**Ruling (D-530):** These files are out of scope for STORY-183. EC-010 extended with explicit
out-of-scope statement naming these 3 paths. New drift row DRIFT-py-surface-outside-bin added
to STATE.md for follow-up at the next planning cycle.

### [knowingly-inert] — Pattern 37 PO-ruling transparency

Pattern 37 in DF-GREEN-DOC-TENSE-SWEEP v6 has a PO ruling that its TIER-2 treatment was
granted without full grep-evidence recording. This is knowingly-inert relative to pass-13:
the adversary noted the transparency gap but the PG-W86-004 standing rule (un-grepped tier
= policy violation) already covers this class. No new finding filed; carried as context for
S-7.02 codification review.

---

## EXECUTION-REQUIRED Flags

The following items require execution evidence at delivery or gate time:

**(i) Python selftest/gate exit codes:** Confirm `bin/test_compute_input_hash.py` exits 0
on develop=e8841d76. This is the baseline for AC-183-009 item (c); must be documented at
delivery.

**(ii) Cargo e2e 66-finding expectation:** Verify the diverse ITI e2e test still holds the
66/20/46 expectation on develop=e8841d76 (PR #439 gate-fix 0ab6f52e set this expectation;
back-merge e8841d76 preserved it — verify still current before delivery of STORY-182).

**(iii) git ls-files result sets:** Document the exact file count from
`git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py` on develop=e8841d76 to anchor
STORY-183's "N files scanned" claim at delivery. Required for AC-183-008 verification.

---

## Verified-Clean Table (pass-13 adversary confirmation)

| Verification Item | Result |
|-------------------|--------|
| All finding counts / pass tallies | EXACT |
| Input-hash values 9a0f34c / 9c9b12f | EXACT — canonical Python tool; unchanged |
| No-literal-phrase sweep (D-529 standing discipline) | CONFIRMED — no TIER-1 literals in v2.2 annotation text |
| AC-183-009 grep-count==0 predicates non-vacuous | CONFIRMED — predicates fail when scrub is skipped |
| Hermetic Task-9 harness (git -C <tmp>) | CONFIRMED — outside-repository error eliminated |
| DF-GREEN-DOC-TENSE-SWEEP v6 compliance | CONFIRMED — no TIER-1 regressions from v2.2 edits |
| DF-INPUT-HASH-CANONICAL-001 | CONFIRMED — hashes unchanged, canonical Python tool |
| DF-SIBLING-SWEEP-001 | CONFIRMED — sweep performed; no sibling regressions |
| Intra-document line-citation sweep (PG-W86-014) | NOT REQUIRED — self-anchors structurally eliminated |
| Path-qualified citations (F-P13-006) | CONFIRMED — 7 loci updated in STORY-183 |

---

## Pass-13 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: substantive — both HIGHs are remediation-induced regressions: F-P13-001 is a new
class (truth-inversion-during-reword); F-P13-002 is the self-anchor drift class (PG-W86-014)
recurring because the D-528 sweep had incomplete coverage.

Severity decay continues: P11 0C/1H/6M/7L → P12 0C/1H/4M/5L → P13 0C/2H/4M/9L.
(The HIGH count increased from 1 to 2 due to two remediation-introduced regressions.)

Pass-14 next after D-530 remediation burst.

Pass tallies (P1–P13): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14 / 10 / 15.
Total across all passes: 228 findings. Canonical hashes: 9a0f34c / 9c9b12f.

---

## Remediation (D-530)

**Date:** 2026-07-26
**Burst:** D-530 STATE BURST — WAVE-86 ADVERSARIAL PASS 13 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 15 findings FIXED at STORY-182 v2.3 / STORY-183 v2.3. Orchestrator verified all fixes
directly by grep before this burst was committed (direct grep-verification of fixes pre-commit,
distinguishing D-530 from prior bursts where verification was delegated to story-writer return).

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P13-001 (HIGH) | FIXED v2.3 | STORY-183 | Pathspec truth restored: src/*.rs strictly subsumes src/**/*.rs; 7-loci agreement confirmed; "both cover same files" framing eliminated |
| F-W86S-P13-002 (HIGH) | FIXED v2.3 | STORY-182 | Structural fix: all intra-document :NNN self-citations eliminated; content-based locators (task names, AC IDs, section headings) substituted throughout; zero :NNN self-citations remain |
| F-W86S-P13-003 (MED) | FIXED v2.3 | STORY-183 | Pattern 33 discriminator corrected to "through"; 2 loci |
| F-W86S-P13-004 (MED) | FIXED v2.3 | STORY-183 | AC-183-009 gating form: test "$(grep -c ...)" -eq 0; all 3 predicates corrected |
| F-W86S-P13-005 (MED) | FIXED v2.3 | BOTH | ci.yml order-dependence acknowledged: "baseline-e8841d76; order-dependent"; 3+2 loci |
| F-W86S-P13-006 (MED) | FIXED v2.3 | STORY-183 | convergence-report.md path-qualified to .factory/cycles/wave-085/STORY-180/convergence-report.md at 7 loci; added to traces_to |
| F-W86S-P13-007 (LOW) | FIXED v2.3 | STORY-183 | TypeIDs 58-64 corrected to 58/59/61/63; 3 loci incl. README row |
| F-W86S-P13-008 (LOW) | FIXED v2.3 | STORY-182 | trap-EXIT/\|\|true restore added to all 3 move-aside procedures |
| F-W86S-P13-009 (LOW) | FIXED v2.3 | STORY-183 | Task 9 import + placement ordering made explicit |
| F-W86S-P13-010 (LOW) | FIXED v2.3 | STORY-183 | Pattern 34 regex: doesn['"]?t at 2 loci |
| F-W86S-P13-011 (LOW) | FIXED v2.3 | STORY-183 | :97 sweep locus added to Task 13 target list |
| F-W86S-P13-012 (LOW) | FIXED v2.3 | BOTH | baseline-e8841d76 label added to all absolute ci.yml citations |
| F-W86S-P13-013 (LOW) | FIXED v2.3 | STORY-183 | Pattern 34 GOOD annotations reworded per D-529 no-literal-phrase standing discipline |
| F-W86S-P13-014 (LOW) | FIXED v2.3 | STORY-183 | Tenth falls-through-to site added; count reconciled at 2 loci |
| F-W86S-P13-015 (LOW) | FIXED v2.3 | STORY-182 | if: !cancelled() + compile-failure scope note |

**5 NITs actioned:** ///→//! doc-comment; token budget ~690-700; fall→falls inflection;
stat→shasum standardization; EC-003 stderr note added to Env B discriminator.

**EC-010 out-of-scope extension:** tests/fixtures/mk_modbus_large_pcap.py,
tests/fixtures/mk_modbus_pcap.py, fuzz/seed_corpus.py explicitly listed as out-of-scope
in STORY-183 EC-010. DRIFT-py-surface-outside-bin added to STATE.md Drift Items.

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**Structural fix discipline imposed D-530:** Truth-inversion-during-reword class (F-P13-001):
when rewording any technical semantics claim, re-derive the claim from first principles and
run all-loci agreement check before committing. Self-anchor elimination (F-P13-002): recommend
as E-11 template convention at S-7.02 — intra-document :NNN self-citations prohibited; use
content-based locators.

**Streak:** 0/3. Pass 14 pending adversary dispatch. Trajectory-tail: →14→10→15.
