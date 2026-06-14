---
document_type: behavioral-contract
level: L3
version: "1.5"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: re-point Proof Method/Evidence from test_stop_after_handshake (done-short-circuit, BC-2.07.003/034 proof) to within-loop-skip tests (F-S058-P3-001/P4-001) — 2026-05-29"
  - "v1.4: reconcile internal done-short-circuit cross-reference (BC-2.07.003 vs 034 consistency; F-S058-P5-002) — 2026-05-29"
  - "v1.5: PG-ARP-F2-007 ss-07 full re-anchor — non-handshake skip 718-736→718-736 — 2026-06-13"
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
2. The early-return at the entry of `on_data` when `done() == true` (BC-2.07.034 — the
   pre-buffering short-circuit; BC-2.07.003 specifies the same guard from the per-record
   behavioral outcome perspective) is a separate mechanism; this BC covers only the
   within-loop `continue` skip for non-handshake record types (tls.rs:718-736).

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
| — | Non-handshake records do not increment parse_errors | unit: test_within_loop_nonhandshake_skip_before_done (canonical — sends non-handshake record + ClientHello in one on_data while flow NOT done, directly hits tls.rs:718-736); test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently (multi-type EC coverage); see also test_appdata_record_skipped_then_hello |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- non-handshake record skipping is part of TLS analysis record dispatch |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:718-736, C-13) |
| Stories | STORY-058 |
| Origin BC | BC-TLS-033 (pass-3 ingestion corpus, HIGH confidence -- dedicated within-loop-skip tests now exist: test_within_loop_nonhandshake_skip_before_done, test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently) |

## Related BCs

- BC-2.07.034 -- related to (after-done short-circuit is a separate mechanism)
- BC-2.07.029 -- related to (parse_errors is for genuine handshake parse failures, not for skips)

## Architecture Anchors

- `src/analyzer/tls.rs:718-736` -- `if record_type != 0x16 { continue; }` in try_parse_records

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:718-736` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if record_type != 0x16 { continue; }`
- **dedicated unit tests**: test_within_loop_nonhandshake_skip_before_done (canonical within-loop-skip proof, tls.rs:718-736); test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently (multi-type EC-001 through EC-004 coverage); test_appdata_record_skipped_then_hello (happy-path sequence)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates client_buf or server_buf (drain) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
