---
document_type: burst-log
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-07-23T00:05:00Z
cycle: "wave-085"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Burst Log — wave-085

## Burst 1 (2026-07-23) — Session Resume + Wave-85 Scoping

Session resumed from D-492 pause (human-approved, 2026-07-23). Wave-85 IEC-104 completion mini-wave scoped. IEC104-TIMED-CMD-GAP-001 DF-VALIDATION-001 research dispatched; SEC-001 + ROUTE-W74 pulled into wave-85. Full structured entry below.

---

## Burst: D-493 SESSION RESUMED + WAVE-85 SCOPED (2026-07-23)

**Parent-commit:** `a1676f0d` — HEAD of factory-artifacts at session resume (factory(pause): session wrap — post-v0.13.1 clean milestone; maint-2026-07-21 COMPLETE; DRIFT-BACKMERGE-SQUASH-001 RESOLVED (D-492)).

**Adversary verdict:** N/A — bookkeeping burst; no adversarial pass conducted as part of this burst. This is a session-resume + wave-scoping state update, not a spec-evolution or code-delivery burst in its own right.

**Files touched (Dim-1): 5 unique files**

- .factory/STATE.md (D-493 transition: current_step, current_cycle wave-085, pipeline ACTIVE, timestamp, EXACT RESUME POINT, Project Metadata Mode + Last Updated rows, Phase Progress wave-085 row, Concurrent Cycles wave-085 row, CPS D-493 add + D-488 roll, Decisions Log D-493, Active Carry-Forwards SEC-001/ROUTE-W74/IEC104-TIMED-CMD-GAP-001 targets, Session Resume Checkpoint, Historical Content notes, size budget banner)
- .factory/cycles/wave-084/burst-log.md (D-488 CPS archival appended)
- .factory/cycles/wave-084/session-checkpoints.md (D-492 checkpoint archived)
- .factory/cycles/wave-085/burst-log.md (this file — created)
- .factory/sidecar-learning.md (uncommitted session-marker lines included in commit)

**Codifications:** None — pure state bookkeeping and wave scoping. No new BCs, VPs, or stories authored. Story authoring for IEC104-TIMED-CMD-GAP-001 blocked on DF-VALIDATION-001 research completion.

**Summary:** Session resumed from D-492 pause (human-approved, 2026-07-23). Worktree health PASS (factory-artifacts a1676f0d in-sync). Ground truth verified: develop=dc7331fb (unchanged), main=47b7d23c (v0.13.1), only open PR = external #407 (DEFERRED, unchanged). Human selected Option A: wave-85 IEC-104 completion mini-wave. Wave-85 scope (all human decisions): (1) IEC104-TIMED-CMD-GAP-001 detection story — DF-VALIDATION-001 research validation DISPATCHED (research-agent, in flight; report target .factory/planning/iec104-timed-cmd-gap-validation.md); (2) IEC-104 holdout scenario authoring; (3) SEC-001 ENIP split-borrow refactor — PULLED INTO WAVE-85 (target-passed resolved); (4) ROUTE-W74 deferred NIT — PULLED INTO WAVE-85 (target-passed resolved). Options B/C/D NOT selected. develop=dc7331fb (UNCHANGED — no code changes this burst). Pipeline ACTIVE.

**Dim-2 Attestation:** N/A — factory-only burst; develop branch UNCHANGED (dc7331fb); no compilation or test execution.
**Dim-5 Attestation:** N/A — no WASM binary changes.
**Dim-6 Attestation:** N/A — no source code changes on develop branch. Burst commits exclusively to factory-artifacts branch.
**Dim-7 Attestation:** N/A — no test suite changes.

**Closes:** D-493 SESSION RESUMED + WAVE-85 SCOPED; pipeline ACTIVE; wave-085 cycle opened.

---

---

## Archived CPS Row — D-489 (rolled from STATE.md CPS under last-5 rule, D-494 burst)

| **D-489 SESSION RESUMED + MAINTENANCE SWEEP maint-2026-07-21 STARTED (2026-07-21, human-approved). Worktree health PASS; develop=1e967bad verified; open PRs = Dependabot #422-425 + external #407 (both deferred, verified). Maintenance sweep maint-2026-07-21 STARTED (human-selected from idle work menu). Human scope decisions: (a) dep-soak eligibility measured from upstream RELEASE DATE, not Dependabot PR open date — security-relevant bumps considered regardless of soak; (b) NO carry-forwards pulled in (PERF-RERUN-001, Routes B/C, PG-W84 DF-VALIDATION-001 all remain at their stated targets). Sweeps 1-5,7,8 dispatched; Sweep 6 DTU SKIP (dtu_required:false); Sweep 9 a11y SKIP (no UI). trajectory-tail →0→0→0→0** | **COMPLETE (D-489)** | maint-2026-07-21 IN PROGRESS → superseded by D-490. trajectory-tail →0→0→0→0 |

---

## Burst 2 (2026-07-23) — Spec-Evolution + Story-Creation Finalization

D-494 WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE. Research validation CONFIRMED HIGH; PO burst: BC-2.19.029/030/022v1.1 + HS-133..136; story burst: STORY-180/181 drafted + STORY-170 v2.1 propagated.

---

## Burst: D-494 WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE (2026-07-23)

**Parent-commit:** factory-artifacts HEAD at time of this burst (prior = D-493 session-resume commit).

**Adversary verdict:** N/A — spec-evolution + story-creation burst; adversarial convergence is the NEXT step (3 clean passes, BC-5.39.001).

**Files touched (Dim-1): 17 unique files**

- .factory/STATE.md (D-494 transition: frontmatter prd_version v1.57→v1.58, current_step D-494, timestamp refresh; EXACT RESUME POINT D-494; Project Metadata spec versions + stories rows; Phase Progress wave-085 row; Concurrent Cycles wave-085 row; CPS D-494 add + D-489 rolled; Decisions Log D-494; Active Carry-Forwards IEC104/SEC-001/ROUTE-W74/OBS-2 updated; Session Resume Checkpoint D-494)
- .factory/cycles/wave-085/burst-log.md (D-489 CPS archival + D-494 burst entry — this file)
- .factory/holdout-scenarios/HS-INDEX.md (v2.14→v2.15; HS-133..136 added)
- .factory/holdout-scenarios/HS-133-iec104-timed-switching-cmds-t1692001.md (NEW)
- .factory/holdout-scenarios/HS-134-iec104-timed-setpoint-bitstring-t1692001-t0836.md (NEW)
- .factory/holdout-scenarios/HS-135-iec104-timed-parity-neighbor-silence-guard.md (NEW)
- .factory/holdout-scenarios/HS-136-iec104-timed-control-real-world-corpus.md (NEW)
- .factory/sidecar-learning.md (session-marker lines)
- .factory/specs/behavioral-contracts/BC-INDEX.md (v2.34→v2.35; BC-2.19.029/030 added, BC-2.19.022 v1.1 noted)
- .factory/specs/behavioral-contracts/ss-19/BC-2.19.022.md (v1.0→v1.1 silent-set narrowed to {52-57, 65-99})
- .factory/specs/behavioral-contracts/ss-19/BC-2.19.029.md (NEW v1.0: timed switching TypeIDs 58-60 → T1692.001)
- .factory/specs/behavioral-contracts/ss-19/BC-2.19.030.md (NEW v1.0: timed set-point/bitstring TypeIDs 61-64 → T1692.001+T0836)
- .factory/specs/prd.md (v1.57→v1.58: §2.19.E rows + §2.19.H BC-2.19.028 backfill + v1.57/v1.58 changelog entries)
- .factory/stories/STORY-INDEX.md (v3.87→v3.88; STORY-180/181 added; STORY-170 v2.1 annotated; 132→134 stories, 775→783 pts)
- .factory/stories/STORY-170.md (v2.0→v2.1 propagation: BC-2.19.022 v1.1 range annotation-only)
- .factory/stories/STORY-180.md (NEW: E-22, 5 pts, wave 85, IEC-104 timed control detection, BC-2.19.029/030/022v1.1)
- .factory/stories/STORY-181.md (NEW: E-20, 3 pts, wave 85, SEC-001 ENIP split-borrow + ROUTE-W74 OBS-1 AC-181-004, BC-2.17.016)

**Codifications:**
- IEC104-TIMED-CMD-GAP-001 CONFIRMED HIGH (DF-VALIDATION-001, planning/iec104-timed-cmd-gap-validation.md)
- BC-2.19.029 NEW v1.0: timed switching commands TypeIDs 58-60 → MITRE T1692.001
- BC-2.19.030 NEW v1.0: timed set-point/bitstring TypeIDs 61-64 → T1692.001 + T0836
- BC-2.19.022 v1.0→v1.1: silent set narrowed from {52-99} to {52-57, 65-99}
- HS-133..136 authored (HS-INDEX v2.14→v2.15)
- prd.md v1.57→v1.58: §2.19.E + §2.19.H BC-2.19.028 backfill
- STORY-180 (E-22, 5 pts, wave 85, IEC-104 timed detection)
- STORY-181 (E-20, 3 pts, wave 85, SEC-001+ROUTE-W74 OBS-1)
- STORY-170 v2.0→v2.1 (BC-2.19.022 v1.1 annotation-only propagation)
- STORY-INDEX v3.87→v3.88 (134 stories / 783 pts)
- ROUTE-W74 disposition: primary absorbed by STORY-166 (wave-84, delivered); OBS-1 residual → AC-181-004 in STORY-181; OBS-2 carry-forward.

**Summary:** Spec-evolution + story-creation burst finalized for wave-85. Research agent confirmed IEC104-TIMED-CMD-GAP-001 HIGH severity via DF-VALIDATION-001. PO authored BC-2.19.029 (NEW), BC-2.19.030 (NEW), and updated BC-2.19.022 v1.1 (silent-set range narrowed). HS-133..136 authored. prd.md updated to v1.58. Story-writer authored STORY-180 (timed detection, E-22, 5 pts) and STORY-181 (SEC-001+ROUTE-W74, E-20, 3 pts), propagated BC-2.19.022 v1.1 to STORY-170 v2.1 (annotation-only). STORY-INDEX v3.88 (134 stories, 783 pts, 85 waves). ROUTE-W74 fully disposed: primary via STORY-166, OBS-1 via AC-181-004, OBS-2 as carry-forward. develop=dc7331fb (UNCHANGED — no code changes). Pipeline ACTIVE. Adversarial convergence next.

**Dim-2 Attestation:** N/A — factory-only burst; develop branch UNCHANGED (dc7331fb).
**Dim-5 Attestation:** N/A — no WASM binary changes.
**Dim-6 Attestation:** N/A — no source code changes on develop branch.
**Dim-7 Attestation:** N/A — no test suite changes.

**Closes:** D-494 WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE. Wave-85 spec locked; adversarial convergence begins next.

---

---

## Archived CPS Row — D-490 (rolled from STATE.md CPS under last-5 rule, D-495 burst)

| **maint-2026-07-21 COMPLETE (2026-07-21, D-490, human-authorized). 8 sweeps total (S1 dep-audit 0C/0H/0M/3L log-only; S2 doc-drift 1H/3M/1L ALL FIXED PR #431 6c47c0ef IEC-104 README+ADR-0001/0002/0012/CLAUDE.md; S3 pattern 4 log-only/NIT; S4 holdout repair HS-087/123/125/132 HS-INDEX v2.14; S5 perf VALID 5OK/2WARN-noise/0CRIT AC-149-003 PASS 23.659µs; S6 DTU SKIP dtu_required:false; S7 spec-coherence 4 new SPEC-008/009/010/011 all addressed; S8 register v2.0 15 new rows 10 resolutions; S9 a11y SKIP no UI). Dependabot #422-425 batch-merged (orchestrator-executed). PR #431 doc-drift (human-executed post-classifier-halt). ARCH-INDEX v2.19→v2.20 (SS-19). STORY-INDEX v3.86→v3.87 (epic TOTAL). develop=6c47c0ef. trajectory-tail →0→0→0→0** | **COMPLETE (D-490)** | maint-2026-07-21 COMPLETE. trajectory-tail →0→0→0→0 |

---

## Burst 3 (2026-07-23) — Adversarial Pass 1 Remediation

D-495 WAVE-85 ADVERSARIAL PASS 1 → REMEDIATED. Pass-1 adversary (spec+story package @ 2202c5b3): 1 CRIT / 2 HIGH / 4 MED / 2 LOW. All remediations applied; F-P1-005 DISPUTED/NON-FIX. HS-INDEX v2.16. Next: adversary pass 2 (fresh context).

---

## Burst: D-495 WAVE-85 ADVERSARIAL PASS 1 → REMEDIATED (2026-07-23)

**Parent-commit:** factory-artifacts HEAD at time of this burst (prior = D-494 spec-evolution + story-creation commit).

**Adversary verdict:** PASS-1 REMEDIATED — 1 CRIT / 2 HIGH / 4 MED / 2 LOW. All actionable findings remediated. F-P1-005 DISPUTED/NON-FIX (MED, process-gap): '## Category: real-world-corpus' heading is template-mandated structural section (validate-template-compliance hook exit_code=2); PO rebuttal accepted pending pass-2 fresh-context confirmation.

**Files touched (Dim-1): 11 unique files**

- .factory/STATE.md (D-495 transition: frontmatter current_step + timestamp; EXACT RESUME POINT D-495; Phase Progress wave-085 row + convergence trajectory; Concurrent Cycles wave-085 row; CPS D-495 add + D-490 rolled; Decisions Log D-495; Session Resume Checkpoint D-495)
- .factory/cycles/wave-085/burst-log.md (D-490 CPS archival + D-495 burst entry — this file)
- .factory/stories/STORY-181.md (v2.0→v2.1 rewritten: SEC-001 re-anchored to enip.rs:992-999 self/self.flows split-borrow via *mut EnipFlowState; take-remove-reinsert fix + specific grep exit gate; input-hash 8253122 unchanged)
- .factory/stories/STORY-170.md (v2.1 modified-note softened; input-hash → 7873f11)
- .factory/tech-debt-register.md (SEC-001 description corrected to enip.rs:992-999 split-borrow via *mut EnipFlowState)
- .factory/maintenance/risk-assumption-monitoring.md (SEC-001 sibling description corrected per DF-SIBLING-SWEEP-001)
- .factory/holdout-scenarios/HS-133-iec104-timed-switching-cmds-t1692001.md (count=0 fix BC-2.19.029/030 Invariant 3; APCI LEN 0x13→0x15; BC-2.19.028 dropped; Fixture Creation Obligation added)
- .factory/holdout-scenarios/HS-134-iec104-timed-setpoint-bitstring-t1692001-t0836.md (count=0 fix; APCI LEN A/B 0x12→0x17 / C 0x13→0x19 / D 0x12→0x18; C_BO_TA_1 QOS field removed per IEC 60870-5-101 Table 8; BC-2.19.028 dropped; Fixture Creation Obligation added)
- .factory/holdout-scenarios/HS-135-iec104-timed-parity-neighbor-silence-guard.md (BC-2.19.017 frontmatter added; Fixture Creation Obligation added)
- .factory/holdout-scenarios/HS-136-iec104-timed-control-real-world-corpus.md (count=0 fix per BC-2.19.029/030 Invariant 3 — count-independent; Fixture Creation Obligation added)
- .factory/holdout-scenarios/HS-INDEX.md (v2.15→v2.16; BC-column updates for HS-133..136)

**Codifications:**
- F-W85S-P1-001 CRITICAL REMEDIATED: STORY-181 re-anchored from enip.rs:825-829 (already-safe carry split-borrow using std::mem::take) to enip.rs:992-999 (real+only unsafe self/self.flows split-borrow via *mut EnipFlowState). STORY-181 rewritten to target take-remove-reinsert fix pattern.
- F-P1-002 HIGH REMEDIATED: tech-debt-register SEC-001 description corrected (DF-SIBLING-SWEEP-001: sibling risk-assumption-monitoring.md also corrected).
- F-P1-003 HIGH REMEDIATED: HS-136 count=0 contradiction fixed — count-independent per BC-2.19.029/030 Invariant 3 (not Inv 2).
- F-P1-004 MED REMEDIATED: HS-135 BC-2.19.017 frontmatter added.
- F-P1-005 MED DISPUTED/NON-FIX (process-gap): '## Category: real-world-corpus' heading is template-mandated structural section (validate-template-compliance hook exit_code=2), not copy-paste artifact; PO rebuttal accepted pending pass-2 fresh-context confirmation.
- F-P1-006 MED REMEDIATED: Fixture Creation Obligation sections added to HS-133/134/135.
- F-P1-007 MED REMEDIATED: APCI LEN bytes recomputed (HS-133 0x13→0x15; HS-134 A/B 0x12→0x17, C 0x13→0x19, D 0x12→0x18 + C_BO_TA_1 QOS field removed per IEC 60870-5-101 Table 8).
- F-P1-008 LOW REMEDIATED: STORY-170 modified-note softened (input-hash → 7873f11).
- F-P1-009 LOW REMEDIATED: BC-2.19.028 dropped from HS-133/134.

**Summary:** Wave-85 adversarial pass-1 remediation burst. Orchestrator verified CRITICAL finding against src/analyzer/enip.rs: STORY-181 was mis-anchored to already-safe carry split-borrow (std::mem::take at enip.rs:825-829); real+only unsafe split-borrow is self/self.flows via *mut EnipFlowState at enip.rs:992-999. STORY-181 rewritten to target 992-999 with take-remove-reinsert fix and specific grep exit gate. All 8 actionable findings remediated. F-P1-005 DISPUTED/NON-FIX per PO rebuttal (template-mandated structural section, validate-template-compliance hook exit_code=2). HS-INDEX v2.16. develop=dc7331fb (UNCHANGED — no code changes). Pipeline ACTIVE. Adversary pass 2 next (fresh context).

**Dim-2 Attestation:** N/A — factory-only burst; develop branch UNCHANGED (dc7331fb).
**Dim-5 Attestation:** N/A — no WASM binary changes.
**Dim-6 Attestation:** N/A — no source code changes on develop branch. Burst commits exclusively to factory-artifacts branch.
**Dim-7 Attestation:** N/A — no test suite changes.

**Closes:** D-495 WAVE-85 ADVERSARIAL PASS 1 → REMEDIATED. Pass-2 adversarial review next (fresh context).

---

---

## Archived CPS Row — D-491 (rolled from STATE.md CPS under last-5 rule, D-496 burst)

| **v0.13.1 RELEASED (2026-07-21, D-491). Dev-tooling patch: green-doc-tense patterns 26-29, validate-citations path:line:anchor, gitignore mutants guard, IEC-104 doc-drift batch (PRs #422-430 wave-84 + maint). Release PR #432 47b7d23c squash-merged to main (human-executed). Tag v0.13.1 lightweight (orchestrator-pushed under explicit human 'proceed' authorization); tag_object = commit SHA. GH release 4 assets (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu). Back-merge PR #433 TRUE-MERGE dc7331fb to develop (human decision). DRIFT-BACKMERGE-SQUASH-001 RESOLVED: main IS ancestor of develop (git merge-base --is-ancestor PASS), first time since v0.12.1/D-436. main=47b7d23c. develop=dc7331fb. trajectory-tail →0→0→0→0** | **RELEASED (D-491)** | v0.13.1 RELEASED. DRIFT-BACKMERGE-SQUASH-001 RESOLVED. trajectory-tail →0→0→0→0 |

---

## Burst 4 (2026-07-23) — Adversarial Pass 2 Remediation

D-496 WAVE-85 ADVERSARIAL PASS 2 → REMEDIATED. Pass-2 adversary (spec+story @ 304bb465, fresh context): 0 CRIT / 0 HIGH / 3 MED / 1 LOW / 1 process-gap. NO merge-blocker. All 4 actionable findings fixed; PG-W85-001 adjudicated upstream. HS-INDEX v2.17. Next: adversary pass 3 (fresh context).

---

## Burst: D-496 WAVE-85 ADVERSARIAL PASS 2 → REMEDIATED (2026-07-23)

**Parent-commit:** factory-artifacts HEAD at time of this burst (prior = D-495 pass-1 remediation commit).

**Adversary verdict:** PASS-2 REMEDIATED — 0 CRIT / 0 HIGH / 3 MED / 1 LOW / 1 process-gap. NO merge-blocker (zero HIGH/CRIT). All 4 actionable findings fixed. F-P2-005 (process-gap) adjudicated as plugin-level defect, NOT a per-file fix; PG-W85-001 filed for upstream resolution.

**Files touched (Dim-1): 5 unique files**

- .factory/STATE.md (D-496 transition: frontmatter current_step + timestamp; EXACT RESUME POINT D-496; Project Metadata Last Updated; Phase Progress wave-085 row + trajectory; Convergence Status wave-85 trajectory; Concurrent Cycles wave-085 row; CPS D-496 add + D-491 rolled; Decisions Log D-496; Drift Items PG-W85-001 added; Session Resume Checkpoint D-496)
- .factory/cycles/wave-085/burst-log.md (D-491 CPS archival + D-496 burst entry — this file)
- .factory/cycles/wave-085/session-checkpoints.md (D-495 checkpoint archived — created this burst)
- .factory/stories/STORY-170.md (line 62: silently-logged range {1–57,65–99,...}→{1–44,52–57,65–99,...})
- .factory/holdout-scenarios/HS-135-iec104-timed-parity-neighbor-silence-guard.md (Case C/D LEN 0x0B→0x0E)
- .factory/holdout-scenarios/HS-136-iec104-timed-control-real-world-corpus.md (BC-2.19.028 citation dropped; Case D jq regex fixed to negate timed-mnemonic set)
- .factory/holdout-scenarios/HS-INDEX.md (v2.16→v2.17; HS-136 BC-column + narrative BC-2.19.028 removal)
- .factory/sidecar-learning.md (session-marker lines)

**Codifications:**
- F-P2-001 MED REMEDIATED: STORY-170:62 silently-logged range corrected {1–57,65–99,...}→{1–44,52–57,65–99,...} (was wrongly folding handled TypeIDs 45–51); 17-site sibling sweep confirmed no other stale ranges. STORY-170 input-hash 7873f11 (unchanged — annotation-only).
- F-P2-002 MED REMEDIATED: HS-136 BC-2.19.028 dropped (Inv-3 text mismatch + DoS-cap not exercised by any case); now absent across all HS-133..136 + HS-INDEX.
- F-P2-003 MED REMEDIATED: HS-136 Case D dead jq regex (_NA/_NB/_NC mnemonics match nothing) fixed to negate timed-mnemonic set (parity with Case A); iec104.rs:764/805 confirm actual summaries use C_SC/C_DC/C_RC and C_SE/C_BO mnemonics.
- F-P2-004 LOW REMEDIATED: HS-135 Case C/D frame LEN 0x0B→0x0E.
- F-P2-005 MED[process-gap] PG-W85-001 ADJUDICATED: plugin-level template+hook defect — holdout-scenario-template.md + validate-template-compliance.sh treat '## Category: real-world-corpus' as unconditionally-required (only 6/136 HS files carry it; HS-122/132 lack it), forcing a contradictory heading on non-corpus files. NOT a per-file fix; does NOT block wave-85 convergence. Filed as PG-W85-001 → DF-VALIDATION-001 + upstream drbothen/vsdd-factory.
- HS-INDEX v2.16→v2.17.

**Summary:** Wave-85 adversarial pass-2 remediation burst. Pass-2 adversary reviewed spec+story package at 304bb465 in fresh context: 0 CRIT / 0 HIGH / 3 MED / 1 LOW / 1 process-gap. No merge-blocker findings. F-P2-001: STORY-170 line 62 silently-logged range was incorrectly listing TypeIDs 45–51 (already handled by the detection path) — corrected to {1–44,52–57,65–99,...} reflecting actual gap; 17-site sibling sweep confirmed no other stale ranges. F-P2-002: HS-136 BC-2.19.028 citation dropped — the Invariant 3 text does not match BC-2.19.028 semantics and no case exercises the DoS-cap constraint; now absent from all HS-133..136 + HS-INDEX. F-P2-003: HS-136 Case D jq regex was keying on _NA/_NB/_NC mnemonics which match no actual iec104.rs output; fixed to negate the timed-mnemonic set (C_SC_NA_1, C_DC_NA_1, etc.) for parity with Case A. F-P2-004: HS-135 Case C/D APCI LEN corrected 0x0B→0x0E. F-P2-005 adjudicated as plugin defect (PG-W85-001): the '## Category: real-world-corpus' heading is a plugin template/hook artefact, not a per-file obligation on HS-135; filed for upstream fix; does not block wave-85 convergence. develop=dc7331fb (UNCHANGED — no code changes). Pipeline ACTIVE. Adversary pass 3 next (fresh context).

**Dim-2 Attestation:** N/A — factory-only burst; develop branch UNCHANGED (dc7331fb).
**Dim-5 Attestation:** N/A — no WASM binary changes.
**Dim-6 Attestation:** N/A — no source code changes on develop branch. Burst commits exclusively to factory-artifacts branch.
**Dim-7 Attestation:** N/A — no test suite changes.

**Closes:** D-496 WAVE-85 ADVERSARIAL PASS 2 → REMEDIATED. Pass-3 adversarial review next (fresh context). clean-pass count = 0 of 3.

---

## Archived CPS Row — D-492 (rolled from STATE.md CPS under last-5 rule, D-497 burst)

| **SESSION WRAP (2026-07-21, D-492). Human-requested pause at clean post-release milestone. Session D-489..D-491 (exhaustive): maint-2026-07-21 COMPLETE (8 sweeps, 0 CRIT/HIGH residual; Dependabot #422-425 + doc-drift #431 merged; holdouts repaired HS-INDEX v2.14; ARCH-INDEX v2.20/STORY-INDEX v3.87; register v2.0); v0.13.1 RELEASED (PR #432 47b7d23c, tag v0.13.1 lightweight, GH release 4 assets); back-merge #433 TRUE-MERGE dc7331fb — DRIFT-BACKMERGE-SQUASH-001 RESOLVED. No in-flight work, no story worktrees, no factory lock. Pipeline PAUSED. trajectory-tail →0→0→0→0** | **PAUSED (D-492)** | Pipeline PAUSED at clean post-release milestone. No in-flight work. trajectory-tail →0→0→0→0 |

---

## Burst 5 (2026-07-23) — Adversarial Pass 3 Remediation

D-497 WAVE-85 ADVERSARIAL PASS 3 → REMEDIATED. Pass-3 adversary (spec+story @ dcc8cc06, fresh context): 0 CRIT / 0 HIGH / 1 MED. NO merge-blocker. F-P3-001: STORY-170 AC-170-005 Note {1–44,...} fix. Clean-pass counter still 0/3. Next: adversary pass 4 (fresh context).

---

## Burst: D-497 WAVE-85 ADVERSARIAL PASS 3 → REMEDIATED (2026-07-23)

**Parent-commit:** `dcc8cc06` — factory(wave-085): adversarial pass-2 remediated (D-496).

**Adversary verdict:** PASS-3 REMEDIATED — 0 CRIT / 0 HIGH / 1 MED. NO merge-blocker. F-P3-001 (MED): STORY-170 AC-170-005 Note (lines 105-106) dropped the [1,44] monitoring-direction segment — partial-fix residual of pass-2 (line 62 BC-table fixed, sibling Note locus missed). 12 other review axes independently re-verified clean by pass-3 (SEC-001 anchor, APCI LEN byte-recompute, jq filters, count=0, BC-2.19.028 orphan-free, TypeID/technique maps, index arithmetic, AC↔BC traces, canonical-frame coverage). Clean-pass counter still 0/3.

**Files touched (Dim-1): 4 unique files**

- .factory/STATE.md (D-497 transition: frontmatter current_step + timestamp; EXACT RESUME POINT D-497; Project Metadata Last Updated; Phase Progress wave-085 row + trajectory; Convergence Status wave-85 trajectory; Concurrent Cycles wave-085 row; CPS D-497 add + D-492 rolled; Decisions Log D-497; Session Resume Checkpoint D-497)
- .factory/cycles/wave-085/burst-log.md (D-492 CPS archival + D-497 burst entry — this file)
- .factory/cycles/wave-085/session-checkpoints.md (D-496 checkpoint archived)
- .factory/stories/STORY-170.md (AC-170-005 Note lines 105-106: {52–57, 65–99,...} → {1–44, 52–57, 65–99,...}; all 11 STORY-170 silent-set loci now consistent; input-hash 7873f11 unchanged)

**Codifications:**
- F-P3-001 MED REMEDIATED: STORY-170 AC-170-005 Note (lines 105-106) corrected — the Note text referred to "silently-logged set" as `{52–57, 65–99, 102, 104, 106–127}` but was missing the `1–44` monitoring-direction segment that had already been correct in line 62 BC-table (pass-2 partial-fix). Corrected to `{1–44, 52–57, 65–99, 102, 104, 106–127}`. Exhaustive in-file sweep confirmed all 11 STORY-170 silent-set loci now consistent; cross-file clean. STORY-170 hash 7873f11 (unchanged — annotation-only fix).
- 12 other review axes independently re-verified clean: SEC-001 anchor (enip.rs:992-999), APCI LEN byte-recompute (HS-133/134 fixed in P1/P2), jq filters (HS-136 Case D fixed in P2), count=0 (Inv-3 fixed in P1), BC-2.19.028 orphan-free (dropped from HS-133/134 in P1, HS-136 in P2, HS-INDEX in P2), TypeID/technique maps (exhaustive), index arithmetic (STORY-INDEX v3.88 consistent), AC↔BC traces (all STORY-170/180/181 ACs trace), canonical-frame coverage (19 frames per F5 baseline).

**Summary:** Wave-85 adversarial pass-3 remediation burst. Pass-3 adversary reviewed spec+story package at dcc8cc06 (fresh context): 0 CRIT / 0 HIGH / 1 MED. F-P3-001: STORY-170 AC-170-005 Note (lines 105-106) was missing the `1–44` monitoring-direction segment in the silently-logged set description — a partial-fix residual from pass-2 (line 62 BC-table row was fixed in P2, but this sibling Note locus at lines 105-106 was missed). Corrected to `{1–44, 52–57, 65–99, 102, 104, 106–127}`; exhaustive in-file sweep confirmed all 11 STORY-170 silent-set loci now consistent; no cross-file stale instances found. 12 other review axes independently re-verified clean. STORY-170 hash 7873f11 unchanged (annotation-only). develop=dc7331fb (UNCHANGED — no code changes). Pipeline ACTIVE. Clean-pass counter still 0/3. Adversary pass 4 next (fresh context).

**Dim-2 Attestation:** N/A — factory-only burst; develop branch UNCHANGED (dc7331fb).
**Dim-5 Attestation:** N/A — no WASM binary changes.
**Dim-6 Attestation:** N/A — no source code changes on develop branch. Burst commits exclusively to factory-artifacts branch.
**Dim-7 Attestation:** N/A — no test suite changes.

**Closes:** D-497 WAVE-85 ADVERSARIAL PASS 3 → REMEDIATED. Pass-4 adversarial review next (fresh context). clean-pass count = 0 of 3.

---

## Archived CPS Row — D-493 (rolled from STATE.md CPS under last-5 rule, D-498 burst)

| **D-493 SESSION RESUMED + WAVE-85 SCOPED (human-approved, 2026-07-23). Resumed from D-492 pause. Worktree health PASS (factory-artifacts a1676f0d in-sync). Ground truth verified: develop=dc7331fb, main=47b7d23c (v0.13.1); only open PR = external #407 (DEFERRED, unchanged). Human selected Option A: wave-85 IEC-104 completion mini-wave. Wave-85 scope: (1) IEC104-TIMED-CMD-GAP-001 detection story — DF-VALIDATION-001 research validation DISPATCHED; (2) IEC-104 holdout scenario authoring; (3) SEC-001 ENIP split-borrow refactor — PULLED INTO WAVE-85; (4) ROUTE-W74 deferred NIT — PULLED INTO WAVE-85. Options B/C/D NOT selected. Pipeline ACTIVE. trajectory-tail →0→0→0→0** | **ACTIVE (D-493)** | Session resumed + wave-85 scoped. Spec-evolution + story-creation next. trajectory-tail →0→0→0→0 |

---

## Burst 6 (2026-07-23) — Adversarial Pass 4 Remediation

D-498 WAVE-85 ADVERSARIAL PASS 4 → REMEDIATED. Pass-4 adversary (spec+story @ 097c3dd1, fresh context): 0 CRIT / 1 HIGH / 0 MED. F-P4-001 (HIGH): STORY-181 title/FSR/AC-trace + risk-register.md R-010 Direction-Keyed Carry Select framing purged. STORY-INDEX v3.89. PG-W85-002 filed. Clean-pass counter still 0/3. Next: adversary pass 5 (fresh context).

---

## Burst: D-498 WAVE-85 ADVERSARIAL PASS 4 → REMEDIATED (2026-07-23)

**Parent-commit:** `097c3dd1` — factory(wave-085): adversarial pass-3 remediated (D-497).

**Adversary verdict:** PASS-4 REMEDIATED — 0 CRIT / 1 HIGH / 0 MED. F-P4-001 (HIGH): STORY-181 body correctly targets the *mut EnipFlowState PDU-dispatch-loop fix (enip.rs:992-999, take-remove-reinsert) but 3 loci retained the REJECTED "Direction-Keyed Carry Select" framing from the pass-1 CRITICAL finding. Second-order propagation tail of F-P1-001. All 4 loci fixed (story-writer 27-hit exhaustive sweep caught 4th locus). 12 other review axes re-verified clean. Clean-pass counter still 0/3.

**Files touched (Dim-1): 6 unique files**

- .factory/STATE.md (D-498 transition: frontmatter current_step + story_index_version v3.88→v3.89 + story_index_note prepended v3.89 entry + timestamp; EXACT RESUME POINT D-498; Project Metadata Last Updated; Phase Progress wave-085 row + trajectory `1C+2H+4M+2L→3M/1L(P2)→1M(P3)→1H(P4)→pending-P5`; Convergence Status trajectory; Concurrent Cycles wave-085 row; CPS D-498 add + D-493 rolled; Decisions Log D-498; Drift Items PG-W85-002 added; Session Resume Checkpoint D-498)
- .factory/cycles/wave-085/burst-log.md (D-493 CPS archival + D-498 burst entry — this file)
- .factory/cycles/wave-085/session-checkpoints.md (D-497 checkpoint archived)
- .factory/stories/STORY-INDEX.md (v3.88→v3.89: STORY-181 line 334 title cell corrected — Direction-Keyed Carry Select framing removed; correct framing Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop; no numeric totals changed)
- .factory/stories/STORY-181.md (line 262 FSR cell + line 119 AC-181-003 trace re-framed to *mut/PDU-dispatch-loop framing; Direction-Keyed Carry Select references purged; input-hash 8253122 unchanged)
- .factory/specs/domain-spec/risk-register.md (R-010 description + mitigation corrected to enip.rs:992-999 PDU dispatch loop split-borrow via *mut EnipFlowState; take-remove-reinsert pattern absorbed into STORY-181; input-hash 0447a72→865986f)

**Codifications:**
- F-P4-001 HIGH REMEDIATED: STORY-INDEX line 334 title cell (canonical story registry) — Direction-Keyed Carry Select framing replaced with "Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop". STORY-INDEX v3.88→v3.89 (patch bump, title-cell correction only; no numeric story/wave/point totals changed).
- F-P4-001 HIGH REMEDIATED: STORY-181 line 262 FSR normative cell — stale "carry-select in split-borrow context (enip.rs:825-829)" framing replaced with "Replace unsafe *mut EnipFlowState split-borrow in the PDU dispatch loop (enip.rs:992-999) with take-remove-reinsert of the owned flow; no behavior change". Direction-Keyed Carry Select framing purged.
- F-P4-001 HIGH REMEDIATED: STORY-181 line 119 AC-181-003 trace — "refactor scope is the PDU dispatch loop only; carry select at 825-829 untouched" note added (BC-2.17.016 invariant cite). Direction-Keyed Carry Select framing purged.
- F-P4-001 HIGH REMEDIATED (4th locus, story-writer exhaustive sweep): risk-register.md R-010 description + mitigation corrected to target enip.rs:992-999 PDU dispatch loop *mut EnipFlowState split-borrow; take-remove-reinsert pattern noted as absorbed into STORY-181. risk-register input-hash 0447a72→865986f.
- 27-hit exhaustive sweep: 23 remaining occurrences of carry-related text verified correct (already-safe notes, unrelated TLS/DNP3/Modbus carry logic). Zero stale loci remaining.
- 12 axes re-verified clean by pass-4: TypeID enums, SEC-001 anchor, APCI LEN, jq filters, count=0, BC-2.19.028 orphan-free, arithmetic, AC↔BC, EC cites, canonical-frame, RED-tense.
- PG-W85-002 filed: recurring remediation-sweep locus-coverage gap — passes P2/P3/P4 each found stale framing not covered by the preceding sweep. Remediation sweeps must cover index titles, FSR cells, AC traces, and cross-spec risk-register loci, not just the cited line.

**Summary:** Wave-85 adversarial pass-4 remediation burst. Pass-4 adversary reviewed spec+story package at 097c3dd1 in fresh context: 0 CRIT / 0 HIGH / 0 MED initially; 1 HIGH F-P4-001 identified: STORY-181 body correctly targeted the *mut EnipFlowState PDU-dispatch-loop fix at enip.rs:992-999 (take-remove-reinsert), but 3 loci in the same STORY-181 file still retained the REJECTED Direction-Keyed Carry Select framing that had been the root of the pass-1 CRITICAL finding — specifically the STORY-INDEX line 334 title cell, STORY-181 line 262 FSR normative cell (which incorrectly directed implementers to the 825-829 carry region the story explicitly forbids touching), and STORY-181 line 119 AC-181-003 trace. Story-writer dispatched a 27-hit exhaustive sweep and caught a 4th locus in risk-register.md R-010 with the same stale framing. All 4 loci fixed. 23 remaining carry-related occurrences verified correct. 12 other review axes independently re-verified clean. STORY-181 full-section consistency confirmed. Hashes: STORY-181 8253122 (unchanged), risk-register 0447a72→865986f. STORY-INDEX v3.89 (patch bump, title-cell correction only). Process-gap PG-W85-002 filed: 3 consecutive passes (P2→STORY-170:62, P3→STORY-170:105, P4→STORY-181 title/FSR/trace + risk-register) found partial-fix propagation residuals — recurring remediation-sweep locus-coverage gap flagged for cycle-close codification. develop=dc7331fb (UNCHANGED — no code changes). Pipeline ACTIVE. Clean-pass counter still 0/3. Adversary pass 5 next (fresh context).

**Dim-2 Attestation:** N/A — factory-only burst; develop branch UNCHANGED (dc7331fb).
**Dim-5 Attestation:** N/A — no WASM binary changes.
**Dim-6 Attestation:** N/A — no source code changes on develop branch. Burst commits exclusively to factory-artifacts branch.
**Dim-7 Attestation:** N/A — no test suite changes.

**Closes:** D-498 WAVE-85 ADVERSARIAL PASS 4 → REMEDIATED. Pass-5 adversarial review next (fresh context). clean-pass count = 0 of 3.

---

<!-- Repeat for each burst. Maintain chronological order. -->
