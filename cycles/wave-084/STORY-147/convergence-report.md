---
document_type: per-story-convergence-report
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-07-19T23:55:00Z
phase: step-4.5-per-story-adversarial
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
story: STORY-147
cycle: wave-084
passes_total: 8
verdict: CONVERGED
criterion: BC-5.39.001
clean_streak: [P6, P7, P8]
final_head: 7ff84f56
base: "49255464"
story_version: "2.8"
---

# Convergence Report — STORY-147 (compact)

## Pipeline Run: 2026-07-19
## Product: wirerust — STORY-147 Repo-Local Mutation-Testing Defaults (E-11, wave-084)
## Iterations: 8

---

## Verdict: CONVERGED — BC-5.39.001 SATISFIED (3 consecutive clean passes: P6/P7/P8)

## Trajectory

`2H+3M -> 1M+1L -> NIT(3L) -> 2M -> 1M+1L -> NIT(2L) -> NIT(1L) -> NIT(1L)`

| Pass | Verdict | HIGH | MED | LOW | Code Tip | Part A (prior-pass fix verification) |
|------|---------|------|-----|-----|----------|----------------------------------------|
| P1 | FAIL_FINDINGS | 2 | 3 | 0 | d466f538 | — (first pass) |
| P2 | FAIL_FINDINGS | 0 | 1 | 1 | 2c802e73 | 5/5 VERIFIED-FIXED |
| P3 | NITPICK_ONLY | 0 | 0 | 3 | b1b50750 | 2/2 VERIFIED-FIXED |
| P4 | FAIL_FINDINGS | 0 | 2 | 0 | e198a725 | 3/3 VERIFIED-FIXED |
| P5 | FAIL_FINDINGS | 0 | 1 | 1 | 8ba2247b | 2/2 VERIFIED-FIXED (one as-scoped) |
| P6 | NITPICK_ONLY | 0 | 0 | 2 | 7ff84f56 | 2/2 VERIFIED-FIXED — streak 1/3 |
| P7 | NITPICK_ONLY | 0 | 0 | 1 | 7ff84f56 (unchanged) | 2/2 CONFIRMED — streak 2/3 |
| P8 | NITPICK_ONLY | 0 | 0 | 1 | 7ff84f56 (unchanged) | 1/1 RESOLVED-CONFIRMED — streak 3/3, CONVERGED |

---

## Headline Narrative

Pass-1 finding **F-S147P1-002** (HIGH, corroborated by F-S147P1-004/-005) caught a
**placebo config**: the v2.1 story specified a repo-root `mutants.toml` with a
`jobs = 1` key as the deliverable. Execution probes against the installed
cargo-mutants 27.0.0 plus 27.1.0 docs/source research established that
cargo-mutants **never reads** a repo-root `mutants.toml` (only `.cargo/mutants.toml`
is read by default), and `jobs` **is not a valid `Config` field** — the parser is
`deny_unknown_fields`, so shipping that file would abort every mutation run with a
fatal parse error rather than setting a safe default. The design pivoted to a
**`.cargo/mutants.toml` timeout floor** (`minimum_test_timeout=300`, optional
`timeout_multiplier`) — the config-file-expressible defense against the actual
PG-MUTANTS-JOBS-001 failure mode (load-induced false timeouts), since `jobs` itself
is CLI-only and no config file can override an explicit `--jobs` flag.

Passes 2-5 closed out residual documentation/key-allowlist/anti-drift findings, all
adversary-verified fixed before the next pass opened. Passes 6-8 held code tip
`7ff84f56` fixed with zero code churn, closing on three consecutive NITPICK_ONLY
passes (BC-5.39.001's 3-clean-streak criterion, satisfied by nitpick-only passes per
established convention — see STORY-161 precedent).

Spec evolved **v2.1 -> v2.8** across the 8 passes; the title itself changed at v2.2
("Repo-Local Mutation-Testing Defaults: mutants.toml (jobs=1) + CLAUDE.md Guidance"
-> "...: .cargo/mutants.toml Timeout Floor + CLAUDE.md Guidance"), cascading a
title-only update into STORY-INDEX v3.79 -> v3.80 (no points/status/wave/epic
change).

## Non-Blocking Residual

- **F-S147P8-001** (LOW): scan-helper prose collapses `timeout_multiplier` and
  `build_timeout_multiplier` into one referenced field name. Documentation-only,
  unexercised by any test/runtime path. Carried for gate ratification, not a
  convergence blocker.

## Process Gaps Noted for Cycle Close

- Stale-inline-version-marker recurrence (3+ instances across this convergence;
  root cause of F-S147P4-001) — human directive: route the durable fix upstream,
  not as a local story amendment.
- PG-HASH-HOOK-DIVERGENCE reconfirmed — advisory-only hook noise (bash `$(cat)`
  newline-stripping vs. canonical Python `bin/compute-input-hash`) recurred across
  all 8 passes with no actionable content; consistent with CLAUDE.md's documented
  advisory-only treatment.

## Traceability

- Full pass-by-pass state: `adversary-convergence-state.json` (this directory)
- Red Gate log + title-update annotation: `implementation/red-gate-log.md`
- Story: `.factory/stories/STORY-147.md` (v2.8)
- STORY-INDEX: `.factory/stories/STORY-INDEX.md` (v3.80)
