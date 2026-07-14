# AC-167-005 — `parse_apci_header` Returns Some(ApciHeader) for Valid Input

**Story:** STORY-167: IEC-104 APCI Core Parser  
**AC:** AC-167-005  
**Traces to:** BC-2.19.005 postconditions 1–6  
**Wave:** 76

---

## Acceptance Criterion

- Given `data.len() >= 6`, `data[0] == 0x68`, `4 <= data[1] <= 253`
- When `parse_apci_header(data)` is called
- Then returns `Some(ApciHeader { len: data[1], cf1: data[2], cf2: data[3], cf3: data[4], cf4: data[5] })`
- `len` (LEN field) is in `[4, 253]`; `len + 2` (total frame bytes) is in `[6, 255]` — no overflow
- CF1–CF4 are copied verbatim; bytes beyond index 5 are not accessed by this function

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_005
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 7 tests
test story_167::test_BC_2_19_005_cf_fields_verbatim_from_data_indices_2_through_5 ... ok
test story_167::test_BC_2_19_005_apci_header_equality_and_field_layout ... ok
test story_167::test_BC_2_19_005_i_frame_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_005_invariant_len_plus_two_in_range_for_boundaries ... ok
test story_167::test_BC_2_19_005_returns_some_for_len_253_maximum_canonical_vector ... ok
test story_167::test_BC_2_19_005_u_frame_startdt_act_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_005_s_frame_all_fields_correct_canonical_vector ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.00s
```

Result: **7/7 PASS**

---

## Test Coverage

| Test Name | Input Vector | Frame Type | Fields Verified | Result |
|-----------|-------------|------------|-----------------|--------|
| `test_BC_2_19_005_u_frame_startdt_act_all_fields_correct_canonical_vector` | `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` | U-frame STARTDT-act | start, len, cf1–cf4 all 6 fields | PASS |
| `test_BC_2_19_005_s_frame_all_fields_correct_canonical_vector` | `[0x68, 0x04, 0x01, 0x00, 0x00, 0x00]` | S-frame | start, len, cf1–cf4 | PASS |
| `test_BC_2_19_005_i_frame_all_fields_correct_canonical_vector` | `[0x68, 0x0E, 0x00, 0x00, 0x00, 0x00, 0xDE, 0xAD, ...]` | I-frame (10 bytes) | start, len=14, cf1–cf4; bytes beyond [5] not accessed | PASS |
| `test_BC_2_19_005_returns_some_for_len_253_maximum_canonical_vector` | `[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00]` | LEN=253 (max valid) | len=253; len+2=255 ∈ [6,255] | PASS |
| `test_BC_2_19_005_invariant_len_plus_two_in_range_for_boundaries` | LEN=4 and LEN=253 | Both boundaries | len+2 ∈ [6,255] for min and max LEN | PASS |
| `test_BC_2_19_005_cf_fields_verbatim_from_data_indices_2_through_5` | `[0x68, 0x10, 0xAA, 0xBB, 0xCC, 0xDD, 0xFF, 0xFF]` | CF verbatim | cf1=0xAA, cf2=0xBB, cf3=0xCC, cf4=0xDD (data[2..6]) | PASS |
| `test_BC_2_19_005_apci_header_equality_and_field_layout` | Struct construction | Struct layout | PartialEq, field layout, u8 types | PASS |

---

## Success-Path Demonstration

### Canonical Vector: U-frame STARTDT-act

Input: `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]`

Expected return:
```
Some(ApciHeader {
    start: 0x68,   // byte 0: IEC-104 start marker
    len:   4,      // byte 1: LEN=4 (minimum valid; 4 CF octets, no ASDU)
    cf1:   0x07,   // byte 2: STARTDT-act (U-format: bits 1:0 = 0b11, bits 7:2 = 0b000001)
    cf2:   0x00,   // byte 3
    cf3:   0x00,   // byte 4
    cf4:   0x00,   // byte 5
})
```

Verified assertions:
- `h.start == 0x68` ✓
- `h.len == 4` ✓
- `h.cf1 == 0x07` ✓
- `h.cf2 == 0x00` ✓
- `h.cf3 == 0x00` ✓
- `h.cf4 == 0x00` ✓
- `h.len as usize + 2 == 6` ✓ (total frame size in [6, 255])

### Boundary: LEN=253 (maximum valid)

Input: `[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00]`

Expected return:
```
Some(ApciHeader {
    start: 0x68,
    len:   253,    // 0xFD: maximum valid LEN
    cf1:   0x01,
    cf2:   0x00,
    cf3:   0x00,
    cf4:   0x00,
})
```

Verified: `h.len + 2 = 255 ∈ [6, 255]` — no integer overflow (BC-2.19.005 invariant 1).

### CF Verbatim Copy

Input: `[0x68, 0x10, 0xAA, 0xBB, 0xCC, 0xDD, 0xFF, 0xFF]` (8 bytes; trailing 0xFF bytes not accessed)

Verified: `cf1=0xAA, cf2=0xBB, cf3=0xCC, cf4=0xDD` — exact verbatim copy of `data[2..6]`.

---

## Verdict

AC-167-005: **PASS** — 7/7 BC-2.19.005 tests green; all six fields correctly extracted; LEN+2 overflow safety invariant verified at both boundaries.
