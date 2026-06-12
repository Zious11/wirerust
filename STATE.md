---
pipeline: V0.5.0_RELEASED
phase: feature-f4
active_feature: "#8-dnp3"
feature_8_status: "F4 IN PROGRESS — waves 35-38 (STORY-106,107,108,109) DELIVERED; wave 39 STORY-110 (FINAL) next"
product: wirerust
mode: brownfield
timestamp: 2026-06-12T01:12:31Z
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
develop_head: 34443f9
main_head: c2df1b5
released_version: v0.5.0
released_at: "2026-06-10"
release_tag: v0.5.0
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.5.0
release_commit: c2df1b5
prior_released_version: v0.4.0
prior_released_at: "2026-06-10"
prior_release_tag: v0.4.0
prior_release_commit: 90aa91e
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 56
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
consistency_audit: CONSISTENT
input_drift_check: "STORY-106/107/108/109 regenerated at delivery (b465d38/2fd1cec/a4218c5/f0fb436). Scan MATCH=62/STALE=0 (STORY-091 structural ERROR pre-existing/out-of-scope). STORY-110 hash a9cdfb5 — confirm TBD-dup cleaned before delivery; run bin/compute-input-hash --write before delivery"
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.5.0 RELEASED (MITRE ATT&CK-ICS v19 remap, issue #222 CLOSED). Feature #8 DNP3 F4 IN PROGRESS — waves 35-38 (STORY-106,107,108,109) DELIVERED; wave 39 = STORY-110 (FINAL) next; v0.6.0 target.**

**Summary:** 63 stories (48 greenfield + 4 F-cycle + 11 F3-new), 400 pts. 268 BCs (244 pre-F2 + 24 SS-15), 23 VPs (22 locked + VP-023 F2-new), 1338+ tests green, holdout 0.967. develop HEAD 34443f9; main HEAD c2df1b5 (v0.5.0). Feature #8 DNP3: F4 in progress; waves 35-38 DELIVERED (STORY-106 PR #225 d0f3586; STORY-107 PR #226 ebb4751; STORY-108 PR #227 9c03fde; STORY-109 PR #228 34443f9 — 13-pass adversarial 3/3 CLEAN; correlated T1691.001/T0827 detections; MitreTactic::IcsImpact; VP-007 atomic seed; BC-2.15.016 v1.2 resync); wave 39 STORY-110 remaining (FINAL: dispatcher wiring + CLI flags). develop is ahead of main by 7 commits.

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
| Maintenance MITRE v19 remap (issue #222) | **RELEASED in v0.5.0** 2026-06-10 | 3-pass adversarial CONVERGED; PR #223→develop; PR #224→main; issue #222 CLOSED |
| Release v0.5.0 | **RELEASED** 2026-06-10 | c2df1b5; 4 binaries; run 27313698900 SUCCESS |
| Feature #8 DNP3 — F2 Spec Evolution | **COMPLETE** 2026-06-10 | SS-15 24 BCs; 268 total; MITRE 23/15/8; thresholds CONFIRMED |
| Feature #8 DNP3 — F3 Story Decomposition | **PASSED** (human-gated 2026-06-11) | 3 decisions accepted: (a) 5 stories as-is (STORY-109 atomic for VP-007), (b) VP placements VP-023@106/110 VP-007@109 VP-004@110, (c) linear chain 106→107→108→109→110 |
| Feature #8 DNP3 — F4 Delta Implementation | IN PROGRESS 2026-06-12 | waves 35-38 DELIVERED (STORY-106 PR #225 d0f3586; STORY-107 PR #226 ebb4751; STORY-108 PR #227 9c03fde; STORY-109 PR #228 34443f9 — 13-pass adversarial 3/3 CLEAN; T1691.001/T0827 correlated detections; MitreTactic::IcsImpact; VP-007 atomic seed T1691.001+T0827; BC-2.15.016 v1.2 byte-walk resync; 3 new drift items); wave 39 STORY-110 (FINAL) remaining; v0.6.0 target |

## Session Resume Checkpoint (2026-06-12 — Feature #8 DNP3 waves 35-38 STORY-106,107,108,109 DELIVERED — wave 39 STORY-110 FINAL next)

**POSITION:** Feature #8 (DNP3 TCP analyzer, issue #8). Phase `feature-f4` (delta implementation). Waves 35-38 DELIVERED (STORY-106 PR #225 d0f3586; STORY-107 PR #226 ebb4751; STORY-108 PR #227 9c03fde; STORY-109 PR #228 34443f9). NEXT: wave 39 = STORY-110 (FINAL: StreamDispatcher port-20000 Rule 6 / DispatchTarget::Dnp3 wiring per BC-2.15.021 + `--dnp3-*` CLI flags per BC-2.15.017; 8 pts; branch from develop@34443f9). After STORY-110 → Feature #8 F5/F6/F7 (scoped adversarial / hardening / convergence) then v0.6.0 release.

**KEY SHAs:** develop HEAD `34443f9`; main HEAD `c2df1b5` (v0.5.0 released 2026-06-10); released_version v0.5.0. factory-artifacts HEAD = run `git -C .factory log -1 --format='%h %s'`.

**RELEASE HISTORY:** v0.1.0 (2026-06-08) greenfield; v0.2.0 (2026-06-09) timestamp threading; v0.3.0 (2026-06-09) multi-tag MITRE schema; v0.4.0 (2026-06-10) Modbus TCP analyzer; v0.5.0 (2026-06-10) MITRE ATT&CK-ICS v19 remap (issue #222 CLOSED). develop ahead of main by 7 commits.

**BLOCKING RESUME PROTOCOL (in order):**
1. Run `vsdd-factory:factory-worktree-health` — verify .factory/ worktree on factory-artifacts branch.
2. Read STATE.md (this file) — orient; confirm waves 35-38 STORY-106,107,108,109 DELIVERED and wave 39 STORY-110 (FINAL) is NEXT.
3. Deliver STORY-110 via per-story-delivery (worktree → stubs → red gate → TDD → Step-4.5 adversarial 3-clean → demos → PR → merge). INPUT-HASH HYGIENE: run `bin/compute-input-hash --write .factory/stories/STORY-110.md` before delivery (hash a9cdfb5 — confirm TBD-dup cleaned). DRIFT-DNP3-DIRECTION-001 is NOT resolved by STORY-110 (re-deferred post-STORY-110 adv P1 F-110-P1-001; see Drift Items table).
4. After STORY-110 → F5 scoped adversarial → F6 targeted hardening → F7 delta convergence → v0.6.0 release.

**LOCKED DNP3 FACTS (for STORY-110):**
- STORY-110: 8 pts, wave 39, NEXT. Epic E-15, 47 pts, strictly-linear chain 106→107→108→109→110. SS-15 = 24 BCs (BC-2.15.001..024).
- MITRE after STORY-109: SEEDED 21→23 (T1691.001+T0827); EMITTED 13→15. VP-007 seeded. MitreTactic::IcsImpact added.
- STORY-110 scope: StreamDispatcher port-20000 Rule 6 / DispatchTarget::Dnp3 (BC-2.15.021) + `--dnp3-*` CLI flags (BC-2.15.017) + VP-004 oracle. NOTE: DRIFT-DNP3-DIRECTION-001 (direction-aware resolve_master_ip) is NOT in STORY-110 scope — re-deferred post-STORY-110 adv P1 F-110-P1-001.
- Thresholds locked: `--dnp3-direct-operate-threshold 10/60s`; 300s correlation window; block-cmd 3/300s; T0827 ≥3/300s; malformed ≥3/300s; req-timeout 10s.
- Holdout scenarios: `.factory/feature/wave-holdout-scenarios/wave-35-39-holdout.md` (22 scenarios).
- INPUT-HASH: STORY-110 hash a9cdfb5 — run `bin/compute-input-hash --write .factory/stories/STORY-110.md` before delivery (confirm TBD-dup cleaned).

**OPEN BACKLOG / DEFERRED:** Drift items: table below. Deferred work: Dependabot PRs, PCAP-CORPUS-001, roadmap #3/#4/#6 (see section below). Process-gap codification PG-5/PG-7/PG-8 pending: `cycles/feature-8-dnp3-v0.5.0/lessons.md`. Prior checkpoint: `cycles/v0.1.0-greenfield-spec/session-checkpoints.md`.

## Decisions Log

D-001..D-046 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.
D-047..D-054 full text archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md` (Feature #8 / v0.5.0 section).

| ID | Decision | Date |
|----|----------|------|
| D-047 | Feature #8 DNP3 F1 gate APPROVED — full F1-F7, TCP-only, DispatchTarget::Dnp3 (port-20000 Rule 6), SS-15, VP-023, ADR-007. MITRE: T1692.001/T1691.001/T0827/T0814/T0836. | 2026-06-10 |
| D-048 | MITRE v19 revocation defect (T0855→T1692.001, T0856→T1692.002) — fix-first (issue #222); DNP3 paused at F1-APPROVED; corrected MITRE IDs locked for Feature #8. | 2026-06-10 |
| D-049 | MITRE v19 remap CONVERGED — 3-pass adversarial (2 propagation shadows + tactic label errors caught and fixed). | 2026-06-10 |
| D-050 | MITRE v19 remap MERGED to develop via PR #223 (33de854); issue #222 CLOSED; Modbus emits T1692.001; release pending. | 2026-06-10 |
| D-051 | v0.5.0 RELEASED (gitflow-proper: release/0.5.0 → PR #224 → main c2df1b5; tag v0.5.0 @ 9b3a1c6; run 27313698900). MITRE v19 remap only. main back-merged to develop (10036fc). | 2026-06-10 |
| D-052 | Feature #8 F2 spec evolution CONVERGED — SS-15 22 BCs + ADR-007 + VP-023; SEEDED 21→23/EMITTED 13→15; 5-pass adversarial. | 2026-06-10 |
| D-053 | Feature #8 F2 gate research-validated COMPLETE — 2 must-add BCs (BC-023 unsolicited→T0814, BC-024 malformed→T0814); SS-15 now 24 BCs / 268 total; 3 thresholds CONFIRMED. | 2026-06-10 |
| D-054 | Feature #8 F3 story decomposition CONVERGED — 5 stories STORY-106..110, E-15, 47 pts, waves 35-39, 22 holdout scenarios, 3-pass adversarial. | 2026-06-10 |
| D-055 | Feature #8 F3 human gate PASSED — (a) accept 5 stories as-is (STORY-109 stays atomic for VP-007 seed+emit invariant), (b) accept VP placements (VP-023 author@106/lock@110, VP-007@109, VP-004@110), (c) accept strictly-linear chain 106→107→108→109→110. F4 TDD authorized. | 2026-06-11 |
| D-056 | STORY-106 (DNP3 parse-core, wave 35) DELIVERED — PR #225 merged d0f3586. Red Gate PASSED; Step-4.5 adversarial 7-pass 3/3 CLEAN (9 findings resolved); VP-023 4/4 Kani harnesses SUCCESSFUL (Sub-A/B/C/D 0 counterexamples); security APPROVE_WITH_NOTES (0 CRITICAL/HIGH/MED); spec bumps VP-023 v1.4 (0x00 CONFIRM→Management) + BC-2.15.005 v1.2 + STORY-106 v1.5. Wave 36 = STORY-107 next. | 2026-06-11 |
| D-057 | STORY-107 (DNP3 per-flow state + carry buffer + bounds, wave 36) DELIVERED — PR #226 merged ebb4751. Red Gate PASSED; Step-4.5 adversarial 3-pass 3/3 CLEAN (F-1/F-2/F-3 + F-P2-001/F-P2-002 resolved); on_data restructured to real carry-walk (gate-before-count); 3 STORY-106 frames wire-valid (LENGTH 0x06); MAX_DNP3_FRAME_LEN consolidated (MAX_DNP3_CARRY_BYTES deprecated alias); DOC-106-001 resolved; security APPROVE-WITH-NOTES (0 CRITICAL/HIGH; 1 MEDIUM SEC-001 accepted; 2 LOW). Wave 37 = STORY-108 next. | 2026-06-11 |
| D-058 | STORY-108 (DNP3 direct detections T1692.001/T0814/T0836, wave 37) DELIVERED — PR #227 merged 9c03fde (2026-06-11T19:46:40Z). Step-4.5 adversarial 5-pass 3/3 CLEAN streak (BC-5.39.001): P1 source_ip/timestamp None violation fixed (c216118); P2 resolve_master_ip helper extracted + direction-deferral documented (78028cf); P3 stale AC-007 citation fixed (fb64529); P4-P5 CLEAN. Input-hash regenerated to a4218c5 (350c8b1). DRIFT-DNP3-DIRECTION-001 recorded. Wave 38 = STORY-109 next. | 2026-06-11 |
| D-059 | STORY-109 (DNP3 correlated detections T1691.001/T0827 + MitreTactic::IcsImpact + VP-007 atomic seed, wave 38) DELIVERED — PR #228 merged 34443f9 (2026-06-12T01:12:31Z). Step-4.5 adversarial 13-pass 3/3 CLEAN streak (BC-5.39.001; P9 real T0827 detection bug fixed; P11 byte-walk resync adjudicated). BC-2.15.016 v1.2 + BC-2.15.014 v1.6 spec evolution. 3 new drift items recorded (DRIFT-MITRE-EMITTED-LABEL-001, DRIFT-BC-2.15.024-EC006-PROSE-001, DRIFT-DNP3-DIRECTION-001 target updated to STORY-110). Wave 39 = STORY-110 (FINAL) next. | 2026-06-12 |

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
| RUSTSEC-2026-0097 | rand 0.8.5 unsound (transitive via tls-parser→phf 0.11); upstream-only fix path | ACCEPTED-TRANSITIVE — revisit when tls-parser bumps phf→0.12+ |
| FE-001 | pcapng input format not supported (.pcap-only) — v2 idea | deferred / v2 / not-filed |
| ACTION-PIN-001 | dtolnay/rust-toolchain @stable/@nightly remain branch-ref — intentionally exempt in the Action pin gate | OPEN P3 — low priority |
| PCAP-CORPUS-001 | E2E pcap test-corpus storage backend (R2/B2/Drive-SA) — PR #221 (fb2c875) landed; large pcaps gitignored | TABLED — human storage-backend decision pending |
| MITRE-V19-REMAP-001 | T0855→T1692.001 / T0856→T1692.002 remap; PR #223 develop; PR #224 main; issue #222 CLOSED | CLOSED — RELEASED in v0.5.0 |
| DRIFT-F2-COUNT-001 | Stale "15 seeded IDs" count in BC-2.10.006.md, prd-supplements/nfr-catalog.md, HS-008+HS-009 | DEFERRED — validate before filing |
| DRIFT-SUPERPOWERS-001 | docs/superpowers/ carries stale pre-F2 catalog (T0855/T0856, singular mitre_technique field) | DEFERRED — reconcile-or-archive decision pending |
| SEC-106-001 (CWE-129) | STORY-107 must call is_valid_dnp3_frame_header before reading data[3] / entering frame-walk (security review SEC-001; overlaps adv B1 gate-before-count) | SATISFIED — implemented in STORY-107 (gate-before-count frame-walk) |
| SEC-106-002 (CWE-400) | STORY-108: MAX_MASTER_ADDRS cap + push guard; pending-seed relocation SEC-006 | SATISFIED — implemented in STORY-108 (PR #227) |
| DOC-106-001 (cosmetic) | Add CONFIRM (0x00) to Dnp3FcClass::Management variant doc example list | SATISFIED — resolved in STORY-107 (PR #226) |
| STORY-107-CARRY-001 | BC-2.15.009 EC-004 (lone-0x05 sub-2-byte sync deferral), BC-2.15.008 EC-006 (parse_errors for <3-byte payloads), BC-2.15.004 PC4 (caller-side parse_errors on gate failure), multi-block CRC-strip payload_buf indexing | SATISFIED — resolved in STORY-107 frame-walk implementation (PR #226) |
| SEC-107-001 (CWE-400) | Dnp3Analyzer.flows HashMap uncapped — accepted offline-pcap design risk; add max_flows cap if live-capture ever added | ACCEPTED — offline-pcap risk; carry to STORY-108 then forward |
| DRIFT-DNP3-DIRECTION-001 | DNP3 source_ip resolution port-20000-heuristic-only; direction-aware resolution NOT resolved by STORY-110 (out of AC scope; ~100 on_data call-site ripple) | DEFERRED — post-v0.6.0 dedicated chore; see tech-debt-register.md |
| DRIFT-MITRE-EMITTED-LABEL-001 | kani EMITTED_IDS labels T0835/T0831 as emitted but neither is actually emitted (13 actual vs 15 labeled); VP-007 Sub-B sound (resolvability only) | DEFERRED — target: system-level catalogue-accuracy pass; severity LOW |
| DRIFT-BC-2.15.024-EC006-PROSE-001 | BC-2.15.024 EC-006 prose says bailed-flow increments parse_errors; conflicts with BC-2.15.009 PC5 no-op (correct behavior); story EC-006 corrected | DEFERRED — target: PO backlog prose-refresh; severity LOW |

## Deferred Next-Work Backlog (recorded 2026-06-10)

**1. Dependabot PR sweep (6 open PRs)** — disposition before next release.

| PR | Package | Action |
|----|---------|--------|
| #202 | actions/checkout | MUST close + SHA-pin manually per ACTION-PIN-001 (do NOT merge tag ref) |
| #203 | serde_json | standard cargo bump — review + merge |
| #204 | assert_cmd | standard cargo bump — review + merge |
| #205 | etherparse 0.16→0.20 | 4-minor jump — review API changes before merging |
| #206 | rayon | standard cargo bump — review + merge |
| #207 | clap | standard cargo bump — review + merge |

**2. PCAP-CORPUS-001 storage backend** — Cloudflare R2 (RECOMMENDED) / Backblaze B2 / Drive-SA. Status: TABLED — human decision pending.

**3. Roadmap features (post-DNP3):**

| Issue | Description |
|-------|-------------|
| #3 | C2 beaconing detection |
| #4 | CSV + SQLite reporters |
| #6 | rayon parallel processing (relates to O-07) |

## Cycle-Close Follow-Up Items

Open codification items (PG-5/PG-6/PG-7/PG-8, CC-001..CC-004) archived to `cycles/v0.1.0-greenfield-spec/decisions-archive.md` (Feature #8 section) with full lesson text in `cycles/feature-8-dnp3-v0.5.0/lessons.md`.

## Governance Policy

Full policy text: `.factory/policies.yaml`. Detail: `cycles/phase-3-tdd/governance-policy-detail.md`.

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

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`. SS-03 gap in BC numbering intentional.
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 1/2 adversary `cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`; phase 6 hardening `cycles/v0.1.0-greenfield-spec/hardening/`.
- Issues: #104/#102 CLOSED (PRs #194/#195), #100 RELEASED v0.2.0, #101 OPEN-DEBT, #103 DEFERRED. Dependabot PR #193 CLOSED (SHA-pin). 7/8 actions SHA-pinned; pin gate enforced (PR #196).
