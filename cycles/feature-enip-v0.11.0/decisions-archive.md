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

---

## D-230 — F2 Human Gate APPROVED; F2 Addendum BC-2.17.026 (--enip-error-burst-threshold) (2026-06-24)

F2 human gate PASSED. Human decisions recorded:

1. **Proceed to F3** — F2 adversarial convergence (4 consecutive 0-H/C passes P10-P13) accepted; consistency audit complete; addendum scoped re-validation pending before F3 entry.
2. **0x00B2-only CIP detection scope accepted** — 0x00B1 connected-item detection remains DEFERRED to v0.12.0 (ADR-010 Decision 8). v0.11.0 detects CIP request operations on 0x00B2 unconnected carriers only.
3. **Both detection thresholds accepted as tunable defaults** — write-burst default=50 (`--enip-write-burst-threshold`, BC-2.17.023) and error-burst default=5 (`--enip-error-burst-threshold`, BC-2.17.026 NEW). Both require `--enip`/`--all` to activate. Neither is an absolute hard-stop; operators may recalibrate via CLI flag.
4. **Recalibrate F6** — the addition of two tunable CLI flags means F6 targeted hardening should include boundary / off-by-one testing for both threshold paths. F6 scope note recorded here for orchestrator.

**F2 addendum committed (feature-enip-v0.11.0, factory-artifacts):**

- `BC-2.17.026` CREATED — `--enip-error-burst-threshold` CLI flag configures T0888 error-burst detection sensitivity; u32, default 5, strict `>` semantics, symmetric with BC-2.17.023 write-burst flag.
- `ADR-010` Decision 9 added — flag spec: `--enip-error-burst-threshold <N>` (u32, default 5); `EnipAnalyzer` gains `enip_error_burst_threshold: u32` field initialised from CLI arg; `ENIP_ERROR_BURST_THRESHOLD` compile-time constant RETIRED in favour of the instance field.
- `BC-2.17.014` updated — replaced hardcoded `ENIP_ERROR_BURST_THRESHOLD` constant reference with configurable `self.enip_error_burst_threshold` field; added BC-2.17.026 cross-reference.
- `BC-2.17.020` updated — added `--enip-error-burst-threshold` to CLI surface (three ENIP flags: `--enip`, `--enip-write-burst-threshold`, `--enip-error-burst-threshold`); added BC-2.17.026 to Related BCs.
- `BC-INDEX` v1.75→v1.76 — SS-17: 25→26 BCs; total on disk: 330→331; active: 329→330.
- `ARCH-INDEX` — SS-17 row updated to BC count 26.
- `prd.md` — §2.17 section header updated; §7 RTM BC-2.17.026 row added.
- `cap-17-enip-cip-analysis.md` — BC-2.17.026 registered.

**Ground truth at D-230:** develop=ff4b82b, main=0cbe922 (v0.10.0), factory-artifacts=this commit.
