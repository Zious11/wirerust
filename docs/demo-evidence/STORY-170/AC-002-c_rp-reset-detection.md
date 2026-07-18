# AC-170-002 — C_RP TypeID 105 Emits T0827 Likely (Loss of Control)

**Story:** STORY-170: IEC-104 Control Command Detection  
**AC:** AC-170-002  
**Traces to:** BC-2.19.020 postconditions 1–2; invariant 1  
**Wave:** 79

---

## Acceptance Criterion

- Given an I-format frame with TypeID=105 (C_RP_NA_1 — Reset Process Command)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then a T0827 "Loss of Control" finding is emitted with confidence **Likely** (not Possible)
- And no T1692.001 finding is emitted (BC-2.19.020 invariant 1)
- C_RP resets RTU/IED processes; adversarial use causes equipment malfunction or loss of control

Critical: verdict is `Verdict::Likely`, NOT `Verdict::Possible`. The v1.1 BC correction
changed this from Possible to Likely to reflect the high operational impact of a process reset.

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_020
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 5 tests
test story_170::test_BC_2_19_020_type105_c_rp_na1_emits_exactly_one_finding ... ok
test story_170::test_BC_2_19_020_type105_category_is_impact ... ok
test story_170::test_BC_2_19_020_type105_does_not_emit_t1692001 ... ok
test story_170::test_BC_2_19_020_type105_emits_t0827_likely_canonical_vector ... ok
test story_170::test_BC_2_19_020_type105_verdict_is_likely_not_possible ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 131 filtered out; finished in 0.00s
```

Result: **5/5 PASS**

---

## Test Coverage

| Test Name | TypeID / Condition | Assertion | Result |
|-----------|-------------------|-----------|--------|
| `test_BC_2_19_020_type105_c_rp_na1_emits_exactly_one_finding` | TypeID=105 | exactly 1 finding (T0827 only) | PASS |
| `test_BC_2_19_020_type105_emits_t0827_likely_canonical_vector` | TypeID=105 (canonical vector) | mitre=T0827, verdict=Likely | PASS |
| `test_BC_2_19_020_type105_verdict_is_likely_not_possible` | TypeID=105 | verdict is Likely, NOT Possible (v1.1 guard) | PASS |
| `test_BC_2_19_020_type105_does_not_emit_t1692001` | TypeID=105 | NO T1692.001 emitted (negative path) | PASS |
| `test_BC_2_19_020_type105_category_is_impact` | TypeID=105 | category=ThreatCategory::Impact | PASS |

---

## Finding Properties Verified

| Property | Expected | Verified By |
|----------|----------|-------------|
| `mitre_techniques` | contains "T0827" | `type105_emits_t0827_likely_canonical_vector` |
| `verdict` | `Verdict::Likely` | `type105_emits_t0827_likely_canonical_vector`, `type105_verdict_is_likely_not_possible` |
| `verdict` (negative) | NOT `Verdict::Possible` | `type105_verdict_is_likely_not_possible` |
| `category` | `ThreatCategory::Impact` | `type105_category_is_impact` |
| count | exactly 1 finding | `type105_c_rp_na1_emits_exactly_one_finding` |
| no T1692.001 (negative) | T1692.001 absent | `type105_does_not_emit_t1692001` |

---

## Verdict Confidence Significance

`Verdict::Likely` (not Possible) is critical here. The distinction reflects operational risk:

- C_RP resets the running process on an RTU/IED — this is not a data read or monitoring action
- Adversarial use results in loss of control over field devices
- BC-2.19.020 v1.1 explicitly elevated the confidence from Possible to Likely (v1.1 correction)
- The `type105_verdict_is_likely_not_possible` test guards against regression to the old v1.0 behavior

---

## Verdict

AC-170-002: **PASS** — All 5 BC-2.19.020 tests green. TypeID=105 (C_RP_NA_1) emits T0827
with `Verdict::Likely`, category `Impact`, exactly 1 finding, and no T1692.001 co-emission.
