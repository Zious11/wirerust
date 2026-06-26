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
| F3 — Incremental Stories | **CONVERGED + HUMAN-APPROVED (D-231, 2026-06-24)** | 9 stories authored: STORY-130..138 (E-20, waves 58-61, 66 pts). All 26 BC-2.17.001..026 assigned. 13 holdouts HS-110..122 (all must-pass). 12 adversary passes total. CONVERGENCE ACHIEVED: 3 consecutive clean passes (P10/P11/P12), 0 HIGH/CRITICAL; BC-5.39.001 MET. Trajectory: 4C/6H→1C/3H→0C/2H→2C/2H→0C/1H→0C/1H→0C/0H→0C/1H→0C/2H→0C/0H→0C/0H→0C/0H. Human gate APPROVED (D-231). Deferred LOW (non-blocking): T0814 prose, BC-2.17.010 PO fix, input-hash:TBD (F4 obligation), STORY-133 mitre.rs baseline reverify. |
| F4 — TDD Implementation | **IN-PROGRESS — Wave 59 FULLY CONVERGED & CLOSED (D-242); Wave 60 IN-PROGRESS: STORY-134 DELIVERED+MERGED PR #323 @e330ccc (D-247, 2026-06-25); stories_delivered=83. NEXT = STORY-135.** | Wave-by-wave cadence (D-231). Wave 58: STORY-130+131 merged (D-234/D-237/D-238). Wave 59: STORY-132+133 merged (D-239/D-241/D-242); develop d562ccc; stories_delivered=82. Wave 60: STORY-134 merged PR #323 develop e330ccc (D-247); stories_delivered=83. Red @5845ff6, Green @f54b9dc; per-story convergence M/N/O 3/3 ACHIEVED (D-246); SEC-001 fix; CI 11/11 green. STORY-135/136/137 remain. Full detail: decisions-archive.md D-232..D-247. |
| F5 — Scoped Adversarial | PENDING | |
| F6 — Targeted Hardening | PENDING | |
| F7 — Delta Convergence | PENDING | |

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
