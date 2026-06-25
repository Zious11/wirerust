---
document_type: story
story_id: STORY-137
title: "ENIP Frame Walk Robustness: Carry Buffer, Non-ENIP Detection, and T0814 DoS Burst"
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
  - BC-2.17.016
  - BC-2.17.018
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.016.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.018.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
input-hash: "24ecccd"
---

# STORY-137: ENIP Frame Walk Robustness: Carry Buffer, Non-ENIP Detection, and T0814 DoS Burst

## Narrative

**As a** security analyst and software engineer,
**I want** the EtherNet/IP analyzer to robustly handle TCP stream reassembly (via carry buffer),
correctly detect and quarantine non-ENIP traffic on port 44818, and detect T0814 DoS bursts
(malformed frame accumulation),
**so that** the analyzer is safe against malformed/adversarial traffic and does not produce
false positives on non-ENIP flows.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.016 | Frame-walk algorithm with carry buffer handles partial frames and multi-frame PDUs | Core robustness implementation |
| BC-2.17.018 | Malformed ENIP frame burst (T0814 DoS) detection | Core detection with windowed threshold |

## Acceptance Criteria

### AC-137-001: Carry buffer accumulates partial ENIP frames across TCP segments
**Traces to:** BC-2.17.016 postconditions 1–3
- `EnipFlowState.carry: Vec<u8>` holds leftover bytes from a previous TCP segment that did not form a complete ENIP frame
- When a new TCP segment arrives:
  1. `data = carry + new_segment_data` (prepend carry)
  2. While `data.len() >= 24`:
     a. Call `parse_enip_header(&data[cursor..cursor+24])`
     b. If `is_valid_enip_frame(&header, data[cursor..].len())`: process the frame (cursor advances by `24 + header.length as usize`)
     c. Else: increment `flow.malformed_count`; break (do not advance past invalid frame)
  3. `carry = data[cursor..]` (save leftover bytes)
- `carry.len()` is always `< 24` after a successful parse cycle (partial header)
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_partial_header`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_two_frames_one_segment`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_three_segments_one_frame`

### AC-137-002: Carry buffer is capped at MAX_ENIP_CARRY_BYTES (600)
**Traces to:** BC-2.17.016 Invariant 2 (MAX_ENIP_CARRY_BYTES = 600)
- When `carry.len()` would exceed `MAX_ENIP_CARRY_BYTES (600)` after a frame-walk cycle:
  - `carry` is truncated to 0 (cleared)
  - `flow.malformed_count += 1`
  - `flow.is_non_enip = true` (permanently quarantine this flow)
- The cap prevents unbounded memory growth from adversarial streams
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_cap_at_600`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_cap_sets_non_enip`

### AC-137-003: Non-ENIP detection via carry-cap or invalid-header accumulation
**Traces to:** BC-2.17.016 Invariant 3 (is_non_enip flag)
- When `flow.malformed_count >= MALFORMED_ANOMALY_THRESHOLD (3)` within the flow lifetime (or the carry cap is hit once):
  - `flow.is_non_enip = true`
  - All subsequent PDUs for this flow skip all detection logic
- `is_non_enip` is a permanent one-way flag: once set, it cannot be cleared
- `MALFORMED_ANOMALY_THRESHOLD = 3` (constant in `src/analyzer/enip.rs`)
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_non_enip_flag_set_after_3_malformed`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_non_enip_flag_permanent`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_non_enip_flag_set_at_carry_cap`

### AC-137-004: T0814 malformed-frame DoS burst detection
**Traces to:** BC-2.17.018 postconditions 1–2
- Given `flow.malformed_count` increments crossing `MALFORMED_ANOMALY_THRESHOLD (3)` within the session
- When `flow.malformed_count == MALFORMED_ANOMALY_THRESHOLD` (on the 3rd malformed frame)
- AND `flow.dos_emitted == false`
- AND `all_findings.len() < MAX_FINDINGS`
- Then ONE `Finding`:
  - `category: ThreatCategory::Denial Of Service` (or equivalent)
  - `verdict: Verdict::Possible`
  - `confidence: Confidence::Medium`
  - `summary: "ENIP malformed frame burst: {N} malformed frames — possible DoS or scanner (T0814)"`
  - `mitre_techniques: vec!["T0814"]`
  - `flow.dos_emitted = true` (one-shot guard)
- `MALFORMED_ANOMALY_THRESHOLD = 3`: at the 3rd malformed frame the finding fires
- After the finding fires, `is_non_enip = true` (flow permanently quarantined)
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_fires_at_threshold`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_one_shot_guard`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_does_not_fire_below_threshold`

### AC-137-005: Valid frames are processed normally; invalid frames increment malformed_count
**Traces to:** BC-2.17.016 postcondition 2c, BC-2.17.018 precondition
- A frame is invalid if `parse_enip_header` returns `None` OR `is_valid_enip_frame` returns `false`
- Each invalid frame increments `flow.malformed_count` by 1
- Valid frames do NOT increment `malformed_count`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_valid_frame_no_malformed_count`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_invalid_frame_increments_malformed_count`

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `EnipFlowState.carry` | `src/analyzer/enip.rs` | `Vec<u8>` — TCP reassembly carry buffer |
| `EnipFlowState.malformed_count` | `src/analyzer/enip.rs` | `u32` — malformed frame counter for T0814 |
| `EnipFlowState.dos_emitted` | `src/analyzer/enip.rs` | `bool` — one-shot guard for T0814 finding |
| `EnipFlowState.is_non_enip` | `src/analyzer/enip.rs` | `bool` — permanent quarantine flag (also used by STORY-134/135/136) |
| `MAX_ENIP_CARRY_BYTES` | `src/analyzer/enip.rs` | `const usize = 600` |
| `MALFORMED_ANOMALY_THRESHOLD` | `src/analyzer/enip.rs` | `const u32 = 3` |
| Frame-walk loop | `src/analyzer/enip.rs` | `EnipAnalyzer::process_pdu` inner loop over carry+data |
| T0814 detection | `src/analyzer/enip.rs` | `if malformed_count >= THRESHOLD && !dos_emitted → emit T0814` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod frame_walk { ... }` |

**Frame-walk pseudocode (ADR-010 Decision 4):**
```
fn process_pdu(flow, data, ...) {
    if flow.is_non_enip { return; }
    let mut buf = flow.carry.clone();
    buf.extend_from_slice(data);
    let mut cursor = 0;
    while cursor + 24 <= buf.len() {
        let Some(header) = parse_enip_header(&buf[cursor..cursor+24]) else {
            flow.malformed_count += 1;
            check_t0814(flow, ...);
            break;
        };
        let frame_end = cursor + 24 + header.length as usize;
        if frame_end > buf.len() {
            break;  // incomplete frame; wait for more data
        }
        if !is_valid_enip_frame(&header, buf[cursor..].len()) {
            flow.malformed_count += 1;
            check_t0814(flow, ...);
            break;
        }
        // Process the complete frame at buf[cursor..frame_end]
        process_frame(flow, &header, &buf[cursor+24..frame_end], ...);
        cursor = frame_end;
    }
    flow.carry = buf[cursor..].to_vec();
    if flow.carry.len() > MAX_ENIP_CARRY_BYTES {
        flow.carry.clear();
        flow.malformed_count += 1;
        flow.is_non_enip = true;
    }
}
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single segment containing exactly 1 complete frame | Frame processed; `carry` empty |
| EC-002 | Single segment containing 2 complete frames | Both processed; `carry` empty |
| EC-003 | Frame split across 2 segments | First segment: `carry` has partial data. Second segment: frame completed and processed |
| EC-004 | Carry grows to 601 bytes | `carry.clear()`; `malformed_count += 1`; `is_non_enip = true` |
| EC-005 | 1 malformed frame | `malformed_count = 1`; no T0814 yet |
| EC-006 | 2 malformed frames | `malformed_count = 2`; no T0814 yet |
| EC-007 | 3rd malformed frame | `malformed_count = 3`; T0814 emitted; `dos_emitted = true`; `is_non_enip = true` |
| EC-008 | 4th malformed frame (is_non_enip=true) | Skipped entirely; no additional T0814 |
| EC-009 | Carry cap hit once (count goes to 1) and then 2 more malformed in same flow | Carry cap also increments malformed_count; when total reaches 3, T0814 fires |
| EC-010 | `is_non_enip=true` from start | process_pdu returns immediately; no frame walk |

## Tasks

- [ ] Define `const MAX_ENIP_CARRY_BYTES: usize = 600` in `src/analyzer/enip.rs`
- [ ] Define `const MALFORMED_ANOMALY_THRESHOLD: u32 = 3` in `src/analyzer/enip.rs`
- [ ] Add to `EnipFlowState`: `carry: Vec<u8>`, `malformed_count: u32`, `dos_emitted: bool`
- [ ] Implement frame-walk loop in `EnipAnalyzer::process_pdu`: carry+data concatenation, cursor advance, `parse_enip_header` + `is_valid_enip_frame` guards, malformed increment, carry update, carry-cap check with `is_non_enip` set
- [ ] Implement `check_t0814` helper or inline: `if malformed_count >= MALFORMED_ANOMALY_THRESHOLD && !dos_emitted && all_findings.len() < MAX_FINDINGS` → emit T0814 finding; `dos_emitted = true`; `is_non_enip = true`
- [ ] Add `mod frame_walk { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-137 tests
- [ ] Construct test data: single-frame segment, two-frame segment, split-frame pair, oversized carry, repeated malformed headers
- [ ] Run `cargo test enip` — all frame_walk tests pass
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod frame_walk { ... }`

```
frame_walk::test_carry_buffer_partial_header
frame_walk::test_carry_buffer_two_frames_one_segment
frame_walk::test_carry_buffer_three_segments_one_frame
frame_walk::test_carry_buffer_cap_at_600
frame_walk::test_carry_cap_sets_non_enip
frame_walk::test_non_enip_flag_set_after_3_malformed
frame_walk::test_non_enip_flag_permanent
frame_walk::test_non_enip_flag_set_at_carry_cap
frame_walk::test_t0814_fires_at_threshold
frame_walk::test_t0814_one_shot_guard
frame_walk::test_t0814_does_not_fire_below_threshold
frame_walk::test_valid_frame_no_malformed_count
frame_walk::test_invalid_frame_increments_malformed_count
```

## Previous Story Intelligence

- STORY-130 provides `parse_enip_header` and `is_valid_enip_frame` — these are the two gatekeeping functions called in the frame-walk loop
- STORY-134/135/136 all depend on `is_non_enip` being set correctly by this story; if STORY-137 is implemented after STORY-134/135/136, verify that the `is_non_enip` field already exists in `EnipFlowState` (added by those stories). If not, this story adds it.
- STORY-137 is the story that sets `is_non_enip = true`; STORY-134/135/136 only read it. Ensure the `is_non_enip` field is declared in a single place (not duplicated across stories).

**Integration note:** STORY-137 implements the outermost `process_pdu` frame-walk loop that all other detection BCs (STORY-134/135/136) plug into. In practice, the implementer should integrate all detection calls (from STORY-134/135/136) into the frame-walk loop implemented here. If STORY-134/135/136 are already merged, STORY-137 adds the carry buffer and malformed-frame logic around the existing detection calls without removing them.

## Architecture Compliance Rules

1. **Carry buffer concatenation with prepend (ADR-010 Decision 4):** The carry buffer is prepended to new data (`carry + data`), NOT appended. `flow.carry.extend_from_slice(new_data); process(flow.carry)` is the pattern. After processing, `flow.carry = remaining_bytes`.
2. **MAX_ENIP_CARRY_BYTES = 600 is a hard cap (ADR-010 Decision 4):** The cap prevents DoS via memory exhaustion. When exceeded: clear carry, mark malformed, set is_non_enip. This is not configurable.
3. **MALFORMED_ANOMALY_THRESHOLD = 3 is a compile-time constant (ADR-010 Decision 5):** Unlike the write-burst and error-burst thresholds, this is NOT a CLI-configurable value. It is a constant in the source.
4. **is_non_enip is a one-way permanent flag:** Once set, it is never cleared for the flow lifetime. Any logic that would reset it is a bug.
5. **T0814 uses MALFORMED_ANOMALY_THRESHOLD as the exact trigger (>= 3):** The threshold check is `malformed_count >= MALFORMED_ANOMALY_THRESHOLD`, NOT `malformed_count > MALFORMED_ANOMALY_THRESHOLD`. This fires ON the 3rd malformed frame, not after it. (Note: this is `>=` unlike the `>` convention for T0836/T0888 — malformed frame detection is a count-at-threshold, not count-exceeds-threshold.)
6. **Break on malformed (do not advance cursor):** When a malformed frame is detected, the frame-walk loop BREAKS (does not advance cursor past the bad bytes). The carry buffer retains the unprocessable data, but the malformed_count increment and subsequent is_non_enip guard prevent infinite loops.

## Library & Framework Requirements

No new external crate dependencies. `Vec<u8>` for carry buffer (owned, cleared on cap).

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `EnipFlowState` carry/malformed fields; implement frame-walk loop in `process_pdu`; add constants `MAX_ENIP_CARRY_BYTES` and `MALFORMED_ANOMALY_THRESHOLD`
- `tests/enip_analyzer_tests.rs` — add `mod frame_walk { ... }` block

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` frame-walk + T0814 additions | ~450 |
| `tests/enip_analyzer_tests.rs` frame_walk mod (13 tests) | ~550 |
| **Total** | **~1,000** |

## Dependency Rationale

Wave 60; depends on STORY-132 (parse layer) and STORY-133 (MITRE catalog for T0814). Parallel with STORY-134/135/136. The `is_non_enip` flag set here is read by STORY-134/135/136's detection logic — if those stories are implemented in the same wave, coordinate the `EnipFlowState` field definition to avoid duplicate field declarations.
