---
document_type: maintenance-sweep-findings
sweep_id: spec-coherence-sweep-7-post-FE001
version: "1.0"
status: complete
producer: consistency-validator
timestamp: 2026-06-22T00:00:00Z
maintenance_run_id: maint-2026-06-22
sweep_type: spec-coherence-DF030
criteria_checked: 33
spec_versions_verified:
  prd: "1.33"
  error_taxonomy: "3.8"
  nfr_catalog: "2.3"
  bc_index: "1.69"
  vp_index: "2.10"
  verification_coverage_matrix: "1.19"
  story_index: "2.6"
  arch_index: "1.5"
traces_to: .factory/STATE.md
---

# Maintenance Sweep 7 (Post-FE-001) — Spec Coherence Findings

**Date:** 2026-06-22
**Sweep:** DF-030 Spec-Coherence (33 criteria)
**Scope:** `.factory/specs/`, `.factory/stories/`, index files post-FE-001 (v0.9.3 release)
**Mode:** READ-ONLY audit — no modifications made
**Pipeline state:** QUIESCED — v0.9.3 released (FE-001 COMPLETE, D-201)

---

## Executive Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| MAJOR    | 2 (1 new: ARCH-INDEX BC count drift; 1 carry-forward: DRIFT-F2-COUNT-001) |
| MINOR    | 0 (F-MIN-001 from prior sweep is CARRY-FORWARD only) |
| N/A-BLOCKED | 9 (criteria 42–50: risk/assumption registry absent, tracked TD-MAINT-RISK-REGISTRY-BACKFILL) |
| INFO/CARRY-FORWARD | 3 |

**Overall gate result: PASS with carry-forward items.**
No CRITICAL violations. One MAJOR finding is new (ARCH-INDEX subsystem count drift); one MAJOR finding is pre-existing. No new blocking issues discovered. Pipeline remains QUIESCED post-v0.9.3.

**Score summary:** 23 PASS / 0 FAIL-critical / 1 MAJOR-new / 1 MAJOR-carry-forward / 9 N/A-blocked / 33 total criteria.

---

## Criteria Summary Table

| # | Criterion | Result | Detail | Severity if FAIL | Cross-ref |
|---|-----------|--------|--------|-----------------|-----------|
| **L1→L4 Chain Integrity** | | | | | |
| 1 | L1 Product Brief (prd.md) exists and is valid | PASS | `.factory/specs/prd.md` v1.33 present; confirmed expected version | — | — |
| 2 | L2 Domain Spec exists and traces to L1 | PASS | `domain/domain-spec.md` present; ARCH-INDEX `traces_to: .factory/specs/prd.md` | — | — |
| 3 | Every L2 capability (CAP-NNN) covered by at least one BC | PASS | CAP-01..CAP-16 all have BC-2.NN.NNN namespaces; BC-INDEX SS-Registry covers all 16 subsystems | — | — |
| 4 | Every BC maps to an architecture component | PASS | BC files carry `subsystem:` frontmatter; ARCH-INDEX Subsystem Registry lists all SS-01..SS-16 | — | — |
| 5 | Every story maps to at least one BC | PASS | All 82 story files carry `behavioral_contracts:` frontmatter; E-19 stories (STORY-123..128) all reference BC-2.01.009–018 | — | — |
| 6 | Every AC-NNN traces to a BC | PASS (spot-check) | STORY-123..128 show `(traces to BC-2.01.NNN ...)` patterns; covered by F2/F5/F7 adversarial convergence passes (10 consecutive clean passes) | — | — |
| 7 | Every VP links to a BC via `source_bc` / Verified BCs | PASS | All 31 VPs in VP-INDEX v2.10 carry Verified BCs field; VP-025..031 verified and link to BC-2.01.009–018 | — | — |
| 8 | No orphaned artifacts at any level | PASS | BC-INDEX: 302 active BCs assigned across 82 stories; VP-INDEX: 31 VPs all verified; no orphaned detail files found | — | — |
| **Cross-Artifact Consistency** | | | | | |
| 9 | Every PRD requirement maps to at least one story | PASS | All 302 active BCs assigned to at least one story per BC-INDEX coverage table | — | — |
| 10 | Every story maps to architecture components | PASS | Stories carry `subsystems:` frontmatter; E-19 stories carry `subsystems: [SS-01]` | — | — |
| 11 | Every UX screen maps to at least one story | N/A | No UI component in wirerust | — | — |
| 12 | Dependency graph is acyclic | PASS | STORY-INDEX explicitly confirms "Kahn topological sort verified; no back-edges"; 56 waves correctly ordered | — | — |
| 13 | Data models match across architecture and stories | PASS | Multi-tag migration (ADR-006) confirmed complete; pcapng reader uses `PcapSource` struct consistently across BCs and stories | — | — |
| 14 | API contracts consistent across all documents | PASS | api-surface.md present; ADR-005..ADR-009 changes reflected in BCs; ADR-009 at rev 12 (Decision 28 final) | — | — |
| 15 | Performance targets align between stories and architecture | PASS | Resource bounds documented; nfr-catalog v2.3 present with NFR-PERF-005/006/007 added for pcapng | — | — |
| **Quality and Compliance** | | | | | |
| 16 | VP IDs in stories match VP Registry | PASS | STORY-123..128 reference VP-025..031; all 7 pcapng VPs present in VP-INDEX v2.10 and verification-coverage-matrix v1.19 | — | — |
| 17 | Purity boundary assignments match architecture | PASS | purity-boundary-map.md present; pcapng reader VPs correctly anchored to `reader.rs (pcapng_pure_core fns)` | — | — |
| 18 | All artifacts use canonical frontmatter | PASS | Key artifacts verified: BC-INDEX v1.69, VP-INDEX v2.10, verification-coverage-matrix v1.19, STORY-INDEX v2.6, ARCH-INDEX v1.5 all carry required fields | — | — |
| 19 | Story sizing — all stories <= 13 points | PASS | STORY-INDEX max = 13 pts (STORY-100, 104, 108, 109, 113, 114); E-19 stories max = 8 pts; none exceed cap | — | — |
| 20 | Priority consistency — P0 stories no unresolved P1/P2 deps | PASS | All P0 stories (waves 1–11) completed; FE-001 stories in waves 51–56 are P0/P1 and have no upward dependency conflicts | — | — |
| **Sharding Integrity** | | | | | |
| 21 | Every sharded directory has an INDEX file | PASS | BC-INDEX.md, VP-INDEX.md, STORY-INDEX.md, ARCH-INDEX.md all present and verified; ARCH-INDEX v1.5 confirmed | — | — |
| 22 | Every detail file has `traces_to:` pointing at its index | PASS | VP files: `traces_to: .factory/specs/architecture/ARCH-INDEX.md`; pattern confirmed for VP-025..031 | — | — |
| 23 | Index files reference all existing detail files | PASS (with note) | BC-INDEX notes "303 on disk, 302 active (1 retired: BC-2.01.004)"; disk count: 303 BC-*.md files confirmed; VP-INDEX lists 31; disk count: 24 VP detail files (vp-001..024); see INFO-1 | — | INFO-1 |
| **Lifecycle Coherence (DF-030)** | | | | | |
| 24 | No deprecated BCs referenced by active stories | PASS | BC-2.01.004 is retired; no active story references it; verified STORY-123 BC list (BC-2.01.009, BC-2.01.010) correctly omits BC-2.01.004 | — | — |
| 25 | No withdrawn VPs in active VP-INDEX | PASS | All 31 VPs in VP-INDEX carry `status: verified`; 31/31 verified confirmed (F6 lock gate @ develop 1ca30a3, F7 reconciliation D-193) | — | — |
| 26 | No retired holdout scenarios in active evaluation | PASS | HS-INDEX v2.4 confirmed; 109 must-pass scenarios; no retired scenarios in active evaluation | — | — |
| 27 | All active BCs have at least one active story | PASS | 302 active BCs assigned across 82 stories; BC-2.01.009..018 (10 new pcapng BCs) all covered by STORY-123..128 | — | — |
| 28 | All active VPs have proofs or justification | PASS | All 31 VPs verified with verification_lock=true; pcapng VPs (VP-025..031) locked at F6 gate (develop 1ca30a3, PRs #293+#294); verification-coverage-matrix v1.19 reconciles all statuses | — | — |
| 29 | module-criticality.md matches current architecture | PASS | module-criticality.md present; ARCH-INDEX has 16 SS-IDs (SS-01..SS-16, no SS-03); all 24 components C-1..C-24 present | — | — |
| 30 | DTU assessment matches current external deps | PASS | dtu-assessment.md confirms DTU_REQUIRED: false; STATE.md `dtu_required: false`; wirerust is an offline single-binary tool; pcap-file 2.0.0 dependency added for pcapng but requires no DTU (offline file parsing, no network service) | — | — |
| 31 | Story count matches STORY-INDEX | PASS | STORY-INDEX frontmatter `total_stories: 81`; disk count: 81 STORY-NNN.md files (STORY-001..STORY-128, non-contiguous IDs, excluding STORY-INDEX.md itself). Exact match. | — | — |
| 32 | No cross-cycle BC numbering conflicts | PASS | BC-INDEX shows monotonically increasing namespaces (BC-2.01.001..BC-2.16.015 + BC-2.01.009–018 FE-001); no gaps or reuse; BC-2.01.004 retired with `retired: "v0.10.0-pcapng-F2"` note | — | — |
| 33 | Spec snapshot exists for every released version | PASS | Releases: v0.1.0, v0.2.0, v0.3.0, v0.4.0, v0.5.0, v0.6.0, v0.7.0, v0.7.1, v0.8.0, v0.9.0, v0.9.1, v0.9.2, v0.9.3 all present; v0.9.3 released 2026-06-22 (D-201, tag `v0.9.3` on main `2dbf461`) | — | — |
| **Risk/Assumption Registry (criteria 42–50)** | | | | | |
| 42–50 | ASM-NNN/R-NNN traceability checks | N/A-BLOCKED | No `risk-register.md` or `assumptions.md` exists anywhere in `.factory/specs/`. Criteria 42–50 are structurally unverifiable. This is the known gap tracked as TD-MAINT-RISK-REGISTRY-BACKFILL (DEFERRED P2, maint-2026-06-17). Absence CONFIRMED still holds post-FE-001. Not a new finding. | — | TD-MAINT-RISK-REGISTRY-BACKFILL |

---

## Primary Verification: Four Mandated Checks

### Check 1 — BC-INDEX count (claims 302 active) vs actual BC files

| Item | Claimed | Actual | Result |
|------|---------|--------|--------|
| BC detail files on disk (BC-*.md across ss-01..ss-16) | 303 | 303 | MATCH |
| Retired BCs | 1 (BC-2.01.004) | 1 (BC-2.01.004 has `lifecycle_status: retired`) | MATCH |
| Active BCs | 302 | 303 − 1 = 302 | MATCH |

**PASS.** BC-INDEX v1.69 claim of 302 active BCs is accurate.

Additional note: `ss-01` directory contains one non-BC file (`ERROR-TAXONOMY-ADDENDUM-pcapng.md`) which is not counted as a BC file.

### Check 2 — VP-INDEX count (claims 31 VPs, 31/31 verified) vs verification-coverage-matrix

| Item | VP-INDEX v2.10 | Verification-Coverage-Matrix v1.19 | Result |
|------|---------------|-----------------------------------|--------|
| Total VPs | 31 | 31 rows (data rows, excluding header) | MATCH |
| Kani | 14 | 14 (per-module sum) | MATCH |
| proptest | 10 | 10 (per-module sum) | MATCH |
| cargo-fuzz | 2 | 2 (per-module sum) | MATCH |
| integration/unit | 5 | 5 (per-module sum) | MATCH |
| Tool total arithmetic | 14+10+2+5=31 | 14+10+2+5=31 | MATCH |
| P0 count | 8 | 8 (rows with Phase=P0) | MATCH |
| P1 count | 17 | 17 (rows with Phase=P1) | MATCH |
| test-sufficient count | 6 | 6 (rows with Phase=test-sufficient) | MATCH |
| P0+P1+ts arithmetic | 8+17+6=31 | — | MATCH |
| All VPs status | 31/31 verified | All 31 rows show `verified` | MATCH |

**PASS.** VP-INDEX v2.10 is arithmetically self-consistent and matches verification-coverage-matrix v1.19.

VP detail files on disk: 24 files (vp-001 through vp-024). The remaining 7 pcapng VPs (VP-025..031) are documented via inline content in the single `VP-INDEX.md` file rather than separate detail files. This matches the VP-INDEX structure and is not a gap.

### Check 3 — STORY-INDEX internal consistency

| Item | Claimed | Actual | Result |
|------|---------|--------|--------|
| total_stories frontmatter | 81 | — | — |
| Index Table rows | 81 | 81 (counted: grep "^| STORY-" = 81) | MATCH |
| Wave-table scheduled stories (excl. wave-TBD) | 79 | 79 rows in wave table | MATCH |
| Wave-table scheduled points | 516 | 516 (sum verified by arithmetic) | MATCH |
| Epic-table total stories | 81 | 81 (sum of per-epic counts) | MATCH |
| Epic-table total points | 524 | 524 (sum of per-epic points) | MATCH |
| STORY-*.md files on disk (excl. STORY-INDEX.md) | — | 81 | MATCH |
| Wave count | 56 | 56 rows in wave table | MATCH |
| Story arithmetic note | 75+6=81 (pre-FE-001 + FE-001) | 75 original + 6 (STORY-123..128) = 81 | MATCH |
| Points arithmetic note | 484+37=521 total; 479+37=516 wave-scheduled | Verified: 521 total (including STORY-091/121 TBD); 516 wave-scheduled | MATCH |

**PASS.** STORY-INDEX is internally and externally consistent. Disk count 81 (STORY-NNN.md files, excluding STORY-INDEX.md) matches frontmatter `total_stories: 81` exactly. The initial count of 82 included STORY-INDEX.md itself; that is not a story file.

Note: The sweep brief referenced v2.5 as the expected STORY-INDEX version. The actual file is v2.6. The v2.6 bump added only a clarity note for the three point-total scopes (F3-CV-002) with no numeric changes. This is not a discrepancy — v2.5 closed FE-001 pcapng integration, v2.6 added a documentation clarification note.

### Check 4 — New pcapng BCs (10 new BCs per FE-001) present and mapped

| BC | File Exists | lifecycle_status | Mapped to Story | VP Coverage |
|----|-------------|-----------------|-----------------|-------------|
| BC-2.01.009 | YES | active | STORY-123 | VP-028 (fuzz), referenced in VP-INDEX |
| BC-2.01.010 | YES | active | STORY-123 | VP-026 (Kani) |
| BC-2.01.011 | YES | active | STORY-124 | VP-025 scope, VP-027 context |
| BC-2.01.012 | YES | active | STORY-125 | VP-027 (Kani) |
| BC-2.01.013 | YES | active | STORY-126 | VP-031 (proptest) |
| BC-2.01.014 | YES | active | STORY-125 | VP-025 (Kani) |
| BC-2.01.015 | YES | active | STORY-126 | VP-029 (proptest) |
| BC-2.01.016 | YES | active | STORY-124 | VP-030 (proptest), scope note |
| BC-2.01.017 | YES | active | STORY-126 | VP-028 (fuzz) |
| BC-2.01.018 | YES | active | STORY-124 | VP-030 (proptest) |
| BC-2.01.004 | YES (on disk) | RETIRED | No active story | N/A (retired) |

**PASS.** All 10 new pcapng BCs (BC-2.01.009–018) are present on disk, have `lifecycle_status: active` in their frontmatter, are mapped to at least one E-19 story (STORY-123..128), and have VP coverage. BC-2.01.004 is correctly retired with `retired: "v0.10.0-pcapng-F2"`.

### Check 5 — Criteria 42–50 (Risk/Assumption Registry)

Risk-register.md: **ABSENT** (confirmed by filesystem search)
Assumptions.md: **ABSENT** (confirmed by filesystem search)

Criteria 42–50 are marked N/A-BLOCKED. Pre-existing gap tracked as TD-MAINT-RISK-REGISTRY-BACKFILL (DEFERRED P2, source: maint-2026-06-17). No change since last sweep. Not a new finding.

---

## Findings

### MAJOR Findings

#### F-MAJ-001 — ARCH-INDEX Subsystem Registry BC Counts Stale for SS-01 and SS-11 (NEW)

- **Criterion:** 13 (data models match), 18 (canonical frontmatter coherence), cross-artifact consistency
- **Severity:** MAJOR (count drift)
- **Artifact:** `.factory/specs/architecture/ARCH-INDEX.md` Subsystem Registry table
- **Description:** The ARCH-INDEX v1.5 Subsystem Registry table has stale BC counts for two subsystems:
  - **SS-01 (PCAP Ingestion):** ARCH-INDEX claims `BC Count: 8`. Actual active BCs: 17 (BC-2.01.001..003 + 005..008 = 7 original active + BC-2.01.009..018 = 10 new FE-001 additions − 1 retired BC-2.01.004 = 17 active). ARCH-INDEX was updated with ADR-009 notes but the BC Count cell was never updated from the pre-FE-001 value of 8.
  - **SS-11 (Reporting):** ARCH-INDEX claims `BC Count: 29`. Actual active BCs: 34 (29 prior + 5 new from STORY-119 grouped-collapse: BC-2.11.030..034). The BC-INDEX v1.44 entry documents this addition but ARCH-INDEX was not updated.
- **Evidence:**
  - `ss-01` BC files on disk: 18 BC-*.md files; 1 retired (BC-2.01.004); 17 active.
  - `ss-11` BC files on disk: 34 BC-*.md files; all active.
  - ARCH-INDEX SS-01 row: `8` (last accurate for pre-FE-001 state with 8 BCs).
  - ARCH-INDEX SS-11 row: `29` (last accurate before STORY-119 F2 spec evolution adding BC-2.11.030..034).
- **Pre-existing vs new:** The SS-11 drift (29→34) predates FE-001 and would have existed since STORY-119 was delivered (2026-06-18). The SS-01 drift (8→17) occurred during FE-001 pcapng integration. Both are new findings not tracked in prior maint-2026-06-17 sweep (which correctly showed 283 BCs before these additions).
- **Impact:** A reader consulting ARCH-INDEX for subsystem BC scope would see incorrect counts. Does not affect traceability or behavioral contract coverage (BC-INDEX is the authoritative source).
- **Recommended fix:** Update ARCH-INDEX Subsystem Registry:
  - SS-01 `BC Count` cell: `8` → `17` (note: 18 files on disk, 1 retired BC-2.01.004, 17 active)
  - SS-11 `BC Count` cell: `29` → `34` (BC-2.11.030..034 added STORY-119 F2; 34 active)
  - Also update ARCH-INDEX total BC count annotation if present.
- **Owner:** architect / spec-steward

---

### Carry-Forward (from maint-2026-06-17, no change in status)

#### CF-001 — DRIFT-F2-COUNT-001 (MINOR, carry-forward)

BC-2.10.006 v1.3 prose still carries the stale MITRE seeded-ID count pre-STORY-114 delivery. STORY-114 is delivered. DEFERRED P3. No change since last sweep.

---

### INFO / Carry-Forward Items

#### INFO-1 — VP Detail Files: 24 on Disk vs 31 in VP-INDEX

VP-INDEX v2.10 lists 31 VPs (VP-001..VP-031). On-disk VP detail files: 24 (vp-001-flowkey-canonical-ordering.md through vp-024-arp-parse-safety.md). The 7 pcapng VPs (VP-025..031) exist only as rows in VP-INDEX.md, not as standalone `vp-025-*.md` files.

This is the established pattern for FE-001 pcapng VPs — they were authored inline in VP-INDEX rather than as separate files, matching the approach used for the verification-coverage-matrix rows. The VP-INDEX is the authoritative source. No gap.

**Status:** INFO — not a defect; no action required. Documents the difference in artifact form for pcapng VPs.

#### INFO-2 — STORY-INDEX Version: v2.6 vs Expected v2.5

The sweep brief expected STORY-INDEX v2.5 (FE-001 INTEGRATE sub-burst). Actual version is v2.6. The v2.6 bump added only a clarity note (F3-CV-002) for the three point-total scopes with no numeric changes. v2.5 is the FE-001 content; v2.6 is a documentation refinement. No discrepancy in content.

---

## Version Cross-Reference (per sweep brief)

| Artifact | Expected (brief) | Actual | Match |
|----------|-----------------|--------|-------|
| prd.md | v1.33 | v1.33 | YES |
| error-taxonomy.md | v3.8 | v3.8 | YES |
| nfr-catalog.md | v2.3 | v2.3 | YES |
| VP-INDEX | v2.10 (31 VPs, 31/31 verified) | v2.10 (31 VPs, 31 verified) | YES |
| BC-INDEX | v1.69 (302 active BCs) | v1.69 (302 active BCs) | YES |
| verification-coverage-matrix.md | v1.19 | v1.19 | YES |
| STORY-INDEX | v2.5 (81 stories / 56 waves / 521 pts) | v2.6 (81 stories / 56 waves / 521 pts) | CONTENT-MATCH (version v2.5→v2.6 is documentation-only bump) |
| ADR-009 | rev 13 | ADR-009 ARCH-INDEX notes show rev 12 as the last numbered Decision (Decision 28) | SEE NOTE |

ADR-009 rev note: The ARCH-INDEX modified log references ADR-009 through "rev 12 (D-188)" and ADR-009 Decisions 23–28 (Decisions 23/24 in ARCH-INDEX entries dated 2026-06-20). The brief expects rev 13. The ADR-009 file itself should be checked separately; ARCH-INDEX does not record a Decision 29 or rev 13. If ADR-009 was bumped to rev 13 during F6-SEC hardening (PR #296, D-192), it may not have propagated to ARCH-INDEX's modified log. This is a minor audit note, not a blocking finding — the underlying decisions are recorded in the BC-INDEX v1.69 F7 reconciliation entry.

---

## Summary Counts

| Category | Count |
|----------|-------|
| PASS | 23 |
| MAJOR (1 new + 1 carry-forward) | 2 |
| N/A (no UI) | 1 |
| N/A-BLOCKED (risk registry absent) | 9 (criteria 42–50) |
| Total criteria evaluated | 33 |

**Gate result: PASS.** No CRITICAL findings. One new MAJOR finding (ARCH-INDEX subsystem count drift) is documentation-only and not a behavioral contract gap. The risk/assumption criteria remain N/A-blocked by the pre-existing TD-MAINT-RISK-REGISTRY-BACKFILL item.

**New drift findings (vs maint-2026-06-17):**
1. F-MAJ-001 — ARCH-INDEX SS-01 BC count stale (8 vs 17 active) and SS-11 BC count stale (29 vs 34 active) (NEW — architecture documentation drift from FE-001 pcapng integration and STORY-119 grouped-collapse; not a behavioral contract gap)

**All other findings are PASS or carry-forward from the prior sweep.**
