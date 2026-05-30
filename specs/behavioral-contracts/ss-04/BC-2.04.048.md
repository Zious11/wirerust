---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/segment.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v1.3: Wave 7 STORY-014 adv-pass-1 F-2 closure: explicit enforcement-mode notation on PC2 (atomic state via automated test; no-second-eprintln sub-property via code review, matching invariant 3 precedent). Added new PC for the `isn_missing_warned_for_testing` / `reset_isn_missing_warned_for_testing` test-seam accessors with the `#[doc(hidden)]` hygiene rationale. — 2026-05-25"
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.048: ISN_MISSING_WARNED Atomic Prevents Repeated eprintln

## Description

The `ISN_MISSING_WARNED` static `AtomicBool` in `segment.rs` ensures that the
"insert_segment called with no ISN set" `eprintln!` fires at most once per process, regardless
of how many times `insert_segment` is called with `isn == None`. This is the process-wide
one-shot warning pattern described in ADR 0004. Once set to `true`, the atomic is never reset.

## Preconditions

1. `insert_segment` is called with `isn == None`.
2. `ISN_MISSING_WARNED` is currently `false` (first occurrence) OR `true` (subsequent).

## Postconditions

On first IsnMissing encounter:
1. `ISN_MISSING_WARNED.swap(true, Ordering::Relaxed)` returns `false` (was not warned).
2. `eprintln!("wirerust: insert_segment called with no ISN set")` fires to stderr.
3. `ISN_MISSING_WARNED` is now `true`.

On subsequent IsnMissing encounters:
1. `ISN_MISSING_WARNED.swap(true, Ordering::Relaxed)` returns `true` (already warned).
2. No eprintln. (Enforcement: the atomic-state latching property is automated-test-verifiable via
   `isn_missing_warned_for_testing()` (see STORY-014 AC-014 combined test); the "no `eprintln!` on
   subsequent calls" sub-property is enforced structurally by the swap-guarded `if`-block at
   `src/reassembly/segment.rs:51-58` and verified by code review, matching the BC-2.04.048
   invariant 3 enforcement precedent.)
3. Both paths return `InsertResult::IsnMissing`.

**PC4 (Test Seam):** A `#[doc(hidden)] pub fn isn_missing_warned_for_testing() -> bool` accessor
in `src/reassembly/segment.rs` exposes the current value of `ISN_MISSING_WARNED` for
integration-test verification. A companion `#[doc(hidden)] pub fn reset_isn_missing_warned_for_testing()`
resets the atomic to `false` so tests can deterministically observe the PC1 `false → true` swap
transition. Both functions are `#[doc(hidden)]` to keep them out of public `cargo doc` output
despite being on the `pub` API (required because integration tests are separate crates and
`#[cfg(test)]` items are not visible to them).

## Invariants

1. `ISN_MISSING_WARNED` is a `static AtomicBool` -- process-wide, not per-instance.
   Multiple `FlowDirection` objects or multiple `TcpReassembler` instances share the same
   atomic.
2. `Ordering::Relaxed` is sufficient because the atomic is only used as a one-shot warning
   guard, not for synchronization with other memory.
3. Per ADR 0004, this is the standard one-shot warning pattern used for programming-error
   detection. The `swap(true)` approach (rather than `load` + conditional `store`) avoids
   a TOCTOU race in hypothetical concurrent scenarios.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First IsnMissing in process | eprintln fires; atomic set |
| EC-002 | Second IsnMissing in same process (different flow) | Silent; no eprintln |
| EC-003 | IsnMissing triggered from CLOSE_FLOW_MISSING_WARNED in lifecycle.rs | Separate atomic -- does not affect ISN_MISSING_WARNED |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| First insert_segment call with isn=None | IsnMissing; eprintln once | happy-path |
| Second insert_segment call with isn=None (same process) | IsnMissing; no eprintln | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | ISN_MISSING_WARNED prevents second eprintln | manual / process-level test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- the ISN_MISSING_WARNED atomic is the observability mechanism for a programming-error guard in TCP reassembly |
| L2 Domain Invariants | INV-6 (related -- the one-shot warning pattern prevents stderr flooding, analogous to the one-shot warning for finalize-skip) |
| Architecture Module | SS-04 (reassembly/segment.rs:16, C-8) |
| Stories | STORY-014 |
| Origin BC | BC-RAS-048 (pass-3 ingestion corpus, LOW confidence -- no direct test) |

## Related BCs

- BC-2.04.032 -- composes with (IsnMissing is the triggering condition for this atomic)
- BC-2.04.029 -- related to (CLOSE_FLOW_MISSING_WARNED is a parallel one-shot warning in lifecycle.rs)

## Architecture Anchors

- `src/reassembly/segment.rs:16` -- ISN_MISSING_WARNED AtomicBool declaration
- `src/reassembly/segment.rs:53-57` -- swap-based one-shot guard

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:16,53-57` |
| **Confidence** | low |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **documentation**: ADR 0004 -- process-wide warning atomics for one-shot bug tripwires
- **guard clause**: `if !ISN_MISSING_WARNED.swap(true, Ordering::Relaxed) { eprintln!(...) }`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes to stderr on first encounter |
| **Global state access** | reads + writes ISN_MISSING_WARNED (AtomicBool) |
| **Deterministic** | no -- depends on prior process state |
| **Thread safety** | atomic access is thread-safe |
| **Overall classification** | effectful shell |
