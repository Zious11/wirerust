---
document_type: story
id: STORY-147
title: "Repo-Local Mutation-Testing Defaults: mutants.toml (jobs=1) + CLAUDE.md Guidance"
epic: E-11
wave: "84"
points: 2
status: ready
version: "2.1"
# BC status: E-11 convention — governance/config-only story; no BCs authored.
depends_on: []
input-hash: d41d8cd
inputs: []
---

# STORY-147 — Repo-Local Mutation-Testing Defaults: mutants.toml (jobs=1) + CLAUDE.md Guidance

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** ready
**Wave:** 84
**Points:** 2

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
  (c) references PG-MUTANTS-JOBS-001 and the fix-tls-clienthello-frag F6 cycle,
  (d) references drbothen/vsdd-factory#654 as the upstream engine-default tracking
      issue (informational pointer only — no wirerust action required for the
      mutation-testing skill's own default; see Disposition).

AC-147-004: A self-audit confirms that after this story ships, a developer running
  `cargo mutants` from a fresh checkout will not silently receive a false-clean
  result due to load-induced timeouts (i.e., the repo-root config-file default is
  the first line of defense; the CLAUDE.md note is the second). This self-audit is
  wirerust-local only — it does not depend on or require any change to the
  mutation-testing skill / formal-verifier agent default (engine-level, tracked
  separately per Disposition).

## Notes

- This is a configuration and documentation story. The `mutants.toml` addition is
  ≤ 5 lines; the `CLAUDE.md` note is ≤ 10 lines. No Rust source changes required.
- Wave 84 (opened 2026-07-19, plan gate approved by human): STORY-147 v2.0 +
  STORY-166 + STORY-176, 7 pts total, all product-local.
- Source process-gap: PG-MUTANTS-JOBS-001 (STATE.md open items, D-314, 2026-07-01),
  cycle fix-tls-clienthello-frag F6.
- Precedent: STORY-143 (release-changelog enumeration hardening, D-301, 2026-06-29,
  now superseded — routed upstream per drbothen/vsdd-factory#695) — same E-11
  pattern: a cycle process-gap follow-up encoding a lesson into project tooling/docs.
- S-7.02 disposition: this story's creation at draft status closed the
  PG-MUTANTS-JOBS-001 open item in STATE.md for S-7.02 cycle-close purposes; the
  v2.0 re-scope (2026-07-19) retains that closure for the product-local half only.

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~1.0 k |
| `mutants.toml` (new file, <=5 lines) | ~0.1 k |
| `CLAUDE.md` (Build & Test section context, amendment target) | ~0.5 k |
| **Total** | **~1.6 k** |

Well within context window. No story split required.

## Disposition

**Status:** ready (v2.0) — SPLIT disposition; product half retained locally,
engine half routed upstream 2026-07-19.

The human-approved E-11 stale-draft disposition plan
(`.factory/planning/e11-stale-draft-disposition-plan.md`) confirmed via a
delivered-by-drift check on the current tree (no `mutants.toml`, no
`[package.metadata.mutants]` table, no "Mutation testing" note in `CLAUDE.md`) that
the product half of this story is genuinely undelivered, and split the story:

| Half | Disposition |
|------|-------------|
| Product (RETAIN LOCALLY, v2.0, this story — wirerust repo files only) | `mutants.toml` (jobs=1) at wirerust repo root + `CLAUDE.md` "Mutation testing" note + self-audit (AC-147-001..004). Points re-scoped 3→2 (engine-skill-default work removed from scope). |
| Engine (mutation-testing skill safe-parallelism default, all VSDD projects) | Routed upstream via drbothen/vsdd-factory#654 evidence comment (posted 2026-07-19): confirming field data — `cargo mutants --jobs 8` reported false "0 missed", hiding two real survivors at tls.rs:950:59/tls.rs:1030:67, surfaced only by a `--jobs 1` re-run (plus eleven more real gaps subsequently closed). |

This story delivers the product half only. No further wirerust delivery expected for
the engine half.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.1 | 2026-07-19 | story-writer | Remediation: added missing "Token Budget Estimate" section (per-story-delivery.md Token Budget Check). No AC or scope content change. |
| 2.0 | 2026-07-19 | story-writer | SPLIT re-scope (human-approved E-11 stale-draft disposition plan): retitled to "Repo-Local Mutation-Testing Defaults" to reflect wirerust-local-only scope; points 3→2 (engine-skill-default work removed); AC-147-003(d) + AC-147-004 clarified as product-local-only; engine half (mutation-skill safe-parallelism default) routed upstream via drbothen/vsdd-factory#654 evidence comment. Wave TBD→84, status draft→ready (plan gate approved by human, mini-wave 166+176+147v2 = 7 pts). |
| 1.0 | 2026-07-08 | state-manager | Added `document_type: story` and `input-hash: d41d8cd` for scanner compatibility (STORY-157 TASK F; `inputs: []` → canonical empty-inputs hash). |
