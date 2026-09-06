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

# BC-2.21.020: Userdata CPU Functions (Group 0x04) Subfunction 0x01 Classified as Read SZL

## Description

Userdata function group `0x04` is **CPU functions**. Subfunction `0x01` is **Read SZL**
(System Status List) — the primary passively-observable discovery/reconnaissance
operation in the Userdata space, and the second of the two group-based T0888 (Remote
System Information Discovery) detection predicates alongside BC-2.21.019's Block
functions. This BC isolates the `group == 0x04, subfn == 0x01` combination as its own
named classification, distinct from other group-`0x04` subfunctions (diagnostics,
alarms — BC-2.21.021), because Read SZL is the specific, well-evidenced discovery
signal the MITRE research validates.

## Preconditions

1. `header.rosctr == Rosctr::Userdata`, `param_length >= 7` (BC-2.21.018 passed).
2. The function-group nibble equals `0x04`.
3. `subfn == 0x01` where `subfn = data[header_len + 5]`.

## Postconditions

1. The frame is classified `S7ClassicFunction::Userdata(S7UserdataFunction::
   CpuReadSzl)`.
2. No SZL-ID-specific decoding (which SZL partial-list is being requested) is
   performed by this BC — the classification is at the subfunction level only;
   SZL-ID-specific granularity is out of B1 scope.

## Invariants

1. **Group `0x04` == CPU functions, subfunction `0x01` == Read SZL** — per ADR-014
   Decision 5's corrected table, distinct from and unrelated to group `0x03`'s Block
   functions or group `0x07`'s Time functions.
2. **Read SZL is a distinct named variant, not folded into a generic `CpuOther`** —
   because it is the single group-`0x04` subfunction with an established MITRE mapping
   (T0888), it is elevated to its own classification rather than sharing
   BC-2.21.021's catch-all.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Group `0x04`, subfn `0x01` | `CpuReadSzl` |
| EC-002 | Group `0x04`, subfn `0x02` (a different CPU function, e.g. diagnostics) | Classified via BC-2.21.021 (`CpuOther`), not this BC |
| EC-003 | (Regression guard) Group `0x03`, subfn `0x01` (List Blocks — structurally similar subfunction number, different group) | Classified via BC-2.21.019, **never** confused with `CpuReadSzl` — group nibble, not subfunction number alone, gates this classification |

## Canonical Test Vectors

| Group nibble / Subfn byte | Expected classification | Category |
|---|---|---|
| `0x04` / `0x01` | `CpuReadSzl` | happy-path: T0888 candidate |
| `0x04` / `0x02` | `CpuOther(0x02)` (BC-2.21.021) | edge-case: same group, different subfunction |
| `0x03` / `0x01` (regression guard) | `BlockFunctions(0x01)` (BC-2.21.019), never `CpuReadSzl` | regression-guard: group discrimination, not subfn-number coincidence |

## Verification Properties

(No independent VP-NNN — table-driven unit test; totality covered by the shared match
anchored to BC-2.21.023.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the second of the two T0888-relevant classification surfaces per ADR-014 Decision 5 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 — "CPU group `0x04` subfn `0x01`" |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0888 (Remote System Information Discovery, CPU group `0x04` subfn `0x01` Read SZL) — **classification surface only; emission is authored in part B2** |

## Related BCs

- BC-2.21.018 — depends on (structural parse)
- BC-2.21.019 — composes with (the sibling T0888-relevant group `0x03`)
- BC-2.21.021 — composes with (other group-`0x04` subfunctions, catch-all)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7UserdataFunction::CpuReadSzl` match arm, group `0x04` subfn `0x01`
- `.factory/research/s7comm-mitre-ics-tagging.md` §S7comm wire-field basis — "`0x04` CPU functions... `0x01` **Read SZL**"

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
