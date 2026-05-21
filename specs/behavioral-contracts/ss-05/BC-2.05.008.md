---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/dispatcher.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-05
capability: CAP-05
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.008: No Analyzer Configured: Dispatcher Early-Returns

## Description

If `StreamDispatcher` is created with `http = None` and `tls = None`, then `on_data` returns
immediately at the very first check (dispatcher.rs:121-123) without running `classify`, without
updating any counters, and without touching the `routes` or `classification_attempts` maps.
This is the "no-op dispatcher" path for captures where neither HTTP nor TLS analysis is requested.

## Preconditions

1. `StreamDispatcher::new(None, None)` was called (both analyzers absent).
2. `on_data` is called for any FlowKey.

## Postconditions

1. Function returns immediately after the `if self.http.is_none() && self.tls.is_none()` check.
2. `routes` unchanged.
3. `classification_attempts` unchanged.
4. `unclassified_flows` unchanged.
5. No analyzer receives data.

## Invariants

1. The early-return guard is the FIRST statement in `on_data` (before route lookup and classify).
2. This guard does NOT affect `on_flow_close`; close events still clean up routes/attempts.
3. A dispatcher configured with only one analyzer (e.g., http=Some, tls=None) is NOT subject
   to this early return.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | http=None, tls=None; any data | on_data returns immediately |
| EC-002 | http=Some, tls=None; TLS data | Normal TLS check; falls to port fallback; may route None |
| EC-003 | http=None, tls=None; on_flow_close | Close still processes normally (no early return in on_flow_close) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| StreamDispatcher(None, None).on_data(...) | Returns immediately; routes empty | happy-path |
| StreamDispatcher(Some(http), None).on_data(TLS bytes) | Not early-returned; classify runs | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | No-analyzer dispatcher returns early from on_data | unit: (inferred from dispatcher creation patterns in tests) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 -- no-analyzer early-return is the efficiency guard for unconfigured dispatchers |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- early return is pre-classification) |
| Architecture Module | SS-05 (dispatcher.rs:121-123, C-21) |
| Stories | STORY-033 |
| Origin BC | BC-DSP-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.004 -- related to (when analyzers ARE configured, None is the fallthrough)

## Architecture Anchors

- `src/dispatcher.rs:121-123` -- early-return guard in on_data
- `tests/dispatcher_tests.rs` -- dispatcher tests that create one-sided dispatchers exercise the non-early path

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:121-123` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.http.is_none() && self.tls.is_none() { return; }`
- **inferred**: tests create one-sided dispatchers; no test exercises the both-None path directly

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (early return before any mutation) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (early-return path) |
