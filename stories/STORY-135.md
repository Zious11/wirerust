---
document_type: story
story_id: STORY-135
title: "ENIP Command Detections: T0858 Mode Change, T0816 Device Reset, and T0836 Write-Attribute Burst"
epic_id: E-20
wave: 60
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-17]
target_module: analyzer/enip
depends_on: [STORY-132, STORY-133]
behavioral_contracts:
  - BC-2.17.011
  - BC-2.17.013
  - BC-2.17.012
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.011.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.013.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.012.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
input-hash: "4ba6bb1"
---

# STORY-135: ENIP Command Detections: T0858 Mode Change, T0816 Device Reset, and T0836 Write-Attribute Burst

## Narrative

**As a** security analyst monitoring industrial control networks,
**I want** wirerust to detect CIP-layer operating mode changes (T0858), device reset commands (T0816),
and suspicious write-attribute bursts (T0836),
**so that** adversary attempts to alter PLC behavior, restart devices, or rapidly modify parameters
via EtherNet/IP are detected and reported with appropriate MITRE ICS technique tagging.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.011 | CIP SetAttribute request emits T0858 (Change Operating Mode) | Core detection |
| BC-2.17.013 | CIP Reset service (0x05) emits T0816 (Device Restart/Shutdown) | Core detection |
| BC-2.17.012 | CIP write-attribute burst within 1s window emits T0836 (Modify Parameter) | Core detection with windowed threshold |

## Acceptance Criteria

### AC-135-001: CIP SetAttribute request emits T0858 (Change Operating Mode)
**Traces to:** BC-2.17.011 postconditions 1–2
- Given a CIP request (`service & 0x80 == 0`) with `classify_cip_service(service)` returning `SetAttributeSingle` (0x10) or `SetAttributeList` (0x02)
- AND item `type_id == 0x00B2` (Unconnected Data Item; F-P9-001)
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding`:
  - `category: ThreatCategory::ImpairProcessControl`
  - `verdict: Verdict::Likely`
  - `confidence: Confidence::High`
  - `summary: "CIP SetAttribute request: potential operating mode change (T0858)"`
  - `evidence: "CIP service=0x{service:02X} ({name}) src={src_ip}"`
  - `mitre_techniques: vec!["T0858"]`
- T0858 fires per-occurrence (not windowed, not one-shot)
- Does NOT fire for `type_id == 0x00B1` items
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_set_attribute_single`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_set_attribute_list`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_connected_item_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_response_no_finding`

### AC-135-002: CIP Reset service (0x05) emits T0816 (Device Restart/Shutdown)
**Traces to:** BC-2.17.013 postconditions 1–2
- Given a CIP request with `service & 0x7F == 0x05` (Reset service)
- AND `type_id == 0x00B2`
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding`:
  - `category: ThreatCategory::InhibitResponseFunction` (or equivalent ICS impair category)
  - `verdict: Verdict::Confirmed`
  - `confidence: Confidence::High`
  - `summary: "CIP Reset service (0x05): device restart/shutdown command (T0816)"`
  - `mitre_techniques: vec!["T0816"]`
- T0816 fires per-occurrence
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0816_reset_service`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0816_response_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0816_connected_item_no_finding`

### AC-135-003: CIP write-attribute burst within 1s window emits T0836 (Modify Parameter)
**Traces to:** BC-2.17.012 postconditions 1–3
- Given CIP SetAttribute requests accumulating in `flow.write_count_in_window` within a 1-second sliding window
- When `flow.write_count_in_window` strictly exceeds `self.enip_write_burst_threshold` (default 50) within 1 second
- AND `flow.write_burst_emitted == false`
- Then ONE `Finding`:
  - `category: ThreatCategory::ImpairProcessControl`
  - `verdict: Verdict::Likely`
  - `confidence: Confidence::Medium`
  - `summary: "CIP write-attribute burst: {N} SetAttribute requests in 1s — possible mass parameter modification (T0836)"`
  - `mitre_techniques: vec!["T0836"]`
  - `flow.write_burst_emitted = true` (one-shot guard per window)
- Window is 1 second (NOT 10 seconds like error burst); window tracks `write_window_start`
- Strict `>` semantics: with default 50, the 51st write fires
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_burst_fires_at_threshold_plus_one`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_burst_one_shot_guard`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_no_fire_at_threshold`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_window_resets_after_1s`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_custom_threshold`

### AC-135-004: `is_non_enip` suppresses T0858, T0816, and T0836 detections
**Traces to:** BC-2.17.011/012/013 preconditions (is_non_enip guard)
- When `flow.is_non_enip == true`, none of the command detections emit findings
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_non_enip_suppresses_command_detections`

### AC-135-005: T0836 write count also increments for each SetAttribute (feeds burst detection)
**Traces to:** BC-2.17.012 preconditions (write_count accumulation)
- Every SetAttribute request in a 0x00B2 item increments `flow.write_count_in_window`
- This happens regardless of whether the burst threshold is crossed (accumulation is separate from emission)
- Accumulation also triggers the T0858 per-occurrence detection (both T0858 and T0836 logic share the SetAttribute trigger path)
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_write_count_accumulates`

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `EnipFlowState.write_count_in_window` | `src/analyzer/enip.rs` | `u32` — SetAttribute count in 1s window |
| `EnipFlowState.write_burst_emitted` | `src/analyzer/enip.rs` | `bool` — one-shot guard for T0836 |
| `EnipFlowState.write_window_start` | `src/analyzer/enip.rs` | `Option<u64>` — 1s window start (millis) |
| T0858 detection | `src/analyzer/enip.rs` | `if SetAttribute && 0x00B2 && !is_non_enip → emit T0858` |
| T0816 detection | `src/analyzer/enip.rs` | `if service & 0x7F == 0x05 && 0x00B2 && !is_non_enip → emit T0816` |
| T0836 detection | `src/analyzer/enip.rs` | Check/reset 1s window; increment count; if count > threshold && !guard → emit T0836` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod command_detections { ... }` |

**Detection precedence note:** When a SetAttribute request arrives, the analyzer runs BOTH T0858 (per-occurrence) and the T0836 accumulation in sequence. A single frame can contribute to T0836's count AND emit a T0858 finding. These are independent detections — both run.

**T0836 window is 1s (NOT 10s):** The write-burst window is 1 second (much shorter than the 10s error-burst window for T0888 Pattern B). The implementer must not conflate these two window durations.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SetAttributeSingle (0x10) via 0x00B2 | T0858 finding emitted |
| EC-002 | SetAttributeList (0x02) via 0x00B2 | T0858 finding emitted |
| EC-003 | SetAttribute via 0x00B1 | No T0858 (F-P9-001 gate) |
| EC-004 | SetAttribute response (0x90 or 0x82) | No T0858 (only requests trigger) |
| EC-005 | Reset service (0x05) via 0x00B2 | T0816 finding emitted |
| EC-006 | Reset response (0x85) | No T0816 (only requests trigger) |
| EC-007 | 50 SetAttribute in 1s (threshold=50, strict >) | No T0836 finding |
| EC-008 | 51 SetAttribute in 1s (threshold=50, strict >) | T0836 finding; guard=true |
| EC-009 | 52nd SetAttribute in same window | No additional T0836 (one-shot guard) |
| EC-010 | 1s window expires; 51 new writes | New window; T0836 can fire again |
| EC-011 | `is_non_enip=true`; SetAttribute | No T0858, no T0836 |
| EC-012 | `all_findings` at MAX_FINDINGS; T0816 arrives | No finding (cap guard) |

## Tasks

- [ ] Add to `EnipFlowState`: `write_count_in_window: u32`, `write_burst_emitted: bool`, `write_window_start: Option<u64>`
- [ ] In `process_pdu`, for SetAttribute (SetAttributeSingle OR SetAttributeList) CIP requests via 0x00B2:
  - Emit T0858 finding (per-occurrence, guarded by MAX_FINDINGS and is_non_enip)
  - Check/reset 1s write window; increment `write_count_in_window`; if `count > threshold && !write_burst_emitted` → emit T0836; set guard
- [ ] In `process_pdu`, for CIP service `service & 0x7F == 0x05` (Reset) requests via 0x00B2: emit T0816 finding (per-occurrence)
- [ ] Add `mod command_detections { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-135 tests
- [ ] Construct minimal ENIP+CPF+CIP byte sequences for each test (reuse helpers from STORY-134 tests)
- [ ] Run `cargo test enip` — all command_detections tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod command_detections { ... }`

```
command_detections::test_t0858_set_attribute_single
command_detections::test_t0858_set_attribute_list
command_detections::test_t0858_connected_item_no_finding
command_detections::test_t0858_response_no_finding
command_detections::test_t0816_reset_service
command_detections::test_t0816_response_no_finding
command_detections::test_t0816_connected_item_no_finding
command_detections::test_t0836_burst_fires_at_threshold_plus_one
command_detections::test_t0836_burst_one_shot_guard
command_detections::test_t0836_no_fire_at_threshold
command_detections::test_t0836_window_resets_after_1s
command_detections::test_t0836_custom_threshold
command_detections::test_non_enip_suppresses_command_detections
command_detections::test_write_count_accumulates
```

## Previous Story Intelligence

- STORY-132 provides `classify_cip_service`, `CipServiceClass::SetAttributeSingle`, `CipServiceClass::SetAttributeList` — used for T0858 and T0836 detection trigger
- STORY-133 provides T0858 and T0816 in `technique_info()` — required for MITRE metadata validation
- STORY-134 demonstrates the per-occurrence vs. one-shot pattern and the `is_non_enip` guard — use the same guard structure here
- T0836 uses a 1s window; T0888 Pattern B uses a 10s window. The implementer must use separate window duration constants; never share `error_window_start` with `write_window_start`.

## Architecture Compliance Rules

1. **T0858 and T0836 share the SetAttribute trigger (BC-2.17.011/012):** Both fire when a SetAttribute request arrives. T0858 fires per-occurrence (no guard). T0836 accumulates and fires once-per-window when the burst threshold is crossed. A single SetAttribute frame increments the write count AND may emit a T0858 finding.
2. **T0816 is CIP service byte only, not ENIP command (BC-2.17.013):** T0816 is triggered by CIP service code `0x05` inside a CPF 0x00B2 item — NOT by an ENIP header command. The ENIP command for the enclosing frame is typically SendRRData (0x0072). The detection is at the CIP layer.
3. **Strict `>` for T0836 (BC-2.17.012 Invariant 2):** `write_count_in_window > enip_write_burst_threshold`. Default threshold=50; the 51st write fires. Same convention as T0888 Pattern B.
4. **1s write-burst window (BC-2.17.012 Invariant 3):** `write_window_start` tracks the start of the current 1-second window. On PDU arrival: if `now - write_window_start > 1s`, reset count to 0, reset `write_burst_emitted = false`, set `write_window_start = now`.
5. **F-P9-001 gate applies to T0858 and T0816 (BC-2.17.011/013):** Only 0x00B2 items trigger CIP service detection. 0x00B1 items are skipped.
6. **`is_non_enip` is checked first (same as STORY-134 pattern).**

## Library & Framework Requirements

No new external crate dependencies. `std::time` or a timestamp integer (u64 millis) for window tracking.

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `EnipFlowState` write-burst fields; implement T0858/T0816/T0836 detection in `process_pdu`
- `tests/enip_analyzer_tests.rs` — add `mod command_detections { ... }` block

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` additions | ~350 |
| `tests/enip_analyzer_tests.rs` command_detections mod (14 tests) | ~600 |
| **Total** | **~950** |

## Dependency Rationale

Wave 60; depends on STORY-132 (CIP service classification) and STORY-133 (T0858/T0816 in technique catalog). STORY-135 is parallel to STORY-134 (no mutual dependency at the code level). Both STORY-134 and STORY-135 add fields to `EnipFlowState` — if implemented simultaneously, merge conflicts in `EnipFlowState` must be resolved.
