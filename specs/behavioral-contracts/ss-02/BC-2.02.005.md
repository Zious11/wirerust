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

# BC-2.02.005: Decode RAW IPv6 TCP Packet Surfacing IPv6 Addresses

## Description

When `decode_packet` receives a RAW (or IPV6) link-type packet whose IP header is IPv6,
it extracts the 128-bit source and destination addresses as `IpAddr::V6` values. The TCP
transport layer is handled identically to the IPv4 path. This contract verifies that IPv6
is a first-class supported protocol, not a degraded path.

## Preconditions

1. `data` begins with an IPv6 header (version nibble 0x60).
2. `datalink` is `DataLink::RAW`, `DataLink::IPV4`, or `DataLink::IPV6`.
3. IPv6 `payload_length` field is consistent with the captured bytes.

## Postconditions

1. Returns `Ok(ParsedPacket)`.
2. `ParsedPacket.src_ip` is `IpAddr::V6` containing the IPv6 source address.
3. `ParsedPacket.dst_ip` is `IpAddr::V6` containing the IPv6 destination address.
4. `ParsedPacket.protocol` is `Protocol::Tcp` if the next header is TCP.
5. `ParsedPacket.transport` is `TransportInfo::Tcp` with correct port and flag values.
6. `ParsedPacket.packet_len` equals `data.len()`.

## Invariants

1. IPv6 address extraction uses etherparse's `Ipv6HeaderSlice::source_addr()` and `destination_addr()`.
2. Extension headers are handled by etherparse; the transport after extension headers is surfaced normally.
3. IPv6 is decoded via the same `from_ip` call as IPv4; no separate code path exists.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IPv6 with TCP payload | IpAddr::V6 src/dst, Protocol::Tcp |
| EC-002 | IPv6 with UDP payload | IpAddr::V6 src/dst, Protocol::Udp |
| EC-003 | IPv6 with ICMP (ICMPv6) | IpAddr::V6 src/dst, Protocol::Icmp, TransportInfo::None |
| EC-004 | IPv6 loopback address (::1) | Decoded normally; IpAddr::V6(::1) |
| EC-005 | IPv6 with extension headers | etherparse traverses extension headers; TCP/UDP surfaced |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| RAW/IPv6/TCP bytes | Ok(ParsedPacket { src_ip: IpAddr::V6(...), protocol: Tcp }) | happy-path |
| RAW/IPv6/UDP bytes dst_port=53 | Ok, protocol=Udp, app_hint=Some("DNS") | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | IPv6 src/dst addresses survive decode round-trip | unit: construct synthetic IPv6/TCP frame, assert IpAddr::V6 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 -- IPv6 support is part of the 5-element accepted link-type whitelist |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | S-TBD |
| Origin BC | BC-DEC-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.003 -- composes with (same from_ip call, IPv4 variant)

## Architecture Anchors

- `src/decoder.rs:209-228` -- `strict_ip_triple`: IPv4 arm at 211-218, IPv6 arm at 219-227
- `src/decoder.rs:231-250` -- `lax_ip_triple`: IPv4 arm at 233-240, IPv6 arm at 241-249

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:209-250` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: NetSlice::Ipv6 arm in strict_ip_triple / lax_ip_triple

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed.
