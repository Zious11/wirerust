---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-12T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-16
capability: CAP-16
lifecycle_status: active
introduced: v0.7.0-feature-arp
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
  - .factory/phase-f1-delta-analysis/mitre-arp-research.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.001: ARP Request Frame Correctly Parsed from ArpPacketSlice

## Description

`extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>`
produces a correctly-populated `ArpFrame` with `operation == 1` when given an etherparse
`ArpPacketSlice` constructed from a well-formed Ethernet/IPv4 ARP Request (opcode 0x0001)
frame. All six address fields are copied exactly from the `ArpPacketSlice` accessors. This
is the happy-path accept contract for ARP Request parsing and is the primary VP-024 Sub-property
A target.

## Preconditions

1. `arp` is an `ArpPacketSlice<'_>` successfully constructed by etherparse 0.20 from a
   28-byte (minimum) Ethernet/IPv4 ARP payload.
2. `arp.hw_addr_type() == ArpHardwareId::ETHERNET` (value 0x0001).
3. `arp.proto_addr_type() == EtherType::IPV4` (value 0x0800).
4. `arp.hw_addr_size() == 6` (Ethernet MAC length).
5. `arp.proto_addr_size() == 4` (IPv4 address length).
6. `arp.operation().0 == 1` (ARP Request opcode).
7. `outer_src_mac` is either `Some([u8; 6])` (Ethernet capture) or `None` (non-Ethernet link, e.g. SLL).
8. `packet_len` is the total on-wire frame length in bytes.

## Postconditions

1. `extract_arp_frame(arp, outer_src_mac, packet_len)` returns `Some(ArpFrame { ... })`.
2. `ArpFrame.operation == 1` (ARP Request).
3. `ArpFrame.sender_mac` equals the first 6 bytes returned by `arp.sender_hw_addr()`, copied exactly.
4. `ArpFrame.sender_ip` equals the first 4 bytes returned by `arp.sender_protocol_addr()`, copied exactly.
5. `ArpFrame.target_mac` equals the first 6 bytes returned by `arp.target_hw_addr()`, copied exactly.
   (In a well-formed ARP Request, target MAC is typically all-zero — the requesting host does not
   know the target MAC.)
6. `ArpFrame.target_ip` equals the first 4 bytes returned by `arp.target_protocol_addr()`, copied exactly.
7. `ArpFrame.outer_src_mac` equals the `outer_src_mac` parameter passed in unchanged.
8. `ArpFrame.packet_len` equals the `packet_len` parameter passed in unchanged.
9. The function NEVER panics for any `ArpPacketSlice` satisfying preconditions 2–6.
10. The function performs no I/O and mutates no external state.

## Invariants

1. **Field copy fidelity**: every address field in `ArpFrame` is a byte-exact copy of the
   corresponding `ArpPacketSlice` accessor output. No endian conversion, no transformation.
   `ArpPacketSlice` accessor return types (`&[u8]` for MAC/IP fields, newtype u16 for operation)
   are confirmed for etherparse 0.20.1 against live docs.rs (ADR-008 §Source).
2. **ARP Request canonical target MAC**: in standard ARP Requests the target MAC field is zeroed
   (`[0x00; 6]`). The extractor copies whatever bytes the slice contains without validation —
   this BC documents the expected value; the detection layer validates anomalies.
3. **Purity**: `extract_arp_frame` is a pure core function — no I/O, no global state, no side
   effects. It is the Kani formal-verification target for VP-024 Sub-property A.
4. **Extraction agnosticism**: the function does not check the ARP opcode. Whether opcode is 1
   (Request), 2 (Reply), or any other value, extraction proceeds identically as long as the
   hw/proto type and size fields meet preconditions 2–5. The classification of opcode meaning
   belongs to the detection layer (`ArpAnalyzer::process_arp`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Minimum 28-byte Ethernet/IPv4 ARP Request buffer | `Some(ArpFrame)` — all six address fields decoded correctly |
| EC-002 | Target MAC is `[0x00; 6]` (standard ARP Request) | `Some(ArpFrame { target_mac: [0,0,0,0,0,0], ... })` — zero target MAC copied faithfully |
| EC-003 | `outer_src_mac` is `None` (SLL capture, no Ethernet header) | `Some(ArpFrame { outer_src_mac: None, ... })` — D12 mismatch check will be skipped by analyzer |
| EC-004 | `outer_src_mac` is `Some(mac)` where `mac != sender_mac` | `Some(ArpFrame { outer_src_mac: Some(mac), ... })` — D12 mismatch detected by analyzer (BC-2.16.007) |
| EC-005 | `operation == 0` (undefined opcode) | `Some(ArpFrame { operation: 0, ... })` — extraction proceeds; analyzer handles undefined opcode as non-Request/non-Reply |
| EC-006 | Sender IP is all-zero (ARP probe per RFC 5227) | `Some(ArpFrame { sender_ip: [0,0,0,0], ... })` — probe detection is a detection-layer concern, not extraction |
| EC-007 | `hw_addr_size == 8` (non-Ethernet hardware address) | `None` — precondition 4 violated; returns None per BC-2.16.009 (malformed) path |
| EC-008 | `proto_addr_size == 16` (IPv6 protocol address) | `None` — precondition 5 violated; returns None per BC-2.16.009 path |

## Canonical Test Vectors

| Input (ARP fields) | `outer_src_mac` | Expected `ArpFrame` fields | Category |
|---|---|---|---|
| op=1, sender_mac=AA:BB:CC:DD:EE:FF, sender_ip=192.168.1.10, target_mac=00:00:00:00:00:00, target_ip=192.168.1.1, pkt_len=42 | Some([0xAA,0xBB,0xCC,0xDD,0xEE,0xFF]) | operation=1, sender_mac=[0xAA,0xBB,0xCC,0xDD,0xEE,0xFF], sender_ip=[192,168,1,10], target_mac=[0,0,0,0,0,0], target_ip=[192,168,1,1], outer_src_mac=Some([0xAA,0xBB,0xCC,0xDD,0xEE,0xFF]), packet_len=42 | happy-path: standard ARP who-has request |
| op=1, sender_mac=DE:AD:BE:EF:00:01, sender_ip=10.0.0.2, target_mac=00:00:00:00:00:00, target_ip=10.0.0.1, pkt_len=60 | None | operation=1, sender_mac=[0xDE,0xAD,0xBE,0xEF,0x00,0x01], outer_src_mac=None, packet_len=60 | SLL capture — outer MAC absent |
| op=0, sender_mac=AA:00:00:00:00:01, sender_ip=0.0.0.0, target_mac=00:00:00:00:00:00, target_ip=192.168.1.5, pkt_len=42 | Some([0xAA,0x00,0x00,0x00,0x00,0x01]) | operation=0, sender_ip=[0,0,0,0] — extracted faithfully; analyzer skips unknown opcode | edge-case: RFC 5227 ARP probe (sender IP zeroed) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Sub-property A (ARP frame extraction parse safety): `extract_arp_frame` never panics for any valid `ArpPacketSlice`; returns `Some(ArpFrame)` with correct field values for Ethernet/IPv4 inputs | Kani: symbolic `[u8; 28]` buffer with HTYPE/PTYPE/HLEN/PLEN fixed for Ethernet/IPv4; OPER+addrs symbolic; field correctness assertions |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — ARP Request parsing is the foundational extraction step that feeds all ARP security detections (D1/D2/D3/D11/D12); no analysis is possible without correctly extracting the sender and target fields from the ArpPacketSlice |
| L2 Domain Invariants | (none directly — pure extraction function with no finding emission) |
| Architecture Module | SS-16 (src/decoder.rs `extract_arp_frame`, C-23 ArpAnalyzer); ADR-008 Decision 2 |
| Stories | TBD (F3 story decomposition) |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — pure extraction function; no finding emission) |

## Related BCs

- BC-2.16.002 — composes with (ARP Reply extraction — same extractor, operation==2 variant)
- BC-2.16.009 — depends on (malformed frame: extractor returns None when hw/proto size fields fail checks)
- BC-2.16.015 — composes with (decode-vs-analysis separation: this extraction always occurs; analysis is gated separately)

## Architecture Anchors

- `src/decoder.rs` — `fn extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>`
- `src/decoder.rs` — `pub struct ArpFrame { operation, sender_mac, sender_ip, target_mac, target_ip, outer_src_mac, packet_len }`
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 2` — ArpFrame struct layout and extraction function signature
- `.factory/specs/architecture/arp-architecture-delta.md §2.1` — ArpFrame struct field types

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-024 — ARP Frame Parse Safety and Binding-Table Invariant (Sub-property A: extraction safety and field-copy correctness for Request path)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 2; arp-architecture-delta.md §2.1; etherparse 0.20.1 ArpPacketSlice accessor names confirmed live docs.rs 2026-06-12 |
| **Confidence** | high — ArpPacketSlice accessor method names (sender_hw_addr, target_hw_addr, sender_protocol_addr, target_protocol_addr, operation, hw_addr_type, proto_addr_type) confirmed from live docs.rs fetch (ADR-008 §Source) |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same ArpPacketSlice always produces same ArpFrame |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-024 Kani target (Sub-A) |
