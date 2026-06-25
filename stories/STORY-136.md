---
document_type: story
story_id: STORY-136
title: "ENIP Connection Lifecycle: ForwardOpen/ForwardClose Detection"
epic_id: E-20
wave: 60
points: 5
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-17]
target_module: analyzer/enip
depends_on: [STORY-132, STORY-133]
behavioral_contracts:
  - BC-2.17.015
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.015.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
input-hash: "2af89b5"
---

# STORY-136: ENIP Connection Lifecycle: ForwardOpen/ForwardClose Detection

## Narrative

**As a** security analyst reviewing EtherNet/IP traffic for anomalies,
**I want** wirerust to detect and record ForwardOpen and ForwardClose CIP connection management
events,
**so that** operators can see when CIP implicit I/O connections are being established or torn
down — even though these operations are not currently mapped to MITRE ICS techniques in v0.11.0.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.015 | ForwardOpen/ForwardClose CIP services detected and recorded as findings | Core implementation |

## Acceptance Criteria

### AC-136-001: ForwardOpen and LargeForwardOpen requests each emit one anomaly finding (no MITRE technique)
**Traces to:** BC-2.17.015 postconditions 1–3, invariant 5
- Given a CIP request with `classify_cip_service(service)` returning `CipServiceClass::ForwardOpen` (service byte `0x54`) **or** `CipServiceClass::LargeForwardOpen` (service byte `0x5B`)
- AND `type_id == 0x00B2`
- AND `cip_header.service & 0x80 == 0` (request, not response)
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding`:
  - `category: ThreatCategory::Anomaly`
  - `verdict: Verdict::Possible`
  - `confidence: Confidence::Low`
  - `summary: "CIP ForwardOpen connection establishment observed from src={src_ip}: connection lifecycle anomaly"`
  - `mitre_techniques: vec![]` (empty — no MITRE ICS technique for CIP connection establishment anomaly per ADR-010 Decision 7)
  - `source_ip: Some(src_ip)`, `timestamp: Some(timestamp)`
- ForwardOpen and LargeForwardOpen each fire per-occurrence (no one-shot guard per BC-2.17.015 Postcondition 3)
- Connection serial number is recorded as `0` in v0.11.0 (deferred per BC-2.17.015 Postcondition 2 / ADR-010 Decision 8)
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_open_emits_finding`
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_open_no_mitre_technique`
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_open_connected_item_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_large_forward_open_emits_finding`

### AC-136-002: ForwardClose request emits one anomaly finding (no MITRE technique)
**Traces to:** BC-2.17.015 postconditions 4–5
- Given a CIP request with `classify_cip_service(service)` returning `CipServiceClass::ForwardClose` (service byte `0x4E`)
- AND `type_id == 0x00B2`
- AND `cip_header.service & 0x80 == 0` (request, not response)
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding`:
  - `category: ThreatCategory::Anomaly`
  - `verdict: Verdict::Possible`
  - `confidence: Confidence::Low`
  - `summary: "CIP ForwardClose connection teardown observed from src={src_ip}: connection lifecycle closed"`
  - `mitre_techniques: vec![]` (empty — no MITRE ICS technique per ADR-010 Decision 7)
  - `source_ip: Some(src_ip)`, `timestamp: Some(timestamp)`
- ForwardClose fires per-occurrence (no one-shot guard per BC-2.17.015 Postcondition 5)
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_close_emits_finding`
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_close_no_mitre_technique`

### AC-136-003: ForwardOpen/ForwardClose responses are not detected (requests only)
**Traces to:** BC-2.17.015 Invariant 2 (request-only detection)
- CIP response service bytes `0xD4` (ForwardOpen response) and `0xCE` (ForwardClose response) do NOT emit findings
- Only request frames (`service & 0x80 == 0`) trigger connection lifecycle detection
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_open_response_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_close_response_no_finding`

### AC-136-004: `is_non_enip` suppresses ForwardOpen/ForwardClose detection
**Traces to:** BC-2.17.015 preconditions
- When `flow.is_non_enip == true`, no ForwardOpen or ForwardClose findings are emitted
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_non_enip_suppresses_connection_lifecycle`

### AC-136-005: `open_connection_count` and `close_connection_count` tracked in flow state
**Traces to:** BC-2.17.015 Invariant 3 (connection count tracking for session summary)
- `EnipFlowState.open_connection_count: u32` increments on each ForwardOpen request
- `EnipFlowState.close_connection_count: u32` increments on each ForwardClose request
- These counts feed the session summary in STORY-138 (BC-2.17.025)
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_connection_counts_tracked`

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `CipServiceClass::ForwardOpen` | `src/analyzer/enip.rs` | From STORY-130; service 0x54 |
| `CipServiceClass::LargeForwardOpen` | `src/analyzer/enip.rs` | From STORY-130; service 0x5B — treated identically to ForwardOpen per BC-2.17.015 Invariant 5 |
| `CipServiceClass::ForwardClose` | `src/analyzer/enip.rs` | From STORY-130; service 0x4E |
| `EnipFlowState.open_connection_count` | `src/analyzer/enip.rs` | `u32` — ForwardOpen + LargeForwardOpen request count |
| `EnipFlowState.close_connection_count` | `src/analyzer/enip.rs` | `u32` — ForwardClose request count |
| ForwardOpen/LargeForwardOpen/Close detection | `src/analyzer/enip.rs` | `if matches!(service_class, ForwardOpen \| LargeForwardOpen \| ForwardClose) && 0x00B2 && !is_non_enip → emit Anomaly/Possible/Low + increment count` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod connection_lifecycle { ... }` |

**`mitre_techniques: vec![]` is intentional (BC-2.17.015 Invariant 1):** ATT&CK for ICS v19.1 has no technique specifically for CIP connection establishment anomaly. The gap is documented in ADR-010 Decision 7. The `vec![]` is a deliberate design choice, not an omission. Findings carry `category: ThreatCategory::Anomaly`, `verdict: Verdict::Possible`, `confidence: Confidence::Low` per BC-2.17.015 Postconditions 1 and 4.

**LargeForwardOpen (0x5B) treated identically to ForwardOpen (0x54):** Per BC-2.17.015 Invariant 5, LargeForwardOpen is detected with the same finding fields and the same Anomaly/Possible/Low classification. Full payload parse is deferred (ADR-010 Decision 8). The `open_connection_count` increment covers both ForwardOpen and LargeForwardOpen requests.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ForwardOpen request (0x54) via 0x00B2 | Anomaly/Possible/Low finding emitted; `open_connection_count += 1` |
| EC-002 | LargeForwardOpen request (0x5B) via 0x00B2 | Anomaly/Possible/Low finding emitted; `open_connection_count += 1` (same as ForwardOpen per BC-2.17.015 Inv 5) |
| EC-003 | ForwardClose request (0x4E) via 0x00B2 | Anomaly/Possible/Low finding emitted; `close_connection_count += 1` |
| EC-004 | ForwardOpen response (0xD4) | No finding (response bit set; `0xD4 & 0x80 != 0`) |
| EC-005 | ForwardClose response (0xCE) | No finding |
| EC-006 | ForwardOpen via 0x00B1 item | No finding (F-P9-001 — 0x00B1 deferral; but note 0x00B2 is CIP protocol requirement for ForwardOpen, so this is double-guarded) |
| EC-007 | `is_non_enip=true`; ForwardOpen | No finding |
| EC-008 | `all_findings` at MAX_FINDINGS; ForwardOpen | No finding (cap guard); `open_connection_count` still increments |
| EC-009 | 5 consecutive ForwardOpen requests | 5 findings (per-occurrence per BC-2.17.015 Post 3); `open_connection_count = 5` |
| EC-010 | Connection serial number in ForwardOpen payload | Recorded as 0 in v0.11.0 — full payload parse deferred per ADR-010 Decision 8 |

## Tasks

- [ ] Add to `EnipFlowState`: `open_connection_count: u32`, `close_connection_count: u32`
- [ ] In `process_pdu`, for `matches!(service_class, CipServiceClass::ForwardOpen | CipServiceClass::LargeForwardOpen)` requests via 0x00B2 and `!is_non_enip`:
  - Emit Anomaly/Possible/Low finding with `summary: "CIP ForwardOpen connection establishment observed from src={src_ip}: connection lifecycle anomaly"` and `mitre_techniques: vec![]`
  - Increment `open_connection_count` (regardless of MAX_FINDINGS cap on finding)
- [ ] In `process_pdu`, for `CipServiceClass::ForwardClose` (0x4E) requests via 0x00B2 and `!is_non_enip`:
  - Emit Anomaly/Possible/Low finding with `summary: "CIP ForwardClose connection teardown observed from src={src_ip}: connection lifecycle closed"` and `mitre_techniques: vec![]`
  - Increment `close_connection_count` (regardless of MAX_FINDINGS cap on finding)
- [ ] Add `mod connection_lifecycle { ... }` test wrapper to `tests/enip_analyzer_tests.rs`
- [ ] Add test `test_large_forward_open_emits_finding` in `mod connection_lifecycle`
- [ ] Run `cargo test enip` — all connection_lifecycle tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod connection_lifecycle { ... }`

```
connection_lifecycle::test_forward_open_emits_finding
connection_lifecycle::test_forward_open_no_mitre_technique
connection_lifecycle::test_forward_open_connected_item_no_finding
connection_lifecycle::test_large_forward_open_emits_finding
connection_lifecycle::test_forward_close_emits_finding
connection_lifecycle::test_forward_close_no_mitre_technique
connection_lifecycle::test_forward_open_response_no_finding
connection_lifecycle::test_forward_close_response_no_finding
connection_lifecycle::test_non_enip_suppresses_connection_lifecycle
connection_lifecycle::test_connection_counts_tracked
```

## Previous Story Intelligence

- STORY-130 defines `CipServiceClass::ForwardOpen` (0x54 & 0x7F) and `::ForwardClose` (0x4E & 0x7F) — available without further modification
- STORY-132 provides `parse_cpf_items` and `parse_cip_header` used by the `process_pdu` path
- STORY-133's MITRE seeding is needed as a wave prereq even though ForwardOpen/Close use `vec![]` — the overall MITRE consistency tests check all emitted technique IDs; having an empty `vec![]` passes these checks
- STORY-138 (Wave 61) reads `open_connection_count` and `close_connection_count` from `EnipFlowState` for the session summary — these counts must be tracked even when the MAX_FINDINGS cap is hit

## Architecture Compliance Rules

1. **Finding fields are Anomaly/Possible/Low (BC-2.17.015 Postconditions 1, 4):** `category: ThreatCategory::Anomaly`, `verdict: Verdict::Possible`, `confidence: Confidence::Low`. NOT Operational/Informational/Benign/High. Do not alter these fields.
2. **`mitre_techniques: vec![]` is explicitly correct (ADR-010 Decision 7 / BC-2.17.015 Invariant 1):** Do not add T0858, T1692.001, or any other technique to ForwardOpen/LargeForwardOpen/Close findings. The empty vec is the spec, not a placeholder.
3. **LargeForwardOpen (0x5B) is NOT omitted (BC-2.17.015 Invariant 5):** `CipServiceClass::LargeForwardOpen` (0x5B) is detected identically to ForwardOpen (0x54). Both increment `open_connection_count`. The match arm MUST include all three: `ForwardOpen | LargeForwardOpen | ForwardClose`.
4. **Count increments regardless of MAX_FINDINGS cap:** The `open_connection_count` and `close_connection_count` must increment even if `all_findings.len() >= MAX_FINDINGS` prevents finding emission. The session summary (STORY-138) needs accurate counts.
5. **F-P9-001 gate (0x00B2 only):** Only 0x00B2 items trigger ForwardOpen/LargeForwardOpen/Close detection. This is a CIP protocol requirement — these are unconnected CIP messages and must ride in 0x00B2 items.
6. **Request-only detection:** Only frames with `service & 0x80 == 0` are detected. Responses (0xD4, 0xCE) are silently passed through.
7. **Connection serial number = 0 in v0.11.0 (BC-2.17.015 Postcondition 2 / ADR-010 Decision 8):** Do not attempt to extract the serial from the payload. Record as 0 in all findings.

## Library & Framework Requirements

No new external crate dependencies.

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `EnipFlowState` connection count fields; implement ForwardOpen/Close detection
- `tests/enip_analyzer_tests.rs` — add `mod connection_lifecycle { ... }` block

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` additions | ~200 |
| `tests/enip_analyzer_tests.rs` connection_lifecycle mod (9 tests) | ~350 |
| **Total** | **~550** |

## Dependency Rationale

Wave 60; depends on STORY-132 (CIP service classification) and STORY-133 (MITRE consistency). Parallel with STORY-134 and STORY-135. STORY-138 (Wave 61) reads the connection counts populated here.
