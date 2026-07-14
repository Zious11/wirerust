# AC-169-003 — COT Broken-Out: `cot_cause`, `cot_pn`, `cot_test`, `cot_originator` from ASDU Bytes 2–3

**Story:** STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields  
**AC:** AC-169-003  
**Traces to:** BC-2.19.017 postconditions 1–4  
**Wave:** 78

---

## Acceptance Criterion

- Given `asdu_body.len() >= 6`
- When `parse_asdu(asdu_body)` returns `Some(asdu)`
- Then:
  - `asdu.cot_cause == asdu_body[2] & 0x3F` (6-bit cause code, 0–63)
  - `asdu.cot_pn == (asdu_body[2] & 0x40) != 0` (P/N flag: positive/negative confirmation)
  - `asdu.cot_test == (asdu_body[2] & 0x80) != 0` (T flag: test transmission)
  - `asdu.cot_originator == asdu_body[3]` (u8; 0 = no originator)

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests story_169::test_BC_2_19_017
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 6 tests
test story_169::test_BC_2_19_017_cot_all_bits_byte2_0xC6_byte3_0x01_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_cause_6_activation_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_cause_max_63_byte2_0x3F_byte3_0xFF_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_originator_verbatim_from_byte_3 ... ok
test story_169::test_BC_2_19_017_cot_pn_true_byte2_0x46_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_test_true_byte2_0x86_canonical_vector ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 85 filtered out; finished in 0.00s
```

Result: **6/6 PASS**

---

## COT Byte 2 Decomposition

```
COT byte (asdu_body[2]):
  bit 7 (0x80):   T flag  → asdu.cot_test  = (byte2 & 0x80) != 0
  bit 6 (0x40):   P/N flag → asdu.cot_pn   = (byte2 & 0x40) != 0
  bits 5:0 (0x3F): cause  → asdu.cot_cause = byte2 & 0x3F

COT byte3 (asdu_body[3]):
  all 8 bits verbatim → asdu.cot_originator
```

## Canonical Vector Table: ASDU bytes 2–3 → COT fields

| byte[2] | byte[3] | `cot_cause` | `cot_pn` | `cot_test` | `cot_originator` | Notes |
|---------|---------|-------------|----------|------------|------------------|-------|
| `0x06` | `0x00` | 6 | false | false | 0 | Cause=6 Activation; no originator; BC-2.19.017 primary canonical vector |
| `0x3F` | `0xFF` | 63 | false | false | 255 | cause max (63); originator max (255) |
| `0x46` | `0x00` | 6 | true | false | 0 | bit6=1 → cot_pn=true; P/N flag set |
| `0x86` | `0x00` | 6 | false | true | 0 | bit7=1 → cot_test=true; test-bit canonical vector (EC-008) |
| `0xC6` | `0x01` | 6 | true | true | 1 | both T and P/N set; originator=1 |

### Cause Codes (IEC-60870-5-104, 6-bit)

| Cause Value | Meaning | Security Relevance |
|-------------|---------|-------------------|
| 6 | Activation | Control command activation; primary detection surface for STORY-170 |
| 7 | Activation confirmation | RTU acknowledges |
| 8 | Deactivation | |
| 10 | Deactivation confirmation | |
| 44–63 | Cause 44–63 range | Some reserved; useful for anomaly detection |
| 63 (0x3F) | Max 6-bit cause | Boundary; extracted verbatim |

---

## Test Coverage

| Test Name | byte[2] | byte[3] | cot_cause | cot_pn | cot_test | cot_orig | Condition |
|-----------|---------|---------|-----------|--------|----------|----------|-----------|
| `test_BC_2_19_017_cot_cause_6_activation_canonical_vector` | `0x06` | `0x00` | 6 | false | false | 0 | Primary canonical vector (BC-2.19.017 PC1-PC4) |
| `test_BC_2_19_017_cot_cause_max_63_byte2_0x3F_byte3_0xFF_canonical_vector` | `0x3F` | `0xFF` | 63 | false | false | 255 | cause=63 max; originator=255 max |
| `test_BC_2_19_017_cot_pn_true_byte2_0x46_canonical_vector` | `0x46` | `0x00` | 6 | true | false | 0 | P/N flag (BC-2.19.017 PC2) |
| `test_BC_2_19_017_cot_test_true_byte2_0x86_canonical_vector` | `0x86` | `0x00` | 6 | false | true | 0 | T-flag (BC-2.19.017 PC3; EC-008) |
| `test_BC_2_19_017_cot_originator_verbatim_from_byte_3` | `0x06` | `0xAB` | 6 | false | false | 0xAB | originator verbatim byte3 (BC-2.19.017 PC4) |
| `test_BC_2_19_017_cot_all_bits_byte2_0xC6_byte3_0x01_canonical_vector` | `0xC6` | `0x01` | 6 | true | true | 1 | T and P/N simultaneously set |

---

## Error-Path Demonstration

The COT layer has no rejection path in `parse_asdu`: any `asdu_body.len() >= 6` produces
a `Some(Asdu)` with all COT fields extracted. Anomaly handling (e.g., suppressing findings
on `cot_test=true` per BC-2.19.017 invariant 1) is the caller's responsibility (STORY-170).

```
Input:  asdu_body[2] = 0x86 (T-bit set: bit7=1, cause=6, pn=false)
        asdu_body[3] = 0x00

Result: Some(Asdu { cot_cause: 6, cot_pn: false, cot_test: true, cot_originator: 0, ... })
        The test-bit is extracted; parse_asdu does NOT suppress it.
        Caller (STORY-170) decides whether to tag findings "[TEST]".
```

---

## Verdict

AC-169-003: **PASS** — 6/6 BC-2.19.017 tests green; `cot_cause` (6-bit mask 0x3F), `cot_pn` (bit6), `cot_test` (bit7), and `cot_originator` (verbatim byte3) all extracted correctly; cause=6 activation primary canonical vector confirmed; T-bit (cot_test=true) extraction verified.
