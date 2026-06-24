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
| F2 — Spec Evolution | ADVERSARIAL PASS 3 IN PROGRESS | D-229: UDP/2222 deferred to v0.12.0. Scope now TCP/44818 + CIP ForwardOpen (TCP only). 24 BCs (BC-2.17.001..024) + BC-2.17.025 (session handshake) = 25 BCs total. ADR-010, VP-032 written. F2 consistency audit complete: 7 findings, 5 annotation defects fixed (F7-001..F7-004, F6-001); F7-005 (VP-007 enum gap) deferred to implementation. Adversarial Pass 1 (2026-06-24): FAIL — 4 CRITICAL, 7 HIGH, 3 MEDIUM, 3 LOW; novelty HIGH. Pass 1 fixes APPLIED + committed (2026-06-24): LE endianness (ADR-010 + 8 BCs), CIP segment-mask (BC-2.17.009), T0846 now-emitted (PRD v1.36 reconcile), frame-skip soundness (VP-032 Sub-B/D non-vacuous), session-handshake BC-2.17.025 added. BC count 329→330; BC-INDEX v1.75. Two pending-human-confirm values: write-burst default=50, ERROR_BURST=5 (OA-001 updated). Adversarial Pass 2 (2026-06-24): FAIL — 4 CRITICAL, 3 HIGH, 3 MEDIUM, 2 LOW; novelty HIGH (dominant pattern: half-propagated Pass-1 fixes left anchor docs stale). Pass 2 fixes APPLIED + committed (2026-06-24): ADR-010 EMITTED 17→20 (T0846/T0816/T0888 emitted_ids added), VP-032 BE→LE residual fix, BC-2.17.007 CIP service table 0x0A=MSP+13named/15total, BC-2.17.008 0x00B2 scope gate, BC-2.17.009 PC-2 exact-match, BC-2.17.015 serial=0 deferral, BC-2.17.018 3rd increment site, PRD BC-2.10.005 25→28 + BC-2.17.025 propagation + 24→25 BCs, BC-INDEX 25 SS-17 BCs note. Convergence counter: 0/3. Pass 3 running. Pending: Pass 3 verdict + F2 human gate. |
| F3 — Incremental Stories | PENDING | ~7-9 stories planned |
| F4 — TDD Implementation | PENDING | |
| F5 — Scoped Adversarial | PENDING | |
| F6 — Targeted Hardening | PENDING | |
| F7 — Delta Convergence | PENDING | |

## Planned Scope (F1 Approved)

| Item | Detail |
|------|--------|
| In scope | TCP/44818 explicit messaging + CIP ForwardOpen connection-lifecycle detection (over TCP) |
| Deferred to v0.12.0 | UDP/2222 cyclic I/O — requires UDP-reassembly + cross-transport ForwardOpen session-correlation not present in wirerust (D-229, ADR-010 Decision 5) |
| Deferred | TLS/2221 encrypted channel |
| Carry-buffer cap | 600 bytes per flow |
| New analyzer | `src/analyzer/enip.rs` |
| New subsystem | SS-17 (CAP-17) |
| ADR | ADR-010 (`.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md`) |
| New VP | VP-032 (`.factory/specs/verification-properties/vp-032-enip-parse-safety.md`) |
| BCs authored | 24 (BC-2.17.001..024) |
| Planned stories | 7-9 |
| DTU required | false (passive parser) |
| Version bump | minor (v0.10.0 → v0.11.0) |

## F2 Spec Artifacts (authored 2026-06-24)

| Artifact | Path | Notes |
|----------|------|-------|
| ADR-010 | `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md` | Decision 5: UDP/2222 deferred to v0.12.0 |
| VP-032 | `.factory/specs/verification-properties/vp-032-enip-parse-safety.md` | 4 Kani harnesses; Sub-A/B/C/D |
| BCs | `.factory/specs/behavioral-contracts/ss-17/BC-2.17.001..025.md` | 25 BCs (BC-2.17.001..024 original + BC-2.17.025 session-handshake added Pass 1 remediation); BC-INDEX v1.75 (330 total / 329 active) |
| Architecture delta | `.factory/phase-f2-spec-evolution/enip-architecture-delta.md` | SS-17 subsystem design |
| PRD delta | `.factory/phase-f2-spec-evolution/enip-prd-delta.md` | §2.17 + §7 RTM |
| ARCH-INDEX | `.factory/specs/architecture/ARCH-INDEX.md` | v1.7 |
| VP-INDEX | `.factory/specs/verification-properties/VP-INDEX.md` | v2.11 (total 32 VPs) |
| BC-INDEX | `.factory/specs/behavioral-contracts/BC-INDEX.md` | v1.75 (330 BCs / 329 active) |
| PRD | `.factory/specs/prd.md` | v1.36 |
| CAP-17 | `.factory/specs/domain/capabilities/cap-17-enip-cip-analysis.md` | New domain capability |
| verification-architecture | `.factory/specs/architecture/verification-architecture.md` | v2.5 |
| verification-coverage-matrix | `.factory/specs/architecture/verification-coverage-matrix.md` | v1.20 |

## Open Items

| ID | Summary | Status |
|----|---------|--------|
| OA-001 | Two pending-human-confirm values at F2 gate: (1) `--enip-write-burst-threshold` default=50 writes/1s (updated from 20 after Pass 1 adversary recommendation for high-write CIP environments); (2) `ENIP_ERROR_BURST_THRESHOLD`=5 consecutive errors before circuit-break. BC-2.17.012, BC-2.17.023, BC-2.17.025 flag these. | OPEN — awaiting human confirm at F2 gate |
| F-P2-010 | SS-10 BC version-bump pending (BC-2.10.005/BC-2.10.008): PRD BC-2.10.005 count was reconciled to 28 seeded technique IDs in Pass-2 remediation, but the SS-10 BC bodies and version fields do not yet list all 28 IDs. Must be resolved before F3 entry. | OPEN — resolve before F3 |

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
