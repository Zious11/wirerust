# Review Findings — FIX-W68-01 (PR #354)

**Fix:** F-W68-01 — `wirerust protocols --json=PATH` silent failure
**PR:** #354 — https://github.com/Zious11/wirerust/pull/354
**Branch:** fix/protocols-json-output-routing @ 4b101ee

## Convergence Table

| Cycle | Reviewer | Total Findings | BLOCKING | NON-BLOCKING | COSMETIC | Fixed | Remaining | Verdict |
|-------|---------|----------------|----------|--------------|---------|-------|-----------|---------|
| 1 | pr-reviewer | 3 | 0 | 1 | 2 | 0 | 0 blocking | APPROVE |

**Convergence achieved: cycle 1 of max 10.**
**DF-CONVERGENCE-BEFORE-MERGE-001: SATISFIED (0 blocking findings at convergence)**

## Security Review

| Cycle | Reviewer | CRITICAL | HIGH | MEDIUM | LOW | INFO | Verdict |
|-------|---------|----------|------|--------|-----|------|---------|
| 1 | security-reviewer | 0 | 0 | 0 | 2 | 3 | CLEAN |

## Findings Detail

### PR Review — Cycle 1

**NON-BLOCKING — `std::process::exit(1)` vs `Err(anyhow!(…))` for CSV rejection** (`src/main.rs:588–593`)
- Severity: NON-BLOCKING
- Description: `run_protocols` returns `Result<()>`, so returning an anyhow error would be more idiomatic and preserve drop semantics. However, at the call site no file handles or sensitive state are live, and the pattern exactly matches existing style elsewhere in `main.rs` (lines 553–554). Tests exercise exit-code + stderr correctly either way.
- Routed to: Deferred to backlog — matches existing codebase idiom, no behavioral consequence
- Status: DEFERRED

**COSMETIC — Test temp-file leak on assertion failure** (`tests/integration_tests.rs`, `test_BC_2_12_022_json_path_writes_file`)
- Severity: COSMETIC
- Description: The `remove_file` call between `read_to_string` and further assertions means a panic in between leaks the temp file. A Drop-guard or `scopeguard::defer!` would make cleanup unconditional.
- Routed to: Backlog — test hygiene, no behavioral consequence
- Status: DEFERRED

**COSMETIC — `src/cli.rs` `--csv` docstring drift**
- Severity: COSMETIC
- Description: The global `--csv` flag doc comment (`src/cli.rs:62`) does not mention that `protocols` now explicitly rejects CSV. Worth a one-line addendum on next touch.
- Routed to: Backlog — doc-comment drift, not blocking
- Status: DEFERRED

### Security Review — Cycle 1

**SEC-001 — Path traversal via `--json=<PATH>` (LOW, CWE-22)**
- Severity: LOW
- Description: Pre-existing `write_output` behavior, not new to this PR. CLI threat model; no privilege boundary crossed. Reviewed clean in PR #353.
- Status: ACKNOWLEDGED / NO ACTION REQUIRED

**SEC-002 — Predictable temp file in test (LOW, CWE-377)**
- Severity: LOW
- Description: `test_BC_2_12_022_json_path_writes_file` uses PID-only temp filename. CI runners are single-tenanted; benign JSON content only.
- Status: ACKNOWLEDGED / DEFERRED (low urgency)

**SEC-003 — `process::exit(1)` Drop bypass (INFO, CWE-404)**
- Severity: INFO
- Description: No sensitive state live at call site; existing codebase pattern.
- Status: ACKNOWLEDGED / NO ACTION REQUIRED

**SEC-004 — Format string safety (INFO — CLEAN)**
- Status: CLEAN / NO FINDING

**SEC-005 — No new dependencies (INFO — CLEAN)**
- Status: CLEAN / NO FINDING

## Pre-Merge Gate Summary

- [x] Review convergence: 0 blocking findings (cycle 1 APPROVE)
- [x] Security: CLEAN (0 CRITICAL/HIGH/MEDIUM)
- [x] Adversarial convergence (pre-PR): 1 pass CLEAN on 4b101ee (0 P0/CRITICAL/HIGH)
- [x] CI: GREEN — 11/11 checks pass on 4b101ee (action-pin-gate, audit, clippy, deny, fmt, fuzz-build, green-doc-tense, help-provenance, semantic-PR, test, trust-boundary)
- [x] Dependency check: STORY-152 (PR #353) merged into develop
- [ ] Human approval for squash merge
