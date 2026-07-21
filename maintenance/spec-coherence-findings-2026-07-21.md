---
producer: consistency-validator
run: maint-2026-07-21
sweep: 7
check: DF-030 Spec Coherence
date: 2026-07-21
artifact_versions:
  bc_index: v2.34
  vp_index: v2.46
  arch_index: v2.19
  prd: v1.57
  story_index: v3.86
  dep_graph: v3.9
  state_md: "2.0"
---

# Spec Coherence Findings — maint-2026-07-21

Sweep 7 (DF-030) across BC-INDEX v2.34, VP-INDEX v2.46, ARCH-INDEX v2.19, PRD v1.57, STORY-INDEX v3.86.
Mandate: PROP-MAINT-03 — every count sourced from named artifact with file+line citation.
Prior baseline: `spec-coherence-findings-2026-07-11.md`.

---

## Findings Table

| ID | Severity | Artifact(s) | Description | Status |
|----|----------|-------------|-------------|--------|
| SPEC-001 | MINOR | HS-INDEX v2.13 | ENIP block stories/waves metadata stale. Line 777: `Stories: STORY-131..STORY-141 (waves 63-68)`. Correct values per STORY-INDEX v3.86: STORY-130..STORY-139 (waves 58–62). Also line 765 maintenance note table still shows "EtherNet/IP (waves 63-68)". | CARRIED (RC-1 maint-2026-07-11) |
| SPEC-002 | NIT | epics.md v2.1 | `total_bcs: 337` (epics.md line 19) vs BC-INDEX v2.34 active count 378 (BC-INDEX.md line 17). Gap widened 10→41 since prior sweep (IEC-104 F2 added 30 BCs + BC-2.11.036/037 + BC-2.19.028). | CARRIED (RC-2 maint-2026-07-11) |
| SPEC-003 | MINOR | ARCH-INDEX v2.19 | SS-11 BC count stale. Subsystem Registry line 192: `35`. BC-INDEX v2.34 line 17 states 348 base (which included BC-2.11.036/037 added v2.21 note: "SS-11: 35→37 BCs"). Correct value: 37. Gap: 2. | CARRIED (NEW-1 maint-2026-07-11) |
| SPEC-004 | MINOR | epics.md v2.1 | Estimated Story Count Summary TOTAL row stale. Line 641: `107` (verified against STORY-INDEX v3.12). STORY-INDEX v3.86 line 8: `total_stories: 132`. Gap widened 10→25 since prior sweep (STORY-167..174 IEC-104 F2, STORY-175..179 wave-84 work added after v3.12). | CARRIED (NEW-2 maint-2026-07-11) |
| SPEC-005 | NIT | STORY-INDEX v3.86 | Coverage verification BC tally stale. Line 553 annotates "total 337 BCs; explicit tally: 219 greenfield + … = 337". BC-INDEX v2.34 active count: 378 (line 17). Gap: 41. IEC-104 F2 added 30 BCs (BC-2.19.001..027 + BC-2.05.012 + BC-2.10.010 + BC-2.12.025 + BC-2.19.028) not reflected. | CARRIED (NEW-3 maint-2026-07-11) |
| SPEC-006 | NIT | BC-INDEX v2.34, src/ | SEC-W70-001 PO backlog action outstanding. No BC authored for `TlsAnalyzer::all_findings` unbounded-by-design behavior. SS-07 last BC is BC-2.07.043 (ARCH-INDEX v2.19 line 192 shows SS-07 = 43 BCs; no BC-2.07.044 present). Action from maint-2026-07-11 sweep: "author symmetric BC in next spec-coherence sweep." Still pending. | CARRIED (NEW-4 maint-2026-07-11) |
| SPEC-007 | NIT | VP-INDEX v2.46 | DRIFT-VP039-BC207038-TLS-TODO-001: VP-INDEX carries stale present-tense "PO must add BC-2.07.038 postcondition/EC + Red-Gate test name" TODOs for VP-039. STATE.md line 204 records this as an open drift item (added D-438, 2026-07-14). Out of current sweep scope per mandate. | CARRIED (STATE.md drift item) |
| SPEC-008 | MINOR | ARCH-INDEX v2.19, BC-INDEX v2.34 | SS-19 BC count stale. Subsystem Registry line 200: `27`. BC-INDEX v2.34 note at line 17: "v2.32: BC-2.19.028" added, making SS-19 active = 28. ARCH-INDEX was last modified 2026-07-15 (same date as BC-INDEX v2.34 per changelog) but BC-2.19.028 was not propagated to the Subsystem Registry SS-19 count. Gap: 1. | NEW |
| SPEC-009 | MINOR | STORY-INDEX v3.86 | Epic table TOTAL row arithmetic error. Line 452: `\| **TOTAL** \| \| **132** \| **776** \|`. Per-epic row sum: E-1(21)+E-2(73)+E-3(13)+E-4(34)+E-5(74)+E-6(5)+E-7(18)+E-8(40)+E-9(28)+E-10(3)+E-11(66)+E-12(18)+E-13(21)+E-14(45)+E-15(58)+E-16(50)+E-17(8)+E-18(16)+E-19(37)+E-20(79)+E-21(32)+E-22(36) = 775. Frontmatter line 10: `total_points: 775`. Epic TOTAL row is 1 over. Root cause: v3.79 changelog confirms STORY-147 re-scope delta 3→2 pt updated E-11 row (67→66) and frontmatter total_points (776→775) but did not decrement the epic table TOTAL cell. | NEW |
| SPEC-010 | NIT | STORY-158 (input-hash scan) | STORY-158 input hash STALE per `bin/compute-input-hash --scan` (stored=ac92b99, computed=5650b57). Not in STORY-INDEX-IN-INPUTS-CHURN known clusters (STATE.md line 205 covers STORY-164/165 and STORY-175..179 only). STORY-158 likely lists CLAUDE.md as an input (AC-158-006 CLAUDE.md gate-close protocol); wave-84 PRs #429/#430 modified CLAUDE.md, re-staling it. Story is delivered; hash is advisory-only per PG-HASH-HOOK-DIVERGENCE notes. | NEW |
| SPEC-011 | NIT | CLAUDE.md | Project References table row for `docs/adr/` (CLAUDE.md line 256) enumerates ADRs 0001–0007, 0009–0012 but omits ADR 0013 (IEC-104). File `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md` confirmed present on disk. ARCH-INDEX v2.19 lines 110/113 document that ADR-013 was added during feature-iec104 F2 (GAP-F2-010); the CLAUDE.md update was not included in the F2 delivery wave. | NEW |

---

## Input-Hash Scan Summary

Scan date: 2026-07-21 (`bin/compute-input-hash --scan`).

| Result | Count | Story IDs |
|--------|-------|-----------|
| MATCH | 124 | — |
| STALE (CARRIED — STORY-INDEX-IN-INPUTS-CHURN, STATE.md line 205) | 7 | STORY-164, STORY-165, STORY-175, STORY-176, STORY-177, STORY-178, STORY-179 |
| STALE (NEW — SPEC-010) | 1 | STORY-158 (stored=ac92b99, computed=5650b57) |
| **Total** | **132** | |

---

## Passed Checks (No Finding)

| Check | Verified |
|-------|----------|
| L1→L4 chain integrity: BC-INDEX traces_to PRD; ARCH-INDEX traces_to domain-spec.md + PRD | Pass |
| VP-INDEX v2.46 arithmetic: P0(9)+P1(32)+test_sufficient(6)=47; Kani(16)+proptest(22)+fuzz(3)+integration_unit(6)=47 (lines 9–16) | Pass |
| 30 IEC-104 BCs from F2: STORY-INDEX v3.86 E-22 epic row cites "BC-2.19.001-027 + BC-2.05.012 + BC-2.10.010 + BC-2.12.025" = 27+3 = 30 | Pass |
| STORY-111..117 supersession: index rows all status=superseded; wave table rows 40–46 show "superseded DELIVERED-BY-DRIFT v0.7.0; D-487"; E-16/E-17 epic rows marked DELIVERED/CLOSED | Pass |
| STORY-147/166/176 delivery: index rows delivered; STORY-INDEX v3.86 wave 84 row (line 543): "3/3 DELIVERED + GATE CLOSED (D-486, 2026-07-21)" | Pass |
| D-487 arithmetic: wave-table TOTAL row "116 \| 692" (STORY-INDEX line 422) + exclusion sum 83 = 775 = frontmatter total_points (line 10); STATE.md line 175 confirms | Pass |
| ADR-013 IEC-104 file exists: `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md` confirmed on disk | Pass |
| STATE.md stories_delivered (line 32: 116) consistent with STORY-INDEX v3.86 wave table TOTAL (116/692) | Pass |

---

## Finding Counts

- CARRIED: 7 (SPEC-001 through SPEC-007)
- NEW: 4 (SPEC-008 through SPEC-011)
- Severity breakdown: 0 MAJOR/BLOCKER; 3 MINOR (SPEC-001, SPEC-003, SPEC-008, SPEC-009 — note SPEC-001 and SPEC-003 are CARRIED MINOR; SPEC-008 and SPEC-009 are NEW MINOR); 4 NIT (SPEC-002, SPEC-005, SPEC-006, SPEC-007 CARRIED; SPEC-010, SPEC-011 NEW).
- Corrected severity counts: MINOR=4 (SPEC-001, SPEC-003 CARRIED; SPEC-008, SPEC-009 NEW), NIT=7 (SPEC-002, SPEC-005, SPEC-006, SPEC-007 CARRIED; SPEC-010, SPEC-011 NEW)
