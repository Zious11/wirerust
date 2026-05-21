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

# BC-2.02.001: Decode Ethernet-framed IPv4 TCP Packet to ParsedPacket

## Description

When `decode_packet` receives a raw frame with `DataLink::ETHERNET`, it invokes
`SlicedPacket::from_ethernet` to strip the Ethernet header, then extracts IPv4 source and
destination addresses plus TCP port and flag fields into a `ParsedPacket`. The payload slice
is set to the TCP segment payload bytes (not the full frame). This is the primary decode path
for most real-world captures.

## Preconditions

1. `data` is a valid Ethernet II frame containing an IPv4 packet containing a TCP segment.
2. `datalink` argument is `DataLink::ETHERNET`.
3. IPv4 `total_length` field is consistent with the captured byte slice (not a snaplen-truncated frame).

## Postconditions

1. Returns `Ok(ParsedPacket)` with no error.
2. `ParsedPacket.src_ip` and `dst_ip` are `IpAddr::V4` values matching the IPv4 header source and destination.
3. `ParsedPacket.protocol` is `Protocol::Tcp`.
4. `ParsedPacket.transport` is `TransportInfo::Tcp { src_port, dst_port, seq_number, syn, ack, fin, rst }` with values from the TCP header.
5. `ParsedPacket.payload` contains the TCP segment payload bytes (post-header bytes).
6. `ParsedPacket.packet_len` equals `data.len()` (the total frame length in bytes).

## Invariants

1. `packet_len` is always the total captured frame length (`data.len()`), not just the payload length.
2. TCP control flags (`syn`, `ack`, `fin`, `rst`) are extracted from the TCP header exactly as etherparse reports them.
3. No application-layer parsing is performed at this layer.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TCP SYN packet (syn=true, ack=false) | TransportInfo::Tcp with syn=true, ack=false, fin=false, rst=false |
| EC-002 | TCP SYN-ACK packet | TransportInfo::Tcp with syn=true, ack=true |
| EC-003 | TCP payload is empty (e.g., pure ACK) | payload is Vec::new() or empty slice; Ok still returned |
| EC-004 | IPv4 packet with TTL=1 | TTL not surfaced; decode proceeds normally |
| EC-005 | Frame with VLAN tag (802.1Q) | etherparse handles; may or may not surface IP depending on version |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Ethernet/IPv4/TCP SYN frame bytes | Ok(ParsedPacket { protocol: Tcp, transport: Tcp { syn: true, ... } }) | happy-path |
| Ethernet/IPv4/TCP frame with 100-byte payload | payload.len() == 100 | happy-path |
| Ethernet/IPv4/TCP RST frame | TransportInfo::Tcp { rst: true } | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Ethernet/IPv4/TCP decodes to correct src/dst IP and ports | unit: build synthetic Ethernet frame, assert fields |
| — | packet_len == data.len() for all decode paths | proptest: generate frames of varying lengths |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 -- this BC specifies the primary Ethernet decode path of the link-type-gated packet decoder |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | S-TBD |
| Origin BC | BC-DEC-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.003 -- related to (RAW/IPV4 path decodes same IPv4/TCP content, different link header)
- BC-2.02.015 -- composes with (TCP flags/seq are surfaced by this same decode path)
- BC-2.02.014 -- composes with (packet_len set here)

## Architecture Anchors

- `src/decoder.rs:128-172` -- decode_packet dispatch and strict parse path
- `src/decoder.rs:255-302` -- build_parsed assembles ParsedPacket from ip triple + transport

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:128-172` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: etherparse SlicedPacket::from_ethernet enforces structure
- **guard clause**: match on net layer; Err if no IP layer

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (operates on in-memory byte slice) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (takes &[u8], returns owned value) |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed -- suitable for formal verification. The function is a pure slice-to-struct transformation.
