---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/dns.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-08
capability: CAP-08
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
  - v1.3: Wave 4 Ph3 per-story adversarial fix N-1: is_dns_port anchor :34-35 → :34-36 synced with STORY-066 v1.2 — 2026-05-22
  - v1.4: Wave 4 Ph3 per-story adversarial fix F-1/F-2/F-3: re-synced all dns.rs anchors after module-doc-comment expansion shifted functions ~8-10 lines; is_dns_port :34-36 → :42-44, can_decode :52-60 → :60-68, Source Evidence Path :34-60 → :42-68; F-3: corrected VP-019 proof-method test from test_dns_analyzer_counts_queries (exercises analyze/summarize, not can_decode) to test_dns_can_decode_port_53_tcp_and_udp — 2026-05-22
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.001: DnsAnalyzer Matches Packets Where Port == 53 (TCP or UDP)

## Description

`DnsAnalyzer::can_decode(packet)` returns `true` for any packet where either the source or
destination port is 53, regardless of whether the transport is TCP or UDP. It returns `false`
for any other port combination and for `TransportInfo::None` (ICMP / unknown protocol).
This gate is the entry condition for DNS analysis.

## Preconditions

1. A `ParsedPacket` is passed to `can_decode`.
2. The packet has any transport: Tcp, Udp, or None.

## Postconditions

1. Returns `true` iff `transport` is `Tcp { src_port: 53, .. }` OR `Tcp { .., dst_port: 53 }`
   OR `Udp { src_port: 53, .. }` OR `Udp { .., dst_port: 53 }`.
2. Returns `false` for `TransportInfo::None` (ICMP, unknown).
3. Returns `false` when neither port is 53.

## Invariants

1. The port check is `src == 53 || dst == 53` via `is_dns_port(src, dst)`.
2. Both TCP/53 and UDP/53 are accepted; there is no protocol preference.
3. No payload inspection is done in `can_decode`; the check is purely port-based.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | UDP src=53, dst=12345 (DNS response to client) | true |
| EC-002 | UDP src=12345, dst=53 (DNS query to server) | true |
| EC-003 | TCP dst=53 (DNS-over-TCP) | true |
| EC-004 | UDP src=54, dst=54 (not DNS) | false |
| EC-005 | ICMP packet (TransportInfo::None) | false |
| EC-006 | UDP dst=443 | false |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ParsedPacket with Udp { src: 12345, dst: 53 } | can_decode returns true | happy-path |
| ParsedPacket with Udp { src: 53, dst: 12345 } | can_decode returns true | happy-path |
| ParsedPacket with Tcp { src: 12345, dst: 53 } | can_decode returns true | happy-path |
| ParsedPacket with Udp { src: 9999, dst: 9999 } | can_decode returns false | edge-case |
| ParsedPacket with TransportInfo::None | can_decode returns false | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-019 | can_decode is true iff port 53 is src or dst | unit: test_dns_can_decode_port_53_tcp_and_udp |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-08 ("DNS traffic analysis") per capabilities.md §CAP-08 |
| Capability Anchor Justification | CAP-08 ("DNS traffic analysis") per capabilities.md §CAP-08 -- can_decode is the port-53 gate that qualifies packets for DNS analysis |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-08 (analyzer/dns.rs, C-11) |
| Stories | STORY-066 |
| Origin BC | BC-DNS-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.08.002 -- composes with (QR-bit dispatch only runs when can_decode is true)
- BC-2.08.004 -- composes with (analyze returns empty findings regardless of port match)

## Architecture Anchors

- `src/analyzer/dns.rs:42-44` -- is_dns_port helper: `src == 53 || dst == 53`
- `src/analyzer/dns.rs:60-68` -- can_decode dispatches on transport variant

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/dns.rs:42-68` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: TransportInfo::None arm returns false unconditionally

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
