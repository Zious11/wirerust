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

# BC-2.21.011: Job/Ack_Data Function-Code Byte Classifies Read Var (FC 0x04)

## Description

`FC == 0x04` classifies as `S7ClassicFunction::ReadVar` — a read request/response for
I/O, data-block, marker, timer, or counter memory. Per the source research, Read Var
is monitoring/reconnaissance context but carries no direct MITRE ATT&CK for ICS
technique mapping in this feature's seeded set (Read Var is passively distinguishable
from Write Var by FC alone, which is the entire point of this classification: it is
what lets B2 correctly *exclude* read-only traffic from any write-indicating
technique). This BC does not attempt item-descriptor (area-code) decoding for Read
Var — area-code extraction is defined only for Write Var (BC-2.21.012), since no
seeded MITRE technique in this feature keys on a *read* target area.

## Preconditions

1. `header.rosctr ∈ {Rosctr::Job, Rosctr::AckData}`.
2. The parameter block is bounds-validated per BC-2.21.009 and `param_length >= 1`.
3. `data[header_len] == 0x04`.

## Postconditions

1. The frame is classified `S7ClassicFunction::ReadVar`.
2. No area-code or item-descriptor decoding is performed for Read Var in this part
   (B1) — flagged as a possible future extension if a Read-Var-keyed technique is
   seeded later, but out of scope for the current MITRE technique set.

## Invariants

1. **Read/Write asymmetry is intentional**: Read Var's classification is coarser than
   Write Var's (BC-2.21.012) because the seeded technique set has no read-target
   detection predicate — this asymmetry is a scope decision grounded in the MITRE
   research brief, not an oversight.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `param_length == 0` | See BC-2.21.017's shared empty-parameter-block treatment |
| EC-002 | A Read Var request (`Job`) is immediately followed by an Ack_Data response with the same FC and a large `data_length` (bulk read) | Both frames classified `ReadVar` independently; correlating request/response volume for a discovery-technique signal is a B2 concern, not decided here |

## Canonical Test Vectors

| `header.rosctr` / `data[header_len]` | Expected classification | Category |
|---|---|---|
| `Job` / `0x04` | `ReadVar` | happy-path: request |
| `AckData` / `0x04` | `ReadVar` | happy-path: response |

## Verification Properties

(No independent VP-NNN — covered by the shared classification match totality anchored
to BC-2.21.017.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — no seeded technique keys on Read Var per `.factory/research/s7comm-mitre-ics-tagging.md`) |

## Related BCs

- BC-2.21.010 — composes with (sibling FC classification arm)
- BC-2.21.012 — composes with (the Write Var counterpart with area-code decoding)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7ClassicFunction::ReadVar` match arm

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
