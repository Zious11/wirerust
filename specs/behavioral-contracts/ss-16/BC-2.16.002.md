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

# BC-2.16.002: ARP Reply Frame Correctly Parsed from ArpPacketSlice

## Description

`extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>`
produces a correctly-populated `ArpFrame` with `operation == 2` when given an etherparse
`ArpPacketSlice` from a well-formed Ethernet/IPv4 ARP Reply (opcode 0x0002). Unlike an ARP
Request, the Reply carries both sender and target hardware addresses as non-zero values (the
responder knows both). This is the happy-path accept contract for ARP Reply parsing and is
the second VP-024 Sub-property A target. The Reply path is the primary vehicle for ARP
spoofing (D1), Gratuitous ARP (D2), and D12 L2/L3 mismatch detections.

## Preconditions

1. `arp` is an `ArpPacketSlice<'_>` successfully constructed by etherparse 0.20 from a
   28-byte (minimum) Ethernet/IPv4 ARP payload.
2. `arp.hw_addr_type() == ArpHardwareId::ETHERNET` (value 0x0001).
3. `arp.proto_addr_type() == EtherType::IPV4` (value 0x0800).
4. `arp.hw_addr_size() == 6`.
5. `arp.proto_addr_size() == 4`.
6. `arp.operation().0 == 2` (ARP Reply opcode).
7. `outer_src_mac` is either `Some([u8; 6])` (Ethernet capture) or `None`.
8. `packet_len` is the total on-wire frame length in bytes.

## Postconditions

1. `extract_arp_frame(arp, outer_src_mac, packet_len)` returns `Some(ArpFrame { ... })`.
2. `ArpFrame.operation == 2` (ARP Reply).
3. `ArpFrame.sender_mac` equals the first 6 bytes returned by `arp.sender_hw_addr()`, copied exactly.
4. `ArpFrame.sender_ip` equals the first 4 bytes returned by `arp.sender_protocol_addr()`, copied exactly.
5. `ArpFrame.target_mac` equals the first 6 bytes returned by `arp.target_hw_addr()`, copied exactly.
   (In a well-formed ARP Reply, target MAC is the unicast MAC of the original requesting host.)
6. `ArpFrame.target_ip` equals the first 4 bytes returned by `arp.target_protocol_addr()`, copied exactly.
7. `ArpFrame.outer_src_mac` equals the `outer_src_mac` parameter passed in unchanged.
8. `ArpFrame.packet_len` equals the `packet_len` parameter passed in unchanged.
9. The function NEVER panics for any `ArpPacketSlice` satisfying preconditions 2–6.
10. The function performs no I/O and mutates no external state.

## Invariants

1. **Field copy fidelity**: every address field in `ArpFrame` is a byte-exact copy of the
   corresponding `ArpPacketSlice` accessor output. No endian conversion, no transformation.
2. **ARP Reply canonical target MAC**: in standard ARP Replies the target MAC field carries the
   MAC of the original requester. A zero or broadcast target MAC in a Reply is unusual and may
   indicate a Gratuitous ARP (GARP) variant. The extractor copies without validation.
3. **Purity and shared extractor**: BC-2.16.001 and BC-2.16.002 describe the same extractor
   function `extract_arp_frame` with different `operation` values. Both paths exercise the
   same code; the distinction is tested by separate canonical vectors.
4. **Sender == Target IP is the GARP trigger**: when `sender_ip == target_ip` on a Reply
   frame, the analyzer (BC-2.16.003) classifies this as a Gratuitous ARP. The extractor is
   agnostic to this condition.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Standard ARP Reply: responder fills target_mac with requester MAC | `Some(ArpFrame { operation: 2, target_mac: <requester MAC>, ... })` — extracted faithfully |
| EC-002 | Gratuitous ARP Reply: sender_ip == target_ip, target_mac may be broadcast | `Some(ArpFrame { sender_ip: X, target_ip: X, ... })` — GARP condition detected by analyzer (BC-2.16.003) |
| EC-003 | ARP Reply with outer_src_mac != sender_mac | `Some(ArpFrame { outer_src_mac: Some(different_mac), sender_mac: other_mac, ... })` — D12 L2/L3 mismatch detected by analyzer (BC-2.16.007) |
| EC-004 | `outer_src_mac` is None (SLL capture) | `Some(ArpFrame { outer_src_mac: None, ... })` — D12 check skipped |
| EC-005 | ARP Reply gratuitous variant: sender_ip == target_ip AND sender_ip already in binding table with different MAC | Extraction succeeds; both D2 (GARP) and D1 (spoof) findings emitted by analyzer (BC-2.16.014) |

## Canonical Test Vectors

| Input (ARP fields) | `outer_src_mac` | Expected `ArpFrame` fields | Category |
|---|---|---|---|
| op=2, sender_mac=11:22:33:44:55:66, sender_ip=192.168.1.1, target_mac=AA:BB:CC:DD:EE:FF, target_ip=192.168.1.10, pkt_len=42 | Some([0x11,0x22,0x33,0x44,0x55,0x66]) | operation=2, sender_mac=[0x11,0x22,0x33,0x44,0x55,0x66], sender_ip=[192,168,1,1], target_mac=[0xAA,0xBB,0xCC,0xDD,0xEE,0xFF], target_ip=[192,168,1,10], outer_src_mac=Some([0x11,0x22,0x33,0x44,0x55,0x66]), packet_len=42 | happy-path: standard ARP Reply (is-at) |
| op=2, sender_mac=DE:AD:00:00:00:01, sender_ip=10.0.0.1, target_mac=00:00:00:00:00:00, target_ip=10.0.0.1, pkt_len=42 | Some([0xDE,0xAD,0x00,0x00,0x00,0x01]) | operation=2, sender_ip=[10,0,0,1], target_ip=[10,0,0,1] — sender_ip == target_ip triggers GARP detection | GARP Reply: sender_ip == target_ip |
| op=2, sender_mac=AA:AA:AA:AA:AA:AA, sender_ip=10.0.0.1, target_mac=BB:BB:BB:BB:BB:BB, target_ip=10.0.0.2, pkt_len=42 | Some([0xFF,0xFF,0xFF,0xFF,0xFF,0xFF]) | operation=2, outer_src_mac=Some([0xFF,0xFF,0xFF,0xFF,0xFF,0xFF]) != sender_mac=[0xAA,0xAA,0xAA,0xAA,0xAA,0xAA] — D12 mismatch | D12 L2/L3 mismatch: outer MAC != ARP sender MAC |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Sub-property A (ARP frame extraction parse safety): `extract_arp_frame` never panics; returns `Some(ArpFrame)` with correct field values for Ethernet/IPv4 Reply inputs | Kani: same harness as BC-2.16.001 — OPER field symbolic covers both op=1 and op=2 paths |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — ARP Reply parsing is critical because ARP Replies are the primary vehicle for cache poisoning (D1), gratuitous ARP (D2), and L2/L3 mismatch (D12) detections; incorrect Reply extraction would cause silent false negatives on all three detection paths |
| L2 Domain Invariants | (none directly — pure extraction function with no finding emission) |
| Architecture Module | SS-16 (src/decoder.rs `extract_arp_frame`, C-23); ADR-008 Decision 2 |
| Stories | TBD (F3 story decomposition) |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — pure extraction function; findings emitted by ArpAnalyzer detection BCs) |

## Related BCs

- BC-2.16.001 — composes with (ARP Request extraction — same extractor, operation==1 variant)
- BC-2.16.003 — depends on (GARP detection uses Reply frames where sender_ip == target_ip)
- BC-2.16.004 — depends on (spoof detection updates binding table from Reply sender fields)
- BC-2.16.007 — depends on (D12 mismatch detection uses outer_src_mac vs sender_mac from Reply)
- BC-2.16.009 — depends on (malformed frame: extractor returns None when hw/proto size fields fail)

## Architecture Anchors

- `src/decoder.rs` — `fn extract_arp_frame(arp: &ArpPacketSlice<'_>, outer_src_mac: Option<[u8; 6]>, packet_len: usize) -> Option<ArpFrame>`
- `src/decoder.rs` — `pub struct ArpFrame { operation, sender_mac, sender_ip, target_mac, target_ip, outer_src_mac, packet_len }`
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 2`
- `.factory/specs/architecture/arp-architecture-delta.md §2.1`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-024 — ARP Frame Parse Safety and Binding-Table Invariant (Sub-property A: extraction safety and field-copy correctness, Reply path)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 2; arp-architecture-delta.md §2.1; RFC 826 §ARP Reply semantics |
| **Confidence** | high — ARP Reply field layout is fixed by RFC 826 and confirmed via etherparse 0.20.1 accessor names |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same ArpPacketSlice always produces same ArpFrame |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-024 Kani target (Sub-A) |
