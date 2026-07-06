---
document_type: maintenance-sweep-report
sweep_id: spec-coherence-sweep-7
version: "2.0"
status: complete
producer: consistency-validator
date: 2026-07-06
trigger: maint-2026-07-06
timestamp: 2026-07-06T00:00:00Z
maintenance_run_id: maint-2026-07-06
sweep_type: spec-coherence
criteria_checked: 33
run_id: maint-2026-07-06
traces_to: .factory/STATE.md
---

# Maintenance Sweep 7 — Spec Coherence Report

**Date:** 2026-07-06
**Sweep:** Spec-coherence (33 criteria, DF-030)
**Scope:** `.factory/specs/`, `.factory/stories/`, index files
**Mode:** READ-ONLY audit — no modifications made
**Prior report:** maint-2026-06-17 (30/33 pass, 3 MAJOR + 4 MINOR + 5 INFO carry-forward)
**Pipeline state:** STEADY_STATE — v0.11.4 released; E-21 protocol-coverage cycle closed

---

## Overall Health: [HEALTHY / NEEDS_ATTENTION / DEGRADED]

Three new MAJOR findings discovered (F-NEW-MAJ-001: 10 phantom VP-INDEX entries with no files; F-NEW-MAJ-002: module-criticality.md frozen below current architecture; F-NEW-MAJ-003: BC-2.16.016 has no story). Three prior MAJOR findings persist. No CRITICAL findings. All implementation BCs are functionally covered; the failures are documentation sharding integrity and spec coverage gaps, not behavioral contract gaps.

| Metric | Value |
|--------|-------|
| Criteria passing | 30/33 |
| New MAJOR findings | 3 |
| Carry-forward MAJOR findings | 3 |
| New MINOR findings | 4 |
| Carry-forward MINOR findings | 4 |
| Blocking behavioral gaps | 0 |

---

## Dependency Audit

**Source:** `.factory/maintenance/dependency-audit.md` (run maint-2026-07-06)
**Result: CLEAN** — zero security advisories, zero cargo-deny errors.

| Severity | Count | Notes |
|----------|-------|-------|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 0 | All prior-cycle advisories cleared |
| LOW | 2 | FINDING-001 (license-not-encountered allowlist bloat, persists); FINDING-002 (syn v1/v2 duplicate, expected) |

Prior-cycle RUSTSEC-2026-0097 (rand), RUSTSEC-2026-0190 (anyhow), and zerocopy precautionary (DEP-005) are all confirmed cleared. The rayon dead dependency (DEP-006) was resolved. No new security items.

---

## Documentation Drift

**Source:** `.factory/maintenance/doc-drift.md` (run maint-2026-07-06)
**Result: NON-BLOCKING** — 8 findings (1 HIGH, 4 MEDIUM, 3 LOW); 13 of 14 prior items resolved or fixed.

| Severity | Count | Key Items |
|----------|-------|-----------|
| HIGH | 1 | ADR-0002 EtherNet/IP row still missing from Existing Analyzers table |
| MEDIUM | 4 | ADR documentation for SS-17/SS-18 subsystem additions; post-E-21 doc lag |
| LOW | 3 | Stale line references, minor ADR snippet drift |

Most prior HIGH findings (README ARP, README CSV, ADR-0002 multi-analyzer) were resolved earlier in the pipeline. Remaining HIGH is authoring-required (not auto-fixable).

---

## Pattern Consistency

**Source:** `.factory/maintenance/pattern-consistency.md` (run maint-2026-07-06)
**Result: NON-BLOCKING** — 20 findings (3 HIGH, 11 MEDIUM, 6 LOW); 8 new findings versus prior sweep.

| Severity | Count | Notes |
|----------|-------|-------|
| HIGH | 3 | PC-001/PC-002/PC-003 — all pre-existing carry-forwards (DNP3 StreamHandler trait conformance, PC-003 dropped counter) |
| MEDIUM | 11 | 8 new findings (PC-013..PC-020) plus 3 carry-forward |
| LOW | 6 | All carry-forward |

No new HIGH findings. The 8 new MEDIUM/LOW findings are pattern divergences introduced by E-20/E-21 feature cycles (ENIP analyzer and protocols.rs module). Per DF-VALIDATION-001, deferred from GitHub issue filing pending research-agent validation.

---

## Holdout Scenario Freshness

**Source:** `.factory/maintenance/holdout-freshness.md` (run maint-2026-07-06)
**Result: NON-BLOCKING** — 4 stale scenarios, 0 retired, 0 FAIL-BUG.

| Metric | Count |
|--------|-------|
| Concrete HS files | 132 |
| Active | 127 |
| Stale | 4 (HS-061, HS-064, HS-066, HS-075) |
| Retired | 0 |
| FAIL-BUG | 0 |

Stale scenarios have assertions outdated by v0.11.4 output changes (new `dropped_map_entries` key in HTTP/TLS, `mitre_attack_version`/`mitre_domain` top-level JSON keys). These are intentional product changes, not regressions. Scenario updates are a product-owner backlog item before the next Phase-4 holdout evaluation run. See F-NEW-MIN-002 in spec coherence findings for full detail.

---

## Performance Baseline

**Source:** `.factory/maintenance/performance-baseline.md` (last run maint-2026-06-22)
**Note:** Performance sweep was not re-executed for maint-2026-07-06 (spec-coherence-only run). The performance-baseline.md reflects v0.9.3 state from maint-2026-06-22.

| Benchmark | vs May-19 baseline | Status |
|-----------|-------------------|--------|
| decode/tls.pcap | +12.2% | REGRESSION-MINOR (stable) |
| reassembly/segmented.pcap | +19.4% | REGRESSION-MINOR (stable) |
| All other benchmarks | < 10% | NOISE |
| Prior CRITICAL (reassembly/tls.pcap +54.5%) | Retracted | Confirmed thermal noise |

Both REGRESSION-MINOR findings are attributable to the `DecodedFrame::Arp` match variant added in the ARP feature cycle. Stable across runs. No NFR latency target exists; informational only. A fresh performance run against v0.11.4 is recommended for the next full maintenance sweep given the E-20/E-21 protocol additions since v0.9.3.

---

## Trend (Last 5 Sweeps)

| Run | Version | Criteria Pass | New MAJOR | Carry-Forward MAJOR | FAIL-BUG | Gate |
|-----|---------|--------------|-----------|---------------------|----------|------|
| maint-2026-06-17 | v0.7.1 | 30/33 | 3 | 0 | 0 | NON-BLOCKING |
| maint-2026-06-22 | v0.9.3 | 30/33 (23 pass + 9 N/A-blocked) | 1 (ARCH-INDEX BC count drift; resolved same run) | 1 | 0 | PASS |
| maint-2026-07-06 | v0.11.4 | **30/33** | **3** | **3** | **0** | **NEEDS_ATTENTION** |

**Trend observations:**
- The pass rate (30/33) has held constant across three sweeps despite a 22% increase in active BCs (283→345) and 54% increase in stories (70→108). The L1→L4 chain integrity and sharding criteria are structurally sound.
- Carry-forward MAJOR findings have accumulated: 0 → 1 → 3. None have been resolved between sweeps. The three new MAJOR findings this run increase total open MAJOR count to 6.
- FAIL-BUG count remains 0 across all sweeps — behavioral correctness is solid.
- The VP file sharding gap (F-NEW-MAJ-001) is the most structurally significant new finding; it has been growing silently as VP-INDEX added VPs without corresponding files (VP-025..031 in the FE-001 pcapng cycle, VP-041..043 in E-21).

---

## Spec Coherence Criteria Summary Table

| # | Criterion | Result | Notes |
|---|-----------|--------|-------|
| **L1→L4 Chain Integrity** | | | |
| 1 | L1 Product Brief exists and is valid | PASS | `prd.md` v1.51 present; ARCH-INDEX traces to it |
| 2 | L2 Domain Spec exists and traces to L1 | PASS | `domain-spec.md` present; CAP-18 registered; ARCH-INDEX traces |
| 3 | Every L2 capability covered by at least one BC | PASS | CAP-01..CAP-18 all have BC namespaces including SS-18 (Protocol Coverage Catalog) |
| 4 | Every BC maps to an architecture component | PASS | All BCs carry `subsystem:` frontmatter; SS-17 and SS-18 registered in ARCH-INDEX Subsystem Registry |
| 5 | Every story maps to at least one BC | PASS | All 108 story files carry `behavioral_contracts:` frontmatter; E-21 stories spot-checked |
| 6 | Every AC-NNN traces to a BC | PASS | No change in methodology; per-story spot-check pattern consistent |
| 7 | Every VP links to a BC via `source_bc` | PASS | All 43 VPs in VP-INDEX carry `source_bc` field; VP-041/042/043 carry BC-2.18.003/004 and BC-2.05.010/011 |
| 8 | No orphaned artifacts at any level | **FAIL** | 10 VP-INDEX entries reference non-existent VP files: VP-025..031 (verified/locked, 7 missing) and VP-041/042/043 (draft, 3 missing). See F-NEW-MAJ-001. |
| **Cross-Artifact Consistency** | | | |
| 9 | Every PRD requirement maps to at least one story | PASS | All 18 CAPs have stories; tally stale at 335 vs. 345 active BCs (see F-NEW-MIN-001) |
| 10 | Every story maps to architecture components | PASS | Stories carry `subsystems:` frontmatter |
| 11 | Every UX screen maps to at least one story | N/A | No UI component in wirerust |
| 12 | Dependency graph is acyclic | PASS | STORY-INDEX confirms "Kahn topological sort verified; no back-edges"; E-21 waves 67→69 acyclic |
| 13 | Data models match across architecture and stories | PASS | SS-17/SS-18 data models reflected in BCs and ARCH-INDEX |
| 14 | API contracts consistent across all documents | PASS | ADR-012 Consequences corrected (F5 reconciliation); `run_protocols` signature anchored |
| 15 | Performance targets align between stories and architecture | PASS | Resource bounds documented; SS-18 HashMap bound noted in ARCH-INDEX |
| **Quality and Compliance** | | | |
| 16 | VP IDs in stories match VP Registry | PASS | STORY-151 `[VP-041]`, STORY-153 `[VP-042/VP-043]` match VP-INDEX entries |
| 17 | Purity boundary assignments match architecture | PASS | `purity-boundary-map.md` present; SS-18 protocols.rs is pure-core |
| 18 | All artifacts use canonical frontmatter | MINOR | F-MAJ-001 ("68-story" parenthetical), F-MAJ-002 (VP-024 version label lag), F-NEW-MIN-003 (stories_delivered=98 vs. actual 93), F-NEW-MIN-004 (story-body propagation pending) |
| 19 | Story sizing — all stories ≤ 13 points | PASS | Max 13 pts observed; no story exceeds cap |
| 20 | P0 priority consistency | PASS | All P0 stories (waves 1–11) delivered |
| **Sharding Integrity** | | | |
| 21 | Every sharded directory has an INDEX file | PASS | BC-INDEX.md, VP-INDEX.md, STORY-INDEX.md, HS-INDEX.md, ARCH-INDEX.md all present |
| 22 | Every detail file has `traces_to:` pointing at its index | PASS | Spot-checked; pattern consistent |
| 23 | Index files reference all existing detail files | PASS | All 33 existing VP files are referenced in VP-INDEX; BC-INDEX references all 346 on disk; criterion 8 covers phantom VP-INDEX entries |
| **Lifecycle Coherence (DF-030)** | | | |
| 24 | No deprecated BCs referenced by active stories | PASS | No active story references BC-ABS-004..009 or BC-2.01.004 (all retired) |
| 25 | No withdrawn VPs in active VP-INDEX | PASS | All 43 VPs are active/verified/draft; none withdrawn |
| 26 | No retired holdout scenarios in active evaluation | PASS | No `lifecycle_status: retired` in any HS file; 4 `stale` scenarios (HS-061/064/066/075) noted separately |
| 27 | All active BCs have at least one active story | **FAIL** | BC-2.16.016 (fix-pc-013-014-015) has no story; BC-INDEX v1.72 notes "story-writer propagation pending." See F-NEW-MAJ-003. |
| 28 | All active VPs have proofs or justification | PASS | All 43 VP proof descriptions present in VP-INDEX; file absence addressed by criterion 8 |
| 29 | module-criticality.md matches current architecture | **FAIL** | module-criticality.md frozen at C-24; current architecture has C-25 (EnipAnalyzer/SS-17) and C-26 (protocols.rs/SS-18). See F-NEW-MAJ-002. |
| 30 | DTU assessment matches current external deps | PASS | `dtu-assessment.md` v1.0 `DTU_REQUIRED: false`; SS-17/SS-18 are offline-only |
| 31 | Story count matches STORY-INDEX | PASS | 108 STORY-*.md files on disk = STORY-INDEX `total_stories: 108` — EXACT MATCH |
| 32 | No cross-cycle BC numbering conflicts | PASS | SS-17 (001..026), SS-18 (001..004) use non-overlapping namespaces; total 346 on disk, 345 active |
| 33 | Spec snapshot for every released version | PASS | v0.1.0..v0.11.4 (20 tags) all present on repo |

---

## Findings

### NEW MAJOR Findings

#### F-NEW-MAJ-001 — VP-INDEX Has 10 Phantom Entries With No Corresponding VP Files

- **Criterion:** 8 (no orphaned artifacts)
- **Severity:** MAJOR
- **Artifact:** `.factory/specs/verification-properties/` directory; `VP-INDEX.md`
- **Description:** VP-INDEX v2.34 declares `total_vps: 43`. On disk there are 33 VP files (vp-001..vp-024 = 24 files; vp-032..vp-040 = 9 files). Missing as standalone files:
  - **VP-025 through VP-031** (7 pcapng VPs, `verified`, locked at develop 1ca30a3): vp-025-pcapng-timestamp-totality, vp-026-pcapng-shb-parse-safety, vp-027-pcapng-epb-parse-safety, vp-028-pcapng-reader-no-panic, vp-029-pcapng-block-walk-skip, vp-030-pcapng-multi-idb-agreement, vp-031-pcapng-spb-captured-len.
  - **VP-041, VP-042, VP-043** (3 E-21 protocol-coverage VPs, `draft`): vp-041-protocol-coverage-catalog-set-difference, vp-042-dispatcher-unclassified-flow-count, vp-043-udp-decode-loop-unclassified-count.
  - VP-INDEX File Naming Convention states: "VP files: `vp-NNN-<short-slug>.md` ... All VP files reside in `.factory/specs/verification-properties/`."
  - VP-025..031 descriptions are embedded inline in VP-INDEX; VP-041..043 appear in catalog table only.
- **Impact:** Sharding integrity violation. Any tooling iterating VP files by directory listing misses 10 VPs. Proof evidence for VP-025..031 is only in VP-INDEX narrative.
- **Status:** NEW — not in prior sweep.
- **Recommended fix:** Create 10 missing VP files from the inline descriptions in VP-INDEX; follow vp-032..040 file structure as template.
- **Owner:** spec-steward

---

#### F-NEW-MAJ-002 — module-criticality.md Frozen at C-24; Misses C-25 (EnipAnalyzer) and C-26 (protocols.rs)

- **Criterion:** 29 (module-criticality.md matches current architecture)
- **Severity:** MAJOR
- **Artifact:** `.factory/specs/module-criticality.md` (frozen v1.5)
- **Description:** module-criticality.md is frozen per `frozen_reason: "Phase 5 long passed. Frozen by spec-steward at Phase-6 gate close per module-criticality lifecycle rule (MUTABLE through Phase 5)."` The file covers C-1..C-24 only. Current architecture has 26 components:
  - C-25: EnipAnalyzer (SS-17, feature-enip-v0.11.0) — added to ARCH-INDEX v1.7, never added to module-criticality.md.
  - C-26: protocols.rs (SS-18, feature-protocol-coverage) — added to ARCH-INDEX v2.6, never added to module-criticality.md.
  - ARCH-INDEX v2.12 updated Document Map description aspirationally to "for all 26 components" but the actual file has 24.
  - `grep "C-25\|C-26\|SS-18"` in module-criticality.md: zero results.
- **Impact:** Criticality assessment and mutation-testing guidance will miss EnipAnalyzer and protocols.rs — both are HIGH-criticality components.
- **Status:** NEW — C-25/C-26 shipped after prior sweep.
- **Recommended fix:** Decision required: (a) lift freeze for additive post-Phase-5 additions; or (b) formally document permanent scope gap and revert ARCH-INDEX Document Map to "for all 24 components."
- **Owner:** spec-steward (freeze policy decision) / architect (C-25/C-26 tier assessment)

---

#### F-NEW-MAJ-003 — BC-2.16.016 Has No Active Story (story-writer Propagation Pending)

- **Criterion:** 27 (all active BCs have at least one active story)
- **Severity:** MAJOR
- **Artifact:** `.factory/specs/behavioral-contracts/ss-16/BC-2.16.016.md`; STORY-INDEX Coverage Verification
- **Description:** BC-2.16.016 ("ARP Findings Output is Unbounded — No MAX_FINDINGS Cap on process_arp Return Vec") was added in BC-INDEX v1.72 (fix-pc-013-014-015 bundle, D-221). BC-INDEX notes: "No stories modified (story-writer propagation pending under bc_array_changes_propagate_to_body_and_acs policy)." STORY-INDEX Coverage Verification only lists "BC-2.16.001..015 in STORY-111..115" — BC-2.16.016 uncovered.
- **Impact:** No TDD/F4 holdout path for BC-2.16.016. Red Gate test `test_BC_2_16_016_arp_findings_vec_has_no_cap` has no story to drive it.
- **Status:** NEW — BC-2.16.016 added after prior sweep.
- **Recommended fix:** Add BC-2.16.016 to STORY-115 (or a new story). Update Coverage Verification tally. Per DF-VALIDATION-001 before filing GitHub issue.
- **Owner:** story-writer / product-owner

---

### NEW MINOR Findings

#### F-NEW-MIN-001 — STORY-INDEX Coverage Verification Tally Stale (335 vs. 345 Active BCs)

- **Criterion:** 27 (auxiliary), 9 (auxiliary)
- **Severity:** MINOR
- **Description:** STORY-INDEX Coverage Verification tally says "total 335 BCs" but BC-INDEX v2.18 says 345 active. The 10-BC gap: BC-2.01.009..018 (10 E-19 pcapng BCs) covered by STORY-123..128 but not in tally; BC-2.16.016 (1 fix-pc-013-014-015) not covered; BC-2.01.004 retired (net −1). Net 10 uncounted active BCs in coverage note.
- **Recommended fix:** Update Coverage Verification to enumerate BC-2.01.009..018 (+10) and BC-2.16.016 (+1); correct total 335→345.
- **Owner:** story-writer

---

#### F-NEW-MIN-002 — Four Stale Holdout Scenarios Not Updated for v0.11.4

- **Criterion:** 26 (adjacent — stale, not retired)
- **Severity:** MINOR
- **Description:** HS-INDEX v2.11 marks HS-061 (HTTP 9→10 keys), HS-064/HS-075 (JSON 3→5 top-level keys), HS-066 (TLS 7→10 keys) as `stale`. Assertions outdated for v0.11.4 `dropped_map_entries` observability counters and MITRE envelope. These will produce spurious FAIL-STALE in Phase-4 holdout evaluation against v0.11.4+.
- **Recommended fix:** Update the 4 scenario assertion text before any Phase-4 holdout run.
- **Owner:** product-owner

---

#### F-NEW-MIN-003 — STORY-INDEX stories_delivered Counter Off by 5

- **Criterion:** 18 (canonical content coherence)
- **Severity:** MINOR
- **Description:** STORY-INDEX v3.13 comment says `stories_delivered=98`. Manual count: 82 completed + 11 merged = 93 delivered. The off-by-5 is persistent (v3.8 also overcounts: says 94 after wave 66, actual 89 at that point). The frontmatter `total_stories: 108` and per-row status cells are authoritative and correct.
- **Recommended fix:** Correct stories_delivered to 93.
- **Owner:** story-writer

---

#### F-NEW-MIN-004 — BC-INDEX v2.17/v2.18 Silent-Limit Audit Story-Body Propagation Outstanding

- **Criterion:** 5 (auxiliary), 18 (canonical content)
- **Severity:** MINOR
- **Description:** BC-INDEX v2.17/v2.18 added observability counters to BC-2.16.010 (ARP binding evictions, 13 keys), BC-2.14.012/021 (Modbus dropped_transactions), BC-2.07.031 (TLS dropped_map_entries), BC-2.06.023 (HTTP dropped_map_entries). BC-INDEX explicitly flags: STORY-113/115 ("eleven keys" → "thirteen"), STORY-046/058/103/104 (BC tables and ACs need new counter keys). Story bodies describe fewer observable keys than their BCs.
- **Recommended fix:** Story-writer to propagate new keys into STORY-113, STORY-115, STORY-046, STORY-058, STORY-103, STORY-104. Rebaseline input-hashes after.
- **Owner:** story-writer

---

### MAJOR Findings — Carry-Forward from maint-2026-06-17

| ID | Description | Status |
|----|-------------|--------|
| F-MAJ-001 | STORY-INDEX "68-story" parenthetical stale (now v3.14 / 108 stories; further outdated) | STILL OPEN |
| F-MAJ-002 | STATE.md VP-024 version label v2.3 vs v2.4 (DRIFT-E17-VERSIONLABEL-LAG-001) | STILL OPEN |
| F-MAJ-003 | epics.md structural debt: subsystem table stale (now shows ~12 of 18 subsystems) | STILL OPEN (further outdated) |

---

### MINOR Findings — Carry-Forward from maint-2026-06-17

| ID | Description | Status |
|----|-------------|--------|
| F-MIN-001 | DRIFT-F2-COUNT-001: BC-2.10.006 stale seeded count (15→25) | STILL OPEN |
| F-MIN-002 | DRIFT-BC-2.15.024-EC006-PROSE-001: EC-006 vs. BC-2.15.009 PC5 conflict | STILL OPEN |
| F-MIN-003 | DRIFT-E16-BC-BACKLINK-GAP-001: BC-2.16.009/015 missing STORY-114/115 backlinks | STILL OPEN |
| F-MIN-004 | DRIFT-VP024-BTREEMAP-PROSE-001: VP-024 feasibility assessment references BTreeMap | STILL OPEN |

---

### INFO / Carry-Forward Items (5 total, all persisting from maint-2026-06-17)

| ID | Description | Status |
|----|-------------|--------|
| INFO-1 | DRIFT-MITRE-EMITTED-LABEL-001: Kani EMITTED_IDS T0835/T0831 over-label | STILL OPEN |
| INFO-2 | DRIFT-SUPERPOWERS-001: docs/superpowers/ pre-F2 catalog stale | STILL OPEN |
| INFO-3 | Non-ARP STALE input-hashes pre-existing | STILL OPEN (non-blocking) |
| INFO-4 | DRIFT-BC-INPUTHASH-TBD-001: SS-15 BC files carry input-hash:TBD | STILL OPEN (by design) |
| INFO-5 | VP-INDEX version freeze (not bumped for E-17, by design) | STILL OPEN (by design) |

---

## Detailed Criterion Verification

### BC File Counts by Subsystem (vs BC-INDEX claims)

| SS | Files on Disk | Active | BC-INDEX Active | Match |
|----|--------------|--------|-----------------|-------|
| SS-01 | 18 (+ 1 ERROR-TAXONOMY-ADDENDUM) | 17 (BC-2.01.004 retired) | 17 | YES |
| SS-02 | 15 | 15 | 15 | YES |
| SS-04 | 55 | 55 | 55 | YES |
| SS-05 | 11 | 11 | 11 | YES |
| SS-06 | 26 | 26 | 26 | YES |
| SS-07 | 43 | 43 | 43 | YES |
| SS-08 | 4 | 4 | 4 | YES |
| SS-09 | 7 | 7 | 7 | YES |
| SS-10 | 9 | 9 | 9 | YES |
| SS-11 | 35 | 35 | 35 | YES |
| SS-12 | 24 | 24 | 24 | YES |
| SS-13 | 4 | 4 | 4 | YES |
| SS-14 | 25 | 25 | 25 | YES |
| SS-15 | 24 | 24 | 24 | YES |
| SS-16 | 16 | 16 | 16 | YES |
| SS-17 | 26 | 26 | 26 | YES |
| SS-18 | 4 | 4 | 4 | YES |
| **Total** | **346** | **345** | **345** | **YES** |

BC-INDEX claim: 346 on disk, 345 active (1 retired). VERIFIED.

### VP File Counts (Criterion 8 failure detail)

| VP Range | Status | Files on Disk | VP-INDEX Entries | Gap |
|----------|--------|--------------|------------------|-----|
| VP-001..024 | verified | 24 | 24 | 0 |
| VP-025..031 | verified/locked | **0** | 7 | **7 MISSING** |
| VP-032..040 | draft | 9 | 9 | 0 |
| VP-041..043 | draft | **0** | 3 | **3 MISSING** |
| **Total** | — | **33** | **43** | **10 MISSING** |

### Story File Count (Criterion 31)

| Source | Count | Match |
|--------|-------|-------|
| STORY-*.md on disk (excluding STORY-INDEX.md) | 108 | — |
| STORY-INDEX `total_stories` frontmatter | 108 | YES |

### Git Release Tags (Criterion 33)

All 20 released versions (v0.1.0, v0.2.0, v0.3.0, v0.4.0, v0.5.0, v0.6.0, v0.7.0, v0.7.1, v0.8.0, v0.9.0, v0.9.1, v0.9.2, v0.9.3, v0.9.4, v0.10.0, v0.11.0, v0.11.1, v0.11.2, v0.11.3, v0.11.4) have git tags. VERIFIED.

---

## Summary

| Category | Count |
|----------|-------|
| Criteria PASS (including N/A) | 30 |
| Criteria FAIL | 3 (criteria 8, 27, 29) |
| Criteria total | 33 |
| New MAJOR findings | 3 |
| New MINOR findings | 4 |
| Carry-forward MAJOR | 3 |
| Carry-forward MINOR | 4 |
| Carry-forward INFO | 5 |

**Gate result: NEEDS_ATTENTION.** No CRITICAL findings. No behavioral regressions (FAIL-BUG = 0). Three new MAJOR findings require disposition before the next F3 spec evolution pass. Three prior MAJOR findings remain unresolved across three consecutive sweeps; escalation recommended.
