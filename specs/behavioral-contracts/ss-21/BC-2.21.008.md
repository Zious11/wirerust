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
input-hash: "8f268fc"
---

# BC-2.21.008: `parse_s7comm_header` for ROSCTR=Ack (0x02) Requires 12 Bytes (Error Class + Error Code)

## Description

The Ack ROSCTR (`0x02`) is a bare acknowledgment carrying no parameter or data block —
only the 10-byte common header (BC-2.21.006) plus two additional bytes: Error Class
(`data[10]`) and Error Code (`data[11]`). This is structurally distinct from Ack_Data
(`0x03`), which carries a full parameter/data block per the common 10-byte header
alone. When `data[1] == 0x02` and `data.len() < 12`, `parse_s7comm_header` returns
`None` (truncated Ack); when `data.len() >= 12`, it returns
`Some(S7commHeader { rosctr: Ack, error_class: Some(data[10]), error_code:
Some(data[11]), header_len: 12, .. })`.

## Preconditions

1. `data.len() >= 10`, `data[0] == 0x32`, `data[1] == 0x02` (Ack ROSCTR).

## Postconditions

1. If `data.len() < 12`: returns `None`. `S7commAnalyzer` treats this as a
   malformed-header condition (shares the dedup flag with BC-2.21.004/007).
2. If `data.len() >= 12`: returns
   `Some(S7commHeader { rosctr: Ack, pdu_reference, param_length, data_length,
   error_class: Some(data[10]), error_code: Some(data[11]), header_len: 12 })`, where
   `pdu_reference`/`param_length`/`data_length` are extracted identically to
   BC-2.21.006 (the common-header fields are present at the same offsets regardless of
   ROSCTR value).
3. `error_class`/`error_code` are `Some` **only** when `rosctr == Ack`; every other
   `S7commHeader` (Job, Ack_Data, Userdata) has both fields `None` (BC-2.21.006
   Postcondition 1).
4. No function-code classification (Group 3/4, BC-2.21.010 onward) is attempted for an
   Ack-ROSCTR header — Ack carries no parameter block to classify; `S7commAnalyzer`
   logs the observed error class/code and takes no further dissection action.

## Invariants

1. **Ack is structurally minimal**: 12 bytes is the complete Ack frame length; any
   `param_length`/`data_length` value extracted for an Ack header is expected to be
   `0` in conformant traffic but is not validated as such by this function (a non-zero
   value on an Ack frame is a downstream anomaly-detection concern, out of B1 scope).
2. **`error_class`/`error_code` presence is exhaustively tied to `rosctr == Ack`**: no
   other ROSCTR value ever produces `Some` for either field.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data.len() == 10` (Ack ROSCTR, but only the common header present) | Returns `None` — truncated Ack, malformed-header dedup |
| EC-002 | `data.len() == 11` (one byte short of the 12-byte Ack minimum) | Returns `None` — truncated Ack |
| EC-003 | `data.len() == 12` exactly | `Some(...)` with `header_len: 12` |
| EC-004 | `error_class == 0x00` and `error_code == 0x0000`-equivalent (no error reported) | Extracted verbatim; a zero error class/code is a normal successful-Ack value, not itself flagged |

## Canonical Test Vectors

| Input (`data`, hex bytes) | Expected result | Category |
|---|---|---|
| `32 02 00 00 00 01 00 00 00 00` (10 bytes) | `None` (truncated Ack) | reject: missing error class/code |
| `32 02 00 00 00 01 00 00 00 00 00` (11 bytes) | `None` (truncated Ack) | reject: one byte short |
| `32 02 00 00 00 01 00 00 00 00 00 00` (12 bytes) | `Some({rosctr: Ack, error_class: Some(0), error_code: Some(0), header_len: 12})` | happy-path: minimal Ack |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| Ack-specific 12-byte minimum is correctly enforced; `error_class`/`error_code` extraction is correct and never produced for non-Ack ROSCTR values | cargo-fuzz P1 (combined harness) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — completes the ROSCTR-conditional header-length contract alongside BC-2.21.006/007 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0814 (Denial of Service) — malformed-header (truncated-Ack) anomaly signal only; emission wiring is a B2 responsibility |

## Related BCs

- BC-2.21.006 — composes with (shares common-header field extraction for `pdu_reference`/`param_length`/`data_length`)
- BC-2.21.004 — composes with (shares malformed-header dedup flag)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `fn parse_s7comm_header`, Ack-specific `header_len == 12` branch

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — cargo-fuzz P1, combined harness.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — cargo-fuzz P1 target |
