# PR Review — STORY-147 (PR #421), fresh-eyes cycle 1

**Reviewer:** pr-reviewer (cognitive-diversity pass; replacement for timed-out reviewer)
**PR:** #421 — `build: add .cargo/mutants.toml timeout floor + mutation-testing guidance (STORY-147)`
**Head SHA reviewed:** `c5feae4bdf7d619715dd5d710217515e996c45c5`
**Base:** `develop` (`49255464`)
**Date:** 2026-07-19

## Verdict: APPROVE

Configuration/docs/tests-only change (`.cargo/mutants.toml`, `CLAUDE.md` note,
`tests/repo_mutation_config_tests.rs`, `docs/demo-evidence/STORY-147/`). Every aggregate
count and every per-test evidence row in the PR description matches ground truth at HEAD
`c5feae4b`. Local scoped test run is 9/9 green. No blocking findings.

## Scope of diff

| File | Change |
|---|---|
| `.cargo/mutants.toml` | NEW, 7 lines — `minimum_test_timeout = 300`, no `jobs` key |
| `CLAUDE.md` | +14 lines — new `### Mutation testing` subsection |
| `tests/repo_mutation_config_tests.rs` | NEW, 554 lines, 9 guard tests |
| `docs/demo-evidence/STORY-147/` | NEW — 5 VHS tape/gif/webm sets + evidence-report.md (16 files) |

Total: additions 1006, deletions 0 (gh-confirmed).

## Row-Verify Table (PG-W74-PRDESC-ROW-VERIFY)

Independently re-verified against `tests/repo_mutation_config_tests.rs` at HEAD:

| Row | Test name (claimed) | Claimed loc | Actual loc | Result |
|-----|---------------------|-------------|-----------|--------|
| 1 | `test_AC_147_001_dot_cargo_mutants_toml_sets_timeout_floor` | line 217 | line 217 | MATCH |
| 3 | `test_AC_147_002_config_keys_are_all_in_v27_allowlist` | (table) | line 321 | MATCH |
| 4 | `test_AC_147_003_claude_md_has_mutation_testing_section` | line 375 | line 375 | MATCH |
| 5 | `test_AC_147_004_both_real_defenses_present_simultaneously` | line 447 | line 447 | MATCH |
| 8 | `test_F_S147P2_001_allowlist_scan_flags_unrecognized_key` | line 522 | line 522 | MATCH |
| 9 | `test_F_S147P2_001_allowlist_scan_accepts_all_pinned_v27_0_0_keys` | (table) | line 540 | MATCH |

All 9 test-function names in the description's table exist in source. No fabricated rows.

## Aggregate-Count Cross-Check

| Claim | Actual | Result |
|-------|--------|--------|
| "9/9 pass" / "9 added" | live `cargo test --test repo_mutation_config_tests` → `9 passed; 0 failed` | MATCH |
| additions 1006, deletions 0 | gh reports `additions: 1006, deletions: 0` | MATCH |
| "5 VHS recordings" + 16 demo files | 5 recordings × {gif,webm,tape} = 15 + evidence-report.md = 16 (name-only confirmed) | MATCH |
| ≥1 demo artifact per AC (AC-147-002 has 2) | every referenced `.gif`/`.webm`/`.tape` resolves to a committed file | MATCH |

## Checklist Verification

- **CHANGELOG gate:** "NOT triggered" claim verified — no path under `src/`, `Cargo.toml`,
  or `bin/`; changed paths are `.cargo/`, `CLAUDE.md`, `tests/`, `docs/` only. Accurate.
- **Semantic title:** `build:` — valid allowed type.
- **CLAUDE.md vs guard assertions:** shipped `### Mutation testing` note contains every
  substring AC-147-003 asserts (`--jobs`, `serial`, `false`+`0 missed`+`wall-clock`,
  `PG-MUTANTS-JOBS-001`, `fix-tls-clienthello-frag`, `#654`, `minimum_test_timeout`).
- **`.cargo/mutants.toml`:** 7 lines, `minimum_test_timeout = 300`, no `jobs` key.
- **Commit/diff coherence:** 13 commits; history shows abandoned repo-root `mutants.toml`
  (commit 2) replaced by `.cargo/mutants.toml` (commit 6), reflecting the documented
  v2.1→v2.2 respec. Final tree has only `.cargo/mutants.toml`; the no-decoy guard test
  actively enforces the repo-root file's absence. Coherent.
- **Test quality (false-green scan):** AC guard tests are legitimate drift guards on real
  files (negative paths demonstrated in demo evidence); the 4 scanner self-checks exercise
  the parser against synthetic in-memory strings, proving discrimination. AC-147-003(b)
  was specifically hardened away from a `contains("timeout")` tautology (lines 393–402).
  No false-green surface found.

## CI status (carried from prior verification artifact)

Prior reviewer artifact recorded: full `cargo test --all-targets` exit 0 (2640 passed
across 94 targets), `cargo clippy --all-targets -- -D warnings` exit 0, `cargo fmt --check`
exit 0, and cargo-mutants 27.0.0 `cargo mutants --list` loading `.cargo/mutants.toml`
successfully against the real strict parser. The single red check observed was a transient
GitHub "Semantic PR" server error (`No server is currently available`), not a title
violation — re-run expected green.

## Findings

**[SUGGESTION] R-421-001 — PR description "Security Review" section is an unfilled
placeholder** (`_Populated after Step 4..._`) and the pre-merge checklist item
"No critical/high security findings unresolved" is unchecked. Security surface is
negligible (no `src/` changes), but the merger should confirm the security-review step is
complete and the section populated before merge so the description does not ship a stub.
Non-blocking.

**[NIT] R-421-002 — `tests/repo_mutation_config_tests.rs:124` / disclosed residual
F-S147P8-001.** `scan_mutants_config` collapses `timeout_multiplier` and
`build_timeout_multiplier` into one field, and a doc comment names them as one. Both are
independently present in the v27 allowlist, so allowlist validation is unaffected; only
the (unused-by-any-AC) multiplier-capture path is touched. Already disclosed as a carried
LOW residual. No action required.

## Rationale for APPROVE

Description is accurate, demo evidence is real and resolves to committed files, aggregate
counts and per-test rows verify against ground truth, and the guard tests genuinely
discriminate good from bad config/doc content. Recommend merge once the transient Semantic
PR check is re-run green.
