---
pipeline: PHASE_1_IN_PROGRESS
phase: phase-1-spec-crystallization
product: wirerust
mode: brownfield
timestamp: 2026-05-20T00:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
remediation_completed: 2026-05-19T22:30:00Z
phase_1_started: 2026-05-20T00:00:00Z
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_1_IN_PROGRESS — Phase 1 (Spec Crystallization) has begun.
The entire Phase C lesson backlog (30 lessons: 5 P0, 7 P1, 11 P2, 7 P3) was
delivered and merged to `develop` prior to Phase 1 entry. No lessons remain open.

**Current develop HEAD:** 0082a0c (merged PR #99 — `CLAUDE.md` governance pointer
for DF-VALIDATION-001).

**Mode:** brownfield (in-repo: target == reference).

**Test suite:** 213 (Phase 0 baseline) → **282** passing. `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
`cargo audit`, and `cargo deny` are all green on `develop`.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | Completed 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons closed; 21 PRs (#69–#89 + #90–#98) |
| Phase 1 — Spec Crystallization | IN PROGRESS | L2 domain spec produced; L3 PRD, BCs, arch, DTU not started |
| Phase 2 — Story Decomposition | NOT STARTED | — |
| Phase 3 — TDD Implementation | NOT STARTED | — |
| Phase 4 — Holdout Evaluation | NOT STARTED | — |
| Phase 5 — Adversarial Refinement | NOT STARTED | — |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 1 — Spec Crystallization (IN PROGRESS)

### Completed steps

- **L2 Domain Specification produced** — 19 sharded docs under
  `.factory/specs/domain/`:
  - 1 index: `domain-spec.md`
  - 11 capability shards
  - 5 entity shards
  - 1 invariants shard: `inv-01-core-invariants.md`
  - 1 `domain-debt.md`
  Reconciled against `develop` HEAD 0082a0c. 14 stale domain-debt items
  retired; 6 genuinely-open items remain (O-01..O-06; see Deferred-Findings
  Triage section below).

### Remaining steps (not started)

- L3 PRD + behavioral contracts
- Architecture / ADR documentation
- DTU assessment
- Adversarial spec convergence

## Governance Policy

**DF-VALIDATION-001** (commit 9b6efd3, `.factory/policies.yaml`): every
deferred/open finding must be research-agent validated before being filed as a
GitHub issue. Pointer added to `CLAUDE.md` on `develop` via PR #99 (0082a0c).

## Deferred-Findings Triage (2026-05-20)

10 candidate findings compiled (STATE.md drift items + 6 reconciliation open
items O-01..O-06 + 2 stale worktree branches) and validated per DF-VALIDATION-001.
Result: **5 filed as issues, 5 dropped**.

### Filed as GitHub issues

| Issue | Finding | Source |
|-------|---------|--------|
| #100 | `Finding.timestamp` always None; thread pcap timestamps | O-01 |
| #101 | Empirically characterize anomaly-threshold FP rates | drift item 1 / lesson P2.05 / O-03 |
| #102 | Cap weak-cipher ClientHello evidence Vec, CWE-405 hardening | NFR RES-023 / O-06 |
| #103 | Bidirectional size-symmetry discriminator for small-segment detector | drift item 2 / directional-symmetry follow-up |
| #104 | Surface control bytes in non-ASCII SNI summary, BC-TLS-037 | reconciliation finding DF-H |

### Dropped after research-agent validation (NOT filed)

| Candidate | Reason |
|-----------|--------|
| HTTP Host/User-Agent detection asymmetry (O-02) | RECLASSIFIED — asymmetry is intentional; minor docs-surface note folded into future docs work |
| 9 catalogued-but-unemitted MITRE technique IDs (O-04) | INVALID — documented intentional "staged entries" design in `src/mitre.rs` |
| `reassembly/mod.rs` ~691 LOC (O-05) | INVALID — 691 LOC is not a maintainability defect; module already decomposed |
| Branch `worktree-http-parse-errors` (8 commits) | RESOLVED/obsolete — HTTP `parse_errors` counter + TooManyHeaders finding already on `develop` |
| Branch `worktree-reassembly-encapsulation` (6 commits) | RESOLVED/obsolete — `max_receive_window` + `close_flow` refactor already on `develop` |

## Drift Items

1. **P2.05 calibration** — filed as **issue #101**. Thresholds are configurable
   and honestly documented; true calibration needs a labelled capture corpus.
2. **small-segment directional-symmetry discriminator** — filed as **issue #103**.
   Bidirectional traffic pattern would let the port-exemption list become advisory
   rather than load-bearing. Not yet implemented.
3. **P3.06** — CLOSED by #98. Branch-naming patterns widened; `CLAUDE.md` now
   version-controlled. All 30 P0–P3 lessons delivered.
4. **`nfs_bad_stalls.cap`** — CLOSED by #90. Snaplen-truncation regression fixture.
5. **Decoder snaplen-truncation** — CLOSED by #91/#94/#95. Strict-first with
   `SliceError::Len` fallback. 2376 → 7032 decoded TCP packets.

### Obsolete worktree branches (housekeeping, not yet deleted)

- `worktree-http-parse-errors` — verified obsolete (work already on `develop`)
- `worktree-reassembly-encapsulation` — verified obsolete (work already on `develop`)

### Adversarial-convergence process-gaps (sub-cycle #90–#95) — all CLOSED by #96

6. Sane-range validation on reassembly CLI flags — CLOSED #96
7. Threshold-boundary test pattern — CLOSED #96
8. etherparse version coupling — CLOSED #96

## Remediation Cycle — 21 PRs (#69–#98) — COMPLETE

### P0 — Correctness gaps — 5/5 CLOSED

| Lesson | Fix | PR |
|--------|-----|----|
| P0.01 | Declare MSRV `rust-version = "1.91"` | #69 |
| P0.02 | Remove `*.pcapng` from directory glob | #69 |
| P0.03 | `impl Drop` lifecycle tripwire + `run_analyze` IIFE | #72 |
| P0.04 | Wire `--json <FILE>`; loud-bail on `--csv` (superseded by P2.03) | #70 |
| P0.05 | Empty-value `Host:` evasion closed; UA asymmetry preserved | #71 |

### P1 — High-ROI improvements — 7/7 CLOSED

| Lesson | Fix | PR |
|--------|-----|----|
| P1.01 | `dropped_findings` counter on `ReassemblyStats` | #73 |
| P1.02 | Symmetric `Option` JSON serialization on `Finding` | #73 |
| P1.03 | `--hosts` flag wired to per-host terminal breakdown | #74 |
| P1.04 | "No unwired CLI flags" convention; 5 dead flags removed | #74 |
| P1.05 | `truncated_records` counter on `TlsAnalyzer` | #73 |
| P1.06 | `#![warn(missing_docs)]` phased rollout | #75 |
| P1.07 | `//!` module headers on all 20 modules | #75 |

### P2 — Worth considering — 11/11 CLOSED

| Lesson | Fix | PR |
|--------|-----|----|
| P2.01 | `reassembly/mod.rs` split into config/stats/lifecycle | #85 |
| P2.02 | Inlined format args + `clippy::uninlined_format_args` lint | #78 |
| P2.03 | CSV reporter implemented (CSV-injection neutralization) | #84 |
| P2.04 | JA3/JA3S property tests via `proptest` | #81 |
| P2.05 | Anomaly thresholds as `ReassemblyConfig` fields + CLI flags | #88 |
| P2.06 | `cargo audit` + `cargo deny` CI jobs + `deny.toml` | #79 |
| P2.07 | Criterion micro-benchmarks for hot paths | #83 |
| P2.08 | `direction` tag on `Finding` | #77 |
| P2.09 | Deterministic (BTreeMap) JSON map ordering | #76 |
| P2.10 | `#[non_exhaustive]` on `ThreatCategory`/`Verdict`/`Confidence` | #76 |
| P2.11 | `max_classification_attempts` knob on `StreamDispatcher` | #80 |

### P3 — Documentation tier — 7/7 CLOSED

ADR 0004/P3.03 (#89), MITRE staged-techniques P3.04 (#89),
test-naming P3.05 (#89), `is_grease_u16` rationale, dead-fixtures README
P3.07 (#86), `Summary.services` divergence doc P3.01 + pluralization helper
P3.02 (#97), branch-naming / `CLAUDE.md` P3.06 (#98).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from
  `develop`. `.factory/logs/` is gitignored.
- `demo-evidence/` added to the repo `.gitignore` (#87).
- NFRs OBS-010 (JSON asymmetry), RES-022 (dropped_findings) addressed by P1.02/P1.01;
  RES-023 weak-cipher heap bound now tracked as issue #102.
- Phase 0 canonical ground truth: `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`.
