---
id: STORY-147
title: "Harden Mutation-Testing Defaults: mutants.toml Low-Parallelism + CLAUDE.md Guidance"
epic: E-11
wave: "~"
points: 3
status: draft
depends_on: []
input-hash: TBD
inputs: []
---

# STORY-147 — Harden Mutation-Testing Defaults: mutants.toml Low-Parallelism + CLAUDE.md Guidance

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 3

## Background

During fix-tls-clienthello-frag Phase F6, `cargo mutants --jobs 8` was used to
validate mutation coverage on the TLS reassembly suite. The run reported "0 missed
mutants", which appeared clean. However, two real surviving mutants at tls.rs:950:59
and tls.rs:1030:67 were hidden: infinite-loop mutants pegged all 8 cores, inflating
other mutants' wall-clock past the auto-timeout threshold and producing false timeouts
instead of real coverage signals.

Only a subsequent `--jobs 1` re-run surfaced the actual survivors. Thirteen real
mutation gaps were then closed by `mod f6_hardening`; two provably-equivalent
survivors were documented and retained.

Root cause and full narrative: `.factory/cycles/fix-tls-clienthello-frag/burst-log.md`
and STATE.md open item PG-MUTANTS-JOBS-001 (D-314, 2026-07-01).

## Goal

Encode lesson PG-MUTANTS-JOBS-001 into the repository so that mutation runs are
reliable by default and future cycles do not silently drop real survivors under
load-induced timeouts. Two concrete deliverables:

1. **`mutants.toml` at the repo root** (or a `[package.metadata.mutants]` section in
   `Cargo.toml`) that sets a low default job count (e.g., `jobs = 1`) so that
   `cargo mutants` invoked without any `--jobs` flag is safe by default.

2. **A "Mutation testing" note in `CLAUDE.md`** documenting:
   - This suite must run at low `--jobs` (recommended: 1) or with a high enough
     `--timeout` to prevent load-induced false timeouts.
   - Why: infinite-loop mutants peg all cores, inflating other mutants' wall-clock
     past the auto-timeout threshold and producing a false "0 missed" result.
   - The process-gap that motivated this guidance (PG-MUTANTS-JOBS-001,
     fix-tls-clienthello-frag F6, 2026-07-01).

## Acceptance Criteria

AC-147-001: A `mutants.toml` file exists at the repo root (or a
  `[package.metadata.mutants]` table exists in `Cargo.toml`) that sets a low default
  job count (≤ 2) or a generous per-mutant timeout sufficient to prevent load-induced
  false timeouts on a standard developer machine (e.g., `jobs = 1`).

AC-147-002: Running `cargo mutants` without any `--jobs` flag on this codebase
  uses the configured low-parallelism default — verified by inspecting the config
  file or a `cargo mutants --list-mutants` dry-run confirming the configured value
  is active.

AC-147-003: `CLAUDE.md` contains a "Mutation testing" note (within "Build & Test"
  or as a dedicated subsection) that:
  (a) states the recommended invocation (`--jobs 1` or equivalent `--timeout`
      increase),
  (b) explains why high `--jobs` is unsafe on this suite (infinite-loop mutants
      inflate wall-clock past auto-timeout → false "0 missed"),
  (c) references PG-MUTANTS-JOBS-001 and the fix-tls-clienthello-frag F6 cycle.

AC-147-004: A self-audit confirms that after this story ships, a developer running
  `cargo mutants` from a fresh checkout will not silently receive a false-clean
  result due to load-induced timeouts (i.e., the config-file default is the first
  line of defense; the CLAUDE.md note is the second).

## Notes

- This is a configuration and documentation story. The `mutants.toml` addition is
  ≤ 5 lines; the `CLAUDE.md` note is ≤ 10 lines. No Rust source changes required.
- Wave assignment is TBD — schedule at v0.12.0 planning along with STORY-091,
  STORY-121, and STORY-143 (all E-11, wave-TBD tooling stories).
- Source process-gap: PG-MUTANTS-JOBS-001 (STATE.md open items, D-314, 2026-07-01),
  cycle fix-tls-clienthello-frag F6.
- Precedent: STORY-143 (release-changelog enumeration hardening, D-301, 2026-06-29)
  — same E-11 pattern: a cycle process-gap follow-up encoding a lesson into project
  tooling/docs.
- S-7.02 disposition: this story's creation at draft status closes the
  PG-MUTANTS-JOBS-001 open item in STATE.md for S-7.02 cycle-close purposes.
