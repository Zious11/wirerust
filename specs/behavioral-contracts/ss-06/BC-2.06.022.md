---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/http.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-06
capability: CAP-06
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3 (2026-06-13): P19-B-08 ss-06 line-anchor re-sync — MAX_HEADER_BUF :21→:23; buffer cap in on_data :513-529→:532-565. Verified against current src/analyzer/http.rs (1044 lines)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.022: Per-Direction Header Buffer Capped at MAX_HEADER_BUF (65536)

## Description

For each `on_data` call, the HttpAnalyzer calculates `remaining = MAX_HEADER_BUF - buf.len()`
for the relevant direction buffer. Only `min(data.len(), remaining)` bytes are appended to
the buffer. Once the buffer reaches `MAX_HEADER_BUF = 65,536` bytes, no further bytes are
written for that direction. Bytes past the cap are silently dropped. A request whose header
exceeds the cap will never parse to completion and will be silently discarded at flow close.

## Preconditions

1. The direction buffer has been partially filled.
2. `on_data` is called with new bytes that would push the buffer past `MAX_HEADER_BUF`.

## Postconditions

1. Only `remaining` bytes are appended (the `min` clamping).
2. The direction buffer size does not exceed `MAX_HEADER_BUF = 65,536` bytes.
3. No error is emitted and no counter is incremented for the dropped bytes.
4. If no complete header can be assembled within the cap, the partial bytes remain buffered
   and are silently discarded on flow close.

## Invariants

1. `MAX_HEADER_BUF = 65,536` (http.rs:23). This is the same value as TLS's `MAX_BUF`.
2. Buffer overflow is silently clamped via `min`; no panic, no error.
3. The cap applies per-direction independently (request_buf and response_buf have separate caps).
4. No finding is emitted when the cap is reached.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | on_data fills buffer to exactly 65536 | Buffer holds 65536 bytes; next on_data appends 0 |
| EC-002 | Buffer at 65535; on_data sends 100 bytes | 1 byte appended; 99 bytes dropped silently |
| EC-003 | Buffer at cap; poisoned (remaining=0, poisoned=false) | Bytes still subject to cap; no parse happens |
| EC-004 | Large header that would require 100KB | Max 65536 buffered; header never completes; discarded on close |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Multiple on_data calls accumulating 65536 bytes without header terminator | Buffer at cap; no parse; no error | edge-case |
| on_data pushing buffer to 65537 | Buffer stays at 65536; 1 byte dropped | edge-case |
| Normal request < 65536 bytes | Parses correctly within cap | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Buffer cap prevents OOM from large headers | unit: test_buffer_cap_no_panic_on_oversized_headers |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- header buffer cap is the memory-bounding mechanism for HTTP analysis |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:532-565, C-12) |
| Stories | STORY-045 |
| Origin BC | BC-HTTP-022 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.003 -- composes with (partial buffering is bounded by this cap)

## Architecture Anchors

- `src/analyzer/http.rs:532-565` -- buffer cap logic in on_data
- `src/analyzer/http.rs:23` -- MAX_HEADER_BUF constant definition
- `tests/http_analyzer_tests.rs` -- test_buffer_cap_no_panic_on_oversized_headers

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:23, 532-565` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `let remaining = MAX_HEADER_BUF.saturating_sub(state.request_buf.len()); if remaining > 0 { state.request_buf.extend_from_slice(&data[..data.len().min(remaining)]); }`
- **assertion**: test_buffer_cap_no_panic_on_oversized_headers

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates request_buf/response_buf |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
