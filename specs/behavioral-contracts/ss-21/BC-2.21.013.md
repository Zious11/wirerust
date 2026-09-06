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

# BC-2.21.013: Program-Download Sequence Classified — Request Download (0x1A), Download Block (0x1B), Download Ended (0x1C)

## Description

Classic S7comm's block-download-to-PLC sequence uses three distinct FCs in temporal
order: Request Download (`0x1A`, initiates the transfer, declares block type/number
and total length), Download Block (`0x1B`, one or more data chunks, repeated as
needed), and Download Ended (`0x1C`, terminates the transfer). Each FC is classified
independently as `S7ClassicFunction::RequestDownload`, `DownloadBlock`, and
`DownloadEnded` respectively — this BC does not itself correlate the three into a
single "download session" state machine (that correlation, needed for B2's T0843/T0889
complete-sequence detection per ADR-014 Decision 5, is a B2-authored behavior building
on this BC's per-frame classification, not a B1 concern). This BC packages the three
FCs together because they are inseparable in practice: a download session is
meaningless as a single frame, and B2 will need all three classified consistently to
build its sequence correlation.

## Preconditions

1. `header.rosctr ∈ {Rosctr::Job, Rosctr::AckData}`.
2. The parameter block is bounds-validated per BC-2.21.009 and `param_length >= 1`.
3. `data[header_len] ∈ {0x1A, 0x1B, 0x1C}`.

## Postconditions

1. `data[header_len] == 0x1A` classifies as `S7ClassicFunction::RequestDownload`.
2. `data[header_len] == 0x1B` classifies as `S7ClassicFunction::DownloadBlock`.
3. `data[header_len] == 0x1C` classifies as `S7ClassicFunction::DownloadEnded`.
4. No block-type, block-number, or block-content interpretation is performed by this
   BC — those are parameter/data-block sub-fields out of B1's FC-classification scope;
   this BC guarantees only that the three FC values are correctly and independently
   distinguished, never confused with the structurally similar Upload triad
   (BC-2.21.014).
5. Correlating a `RequestDownload → DownloadBlock (× N) → DownloadEnded` sequence on a
   single flow into one logical "download session" (the T0843/T0889 detection
   predicate per ADR-014 Decision 5) is explicitly deferred to part B2; this BC's
   contract is discharged once each individual frame is correctly classified.

## Invariants

1. **Download and Upload are structurally distinct triads never confused**: `0x1A/
   0x1B/0x1C` (download, station→PLC) and `0x1D/0x1E/0x1F` (upload, PLC→station,
   BC-2.21.014) are eight-apart in FC-space and map to entirely disjoint
   `S7ClassicFunction` variants — no shared classification path exists between them.
2. **Per-frame classification, not sequence tracking**: this BC's scope is limited to
   correctly labeling each individual frame; sequence-level correlation state (e.g., a
   `download_in_progress: bool` on `S7commFlowState`) is a B2 concern, since it exists
   solely to support MITRE emission and has no dissection-layer purpose of its own.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A flow observes `0x1B` (Download Block) with no preceding `0x1A` on the same flow | Classified `DownloadBlock` regardless — this BC does not require or check for a preceding `RequestDownload`; out-of-sequence detection is a B2 anomaly-detection concern |
| EC-002 | A flow observes `0x1A` followed immediately by `0x1C` with zero `0x1B` frames in between (empty download) | Both classified independently and correctly (`RequestDownload`, `DownloadEnded`); whether an empty download is itself anomalous is out of B1 scope |
| EC-003 | `0x28 _INSE` (PLC Control block-activate, BC-2.21.015) follows a Download Ended frame | Classified independently by BC-2.21.015; the two BCs do not share state — sequence-level "download + activate" correlation for T0889 is a B2 concern |

## Canonical Test Vectors

| `data[header_len]` | Expected classification | Category |
|---|---|---|
| `0x1A` | `RequestDownload` | happy-path |
| `0x1B` | `DownloadBlock` | happy-path |
| `0x1C` | `DownloadEnded` | happy-path |

## Verification Properties

(No independent VP-NNN — table-driven unit tests for the three FC values; totality
covered by the shared match anchored to BC-2.21.017.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the per-frame classification surface B2's T0843/T0889 complete-sequence detection is built on |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 (T0843/T0889 detection pattern references this three-FC sequence) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0843 (Program Download), T0889 (Modify Program) — **named per ADR-014 Decision 5 as the sequence this classification feeds; sequence correlation and emission are authored in part B2, not this BC** |

## Related BCs

- BC-2.21.014 — composes with (the structurally parallel, disjoint Upload triad)
- BC-2.21.015 — composes with (`0x28 _INSE`/`_DELE` PLC Control, an alternative T0889 co-tag path per ADR-014 Decision 5)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7ClassicFunction::{RequestDownload, DownloadBlock, DownloadEnded}` match arms
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table — T0843/T0889 detection-pattern source

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
