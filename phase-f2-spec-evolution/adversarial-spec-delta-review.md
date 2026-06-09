---
document_type: adversarial-spec-delta-review
feature: issue-7-modbus-tcp-analyzer
phase: F2
version: "2.0"
status: CONVERGED_REVISION
produced_by: state-manager (on behalf of orchestrator)
date: 2026-06-09
rounds_initial: 3
rounds_revision: 3_claude_plus_gemini_hybrid
adversaries:
  - Claude adversary agent
  - consistency-validator (parallel, round 1)
  - Gemini cross-model hybrid (2 slices — design + BCs, revision round)
verdict: CONVERGED_REVISION
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

**VERDICT: CONVERGED (initial). Feature #7 F2 spec evolution complete (3-round Claude).**
**Total findings across all rounds: 19 + 4 + 1 = 24, all resolved.**

---

## F2 Revision Round — Research-Validated Design Changes + Multi-Pass Re-Convergence

### Trigger

After initial F2 convergence (D-033), human requested research-agent validation of 3 design decisions before proceeding to F3.

### Research-Agent Validation (DF-VALIDATION-001)

Research-agent validated 3 design decisions. Human adopted all 3:

1. **Dual-window write threshold** (adopted over single-window): burst >20/1s OR sustained >10/s over >=2s window; microsecond-scale truncation-free math (integer microseconds, wrapping_sub for wrap-safety); 2 CLI flags (`--modbus-write-threshold-burst` and `--modbus-write-threshold-sustained`). Replaces the single `--modbus-write-threshold` flag from initial F2.

2. **T0846 → T0888 recon correctness fix**: T0846 was misassigned. Correct MITRE ICS technique for read-coil/register reconnaissance is T0888 Remote System Information Discovery. Updated across all BCs and indexes. SEEDED 21 / EMITTED 13 (unchanged emission count; seeded count +1 from T0846 recon fixture correction).

3. **Full multi-tag Finding** (breaking schema change): `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>`. ADR-006 authored (multi-technique finding attribution). Breaking JSON/CSV schema change — feature now targets v0.3.0. Cross-cuts SS-09/10/11/14 + all analyzers + reporters. ~28 BCs revised.

### Revision Applied — Affected Subsystems

| Subsystem | BCs Revised | Nature |
|-----------|-------------|--------|
| SS-14 (Modbus/ICS) | BC-2.14.001, 003, 004, 011, 013, 014, 015, 016, 017, 018, 019, 020, 022, 024 (14 BCs) | dual-window thresholds + T0888 fix + multi-tag schema |
| SS-09 (Findings) | BC-2.09.001, 006 | mitre_techniques Vec migration |
| SS-10 (Reporters) | BC-2.10.005, 007, 008 | CSV/JSON schema multi-tag output |
| SS-11 (Adversarial) | BC-2.11.013, 015, 017, 020, 024 | multi-tag expectation in test fixtures |
| ADR-006 | new | multi-technique finding attribution rationale |

**Total revised BCs: ~28 across 4 subsystems. BC-INDEX, VP-007, VP-INDEX, ARCH-INDEX, prd.md, prd-delta.md, spec-changelog.md, architecture-delta.md, f2-fix-directives.md, verification-delta.md all updated.**

---

## F2 Revision Adversarial Round 1 (Claude)

**Adversary:** Claude adversary (primary).

**CRITICAL (3):**

| ID | Finding | Resolution |
|----|---------|------------|
| F-REV-CRIT-001 | `Verdict` enum not migrated — BC-2.09.001 updated `mitre_technique` → `mitre_techniques` but left `Verdict` enum definition referencing the old single-tag field in its invariant prose | Fixed: BC-2.09.001 Verdict enum invariant updated; all `verdict.mitre_technique` → `verdict.mitre_techniques` references swept |
| F-REV-CRIT-002 | Architecture-delta.md stale — architecture-delta.md still described the original single-window threshold design; dual-window threshold architecture not reflected | Fixed: architecture-delta.md §Threshold-Architecture section rewritten for dual-window |
| F-REV-CRIT-003 | T0831 detection model broken by dual-window migration — T0831 (Manipulation of Control) in BC-2.14.015 referenced the old `write_burst_count` single field; dual-window adds `write_burst_1s_count` and `write_sustained_count` but BC-2.14.015 body still joined against removed field | Fixed: BC-2.14.015 updated to reference dual-window state fields correctly |

**Companion propagation (2 MED):**

| ID | Finding | Resolution |
|----|---------|------------|
| F-REV-MED-001 | BC-2.10.007 (JSON reporter) used `mitre_technique` key name in annotated output example after multi-tag migration | Fixed: BC-2.10.007 example JSON updated to `mitre_techniques` array |
| F-REV-MED-002 | BC-2.11.020 test-fixture expectation still asserted single-string `mitre_technique` equality | Fixed: BC-2.11.020 assertion updated to Vec membership check |

**Round total:** 3 CRIT + 2 MED = 5 findings. All resolved.

---

## F2 Revision Round 2 (Claude)

**Adversary:** Claude adversary (primary).

0 new findings. All R1 fixes confirmed clean. **CONVERGED (Claude).**

---

## F2 Revision Round 3 (Claude)

**Adversary:** Claude adversary (primary). Confirmation pass.

0 findings. **CONVERGED (Claude, 3/3 clean).**

---

## Gemini Cross-Model Hybrid Pass (2 Slices — Blind to Claude Findings)

**Model:** Gemini (cross-family; genuine non-Claude model diversity). **Date:** 2026-06-09.

Two review slices dispatched independently: (1) design/architecture slice, (2) BC corpus slice. Blind to Claude's revision findings.

### Slice 1: Design / Architecture

**HIGH findings:**

| ID | Finding | Resolution |
|----|---------|------------|
| G-HIGH-001 | Sustained-rate integer-truncation bias — dual-window sustained-rate check computed `write_sustained_count / elapsed_seconds` using integer division; at 9 writes over 2.0s this truncates to 4/s (below threshold 10/s) but the true rate is 4.5/s; under-trigger on sub-11-writes-per-2s bursts | Fixed: division replaced with microsecond-scale rate math (`write_sustained_count * 1_000_000 / elapsed_us >= threshold_per_s`); no truncation; wrapping_sub used for elapsed_us to be wrap-safe on pcap timestamp wraps |
| G-HIGH-002 | BC-001 frame-bounds off-by-six ADU advance — MBAP PDU max is 253 bytes + 6 bytes header = 259 bytes max ADU; BC-2.14.001 frame-bounds check used `length_field + 6` for total but advance logic advanced by `length_field` only (missing the 6-byte MBAP header), causing `6 + length` double-count advance on malformed short frames | Fixed: advance logic corrected to `6 + length_field`; VP-022 sub-property bounds note updated to reflect correct advance formula |

**MEDIUM findings:**

| ID | Finding | Resolution |
|----|---------|------------|
| G-MED-001 | BC-018/019 not multi-tag-migrated — BC-2.14.018 (T0835 Manipulate I/O Image) and BC-2.14.019 (T0806 Brute Force I/O) still used `mitre_technique: String` in their Finding emission examples; the multi-tag migration (revision trigger #3) was applied to most BCs but missed these two | Fixed: BC-2.14.018 and BC-2.14.019 emission examples updated to `mitre_techniques: Vec<String>` with appropriate tag arrays |
| G-MED-002 | BC-011 attribution-validation + emission-clarity — BC-2.14.011 (transaction-correlation emission) lacked explicit invariant that emitted `mitre_techniques` must be non-empty; an empty Vec would produce a malformed Finding | Fixed: BC-2.14.011 updated with non-empty invariant on mitre_techniques at emission |

### Slice 2: BC Corpus

**LOW findings:**

| ID | Finding | Resolution |
|----|---------|------------|
| G-LOW-001 | Length-gate off-by-one — BC-2.14.001 frame-validation condition used `length_field > 253` (exclusive); MBAP protocol max PDU is 253 bytes, so `> 253` allows a 254-byte PDU (off by one); correct gate is `length_field > 253` → `[2, 254)` which actually should be `length_field >= 254` to reject 254+ | Fixed: gate updated to `length_field > 253` → `length_field >= 254` i.e. valid range is `[2, 254)` per Modbus spec; VP-022 sub-property updated accordingly |
| G-LOW-002 | FC 0x17 (Read/Write Multiple Registers) T0831 evasion gap — FC 0x17 simultaneously reads and writes registers; T0831 detection only listed FC 0x05/0x06/0x0F/0x10; an attacker using FC 0x17 for covert write evades detection | Fixed: BC-2.14.015 T0831 condition extended to include FC 0x17 |
| G-LOW-003 | Bucket-order + CSV-delimiter inconsistency — BC-2.10.008 (CSV reporter) specified `mitre_techniques` as a semicolon-delimited string for multi-tag but BC-2.10.005 (summary reporter) described comma-delimited; inconsistent within same reporter subsystem | Fixed: canonical delimiter standardized to comma (`,`) across BC-2.10.005 and BC-2.10.008; comma chosen per RFC 4180 CSV convention |

**MEDIUM finding:**

| ID | Finding | Resolution |
|----|---------|------------|
| G-MED-003 | u32 timestamp wrap in dual-window sustained-rate — `elapsed_us` computed as `current_ts_us.wrapping_sub(window_start_ts_us)` but BCs described both timestamps as `u64`; `wrapping_sub` on u64 is correct, but BC body described the field type as `u32` in one place (inconsistent with ModbusFlowState 15-field table which used u64) | Fixed: BC-2.14.016 (sustained-rate window tracking) field type corrected to `u64` throughout; wrapping_sub u64 confirmed correct |

### Gemini False Positive

| ID | Finding | Disposition |
|----|---------|------------|
| G-FP-001 | BC-020 not multi-tag-migrated — Gemini flagged BC-2.14.020 as still using `mitre_technique: String` | REFUTED: BC-2.14.020 was correctly migrated in the revision burst; Gemini was reviewing a stale artifact snapshot from before the migration commit; post-fix verification confirmed `mitre_techniques: Vec<String>` present. Consistent with D-023 hybrid-catches-hallucination pattern. |

**Gemini total:** 2 HIGH + 3 MED + 3 LOW = 8 real findings + 1 false positive. All 8 real findings fixed. False positive discarded after verification.

---

## Hybrid Convergence Summary

| Round | Model | Findings | Resolved |
|-------|-------|----------|----------|
| Initial R1 | Claude adversary + consistency-validator | 19 (4C+8H+7M) | 19 |
| Initial R2 | Claude adversary | 4 (1H+3M) | 4 |
| Initial R3 | Claude adversary | 1 (1H) | 1 |
| Revision R1 | Claude adversary | 5 (3C+2M) | 5 |
| Revision R2 | Claude adversary | 0 | — |
| Revision R3 | Claude adversary (confirm) | 0 | — |
| Gemini Hybrid | Gemini (2 slices, blind) | 8 real + 1 FP | 8 (FP discarded) |
| **TOTAL** | | **37 real findings** | **37** |

**Cross-model observation:** Gemini's blind pass caught a distinct defect class (arithmetic precision: integer-truncation bias, off-by-six ADU advance, off-by-one length gate, u32-wrap inconsistency) that 3 Claude rounds missed. This is the 2nd time the Gemini hybrid caught an arithmetic-precision defect class Claude missed (1st occurrence: D-023, fabricated red flags discarded; this is a clean catch). Pattern consistent with complementary model blind-spots on numeric edge cases in detection-math-heavy features.

**FINAL VERDICT: F2 REVISION CONVERGED (Claude + Gemini cross-model hybrid).**
**Total real findings all rounds: 37. All resolved. 1 false positive caught by verification.**
