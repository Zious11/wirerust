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

# BC-2.17.009: parse_cip_request_path Class and Instance Segment Extraction

## Description

`parse_cip_request_path(path: &[u8]) -> Vec<CipPathSegment>` extracts Class and Instance
path segments from a CIP request path. Per ODVA CIP Volume 1, a path segment is at least
2 bytes: a segment type byte followed by a value byte (logical segment, 8-bit format). The
function walks the path byte-by-byte, identifying Class (segment type 0x20), Instance
(0x24), and Attribute (0x30) segments with 8-bit addressing (plus a pad byte to maintain
word alignment). For v0.11.0, only 8-bit logical segments are in scope. The function stops
iteration on any bounds violation and returns whatever segments were successfully extracted.
Used by detection BCs to identify the target CIP object class (e.g., Identity Object = 0x01).

## Preconditions

1. `path` is `CipHeader.request_path` — the raw path bytes from BC-2.17.006.
2. `path.len()` may be 0 (valid: zero-length path).

## Postconditions

1. If `path.len() == 0`, returns `vec![]`.
2. For each pair of bytes at `cursor` and `cursor+1`:
   - `segment_type = path[cursor]`.
   - Compare `segment_type` exactly (no mask applied):
     - If `segment_type == 0x20` → Class segment: `value = path[cursor+1]`;
       push `CipPathSegment::Class(value)`.
     - If `segment_type == 0x24` → Instance segment: `value = path[cursor+1]`;
       push `CipPathSegment::Instance(value)`.
     - If `segment_type == 0x30` → Attribute segment: `value = path[cursor+1]`;
       push `CipPathSegment::Attribute(value)`.
     - Other segment types: skipped; advance by 2.
   - `cursor += 2` after each segment.
   - If `cursor + 2 > path.len()` at any point: break.

   **Rationale**: exact-match (`== 0x20 / 0x24 / 0x30`) is used for v0.11.0 8-bit logical
   segments. The `& 0xE0` mask would incorrectly match 0x24 as 0x20 (Class) since
   `0x24 & 0xE0 == 0x20`. The `& 0xFC` mask (clears the 2-bit format field) is reserved for
   future use when 16-bit extended segment variants (0x21 Class, 0x25 Instance, 0x31 Attribute)
   are in scope; for v0.11.0, exact-match is correct and unambiguous.
3. Returns the segments extracted.
4. No panic for any `path` content or length.

## Invariants

1. **8-bit logical segments only (v0.11.0 scope)**: per ADR-010 Decision 8 (Deferred list),
   16-bit extended segment addressing (`0x21`, `0x25`, `0x31`) and Electronic Key segments
   are deferred. The function silently skips unrecognized segment types, advancing by 2 bytes.
2. **Word alignment**: CIP segments are 2-byte aligned (segment type + 8-bit value, or
   segment type + padding + 16-bit value for extended). For 8-bit segments, each is exactly
   2 bytes. This allows simple `cursor += 2` advancement.
3. **Bounds safety**: `cursor + 2 > path.len()` check prevents any out-of-bounds access.
4. **Identity Object detection**: `CipPathSegment::Class(0x01)` indicates the request targets
   the CIP Identity Object — the primary T0888 trigger (BC-2.17.014).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty path | Returns `vec![]` |
| EC-002 | `path = [0x20, 0x01]` (Class 0x01 = Identity) | Returns `vec![Class(0x01)]` |
| EC-003 | `path = [0x20, 0x01, 0x24, 0x01, 0x30, 0x07]` (Identity Class 1, Instance 1, Attr 7) | Returns `vec![Class(0x01), Instance(0x01), Attribute(0x07)]` |
| EC-004 | Path has 1 byte only | `cursor+2 > 1`: breaks immediately; returns `vec![]` |
| EC-005 | Unrecognized segment type 0x40 | Skip 2 bytes; do not push segment; continue |
| EC-006 | Path has exactly 4 bytes but declares 3 segments | First segment (bytes 0–1) parsed OK; second segment (bytes 2–3) parsed OK; at cursor=4, `cursor+2 > 4` — break. Returns 2 segments (or fewer if a byte is an unrecognized type that was still advanced past). |

## Canonical Test Vectors

| path (hex) | Expected `Vec<CipPathSegment>` | Identity target? |
|-----------|-------------------------------|-----------------|
| `[]` | `vec![]` | No |
| `20 01` | `[Class(0x01)]` | Yes — Identity Object |
| `20 01 24 01` | `[Class(0x01), Instance(0x01)]` | Yes |
| `20 01 24 01 30 07` | `[Class(0x01), Instance(0x01), Attribute(0x07)]` | Yes |
| `20 02 24 01` | `[Class(0x02), Instance(0x01)]` | No (Class 0x02 = Message Router) |
| `40 00 20 01` | `[Class(0x01)]` (0x40 unrecognized, skipped; then 0x20=Class 0x01) | Yes |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Bounds-safe segment walk, segment type recognition: unit test | unit test |

Note: `parse_cip_request_path` is an additional pure-core function in ADR-010 §4.3; not a VP-032
Kani target in v0.11.0.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — request path segment extraction identifies the target CIP object class (Identity = 0x01) for T0888 recon detection and the target object class for T0836 write-attribute detections |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 8 (v0.11.0 scope: Class+Instance+Attribute 8-bit only) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — path parse function; no finding emission) |

## Related BCs

- BC-2.17.006 — depends on (request_path comes from CipHeader.request_path)
- BC-2.17.014 — composes with (Identity Object class=0x01 in path → T0888 trigger)
- BC-2.17.012 — composes with (path segments identify write target object)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn parse_cip_request_path(path: &[u8]) -> Vec<CipPathSegment>` — pure-core free function
- `src/analyzer/enip.rs` — `enum CipPathSegment { Class(u8), Instance(u8), Attribute(u8) }`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 8` — MVP CIP object-model scope (Class+Instance+Attribute 8-bit)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — additional pure-core; not in VP-032 Kani scope v0.11.0)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 8; ODVA CIP Specification Vol 1 §C-1.3 (path segment encoding: 8-bit logical) |
| **Confidence** | high — segment type byte encoding is normative ODVA CIP |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same path bytes always produce same segments |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core (additional; not VP-032 Kani target in v0.11.0) |
