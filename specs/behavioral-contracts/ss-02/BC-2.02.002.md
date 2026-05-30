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
capability: CAP-02
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

# BC-2.02.002: Decode Ethernet-framed IPv4 UDP Packet with DNS Port Hint

## Description

When `decode_packet` receives an Ethernet-framed IPv4 UDP packet, it extracts the IPv4
addresses and UDP source/destination ports into a `ParsedPacket`. The `app_protocol_hint`
method on the returned `ParsedPacket` returns `Some("DNS")` when either port is 53,
providing the application-layer routing hint that the DNS analyzer uses via `can_decode`.
This BC covers the UDP decode path including the DNS-port case.

## Preconditions

1. `data` is a valid Ethernet II frame containing an IPv4 packet containing a UDP datagram.
2. `datalink` argument is `DataLink::ETHERNET`.
3. IPv4 `total_length` is consistent with the captured bytes.

## Postconditions

1. Returns `Ok(ParsedPacket)`.
2. `ParsedPacket.protocol` is `Protocol::Udp`.
3. `ParsedPacket.transport` is `TransportInfo::Udp { src_port, dst_port }`.
4. `ParsedPacket.payload` contains the UDP payload bytes.
5. `ParsedPacket.packet_len` equals `data.len()`.
6. When `dst_port == 53` or `src_port == 53`, `app_protocol_hint()` returns `Some("DNS")`.

## Invariants

1. `TransportInfo::Udp` carries only port numbers (no flags, no sequence number).
2. `app_protocol_hint()` checks both src and dst port for 53; either match returns DNS.
3. Payload for UDP is the UDP datagram body bytes (excluding UDP header).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | UDP dst port 53 (DNS query outbound) | app_protocol_hint() returns Some("DNS") |
| EC-002 | UDP src port 53 (DNS response inbound) | app_protocol_hint() returns Some("DNS") |
| EC-003 | UDP port 1234 (unknown service) | app_protocol_hint() returns None |
| EC-004 | UDP payload is empty | payload is Vec::new(); Ok still returned |
| EC-005 | UDP dst port 80 | app_protocol_hint() returns Some("HTTP") |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Ethernet/IPv4/UDP frame, dst_port=53 | Ok, protocol=Udp, app_hint=Some("DNS") | happy-path |
| Ethernet/IPv4/UDP frame, src_port=53 | Ok, protocol=Udp, app_hint=Some("DNS") | happy-path |
| Ethernet/IPv4/UDP frame, port=9999 | Ok, protocol=Udp, app_hint=None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | app_protocol_hint returns Some("DNS") iff port 53 is src or dst | unit: test_app_protocol_hint_dns |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per domain/capabilities/cap-02-link-type-gating.md |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per domain/capabilities/cap-02-link-type-gating.md -- UDP decode is part of the packet decoder that gates on link type |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-002 |
| Origin BC | BC-DEC-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.001 -- related to (same decode path, TCP variant)
- BC-2.02.012 -- composes with (app_protocol_hint uses port map)
- BC-2.08.001 -- composes with (DnsAnalyzer.can_decode reads port from TransportInfo::Udp)

## Architecture Anchors

- `src/decoder.rs:275-281` -- Udp arm in build_parsed
- `src/decoder.rs:94-116` -- app_protocol_hint port map

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:94-116` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: TransportSlice::Udp arm in match
- **guard clause**: match on src/dst port pair in app_protocol_hint

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed -- suitable for formal verification.
