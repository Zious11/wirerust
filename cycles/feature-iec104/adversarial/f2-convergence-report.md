---
document_type: f2-convergence-report
cycle: feature-iec104
phase: F2-spec-evolution
status: CONVERGED
convergence_passes: 12
consecutive_clean_streak: 3
clean_passes: P10/P11/P12
consistency_audits: 12
date: 2026-07-14
decision: D-438
---

# F2 Spec-Evolution Convergence Report — feature-iec104

**Cycle:** feature-iec104 (IEC 60870-5-104 passive analyzer, TCP 2404)
**Phase:** F2 Spec-Evolution
**Status:** CONVERGED — 3 consecutive clean passes (P10/P11/P12)
**Total Passes:** 12 adversarial + 12 fresh-context consistency audits
**Date:** 2026-07-14
**Decision:** D-438

---

## Per-Pass Summary Table

| Pass | Verdict | CRIT | HIGH | MED | LOW/NIT | Key Findings | Resolution Burst |
|------|---------|------|------|-----|---------|--------------|-----------------|
| P1 | FINDINGS | 1 | 3 | 10 | — | C1: fabricated TypeID C_SE_ND_1; control range 45-52→45-51, TypeID 52 reserved. H1: T0809 mis-identity (should be T0881). H2: T1692.001 name wrong ("Exploitation of Remote Services"→"Unauthorized Message: Command Message"). H3: VP-044 over-scope beyond 3 BCs. Consistency audit: 1 CRIT + 9 MAJOR + 2 MINOR. | Research-agent canonical-fact validation triggered. Remediation burst: TypeID range corrected, T0809→T0881, T1692.001 name corrected, VP-044 scope narrowed. |
| P1-research | RESEARCH VALIDATED | — | — | — | — | T0809="Data Destruction" WRONG; correct for STOPDT-abuse intent = T0881 "Service Stop"/IcsInhibitResponseFunction. Control TypeIDs 45-51 confirmed; C_SE_ND_1 nonexistent; C_BO_NA_1=51; TypeID 52 reserved. | Canonical facts anchored across BCs + ADR-0013. |
| P2 | FINDINGS | 1 | 1 | 3 | — | C1: SS-19 shard fully stale — sibling-sweep miss (SS-19 not updated when BCs were amended). H1: VP-044 over-scope 3 BCs. Consistency: 11/12 gaps from P1 closed; 1 residual. | SS-19 shard swept and reconciled. VP-044 scope narrowed to 3 target BCs. |
| P3 | FINDINGS | 0 | 1 | 3 | — | H1: VP-044 in BC-006/015 (scope residual). M1: ADR copy divergence (ADR-0013 docs/adr/ vs .factory/specs/architecture/decisions/ not byte-identical). M2: delta T1692.002 contradiction. M3: VP-045 reciprocity gap. Consistency audit PASS. | VP-044 scope corrected. ADR-0013 mirror synced byte-identical. T1692.002 reconciled. VP-045 reciprocity added. |
| P4 | FINDINGS | 0 | 2 | 2 | — | H1: VP-044 registry forward-progress language (in arch docs). H2: cursor-advance 1-vs-2-byte ambiguity in BCs. M1: C_SE label range 48-51 wording. M2: VP-045 harness names not matching VP-INDEX. | Registry language corrected. Cursor-advance byte width clarified (2-byte T-field). C_SE range prose fixed. VP-045 harness names synced. |
| P5 | FINDINGS | 0 | 2 | 1 | 2 | H1: bad-start carry-clear in BCs vs ADR (BC states carry clears on STARTDT-ACT; ADR states carry clears on timeout — reconciled to: carry clears on either condition). H2: VP-044 loop language in arch docs. M1: SS-19 T0836 label. L1/L2: cosmetic. | Bad-start carry-clear reconciled across BCs + ADR. VP-044 loop language corrected. T0836 label fixed. |
| P6 | FINDINGS | 0 | 1 | 1 | — | H1: BC-2.19.019 title "Set-Point" mislabel (correct: "Step Command"). M1: VP-045 stale PO-obligation TODO. Consistency audit PASS. | BC-2.19.019 title corrected. VP-045 PO-obligation resolved. |
| P7 | FINDINGS | 0 | 0 | 3 | 4 | M1: BC-2.19.005 title sync (title row vs H1 diverged). M2: VP-044 doctrine facet gloss missing. M3: T0831 not-emitted marker absent in BC. L1-L4: cosmetic prose. | Full 32-BC title-sync sweep applied. VP-044 doctrine facet gloss added. T0831 not-emitted marker added. |
| P8 | FINDINGS | 0 | 0 | 2 | 2 | M1: STOPDT-con 0x23 spurious T0881 emission (copy-paste from DNP3/ENIP — STOPDT confirmation does not warrant T0881; only STOPDT request triggers IcsInhibitResponseFunction). M2: SS-19 phantom "VP-045 Sub-B/C" window mis-anchor. L1: vestigial window_start_ts field. L2: cosmetic. Root cause: DNP3/ENIP copy-paste in BC-2.19.* boilerplate. | STOPDT-con T0881 emission removed. SS-19 window anchor corrected. Vestigial field removed. |
| P9 | FINDINGS | 0 | 0 | 1 | 1 | M1: delta-analysis saturating_sub window clause (window counter used wrapping sub in one BC variant). L1: consistency audit found ARCH-INDEX:244 window phrase. Exhaustive repo-wide window grep → all loci swept (delta-analysis + ARCH-INDEX + SS-19 + BCs). | saturating_sub corrected. All window loci swept via repo-wide grep. |
| P10 | CLEAN (streak 1) | 0 | 0 | 0 | 1+1 nit | L1: BC-2.19.024 mid-capture first-frame (N(S)>k=12 may emit Possible-confidence T1692.001 — adjudicated LOW, carry to F3). NIT-1. Consistency audit PASS. | No fixes applied. F3 handoff item recorded. |
| P11 | CLEAN (streak 2) | 0 | 0 | 0 | 1 | L1: research-doc §3.3/§3.5/§3.8 pre-v19 MITRE names lack dated reconciliation note (authoritative set in ADR/BCs correct). Consistency audit PASS. | No fixes applied. Recorded as F3-handoff documentation note. |
| P12 | CLEAN (streak 3 → CONVERGED) | 0 | 0 | 0 | 1 nit | NIT-1: BC-2.19.017 COT described "2-byte little-endian"; correct is "2 octets: cause/P-N/T + originator" (byte-offset extraction already correct). Consistency audit PASS. | No fixes applied. NIT recorded as F3-handoff item (e). CONVERGENCE ACHIEVED. |

**HIGH-count decay trajectory:** 4→1→1→2→2→1→0→0→0→0→0→0

---

## Final Spec State

| Artifact | Version | Delta |
|----------|---------|-------|
| BC-INDEX | v2.28 | +6 versions from v2.22: 30 new BCs (BC-2.19.001-027 + BC-2.05.012 + BC-2.10.010 + BC-2.12.025); 2 amended (BC-2.18.003 v1.5 + BC-2.18.004 v1.3) |
| PRD | v1.56 | +5 versions from v1.51 |
| VP-INDEX | v2.46 | +6 versions from v2.40: VP-044 Kani / VP-045+046 proptest / VP-047 fuzz |
| ARCH-INDEX | v2.16 | +4 versions from v2.12: SS-19 subsystem + ADR-0013 |
| SS-19 shard | v1.5 | New subsystem spec; final version after 4 P1-P9 remediation rounds |
| verification-architecture | v2.33 | Updated for VP-044..047 |
| verification-coverage-matrix | v1.48 | IEC-104 coverage rows added |
| ADR-0013 | v1.0 | New: IEC 60870-5-104 stream dispatch and parser design |

**ADR-0013 mirrors:** docs/adr/ + .factory/specs/architecture/decisions/ — byte-identical (P3 remediation).

**Input-hash audit (canonical Python tool, STALE=0):**
- BC-2.19.* stories: f5a97d3
- Cross-subsystem stories: 8b69772
- BC-2.18.* stories: 84318a1

---

## Dispositioned F3-Handoff Items (5 — non-blocking, adjudicated LOW/NIT)

| ID | Severity | Description | F3 Decision |
|----|----------|-------------|-------------|
| F3-H-001 | LOW | BC-2.19.024 mid-capture first-frame: N(S)>k=12 may emit Possible-confidence T1692.001 — this can fire on first packet of a capture if sequence counter starts high. | F3 decides: add first_frame_seen guard vs accept as MVP behavior. |
| F3-H-002 | LOW | BC-2.19.023 Description prose on N(S) bit layout garbled (extraction formula correct). | F3 prose cleanup during implementation (no BC logic change). |
| F3-H-003 | LOW | BC-2.10.010 VP table should also cite verify_all_emitted_ids_resolve for the EMITTED postcondition. | F3 story-writer to add VP table annotation. |
| F3-H-004 | LOW | feature-iec104-research.md §3.3/§3.5/§3.8 pre-v19-remap MITRE names lack dated reconciliation note (authoritative set in ADR-0013 + BCs is correct). | F3-handoff doc cleanup: add "pre-v19, authoritative per ADR-0013 Decision X" annotation. |
| F3-H-005 | NIT | BC-2.19.017 COT described "2-byte little-endian"; correct is "2 octets: cause/P-N/T + originator" (byte-offset extraction already correct in BCs). | NIT: prose clarification only; extraction logic unaffected. F3 prose cleanup. |

---

## F3 Code Obligations (4 — documented, not defects)

These are new code obligations discovered during F2 spec-evolution, not defects in existing code.
IEC-104 is a new analyzer; there is no existing implementation to compare against.

| ID | Obligation | Source | Target File |
|----|-----------|--------|-------------|
| F3-C-001 | T0881 six-part atomic in `src/mitre.rs`: SEEDED counter 28→29, EMITTED counter 20→21, per BC-2.10.010 / ADR-0013 Decision 10. | BC-2.10.010 + ADR-0013 | src/mitre.rs |
| F3-C-002 | Port 2404 in `src/protocols.rs` SUPPORTED_PORTS (BC-2.18.003). | BC-2.18.003 v1.5 | src/protocols.rs |
| F3-C-003 | `DispatchTarget::Iec104` + `classify()` Rule 8 (BC-2.05.012). | BC-2.05.012 | src/dispatcher.rs |
| F3-C-004 | New analyzer module `src/analyzer/iec104.rs` — full IEC 60870-5-104 APDU parser + T1692.001 / T0881 / T0836 / T0831 detection. | BC-2.19.001-027 | src/analyzer/iec104.rs |

---

## Process-Gap Notes (S-7.02 cycle-close)

These are process observations for the cycle-close retrospective. They do not block F3.

**(i) DF-SIBLING-SWEEP-001 under-swept (repeated):** 8 remediation bursts each left 1-2 sibling loci (VP-044 scope across BCs + registry docs; window machinery across shard/ADR/ARCH-INDEX/delta/research). A mechanized cross-artifact sweep (grep-driven) at the first remediation would have collapsed passes P2-P9 into fewer rounds. The lesson from feature-protocol-coverage (PG-F5-RECONCILE-INCOMPLETE-001) was not consistently applied.

**(ii) PG-VP-CONSISTENCY-HOOK-FP + PG-VP-SUMMARY-HOOK-FP:** The validate-vp-consistency hook fires false-positives on "Summary" substring in the modified-log YAML block — analogous to PG-HASH-HOOK-DIVERGENCE. Both hook errors are advisory-only until the plugin is reconciled. Do not treat hook alarms on the "Summary" key as spec defects.

**(iii) Research-doc same-file reconciliation-sweep gap (P11-L1):** After every MITRE name reconciliation burst, the feature research doc (feature-iec104-research.md) requires an explicit reconciliation-note sweep to ensure pre-v19-remap names carry dated clarifications. Add research-doc to the post-reconciliation sibling checklist.

---

## Pre-Existing Drift Item (out of feature scope)

**VP-039/BC-2.07.038 TLS drift (discovered by architect during F2 review):**
VP-INDEX carries stale present-tense "PO must add BC-2.07.038 postcondition/EC + Red-Gate test name" TODOs for VP-039 (TLS reassembly subsystem). These TODOs belong to the SS-07 (TLS) subsystem owner, not the IEC-104 team. Recorded in STATE.md Drift Items as DRIFT-VP039-BC207038-TLS-TODO-001.

---

## Consistency Audit Summary

All 12 fresh-context consistency audits were run in parallel with the adversarial passes:
- P1 audit: 1 CRIT + 9 MAJOR + 2 MINOR found → remediated in P1 burst
- P2-P9 audits: progressive gaps closed; PASS by P3 audit
- P10/P11/P12 audits: all PASS

Final audit state: PASS (zero blocking findings; all 30 new BCs + 4 VPs consistent with ARCH-INDEX SS-19 + ADR-0013 + PRD v1.56).
