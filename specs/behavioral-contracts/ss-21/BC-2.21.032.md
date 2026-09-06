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

# BC-2.21.032: T0821 Modify Controller Tasking Co-Tagged on Download Completion, Gated by Block-Type Decodability

## Description

T0821 ("Modify Controller Tasking") is co-tagged onto BC-2.21.030's completed-download-session
finding when the transferred block is an Organization Block (OB — the cyclic/interrupt-driven
execution unit of an S7 CPU, e.g. OB1 for the main cyclic program). Per ADR-014 Decision 5 and
the source research, "FC alone can't prove tasking change; needs transferred-block-type ID" —
this BC's confidence and even its firing condition are **gated by BC-2.21.029's
`block_type_hint`**: a decoded `OrganizationBlock` hint upgrades to a positive, medium-confidence
co-tag; a decoded `OtherBlockType` hint (a non-OB block, e.g. a plain DB or FB) **suppresses**
T0821 entirely for that session (the transfer is not a tasking change); an `Undeterminable`
hint (decode failed or was not attempted) falls back to a low-confidence co-tag, honestly
reflecting that OB-ness could not be ruled out. T0821 reuses the existing
`MitreTactic::IcsExecution` (`TA0104`) — no new `MitreTactic` variant, unlike T0843/T0889.

## Preconditions

1. BC-2.21.030's Precondition 1 holds (a download session has just transitioned to `Completed
   { blocks_seen, block_type_hint }`) — T0821 has no independent, non-download-session trigger.

## Postconditions

1. If `block_type_hint == S7BlockTypeHint::OrganizationBlock`: `"T0821"` is appended to the SAME
   `Finding` BC-2.21.030 (and, if applicable, BC-2.21.031 path a) pushes for this session, and
   the finding's evidence field additionally notes `"; transferred block decoded as Organization
   Block (OB) — cyclic/interrupt task table modification"`. This is the higher-confidence path.
2. If `block_type_hint == S7BlockTypeHint::OtherBlockType(code)`: T0821 is **NOT** appended —
   the transfer is affirmatively known not to be an OB, so tasking-change evidence is absent
   for this session. (T0843/T0889 are unaffected and still fire per their own BCs.)
3. If `block_type_hint == S7BlockTypeHint::Undeterminable`: `"T0821"` is appended to the same
   finding as in Postcondition 1, but the finding's OVERALL confidence for the T0821 aspect is
   recorded as low — since `Finding` carries one shared `confidence` field for the whole
   finding (not per-technique), and BC-2.21.030 already sets `Confidence::High` for the
   T0843/T0889 evidence, this BC does NOT downgrade the shared `confidence` field; instead, the
   evidence string appends `"; block type undeterminable — T0821 tagged at reduced confidence,
   see evidence"` so the human-readable distinction survives even though the structured
   `confidence` field is shared. (See Invariant 3 for the design rationale.)

## Invariants

1. **Gate, not blanket co-tag**: unlike T0889 (always co-tagged on completion, BC-2.21.031
   Postcondition 1), T0821's presence in `mitre_techniques` is CONDITIONAL on the block-type
   hint — this is the "guard on block-type when decodable" the source research specifies.
   Decodable-and-OB → tag; decodable-and-not-OB → no tag; undeterminable → tag (best-effort,
   cannot rule out OB).
2. **No new `MitreTactic` variant**: T0821 reuses `MitreTactic::IcsExecution` (`TA0104`), the
   same variant BC-2.17.011 (ENIP T0858) already established — ADR-014 Decision 5's live-page
   verification confirmed T0821's tactic is Execution, with no gap in the existing enum.
3. **Shared-`confidence`-field limitation is a known, PRE-EXISTING, accepted gap — NOT fixed
   this cycle (RESOLVED at F2 INTEGRATE, 2026-09-06)**: `Finding` (per `src/findings.rs`) has
   one `confidence: Confidence` field per finding, not a per-`mitre_techniques`-entry
   confidence. When T0821 (gated, sometimes low-confidence) and T0843/T0889 (always High per
   BC-2.21.030) share one finding, the finding's structured `confidence` field reflects the
   STRONGER T0843/T0889 evidence, and T0821's own confidence nuance is only recoverable from
   the evidence string, not a structured field. **INTEGRATE disposition**: this limitation
   predates feature-s7comm — it is inherited from the existing `Finding` schema (already
   exercised by every prior multi-technique-per-finding protocol, e.g. Modbus BC-2.14.013/
   014/015) and is not introduced, worsened, or newly discovered by this BC. It is recorded
   here as a **known constraint, not a defect to remediate in this feature cycle**: no schema
   change is made or scheduled by feature-s7comm. If per-technique confidence granularity is
   desired, `Finding` would need a schema change (e.g. `Vec<(String, Confidence)>` instead of
   `Vec<String>` for `mitre_techniques`) — that change, if ever undertaken, is a cross-cutting
   `src/findings.rs` schema decision spanning every protocol analyzer, not a scoped
   `feature-s7comm`/BC-2.21.032 fix, and is explicitly out of scope for this feature.
4. **Block-type decode is BC-2.21.029's concern, not re-derived here**: this BC consumes
   `block_type_hint` as already decoded; it performs no additional byte-level parsing.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Download session completes, `block_type_hint: OrganizationBlock` (decoded `_OB00001`) | `"T0821"` appended; evidence notes OB decode |
| EC-002 | Download session completes, `block_type_hint: OtherBlockType("DB")` | NO `"T0821"` — session's `mitre_techniques` is `["T0843","T0889"]` only |
| EC-003 | Download session completes, `block_type_hint: Undeterminable` | `"T0821"` appended (best-effort); evidence notes reduced confidence |
| EC-004 | Empty download (`0x1A→0x1C`, zero blocks), `block_type_hint: OrganizationBlock` | `"T0821"` still appended — Postcondition 1 does not require `blocks_seen > 0` |
| EC-005 | Standalone `_INSE`/`_DELE` with no preceding download (BC-2.21.031 path b) | NO T0821 — this BC has no standalone trigger (Precondition 1 requires a download-session completion) |

## Canonical Test Vectors

| Session / `block_type_hint` | Expected `mitre_techniques` (T0821 contribution) | Category |
|---|---|---|
| Complete session, `OrganizationBlock` | `T0821` present | happy-path: OB download |
| Complete session, `OtherBlockType("DB")` | `T0821` absent | negative: non-OB block, gated out |
| Complete session, `OtherBlockType("FB")` | `T0821` absent | negative: non-OB block, gated out |
| Complete session, `Undeterminable` | `T0821` present (low-confidence evidence note) | edge-case: best-effort fallback |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Block-type-hint → T0821-presence gate totality over the three `S7BlockTypeHint` variants: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — T0821 completes the three-technique download-session emission surface CAP-21 names explicitly (T0843, T0889, T0821) per ADR-014 Decision 5 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (T0821 new catalog entry, reuses `IcsExecution`) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0821 — Modify Controller Tasking (ICS Execution, TA0104 — REUSES existing `MitreTactic::IcsExecution`, no new variant; NEW `technique_info("T0821")` catalog arm; VP-007 six-part atomic obligation, `SEEDED_TECHNIQUE_ID_COUNT` 29 → 32 — shared obligation with BC-2.21.030/031) |

## Related BCs

- BC-2.21.029 — depends on (`block_type_hint` decode this BC's gate consumes)
- BC-2.21.030 — composes with (T0821 conditionally appends to BC-2.21.030's finding)
- BC-2.21.031 — composes with (all three BCs jointly define the completed-session finding's final `mitre_techniques` tag set)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — session-completion handler (shared with BC-2.21.030/031): `match block_type_hint { OrganizationBlock => append "T0821" (evidence: OB), OtherBlockType(_) => no-op, Undeterminable => append "T0821" (evidence: reduced-confidence note) }`
- `src/mitre.rs` — `technique_info("T0821")` arm (NEW) returning `MitreTactic::IcsExecution` (existing variant, no enum change)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 5`
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table (T0821 row)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst, anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `all_findings`; reads shared session-completion state with BC-2.21.030/031 |
| **Deterministic** | yes |
| **Thread safety** | flow state is per-flow |
| **Overall classification** | effectful shell |
