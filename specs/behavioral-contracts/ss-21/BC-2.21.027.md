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

# BC-2.21.027: DT Frame With `protocol_id: Some(other)` (Neither 0x32 Nor 0x72) Left Unclassified — Never Force-Fit to S7comm (MMS/ICCP/Unrecognized)

## Description

Per ADR-014 Decision 2's disambiguation table, this is the load-bearing correctness
row this ADR explicitly names: "a COTP DT-TPDU on port 102 whose protocol-ID is not
`0x32` or `0x72` must never be misattributed to S7comm." When `protocol_id` is
`Some(other)` for `other ∉ {0x32, 0x72}` — most plausibly IEC 61850 MMS, ICCP/TASE.2,
or any other ISO-on-TCP traffic sharing port 102 — this BC guarantees the frame is left
entirely unclassified: no `S7Protocol` variant is assigned, no S7comm-classic or
S7comm-plus dissection path is entered, and the flow continues to surface through the
existing `(TransportProto, u16)` unclassified-port-count mechanism
(`dispatcher.rs::unclassified_port_counts`) rather than being silently absorbed into
S7comm's counted traffic.

## Preconditions

1. `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: Some(other), .. })` where `other ∉ {0x32, 0x72}` (BC-2.21.002 Postcondition 5, first
   sub-case).

## Postconditions

1. `S7commFlowState.classified_protocol` is set to `Some(S7Protocol::Unclassified)` —
   a distinct variant from both `Classic` and `Plus`, never left as an ambiguous
   `None` once a DT frame has actually been observed and inspected (distinguishing
   "never saw a DT frame yet" from "saw one, but it wasn't S7comm").
2. No bytes beyond `protocol_id` are read or interpreted for this frame.
3. This flow's traffic is **not** counted toward S7comm's `Support::Supported`
   coverage in any report — it remains visible via the existing unclassified-port
   mechanism, preserving the correctness property that `protocols.rs`'s
   `known-supported`/`known-unsupported` catalog partition (ADR-014 Decision 3) is
   never silently inflated by non-S7comm port-102 traffic.
4. Once a flow's `classified_protocol` is set to `Some(S7Protocol::Unclassified)`
   (first-classification-wins, BC-2.21.001/002), it remains so for the flow's
   lifetime even if a later frame on the same flow happens to carry `protocol_id:
   Some(0x32)` or `Some(0x72)` — sticky-first-classification applies uniformly across
   all three outcomes (Classic, Plus, Unclassified), per BC-2.21.002 Postcondition 6.

## Invariants

1. **No force-fit to S7comm, ever**: this is the single correctness property ADR-014
   calls out by name as "the load-bearing correctness property this ADR must
   guarantee" — a defect here is a direct violation of the ADR's stated purpose.
2. **Distinct from "never observed a DT frame"**: `S7Protocol::Unclassified` (this BC)
   and `classified_protocol: None` (a flow that has only exchanged CR/CC so far,
   BC-2.21.001 Edge Case EC-001-adjacent) are different states — the former means "we
   looked, and it's not S7comm"; the latter means "we haven't looked yet."
3. **No S7comm-plus-shaped guess for an unrecognized byte close to `0x72`**: e.g.
   `protocol_id: Some(0x73)` is `Unclassified`, not treated as a "probably S7comm-plus
   typo/variant" — there is no proximity-based fallback anywhere in this dispatch.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `protocol_id: Some(0x00)` (a value with no known ISO-on-TCP protocol association) | `Unclassified` |
| EC-002 | `protocol_id: Some(0x73)` (adjacent to the recognized `0x72`) | `Unclassified` — no proximity guess |
| EC-003 | A flow's traffic is, in ground truth, IEC 61850 MMS (which the pcap fixture author knows but the analyzer cannot) | `Unclassified` — MMS/ICCP dissection is out of scope for this entire feature (ADR-014 Decision 10); the analyzer correctly does not attempt to identify *which* other protocol it is, only that it is not S7comm |

## Canonical Test Vectors

| `protocol_id` | Expected `classified_protocol` | Category |
|---|---|---|
| `Some(0x00)` | `Unclassified` | reject: unrecognized protocol-ID, no force-fit |
| `Some(0x73)` | `Unclassified` | reject: adjacent-value, no proximity guess |
| `Some(0xE0)` (a value observed in some real MMS/ICCP prior-art discussions) | `Unclassified` | reject: plausible-MMS byte, still no force-fit |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| For every `u8` value `b ∉ {0x32, 0x72}`, `classified_protocol` is `Some(Unclassified)`, never `Some(Classic)` or `Some(Plus)` — totality/exhaustiveness of the protocol_id-to-S7Protocol mapping over all 254 remaining `u8` values | proptest P1 (mirrors VP-046's totality treatment; this is the single highest-value correctness proof in this feature per ADR-014's own framing) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21's description table names this exact row ("`Some(other)` / unparseable... Left unclassified — never misattributed to S7comm") verbatim |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — this BC ensures port-102 fallback dispatch does not silently over-attribute non-S7comm traffic once inside the S7comm-classified flow) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 2 — "the load-bearing correctness property this ADR must guarantee" |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — unclassified traffic produces no S7comm-attributed finding of any kind) |

## Related BCs

- BC-2.21.002 — depends on (the dispatch branch that reaches this outcome)
- BC-2.21.028 — composes with (the sibling unclassified-gap path for unparseable COTP payloads)
- BC-2.21.024 — composes with (contrasted: the `0x72` branch that IS given a name, `Plus`, unlike this branch's `other` values)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7Protocol::Unclassified` match arm in `on_data`
- `src/analyzer/dispatcher.rs` (planned amendment) — `unclassified_port_counts` mechanism (pre-existing, unmodified) as the visibility path for this traffic
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — the four-row disambiguation table, row 4

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — proptest P1, anticipated VP-048 range; the highest-priority correctness proof in this BC set per ADR-014's own emphasis.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | updates `S7commFlowState.classified_protocol` (per-flow, not global) |
| **Deterministic** | yes |
| **Thread safety** | single-flow-owner access pattern |
| **Overall classification** | effectful shell (classification-state update) |
