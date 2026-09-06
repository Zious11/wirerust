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

# BC-2.20.007: `parse_cotp_header` Recognizes Connect Request (CR) TPDU

## Description

A COTP Connect Request (CR) TPDU is identified by the high nibble `0xE` of the
TPDU-code byte at `tpkt_payload[1]` (ISO 8073 §13.3/Table 5). CR carries no upper-layer
payload — it is a session-establishment TPDU exchanged before any S7comm data flows.
`parse_cotp_header` returns `CotpHeader { tpdu_type: ConnectRequest, protocol_id: None, payload_offset }`
where `payload_offset = 1 + LI` (the byte immediately following the fixed CR header;
per ADR-014 Decision 1, `protocol_id` is always `None` for CR since no upper-layer
payload exists at this TPDU type).

## Preconditions

1. `tpkt_payload.len() >= 1 + LI` (LI-truncation check passed — BC-2.20.006).
2. `tpkt_payload[1] & 0xF0 == 0xE0` (CR TPDU-code high nibble).

## Postconditions

1. `parse_cotp_header(tpkt_payload)` returns
   `Some(CotpHeader { tpdu_type: CotpTpduType::ConnectRequest, protocol_id: None, payload_offset })`.
2. `payload_offset == 1 + LI` where `LI = tpkt_payload[0] as usize`.
3. `protocol_id` is unconditionally `None` for a CR TPDU — no attempt is made to
   interpret any bytes beyond the fixed CR header as an upper-layer protocol-ID byte,
   even if such bytes happen to be present in `tpkt_payload` (they would be COTP
   variable parameters or padding, never upper-layer payload, at this TPDU type).
4. `S7commAnalyzer` (SS-21) uses the CR observation to begin tracking connection
   establishment on `S7commFlowState`; classification of the eventual application
   protocol (S7comm vs. S7comm-plus vs. other) is deferred until the first DT frame
   (BC-2.20.009/010).

## Invariants

1. **CR carries no upper-layer payload** (ADR-014 Decision 1) — this is a structural
   fact of the COTP class-0 connection-establishment exchange, not a heuristic.
2. **High-nibble discrimination only**: the low nibble/bits of `tpkt_payload[1]` (class
   and option bits) are not inspected by this function; only the high nibble
   distinguishes CR from CC from DT from an unrecognized TPDU type.
3. **Purity**: no state mutation; deterministic for any input satisfying the
   preconditions.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tpkt_payload[1] == 0xE0` (canonical CR, no low-bit options set) | Recognized as CR |
| EC-002 | `tpkt_payload[1] == 0xE1` (CR with a non-zero low nibble — reserved/unused bits) | Still recognized as CR (high-nibble-only discrimination) |
| EC-003 | A CR TPDU with COTP variable parameters present (LI > 6) | `payload_offset` correctly accounts for the full LI-declared header length; `protocol_id` remains `None` |
| EC-004 | Minimal CR with no variable parameters, `LI == 6` (TPDU-code + 2-byte DST-REF + 2-byte SRC-REF + 1-byte class/options) | `payload_offset == 7` |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x06, 0xE0, 0x00, 0x00, 0x00, 0x01, 0x00]` (LI=6, CR, DST-REF=0x0000, SRC-REF=0x0001, class=0) | `Some(CotpHeader{tpdu_type: ConnectRequest, protocol_id: None, payload_offset: 7})` | happy-path: minimal CR |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all inputs with `tpkt_payload[1] & 0xF0 == 0xE0` and sufficient length, `parse_cotp_header` returns `Some` with `tpdu_type: ConnectRequest` and `protocol_id: None`; `payload_offset` exactly equals `1 + LI` | proptest P1 (protocol-ID branch totality per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines CR-TPDU recognition, one of the three frozen `CotpTpduType` variants (ADR-014 Decision 1) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decision 1 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — session establishment observation only; no finding emission) |

## Related BCs

- BC-2.20.006 — composes with (LI-truncation check, evaluated before TPDU-type recognition)
- BC-2.20.008 — composes with (sibling: CC TPDU recognition)
- BC-2.20.009 — composes with (sibling: DT TPDU recognition)
- BC-2.20.011 — composes with (complement: unrecognized TPDU-type codes)
- BC-2.20.016 — depends on (frozen interface / no-per-flow-state boundary this BC's output crosses into SS-21)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `pub enum CotpTpduType { ConnectRequest, ConnectConfirm, DataTransfer }` (frozen, ADR-014 Decision 1)
- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header`: CR-recognition branch, `tpkt_payload[1] & 0xF0 == 0xE0`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — "CR (Connect Request) and CC (Connect Confirm) TPDUs perform session establishment and carry no upper-layer payload"

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
