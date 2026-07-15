# AC-170-001 — Control TypeIDs 45–51: T1692.001 for All; T0836 Also for Set-Point/Bitstring TypeIDs 48–51

**Story:** STORY-170: IEC-104 Control Command Detection  
**AC:** AC-170-001  
**Traces to:** BC-2.19.019 postconditions 1–3; invariant 2  
**Wave:** 79

---

## Acceptance Criterion

- Given an I-format frame whose ASDU TypeID is in [45, 51]:
  - Switching commands 45–47 (C_SC, C_DC, C_RC): T1692.001 Possible only — 1 finding
  - Set-point/bitstring TypeIDs 48–51 (C_SE_NA, C_SE_NB, C_SE_NC, C_BO_NA): T1692.001 Possible + T0836 Possible — 2 findings
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then mitre_techniques, verdict=Possible, category=Impact are set correctly
- And finding evidence contains CASDU and first_ioa target-address context

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_019
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 15 tests
test story_170::test_BC_2_19_019_invariant_all_findings_have_possible_verdict ... ok
test story_170::test_BC_2_19_019_invariant_setpoint_types_48_to_51_each_emit_two_findings ... ok
test story_170::test_BC_2_19_019_invariant_switching_types_45_to_47_each_emit_one_finding ... ok
test story_170::test_BC_2_19_019_invariant_t1692001_present_for_all_types_45_to_51 ... ok
test story_170::test_BC_2_19_019_type45_c_sc_na1_emits_exactly_one_finding ... ok
test story_170::test_BC_2_19_019_type45_does_not_emit_t0836 ... ok
test story_170::test_BC_2_19_019_type45_emits_t1692001_possible_impact ... ok
test story_170::test_BC_2_19_019_type46_c_dc_na1_emits_t1692001_only ... ok
test story_170::test_BC_2_19_019_type47_c_rc_na1_emits_t1692001_only ... ok
test story_170::test_BC_2_19_019_type48_c_se_na1_emits_exactly_two_findings ... ok
test story_170::test_BC_2_19_019_type48_emits_t0836_possible ... ok
test story_170::test_BC_2_19_019_type48_emits_t1692001_possible ... ok
test story_170::test_BC_2_19_019_type49_c_se_nb1_emits_t1692001_and_t0836 ... ok
test story_170::test_BC_2_19_019_type50_c_se_nc1_emits_t1692001_and_t0836 ... ok
test story_170::test_BC_2_19_019_type51_c_bo_na1_emits_exactly_two_findings_canonical_vector ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 121 filtered out; finished in 0.00s
```

Result: **15/15 PASS**

---

## CASDU / first_ioa Evidence Tests

Command:
```
cargo test --test iec104_analyzer_tests "story_170::test_F_170"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 2 tests
test story_170::test_F_170_001_casdu_appears_in_finding_evidence_for_control_type ... ok
test story_170::test_F_170_001_first_ioa_appears_in_finding_evidence_when_some ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 134 filtered out; finished in 0.00s
```

Result: **2/2 PASS**

---

## Test Coverage

### Switching Commands 45–47 (T1692.001 only — 1 finding each)

| Test Name | TypeID / Condition | Assertion | Result |
|-----------|-------------------|-----------|--------|
| `test_BC_2_19_019_type45_c_sc_na1_emits_exactly_one_finding` | TypeID=45 (C_SC_NA_1) | exactly 1 finding emitted | PASS |
| `test_BC_2_19_019_type45_emits_t1692001_possible_impact` | TypeID=45 | mitre=T1692.001, verdict=Possible, category=Impact | PASS |
| `test_BC_2_19_019_type45_does_not_emit_t0836` | TypeID=45 (switching) | NO T0836 emitted (negative path) | PASS |
| `test_BC_2_19_019_type46_c_dc_na1_emits_t1692001_only` | TypeID=46 (C_DC_NA_1) | 1 finding, T1692.001, no T0836 | PASS |
| `test_BC_2_19_019_type47_c_rc_na1_emits_t1692001_only` | TypeID=47 (C_RC_NA_1) | 1 finding, T1692.001, no T0836 (EC-007) | PASS |

### Set-Point/Bitstring Commands 48–51 (T1692.001 + T0836 — 2 findings each)

| Test Name | TypeID / Condition | Assertion | Result |
|-----------|-------------------|-----------|--------|
| `test_BC_2_19_019_type48_c_se_na1_emits_exactly_two_findings` | TypeID=48 (C_SE_NA_1) | exactly 2 findings | PASS |
| `test_BC_2_19_019_type48_emits_t1692001_possible` | TypeID=48 | T1692.001 Possible in findings | PASS |
| `test_BC_2_19_019_type48_emits_t0836_possible` | TypeID=48 | T0836 Possible in findings | PASS |
| `test_BC_2_19_019_type49_c_se_nb1_emits_t1692001_and_t0836` | TypeID=49 (C_SE_NB_1) | T1692.001 + T0836 both present | PASS |
| `test_BC_2_19_019_type50_c_se_nc1_emits_t1692001_and_t0836` | TypeID=50 (C_SE_NC_1) | T1692.001 + T0836 both present | PASS |
| `test_BC_2_19_019_type51_c_bo_na1_emits_exactly_two_findings_canonical_vector` | TypeID=51 (C_BO_NA_1) | exactly 2 findings; BC canonical vector | PASS |

### Invariants

| Test Name | Invariant | Assertion | Result |
|-----------|-----------|-----------|--------|
| `test_BC_2_19_019_invariant_switching_types_45_to_47_each_emit_one_finding` | BC-2.19.019 inv2 | all of 45/46/47 → 1 finding only | PASS |
| `test_BC_2_19_019_invariant_setpoint_types_48_to_51_each_emit_two_findings` | BC-2.19.019 PC2 | all of 48/49/50/51 → exactly 2 findings | PASS |
| `test_BC_2_19_019_invariant_t1692001_present_for_all_types_45_to_51` | BC-2.19.019 PC1 | T1692.001 present for every TypeID in range | PASS |
| `test_BC_2_19_019_invariant_all_findings_have_possible_verdict` | BC-2.19.019 PC1 | all findings from 45–51 have Verdict::Possible | PASS |

### CASDU / first_ioa Evidence (BC-2.19.019 postcondition 3)

| Test Name | Input | Assertion | Result |
|-----------|-------|-----------|--------|
| `test_F_170_001_casdu_appears_in_finding_evidence_for_control_type` | TypeID=48, casdu=100 | evidence contains "CASDU=100" | PASS |
| `test_F_170_001_first_ioa_appears_in_finding_evidence_when_some` | TypeID=48, first_ioa=Some(0x1234=4660) | evidence contains "first_ioa=4660" | PASS |

---

## Dispatch Behavior Summary

| TypeID | Name | Findings | MITRE Techniques | Verdict |
|--------|------|----------|-----------------|---------|
| 45 | C_SC_NA_1 (single-point switching) | 1 | T1692.001 | Possible |
| 46 | C_DC_NA_1 (double-point switching) | 1 | T1692.001 | Possible |
| 47 | C_RC_NA_1 (regulating step) | 1 | T1692.001 | Possible |
| 48 | C_SE_NA_1 (set-point normalized) | 2 | T1692.001 + T0836 | Possible |
| 49 | C_SE_NB_1 (set-point scaled) | 2 | T1692.001 + T0836 | Possible |
| 50 | C_SE_NC_1 (set-point float) | 2 | T1692.001 + T0836 | Possible |
| 51 | C_BO_NA_1 (bitstring 32-bit) | 2 | T1692.001 + T0836 | Possible |

---

## Verdict

AC-170-001: **PASS** — All 17 tests (15 BC-2.19.019 + 2 F_170 evidence) green.
Switching commands 45–47 emit T1692.001 only. Set-point/bitstring 48–51 emit T1692.001 + T0836.
CASDU and first_ioa appear in finding evidence as required by BC-2.19.019 postcondition 3.
