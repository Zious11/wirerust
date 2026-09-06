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

# BC-2.20.012: `parse_cotp_header`'s `protocol_id` Is Extracted Verbatim, Never Interpreted (Frozen SS-20→SS-21 Boundary)

## Description

This BC formalizes the frozen-interface non-interpretation guarantee already implied by
BC-2.20.009: `parse_cotp_header` extracts the protocol-ID byte from a non-empty DT
payload as a raw `u8` and performs **no** comparison, matching, or branching on its
value. `0x32` (classic S7comm), `0x72` (S7comm-plus), and every other possible `u8`
value are treated identically by SS-20 — the disambiguation table that assigns meaning
to these values (ADR-014 Decision 2) lives entirely in `S7commAnalyzer` (SS-21). This is
the load-bearing correctness boundary that makes CAP-20 reusable by a future IEC 61850
MMS or ICCP/TASE.2 cycle without modification: SS-20 has no S7comm-specific knowledge
anywhere in its parsing logic.

## Preconditions

1. `parse_cotp_header` reaches the DT-with-non-empty-payload branch (BC-2.20.009
   preconditions hold).

## Postconditions

1. For any `u8` value `b` at `tpkt_payload[payload_offset]`, `parse_cotp_header`
   returns `protocol_id: Some(b)` — the mapping from input byte to output value is the
   identity function, total over all 256 possible `u8` values.
2. No branch, match arm, or conditional inside `parse_cotp_header` inspects whether `b`
   equals `0x32`, `0x72`, or any other specific value.
3. `iso_on_tcp.rs` contains no reference to the literals `0x32` or `0x72`, nor to the
   strings "S7comm" or "S7comm-plus", anywhere in its parsing logic (doc comments
   describing the frozen struct fields, per ADR-014 Decision 1's own code excerpt, are
   documentation, not parsing logic, and are exempted from this constraint).
4. `S7commAnalyzer` (SS-21) is solely responsible for interpreting the returned
   `protocol_id` per ADR-014 Decision 2's four-row disambiguation table.

## Invariants

1. **Module boundary purity**: this is an architectural invariant, not just a
   behavioral one — it is what makes the "build once, benefit three times" claim
   (ADR-014 Decision 1 rationale) actually true rather than aspirational.
2. **Identity extraction is total**: unlike the TPDU-type match (BC-2.20.007..011),
   which partitions `tpkt_payload[1]`'s high nibble into 4 outcomes, the protocol-ID
   byte extraction has exactly 1 outcome shape (`Some(b)`) for all 256 values of `b`
   once the DT-non-empty-payload precondition holds.
3. **Testable via code-inspection regression guard**: a future maintenance sweep or
   adversarial pass can grep `src/analyzer/iso_on_tcp.rs` for the literals `0x32`/`0x72`
   as a cheap regression check for this invariant.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `protocol_id byte == 0x32` | `Some(0x32)` — no special-cased fast path or eager S7comm-specific validation |
| EC-002 | `protocol_id byte == 0x72` | `Some(0x72)` — no special-cased fast path |
| EC-003 | `protocol_id byte == 0xFF` (maximum `u8`, no known protocol association) | `Some(0xFF)` — identical code path to `0x32`/`0x72` |
| EC-004 | `protocol_id byte == 0x00` | `Some(0x00)` — identical code path |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|---------|
| Protocol-ID byte swept over all 256 `u8` values with an otherwise-fixed valid DT COTP header | `protocol_id == Some(byte)` for every value, with identical `tpdu_type`/`payload_offset` across the sweep | totality: identity extraction |
| Static analysis: `grep -c '0x32\|0x72' src/analyzer/iso_on_tcp.rs` (parsing logic only, excluding doc comments) | `0` | architectural regression guard |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `parse_cotp_header`'s protocol-ID extraction is the identity function over all 256 `u8` values — no value-dependent branching | proptest P1 (protocol-ID branch totality per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC is the architectural correctness property that makes CAP-20 a genuinely reusable, protocol-agnostic capability rather than an S7comm-specific parser masquerading as one |
| L2 Domain Invariants | None directly (architectural/module-boundary invariant; no brownfield domain invariant applies) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); ADR-014 Decisions 1, 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure extraction function) |

## Related BCs

- BC-2.20.009 — composes with (this BC formalizes a property of BC-2.20.009's postcondition as its own contract)
- BC-2.20.016 — composes with (frozen module boundary: no `StreamAnalyzer` impl, no per-flow state, and — per this BC — no S7comm-specific interpretation either)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — `fn parse_cotp_header`: `protocol_id: Some(tpkt_payload[payload_offset])` — a direct byte read, no match/comparison
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — "a future MMS or ICCP cycle imports `iso_on_tcp::parse_tpkt_header`/`iso_on_tcp::parse_cotp_header` directly, touching zero lines of `s7comm.rs`"
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — the four-row disambiguation table that consumes this BC's output, entirely inside `S7commAnalyzer` (SS-21)

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
| **Overall classification** | pure core |
