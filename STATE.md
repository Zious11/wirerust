---
pipeline: V0.5.0_RELEASED
phase: feature-f6
active_feature: "#8-dnp3"
feature_8_status: "F5 COMPLETE (scoped adversarial + remediation merged); NEXT = F6 formal hardening → F7 convergence → v0.6.0"
product: wirerust
mode: brownfield
timestamp: 2026-06-12T15:23:29Z
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
develop_head: e685664
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
stories_delivered: 57
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
consistency_audit: CONSISTENT
input_drift_check: "STORY-106/107/108/109/110 regenerated at delivery. Scan MATCH at delivery; STORY-110 hash a9cdfb5 confirmed. All 5 DNP3 stories DELIVERED."
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.5.0 RELEASED (MITRE ATT&CK-ICS v19 remap, issue #222 CLOSED). Feature #8 DNP3 F5 COMPLETE — scoped adversarial + remediation merged (PR #230 e685664); NEXT = F6 formal hardening → F7 convergence → v0.6.0.**

**Summary:** 63 stories (48 greenfield + 4 F-cycle + 11 F3-new), 400 pts. 268 BCs (244 pre-F2 + 24 SS-15), 23 VPs (22 locked + VP-023 F2-new draft), 1338+ tests green, holdout 0.967. develop HEAD e685664; main HEAD c2df1b5 (v0.5.0). Feature #8 DNP3: F5 COMPLETE; 4 issues fixed (F-A-001 DIR-bit P0, F-F5-001 unexpected-source P0, F-F5-002 IcsImpact display, F-F5-003 resync data-loss); 4 BCs bumped (BC-2.15.009/010/016/024). develop ahead of main by 13 commits. 4 F6-gate obligations (3 carried + 1 new: DIR-bit Kani revalidation).

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
| Feature #8 DNP3 — F4 Delta Implementation | **COMPLETE** 2026-06-12 | waves 35-39 / stories 106-110 ALL DELIVERED (STORY-106 PR #225 d0f3586; STORY-107 PR #226 ebb4751; STORY-108 PR #227 9c03fde; STORY-109 PR #228 34443f9; STORY-110 PR #229 ddfa576 — dispatcher Rule 6 + CLI flags; VP-004 oracle synced; 6-pass adversarial 3-clean) |
| Feature #8 DNP3 — F5 Scoped Adversarial + Remediation | **COMPLETE** 2026-06-12 | PR #230 e685664 merged. 4 issues fixed (F-A-001 DIR-bit P0; F-F5-001 unexpected-source P0; F-F5-002 IcsImpact display; F-F5-003 resync data-loss). 10-pass convergence 3/3 CLEAN. 4 BCs bumped. Detail: cycles/feature-8-dnp3-v0.5.0/F5-remediation/ |
| Feature #8 DNP3 — F6 Formal Hardening | **NEXT** | 4 obligations: (1) AC-005 Kani verify_content_first_precedence_exhaustive; (2) VP-023 draft→verified + VP-INDEX bump; (3) VP-004 relock Rules 5/6; (4) NEW: confirm VP-023 Kani harnesses + master-frame proofs hold under corrected is_master_frame (0x80). After F6 → F7 delta convergence → v0.6.0. |

## Session Resume Checkpoint (2026-06-12 — Feature #8 DNP3 F5 COMPLETE — F6 formal hardening NEXT)

**POSITION:** Feature #8 (DNP3 TCP analyzer, issue #8). Phase `feature-f6` (formal hardening). F5 Scoped Adversarial COMPLETE — PR #230 merged e685664 (2026-06-12T15:23:29Z). 4 issues fixed (F-A-001 DIR-bit P0; F-F5-001 unexpected-source P0; F-F5-002 IcsImpact display; F-F5-003 resync data-loss). 10-pass convergence 3/3 CLEAN. 4 BCs bumped (BC-2.15.009/010/016/024). NEXT: Feature #8 F6 formal hardening — execute 4 F6-gate obligations.

**KEY SHAs:** develop HEAD `e685664`; main HEAD `c2df1b5` (v0.5.0 released 2026-06-10); released_version v0.5.0. factory-artifacts HEAD = run `git -C .factory log -1 --format='%h %s'`.

**RELEASE HISTORY:** v0.1.0 (2026-06-08) greenfield; v0.2.0 (2026-06-09) timestamp threading; v0.3.0 (2026-06-09) multi-tag MITRE schema; v0.4.0 (2026-06-10) Modbus TCP analyzer; v0.5.0 (2026-06-10) MITRE ATT&CK-ICS v19 remap (issue #222 CLOSED). develop ahead of main by 13 commits.

**BLOCKING RESUME PROTOCOL (in order):**
1. Run `vsdd-factory:factory-worktree-health` — verify .factory/ worktree on factory-artifacts branch.
2. Read STATE.md (this file) — orient; confirm F5 COMPLETE (PR #230 e685664); NEXT = F6 formal hardening.
3. Execute 4 F6-gate obligations:
   a. AC-005 Kani `verify_content_first_precedence_exhaustive` proof (STORY-106).
   b. VP-023 draft→verified + VP-INDEX bump (after 4 STORY-106 Kani proofs).
   c. VP-004 locked-prose relock to include Rules 5/6.
   d. NEW (F5): Confirm VP-023 Kani harnesses + master-frame-dependent proofs hold under corrected `is_master_frame` mask (0x80) — run `cargo kani` on all VP-023 harnesses; confirm 0 counterexamples.
4. After F6 → F7 delta convergence → v0.6.0 release.

**F5 REMEDIATION SUMMARY:** 4 issues found by F5 holistic + agentic-sliced pre-impl review (3 parallel slices). F-A-001 (P0 DIR-bit: 0x10→0x80, latent since STORY-107, BC-2.15.016 v1.3); F-F5-001 (P0 unexpected-source detection unimplemented, BC-2.15.010 v1.5); F-F5-002 (IcsImpact display collision); F-F5-003 (resync data-loss, BC-2.15.024 v1.3). 10-pass convergence, 2 ARCHITECT REVISION-2 directives. Detail: cycles/feature-8-dnp3-v0.5.0/F5-remediation/

**OPEN BACKLOG / DEFERRED:** Drift items: table below. Deferred work: Dependabot PRs, PCAP-CORPUS-001, roadmap #3/#4/#6 (see section below). Process-gap codification PG-5/PG-7/PG-8 + F5 lessons in `cycles/feature-8-dnp3-v0.5.0/lessons.md` and `cycles/feature-8-dnp3-v0.5.0/F5-remediation/lessons.md`. Prior checkpoint: `cycles/v0.1.0-greenfield-spec/session-checkpoints.md`.

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
| D-060 | STORY-110 (DNP3 StreamDispatcher Rule 6 port-20000/DispatchTarget::Dnp3 + --dnp3-* CLI flags + VP-004 oracle arm, wave 39 FINAL) DELIVERED — PR #229 merged ddfa576 (2026-06-12T03:36:45Z). Step-4.5 adversarial 6-pass 3/3 CLEAN streak (BC-5.39.001): P1 DRIFT-DNP3-DIRECTION-001 re-deferred + AC-010 strengthened; P2 phantom test citation vp007_catalog_drift_guard fixed; P3 phantom Kani citation verify_all_emitted_ids_resolve fixed; P4/P5/P6 CLEAN. VP-004 oracle/production sync CLEAN throughout. Feature #8 F4 Delta Implementation COMPLETE — all 5 DNP3 stories (106-110) delivered. 3 F6-gate obligations recorded (AC-005 Kani; VP-023 lock; VP-004 relock). NEXT = F5 scoped adversarial. | 2026-06-12 |
| D-061 | Feature #8 F5 Scoped Adversarial + Remediation COMPLETE — PR #230 merged e685664 (2026-06-12T15:23:29Z). 4 issues fixed: F-A-001 DIR-bit P0 (is_master_frame 0x10→0x80, latent since STORY-107, BC-2.15.016 v1.3); F-F5-001 unexpected-source detection unimplemented P0 (BC-2.15.010 v1.5, EC-009/010/011); F-F5-002 IcsImpact display collision; F-F5-003 resync data-loss Crain-Sistrunk evasion (BC-2.15.024 v1.3). 10-pass convergence 3/3 CLEAN (P6/P8/P10); 2 ARCHITECT REVISION-2 directives. 4 F6-gate obligations (3 carried + 1 new: VP-023 Kani revalidation under corrected 0x80 mask). | 2026-06-12 |

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
