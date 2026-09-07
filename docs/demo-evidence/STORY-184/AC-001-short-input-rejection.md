# AC-184-001 — `parse_tpkt_header` Returns None for Input Shorter Than 4 Bytes

**Story:** STORY-184: S7comm TPKT Core Parser: `parse_tpkt_header` Pure-Core Free
Function + VP-048 Kani Skeleton
**AC:** AC-184-001
**Traces to:** BC-2.20.001 postconditions 1–3
**Wave:** 87

---

## Acceptance Criterion

- Given a `&[u8]` slice with `data.len() < 4` (including empty, 1-byte, 3-byte slices)
- When `parse_tpkt_header(data)` is called
- Then returns `None` without accessing any byte in `data`; no panics
  (traces to BC-2.20.001 postcondition 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_001
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 5 tests
test story_184::test_BC_2_20_001_invariant_no_panic_on_truncated_inputs ... ok
test story_184::test_BC_2_20_001_returns_none_for_empty_slice ... ok
test story_184::test_BC_2_20_001_returns_none_for_one_byte ... ok
test story_184::test_BC_2_20_001_returns_none_for_three_bytes_canonical_vector ... ok
test story_184::test_BC_2_20_001_returns_none_for_two_bytes ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.00s
```

Result: **5/5 PASS**

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_001_returns_none_for_empty_slice` | `[]` (0 bytes) | len=0 < 4 | PASS |
| `test_BC_2_20_001_returns_none_for_one_byte` | `[0x03]` | len=1 < 4, even with valid version byte | PASS |
| `test_BC_2_20_001_returns_none_for_two_bytes` | `[0x03, 0x00]` | len=2 < 4 | PASS |
| `test_BC_2_20_001_returns_none_for_three_bytes_canonical_vector` | `[0x03, 0x00, 0x00]` | len=3 < 4, canonical BC-2.20.001 vector | PASS |
| `test_BC_2_20_001_invariant_no_panic_on_truncated_inputs` | 7 inputs, len 0–3, varied content | No panic on any truncated input (purity invariant) | PASS |

---

## Error-Path Demonstration

The error path is the primary path for this AC: all inputs with `len < 4` must return
`None`.

Key behavioral assertions verified:
- Empty slice `&[]` -> `None` (EC-001: no bytes accessed).
- 1-byte slice `&[0x03]` -> `None` (the valid version byte alone is insufficient; the
  length-guard fires before the version byte is ever inspected).
- 3-byte slice `[0x03, 0x00, 0x00]` -> `None` (one byte short of the 4-byte minimum;
  BC-2.20.001 canonical vector, EC-002).
- Purity invariant: no panic across 7 sampled truncated inputs (lengths 0–3, all-zero and
  all-0xFF content).

---

## Verdict

AC-184-001: **PASS** — All 5 BC-2.20.001 tests green; purity invariant verified.
