# AC-169-002 — TypeID and VSQ Broken-Out: `type_id`, `sq`, `count` from ASDU Bytes 0–1

**Story:** STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields  
**AC:** AC-169-002  
**Traces to:** BC-2.19.016 postconditions 1–4  
**Wave:** 78

---

## Acceptance Criterion

- Given `asdu_body.len() >= 6`
- When `parse_asdu(asdu_body)` returns `Some(asdu)`
- Then:
  - `asdu.type_id == asdu_body[0]` (u8, verbatim; TypeID 0 is undefined, passed through)
  - `asdu.sq == (asdu_body[1] & 0x80) != 0` (SQ bit: true = contiguous sequence)
  - `asdu.count == asdu_body[1] & 0x7F` (number of information objects, 0–127)
  - TypeID 0 (undefined per IEC-60870-5-104) is extracted without rejection

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests story_169::test_BC_2_19_016
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 7 tests
test story_169::test_BC_2_19_016_type_id_45_c_sc_na_1_canonical_vector ... ok
test story_169::test_BC_2_19_016_type_id_0_undefined_passthrough_canonical_vector ... ok
test story_169::test_BC_2_19_016_type_id_255_vsq_0x80_sq_true_count_0_canonical_vector ... ok
test story_169::test_BC_2_19_016_type_id_extracted_verbatim_from_byte_0 ... ok
test story_169::test_BC_2_19_016_vsq_0x03_sq_false_count_3 ... ok
test story_169::test_BC_2_19_016_vsq_0x81_sq_true_count_1 ... ok
test story_169::test_BC_2_19_016_vsq_0x7F_sq_false_count_127_max ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 84 filtered out; finished in 0.00s
```

Result: **7/7 PASS**

---

## Canonical Vector Table: ASDU bytes 0–1 → TypeID / sq / count

| Byte 0 (TypeID) | Byte 1 (VSQ) | `type_id` | `sq` | `count` | Notes |
|-----------------|--------------|-----------|------|---------|-------|
| `0x2D` (45) | `0x01` | 45 | false | 1 | C_SC_NA_1 single command; BC-2.19.016 primary canonical vector |
| `0x00` | `0x01` | 0 | false | 1 | TypeID=0 undefined; passed through (EC-007) |
| `0xFF` | `0x80` | 255 | true | 0 | All-bits TypeID; SQ=true; count=0 (EC-005) |
| `0x01` | `0x81` | 1 | true | 1 | VSQ=0x81: bit7=1 (sq=true), bits6:0=1 (count=1) |
| `0x01` | `0x03` | 1 | false | 3 | VSQ=0x03: bit7=0 (sq=false), bits6:0=3 (count=3) |
| `0x01` | `0x7F` | 1 | false | 127 | VSQ=0x7F: bit7=0 (sq=false), bits6:0=127 (max count) |

### VSQ Bit Decomposition

```
VSQ byte (asdu_body[1]):
  bit 7 (0x80):  SQ flag  → asdu.sq   = (vsq & 0x80) != 0
  bits 6:0 (0x7F): count  → asdu.count = vsq & 0x7F

Examples:
  VSQ=0x01: sq=false, count=1    (C_SC_NA_1 canonical: 1 object, non-sequence)
  VSQ=0x81: sq=true,  count=1    (bit7 set; sequence addressing enabled)
  VSQ=0x7F: sq=false, count=127  (maximum count with non-sequence mode)
  VSQ=0x80: sq=true,  count=0    (sequence mode, no object count declared)
```

---

## Test Coverage

| Test Name | byte[0] | byte[1] | type_id | sq | count | Condition |
|-----------|---------|---------|---------|-----|-------|-----------|
| `test_BC_2_19_016_type_id_45_c_sc_na_1_canonical_vector` | `0x2D` | `0x01` | 45 | false | 1 | Primary canonical vector (BC-2.19.016 PC1-PC3) |
| `test_BC_2_19_016_type_id_extracted_verbatim_from_byte_0` | `0xAB` | any | 0xAB | - | - | type_id verbatim (BC-2.19.016 PC1) |
| `test_BC_2_19_016_type_id_0_undefined_passthrough_canonical_vector` | `0x00` | `0x01` | 0 | false | 1 | TypeID=0 passthrough (EC-007) |
| `test_BC_2_19_016_type_id_255_vsq_0x80_sq_true_count_0_canonical_vector` | `0xFF` | `0x80` | 255 | true | 0 | All-bits TypeID; SQ=true; count=0 |
| `test_BC_2_19_016_vsq_0x81_sq_true_count_1` | any | `0x81` | - | true | 1 | sq bit extraction (BC-2.19.016 PC2) |
| `test_BC_2_19_016_vsq_0x03_sq_false_count_3` | any | `0x03` | - | false | 3 | count extraction (BC-2.19.016 PC3) |
| `test_BC_2_19_016_vsq_0x7F_sq_false_count_127_max` | any | `0x7F` | - | false | 127 | max count boundary (BC-2.19.016 PC3) |

---

## Verdict

AC-169-002: **PASS** — 7/7 BC-2.19.016 tests green; `type_id` verbatim from byte 0 confirmed; `sq` bit7 extraction and `count` bits6:0 extraction both verified; TypeID=0 undefined passthrough confirmed; count=127 maximum boundary confirmed.
