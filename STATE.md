---
pipeline: V0.5.0_RELEASED
phase: feature-f3
active_feature: "#8-dnp3"
feature_8_status: "F3-CONVERGED — awaiting F3 gate"
product: wirerust
mode: brownfield
timestamp: 2026-06-10T20:00:00Z
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
develop_head: 10036fc
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
stories_delivered: 54
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
consistency_audit: CONSISTENT
input_drift_check: "CLEAN — MATCH=62/STALE=0/ERROR=1 (STORY-091 no-inputs pre-existing; STORY-106..110 new hashes + 10 prior-stale refreshed post F3 edits)"
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.5.0 RELEASED (MITRE ATT&CK-ICS v19 remap, issue #222 CLOSED). Feature #8 DNP3 F3 CONVERGED (3-pass adversarial; 5 stories STORY-106..110, E-15, v0.6.0 target). Awaiting F3 human gate. D-054.**

**Summary:** 63 stories (48 greenfield + 4 F-cycle + 11 F3-new), 400 pts. 268 BCs (244 pre-F2 + 24 SS-15), 23 VPs (22 locked + VP-023 F2-new), 1338 tests green, holdout 0.967. develop HEAD 10036fc; main HEAD c2df1b5 (v0.5.0). Feature #7: COMPLETE. Feature #8 DNP3: F3 CONVERGED, awaiting gate. develop is ahead of main by 3 non-release chore commits (eb010a1, 92773a4, fb2c875 — PR #221 local-only E2E pcap tooling).

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements; trajectory: `17→…→0→0→0` |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 49 stories / 11 epics / 27 waves; story-adversary 3/3 SATISFIED; input-hash drift CLEAN |
| Phase 3 — TDD Implementation | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves; develop HEAD 6158e6e (PR#170); detail: cycles/phase-3-tdd/ |
| Phase 4 — Holdout Evaluation | **PASSED** 2026-06-01 | mean 0.949, 0 must-pass <0.6; HS-043 real defect fixed (PR #171/#172); detail: cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md |
| Phase 5 — Adversarial Refinement | **PASSED** 2026-06-01 | Adversary gate 3/3 + secondary review COMPLETE; 4 fix-PRs; trajectory: `P1-MED→…→P14-CLEAN-GATE` |
| Phase 6 — Formal Hardening | **PASSED** 2026-06-02 | 8 Kani VPs proven; 6 proptest VPs; fuzz VP-008 21.7M execs 0 crashes; mutation targets met; RUSTSEC-2025-0119 FIXED; 20 VPs LOCKED (614e0e0) |
| Phase 7 — Convergence | **PASSED + RELEASED** (human-approved 2026-06-08) | 6 PASS / 1 CONCERN (Perf — non-blocking); 1126 tests; consistency CONSISTENT (8/8); 20 VPs locked |
| Release — v0.1.0 | **RELEASED** 2026-06-08 | GitHub Release; 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); run 27155277051 all jobs success |
| Feature #100 F4→F7 Cycle | **COMPLETE + RELEASED** 2026-06-09 | STORY-097/098/099 delivered; VP-021 LOCKED; D-030 human gate; shipped in v0.2.0 |
| Release — v0.2.0 | **RELEASED** 2026-06-09 | gitflow-proper: release/0.2.0 → PR #208 → 18be1ba; 4 binaries; run 27216925948; GitHub Release published 2026-06-09T15:28:38Z; D-031 |
| Feature #7 F3 — Incremental Stories | **COMPLETE** 2026-06-09 | 6 new stories STORY-100..105, 58 total / 353 pts; wave schedule + 22 holdout scenarios; D-036 |
| Feature #7 F4 Wave 1 — E-13 Multi-Tag Migration | **DELIVERED** 2026-06-09 | STORY-100+101 PR #209 -> develop c846b3b; 1189 tests; 9/9 CI; AI review APPROVE; OPEN: mitre_attack_version F4-PIN; D-037 |
| Release — v0.3.0 | **RELEASED** 2026-06-09 | gitflow-proper: release/0.3.0 → PR #210 → 9ef5af1; 4 binaries; run 27240476896; GitHub Release published; BREAKING: mitre_techniques array (ECS-aligned); F4-PIN resolved ics-attack-19.1; D-038 |
| Feature #7 F4 Wave 2 — E-14 Modbus Core | **COMPLETE** 2026-06-09 | All 4 stories MERGED: PR #211 (STORY-102, 26d58bb), PR #212 (STORY-103, d894464), PR #213 (STORY-104, dba...), PR #214 (STORY-105, dba5f26). 1324 tests. Modbus LIVE. D-042 |
| Feature #7 F5/F6/F7 — Hardening + Convergence | **F7 CONVERGED** 2026-06-09 | F5 CRITICAL timestamp-units (PR #215); F6 Kani 5/5 + fuzz 3.7M/0 + mutation 100% + audit clean (PR #216); F7 e2e + mod-wrappers (PR #217, 70abc27). Consistency 5-shadow sweep FIXED. 1338 tests. Holdout 0.967. D-044 |
| Release — v0.4.0 | **RELEASED** 2026-06-10 | gitflow-proper: release/0.4.0 → PR #219 → main merge 90aa91e; annotated tag v0.4.0; 4 binaries; run 27254720396; GitHub Release published 2026-06-10T05:12:40Z; Feature #7 COMPLETE + issue #7 CLOSED; main back-merged to develop (8e38041). D-046 |
| Maintenance — MITRE v19 remap (issue #222) | **RELEASED in v0.5.0** 2026-06-10 | 3-pass adversarial CONVERGED; T0855→T1692.001 emitted, T0856→T1692.002 catalogue-only; PR #223 develop; PR #224 → main c2df1b5; annotated tag v0.5.0 @ 9b3a1c6; run 27313698900; 4 binaries; issue #222 CLOSED. D-049/D-050/D-051 |
| Release — v0.5.0 | **RELEASED** 2026-06-10 | gitflow-proper: release/0.5.0 → PR #224 → main merge c2df1b5; annotated tag v0.5.0 @ 9b3a1c6; 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); run 27313698900 SUCCESS; GitHub Release published (not draft); main back-merged to develop (10036fc). D-051 |
| Feature #8 DNP3 — F2 Spec Evolution | **COMPLETE** 2026-06-10 | Research-validated gate (DF-VALIDATION-001). Must-adds: BC-2.15.023 (ENABLE/DISABLE_UNSOLICITED→T0814) + BC-2.15.024 (malformed frame→T0814). SS-15 now 24 BCs. Thresholds CONFIRMED (10/60s, 300s, ≥3). Adversarial delta convergence 3-pass. Total 268 BCs. MITRE 23/15/8 UNCHANGED. D-047/D-048/D-052/D-053. |
| Feature #8 DNP3 — F3 Story Decomposition | **CONVERGED** 2026-06-10; awaiting F3 gate | 5 stories STORY-106..110, epic E-15 'DNP3/ICS Analyzer', 47 pts, waves 35-39, strictly-linear chain. 24 SS-15 BCs covered 1:1. 22 holdout scenarios HS-W35..39. STORY-INDEX 58→63/353→400 pts. 3-pass adversarial CONVERGED. Release target v0.6.0. D-054. |

## Session Resume Checkpoint (2026-06-10 — Feature #8 DNP3 F3 CONVERGED — awaiting F3 gate)

**POSITION:** wirerust v0.5.0 RELEASED (issue #222 CLOSED). develop HEAD `10036fc`; main HEAD `c2df1b5` (v0.5.0). Feature #8 (DNP3) F3 CONVERGED — 3-pass adversarial complete (D-054). STORY-106..110 authored, epic E-15 (47 pts), waves 35-39, 22 holdout scenarios, input-hash MATCH=62/STALE=0/ERROR=1. Release target v0.6.0. NEXT: F3 human gate, then F4 TDD wave 35 (STORY-106 parse-core).

**RELEASE HISTORY:** v0.1.0 (2026-06-08) greenfield; v0.2.0 (2026-06-09) timestamp threading; v0.3.0 (2026-06-09) multi-tag MITRE schema; v0.4.0 (2026-06-10) Modbus TCP analyzer; v0.5.0 (2026-06-10) MITRE ATT&CK-ICS v19 remap.

**DNP3 F3 CONVERGED SPEC STATE:**
- STORY-106..110: strictly-linear chain; 106→107→108→109→110; E-15; waves 35-39; 47 pts total
- STORY-106 (8 pts, w35): DNP3 parse/classify pure-core — VP-023 Kani (4 sub-properties)
- STORY-107 (8 pts, w36): per-flow state + correlation (Dnp3FlowState, 300s window)
- STORY-108 (11 pts, w37): direct detections T1692.001/T0814/T0836 + co-emission + summarize
- STORY-109 (13 pts, w38): correlated/derived T1691.001/T0827 + broadcast/unsolicited/malformed; VP-007 atomic seed; SEEDED 21→23/EMITTED 13→15
- STORY-110 (7 pts, w39): dispatcher Rule-6 + CLI (VP-004 oracle; VP-023 draft→verified)
- 22 holdout scenarios HS-W35..39 (incl. Trace-B spaced-event + Crain-Sistrunk + FP-guards)

**RESUME PROTOCOL FOR NEXT SESSION:**
1. Run `vsdd-factory:factory-worktree-health` — verify .factory/ worktree on factory-artifacts
2. Read STATE.md (this file) — orient; confirm F3 human gate status
3. If gate PASSED: invoke `vsdd-factory:phase-f4-delta-implementation` wave 35 (STORY-106)

**CARRY-FORWARD / OPEN ITEMS:**
- STORY-091: draft, P1, 5 pts, E-11 — anchor-validation tooling; deferred
- CC-001..CC-004 / PG-5/PG-6/PG-7/PG-8: process-gap codification deferred
- Drift items: O-07 (rayon unused), O-08 (dns.rs stale doc), F-W25-S088-P6-001
- RUSTSEC-2026-0097: accepted-transitive; ACTION-PIN-001: dtolnay/rust-toolchain exempt (OPEN P3)
- PCAP-CORPUS-001 (TABLED): storage-backend decision pending
- Dependabot PRs #202-#207: disposition before next release — see Deferred Next-Work Backlog

Prior checkpoint archived: cycles/v0.1.0-greenfield-spec/session-checkpoints.md.

## Decisions Log

D-001..D-046 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.

| ID | Decision | Date | Rationale |
|----|----------|------|-----------|
| D-047 | Feature #8 (DNP3 analyzer, issue #8) F1 delta analysis APPROVED by human (2026-06-10). Intent=feature, type=backend, non-trivial → full F1-F7. Integration: Dnp3Analyzer implements StreamHandler+StreamAnalyzer, wired into StreamDispatcher as DispatchTarget::Dnp3 (port-20000 Rule 6) — mirrors Modbus (D-032), NOT the UDP/ProtocolAnalyzer path; UDP DNP3 deferred to v2. New: src/analyzer/dnp3.rs, subsystem SS-15 'DNP3/ICS', VP-023 (Kani candidate, parse/classify pure core), ADR-007 (binary-ICS TCP integration). Modified (5): dispatcher.rs (HIGH — DispatchTarget::Dnp3 + port-20000 classification + VP-004 oracle), mitre.rs (HIGH/CRITICAL — VP-007 drift guard; T0803 AND T0828 are NEW to catalog, must seed+emit atomically), analyzer/mod.rs, main.rs, cli.rs. DTU_REQUIRED=false (no external service, confirmed). HUMAN SCOPE DECISIONS: (1) integration = TCP-only first (StreamDispatcher); (2) CRC-16/DNP = structure-only, strip-not-validate in v1; (3) MITRE = EXPANDED set T0803(new)+T0828(new)+T0855+T0814+T0836 — human chose to add T0828 Loss of Control beyond the architect's minimal recommendation; both T0803 and T0828 need ATT&CK-ICS v19.1 confirmation (research dispatched); (4) app-layer parse = FIR=1 first-fragment only; (5) CLI = add --dnp3-direct-operate-threshold (mirrors --modbus-write-burst-threshold). Delta-analysis doc: .factory/phase-f1-delta-analysis/dnp3-delta-analysis.md. | 2026-06-10 | Feature #8 F1 gate APPROVED — full F1-F7, TCP-only, expanded MITRE (T0803+T0828 new) |
| D-048 | Two independent research passes (DF-VALIDATION-001 satisfied) confirmed a release-safety defect: the MITRE catalog emits/seeds technique IDs REVOKED in ATT&CK-for-ICS v19.0 while the envelope advertises ics-attack-19.1. Full 21-ID blast-radius audit (.factory/research/mitre-ics-v19-catalog-audit.md): exactly 2 IDs affected — T0855 Unauthorized Command Message → T1692.001 (EMITTED by Modbus in v0.4.0; catalogued v0.3.0+v0.4.0) and T0856 Spoof Reporting Message → T1692.002 (catalogue-only, both releases); both fold into new ICS parent T1692 'Unauthorized Message' (v19 introduced ICS sub-techniques). Other 19 IDs ACTIVE-unchanged. VP-007 structurally cannot catch this (closed-world consistency proof, no external-currency oracle). HUMAN DECISIONS (2026-06-10): (1) FIX-FIRST — run a scoped maintenance fix cycle now (remap T0855→T1692.001, T0856→T1692.002 across mitre.rs + modbus.rs emission sites + tests + affected BCs SS-09/10/11/14 + VP-007 sub-technique-format acceptance + correct stale attack-ics-version-pin.md), ship as its own release (v0.4.1/v0.5.0 TBD), THEN resume DNP3 on the corrected base — mirrors D-035 'isolate the correctness change' precedent. (2) DNP3 (Feature #8) MITRE set corrected to v19.1-accurate IDs: T1692.001 (unauthorized command), T1691.001 (block command, ex-T0803), T0827 Loss of Control (correlated finding, not per-packet; replaces the T0828 misread), T0814, T0836. Issue #222 filed. Feature #8 PAUSED at F1-APPROVED. | 2026-06-10 | MITRE v19 revocation defect — fix-first; DNP3 paused; corrected IDs locked |
| D-049 | MITRE v19 remap fix (issue #222) CONVERGED. Spec delta + code/test remap (T0855→T1692.001 emitted, T0856→T1692.002 catalogue-only) across ~30 spec files + 6 code files + 8 test files. Adversarial convergence: Pass 1 NOT-CONVERGED (incomplete sibling sweep — ADR-005/006 authoritative emission tables, cap-10 counts, domain-debt staged list, stale test fn name; PG-5 propagation-shadow recurrence); Pass 2 adversary CONVERGED but consistency caught story-writer's wrong AC-014 tactic labels (T1692.001/.002→CommandAndControl, T0836→IcsInhibitResponseFunction, T0888→IcsImpairProcessControl) + AC-015 count 6→13; Pass 3 (final) CONSISTENT. Code: 1339 tests green, clippy/fmt clean, Kani VP-007 4/4 SUCCESSFUL, sub-technique format T[0-9]{4}(\.[0-9]{3})? accepted. develop-branch code on fix/mitre-ics-v19-remap (HEAD post-2fbab82). NEXT: code PR → develop, then release. | 2026-06-10 | MITRE v19 remap CONVERGED — 3-pass adversarial (caught 2 propagation shadows + tactic errors) |
| D-050 | MITRE v19 remap fix (issue #222) MERGED to develop via PR #223 (merge commit 33de854; repo allows merge-commits only, not squash). 9/9 CI green on final commit 14a52c6 (test/clippy/fmt/audit/deny/fuzz-build/semantic-PR/action-pin-gate/trust-boundary), security review PASS, AI review APPROVE (0 blocking). Final pr-review finding (7 stale `t0855` test-fn identifiers in modbus_detection_tests.rs) resolved before merge — renamed to `t1692_001` (commit 14a52c6), completing the symbol-level sweep. Behavioral change shipped to develop: Modbus findings now emit T1692.001 (JSON/CSV/terminal) instead of revoked T0855; envelope conforms to pinned ics-attack-19.1. Issue #222 CLOSED. Spec delta on factory-artifacts (01451fe/d1dabf9/c4765e6). NEXT: human-gated gitflow release to main (version disposition v0.4.1 patch vs v0.5.0 minor — changes emitted output). | 2026-06-10 | MITRE v19 remap MERGED to develop (PR #223, 33de854); release pending |
| D-051 | Published wirerust v0.5.0 (gitflow-proper) — MITRE ATT&CK-ICS v19 revocation fix (issue #222). release/0.5.0 → PR #224 → main merge c2df1b5; annotated tag v0.5.0 @ 9b3a1c6; release.yml run 27313698900 SUCCESS, 4 binaries (linux x86_64, macos arm64+x86_64, windows msvc); GitHub Release published (not draft). CHANGELOG [0.5.0]: remap T0855→T1692.001 (emitted by Modbus; behavioral change to JSON/CSV/terminal mitre_techniques output) + T0856→T1692.002 (catalogue-only); conforms to pinned ics-attack-19.1. Shipped MITRE-fix-alone (Dependabot sweep deferred per human). Treated as MINOR (output-contract change) per human disposition. main back-merged to develop (10036fc) — branches in sync, no divergence. Feature #8 DNP3 now resumes at F2 on the corrected base. | 2026-06-10 | v0.5.0 release — MITRE v19 remap (issue #222 CLOSED) |
| D-052 | Feature #8 (DNP3) Phase F2 spec evolution CONVERGED (2026-06-10). Delta: SS-15 'DNP3/ICS Analysis' (22 BCs BC-2.15.001-022, CAP-15), ADR-007 (TCP/ICS integration, port-20000 dispatcher Rule 6, frame_len=5+LENGTH+2*ceil((LENGTH-5)/16) max 292, CRC structure-only, FIR=1), VP-023 (Kani parse-safety, 4 sub-properties incl. frame-len arithmetic), MITRE catalog grown SEEDED 21→23 / EMITTED 13→15 (+T1691.001 'Block Operational Technology Message: Command Message'→IcsInhibitResponseFunction, +T0827 'Loss of Control'→new MitreTactic::IcsImpact), PRD v1.5 (266 BCs), Dnp3FlowState correlation-state (single 300s window, wrapping_sub, pending_requests bounded 256). Adversarial convergence: 5 fresh-context passes — P1 (3 HIGH: fabricated T1691.001 name across 5 sites + tactic cardinality + cap-10 un-propagated; arithmetic slice clean) → P2 (2 CRITICAL regressions from P1 fixes: changelog name leak + window-reset contradiction; struct-orphaned fields; VP-007 not updated) → P3 (consistency CONSISTENT; 2 HIGH: orphaned pending_requests DoS bound + panic-prone subtraction) → P4 (cosmetic) → P5 CONVERGED (0 CRITICAL/0 HIGH). 3 [F2-GATE] human decisions pending: --dnp3-direct-operate-threshold default (10/60s), block-command/T0827 correlation window (300s), T0827 guard (≥3 combined events). NEXT: F2 human gate, then F3 story decomposition. | 2026-06-10 | DNP3 F2 spec evolution CONVERGED (5-pass adversarial) |
| D-053 | Feature #8 (DNP3) F2 gate research-validated + COMPLETE (2026-06-10). Human delegated the F2 gate to research validation (research-agent: .factory/research/dnp3-f2-scope-threshold-validation.md, cited to CISA icsnpp-dnp3 / Crain-Sistrunk / IEEE 1815 / vendor device profiles; it caught+discarded its own deep-research hallucinations). VERDICT: scope COMPLETE for v1 with 2 MUST-ADD detections (applied) + all 3 thresholds CONFIRMED. Applied: BC-2.15.023 (ENABLE/DISABLE_UNSOLICITED 0x14/0x15 → T0814, alarm-suppression primitive) + BC-2.15.024 (malformed/structural-frame anomaly → T0814, the ONLY coverage for the Crain-Sistrunk valid-CRC crash class; CRC-deferral orthogonal to malformed coverage). SS-15 now 24 BCs / total 268. Both map to existing T0814 — MITRE counts UNCHANGED (23/15/8). Thresholds CONFIRMED: --dnp3-direct-operate-threshold 10/60s (count=1 for unexpected-source), correlation window 300s (block 3-of-300s, req-timeout 10s, exclude DIRECT_OPERATE_NR 0x06), T0827 ≥3 distinct events/300s. Additive-delta adversarial convergence: pass1 (C-1 orphaned malformed fields + C-2 reset-owner/parse_errors-collision) → pass2 (F-1 BC-INDEX title + F-2 ADR CRC-rationale) → pass3 (F-3 spec-changelog) — all the SAME sibling-propagation class, core two-counter model (parse_errors lifetime + malformed_in_window windowed) clean throughout; now CONVERGED. NEXT: F3 story decomposition. | 2026-06-10 | DNP3 F2 gate research-validated COMPLETE — 2 must-adds, thresholds confirmed |
| D-054 | Feature #8 (DNP3) Phase F3 story decomposition CONVERGED (2026-06-10). 5 stories STORY-106..110 (epic E-15 'DNP3/ICS Analyzer', 47 pts, waves 35-39, strictly-linear acyclic chain): 106 parse/classify pure-core (VP-023 Kani, 4 sub-properties) → 107 per-flow state+correlation → 108 direct detections (T1692.001/T0814/T0836+co-emission+summarize) → 109 correlated/derived+anomaly (T1691.001/T0827/broadcast/unsolicited/malformed; VP-007 atomic seed T1691.001+T0827, SEEDED 21→23/EMITTED 13→15) → 110 dispatcher Rule-6+CLI (VP-004 oracle; VP-023 draft→verified propagation). All 24 SS-15 BCs covered 1:1. 22 holdout scenarios (HS-W35..39, incl. Trace-B spaced-event + Crain-Sistrunk + FP-guards). STORY-INDEX 58→63 stories/353→400 pts; epics total_bcs→268. Release target v0.6.0 (v0.5.0 already shipped). Adversarial convergence: P1 (C-1 release-target v0.5.0→v0.6.0 + epics 243→268 + VP-coherence + phantom symbols) → P2 (un-swept propagation tail of P1) → P3 CONVERGED after global grep-exhaustion sweep. Core decomposition (BC coverage, acyclic deps, AC↔BC traceability, VP placement, thresholds) clean throughout. NEXT: F3 human gate, then F4 TDD (wave 35 = STORY-106 parse-core). | 2026-06-10 | DNP3 F3 story decomposition CONVERGED (5 stories, E-15, v0.6.0 target) |

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
| FE-001 | pcapng input format not supported (.pcap-only) — v2 idea; see tech-debt-register.md | deferred / v2 / not-filed |
| ACTION-PIN-001 | dtolnay/rust-toolchain @stable and @nightly remain branch-ref — intentionally exempt in the Action pin gate (toolchain installer, channel-selected). | OPEN P3 — low priority |
| PCAP-CORPUS-001 | E2E pcap test-corpus storage backend (R2/B2/Drive-SA) — design ready; precursor PR #221 (fb2c875) landed (E2E-PCAPS.md + bin/fetch-e2e-pcaps + mk_modbus_large_pcap.py); large pcaps gitignored. Detail in session-checkpoints.md. | TABLED — human storage-backend decision pending |
| MITRE-V19-REMAP-001 | MITRE ATT&CK-ICS v19 revocation defect (issue #222, D-048/D-049/D-050/D-051): T0855→T1692.001 and T0856→T1692.002 remapped across mitre.rs + modbus.rs emission sites + tests + BCs SS-09/10/11/14 + VP-007 sub-technique-format acceptance. Spec commit c4765e6 (factory-artifacts). PR #223 MERGED to develop (33de854); PR #224 MERGED to main (c2df1b5); issue #222 CLOSED. | CLOSED — RELEASED in v0.5.0 (D-051) |
| DRIFT-F2-COUNT-001 | Stale "15 seeded IDs" count (true=21) in OUT-OF-SCOPE files: BC-2.10.006.md, prd-supplements/nfr-catalog.md, holdout-scenarios/HS-008 + HS-009. Pre-existing from F2/STORY-100 expansion. Requires DF-VALIDATION-001 before filing. | DEFERRED — separate cleanup, validate before filing |
| DRIFT-SUPERPOWERS-001 | docs/superpowers/specs/2026-04-13-mitre-attack-mapping-design.md + plans/2026-04-13-mitre-attack-mapping.md carry stale pre-F2 catalog (T0855/T0856, singular mitre_technique field). Multiply-stale design drafts. Requires DF-VALIDATION-001 before filing. | DEFERRED — reconcile-or-archive decision pending |

## Deferred Next-Work Backlog (recorded 2026-06-10)

**1. Dependabot PR sweep (6 open PRs)** — disposition before next release. Status: DEFERRED.

| PR | Package | Action |
|----|---------|--------|
| #202 | actions/checkout | MUST close + SHA-pin manually per ACTION-PIN-001 (do NOT merge tag ref) |
| #203 | serde_json | standard cargo bump — review + merge |
| #204 | assert_cmd | standard cargo bump — review + merge |
| #205 | etherparse 0.16→0.20 | 4-minor jump — review API changes before merging |
| #206 | rayon | standard cargo bump — review + merge |
| #207 | clap | standard cargo bump — review + merge |

**2. PCAP-CORPUS-001 storage backend** — Cloudflare R2 (RECOMMENDED) / Backblaze B2 / Drive-SA. Status: TABLED — human decision pending.

**3. Roadmap feature options (post-DNP3)** — candidate next features after Feature #8 ships.

| Issue | Description | Note |
|-------|-------------|------|
| #3 | C2 beaconing detection | — |
| #4 | CSV + SQLite reporters | — |
| #6 | rayon parallel processing | relates to drift O-07 rayon-unused |
| #64/#62/#63 | reporter improvements | — |
| #101 | FP/TP characterization | OPEN-DEBT; blocked on labelled corpus |
| #103 | size-symmetry evasion discriminator | DEFERRED; blocked on labelled corpus |
| FE-001 | pcapng support | deferred v2 |

Status: DEFERRED — pick after Feature #8.

## Cycle-Close Follow-Up Items

CLOSED items (PROCESS-GAP-P5-001, PG-1–PG-4, CC-005, CC-006) archived to `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.

| ID | Description | Status |
|----|-------------|--------|
| PG-5 | DF-SIBLING-SWEEP intra-SS propagation-shadow — 3rd recurrence this cycle. Codify DF-SIBLING-SWEEP-001 v5 (grep-sweep gate after FC-set/title/enum change across intra-SS sibling BCs + VP files + BC-INDEX). | OPEN — codification pending |
| PG-7 | [process-gap] DF-SIBLING-SWEEP-001 does not enumerate architecture-decision records (specs/architecture/decisions/ADR-*.md), domain-debt.md, or docs/superpowers/ design drafts as mandatory sweep targets when a technique-ID/enum changes. This let ADR-005/006 + domain-debt retain the revoked ID through the first sweep (PG-5 lineage, recurrence). Codify: extend DF-SIBLING-SWEEP target list to ADRs + domain-debt + canonical per-event vector tables. | OPEN — codification pending |
| PG-6 | Gemini hybrid caught arithmetic-precision class (truncation-bias, off-by-six ADU, length-gate off-by-one) that 3 Claude rounds missed. Codify PROCESS-ARITHMETIC-REVIEW-001 (dedicated numeric review slice for binary-protocol/threshold features). | OPEN — codification pending |
| CC-001 | DF-SIBLING-SWEEP extension — extend to test-file comments + canonical-vector arithmetic lint (3 recurrences this cycle). | DRAFT — deferred to policy-codification pass |
| CC-002 | VP-lock propagation checklist — must propagate to VP-INDEX, coverage-matrix (tool-column), architecture VP anchors, BC VP-anchor prose, AND recompute consuming-story input-hashes. | DRAFT — deferred to policy-codification pass |
| CC-003 | PROCESS-ARITHMETIC-REVIEW-001 codification — Gemini cross-model slice for detection-math-heavy features (caught truncation/wrap/off-by-one F2; caught seconds-vs-micros CRITICAL F5). | DRAFT — deferred to policy-codification pass |
| CC-004 | F5 dispatcher-boundary test gap — any on_data(timestamp:u32) analyzer needs >=1 test through real dispatcher/reassembler boundary with timestamp_secs-shaped values. | DRAFT — deferred to policy-codification pass |
| PG-8 | [process-gap] RECURRING orphaned-struct-field / sibling-propagation pattern — each BC-authoring burst that adds a Dnp3FlowState field or renames a counter required a follow-up adversarial pass because the canonical struct (architecture-delta + ADR struct sketch), the single reset-owner BC, BC-INDEX titles, and spec-changelog were not updated in the SAME burst (recurred across F2 HIGH-1/2, must-add C-1/C-2, F-1/F-3). Codify a "new/renamed flow-state field" checklist into the BC-authoring dispatch + DF-SIBLING-SWEEP target list: struct (both copies) + reset owner + BC-INDEX/PRD/ARCH-INDEX titles + spec-changelog, all in-burst. | OPEN — codification pending |

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
