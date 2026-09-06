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

# BC-2.21.030: Completed Download Session Emits T0843 Program Download Finding

## Description

When the download-session state machine (BC-2.21.029) transitions to `Completed { blocks_seen,
block_type_hint }`, a `Finding` carrying `T0843` ("Program Download") is emitted. This is the
new-technique seeding decision of ADR-014 Decision 5 (`SEEDED_TECHNIQUE_ID_COUNT` 29 → 32),
requiring the new `MitreTactic::IcsLateralMovement` (`TA0109`) variant. This BC defines the
**base per-session finding shape** that BC-2.21.031 (T0889, always co-tagged) and BC-2.21.032
(T0821, conditionally co-tagged) add their technique tags to — mirroring the established
Modbus multi-tag-per-event convention (BC-2.14.013/014/015): one `Finding` per completed
download session, carrying every applicable technique in `mitre_techniques: Vec<String>`, not
one `Finding` object per technique.

## Preconditions

1. `S7commFlowState.download_state` transitions from `InProgress { blocks_seen, block_type_hint }`
   to `Completed { blocks_seen, block_type_hint }` per BC-2.21.029 Postcondition 7 (a
   `DownloadEnded` frame observed while a session is `InProgress`).
2. `self.all_findings.len() < MAX_S7COMM_FINDINGS` (new constant, mirrors
   `dnp3::MAX_FINDINGS`/`modbus::MAX_FINDINGS` = `10_000` engineering default for a
   per-analyzer DoS-resistant findings cap).

## Postconditions

1. Exactly ONE `Finding` is pushed to `self.all_findings` for this completed session:
   - `category: ThreatCategory::LateralMovement`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary: "S7comm program download sequence observed (Request Download → Download Block ×N → Download Ended): PLC program deployment (T0843)"`
   - `evidence`: one entry — `"S7comm FC 0x1A→0x1B(×{blocks_seen})→0x1C from src={src_ip} dst={dst_ip}:102"`
   - `mitre_techniques`: initialized to `vec!["T0843"]` — BC-2.21.031 unconditionally appends
     `"T0889"` to this SAME vec (see BC-2.21.031 Postcondition 1); BC-2.21.032 conditionally
     appends `"T0821"` to this SAME vec (see BC-2.21.032). This BC's contract is discharged
     once `"T0843"` is present; the final vec's full contents are the union of all three BCs'
     contributions.
   - `source_ip: Some(<flow client endpoint>)`, `timestamp: Some(...)` (pcap-relative,
     timestamp of the `DownloadEnded` frame that completed the session)
2. No one-shot guard beyond the state machine itself: a NEW completed session (a subsequent
   full `RequestDownload → ... → DownloadEnded` cycle on the same flow, per BC-2.21.029
   Postcondition 4) emits a NEW finding — repeated legitimate or malicious deployments are each
   independently significant.
3. `blocks_seen == 0` (empty download, BC-2.21.029 EC-002) still emits — an empty download
   session is a completed, well-formed download-protocol exchange and is not suppressed.

## Invariants

1. **T0843 is the correct v19.1/v19.2 technique** [MITRE: s7comm-mitre-ics-tagging.md
   §Per-technique validation table]: T0843 "Program Download" is current (parent technique;
   gained `.001`/`.002`/`.003` sub-techniques in v19 which do not affect the parent's validity),
   not seeded before this feature. Tactic: `MitreTactic::IcsLateralMovement` (`TA0109`) — a
   NEW variant per ADR-014 Decision 5's live-page tactic verification (no existing ICS
   `MitreTactic` variant covers TA0109).
2. **High confidence, Likely verdict**: a complete `0x1A→0x1B(×N)→0x1C` sequence is
   structurally unambiguous evidence of a block transfer to the PLC — this is a **Direct**
   passive-confidence signal per the source research (the transfer itself, not the
   malicious-intent claim, is what T0843 asserts).
3. **Session-scoped, not per-frame**: unlike the reused per-occurrence techniques
   (BC-2.21.034 onward), T0843 fires once per completed session, not once per
   `RequestDownload`/`DownloadBlock`/`DownloadEnded` frame — the session IS the unit of
   evidence.
4. **`ThreatCategory::LateralMovement` maps `MitreTactic::IcsLateralMovement`**: the
   `Finding.category` (Enterprise-flavored `ThreatCategory`, `src/findings.rs`) and the MITRE
   catalog's per-technique `MitreTactic` (`src/mitre.rs`, ICS-flavored `tactic_id()`) are two
   independent taxonomies in this codebase (per BC-2.17.011's established pattern for T0858/
   `IcsExecution`↔`ThreatCategory::Execution`); this BC pins the `ThreatCategory::LateralMovement`
   choice as the closest existing Enterprise-flavored category to "PLC program deployment,"
   consistent with that established dual-taxonomy convention.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `RequestDownload → DownloadBlock ×3 → DownloadEnded` | One T0843 finding; `mitre_techniques` includes `"T0843"` (plus T0889/T0821 per their own BCs) |
| EC-002 | Empty download (`RequestDownload → DownloadEnded`, zero blocks) | One T0843 finding still emitted (Invariant 3/Postcondition 3) |
| EC-003 | Two complete sessions on the same flow, sequentially | Two independent T0843 findings (Postcondition 2) |
| EC-004 | An incomplete session (`RequestDownload → DownloadBlock`, no `DownloadEnded`, flow closes) | No T0843 finding — never reaches `Completed` (BC-2.21.029 Postcondition 10) |
| EC-005 | `self.all_findings.len() == MAX_S7COMM_FINDINGS` when the session completes | No finding pushed; `download_state` still resets to `Idle` per BC-2.21.029 Postcondition 9 |
| EC-006 | An Upload sequence (`0x1D`-`0x1F`) completes on the flow | NO T0843 finding — Upload never drives `download_state` (BC-2.21.029 only transitions on `RequestDownload`/`DownloadBlock`/`DownloadEnded`); this is the load-bearing Upload/Download separation BC-2.21.014 establishes at the classification layer |

## Canonical Test Vectors

| Session sequence | Expected `mitre_techniques` (T0843 contribution) | Category |
|---|---|---|
| `0x1A → 0x1B → 0x1C` | `["T0843", ...]` | happy-path: minimal complete download |
| `0x1A → 0x1B ×5 → 0x1C` | `["T0843", ...]` (one finding regardless of block count) | happy-path: multi-block download |
| `0x1A → 0x1C` (empty) | `["T0843", ...]` | edge-case: empty download still completes |
| `0x1A → 0x1B` (flow closes, no `0x1C`) | (no finding) | negative: incomplete session |
| `0x1D → 0x1E → 0x1F` (Upload triad) | (no finding — Upload never emits T0843) | regression-guard: Upload/Download separation |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Session-completion → exactly one T0843-tagged finding: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — detecting PLC program download is the highest-value ICS threat-detection behavior this capability's MITRE-emission scope names first (per ADR-014 Decision 5 and CAP-21's own description: "Classic S7comm PDU dissection drives new MITRE ATT&CK for ICS technique emissions — T0843 (Program Download)...") |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (T0843 new catalog entry + `IcsLateralMovement` variant) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0843 — Program Download (ICS Lateral Movement, TA0109; NEW in feature-s7comm; requires new `technique_info("T0843")` arm + `MitreTactic::IcsLateralMovement` variant in `src/mitre.rs`, VP-007 six-part atomic obligation, `SEEDED_TECHNIQUE_ID_COUNT` 29 → 32) |

## Related BCs

- BC-2.21.013 — depends on (per-frame classification of the download triad)
- BC-2.21.029 — depends on (the session-completion state transition this BC's Precondition 1 keys on)
- BC-2.21.014 — composes with (the Upload triad this BC must never fire from, per EC-006)
- BC-2.21.031 — composes with (T0889 unconditionally appends to this same finding's `mitre_techniques`)
- BC-2.21.032 — composes with (T0821 conditionally appends to this same finding's `mitre_techniques`)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `on_data` / session-completion handler: `if let S7DownloadSessionState::Completed{..} = new_state { /* emit T0843 finding, apply BC-031/032 tag amendments, reset to Idle */ }`
- `src/analyzer/s7comm.rs` (planned) — `const MAX_S7COMM_FINDINGS: usize = 10_000;` (mirrors `dnp3::MAX_FINDINGS`/`modbus::MAX_FINDINGS`)
- `src/mitre.rs` — `technique_info("T0843")` arm (NEW) returning `MitreTactic::IcsLateralMovement`
- `src/mitre.rs` — `MitreTactic::IcsLateralMovement` variant (NEW): `tactic_id() -> "TA0109"`, `Display -> "Lateral Movement (ICS)"`
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 5`
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table (T0843 row)

## Story Anchor

STORY-191

## VP Anchors

- VP-007 (Kani P0, amended) — MITRE Technique ID Format and Catalog Completeness;
  registered F2 INTEGRATE sub-burst per VP-INDEX.md v2.48; source_bc extended to
  include BC-2.21.030 (T0843/T0889/T0821 seeding, SEEDED_TECHNIQUE_ID_COUNT 29→32)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `all_findings`, reads/resets `S7commFlowState.download_state` |
| **Deterministic** | yes — same frame sequence produces same finding |
| **Thread safety** | flow state is per-flow |
| **Overall classification** | effectful shell |
