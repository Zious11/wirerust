# AC-167-003 / AC-167-004 — `parse_apci_header` Returns None for LEN Out of Range

**Story:** STORY-167: IEC-104 APCI Core Parser  
**ACs:** AC-167-003 (LEN < 4) and AC-167-004 (LEN > 253)  
**Traces to:** BC-2.19.003 postcondition 1 / BC-2.19.004 postcondition 1 and invariant 1  
**Wave:** 76

---

## Acceptance Criteria

**AC-167-003:**
- Given `data.len() >= 6`, `data[0] == 0x68`, and `data[1] < 4` (LEN byte < 4)
- When `parse_apci_header(data)` is called
- Then returns `None`; LEN=4 is the minimum (U-frame CF1–CF4, no ASDU)

**AC-167-004:**
- Given `data.len() >= 6`, `data[0] == 0x68`, and `data[1] > 253` (LEN byte > 253)
- When `parse_apci_header(data)` is called
- Then returns `None`; LEN=253 is the maximum (LEN+2=255 total, fitting in one u8)

---

## Test Suite Execution

### AC-167-003 (LEN lower bound)

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_003
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_167::test_BC_2_19_003_returns_none_for_len_1_and_len_2 ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_3_off_by_one_canonical_vector ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_zero_canonical_vector ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

### AC-167-004 (LEN upper bound)

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_004
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 2 tests
test story_167::test_BC_2_19_004_returns_none_for_len_254_canonical_vector ... ok
test story_167::test_BC_2_19_004_returns_none_for_len_255_canonical_vector ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.00s
```

Result: **2/2 PASS**

---

## Test Coverage

### AC-167-003: LEN < 4 (lower bound rejection)

| Test Name | Canonical Vector | LEN | Condition | Result |
|-----------|-----------------|-----|-----------|--------|
| `test_BC_2_19_003_returns_none_for_len_zero_canonical_vector` | `[0x68, 0x00, 0x07, 0x00, 0x00, 0x00]` | LEN=0 | BC-2.19.003 canonical vector | PASS |
| `test_BC_2_19_003_returns_none_for_len_3_off_by_one_canonical_vector` | `[0x68, 0x03, 0x07, 0x00, 0x00, 0x00]` | LEN=3 | Off-by-one below minimum; BC-2.19.003 canonical vector; STORY-167 EC-004 | PASS |
| `test_BC_2_19_003_returns_none_for_len_1_and_len_2` | LEN=1 and LEN=2 | LEN=1, LEN=2 | Sweep of remaining sub-minimum values | PASS |

### AC-167-004: LEN > 253 (upper bound rejection)

| Test Name | Canonical Vector | LEN | Condition | Result |
|-----------|-----------------|-----|-----------|--------|
| `test_BC_2_19_004_returns_none_for_len_254_canonical_vector` | `[0x68, 0xFE, 0x01, 0x00, 0x00, 0x00]` | LEN=254 (0xFE) | Off-by-one above maximum; BC-2.19.004 canonical vector; STORY-167 EC-007 | PASS |
| `test_BC_2_19_004_returns_none_for_len_255_canonical_vector` | `[0x68, 0xFF, 0x01, 0x00, 0x00, 0x00]` | LEN=255 (0xFF) | Maximum u8 value; BC-2.19.004 canonical vector | PASS |

---

## Boundary Analysis

The LEN field valid range is `[4, 253]`. Rationale:
- **Lower bound (LEN=4):** Minimum IEC-104 APCI carries 4 control octets (CF1–CF4); U-frames with no ASDU are 6 bytes total (2 header + 4 CF).
- **Upper bound (LEN=253):** Maximum ensures `LEN + 2 <= 255`, which fits in a `u8` with no integer overflow. `253 + 2 = 255`.

The tests probe:
- LEN=3 (off-by-one below lower bound) → None
- LEN=4 (minimum valid) → Some (verified in AC-167-005)
- LEN=253 (maximum valid) → Some (verified in AC-167-005)
- LEN=254 (off-by-one above upper bound) → None
- LEN=255 (0xFF, absolute maximum u8) → None

---

## Verdict

AC-167-003: **PASS** — 3/3 LEN lower-bound rejection tests green.  
AC-167-004: **PASS** — 2/2 LEN upper-bound rejection tests green.
