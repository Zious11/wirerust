---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.008: CIP Error Response Detection — general_status Extraction from Response Frames

## Description

When `classify_cip_service(service)` returns `CipServiceClass::Response` (high bit of service
byte is set), `process_pdu` extracts the CIP general_status byte from the response data. The
general_status is at a fixed offset within a CIP Unconnected Response frame. A non-zero
general_status indicates a CIP error response. Error responses are accumulated in the windowed
`error_counts_in_window: HashMap<u8, u64>` on `EnipFlowState`. A burst of CIP error responses
(especially repeated status 0x08 "Service Not Supported" or 0x09 "Invalid Attribute Value")
may indicate an adversary probing for supported services — a T0888 reconnaissance pattern.
This BC specifies the general_status extraction contract.

## Preconditions

1. `classify_cip_service(cip_header.service)` returns `CipServiceClass::Response`.
2. The carrying CPF item `type_id == 0x00B2` (Unconnected Data Item). Items with
   `type_id == 0x00B1` (Connected Data Item) include a 2-byte sequence number prefix
   that shifts the `general_status` offset; extraction for `0x00B1` items is deferred
   to v0.12.0. This is a HARD scope gate: if `type_id != 0x00B2`, skip `general_status`
   extraction entirely.
3. `cip_item_data.len() >= 4` — CIP response frame has at least 4 bytes: service (1) +
   reserved (1) + general_status (1) + additional_status_size (1).
4. `flow.is_non_enip == false`.

## Postconditions

**Scope gate (Precondition 2):** If `cpf_item.type_id != 0x00B2`, the function returns
immediately without any counter update. All postconditions below apply only when
`cpf_item.type_id == 0x00B2`.

1. `general_status = cip_item_data[2]` — third byte of CIP response frame (per ODVA CIP
   response format: byte 0 = service|0x80, byte 1 = reserved 0x00, byte 2 = general_status,
   byte 3 = additional_status_size).
2. If `general_status != 0x00` (error response):
   - `flow.error_counts_in_window.entry(general_status).or_insert(0) += 1`.
   - If `flow.error_window_start_ts == 0` (first error in window): seed
     `flow.error_window_start_ts = now_ts`.
3. If `general_status == 0x00` (success response): no error counter update.
4. Window management: if `now_ts.wrapping_sub(flow.error_window_start_ts) > 10` (10-second
   window expired): reset `flow.error_counts_in_window.clear()`,
   `flow.error_window_start_ts = now_ts`, `flow.error_rate_emitted = false`.
5. The function does not emit a Finding directly — error-rate-based T0888 findings are
   emitted by BC-2.17.014 when the error burst threshold is crossed.

## Invariants

1. **general_status offset is fixed**: byte 2 of any CIP response frame is the general_status.
   This is normative ODVA CIP: byte 0 = service | 0x80, byte 1 = reserved, byte 2 =
   general_status. [SPEC: ODVA CIP Specification Vol 1 §2-4.2]
2. **Error accumulation is windowed**: `error_counts_in_window` is per general_status code
   within the 10-second window. The window resets on expiry. The lifetime error count is
   reflected in `EnipAnalyzer.error_count` (aggregate, incremented on every error response).
3. **Zero status = success**: `general_status == 0x00` indicates success. Success responses
   do not increment error counters.
4. **10-second window**: distinct from the 1-second write-burst window (BC-2.17.012). The
   10-second window is appropriate for error-rate detection: a legitimate host typically
   produces few CIP errors; a scanning host accumulates many distinct error codes quickly.
5. **No finding emission here**: this BC only extracts and accumulates. The T0888
   error-rate finding is emitted by BC-2.17.014.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Response with `general_status = 0x00` (success) | No counter update; success is not anomalous |
| EC-002 | Response with `general_status = 0x08` ("Service Not Supported") | `error_counts_in_window[0x08] += 1`; first scan probe |
| EC-003 | Response with `general_status = 0x09` ("Invalid Attribute Value") | `error_counts_in_window[0x09] += 1` |
| EC-004 | CIP response item data length < 4 | general_status extraction skipped (cannot safely index byte 2) |
| EC-005 | 10-second window expires; new error response arrives | Window reset; `error_counts_in_window` cleared; `error_rate_emitted = false`; new error seeds window |
| EC-006 | `error_rate_emitted = true` (finding already emitted in current window) | error_counts_in_window still updated; no new finding (one-shot guard owned by BC-2.17.014) |
| EC-007 | Response from Connected Data Item (0x00B1, 2-byte sequence prefix present) | general_status extraction scoped to Unconnected Data Items (0x00B2) for v0.11.0; Connected (0x00B1) response extraction deferred to v0.12.0 (byte offset shifts by 2 due to sequence prefix) |

## Canonical Test Vectors

**CIP Error Response (Service Not Supported 0x08):**
```
CIP response item_data (hex): 8E 00 08 00
byte[0]: 0x8E = service 0x0E | 0x80 (GetAttributeSingle response)
byte[1]: 0x00 = reserved
byte[2]: 0x08 = general_status (Service Not Supported)
byte[3]: 0x00 = additional_status_size = 0
```
Expected: `flow.error_counts_in_window[0x08] += 1`; `EnipAnalyzer.error_count += 1`

**CIP Success Response:**
```
CIP response item_data (hex): 8E 00 00 00 <response data>
byte[2]: 0x00 = general_status (success)
```
Expected: no error counter update

| general_status | Meaning | Counter updated? |
|----------------|---------|-----------------|
| 0x00 | Success | No |
| 0x08 | Service Not Supported | Yes |
| 0x09 | Invalid Attribute Value | Yes |
| 0x0C | Object State Conflict | Yes |
| 0xFF | (vendor-specific) | Yes |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | general_status extraction, windowed accumulation: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — CIP error response detection is required for T0888 error-burst recon detection: an adversary scanning for supported CIP services generates a burst of error responses (Service Not Supported, Invalid Attribute) that this BC accumulates and BC-2.17.014 converts into a finding |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (EnipFlowState error fields) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none directly — accumulation BC; T0888 emitted by BC-2.17.014) |

## Related BCs

- BC-2.17.007 — depends on (Response classification is the precondition for this BC)
- BC-2.17.014 — composes with (error-rate T0888 finding emitted when burst threshold crossed)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipFlowState.error_counts_in_window: HashMap<u8, u64>`
- `src/analyzer/enip.rs` — `EnipFlowState.error_window_start_ts: u32`
- `src/analyzer/enip.rs` — `EnipFlowState.error_rate_emitted: bool`
- `src/analyzer/enip.rs` — `EnipAnalyzer.error_count: u64` (aggregate lifetime counter)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 4` — EnipFlowState error window fields

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — effectful shell; unit test)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (error_counts_in_window field); ODVA CIP Specification Vol 1 §2-4.2 (response frame layout: byte 2 = general_status) |
| **Confidence** | high — general_status offset is normative ODVA CIP |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.error_counts_in_window, flow.error_window_start_ts; reads EnipAnalyzer.error_count |
| **Deterministic** | yes — same response sequence produces same counter state |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (response detection within process_pdu) |
