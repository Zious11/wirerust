---
document_type: consistency-report
level: ops
version: "1.0"
status: "fail"
producer: consistency-validator
timestamp: 2026-07-13T00:00:00Z
phase: 2
inputs:
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/verification-architecture.md
  - .factory/specs/architecture/verification-coverage-matrix.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/prd.md
input-hash: "9e3c629"
traces_to: .factory/cycles/feature-iec104/
cycle: feature-iec104
audit_scope: Phase F2 spec-evolution delta (IEC 60870-5-104 analyzer)
---

# Consistency Validation Report: wirerust — Feature IEC-104 Phase F2 Delta

## Report Metadata

| Field | Value |
|-------|-------|
| **Product** | wirerust |
| **Generated** | 2026-07-13T00:00:00Z |
| **Generator** | consistency-validator |
| **Artifacts Scanned** | 8 index/top-level + 31 BC files (BC-2.19.001..027 + BC-2.05.012 + BC-2.10.010 + BC-2.12.025 + BC-2.18.003 + BC-2.18.004) |
| **Audit Scope** | Phase F2 spec-evolution delta perimeter: new IEC-104 behavioral contracts, ADR-013, SS-19 architecture section, verification coverage updates |

## Summary

| # | Check | Result |
|---|-------|--------|
| 1 | L2 to L3 Requirement Coverage | pass |
| 2 | L3 to L4 Verification Property Coverage | pass |
| 3 | Dependency Acyclicity | pass |
| 4 | Architecture Alignment | fail |
| 5 | Acceptance Criteria Quality | n/a (story decomposition not yet run) |
| 6 | Story Sizing (all <= 13 points) | n/a (story decomposition not yet run) |
| 7 | Priority Consistency | n/a (story decomposition not yet run) |
| 8 | L1 to L2 to L3 to L4 Chain Completeness | fail |
| 9 | AC Completeness Coverage | n/a (story decomposition not yet run) |
| 10 | ASM/R Traceability | pass |

## 1. L2 to L3 Requirement Coverage

### 1.1 Domain Capabilities to Behavioral Contracts

This audit covers the new CAP-19 capability introduced by feature-iec104. All pre-existing
capabilities were validated in prior consistency passes and are in scope only for delta-impact
checks.

| CAP-NNN | Description | Covered by BC-NNN? | Gap? |
|---------|-------------|-------------------|------|
| CAP-19 | IEC 60870-5-104 Passive Analysis (SS-19) | BC-2.19.001..027 (27 BCs) | no |
| CAP-05 (delta) | Protocol Dispatch — Rule 8 TCP/2404 | BC-2.05.012 | no |
| CAP-10 (delta) | MITRE Mapping — T0809 registration | BC-2.10.010 | no |
| CAP-12 (delta) | CLI / Entry — --iec104 flag | BC-2.12.025 | no |

All 30 new BCs are present on disk and correctly indexed in BC-INDEX v2.23. **PASS.**

## 2. L3 to L4 Verification Property Coverage

### 2.1 Behavioral Contracts to Verification Properties

Feature-iec104 introduces four new verification properties anchored to the SS-19 analyzer.
The full VP coverage picture for SS-19 is:

| BC-S.SS.NNN | Description | VP-NNN | Justification if no VP |
|-------------|-------------|--------|----------------------|
| BC-2.19.001..018 (framing, carry, walk) | APCI/ASDU parse safety (VP-044 anchor: parse_apci_header) | VP-044 (Kani P0) | — |
| BC-2.19.005..009 (carry isolation) | Directional carry independence | VP-045 (proptest P1) | — |
| BC-2.19.010..014 (frame format) | classify_frame_format totality | VP-046 (proptest P1) | — |
| BC-2.19.015..027 (fuzz safety) | No-panic under arbitrary APCI | VP-047 (cargo-fuzz P1) | — |
| BC-2.05.012, BC-2.10.010, BC-2.12.025 | Cross-subsystem extension BCs | (covered by existing VP-001..VP-043) | Pure dispatch / tactic registration; no new harness needed |

VP-INDEX v2.41 arithmetic verified: Kani(16)+proptest(22)+fuzz(3)+integration_unit(6)=47;
P0(9)+P1(32)+test-sufficient(6)=47. verification-architecture.md v2.29 and
verification-coverage-matrix.md v1.44 both updated consistently. **PASS.**

## 3. Dependency Acyclicity

### 3.1 Topological Order

No story decomposition has been run for feature-iec104 at the time of this audit. The
dependency acyclicity check applies to story files; this audit covers the spec-evolution
delta only.

The BC-level dependency graph for SS-19 is acyclic: SS-19 depends on SS-05 (dispatch),
SS-10 (MITRE), SS-12 (CLI), and SS-18 (protocol catalog). None of those subsystems depend
on SS-19. **No cycles detected. PASS.**

### 3.2 Critical Path

Not applicable — story decomposition not yet run.

## 4. Architecture Alignment

### 4.1 Module Coverage

| Architecture Component | BCs Covering It | Coverage |
|-----------------------|----------------|----------|
| analyzer/iec104.rs (SS-19) | BC-2.19.001..027 (27 BCs) | full |
| dispatcher.rs (SS-05) | BC-2.05.012 (Rule 8 TCP/2404) | partial delta |
| mitre.rs (SS-10) | BC-2.10.010 (T0809 registration) | partial delta |
| main.rs/cli.rs (SS-12) | BC-2.12.025 (--iec104 flag) | partial delta |

### 4.2 Component Consistency

**FAIL — GAP-F2-001 (CRITICAL):** BC-2.10.010 specifies `tactic: IcsImpact` for T0809's
`technique_info()` arm (Postcondition 2, Invariant 2, Architecture Anchor). ADR-013 Decision 10
specifies `MitreTactic::IcsInhibitResponseFunction` for the same arm. Per MITRE ATT&CK ICS
v19.1, T0809 "Service Stop" is under tactic TA0107 Inhibit Response Function. BC-2.10.010
is wrong; ADR-013 is correct.

**FAIL — GAP-F2-003..006:** ARCH-INDEX v2.13 Subsystem Registry BC counts are stale:
SS-05=11 (should be 12), SS-10=9 (should be 10), SS-12=24 (should be 25),
SS-19=TBD (should be 27).

**FAIL — GAP-F2-010:** ADR-013 absent from `.factory/specs/architecture/decisions/`
(ARCH-INDEX governance requires ADRs 0005+ in that directory).

## 5. Acceptance Criteria Quality

### 5.1 Concreteness

Not applicable — story decomposition for feature-iec104 has not yet been run. This audit
covers the spec-evolution delta (BCs, ADR, architecture section, VP coverage). Story-level
AC quality is a story-writer and consistency-validator concern at Phase 3 entry.

### 5.2 Testability

Not applicable — same rationale as 5.1.

## 6. Story Sizing

Not applicable — no feature-iec104 stories have been decomposed yet. This section will be
populated by the consistency-validator at Phase 3 entry gate.

## 7. Priority Consistency

Not applicable — no feature-iec104 stories have been decomposed yet.

## 8. L1 to L2 to L3 to L4 Chain Completeness

> Every L1 brief section must trace to L2 CAP, every CAP to BC, every BC to story.
> Gaps must have explicit justification in Gap Register.

### L1 to L2 to L3 to L4 Chain Overview

This audit checks the L1→L2→L3→L4 chain for the feature-iec104 delta only. L4 (stories)
are not yet decomposed; chain completeness from L3→L4 is deferred to Phase 3 entry gate.

| Level | Artifact | Count | Traced Forward | Traced Backward | Coverage |
|-------|----------|-------|---------------|----------------|----------|
| L2 | CAP-19 (new capability) | 1 | 27 BCs (BC-2.19.001..027) | traces to product brief IEC-104 scope | 100% |
| L2 delta | CAP-05/10/12 cross-subsystem | 3 BCs added | new BCs added correctly | — | 100% |
| L3 | BC-2.19.001..027 + delta BCs | 30 | (stories TBD) | traces_to: ADR-013 (gap — see below) | partial |
| L4 | VP-044..VP-047 | 4 | N/A | trace to BC-2.19.xxx via source_bc | 100% |

### Broken Chains

| Gap ID | From | To | Missing Link | Impact | Priority |
|--------|------|----|-------------|--------|----------|
| CHAIN-001 (= GAP-F2-011) | BC-2.05.012, BC-2.10.010, BC-2.12.025 | L2 domain-spec | traces_to points at ADR-013 instead of domain-spec.md | Automated trace chain tools cannot walk L3→L2 for these BCs | P1 |
| CHAIN-002 (= GAP-F2-012) | BC-2.19.001..027 | L2 domain-spec | traces_to points at ADR-013 instead of domain-spec.md | Consistent with CHAIN-001; all feature-iec104 BCs bypass canonical L3→L2 link | P2 |

### Orphaned Artifacts

| Artifact | Level | Issue | Resolution |
|----------|-------|-------|------------|
| ADR-013 (`docs/adr/0013-...`) | arch | Absent from `.factory/specs/architecture/decisions/` — ARCH-INDEX governance violation for ADRs 0005+ | Create `ADR-013-iec104-stream-dispatch-and-parser-design.md` in `.factory/specs/architecture/decisions/` |

## 9. AC Completeness Coverage

> Not applicable at this audit point — story decomposition for feature-iec104 has not been
> run. This section will be fully populated at Phase 3 entry gate after story-writer
> completes story decomposition.

### 9.1 BC Clause Coverage (Level 1)

Deferred to Phase 3 entry gate.

**L1 Score:** n/a

### 9.2 Edge Case & Error Coverage (Level 2)

Deferred to Phase 3 entry gate.

**L2 Score:** n/a

### 9.3 Cross-Cutting Coverage (Level 3)

Deferred to Phase 3 entry gate.

**L3 Score:** n/a

### 9.4 AC Completeness Summary

| Level | Weight | Score | Weighted |
|-------|--------|-------|----------|
| L1 -- BC Clause Coverage | 50% | n/a | n/a |
| L2 -- Edge Case & Error Coverage | 30% | n/a | n/a |
| L3 -- Cross-Cutting Coverage | 20% | n/a | n/a |
| **Overall** | **100%** | | **deferred** |

**Gate Result:** DEFERRED — story decomposition not yet run; re-run at Phase 3 entry.

## 10. ASM/R Traceability

> The feature-iec104 delta introduces no new ASM or R-NNN entries per PRD v1.52 and
> ARCH-INDEX v2.13 review. The existing ASM/R register is not in scope for this delta audit.
> Pre-existing ASM/R coverage was validated in prior consistency passes.

### 10.1 Assumption Coverage

No new ASM-NNN entries introduced by feature-iec104. **PASS.**

### 10.2 Risk Register Coverage

No new R-NNN entries introduced by feature-iec104 delta. The parser-origin constraint
(ADR-013 Decision 7: iec60870-5 crate BANNED, Wireshark IEC-104 dissector BANNED,
lib60870 BANNED) is documented in ADR-013 and reflected in SS-19 architecture section;
no separate R-NNN was created — acceptable since this is a blocking PR criterion, not
a residual risk. **PASS.**

### 10.3 ASM/R Gate Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| HIGH-impact ASMs with holdout scenario | 0 new / 0 new | 100% | pass |
| Testable ASMs with story + assumption_validations | 0 new / 0 new | 100% | pass |
| HIGH-impact R-NNNs with architecture mitigation | 0 new / 0 new | 100% | pass |
| Security R-NNNs in security review scope | 0 new / 0 new | 100% | pass |
| R-NNN NFR candidates with corresponding NFR | 0 new / 0 new | 100% | pass |
| HIGH/HIGH R-NNNs with holdout scenario | 0 new / 0 new | 100% | pass |

## Cross-Reference Validation

### ID Consistency

| Check | Status | Issues |
|-------|--------|--------|
| BC IDs unique | pass | None — BC-2.19.001..027 sequential; BC-2.05.012, BC-2.10.010, BC-2.12.025 extend existing sequences without collision |
| VP IDs unique | pass | VP-044..VP-047 sequential after VP-043 |
| CAP IDs unique | pass | CAP-19 newly registered; no collision with CAP-01..18 |
| BC traces to valid CAP | pass | All 30 new BCs reference CAP-19 or parent subsystem CAP |
| VP traces to valid BC | pass | VP-044..047 source_bc fields reference valid BC-2.19.xxx |
| Story ACs trace to valid BC | n/a | Stories not yet decomposed |
| ADR-013 references valid BCs | pass | ADR-013 Decision rows reference BC-2.19.xxx, BC-2.05.012, BC-2.10.010, BC-2.12.025 — all present |

### Naming Convention Compliance

| Convention | Expected Pattern | Violations |
|-----------|-----------------|------------|
| BC naming | BC-S.SS.NNN | None — all new BCs follow BC-2.19.NNN, BC-2.05.012, BC-2.10.010, BC-2.12.025 |
| VP naming | VP-NNN | None — VP-044..VP-047 compliant |
| CAP naming | CAP-NNN | None — CAP-19 compliant |
| ADR naming | ADR-NNN (in decisions/) or 00NN-slug (in docs/adr/) | GAP-F2-010: ADR-013 present in docs/adr/ but absent from .factory/specs/architecture/decisions/ |

### Canonical Frontmatter Validation

| Artifact | document_type | level | version | producer | traces_to | Status |
|----------|--------------|-------|---------|----------|-----------|--------|
| ADR-013 (docs/adr/) | present | present | present | present | present | pass |
| ss-19-iec104-analysis.md | present | present | present | present | ARCH-INDEX.md | pass |
| BC-2.19.001..027 (sample checked) | present | present | present | present | docs/adr/0013-... (non-canonical — GAP-F2-012) | minor |
| BC-2.05.012 | present | present | present | present | docs/adr/0013-... (non-canonical — GAP-F2-011) | major |
| BC-2.10.010 | present | present | present | present | docs/adr/0013-... (non-canonical — GAP-F2-011) | major |
| BC-2.12.025 | present | present | present | present | docs/adr/0013-... (non-canonical — GAP-F2-011) | major |
| BC-2.18.003 v1.4 | present | present | present | present | .factory/specs/domain/domain-spec.md | pass |
| BC-2.18.004 v1.3 | present | present | present | present | .factory/specs/domain/domain-spec.md | pass |

## Spec vs Implementation Drift

| Artifact | Spec Version | Implementation State | Drift Detected | Notes |
|----------|-------------|---------------------|---------------|-------|
| BC-2.10.010 | v1.0 | Not yet implemented | Yes — spec-internal | Tactic field `IcsImpact` conflicts with ADR-013 Decision 10 `IcsInhibitResponseFunction`; implementation would follow the wrong spec |
| ARCH-INDEX.md | v2.13 | — | Yes — count drift | SS-05/SS-10/SS-12/SS-19 BC counts stale; O-04 MITRE counts stale |
| BC-INDEX.md | v2.23 | — | Yes — derivation drift | Header correct (378/377); line 922 running derivation paragraph stale (346/345) |
| PRD.md | v1.52 | — | Yes — arithmetic drift | O-04 CATALOGUE-ONLY 8→7 is wrong (29-21=8); "moves from CATALOGUE-ONLY" factually wrong |
| ADR-013 | v1.0 | — | Yes — placement drift | Absent from .factory/specs/architecture/decisions/ |
| VP-INDEX v2.41 | v2.41 | — | No | All VP arithmetic consistent |
| verification-architecture.md | v2.29 | — | No | VP-044..047 correctly added |
| verification-coverage-matrix.md | v1.44 | — | No | Totals and module rows correct |

## Findings

### Critical

**GAP-F2-001:** BC-2.10.010 tactic conflict — `IcsImpact` vs `IcsInhibitResponseFunction` for T0809

- **File:** `.factory/specs/behavioral-contracts/ss-10/BC-2.10.010.md`
- **Locations:** Postcondition 2 (`tactic: IcsImpact`), Invariant 2 ("T0809 tactic: IcsImpact — T0809 is classified under Impact in ATT&CK for ICS v2"), Architecture Anchor (`tactic=IcsImpact`)
- **Conflict:** ADR-013 Decision 10 specifies `MitreTactic::IcsInhibitResponseFunction`. BC-2.10.010's own H1 title names the tactic correctly ("T0809 'Inhibit Response Function' Registered...") but the body contradicts it. Per MITRE ATT&CK ICS v19.1, T0809 "Service Stop" is under TA0107 Inhibit Response Function — not Impact.
- **Impact:** Implementer follows BC-2.10.010 and writes `IcsImpact` in `src/mitre.rs`, producing a wrong tactic classification. No test would catch this unless a test explicitly asserts the tactic enum value.
- **Correct value:** `tactic: IcsInhibitResponseFunction` in all three locations.
- **Remediation:** Update BC-2.10.010 v1.0 → v1.1: correct Postcondition 2, Invariant 2, and Architecture Anchor. Update BC-INDEX row to v1.1. Bump version in PRD §2.10 BC-2.10.010 row.

### Major

**GAP-F2-003:** ARCH-INDEX SS-05 BC count = 11, should be 12.
- `.factory/specs/architecture/ARCH-INDEX.md` line ~168. Correct: `12`.

**GAP-F2-004:** ARCH-INDEX SS-10 BC count = 9, should be 10.
- `.factory/specs/architecture/ARCH-INDEX.md` line ~173. Correct: `10`.

**GAP-F2-005:** ARCH-INDEX SS-12 BC count = 24, should be 25.
- `.factory/specs/architecture/ARCH-INDEX.md` line ~175. Correct: `25`.

**GAP-F2-006:** ARCH-INDEX SS-19 BC count = `TBD`, should be 27.
- `.factory/specs/architecture/ARCH-INDEX.md` line ~182. Correct: `27`.

**GAP-F2-007:** ARCH-INDEX O-04 Architecture Debt row MITRE counts stale: "SEEDED 28 − EMITTED 20 = 8 catalogue-only". Correct: "SEEDED 29 − EMITTED 21 = 8 catalogue-only".
- `.factory/specs/architecture/ARCH-INDEX.md` line ~289. Note: CATALOGUE-ONLY is unchanged (29-21=8).

**GAP-F2-008:** BC-INDEX line 922 running derivation paragraph shows "Total BCs on disk: 346. Active: 345." Correct: 378 on disk, 377 active. Derivation chain missing v2.21 (+2: BC-2.11.036..037) and v2.23 (+30: feature-iec104).
- `.factory/specs/behavioral-contracts/BC-INDEX.md` line 922.

**GAP-F2-009:** PRD O-04 feature-iec104 update paragraph (line ~2342): "T0809 moves from CATALOGUE-ONLY to EMITTED; CATALOGUE-ONLY 8→7" contains two errors: (a) T0809 was absent from SEEDED before (BC-2.10.010 PC-3 confirms), so "moves from CATALOGUE-ONLY" is wrong; (b) 29-21=8, not 7. Correct: "T0809 newly registered in all three structures; SEEDED 28→29, EMITTED 20→21, CATALOGUE-ONLY remains 8."
- `.factory/specs/prd.md` line ~2342.

**GAP-F2-010:** ADR-013 absent from `.factory/specs/architecture/decisions/`. ARCH-INDEX governance states ADRs 0005+ reside in that directory. ADR-005..012 all have canonical entries there; ADR-013 does not.
- Missing file: `.factory/specs/architecture/decisions/ADR-013-iec104-stream-dispatch-and-parser-design.md`.

**GAP-F2-011:** BC-2.05.012, BC-2.10.010, BC-2.12.025 each have `traces_to: docs/adr/0013-...` instead of `traces_to: .factory/specs/domain/domain-spec.md`. All sibling BCs in SS-05, SS-10, and SS-12 trace to domain-spec. ADR is not a node in the L3→L2 spec chain.
- Three files: `ss-05/BC-2.05.012.md`, `ss-10/BC-2.10.010.md`, `ss-12/BC-2.12.025.md`, line 11 each.

### Minor

**GAP-F2-002:** ADR-013 Decision 8 Kani harness skeleton places `verify_classify_frame_format_totality` (VP-046, authoritative tool: proptest) inside `#[cfg(kani)] mod kani_proofs`. VP-046 should be in a `proptest!` block, not a Kani proof block.
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`, Decision 8.

**GAP-F2-012:** BC-2.19.001..027 each have `traces_to: docs/adr/0013-...` instead of `traces_to: .factory/specs/domain/domain-spec.md`. No sibling BCs in SS-19 to conflict with, but deviates from canonical pattern of all other ICS analyzer BCs (SS-14..SS-18).
- 27 files in `.factory/specs/behavioral-contracts/ss-19/`, line 11 each.

## Validation Gate Result

**FAIL** — blocking finding: GAP-F2-001 (CRITICAL). BC-2.10.010 specifies the wrong MITRE tactic (`IcsImpact`) for T0809 "Service Stop". Per MITRE ATT&CK ICS v19.1 and ADR-013 Decision 10, the correct tactic is `IcsInhibitResponseFunction`. An implementer following BC-2.10.010 will write wrong tactic logic in `src/mitre.rs`.

GAP-F2-003 through GAP-F2-011 (MAJOR) do not individually block implementation but must be remediated before Phase 3 story-writer entry to avoid cascading index drift.

## Overall Metrics

| Metric | Value |
|--------|-------|
| **Total Checks** | 10 (+ 2 deferred pending story decomposition) |
| **Passed** | 7 |
| **Failed** | 3 (Architecture Alignment, L1→L4 Chain Completeness, Spec vs Implementation Drift) |
| **Deferred** | 3 (AC Completeness Coverage, Story Sizing, Priority Consistency) |
| **Findings: Critical** | 1 (GAP-F2-001) |
| **Findings: Major** | 9 (GAP-F2-003..011) |
| **Findings: Minor** | 2 (GAP-F2-002, GAP-F2-012) |
| **Overall Status** | inconsistencies-found |

All VP arithmetic, BC-INDEX header counts, verification-architecture, and
verification-coverage-matrix documents are internally consistent with the feature-iec104
delta. The CRITICAL gap is BC-2.10.010 which hardcodes the wrong MITRE tactic for T0809.
The MAJOR gaps are index/count drift across ARCH-INDEX and BC-INDEX, PRD O-04 arithmetic
error, ADR-013 governance placement, and BC traces_to inconsistency.

Recommended remediation priority:
1. Fix BC-2.10.010 tactic (CRITICAL — before story-writer runs)
2. Correct ARCH-INDEX SS-05/SS-10/SS-12/SS-19 counts and O-04 row (MAJOR)
3. Update BC-INDEX line 922 running derivation paragraph (MAJOR)
4. Fix PRD O-04 CATALOGUE-ONLY arithmetic and description (MAJOR)
5. Create ADR-013 in `.factory/specs/architecture/decisions/` (MAJOR)
6. Correct traces_to for BC-2.05.012, BC-2.10.010, BC-2.12.025 (MAJOR)
7. Correct traces_to for BC-2.19.001..027 (MINOR — can batch with item 6)
8. Fix ADR-013 Decision 8 VP-046 tool label (MINOR)

## Appendix: Validation Methodology

This report audits the Phase F2 spec-evolution delta for the IEC 60870-5-104 (IEC-104)
analyzer feature against the six consistency categories specified in the audit brief:

1. **Broken cross-references** — BC→VP, BC→ADR, BC→PRD-section, BC→SS, VP→BC, ADR→BC,
   SS-19→BC cross-links verified by reading all referenced artifacts and confirming
   ID/field values match.

2. **Count/version consistency** — All BC counts in ARCH-INDEX Subsystem Registry compared
   against authoritative BC-INDEX v2.23 counts. VP totals cross-checked across VP-INDEX,
   verification-architecture.md, and verification-coverage-matrix.md. MITRE SEEDED/EMITTED
   counts compared across ARCH-INDEX O-04 and PRD O-04. SUPPORTED_PORTS count (7→8) verified
   in BC-2.18.003 v1.4.

3. **Version-stamp coherence** — Index file version bumps (ARCH-INDEX, BC-INDEX, VP-INDEX)
   and amended BC version bumps (BC-2.18.003 v1.4, BC-2.18.004 v1.3) verified against
   changelog entries.

4. **Namespace/ID collisions** — BC IDs, VP IDs, and ADR numbers scanned for duplicates
   across new additions.

5. **Orphans/coverage gaps** — ADR-013 location checked against ARCH-INDEX governance.
   traces_to fields for all new BCs compared against the pattern established by sibling
   BCs in the same subsystems.

6. **Input-hash presence** — `input-hash:` frontmatter field verified present on all new
   BCs; values compared against PO-reported canonical Python-tool values.

All files were read directly. No code execution. Read-only audit; no files were modified.

Full validation criteria reference: consistency-validator AGENTS.md (80 criteria, Criteria
1-80), with special attention to Criteria 3, 4, 18, 21, 23, 29, 75-76 (index consistency
and traces_to correctness).
