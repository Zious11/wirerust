---
pipeline: PHASE_5_ADVERSARIAL_REFINEMENT
phase: phase-5-adversarial-refinement
product: wirerust
mode: brownfield
timestamp: 2026-06-01T16:45:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_started: "2026-05-21"
phase_3_completed: "2026-05-31"
phase_3_to_4_gate: PASSED
phase_4_started: "2026-06-01"
phase_4_completed: "2026-06-01"
phase_4_to_5_gate: "PASSED (human-approved 2026-06-01, conditioned on HS-043 regression tests — merged PR #172)"
phase_5_started: "2026-06-01"
develop_head: e0451ef
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 48
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 0/3
adversary_gate: NOT_YET_SATISFIED
convergence_trajectory: "Full Phase-1→W27 trajectory archived in cycles/phase-3-tdd/convergence-trajectory.md. Latest: ...W26-CLOSED(PR#169→450d33e;BC-2.12.014..017;6ps-3clean;17-mutation-matrix)|W27-S090:3ps-3clean(R1/R2/R3;4→2→0;BC-2.12.018..021;18t;2-remediation-rounds-traceability;corpus-uniqueness-sweep)|W27-CONVERGED-CLOSED(single-story;per-story==wave-level;BC-5.39.001;PR#170→6158e6e)|PHASE-3-COMPLETE(48/48;27/27)"
consistency_audit: CONSISTENT
input_drift_check: CLEAN (Phase-4-entry gate confirmed: bin/compute-input-hash --scan = MATCH=48/STALE=0; zero stale hashes across full corpus)
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
wave_history_archived: "cycles/phase-3-tdd/wave-history.md (all waves 1-27 detail + final-steps; compacted 2026-06-01)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE 5 — Adversarial Refinement IN PROGRESS. Phase 4 PASSED (gate approved; HS-043 fix + regression guards merged). develop e0451ef.
Phase 4 result: 80-scenario rotation, mean 0.949, 0 must-pass <0.6; HS-043 real defect found+fixed (PR #171); HS-006/016 non-defects; model-family caveat documented.

**Mode:** brownfield (in-repo: target == reference).

**Test suite:** passing on develop (all 48 stories / 27 waves delivered; ~1086 tests green). `cargo fmt --check`,
`cargo clippy`, `cargo test --all-targets` all green. CI: 8 checks (semantic-pr, test, clippy, fmt, fuzz-build, audit, deny, trust-boundary; `fuzz-build` pinned `nightly-2026-05-21` + `cargo-fuzz 0.13.1` + `timeout-minutes: 25` after PR #111 hotfix; `trust-boundary` added PR #148;
the nightly pin is a deliberate periodic-maintenance item — do NOT enable automated
dependency bumping for it).

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements; 33 adversary passes; trajectory: `17→…→0→0→0` (detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md) |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 48 stories / 10 epics / 27 waves / 100 holdout scenarios / 282 points; story-adversary 3/3 (10 passes) SATISFIED; input-hash drift CLEAN (153/153) |
| Phase 3 — TDD Implementation | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves, all CLOSED/CONVERGED; E-1..E-10 ALL COMPLETE; develop HEAD 6158e6e (PR#170); BC-5.39.001 ACHIEVED across all waves; trajectory detail: cycles/phase-3-tdd/convergence-trajectory.md |
| Phase 4 — Holdout Evaluation | **PASSED** 2026-06-01 | 80-scenario rotation, mean 0.949, 0 must-pass <0.6; HS-043 real defect found+fixed (PR #171); HS-006/016 non-defects; model-family caveat documented; detail: cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md; Phase 4→5 gate PASSED 2026-06-01 (PR #172 regression tests merged) |
| Phase 5 — Adversarial Refinement | **IN PROGRESS** STARTED 2026-06-01 | HS043-pass-2 COMPLETE; Pass 1 NOT_CONVERGED MED REMEDIATED (2b33284); Pass 2 NOT_CONVERGED MED REMEDIATED (aa6d73b); whole-impl Pass 3 COMPLETE — NOT_CONVERGED: 0 CRIT/1 HIGH (ADV-IMPL-P03-HIGH-001 consuming-artifact anchor drift) + 1 LOW (ADV-IMPL-P03-LOW-001 BC-2.12.019 prose) REMEDIATED via exhaustive 28-citation/9-file sweep (c7a0012); ss-08/09/11/12 fidelity, integration seams, test depth verified clean. CLEAN-PASS COUNTER=0 (all 3 passes ≥MEDIUM, same HS-043 anchor-drift class; class now EXHAUSTIVELY CLOSED per PO). NEXT: whole-impl adversarial Pass 4 (fresh context). |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 3 — Wave Summary (COMPLETE)

Waves 1–27 ALL CLOSED/CONVERGED — per-wave detail: `cycles/phase-3-tdd/wave-history.md`

## Spec Package Summary (Phase 1 — PASSED)

| Artifact | Location | Count |
|----------|----------|-------|
| L2 Domain Specification | `.factory/specs/domain/` | 20 shards |
| L3 PRD | `.factory/specs/prd.md` | 1 file |
| Behavioral Contracts | `.factory/specs/behavioral-contracts/ss-01..ss-13/` | 217 BCs across 12 subsystems |
| BC Index | `.factory/specs/behavioral-contracts/BC-INDEX.md` | 1 file |
| Architecture Package | `.factory/specs/architecture/` | 9 files + ARCH-INDEX.md |
| Module Criticality | `.factory/specs/module-criticality.md` | 1 file |
| DTU Assessment | `.factory/specs/dtu-assessment.md` | DTU_REQUIRED: false |
| Verification Properties | `.factory/specs/verification-properties/vp-001..vp-020` | 20 VPs + VP-INDEX.md |
| PRD Supplements | `.factory/specs/prd-supplements/` | 4 files |

Full Phase 1 convergence detail: `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`

## Session Resume Checkpoint (2026-06-01 — PHASE 5, whole-impl Pass 3 REMEDIATED)

**POSITION:** Phase 5 (Adversarial Refinement) IN PROGRESS. Whole-implementation adversarial Pass 3 COMPLETE — verdict NOT_CONVERGED. 0 CRIT / 1 HIGH (ADV-IMPL-P03-HIGH-001 consuming-artifact anchor drift in vp-003/inv-01/error-taxonomy/nfr-catalog) + 1 LOW (ADV-IMPL-P03-LOW-001 BC-2.12.019 prose "TLS/SSL"→"TLS"). REMEDIATED via exhaustive 28-citation/9-file mod.rs sweep committed c7a0012, pushed origin/factory-artifacts. CLEAN-PASS COUNTER = 0 (all 3 passes had ≥MEDIUM — all same HS-043 anchor-drift class; anchor class now EXHAUSTIVELY CLOSED per PO full-corpus verification). develop HEAD e0451ef (unchanged — no source code modified).

**EXACT NEXT ACTION:** Run whole-implementation adversarial Pass 4 (fresh context) via `/vsdd-factory:adversarial-review implementation`. A Pass-4 anchor residual would indicate the exhaustive PO verification is unreliable; otherwise expect convergence to begin. Minimum 3 consecutive CLEAN passes required for CONVERGENCE_REACHED.

**MODEL-FAMILY CAVEAT (carry forward):** True non-Claude (GPT) evaluator/adversary unavailable in this environment. Use opus-tier fresh-context + strict info-asymmetry as substitute. Document at each gate.

**WHOLE-IMPL PASS LOG:** Pass 1 NOT_CONVERGED MED (32 BCs) REMEDIATED 2b33284. Pass 2 NOT_CONVERGED MED (2 BCs) REMEDIATED aa6d73b. Pass 3 NOT_CONVERGED HIGH+LOW (8 files, 28 citations) REMEDIATED c7a0012. All 3 trace to PROCESS-GAP-P5-001/DF-SIBLING-SWEEP-001. Anchor class EXHAUSTIVELY CLOSED (PO-verified every mod.rs citation in full spec tree vs HEAD e0451ef). Detail: cycles/v0.1.0-greenfield-spec/burst-log.md Bursts P5-1/P5-2/P5-3.

**OPEN/ACCEPTED ITEMS a fresh session must know:**
- ADV-IMPL-P01-MED-001: REMEDIATED (2b33284). ADV-IMPL-P02-MED-001: REMEDIATED (aa6d73b). ADV-IMPL-P03-HIGH-001: REMEDIATED (c7a0012). ADV-IMPL-P03-LOW-001: REMEDIATED (c7a0012).
- ADV-IMPL-P01-LOW-001: ACCEPTED/optional — findings.rs:104-105 stale doc-comment (same class as O-08).
- ADV-IMPL-P01-LOW-002: ACCEPTED — folded into SS-04 sweep.
- ADV-HS043-P02-MED-001: ACCEPTED offline scope — re-open when live-capture added (see Drift Items).
- ADV-HS043-P02-LOW-001: ACCEPTED — BC-2.04.013 PC0 naming note (non-blocking).
- F-W25-S088-P6-001 LOW: warning-once inv-2 count assertion; test-strength only; target next main.rs touch.
- PROCESS-GAP-P5-001: OPEN (3 consecutive passes same class) — DF-SIBLING-SWEEP-001 must enumerate ALL consuming-artifact tiers. Disposition (story vs deferral) at Phase-5 cycle close per S-7.02.

**PROCESS NOTE (W24.L3):** pr-manager has repeatedly stopped before executing merges (~5 PRs this session). ALWAYS independently verify a merge landed via `gh pr view <N>` + `git rev-parse origin/develop` before declaring a PR merged.

**PHASE CONTEXT:**
- Phase 4 result: 80-scenario rotation, mean 0.949, 0 must-pass <0.6. HS-043 real defect found+fixed (PR #171 → c3cd4bd); HS-006/016 non-defects. Regression guards merged PR #172 → e0451ef.
- Prior checkpoint (Pass 2 remediated) archived: cycles/v0.1.0-greenfield-spec/session-checkpoints.md.

## Phase 3→4 Gate — PASSED 2026-06-01

(a) Input-drift CLEAN (MATCH=48/STALE=0). (b) Consistency audit READY (report: cycles/phase-3-tdd/phase-4-entry-consistency-audit.md; 217/217 BCs covered, 100/100 HS CLEAR, 20/20 VPs consistent). (c) Human approval GRANTED. D-001 RESOLVED (STORY-053 EC fixed f368f53). D-002 RESOLVED (this burst: 6 story statuses + wave rows 3-22 backfilled). Phase-4-ENTRY deferred item CLOSED.

## Phase 4→5 Gate — PASSED 2026-06-01

Phase 4 criteria met: mean 0.949 >= 0.85; 0 must-pass < 0.6; std-dev < 0.15; HS-043 real defect fixed (PR #171 → c3cd4bd); HS-006/016 non-defects confirmed. Model-family caveat documented (no true non-Claude GPT evaluator available — opus-tier with strict info-asymmetry). Human approval GRANTED 2026-06-01, conditioned on HS-043 regression tests — merged PR #172 → e0451ef. Full detail: cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md.

## Wave Retrospectives

Compacted summary table + full prose: `.factory/cycles/phase-3-tdd/lessons.md` (archived 2026-05-29 — content-routing rule S-7.02).

## Decisions Log

| ID | Decision | Date | Rationale |
|----|----------|------|-----------|
| D-001 | Brownfield mode (target == reference) | 2026-05-19 | No parallel reference repo; in-repo formalization only |
| D-002 | DTU not required | 2026-05-20 | No external service clones needed per dtu-assessment |
| D-003 | CI hotfix: cargo audit shell step | 2026-05-22 | rustsec/audit-check@v2.0.0 fails on push events; PR #111 |
| D-004 | Nightly pin nightly-2026-05-21 is periodic-maintenance | 2026-05-22 | Bumping requires verifying fuzz build; do NOT automate |
| D-005 | Demo recordings local-only (gitignored) | 2026-05-22 | factory-artifacts gitignores cycles/**/demos/; 49 prior files untracked |
| D-006 | [correction 2026-05-29/30] Wave-20/STORY-076 real merge SHA is e5cb2b1 (PR #157). Two earlier recorded SHAs were wrong and have been corrected: a8f3d21 (phantom, pre-merge write) and 4d9e1c7 (transient pre-resolution id). Root cause: post-merge state written before pr-manager's authoritative merge SHA was confirmed; rectified. | 2026-05-29 | Orchestrator supplied SHA before actual merge; real merge commit confirmed e5cb2b1 on origin/develop |
| D-007 | Deferred-item cleanup: DF-16.B closed (bulk 209-BC sweep commit b17c5f0; 0 remaining broken citations); OBS-7 closed (covered by STORY-076 BC-2.11.003 / test_BC_2_11_003_c0_esc_escaped_in_json; PR #157→e5cb2b1); 4 governance candidates codified to policies.yaml (DF-INPUT-HASH-CANONICAL-001, DF-ADVERSARY-CHECKOUT-GUARD-001, DF-TEST-CITATION-SWEEP-001, DF-TEST-NAMESPACE-001); 6 externally-blocked items archived to cycles/phase-3-tdd/deferred-items-archive.md (W9-D2/D3/D4 upstream-plugin, W9-D12 awaiting-PO, W1.3/W2.5 upstream, W7.1 public-api, Phase-4-ENTRY, F-S058-P13-O4). | 2026-05-30 | STATE.md deferred-item cleanup burst; no information lost |
| D-008 | [2026-05-30] STORY-079 input BC-2.11.020 corrected v1.2→v1.3 (CRLF→LF). STORY-079 input-hash NOT recomputed because canonical bin/compute-input-hash is missing from repo (DF-INPUT-HASH-CANONICAL-001 forbids hand-compute). Logged F-W21-S079-HASH + F-W21-TOOL-001; input-hash re-validated at Phase-4 gate after tool restore. Decision: do not block STORY-079 per-story convergence on a stale-hash finding that cannot be mechanically resolved and is gated for Phase-4 anyway (zero src/behavioral impact; test↔spec sync intact; AC test-name citations unchanged). | 2026-05-30 | STORY-079 Pass-1 adversarial review F-002; unblocking per-story convergence on non-mechanical, phase-gated gap |
| D-009 | [2026-06-01] ADV-HS043-P02-MED-001 accepted for current offline pcap scope — finalize() reclaims all flows; no unbounded growth risk. High-water-clock fix rejected: empirically breaks legitimate multi-epoch offline analysis (story-088 http-ooo tests fail). Throwaway fix branch + worktree discarded; develop unchanged (HEAD e0451ef). Gated on live-capture support; re-open then. ADV-HS043-P02-LOW-001 accepted non-blocking (BC naming note). Human-approved 2026-06-01. | 2026-06-01 | Phase-5 HS043-pass-2 disposition; human decision 2026-06-01 |

## Blocking Issues

None open.

## Drift Items

All items below require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Closed items archived in `.factory/cycles/drift-remediation-2026-05-29/closed-items.md`.
Externally-blocked / phase-gated items (W9-D2/D3/D4 upstream-plugin, W9-D12 awaiting-PO, W1.3/W2.5 upstream, W7.1 public-api, Phase-4-ENTRY, F-S058-P13-O4) archived to cycles/phase-3-tdd/deferred-items-archive.md — revisit at their named gate/phase.

| ID | Finding | Category | Target | Status |
|----|---------|----------|--------|--------|
| F-W25-S088-P6-001 | [test-strength, LOW] AC-004 warning uses .contains() so a doubled eprintln! (BC-2.12.009 inv-2 "warning printed once") would not be caught. Invariant HOLDS in source (single pre-loop emission, adversary-verified P6); AC-004 traces to PC-5/inv-1 not inv-2 — not a traceability defect. Optional one-line count-assertion hardening; target: STORY-090 touch (next main.rs-adjacent story) or accepted. Per DF-VALIDATION-001, no GitHub issue without research-agent validation. | test-strength | STORY-090 delivery (wave 27) or accept | OPEN |
| ADV-HS043-P02-MED-001 | [Phase-5, HS043-pass-2, MED] Idle-flow expiry sweep gate `timestamp > last_expiry_sweep_secs` is monotonic; on out-of-order / multi-epoch / clock-regressing captures the watermark stalls and idle sweeps stop firing for the rest of the run (flows_expired stuck at 0). BC-2.04.013. Current scope: offline pcap — finalize() reclaims all flows at end-of-capture; no unbounded growth. High-water-clock fix rejected (breaks multi-epoch offline analysis — story-088 http-ooo tests fail). Probe premise also flawed (20 new flows at t=10 are not idle). No GitHub issue per DF-VALIDATION-001. Full rationale: cycles/v0.1.0-greenfield-spec/burst-log.md Burst 3. | implementation / memory-bound (BC-2.04.013) | Live-capture support feature | ACCEPTED — GATED ON LIVE-CAPTURE SUPPORT. Re-open when live-capture added; correct fix is epoch-boundary flush or wall-clock sweep tick, NOT high-water-clock. |
| ADV-HS043-P02-LOW-001 | [Phase-5, HS043-pass-2, LOW] BC-2.04.013 PC0 literally names `expire_flows` but the impl wires `expire_idle_by_timeout`; the split is justified (keeps the Closed-clause off the hot path per BC-2.04.017) and documented in source. Optional one-line BC wording note; not blocking. | spec-naming / docs | Optional BC wording touch | ACCEPTED (non-blocking) |
| ADV-IMPL-P01-MED-001 | [Phase-5, whole-impl Pass 1, MED] SS-04 behavioral-contract source-line anchors stale after HS-043 merges (PR #171 + #172) shifted code in src/reassembly/mod.rs. No semantic/PC/inv changes; line-number citations only. | spec-anchor drift (DF-SIBLING-SWEEP-001) | SS-04 BC re-anchor sweep | REMEDIATED — 32 BCs re-anchored, commit 2b33284, pushed 2026-06-01 |
| ADV-IMPL-P02-MED-001 | [Phase-5, whole-impl Pass 2, MED] Residual SS-04 anchor drift missed by v1.6 sweep: BC-2.04.052 (2 anchors: traceability row + Architecture Anchors, mod.rs:306-312 → mod.rs:335-341) and BC-2.04.032 (1 prose anchor, mod.rs:306-319 → mod.rs:335-349). No semantic/PC/inv changes; line-number citations only. Recurring root cause: HS-043 mod.rs insertion (PROCESS-GAP-P5-001). | spec-anchor drift (DF-SIBLING-SWEEP-001) | BC-2.04.052/.032 re-anchor | REMEDIATED — 2 BCs re-anchored, commit aa6d73b, pushed 2026-06-01 |
| ADV-IMPL-P03-HIGH-001 | [Phase-5, whole-impl Pass 3, HIGH] Consuming-artifact anchor drift: vp-003-max-findings-cap.md, domain/invariants/inv-01-core-invariants.md, prd-supplements/error-taxonomy.md, prd-supplements/nfr-catalog.md all cite src/reassembly/mod.rs at stale line numbers from pre-HS-043 positions. Same HS-043 root cause as Pass 1/2; deeper artifact tier (VPs, domain invariants, supplements, entities) missed by prior SS-04-only sweeps. 28 total citation corrections across 8 files. No semantic/PC/inv changes. | spec-anchor drift (DF-SIBLING-SWEEP-001) | Exhaustive 28-citation/8-file re-anchor sweep | REMEDIATED — commit c7a0012, pushed 2026-06-01; PO verified every mod.rs citation in full spec tree vs HEAD e0451ef (anchor class EXHAUSTIVELY CLOSED) |
| ADV-IMPL-P03-LOW-001 | [Phase-5, whole-impl Pass 3, LOW] BC-2.12.019 prose used "TLS/SSL" where spec language is "TLS"; corrected in the Pass-3 sweep. No semantic change. | spec-prose consistency | BC-2.12.019 prose fix | REMEDIATED — commit c7a0012, pushed 2026-06-01 |
| ADV-IMPL-P01-LOW-001 | [Phase-5, whole-impl Pass 1, LOW] findings.rs:104-105 ThreatCategory::Persistence doc-comment stale — variant never constructed in current codebase. Same class as O-08 (dns.rs stale module doc). No behavioral impact. | docs / tech-debt (same class as O-08) | Optional doc-comment touch | ACCEPTED/optional — no action before Phase-6 |
| ADV-IMPL-P01-LOW-002 | [Phase-5, whole-impl Pass 1, LOW] Additional anchor citation gap folded into the SS-04 sweep scope. | spec-anchor drift | SS-04 sweep | ACCEPTED — folded into ADV-IMPL-P01-MED-001 sweep; no residual |

## Cycle-Close Follow-Up Items (OPEN)

Closed items archived in `cycles/drift-remediation-2026-05-29/closed-items.md`. Phase-gated/externally-blocked items in `cycles/phase-3-tdd/deferred-items-archive.md`. Historical PG items: `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.

| ID | Description | Status |
|----|-------------|--------|
| PROCESS-GAP-P5-001 | [process-gap, 2026-06-01] HS-043 cross-cutting reassembly merges dispatched without a complete sibling sweep — DF-SIBLING-SWEEP-001 enforcement gap. THREE consecutive passes hit the same anchor-drift class at progressively deeper tiers: Pass 1 (BC files, 32 BCs), Pass 2 (BC secondary anchors, 2 BCs), Pass 3 (consuming VP/invariant/supplement/entity artifacts, 28 citations / 8 files). Anchor class EXHAUSTIVELY CLOSED 2026-06-01 (PO-verified every mod.rs citation in full spec tree vs HEAD e0451ef). DF-SIBLING-SWEEP-001 checklist must be extended to enumerate ALL consuming-artifact tiers (VPs, domain invariants, entities, prd-supplements, scope-decision docs) as mandatory sweep targets whenever a merge shifts cited source lines. MUST NOT be silently closed before Phase-5 cycle close. Disposition (follow-up story vs justified deferral) per S-7.02 at cycle close. | OPEN — DF-SIBLING-SWEEP-001 tier-extension required; disposition at Phase-5 cycle close per S-7.02 |

## Governance Policy

Full policy text: `.factory/policies.yaml` (canonical). Prose detail archived: `cycles/phase-3-tdd/governance-policy-detail.md`.
4 policies codified 2026-05-30 from PG-HASH-001 + PG-W18-001/002/003 (detail: cycles/phase-3-tdd/lessons.md).

| Policy | Severity |
|--------|----------|
| DF-VALIDATION-001 | required-before-issue |
| DF-SIBLING-SWEEP-001 (v1→v4) | CRITICAL |
| DF-PR-MANAGER-COMPLETE-001 | HIGH |
| DF-ADVERSARY-METHODOLOGY-001 | HIGH |
| DF-AC-TEST-NAME-SYNC-001 (v2) | MEDIUM |
| DF-CONVERGENCE-BEFORE-MERGE-001 | CRITICAL |
| DF-DEVELOP-FRESHNESS-001 | HIGH |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | HIGH |
| DF-INPUT-HASH-CANONICAL-001 | HIGH |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | HIGH |
| DF-TEST-CITATION-SWEEP-001 | HIGH |
| DF-TEST-NAMESPACE-001 | MEDIUM |

## Tech Debt (Open)

| ID | Description | Priority | Source |
|----|-------------|----------|--------|
| O-07 | `rayon` declared in Cargo.toml but unused in `src/` — dead dependency | P2 | adversarial pass 1 (LOW finding) |
| O-08 | `src/analyzer/dns.rs` module doc-comment stale — references removed behavior | P3 | adversarial pass 29 (observation O-1); recorded in domain-debt.md |

Full register: `.factory/tech-debt-register.md`

## Open Issues (from Phase 0 / deferred findings)

| Issue | Summary |
|-------|---------|
| #100 | `Finding.timestamp` always None; thread pcap timestamps |
| #101 | Empirically characterize anomaly-threshold FP rates |
| #102 | Cap weak-cipher ClientHello evidence Vec, CWE-405 |
| #103 | Bidirectional size-symmetry discriminator for small-segment detector |
| #104 | Surface control bytes in non-ASCII SNI summary, BC-TLS-037 |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- SS-03 gap in BC numbering is intentional (subsystem not applicable).
- Phase 0 canonical ground truth: `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`.
- Wave-level convergence history: `.factory/cycles/phase-3-tdd/convergence-trajectory.md`.
- Phase 1/2 adversary pass detail: `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md` (P1) and `story-adversary-pass-*.md` (P2).
- Phase 4 holdout eval detail: `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`. Wave 1-27 history: `cycles/phase-3-tdd/wave-history.md`.
