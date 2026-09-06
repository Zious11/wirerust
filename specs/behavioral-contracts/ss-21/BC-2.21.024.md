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

# BC-2.21.024: S7comm-plus DT Frame (`protocol_id: Some(0x72)`) Classified as Observed Session — Framing-Level Only, No Function-Code Decode

## Description

Per ADR-014 Decision 6, S7comm-plus (`protocol_id == Some(0x72)`) is "observed, not
dissected": a COTP DT-TPDU with `protocol_id: Some(0x72)` is counted and reported as an
observed S7comm-plus session, but **no** S7comm-plus function-code catalog, object/
service dissection, or `S7commPlusAnalyzer` exists. This BC defines the exact
boundary of what B1 does for a `0x72` frame: classify it as `S7Protocol::Plus`
(BC-2.21.001's `classified_protocol` field) and increment an observation counter; no
attempt is made to interpret any byte beyond `protocol_id` itself as an S7comm-plus
opcode, object ID, or service field, **except** the bounded session-setup metadata
observation defined separately in BC-2.21.025.

## Preconditions

1. `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: Some(0x72), .. })` (BC-2.21.002 Postcondition 4 branch).

## Postconditions

1. `S7commFlowState.classified_protocol` is set to `Some(S7Protocol::Plus)` if not
   already set (first-classification-wins, BC-2.21.001).
2. The frame contributes to an "observed S7comm-plus session" count/report entry — it
   does **not** register the flow as `known-supported` in `protocols.rs` (ADR-014
   Decision 3; S7comm-plus remains `Support::DetectionOnly`).
3. No bytes beyond `protocol_id` are interpreted as an S7comm-plus function code,
   object ID, integrity field, or any other semantic structure — the classic
   `S7ClassicFunction` classification surface (BC-2.21.010-023) is never applied to a
   `0x72` frame.
4. This classification is applied identically for every `0x72` DT frame on the flow,
   not only the first — each subsequent `0x72` frame reaffirms/continues the observed
   session, it does not require re-classification logic beyond BC-2.21.002's
   sticky-first-classification rule.

## Invariants

1. **No `S7commPlusAnalyzer`, ever, this cycle**: this BC's scope boundary is a direct
   restatement of ADR-014 Decision 6's explicit non-goal — a future feature cycle
   could add one, but this feature never does.
2. **`DetectionOnly`, not `Supported`**: the catalog-level consequence (SS-18,
   `protocols.rs::Support::DetectionOnly`) is asymmetric with classic S7comm's
   `Supported` status — this BC's framing-only classification is what that catalog
   entry's semantics describe operationally.
3. **Byte-boundary discipline**: this BC's "no interpretation beyond `protocol_id`"
   guarantee is the S7comm-plus analogue of BC-2.20.012's protocol_id
   non-interpretation guarantee at the SS-20 layer — each layer commits to not
   over-reaching into the next.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A flow observes 50 consecutive `0x72` DT frames (a long-running S7comm-plus session, e.g. TIA Portal engineering traffic) | All 50 classified `S7Protocol::Plus`; no per-frame function-code decode attempted for any of them |
| EC-002 | A `0x72` frame's payload happens to contain bytes that would resemble a classic S7comm FC value at some offset (coincidental byte pattern) | Never interpreted as such — the classic FC classification surface (BC-2.21.010 onward) is structurally unreachable from the `0x72` branch of BC-2.21.002's dispatch |

## Canonical Test Vectors

| `protocol_id` | Expected outcome | Category |
|---|---|---|
| `Some(0x72)` | `classified_protocol: Some(Plus)`; observed-session count incremented; no FC decode | happy-path: framing-only classification |

## Verification Properties

(No independent VP-NNN — this BC's boundary is verified by a regression-guard test
asserting no `S7ClassicFunction` classification call is ever reached from a `0x72`
dispatch branch, mirroring BC-2.20.016's static-grep regression-guard style.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21's description names S7comm-plus's "framing-level classification + unencrypted session-setup metadata only" as an explicit consumer-analyzer behavior |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 6 ("observed, not dissected"), Decision 3 (`Support::DetectionOnly`) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — framing-level observation only; no function-code-level evidence exists for a `0x72` frame, so no MITRE technique can be defensibly emitted from this classification) |

## Related BCs

- BC-2.21.002 — depends on (the dispatch branch that reaches this classification)
- BC-2.21.025 — composes with (the bounded session-setup metadata extension to this framing-only scope)
- BC-2.21.026 — composes with (the TLS-upgrade deferral boundary)
- BC-2.20.012 — composes with (the SS-20 non-interpretation precedent this BC mirrors at the SS-21 layer)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7Protocol::Plus` branch in `on_data`, observed-session counter
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 6` — S7comm-plus scope boundary

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated — regression-guard test, not a proof-harness candidate.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | updates `S7commFlowState.classified_protocol` and an observation counter (per-flow/per-analyzer, not global) |
| **Deterministic** | yes |
| **Thread safety** | single-flow-owner access pattern |
| **Overall classification** | effectful shell (classification-state update) |
