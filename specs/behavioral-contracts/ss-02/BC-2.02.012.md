---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/decoder.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-02
capability: CAP-03
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

# BC-2.02.012: app_protocol_hint Returns Service Strings from Port Number

## Description

`ParsedPacket::app_protocol_hint` inspects `TransportInfo` for src and dst port numbers and
returns a `&'static str` naming the application-layer protocol when a port matches a known
service. The function implements a fixed table of 7 port-to-service mappings used for DNS
routing, summary statistics, and the content-first dispatch fallback (BC-2.05.003).

## Preconditions

1. `ParsedPacket` was constructed from a successful decode.
2. `transport` is `TransportInfo::Tcp { .. }` or `TransportInfo::Udp { .. }`.

## Postconditions

1. Returns `Some(&'static str)` if either port (src or dst) matches an entry in the table.
2. Returns `None` if neither port matches.
3. The 7 recognized ports and their return values are:
   - 53 (src or dst) => "DNS"
   - 80 (src or dst) => "HTTP"
   - 443 (src or dst) => "TLS"
   - 22 (src or dst) => "SSH"
   - 445 (src or dst) => "SMB"
   - 502 (src or dst) => "Modbus"
   - 20000 (src or dst) => "DNP3"
4. When both ports match different entries, the `(src, dst)` tuple match order determines
   which entry wins (match is ordered; first matching arm wins).

## Invariants

1. The function is deterministic and stateless; same ports always return the same string.
2. The return value is `Option<&'static str>` -- no heap allocation.
3. Port matching covers both src and dst: `(53, _) | (_, 53) => Some("DNS")`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | src=53, dst=9999 | Some("DNS") |
| EC-002 | src=9999, dst=53 | Some("DNS") |
| EC-003 | src=80, dst=443 | Some("HTTP") (80 arm fires before 443 arm in match order) |
| EC-004 | src=9999, dst=9999 | None |
| EC-005 | src=8080, dst=0 | None (8080 not in table) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Tcp { src_port: 53, dst_port: 12345 } | Some("DNS") | happy-path |
| Tcp { src_port: 443, dst_port: 55123 } | Some("TLS") | happy-path |
| Tcp { src_port: 5555, dst_port: 5555 } | None | edge-case |
| Udp { src_port: 53, dst_port: 53 } | Some("DNS") | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | All 7 recognized ports return the correct string | unit: test_app_protocol_hint_port_map |
| — | Unknown ports return None | proptest: generate ports 0..65535, filter out known ports, assert None |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 |
| Capability Anchor Justification | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 -- app_protocol_hint is a decode-layer enrichment function providing the service-name label downstream consumers (dispatcher, summary) need |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-004 |
| Origin BC | BC-DEC-012 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.013 -- composes with (TransportInfo::None causes early return from this function)
- BC-2.05.003 -- composes with (dispatcher fallback uses app_protocol_hint for port-based routing)

## Architecture Anchors

- `src/decoder.rs:94-116` -- app_protocol_hint implementation

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:94-116` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: static match arms with &'static str return values

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed. The port table is intentionally small; exhaustive listing aids
formal verification of the None path.
