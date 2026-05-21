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

# BC-2.02.011: Classify Other IP Protocols as Protocol::Other(byte)

## Description

When `build_parsed` encounters a transport layer that is neither TCP, UDP, nor ICMP (e.g.,
GRE, OSPF, ESP), it falls through to the `None` arm in the transport match. The IP protocol
number byte is wrapped in `Protocol::Other(u8)` and `TransportInfo::None` is set. This
catch-all preserves the IP addresses and protocol number while gracefully degrading when no
transport-layer detail is available.

## Preconditions

1. An IP packet is decoded with a transport protocol other than TCP, UDP, ICMPv4, or ICMPv6.
2. etherparse returns `transport: None` (no transport slice matched).
3. `ip_protocol` in the `IpTriple` contains the raw IP protocol number.

## Postconditions

1. `ParsedPacket.protocol` is `Protocol::Other(ip_protocol.0)` where `ip_protocol.0` is the
   raw u8 IP protocol number.
2. `ParsedPacket.transport` is `TransportInfo::None`.
3. `ParsedPacket.payload` is `Vec::new()`.
4. `app_protocol_hint()` returns `None`.
5. IP addresses are still correctly set.

## Invariants

1. The `Other(u8)` variant preserves the raw protocol byte from the IP header.
2. This is the fallback arm: any transport not explicitly matched becomes `Protocol::Other`.
3. Snaplen-truncated packets where the transport header was cut off (but the IP header was
   preserved by lax parsing) also produce `Protocol::Other` -- the protocol number is
   available even though no transport detail was decoded.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | GRE encapsulation (IP proto 47) | Protocol::Other(47), TransportInfo::None |
| EC-002 | ESP (IP proto 50) | Protocol::Other(50), TransportInfo::None |
| EC-003 | Snaplen-truncated TCP where transport header was cut | Protocol::Other(6) -- TCP proto number -- with TransportInfo::None |
| EC-004 | Protocol number 0 (HOPOPT, rare) | Protocol::Other(0) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| IPv4 packet with IP protocol 47 (GRE) | Protocol::Other(47), TransportInfo::None | happy-path |
| IPv4/TCP snaplen-cut at transport header | Protocol::Other(6), TransportInfo::None | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | IP protocol byte is preserved in Protocol::Other | unit: build GRE IP packet, assert Other(47) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 |
| Capability Anchor Justification | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 -- Protocol::Other is the CAP-03 degraded-but-safe decode result for unrecognized IP protocols |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-004 |
| Origin BC | BC-DEC-011 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.010 -- related to (ICMP is a special case before the Other fallback)

## Architecture Anchors

- `src/decoder.rs:285` -- `None => (Protocol::Other(ip_protocol.0), TransportInfo::None)`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:285` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: None arm in match on transport wraps ip_protocol byte

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
