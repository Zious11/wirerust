---
document_type: story
story_id: "STORY-031"
epic_id: "E-3"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.001.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.003.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-021]
blocks: [STORY-032, STORY-033]
behavioral_contracts:
  - BC-2.05.001
  - BC-2.05.002
  - BC-2.05.003
verification_properties: [VP-004]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 12
target_module: src/dispatcher.rs
subsystems: [SS-05]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield
---

> **Execute:** `/vsdd-factory:deliver-story STORY-031`

# STORY-031: Content-First Classification — TLS Signature, HTTP Method Prefix, Port Fallback

## Narrative
- **As a** forensic analyst
- **I want to** have TCP flows classified by their payload content first (TLS record type then HTTP method prefix) and only fall back to port-based heuristics when content is insufficient
- **So that** port-obfuscation attacks — such as running TLS on port 80 or HTTP on port 443 — are identified by what the data actually is, not by what port convention suggests

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.05.001 | TLS Content Signature Routes Flow to TLS Regardless of Port |
| BC-2.05.002 | HTTP Method Prefix Routes Flow to HTTP |
| BC-2.05.003 | Port Fallback: 443/8443->TLS, 80/8080->HTTP When Content Insufficient |

## Acceptance Criteria

### AC-001 (traces to BC-2.05.001 postcondition 1)
When the first bytes of reassembled TCP content have `data.len() >= 5 AND data[0] == 0x16 AND data[1] == 0x03`, the `classify` function returns `DispatchTarget::Tls` regardless of the flow's port numbers.
- **Test:** `test_dispatcher_routes_tls`

### AC-002 (traces to BC-2.05.001 invariant 2-3)
Content-first dispatch (INV-2) takes precedence: a TLS ClientHello on port 80 is routed to Tls, not Http. Only bytes 0 and 1 are checked (`0x16 0x03`); byte 2 (minor version) and bytes 3-4 (record length) are NOT checked.
- **Test:** `test_dispatcher_content_detection_tls_on_port_80`

### AC-003 (traces to BC-2.05.001 precondition 2)
When `data.len() < 5`, the TLS content check is skipped (falls through to HTTP check or port fallback).
- **Test:** `test_dispatcher_port_fallback_short_data`

### AC-004 (traces to BC-2.05.002 postcondition 1)
When the TLS content check fails and `data.starts_with` one of the 10 HTTP method/version prefix byte strings (`b"GET "`, `b"POST "`, `b"PUT "`, `b"DELETE "`, `b"HEAD "`, `b"OPTIONS "`, `b"PATCH "`, `b"CONNECT "`, `b"TRACE "`, `b"HTTP/"`), the `classify` function returns `DispatchTarget::Http`.
- **Test:** `test_dispatcher_routes_http`

### AC-005 (traces to BC-2.05.002 invariant 2-3)
`b"HTTP/"` handles server-initiated or response-first flows. Method prefix strings include a trailing space — `b"GET"` (3 bytes, no space) does NOT match. The comparison is case-sensitive; `b"get "` does not match.
- **Test:** `test_http_no_space_does_not_match`

### AC-006 (traces to BC-2.05.002 invariant 1)
The HTTP content signature check is evaluated AFTER the TLS check (INV-2). Data starting with `0x16 0x03` cannot trigger the HTTP match because TLS check is first.
- **Test:** `test_tls_takes_priority_over_http_methods_check`

### AC-007 (traces to BC-2.05.003 postcondition 1-2)
When both TLS and HTTP content checks fail, `classify` falls back to port-based heuristics: ports 443 or 8443 return `DispatchTarget::Tls`; ports 80 or 8080 return `DispatchTarget::Http`.
- **Test:** `test_dispatcher_port_fallback_short_data`

### AC-008 (traces to BC-2.05.003 invariant 1-2)
TLS port check (443/8443) is evaluated before HTTP port check (80/8080) per source order. Port lookup uses `flow_key.lower_port()` and `flow_key.upper_port()` — the canonically ordered pair — so a flow on (src=8443, dst=9000) finds 8443 in the port slice.
- **Test:** `test_port_fallback_uses_canonical_port_ordering`

### AC-009 (traces to BC-2.05.003 invariant 3)
Port fallback is only reached when BOTH content checks fail (INV-2). A valid HTTP GET request on port 443 is classified as Http by content, not as Tls by port.
- **Test:** `test_http_content_on_port_443_routes_to_http`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| classify (content check logic) | src/dispatcher.rs:90-116 | pure-core (pure classification logic, no state mutation) |
| StreamDispatcher.on_data | src/dispatcher.rs:120-160 | effectful-shell (calls classify, mutates routes cache) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | TLS ClientHello on port 80 | Routed to Tls (content wins over port) |
| EC-002 | TLS on port 443 | Routed to Tls via content signature |
| EC-003 | data starts with 0x16 0x03 but is not valid TLS | Routed to Tls; TlsAnalyzer handles parse error |
| EC-004 | data.len() == 4 (one byte short) | Falls through to HTTP method check |
| EC-005 | data[0] == 0x16 but data[1] != 0x03 | Falls through to HTTP check |
| EC-006 | b"GET /index HTTP/1.1" on port 8081 | Routed to Http |
| EC-007 | b"GET" (no space, 3 bytes) on port 9999 | Not matched; falls to port fallback; returns None (port unknown) |
| EC-008 | b"HTTP/1.1 200 OK" (response-first) on port 9999 | Routed to Http |
| EC-009 | Unknown bytes on port 443 | Routed to Tls (port fallback) |
| EC-010 | Unknown bytes on port 8080 | Routed to Http (port fallback) |
| EC-011 | b"GET " on port 443 | Routed to Http (content wins over port 443 hint) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/dispatcher.rs (classify function) | pure-core | No I/O, no global state mutation; deterministic |
| src/dispatcher.rs (on_data) | effectful-shell | Mutates routes HashMap via cache insert |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| Referenced code (dispatcher.rs:90-160) | ~3,000 |
| Test files (dispatcher_tests.rs) | ~3,000 |
| BC files (3 BCs) | ~3,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~13,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-009 (test-writer)
2. [ ] Verify Red Gate: all tests fail before implementation
3. [ ] Implement TLS content signature check per BC-2.05.001 (`data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03`; returns Tls regardless of port)
4. [ ] Implement HTTP method prefix check per BC-2.05.002 (10 prefixes including trailing space; `data.starts_with`; case-sensitive; after TLS check)
5. [ ] Implement port fallback per BC-2.05.003 (443/8443->Tls, 80/8080->Http; TLS ports checked before HTTP ports; only reached when both content checks fail)
6. [ ] Confirm TLS check takes priority over HTTP check
7. [ ] Confirm content classification takes priority over port fallback
8. [ ] Run all tests; verify all pass
9. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-3 | N/A | N/A | N/A |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Content-first takes precedence over port-based classification at all times (INV-2) | ADR 0001 / BC-2.05.001 invariant 2 | Unit tests: AC-002, AC-009 |
| TLS check (0x16 0x03) is first; HTTP check is second; port fallback is last | BC-2.05.002 invariant 1, BC-2.05.003 invariant 3 | Code review: source order in classify function |
| Loose TLS gate: only bytes 0 and 1 are checked; bytes 2-4 are NOT checked | BC-2.05.001 invariant 3 | Code review: confirm no additional byte checks |
| HTTP method prefixes include trailing space (e.g., `b"GET "` not `b"GET"`) | BC-2.05.002 invariant 3 | Code review: confirm prefix strings |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | slice::starts_with, indexing |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/dispatcher.rs | modify | classify function (90-116): TLS check, HTTP prefix check, port fallback |
| tests/dispatcher_tests.rs | modify | Add: test_dispatcher_routes_tls, test_dispatcher_content_detection_tls_on_port_80, test_dispatcher_routes_http, test_dispatcher_port_fallback_short_data, test_http_content_on_port_443_routes_to_http |
