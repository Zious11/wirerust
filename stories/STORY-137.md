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
  2. While `buf.len() - cursor >= 24`:
     a. Call `parse_enip_header(&buf[cursor..cursor+24])`
     b. If header parse fails or `!is_valid_enip_frame`: increment `flow.parse_errors` + `flow.malformed_in_window`; byte-walk or frame-skip; continue (see AC-137-003 for exact cursor behavior)
     c. If partial frame: stash into carry; break
     d. Else: call `process_pdu`; cursor advances by `24 + header.length`
  3. `carry = buf[cursor..]` (save leftover bytes)
- `carry.len()` is bounded by `MAX_ENIP_CARRY_BYTES = 600` after each `on_data` call (BC-2.17.016 Invariant 1)
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_partial_header`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_two_frames_one_segment`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_three_segments_one_frame`

### AC-137-002: Carry buffer is capped at MAX_ENIP_CARRY_BYTES (600) — overflow sets is_non_enip ONLY
**Traces to:** BC-2.17.016 Invariant 4, Postcondition 4
- When `flow.carry.len() > MAX_ENIP_CARRY_BYTES (600)` after any carry stash:
  - `flow.is_non_enip = true` (permanently quarantine this flow)
  - `flow.parse_errors += 1`
  - `flow.malformed_in_window += 1`
  - `flow.carry` is cleared (prevents unbounded memory growth)
- **CRITICAL constraint (BC-2.17.016 Invariant 4):** `is_non_enip` is set to `true` EXCLUSIVELY by carry-buffer overflow. It is NOT set when an oversized declared frame is detected (the frame-skip path). An oversized declared frame (`total_frame_len > MAX_ENIP_CARRY_BYTES`) is handled by the frame-skip path (see AC-137-003), NOT by setting `is_non_enip`.
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_buffer_cap_at_600`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_carry_cap_sets_non_enip`

### AC-137-003: Frame-walk resync and frame-skip — correct cursor behavior per BC-2.17.016
**Traces to:** BC-2.17.016 Postcondition 1 (frame-walk loop body)
- **Unknown/invalid ENIP command (byte-walk resync path):**
  - When `is_valid_enip_frame(&header)` returns `false` (unknown command):
    - `flow.parse_errors += 1`; `flow.malformed_in_window += 1`
    - `cursor += 1` (advance by ONE byte only — byte-walk resync)
    - `continue` the loop (do NOT break; re-attempt parse at the next byte)
  - This allows resynchronization to a valid frame boundary within the same TCP segment
- **Oversized declared frame (frame-skip path):**
  - When `is_valid_enip_frame` passes but `24 + header.length as usize > MAX_ENIP_CARRY_BYTES (600)`:
    - `flow.parse_errors += 1`; `flow.malformed_in_window += 1`
    - `cursor += min(24 + header.length as usize, buf.len() - cursor)` (advance past declared frame, bounded by buffer)
    - `continue` the loop (do NOT break; do NOT set `is_non_enip`)
  - `is_non_enip` is NOT set on the frame-skip path (BC-2.17.016 Invariant 4)
- **Partial frame (stash path):** when `buf.len() - cursor < 24 + header.length`: stash `buf[cursor..]` into carry; apply cap check; break
- **`is_non_enip` is a permanent one-way flag:** once set (carry-cap ONLY), it cannot be cleared. When set, all subsequent `on_data` calls are immediate no-ops.
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_non_enip_flag_set_at_carry_cap`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_non_enip_flag_permanent`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_byte_walk_resync_invalid_command`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_oversize_frame_skip_continue`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_oversize_frame_does_not_set_non_enip`

### AC-137-004: T0814 malformed-frame DoS burst detection — windowed, per BC-2.17.018
**Traces to:** BC-2.17.018 postconditions 1–4, invariants 1/3/4
- **Two-counter model (BC-2.17.018 Invariant 1):**
  - `flow.parse_errors: u64` — LIFETIME, monotonically increasing, NEVER reset
  - `flow.malformed_in_window: u64` — WINDOWED, reset at 300s window expiry
  - Both are incremented on every structural reject (invalid command; oversized declared frame; carry overflow)
- **On every structural reject (unconditional, BC-2.17.018 Postconditions 1–2):**
  - `flow.parse_errors += 1`
  - `flow.malformed_in_window += 1`
- **Conditional T0814 emission (when all hold):**
  - `flow.malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD` (= 3) within the 300s window
  - `flow.malformed_anomaly_emitted == false`
  - `flow.is_non_enip == false` at the time of the triggering reject
  - `all_findings.len() < MAX_FINDINGS`
  - Then ONE `Finding` (BC-2.17.018 Postcondition 3):
    - `category: ThreatCategory::Anomaly`
    - `verdict: Verdict::Possible`
    - `confidence: Confidence::Low`
    - `summary: "EtherNet/IP structural anomaly: {count} malformed frames in {elapsed}s window (flow {src_ip}→{dest_ip}) — possible crash-probe"`
    - `mitre_techniques: vec!["T0814"]`
    - `source_ip: Some(...)`, `timestamp: Some(...)`
  - `flow.malformed_anomaly_emitted = true` (one-shot guard per window, BC-2.17.018 Postcondition 4)
- **Window-expiry reset (300s, BC-2.17.018 Postcondition 5):**
  - `flow.malformed_in_window = 0`
  - `flow.malformed_anomaly_emitted = false`
  - `flow.parse_errors` is NOT reset (lifetime counter)
  - After reset: a fresh burst of 3 malformed frames in the new window fires a NEW T0814
- **`is_non_enip` is NOT set when T0814 fires** (BC-2.17.016 Invariant 4): `is_non_enip` is set ONLY on carry-buffer overflow, not when the T0814 threshold is crossed
- `MALFORMED_ANOMALY_THRESHOLD = 3` — a compile-time constant (NOT CLI-configurable per ADR-010 Decision 5)
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_fires_at_threshold`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_one_shot_guard_per_window`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_does_not_fire_below_threshold`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_refire_after_window_reset`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_parse_errors_not_reset_on_window_expiry`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_t0814_non_enip_not_set_at_threshold` (satisfies HS-117 Case D)

### AC-137-005: Valid frames are processed normally; invalid frames increment parse_errors and malformed_in_window
**Traces to:** BC-2.17.016 Postcondition 1, BC-2.17.018 Postconditions 1–2
- A structural reject fires when: `is_valid_enip_frame` returns `false` (unknown command), OR declared frame is oversized (`total_frame_len > MAX_ENIP_CARRY_BYTES`), OR carry-buffer overflows
- Each structural reject increments `flow.parse_errors += 1` (LIFETIME, never reset) AND `flow.malformed_in_window += 1` (WINDOWED, reset at 300s) per BC-2.17.018 Postconditions 1–2
- Valid frames do NOT increment `parse_errors` or `malformed_in_window`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_valid_frame_no_malformed_count`
- **Test:** `tests/enip_analyzer_tests.rs::frame_walk::test_invalid_frame_increments_malformed_count`

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `EnipFlowState.carry` | `src/analyzer/enip.rs` | `Vec<u8>` — TCP reassembly carry buffer |
| `EnipFlowState.parse_errors` | `src/analyzer/enip.rs` | `u64` — LIFETIME malformed frame counter (never reset) |
| `EnipFlowState.malformed_in_window` | `src/analyzer/enip.rs` | `u64` — WINDOWED malformed frame counter (reset at 300s) |
| `EnipFlowState.malformed_anomaly_emitted` | `src/analyzer/enip.rs` | `bool` — one-shot guard for T0814 per window |
| `EnipFlowState.is_non_enip` | `src/analyzer/enip.rs` | `bool` — permanent quarantine flag (set ONLY on carry-buffer overflow) |
| `MAX_ENIP_CARRY_BYTES` | `src/analyzer/enip.rs` | `const usize = 600` |
| `MALFORMED_ANOMALY_THRESHOLD` | `src/analyzer/enip.rs` | `const u64 = 3` |
| Frame-walk loop | `src/analyzer/enip.rs` | `EnipAnalyzer::on_data` outer loop (not process_pdu — on_data IS the frame-walk per BC-2.17.016) |
| T0814 detection | `src/analyzer/enip.rs` | `if malformed_in_window >= THRESHOLD && !malformed_anomaly_emitted && !is_non_enip → emit T0814 Anomaly/Possible/Low` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod frame_walk { ... }` |

**Frame-walk pseudocode (BC-2.17.016 Postcondition 1, ADR-010 Decision 4) — CORRECTED:**
```
fn on_data(flow, data, now_ts, ...) {
    if flow.is_non_enip { return; }
    // Check/reset 300s window (BC-2.17.018 Postcondition 5)
    if now_ts - flow.malformed_window_start >= 300 {
        flow.malformed_in_window = 0;
        flow.malformed_anomaly_emitted = false;
        flow.malformed_window_start = now_ts;
    }
    let mut buf = flow.carry.clone();
    buf.extend_from_slice(data);
    let mut cursor = 0;
    while buf.len() - cursor >= 24 {
        let Some(header) = parse_enip_header(&buf[cursor..cursor+24]) else {
            // Unknown/invalid header: byte-walk resync (BC-2.17.016 Post 1)
            flow.parse_errors += 1;
            flow.malformed_in_window += 1;
            check_t0814(flow, ...);
            cursor += 1;          // advance by 1 byte — NOT break
            continue;
        };
        let total_frame_len = 24 + header.length as usize;
        if total_frame_len > MAX_ENIP_CARRY_BYTES {
            // Oversized declared frame: frame-skip path (BC-2.17.016 Post 1)
            // DO NOT set is_non_enip (BC-2.17.016 Inv 4)
            flow.parse_errors += 1;
            flow.malformed_in_window += 1;
            check_t0814(flow, ...);
            cursor += min(total_frame_len, buf.len() - cursor); // advance past declared frame
            continue;             // NOT break
        }
        if buf.len() - cursor < total_frame_len {
            // Partial frame: stash and break
            break;
        }
        // Valid complete frame
        process_pdu(flow, &buf[cursor..cursor+total_frame_len], ...);
        cursor += total_frame_len;
    }
    // Stash remaining bytes into carry (BC-2.17.016 Post 3)
    flow.carry = buf[cursor..].to_vec();
    // Carry-cap check — is_non_enip set ONLY here (BC-2.17.016 Post 4 / Inv 4)
    if flow.carry.len() > MAX_ENIP_CARRY_BYTES {
        flow.is_non_enip = true;
        flow.parse_errors += 1;
        flow.malformed_in_window += 1;
        check_t0814(flow, ...);
        flow.carry.clear();
    }
}
```

**Key behavioral corrections from BC-2.17.016:**
- Unknown command → `cursor += 1; continue` (byte-walk resync), NOT break
- Oversized declared frame → advance past it, `continue`, NOT break, NOT set `is_non_enip`
- `is_non_enip` is set ONLY on carry-buffer overflow (BC-2.17.016 Invariant 4)
- T0814 uses WINDOWED `malformed_in_window` counter (not a lifetime counter), reset every 300s (BC-2.17.018)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single segment containing exactly 1 complete frame | Frame processed; `carry` empty |
| EC-002 | Single segment containing 2 complete frames | Both processed in same `on_data` call; `carry` empty |
| EC-003 | Frame split across 2 segments | First segment: `carry` has partial data. Second segment: frame completed and processed |
| EC-004 | Carry grows to 601 bytes | `is_non_enip = true`; `parse_errors += 1`; `malformed_in_window += 1`; `carry.clear()` — T0814 may fire if window threshold crossed |
| EC-005 | 1 malformed frame in window | `parse_errors = 1`; `malformed_in_window = 1`; no T0814 yet |
| EC-006 | 2 malformed frames in window | `parse_errors = 2`; `malformed_in_window = 2`; no T0814 yet |
| EC-007 | 3rd malformed frame in window (threshold) | `parse_errors = 3`; `malformed_in_window = 3`; T0814 Anomaly/Possible/Low emitted; `malformed_anomaly_emitted = true`; `is_non_enip` NOT set (carry not overflowed) |
| EC-008 | 4th malformed frame in same window (guard set) | `parse_errors = 4`; `malformed_in_window = 4`; no additional T0814 (guard set) |
| EC-009 | 300s window expires; 3 fresh malformed frames | Window reset: `malformed_in_window = 0`, `malformed_anomaly_emitted = false`; `parse_errors` unchanged (lifetime); new 3-frame burst fires fresh T0814 |
| EC-010 | Oversized declared frame (header.length=600; total=624 in 624-byte buffer) | Frame-skip: `parse_errors += 1`; `malformed_in_window += 1`; `cursor += 624`; `is_non_enip` NOT set; loop continues (Case D — HS-117 requirement) |
| EC-011 | `is_non_enip=true` from start | `on_data` returns immediately; no frame walk, no counter updates |
| EC-012 | Unknown command byte followed immediately by valid ENIP frame | Byte-walk: `cursor += 1`; `parse_errors += 1`; loop continues; valid frame parsed on next iteration |

## Tasks

- [ ] Define `const MAX_ENIP_CARRY_BYTES: usize = 600` in `src/analyzer/enip.rs`
- [ ] Define `const MALFORMED_ANOMALY_THRESHOLD: u64 = 3` in `src/analyzer/enip.rs`
- [ ] Add to `EnipFlowState`: `carry: Vec<u8>`, `parse_errors: u64` (LIFETIME, never reset), `malformed_in_window: u64` (WINDOWED, reset at 300s), `malformed_anomaly_emitted: bool`, `malformed_window_start: Timestamp`
- [ ] Implement frame-walk loop in `EnipAnalyzer::on_data` (NOT process_pdu — on_data IS the outer loop per BC-2.17.016):
  - Window expiry check at top of on_data (reset malformed_in_window + guard at 300s)
  - carry+data concatenation
  - On unknown command (is_valid_enip_frame false): `parse_errors += 1`; `malformed_in_window += 1`; `check_t0814()`; `cursor += 1`; `continue` — byte-walk resync
  - On oversized declared frame (total_frame_len > MAX_ENIP_CARRY_BYTES): `parse_errors += 1`; `malformed_in_window += 1`; `check_t0814()`; `cursor += min(total_frame_len, buf.len() - cursor)`; `continue` — frame-skip; do NOT set is_non_enip
  - On partial frame: stash into carry; break
  - Carry-cap check after loop: if carry.len() > MAX_ENIP_CARRY_BYTES: `is_non_enip = true`; `parse_errors += 1`; `malformed_in_window += 1`; `check_t0814()`; `carry.clear()`
- [ ] Implement `check_t0814` helper: `if malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD && !malformed_anomaly_emitted && !is_non_enip && all_findings.len() < MAX_FINDINGS` → emit T0814 Anomaly/Possible/Low finding; `malformed_anomaly_emitted = true` (do NOT set `is_non_enip` here)
- [ ] Add `mod frame_walk { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-137 tests including windowed reset and byte-walk/frame-skip cases
- [ ] Construct test data: single-frame segment, two-frame segment, split-frame pair, oversized carry (601 bytes), oversized declared frame (total > 600), repeated malformed headers, window expiry re-fire
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
frame_walk::test_byte_walk_resync_invalid_command
frame_walk::test_oversize_frame_skip_continue
frame_walk::test_oversize_frame_does_not_set_non_enip
frame_walk::test_non_enip_flag_permanent
frame_walk::test_non_enip_flag_set_at_carry_cap
frame_walk::test_t0814_fires_at_threshold
frame_walk::test_t0814_one_shot_guard_per_window
frame_walk::test_t0814_does_not_fire_below_threshold
frame_walk::test_t0814_refire_after_window_reset
frame_walk::test_parse_errors_not_reset_on_window_expiry
frame_walk::test_t0814_non_enip_not_set_at_threshold
frame_walk::test_valid_frame_no_malformed_count
frame_walk::test_invalid_frame_increments_malformed_count
```

## Previous Story Intelligence

- STORY-130 provides `parse_enip_header` and `is_valid_enip_frame` — these are the two gatekeeping functions called in the frame-walk loop
- STORY-134/135/136 all depend on `is_non_enip` being set correctly by this story; if STORY-137 is implemented after STORY-134/135/136, verify that the `is_non_enip` field already exists in `EnipFlowState` (added by those stories). If not, this story adds it.
- STORY-137 is the story that sets `is_non_enip = true`; STORY-134/135/136 only read it. Ensure the `is_non_enip` field is declared in a single place (not duplicated across stories).

**Integration note:** STORY-137 implements the outermost `process_pdu` frame-walk loop that all other detection BCs (STORY-134/135/136) plug into. In practice, the implementer should integrate all detection calls (from STORY-134/135/136) into the frame-walk loop implemented here. If STORY-134/135/136 are already merged, STORY-137 adds the carry buffer and malformed-frame logic around the existing detection calls without removing them.

## Architecture Compliance Rules

1. **Carry buffer concatenation with prepend (ADR-010 Decision 4):** The carry buffer is prepended to new data (`carry + data`), NOT appended. After processing, `flow.carry = remaining_bytes`.
2. **MAX_ENIP_CARRY_BYTES = 600 is a hard cap (ADR-010 Decision 3/4):** The cap prevents DoS via memory exhaustion. When CARRY OVERFLOWS (not declared frame size): clear carry, increment parse_errors + malformed_in_window, set is_non_enip. This is not configurable.
3. **MALFORMED_ANOMALY_THRESHOLD = 3 is a compile-time constant (ADR-010 Decision 5):** NOT CLI-configurable. It is a `const u64 = 3` in the source.
4. **is_non_enip is a one-way permanent flag set ONLY on carry-buffer overflow (BC-2.17.016 Invariant 4):** Once set, it is never cleared. It is NOT set when T0814 fires. It is NOT set on oversized declared frame skip. It is NOT set on unknown command byte-walk. Any other trigger for is_non_enip is a bug.
5. **T0814 uses WINDOWED `malformed_in_window`, NOT a lifetime `malformed_count` (BC-2.17.018 Invariant 1):** Two counters: `parse_errors` (lifetime, never reset) and `malformed_in_window` (windowed, reset at 300s). T0814 threshold check uses `malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD`.
6. **Unknown command → `cursor += 1; continue` (byte-walk resync) (BC-2.17.016 Postcondition 1):** Do NOT break when an unknown command is seen. Advance cursor by 1 byte and continue the loop to resynchronize.
7. **Oversized declared frame → advance past + `continue` (frame-skip) (BC-2.17.016 Postcondition 1):** Do NOT break. Do NOT set is_non_enip. Advance cursor by `min(total_frame_len, buf.len()-cursor)` and continue.
8. **T0814 finding fields are Anomaly/Possible/Low (BC-2.17.018 Postcondition 3):** NOT DenialOfService/Medium. Category `ThreatCategory::Anomaly`, verdict `Verdict::Possible`, confidence `Confidence::Low`.
9. **T0814 re-fires after 300s window reset (BC-2.17.018 Postcondition 5 / EC-005):** The `malformed_anomaly_emitted` guard is per-window, not per-flow-lifetime. After reset, a fresh burst of 3 malformed frames fires a new T0814.

## Library & Framework Requirements

No new external crate dependencies. `Vec<u8>` for carry buffer (owned, cleared on cap).

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `EnipFlowState` carry/malformed fields; implement frame-walk loop in `EnipAnalyzer::on_data` (NOT `process_pdu` — `on_data` IS the outer frame-walk per BC-2.17.016); add constants `MAX_ENIP_CARRY_BYTES` and `MALFORMED_ANOMALY_THRESHOLD`
- `tests/enip_analyzer_tests.rs` — add `mod frame_walk { ... }` block

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` frame-walk + T0814 additions | ~450 |
| `tests/enip_analyzer_tests.rs` frame_walk mod (13 tests) | ~550 |
| **Total** | **~1,000** |

## Dependency Rationale

Wave 60; depends on STORY-132 (parse layer) and STORY-133 (MITRE catalog for T0814). Parallel with STORY-134/135/136. The `is_non_enip` flag set here is read by STORY-134/135/136's detection logic — if those stories are implemented in the same wave, coordinate the `EnipFlowState` field definition to avoid duplicate field declarations.
