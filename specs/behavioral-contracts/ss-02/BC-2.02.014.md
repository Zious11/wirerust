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

# BC-2.02.014: packet_len is Set to Total Frame Length, Not Just Payload Length

## Description

`ParsedPacket.packet_len` is set to `data.len()` -- the total number of bytes in the
captured frame passed to `decode_packet`, including all headers at every layer (Ethernet,
IP, TCP/UDP, and payload). It is NOT the payload length. This value is used by
`Summary::ingest` to accumulate `total_bytes`, which represents total wire bytes processed,
not application payload bytes.

## Preconditions

1. `decode_packet` is called with a `data` slice of any length.
2. Decode succeeds and returns `Ok(ParsedPacket)`.

## Postconditions

1. `ParsedPacket.packet_len == data.len()`.
2. `packet_len` is set in `build_parsed` by passing `data.len()` as the third argument:
   strict path at decoder.rs:145, lax path at decoder.rs:161.
3. `packet_len` is independent of whether the packet has a payload or not.

## Invariants

1. `packet_len` is always the full frame length, never a partial or payload-only length.
2. For snaplen-truncated captures, `packet_len` is the captured (truncated) length, which
   is less than the on-wire length. There is no separate `on_wire_len` field.
3. `packet_len == 0` is possible if a zero-length slice is somehow decoded (very rare).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 1500-byte Ethernet frame | packet_len == 1500 |
| EC-002 | 60-byte minimum Ethernet frame (with padding) | packet_len == 60 |
| EC-003 | Snaplen-truncated at 100 bytes | packet_len == 100 (truncated, not on-wire 1500) |
| EC-004 | TCP ACK with no payload (e.g., 54 bytes total) | packet_len == 54 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 100-byte Ethernet/IPv4/TCP frame | packet_len == 100 | happy-path |
| 54-byte pure-ACK Ethernet frame | packet_len == 54 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | packet_len == data.len() for any decoded frame | proptest: generate frames of varying lengths |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-03 ("Packet decoding") per domain/capabilities/cap-03-packet-decoding.md |
| Capability Anchor Justification | CAP-03 ("Packet decoding") per domain/capabilities/cap-03-packet-decoding.md -- packet_len is a decoded output field of the packet decoding layer |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-005 |
| Origin BC | BC-DEC-014 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.001 -- composes with (packet_len is set on all successful decode paths)

## Architecture Anchors

- `src/decoder.rs:255-259` -- `build_parsed` function signature: `fn build_parsed(ip: IpTriple, transport: &Option<TransportSlice<'_>>, packet_len: usize) -> ParsedPacket`
- `src/decoder.rs:142-146` -- strict path call site: `build_parsed(strict_ip_triple(net), &slice.transport, data.len())`
- `src/decoder.rs:161` -- lax path call site: `build_parsed(lax_ip_triple(net), &lax.transport, data.len())`
- `src/decoder.rs:300` -- `packet_len` field assignment in ParsedPacket construction

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:255-259, 142-146, 161` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: packet_len field is set unconditionally from data.len()

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
