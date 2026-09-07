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
input-hash: "cf116b5"
---

# BC-2.20.003: `parse_tpkt_header` Returns None for Length Field < 7 (Malformed, Includes Zero-Length)

## Description

The TPKT `length` field (`data[2..4]`, big-endian `u16`) is defined by RFC 1006 §6 as
"length of entire packet in octets, including packet header." RFC 1006 §6 additionally
states the packet-length minimum as `7` — the 4-byte TPKT header plus the 3-byte minimum
COTP unit that must follow it. No valid TPKT packet can declare a length smaller than 7;
a length of `0`, `1`, `2`, `3`, `4`, `5`, or `6` is structurally impossible (either
smaller than the header itself, or too small to carry any COTP unit) and is rejected as
malformed. This is the zero-length / degenerate-length edge case called out in the F2
authoring scope.

**Clarifying note (threshold disambiguation):** do not conflate this BC's decoded-length
semantic floor (`7`) with BC-2.20.001's `data.len() < 4` **structural read-guard**. The
`4`-byte guard in BC-2.20.001 only concerns whether enough bytes are present to read the
header fields at all; it is unchanged. This BC's `7` threshold concerns the numeric value
*decoded from* the length field, evaluated only once the header has already been
successfully read and the version byte validated.

## Preconditions

1. `data.len() >= 4`.
2. `data[0] == 0x03` (version check already passed — see BC-2.20.002).
3. The big-endian `u16` decoded from `data[2..4]` is `< 7`.

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

1. **Minimum valid length is 7** — per RFC 1006 §6, a valid TPKT packet must contain the
   4-byte TPKT header plus at least a 3-byte minimum COTP unit; no smaller declared
   length is ever legal.
2. **No arithmetic overflow**: decoding `data[2..4]` as `u16::from_be_bytes` cannot
   overflow (fixed 2-byte read); the subsequent `< 7` comparison is total over all
   `u16` values.
3. **Independent of version check**: this function evaluates the length-field bound
   only after the version byte (BC-2.20.002) has already been confirmed `0x03`; the two
   checks are independent preconditions that must both pass for BC-2.20.004's accept path.

### Rationale Note: RFC 1006 §6-Conformant Minimum (`min=7`)

RFC 1006 §6 states the TPKT packet-length minimum as `7` (the 4-byte TPKT header plus the
3-byte minimum COTP unit that must follow it). `parse_tpkt_header` validates the decoded
length field against exactly this `< 7` threshold, so any declared length of `4`, `5`, or
`6` — previously accepted under a prior `>= 4` structural-only floor — is now rejected.
This threshold is RFC 1006 §6-conformant: it is no longer a deliberate cross-layer
divergence from the RFC's stated packet-level minimum, and there is no remaining gap for
the COTP layer (SS-21) to backstop at the length-floor level. (Human ruling,
2026-09-06: the previous "≥4 vs RFC-min-7 layering divergence" rationale is retired and
replaced by this note.)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `length == 0` (zero-length) | Returns `None` — cannot be smaller than the header itself |
| EC-002 | `length == 1` | Returns `None` |
| EC-003 | `length == 4` (former structural floor under the prior `>= 4` rule — now rejected under RFC 1006 §6 `min=7`) | Returns `None` |
| EC-004 | `length == 5` | Returns `None` |
| EC-005 | `length == 6` (one less than the RFC-conformant minimum) | Returns `None` |
| EC-006 | `length == 7` (exactly minimum — 4-byte TPKT header + 3-byte minimum COTP unit, RFC 1006 §6 conformant) | Proceeds to accept path; see BC-2.20.004 |
| EC-007 | `length` bytes are `[0x00, 0x00]` (all-zero length field, most degenerate case) | Returns `None` |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x03, 0x00, 0x00, 0x00]` (length=0) | `None` | reject: zero-length |
| `[0x03, 0x00, 0x00, 0x01]` (length=1) | `None` | reject: below minimum |
| `[0x03, 0x00, 0x00, 0x04]` (length=4) | `None` | reject: below RFC-conformant minimum (formerly accepted under prior `>= 4` floor) |
| `[0x03, 0x00, 0x00, 0x06]` (length=6) | `None` | reject: one below minimum |
| `[0x03, 0x00, 0x00, 0x07]` (length=7) | `Some(TpktHeader{version:3, length:7})` | accept: minimum valid (RFC 1006 §6 `min=7`) — see BC-2.20.004 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all decoded `length < 7`, `parse_tpkt_header` returns `None`; no panic or overflow for any `u16` length value | Kani P0 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

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
- BC-2.20.004 — composes with (accept path: length in `[7, 65535]`)
- BC-2.20.014 — depends on (resync/dedup machinery for malformed frames mid-walk)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_tpkt_header`: `let length = u16::from_be_bytes([data[2], data[3]]); if length < 7 { return None }`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — TPKT length field: "total TPKT packet length including this 4-byte header"

## Story Anchor

STORY-184

## VP Anchors

- VP-048 (Kani P0) — TPKT Header Parse Safety and Four-Way Totality; registered F2
  INTEGRATE sub-burst per VP-INDEX.md v2.48; traces BC-2.20.001..004

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — Kani P0 target |
