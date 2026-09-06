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

# BC-2.21.001: `S7commFlowState` Owns TPKT/COTP Carry Buffers, S7comm Classification State, and Per-Direction Dedup Flags

## Description

`S7commFlowState` (`src/analyzer/s7comm.rs`, SS-21) is the per-flow bookkeeping struct
for a TCP/102 flow classified `DispatchTarget::S7comm`. Per ADR-014 Decision 1, SS-20
(`iso_on_tcp.rs`) is deliberately stateless — the directional TPKT/COTP carry buffers
it requires (BC-2.20.013/014) are fields on this struct, not on a separate
`IsoOnTcpFlowState`. This BC is the SS-21 counterpart to BC-2.20.016: it defines the
concrete field set `S7commFlowState` carries, establishing the single source of truth
every other BC-2.21.NNN contract's flow-state references point back to.

## Preconditions

1. A TCP flow has been classified `DispatchTarget::S7comm` (port 102, Rule 9, ADR-014
   Decision 2) and `S7commAnalyzer::on_data` has been called at least once for it.

## Postconditions

1. `S7commFlowState` contains, at minimum, the following fields:
   - `carry_c2s: Vec<u8>`, `carry_s2c: Vec<u8>` — directional TPKT frame-reassembly
     carry buffers (BC-2.20.013), bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`
     (BC-2.20.014).
   - `carry_overflow_reported_c2s: bool`, `carry_overflow_reported_s2c: bool` —
     per-direction dedup flags for the carry-overflow T0814 finding (BC-2.20.014).
   - `session_established: bool` — set when a COTP CR (BC-2.20.007) is followed by a
     matching CC (BC-2.20.008) on this flow; classification of the upper-layer
     protocol is deferred until the first DT frame regardless of this flag's value.
   - `classified_protocol: Option<S7Protocol>` — set on the first DT frame with a
     non-`None` `protocol_id` (BC-2.21.002); `S7Protocol` distinguishes `Classic`,
     `Plus`, and `Unclassified` (BC-2.21.027/028); remains `None` until the first DT
     frame is observed.
   - `malformed_header_reported_c2s: bool`, `malformed_header_reported_s2c: bool` —
     per-direction dedup flags for S7comm-header-level bounds/truncation rejects
     (BC-2.21.004/007/008/009), distinct from SS-20's carry-overflow dedup flags
     (Invariant 2).
2. No field on `S7commFlowState` duplicates a field SS-20 owns — the carry buffers
   listed above are the *only* SS-20-originated state; everything else is
   S7comm-specific.
3. `S7commFlowState` is created lazily on the first `on_data` call for a newly
   classified flow and stored in the analyzer's per-flow map, keyed by `FlowKey`
   (mirrors `Iec104FlowState`/`DnpFlowState` precedent).

## Invariants

1. **Single state owner**: exactly one `S7commFlowState` exists per classified flow;
   no shadow or duplicate state struct exists elsewhere in SS-21.
2. **Dedup-flag separation**: carry-overflow dedup (SS-20-originated, BC-2.20.014) and
   malformed-S7comm-header dedup (SS-21-originated, this BC) are tracked by distinct
   flags, mirroring the IEC-104 precedent (BC-2.19.026 Invariant 5) of never
   conflating anomaly classes under one suppression flag.
3. **No S7comm-plus-specific decode state**: per ADR-014 Decision 6, `S7commFlowState`
   does not carry any field implying function-code-level S7comm-plus state (e.g., no
   `plus_last_function` field) — only `classified_protocol` and the bounded
   session-setup-metadata fields defined in BC-2.21.025.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A flow is classified `DispatchTarget::S7comm` but never sends any bytes before close | `S7commFlowState` is never created (lazy-on-first-`on_data`); `on_flow_close` is a no-op for this flow |
| EC-002 | The same flow observes a DT frame, then later another DT frame with a *different* `protocol_id` value than the first | `classified_protocol` is set only once, from the *first* DT frame observed (first-classification-wins); subsequent DT frames with a different `protocol_id` do not overwrite it — flagged as a B2 (MITRE emission) anomaly-detection concern, not a B1 dissection concern |
| EC-003 | `carry_overflow_reported_c2s` and `malformed_header_reported_c2s` are both set to `true` on the same flow direction | Both flags coexist independently; each governs only its own anomaly class's dedup, per Invariant 2 |

## Canonical Test Vectors

| Scenario | Expected `S7commFlowState` state | Category |
|----------|-----------------------------------|---------|
| First `on_data` call for a newly classified flow, zero bytes delivered | Struct created with all `Vec` fields empty, all `bool` fields `false`, `classified_protocol: None` | happy-path: initialization |
| CR observed, then CC observed on the same flow | `session_established == true`, `classified_protocol` still `None` | happy-path: session tracking without classification |
| First DT frame with `protocol_id: Some(0x32)` observed | `classified_protocol == Some(S7Protocol::Classic)` | happy-path: classic classification |

## Verification Properties

(No independent VP-NNN — this BC is a structural/architectural contract, verified by
code-review inspection of the struct definition and field usage, mirroring
BC-2.20.016's treatment. Runtime behaviors of individual fields are exercised by the
proptest/cargo-fuzz harnesses anchored to the specific BCs that mutate each field.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this BC defines the per-flow state struct that is the load-bearing data structure for every other CAP-21 dissection behavior |
| L2 Domain Invariants | None directly (architectural/structural state-ownership contract; carry-buffer fields are governed by SS-20's INV-2-adjacent framing, not a distinct domain invariant of their own) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); ADR-014 Decision 1 (per-flow state placement ruling) |
| ADR | ADR-014 Decisions 1, 8 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — structural contract, no finding emission) |

## Related BCs

- BC-2.20.013 — depends on (carry-buffer fields defined at the SS-20 boundary, hosted here per ADR-014 Decision 1)
- BC-2.20.014 — depends on (carry-overflow dedup flags hosted here)
- BC-2.20.016 — composes with (SS-20's module-boundary contract; this BC is its SS-21-side counterpart)
- BC-2.21.002 — composes with (`on_data` reads/writes this struct's fields on every call)
- BC-2.21.003 — composes with (`on_flow_close` removes this struct)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `struct S7commFlowState { ... }` field definition
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — "Per-flow state placement (resolves F1 §2.3 open question)"
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 8` — carry-buffer sizing and dedup-flag placement

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None — structural/architectural contract.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (per-flow state, not global) |
| **Deterministic** | n/a — structural data-definition contract |
| **Thread safety** | n/a (single-flow-owner access pattern, mirrors sibling analyzers) |
| **Overall classification** | architectural boundary contract |
