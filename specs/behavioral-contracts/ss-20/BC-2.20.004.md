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

# BC-2.20.004: `parse_tpkt_header` Returns Some(TpktHeader) for Valid Input (Happy Path)

## Description

When `data.len() >= 4`, `data[0] == 0x03`, and the big-endian `u16` decoded from
`data[2..4]` is in `[4, 65535]`, `parse_tpkt_header` returns
`Some(TpktHeader { version: 3, length })`. The reserved byte at `data[1]` is read as
part of the struct's implicit layout but is **not validated** — any value is accepted,
matching common real-world ISO-on-TCP stacks that do not always zero it. This is the
accept path composing the three reject paths of BC-2.20.001/002/003.

## Preconditions

1. `data.len() >= 4`.
2. `data[0] == 0x03`.
3. The big-endian `u16` decoded from `data[2..4]` is in `[4, 65535]` (the full valid
   range of a `u16` minus the `[0,3]` malformed band rejected by BC-2.20.003).

## Postconditions

1. `parse_tpkt_header(data)` returns `Some(TpktHeader { version: 3, length })` where
   `length` is exactly the big-endian `u16` decoded from `data[2..4]`.
2. `data[1]` (reserved byte) is not inspected for validity; its value has no effect on
   the result.
3. No panic occurs for any valid input satisfying the preconditions.
4. The caller (`S7commAnalyzer::on_data`, SS-21) uses `length` to determine how many
   total bytes (including the 4-byte TPKT header) constitute this frame; if
   `data.len() < length as usize`, the frame is incomplete and the walk stashes the
   partial bytes to the direction's carry buffer (BC-2.20.013) rather than treating this
   as a parse failure — `parse_tpkt_header` itself has already succeeded and is not
   re-invoked on the same header bytes.

## Invariants

1. **Reserved byte is a don't-care**: `TpktHeader` (ADR-014 Decision 1 frozen struct)
   has exactly two fields, `version` and `length` — there is no `reserved` field.
   `parse_tpkt_header` never surfaces the reserved byte's value to callers.
2. **Length upper bound is representational, not enforced**: `65535` is the maximum
   value representable by the `u16` length field; `parse_tpkt_header` performs no
   additional upper-bound check beyond what the type system already guarantees. A
   `length` of exactly `65535` (the maximum-representable, "oversized" edge case called
   out in the F2 authoring scope) is a legal accept — see BC-2.20.014 for why this
   maximum coincides exactly with the carry-buffer ceiling.
3. **Total ordering of the three reject paths**: BC-2.20.001 (too-short input),
   BC-2.20.002 (bad version), and BC-2.20.003 (length `< 4`) together with this BC's
   accept path are jointly exhaustive and mutually exclusive over all possible `data`
   inputs — every call to `parse_tpkt_header` falls into exactly one of these four BCs.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `length == 4` (header-only TPKT packet, no COTP payload at all) | `Some(TpktHeader{version:3, length:4})` — a legal, if degenerate, TPKT packet |
| EC-002 | `length == 65535` (maximum representable `u16` — "oversized-length-field" edge case) | `Some(TpktHeader{version:3, length:65535})` — accepted; stresses the carry buffer to its exact ceiling (BC-2.20.014) |
| EC-003 | `data[1]` (reserved byte) is a non-zero value, e.g. `0xFF` | Accepted identically to `data[1] == 0x00` — reserved byte is never validated |
| EC-004 | `data.len() > length as usize` (more bytes delivered than the declared frame length — a second frame follows immediately) | `parse_tpkt_header` still returns `Some` for the first `length` bytes; the frame-walk loop advances the cursor by `length` and re-invokes on the remainder |
| EC-005 | `data.len() == length as usize` exactly (single complete frame, no trailing bytes) | `Some(TpktHeader{..})`; no carry stash needed |

## Canonical Test Vectors

| Input (hex bytes) | Expected result | Category |
|--------------------|----------------|---------|
| `[0x03, 0x00, 0x00, 0x04]` | `Some(TpktHeader{version:3, length:4})` | happy-path: minimum valid |
| `[0x03, 0x00, 0x00, 0x07]` | `Some(TpktHeader{version:3, length:7})` | happy-path: minimal CR/CC-carrying frame |
| `[0x03, 0xFF, 0xFF, 0xFF]` | `Some(TpktHeader{version:3, length:65535})` | happy-path: maximum-representable length (reserved byte non-zero, ignored) |
| `[0x03, 0x00, 0x00, 0x0A, ...6 more payload bytes]` (10 bytes total) | `Some(TpktHeader{version:3, length:10})` | happy-path: header + 6-byte payload |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For all valid inputs, `parse_tpkt_header` returns `Some(TpktHeader{version:3, length})` with `length` exactly matching the decoded big-endian `u16`; no panic; the four BC-2.20.001/002/003/004 paths are exhaustive and non-overlapping | Kani P0 (per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |
| No panic on arbitrary byte input at `on_data` entry (combined TPKT→COTP→S7comm parse chain fuzz harness) | cargo-fuzz P1 (per ADR-014 Decision 9) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC defines the accept path for the TPKT header parser, the entry point every downstream COTP/S7comm parse depends on |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decisions 1, 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.20.001 — composes with (length-reject-by-input-size path)
- BC-2.20.002 — composes with (version-reject path)
- BC-2.20.003 — composes with (length-field-too-small reject path)
- BC-2.20.005 — depends on (the accepted `TpktHeader`'s payload — `data[4..length]` — is what `parse_cotp_header` subsequently consumes)
- BC-2.20.013 — depends on (incomplete-frame handling when `data.len() < length`)
- BC-2.20.014 — depends on (the `length == 65535` maximum matching the carry-buffer ceiling)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `pub struct TpktHeader { pub version: u8, pub length: u16 }` (frozen, ADR-014 Decision 1)
- `src/analyzer/iso_on_tcp.rs` (planned) — `pub fn parse_tpkt_header(data: &[u8]) -> Option<TpktHeader>` accept-path branch
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — frozen interface: "version: u8, // always 3 for a valid TPKT packet"; "length: u16, // total packet length INCLUDING this 4-byte header"

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst per ADR-014 Decision 9,
anticipated VP-048 range. This BC is a Kani P0 candidate for exhaustiveness of the
four-way parse-result partition, and a cargo-fuzz P1 candidate.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — Kani P0 target |
