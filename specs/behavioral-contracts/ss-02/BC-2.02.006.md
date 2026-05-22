---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - v1.3: Correct Architecture Anchors — lax_parse function spans 176-206 (not 184-205); add LINUX_SLL arm sub-range; fix Source Evidence path (STORY-003 m-1) — 2026-05-22
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.006: Decode Linux SLL (Cooked) TCP Packets

## Description

`DataLink::LINUX_SLL` captures (produced by `tcpdump -i any`) use the Linux cooked-capture
header format. `decode_packet` calls `SlicedPacket::from_linux_sll(data)` for the strict
path. When that fails with a length error (truncated capture), `lax_parse` uses a manual
16-byte header strip and `LaxSlicedPacket::from_ether_type` as a fallback, because
etherparse 0.16 has no `LaxSlicedPacket::from_linux_sll`. Successful decodes surface the
same `ParsedPacket` structure as Ethernet and RAW paths.

## Preconditions

1. `data` is a Linux SLL frame: 16-byte cooked header followed by IP payload.
2. `datalink` is `DataLink::LINUX_SLL`.
3. The SLL header is at least 16 bytes (otherwise strict parse fails with non-Len error, no lax retry).

## Postconditions

1. Returns `Ok(ParsedPacket)` with IP addresses, transport, and payload populated.
2. `ParsedPacket.packet_len` equals `data.len()`.
3. For TCP payloads: `protocol = Protocol::Tcp` and `TransportInfo::Tcp` with port/flags.
4. For UDP payloads: `protocol = Protocol::Udp` and `TransportInfo::Udp`.

## Invariants

1. SLL header length is exactly 16 bytes (`SLL_HEADER_LEN = 16`).
2. For snaplen-truncated SLL captures, lax recovery strips the first 16 bytes and extracts
   the EtherType from bytes [14..16] to invoke `LaxSlicedPacket::from_ether_type`.
3. An SLL frame shorter than 16 bytes fails the strict parse with a non-Len error and is
   rejected; the lax fallback is NOT invoked for such frames.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SLL frame with IPv4 TCP payload | Decoded; IpAddr::V4, Protocol::Tcp |
| EC-002 | SLL frame with IPv6 TCP payload | Decoded; IpAddr::V6, Protocol::Tcp |
| EC-003 | SLL frame snaplen-truncated at IP payload | Lax parse invoked; IP layer recovered; transport may be Protocol::Other |
| EC-004 | SLL frame < 16 bytes total | Strict parse fails with non-Len error; Err returned (not lax-retried) |
| EC-005 | SLL frame exactly 16 bytes (no payload) | Strict parse fails; likely "No IP layer found" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid SLL/IPv4/TCP frame | Ok(ParsedPacket { protocol: Tcp }) | happy-path |
| SLL/IPv6/UDP frame | Ok(ParsedPacket { protocol: Udp }) | happy-path |
| SLL frame with truncated IP payload (snaplen) | Ok (lax path) or Err depending on where truncation lands | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | LINUX_SLL frames decode successfully for IPv4 and IPv6 | unit: synthetic SLL frame bytes |
| — | Sub-16-byte SLL frame is rejected, never lax-retried | unit: assert Err on 15-byte frame |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 -- LINUX_SLL is one of the 5 accepted link types; its decode path is the subject of this BC |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-003 |
| Origin BC | BC-DEC-006 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.007 -- composes with (malformed input on the SLL path produces Err, not panic)

## Architecture Anchors

- `src/decoder.rs:135` -- `DataLink::LINUX_SLL => SlicedPacket::from_linux_sll(data)` -- LINUX_SLL strict dispatch
- `src/decoder.rs:176-206` -- `fn lax_parse` function (full span: signature at 176, closing brace at 206)
- `src/decoder.rs:184-203` -- LINUX_SLL arm within lax_parse: manual 16-byte header strip and `LaxSlicedPacket::from_ether_type` call
- `src/decoder.rs:119-121` -- SLL_HEADER_LEN constant

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:176-206` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `.get(SLL_HEADER_LEN - 2..SLL_HEADER_LEN).ok_or_else(...)` bounds check
- **documentation**: inline comment explains why LaxSlicedPacket::from_linux_sll is absent

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed. The manual SLL strip is defensive but correct; etherparse upstream
may add LaxSlicedPacket::from_linux_sll in a future version, at which point this workaround
can be removed. The contract test at BC-2.02.006 serves as the regression anchor.
