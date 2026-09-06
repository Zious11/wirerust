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

# BC-2.21.015: PLC Control (FC 0x28) Classified With PI-Service String Decode — `P_PROGRAM`/`_INSE`/`_DELE`/`_GARB`/`_MODU`

## Description

`FC == 0x28` (PLC Control / PI-Service) is a multiplexed function whose actual
operation is carried as an ASCII service-name string within the parameter block, not
by the FC byte alone. Per ADR-014's flagged ambiguity ("`0x28` PI-Service ambiguity...
the analyzer MUST decode the service name before mapping"), this BC requires the
service string to be decoded before any `S7ClassicFunction::PlcControl` variant more
specific than "PI-Service, service undetermined" can be assigned. Five service strings
are recognized: `P_PROGRAM` (start/run/state control), `_INSE` (activate/insert a
block), `_DELE` (delete a block), `_GARB` (memory compress), `_MODU` (RAM→ROM module
transfer). Any other ASCII content, or an unreadable/truncated service-string field, is
classified `PlcControlService::Unrecognized` — never force-fit to one of the five known
services.

## Preconditions

1. `header.rosctr ∈ {Rosctr::Job, Rosctr::AckData}`.
2. The parameter block is bounds-validated per BC-2.21.009 and `param_length >= 1`.
3. `data[header_len] == 0x28`.

## Postconditions

1. The frame is classified `S7ClassicFunction::PlcControl(service)`.
2. If the parameter block contains a length-prefixed ASCII string field matching
   exactly one of `"P_PROGRAM"`, `"_INSE"`, `"_DELE"`, `"_GARB"`, `"_MODU"` (byte-exact
   comparison, no case-folding — S7comm service strings are fixed-case per the
   observed wire format), `service` is set to the corresponding
   `PlcControlService` variant (`ProgramStart`, `BlockActivate`, `BlockDelete`,
   `MemoryCompress`, `RamToRom`).
3. If the service-string field cannot be read (insufficient parameter-block bytes) or
   its content does not byte-exactly match any of the five known strings, `service` is
   `PlcControlService::Unrecognized` — the frame remains classified `PlcControl`, with
   the specific service left undetermined; this is never a hard reject of the whole
   frame.
4. Bare `FC == 0x28` classification alone (without service-string decode) is never
   sufficient evidence for a downstream B2 technique tag — Postcondition 2/3
   collectively enforce that `PlcControlService` is always populated (with
   `Unrecognized` as the honest fallback) before B2 can act on it.

## Invariants

1. **No bare-0x28 force-fit**: this BC exists specifically to prevent the error ADR-014
   flags — treating any `0x28` frame as, say, "device restart" without checking the
   service string. `PlcControlService::Unrecognized` is the always-available safe
   fallback.
2. **Byte-exact string comparison**: the five service strings are compared verbatim
   against the wire bytes; no normalization, trimming, or fuzzy matching is performed.
3. **`P_PROGRAM` ambiguity is inherited, not resolved, by this BC**: `P_PROGRAM` alone
   does not distinguish start/stop/state-query sub-operations (those require decoding
   additional PI-service parameter bytes beyond the service-name string); B2's T0858
   (Change Operating Mode) emission call-site is responsible for any further
   sub-operation disambiguation it needs — this BC guarantees only that the service
   name itself is correctly extracted.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Service string exactly `"P_PROGRAM"` | `PlcControlService::ProgramStart` |
| EC-002 | Service string exactly `"_INSE"` | `PlcControlService::BlockActivate` |
| EC-003 | Service string exactly `"_DELE"` | `PlcControlService::BlockDelete` |
| EC-004 | Service string exactly `"_GARB"` | `PlcControlService::MemoryCompress` |
| EC-005 | Service string exactly `"_MODU"` | `PlcControlService::RamToRom` |
| EC-006 | Service string is a truncated fragment, e.g. `"_INS"` (missing final byte due to a short parameter block) | `PlcControlService::Unrecognized` — no partial-prefix matching |
| EC-007 | Service string is a case-variant, e.g. `"p_program"` | `PlcControlService::Unrecognized` — byte-exact comparison per Invariant 2, no case-folding |
| EC-008 | `param_length` too short to contain any service-string field at all | `PlcControlService::Unrecognized` |

## Canonical Test Vectors

| Service-string bytes | Expected `PlcControlService` | Category |
|---|---|---|
| `"P_PROGRAM"` | `ProgramStart` | happy-path: T0858/T0816 candidate |
| `"_INSE"` | `BlockActivate` | happy-path: T0889 candidate |
| `"_DELE"` | `BlockDelete` | happy-path: T0889 candidate |
| `"_XXXX"` (unrecognized) | `Unrecognized` | edge-case: no force-fit |

## Verification Properties

(No independent VP-NNN — table-driven unit tests for the five-string exact-match
decode plus the unrecognized fallback; proptest P1 candidate for byte-string
fuzz-matching totality, mirroring the area-code totality treatment in BC-2.21.012.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — resolves the ADR-014-flagged `0x28` PI-Service ambiguity that is load-bearing for correct downstream MITRE mapping |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 ("`0x28` PI-Service ambiguity... MUST decode the service name") |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0858 (Change Operating Mode, via `ProgramStart`), T0816 (Device Restart/Shutdown, via a decoded restart operation — **not yet a distinct `PlcControlService` variant in this BC; B2 must further decode `ProgramStart`'s sub-operation if restart-specific evidence is required, per Invariant 3**), T0889 (Modify Program, via `BlockActivate`/`BlockDelete`) — **classification surface only; emission is authored in part B2** |

## Related BCs

- BC-2.21.013 — composes with (an `_INSE`/`_DELE` block-activate/delete may follow a download sequence; B2 correlates the two for T0889)
- BC-2.21.016 — composes with (PLC Stop, the FC `0x29` sibling operating-mode-change function)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7ClassicFunction::PlcControl(PlcControlService)` match arm, service-string decode helper
- `enum PlcControlService { ProgramStart, BlockActivate, BlockDelete, MemoryCompress, RamToRom, Unrecognized }` (planned, this BC's design)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 5` — "`0x28` PI-Service ambiguity" flag
- `.factory/research/s7comm-mitre-ics-tagging.md` §Flagged / unverifiable, item 4 — service-string decode requirement

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — proptest P1 for service-string matching totality.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
