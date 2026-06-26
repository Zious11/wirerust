# Demo Evidence Report — STORY-137

**Story:** ENIP Frame Walk Robustness: Carry Buffer, Non-ENIP Detection, and T0814 DoS Burst
**Story ID:** STORY-137
**Wave:** 60
**Product type:** Pure-core library (no CLI/UI surface — EtherNet/IP analyzer frame-walk robustness only)
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)
**Recorded:** 2026-06-26
**Test result at recording time:** 28 passed / 0 failed / 0 ignored (mod frame_walk)

---

## AC Coverage Map

| AC | Title | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|-----------------|---------------|----------------|------|
| AC-137-001 | Carry buffer accumulates partial ENIP frames across TCP segments | `frame_walk::test_carry_buffer`, `frame_walk::test_max_enip` | `AC-001-carry-buffer.gif` | `AC-001-carry-buffer.webm` | `AC-001-carry-buffer.tape` |
| AC-137-002 | Carry buffer bounded below cap; is_non_enip latch unreachable (RULING-137-002) | `frame_walk::test_carry_stays`, `frame_walk::test_carry_cap`, `frame_walk::test_subframe`, `frame_walk::test_carry_overflow` | `AC-002-carry-bounded.gif` | `AC-002-carry-bounded.webm` | `AC-002-carry-bounded.tape` |
| AC-137-003 | Byte-walk resync + oversized-frame-skip; both `continue` (EC-010/EC-012) | `frame_walk::test_byte_walk`, `frame_walk::test_oversize`, `frame_walk::test_non_enip` | `AC-003-byte-walk-resync.gif` | `AC-003-byte-walk-resync.webm` | `AC-003-byte-walk-resync.tape` |
| AC-137-004 | T0814 DoS burst detection — windowed threshold=3, one-shot guard, re-fire after 300s | `frame_walk::test_t0814`, `frame_walk::test_valid_frame`, `frame_walk::test_invalid_frame`, `frame_walk::test_parse_errors` | `AC-004-t0814-dos-burst.gif` | `AC-004-t0814-dos-burst.webm` | `AC-004-t0814-dos-burst.tape` |
| AC-137-005 | Valid frames no malformed count; invalid frames increment both counters | included in AC-004 filter (`test_valid_frame`, `test_invalid_frame`) + master suite (`AC-ALL`) | `AC-ALL-frame-walk-28-green.gif` | `AC-ALL-frame-walk-28-green.webm` | `AC-ALL-frame-walk-28-green.tape` |
| AC-137-006 | `command_counts` single canonical site incl. Unknown (0xFF00) (BC-2.17.016 PC-0) | `frame_walk::test_command_counts` | `AC-006-command-counts.gif` | `AC-006-command-counts.webm` | `AC-006-command-counts.tape` |

**Master green-run** covering all 28 tests (AC-137-001 through AC-137-006):

| Artifact | Description |
|----------|-------------|
| `AC-ALL-frame-walk-28-green.gif` | Full `mod frame_walk` — 28/28 green |
| `AC-ALL-frame-walk-28-green.webm` | Full `mod frame_walk` — 28/28 green |
| `AC-ALL-frame-walk-28-green.tape` | VHS script for master suite |

---

## Recordings Detail

### AC-001-carry-buffer

Demonstrates `EnipFlowState.carry: Vec<u8>` TCP reassembly carry buffer accumulates partial ENIP
frames across TCP segments (BC-2.17.016 Postconditions 1–3).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'frame_walk::test_carry_buffer|frame_walk::test_max_enip'`
- `test_carry_buffer_partial_header`: segment split after 10 bytes → carry holds 10 bytes; second
  segment completes the frame and processes it
- `test_carry_buffer_two_frames_one_segment`: two full frames in one segment → both processed in
  same `on_data` call; carry empty after
- `test_carry_buffer_three_segments_one_frame`: frame split across 3 segments → frame completed
  and processed on the 3rd call; carry empty after
- `test_multi_call_carry_residue_counting`: multi-call residue accumulation; parse_errors counted
  correctly across calls (AC-137-001/004 combined)
- `test_max_enip_carry_bytes_is_600`: `MAX_ENIP_CARRY_BYTES` compile-time constant equals 600
- All 5 tests pass green

**Tests in recording:**
- `test_carry_buffer_partial_header`
- `test_carry_buffer_two_frames_one_segment`
- `test_carry_buffer_three_segments_one_frame`
- `test_multi_call_carry_residue_counting`
- `test_max_enip_carry_bytes_is_600`

---

### AC-002-carry-bounded

Demonstrates that `carry.len()` stays bounded below `MAX_ENIP_CARRY_BYTES (600)` under the spec
frame-walk algorithm — and documents RULING-137-002 (the carry-overflow `is_non_enip` latch is
provably unreachable in practice, deferred to v0.12.0 as `spec-defect-is_non_enip-dead-latch`).
Also confirms the critical T0814-before-latch ordering constraint (BC-2.17.018 EC-007 / Precondition 6).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to carry-bounds tests
- `test_carry_stays_bounded_below_cap`: carry stays ≤23 bytes; never exceeds 600
- `test_carry_cap_does_not_fire_under_spec_algorithm`: `is_non_enip` latch does NOT fire under
  normal spec-compliant frame-walk (RULING-137-002)
- `test_subframe_accumulation_keeps_carry_bounded_no_latch`: carry bounded (≤23 bytes) across
  subframe accumulation; `is_non_enip` stays false (BC-2.17.016 Inv-1/Post-4/Inv-4)
- `test_carry_overflow_third_malformed_fires_t0814_before_latch`: T0814 fires BEFORE
  `is_non_enip` is latched on the 3rd malformed event (BC-2.17.018 EC-007 ordering constraint;
  `check_t0814` guard includes `&& !flow.is_non_enip` — latch first would suppress T0814)
- All 4 tests pass green

**Tests in recording:**
- `test_carry_stays_bounded_below_cap`
- `test_carry_cap_does_not_fire_under_spec_algorithm`
- `test_subframe_accumulation_keeps_carry_bounded_no_latch`
- `test_carry_overflow_third_malformed_fires_t0814_before_latch`

---

### AC-003-byte-walk-resync

Demonstrates both `continue`-path variants in the frame-walk loop: byte-walk resync on unknown
command (cursor += 1; continue) and oversized-frame-skip (cursor += min(total, remaining); continue)
— neither sets `is_non_enip` (BC-2.17.016 Invariant 4). Also confirms the permanent one-way
`is_non_enip` flag behavior.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to byte-walk, oversize, and non_enip tests
- `test_byte_walk_resync_to_valid_frame_same_segment`: garbage byte + valid frame in one segment →
  valid frame processed same call (EC-012; byte-walk resync path)
- `test_oversize_frame_skip_then_valid_frame_processed`: oversized declared frame + trailing valid
  frame → trailing frame processed; `is_non_enip` stays false (EC-010; frame-skip path)
- `test_byte_walk_resync_invalid_command`: lone unknown command → byte-walk; `process_pdu` NOT called
- `test_oversize_frame_skip_continue`: oversized declared frame → `cursor` advances past it; continue
- `test_oversize_frame_does_not_set_non_enip`: oversized frame → `is_non_enip` stays false
  (BC-2.17.016 Invariant 4 — `is_non_enip` set ONLY on carry-buffer overflow)
- `test_non_enip_not_latched_at_carry_cap`: normal carry stash does NOT latch `is_non_enip`
  (RULING-137-002 boundary confirmation)
- `test_non_enip_flag_permanent`: once `is_non_enip=true`, subsequent `on_data` calls are
  immediate no-ops (one-way permanent flag; EC-011)
- `test_byte_walk_resync_24_garbage_bytes_then_valid_frame`: 24 garbage bytes → `parse_errors=24`;
  valid frame processed on the 25th byte; T0814 fires at threshold (AC-137-003/004 combined)
- All 8 tests pass green

**Tests in recording:**
- `test_byte_walk_resync_invalid_command`
- `test_byte_walk_resync_to_valid_frame_same_segment`
- `test_byte_walk_resync_24_garbage_bytes_then_valid_frame`
- `test_oversize_frame_skip_continue`
- `test_oversize_frame_does_not_set_non_enip`
- `test_oversize_frame_skip_then_valid_frame_processed`
- `test_non_enip_not_latched_at_carry_cap`
- `test_non_enip_flag_permanent`

---

### AC-004-t0814-dos-burst

Demonstrates T0814 malformed-frame DoS burst detection: windowed counter model
(`parse_errors` = LIFETIME, `malformed_in_window` = WINDOWED reset at 300s), threshold=3,
one-shot guard per window, re-fire after window reset (BC-2.17.018 Postconditions 1–5).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` filtered to T0814, valid/invalid frame, and
  parse_errors tests
- `test_t0814_fires_at_threshold`: 3 malformed frames → exactly one T0814 Anomaly/Possible/Low
  emitted; `mitre_techniques=["T0814"]`; `malformed_anomaly_emitted=true`
- `test_t0814_does_not_fire_below_threshold`: 1 or 2 malformed frames → no T0814 (EC-005/006)
- `test_t0814_one_shot_guard_per_window`: 4th+ malformed frame in same window → no second T0814
  (`malformed_anomaly_emitted` guard blocks re-fire within window; EC-008)
- `test_t0814_refire_after_window_reset`: 300s window expires → `malformed_in_window=0`,
  `malformed_anomaly_emitted=false`; fresh 3-frame burst fires new T0814 (EC-009)
- `test_parse_errors_not_reset_on_window_expiry`: `parse_errors` (lifetime) unchanged after
  window reset; `malformed_in_window` reset; two-counter model (BC-2.17.018 Invariant 1)
- `test_t0814_non_enip_not_set_at_threshold`: T0814 fires → `is_non_enip` stays false
  (HS-117 Case D; T0814 is detection-only, NOT quarantine)
- `test_t0814_fires_on_third_byte_walk_reject`: T0814 fires on 3rd structural reject via
  byte-walk path (not only carry-overflow path)
- `test_valid_frame_no_malformed_count`: valid frame → `parse_errors`/`malformed_in_window`
  both unchanged (BC-2.17.016 Post-1)
- `test_invalid_frame_increments_malformed_count`: structural reject → both counters increment
  (BC-2.17.018 Post-1/2)
- All 9 tests pass green

**Tests in recording:**
- `test_t0814_fires_at_threshold`
- `test_t0814_does_not_fire_below_threshold`
- `test_t0814_one_shot_guard_per_window`
- `test_t0814_refire_after_window_reset`
- `test_parse_errors_not_reset_on_window_expiry`
- `test_t0814_non_enip_not_set_at_threshold`
- `test_t0814_fires_on_third_byte_walk_reject`
- `test_valid_frame_no_malformed_count`
- `test_invalid_frame_increments_malformed_count`

---

### AC-006-command-counts

Demonstrates the single canonical `command_counts` increment site in the frame-walk loop
(immediately after `parse_enip_header` returns `Some`, BEFORE `is_valid_enip_frame`) — ensures
ALL parsed headers including Unknown (0xFF00) are counted, and that `process_pdu` does NOT
duplicate the increment (BC-2.17.016 PC-0 / BC-2.17.004 Invariant 3).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'frame_walk::test_command_counts'`
- `test_command_counts_increments_for_unknown_command`: frame with unknown cmd 0xFF00 (full
  24-byte header) → `command_counts[0xFF00]==1` AND `pdu_count==0` (`process_pdu` NOT called);
  confirms Unknown bucket is countable via frame-walk site, independent of `is_valid_enip_frame`
- `test_command_counts_single_site_not_doubled`: valid known-cmd frame → `command_counts[cmd]==1`
  (not 2); confirms no duplicate increment in `process_pdu` (single canonical site)
- Both tests pass green

**Tests in recording:**
- `test_command_counts_increments_for_unknown_command`
- `test_command_counts_single_site_not_doubled`

---

### AC-ALL-frame-walk-28-green

Master green-run for the full `mod frame_walk` suite — 28 tests covering all STORY-137
acceptance criteria.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests frame_walk`
- All 28 tests in `mod frame_walk` execute and pass
- Test result line: `test result: ok. 28 passed; 0 failed; 0 ignored`

**Tests in recording (all 28):**
- `test_byte_walk_resync_24_garbage_bytes_then_valid_frame`
- `test_byte_walk_resync_invalid_command`
- `test_byte_walk_resync_to_valid_frame_same_segment`
- `test_carry_buffer_partial_header`
- `test_carry_buffer_three_segments_one_frame`
- `test_carry_buffer_two_frames_one_segment`
- `test_carry_cap_does_not_fire_under_spec_algorithm`
- `test_carry_overflow_third_malformed_fires_t0814_before_latch`
- `test_carry_stays_bounded_below_cap`
- `test_command_counts_increments_for_unknown_command`
- `test_command_counts_single_site_not_doubled`
- `test_invalid_frame_increments_malformed_count`
- `test_max_enip_carry_bytes_is_600`
- `test_multi_call_carry_residue_counting`
- `test_non_enip_flag_permanent`
- `test_non_enip_not_latched_at_carry_cap`
- `test_oversize_frame_does_not_set_non_enip`
- `test_oversize_frame_skip_continue`
- `test_oversize_frame_skip_then_valid_frame_processed`
- `test_parse_errors_not_reset_on_window_expiry`
- `test_subframe_accumulation_keeps_carry_bounded_no_latch`
- `test_t0814_does_not_fire_below_threshold`
- `test_t0814_fires_at_threshold`
- `test_t0814_fires_on_third_byte_walk_reject`
- `test_t0814_non_enip_not_set_at_threshold`
- `test_t0814_one_shot_guard_per_window`
- `test_t0814_refire_after_window_reset`
- `test_valid_frame_no_malformed_count`

---

## Full frame_walk Test Suite Summary

All 28 tests in `mod frame_walk` pass at recording time:

```
test frame_walk::test_byte_walk_resync_24_garbage_bytes_then_valid_frame ... ok
test frame_walk::test_byte_walk_resync_invalid_command ... ok
test frame_walk::test_byte_walk_resync_to_valid_frame_same_segment ... ok
test frame_walk::test_carry_buffer_partial_header ... ok
test frame_walk::test_carry_buffer_three_segments_one_frame ... ok
test frame_walk::test_carry_buffer_two_frames_one_segment ... ok
test frame_walk::test_carry_cap_does_not_fire_under_spec_algorithm ... ok
test frame_walk::test_carry_overflow_third_malformed_fires_t0814_before_latch ... ok
test frame_walk::test_carry_stays_bounded_below_cap ... ok
test frame_walk::test_command_counts_increments_for_unknown_command ... ok
test frame_walk::test_command_counts_single_site_not_doubled ... ok
test frame_walk::test_invalid_frame_increments_malformed_count ... ok
test frame_walk::test_max_enip_carry_bytes_is_600 ... ok
test frame_walk::test_multi_call_carry_residue_counting ... ok
test frame_walk::test_non_enip_flag_permanent ... ok
test frame_walk::test_non_enip_not_latched_at_carry_cap ... ok
test frame_walk::test_oversize_frame_does_not_set_non_enip ... ok
test frame_walk::test_oversize_frame_skip_continue ... ok
test frame_walk::test_oversize_frame_skip_then_valid_frame_processed ... ok
test frame_walk::test_parse_errors_not_reset_on_window_expiry ... ok
test frame_walk::test_subframe_accumulation_keeps_carry_bounded_no_latch ... ok
test frame_walk::test_t0814_does_not_fire_below_threshold ... ok
test frame_walk::test_t0814_fires_at_threshold ... ok
test frame_walk::test_t0814_fires_on_third_byte_walk_reject ... ok
test frame_walk::test_t0814_non_enip_not_set_at_threshold ... ok
test frame_walk::test_t0814_one_shot_guard_per_window ... ok
test frame_walk::test_t0814_refire_after_window_reset ... ok
test frame_walk::test_valid_frame_no_malformed_count ... ok

test result: ok. 28 passed; 0 failed; 0 ignored
```

---

## Deferred / Not Applicable

- AC-137-002 carry-overflow `is_non_enip` latch: the latch sequence is implemented per
  BC-2.17.016 Post-4/Inv-4 but is provably unreachable under the spec frame-walk algorithm
  (RULING-137-002: `spec-defect-is_non_enip-dead-latch`, target v0.12.0). The renamed
  bounded-carry tests (`test_carry_stays_bounded_below_cap`,
  `test_carry_cap_does_not_fire_under_spec_algorithm`,
  `test_subframe_accumulation_keeps_carry_bounded_no_latch`) verify carry boundedness and
  confirm the latch does not fire; these are captured in the `AC-002-carry-bounded` recording.
  The T0814-before-latch ordering constraint (BC-2.17.018 EC-007) is demonstrated by
  `test_carry_overflow_third_malformed_fires_t0814_before_latch` in the same recording.

## Approach Note

**Test-run recordings used (not pcap-driven CLI output).** This is the established pattern for
STORY-13x demos: the ENIP analyzer is a pure-core library with no standalone CLI entry point
for injecting hand-crafted CIP/ENIP frames. The acceptance criteria are fully expressed as unit
tests in `tests/enip_analyzer_tests.rs`. Recordings show `cargo test` output filtered to
relevant test names and the final `test result:` line, identical to STORY-134, STORY-135, and
STORY-136 precedent.
