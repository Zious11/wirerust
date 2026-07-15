# AC-171-003 — First I-Frame Sets Option<u16> Baseline with No Finding

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking  
**AC:** AC-171-003  
**Traces to:** BC-2.19.024 postcondition Path A; invariant 3 (mid-capture correctness)  
**Wave:** 80

---

## Acceptance Criterion

- Given `last_ns_c2s` or `last_ns_s2c` is `None` (fresh flow or mid-capture start)
- When the first I-format frame with any N(S) value is received
- Then the directional field is set to `Some(ns)`; NO finding is emitted unconditionally
- This is the mid-capture correctness guard: first observed N(S) may be arbitrary (e.g., 5000)
  and must never generate a desync finding regardless of its value

---

## Test Suite Execution — BC-2.19.024 Path A

Command:
```
cargo test --test iec104_analyzer_tests "path_a"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_171::test_BC_2_19_024_path_a_first_frame_c2s_ns_0_no_finding_state_becomes_some_0 ... ok
test story_171::test_BC_2_19_024_path_a_first_frame_s2c_ns_0_no_finding_state_becomes_some_0 ... ok
test story_171::test_BC_2_19_024_path_a_mid_capture_first_frame_c2s_ns_5000_no_finding_state_becomes_some_5000 ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 163 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | Initial State | N(S) Input | Expected State | Expected Findings | Result |
|-----------|--------------|------------|----------------|-------------------|--------|
| `test_BC_2_19_024_path_a_first_frame_c2s_ns_0_no_finding_state_becomes_some_0` | C2S: `None` | N(S)=0 | C2S: `Some(0)` | 0 findings | PASS |
| `test_BC_2_19_024_path_a_first_frame_s2c_ns_0_no_finding_state_becomes_some_0` | S2C: `None` | N(S)=0 | S2C: `Some(0)` | 0 findings | PASS |
| `test_BC_2_19_024_path_a_mid_capture_first_frame_c2s_ns_5000_no_finding_state_becomes_some_5000` | C2S: `None` | N(S)=5000 | C2S: `Some(5000)` | 0 findings (mid-capture guard) | PASS |

---

## Mid-Capture Correctness (Invariant 3)

The `None` sentinel is critical. A bare `u16` default of 0 would compute a gap of 5000
on the first real packet of a mid-capture session, generating a spurious T1692.001 finding.

The `Option<u16>` guard prevents this:

```
match last_ns_dir {
    None => {
        // Path A: first frame — set baseline, no finding
        *last_ns_dir = Some(current_ns);
    }
    Some(prev) => {
        // Path B or C: check gap
        ...
    }
}
```

Both the `N(S)=0` (fresh flow, EC-001) and `N(S)=5000` (mid-capture, EC-002) test vectors
confirm no finding is emitted from `None` state regardless of the N(S) value.

---

## Verdict

AC-171-003: **PASS** — All 3 Path-A tests green. `None → Some(ns)` baseline transition
confirmed for both C2S and S2C directions and for both fresh-flow (N(S)=0) and mid-capture
(N(S)=5000) scenarios. No finding emitted unconditionally on first frame.
