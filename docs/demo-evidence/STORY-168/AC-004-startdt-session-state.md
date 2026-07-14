# AC-168-004 — STARTDT-act/con Sets session_started=true; No Finding

**Story:** STORY-168: IEC-104 Frame Format Discrimination + U-Format Session State Machine  
**AC:** AC-168-004  
**Traces to:** BC-2.19.010 postconditions 1–4 and invariant 1  
**Wave:** 77

---

## Acceptance Criterion

- Given a U-format frame with CF1=0x07 (STARTDT-act) or CF1=0x0B (STARTDT-con)
- When `process_u_frame(&mut state, cf1)` is called
- Then `Iec104FlowState::session_started` is set to `true`; no finding is emitted
- Receiving STARTDT-act when already started is idempotent (state remains true, no finding)
- STARTDT-con (CF1=0x0B) is also recognized; sets session_started=true if not already set

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests story_168::test_BC_2_19_010
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_168::test_BC_2_19_010_startdt_act_sets_session_started_true_on_fresh_flow ... ok
test story_168::test_BC_2_19_010_startdt_act_idempotent_when_already_started ... ok
test story_168::test_BC_2_19_010_startdt_con_sets_session_started_true_on_fresh_flow ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 61 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | CF1 | Prior State | Post State | Finding | Condition Exercised |
|-----------|-----|-------------|------------|---------|---------------------|
| `test_BC_2_19_010_startdt_act_sets_session_started_true_on_fresh_flow` | `0x07` | `session_started=false` | `session_started=true` | `None` | STARTDT-act on fresh flow (BC-2.19.010 PC1-PC3) |
| `test_BC_2_19_010_startdt_act_idempotent_when_already_started` | `0x07` | `session_started=true` | `session_started=true` | `None` | STARTDT-act idempotent (BC-2.19.010 invariant 1) |
| `test_BC_2_19_010_startdt_con_sets_session_started_true_on_fresh_flow` | `0x0B` | `session_started=false` | `session_started=true` | `None` | STARTDT-con on fresh flow (BC-2.19.010 PC4) |

---

## Success-Path Demonstration

### STARTDT-act (CF1=0x07): Fresh Flow

```
Precondition:  state.session_started == false
Input:         CF1=0x07 (STARTDT-act; bits1:0=0b11 → UFormat)
Call:          process_u_frame(&mut state, 0x07)

Postcondition: state.session_started == true
Finding:       None
```

BC-2.19.010 postcondition 1: `session_started=true` after STARTDT-act.
BC-2.19.010 postcondition 3: no finding emitted (STARTDT is expected IEC-104 behavior).

### STARTDT-act: Idempotent (BC-2.19.010 invariant 1)

```
Precondition:  state.session_started == true   (already started)
Input:         CF1=0x07 (duplicate STARTDT-act)
Call:          process_u_frame(&mut state, 0x07)

Postcondition: state.session_started == true   (unchanged)
Finding:       None
```

### STARTDT-con (CF1=0x0B): Fresh Flow (BC-2.19.010 postcondition 4)

```
Precondition:  state.session_started == false
Input:         CF1=0x0B (STARTDT-con; RTU acknowledges session start)
Call:          process_u_frame(&mut state, 0x0B)

Postcondition: state.session_started == true
Finding:       None
```

---

## Verdict

AC-168-004: **PASS** — 3/3 BC-2.19.010 tests green; STARTDT-act fresh-flow, idempotent, and STARTDT-con all correctly set session_started=true with no finding.
