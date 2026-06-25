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

### AC-136-001: ForwardOpen request emits an informational finding (no MITRE technique)
**Traces to:** BC-2.17.015 postconditions 1–2
- Given a CIP request with `classify_cip_service(service)` returning `CipServiceClass::ForwardOpen` (service byte `0x54`)
- AND `type_id == 0x00B2`
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding`:
  - `category: ThreatCategory::Operational` (or `ThreatCategory::Informational` — whichever is correct for the project's existing ThreatCategory enum)
  - `verdict: Verdict::Benign` (connection establishment is normal behavior; flagged for visibility)
  - `confidence: Confidence::High`
  - `summary: "CIP ForwardOpen: implicit I/O connection established src={src_ip}"`
  - `mitre_techniques: vec![]` (no MITRE technique — explicit design decision per ADR-010 Decision 7)
  - `source_ip: Some(src_ip)`, `timestamp: Some(timestamp)`
- ForwardOpen fires per-occurrence
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_open_emits_finding`
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_open_no_mitre_technique`
- **Test:** `tests/enip_analyzer_tests.rs::connection_lifecycle::test_forward_open_connected_item_no_finding`

### AC-136-002: ForwardClose request emits an informational finding (no MITRE technique)
**Traces to:** BC-2.17.015 postconditions 3–4
- Given a CIP request with `classify_cip_service(service)` returning `CipServiceClass::ForwardClose` (service byte `0x4E`)
- AND `type_id == 0x00B2`
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding`:
  - Same structure as ForwardOpen finding but with summary: `"CIP ForwardClose: implicit I/O connection torn down src={src_ip}"`
  - `mitre_techniques: vec![]`
- ForwardClose fires per-occurrence
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
| `CipServiceClass::ForwardOpen` | `src/analyzer/enip.rs` | From STORY-130; service 0x54 & 0x7F |
| `CipServiceClass::ForwardClose` | `src/analyzer/enip.rs` | From STORY-130; service 0x4E & 0x7F |
| `EnipFlowState.open_connection_count` | `src/analyzer/enip.rs` | `u32` — ForwardOpen request count |
| `EnipFlowState.close_connection_count` | `src/analyzer/enip.rs` | `u32` — ForwardClose request count |
| ForwardOpen/Close detection | `src/analyzer/enip.rs` | `if ForwardOpen/Close && 0x00B2 && !is_non_enip → emit + increment count` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod connection_lifecycle { ... }` |

**`mitre_techniques: vec![]` is intentional:** ADR-010 Decision 7 explicitly places ForwardOpen/ForwardClose in a "no MITRE technique gap" category for v0.11.0. These events are detected and logged as operational/informational findings, but no MITRE ICS technique is currently mapped to CIP connection management in the MITRE ICS ATT&CK framework at the level of specificity needed. The `vec![]` is a deliberate design choice, not an omission.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ForwardOpen request (0x54) via 0x00B2 | Finding emitted; `open_connection_count += 1` |
| EC-002 | ForwardClose request (0x4E) via 0x00B2 | Finding emitted; `close_connection_count += 1` |
| EC-003 | ForwardOpen response (0xD4) | No finding |
| EC-004 | ForwardClose response (0xCE) | No finding |
| EC-005 | ForwardOpen via 0x00B1 item | No finding (F-P9-001) |
| EC-006 | `is_non_enip=true`; ForwardOpen | No finding |
| EC-007 | `all_findings` at MAX_FINDINGS; ForwardOpen | No finding (cap guard); count still increments |
| EC-008 | 5 consecutive ForwardOpen requests | 5 findings (per-occurrence); `open_connection_count = 5` |

## Tasks

- [ ] Add to `EnipFlowState`: `open_connection_count: u32`, `close_connection_count: u32`
- [ ] In `process_pdu`, for `CipServiceClass::ForwardOpen` and `::ForwardClose` requests via 0x00B2 and `!is_non_enip`:
  - Emit finding with `mitre_techniques: vec![]`
  - Increment `open_connection_count` or `close_connection_count` (regardless of MAX_FINDINGS cap on finding)
- [ ] Add `mod connection_lifecycle { ... }` test wrapper to `tests/enip_analyzer_tests.rs`
- [ ] Run `cargo test enip` — all connection_lifecycle tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod connection_lifecycle { ... }`

```
connection_lifecycle::test_forward_open_emits_finding
connection_lifecycle::test_forward_open_no_mitre_technique
connection_lifecycle::test_forward_open_connected_item_no_finding
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

1. **`mitre_techniques: vec![]` is explicitly correct (ADR-010 Decision 7):** Do not add T0858 or any other technique to ForwardOpen/Close findings. The empty vec is the spec, not a placeholder.
2. **Count increments regardless of MAX_FINDINGS cap:** The `open_connection_count` and `close_connection_count` must increment even if `all_findings.len() >= MAX_FINDINGS` prevents finding emission. The session summary (STORY-138) needs accurate counts.
3. **F-P9-001 gate (0x00B2 only):** Same as all other CIP-layer detections — only 0x00B2 items trigger ForwardOpen/Close detection.
4. **Request-only detection:** Only frames with `service & 0x80 == 0` are detected. Responses (0xD4, 0xCE) are silently passed through.

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
