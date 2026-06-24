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
