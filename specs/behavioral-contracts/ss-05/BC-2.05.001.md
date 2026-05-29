---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - v1.3: Pass-2 BC pre-merge re-anchor (per DF-SIBLING-SWEEP-001 v2 codified W11.L1) — updated test name test_dispatcher_routes_tls → test_tls_content_wins_over_port_8080 + test_tls_content_routes_tls_on_port_443 (renamed/split in STORY-031 pass-1); updated classify line range 90-116 → 90-117 to match actual function close. Closes F-W12P2-002, F-W12P2-004. — 2026-05-27
  - v1.4: Pass-4 anchor-completeness sweep (DF-SIBLING-SWEEP-001 v2, doctrine application extended from pass-3 BC-2.05.002 to siblings BC-2.05.001 + BC-2.05.003). Added test_tls_check_skipped_below_len_5 (PC2 boundary at len=4, EC-004), test_tls_check_requires_byte1_equals_0x03 (PC4 specificity, EC-005), test_tls_takes_priority_over_http_methods_check (INV-1 ordering) to VP-004 table and Architecture Anchors. Closes F-W12P4-001. — 2026-05-27
  - v1.5: W12-D2 EC table inline test citations added (DF-SIBLING-SWEEP-001 v3) — added `covered by` test citations to EC-001 through EC-005 matching sibling BC-2.05.002 style. Closes W12-D2. — 2026-05-28
  - v1.6: F-DRIFT2A-001 — fixed stale capabilities.md §CAP-05 citation to domain/capabilities/cap-05-content-first-dispatch.md in L2 Capability and Capability Anchor Justification rows. — 2026-05-29
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
| EC-001 | TLS ClientHello on port 80 (non-standard) | Routed to TLS (content wins over port); covered by `test_dispatcher_content_detection_tls_on_port_80` |
| EC-002 | TLS on port 443 (standard) | Routed to TLS via content signature; covered by `test_tls_content_routes_tls_on_port_443` |
| EC-003 | Data starts with 0x16 0x03 but is not valid TLS | Routed to TLS; TlsAnalyzer handles the parse error; covered by `test_tls_content_wins_over_port_8080` (non-standard port variant) |
| EC-004 | data.len() == 4 (one byte short) | Falls through to HTTP method check; TLS NOT matched; covered by `test_tls_check_skipped_below_len_5` |
| EC-005 | data[0] == 0x16 but data[1] != 0x03 | Falls through to HTTP method check; covered by `test_tls_check_requires_byte1_equals_0x03` |

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
| VP-004 | 0x16,0x03 prefix always routes to TLS regardless of port | unit: test_dispatcher_content_detection_tls_on_port_80, test_tls_content_wins_over_port_8080, test_tls_content_routes_tls_on_port_443 |
| VP-004 | TLS length guard: data.len() < 5 skips TLS check (PC2 boundary) | unit: test_tls_check_skipped_below_len_5 |
| VP-004 | data[1] must equal 0x03 for TLS match (PC4 specificity, EC-005) | unit: test_tls_check_requires_byte1_equals_0x03 |
| VP-004 | TLS check evaluated BEFORE HTTP check (INV-1 ordering) | unit: test_tls_takes_priority_over_http_methods_check |
| VP-004 | TLS classification is cached after first match | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per domain/capabilities/cap-05-content-first-dispatch.md -- TLS content signature routing is the primary dispatch rule per ADR 0001 |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence) |
| Architecture Module | SS-05 (dispatcher.rs:90-117, C-21) |
| Stories | STORY-031 |
| Origin BC | BC-DSP-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.002 -- related to (HTTP check is only reached when TLS bytes are absent)
- BC-2.05.003 -- related to (port fallback is reached when both content checks fail)
- BC-2.05.005 -- composes with (Tls result is cached after this match)

## Architecture Anchors

- `src/dispatcher.rs:90` -- `fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget`
- `src/dispatcher.rs:92-94` -- TLS check: `data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03`
- `tests/dispatcher_tests.rs` -- test_tls_content_wins_over_port_8080, test_tls_content_routes_tls_on_port_443, test_dispatcher_content_detection_tls_on_port_80, test_tls_check_skipped_below_len_5, test_tls_check_requires_byte1_equals_0x03, test_tls_takes_priority_over_http_methods_check

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
