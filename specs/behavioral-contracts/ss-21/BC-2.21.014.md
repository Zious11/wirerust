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

# BC-2.21.014: Upload Sequence Classified — Start Upload (0x1D), Upload (0x1E), End Upload (0x1F) — Distinguished From Program Download

## Description

Classic S7comm's block-upload-from-PLC sequence (Start Upload `0x1D`, Upload `0x1E`,
End Upload `0x1F`) transfers a block **from** the PLC to the engineering station —
backup or collection traffic, structurally and directionally the inverse of the
program-download triad (BC-2.21.013). Per the source research
(`.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis), Upload is
explicitly **not** program deployment and must never be classified or later tagged as
Program Download/Modify Program — this BC exists specifically to guarantee that
separation at the classification layer, since a naive FC-range check (`0x1A..=0x1F`)
could otherwise conflate the two triads.

## Preconditions

1. `header.rosctr ∈ {Rosctr::Job, Rosctr::AckData}`.
2. The parameter block is bounds-validated per BC-2.21.009 and `param_length >= 1`.
3. `data[header_len] ∈ {0x1D, 0x1E, 0x1F}`.

## Postconditions

1. `data[header_len] == 0x1D` classifies as `S7ClassicFunction::StartUpload`.
2. `data[header_len] == 0x1E` classifies as `S7ClassicFunction::Upload`.
3. `data[header_len] == 0x1F` classifies as `S7ClassicFunction::EndUpload`.
4. None of the three Upload variants is ever classified as, aliased to, or conflated
   with `RequestDownload`/`DownloadBlock`/`DownloadEnded` (BC-2.21.013) — the two
   triads are structurally and semantically disjoint despite adjacent FC-value ranges
   (`0x1A`-`0x1C` vs. `0x1D`-`0x1F`).

## Invariants

1. **Directional semantics are load-bearing for B2**: Upload = PLC→station (backup),
   Download = station→PLC (deployment) — B2 must never emit T0843 (Program Download)
   or T0889 (Modify Program) from an Upload-classified sequence; this BC's separation
   guarantee is what makes that correctness property enforceable downstream.
2. **No shared match arm**: the FC-classification match treats `0x1A..=0x1C` and
   `0x1D..=0x1F` as two disjoint sub-ranges, never a single `0x1A..=0x1F` range
   collapsed to one variant.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A flow observes a Download sequence (`0x1A`) immediately followed by an Upload sequence (`0x1D`) — e.g. an engineer verifying a just-deployed block by reading it back | Both classified independently and correctly; no cross-contamination between the two sequences' classification state |
| EC-002 | `0x1E` (Upload) with no preceding `0x1D` on the flow | Classified `Upload` regardless — out-of-sequence detection is a B2 concern, mirroring BC-2.21.013 Edge Case EC-001 |

## Canonical Test Vectors

| `data[header_len]` | Expected classification | Category |
|---|---|---|
| `0x1D` | `StartUpload` | happy-path |
| `0x1E` | `Upload` | happy-path |
| `0x1F` | `EndUpload` | happy-path |
| `0x1A` (adjacent Download value, negative control) | `RequestDownload`, never `StartUpload` | regression-guard: no triad confusion |

## Verification Properties

(No independent VP-NNN — table-driven unit tests; totality covered by the shared match
anchored to BC-2.21.017. A dedicated regression-guard test asserts the `0x1A..=0x1C`
and `0x1D..=0x1F` sub-ranges never share a match arm.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the Upload/Download separation this capability's dissection scope must preserve to avoid false T0843/T0889 evidence |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — Upload is explicitly excluded from T0843/T0889 evidence per the source research; this BC's contract is a negative-evidence guarantee for B2, not a positive emission surface) |

## Related BCs

- BC-2.21.013 — composes with (the structurally parallel, disjoint Download triad this BC must never be confused with)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7ClassicFunction::{StartUpload, Upload, EndUpload}` match arms, kept structurally separate from the Download arms
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis — "`0x1D` Start Upload: block upload PLC→station (backup/collection, **not** program download)"

## Story Anchor

STORY-188

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
