# Demo Evidence Report — STORY-135

**Story:** ENIP Command Detections: T0858 Mode Change, T0816 Device Reset, and T0836 Write-Attribute Burst
**Story ID:** STORY-135
**Wave:** 60
**Product type:** Pure-core library (no CLI/UI surface — EtherNet/IP analyzer command detection only)
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)
**Recorded:** 2026-06-25
**Test result at recording time:** 16 passed / 0 failed / 0 ignored (mod command_detections)

---

## AC Coverage Map

| AC | Title | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|-----------------|---------------|----------------|------|
| AC-135-001 | CIP Stop (0x07) emits T0858 (Change Operating Mode) | `command_detections::test_t0858` | `AC-001-t0858-cip-stop.gif` | `AC-001-t0858-cip-stop.webm` | `AC-001-t0858-cip-stop.tape` |
| AC-135-002 | CIP Reset (0x05) emits T0816 (Device Restart/Shutdown) | `command_detections::test_t0816` | `AC-002-t0816-cip-reset.gif` | `AC-002-t0816-cip-reset.webm` | `AC-002-t0816-cip-reset.tape` |
| AC-135-003 | CIP write-attribute burst within 1s window emits T0836 (Modify Parameter) | `command_detections::test_t0836` | `AC-003-t0836-write-burst.gif` | `AC-003-t0836-write-burst.webm` | `AC-003-t0836-write-burst.tape` |
| AC-135-004 | `is_non_enip` suppresses T0858, T0816, and T0836 | included in master suite (`AC-ALL`) | `AC-ALL-command-detections-16-green.gif` | `AC-ALL-command-detections-16-green.webm` | `AC-ALL-command-detections-16-green.tape` |
| AC-135-005 | T0836 write count increments for each SetAttribute; aggregate write_count increments | included in master suite (`AC-ALL`) | `AC-ALL-command-detections-16-green.gif` | `AC-ALL-command-detections-16-green.webm` | `AC-ALL-command-detections-16-green.tape` |

**Master green-run** covering all 16 tests (AC-135-001 through AC-135-005):

| Artifact | Description |
|----------|-------------|
| `AC-ALL-command-detections-16-green.gif` | Full `mod command_detections` — 16/16 green |
| `AC-ALL-command-detections-16-green.webm` | Full `mod command_detections` — 16/16 green |
| `AC-ALL-command-detections-16-green.tape` | VHS script for master suite |

---

## Recordings Detail

### AC-001-t0858-cip-stop

Demonstrates `EnipAnalyzer::process_pdu` CIP Stop (service=0x07) → T0858 per-occurrence detection
(BC-2.17.011).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'command_detections::test_t0858'`
- `test_t0858_stop_service_0x07`: CIP request service=0x07 + type_id=0x00B2 + !is_non_enip →
  exactly one Finding with category=Execution, verdict=Likely, confidence=High,
  mitre_techniques=["T0858"],
  summary="CIP Stop service observed: controller run→stop transition command (T0858)"
- `test_t0858_stop_response_no_finding`: CIP response service=0x87 (high bit set → Response class)
  → no T0858 finding; classifier handles response-bit masking
- `test_t0858_connected_item_no_finding`: CIP Stop via type_id=0x00B1 → no T0858 (F-P9-001 gate)
- `test_t0858_set_attribute_no_t0858`: SetAttributeSingle (0x10) via 0x00B2 → no T0858; only
  feeds write-burst counter for T0836 path
- All 4 tests pass green

**Tests in recording:**
- `test_t0858_stop_service_0x07`
- `test_t0858_stop_response_no_finding`
- `test_t0858_connected_item_no_finding`
- `test_t0858_set_attribute_no_t0858`

---

### AC-002-t0816-cip-reset

Demonstrates `EnipAnalyzer::process_pdu` CIP Reset (classify_cip_service → CipServiceClass::Reset)
→ T0816 per-occurrence detection (BC-2.17.013).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'command_detections::test_t0816'`
- `test_t0816_reset_service`: CIP request service=0x05 + type_id=0x00B2 + !is_non_enip →
  exactly one Finding with category=Execution, verdict=Likely, confidence=High,
  mitre_techniques=["T0816"],
  summary="CIP Reset service observed: adversary-triggered device restart (T0816)"
- `test_t0816_response_no_finding`: CIP Reset response (0x85) → no T0816 (response-bit invariant
  handled inside `classify_cip_service`)
- `test_t0816_connected_item_no_finding`: CIP Reset via type_id=0x00B1 → no T0816 (F-P9-001 gate)
- All 3 tests pass green

**Tests in recording:**
- `test_t0816_reset_service`
- `test_t0816_response_no_finding`
- `test_t0816_connected_item_no_finding`

---

### AC-003-t0836-write-burst

Demonstrates T0836 write-attribute burst detection with 1-second sliding window, strict `>` threshold
semantics, one-shot guard, window reset, and EC-007 threshold=0 case (BC-2.17.012).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'command_detections::test_t0836'`
- `test_t0836_burst_fires_at_threshold_plus_one`: 51 SetAttribute requests in 1s window with
  default threshold=50 → ONE T0836 Finding (category=Execution, verdict=Likely,
  confidence=Medium, mitre=["T0836"]); `write_burst_emitted=true`
- `test_t0836_burst_one_shot_guard`: 52nd write in same window after guard is set → no additional
  T0836 finding (one-shot guard per window, EC-009)
- `test_t0836_no_fire_at_threshold`: exactly 50 writes with threshold=50 → no finding
  (strict `>` semantics, EC-008)
- `test_t0836_threshold_zero_fires_on_first_write`: threshold=0; first write (count=1 > 0) →
  T0836 fires immediately (EC-007)
- `test_t0836_window_resets_after_1s`: 51 writes at ts=0; then ts=2 (window expired); 51 more
  writes → second T0836 fires in new window; `write_burst_emitted` reset to false (EC-010)
- `test_t0836_custom_threshold`: custom threshold=10; 11 writes → T0836 fires
- All 6 tests pass green

**Tests in recording:**
- `test_t0836_burst_fires_at_threshold_plus_one`
- `test_t0836_burst_one_shot_guard`
- `test_t0836_no_fire_at_threshold`
- `test_t0836_threshold_zero_fires_on_first_write`
- `test_t0836_window_resets_after_1s`
- `test_t0836_custom_threshold`

---

### AC-ALL-command-detections-16-green

Master green-run for the full `mod command_detections` suite — 16 tests covering all
STORY-135 acceptance criteria including `is_non_enip` suppression (AC-135-004) and write-count
accumulation (AC-135-005).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests command_detections`
- All 16 tests in `mod command_detections` execute and pass
- Test result line: `test result: ok. 16 passed; 0 failed; 0 ignored`

**Tests in recording (all 16):**
- `test_t0858_stop_service_0x07`
- `test_t0858_stop_response_no_finding`
- `test_t0858_connected_item_no_finding`
- `test_t0858_set_attribute_no_t0858`
- `test_t0816_reset_service`
- `test_t0816_response_no_finding`
- `test_t0816_connected_item_no_finding`
- `test_t0836_burst_fires_at_threshold_plus_one`
- `test_t0836_burst_one_shot_guard`
- `test_t0836_no_fire_at_threshold`
- `test_t0836_threshold_zero_fires_on_first_write`
- `test_t0836_window_resets_after_1s`
- `test_t0836_custom_threshold`
- `test_non_enip_suppresses_command_detections`
- `test_write_count_accumulates`
- `test_aggregate_write_count_increments`

---

## Full command_detections Test Suite Summary

All 16 tests in `mod command_detections` pass at recording time:

```
test command_detections::test_aggregate_write_count_increments ... ok
test command_detections::test_non_enip_suppresses_command_detections ... ok
test command_detections::test_t0816_connected_item_no_finding ... ok
test command_detections::test_t0816_reset_service ... ok
test command_detections::test_t0816_response_no_finding ... ok
test command_detections::test_t0836_burst_fires_at_threshold_plus_one ... ok
test command_detections::test_t0836_burst_one_shot_guard ... ok
test command_detections::test_t0836_custom_threshold ... ok
test command_detections::test_t0836_no_fire_at_threshold ... ok
test command_detections::test_t0836_threshold_zero_fires_on_first_write ... ok
test command_detections::test_t0836_window_resets_after_1s ... ok
test command_detections::test_t0858_connected_item_no_finding ... ok
test command_detections::test_t0858_set_attribute_no_t0858 ... ok
test command_detections::test_t0858_stop_response_no_finding ... ok
test command_detections::test_t0858_stop_service_0x07 ... ok
test command_detections::test_write_count_accumulates ... ok

test result: ok. 16 passed; 0 failed; 0 ignored
```

---

## Deferred / Not Applicable

None. All 5 ACs have recorded demo coverage.

- AC-135-001 (T0858): dedicated `AC-001-t0858-cip-stop` recording + master suite.
- AC-135-002 (T0816): dedicated `AC-002-t0816-cip-reset` recording + master suite.
- AC-135-003 (T0836 write-burst): dedicated `AC-003-t0836-write-burst` recording (6 tests)
  covering threshold-fire, one-shot guard, strict `>` semantics, EC-007 threshold=0, 1s window
  reset, and custom threshold + master suite.
- AC-135-004 (`is_non_enip` suppression): covered in master suite via
  `test_non_enip_suppresses_command_detections`.
- AC-135-005 (write_count accumulation + aggregate): covered in master suite via
  `test_write_count_accumulates` and `test_aggregate_write_count_increments`.
