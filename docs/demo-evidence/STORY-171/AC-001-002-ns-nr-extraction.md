# AC-171-001 / AC-171-002 — N(S) and N(R) 15-Bit Sequence Number Extraction

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking  
**ACs:** AC-171-001, AC-171-002  
**Traces to:** BC-2.19.023 postconditions 1–4; proptest VP-045 (15-bit range invariant)  
**Wave:** 80

---

## Acceptance Criteria

**AC-171-001:** `extract_ns(cf1, cf2) -> u16` computes `((cf1 as u16) >> 1) | ((cf2 as u16) << 7)`, range [0, 32767].

**AC-171-002:** `extract_nr(cf3, cf4) -> u16` uses the same symmetric formula. N(R) is transient — `Iec104FlowState` has NO `last_nr` field.

---

## Test Suite Execution — BC-2.19.023

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_023
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 12 tests
test story_171::test_BC_2_19_023_extract_nr_cf3_0x00_cf4_0x00_returns_0 ... ok
test story_171::test_BC_2_19_023_extract_nr_cf3_0x02_cf4_0x00_returns_1 ... ok
test story_171::test_BC_2_19_023_extract_nr_cf3_0xFE_cf4_0xFF_returns_32767 ... ok
test story_171::test_BC_2_19_023_extract_nr_is_transient_no_last_nr_field_in_flow_state ... ok
test story_171::test_BC_2_19_023_extract_nr_symmetric_formula_equal_inputs_equal_outputs ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x00_returns_0 ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x80_returns_16384 ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0x02_cf2_0x00_returns_1 ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0xFE_cf2_0xFF_returns_32767 ... ok
test story_171::test_BC_2_19_023_invariant_extract_ns_range_and_exact_values_boundary_inputs ... ok
test story_171::test_BC_2_19_023_proptest_extract_ns_always_in_15bit_range ... ok
test story_171::test_BC_2_19_023_proptest_extract_nr_always_in_15bit_range ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 154 filtered out; finished in 0.01s
```

Result: **12/12 PASS**

---

## AC-171-001: N(S) Extraction Test Coverage

### Canonical Vector Tests (BC-2.19.023 PC1, PC3)

| Test Name | Input (CF1, CF2) | Expected N(S) | Formula Verification | Result |
|-----------|-----------------|---------------|----------------------|--------|
| `test_BC_2_19_023_extract_ns_cf1_0x02_cf2_0x00_returns_1` | CF1=0x02, CF2=0x00 | 1 | (0x02 >> 1) \| (0x00 << 7) = 1 \| 0 = 1 | PASS |
| `test_BC_2_19_023_extract_ns_cf1_0xFE_cf2_0xFF_returns_32767` | CF1=0xFE, CF2=0xFF | 32767 | (0xFE >> 1) \| (0xFF << 7) = 0x7F \| 0x7F80 = 0x7FFF = 32767 | PASS |
| `test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x00_returns_0` | CF1=0x00, CF2=0x00 | 0 | (0x00 >> 1) \| (0x00 << 7) = 0 | PASS |
| `test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x80_returns_16384` | CF1=0x00, CF2=0x80 | 16384 | (0x00 >> 1) \| (0x80 << 7) = 0 \| 0x4000 = 16384 | PASS |

### Invariant and Proptest

| Test Name | Invariant | Assertion | Result |
|-----------|-----------|-----------|--------|
| `test_BC_2_19_023_invariant_extract_ns_range_and_exact_values_boundary_inputs` | BC-2.19.023 inv | N(S) always in [0, 32767] for all boundary inputs | PASS |
| `test_BC_2_19_023_proptest_extract_ns_always_in_15bit_range` | VP-045 (15-bit range) | `extract_ns(any_cf1, any_cf2) & 0x8000 == 0` for all inputs | PASS |

---

## AC-171-002: N(R) Extraction Test Coverage

### Canonical Vector Tests (BC-2.19.023 PC2, PC4)

| Test Name | Input (CF3, CF4) | Expected N(R) | Formula Verification | Result |
|-----------|-----------------|---------------|----------------------|--------|
| `test_BC_2_19_023_extract_nr_cf3_0x02_cf4_0x00_returns_1` | CF3=0x02, CF4=0x00 | 1 | (0x02 >> 1) \| (0x00 << 7) = 1 | PASS |
| `test_BC_2_19_023_extract_nr_cf3_0xFE_cf4_0xFF_returns_32767` | CF3=0xFE, CF4=0xFF | 32767 | (0xFE >> 1) \| (0xFF << 7) = 32767 | PASS |
| `test_BC_2_19_023_extract_nr_cf3_0x00_cf4_0x00_returns_0` | CF3=0x00, CF4=0x00 | 0 | (0x00 >> 1) \| (0x00 << 7) = 0 | PASS |
| `test_BC_2_19_023_extract_nr_symmetric_formula_equal_inputs_equal_outputs` | CF3=CF1, CF4=CF2 | extract_nr == extract_ns | symmetric formula identity | PASS |

### Transient / No Storage (BC-2.19.023 PC4 — N(R) NOT stored)

| Test Name | Assertion | Result |
|-----------|-----------|--------|
| `test_BC_2_19_023_extract_nr_is_transient_no_last_nr_field_in_flow_state` | `Iec104FlowState` has no `last_nr_c2s` / `last_nr_s2c` field — N(R) is compute-only | PASS |
| `test_BC_2_19_023_proptest_extract_nr_always_in_15bit_range` | `extract_nr(any_cf3, any_cf4) & 0x8000 == 0` for all inputs | PASS |

---

## Extraction Formula Summary

Both `extract_ns` and `extract_nr` implement the IEC 60870-5-104 15-bit sequence
number formula:

```
N(S) = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7)
N(R) = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7)
```

The LSB of CF1/CF3 is a format discriminator bit, not part of the sequence number —
hence the right-shift discards it. The remaining 7 bits from CF1 form the low 7 bits
of the 15-bit value; CF2/CF4 provides bits 7–14.

| CF1/CF3 | CF2/CF4 | N(S)/N(R) | Notes |
|---------|---------|-----------|-------|
| 0x02 | 0x00 | 1 | Minimum non-zero (BC canonical vector) |
| 0xFE | 0xFF | 32767 | Maximum (BC canonical vector) |
| 0x00 | 0x00 | 0 | Zero / fresh state |
| 0x00 | 0x80 | 16384 | Midpoint (third BC canonical vector) |

---

## Verdict

AC-171-001: **PASS** — All 6 `extract_ns` tests green (4 canonical vectors + 1 invariant + 1 proptest).  
AC-171-002: **PASS** — All 6 `extract_nr` tests green (3 canonical vectors + 1 symmetric + 1 transient + 1 proptest).  
Both pure free functions confirmed; `Iec104FlowState` has no `last_nr` storage as required.
