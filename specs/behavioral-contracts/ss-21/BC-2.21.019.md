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

# BC-2.21.019: Userdata Block Functions (Group 0x03) Classified — List Blocks / List Blocks of Type / Get Block Info (Load-Bearing Group-0x03 Correction)

## Description

**This BC encodes the load-bearing correction ADR-014 Decision 5 and the source
research both flag explicitly**: Userdata function group `0x03` is **Block
functions** — block enumeration and metadata — with subfunctions `0x01` List Blocks,
`0x02` List Blocks of Type, `0x03` Get Block Info. This is the *reverse* of a common
documentation error some secondary sources make (mis-stating block enumeration as
group `0x07`, which is actually Time functions — BC-2.21.022). Getting this mapping
right is load-bearing for B2's T0888 (Remote System Information Discovery) emission
call-site, which keys specifically on group `0x03`.

## Preconditions

1. `header.rosctr == Rosctr::Userdata`, `param_length >= 7` (BC-2.21.018 structural
   parse succeeded).
2. The function-group nibble (low nibble of `data[header_len + 4]`) equals `0x03`.

## Postconditions

1. The frame is classified `S7ClassicFunction::Userdata(S7UserdataFunction::
   BlockFunctions(subfn))` where `subfn = data[header_len + 5]`.
2. If `subfn == 0x01`: the specific operation is "List Blocks."
3. If `subfn == 0x02`: the specific operation is "List Blocks of Type."
4. If `subfn == 0x03`: the specific operation is "Get Block Info."
5. If `subfn` is any other value: the operation remains classified
   `BlockFunctions(subfn)` with the raw subfunction byte preserved — group `0x03` is
   recognized, but the specific subfunction is not one of the three named operations;
   never force-fit to one of the three.

## Invariants

1. **Group `0x03` == Block functions, definitively** — this is the corrected mapping;
   any implementation, test, or downstream BC that states group `0x07` is Block
   functions is defectively wrong per this BC and must be corrected to match.
2. **Subfunction values `0x01`/`0x02`/`0x03` are named; all others are recognized-group,
   unnamed-subfunction** — consistent with the no-force-fit principle applied
   throughout this feature.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Group `0x03`, subfn `0x01` (List Blocks) | `BlockFunctions(0x01)`, named "List Blocks" |
| EC-002 | Group `0x03`, subfn `0x05` (not one of the 3 named subfunctions) | `BlockFunctions(0x05)`, unnamed but group-recognized |
| EC-003 | (Regression guard) Group `0x07`, subfn `0x01` | Classified via BC-2.21.022 (Time functions), **never** via this BC — this is the explicit negative-evidence check for the group-0x03/0x07 correction |

## Canonical Test Vectors

| Group nibble / Subfn byte | Expected classification | Category |
|---|---|---|
| `0x03` / `0x01` | `BlockFunctions(0x01)` "List Blocks" | happy-path: T0888 candidate |
| `0x03` / `0x02` | `BlockFunctions(0x02)` "List Blocks of Type" | happy-path: T0888 candidate |
| `0x03` / `0x03` | `BlockFunctions(0x03)` "Get Block Info" | happy-path: T0888 candidate |
| `0x07` / `0x01` (regression guard, must NOT classify as Block functions) | Routed to BC-2.21.022 (Time functions) instead | regression-guard: group-0x03/0x07 correction |

## Verification Properties

(No independent VP-NNN — table-driven unit tests for the three named subfunctions plus
the group-0x07-is-not-block-functions regression guard, which is the single most
important test vector in this feature given ADR-014's explicit correction flag.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this BC IS the load-bearing group-0x03-correction capability ADR-014 Decision 5 mandates be reflected in the Userdata subfunction match arms |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 — "**Group-`0x03` block-function correction**... load-bearing for the T0888... emission call-site and MUST be reflected in the `s7comm.rs` Userdata subfunction match arms" |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0888 (Remote System Information Discovery, block-group `0x03`) — **classification surface only; emission is authored in part B2** |

## Related BCs

- BC-2.21.018 — depends on (structural parse this BC classifies on top of)
- BC-2.21.020 — composes with (group `0x04` CPU functions, the other T0888-relevant group)
- BC-2.21.022 — composes with (group `0x07` Time functions — the group this BC's subfunctions must NEVER be classified as)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7UserdataFunction::BlockFunctions(u8)` match arm, group `0x03`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 5` — "Group-`0x03` block-function correction"
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis — corrected Userdata group table

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated — covered by the totality proptest anchored to BC-2.21.023.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
