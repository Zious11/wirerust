---
pipeline: REMEDIATION_COMPLETE
phase: lesson-backlog-remediation-complete
product: wirerust
mode: brownfield
timestamp: 2026-05-19T22:30:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
remediation_completed: 2026-05-19T22:30:00Z
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** REMEDIATION_COMPLETE — the full Phase C lesson backlog (P0 + P1 +
P2) has been delivered as 21 merged PRs on `develop`, plus the high-value P3
documentation subset. Three cosmetic P3 items deliberately deferred.

**Mode:** brownfield (in-repo: target == reference).

**Test suite:** 213 (Phase 0 baseline) → **265** passing. `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
`cargo audit`, and `cargo deny` are all green on `develop`.

## Phase 0 Ingestion Summary (historical)

Brownfield ingestion completed 2026-05-19T20:00:00Z. All 6 deepening passes
converged; Phase B.5 coverage audit PASS; Phase B.6 extraction validation PASS
(18/20 CONFIRMED BCs sampled, 0 HALLUCINATED). Canonical ground truth:
`.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md` (Phase C). The 21
ingestion artifacts under `.factory/semport/wirerust/` remain the reference
corpus; the Phase 0 metric table and convergence record are preserved there.

## Remediation Cycle — 21 PRs (#69–#89)

### P0 — Correctness gaps — 5/5 CLOSED

| Lesson | Fix | PR |
|--------|-----|-----|
| P0.01 | Declare MSRV `rust-version = "1.91"` (clippy `incompatible_msrv` corrected the date-inferred 1.86) | #69 |
| P0.02 | Remove `*.pcapng` from the directory glob (reader rejects it) | #69 |
| P0.03 | `impl Drop` lifecycle tripwire + `run_analyze` IIFE so `finalize()` always runs | #72 |
| P0.04 | Wire `--json <FILE>` to `fs::write`; loud-bail on `--csv` (later superseded by P2.03) | #70 |
| P0.05 | Empty-value `Host:` evasion closed; UA asymmetry preserved with research-cited rationale | #71 |

### P1 — High-ROI improvements — 7/7 CLOSED

| Lesson | Fix | PR |
|--------|-----|-----|
| P1.01 | `dropped_findings` counter on `ReassemblyStats` | #73 |
| P1.02 | Symmetric `Option` JSON serialization on `Finding` | #73 |
| P1.03 | `--hosts` flag wired to a per-host terminal breakdown | #74 |
| P1.04 | "No unwired CLI flags" convention; 5 dead flags removed | #74 |
| P1.05 | `truncated_records` counter on `TlsAnalyzer` | #73 |
| P1.06 | `#![warn(missing_docs)]` phased rollout | #75 |
| P1.07 | `//!` module headers on all 20 modules | #75 |

### P2 — Worth considering — 11/11 CLOSED

| Lesson | Fix | PR |
|--------|-----|-----|
| P2.01 | `reassembly/mod.rs` split into config/stats/lifecycle + `process_packet` decomposed | #85 |
| P2.02 | Inlined format args + `clippy::uninlined_format_args` enforcement lint | #78 |
| P2.03 | CSV reporter implemented (with CSV-injection neutralization); loud-bail retired | #84 |
| P2.04 | JA3/JA3S property tests via `proptest` (+ #82 lockfile/regressions follow-up) | #81 |
| P2.05 | Anomaly thresholds made `ReassemblyConfig` fields + CLI flags, research-documented | #88 |
| P2.06 | `cargo audit` + `cargo deny` CI jobs + `deny.toml` | #79 |
| P2.07 | Criterion micro-benchmarks for the hot paths | #83 |
| P2.08 | `direction` tag on `Finding` | #77 |
| P2.09 | Deterministic (BTreeMap) JSON map ordering | #76 |
| P2.10 | `#[non_exhaustive]` on `ThreatCategory` / `Verdict` / `Confidence` | #76 |
| P2.11 | `max_classification_attempts` knob on `StreamDispatcher` | #80 |

### P3 — Documentation tier — curated subset done (#89)

DONE: ADR 0004 (process-wide warning atomics), MITRE staged-techniques module
note, test-naming convention in README, `is_grease_u16` rationale comment,
`tests/fixtures/README.md` provenance doc (#86).

DEFERRED (cosmetic, per agreed curation): pluralization-helper extraction,
`<type>/<slug>` branch-naming doc widening, services-taxonomy split doc.

### Non-lesson PRs

- **#86** — added 2 Wireshark-wiki TCP reassembly fixtures (benign baselines
  toward P2.05) + `tests/fixtures/README.md`.
- **#87** — `fix(reader)`: accept snaplen-truncated captures (`tcpdump -s`).
  A genuine reader bug discovered while adding fixtures; pcap-file 2.0.0's
  validated path wrongly rejects `orig_len > snap_len`.
- **#82** — chore: commit `Cargo.lock` + proptest regressions for P2.04.

## Drift Items / open follow-ups

1. **P2.05 not empirically calibrated.** Research established no NIDS exposes
   a comparable count-and-alert-at-N threshold; the lesson was closed as
   *configurable + honestly documented*. True calibration needs a labelled
   capture corpus (benign + adversarial) measured for FP/TP rates.
2. **small-segment default `2048`** — research flags it as likely Snort's knob
   *ceiling* mistaken for a default (near-inert). Kept unchanged (an 8× cut
   without FP data is another guess); operators can lower it via
   `--small-segment-threshold`. Candidate for a data-backed default change.
3. **3 deferred P3 cosmetic items** (see above).
4. **`nfs_bad_stalls.cap`** could now be re-added as a benign reassembly
   fixture — #87 made snaplen-truncated captures readable.

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from
  `develop`. `.factory/logs/` is gitignored.
- `demo-evidence/` added to the repo `.gitignore` (#87).
- Architecture smell #9 (no-Drop / finalize-fragile) closed by P0.03.
- NFRs OBS-010 (JSON asymmetry), RES-022 (dropped_findings) addressed by P1.02
  / P1.01; RES-023 weak-cipher heap bound remains as catalogued.
- The pcap-file `orig_len > snap_len` bug (#87) is worth an upstream report.
