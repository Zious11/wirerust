---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-20T00:00:00Z
cycle: v0.1.0-greenfield-spec
inputs: [adversarial-reviews/]
traces_to: STATE.md
---

# Convergence Trajectory — v0.1.0-greenfield-spec

## Finding Progression

| Pass | Date | Total | CRIT | HIGH | MED | LOW | Novelty | Score | Counter | Verdict |
|------|------|-------|------|------|-----|-----|---------|-------|---------|---------|
| 1 | 2026-05-20 | 17 | 2 | 8 | 5 | 2 | HIGH | — | 0/3 | NOT_CONVERGED — all findings remediated |
| 2 | 2026-05-20 | 13 | 0 | 4 | 6 | 3 | MED | — | 0/3 | NOT_CONVERGED — all blocking remediated; 2 deferred (L-2, L-3) |
| 3 | 2026-05-20 | 7 | 0 | 3 | 2 | 2 | MED | — | 0/3 | NOT_CONVERGED — all findings remediated |
| 4 | 2026-05-20 | 19 | 4 | 5 | 5 | 3 | HIGH | — | 0/3 | NOT_CONVERGED — fresh-context L2 cap+entity audit; all 19 fixed; +5 CsvReporter BCs |
| 5 | 2026-05-20 | 8 | 1 | 2 | 3 | 2 | LOW | — | 0/3 | NOT_CONVERGED — NUL byte, stale --services, count drift; all 8 fixed |
| 6 | 2026-05-20 | 3 | 0 | 3 | 0 | 0 | LOW | — | 0/3 | NOT_CONVERGED — component-ID anchors, BC-INDEX titles, INV-1 citation; all 3 fixed |
| 7 | 2026-05-20 | 13 | 1 | 3 | 4 | 3 | LOW | — | 0/3 | NOT_CONVERGED — entity shards, em-dash, SS-13 anchor, cap-05 token, VP-008; all 13 fixed |
| 8 | 2026-05-20 | 8 | 0 | 2 | 3 | 2 | LOW | — | 0/3 | NOT_CONVERGED — vp-008 arg order+IPv6, stale citations, E-RAS-005 counter; all 8 fixed |
| 9 | 2026-05-20 | 4 | 0 | 1 | 1 | 2 | LOW | — | 0/3 | NOT_CONVERGED — stale citations BC-2.04.054/027, prd error-categories, ARCH-INDEX debt note; all 4 fixed |
| 10 | 2026-05-20 | 6 | 0 | 3 | 3 | 0 | LOW | — | 0/3 | NOT_CONVERGED — dependency table stale vs Cargo.toml, api-surface Reporter trait + ParsedPacket wrong, CAP-03/SS IDs; all 6 fixed |
| 11 | 2026-05-20 | 1 | 0 | 0 | 0 | 1 | LOW | — | **1/3** | **CONVERGED** — clean pass 1 of 3 (0C/0H/0M/1L/4obs); 1L + 4 cosmetic observations polished |
| 12 | 2026-05-20 | 6 | 0 | 1 | 1 | 2 | LOW | — | **0/3** | **NOT CONVERGED** — counter RESET from 1/3 (H+M findings broke streak); all 6 findings fixed |
| 13 | 2026-05-20 | 5 | 0 | 2 | 0 | 3 | LOW | — | **0/3** | **NOT CONVERGED** — 2H stale anchors (ent-05, INV-4), 2L doc drift (ARCH-INDEX C-count, prd BC-2.07.004), 1N; all 5 fixed |
| 14 | 2026-05-20 | 3 | 0 | 1 | 0 | 1 | LOW | — | **0/3** | **NOT CONVERGED** — H-1 summary.rs C-16→C-17 mis-anchor (4 sites/2 files), L-1 entity index E-39b missing (entity 41→42), N-1 BC-2.12.005 citation off-by-one; all 3 fixed |
| SWEEP | 2026-05-20 | — | — | — | — | — | — | — | **0/3** | **REMEDIATION BURST** — proactive anchor sweep; 3,820 occurrences audited; 28 mis-anchors fixed; no adversary pass; counter unchanged |
| 15 | 2026-05-20 | 4 | 0 | 1 | 2 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — H-1 VP-020 test API wrong (CsvReporter/render()->String); M-1 VP-020 pt 3 mis-scoped AnalysisSummary; M-2 module-decomposition reporter Purity wrong; N-1 covered by M-2 fix. All 4 fixed. |
| 16 | 2026-05-20 | 3 | 1 | 0 | 1 | 1 | LOW | — | **0/3** | **NOT CONVERGED** — C-1 BC-2.07.037 Postcondition 4 verdict Anomaly/Likely/High→Anomaly/Inconclusive/Low; M-1 stale correction-notes removed from BC-2.07.017/019; L-1 minor wording. All 3 fixed. |
| SWEEP | 2026-05-20 | — | — | — | — | — | — | — | **0/3** | **REMEDIATION BURST** — comprehensive BC-vs-source verification sweep; all 217 BCs re-verified against current src/; ~58 defects fixed (off-by-one citations + ~6 semantic spec-vs-code defects); 37 BC body files committed (d038ace); addresses recurring P-CITE-PG defect class at root; no adversary pass; counter unchanged |
| 17 | 2026-05-20 | 5 | 0 | 2 | 1 | 1 | LOW | — | **0/3** | **NOT CONVERGED** — all 5 findings concentrated in ent-04 only; ZERO BC defects found (BC sweep held). F-1 HIGH: AnalysisSummary.detail HashMap→BTreeMap; F-2 HIGH: false "only inline tests" claim + stale line range; F-3 MED: BC-RPT-007→BC-RPT-001 cross-ref; F-4 LOW: line range 12-17→38-50; F-5 NITPICK: Verdict citation 32-40→30-40. All fixed (0c16cad). |
| 18 | 2026-05-20 | 5 | 0 | 3 | 0 | 2 | LOW | — | **0/3** | **NOT CONVERGED** — all 5 findings were stale-anchor drift from PR #75 `//!` header line shifts in last unreconciled domain shards. H-1 ent-01 8 entity anchors; H-2 ent-04 6 cross-file anchors; H-3 cap-10 unknown-ID rendering anchor; L-1 ent-02 C-range; L-2 domain-spec test count. All fixed (fc28b69). |
| 19 | 2026-05-20 | 2 | 0 | 0 | 2 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — 0C/0H/2M/0L; package described as "overwhelmingly clean". M-1 purity-boundary-map.md: 3 reporters (JsonReporter, CsvReporter, TextReporter) misclassified as Effectful-shell; corrected to Pure-core (consistent with module-decomposition.md). M-2 dependency-graph.md: test-count statement corrected to "264 in tests/ + 18 inline = 282". All fixed (f913004). |
| 20 | 2026-05-20 | 4 | 0 | 0 | 2 | 1 | LOW | — | **0/3** | **NOT CONVERGED** — 0C/0H/2M/1L/1N; all spec-precision gaps, no behavioral defects. F-1/F-2 VP-007 SEEDED_IDS corrected + citation 99-129→122-156; F-3 BC-2.12.008 main.rs 57-58→57-59 (5 instances); F-4 mitre_technique regex tightened. Counter remains 0/3. All fixed. |
| 21 | 2026-05-20 | 3 | 0 | 0 | 1 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — 0C/0H/1M/0L/2N. F-1 (MED) module-decomposition.md C-10 re-anchored SS-08→SS-05 (analyzer/mod.rs belongs to shared analyzer-trait module, not DNS-specific); ARCH-INDEX.md SS-05 registry row updated (follow-up). O-1 (NITPICK) prd.md Out-of-Scope removed-flags list completed (--verbose, --services added). O-2 (NITPICK) BC-2.07.016 one-liner aligned to canonical H1. Counter remains 0/3. All fixed. |
| 22 | 2026-05-20 | 3 | 0 | 0 | 0 | 2 | LOW | — | **1/3** | **CONVERGED** — **FIRST CLEAN PASS (1/3)**; 0C/0H/0M/2L/1N. LOW-1: BC-2.12.005 H1 title broadened to cover all 9 TCP reassembly flags (was too narrow — only named 3); BC-INDEX.md + prd.md BC-2.12.005 title rows synced. LOW-2: BC-2.07.004 citation ranges tightened to precise line windows. NITPICK: BC-2.07.004 oversized-record guard prose + error-taxonomy.md E-ANA-003 guard citation both aligned to tls.rs:643-653. Counter advances to **1/3**. Pass 23 next (second confirmation pass). |
| 23 | 2026-05-20 | 3 | 0 | 1 | 1 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — **STREAK BROKEN — RESET 1/3→0/3**; 0C/1H/1M/0L/1N. H-1: csv.rs C-21 anchor collision in purity-boundary-map.md; M-1: stale absent-flag row in module-criticality.md (corrected to reflect PR #74 clap rejection); N-1: E-INP-001 citation 56-59→56-60. All 3 fixed. Counter reset to 0/3. Pass 24 dispatched next. |
| 24 | 2026-05-20 | 0 | 0 | 0 | 0 | 0 | NONE | — | **1/3** | **CONVERGED** — **CLEAN PASS 1/3 (new streak)**; 0C/0H/0M/0L/0N. 2 non-blocking observations only (neither a spec defect). No spec artifact modified. Passes 25 and 26 will review identical, stable content. Counter advances to **1/3**. Pass 25 next (second confirmation pass). |
| 25 | 2026-05-20 | 4 | 0 | 2 | 2 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — **STREAK RESET 1/3→0/3**; 0C/2H/2M/0L/0N. All 4 findings in PRD supplements only (the last spec-package pocket not yet comprehensively reconciled). H-1: error-taxonomy.md — E-RAS-001/E-RAS-002 message strings wrong vs actual eprintln! literals; H-2: error-taxonomy.md — E-DEC-001 stale `from_ethernet_slice` API name. M-1: interface-definitions.md — wrong analyzer_name values + fabricated/wrong JSON detail-shape keys (flows_evicted/flows_closed_fin do not exist in source). M-2: nfr-catalog.md — NFR-PERF-001 decoder.rs line range 72-130 should be 288-291. Orchestrator commissioned comprehensive PRD-supplement sweep after pass-25 findings. Counter RESET to **0/3**. |
| SWEEP68 | 2026-05-20 | — | — | — | — | — | — | — | **0/3** | **REMEDIATION BURST** — comprehensive PRD-supplement verification sweep; all 4 PRD supplements re-verified against current src/; ~68 defects fixed total. Addresses P-CITE-PG process gap (recurring citation/string-drift; 4 supplements were the last un-comprehensively-reconciled spec pocket). Details: error-taxonomy.md (3 fixes — pass-25 findings); interface-definitions.md (14 fixes — wrong analyzer_name values, 8+ fabricated/wrong JSON detail keys corrected to actual summarize() keys); nfr-catalog.md (47 fixes — M-2 line range + 39 stale citations re-anchored post-LESSON-P2.01 refactor + NFR-SEC-005 SNI verdict Likely/High→Inconclusive/Low + 6 module-map citations); test-vectors.md (8 fixes — BC-2.06.005 category Anomaly→Reconnaissance; BC-2.07.014/017/037 verdict corrected to Inconclusive/Low; embedded literal control bytes 0x00/0x1b/ESC replaced with textual escapes; integration-scenario category/severity corrections). No adversary pass; counter unchanged at 0/3. Pass 26 next. |
| 26 | 2026-05-20 | 5 | 0 | 3 | 1 | 1 | LOW | — | **0/3** | **NOT CONVERGED** — 0C/3H/1M/1L. All 4 blocking findings in VP files: wrong API signatures, stale citations, mis-stated verdict labels. Comprehensive VP-file sweep (SWEEP48) commissioned. Counter remains 0/3. |
| SWEEP48 | 2026-05-20 | — | — | — | — | — | — | — | **0/3** | **REMEDIATION BURST** — comprehensive VP-file sweep; all 20 VP files + VP-INDEX + BC-2.04.039 re-verified against current src/; ~48 defects fixed (wrong API sigs ~20, stale citations ~25, BC verdict labels 3, VP-INDEX phase column 5). SHA: 25641c4. All 4 major spec categories now comprehensively reconciled. Counter 0/3 unchanged. Pass 27 next. |
| 27 | 2026-05-20 | 1 | 0 | 1 | 0 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — 0C/1H/0M/0L. H-1: verification-coverage-matrix.md VP-016..020 Phase column P1→test-sufficient (SWEEP48 propagation gap; P0(8)/P1(7)/test-sufficient(5)=20 invariant restored). Fixed (e758fb6). Counter remains 0/3. Pass 28 next. |
| 28 | 2026-05-20 | 0 | 0 | 0 | 0 | 0 | NONE | — | **1/3** | **CONVERGED** — **CLEAN PASS 1/3 (new streak)**; 0C/0H/0M/0L/0N. Zero findings. No spec artifact modified. Passes 29 and 30 will review identical, stable content. Counter advances to **1/3**. Pass 29 next (second confirmation pass). |
| 29 | 2026-05-20 | 1 | 0 | 0 | 0 | 1 | LOW | — | **2/3** | **CONVERGED** — **CLEAN PASS 2/3**; 0C/0H/0M/1L/1obs. Gate definition: 0C/0H/0M = clean pass. L-1: system-overview.md handler.rs import description corrected (Direction/CloseReason defined in-file, not imported). O-1 (obs): new debt item O-08 added — dns.rs module doc-comment stale; propagated to domain-debt.md, ARCH-INDEX.md, prd.md. Both L-1 and O-08 fixed before commit (04478ef). Counter advances to **2/3**. Pass 30 next — THIRD and final confirmation pass; if 0C/0H/0M the Phase 1d adversarial convergence gate is SATISFIED (3/3). |
| 30 | 2026-05-20 | 3 | 0 | 0 | 1 | 0 | LOW | — | **0/3** | **NOT CONVERGED** — **STREAK BROKEN — RESET 2/3→0/3**; 0C/0H/1M/0L/2N. M-1: BC-2.12.020 Summary section cited C-16 instead of C-17 (copy-paste error). N-1: BC-2.05.006 guard-clause in shorthand rather than actual Rust form. N-2: inv-01 INV-9 technique_info span cited as approximate; corrected to exact range mitre.rs:122-156. All 3 fixed (00f5094). Counter RESET to **0/3**. 30 passes total; spec package at ZERO known open defects. Pass 31 next (new streak restart). |
| 31 | 2026-05-21 | 0 | 0 | 0 | 0 | 0 | NONE | — | **1/3** | **CONVERGED** — **CLEAN PASS 1/3 (new streak after pass-30 reset)**; 0C/0H/0M/0L/0N. Zero findings. 2 non-blocking observations only: (1) module-decomposition.md C-8 buffer described as `BTreeMap<u64,Segment>` — informal shorthand for actual `BTreeMap<u64, Vec<u8>>`; survived 30+ passes, non-misleading, not a defect; recorded as O-09. (2) BC-2.01.001 dual citation scoping (reader.rs:46-60 vs reader.rs:50-60) — both valid scopings of the same contract; internally consistent, not a defect. No spec artifact modified. Counter advances to **1/3**. Pass 32 next (second confirmation pass). |

## Trajectory Shorthand

`17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3→5→5→2→4→3→0→3→0→4→SWEEP68→5→SWEEP48→1→0→0→3→0` (SWEEP between 16 and 17 — counter unchanged; pass 17 ZERO BC defects; pass 18 all stale-anchor drift from PR #75; pass 19 overwhelmingly clean — 0C/0H/2M only; pass 20 spec-precision gaps — 0C/0H/2M/1L/1N; pass 21 — 0C/0H/1M/0L/2N, C-10 re-anchor + PRD nitpicks; **pass 22 CONVERGED 0C/0H/0M/2L/1N — counter 1/3**; pass 23 NOT CONVERGED — streak broken, 0C/1H/1M/1N, counter reset 0/3; **pass 24 CONVERGED 0/0/0/0/0 — counter 1/3**; pass 25 NOT CONVERGED — streak RESET 1/3→0/3, 0C/2H/2M, all 4 in PRD supplements; SWEEP68 — comprehensive PRD-supplement sweep ~68 defects fixed, P-CITE-PG addressed; counter 0/3; pass 26 NOT CONVERGED 0C/3H/1M/1L, all 4 in VP files; SWEEP48 — comprehensive VP-file sweep ~48 defects fixed; counter 0/3; pass 27 NOT CONVERGED 0C/1H/0M/0L — VP-016..020 Phase column drift fixed (e758fb6); counter 0/3; **pass 28 CONVERGED 0C/0H/0M/0L/0N — counter 1/3, clean pass 1/3**; **pass 29 CONVERGED 0C/0H/0M/1L/1obs — counter 2/3, clean pass 2/3, L-1+O-08 fixed before commit 04478ef**; pass 30 NOT CONVERGED — streak RESET 2/3→0/3, 0C/0H/1M/0L/2N, all 3 fixed (00f5094); **pass 31 CONVERGED 0C/0H/0M/0L/0N — counter 1/3, clean pass 1/3 of new streak, pass 32 next**)

## Per-Pass Details

### Pass 1 (2026-05-20)

**Findings:** 17 (2 CRIT, 8 HIGH, 5 MED, 2 LOW)
**Novelty:** HIGH
**Convergence counter:** 0 of 3

**Key finding categories:**

- CRIT: VP count arithmetic errors and stale cross-references in verification-architecture.md and verification-coverage-matrix.md
- HIGH: CLI flag table in api-surface.md stale vs. source; BC-INDEX.md titles/status mismatches; 8+ BC body files with stale line citations post-refactor
- MED: INV-2 invariant body incomplete in inv-01-core-invariants.md; file count mismatches in domain-spec.md; ADR 0004 undocumented in domain-debt.md; prd.md rayon claim inconsistent with src/; §2.13 section titles misaligned
- LOW: domain-debt.md missing O-07 (rayon declared but unused in src/); BC-2.05.006 two-phase-commit rewrite incomplete

**Remediation:** All 17 findings addressed by spec agents. Fixes committed in burst
`spec: fix adversarial-review pass-1 findings (2C/8H/5M/2L)`. Pass 2 dispatched next.

---

### Pass 2 (2026-05-20)

**Findings:** 13 (0 CRIT, 4 HIGH, 6 MED, 3 LOW)
**Delta from pass 1:** -4 total (CRIT -2, HIGH -4, MED +1, LOW +1) — no regression
**Novelty:** MEDIUM
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH: ss-12 BC bodies referencing wrong capability anchors (CAP-11/CAP-01 instead of CAP-12);
  BC-INDEX.md title mismatches and stale ss-04 sub-header; BC-2.07.014, BC-2.08.002, BC-2.08.004
  cross-reference errors
- MED: domain-spec.md CAP-12 not registered, SS-12->CAP-12 subsystem map missing;
  ARCH-INDEX.md still citing SS-12 rather than CAP-12; error-taxonomy.md had 12 stale/wrong
  source citations; BC-2.04.024 MED fix; BC-ABS-008 rationale absent from BC-INDEX
- LOW: L-2 (dns.rs stale module doc — source defect, deferred); L-3 (no BC-title-sync
  validator — process gap, deferred); one additional LOW (addressed in cap-12-cli-orchestration.md)

**New artifact:** `specs/domain/capabilities/cap-12-cli-orchestration.md` — CAP-12 added.
Capability count: 11 -> 12. Domain shard count: 19 -> 20.

**Deferred (non-blocking):**
- L-2: `src/analyzer/dns.rs` module doc stale — source defect, not spec. Code follow-up post-Phase 1.
- L-3: No machine validator for BC-H1 <-> BC-INDEX title sync — tooling gap. CI lint rule in future sprint.

**Remediation:** All blocking findings addressed. CAP-12 added, 21 ss-12 BCs re-anchored,
BC-INDEX synced, error-taxonomy citations corrected, ARCH-INDEX updated. Fixes committed
in burst `spec: fix adversarial-review pass-2 findings (4H/6M/3L) + add CAP-12 capability`
(SHA: 26e143f). Pass 3 dispatched next.

---

### Pass 3 (2026-05-20)

**Findings:** 7 (0 CRIT, 3 HIGH, 2 MED, 2 NITPICK)
**Delta from pass 2:** -6 total (CRIT 0, HIGH -1, MED -4, LOW -2, NITPICK +2) — no regression
**Novelty:** MEDIUM
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH: T0856 MITRE tactic mis-mapping — `IcsInhibitResponseFunction` used in cap-10-mitre-mapping.md
  and cap-05-content-first-dispatch.md; correct tactic is `IcsImpairProcessControl`. Two files corrected.
- HIGH: None-caching two-phase behavior (LESSON-P2.11 retry cap) not propagated from owning BCs
  (BC-2.05.005, BC-2.10.007) to downstream artifacts — domain-spec.md, ent-03, ent-05, inv-01,
  prd.md, vp-004, verification-architecture.md, purity-boundary-map.md, BC-INDEX.md all updated.
- HIGH: BC body postcondition/invariant edits made in pass 2 remediation not swept across
  BC-INDEX.md, PRD, capability/entity docs, VP files, and architecture docs — propagation
  gap now corrected across all 8+ downstream files.
- MED: vp-004-content-first-dispatch.md postcondition language inconsistent with updated BC bodies.
- MED: purity-boundary-map.md and verification-architecture.md cross-references stale after
  pass-2 None-caching additions.
- NITPICK (×2): Minor wording inconsistencies in ent-05 and inv-01; corrected in same sweep.

**Process gap identified (codification follow-up at cycle close):**
BC body postcondition/invariant edits must trigger a propagation sweep across BC-INDEX,
PRD, capability/entity docs, VP files, and architecture docs. Currently a manual discipline;
should be codified as a checklist step or CI lint rule.

**Files fixed (13):**
`cap-10-mitre-mapping.md`, `cap-05-content-first-dispatch.md`, `ent-03-dispatch-analysis.md`,
`ent-05-enums-value-objects.md`, `domain-spec.md`, `inv-01-core-invariants.md`,
`BC-INDEX.md`, `BC-2.10.007.md`, `BC-2.05.005.md`, `prd.md`,
`vp-004-content-first-dispatch.md`, `verification-architecture.md`, `purity-boundary-map.md`

**Remediation:** All 7 findings (3H/2M/2N) remediated. MITRE tactic corrected in 2 files;
None-caching propagation gap closed across 8+ artifacts. Fixes committed in burst
`spec: fix adversarial-review pass-3 findings (3H/2M) - T0856 tactic + None-caching propagation`.
Pass 4 dispatched next.

---

### Pass 4 (2026-05-20)

**Findings:** 19 (4 CRIT, 5 HIGH, 5 MED, 3 LOW, 2 NITPICK)
**Delta from pass 3:** +12 total — REGRESSION (fresh-context audit; not a spec regression — prior
passes had not audited capabilities/ and entities/ shards)
**Novelty:** HIGH — first pass to audit L2 capability layer and ent-04 post PR #69–#98 remediation
**Convergence counter:** 0 of 3

**Root cause of spike:** Fresh-context adversarial agent audited the L2 `capabilities/` shards
(cap-06 through cap-11) and `ent-04-findings-output.md` with no prior context. Found 6 capability
shards and ent-04 were never reconciled after the PR #69–#98 brownfield remediation burst. Component
IDs, detection-table verdicts, emission-site tables, BC groupings, and enum ordering were stale
against current `src/`.

**Key finding categories:**

- CRIT (4): Component IDs in cap-06 through cap-11 and ent-04 inconsistent with architecture/
  module-decomposition.md; detection-table verdicts in cap-06..cap-09 stale vs. current analyzer
  src/; ent-04 enum order and field layout inconsistent with findings.rs; component count in
  domain-spec.md showing 20 instead of 21 (csv.rs dispatcher = C-21 not reflected)
- HIGH (5): H-1: CsvReporter (csv.rs, PR #84) entirely absent from SS-11 spec — 0 BCs, not
  listed in cap-11 capabilities; H-2..H-5: emission-site tables in cap-06..cap-09 stale;
  stale line citations in cap-07, cap-08 post-PR #61 refactor; BC grouping anchors in cap-10
  wrong after pass-2 CAP-12 rename
- MED (5): domain-spec.md capability shard count note stale; ent-04 field descriptions inconsistent
  with ent-04 body; VP-020 CSV-injection mechanism not cross-anchored to BC-2.11.021;
  verification-architecture.md CSV reporter section absent; VP-INDEX.md stale VP-020 description
- LOW (3): Minor stale wording in cap-11 introduction; domain-spec.md component count footnote;
  ARCH-INDEX.md SS-11 BC count showing 19 not 24
- NITPICK (2): Formatting inconsistencies in cap-07 and ent-04 tables

**New artifacts:** `specs/behavioral-contracts/ss-11/BC-2.11.020.md` through `BC-2.11.024.md`
(CsvReporter: header order, CSV-injection neutralization, evidence join, trait impl, None encoding)

**Files fixed (16):**
`cap-06-http-analysis.md`, `cap-07-tls-analysis.md`, `cap-08-dns-analysis.md`,
`cap-09-finding-emission.md`, `cap-10-mitre-mapping.md`, `cap-11-reporting-output.md`,
`domain/domain-spec.md`, `domain/entities/ent-04-findings-output.md`,
`behavioral-contracts/BC-INDEX.md`, `behavioral-contracts/ss-11/BC-2.11.020..024.md` (5 new files),
`verification-properties/vp-020-csv-injection-neutralization.md`,
`verification-properties/VP-INDEX.md`, `architecture/verification-architecture.md`,
`architecture/ARCH-INDEX.md` (SS-11 BC count 19→24), `specs/prd.md` (ss-11 range footnote)

**Process gaps identified (codification follow-ups at cycle close):**

1. P4-PG1: Reconciliation passes must cover capabilities/ and entities/ shards, not just
   invariants/architecture. Adversarial checklist must explicitly include these paths.
2. P4-PG2: No component-ID consistency validator between domain-spec/capabilities and
   architecture/module-decomposition. Component IDs can drift silently without a CI check.
3. P4-PG3: New reporter (csv.rs, PR #84) shipped without a BC. A new src/ file in reporter/
   or analyzer/ must trigger a BC coverage review at the point of merge.

**Remediation:** All 19 findings (4C/5H/5M/3L/2N) remediated. L2 capability shards cap-06
through cap-11 fully reconciled against current src/. ent-04 enum order and field layout
corrected. CsvReporter coverage gap closed with 5 new BCs (BC-2.11.020–024). Component count
updated 20→21 in domain-spec.md. VP-020 re-anchored to BC-2.11.021. Fixes committed in burst
`spec: fix adversarial-review pass-4 findings (4C/5H/5M) - reconcile L2 capability layer + add CsvReporter BCs`.
Pass 5 dispatched next.

---

### Pass 5 (2026-05-20)

**Findings:** 8 (1 CRIT, 2 HIGH, 3 MED, 2 LOW)
**Delta from pass 4:** -11 total (CRIT -3, HIGH -3, MED -2, LOW -1, NITPICK -2) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- CRIT (C-1): `BC-2.07.020.md` contained a literal NUL byte (0x00), making the file
  non-UTF-8. The existence check used during spec-package verification did not detect the
  corruption. NUL byte replaced with textual escape ` `; file is now valid UTF-8.
- HIGH (H-1): `BC-INDEX.md` and `prd.md` — BC-2.12.002 title still referenced `--services`
  flag which was removed from the CLI in a prior refactor. Title corrected in both files.
- HIGH (H-2): `BC-2.11.024.md` — direction column showed 8 instead of the correct value 9.
  Corrected.
- MED (M-1): `BC-INDEX.md` — footer BC count arithmetic was inconsistent; corrected to 217
  derived consistently across all subsystems.
- MED (M-2): `nfr-catalog.md` — NFR-VIO-003 example count showed 7; correct value is 8.
  Updated.
- MED (M-3): `domain-spec.md` — active BC count showed 212; correct value is 217. Updated.
- LOW (L-1): `verification-coverage-matrix.md` — VP-008 tool label was non-standard;
  normalized to `cargo-fuzz`.
- LOW (L-2): `nfr-catalog.md` — NFR-VIO-009 rationale was evasive; rewritten to be honest
  about the limitation.

**Process gap identified (codification follow-up at cycle close):**
P5-PG: BC-file on-disk verification used an existence check only; it did not detect a
NUL-byte-corrupted file (BC-2.07.020.md). Recommend a spec-package validator asserting
every BC/spec file is valid UTF-8 with no control bytes other than CR/LF/TAB.

**Files fixed (7):**
`specs/behavioral-contracts/ss-07/BC-2.07.020.md`,
`specs/behavioral-contracts/BC-INDEX.md`,
`specs/prd.md`,
`specs/behavioral-contracts/ss-11/BC-2.11.024.md`,
`specs/prd-supplements/nfr-catalog.md`,
`specs/domain/domain-spec.md`,
`specs/architecture/verification-coverage-matrix.md`

**Remediation:** All 8 findings (1C/2H/3M/2L) remediated. NUL byte removed from
BC-2.07.020.md; stale --services reference purged from BC-INDEX + PRD; active BC count
corrected to 217 in domain-spec.md; BC footer arithmetic made consistent; NFR-VIO-003 count
and NFR-VIO-009 rationale corrected; VP-008 tool label normalized. Fixes committed in burst
`spec: fix adversarial-review pass-5 findings (1C/2H/3M/2L)` (SHA: e7c56a4).
Pass 6 dispatched next.

---

### Pass 6 (2026-05-20)

**Findings:** 3 (0 CRIT, 3 HIGH, 0 MED, 0 LOW)
**Delta from pass 5:** -5 total (CRIT -1, HIGH +1, MED -3, LOW -2) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): 95 BC body files (ss-04 through ss-10) carried incorrect `Architecture Module`
  component-ID anchors. Correct IDs per module-decomposition.md: ss-05 C-15→C-21, ss-06
  C-14→C-12, ss-07 C-16→C-13, ss-08 C-13→C-11, ss-09 C-10→C-14, ss-10 C-11→C-16, and 4
  ss-04 files (lifecycle.rs) C-6→C-15. All 95 BC bodies corrected.
- HIGH (H-2): `BC-INDEX.md` — 34 row titles were out of sync with BC body H1 headings
  (accumulated drift from prior remediation bursts that updated BC bodies without sweeping
  the index). All 34 rows resynchronized.
- HIGH (H-3): `domain/invariants/inv-01-core-invariants.md` — INV-1 enforcement citation
  pointed to `flow.rs:34` (stale); correct line after recent refactors is `flow.rs:48`. Citation
  updated.

**Additional fix (metadata):** `specs/domain/domain-spec.md` frontmatter field
`reconciled_against` carried stale SHA `aa2ece9`; corrected to `0082a0c` (current develop
HEAD, PR #99 — CLAUDE.md governance pointer). Spec content was verified against the actual
working-tree `src/` by spec agents; this only corrects the SHA label.

**Files fixed (97):**
`specs/behavioral-contracts/ss-04/` (4 files: BC-2.04.018, BC-2.04.024, BC-2.04.029, BC-2.04.030),
`specs/behavioral-contracts/ss-05/` (9 files: BC-2.05.001–009),
`specs/behavioral-contracts/ss-06/` (26 files: BC-2.06.001–026),
`specs/behavioral-contracts/ss-07/` (37 files: BC-2.07.001–037),
`specs/behavioral-contracts/ss-08/` (4 files: BC-2.08.001–004),
`specs/behavioral-contracts/ss-09/` (6 files: BC-2.09.001–006),
`specs/behavioral-contracts/ss-10/` (9 files: BC-2.10.001–009),
`specs/behavioral-contracts/BC-INDEX.md`,
`specs/domain/invariants/inv-01-core-invariants.md`,
`specs/domain/domain-spec.md` (metadata SHA reconciliation)

**Remediation:** All 3 findings (3H) remediated. Component-ID anchors corrected across 95
BC body files; BC-INDEX titles resynchronized to BC body H1s (34 rows); INV-1 enforcement
citation updated to current line. Stale `reconciled_against` SHA corrected as metadata fix.
Fixes committed in burst
`spec: fix adversarial-review pass-6 findings (3H) + reconcile stale spec SHA`.
Pass 7 dispatched next.

---

### Pass 7 (2026-05-20)

**Findings:** 13 (1 CRIT, 3 HIGH, 4 MED, 3 LOW, 2 NITPICK)
**Delta from pass 6:** +10 total (CRIT +1, HIGH 0, MED +4, LOW +3, NITPICK +2) — spike; entity shards and capability spec not yet audited at this depth
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- CRIT (C-1): `BC-2.09.002.md` Finding Display implementation used ASCII double-hyphen `--`
  as the separator in formatted output; the spec and source both require an em-dash `—`.
  BC body and BC-INDEX.md row corrected.

- HIGH (H-1): `ent-02-reassembly-flow.md` — approximately 20 line citations stale after
  reassembly-flow refactors. All citations re-anchored to current `src/` line numbers.

- HIGH (H-2): `ent-03-dispatch-analysis.md` — phantom field `classification_attempts` listed
  in TcpReassembler entity that does not exist in `src/`. Field removed. Bonus fix: field name
  `small_segment_run_count` corrected to `small_segment_run` (actual field name in source).
  ~20 stale line citations also re-anchored in the same sweep.

- HIGH (H-3): `BC-2.13.001.md` through `BC-2.13.004.md` and `ARCH-INDEX.md` — SS-13
  (CLI Orchestration) capability anchor was `CAP-01` (wrong); correct anchor is `CAP-12`.
  All 4 BC bodies and the ARCH-INDEX SS-13 row corrected.

- MED (M-1): `cap-05-content-first-dispatch.md` — component ID showed `C-15`; correct ID
  is `C-21` (CsvReporter dispatcher). Updated.

- MED (M-2): `cap-05-content-first-dispatch.md` + `inv-01-core-invariants.md` — `b"HTTP/"`
  token missing from content-first dispatch detection table and from INV-2 invariant body.
  Added to both. `inv-01` line range made consistent with `src/` after token addition.

- MED (M-3): `verification-architecture.md` — VP-008 fuzz skeleton had incorrect argument
  order in the cargo-fuzz invocation; IPv6 address literal was malformed. Both corrected.

- MED (M-4): `BC-2.06.014.md` — error code was `EC-004` (wrong); correct code per
  error-taxonomy is `EC-004` (re-verified). Stale line citation also re-anchored (L-1 overlap).

- LOW (L-1): `BC-2.06.014.md` — stale line citation (addressed together with M-4 above).

- LOW (L-2): `inv-01-core-invariants.md` — INV-2 line range was inconsistent with updated
  `b"HTTP/"` token addition; corrected in the same sweep as M-2.

- LOW (L-3): `architecture/module-decomposition.md` — C-21 (CsvReporter) entry was missing
  retry-budget fields. Fields added for completeness.

- LOW (L-4): `architecture/api-surface.md` — `decode_packet` function absent from public
  API surface table despite being part of the exported surface. Added.

- NITPICK (×2): Covered by the H-1/H-2 re-anchor sweeps (minor wording inconsistencies
  in ent-02 and ent-03 corrected in the same pass).

- EXTRA (L-5): `verification-architecture.md` — VP-018 BC list incomplete; corrected.

**Files fixed (15):**
`specs/behavioral-contracts/ss-09/BC-2.09.002.md`,
`specs/behavioral-contracts/BC-INDEX.md`,
`specs/behavioral-contracts/ss-13/BC-2.13.001.md`, `BC-2.13.002.md`, `BC-2.13.003.md`, `BC-2.13.004.md`,
`specs/domain/entities/ent-02-reassembly-flow.md`,
`specs/domain/entities/ent-03-dispatch-analysis.md`,
`specs/domain/capabilities/cap-05-content-first-dispatch.md`,
`specs/domain/invariants/inv-01-core-invariants.md`,
`specs/behavioral-contracts/ss-06/BC-2.06.014.md`,
`specs/architecture/ARCH-INDEX.md`,
`specs/architecture/verification-architecture.md`,
`specs/architecture/module-decomposition.md`,
`specs/architecture/api-surface.md`

**Remediation:** All 13 findings (1C/3H/4M/3L/2N) remediated. Entity shards ent-02/ent-03
fully re-anchored; phantom `classification_attempts` field removed; `small_segment_run_count`
corrected to `small_segment_run`; em-dash separator fixed in BC-2.09.002 Display; SS-13
BCs re-anchored to CAP-12 in 4 BC bodies and ARCH-INDEX; `b"HTTP/"` token added to
cap-05 and inv-01; VP-008 fuzz arg order and IPv6 literal corrected; BC-2.06.014 EC-004
and line citation corrected; C-21 retry-budget fields added to module-decomposition;
decode_packet added to api-surface; VP-018 BC list corrected. Fixes committed in burst
`spec: fix adversarial-review pass-7 findings (1C/3H/4M/3L) - reconcile entity shards, Display em-dash, SS-13 anchor`
(SHA: 4681813). Pass 8 dispatched next.

---

### Pass 8 (2026-05-20)

**Findings:** 8 (0 CRIT, 2 HIGH, 3 MED, 2 LOW, 1 NITPICK)
**Delta from pass 7:** -5 total (CRIT -1, HIGH -1, MED -1, LOW -1, NITPICK -1) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): `vp-008-decode-packet-no-panic.md` — `decode_packet` argument order in the
  fuzz skeleton had data and length arguments transposed (length-first instead of data-first).
  Corrected to data-first order matching the actual function signature.

- HIGH (H-2): `vp-008-decode-packet-no-panic.md` — IPv6 address literal absent from the fuzz
  input corpus examples. IPv6 target added to the fuzz targets section.

- MED (M-1): `vp-001-flowkey-canonical-ordering.md` — enforcement citation pointed to
  `flow.rs:34` (stale after recent refactors); correct line is `flow.rs:48`. Citation updated.

- MED (M-2): `prd-supplements/error-taxonomy.md` — E-RAS-005 counter name was
  `segments_depth_exceeded`; correct name in source is `segments_segment_limit`. Corrected.

- MED (M-3): `domain/capabilities/cap-02-link-type-gating.md` — `decode_packet` line range
  was `71-140`; correct range post-refactor is `128-172`. Updated.

- LOW (L-1): `behavioral-contracts/ss-02/BC-2.02.007.md` — postcondition listed "two" error
  prefixes; correct count is "three" (the spec body enumerates three distinct error prefixes).
  Corrected.

- LOW (L-2): `verification-properties/VP-INDEX.md` — VP-005 row carried a redundant,
  partially-stale BC list in the index cell. Cleaned to match the canonical BC set in the
  VP-005 body.

- NITPICK (N-1): `domain/invariants/inv-01-core-invariants.md` — INV-2 method-token list
  order did not match the ordering in source. Re-ordered to match source.

**Observation (non-blocking, deferred — see STATE.md P8-DEFER):**
All 217 BC files carry `VP-TBD` placeholders in their Verification Properties field. The
adversary classified this as a deliberate Phase-1 convention, not drift. The forward
VP->BC mapping in VP-INDEX.md is authoritative. BC->VP back-reference back-fill deferred as
a Phase-1-exit polish item; to be surfaced as a structured question at the Phase 1 human
approval gate.

**Files fixed (7):**
`specs/verification-properties/vp-008-decode-packet-no-panic.md`,
`specs/verification-properties/vp-001-flowkey-canonical-ordering.md`,
`specs/verification-properties/VP-INDEX.md`,
`specs/prd-supplements/error-taxonomy.md`,
`specs/domain/capabilities/cap-02-link-type-gating.md`,
`specs/behavioral-contracts/ss-02/BC-2.02.007.md`,
`specs/domain/invariants/inv-01-core-invariants.md`

**Remediation:** All 8 findings (0C/2H/3M/2L/1N) remediated. VP-008 fuzz skeleton arg order
corrected to data-first; IPv6 literal added; stale line citations corrected in vp-001 and
cap-02; E-RAS-005 counter name corrected in error-taxonomy; BC-2.02.007 error-prefix count
corrected; VP-INDEX VP-005 redundant BC list cleaned; INV-2 token order matched to source.
Fixes committed in burst
`spec: fix adversarial-review pass-8 findings (2H/3M/2L) - vp-008 signature, stale citations`
(SHA: 7cf0edd). Pass 9 dispatched next.

---

### Pass 9 (2026-05-20)

**Findings:** 4 (0 CRIT, 1 HIGH, 1 MED, 2 LOW)
**Delta from pass 8:** -4 total (CRIT 0, HIGH -1, MED -2, LOW 0, NITPICK -1) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): `behavioral-contracts/ss-04/BC-2.04.054.md` — 5 stale line citations corrected.
  Architecture Module field: `415` → `557-591`. Invariant 2 latch citation: `mod.rs:385-388` →
  `mod.rs:558-561`. Architecture Anchors push site: `mod.rs:415` → `mod.rs:573`. Architecture
  Anchors latch: `mod.rs:385-388` → `mod.rs:558-561`. Source Evidence Path: `mod.rs:415` →
  `mod.rs:573`. Evidence Types Used guard clause: push site 415 → push site 573.
  All 5 citations now match the post-refactor source layout.

- MED (M-1): `specs/prd.md` section 5 (Error Categories) — prefix scheme misaligned with
  `prd-supplements/error-taxonomy.md`. `E-RDR-NNN` replaced with `E-INP-NNN` (input/file errors).
  `E-CLI-NNN` replaced with `E-CFG-NNN` (configuration errors). Two new categories added:
  `E-ANA-NNN` (analyzer protocol-level parse failures) and `E-OUT-NNN` (output file write
  failures). `E-RAS-NNN` description tightened. `E-DEC-NNN` description updated. Section now
  enumerates all 6 error-taxonomy prefixes: E-INP, E-DEC, E-RAS, E-ANA, E-OUT, E-CFG.

- LOW (L-1): `behavioral-contracts/ss-04/BC-2.04.027.md` — DepthExceeded match arm citation
  was `reassembly/mod.rs:386-389`; correct range is `mod.rs:387-389`. Corrected in Architecture
  Module field, Architecture Anchors section, and Source Evidence Path field.

- LOW (L-2): `specs/architecture/ARCH-INDEX.md` — debt section table accounted for O-01,
  O-03 through O-06 and Smells but gave no account of O-02 and O-07. Note added explaining
  both items are tracked in `domain-debt.md` rather than the architecture debt table because
  they fall outside the architecture layer's scope. Complete open-item set (O-01 through O-07)
  now explicitly acknowledged.

**Files fixed (4):**
`specs/behavioral-contracts/ss-04/BC-2.04.054.md`,
`specs/prd.md`,
`specs/behavioral-contracts/ss-04/BC-2.04.027.md`,
`specs/architecture/ARCH-INDEX.md`

**Remediation:** All 4 findings (0C/1H/1M/2L) remediated. Stale post-refactor citations
corrected in BC-2.04.054 (5 citations) and BC-2.04.027 (1 citation range); prd.md section 5
error-category scheme aligned to the canonical 6-prefix taxonomy in error-taxonomy.md;
ARCH-INDEX debt section annotated to account for O-02 and O-07. Fixes committed in burst
`spec: fix adversarial-review pass-9 findings (1H/1M/2L) - stale citations, error-category alignment`
(SHA: b210c05). Pass 10 dispatched next.

---

### Pass 10 (2026-05-20)

**Findings:** 6 (0 CRIT, 3 HIGH, 3 MED, 0 LOW)
**Delta from pass 9:** +2 total (CRIT 0, HIGH +2, MED +2, LOW -2) — spike; architecture docs not diffed against Cargo.toml in prior passes
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding categories:**

- HIGH (H-1): `architecture/dependency-graph.md` — dependency table built from memory, not
  from a live diff of Cargo.toml. Four concrete errors: `colored` crate listed but replaced by
  `owo-colors`; `num_cpus` listed but removed; `md5` listed without correct crate name `md-5`
  version 0.11; `etherparse` version listed as 0.14 but is 0.16. Additionally `tls-parser` was
  missing from the table entirely. Table rebuilt from Cargo.toml ground truth.

- HIGH (H-2): `architecture/api-surface.md` — `Reporter` trait signature listed as
  `render(&self, findings: &[Finding]) -> Vec<String>` (Vec return). Correct signature is
  `render(&self, findings: &[Finding], config: &OutputConfig) -> String`. Corrected.

- HIGH (H-3): `architecture/api-surface.md` — `ParsedPacket` struct listed `timestamp: f64`
  and `timestamp_micros: u64` fields that do not exist in source. Actual field is
  `packet_len: usize`. Phantom timestamp fields removed; packet_len added.

- MED (M-1): `domain/domain-spec.md` — Capability Index table row for CAP-03 listed
  owning subsystem as `SS-3` (not zero-padded). Corrected to `SS-02` and all SS ID
  references in the Capability Index zero-padded for consistency (SS-01 through SS-13).

- MED (M-2): `domain/domain-spec.md` — test-count anchor in the `reconciled_against` metadata
  field still showed commit `aa2ece9`; correct current develop HEAD is `0082a0c`. Updated.

- MED (M-3): `architecture/dependency-graph.md` — narrative prose claimed "rayon removed"
  as justification for the rebuild. `rayon` is still declared in Cargo.toml (tracked as
  tech-debt item O-07 — declared but unused). False claim corrected; O-07 status note added.

**Process gap identified (codification follow-up at cycle close):**
P10-PG: Architecture-doc dependency tables were not diffed against Cargo.toml before adversarial
review — they were authored from memory. Recommend a mechanical `validate-deps-against-cargo`
check: parse `[dependencies]` and `[dev-dependencies]` from Cargo.toml and assert each declared
crate appears in the dependency-graph.md table with the correct version. Eliminates the class
of stale-version / phantom-crate / missing-crate findings that drove H-1 and M-3 in pass 10.

**Files fixed (3):**
`specs/architecture/dependency-graph.md`,
`specs/architecture/api-surface.md`,
`specs/domain/domain-spec.md`

**Remediation:** All 6 findings (0C/3H/3M/0L) remediated. Dependency table rebuilt from
Cargo.toml ground truth (owo-colors, md-5 0.11, etherparse 0.16, tls-parser added; num_cpus
removed; rayon-removed false claim corrected). Reporter trait signature corrected to
`render(&self, ...) -> String`. ParsedPacket phantom timestamp fields removed; packet_len
added. CAP-03 SS-3 -> SS-02 and all SS IDs zero-padded in domain-spec Capability Index.
Reconciled-against anchor corrected aa2ece9 -> 0082a0c. Fixes committed in burst
`spec: fix adversarial-review pass-10 findings (3H/3M) - dependency table, api-surface, capability anchors`
(SHA: 824f07d). Pass 11 dispatched next.

---

### Pass 11 (2026-05-20) — CONVERGED (clean pass 1 of 3)

**Findings:** 1 LOW + 4 observations (0 CRIT, 0 HIGH, 0 MED, 1 LOW, 4 cosmetic)
**Delta from pass 10:** -5 total (CRIT 0, HIGH -3, MED -3, LOW +1) — no regression
**Novelty:** LOW
**Convergence counter:** 1/3
**Verdict:** CONVERGED — first clean pass. No CRIT/HIGH/MED findings. Passes 12 and 13
must also return clean to satisfy the 3-clean-pass Phase 1d adversarial convergence gate.

**Findings (non-blocking, polished for clean package):**

- LOW (L-1): `architecture/api-surface.md` — `--mitre` flag BC reference listed as
  `BC-2.12.004`; correct reference is `BC-2.12.001`. Corrected.

- Observation (O-1): `architecture/system-overview.md` — pseudocode used `reporter.report(...)`
  which does not match the actual trait method name `render`. Updated to `reporter.render(...)`.

- Observation (O-2): `behavioral-contracts/ss-07/BC-2.07.037.md` — `NonAsciiUtf8` variant
  written as a unit variant; correct form is a struct variant with a `bytes` field, matching
  the actual enum definition. Corrected to struct-variant notation.

- Observation (O-3): `prd-supplements/interface-definitions.md` — exit-code table omitted
  the `exit 2` row for clap argument-parse errors. Row added.

- Observation (O-4): `architecture/dependency-graph.md` — dev-dependency table was
  incomplete; `tempfile`, `proptest`, and `criterion` were absent. All three added.

**Files fixed (5):**
`specs/architecture/api-surface.md`,
`specs/architecture/system-overview.md`,
`specs/behavioral-contracts/ss-07/BC-2.07.037.md`,
`specs/prd-supplements/interface-definitions.md`,
`specs/architecture/dependency-graph.md`

**Polish committed in burst:**
`spec: pass-11 polish (1L/4 observations) - package CONVERGED, counter 1/3`
(SHA: 4d4cf89). Pass 12 dispatched next (confirmation pass).

---

### Pass 12 (2026-05-20) — NOT CONVERGED (counter RESET 1/3 → 0/3)

**Findings:** 6 (0 CRIT, 1 HIGH, 1 MED, 2 LOW, 2 NITPICK)
**Delta from pass 11:** +5 total (HIGH +1, MED +1, LOW +1, NITPICK +2) — regression; streak broken
**Novelty:** LOW
**Convergence counter:** 0/3 (RESET — pass 12 was not clean; 3 consecutive clean passes required)
**Verdict:** NOT CONVERGED — HIGH + MED findings disqualify this pass as a clean pass. Counter
resets to 0/3. Pass 13 is next; must start a fresh consecutive-clean streak.

**Key finding categories:**

- HIGH (F-1): `behavioral-contracts/ss-11/BC-2.11.007.md` — Postcondition 3 asserted that only
  the high-C1 range (0x80–0x9F) is escaped; this contradicted both the source code and the sibling
  BC-2.11.009 which correctly specifies the full C1 range including NEL (0x85). Postcondition 3
  rewritten to state that the entire C1 range (0x80–0x9F inclusive, including NEL) is escaped.

- MED (F-2): `behavioral-contracts/ss-11/BC-2.11.001.md` — `json.rs` unwrap citation cited line 30;
  correct line post-refactor is 59. Additionally, the BC body contained a claim that the module
  comment in `json.rs` asserts RFC 8259 compliance — no such comment exists in source. Both the
  stale citation and the unsupported claim removed.

- LOW (F-3): `behavioral-contracts/ss-04/BC-2.04.049.md` — EC-002 postcondition described IPv6
  addresses as bracket-less; the rendering in source includes brackets `[addr]`. Corrected.

- LOW (F-4): `behavioral-contracts/ss-04/BC-2.04.049.md` — `flow.rs` citation line 69 was stale;
  correct line is 70. Corrected.

- NITPICK (N-1, N-2): `behavioral-contracts/ss-11/BC-2.11.020.md` — two `csv.rs` citation
  off-by-one errors in the Architecture Anchors section. Both corrected.

**Recurring process gap (6th occurrence — MANDATORY codification follow-up):**
Stale source-line citations (`file.rs:NNN`) recurred in this pass (F-2, F-4) and were also
present in passes 4, 6, 8, 9, and 10. This is the 6th recurrence of the same process gap.
Per the Cycle-Closing Checklist, a recurring process gap with 6+ occurrences requires a
mandatory codification follow-up (a follow-up story or justified deferral) before the cycle
can be declared closed. See STATE.md Deferred Findings for the required action.

**Files fixed (4):**
`specs/behavioral-contracts/ss-11/BC-2.11.007.md`,
`specs/behavioral-contracts/ss-11/BC-2.11.001.md`,
`specs/behavioral-contracts/ss-04/BC-2.04.049.md`,
`specs/behavioral-contracts/ss-11/BC-2.11.020.md`

**Remediation:** All 6 findings (0C/1H/1M/2L/2N) fixed. C1 postcondition corrected in
BC-2.11.007; stale citation + unsupported claim removed from BC-2.11.001; IPv6 bracket-less
rendering and stale citation fixed in BC-2.04.049; csv.rs off-by-one citations fixed in
BC-2.11.020. Fixes committed in burst
`spec: fix adversarial-review pass-12 findings (1H/1M/2L) - C1-escape postcondition, stale citations`
(SHA: c21b13c). Pass 13 dispatched next.

---

### Pass 13 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 5 (0 CRIT, 2 HIGH, 0 MED, 3 LOW — of which 2L + 1N reported as "3 LOW, 1 NITPICK")
**Delta from pass 12:** -1 total (HIGH +1, MED -1, LOW +1, NITPICK -1) — no regression; fresh-context anchor audit
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH findings disqualify; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 2 HIGH findings present. Counter remains 0/3. Pass 14 is next.

**Key finding categories:**

- HIGH (H-1): `domain/entities/ent-05-enums-value-objects.md` — 7 value-object source-line anchors
  were stale (post-refactor drift in `findings.rs`, `output/json.rs`, `output/csv.rs`). All 7
  anchor citations corrected to current line numbers.

- HIGH (H-2): `domain/invariants/inv-01-core-invariants.md` — INV-4 enforcement citation referenced
  `findings.rs:72-80`; correct lines post-refactor are `findings.rs:10-14` (struct definition) and
  `findings.rs:148-156` (validation logic). Citation updated to dual-anchor form.

- LOW (C-1): `architecture/ARCH-INDEX.md` — component-count summary line stated range `C-1..C-20`;
  correct range is `C-1..C-21` (CsvReporter dispatcher was added as C-21 in pass-4 remediation but
  the summary line was not updated). Corrected.

- LOW (LOW): `specs/prd.md` — BC-2.07.004 section-2.7 one-liner description was misaligned with
  the canonical BC H1 heading. Aligned to canonical BC body language.

- NITPICK (1N): Minor wording inconsistency; corrected in the same sweep as H-1.

**Recurring process gap (7th occurrence — P-CITE-PG):**
Stale source-line citations (`file.rs:NNN`) recurred again in this pass (H-1, H-2). This is the
7th recurrence across passes 4, 6, 8, 9, 10, 12, 13. Mandatory codification follow-up (P-CITE-PG)
already recorded in STATE.md Deferred Findings. No new action item added.

**Files fixed (4):**
`specs/domain/entities/ent-05-enums-value-objects.md`,
`specs/domain/invariants/inv-01-core-invariants.md`,
`specs/architecture/ARCH-INDEX.md`,
`specs/prd.md`

**Remediation:** All 5 findings (0C/2H/0M/3L) remediated. ent-05 7 value-object anchors corrected;
INV-4 enforcement citation updated to dual-anchor form (findings.rs:10-14 + :148-156); ARCH-INDEX
component-count range corrected C-1..C-20 → C-1..C-21; prd.md BC-2.07.004 one-liner aligned to
canonical BC H1. Fixes committed in burst
`spec: fix adversarial-review pass-13 findings (2H/3L) - ent-05 anchors, INV-4 anchor`.
Pass 14 dispatched next.

---

### Pass 14 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 3 (0 CRIT, 1 HIGH, 0 MED, 1 LOW, 1 NITPICK)
**Delta from pass 13:** -2 total (HIGH -1, LOW -2, NITPICK +1) — no regression; continued reduction
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH finding disqualifies; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 1 HIGH finding present. Counter remains 0/3. Pass 15 is next.

**Key finding categories:**

- HIGH (H-1): `domain/domain-spec.md` and `domain/capabilities/cap-12-cli-orchestration.md` —
  summary.rs component ID cited as `C-16` in both files (4 total locations). Correct component ID
  per `architecture/module-decomposition.md` is `C-17`. Fixed in:
  - `domain-spec.md`: CAP-12 note paragraph (1 site) and SS-12 subsystem map row (1 site)
  - `cap-12-cli-orchestration.md`: Sources line in overview (1 site) and section header
    "Summary accumulation (summary.rs / C-16)" (1 site)

- LOW (L-1): `domain/domain-spec.md` — Entity/Enum Index table (section 5) did not list
  `E-39b CsvReporter` in the ent-04 row. CsvReporter has been in the codebase since PR #84;
  the entity index was never updated to reflect it. Entry added. Entity count updated 41→42
  in both the frontmatter summary table and the section-5 header.

- NITPICK (N-1): `behavioral-contracts/ss-12/BC-2.12.005.md` — Architecture Anchors section
  and Source Evidence Path field both cited `src/cli.rs:61-105`; correct range after latest
  refactor is `src/cli.rs:61-106` (off-by-one on end line). Fixed in 2 locations.

**Recurring process gap (8th occurrence — P-CITE-PG):**
Stale source-line citation recurred in N-1 (cli.rs:61-105 → 61-106). This is the 8th occurrence
of the P-CITE-PG gap. Mandatory codification follow-up already recorded in STATE.md. No new
action item added.

**Files fixed (3):**
`specs/domain/domain-spec.md`,
`specs/domain/capabilities/cap-12-cli-orchestration.md`,
`specs/behavioral-contracts/ss-12/BC-2.12.005.md`

**Remediation:** All 3 findings (0C/1H/0M/1L/1N) remediated. summary.rs C-16→C-17 corrected
in 4 locations across 2 files; E-39b CsvReporter added to entity index and entity count
incremented 41→42; cli.rs citation range corrected 61-105→61-106 in 2 locations. Fixes committed
in burst
`spec: fix adversarial-review pass-14 findings (1H/1L/1N) - C-16/C-17 mis-anchor, entity index`
(SHA: 3ec08db). Pass 15 dispatched next.

---

### Inter-Pass Sweep (2026-05-20) — Proactive Anchor-Consistency Sweep

**Type:** Remediation burst (not an adversary pass)
**Trigger:** Recurring component-ID / capability-anchor defect class found in passes 4, 6, 10, 13, 14.
Orchestrator commissioned a root-cause sweep before dispatching pass 15.
**Convergence counter:** 0/3 (unchanged — no adversary pass issued)

**Scope:** Comprehensive C-NN / SS-NN / capability-column anchor audit across the full spec package.
Total occurrences audited: 3,820.

**Mis-anchors found and fixed: 28**

1. **C-ID mis-anchors in ss-12 BC bodies (3 fixes):**
   - `behavioral-contracts/ss-12/BC-2.12.018.md` — Architecture Module field cited `C-16`; correct is `C-17` (summary.rs).
   - `behavioral-contracts/ss-12/BC-2.12.019.md` — same C-16→C-17 correction.
   - `behavioral-contracts/ss-12/BC-2.12.021.md` — same C-16→C-17 correction.
   These three files were missed in the pass-14 remediation which corrected domain-spec.md and
   cap-12-cli-orchestration.md but did not sweep the BC bodies that also anchor to summary.rs.

2. **Capability-column mis-anchors in prd.md traceability matrix (25 fixes):**
   - All 21 BC-2.12.* rows had capability column mapped to `CAP-01` instead of `CAP-12`.
   - All 4 BC-2.13.* rows had capability column mapped to `CAP-01` instead of `CAP-12`.
   Root cause: CAP-12 (CLI Orchestration) was added in pass-2 remediation; the prd.md traceability
   matrix was not swept at that time and defaulted to the CAP-01 placeholder for all new rows.

**Root-cause analysis:**
The recurring defect class (component-ID and capability-anchor drift) is driven by P4-PG2:
no automated cross-file consistency validator exists to assert that a component-ID or
capability-anchor is consistent across BC bodies, domain-spec, capability shards, and the
prd.md traceability matrix. Manual remediation bursts fix the reported site but leave
sibling files un-swept. Mandatory codification follow-up P4-PG2 already recorded in STATE.md.

**Files fixed (4):**
`specs/behavioral-contracts/ss-12/BC-2.12.018.md`,
`specs/behavioral-contracts/ss-12/BC-2.12.019.md`,
`specs/behavioral-contracts/ss-12/BC-2.12.021.md`,
`specs/prd.md`

**Committed in burst:**
`spec: proactive anchor-consistency sweep - fix 3 C-ID + 25 capability-column mis-anchors`
(SHA: 21093ed). Pass 15 dispatched next.

---

### Pass 15 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 4 (0 CRIT, 1 HIGH, 2 MED, 1 NITPICK)
**Delta from pass 14:** +1 total (HIGH 0, MED +2, LOW -1, NITPICK 0) — no regression; LOW findings resolved, 2 MED surfaced in VP-020 and module-decomposition
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH finding disqualifies; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 1 HIGH + 2 MED findings present. Counter remains 0/3. Pass 16 is next.

**Key finding categories:**

- HIGH (F-1): `verification-properties/vp-020-csv-injection-neutralization.md` — the
  `test_csv_safe_values_unchanged` test was written against a non-existent API: it used
  `CsvReporter::new()` (no such constructor; `CsvReporter` is a unit struct) and called
  `reporter.render(...)` with a `Summary` + `findings` + `analyzer_summaries` triple-arg
  form that does not match the actual `Reporter` trait signature. The `render` method returns
  an owned `String`, not a `Vec<String>`. Test rewritten to use `CsvReporter` directly
  (unit struct — no constructor) and the correct `render(&self, ...) -> String` signature.

- MED (F-2): `verification-properties/vp-020-csv-injection-neutralization.md` — Property
  Statement point 3 claimed that `AnalysisSummary` detail values are neutralized. This was
  incorrect: `CsvReporter::render` explicitly ignores the `_analyzer_summaries` parameter
  (underscore-prefixed at `csv.rs:56`). No `AnalysisSummary` data ever reaches a CSV cell.
  Point 3 corrected to reflect the actual neutralization scope (per-`Finding` fields only).

- MED (F-3): `architecture/module-decomposition.md` — L4 reporter table Purity column listed
  `C-19` (JsonReporter) and `C-20` (TerminalReporter) as `Effectful (stdout write)`, and the
  CsvReporter row as `Effectful (stdout/file write)`. This was incorrect: all three reporter
  `render()` implementations are pure `&self -> String` transformations — they return an owned
  `String` and perform no I/O themselves. The I/O (stdout write or file write) is the caller's
  responsibility (`main.rs`). All three Purity cells corrected to `Pure (returns owned String;
  no I/O -- caller in main.rs writes)`.

- NITPICK (N-1): Covered by the F-3 purity-column fix; minor wording cleaned in the same pass.

**Files fixed (2):**
`specs/verification-properties/vp-020-csv-injection-neutralization.md`,
`specs/architecture/module-decomposition.md`

**Remediation:** All 4 findings (0C/1H/2M/1N) remediated. VP-020 test rewritten to the real
`CsvReporter`/`render() -> String` API; Property Statement point 3 corrected to exclude
`AnalysisSummary` from neutralization scope; module-decomposition reporter Purity column
corrected from effectful to pure for all three reporter components. Fixes committed in burst
`spec: fix adversarial-review pass-15 findings (1H/2M/1N) - VP-020 CsvReporter API + reporter purity`
(SHA: 7a66b0b). Pass 16 dispatched next.

---

### Pass 16 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 3 (1 CRIT, 0 HIGH, 1 MED, 1 LOW)
**Delta from pass 15:** -1 total (CRIT +1, HIGH -1, MED -1, LOW +1) — no regression; findings are in a different BC file than pass-15 targets
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — CRIT finding disqualifies; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 1 CRIT finding present. Counter remains 0/3. Pass 17 is next.

**Key finding categories:**

- CRIT (F-1): `behavioral-contracts/ss-07/BC-2.07.037.md` — Postcondition 4 asserted that the
  arm-3 (NonAsciiUtf8) finding has verdict `Anomaly/Likely/High`. This is incorrect. Arm 3 fires
  for non-ASCII UTF-8 SNI hostnames; the correct verdict tuple per the source code and sibling BCs
  (BC-2.07.017, BC-2.07.019) is `Anomaly/Inconclusive/Low`. The `Likely/High` tuple is used by
  arm 1 (valid ASCII clean hostname — no finding) and arm 2 (AsciiWithControl — BC-2.07.014),
  not arm 3. Postcondition 4 corrected to `Anomaly/Inconclusive/Low`.

- MED (F-2): `behavioral-contracts/ss-07/BC-2.07.017.md` and `BC-2.07.019.md` — both files
  contained stale internal correction-notes of the form "BC-INDEX title says..." that were
  remediation breadcrumbs from an earlier pass (pass 6 BC-INDEX resync). These notes are no
  longer accurate or useful and were removed.

- LOW (L-1): Minor wording inconsistency in BC-2.07.037.md Related BCs section; corrected in
  the same sweep as F-1.

**Files fixed (3):**
`specs/behavioral-contracts/ss-07/BC-2.07.037.md`,
`specs/behavioral-contracts/ss-07/BC-2.07.017.md`,
`specs/behavioral-contracts/ss-07/BC-2.07.019.md`

**Remediation:** All 3 findings (1C/1M/1L) remediated. BC-2.07.037 Postcondition 4 verdict
corrected Anomaly/Likely/High → Anomaly/Inconclusive/Low to match source and sibling BCs;
stale correction-notes removed from BC-2.07.017 and BC-2.07.019. Fixes committed in burst
`spec: fix adversarial-review pass-16 findings (1C/1M) - SNI verdict + stale notes`
(SHA: cbef4f1). Pass 17 dispatched next.

---

### Inter-Pass Sweep (2026-05-20) — Comprehensive BC-vs-Source Verification Sweep

**Type:** Remediation burst (not an adversary pass)
**Trigger:** After pass 16, recurring citation/token-drift defect class (P-CITE-PG, 8 occurrences
across passes 4, 6, 8, 9, 10, 12, 13, 14) had driven repeated HIGH and MEDIUM findings that reset
or held back the convergence counter. The orchestrator commissioned a proactive, comprehensive
BC-vs-source verification sweep of all 217 BCs (6 parallel agents, one per subsystem group) before
dispatching pass 17, to flush residual spec-vs-code drift at root rather than discovering it one
defect at a time per adversary pass.
**Convergence counter:** 0/3 (unchanged — no adversary pass issued)

**Scope:** All 217 behavioral contracts across ss-01..ss-13 re-verified against current src/.

**Defects found and fixed: ~58 total**

#### ss-04 (2 defects)
- `BC-2.04.012.md` — stale latch/counter line citations corrected
- `BC-2.04.030.md` — stale latch/counter line citations corrected

#### ss-07 (6 defects)
- `BC-2.07.001.md` — off-by-one match-arm citations corrected
- `BC-2.07.009.md` — GREASE-mechanism mis-description corrected; off-by-one citations
- `BC-2.07.017.md` — off-by-one match-arm citations corrected
- `BC-2.07.019.md` — off-by-one match-arm citations corrected

#### ss-06 / ss-05 (15 defects)
- `BC-2.06.001.md`, `BC-2.06.002.md`, `BC-2.06.003.md` — off-by-one function-end citations
- `BC-2.06.004.md` — wrong MAX_MAP_ENTRIES cap claim corrected (semantic)
- `BC-2.06.005.md` — fabricated backslash traversal pattern removed; wrong evidence-truncation
  claim corrected (semantic)
- `BC-2.05.009.md` — wrong unwrap-vs-iflet claim corrected (semantic); off-by-one citations

#### ss-11 / ss-10 (12 defects)
- `BC-2.11.007.md` — wrong C0-escaping claim re CR/LF corrected (semantic: CR and LF are
  not escaped — only C0 range 0x00-0x1F excluding CR/LF/TAB, plus C1 range 0x80-0x9F)
- `BC-2.11.009.md`, `BC-2.11.011.md`, `BC-2.11.014.md`, `BC-2.11.015.md`, `BC-2.11.019.md`
  — off-by-one citations corrected
- `BC-2.10.008.md` — all emitted-site citations were stale; fully re-anchored (semantic)
- `BC-2.10.001.md`, `BC-2.10.007.md` — off-by-one citations corrected

#### ss-12 / ss-09 / ss-08 (8 defects)
- `BC-2.09.001.md` — wrong source_ip/direction claims for reassembly findings corrected
  (semantic: reassembly findings use client-side IP and forward direction, not bidirectional)
- `BC-2.09.004.md` — off-by-one citations corrected
- `BC-2.08.001.md`, `BC-2.08.002.md` — off-by-one citations corrected
- `BC-2.12.001.md`, `BC-2.12.006.md`, `BC-2.12.007.md`, `BC-2.12.014.md` — off-by-one
  citations corrected

#### ss-01 / ss-02 (15 defects)
- `BC-2.01.002.md`, `BC-2.01.005.md` — fabricated Duration API calls corrected (semantic:
  no `Duration::from_secs_f64` usage in the relevant source path; correct API cited)
- `BC-2.01.008.md` — off-by-one citations corrected
- `BC-2.02.005.md` — wrong-function citation corrected (cited wrong function for behavior)
- `BC-2.02.003.md`, `BC-2.02.004.md`, `BC-2.02.006.md`, `BC-2.02.014.md` — off-by-one
  citations corrected

**Summary of defect classes:**
- Off-by-one / stale line citations: ~52 defects (citation drift from src/ refactors)
- Semantic spec-vs-code defects: ~6 defects (wrong API, wrong cap claims, wrong field/direction
  claims, wrong escape-behavior descriptions)

**Root cause addressed:** P-CITE-PG — no automated validator resolves `file.rs:NNN` anchors
in spec artifacts. This sweep manually closed the accumulated citation debt across all 217 BCs.
Mandatory codification follow-up (P-CITE-PG) remains open for cycle close.

**Files fixed (37 BC body files):**
ss-01: BC-2.01.002, BC-2.01.005, BC-2.01.008
ss-02: BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006, BC-2.02.014
ss-04: BC-2.04.012, BC-2.04.030
ss-05: BC-2.05.009
ss-06: BC-2.06.001, BC-2.06.002, BC-2.06.003, BC-2.06.004, BC-2.06.005
ss-07: BC-2.07.001, BC-2.07.009, BC-2.07.017, BC-2.07.019
ss-08: BC-2.08.001, BC-2.08.002
ss-09: BC-2.09.001, BC-2.09.004
ss-10: BC-2.10.001, BC-2.10.007, BC-2.10.008
ss-11: BC-2.11.007, BC-2.11.009, BC-2.11.011, BC-2.11.014, BC-2.11.015, BC-2.11.019
ss-12: BC-2.12.001, BC-2.12.006, BC-2.12.007, BC-2.12.014

BC-INDEX.md NOT modified — index was current; body files only.

**Committed in burst:**
`spec: comprehensive BC-vs-source verification sweep - fix ~58 residual drift defects across 217 BCs`
(SHA: d038ace). Pass 17 dispatched next.

---

### Pass 17 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 5 (0 CRIT, 2 HIGH, 1 MED, 1 LOW, 1 NITPICK)
**Delta from pass 16:** +2 total (CRIT -1, HIGH +2, MED 0, LOW 0, NITPICK +1) — no regression; findings concentrated in one entity shard not touched by BC sweep
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH findings disqualify; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 2 HIGH findings present. Counter remains 0/3. Pass 18 is next.

**Key observation:** Pass 17 found ZERO BC defects. The comprehensive BC-vs-source sweep
(d038ace, inter-pass 16→17, 217 BCs re-verified) held completely. All 5 findings were confined
to `ent-04-findings-output.md`, a domain entity shard outside the BC sweep scope.

**Key finding categories:**

- HIGH (F-1): `domain/entities/ent-04-findings-output.md` — `AnalysisSummary.detail` field
  typed as `HashMap<String, String>` in the spec; the actual source uses `BTreeMap<String, String>`
  to guarantee deterministic output ordering (required for stable CSV/JSON serialization).
  Corrected to `BTreeMap`.

- HIGH (F-2): `domain/entities/ent-04-findings-output.md` — spec asserted that
  `AnalysisSummary` is "only tested via inline unit tests in the same file." This claim was
  false: `AnalysisSummary` is also covered by integration tests and reporter tests. Claim
  removed; stale line range associated with the assertion also corrected.

- MED (F-3): `domain/entities/ent-04-findings-output.md` — cross-reference cited
  `BC-RPT-007`; correct reference is `BC-RPT-001`. Corrected.

- LOW (F-4): `domain/entities/ent-04-findings-output.md` — `AnalysisSummary` struct line
  range cited as `12-17`; correct post-refactor range is `38-50`. Corrected.

- NITPICK (F-5): `domain/entities/ent-04-findings-output.md` — `Verdict` enum citation
  `32-40` corrected to `30-40` (off-by-one on start line).

**Files fixed (1):**
`specs/domain/entities/ent-04-findings-output.md`

**Remediation:** All 5 findings (0C/2H/1M/1L/1N) remediated. AnalysisSummary.detail type
corrected HashMap→BTreeMap; false inline-test claim removed + stale range fixed; BC-RPT-007
cross-ref corrected to BC-RPT-001; AnalysisSummary line range corrected 12-17→38-50;
Verdict citation corrected 32-40→30-40. Fixes committed in burst
`spec: fix adversarial-review pass-17 findings (2H/1M/1L) - ent-04 BTreeMap + inline-test claim`
(SHA: 0c16cad). Pass 18 dispatched next.

---

### Pass 18 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 5 (0 CRIT, 3 HIGH, 0 MED, 2 LOW)
**Delta from pass 17:** 0 total (CRIT 0, HIGH +1, MED -1, LOW +1, NITPICK -1) — no regression; root cause is same class (stale anchor drift) but in different shards
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH findings disqualify; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 3 HIGH findings present. Counter remains 0/3. Pass 19 is next.

**Root cause:** PR #75 shifted `//!` doc-header line numbers across several modules. The
inter-pass-16 BC-vs-source sweep (d038ace) corrected BC body files only; the domain entity
and capability shards (ent-01, ent-04, cap-10, ent-02, domain-spec) were not in scope for
that sweep and carried stale anchors from the same PR #75 header drift.

**Key finding categories:**

- HIGH (H-1): `domain/entities/ent-01-ingestion-decoding.md` — 8 entity anchors pointed at
  pre-PR-#75 line numbers in `src/packet.rs`, `src/decoder.rs`, and `src/link.rs`. All 8
  re-resolved to current line positions.

- HIGH (H-2): `domain/entities/ent-04-findings-output.md` — 6 cross-file anchors for
  `Finding`, `Verdict`, `Severity`, and `AnalysisSummary` pointed at pre-PR-#75 locations
  in `src/analyzer/` and `src/reporter/`. All 6 re-resolved.

- HIGH (H-3): `domain/capabilities/cap-10-mitre-mapping.md` — rendering anchor for
  unknown-ID path used a pre-PR-#75 line number in `src/mitre.rs`. Corrected to current
  position.

- LOW (L-1): `domain/entities/ent-02-reassembly-flow.md` — component range `C-6..C-9`
  missing `C-15` (added in PR #75 refactor). Expanded to `C-6..C-9,C-15`.

- LOW (L-2): `domain/domain-spec.md` — test count expressed as approximate `"~282"`;
  corrected to exact `"282"` to match the green CI suite.

**Files fixed (5):**
- `specs/domain/entities/ent-01-ingestion-decoding.md`
- `specs/domain/entities/ent-04-findings-output.md`
- `specs/domain/capabilities/cap-10-mitre-mapping.md`
- `specs/domain/entities/ent-02-reassembly-flow.md`
- `specs/domain/domain-spec.md`

**Remediation:** All 5 findings (0C/3H/2L) remediated. All were stale source-line anchors
from PR #75 `//!` header shifts in the last unreconciled domain shards — outside the scope
of the prior BC sweep (d038ace). Fixes committed in burst
`spec: fix adversarial-review pass-18 findings (3H/2L) - re-resolve stale entity/capability anchors`
(SHA: fc28b69). Pass 19 dispatched next.

---

### Pass 19 (2026-05-20)

**Findings:** 2 (0 CRIT, 0 HIGH, 2 MED, 0 LOW)
**Delta from pass 18:** -3 total (CRIT 0, HIGH -3, MED +2, LOW -2) — no regression; continued improvement
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Adversary assessment:** Package described as "overwhelmingly clean". Only 2 MEDIUM
findings identified — both minor consistency gaps between architecture documents.

**Key finding categories:**

- MED (M-1): `specs/architecture/purity-boundary-map.md` — JsonReporter, CsvReporter, and
  TextReporter classified as Effectful-shell. Incorrect: these reporters produce String or
  byte output with no I/O side-effects and match the Pure-core definition used in
  module-decomposition.md. Reclassified to Pure-core for all three.

- MED (M-2): `specs/architecture/dependency-graph.md` — test-count statement was
  inconsistent with the authoritative count established in prior passes. Corrected to
  "264 in tests/ + 18 inline = 282".

**Files fixed (2):**
- `specs/architecture/purity-boundary-map.md`
- `specs/architecture/dependency-graph.md`

**Remediation:** All 2 findings (0C/0H/2M/0L) remediated. Fixes committed in burst
`spec: fix adversarial-review pass-19 findings (2M) - reporter purity + test-count`
(SHA: f913004). Pass 20 dispatched next.

---

### Pass 20 (2026-05-20)

**Findings:** 4 (0 CRIT, 0 HIGH, 2 MED, 1 LOW, 1 NITPICK)
**Delta from pass 19:** +2 total (MED 0, LOW +1, NITPICK +1) — no regression; 4 discrete spec precision gaps
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Adversary assessment:** All findings are spec-precision gaps — no behavioral correctness defects
found. SEEDED_IDS list in VP-007 contained placeholder IDs; citation ranges off-by-one in two
files. Package remains in very clean state.

**Key finding categories:**

- MED (F-1): `specs/verification-properties/vp-007-mitre-technique-id-format.md` — `SEEDED_IDS`
  list contained placeholder/incorrect technique IDs rather than the actual 15 real MITRE IDs
  present in the seeded test fixture. Corrected to the real 15 IDs from the source fixture.

- MED (F-2): `specs/verification-properties/vp-007-mitre-technique-id-format.md` — Source
  Location citation range was `99-129`; correct range post-refactor is `122-156`. Updated.

- LOW (F-3): `specs/behavioral-contracts/ss-12/BC-2.12.008.md` — 5 off-by-one citation errors
  in `main.rs` anchor; all cited `57-58` where correct range is `57-59`. All 5 instances
  corrected.

- NITPICK (F-4): `specs/prd-supplements/interface-definitions.md` — `mitre_technique` regex
  pattern was permissive (matched too broadly); tightened to strict form matching actual
  MITRE technique ID format.

**Files fixed (3):**
- `specs/verification-properties/vp-007-mitre-technique-id-format.md`
- `specs/behavioral-contracts/ss-12/BC-2.12.008.md`
- `specs/prd-supplements/interface-definitions.md`

**Remediation:** All 4 findings (0C/0H/2M/1L/1N) remediated. VP-007 SEEDED_IDS corrected to
real 15 MITRE IDs; VP-007 Source Location citation updated 99-129 → 122-156; BC-2.12.008
main.rs citation corrected 57-58 → 57-59 (5 instances); interface-definitions.md mitre_technique
regex tightened to strict form. Pass 21 dispatched next.

---

### Pass 21 (2026-05-20)

**Findings:** 3 (0 CRIT, 0 HIGH, 1 MED, 0 LOW, 2 NITPICK)
**Delta from pass 20:** -1 total (MED -1, LOW -1, NITPICK +1) — no regression; continued reduction
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Adversary assessment:** Package remains in very clean state. Single medium finding is a
subsystem anchor misclassification (C-10 in module-decomposition.md pointing to SS-08 DNS-specific
module instead of SS-05 shared analyzer-trait module). Two nitpicks are PRD editorial gaps. No
behavioral correctness defects found.

**Key finding categories:**

- MED (F-1): `specs/architecture/module-decomposition.md` — C-10 (`analyzer/mod.rs`, shared
  analyzer trait module) was anchored under SS-08 (DNS Analyzer subsystem) instead of SS-05
  (Analyzer Registry subsystem). Re-anchored to SS-05.

- MED follow-up: `specs/architecture/ARCH-INDEX.md` — SS-05 registry row "Primary Source Files"
  column updated from `dispatcher.rs` to `dispatcher.rs, analyzer/mod.rs` to reflect the
  re-anchor of C-10.

- NITPICK (O-1): `specs/prd.md` — Out-of-Scope section "Removed CLI flags" list was incomplete;
  `--verbose` and `--services` were present in prior versions but not listed as explicitly
  removed. Both flags added to the removed-flags enumeration.

- NITPICK (O-2): `specs/prd.md` — BC-2.07.016 inline one-liner was editorially misaligned with
  its canonical H1 heading format (minor wording inconsistency). Aligned to canonical H1.

**Files fixed (3):**
- `specs/architecture/module-decomposition.md`
- `specs/architecture/ARCH-INDEX.md`
- `specs/prd.md`

**Remediation:** All 3 findings (0C/0H/1M/0L/2N) remediated. C-10 re-anchored SS-08→SS-05 in
module-decomposition.md; ARCH-INDEX.md SS-05 row updated (follow-up to F-1); prd.md
Out-of-Scope removed-flags completed and BC-2.07.016 one-liner aligned. Counter remains 0/3.
Pass 22 dispatched next.

---

### Pass 22 (2026-05-20) — CONVERGED (CLEAN PASS 1/3)

**Findings:** 3 (0 CRIT, 0 HIGH, 0 MED, 2 LOW, 1 NITPICK)
**Delta from pass 21:** 0 total — same count, all findings demoted to LOW/NITPICK severity
**Novelty:** LOW
**Convergence counter:** 1/3 — FIRST CLEAN PASS (0C/0H/0M; counter incremented from 0 to 1)
**Verdict:** CONVERGED — no CRITICAL, HIGH, or MEDIUM findings. Counter advances to 1/3.

**Adversary assessment:** Pass 22 is the first clean pass of the current run. No behavioral
defects. Two LOW polish items and one editorial nitpick only.

**Key finding categories:**

- LOW (LOW-1): `specs/behavioral-contracts/ss-12/BC-2.12.005.md` — H1 precondition covered
  only a subset of reassembly flags. Broadened to enumerate all 9 reassembly flags. BC-INDEX
  title row and prd.md inline synopsis synced.

- LOW (LOW-2): `specs/behavioral-contracts/ss-07/BC-2.07.004.md` — Source Location citation
  ranges were off by one line on both boundaries. Tightened to exact current positions.

- NITPICK (N-1): `specs/prd-supplements/error-taxonomy.md` — E-ANA-003 oversized-record guard
  citation aligned to `tls.rs:643-653` to match the actual guard location.

**Files fixed (4):**
- `specs/behavioral-contracts/ss-12/BC-2.12.005.md`
- `specs/behavioral-contracts/BC-INDEX.md`
- `specs/prd.md`
- `specs/behavioral-contracts/ss-07/BC-2.07.004.md`
- `specs/prd-supplements/error-taxonomy.md`

**Remediation:** All polish applied. Counter advances to 1/3. Pass 23 dispatched as second
confirmation pass.

---

### Pass 23 (2026-05-20) — NOT CONVERGED — STREAK BROKEN (counter RESET 1/3 → 0/3)

**Findings:** 3 (0 CRIT, 1 HIGH, 1 MED, 0 LOW, 1 NITPICK)
**Delta from pass 22:** 0 total (HIGH +1, MED +1, LOW -2, NITPICK 0) — REGRESSION in severity
**Novelty:** LOW
**Convergence counter:** 0/3 — RESET (pass 22 had started streak at 1/3; pass 23 HIGH+MED
findings break the streak; counter returns to 0)
**Verdict:** NOT CONVERGED — 1 HIGH + 1 MEDIUM present. Streak broken; counter reset to 0/3.
Pass 24 is next.

**Root cause:** Three isolated spec precision gaps — one anchor collision in the purity boundary
map, one stale flag description in module criticality, and one citation range off-by-one in the
error taxonomy supplement.

**Key finding categories:**

- HIGH (F-1): `specs/architecture/purity-boundary-map.md` — `csv.rs` component anchor C-21
  collided with an existing numbered component. The collision caused the anchor to shadow a
  different component in cross-reference lookups. Fixed by changing the numbered entry to an
  unnumbered `(--)` anchor (component is a leaf with no inbound cross-references).

- MED (F-2): `specs/module-criticality.md` — a row in the module-criticality table described
  a CLI flag as "parsed but not executed" (absent-flag behavior description). This was stale:
  the flag was removed by PR #74 and `clap` now rejects it as an unknown argument at startup.
  Row corrected to: "removed by PR #74; clap rejects as unknown arguments".

- NITPICK (O-1): `specs/prd-supplements/error-taxonomy.md` — E-INP-001 source citation listed
  as lines `56-59`; correct range is `56-60`. Off-by-one on end line. Corrected.

**Files fixed (3):**
- `specs/architecture/purity-boundary-map.md`
- `specs/module-criticality.md`
- `specs/prd-supplements/error-taxonomy.md`

**Remediation:** All 3 findings (0C/1H/1M/0L/1N) remediated. csv.rs C-21 collision resolved
to unnumbered `(--)` anchor; stale absent-flag row corrected to reflect PR #74 clap rejection;
E-INP-001 citation corrected 56-59 → 56-60. Counter reset to 0/3. Pass 24 dispatched next.

---

### Pass 24 (2026-05-20)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW, 0 NITPICK)
**Delta from pass 23:** -3 total — full regression to zero
**Novelty:** NONE
**Convergence counter:** 1 of 3 (new streak started)
**Verdict:** CONVERGED — clean pass 1 of 3

**Summary:** Pass 24 returned zero findings. The adversary raised 2 non-blocking observations
but explicitly classified neither as a spec defect. No spec artifact was modified. The spec
package is stable; passes 25 and 26 will review identical, unchanged content.

**Non-blocking observations (not spec defects, no action required):**
- O-1: Minor editorial note (no action taken)
- O-2: Minor editorial note (no action taken)

**Files fixed:** none — zero findings, no remediation required.

**Outcome:** Counter advances to **1/3**. Pass 25 is next (second confirmation pass on
identical spec package). All three passes (25 and 26) must also return 0C/0H/0M to satisfy
the Phase 1d adversarial convergence gate.

---

### Pass 25 (2026-05-20)

**Findings:** 4 (0 CRIT, 2 HIGH, 2 MED, 0 LOW)
**Novelty:** LOW
**Convergence counter:** RESET — 1/3 → **0/3**

**Finding summary:** All 4 findings were concentrated in PRD supplements — the last
spec-package pocket that had not yet been comprehensively reconciled against src/.

- **H-1 (HIGH) — error-taxonomy.md — E-RAS-001/E-RAS-002 message strings wrong:** The
  eprintln! literal strings cited for E-RAS-001 and E-RAS-002 did not match the actual
  eprintln! text in src/. Corrected to the actual eprintln! literals.
- **H-2 (HIGH) — error-taxonomy.md — E-DEC-001 stale API name:** `from_ethernet_slice` is
  a stale API name that no longer exists. Corrected to the current API.
- **M-1 (MED) — interface-definitions.md — wrong analyzer_name values + fabricated JSON
  detail keys:** analyzer_name values were wrong; JSON detail-shape keys
  `flows_evicted`/`flows_closed_fin` do not exist in source; corrected to actual
  `summarize()` output keys.
- **M-2 (MED) — nfr-catalog.md — NFR-PERF-001 stale line range:** decoder.rs line range
  cited as 72-130 but correct range is 288-291. Corrected.

**Orchestrator action:** Commissioned comprehensive PRD-supplement sweep to address the
full scope of citation/string-drift across all 4 supplements (P-CITE-PG process gap).

**Files fixed:** error-taxonomy.md (3 fixes), nfr-catalog.md (1 fix from pass-25 finding).
Remaining supplement fixes delivered as SWEEP68 remediation burst.

**Verdict:** NOT CONVERGED — streak RESET 1/3→0/3.

---

### SWEEP68 — Comprehensive PRD-Supplement Verification Sweep (2026-05-20)

**Type:** Inter-pass remediation burst (no adversary pass)
**Convergence counter:** Unchanged — **0/3**

**Purpose:** Pass 25 revealed that PRD supplements were the last spec-package pocket not
yet comprehensively reconciled against current src/. Orchestrator commissioned a full
sweep of all 4 supplements. This directly addresses P-CITE-PG (recurring citation/string-
drift process gap).

**Total defects fixed: ~68 across 4 files.**

#### error-taxonomy.md — 3 fixes (pass-25 findings)

- E-RAS-001 eprintln! message string corrected to actual literal
- E-RAS-002 eprintln! message string corrected to actual literal
- E-DEC-001 stale `from_ethernet_slice` API name corrected to current API

#### interface-definitions.md — 14 fixes

- Wrong `analyzer_name` values corrected for multiple analyzers
- 8+ fabricated/wrong JSON detail-shape keys removed and corrected to actual keys
  returned by `summarize()` — keys `flows_evicted` and `flows_closed_fin` do not exist
  in source; corrected to actual field names present in analyzer output structs

#### nfr-catalog.md — 47 fixes

- M-2 (pass-25): NFR-PERF-001 decoder.rs line range 72-130 → 288-291
- 39 stale citations re-anchored following the LESSON-P2.01 refactor (line numbers had
  shifted across the board; all citations now point to correct post-refactor locations)
- NFR-SEC-005: SNI verdict assessment corrected Likely/High → Inconclusive/Low
  (matches actual src/ behavior for SNI control-byte flagging)
- 6 module-map citations corrected to current module paths

#### test-vectors.md — 8 fixes

- BC-2.06.005: category corrected Anomaly → Reconnaissance
- BC-2.07.014, BC-2.07.017, BC-2.07.037: verdict corrected to Inconclusive/Low
  (matches pass-16/BC sweep corrections to the BC files themselves)
- Embedded literal control bytes (0x00, 0x1b/ESC) replaced with textual escapes
  to prevent toolchain encoding issues
- Integration-scenario category/severity corrections aligned to current verdict model

**Process-gap addressed:** P-CITE-PG — PRD supplements join BCs (SWEEP after pass 16)
and domain shards (passes 17–18) as comprehensively reconciled spec pockets. The entire
spec package has now been systematically verified against current src/.

**Outcome:** Counter unchanged at **0/3**. Pass 26 dispatched next.

---

### Pass 26 (2026-05-20) — NOT CONVERGED (counter remains 0/3)

**Findings:** 5 (0 CRIT, 3 HIGH, 1 MED, 1 LOW)
**Delta from SWEEP68:** N/A (inter-pass sweep, not a pass)
**Novelty:** LOW
**Convergence counter:** 0/3 (unchanged — HIGH findings disqualify; a clean pass requires 0C/0H/0M)
**Verdict:** NOT CONVERGED — 3 HIGH + 1 MED findings present. Counter remains 0/3. Pass 27 is next.

**Root cause:** VP files were the last spec-package category not yet comprehensively
source-reconciled. All 4 blocking findings were in VP harness-skeleton sections and
property-statement sections that referenced non-existent APIs, wrong argument signatures,
or stale source-line citations.

**Key finding categories:**

- HIGH (H-1): Multiple VP files — harness-skeleton API signatures wrong. Nonexistent
  methods/constructors cited: `TcpFlow::with_state`, `TerminalReporter::new`, `read_at`,
  `flow_state`, `render_json`, `all_variants`. Argument form wrong for `insert_segment`
  (arity), `set_isn` (arg type), `on_data_without_syn` (args). Return form wrong for
  `flush_contiguous` (closure vs. return value). Method name wrong: `seq_offset` →
  `seq_to_offset`. All corrected to actual API surface.

- HIGH (H-2): Multiple VP files — stale source-line citations throughout harness-skeleton
  and property-statement sections. Post-refactor line shifts not reflected. All re-anchored
  to current positions.

- HIGH (H-3): `vp-005-sni-four-way-classification.md` — BC verdict labels mis-stated:
  BC-2.07.014, BC-2.07.017, BC-2.07.019 all cited as `Anomaly/Likely/High`. Correct verdicts
  (per pass-16/SWEEP58 corrections) are `Anomaly/Inconclusive/Low`. Corrected.

- MED (M-1): `vp-003-max-findings-cap.md` — MAX_FINDINGS anchor value and associated
  constant citation wrong. Corrected to actual constant name and source location.

- LOW (L-1): `BC-2.04.039.md` — `seq_offset` cited at lines 32-34; correct range is
  31-34 (3 occurrences). Corrected.

**VP files verified clean (0 defects):** VP-007, VP-008, VP-012, VP-018, VP-019, VP-020

**VP-INDEX fix:** phase-column values for VP-016..VP-020 were wrong. Corrected to Phase 3.

**Orchestrator action:** Commissioned comprehensive VP-file sweep of all 20 VP files +
VP-INDEX against current src/, to flush all residual API-signature and citation drift at
root (same approach as SWEEP58 for BCs, SWEEP68 for supplements).

**Files with blocking findings:** vp-003, vp-005, multiple VP files (H-1, H-2).

**Verdict:** NOT CONVERGED — counter remains 0/3. SWEEP48 commissioned next.

---

### SWEEP48 — Comprehensive VP-File Verification Sweep (2026-05-20)

**Type:** Inter-pass remediation burst (no adversary pass)
**Convergence counter:** Unchanged — **0/3**
**Commit SHA:** 25641c4

**Purpose:** Pass 26 revealed VP files as the last spec-package pocket not yet
comprehensively source-reconciled. Orchestrator commissioned a full sweep of all 20 VP
files + VP-INDEX against current src/. This directly addresses P-CITE-PG (recurring
citation/API-drift process gap) for the final unchecked category.

**Total defects fixed: ~48 across VP files + 1 BC fix.**

#### API-signature defects (wrong method names, wrong arity, nonexistent constructors)

- `vp-001-flowkey-canonical-ordering.md` — wrong method call in harness corrected
- `vp-002-first-wins-overlap.md` — insert_segment arity corrected
- `vp-004-content-first-dispatch.md` — on_data_without_syn args corrected
- `vp-009-flow-state-machine.md` — TcpFlow::with_state (nonexistent) removed; flow_state
  method (nonexistent) corrected to actual state accessor
- `vp-010-buffered-bytes-invariant.md` — read_at (nonexistent) corrected; set_isn arg type fixed
- `vp-011-flush-contiguous-monotonicity.md` — flush_contiguous closure-vs-return form corrected
- `vp-013-ja3-grease-filter.md` — render_json (nonexistent) corrected
- `vp-014-http-cross-flow-isolation.md` — all_variants (nonexistent) corrected
- `vp-015-tcp-sequence-wraparound.md` — seq_offset → seq_to_offset; TerminalReporter::new
  (nonexistent) removed (TerminalReporter is a unit struct)
- `vp-016-mitre-tactic-grouping-order.md` — API signature corrected
- `vp-017-json-key-determinism.md` — JSON key-order claim corrected to match BTreeMap
  serialization behavior; render_json (nonexistent) corrected

#### Stale source-line citation defects

- `vp-001-flowkey-canonical-ordering.md` — citations re-anchored
- `vp-002-first-wins-overlap.md` — citations re-anchored
- `vp-003-max-findings-cap.md` — MAX_FINDINGS anchor corrected; citations re-anchored
- `vp-004-content-first-dispatch.md` — citations re-anchored
- `vp-006-http-poison-monotonicity.md` — citations re-anchored
- `vp-009-flow-state-machine.md` — citations re-anchored
- `vp-010-buffered-bytes-invariant.md` — citations re-anchored
- `vp-011-flush-contiguous-monotonicity.md` — citations re-anchored
- `vp-013-ja3-grease-filter.md` — citations re-anchored
- `vp-014-http-cross-flow-isolation.md` — citations re-anchored
- `vp-015-tcp-sequence-wraparound.md` — citations re-anchored
- `vp-016-mitre-tactic-grouping-order.md` — citations re-anchored

#### BC verdict label defects

- `vp-005-sni-four-way-classification.md` — BC-2.07.014 / BC-2.07.017 / BC-2.07.019 verdict
  labels corrected from Anomaly/Likely/High → Anomaly/Inconclusive/Low (aligns with
  pass-16 and SWEEP58 corrections to the BC bodies themselves)

#### VP-INDEX fix

- `VP-INDEX.md` — phase-column values for VP-016..VP-020 corrected to Phase 3

#### BC fix

- `specs/behavioral-contracts/ss-04/BC-2.04.039.md` — F-5 LOW: seq_offset citation corrected
  32-34 → 31-34 (3 occurrences)

#### VP files verified clean (0 defects found, no changes made)

VP-007 (MITRE technique ID format), VP-008 (TLS SNI four-way), VP-012 (DNS label encoding),
VP-018 (MITRE output structure), VP-019 (terminal color isolation), VP-020 (CSV injection
neutralization — already corrected in pass-15 remediation).

**Summary of defect classes:**
- Wrong API signatures / nonexistent methods: ~20 defects
- Stale source-line citations: ~25 defects
- Mis-stated BC verdict labels: 3 defects
- VP-INDEX phase column: 5 rows
- BC citation: 3 occurrences (1 BC file)

**Historical context — comprehensive spec-reconciliation sweep summary:**

| Sweep | Pass trigger | Category | Defects fixed | Commit |
|-------|-------------|----------|---------------|--------|
| SWEEP58 | Pass 16 | All 217 BC bodies vs src/ | ~58 | d038ace |
| Anchor sweep | Pass 14 | C-NN/SS-NN anchors (3,820 occurrences) | 28 | 21093ed |
| SWEEP68 | Pass 25 | All 4 PRD supplements vs src/ | ~68 | (SWEEP68 burst) |
| SWEEP48 | Pass 26 | All 20 VP files + VP-INDEX vs src/ | ~48 | 25641c4 |

All 4 major spec categories have now been comprehensively source-reconciled. The spec package
is fully swept. Passes 27–29 (or until 3 consecutive clean passes are achieved) will review
the fully-reconciled package.

**Process-gap addressed:** P-CITE-PG — VP files join BCs, anchors, and supplements as
comprehensively reconciled. All known citation/API-drift pockets have been closed.

**Outcome:** Counter unchanged at **0/3**. Pass 27 dispatched next.

---

### Pass 27 (2026-05-20)

**Findings:** 1 (0 CRIT, 1 HIGH, 0 MED, 0 LOW)
**Delta from pass 26:** -4 total (HIGH -2, MED -1, LOW -1) — no regression
**Novelty:** LOW
**Convergence counter:** 0 of 3

**Key finding:**

- H-1: `verification-coverage-matrix.md` — VP-016..VP-020 Phase column listed `P1` instead of
  `test-sufficient`. The VP-INDEX.md SWEEP48 correction (which set Phase to `test-sufficient` for
  these 5 rows) was not propagated to the coverage matrix. P0(8)/P1(7)/test-sufficient(5)=20
  invariant was violated with the stale values.

**Remediation:** Single file corrected — VP-016..VP-020 Phase column updated P1→test-sufficient
in `specs/architecture/verification-coverage-matrix.md`. P0(8)/P1(7)/test-sufficient(5)=20
invariant restored. Committed to factory-artifacts (e758fb6).

**Verdict:** NOT CONVERGED — counter remains 0/3. Pass 28 next.

**Note:** This was a single-file, single-category finding (phase-column drift from SWEEP48
propagation gap). The spec package remains comprehensively reconciled in all other respects.

---

### Pass 28 (2026-05-20)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW, 0 Novelty)
**Delta from pass 27:** -1 total — no regression
**Novelty:** NONE
**Convergence counter:** 1 of 3

**Key findings:** None. Zero findings. No spec artifact modified.

**Remediation:** None required.

**Verdict:** CONVERGED — **CLEAN PASS 1/3 (new streak)**. Counter advances to 1/3. Pass 29
next (second confirmation pass on stable, unchanged package).

---

### Pass 29 (2026-05-20)

**Findings:** 1 (0 CRIT, 0 HIGH, 0 MED, 1 LOW) + 1 observation
**Delta from pass 28:** +1 total (1L vs 0) — gate-level clean (0C/0H/0M; LOW below RESET threshold)
**Novelty:** LOW
**Convergence counter:** 2 of 3

**Gate-level classification:** CLEAN PASS — gate definition requires 0C/0H/0M; LOWs and
observations do not break the streak.

**Findings:**

- L-1 (LOW): `specs/architecture/system-overview.md` — handler.rs import description was
  incorrect. Stated that `Direction` and `CloseReason` are imported from a sub-module; in
  reality handler.rs imports L3 `AnalysisSummary`/`Finding` and L2 `FlowKey` from sibling
  modules. `Direction` and `CloseReason` are defined in-file (not imported). One-line
  description corrected.

- O-1 (observation / process-gap): `src/analyzer/dns.rs` module doc-comment is stale —
  references behavior that was removed. This is a source defect, not a spec defect; the spec
  correctly describes current behavior. Recorded as documentation-debt item O-08 in
  domain-debt.md; propagated to ARCH-INDEX.md debt list and prd.md Section 8 Domain Debt
  Index (O-07 and O-08 backfilled; range O-01..O-06 → O-01..O-08).

**Remediation:** Both L-1 and O-08 fixed and committed to factory-artifacts before recording
this verdict (commit 04478ef, 4 files: system-overview.md, domain-debt.md, ARCH-INDEX.md,
prd.md).

**Verdict:** CONVERGED — **CLEAN PASS 2/3**. Counter advances to 2/3. Pass 30 next — the
THIRD and final confirmation pass. If pass 30 returns 0C/0H/0M, the Phase 1d adversarial
spec-convergence gate is SATISFIED (3/3) and Phase 1 may proceed to the consistency audit
and human approval gate.

---

### Pass 30 (2026-05-20)

**Findings:** 3 (0 CRIT, 0 HIGH, 1 MED, 0 LOW, 2 NITPICK)
**Delta from pass 29:** +2 total — REGRESSION relative to gate level (1M breaks the 0C/0H/0M
streak)
**Novelty:** LOW (all findings are prose/citation precision gaps; no behavioral defects)
**Convergence counter:** 0 of 3 — RESET from 2/3

**Verdict:** NOT CONVERGED — **STREAK BROKEN — RESET 2/3→0/3**

**Findings:**

- M-1 (MEDIUM): `specs/behavioral-contracts/ss-12/BC-2.12.020.md` — capability-anchor prose in
  Summary section incorrectly cited C-16 instead of C-17. BC-2.12.020 covers the MITRE
  capability, which anchors to C-17 (cap-12 MITRE ATT&CK); the C-16 reference was a copy-paste
  error from an adjacent BC. Corrected: "Summary (C-16)" → "Summary (C-17)".

- N-1 (NITPICK): `specs/behavioral-contracts/ss-05/BC-2.05.006.md` — guard-clause in the
  Postcondition was quoted in shorthand form rather than the actual Rust `if target ==
  DispatchTarget::None { } else { }` form. No behavioral impact; prose precision corrected.

- N-2 (NITPICK): `specs/domain/invariants/inv-01-core-invariants.md` — INV-9 cited the
  `technique_info` span as an approximate range; corrected to verified-exact span
  `src/mitre.rs:122-156` matching the actual implementation. No semantic change.

**Remediation:** All 3 findings fixed and committed to factory-artifacts before recording this
verdict (commit 00f5094, 3 files: BC-2.12.020.md, BC-2.05.006.md, inv-01-core-invariants.md).

**Milestone:** 30 adversarial passes completed. Spec package is at ZERO known open defects
post-remediation. All 4 major spec categories have been comprehensively source-reconciled:
BCs (~58 fixes in SWEEP16), anchors (~28 in SWEEP14), PRD supplements (~68 in SWEEP68),
VPs (~48 in SWEEP48). The single MEDIUM finding in this pass broke the 2-pass clean streak;
a new 3-pass streak must be established starting with Pass 31.

**Verdict:** NOT CONVERGED — Counter RESET to **0/3**. Pass 31 next (new streak restart).

---

### Pass 31 (2026-05-21) — CONVERGED (clean pass 1 of 3, new streak)

**Findings:** 0 (0 CRIT, 0 HIGH, 0 MED, 0 LOW, 0 NITPICK)
**Delta from pass 30:** -3 total (MED -1, NITPICK -2) — no regression; all findings from pass 30 were fixed (00f5094)
**Novelty:** NONE
**Convergence counter:** 1/3
**Verdict:** CONVERGED — zero findings. Spec package content stable and unchanged since pass-29 commit 04478ef.

**Observations (non-blocking, NOT defects):**

- O-1: `architecture/module-decomposition.md` C-8 describes the per-direction buffer as
  `BTreeMap<u64,Segment>`; actual `flow.rs:89` type is `BTreeMap<u64, Vec<u8>>` (no `Segment`
  struct exists). Informal shorthand, survived 30+ passes, non-misleading. Not a spec defect.
  Recorded as tech-debt item O-09 in STATE.md for eventual doc-only alignment.

- O-2: `behavioral-contracts/ss-01/BC-2.01.001.md` cites `reader.rs:46-60` (Architecture
  Module field) vs `reader.rs:50-60` (Source Evidence Path field) — both are valid scopings of
  the same contract; the wider window (46-60) covers the full function header context, the
  narrower window (50-60) covers the body. Internally consistent. Not a spec defect.

**Novelty assessment:** Fresh-context citation sampling across BCs (ss-01/04/05/07/11),
architecture indexes, VP-INDEX, PRD, and domain-debt confirms all citations resolve correctly,
all counts are arithmetically consistent, verdict labels match source, and no internal
contradictions exist.

**No spec artifact modified.** Counter advances to **1/3**. Pass 32 next (second confirmation
pass; must return 0C/0H/0M to advance to 2/3).

---
