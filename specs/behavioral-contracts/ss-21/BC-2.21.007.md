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

# BC-2.21.007: `parse_s7comm_header` Returns None for an Unrecognized ROSCTR Byte (Safe-Reject, No Force-Fit)

## Description

`Rosctr` (this BC's design, ADR-014 Decision 9 item 3) models exactly the four ROSCTR
values classic S7comm's steady-state traffic uses: `0x01` Job, `0x02` Ack, `0x03`
Ack_Data, `0x07` Userdata. When `data[1]` (the ROSCTR byte) is none of these four
values, `parse_s7comm_header` returns `None` rather than guessing a classification —
mirroring SS-20's `CotpTpduType` exhaustive-but-bounded design (BC-2.20.011) and
IEC-104's reserved-TypeID handling (BC-2.19.022). This is the primary safe-reject path
for malformed or non-conformant classic S7comm traffic at the ROSCTR layer.

## Preconditions

1. `data.len() >= 10`, `data[0] == 0x32` (BC-2.21.004/005 passed).
2. `data[1] ∉ {0x01, 0x02, 0x03, 0x07}`.

## Postconditions

1. `parse_s7comm_header(data)` returns `None`.
2. No panic occurs for any of the 252 remaining `u8` values not covered by the four
   recognized ROSCTR values.
3. `S7commAnalyzer` treats this `None` as a malformed-header condition, subject to the
   same per-direction T0814 dedup-and-emit treatment as BC-2.21.004's length-reject
   path (`malformed_header_reported_c2s`/`_s2c`, BC-2.21.001) — the two `None`-producing
   conditions (too-short, unrecognized-ROSCTR) share one dedup flag per direction
   since both represent "this frame's S7comm header could not be parsed," not two
   distinct anomaly classes.

## Invariants

1. **No force-fit**: an unrecognized ROSCTR byte is never coerced into one of the four
   modeled variants — this is a deliberate scope decision matching the "never
   force-fit" language used throughout ADR-014 (Decision 2's protocol_id table, this
   feature's dispatch philosophy generally).
2. **Exhaustive-but-bounded ROSCTR set**: `Rosctr` models exactly the values S7comm's
   steady-state protocol uses in practice; it is not exhaustive over all 256 `u8`
   values by design, symmetric with `CotpTpduType`'s three-of-many-ISO-8073-types
   scope (BC-2.20.011 Invariant 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data[1] == 0x00` | Returns `None`; malformed-header T0814 (first occurrence per direction) |
| EC-002 | `data[1] == 0x04` (plausible off-by-one confusion with the Read Var function code, which is a parameter-block value, not a ROSCTR value) | Returns `None` — the ROSCTR field and function-code field are structurally distinct positions; no cross-field confusion is possible in a correct implementation |
| EC-003 | `data[1] == 0xFF` | Returns `None` |

## Canonical Test Vectors

| Input `data[1]` | Expected result | Category |
|---|---|---|
| `0x00` | `None` | reject: unrecognized ROSCTR |
| `0x04` | `None` | reject: unrecognized ROSCTR (not to be confused with FC 0x04) |
| `0xFF` | `None` | reject: unrecognized ROSCTR |
| `0x01` | `Some(...)` with `rosctr: Job` | accept — see BC-2.21.006 |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `parse_s7comm_header` returns `None` for exactly the 252 `u8` values not in `{0x01, 0x02, 0x03, 0x07}`, and `Some` for exactly those 4 (totality/exhaustiveness of the ROSCTR match) | proptest P1 (mirrors VP-046's `classify_frame_format` totality treatment) — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — safe-reject discipline for the ROSCTR field, directly analogous to CAP-20's protocol_id no-force-fit guarantee (BC-2.20.012) |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0814 (Denial of Service) — malformed-header anomaly signal only; emission wiring is a B2 responsibility |

## Related BCs

- BC-2.21.004 — composes with (shares the malformed-header dedup flag)
- BC-2.21.006 — composes with (the accept-path sibling for the 4 recognized ROSCTR values)
- BC-2.20.011 — composes with (the SS-20 `CotpTpduType` no-force-fit precedent this BC mirrors)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `fn parse_s7comm_header`, ROSCTR match arm with a `_ => return None` fallthrough

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — proptest P1, VP allocation pending.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core — proptest P1 target |
