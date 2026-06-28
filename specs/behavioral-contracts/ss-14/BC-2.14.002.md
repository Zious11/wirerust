---
document_type: behavioral-contract
level: L3
version: "2.0"
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
modified:
  - version: "2.0"
    date: 2026-06-28
    change: "RULING-MODBUS-SIBLING-001 (2026-06-28, §4.1): Breaking — replace single `carry: Vec<u8>` with `carry_c2s: Vec<u8>` (client-to-server) and `carry_s2c: Vec<u8>` (server-to-client). Precondition 3 updated: buf prepend selects directional carry by existing `direction` param (no on_data signature change). Postcondition 1 updated: stash targets directional carry. Invariant 1 updated: per-direction 260-byte cap (MAX_ADU_CARRY_BYTES). New Invariant 4 added: direction-isolation guarantee (carry_c2s and carry_s2c never mixed). New EC-007 added: partial c2s stashed in carry_c2s; next s2c call uses carry_s2c (empty) — clean s2c parse, carry_c2s retains c2s partial. Architecture Anchors updated to split fields. DRIFT-MODBUS-DIRECTION-001 fix."
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
3. The combined buffer `buf` is constructed by prepending the appropriate directional carry
   before `data`: `buf = (match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c }) ++ data`.
   The existing `direction: Direction` parameter in `on_data` is used to select the carry
   buffer — no on_data signature change is required. (RULING-MODBUS-SIBLING-001 §4.1.1)
4. No well-formed Modbus MBAP header can exist in fewer than 8 bytes (7-byte header + 1-byte FC).

## Postconditions

1. `parse_mbap_header(data)` returns `None`.
2. The function does NOT panic — no slice index out-of-bounds, no `unwrap` on a short slice.
3. No fields are written; no `MbapHeader` struct is constructed.
4. `ModbusAnalyzer.parse_errors` is NOT incremented by this case — a short-input return of
   `None` from `parse_mbap_header` causes the loop to `break` (await more data), not to
   increment `parse_errors`. Parse errors are incremented only when a fully-parseable header
   fails the 3-point validity gate (BC-2.14.003, BC-2.14.004).
5. The `on_data` offset-advancing loop does NOT advance `offset` when `parse_mbap_header`
   returns `None` due to short input — the partial bytes are stashed into the directional
   carry: `match direction { ClientToServer => flow.carry_c2s = remaining, ServerToClient => flow.carry_s2c = remaining }`.
   (RULING-MODBUS-SIBLING-001 §4.1.2)

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
   Per-direction bounded carry: `flow.carry_c2s.len() <= MAX_ADU_CARRY_BYTES = 260` AND
   `flow.carry_s2c.len() <= MAX_ADU_CARRY_BYTES = 260`. Each direction has its own independent
   260-byte DoS cap. The carry-cap overflow check at each stash point references the active
   directional carry: if `active_carry.len() + remaining.len() > MAX_ADU_CARRY_BYTES`, the flow
   sets `is_non_modbus = true` and terminates. (RULING-MODBUS-SIBLING-001 §1.5; replaces the
   v1.0 single `flow.carry.len() <= 260` invariant.)
4. **Direction isolation**: `carry_c2s` and `carry_s2c` are NEVER mixed. `on_data` selects
   exactly one buffer per call based on the `direction` argument. No frame-walk loop ever
   prepends bytes from one direction into the other. A partial c2s ADU stashed in `carry_c2s`
   is never prepended to an s2c delivery, and vice versa. Prevents DRIFT-MODBUS-DIRECTION-001
   (RULING-MODBUS-SIBLING-001 §1.1). The carry-cap overflow path sets the shared `is_non_modbus`
   flag regardless of direction — a per-flow latch is correct here (if either direction's carry
   is being DoS'd, the entire flow is suspect). (RULING-MODBUS-SIBLING-001 §1.4)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty slice: `data = []` (length 0) | `None` — no panic; length guard fires at `data.len() = 0 < 8` |
| EC-002 | Single byte: `data = [0x00]` | `None` — no panic |
| EC-003 | 7 bytes (one byte short of minimum): `data = [00 01 00 00 00 06 01]` | `None` — the 7-byte MBAP header without FC is not parseable; FC at byte 7 is absent |
| EC-004 | 4 bytes containing only MBAP Transaction ID + Protocol ID | `None` — cannot decode Length, Unit ID, or FC |
| EC-005 | PDU data for a known FC (e.g. `[0x03]`) without header | `None` — a single-byte FC without the 7-byte MBAP prefix does not meet the 8-byte minimum |
| EC-006 | `on_data` receives exactly 7 bytes as its entire `data` argument | `None` returned; loop breaks; `parse_errors` NOT incremented; offset NOT advanced |
| EC-007 | Partial c2s MBAP (< 8 bytes, e.g. 6 bytes of `[0x00,0x01,0x00,0x00,0x00,0x06]`) stashed in `carry_c2s`; next `on_data` call is `direction=ServerToClient` with a complete s2c ADU (e.g. FC=0x03, 13 bytes) | `carry_s2c` is empty → prepended buf = `[] ++ s2c_data` = clean s2c bytes. `parse_mbap_header` parses the correct s2c MBAP header; `fn_code_counts[0x03]` incremented. `carry_c2s` retains the c2s partial (unchanged). The c2s splice (DRIFT-MODBUS-DIRECTION-001) does NOT occur. (RULING-MODBUS-SIBLING-001 §4.1.5) |

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
- `src/analyzer/modbus.rs` — `ModbusFlowState.carry_c2s: Vec<u8>` (replaces `carry: Vec<u8>`, pre-fix line 170)
- `src/analyzer/modbus.rs` — `ModbusFlowState.carry_s2c: Vec<u8>` (new field, symmetric; pre-fix line 170 region)
- `src/analyzer/modbus.rs` — `on_data` carry operations (pre-fix lines 1043-1056 prepend, 1080-1085 MBAP-partial stash, 1120-1125 ADU-partial stash): all reference active directional carry selected by `direction` param
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.4` — "if data.len() < 8 { return None; }"
- `.factory/phase-f2-spec-evolution/architecture-delta.md §9` — VP-022 sub-property A formal design
- Note: line numbers are PRE-fix (STORY-141 implementer re-anchors to post-fix lines)

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
