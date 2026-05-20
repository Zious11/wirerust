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

**Test suite:** 213 (Phase 0 baseline) → **282** passing. `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
`cargo audit`, and `cargo deny` are all green on `develop`.

**Snaplen / small-segment sub-cycle (#90–#95): CONVERGED.** The
snaplen-truncation + small-segment-detector work was taken through a
3-pass fresh-context adversarial review (vsdd-factory:adversary):
pass 1 found a real decoder regression (lax recovery applied to
malformed packets) → fixed in #94; pass 2 found test-quality gaps
(vacuous assertions) → fixed in #95; pass 3 returned `CONVERGED` with
no CRITICAL/HIGH/MEDIUM findings.

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

DONE: ADR 0004 / P3.03 (#89), MITRE staged-techniques P3.04 (#89),
test-naming convention P3.05 (#89, in README), `is_grease_u16` rationale,
dead-fixtures README P3.07 (#86), `Summary.services` divergence doc P3.01
and pluralization helper P3.02 (#97).

REMAINING: P3.06 (widen branch-naming patterns to `<type>/<slug>`) —
blocked: it targets `CLAUDE.md`, which is currently untracked in the
repo. See drift item 3.

### Non-lesson PRs

- **#86** — added 2 Wireshark-wiki TCP reassembly fixtures (benign baselines
  toward P2.05) + `tests/fixtures/README.md`.
- **#87** — `fix(reader)`: accept snaplen-truncated captures (`tcpdump -s`).
  A genuine reader bug discovered while adding fixtures; pcap-file 2.0.0's
  validated path wrongly rejects `orig_len > snap_len`.
- **#82** — chore: commit `Cargo.lock` + proptest regressions for P2.04.
- **#90** — `test`: re-add `nfs_bad_stalls.cap` as a snaplen-truncation
  regression fixture (drift item 4 below, now closed) + 2 tests.
- **#91** — `fix(decoder)`: lax-parse snaplen-truncated IP packets
  (drift item 5 below, now closed). 7037/7038 packets of
  `nfs_bad_stalls.cap` now decode (was 2376) + 4 tests.
- **#92** — `fix(reassembly)`: small-segment detector redesigned to
  consecutive-run counting (drift item 2 below). Count default
  2048→100, size cutoff 8→16 + configurable, +2 tests.
- **#93** — `feat(reassembly)`: interactive-port exemption for the
  small-segment detector (`small_segment_ignore_ports`, default
  `[23, 513]`; `--small-segment-ignore-ports` flag) — closes the #92
  per-port follow-up. +1 test.
- **#94** — `fix(decoder)`: restrict the strict→lax fallback to
  `SliceError::Len` (truncation) so structurally-malformed packets stay
  rejected — adversarial-review pass-1 fix (H1/H2 + Mediums).
- **#95** — `test`: close adversarial-review pass-2 test-quality gaps
  (load-bearing boundary/reset tests, physical-buffer truncation test,
  IPv6 truncation coverage, exact decode-count pin). Docs/tests only.
- **#96** — `fix(cli)`: enforce sane ranges on the reassembly threshold
  flags + threshold-boundary tests for overlap/out-of-window + etherparse
  version-coupling docs — closes process-gap deferrals 6, 7, 8 below.
- **#97** — `chore`: P3.01 (`Summary.services` port-vs-content
  divergence doc) + P3.02 (pluralization helper extraction). 6/7 P3
  lessons now done; only P3.06 remains (see drift item 3).

## Drift Items / open follow-ups

1. **P2.05 not empirically calibrated.** Research established no NIDS exposes
   a comparable count-and-alert-at-N threshold; the lesson was closed as
   *configurable + honestly documented*. The overlap and out-of-window
   thresholds remain conservative engineering defaults. True calibration
   needs a labelled capture corpus (benign + adversarial) measured for
   FP/TP rates.
2. **small-segment detector** — REDESIGNED by #92 + #93. The
   cumulative-2048 design (a dead detector) is now consecutive-run
   counting (default 100), a configurable `< 16 byte` cutoff, and an
   interactive-port exemption (`[23, 513]` default). New follow-up: a
   port-independent **directional-symmetry discriminator** — benign
   interactive traffic is bidirectionally tiny (keystroke echo) while
   segmentation evasion is a one-directional burst. Research flagged
   this as a sound but not-yet-validated design proposal; it would let
   the port list become advisory rather than load-bearing. Not
   implemented.
3. **P3.06 — widen branch-naming patterns** (`<type>/<slug>`) in
   `CLAUDE.md`. The other six P3 lessons are done (P3.01/P3.02 by #97);
   P3.06 is the last open lesson of the entire P0–P3 backlog. It is
   blocked on a user decision: `CLAUDE.md` is untracked (not gitignored
   — just never `git add`ed), so the edit cannot land in version
   control. Either commit `CLAUDE.md` to the repo (then apply P3.06) or
   accept it as a local-only guidance file.
4. **`nfs_bad_stalls.cap`** — CLOSED by #90. Re-added as a
   snaplen-truncation regression fixture (not a reassembly baseline).
5. **Decoder snaplen-truncation** — CLOSED by #91, hardened by #94/#95.
   The decoder parses strict-first and falls back to
   `etherparse::LaxSlicedPacket` only on `SliceError::Len`.
   `nfs_bad_stalls.cap` went from 2376 → 7032 decoded TCP packets.

### Adversarial-convergence process-gaps (sub-cycle #90–#95)

The 3-pass adversarial review tagged three process-gaps — all CLOSED by
#96:

6. **Sane-range validation on reassembly CLI flags** — CLOSED by #96.
   `--overlap-threshold` (0–255), `--small-segment-threshold` (0–2048)
   and `--small-segment-max-bytes` (0–2048) now reject out-of-range
   values at parse time.
7. **Threshold-boundary test pattern** — CLOSED by #96. Exactly-threshold
   negative tests now exist for the overlap and out-of-window detectors,
   not just the small-segment one.
8. **etherparse version coupling** — CLOSED by #96. The `SliceError::Len`
   coupling is documented in `Cargo.toml` and `src/decoder.rs`; the
   `"0.16"` constraint already excludes 0.17, and the IPv6 / corrupt-packet
   tests are the contract tests for a future bump.

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from
  `develop`. `.factory/logs/` is gitignored.
- `demo-evidence/` added to the repo `.gitignore` (#87).
- Architecture smell #9 (no-Drop / finalize-fragile) closed by P0.03.
- NFRs OBS-010 (JSON asymmetry), RES-022 (dropped_findings) addressed by P1.02
  / P1.01; RES-023 weak-cipher heap bound remains as catalogued.
- The pcap-file `orig_len > snap_len` bug (#87) is worth an upstream report.
