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

# BC-2.21.034: Write Var to I/O Area Emits T0835 Manipulate I/O Image Finding

## Description

When `S7ClassicFunction::WriteVar(area)` (BC-2.21.012) has `area ∈ {S7AreaCode::DirectPeripheral,
S7AreaCode::Inputs, S7AreaCode::Outputs}` (`0x80`/`0x81`/`0x82`), a `Finding` carrying `T0835`
("Manipulate I/O Image") is emitted. T0835 is **already seeded and already emitted** in
`src/mitre.rs` (Modbus) per the source research's confirmation — this BC adds ONLY the S7comm
emission call-site; no catalog entry, `MitreTactic` variant, or `SEEDED_TECHNIQUE_IDS` change is
required.

## Preconditions

1. `S7ClassicFunction::WriteVar(area)` classified per BC-2.21.012.
2. `area ∈ {DirectPeripheral, Inputs, Outputs}`.
3. `self.all_findings.len() < MAX_S7COMM_FINDINGS` (BC-2.21.030's cap constant).

## Postconditions

1. Exactly ONE `Finding` is pushed per Write Var frame satisfying the preconditions:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary: "S7comm Write Var to I/O area ({area}) observed: process-image manipulation (T0835)"`
   - `evidence`: one entry — `"S7comm FC 0x05 (WriteVar) area={area:#04X} from src={src_ip} dst={dst_ip}:102"`
   - `mitre_techniques: vec!["T0835"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. No one-shot guard: each qualifying Write Var frame generates one finding (per-occurrence,
   mirroring the Modbus/ENIP established convention for high-value write signals).
3. `S7AreaCode::Unrecognized(_)` and any area outside the three named I/O values never trigger
   this BC (BC-2.21.035 handles the DB/marker set; other unrecognized areas trigger neither).

## Invariants

1. **T0835 already seeded + emitted** [MITRE: s7comm-mitre-ics-tagging.md §Already-seeded
   confirmation]: `"T0835"` is already in `SEEDED_TECHNIQUE_IDS` and `EMITTED_IDS` (Modbus,
   `MitreTactic::IcsImpairProcessControl`, `TA0106`) — this BC requires NO change to
   `SEEDED_TECHNIQUE_ID_COUNT`, no new `technique_info` arm, no new `MitreTactic` variant.
2. **Direct-passive-confidence area classification**: BC-2.21.012 pins the I/O-area
   set (`0x80`/`0x81`/`0x82`) as T0835-eligible per ADR-014 Decision 5's reuse table — this BC
   consumes that classification without re-deriving it.
3. **Per-occurrence, request-direction only**: this BC fires on the WriteVar request
   (`header.rosctr == Rosctr::Job`); a corresponding `Ack_Data` response is not itself a write
   command and does not re-trigger emission (Write Var responses carry no area code to
   re-classify against — BC-2.21.012's area-extraction is scoped to the request parameter
   block).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `WriteVar(DirectPeripheral)` | T0835 finding |
| EC-002 | `WriteVar(Inputs)` | T0835 finding |
| EC-003 | `WriteVar(Outputs)` | T0835 finding |
| EC-004 | `WriteVar(DataBlock)` (`0x84`) | NO T0835 — this is BC-2.21.035's (T0836) set |
| EC-005 | `WriteVar(Unrecognized(0x9A))` | NO T0835 (and no T0836) — area not in either named set |
| EC-006 | `self.all_findings.len() == MAX_S7COMM_FINDINGS` | No finding pushed |

## Canonical Test Vectors

| Area code | Expected `mitre_techniques` | Category |
|---|---|---|
| `0x81` (Inputs) | `["T0835"]` | happy-path |
| `0x82` (Outputs) | `["T0835"]` | happy-path |
| `0x80` (DirectPeripheral) | `["T0835"]` | happy-path |
| `0x84` (DataBlock) | (no T0835; see BC-2.21.035) | negative |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | I/O-area WriteVar → T0835-tagged finding: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21 explicitly names T0835 among the 8 reused technique IDs this capability's MITRE-emission scope covers, per ADR-014 Decision 5 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (existing T0835 catalog entry, no change) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0835 — Manipulate I/O Image (ICS Impair Process Control, TA0106; already seeded + emitted via Modbus; S7comm adds an emission call-site only — no `src/mitre.rs` catalog change) |

## Related BCs

- BC-2.21.012 — depends on (`WriteVar(area)` classification and area-code extraction)
- BC-2.21.035 — composes with (the sibling T0836 area set, `0x83`/`0x84`)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `WriteVar` dispatch: `if matches!(area, DirectPeripheral | Inputs | Outputs) { /* emit T0835 */ }`
- `src/mitre.rs` — `technique_info("T0835")` arm (existing; shared with Modbus)
- `.factory/research/s7comm-mitre-ics-tagging.md` §Already-seeded confirmation (T0835)

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
