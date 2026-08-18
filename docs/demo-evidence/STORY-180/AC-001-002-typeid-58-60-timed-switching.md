# AC-180-001 / AC-180-002 — TypeIDs 58–60 (C_SC_TA/C_DC_TA/C_RC_TA): T1692.001 Only, No T0836

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64  
**ACs:** AC-180-001, AC-180-002  
**Traces to:** BC-2.19.029 postconditions 1–3; invariant 2  
**Wave:** 85

---

## Acceptance Criteria

**AC-180-001:** TypeIDs 58–60 emit exactly one T1692.001 Possible finding with CASDU and
first_ioa evidence — identical parity to untimed arm 45..=47.

**AC-180-002:** TypeIDs 58–60 do NOT emit T0836 (switching commands are binary control,
not parameter writes) — mirrors BC-2.19.019 Invariant 2.

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_029"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 10 tests
test story_180::test_BC_2_19_029_type_id_58_count_zero_still_emits ... ok
test story_180::test_BC_2_19_029_timed_summary_contains_time_tagged_qualifier ... ok
test story_180::test_BC_2_19_029_type_id_58_emits_t1692_001_only ... ok
test story_180::test_BC_2_19_029_timed_summary_differs_from_untimed_twin ... ok
test story_180::test_BC_2_19_029_casdu_first_ioa_evidence ... ok
test story_180::test_BC_2_19_029_type_id_58_verdict_confidence_category ... ok
test story_180::test_BC_2_19_029_type_id_60_cot_test_suffix ... ok
test story_180::test_BC_2_19_029_type_id_59_first_ioa_none_no_first_ioa_evidence ... ok
test story_180::test_BC_2_19_029_type_id_59_emits_t1692_001_only ... ok
test story_180::test_BC_2_19_029_type_id_60_emits_t1692_001_only ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 238 filtered out; finished in 0.00s
```

Result: **10/10 PASS**

---

## Test Coverage

### AC-180-001: Exactly One T1692.001 Possible Finding per TypeID (BC-2.19.029 PC1 + PC3)

| Test Name | TypeID / Condition | Assertion | Result |
|-----------|-------------------|-----------|--------|
| `test_BC_2_19_029_type_id_58_emits_t1692_001_only` | TypeID=58 (C_SC_TA_1) | exactly 1 finding; T1692.001 present | PASS |
| `test_BC_2_19_029_type_id_59_emits_t1692_001_only` | TypeID=59 (C_DC_TA_1) | exactly 1 finding; T1692.001 present | PASS |
| `test_BC_2_19_029_type_id_60_emits_t1692_001_only` | TypeID=60 (C_RC_TA_1) | exactly 1 finding; T1692.001 present | PASS |
| `test_BC_2_19_029_type_id_58_verdict_confidence_category` | TypeID=58 | Verdict::Possible, Confidence::Medium, ThreatCategory::Impact | PASS |
| `test_BC_2_19_029_casdu_first_ioa_evidence` | TypeID=58, casdu=1, first_ioa=Some(100) | evidence contains "CASDU=1" and "first_ioa=100" | PASS |
| `test_BC_2_19_029_type_id_59_first_ioa_none_no_first_ioa_evidence` | TypeID=59, first_ioa=None | evidence contains "CASDU=" but NOT "first_ioa=" (EC-008) | PASS |

### AC-180-002: No T0836 for Switching Commands (BC-2.19.029 invariant 2)

The `test_BC_2_19_029_type_id_58_emits_t1692_001_only`,
`test_BC_2_19_029_type_id_59_emits_t1692_001_only`, and
`test_BC_2_19_029_type_id_60_emits_t1692_001_only` tests each assert `findings.len() == 1`,
which proves T0836 is never emitted (exactly one finding, not two).

---

## Dispatch Behavior Summary

| TypeID | IEC-104 Name | Findings | MITRE Techniques | Verdict |
|--------|-------------|----------|-----------------|---------|
| 58 | C_SC_TA_1 (timed single-point switching) | 1 | T1692.001 | Possible |
| 59 | C_DC_TA_1 (timed double-point switching) | 1 | T1692.001 | Possible |
| 60 | C_RC_TA_1 (timed regulating step) | 1 | T1692.001 | Possible |

---

## Verdict

AC-180-001: **PASS** — All three timed switching TypeIDs (58, 59, 60) emit exactly one
T1692.001 Possible finding with CASDU and conditional first_ioa evidence.

AC-180-002: **PASS** — T0836 is never emitted for TypeIDs 58–60; single-finding assertion
in each test is the negative proof.
