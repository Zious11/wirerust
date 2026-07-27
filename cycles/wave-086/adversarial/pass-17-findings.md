---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-27T00:00:00Z
cycle: "wave-086"
pass: 17
verdict: NOT_CONVERGED
novelty: "med-high"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 17

**Date:** 2026-07-27
**Pass:** 17 of N
**Verdict:** NOT CONVERGED
**Novelty:** med-high — adversary explicitly flagged "partial-fix-regression axis is now the dominant
defect generator": 3 of 7 MEDs are self-inflicted by v2.5/v2.6 fixes (stale content-locator from P16-001
fix; Env-B greps introduced without count-pinning in v2.6; attribution destination omitted during
citation-currency update). Two genuine discipline-gap findings (needle-discipline not propagated to Task 7/ACR;
scope-prose sibling unswepped at :213-214). FIFTH consecutive zero-HIGH pass.
**Tally:** 13 findings — 0 CRIT / 0 HIGH / 7 MED / 5 LOW + 1 NIT (all fixed)
**Status:** REMEDIATED — D-534 state burst; STORY-182 v2.6→v2.7 + STORY-183 v2.6→v2.7
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Strategy escalation:** Orchestrator escalated strategy question at pass-17 completion; human re-confirmed
strategy (b) mechanical remediation ("keep grinding"); pipeline continues.
**Positives from adversary (verified-clean axes):** no-literal-phrase standing discipline (D-529) satisfied;
DF-SIBLING-SWEEP-001 compliance confirmed; pathspec subsumption direction consistent at all 7 loci (v2.6
holds); self-anchor elimination complete (zero :NNN intra-doc self-citations); canonical hashes 9a0f34c/9c9b12f
unchanged; Task-8 8a/8b split verified sound; set -euo pipefail in all main Task-8b blocks; ci.yml
order-dependence labels present; PASS/FAIL predicate-first form holds in all main verification blocks;
EXECUTION-REQUIRED flags (i)-(ix) consistent; scope-containment + accepted-residual disciplines (D-533) intact.

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P17-001 | MED | STORY-183 | FIXED v2.7 | Stale content-locator — Task-9 locator prose still searches for `set -o pipefail` (old form), but v2.6 F-W86S-P16-001 fix changed the block to `set -euo pipefail`; locator would not find the block post-fix. |
| F-W86S-P17-002 | MED | STORY-182 | FIXED v2.7 | Five unpinned Env-B greps — verification grep commands for Env-B evidence do not assert a specific count; any positive match (1/4, 2/4, ..., 4/4) passes; cannot discriminate 1-committed from 4-committed. |
| F-W86S-P17-003 | MED | STORY-182 | FIXED v2.7 | Attribution text had no destination locus — CC-BY-4.0 attribution obligation added in v2.6 (F-W86S-P16-004 fix) prescribes attribution text but does not specify the destination file path + line anchor where the implementer must place it. |
| F-W86S-P17-004 | MED | STORY-182 | FIXED v2.7 | Inert wave-gate BLOCKS clause — ci.yml coverage gate has a BLOCKS condition `if captured_count < 1` that is unreachable given the immediately preceding step that already gates on `captured_count >= 1`; the BLOCKS clause is independently impossible (N<1 impossible at that point in the pipeline). |
| F-W86S-P17-005 | MED | STORY-183 | FIXED v2.7 | Needle false-failure discipline not in Task 7/ACR — the no-literal-phrase standing discipline (D-529) requires that prescribed needle strings in test verification blocks must not quote literal TIER-1 flagged phrases; this discipline was never added to Task 7 or the ACR section of STORY-183; an implementer following Task 7 would not know about the constraint. |
| F-W86S-P17-006 | MED | STORY-183 | FIXED v2.7 | Unswept scope-prose sibling bin/check-green-doc-tense:213-214 — lines 213-214 of bin/check-green-doc-tense contain scope description prose ("This checker scans phrase-level patterns in…") that was not updated when the scope was narrowed in v2.6; the prose describes an older, broader scope. |
| F-W86S-P17-007 | MED | STORY-182 | FIXED v2.7 | Unbounded after-:905 placement admitting dead-code false-green — STORY-182 Task 6 prescribes that the mod iec104_e2e_real_pcaps block must be placed "after line :905" with no upper bound; an implementer could place it after the closing `#[cfg(test)]` brace of the test module, making the block dead code that never runs; a superficial grep passes while the test is unreachable. |
| F-W86S-P17-008 | LOW | STORY-183 | FIXED v2.7 | Task-10 derivation count stale — the v2.6 fix for F-W86S-P16-005 added an explicit exit-1 predicate to the BAD_CASE verification form, making AC-183-009 gate-check set 11 predicates, not 10; the Task-10 derivation note still reads "22 = 8+10+4" when the correct count is "23 = 8+11+4". |
| F-W86S-P17-009 | LOW | STORY-182 | FIXED v2.7 | Passive-voice residual in Notes §"Env-B greps" — one sentence in the Env-B greps rationale note uses passive voice ("the count is verified by") rather than the E-11 active-imperative form ("the verification command asserts"); violates active-voice convention established in P15/P16 NIT fixes. |
| F-W86S-P17-010 | LOW | STORY-183 | FIXED v2.7 | Pattern-33 tdd_mode comment uses wrong task cross-reference — comment body in the prescribed Pattern-33 code block says "see Task 8 for invocation" (ambiguous post-split); should reference "Task 8b" per the Task-8 split discipline (D-531); same issue class as NIT-03 from pass-16 but in a different code block. |
| F-W86S-P17-011 | LOW | STORY-182 | FIXED v2.7 | `--exact` flag missing from Task 4 precondition test invocation — Task 4 step 4b recommends `cargo test test_iec104_iti_diverse_e2e_expectations` without `--exact`; the `--exact` flag mandate was established in v1.6 (F-W86S-P6-004) and confirmed at 6 loci in v2.4; Task 4 step 4b was not updated in the scope of prior sweeps. |
| F-W86S-P17-012 | LOW | STORY-183 | FIXED v2.7 | Scope note at bin/check-green-doc-tense:213-214 (P17-006 companion) — the correction to the scope prose at :213-214 (P17-006) exposed a companion footnote two lines below (:216-217) that still describes the pre-v2.6 scope; the P17-006 fix updated the headline prose but not the footnote. |
| NIT-01 | NIT | STORY-182 | FIXED v2.7 | Step numbering gap in Task 1 — after v2.6 added "Step 1e" (CC-BY-4.0 citation row), the next step is labeled "Step 1g" in the story body, skipping "Step 1f"; the gap is purely cosmetic (no functional impact) but creates an inconsistent step sequence. |

---

## Findings Detail

### F-W86S-P17-001 (MED) — Stale content-locator in Task-9 (set -o vs -euo)

**Story:** STORY-183
**Status:** FIXED v2.7 — Task-9 locator prose updated to reference `set -euo pipefail`

**Description:** Pass-16 finding F-W86S-P16-001 fixed STORY-183 Task-9's hermetic-harness
confirmation block to use `set -euo pipefail` (replacing `set -o pipefail`). STORY-183 also contains
content-based locator prose that guides an implementer (or the adversary's sweep) to the correct code
block by searching for a characteristic string. That locator prose still reads:

> "Locate the hermetic-harness block beginning with `set -o pipefail`."

After the v2.6 fix, no block in STORY-183 begins with `set -o pipefail`. The locator has become stale
and would fail to find the target block. An implementer following the locator would be directed to a
non-existent block, causing confusion or locating a wrong block.

This is a partial-fix regression: F-W86S-P16-001 fixed the shell block itself but did not update the
prose locator that references the old form.

**Fix (v2.7):** Task-9 locator prose updated to reference `set -euo pipefail` as the characteristic
search string.

---

### F-W86S-P17-002 (MED) — Five unpinned Env-B greps (1/4 vs 4/4 indiscriminate)

**Story:** STORY-182
**Status:** FIXED v2.7 — Env-B grep commands pinned to explicit count with test-result-ok check

**Description:** STORY-182 v2.6 added five verification grep commands for Env-B (real-pcap environment)
evidence. These greps use a form such as:

```bash
grep -l "iec104-iti-diverse" tests/fixtures/ | grep -q "."
```

This form passes on any positive match — whether 1, 2, 3, or 4 committed captures are present. The
adversary ran the verification greps against a hypothetical state where only 1 of 4 expected captures
was present and the check still passed. The intent is to confirm 4/4 committed captures (or at minimum
a documented threshold such as 1/4 with an explicit "test-result-ok" guard), but the unpinned form
cannot discriminate partial from full coverage.

This finding is a partial-fix regression: the grep commands were added in v2.6 to satisfy Env-B
verification obligations but without count anchoring.

**Fix (v2.7):** Env-B grep commands pinned to explicit count: `grep -c "pattern" file | grep -q "^4$"`
(or "^1$" with a test-result-ok guard where 1-commit is the documented minimum). Count-pinning
discipline added to the story's Env-B verification rationale note.

---

### F-W86S-P17-003 (MED) — Attribution text has no destination locus

**Story:** STORY-182
**Status:** FIXED v2.7 — Attribution obligation includes file path + line anchor as destination locus

**Description:** STORY-182 v2.6 (F-W86S-P16-004 fix) updated the ci.yml grep anchor to the canonical
"ITI CC-BY-4.0" prefix string. As part of this fix, the story's attribution obligation prose was also
updated to require that an "ITI CC-BY-4.0" attribution line appear in the README provenance table.
However, the obligation prose added in v2.6 does not specify:

- Which file contains the provenance table (README.md vs docs/pcap-sources.md)
- Which section/line range the attribution must appear in

An implementer reading the obligation prose knows WHAT attribution text to add but not WHERE to add it.
Without a destination locus (file path + line anchor), the implementer might add the attribution to a
comment-only block that is compiled out, or to the wrong file, while the AC verification grep still
passes (since it searches the whole tree).

**Fix (v2.7):** Attribution obligation updated with explicit destination locus: "Add attribution row to
`README.md` lines 41-44 provenance table under the IEC-104 capture section."

---

### F-W86S-P17-004 (MED) — Inert wave-gate BLOCKS clause (N<1 impossible independently)

**Story:** STORY-182
**Status:** FIXED v2.7 — Inert BLOCKS clause reworded to evidence-artifact obligation

**Description:** STORY-182 Task 5 prescribes a ci.yml coverage gate with the following BLOCKS condition:

> "BLOCKS if `captured_count < 1`"

However, earlier in the same task the gate logic includes:

> "Precondition: `captured_count >= 1` (enforced by preceding step FETCH-PCAPS-GATE)"

Given the FETCH-PCAPS-GATE precondition, `captured_count < 1` is impossible at the point where the
BLOCKS clause is evaluated — the preceding gate has already ensured at least 1 captured file exists.
The BLOCKS clause is independently unreachable dead logic.

The adversary notes that this class of inert-precondition gate is a recurring issue in E-11 stories
(wave-86 partial-fix-regression axis): a BLOCKS clause is copied from an earlier template and not
updated to reflect the current gate topology, creating the appearance of safety without the substance.

**Fix (v2.7):** Inert BLOCKS clause reworded to an evidence-artifact obligation: "The ci.yml step
must produce a `fixture-count-gate-entry.md` artifact confirming `captured_count >= N` for the
documented N. BLOCKS if artifact is absent or records `captured_count < N`." This form gates on
a produced artifact rather than a redundant precondition.

---

### F-W86S-P17-005 (MED) — Needle false-failure discipline not in Task 7/ACR

**Story:** STORY-183
**Status:** FIXED v2.7 — No-literal-phrase discipline added to Task 7 and ACR section

**Description:** The no-literal-phrase standing discipline (D-529) requires that prescribed needle
strings in test verification blocks — specifically in `assert!` macro arguments, fixture annotation
text, and verification comments — must not quote literal TIER-1 flagged phrases from
DF-GREEN-DOC-TENSE-SWEEP. The rationale: if a story file containing those literal phrases is scanned
by the deployed checker, the annotations become self-referential false-FAIL sites (the checker flags
its own prescription).

The discipline was imposed after the 5th self-referential-predicate recurrence at D-529 and
documented in the Session Resume Checkpoint and STATE.md disciplines log. However, it was never
propagated to Task 7 (the task that prescribes verification block content) or to the ACR section
of STORY-183. An implementer reading Task 7 in isolation would not know about the constraint.

**Fix (v2.7):** No-literal-phrase discipline added to Task 7 header note: "Prescribed needle strings
and AC assertion text MUST NOT quote literal TIER-1 flagged phrases (see D-529 standing discipline).
Use paraphrased forms or concatenated string literals to avoid self-referential false-FAIL." Mirrored
in ACR section under "Standing Disciplines."

---

### F-W86S-P17-006 (MED) — Unswept scope-prose sibling bin/check-green-doc-tense:213-214

**Story:** STORY-183
**Status:** FIXED v2.7 — Scope prose at :213-214 updated to reflect current post-v2.6 scope

**Description:** bin/check-green-doc-tense lines 213-214 contain the checker's self-description
of its scan scope:

```python
# This checker scans phrase-level patterns in .rs, .md, .py, and .toml files
# across the full src/, tests/, bin/, docs/, and cycles/ directory tree.
```

STORY-183 v2.6 narrowed the scope of the pattern-set extension to `bin/*.py` files only. The scope
prose at :213-214 still describes the original broad scope. The DF-SIBLING-SWEEP-001 sweep in v2.6
covered the story's own scope sections but did not extend to the checker binary's self-description
at these specific lines.

This is a partial-fix regression from v2.6's scope-containment work: the scope was narrowed in the
story's prescriptions but the checker's own scope-documentation prose was not updated correspondingly.

**Fix (v2.7):** STORY-183 Task sweep task updated to include bin/check-green-doc-tense:213-214 in
the scope-prose sweep. Scope prose at :213-214 updated to describe the current `bin/*.py`-scoped
pattern-set extension.

---

### F-W86S-P17-007 (MED) — Unbounded after-:905 placement admitting dead-code false-green

**Story:** STORY-182
**Status:** FIXED v2.7 — Placement obligation bounded above with explicit upper bound

**Description:** STORY-182 Task 6 prescribes that the `mod iec104_e2e_real_pcaps` block must be
placed "after line :905 in tests/iec104_analyzer_tests.rs." The lower bound `:905` prevents
placement before the existing test content, but there is no upper bound. Specifically, there is no
constraint preventing placement AFTER the closing `}` of the existing `#[cfg(test)]` block.

If the `mod iec104_e2e_real_pcaps` block is placed outside the `#[cfg(test)]` block:
- `cargo test` compiles and runs the outer file successfully
- The mod block is conditionally compiled only when `cfg(test)` is active — but it is already
  outside the block, so it compiles unconditionally as a module, not as a test
- `cargo test --all-targets` may report the tests as "found" but they are actually unreachable
  from the test runner

Result: a false-green `cargo test` run that reports success while the prescribed tests never execute.

**Fix (v2.7):** Placement obligation bounded above: "Place mod iec104_e2e_real_pcaps after line :905
AND before the closing `}` of the `#[cfg(test)]` block (confirmed range: :905-:1,050)."

---

### F-W86S-P17-008 (LOW) — Task-10 derivation count stale after P16-005 BAD_CASE tightening

**Story:** STORY-183
**Status:** FIXED v2.7 — Derivation note updated to 23 passes (8+11+4)

**Description:** Pass-16 finding F-W86S-P16-005 fixed the BAD_CASE verification form to use an
explicit exit-1 predicate (`[ "$EXIT" -eq 1 ]`). Pass-16 finding F-W86S-P16-007 had already
updated the Task-10 derivation from 21 (8+9+4) to 22 (8+10+4), adding the exit-2+ error guard
as the 10th predicate. The F-W86S-P16-005 fix added one more explicit predicate (exit-1 assert),
making AC-183-009 gate-check set 11 predicates. The Task-10 derivation still reads "22 = 8+10+4".

**Fix (v2.7):** Derivation note updated: "23 = 8 TIER-1 pattern self-tests (Patterns 30-37 via
tdd_mode, Task-8b) + 11 AC-183-009 gate-check predicates (9 case-outcome predicates + 1 exit-2+
error guard (v2.5) + 1 explicit exit-1 BAD_CASE assert (v2.6)) + 4 AC-183-007 scanner-output-format
assertions."

---

### F-W86S-P17-009 (LOW) — Passive-voice residual in Notes §"Env-B greps"

**Story:** STORY-182
**Status:** FIXED v2.7 — Passive-voice phrase reworded to active form

**Description:** The v2.6 Notes §"Env-B greps" rationale sentence reads: "the count is verified by
the grep command to ensure at least one committed capture is present." The E-11 active-imperative
present-tense convention (established in NIT fixes at P15/P16) requires active voice: the subject
(the grep command, or the task) performs the action. The passive form also obscures what executes
the verification.

**Fix (v2.7):** Reworded: "The grep command asserts that at least N committed captures are present
(expected count: N/4 per Env-B baseline)."

---

### F-W86S-P17-010 (LOW) — Pattern-33 tdd_mode comment uses ambiguous Task 8 cross-reference

**Story:** STORY-183
**Status:** FIXED v2.7 — Cross-reference updated to "Task 8b"

**Description:** The prescribed Pattern-33 code block in STORY-183 Task-8a includes an explanatory
comment: "see Task 8 for invocation details." After the Task-8 split (Task 8a/8b per F-W86S-P14-001,
D-531), "Task 8" is ambiguous — it does not specify which sub-task. The invocation details are in
Task 8b (verification), not Task 8a (implementation). This is the same ambiguity class as NIT-03
from pass-16 (which fixed a different cross-reference to "Task 8" → "Task 8b"), but this instance
in a different code comment block was not caught by the P16 NIT-03 sweep.

**Fix (v2.7):** Cross-reference updated to "see Task 8b — RED gate verification."

---

### F-W86S-P17-011 (LOW) — `--exact` flag missing from Task 4 step 4b test invocation

**Story:** STORY-182
**Status:** FIXED v2.7 — `--exact` flag added to Task 4 step 4b

**Description:** The `--exact` flag mandate for `cargo test` invocations was established in
STORY-182 v1.6 (F-W86S-P6-004) and confirmed at six loci in v2.4 (F-W86S-P14-002). Task 4 step 4b
reads:

```
4b. Run: cargo test test_iec104_iti_diverse_e2e_expectations
```

The `--exact` flag is absent. Without `--exact`, the test name is matched by substring and could
match additional tests if any future test name contains `test_iec104_iti_diverse_e2e_expectations`
as a substring. This is the last locus that escaped the v2.4 six-loci sweep.

**Fix (v2.7):** Task 4 step 4b updated to `cargo test test_iec104_iti_diverse_e2e_expectations --exact`.

---

### F-W86S-P17-012 (LOW) — Scope-footnote companion at :216-217 not updated with P17-006 fix

**Story:** STORY-183
**Status:** FIXED v2.7 — Companion footnote at :216-217 updated to match post-v2.6 scope

**Description:** The P17-006 fix updated the headline scope prose at bin/check-green-doc-tense
:213-214. Two lines below (:216-217), there is a footnote comment that elaborates on the scope
description:

```python
# Note: .toml and .py scanning added in v2.6 (see STORY-183 v2.3).
```

After v2.6 narrowed the .py scope to `bin/*.py` only, this note is stale — it still implies
blanket `.py` scanning was added. The P17-006 fix reached :213-214 but not :216-217.

**Fix (v2.7):** Companion footnote updated: "Note: Pattern-set extension to `bin/*.py` (STORY-183
v2.3+; phrase-level scope; full-tree .py scanning deferred, see DRIFT-py-surface-outside-bin)."

---

## NIT-Observations (1 item — actioned)

1. **NIT-01 (ACTIONED, STORY-182):** Step numbering gap in Task 1 — after v2.6 inserted "Step 1e"
   (CC-BY-4.0 citation row), the following step was labeled "Step 1g" in the story body, skipping
   "Step 1f". Renumbered to "Step 1f" to restore sequential step labels.

---

## EXECUTION-REQUIRED Flags (9 items — carried from pass-16 unchanged)

Items (i)-(ix) from pass-16-findings.md carried forward unchanged. No new EXECUTION-REQUIRED
flags added from pass-17.

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

## Verified-Clean Table (pass-17 adversary confirmation)

| Verification Item | Result |
|-------------------|--------|
| All finding counts / pass tallies | EXACT |
| Input-hash values 9a0f34c / 9c9b12f | EXACT — canonical Python tool; unchanged |
| No-literal-phrase sweep (D-529 standing discipline) | CONFIRMED — no TIER-1 literals in annotation text |
| Intra-document :NNN self-citations | CONFIRMED ZERO |
| Pathspec subsumption direction (7-loci agreement) | CONFIRMED — src/*.rs strictly subsumes src/**/*.rs at all 7 loci in v2.6 |
| DF-GREEN-DOC-TENSE-SWEEP v6 compliance | CONFIRMED — no TIER-1 regressions from v2.6 edits |
| DF-SIBLING-SWEEP-001 | CONFIRMED — sweep performed; no sibling regressions |
| if:always() loci | CONFIRMED ZERO |
| Task-8 8a/8b split ordering | CONFIRMED — v2.6 split intact |
| PASS/FAIL-convention predicate-first form (main blocks) | CONFIRMED — all main blocks predicate-first in v2.6 |
| ci.yml order-dependence labels | CONFIRMED — present |
| Ground-truth axis | CONFIRMED CLEAN — zero findings |
| Scope-containment + accepted-residual disciplines (D-533) | CONFIRMED — no regression |

---

## Pass-17 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: med-high — adversary explicitly flagged partial-fix-regression axis as the dominant
defect generator for this wave; 3 of 7 MEDs are second-order regressions from v2.5/v2.6 fixes
(P17-001 from P16-001; P17-002 new verification code without count-pinning; P17-003 attribution
obligation without locus). Two genuine discipline-gap findings (P17-005 needle-discipline missing
from Task 7/ACR; P17-006 scope-prose sibling not swept). Two structural safety findings (P17-004
inert BLOCKS clause; P17-007 unbounded placement). 5 LOWs + 1 NIT all minor.
No HIGH findings. FIFTH consecutive zero-HIGH pass (P10, P14, P15, P16, P17).

Strategy escalation: orchestrator brought strategy question to human. Human re-confirmed strategy
(b) mechanical remediation ("keep grinding"). Pipeline continues at pass-18.

Severity profile: P15 0C/0H/5M/6L → P16 0C/0H/6M/3L → P17 0C/0H/7M/5L.
MED count +1 pass-over-pass; adversary attributes this to the partial-fix-regression axis
generating fresh MEDs at the same rate as prior MEDs are closed.

HIGH count history: P10:0H → P11:1H → P12:1H → P13:2H → P14:0H → P15:0H → P16:0H → P17:0H.
Fifth consecutive zero-HIGH confirms HIGH-severity axis remains clean. Streak clock not
advanced due to remaining MED/LOW findings.

Pass tallies (P1–P17): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14 / 10 / 15 / 8 / 14 / 13 / 13.
Total across all passes: 276 findings. Canonical hashes: 9a0f34c / 9c9b12f.

Pass-18 next after D-534 remediation burst.

---

## Remediation (D-534)

**Date:** 2026-07-27
**Burst:** D-534 STATE BURST — WAVE-86 ADVERSARIAL PASS 17 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 13 findings FIXED at STORY-182 v2.7 / STORY-183 v2.7.

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P17-001 (MED) | FIXED v2.7 | STORY-183 | Task-9 content-locator updated: `set -o pipefail` → `set -euo pipefail` |
| F-W86S-P17-002 (MED) | FIXED v2.7 | STORY-182 | Env-B grep commands pinned to explicit count with test-result-ok check |
| F-W86S-P17-003 (MED) | FIXED v2.7 | STORY-182 | Attribution obligation includes destination locus (README.md lines 41-44) |
| F-W86S-P17-004 (MED) | FIXED v2.7 | STORY-182 | Inert BLOCKS clause reworded to evidence-artifact obligation |
| F-W86S-P17-005 (MED) | FIXED v2.7 | STORY-183 | No-literal-phrase discipline added to Task 7 header + ACR section |
| F-W86S-P17-006 (MED) | FIXED v2.7 | STORY-183 | Scope prose at bin/check-green-doc-tense:213-214 updated; sweep task extended |
| F-W86S-P17-007 (MED) | FIXED v2.7 | STORY-182 | Placement obligation bounded above (`:905-:1,050`) |
| F-W86S-P17-008 (LOW) | FIXED v2.7 | STORY-183 | Task-10 derivation updated to 23 passes (8+11+4) |
| F-W86S-P17-009 (LOW) | FIXED v2.7 | STORY-182 | Passive-voice phrase reworded in Notes §"Env-B greps" |
| F-W86S-P17-010 (LOW) | FIXED v2.7 | STORY-183 | Pattern-33 cross-reference updated to "Task 8b" |
| F-W86S-P17-011 (LOW) | FIXED v2.7 | STORY-182 | `--exact` flag added to Task 4 step 4b |
| F-W86S-P17-012 (LOW) | FIXED v2.7 | STORY-183 | Companion footnote at :216-217 updated |
| NIT-01 | FIXED v2.7 | STORY-182 | Task 1 step numbering gap closed (1e/1f sequential) |

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**Fifth zero-HIGH pass confirmed:** P10:0H → P11:1H → P12:1H → P13:2H → P14:0H → P15:0H → P16:0H → P17:0H.
Streak 0/3 (MED/LOW remain).

**Streak:** 0/3. Pass 18 pending adversary dispatch. Trajectory-tail: →14→13→13.
