---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.011: CIP Stop Service Observed Emits T0858 Change Operating Mode Finding

## Description

When `classify_cip_service(cip_header.service)` returns `CipServiceClass::Stop` (CIP service
code 0x07), a `Finding` is emitted carrying `T0858` ("Change Operating Mode"). The CIP Stop
service halts the user program execution on the target PLC — transitioning the controller from
Run state to Stop state. This is a high-confidence, high-severity ICS attack primitive used
in TRITON, PLC-Blaster, and INCONTROLLER attacks to halt industrial processes prior to
delivering malicious logic. Detection is per-occurrence: each CIP Stop observed on the flow
generates one finding (up to the MAX_FINDINGS cap).

## Preconditions

1. `classify_cip_service(cip_header.service)` returns `CipServiceClass::Stop`.
2. `cip_header.service & 0x80 == 0` (request, not response — a Stop response is response class).
3. The CIP item type_id is **0x00B2 (Unconnected Data Item) only**. CIP service detection does
   NOT apply to type_id 0x00B1 (Connected Data Item) in v0.11.0 — Connected items prepend a
   2-byte CIP connected sequence-count before the CIP PDU, making byte 0 of `item_data` the
   sequence-count low byte, not the CIP service byte. CIP Stop detection on 0x00B1 items is
   deferred to v0.12.0 (F-P9-001 / locked decision Option A).
4. `flow.is_non_enip == false`.
5. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. Exactly ONE `Finding` is pushed to `self.all_findings`:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary: "CIP Stop service observed: controller run→stop transition command (T0858)"`
   - `evidence`: one entry — `"CIP service=0x07 (Stop) from src={src_ip} ENIP cmd={enip_cmd:#06X} session={session_handle}"`
   - `mitre_techniques: vec!["T0858"]`
   - `source_ip: Some(<source endpoint>)` — resolved from flow_key
   - `timestamp: Some(...)` — pcap-relative capture timestamp
2. No one-shot guard: each CIP Stop frame generates one finding. An attacker repeatedly
   cycling a PLC between Stop and Run generates one T0858 finding per Stop (up to MAX_FINDINGS).

## Invariants

1. **T0858 is the correct v19.1 technique** [MITRE: enip-mitre-ics-tagging.md §1]:
   T0858 "Change Operating Mode" (ICS Execution, TA0104). Confirmed via live ATT&CK v19.1
   page: T0858 explicitly models halting PLC user programs, cites TRITON, PLC-Blaster,
   INCONTROLLER. Revoked alternatives T0814 (DoS) and T0857 (firmware — revoked) must NOT
   be emitted for CIP Stop. [VERIFIED: enip-mitre-ics-tagging.md §1, 2026-06-24]
2. **New MitreTactic variant required**: T0858 maps to `MitreTactic::IcsExecution` (TA0104)
   — a new variant added atomically in STORY-EIP-09 (ADR-010 Decision 7, VP-007 obligation).
   The Display string is `"Execution (ICS)"`.
3. **Per-occurrence detection**: CIP Stop is a high-value signal; each occurrence is
   independently significant. An attacker may issue multiple Stop commands as part of a
   coordinated attack sequence.
4. **Request only**: the detection fires on CIP Stop *requests* (service byte `0x07`, high
   bit clear). CIP Stop *responses* have high bit set (0x87) and are classified as
   `CipServiceClass::Response`; responses do not trigger this detection.
5. **High confidence**: CIP Stop in an SCADA/OT environment is rarely if ever legitimate
   from an unexpected source. `Confidence::High` is appropriate.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single CIP Stop (0x07) request | One T0858 finding emitted; Likely/High |
| EC-002 | CIP Stop *response* (0x87) | `CipServiceClass::Response` — no T0858 finding; covered by BC-2.17.008 error-response tracking if status non-zero |
| EC-003 | Multiple CIP Stop commands in sequence | One T0858 finding per Stop (per-occurrence); all up to MAX_FINDINGS cap |
| EC-004 | CIP Stop on `type_id == 0x00B1` (Connected Data Item) | NO finding in v0.11.0. The analyzer skips CIP-service detection entirely for 0x00B1 items (the 2-byte sequence-count prefix would cause parse_cip_header to misread byte 0 as CIP service). Connected-item CIP Stop detection is deferred to v0.12.0 (F-P9-001 / locked decision Option A). |
| EC-005 | `all_findings.len() == MAX_FINDINGS` when Stop arrives | No finding pushed; per BC-2.17.022 cap |
| EC-006 | CIP Stop in same session as ForwardOpen | T0858 finding emitted; ForwardOpen anomaly finding (BC-2.17.015) also present — co-emission allowed; two separate findings |

## Canonical Test Vectors

**CIP Stop in SendRRData frame:**
```
ENIP header: command=0x006F (SendRRData), length=<N>, session=0x01020304
CPF item: type_id=0x00B2 (Unconnected Data), item_data[0]=0x07 (Stop)
```
Expected: `Finding { mitre_techniques: ["T0858"], verdict: Likely, confidence: High,
summary: "CIP Stop service observed: controller run→stop transition command (T0858)" }`

| CIP service byte | classify_cip_service result | Finding emitted? | Technique |
|-----------------|---------------------------|-----------------|---------|
| `0x07` | `Stop` | Yes | T0858 |
| `0x87` | `Response` | No (BC-2.17.008) | — |
| `0x05` | `Reset` | No (BC-2.17.013) | T0816 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-D (indirect): `classify_cip_service(0x07)` returns `Stop` — precondition verified | Kani Sub-D totality proof |
| (none) | Per-occurrence finding emission, T0858 tag: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — detecting CIP Stop is the highest-priority EtherNet/IP threat detection: the CIP Stop service (0x07) directly halts PLC user program execution (run→stop transition), the same primitive used by TRITON, PLC-Blaster, and INCONTROLLER to disable safety systems and halt processes; T0858 (ics-attack-19.1 pin) is the authoritative technique |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 7 (T0858 technique); src/mitre.rs (T0858 new catalog entry + IcsExecution variant) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | T0858 — Change Operating Mode (ICS Execution TA0104; new in v0.11.0; requires new `technique_info("T0858")` arm + `IcsExecution` MitreTactic variant in src/mitre.rs; VP-007 atomic burst STORY-EIP-09) |

## Related BCs

- BC-2.17.007 — depends on (classify_cip_service returning Stop is the precondition)
- BC-2.17.013 — composes with (CIP Reset T0816 is a sibling per-occurrence detection)
- BC-2.17.015 — composes with (ForwardOpen in same session may co-emit T1692.001 on command)
- BC-2.17.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/enip.rs` — `process_pdu`: CIP dispatch `if matches!(service_class, CipServiceClass::Stop) { /* emit T0858 */ }`
- `src/mitre.rs` — `technique_info("T0858")` arm — NEW in v0.11.0 (STORY-EIP-09 VP-007 burst)
- `src/mitre.rs` — `MitreTactic::IcsExecution` variant — NEW in v0.11.0
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 7` (T0858 active set)
- `.factory/research/enip-mitre-ics-tagging.md §1` (T0858 CIP Stop mapping)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-D (indirect — verifies classify_cip_service precondition for 0x07)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 7 (T0858 active technique); enip-mitre-ics-tagging.md §1 ("T0858 Change Operating Mode — High confidence, CIP service 0x07 (Stop)"); live ATT&CK T0858 page verified 2026-06-24 (Execution TA0104, cites TRITON/PLC-Blaster/INCONTROLLER) |
| **Confidence** | high — T0858 technique verified live (ics-attack-19.1); CIP service 0x07 mapping is normative ODVA CIP |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes — same CIP Stop frame produces same finding |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (detection within process_pdu) |
