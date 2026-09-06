# AC-180-003 — TypeIDs 61–64 (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA): T1692.001 + T0836 Both Possible

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64  
**AC:** AC-180-003  
**Traces to:** BC-2.19.030 postconditions 1–3  
**Wave:** 85

---

## Acceptance Criterion

- Given an I-format ASDU with TypeID in {61, 62, 63, 64}
- Then exactly two findings are emitted:
  1. T1692.001 "Unauthorized Message: Command Message" with Verdict::Possible, Confidence::Medium, ThreatCategory::Impact
  2. T0836 "Modify Parameter" with Verdict::Possible, Confidence::Medium, ThreatCategory::Impact
- Both findings' evidence vectors include CASDU and, when present, first_ioa
- T0836 is co-emitted because TypeIDs 61–64 are ICS parameter writes (set-point and bitstring output register writes)

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_030"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 11 tests
test story_180::test_BC_2_19_030_type_id_61_count_zero_still_emits_two_findings ... ok
test story_180::test_BC_2_19_030_timed_summaries_differ_from_untimed_twin ... ok
test story_180::test_BC_2_19_030_timed_summaries_contain_time_tagged_and_mnemonics ... ok
test story_180::test_BC_2_19_030_type_id_61_casdu_first_ioa_evidence_both_findings ... ok
test story_180::test_BC_2_19_030_type_id_64_emits_two_findings ... ok
test story_180::test_BC_2_19_030_type_id_62_emits_two_findings ... ok
test story_180::test_BC_2_19_030_type_id_61_verdict_confidence_category_both_findings ... ok
test story_180::test_BC_2_19_030_type_id_64_cot_test_both_findings_tagged ... ok
test story_180::test_BC_2_19_030_type_id_61_emits_two_findings ... ok
test story_180::test_BC_2_19_030_type_id_62_first_ioa_none_no_first_ioa_evidence ... ok
test story_180::test_BC_2_19_030_type_id_63_emits_two_findings ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 237 filtered out; finished in 0.00s
```

Result: **11/11 PASS**

---

## Test Coverage

### Exactly Two Findings per TypeID — T1692.001 + T0836

| Test Name | TypeID / Condition | Assertion | Result |
|-----------|-------------------|-----------|--------|
| `test_BC_2_19_030_type_id_61_emits_two_findings` | TypeID=61 (C_SE_TA_1) | exactly 2 findings; T1692.001 + T0836 present | PASS |
| `test_BC_2_19_030_type_id_62_emits_two_findings` | TypeID=62 (C_SE_TB_1) | exactly 2 findings; T1692.001 + T0836 present | PASS |
| `test_BC_2_19_030_type_id_63_emits_two_findings` | TypeID=63 (C_SE_TC_1) | exactly 2 findings; T1692.001 + T0836 present | PASS |
| `test_BC_2_19_030_type_id_64_emits_two_findings` | TypeID=64 (C_BO_TA_1, bitstring) | exactly 2 findings; T1692.001 + T0836 present | PASS |

### Verdict, Confidence, Category (BC-2.19.030 PC1 + PC2)

| Test Name | Condition | Assertion | Result |
|-----------|-----------|-----------|--------|
| `test_BC_2_19_030_type_id_61_verdict_confidence_category_both_findings` | TypeID=61 | Both findings: Verdict::Possible, Confidence::Medium, ThreatCategory::Impact | PASS |

### CASDU / first_ioa Evidence in Both Findings (BC-2.19.030 PC3)

| Test Name | Input | Assertion | Result |
|-----------|-------|-----------|--------|
| `test_BC_2_19_030_type_id_61_casdu_first_ioa_evidence_both_findings` | TypeID=61, casdu=5, first_ioa=Some(200) | Both findings contain "CASDU=5" and "first_ioa=200" | PASS |
| `test_BC_2_19_030_type_id_62_first_ioa_none_no_first_ioa_evidence` | TypeID=62, first_ioa=None | Both findings contain "CASDU=" but NOT "first_ioa=" | PASS |

---

## Dispatch Behavior Summary

| TypeID | IEC-104 Name | Findings | MITRE Techniques | Verdict |
|--------|-------------|----------|-----------------|---------|
| 61 | C_SE_TA_1 (timed set-point normalized value) | 2 | T1692.001 + T0836 | Possible |
| 62 | C_SE_TB_1 (timed set-point scaled value) | 2 | T1692.001 + T0836 | Possible |
| 63 | C_SE_TC_1 (timed set-point short float) | 2 | T1692.001 + T0836 | Possible |
| 64 | C_BO_TA_1 (timed bitstring of 32 bits) | 2 | T1692.001 + T0836 | Possible |

---

## Verdict

AC-180-003: **PASS** — All four timed set-point/bitstring TypeIDs (61, 62, 63, 64) emit
exactly two findings (T1692.001 Possible + T0836 Possible) with CASDU and conditional
first_ioa evidence in both findings, matching BC-2.19.030 postconditions 1–3.
