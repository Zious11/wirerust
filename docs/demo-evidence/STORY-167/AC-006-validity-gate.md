# AC-167-006 — `is_valid_iec104_frame` Post-Classification Validity Gate

**Story:** STORY-167: IEC-104 APCI Core Parser  
**AC:** AC-167-006  
**Traces to:** BC-2.19.006 postconditions 1–3 and invariant 3  
**Wave:** 76

---

## Acceptance Criterion

- Given a `&[u8]` slice from a port-2404-dispatched flow
- When `is_valid_iec104_frame(data)` is called
- Then returns `true` iff `data.len() >= 2 && data[0] == 0x68 && 4 <= data[1] <= 253`
- Returns `false` for empty slice, wrong start byte, or out-of-range LEN
- Any input where `is_valid_iec104_frame` returns `true` and `data.len() >= 6` will yield `Some` from `parse_apci_header`

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_006
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 10 tests
test story_167::test_BC_2_19_006_invariant_consistency_with_parse_apci_header ... ok
test story_167::test_BC_2_19_006_returns_false_for_empty_slice ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_3_below_minimum ... ok
test story_167::test_BC_2_19_006_invariant_false_gate_implies_none_from_parse ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_254_above_maximum ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_ff_out_of_range_canonical_vector ... ok
test story_167::test_BC_2_19_006_returns_false_for_one_byte_slice ... ok
test story_167::test_BC_2_19_006_returns_false_for_wrong_start_byte_canonical_vector ... ok
test story_167::test_BC_2_19_006_returns_true_for_valid_start_and_len_253 ... ok
test story_167::test_BC_2_19_006_returns_true_for_valid_start_and_len_4_canonical_vector ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s
```

Result: **10/10 PASS**

---

## Test Coverage

### Success Path (returns true)

| Test Name | Input Vector | LEN | Result |
|-----------|-------------|-----|--------|
| `test_BC_2_19_006_returns_true_for_valid_start_and_len_4_canonical_vector` | `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` | LEN=4 (min) | PASS |
| `test_BC_2_19_006_returns_true_for_valid_start_and_len_253` | `[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00]` | LEN=253 (max) | PASS |

### Error Path (returns false)

| Test Name | Input Vector | Failure Reason | Result |
|-----------|-------------|----------------|--------|
| `test_BC_2_19_006_returns_false_for_empty_slice` | `[]` | len < 2 (can't read start byte) | PASS |
| `test_BC_2_19_006_returns_false_for_one_byte_slice` | `[0x68]` | len=1 < 2 (can't read LEN byte); EC-008 | PASS |
| `test_BC_2_19_006_returns_false_for_wrong_start_byte_canonical_vector` | `[0x48, 0x04, ...]` | start≠0x68 (BC-2.19.006 canonical vector) | PASS |
| `test_BC_2_19_006_returns_false_for_len_ff_out_of_range_canonical_vector` | `[0x68, 0xFF, ...]` | LEN=255 > 253 (BC-2.19.006 canonical vector) | PASS |
| `test_BC_2_19_006_returns_false_for_len_3_below_minimum` | `[0x68, 0x03, ...]` | LEN=3 < 4 (EC-004) | PASS |
| `test_BC_2_19_006_returns_false_for_len_254_above_maximum` | `[0x68, 0xFE, ...]` | LEN=254 > 253 (EC-005) | PASS |

### Invariant Tests (consistency with `parse_apci_header`)

| Test Name | Purpose | Result |
|-----------|---------|--------|
| `test_BC_2_19_006_invariant_consistency_with_parse_apci_header` | Forward: gate=true AND len>=6 → parse returns Some (3 valid inputs) | PASS |
| `test_BC_2_19_006_invariant_false_gate_implies_none_from_parse` | Contrapositive: gate=false → parse returns None (3 invalid inputs) | PASS |

---

## Invariant 2 Demonstration (BC-2.19.006 consistency)

The two invariant tests prove the gate's consistency contract with `parse_apci_header`:

**Forward direction** — inputs `[0x68,0x04,...], [0x68,0xFD,...], [0x68,0x64,...]`:
```
is_valid_iec104_frame(data) == true  (for each)
data.len() >= 6                       (verified)
parse_apci_header(data) == Some(_)   (confirmed)
```

**Contrapositive** — inputs `[0x00,0x04,...], [0x68,0x03,...], [0x68,0xFE,...]`:
```
is_valid_iec104_frame(data) == false  (for each)
parse_apci_header(data) == None       (confirmed)
```

The gate is a **lightweight 2-byte check** (ADR-013 Decision 1) used on port-2404-dispatched
flows to compensate for false-positive port classification without adding a content signature
to the `classify()` rule table.

---

## Verdict

AC-167-006: **PASS** — 10/10 BC-2.19.006 tests green; both success and error paths verified; invariant consistency with `parse_apci_header` proved in both directions.
