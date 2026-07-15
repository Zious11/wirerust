# AC-169-001 — `parse_asdu` Minimum-Length Guard: Rejects ASDU Body Shorter Than 6 Bytes

**Story:** STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields  
**AC:** AC-169-001  
**Traces to:** BC-2.19.015 postconditions 1–3 and invariant 2  
**Wave:** 78

---

## Acceptance Criterion

- Given an I-format APCI frame whose extracted ASDU body has fewer than 6 bytes
  (TypeID(1) + VSQ(1) + COT(2) + CASDU(2) = 6-byte DUI minimum)
- When `parse_asdu(asdu_body)` is called
- Then it returns `None`; no TypeID, VSQ, COT, CASDU, or IOA fields are accessed; no panic occurs
- Given exactly 6 bytes, it returns `Some(Asdu{...})` with `first_ioa=None`
- The determinism invariant: two calls to `parse_asdu` with identical input return identical `Option<Asdu>`

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests story_169::test_BC_2_19_015
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 5 tests
test story_169::test_BC_2_19_015_invariant_no_panic_on_all_short_lengths ... ok
test story_169::test_BC_2_19_015_returns_none_for_five_bytes_canonical_vector ... ok
test story_169::test_BC_2_19_015_returns_none_for_empty_body ... ok
test story_169::test_BC_2_19_015_invariant_parse_asdu_pure_deterministic ... ok
test story_169::test_BC_2_19_015_returns_some_for_exactly_six_bytes_minimum_valid ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s
```

Result: **5/5 PASS**

---

## Test Coverage

| Test Name | Input | Expected | Condition Exercised |
|-----------|-------|----------|---------------------|
| `test_BC_2_19_015_returns_none_for_empty_body` | `[]` (0 bytes) | `None` | Empty body — far below 6-byte minimum |
| `test_BC_2_19_015_returns_none_for_five_bytes_canonical_vector` | `[0x2D,0x01,0x06,0x00,0x01,0x00]` truncated to 5 bytes | `None` | One byte short of minimum (EC-001) |
| `test_BC_2_19_015_returns_some_for_exactly_six_bytes_minimum_valid` | exactly 6 bytes | `Some(Asdu{first_ioa:None,...})` | Boundary: minimum valid DUI (EC-002) |
| `test_BC_2_19_015_invariant_no_panic_on_all_short_lengths` | lengths 0–5 exhaustive | `None` for all | No panic on any under-minimum length |
| `test_BC_2_19_015_invariant_parse_asdu_pure_deterministic` | same 6-byte body twice | two equal `Option<Asdu>` | Determinism invariant (BC-2.19.015 invariant 2) |

---

## Success-Path Demonstration

### Rejection: body shorter than 6 bytes (BC-2.19.015 PC1)

```
Input:         asdu_body = &[0x2D, 0x01, 0x06, 0x00, 0x01]  // 5 bytes
Call:          parse_asdu(asdu_body)

Result:        None
Side effects:  none (pure function; no state mutation)
```

### Boundary: exactly 6 bytes (BC-2.19.015 PC2–PC3)

```
Input:         asdu_body = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00]  // 6 bytes
                            TypeID=0x2D(45) VSQ=0x01 COT_byte2=0x06 COT_byte3=0x00 CASDU_lo=0x01 CASDU_hi=0x00
Call:          parse_asdu(asdu_body)

Result:        Some(Asdu { type_id: 45, sq: false, count: 1, cot_cause: 6,
                            cot_pn: false, cot_test: false, cot_originator: 0,
                            casdu: 1, first_ioa: None })
               first_ioa=None because count=1 but len=6 < 9 (insufficient bytes for 3-byte IOA)
```

---

## Verdict

AC-169-001: **PASS** — 5/5 BC-2.19.015 tests green; guard threshold `< 6` confirmed; boundary 5→None, 6→Some both verified; no-panic invariant over all lengths 0–5; determinism invariant confirmed.
