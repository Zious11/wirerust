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

# BC-2.21.010: Job/Ack_Data Function-Code Byte Classifies Setup Communication (FC 0xF0)

## Description

For `rosctr ∈ {Job, AckData}` (Postcondition of BC-2.21.006/007's ROSCTR gate) with a
bounds-validated, non-empty parameter block (BC-2.21.009), the first byte of the
parameter block (`data[header_len]`) is the S7comm function code (FC). This BC and
BC-2.21.011 through BC-2.21.017 jointly define `S7ClassicFunction`, the classification
label enum that part B2 maps MITRE ATT&CK for ICS techniques onto. `FC == 0xF0`
classifies as `S7ClassicFunction::SetupCommunication` — the session-negotiation
function exchanged once per S7comm session (analogous to IEC-104's STARTDT, but
carrying protocol-version/PDU-size negotiation parameters rather than a bare control
function). Setup Communication applies symmetrically to both Job (request) and
Ack_Data (response) ROSCTR — the same FC byte identifies the operation in both
directions of the exchange (this BC's modeling decision, stated once here and inherited
by BC-2.21.011 through BC-2.21.016).

## Preconditions

1. `header.rosctr ∈ {Rosctr::Job, Rosctr::AckData}`.
2. The parameter block is bounds-validated per BC-2.21.009 and `param_length >= 1`.
3. `data[header_len] == 0xF0`.

## Postconditions

1. The frame is classified `S7ClassicFunction::SetupCommunication`.
2. No further parameter-block bytes (protocol version, max AMQ, PDU size negotiation
   fields) are interpreted by this BC — Setup Communication's negotiated parameters
   carry no MITRE-technique-relevant signal per the source research
   (`.factory/research/s7comm-mitre-ics-tagging.md`) and are out of B1 dissection
   scope beyond FC-level classification.
3. This classification applies identically whether `header.rosctr == Job` (request) or
   `header.rosctr == AckData` (response) — `S7ClassicFunction::SetupCommunication`
   carries no request/response discriminant of its own; direction is available
   separately from the flow's `c2s`/`s2c` delivery direction if a future consumer
   needs it.

## Invariants

1. **Symmetric Job/Ack_Data FC semantics**: this BC establishes the modeling
   convention — stated once, applying to every FC value in this group — that the same
   `S7ClassicFunction` variant set covers both request and response PDUs, since the FC
   byte position and meaning do not change between the two ROSCTR values.
2. **No force-fit**: FC `0xF0` maps to exactly one variant; no other FC value maps to
   `SetupCommunication`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `param_length == 0` (no FC byte present at all) — see BC-2.21.017 Edge Case for the shared empty-parameter-block treatment | Classified as `S7ClassicFunction::Unrecognized`-adjacent "no function code present" case, defined once in BC-2.21.017 and referenced by every classification BC in this group |
| EC-002 | `data[header_len] == 0xF0` but `header.rosctr == Userdata` | Not reachable — Userdata's parameter block has a structurally different layout (BC-2.21.018); this precondition's ROSCTR gate prevents cross-interpretation |

## Canonical Test Vectors

| `header.rosctr` / `data[header_len]` | Expected classification | Category |
|---|---|---|
| `Job` / `0xF0` | `SetupCommunication` | happy-path: request |
| `AckData` / `0xF0` | `SetupCommunication` | happy-path: response |

## Verification Properties

(No independent VP-NNN — classification-label mapping verified by table-driven unit
tests, mirroring IEC-104's function-code/TypeID classification precedent, BC-2.19.019
et al. The proptest P1 totality obligation for the full FC match is anchored to
BC-2.21.017, the terminal `Unrecognized` fallback arm.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — function-code classification is the core dissection behavior CAP-21's description names ("full S7comm PDU dissection (function codes...)") |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 (function-code table, informational for classification — MITRE emission is B2 scope) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none emitted by this BC — Setup Communication carries no MITRE mapping per `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis, "session negotiation; scan/flood context" is a T0814/T0846 *aggregate* signal, not a per-PDU FC 0xF0 tag; B2 decides whether/how to use Setup Communication frequency as burst-detection evidence) |

## Related BCs

- BC-2.21.006 — depends on (ROSCTR/param_length this classification reads)
- BC-2.21.009 — depends on (bounds check precedes FC-byte access)
- BC-2.21.011 through BC-2.21.017 — composes with (sibling FC classification arms of the same match)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `fn classify_job_ackdata_function(fc: u8) -> S7ClassicFunction`
- `enum S7ClassicFunction { SetupCommunication, ReadVar, WriteVar(S7AreaCode), RequestDownload, DownloadBlock, DownloadEnded, StartUpload, Upload, EndUpload, PlcControl(PlcControlService), PlcStop, Unrecognized(u8) }` (planned, this BC's design — the classification surface part B2 maps MITRE techniques onto)
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis — FC table source

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated — covered by BC-2.21.017's totality proptest.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
