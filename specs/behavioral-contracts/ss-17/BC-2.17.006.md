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

# BC-2.17.006: parse_cip_header Extracts Service Code and Request Path from Item Data

## Description

`parse_cip_header(item_data: &[u8]) -> Option<CipHeader>` extracts the CIP service byte and
request path from a CPF item's data field (for `type_id` 0x00B1 or 0x00B2). The CIP message
begins at byte 0 of `item_data`: byte 0 is the service code (raw `u8`), byte 1 is the
request_path_size (in 16-bit words), and bytes 2 through `2 + (request_path_size * 2)` are
the request path. Returns `None` if `item_data.len() < 2` or if the path exceeds the
available data. The CIP service byte's high bit (0x80) indicates a response; the low 7 bits
identify the service. This function does not classify the service — classification is
`classify_cip_service` (BC-2.17.007).

## Preconditions

1. `item_data` is the `data` field of a `CpfItem` with `type_id == 0x00B1` or `0x00B2`.
2. `item_data.len() >= 0` — zero-length data is a valid CpfItem; results in `None`.

## Postconditions

1. If `item_data.len() < 2`, returns `None` (cannot read service byte + path size).
2. `service = item_data[0]` — raw service byte; high bit 0x80 = response.
3. `request_path_size = item_data[1] as usize` — path size in 16-bit words.
4. `path_byte_count = request_path_size * 2`.
5. If `item_data.len() < 2 + path_byte_count`, returns `None` (path truncated).
6. `request_path = item_data[2 .. 2 + path_byte_count]` — slice (not copied; lifetime of item_data).
7. Returns `Some(CipHeader { service, request_path })`.
8. The function is pure: no I/O, no state mutation, no panics.

## Invariants

1. **Service byte semantics**: the CIP service byte encodes both direction and service ID.
   High bit 0x80 set → response (reply from target); high bit clear → request. The low 7
   bits are the service code. [SPEC: ODVA CIP Spec Vol 1 §2-4.1; ADR-010 Decision 2]
2. **Path size in words**: `request_path_size` is in 16-bit words; byte count is `* 2`.
   A path size of `0` means no path (CipHeader.request_path is an empty slice).
3. **Bounds-safe path extraction**: `item_data.len() < 2 + path_byte_count` returns `None`
   rather than panicking. An attacker cannot cause an out-of-bounds access via a large
   request_path_size byte.
4. **Purity**: `parse_cip_header` is a pure-core function (not a VP-032 Kani target in v0.11.0
   but satisfies the same purity invariants). ADR-010 Decision 2 lists it as an additional
   pure-core function.
5. **caller responsibility**: the caller (`process_pdu`) is responsible for checking
   `type_id == 0x00B1 || type_id == 0x00B2` before calling this function.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `item_data.len() == 0` | Returns `None` |
| EC-002 | `item_data.len() == 1` | Returns `None` (cannot read path_size) |
| EC-003 | `item_data = [0x10, 0x00]` (SetAttributeSingle, path_size=0) | Returns `Some(CipHeader{service:0x10, request_path:[]})` — zero-length path |
| EC-004 | `item_data = [0x10, 0x03, <5 path bytes>]` (path_size=3 → 6 bytes needed, only 5 present) | Returns `None` (path truncated) |
| EC-005 | `item_data[0] = 0x90` (high bit set — response to SetAttributeSingle) | Returns `Some(CipHeader{service:0x90, request_path:[...]})` — response; caller's `classify_cip_service` maps to Response |
| EC-006 | `item_data = [0x0E, 0x03, 0x20, 0x01, 0x24, 0x01, 0x30, 0x03]` (GetAttributeSingle, path: class 1, instance 1, attr 3) | Returns `Some(CipHeader{service:0x0E, request_path:[0x20,0x01,0x24,0x01,0x30,0x03]})` |
| EC-007 | `request_path_size = 0xFF` (255 words = 510 bytes of path) with insufficient data | Returns `None` — bounds check fails |

## Canonical Test Vectors

**GetAttributeSingle to Identity Object Class=0x01, Instance=1, Attr=7:**
```
item_data (hex): 0E 03 20 01 24 01 30 07
byte[0]: service = 0x0E (GetAttributeSingle, high bit clear = request)
byte[1]: request_path_size = 0x03 (3 words = 6 bytes of path)
bytes[2..8]: path = [0x20, 0x01, 0x24, 0x01, 0x30, 0x07] (Class=1, Instance=1, Attr=7)
```
Expected: `Some(CipHeader{service: 0x0E, request_path: [0x20,0x01,0x24,0x01,0x30,0x07]})`

**Response to GetAttributeSingle (high bit set):**
```
item_data (hex): 8E 00 ...
byte[0]: service = 0x8E (response bit set; 0x8E & 0x7F = 0x0E → GetAttributeSingle response)
```
Expected: `Some(CipHeader{service: 0x8E, request_path: []})` (path_size=0)

| item_data | Expected result | Notes |
|-----------|----------------|-------|
| `[]` | `None` | empty |
| `[0x10, 0x00]` | `Some{service:0x10, path:[]}` | SetAttributeSingle, no path |
| `[0x90, 0x00]` | `Some{service:0x90, path:[]}` | response, no path |
| `[0x0E, 0x03, <6 bytes path>]` | `Some{service:0x0E, path:[6 bytes]}` | GetAttributeSingle |
| `[0x0E, 0x03, <5 bytes only>]` | `None` | truncated path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Bounds-safe extraction, path-size arithmetic, high-bit semantics: unit test | unit test |

Note: `parse_cip_header` is an additional pure-core function listed in ADR-010 §4.3 but
not a VP-032 Kani target in v0.11.0. The four Kani targets are Sub-A/B/C/D (BCs 001/004/003/007).

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — CIP header extraction is the foundation for all CIP-level MITRE detections (T0858/T0816/T0836/T0888/T0846/ForwardOpen); without a safe, bounds-checked CIP header parse, no CIP service detection is possible |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — CIP parse function; no finding emission directly) |

## Related BCs

- BC-2.17.005 — depends on (CipItem.data is the input to this function)
- BC-2.17.007 — composes with (classify_cip_service operates on CipHeader.service from this function)
- BC-2.17.009 — composes with (parse_cip_request_path operates on CipHeader.request_path)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn parse_cip_header(item_data: &[u8]) -> Option<CipHeader>` — pure-core free function
- `src/analyzer/enip.rs` — `struct CipHeader { service: u8, request_path: Vec<u8> }`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 2` — CIP header parse pseudocode

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — additional pure-core function; not in VP-032 Kani scope for v0.11.0)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 2 (parse_cip_header pseudocode); ODVA CIP Specification Vol 1 §2-4.1 (service byte and path structure) |
| **Confidence** | high — CIP service byte semantics and path_size-in-words convention are normative ODVA |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same item_data always produces same result |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core (additional; not VP-032 Kani target in v0.11.0) |
