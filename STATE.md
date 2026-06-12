---
pipeline: V0.6.0_RELEASED
phase: released
phase_status: RELEASED
active_feature: "#8-dnp3"
feature_8_status: "v0.6.0 RELEASED 2026-06-12 — DNP3 TCP analyzer; F7 5-dim CONVERGED; tag v0.6.0 + 4 binaries"
product: wirerust
mode: brownfield
timestamp: 2026-06-12T17:08:39Z
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
develop_head: 04f8ccb
main_head: 3e29891
released_version: v0.6.0
released_at: "2026-06-12"
release_tag: v0.6.0
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.6.0
release_commit: 3e29891
prior_released_version: v0.5.0
prior_released_at: "2026-06-10"
prior_release_tag: v0.5.0
prior_release_commit: c2df1b5
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 57
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
f7_convergence_trajectory: "6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM)"
consistency_audit: CONSISTENT
input_drift_check: "MATCH=62 STALE=0 ERROR=1 (STORY-091 known); STORY-106 d0ef956 / STORY-109 cf0bb94 re-stamped"
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.6.0 RELEASED (DNP3 TCP analyzer, issue #8). F7 5-dim convergence achieved (6 fresh-context passes); tag v0.6.0 + 4 binaries via release.yml.**

**Summary:** 63 stories (48 greenfield + 4 F-cycle + 11 F3-new), 400 pts. 268 BCs (244 pre-F2 + 24 SS-15), 23 VPs (22+VP-023 ALL LOCKED), 1496 tests green, holdout 0.967. develop HEAD 04f8ccb; main HEAD 3e29891 (v0.6.0). Feature #8 DNP3: F7 CONVERGED — 6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM); BC-2.15.009 v1.3; HS-INDEX feature-holdouts indexed.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 49 stories / 11 epics / 27 waves; input-hash drift CLEAN |
| Phase 3 — TDD Implementation | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves; develop HEAD 6158e6e |
| Phase 4 — Holdout Evaluation | **PASSED** 2026-06-01 | mean 0.949; detail: cycles/v0.1.0-greenfield-spec/ |
| Phase 5 — Adversarial Refinement | **PASSED** 2026-06-01 | Adversary gate 3/3; trajectory: P1-P14 GATE |
| Phase 6 — Formal Hardening | **PASSED** 2026-06-02 | 8 Kani VPs proven; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 — Convergence | **PASSED + RELEASED** 2026-06-08 | 1126 tests; consistency 8/8 CONSISTENT |
| Release v0.1.0..v0.4.0 | **RELEASED** | v0.1.0 greenfield; v0.2.0 timestamp; v0.3.0 multi-tag; v0.4.0 Modbus |
| Maintenance MITRE v19 remap (issue #222) | **RELEASED in v0.5.0** 2026-06-10 | 3-pass adversarial CONVERGED; PR #223→develop; PR #224→main |
| Release v0.5.0 | **RELEASED** 2026-06-10 | c2df1b5; 4 binaries; run 27313698900 SUCCESS |
| Feature #8 DNP3 — F2 Spec Evolution | **COMPLETE** 2026-06-10 | SS-15 24 BCs; 268 total; MITRE 23/15/8 |
| Feature #8 DNP3 — F3 Story Decomposition | **PASSED** (human-gated 2026-06-11) | 5 stories STORY-106..110; linear chain; VP placements |
| Feature #8 DNP3 — F4 Delta Implementation | **COMPLETE** 2026-06-12 | waves 35-39 / stories 106-110 ALL DELIVERED |
| Feature #8 DNP3 — F5 Scoped Adversarial + Remediation | **COMPLETE** 2026-06-12 | PR #230 e685664; 4 issues fixed; 10-pass 3/3 CLEAN |
| Feature #8 DNP3 — F6 Formal Hardening | **COMPLETE** 2026-06-12 | PR #231 a125c69; 9/9 Kani; 89% mut kill; VP-023 LOCKED |
| Feature #8 DNP3 — F7 Delta Convergence | **CONVERGED** 2026-06-12 | 5-dim convergence; 6 fresh-context passes (final 3 consecutive CONVERGED); PRs #232/#233; BC-2.15.009 v1.3 |
| Release v0.6.0 | **RELEASED** 2026-06-12 | PR #234 (release/0.6.0 → main 3e29891); fixup fb3935c; tag v0.6.0; 4 binaries (release.yml); develop merge-back 04f8ccb |

## Session Resume Checkpoint (2026-06-12 — v0.6.0 RELEASED)

**POSITION:** Feature #8 (DNP3 TCP analyzer, issue #8). v0.6.0 RELEASED 2026-06-12 via gitflow. release/0.6.0 → PR #234 → main 3e29891; fixup fb3935c; tag v0.6.0 annotated; 4 binaries auto-built by release.yml (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu); develop merge-back 04f8ccb.

**KEY SHAs (verified live — do not hardcode factory-artifacts HEAD in current-state sections):**
- develop HEAD: `04f8ccb` (main merge-back; 2026-06-12)
- main HEAD: `3e29891` (v0.6.0 released 2026-06-12)
- factory-artifacts HEAD: `git -C .factory log -1 --format='%h %s'` (run live; never hardcode here)
- released_version: v0.6.0; prior: v0.5.0 (c2df1b5)

**v0.6.0 RELEASE SUMMARY:**
- F7 5-dim: spec(BC-2.15.009 v1.3) / tests(1496 green) / impl(clippy+fmt clean) / verification(VP-023 LOCKED, 9/9 Kani, 89% mutation, 3.19M fuzz/0) / docs(README+CHANGELOG PRs #232/#233)
- Release: PR #234 (9 CI green) → main 3e29891; fixup fb3935c; tag v0.6.0; GitHub Release with 4 binaries
- Input-hash scan: MATCH=62 STALE=0 ERROR=1 (STORY-091 known/out-of-scope)
- DRIFT-DNP3-DOC-T0814-COMPLETENESS-001: RESOLVED in v0.6.0 (T0814 ENABLE/DISABLE trigger sources added to README/CHANGELOG on release branch)
- DRIFT-ENGINE-RELEASECONFIG-STALE-001: PARTIALLY RESOLVED — release-config.yaml prose refreshed this burst

**MERGED PR / SHA HISTORY (F4-F7 + release — immutable audit trail):**
- STORY-106..110 PRs #225-229 → d0f3586/ebb4751/9c03fde/34443f9/ddfa576
- F5 remediation PR #230 → e685664; F6 hardening PR #231 → a125c69
- F7 test-citation PR #232; docs PR #233 → f217f27
- Release fixup fb3935c; PR #234 → main 3e29891; tag v0.6.0; develop merge-back 04f8ccb

**NEXT ACTIONS (post-v0.6.0):**
- Dependabot PR sweep (#202-207); roadmap: #3 C2 beaconing, #4 CSV+SQLite reporters, #6 rayon.

**OPEN DRIFT ITEMS (carry to v0.6.0 release prep):**
- DRIFT-DNP3-DIRECTION-001: resolve_master_ip port-heuristic-only; direction-aware deferred post-v0.6.0.
- DRIFT-MITRE-EMITTED-LABEL-001: kani EMITTED_IDS T0835/T0831 over-label; LOW.
- DRIFT-BC-2.15.024-EC006-PROSE-001: EC-006 prose vs BC-2.15.009 PC5 conflict; LOW.
- DRIFT-SEMGREP-001: semgrep absent; manual CLEAN; non-blocking LOW.
- DRIFT-ENGINE-RELEASECONFIG-STALE-001: release-config.yaml human-prose fields stale since v0.2.0; fix at v0.6.0 release prep; MEDIUM.
- DRIFT-DNP3-DOC-T0814-COMPLETENESS-001: README/CHANGELOG omit T0814 ENABLE/DISABLE_UNSOLICITED as trigger sources; LOW.
- DRIFT-BC-INPUTHASH-TBD-001: SS-15 BC files carry input-hash:TBD; by-design (tool scopes to stories/); known/accepted.

**SESSION-REVIEW LESSONS:** `cycles/feature-8-dnp3-v0.5.0/session-review-f4-f6.md` + `cycles/feature-8-dnp3-v0.5.0/lessons.md`

**PRIOR CHECKPOINTS:** `cycles/v0.1.0-greenfield-spec/session-checkpoints.md`

## Decisions Log

D-001..D-046 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.
D-047..D-054 full text archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md` (Feature #8 / v0.5.0 section).

| ID | Decision | Date |
|----|----------|------|
| D-047 | Feature #8 DNP3 F1 gate APPROVED — full F1-F7, TCP-only, DispatchTarget::Dnp3 (port-20000 Rule 6), SS-15, VP-023, ADR-007. MITRE: T1692.001/T1691.001/T0827/T0814/T0836. | 2026-06-10 |
| D-048 | MITRE v19 revocation defect (T0855→T1692.001, T0856→T1692.002) — fix-first (issue #222); DNP3 paused; corrected MITRE IDs locked. | 2026-06-10 |
| D-049 | MITRE v19 remap CONVERGED — 3-pass adversarial. | 2026-06-10 |
| D-050 | MITRE v19 remap MERGED to develop via PR #223 (33de854); issue #222 CLOSED. | 2026-06-10 |
| D-051 | v0.5.0 RELEASED (gitflow-proper: release/0.5.0 → PR #224 → main c2df1b5; tag v0.5.0; run 27313698900). | 2026-06-10 |
| D-052 | Feature #8 F2 spec evolution CONVERGED — SS-15 22 BCs + ADR-007 + VP-023; 5-pass adversarial. | 2026-06-10 |
| D-053 | Feature #8 F2 gate COMPLETE — 2 must-add BCs (BC-023/024); SS-15 now 24 BCs / 268 total; 3 thresholds CONFIRMED. | 2026-06-10 |
| D-054 | Feature #8 F3 story decomposition CONVERGED — 5 stories STORY-106..110, E-15, 47 pts, waves 35-39, 22 holdout scenarios. | 2026-06-10 |
| D-055 | Feature #8 F3 human gate PASSED — 5 stories accepted; VP placements; strictly-linear chain. F4 TDD authorized. | 2026-06-11 |
| D-056 | STORY-106 DELIVERED — PR #225 d0f3586. VP-023 4/4 Kani SUCCESSFUL. | 2026-06-11 |
| D-057 | STORY-107 DELIVERED — PR #226 ebb4751. Carry-walk gate-before-count; STORY-106 frames wire-valid. | 2026-06-11 |
| D-058 | STORY-108 DELIVERED — PR #227 9c03fde. 5-pass adversarial 3/3 CLEAN. DRIFT-DNP3-DIRECTION-001 recorded. | 2026-06-11 |
| D-059 | STORY-109 DELIVERED — PR #228 34443f9. 13-pass 3/3 CLEAN; MitreTactic::IcsImpact; VP-007 seed. | 2026-06-12 |
| D-060 | STORY-110 DELIVERED — PR #229 ddfa576. Rule 6 + CLI flags + VP-004 oracle. F4 COMPLETE. | 2026-06-12 |
| D-061 | Feature #8 F5 COMPLETE — PR #230 e685664. 4 issues fixed (DIR-bit P0; unexpected-source P0; IcsImpact display; resync). 10-pass 3/3 CLEAN. | 2026-06-12 |
| D-062 | Feature #8 F6 HARDENED — PR #231 a125c69. 9/9 Kani; 89% mut; 3.19M fuzz/0; VP-023 LOCKED v1.5; VP-004 relocked. 4/4 F6 obligations SATISFIED. | 2026-06-12 |
| D-063 | Feature #8 F7 CONVERGED — 5-dim delta convergence after remediation of F-S2-001 (canonical-frame IEEE 1815 provenance: holdout HS-W37-002 + test, PR #232), F-S1-001 (BC-2.15.009 v1.3 initial-delivery-only reconciliation + BC-INDEX/STORY-106 propagation), F-PG-001 (HS-INDEX feature-holdout indexing), F-CC-001 (HS-W36-001 stale carry assertion), F-CC-002 (STORY-106..110 status drift), F-CC-003/004 (README/CHANGELOG DNP3 docs, PR #233). 6 fresh-context adversarial passes; final 3 consecutive CONVERGED. develop f217f27. NEXT = human gate → v0.6.0. | 2026-06-12 |
| D-064 | v0.6.0 RELEASED — gitflow release/0.6.0 → PR #234 → main 3e29891; fixup fb3935c; tag v0.6.0; GitHub Release WITH 4 binaries (release.yml auto-build); develop merge-back 04f8ccb. DNP3 TCP analyzer is the headline feature. | 2026-06-12 |

## Blocking Issues

None open.

## Drift Items / Tech Debt Pointers

All items require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Full tech-debt register: `.factory/tech-debt-register.md`.

| ID | Summary | Status |
|----|---------|--------|
| ADV-HS043-P02-MED-001 | Idle-flow expiry monotonic watermark stalls on multi-epoch captures | ACCEPTED — gated on live-capture support |
| O-07 | rayon declared in Cargo.toml but unused | OPEN P2 |
| O-08 | dns.rs module doc-comment stale | OPEN P3 |
| F-W25-S088-P6-001 | AC-004 warning .contains() — weaker than count-assertion | OPEN — target next main.rs touch or accept |
| RUSTSEC-2026-0097 | rand 0.8.5 unsound (transitive via tls-parser→phf 0.11); upstream-only fix | ACCEPTED-TRANSITIVE |
| FE-001 | pcapng input format not supported — v2 idea | deferred / v2 |
| ACTION-PIN-001 | dtolnay/rust-toolchain @stable/@nightly exempt in pin gate | OPEN P3 |
| PCAP-CORPUS-001 | E2E pcap test-corpus storage backend — PR #221 landed; large pcaps gitignored | TABLED — human decision pending |
| MITRE-V19-REMAP-001 | T0855/T0856 remap; PR #223 develop; PR #224 main | CLOSED — RELEASED in v0.5.0 |
| DRIFT-F2-COUNT-001 | Stale "15 seeded IDs" count in BC-2.10.006.md, prd-supplements, HS-008/009 | DEFERRED |
| DRIFT-SUPERPOWERS-001 | docs/superpowers/ carries stale pre-F2 catalog | DEFERRED |
| SEC-106-001..002 | CWE-129 gate-before-count; CWE-400 MAX_MASTER_ADDRS cap | SATISFIED |
| STORY-107-CARRY-001 | BC EC-004/EC-006/PC4 deferrals; multi-block indexing | SATISFIED |
| DRIFT-DNP3-DIRECTION-001 | source_ip resolution port-heuristic-only; direction-aware deferred post-v0.6.0 | DEFERRED |
| DRIFT-MITRE-EMITTED-LABEL-001 | kani EMITTED_IDS T0835/T0831 over-label; system-level | DEFERRED LOW |
| DRIFT-BC-2.15.024-EC006-PROSE-001 | EC-006 prose vs BC-2.15.009 PC5 conflict; PO prose-refresh | DEFERRED LOW |
| DRIFT-SEMGREP-001 | semgrep absent; manual CLEAN; non-blocking | DEFERRED LOW |
| DRIFT-ENGINE-CHECKOUT-GUARD-001 | adversary dispatch template missing checkout-guard; engine fix needed | ENGINE-NOTE HIGH |
| DRIFT-ENGINE-PRMGR-REPORT-001 | pr-manager omitting consolidated report on 4/5 PRs; engine fix needed | ENGINE-NOTE MEDIUM |
| DRIFT-ENGINE-RELEASECONFIG-STALE-001 | release-config.yaml human-prose fields refreshed this burst (human_approval_prompt version-agnostic; test counts updated to 1496 / 23 VPs; release.yml stale note corrected); engine template follow-up (version_sources) DEFERRED | PARTIALLY RESOLVED |
| DRIFT-DNP3-DOC-T0814-COMPLETENESS-001 | RESOLVED in v0.6.0 — README/CHANGELOG T0814 ENABLE/DISABLE_UNSOLICITED trigger sources added on release branch; also corrected pre-existing README "—" technique error for unsolicited-response row → T0814 | RESOLVED |
| DRIFT-BC-INPUTHASH-TBD-001 | all 24 SS-15 BC files carry input-hash:TBD; compute-input-hash scopes to .factory/stories/ per CLAUDE.md; by-design; known/accepted, non-blocking | BY-DESIGN LOW |

## Deferred Next-Work Backlog

**1. Dependabot PR sweep (6 open PRs):**

| PR | Package | Action |
|----|---------|--------|
| #202 | actions/checkout | MUST close + SHA-pin manually (do NOT merge tag ref) |
| #203 | serde_json | review + merge |
| #204 | assert_cmd | review + merge |
| #205 | etherparse 0.16→0.20 | review API changes before merging |
| #206 | rayon | review + merge |
| #207 | clap | review + merge |

**2. PCAP-CORPUS-001:** R2/B2/Drive-SA — TABLED, human decision pending.

**3. Roadmap (post-DNP3):** #3 C2 beaconing | #4 CSV+SQLite reporters | #6 rayon parallel (relates to O-07).

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
| DF-DEVELOP-FRESHNESS-001 | HIGH |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | MEDIUM |
| DF-INPUT-HASH-CANONICAL-001 | HIGH |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | HIGH |
| DF-TEST-CITATION-SWEEP-001 | HIGH |
| DF-TEST-NAMESPACE-001 | MEDIUM |
| DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 | HIGH |
| DF-CANONICAL-FRAME-HOLDOUT-001 | CRITICAL |
| DF-BC-COMPLETENESS-SWEEP-001 | HIGH |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`; F6 hardening `cycles/feature-8-dnp3-v0.5.0/F6-hardening/`.
- Issues: #104/#102 CLOSED (PRs #194/#195), #100 RELEASED v0.2.0, #101 OPEN-DEBT, #103 DEFERRED. Dependabot PRs #202-207 open (see backlog). 7/8 actions SHA-pinned; pin gate enforced (PR #196).
