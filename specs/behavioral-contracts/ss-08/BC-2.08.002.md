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
  - v1.3: Wave 4 Ph3 per-story adversarial fix F-1/F-2: re-synced all dns.rs anchors after module-doc-comment expansion shifted functions ~8-10 lines; is_query :38-44 → :46-52, analyze :62-70 → :70-78, inline prose length-guard reference "at dns.rs:40" → lines 48-50, Source Evidence Path :38-70 → :46-78 — 2026-05-22
  - v1.4: Wave 4 Ph3 per-story adversarial fix M-2/M-3: corrected VP proof-method test references to real tests that genuinely exercise the property — propagating the BC-2.08.001 v1.4 fix to sibling BCs; row 1 now cites both test_dns_analyzer_counts_queries and test_dns_analyzer_counts_responses (QR=0 and QR=1 sides); row 2 replaced description "short-payload test" with test_dns_short_payload_counted_as_response — 2026-05-22
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.002: DNS QR-Bit Dispatch: response_count Incremented if Set; query_count Otherwise

## Description

`DnsAnalyzer::analyze(packet)` inspects bit 15 of the 16-bit Flags field at byte offset 2
of the DNS payload. If the QR bit (bit 7 of `payload[2]`) is set (value 1), the packet is
a DNS response and `response_count` is incremented. If the bit is clear (value 0), the
packet is a DNS query and `query_count` is incremented. Payloads shorter than 12 bytes
cannot be inspected: `is_query` returns `false` (because the length guard at `dns.rs:48-50`
fires before the bit test), so the `else` branch in `analyze` increments `response_count`.

## Preconditions

1. `can_decode` returned `true` (port 53 matched).
2. `analyze` is called with the packet.

## Postconditions

1. If `payload.len() >= 12` AND `(payload[2] & 0x80) == 0`: `query_count += 1`.
2. If `payload.len() >= 12` AND `(payload[2] & 0x80) != 0`: `response_count += 1`.
3. If `payload.len() < 12`: `is_query` returns `false`; `response_count += 1`.
4. Returns `Vec<Finding>` which is always empty (see BC-2.08.004).

## Invariants

1. Exactly one counter (query_count or response_count) is incremented per call to analyze.
2. `is_query` returns `false` for payloads shorter than 12 bytes (the DNS minimum header
   length); such packets are counted as responses by the else branch.
3. The QR bit is at offset 2, bit 7 of the DNS message (MSB of the Flags high byte).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | payload[2] == 0x00 (QR=0, standard query) | query_count++ |
| EC-002 | payload[2] == 0x80 (QR=1, standard response) | response_count++ |
| EC-003 | payload.len() == 11 (truncated DNS header) | is_query returns false; response_count++ |
| EC-004 | payload.len() == 0 (no DNS bytes) | is_query returns false; response_count++ |
| EC-005 | payload[2] == 0xFF (QR=1 + other flags) | response_count++ (QR bit is set) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Port-53 UDP packet with valid DNS query bytes | query_count == 1 after analyze | happy-path |
| Port-53 UDP packet with valid DNS response bytes | response_count == 1 | happy-path |
| Port-53 UDP packet with 6-byte payload | response_count++ (is_query false) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-019 | QR bit at payload[2] bit 7 controls which counter increments | unit: test_dns_analyzer_counts_queries (QR=0 → query_count++), test_dns_analyzer_counts_responses (QR=1 → response_count++) |
| VP-019 | Short payload increments response_count | unit: test_dns_short_payload_counted_as_response |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-08 ("DNS traffic analysis") per domain/capabilities/cap-08-dns-analysis.md |
| Capability Anchor Justification | CAP-08 ("DNS traffic analysis") per domain/capabilities/cap-08-dns-analysis.md -- QR-bit dispatch is the core statistical classification in the DNS analysis capability |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-08 (analyzer/dns.rs, C-11) |
| Stories | STORY-066 |
| Origin BC | BC-DNS-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.08.003 -- composes with (summarize reports these counters)
- BC-2.08.004 -- composes with (analyze always returns empty findings)

## Architecture Anchors

- `src/analyzer/dns.rs:46-52` -- is_query helper: payload length guard + bit test
- `src/analyzer/dns.rs:70-78` -- analyze: increment logic based on is_query result, returns Vec::new()

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/dns.rs:46-78` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if payload.len() < 12 { return false }` in is_query
- **assertion**: test_dns_analyzer_counts_queries; test_dns_analyzer_counts_responses; test_dns_short_payload_counted_as_response

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates query_count/response_count (struct fields) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed (pure logic; counter mutation is trivial) |

## Refactoring Notes

If is_query returned true for short payloads (counting them as queries), this would be
a miscount. The current implementation counts short payloads as responses; this is a
minor inaccuracy but the DNS subsystem is statistics-only with no anomaly detection,
so the impact is negligible.
