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

# BC-2.05.002: HTTP Method Prefix Routes Flow to HTTP

## Description

When the leading bytes of reassembled TCP content match one of the known HTTP method prefix
strings, the flow is routed to `DispatchTarget::Http`. This check is performed inside the
`classify` function and is evaluated AFTER the TLS check (INV-2). The method prefix list is:
`b"GET "`, `b"POST "`, `b"PUT "`, `b"DELETE "`, `b"HEAD "`, `b"OPTIONS "`, `b"PATCH "`,
`b"CONNECT "`, `b"TRACE "`, and `b"HTTP/"` (for response-first flows). The check uses
`data.starts_with(prefix)`.

## Preconditions

1. The TCP flow has not previously been classified.
2. The TLS content signature check has FAILED (data does not start with `0x16 0x03`).
3. `data.starts_with` one of the 10 HTTP method/version prefix byte strings.

## Postconditions

1. Returns `DispatchTarget::Http`.
2. The route is cached in `routes: HashMap<FlowKey, DispatchTarget>`.
3. The data is forwarded to the HttpAnalyzer (if configured).
4. All subsequent `on_data` calls use the cached Http route.

## Invariants

1. HTTP content signature check is UNREACHABLE for data starting with `0x16 0x03` (INV-2).
2. `b"HTTP/"` handles server-initiated or response-first flows (unusual but possible).
3. The method prefix strings include a trailing space (e.g., `b"GET "` not `b"GET"`). A
   buffer containing only `b"GET"` (3 bytes, no space) does NOT match.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | data starts with b"GET " on port 443 | Routed to Http (content wins over port 443 TLS hint) |
| EC-002 | data starts with b"POST " | Routed to Http |
| EC-003 | data starts with b"HTTP/" (response) | Routed to Http |
| EC-004 | data = b"GET" (no trailing space, 3 bytes) | No HTTP match; falls through to port fallback |
| EC-005 | data starts with b"get " (lowercase) | No match (case-sensitive); falls to port fallback |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| bytes b"GET /index HTTP/1.1...", port=8081 | DispatchTarget::Http | happy-path |
| bytes b"POST /submit HTTP/1.1...", port=80 | DispatchTarget::Http | happy-path |
| bytes b"HTTP/1.1 200 OK...", port=9999 | DispatchTarget::Http (response-first) | edge-case |
| bytes b"GET" (no space), port=9999 | Not Http (no match; falls to port fallback -> None) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | HTTP method prefix routes to Http regardless of port | unit: test_dispatcher_routes_http |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 -- HTTP method prefix routing is the second content-first dispatch rule per ADR 0001 |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- HTTP check is second after TLS) |
| Architecture Module | SS-05 (dispatcher.rs:95-107, C-21) |
| Stories | S-TBD |
| Origin BC | BC-DSP-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.001 -- related to (TLS check is first; HTTP check only reached when TLS bytes absent)
- BC-2.05.003 -- related to (port fallback is reached when both content checks fail)
- BC-2.05.005 -- composes with (Http result is cached after this match)

## Architecture Anchors

- `src/dispatcher.rs:95-107` -- HTTP method prefix check in classify function
- `tests/dispatcher_tests.rs` -- test_dispatcher_routes_http

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:95-107` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `data.starts_with(b"GET ")` etc.
- **assertion**: dispatcher tests verify HTTP routing on non-standard port

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates routes HashMap (cached result) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
