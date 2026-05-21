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

# BC-2.02.015: Extract TCP Control Flags and Sequence Number into TransportInfo::Tcp

## Description

When a TCP segment is decoded, `build_parsed` extracts six fields from the TCP header into
`TransportInfo::Tcp`: `src_port`, `dst_port`, `seq_number`, and the four control flags
(`syn`, `ack`, `fin`, `rst`). The `seq_number` is the 32-bit sequence number from the TCP
header. These fields are used by the TCP reassembly engine to drive the state machine and
detect anomalies.

## Preconditions

1. An IP packet with a TCP transport layer is decoded.
2. etherparse surfaces the TCP header via `TransportSlice::Tcp(tcp)`.

## Postconditions

1. `TransportInfo::Tcp.src_port` equals the TCP source port (2 bytes, network byte order, converted to host).
2. `TransportInfo::Tcp.dst_port` equals the TCP destination port.
3. `TransportInfo::Tcp.seq_number` equals the 32-bit sequence number from the TCP header.
4. `TransportInfo::Tcp.syn` is `true` iff the SYN control bit is set.
5. `TransportInfo::Tcp.ack` is `true` iff the ACK control bit is set.
6. `TransportInfo::Tcp.fin` is `true` iff the FIN control bit is set.
7. `TransportInfo::Tcp.rst` is `true` iff the RST control bit is set.
8. `ParsedPacket.payload` contains the TCP segment payload (bytes after the TCP header).

## Invariants

1. Sequence number extraction uses `tcp.to_header().sequence_number` (etherparse API).
2. All four flags are extracted unconditionally; they are `bool` fields.
3. The PSH and URG flags are NOT extracted (not needed by any current consumer).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SYN packet (SYN=1, ACK=0) | syn=true, ack=false |
| EC-002 | SYN-ACK (SYN=1, ACK=1) | syn=true, ack=true |
| EC-003 | RST packet | rst=true |
| EC-004 | FIN-ACK | fin=true, ack=true |
| EC-005 | Sequence number 0xFFFFFFFF (wraparound boundary) | seq_number == 4294967295 |
| EC-006 | TCP with 0-byte payload (pure ACK) | payload is empty vec |
| EC-007 | All flags unset (data segment) | syn=false, ack=false, fin=false, rst=false |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| TCP SYN frame | TransportInfo::Tcp { syn: true, ack: false, seq_number: N } | happy-path |
| TCP data frame with 50-byte payload | payload.len() == 50, syn=false | happy-path |
| TCP RST frame | rst=true, payload empty | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | All 4 TCP flags extracted correctly | unit: build frames with each flag combination |
| — | seq_number matches raw TCP header bytes | unit: compare with known-good frame |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 |
| Capability Anchor Justification | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 -- TCP flag/sequence extraction is the core L4 decoding specified in CAP-03 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | S-TBD |
| Origin BC | BC-DEC-015 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.001 -- composes with (this BC specifies what TransportInfo::Tcp contains)
- BC-2.04.004 -- depends on (TCP reassembly uses syn flag from this struct)

## Architecture Anchors

- `src/decoder.rs:263-274` -- TransportSlice::Tcp arm in build_parsed

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:263-274` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: etherparse TcpHeaderSlice API provides typed accessors

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed. The PSH/URG exclusion is intentional; adding them would require a
struct change and all call sites to be updated. Only add if a consumer requires them.
