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

# BC-2.21.035: Write Var to Data Block or Marker Area Emits T0836 Modify Parameter Finding

## Description

When `S7ClassicFunction::WriteVar(area)` (BC-2.21.012) has `area ∈ {S7AreaCode::Markers,
S7AreaCode::DataBlock}` (`0x83`/`0x84`), a `Finding` carrying `T0836` ("Modify Parameter") is
emitted. T0836 is **already seeded and already emitted** in `src/mitre.rs` (Modbus,
`MitreTactic::IcsImpairProcessControl`) — this BC adds ONLY the S7comm emission call-site.

## Preconditions

1. `S7ClassicFunction::WriteVar(area)` classified per BC-2.21.012.
2. `area ∈ {Markers, DataBlock}`.
3. `self.all_findings.len() < MAX_S7COMM_FINDINGS`.

## Postconditions

1. Exactly ONE `Finding` is pushed per Write Var frame satisfying the preconditions:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium`
   - `summary: "S7comm Write Var to {area} observed: parameter/data-block modification (T0836)"`
   - `evidence`: one entry — `"S7comm FC 0x05 (WriteVar) area={area:#04X} from src={src_ip} dst={dst_ip}:102"`
   - `mitre_techniques: vec!["T0836"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. No one-shot guard: per-occurrence, mirroring BC-2.21.034.
3. `Confidence::Medium` (not `High`, unlike BC-2.21.034's I/O-area case) reflects that a DB/
   marker write is engineering-parameter modification, which occurs far more routinely in
   legitimate operations (recipe changes, setpoint tuning) than a direct I/O-image force —
   this mirrors the confidence differential the source research implies between the two
   reused areas without asserting an unverified numeric false-positive rate.

## Invariants

1. **T0836 already seeded + emitted** [MITRE: s7comm-mitre-ics-tagging.md §Already-seeded
   confirmation]: no `src/mitre.rs` catalog change required — S7comm adds an emission
   call-site only.
2. **Mutually exclusive with T0835 by area-code construction**: `S7AreaCode` is a single enum
   value per WriteVar frame (BC-2.21.012 Invariant 1's exhaustive-but-open mapping); no area
   value is ever in both this BC's set and BC-2.21.034's set, so a single WriteVar frame never
   emits both T0835 and T0836.
3. **Counters (`0x1C`) and Timers (`0x1D`) are NOT in either reused set**: per ADR-014 Decision
   5's reuse table, only `0x80`-`0x84` are named; Counters/Timers areas (which B1's
   `S7AreaCode` also models, BC-2.21.012) trigger neither T0835 nor T0836 in this cycle —
   flagged as a documented, deliberate scope boundary, not an oversight, since the source
   research did not establish a technique mapping for counter/timer writes specifically.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `WriteVar(DataBlock)` (`0x84`) | T0836 finding |
| EC-002 | `WriteVar(Markers)` (`0x83`) | T0836 finding |
| EC-003 | `WriteVar(Counters)` (`0x1C`) or `WriteVar(Timers)` (`0x1D`) | NO T0835/T0836 — outside both reused sets |
| EC-004 | `WriteVar(InstanceDb)` (`0x85`) | NO T0835/T0836 — instance DB not named in ADR-014 Decision 5's reuse table (distinct from plain `DataBlock` `0x84`) |
| EC-005 | `self.all_findings.len() == MAX_S7COMM_FINDINGS` | No finding pushed |

## Canonical Test Vectors

| Area code | Expected `mitre_techniques` | Category |
|---|---|---|
| `0x84` (DataBlock) | `["T0836"]` | happy-path |
| `0x83` (Markers) | `["T0836"]` | happy-path |
| `0x1C` (Counters) | (no finding) | negative: outside scope |
| `0x85` (InstanceDb) | (no finding) | negative: not named by ADR-014 Decision 5 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | DB/marker WriteVar → T0836-tagged finding: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21 explicitly names T0836 among the 8 reused technique IDs, per ADR-014 Decision 5 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (existing T0836 catalog entry, no change) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0836 — Modify Parameter (ICS Impair Process Control, TA0106; already seeded + emitted via Modbus; S7comm adds an emission call-site only) |

## Related BCs

- BC-2.21.012 — depends on (`WriteVar(area)` classification and area-code extraction)
- BC-2.21.034 — composes with (the sibling T0835 area set, `0x80`/`0x81`/`0x82`; mutually exclusive)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `WriteVar` dispatch: `if matches!(area, Markers | DataBlock) { /* emit T0836 */ }`
- `src/mitre.rs` — `technique_info("T0836")` arm (existing; shared with Modbus)
- `.factory/research/s7comm-mitre-ics-tagging.md` §Already-seeded confirmation (T0836)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(None dedicated.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `all_findings` |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
