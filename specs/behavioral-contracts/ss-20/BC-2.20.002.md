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
subsystem: SS-20
capability: CAP-20
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

# BC-2.20.002: `parse_tpkt_header` Returns None for Version Byte ≠ 0x03

## Description

RFC 1006 §5 fixes the TPKT version byte (offset 0) at `0x03`. When `data.len() >= 4`
but `data[0] != 0x03`, `parse_tpkt_header` returns `None` without decoding the length
field. This non-`0x03` version byte is also the resync candidate the frame-walk loop
searches for after a bound-trip (BC-2.20.015) — it is the ISO-on-TCP analogue of
IEC-104's `0x68` start-byte resync anchor.

## Preconditions

1. `data.len() >= 4`.
2. `data[0] != 0x03` (non-canonical TPKT version byte).

## Postconditions

1. `parse_tpkt_header(data)` returns `None`.
2. The length field (`data[2..4]`) is never decoded when the version byte is invalid —
   no partial decode.
3. The function is pure: no I/O, no global state mutation, no panic for any `u8` value
   of `data[0]`.
4. The frame-walk caller treats a non-`0x03` version byte as a bad-start-byte condition:
   advance exactly 1 byte and retry (resync semantics, BC-2.20.015) — never a permanent
   desync latch.

## Invariants

1. **Fixed version constant**: `0x03` is not configurable; it is the sole valid TPKT
   version per RFC 1006.
2. **Purity**: no state mutation; deterministic for any `u8` input at offset 0.
3. **Reserved-byte independence**: this check depends only on `data[0]`; the reserved
   byte at `data[1]` is never inspected by this function (it is not validated at all —
   see BC-2.20.004 Invariant 3).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data[0] == 0x00` | Returns `None` |
| EC-002 | `data[0] == 0x04` (off-by-one from valid version) | Returns `None` — no leniency |
| EC-003 | `data[0] == 0xFF` | Returns `None` |
| EC-004 | `data[0] == 0x03` (valid) | Proceeds to length-field validation; see BC-2.20.003/004 |
| EC-005 | A byte stream containing a spurious `0x03` at offset 1 (not offset 0) while `data[0] != 0x03` | Not treated as the frame start by this call; only the resync walk (BC-2.20.015) advances byte-by-byte to find it |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x00, 0x00, 0x00, 0x04]` | `None` | reject: version = 0x00 |
| `[0x04, 0x00, 0x00, 0x04]` | `None` | reject: version = 0x04 (off-by-one) |
| `[0xFF, 0x00, 0x00, 0x04]` | `None` | reject: version = 0xFF |
| `[0x03, 0x00, 0x00, 0x04]` | proceeds to length check | accept-boundary — see BC-2.20.003/004 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all `data[0] != 0x03`, `parse_tpkt_header` returns `None`; no panic for any `u8` value | Kani P0 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |
| Resync loop advances exactly 1 byte per bad-version-byte iteration (never 2), guaranteeing no real `0x03` frame start is skipped | proptest P1 (per ADR-014 Decision 8/9) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the non-canonical-version rejection path, which is also the resync anchor for carry-overflow recovery |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decisions 1, 8 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none directly — but see BC-2.20.015 for the resync-driven T0814 carry-overflow emission) |

## Related BCs

- BC-2.20.001 — composes with (length-reject path, evaluated before this check)
- BC-2.20.003 — composes with (length field < 4 rejection path, evaluated after version passes)
- BC-2.20.004 — composes with (accept path)
- BC-2.20.015 — depends on (resync anchor semantics reuse this exact version-byte check)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_tpkt_header`: `if data[0] != 0x03 { return None }`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 8` — "the TPKT `version` byte (always `0x03` for a valid TPKT packet) is the resync candidate on a bad-start-byte condition"

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst per ADR-014 Decision 9,
anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — Kani P0 target |
