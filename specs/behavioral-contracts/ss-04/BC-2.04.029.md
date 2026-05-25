---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/lifecycle.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave 8 STORY-019 adv-pass-1 F-4 closure: PC5 enforcement-mode notation (atomic state via automated test; no-second-eprintln sub-property via code review of swap-guarded if-block at lifecycle.rs:42-50, mirroring BC-2.04.048 PC2 / inv-3 / ADR-0004 amendment precedent). Added new PC for the close_flow_missing_warned_for_testing + reset_close_flow_missing_warned_for_testing + trigger_close_flow_missing_key_for_testing test-seam accessors (#[doc(hidden)] hygiene; replicate-body rationale due to production debug_assert per PC6) — 2026-05-25"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.029: close_flow for Missing Key Logs One-Shot Process-Wide Warning

## Description

When `TcpReassembler::close_flow` is invoked with a `FlowKey` that does not exist in
`self.flows`, it emits a one-shot `eprintln!` warning naming the missing key and the close
reason, then returns early without modifying any other state. The `CLOSE_FLOW_MISSING_WARNED`
atomic ensures this warning fires at most once per process, preventing stderr flooding from a
recurring bug.

## Preconditions

1. `close_flow(key, reason, handler)` is called.
2. `key` is NOT present in `self.flows`.
3. This is the first occurrence (or a subsequent occurrence after the first warning already fired).

## Postconditions

1. No flow is closed; `self.flows` is unmodified.
2. No `handler.on_flow_close` callback is issued.
3. No memory accounting change (`self.total_memory` unchanged).
4. If `CLOSE_FLOW_MISSING_WARNED` was `false` before the call: it is set to `true` and
   `eprintln!` fires with a message containing the key and reason.
5. If `CLOSE_FLOW_MISSING_WARNED` was already `true`: silent return, no eprintln.
   (Enforcement: the atomic-state latching property (`CLOSE_FLOW_MISSING_WARNED` remains `true`) is automated-test-verifiable via `close_flow_missing_warned_for_testing()` (see STORY-019 AC-014 combined test); the "no `eprintln!` on subsequent calls" sub-property is enforced structurally by the swap-guarded `if`-block at `src/reassembly/lifecycle.rs:42-50` and verified by code review, matching the BC-2.04.048 PC2 / invariant 3 enforcement-mode precedent and the ADR-0004 amendment.)
6. A `debug_assert!(false, ...)` fires in debug builds (expected to surface in test runs).
7. **PC7 (Test Seam):** A `#[doc(hidden)] pub fn close_flow_missing_warned_for_testing() -> bool`
   accessor in `src/reassembly/lifecycle.rs` exposes the current value of
   `CLOSE_FLOW_MISSING_WARNED` for integration-test verification. A companion
   `#[doc(hidden)] pub fn reset_close_flow_missing_warned_for_testing()` resets the atomic to
   `false` so tests can deterministically observe the PC4 `false → true` swap transition. A third
   `#[doc(hidden)] pub fn trigger_close_flow_missing_key_for_testing(...)` replicates the
   post-debug_assert body of the missing-key branch (atomic swap + one-shot eprintln) so tests can
   observe the post-call atomic state without panicking on the production `debug_assert!(false, ...)`
   per PC6 (cargo's default test profile is debug-mode). All three functions are `#[doc(hidden)]`
   to keep them out of public `cargo doc` output despite being on the `pub` API (integration tests
   are separate crates; `#[cfg(test)]` items are not visible).

## Invariants

1. `CLOSE_FLOW_MISSING_WARNED` is a `static AtomicBool`; once set to `true` it is never
   reset to `false` within the process lifetime (per ADR 0004 / one-shot warning pattern).
2. The warning is process-wide, not per-instance: multiple `TcpReassembler` instances share
   the same atomic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | close_flow called for missing key on first call | eprintln fires; atomic set to true |
| EC-002 | close_flow called for missing key a second time | Silent return; no second eprintln |
| EC-003 | close_flow called for missing key from two different reassembler instances | Only the first call across both instances fires the warning |
| EC-004 | close_flow called for a key that exists | Normal close path; no warning involved |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| close_flow for FlowKey not in flows, WARNED=false | eprintln fires, function returns, flows unchanged | happy-path |
| close_flow for FlowKey not in flows, WARNED=true | Silent return, no eprintln | edge-case |
| close_flow for FlowKey that IS in flows | Normal close behavior, no warning | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | close_flow for missing key never panics | manual: debug_assert is not panic in release mode |
| — | flows unchanged after missing-key close_flow call | manual/unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- the close_flow missing-key guard is a lifecycle defensive contract for TCP flow retirement |
| L2 Domain Invariants | INV-7 (Finalize-once latch -- this is a related lifecycle defensive pattern) |
| Architecture Module | SS-04 (reassembly/lifecycle.rs:42-50, C-15) |
| Stories | STORY-019 |
| Origin BC | BC-RAS-029 (pass-3 ingestion corpus, LOW confidence -- no direct test) |

## Related BCs

- BC-2.04.010 -- related to (RST close uses close_flow)
- BC-2.04.011 -- related to (FIN close uses close_flow)
- BC-2.04.012 -- related to (finalize uses close_flow for all remaining flows)

## Architecture Anchors

- `src/reassembly/lifecycle.rs:31` -- CLOSE_FLOW_MISSING_WARNED AtomicBool declaration
- `src/reassembly/lifecycle.rs:42-50` -- missing-key guard and one-shot eprintln

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/lifecycle.rs:42-50` |
| **Confidence** | low |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `let Some(mut flow) = self.flows.remove(key) else { ... }` at lifecycle.rs:42
- **documentation**: one-shot warning pattern per ADR 0004

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes to stderr (eprintln) on first miss |
| **Global state access** | reads + writes CLOSE_FLOW_MISSING_WARNED (AtomicBool) |
| **Deterministic** | no -- depends on prior process state (atomic) |
| **Thread safety** | atomic access is thread-safe |
| **Overall classification** | effectful shell (side-effectful stderr write) |
