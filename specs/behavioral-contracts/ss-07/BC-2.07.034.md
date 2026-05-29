---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3 (2026-05-28): F-W16-S052-P5-001 anchor tightening — replaced coarse `718-724` with precise line citations in Invariant-1 prose, Architecture Anchors, and Source Evidence: done-check at tls.rs:721 (`let done = self.flows.get(flow_key).is_some_and(...)`), early return at tls.rs:723 (`return;`). Matches sibling BC-2.07.003 v1.3 precision. Verified against src/analyzer/tls.rs:718-724. Closes F-W16-S052-P5-001. — 2026-05-28"
  - "v1.4 (2026-05-29): F-DRIFT2A-001 + F-DRIFT2A-003 (LOW) — fixed stale capabilities.md §CAP-07 citation to domain/capabilities/cap-07-tls-analysis.md; corrected invariant-1 `if done { return; }` prose to cite guard line 722 and return line 723 separately (the guard `if done {` is at tls.rs:722, `return;` is at tls.rs:723)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.034: After Both Hellos Seen, on_data Short-Circuits

## Description

This BC restates BC-2.07.003 from the `on_data` method's perspective, emphasizing the
short-circuit mechanism at the entry of `on_data`. Before any buffering occurs, `on_data`
reads the flow's `done()` state. If `done() == true`, the method returns immediately
without acquiring a mutable reference to the flow state or copying any bytes. This is
a defensive optimization: no buffering, no parsing, no state mutation of any kind.

## Preconditions

1. A flow entry exists in `self.flows`.
2. `self.flows[flow_key].done() == true` (both hellos seen).

## Postconditions

1. `on_data` returns without modifying any state.
2. No bytes from `data` are appended to any buffer.
3. No call to `try_parse_records`.
4. No counter changes.

## Invariants

1. The `done` check is the FIRST operation in `on_data`, before the mutable borrow of
   the flow entry. The done-check is at tls.rs:721 (`let done = self.flows.get(flow_key).is_some_and(|s| s.done())`);
   the guard `if done {` is at tls.rs:722; the `return;` is at tls.rs:723.
2. If `done()` is true, NO state mutation can occur for this flow for the lifetime
   of the `on_data` call.
3. This is a stronger statement than BC-2.07.003 which focuses on the per-record
   behavior AFTER buffering; BC-2.07.034 focuses on the pre-buffering short-circuit.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Large burst of data after both hellos | All bytes discarded; state unchanged |
| EC-002 | Flow not in self.flows map | `is_some_and` returns false; `done = false`; flow is created and data buffered normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello, ServerHello, then 1 MB of application data | All counters reflect only the two hellos; no parse_errors from app data | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | After done, on_data does not buffer or parse | unit: test_stop_after_handshake |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS Traffic Analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS Traffic Analysis") per domain/capabilities/cap-07-tls-analysis.md -- on_data short-circuit is a resource-bounding mechanism of TLS analysis |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:718-724; done-check at 721, early return at 723; C-13) |
| Stories | STORY-052 |
| Origin BC | BC-TLS-034 (pass-3 ingestion corpus, MEDIUM confidence -- exercised by test_stop_after_handshake) |

## Related BCs

- BC-2.07.003 -- composes with (same short-circuit viewed from the per-record level)

## Architecture Anchors

- `src/analyzer/tls.rs:718-724` -- `on_data` done-check and early return
- `src/analyzer/tls.rs:721` -- `let done = self.flows.get(flow_key).is_some_and(|s| s.done())` (done-check)
- `src/analyzer/tls.rs:722` -- `if done {` (guard)
- `src/analyzer/tls.rs:723` -- `return;` (early return when done)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:718-724` (done-check at 721; early return at 723) |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `let done = self.flows.get(flow_key).is_some_and(|s| s.done()); if done { return; }`
- **inferred**: co-pinned with test_stop_after_handshake

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads flows HashMap only (when done path taken) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (when done path taken) |
