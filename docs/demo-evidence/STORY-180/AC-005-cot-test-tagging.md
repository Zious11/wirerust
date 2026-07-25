# AC-180-005 — cot_test=true Appends [TEST] Suffix to All Timed-Command Findings

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64  
**AC:** AC-180-005  
**Traces to:** BC-2.19.029 postcondition 6; BC-2.19.030 postcondition 7; BC-2.19.017 invariant 1  
**Wave:** 85

---

## Acceptance Criterion

- Given an I-format ASDU with TypeID in {58..=64} and `asdu.cot_test == true`
- Then the ` [TEST]` suffix is appended to all emitted findings' `summary` fields
- The existing post-emission loop at `detect_iec104_threats` (lines 1027–1030) covers all
  findings added during the call — no extra wiring needed in the new arms

---

## Test Suite Execution — TypeIDs 58–60 (cot_test=true, 1 finding tagged)

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_029_type_id_60_cot_test"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 1 test
test story_180::test_BC_2_19_029_type_id_60_cot_test_suffix ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 247 filtered out; finished in 0.00s
```

---

## Test Suite Execution — TypeID 64 (cot_test=true, 2 findings both tagged)

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_030_type_id_64_cot_test"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 1 test
test story_180::test_BC_2_19_030_type_id_64_cot_test_both_findings_tagged ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 247 filtered out; finished in 0.00s
```

---

## Test Coverage

| Test Name | TypeID / cot_test | Assertion | Result |
|-----------|-------------------|-----------|--------|
| `test_BC_2_19_029_type_id_60_cot_test_suffix` | TypeID=60, cot_test=true | T1692.001 summary ends with " [TEST]" (EC-009) | PASS |
| `test_BC_2_19_030_type_id_64_cot_test_both_findings_tagged` | TypeID=64, cot_test=true | BOTH T1692.001 and T0836 summaries end with " [TEST]" (EC-010) | PASS |

---

## Implementation Anchor

The [TEST] loop at `src/analyzer/iec104.rs` lines 1027–1030:
```rust
if asdu.cot_test {
    for f in &mut findings[start_idx..] {
        f.summary.push_str(" [TEST]");
    }
}
```
This runs after both new arms (58..=60 and 61..=64) push their findings. The loop iterates
over the entire `findings[start_idx..]` slice, so all findings added during the call are
tagged regardless of how many arms fired. No wiring was added to the new arms.

---

## Verdict

AC-180-005: **PASS** — Both cot_test tagging tests pass. The existing post-emission
[TEST] loop automatically covers the new timed-command arms with no extra implementation.
TypeID=60 with cot_test=true: 1 finding tagged. TypeID=64 with cot_test=true: both T1692.001
and T0836 findings tagged.
