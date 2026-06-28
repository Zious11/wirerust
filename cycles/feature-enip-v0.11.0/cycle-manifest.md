---
document_type: cycle-manifest
cycle_id: feature-enip-v0.11.0
cycle_type: feature
version: v0.11.0
status: IN-PROGRESS
started: 2026-06-24T00:00:00Z
completed: ~
producer: orchestrator
github_issue: 316
github_issue_url: https://github.com/Zious11/wirerust/issues/316
release_tag: ~
release_commit: ~
release_pr: ~
release_yml_run: ~
---

# Cycle Manifest: feature-enip-v0.11.0 (Feature)

## Summary

GitHub issue #316: Add EtherNet/IP + CIP ICS analyzer (SS-17). Passive TCP/UDP
parser covering TCP/44818 explicit messaging, UDP/2222 cyclic I/O, and CIP
ForwardOpen connection-lifecycle tracking. Minor version bump v0.10.0 → v0.11.0
(new subsystem, additive).

F1 delta analysis: `.factory/feature-f1-delta-analysis/enip-delta-analysis.md`.
Research inputs: `.factory/research/next-ics-protocol-prevalence.md`,
`.factory/research/enip-mitre-ics-tagging.md`.

## Phase Status

| Phase | Status | Notes |
|-------|--------|-------|
| F1 — Delta Analysis | PASSED 2026-06-24 | Human-approved (D-228). TCP/44818 + UDP/2222 + ForwardOpen in scope. TLS/2221 deferred. |
| F2 — Spec Evolution | **COMPLETE — human gate PASSED (D-230, 2026-06-24)** | D-229: UDP/2222 deferred to v0.12.0. Scope: TCP/44818 + CIP ForwardOpen (TCP only; 0x00B2 unconnected carriers only; 0x00B1 deferred v0.12.0). 26 BCs total (BC-2.17.001..026). ADR-010 (Decisions 1-9), VP-032 written. F2 adversarial convergence: 4 consecutive 0-H/C passes (P10/P11/P12/P13). Severity trajectory: 4C/7H→4C/3H→3C/4H→0C/1H→0C/1H→0C/0H→0C/1H→0C/0H→0C/1H→0C/0H→0C/0H→0C/0H(P12)→0C/0H(P13). F2 addendum (D-230): BC-2.17.026 created (--enip-error-burst-threshold CLI flag, u32 default 5, symmetric with --enip-write-burst-threshold); ADR-010 Decision 9 added; ENIP_ERROR_BURST_THRESHOLD constant retired; BC-2.17.014 configurable field; BC-2.17.020 CLI surface updated. BC-INDEX v1.76 (331/330 active). Human gate decisions: (1) proceed to F3; (2) 0x00B2-only scope accepted; (3) thresholds 50/5 as tunable defaults; (4) recalibrate F6. |
| F3 — Incremental Stories | **CONVERGED + HUMAN-APPROVED (D-231, 2026-06-24); STORY-139 EC-X1/EC-X2 fix added (RULING-EDGECASE-001, 2026-06-27)** | 10 stories authored: STORY-130..139 (E-20, waves 58-62, 74 pts). All 26 BC-2.17.001..026 assigned + BC-2.17.016 v2.0 / BC-2.17.008 v1.3 / BC-2.17.012 v1.2 / BC-2.17.018 v1.1 amended by RULING-EDGECASE-001. 13 holdouts HS-110..122 (all must-pass). 12 adversary passes total. CONVERGENCE ACHIEVED: 3 consecutive clean passes (P10/P11/P12), 0 HIGH/CRITICAL; BC-5.39.001 MET. Human gate APPROVED (D-231). STORY-139: release-blocker fix (EC-X1 + EC-X2 + EC-X4 + DRIFT-ENIP-DIRECTION-001); wave 62; dep=STORY-138; status=ready; input-hash=759464a. |
| F4 — TDD Implementation | **IN-PROGRESS — Wave 61 CODE-COMPLETE (D-259). Wave-61 integration gate NEXT.** | Wave-by-wave cadence (D-231). Wave 58: STORY-130+131 merged (D-234/D-237/D-238). Wave 59: STORY-132+133 merged (D-239/D-241/D-242). Wave 60: STORY-134 PR #323 (D-247); STORY-135 PR #324 @84be2fb (D-249); STORY-136 PR #326 @a2cb795 (D-252); STORY-137 PR #327 @72a9106 (D-253/D-254); fix-PR #328 @0f345c6 (D-256): resolve_enip_client_ip port-44818 heuristic, F-W60-001 RESOLVED. Wave-60 integration gate: re-convergence passes A/B/C CLEAN (D-257); human-approved D-258. Wave 61: STORY-138 PR #329 @b4624ef (D-259); 3/3 per-story convergence (BC-5.39.001 MET); CI 11/11 green; F-138-P1-004 OPEN (on_flow_close not invoked — BLOCKS wave gate). stories_delivered=87. Full detail: decisions-archive.md D-232..D-259. |
| F5 — Scoped Adversarial | **CONVERGED** | Wave 61 F5 PASS (D-263). Wave 62 F5 CONVERGED @0607b82 (D-273). Wave 63 F5 CONVERGED @e16ee56 (D-282). Wave 64 F5 CONVERGED @ab37fb5 (D-295). |
| F6 — Targeted Hardening | **PASS** | Wave 61 F6 PASS @fd0c7f3 (D-265). Wave 62 F6 PASS @cee85c0 (D-274). Wave 63 F6 PASS @499c778 (D-283). Wave 64 F6 PASS @235a4a1 (D-296/D-297). |
| F7 — Delta Convergence | **CONVERGED + MERGED** | Wave 61 F7 human-APPROVED (D-267). Wave 62 F7 MERGED PR #334 develop `99a06f4` (D-277). Wave 63 F7 MERGED PR #335 develop `b6d7a01` (D-288). Wave 64 F7 MERGED PR #336 develop `a13b5c5` (D-299, 2026-06-28). stories_delivered=91. v0.11.0 HELD pending human release go-ahead. |

## Planned Scope (F1 Approved)

| Item | Detail |
|------|--------|
| In scope | TCP/44818 explicit messaging + CIP ForwardOpen connection-lifecycle detection (over TCP) |
| Deferred to v0.12.0 | UDP/2222 cyclic I/O — requires UDP-reassembly + cross-transport ForwardOpen session-correlation not present in wirerust (D-229, ADR-010 Decision 5) |
| Deferred to v0.12.0 | 0x00B1 connected-item CIP REQUEST detection (Stop/Reset/write/Identity via 0x00B1 carriers) — sequence-count offset bug found in Pass 9 (F-P9-001 HIGH); deferred via Option A scope reduction. v0.11.0 detects CIP request operations on 0x00B2 unconnected carriers only. ADR-010 Decision 8. |
| Deferred | TLS/2221 encrypted channel |
| Carry-buffer cap | 600 bytes per flow |
| New analyzer | `src/analyzer/enip.rs` |
| New subsystem | SS-17 (CAP-17) |
| ADR | ADR-010 (`.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md`) |
| New VP | VP-032 (`.factory/specs/verification-properties/vp-032-enip-parse-safety.md`) |
| BCs authored | 26 (BC-2.17.001..026; +BC-2.17.025 Pass-1, +BC-2.17.026 F2 addendum D-230) |
| Planned stories | 7-9 |
| DTU required | false (passive parser) |
| Version bump | minor (v0.10.0 → v0.11.0) |

## F2 Spec Artifacts (authored 2026-06-24)

| Artifact | Path | Notes |
|----------|------|-------|
| ADR-010 | `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md` | Decision 5: UDP/2222 deferred to v0.12.0 |
| VP-032 | `.factory/specs/verification-properties/vp-032-enip-parse-safety.md` | 4 sub-properties; 5 Kani harnesses; Sub-A/B/C/D + vp032_cip_service_request_partition |
| BCs | `.factory/specs/behavioral-contracts/ss-17/BC-2.17.001..026.md` | 26 BCs (BC-2.17.001..024 original + BC-2.17.025 session-handshake Pass 1 + BC-2.17.026 error-burst CLI flag F2 addendum D-230); BC-INDEX v1.76 (331 total / 330 active) |
| Architecture delta | `.factory/phase-f2-spec-evolution/enip-architecture-delta.md` | SS-17 subsystem design |
| PRD delta | `.factory/phase-f2-spec-evolution/enip-prd-delta.md` | §2.17 + §7 RTM |
| ARCH-INDEX | `.factory/specs/architecture/ARCH-INDEX.md` | v1.8 |
| VP-INDEX | `.factory/specs/verification-properties/VP-INDEX.md` | v2.11 (total 32 VPs) |
| BC-INDEX | `.factory/specs/behavioral-contracts/BC-INDEX.md` | v1.75 (330 BCs / 329 active) |
| PRD | `.factory/specs/prd.md` | v1.36 |
| CAP-17 | `.factory/specs/domain/capabilities/cap-17-enip-cip-analysis.md` | New domain capability |
| verification-architecture | `.factory/specs/architecture/verification-architecture.md` | v2.5 |
| verification-coverage-matrix | `.factory/specs/architecture/verification-coverage-matrix.md` | v1.20 |

## F3 Holdout Scenarios (authored 2026-06-24)

13 holdout scenarios HS-110..122 written for the EtherNet/IP + CIP feature cycle.
All are **must-pass**. 12 require pcap fixture files (flagged for F4 fixture creation).
HS-110 satisfies policy DF-CANONICAL-FRAME-HOLDOUT-001 (canonical-frame LE holdout).

| HS | Title | Must-Pass | Pcap Fixture Required |
|----|-------|-----------|----------------------|
| HS-110 | enip-canonical-frame-le-header-decode | yes | yes (canonical frame) |
| HS-111 | enip-cip-stop-t0858 | yes | yes |
| HS-112 | enip-cip-reset-t0816 | yes | yes |
| HS-113 | enip-cip-write-burst-t0836-threshold | yes | yes |
| HS-114 | enip-listidentity-t0846-one-shot | yes | yes |
| HS-115 | enip-error-burst-t0888-threshold | yes | yes |
| HS-116 | enip-forwardopen-close-empty-mitre | yes | yes |
| HS-117 | enip-malformed-t0814-structural-anomaly | yes | yes |
| HS-118 | enip-oversize-frame-carry-skip | yes | yes |
| HS-119 | enip-0x00b1-deferral-negative | yes | yes |
| HS-120 | enip-dispatch-port-44818 | yes | yes |
| HS-121 | enip-max-findings-dos-bound | yes | no (synthetic) |
| HS-122 | enip-real-world-corpus | yes | yes |

Files: `.factory/holdout-scenarios/HS-110..122-*.md`

## Open Items

| ID | Summary | Status |
|----|---------|--------|
| OA-001 | Two threshold values confirmed at F2 human gate (D-230): (1) `--enip-write-burst-threshold` default=50 writes/1s (BC-2.17.023) — CONFIRMED tunable default; (2) `--enip-error-burst-threshold` default=5 (BC-2.17.026 NEW) — CONFIRMED tunable default. Both flags require `--enip`/`--all` to activate. | RESOLVED — D-230 human gate |
| F-P2-010 | SS-10 BC version-bump: BC-2.10.005 v1.12 (28 seeded IDs, ICS 13→16, +T0858/T0816/T1693.001) + BC-2.10.008 v1.14 (20 emitted IDs, +T0858/T0816/T0846). BC-INDEX v1.77. | **RESOLVED** — pre-F3 SS-10 catalog bump committed 2026-06-24 |
| F8-01 | 0x4B/GetAndClear labeled "firmware download marker" + T1693.001 — ODVA grounding is a wirerust convention, not normative common service. Add citation/vendor-specific note. | RESOLVED — Pass-10 final-polish burst: note added to BC-2.17.007 Invariant 6 and ADR-010 Decision 7. |
| F8-02 | [process-gap] ADR-010 Decision 4 specifies EnipFlowState but never sketches the EnipAnalyzer aggregate struct. | RESOLVED — Pass-10 final-polish burst: full EnipAnalyzer struct sketch added to ADR-010 Decision 4 with BC cross-reference annotations. |
| F8-03 | BC-2.17.014 should state total_error_count = flow.error_counts_in_window.values().sum(). | RESOLVED — Pass-10 final-polish burst: one-sentence clarification added to BC-2.17.014 Invariant 3. |

## F2 Deferred LOW Polish (RESOLVED in Pass-10 final-polish burst)

Pass 8 PASS (2026-06-24). Three LOWs deferred as non-blocking. All resolved in Pass-10 final-polish burst (2026-06-24).

| ID | Finding | Resolution | Process Gap? |
|----|---------|-----------|--------------|
| F8-01 | 0x4B/GetAndClear — ODVA grounding is a wirerust convention, not normative common service. | RESOLVED: note added to BC-2.17.007 Invariant 6 and ADR-010 Decision 7 table row. | No |
| F8-02 | ADR-010 Decision 4 never sketches the EnipAnalyzer aggregate struct. | RESOLVED: full EnipAnalyzer struct added to ADR-010 Decision 4 with BC cross-reference annotations. | Yes — [process-gap] ADR struct-completeness |
| F8-03 | BC-2.17.014 Invariant 3 missing `total_error_count = flow.error_counts_in_window.values().sum()`. | RESOLVED: one-sentence clarification added to BC-2.17.014 Invariant 3. | No |

## MITRE ATT&CK for ICS Tagging (F2 carry-forward)

Source: `.factory/research/enip-mitre-ics-tagging.md` (v19.1)

| CIP Operation | Technique | Note |
|--------------|-----------|------|
| CIP Stop | T0858 (Change Operating Mode) | Active — use |
| CIP Reset | T0816 | Active — use |
| CIP firmware write | T1693.001 | T0857 REVOKED — use T1693.001 |
| Identity read | T0888 / T0846 | Active — use both |
| SetAttribute / write | T0836 | Active — use |
| UDP/2222 I/O abuse | T1692.001 / T1692.002 | Active — use |
| ForwardOpen | No dedicated technique | Document gap in ADR-010 |

**Revoked — do NOT seed:** T0855, T0856, T0857.

**Open design item for F2:** T0858 and T1693.001 carry multi-tactic pairings
that the single-tactic `MitreTactic` enum does not model yet. VP-007
atomic-obligation decision needed in F2 spec phase.

## Delivered (in progress)

| Story | Title | PR | Merge Commit | Adversarial Convergence | Notes |
|-------|-------|----|-------------|------------------------|-------|
| STORY-130 | EtherNet/IP pure-core parse (BC-2.17.001..004) | #317 | 235ae60 | 3/3 consecutive clean passes (BC-5.39.001 MET; Pass 2/3/4, 0 H/C) | 21/21 tests, SEC-002 hardened, ADR-0010 shipped, demo evidence at docs/demo-evidence/STORY-130/. Deferred LOW: AC-130-001 postcondition citation precision. |
| STORY-131 | EtherNet/IP dispatcher + on_data stub (BC-2.17.019..022) | #318 | edce3bd | 3/3 consecutive clean passes (BC-5.39.001 MET; Pass 1: 1H fixed; Pass 2/3 clean) | 15/15 dispatch + 21/21 parse tests. on_data = bytes_received counter only (D-235 boundary). VP-004 oracle 44818 arm. BC-2.17.023/026 v1.1 (D-236). BC-INDEX v1.82. |
| STORY-132 | CPF + CIP frame-walk + VP-032 Sub-D (BC-2.17.005..009) | #319 | 16d3ce7 | 3/3 consecutive clean passes (BC-5.39.001 MET; Pass 1: 1H fixed; Pass 2/3/4 clean) | 19 cpf_cip tests. VP-032 Sub-D Kani present. M-001 RESOLVED (docs/adr/0010 synced). WAVE59-E2E-001+DEADCODE-001 re-targeted STORY-137. stories_delivered: 80→81. |
| STORY-133 | MITRE seeding T0858/T0816/T1693.001 + VP-007 (BC-2.17.011..013) | #320 | 7f040de | 3/3 consecutive clean passes (BC-5.39.001 MET; Pass 1: 2CRIT+2HIGH fixed; Pass 2/3/4 clean) | 10 mitre_seeding tests. VP-007 6-step complete (EMITTED 17→20, SEEDED 25→28). T1693.001 name/tactic fixed (D-240). stories_delivered: 81→82. |
| STORY-134 | Recon detections T0846/T0888 (BC-2.17.008/010/014) | #323 | e330ccc | 3/3 consecutive clean passes (BC-5.39.001 MET; passes M/N/O, 0 H/C) | 20/20 recon tests. T0846 ListIdentity one-shot + T0888 Pattern A+B + is_non_enip gate. SEC-001 saturating_add fix. BC-2.17.010 v1.1 F8-001, BC-2.17.008 v1.2 sentinel, ADR-010 Decision 4 roster. input-hash 16d03a6. Demo recorded. CI 11/11 green. stories_delivered: 82→83. |
| STORY-135 | CIP command detections T0858/T0816/T0836 (BC-2.17.011/012/013) | #324 | 84be2fb | 3/3 consecutive clean passes (BC-5.39.001 MET; passes 5/6/7, 0 H/C) | 16/16 command_detections tests. Green-doc-tense gate strengthened (22 patterns / self-test 54). GREEN-DOC-TENSE-GATE-COVERAGE-001 RESOLVED. input-hash ae2d871. Demo recorded. CI 11/11 green. stories_delivered: 83→84. |
| STORY-136 | CIP connection-lifecycle detections (BC-2.17.015) | #326 | a2cb795 | 3/3 consecutive clean passes (BC-5.39.001 MET; passes 3/4/5 @b003547, 0 H/C) | 10/10 connection_lifecycle tests. Trajectory 2H→0H(1MED)→CLEAN×3. input-hash 0846e0e MATCH. CI 11/11 green. pr-reviewer APPROVE (NITs: PRF-001 close-count cap-bypass deferred STORY-138; PRF-002/003 spec-correct); security PASS (SEC-006 LOW deferred W7.1). Demo evidence: docs/demo-evidence/STORY-136/. stories_delivered: 84→85 (D-252). |
| STORY-137 | EtherNet/IP frame-walk loop + carry-buffer + T0814 + command_counts single-site + dead-code removal (BC-2.17.016/004/018) | #327 | 72a9106 | 3/3 consecutive clean passes (BC-5.39.001 MET; passes B/C/D, 0 H/C) | on_data = frame-walk loop; `pub flows` added to EnipAnalyzer; `command_counts` relocated to single frame-walk site; `#![allow(dead_code)]` removed; byte-walk + frame-skip use `continue` (RULING-137-001). Trajectory: 2CRIT+2HIGH (P1) → fix → 2HIGH → fix → CLEAN(1MED) → fix → CLEAN×3 (B/C/D). RULING-137-002: carry-overflow is_non_enip latch unreachable (deferred v0.12.0). CI 11/11 green; pr-reviewer APPROVE (0 blocking); security PASS (SEC-137-001 MEDIUM pre-authorized-deferred; SEC-137-002/003 LOW). input-hash f4c8390 MATCH. Demo evidence: docs/demo-evidence/STORY-137/. stories_delivered: 85→86 (D-254). |
| STORY-138 | Session lifecycle + stats + DoS guard + analyzer summary (BC-2.17.025/017/022/021/024) | #329 | b4624ef | 3/3 consecutive clean passes (BC-5.39.001 MET; 0 H/C) | RegisterSession/UnRegisterSession classify+no-finding (BC-2.17.025); on_flow_close fold (BC-2.17.017); MAX_FINDINGS/dropped_findings DoS guard (BC-2.17.022); summarize()→enip_summary canonical keys (BC-2.17.021); pdu_count (BC-2.17.024). F-W60-P1-001 command_counts count-once fix shipped. WAVE-60-TEST-DOC-SWEEP resolved. CI 11/11 green; pr-reviewer APPROVE (2 cycles); security PASS (SEC-001 MEDIUM saturating_add fixed @3f55f11; SEC-002/003/004 LOW). input-hash 0f60353 MATCH. OPEN: F-138-P1-004 (on_flow_close not invoked by dispatcher — BLOCKS Wave-61 gate), F-138-P1-002 (cycle-close). stories_delivered: 86→87 (D-259). |
| STORY-139 | EC-X1/EC-X2 direction-carry + saturating-window fix (BC-2.17.016 v2.0 + BC-2.17.008 v1.3 + BC-2.17.012 v1.2 + BC-2.17.018 v1.1; VP-033 + VP-034) | #334 | 99a06f4 | 3/3 consecutive clean passes (BC-5.39.001 MET); MERGED 2026-06-27 (D-277) | Per-direction carry split (carry_c2s/carry_s2c), on_data direction threading (Modbus pattern), resolve_enip_client_ip → direction-based src_ip, wrapping_sub → saturating_sub (3 windows), T0814 >= 300 → > 300 (EC-X4 pin), malformed_window_start → malformed_window_start_ts. VP-033 + VP-034 proptests. Wave 62. input-hash 581b0fd. |

## Research Artifacts

| File | Description |
|------|-------------|
| `.factory/research/next-ics-protocol-prevalence.md` | Protocol selection rationale — EtherNet/IP ranked #1 |
| `.factory/research/enip-mitre-ics-tagging.md` | MITRE ATT&CK for ICS v19.1 technique mapping for EtherNet/IP + CIP |
| `.factory/feature-f1-delta-analysis/enip-delta-analysis.md` | F1 delta analysis document |

## Key Design Decisions

See `decisions-archive.md` (D-228+).

## Notes

- Carry-buffer cap 600 bytes per flow chosen at F1 gate (D-228).
- DTU assessment: passive parser, no external service calls — DTU NOT required.
