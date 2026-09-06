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

# BC-2.21.021: Userdata CPU Functions (Group 0x04) Other Subfunctions Classified `CpuOther(subfn)` — No Force-Fit to Read SZL

## Description

Group `0x04` (CPU functions) carries subfunctions beyond `0x01` Read SZL (BC-2.21.020)
— diagnostics, alarm-related, and other CPU-service subfunctions the source research
does not individually enumerate or map to a MITRE technique. This BC classifies any
group-`0x04` subfunction other than `0x01` as `S7UserdataFunction::CpuOther(subfn)`,
preserving the raw subfunction byte without inventing a specific named operation the
research does not support.

## Preconditions

1. `header.rosctr == Rosctr::Userdata`, `param_length >= 7` (BC-2.21.018 passed).
2. The function-group nibble equals `0x04`.
3. `subfn != 0x01`.

## Postconditions

1. The frame is classified `S7ClassicFunction::Userdata(S7UserdataFunction::
   CpuOther(subfn))` where `subfn = data[header_len + 5]`.
2. No specific named operation (e.g. "diagnostics") is asserted for any `CpuOther`
   value — the classification is honest about the boundary of what the source
   research validates, avoiding an unverified semantic claim.

## Invariants

1. **Recognized-group, unenumerated-subfunction is a legitimate, first-class
   classification outcome** — not an error condition, and not the same as
   BC-2.21.023's fully-unrecognized-group catch-all.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Group `0x04`, subfn `0x02` | `CpuOther(0x02)` |
| EC-002 | Group `0x04`, subfn `0x00` | `CpuOther(0x00)` |

## Canonical Test Vectors

| Group nibble / Subfn byte | Expected classification | Category |
|---|---|---|
| `0x04` / `0x02` | `CpuOther(0x02)` | edge-case: recognized group, unenumerated subfunction |
| `0x04` / `0x01` (negative control) | `CpuReadSzl` (BC-2.21.020), never `CpuOther` | regression-guard |

## Verification Properties

(No independent VP-NNN — table-driven unit test; totality covered by the shared match
anchored to BC-2.21.023.)

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
| MITRE Techniques | (none — no seeded technique maps to unenumerated group-`0x04` subfunctions) |

## Related BCs

- BC-2.21.020 — composes with (the named `0x01` Read SZL sibling arm)
- BC-2.21.018 — depends on (structural parse)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7UserdataFunction::CpuOther(u8)` match arm, group `0x04` catch-all

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
