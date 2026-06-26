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
input-hash: "ae2d871"
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
| BC-2.17.011 | CIP Stop Service Observed Emits T0858 Change Operating Mode Finding | Core detection |
| BC-2.17.013 | CIP Reset service (0x05) emits T0816 (Device Restart/Shutdown) | Core detection |
| BC-2.17.012 | CIP write-attribute burst within 1s window emits T0836 (Modify Parameter) | Core detection with windowed threshold |

## Acceptance Criteria

### AC-135-001: CIP Stop service (0x07) emits T0858 (Change Operating Mode)
**Traces to:** BC-2.17.011 postconditions 1–2
- Given a CIP request (`service & 0x80 == 0`) with `classify_cip_service(service)` returning `CipServiceClass::Stop` (CIP service code 0x07)
- AND item `type_id == 0x00B2` (Unconnected Data Item; F-P9-001)
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding`:
  - `category: ThreatCategory::Execution`
  - `verdict: Verdict::Likely`
  - `confidence: Confidence::High`
  - `summary: "CIP Stop service observed: controller run→stop transition command (T0858)"`
  - `evidence: "CIP service=0x07 (Stop) from src={src_ip} ENIP cmd={enip_cmd:#06X} session={session_handle}"`
  - `mitre_techniques: vec!["T0858"]`
- T0858 fires per-occurrence (each CIP Stop frame generates one finding, up to MAX_FINDINGS cap)
- Does NOT fire for CIP Stop responses (0x87 — high bit set → classified as `CipServiceClass::Response`)
- Does NOT fire for `type_id == 0x00B1` items (F-P9-001 gate)
- SetAttribute services (0x02 SetAttributesAll, 0x04 SetAttributeList, 0x10 SetAttributeSingle) do NOT trigger T0858 — they feed the T0836 write-burst detection path (BC-2.17.012) instead
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_stop_service_0x07`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_stop_response_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_connected_item_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0858_set_attribute_no_t0858`

### AC-135-002: CIP Reset service (0x05) emits T0816 (Device Restart/Shutdown)
**Traces to:** BC-2.17.013 postconditions 1–2; BC-2.17.007 invariant 1
- Given a CIP request where `classify_cip_service(cip_header.service)` returns `CipServiceClass::Reset` (BC-2.17.013 precondition 1)
  - NOTE: the response-bit check (`service & 0x80 != 0`) is handled INSIDE `classify_cip_service` per BC-2.17.007 invariant 1 — do NOT use the raw predicate `service & 0x7F == 0x05` in detection logic; always route through `classify_cip_service`
- AND `type_id == 0x00B2`
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- When the analyzer processes the frame
- Then ONE `Finding` (BC-2.17.013 postcondition 1):
  - `category: ThreatCategory::Execution` (BC-2.17.013 postcondition 1 — EXACT value; NOT InhibitResponseFunction)
  - `verdict: Verdict::Likely` (BC-2.17.013 postcondition 1)
  - `confidence: Confidence::High`
  - `summary: "CIP Reset service observed: adversary-triggered device restart (T0816)"` (BC-2.17.013 postcondition 1 — EXACT string)
  - `evidence`: one entry — `"CIP service=0x05 (Reset) from src={src_ip} ENIP cmd={enip_cmd:#06X} session={session_handle}"` (BC-2.17.013 postcondition 1)
  - `mitre_techniques: vec!["T0816"]`
- T0816 fires per-occurrence (no one-shot guard — BC-2.17.013 postcondition 2)
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0816_reset_service`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0816_response_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0816_connected_item_no_finding`

### AC-135-003: CIP write-attribute burst within 1s window emits T0836 (Modify Parameter)
**Traces to:** BC-2.17.012 postconditions 1–3
- Given CIP SetAttribute requests accumulating in `flow.write_count_in_window` within a 1-second sliding window
- When `flow.write_count_in_window` strictly exceeds `self.enip_write_burst_threshold` (default 50) within 1 second
- AND `flow.write_burst_emitted == false`
- Then ONE `Finding`:
  - `category: ThreatCategory::Execution` (BC-2.17.012 postcondition 5 — EXACT value; NOT ImpairProcessControl)
  - `verdict: Verdict::Likely`
  - `confidence: Confidence::Medium`
  - `summary: "CIP write-class service burst: {count} SetAttribute operations in 1s window (threshold {threshold}) — possible parameter modification attack (T0836)"` (BC-2.17.012 postcondition 5 — EXACT format)
  - `evidence`: one entry — `"CIP service=0x{service:02X} ({service_name}) src={src_ip} ENIP session={session}"` (BC-2.17.012 postcondition 5)
  - `mitre_techniques: vec!["T0836"]`
  - `flow.write_burst_emitted = true` (one-shot guard per window)
- Window is 1 second (NOT 10 seconds like error burst); window tracks `write_window_start_ts: u32` (BC-2.17.012 postcondition 3 — `flow.write_window_start_ts = now_ts`; u32 arithmetic matches error_window_start_ts pattern)
- Strict `>` semantics: with default 50, the 51st write fires
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_burst_fires_at_threshold_plus_one`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_burst_one_shot_guard`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_no_fire_at_threshold`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_threshold_zero_fires_on_first_write`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_window_resets_after_1s`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_t0836_custom_threshold`

### AC-135-004: `is_non_enip` suppresses T0858, T0816, and T0836 detections
**Traces to:** BC-2.17.011/012/013 preconditions (is_non_enip guard)
- When `flow.is_non_enip == true`, none of the command detections emit findings
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_non_enip_suppresses_command_detections`

### AC-135-005: T0836 write count increments for each SetAttribute (feeds burst detection) and aggregate write_count increments
**Traces to:** BC-2.17.012 postconditions 1–2 (write_count accumulation; aggregate lifetime counter)
- Every SetAttribute request (SetAttributeSingle 0x10, SetAttributesAll 0x02, SetAttributeList 0x04) in a 0x00B2 item increments BOTH:
  - `flow.write_count_in_window += 1` — per-flow burst window counter (BC-2.17.012 postcondition 1)
  - `self.write_count += 1` — `EnipAnalyzer` aggregate lifetime counter (BC-2.17.012 Postcondition 2; consumed by `summarize()` per BC-2.17.021 postcondition 1 `write_count` field)
- Both increments happen on EVERY qualifying write-class request, regardless of whether the burst threshold is crossed (accumulation is separate from emission)
- SetAttribute accumulation is independent of T0858 — T0858 is triggered by CIP Stop (0x07), not by SetAttribute
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_write_count_accumulates`
- **Test:** `tests/enip_analyzer_tests.rs::command_detections::test_aggregate_write_count_increments` (verify `analyzer.write_count` after N SetAttribute frames == N)

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `EnipFlowState.write_count_in_window` | `src/analyzer/enip.rs` | `u64` — SetAttribute count in 1s window (BC-2.17.012 Architecture Anchors) |
| `EnipFlowState.write_burst_emitted` | `src/analyzer/enip.rs` | `bool` — one-shot guard for T0836 |
| `EnipFlowState.write_window_start_ts` | `src/analyzer/enip.rs` | `u32` — 1s window start timestamp (BC-2.17.012 postcondition 3/4; matches error_window_start_ts: u32 pattern) |
| `EnipAnalyzer.write_count` | `src/analyzer/enip.rs` | `u64` — aggregate lifetime write counter (BC-2.17.012 Postcondition 2; consumed by BC-2.17.021 summarize()) |
| T0858 detection | `src/analyzer/enip.rs` | `if CipServiceClass::Stop (0x07) && 0x00B2 && !is_non_enip → emit T0858` |
| T0816 detection | `src/analyzer/enip.rs` | `if classify_cip_service(service) == CipServiceClass::Reset && type_id == 0x00B2 && !is_non_enip → emit T0816` |
| T0836 detection | `src/analyzer/enip.rs` | Check/reset 1s window; increment count; if count > threshold && !guard → emit T0836` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod command_detections { ... }` |

**Detection independence note:** T0858 (Stop, 0x07) and T0836 (SetAttribute burst) are triggered by DIFFERENT CIP service codes and are fully independent detections. A CIP Stop frame emits T0858 but does NOT increment the write-burst counter. A SetAttribute frame increments the write-burst counter (and may emit T0836) but does NOT emit T0858. A single frame cannot trigger both.

**T0836 window is 1s (NOT 10s):** The write-burst window is 1 second (much shorter than the 10s error-burst window for T0888 Pattern B). The implementer must not conflate these two window durations.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CIP Stop (0x07) via 0x00B2 | T0858 finding emitted (Execution, Likely, High) |
| EC-002 | CIP Stop response (0x87, high bit set) | No T0858 — `classify_cip_service(0x87)` → `Response`; response-bit invariant |
| EC-003 | SetAttributeSingle (0x10) via 0x00B2 | No T0858 — increments write_count only; T0836 path, not T0858 |
| EC-004 | CIP Stop via 0x00B1 | No T0858 in v0.11.0 (F-P9-001 gate; 0x00B1 skipped) |
| EC-005 | Reset service (0x05) via 0x00B2 | T0816 finding emitted |
| EC-006 | Reset response (0x85) | No T0816 (only requests trigger) |
| EC-007 | `enip_write_burst_threshold=0`; first write | T0836 fires immediately (count=1 > 0) |
| EC-008 | 51 SetAttribute in 1s (threshold=50, strict >) | T0836 finding; guard=true |
| EC-009 | 52nd SetAttribute in same window | No additional T0836 (one-shot guard) |
| EC-010 | 1s window expires; 51 new writes | New window; T0836 can fire again |
| EC-011 | `is_non_enip=true`; CIP Stop | No T0858; `is_non_enip` guard applied first |
| EC-012 | `all_findings` at MAX_FINDINGS; T0816 arrives | No finding (cap guard) |

## Tasks

- [ ] Add to `EnipFlowState`: `write_count_in_window: u64`, `write_burst_emitted: bool`, `write_window_start_ts: u32` (BC-2.17.012 Architecture Anchors; use exact field names; write_window_start_ts is u32 seconds NOT milliseconds)
- [ ] Add `write_count: u64` field to `EnipAnalyzer` struct (aggregate lifetime counter; BC-2.17.012 Postcondition 2; BC-2.17.021 Architecture Anchors `EnipAnalyzer.write_count: u64`; feeds summarize())
- [ ] In `process_pdu`, for CIP Stop (`CipServiceClass::Stop`, service 0x07) requests via 0x00B2 and !is_non_enip:
  - Emit T0858 finding (per-occurrence, guarded by MAX_FINDINGS); category=Execution, verdict=Likely, confidence=High
- [ ] In `process_pdu`, for SetAttribute (SetAttributeSingle 0x10, SetAttributesAll 0x02, SetAttributeList 0x04) CIP requests via 0x00B2:
  - `self.write_count += 1` (BC-2.17.012 Postcondition 2; aggregate — separate from per-flow burst counter)
  - Check/reset 1s write window via `write_window_start_ts: u32`; increment `write_count_in_window`; if `count > threshold && !write_burst_emitted` → emit T0836; set guard
- [ ] In `process_pdu`, for CIP Reset (where `classify_cip_service(service) == CipServiceClass::Reset`) requests via 0x00B2 and `!is_non_enip`: emit T0816 finding (per-occurrence; category=Execution, verdict=Likely, confidence=High, summary="CIP Reset service observed: adversary-triggered device restart (T0816)")
- [ ] Add `mod command_detections { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-135 tests
- [ ] Construct minimal ENIP+CPF+CIP byte sequences for each test (reuse helpers from STORY-134 tests)
- [ ] Run `cargo test enip` — all command_detections tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod command_detections { ... }`

```
command_detections::test_t0858_stop_service_0x07
command_detections::test_t0858_stop_response_no_finding
command_detections::test_t0858_connected_item_no_finding
command_detections::test_t0858_set_attribute_no_t0858
command_detections::test_t0816_reset_service
command_detections::test_t0816_response_no_finding
command_detections::test_t0816_connected_item_no_finding
command_detections::test_t0836_burst_fires_at_threshold_plus_one
command_detections::test_t0836_burst_one_shot_guard
command_detections::test_t0836_no_fire_at_threshold
command_detections::test_t0836_threshold_zero_fires_on_first_write
command_detections::test_t0836_window_resets_after_1s
command_detections::test_t0836_custom_threshold
command_detections::test_non_enip_suppresses_command_detections
command_detections::test_write_count_accumulates
command_detections::test_aggregate_write_count_increments
```

## Previous Story Intelligence

- STORY-132 provides `classify_cip_service`, `CipServiceClass::Stop` (T0858 trigger), `CipServiceClass::Reset` (T0816 trigger), `CipServiceClass::SetAttributeSingle`/`SetAttributesAll`/`SetAttributeList` (T0836 write-count trigger)
- STORY-133 provides T0858 and T0816 in `technique_info()` — required for MITRE metadata validation
- STORY-134 demonstrates the per-occurrence vs. one-shot pattern and the `is_non_enip` guard — use the same guard structure here
- T0836 uses a 1s window; T0888 Pattern B uses a 10s window. The implementer must use separate window duration constants; never share `error_window_start_ts` with `write_window_start_ts`. Both are `u32` and use `wrapping_sub` arithmetic.

## Architecture Compliance Rules

1. **T0858 and T0836 are triggered by DIFFERENT services (BC-2.17.011/012):** T0858 fires when `CipServiceClass::Stop` (0x07) is observed — per-occurrence, no guard (BC-2.17.011). T0836 fires when the SetAttribute write count exceeds the burst threshold within 1s (BC-2.17.012). A CIP Stop frame triggers T0858 only. A SetAttribute frame increments the write counter and may trigger T0836. These two detection paths are mutually exclusive per frame.
2. **T0816 uses `classify_cip_service`, not raw byte predicate (BC-2.17.013 + BC-2.17.007 invariant 1):** T0816 is triggered when `classify_cip_service(cip_header.service)` returns `CipServiceClass::Reset` — the response-bit masking is handled inside `classify_cip_service`. Do NOT use `service & 0x7F == 0x05` directly in detection logic; this bypasses the classifier and violates BC-2.17.007 invariant 1. The ENIP command for the enclosing frame is typically SendRRData (0x006F). The detection is at the CIP layer inside a CPF 0x00B2 item.
3. **Strict `>` for T0836 (BC-2.17.012 Invariant 2):** `write_count_in_window > enip_write_burst_threshold`. Default threshold=50; the 51st write fires. Same convention as T0888 Pattern B.
4. **1s write-burst window (BC-2.17.012 postcondition 4):** `write_window_start_ts: u32` tracks the start of the current 1-second window using u32 timestamps (same type as `error_window_start_ts`; use wrapping_sub arithmetic). On PDU arrival: if `now_ts.wrapping_sub(flow.write_window_start_ts) > 1` (window expired), reset `write_count_in_window = 1` (current write seeds new window), `write_window_start_ts = now_ts`, `write_burst_emitted = false`. On first write (count becomes 1): seed `flow.write_window_start_ts = now_ts` (BC-2.17.012 postcondition 3). Field name is `write_window_start_ts` — NOT `write_window_start` or `Option<u64>`.
5. **F-P9-001 gate applies to T0858 and T0816 (BC-2.17.011/013):** Only 0x00B2 items trigger CIP service detection. 0x00B1 items are skipped.
6. **`is_non_enip` is checked first (same as STORY-134 pattern).**

## Library & Framework Requirements

No new external crate dependencies. `std::time` or a timestamp integer (`u32` seconds, `now_ts`) for window tracking. Window START timestamps (`write_window_start_ts`, `error_window_start_ts`) use `u32` seconds with wrapping_sub arithmetic — NOT milliseconds. The window-tracking arithmetic is `>1` (write burst, 1s) and `>10` (error burst, 10s) using `u32` second-resolution values. Note: `write_count_in_window` is `u64` (a count of write-class requests, unbounded in principle); only the window START timestamp field (`write_window_start_ts`) is `u32`. Do not conflate the timestamp type with the counter type.

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `EnipFlowState` write-burst fields; implement T0858/T0816/T0836 detection in `process_pdu`
- `tests/enip_analyzer_tests.rs` — add `mod command_detections { ... }` block

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` additions | ~350 |
| `tests/enip_analyzer_tests.rs` command_detections mod (16 tests) | ~650 |
| **Total** | **~950** |

## Dependency Rationale

Wave 60; depends on STORY-132 (CIP service classification) and STORY-133 (T0858/T0816 in technique catalog). STORY-135 is parallel to STORY-134 (no mutual dependency at the code level). Both STORY-134 and STORY-135 add fields to `EnipFlowState` — if implemented simultaneously, merge conflicts in `EnipFlowState` must be resolved.
