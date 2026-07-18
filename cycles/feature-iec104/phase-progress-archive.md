---
document_type: phase-progress-archive
level: ops
version: "1.0"
status: archive
producer: state-manager
timestamp: 2026-07-18T04:30:00Z
cycle: "feature-iec104"
traces_to: STATE.md
---

# Phase Progress Archive — feature-iec104

Granular per-wave, per-story adversary, and per-fix-batch Phase Progress rows extracted from
STATE.md on 2026-07-18 to slim the main state document. The compact summary rows (F1..F7,
E2E, PR #407) remain inline in STATE.md.

## Extracted Rows

| Phase | Status | Notes |
|-------|--------|-------|
| feature-iec104 — D-451 spec-remediation burst | **COMPLETE** | SR-172-01 BLOCKING (FlowId→FlowKey); SR-172-02 MEDIUM (carry-overflow discard-all-new); SR-172-03 MEDIUM (malformed-LEN EMIT-WITH-DEDUP, research-validated). 3rd F3-DECOMPOSITION-BC-FIDELITY — CODIFY-NOW. |
| feature-iec104 — F4 per-story adversary (STORY-172) | **CONVERGED (D-454)** | 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6 (BC-5.39.001 SATISFIED). F-172-001/002 HIGH remediated D-452; F-172-201 LOW remediated D-453; F-172-301 NIT remediated fec9bfa. Deferred: F-172-003 LOW STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. |
| feature-iec104 — F4 per-story adversary (STORY-173) | **CONVERGED (D-457/D-458)** | 17 passes total; initial streak P12/P13/P14 (D-457); pre-merge LOW fix burst (3 LOWs fixed; see next row); fresh 3-clean A/B/C (D-458). IEC104-FINDINGS-CAP-001 RESOLVED. BC-2.19.006 v1.2; BC-INDEX v2.33. |
| feature-iec104 — STORY-173 pre-merge LOW fix burst | **COMPLETE** | 3 LOWs FIXED pre-merge (human approved all 3): LOW#1 flows_analyzed real cumulative counter (mirrors ENIP; 0bfc977); LOW#2 packets_analyzed valid-APDU frame counter (mirrors DNP3; 5325cf2); SEC-001/A-173-A-01 is_valid_iec104_frame doc overstated gate role + BC-2.19.006 v1.2; BC-INDEX v2.32→v2.33 (3ec6ac1). 2602→2604 tests. Triggered fresh A/B/C re-convergence. |
| feature-iec104 — F4 per-story adversary (STORY-174) | **CONVERGED (D-462)** | 7 passes; streak P5/P6/P7 (BC-5.39.001 SATISFIED). Trajectory P1(1M)->P2(1M)->P3(NIT)->P4(1M)->P5/P6/P7 CLEAN. F-174-001 VP-044 non-vacuity (Kani 82→89); F-174-002 skeleton/CI-claim prose + 8-site sweep; F-174-P4-001 BC-2.19.025 invariant-2 mis-anchor (e62701f). Story v2.2; STORY-INDEX v3.75. PG-GATE-VOCAB-BLINDSPOT filed. |
| feature-iec104 — wave-79 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P3/P4/P5) on STORY-170 diff == wave-level adversarial; CI 13/13 develop 0bd93f8; D-447 |
| feature-iec104 — wave-80 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P2/P3/P4) on STORY-171 diff == wave-level adversarial; CI 13/13 develop 1a64380; D-448 |
| feature-iec104 — wave-81 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P4/P5/P6) on STORY-172 diff == wave-level adversarial; CI 13/13 develop d64e5fe; D-455 |
| feature-iec104 — wave-82 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean A/B/C (17 total passes) on STORY-173 diff == wave-level adversarial; CI 13/13 develop 084ff93; D-458 |
| feature-iec104 — wave-83 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P5/P6/P7) on STORY-174 diff == wave-level adversarial; CI 13/13 + post-merge develop CI SUCCESS; D-463 |
| feature-iec104 — pre-F5 fix-PR (FIX-P4-001) | **DELIVERED (D-464)** | PR #410 7e95f71; IEC104-FINDING-DIRECTION-001 resolved — all 10 IEC-104 emit sites direction: Some(...); 11 direction-assertion tests (mod fix_p4_001); F5 scoped adversarial UNBLOCKED |
| feature-iec104 — F5 fix batch (FIX-F5-001) | **DELIVERED (D-466)** | PR #411 9c5aa9a; F-01..F-05 resolved; source_ip+timestamp enrichment all 10 emit sites; BC-2.19.011 PC-3 SATISFIED |
| feature-iec104 — F5 fix batch (FIX-F5-002) | **DELIVERED (D-467)** | PR #412 b356545; F5R2 doc-accuracy fixes: provenance corrected (S-139/S-140 lineage), fabricated T0881 JSON corrected, year 2025→2026, CHANGELOG direction-parity wording |
| feature-iec104 — F5 fix batch (FIX-F5-003) | **DELIVERED** | PR #413 9eab53f; F-B1 HIGH resolved — FIX-P4-001 demo-evidence ×4 files corrected: real enum variants, real JSON, MITRE T1692.001; CHANGELOG false-correction claim removed |
| feature-iec104 — F5 fix batch (FIX-F5-004) | **DELIVERED** | PR #415 b36b884; F-B2 MEDIUM resolved — CHANGELOG Example-3 mitre_techniques [] → ["T0814"]; intro "8 function" → "10 total / 8 function" |
