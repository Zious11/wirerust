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
capability: CAP-03
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
  - v1.3: Reframe PC2 as structural invariant; fix Architecture Anchor line range; expand Invariant 1 to cover both dst and src early-return arms — 2026-05-22
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.013: app_protocol_hint Returns None When TransportInfo is None

## Description

`app_protocol_hint` performs an early return of `None` when `self.transport` is
`TransportInfo::None`. No port lookup is attempted. This covers ICMP packets, packets
decoded via the `Protocol::Other` path, and any future case where no transport header was
parsed. The early return is explicit in the source; no fallthrough to the port table occurs.

## Preconditions

1. `ParsedPacket.transport` is `TransportInfo::None`.

## Postconditions

1. `app_protocol_hint()` returns `None`.
2. (Structural invariant — verified by code inspection, not black-box test.) The port match
   table is not consulted. A function returning `None` is observationally indistinguishable
   from one that consulted and then discarded a table result; this property is confirmed by
   inspecting that both `TransportInfo::None` arms in `app_protocol_hint` are explicit
   `return None` statements (src/decoder.rs:98 and :103) that precede the `match (src, dst)`
   port table entirely.

## Invariants

1. `TransportInfo::None` triggers an early `return None` in BOTH the `dst` extraction match
   (src/decoder.rs:98) and the `src` extraction match (src/decoder.rs:103). Either arm
   encountered first short-circuits the function before the port table is reached. The
   invariant therefore holds for both arms, not only the `dst` arm.
2. This invariant holds even if port 53 or 443 appears in some other field -- only
   `TransportInfo::Tcp` and `TransportInfo::Udp` carry port numbers.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ICMP packet (TransportInfo::None) | None -- even though ICMP is over IP |
| EC-002 | Protocol::Other(6) TCP packet truncated at transport layer | TransportInfo::None -> None |
| EC-003 | Direct ParsedPacket construction with transport=None | None |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ParsedPacket { transport: TransportInfo::None } | app_protocol_hint() returns None | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | TransportInfo::None always yields None from app_protocol_hint | unit: assert None for ICMP packet |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-03 ("Packet decoding") per domain/capabilities/cap-03-packet-decoding.md |
| Capability Anchor Justification | CAP-03 ("Packet decoding") per domain/capabilities/cap-03-packet-decoding.md -- early-return for no-transport is a CAP-03 decode output contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-004 |
| Origin BC | BC-DEC-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.012 -- composes with (this BC is the None arm of app_protocol_hint)

## Architecture Anchors

- `src/decoder.rs:98` -- `TransportInfo::None => return None` (dst extraction match arm)
- `src/decoder.rs:103` -- `TransportInfo::None => return None` (src extraction match arm)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:98` (dst arm), `src/decoder.rs:103` (src arm) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: early return on TransportInfo::None

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
