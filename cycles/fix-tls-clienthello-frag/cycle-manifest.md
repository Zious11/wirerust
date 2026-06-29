---
document_type: cycle-manifest
cycle_id: fix-tls-clienthello-frag
cycle_type: feature
version: DEFERRED
status: in-progress
started: 2026-06-29T15:00:00Z
completed: ~
producer: orchestrator
---

# Cycle Manifest: fix-tls-clienthello-frag (Feature — Security-Correctness Fix)

## Finding

| Field | Value |
|-------|-------|
| Finding ID | TLS-CLIENTHELLO-FRAG-001 |
| Severity | HIGH (research-agent downgrade from CRIT candidate; see validation) |
| Validation status | CONFIRMED — DF-VALIDATION-001 SATISFIED |
| Validation artifact | `.factory/research/TLS-CLIENTHELLO-FRAG-001-validation.md` |
| Nature | Silent SNI/JA3 evasion via fragmented TLS ClientHello across records |
| Source location | `src/analyzer/tls.rs` ~763–792 |
| Severity rationale | Passive analyzer; no RCE / DoS / auth dimension. Impact = silent evasion of SNI extraction and JA3 fingerprinting when ClientHello is fragmented across TLS records. Downgraded from CRIT candidate to HIGH after research-agent confirmed no active exploit path in a passive PCAP analyzer. |

## Scope

Add bounded per-direction TLS handshake-message reassembly (content-type 0x16) across
records in `src/analyzer/tls.rs`. Reassembly must:

- Buffer incomplete handshake messages across record boundaries within a flow direction.
- Bound carry buffer to prevent unbounded memory growth (carry cap to be determined at F1/F2).
- Preserve truncation semantics: snaplen-truncated captures must NOT inflate
  `parse_errors` or produce false-positive SNI/JA3 findings. (Interacts with
  READER cand-05 from EDGE-CASE-HUNT-REGISTER-2026-06-28.)
- Not alter behavior for already-complete single-record ClientHellos.

## Release Version

DEFERRED — human decision at F7 convergence. Candidates: v0.11.1 (patch) or
bundled into v0.12.0. Both options are open; no version committed yet.

## Develop HEAD at Cycle Start

`a2d8c13ff9e23f49d5ab93ab6453da4442658bcc`

## Pipeline Process

Full F1–F7 VSDD Feature-Mode process. Selected by human (2026-06-29, D-303).
Maintenance sweeps PAUSED for duration of this cycle.

## Phase Status

| Phase | Status | Notes |
|-------|--------|-------|
| F1 — Delta Analysis | DONE | delta-analysis.md committed; architect completed |
| F2 — Spec Evolution | **CONVERGED 2026-06-29 — AWAITING HUMAN GATE** | 5 new BCs (BC-2.07.038-042) + 2 amended + VP-039 (17 harnesses) + ADR-011; 12 fix bursts; 3+ clean adversary passes (consistency + implementer + completeness + holistic/attacker lenses). BC-INDEX v1.98, VP-INDEX v2.21, ARCH-INDEX v2.3, PRD v1.43. |
| F3 — Incremental Stories | PENDING | Awaiting human F2 approval |
| F4 — TDD Delta Implementation | PENDING | |
| F5 — Scoped Adversarial Review | PENDING | |
| F6 — Targeted Hardening | PENDING | |
| F7 — Delta Convergence | PENDING | Version decision at gate |

## Delivered

*Populated at cycle close (F7 gate).*

| Metric | Value |
|--------|-------|
| Stories delivered | TBD |
| BCs created | TBD |
| VPs created | TBD |
| Adversarial passes | TBD |
| Final holdout satisfaction | TBD |
| Release version | DEFERRED — decided at F7 |

## Spec Changes

*F2 delta (2026-06-29 — converged, awaiting human gate):*

| Artifact | Change | Version |
|----------|--------|---------|
| BC-2.07.038 | NEW — reassembly across records | v2.7 |
| BC-2.07.039 | NEW — bounded carry clear-and-recover overflow | v2.4 |
| BC-2.07.040 | NEW — truncation-safety | v1.3 |
| BC-2.07.041 | NEW — per-flow+per-direction isolation | v1.2 |
| BC-2.07.042 | NEW — coalesced dispatch | v1.4 |
| BC-2.07.001 | AMENDED — scope expansion to fragmented-then-assembled | v1.9 |
| BC-2.07.002 | AMENDED — scope expansion to fragmented-then-assembled | v1.6 |
| VP-039 | NEW — proptest+unit; 17 harnesses (4 proptest + 13 unit) | — |
| ADR-011 | NEW — TLS handshake reassembly design decisions | — |
| BC-INDEX | UPDATED | v1.98 |
| VP-INDEX | UPDATED (39 VPs total) | v2.21 |
| ARCH-INDEX | UPDATED | v2.3 |
| PRD | UPDATED | v1.43 |

## Tech Debt Created

*Populated during cycle as items are identified.*

## Notes

- Interact with READER cand-05 (snaplen truncation) from EDGE-CASE-HUNT-REGISTER-2026-06-28
  must be handled carefully: reassembly should fail-open (skip SNI extraction) rather than
  emit parse_errors on a truncated record that cannot be completed.
- Carry cap design is a key F1/F2 decision: must be documented in an ADR or BC note.
- SEC-001 (unsafe split-borrow in enip.rs) remains in backlog; not part of this cycle.
