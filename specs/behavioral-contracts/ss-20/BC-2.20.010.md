---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-20
capability: CAP-20
lifecycle_status: active
introduced: feature-s7comm
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ARCH-INDEX.md
input-hash: "8f268fc"
---

# BC-2.20.010: `parse_cotp_header` Recognizes Data Transfer (DT) TPDU With Empty Payload — `protocol_id: None`

## Description

A DT TPDU (high nibble `0xF` at `tpkt_payload[1]`) whose payload region is empty — i.e.
`tpkt_payload.len() == payload_offset` exactly, with no trailing byte to serve as a
protocol-ID — cannot yield a `protocol_id`. Per ADR-014 Decision 1, `CotpHeader.protocol_id`
is `Some(byte)` "only for a DT-TPDU whose payload begins with a recognized protocol-ID
byte ... `None` ... when the DT payload is empty." `parse_cotp_header` returns
`CotpHeader { tpdu_type: DataTransfer, protocol_id: None, payload_offset }` for this
case — the DT sibling of the CR/CC `protocol_id: None` paths (BC-2.20.007/008), but
reached for a structurally different reason (no payload present at all, rather than the
TPDU type structurally never carrying one).

## Preconditions

1. `tpkt_payload.len() >= 1 + LI` (LI-truncation check passed — BC-2.20.006).
2. `tpkt_payload[1] & 0xF0 == 0xF0` (DT TPDU-code high nibble).
3. `tpkt_payload.len() == payload_offset` where `payload_offset = 1 + LI` — exactly zero
   bytes of upper-layer payload follow the DT header.

## Postconditions

1. `parse_cotp_header(tpkt_payload)` returns
   `Some(CotpHeader { tpdu_type: CotpTpduType::DataTransfer, protocol_id: None, payload_offset })`.
2. No out-of-bounds index is attempted at `tpkt_payload[payload_offset]` — the function
   checks `tpkt_payload.len() > payload_offset` before ever reading that index (see
   BC-2.20.009 Precondition 3); this BC is precisely the negation of that check.
3. `S7commAnalyzer` observing a `DataTransfer` TPDU with `protocol_id: None` treats it
   as an empty keepalive-shaped or degenerate DT frame; no classification decision can
   be made from this frame alone (no finding emission at this layer).

## Invariants

1. **Empty-payload DT is structurally distinct from CR/CC's "never has payload"**: a DT
   TPDU is the type that *can* carry upper-layer data; `protocol_id: None` here reflects
   "this particular instance happened to carry zero bytes," not "this TPDU type never
   carries payload."
2. **No panic on the boundary**: `tpkt_payload.len() == payload_offset` is the exact
   boundary condition distinguishing this BC from BC-2.20.009; both are total and safe.
3. **Purity**: no state mutation; deterministic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tpkt_payload.len() == payload_offset` exactly (minimal DT, LI=2, no trailing byte) | `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: None, payload_offset: 3})` |
| EC-002 | A TPKT frame with `length == 4 + 3 = 7` (4-byte TPKT header + 3-byte DT-only COTP header, no payload at all) | Reaches this BC; the whole TPKT+COTP frame carries zero upper-layer bytes |
| EC-003 | `tpkt_payload.len() == payload_offset + 1` (exactly one trailing byte) | Falls to BC-2.20.009, not this BC (`protocol_id: Some(byte)`) |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x02, 0xF0, 0x80]` (LI=2, DT, TPDU-NR=0x80, no trailing payload byte) | `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: None, payload_offset: 3})` | happy-path: empty-payload DT |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all inputs with `tpkt_payload[1] & 0xF0 == 0xF0` and `len == payload_offset`, `parse_cotp_header` returns `Some` with `protocol_id: None`; no out-of-bounds access | Kani P0 (bounds-safety component per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the empty-DT-payload boundary condition explicitly named in the frozen `CotpHeader` interface (ADR-014 Decision 1) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decision 1 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.20.009 — composes with (complement: DT with non-empty payload, `protocol_id: Some(byte)`)
- BC-2.20.007 — composes with (CR TPDU, also `protocol_id: None`, for a different structural reason)
- BC-2.20.008 — composes with (CC TPDU, also `protocol_id: None`, for a different structural reason)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header`: DT-empty-payload branch, `tpkt_payload.len() == payload_offset`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — "`None` for CR/CC (no upper-layer payload exists yet) or when the DT payload is empty"

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst per ADR-014 Decision 9,
anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — Kani P0 target (bounds safety) |
