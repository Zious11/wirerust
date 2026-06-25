---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
timestamp: 2026-06-24T00:00:00Z
traces_to: .factory/stories/STORY-INDEX.md
cycle: feature-enip-v0.11.0
stories_audited: STORY-130..138
holdout_scenarios_audited: HS-110..122
---

# F3 Story Decomposition Consistency Audit — EtherNet/IP (feature-enip-v0.11.0)

**Audit date:** 2026-06-24
**Scope:** 9 stories (STORY-130..138), 13 holdout scenarios (HS-110..122), 26 SS-17 BCs
**Purpose:** Perimeter consistency check before adversarial story convergence and F3 gate
**Verdict:** NOT CONSISTENT — 3 CRITICAL defects, 2 MEDIUM defects, 1 LOW defect

---

## Summary Table

| Dimension | Result | Severity of worst finding |
|-----------|--------|--------------------------|
| 1. AC→BC Traceability | FAIL | CRITICAL |
| 2. BC Coverage Map | FAIL | CRITICAL |
| 3. Test-Name Sync | PASS | — |
| 4. Dependency / Wave Validity | PASS | — |
| 5. Index / Count Integrity | PASS | — |
| 6. Input-Hash Validity | FAIL | MEDIUM |
| 7. Holdout Coverage | FAIL | CRITICAL |
| 8. Scope Coherence | FAIL | MEDIUM |

---

## Dimension 1 — AC→BC Traceability

**Result: FAIL**

### CRITICAL-1: CipServiceClass enum incomplete — 7 variants missing (STORY-130 vs BC-2.17.007)

**Artifact:** `STORY-130.md` AC-130-005 vs `BC-2.17.007.md`

BC-2.17.007 ("classify_cip_service Total Classification … 13 Named Request Services + Response + Unknown = 15 Variants") requires these variants:

| Code | Variant Required by BC-2.17.007 | Present in STORY-130 AC-130-005? |
|------|--------------------------------|----------------------------------|
| 0x01 | GetAttributesAll | No (story has GetAttributeSingle only) |
| 0x02 | SetAttributesAll | No (story maps 0x02 → SetAttributeList — WRONG) |
| 0x03 | GetAttributeList | No (story maps 0x55 → GetAttributeList — WRONG code) |
| 0x04 | SetAttributeList | Yes |
| 0x05 | Reset | No |
| 0x07 | Stop | No |
| 0x0A | MultipleServicePacket | No |
| 0x0E | GetAttributeSingle | Yes |
| 0x10 | SetAttributeSingle | Yes |
| 0x4B | GetAndClear | No |
| 0x4E | ForwardClose | Yes |
| 0x54 | ForwardOpen | Yes |
| 0x5B | LargeForwardOpen | No |
| high-bit | Response | No |
| else | Unknown(u8) | Yes |

STORY-130 specifies only 8 named variants (GetAttributeSingle, GetAttributesAll, GetAttributeList, SetAttributeSingle, SetAttributeList, ForwardOpen, ForwardClose, UnconnectedSend) instead of 15. Seven variants are missing from AC-130-005: Stop(0x07), Reset(0x05), SetAttributesAll(0x02), MultipleServicePacket(0x0A), GetAndClear(0x4B), LargeForwardOpen(0x5B), Response.

Additionally, two service code mappings conflict with BC-2.17.007:
- STORY-130 maps 0x02 → SetAttributeList; BC-2.17.007 maps 0x02 → SetAttributesAll
- STORY-130 maps 0x55 → GetAttributeList; BC-2.17.007 maps 0x03 → GetAttributeList (0x55 has no named variant)

**Cascade effect:** STORY-135 depends on `CipServiceClass::Stop` and `CipServiceClass::Reset` (via raw-byte workaround for Reset, but the Stop arm does not exist). HS-111 (must-pass) tests `classify_cip_service(0x07) == Stop` which requires the Stop variant. The enum gap blocks correct implementation of STORY-135 AC-135-001 and the holdout scenario fixture.

**Remediation:** Revise STORY-130 AC-130-005 to enumerate all 15 variants per BC-2.17.007. Correct service code mappings: 0x02 → SetAttributesAll, 0x03 → GetAttributeList. Add Stop(0x07), Reset(0x05), SetAttributesAll(0x02), MultipleServicePacket(0x0A), GetAndClear(0x4B), LargeForwardOpen(0x5B), Response (high-bit set arm).

---

### CRITICAL-3: STORY-135 implements T0858 on wrong trigger — SetAttribute instead of Stop

**Artifact:** `STORY-135.md` AC-135-001 vs `BC-2.17.011.md`

BC-2.17.011 title: "CIP Stop Service Observed Emits T0858 Change Operating Mode Finding"
BC-2.17.011 Precondition 1: `classify_cip_service(cip_header.service)` returns `CipServiceClass::Stop`
BC-2.17.011 Postcondition: `summary: "CIP Stop service observed: controller run→stop transition command (T0858)"`

STORY-135 AC-135-001 (traces to BC-2.17.011) instead specifies:
- Trigger: `classify_cip_service(service)` returning `SetAttributeSingle (0x10)` OR `SetAttributeList (0x02)`
- Summary: `"CIP SetAttribute request: potential operating mode change (T0858)"`

STORY-135's own BC table relabels BC-2.17.011 as "CIP SetAttribute request emits T0858 (Change Operating Mode)" — directly contradicting BC-2.17.011's canonical title and content.

**Consequence:** No story implements Stop(0x07) → T0858. BC-2.17.011's actual postcondition is orphaned. HS-111 (must-pass holdout, "CIP Stop Service on 0x00B2 Emits T0858") will FAIL on any implementation built from STORY-135.

**Note:** The SetAttribute → T0858 mapping as "potential operating mode change" may be a reasonable security signal, but it belongs in a different BC (not BC-2.17.011) or BC-2.17.011 must be corrected in the BC source. As written, STORY-135 neither implements BC-2.17.011's postcondition nor provides a corrected forward trace.

**Remediation:** Revise STORY-135 AC-135-001 to implement Stop(0x07) → T0858 per BC-2.17.011. If SetAttribute → T0858 is also desired, it requires a new BC or an amendment to BC-2.17.011 before the story can correctly reference it.

---

## Dimension 2 — BC Coverage Map

**Result: FAIL**

### BC-to-Story Assignment Summary (26 BCs × 9 stories)

| BC | Assigned Story | Coverage Notes |
|----|---------------|----------------|
| BC-2.17.001 | STORY-130 | Covered |
| BC-2.17.002 | STORY-130 | Covered |
| BC-2.17.003 | STORY-130 | Covered |
| BC-2.17.004 | STORY-130 | Covered |
| BC-2.17.005 | STORY-132 | Covered |
| BC-2.17.006 | STORY-132 | Covered |
| BC-2.17.007 | STORY-130 + STORY-132 | CRITICAL-1 breaks Stop/Reset arms |
| BC-2.17.008 | STORY-134 | Covered |
| BC-2.17.009 | STORY-132 | Covered |
| BC-2.17.010 | STORY-134 | Covered |
| BC-2.17.011 | STORY-135 | CRITICAL-3: story implements wrong trigger |
| BC-2.17.012 | STORY-135 | Covered |
| BC-2.17.013 | STORY-135 | Covered |
| BC-2.17.014 | STORY-134 | Covered |
| BC-2.17.015 | STORY-136 | Covered |
| BC-2.17.016 | STORY-137 | Covered |
| BC-2.17.017 | STORY-138 | Covered |
| BC-2.17.018 | STORY-137 | Covered |
| BC-2.17.019 | STORY-131 | Covered |
| BC-2.17.020 | STORY-131 | Covered |
| BC-2.17.021 | STORY-138 | Covered |
| BC-2.17.022 | STORY-138 | Covered |
| BC-2.17.023 | STORY-131 | Covered |
| BC-2.17.024 | STORY-138 | Covered |
| BC-2.17.025 | STORY-138 | Covered |
| BC-2.17.026 | STORY-131 | Covered |

**No orphaned BCs** (all 26 assigned). No doubled-up BCs (each BC assigned to exactly one story except BC-2.17.007 which spans STORY-130 and STORY-132 — acceptable as the classify function is defined in STORY-130 and CipHeader parsing in STORY-132).

**CRITICAL finding on BC-2.17.011:** Although assigned, the BC is not correctly implemented per its contract (see CRITICAL-3). Effective postcondition coverage = 0% for Stop(0x07) → T0858.

---

## Dimension 3 — Test-Name Sync (DF-AC-TEST-NAME-SYNC-001 + DF-TEST-NAMESPACE-001)

**Result: PASS**

All 9 stories declare test modules in `tests/enip_analyzer_tests.rs` with `mod` wrappers. No story omits a test namespace.

| Story | Test Module | Compliance |
|-------|-------------|-----------|
| STORY-130 | (VP-032 Kani tests in `src/` inline; unit tests in `tests/enip_analyzer_tests.rs`) | Pass |
| STORY-131 | `mod dispatch` | Pass |
| STORY-132 | `mod cpf_cip` | Pass |
| STORY-133 | `mod mitre_seeding` | Pass |
| STORY-134 | `mod recon` | Pass |
| STORY-135 | `mod command_detections` | Pass |
| STORY-136 | `mod connection_lifecycle` | Pass |
| STORY-137 | `mod frame_walk` | Pass |
| STORY-138 | `mod session_lifecycle` | Pass |

Test names declared within each story follow AC-level naming (`test_<ac_slug>`) and are scoped inside the per-story `mod`. No conflicts between modules detected.

---

## Dimension 4 — Dependency / Wave Validity

**Result: PASS**

### Wave Schedule

| Wave | Stories | Dependencies |
|------|---------|--------------|
| W58 | 130, 131 | None (independent foundations) |
| W59 | 132, 133 | 132 depends on 130; 133 depends on 131 |
| W60 | 134, 135, 136, 137 | 134 depends on 132+133; 135 depends on 132+133; 136 depends on 132+133; 137 depends on 132+133 |
| W61 | 138 | Depends on 134+135+136+137 |

### Intra-E-20 Edge Set (14 edges)

130→132, 131→133, 132→134, 132→135, 132→136, 132→137, 133→134, 133→135, 133→136, 133→137, 134→138, 135→138, 136→138, 137→138

All edges are forward-only (no cycles). Wave assignments are consistent with declared `depends_on` fields in each story frontmatter. Dependency graph v3.1 confirms acyclic: true.

**Topological ordering note:** STORY-133 (W59) seeds T0846 into EMITTED_IDS before STORY-134 (W60) first emits T0846. Ordering is correct.

VP-007 atomic obligation: STORY-133 lands the 6-part burst (technique_info arms, SEEDED 25→28, SEEDED_TECHNIQUE_ID_COUNT 25→28, EMITTED_IDS 17→20, MitreTactic::IcsExecution variant, cargo test) in a single story at W59 before any emitting story (W60). This is correct per ADR-010 Decision 7.

---

## Dimension 5 — Index / Count Integrity

**Result: PASS**

### STORY-INDEX.md (v2.8)

- Total stories: 91 (82 pre-E-20 + 9 E-20 additions). All 9 ENIP stories (130..138) present in wave table.
- Wave count: 61 waves (W58–W61 are the ENIP additions).
- Total points: 592 (noted as three-scope explanation in INDEX).
- E-20 registered with 9 stories, 66 pts, BC-2.17.001..026 listed. Matches story frontmatter point values: 8+8+8+5+8+8+5+8+8 = 66.

### epics.md (v1.8)

- E-20 epic present with 9 stories. total_bcs: 328 (includes 26 new SS-17 BCs). BC listing matches all 26.

### dependency-graph.md (v3.1)

- 89 product stories listed (count per graph; STORY-INDEX reports 91 total — delta of 2 may reflect infra/tooling stories not in product graph, consistent with prior cycles).
- 120 edges (99 intra, 21 cross). 14 intra-E-20 edges confirmed above.
- acyclic: true.

All counts are internally consistent. No orphaned story files detected.

---

## Dimension 6 — Input-Hash Validity

**Result: FAIL**

### Stories: PASS

All 9 story files carry non-TBD input hashes:

| Story | input-hash |
|-------|-----------|
| STORY-130 | d709bd4 |
| STORY-131 | c9970ba |
| STORY-132 | 3343540 |
| STORY-133 | 7104101 |
| STORY-134 | 33352dc |
| STORY-135 | 4ba6bb1 |
| STORY-136 | 2af89b5 |
| STORY-137 | 24ecccd |
| STORY-138 | fe79905 |

All story hashes are 7-character hex strings. Freshness (computed vs stored) was not verified in this read-only audit; run `bin/compute-input-hash --scan` against stories 130–138 to confirm stored hashes match current inputs.

### Holdout Scenarios: FAIL (MEDIUM-1)

**All 13 holdout scenarios carry `input-hash: "tbd"`** — none have been computed.

Affected files: HS-110 through HS-122.

Per CLAUDE.md canonical algorithm, hashes must be computed from the `inputs:` list in each holdout scenario's frontmatter. This is a pre-F4 obligation. Hashes should be computed with `bin/compute-input-hash --write HS-NNN.md` for each of the 13 files before the F4 evaluation gate.

**Severity:** MEDIUM (does not block F3 spec gate; blocks F4 evaluation gate integrity).

---

## Dimension 7 — Holdout Coverage

**Result: FAIL**

### BC-to-Holdout Coverage Matrix (selected)

| BC | Holdout | must_pass | Coverage |
|----|---------|-----------|---------|
| BC-2.17.001/002/003 (parse_enip_header) | HS-110 | true | Covered — DF-CANONICAL-FRAME-HOLDOUT-001 compliant |
| BC-2.17.011 (Stop → T0858) | HS-111 | true | FAILS per CRITICAL-3 (story implements SetAttribute, not Stop) |
| BC-2.17.019/020 (dispatch port 44818) | HS-120 | true | Covered |
| BC-2.17.005/011/012/013 (0x00B1 deferral) | HS-119 | true | Covered |
| All CIP detection BCs | HS-119 Case D positive control | true | Covered (requires Stop→T0858, so also blocked by CRITICAL-3) |

### HS-111 Holdout Will Fail on CRITICAL-3

HS-111 ("CIP Stop Service on 0x00B2 Emits T0858") is a `must-pass` holdout. It presents a PCAP with CIP service byte 0x07 (Stop) in a 0x00B2 item and expects exactly one T0858 finding. Per STORY-135's current specification, the T0858 trigger is SetAttribute (0x10/0x02), not Stop (0x07). An implementation built from STORY-135 as written will produce zero T0858 findings on a CIP Stop frame, causing HS-111 to FAIL with score 0.0 on the 0.50-weight "T0858 emission correctness" rubric item.

HS-119 Case D (positive control: CIP Stop in 0x00B2 fires T0858) has the same failure mode.

### Holdout Completeness Assessment

All 13 holdout scenarios (HS-110..122) are active, have BC linkage tables, fixture creation obligations, and evaluation rubrics. No missing holdout coverage for the major behavioral areas (endianness, service detection, scope gates, dispatch, session, DoS thresholds, recon, write burst, Kani proofs, MITRE seeding, connection lifecycle).

---

## Dimension 8 — Scope Coherence

**Result: FAIL (MEDIUM)**

### MEDIUM-2: STORY-133 subsystems: [SS-17] — wrong subsystem anchor

**Artifact:** `STORY-133.md` frontmatter

STORY-133 delivers the VP-007 atomic update to `src/mitre.rs` — the MITRE technique catalog module. Per the architecture, `src/mitre.rs` belongs to SS-10 (MITRE Mapping subsystem), not SS-17 (EtherNet/IP subsystem).

STORY-133's `subsystems: [SS-17]` is semantically incorrect. The story modifies zero SS-17 source files; its entire deliverable is in SS-10 scope.

**Severity:** MEDIUM. Does not break implementation correctness but creates a semantic anchor error that could mislead dependency analysis and future cross-story subsystem queries.

**Remediation:** Change STORY-133 frontmatter `subsystems: [SS-17]` to `subsystems: [SS-10]`. If SS-17 is also listed because STORY-133 is logically part of the E-20 epic, add a `epic_subsystem_note` field rather than misassigning the primary subsystem.

---

## Defect Register

| ID | Severity | Dimension | Story / Artifact | Description |
|----|----------|-----------|-----------------|-------------|
| CRITICAL-1 | CRITICAL | 1, 2 | STORY-130 / BC-2.17.007 | CipServiceClass enum missing 7 variants (Stop, Reset, SetAttributesAll, MultipleServicePacket, GetAndClear, LargeForwardOpen, Response). Two service codes mapped to wrong variants (0x02→SetAttributeList should be SetAttributesAll; 0x55→GetAttributeList is not a valid code — should be 0x03). Cascades to STORY-135 and HS-111. |
| CRITICAL-3 | CRITICAL | 1, 7 | STORY-135 / BC-2.17.011 | AC-135-001 implements T0858 on SetAttribute trigger, not Stop(0x07) as required by BC-2.17.011. Must-pass holdout HS-111 will fail on implementation built from STORY-135 as written. BC-2.17.011 postcondition is unimplemented; no story provides Stop(0x07)→T0858 coverage. |
| MEDIUM-1 | MEDIUM | 6 | HS-110..122 | All 13 holdout scenarios have `input-hash: "tbd"`. Must be computed before F4 evaluation gate. |
| MEDIUM-2 | MEDIUM | 8 | STORY-133 | `subsystems: [SS-17]` is wrong; deliverable targets `src/mitre.rs` (SS-10). Semantic anchor error. |
| LOW-1 | LOW | 1 | STORY-132 | Input list contains double-slash path `.factory/specs/behavioral-contracts/ss-17//BC-2.17.009.md`. Cosmetic; file is accessible on macOS/Linux. Fix by removing the extra slash. |

**Note on retired CRITICAL-4:** An earlier draft of this audit flagged STORY-133 VP-007 Step 4 as omitting T0888 from EMITTED_IDS (17→20 adds only T0858/T0816/T0846). This was incorrect. T0888 was already in EMITTED_IDS from STORY-100 (which set EMITTED to 13, including T0888). The 17-count pre-STORY-133 baseline (established by STORY-109: 13→15, then STORY-114: 15→17) already includes T0888. The 17→20 addition correctly adds only T0858, T0816, and T0846 (which transitions from seeded-only to emitted). STORY-133 is correct on this point.

---

## Blocking Gate Assessment

### F3 Story Spec Gate: BLOCKED

Two CRITICAL defects prevent F3 gate passage:

1. **CRITICAL-1** (STORY-130 enum incompleteness): The `classify_cip_service` function's enum contract, as specified in STORY-130, is materially different from BC-2.17.007. Any test built to BC-2.17.007's 15-variant contract will fail against an implementation built to STORY-130's 8-variant specification. This is a spec-level defect that must be corrected before story handoff to the implementer.

2. **CRITICAL-3** (STORY-135 T0858 trigger): The T0858 trigger in STORY-135 contradicts BC-2.17.011. HS-111 is a must-pass holdout that directly tests the correct trigger. Implementation built from current STORY-135 will fail HS-111 with certainty.

### Required Actions Before Gate Passage

1. Revise `STORY-130.md` AC-130-005: expand CipServiceClass to 15 variants per BC-2.17.007; correct code mappings (0x02→SetAttributesAll, 0x03→GetAttributeList).
2. Revise `STORY-135.md` AC-135-001: change T0858 trigger from SetAttribute to Stop(0x07) per BC-2.17.011. Decide whether SetAttribute→T0858 is a separate detection needing a new BC or an amendment to BC-2.17.011.
3. (Pre-F4) Compute and write input hashes for HS-110..122 using `bin/compute-input-hash --write`.
4. (Recommended) Correct `STORY-133.md` subsystems to `[SS-10]`.
5. (Cleanup) Remove double-slash from `STORY-132.md` input path.

---

*Audit performed by consistency-validator. Read-only; no source artifacts modified.*
