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

# BC-2.20.001: `parse_tpkt_header` Returns None for Input Shorter Than 4 Bytes

## Description

`parse_tpkt_header(data: &[u8]) -> Option<TpktHeader>` is the pure-core entry function
for TPKT (RFC 1006) header parsing — the outer 4-byte framing layer present on every
TCP segment carrying ISO-on-TCP traffic. The TPKT header is exactly 4 bytes: version (1
byte), reserved (1 byte), and length (2 bytes, big-endian). When `data.len() < 4`, the
function returns `None` immediately without accessing any bytes. This is the
length-reject path; the accept path is BC-2.20.004.

**Clarifying note (threshold disambiguation):** the `4` in this BC's title is a
**structural read-guard** — the minimum number of bytes present in `data` needed to even
read the four TPKT header fields (version, reserved, length) without an out-of-bounds
access. It is unrelated to, and must not be conflated with, the **decoded-length semantic
floor of `7`** enforced by BC-2.20.003/BC-2.20.004, which governs the numeric value found
*inside* the length field once the header has been successfully read. `data.len() < 4` is
a read-guard failure (this BC); a decoded `length < 7` is a semantic-floor failure
(BC-2.20.003) that only applies once `data.len() >= 4` and the version byte is valid.

## Preconditions

1. `data` is a `&[u8]` slice of reassembled, in-order TCP bytes for a flow classified
   `DispatchTarget::S7comm` (TCP port 102), delivered to `S7commAnalyzer::on_data`.
2. `data.len() < 4` — fewer bytes than the minimum 4-byte TPKT header.
3. No alignment assumptions affect this path: zero bytes are read.

## Postconditions

1. `parse_tpkt_header(data)` returns `None`.
2. No bytes in `data` are accessed; no panics are possible.
3. The function is pure: no I/O, no global state mutation, no side effects.
4. The frame-walk caller (SS-21) treats `None` as "incomplete TPKT header" and stashes
   `data` into the direction's carry buffer (`carry_c2s`/`carry_s2c` on
   `S7commFlowState`, bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535` per
   BC-2.20.014).

## Invariants

1. **Minimum-header guard**: the 4-byte minimum is fixed by RFC 1006 §6 (version + reserved +
   2-byte length). It is not configurable.
2. **Purity**: `parse_tpkt_header` is a pure-core free function — Kani P0 target per
   ADR-014 Decision 9. No state mutation occurs inside this function.
3. **No partial decode**: the function does not attempt to decode the version or length
   field when `data.len() < 4`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data.len() == 0` (empty slice — zero-length TCP payload) | Returns `None` — no bytes accessed, no panic |
| EC-002 | `data.len() == 1` | Returns `None` — one byte, not accessed |
| EC-003 | `data.len() == 3` (one byte short) | Returns `None` — 3 bytes, none accessed |
| EC-004 | `data.len() == 4` (exactly minimum) | Proceeds to version and length validation; see BC-2.20.002/003/004 |
| EC-005 | `data = [0x03, 0x00, 0x00]` (3 bytes, looks like start of valid frame) | Returns `None` — length check fires before any field access |

## Canonical Test Vectors

| Input (hex bytes, total length) | Expected result | Category |
|---------------------------------|----------------|---------|
| `[]` (0 bytes) | `None` | reject: empty |
| `[0x03, 0x00, 0x00]` (3 bytes) | `None` | reject: one byte short |
| `[0x03, 0x00, 0x00, 0x07]` (4 bytes) | `Some(TpktHeader{version:3, length:7})` | accept: exact minimum — see BC-2.20.004 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `parse_tpkt_header(data)` returns `None` for all inputs with `len < 4`; never panics for any symbolic input up to a bounded length; for any returned `Some(h)`, `h.length` is in `[7, 65535]` with no integer overflow | Kani P0 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst (anticipated VP-048 range) |
| No panic on arbitrary byte input at the `on_data` entry point (fuzz harness covering the TPKT→COTP→S7comm parse chain) | cargo-fuzz P1 (per ADR-014 Decision 9) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the length-reject path for the TPKT header parser, the foundational entry function for all ISO-on-TCP framing in SS-20 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — S7comm/ISO-on-TCP flows are only routed after TLS/HTTP content rules fail; this BC fires only on port-102-classified flows per ADR-014 Decision 2 Rule 9) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned — not yet in src tree); ADR-014 Decisions 1, 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.20.002 — composes with (next rejection: version byte ≠ 0x03 when len ≥ 4)
- BC-2.20.003 — composes with (length field < 7 rejection path)
- BC-2.20.004 — composes with (accept path: `data.len() >= 4`, version valid, length valid)
- BC-2.20.013 — depends on (carry-buffer stash behavior when this function returns `None` mid-walk)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_tpkt_header(data: &[u8]) -> Option<TpktHeader>` pure-core free function
- `src/analyzer/iso_on_tcp.rs` (planned) — frame-walk loop in `S7commAnalyzer::on_data()`: `if buf.len() - cursor < 4 { carry = buf[cursor..].to_vec(); break }`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — frozen `TpktHeader`/interface definition
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 9` — pure-core free-fn design, Kani target

## Story Anchor

STORY-184 (also a formal-hardening re-verification anchor for STORY-194)

## VP Anchors

- VP-048 (Kani P0) — TPKT Header Parse Safety and Four-Way Totality; registered F2
  INTEGRATE sub-burst per VP-INDEX.md v2.48; traces BC-2.20.001..004
- VP-055 (cargo-fuzz P1) — S7comm/ISO-on-TCP combined parse-chain no-panic fuzz
  (`fuzz_s7comm_parser`); registered representative-subset source_bc includes this BC

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same bytes always produce same result |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — Kani P0 target |
