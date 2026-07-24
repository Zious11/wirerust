---
document_type: red-gate-log
level: ops
version: "1.0"
status: final
producer: test-writer
timestamp: 2026-07-24T00:00:00
phase: 3
inputs: []
input-hash: "d41d8cd"
traces_to: "STORY-181"
stub_architect_agent: "[orchestrator-verified 2026-07-24]"
stub_compile_verified: true
test_writer_agent: "[orchestrator-verified 2026-07-24]"
red_gate_verified: true
---

# Red Gate Log: STORY-181 — SEC-001 EtherNet/IP Unsafe Split-Borrow Elimination

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| STORY-181 | 0 (NO-NEW-TESTS-REQUIRED) | N/A-BY-DESIGN | N/A-BY-DESIGN |

**Verdict: N/A-BY-DESIGN.** STORY-181 is a behavior-preserving refactor. The
"red" precondition is satisfied by grep-detectable presence of the unsafe pattern
in the pre-refactor baseline; the implementer's exit gate greps must confirm
elimination. The traditional red-first TDD cycle does not apply to this story class.

## Stubs Created

### STORY-181: NO-STUB-REQUIRED

- Story is MODIFY-only: `on_data` body refactor at `src/analyzer/enip.rs` (unsafe
  split-borrow elimination) and an advisory docstring update in `bin/`.
- `todo!()` stubs were considered and rejected — a stub would alter observable
  behavior in a behavior-preserving story, defeating the regression guard.
- Pre-refactor baseline compile verified clean at commit `421bf572`:
  `cargo test --all-targets` = 2667 passed / 0 failed / 5 ignored.

## Red Gate Verification

### STORY-181 — N/A-BY-DESIGN (Refactor Story)

The standard Red Gate (write failing tests, confirm all fail before implementation)
does not apply to STORY-181 because:

1. The story's only behavioral obligation is behavioral equivalence — no new
   observable output, no new behavioral contracts, no new assertions.
2. A red-first source-inspection test was considered (precedent:
   `tests/bc_149_single_borrow_invariant_tests.rs`) and explicitly rejected:
   `clippy -D warnings` and the 184-test EtherNet/IP behavioral suite provide
   adequate regression guard; the story specification prohibits test-assertion changes.
3. The "red" signal is the grep-detectable presence of the unsafe pattern in the
   pre-refactor source, not a failing test. The implementer's exit gate must show
   the pattern absent after the refactor.

**Pre-refactor unsafe pattern locations (to be eliminated by implementer):**

| Location | Detail |
|----------|--------|
| `src/analyzer/enip.rs:992–995` | `*mut EnipFlowState` raw pointer dereference (unsafe split-borrow) |
| `src/analyzer/enip.rs:998` | `#[allow(clippy::ptr_as_ptr)]` suppression attribute |
| `src/analyzer/enip.rs:999` | `unsafe { ... }` call site |
| `src/analyzer/enip.rs:979, 986` | SAFETY comments supporting the unsafe block |

**Regression guard (pre-existing; must remain 100 % passing after refactor):**

| Test Set | Count | Pre-refactor Status |
|----------|-------|---------------------|
| EtherNet/IP analyzer suite (`enip_analyzer_tests`) | 184 | all pass |
| `frame_walk` BC-2.17.016 tests | 28 | all pass (subset of 184) |
| `direction_and_clock` tests | 2 | all pass (subset of 184) |
| VP-033 proptests | 2 | all pass (subset of 184) |
| `f6_boundary_hardening` tests | 3 | all pass (subset of 184) |
| Full suite (`cargo test --all-targets`) | 2667 | 2667 passed / 0 failed / 5 ignored |

### Full cargo test baseline (pre-refactor, commit 421bf572)

```
cargo test --all-targets
running tests: 2667 passed / 0 failed / 5 ignored
```

## Regression Check

| Test Set | Status |
|----------|--------|
| 2667 pre-existing tests at baseline commit 421bf572 | all pass |
| 184 EtherNet/IP analyzer tests | all pass |

The pre-refactor baseline serves as the Red Gate equivalent for this story class.
A green baseline confirms no pre-existing breakage; the implementer's exit gate
must show the identical pass/fail counts post-refactor.

## Hand-Off to Implementer

- Stories ready for implementation: STORY-181
- Implementation guidance:
  - Target: `src/analyzer/enip.rs`, `on_data` method body, lines 979–999 region.
  - Eliminate the `*mut EnipFlowState` split-borrow pattern by restructuring the
    borrow so that the mutable reference to one field and the immutable reference
    to another field do not overlap within the same borrow scope (e.g., extract the
    needed value before taking the mutable borrow, or split the struct fields).
  - Remove `#[allow(clippy::ptr_as_ptr)]` at line 998 and the `unsafe { ... }` block
    at line 999.
  - Remove or revise the SAFETY comments at lines 979 and 986 that document the
    (now-eliminated) unsafe invariant.
  - Update the advisory docstring in `bin/` if present (AC-181-002).
  - Exit gate: `grep -n 'unsafe' src/analyzer/enip.rs` must not match the
    eliminated site; `cargo clippy --all-targets -- -D warnings` must pass with
    zero warnings; `cargo test --all-targets` must show 2667 passed / 0 failed /
    5 ignored (or higher if other stories land concurrently).
  - No test files are to be modified — the story explicitly forbids test-assertion
    changes.
