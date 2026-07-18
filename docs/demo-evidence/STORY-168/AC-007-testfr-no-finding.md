# AC-168-007 — TESTFR-act and TESTFR-con Produce No Finding

**Story:** STORY-168: IEC-104 Frame Format Discrimination + U-Format Session State Machine  
**AC:** AC-168-007  
**Traces to:** BC-2.19.013 postconditions 1–2 and invariant 1  
**Wave:** 77

---

## Acceptance Criterion

- Given a U-format frame with CF1=0x43 (TESTFR-act) or CF1=0x83 (TESTFR-con)
- When `process_u_frame(&mut state, cf1)` is called
- Then no finding is emitted; session state is unchanged
- TESTFR is a keepalive mechanism; observation is normal IEC-104 behavior

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests story_168::test_BC_2_19_013
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_168::test_BC_2_19_013_testfr_act_emits_no_finding_canonical_vector ... ok
test story_168::test_BC_2_19_013_testfr_con_emits_no_finding_canonical_vector ... ok
test story_168::test_BC_2_19_013_invariant_testfr_does_not_modify_session_started ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 61 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | CF1 | Prior State | Post State | Finding | Condition Exercised |
|-----------|-----|-------------|------------|---------|---------------------|
| `test_BC_2_19_013_testfr_act_emits_no_finding_canonical_vector` | `0x43` | `session_started=false` | unchanged | `None` | TESTFR-act no-op (BC-2.19.013 PC1) |
| `test_BC_2_19_013_testfr_con_emits_no_finding_canonical_vector` | `0x83` | `session_started=false` | unchanged | `None` | TESTFR-con no-op (BC-2.19.013 PC1) |
| `test_BC_2_19_013_invariant_testfr_does_not_modify_session_started` | `0x43` | `session_started=true` | `session_started=true` | `None` | State-preservation invariant (BC-2.19.013 invariant 1) |

---

## Success-Path Demonstration

### TESTFR-act (CF1=0x43): No Finding

```
Precondition:  state.session_started == false
Input:         CF1=0x43 (TESTFR-act; bits1:0=0b11 → UFormat; bits7:2=0b010000)
Call:          process_u_frame(&mut state, 0x43)

Postcondition: state.session_started == false   (unchanged)
Finding:       None
```

### TESTFR-con (CF1=0x83): No Finding

```
Precondition:  state.session_started == false
Input:         CF1=0x83 (TESTFR-con; bits1:0=0b11 → UFormat; bits7:2=0b100000)
Call:          process_u_frame(&mut state, 0x83)

Postcondition: state.session_started == false   (unchanged)
Finding:       None
```

### State Preservation: TESTFR While Session Active (BC-2.19.013 invariant 1)

```
Precondition:  state.session_started == true    (session is live)
Input:         CF1=0x43 (TESTFR-act)
Call:          process_u_frame(&mut state, 0x43)

Postcondition: state.session_started == true    (unchanged; keepalive does not close session)
Finding:       None
```

TESTFR is a keepalive mechanism defined in IEC 60870-5-104 section 5.3.  Observing TESTFR is
normal IEC-104 behavior — it confirms the TCP/IP connection is alive and the RTU is responsive.
No anomaly or finding is warranted.

---

## Verdict

AC-168-007: **PASS** — 3/3 BC-2.19.013 tests green; TESTFR-act and TESTFR-con both produce no finding; session state is not modified by keepalive frames.
