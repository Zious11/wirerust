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

# BC-2.21.036: PLC Stop or PLC Control Program-Start Emits T0858 Change Operating Mode Finding

## Description

Two classic-S7comm signals map to T0858 ("Change Operating Mode"): `PlcStop` (`0x29`, BC-2.21.016
— a dedicated, unambiguous STOP request) and `PlcControl(ProgramStart)` (`0x28` PI-service
`P_PROGRAM`, BC-2.21.015 — a start/run-state control). T0858 is **already seeded and already
emitted** in `src/mitre.rs` (ENIP, `MitreTactic::IcsExecution`) — this BC adds ONLY the S7comm
emission call-sites. Per BC-2.21.015 Invariant 3, `P_PROGRAM` alone does not distinguish
start/stop/state-query sub-operations without decoding parameter bytes beyond the service name;
this BC treats bare, sub-operation-undecoded `P_PROGRAM` as a T0858 candidate at reduced
confidence relative to the unambiguous `PlcStop`, honestly reflecting that boundary (see
Invariant 3; BC-2.21.037 defines the separate, gated T0816 restart path).

## Preconditions

**Path (a) — PLC Stop:**
1. `S7ClassicFunction::PlcStop` classified (BC-2.21.016).

**Path (b) — PLC Control Program Start:**
2. `S7ClassicFunction::PlcControl(PlcControlService::ProgramStart)` classified (BC-2.21.015).

**Both paths:**
3. `self.all_findings.len() < MAX_S7COMM_FINDINGS`.

## Postconditions

1. Path (a): exactly ONE `Finding` is pushed per `PlcStop` frame:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary: "S7comm PLC Stop (0x29) observed: controller run→stop transition command (T0858)"`
   - `evidence`: one entry — `"S7comm FC 0x29 (PlcStop) from src={src_ip} dst={dst_ip}:102"`
   - `mitre_techniques: vec!["T0858"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. Path (b): exactly ONE `Finding` is pushed per `PlcControl(ProgramStart)` frame:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::Medium` (lower than path (a) — see Invariant 3)
   - `summary: "S7comm PLC Control P_PROGRAM observed: run/start-state control, sub-operation not decoded (T0858)"`
   - `evidence`: one entry — `"S7comm FC 0x28 (PlcControl) service=P_PROGRAM from src={src_ip} dst={dst_ip}:102"`
   - `mitre_techniques: vec!["T0858"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
3. No one-shot guard on either path: each qualifying frame generates one finding
   (per-occurrence, mirroring ENIP's BC-2.17.011).

## Invariants

1. **T0858 already seeded + emitted** [MITRE: s7comm-mitre-ics-tagging.md §Already-seeded
   confirmation]: `MitreTactic::IcsExecution` (`TA0104`) — no `src/mitre.rs` catalog or enum
   change required.
2. **`PlcStop` is Direct-confidence, `P_PROGRAM` is Conditional-confidence**: `PlcStop`
   (`0x29`) has exactly one meaning (BC-2.21.016 Invariant 1); `P_PROGRAM`'s sub-operation is
   undecoded at this feature's current scope (BC-2.21.015 Invariant 3) — the confidence
   differential (`High` vs. `Medium`) is this BC's honest reflection of that evidence-strength
   gap, per the source research's Direct-vs-Conditional passive-confidence legend.
3. **T0858 fires on P_PROGRAM regardless of restart-vs-start ambiguity; T0816
   (BC-2.21.037) is additive, not exclusive**: because `P_PROGRAM`'s sub-operation is not
   decoded by B1, this BC treats EVERY `P_PROGRAM` occurrence as a T0858 candidate (the safer,
   more general interpretation) — BC-2.21.037 layers an ADDITIONAL, gated T0816 co-tag onto
   the SAME finding only when a further sub-operation decode (new B2 surface, BC-2.21.037)
   confirms a restart operation specifically. This is a **flagged gap for INTEGRATE**: the
   source research names "`0x28` restart PI-service" as T0816's detection pattern, but B1's
   `PlcControlService` enum (BC-2.21.015) has no distinct restart variant — restart, if it
   exists as a wire-observable signal at all, is encoded in `P_PROGRAM`'s trailing parameter
   bytes, not a 6th top-level service string. See BC-2.21.037 for the full disclosure.
4. **No `PlcControlService::Unrecognized` co-tag**: an `Unrecognized` service string
   (BC-2.21.015 EC-006/007/008) never triggers this BC — only exact-match `ProgramStart`
   (via `P_PROGRAM`) or the dedicated `PlcStop` FC.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `PlcStop` (`0x29`) | T0858 finding, `Confidence::High` |
| EC-002 | `PlcControl(ProgramStart)` (`P_PROGRAM`) | T0858 finding, `Confidence::Medium` |
| EC-003 | `PlcControl(BlockActivate)` (`_INSE`) | NO T0858 — handled by BC-2.21.031 (T0889) instead |
| EC-004 | `PlcControl(Unrecognized)` | NO T0858 |
| EC-005 | `self.all_findings.len() == MAX_S7COMM_FINDINGS` | No finding pushed |

## Canonical Test Vectors

| Trigger | Expected `mitre_techniques` / confidence | Category |
|---|---|---|
| `0x29` PlcStop | `["T0858"]`, High | happy-path: direct stop |
| `0x28 P_PROGRAM` | `["T0858"]`, Medium | happy-path: start/state candidate |
| `0x28 _INSE` | (no T0858; see BC-2.21.031) | negative |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | PlcStop/P_PROGRAM → T0858-tagged finding, correct confidence per path: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21 explicitly names T0858 among the 8 reused technique IDs, per ADR-014 Decision 5 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (existing T0858 catalog entry, no change) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0858 — Change Operating Mode (ICS Execution, TA0104; already seeded + emitted via ENIP; S7comm adds emission call-sites only) |

## Related BCs

- BC-2.21.015 — depends on (`PlcControl(ProgramStart)` classification)
- BC-2.21.016 — depends on (`PlcStop` classification)
- BC-2.21.037 — composes with (T0816's gated additional co-tag onto the P_PROGRAM path's finding)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `PlcStop` and `PlcControl(ProgramStart)` dispatch arms
- `src/mitre.rs` — `technique_info("T0858")` arm (existing; shared with ENIP)
- `.factory/research/s7comm-mitre-ics-tagging.md` §Already-seeded confirmation (T0858)

## Story Anchor

STORY-192

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
