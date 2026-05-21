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

# BC-2.02.004: DataLink::IPV4 Decodes Identically to DataLink::RAW

## Description

`DataLink::IPV4` and `DataLink::RAW` are handled by the same match arm in `decode_packet`:
both call `SlicedPacket::from_ip(data)`. There is no behavioral difference between them.
This BC formally states that equivalence, which is significant because pcap files written
by some tools use IPV4 (numeric 228) rather than RAW (numeric 101) as their link type
even for raw IP captures.

## Preconditions

1. Two calls to `decode_packet` are made with byte-identical `data`.
2. One call uses `DataLink::RAW`; the other uses `DataLink::IPV4`.

## Postconditions

1. Both calls return `Ok(ParsedPacket)` or both return `Err`.
2. When both return `Ok`, the `ParsedPacket` values are field-for-field identical.
3. The difference in `DataLink` variant has zero observable effect on the output.

## Invariants

1. The Rust match arm is: `DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data)`.
2. This equivalence is guaranteed by source structure -- there is no conditional on the variant within the arm.
3. IPV6 link type is also in the same arm; it additionally supports IPv6 content (see BC-2.02.005).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | pcap file with link type 228 (IPV4) | Decoded as raw IP; no error |
| EC-002 | pcap file with link type 101 (RAW) | Same result as 228 |
| EC-003 | pcap file with link type 229 (IPV6) | Same arm; IPv6 content supported |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Same IPv4/TCP bytes decoded with RAW then IPV4 | Both produce identical ParsedPacket | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | decode_packet(data, RAW) == decode_packet(data, IPV4) for any valid IP bytes | unit: assert equality of both results |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 -- link-type equivalence rules are part of the gating whitelist contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | S-TBD |
| Origin BC | BC-DEC-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.003 -- composes with (the shared code path this BC documents)

## Architecture Anchors

- `src/decoder.rs:134` -- single match arm: `DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data)`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:134` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: Rust `|` in match arm provides compile-time grouping guarantee

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
