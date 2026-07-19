---
document_type: story
id: STORY-147
title: "Repo-Local Mutation-Testing Defaults: .cargo/mutants.toml Timeout Floor + CLAUDE.md Guidance"
epic: E-11
wave: "84"
points: 2
status: ready
version: "2.8"
# BC status: E-11 convention — governance/config-only story; no BCs authored.
depends_on: []
input-hash: d41d8cd
inputs: []
---

# STORY-147 — Repo-Local Mutation-Testing Defaults: .cargo/mutants.toml Timeout Floor + CLAUDE.md Guidance

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

### Execution-evidence correction (2026-07-19, Pass-1 adversarial F-S147P1-002/-004/-005)

Step 4.5 adversarial findings F-S147P1-002, F-S147P1-004, and F-S147P1-005
(CONFIRMED by execution evidence — cargo-mutants 27.0.0 local probes — plus
27.1.0 docs/source research) established that the original v2.1 Goal/AC text
described a config surface that does not exist as specified. Ground truth:

1. cargo-mutants reads **only `.cargo/mutants.toml`** (source-tree root) as its
   default config file. It does **not** read a repo-root `mutants.toml` (silently
   ignored) and does **not** read a `[package.metadata.mutants]` section in
   `Cargo.toml`. (https://mutants.rs/config-file.html)
2. `jobs` is **not a valid config key** — the `Config` struct has no `jobs` field
   and is `#[serde(default, deny_unknown_fields)]`: a `jobs = 1` line in
   `.cargo/mutants.toml` raises a FATAL parse error and breaks the run.
   Parallelism is CLI-only (`--jobs`/`-j`, env `CARGO_MUTANTS_JOBS`).
   (cargo-mutants `src/config.rs`)
3. Bare `cargo mutants` is **already serial by default** (jobs=1-equivalent). The
   PG-MUTANTS-JOBS-001 incident came from an **explicit** `--jobs 8`.
   (https://mutants.rs/parallelism.html)
4. The CLI wins over the config file for scalar settings — no config file can
   override an explicit `--jobs 8`. (https://mutants.rs/config-file.html)
5. Valid, probe-verified config keys for the load-induced-false-timeout defense
   are `minimum_test_timeout` (e.g. `300`) and `build_timeout_multiplier` /
   `timeout_multiplier`. Note: `test_timeout_multiplier` is **not** a real key —
   the correct key is `timeout_multiplier`.

These facts supersede the v2.1 Goal/AC text below wherever they conflict. The
config-file defense is a **timeout floor**, not a parallelism default — no
config file can set a safe default `jobs` value because `jobs` is not a
config key at all.

Pass-3 probe result (2026-07-19): local strict-parser allowlist probes against
the installed cargo-mutants 27.0.0 confirmed `test_tool` as a valid `Config`
field and `common` as invalid; the AC-147-002 test allowlist was corrected
accordingly (-`common` +`test_tool`).

## Goal

Encode lesson PG-MUTANTS-JOBS-001 into the repository so that mutation runs are
reliable by default and future cycles do not silently drop real survivors under
load-induced timeouts. Two concrete deliverables:

1. **`.cargo/mutants.toml`** — the only location cargo-mutants actually reads by
   default — setting a generous per-mutant timeout floor (`minimum_test_timeout`
   >= 300, optionally paired with `timeout_multiplier`) so that infinite-loop
   mutants cannot inflate other mutants past the auto-timeout and produce a false
   "0 missed" result. This file MUST NOT contain a `jobs` key: `jobs` is not a
   valid `Config` field (the parser is `deny_unknown_fields`) and would abort
   every mutation run with a fatal parse error.

2. **A "Mutation testing" note in `CLAUDE.md`** documenting:
   - The recommended invocation stays low-parallelism: bare `cargo mutants`
     (already serial by default) or an explicit `--jobs 1` / `CARGO_MUTANTS_JOBS=1`.
     Explicitly warn against high `--jobs` (e.g. 8) — this is what caused the
     incident — and note that no config file can override an explicit CLI
     `--jobs` flag.
   - Why: infinite-loop mutants peg all cores, inflating other mutants' wall-clock
     past the auto-timeout threshold and producing a false "0 missed" result.
   - The process-gap that motivated this guidance (PG-MUTANTS-JOBS-001,
     fix-tls-clienthello-frag F6, 2026-07-01).

## Acceptance Criteria

AC-147-001: A `.cargo/mutants.toml` file exists at the exact path cargo-mutants
  reads by default (`.cargo/mutants.toml`, source-tree root — this is the ONLY
  location cargo-mutants reads; a repo-root `mutants.toml` and a
  `[package.metadata.mutants]` table in `Cargo.toml` are both silently ignored
  and are NOT accepted locations) that sets a generous timeout floor —
  `minimum_test_timeout` >= 300 (and optionally `timeout_multiplier`) —
  sufficient to prevent load-induced false timeouts on a standard developer
  machine. The file MUST NOT contain a `jobs` key: `jobs` is not a valid
  `Config` field and would abort every run with a fatal parse error.

AC-147-002: File-content verification only (no runtime confirmation required —
  see F-S147P1-005): `.cargo/mutants.toml` exists at the exact path cargo-mutants
  reads (`.cargo/mutants.toml`), uses only valid config keys under the parser's
  `deny_unknown_fields` policy (i.e., contains no `jobs` key and no other
  unrecognized key, which would abort every mutation run with a fatal parse
  error), and sets `minimum_test_timeout` >= 300. "No other unrecognized key" is
  machine-checked by the test's cargo-mutants `Config` field allowlist, which is
  pinned to the EXECUTION-VERIFIED v27.0.0 key set (local strict-parser probes,
  2026-07-19), cross-referenced with v27.1.0 `src/config.rs` research
  (`deny_unknown_fields`); quoted-string numeric values (e.g.
  `minimum_test_timeout = "300"`) are rejected as TOML type errors under the
  same check, not merely a style nit.

AC-147-003: `CLAUDE.md` contains a "Mutation testing" note (within "Build & Test"
  or as a dedicated subsection) that:
  (a) states the recommended invocation stays low-parallelism — bare
      `cargo mutants` (already serial by default) or explicit `--jobs 1` /
      `CARGO_MUTANTS_JOBS=1` — and explicitly WARNS that a high `--jobs` (e.g. 8)
      caused the PG-MUTANTS-JOBS-001 incident and that no config file can
      override an explicit CLI `--jobs` flag,
  (b) explains why high `--jobs` is unsafe on this suite (infinite-loop mutants
      inflate wall-clock past auto-timeout → false "0 missed"),
  (c) references PG-MUTANTS-JOBS-001 and the fix-tls-clienthello-frag F6 cycle,
  (d) references drbothen/vsdd-factory#654 as the upstream engine-default tracking
      issue (informational pointer only — no wirerust action required for the
      mutation-testing skill's own default; see Disposition),
  (e) notes that the config-file defense (AC-147-001) is a `.cargo/mutants.toml`
      timeout floor, not a parallelism default — `jobs` is not a config key at
      all, so parallelism safety can only be enforced by CLI/env convention,
      documented here.

AC-147-004: A self-audit confirms that after this story ships, a developer running
  `cargo mutants` from a fresh checkout will not silently receive a false-clean
  result due to load-induced timeouts. Two defenses, both required (conjunction):
  first line of defense — the `.cargo/mutants.toml` timeout floor (real,
  machine-enforced, defends the auto-timeout mechanism even under parallel load);
  second line of defense — the `CLAUDE.md` note (human-facing guidance against
  explicit high `--jobs`). This self-audit is
  wirerust-local only — it does not depend on or require any change to the
  mutation-testing skill / formal-verifier agent default (engine-level, tracked
  separately per Disposition).

## Notes

- This is a configuration and documentation story. The `.cargo/mutants.toml`
  addition is 7 lines (6 comment + 1 config); the `CLAUDE.md` note is ~12 lines.
  No Rust source changes required.
- Wave 84 (opened 2026-07-19, plan gate approved by human): STORY-147
  (story since revised — see Changelog) + STORY-166 + STORY-176, 7 pts total,
  all product-local.
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
| `.cargo/mutants.toml` (new file, 7 lines) | ~0.1 k |
| `CLAUDE.md` (Build & Test section context, amendment target) | ~0.5 k |
| **Total** | **~1.6 k** |

Well within context window. No story split required.

## Disposition

**Status:** ready — SPLIT disposition (decided at the v2.0 re-scope, 2026-07-19);
product half retained locally, engine half routed upstream.

The human-approved E-11 stale-draft disposition plan
(`.factory/planning/e11-stale-draft-disposition-plan.md`) confirmed via a
delivered-by-drift check on the current tree (no `.cargo/mutants.toml`, no
repo-root `mutants.toml`, no `[package.metadata.mutants]` table, no "Mutation
testing" note in `CLAUDE.md`) that the product half of this story is genuinely
undelivered, and split the story:

| Half | Disposition |
|------|-------------|
| Product (RETAIN LOCALLY, this story — wirerust repo files only; split decided at the v2.0 re-scope) | `.cargo/mutants.toml` timeout floor (`minimum_test_timeout` >= 300, no `jobs` key) in the wirerust repo (`<repo-root>/.cargo/mutants.toml`) + `CLAUDE.md` "Mutation testing" note + self-audit (AC-147-001..004). Points re-scoped 3→2 (engine-skill-default work removed from scope). |
| Engine (mutation-testing skill safe-parallelism default, all VSDD projects) | Routed upstream via drbothen/vsdd-factory#654 evidence comment (posted 2026-07-19): confirming field data — `cargo mutants --jobs 8` reported false "0 missed", hiding two real survivors at tls.rs:950:59/tls.rs:1030:67, surfaced only by a `--jobs 1` re-run (plus eleven more real gaps subsequently closed). |

This story delivers the product half only. No further wirerust delivery expected for
the engine half.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.8 | 2026-07-19 | story-writer | Pass-7 nit: line-count bounds updated to shipped artifact sizes — mutants.toml 7 lines, CLAUDE.md note ~12 (F-S147P7-001). |
| 2.7 | 2026-07-19 | story-writer | Pass-6 nit: Execution-evidence header citation completed to F-S147P1-002/-004/-005 (F-S147P6-001). |
| 2.6 | 2026-07-19 | story-writer | Pass-5 correction: SPLIT provenance anchor fixed to the v2.0 re-scope (F-S147P5-001 — v2.5's event-anchored phrasing carried the wrong version; v2.2 changed no scope). |
| 2.5 | 2026-07-19 | story-writer | Pass-4 anti-drift fix: inline version markers in Disposition converted to event-anchored provenance form (F-S147P4-002; 3rd stale-marker recurrence — process-gap noted for cycle-close). |
| 2.4 | 2026-07-19 | story-writer | Pass-3 execution-evidence alignment: allowlist provenance re-pinned to verified v27.0.0 set (-common +test_tool); Disposition repo-root phrasing disambiguated (F-S147P3-001/-003). |
| 2.3 | 2026-07-19 | story-writer | Pass-2 alignment: AC-147-002 unrecognized-key clause now cites the v27.1.0 allowlist enforcement; stale v2.0 token removed from Notes (F-S147P2-001 + LOW observation). |
| 2.2 | 2026-07-19 | story-writer | Spec-route remediation (Step 4.5 adversarial findings F-S147P1-002/-004/-005, CONFIRMED by execution evidence — cargo-mutants 27.0.0 local probes — + 27.1.0 docs/source research): retitled to "Repo-Local Mutation-Testing Defaults: .cargo/mutants.toml Timeout Floor + CLAUDE.md Guidance"; corrected the config deliverable from a fictional repo-root `mutants.toml`/`Cargo.toml`-metadata surface to the real one — `.cargo/mutants.toml` is the ONLY location cargo-mutants reads by default, and `jobs` is not a valid config key (would abort every run with a fatal parse error under `deny_unknown_fields`) — parallelism safety is CLI/env-only, not config-settable. AC-147-001 rewritten to the `.cargo/mutants.toml` timeout-floor deliverable (`minimum_test_timeout` >= 300); AC-147-002 narrowed to file-content verification only, dropping the implied runtime confirmation (resolves F-S147P1-005's false-green surface); AC-147-003 reworded to recommend low-parallelism invocation (bare `cargo mutants` or explicit `--jobs 1`) with an explicit warning against high `--jobs` and a note that no config file overrides an explicit CLI flag; AC-147-004 "two defenses" reworded to `.cargo/mutants.toml` timeout floor (first, machine-enforced) + CLAUDE.md guidance (second, human-facing). Background appended with an "Execution-evidence correction" subsection citing https://mutants.rs/config-file.html and https://mutants.rs/parallelism.html. No points/status/BC-scope change. |
| 2.1 | 2026-07-19 | story-writer | Remediation: added missing "Token Budget Estimate" section (per-story-delivery.md Token Budget Check). No AC or scope content change. |
| 2.0 | 2026-07-19 | story-writer | SPLIT re-scope (human-approved E-11 stale-draft disposition plan): retitled to "Repo-Local Mutation-Testing Defaults" to reflect wirerust-local-only scope; points 3→2 (engine-skill-default work removed); AC-147-003(d) + AC-147-004 clarified as product-local-only; engine half (mutation-skill safe-parallelism default) routed upstream via drbothen/vsdd-factory#654 evidence comment. Wave TBD→84, status draft→ready (plan gate approved by human, mini-wave 166+176+147v2 = 7 pts). |
| 1.0 | 2026-07-08 | state-manager | Added `document_type: story` and `input-hash: d41d8cd` for scanner compatibility (STORY-157 TASK F; `inputs: []` → canonical empty-inputs hash). |
