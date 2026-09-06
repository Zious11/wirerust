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

# BC-2.20.008: `parse_cotp_header` Recognizes Connect Confirm (CC) TPDU

## Description

A COTP Connect Confirm (CC) TPDU is identified by the high nibble `0xD` of the
TPDU-code byte at `tpkt_payload[1]` (ISO 8073 §13.4/Table 5) — the responder's
acknowledgment completing session establishment after a CR. Like CR, CC carries no
upper-layer payload. `parse_cotp_header` returns
`CotpHeader { tpdu_type: ConnectConfirm, protocol_id: None, payload_offset }` with the
same `payload_offset = 1 + LI` derivation as BC-2.20.007.

## Preconditions

1. `tpkt_payload.len() >= 1 + LI` (LI-truncation check passed — BC-2.20.006).
2. `tpkt_payload[1] & 0xF0 == 0xD0` (CC TPDU-code high nibble).

## Postconditions

1. `parse_cotp_header(tpkt_payload)` returns
   `Some(CotpHeader { tpdu_type: CotpTpduType::ConnectConfirm, protocol_id: None, payload_offset })`.
2. `payload_offset == 1 + LI`.
3. `protocol_id` is unconditionally `None` — identical reasoning to BC-2.20.007
   Postcondition 3.
4. `S7commAnalyzer` observing a CC after a tracked CR marks the session as
   established on `S7commFlowState`; classification of the application protocol is
   still deferred to the first DT frame.

## Invariants

1. **CC carries no upper-layer payload** (ADR-014 Decision 1), symmetric with CR.
2. **High-nibble discrimination only** — same as BC-2.20.007 Invariant 2.
3. **Purity**: no state mutation; deterministic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tpkt_payload[1] == 0xD0` (canonical CC) | Recognized as CC |
| EC-002 | `tpkt_payload[1] == 0xD1` (CC with non-zero low nibble) | Still recognized as CC (high-nibble-only) |
| EC-003 | A CC observed with no preceding CR tracked in `S7commFlowState` (out-of-order or mid-stream capture join) | `parse_cotp_header` still returns `Some(CotpHeader{..ConnectConfirm..})` — this BC is stateless; the flow-state-level "CC without CR" observation is a SS-21 concern, out of CAP-20's scope |
| EC-004 | Minimal CC with no variable parameters, `LI == 6` | `payload_offset == 7` |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x06, 0xD0, 0x00, 0x01, 0x00, 0x00, 0x00]` (LI=6, CC, DST-REF=0x0001, SRC-REF=0x0000, class=0) | `Some(CotpHeader{tpdu_type: ConnectConfirm, protocol_id: None, payload_offset: 7})` | happy-path: minimal CC |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all inputs with `tpkt_payload[1] & 0xF0 == 0xD0` and sufficient length, `parse_cotp_header` returns `Some` with `tpdu_type: ConnectConfirm` and `protocol_id: None` | proptest P1 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines CC-TPDU recognition, the second of three frozen `CotpTpduType` variants |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decision 1 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — session establishment observation only) |

## Related BCs

- BC-2.20.006 — composes with (LI-truncation check)
- BC-2.20.007 — composes with (sibling: CR TPDU recognition)
- BC-2.20.009 — composes with (sibling: DT TPDU recognition)
- BC-2.20.011 — composes with (complement: unrecognized TPDU-type codes)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header`: CC-recognition branch, `tpkt_payload[1] & 0xF0 == 0xD0`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — "CR (Connect Request) and CC (Connect Confirm) TPDUs perform session establishment and carry no upper-layer payload"

## Story Anchor

STORY-185

## VP Anchors

- VP-049 (Kani P0) — COTP Header Parse Safety, TPDU-Type Exhaustiveness, and
  Protocol-ID Extraction Totality; registered F2 INTEGRATE sub-burst per VP-INDEX.md
  v2.48; traces BC-2.20.005..012

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
