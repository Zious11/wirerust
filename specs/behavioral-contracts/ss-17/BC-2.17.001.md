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

# BC-2.17.001: parse_enip_header Returns None for Input Shorter Than 24 Bytes

## Description

`parse_enip_header(data: &[u8]) -> Option<EnipHeader>` is the pure-core entry function for
EtherNet/IP encapsulation-header parsing. When `data.len() < 24`, the function returns `None`
immediately without accessing any bytes. The ENIP encapsulation header is exactly 24 bytes
(fixed by the ODVA EtherNet/IP specification): command (2 LE) + length (2 LE) +
session_handle (4 LE) + status (4 LE) + sender_context (8 opaque) + options (4 LE). No
partial header is ever decoded. This is the reject/short path contract; the accept path is
BC-2.17.002.

## Preconditions

1. `data` is a `&[u8]` slice of reassembled, in-order TCP bytes (per `StreamHandler::on_data`).
2. `data.len() < 24` — fewer bytes than the minimum ENIP encapsulation header.
3. No alignment or endianness assumptions affect this path: zero bytes are read.

## Postconditions

1. `parse_enip_header(data)` returns `None`.
2. No bytes in `data` are accessed (no panics possible by construction).
3. The function is pure: no I/O, no global state mutation, no side effects.
4. The caller (frame-walk loop in `on_data`) treats `None` as "partial header" and stashes
   `data` into the carry buffer.

## Invariants

1. **Minimum-header guard**: the 24-byte minimum is fixed by the ODVA EtherNet/IP
   encapsulation specification and is not configurable. Any data shorter than 24 bytes is
   necessarily incomplete. [SPEC: ADR-010 Decision 2]
2. **Purity**: `parse_enip_header` is a pure-core free function — VP-032 Sub-A Kani target.
   No mutation of any state occurs inside this function.
3. **No partial decode**: the function does not attempt to decode the command or any other
   field when `data.len() < 24`. Partial decoding would produce misleading struct values.
4. **Carry-buffer handoff**: `None` from this function is the signal for the frame-walk loop
   to stash remaining bytes into `EnipFlowState.carry` (bounded to
   `MAX_ENIP_CARRY_BYTES = 600`). The carry-buffer cap is enforced by the caller, not this
   function.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data.len() == 0` (empty slice) | Returns `None` — zero bytes, no access, no panic |
| EC-002 | `data.len() == 1` | Returns `None` — one byte, not accessed, no panic |
| EC-003 | `data.len() == 23` (one byte short) | Returns `None` — 23 bytes, none accessed, no panic |
| EC-004 | `data.len() == 24` (exactly minimum) | Returns `Some(EnipHeader{...})` — accept path; see BC-2.17.002 |
| EC-005 | `data` is all zeroes, `len == 23` | Returns `None`; does not decode `command=0x0000` from bytes 0–1 |

## Canonical Test Vectors

| Input (hex bytes, total length) | Expected result | Category |
|---------------------------------|----------------|---------|
| `[/* 0 bytes */]` | `None` | reject: empty |
| `[0x65, 0x00, /* 22 more */]` (23 bytes total) | `None` | reject: one byte short |
| `[0x65, 0x00, 0x04, 0x00, /* 20 more */]` (24 bytes) | `Some(EnipHeader{command:0x0065,...})` | accept: exact minimum — see BC-2.17.002 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-A: `parse_enip_header(data)` returns `None` for all inputs with `len < 24`; never panics for any symbolic input up to BOUND=48 | Kani: `#[kani::proof] #[kani::unwind(49)]` with symbolic `[u8; 48]` and symbolic `len <= 48`; asserts `result.is_none()` when `len < 24` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — this BC defines the reject-path contract for the ENIP header parser, which is the foundational entry function for all EtherNet/IP analysis in the ICS/OT analyzer |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — ENIP flows are only routed after TLS/HTTP content rules fail; this BC fires only on port-44818 classified flows) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.17.002 — composes with (accept path: `data.len() >= 24` returns `Some(EnipHeader)`)
- BC-2.17.003 — depends on (validity gate uses parsed header from BC-2.17.002)
- BC-2.17.016 — composes with (carry-buffer stash / `is_non_enip` latch is the caller's response to `None`)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn parse_enip_header(data: &[u8]) -> Option<EnipHeader>` pure-core free function
- `src/analyzer/enip.rs` — frame-walk loop in `EnipAnalyzer::on_data()`: `if buf.len() - cursor < 24 { carry = buf[cursor..]; break }`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 2` — header parse pseudocode
- `.factory/specs/verification-properties/vp-032-enip-parse-safety.md §Sub-A` — Kani proof skeleton

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-A — ENIP header parse safety (None path: `len < 24` assertion)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 2 (parse_enip_header pseudocode); VP-032 Sub-A skeleton; ODVA EtherNet/IP spec (24-byte fixed header) |
| **Confidence** | high — 24-byte fixed header length is normative ODVA specification; VP-032 Sub-A Kani proof verifies this for all symbolic inputs |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same bytes always produce same result |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-032 Sub-A Kani target |
