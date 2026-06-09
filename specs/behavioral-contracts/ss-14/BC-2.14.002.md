---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.002: MBAP Header Rejected for ADU Shorter than 8 Bytes (Truncation Safety)

## Description

`parse_mbap_header(data: &[u8]) -> Option<MbapHeader>` returns `None` for any input shorter
than 8 bytes, without panicking or reading out-of-bounds. This is the reject / truncation-safety
path of BC-2.14.001: it guarantees that partial ADUs (which arise from mid-capture TCP
reassembly boundaries) produce a safe `None` result and cause the `on_data` loop to break and
await more bytes. This property is the VP-022 sub-property A formal verification target —
provable via Kani over all inputs of length 0..7.

## Preconditions

1. `data` is a `&[u8]` of any length 0..=7 (i.e., `data.len() < 8`).
2. The call arises from the `on_data` parsing loop encountering a partial trailing ADU at the
   end of a TCP segment, or from a TCP segment that begins mid-PDU (rare; reassembler delivers
   in-order bytes but a fresh flow's first `on_data` call may arrive with incomplete data).
3. No well-formed Modbus MBAP header can exist in fewer than 8 bytes (7-byte header + 1-byte FC).

## Postconditions

1. `parse_mbap_header(data)` returns `None`.
2. The function does NOT panic — no slice index out-of-bounds, no `unwrap` on a short slice.
3. No fields are written; no `MbapHeader` struct is constructed.
4. `ModbusAnalyzer.parse_errors` is NOT incremented by this case — a short-input return of
   `None` from `parse_mbap_header` causes the loop to `break` (await more data), not to
   increment `parse_errors`. Parse errors are incremented only when a fully-parseable header
   fails the 3-point validity gate (BC-2.14.003, BC-2.14.004).
5. The `on_data` offset-advancing loop does NOT advance `offset` when `parse_mbap_header`
   returns `None` due to short input — the partial bytes are left at the current position
   for the next `on_data` delivery.

## Invariants

1. **No-panic guarantee**: `parse_mbap_header` must be safe to call with any `&[u8]`,
   including empty slices. The Rust slice indexing in `data[7]` would panic on lengths < 8
   without the length guard; the guard `if data.len() < 8 { return None; }` MUST be the
   first statement of the function body.
2. **VP-022 sub-property A coverage**: Kani proves `parse_mbap_header(data) == None` for all
   symbolic `data` where `data.len() < 8`, exhaustively over all byte values for lengths 0..7.
3. **Partial-ADU wait semantics**: a `None` return from `parse_mbap_header` due to insufficient
   data means "insufficient bytes; break and wait" — the reassembler will deliver the remaining
   bytes on the next `on_data` call as the TCP stream advances.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty slice: `data = []` (length 0) | `None` — no panic; length guard fires at `data.len() = 0 < 8` |
| EC-002 | Single byte: `data = [0x00]` | `None` — no panic |
| EC-003 | 7 bytes (one byte short of minimum): `data = [00 01 00 00 00 06 01]` | `None` — the 7-byte MBAP header without FC is not parseable; FC at byte 7 is absent |
| EC-004 | 4 bytes containing only MBAP Transaction ID + Protocol ID | `None` — cannot decode Length, Unit ID, or FC |
| EC-005 | PDU data for a known FC (e.g. `[0x03]`) without header | `None` — a single-byte FC without the 7-byte MBAP prefix does not meet the 8-byte minimum |
| EC-006 | `on_data` receives exactly 7 bytes as its entire `data` argument | `None` returned; loop breaks; `parse_errors` NOT incremented; offset NOT advanced |

## Canonical Test Vectors

| Input (hex) | `parse_mbap_header` return | Category |
|-------------|---------------------------|----------|
| `` (empty, len=0) | `None` | edge-case: zero-byte input, no panic |
| `00` (len=1) | `None` | edge-case |
| `00 01 00 00 00 06 01` (len=7, full MBAP minus FC) | `None` | edge-case: one byte short |
| `00 01 00 00 00` (len=5) | `None` | edge-case: partial Length field |
| `00 01 00 00 00 06 01 03` (len=8) | `Some(txn=1, proto=0, len=6, unit=1, fc=3)` | regression: exactly-8 must NOT return None (boundary test confirming the reject/accept boundary is strictly `< 8`) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Sub-property A: `parse_mbap_header` returns `None` for any `data.len() < 8`; returns `Some(_)` for `data.len() >= 8`; never panics | Kani: symbolic `&[u8]` lengths 0..16 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the truncation-safety and no-panic guarantee for the MBAP parser, which is a security-critical property of the ICS analysis capability's parse boundary |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation — no formatting occurs on a `None` return) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22); ADR-005 §2.4 (VP-022 sub-property A) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.001 — composes with (accept path; this is the reject/truncation path of the same function)
- BC-2.14.003 — sibling (reject path for Protocol ID invalidity — distinct from truncation)
- BC-2.14.004 — sibling (reject path for Length invalidity — distinct from truncation)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `parse_mbap_header`: length guard `if data.len() < 8 { return None; }` MUST be first statement
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.4` — "if data.len() < 8 { return None; }"
- `.factory/phase-f2-spec-evolution/architecture-delta.md §9` — VP-022 sub-property A formal design

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — sub-property A (no-panic + None-for-short-input; Kani-provable)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.4 and §9; modbus-tcp-research.md §1 (minimum ADU size) |
| **Confidence** | high — no-panic guarantee is directly Kani-provable; length guard is specified in the architecture delta |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — empty or short input always returns None |
| **Thread safety** | Send + Sync (pure function) |
| **Overall classification** | pure core — VP-022 Kani target |
