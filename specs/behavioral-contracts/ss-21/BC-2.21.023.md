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

# BC-2.21.023: Unrecognized Userdata Function Group Classified `OtherGroup(group, subfn)` — Totality of the Userdata Group Match; No Invented Security-Group ID

## Description

This BC is the terminal fallback arm and totality anchor for the Userdata
function-group classification (BC-2.21.019 through 022): any function-group nibble
other than `0x03` (Block functions), `0x04` (CPU functions), or `0x07` (Time
functions) classifies as `S7UserdataFunction::OtherGroup(group, subfn)`. The source
research (`.factory/research/s7comm-mitre-ics-tagging.md`) verifies exactly these
three groups with confidence; this feature deliberately does **not** assert a specific
group ID for "Security" functions or any other Userdata group category some secondary
sources describe, since no verified, web-grounded group-ID mapping for such a category
was established during F1/F2 research — asserting an unverified group number would
violate this feature's evidence-grounding discipline (mirroring the MITRE research's
own "no training-data-only claims" standard). If a future research pass verifies
additional group IDs (e.g. Security, group `0x02` per some prose sources), they are
added as new named arms at that time, following BC-2.21.019/020's pattern — not
retrofitted into this BC's catch-all silently.

## Preconditions

1. `header.rosctr == Rosctr::Userdata`, `param_length >= 7` (BC-2.21.018 passed).
2. The function-group nibble is not `0x03`, `0x04`, or `0x07`.

## Postconditions

1. The frame is classified `S7ClassicFunction::Userdata(S7UserdataFunction::
   OtherGroup(group, subfn))` where `group` is the low nibble of `data[header_len + 4]`
   and `subfn = data[header_len + 5]` — both raw values preserved.
2. No `Finding` is emitted at the B1 dissection layer for an unrecognized-but-parseable
   group value — this is a coverage-gap-shaped observation (an unenumerated but
   structurally valid Userdata group), not itself a malformed-frame condition.

## Invariants

1. **Totality of the Userdata group match**: every possible 4-bit group nibble value
   (16 total) maps to exactly one outcome across BC-2.21.019/020/021/022/this BC — 3
   named groups (`0x03`, `0x04`, `0x07`), 1 catch-all covering the remaining 13 values.
2. **No invented group IDs**: this BC's catch-all exists specifically so the feature
   never has to guess at an unverified group-ID/category mapping (e.g. "Security") —
   evidence-grounding discipline takes precedence over classification completeness.
3. **No force-fit, terminal confirmation**: mirrors BC-2.21.017's role for the
   Job/Ack_Data FC match — this is the third and final "never force-fit" totality gate
   in this feature's dissection chain (SS-20 protocol_id/TPDU-type, Group 3 FC, Group 4
   Userdata function-group).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Group `0x00` | `OtherGroup(0x00, subfn)` |
| EC-002 | Group `0x02` (a value some non-verified prose sources associate with "Security" or "cyclic data") | `OtherGroup(0x02, subfn)` — this feature does not assert the "Security" label without independent web-grounded verification |
| EC-003 | Group `0x0F` (maximum 4-bit value) | `OtherGroup(0x0F, subfn)` |

## Canonical Test Vectors

| Group nibble | Expected classification | Category |
|---|---|---|
| `0x00` | `OtherGroup(0x00, subfn)` | edge-case: unrecognized group |
| `0x02` | `OtherGroup(0x02, subfn)` | edge-case: no invented "Security" label |
| `0x03` (negative control) | `BlockFunctions` (BC-2.21.019), never `OtherGroup` | regression-guard |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| The full Userdata function-group classification (BC-2.21.019 through this BC) is total and non-overlapping over all 16 possible 4-bit group nibble values | proptest P1 (mirrors VP-046's totality treatment) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the totality guarantee completing the Userdata classification surface |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 (Userdata Security/Time noted "as applicable" — this BC documents that Security was not independently verified this cycle and is therefore left in the generic catch-all rather than force-fit to an unverified group ID) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — negative-classification/totality contract) |

## Related BCs

- BC-2.21.019, BC-2.21.020, BC-2.21.021, BC-2.21.022 — composes with (all sibling Userdata group classification arms; this BC is their totality anchor)
- BC-2.21.017 — composes with (the analogous totality anchor for the Job/Ack_Data FC match)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7UserdataFunction::OtherGroup(u8, u8)` terminal match arm

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — proptest P1, anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — proptest P1 target |
