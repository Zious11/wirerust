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
subsystem: SS-21
capability: CAP-21
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

# BC-2.21.004: `parse_s7comm_header` Returns None for Input Shorter Than 10 Bytes

## Description

`parse_s7comm_header(data: &[u8]) -> Option<S7commHeader>` is the pure-core entry
function for classic S7comm (protocol-ID `0x32`) header parsing (ADR-014 Decision 9
item 3). Per this BC's design decision, `data` is the COTP DT payload slice beginning
**at** the already-classified protocol-ID byte (`&tpkt_payload[payload_offset..]` from
BC-2.20.009), so `data[0]` is expected to equal `0x32` (re-validated defensively, see
BC-2.21.005). The classic S7comm common header — Protocol ID (1) + ROSCTR (1) +
Reserved (2) + PDU Reference (2, big-endian) + Parameter Length (2, big-endian) + Data
Length (2, big-endian) — is exactly 10 bytes for Job/Ack_Data/Userdata ROSCTR values
(Ack requires 2 additional bytes, BC-2.21.008). When `data.len() < 10`, the function
returns `None` immediately without accessing any field beyond the length check.

## Preconditions

1. `data` is the slice `S7commAnalyzer::on_data` passes for a DT frame already
   classified `protocol_id == Some(0x32)` (BC-2.21.002 Postcondition 3).
2. `data.len() < 10`.

## Postconditions

1. `parse_s7comm_header(data)` returns `None`.
2. No bytes beyond the length check are accessed; no panics are possible for any
   `data.len()` in `[0, 9]`.
3. The function is pure: no I/O, no global state mutation, no side effects.
4. `S7commAnalyzer` treats this `None` identically to an incomplete-TPKT-frame
   condition at the SS-20 layer: the already-extracted TPKT frame is presumed
   internally inconsistent (a TPKT frame whose declared `length` was large enough to
   be accepted by BC-2.20.004, yet whose COTP+S7comm payload does not contain a full
   10-byte S7comm header) — this is a malformed-frame condition, not a
   carry-buffer-incomplete condition (the frame-walk loop already confirmed the full
   TPKT frame was delivered; the shortfall is *within* the delivered frame). The first
   occurrence per flow direction emits one T0814 (Anomaly/Possible/Medium) finding via
   `malformed_header_reported_c2s`/`_s2c` (BC-2.21.001), mirroring IEC-104's
   malformed-ASDU-length treatment (BC-2.19.026).

## Invariants

1. **Minimum-header guard**: 10 bytes is the smallest slice from which any S7comm
   ROSCTR-driven header can be identified; not configurable.
2. **Purity**: `parse_s7comm_header` is a pure-core free function — cargo-fuzz P1
   candidate per ADR-014 Decision 9 (combined with the frame-walk loop, not a Kani P0
   target — S7comm header parsing follows `parse_asdu`'s fuzz-not-Kani precedent,
   BC-2.19.015 sibling).
3. **Distinct from SS-20's incomplete-frame path**: this `None` occurs on an
   *already-complete* TPKT frame per BC-2.20.004's accept path — it is a
   malformed-content finding, not a reassembly-in-progress condition, and is
   therefore dedup-flagged and finding-emitting, unlike SS-20's carry-stash `None`
   paths.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data.len() == 0` (DT frame with `protocol_id: Some(0x32)` but the payload_offset+1 slice is empty — a TPKT frame containing only the protocol-ID byte) | Returns `None`; T0814 emitted (first occurrence per direction) |
| EC-002 | `data.len() == 9` (one byte short of the 10-byte minimum) | Returns `None`; T0814 emitted (first occurrence per direction) |
| EC-003 | `data.len() == 10` (exactly minimum) | Proceeds to ROSCTR/field validation; see BC-2.21.006/007 |
| EC-004 | A second malformed-length frame arrives on the same flow direction after the first triggered the dedup flag | Returns `None`; **no** second T0814 emitted (dedup flag already set) |

## Canonical Test Vectors

| Input (`data`, hex bytes, length) | Expected result | Category |
|---|---|---|
| `[]` (0 bytes) | `None` + T0814 (first occurrence) | reject: empty |
| `[0x32, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02]` (8 bytes) | `None` + T0814 (first occurrence) | reject: one byte short of common-header minimum minus buffer |
| `[0x32, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00]` (9 bytes) | `None` + T0814 (first occurrence) | reject: exactly one byte short |
| `[0x32, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00]` (10 bytes) | `Some(S7commHeader{..})` | accept: exact minimum — see BC-2.21.006 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `parse_s7comm_header` returns `None` for all inputs with `len < 10`; never panics for any symbolic input up to a bounded length | cargo-fuzz P1 (combined TPKT→COTP→S7comm no-panic harness per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this BC is the length-reject path for the classic S7comm header parser, the entry function for all classic-S7comm dissection |
| L2 Domain Invariants | None directly (bounds-safety contract; findings-cap concerns belong to B2/INV-6) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); ADR-014 Decision 9 |
| ADR | ADR-014 Decisions 2, 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0814 (Denial of Service) — malformed-length anomaly signal only; full emission wiring (verdict/confidence/dedup call-site) is a B2 (MITRE technique-emission BC) responsibility; this BC names the obligation, does not author the emission contract |

## Related BCs

- BC-2.20.009 — depends on (the DT payload slice this function receives)
- BC-2.21.001 — depends on (`malformed_header_reported_c2s`/`_s2c` dedup flags)
- BC-2.21.005 — composes with (next rejection: protocol-ID byte mismatch when len ≥ 10)
- BC-2.21.006 — composes with (accept path)
- BC-2.19.026 — composes with (IEC-104 malformed-length EMIT-WITH-DEDUP precedent this BC mirrors)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `fn parse_s7comm_header(data: &[u8]) -> Option<S7commHeader>` pure-core free function
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 9` — pure-core free-fn design, cargo-fuzz target

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst, anticipated VP-048
range. cargo-fuzz P1 candidate.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (the function itself is pure; the finding-emission/dedup consequence is the caller's responsibility) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — cargo-fuzz P1 target |
