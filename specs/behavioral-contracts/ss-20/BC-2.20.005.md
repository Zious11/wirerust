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

# BC-2.20.005: `parse_cotp_header` Returns None for Input Shorter Than 2 Bytes

## Description

`parse_cotp_header(tpkt_payload: &[u8]) -> Option<CotpHeader>` is the pure-core entry
function for COTP (ISO 8073 / ITU-T X.224) TPDU parsing. `tpkt_payload` is the slice
following the 4-byte TPKT header (i.e. `data[4..length]` from an already-accepted
`TpktHeader`, BC-2.20.004). The minimum readable COTP prefix is 2 bytes: the Length
Indicator (LI, offset 0) and the TPDU-code byte (offset 1, whose high nibble identifies
CR/CC/DT). When `tpkt_payload.len() < 2`, the function returns `None` without
attempting to classify a TPDU type. This is the length-reject path; the TPDU-type
recognition paths are BC-2.20.007 through BC-2.20.011.

## Preconditions

1. `tpkt_payload` is the byte slice following an already-validated 4-byte TPKT header
   (per BC-2.20.004's accept path); it is `&[u8]`, not assumed non-empty.
2. `tpkt_payload.len() < 2` — fewer bytes than the minimum LI + TPDU-code prefix.

## Postconditions

1. `parse_cotp_header(tpkt_payload)` returns `None`.
2. No bytes in `tpkt_payload` are accessed beyond the length check; no panics are
   possible, including for `tpkt_payload.len() == 0`.
3. The function is pure: no I/O, no global state mutation, no side effects.
4. `S7commAnalyzer` (SS-21) treats this `None` as an incomplete-COTP-header condition:
   the already-accepted TPKT frame's bytes are stashed to the carry buffer to await
   more data, mirroring the TPKT-level incomplete-frame handling in BC-2.20.013 (a TPKT
   frame with `length == 4` — header-only — legitimately produces an empty
   `tpkt_payload`, which also falls into this reject path).

## Invariants

1. **Minimum-prefix guard**: 2 bytes (LI + TPDU-code) is the smallest slice from which
   any TPDU type can be identified. It is not configurable.
2. **Purity**: `parse_cotp_header` is a pure-core free function — Kani P0 target per
   ADR-014 Decision 9.
3. **No partial decode**: the function does not attempt to read the LI value or the
   TPDU-code nibble when `tpkt_payload.len() < 2`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `tpkt_payload.len() == 0` (a TPKT `length == 4` header-only frame) | Returns `None` — legitimately empty payload, not itself an error at the TPKT layer |
| EC-002 | `tpkt_payload.len() == 1` (only the LI byte delivered so far) | Returns `None` |
| EC-003 | `tpkt_payload.len() == 2` (exactly minimum) | Proceeds to TPDU-type recognition; see BC-2.20.006 through BC-2.20.011 |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[]` (0 bytes) | `None` | reject: empty (TPKT header-only frame) |
| `[0x02]` (1 byte) | `None` | reject: one byte short |
| `[0x02, 0xF0]` (2 bytes) | proceeds to TPDU-type/LI validation | accept-boundary — see BC-2.20.006/009 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `parse_cotp_header(tpkt_payload)` returns `None` for all inputs with `len < 2`; never panics for `len == 0`; no out-of-bounds index | Kani P0 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the length-reject path for the COTP header parser, the second layer of ISO-on-TCP framing consumed by every S7comm session |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decisions 1, 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.20.004 — depends on (`tpkt_payload` is derived from an already-accepted `TpktHeader`)
- BC-2.20.006 — composes with (LI-declares-more-than-available truncation path)
- BC-2.20.007 — composes with (CR TPDU recognition, once length passes)
- BC-2.20.008 — composes with (CC TPDU recognition)
- BC-2.20.009 — composes with (DT TPDU recognition, non-empty payload)
- BC-2.20.010 — composes with (DT TPDU recognition, empty payload)
- BC-2.20.011 — composes with (unrecognized TPDU-type rejection)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header(tpkt_payload: &[u8]) -> Option<CotpHeader>` pure-core free function
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — frozen `CotpHeader`/`CotpTpduType` interface definition
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 4` — ITU-T X.224 ≡ ISO/IEC 8073:1997 as the permitted primary specification source for COTP field layout

## Story Anchor

STORY-185

## VP Anchors

- VP-049 (Kani P0) — COTP Header Parse Safety, TPDU-Type Exhaustiveness, and
  Protocol-ID Extraction Totality; registered F2 INTEGRATE sub-burst per VP-INDEX.md
  v2.48; traces BC-2.20.005..012
- VP-055 (cargo-fuzz P1) — S7comm/ISO-on-TCP combined parse-chain no-panic fuzz
  (`fuzz_s7comm_parser`); registered representative-subset source_bc includes this BC

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — Kani P0 target |
