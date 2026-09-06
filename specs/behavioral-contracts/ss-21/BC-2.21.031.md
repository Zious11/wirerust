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

# BC-2.21.031: T0889 Modify Program Co-Tagged on Download Completion, or Standalone on Block Activate/Delete

## Description

T0889 ("Modify Program") has TWO independent emission paths per ADR-014 Decision 5, mirroring
the source research's "SEED + EMIT (co-tag with T0843)" recommendation plus the `_INSE`/`_DELE`
alternative detection pattern: **(a)** every completed download session (BC-2.21.030) inherently
modifies the PLC's program, so T0889 is unconditionally appended to BC-2.21.030's finding
whenever it fires; **(b)** a bare `0x28` PLC Control frame decoded as `_INSE` (BlockActivate) or
`_DELE` (BlockDelete) — BC-2.21.015 — is independently significant evidence of program
modification even with **no** preceding download session on the flow (e.g. activating a block
that was deployed via an out-of-band mechanism, or deleting a block outright) and emits its OWN
finding. This is the disambiguation the source research flags: T0843 requires a **completed
download**; T0889 fires on a completed download **OR** on activate/delete alone — T0889's
emission surface is a strict superset of T0843's.

## Preconditions

**Path (a) — download co-tag:**
1. BC-2.21.030's Precondition 1 holds (a download session has just transitioned to `Completed`).

**Path (b) — standalone activate/delete:**
2. `S7ClassicFunction::PlcControl(service)` where `service ∈
   {PlcControlService::BlockActivate, PlcControlService::BlockDelete}` (BC-2.21.015).
3. `self.all_findings.len() < MAX_S7COMM_FINDINGS` (path (b) only — path (a) is gated by
   BC-2.21.030's own cap check, since it appends to an already-admitted finding).

## Postconditions

**Path (a):**
1. `"T0889"` is unconditionally appended to `mitre_techniques` of the SAME `Finding` object
   BC-2.21.030 pushes for this completed session — no separate `Finding` is created. The
   finding's `category`/`verdict`/`confidence`/`summary`/`evidence` remain as BC-2.21.030
   defines them (T0889 does not independently raise or lower confidence for path (a); the
   download-transfer evidence is shared).

**Path (b):**
2. If path (a) did NOT fire for this frame (i.e., this `PlcControl(BlockActivate|BlockDelete)`
   frame is NOT the frame that also completed a download session in the same `on_data` call —
   see Invariant 3 for why these are mutually exclusive triggers) AND
   `self.all_findings.len() < MAX_S7COMM_FINDINGS`: exactly ONE `Finding` is pushed:
   - `category: ThreatCategory::Persistence`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium`
   - `summary: "S7comm PLC Control block-{activate|delete} observed ({_INSE|_DELE}): program modification without an observed prior download (T0889)"`
   - `evidence`: one entry — `"S7comm FC 0x28 PI-service={_INSE|_DELE} from src={src_ip} dst={dst_ip}:102"`
   - `mitre_techniques: vec!["T0889"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
3. No one-shot guard on path (b): each standalone activate/delete frame generates one finding
   (mirrors ENIP's per-occurrence convention, BC-2.17.011).

## Invariants

1. **T0889 is the correct v19.1/v19.2 technique** [MITRE: s7comm-mitre-ics-tagging.md
   §Per-technique validation table]: T0889 "Modify Program" is current, not previously seeded.
   Tactic: `MitreTactic::IcsPersistence` (`TA0110`) — a NEW variant per ADR-014 Decision 5
   (no existing ICS `MitreTactic` variant covers TA0110).
2. **`ThreatCategory::Persistence` maps `MitreTactic::IcsPersistence`**: consistent with the
   dual-taxonomy convention (BC-2.21.030 Invariant 4); path (b)'s standalone finding uses
   `Persistence` directly since there is no shared finding to append to.
3. **Path (a) vs. path (b) are mutually exclusive per triggering frame, not per session**: a
   `DownloadEnded` frame can never itself be a `PlcControl` frame (BC-2.21.013/015 classify
   disjoint FC values, `0x1C` vs `0x28`), so no single frame can trigger both paths
   simultaneously. However, a download session's completion (path a) and a LATER,
   independent `_INSE` activating that just-downloaded block (path b) on the SAME flow are
   both legitimate and both fire — this is intentional: the download proves transfer, the
   activate proves the transferred block was put into effect, and both are independently
   forensically significant (mirrors BC-2.21.013 EC-003's "the two BCs do not share state"
   design note, now resolved at the B2 emission layer as "both may fire, deliberately").
4. **T0889's emission surface is a strict superset of T0843's**: every path-(a) trigger also
   satisfies T0843 (BC-2.21.030); path (b) triggers (`_INSE`/`_DELE` alone) do NOT satisfy
   T0843 (no download session required) — this is the disambiguation invariant the source
   research names explicitly.
5. **`BlockActivate`/`BlockDelete` without decode of the target block number is honest, not a
   defect**: per BC-2.21.015's design, this BC does not decode which specific block is
   activated/deleted (only that a block activate/delete operation was observed) — the finding's
   evidence captures the service name only.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Complete download session (`0x1A→0x1B→0x1C`), no follow-on `_INSE`/`_DELE` | One finding (BC-2.21.030's), `mitre_techniques` includes `"T0843","T0889"` |
| EC-002 | Bare `_INSE` with NO preceding download on the flow | One standalone T0889 finding (path b), `mitre_techniques: ["T0889"]` only — no T0843 |
| EC-003 | Bare `_DELE` with NO preceding download | Same as EC-002, `_DELE` variant |
| EC-004 | Download session completes, THEN a LATER `_INSE` activates the deployed block | TWO findings: BC-2.21.030's session finding (`T0843,T0889`), AND a standalone path-(b) T0889 finding for the `_INSE` (Invariant 3) |
| EC-005 | `_GARB` (MemoryCompress) or `_MODU` (RamToRom) PLC Control | NO T0889 finding — neither is `BlockActivate` nor `BlockDelete` (out of this BC's precondition set) |
| EC-006 | `self.all_findings.len() == MAX_S7COMM_FINDINGS` when a standalone `_INSE` arrives | No path-(b) finding pushed |

## Canonical Test Vectors

| Trigger | Expected `mitre_techniques` | Category |
|---|---|---|
| `0x1A→0x1B→0x1C` complete session | `["T0843","T0889"]` (one finding, BC-2.21.030's) | happy-path: download co-tag |
| Bare `0x28 _INSE`, no prior download | `["T0889"]` (one standalone finding) | happy-path: activate-alone |
| Bare `0x28 _DELE`, no prior download | `["T0889"]` (one standalone finding) | happy-path: delete-alone |
| `0x28 _GARB` | (no T0889 finding) | negative: not activate/delete |
| Complete session THEN later `0x28 _INSE` | Two findings: `["T0843","T0889"]` + `["T0889"]` | edge-case: both paths fire |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Path (a)/(b) mutual-exclusivity-per-frame and both-may-fire-per-session: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — program-modification detection (T0889) is named alongside T0843 in CAP-21's own description of this capability's MITRE-emission scope, per ADR-014 Decision 5 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (T0889 new catalog entry + `IcsPersistence` variant) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0889 — Modify Program (ICS Persistence, TA0110; NEW in feature-s7comm; requires new `technique_info("T0889")` arm + `MitreTactic::IcsPersistence` variant in `src/mitre.rs`, VP-007 six-part atomic obligation, `SEEDED_TECHNIQUE_ID_COUNT` 29 → 32 — shared obligation with BC-2.21.030's T0843) |

## Related BCs

- BC-2.21.015 — depends on (`PlcControlService::BlockActivate`/`BlockDelete` classification, path b)
- BC-2.21.030 — composes with (path a appends to BC-2.21.030's finding; the two BCs together define one finding's full `mitre_techniques` contribution for a completed session)
- BC-2.21.032 — composes with (T0821 also conditionally appends to BC-2.21.030's finding — all three BCs jointly define its final tag set)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — session-completion handler (shared with BC-2.21.030): appends `"T0889"` unconditionally
- `src/analyzer/s7comm.rs` (planned) — standalone `PlcControl(BlockActivate|BlockDelete)` handler: `if !session_just_completed_this_frame { /* emit standalone T0889 */ }`
- `src/mitre.rs` — `technique_info("T0889")` arm (NEW) returning `MitreTactic::IcsPersistence`
- `src/mitre.rs` — `MitreTactic::IcsPersistence` variant (NEW): `tactic_id() -> "TA0110"`, `Display -> "Persistence (ICS)"`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 5`
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table (T0889 row)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — VP allocation happens in the F2 INTEGRATE sub-burst, anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `all_findings`; path (a) reads shared session-completion state with BC-2.21.030 |
| **Deterministic** | yes |
| **Thread safety** | flow state is per-flow |
| **Overall classification** | effectful shell |
