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

# BC-2.17.013: CIP Reset Service Observed Emits T0816 Device Restart/Shutdown Finding

## Description

When `classify_cip_service(cip_header.service)` returns `CipServiceClass::Reset` (CIP service
code 0x05), a `Finding` is emitted carrying `T0816` ("Device Restart/Shutdown"). The CIP Reset
service causes the target device to perform a software reset (cold restart). This is a
high-confidence ICS threat signal: adversary-triggered device resets disable protection relays,
RTUs, or PLCs in the OT network, creating windows for follow-on attacks. Detection is
per-occurrence: each CIP Reset request generates one finding (up to MAX_FINDINGS cap).

## Preconditions

1. `classify_cip_service(cip_header.service)` returns `CipServiceClass::Reset`.
2. `cip_header.service & 0x80 == 0` (request, not response).
3. The CIP item type_id is **0x00B2 (Unconnected Data Item) only**. CIP service detection does
   NOT apply to type_id 0x00B1 (Connected Data Item) in v0.11.0 — Connected items prepend a
   2-byte CIP connected sequence-count before the CIP PDU, making byte 0 of `item_data` the
   sequence-count low byte, not the CIP service byte. CIP Reset detection on 0x00B1 items is
   deferred to v0.12.0 (F-P9-001 / locked decision Option A).
4. `flow.is_non_enip == false`.
5. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. Exactly ONE `Finding` is pushed to `self.all_findings`:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary: "CIP Reset service observed: adversary-triggered device restart (T0816)"`
   - `evidence`: one entry — `"CIP service=0x05 (Reset) from src={src_ip} ENIP cmd={enip_cmd:#06X} session={session_handle}"`
   - `mitre_techniques: vec!["T0816"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. No one-shot guard: per-occurrence detection.

## Invariants

1. **T0816 is the correct v19.1 technique** [MITRE: enip-mitre-ics-tagging.md §2]:
   T0816 "Device Restart/Shutdown" (ICS Inhibit Response Function, TA0107; tactic variant
   `IcsInhibitResponseFunction` — existing in `src/mitre.rs`). Confirmed against ATT&CK v19.1.
   New catalog entry required in v0.11.0 (`technique_info("T0816")` arm — VP-007 atomic burst
   STORY-EIP-09). [VERIFIED: enip-mitre-ics-tagging.md §2, 2026-06-24]
2. **Per-occurrence**: each CIP Reset is independently significant. No threshold or window.
3. **High confidence**: CIP Reset in production OT environments is almost always adversarial
   or a serious operator error; `Confidence::High` is appropriate.
4. **Request-only**: CIP Reset *responses* (service byte 0x85) are Response class — no finding.
5. **Distinct from CIP Stop**: Stop (0x07, T0858) halts the user program; Reset (0x05, T0816)
   cold-reboots the device. Both generate independent per-occurrence findings.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single CIP Reset (0x05) request | One T0816 finding emitted; Likely/High |
| EC-002 | CIP Reset response (0x85) | `CipServiceClass::Response` — no T0816 finding |
| EC-003 | CIP Reset followed by CIP Stop | Two independent findings: T0816 + T0858 (per-occurrence, separate detections) |
| EC-004 | Multiple CIP Reset commands (attack loop) | One T0816 per Reset up to MAX_FINDINGS |
| EC-005 | `all_findings.len() == MAX_FINDINGS` when Reset arrives | No finding pushed (cap) |
| EC-006 | CIP Reset in a type_id=0x00B1 (Connected Data Item) | NO finding in v0.11.0. The analyzer skips CIP-service detection entirely for 0x00B1 items. Connected-item CIP Reset detection is deferred to v0.12.0 (F-P9-001 / locked decision Option A). |

## Canonical Test Vectors

**CIP Reset in SendRRData frame:**
```
ENIP header: command=0x006F (SendRRData), session=0xAABBCCDD
CPF item: type_id=0x00B2, service=0x05 (Reset), path_size=0 (no path — applies to device-level)
```
Expected: `Finding { mitre_techniques: ["T0816"], verdict: Likely, confidence: High,
summary: "CIP Reset service observed: adversary-triggered device restart (T0816)" }`

| CIP service byte | classify result | Finding? | Technique |
|-----------------|----------------|---------|---------|
| `0x05` | `Reset` | Yes | T0816 |
| `0x85` | `Response` | No | — |
| `0x07` | `Stop` | No (BC-2.17.011) | T0858 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-D (indirect): `classify_cip_service(0x05)` returns `Reset` | Kani Sub-D |
| (none) | Per-occurrence finding, T0816 tag: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — CIP Reset detection is a required ICS threat-detection capability: adversary-triggered device resets (T0816) disable safety instrumented systems and protection relays, creating attack windows; the CIP Reset service (0x05) is the primary mechanism for remote device restart in EtherNet/IP environments |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 7 (T0816); src/mitre.rs (T0816 new entry) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | T0816 — Device Restart/Shutdown (ICS Inhibit Response Function TA0107; new in v0.11.0; requires `technique_info("T0816")` arm in src/mitre.rs; VP-007 atomic burst STORY-EIP-09) |

## Related BCs

- BC-2.17.007 — depends on (classify_cip_service returning Reset)
- BC-2.17.011 — composes with (sibling Stop/T0858 per-occurrence detection)
- BC-2.17.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/enip.rs` — `process_pdu`: `if matches!(service_class, CipServiceClass::Reset) { /* emit T0816 */ }`
- `src/mitre.rs` — `technique_info("T0816")` arm — NEW in v0.11.0 (VP-007 burst STORY-EIP-09)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 7` (T0816 active set)
- `.factory/research/enip-mitre-ics-tagging.md §2` (T0816 CIP Reset mapping)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-D (indirect — verifies classify_cip_service for 0x05)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 7 (T0816 active); enip-mitre-ics-tagging.md §2 (T0816 CIP Reset — "T0816 Device Restart/Shutdown, Inhibit Response Function, confirmed"); new mitre.rs entry required |
| **Confidence** | high — T0816 confirmed live (ics-attack-19.1); CIP Reset (0x05) is unambiguous device restart |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
