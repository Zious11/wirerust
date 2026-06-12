---
story_id: STORY-107
bc: BC-2.15.016
date: 2026-06-11
recorder: demo-recorder
---

# STORY-107 Demo Evidence Report

DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds (BC-2.15.016)

## AC Coverage (6/6)

| AC | Test Name | Assertion | Recording | Result |
|----|-----------|-----------|-----------|--------|
| AC-001 | `test_carry_buffer_cap_at_292` | carry.len()==292, parse_errors==1, 3 bytes discarded | [GIF](AC-001-carry-buffer-cap.gif) / [WEBM](AC-001-carry-buffer-cap.webm) | PASS |
| AC-002 | `test_carry_buffer_frame_consumption` | 11 bytes remain after 10-byte frame drained, frame_count==1 | [GIF](AC-002-carry-frame-consumption.gif) / [WEBM](AC-002-carry-frame-consumption.webm) | PASS |
| AC-003 | `test_master_addrs_cap_at_64` | master_addrs_seen.len()==64 after 65th unique src addr | [GIF](AC-003-master-addrs-cap.gif) / [WEBM](AC-003-master-addrs-cap.webm) | PASS |
| AC-004 | `test_frame_count_increments` | frame_count==3 after 3 complete frames delivered in one call | [GIF](AC-004-frame-count-increments.gif) / [WEBM](AC-004-frame-count-increments.webm) | PASS |
| AC-005 | `test_pending_requests_eviction_at_256` | map.len()==256 after 257th insert; oldest (ts=0) evicted; new (dest=256) present | [GIF](AC-005-pending-requests-eviction.gif) / [WEBM](AC-005-pending-requests-eviction.webm) | PASS |
| AC-006 | `test_carry_drain_boundary_min_frame` | carry.len()==1 after min-frame (10B) drained; frame_count==1; no panic | [GIF](AC-006-carry-drain-boundary.gif) / [WEBM](AC-006-carry-drain-boundary.webm) | PASS |

## Edge Case Coverage (EC-001..EC-006)

| EC | Test Name | Assertion | Result |
|----|-----------|-----------|--------|
| EC-001 | `test_EC_001_partial_frame_in_carry` | carry.len()==7, frame_count==0 (partial frame, no consume) | PASS |
| EC-002 | `test_EC_002_two_complete_frames_one_call` | carry.len()==0, frame_count==2 (while-loop consumed both) | PASS |
| EC-003 | `test_EC_003_carry_291_plus_2_overflow` | carry.len()==292, parse_errors==1 (1 accepted, 1 discarded) | PASS |
| EC-004 | `test_EC_004_bailed_flow_is_noop` | is_non_dnp3==true, carry.len()==0 after delivery to bailed flow | PASS |
| EC-005 | `test_EC_005_pending_requests_tie_break_eviction` | map.len()==256; at least one of (0,0)/(1,0) evicted | PASS |
| EC-006 | `test_EC_006_invalid_length_byte_increments_parse_errors` | parse_errors==1, carry.len()==9 (drain-1 resync advanced past invalid LENGTH) | PASS |

Full test run output:

```
running 13 tests
test story_107::test_BC_2_15_016_is_master_frame_dir_bit ... ok
test story_107::test_EC_001_partial_frame_in_carry ... ok
test story_107::test_EC_002_two_complete_frames_one_call ... ok
test story_107::test_EC_003_carry_291_plus_2_overflow ... ok
test story_107::test_EC_004_bailed_flow_is_noop ... ok
test story_107::test_EC_005_pending_requests_tie_break_eviction ... ok
test story_107::test_EC_006_invalid_length_byte_increments_parse_errors ... ok
test story_107::test_carry_buffer_cap_at_292 ... ok
test story_107::test_carry_buffer_frame_consumption ... ok
test story_107::test_carry_drain_boundary_min_frame ... ok
test story_107::test_frame_count_increments ... ok
test story_107::test_master_addrs_cap_at_64 ... ok
test story_107::test_pending_requests_eviction_at_256 ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_flow_state_tests`

## Bounds-Safety Narrative

### Carry Buffer (MAX_DNP3_FRAME_LEN = 292 bytes)

The carry buffer accumulates raw wire bytes across `on_data` calls. The implementation enforces a hard cap: when `carry.len() + new_bytes.len() > 292`, only the bytes that fit are appended and `parse_errors` is incremented. This prevents adversarial partial-frame floods from exhausting per-flow memory. The cap is derived from the maximum DNP3 link-layer frame size (ADR-007 Decision 2), proved safe by VP-023 Sub-D.

AC-001 demonstrates the cap: pre-loading 290 bytes then delivering 5 more results in carry=292 and parse_errors=1 (3 bytes discarded). AC-006 demonstrates the VP-023 Sub-D boundary: the minimum frame (LENGTH=5, frame_len=10) drains cleanly with no panic or index-out-of-bounds. EC-003 demonstrates the exact 291+2=292 boundary (1 accepted, 1 discarded).

### Master Address Tracker (MAX_MASTER_ADDRS = 64)

`master_addrs_seen: Vec<u16>` tracks unique source addresses from master-direction frames (DIR=1, `control & 0x10 != 0`). Once the vec reaches 64 entries, new source addresses are silently ignored. This bounds the memory cost of tracking master devices against address-spoofing floods. AC-003 demonstrates the cap at exactly 64.

### Pending Requests Table (MAX_PENDING_REQUESTS = 256, oldest-eviction)

`pending_requests: HashMap<(u16, u8), u32>` (key=`(dest_addr, app_seq)`, value=`request_ts`) is bounded at 256 entries. When a new Control-class request would exceed the cap, the entry with the minimum `request_ts` (oldest) is evicted before the new entry is inserted. The evicted entry generates NO T1691.001 timeout event — this is the DoS-safe overflow behavior (BC-2.15.016 postcondition 10). AC-005 demonstrates the eviction: after 256 pre-seeded entries (ts 0..=255), a 257th delivery at ts=300 evicts the oldest (ts=0, key `(0,0)`) and inserts the new entry.

## STORY-106 Regression — CLEAN (36/36)

STORY-107 restructured `on_data` to implement the real carry-walk (accumulate → consume loop → master-addr tracking). This required modifying `src/analyzer/dnp3.rs` and could have broken STORY-106's parse-core contracts. The full STORY-106 regression suite confirms no regressions:

```
running 36 tests
test story_106::test_BC_2_15_001_invariant_parse_does_not_gate_on_sync ... ok
test story_106::test_BC_2_15_001_minimum_length_control_frame ... ok
test story_106::test_BC_2_15_001_trailing_bytes_not_decoded ... ok
test story_106::test_BC_2_15_002_boundary_sweep_all_short_lengths ... ok
test story_106::test_BC_2_15_002_ec001_zero_length_no_panic ... ok
test story_106::test_BC_2_15_002_ec002_nine_bytes_returns_none ... ok
test story_106::test_BC_2_15_003_ec003_invalid_sync_returns_some_with_raw_fields ... ok
test story_106::test_BC_2_15_003_ec004_broadcast_0xffff ... ok
test story_106::test_BC_2_15_004_ec005_length_4_rejected_by_gate ... ok
test story_106::test_BC_2_15_004_length_255_valid ... ok
test story_106::test_BC_2_15_004_length_zero_false ... ok
test story_106::test_BC_2_15_005_canonical_vectors ... ok
test story_106::test_BC_2_15_005_totality_sweep_all_256_values ... ok
test story_106::test_BC_2_15_006_ec007_direct_operate_nr_is_control ... ok
test story_106::test_BC_2_15_006_ec008_unsolicited_response_is_response ... ok
test story_106::test_BC_2_15_007_ec005_length_4_returns_none ... ok
test story_106::test_BC_2_15_007_ec006_length_255_returns_292 ... ok
test story_106::test_BC_2_15_007_length_14_direct_operate ... ok
test story_106::test_BC_2_15_007_result_bounds_all_valid_lengths ... ok
test story_106::test_BC_2_15_008_ec009_fir0_continuation_returns_false ... ok
test story_106::test_BC_2_15_008_fir_biconditional_all_256_transport_octets ... ok
test story_106::test_BC_2_15_009_ec010_sync_at_offset_2_triggers_bail ... ok
test story_106::test_BC_2_15_009_flow_state_defaults_to_not_bailed ... ok
test story_106::test_BC_2_15_009_valid_sync_no_bail ... ok
test story_106::test_classify_dnp3_fc_set_membership ... ok
test story_106::test_classify_dnp3_fc_total ... ok
test story_106::test_compute_dnp3_frame_len_formula ... ok
test story_106::test_desync_bail_non_dnp3_traffic ... ok
test story_106::test_fir_gating_extract_on_fir1_skip_on_fir0 ... ok
test story_106::test_has_user_data_link_fc_guard ... ok
test story_106::test_is_valid_dnp3_frame_header_biconditional ... ok
test story_106::test_on_data_fir_but_non_user_data_link_fc_no_extraction ... ok
test story_106::test_on_data_fir_gating_updates_counters ... ok
test story_106::test_parse_dnp3_dl_header_le_address_decode ... ok
test story_106::test_parse_dnp3_dl_header_rejects_truncated_input ... ok
test story_106::test_parse_dnp3_dl_header_returns_some_for_minimum_10_bytes ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_parse_core_tests`

## VP-023 Kani Formal Verification — SUCCESSFUL (4/4 harnesses)

VP-023 (DNP3 parse safety) covers 4 sub-properties verified by Kani model-checking. STORY-107 relies on VP-023 Sub-D (frame-len arithmetic bounds, [10,292]) for the carry-drain correctness claim. All 4 harnesses remain SUCCESSFUL after STORY-107's modifications to `src/analyzer/dnp3.rs`:

| Harness | Sub-Property | Checks | Result |
|---------|-------------|--------|--------|
| `verify_parse_dnp3_dl_header_safety` | Sub-A: parse no OOB for all lengths 0..=12 | 141 | SUCCESSFUL |
| `verify_is_valid_dnp3_frame_gate` | Sub-C: validity gate biconditional (sync AND length>=5) | 19 | SUCCESSFUL |
| `verify_classify_dnp3_fc_total` | Sub-B: FC classification totality over all 256 u8 values | 20 | SUCCESSFUL |
| `verify_compute_dnp3_frame_len` | Sub-D: frame_len in [10,292] for all valid LENGTH values | 30 | SUCCESSFUL |

Tool: Kani Rust Verifier 0.67.0 (CBMC 6.8.0)
Command: `cargo kani --harness kani_proofs::<name>` (run for each harness individually)

Sub-D is the critical invariant that makes AC-006's carry-drain safety claim formal: `compute_dnp3_frame_len(length)` returns values in `[10, 292]` for all valid inputs, guaranteeing that `carry.drain(..frame_len)` cannot index out of bounds when `carry.len() >= frame_len`.

## Tape Sources

All VHS tape scripts are committed alongside the recordings. Each tape:
- Hides the build step from the viewer
- Shows only the targeted `cargo test` invocation and its output
- Covers the specific AC named in the filename

| Tape | AC |
|------|----|
| [AC-001-carry-buffer-cap.tape](AC-001-carry-buffer-cap.tape) | AC-001 |
| [AC-002-carry-frame-consumption.tape](AC-002-carry-frame-consumption.tape) | AC-002 |
| [AC-003-master-addrs-cap.tape](AC-003-master-addrs-cap.tape) | AC-003 |
| [AC-004-frame-count-increments.tape](AC-004-frame-count-increments.tape) | AC-004 |
| [AC-005-pending-requests-eviction.tape](AC-005-pending-requests-eviction.tape) | AC-005 |
| [AC-006-carry-drain-boundary.tape](AC-006-carry-drain-boundary.tape) | AC-006 |
