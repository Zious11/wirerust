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

# BC-2.21.022: Userdata Time Functions (Group 0x07) Classified `TimeFunctions(subfn)` — Corrected Group Meaning, NOT Block Functions

## Description

**This BC is the direct negative-space counterpart to BC-2.21.019's load-bearing
correction.** Userdata function group `0x07` is **Time functions** (clock read/set) —
**not** Block functions, contrary to a documentation error some secondary sources make.
Group `0x07` subfunctions are classified generically as
`S7UserdataFunction::TimeFunctions(subfn)`; the source research does not enumerate
specific named time-function subfunction values (e.g. distinguishing "read clock" from
"set clock") with the same confidence as group `0x03`'s three named block
subfunctions, so this BC does not assert named subfunction operations beyond the group
level, avoiding an unverified claim.

## Preconditions

1. `header.rosctr == Rosctr::Userdata`, `param_length >= 7` (BC-2.21.018 passed).
2. The function-group nibble equals `0x07`.

## Postconditions

1. The frame is classified `S7ClassicFunction::Userdata(S7UserdataFunction::
   TimeFunctions(subfn))` where `subfn = data[header_len + 5]`.
2. This classification is **never** conflated with, aliased to, or co-classified as
   `BlockFunctions` (BC-2.21.019) — group `0x07` and group `0x03` are structurally and
   semantically disjoint despite the historical documentation-error risk this BC
   exists specifically to guard against.

## Invariants

1. **Group `0x07` == Time functions, definitively** — the corrected mapping, mirrored
   from BC-2.21.019's corrected group `0x03` == Block functions. These two BCs are a
   matched pair: correcting one without the other would leave the feature
   half-corrected and still capable of the documentation-error-class defect ADR-014
   flags.
2. **No named subfunction claims beyond group level**: unlike BC-2.21.019's three named
   Block-function subfunctions, this BC intentionally does not assert specific
   Time-function subfunction names, since the source research does not establish them
   with equivalent confidence — honesty about the boundary of verified knowledge takes
   precedence over completeness.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Group `0x07`, any subfn value | `TimeFunctions(subfn)`, group-level classification only |
| EC-002 | (Regression guard) Group `0x03` frame is ever mis-routed to this BC's match arm | Must not occur — this is the explicit negative test BC-2.21.019's EC-003 also names, verified from both directions |

## Canonical Test Vectors

| Group nibble / Subfn byte | Expected classification | Category |
|---|---|---|
| `0x07` / `0x01` | `TimeFunctions(0x01)` | happy-path |
| `0x03` / `0x01` (regression guard, must NOT classify as Time functions) | `BlockFunctions(0x01)` (BC-2.21.019), never `TimeFunctions` | regression-guard: group-0x03/0x07 correction, verified bidirectionally |

## Verification Properties

(No independent VP-NNN — table-driven unit test plus the bidirectional group-0x03/0x07
regression guard shared with BC-2.21.019; totality covered by the shared match anchored
to BC-2.21.023.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — completes the load-bearing group-0x03/0x07 correction alongside BC-2.21.019 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 — "group `0x07` = **Time functions** (clock read/set) — the reverse of a common documentation error" |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — no seeded technique maps to Time functions per the source research) |

## Related BCs

- BC-2.21.019 — composes with (the matched-pair Block-functions correction; both BCs must be verified together)
- BC-2.21.018 — depends on (structural parse)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7UserdataFunction::TimeFunctions(u8)` match arm, group `0x07`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 5` — "group `0x07` = **Time functions**"
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis — corrected Userdata group table

## Story Anchor

STORY-189 (also a formal-hardening re-verification anchor for STORY-194)

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
