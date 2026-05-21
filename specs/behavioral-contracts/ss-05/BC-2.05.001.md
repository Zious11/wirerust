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

# BC-2.05.001: TLS Content Signature Routes Flow to TLS Regardless of Port

## Description

When the first bytes of reassembled TCP content satisfy `data.len() >= 5 AND data[0] == 0x16
AND data[1] == 0x03`, the flow is routed to `DispatchTarget::Tls` regardless of the port
numbers. This is the primary differentiation of content-first dispatch (ADR 0001 and INV-2):
an attacker running TLS on port 80 is still identified as TLS, not HTTP.

## Preconditions

1. The TCP flow has not previously been classified (no cached route for this FlowKey).
2. At least 5 bytes of reassembled data are available for inspection.
3. `data[0] == 0x16` (TLS record type: ContentType::Handshake).
4. `data[1] == 0x03` (TLS major version byte: TLS 1.x or SSL 3.0).

## Postconditions

1. Returns `DispatchTarget::Tls`.
2. The route is cached in `routes: HashMap<FlowKey, DispatchTarget>`.
3. The data is forwarded to the TlsAnalyzer (if configured).
4. All subsequent `on_data` calls for this FlowKey use the cached Tls route without re-inspecting.

## Invariants

1. TLS content signature wins over HTTP content signature. A data buffer starting with
   `0x16 0x03` can never match an HTTP method prefix because HTTP methods start with ASCII
   letters (0x41-0x5A), not 0x16. The HTTP check is unreachable for TLS data.
2. INV-2: content-first takes precedence over port-based classification at all times.
3. Loose TLS gate (Smell #10): only bytes 0 and 1 are checked. Byte 2 (minor version) and
   bytes 3-4 (record length) are NOT checked. Any data beginning with `0x16 0x03 <any>...`
   is routed to TLS.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TLS ClientHello on port 80 (non-standard) | Routed to TLS (content wins over port) |
| EC-002 | TLS on port 443 (standard) | Routed to TLS via content signature |
| EC-003 | Data starts with 0x16 0x03 but is not valid TLS | Routed to TLS; TlsAnalyzer handles the parse error |
| EC-004 | data.len() == 4 (one byte short) | Falls through to HTTP method check; TLS NOT matched |
| EC-005 | data[0] == 0x16 but data[1] != 0x03 | Falls through to HTTP method check |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| bytes [0x16, 0x03, 0x03, 0x00, 0x50, ...], port=8080 | DispatchTarget::Tls | happy-path |
| bytes [0x16, 0x03, 0x01, ...], port=80 | DispatchTarget::Tls (content wins over port 80) | happy-path |
| bytes [0x16, 0x03, ...], len=4 | Not TLS (length check fails); HTTP method or port fallback | edge-case |
| bytes [0x47, 0x45, 0x54, ...] (b"GET "), port=443 | DispatchTarget::Http (GET wins; not TLS bytes) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | 0x16,0x03 prefix always routes to TLS regardless of port | unit: test_dispatcher_content_detection_tls_on_port_80 |
| VP-004 | TLS classification is cached after first match | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-first protocol dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-first protocol dispatch") per capabilities.md §CAP-05 -- TLS content signature routing is the primary dispatch rule per ADR 0001 |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence) |
| Architecture Module | SS-05 (dispatcher.rs:90-116, C-21) |
| Stories | S-TBD |
| Origin BC | BC-DSP-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.002 -- related to (HTTP check is only reached when TLS bytes are absent)
- BC-2.05.003 -- related to (port fallback is reached when both content checks fail)
- BC-2.05.005 -- composes with (Tls result is cached after this match)

## Architecture Anchors

- `src/dispatcher.rs:90` -- `fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget`
- `src/dispatcher.rs:92-94` -- TLS check: `data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03`
- `tests/dispatcher_tests.rs` -- test_dispatcher_routes_tls, test_dispatcher_content_detection_tls_on_port_80

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:90-94` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: explicit if check on data[0] and data[1]
- **assertion**: dispatcher tests verify TLS routing on non-standard port

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates routes HashMap (cached result) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed (classification logic is pure; caching mutates state) |
