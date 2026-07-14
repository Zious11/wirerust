---
document_type: consistency-report
level: ops
version: "1.0"
producer: vsdd-factory:consistency-validator
traces_to: f2-consistency-reaudit-3.md
timestamp: "2026-07-13T00:00:00Z"
cycle: feature-iec104
pass: 4
---

# F2 Spec-Evolution Consistency Audit — Pass 4

**Cycle:** feature-iec104  
**Stage:** F2 spec-evolution (spec written before implementation)  
**Pass:** 4 (post-third-remediation-burst)  
**Date:** 2026-07-13  
**Auditor:** vsdd-factory:consistency-validator (fresh context)

---

## Verdict: PASS

Zero blocking findings. Zero new gaps introduced by the third remediation burst. The single MINOR finding from Pass 3 (BURST-GAP-001) is closed.

---

## Summary Table

| Check Category | Result | Notes |
|----------------|--------|-------|
| BURST-GAP-001 closure | PASS | BC-2.18.003 has exactly one v1.4 inline comment in BC-INDEX |
| VP reciprocity — VP-044 | PASS | source_bc {001–005} ↔ forward anchors in all five BCs |
| VP reciprocity — VP-044 over-scope (006, 015) | PASS | Both BCs anchor VP-047 only; VP-044 absent from both |
| VP reciprocity — VP-045 | PASS | source_bc {025,026} ↔ VP-045 forward anchor in both BCs |
| VP reciprocity — VP-046 | PASS | source_bc {007,008,009} ↔ forward anchors in all three BCs |
| VP reciprocity — VP-047 fuzz asymmetry | PASS | Intentional fuzz-VP convention; VP-028 precedent; not a gap |
| VP reciprocity — VP-004 (BC-2.05.012) | PASS | BC-2.05.012 forward-anchors VP-004; VP-INDEX source_bc includes BC-2.05.012 |
| VP reciprocity — VP-007 (BC-2.10.010) | PASS | BC-2.10.010 forward-anchors VP-007; VP-INDEX source_bc includes BC-2.10.010 |
| ADR-013 copy identity | PASS | docs/ and .factory/ copies are byte-identical |
| BC count coherence | PASS | BC-INDEX header: 378 on-disk / 377 active; SS-19 = 27 |
| VP total coherence | PASS | 16+22+3+6=47; consistent across VP-INDEX, verification-architecture, coverage-matrix |
| Version stamps | PASS | All match expected values (see detail below) |
| SEEDED/EMITTED source gap | PRE-EXISTING | SEEDED=28 in source (spec: 29 post-impl); EMITTED=21; iec104.rs not yet created |
| SUPPORTED_PORTS source gap | PRE-EXISTING | 8 entries; 2404 absent (implementation not done) |
| No new inconsistencies from burst 3 | PASS | All burst-3 artifacts checked; no orphans or collisions |

---

## Detailed Check Results

### 1. BURST-GAP-001 — BC-2.18.003 Duplicate v1.4 Comment

**Status: CLOSED**

BC-INDEX v2.26, line 846: exactly one inline HTML comment for BC-2.18.003 — `<!-- v1.4: feature-iec104 — SUPPORTED_PORTS adds 2404 (IEC-104 TCP Rule 8); supported entries count 7→8 -->`. The duplicate present in v2.25 was deduplicated in v2.26 per the modification log.

BC-2.18.003.md modification history (lines 15–19): single v1.4 entry — "v1.4: BC-INDEX v2.23 feature-iec104 amendment — SUPPORTED_PORTS adds port 2404 (IEC-104); supported entries count 7→8; port 2404 reflected in Precondition 3, Invariant 1, EC-005, Canonical Test Vectors. NEW-GAP-002: inputs and input-hash frontmatter added."

No residual duplicate. BURST-GAP-001 fully closed.

---

### 2. VP Reciprocity — All IEC-104 VPs

#### VP-044 (Kani P0 — parse_apci_header no-panic + bounds arithmetic)

**source_bc in VP-INDEX:** {BC-2.19.001, BC-2.19.002, BC-2.19.003, BC-2.19.004, BC-2.19.005}

DOCTRINE note codified in VP-INDEX v2.43: BCs 006 and 015 are fuzz-nocrash targets routed to VP-047 (parse_apci_header is their scope boundary); BC-2.19.026 is a scoped sub-call, not a VP-044 source_bc.

Forward anchors in source BCs:

| BC | VP-044 forward anchor | VP-047 forward anchor |
|----|----------------------|-----------------------|
| BC-2.19.001 | present | present (supplementary) |
| BC-2.19.002 | present | present (supplementary) |
| BC-2.19.003 | present | present (supplementary) |
| BC-2.19.004 | present | present (supplementary) |
| BC-2.19.005 | present | present (supplementary) |

BC-2.19.006 v1.1: VP-044 absent; VP-047 present. Modification note: "F-P3-H1 — VP-044 over-scope: is_valid_iec104_frame is not parse_apci_header; re-anchored to VP-047 per ADR-013 Decision 8."

BC-2.19.015 v1.1: VP-044 absent; VP-047 present. Modification note: "F-P3-H1 — VP-044 over-scope: parse_asdu is covered by VP-047 fuzz, not VP-044 Kani per ADR-013 Decision 8."

Verdict: CLEAN — no VP-044 over-scope forward anchors; no missing forward anchors among source_bc BCs.

#### VP-045 (proptest P1 — IEC-104 carry-buffer direction isolation)

**source_bc in VP-INDEX:** {BC-2.19.025, BC-2.19.026}

VP-INDEX v2.43 note: intentional; BC-2.19.026 drives directional carry management. PO OBLIGATION note added.

BC-2.19.025: VP-045 forward anchor present; VP-047 forward anchor also present (supplementary).

BC-2.19.026 v1.2: VP-045 forward anchor present (added v1.1→v1.2 per "P3 fix — VP-045 forward-anchor added"); VP-044 present (parse_apci_header sub-call scope); VP-047 present (on_data no-panic).

Verdict: CLEAN — bidirectional reciprocity confirmed.

#### VP-046 (proptest P1 — classify_frame_format totality over all 256 u8)

**source_bc in VP-INDEX:** {BC-2.19.007, BC-2.19.008, BC-2.19.009}

BC-2.19.007: VP-046 forward anchor present.  
BC-2.19.008: VP-046 forward anchor present.  
BC-2.19.009: VP-046 forward anchor present (primary totality target).

Verdict: CLEAN — full bidirectional reciprocity.

#### VP-047 (cargo-fuzz P1 — fuzz_iec104_parser no-panic)

**source_bc in VP-INDEX:** {BC-2.19.001, BC-2.19.025, BC-2.19.027}

Forward anchors confirmed: BC-2.19.001 (present), BC-2.19.025 (present), BC-2.19.027 (present).

Additional forward-only anchors (not in source_bc): BC-2.19.002, .003, .004, .005, .006, .015, .026 each cite VP-047 in their VP tables as supplementary fuzz-nocrash coverage. These forward-only anchors are intentional and follow the established VP-028 (pcapng no-panic fuzz) convention: VP-028 source_bc = {BC-2.01.017} only, yet BC-2.01.013 cites VP-028 as supplementary coverage. Fuzz VPs have a primary driver scope; additional BCs acknowledging coverage do not require source_bc listing.

VP-INDEX v2.43 DOCTRINE note explicitly acknowledges this pattern for VP-047.

Verdict: CLEAN — no bidirectionality violation; asymmetric forward-only citations follow documented fuzz-VP convention.

#### VP-004 and VP-007 (pre-existing VPs with IEC-104 amendments)

VP-004: BC-2.05.012 added to VP-INDEX source_bc in v2.42. BC-2.05.012 forward-anchors VP-004 (line 82) and lists VP-004 in VP Anchors (line 113). Bidirectional. CLEAN.

VP-007: BC-2.10.010 added to VP-INDEX source_bc in v2.42. BC-2.10.010 forward-anchors VP-007 (line 79) and lists VP-007 in VP Anchors (line 110). SEEDED count annotation present. Bidirectional. CLEAN.

---

### 3. ADR-013 Copy Identity

`docs/adr/0013-iec104-stream-dispatch-and-parser-design.md` vs `.factory/specs/architecture/decisions/ADR-013-iec104-stream-dispatch-and-parser-design.md`

Both files contain identical section structure: Context (Context anchor), Decision, Rationale, Consequences, Alternatives Considered, Source / Origin. `diff` of the two files returned no output — byte-identical.

Verdict: CLEAN.

---

### 4. Count and Version Coherence

| Artifact | Expected | Confirmed |
|----------|----------|-----------|
| BC-INDEX on-disk count | 378 | 378 |
| BC-INDEX active count | 377 | 377 (BC-2.01.004 retired) |
| SS-19 BC count | 27 | 27 (BC-2.19.001..027) |
| VP total | 47 | 47 |
| VP Kani | 16 | 16 |
| VP proptest | 22 | 22 |
| VP cargo-fuzz | 3 | 3 |
| VP integration/unit | 6 | 6 |
| EMITTED_IDS in source | 21 | 21 |
| SUPPORTED_PORTS count in source | 8 | 8 |
| BC-INDEX version | v2.26 | v2.26 |
| PRD version | v1.53 | v1.53 |
| VP-INDEX version | v2.43 | v2.43 |
| ARCH-INDEX version | v2.14 | v2.14 |
| verification-architecture version | v2.31 | v2.31 |
| verification-coverage-matrix version | v1.46 | v1.46 |
| ss-19-iec104-analysis version | v1.1 | v1.1 |

All values match. VP totals (16+22+3+6=47) are consistent across VP-INDEX, verification-architecture.md, and verification-coverage-matrix.md.

---

### 5. Pre-Existing Source Gaps (Not Gaps in This Audit)

These gaps were accepted in Pass 1 and remain unchanged. They are spec-ahead-of-implementation conditions for a feature in F2 (no source implementation yet).

**SEEDED count (src/mitre.rs):** `SEEDED_TECHNIQUE_IDS` has 28 entries; T0881 absent. `SEEDED_TECHNIQUE_ID_COUNT = 28`. Spec (BC-2.10.010, VP-007) projects 29 post-implementation. Gap closes when `src/analyzer/iec104.rs` is created.

**SUPPORTED_PORTS (src/protocols.rs):** 8 entries `{502, 20000, 44818, 443, 8443, 80, 8080, 53}`; 2404 absent. Port 2404 IS present in `KNOWN_PROTOCOLS.canonical_ports` for the IEC-104 entry. Gap closes when `src/analyzer/iec104.rs` is created and `SUPPORTED_PORTS` is expanded.

`src/analyzer/iec104.rs` does not exist. These are expected pre-implementation conditions.

---

### 6. Namespace / ID Collisions

No VP, BC, or ADR ID collisions detected. VP-044 through VP-047 are new IDs with no prior usage. ADR-013 is the only file with the 0013 prefix. BC-2.19.001..027 are all under SS-19 with no collision against any other SS.

---

### 7. Orphan Check

All 27 SS-19 BCs are referenced in BC-INDEX SS-19 section. All four new VPs (VP-044, VP-045, VP-046, VP-047) appear in VP-INDEX, verification-architecture.md (Must Prove / Should Prove tables), and verification-coverage-matrix.md. ADR-013 is referenced in BC-2.19.006 and BC-2.19.015 modification history. No orphaned artifacts detected.

---

### 8. Input-Hash Presence

All 32 IEC-104 BCs (27 SS-19 BCs + BC-2.18.003, BC-2.18.004, BC-2.05.012, BC-2.10.010, and the remaining amended BCs) carry `input-hash:` frontmatter. Spot-checked: BC-2.18.003 `input-hash: "84318a1"` present; BC-2.10.010 `input-hash: "89a6214"` present.

---

## Gap Register

| Gap ID | Status | Description |
|--------|--------|-------------|
| BURST-GAP-001 | CLOSED | BC-2.18.003 duplicate v1.4 inline comment in BC-INDEX — deduplicated in v2.26 |
| NEW-GAP-001 | CLOSED (pass 3) | VP-044 source_bc limited to {001,002,003} — expanded to {001,002,003,004,005} in VP-INDEX v2.43 |
| NEW-GAP-002 | CLOSED (pass 2) | Missing inputs/input-hash frontmatter on IEC-104 BCs |
| SOURCE-GAP-001 | PRE-EXISTING | SEEDED count = 28 in source; spec projects 29 — closes when iec104.rs is written |
| SOURCE-GAP-002 | PRE-EXISTING | SUPPORTED_PORTS count = 8; 2404 absent — closes when iec104.rs is written |

---

## Total by Category

| Category | Gaps Found | Gaps Closed This Pass | Pre-Existing (Accepted) |
|----------|-----------|----------------------|-------------------------|
| BC-INDEX comment integrity | 0 | 1 (BURST-GAP-001) | 0 |
| VP reciprocity | 0 | 0 | 0 |
| ADR copy identity | 0 | 0 | 0 |
| Count/version coherence | 0 | 0 | 0 |
| Source implementation gaps | 0 | 0 | 2 (SOURCE-GAP-001, -002) |
| ID collisions / orphans | 0 | 0 | 0 |
| **Total new gaps this pass** | **0** | — | — |

---

## Gate Decision

**PASS.** The F2 spec-evolution delta for the IEC-104 feature is internally consistent. The third remediation burst introduced no new inconsistencies. BURST-GAP-001 is confirmed closed. VP reciprocity is clean across all six IEC-104-touching VPs (VP-004, VP-007, VP-044, VP-045, VP-046, VP-047). ADR-013 copies are byte-identical. All count, version, and ID cross-references are coherent. The two pre-existing source gaps are accepted spec-ahead-of-implementation conditions that close when `src/analyzer/iec104.rs` is created.
