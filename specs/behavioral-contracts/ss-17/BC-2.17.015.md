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

# BC-2.17.015: ForwardOpen and ForwardClose Connection-Lifecycle Anomaly Detected with Empty MITRE Technique Set

## Description

When `classify_cip_service(cip_header.service)` returns `CipServiceClass::ForwardOpen` (0x54),
`CipServiceClass::LargeForwardOpen` (0x5B), or `CipServiceClass::ForwardClose` (0x4E), the
analyzer detects a CIP connection lifecycle event. ForwardOpen establishes a CIP connection
(allocating connection IDs and RPIs); ForwardClose tears it down. Per ADR-010 Decision 5
(in-scope for v0.11.0) and Decision 7 (MITRE technique gap), findings are emitted with **no
MITRE technique tag** (`mitre_techniques: vec![]`). T1692.001 is emitted on the CIP command
carrying an unauthorized action in the same session — not on the ForwardOpen or ForwardClose
itself. For ForwardOpen, the connection serial number (bytes 14–15 of the CIP request data per
ODVA Connection Manager Object) is extracted best-effort and stored for ForwardClose
correlation; if extraction fails (payload shorter than 16 bytes), correlation is skipped and
the serial number is recorded as 0 (explicitly best-effort / deferred to v0.12.0 for full
validation).

## Preconditions

1. `classify_cip_service(cip_header.service)` returns `CipServiceClass::ForwardOpen`,
   `CipServiceClass::LargeForwardOpen`, or `CipServiceClass::ForwardClose`.
2. `cip_header.service & 0x80 == 0` (request, not response).
3. `flow.is_non_enip == false`.
4. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

**ForwardOpen / LargeForwardOpen:**
1. Exactly ONE `Finding` is pushed per ForwardOpen/LargeForwardOpen request:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Possible`
   - `confidence: Confidence::Low`
   - `summary: "CIP ForwardOpen connection establishment observed from src={src_ip}: connection lifecycle anomaly"`
   - `evidence`: one entry — `"CIP service=0x{service:02X} ({name}) from src={src_ip} session={session}. No dedicated MITRE ICS technique for CIP connection establishment anomaly; T1692.001 applies only when connection demonstrably carries unauthorized command (ADR-010 Decision 7)"`
   - `mitre_techniques: vec![]` — empty; no technique tag (per ADR-010 Decision 7 policy)
   - `source_ip: Some(...)`, `timestamp: Some(...)`
2. Connection serial number (bytes 14–15 of CIP ForwardOpen request data, per ODVA Connection
   Manager Object specification) extracted best-effort:
   - If `cip_item_data.len() >= 16`: `serial = u16::from_le_bytes([cip_item_data[14], cip_item_data[15]])`; stored for ForwardClose correlation.
   - If `cip_item_data.len() < 16`: serial = 0 (extraction failed); correlation skipped. Best-effort only; full validation deferred to v0.12.0.
3. No one-shot guard: each ForwardOpen/LargeForwardOpen generates a finding.

**ForwardClose (0x4E):**
4. Exactly ONE `Finding` is pushed per ForwardClose request:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Possible`
   - `confidence: Confidence::Low`
   - `summary: "CIP ForwardClose connection teardown observed from src={src_ip}: connection lifecycle closed"`
   - `evidence`: one entry — `"CIP service=0x4E (ForwardClose) from src={src_ip} session={session}. Connection lifecycle closed; no dedicated MITRE ICS technique (ADR-010 Decision 7)"`
   - `mitre_techniques: vec![]` — empty; no technique tag (per ADR-010 Decision 7 policy)
   - `source_ip: Some(...)`, `timestamp: Some(...)`
5. No one-shot guard: each ForwardClose generates a finding (connection closure is individually observable).

## Invariants

1. **Empty MITRE technique set is intentional**: ATT&CK for ICS v19.1 has no technique
   specifically for CIP connection establishment anomaly. Emitting T1692.001 on a bare
   ForwardOpen would be speculative. The gap is documented in ADR-010 Decision 7 and in
   the finding's evidence field. [SOURCE: enip-mitre-ics-tagging.md §7; ADR-010 Decision 7]
2. **T1692.001 deferred**: T1692.001 is only emitted when the connection established by
   ForwardOpen demonstrably carries an unauthorized CIP command in the same session. This
   would require cross-BC state (ForwardOpen → CIP command in same session). In v0.11.0,
   ForwardOpen findings stand alone with empty mitre_techniques.
3. **Low confidence**: ForwardOpen from a new source may be legitimate (new device joining
   the network). Low confidence and Possible verdict communicate this uncertainty.
4. **ForwardClose tracking**: correlating ForwardOpen with ForwardClose is scoped to
   connection serial number. Full TCP/UDP cross-channel correlation is deferred (ADR-010
   Decision 6).
5. **LargeForwardOpen (0x5B)**: treated identically to ForwardOpen for detection purposes.
   Full parameter parse of the LargeForwardOpen payload is deferred (ADR-010 Decision 8).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ForwardOpen (0x54) from new source | Finding emitted; Possible/Low; `mitre_techniques: vec![]` |
| EC-002 | LargeForwardOpen (0x5B) | Finding emitted; same policy; `mitre_techniques: vec![]` |
| EC-003 | ForwardOpen *response* (0xD4 = 0x54 | 0x80) | `CipServiceClass::Response` — no ForwardOpen finding |
| EC-004 | ForwardOpen immediately followed by CIP Stop (0x07) in same session | Two findings: ForwardOpen (vec![]) + T0858 Stop. T1692.001 not added to ForwardOpen finding in v0.11.0 (cross-BC state tracking deferred) |
| EC-005 | ForwardClose (0x4E) request | Classified as `CipServiceClass::ForwardClose`; one Finding emitted: Anomaly/Possible/Low, `mitre_techniques: vec![]`, summary "CIP ForwardClose connection teardown observed..." (see Postconditions §ForwardClose) |
| EC-006 | `all_findings.len() == MAX_FINDINGS` when ForwardOpen arrives | No finding pushed; connection serial not stored (or stored separately) |

## Canonical Test Vectors

**ForwardOpen in SendRRData frame:**
```
ENIP: SendRRData, session=0x12345678
CIP item: type_id=0x00B2 (Unconnected), service=0x54 (ForwardOpen)
```
Expected: `Finding { mitre_techniques: [], verdict: Possible, confidence: Low,
summary: "CIP ForwardOpen connection establishment observed ..." }`

| CIP service | classify result | mitre_techniques | verdict | confidence | summary prefix |
|------------|----------------|-----------------|---------|-----------|----------------|
| `0x54` ForwardOpen | `ForwardOpen` | `[]` (empty) | Possible | Low | "CIP ForwardOpen connection establishment..." |
| `0x5B` LargeForwardOpen | `LargeForwardOpen` | `[]` (empty) | Possible | Low | "CIP ForwardOpen connection establishment..." |
| `0xD4` (FwdOpen response) | `Response` | (no finding) | — | — | — |
| `0x4E` ForwardClose | `ForwardClose` | `[]` (empty) | Possible | Low | "CIP ForwardClose connection teardown..." |

**ForwardClose canonical frame:**
```
ENIP: SendRRData, session=0x12345678
CIP item: type_id=0x00B2 (Unconnected), service=0x4E (ForwardClose)
```
Expected: `Finding { category: Anomaly, verdict: Possible, confidence: Low,
mitre_techniques: [], summary: "CIP ForwardClose connection teardown observed ..." }`

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-D (indirect): `classify_cip_service(0x54)` returns `ForwardOpen` | Kani Sub-D |
| (none) | Empty mitre_techniques, ForwardOpen anomaly finding: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — ForwardOpen connection-lifecycle detection is explicitly in-scope for v0.11.0 (ADR-010 Decision 5, F1 gate D-228); detecting anomalous connection establishment from unexpected sources provides ICS visibility even without a dedicated MITRE technique; the intentionally empty mitre_techniques is consistent with wirerust finding attribution design (ADR-006: zero-technique findings are supported) |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 5 (ForwardOpen in-scope), Decision 7 (MITRE gap) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (intentionally empty — no dedicated ATT&CK for ICS v19.1 technique for CIP connection establishment anomaly per ADR-010 Decision 7 and enip-mitre-ics-tagging.md §7) |

## Related BCs

- BC-2.17.007 — depends on (ForwardOpen/LargeForwardOpen classification)
- BC-2.17.011 — composes with (Stop in same session may co-exist with ForwardOpen finding)
- BC-2.17.022 — depends on (MAX_FINDINGS cap)

## Architecture Anchors

- `src/analyzer/enip.rs` — `process_pdu`: `if matches!(service_class, CipServiceClass::ForwardOpen | CipServiceClass::LargeForwardOpen | CipServiceClass::ForwardClose) { /* emit anomaly */ }`
- `src/analyzer/enip.rs` — connection serial number extraction from CIP ForwardOpen payload bytes
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 5` (ForwardOpen in-scope), §Decision 7 (empty technique policy)
- `.factory/research/enip-mitre-ics-tagging.md §7` (ForwardOpen ambiguous — no dedicated technique)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-D (indirect — verifies ForwardOpen service classification)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 5 (ForwardOpen in-scope v0.11.0); ADR-010 Decision 7 (MITRE gap: "ATT&CK for ICS v19.1 contains no technique specifically named for CIP connection establishment anomalies or ForwardOpen abuse"); enip-mitre-ics-tagging.md §7 ("Recommendation: if the analyzer emits a finding for an anomalous ForwardOpen in isolation, prefer NOT tagging a process-impact technique") |
| **Confidence** | high for detection behavior (ForwardOpen is observable); low for threat confidence (ForwardOpen may be legitimate) |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings; reads/writes connection serial number tracking state |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
