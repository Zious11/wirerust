# AC-169-004/005 — CASDU (16-bit LE, bytes 4–5) and first_ioa Option (24-bit LE, bytes 6–8)

**Story:** STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields  
**ACs:** AC-169-004, AC-169-005  
**Traces to:** BC-2.19.018 postconditions 1–3  
**Wave:** 78

---

## Acceptance Criteria

- AC-169-004: Given `asdu_body.len() >= 6`, `asdu.casdu == u16::from_le_bytes([asdu_body[4], asdu_body[5]])`
- AC-169-005:
  - Given `asdu_body.len() >= 9` AND `asdu.count > 0`:
    `asdu.first_ioa == Some(u32::from_le_bytes([asdu_body[6], asdu_body[7], asdu_body[8], 0]))`
  - Given `asdu.count == 0` OR `asdu_body.len() < 9`:
    `asdu.first_ioa == None`

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests story_169::test_BC_2_19_018
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 9 tests
test story_169::test_BC_2_19_018_casdu_0_undefined_extracted_without_rejection ... ok
test story_169::test_BC_2_19_018_casdu_little_endian_1_canonical_vector ... ok
test story_169::test_BC_2_19_018_casdu_max_65535_canonical_vector ... ok
test story_169::test_BC_2_19_018_first_ioa_le_byte_order_verified ... ok
test story_169::test_BC_2_19_018_first_ioa_max_0xFFFFFF_canonical_vector ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_7_or_8_bytes_count_gt_0 ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_count_0_regardless_of_length ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_exactly_6_bytes_count_gt_0 ... ok
test story_169::test_BC_2_19_018_first_ioa_some_count_1_len_9_canonical_vector ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 82 filtered out; finished in 0.00s
```

Result: **9/9 PASS** (3 CASDU tests + 6 first_ioa tests)

---

## AC-169-004: CASDU Extraction (BC-2.19.018 PC1)

### CASDU Byte Layout

```
CASDU = u16::from_le_bytes([asdu_body[4], asdu_body[5]])
      = asdu_body[4] as u16 | (asdu_body[5] as u16) << 8

Examples:
  bytes[4..=5] = [0x01, 0x00] → casdu = 1     (RTU/IED address 1; canonical)
  bytes[4..=5] = [0xFF, 0xFF] → casdu = 65535  (global address / max)
  bytes[4..=5] = [0x00, 0x00] → casdu = 0      (undefined; extracted without rejection)
```

### CASDU Test Coverage

| Test Name | bytes[4..=5] | `casdu` | Condition |
|-----------|-------------|---------|-----------|
| `test_BC_2_19_018_casdu_little_endian_1_canonical_vector` | `[0x01, 0x00]` | 1 | LE byte order; primary canonical vector (BC-2.19.018 PC1) |
| `test_BC_2_19_018_casdu_max_65535_canonical_vector` | `[0xFF, 0xFF]` | 65535 | Maximum 16-bit CASDU (global address) |
| `test_BC_2_19_018_casdu_0_undefined_extracted_without_rejection` | `[0x00, 0x00]` | 0 | CASDU=0 undefined; passed through (no rejection) |

---

## AC-169-005: first_ioa Option Extraction (BC-2.19.018 PC2–PC3)

### first_ioa Logic

```rust
let first_ioa = if count > 0 && asdu_body.len() >= 9 {
    Some(u32::from_le_bytes([asdu_body[6], asdu_body[7], asdu_body[8], 0]))
} else {
    None
};
```

Two independent conditions must both hold for `Some(...)`:
1. `count > 0` — at least one information object declared in VSQ
2. `asdu_body.len() >= 9` — bytes 6, 7, 8 exist to form 24-bit LE IOA

If either condition is false, `first_ioa = None`.

### IOA Canonical Vector Table

| Body length | count | bytes[6..=8] | `first_ioa` | Condition |
|-------------|-------|-------------|-------------|-----------|
| 9 | 1 | `[0x01, 0x00, 0x00]` | `Some(1)` | Minimum valid IOA (EC-004; BC-2.19.018 PC2) |
| 9 | 1 | `[0xFF, 0xFF, 0xFF]` | `Some(16777215)` | IOA = 0xFFFFFF max 24-bit (EC-006) |
| 6 | 1 | absent | `None` | count>0 but len<9: insufficient bytes (EC-003) |
| 7 | 1 | partial | `None` | 7 bytes: still insufficient for 3-byte IOA |
| 8 | 1 | partial | `None` | 8 bytes: still insufficient for 3-byte IOA |
| 9 | 0 | any | `None` | count=0: no objects declared (EC-005) |
| 12 | 0 | any | `None` | count=0 overrides len>=9 (EC-005) |

### IOA Byte Order Verification

```
24-bit LE IOA: IOA = bytes[6] | (bytes[7] << 8) | (bytes[8] << 16)

bytes[6..=8] = [0x01, 0x00, 0x00] → IOA = 1         (address 1)
bytes[6..=8] = [0xFF, 0xFF, 0xFF] → IOA = 16777215   (0xFFFFFF; max 24-bit)
bytes[6..=8] = [0x56, 0x34, 0x12] → IOA = 0x123456   (byte-order check: lo-byte first)
```

### first_ioa Test Coverage

| Test Name | len | count | `first_ioa` | Condition |
|-----------|-----|-------|-------------|-----------|
| `test_BC_2_19_018_first_ioa_some_count_1_len_9_canonical_vector` | 9 | 1 | `Some(1)` | Both conditions met (BC-2.19.018 PC2) |
| `test_BC_2_19_018_first_ioa_max_0xFFFFFF_canonical_vector` | 9 | 1 | `Some(16777215)` | IOA = 0xFFFFFF max (EC-006) |
| `test_BC_2_19_018_first_ioa_le_byte_order_verified` | 9 | 1 | `Some(0x123456)` | LE byte-order correctness |
| `test_BC_2_19_018_first_ioa_none_when_exactly_6_bytes_count_gt_0` | 6 | 1 | `None` | count>0 but len=6 < 9 (EC-003) |
| `test_BC_2_19_018_first_ioa_none_when_7_or_8_bytes_count_gt_0` | 7, 8 | 1 | `None` | count>0 but len still < 9 (EC-003) |
| `test_BC_2_19_018_first_ioa_none_when_count_0_regardless_of_length` | 9+ | 0 | `None` | count=0 overrides len>=9 (EC-005) |

---

## Success-Path Demonstration

### AC-169-004: CASDU=1 (primary canonical vector)

```
Input:  asdu_body = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00]
                                              ^^^^  ^^^^
                                         bytes[4]  bytes[5] → casdu = u16::from_le_bytes([0x01, 0x00]) = 1

Result: Some(Asdu { casdu: 1, ... })
```

### AC-169-005: first_ioa=Some (count=1, len=9)

```
Input:  asdu_body = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00]
                                                           ^^^^  ^^^^  ^^^^
                                                      bytes[6]  [7]   [8] → IOA = 1

Result: Some(Asdu { count: 1, first_ioa: Some(1), ... })
```

### AC-169-005: first_ioa=None (count=0, len=9)

```
Input:  asdu_body = &[0x2D, 0x00, 0x06, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00]
                           ^^^^
                      VSQ byte1=0x00 → count = 0x00 & 0x7F = 0

Result: Some(Asdu { count: 0, first_ioa: None, ... })
        count=0 → first_ioa=None regardless of body length
```

---

## Verdict

- AC-169-004: **PASS** — 3/3 CASDU tests green; 16-bit LE extraction verified; CASDU=0 passthrough and CASDU=65535 max boundary confirmed
- AC-169-005: **PASS** — 6/6 first_ioa tests green; `Some(IOA)` requires both count>0 AND len>=9; `None` on count=0 (regardless of length) and None on len<9 (EC-003) both confirmed; IOA=0xFFFFFF max and LE byte-order both verified
