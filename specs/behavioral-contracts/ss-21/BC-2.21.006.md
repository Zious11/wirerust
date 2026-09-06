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
subsystem: SS-21
capability: CAP-21
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

# BC-2.21.006: `parse_s7comm_header` Extracts ROSCTR, PDU Reference, Parameter Length, and Data Length from a Valid 10-byte Common Header (Happy Path)

## Description

Given `data.len() >= 10` and `data[0] == 0x32` (BC-2.21.004/005 passed),
`parse_s7comm_header` extracts the common S7comm header fields: `data[1]` is the
ROSCTR byte (`0x01` Job, `0x02` Ack, `0x03` Ack_Data, `0x07` Userdata — the four
recognized values, BC-2.21.007 covers all others); `data[2..4]` is a 2-byte Reserved
field (read but not semantically interpreted); `data[4..6]` is the PDU Reference
(`u16`, big-endian); `data[6..8]` is the Parameter Length (`u16`, big-endian);
`data[8..10]` is the Data Length (`u16`, big-endian). For ROSCTR ∈ {Job, Ack_Data,
Userdata}, this 10-byte common header is the complete header (`header_len == 10`); for
ROSCTR == Ack, two additional bytes follow (BC-2.21.008).

## Preconditions

1. `data.len() >= 10`.
2. `data[0] == 0x32`.
3. `data[1] ∈ {0x01, 0x02, 0x03, 0x07}` (a recognized ROSCTR value).

## Postconditions

1. `parse_s7comm_header(data)` returns `Some(S7commHeader { rosctr, pdu_reference,
   param_length, data_length, error_class: None, error_code: None, header_len: 10 })`
   for `rosctr ∈ {Job, AckData, Userdata}` (i.e. `data[1] ∈ {0x01, 0x03, 0x07}`); for
   `data[1] == 0x02` (Ack), control passes to BC-2.21.008 instead (this BC's
   Postconditions apply only to the three non-Ack values).
2. `pdu_reference = u16::from_be_bytes([data[4], data[5]])`.
3. `param_length = u16::from_be_bytes([data[6], data[7]])`.
4. `data_length = u16::from_be_bytes([data[8], data[9]])`.
5. `data[2..4]` (Reserved) is read for header-length bookkeeping but never compared,
   matched, or branched on.
6. No bounds-consistency check between `param_length`/`data_length` and the actual
   remaining bytes in `data` occurs in this function — that check is BC-2.21.009's
   responsibility, applied by the caller after this function returns `Some`.

## Invariants

1. **Field order is fixed**: Protocol ID, ROSCTR, Reserved, PDU Reference, Parameter
   Length, Data Length — this ordering is a structural fact of the S7comm wire format
   (per free-to-read prose sources, ADR-014 Decision 4), not a design choice.
2. **Big-endian multi-byte fields**: PDU Reference, Parameter Length, and Data Length
   are all big-endian `u16` — consistent across all ROSCTR values.
3. **Purity**: no state mutation; deterministic; no panic for any `data.len() >= 10`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `param_length == 0` and `data_length == 0` (Setup Communication response, or a Userdata frame with all information in the parameter block only) | Extracted normally; a zero-length parameter/data block is not itself an error at this layer |
| EC-002 | `pdu_reference == 0x0000` | Extracted verbatim; PDU reference `0` is not treated as invalid at the header-parse layer (used only for later request/response correlation, out of B1 scope) |
| EC-003 | Reserved bytes (`data[2..4]`) are non-zero (spec typically expects `0x0000`) | Extracted and discarded; no rejection — the field is documented as read-but-unvalidated by Postcondition 5 |

## Canonical Test Vectors

| Input (`data`, hex bytes) | Expected `S7commHeader` | Category |
|---|---|---|
| `32 01 00 00 00 01 00 02 00 00` | `{rosctr: Job, pdu_reference: 1, param_length: 2, data_length: 0, header_len: 10}` | happy-path: Job, minimal |
| `32 03 00 00 00 01 00 02 00 04` | `{rosctr: AckData, pdu_reference: 1, param_length: 2, data_length: 4, header_len: 10}` | happy-path: Ack_Data with response data |
| `32 07 00 00 00 05 00 08 00 00` | `{rosctr: Userdata, pdu_reference: 5, param_length: 8, data_length: 0, header_len: 10}` | happy-path: Userdata |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| Field extraction is correct (matches byte-for-byte expected values) for all 10-byte-minimum inputs with a recognized ROSCTR; no panic for any symbolic `data` of length ≥ 10 | cargo-fuzz P1 (combined harness) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this is the happy-path common-header extraction that every downstream function-code classification BC (010–023) depends on |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure extraction, no finding emission) |

## Related BCs

- BC-2.21.004 — composes with (length-reject precedes this path)
- BC-2.21.005 — composes with (protocol-ID guard precedes this path)
- BC-2.21.007 — composes with (unrecognized-ROSCTR sibling reject path)
- BC-2.21.008 — composes with (Ack ROSCTR's additional 2-byte requirement)
- BC-2.21.009 — depends on (this BC's `param_length`/`data_length` are consumed by the bounds check there)
- BC-2.21.010 through BC-2.21.023 — depend on (all function-code and Userdata classification is downstream of this extraction)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `fn parse_s7comm_header`, common-header field extraction
- `struct S7commHeader { rosctr: Rosctr, pdu_reference: u16, param_length: u16, data_length: u16, error_class: Option<u8>, error_code: Option<u8>, header_len: usize }` (planned, this BC's design)
- `enum Rosctr { Job, Ack, AckData, Userdata }` (planned, this BC's design)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — cargo-fuzz P1, combined harness.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — cargo-fuzz P1 target |
