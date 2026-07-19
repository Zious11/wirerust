---
document_type: blocking-issues-resolved
level: ops
version: "1.0"
status: archive
producer: state-manager
timestamp: 2026-07-18T04:30:00Z
cycle: "feature-iec104"
inputs: []
input-hash: "d41d8cd"
traces_to: STATE.md
---

# Resolved Carry-Forwards — feature-iec104

Carry-forward items marked RESOLVED/CLOSED during the feature-iec104 cycle, extracted from
STATE.md Active Carry-Forwards table on 2026-07-18. Open items remain in STATE.md.

| ID | Summary | Resolution | Resolved Date |
|----|---------|------------|---------------|
| IEC104-FINDING-DIRECTION-001 | RESOLVED (PR #410, D-464) — CLOSED. All 10 IEC-104 emit sites now direction: Some(...). Finding.direction was None though direction was known; direction threaded through process_u_frame + detect_iec104_threats; 11 direction-assertion tests (mod fix_p4_001). | CLOSED (D-464, PR #410 7e95f71) | 2026-07-17 |
| F5R2-01 / F5R2-02 / F-B1 / F-B2 | RESOLVED — CLOSED. F5R2-01/02 (2 MEDIUM doc-accuracy) fixed by FIX-F5-002 (PR #412 b356545); F-B1 (1 HIGH demo-evidence fabrication) fixed by FIX-F5-003 (PR #413 9eab53f); F-B2 (1 MEDIUM CHANGELOG mitre_techniques) fixed by FIX-F5-004 (PR #415 b36b884). All F5 round findings resolved. | CLOSED (D-468, 2026-07-17) | 2026-07-17 |
| B-001 (PRD v1.56→v1.57 doc nit) | RESOLVED (D-475, 2026-07-18) — CLOSED. BC-2.19.002 v1.2→v1.3 title corrected ("IEC-104 Session Tracking and Frame Validation" → "IEC-104 Session Tracking, Frame Validation, and Threat Detection"); title cascade to VP-044 + BC-INDEX v2.33→v2.34. PRD v1.56→v1.57 (section 4.1 accuracy nit). STORY-167 v1.1 AC propagation (added AC-167-007a + AC-167-007b in STORY-167.md). | CLOSED (D-475, 2026-07-18) | 2026-07-18 |
| B-002 (BC-INDEX title drift) | RESOLVED (D-475, 2026-07-18) — CLOSED. Subsumed by B-001 resolution: BC-INDEX v2.34 title cascade applied, BC-2.19.002 BC-INDEX entry updated to match corrected title. | CLOSED (D-475, 2026-07-18) | 2026-07-18 |
| PG-DEMO-JSON-FABRICATION | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-175 (E-11, draft, wave-TBD, 3 pts). Root cause: hand-written JSON bypasses serde serialization; 3 occurrences in feature-iec104. | CODIFIED→STORY-175 | 2026-07-18 |
| PG-GATE-VOCAB-BLINDSPOT | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-176 (E-11, draft, wave-TBD, 2 pts). Root cause: stub-era vocabulary surviving into green deliveries. | CODIFIED→STORY-176 | 2026-07-18 |
| PG-DOC-CURRENCY-SWEEP | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-176. Root cause: missing mandatory pre-adversary doc sweep. | CODIFIED→STORY-176 | 2026-07-18 |
| PG-ADVERSARY-SEVERITY-CALIBRATION | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-176. Root cause: adversary raised severity on code-frozen surfaces without re-confirming frozen status. | CODIFIED→STORY-176 | 2026-07-18 |
| PG-MERGE-AUTH-SUBAGENT-CLASSIFIER | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-177 (E-11, draft, wave-TBD, 2 pts). Reconfirmed at PR #419 (2026-07-18): step-8 halt, human-direct merge required. | CODIFIED→STORY-177 | 2026-07-18 |
| PG-ADVERSARY-IDLE-NO-REPORT | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-177 (agent-generic, not adversary-specific; fresh occurrence 2026-07-18 spec-steward dispatch). | CODIFIED→STORY-177 | 2026-07-18 |
| F3-DECOMPOSITION-BC-FIDELITY | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-178 (E-11, draft, wave-TBD, 3 pts). 4 distinct BC-fidelity failures in feature-iec104. | CODIFIED→STORY-178 | 2026-07-18 |
| PG-SPEC-VERSION-CITATION-CURRENCY | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-178. Root cause: spec-version bumps did not propagate simultaneously to src/ comments and CHANGELOG. | CODIFIED→STORY-178 | 2026-07-18 |
| PG-STATE-RECOVERY-SCOPE + PG-VERIFY-ALL-WORKTREES | CODIFIED (D-475, 2026-07-18) — REMOVED from carry-forwards. Codified into STORY-179 (E-11, draft, wave-TBD, 2 pts). Root event: stray commit 105497f to main develop checkout during F4 STORY-171 delivery. | CODIFIED→STORY-179 | 2026-07-18 |
| STORY-164/165 input-hash staleness | RESOLVED BENIGN (D-475, 2026-07-18) — CLOSED. Re-baselined using canonical Python tool `bin/compute-input-hash --write`; `--scan` confirmed 132/0 STALE. No spec drift — expected re-baseline after live-state inputs settled. | CLOSED BENIGN (D-475, 2026-07-18) | 2026-07-18 |
| DRIFT-SPRINT-STATE-FIELD-FORM-001 | PRE-RESOLVED (D-475, 2026-07-18) — CLOSED. sprint-state.yaml absent from all worktrees; field-form enforcement constraint already retired. Drift item removed from STATE.md. | CLOSED (D-475, 2026-07-18) | 2026-07-18 |
