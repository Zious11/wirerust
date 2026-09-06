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

# BC-2.21.038: Read SZL or Block-List Userdata Emits T0888 Remote System Information Discovery Finding

## Description

When `S7ClassicFunction::Userdata(S7UserdataFunction::CpuReadSzl)` (BC-2.21.020, group `0x04`
subfn `0x01`) OR `S7ClassicFunction::Userdata(S7UserdataFunction::BlockFunctions(_))`
(BC-2.21.019, group `0x03`, any subfunction) is classified, a `Finding` carrying `T0888`
("Remote System Information Discovery") is emitted. This is the direct consumer of B1's
load-bearing group-`0x03`/`0x07` correction (BC-2.21.019/022) — getting that correction right
is exactly what makes this emission call-site correct. T0888 is **already seeded and already
emitted** in `src/mitre.rs` (Modbus, `MitreTactic::IcsDiscovery`) — this BC adds ONLY the S7comm
emission call-sites.

## Preconditions

1. `S7ClassicFunction::Userdata(S7UserdataFunction::CpuReadSzl)` (BC-2.21.020) OR
   `S7ClassicFunction::Userdata(S7UserdataFunction::BlockFunctions(subfn))` (BC-2.21.019, any
   `subfn`) classified.
2. `self.all_findings.len() < MAX_S7COMM_FINDINGS`.

## Postconditions

1. Exactly ONE `Finding` is pushed per qualifying Userdata frame:
   - `category: ThreatCategory::Reconnaissance`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary`: `"S7comm Read SZL (System Status List) observed: CPU/device profiling (T0888)"`
     for `CpuReadSzl`, or `"S7comm block enumeration observed (List Blocks / List Blocks of
     Type / Get Block Info): PLC program inventory reconnaissance (T0888)"` for
     `BlockFunctions(_)`
   - `evidence`: one entry — `"S7comm FC 0x07 (Userdata) group={0x04|0x03} subfn={subfn:#04X} from src={src_ip} dst={dst_ip}:102"`
   - `mitre_techniques: vec!["T0888"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. No one-shot guard: per-occurrence, mirroring the established convention (BC-2.21.034/035/036).
3. `S7UserdataFunction::CpuOther(_)`, `TimeFunctions(_)`, and `OtherGroup(_, _)` never trigger
   this BC — only the two named T0888-relevant classifications do.

## Invariants

1. **T0888 already seeded + emitted** [MITRE: s7comm-mitre-ics-tagging.md §Already-seeded
   confirmation]: `MitreTactic::IcsDiscovery` (`TA0102`) — no `src/mitre.rs` catalog or enum
   change required.
2. **Group-`0x03` correctness is load-bearing here**: this BC's `BlockFunctions(_)` trigger is
   only correct because BC-2.21.019 correctly maps group `0x03` to Block functions (the reverse
   of the common documentation error) — had B1 inverted the mapping, this BC would fire on the
   wrong wire evidence (Time functions, group `0x07`) instead. BC-2.21.019/022's bidirectional
   regression guards directly protect this BC's correctness.
3. **Direct passive-confidence**: both Read SZL and block enumeration are unambiguous
   discovery/reconnaissance operations on the wire (per the source research's "Direct
   (discovery op)" classification) — no additional corroboration is needed for `High`
   confidence, mirroring ENIP's T0846 ListIdentity treatment (BC-2.17.010).
4. **`BlockFunctions` subfunction value does not gate emission**: this BC fires on ANY
   `BlockFunctions(subfn)` value, including unnamed subfunctions beyond the three named ones
   (BC-2.21.019 Postcondition 5) — even an unenumerated group-`0x03` subfunction is still block
   enumeration in spirit and still discovery-relevant.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `CpuReadSzl` (group `0x04` subfn `0x01`) | T0888 finding, "Read SZL" summary |
| EC-002 | `BlockFunctions(0x01)` (List Blocks) | T0888 finding, "block enumeration" summary |
| EC-003 | `BlockFunctions(0x02)` (List Blocks of Type) | T0888 finding |
| EC-004 | `BlockFunctions(0x03)` (Get Block Info) | T0888 finding |
| EC-005 | `BlockFunctions(0x05)` (unnamed group-`0x03` subfunction) | T0888 finding still emitted (Invariant 4) |
| EC-006 | `CpuOther(0x02)` (group `0x04`, not Read SZL) | NO T0888 |
| EC-007 | `TimeFunctions(_)` (group `0x07`) | NO T0888 — this is the negative-space check protecting against the group-0x03/0x07 documentation error |
| EC-008 | `self.all_findings.len() == MAX_S7COMM_FINDINGS` | No finding pushed |

## Canonical Test Vectors

| Classification | Expected `mitre_techniques` | Category |
|---|---|---|
| `CpuReadSzl` | `["T0888"]` | happy-path: Read SZL |
| `BlockFunctions(0x01)` | `["T0888"]` | happy-path: List Blocks |
| `TimeFunctions(0x01)` (regression guard) | (no T0888) | regression-guard: group-0x03/0x07 correction |
| `CpuOther(0x02)` | (no T0888) | negative |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Read SZL / block-list → T0888-tagged finding, regression guard for group correction: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — CAP-21 names T0888 among the 8 reused technique IDs (ADR-014 Decision 5), and it is the direct consumer of CAP-21's own load-bearing group-0x03/0x07 correction |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); `src/mitre.rs` (existing T0888 catalog entry, no change) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0888 — Remote System Information Discovery (ICS Discovery, TA0102; already seeded + emitted via Modbus; S7comm adds emission call-sites only) |

## Related BCs

- BC-2.21.019 — depends on (`BlockFunctions(_)` classification, the load-bearing group-0x03 correction)
- BC-2.21.020 — depends on (`CpuReadSzl` classification)
- BC-2.21.022 — composes with (the group-0x07 regression guard this BC's EC-007 exercises)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `Userdata` dispatch: `if matches!(fn, CpuReadSzl | BlockFunctions(_)) { /* emit T0888 */ }`
- `src/mitre.rs` — `technique_info("T0888")` arm (existing; shared with Modbus)
- `.factory/research/s7comm-mitre-ics-tagging.md` §Already-seeded confirmation (T0888)

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
