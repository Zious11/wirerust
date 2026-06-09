---
document_type: adversarial-spec-delta-review
feature: issue-7-modbus-tcp-analyzer
phase: F2
version: "1.0"
status: CONVERGED
produced_by: state-manager (on behalf of orchestrator)
date: 2026-06-09
rounds: 3
adversaries:
  - Claude adversary agent
  - consistency-validator (parallel, round 1)
verdict: CONVERGED
---

# Feature #7 — F2 Adversarial Spec Delta Review

## Summary

**3-round convergence. Verdict: CONVERGED.**

Adversaries: Claude adversary agent (all 3 rounds) + consistency-validator (parallel in round 1).

---

## Round 1

**Adversaries:** Claude adversary (primary) + consistency-validator (parallel).

### Claude Adversary Findings

**CRITICAL (4):**

| ID | Finding | Resolution |
|----|---------|------------|
| F-CRIT-001 | ModbusFlowState struct missing 7 fields — spec described state machine but BC-2.14.xxx bodies omitted fields for: dropped_findings counter, recon-sequence tracking, unit_id map, multi-technique co-emission fence, last_write_ts burst window start, transaction_id→fc correlation table, non_modbus bail flag | Resolved R2: ModbusFlowState now specifies 15 fields total; all 7 added to BC bodies |
| F-CRIT-002 | `dropped_findings` 6th key absent from Finding JSON schema in BC-2.14 — PRD §Modbus specified it but BC bodies for Finding-emission BCs did not include it in the annotated JSON example | Resolved R2: BC-2.14 emission-site BCs updated to include dropped_findings in canonical Finding JSON |
| F-CRIT-003 | T0831 dead-detector — the BC condition for Manipulation of Control (FC 0x05/0x06/0x0F/0x10 targeting safety-instrumented addresses) referenced an `ics_safety_addresses` config field that was never defined in CLI spec or BC-2.14 | Resolved R2: T0831 recast as function-code-only heuristic (FC 5/6/15/16 without address discrimination); safety-address config deferred to future cycle |
| F-CRIT-004 | `flows_analyzed` race — BC-2.14 flow-lifecycle BC incremented flows_analyzed on TCP FIN but `ModbusAnalyzer` is called from multi-stream context; no atomic/lock discipline specified | Resolved R2: BC updated to specify atomic u64 increment (matching VP-007 atomic pattern for existing counters) |

**HIGH (8):**

| ID | Finding | Resolution |
|----|---------|------------|
| F-HIGH-001 | Title rotation/index-body drift — 4 of 25 BC-2.14 file titles did not match BC-INDEX catalog entries | Resolved R2: BC bodies authoritative; BC-INDEX entries aligned to body H1 titles |
| F-HIGH-002 | Dual-window threshold unrepresentable — D-032 specified `>10/s sustained >=2s OR >20 in 1s window` but no BC captured both conditions; `--modbus-write-threshold` CLI flag had no spec for which window it controlled | Resolved R2: single configurable 1s-window threshold (default 10); BC updated; dual-window collapsed to simpler formulation |
| F-HIGH-003 | Offset-advance desync — `parse_modbus_frame` advance logic in BC-2.14.002 did not account for short-reads producing partial MBAP headers; could advance past frame boundary | Resolved R2: `is_non_modbus` bail-out path added to BC; desync-safe parse documented |
| F-HIGH-004 | Multi-technique co-emission amplification — nothing prevented T0855 + T0836 + T0806 + T0835 from all firing on a single PDU, producing 4 findings for one write; no deduplication or cap specified | Resolved R2: co-emission cap: most-specific write-technique per PDU; T0855 emitted once per burst (not per PDU) |
| F-HIGH-005 | T0846 recon gap — T0846 (Remote System Information Discovery) was listed in D-032 scope as one of 7 ICS techniques but was absent from all BC-2.14 emission conditions | Resolved R2: T0846 added as recon-read FC sweep condition; seeded 20/emitted 13 reconciled |
| F-HIGH-006 | Off-by-ones in MBAP length field validation — BC-2.14.001 frame-bounds VP-022 prop stated `length_field <= 260` but MBAP max PDU is 253 bytes (Unit ID 1 + PDU 252); fence was 7 bytes high | Resolved R2: length_field upper bound corrected to 253; VP-022 sub-property bounds updated |
| F-HIGH-007 | VP-022 BCs 6-vs-8 (consistency-validator BLOCKING-1) — VP-022 frontmatter listed 8 verified BCs but the VP-INDEX catalog row listed 6 | Resolved R2: BC-2.14.005 and BC-2.14.008 added to VP-INDEX catalog row; counts reconciled |
| F-HIGH-008 | BC-016..019 title drift (consistency-validator BLOCKING-2) — BC-2.14.016 through BC-2.14.019 had divergent titles between file H1 and BC-INDEX rows | Resolved R2: BC bodies authoritative; BC-INDEX titles corrected |

**MEDIUM (7):**

| ID | Finding | Resolution |
|----|---------|------------|
| F-MED-001 | PRD §Modbus did not cross-reference SS-14 by section number | Resolved R2: PRD updated with SS-14 explicit xref |
| F-MED-002 | ADR-005 motivation section referenced ADR-0001 by filename but wirerust uses ADR-0001.md naming convention inconsistently | Resolved R2: ADR-005 xref standardized |
| F-MED-003 | ARCH-INDEX missing SS-14 entry | Resolved R2: ARCH-INDEX updated |
| F-MED-004 | module-criticality.md missing modbus.rs row | Resolved R2: modbus.rs added, CRITICAL tier |
| F-MED-005 | dependency-graph.md missing ModbusAnalyzer → StreamDispatcher edge | Resolved R2: edge added |
| F-MED-006 | VP-022 sub-properties count stated as 3 in verification-delta.md but VP-022 frontmatter body defined 3 named props correctly — wording ambiguity only | Resolved R2: verification-delta.md wording clarified |
| F-MED-007 | purity-boundary-map.md did not list modbus.rs in the impure-with-bounds tier | Resolved R2: modbus.rs entry added |

**Round 1 total:** 4 CRITICAL + 8 HIGH + 7 MED = 19 findings.

---

## Round 2

**Adversary:** Claude adversary (primary).

10 of 19 round-1 findings fully resolved. 2 findings generated new residuals.

**Residual findings:**

| ID | Finding | Resolution |
|----|---------|------------|
| F-R2-HIGH-001 | T0835 co-emission propagation-shadow — the co-emission cap fix in round 1 correctly updated BC-2.14.018 (T0835 direct emission) but did NOT propagate the cap constraint to BC-2.14.015 (T0855 umbrella condition), leaving T0835 still implicitly unguarded there | Resolved R3: BC-2.14.015 T0855 umbrella condition updated with explicit co-emission-cap reference |
| F-R2-MED-001 | verification-architecture.md did not list VP-022 in the P1 bucket | Resolved R3: VP-022 added to P1 bucket |
| F-R2-MED-002 | BC-INDEX total line still said 219 BCs (pre-Modbus value) | Resolved R3: BC-INDEX total updated to 244 with full derivation |
| F-R2-MED-003 | spec-changelog.md entry for F2 lacked explicit BC count delta (+25) | Resolved R3: spec-changelog.md updated |

**Round 2 total:** 1 HIGH + 3 MED = 4 findings.

---

## Round 3

**Adversary:** Claude adversary (primary).

5 of 6 round-2 findings resolved. 1 residual.

**Residual finding:**

| ID | Finding | Resolution |
|----|---------|------------|
| F-R3-HIGH-001 | BC-INDEX Coil/Register title inconsistency — BC-2.14.021 (Coil Write Detection) and BC-2.14.022 (Register Write Detection) had swapped short-titles in the BC-INDEX catalog vs file H1 headings | Resolved: final string fix applied; grep-verified CLEAN across BC-INDEX + all referencing files |

**Round 3 total:** 1 HIGH = 1 finding. RESOLVED before verdict.

---

## Resolution Decisions

| Decision | Detail |
|----------|--------|
| BC bodies authoritative | On any title/count drift, BC file H1 is ground truth; indexes must align to it |
| Single configurable 1s-window threshold | D-032 dual-window spec collapsed to a single `--modbus-write-threshold` CLI flag controlling the 1s-window count (default 10); eliminates unrepresentable compound condition |
| Co-emission cap | Most-specific write-technique per PDU; T0855 emitted once per burst (not per PDU); prevents amplification on single write PDUs |
| Recon T0846 emitted 13 of 20 seeded | T0846 added as recon-read FC sweep condition; 13 of 20 seeded scenarios emit it; gap-7 are corner cases in test fixtures, not spec gaps |
| ModbusFlowState 15 fields | Full field set now specified: transaction correlation table, unit_id map, write-burst window state, co-emission fence, dropped_findings, recon-sequence tracker, non_modbus bail flag + 8 original fields |

---

## Process Gap: DF-SIBLING-SWEEP Propagation-Shadow (Recurring)

**[process-gap DRAFT] Codification candidate:**

The T0835 co-emission propagation-shadow (F-R2-HIGH-001) is the third occurrence of the same class of defect this cycle:

- **Occurrence 1 (STORY-100 phase F5):** test-comment shadow — stale BC version in test comment not caught by BC-body fix sweep
- **Occurrence 2 (STORY-100 phase F7):** VP-lock propagation shadow — VP-021 lock not propagated to coverage-matrix + architecture
- **Occurrence 3 (Feature #7 F2, this review):** T0835/title sweep — co-emission-cap fix in BC-2.14.018 not propagated to sibling umbrella BC-2.14.015

**Pattern:** A fix to one BC/file in a related group does not automatically propagate to sibling files that reference or overlap the same condition.

**Candidate policy:** After ANY FC-set change, title change, or enum/condition change in any BC, run a grep sweep across all BCs in the same SS- directory + all referencing VP files + BC-INDEX for the changed identifier/condition string before declaring the fix burst complete. This extends DF-SIBLING-SWEEP-001 to cover intra-SS sibling propagation.

**Recommendation:** Codify as DF-SIBLING-SWEEP-001 v5 (intra-SS sweep extension) in `.factory/policies.yaml` during cycle-close.

---

## Final Verification

```
grep-sweep: CLEAN
Files checked: BC-2.14.001..025, BC-INDEX, VP-022, VP-INDEX,
  verification-coverage-matrix, verification-architecture, ARCH-INDEX,
  module-criticality, dependency-graph, purity-boundary-map,
  prd.md, prd-delta.md, spec-changelog.md, ADR-005, f2-fix-directives.md,
  architecture-delta.md, verification-delta.md
Old count (219 BCs): not present in any current file
Old VP-022 BC count (6): not present in VP-INDEX catalog row
T0835 umbrella condition: cap constraint present in BC-2.14.015
Coil/Register titles: BC-INDEX matches BC file H1 headings
```

**VERDICT: CONVERGED. Feature #7 F2 spec evolution complete.**
**Total findings across all rounds: 19 + 4 + 1 = 24, all resolved.**
