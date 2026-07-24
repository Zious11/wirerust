# Evidence Report — STORY-180

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64 (BC-2.19.029 + BC-2.19.030 + BC-2.19.022 v1.1 Regression Guard)  
**Wave:** 85  
**Date:** 2026-07-24  
**Branch:** feature/STORY-180-iec104-timed-cmd-detection  
**Product type:** Library (effectful detection function — no CLI/web surface; dispatch wiring in iec104 analyzer)

---

## Full Test Suite: 248/248 PASS

Command:
```
cargo test --test iec104_analyzer_tests
```

Output (tail):
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

test result: ok. 248 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

**STORY-180 contribution:** 27 tests (story_180 module)  
**Predecessor contributions:** 221 tests (story_167..story_174 modules)  
**Total:** 248/248 PASS

---

## STORY-180 Test Run (27 tests)

Command:
```
cargo test --test iec104_analyzer_tests story_180
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-09617f6be29af6e9)

running 27 tests
test story_180::test_BC_2_19_022_v1_1_type_id_57_no_finding ... ok
test story_180::test_BC_2_19_022_v1_1_type_id_65_no_finding ... ok
test story_180::test_BC_2_19_022_v1_1_type_id_52_no_finding ... ok
test story_180::test_BC_2_19_019_v1_1_regression_type_id_45_still_one_finding ... ok
test story_180::test_BC_2_19_022_v1_1_type_id_99_no_finding ... ok
test story_180::test_BC_2_19_029_casdu_first_ioa_evidence ... ok
test story_180::test_BC_2_19_019_v1_1_regression_type_id_51_still_two_findings ... ok
test story_180::test_BC_2_19_029_timed_summary_differs_from_untimed_twin ... ok
test story_180::test_BC_2_19_029_timed_summary_contains_time_tagged_qualifier ... ok
test story_180::test_BC_2_19_029_type_id_58_emits_t1692_001_only ... ok
test story_180::test_BC_2_19_029_type_id_58_count_zero_still_emits ... ok
test story_180::test_BC_2_19_029_type_id_58_verdict_confidence_category ... ok
test story_180::test_BC_2_19_029_type_id_59_emits_t1692_001_only ... ok
test story_180::test_BC_2_19_029_type_id_60_cot_test_suffix ... ok
test story_180::test_BC_2_19_029_type_id_59_first_ioa_none_no_first_ioa_evidence ... ok
test story_180::test_BC_2_19_029_type_id_60_emits_t1692_001_only ... ok
test story_180::test_BC_2_19_030_timed_summaries_contain_time_tagged_and_mnemonics ... ok
test story_180::test_BC_2_19_030_timed_summaries_differ_from_untimed_twin ... ok
test story_180::test_BC_2_19_030_type_id_61_casdu_first_ioa_evidence_both_findings ... ok
test story_180::test_BC_2_19_030_type_id_61_count_zero_still_emits_two_findings ... ok
test story_180::test_BC_2_19_030_type_id_61_emits_two_findings ... ok
test story_180::test_BC_2_19_030_type_id_61_verdict_confidence_category_both_findings ... ok
test story_180::test_BC_2_19_030_type_id_62_emits_two_findings ... ok
test story_180::test_BC_2_19_030_type_id_62_first_ioa_none_no_first_ioa_evidence ... ok
test story_180::test_BC_2_19_030_type_id_63_emits_two_findings ... ok
test story_180::test_BC_2_19_030_type_id_64_cot_test_both_findings_tagged ... ok
test story_180::test_BC_2_19_030_type_id_64_emits_two_findings ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 221 filtered out; finished in 0.00s
```

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-180-001 | TypeIDs 58–60 emit exactly one T1692.001 Possible finding with CASDU and first_ioa evidence | BC-2.19.029 PC1 + PC3 | 6 | `AC-001-002-typeid-58-60-timed-switching.md` | PASS |
| AC-180-002 | TypeIDs 58–60 do NOT emit T0836 (switching commands, not parameter writes) | BC-2.19.029 invariant 2 | included in AC-001 tests (single-finding assertion) | `AC-001-002-typeid-58-60-timed-switching.md` | PASS |
| AC-180-003 | TypeIDs 61–64 emit exactly one T1692.001 Possible AND one T0836 Possible finding | BC-2.19.030 PC1-PC3 | 7 | `AC-003-typeid-61-64-timed-setpoint.md` | PASS |
| AC-180-004 | Timed-variant summary wording distinguishes from untimed twin summaries | BC-2.19.029 PC4; BC-2.19.030 PC4-PC5 | 4 | `AC-004-timed-summary-wording.md` | PASS |
| AC-180-005 | cot_test=true appends [TEST] suffix to all emitted timed-command findings | BC-2.19.017 inv1; BC-2.19.029 PC6; BC-2.19.030 PC7 | 2 | `AC-005-cot-test-tagging.md` | PASS |
| AC-180-006 | TypeIDs 52–57 and 65–99 still produce zero findings (BC-2.19.022 v1.1 regression guard) | BC-2.19.022 v1.1 inv1 | 4 + 2 regression | `AC-006-silence-regression-guard.md` | PASS |
| AC-180-007 | Silent-range code comment narrowed to {52–57, 65–99} with BC-2.19.029/030 note | BC-2.19.022 v1.1 arch anchor | source-level verification (grep output) | `AC-007-silent-range-comment.md` | PASS |
| AC-180-008 | Emission is count-independent — one finding set per ASDU regardless of VSQ count | BC-2.19.029 inv3; BC-2.19.030 inv3 | 2 | `AC-008-count-independent-emission.md` | PASS |

**Total STORY-180 test-based coverage: 27/27 (all AC-180-001..008)**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Key Test Names |
|----|-----|-----------|----------------|
| AC-180-001/002 | BC-2.19.029 PC1-PC3; inv2 | 6 | type_id_58/59/60_emits_t1692_001_only; verdict_confidence_category; casdu_first_ioa_evidence; first_ioa_none_no_first_ioa_evidence |
| AC-180-003 | BC-2.19.030 PC1-PC3 | 7 | type_id_61/62/63/64_emits_two_findings; verdict_confidence_category_both; casdu_first_ioa_evidence_both; first_ioa_none_no_first_ioa_evidence |
| AC-180-004 | BC-2.19.029 PC4; BC-2.19.030 PC4-PC5 | 4 | timed_summary_contains_time_tagged_qualifier; timed_summary_differs_from_untimed_twin (both arms) |
| AC-180-005 | BC-2.19.017 inv1 | 2 | type_id_60_cot_test_suffix; type_id_64_cot_test_both_findings_tagged |
| AC-180-006 | BC-2.19.022 v1.1 inv1 | 6 | type_id_52/57/65/99_no_finding; regression_type_id_45_still_one_finding; regression_type_id_51_still_two_findings |
| AC-180-007 | BC-2.19.022 v1.1 arch anchor | — (source-level) | grep confirms {52–57, 65–99} in catch-all comment and "58–64 were here prior" note |
| AC-180-008 | BC-2.19.029 inv3; BC-2.19.030 inv3 | 2 | type_id_58_count_zero_still_emits; type_id_61_count_zero_still_emits_two_findings |

---

## Updated Dispatch Table (after STORY-180)

| TypeID Range | Finding(s) Emitted | MITRE Techniques | Verdict | BC |
|-------------|-------------------|-----------------|---------|-----|
| 0 | T0814 "Denial of Service" | T0814 | Possible | BC-2.19.022 |
| 1–44 | None (monitoring direction) | — | — | BC-2.19.022 |
| 45–47 | T1692.001 "Command Message" | T1692.001 | Possible | BC-2.19.019 |
| 48–51 | T1692.001 "Command Message" + T0836 "Modify Parameter" | T1692.001, T0836 | Possible | BC-2.19.019 |
| 52–57 | None (reserved — silently logged) | — | — | BC-2.19.022 v1.1 |
| **58–60** | **T1692.001 "Command Message"** | **T1692.001** | **Possible** | **BC-2.19.029 (NEW)** |
| **61–64** | **T1692.001 "Command Message" + T0836 "Modify Parameter"** | **T1692.001, T0836** | **Possible** | **BC-2.19.030 (NEW)** |
| 65–99 | None (unhandled — silently logged) | — | — | BC-2.19.022 v1.1 |
| 100, 101, 103 | None (interrogation/clock-sync benign) | — | — | BC-2.19.021 |
| 102, 104, 106–127 | None (defined-but-unhandled) | — | — | BC-2.19.022 |
| 105 | T0827 "Loss of Control" | T0827 | **Likely** | BC-2.19.020 |
| 128–255 | T0814 "Denial of Service" | T0814 | Possible | BC-2.19.022 |

---

## Edge Case Coverage Summary

| Edge Case | BC | Test Covering | Verdict |
|-----------|-----|--------------|---------|
| EC-001: TypeID=58 (C_SC_TA_1) | BC-2.19.029 | `type_id_58_emits_t1692_001_only` | PASS |
| EC-002: TypeID=59 (C_DC_TA_1) | BC-2.19.029 | `type_id_59_emits_t1692_001_only` | PASS |
| EC-003: TypeID=60 (C_RC_TA_1) | BC-2.19.029 | `type_id_60_emits_t1692_001_only` | PASS |
| EC-004: TypeID=61 (C_SE_TA_1) | BC-2.19.030 | `type_id_61_emits_two_findings` | PASS |
| EC-005: TypeID=64 (C_BO_TA_1) | BC-2.19.030 | `type_id_64_emits_two_findings` | PASS |
| EC-006: TypeID=57 (RESERVED, upper neighbor below 58) | BC-2.19.022 v1.1 | `type_id_57_no_finding` | PASS |
| EC-007: TypeID=65 (lower neighbor above 64) | BC-2.19.022 v1.1 | `type_id_65_no_finding` | PASS |
| EC-008: TypeID=58, first_ioa=None | BC-2.19.029 | `type_id_59_first_ioa_none_no_first_ioa_evidence` | PASS |
| EC-009: TypeID=60, cot_test=true | BC-2.19.029 | `type_id_60_cot_test_suffix` | PASS |
| EC-010: TypeID=64, cot_test=true (both findings) | BC-2.19.030 | `type_id_64_cot_test_both_findings_tagged` | PASS |
| EC-011: TypeID=58, asdu.count=0 | BC-2.19.029 | `type_id_58_count_zero_still_emits` | PASS |
| EC-012: TypeID=45 (untimed twin regression) | BC-2.19.019 | `regression_type_id_45_still_one_finding` | PASS |
| EC-013: TypeID=51 (untimed twin regression) | BC-2.19.019 | `regression_type_id_51_still_two_findings` | PASS |

---

## Recording Method

This is an effectful library story (no CLI binary, no web UI). Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Inline dispatch table and edge-case coverage tables
- Source-level verification via grep for AC-180-007 (comment-only AC)

VHS/Playwright recordings are not applicable at this story scope. The timed-command
detection function (`detect_iec104_threats`) is an internal effectful function with no
interactive CLI surface in STORY-180.

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-002-typeid-58-60-timed-switching.md` | AC-180-001 (BC-2.19.029 PC1+PC3), AC-180-002 (BC-2.19.029 inv2) |
| `AC-003-typeid-61-64-timed-setpoint.md` | AC-180-003 (BC-2.19.030 PC1-PC3) |
| `AC-004-timed-summary-wording.md` | AC-180-004 (BC-2.19.029 PC4; BC-2.19.030 PC4-PC5) |
| `AC-005-cot-test-tagging.md` | AC-180-005 (BC-2.19.017 inv1; BC-2.19.029 PC6; BC-2.19.030 PC7) |
| `AC-006-silence-regression-guard.md` | AC-180-006 (BC-2.19.022 v1.1 inv1) |
| `AC-007-silent-range-comment.md` | AC-180-007 (BC-2.19.022 v1.1 arch anchor; source-level) |
| `AC-008-count-independent-emission.md` | AC-180-008 (BC-2.19.029 inv3; BC-2.19.030 inv3) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

Command run before commit (PG-W70-DEMO-SCRUB canonical pattern):
```
grep -rE '<host-path-pattern>' docs/demo-evidence/STORY-180/
```

Result: **zero matches** — no absolute host paths present in any evidence file.

Result: **zero matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-24).
