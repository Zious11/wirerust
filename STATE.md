---
pipeline: FEATURE
phase: F7
phase_status: "FE-001 COMPLETE — F1–F7 all converged + human-approved (D-194). pcapng reader shipped to develop @ fcb8dce. Cycle feature-pcapng-reader CLOSED. RELEASED as v0.9.3 (2026-06-22, D-201). CORPUS-OBS-PCAPNG-IFFCSLEN-001 RESOLVED + decision-thread (b) CLOSED (D-202). Pipeline quiesced. D-203 SAFE-TO-CLEAR checkpoint written."
product: wirerust
mode: brownfield
timestamp: 2026-06-22T13:00:00Z

# Release chain
released_version: v0.9.3
released_at: "2026-06-22"
release_tag: v0.9.3
release_commit: 2dbf461
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.9.3
release_yml_run: "27984557297 SUCCESS — 4 binaries published"
prior_released_version: v0.9.2
prior_released_at: "2026-06-19"
prior_release_tag: v0.9.2
prior_release_tag_object: a298dbe
prior_release_commit: b73b242
v091_release_tag: v0.9.1
v091_release_commit: ad4eec8
v090_release_tag: v0.9.0
v090_release_commit: 986e148

# Ground-truth HEADs (verified at D-203 — 2026-06-22)
develop_head: dd3b069
main_head: 2dbf461
factory_artifacts_head: "run: git -C .factory log -1 --format='%h %s'"

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_completed: "2026-05-31"
phase_4_completed: "2026-06-01"
phase_5_completed: "2026-06-01"
phase_6_completed: "2026-06-02"
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 77
current_cycle: feature-pcapng-reader
current_wave: "F7 PASSED + HUMAN-APPROVED (D-194) — FE-001 COMPLETE. Cycle CLOSED. RELEASED as v0.9.3 (D-201). Pipeline quiesced."

# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []

# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-06-17
maintenance_completed_at: "2026-06-17"
maintenance_findings_count: 48
maintenance_blocking: false

# Convergence (archive pointer)
adversary_convergence_counter: SATISFIED
convergence_trajectory: "Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
---

# VSDD Pipeline State — wirerust

## SESSION PAUSED — SAFE TO CLEAR (D-203)

**Previous checkpoint: D-202 (CORPUS-OBS-PCAPNG-IFFCSLEN-001 resolved, 2026-06-22). Archived; all decisions from that checkpoint remain final.**

**WARNING — DO NOT REDO:**
- Do NOT re-cut the v0.9.3 release. It shipped. Tag `v0.9.3` is on main `2dbf461`, GitHub Release with 4 binaries (run 27984557297) is live.
- Do NOT reopen decision-threads (a), (b), or (c) — all three are resolved and closed (a: D-201, b: D-202, c: D-200).
- Do NOT re-run CORPUS-OBS-PCAPNG-IFFCSLEN-001 investigation — RESOLVED (D-202). Root cause confirmed (non-conformant `if_fcslen` option in legacy dumpcap 1.10.0rc file); synthetic SPB coverage accepted; PR #303 merged.
- Do NOT re-run F2/F3/F4/F5/F6/F7 — all phases CONVERGED+COMPLETE+HUMAN-APPROVED.
- Do NOT re-run input-hash rebaseline (STORY-123..128 re-baselined at D-193, BENIGN).

### GROUND-TRUTH HEADs (verified at D-203 — 2026-06-22)

Re-verify on resume before taking any action:

- **develop:** `dd3b069` — PR #303 merge commit (`docs(e2e): record root cause for SPB fixture rejection; resolve CORPUS-OBS-PCAPNG-IFFCSLEN-001`). Verify: `git log -1 --format='%h' develop` must equal `dd3b069`.
- **main:** `2dbf461` — PR #302 merge commit (`chore: release v0.9.3`); tag `v0.9.3` on this commit. Unchanged. Verify: `git log -1 --format='%h' main` must equal `2dbf461`.
- **factory-artifacts:** local == remote at the commit produced by this D-203 checkpoint. Verify: `git -C .factory status` must be clean; `git -C .factory log -1 --format='%h %s'` for reference.
- **Open PRs:** None. Verify: `gh pr list` must return empty.
- **Worktrees:** main repo (develop) + `.factory/` only. No story/feature worktrees open.
- **In-flight work:** Nothing running. Session fully quiesced. All three D-200-era decision threads closed.

### WHAT WAS ACCOMPLISHED THIS SESSION

- **v0.9.3 released (D-201, 2026-06-22):** pcapng capture-format reader (FE-001, E-19 epic) + CWE-407/CWE-401 TCP reassembly DoS fix (PR #298) + F6 formal hardening (VP-INDEX v2.10, 31/31 VPs verified, 94.4%/100% mutation gate) + E2E corpus tests (PRs #299/#300). PR #302 merged to main (`2dbf461`), tag `v0.9.3`, 4 binaries published (GH run 27984557297). Decision-thread (a) CLOSED.
- **E2E corpus smoke test (D-200, PR #301):** `tests/e2e_corpus_smoke_tests.rs` (354 lines): 29 captures iterated, 26 Ok reader-packet-count pins + 3 Err error-class pins, no-panic guard, CI-safe self-skip. Decision-thread (c) CLOSED.
- **CORPUS-OBS-PCAPNG-IFFCSLEN-001 resolved (D-202, PR #303 @ `dd3b069`):** Root cause of SPB fixture rejection byte-level verified — `pcapng-spb-only.pcapng` carries non-conformant `if_fcslen` option (`option_length = 4`; spec mandates 1 byte); produced by legacy dumpcap 1.10.0rc. wirerust and pcap-file crate are correct. Synthetic SPB coverage (STORY-126 + VP-031) accepted as sufficient. Declined Option C1 (pre-strip normalization). Decision-thread (b) CLOSED.
- **All three D-200-era decision threads now closed (a/b/c).** Pipeline fully quiesced: no open PRs, no active cycle, no in-flight work.

### RESUME PROCEDURE (execute in order — do not skip steps)

1. **Run `vsdd-factory:factory-worktree-health`** (BLOCKING) — verify `.factory/` worktree is mounted on `factory-artifacts`, no detached HEAD, no drift.
2. **Read this STATE.md** in full — absorb current state before taking any action.
3. **Verify HEADs match:** `git log -1 --format='%h' develop` must be `dd3b069`; `git log -1 --format='%h' main` must be `2dbf461`.
4. **Confirm no in-flight work:** `gh pr list` must return empty; no story worktrees open (`git worktree list`).
5. **Confirm both trees are clean:** `git status` on develop; `git -C .factory status` on factory-artifacts.
6. **No active cycle** — await human direction. Options: start a new feature cycle, maintenance sweep, or other task.

### OPEN ITEMS (backlog — non-blocking, no active work)

| ID | Summary | Status |
|----|---------|--------|
| SEC-008 | Residual unbounded EPB accumulation on `from_pcap_reader` STREAM path (not CLI-reachable). Streaming-reader hardening story target. DF-VALIDATION-001 required before GitHub issue. | DEFERRED — latent |
| PERF-REASM-NFR-001 | Formal NFR/VP for reassembly per-packet CPU O(1) amortised. Currently covered by regression tests only. | BACKLOG |
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer is statistics-only; tunneling detection is a new feature scope decision for a human. | OPEN — human decision |
| STORY-121 | E-11 process-gap follow-ups. Open draft — human decision on scope. | OPEN DRAFT |
| INPUT-HASH-ERROR-PRESTORY | STORY-001/091/121 have persistent ERROR from `bin/compute-input-hash --scan` (missing/absent inputs blocks; pre-existing). | BACKLOG |

**Not open (resolved — do not reopen):** CORPUS-OBS-PCAPNG-IFFCSLEN-001 (D-202), decision-threads (a)/(b)/(c) (D-200/D-201/D-202), PERF-REASM-DOS-001 (D-197, PR #298), D-194 FE-001 release deferral (released as v0.9.3), CORPUS-OBS-LINKTYPE-NULL-001 (accepted scope decision), all F6 checklist items, all D-200-era items.

### SPEC VERSIONS (final at FE-001 cycle close — D-193/D-194)

prd.md v1.33, error-taxonomy v3.8 (next_free E-INP-016), nfr-catalog v2.3, ADR-009 rev 13, VP-INDEX v2.10 (total 31, 31/31 verified), BC-INDEX v1.69, BC-2.01.009 v1.8 / .010 v2.2 / .011 v1.9 / .012 v2.0 / .013 v1.10 / .014 v1.6 / .015 v1.8 / .016 v1.4 / .017 v1.7 / .018 v1.6, BC-2.12.011 v1.5. 302 active BCs. verification-coverage-matrix.md v1.19 (VP-025..031 status verified).

---

## Status

**FEATURE MODE — pcapng reader cycle CLOSED (feature-pcapng-reader). FE-001 COMPLETE (D-194, human-approved). RELEASED as v0.9.3 (D-201, 2026-06-22). Pipeline quiesced. D-203 SAFE-TO-CLEAR checkpoint written. DO NOT re-run F5/F6/F7 — all CONVERGED+HUMAN-APPROVED.**

Latest release: v0.9.3 (main `2dbf461`, tag `v0.9.3`, 4 binaries, run 27984557297). develop=dd3b069. stories_delivered=77.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog | PASSED | 30/30 lessons; PRs #69-#99 |
| Phase 1 — Spec Crystallization | PASSED 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs |
| Phase 2 — Story Decomposition | PASSED 2026-05-21 | 49 stories / 11 epics / 27 waves |
| Phase 3 — TDD Implementation | PASSED 2026-05-31 | 48/48 stories, 27/27 waves |
| Phase 4 — Holdout Evaluation | PASSED 2026-06-01 | mean 0.949; detail: cycles/v0.1.0-greenfield-spec/ |
| Phase 5 — Adversarial Refinement | PASSED 2026-06-01 | Adversary gate 3/3 SATISFIED |
| Phase 6 — Formal Hardening | PASSED 2026-06-02 | 8 Kani VPs; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0 | RELEASED 2026-06-12 | SS-15 24 BCs; F7 5-dim; tag v0.6.0. Detail: cycles/feature-8-dnp3-v0.5.0/ |
| Feature ARP (E-16) + v0.7.0 | RELEASED 2026-06-16 | STORY-111..115; VP-024 LOCKED. Detail: cycles/feature-arp-v0.7.0/ |
| E-17 ARP QinQ/MACsec + v0.7.1 | RELEASED 2026-06-17 | STORY-116/117; tag v0.7.1 b98a72f |
| Maintenance maint-2026-06-17 | COMPLETE 2026-06-17 | 2 PRs (#261/#262); 5 deferred; 0 blocking |
| E-18 finding-collapse (STORY-118) + v0.8.0 | RELEASED 2026-06-17 | STORY-118; SS-11=29 BCs. Detail: cycles/feature-collapse-v0.8.0/ |
| E-18/E-8 STORY-119 cycle (F1-F7) + v0.9.0 | RELEASED + CLOSED 2026-06-19 | STORY-120/122/119; 293 BCs; tag v0.9.0 986e148. Detail: cycles/feature-story-119-grouped-collapse/ |
| v0.9.1 patch | RELEASED 2026-06-19 | Doc/help; PRs #277/#278; tag v0.9.1 ad4eec8 |
| v0.9.2 patch | RELEASED 2026-06-19 | DNP3 determinism + E2E fixtures; PRs #279/#280; tag v0.9.2 b73b242 |
| Feature pcapng-reader (FE-001) + v0.9.3 | **COMPLETE + RELEASED 2026-06-22 (D-201)** | F1-F7 CONVERGED+HUMAN-APPROVED (D-194). 10 new BCs, VP-INDEX v2.10 (31/31). PR #302 → main `2dbf461`. 4 binaries. Cycle CLOSED. |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`
D-136..D-202: `cycles/feature-pcapng-reader/decisions-archive.md` (archived at cycle close)

| ID | Decision | Date |
|----|----------|------|
| D-200 | E2E corpus smoke test (PR #301, merge `333fd62`) delivered. Decision-thread (c) CLOSED. | 2026-06-22 |
| D-201 | v0.9.3 RELEASED. PR #302 merged to main `2dbf461`; tag `v0.9.3`; 4 binaries run 27984557297; develop back-merged `a7096e1`. Decision-thread (a) CLOSED. | 2026-06-22 |
| D-202 | CORPUS-OBS-PCAPNG-IFFCSLEN-001 RESOLVED. Root cause: non-conformant `if_fcslen` in legacy dumpcap 1.10.0rc file — NOT a wirerust defect. Synthetic SPB coverage accepted. PR #303 (dd3b069). Decision-thread (b) CLOSED. | 2026-06-22 |
| D-203 | SESSION PAUSED — SAFE TO CLEAR checkpoint written. All three D-200-era decision threads closed. Pipeline fully quiesced: no open PRs, no active cycle, no in-flight work. | 2026-06-22 |

## Governance Policy

Full policy text: `.factory/policies.yaml`.

| Policy | Severity |
|--------|----------|
| DF-VALIDATION-001 | HIGH |
| DF-SIBLING-SWEEP-001 (v4) | CRITICAL |
| DF-PR-MANAGER-COMPLETE-001 | HIGH |
| DF-ADVERSARY-METHODOLOGY-001 | HIGH |
| DF-AC-TEST-NAME-SYNC-001 (v2) | MEDIUM |
| DF-CONVERGENCE-BEFORE-MERGE-001 | CRITICAL |
| DF-DEVELOP-FRESHNESS-001 (v2) | HIGH |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | MEDIUM |
| DF-INPUT-HASH-CANONICAL-001 | HIGH |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | HIGH |
| DF-TEST-CITATION-SWEEP-001 | HIGH |
| DF-TEST-NAMESPACE-001 | MEDIUM |
| DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 | HIGH |
| DF-CANONICAL-FRAME-HOLDOUT-001 | CRITICAL |
| DF-BC-COMPLETENESS-SWEEP-001 | HIGH |
| DF-GREEN-DOC-TENSE-SWEEP (v2) | HIGH |
| DF-KANI-NONVACUITY-001 | HIGH |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`.
- Issues: #104/#102 CLOSED; all actions SHA-pinned; pin gate enforced; dtolnay/rust-toolchain @stable/@nightly exempted.
- STORY-INDEX.md is authoritative (81 stories / 56 waves / 521 pts — v2.5, post-F3 D-166). STORY-119/120/122/123 status=done confirmed.
- Drift item detail + per-story adversarial convergence logs: `cycles/feature-pcapng-reader/`.
- Decisions D-136..D-199 archived in `cycles/feature-pcapng-reader/decisions-archive.md` at cycle close.
