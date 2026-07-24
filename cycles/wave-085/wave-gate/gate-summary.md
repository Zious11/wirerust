---
document_type: wave-gate-summary
level: ops
version: "1.0"
status: closed
producer: state-manager
timestamp: 2026-07-24T00:00:00Z
cycle: "wave-085"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Wave-85 Integration Gate Summary

**Decision:** D-510 — 2026-07-24  
**Verdict:** GATE CLOSED — all six gates pass/skip; DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED  
**develop HEAD at close:** `0ab6f52ee3be21687437d29923fadc903ca70387` (PR #439, gate-fix)  
**Wave stories:** STORY-180 (PR #437 421bf572) + STORY-181 (PR #438 5555495b)  
**Fix-PR chain:** #439 0ab6f52e (ITI e2e 31→66 fixture expectation, derived decomposition)

---

## Six-Gate Verdict Table

| Gate | Name | Verdict | Notes |
|------|------|---------|-------|
| 1 | Test suite | **PASS** (after gate-fix) | Initial run: FAIL — ITI e2e 31-vs-66 (machine-local fixture not committed to worktree; timed tests only present on fixture-bearing host). Gate-fix PR #439 0ab6f52e: updated e2e expectation to 66 timed; derived decomposition +35 timed = 15×TypeID-58/59 + 10×TypeID-61/63×2; T1692.001 assertions 46, T0836 assertions 20. Re-validated PASS: full suite 0 failed, clippy `-D warnings` exit 0, `cargo fmt --check` clean, bin self-tests 162/162, release build OK. |
| 2 | DTU validation | **SKIP** | `dtu_required: false`. Passive network analyzer; no external service calls; no DTU-covered modules. |
| 3 | Adversarial review | **PASS / CONVERGED** | 3 passes; all NITPICK_ONLY (P1/P2/P3); code frozen `0ab6f52e`; DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. Zero CRIT/HIGH/MED across all passes. 1 gate-fix PR delivered (see below). |
| 3b | Consistency + code review | **PASS** | consistency-validator: 3 MINOR (CV-W85G-001/002/003 — all remediated this burst). code-reviewer: 0 MAJOR / 1 MINOR / 5 NIT (CR-001..006 — see code-review.md for full dispositions). security-reviewer: APPROVE (0C/0H/0M/0L; SEC-001 closure confirmed sound). |
| 4 | Demo evidence | **PASS** | STORY-180: 8 demo artifacts on develop. STORY-181: 5 demo artifacts on develop. Demo-evidence path-scrub PASSED (PG-W70-DEMO-SCRUB). |
| 5 | Holdout evaluation | **PASS** | Mean score 0.98. HS-133/134/135 = 1.0 (IEC-104 timed control-command detection TypeIDs 58/59/61/63). HS-136 = 0.9 (corpus-availability caveat — score reflects ITI corpus completeness, not a product defect). ENIP HS-118/120 = 1.0 (no STORY-181 regression). Wave-level integration demo evidence = holdout evaluator's real-capture ITI corpus runs; 0 timed false-positives on known-good traffic. |

---

## Gate-3 Adversarial Trajectory

Wave-level adversarial review over the wave-85 diff (STORY-180 + STORY-181 combined):

| Pass | Verdict | Findings | Fix-PR |
|------|---------|----------|--------|
| P1 | NITPICK_ONLY | F-W85G-P1-001 LOW: duplicate demo evidence line in STORY-180 evidence artifact (cosmetic — demo line repeated once) | #439 0ab6f52e (ITI e2e + P1 nit fixed together) |
| P2 | NITPICK_ONLY | 2 non-defect observations (factory-side; see code-review.md §Adversary Observations) | — |
| P3 | NITPICK_ONLY | F-W85G-P3-001 LOW: tech-debt-register SEC-001 line-cite 992-999 → 993-1000 (FIXED this burst); F-W85G-P3-002 LOW: no-action observation; F-W85G-P3-003 INFO: informational | — (F-W85G-P3-001 fixed in factory artifacts this burst) |

**Trajectory shorthand:** `NITPICK/1L(P1) → NITPICK/0(P2) → NITPICK/2L-factory+1I(P3) → CONVERGED 3/3`

---

## Fix-PR Chain

| PR | SHA | Title | Findings fixed |
|----|-----|-------|----------------|
| #439 | `0ab6f52e` | fix(wave-85): gate ITI e2e fixture expectation + P1 demo nit | ITI e2e 31→66 timed; F-W85G-P1-001 LOW demo dup line |

Human-authorized squash-merge; CI full-suite 0 failed, clippy/fmt clean, bin self-tests 162/162, release build OK.

---

## Holdout Evaluation Table

| Scenario | Score | Notes |
|----------|-------|-------|
| HS-133 (IEC-104 TypeID 58/59 timed control detection) | 1.0 | PASS — correct detection on ITI corpus TypeID-58/59 frames |
| HS-134 (IEC-104 TypeID 61 timed control detection) | 1.0 | PASS — correct detection on ITI corpus TypeID-61 frames |
| HS-135 (IEC-104 TypeID 63 timed control detection) | 1.0 | PASS — correct detection on ITI corpus TypeID-63 frames |
| HS-136 (IEC-104 timed control corpus availability) | 0.9 | CORPUS-CAVEAT — score reflects ITI corpus coverage of TypeIDs 58-64; not a product defect; real captures with these TypeIDs are sparse in the public corpus |
| HS-118 (ENIP regression: SEC-001 fix no false-neg) | 1.0 | PASS — STORY-181 refactor produces identical ENIP detection output; no regression |
| HS-120 (ENIP regression: take-remove-reinsert semantics) | 1.0 | PASS — flow-state lifecycle preserved; no detection-path divergence |

**Mean holdout score: 0.98** (5 × 1.0 + 1 × 0.9) / 6 = 0.983 ≈ 0.98

---

## GATE_CHECK Telemetry

```
GATE_CHECK gate=1 status=PASS note="Initial FAIL (ITI e2e 31-vs-66 machine-local fixture). Gate-fix #439 0ab6f52e: +35 timed expectations. Re-validated: full suite 0 failed; clippy -D warnings exit 0; fmt clean; bin self-tests 162/162; release build OK. develop=0ab6f52e."
GATE_CHECK gate=2 status=SKIP note="dtu_required:false. Passive analyzer. No DTU-covered modules."
GATE_CHECK gate=3 status=PASS note="CONVERGED. 3 passes. ALL NITPICK_ONLY (P1/P2/P3). Streak P1/P2/P3 = 3/3. Zero CRIT/HIGH/MED. 1 gate-fix PR (#439). Code frozen 0ab6f52e. DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED."
GATE_CHECK gate=3b status=PASS note="consistency-validator 3 MINOR (CV-W85G-001/002/003 remediated this burst); code-reviewer 0 MAJOR/1 MINOR/5 NIT (CR-001..006 dispositioned — see code-review.md); security APPROVE 0C/0H/0M/0L (SEC-001 closure confirmed sound)."
GATE_CHECK gate=4 status=PASS note="STORY-180 8 demo artifacts + STORY-181 5 demo artifacts on develop 0ab6f52e. PG-W70-DEMO-SCRUB PASSED."
GATE_CHECK gate=5 status=PASS note="Mean 0.98. HS-133/134/135=1.0; HS-136=0.9 corpus-caveat (not a product defect); ENIP HS-118/120=1.0 no regression. ITI real-capture runs; 0 timed false-positives on known-good traffic."
```
