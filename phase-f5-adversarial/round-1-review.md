---
document_type: f5-adversarial-round-review
producer: adversary via orchestrator
date: 2026-07-17
cycle: feature-iec104
round: 1
reviewed_sha: 7e95f71
base: fedcea4
---

# F5 Scoped Adversarial — feature-iec104 — Round 1
Attestation: develop @ 7e95f71 verified; grep-counts matched (fn test_fix_p4_001=11; direction: None=0).
## BC-Set Completeness Sweep (DF-BC-COMPLETENESS-SWEEP-001)
31 BCs checked (BC-2.19.001..028 + BC-2.05.012 + BC-2.10.010 + BC-2.12.025); 31 with implementation path; 0 without. No P0 BLOCKER. BC-2.19.011 partial (PC-3 unmet → F-01).
## Canonical-Frame Sweep (DF-CANONICAL-FRAME-HOLDOUT-001)
19 framing invariants independently derived from IEC 60870-5-104 and verified byte-exact (start 0x68; LEN [4,253]; I/S/U CF1 bits; U-frame octets 0x07/0x0B/0x13/0x23/0x43/0x83; N(S)=cf1>>1|cf2<<7; N(R)=cf3>>1|cf4<<7; 15-bit gap mask; k=12; ASDU TypeID/VSQ/COT/CASDU-LE/IOA-24bit-LE). ZERO wrong canonical bytes. No DNP3-DIR-bit-class defect.
## Findings
- F-01 [HIGH][spec-fidelity] BC-2.19.011 PC-3 unmet: T0881 STOPDT finding source_ip: None (iec104.rs:390); NO test anywhere asserts source_ip on a real finding (BC+tests share blind spot). Fix: enrich from flow key per DNP3 master_ip pattern + tests.
- F-02 [MEDIUM][convention] All 10 IEC-104 findings discard source_ip+timestamp (let _ = ts; iec104.rs:1148); DNP3/ENIP populate both. JSON parity divergence.
- F-03 [MEDIUM][test-quality] Stale RED-phase prose in test messages: known set (:5890/:5898/:6056/:6109-6113/:6136-6142) CONFIRMED + 4 unlisted siblings found (:5974/:6019/:6048/:6099).
- F-04 [MEDIUM][convention] iec104.rs:1029 false forward-reference "enriched in STORY-173" (never scoped).
- F-05 [MEDIUM][convention] protocols_tests.rs:208 stale count comment (7+23 vs actual 8+22 asserted same-file :569/:841).
## Cleared axes
Regression risk CLEAN (Rule 8 ordering; mechanical sibling-test edits); Security CLEAN (length-guarded slices, caps enforced, fuzz 1.95M execs 0 crashes); Kani non-vacuity PASS; FIX-P4-001 direction tests non-vacuous; MITRE internal consistency sound.
## EXECUTION-REQUIRED resolution (orchestrator)
Live-ATT&CK cross-check satisfied by existing D-439 research (.factory/cycles/feature-iec104/research/f2-mitre-pin-confirmation.md): T0881/T1692.001/T0836/T0814/T0827 all CONFIRMED-AT-v19.1.
## Deferred-item adjudications
BC-2.19.006 VP-044 back-reference: LOW documentation nicety (anchor is VP-047, semantically correct as-is). Mutants-disposition 2.4 wording: LOW, conclusion sound.
## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (5/5) |
| **Median severity** | 3.0 (1 HIGH + 4 MEDIUM) |
| **Trajectory** | 5 (round 1 baseline) |
| **Verdict** | FINDINGS_REMAIN |

## Verdict
CLASSIFICATION: FINDINGS. Triage (orchestrator): ALL FIVE routed to single fix PR FIX-F5-001 (F-01 subset of F-02 enrichment; F-03/04/05 ride along).
