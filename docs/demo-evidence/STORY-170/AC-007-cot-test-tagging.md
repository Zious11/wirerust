# AC-170-007 — Control-Command Findings Tagged [TEST] When cot_test Is True

**Story:** STORY-170: IEC-104 Control Command Detection  
**AC:** AC-170-007  
**Traces to:** BC-2.19.017 invariant 1  
**Wave:** 79

---

## Acceptance Criterion

- Given an I-format frame whose ASDU would otherwise produce a finding (TypeID in a detection set)
- And the parsed `Asdu` has `cot_test == true` (bit 7 of COT byte 2 is set, per BC-2.19.017 PC3)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then the emitted finding's `summary` field is appended with ` [TEST]` to tag it as a test-frame finding
- And when `cot_test == false`, NO `[TEST]` tag appears in any finding's summary
- No new `Finding` struct field is required — the tag is applied to the existing `Finding::summary` string

This reduces analyst noise: test transmissions are still recorded but visually distinguished
from operational findings.

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests "story_170::test_BC_2_19_017"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 7 tests
test story_170::test_BC_2_19_017_cot_test_false_control_command_summary_has_no_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_control_command_summary_has_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_setpoint_both_findings_have_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_t0814_reserved_type_summary_has_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_t0827_summary_has_test_tag ... ok
test story_170::test_BC_2_19_017_invariant_cot_test_false_never_adds_test_tag ... ok
test story_170::test_BC_2_19_017_start_idx_guard_preexisting_finding_not_tagged ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 129 filtered out; finished in 0.00s
```

Result: **7/7 PASS**

---

## Test Coverage

### Success Path: cot_test=true → summary contains " [TEST]"

| Test Name | TypeID / cot_test | Finding Type | Assertion | Result |
|-----------|-------------------|--------------|-----------|--------|
| `test_BC_2_19_017_cot_test_true_control_command_summary_has_test_tag` | TypeID=45, cot_test=true | T1692.001 | summary contains " [TEST]" (EC-009) | PASS |
| `test_BC_2_19_017_cot_test_true_t0827_summary_has_test_tag` | TypeID=105, cot_test=true | T0827 Likely | summary contains " [TEST]" | PASS |
| `test_BC_2_19_017_cot_test_true_setpoint_both_findings_have_test_tag` | TypeID=48, cot_test=true | T1692.001 + T0836 | BOTH summaries contain " [TEST]" | PASS |
| `test_BC_2_19_017_cot_test_true_t0814_reserved_type_summary_has_test_tag` | TypeID=128, cot_test=true | T0814 | summary contains " [TEST]" | PASS |

### Error Path: cot_test=false → summary does NOT contain " [TEST]"

| Test Name | TypeID / cot_test | Assertion | Result |
|-----------|-------------------|-----------|--------|
| `test_BC_2_19_017_cot_test_false_control_command_summary_has_no_test_tag` | TypeID=45, cot_test=false | NO " [TEST]" in summary | PASS |
| `test_BC_2_19_017_invariant_cot_test_false_never_adds_test_tag` | TypeIDs 45/48/105/128, cot_test=false | NO " [TEST]" for any | PASS |

### Invariant: start_idx guard — pre-existing findings are not re-tagged

| Test Name | Setup | Assertion | Result |
|-----------|-------|-----------|--------|
| `test_BC_2_19_017_start_idx_guard_preexisting_finding_not_tagged` | Pre-populated findings Vec + TypeID=45 cot_test=true | Only NEW findings tagged; pre-existing finding unchanged | PASS |

---

## [TEST] Tagging Scope Verified

The cot_test tag applies universally across all finding types emitted by `detect_iec104_threats`:

| Finding Type | TypeID Example | cot_test=true Tags? | Verified by Test |
|-------------|---------------|---------------------|-----------------|
| T1692.001 (switching command) | 45 | Yes | `cot_test_true_control_command_summary_has_test_tag` |
| T0827 (loss of control / reset) | 105 | Yes | `cot_test_true_t0827_summary_has_test_tag` |
| T1692.001 + T0836 (both co-emitted) | 48 | Yes — both | `cot_test_true_setpoint_both_findings_have_test_tag` |
| T0814 (reserved TypeID anomaly) | 128 | Yes | `cot_test_true_t0814_reserved_type_summary_has_test_tag` |

---

## Verdict

AC-170-007: **PASS** — All 7 BC-2.19.017 tests green. When `cot_test=true`, all
emitted findings have ` [TEST]` appended to `Finding::summary`. When `cot_test=false`,
no finding carries the tag. Pre-existing findings in the buffer are not retroactively
tagged (start_idx guard). Tag applies uniformly to T1692.001, T0836, T0827, and T0814.
