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

# BC-2.02.010: Classify ICMP as Protocol::Icmp with TransportInfo::None

## Description

When the IP packet contains an ICMP (ICMPv4) or ICMPv6 payload, `build_parsed` maps it to
`Protocol::Icmp` and sets `TransportInfo::None`. No port numbers are extracted (ICMP has
none). The `app_protocol_hint` method returns `None` for ICMP packets because `TransportInfo::None`
short-circuits the port lookup. ICMP is recognized as its own protocol but carries no
transport-layer detail.

## Preconditions

1. An IP packet with protocol number ICMP (1) or ICMPv6 (58) is being decoded.
2. etherparse surfaces the ICMP payload as `TransportSlice::Icmpv4` or `TransportSlice::Icmpv6`.

## Postconditions

1. `ParsedPacket.protocol` is `Protocol::Icmp`.
2. `ParsedPacket.transport` is `TransportInfo::None`.
3. `ParsedPacket.payload` is `Vec::new()` (empty; ICMP body bytes are not extracted).
4. `app_protocol_hint()` returns `None`.

## Invariants

1. `Protocol::Icmp` is produced for BOTH ICMPv4 and ICMPv6 -- there is no separate
   `Protocol::Icmpv6` variant; they map to the same enum value.
2. `TransportInfo::None` means no port extraction; `app_protocol_hint` always returns None
   when transport is None (see BC-2.02.013).
3. ICMP payload bytes are NOT included in `ParsedPacket.payload`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ICMP echo request (type 8) | Protocol::Icmp, TransportInfo::None |
| EC-002 | ICMP echo reply (type 0) | Protocol::Icmp, TransportInfo::None |
| EC-003 | ICMPv6 (type 135 -- neighbor solicitation) | Protocol::Icmp, TransportInfo::None |
| EC-004 | ICMP over IPv6 | IpAddr::V6, Protocol::Icmp, TransportInfo::None |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| IPv4 ICMP echo-request bytes | Ok({ protocol: Icmp, transport: None, payload: [] }) | happy-path |
| IPv6 ICMPv6 bytes | Ok({ protocol: Icmp, src_ip: IpAddr::V6 }) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | ICMPv4 and ICMPv6 both produce Protocol::Icmp | unit: build synthetic ICMP frames |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-03 ("Packet decoding") per domain/capabilities/cap-03-packet-decoding.md |
| Capability Anchor Justification | CAP-03 ("Packet decoding") per domain/capabilities/cap-03-packet-decoding.md -- ICMP classification is part of the L2-L4 header parsing that CAP-03 describes; per architect CAP-03 is merged into SS-02 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-004 |
| Origin BC | BC-DEC-010 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.011 -- related to (Protocol::Other is the fallback for unknown IP protocols)
- BC-2.02.013 -- composes with (TransportInfo::None causes app_protocol_hint to return None)

## Architecture Anchors

- `src/decoder.rs:282-284` -- `Icmpv4 | Icmpv6 => (Protocol::Icmp, TransportInfo::None)`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:282-284` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: TransportSlice::Icmpv4 | Icmpv6 match arm

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
