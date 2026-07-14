# AC-167-001 — `parse_apci_header` Returns None for Input Shorter Than 6 Bytes

**Story:** STORY-167: IEC-104 APCI Core Parser  
**AC:** AC-167-001  
**Traces to:** BC-2.19.001 postconditions 1–3  
**Wave:** 76

---

## Acceptance Criterion

- Given a `&[u8]` slice with `len < 6` (including empty slice, 1-byte, 5-byte)
- When `parse_apci_header(data)` is called
- Then returns `None` without accessing any bytes; no panics; no partial decode
- The function is pure: no side effects, no global state mutation

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests BC_2_19_001
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 5 tests
test story_167::test_BC_2_19_001_invariant_no_panic_on_truncated_inputs ... ok
test story_167::test_BC_2_19_001_returns_none_for_empty_slice ... ok
test story_167::test_BC_2_19_001_returns_none_for_two_bytes ... ok
test story_167::test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_one_byte ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.00s
```

Result: **5/5 PASS**

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-----------------|---------------------|--------|
| `test_BC_2_19_001_returns_none_for_empty_slice` | `[]` (0 bytes) | len=0 < 6 | PASS |
| `test_BC_2_19_001_returns_none_for_one_byte` | `[0x68]` | len=1 < 6, even with valid start byte | PASS |
| `test_BC_2_19_001_returns_none_for_two_bytes` | `[0x68, 0x04]` | len=2 < 6 | PASS |
| `test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector` | `[0x68, 0x04, 0x07, 0x00, 0x00]` | len=5 < 6, canonical BC vector | PASS |
| `test_BC_2_19_001_invariant_no_panic_on_truncated_inputs` | 8 inputs, len 0–5 | No panic on any truncated input (purity invariant) | PASS |

---

## Error-Path Demonstration

The error path is the primary path for this AC: all inputs with `len < 6` must return `None`.

Key behavioral assertions verified:
- Empty slice `&[]` → `None` (EC-001: no bytes accessed)
- 1-byte slice `&[0x68]` → `None` (even the valid start byte alone is insufficient)
- 5-byte slice `[0x68, 0x04, 0x07, 0x00, 0x00]` → `None` (one short of minimum; BC-2.19.001 canonical vector)
- Purity invariant: no panic for any of 8 sampled truncated inputs (lengths 0–5, various content)

---

## Verdict

AC-167-001: **PASS** — All 5 BC-2.19.001 tests green; purity invariant verified.
