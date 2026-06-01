---
pipeline: PHASE_5_ADVERSARIAL_REFINEMENT
phase: phase-5-adversarial-refinement
product: wirerust
mode: brownfield
timestamp: 2026-06-02T00:15:00Z
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
develop_head: 68137b4b
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 48
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 0/3  # Pass 10 NOT_CONVERGED (2M/1L); clean-streak RESET; FIX-P5-004 MERGED PR#175→68137b4b; Pass 11 IN PROGRESS
adv_impl_remediated: "P04-MED(PR#173→472b45e9);P06-HIGH(PR#174→cfe0112a);P06-MED(PR#174→cfe0112a);P07-MED(288cba3);P07-LOW(288cba3);P08-HIGH(e817d3c);P10-MED-001(422e4ee);P10-MED-002+LOW-001(FIX-P5-004/PR#175→68137b4b)"
adversary_gate: NOT_YET_SATISFIED
convergence_trajectory: "P5:P1-MED(2b33284)|P2-MED(aa6d73b)|P3-HIGH+LOW(c7a0012)|P4-MED(PR#173→472b45e9)|P5-ZERO(voided)|P6-HIGH+MED(PR#174→cfe0112a)|P7-MED+LOW(288cba3+d26eef0)|P8-HIGH-REMEDIATED(e817d3c)|P9-CLEAN(1/3,voided)|P10-MED+MED+LOW-REMEDIATED(422e4ee+PR#175→68137b4b)|P11-IN-PROGRESS. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
consistency_audit: CONSISTENT
input_drift_check: "CLEAN (re-baselined post ADV-IMPL-P08 anchor sweep: MATCH=48/STALE=0; 11 stories rewritten; commit 0f22508)"
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
wave_history_archived: "cycles/phase-3-tdd/wave-history.md (all waves 1-27 detail + final-steps; compacted 2026-06-01)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE 5 — Adversarial Refinement IN PROGRESS. develop 68137b4b (FIX-P5-004 merged PR #175).
Pass 10 NOT_CONVERGED: 0C/0H/2M/1L — all REMEDIATED+MERGED. ADV-IMPL-P10-MED-001 (BC-2.04.013 fn-name anchor) REMEDIATED commit 422e4ee. ADV-IMPL-P10-MED-002+LOW-001 (stale HS-043 test docstrings + misleading test name) REMEDIATED+MERGED FIX-P5-004 PR #175 → 68137b4b. HS-043 BC/doc-coherence finding class CLOSED. CLEAN-PASS COUNTER: 0/3 (streak restart after Pass-10 remediation). Pass 11 IN PROGRESS. Need 3 consecutive clean.

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
| Phase 5 — Adversarial Refinement | **IN PROGRESS** STARTED 2026-06-01 | Passes 1–10 complete; all P10 findings REMEDIATED+MERGED. FIX-P5-004 MERGED (PR #175 → 68137b4b); HS-043 BC/doc-coherence closed. CLEAN-PASS COUNTER 0/3 (reset). Pass 11 IN PROGRESS. P1→P10: MED→MED→HIGH+LOW→MED→ZERO→HIGH+MED→MED+LOW→HIGH→ZERO(voided)→MED+MED+LOW. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 3 — Wave Summary (COMPLETE)

Waves 1–27 ALL CLOSED/CONVERGED — detail: `cycles/phase-3-tdd/wave-history.md`. Spec package (Phase 1): 20 L2 shards, 1 PRD, 217 BCs, 20 VPs, 4 supplements, 9 arch files — detail: `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.

## Session Resume Checkpoint (2026-06-02 — PHASE 5, Pass 11 IN PROGRESS)

**POSITION:** Phase 5 IN PROGRESS. develop HEAD 68137b4b (FIX-P5-004 squash-merged PR #175). Pass 10 all findings REMEDIATED+MERGED: ADV-IMPL-P10-MED-001 (BC-2.04.013 v1.7 re-anchor, commit 422e4ee, MATCH=48/STALE=0); ADV-IMPL-P10-MED-002+LOW-001 (stale HS-043 test docstrings + misleading test name, FIX-P5-004 PR #175 → 68137b4b). HS-043 BC/doc-coherence finding class CLOSED. CLEAN-PASS COUNTER: 0/3 (reset after Pass-10 remediation).

**EXACT NEXT ACTION:** Whole-impl adversarial Pass 11 now running (dispatched fresh context). Need 3 consecutive CLEAN passes (0C/0H/0M) for CONVERGENCE_SATISFIED (3/3).

**MODEL-FAMILY CAVEAT (carry forward):** True non-Claude (GPT) evaluator unavailable. Use opus-tier fresh-context + strict info-asymmetry as substitute. Document at each gate.

**OPEN/ACCEPTED ITEMS a fresh session must know:**
- All P1–P10 findings REMEDIATED+MERGED. develop HEAD 68137b4b is clean.
- ADV-IMPL-P01-LOW-001: ACCEPTED/optional (findings.rs stale doc-comment, same class as O-08).
- ADV-HS043-P02-MED-001: ACCEPTED offline scope — re-open when live-capture added (see Drift Items).
- F-W25-S088-P6-001 LOW: test-strength only; target next main.rs touch.
- PROCESS-GAP-P5-001: OPEN — HIGH-PRIORITY. Durable-fix disposition REQUIRED at Phase-5 cycle close per S-7.02.

**PROCESS NOTE (W24.L3):** Verify merges via `gh pr view <N>` + `git rev-parse origin/develop` before declaring landed.

Prior checkpoint (Pass 10 REMEDIATED, Pass 11 NEXT) archived: cycles/v0.1.0-greenfield-spec/session-checkpoints.md.

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
| ADV-HS043-P02-LOW-001 | [Phase-5, HS043-pass-2, LOW] BC-2.04.013 PC0 literally names `expire_flows` but the impl wires `expire_idle_by_timeout`. Originally accepted as non-blocking. | spec-naming / docs | CLOSED | SUPERSEDED-BY-ADV-IMPL-P10-MED-001 — fresh-context Pass 10 upgraded to MED and properly fixed in BC-2.04.013 v1.7 (commit 422e4ee). No longer merely accepted. |
| ADV-IMPL-P10-MED-001 | [Phase-5, whole-impl Pass 10, MED] BC-2.04.013 PC0/anchors named `expire_flows` as production-wired enforcer; actual wired enforcer is `expire_idle_by_timeout` (called per-packet). `expire_flows` is public/offline API (called only by `finalize()`). Supersedes ADV-HS043-P02-LOW-001. | spec-naming / docs (BC-2.04.013) | CLOSED | REMEDIATED — BC-2.04.013 v1.7, STORY-019 v1.7 propagation, commit 422e4ee; input-hash 155cc08. MATCH=48/STALE=0. |
| ADV-IMPL-P10-MED-002 | [Phase-5, whole-impl Pass 10, MED] Stale HS-043 test docstrings — test functions in HS-043 region retained docstrings describing old `expire_flows` API rather than `expire_idle_by_timeout` production path. | test-coherence (HS-043 test files) | CLOSED | REMEDIATED + MERGED — FIX-P5-004 PR #175 squash-merged → develop 68137b4b 2026-06-02. HS-043 BC/doc-coherence class CLOSED. |
| ADV-IMPL-P10-LOW-001 | [Phase-5, whole-impl Pass 10, LOW] Misleading test name in HS-043 region — test name did not reflect the wired production function. | test-naming | CLOSED | REMEDIATED + MERGED — FIX-P5-004 PR #175 squash-merged → develop 68137b4b 2026-06-02. |
| ADV-IMPL-P04-MED-001 | [Phase-5, whole-impl Pass 4, MED] BC-2.12.005 zero-rejection contract gap: spec required depth/memcap >=1 but lacked canonical PCs, error codes E-CFG-007/E-CFG-008, and test-citation anchors. Code did not enforce the contract. FIX MERGED — spec reconciled: BC-2.12.005 v1.3 (depth/memcap >=1 PC5/PC6, EC-006/EC-007, canonical vectors), NFR-REL-004 impl note, E-RAS-004 + E-CFG-007/008 taxonomy, STORY-087 v1.3 (EC-001 revised, EC-006/AC-013/AC-014 added). Code fix PR #173 squash-merged → develop 472b45e9 2026-06-01. Demo recorded (.factory/demo-evidence/FIX-P5-002/, exit 2/2/0). Pass 5 adversary verified correct. | spec-contract gap + missing code enforcement | CLOSED | REMEDIATED + MERGED — PR #173 squash-merged → develop 472b45e9 2026-06-01; spec reconciled BC-2.12.005 v1.3/NFR-REL-004/E-RAS-004/E-CFG-007-008/STORY-087 v1.3; Pass 5 adversary confirmed correct |
| ADV-IMPL-P01-MED-001 | [Phase-5, whole-impl Pass 1, MED] SS-04 behavioral-contract source-line anchors stale after HS-043 merges (PR #171 + #172) shifted code in src/reassembly/mod.rs. No semantic/PC/inv changes; line-number citations only. | spec-anchor drift (DF-SIBLING-SWEEP-001) | SS-04 BC re-anchor sweep | REMEDIATED — 32 BCs re-anchored, commit 2b33284, pushed 2026-06-01 |
| ADV-IMPL-P02-MED-001 | [Phase-5, whole-impl Pass 2, MED] Residual SS-04 anchor drift missed by v1.6 sweep: BC-2.04.052 (2 anchors: traceability row + Architecture Anchors, mod.rs:306-312 → mod.rs:335-341) and BC-2.04.032 (1 prose anchor, mod.rs:306-319 → mod.rs:335-349). No semantic/PC/inv changes; line-number citations only. Recurring root cause: HS-043 mod.rs insertion (PROCESS-GAP-P5-001). | spec-anchor drift (DF-SIBLING-SWEEP-001) | BC-2.04.052/.032 re-anchor | REMEDIATED — 2 BCs re-anchored, commit aa6d73b, pushed 2026-06-01 |
| ADV-IMPL-P03-HIGH-001 | [Phase-5, whole-impl Pass 3, HIGH] Consuming-artifact anchor drift: vp-003-max-findings-cap.md, domain/invariants/inv-01-core-invariants.md, prd-supplements/error-taxonomy.md, prd-supplements/nfr-catalog.md all cite src/reassembly/mod.rs at stale line numbers from pre-HS-043 positions. Same HS-043 root cause as Pass 1/2; deeper artifact tier (VPs, domain invariants, supplements, entities) missed by prior SS-04-only sweeps. 28 total citation corrections across 8 files. No semantic/PC/inv changes. | spec-anchor drift (DF-SIBLING-SWEEP-001) | Exhaustive 28-citation/8-file re-anchor sweep | REMEDIATED — commit c7a0012, pushed 2026-06-01; PO verified every mod.rs citation in full spec tree vs HEAD e0451ef (anchor class EXHAUSTIVELY CLOSED) |
| ADV-IMPL-P03-LOW-001 | [Phase-5, whole-impl Pass 3, LOW] BC-2.12.019 prose used "TLS/SSL" where spec language is "TLS"; corrected in the Pass-3 sweep. No semantic change. | spec-prose consistency | BC-2.12.019 prose fix | REMEDIATED — commit c7a0012, pushed 2026-06-01 |
| ADV-IMPL-P01-LOW-001 | [Phase-5, whole-impl Pass 1, LOW] findings.rs:104-105 ThreatCategory::Persistence doc-comment stale — variant never constructed in current codebase. Same class as O-08 (dns.rs stale module doc). No behavioral impact. | docs / tech-debt (same class as O-08) | Optional doc-comment touch | ACCEPTED/optional — no action before Phase-6 |
| ADV-IMPL-P01-LOW-002 | [Phase-5, whole-impl Pass 1, LOW] Additional anchor citation gap folded into the SS-04 sweep scope. | spec-anchor drift | SS-04 sweep | ACCEPTED — folded into ADV-IMPL-P01-MED-001 sweep; no residual |
| ADV-IMPL-P07-MED-001 | [Phase-5, whole-impl Pass 7, MED] VP-017 documented wrong JSON key ordering mechanism — claimed indexmap/insertion-order; actual is BTreeMap/alphabetical (preserve_order OFF; no indexmap dep). Determinism property HOLDS; spec's mechanism explanation was wrong. Doc-only fix. | spec-mechanism doc error (VP-017) | CLOSED | REMEDIATED doc-only — VP-017 v1.2, commit 288cba3, 2026-06-01 |
| ADV-IMPL-P07-LOW-001 | [Phase-5, whole-impl Pass 7, LOW] BC-2.11.001 and STORY-076 cited json.rs:59 for infallible unwrap(); actual line is json.rs:60 (line 59 is `});` closing the json! macro). Doc-only fix. | spec-anchor off-by-one (BC-2.11.001, STORY-076) | CLOSED | REMEDIATED doc-only — BC-2.11.001 v1.4 / STORY-076 v1.2, commit 288cba3 + d26eef0 (input-hash), 2026-06-01 |
| ADV-IMPL-P08-HIGH-001 | [Phase-5, whole-impl Pass 8, HIGH] Stale TEST-FILE line anchors — 4th anchor-drift dimension discovered. Prior sweeps closed source/fuzz/consuming/story-body dims; test-file dim missed. Exhaustive sweep of ALL 1305 corpus citations vs HEAD cfe0112a found 83 stale `.rs:NNN` citations across 44 spec files (BC-2.07, BC-2.09, BC-2.11, BC-2.12, vp-006, cap/ent/nfr/error-taxonomy/phase-4 scope-decision). Line-number citations only — no semantics changed. | spec-anchor drift (DF-SIBLING-SWEEP-001; test-file dim) | CLOSED | REMEDIATED — exhaustive 83-citation/44-file sweep, commit e817d3c (2026-06-02); input-hash re-baselined 11 stories, commit 0f22508. Line-anchor class CLOSED ALL dimensions. |
| ADV-IMPL-P06-HIGH-001 | [Phase-5, whole-impl Pass 6, HIGH] Non-deterministic JSON output ordering of top_snis/top_hosts tie entries — HashMap iteration not stabilized. REAL defect. FIX MERGED PR #174 → cfe0112a; determinism class CLOSED; Pass 7 verified correct. | implementation / determinism | CLOSED | REMEDIATED + MERGED — PR #174 → develop cfe0112a, 2026-06-01; Pass 7 clean |
| ADV-IMPL-P06-MED-001 | [Phase-5, whole-impl Pass 6, MED] Non-deterministic terminal PROTOCOLS/SERVICES ordering — BTreeMap keys not propagated sorted. REAL defect. FIX MERGED PR #174 → cfe0112a; Pass 7 verified correct. | implementation / determinism | CLOSED | REMEDIATED + MERGED — PR #174 → develop cfe0112a, 2026-06-01; Pass 7 clean |

## Cycle-Close Follow-Up Items (OPEN)

Closed items archived in `cycles/drift-remediation-2026-05-29/closed-items.md`. Phase-gated/externally-blocked items in `cycles/phase-3-tdd/deferred-items-archive.md`. Historical PG items: `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.

| ID | Description | Status |
|----|-------------|--------|
| PROCESS-GAP-P5-001 | [process-gap, 2026-06-01; ESCALATED 2026-06-02; REINFORCED Pass 10] HS-043 cross-cutting reassembly merges dispatched without a complete sibling sweep — DF-SIBLING-SWEEP-001 enforcement gap. SYSTEMIC: over 10 adversarial passes anchor/coherence gaps recurred: Pass 1 (BC files, 32 BCs), Pass 2 (BC secondary anchors, 2 BCs), Pass 3 (consuming VP/invariant/supplement/entity, 28 citations / 8 files), Pass 8 (test-file dim — 83 citations / 44 files), Pass 10 (BC fn-name + test docstring coherence: expire_flows vs expire_idle_by_timeout). ROOT CAUSE: sweeps are reactive, not preventive. When a fix renames/introduces a function, sweeps must also cover BC PC/anchors + test docstrings naming the old/related function. ESCALATED to HIGH-PRIORITY. Cycle-close disposition MUST propose a durable fix. | OPEN — HIGH-PRIORITY; durable-fix disposition REQUIRED at Phase-5 cycle close per S-7.02 |

## Governance Policy

Full policy text: `.factory/policies.yaml` (canonical). Prose detail archived: `cycles/phase-3-tdd/governance-policy-detail.md`.
4 policies codified 2026-05-30 from PG-HASH-001 + PG-W18-001/002/003 (detail: cycles/phase-3-tdd/lessons.md).

| Policy | Severity |
|--------|----------|
| DF-VALIDATION-001 | required-before-issue |
| DF-SIBLING-SWEEP-001 (v1→v5) | CRITICAL |
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
- Phase 1/2 adversary pass detail: `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md` (P1) and `story-adversary-pass-*.md` (P2). Phase 4 holdout eval: `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`. Wave 1-27 history: `cycles/phase-3-tdd/wave-history.md`.
