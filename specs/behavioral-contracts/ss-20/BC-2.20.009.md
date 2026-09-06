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

# BC-2.20.009: `parse_cotp_header` Recognizes Data Transfer (DT) TPDU With Non-Empty Payload — Extracts `protocol_id`

## Description

A COTP Data Transfer (DT) TPDU is identified by the high nibble `0xF` of the
TPDU-code byte at `tpkt_payload[1]` (ISO 8073 §13.9/Table 5). DT carries the
steady-state upper-layer payload, prefixed by a single protocol-ID byte (`0x32`
classic S7comm, `0x72` S7comm-plus, or any other observed value — this function does
**not** interpret the byte, see BC-2.20.012). For the minimal, non-extended
class-0 DT format used by ISO-on-TCP S7comm traffic, `LI == 2` (TPDU-code byte + one
TPDU-NR/EOT byte), so `payload_offset = 1 + LI = 3`. When at least one byte follows
`payload_offset` in `tpkt_payload`, that byte is the protocol-ID.

## Preconditions

1. `tpkt_payload.len() >= 1 + LI` (LI-truncation check passed — BC-2.20.006).
2. `tpkt_payload[1] & 0xF0 == 0xF0` (DT TPDU-code high nibble).
3. `tpkt_payload.len() > payload_offset` where `payload_offset = 1 + LI` — at least one
   byte of upper-layer payload is present.

## Postconditions

1. `parse_cotp_header(tpkt_payload)` returns
   `Some(CotpHeader { tpdu_type: CotpTpduType::DataTransfer, protocol_id: Some(tpkt_payload[payload_offset]), payload_offset })`.
2. `protocol_id` is the byte at `tpkt_payload[payload_offset]` verbatim, with no
   validation or interpretation of its value (BC-2.20.012).
3. `payload_offset == 1 + LI`; for the minimal class-0 non-extended DT format,
   `LI == 2` and `payload_offset == 3`.
4. `S7commAnalyzer` (SS-21) branches on the returned `protocol_id` to decide
   classic-S7comm full dissection (`Some(0x32)`), S7comm-plus framing-only
   classification (`Some(0x72)`), or leaves the traffic unclassified (any other value)
   — per ADR-014 Decision 2. This branching is entirely outside SS-20's scope.

## Invariants

1. **DT carries the upper-layer payload** (ADR-014 Decision 1) — the structural
   opposite of CR/CC.
2. **No interpretation of the protocol-ID byte** — `parse_cotp_header` extracts the
   byte but performs zero comparison against `0x32`/`0x72`/any other value; keeping
   SS-20 genuinely protocol-agnostic is the entire rationale for the two-module split
   (ADR-014 Decision 1).
3. **Non-extended class-0 format assumed**: this BC covers the minimal `LI == 2` DT
   format used by S7comm's ISO-on-TCP profile. An extended-format DT TPDU (larger LI,
   additional TPDU-NR bytes for windowed transport classes) is not part of S7comm's
   observed wire behavior and is out of scope for this cycle; if encountered, the
   general LI-based `payload_offset = 1 + LI` computation still applies correctly (the
   function does not hardcode `LI == 2`), but no canonical test vector exercises it.
4. **Purity**: no state mutation; deterministic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tpkt_payload[1] == 0xF0` (canonical DT, EOT bit set, single-segment) | Recognized as DT |
| EC-002 | `protocol_id byte == 0x32` (classic S7comm) | `Some(0x32)` extracted verbatim; SS-21 disambiguates, not this function |
| EC-003 | `protocol_id byte == 0x72` (S7comm-plus) | `Some(0x72)` extracted verbatim; SS-21 disambiguates, not this function |
| EC-004 | `protocol_id byte == 0x00` or any other value not `0x32`/`0x72` (MMS, ICCP, or unrecognized) | `Some(byte)` extracted verbatim — never coerced to `None` or force-fit; SS-21's disambiguation table (ADR-014 Decision 2) leaves this unclassified downstream |
| EC-005 | `tpkt_payload.len() == payload_offset` exactly (DT header present but zero payload bytes) | Falls to BC-2.20.010, not this BC (`protocol_id: None`) |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x02, 0xF0, 0x80, 0x32, ...S7comm payload bytes]` (LI=2, DT, TPDU-NR=0x80, protocol-ID=0x32) | `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: Some(0x32), payload_offset: 3})` | happy-path: classic S7comm |
| `[0x02, 0xF0, 0x80, 0x72, ...S7comm-plus payload bytes]` (protocol-ID=0x72) | `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: Some(0x72), payload_offset: 3})` | happy-path: S7comm-plus |
| `[0x02, 0xF0, 0x80, 0x01, ...arbitrary bytes]` (protocol-ID=0x01, e.g. simulating MMS/ICCP) | `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: Some(0x01), payload_offset: 3})` | happy-path: non-S7comm protocol-ID passed through verbatim |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all inputs with `tpkt_payload[1] & 0xF0 == 0xF0` and sufficient trailing bytes, `parse_cotp_header` returns `Some` with `tpdu_type: DataTransfer` and `protocol_id` equal to the verbatim trailing byte for every possible `u8` value (protocol-ID branch totality) | proptest P1 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines DT-TPDU recognition and protocol-ID extraction, the frozen SS-20→SS-21 handoff (ADR-014 Decision 1) that every S7comm and S7comm-plus classification depends on |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decisions 1, 2, 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure parse/extraction function; classification and finding emission are SS-21 concerns) |

## Related BCs

- BC-2.20.006 — composes with (LI-truncation check)
- BC-2.20.007 — composes with (sibling: CR TPDU recognition)
- BC-2.20.008 — composes with (sibling: CC TPDU recognition)
- BC-2.20.010 — composes with (DT-with-empty-payload variant: `protocol_id: None`)
- BC-2.20.011 — composes with (complement: unrecognized TPDU-type codes)
- BC-2.20.012 — depends on (non-interpretation guarantee for the extracted `protocol_id`)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `pub struct CotpHeader { pub tpdu_type: CotpTpduType, pub protocol_id: Option<u8>, pub payload_offset: usize }` (frozen, ADR-014 Decision 1)
- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header`: DT-recognition branch, `tpkt_payload[1] & 0xF0 == 0xF0`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — "`Some(byte)` only for a DT-TPDU whose payload begins with a recognized protocol-ID byte ... `None` for CR/CC ... or when the DT payload is empty"
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — in-analyzer disambiguation table consuming this BC's `protocol_id` output

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
