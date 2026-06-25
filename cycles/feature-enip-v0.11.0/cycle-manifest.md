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
| F2 — Spec Evolution | ADVERSARIAL PASS 4 IN PROGRESS | D-229: UDP/2222 deferred to v0.12.0. Scope now TCP/44818 + CIP ForwardOpen (TCP only). 25 BCs total (BC-2.17.001..025). ADR-010, VP-032 written. F2 consistency audit: 7 findings fixed. Pass 1 (2026-06-24): FAIL 4C/7H/3M/3L — LE endianness, T0846, frame-skip soundness, BC-2.17.025 added, BC-INDEX v1.75. Pass 2 (2026-06-24): FAIL 4C/3H/3M/2L — ADR EMITTED 17→20, VP-032 LE, BC-007/008/009/015/018, PRD 24→25 BCs. Pass 3 (2026-06-24): FAIL 3C/4H/4M — dominant pattern propagation-lag; 11 adversary + 3 bonus BE residues REMEDIATED via exhaustive sweep (BC-2.17.020 write-burst 50, PRD/coverage-matrix BE→LE, --all/--enip, variant count, harness count, service labels, BC-INDEX comments). Process-gap PROPAGATION-LAG-001 recorded in lessons.md; ENGINE-PROPAGATION-GREP-GATE-001 in STATE.md OPEN ITEMS. Two pending-human-confirm: write-burst default=50, ERROR_BURST=5 (OA-001). Convergence counter: 0/3. Pass 4 running. Pending: Pass 4 verdict + F2 human gate. |
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
