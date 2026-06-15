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

## Current Status

**arp_f4_wave_adversary_convergence_counter: 0/3 — RE-STREAK RESTARTED on 6abcd8f (IN PROGRESS)**

Prior 3/3 CONVERGED (fee71ee) invalidated by post-convergence D-075 holdout catch + D-077
CRITICAL surfaced by human-directed 3-pass re-streak. Counter reset to 0/3.

Trajectory shorthand:
`1M→(remediated)→P1/3-CLEAN→P2/3-CLEAN→P3/3-CLEAN-GATE-SATISFIED→[HOLDOUT-PASS;D-075;D-076;D-077-CRITICAL]→RESET-0/3-6abcd8f`

Next action: 3 fresh-context adversary passes on develop 6abcd8f. Mandatory per-pass checks:
field-value verification (`Verdict::Likely` in D1 HIGH) AND reject-path verification
(non-Ethernet hw type + non-IPv4 proto type → `Err` return).
