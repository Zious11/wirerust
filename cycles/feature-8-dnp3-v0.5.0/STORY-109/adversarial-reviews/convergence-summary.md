---
document_type: convergence-summary
story: STORY-109
wave: 38
feature: "#8-dnp3"
cycle: feature-8-dnp3-v0.5.0
producer: state-manager
timestamp: 2026-06-12T01:12:31Z
pr: "#228"
merge_commit: 34443f9
verdict: CONVERGED
clean_streak: 3/3
bc_gate: BC-5.39.001
---

# STORY-109 Adversarial Convergence Summary

Wave 38, Feature #8 DNP3. CONVERGED after 13 passes (3 consecutive CLEAN — passes 11/12/13).

## Trajectory

| Pass | Verdict | Findings | Fix Summary |
|------|---------|----------|-------------|
| P1 | FINDINGS | Malformed-category mis-tagged as `Anomaly` instead of `Security`; 3 adversary summary-format issues | Category corrected; summary prose aligned |
| P2 | FINDINGS-MINOR | 4 BC-PC1 summary/evidence prose drifts between story AC text and implementation | Reconciled 4 evidence-string drifts |
| P3 | FINDINGS-MAJOR | T1691.001 evidence format diverged: implementation emitted free-form prose vs BC-2.15.014 PC1 required `block_event_count={count} in correlation window; threshold={threshold}`; BC-2.15.014 bumped v1.5→v1.6 | Evidence format canonicalized; BC-2.15.014 v1.6 accepted |
| P4 | CLEAN | — | — |
| P5 | CLEAN | — | — |
| P6 | FINDINGS-MAJOR | BC-2.15.023 ENABLE/DISABLE evidence format divergence — the one un-pinned sibling evidence string not caught in earlier passes | Evidence string pinned to match BC spec |
| P7 | FINDINGS-MINOR (NIT) | Minor prose nit | Resolved cosmetically |
| P8 | CLEAN | — | — |
| P9 | FINDINGS-MAJOR | F-P9-001: T0827 loss-of-control failed to emit when block-crossing arrival order and `block_event_count < 3` — real functional detection bug; T0827 emission path not reached for the first event on a new block crossing | Hoisted `maybe_emit_t0827` call; +2 regression tests added |
| P10 | FINDINGS-MINOR (NIT) | Minor doc/test alignment nit | Resolved |
| P11 | CLEAN (+ realignment) | +1 coverage test for byte-walk recover branch (adjudication Step 6 STORY-109-resync) | Adjudication doc written; BC-2.15.016 v1.2 accepted |
| P12 | CLEAN | — | — |
| P13 | CLEAN | — | — |

## Mid-Delivery Regressions Fixed

5 prior-story regression failures were fixed during STORY-109 delivery:
- **correlation_window_seeded spurious-expiry bug:** a window-seeded guard condition
  caused the correlation window to expire when `block_event_count` was already
  set, triggering false-positive T1691.001 emissions in regression tests.
- **STORY-107 carry-cap/EC-006 test evolution:** authorized by the resync adjudication
  (see `.factory/phase-f2-spec-evolution/STORY-109-resync-adjudication.md`);
  BC-2.15.016 EC-007 resync carry-forward spec clarified.

## Spec Evolution During STORY-109

| BC | Change | Detail |
|----|--------|--------|
| BC-2.15.016 | v1.1 → v1.2 | Added EC-007 byte-walk-forward carry resync; realizes STORY-107 deferred resync; architect-adjudicated (doc: `.factory/phase-f2-spec-evolution/STORY-109-resync-adjudication.md`) |
| BC-2.15.014 | v1.5 → v1.6 | T1691.001 PC3 evidence format reconciled to producible format: `block_event_count={count} in correlation window; threshold={threshold}` |
| BC-2.15.024 EC-006 | prose correction | Bailed-flow no-op behavior aligned with BC-2.15.009 PC5; story EC-006 corrected; BC-2.15.024 prose deferred (see DRIFT-BC-2.15.024-EC006-PROSE-001) |

## Input-Hash Record

| Story | Hash at Delivery | Note |
|-------|-----------------|------|
| STORY-107 | 8d3d02a | Regenerated during STORY-109 resync adjudication |
| STORY-109 | f0fb436 | Regenerated before delivery |
| STORY-110 | a9cdfb5 | Regenerated; confirm TBD dup cleaned before delivery |

Full scan result: MATCH=62 / STALE=0. STORY-091 structural ERROR is pre-existing and out of scope.

## VP-007 Update

VP-007 seeded atomically with T1691.001 + T0827:
- Sub-B proof asserts resolvability only (sound: does not require T0835/T0831 to be
  emitted — see DRIFT-MITRE-EMITTED-LABEL-001).
- MitreTactic::IcsImpact added to catalog.

## Delivery Record

- PR #228 merged into develop @ 2026-06-12T01:12:31Z
- Merge commit: `34443f9`
- develop HEAD after merge: `34443f9`
- main HEAD: `c2df1b5` (v0.5.0 — unchanged)
- Input-hash STORY-109 regenerated to `f0fb436` before delivery
