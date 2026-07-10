---
document_type: maintenance-sweep-report
sweep_id: spec-coherence-sweep-7
version: "3.0"
status: complete
producer: consistency-validator
date: 2026-07-09
trigger: maint-2026-07-09
timestamp: 2026-07-09T12:00:00Z
maintenance_run_id: maint-2026-07-09
sweep_type: spec-coherence
criteria_checked: 33
run_id: maint-2026-07-09
traces_to: .factory/STATE.md
spec_versions:
  BC-INDEX: v2.22
  VP-INDEX: v2.39
  ARCH-INDEX: v2.12
  PRD: v1.51
  STORY-INDEX: v3.33
  epics: v2.1
---

# Maintenance Sweep 7 — Spec Coherence Report

**Date:** 2026-07-09
**Sweep:** Spec-coherence (33 criteria, DF-030)
**Scope:** `.factory/specs/`, `.factory/stories/`, index files
**Mode:** READ-ONLY audit — no modifications made
**Prior report:** maint-2026-07-08 (0 FAIL, 5 LOW admin findings)
**Pipeline state:** wave-72 CLOSED; maint-2026-07-09 STARTED — v0.11.5 released

---

## Summary

| Sweep | Status | Findings | PRs Opened | Issues Created |
|-------|--------|----------|-----------|----------------|
| Dependency Audit | **CLEAN** | 0 new; 2 pre-existing LOW (DEP-006/007, deferred) | 0 | 0 |
| Documentation Drift | **2 DEFERRED** | Route B deferred (NEW-002/003); see maint-2026-07-08 | 0 | 0 |
| Pattern Consistency | **CLEAN** | Clippy 0 warnings; pre-existing carry-forwards only | 0 | 0 |
| Holdout Freshness | **21/21 PASS (CLEAN)** | 0 stale; 132 active | 0 | 0 |
| Performance Baseline | **NOISE-SUSPECT** | PERF-RERUN-001 open (controlled re-run required) | 0 | 0 |
| Spec Coherence (7) | **PASS** | 0 FAIL criteria; 1 new LOW SC-001; 9 carry-forward | 0 | 0 |

---

## Overall Health: [HEALTHY / NEEDS_ATTENTION / DEGRADED]

**Assessment: HEALTHY**

All three prior FAIL criteria (C8 / C27 / C29) are resolved. Zero FAIL criteria this run. Zero behavioral regressions (FAIL-BUG = 0). All wave-72 spec deltas (VP-INDEX v2.39 Multi-File Proof Anchor Algorithm, VP-024 v2.5 proof_file_hash, STORY-162 draft) landed coherently. Open items are LOW/MINOR admin drift items deferred by prior human decision (Route C) or pending story-writer propagation. No blocking issues at sweep close.

---

## Dependency Audit

**Source:** `.factory/maintenance/dependency-audit.md` (maint-2026-07-08 most recent run; no new audit this pass)
**Result: CLEAN** — zero security advisories.

### New Vulnerabilities

| Dependency | Version | CVE/Advisory | Severity | Fix Available | Action |
|-----------|---------|-------------|----------|--------------|--------|
| — | — | None | — | — | No new advisory since maint-2026-07-08 |

### Hygiene Findings (LOW, pre-existing)

| ID | Finding | Action |
|----|---------|--------|
| DEP-006 | Unused license allowlist entries in cargo-deny config | Maintenance backlog (deferred) |
| DEP-007 | syn v1/v2 duplicate dependency in tree | Maintenance backlog (expected) |

No new dependency findings. indicatif 0.18.5 → 0.18.6 (Dependabot PR #386, squash-merged 2026-07-09, D-417) introduces no advisories.

---

## Documentation Drift

**Source:** `.factory/maintenance/doc-drift.md` (maint-2026-07-08 run)
**Result: NON-BLOCKING** — Route B deferred items outstanding; no new doc-drift findings this pass.

### Stale Documentation

| Document | Section | Drift Type | Severity | Action |
|----------|---------|-----------|----------|--------|
| README.md | `--coverage-gaps` flag description | Flag undocumented (NEW-002 from maint-2026-07-08) | LOW | Route B deferred (batch into next chore/docs PR) |
| ADR-0001 | STORY-153 fields snippet | Missing STORY-153 `input` fields (NEW-003 from maint-2026-07-08) | LOW | Route B deferred (batch into next chore/docs PR) |

Note: ADR-012 public-doc authoring (NEW-001 maint-2026-07-08) was addressed via STORY-159 (PR #388, merged wave-72, 2026-07-09). That finding is RESOLVED.

---

## Pattern Consistency

**Source:** `.factory/maintenance/pattern-consistency.md` (maint-2026-07-08 run)
**Result: CLEAN this pass** — Clippy: 0 warnings on develop HEAD 716054a. PF-001 (109 `+=` → `saturating_add`) resolved by PR #384 (maint-2026-07-08). Pre-existing carry-forward HIGH findings (PC-001/PC-002/PC-003) unchanged.

### Inconsistencies Detected

| Pattern | Expected | Found In | Severity | Action |
|---------|----------|----------|----------|--------|
| PC-001: StreamHandler trait conformance | Consistent StreamHandler impl | DNP3 (pre-existing carry-forward) | HIGH | DF-VALIDATION-001-gated |
| PC-002: findings-import style | Consistent use-import | Multiple files (pre-existing) | HIGH | DF-VALIDATION-001-gated |
| PC-003: dropped_findings counter | All analyzers | DNP3 missing (pre-existing; NOTE: BC-2.15.022 v1.5 now adds `dropped_findings` PC-5 per BC-INDEX v2.22 — story-body propagation pending STORY-108) | HIGH | DF-VALIDATION-001-gated |

No new pattern-consistency findings from wave-72 deliveries.

---

## Holdout Scenario Freshness

**Source:** `.factory/maintenance/holdout-freshness.md` (maint-2026-07-09 sweep 4 run)
**Result: CLEAN** — 0 stale scenarios. Prior F-NEW-MIN-002 (4 stale: HS-061/064/066/075) RESOLVED.

| Metric | Value |
|--------|-------|
| Total scenarios on disk | 132 |
| Active (`lifecycle_status: active`) | 132 |
| Stale (`lifecycle_status: stale`) | **0** |
| Retired | 0 |
| Missing coverage (features with 0 scenarios) | 0 |

HS-064/075 updated for `schema_version` JSON envelope and enum-casing changes (STORY-160, BC-2.11.036/037). HS-061/066 verified against v0.11.5 output. All 132 scenarios now pass freshness check. Prior FAIL-BUG = 0 maintained.

---

## Performance Baseline

**Source:** `.factory/maintenance/performance-baseline.md` (maint-2026-07-08 run; noise-suspect)
**Result: NOISE-SUSPECT** — PERF-RERUN-001 outstanding; controlled re-run required before treating as regression anchor.

| Benchmark | Previous (maint-2026-06-22) | maint-2026-07-08 | Delta | Status |
|-----------|------------|---------|-------|--------|
| reassembly/tls.pcap | 23.281 µs | 26.353 µs | +13.2% | REGRESSION-MINOR (stable across runs; attributable to ARP DecodedFrame variant) |
| decode/tls.pcap | baseline | +12.2% | — | REGRESSION-MINOR (stable) |
| All other benchmarks | — | within ±10% | — | NOISE |

Note: maint-2026-07-08 run showed 11–20 severe outliers per 100 samples (same thermal-noise fingerprint as June-17). The STORY-150 TLS drain-loop refactor impact verified at +1.8% (within noise, p=0.37). PERF-RERUN-001 deferred; no new perf evidence this pass.

---

## Spec Coherence

Sweep 7 (spec-coherence). **33/33 checks PASS (including N/A).** 0 FAIL. Improvement from maint-2026-07-06 (3 FAIL) and maint-2026-07-08 (0 FAIL, 5 LOW).

---

### Input Hash Scan Result

```
bin/compute-input-hash --scan
MATCH=115  STALE=0
```

All 115 stories have correct input-hashes under the canonical Python tool.

---

### Artifact Counts (PROP-MAINT-03 — from index files)

| Artifact | Disk Count | Index Claim | Match |
|----------|-----------|-------------|-------|
| STORY-*.md (excl. INDEX) | 115 | STORY-INDEX `total_stories: 115` | YES |
| VP files (vp-*.md) | 43 | VP-INDEX `total_vps: 43` | YES |
| BC files (active) | 347 | BC-INDEX v2.22: 348 on disk, 347 active | YES |
| Release tags (v-prefix) | 21 | v0.1.0..v0.11.5 | YES |

---

### Wave-72 Spec Delta Coherence Verification

| Delta | Artifact | Result |
|-------|----------|--------|
| VP-INDEX v2.35 → v2.39 | Multi-File Proof Anchor Algorithm + EC-005 bidirectional cross-links | COHERENT |
| VP-024 v2.4 → v2.5 | proof_file_hash = `48296b21a5bbce59750e6210da8d55be8bf7d3d4a1ed6719088dd4ef59a2c8a5`; verification_lock:true | COHERENT |
| STORY-162 draft | E-11, wave-TBD, 3 pts, behavioral_contracts:[]; input-hash MATCH | COHERENT |
| STORY-INDEX v3.33 wave-72 row | DELIVERED & CLOSED (D-416, human-approved) | COHERENT |
| BC-INDEX v2.22 | BC-2.11.036/037 added; STORY-INDEX tally updated 335→337 | COHERENT (persistent 10-BC arithmetic gap — see SC-PERSIST-001) |

---

### 33-Criterion Results Table

| # | Criterion | Result | Notes |
|---|-----------|--------|-------|
| 1 | L1 Product Brief exists and is valid | **PASS** | `prd.md` v1.51; ARCH-INDEX traces |
| 2 | L2 Domain Spec exists and traces to L1 | **PASS** | `domain-spec.md`; CAP-18; ARCH-INDEX traces |
| 3 | Every L2 capability covered by ≥1 BC | **PASS** | CAP-01..CAP-18 all have BC namespaces |
| 4 | Every BC maps to architecture component | **PASS** | All BCs carry `subsystem:` frontmatter |
| 5 | Every story maps to ≥1 BC | **PASS** | All 115 stories carry `behavioral_contracts:` frontmatter; E-11 pattern ([] by design) accepted |
| 6 | Every AC-NNN traces to BC | **PASS** | Per-story spot-check consistent; wave-72 verified |
| 7 | Every VP links to BC via `source_bc` | **PASS** | All 43 VPs carry `source_bc` field |
| 8 | No orphaned artifacts | **PASS** | **43 VP files on disk = VP-INDEX total_vps:43. F-NEW-MAJ-001 RESOLVED.** |
| 9 | Every PRD req maps to ≥1 story | **PASS** | All 18 CAPs covered; tally arithmetic note (SC-PERSIST-001) |
| 10 | Every story maps to arch components | **PASS** | Stories carry `subsystems:` frontmatter |
| 11 | Every UX screen maps to ≥1 story | **N/A** | No UI component |
| 12 | Dependency graph is acyclic | **PASS** | Kahn sort verified; 115 stories, no back-edges |
| 13 | Data models match across architecture and stories | **PASS** | SS-17/18 reflected; BC-2.11.036/037 consistent with STORY-160 |
| 14 | API contracts consistent | **PASS** | ADR-012 corrected F5; `run_protocols` signature anchored |
| 15 | Performance targets align | **PASS** | Resource bounds documented |
| 16 | VP IDs in stories match VP Registry | **PASS** | VP-041/042/043 files verified; VP-024 v2.5 cited in STORY-112/113/161 |
| 17 | Purity boundary assignments match architecture | **PASS** | `purity-boundary-map.md` present; SS-18 pure-core |
| 18 | All artifacts use canonical frontmatter | **MINOR** | SC-PERSIST-002 (stories_delivered off by 5); SC-PERSIST-003 (BC body propagation pending 5 stories) |
| 19 | Story sizing ≤13 pts | **PASS** | Max 8 pts in wave-72 |
| 20 | P0 priority consistency | **PASS** | All P0 stories delivered |
| 21 | Every sharded directory has INDEX file | **PASS** | BC/VP/STORY/HS/ARCH-INDEX all present |
| 22 | Every detail file has `traces_to:` → index | **PASS** | Spot-checked; pattern consistent |
| 23 | Index files reference all existing detail files | **PASS** | 43 VP files all referenced; 115 stories all in catalog |
| 24 | No deprecated BCs in active stories | **PASS** | No active story references retired BCs |
| 25 | No withdrawn VPs in active VP-INDEX | **PASS** | All 43 VPs active/verified/draft |
| 26 | No retired holdout scenarios in active evaluation | **PASS** | **0 stale scenarios. F-NEW-MIN-002 RESOLVED.** |
| 27 | All active BCs have ≥1 active story | **PASS** | **BC-2.16.016 covered by STORY-156 (PR #378). F-NEW-MAJ-003 RESOLVED.** |
| 28 | All active VPs have proofs or justification | **PASS** | VP-024 v2.5 proof_file_hash populated; verification_lock:true |
| 29 | module-criticality.md matches current architecture | **PASS** | **v1.6 includes C-25/C-26. F-NEW-MAJ-002 RESOLVED.** |
| 30 | DTU assessment matches current external deps | **PASS** | `dtu_required: false`; all SS offline-only |
| 31 | Story count matches STORY-INDEX | **PASS** | 115 on disk = STORY-INDEX `total_stories: 115` |
| 32 | No cross-cycle BC numbering conflicts | **PASS** | BC-2.11.036/037 non-conflicting; 347 active |
| 33 | Spec snapshot for every released version | **PASS** | 21 tags v0.1.0..v0.11.5 verified |

---

### BC File Counts by Subsystem (BC-INDEX v2.22)

| SS | Files on Disk | Active | BC-INDEX Active | Match |
|----|--------------|--------|-----------------|-------|
| SS-01 | 19 (1 retired on disk) | 17 | 17 | YES |
| SS-02 | 15 | 15 | 15 | YES |
| SS-04 | 8 | 8 | 8 | YES |
| SS-05 | 11 | 11 | 11 | YES |
| SS-06 | 18 | 18 | 18 | YES |
| SS-07 | 43 | 43 | 43 | YES |
| SS-08 | 14 | 14 | 14 | YES |
| SS-09 | 15 | 15 | 15 | YES |
| SS-10 | 17 | 17 | 17 | YES |
| SS-11 | 37 | 37 | 37 | YES |
| SS-12 | 24 | 24 | 24 | YES |
| SS-13 | 10 | 10 | 10 | YES |
| SS-14 | 19 | 19 | 19 | YES |
| SS-15 | 22 | 22 | 22 | YES |
| SS-16 | 16 | 16 | 16 | YES |
| SS-17 | 9 | 9 | 9 | YES |
| SS-18 | 4 | 4 | 4 | YES |
| **Total** | **348** | **347** | **347** | **YES** |

---

### New Findings (maint-2026-07-09)

#### SC-001 — STORY-INDEX Registry Header Description Omits Wave-72 Stories

- **Criterion:** 18 (auxiliary)
- **Severity:** LOW
- **Artifact:** `.factory/stories/STORY-INDEX.md` line 99 (registry header narrative blockquote)
- **Description:** The narrative registry description lists stories through STORY-157 but omits STORY-158, STORY-159, STORY-160, STORY-161, and STORY-162 (5 wave-72 / wave-TBD stories added since the description was last updated). The authoritative numeric fields (`total_stories: 115`, wave-table, epic-table) are all correct; only the narrative text is incomplete.
- **Impact:** Informational gap only; the narrative is not machine-parsed.
- **Recommended fix:** Extend line 99 narrative to include wave-72 additions in a future maintenance pass.
- **Owner:** story-writer

---

### Prior Findings Disposition

#### From maint-2026-07-06 NEW MAJOR

| Finding ID | Description | Disposition |
|-----------|-------------|-------------|
| F-NEW-MAJ-001 | VP-INDEX 10 phantom entries (VP-025..031, VP-041..043 missing files) | **RESOLVED** — 43 VP files on disk, 0 gap |
| F-NEW-MAJ-002 | module-criticality.md frozen at C-24; missing C-25/C-26 | **RESOLVED** — module-criticality.md v1.6 adds C-25/C-26; freeze policy amended |
| F-NEW-MAJ-003 | BC-2.16.016 has no active story | **RESOLVED** — STORY-156 (wave 71, PR #378, merged 2026-07-08) delivers BC-2.16.016 |

#### From maint-2026-07-06 MAJOR carry-forwards

| Finding ID | Description | Disposition |
|-----------|-------------|-------------|
| F-MAJ-001 | STORY-INDEX "68-story" parenthetical stale | **RESOLVED** — no such parenthetical present in STORY-INDEX v3.33 |
| F-MAJ-002 | STATE.md VP-024 version label v2.3 vs v2.4 | **RESOLVED** — VP-024 now v2.5 (STORY-161, wave-72); STATE.md correctly records "VP-024 v2.5" |
| F-MAJ-003 | epics.md structural debt (total_bcs stale, subsystem table stale) | **PERSISTS** — total_bcs=337 vs 347 active (gap +2 since maint-2026-07-08); story count 107 vs 115 (gap +4 since maint-2026-07-08); deferred Route C |

#### From maint-2026-07-06 NEW MINOR

| Finding ID | Description | Disposition |
|-----------|-------------|-------------|
| F-NEW-MIN-001 | STORY-INDEX tally 335 vs 345 BCs | **PERSISTS** as SC-PERSIST-001 — now 337 vs 347, gap=10; deferred Route C |
| F-NEW-MIN-002 | 4 stale holdout scenarios (HS-061/064/066/075) | **RESOLVED** — 0 stale scenarios; all 132 active |
| F-NEW-MIN-003 | STORY-INDEX stories_delivered counter off by 5 | **PERSISTS** as SC-PERSIST-002 — counter=106, actual=101; off by 5 |
| F-NEW-MIN-004 | BC body propagation pending (STORY-046/058/103/104) | **PERSISTS AND EXTENDED** as SC-PERSIST-003 — STORY-108 added by BC-INDEX v2.22 DNP3 amendments |

#### From maint-2026-07-06 MINOR carry-forwards

| Finding ID | Description | Disposition |
|-----------|-------------|-------------|
| F-MIN-001 | DRIFT-F2-COUNT-001: BC-2.10.006 seeded count stale ("25 after STORY-114 PLANNED") | **PERSISTS** — STORY-114 is delivered; count should read "25 current" |
| F-MIN-002 | DRIFT-BC-2.15.024-EC006-PROSE-001: EC-006 references "PC5-6", correct is "PC3" | **PERSISTS** — spec-wording only, no behavioral gap |
| F-MIN-003 | DRIFT-E16-BC-BACKLINK-GAP-001: BC-2.16.009/015 missing STORY-114/115 backlinks | **PERSISTS** — BC-2.16.009 Stories: {STORY-113, STORY-116, STORY-117}; BC-2.16.015 Stories: {STORY-112, STORY-116, STORY-117} |
| F-MIN-004 | DRIFT-VP024-BTREEMAP-PROSE-001: VP-024 feasibility table says "BTreeMap with 8 entries maximum" | **PERSISTS** — VP-024 v2.5 Input Space Size table Sub-D row still says "BTreeMap"; should say "array surrogate CAP=8" |

#### From maint-2026-07-08

| Finding ID | Description | Disposition |
|-----------|-------------|-------------|
| STORY-INDEX-TALLY-DRIFT-001 | Tally 335 vs 345 BCs | **PERSISTS** — 337 vs 347 now; gap held at 10 |
| EPICS-STORY-COUNT-DRIFT-001 | epics.md count 107 vs 111 actual | **PERSISTS** — 107 vs 115 now; gap grew 4→8; deferred Route C |
| INPUT-HASH-DESCRIPTION-DRIFT-001 | STATE.md backlog description inaccurate | **RESOLVED** — per maint-2026-07-08 DF-VALIDATION-001 triage |
| STATE-BACKLOG-STALE-STORY-148-001 | STATE.md backlog row OPEN after D-399 | **RESOLVED** — per maint-2026-07-08 triage |
| STATE-BACKLOG-STALE-F-F3P18-O1 | STATE.md F-F3P18-O1 row OPEN after D-375 | **RESOLVED** — per maint-2026-07-08 triage |

#### INFO carry-forwards

| Finding ID | Description | Disposition |
|-----------|-------------|-------------|
| INFO-1 | DRIFT-MITRE-EMITTED-LABEL-001: Kani EMITTED_IDS T0835/T0831 over-label | STILL OPEN |
| INFO-2 | DRIFT-SUPERPOWERS-001: docs/superpowers/ pre-F2 catalog stale | STILL OPEN |
| INFO-3 | Non-ARP STALE input-hashes pre-existing | **RESOLVED** — MATCH=115, STALE=0 |
| INFO-4 | DRIFT-BC-INPUTHASH-TBD-001: SS-15 BC files carry input-hash:TBD (21 files) | STILL OPEN — by design |
| INFO-5 | VP-INDEX version freeze (not bumped for E-17, by design) | **SUPERSEDED** — VP-INDEX actively maintained at v2.39 |

---

## Trend (Last 5 Sweeps)

| Run | Version | Criteria Pass | FAIL | FAIL-BUG | Stale Holdouts | Gate |
|-----|---------|--------------|------|----------|----------------|------|
| maint-2026-06-17 | v0.7.1 | 30/33 | 3 | 0 | 4 | NEEDS_ATTENTION |
| maint-2026-06-22 | v0.9.3 | 30/33 | 1 (resolved same run) | 0 | 4 | PASS |
| maint-2026-07-06 | v0.11.4 | 30/33 | 3 (C8/C27/C29) | 0 | 4 | NEEDS_ATTENTION |
| maint-2026-07-08 | v0.11.5 | 33/33 | 0 | 0 | 0* | PASS |
| maint-2026-07-09 | v0.11.5 | 33/33 | 0 | 0 | 0 | **PASS** |

\* maint-2026-07-08 didn't run Sweep 4 independently; stale holdouts addressed via STORY-160 deliverable in wave-72.

**Trend observations:**
- The three maint-2026-07-06 FAIL criteria (C8/C27/C29) are all resolved this run. No FAIL criteria remain.
- All three prior MAJOR findings (F-NEW-MAJ-001/002/003) resolved by wave-71/72 deliverables.
- Holdout scenarios went from 4 stale (maint-2026-07-06) to 0 stale. FRESHNESS: CLEAN.
- The LOW admin drift items (epics.md staleness, STORY-INDEX tally arithmetic) persist but have been explicitly deferred Route C by human decision.
- FAIL-BUG = 0 across all 5 sweeps.
