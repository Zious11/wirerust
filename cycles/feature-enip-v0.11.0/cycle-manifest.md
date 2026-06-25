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
| F3 — Incremental Stories | **COMPLETE — 2026-06-24** | 9 stories authored: STORY-130..138 (E-20, waves 58-61, 66 pts). All 26 BC-2.17.001..026 assigned. STORY-INDEX v2.8: 91 stories / 61 waves / 592 pts. Epics.md v1.8 (E-20 registered). Dependency-graph.md v3.1 (ENIP chain verified acyclic). Story-location bug fixed: nested stories/stories/ → flat stories/STORY-NNN.md; document_type:story added to all 9; input-hash --scan MATCH. Next: holdout scenarios + adversarial story convergence + F3 gate. |
| F4 — TDD Implementation | PENDING | |
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

TBD — populated at cycle close.

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
