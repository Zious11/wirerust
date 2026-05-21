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

# BC-2.07.033: TLS Analyzer Ignores Non-Handshake Records

## Description

In `try_parse_records`, after consuming record bytes from the buffer, the code checks
`record_type != 0x16` (non-Handshake). If so, the record is silently skipped via
`continue` in the parsing loop. No `parse_errors` increment, no finding, no counter
change. This covers TLS ChangeCipherSpec (0x14), Alert (0x15), and ApplicationData
(0x17) records.

## Preconditions

1. A complete TLS record has been extracted from the buffer.
2. `record_type != 0x16` (content type is not Handshake).
3. `payload_len <= MAX_RECORD_PAYLOAD` (not an oversized record).

## Postconditions

1. The record bytes are consumed (drained from the buffer).
2. No `parse_errors` increment.
3. No finding emitted.
4. The loop `continue`s to check for the next complete record.

## Invariants

1. Non-handshake records are consumed (drained from the buffer) even though they are
   not parsed. This prevents buffer stalls.
2. The early-return for `done()` at the start of `on_data` (BC-2.07.003) is a separate
   mechanism; this BC covers the within-loop skip for non-handshake records.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ApplicationData (0x17) record | Consumed silently; loop continues |
| EC-002 | ChangeCipherSpec (0x14) record | Consumed silently |
| EC-003 | Alert (0x15) record | Consumed silently |
| EC-004 | Unknown record type (e.g., 0x18) | Consumed silently (same code path) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| TLS record with type 0x17 (AppData) followed by valid ClientHello | parse_errors=0; handshakes_seen=1 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Non-handshake records do not increment parse_errors | unit: test_stop_after_handshake exercises this (records after hellos are app data) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- non-handshake record skipping is part of TLS analysis record dispatch |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:678-682, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-033 (pass-3 ingestion corpus, MEDIUM confidence -- inferred from code; test_stop_after_handshake exercises it) |

## Related BCs

- BC-2.07.034 -- related to (after-done short-circuit is a separate mechanism)
- BC-2.07.029 -- related to (parse_errors is for genuine handshake parse failures, not for skips)

## Architecture Anchors

- `src/analyzer/tls.rs:678-682` -- `if record_type != 0x16 { continue; }` in try_parse_records

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:678-682` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if record_type != 0x16 { continue; }`
- **inferred**: co-pinned with test_stop_after_handshake which sends app-data records

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates client_buf or server_buf (drain) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
