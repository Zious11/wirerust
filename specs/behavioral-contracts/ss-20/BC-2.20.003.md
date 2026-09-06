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

# BC-2.20.003: `parse_tpkt_header` Returns None for Length Field < 4 (Malformed, Includes Zero-Length)

## Description

The TPKT `length` field (`data[2..4]`, big-endian `u16`) is defined by RFC 1006 §5 as
"length of entire packet in octets, including packet header." Since the header itself
is 4 bytes, no valid TPKT packet can declare a length smaller than 4 — a length of `0`,
`1`, `2`, or `3` is structurally impossible and is rejected as malformed. This is the
zero-length / degenerate-length edge case called out in the F2 authoring scope.

## Preconditions

1. `data.len() >= 4`.
2. `data[0] == 0x03` (version check already passed — see BC-2.20.002).
3. The big-endian `u16` decoded from `data[2..4]` is `< 4`.

## Postconditions

1. `parse_tpkt_header(data)` returns `None`.
2. No panic occurs for any `u16` value of the decoded length field, including `0`.
3. The frame-walk caller treats this as a malformed-length condition: unlike the
   bad-version-byte path (BC-2.20.002/BC-2.20.015), a malformed in-range-version-but-bad-length
   frame is a genuine protocol violation on an otherwise-recognized frame start, so it
   is resynced identically (clear the direction's carry state for the current attempt,
   advance and search for the next `0x03` candidate) — see BC-2.20.014/015 for the
   shared resync/anomaly-emission machinery.

## Invariants

1. **Minimum valid length is 4** — a TPKT packet with zero payload is still exactly
   4 bytes (header only); no smaller value is ever legal.
2. **No arithmetic overflow**: decoding `data[2..4]` as `u16::from_be_bytes` cannot
   overflow (fixed 2-byte read); the subsequent `< 4` comparison is total over all
   `u16` values.
3. **Independent of version check**: this function evaluates the length-field bound
   only after the version byte (BC-2.20.002) has already been confirmed `0x03`; the two
   checks are independent preconditions that must both pass for BC-2.20.004's accept path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `length == 0` (zero-length) | Returns `None` — cannot be smaller than the header itself |
| EC-002 | `length == 1` | Returns `None` |
| EC-003 | `length == 3` (one less than minimum) | Returns `None` |
| EC-004 | `length == 4` (exactly minimum — header-only packet, no COTP payload) | Proceeds to accept path; see BC-2.20.004 |
| EC-005 | `length` bytes are `[0x00, 0x00]` (all-zero length field, most degenerate case) | Returns `None` |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x03, 0x00, 0x00, 0x00]` (length=0) | `None` | reject: zero-length |
| `[0x03, 0x00, 0x00, 0x01]` (length=1) | `None` | reject: below minimum |
| `[0x03, 0x00, 0x00, 0x03]` (length=3) | `None` | reject: below minimum |
| `[0x03, 0x00, 0x00, 0x04]` (length=4) | `Some(TpktHeader{version:3, length:4})` | accept: minimum valid — see BC-2.20.004 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all decoded `length < 4`, `parse_tpkt_header` returns `None`; no panic or overflow for any `u16` length value | Kani P0 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the malformed-length rejection path, the structural minimum-length invariant for every TPKT packet |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decision 1 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none directly — malformed-length frames are silently rejected by this pure function; anomaly emission, if any, is a frame-walk-loop concern per BC-2.20.014) |

## Related BCs

- BC-2.20.001 — composes with (length-reject-by-input-size path, evaluated before this check)
- BC-2.20.002 — composes with (version-reject path, evaluated before this check)
- BC-2.20.004 — composes with (accept path: length in `[4, 65535]`)
- BC-2.20.014 — depends on (resync/dedup machinery for malformed frames mid-walk)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_tpkt_header`: `let length = u16::from_be_bytes([data[2], data[3]]); if length < 4 { return None }`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — TPKT length field: "total TPKT packet length including this 4-byte header"

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
