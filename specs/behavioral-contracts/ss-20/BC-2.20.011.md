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
input-hash: "cf116b5"
---

# BC-2.20.011: `parse_cotp_header` Returns None for an Unrecognized TPDU-Type Code (Not CR/CC/DT)

## Description

`CotpTpduType` (ADR-014 Decision 1, frozen) has exactly three variants:
`ConnectRequest`, `ConnectConfirm`, `DataTransfer`. ISO 8073 defines several additional
TPDU types not modeled by this enum — Disconnect Request (DR, high nibble `0x8`),
Disconnect Confirm (DC, `0xC`), Expedited Data (ED, `0x1`), Data Acknowledgement (AK,
`0x6`), Expedited Data Acknowledgement (EA, `0x2`), Reject (RJ, `0x5`), TPDU Error (ER,
`0x7`). When `tpkt_payload[1]`'s high nibble matches none of `0xE` (CR), `0xD` (CC), or
`0xF` (DT), `parse_cotp_header` returns `None` — the frame is left unparsed, never
force-fit into one of the three recognized variants. This is the direct implementation
of the frozen-interface design constraint: SS-20 models exactly the TPDU types S7comm's
ISO-on-TCP profile actually uses, and explicitly declines to guess for the rest.

## Preconditions

1. `tpkt_payload.len() >= 1 + LI` (LI-truncation check passed — BC-2.20.006).
2. `tpkt_payload[1] & 0xF0` is not `0xE0`, `0xD0`, or `0xF0`.

## Postconditions

1. `parse_cotp_header(tpkt_payload)` returns `None`.
2. No panic occurs for any of the 13 remaining `u8` high-nibble values not covered by
   BC-2.20.007/008/009/010 (`0x0`, `0x1`, `0x2`, `0x3`, `0x4`, `0x5`, `0x6`, `0x7`,
   `0x8`, `0x9`, `0xA`, `0xB`, `0xC`).
3. `S7commAnalyzer` treats this `None` as "unparseable COTP payload" — the same
   consequence as the "left unclassified" arm of ADR-014 Decision 2's disambiguation
   table (an unparseable DT payload is grouped with the `Some(other)` protocol-ID case
   for classification purposes: neither is ever misattributed to S7comm).

## Invariants

1. **`CotpTpduType` is exhaustive over exactly 3 variants, not all ISO 8073 TPDU
   types**: this is a deliberate scope decision (ADR-014 Decision 1), not an
   oversight — S7comm's ISO-on-TCP profile in practice only exercises CR, CC, and DT.
2. **No force-fit**: the function never coerces an unrecognized high nibble into the
   "closest" recognized variant (e.g., DR is never treated as a degenerate DT).
3. **Purity**: total and safe over all 256 possible values of `tpkt_payload[1]` — the
   high-nibble match is exhaustively partitioned across BC-2.20.007 (`0xE`), BC-2.20.008
   (`0xD`), BC-2.20.009/010 (`0xF`), and this BC (all 13 remaining nibble values).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tpkt_payload[1] & 0xF0 == 0x80` (DR, Disconnect Request) | Returns `None` — not modeled |
| EC-002 | `tpkt_payload[1] & 0xF0 == 0xC0` (DC, Disconnect Confirm) | Returns `None` — not modeled |
| EC-003 | `tpkt_payload[1] & 0xF0 == 0x70` (ER, TPDU Error) | Returns `None` — not modeled |
| EC-004 | `tpkt_payload[1] == 0x00` (all-zero TPDU-code byte, no valid high nibble) | Returns `None` |
| EC-005 | An MMS or ICCP/TASE.2 session using a COTP TPDU type wirerust does not model | Returns `None` — this traffic is never misattributed to S7comm at the COTP layer; it surfaces (if at all) through the unclassified-port-count mechanism, not through S7comm-specific findings |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x02, 0x80, 0x00]` (DR-shaped) | `None` | reject: unrecognized TPDU type (DR) |
| `[0x02, 0xC0, 0x00]` (DC-shaped) | `None` | reject: unrecognized TPDU type (DC) |
| `[0x02, 0x70, 0x00]` (ER-shaped) | `None` | reject: unrecognized TPDU type (ER) |
| `[0x02, 0x00, 0x00]` (all-zero) | `None` | reject: no valid high nibble |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all 13 high-nibble values not in `{0xD, 0xE, 0xF}`, `parse_cotp_header` returns `None`; the four-way partition (CR/CC/DT/unrecognized) over all `u8` values of `tpkt_payload[1] & 0xF0` is exhaustive and non-overlapping | proptest P1 (protocol-ID/TPDU-type branch totality per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the never-misattribute boundary of the COTP parser, directly implementing ADR-014 Decision 2's "left unclassified, never misattributed" correctness property one layer down |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decisions 1, 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — unparseable frames are silently left unclassified, not flagged as anomalies at this layer) |

## Related BCs

- BC-2.20.007 — composes with (CR recognition, complement)
- BC-2.20.008 — composes with (CC recognition, complement)
- BC-2.20.009 — composes with (DT recognition, complement)
- BC-2.20.010 — composes with (DT-empty-payload recognition, complement)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header`: exhaustive `match tpkt_payload[1] & 0xF0 { 0xE0 => .., 0xD0 => .., 0xF0 => .., _ => None }`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — frozen `CotpTpduType` enum, exactly 3 variants
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — "a COTP DT-TPDU on port 102 whose protocol-ID is not `0x32` or `0x72` must never be misattributed to S7comm" (analogous never-misattribute guarantee, one layer up)

## Story Anchor

STORY-185

## VP Anchors

- VP-049 (Kani P0) — COTP Header Parse Safety, TPDU-Type Exhaustiveness, and
  Protocol-ID Extraction Totality; registered F2 INTEGRATE sub-burst per VP-INDEX.md
  v2.48; traces BC-2.20.005..012 (supersedes this BC's own speculative "proptest P1"
  candidate note — the registered VP-049 is Kani and covers this property)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
