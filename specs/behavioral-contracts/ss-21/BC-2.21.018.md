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
input-hash: "8f268fc"
---

# BC-2.21.018: Userdata (ROSCTR 0x07) Parameter Block Structural Parse — Parameter Head, Group/Subfunction Extraction, Bounds-Safe Reject

## Description

For `rosctr == Userdata` (`0x07`), the parameter block has a structurally different
layout from the Job/Ack_Data function-code byte (Group 3, BC-2.21.010-017): a 3-byte
Parameter Head (a fixed marker, conventionally `0x00 0x01 0x12`, identifying a
Siemens-specific Userdata parameter block), a 1-byte Parameter Length, a 1-byte
Type/Function-Group byte (high nibble = request/response type, low nibble = function
group), a 1-byte Subfunction, and a 1-byte Sequence Number — 7 bytes minimum. This BC
defines the structural parse and its bounds-safe-reject path; BC-2.21.019 through
BC-2.21.023 define the function-group-specific classification built on top of it. The
Parameter Head's 3 fixed bytes are checked for presence but not semantically branched
on beyond the length/marker sanity check; the Sequence Number is extracted but not
interpreted (correlation with a later response is out of B1 scope).

## Preconditions

1. `header.rosctr == Rosctr::Userdata`.
2. The parameter block is bounds-validated per BC-2.21.009.

## Postconditions

1. If `param_length < 7` (insufficient bytes for Parameter Head + Parameter Length +
   Type/Group + Subfunction + Sequence Number): the Userdata parameter block is
   treated as malformed — one T0814 (Anomaly/Possible/Medium) per flow direction,
   sharing the `malformed_header_reported_c2s`/`_s2c` dedup flag (BC-2.21.001,
   BC-2.21.004/007/008/009) — no function-group classification is attempted.
2. If `param_length >= 7`: the function group is extracted as the low nibble of
   `data[header_len + 4]` (the Type/Group byte), and the subfunction as
   `data[header_len + 5]`.
3. The Parameter Head (`data[header_len..header_len+3]`) is read for presence but its
   exact byte values are not validated against the conventional `0x00 0x01 0x12`
   marker in this BC — a mismatch does not itself trigger rejection (this feature's
   dissection scope treats the marker as informational, not a hard gate, since the
   marker's exact conventional value is drawn from prose sources, not an official
   specification, per ADR-014 Decision 4).
4. The Sequence Number (`data[header_len + 6]`) is extracted but never compared,
   matched, or branched on by B1 — available for a future request/response
   correlation extension, not used here.

## Invariants

1. **Distinct parameter-block layout from Job/Ack_Data**: this BC and BC-2.21.010-017
   define two entirely separate parameter-block interpretations, gated exclusively by
   `header.rosctr` — there is no code path that could apply Group 3's FC-byte
   interpretation to a Userdata parameter block or vice versa.
2. **7-byte minimum is a bounds-safety fact, not a semantic validation of the marker
   bytes**: Postcondition 3 clarifies that only the *length* is safety-gated; the
   marker's *content* is informational.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `param_length == 6` (one byte short of the 7-byte minimum) | Malformed Userdata parameter block; T0814 (dedup-guarded) |
| EC-002 | `param_length == 7` exactly | Group/subfunction extraction proceeds; see BC-2.21.019 onward |
| EC-003 | Parameter Head bytes are `0x00 0x01 0x13` (differs from the conventional `0x12`) | Extraction still proceeds per Postcondition 3 — no hard reject on marker mismatch |

## Canonical Test Vectors

| `param_length` | Expected outcome | Category |
|---|---|---|
| `6` | Malformed, T0814 (dedup-guarded) | reject: below 7-byte minimum |
| `7` (Type/Group=`0x03`, Subfn=`0x01`) | Group=`0x03`, Subfn=`0x01` extracted | happy-path: minimal Userdata Block-functions frame — see BC-2.21.019 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| No out-of-bounds access for any `param_length` value; correct group/subfunction extraction for `param_length >= 7` | cargo-fuzz P1 (combined harness, ADR-014 Decision 9) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the structural entry point for Userdata dissection, on which the load-bearing group-0x03/0x04/0x07 correction (BC-2.21.019-022) depends |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 (Userdata group table, informational for classification) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0814 (Denial of Service) — malformed-parameter-block anomaly signal only; emission wiring is a B2 responsibility |

## Related BCs

- BC-2.21.009 — depends on (bounds check precedes this structural parse)
- BC-2.21.019 through BC-2.21.023 — composes with (function-group-specific classification built on this parse)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `fn parse_userdata_parameter_block(data: &[u8], header_len: usize) -> Option<(u8, u8)>` (group, subfunction) or equivalent
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis — "Userdata (ROSCTR `0x07`) groups / subfunctions"

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
