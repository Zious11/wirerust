# AC-172-003 — Frame-Walk Loop: Multi-Frame Dispatch + PC2 Effects

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
**AC:** AC-172-003
**Traces to:** BC-2.19.026 postconditions 1–3; F-172-002
**Wave:** 81

---

## Acceptance Criterion

- Given data containing multiple complete APCI frames
- When `on_data(flow_key, data, ts, direction)` is called
- Then all complete frames are parsed and dispatched sequentially
- Remaining incomplete bytes are stashed into the directional carry buffer
- Dispatch-effect tests verify that each parsed frame correctly invokes downstream handlers
  (session state machine, ASDU type detection, sequence gap checker)

---

## Test Suite Execution — frame-walk multi-frame + dispatch effects

Command (multi-frame basic):
```
cargo test --test iec104_analyzer_tests "BC_2_19_026_multiple"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 1 test
test story_172::test_BC_2_19_026_multiple_complete_frames_processed_sequentially ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 191 filtered out; finished in 0.00s
```

Command (EC-009 three-frame):
```
cargo test --test iec104_analyzer_tests "BC_2_19_026_ec_009"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 1 test
test story_172::test_BC_2_19_026_ec_009_back_to_back_three_frames ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 191 filtered out; finished in 0.00s
```

Command (dispatch-effect tests):
```
cargo test --test iec104_analyzer_tests "pc2_dispatch"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 6 tests
test story_172::test_BC_2_19_026_pc2_dispatch_startdt_act_sets_session_started ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_type105_i_frame_emits_t0827 ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_multi_frame_startdt_plus_type105_joint_effects ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_stopdt_act_after_startdt_emits_t0881 ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_ns_desync_via_on_data_emits_t1692_001 ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_type45_control_command_emits_t1692_001 ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 186 filtered out; finished in 0.00s
```

Result: **8/8 PASS** (1 + 1 + 6)

---

## Test Coverage

### Basic Multi-Frame Processing

| Test Name | Scenario | Assertion | Result |
|-----------|----------|-----------|--------|
| `test_BC_2_19_026_multiple_complete_frames_processed_sequentially` | Two complete STARTDT frames back-to-back in one delivery | Both frames dispatched; findings from both included in output | PASS |
| `test_BC_2_19_026_ec_009_back_to_back_three_frames` | Three complete STARTDT frames back-to-back | All 3 dispatched sequentially; EC-009 | PASS |

### Dispatch-Effect Tests (F-172-002)

| Test Name | BC Invoked | Observable Effect | Result |
|-----------|-----------|-------------------|--------|
| `test_BC_2_19_026_pc2_dispatch_startdt_act_sets_session_started` | BC-2.19.010 | STARTDT_ACT U-frame delivered via `on_data` sets `session_started=true` on the flow state | PASS |
| `test_BC_2_19_026_pc2_dispatch_stopdt_act_after_startdt_emits_t0881` | BC-2.19.011 | STOPDT_ACT after STARTDT session emits T0881 "Denial of Service / Stop" Possible | PASS |
| `test_BC_2_19_026_pc2_dispatch_type105_i_frame_emits_t0827` | BC-2.19.020 | Type-105 (C_RP_NA_1) I-frame decoded from ASDU body emits T0827 "Impact / Reset" Likely | PASS |
| `test_BC_2_19_026_pc2_dispatch_type45_control_command_emits_t1692_001` | BC-2.19.019 | Type-45 (C_SC_NA_1) control command I-frame emits T1692.001 "Unauthorized Message" Possible | PASS |
| `test_BC_2_19_026_pc2_dispatch_ns_desync_via_on_data_emits_t1692_001` | BC-2.19.024 | I-frame with N(S) gap > k=12 delivered via `on_data` emits T1692.001 Possible | PASS |
| `test_BC_2_19_026_pc2_dispatch_multi_frame_startdt_plus_type105_joint_effects` | BC-2.19.010 + BC-2.19.020 | Concatenated STARTDT_ACT + Type-105 I-frame → both dispatched; `session_started=true` and T0827 both appear in findings | PASS |

---

## Frame-Walk Loop Mechanics

The loop invariant — driven by ADR-013 Decision 3 — terminates because every iteration
advances the cursor by at least 1 byte. For a complete valid frame the cursor advances
LEN+2 bytes. For an insufficient-data case the cursor is not advanced but the loop
returns immediately (remaining bytes stashed to carry). The loop cannot cycle on the
same position.

The multi-frame test demonstrates sequential processing: bytes for both frames arrive
in one `on_data` call; the loop completes the first frame, increments cursor, finds the
second 0x68 start byte, completes the second frame, increments cursor, then exhausts
the data and returns. No findings are dropped between frames.

---

## Verdict

AC-172-003: **PASS** — Frame-walk loop processes all complete APCI frames per `on_data`
call. Sequential dispatch confirmed for 2- and 3-frame back-to-back deliveries. All 6
dispatch-effect tests confirm that parsed frames invoke the correct downstream handlers
from STORY-167..171 and produce the expected findings or state mutations.
