# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 11

**Date:** 2026-06-24
**Cycle:** feature-enip-v0.11.0
**Phase:** F3 — Incremental Stories (Adversarial Convergence)
**Scope:** STORY-130..138 (9 stories, E-20, waves 58-61, 66 pts) + HS-110..122 (13 holdout scenarios)
**Adversary:** Independent (no prior pass context)

---

## VERDICT: PASS

0 CRITICAL, 0 HIGH, 1 MEDIUM (REMEDIATED), 4 observations (non-blocking, confirmed locked/known).
Novelty: LOW.

Adversary independently re-confirms: **"the decomposition has CONVERGED."**

All structural axes clean:
- 26-BC coverage (BC-2.17.001..026) — all assigned, no orphans
- VP-032 placement — correct (F6 Kani harness obligation)
- Increment-site single-ownership — verified; no double-increment paths
- Finding-field exactness — category/verdict/confidence/summary/evidence/mitre_techniques verified byte-for-byte against BC postconditions
- Holdout linkage — HS-110..122 all traceable to stories; HS-110 satisfies DF-CANONICAL-FRAME-HOLDOUT-001
- Acyclicity — dependency graph verified; STORY-131 depends_on:[] intentional (parallel W58 root)
- Scope gates — F-P9-001 (0x00B1 deferral) consistently enforced across all detection ACs
- BC-2.17.016 PC-4 terse vs STORY-137 correct — confirmed match; terse is adequate

---

## Findings

### F-P11-001 [MEDIUM] — REMEDIATED

**Story:** STORY-135, line 220 (Library & Framework Requirements) + Architecture Mapping table row  
**Finding:** `write_count_in_window` prose typed as "`u64` millis" in the Architecture Mapping table, while the window-tracking arithmetic in AC-135-003 (postcondition 3/4), BC-2.17.008/012, and the `>1`/`>10` second comparisons all use `u32` second-resolution values. A `u64` millisecond counter paired with `>1`/`>10` comparisons would fire immediately (any timestamp > 1 ms), breaking the 1s and 10s burst windows.

**Root cause:** Copy-paste artifact — Architecture Mapping table row for `write_count_in_window` was authored with `u64` from the aggregate `write_count: u64` row; the "millis" qualifier was carried over from an earlier draft before the u32-seconds decision was locked in BC-2.17.012 postcondition 3.

**Load-bearing:** YES. The type contradiction would misdirect the implementer on `EnipFlowState.write_count_in_window` field type, producing a silent burst-window breakage (threshold comparison arithmetic mismatch).

**Remediation applied:**
1. Architecture Mapping table — `write_count_in_window` row: `u64` → `u32`, added "NOT milliseconds" note.
2. Tasks section — `write_count_in_window: u64` → `u32`, added "u32 seconds NOT milliseconds".
3. Library & Framework Requirements — clarified that window-tracking arithmetic uses `u32` second-resolution values; no `u64` millisecond counters; added `>1`/`>10` comparison context.

**Note:** `EnipAnalyzer.write_count: u64` (aggregate lifetime write counter, Architecture Mapping row 4) correctly remains `u64` — this is the per-session cumulative count, not a window counter, and `u64` is appropriate there. Only `write_count_in_window` (per-window reset counter) was wrong.

**Status: REMEDIATED.**

---

## Observations (non-blocking)

| ID | Observation | Status |
|----|-------------|--------|
| OBS-P11-001 | STORY-131 `depends_on: []` — intentional parallel W58 root entry point; STORY-132..138 all depend on STORY-130 or STORY-131 downstream. No issue. | CONFIRMED LOCKED |
| OBS-P11-002 | BC-2.17.016 PC-4 terse description vs STORY-137 full AC prose — STORY-137 is the load-bearing document; BC PC-4 adequately summarizes. Known pattern in this codebase. | CONFIRMED KNOWN |
| OBS-P11-003 | BC-2.17.010 Description "per-occurrence" — known deferred PO polish item, non-load-bearing. Flagged in prior passes. | KNOWN-DEFERRED PO fix |
| OBS-P11-004 | BC frontmatter `input-hash: TBD` across all STORY-130..138 — F4 obligation (bin/compute-input-hash --write --scan after stories are frozen + input files stable). Correct deferral; no F3 action needed. | [process-gap] F4 obligation |

---

## Structural Axes Summary (all CLEAN)

| Axis | Result | Notes |
|------|--------|-------|
| 26-BC coverage | CLEAN | All BC-2.17.001..026 assigned; no orphans |
| VP-032 placement | CLEAN | Correctly deferred to F6 Kani phase |
| Increment-site single-ownership | CLEAN | `write_count += 1` (aggregate) vs `write_count_in_window += 1` (window) — separate, correct |
| Finding-field exactness | CLEAN | category/verdict/confidence/summary/evidence byte-for-byte verified against BC postconditions |
| Holdout linkage | CLEAN | HS-110..122 all traceable; HS-110 satisfies DF-CANONICAL-FRAME-HOLDOUT-001 |
| Acyclicity | CLEAN | Dependency graph verified; STORY-131 intentional root |
| Scope gates | CLEAN | F-P9-001 (0x00B1) consistently enforced |
| BC PC terse/story full prose | CLEAN | BC adequately summarizes; story is load-bearing |

---

## Severity Trajectory (F3 Adversarial Story Review)

| Pass | C | H | M | L | Result |
|------|---|---|---|---|--------|
| P1 | 4 | 6 | — | — | FAIL — REMEDIATED |
| P2 | 1 | 3 | — | — | FAIL — REMEDIATED |
| P3 | 0 | 2 | — | — | FAIL — REMEDIATED |
| P4 | 2 | 2 | — | — | FAIL — REMEDIATED |
| P5 | 0 | 1 | — | — | FAIL — REMEDIATED |
| P6 | 0 | 1 | — | — | FAIL — REMEDIATED |
| P7 | 0 | 0 | — | — | PASS |
| P8 | 0 | 1 | — | 1 | FAIL — REMEDIATED (1H) |
| P9 | 0 | 2 | — | — | FAIL — REMEDIATED (2H: F-P9-001 0x00B1 scope reduction + session-handshake BC) |
| P10 | 0 | 0 | 0 | 0 | **PASS (1/3)** |
| **P11** | **0** | **0** | **1** | **0** | **PASS (2/3) — 1M REMEDIATED** |

Convergence counter: **2/3**. Pass 12 pending for 3-consecutive-clean confirmation.

---

## Conclusion

The F3 story decomposition for feature-enip-v0.11.0 (SS-17, STORY-130..138) has converged. Pass 11 is PASS with one MEDIUM finding remediated (F-P11-001: `write_count_in_window` type correction `u64 millis` → `u32 seconds`). All structural axes clean. Content frozen pending Pass 12 final confirmation.

Pass 12 expected to be the final confirmation pass (3/3 consecutive clean on HIGH/CRITICAL → F3 gate can close).
