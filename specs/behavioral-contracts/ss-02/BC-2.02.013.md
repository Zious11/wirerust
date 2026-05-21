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
2. The port match table is not consulted.

## Invariants

1. `TransportInfo::None` as the `dst` pattern always returns `None` immediately.
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
| L2 Capability | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 |
| Capability Anchor Justification | CAP-03 ("Packet decoding") per capabilities.md §CAP-03 -- early-return for no-transport is a CAP-03 decode output contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | STORY-004 |
| Origin BC | BC-DEC-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.02.012 -- composes with (this BC is the None arm of app_protocol_hint)

## Architecture Anchors

- `src/decoder.rs:98-99` -- `TransportInfo::None => return None`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:98-99` |
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
