---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-27T00:00:00Z
cycle: "wave-086"
pass: 16
verdict: NOT_CONVERGED
novelty: "med-high"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 16

**Date:** 2026-07-27
**Pass:** 16 of N
**Verdict:** NOT CONVERGED
**Novelty:** med-high — 2 genuine efficacy findings from codebase (Pattern-31 contiguity blind spot with
live stale site iec104_analyzer_tests.rs:6948-6953; Pattern-32 bare "RED GATE: " heading lines
= residual-coverage overclaim) + 4 v2.5-induced propagation regressions. FOURTH consecutive zero-HIGH pass.
**Tally:** 13 findings — 0 CRIT / 0 HIGH / 6 MED / 3 LOW + 4 NITs (all fixed)
**Status:** REMEDIATED — D-533 state burst; STORY-182 v2.5→v2.6 + STORY-183 v2.5→v2.6
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Positives from adversary (verified-clean axes):** ground-truth predicate correctness; no-literal-phrase
standing discipline (D-529) satisfied; DF-SIBLING-SWEEP-001 compliance confirmed; pathspec subsumption
direction consistent at all 7 loci (v2.5 holds); self-anchor elimination complete (zero :NNN intra-doc
self-citations); canonical hashes 9a0f34c/9c9b12f unchanged; Task-8 8a/8b split verified sound;
set -euo pipefail confirmed in all main Task-8b blocks; ci.yml order-dependence labels present;
PASS/FAIL predicate-first form holds in all main verification blocks; EXECUTION-REQUIRED flags (i)-(ix) consistent.

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P16-001 | MED | STORY-183 | FIXED v2.6 | set -euo propagation gap — Task-9 hermetic-harness confirmation block retained `set -o pipefail` without `-e`/`-u` flags after v2.5 hardening sweep; one verification block missed the P15 discipline. |
| F-W86S-P16-002 | MED | STORY-182 | FIXED v2.6 | move-aside guard condition inconsistency — v2.5 added source-existence guards but two of three trap-restore sites use `[ -e "$BACKUP" ]` (file-or-dir) while a third uses `[ -f "$BACKUP" ]` (file-only); all should be `-f` for consistency. |
| F-W86S-P16-003 | MED | STORY-183 | FIXED v2.6 (DEFERRED-SCRUB) | Pattern-31 contiguity blind spot — tests/iec104_analyzer_tests.rs:6948-6953 is a stale "RED GATE: " site that currently falls through the `_` catch-all; post-STORY-180, the timed-command detection landed but these lines were not scrubbed; deferred to DRIFT-stale-red-scrub; regex widening declined per scope-containment ruling D-533. |
| F-W86S-P16-004 | MED | STORY-182 | FIXED v2.6 | ci.yml citation-currency grep anchor not updated — v2.5 fixed README row to include "ITI CC-BY-4.0" prefix but the ci.yml citation-currency verification step still grep-anchors on the old paraphrase form; anchor and canonical string are now inconsistent. |
| F-W86S-P16-005 | MED | STORY-183 | FIXED v2.6 | Exit-code contradiction — v2.5 added an exit-code semantics table (exit 0 = clean; exit 1 = violations; exit 2+ = error) but the BAD_CASE verification form in AC-183-009 still uses `[ $? -ne 0 ]`, accepting exit 2+ (Python crash) as a successful BAD_CASE run; contradicts the table. |
| F-W86S-P16-006 | MED | STORY-183 | FIXED v2.6 (ACCEPTED-RESIDUAL) | Pattern-32 bare-token heading lines — "RED GATE: " appearing as a bare token in heading-level lines (e.g., `## RED GATE: ...`) is not caught by the phrase-level pattern match; STORY-183 v2.5 claimed residual coverage sufficient but bare-token-only heading lines represent an uncovered subclass; accepted residual (phrase-level-by-design) per ruling D-533. |
| F-W86S-P16-007 | LOW | STORY-183 | FIXED v2.6 | 21-pass derivation arithmetic stale — v2.5 added an exit-code semantics predicate to AC-183-009 (exit 2+ error-condition check) making the AC-183-009 gate-check set 10 items, not 9; derivation still reads 8+9+4=21; should be 8+10+4=22. |
| F-W86S-P16-008 | LOW | STORY-182 | FIXED v2.6 | DRIFT-e2e-sibling-harnesses wording residual — STORY-182 v2.5 Notes describes e2e_corpus_smoke_tests.rs as using "the same fixture_present idiom" as enip_e2e_real_pcaps_tests.rs; truth: e2e_corpus_smoke_tests.rs uses a directory-level skip VARIANT at :206-224, not the identical fixture_present conditional. |
| F-W86S-P16-009 | LOW | STORY-183 | FIXED v2.6 | Pattern-33 task comment past-tense — Task-8a comment prescribing the Pattern-33 description block uses "was introduced to handle" (past-tense); violates E-11 active-imperative present-tense convention. |

---

## Findings Detail

### F-W86S-P16-001 (MED) — set -euo propagation gap in Task-9 hermetic-harness block

**Story:** STORY-183
**Status:** FIXED v2.6 — `set -euo pipefail` applied to Task-9 block

**Description:** STORY-183 v2.5 introduced `set -euo pipefail` uniformly in Task-8b blocks (fixing
F-W86S-P15-003). The v2.5 hardening sweep did not reach the Task-9 hermetic-harness confirmation
block, which verifies that the hermetic script copy produces a FAIL line when run against violating.py.
That block retains `set -o pipefail` without `-e` or `-u`.

Without `set -e`, a failing intermediate command in the hermetic-harness sequence (e.g., a failed `cp`
or `git add`) does not abort the block. Execution continues and may emit false-PASS output.

**Fix (v2.6):** `set -o pipefail` replaced with `set -euo pipefail` in Task-9 block. All shell
verification blocks now consistently use the full three-flag form.

---

### F-W86S-P16-002 (MED) — move-aside guard condition inconsistency

**Story:** STORY-182
**Status:** FIXED v2.6 — all trap-restore sites normalized to `[ -f "$BACKUP" ]`

**Description:** STORY-182 v2.5 added source-existence guards to move-aside procedures (fixing
F-W86S-P15-002). The fix normalized the `mv` guard to `[ -f source ]` but the trap-restore guards
were written inconsistently: two sites use `[ -e "$BACKUP" ]` (succeeds for directories and symlinks,
not just regular files), while the third uses `[ -f "$BACKUP" ]` (regular file only). In all practical
cases `$BACKUP` is a regular `.pcap` file so the difference is inert, but the inconsistency is a
maintenance hazard and violates the uniform-guard convention established in v2.5.

**Fix (v2.6):** All three trap-restore sites normalized to `[ -f "$BACKUP" ]`. Note added: "Use `-f`
(regular file test) not `-e` (existence test) to guard against accidental directory matches."

---

### F-W86S-P16-003 (MED) — Pattern-31 contiguity blind spot with live stale site

**Story:** STORY-183
**Status:** FIXED v2.6 (DEFERRED-SCRUB) — live stale site added to story's deferred-scrub list;
regex widening declined per scope-containment ruling D-533

**Description:** Pattern-31 in the TIER-1 pattern table matches "RED GATE: " tokens in specific
phrase-context patterns. Codebase inspection reveals that tests/iec104_analyzer_tests.rs lines 6948-6953
contain stale "RED GATE: " prose that currently falls through the `_` catch-all — not matched by
Pattern-31 or any other TIER-1 pattern. These lines are stale post-STORY-180: the timed-command
detection feature landed in v0.13.2 but the test file comments at :6948-6953 were not updated.

This is a genuine efficacy finding: a live codebase site that should be flagged by the deployed checker
is not flagged. The cause is a contiguity limitation in Pattern-31's regex scope — the pattern does not
match the specific surrounding context at those lines.

**Orchestrator ruling (D-533):** Regex widening is NOT applied mid-convergence (scope containment).
The live stale site at iec104_analyzer_tests.rs:6948-6953 is added to STORY-183's deferred-scrub list,
flowing to DRIFT-stale-red-scrub as a third adjudicated site. Pattern-31/32 contiguity limitation
documented in STORY-183 v2.6 for implementer awareness; regex widening deferred to post-delivery.

**Fix (v2.6):** iec104_analyzer_tests.rs:6948-6953 added to STORY-183's deferred-scrub sites list
with note: "currently falls through the `_` catch-all; stale post-STORY-180; past-tense reword
prescribed; Pattern-31/32 regex extension deferred per D-533 scope-containment ruling."

---

### F-W86S-P16-004 (MED) — ci.yml citation-currency grep anchor not updated

**Story:** STORY-182
**Status:** FIXED v2.6 — ci.yml grep anchor updated to match canonical "ITI CC-BY-4.0" prefix

**Description:** STORY-182 v2.5 corrected the README.md provenance table row to use the exact canonical
"ITI CC-BY-4.0" prefix string (fixing F-W86S-P15-001). The ci.yml citation-currency verification step
was not updated correspondingly: it still grep-anchors on the old paraphrase form rather than the new
canonical "ITI CC-BY-4.0" prefix string.

Result: the ci.yml gate would pass on the old paraphrase row and fail on the newly correct canonical
row — opposite of the intended behavior.

**Fix (v2.6):** ci.yml grep anchor updated to `grep -q "ITI CC-BY-4.0"` at all citation-currency
verification sites in STORY-182. AC-182-002 cross-reference note updated to cite the ci.yml grep form.

---

### F-W86S-P16-005 (MED) — Exit-code contradiction in BAD_CASE verification form

**Story:** STORY-183
**Status:** FIXED v2.6 — BAD_CASE verification form tightened to explicit exit 1 check

**Description:** STORY-183 v2.5 added an exit-code semantics table to AC-183-009:
- exit 0: scanner ran and found zero violations
- exit 1: scanner ran and found ≥1 violation
- exit 2+: argument error, import failure, or runtime exception

The table explicitly flags exit 2+ as an error condition that must fail the gate regardless of case type.
However, the BAD_CASE verification form still reads `[ $? -ne 0 ]`, accepting any non-zero exit including
2+ (Python crash). A conftest crash on a BAD_CASE run satisfies the `ne 0` check, contradicting the
table's declaration.

**Fix (v2.6):** BAD_CASE verification form updated to explicitly check for exit 1:
```bash
EXIT=$?
[ "$EXIT" -eq 1 ] || { echo "FAIL: expected exit 1 (violations), got $EXIT"; exit 1; }
```
Consistent with the exit-code semantics table added in v2.5.

---

### F-W86S-P16-006 (MED) — Pattern-32 bare-token heading lines = residual-coverage overclaim

**Story:** STORY-183
**Status:** FIXED v2.6 (ACCEPTED-RESIDUAL) — bare-token heading class documented as accepted residual
per ruling D-533

**Description:** STORY-183 v2.5 claims residual coverage is sufficient after the TIER-1 pattern set
expansion. Codebase review reveals that "RED GATE: " appearing as a bare token in Markdown heading
lines (e.g., `## RED GATE: Expected Failure`) is not caught by any TIER-1 pattern. The phrase-level
patterns match "RED GATE: " in non-heading prose contexts; heading-level lines are structurally distinct
(leading `#` characters) and fall through the current pattern set.

This is a genuine efficacy finding: heading-level "RED GATE: " lines represent an uncovered subclass.

**Orchestrator ruling (D-533):** The bare-token heading class is added to the story's accepted residuals
as "phrase-level-by-design." The pattern checker targets phrase-context prose; heading-level bare tokens
are a known limitation accepted by design. Bare-token heading detection deferred to avoid scope creep
mid-convergence.

**Fix (v2.6):** Accepted-residual class documented in STORY-183 v2.6 residuals section: "bare 'RED GATE: '
tokens in Markdown heading lines (e.g., `## RED GATE: ...`) are not caught by phrase-level patterns;
accepted-by-design (phrase-level scope). Heading-level detection deferred to a future story."

---

### F-W86S-P16-007 (LOW) — 21-pass derivation arithmetic stale

**Story:** STORY-183
**Status:** FIXED v2.6 — derivation note updated to 22 passes (8+10+4)

**Description:** STORY-183 v2.5 added exit-code semantics to AC-183-009, including an explicit exit-2+
error check, adding one gate-check predicate to the AC-183-009 verification set. The Task-10 derivation
note (added in v2.5 fixing F-W86S-P15-009) reads:

"21 = 8 TIER-1 pattern self-tests (Patterns 30-37 via tdd_mode, Task-8b) + 9 AC-183-009 gate-check
predicates + 4 AC-183-007 scanner-output-format assertions."

The AC-183-009 gate-check set now has 10 predicates (9 original + 1 exit-2+ error guard added in v2.5).
The derivation still reads 9, making the expected total 21 rather than the correct 22.

**Fix (v2.6):** Derivation note updated: "22 = 8 TIER-1 pattern self-tests (Patterns 30-37) + 10
AC-183-009 gate-check predicates (9 case-outcome predicates + 1 exit-2+ error guard added v2.5) +
4 AC-183-007 scanner-output-format assertions. If this arithmetic changes, update before commit."

---

### F-W86S-P16-008 (LOW) — DRIFT-e2e-sibling-harnesses wording residual

**Story:** STORY-182
**Status:** FIXED v2.6 — Notes §"Sibling e2e harnesses" wording corrected for e2e_corpus_smoke_tests.rs

**Description:** STORY-182 v2.5 Notes §"Sibling e2e harnesses (deferred)" describes e2e_corpus_smoke_tests.rs
as using "the same fixture_present idiom" as enip_e2e_real_pcaps_tests.rs. Inspection of that file shows
it uses a directory-level skip variant at lines ~206-224: the skip condition tests whether a local samples
directory is present, not whether a specific fixture_present boolean is set. It is in the same class
(directory-absent = skip), but uses a structurally different idiom from fixture_present.

The same wording inaccuracy propagated to the STATE.md DRIFT-e2e-sibling-harnesses row.

**Fix (v2.6):** STORY-182 Notes §"Sibling e2e harnesses" updated: e2e_corpus_smoke_tests.rs described
as "directory-level skip VARIANT (:206-224) — skips when local samples directory absent, not via
fixture_present; same class but different idiom." STATE.md DRIFT-e2e-sibling-harnesses row aligned per D-533.

---

### F-W86S-P16-009 (LOW) — Pattern-33 task comment uses past-tense voice

**Story:** STORY-183
**Status:** FIXED v2.6 — Task-8a comment reworded to active present-tense imperative

**Description:** Task-8a in STORY-183 v2.5 includes an explanatory comment in the prescribed
Pattern-33 code block using past-tense narrative voice ("was introduced to handle"). The E-11
writing convention (established in v2.5 NIT-01 fix per F-W86S-P15) requires active-imperative
present-tense voice for task-body prose. The past-tense framing also invites drift.

**Fix (v2.6):** Comment reworded to active present-tense imperative form.

---

## NIT-Observations (4 items — all actioned)

1. **NIT-01 (ACTIONED, STORY-183):** Task-9 code block fence did not include a language tag; added
   `bash` annotation for rendering consistency with all other shell blocks in the story.

2. **NIT-02 (ACTIONED, STORY-182):** Notes §"move-aside rationale" — one passive-voice phrase survived
   the v2.5 NIT-01 fix: "the backup is not created" reworded to active form "the procedure creates no
   backup."

3. **NIT-03 (ACTIONED, STORY-183):** Cross-reference in Task-9 reads "see Task 8" without specifying
   Task 8a or Task 8b — ambiguous post-split (Task-8 split introduced in v2.4 per D-531). Updated
   to "see Task 8b — RED gate verification."

4. **NIT-04 [process-gap] (ACTIONED, STORY-183):** wc-c/stat flip-flop across v2.3/v2.5 — the
   fixture-size gate verification used `wc -c` in v2.3, was changed to `stat` in v2.4, then
   changed back to `wc -c` in v2.5; the same gate was reworked twice with the current form
   correct. Process gap: the portability constraint (macOS `stat` vs GNU `stat` incompatible flags)
   was not documented precisely enough to prevent re-litigation. Story updated with a portability
   note explaining why `wc -c` is the portable form. Noted as E-11 convention candidate for future
   reference.

---

## EXECUTION-REQUIRED Flags (9 items — carried from pass-15 unchanged)

Items (i)-(ix) from pass-15-findings.md carried forward unchanged. No new EXECUTION-REQUIRED
flags added from pass-16.

**(i) Python selftest exit code:** Confirm `python3 bin/test_compute_input_hash.py` exits 0
on develop=e8841d76. Baseline for AC-183-009 item (c).

**(ii) cargo test/clippy on prescribed block:** Confirm `cargo test --all-targets` and
`cargo clippy --all-targets -- -D warnings` both exit 0 on develop=e8841d76.

**(iii) 66-finding expectation vs committed capture (CI-GATING):** Verify
`cargo test test_iec104_iti_diverse_e2e_expectations` passes with 66/20/46 expectation.

**(iv) sha256/size of fetched captures:** Document exact sha256 and byte-size of
`tests/fixtures/iec104-iti-diverse.pcap` on develop=e8841d76.

**(v) hermetic harness end-to-end:** Execute Task 9 hermetic harness (copy script into
fresh tmp dir, run against violating.py, confirm FAIL line appears).

**(vi) ci.yml step behavior on runner:** Confirm ci.yml step produces "Fixture coverage: N/4
committed" + explicit count-gate assertion passes on GitHub Actions runner against develop=e8841d76.

**(vii) git ls-files result sets (with dedup):** Document exact file count from
`git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py | sort -u | wc -l` on develop=e8841d76.

**(viii) Pattern self-test exit codes:** Confirm each of Patterns 30-37 produces at least one
FAIL line against violating.py in tdd_mode on develop=e8841d76.

**(ix) cargo e2e 66-finding CI gate (new — P15):** Run `cargo test
test_iec104_iti_diverse_e2e_expectations --release` on delivery branch; confirm 66/20/46 holds.

---

## Verified-Clean Table (pass-16 adversary confirmation)

| Verification Item | Result |
|-------------------|--------|
| All finding counts / pass tallies | EXACT |
| Input-hash values 9a0f34c / 9c9b12f | EXACT — canonical Python tool; unchanged |
| No-literal-phrase sweep (D-529 standing discipline) | CONFIRMED — no TIER-1 literals in annotation text |
| Intra-document :NNN self-citations | CONFIRMED ZERO |
| Pathspec subsumption direction (7-loci agreement) | CONFIRMED — src/*.rs strictly subsumes src/**/*.rs at all 7 loci in v2.5 |
| DF-GREEN-DOC-TENSE-SWEEP v6 compliance | CONFIRMED — no TIER-1 regressions from v2.5 edits |
| DF-SIBLING-SWEEP-001 | CONFIRMED — sweep performed; no sibling regressions |
| if:always() loci | CONFIRMED ZERO |
| Task-8 8a/8b split ordering | CONFIRMED — v2.5 split intact |
| PASS/FAIL-convention predicate-first form (main blocks) | CONFIRMED — all main blocks predicate-first in v2.5 |
| ci.yml order-dependence labels | CONFIRMED — present |
| Ground-truth axis | CONFIRMED CLEAN — zero findings |
| Inert-predicate axis | CONFIRMED CLEAN — zero live if:always() loci |
| Policy-compliance axis | CONFIRMED CLEAN — zero findings |

---

## Pass-16 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: med-high — 2 genuine efficacy findings from codebase (Pattern-31 contiguity blind spot,
live stale site iec104_analyzer_tests.rs:6948-6953; Pattern-32 bare-token heading class uncovered)
+ 4 v2.5-induced propagation regressions + 3 LOW + 4 NITs.
No HIGH findings. FOURTH consecutive zero-HIGH pass (P10, P14, P15, P16).

Severity profile: P14 0C/0H/3M/3L → P15 0C/0H/5M/6L → P16 0C/0H/6M/3L.
Genuine efficacy findings confirm that a pass is warranted despite the zero-HIGH streak.
Pattern-31/32 scope limitations documented; regex widening and bare-token heading detection
deferred per scope-containment ruling D-533.

HIGH count history: P10:0H → P11:1H → P12:1H → P13:2H → P14:0H → P15:0H → P16:0H.
Fourth consecutive zero-HIGH confirms HIGH-severity axis remains clean. Streak clock not
advanced due to remaining MED/LOW findings.

Pass tallies (P1–P16): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14 / 10 / 15 / 8 / 14 / 13.
Total across all passes: 263 findings. Canonical hashes: 9a0f34c / 9c9b12f.

Pass-17 next after D-533 remediation burst.

---

## Remediation (D-533)

**Date:** 2026-07-27
**Burst:** D-533 STATE BURST — WAVE-86 ADVERSARIAL PASS 16 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 13 findings FIXED at STORY-182 v2.6 / STORY-183 v2.6. STATE.md DRIFT rows updated per
F-W86S-P16-003 (DRIFT-stale-red-scrub +1 site) and F-W86S-P16-008 (DRIFT-e2e-sibling-harnesses
wording aligned).

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P16-001 (MED) | FIXED v2.6 | STORY-183 | `set -euo pipefail` added to Task-9 hermetic-harness block |
| F-W86S-P16-002 (MED) | FIXED v2.6 | STORY-182 | All trap-restore sites normalized to `[ -f "$BACKUP" ]` |
| F-W86S-P16-003 (MED) | FIXED v2.6 (DEFERRED-SCRUB) | STORY-183 | iec104_analyzer_tests.rs:6948-6953 added to deferred-scrub list; DRIFT-stale-red-scrub updated (D-533) |
| F-W86S-P16-004 (MED) | FIXED v2.6 | STORY-182 | ci.yml grep anchor updated to "ITI CC-BY-4.0" at all citation-currency verification sites |
| F-W86S-P16-005 (MED) | FIXED v2.6 | STORY-183 | BAD_CASE verification tightened to explicit `[ "$EXIT" -eq 1 ]` check |
| F-W86S-P16-006 (MED) | FIXED v2.6 (ACCEPTED-RESIDUAL) | STORY-183 | bare-token heading class documented as accepted residual (phrase-level-by-design); D-533 ruling |
| F-W86S-P16-007 (LOW) | FIXED v2.6 | STORY-183 | Task-10 derivation note updated to 22 passes (8+10+4) |
| F-W86S-P16-008 (LOW) | FIXED v2.6 | STORY-182 | Notes §"Sibling e2e harnesses" wording corrected; DRIFT-e2e-sibling-harnesses row aligned (D-533) |
| F-W86S-P16-009 (LOW) | FIXED v2.6 | STORY-183 | Pattern-33 task comment reworded to active present-tense |

**4 NITs actioned:** Task-9 fence tag (STORY-183); move-aside passive-voice residual (STORY-182);
Task-8/8b cross-reference specificity (STORY-183); wc-c/stat flip-flop [process-gap] noted +
portability rationale added (STORY-183).

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**Fourth zero-HIGH pass confirmed:** P10:0H → P11:1H → P12:1H → P13:2H → P14:0H → P15:0H → P16:0H.
Streak 0/3 (MED/LOW remain).

**Streak:** 0/3. Pass 17 pending adversary dispatch. Trajectory-tail: →8→14→13.
