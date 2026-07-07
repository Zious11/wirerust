---
document_type: red-gate-log
level: ops
version: "1.0"
status: verified
producer: test-writer
timestamp: 2026-07-07T00:00:00Z
phase: 3
inputs:
  - .factory/stories/STORY-149.md
input-hash: "d2bd33e"
traces_to: STORY-149
stub_architect_agent: stub-architect (wave-70)
stub_compile_verified: true
test_writer_agent: test-writer (wave-70)
red_gate_verified: true
---

# Red Gate Log: Wave 70 — STORY-149

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|---------------|-----------------|------|
| STORY-149 | 5 | Yes (5/5 FAIL, 0 regressions) | PASS |

## Stubs Created

### STORY-149: TLS Carry-Path Performance Recovery + Fragmented-Handshake Benchmark Fixture

Stub commit `7ee8078` "feat(STORY-149): add module stubs":

- `benches/tls_fragmented.rs` — Criterion bench file with `todo!()` builder body for the
  fragmented TLS handshake fixture (AC-149-002). Registers no runnable benchmark until
  implemented.
- `Cargo.toml` — added `[[bench]] name = "tls_fragmented"` entry pointing at the stub file.

`cargo check`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` all
passed clean at stub commit.

## Red Gate Verification

### STORY-149

Test commit `e951664` "test(STORY-149): add failing tests for AC-149-001/002 (PERF-001/002, issue #360)".

**File: `tests/bc_149_single_borrow_invariant_tests.rs`** (AC-149-001, 2 tests)

| AC | Test Name | Result | Failure Reason |
|----|-----------|--------|---------------|
| AC-149-001 | `test_BC_149_001_at_most_one_flows_borrow_in_try_parse_records` | FAIL | Assertion: "Found 13 `flows.get_mut(` + 6 `flows.get(` = 19 total borrow call site(s)" — limit is ≤1; existing `try_parse_records` in `src/analyzer/tls.rs` violates the single-borrow invariant |
| AC-149-001 | `test_BC_149_001_single_borrow_invariant_comment_marker_present` | FAIL | Assertion: marker string `SINGLE-BORROW INVARIANT` absent from `src/analyzer/tls.rs` — comment has not yet been written |

**File: `tests/bc_149_fragmented_fixture_tests.rs`** (AC-149-002, 3 tests)

| AC | Test Name | Result | Failure Reason |
|----|-----------|--------|---------------|
| AC-149-002 | `test_BC_149_002_fixture_spans_at_least_3_records` | FAIL | `todo!()` panic: "STORY-149: implement synthetic ≥3-record fragmented TLS handshake builder" |
| AC-149-002 | `test_BC_149_002_carry_drain_loop_exercised_across_records` | FAIL | `todo!()` panic: "STORY-149: implement synthetic ≥3-record fragmented TLS handshake builder" |
| AC-149-002 | `test_BC_149_002_fixture_is_deterministic` | FAIL | `todo!()` panic: "STORY-149: implement synthetic ≥3-record fragmented TLS handshake builder" |

**Toolchain gates at test commit:**

- `cargo clippy --all-targets -- -D warnings` — PASS
- `cargo fmt --check` — PASS

## Regression Check

| Existing Test Suites | Status |
|----------------------|--------|
| All pre-existing test suites (base develop `19569ae`) | PASS — 0 regressions |

`cargo test --all-targets` compiled cleanly. All pre-existing suites were green. The 5
failures above are the only failures; all are in the newly added test files and are
expected by Red Gate discipline.

## Hand-Off to Implementer

- Stories ready for implementation: STORY-149
- Branch: `feature/STORY-149-tls-carry-perf`
- Worktree: `.worktrees/STORY-149`
- Base develop commit: `19569ae`

Implementation guidance:

1. **AC-149-001 — Single-borrow invariant.** Refactor `try_parse_records` in
   `src/analyzer/tls.rs` to acquire exactly one `flows.get_mut()` per invocation,
   eliminating the 13 `get_mut` + 6 `get` call sites found by the grep-based test.
   Add an inline comment containing the exact string `SINGLE-BORROW INVARIANT` at
   the borrow site. Replace per-record carry Vec allocation with `std::mem::replace`
   swap pattern (PERF-002).

2. **AC-149-002 — Fragmented benchmark fixture.** Implement the synthetic ≥3-record
   TLS handshake builder in `benches/tls_fragmented.rs` so the carry-drain loop
   executes at least twice per synthetic handshake. Fixture must be deterministic
   and repeatable (no random seeding). This closes issue #360.

3. Run `cargo test --all-targets` after implementation — all 5 tests above must turn
   GREEN; zero pre-existing regressions allowed (VP-039 / VP-040 must remain green).
