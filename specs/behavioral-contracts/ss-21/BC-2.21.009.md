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

# BC-2.21.009: Declared `param_length`/`data_length` Are Bounds-Checked Against Remaining Bytes Before Parameter/Data Block Access (Safe-Reject on Inconsistency)

## Description

`parse_s7comm_header` (BC-2.21.006/008) extracts `param_length` and `data_length` as
raw `u16` values without validating them against the actual number of bytes remaining
in `data` after the header. This BC specifies the caller-side (`S7commAnalyzer`)
obligation: before slicing out the parameter block (`data[header_len..header_len +
param_length]`) or the data block (`data[header_len + param_length..header_len +
param_length + data_length]`), the caller MUST verify
`data.len() >= header_len + param_length as usize + data_length as usize`. If this
check fails, the frame is treated as malformed (declared lengths exceed available
bytes) — safe-reject, no out-of-bounds slice is ever attempted. This mirrors IEC-104's
ASDU minimum-length guard (BC-2.19.015) applied to S7comm's two-length-field header
instead of ASDU's implicit body length.

## Preconditions

1. `parse_s7comm_header(data)` returned `Some(header)` (BC-2.21.006 or BC-2.21.008).
2. `data.len() < header.header_len + header.param_length as usize + header.data_length as usize`
   (declared lengths exceed what is actually present).

## Postconditions

1. No slice into `data` beyond `data.len()` is ever attempted — the bounds check
   happens strictly before any `data[header_len..]` or `data[header_len +
   param_length..]` indexing.
2. The frame is treated as malformed: `S7commAnalyzer` emits one T0814
   (Anomaly/Possible/Medium) per flow direction, guarded by the same
   `malformed_header_reported_c2s`/`_s2c` dedup flag as BC-2.21.004/007/008 (all four
   conditions collectively answer "was this frame's declared structure internally
   consistent with its actual byte length?").
3. No function-code or Userdata classification (BC-2.21.010 onward) is attempted for a
   frame that fails this check — classification always requires a successfully
   bounds-validated parameter block.

## Invariants

1. **`u16 + u16` cannot overflow `usize`**: on any platform wirerust targets (32-bit or
   64-bit), `header_len (10 or 12) + param_length (max 65,535) + data_length (max
   65,535)` fits comfortably within `usize::MAX`; no arithmetic overflow is possible
   in this bounds computation.
2. **Bounds check precedes every downstream slice**: this is the single choke point
   through which all classification logic (Groups 3 and 4) must pass; no BC in this
   feature ever slices the parameter or data block without this check having already
   succeeded.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `param_length == 0` and `data_length == 0` and `data.len() == header_len` exactly | Bounds check passes trivially (`data.len() >= header_len + 0 + 0`); an empty parameter block is legitimate (e.g. some Ack_Data responses) — classification proceeds to the empty-parameter-block case (BC-2.21.017's "no FC byte present" edge) |
| EC-002 | `param_length == 65,535` (maximum representable `u16`) but only 20 bytes actually follow the header | Bounds check fails; malformed-header T0814 (dedup-guarded) |
| EC-003 | `data_length` is large and plausible (e.g. a multi-kilobyte Download Block payload) and the TPKT frame's declared length (SS-20) was large enough to carry it | Bounds check passes; this is the expected shape for large block-download traffic (ADR-014 Decision 8's rationale for the 65,535-byte carry-buffer ceiling) |

## Canonical Test Vectors

| `header.param_length` / `header.data_length` / `data.len() - header_len` | Expected outcome | Category |
|---|---|---|
| `2` / `0` / `2` (exact match) | Bounds check passes | happy-path |
| `2` / `0` / `1` (one byte short) | Bounds check fails; malformed-header T0814 | reject: declared length exceeds available bytes |
| `0` / `0` / `0` (empty parameter and data blocks) | Bounds check passes trivially | edge-case: no function code present |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| No out-of-bounds slice is ever constructed from `header_len`, `param_length`, and `data_length` for any combination of `u16` values and any `data.len()` | Kani P0 candidate (arithmetic/bounds safety over the full `u16 × u16` space is small enough for exhaustive symbolic proof) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — the bounds gate that makes all downstream function-code classification memory-safe |
| L2 Domain Invariants | None directly (bounds-safety contract) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0814 (Denial of Service) — malformed-header anomaly signal only; emission wiring is a B2 responsibility |

## Related BCs

- BC-2.21.006 — depends on (`param_length`/`data_length` values this BC validates)
- BC-2.21.008 — depends on (same validation applies to Ack's `header_len: 12`)
- BC-2.21.010 through BC-2.21.023 — depend on (this bounds check is a precondition for every classification BC)
- BC-2.19.015 — composes with (IEC-104 ASDU minimum-length guard precedent this BC mirrors)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — bounds check in `S7commAnalyzer::on_data` (or a helper) immediately after `parse_s7comm_header` returns `Some`, before any parameter/data-block slicing

## Story Anchor

STORY-187 (also a formal-hardening re-verification anchor for STORY-194)

## VP Anchors

- VP-051 (Kani P0) — S7comm Header Bounds-Before-Slice Safety; registered F2
  INTEGRATE sub-burst per VP-INDEX.md v2.48; traces BC-2.21.004, BC-2.21.009
- VP-055 (cargo-fuzz P1) — S7comm/ISO-on-TCP combined parse-chain no-panic fuzz
  (`fuzz_s7comm_parser`); registered representative-subset source_bc includes this BC

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads header fields only; the emit-on-failure consequence touches per-flow dedup state |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (the bounds-check arithmetic itself; caller-side finding emission follows the analyzer's single-flow-owner pattern) |
| **Overall classification** | pure core (bounds arithmetic) — Kani P0 target |
