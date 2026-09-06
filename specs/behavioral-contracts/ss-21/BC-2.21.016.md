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

# BC-2.21.016: PLC Stop (FC 0x29) Classified — Dedicated STOP Request, No Service-String Ambiguity

## Description

`FC == 0x29` is a dedicated PLC Stop request — unlike `0x28` (BC-2.21.015), it carries
no multiplexed service-string field; the FC byte alone fully identifies the operation.
This BC classifies `FC == 0x29` as `S7ClassicFunction::PlcStop` directly, with no
further parameter-block decode required. It is packaged separately from BC-2.21.015
specifically because it does **not** share `0x28`'s ambiguity — conflating the two into
one BC would understate the material difference in decode complexity ADR-014 flags for
`0x28` alone.

## Preconditions

1. `header.rosctr ∈ {Rosctr::Job, Rosctr::AckData}`.
2. The parameter block is bounds-validated per BC-2.21.009 and `param_length >= 1`.
3. `data[header_len] == 0x29`.

## Postconditions

1. The frame is classified `S7ClassicFunction::PlcStop`.
2. No sub-operation decode is required or attempted — `0x29` has exactly one meaning.

## Invariants

1. **No ambiguity, no decode**: `PlcStop` is the simplest classification arm in this
   group — a direct FC-to-variant mapping — precisely because the real protocol
   affords it that simplicity, unlike `0x28`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `param_length == 0` following the FC byte (no additional parameters) | Still classified `PlcStop` — the FC byte alone is sufficient; an empty remainder is expected, not anomalous |

## Canonical Test Vectors

| `data[header_len]` | Expected classification | Category |
|---|---|---|
| `0x29` | `PlcStop` | happy-path |

## Verification Properties

(No independent VP-NNN — single-arm unit test; totality covered by the shared match
anchored to BC-2.21.017.)

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
| MITRE Techniques | T0858 (Change Operating Mode, run→stop) — **classification surface only; emission is authored in part B2** |

## Related BCs

- BC-2.21.015 — composes with (the `0x28` sibling operating-mode-change function, contrasted for decode complexity)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7ClassicFunction::PlcStop` match arm

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
