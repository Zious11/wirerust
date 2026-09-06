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

# BC-2.21.029: Download-Session Correlation State Machine on `S7commFlowState`, With Block-Type Hint Capture

## Description

Part B1 classifies `RequestDownload` (`0x1A`), `DownloadBlock` (`0x1B`), and `DownloadEnded`
(`0x1C`) independently per-frame (BC-2.21.013) and explicitly defers "correlating a
`RequestDownload → DownloadBlock (× N) → DownloadEnded` sequence on a single flow into one
logical 'download session'" to part B2 (BC-2.21.013 Postcondition 5). This BC is that deferred
state machine: it adds a `download_state: S7DownloadSessionState` field to `S7commFlowState`
that tracks the lifecycle of a download session per-flow, and — because ADR-014 Decision 5's
T0821 (Modify Controller Tasking) detection pattern needs the transferred block's type — this
BC also defines the `S7BlockTypeHint` decode captured at `RequestDownload` time and carried
through to session completion. This BC has **no MITRE emission surface of its own**; BC-2.21.030
(T0843), BC-2.21.031 (T0889), and BC-2.21.032 (T0821) all consume the `Completed` transition
this BC produces.

## Preconditions

1. A frame on the flow has been classified by part B1's Job/Ack_Data function-code match
   (BC-2.21.010 through BC-2.21.017).

## Postconditions

**State shape:**

1. `S7commFlowState.download_state: S7DownloadSessionState` where:
   ```
   enum S7DownloadSessionState {
       Idle,
       InProgress { blocks_seen: u32, block_type_hint: S7BlockTypeHint },
       Completed { blocks_seen: u32, block_type_hint: S7BlockTypeHint },
   }
   ```
2. `S7BlockTypeHint` (NEW type, authored in this BC — not a B1 type):
   ```
   enum S7BlockTypeHint {
       OrganizationBlock,        // decoded 2-char block-type code == "OB"
       OtherBlockType(String),   // decoded but not "OB" (e.g. "DB", "FB", "FC", "SB", "SF", "SD")
       Undeterminable,           // filename field absent, truncated, or unparseable
   }
   ```

**Transitions (evaluated in the order a frame is classified, after B1's per-frame FC match):**

3. `RequestDownload` observed while `Idle`: transitions to `InProgress { blocks_seen: 0,
   block_type_hint }`, where `block_type_hint` is decoded from the Request Download parameter
   block's filename-style field per Invariant 2 below.
4. `RequestDownload` observed while `InProgress { .. }` or `Completed { .. }`: the prior session
   (complete or incomplete) is abandoned without emission and a NEW session starts —
   `download_state` resets to `InProgress { blocks_seen: 0, block_type_hint }` for the new
   `RequestDownload`'s own decoded hint. An abandoned incomplete session (a `RequestDownload`
   with no `DownloadEnded`, superseded by a fresh `RequestDownload`) never retroactively emits
   T0843/T0889/T0821 for the abandoned session.
5. `DownloadBlock` observed while `InProgress { blocks_seen, block_type_hint }`: transitions to
   `InProgress { blocks_seen: blocks_seen + 1, block_type_hint }` (hint is carried, never
   re-decoded from a Download Block frame).
6. `DownloadBlock` observed while `Idle` or `Completed { .. }` (no active session): `download_state`
   is unchanged — this is an out-of-sequence frame per BC-2.21.013 EC-001; it is still classified
   `DownloadBlock` at the B1 layer, but does not start or extend a session at this B2 layer.
7. `DownloadEnded` observed while `InProgress { blocks_seen, block_type_hint }`: transitions to
   `Completed { blocks_seen, block_type_hint }` — this is the session-completion event BC-2.21.030
   /031/032 key their emission on. A zero-block session (`blocks_seen == 0`, per BC-2.21.013
   EC-002's "empty download") still transitions to `Completed`.
8. `DownloadEnded` observed while `Idle` or `Completed { .. }` (no active session): `download_state`
   is unchanged — not a completion event at this B2 layer (mirrors Postcondition 6's symmetry).
9. After a `Completed { .. }` transition is consumed by BC-2.21.030's emission (single tick),
   `download_state` resets to `Idle` — the flow is ready for a new session. There is no
   "recency window" carried forward: BC-2.21.031's independent `_INSE`/`_DELE`-only emission
   path does not consult `download_state` at all (see BC-2.21.031 Invariant 2).
10. Flow close (BC-2.21.003) discards `download_state` along with all other `S7commFlowState`
    fields — an in-progress session that never completes before flow close never emits.

## Invariants

1. **One active session per flow, no nesting**: a flow can have at most one `InProgress` session
   at a time; a new `RequestDownload` always starts a fresh session (Postcondition 4), never a
   nested or parallel one.
2. **Block-type-hint decode mechanics are implementation-deferred, semantics are pinned**: per
   the pattern established in BC-2.21.012 (item-descriptor byte offset) and BC-2.21.015
   (service-string decode), this BC pins the *three-way semantic outcome*
   (`OrganizationBlock` / `OtherBlockType(code)` / `Undeterminable`) that the decode must
   produce, not the exact byte offset within the Request Download parameter block. The
   two-character block-type-code convention (`_<type><number>` filename style, e.g. `_OB00001`
   for OB1) is drawn from secondary S7comm documentation (Wireshark dissector conventions,
   snap7), not independently re-verified via a fresh live-source pass in this research burst
   — **flagged for INTEGRATE**: if the implementer's byte-level verification finds a different
   on-wire convention, only the extraction mechanics need correction; the three-way semantic
   contract holds regardless.
3. **No force-fit on block-type**: any 2-char code other than `"OB"` is `OtherBlockType(code)`,
   never coerced to `OrganizationBlock`; any undecodable filename field is `Undeterminable`,
   never guessed.
4. **This BC is state-only**: no `Finding` is emitted by this BC under any transition — all
   emission is delegated to BC-2.21.030/031/032.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `RequestDownload` → `DownloadBlock` ×3 → `DownloadEnded`, filename decodes to `_OB00001` | `Completed { blocks_seen: 3, block_type_hint: OrganizationBlock }` |
| EC-002 | `RequestDownload` → `DownloadEnded` (zero blocks) | `Completed { blocks_seen: 0, block_type_hint }` — empty download still completes |
| EC-003 | `RequestDownload` (hint A) → `DownloadBlock` → `RequestDownload` (hint B, new session) → `DownloadEnded` | First session abandoned (no emission); `Completed { blocks_seen: 0, block_type_hint: B }` for the second session only |
| EC-004 | `DownloadBlock` with no preceding `RequestDownload` on the flow | `download_state` remains `Idle`; frame still classified `DownloadBlock` at B1 layer (BC-2.21.013 EC-001) |
| EC-005 | Filename field truncated / unparseable | `block_type_hint: Undeterminable`, session still starts and can still complete |
| EC-006 | Flow closes mid-session (`InProgress`) | State discarded (BC-2.21.003); no emission for the abandoned session |

## Canonical Test Vectors

| Frame sequence | `download_state` after sequence | Category |
|---|---|---|
| `RequestDownload(_OB00001)`, `DownloadBlock`, `DownloadEnded` | `Completed{blocks_seen:1, OrganizationBlock}` | happy-path: OB download |
| `RequestDownload(_DB00010)`, `DownloadEnded` | `Completed{blocks_seen:0, OtherBlockType("DB")}` | happy-path: empty DB download |
| `RequestDownload(garbled filename)`, `DownloadBlock`, `DownloadEnded` | `Completed{blocks_seen:1, Undeterminable}` | edge-case: undecodable hint |
| `DownloadBlock` (no prior RequestDownload) | `Idle` (unchanged) | edge-case: out-of-sequence, no session |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| The `download_state` transition table is total over `{Idle, InProgress, Completed} × {RequestDownload, DownloadBlock, DownloadEnded, other}` — every combination has exactly one defined outcome | proptest P1 — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this BC is the sequence-correlation substrate CAP-21's part-B2 MITRE technique-emission scope (T0843/T0889/T0821) is built on, explicitly deferred from part B1 (BC-2.21.013 Postcondition 5) |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) |
| ADR | ADR-014 Decision 5 (T0843/T0889/T0821 detection patterns reference this sequence) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — state-only substrate BC; emission is BC-2.21.030/031/032) |

## Related BCs

- BC-2.21.013 — depends on (per-frame `RequestDownload`/`DownloadBlock`/`DownloadEnded` classification this state machine correlates)
- BC-2.21.003 — composes with (flow-close discard applies to `download_state`)
- BC-2.21.030, BC-2.21.031, BC-2.21.032 — composes with (all three consume the `Completed` transition and/or the `block_type_hint`)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7commFlowState.download_state: S7DownloadSessionState`
- `src/analyzer/s7comm.rs` (planned) — `enum S7BlockTypeHint { OrganizationBlock, OtherBlockType(String), Undeterminable }`, block-type decode helper operating on the Request Download parameter block's filename field
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table — T0821 "needs transferred-block-type ID" requirement this decode satisfies

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — proptest P1, anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | per-flow mutable state (`S7commFlowState.download_state`) |
| **Deterministic** | yes — given the same sequence of `on_data` calls |
| **Thread safety** | flow state is per-flow |
| **Overall classification** | stateful orchestration; the transition table itself is a pure, proptest-provable sub-property |
