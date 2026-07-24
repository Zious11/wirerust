# AC-180-008 — Emission is Count-Independent: One Finding Set per ASDU Regardless of VSQ Object Count

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64  
**AC:** AC-180-008  
**Traces to:** BC-2.19.029 postcondition 5 and invariant 3; BC-2.19.030 postcondition 6 and invariant 3  
**Wave:** 85

---

## Acceptance Criterion

- Given an I-format ASDU with TypeID in {58..=64} and `asdu.count == 0`
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then the same finding(s) are still emitted as for `count > 0` — emission is per-ASDU,
  not per-object

---

## Test Suite Execution — TypeID 58, count=0

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_029_type_id_58_count_zero"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 1 test
test story_180::test_BC_2_19_029_type_id_58_count_zero_still_emits ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 247 filtered out; finished in 0.00s
```

---

## Test Suite Execution — TypeID 61, count=0

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_030_type_id_61_count_zero"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 1 test
test story_180::test_BC_2_19_030_type_id_61_count_zero_still_emits_two_findings ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 247 filtered out; finished in 0.00s
```

---

## Test Coverage

| Test Name | TypeID | asdu.count | Assertion | Result |
|-----------|--------|------------|-----------|--------|
| `test_BC_2_19_029_type_id_58_count_zero_still_emits` | 58 (C_SC_TA_1) | 0 | 1 finding emitted (EC-011: count-independent) | PASS |
| `test_BC_2_19_030_type_id_61_count_zero_still_emits_two_findings` | 61 (C_SE_TA_1) | 0 | 2 findings emitted (T1692.001 + T0836); count-independent | PASS |

---

## Implementation Anchor

The new detection arms operate on `asdu.type_id` and `asdu.casdu`/`asdu.first_ioa` only.
The `asdu.count` field (VSQ object count) is not consulted by the detection arms — findings
are emitted per-ASDU, not per-object. This is the same design as the untimed arms (45..=47
and 48..=51), which also do not gate on count.

---

## Verdict

AC-180-008: **PASS** — Both count=0 tests pass. TypeID=58 with count=0 still emits one
T1692.001 finding. TypeID=61 with count=0 still emits two findings (T1692.001 + T0836).
Emission is confirmed per-ASDU, count-independent.
