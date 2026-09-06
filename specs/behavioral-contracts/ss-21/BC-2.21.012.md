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

# BC-2.21.012: Job/Ack_Data Function-Code Byte Classifies Write Var (FC 0x05) With Area-Code Extraction

## Description

`FC == 0x05` classifies as `S7ClassicFunction::WriteVar(area)` — the primary write
indicator for classic S7comm, per the source research the single most MITRE-relevant
classic-S7comm function code (feeds T0835 Manipulate I/O Image and T0836 Modify
Parameter in part B2). The write target's memory area is carried in the first
S7ANY-syntax address-item descriptor within the parameter block; this BC scopes
extraction to the **first** item's area-code byte only (S7comm parameter blocks can in
principle carry multiple items per PDU; multi-item decoding is out of B1 scope — see
Invariant 2). `S7AreaCode` maps the recognized area values: `0x80` Direct Peripheral,
`0x81` Inputs, `0x82` Outputs, `0x83` Markers, `0x84` Data Block, `0x85` Instance DB,
`0x1C` Counters, `0x1D` Timers; any other byte value maps to
`S7AreaCode::Unrecognized(byte)` — never force-fit to one of the eight named areas.

## Preconditions

1. `header.rosctr ∈ {Rosctr::Job, Rosctr::AckData}`.
2. The parameter block is bounds-validated per BC-2.21.009 and `param_length >= 1`.
3. `data[header_len] == 0x05`.

## Postconditions

1. The frame is classified `S7ClassicFunction::WriteVar(area)`.
2. If the parameter block contains a well-formed first address-item descriptor with a
   readable area-code byte at its expected offset (implementation-detail offset within
   the S7ANY item-descriptor convention, deferred to architect/implementer — the
   *values* this BC pins are load-bearing, the exact byte offset within a
   variable-length item descriptor is not), `area` is set per the mapping table above.
3. If the item descriptor cannot be read (insufficient remaining parameter-block
   bytes for a full item descriptor, or a non-S7ANY syntax ID that this feature does
   not decode), `area` is `S7AreaCode::Unrecognized(0xFF)`-equivalent placeholder —
   concretely, the classification remains `WriteVar` with an area value signaling
   "not decoded," never a hard reject of the whole frame (FC-level classification is
   still valid even when finer address decoding fails).
4. Multi-item Write Var parameter blocks (more than one address item in a single PDU)
   are classified using only the first item's area code; any additional items are not
   independently classified in this part.

## Invariants

1. **Area-code value mapping is exhaustive-but-open**: exactly 8 named values plus an
   `Unrecognized(u8)` catch-all — every possible `u8` byte maps to exactly one
   `S7AreaCode` variant, with no gaps and no force-fit of an unrecognized byte into a
   named area.
2. **Single-item scope is a stated B1 boundary, not a defect**: multi-item Write Var
   PDUs exist in the real protocol (writing to several areas in one PDU); classifying
   only the first item is a deliberate scope reduction for this dissection pass,
   flagged here for a future extension rather than silently under-specified.
3. **`0x80`/`0x81`/`0x82` group corresponds to T0835's area set; `0x83`/`0x84` group
   corresponds to T0836's area set** (per ADR-014 Decision 5's reuse table) — this
   grouping is the load-bearing classification surface B2 keys its two-technique
   split on; this BC does not itself decide which group maps to which technique
   (that is B2's job), it only guarantees the area byte is available and correctly
   valued for B2 to consume.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Area byte `0x80` (Direct Peripheral) | `S7AreaCode::DirectPeripheral` |
| EC-002 | Area byte `0x86` (not one of the 8 recognized values) | `S7AreaCode::Unrecognized(0x86)` — never coerced to a named area |
| EC-003 | `param_length >= 1` but too short to contain a full item descriptor after the FC byte | `WriteVar` classification stands; area is the not-decoded placeholder (Postcondition 3) |
| EC-004 | `param_length == 0` | See BC-2.21.017's shared empty-parameter-block treatment |

## Canonical Test Vectors

| `data[header_len]` / area byte | Expected classification | Category |
|---|---|---|
| `0x05` / `0x81` (Inputs) | `WriteVar(Inputs)` | happy-path: T0835-eligible area |
| `0x05` / `0x84` (Data Block) | `WriteVar(DataBlock)` | happy-path: T0836-eligible area |
| `0x05` / `0x9A` (not one of the 8 named values) | `WriteVar(Unrecognized(0x9A))` | edge-case: unrecognized area, no force-fit |

## Verification Properties

(No independent VP-NNN — table-driven unit tests for the area-code mapping's
exhaustiveness over all 256 `u8` values; proptest P1 candidate mirroring VP-046's
totality treatment.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the area-code classification surface B2's T0835/T0836 emission call-sites key on directly (ADR-014 Decision 5) |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 (T0835/T0836 area-code table, informational for classification — emission is B2 scope) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0835 (Manipulate I/O Image, areas `0x80`/`0x81`/`0x82`), T0836 (Modify Parameter, areas `0x83`/`0x84`) — classification surface named per ADR-014 Decision 5; **emission (verdict/confidence/dedup) is authored in part B2, not this BC** |

## Related BCs

- BC-2.21.010 — composes with (sibling FC classification arm)
- BC-2.21.011 — composes with (the Read Var counterpart without area decoding)
- BC-2.21.017 — composes with (empty-parameter-block shared edge case)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7ClassicFunction::WriteVar(S7AreaCode)` match arm and item-descriptor area-byte extraction
- `enum S7AreaCode { DirectPeripheral, Inputs, Outputs, Markers, DataBlock, InstanceDb, Counters, Timers, Unrecognized(u8) }` (planned, this BC's design)
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7ANY area codes — source table

## Story Anchor

STORY-188

## VP Anchors

(None dedicated — no VP-NNN was registered for area-code mapping totality in the F2
INTEGRATE sub-burst; VP-INDEX.md v2.48 registers no VP with this BC in its source_bc.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
