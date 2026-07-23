---
document_type: story
level: ops
story_id: STORY-181
title: "Fix SEC-001 ENIP Unsafe Split-Borrow in on_data: Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop (Behavior-Preserving Refactor)"
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

# STORY-181: Fix SEC-001 ENIP Unsafe Split-Borrow in on_data: Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop (Behavior-Preserving Refactor)

**Epic:** E-20 (EtherNet/IP ENIP/CIP Analyzer)
**Status:** draft
**Wave:** 85
**Points:** 3
**Priority:** P2

## Narrative

**As a** security engineer maintaining the wirerust codebase,
**I want** the `on_data` function in `src/analyzer/enip.rs` to dispatch PDUs to
`process_pdu` using a safe Rust borrow pattern rather than a raw-pointer-derived
`&mut EnipFlowState`,
**so that** the ENIP analyzer no longer contains a fragile `unsafe` block in its PDU
dispatch loop, the SAFETY comment invariant ("process_pdu does NOT access self.flows")
is enforced structurally rather than by convention, and future refactoring of
`EnipAnalyzer` cannot silently break the split-borrow soundness proof.

This story resolves SEC-001 from `.factory/tech-debt-register.md` (MEDIUM, filed PR #334
security review, re-triaged into wave-85 at D-493 after the "next feature wave" target
passed without pickup).

**Root cause of SEC-001:** In the PDU dispatch `for pdu in pdu_queue` loop (lines 985–1000),
`on_data` acquires a `*mut EnipFlowState` raw pointer via `self.flows.get_mut(&flow_key)`,
then calls `self.process_pdu(unsafe { &mut *flow_ptr }, &pdu, ...)`. This creates a
simultaneous aliasing situation: `self.process_pdu` requires `&mut self` (which includes
`self.flows`), while `flow_ptr` is a live raw pointer into `self.flows[flow_key]`. The
compiler cannot verify disjointness; a multi-line SAFETY comment at lines 986–991
documents the required invariant that `process_pdu` never accesses `self.flows`. The
pattern is sound as written but fragile — any future change to `process_pdu` that touches
`self.flows` would silently break soundness.

Note: the carry-buffer select at lines 825–829 already uses `std::mem::take` on
`flow.carry_c2s` / `flow.carry_s2c` and is safe. SEC-001 is exclusively the `*mut
EnipFlowState` raw pointer at lines 992–999 in the PDU dispatch loop.

**Target design (take-remove-reinsert):** Before the `for pdu in pdu_queue` loop, remove
the flow from `self.flows` with `self.flows.remove(&flow_key)`. The resulting local owned
`EnipFlowState` no longer aliases `self.flows`, so `self.process_pdu(&mut flow, &pdu, ...)`
is unambiguously safe — `&mut self` (for process_pdu) and `&mut flow` (the local variable)
are disjoint. After the loop, re-insert with `self.flows.insert(flow_key, flow)`.
`process_pdu`'s signature is unchanged; behavior is identical because `process_pdu`
(per its SAFETY comment) never accesses `self.flows`.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.016 | ENIP Per-Direction Carry Buffer (from STORY-139) | Regression-guard anchor: the refactored on_data MUST preserve all carry-buffer postconditions and invariants; all existing BC-2.17.016 test coverage must pass unchanged |

## Acceptance Criteria

### AC-181-001: Unsafe *mut EnipFlowState split-borrow in PDU dispatch loop is eliminated
- The `unsafe` block at `src/analyzer/enip.rs` lines 992–999 (approximately) — which casts
  `self.flows.get_mut(&flow_key)` to `*mut EnipFlowState` then passes `unsafe { &mut
  *flow_ptr }` to `self.process_pdu` — is removed and replaced with a safe pattern
- Specifically, before the `for pdu in pdu_queue` loop `self.flows.remove(&flow_key)`
  produces an owned local `EnipFlowState`; `self.process_pdu(&mut flow, &pdu, ...)` is
  called with this local variable (no aliasing with `self.flows`); after the loop
  `self.flows.insert(flow_key, flow)` re-inserts the flow
- After the fix, `grep -n "unsafe" src/analyzer/enip.rs` returns no match at the former
  `flow_ptr` site; the `*mut EnipFlowState` declaration and the `unsafe { &mut *flow_ptr
  }` expression are both absent from the PDU dispatch loop (the `#[allow(clippy::ptr_as_ptr)]`
  annotation there is also removed); any `unsafe` elsewhere in the file is out of scope
- `process_pdu`'s method signature (`pub fn process_pdu(&mut self, flow: &mut
  EnipFlowState, ...)`) is unchanged — the fix is local to the `on_data` call site
(traces to BC-2.17.016 invariant: PDU dispatch refactor must preserve carry behavior and all existing test postconditions)

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
| `on_data` PDU dispatch loop (refactor site) | SS-17 ENIP PDU dispatch | `src/analyzer/enip.rs` | Effectful (stream dispatch) |
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
| PDU dispatch loop (take-remove-reinsert) | Effectful-shell | Removes flow from self.flows, calls process_pdu for each PDU, re-inserts flow |
| `parse_line()` docstring fix | Pure (doc only) | No code logic change; docstring addition only |

## Tasks

- [ ] Locate the `unsafe` PDU dispatch block in `src/analyzer/enip.rs` `on_data` function
  (approximately lines 985–1000 per tech-debt-register.md SEC-001). Confirm the exact lines:
  the `let flow_ptr: *mut EnipFlowState = self.flows.get_mut(&flow_key)...` declaration
  and the `self.process_pdu(unsafe { &mut *flow_ptr }, &pdu, ...)` call inside the
  `for pdu in pdu_queue` loop.
- [ ] Replace the unsafe split-borrow with the take-remove-reinsert pattern:
  - Before the `for pdu in pdu_queue` loop, add:
    `let mut flow = self.flows.remove(&flow_key).expect("flow exists: inserted above and not removed");`
  - Replace the per-iteration `flow_ptr` + `unsafe { &mut *flow_ptr }` with:
    `self.process_pdu(&mut flow, &pdu, timestamp, src_ip);`
  - After the loop, add: `self.flows.insert(flow_key, flow);`
  - Remove the `#[allow(clippy::ptr_as_ptr)]` annotation that accompanied the unsafe block
- [ ] Verify no `unsafe` block remains at the former `flow_ptr` site with
  `grep -n "unsafe" src/analyzer/enip.rs`; confirm the `*mut EnipFlowState` cast is gone
- [ ] Run `cargo test --all-targets` and confirm zero failures
- [ ] Run `cargo clippy --all-targets -- -D warnings` and confirm no new warnings
- [ ] (Optional, advisory) Update `parse_line()` docstring in `bin/validate-citations` to
  add the regex-mismatch None return case (AC-181-004 OBS-1 residual)
- [ ] If the bin/ docstring is updated, verify `python3 bin/test_validate_citations.py` still passes
- [ ] Update PR description with at least 3 BC-2.17.016 test names as regression evidence
  (e.g. `test_BC_2_17_016_*` tests from STORY-139 delivery)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Multiple PDUs in a single pdu_queue (common case) | All PDUs are dispatched sequentially with the local owned flow; mutations between iterations persist; flow is re-inserted after the last PDU; behavior identical to pre-refactor |
| EC-002 | Empty pdu_queue (is_non_enip was set or no valid PDUs) | The `for` body never executes; remove + re-insert is a no-op; no flow state is lost |
| EC-003 | BC-2.17.016 per-direction carry postconditions | Carry buffer accumulation, drain, and per-direction isolation are unchanged — the refactor touches only the PDU dispatch loop, not the carry-select at lines 825–829 |
| EC-004 | Future process_pdu modification that accesses self.flows | Such a change would be a logic error (self.flows[flow_key] absent during the loop) detectable by tests rather than an invisible soundness violation; structural safety is improved over the prior SAFETY-comment-only invariant |
| EC-005 | `bin/validate-citations` parse_line() with regex-mismatch input | Returns `None`; the caller correctly treats this as MALFORMED; the updated docstring describes this case |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~2,000 |
| BC-2.17.016 (~900 tokens) | ~900 |
| src/analyzer/enip.rs (large file, full scan needed) | ~25,000 |
| src/analyzer/enip.rs process_pdu signature + SAFETY comment (verify fields mutated) | ~1,000 |
| bin/validate-citations (if OBS-1 is fixed; ~310 lines) | ~2,500 |
| **TOTAL** | **~33,400** |

Agent context window ~200k tokens. This story uses ~17% — within budget.

## Previous Story Intelligence

**Primary predecessor: STORY-139** (ENIP per-direction carry buffer + saturating window
monotonicity, wave 62, delivered):
- STORY-139 established the per-direction carry fields `carry_c2s` and `carry_s2c` in
  `EnipFlowState` and fixed the EC-X1 cross-direction carry splice bug. Its delivery also
  introduced the safe `std::mem::take` carry select at lines 825–829.
- The SEC-001 `*mut EnipFlowState` unsafe block (lines 992–999) predates STORY-139 and was
  noted in the PR #334 security review but not fixed then; it is a distinct site from the
  carry select.
- Read STORY-139's delivery PR to understand `EnipFlowState` field shapes and the BC-2.17.016
  test suite that forms the regression guard for this story.

**Analogy: STORY-142** (DNP3 desync-latch one-line fix, wave 64, 3 pts):
- STORY-142 was a targeted one-function fix (3 pts) that was behavior-preserving and
  required all existing tests to pass unchanged — the same profile as SEC-001.
- The fix pattern: locate the exact lines, replace the fragile construct, verify tests pass.

**STORY-166 precedent (ROUTE-W74 items)**:
- STORY-166 (wave-84, delivered, PR #426) handled ROUTE-W74 MINOR-1/2/NIT-1/4 items as
  part of AC-166-001(g). The residual OBS-1 docstring item was NOT in STORY-166's task list
  and is carried as an advisory AC-181-004 here.

## Architecture Compliance Rules

From `src/analyzer/enip.rs` SAFETY comment (lines 979–991, the invariant being made structural):
- The existing SAFETY comment states: "process_pdu does NOT access self.flows (verified by
  inspection); the flow we pass is from self.flows[flow_key], and process_pdu only mutates
  self.all_findings, self.error_count, self.write_count, self.dropped_findings, and threshold
  fields." The take-remove-reinsert pattern makes this invariant structurally enforced: the
  flow is absent from `self.flows` during the loop, so no `self.flows` access by process_pdu
  could conflict.
- The carry-buffer select (lines 825–829) already uses `std::mem::take` and is safe — do NOT
  modify it. SEC-001 is exclusively the PDU dispatch loop (lines 992–999).

From ADR-010 Decision 4 (frame-walk / detection order):
- `on_data` collects valid PDUs during the frame-walk loop, then dispatches them in a
  separate PDU dispatch phase. The take-remove-reinsert pattern is applied to the dispatch
  phase only; the frame-walk loop borrow is released at its block exit (line 975 comment)
  before the dispatch phase begins.

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

- New `unsafe` blocks in the PDU dispatch loop — the fix MUST be safe Rust with no raw-pointer casts
- Changes to `process_pdu`'s method signature or `EnipFlowState` public fields — behavior-preserving scope only; `process_pdu` must remain `pub fn process_pdu(&mut self, flow: &mut EnipFlowState, ...)`
- Changes to any test assertion — tests must pass exactly as-is
