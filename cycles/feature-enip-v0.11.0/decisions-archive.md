---
document_type: decisions-archive
cycle_id: feature-enip-v0.11.0
archived_from: STATE.md Decisions Log
archived_at: ~
archived_decisions: D-228..
---

# Decisions Archive — feature-enip-v0.11.0 (D-228+)

*Active cycle decisions are recorded here as they are archived from STATE.md.*

---

## D-228 — Feature Mode Opened; F1 Delta Analysis PASSED; Human-Approved Scope (2026-06-24)

Pipeline transitions QUIESCED → FEATURE-MODE for GitHub issue #316 (EtherNet/IP + CIP ICS analyzer, SS-17). F1 delta analysis complete at `.factory/feature-f1-delta-analysis/enip-delta-analysis.md`. Human-approved scope at F1 gate:

- **In scope for v0.11.0:** TCP/44818 explicit messaging + UDP/2222 cyclic (implicit) I/O + CIP ForwardOpen connection-lifecycle tracking.
- **Deferred:** TLS/2221 encrypted channel.
- **Carry-buffer cap:** 600 bytes per flow.

Planned spec delta: new subsystem SS-17 (CAP-17), new analyzer `src/analyzer/enip.rs`, ADR-010, VP-032, ~24+ BCs (BC-2.17.xxx), 7-9 stories. DTU NOT required (passive parser).

Research inputs registered:
- `.factory/research/next-ics-protocol-prevalence.md` — protocol selection rationale (EtherNet/IP #1).
- `.factory/research/enip-mitre-ics-tagging.md` — MITRE ATT&CK for ICS v19.1 technique mapping.

MITRE key findings for F2 carry-forward: CIP Stop → T0858, CIP Reset → T0816, CIP firmware → T1693.001 (T0857 REVOKED), identity read → T0888/T0846, SetAttribute/write → T0836, UDP/2222 I/O abuse → T1692.001/.002, ForwardOpen → no dedicated technique (document gap in ADR-010). Do NOT seed T0855/T0856/T0857 (revoked). Open design item: T0858 and T1693.001 carry multi-tactic pairings the single-tactic `MitreTactic` enum does not currently model — VP-007 atomic-obligation decision needed in F2.

Ground truth at open: develop=ff4b82b, main=0cbe922 (v0.10.0). Next: F2 Spec Evolution.

---

## D-229 — F2 Scope Refinement: UDP/2222 Deferred to v0.12.0 (2026-06-24)

At the F2 architecture review, the architect found that UDP/2222 cyclic I/O requires UDP-reassembly infrastructure plus cross-transport ForwardOpen session-correlation that is not present in wirerust (wirerust is TCP-stream-oriented in dispatch). Human approved deferring UDP/2222 to a follow-on cycle (v0.12.0).

**v0.11.0 scope (revised):** TCP/44818 explicit messaging + CIP ForwardOpen connection-lifecycle detection (over TCP only). No T1692.001/.002 (UDP cyclic I/O abuse) BCs written this cycle.

**v0.12.0 backlog:** UDP/2222 cyclic I/O + cross-transport ForwardOpen session-correlation + T1692.001/.002 detection.

ADR-010 Decision 5 documents the deferral rationale. 24 BCs authored (BC-2.17.001..024) covering TCP/44818 path; no SS-17 UDP BCs exist. BC-INDEX v1.74 (329 total / 328 active). OA-001 open: `--enip-write-burst-threshold` default (20/1s) awaiting human confirm at F2 gate.
