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

# BC-2.21.028: Unparseable COTP DT Payload Reaching `S7commAnalyzer` Receives the Same Unclassified-Gap Treatment as an Unrecognized `protocol_id`

## Description

When `parse_cotp_header` returns `None` for a DT-shaped byte sequence it cannot
otherwise parse (BC-2.20.011's unrecognized-TPDU-type reject, or a COTP-layer
truncation the carry buffer could not repair before an overflow/resync event,
BC-2.20.014/015), `S7commAnalyzer` treats the resulting gap identically to
BC-2.21.027's `protocol_id: Some(other)` case for attribution purposes: the frame
contributes to the unclassified-port count, never to S7comm's counted traffic, and
`S7commFlowState.classified_protocol` is set to `Some(S7Protocol::Unclassified)` if not
already classified. This BC exists to close a distinct-but-adjacent gap from
BC-2.21.027: a `None` from `parse_cotp_header` carries no `protocol_id` at all to
inspect (unlike the `Some(other)` case), so it needs its own explicit contract rather
than being assumed to fall out of BC-2.21.027's logic by inspection.

## Preconditions

1. `parse_cotp_header(tpkt_payload)` returns `None` for a frame that was not itself a
   TPKT-level reject (BC-2.20.001/002/003 — those never reach SS-21 at all, being
   stashed to carry at the SS-20 boundary) — i.e., a complete TPKT frame was extracted,
   but its COTP payload could not be parsed into a recognized `CotpHeader` (BC-2.20.011).

## Postconditions

1. `S7commFlowState.classified_protocol` is set to `Some(S7Protocol::Unclassified)` if
   not already set (first-classification-wins, shared with BC-2.21.027).
2. The frame is never counted as S7comm traffic in any report.
3. No bytes beyond what `parse_cotp_header` itself already inspected (and rejected) are
   further interpreted by SS-21 — this BC adds no new byte-reading behavior of its
   own; it only specifies the classification consequence of SS-20's existing reject
   outcome.
4. This outcome is indistinguishable, from the perspective of any report or finding
   surfaced to a wirerust user, from BC-2.21.027's `Some(other)` outcome — both are
   "this port-102 traffic is not S7comm" with no further distinction drawn between
   "we saw a protocol-ID byte we don't recognize" and "we couldn't even parse the COTP
   frame structure."

## Invariants

1. **Unified unclassified-gap semantics**: BC-2.21.027 and this BC are two input
   conditions (`Some(other)` vs. `None`-from-`parse_cotp_header`) mapping to one
   observable outcome (`Unclassified`, uncounted) — consistent with ADR-014 Decision
   2's framing of these as jointly the same "load-bearing correctness property" row.
2. **No re-attempt or fallback parse**: SS-21 does not attempt any secondary,
   more-lenient parse of a payload SS-20 already rejected — a `None` from
   `parse_cotp_header` is final for that frame.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A COTP frame with a TPDU-code high nibble the analyzer doesn't recognize (e.g. `0x8` Disconnect Request, per BC-2.20.011's enumerated non-CR/CC/DT list) | `parse_cotp_header` returns `None`; `Unclassified`, uncounted |
| EC-002 | A COTP frame that survives the carry-overflow/resync cycle (BC-2.20.014/015) and is subsequently re-walked successfully | The *resynced* frame, once successfully parsed, is classified normally per BC-2.21.002/024/027; only the *discarded* pre-resync bytes fall under this BC's `None` treatment |

## Canonical Test Vectors

| `parse_cotp_header` outcome | Expected `classified_protocol` | Category |
|---|---|---|
| `None` (unrecognized TPDU-type high nibble) | `Unclassified` | reject: unparseable COTP payload |
| `None` (post-carry-overflow discarded fragment) | `Unclassified` | reject: discarded malformed fragment |

## Verification Properties

(No independent VP-NNN — covered by the same proptest P1 totality obligation anchored
to BC-2.21.027, since this BC and that one jointly define the complete unclassified-gap
outcome space.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — completes CAP-21's "`Some(other)` / unparseable" description row alongside BC-2.21.027 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — shared with BC-2.21.027) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none) |

## Related BCs

- BC-2.21.027 — composes with (the joint definition of the complete unclassified-gap outcome space)
- BC-2.20.011 — depends on (the SS-20 reject condition this BC's classification consequence responds to)
- BC-2.20.014 — depends on (the carry-overflow/resync path that can also produce a discarded, unparseable fragment)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7Protocol::Unclassified` match arm, `None`-from-`parse_cotp_header` branch (shared code path with BC-2.21.027's `Some(other)` branch, differing only in the input condition)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — the four-row disambiguation table, row 1

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — shares BC-2.21.027's proptest P1 obligation.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | updates `S7commFlowState.classified_protocol` (per-flow, not global) |
| **Deterministic** | yes |
| **Thread safety** | single-flow-owner access pattern |
| **Overall classification** | effectful shell (classification-state update) |
