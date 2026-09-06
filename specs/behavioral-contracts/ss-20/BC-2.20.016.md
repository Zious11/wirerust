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

# BC-2.20.016: Frozen `iso_on_tcp.rs` Module Boundary — Pure Free Functions Only, No `StreamAnalyzer` Impl, No Per-Flow State of Its Own

## Description

`src/analyzer/iso_on_tcp.rs` (SS-20) exports **only** the pure free functions
`parse_tpkt_header` and `parse_cotp_header`, plus the frozen `TpktHeader` /
`CotpTpduType` / `CotpHeader` types (ADR-014 Decision 1). It implements **no**
`StreamAnalyzer` trait, registers **no** `DispatchTarget::IsoOnTcp` dispatcher variant,
and owns **no** per-flow state of its own — the directional TPKT/COTP carry buffers
(BC-2.20.013/014) live on `S7commFlowState` (SS-21), not on any `IsoOnTcpFlowState`.
This BC formalizes the module-boundary contract that makes SS-20 a genuinely reusable
parsing library rather than a disguised second analyzer, and is the anchor point every
other BC-2.20.NNN contract's "frozen interface" language refers back to.

## Preconditions

1. `S7commAnalyzer` (SS-21) is the sole consumer of `iso_on_tcp.rs` in this cycle.

## Postconditions

1. `iso_on_tcp.rs` contains no `impl StreamAnalyzer for ...` block.
2. `dispatcher.rs`'s `DispatchTarget` enum gains exactly one new variant for this
   feature, `S7comm` (ADR-014 Decision 2 Rule 9) — there is no `DispatchTarget::IsoOnTcp`
   or equivalent.
3. `iso_on_tcp.rs` declares no struct analogous to `S7commFlowState` for its own
   bookkeeping; any state needed for reassembly (carry buffers) is a field on the
   *consumer's* flow-state struct (`S7commFlowState`, SS-21), populated and read by the
   consumer's `on_data`/`on_flow_close` methods, which call into `iso_on_tcp.rs`'s pure
   functions but do not delegate state ownership to it.
4. `S7commAnalyzer::on_data` calls `iso_on_tcp::parse_tpkt_header` then
   `iso_on_tcp::parse_cotp_header` on every extracted TPKT frame (ADR-014 Decision 1);
   this call sequence is the entire SS-20→SS-21 integration surface.
5. A future catalog-entry promotion (IEC 61850 MMS or ICCP/TASE.2, ADR-014 Decision 3
   critical caveat) can import `parse_tpkt_header`/`parse_cotp_header` directly with
   **zero** changes to `iso_on_tcp.rs` — its own analyzer would define its own flow-state
   carry buffers, analogous to but independent from `S7commFlowState`'s.

## Invariants

1. **Statelessness is architectural, not incidental**: ADR-014 Decision 1 states this
   placement explicitly resolves "F1 §2.3 open question" — the carry buffers were
   deliberately NOT placed in a hypothetical `IsoOnTcpFlowState` because doing so "would
   contradict [SS-20's] design and would need to be threaded through SS-21 regardless."
2. **No dispatcher coupling**: `iso_on_tcp.rs` never references `dispatcher::DispatchTarget`
   or any dispatcher type — the module has zero dependency on the dispatch layer,
   consistent with `protocols.rs`'s (CAP-18) documented pure-core-leaf discipline
   (BC-2.05... precedent; `protocols.rs` "must not depend on `dispatcher`").
3. **Single dispatcher rule for the whole port**: `classify()` gains exactly one new
   port-102 arm (Rule 9, `DispatchTarget::S7comm`) — never a per-protocol-identity rule,
   since the dispatcher cannot cheaply distinguish COTP protocol-IDs without performing
   the TPKT/COTP parse itself (ADR-014 Decision 2, "Alternatives Considered").

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A future MMS cycle wants to reuse `parse_tpkt_header`/`parse_cotp_header` | Imports them directly; defines its own `MmsFlowState.carry_c2s`/`carry_s2c` fields, structurally parallel to but independent of `S7commFlowState`'s — zero lines of `iso_on_tcp.rs` change |
| EC-002 | A code reviewer greps `iso_on_tcp.rs` for `impl StreamAnalyzer` | Zero matches — this is a regression-guard-style check for this BC's Postcondition 1 |
| EC-003 | `dispatcher.rs`'s VP-004 six-step atomic obligation (ADR-014 Decision 2) is applied | Only `DispatchTarget::S7comm` is added; no `DispatchTarget::IsoOnTcp` variant is ever introduced at any step |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|---------|
| Static analysis: `grep -c "impl StreamAnalyzer" src/analyzer/iso_on_tcp.rs` | `0` | architectural regression guard |
| Static analysis: `grep -c "IsoOnTcpFlowState" src/**/*.rs` | `0` (no such type exists anywhere in the tree) | architectural regression guard |
| `DispatchTarget` enum variant count added for this feature | Exactly 1 (`S7comm`) | happy-path: single dispatcher rule |

## Verification Properties

(No independent VP-NNN — this BC's postconditions are architectural/structural and are
verified by code-review inspection and the regression-guard greps above, not by a
runtime proof harness. It is anchored to the same VP-004 six-step atomic obligation
that governs `DispatchTarget::S7comm`'s dispatcher-level correctness, per ADR-014
Decision 2 — see the cross-subsystem dispatcher BC that part B / the INTEGRATE
sub-burst will author for `DispatchTarget::S7comm` Rule 9, mirroring BC-2.05.012's
IEC-104 precedent.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — this BC is the module-boundary contract that defines what CAP-20 *is* (a stateless parsing library) as distinct from what it is *not* (a second dispatcher-registered analyzer), which is the entire justification for splitting SS-20 out of SS-21 in the first place |
| L2 Domain Invariants | None directly (architectural/module-boundary invariant) |
| Architecture Module | SS-20 (`src/analyzer/iso_on_tcp.rs`, planned); SS-21 boundary (`S7commAnalyzer`, `S7commFlowState`, planned); `dispatcher.rs` Rule 9 |
| ADR | ADR-014 Decisions 1, 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — architectural contract, no runtime finding emission) |

## Related BCs

- BC-2.20.012 — composes with (the non-interpretation guarantee is one facet of this BC's broader statelessness/no-coupling contract)
- BC-2.20.013 — depends on (carry-buffer state placement on `S7commFlowState`, not an `IsoOnTcpFlowState`)
- BC-2.20.014 — depends on (same state-placement rule for the carry-overflow dedup flags)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` (planned) — module-level doc comment stating "pure free functions only, no `StreamAnalyzer` impl, no per-flow state" (per ADR-014 Decision 1's own frozen-interface code excerpt)
- `src/analyzer/dispatcher.rs` (planned amendment) — `DispatchTarget::S7comm` variant, Rule 9, VP-004 six-step atomic obligation (ADR-014 Decision 2)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 1` — "no `StreamAnalyzer` implementation of its own, and no per-flow state"; "Per-flow state placement (resolves F1 §2.3 open question)"
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 2` — single `DispatchTarget::S7comm`, no per-protocol-identity dispatcher rule

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None — architectural/structural contract verified by code-review and static-grep
regression guards, not a runtime proof harness. The dispatcher-level `DispatchTarget::S7comm`
correctness this BC references is anchored to VP-004, a pre-existing obligation extended
per ADR-014 Decision 2, not a new VP.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (this BC asserts the *absence* of module-owned state) |
| **Deterministic** | n/a — structural/architectural contract |
| **Thread safety** | n/a |
| **Overall classification** | architectural boundary contract |
