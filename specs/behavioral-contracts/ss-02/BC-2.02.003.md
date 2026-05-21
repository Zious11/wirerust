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

# BC-2.02.003: Decode RAW Link-Layer IPv4 TCP Packet via from_ip

## Description

When `decode_packet` receives data with `DataLink::RAW` or `DataLink::IPV4`, it calls
`SlicedPacket::from_ip(data)` directly, skipping any link-layer header stripping. The raw
bytes begin at the IP header. This path is used for loopback captures and tunnel-based
captures where no Ethernet framing is present.

## Preconditions

1. `data` is a raw IP packet starting at the IPv4 header (no link-layer prefix).
2. `datalink` is `DataLink::RAW` or `DataLink::IPV4`.
3. IPv4 `total_length` is consistent with the captured bytes.

## Postconditions

1. Returns `Ok(ParsedPacket)` identical in structure to the Ethernet path (BC-2.02.001).
2. `ParsedPacket.src_ip` and `dst_ip` are `IpAddr::V4` values from the IPv4 header.
3. `ParsedPacket.protocol` is `Protocol::Tcp` for TCP payloads.
4. `ParsedPacket.transport`, `payload`, and `packet_len` are set identically to the Ethernet path.

## Invariants

1. `DataLink::RAW` and `DataLink::IPV4` invoke the same code path (`SlicedPacket::from_ip`).
2. No link-layer bytes are consumed; `data[0]` must be the IP version nibble (0x45 for IPv4).
3. The result is structurally identical to BC-2.02.001 for the same IP+TCP content.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DataLink::IPV4 with IPv4 TCP packet | Decoded identically to DataLink::RAW (same code branch) |
| EC-002 | DataLink::RAW with first byte = 0x60 (IPv6 version nibble) | etherparse from_ip handles IPv6; IpAddr::V6 returned |
| EC-003 | Empty slice | from_ip returns parse error; Err propagated |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| RAW IPv4 TCP bytes | Ok(ParsedPacket { protocol: Tcp }) | happy-path |
| IPV4-link IPv4 TCP bytes | Same result as RAW | happy-path |
| Malformed IP header bytes | Err (no panic) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | RAW and IPV4 link types produce identical output for same IP content | unit: call decode_packet twice with same bytes, different DataLink |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 -- RAW link-type handling is part of the 5-element link-type acceptance whitelist |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-002 |
| Origin BC | BC-DEC-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.004 -- supersedes with (DataLink::IPV4 decodes identically to DataLink::RAW) |
- BC-2.02.001 -- related to (same decode result for IP+TCP content; different link-layer entry)

## Architecture Anchors

- `src/decoder.rs:134` -- `DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data)`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:134` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: match arm covers RAW | IPV4 | IPV6 together

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
