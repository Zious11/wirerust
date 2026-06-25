---
pipeline: FEATURE-MODE
phase: F2
phase_status: "F2 adversary convergence: Pass 11 PASS (2/3); content frozen; Pass 12 pending. Severity: 4C/7H→...→0C/0H→0C/1H→0C/0H→0C/0H. 2 open LOW (F-P11-001/002) tracked for pre-gate tidy. Scope reduction: 0x00B1 CIP request detection DEFERRED to v0.12.0 (ADR-010 Decision 8). F2 human gate must confirm scope reduction. Two pending-human-confirm: write-burst default=50, ERROR_BURST=5."
product: wirerust
mode: feature-mode
timestamp: 2026-06-24T15:00:00Z

# Release chain (latest)
released_version: v0.10.0
released_at: "2026-06-24"
release_tag: v0.10.0
release_commit: 0cbe922
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.10.0
release_yml_run: "28109367603 SUCCESS — 4 binaries"
prior_released_version: v0.9.4
prior_released_at: "2026-06-23"
prior_release_tag: v0.9.4
prior_release_commit: 96b49e8
v092_release_tag: v0.9.2
v092_release_commit: b73b242
v091_release_tag: v0.9.1
v091_release_commit: ad4eec8
v090_release_tag: v0.9.0
v090_release_commit: 986e148

# Ground-truth HEADs (verified D-227 — 2026-06-24)
develop_head: ff4b82b
main_head: 0cbe922
factory_artifacts_head: (run `git -C .factory log -1 --format='%h'`)

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
stories_delivered: 78
current_cycle: feature-enip-v0.11.0 (D-228, 2026-06-24)
current_wave: F2 — Spec Evolution SPEC-CONTENT COMPLETE (SS-17 TCP/44818+CIP, 24 BCs BC-2.17.001..024, ADR-010, VP-032; UDP/2222 deferred D-229)

# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []

# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-06-22
maintenance_completed_at: "2026-06-23"
maintenance_findings_count: 38
maintenance_blocking: false
maintenance_prior_run_id: maint-2026-06-17
maintenance_prior_completed_at: "2026-06-17"

# Convergence
adversary_convergence_counter: SATISFIED
convergence_trajectory: "Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
---

# VSDD Pipeline State — wirerust

## Status

**PIPELINE FEATURE-MODE. Cycle `feature-enip-v0.11.0` OPEN. F1 PASSED. F2 Pass 11 PASS (2026-06-24): 0C/0H/0M/2L — adversary states "has converged." All 9 axes clean. 2 LOWs tracked for pre-F2-gate tidy (F-P11-001: VP-032 table Module cell src/ prefix outlier; F-P11-002: BC-2.17.005 Inv 3 DoS arithmetic illustration). Content frozen. Convergence counter 2/3. Pass 12 pending for 3/3. SCOPE NOTE FOR F2 HUMAN GATE: Pass 9 scope reduction — 0x00B1 CIP request detection DEFERRED to v0.12.0; human confirm required. Two pending-human-confirm: write-burst default=50, ERROR_BURST=5. BC-INDEX v1.75 (330/329 active, 25 SS-17 BCs). PROPAGATION-LAG-001 in lessons.md; ENGINE-PROPAGATION-GREP-GATE-001 in OPEN ITEMS.**

Latest release: v0.10.0 (main `0cbe922`, tag `v0.10.0`, 4 binaries, run 28109367603). develop=`ff4b82b`. stories_delivered=78. Target: v0.11.0 (SS-17 EtherNet/IP + CIP TCP/44818). GitHub issue #316.

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run fix cycle fix-pc-013-014-015 — CLOSED (D-226). PRs #310/#312/#313/#314 all merged to develop; #315 merged to main; v0.10.0 released.
- Do NOT re-cut v0.10.0 — RELEASED (main `0cbe922`, tag `v0.10.0`, run 28109367603, 4 binaries).
- Do NOT re-apply the dnp3 `parse_errors` rename (`total_parse_errors` → `parse_errors`) — already on develop and main via PR #313.
- Do NOT re-open PC-013, PC-014, or PC-015 — all resolved (D-222/D-223/D-224/D-225).
- Do NOT convert the 4 `arp.rs` `.expect()` sites to `if-let` — deliberately retained (D-223).
- Do NOT re-run feature-mitre-json-names cycle — CLOSED (D-217). v0.9.4 released.

### GROUND-TRUTH HEADs (verified D-227 — 2026-06-24)

- **develop:** `ff4b82b` — back-merge after v0.10.0 release PR #315. Re-verify: `git rev-parse --short develop` == `ff4b82b`.
- **main:** `0cbe922` — PR #315 merge commit (release v0.10.0); annotated tag `v0.10.0` present on this commit.
- **factory-artifacts:** this D-227 checkpoint commit. Re-verify: `git -C .factory log -1 --format='%h %s'`.
- **Open PRs:** Dependabot #311 only (actions/checkout 6.0.3→7.0.0 — unreviewed, non-blocking). Re-verify: `gh pr list`.
- **Worktrees:** main repo (develop) + `.factory/` only. No story/fix/release worktrees open. Re-verify: `git worktree list`.

### RESUME PROCEDURE (execute in order)

1. Run `vsdd-factory:factory-worktree-health` — BLOCKING. Do not proceed until PASS.
2. Read `.factory/STATE.md` in full.
3. Verify: `git rev-parse --short develop` == `ff4b82b` AND `git rev-parse --short main` == `0cbe922` AND `git tag -l v0.10.0` shows the tag.
4. Verify: `gh pr list` shows only Dependabot #311 (unreviewed).
5. Verify: `git worktree list` shows only main repo + `.factory/` (no story/fix/release worktrees).
6. Confirm both trees clean: `git status` (main repo) and `git -C .factory status`.
7. Pipeline QUIESCED. No active cycle. Await human direction before starting new work.

### OPEN ITEMS (backlog — non-blocking, no active work)

| ID | Summary | Status |
|----|---------|--------|
| ENGINE-PROPAGATION-GREP-GATE-001 | Mechanical changed-value sibling-grep gate (engine improvement); from feature-enip-v0.11.0 F2 Passes 1-3 propagation-lag pattern; see cycles/feature-enip-v0.11.0/lessons.md PROPAGATION-LAG-001. Human decision: story / policy / defer. Must resolve before cycle CLOSE. | OPEN — human review |
| DEPENDABOT-311 | Dependabot PR #311 (actions/checkout 6.0.3→7.0.0) open and unreviewed. | OPEN — human triage |
| DEMO-TAPE-PATH-001 | Demo .tape scripts hardcode ephemeral worktree cd path — should reference stable path. DF-VALIDATION-001 required before GitHub issue. | BACKLOG — low |
| DEMO-MEDIA-CHECKSUM-001 | Demo-evidence binary media lacks a SHA-256 checksum manifest. DF-VALIDATION-001 required before GitHub issue. | BACKLOG — low |
| PO-BACKLOG-MAINT-2026-06-22 | DNP3/ARP/Modbus/finding-collapse holdout coverage gap + HS-064/075/090/098/108 staleness. Human scope decision needed. | OPEN — product-owner |
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer statistics-only; tunneling detection is a human feature scope decision. | OPEN — human decision |
| ISSUE-TRIAGE-OPEN-9 | 9 open GitHub issues triaged: keep-open #255/#252/#103/#101/#67/#63/#3; reframe-needed #6 (rayon obsolete) and #4 (narrow to SQLite — CSV shipped). | OPEN — product-owner |
| STORY-121 | E-11 process-gap follow-ups. Open draft — human decision on scope. | OPEN DRAFT |
| SEC-008 | Residual unbounded EPB accumulation on `from_pcap_reader` STREAM path (not CLI-reachable). DF-VALIDATION-001 required before GitHub issue. | DEFERRED — latent |
| PERF-REASM-NFR-001 | Formal NFR/VP for reassembly per-packet CPU O(1) amortised. | BACKLOG |
| INPUT-HASH-ERROR-PRESTORY | STORY-001/091/121 persistent ERROR from `bin/compute-input-hash --scan` (pre-existing). | BACKLOG |
| INPUT-HASH-STALE | STORY-002..005/076..080/101/120 STALE (pre-existing). | BACKLOG |
| ENGINE-IMPROVEMENT-BACKLOG | ~18 engine proposals pending human review, incl. pr-manager shortstop PAT-001; lessons.md Lessons 1 & 2 / policy candidates convergence-clean-tree-guard + magic-number-sweep-on-count-change. Pointer: `cycles/feature-mitre-json-names/session-review.md`. | BACKLOG — human review |
| ADV-4 | ci.yml audit comment rationale lost (LOW). | DEFERRED LOW |
| DRIFT-UNCOMMITTED-TEST-EDITS-001 | [MEDIUM, process-gap]: F5 committed only src/mitre.rs; 3 test files were working-tree edits; CI caught on push. | DEFERRED MEDIUM |
| DRIFT-BC-TEMPLATE-EC-VP-MAP-001 | [LOW, process-gap]: BC template EC table can have more rows than VP/test-name table. | DEFERRED LOW |
| DRIFT-MITRE-SUBSET-COUNT-TESTS-001 | [LOW]: mitre/multitag dual-count subset tests (21/13 vs 25/17) — pre-existing cruft, no correctness impact. | DEFERRED LOW |
| DRIFT-ARP-DEMO-FIXTURE-001 | [LOW]: No ARP pcap fixture; T0830→TA0100 unit-tested but not demoed live. | DEFERRED LOW |
| DRIFT-READER-ADR-CITATION-001 | reader.rs ADR citation numbers (LOW). | DEFERRED LOW |

All GitHub-issue creation remains DF-VALIDATION-001-gated.

**Resolved — do not reopen:** PC-013 (D-224, spec correction D-223 + test-only PR #312), PC-014 (D-225, PR #313 merged develop `f5c002a` + PR #314 drift fix merged develop `2b348a1`), PC-015 (D-222, PR #310), maint-2026-06-22, O-07, DEP-001/005, DOC-001..009, F-MAJ-001, CORPUS-OBS-PCAPNG-IFFCSLEN-001, decision-threads (a)/(b)/(c), PERF-REASM-DOS-001, all F6 items, feature-mitre-json-names F1-F7 (D-206..D-217).

---

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
| Feature pcapng-reader (FE-001) + v0.9.3 | RELEASED + CLOSED 2026-06-22 (D-201) | F1-F7 CONVERGED+HUMAN-APPROVED (D-194). 10 new BCs, VP-INDEX v2.10. PR #302 → main 2dbf461. 4 binaries. |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking; F-MAJ-001 fixed (a6efb23); PR #304 (e458ce2) + PR #305 (e4abbe2). |
| Feature mitre-json-names (issue #64) + v0.9.4 | RELEASED + CLOSED 2026-06-23 (D-217) | F1-F7 CONVERGED. 5 BCs bumped. BC-INDEX v1.71 (303 BCs). PRs #306/307/308/309. tag v0.9.4 96b49e8. 4 binaries. stories_delivered=78. |
| Fix cycle fix-pc-013-014-015 + v0.10.0 | **CONVERGED + RELEASED + CLOSED 2026-06-24 (D-226)** | All 3 fixes: PC-015 (#310), PC-013 (#312 + spec D-223), PC-014 (#313 breaking rename + CHANGELOG). Evidence resync #314. v0.10.0: PR #315 → main 0cbe922, tag v0.10.0, 4 binaries, run 28109367603. develop back-merged ff4b82b. BC-INDEX v1.73 (305 BCs / 304 active). |
| Feature EtherNet/IP + CIP (issue #316) — v0.11.0 | **F2 ADVERSARIAL CONVERGENCE 2/3** | Pass 11 PASS (2026-06-24): 0C/0H/0M/2L — adversary "has converged." Content frozen. 2 LOWs for pre-F2-gate tidy (F-P11-001/002). Severity: 4C/7H→...→0C/0H→0C/0H. BC-INDEX v1.75 (330/329 active, 25 SS-17 BCs). F-P2-010: SS-10 BC version-bump pending (resolve before F3). HUMAN GATE: must confirm 0x00B1 scope deferral. Pass 12 pending for 3/3. Detail: cycles/feature-enip-v0.11.0/ |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`
D-136..D-202: `cycles/feature-pcapng-reader/decisions-archive.md`
D-206..D-217: `cycles/feature-mitre-json-names/decisions-archive.md`
D-219..D-226: `cycles/fix-pc-013-014-015/decisions-archive.md`
D-228+: `cycles/feature-enip-v0.11.0/decisions-archive.md`

| ID | Decision | Date |
|----|----------|------|
| D-203 | SESSION PAUSED — SAFE TO CLEAR. All D-200-era threads closed. Pipeline quiesced. | 2026-06-22 |
| D-204 | Maintenance sweep maint-2026-06-22 COMPLETE. PR #304 (e458ce2) + PR #305 (e4abbe2) merged. F-MAJ-001 fixed. | 2026-06-23 |
| D-205 | SAFE-TO-CLEAR checkpoint refreshed. main=2dbf461, develop=e4abbe2, PRs=0. Pipeline quiesced. | 2026-06-23 |
| D-217 | v0.9.4 RELEASED. PR #309 → main 96b49e8; tag v0.9.4; run 28053327452 SUCCESS, 4 binaries. feature-mitre-json-names CLOSED. stories_delivered=78. | 2026-06-23 |
| D-218 | SAFE-TO-CLEAR. feature-mitre-json-names CLOSED. develop=0115d0e, main=96b49e8, v0.9.4 released, 0 open PRs. | 2026-06-23 |
| D-225 | PC-014 DELIVERED & MERGED. PR #313 `fix(dnp3)!: rename total_parse_errors -> parse_errors` merged develop `f5c002a` (BREAKING JSON change, human-approved D-220). Anchored BC-2.15.020 v1.4 → STORY-108 AC-010. CHANGELOG breaking entry + jq migration snippet. Post-merge consistency audit: CONSISTENT (7/7 core checks MATCH). DRIFT fix PR #314 `chore(dnp3): resync STORY-108 demo evidence + test comment` merged develop `2b348a1`. Both AC-010/AC-011 demos re-recorded (VHS). DRIFT-1 + DRIFT-2 CLOSED. | 2026-06-24 |
| D-226 | v0.10.0 RELEASED. PR #315 merged main `0cbe922`; annotated tag `v0.10.0` (tag obj 92216e5); release.yml run `28109367603` SUCCESS, 4 binaries; develop back-merged `ff4b82b`. Cycle `fix-pc-013-014-015` CONVERGED + RELEASED + CLOSED. All 3 fixes: PC-015 (#310), PC-013 (#312 + spec D-223), PC-014 (#313). Lessons: cycles/fix-pc-013-014-015/lessons.md (S-7.02 satisfied). | 2026-06-24 |
| D-227 | SAFE-TO-CLEAR checkpoint written. Session that delivered fix-pc-013-014-015 bundle (PC-013/014/015) + released v0.10.0 is COMPLETE and CLOSED. Safe to clear the session. Factory-artifacts durability commit: cycle artifacts (code-delivery/STORY-108/pr-review.md, code-delivery/fix-pc-013-014-015/pr-description.md + review-findings.md, code-delivery/release-0.10.0/pr-description.md) committed to factory-artifacts. | 2026-06-24 |
| D-228 | Feature Mode OPENED. Cycle `feature-enip-v0.11.0` started (issue #316). F1 Delta Analysis PASSED, human-approved. Scope: TCP/44818 explicit messaging + UDP/2222 cyclic I/O + CIP ForwardOpen. Deferred: TLS/2221. Carry-buffer cap: 600 bytes/flow. Planned: SS-17, `src/analyzer/enip.rs`, ADR-010, VP-032, ~24+ BCs (BC-2.17.xxx), 7-9 stories. DTU NOT required. MITRE carry-forward in decisions-archive. F2 Spec Evolution next. | 2026-06-24 |
| D-229 | F2 scope refinement: UDP/2222 deferred to v0.12.0. Architect found UDP/2222 cyclic I/O requires UDP-reassembly + cross-transport ForwardOpen session-correlation not present (TCP-stream-oriented dispatch). Human-approved. v0.11.0 scope now: TCP/44818 explicit messaging + CIP ForwardOpen (TCP only). No T1692.001/.002 BCs this cycle. ADR-010 Decision 5. 24 BCs written (BC-2.17.001..024); BC-INDEX v1.74 (329/328 active). OA-001: --enip-write-burst-threshold default=20 pending human confirm at F2 gate. | 2026-06-24 |

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`.
- Issues: #104/#102/#64 CLOSED; all actions SHA-pinned; dtolnay/rust-toolchain @stable/@nightly exempted.
- STORY-INDEX.md authoritative (82 stories / 57 waves / 526 pts — v2.7).
- Active cycle: `cycles/feature-enip-v0.11.0/` (cycle-manifest.md, decisions-archive.md D-228+). Issue #316.
- Closed cycle: `cycles/fix-pc-013-014-015/` (decisions-archive.md D-219..D-226, lessons.md S-7.02).
- Prior cycle artifacts: `cycles/feature-mitre-json-names/` (decisions-archive.md D-206..D-217, cycle-manifest.md, lessons.md, session-review.md).
