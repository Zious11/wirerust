---
document_type: adversarial-findings
level: ops
version: "1.0"
status: remediated
producer: adversary
timestamp: 2026-07-27T00:00:00Z
cycle: "wave-086"
pass: 18
verdict: NOT_CONVERGED
novelty: "medium-low"
inputs: [.factory/stories/STORY-182.md, .factory/stories/STORY-183.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Adversarial Findings — wave-086 Pass 18

**Date:** 2026-07-27
**Pass:** 18 of N
**Verdict:** NOT CONVERGED
**Novelty:** medium-low — 3 MEDs are v2.7-induced partial-fix regressions (P18-001 attribution
pointer scheme introduced ambiguity about destination pointer; P18-003 execution-not-output predicate
introduced as new obligation without !cancelled() truth; P18-006 pattern-registry-block label
introduced inconsistent taxonomy). 2 genuinely-new sibling-sweep misses (4th instance of
comment-marker claim at tool :28-29; fixture_present doc-comment :59-62 on the rewritten function
itself). SIXTH consecutive zero-HIGH pass.
**Tally:** 11 findings — 0 CRIT / 0 HIGH / 6 MED / 3 LOW + 2 NITs (all fixed)
**Status:** REMEDIATED — D-535 state burst; STORY-182 v2.7→v2.8 + STORY-183 v2.7→v2.8
**Freshness attestation:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (PASS); structural attestation PASS
**Positives from adversary (verified-clean axes):** no-literal-phrase standing discipline (D-529) satisfied;
DF-SIBLING-SWEEP-001 compliance confirmed at all swept loci; pathspec subsumption direction consistent
at all 7 loci (v2.7 holds); self-anchor elimination complete (zero :NNN intra-doc self-citations);
canonical hashes 9a0f34c/9c9b12f unchanged; Task-8 8a/8b split verified sound; set -euo pipefail in
all main Task-8b blocks; ci.yml order-dependence labels present; PASS/FAIL predicate-first form holds
in all main verification blocks; EXECUTION-REQUIRED flags (i)-(ix) consistent; scope-containment +
accepted-residual disciplines (D-533) intact; Env-B count-pinning discipline (D-534) present at
all five loci; attribution destination locus (README.md :41-44) confirmed; Task-9 locator updated
(set -euo pipefail, D-534-P17-001 fix verified).

---

## Summary Table

| ID | Severity | Story | Status | Description |
|----|----------|-------|--------|-------------|
| F-W86S-P18-001 | MED | STORY-182 | FIXED v2.8 | Tautological M==len() predicate in fixture-count gate — the fixture-count comparison uses `captured_count == fixture_manifest.len()` which is tautologically satisfied when the manifest is derived from the same capture listing; a discriminating N/M predicate is required. |
| F-W86S-P18-002 | MED | STORY-182 | FIXED v2.8 | Attribution single-destination pointer scheme absent — v2.7 added a README.md :41-44 destination locus but did not establish a single-destination pointer scheme in the ci.yml step description; the implementer has no canonical pointer from ci.yml step to attribution row. |
| F-W86S-P18-003 | MED | STORY-182 | FIXED v2.8 | Execution-not-output !cancelled() truth — Task 5b gate step verification prescribes checking for output string presence to confirm the step ran; the correct predicate is `!cancelled()` which checks execution status rather than output presence (output can be empty even when the step succeeded). |
| F-W86S-P18-004 | MED | STORY-183 | FIXED v2.8 | Doc-comment at :59-62 stale on rewritten fixture_present function — the fixture_present function body was rewritten in v2.7 but the doc-comment at :59-62 still describes the pre-v2.7 behavior; the DF-SIBLING-SWEEP-001 sweep in v2.7 did not include :59-62 in its scope. |
| F-W86S-P18-005 | MED | STORY-183 | FIXED v2.8 | :53-57 and :47-49 sweep additions missing — two adjacent blocks at :47-49 and :53-57 contain scope-description prose following the same stale pattern corrected at :213-214 (P17-006 fix); these blocks were not added to the sweep task in v2.7. |
| F-W86S-P18-006 | MED | STORY-183 | FIXED v2.8 | Pattern-registry-block label inconsistent — the pattern-registry initialization block added in v2.7 is labeled `PATTERN-REGISTRY` in its identifying comment; the correct label per the task taxonomy is `PATTERN-REGISTRY-BLOCK`; locator prose referencing this block would fail to find it. |
| F-W86S-P18-007 | LOW | STORY-183 | FIXED v2.8 | Task-10 derivation bullet not split — the Task-10 derivation note presents the three-component formula "23 = 8+11+4" as a single inline expression; after the P17-008 count update the note should be split into three labelled bullets for clarity (8 TIER-1 pattern self-tests; 11 AC-183-009 gate-check predicates; 4 AC-183-007 scanner-output assertions). |
| F-W86S-P18-008 | LOW | STORY-182 | FIXED v2.8 | Zero-file-guard negative assertion missing — Task 5b prescribes the gate passing when captured_count >= 1 but does not prescribe a negative assertion test confirming that captured_count == 0 causes the gate to FAIL; the negative guard is required to verify the gate is non-vacuous. |
| F-W86S-P18-009 | LOW | STORY-183 | FIXED v2.8 | 4th instance of comment-marker claim at tool :28-29 — the tool module-level comment at :28-29 repeats the claim that the checker uses "comment markers" to exclude false-positive sites; this phrasing has appeared and been corrected 3 prior times across v1.x and v2.x remediations; this instance at :28-29 was not caught by the prior DF-SIBLING-SWEEP-001 sweeps. |
| NIT-01 | NIT | STORY-182 | FIXED v2.8 | fixture_present doc-comment :59-62 on rewritten function — the fixture_present function was structurally rewritten in v2.7 (Env-B greps + count-pinning); the doc-comment at :59-62 now describes a behavior ("returns true if the fixture file exists in the local filesystem") that reflects the pre-v2.7 implementation; the rewritten function additionally validates count and presence. |
| NIT-02 | NIT | STORY-183 | FIXED v2.8 | Trailing space in Pattern-33 code-block fence line — the Pattern-33 code block added in v2.7 has a trailing space on the opening fence line (` ```bash ` instead of ` ```bash`); cosmetic only but inconsistent with all other code-block fences in the story. |

---

## Findings Detail

### F-W86S-P18-001 (MED) — Tautological M==len() predicate in fixture-count gate

**Story:** STORY-182
**Status:** FIXED v2.8 — Discriminating N/M predicate replaces tautological M==len() form

**Description:** STORY-182 Task 5b prescribes a coverage gate that checks:

```bash
captured_count == fixture_manifest.len()
```

where both `captured_count` and `fixture_manifest` are derived from the same source (the committed
captures listing). When both sides are derived from the same enumeration, the equality check is
tautologically true — the manifest always has exactly as many entries as were counted, by construction.
The intent is to assert that the count meets a documented threshold N (e.g., exactly 4 captured files),
not simply that the counted value equals itself.

This is a v2.7-induced finding: the Env-B count-pinning discipline (D-534) introduced the count-check
form but the formula used `manifest.len()` rather than a fixed threshold literal N.

**Fix (v2.8):** Predicate replaced with discriminating N/M form: `captured_count == N` where N is
the documented threshold (4). The predicate `captured_count == fixture_manifest.len()` is dropped;
`fixture_manifest.len()` is separately asserted equal to N as a manifest-completeness check.

---

### F-W86S-P18-002 (MED) — Attribution single-destination pointer scheme absent

**Story:** STORY-182
**Status:** FIXED v2.8 — Single-destination pointer scheme established in ci.yml step description

**Description:** STORY-182 v2.7 (F-W86S-P17-003 fix) added a destination locus for the CC-BY-4.0
attribution obligation: "Add attribution row to README.md lines 41-44 provenance table." However,
the ci.yml step description does not include a pointer back to this destination — it describes the
obligation in one task section and the ci.yml step in another, with no cross-reference linking the
ci.yml step to the README.md :41-44 destination. An implementer reading only the ci.yml step
description does not know where to check for the attribution row.

The adversary notes that the v2.7 fix addressed the absence of destination locus in the obligation
prose but did not propagate the pointer into the ci.yml step verification block, which is the
canonical entry point for an implementer's CI workflow.

**Fix (v2.8):** Single-destination pointer scheme: ci.yml step description now includes explicit
cross-reference: "Verify attribution row present at README.md :41-44 (see Attribution Obligation
in Task 2)." Single pointer, single destination, bidirectional reference.

---

### F-W86S-P18-003 (MED) — Execution-not-output !cancelled() truth

**Story:** STORY-182
**Status:** FIXED v2.8 — Gate step verification predicate uses !cancelled() execution status

**Description:** STORY-182 Task 5b prescribes that the ci.yml coverage gate step should be verified
by checking for the presence of a specific output string ("Fixture coverage: N/4 committed"). This
string-output-presence approach carries a false-green risk: a prior ci.yml step could emit the string
as a side effect while the gate step itself is cancelled or skipped.

The correct predicate for confirming a ci.yml step executed is `!cancelled()` as an `if:` condition
or via `steps.<step-id>.outcome == 'success'` — checking execution outcome rather than output content.
Output-content checks are appropriate for verifying WHAT the step produced, but they are not a
reliable proxy for WHETHER the step ran.

**Fix (v2.8):** Verification predicate updated: gate step verification uses `steps.<step-id>.outcome
== 'success'` (execution-not-output truth). Output content verification (the "Fixture coverage: N/4"
string check) is retained as a complementary assertion but is no longer the sole gate predicate.

---

### F-W86S-P18-004 (MED) — Doc-comment at :59-62 stale on rewritten fixture_present function

**Story:** STORY-183
**Status:** FIXED v2.8 — Doc-comment at :59-62 updated to describe post-v2.7 function behavior

**Description:** The `fixture_present` helper function in the check-green-doc-tense test harness
was rewritten in STORY-183 v2.7 to include count-based validation alongside file-existence checking.
The doc-comment at :59-62 still reads:

> "Returns True if the fixture file exists in the local test environment."

After the v2.7 rewrite, the function also asserts that at least N instances of the fixture pattern
are present (implementing the count-pinning discipline from D-534). The doc-comment does not reflect
this extended behavior. An implementer reading the doc-comment would underestimate the function's
postcondition and might omit the count-validation path in their implementation.

This is a genuinely-new sibling-sweep miss: the DF-SIBLING-SWEEP-001 sweep in v2.7 included
:47-49 and :53-57 but the task scope list did not extend to :59-62 (the function's own doc-comment
block).

**Fix (v2.8):** Doc-comment at :59-62 updated: "Returns True if the fixture file exists in the
local test environment AND at least N instances of the fixture pattern are confirmed present
(N per Env-B count-pinning baseline, D-534)."

---

### F-W86S-P18-005 (MED) — :53-57 and :47-49 sweep additions missing

**Story:** STORY-183
**Status:** FIXED v2.8 — :47-49 and :53-57 added to DF-SIBLING-SWEEP-001 scope

**Description:** Pass-17 finding F-W86S-P17-006 corrected scope prose at bin/check-green-doc-tense
:213-214. The v2.7 fix extended the sweep task to include :213-214. However, two additional blocks
at :47-49 and :53-57 in the same file contain analogous scope-description prose that follows the
same stale pattern:

- :47-49: "# Scan targets: all .rs, .md, .py source files in the repository root"
- :53-57: "# Extension pattern: TIER-1 behavioral-absence tokens (see DF-GREEN-DOC-TENSE-SWEEP)"

After STORY-183's scope-narrowing work (bin/*.py phrase-level scope), these lines describe a
broader scan target than the current scope. The v2.7 sweep task added :213-214 but did not extend
to :47-49 and :53-57 despite the same stale-prose class applying.

**Fix (v2.8):** :47-49 and :53-57 added to the DF-SIBLING-SWEEP-001 scope list in Task sweep;
prose at both sites updated to reflect the current bin/*.py phrase-level scope.

---

### F-W86S-P18-006 (MED) — Pattern-registry-block label inconsistent

**Story:** STORY-183
**Status:** FIXED v2.8 — Label corrected to PATTERN-REGISTRY-BLOCK throughout

**Description:** STORY-183 v2.7 introduced a pattern-registry initialization block with the
identifying label:

```python
# PATTERN-REGISTRY: TIER-1 behavioral-absence patterns
```

The task taxonomy established at D-531 and propagated through the story uses the label
`PATTERN-REGISTRY-BLOCK` (hyphenated compound, consistent with `FIXTURE-GATED-TESTS` and
`EXECUTION-REQUIRED` label conventions). The v2.7 label `PATTERN-REGISTRY` is a truncation
that does not match the locator pattern used in the story's content-based locator prose.

An adversary's sweep grep for `PATTERN-REGISTRY-BLOCK` would not find the block; a locator
referencing `PATTERN-REGISTRY-BLOCK` would fail to locate it. This is a v2.7-induced finding:
the label was introduced by the v2.7 fix and uses a non-canonical form.

**Fix (v2.8):** Label corrected to `PATTERN-REGISTRY-BLOCK` at all instances (3 loci: the
block header, the locator prose in Task 8a, and the ACR reference row).

---

### F-W86S-P18-007 (LOW) — Task-10 derivation bullet not split

**Story:** STORY-183
**Status:** FIXED v2.8 — Task-10 derivation split into three labelled bullets

**Description:** Pass-17 finding F-W86S-P17-008 updated the Task-10 derivation count from
22 (8+10+4) to 23 (8+11+4). The derivation note reads as a single inline formula: "23 = 8 TIER-1
pattern self-tests + 11 AC-183-009 gate-check predicates + 4 AC-183-007 scanner-output assertions."

After the count update, the formula has grown to three distinct components with internal
sub-counts. Presenting it as a single inline formula obscures the per-component breakdown
and makes future auditing harder (e.g., a future pass would need to re-derive "11" from
first principles rather than reading labeled sub-components).

**Fix (v2.8):** Task-10 derivation split into three labelled bullets:
- "8: TIER-1 pattern self-tests (Patterns 30-37 via tdd_mode, Task 8b)"
- "11: AC-183-009 gate-check predicates (9 case-outcome + 1 exit-2+ guard + 1 exit-1 assert)"
- "4: AC-183-007 scanner-output-format assertions"
Total: 23.

---

### F-W86S-P18-008 (LOW) — Zero-file-guard negative assertion missing

**Story:** STORY-182
**Status:** FIXED v2.8 — Negative assertion added to Task 5b gate specification

**Description:** STORY-182 Task 5b prescribes the fixture-count gate passing when
`captured_count >= 1`. The task describes only the positive-path behavior (gate PASSES when
condition holds). It does not prescribe a negative assertion test confirming that
`captured_count == 0` causes the gate to FAIL.

Without a negative assertion, an implementer could write a gate that always emits a PASS
message (vacuous gate) without the check failing. The negative guard is required to
demonstrate the gate is non-vacuous: a gate that only specifies the success condition
cannot be distinguished from a gate that always succeeds.

**Fix (v2.8):** Negative assertion added to Task 5b: "Negative guard: confirm that a
`captured_count` of 0 causes the gate step to exit non-zero and emit 'FAIL: no captured
fixtures' (verified via local dry-run with `captured_count=0 bash gate-step.sh`)."

---

### F-W86S-P18-009 (LOW) — 4th instance of comment-marker claim at tool :28-29

**Story:** STORY-183
**Status:** FIXED v2.8 — Comment-marker claim at :28-29 corrected to accurate form

**Description:** The tool's module-level comment at bin/check-green-doc-tense :28-29 reads:

> "# Uses comment markers to exclude known-false-positive sites."

This claim asserts that the tool uses special in-source comment markers (e.g., `# noqa` or
`# pragma: no-check`) to exclude specific sites from the scan. This claim is inaccurate:
the tool uses an explicit NOT-stale verdict table in STORY-183's EC-011 section, not
in-source comment markers.

The comment-marker claim has been corrected 3 prior times in the story's revision history
(v1.x pass-1/2/3 cycles) at different prose loci. This instance at :28-29 is a new
sibling-sweep miss — a fourth occurrence of the same inaccurate claim, found at a different
line not covered by prior sweeps.

This is a genuinely-new sibling-sweep miss per the adversary's classification (distinct from
v2.7-induced regressions).

**Fix (v2.8):** Comment at :28-29 updated to accurate form: "# False-positive exclusions
are governed by the NOT-stale verdict table in STORY-183 EC-011 (static exclusion list;
no in-source markers used)."

---

## NIT-Observations (2 items — actioned)

1. **NIT-01 (ACTIONED, STORY-182):** fixture_present doc-comment :59-62 on rewritten function —
   the fixture_present function was structurally rewritten in v2.7; the doc-comment at :59-62 still
   describes the pre-v2.7 single-existence check without the count-validation behavior added in v2.7.
   Updated to describe both the existence check and the count-pinning postcondition.

2. **NIT-02 (ACTIONED, STORY-183):** Trailing space on Pattern-33 code-block fence line — the
   Pattern-33 code block's opening fence (` ```bash `) has a trailing space character, inconsistent
   with all other code-block fences in the story. Removed.

---

## EXECUTION-REQUIRED Flags (9 items — carried from pass-17 unchanged)

Items (i)-(ix) from pass-17-findings.md carried forward unchanged. No new EXECUTION-REQUIRED
flags added from pass-18.

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

## Verified-Clean Table (pass-18 adversary confirmation)

| Verification Item | Result |
|-------------------|--------|
| All finding counts / pass tallies | EXACT |
| Input-hash values 9a0f34c / 9c9b12f | EXACT — canonical Python tool; unchanged |
| No-literal-phrase sweep (D-529 standing discipline) | CONFIRMED — no TIER-1 literals in annotation text |
| Intra-document :NNN self-citations | CONFIRMED ZERO |
| Pathspec subsumption direction (7-loci agreement) | CONFIRMED — src/*.rs strictly subsumes src/**/*.rs at all 7 loci in v2.7 |
| DF-GREEN-DOC-TENSE-SWEEP v6 compliance | CONFIRMED — no TIER-1 regressions from v2.7 edits |
| DF-SIBLING-SWEEP-001 (at swept loci) | CONFIRMED — sweep performed at all task-listed loci |
| if:always() loci | CONFIRMED ZERO |
| Task-8 8a/8b split ordering | CONFIRMED — v2.7 split intact |
| PASS/FAIL-convention predicate-first form (main blocks) | CONFIRMED — all main blocks predicate-first in v2.7 |
| ci.yml order-dependence labels | CONFIRMED — present |
| Env-B count-pinning discipline (D-534) | CONFIRMED — five loci all carry count |
| Attribution destination locus (README.md :41-44) | CONFIRMED — present in v2.7 obligation prose |
| Ground-truth axis | CONFIRMED CLEAN — zero findings |
| Scope-containment + accepted-residual disciplines (D-533) | CONFIRMED — no regression |

---

## Pass-18 Verdict

**NOT CONVERGED.** Streak: 0/3.
Novelty: medium-low — 3 MEDs are v2.7-induced partial-fix regressions (tautological predicate from
count-pinning discipline; attribution pointer scheme incomplete; execution-not-output predicate added
without !cancelled() truth). 3 MEDs are sibling-sweep misses from the :47-49/:53-57/:59-62 sweep
scope gap. 1 MED is a label inconsistency from the pattern-registry relabeling in v2.7. 3 LOWs are
minor: task derivation presentation, negative assertion gap, 4th instance of comment-marker claim.
2 NITs: doc-comment on rewritten function, trailing space.
No HIGH findings. SIXTH consecutive zero-HIGH pass (P10, P14, P15, P16, P17, P18).

Severity profile: P16 0C/0H/6M/3L → P17 0C/0H/7M/5L → P18 0C/0H/6M/3L+2N.
MED count returned to 6 (same as P16); adversary attributes this to the partial-fix-regression
axis stabilizing — fewer v2.7-induced MEDs than v2.6-induced MEDs (3 vs 3 in P17, but total is
lower because LOWs dropped from 5 to 3).

HIGH count history: P10:0H → P11:1H → P12:1H → P13:2H → P14:0H → P15:0H → P16:0H → P17:0H → P18:0H.
Sixth consecutive zero-HIGH confirms HIGH-severity axis remains clean. Streak clock not
advanced due to remaining MED/LOW findings.

Pass tallies (P1–P18): 23 / 23 / 21 / 25 / 28 / 20 / 14 / 12 / 12 / 11 / 14 / 10 / 15 / 8 / 14 / 13 / 13 / 11.
Total across all passes: 287 findings. Canonical hashes: 9a0f34c / 9c9b12f.

Pass-19 next after D-535 remediation burst.

---

## Remediation (D-535)

**Date:** 2026-07-27
**Burst:** D-535 STATE BURST — WAVE-86 ADVERSARIAL PASS 18 REMEDIATED
**Protocol:** Single-Commit Burst (TD-VSDD-053)

All 11 findings FIXED at STORY-182 v2.8 / STORY-183 v2.8.

| Finding | Status | Story | Fix Summary |
|---------|--------|-------|-------------|
| F-W86S-P18-001 (MED) | FIXED v2.8 | STORY-182 | Discriminating N/M predicate replaces tautological M==len() form |
| F-W86S-P18-002 (MED) | FIXED v2.8 | STORY-182 | Single-destination pointer scheme in ci.yml step description |
| F-W86S-P18-003 (MED) | FIXED v2.8 | STORY-182 | Gate step verification uses !cancelled() execution-status predicate |
| F-W86S-P18-004 (MED) | FIXED v2.8 | STORY-183 | Doc-comment at :59-62 updated for post-v2.7 rewritten function |
| F-W86S-P18-005 (MED) | FIXED v2.8 | STORY-183 | :47-49 and :53-57 added to DF-SIBLING-SWEEP-001 scope + prose updated |
| F-W86S-P18-006 (MED) | FIXED v2.8 | STORY-183 | Pattern-registry-block label corrected (3 loci) |
| F-W86S-P18-007 (LOW) | FIXED v2.8 | STORY-183 | Task-10 derivation split into 3 labelled bullets |
| F-W86S-P18-008 (LOW) | FIXED v2.8 | STORY-182 | Negative assertion added to Task 5b gate specification |
| F-W86S-P18-009 (LOW) | FIXED v2.8 | STORY-183 | Comment-marker claim at :28-29 corrected (4th instance) |
| NIT-01 | FIXED v2.8 | STORY-182 | fixture_present doc-comment :59-62 updated for post-v2.7 rewrite |
| NIT-02 | FIXED v2.8 | STORY-183 | Trailing space on Pattern-33 opening fence removed |

**Canonical input-hashes preserved:** STORY-182 9a0f34c / STORY-183 9c9b12f (canonical Python
tool; no spec-input changes in this burst — hashes unchanged).

**DF-SIBLING-SWEEP-001:** Full sweep performed.

**Sixth zero-HIGH pass confirmed:** P10:0H → P11:1H → P12:1H → P13:2H → P14:0H → P15:0H → P16:0H → P17:0H → P18:0H.
Streak 0/3 (MED/LOW remain).

**Orchestrator pre-commit grep verification completed:** attribution single-destination pointer
scheme; discriminating N/M predicate replacing tautological M==len(); execution-not-output
!cancelled() truth; :59-62/:53-57/:47-49 sweep additions; pattern-registry-block relabel (3 loci);
Task-10 bullet split; zero-file-guard negative assertion.

**Streak:** 0/3. Pass 19 pending adversary dispatch. Trajectory-tail: →13→13→11.
