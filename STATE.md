---
pipeline: PHASE_2_STORY_DECOMPOSITION
phase: phase-2-story-decomposition
product: wirerust
mode: brownfield
timestamp: 2026-05-21T00:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
remediation_completed: 2026-05-19T22:30:00Z
phase_1_started: 2026-05-20T00:00:00Z
phase_1_spec_package_committed: 2026-05-20
phase_1_human_approved: "2026-05-21"
phase_1_completed: "2026-05-21"
p8_defer_resolved: "2026-05-21"
phase_2_started: "2026-05-21"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
story_adversary_convergence_counter: 0/3
story_adversary_pass_1_date: "2026-05-21"
story_adversary_pass_1_verdict: NOT_CONVERGED
story_adversary_pass_1_findings: "11 (1C/3H/3M/2L/2N) — all blocking findings remediated; 3 process-gap NITPICKs deferred for cycle-close codification. Pass 2 next."
adversary_pass_30_date: "2026-05-20"
adversary_pass_30_verdict: NOT_CONVERGED
adversary_pass_30_findings: "3 (0C/0H/1M/0L/2N) — STREAK RESET 2/3→0/3. M-1 BC-2.12.020 C-16→C-17 prose; N-1 BC-2.05.006 guard-clause quote; N-2 inv-01 INV-9 citation. All 3 fixed (00f5094). Pass 31 next."
adversary_pass_31_date: "2026-05-21"
adversary_pass_31_verdict: CONVERGED
adversary_pass_31_findings: "0 (0C/0H/0M/0L/0N) — CLEAN PASS 1/3 (new streak after pass-30 reset). Zero findings; 2 non-blocking observations. Pass 32 next."
adversary_pass_32_date: "2026-05-21"
adversary_pass_32_verdict: CONVERGED
adversary_pass_32_findings: "0 blocking (0C/0H/0M/0L/1N) — CLEAN PASS 2/3. N-1 NITPICK (domain-spec §8 ADR-0004 omission) non-blocking, deferred to pre-approval polish. Pass 33 final."
adversary_pass_33_date: "2026-05-21"
adversary_pass_33_verdict: CONVERGED
adversary_pass_33_findings: "0 blocking (0C/0H/0M/0L/2N) — CLEAN PASS 3/3. ADVERSARIAL CONVERGENCE GATE SATISFIED. 2 NITPICKs (BC-2.12.016 doc-comment range 304-310→304-311; BC-2.11.021 csv.rs range 40-44→40-45) non-blocking, deferred to pre-approval polish."
convergence_trajectory: "17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3→5→5→2→4→3→0→3→0→4→SWEEP68→5→SWEEP48→1→0→0→3→0→0→0"
consistency_audit: CONSISTENT
consistency_audit_date: "2026-05-21"
input_drift_check: CLEAN
input_drift_check_date: "2026-05-21"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_2_STORY_DECOMPOSITION — Phase 1 COMPLETE. All 4 gates PASSED; human-approved
2026-05-21. (1) Adversarial spec-convergence gate **SATISFIED** (3/3; passes 31/32/33 CONVERGED,
0C/0H/0M; 33 passes total; ZERO blocking defects). (2) Consistency audit **CONSISTENT** — 5
findings (F-1 MAJOR/F-2–F-4 MINOR/F-5 NITPICK); all remediated; re-audit CONSISTENT. (3)
Input-hash drift check **CLEAN** — 5 hashes bumped; re-scan MATCH=5/STALE=0. (4) Human approval
**GRANTED 2026-05-21**. P8-DEFER VP back-reference back-fill **DONE** (2026-05-21): all 217 BC
files updated; 69 BCs now cite formal VP IDs, 148 show `—`; BC versions bumped to 1.2. Pipeline
now entering Phase 2 story decomposition.

**Current develop HEAD:** 0082a0c (PR #99 — CLAUDE.md governance pointer).

**Mode:** brownfield (in-repo: target == reference).

**Test suite:** 282 passing on develop. `cargo fmt --check`, `cargo clippy`,
`cargo test --all-targets`, `cargo audit`, `cargo deny` all green.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** — all 4 gates + human approval 2026-05-21; P8-DEFER back-fill DONE | 20 L2 shards, 217 BCs, 11 arch files, 20 VPs, 4 supplements; trajectory: `17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3→5→5→2→4→3→0→3→0→4→SWEEP68→5→SWEEP48→1→0→0→3→0→0→0` |
| Phase 2 — Story Decomposition | **IN PROGRESS** — Steps A–F COMPLETE; Step G adversarial Pass 1 NOT_CONVERGED (remediated); Pass 2 next; convergence counter 0/3 | 10 epics, 217/217 BCs traced to ≥1 story, 48 stories, 78 edges, 27 waves, acyclic, 282 story points; 100 holdout scenarios; decomposition gate PASSED; story-adversary Pass 1 findings: 1C/3H/3M/2L/2N — all blocking remediated (VP matrix, VP assignments, HS subtotals, BC body tables, depends_on/blocks, STORY-013 AC-008); 3 process-gap NITPICKs deferred |
| Phase 3 — TDD Implementation | NOT STARTED | — |
| Phase 4 — Holdout Evaluation | NOT STARTED | — |
| Phase 5 — Adversarial Refinement | NOT STARTED | — |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 1 — Spec Crystallization (PASSED — 2026-05-21)

### Spec Package Contents

| Artifact | Location | Count |
|----------|----------|-------|
| L2 Domain Specification | `.factory/specs/domain/` | 20 shards (1 index, 12 cap, 5 entity, 1 inv, 1 debt) |
| L3 PRD | `.factory/specs/prd.md` | 1 file |
| Behavioral Contracts | `.factory/specs/behavioral-contracts/ss-01..ss-13/` | 217 BCs across 12 subsystems (no ss-03) |
| BC Index | `.factory/specs/behavioral-contracts/BC-INDEX.md` | 1 file |
| Architecture Package | `.factory/specs/architecture/` | 9 files + ARCH-INDEX.md |
| Module Criticality | `.factory/specs/module-criticality.md` | 1 file |
| DTU Assessment | `.factory/specs/dtu-assessment.md` | DTU_REQUIRED: false |
| Verification Properties | `.factory/specs/verification-properties/vp-001..vp-020` | 20 VPs + VP-INDEX.md |
| PRD Supplements | `.factory/specs/prd-supplements/` | 4 files (interface-definitions, error-taxonomy, test-vectors, nfr-catalog) |

**Architecture files:** ARCH-INDEX.md, system-overview.md, module-decomposition.md,
dependency-graph.md, api-surface.md, purity-boundary-map.md,
verification-architecture.md, tooling-selection.md, verification-coverage-matrix.md.

### BC Breakdown by Subsystem

| SS | Count |
|----|-------|
| SS-01 | 8 |
| SS-02 | 15 |
| SS-04 | 54 |
| SS-05 | 9 |
| SS-06 | 26 |
| SS-07 | 37 |
| SS-08 | 4 |
| SS-09 | 6 |
| SS-10 | 9 |
| SS-11 | 24 |
| SS-12 | 21 |
| SS-13 | 4 |
| **Total** | **217** |

### Adversarial Spec-Convergence Log

| Pass | Date | Findings | Verdict | Status |
|------|------|----------|---------|--------|
| 1 | 2026-05-20 | 17 (2C/8H/5M/2L) | NOT_CONVERGED | REMEDIATED — all 17 fixed |
| 2 | 2026-05-20 | 13 (0C/4H/6M/3L) | NOT_CONVERGED | REMEDIATED — all blocking fixed; 2 deferred (non-blocking) |
| 3 | 2026-05-20 | 7 (0C/3H/2M/2N) | NOT_CONVERGED | REMEDIATED — all 7 fixed |
| 4 | 2026-05-20 | 19 (4C/5H/5M/3L/2N) | NOT_CONVERGED | REMEDIATED — all 19 fixed; +5 CsvReporter BCs (020–024) |
| 5 | 2026-05-20 | 8 (1C/2H/3M/2L) | NOT_CONVERGED | REMEDIATED — all 8 fixed; NUL byte, stale --services, count drift |
| 6 | 2026-05-20 | 3 (0C/3H/0M/0L) | NOT_CONVERGED | REMEDIATED — all 3 fixed; component-ID anchors (95 BCs), BC-INDEX titles (34 rows), INV-1 citation |
| 7 | 2026-05-20 | 13 (1C/3H/4M/3L/2N) | NOT_CONVERGED | REMEDIATED — all 13 fixed; entity shards, em-dash, SS-13 anchor, cap-05 token, VP-008 |
| 8 | 2026-05-20 | 8 (0C/2H/3M/2L/1N) | NOT_CONVERGED | REMEDIATED — all 8 fixed; vp-008 arg order+IPv6, stale citations, E-RAS-005 counter |
| 9 | 2026-05-20 | 4 (0C/1H/1M/2L) | NOT_CONVERGED | REMEDIATED — all 4 fixed; stale citations BC-2.04.054/027, prd error-categories, ARCH-INDEX debt note |
| 10 | 2026-05-20 | 6 (0C/3H/3M/0L) | NOT_CONVERGED | REMEDIATED — all 6 fixed; dependency table vs Cargo.toml, Reporter trait, ParsedPacket struct, CAP-03/SS IDs |
| 11 | 2026-05-20 | 1L/4obs (0C/0H/0M/1L) | **CONVERGED** | CLEAN PASS 1/3 — polish applied (L-1 BC ref, O-1 pseudocode, O-2 struct-variant, O-3 exit-2, O-4 dev-deps) |
| 12 | 2026-05-20 | 6 (0C/1H/1M/2L/2N) | NOT CONVERGED | RESET 1/3→0/3 — F-1 C1 postcondition, F-2 stale citation+unsupported claim, F-3/F-4 IPv6 bracket+citation, 2N csv.rs off-by-one. All fixed. |
| 13 | 2026-05-20 | 5 (0C/2H/0M/3L) | NOT CONVERGED | Counter remains 0/3 — H-1 ent-05 7 stale anchors, H-2 INV-4 ADR-0003 anchor, C-1 ARCH-INDEX C-count, prd.md BC-2.07.004 one-liner, 1N. All fixed. |
| 14 | 2026-05-20 | 3 (0C/1H/0M/1L/1N) | NOT CONVERGED | Counter remains 0/3 — H-1 summary.rs C-16→C-17 mis-anchor (domain-spec+cap-12, 4 sites), L-1 ent-04 E-39b CsvReporter missing from entity index (entity count 41→42), N-1 BC-2.12.005 cli.rs citation 61-105→61-106. All fixed. |
| SWEEP | 2026-05-20 | — (inter-pass) | REMEDIATION BURST | Proactive anchor-consistency sweep: 3,820 occurrences audited; 28 mis-anchors fixed (3 C-ID in BC-2.12.018/019/021 + 25 capability-column in prd.md). Recurring C-ID/capability-anchor defect class addressed at root (passes 4,6,10,13,14). Counter remains 0/3. Pass 15 next. |
| 15 | 2026-05-20 | 4 (0C/1H/2M/1N) | NOT CONVERGED | Counter remains 0/3 — H-1 VP-020 test_csv_safe_values_unchanged using wrong API; M-1 VP-020 Property Statement pt 3 mis-scoped AnalysisSummary; M-2/N-1 module-decomposition reporter Purity column wrong (effectful→pure). All fixed. |
| 16 | 2026-05-20 | 3 (1C/1M/1L) | NOT CONVERGED | Counter remains 0/3 — C-1 BC-2.07.037 Postcondition 4 verdict Anomaly/Likely/High→Anomaly/Inconclusive/Low; M-1 stale correction-notes removed from BC-2.07.017/019; L-1. All fixed. |
| SWEEP | 2026-05-20 | — (inter-pass) | REMEDIATION BURST | BC-vs-source sweep: all 217 BCs re-verified against current src/; ~58 defects fixed (off-by-one citations + ~6 semantic); addresses P-CITE-PG at root. 37 BC body files committed (d038ace). Counter: 0/3 unchanged. Pass 17 next. |
| 17 | 2026-05-20 | 5 (0C/2H/1M/1L/1N) | NOT CONVERGED | Counter remains 0/3 — all 5 findings in ent-04 only; ZERO BC defects (BC sweep held). F-1 HashMap→BTreeMap; F-2 false inline-test claim+stale range; F-3 BC-RPT-007→BC-RPT-001; F-4 line range 12-17→38-50; F-5 Verdict cite 32-40→30-40. All fixed (0c16cad). |
| 18 | 2026-05-20 | 5 (0C/3H/2L) | NOT CONVERGED | Counter remains 0/3 — all 5 findings were stale-anchor drift from PR #75 `//!` header line shifts in last unreconciled domain shards. H-1 ent-01 8 entity anchors re-resolved; H-2 ent-04 6 cross-file anchors re-resolved; H-3 cap-10 unknown-ID rendering anchor corrected; L-1 ent-02 component range C-6..C-9→C-6..C-9,C-15; L-2 domain-spec "~282"→"282". All fixed (fc28b69). |
| 19 | 2026-05-20 | 2 (0C/0H/2M/0L) | NOT CONVERGED | Counter remains 0/3 — M-1 purity-boundary-map.md 3 reporters misclassified Effectful-shell (should be Pure-core per module-decomposition.md); M-2 dependency-graph.md test-count statement inconsistent (corrected to "264 in tests/ + 18 inline = 282"). Package described as "overwhelmingly clean". All fixed (f913004). |
| 20 | 2026-05-20 | 4 (0C/0H/2M/1L/1N) | NOT CONVERGED | Counter remains 0/3 — F-1/F-2 VP-007 SEEDED_IDS corrected to real 15 MITRE IDs + citation 99-129→122-156; F-3 BC-2.12.008 main.rs 57-58→57-59 (5 instances); F-4 mitre_technique regex tightened. All spec-precision gaps; no behavioral defects. All fixed. |
| 21 | 2026-05-20 | 3 (0C/0H/1M/0L/2N) | NOT CONVERGED | Counter remains 0/3 — F-1 (MED) C-10 re-anchored SS-08→SS-05 (module-decomposition.md + ARCH-INDEX follow-up); O-1 (NITPICK) prd.md removed-flags completed; O-2 (NITPICK) BC-2.07.016 one-liner aligned. No behavioral defects. All fixed. |
| 22 | 2026-05-20 | 3 (0C/0H/0M/2L/1N) | **CONVERGED** | **CLEAN PASS 1/3** — LOW-1 BC-2.12.005 H1 broadened (all 9 reassembly flags; BC-INDEX + prd.md synced); LOW-2 BC-2.07.004 citation ranges tightened; NITPICK oversized-record guard aligned tls.rs:643-653 (BC-2.07.004 + error-taxonomy.md E-ANA-003). Counter: **1/3**. |
| 23 | 2026-05-20 | 3 (0C/1H/1M/0L/1N) | NOT CONVERGED | **STREAK BROKEN — RESET 1/3→0/3** — H-1 csv.rs C-21 anchor collision fixed (purity-boundary-map.md → unnumbered `(--)`) ; M-1 stale absent-flag row corrected to "removed by PR #74; clap rejects" (module-criticality.md); N-1 E-INP-001 citation 56-59→56-60 (error-taxonomy.md). All fixed. Counter: **0/3**. |
| 24 | 2026-05-20 | 0 (0C/0H/0M/0L/0N) | **CONVERGED** | **CLEAN PASS 1/3 (new streak)** — zero findings; 2 non-blocking observations only, neither a spec defect. No spec artifact modified. Counter: **1/3**. Pass 25 next (second confirmation pass on stable, unchanged package). |
| 25 | 2026-05-20 | 4 (0C/2H/2M/0L) | NOT CONVERGED | **STREAK RESET 1/3→0/3** — all 4 findings in PRD supplements. Commissioned SWEEP68. Counter: **0/3**. |
| SWEEP68 | 2026-05-20 | — | REMEDIATION BURST | ~68 defects in all 4 PRD supplements vs src/. Supplements comprehensively reconciled. Counter: **0/3** unchanged. |
| 26 | 2026-05-20 | 5 (0C/3H/1M/1L) | NOT CONVERGED | Counter remains **0/3** — all 4 blocking findings in VP files (wrong API signatures, stale citations, mis-stated verdict labels). Commissioned SWEEP48. |
| SWEEP48 | 2026-05-20 | — | REMEDIATION BURST | ~48 defects across all 20 VP files + VP-INDEX + BC-2.04.039 vs src/. All 4 major spec categories now comprehensively reconciled: BCs (~58), anchors (~28), supplements (~68), VPs (~48). SHA: 25641c4. Counter: **0/3** unchanged. |
| 27 | 2026-05-20 | 1 (0C/1H/0M/0L) | NOT CONVERGED | Counter remains **0/3** — H-1 verification-coverage-matrix.md VP-016..020 Phase column P1→test-sufficient (P0(8)/P1(7)/test-sufficient(5)=20 invariant restored). Fixed (e758fb6). |
| 28 | 2026-05-20 | 0 (0C/0H/0M/0L/0N) | **CONVERGED** | **CLEAN PASS 1/3** — zero findings; no spec artifact modified. Counter advances to **1/3**. |
| 29 | 2026-05-20 | 1 (0C/0H/0M/1L/1obs) | **CONVERGED** | **CLEAN PASS 2/3** — L-1 system-overview.md handler.rs import desc corrected; O-08 dns.rs stale doc-comment recorded as debt. Both fixed before commit 04478ef. Counter advances to **2/3**. |
| 30 | 2026-05-20 | 3 (0C/0H/1M/0L/2N) | NOT CONVERGED | **STREAK RESET 2/3→0/3** — M-1 BC-2.12.020 C-16→C-17 prose anchor; N-1 BC-2.05.006 guard-clause quote; N-2 inv-01 INV-9 mitre.rs:122-156 citation. All 3 fixed (00f5094). Counter: **0/3**. 30 passes total; ZERO open defects. Pass 31 next. |
| 31 | 2026-05-21 | 0 (0C/0H/0M/0L/0N) | **CONVERGED** | CLEAN PASS 1/3 — zero findings; 2 non-blocking observations (C-8 BTreeMap shorthand, BC-2.01.001 dual scoping), neither a spec defect; no spec artifact modified. Counter advances to **1/3**. Pass 32 next. |
| 32 | 2026-05-21 | 0 blocking (0C/0H/0M/0L/1N) | **CONVERGED** | CLEAN PASS 2/3 — zero blocking findings; 1 NITPICK N-1 (domain-spec §8 omits ADR 0004; defensibly correct-by-construction) deferred to pre-approval polish; package left byte-identical for pass 33. Counter advances to **2/3**. Pass 33 final. |
| 33 | 2026-05-21 | 0 blocking (0C/0H/0M/0L/2N) | **CONVERGED** | CLEAN PASS 3/3 — ADVERSARIAL CONVERGENCE GATE SATISFIED. Zero blocking findings; 2 NITPICKs (BC-2.12.016 + BC-2.11.021 doc-comment/brace range slack) deferred to pre-approval polish. 33 passes total. |
| CONSISTENCY-REMEDIATION | 2026-05-21 | — | REMEDIATION BURST | Post-convergence consistency-remediation burst — 9 corrective edits across 7 spec files. F-1 MAJOR (PRD §2.11 RTM: 5 BC-2.11.020–024 rows added to table + §7 RTM); F-2 MINOR (PRD §2.12 header CAP-01→CAP-12); F-3 MINOR (VP-INDEX 5 title mismatches aligned); F-4 MINOR / pass-32 N-1 (domain-spec §8 ADR →0004); F-5 NITPICK confirmed not-a-defect; O-09 (module-decomposition C-8 BTreeMap<u64,Vec<u8>>); pass-33 N-1 (BC-2.12.016 range 304-311); pass-33 N-2 (BC-2.11.021 range 40-45); 5 prd-supplement input-hashes bumped (corrective cluster). Consistency re-audit: CONSISTENT. Input-drift re-scan: MATCH=5/STALE=0. All 4 pre-approval polish items RESOLVED. |

Full per-pass details: `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`

### Phase 1 Gate Summary (ALL PASSED)

1. ~~**Adversarial spec-convergence gate**~~ — **SATISFIED** (3/3; passes 31/32/33; 33 total; ZERO blocking defects).
2. ~~**Consistency audit**~~ — **CONSISTENT** (2026-05-21; 5 findings F-1–F-5; F-1–F-4 remediated; F-5 not-a-defect).
3. ~~**Input-hash drift check**~~ — **CLEAN** (2026-05-21; 5 STALE hashes bumped; re-scan 5/5 MATCH).
4. ~~**Human approval**~~ — **GRANTED** 2026-05-21.
5. ~~**P8-DEFER VP back-reference back-fill**~~ — **DONE** 2026-05-21 (human-directed; 217 BCs updated; 69→formal VP IDs, 148→`—`; versions 1.1→1.2).

**Phase 1 CLOSED.**

## Phase 2 — Story Decomposition (IN PROGRESS)

### Steps

| Step | Status | Notes |
|------|--------|-------|
| A. `define-epics` | **COMPLETE** 2026-05-21 | 10 epics; 217/217 BCs assigned; 48 stories estimated — `stories/epics.md` |
| B. `create-stories` | **COMPLETE** 2026-05-21 | 48 STORY-NNN.md files across 10 epics; all 217 BCs traced to ≥1 story; STORY-INDEX.md draft committed — `stories/STORY-*.md` |
| C. `dependency-graph` | **COMPLETE** 2026-05-21 | 48 stories, 78 dependency edges, 27 waves, acyclic, 282 story points — `stories/dependency-graph.md` |
| D. `wave-schedule` | **COMPLETE** 2026-05-21 | 27 waves; all 48 stories wave-assigned; `wave-schedule.md`, `STORY-INDEX.md` rebuilt, `sprint-state.yaml` initialized (48 entries, current_wave 1) — `cycles/v0.1.0-greenfield-spec/wave-schedule.md`, `stories/STORY-INDEX.md`, `stories/sprint-state.yaml`, `stories/STORY-*.md` |
| E. `holdout-scenarios` | **COMPLETE** 2026-05-21 | 100 holdout scenarios HS-001–HS-100; 99 must-pass / 1 should-pass; 36 behavioral-subtleties, 19 edge-case-combinations, 18 integration-boundaries, 17 security-probes, 10 real-world-corpus; all 27 waves covered — `holdout-scenarios/` |
| F. `decomposition-gate` | **COMPLETE** 2026-05-21 | Consistency audit found 3 blocking findings (B-1 BC matrix divergence 31/48 stories, B-2 edge-count off-by-one 64→64 fixed, B-3 stale cycle fields 37/48 stories); all 3 remediated; 2 NUL-byte sanitizations (STORY-070/076, same class as P5-PG); re-audit CONSISTENT 100/100. Gate PASSED. |
| G. `adversarial-story-gate` | **IN PROGRESS** — Pass 1 NOT_CONVERGED (1C/3H/3M/2L/2N); all blocking findings remediated; Pass 2 next | Adversarial convergence review of story decomposition + holdout scenarios; 3 consecutive clean passes required; convergence counter 0/3; 3 process-gap NITPICKs (N-1/N-2/N-3) deferred for cycle-close codification |
| H. `human-approval` | NOT STARTED | Human sign-off before Phase 3 |

## Governance Policy

**DF-VALIDATION-001** (commit 9b6efd3, `.factory/policies.yaml`): every
deferred/open finding must be research-agent validated before filing as a
GitHub issue. Pointer in `CLAUDE.md` on `develop` via PR #99 (0082a0c).

## Tech Debt (Open)

| ID | Description | Priority | Source |
|----|-------------|----------|--------|
| O-07 | `rayon` declared in Cargo.toml but unused in `src/` — dead dependency | P2 | adversarial pass 1 (LOW finding) |
| O-08 | `src/analyzer/dns.rs` module doc-comment is stale — references removed behavior | P3 | adversarial pass 29 (observation O-1); recorded in domain-debt.md |
| O-09 | `architecture/module-decomposition.md` C-8 buffer described as `BTreeMap<u64,Segment>` (informal shorthand); actual `flow.rs:89` type is `BTreeMap<u64, Vec<u8>>` (no `Segment` struct). Non-misleading shorthand; not a spec defect. | P3 | **RESOLVED 2026-05-21** — consistency-remediation burst aligned to `BTreeMap<u64, Vec<u8>>` |
| N-1 | `specs/domain/domain-spec.md`:168 §8 "Cross-Reference to Corpus IDs" lists "ADR 0001/0002/0003"; intra-file consistency suggests "0001/0002/0003/0004". Defensibly correct-by-construction (§8 lists ingestion-corpus IDs; ADR 0004 post-dates ingestion, 2026-05-14). One-token fix. | P3 | **RESOLVED 2026-05-21** — F-4 in consistency audit; §8 updated to "0001/0002/0003/0004" |
| N-2 | `specs/behavioral-contracts/ss-12/BC-2.12.016.md` "Evidence Types Used" cites `resolve_format` doc comment range as main.rs:304-310; actual span is 304-311 (one-line under-reach; lands on real content). Non-blocking. | P3 | **RESOLVED 2026-05-21** — pass-33 N-1; range corrected to 304-311 |
| N-3 | `specs/behavioral-contracts/ss-11/BC-2.11.021.md` cites `neutralize_csv_injection` function as csv.rs:40-44; closing brace is at line 45. One-line under-reach; lands on real content. Non-blocking. | P3 | **RESOLVED 2026-05-21** — pass-33 N-2; range corrected to 40-45 |

Full register: `.factory/tech-debt-register.md` (when populated).

## Deferred Findings (non-blocking, pass 2)

| ID | Finding | Disposition | Follow-up |
|----|---------|-------------|-----------|
| L-2 | `src/analyzer/dns.rs` module doc-comment is stale — references removed behavior | Source defect, not a spec defect. Spec is correct. | File as code follow-up issue on `develop` after Phase 1 gate. |
| L-3 | No machine validator for BC-H1 <-> BC-INDEX title sync (process gap) | Tooling gap, not a spec gap. Spec BCs and INDEX are now in sync. | Codify as CI lint rule in a future sprint. |
| P3-PG | BC body postcondition/invariant edits must trigger a propagation sweep across BC-INDEX, PRD, capability/entity docs, VP files, and architecture docs (process gap, pass 3) | Discipline gap — not a spec defect. All 8+ affected files corrected in pass-3 remediation. | Codify as checklist step or CI lint rule at cycle close. |
| P4-PG1 | Reconciliation passes must cover capabilities/ and entities/ shards, not just invariants/architecture (process gap, pass 4) | Scope gap — fresh-context audit found 6 cap shards + ent-04 unreconciled post PR #69–#98. All corrected in pass-4 remediation. | Codify explicit cap+entity reconciliation pass in adversarial review checklist at cycle close. |
| P4-PG2 | No component-ID consistency validator between domain-spec/capabilities and architecture/module-decomposition (process gap, pass 4) | Tooling gap — component IDs drift silently. No machine check exists. | Codify as CI lint rule or reviewer checklist item at cycle close. |
| P4-PG3 | New reporter (csv.rs, PR #84) shipped without a BC — CsvReporter coverage gap not detected until pass-4 fresh-context audit (process gap, pass 4) | Coverage gap — 5 CsvReporter BCs (BC-2.11.020–024) added in pass-4 remediation. | Codify: every new src/ file in reporter/ or analyzer/ must trigger a BC coverage check at cycle close. |
| P5-PG | BC-file on-disk verification used an existence check only; a NUL-byte-corrupted file (BC-2.07.020.md) was not detected until pass-5 adversarial audit (process gap, pass 5) | Tooling gap — no UTF-8 + control-byte validator on the spec package. Pass-5 remediation removed the NUL byte. | Codify as cycle-close follow-up: add spec-package validator asserting every BC/spec file is valid UTF-8 with no control bytes other than CR/LF/TAB. |
| P8-DEFER | All 217 BC files carry `VP-TBD` placeholders in their Verification Properties field. The forward VP->BC mapping exists and is authoritative in VP-INDEX.md; the BC->VP back-reference back-fill is deferred as a Phase-1-exit polish item. | **RESOLVED 2026-05-21** — human directed back-fill before Phase 1 sign-off. All 217 BC files updated: 69 BCs now cite formal VP IDs; 148 show `—`. BC versions bumped 1.1→1.2 with `modified:` entry. Zero `VP-TBD` remain. | CLOSED. |
| P10-PG | Architecture-doc dependency tables were not diffed against Cargo.toml — authored from memory, causing stale versions, phantom crates, and missing crates (pass-10 H-1, M-3). | Tooling gap — no mechanical validator exists to assert dependency-graph.md matches Cargo.toml. Pass-10 remediation corrected the table manually. | Codify as cycle-close follow-up: add a `validate-deps-against-cargo` check that parses `[dependencies]` + `[dev-dependencies]` from Cargo.toml and asserts each crate appears in dependency-graph.md with the correct version. |
| **P-CITE-PG** **(MANDATORY — 6 recurrences: passes 4, 6, 8, 9, 10, 12)** | No automated validator resolves `file.rs:NNN` anchors in spec artifacts. Stale line-citations have recurred across 6 passes, driving HIGH and MEDIUM findings repeatedly. | RECURRING PROCESS GAP — per Cycle-Closing Checklist, 6 occurrences = mandatory codification follow-up required before cycle can be declared closed. A follow-up story or a justified deferral must be recorded. | **Required action:** Create a spec-CI citation-checker that resolves every `file.rs:NNN` anchor in spec artifact files, flags when the cited line is a comment/blank, and flags when an asserted symbol name is absent from the surrounding 5-line context. File as a follow-up story or record an explicit justified deferral at cycle close. **DONE — research-agent validated under DF-VALIDATION-001 (verdict VALIDATED-WITH-CHANGES; report at `.factory/research/citation-checker-validation.md`); follow-up filed as engine issue drbothen/vsdd-factory#151 (reframed: drift-resistant citation convention + checker). Mandatory-codification requirement satisfied.** |

## Open Issues (from Phase 0 / deferred findings)

| Issue | Summary |
|-------|---------|
| #100 | `Finding.timestamp` always None; thread pcap timestamps |
| #101 | Empirically characterize anomaly-threshold FP rates |
| #102 | Cap weak-cipher ClientHello evidence Vec, CWE-405 |
| #103 | Bidirectional size-symmetry discriminator for small-segment detector |
| #104 | Surface control bytes in non-ASCII SNI summary, BC-TLS-037 |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from
  `develop`. `.factory/logs/` is gitignored.
- SS-03 gap in BC numbering is intentional (subsystem not applicable).
- DTU assessment confirmed: no external service clones required.
- Phase 0 canonical ground truth: `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`.
- **2026-05-21 — P8-DEFER VP back-reference back-fill:** all 217 BC files updated; 69 BCs
  now carry formal VP IDs (VP-001:2, VP-002:6, VP-003:2, VP-004:6, VP-005:7, VP-006:3,
  VP-007:4, VP-008:3, VP-009:5, VP-010:2, VP-011:3, VP-012:6, VP-013:3, VP-014:2, VP-015:1,
  VP-016:5, VP-017:2, VP-018:2, VP-019:4, VP-020:1); 148 BCs show `—` (covered by story-level
  tests only); all BC versions bumped 1.1→1.2. Zero `VP-TBD` remain. **Phase 1 CLOSED.**
