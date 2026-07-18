# AC-170-004/005/006 — Reserved/Invalid TypeIDs (T0814) + Defined-But-Unhandled (Silent) + Dispatch Table

**Story:** STORY-170: IEC-104 Control Command Detection  
**ACs:** AC-170-004, AC-170-005, AC-170-006  
**Traces to:** BC-2.19.022 postconditions 1–2; invariant 1  
**Wave:** 79

---

## Acceptance Criteria

**AC-170-004:** TypeID=0 or TypeID in [128, 255] emits T0814 Possible  
- TypeID=0 is undefined by IEC 60870-5-104
- TypeIDs 128–255 are the private-use/reserved range

**AC-170-005:** Defined-but-unhandled TypeIDs in [1, 127] produce no finding (silently logged)  
- Ranges: 1–44 (monitoring direction), 52–99 (above control range), 102, 104, 106–127 (future-defined)
- These are valid IEC-104 TypeIDs that are not in any detection set

**AC-170-006:** TypeID dispatch is exhaustive — every TypeID (0–255) produces exactly one outcome  
- The dispatch table covers the complete 256-value TypeID space with no fallthrough

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_022
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 12 tests
test story_170::test_BC_2_19_022_invariant_silent_range_sample_emits_no_findings ... ok
test story_170::test_BC_2_19_022_type0_undefined_emits_t0814_anomaly_canonical_vector ... ok
test story_170::test_BC_2_19_022_type102_c_rd_not_in_detection_set_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type104_defined_unhandled_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type127_max_defined_unhandled_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type128_emits_t0814_possible_canonical_vector ... ok
test story_170::test_BC_2_19_022_type1_minimum_defined_unhandled_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type200_private_use_emits_t0814_possible ... ok
test story_170::test_BC_2_19_022_type255_emits_t0814_possible_canonical_vector ... ok
test story_170::test_BC_2_19_022_type44_max_monitoring_emits_no_finding_canonical_vector ... ok
test story_170::test_BC_2_19_022_type52_reserved_above_control_range_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type99_defined_unhandled_emits_no_finding ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 124 filtered out; finished in 0.00s
```

Result: **12/12 PASS**

---

## Test Coverage

### AC-170-004: Reserved/Undefined TypeIDs → T0814 Possible

| Test Name | TypeID | Condition | Assertion | Result |
|-----------|--------|-----------|-----------|--------|
| `test_BC_2_19_022_type0_undefined_emits_t0814_anomaly_canonical_vector` | 0 | undefined (BC canonical vector EC-001) | T0814 Possible emitted | PASS |
| `test_BC_2_19_022_type128_emits_t0814_possible_canonical_vector` | 128 | first private-use (BC canonical vector) | T0814 Possible emitted | PASS |
| `test_BC_2_19_022_type200_private_use_emits_t0814_possible` | 200 | mid private-use range | T0814 Possible emitted | PASS |
| `test_BC_2_19_022_type255_emits_t0814_possible_canonical_vector` | 255 | max TypeID (BC canonical vector EC-008) | T0814 Possible emitted | PASS |

### AC-170-005: Defined-But-Unhandled TypeIDs → No Finding

| Test Name | TypeID | Range / Note | findings after call | Result |
|-----------|--------|-------------|---------------------|--------|
| `test_BC_2_19_022_type1_minimum_defined_unhandled_emits_no_finding` | 1 | min monitoring direction | empty | PASS |
| `test_BC_2_19_022_type44_max_monitoring_emits_no_finding_canonical_vector` | 44 | max monitoring (BC canonical EC-002) | empty | PASS |
| `test_BC_2_19_022_type52_reserved_above_control_range_emits_no_finding` | 52 | above control range (EC-005) | empty | PASS |
| `test_BC_2_19_022_type99_defined_unhandled_emits_no_finding` | 99 | above control range, defined | empty | PASS |
| `test_BC_2_19_022_type102_c_rd_not_in_detection_set_emits_no_finding` | 102 | C_RD_NA_1 read command (EC-007) | empty | PASS |
| `test_BC_2_19_022_type104_defined_unhandled_emits_no_finding` | 104 | between C_RP and C_IC | empty | PASS |
| `test_BC_2_19_022_type127_max_defined_unhandled_emits_no_finding` | 127 | max defined-but-unhandled | empty | PASS |
| `test_BC_2_19_022_invariant_silent_range_sample_emits_no_findings` | 1,30,44,52,99,102,104,106,127 | cross-section of silent range | empty for each | PASS |

---

## AC-170-006: Complete Dispatch Table

Every TypeID (0–255) maps to exactly one outcome. No fallthrough exists.

| TypeID Range | TypeID Name / Class | Outcome | BC Reference |
|-------------|---------------------|---------|-------------|
| 0 | Undefined (no IEC 60870-5-104 assignment) | T0814 Possible | BC-2.19.022 PC2 |
| 1–44 | Monitoring direction (M_ types: M_SP, M_DP, M_ME, M_IT, M_EP, …) | Silent — no finding | BC-2.19.022 inv1 |
| 45–47 | Switching commands (C_SC, C_DC, C_RC) | T1692.001 Possible | BC-2.19.019 inv2 |
| 48–51 | Set-point / bitstring (C_SE_NA/NB/NC, C_BO_NA) | T1692.001 + T0836 Possible | BC-2.19.019 PC2 |
| 52–99 | Defined-but-unhandled above control range | Silent — no finding | BC-2.19.022 inv1 |
| 100, 101, 103 | Interrogation / clock-sync (C_IC, C_CI, C_CS) | Silent — no finding | BC-2.19.021 PC1 |
| 102, 104, 106–127 | Other defined (C_RD, gap, future-defined) | Silent — no finding | BC-2.19.022 inv1 |
| 105 | C_RP_NA_1 (Reset Process) | T0827 Likely | BC-2.19.020 PC1 |
| 128–255 | Private-use / reserved range | T0814 Possible | BC-2.19.022 PC2 |

The dispatch is implemented as an exhaustive Rust `match` with a wildcard arm that
silently logs unhandled TypeIDs in [1, 127]. This ensures future IEC-104 TypeID
additions are absorbed without emitting false findings.

---

## Verdict

AC-170-004/005/006: **PASS** — All 12 BC-2.19.022 tests green.
TypeID=0 and 128–255 correctly emit T0814 Possible.
Defined-but-unhandled TypeIDs in [1, 127] (excluding detection sets) produce no finding.
Dispatch table is exhaustive across all 256 TypeID values.
