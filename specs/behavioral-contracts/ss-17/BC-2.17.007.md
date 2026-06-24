---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.007: classify_cip_service Total Classification with Response-Bit Mask — 13 Named Request Services + Response + Unknown = 16 Variants

## Description

`classify_cip_service(service: u8) -> CipServiceClass` is a pure-core classification function
that maps all 256 possible `u8` CIP service byte values to a named `CipServiceClass` variant.
The function is total and never panics. The response-bit invariant: when `service & 0x80 != 0`
(high bit set), the function returns `CipServiceClass::Response` regardless of the lower 7 bits.
For request service codes (high bit clear), 13 named service codes map to named variants; all
other request values map to `CipServiceClass::Unknown`. The `Unknown` arm is reachable and
proven non-vacuous. Formally verified by VP-032 Sub-D.

## Preconditions

1. `service` is a `u8` — all 256 values are valid inputs with defined behavior.
2. `service` is `CipHeader.service` from `parse_cip_header` (BC-2.17.006).

## Postconditions

1. `classify_cip_service(service)` returns a valid `CipServiceClass` variant for every input.
2. **Response-bit invariant**: if `service & 0x80 != 0` → returns `CipServiceClass::Response`.
   This applies to all 128 values in the range 0x80–0xFF.
3. For `service & 0x80 == 0` (values 0x00–0x7F), the 13 named service codes map to:
   - `0x01` → `CipServiceClass::GetAttributesAll`
   - `0x02` → `CipServiceClass::SetAttributesAll`
   - `0x03` → `CipServiceClass::GetAttributeList`
   - `0x04` → `CipServiceClass::SetAttributeList`
   - `0x05` → `CipServiceClass::Reset`
   - `0x07` → `CipServiceClass::Stop`
   - `0x09` → `CipServiceClass::MultipleServicePacket`
   - `0x0A` → `CipServiceClass::ApplyAttributes`
   - `0x0E` → `CipServiceClass::GetAttributeSingle`
   - `0x10` → `CipServiceClass::SetAttributeSingle`
   - `0x4B` → `CipServiceClass::GetAndClear` (firmware download marker)
   - `0x4E` → `CipServiceClass::ForwardClose`
   - `0x54` → `CipServiceClass::ForwardOpen`
   - `0x5B` → `CipServiceClass::LargeForwardOpen`
4. All other request values (high bit clear, not in named set) → `CipServiceClass::Unknown`.
5. `Unknown` is reachable (e.g., `service = 0x7F`).
6. The function is pure: no I/O, no state mutation, terminates for all inputs.

## Invariants

1. **Response-bit mask takes priority**: the `service & 0x80 != 0` check is applied first,
   before any named-service lookup. This is the correct CIP interpretation: a response is
   always identifiable by the high bit, regardless of service code.
2. **Named service count**: 13 named request services + Response + Unknown = 16 total variants
   in the enum. VP-032 Sub-D proves totality.
3. **Stop (0x07) and Reset (0x05)**: critical MITRE trigger services. 0x07 (Stop) → T0858;
   0x05 (Reset) → T0816. These must map to their named variants without ambiguity.
4. **SetAttribute variants (0x02, 0x04, 0x10)**: SetAttributesAll, SetAttributeList,
   SetAttributeSingle — all trigger the write-burst T0836 detection path (BC-2.17.012).
5. **ForwardOpen (0x54) and LargeForwardOpen (0x5B)**: connection-lifecycle services detected
   by BC-2.17.015.
6. **Firmware service (0x4B GetAndClear)**: seeded for staged T1693.001 detection (not emitted
   in v0.11.0; see ADR-010 Decision 8 Deferred list).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `service = 0x07` (Stop) | Returns `CipServiceClass::Stop` — T0858 trigger |
| EC-002 | `service = 0x05` (Reset) | Returns `CipServiceClass::Reset` — T0816 trigger |
| EC-003 | `service = 0x10` (SetAttributeSingle) | Returns `CipServiceClass::SetAttributeSingle` — T0836 write trigger |
| EC-004 | `service = 0x0E` (GetAttributeSingle) | Returns `CipServiceClass::GetAttributeSingle` — identity read T0888 trigger |
| EC-005 | `service = 0x80` (high bit set) | Returns `CipServiceClass::Response` — response mask |
| EC-006 | `service = 0xFF` (all bits set) | Returns `CipServiceClass::Response` — high bit set |
| EC-007 | `service = 0x7F` (high bit clear, not named) | Returns `CipServiceClass::Unknown` — reachable Unknown arm |
| EC-008 | `service = 0x00` (high bit clear, not named) | Returns `CipServiceClass::Unknown` |
| EC-009 | `service = 0x54` (ForwardOpen) | Returns `CipServiceClass::ForwardOpen` — connection lifecycle |
| EC-010 | `service = 0x5B` (LargeForwardOpen) | Returns `CipServiceClass::LargeForwardOpen` |
| EC-011 | `service = 0x4E` (ForwardClose) | Returns `CipServiceClass::ForwardClose` |
| EC-012 | `service = 0x4B` (GetAndClear / firmware) | Returns `CipServiceClass::GetAndClear` — staged, not emitted in v0.11.0 |

## Canonical Test Vectors

| `service` (hex) | Name | Expected `CipServiceClass` | MITRE relevance |
|-----------------|------|---------------------------|----------------|
| `0x01` | GetAttributesAll | `GetAttributesAll` | identity read T0888 |
| `0x02` | SetAttributesAll | `SetAttributesAll` | write T0836 |
| `0x04` | SetAttributeList | `SetAttributeList` | write T0836 |
| `0x05` | Reset | `Reset` | T0816 |
| `0x07` | Stop | `Stop` | T0858 |
| `0x0E` | GetAttributeSingle | `GetAttributeSingle` | identity read T0888 |
| `0x10` | SetAttributeSingle | `SetAttributeSingle` | write T0836 |
| `0x4B` | GetAndClear | `GetAndClear` | staged T1693.001 (not emitted v0.11.0) |
| `0x4E` | ForwardClose | `ForwardClose` | connection lifecycle |
| `0x54` | ForwardOpen | `ForwardOpen` | connection lifecycle |
| `0x5B` | LargeForwardOpen | `LargeForwardOpen` | connection lifecycle |
| `0x80` | (response) | `Response` | response mask |
| `0xFF` | (response) | `Response` | all-bits-set, response |
| `0x7F` | (none) | `Unknown` | Unknown non-vacuity |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-D: `classify_cip_service` is total over all 256 `u8` values; `service & 0x80 != 0` → `Response` (proven for all 128 values 0x80–0xFF); `Unknown` arm reachable via `service=0x7F` | Kani: symbolic `u8`; asserts response-bit invariant; companion: `matches!(classify_cip_service(0x7F), CipServiceClass::Unknown)` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — CIP service classification is the core dispatch mechanism for all CIP-level MITRE detections; the response-bit invariant is required by the CIP specification; totality ensures no u8 service value causes a panic |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — classification function; no finding emission) |

## Related BCs

- BC-2.17.006 — depends on (service byte comes from CipHeader.service)
- BC-2.17.011 — depends on (Stop → T0858 detection path; BC-2.17.011 precondition requires classify returns Stop)
- BC-2.17.012 — depends on (SetAttribute* → T0836 write-burst; BC-2.17.012 precondition requires classify returns SetAttribute*)
- BC-2.17.013 — depends on (Reset → T0816 detection path)
- BC-2.17.014 — depends on (GetAttribute*/GetAttributeSingle → T0888 identity read)
- BC-2.17.015 — depends on (ForwardOpen/LargeForwardOpen → connection lifecycle)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn classify_cip_service(service: u8) -> CipServiceClass` — pure-core free function
- `src/analyzer/enip.rs` — `enum CipServiceClass { GetAttributesAll, SetAttributesAll, GetAttributeList, SetAttributeList, Reset, Stop, MultipleServicePacket, ApplyAttributes, GetAttributeSingle, SetAttributeSingle, GetAndClear, ForwardClose, ForwardOpen, LargeForwardOpen, Response, Unknown }`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 2` — CIP service extraction; §Decision 8 — MVP CIP object-model scope
- `.factory/specs/verification-properties/vp-032-enip-parse-safety.md §Sub-D` — Kani proof skeleton

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-D — CIP service classification totality (all 256 u8 values; response-bit mask; Unknown non-vacuity)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 2 (classify_cip_service); ADR-010 Decision 8 (MVP service scope); VP-032 Sub-D skeleton; ODVA CIP Specification Vol 1 §2-4.1 (service byte / response bit) |
| **Confidence** | high — response-bit convention and service codes are normative ODVA; VP-032 Sub-D Kani proves totality and response-bit invariant for all 256 inputs |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same u8 always produces same CipServiceClass |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-032 Sub-D Kani target |
