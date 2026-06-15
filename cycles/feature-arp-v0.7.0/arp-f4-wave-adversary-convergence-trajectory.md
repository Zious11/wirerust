# F4 Wave-Level Adversary Convergence Trajectory — ARP Security Analyzer

Gate: 3 consecutive fresh-context passes with zero findings (any severity) over the full ARP
delta (develop HEAD vs baseline 31d1231).

---

## Pass Summary Table

| Pass | Date | Findings | Severity | Consistency | Counter | Outcome |
|------|------|----------|----------|-------------|---------|---------|
| Pass 1 | 2026-06-15 | 1 genuine + LOW obs | 1 MEDIUM (F-ARP-F4P1-001) | CONSISTENT | 0/3 | NOT CLEAN — remediated via PR #242 + D-074; counter reset |
| Pass 1/3 (restart) | 2026-06-15 | 0 | — | CONSISTENT (full-corpus) | 1/3 | PASS CLEAN — fresh-context, full ARP delta, DF-BC-COMPLETENESS-SWEEP all 15 SS-16 BCs, policy rubric applied (develop HEAD fee71ee) |
| Pass 2/3 | 2026-06-15 | 0 | — | — | 2/3 | PASS CLEAN — fresh-context, full ARP delta, DF-BC-COMPLETENESS-SWEEP all 15 SS-16 BCs |
| Pass 3/3 | 2026-06-15 | 0 | — | CONSISTENT (final full-corpus audit, zero gaps) | 3/3 | PASS CLEAN — fresh-context, full ARP delta, DF-BC-COMPLETENESS-SWEEP all 15 SS-16 BCs. GATE SATISFIED |

---

## Pass 1 Detail (2026-06-15)

**Develop HEAD:** d038711 (STORY-115 delivered; pre-PR-#242)
**Adversary stance:** fresh-context, full ARP delta scope

### Findings

**F-ARP-F4P1-001 (MEDIUM — GENUINE)**
- Title: `--arp-storm-rate 0` and `--arp-spoof-threshold 0` not rejected at CLI.
- Root cause: ARP comparisons are inclusive (`>=`); 0 triggers on every packet (degenerate
  always-fire). No fail-fast guard present post-STORY-115 delivery.
- Adjudication: GENUINE. D-074 issued (2026-06-15): reject 0 with `anyhow::bail!` at CLI
  parse time. Research-agent validated HIGH confidence.
- Remediation: PR #242 (impl commit 3c1cecb, merge commit fee71ee). +10 lines src/main.rs;
  4 new tests in `tests/bc_2_16_d074_arp_threshold_zero_tests.rs` RED→GREEN.
- Spec back-propagation: BC-2.16.008 v1.7→v1.8; BC-2.16.012 v1.2→v1.3; BC-2.16.013 v1.2→v1.3.
- Story back-propagation: STORY-114 v1.1→v1.2; STORY-115 v1.1→v1.2.

**LOW observations (not counted as findings requiring counter reset — informational):**
Noted by adversary; adjudicated as LOW / cosmetic / previously tracked. Details in
adversarial review session. Consistency-validator cross-check: CONSISTENT — zero gaps.

### Outcome

NOT CLEAN (1 MEDIUM finding). Convergence counter = 0/3. Remediation completed.
Next pass = fresh Pass 1 restart (counter does not advance on a non-clean pass).

---

## Clean-Streak Passes (Post-Remediation Restart; develop HEAD fee71ee)

### Pass 1/3 (2026-06-15)

**Develop HEAD:** fee71ee (PR #242 D-074 fix merged)
**Adversary stance:** fresh-context, full ARP delta scope, DF-BC-COMPLETENESS-SWEEP over all 15 SS-16 BCs (BC-2.16.001..015), policy rubric applied

**Findings:** 0

**Consistency check:** Full-corpus consistency-validator audit — CONSISTENT, zero gaps.

**Outcome:** PASS CLEAN. Clean-streak 0/3 → 1/3.

---

### Pass 2/3 (2026-06-15)

**Develop HEAD:** fee71ee
**Adversary stance:** fresh-context, full ARP delta scope, DF-BC-COMPLETENESS-SWEEP over all 15 SS-16 BCs, policy rubric applied

**Findings:** 0

**Outcome:** PASS CLEAN. Clean-streak 1/3 → 2/3.

---

### Pass 3/3 (2026-06-15)

**Develop HEAD:** fee71ee
**Adversary stance:** fresh-context, full ARP delta scope, DF-BC-COMPLETENESS-SWEEP over all 15 SS-16 BCs, policy rubric applied

**Findings:** 0

**Final consistency check:** Full-corpus consistency-validator audit — CONSISTENT, zero gaps.

**Outcome:** PASS CLEAN. Clean-streak 2/3 → 3/3. **F4 WAVE-LEVEL ADVERSARIAL GATE SATISFIED.**

**Process note [scope]:** The adversary agent (operating without Bash access) twice reported
the git ref as a stale value (d038711 / packed-ref lag) while correctly reviewing fee71ee
FILE CONTENTS — the review was accurate because file contents on develop were fee71ee at time
of dispatch. Future adversary dispatches already include the instruction "trust file contents
over .git refs" (added in Pass 3 dispatch). No policy change required; recorded here as a
durable scope note for future audit.

---

---

## F4 Holdout Evaluation (2026-06-15)

**Develop HEAD:** fee71ee (pre-D-075)

### Summary

Initial run mean 0.997. G1 (D1 ARP-spoof verdict field) scored 0.95 — D1 HIGH finding was
emitting `Verdict::Possible` instead of `Verdict::Likely` (BC-2.16.004 L45/L74/L118).
The static adversary (3/3 scenarios) flagged Likely; consistency-validator missed it (checked
structure, not field value).

D-075 issued (2026-06-15): D1 HIGH finding carries `Verdict::Likely`. PR #243 (merge 4ee7a9d).
G1 re-run = 1.0 post-fix.

Full corpus 15/15 post-D-075: mean 1.0. RFC-826 canonical frame scenario: PASS.
Non-D1 verdicts: unregressed.

### Holdout Gate Verdict: PASS

D-076 (PR #244 merge 52437f8): D-075 regression-test doc-comments corrected from present-tense
RED prose to past-tense regression-guard framing. Recurrence of
PG-ARP-F4-REDTEST-DOC-TENSE despite codification — agent-prompt/hook strengthening needed
(recorded as PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE in drift items).

---

## Post-Holdout Human-Directed 3-Pass Adversary Re-Streak (2026-06-15)

**Develop HEAD during re-streak:** 4ee7a9d / 52437f8 → culminating in 6abcd8f after D-077.

The orchestrator directed a focused 3-pass re-streak to probe the D-077 security boundary
(hw/proto type-reject in `extract_arp_frame`), surfacing a CRITICAL defect invisible to 4
prior adversary passes and holdout.

### Re-Streak Finding — D-077 CRITICAL

**BC-2.16.001 PC2/PC3 + BC-2.16.009 PC3a/3b/EC-001/EC-002:**
`extract_arp_frame` admitted crafted valid-size/wrong-type ARP frames (non-Ethernet hw type
or non-IPv4 proto type) into the detection pipeline instead of returning `Err`.
Root cause: half-implemented D11 security boundary. Implementation + unit tests + (deferred)
Kani harness all consistently omitted the type-field check, making the omission self-consistent
and invisible to structural review across 4 prior adversary passes AND holdout.

This is the strongest evidence yet for the "holdout + multi-pass fresh-context re-streak catches
what single-perimeter review misses" principle.

**Remediation:** PR #245 (merge 6abcd8f): `extract_arp_frame` now rejects non-Ethernet
hw_addr_type and non-IPv4 proto_addr_type with `Err`. Security review PASS (CWE-20,
panic-free). F-2 LOW also fixed: GARP-conflict summary now states "with binding conflict"
(BC-2.16.014 PC1).

D-077 issued (2026-06-15). Adversary convergence counter RESET to 0/3.

### Process Gaps Recorded

- **PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE:** PG-ARP-F4-REDTEST-DOC-TENSE recurred in D-075
  regression test despite codification; codified policy text alone insufficient. Agent-prompt /
  hook strengthening needed. Self-improvement epic opened.
- **PG-ARP-F4-TYPE-BRANCH-NARROWING:** Self-consistent omission of reject-branch invisible to
  structural review. DF-BC-COMPLETENESS-SWEEP must cross-check EACH BC's FULL
  precondition/edge-case set (negative/reject branches), not just happy-path + present structure.
- **VP-024 Sub-A Kani (F6 note):** When filled, harness MUST cover type-field rejection (not
  just size). See PG-ARP-F4-TYPE-BRANCH-NARROWING.

---

## Re-Streak (3/3 CONVERGED — bcb1bd6 + specs f9eaccd / VP-024 v1.9)

**Develop HEAD during re-streak:** bcb1bd6 (origin/develop; PR #246 rename-revert + VP-024 v1.9)
**Factory-artifacts HEAD at re-streak start:** f9eaccd

The counter was reset to 0/3 after D-077. A second full re-streak was run on bcb1bd6. Three
independent fresh-context passes each verified:
1. Per-finding field VALUES vs BC (e.g., D1 HIGH → `Verdict::Likely`; BC-2.16.004 L45/L74/L118).
2. The 4-part `extract_arp_frame` reject path (non-Ethernet hw type + non-IPv4 proto type → `Err`).
3. All 15 BCs' (BC-2.16.001..015) full precondition/edge-case sets including negative/reject branches.

Pass 3/3 ran solo for strict independence.

### Re-Streak Pass Summary

| Pass | Date | Findings | Counter | Outcome |
|------|------|----------|---------|---------|
| Re-streak Pass 1/3 | 2026-06-15 | 0 | 1/3 | PASS CLEAN — fresh-context; field-value + reject-path verified |
| Re-streak Pass 2/3 | 2026-06-15 | 0 | 2/3 | PASS CLEAN — fresh-context; all 15 BCs' full precondition sets |
| Re-streak Pass 3/3 | 2026-06-15 | 0 | 3/3 | PASS CLEAN — solo run, strict independence; field-value + reject-path verified |

### Re-Streak Remediation Context (this cycle)

| Item | Description | PR | Develop SHA |
|------|-------------|-----|-------------|
| D-074 | Reject `--arp-storm-rate 0` / `--arp-spoof-threshold 0` at CLI | #242 | fee71ee |
| D-075 | D1 HIGH finding carries `Verdict::Likely` (holdout-caught) | #243 | 4ee7a9d |
| D-076 | D-075 regression-test doc-comments corrected (doc-tense) | #244 | 52437f8 |
| D-077 | CRITICAL: `extract_arp_frame` type-reject security boundary | #245 | 6abcd8f |
| O-1 | VP-024 v1.8 harness rename reverted (propagation fix; v1.9 retained widening) | #246 | bcb1bd6 |

### New Process Gaps (this cycle)

- **PG-ARP-FIXBURST-CONSUMER-SWEEP (NEW):** VP-024 v1.8 harness rename didn't sweep its 11
  consuming artifacts (DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 dim 3 not applied at the rename
  burst); resolved by reverting the rename (PR #246). Lesson: any canonical-symbol rename must
  grep+update ALL consumers in the same burst, or avoid cosmetic renames entirely. OPEN.

### Gate Verdict

**F4 WAVE-LEVEL ADVERSARIAL CONVERGENCE GATE SATISFIED (re-streak 3/3 CLEAN on bcb1bd6).**

Note on earlier 3/3 (fee71ee): the initial gate was satisfied, then post-convergence work (D-075
holdout catch + human-directed re-streak surfacing D-077 CRITICAL) required a second full re-streak.
The gate is now definitively satisfied on bcb1bd6.

---

## Current Status

**arp_f4_wave_adversary_convergence_counter: 3/3 CONVERGED (re-streak on bcb1bd6) — F4 WAVE-LEVEL ADVERSARIAL GATE SATISFIED**

**F4 PHASE COMPLETE:**
- Delta-implementation: 5 stories (STORY-111..115) ALL DELIVERED.
- Wave-level adversarial convergence: 3/3 GATE SATISFIED (re-streak on bcb1bd6).
- Holdout evaluation: GATE PASS (15/15 mean 1.0; RFC-826 canonical PASS).
- PRs this convergence cycle: #242 / #243 / #244 / #245 / #246.

Trajectory shorthand (complete):
`1M→(remediated D-074)→P1/3-CLEAN→P2/3-CLEAN→P3/3-CLEAN-GATE-SATISFIED(fee71ee)→[HOLDOUT-PASS;D-075;D-076;D-077-CRITICAL-RESET]→ReStreak-P1/3-CLEAN→ReStreak-P2/3-CLEAN→ReStreak-P3/3-CLEAN-GATE-SATISFIED(bcb1bd6)`

**Next phase:** F5 scoped-adversarial (`vsdd-factory:phase-f5-scoped-adversarial`).
