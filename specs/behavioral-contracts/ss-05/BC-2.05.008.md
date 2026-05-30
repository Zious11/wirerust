---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - v1.3: W14 Pass 1 remediation: VP confidence uplift (inferred→concrete tests), Architecture Anchors concrete, Evidence Types overhaul — 2026-05-28
  - v1.4: W14-D2 EC-002 disambiguation — rewrote ambiguous EC-002 description to precisely state the input trigger (`http=Some, tls=None; TLS-signature bytes 0x16 0x03`) and the actual path taken (classify fires, returns DispatchTarget::Tls via content match, NOT port-fallback; tls=None so no data forwarded). Removes "may route None" ambiguity. Closes W14-D2. — 2026-05-28
  - v1.5: F-DRIFT2A-001 — fixed stale domain/capabilities/cap-05-content-first-dispatch.md citation to domain/capabilities/cap-05-content-first-dispatch.md in L2 Capability and Capability Anchor Justification rows. — 2026-05-29
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
| EC-002 | http=Some, tls=None; data starts with `0x16 0x03` (TLS content-signature bytes) | Not early-returned (guard requires BOTH None); classify fires and returns DispatchTarget::Tls via content match — port-fallback is NOT reached; tls=None so the Tls dispatch arm is a no-op (no data forwarded); route cached as Tls |
| EC-003 | http=None, tls=None; on_flow_close | Close still processes normally (no early return in on_flow_close) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| StreamDispatcher(None, None).on_data(...) | Returns immediately; routes empty | happy-path |
| StreamDispatcher(Some(http), None).on_data(TLS bytes) | Not early-returned; classify runs | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | No-analyzer dispatcher (both None) returns early from on_data; on_flow_close processes without incrementing unclassified | unit: STORY-033 test_BC_2_05_008_no_analyzer_dispatcher_early_returns |
| — | Single-analyzer dispatcher (one Some, one None) is NOT subject to early return; on_data runs classify and routes normally | unit: STORY-033 test_BC_2_05_008_single_analyzer_not_early_returned |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md -- no-analyzer early-return is the efficiency guard for unconfigured dispatchers |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- early return is pre-classification) |
| Architecture Module | SS-05 (dispatcher.rs:121-123, C-21) |
| Stories | STORY-033 |
| Origin BC | BC-DSP-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.004 -- related to (when analyzers ARE configured, None is the fallthrough)

## Architecture Anchors

- `src/dispatcher.rs:121-123` -- early-return guard in on_data
- `tests/dispatcher_tests.rs` -- test_BC_2_05_008_no_analyzer_dispatcher_early_returns (both-None early-return path directly verified), test_BC_2_05_008_single_analyzer_not_early_returned (one-sided dispatcher non-early-return verified)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:121-123` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.http.is_none() && self.tls.is_none() { return; }`
- **test-vector**: both-None early-return path directly verified by test_BC_2_05_008_no_analyzer_dispatcher_early_returns (unclassified_flows stays 0 after on_flow_close with both analyzers None); single-analyzer non-early-return verified by test_BC_2_05_008_single_analyzer_not_early_returned (http=Some/tls=None routes GET bytes; http=None/tls=Some routes TLS bytes)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (early return before any mutation) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (early-return path) |
