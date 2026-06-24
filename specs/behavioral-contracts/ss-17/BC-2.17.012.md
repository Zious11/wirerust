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

# BC-2.17.012: CIP Write-Class Service Burst Exceeding Threshold Emits T0836 Modify Parameter Finding

## Description

When `classify_cip_service(cip_header.service)` returns any write-class variant
(`SetAttributesAll` 0x02, `SetAttributeList` 0x04, or `SetAttributeSingle` 0x10), the
per-flow `write_count_in_window` counter is incremented. When the count exceeds
`enip_write_burst_threshold` within the 1-second window, a `Finding` is emitted carrying
`T0836` ("Modify Parameter"). The one-shot guard prevents more than one T0836 finding per
window per flow. This BC models CIP write-burst detection: a high rate of attribute writes in
a short window is consistent with an adversary systematically modifying PLC setpoints,
thresholds, or configuration parameters.

## Preconditions

1. `classify_cip_service(cip_header.service)` returns `CipServiceClass::SetAttributesAll`,
   `CipServiceClass::SetAttributeList`, or `CipServiceClass::SetAttributeSingle`.
2. `cip_header.service & 0x80 == 0` (request, not response).
3. `flow.is_non_enip == false`.
4. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

**Counter increment (every write-class service):**
1. `flow.write_count_in_window += 1`.
2. `EnipAnalyzer.write_count += 1` (aggregate lifetime counter).
3. If `flow.write_count_in_window == 1` (first in window): `flow.write_window_start_ts = now_ts`.

**Window expiry:**
4. If `now_ts.wrapping_sub(flow.write_window_start_ts) > 1` (1-second window expired):
   - `flow.write_count_in_window = 1` (new window, current write seeds it).
   - `flow.write_window_start_ts = now_ts`.
   - `flow.write_burst_emitted = false`.

**Finding emission (when threshold exceeded AND guard not set):**
5. When `flow.write_count_in_window > enip_write_burst_threshold`
   AND `flow.write_burst_emitted == false`
   AND window has NOT expired:
   - Push exactly ONE `Finding`:
     - `category: ThreatCategory::Execution`
     - `verdict: Verdict::Likely`
     - `confidence: Confidence::Medium`
     - `summary: "CIP write-class service burst: {count} SetAttribute operations in 1s window (threshold {threshold}) — possible parameter modification attack (T0836)"`
     - `evidence`: one entry — `"CIP service=0x{service:02X} ({service_name}) src={src_ip} ENIP session={session}"`
     - `mitre_techniques: vec!["T0836"]`
     - `source_ip: Some(...)`, `timestamp: Some(...)`
   - `flow.write_burst_emitted = true` (one-shot guard).

## Invariants

1. **Write-class services**: SetAttributesAll (0x02), SetAttributeList (0x04), SetAttributeSingle
   (0x10) are the three write services. All three increment `write_count_in_window`. No other
   service codes contribute to this counter.
2. **1-second window**: distinct from the 10-second error-rate window (BC-2.17.008). Write
   bursts are high-frequency in attacks; a 1-second window tightly bounds the detection.
3. **Default threshold**: `enip_write_burst_threshold` default = 20 (set by CLI flag
   `--enip-write-burst-threshold`, BC-2.17.023). [OA-001: human to confirm; 20 matches Modbus]
4. **T0836 is the correct v19.1 technique** [MITRE: enip-mitre-ics-tagging.md §5]:
   T0836 "Modify Parameter" (ICS Impair Process Control, TA0105). Already seeded in
   `src/mitre.rs`; no new catalog entry required.
5. **One finding per window per flow**: the `write_burst_emitted` guard prevents flooding
   `all_findings` with T0836 findings on a sustained write burst.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First write-class service on new flow | `write_count_in_window=1`; window seeded; no finding |
| EC-002 | Write count reaches threshold+1 | Finding emitted; `write_burst_emitted=true` |
| EC-003 | Additional writes after guard set (same window) | `write_count_in_window` incremented; NO additional finding |
| EC-004 | 1-second window expires; new writes arrive | Window reset; counter=1; `write_burst_emitted=false` |
| EC-005 | Mix of SetAttributeSingle and SetAttributesAll in same window | Both counted; threshold checked against combined count |
| EC-006 | `all_findings.len() == MAX_FINDINGS` when threshold crossed | No finding; guard NOT set |
| EC-007 | `enip_write_burst_threshold = 0` | First write immediately triggers finding (count=1 > 0) |

## Canonical Test Vectors

| Scenario | Threshold | write_count_in_window | Finding emitted? |
|----------|-----------|----------------------|-----------------|
| 20 SetAttributeSingle (1s window, threshold=20) | 20 | 20 | No (count=20, 20 > 20 = false) |
| 21 SetAttributeSingle | 20 | 21 | Yes — T0836, Likely/Medium |
| 21 writes then 5 more (same window) | 20 | 26 | One finding at count=21; none for 22–26 (guard) |
| 21 writes in window 1, 21 in window 2 | 20 | 21 (each) | Two findings — one per window |

**CIP SetAttributeSingle byte-level vector:**
```
ENIP cmd: 0x006F (SendRRData)
CIP item: type_id=0x00B2, service=0x10 (SetAttributeSingle), path=[Class 0x04, Instance 1, Attr 3]
```
Expected at count=21 (threshold=20): T0836 finding emitted.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-D (indirect): `classify_cip_service(0x10)` returns `SetAttributeSingle` | Kani Sub-D |
| (none) | Write-burst threshold, window, one-shot guard: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — detecting CIP write-class service bursts is required for T0836 Modify Parameter detection: adversaries use SetAttribute operations to modify PLC setpoints, configuration parameters, and thresholds to impair process control; a burst above the threshold within 1 second indicates systematic parameter manipulation |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (write window fields), Decision 9 (CLI threshold flag) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | T0836 — Modify Parameter (ICS Impair Process Control TA0105; active ics-attack-19.1; already seeded in src/mitre.rs) |

## Related BCs

- BC-2.17.007 — depends on (SetAttribute* service classification)
- BC-2.17.023 — composes with (`--enip-write-burst-threshold` CLI flag controls threshold value)
- BC-2.17.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipFlowState.write_count_in_window: u64`
- `src/analyzer/enip.rs` — `EnipFlowState.write_window_start_ts: u32`
- `src/analyzer/enip.rs` — `EnipFlowState.write_burst_emitted: bool`
- `src/analyzer/enip.rs` — `EnipAnalyzer.write_count: u64` (aggregate)
- `src/analyzer/enip.rs` — `EnipAnalyzer.enip_write_burst_threshold: u32`
- `src/mitre.rs` — `technique_info("T0836")` arm (existing; shared with Modbus)
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 4` (write window fields), §Decision 9 (CLI threshold)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-D (indirect — verifies classify_cip_service returns SetAttribute* variants)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (write window fields), Decision 9 (threshold flag); enip-mitre-ics-tagging.md §5 (T0836 SetAttribute primary mapping); src/mitre.rs (T0836 existing seeded) |
| **Confidence** | high for T0836 mapping; medium for 20/1s default threshold [OA-001: human to confirm] |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates write_count_in_window, write_window_start_ts, write_burst_emitted, write_count, all_findings |
| **Deterministic** | yes — same write sequence + threshold produces same findings |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (write-burst detection within process_pdu) |
