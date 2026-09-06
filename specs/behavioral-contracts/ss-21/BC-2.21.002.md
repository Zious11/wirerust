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

# BC-2.21.002: `S7commAnalyzer::on_data` Four-Way Dispatch on `CotpHeader::protocol_id`

## Description

`S7commAnalyzer::on_data` is the SS-21 entry point that drives the frame-walk loop
defined at the SS-20 boundary (BC-2.20.013): for every complete TPKT frame extracted,
it calls `iso_on_tcp::parse_tpkt_header` then `iso_on_tcp::parse_cotp_header`, then
branches on the returned `CotpHeader::protocol_id` per ADR-014 Decision 2's four-row
disambiguation table. This BC formalizes that dispatch as the single integration point
between SS-20's frame extraction and SS-21's protocol-specific dissection — every
other BC-2.21.NNN classification/parsing contract is reached through exactly one of
this BC's four branches.

## Preconditions

1. A complete TPKT frame has been extracted by the frame-walk loop (BC-2.20.013), and
   `parse_cotp_header` has been called on its COTP payload.
2. `S7commFlowState` exists for the flow (created lazily per BC-2.21.001 if this is the
   first `on_data` call).

## Postconditions

1. If `parse_cotp_header` returns `None` (BC-2.20.011, unrecognized TPDU type or
   truncated-beyond-carry-repair): the frame is routed to the unclassified-gap path
   (BC-2.21.028) — never force-fit to any of the three recognized branches.
2. If `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: ConnectRequest | ConnectConfirm, .. })`:
   `S7commFlowState.session_established` is updated per BC-2.21.001 Postcondition 1;
   no protocol classification occurs (classification is deferred to the first DT
   frame, per ADR-014 Decision 2 row 3).
3. If `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: Some(0x32), .. })`:
   dispatch to classic S7comm dissection — `parse_s7comm_header` is called on the
   slice beginning at `payload_offset` (BC-2.21.004 onward).
4. If `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: Some(0x72), .. })`:
   dispatch to the S7comm-plus framing-only path (BC-2.21.024/025/026).
5. If `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: Some(other), .. })`
   where `other ∉ {0x32, 0x72}`, or `protocol_id: None` on a DT frame with an empty
   payload (BC-2.20.010): dispatch to the unclassified-gap path (BC-2.21.027) —
   identical treatment to Postcondition 1's `None`-from-`parse_cotp_header` case for
   attribution purposes (neither is ever counted as S7comm).
6. On the first DT frame observed for a flow (any `protocol_id` value, including
   `None`), `S7commFlowState.classified_protocol` is set exactly once
   (first-classification-wins, BC-2.21.001 Edge Case EC-002); subsequent DT frames on
   the same flow do not overwrite it even if their `protocol_id` differs.

## Invariants

1. **Exhaustive four-way branch**: every possible `parse_cotp_header` return value
   (`None`; `Some` with `tpdu_type` ∈ {ConnectRequest, ConnectConfirm}; `Some` with
   `tpdu_type: DataTransfer` and `protocol_id` ∈ {`Some(0x32)`, `Some(0x72)`,
   `Some(other)`, `None`}) is routed to exactly one of the branches above — no branch
   is reachable from more than one input class, and no input class reaches zero
   branches.
2. **Load-bearing correctness (ADR-014 Decision 2)**: this dispatch is the single
   location in wirerust's binary-ICS analyzer family where post-classification
   disambiguation determines *which named protocol* a flow is attributed to, not
   merely whether the flow is malformed. A defect here can misattribute non-S7comm
   traffic (MMS, ICCP, unrecognized) to S7comm — the correctness property this BC and
   BC-2.21.027/028 jointly guarantee never occurs.
3. **No re-dispatch on protocol change**: per Postcondition 6, a flow's classification
   is sticky from its first DT frame; this is a deliberate simplicity choice, not an
   oversight — flagged for B2's consideration as a possible anomaly signal, not
   re-classified by B1.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A flow observes only CR/CC frames, never a DT frame, before the pcap ends | `classified_protocol` remains `None` for the lifetime of the flow; no dissection of any kind occurs; the flow is simply "S7comm-port-102, session tracked, never classified" |
| EC-002 | A flow's very first observed frame is a DT frame with `protocol_id: Some(0x32)` (no prior CR/CC observed, e.g. mid-capture start) | Classification proceeds normally from the DT frame alone; `session_established` remains `false` (no CR/CC observed) but this does not block classic S7comm dissection |
| EC-003 | Two DT frames arrive back-to-back within a single `on_data` call (multiple frames per delivery, mirrors BC-2.20.013's multi-frame walk) | Each frame is dispatched independently through this BC's branches; `classified_protocol`'s first-write-wins rule applies across the pair in arrival order |

## Canonical Test Vectors

| `parse_cotp_header` result | Dispatch outcome | Category |
|---|---|---|
| `None` | Unclassified gap (BC-2.21.028) | reject: unparseable COTP payload |
| `Some(CotpHeader{tpdu_type: ConnectRequest, protocol_id: None, ..})` | Session tracking only, no classification | happy-path: session establishment |
| `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: Some(0x32), ..})` | Classic S7comm dissection entry (BC-2.21.004) | happy-path: classic dispatch |
| `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: Some(0x72), ..})` | S7comm-plus framing-only path (BC-2.21.024) | happy-path: plus dispatch |
| `Some(CotpHeader{tpdu_type: DataTransfer, protocol_id: Some(0x00), ..})` | Unclassified gap (BC-2.21.027) | reject: neither classic nor plus |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| The four-way dispatch is total and non-overlapping over every possible `Option<CotpHeader>` value reachable from `parse_cotp_header` — no input value reaches zero or more than one branch | proptest P1 (mirrors VP-046's `classify_frame_format` totality treatment) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this BC is the dispatch surface CAP-21's description explicitly names as the analyzer's core disambiguation behavior |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — this dispatch fires only inside a flow already routed to `DispatchTarget::S7comm` via port-102 fallback; it does not itself perform content-first classification, it consumes its result) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); ADR-014 Decision 2 |
| ADR | ADR-014 Decision 2 (in-analyzer disambiguation, load-bearing) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — dispatch contract only; individual classification branches carry no finding emission in this part; B2 authors emission) |

## Related BCs

- BC-2.20.009 — depends on (`protocol_id` extraction this BC branches on)
- BC-2.20.011 — depends on (the `None` case this BC's Postcondition 1 handles)
- BC-2.21.001 — depends on (`S7commFlowState` fields this BC reads/writes)
- BC-2.21.004 — composes with (classic S7comm dissection entry point)
- BC-2.21.024 — composes with (S7comm-plus framing-only entry point)
- BC-2.21.027 — composes with (unrecognized-protocol_id unclassified-gap path)
- BC-2.21.028 — composes with (unparseable-COTP-payload unclassified-gap path)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `impl StreamAnalyzer for S7commAnalyzer { fn on_data(...) }`, the four-way `match` on `CotpHeader`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — the frozen four-row disambiguation table

## Story Anchor

STORY-187 (also a formal-hardening re-verification anchor for STORY-194)

## VP Anchors

- VP-053 (proptest P0) — `protocol_id` Four-Way Dispatch Totality and Unclassified
  Never-Force-Fit; registered F2 INTEGRATE sub-burst per VP-INDEX.md v2.48; traces
  BC-2.21.002, BC-2.21.027, BC-2.21.028 (this is the BC's protocol_id-dispatch
  concern, superseding the earlier "anticipated VP-048 range" speculation)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads/writes `S7commFlowState` (per-flow, not global) |
| **Deterministic** | yes — same byte sequence and prior flow state always produce the same dispatch outcome |
| **Thread safety** | single-flow-owner access pattern (mirrors sibling analyzers) |
| **Overall classification** | effectful shell (flow-state mutation) around pure SS-20 parse calls |
