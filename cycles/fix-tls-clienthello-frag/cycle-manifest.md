---
document_type: cycle-manifest
cycle_id: fix-tls-clienthello-frag
cycle_type: feature
version: v0.11.1
status: completed
started: 2026-06-29T15:00:00Z
completed: 2026-07-01T00:00:00Z
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

v0.11.1 — released 2026-07-01. PR #347 gitflow merge into main (`4e2b285`). Tag `v0.11.1` (object `e8a8a2d4e7cd03e337b066859586e2c610208888`). GitHub Release published (4 assets: aarch64/x86_64 darwin, x86_64 windows-msvc, x86_64 linux-gnu). Not published to crates.io (D-300 human decision). Back-merged to develop via PR #348 squash (`ba6fbd8`). Both branches at 0.11.1, in sync.

## Develop HEAD at Cycle Start

`a2d8c13ff9e23f49d5ab93ab6453da4442658bcc`

## Pipeline Process

Full F1–F7 VSDD Feature-Mode process. Selected by human (2026-06-29, D-303).
Maintenance sweeps PAUSED for duration of this cycle.

## Phase Status

| Phase | Status | Notes |
|-------|--------|-------|
| F1 — Delta Analysis | DONE | delta-analysis.md committed; architect completed |
| F2 — Spec Evolution | **CONVERGED + HUMAN-APPROVED 2026-06-29 (D-305, incl scope addition)** | 6 new BCs (BC-2.07.038-043) + 3 amended (BC-2.07.001 v1.9, BC-2.07.002 v1.6, BC-2.07.005 v1.7) + VP-039 (17 harnesses) + VP-040 (6 harnesses) + ADR-011. F-EV-001 defense-in-depth scope addition: BC-2.07.043 buffer_saturation_drops + BC-2.07.005 v1.7 reconciliation. BC-INDEX v2.1, VP-INDEX v2.25 (40 VPs), ARCH-INDEX v2.4, PRD v1.45. SS-07 43 BCs. |
| F3 — Incremental Stories | **APPROVED 2026-06-29 (D-306)** | STORY-144..146 authored; STORY-INDEX v3.6 (99 stories, 65 waves); holdout registry HS-F4-001..012; input-hashes refreshed (144: 3dfe20c, 145: 88e29c9, 146: 6d9da65); pre-F4 verification PASS |
| F4 — TDD Delta Implementation | **DONE** | All 3 stories MERGED. STORY-144 PR #341 squash `0986e878` (Wave 65). STORY-145 PR #343 squash `d3d2e19` (Wave 66). STORY-146 PR #344 squash `8b52046` (Wave 66). stories_delivered=94. |
| F5 — Scoped Adversarial Review | **DONE/CONVERGED** | 5 passes. BC-completeness 60/60. 0 P0. BC-2.07.038 v2.10. Re-anchor 7 BCs. BC-INDEX v2.3. |
| F6 — Targeted Hardening | **DONE** | Kani VP-039 3 non-vacuous proofs PASS. fuzz_tls_reassembly 1.9M execs 0 crashes. mod f6_hardening 12 tests — 100% real-gap kill (13 gaps; 2 provably-equiv dead-code survivors at tls.rs:950:59, ADR-011). anyhow 1.0.103 (RUSTSEC-2026-0190 cleared). PRs #345+#346. |
| F7 — Delta Convergence | **DONE/CONVERGED** | v0.11.1 released PR #347 into main (`4e2b285`). Back-merge PR #348 to develop (`ba6fbd8`). S-7.02 SATISFIED. Cycle CLOSED (D-316). |

## Delivered

| Metric | Value |
|--------|-------|
| Stories delivered | 3 (STORY-144, STORY-145, STORY-146) |
| BCs created | 6 new (BC-2.07.038–043) + 3 amended (BC-2.07.001 v1.9, BC-2.07.002 v1.6, BC-2.07.005 v1.7) |
| VPs created | 2 (VP-039: 17 harnesses; VP-040: 6 harnesses) |
| Adversarial passes | F5: 5 passes + per-story convergence (STORY-145: 5 passes; STORY-146: multi-pass) |
| Final holdout satisfaction | 0.904 mean / 8-of-8 must-pass (F4 gate) |
| Release version | v0.11.1 (released 2026-07-01; PR #347; tag `v0.11.1`) |

## Spec Changes

*F2 delta (2026-06-29 — converged, human-approved D-305, including scope addition):*

| Artifact | Change | Version |
|----------|--------|---------|
| BC-2.07.038 | NEW — reassembly across records | v2.7 |
| BC-2.07.039 | NEW — bounded carry clear-and-recover overflow | v2.4 |
| BC-2.07.040 | NEW — truncation-safety | v1.3 |
| BC-2.07.041 | NEW — per-flow+per-direction isolation | v1.2 |
| BC-2.07.042 | NEW — coalesced dispatch | v1.4 |
| BC-2.07.043 | NEW (scope addition D-305) — buffer_saturation_drops aggregate counter; TlsAnalyzer u64; incremented on on_data tail-drop; hoisted after &mut state block (borrow constraint); surfaced in summarize(); no finding/no parse_errors; test seam fill_buf_for_testing | — |
| BC-2.07.001 | AMENDED — scope expansion to fragmented-then-assembled | v1.9 |
| BC-2.07.002 | AMENDED — scope expansion to fragmented-then-assembled | v1.6 |
| BC-2.07.005 | AMENDED (scope addition D-305) — silent-truncation Inv-3 superseded; reconciled with BC-2.07.043 three-counter telemetry model | v1.7 |
| VP-039 | NEW — proptest+unit; 17 harnesses (4 proptest + 13 unit) | — |
| VP-040 | NEW (scope addition D-305) — buffer saturation observability; 6 harnesses | — |
| ADR-011 | NEW — TLS handshake reassembly design decisions | — |
| BC-INDEX | UPDATED (scope addition D-305; 337 on disk / 336 active) | v2.1 |
| VP-INDEX | UPDATED (40 VPs total) | v2.25 |
| ARCH-INDEX | UPDATED | v2.4 |
| PRD | UPDATED | v1.45 |

## Tech Debt Created

| ID | Description | Priority | Target |
|----|-------------|----------|--------|
| SEC-002 | Narrow non-RFC overflow window [MAX_BUF-3, MAX_BUF] — message body in this range clears-and-recovers rather than assembling. Low exploitability; primary DoS path fixed in STORY-144. | LOW | F6 hardening |
| SEC-004 | parse_errors plain `+=` in tls.rs — theoretical u64 overflow on extremely long-running high-parse-error flows (cosmetic counter wrap, no safety impact). | LOW | Maintenance sweep |
| DONE-MID-LOOP-CROSS-DIRECTION | done()-mid-loop cross-direction carry interaction — if done() fires while partial message spans both directions, cross-direction carry residue may be silently cleared. Pre-existing. | LOW | Wave-gate review |

## Notes

- Interact with READER cand-05 (snaplen truncation) from EDGE-CASE-HUNT-REGISTER-2026-06-28
  must be handled carefully: reassembly should fail-open (skip SNI extraction) rather than
  emit parse_errors on a truncated record that cannot be completed.
- Carry cap design is a key F1/F2 decision: must be documented in an ADR or BC note.
- SEC-001 (unsafe split-borrow in enip.rs) remains in backlog; not part of this cycle.
