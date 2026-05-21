---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: ["v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.004: TLS Record Payload > MAX_RECORD_PAYLOAD Increments parse_errors and truncated_records

## Description

During `try_parse_records`, after reading the 5-byte TLS record header and computing
`payload_len = u16::from_be_bytes([buf[3], buf[4]])`, if `payload_len > MAX_RECORD_PAYLOAD`
(18,432 bytes), the record is rejected as a DoS protection measure. Both `parse_errors`
and `truncated_records` are incremented, the direction buffer is cleared, and the loop
returns. No attempt is made to parse the record's payload. This is the CNV-PAT-002
instrumentation added in LESSON-P1.05.

## Preconditions

1. The direction buffer contains at least 5 bytes (a complete record header).
2. The declared `payload_len` in bytes 3-4 of the record header is > 18,432.

## Postconditions

1. `parse_errors` is incremented by 1.
2. `truncated_records` is incremented by 1.
3. The direction buffer (`client_buf` or `server_buf`) is cleared entirely.
4. `try_parse_records` returns (the parsing loop exits).
5. No finding is emitted for this truncation event.
6. `handshakes_seen` is NOT incremented.

## Invariants

1. Both `parse_errors` and `truncated_records` are always incremented together for
   oversized records. They are NEVER incremented independently for this case.
2. Buffer clearing is unconditional: all buffered bytes for the direction are dropped,
   including any partial records that preceded the oversized one.
3. `MAX_RECORD_PAYLOAD = 18,432` is a constant (RFC 5246 ciphertext max). TLS 1.3
   max is 16,640; the larger value is used as a safe upper bound.
4. The two counters serve different consumer audiences: `parse_errors` is
   back-compatible with existing dashboards; `truncated_records` lets JSON consumers
   distinguish DoS-protection drops from genuine parse failures.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | payload_len = 18,432 exactly (boundary, not over) | Accepted; no truncation counter increment |
| EC-002 | payload_len = 18,433 (one over) | Both counters incremented; buffer cleared |
| EC-003 | payload_len = 65,535 (max u16) | Both counters incremented; buffer cleared |
| EC-004 | Multiple oversized records in sequence on same flow | Each one increments both counters independently |
| EC-005 | Oversized record after a valid ClientHello on same flow | parse_errors > 0; handshakes_seen from prior hello preserved |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| TLS record header with payload_len = 18,433 | parse_errors=1; truncated_records=1; buffer cleared | happy-path |
| TLS record header with payload_len = 18,432 (boundary) | parse_errors=0; truncated_records=0; record accepted for parsing | edge-case |
| ClientHello then oversized record | handshakes_seen=1; parse_errors=1; truncated_records=1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | payload_len > 18432 increments both parse_errors and truncated_records | unit: test_oversized_sni_exceeds_record_payload_limit |
| — | payload_len == 18432 does not increment truncated_records | unit: boundary test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- MAX_RECORD_PAYLOAD guard is part of TLS analysis bounded-resource design |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation -- truncated records are not stored) |
| Architecture Module | SS-07 (analyzer/tls.rs:643-653, C-13) |
| Stories | STORY-058 |
| Origin BC | BC-TLS-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.029 -- related to (bad TLS body also increments parse_errors, but NOT truncated_records)
- BC-2.07.033 -- related to (non-handshake records are also skipped, but silently)

## Architecture Anchors

- `src/analyzer/tls.rs:643-653` -- oversized-record guard and buffer clear
- `src/analyzer/tls.rs:33` -- MAX_RECORD_PAYLOAD constant definition
- `src/analyzer/tls.rs:312` -- truncated_records field declaration
- `tests/tls_analyzer_tests.rs` -- test_oversized_sni_exceeds_record_payload_limit

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:643-653` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if payload_len > MAX_RECORD_PAYLOAD { self.parse_errors += 1; self.truncated_records += 1; ... return; }`
- **assertion**: test_oversized_sni_exceeds_record_payload_limit verifies parse_errors increment

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates parse_errors, truncated_records, client_buf or server_buf |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
