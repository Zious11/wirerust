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

# BC-2.17.014: CIP Identity-Read to Identity Object or Error Burst Emits T0888 Remote System Information Discovery

## Description

T0888 ("Remote System Information Discovery") is emitted for two complementary patterns of
CIP-layer device reconnaissance: (A) a GetAttribute service targeting the CIP Identity Object
(Class 0x01), which retrieves device identification information (vendor ID, device type,
product code, revision, serial number, product name); and (B) a burst of CIP error responses
within the 10-second window (accumulated by BC-2.17.008), indicating an adversary probing for
supported services. Pattern A fires per-occurrence. Pattern B fires once per window when the
error burst threshold is exceeded (one-shot guard via `error_rate_emitted`).

## Preconditions

**Pattern A (Identity Object read):**
1. `classify_cip_service(cip_header.service)` returns `CipServiceClass::GetAttributeSingle`,
   `CipServiceClass::GetAttributesAll`, or `CipServiceClass::GetAttributeList`.
2. `parse_cip_request_path(cip_header.request_path)` contains `CipPathSegment::Class(0x01)` —
   the Identity Object class.
3. `cip_header.service & 0x80 == 0` (request).
4. `flow.is_non_enip == false`.
5. `self.all_findings.len() < MAX_FINDINGS`.

**Pattern B (error-rate burst):**
1. `flow.error_counts_in_window` total count across all status codes exceeds
   `ENIP_ERROR_BURST_THRESHOLD` (= 5; strict `>`; fires on the 6th error response within 10s — 5 errors do NOT fire).
2. `flow.error_rate_emitted == false`.
3. `flow.is_non_enip == false`.
4. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

**Pattern A finding:**
1. Exactly ONE `Finding` pushed per Identity Object GetAttribute request:
   - `category: ThreatCategory::Reconnaissance`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary: "CIP Identity Object attribute read: single-device reconnaissance (T0888)"`
   - `evidence`: `"CIP service=0x{service:02X} ({name}) path targets Identity Object (class 0x01) src={src_ip}"`
   - `mitre_techniques: vec!["T0888"]`
   - `source_ip: Some(...)`, `timestamp: Some(...)`

**Pattern B finding:**
2. Exactly ONE `Finding` per window when error burst crossed:
   - `category: ThreatCategory::Reconnaissance`
   - `verdict: Verdict::Possible`
   - `confidence: Confidence::Medium`
   - `summary: "CIP error-response burst: {total_errors} error responses in 10s window — possible service enumeration (T0888)"`
   - `evidence`: `"error_counts_in_window={map:?} within 10s; possible service probe"`
   - `mitre_techniques: vec!["T0888"]`
   - `flow.error_rate_emitted = true` (one-shot guard for window)

## Invariants

1. **T0888 is the correct v19.1 technique** [MITRE: enip-mitre-ics-tagging.md §4a]:
   T0888 "Remote System Information Discovery" (ICS Discovery, TA0102). Already seeded in
   `src/mitre.rs`; no new catalog entry required. [VERIFIED: enip-mitre-ics-tagging.md §4a]
2. **Pattern A is per-occurrence; Pattern B is windowed one-shot**: Identity reads are
   always individually significant (direct device profiling). Error bursts require
   accumulation before the finding fires.
3. **Error burst threshold**: named constant `ENIP_ERROR_BURST_THRESHOLD = 5` — Pattern B
   fires when the total count of CIP error responses (any non-zero general_status) within 10
   seconds strictly exceeds (`>`) this threshold: 6 errors fire, 5 do not. Matches BC-2.17.012
   strict `>` convention so the analyzer uses one comparison semantics throughout. Operators
   with noisy SCADA systems may raise this to reduce false positives.
   [MEDIUM-confidence, un-calibrated; ref O-03; defined in ADR-010 Open Items]
4. **Distinct from T0846**: T0888 is single-device profiling; T0846 is network enumeration
   (ListIdentity). These are complementary and independent.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | GetAttributeSingle to Identity Object (Class=0x01) | Pattern A: T0888 finding Likely/High |
| EC-002 | GetAttributesAll to Class=0x01 | Pattern A: T0888 finding Likely/High |
| EC-003 | GetAttributeSingle to Class=0x04 (Assembly — not Identity) | No T0888 finding (not Identity Object) |
| EC-004 | 6 CIP error responses in 10s (threshold=5, strict `>`) | Pattern B: T0888 finding Possible/Medium; `error_rate_emitted=true` |
| EC-005 | 5 error responses (threshold=5, strict `>`: 5 > 5 is false) | No Pattern B finding |
| EC-006 | Pattern B guard set; 5 more errors arrive in same window | Guard prevents additional Pattern B finding |
| EC-007 | 10s window expires; 5 more errors | New window; `error_rate_emitted=false`; Pattern B can fire again |
| EC-008 | ListIdentity (T0846) followed by GetAttributeSingle to Identity | T0846 finding + T0888 finding — both independent detections |

## Canonical Test Vectors

**GetAttributeSingle to Identity Object:**
```
ENIP: SendRRData, session=0x00000001
CIP: service=0x0E, path=[0x20, 0x01, 0x24, 0x01, 0x30, 0x07]
     Class=0x01 (Identity), Instance=1, Attr=7 (ProductName)
```
Expected Pattern A: T0888 Likely/High

**Error burst (6 responses with non-zero status; strict `>` threshold=5):**
```
6 CIP responses each with general_status != 0x00 within 10 seconds
```
Expected Pattern B: T0888 Possible/Medium (5 errors → no finding; 6th crosses strict > threshold)

| Scenario | Pattern | Finding verdict | Confidence |
|----------|---------|-----------------|-----------|
| GetAttributeSingle to Class=0x01 | A | Likely | High |
| GetAttributesAll to Class=0x01 | A | Likely | High |
| 6 CIP errors in 10s (threshold=5, strict >) | B | Possible | Medium |
| 5 CIP errors in 10s (threshold=5, strict >) | — | None | — |
| GetAttributeSingle to Class=0x04 | — | None | — |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-D (indirect): GetAttribute service classification | Kani Sub-D |
| (none) | Identity Object path check, error-burst threshold, one-shot guard: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — detecting CIP Identity Object reads and CIP error-response bursts is required for T0888 Remote System Information Discovery: adversaries systematically query EtherNet/IP devices for vendor/model/revision/serial data before deploying targeted payloads; error bursts from probing unknown service codes are a secondary reconnaissance signal |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 7 (T0888 active technique) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | T0888 — Remote System Information Discovery (ICS Discovery TA0102; active ics-attack-19.1; already seeded in src/mitre.rs) |

## Related BCs

- BC-2.17.007 — depends on (GetAttribute service classification; Pattern A precondition 1)
- BC-2.17.009 — depends on (CIP path parse identifies Identity Object Class=0x01; Pattern A precondition 2)
- BC-2.17.008 — depends on (error_counts_in_window accumulation feeds Pattern B)
- BC-2.17.010 — composes with (T0846 ListIdentity is the network-scope sibling)
- BC-2.17.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/enip.rs` — Pattern A: `if matches!(service_class, CipServiceClass::GetAttributeSingle | ...) && path_contains_identity_class { /* emit T0888 */ }`
- `src/analyzer/enip.rs` — `const ENIP_ERROR_BURST_THRESHOLD: u64 = 5;`
- `src/analyzer/enip.rs` — Pattern B: `if total_error_count > ENIP_ERROR_BURST_THRESHOLD && !flow.error_rate_emitted { /* emit T0888 */ flow.error_rate_emitted = true; }`
- `src/analyzer/enip.rs` — `EnipFlowState.error_rate_emitted: bool`
- `src/mitre.rs` — `technique_info("T0888")` arm (existing)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 7` (T0888 active)
- `.factory/research/enip-mitre-ics-tagging.md §4a` (T0888 identity read mapping)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-D (indirect — GetAttribute service classification)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 7 (T0888 active); enip-mitre-ics-tagging.md §4a (T0888 "for device-specific Identity-Object attribute reads — vendor/product/revision/serial profiling" — verified 2026-06-24); CIP Identity Object Class 0x01 per ODVA CIP Specification Vol 1 §5-2 |
| **Confidence** | high for Pattern A (direct Identity read; T0888 confirmed); medium for Pattern B (error burst is indirect evidence) |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads flow.error_counts_in_window; mutates flow.error_rate_emitted, all_findings |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
