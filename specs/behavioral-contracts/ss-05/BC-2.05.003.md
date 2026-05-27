---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - v1.3: Pass-2 BC pre-merge re-anchor (per DF-SIBLING-SWEEP-001 v2 codified W11.L1) — updated stale test citation test_dispatcher_port_fallback_short_data (non-existent) → test_port_fallback_443_to_tls, test_port_fallback_8443_to_tls, test_port_fallback_80_to_http, test_port_fallback_8080_to_http (the four tests that cover port fallback in STORY-031). Discovered in sibling sweep. — 2026-05-27
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.003: Port Fallback: 443/8443->TLS, 80/8080->HTTP When Content Insufficient

## Description

When both the TLS content signature and the HTTP method prefix checks fail (either because
the data does not match either pattern, or because data.len() < 5 for the TLS check), the
`classify` function falls back to port-based heuristics. If `flow_key.lower_port()` or
`flow_key.upper_port()` is 443 or 8443, the result is `DispatchTarget::Tls`. If the port is
80 or 8080, the result is `DispatchTarget::Http`. Port checks are evaluated via
`flow_key.lower_port()` and `flow_key.upper_port()` -- the canonically ordered pair from the
FlowKey (INV-1).

## Preconditions

1. TLS content signature check failed (data not starting with `0x16 0x03`, OR data.len() < 5).
2. HTTP method prefix check failed (data does not start with any known method).
3. `flow_key.lower_port()` or `flow_key.upper_port()` is in {443, 8443, 80, 8080}.

## Postconditions

1. For ports 443 or 8443: returns `DispatchTarget::Tls`.
2. For ports 80 or 8080: returns `DispatchTarget::Http`.
3. Returned result is cached in `routes` (non-None results are cached, per BC-2.05.005).
4. Data is forwarded to the matched analyzer.

## Invariants

1. TLS port check (443/8443) is evaluated before HTTP port check (80/8080) per source order.
2. Port lookup uses `flow_key.lower_port()` and `flow_key.upper_port()` -- the canonical
   ordered ports; a flow on (src=8443, dst=9000) is checked as (lower=8443, upper=9000),
   so 8443 is found in the `ports` slice.
3. INV-2: content-first takes precedence. Port fallback is only reached when BOTH content
   checks fail.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | data.len() == 4 (too short for TLS), port=443 | TLS fallback: DispatchTarget::Tls |
| EC-002 | data = b"random bytes", port=80 | HTTP fallback: DispatchTarget::Http |
| EC-003 | Non-HTTP data on port 8080 | DispatchTarget::Http (port fallback) |
| EC-004 | data = b"GET " (would match HTTP content), port=443 | DispatchTarget::Http (content wins; port fallback unreachable) |
| EC-005 | data.len() == 4, port=8443 | DispatchTarget::Tls |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| data=b"\x00\x01" (non-TLS, non-HTTP), port=443 | DispatchTarget::Tls | happy-path |
| data=b"\x00\x01", port=80 | DispatchTarget::Http | happy-path |
| data=b"\x00\x01", port=8443 | DispatchTarget::Tls | happy-path |
| data=b"\x00\x01", port=8080 | DispatchTarget::Http | happy-path |
| data.len()=4 (short), port=443 | DispatchTarget::Tls (TLS content check skipped; port fallback) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-004 | Port fallback fires when content check fails | unit: test_port_fallback_443_to_tls, test_port_fallback_8443_to_tls, test_port_fallback_80_to_http, test_port_fallback_8080_to_http |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 -- port-based fallback is the third and last classification rule per ADR 0001 |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- port fallback is LAST, after both content checks fail) |
| Architecture Module | SS-05 (dispatcher.rs:108-116, C-21) |
| Stories | STORY-031 |
| Origin BC | BC-DSP-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.001 -- related to (TLS content check is first; port fallback only reached if it fails)
- BC-2.05.002 -- related to (HTTP content check is second; port fallback only reached if both fail)
- BC-2.05.004 -- related to (unknown port + unknown content = None, no fallback)

## Architecture Anchors

- `src/dispatcher.rs:108-116` -- port fallback in classify function
- `tests/dispatcher_tests.rs` -- test_port_fallback_443_to_tls, test_port_fallback_8443_to_tls, test_port_fallback_80_to_http, test_port_fallback_8080_to_http

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:108-116` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: port contains check with `[lower_port, upper_port]` slice
- **assertion**: test_port_fallback_443_to_tls, test_port_fallback_8443_to_tls, test_port_fallback_80_to_http, test_port_fallback_8080_to_http

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates routes HashMap (cached result) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
