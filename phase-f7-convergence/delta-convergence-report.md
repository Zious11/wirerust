---
document_type: delta-convergence-report
producer: orchestrator
date: 2026-07-17
feature: feature-iec104
reviewed_sha: b36b884
---

# Delta Convergence Report: feature-iec104 (IEC 60870-5-104 passive analyzer)

## Feature Summary

- **Stories:** STORY-167..174 (8 stories, waves 76–83) + fix PRs FIX-P4-001, FIX-F5-001/002/003/004
- **Spec:** PRD v1.56; BC-INDEX v2.33 (30 new BCs: BC-2.19.001–028 + BC-2.05.012 + BC-2.10.010 +
  BC-2.12.025); VP-INDEX v2.46 (VP-044/045/046/047); ARCH-INDEX v2.19 (SS-19, ADR-013)
- **develop:** b36b884 (12 unreleased: STORY-167..174 PRs #401–409 + FIX-P4-001 #410 +
  FIX-F5-001..004 #411/#412/#413/#415)

---

## Five-Dimensional Convergence (Delta)

| Dimension      | Metric                              | Target | Actual                                                                | Status |
|----------------|-------------------------------------|--------|-----------------------------------------------------------------------|--------|
| Spec           | Adversary novelty (F5 R5)           | <0.15  | LOW (NITPICK_ONLY)                                                    | PASS   |
| Test           | Mutation kill rate (F6)             | ≥90%   | 95.9% (118/123)                                                       | PASS   |
| Implementation | Open CRITICAL/HIGH (F5 R2+)         | 0      | 0                                                                     | PASS   |
| Verification   | Kani + fuzz + audit (F6)            | All pass | Kani 5 harnesses SUCCESSFUL; fuzz 2.64M runs / 0 crashes; cargo-audit 0 vulns | PASS |
| Holdout        | Black-box acceptance satisfaction   | ≥0.85  | 0.99 (RELEASE-READY; must-pass #1/#4/#6 all 1.0)                     | PASS   |

---

## Regression

2627 pass / 0 fail; clippy -D warnings clean; fmt clean (F6 confirmation).

---

## Input-Hash Drift

6 delta stories (STORY-167..172) re-baselined — adjudicated BENIGN by F7 consistency audit
(consistency-audit.md). STORY-164/165 report STALE but are pre-feature stories, out of scope
for this feature's F7 gate; separate re-baseline pass required at next opportunity.

---

## Consistency Audit

2 MINOR doc-only findings — both code-correct, non-blocking, deferred to cycle-close:

- **B-001:** PRD RTM entry for BC-2.19.006 carries stale title text; code behavior correct.
- **B-002:** BC-2.19.002 PC-2 references T0814 (superseded by BC-2.19.026 PC-4 for reserved
  TypeIDs); no code impact.

Full audit: `.factory/phase-f7-convergence/consistency-audit.md`.

---

## Holdout Evaluation

- **Method:** holdout-evaluator, strict information asymmetry, canonical IEC 60870-5-104 frames
  synthesized independently (9 scenarios, 14 pcaps).
- **Mean score:** 0.99 (RELEASE-READY).
- **Must-pass scenarios:** #1 (STARTDT/STOPDT session lifecycle), #4 (ASDU control-command
  detection), #6 (findings cap enforcement) — all scored 1.0.
- **Not exercised by holdout** (covered by other means): N(S)/N(R) desync + multi-frame carry
  paths — covered by VP-045 proptest + VP-047 fuzz (2.64M execs / 0 crashes).

---

## Adversarial Summary (F5)

5 rounds to convergence. Feature code frozen since R2 (9c5aa9a); R3–R5 tail addressed
demo-evidence/CHANGELOG doc-accuracy only (root cause: PG-DEMO-JSON-FABRICATION).
R5 NITPICK_ONLY: 0 CRITICAL/HIGH/MEDIUM; 1 LOW non-blocking (IEC104-DEMO-TYPEID45-MISLABEL —
TypeID 45 prose-only mislabel in demo-evidence, code correct at iec104.rs:744–748).
BC-completeness 31/31 + canonical-frame 19 byte-exact clean.

Fix PRs: FIX-F5-001 (#411), FIX-F5-002 (#412), FIX-F5-003 (#413), FIX-F5-004 (#415).

---

## F6 Hardening Summary

- Kani VP-044/004/007: all SUCCESSFUL (VP-044: 89 checks / 5 facets; VP-004: 440/407/183;
  VP-007: 122, SEEDED=29); VP-045/046 proptest non-vacuous; VP-047 fuzz 2.64M runs / 0 crashes.
- Mutation testing: 95.9% (118/123); 5 equivalent survivors documented.
- cargo-audit: 0 vulnerabilities / 193 deps.
- semgrep: skipped (absent); cargo-audit + per-PR security reviews cover surface.

---

## Cost-Benefit (DF-027)

F5 ran 5 rounds: R2 code-converged; R3–R5 were demo-doc accuracy (root cause
PG-DEMO-JSON-FABRICATION). Additional refinement cycles would target only sub-LOW doc nits.
MAXIMUM_VIABLE_REFINEMENT_REACHED for code; residual doc items batched to cycle-close.

---

## Recommendation

**CONVERGED — READY FOR RELEASE**

Release HELD by human direction (2026-07-17); v0.13.0 MINOR cut deferred.
All 5 dimensions: PASS. Regression: 2627/0 CLEAN. Consistency: 2 MINOR doc-only (non-blocking).
F7 human gate: convergence approved, release-cut deferred.
