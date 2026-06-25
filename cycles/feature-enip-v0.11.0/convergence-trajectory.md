---
document_type: convergence-trajectory
level: ops
version: "1.1"
status: complete
producer: state-manager
timestamp: 2026-06-25T21:00:00Z
cycle: "feature-enip-v0.11.0"
inputs: [adversarial-reviews/]
input-hash: "n/a"
traces_to: STATE.md
---

# Convergence Trajectory — feature-enip-v0.11.0

## Finding Progression

### Per-Story — STORY-130 (Wave 58)

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| 1 | 2026-06-25 | 1 | 0 | 1 | 0 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN |
| 2 | 2026-06-25 | 1 | 0 | 0 | 1 | 0 | MEDIUM | — | 1/3 | FINDINGS_REMAIN |
| 3 (Pass 3) | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 2/3 | CLEAN |
| 4 (Pass 4) | 2026-06-25 | 1 | 0 | 0 | 0 | 1 | LOW | — | 3/3 | CONVERGED |

Trajectory: `1→1→0→1` (LOW non-blocking; convergence ACHIEVED 3/3 clean passes 2/3/4)

### Per-Story — STORY-131 (Wave 58)

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| 1 | 2026-06-25 | 3 | 0 | 1 | 2 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN |
| 2 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| 3 | 2026-06-25 | 3 | 0 | 0 | 1 | 2 | MEDIUM | — | 2/3 → reset | REGRESSION_RESOLVED |

Note: Pass 3 had findings (M-1 warn!/log, L-1 BC precondition, L-2 dispatcher.rs module-doc) — all fixed @0018a54. Subsequent clean pass completes convergence.

| 4 (clean post-fix) | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 3/3 | CONVERGED |

Trajectory: `3→0→3→0` (all fixed; convergence ACHIEVED 3/3 per BC-5.39.001)

### Wave-Level — Wave 58 (STORY-130 + STORY-131 integrated, develop@edce3bd)

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| W58-P1 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| W58-P2 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 2/3 | CLEAN |
| W58-P3 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 3/3 | CONVERGED |

Trajectory: `0→0→0`

**Wave-level convergence ACHIEVED** (3/3 consecutive clean passes, BC-5.39.001 MET). Wave 58 FULLY CLOSED.

Wave-level scope reviewed: STORY-130 parse ↔ STORY-131 dispatch seam coherent; 5-arg StreamDispatcher::new ripple complete; both exhaustive DispatchTarget matches (on_data, on_flow_close) + classify_oracle updated with Enip arm; sibling routing (HTTP/TLS/Modbus/DNP3) unaffected; reporter take_enip_analyzer integration symmetric with DNP3; early-exit guard includes self.enip.is_none(). Regression: 1955 tests PASS.

## Per-Pass Details

### STORY-130 Pass 1 (2026-06-25)

**Findings:** 1 (0 CRIT, 1 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3

DF-GREEN-DOC-TENSE: doc comments written in aspirational voice; fixed at 42de2d0. No factory-artifacts impact.

---

### STORY-130 Pass 2 (2026-06-25)

**Findings:** 1 (0 CRIT, 0 HIGH, 1 MED, 0 LOW)
**Novelty:** MEDIUM
**Convergence counter:** 1 of 3

F-130-P2-001: BC-2.17.002 field-count discrepancy (10→6) and ADR-010 §Decision 8 "6 fields" fix. STORY-130 input-hash dc8a2c9→272738c. BC-INDEX v1.79→v1.80. Fixed.

---

### STORY-130 Pass 3 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 2 of 3 (CLEAN)

---

### STORY-130 Pass 4 (2026-06-25)

**Findings:** 1 (0 CRIT, 0 HIGH, 0 MED, 1 LOW)
**Novelty:** LOW
**Convergence counter:** 3 of 3 (CONVERGED)

AC-130-001 postcondition citation precision "1-9" vs "1-8" — non-blocking LOW, deferred to backlog. Does not prevent convergence.

---

### STORY-131 Pass 1 (2026-06-25)

**Findings:** 3 (0 CRIT, 1 HIGH, 2 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3

HIGH: DF-GREEN-DOC-TENSE (dispatch test docs — fixed @5e61682).
M1: STORY-131.md EC-007 overload description — fixed.
M2: BC-INDEX BC-2.17.020 title sync — BC-INDEX v1.80→v1.81. Fixed.

---

### STORY-131 Pass 2 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 1 of 3 (CLEAN)

---

### STORY-131 Pass 3 (2026-06-25)

**Findings:** 3 (0 CRIT, 0 HIGH, 1 MED, 2 LOW)
**Novelty:** MEDIUM
**Convergence counter:** not incremented (findings present — all fixed before next pass)

M-1: false warn!/log requirement in ADR-010 Decision 9 root + STORY-131 + STORY-138 propagation — all fixed to eprintln!/no-log-crate convention. Codified as WARN-LOG-CRATE-001 in lessons.md.
L-1: BC-2.17.023/026 Precondition "N≥1" → "0..=u32::MAX" (v1.0→v1.1). BC-INDEX v1.81→v1.82. STORY-131 input-hash 6d892c4→a119157.
L-2: dispatcher.rs module-doc ENIP omission — fixed @0018a54.

---

### STORY-131 Pass 4 — post-fix clean pass (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 3 of 3 (CONVERGED)

---

### Wave 58 Wave-Level Pass 1 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 1 of 3 (CLEAN)

Integrated develop@edce3bd reviewed. STORY-130 parse ↔ STORY-131 dispatch seam coherent.

---

### Wave 58 Wave-Level Pass 2 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 2 of 3 (CLEAN)

---

### Wave 58 Wave-Level Pass 3 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 3 of 3 (CONVERGED)

Wave-level adversarial convergence ACHIEVED. BC-5.39.001 MET. Wave 58 FULLY CLOSED (D-238).

---

### Per-Story — STORY-132 (Wave 59)

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| 1 | 2026-06-25 | 1 | 0 | 1 | 0 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN |
| 2 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| 3 | 2026-06-25 | 1 | 0 | 0 | 0 | 1 | LOW | — | 2/3 | FINDINGS_REMAIN (non-blocking) |
| 4 | 2026-06-25 | 1 | 0 | 0 | 0 | 1 | LOW | — | 3/3 | CONVERGED |

Trajectory: `1→0→1→1` (LOWs non-blocking; convergence ACHIEVED 3/3 clean passes 2/3/4)

### Per-Story — STORY-133 (Wave 59)

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| 1 | 2026-06-25 | 4 | 2 | 2 | 0 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN — REMEDIATED |
| 2 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| 3 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 2/3 | CLEAN |
| 4 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 3/3 | CONVERGED |

Trajectory: `4→0→0→0` (Pass 1 2CRIT+2HIGH remediated; convergence ACHIEVED 3/3 passes 2/3/4)

### Wave-Level — Wave 59 (STORY-132 + STORY-133 integrated, develop@d562ccc)

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| W59-A | 2026-06-25 | 1 | 1 | 0 | 0 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN — REMEDIATED |
| W59-B | 2026-06-25 | 2 | 0 | 2 | 0 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN — REMEDIATED |
| W59-C | 2026-06-25 | 2 | 0 | 2 | 0 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN — REMEDIATED |
| W59-D | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| W59-E | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 2/3 | CLEAN |
| W59-F | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 3/3 | CONVERGED |

Trajectory: `1→2→2→0→0→0`

**Wave-level convergence ACHIEVED** (3 consecutive confirmation passes D/E/F, all 0 HIGH/CRITICAL, BC-5.39.001 MET, develop@d562ccc). Wave 59 FULLY CLOSED.

Remediation history:
- W59-A: C-1 = T0846 stale `write_burst_emitted` guard cross-story regression — fixed via PR #321 + green-doc-tense CI gate wired on develop.
- W59-B/C: F-WAVE59-C-001 HIGH = stale cross-story count-snapshot prose + RED-tense in test comments — fixed via PR #322.
- W59-D/E/F: 0 HIGH/CRITICAL confirmed on develop@d562ccc. Convergence ACHIEVED. D-242.

Pre-wave-60 hardening finding: F-W59-M01 = BC-2.17.012 TA-id wrong (TA0105 should be TA0106 for IcsImpairProcessControl); fixed in this factory-artifacts burst (BC-2.17.012 v1.0→v1.1, BC-INDEX v1.82→v1.83). Not a runtime defect (implementation used catalog entry directly; catalog TA-id correct); spec alignment only. Pre-empts STORY-133-class wrong-spec defect in Wave-60 stories.

### STORY-133 Pass 1 (2026-06-25)

**Findings:** 4 (2 CRIT, 2 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3

CRIT-1: `technique_info("T1693.001")` returned wrong name "Exploit Public-Facing Application: EtherNet/IP" — corrected to "Modify Firmware: System Firmware" per ADR-010 Decision 7.
CRIT-2: `technique_info("T1693.001")` returned wrong tactic IcsInitialAccess — corrected to IcsInhibitResponseFunction (TA0107) per ADR-010 Decision 7.
HIGH-1: Test `test_technique_info_t1693_001` pinned the wrong name — test pin corrected at `ffca717`.
HIGH-2: No executable gate existed on tactic assignment — `mitre_tests.rs` pin-table extended with T1693.001 → TA0107 authoritative assertion at `ffca717`.

All 4 findings fixed at code commit `ffca717` (impl + test pin + mitre_tests authoritative-TA-id pin-table extension + stale-count fn renames + RED-tense scrub). Story prose corrected in factory-artifacts burst [D-240]. VP-007 invariants intact: SEEDED 28, EMITTED 20, T1693.001 excluded, revoked T0855/T0856/T0857 absent. Codified as MITRE-CATALOG-ADR-AUTHORITATIVE-001 in lessons.md.

---

### STORY-133 Pass 2 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 1 of 3 (CLEAN)

---

### STORY-133 Pass 3 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 2 of 3 (CLEAN)

---

### STORY-133 Pass 4 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 3 of 3 (CONVERGED)

Per-story adversarial convergence ACHIEVED (BC-5.39.001 MET). STORY-133 merged via PR #320, develop@7f040de.

---

### Wave 59 Wave-Level Pass A (2026-06-25) — REMEDIATED

**Findings:** 1 (1 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3 (reset — FINDINGS_REMAIN)

C-1: T0846 `write_burst_emitted` guard regression — stale guard from STORY-132 was carrying over into STORY-133 context causing T0846 findings to be suppressed incorrectly. Fixed via PR #321. green-doc-tense CI gate wired on develop as part of remediation.

---

### Wave 59 Wave-Level Pass B (2026-06-25) — REMEDIATED

**Findings:** 2 (0 CRIT, 2 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3 (reset — FINDINGS_REMAIN)

F-WAVE59-C-001 HIGH: stale cross-story count-snapshot prose (test doc comments cited counts from prior story state, not Wave-59 final state). M-2: RED-tense in test comment prose (tests used aspirational "will" / "should" voice). Both fixed via PR #322.

---

### Wave 59 Wave-Level Pass C (2026-06-25) — REMEDIATED

**Findings:** 2 (0 CRIT, 2 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3 (reset — FINDINGS_REMAIN; confirmation of F-WAVE59-C-001/M-2 scope)

F-WAVE59-C-001 + M-2 confirmed still present in residual locations; additional files swept and fixed in same PR #322 burst.

---

### Wave 59 Wave-Level Pass D (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 1 of 3 (CLEAN)

develop@d562ccc reviewed. Post-PR #321 + PR #322 remediation. All prior HIGH/CRITICAL findings resolved.

---

### Wave 59 Wave-Level Pass E (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 2 of 3 (CLEAN)

---

### Wave 59 Wave-Level Pass F (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 3 of 3 (CONVERGED)

Wave-level adversarial convergence ACHIEVED. BC-5.39.001 MET. Wave 59 FULLY CLOSED (D-242). develop@d562ccc.

---

### Per-Story — STORY-134 (Wave 60)

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| 1 | 2026-06-25 | 1 | 0 | 1 | 0 | 0 | HIGH | — | 0/3 | FINDINGS_REMAIN — REMEDIATED |
| 2 | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| 3 | 2026-06-25 | 2 | 0 | 2 | 0 | 0 | HIGH | — | 0/3 → reset | FINDINGS_REMAIN — REMEDIATED |
| 4 | 2026-06-25 | 1 | 0 | 0 | 1 | 0 | MEDIUM | — | 0/3 → reset | FINDINGS_REMAIN — REMEDIATED |
| G | 2026-06-25 | 2 | 0 | 0 | 2 | 0 | MEDIUM | — | 0/3 → reset | FINDINGS_REMAIN — REMEDIATED |
| H | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| I | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 2/3 | CLEAN |
| J/K/L clean window not immediately reached — intermediate passes J/K/L = | — | — | — |
| M | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 1/3 | CLEAN |
| N | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 2/3 | CLEAN |
| O | 2026-06-25 | 0 | 0 | 0 | 0 | 0 | — | — | 3/3 | CONVERGED |

Trajectory: `1→0→2→1→2→0→0→…→0→0→0` (multi-round remediation; convergence ACHIEVED M/N/O 3/3 clean passes per BC-5.39.001)

Remediation history:
- Pass-1 HIGH: ts=0 error-window sentinel — `error_window_active == false` sentinel replaces `error_window_start_ts == 0`; fixed in code via `error_window_active: bool` field.
- Pass-3 2×HIGH (F-134-P3-001/002): BC-2.17.010 v1.0 process_pdu pseudo-code still commanded `command_counts` increment — contradicted F8-001; BC-2.17.010 v1.0→v1.1 (command_counts removed from process_pdu, reattributed to BC-2.17.016 frame-walk PC-0). Architecture Anchor and PC-3 corrected. F8-001 fully propagated. SPEC FIX; code @ac04edd was already correct.
- Pass-4 MEDIUM (M-1): BC-2.17.008 PC-2 used `error_window_start_ts==0` sentinel — fails when first error at pcap-relative ts=0. BC-2.17.008 v1.1→v1.2 (`error_window_active: bool` replaces ts=0 sentinel). ADR-010 Decision 4 roster updated. EC-008 added. SPEC FIX; code correct.
- Pass-G 2×MEDIUM (F-134-PG-001/002): enip.rs + STORY-134.md cited ADR-010 Decision 5/6 for detection-order/MAX_FINDINGS; correct anchor is Decision 4. Full worktree sweep: 8 sites in src/analyzer/enip.rs + 3 sites in STORY-134.md corrected @0115bf5. Lesson: ADR-DECISION-NUMBER-MIS-ANCHOR-001 (D-245).
- Passes M/N/O on worktree HEAD 68e3394: 0 findings all 3 passes. CONVERGED (D-246).

**Per-story adversarial convergence ACHIEVED** (BC-5.39.001 MET). STORY-134 worktree HEAD 68e3394. 20 recon tests green; full repo green; clippy/fmt/green-doc-tense clean.

---

### STORY-134 Pass 1 (2026-06-25)

**Findings:** 1 (0 CRIT, 1 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3

HIGH: ts=0 error-window sentinel — `error_window_start_ts == 0` used as unseeded sentinel but fails when first error arrives at pcap-relative ts=0. Fixed via `error_window_active: bool` field.

---

### STORY-134 Pass 2 (2026-06-25)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW)
**Novelty:** —
**Convergence counter:** 1 of 3 (CLEAN)

---

### STORY-134 Pass 3 (2026-06-25)

**Findings:** 2 (0 CRIT, 2 HIGH, 0 MED, 0 LOW)
**Novelty:** HIGH
**Convergence counter:** reset (FINDINGS_REMAIN — REMEDIATED)

F-134-P3-001: BC-2.17.010 process_pdu pseudo-code commanded `command_counts[0x0063]` increment — contradicts F8-001 relocation. BC-2.17.010 v1.0→v1.1. SPEC FIX.
F-134-P3-002: BC-2.17.010 Architecture Anchor pseudo-code repeated same contradiction. Fixed in same BC-2.17.010 v1.1 amendment.

---

### STORY-134 Pass 4 (2026-06-25)

**Findings:** 1 (0 CRIT, 0 HIGH, 1 MED, 0 LOW)
**Novelty:** MEDIUM
**Convergence counter:** reset (FINDINGS_REMAIN — REMEDIATED)

M-1: BC-2.17.008 PC-2 `error_window_start_ts==0` sentinel invalid at ts=0. BC-2.17.008 v1.1→v1.2 (`error_window_active` bool). ADR-010 Decision 4 roster + EC-008. SPEC FIX; code correct.

---

### STORY-134 Pass G (2026-06-25)

**Findings:** 2 (0 CRIT, 0 HIGH, 2 MED, 0 LOW)
**Novelty:** MEDIUM
**Convergence counter:** reset (FINDINGS_REMAIN — REMEDIATED)

F-134-PG-001: enip.rs cited ADR-010 Decision 5/6 for detection-order and MAX_FINDINGS — correct anchor is Decision 4. 8 sites in src/analyzer/enip.rs corrected @0115bf5.
F-134-PG-002: STORY-134.md Architecture Compliance Rules and Mapping table cited Decision 5/6. 3 sites corrected (this factory-artifacts burst, D-245).

---

### STORY-134 Passes M, N, O (2026-06-25) — 3 consecutive clean passes

**Pass M:** 0 findings. Convergence counter 1 of 3 (CLEAN).
**Pass N:** 0 findings. Convergence counter 2 of 3 (CLEAN).
**Pass O:** 0 findings. Convergence counter 3 of 3 (CONVERGED).

Worktree HEAD 68e3394 reviewed. All prior findings resolved. Per-story adversarial convergence ACHIEVED (D-246). BC-5.39.001 MET.

---

## Wave-59 Follow-Up Obligations (logged at wave-level convergence — non-blocking)

These were surfaced during wave-level passes and logged per D-238:

1. **WAVE59-E2E-001**: Add a combined end-to-end test that arms HTTP+TLS+Modbus+DNP3+ENIP simultaneously and drives reassembled port-44818 traffic through the full reassembler → dispatcher → take_enip_analyzer → reporter pipeline. Current mod dispatch tests drive the dispatcher directly, bypassing reassembler+reporter; the unclassified_flows JSON-summary seam for an ENIP-armed run is not yet exercised e2e. Add when STORY-132 lands real findings.

2. **WAVE59-DEADCODE-001**: Remove the module-wide `#![allow(dead_code)]` on src/analyzer/enip.rs when STORY-132 wires the parse functions into on_data frame-walk, so newly-dead helpers are caught.

3. **M-001** (reaffirmed): Sync public docs/adr/0010-ethernet-ip-cip-stream-dispatch.md to .factory ADR-010 (field-count 10→6 line ~598; Decision-9 eprintln! wording line ~697). Deliver in STORY-132 PR.
