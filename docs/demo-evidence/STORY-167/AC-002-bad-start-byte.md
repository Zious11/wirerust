# AC-167-002 — `parse_apci_header` Returns None for Start Byte ≠ 0x68

**Story:** STORY-167: IEC-104 APCI Core Parser  
**AC:** AC-167-002  
**Traces to:** BC-2.19.002 postcondition 1 and invariant 1  
**Wave:** 76

---

## Acceptance Criterion

- Given `data.len() >= 6` and `data[0] != 0x68`
- When `parse_apci_header(data)` is called
- Then returns `None`; the IEC-104 start byte (0x68) is fixed by protocol specification

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_002
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x00_canonical_vector ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x69_off_by_one ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0xFF_canonical_vector ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-----------------|---------------------|--------|
| `test_BC_2_19_002_returns_none_for_start_byte_0x00_canonical_vector` | `[0x00, 0x04, 0x07, 0x00, 0x00, 0x00]` | start=0x00 (BC-2.19.002 canonical vector) | PASS |
| `test_BC_2_19_002_returns_none_for_start_byte_0xFF_canonical_vector` | `[0xFF, 0x04, 0x07, 0x00, 0x00, 0x00]` | start=0xFF (BC-2.19.002 canonical vector) | PASS |
| `test_BC_2_19_002_returns_none_for_start_byte_0x69_off_by_one` | `[0x69, 0x04, 0x07, 0x00, 0x00, 0x00]` | start=0x69 (off-by-one from 0x68; EC-003) | PASS |

---

## Error-Path Demonstration

All three vectors have `len >= 6` (length guard passes) but `data[0] != 0x68` (start byte guard fires):

- `[0x00, 0x04, 0x07, 0x00, 0x00, 0x00]` → `None`: start byte is null (0x00)
- `[0xFF, 0x04, 0x07, 0x00, 0x00, 0x00]` → `None`: start byte is max (0xFF)
- `[0x69, 0x04, 0x07, 0x00, 0x00, 0x00]` → `None`: start byte is 0x69, one above 0x68 (STORY-167 EC-003)

The IEC 60870-5-104 standard §5.1 mandates 0x68 as the unique frame start marker. Any other value
is a non-IEC-104 byte stream and must be rejected.

---

## Verdict

AC-167-002: **PASS** — All 3 BC-2.19.002 tests green; start-byte rejection verified for null, max, and off-by-one values.
