---
document_type: story
level: ops
story_id: STORY-181
title: "Fix SEC-001 ENIP Unsafe Split-Borrow in on_data: Direction-Keyed Carry Select (Behavior-Preserving Refactor)"
epic_id: E-20
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-23T00:00:00Z
phase: f2
traces_to: .factory/specs/prd.md
points: 3
depends_on: []
blocks: []
# BC status: BC-2.17.016 is used as regression-guard anchor (carry behavior must be
# preserved); no new behavioral contracts are introduced by this refactor.
behavioral_contracts:
  - BC-2.17.016
verification_properties: []
priority: P2
cycle: maint-2026-07-23
wave: 85
target_module: analyzer/enip
subsystems: [SS-17]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: null
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.016.md
input-hash: "8253122"
---

# STORY-181: Fix SEC-001 ENIP Unsafe Split-Borrow in on_data: Direction-Keyed Carry Select (Behavior-Preserving Refactor)

**Epic:** E-20 (EtherNet/IP ENIP/CIP Analyzer)
**Status:** draft
**Wave:** 85
**Points:** 3
**Priority:** P2

## Narrative

**As a** security engineer maintaining the wirerust codebase,
**I want** the `on_data` function in `src/analyzer/enip.rs` to select the active carry
buffer using the safe direction-keyed owned-borrow pattern (as used by `modbus.rs`) rather
than the unsafe pointer-derived split-borrow currently in place,
**so that** the ENIP analyzer no longer contains a fragile `unsafe` block in its hot path,
future refactoring of `EnipFlowState` carries no risk of silently breaking the borrow-split
invariant, and the carry-buffer acquisition code is consistent with the house pattern.

This story resolves SEC-001 from `.factory/tech-debt-register.md` (MEDIUM, filed PR #334
security review, re-triaged into wave-85 at D-493 after the "next feature wave" target
passed without pickup).

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.016 | ENIP Per-Direction Carry Buffer (from STORY-139) | Regression-guard anchor: the refactored on_data MUST preserve all carry-buffer postconditions and invariants; all existing BC-2.17.016 test coverage must pass unchanged |

## Acceptance Criteria

### AC-181-001: Unsafe split-borrow in src/analyzer/enip.rs on_data carry acquisition is eliminated
- The `unsafe` block at `src/analyzer/enip.rs` lines 992–999 (approximately) that derives
  simultaneous `&mut` borrows of `state.carry_c2s` and `state.carry_s2c` using raw pointer
  casts is replaced with the direction-keyed owned-borrow pattern
- After the refactor, the carry-buffer acquisition in `on_data` uses a single-direction
  select at call entry, for example:
  ```rust
  let carry = match direction {
      Direction::ClientToServer => &mut state.carry_c2s,
      Direction::ServerToClient => &mut state.carry_s2c,
  };
  ```
  or equivalent (the exact pattern must avoid simultaneous `&mut` borrows of both fields)
- `grep -n "unsafe" src/analyzer/enip.rs` must NOT match any carry-acquisition site in
  `on_data` after the fix (other pre-existing `unsafe` blocks, if any, are out of scope)
(traces to BC-2.17.016 invariant: carry buffer acquisition must be sound and fragility-free)

### AC-181-002: All existing ENIP tests pass unchanged — behavior is identical
- `cargo test --all-targets` passes with zero failures after the refactor
- No test assertion is modified: the refactor is behavior-preserving; carry buffer
  accumulation, drain, and frame-walk behavior are identical to pre-refactor
- The PR description MUST enumerate at least three existing tests that exercise the carry
  path (e.g., `test_BC_2_17_016_*` tests from the STORY-139 delivery) as evidence that
  the regression guard is satisfied
(traces to BC-2.17.016 postconditions: carry buffer accumulation and per-direction isolation
preserved under refactor)

### AC-181-003: No public API surface change
- The refactor is implementation-internal to the `on_data` function body
- No `pub` or `pub(crate)` signatures in `src/analyzer/enip.rs` change
- No Cargo.toml changes, no new crate dependencies
(traces to BC-2.17.016 invariant: refactor scope is carry acquisition only)

### AC-181-004 (ROUTE-W74 OBS-1 residual, bin/ housekeeping): parse_line() docstring in bin/validate-citations clarified for regex-mismatch None return path
- The docstring for `parse_line()` in `bin/validate-citations` is updated to add the
  missing third return case: "or None if the line fails the citation regex (caller should
  treat as MALFORMED)"
- All existing tests in `bin/test_validate_citations.py` (25+ after STORY-166 T23/T24/T25)
  pass unchanged — this is a documentation-only one-line addition
- This resolves wave-74 gate-summary OBS-1 (`.factory/cycles/wave-74/wave-gate/gate-summary.md`
  deferred register), which was not in STORY-166's AC-166-001(g) task list and remains open
  as a carry-forward; it is folded here as the next opportunity to touch a bin/ file in the
  same PR (if the implementer determines the bin/ file change is not worth a separate commit,
  it may be batched with a future housekeeping PR — this AC is advisory, not blocking)

  **Note:** The primary ROUTE-W74 items (MINOR-1, MINOR-2, NIT-1, NIT-4) were absorbed by
  STORY-166 (wave-84, delivered, PR #426). OBS-1 is the sole residual item not covered by
  STORY-166. No wave-85 story naturally touches `bin/validate-citations`; this AC is the
  closest opportunity to close the carry-forward.
(traces to BC-2.17.016 N/A for housekeeping task — included for ROUTE-W74 disposition; bin/ change is behavior-neutral)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `on_data` carry acquisition (refactor site) | SS-17 ENIP carry buffer | `src/analyzer/enip.rs` | Effectful (stream dispatch) |
| `parse_line()` docstring (OBS-1, optional) | bin/ tooling | `bin/validate-citations` | Pure (no behavior change) |

Subsystem anchor: SS-17 owns this story's scope because the ENIP carry buffer is a core
stream-processing component of the EtherNet/IP passive analyzer per ARCH-INDEX.md §SS-17.
The SEC-001 split-borrow is within SS-17's `on_data` implementation.

Dependency anchor: STORY-181 has no blocking predecessors (depends_on: []) because all
prior E-20 stories including STORY-139 (ENIP per-direction carry buffer, wave 62) are
already delivered on develop. The refactor can proceed against the current develop HEAD.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `on_data` (enip.rs, post-refactor) | Effectful-shell | Processes TCP stream bytes, updates EnipFlowState, emits findings |
| Carry-buffer select | Effectful-shell | Selects mutable borrow of carry_c2s or carry_s2c based on direction |
| `parse_line()` docstring fix | Pure (doc only) | No code logic change; docstring addition only |

## Tasks

- [ ] Locate the `unsafe` split-borrow block in `src/analyzer/enip.rs` `on_data` function
  (approximately lines 992–999 per tech-debt-register.md SEC-001). Confirm the exact lines
  and the pattern used (raw pointer cast to obtain `&mut carry_c2s` and `&mut carry_s2c`
  simultaneously).
- [ ] Replace the unsafe split-borrow with a direction-keyed carry select at the start of
  `on_data`:
  - Select `&mut state.carry_c2s` when `direction == Direction::ClientToServer`
  - Select `&mut state.carry_s2c` when `direction == Direction::ServerToClient`
  - Propagate the single `carry` reference through all downstream carry operations in the
    function that currently borrow both fields simultaneously
  - Use the same pattern as `src/analyzer/modbus.rs` for consistency (the house pattern)
- [ ] Verify no `unsafe` block remains in the carry-acquisition path of `on_data` with
  `grep -n "unsafe" src/analyzer/enip.rs`
- [ ] Run `cargo test --all-targets` and confirm zero failures
- [ ] Run `cargo clippy --all-targets -- -D warnings` and confirm no new warnings
- [ ] (Optional, advisory) Update `parse_line()` docstring in `bin/validate-citations` to
  add the regex-mismatch None return case (AC-181-004 OBS-1 residual)
- [ ] If the bin/ docstring is updated, verify `python3 bin/test_validate_citations.py` still passes
- [ ] Update PR description with at least 3 carry-path test names as regression evidence

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | C→S direction carry buffer accumulation | Carry drains correctly into `state.carry_c2s`; behavior identical to pre-refactor |
| EC-002 | S→C direction carry buffer accumulation | Carry drains correctly into `state.carry_s2c`; behavior identical to pre-refactor |
| EC-003 | Single-direction flow (only C→S frames seen) | Only `carry_c2s` is touched; `carry_s2c` unchanged |
| EC-004 | Refactored code with a future EnipFlowState field rename | No unsafe raw-pointer arithmetic means a rename is caught by the compiler, not silently broken |
| EC-005 | `bin/validate-citations` parse_line() with regex-mismatch input | Returns `None`; the caller correctly treats this as MALFORMED; the updated docstring describes this case |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~2,000 |
| BC-2.17.016 (~900 tokens) | ~900 |
| src/analyzer/enip.rs (large file, full scan needed) | ~25,000 |
| src/analyzer/modbus.rs (house-pattern reference, carry select section) | ~3,000 |
| bin/validate-citations (if OBS-1 is fixed; ~310 lines) | ~2,500 |
| **TOTAL** | **~33,400** |

Agent context window ~200k tokens. This story uses ~17% — within budget.

## Previous Story Intelligence

**Primary predecessor: STORY-139** (ENIP per-direction carry buffer + saturating window
monotonicity, wave 62, delivered):
- STORY-139 established the per-direction carry fields `carry_c2s` and `carry_s2c` in
  `EnipFlowState` and fixed the EC-X1 cross-direction carry splice bug
- The unsafe split-borrow SEC-001 predates STORY-139 and was noted in the PR #334 security
  review but not fixed then; STORY-139's carry-direction work happened concurrently
- Read STORY-139's deliver PR (#384/enip carry) to understand the carry field shapes and
  any existing test coverage for per-direction isolation

**Analogy: STORY-142** (DNP3 desync-latch one-line fix, wave 64, 3 pts):
- STORY-142 was a targeted one-function fix (3 pts) that was behavior-preserving and
  required all existing tests to pass unchanged — the same profile as SEC-001
- The fix pattern: locate the exact lines, replace the fragile construct, verify tests pass

**STORY-166 precedent (ROUTE-W74 items)**:
- STORY-166 (wave-84, delivered, PR #426) handled ROUTE-W74 MINOR-1/2/NIT-1/4 items as
  part of AC-166-001(g). The residual OBS-1 docstring item was NOT in STORY-166's task list
  and is carried as an advisory AC-181-004 here.

## Architecture Compliance Rules

From `src/analyzer/modbus.rs` (house pattern reference):
- Direction-keyed carry select: at the start of `on_data`, select the appropriate carry
  buffer with a `match direction { ... }` expression. Use the selected reference throughout
  the function. This avoids ever needing two `&mut` borrows of the same struct simultaneously.
- No unsafe: the house style for carry buffers is safe Rust only. `unsafe` split-borrow is
  a known fragility point (SEC-001 tech-debt-register entry); the direction-keyed pattern
  is the correct replacement.
- All carry modifications use `extend_from_slice`, `drain`, and direct `.len()` calls on
  the selected `carry` reference — the same idioms already present in the unsafe path.

From ADR-0012 and ADR-0013 (architectural consistency):
- ENIP and IEC-104 analyzers should follow the same carry-buffer pattern as Modbus and DNP3
  (per the carry-direction fix series STORY-139 through STORY-142)

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | `match`, `&mut` borrow, no unsafe |

No new crate dependencies. No Cargo.toml changes.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/enip.rs` | MODIFY | Replace unsafe split-borrow carry acquisition with direction-keyed owned-borrow pattern in `on_data`; no behavior change |
| `bin/validate-citations` | MODIFY (advisory) | Add regex-mismatch None case to `parse_line()` docstring (AC-181-004 OBS-1 residual; one-line docstring addition) |

## Forbidden Dependencies

- New `unsafe` blocks in the carry-acquisition path — the fix MUST be safe Rust
- Changes to `EnipFlowState` public fields or public methods — behavior-preserving scope only
- Changes to any test assertion — tests must pass exactly as-is
