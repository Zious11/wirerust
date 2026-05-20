---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.029: Bad TLS Record Body Increments parse_errors; No Panic

## Description

When `parse_tls_plaintext` is called on a well-sized record (payload_len <=
MAX_RECORD_PAYLOAD) and returns `Err(NomErr::Incomplete(_))` or any other nom error,
`parse_errors` is incremented and the parsing loop `continue`s or returns. No panic
occurs, no finding is emitted, and the flow state is not cleared. The error is silently
absorbed as a parse-error counter increment.

## Preconditions

1. The direction buffer contains a complete TLS record header with valid length.
2. `payload_len <= MAX_RECORD_PAYLOAD` (so the oversized-record path is NOT taken).
3. `record_type == 0x16` (Handshake; non-handshake records are skipped before parse).
4. `parse_tls_plaintext` returns `Err(_)`.

## Postconditions

1. `parse_errors` is incremented by 1.
2. No finding is pushed.
3. No panic.
4. The flow remains in the `flows` HashMap (state not cleared).
5. The parsing loop continues to the next record (or returns if no more data).

## Invariants

1. `parse_errors` increments ONLY for genuine parse failures, not for oversized records
   (which use `truncated_records` in addition to `parse_errors`).
2. The difference between `truncated_records` and `parse_errors - truncated_records`
   is the count of genuine parse failures.
3. No panic is allowed; `tls_parser` returns `Result`, not panics.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Record with `record_type=0x16` but garbage payload | parse_errors++ |
| EC-002 | NomErr::Incomplete from parse_tls_plaintext (shouldn't happen after length check) | parse_errors++ (defensive) |
| EC-003 | Multiple consecutive malformed records | parse_errors incremented for each |
| EC-004 | Malformed record after a successful ClientHello | handshakes_seen unchanged; parse_errors++ |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| TLS Handshake record with garbage payload | parse_errors=1; no panic; no finding | happy-path |
| Valid ClientHello then malformed handshake | handshakes_seen=1; parse_errors=1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Bad TLS record body increments parse_errors without panic | unit: test_parse_error_counter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- graceful parse-error handling is a robustness property of TLS analysis |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:700-712, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-029 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.004 -- related to (oversized records also increment parse_errors, plus truncated_records)
- BC-2.07.031 -- composes with (parse_errors is surfaced in summarize)

## Architecture Anchors

- `src/analyzer/tls.rs:700-712` -- nom error match arms in try_parse_records
- `tests/tls_analyzer_tests.rs` -- test_parse_error_counter

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:700-712` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `Err(NomErr::Incomplete(_)) => { self.parse_errors += 1; }` and `Err(_) => { self.parse_errors += 1; }`
- **assertion**: test_parse_error_counter

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates parse_errors |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
