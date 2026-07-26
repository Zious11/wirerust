---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-26T23:00:00Z
cycle: "wave-086"
pass: 12
verdict: NOT_CONVERGED
novelty: "substantive"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 12

**Date:** 2026-07-26
**Pass:** 12 of N
**Verdict:** NOT CONVERGED
**Novelty:** substantive — 5th recurrence of inert/self-referential-predicate class; locus class: story-prescribed fixture-block annotations quoting literal flagged phrases; pass-9-added AC-183-007 block never re-swept with the Task-4/6 no-literal-phrase rule
**Tally:** 10 findings — 0 CRIT / 1 HIGH / 4 MED / 5 LOW + 5 NITs (all fixed)
**Status:** REMEDIATED — D-529 state burst; STORY-182 v2.1→v2.2 + STORY-183 v2.1→v2.2
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Positives from adversary:** All counts/hashes/citations exact; predicate family (concat!-needle) non-inert; forbidden-committed guard non-inert; hermetic Task-9 harness sound; DF-GREEN-DOC-TENSE-SWEEP v6 DIRECT-SCRUB OBLIGATION verified; both-directions predicate verification for F-001 (zero annotation-vs-pattern matches); DF-INPUT-HASH-CANONICAL-001 compliance confirmed (9a0f34c / 9c9b12f unchanged)

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P12-001 | HIGH | STORY-183 | FIXED v2.2 | AC-183-007 prescribed `#` annotations at :572/:583/:597/:604/:611 quoted 5 literal flagged phrases ("currently asserts" / "falls to the wildcard" / "currently has no" / "currently satisfied by" / "will be GREEN currently"); file is in-scan post-story delivery → 5 false FAILs → green-doc-tense-gate CI red on a gate that must exit 0 at 6 ACs. 5th self-referential-predicate recurrence (P3/P5/P9/P11/P12). pass-9-added block never re-swept with Task-4/6 no-literal-phrase rule. |
| F-W86S-P12-002 | MED | STORY-182 | FIXED v2.2 | Task 9 Env A discriminator zero-SKIP assertion doubly inert: (a) no `2>&1` redirect — SKIP messages land on stderr, not in coverage-out.txt; (b) wrong test — the manifest test never calls `fixture_present`, so SKIP count is structurally zero regardless of fixture state. False-green wave-gate signal. |
| F-W86S-P12-003 | MED | STORY-183 | FIXED v2.2 | Strict-TDD RED ordering omitted Task 3 (suffix-aware `_is_comment_line`); 4 `.py` BAD_CASES cannot clear the RED→GREEN gate until Task 3 is delivered → GREEN checkpoint is unreachable as ordered. |
| F-W86S-P12-004 | MED | STORY-182 | FIXED v2.2 | Resolver-coupling comment overclaimed "fails in CI" for inverted ordering; actual failure surface is fixture-bearing host only — CI lacks local-samples, so the CI inverted-ordering failure path is absent. |
| F-W86S-P12-005 | MED | STORY-183 | FIXED v2.2 | AC-183-009 inert w.r.t. Task-13 scrub: targets are unscanned docstring lines and non-matching `:125` — cannot distinguish scrub-done from scrub-skipped despite DF-GREEN-DOC-TENSE-SWEEP v6 DIRECT-SCRUB OBLIGATION applying to those lines. |
| F-W86S-P12-006 | LOW | STORY-183 | FIXED v2.2 | git wildmatch semantics: `src/*.rs` crosses `/` and subsumes `src/**/*.rs`; prose implied top-level-only coverage; git deduplicates so no double-count, but 5 documentation loci described the semantics incorrectly. |
| F-W86S-P12-007 | LOW | STORY-183 | FIXED v2.2 | Task 10 left bin/check-green-doc-tense:87-88 false claim ("string-literal FPs avoided") uncorrected; string-literal comment-shaped lines ARE flagged by design per EC-005/Task 5. |
| F-W86S-P12-008 | LOW | STORY-182 | FIXED v2.2 | SKIP diagnostic displayed committed path even for never-commit (licensing-restricted) fixtures, creating a licensing trap for implementers. |
| F-W86S-P12-009 | LOW | cross-story | FIXED v2.2 | Both stories touch ci.yml in disjoint regions with no mutual reference; concurrent delivery creates silent rebase-conflict risk. |
| F-W86S-P12-010 | LOW | STORY-183 | FIXED v2.2 | AC-183-009 "covers both" understated the D-525 three-item scope: (a) registration, (b) test_lint_cycle_artifact.py, (c) test_compute_input_hash.py. |

---

## Findings Detail

### F-W86S-P12-001 (HIGH) — AC-183-007 fixture-block annotations quote literal flagged phrases

**Story:** STORY-183
**Status:** FIXED v2.2
**5th recurrence:** inert/self-referential-predicate class. Prior instances: P3-F005 (vacuous-pass,
STORY-183), P5-F001 (vacuous-pass, STORY-182), P9-F003 (vacuous-pass, STORY-182), P11-F001
(false-FAIL, STORY-182). P12 is a false-FAIL instance in STORY-183.
**Description:** AC-183-007 contains a table of `#` fixture-block annotations at lines :572, :583,
:597, :604, :611. These annotations document the expected gate behavior by quoting the literal
phrases the tool scans for. Specifically, the annotation text contained the contiguous literal
strings:

- `:572` — "currently asserts"
- `:583` — "falls to the wildcard"
- `:597` — "currently has no"
- `:604` — "currently satisfied by"
- `:611` — "will be GREEN currently"

Because STORY-183 is itself a file scanned by `bin/check-green-doc-tense` post-delivery (the
`.factory/stories/` tree is included in the scan corpus), each of these 5 annotations would
trigger a TIER-1 false FAIL when the gate runs on the delivered state. The gate is required to
exit 0 at 6 ACs (must-exit-0 clause); 5 false FAILs would make the gate red, contradicting
every must-exit-0 clause in the story.

The root cause is structural: the pass-9 remediation added this annotation block
(F-W86S-P9-010) but no-literal-phrase sweep was applied to the AC-183-007 text at that time.
The Task-4/6 no-literal-phrase rule (established in earlier passes) was not retroactively
enforced on pass-9-added content.

**Fix (v2.2):** All 5 annotations reworded to describe-without-quoting (e.g., "the currently-
asserts phrase class" instead of quoting the literal string); no-literal-phrase sweep obligation
clause added to AC-183-007 prose; both-direction zero-match verification recorded (zero
annotation-vs-pattern matches confirmed; literal phrases absent from annotation text confirmed).

---

### F-W86S-P12-002 (MED) — Task 9 Env A discriminator zero-SKIP assertion doubly inert

**Story:** STORY-182
**Status:** FIXED v2.2 per orchestrator ruling
**Description:** The Env A discriminator block in Task 9 contained a zero-SKIP count assertion
(asserting that no SKIP messages appear when fixtures are present). This assertion was inert for
two independent reasons:

1. **No `2>&1` redirect:** SKIP messages are written to stderr, not stdout. The coverage-out.txt
   file captures only stdout. The assertion was testing a grep against a file that would never
   contain SKIP messages regardless of the fixture state.

2. **Wrong test target:** The manifest test (`test_fixture_manifest_all_present`) does not call
   `fixture_present()` — it calls a different verification path. The SKIP count from
   `fixture_present()` is structurally zero in this test regardless of whether fixtures exist on
   disk.

Together, the assertion could never fail even in a clean worktree with no fixtures. It produced
a false-green Env A discriminator result.

**Orchestrator ruling:** Zero-SKIP assertion deleted from discriminator block; the 4/4 grep
match against fixture names remains the sole discriminator. Sequential-overwrite note added
for shared coverage-out.txt (Env A and Env B both write to the same file name).

**Fix (v2.2):** Per orchestrator ruling applied.

---

### F-W86S-P12-003 (MED) — Strict-TDD RED ordering omitted Task 3

**Story:** STORY-183
**Status:** FIXED v2.2
**Description:** The strict-TDD RED→GREEN ordering prescribed Task 7 (add BAD_CASES for `.rs`
files) and Task 8 (add BAD_CASES for `.py` files) as the RED step, with Task 6 (add patterns)
as the GREEN step. However, 4 of the `.py` BAD_CASES introduced by Tasks 7/8 require the
suffix-aware `_is_comment_line` extension (Task 3) to clear. Without Task 3, the runner cannot
correctly classify `.py` comment lines, so these 4 cases cannot pass Task 6's GREEN checkpoint.
The GREEN checkpoint is unreachable in the prescribed ordering.

**Fix (v2.2):** Task 3 inserted as pre-RED step (ordering: Task 1 → Task 3 → Tasks 7/8 RED →
Task 6 GREEN → remaining tasks). The `.py` comment-line classification from Task 3 is required
before the `.py` BAD_CASES can be exercised.

---

### F-W86S-P12-004 (MED) — Resolver-coupling comment overclaimed "fails in CI" for inverted ordering

**Story:** STORY-182
**Status:** FIXED v2.2
**Description:** A resolver-coupling comment stated that delivering the FIXTURE_MANIFEST and
`fixture_present()` in inverted order "fails in CI." This overclaims the failure surface: the
inverted-ordering failure requires a fixture-bearing host (a developer machine with
local-samples present). In CI, local-samples are absent, so the inverted-ordering failure
path does not activate — CI would show a SKIP rather than a FAIL.

**Fix (v2.2):** Comment reworded to split the two failure environments precisely: (a) on a
fixture-bearing developer host, inverted ordering produces FAIL because `fixture_present()`
finds files but the manifest constant is not yet registered; (b) in CI (no local-samples),
tests skip and the ordering error is silent.

---

### F-W86S-P12-005 (MED) — AC-183-009 inert w.r.t. Task-13 scrub

**Story:** STORY-183
**Status:** FIXED v2.2
**Description:** AC-183-009 specified acceptance criteria for the Task-13 scrub of stale tense
violations from the scanned corpus. However, the AC's test predicates targeted either (a)
unscanned docstring lines (Python `"""..."""` content, which STORY-183 explicitly defers via
DRIFT-docstring-scan) or (b) the line `:125` which does not match the TIER-1 patterns. A
passing AC-183-009 therefore cannot distinguish between "Task 13 was executed correctly" and
"Task 13 was skipped" — the AC is satisfied regardless.

This violates DF-GREEN-DOC-TENSE-SWEEP v6's DIRECT-SCRUB OBLIGATION which requires that the
scrubbed content be verifiably absent post-delivery.

**Fix (v2.2):** Three mechanical `grep-count==0` predicates added:
1. `grep -c "RED GATE version" bin/check-green-doc-tense.py` must return 0 (original stale locus)
2. `grep -c "MUST FAIL until bin/lint-cycle-artifact" tests/test_lint_cycle_artifact.py` must
   return 0 (second stale locus)
3. `grep -c "TC1" tests/test_compute_input_hash.py` must return 0 (or be verified not stale)

These predicates fail exactly when Task 13 is skipped, making the AC non-vacuous.

---

### F-W86S-P12-006 (LOW) — git wildmatch `src/*.rs` subsumes `src/**/*.rs`

**Story:** STORY-183
**Status:** FIXED v2.2 per orchestrator ruling
**Description:** The git wildmatch semantics for `src/*.rs` in a `git ls-files` invocation
cross the `/` boundary — `src/*.rs` matches all files directly under `src/` AND, in git's
wildmatch with FNM_PATHNAME disabled in this context, also files in subdirectories. The merged
4-glob (`git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py`) therefore has `src/*.rs`
subsuming `src/**/*.rs`. Git deduplicates the output so no file is double-counted, but 5
documentation loci in the story described the semantics as "src/*.rs covers top-level only"
which is incorrect.

**Orchestrator ruling:** The merged 4-glob is kept for explicitness (both globs present makes
intent clear to implementers); 5 documentation loci corrected to state the true git semantics
(git dedup ensures no double-count; both globs explicit for clarity).

**Fix (v2.2):** Per orchestrator ruling applied.

---

### F-W86S-P12-007 (LOW) — Task 10 left false claim about string-literal FPs

**Story:** STORY-183
**Status:** FIXED v2.2
**Description:** Task 10 described bin/check-green-doc-tense:87-88 and included the claim that
string-literal false positives are "avoided" by the suffix-aware anchoring. This is incorrect:
comment-shaped lines inside string literals ARE flagged by the tool — this is documented as
intentional behavior in EC-005 and Task 5 (string-literal comment-shaped lines are treated
as normal comment candidates per the tool's design). Claiming FPs are "avoided" contradicts
EC-005/Task 5.

**Fix (v2.2):** Task 10 extended: suffix-aware anchoring description corrected; note added
that string-literal comment-shaped lines ARE flagged by design, cross-referencing EC-005 and
Task 5 for the rationale.

---

### F-W86S-P12-008 (LOW) — SKIP diagnostic showed committed path for licensing-restricted fixtures

**Story:** STORY-182
**Status:** FIXED v2.2
**Description:** The SKIP diagnostic message displayed the committed fixture path for all
skipped fixtures, including those that are never committable (licensing-restricted captures
covered by CC-BY-4.0 or commercial license). Showing the committed path for a
never-committable fixture implies to implementers that they should commit that file to
`tests/fixtures/`, which would be a licensing violation.

**Fix (v2.2):** Message logic branches on `COMMITTED_FIXTURES.contains(fixture_name)`:
- Committable fixtures: show `tests/fixtures/<name>` path + fetch instruction.
- Non-committable fixtures: show `local-samples/<name>` path + explicit "DO NOT COMMIT —
  <license> licensing applies" clause.

---

### F-W86S-P12-009 (LOW) — Both stories touch ci.yml with no mutual reference

**Story:** cross-story (STORY-182 + STORY-183)
**Status:** FIXED v2.2
**Description:** STORY-182 and STORY-183 both modify `ci.yml` in disjoint regions (STORY-182
modifies the E2E fixtures step; STORY-183 modifies the bin-selftest job). No mutual reference
existed in either story, creating a silent rebase-conflict risk if the two stories are
delivered concurrently or in close succession. Without awareness of the sibling story's CI
region, an implementer would not know to rebase before opening the second PR.

**Fix (v2.2):** Sibling-story note added to Notes §Develop PR in both stories: identifies the
disjoint ci.yml regions modified by each story and prescribes "rebase if the sibling story's
PR is in-flight" before pushing.

---

### F-W86S-P12-010 (LOW) — AC-183-009 "covers both" understated three-item scope

**Story:** STORY-183
**Status:** FIXED v2.2
**Description:** AC-183-009 stated that the test step "covers both" items. The actual scope
from D-525 (PG-W84-012 extended) is three items: (a) registration of bin-selftest as a
required-status-check, (b) adding test_lint_cycle_artifact.py to the CI job step, and
(c) adding test_compute_input_hash.py to the CI job step. "Covers both" implied a two-item
scope and omitted item (c).

**Fix (v2.2):** AC-183-009 enumerated all three items explicitly: (a) registration +
(b) test_lint_cycle_artifact.py + (c) test_compute_input_hash.py.

---

## NIT-Observations (5 items — all actioned)

1. **NIT-01 (ACTIONED):** `:466-474` span correction in STORY-182 AC body — a line-citation
   span was off by one line due to a prior insertion; corrected to match the actual enclosing
   block boundaries.

2. **NIT-02 (ACTIONED):** Line-4 scope text in STORY-183 Task 3 description was ambiguous
   about which file extension class was being handled; reworded for precision.

3. **NIT-03 (ACTIONED):** "sole authoritative" wording in STORY-182 task description changed
   to "primary citation; non-exhaustive" — the canonical tool is primary, not the exclusive
   authority (DF-SIBLING-SWEEP-001 secondary sources remain valid).

4. **NIT-04 (ACTIONED):** N/4 prose in STORY-182 (three loci) annotated with
   "(currently 4 — tracks FIXTURE_MANIFEST.len())" to prevent future drift when the manifest
   grows beyond 4 entries.

5. **NIT-05 (ACTIONED):** `test_fixture_manifest_report` deliberate-exclusion sentence added
   noting that this test is intentionally excluded from DF-TEST-CITATION-SWEEP-001 item 4
   scope — the test is a manifest-level report, not an individual fixture citation.

---

## EXECUTION-REQUIRED Flags

The following items were recorded by the adversary as requiring execution evidence at
delivery or gate time. They are NOT blocking the spec review, but must be supplied
before pass-13 adversary dispatch:

**(i) ITI e2e diverse-expectation baseline:** Current pass/fail of the diverse ITI e2e test
with the 66/20/46 expectation on develop=e8841d76 (PR #439 gate-fix 0ab6f52e set this
expectation; verify it still holds on e8841d76 back-merge).

**(ii) Python selftest baseline exit codes:** Confirm `bin/test_compute_input_hash.py` and
the forthcoming `bin/test_check_green_doc_tense.py` both exit 0 on current develop HEAD.
These are the selftests gated by PG-W84-012 and will be wired in AC-183-009; their baseline
exit codes must be documented at delivery.

---

## Verified-Clean Table (pass-12 adversary confirmation)

| Verification Item | Result |
|-------------------|--------|
| All finding counts / pass tallies | EXACT |
| Input-hash values 9a0f34c / 9c9b12f | EXACT — canonical Python tool |
| Citations to PG-W84-012, PG-W84-010, PG-W85-003 | EXACT |
| concat!-needle predicate (F-P11-001 fix) non-inert | CONFIRMED — count 4, no comment occurrences |
| Forbidden-committed guard (tests/fixtures/ only) non-inert | CONFIRMED — non-committable fixture reject path sound |
| Hermetic Task-9 harness (git -C <tmp>) sound | CONFIRMED — outside-repository error eliminated |
| DF-GREEN-DOC-TENSE-SWEEP v6 compliance | CONFIRMED — no TIER-1 regressions from v2.1 edits |
| DF-INPUT-HASH-CANONICAL-001 | CONFIRMED — hashes unchanged, canonical Python tool |
| DF-CANONICAL-FRAME-HOLDOUT-001 | N/A (E-11 template stories) |
| DF-SIBLING-SWEEP-001 | CONFIRMED — sweep performed post-remediation |
| Intra-document line-citation re-anchor sweep | CLEAN — sweep executed per PG-W86-014 mitigation |

---

## Pass-12 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: substantive — 5th recurrence of inert/self-referential-predicate class (locus class:
story-prescribed fixture annotations quoting literal flagged phrases). HIGH = pass-12 new
instance in STORY-183; distinct from the P11 instance which was in STORY-182 harness comments.

Severity decay continues: P10 0C/0H/5M/6L → P11 0C/1H/6M/7L → P12 0C/1H/4M/5L.

Pass-13 next after D-529 remediation burst.

Pass tallies (P1–P12): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14 / 10.
Total across all passes: 213 findings. Canonical hashes: 9a0f34c / 9c9b12f.

---

## Remediation (D-529)

**Date:** 2026-07-26
**Burst:** D-529 STATE BURST — WAVE-86 ADVERSARIAL PASS 12 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 10 findings FIXED at STORY-182 v2.2 / STORY-183 v2.2 with per-fix grep evidence
per PG-W86-010 mandate. Full DF-SIBLING-SWEEP-001 sweep performed. Intra-document
line-citation re-anchor sweep executed per PG-W86-014 mitigation (table clean).

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P12-001 (HIGH) | FIXED v2.2 | STORY-183 | 5 annotations reworded to describe-without-quoting; no-literal-phrase sweep obligation added to AC-183-007; both-direction zero-match verification recorded |
| F-W86S-P12-002 (MED) | FIXED v2.2 per orch. ruling | STORY-182 | Zero-SKIP line deleted from discriminator block; 4/4 grep sole discriminator; sequential-overwrite note for coverage-out.txt |
| F-W86S-P12-003 (MED) | FIXED v2.2 | STORY-183 | Task 3 inserted pre-RED; ordering: Task 1 → Task 3 → Tasks 7/8 RED → Task 6 GREEN |
| F-W86S-P12-004 (MED) | FIXED v2.2 | STORY-182 | Comment split: fixture-bearing host → FAIL; CI (no local-samples) → silent SKIP |
| F-W86S-P12-005 (MED) | FIXED v2.2 | STORY-183 | Three grep-count==0 predicates added to AC-183-009; non-vacuous w.r.t. Task-13 skip |
| F-W86S-P12-006 (LOW) | FIXED v2.2 per orch. ruling | STORY-183 | Merged 4-glob kept; 5 documentation loci corrected to true git wildmatch semantics |
| F-W86S-P12-007 (LOW) | FIXED v2.2 | STORY-183 | Task 10 corrected: string-literal comment-shaped lines flagged by design; EC-005/Task 5 cross-ref |
| F-W86S-P12-008 (LOW) | FIXED v2.2 | STORY-182 | SKIP diagnostic branches on COMMITTED_FIXTURES.contains; non-committable fixtures show local-samples path + do-not-commit licensing clause |
| F-W86S-P12-009 (LOW) | FIXED v2.2 | cross-story | Sibling-story note added to both Notes §Develop PR; disjoint ci.yml regions documented; rebase-if-in-flight note |
| F-W86S-P12-010 (LOW) | FIXED v2.2 | STORY-183 | All three D-525 items enumerated: (a) registration + (b) test_lint_cycle_artifact.py + (c) test_compute_input_hash.py |

**5 NITs actioned:** :466-474 span correction; line-4 scope text precision; "sole authoritative"
→ "primary citation; non-exhaustive"; N/4 annotated "(currently 4 — tracks FIXTURE_MANIFEST.len())"
at 3 loci; test_fixture_manifest_report deliberate-exclusion sentence (DF-TEST-CITATION-SWEEP-001
item 4 considered).

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**Intra-document line-citation re-anchor sweep (PG-W86-014 mitigation):**
Post-edit sweep of all :NNN citations in both stories executed. Table clean.

**Orchestrator rulings recorded:**
- F-W86S-P12-002: Zero-SKIP assertion deleted (doubly inert); 4/4 grep sole discriminator.
- F-W86S-P12-003: Task 3 must precede RED checkpoint; ordering Task 1 → Task 3 → Tasks 7/8 → Task 6.
- F-W86S-P12-006: Merged 4-glob kept for explicitness; git wildmatch semantics corrected at 5 loci.
- F-W86S-P12-010: Three-item PG-W84-012-extended scope enumerated: registration + two test steps.

**STORY-INDEX:** v4.07→v4.08 (wave-86 row v2.1→v2.2 both stories; no numeric totals changed).

**5th self-referential-predicate recurrence codified:** locus class = story-prescribed fixture
annotations quoting literal flagged phrases. Standing discipline: any pass that ADDS prose
naming a scanned pattern must run the no-literal-phrase sweep over the added text before commit.
Imposed as standing story-writer dispatch discipline as of D-529.

**Streak:** 0/3. Pass 13 pending adversary dispatch. Trajectory-tail: →11→14→10.
