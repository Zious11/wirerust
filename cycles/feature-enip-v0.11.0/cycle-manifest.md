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
| F2 — Spec Evolution | ADVERSARIAL CONVERGENCE 1/3 — Pass 10 PASS; Passes 11-12 pending | D-229: UDP/2222 deferred to v0.12.0. Scope now TCP/44818 + CIP ForwardOpen (TCP only). 25 BCs total (BC-2.17.001..025). ADR-010, VP-032 written. F2 consistency audit: 7 findings fixed. Pass 1 (2026-06-24): FAIL 4C/7H/3M/3L — LE endianness, T0846, frame-skip soundness, BC-2.17.025 added, BC-INDEX v1.75. Pass 2 (2026-06-24): FAIL 4C/3H/3M/2L — ADR EMITTED 17→20, VP-032 LE, BC-007/008/009/015/018, PRD 24→25 BCs. Pass 3 (2026-06-24): FAIL 3C/4H/4M — dominant pattern propagation-lag; 11 adversary + 3 bonus BE residues REMEDIATED via exhaustive sweep. Pass 4 (2026-06-24): FAIL 0C/1H/4M/2L — anchor/RTM/capability residues all REMEDIATED. Pass 5 (2026-06-24): FAIL 0C/1H/3M/1L REMEDIATED — ARCH-INDEX EMITTED 17→20/catalogue-only 8, SS-17 BC count 24→25, PRD §6.5 T0846 emitted. ARCH-INDEX v1.7→v1.8. Pass 6 (2026-06-24): PASS 0C/0H/2M/1L — FIRST CLEAN PASS. All REMEDIATED. Pass 7 (2026-06-24): FAIL 0C/1H/0M/1L REMEDIATED — ADR-010 Decision 4 doc-comment strict `>` reword. Convergence counter RESET to 0/3. Pass 8 (2026-06-24): PASS 0C/0H/0M/3L — ALL 9 AXES CLEAN. 3 LOW deferred. Counter: 1/3. Pass 9 (2026-06-24): FAIL 0C/1H/1M/2L-obs — 0x00B1 protocol-correctness bug; REMEDIATED via Option A scope reduction (0x00B1 CIP request detection DEFERRED to v0.12.0; BC-2.17.006/011/012/013/014/015 + ADR-010 Decision 8 updated). Counter RESET to 0/3. Pass 10 (2026-06-24): PASS 0C/0H/0M/3L — ALL 9 AXES CLEAN. 3 P10 LOWs REMEDIATED (VP-032 module label, BC-2.17.014 PC4 mirror-rationale, ADR-010 status proposed→accepted). 3 P8-deferred LOWs REMEDIATED (F8-01 0x4B convention note, F8-02 EnipAnalyzer struct sketch, F8-03 total_error_count sum clarification). ADR-010 accepted 2026-06-24. Convergence counter: 1/3. Severity trajectory: 4C/7H→4C/3H→3C/4H→0C/1H→0C/1H→0C/0H→0C/1H→0C/0H→0C/1H→0C/0H. NOTE FOR F2 HUMAN GATE: Pass 9 scope reduction — 0x00B1 CIP request detection DEFERRED to v0.12.0; human confirmation required at F2 gate. |
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
| Deferred to v0.12.0 | 0x00B1 connected-item CIP REQUEST detection (Stop/Reset/write/Identity via 0x00B1 carriers) — sequence-count offset bug found in Pass 9 (F-P9-001 HIGH); deferred via Option A scope reduction. v0.11.0 detects CIP request operations on 0x00B2 unconnected carriers only. ADR-010 Decision 8. |
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
| VP-032 | `.factory/specs/verification-properties/vp-032-enip-parse-safety.md` | 4 sub-properties; 5 Kani harnesses; Sub-A/B/C/D + vp032_cip_service_request_partition |
| BCs | `.factory/specs/behavioral-contracts/ss-17/BC-2.17.001..025.md` | 25 BCs (BC-2.17.001..024 original + BC-2.17.025 session-handshake added Pass 1 remediation); BC-INDEX v1.75 (330 total / 329 active) |
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
| OA-001 | Two pending-human-confirm values at F2 gate: (1) `--enip-write-burst-threshold` default=50 writes/1s (updated from 20 after Pass 1 adversary recommendation for high-write CIP environments); (2) `ENIP_ERROR_BURST_THRESHOLD`=5 consecutive errors before circuit-break. BC-2.17.012, BC-2.17.023, BC-2.17.025 flag these. | OPEN — awaiting human confirm at F2 gate |
| F-P2-010 | SS-10 BC version-bump pending (BC-2.10.005/BC-2.10.008): PRD BC-2.10.005 count was reconciled to 28 seeded technique IDs in Pass-2 remediation, but the SS-10 BC bodies and version fields do not yet list all 28 IDs. Must be resolved before F3 entry. | OPEN — resolve before F3 |
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
