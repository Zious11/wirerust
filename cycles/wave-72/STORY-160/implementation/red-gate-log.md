---
document_type: red-gate-log
level: ops
version: "1.0"
status: complete
producer: test-writer
timestamp: 2026-07-09T00:00:00Z
phase: 3
inputs: []
input-hash: "d41d8cd"
traces_to: ".factory/stories/STORY-160.md"
stub_architect_agent: "N/A"
stub_compile_verified: false
test_writer_agent: "claude-sonnet-4-6 (test-writer role)"
red_gate_verified: true
---

# Red Gate Log: wave-72 / STORY-160

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|-------------|-----------------|------|
| STORY-160 | 14 new + 1 updated | 11 fail (assertion-type); 4 pass (regression guards, by design) | PASSED |

## Stubs Created

### STORY-160: Align JSON Finding-Enum Serialization to Lowercase/snake_case + schema_version Envelope

No new stubs were generated in this test-writer pass. The stub-architect step (Task #11,
completed before this pass) created the compilation skeleton. This test-writer pass adds
only test code; no new production-code stubs are introduced.

## Test File Modified

`tests/reporter_json_tests.rs` (flat file root, per STORY-160 Task 3 / F-W72-P9-L02 ruling:
DF-TEST-NAMESPACE-001 does NOT apply to reporter test files)

**Total change:** 370 insertions, 6 deletions.

## Tests Added (14 new)

### BC-2.11.036: JSON enum-value casing (9 tests)

| AC | Test Name | Status |
|----|-----------|--------|
| AC-160-001 | `test_BC_2_11_036_verdict_likely_serializes_lowercase` | RED (assertion: "Likely" != "likely") |
| AC-160-001 | `test_BC_2_11_036_verdict_all_variants_lowercase` | RED (assertion: "Likely" != "likely" on first variant) |
| AC-160-002 | `test_BC_2_11_036_confidence_high_serializes_lowercase` | RED (assertion: "High" != "high") |
| AC-160-002 | `test_BC_2_11_036_confidence_all_variants_lowercase` | RED (assertion: "High" != "high" on first variant) |
| AC-160-003 | `test_BC_2_11_036_threat_category_lateral_movement_snake_case` | RED (assertion: "LateralMovement" != "lateral_movement") |
| AC-160-003 | `test_BC_2_11_036_threat_category_c2_snake_case` | RED (assertion: "C2" != "c2") |
| AC-160-003 | `test_BC_2_11_036_threat_category_all_variants_snake_case` | RED (assertion on first variant) |
| AC-160-005 | `test_BC_2_11_036_terminal_display_unchanged` | GREEN (regression guard: Display already correct; expected) |
| AC-160-006 | `test_BC_2_11_036_csv_category_unchanged` | GREEN (regression guard: CSV uses Display, not Serialize; expected) |

### BC-2.11.037: schema_version envelope field (5 tests)

| AC | Test Name | Status |
|----|-----------|--------|
| AC-160-004 | `test_BC_2_11_037_schema_version_present_in_json` | RED (assertion: key absent from envelope) |
| AC-160-004 | `test_BC_2_11_037_schema_version_value_is_two` | RED (assertion: Null != String("2")) |
| AC-160-004 | `test_BC_2_11_037_schema_version_unconditional_empty_findings` | RED (assertion: key absent from envelope) |
| AC-160-006 | `test_BC_2_11_037_schema_version_absent_from_csv` | GREEN (regression guard: CSV has no envelope; expected) |
| AC-160-006 | `test_BC_2_11_037_schema_version_absent_from_terminal` | GREEN (regression guard: terminal has no envelope; expected) |

## Test Updated (DF-SIBLING-SWEEP-001)

| Test | Change | Status |
|------|--------|--------|
| `test_BC_2_11_001_top_level_keys` | vec updated from five-key to six-key form (added "schema_version" alphabetically between "mitre_domain" and "summary"); doc comment updated; schema_version contains_key assertion added | RED (assertion: 5 keys found, 6 expected) |

## Red Gate Verification

### STORY-160: reporter_json_tests.rs

Cargo test output (`cargo test --all-targets`):

```
test result: FAILED. 29 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Failing tests (11 total):**

1. `test_BC_2_11_001_top_level_keys` — FAIL: assert_eq! on key vec, 5 keys found vs 6 expected
2. `test_BC_2_11_036_confidence_all_variants_lowercase` — FAIL: String("High") != String("high")
3. `test_BC_2_11_036_confidence_high_serializes_lowercase` — FAIL: String("High") != String("high")
4. `test_BC_2_11_036_threat_category_all_variants_snake_case` — FAIL: String("Reconnaissance") != String("reconnaissance")
5. `test_BC_2_11_036_threat_category_c2_snake_case` — FAIL: String("C2") != String("c2")
6. `test_BC_2_11_036_threat_category_lateral_movement_snake_case` — FAIL: String("LateralMovement") != String("lateral_movement")
7. `test_BC_2_11_036_verdict_all_variants_lowercase` — FAIL: String("Likely") != String("likely")
8. `test_BC_2_11_036_verdict_likely_serializes_lowercase` — FAIL: String("Likely") != String("likely")
9. `test_BC_2_11_037_schema_version_present_in_json` — FAIL: key "schema_version" absent from envelope
10. `test_BC_2_11_037_schema_version_unconditional_empty_findings` — FAIL: key "schema_version" absent from envelope
11. `test_BC_2_11_037_schema_version_value_is_two` — FAIL: Null != String("2")

**All failures are assertion-type panics — no build errors, no todo!/unimplemented! macros.**

### Regression-guard tests (pass before AND after implementation, by design)

These 4 tests verify existing correct behavior; they are always green:
- `test_BC_2_11_036_terminal_display_unchanged` — tests that Display is unchanged (already correct)
- `test_BC_2_11_036_csv_category_unchanged` — tests CSV is unchanged (already correct)
- `test_BC_2_11_037_schema_version_absent_from_csv` — tests CSV has no schema_version (correct now and after)
- `test_BC_2_11_037_schema_version_absent_from_terminal` — tests terminal has no schema_version (correct now and after)

## Regression Check

| Test Suite | Status |
|-----------|--------|
| All other test files (80+ suites) | 100% green — 0 failures |
| `reporter_json_tests.rs` pre-existing tests (29 tests) | All pass |

## Commit

SHA: `9d49ff3`
Branch: `feature/STORY-160-json-enum-casing`
Message: `test(STORY-160): add failing tests for BC-2.11.036/037 + BC-2.11.001 six-key envelope`

## Hand-Off to Implementer

- **Story ready for implementation:** STORY-160
- **Implementation guidance:**
  1. In `src/findings.rs`: add `#[serde(rename_all = "lowercase")]` to the `Verdict` and `Confidence` derive groups; add `#[serde(rename_all = "snake_case")]` to the `ThreatCategory` derive group. Do NOT touch `impl fmt::Display` for any of the three enums.
  2. In `src/reporter/json.rs`: add `const SCHEMA_VERSION: &str = "2";` and wire `"schema_version": SCHEMA_VERSION` into the top-level JSON envelope.
  3. Run `cargo test --all-targets` after each step — each set of fixes should turn the corresponding tests green.
  4. After implementation, `test_BC_2_11_001_top_level_keys` will turn green only when both step 2 is complete (schema_version in envelope).
  5. The 4 regression-guard tests must remain green throughout.
