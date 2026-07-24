# AC-180-006 — TypeIDs 52–57 and 65–99 Still Produce Zero Findings (BC-2.19.022 v1.1 Regression Guard)

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64  
**AC:** AC-180-006  
**Traces to:** BC-2.19.022 v1.1 invariant 1; BC-2.19.029 invariant 6; BC-2.19.030 invariant 6  
**Wave:** 85

---

## Acceptance Criterion

- TypeIDs {52–57} (reserved, below new arms) and {65–99} (unhandled, above new arms)
  remain in the silently-logged set — zero findings emitted
- Regression guard: TypeIDs 52, 57, 65, 99 each explicitly tested
- Untimed twins (45, 51) still produce correct findings (no regression)

---

## Test Suite Execution — BC-2.19.022 v1.1 Neighbor Silence

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_022_v1_1"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 4 tests
test story_180::test_BC_2_19_022_v1_1_type_id_57_no_finding ... ok
test story_180::test_BC_2_19_022_v1_1_type_id_65_no_finding ... ok
test story_180::test_BC_2_19_022_v1_1_type_id_52_no_finding ... ok
test story_180::test_BC_2_19_022_v1_1_type_id_99_no_finding ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 244 filtered out; finished in 0.00s
```

Result: **4/4 PASS** (silence regression guard green)

---

## Test Suite Execution — Untimed Twin Regression (BC-2.19.019 parity unchanged)

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_019_v1_1"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 2 tests
test story_180::test_BC_2_19_019_v1_1_regression_type_id_45_still_one_finding ... ok
test story_180::test_BC_2_19_019_v1_1_regression_type_id_51_still_two_findings ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 246 filtered out; finished in 0.00s
```

Result: **2/2 PASS** (untimed twins unaffected)

---

## Test Coverage

### Silent-range neighbors (negative path — error path)

| Test Name | TypeID | Boundary Role | Assertion | Result |
|-----------|--------|---------------|-----------|--------|
| `test_BC_2_19_022_v1_1_type_id_52_no_finding` | 52 (RESERVED) | lower bound of 52–57 block | 0 findings | PASS |
| `test_BC_2_19_022_v1_1_type_id_57_no_finding` | 57 (RESERVED) | upper neighbor just below arm 58 | 0 findings | PASS |
| `test_BC_2_19_022_v1_1_type_id_65_no_finding` | 65 (unhandled) | lower neighbor just above arm 64 | 0 findings | PASS |
| `test_BC_2_19_022_v1_1_type_id_99_no_finding` | 99 (unhandled) | upper bound of 65–99 block | 0 findings | PASS |

### Untimed twin regression guard

| Test Name | TypeID | Assertion | Result |
|-----------|--------|-----------|--------|
| `test_BC_2_19_019_v1_1_regression_type_id_45_still_one_finding` | 45 (C_SC_NA_1) | 1 finding; T1692.001 present; arm 45..=47 unaffected | PASS |
| `test_BC_2_19_019_v1_1_regression_type_id_51_still_two_findings` | 51 (C_BO_NA_1) | 2 findings; T1692.001 + T0836 present; arm 48..=51 unaffected | PASS |

---

## Verdict

AC-180-006: **PASS** — TypeIDs 52, 57, 65, and 99 all produce zero findings.
Boundary silence (TypeID=57 just below new arm, TypeID=65 just above new arm) confirmed.
Untimed twin arms 45..=47 and 48..=51 remain unaffected by the new arms.
