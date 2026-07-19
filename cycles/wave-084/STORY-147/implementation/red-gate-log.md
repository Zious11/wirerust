---
document_type: red-gate-log
level: ops
version: "1.0"
status: draft
producer: test-writer
timestamp: 2026-07-19T20:01:44
phase: 3
inputs: []
input-hash: "d41d8cd"
traces_to: "STORY-147"
stub_architect_agent: "n/a (NO-STUBS verdict)"
stub_compile_verified: true
test_writer_agent: "test-writer"
red_gate_verified: true
---

# Red Gate Log: Wave 84 / STORY-147

Story: STORY-147 v2.1 "Repo-Local Mutation-Testing Defaults: mutants.toml
(jobs=1) + CLAUDE.md Guidance"
Branch: `feature/STORY-147-mutation-testing-defaults`
Worktree: `.worktrees/STORY-147`
Base SHA: `492554642c7d` (develop)

## Summary
| Story | Tests Written | All Fail (Red)? | Gate |
|-------|-------------|-----------------|------|
| STORY-147 | 4 (AC-147-001..004) | Yes — 0 passed / 4 failed | PASSED |

## Stubs Created

### STORY-147: Repo-Local Mutation-Testing Defaults

**Step 2 verdict: NO-STUBS.** This is a config/docs-only story (no source
functions to stub). Stub state for the config artifacts is their current
absence from the tree — no `mutants.toml`, no `.cargo/mutants.toml`, no
`[package.metadata.mutants]` table in `Cargo.toml`, and no "Mutation
testing" section in `CLAUDE.md` exist yet. The existing `tests/` harness is
sufficient to host the new failing tests without additional scaffolding.

`cargo check --all-targets` was clean at base SHA `492554642c7d` prior to
any test authoring. No stub commit was made — this is a valid, documented
outcome for a config/docs-only story per Step 2 guidance.

## Red Gate Verification

### STORY-147

Test file: `tests/repo_mutation_config_tests.rs`
Commit: `fa23ce0a` — "test(STORY-147): add failing tests for AC-147-001..004
(Red Gate)"
No dev-dependencies were added; tests use a hand-rolled line-oriented TOML
scanner in-test rather than pulling in a TOML crate.

- AC-147-001: `test_AC_147_001_low_parallelism_mutation_config_exists` —
  FAIL (expected) — "no mutation-testing config sets a low-parallelism
  default... None of the checked locations exist yet on this tree
  (mutants.toml root, .cargo/mutants.toml, Cargo.toml
  [package.metadata.mutants])"
- AC-147-002: `test_AC_147_002_low_parallelism_value_active_at_cargo_mutants_read_location`
  — FAIL (expected) — "no cargo-mutants-readable config location
  currently sets an active low-parallelism default"
- AC-147-003: `test_AC_147_003_claude_md_has_mutation_testing_section` —
  FAIL (expected) — "AC-147-003(a): CLAUDE.md is missing a 'Mutation
  testing' section/heading."
- AC-147-004: `test_AC_147_004_both_defenses_present_simultaneously` —
  FAIL (expected) — "first line of defense (repo-root low-parallelism
  mutation config) is absent"

## Regression Check

Independent orchestrator verification via `cargo test --all-targets` in
the worktree, 2026-07-19:

| Existing Tests | Status |
|---------------|--------|
| Pre-existing binaries (97+10+3+3+5+10+22+19+31+27+20+29+24+21+18+16+42+9+23+30 tests) | all pass (0 failures) |
| New binary (`repo_mutation_config_tests`) | 0 passed / 4 failed — assertion panics with AC-referenced behavior messages (not compile errors, not "not implemented") |

**RED GATE: PASSED** — correctly red. All 4 new tests fail with
substantive, AC-traceable assertion messages; no pre-existing test broke.

## Hand-Off to Implementer

- Stories ready for implementation: STORY-147
- Implementation guidance: Add repo-root `mutants.toml` (or equivalent
  cargo-mutants-readable location) setting `jobs = 1` as the low-parallelism
  default (AC-147-001, AC-147-002), and add a "Mutation testing" section to
  `CLAUDE.md` documenting the guidance (AC-147-003). AC-147-004 requires
  both defenses to be present simultaneously — implement together and
  re-run `tests/repo_mutation_config_tests.rs` to confirm all 4 tests go
  green with no regressions in the pre-existing suite.
