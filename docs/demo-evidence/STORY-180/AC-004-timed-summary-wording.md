# AC-180-004 — Timed-Variant Summary Wording Distinguishes from Untimed Twin Summaries

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64  
**AC:** AC-180-004  
**Traces to:** BC-2.19.029 postcondition 4; BC-2.19.030 postconditions 4 and 5  
**Wave:** 85

---

## Acceptance Criterion

- For TypeIDs 58–60: the T1692.001 finding `summary` field uses the "time-tagged" qualifier
  and names the timed mnemonics (C_SC_TA/C_DC_TA/C_RC_TA)
- For TypeIDs 61–64: both the T1692.001 and T0836 finding `summary` fields name the timed
  mnemonics (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA)
- Neither timed summary string is identical to the corresponding untimed arm's summary —
  analysts can distinguish timed from untimed findings in output

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests "timed_summary"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 4 tests
test story_180::test_BC_2_19_029_timed_summary_contains_time_tagged_qualifier ... ok
test story_180::test_BC_2_19_029_timed_summary_differs_from_untimed_twin ... ok
test story_180::test_BC_2_19_030_timed_summaries_contain_time_tagged_and_mnemonics ... ok
test story_180::test_BC_2_19_030_timed_summaries_differ_from_untimed_twin ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 244 filtered out; finished in 0.00s
```

Result: **4/4 PASS**

---

## Test Coverage

| Test Name | Scope | Assertion | Result |
|-----------|-------|-----------|--------|
| `test_BC_2_19_029_timed_summary_contains_time_tagged_qualifier` | TypeID=58 T1692.001 summary | summary contains "time-tagged" and "C_SC_TA/C_DC_TA/C_RC_TA" | PASS |
| `test_BC_2_19_029_timed_summary_differs_from_untimed_twin` | TypeID=58 vs TypeID=45 T1692.001 summary | timed summary != untimed arm-45 summary | PASS |
| `test_BC_2_19_030_timed_summaries_contain_time_tagged_and_mnemonics` | TypeID=61 both findings | both summaries contain "time-tagged" and "C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA" | PASS |
| `test_BC_2_19_030_timed_summaries_differ_from_untimed_twin` | TypeID=61 vs TypeID=48 summaries | timed T1692.001 summary != untimed arm-48 T1692.001 summary | PASS |

---

## Source-Level Verification

Summary string for arm 58..=60 (src/analyzer/iec104.rs):
```
"IEC-104 time-tagged control command TypeID={type_id} \
 (C_SC_TA/C_DC_TA/C_RC_TA): time-tagged switching control command \
 observed on passive monitor \
 (T1692.001 unauthorized command message; BC-2.19.029)"
```

Summary string for arm 61..=64 T1692.001 (src/analyzer/iec104.rs):
```
"IEC-104 time-tagged control command TypeID={type_id} \
 (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA): time-tagged set-point or bitstring \
 write command observed on passive monitor \
 (T1692.001 unauthorized command message; BC-2.19.030)"
```

Summary string for arm 61..=64 T0836 (src/analyzer/iec104.rs):
```
"IEC-104 time-tagged parameter modification TypeID={type_id} \
 (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA): time-tagged set-point or bitstring \
 write modifying ICS control parameter on passive monitor \
 (T0836 modify parameter; BC-2.19.030 postcondition 2)"
```

Confirmed by grep:
```
grep -n "C_SC_TA/C_DC_TA/C_RC_TA\|C_SE_TA/C_SE_TB" src/analyzer/iec104.rs
```
Returns lines 857–860 and 904–907, 920–923 — all contain "time-tagged".

---

## Verdict

AC-180-004: **PASS** — All four summary-wording tests pass. Timed summaries include
"time-tagged" qualifier and timed-arm mnemonics; diff tests confirm no string identity
with the untimed twin arm summaries.
