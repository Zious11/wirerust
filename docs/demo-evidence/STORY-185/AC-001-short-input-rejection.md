# AC-185-001 — `parse_cotp_header` Returns None for Input Shorter Than 2 Bytes

**Story:** STORY-185: S7comm COTP TPDU-Type Parser: `parse_cotp_header`, Protocol-ID
Extraction, VP-049 Kani Skeleton
**AC:** AC-185-001
**Traces to:** BC-2.20.005 postconditions 1–3
**Wave:** 88

---

## Acceptance Criterion

- Given `tpkt_payload.len() < 2` (including the empty-payload case from a TPKT
  `length == 4` header-only frame)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `None`; no bytes accessed beyond the length check, no panic even for
  `len() == 0` (traces to BC-2.20.005 postcondition 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_005
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 2 tests
test story_185::test_BC_2_20_005_invariant_no_panic_across_short_inputs ... ok
test story_185::test_BC_2_20_005_len_shorter_than_2_returns_none ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.00s
```

Result: **2/2 PASS**

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_005_len_shorter_than_2_returns_none` | `[]` (0 bytes), `[0x02]` (1 byte, EC-002) | len=0 and len=1, both < 2 | PASS |
| `test_BC_2_20_005_invariant_no_panic_across_short_inputs` | `[]`, `[0x00]`, `[0xFF]`, `[0x02]` | No panic on any 0- or 1-byte input, including all-zero and all-0xFF content | PASS |

---

## Error-Path Demonstration

The error path is the primary path for this AC: all inputs with `len < 2` must return
`None`.

Key behavioral assertions verified:
- Empty slice `&[]` -> `None` (EC-001: the legitimately-empty payload from a TPKT
  `length == 4` header-only frame — no bytes accessed).
- 1-byte slice `&[0x02]` -> `None` (EC-002: the LI byte alone is insufficient; the
  TPDU-code byte at offset 1 is never read).
- Purity invariant: no panic across 4 sampled short inputs (lengths 0–1, all-zero and
  all-0xFF content).

---

## Verdict

AC-185-001: **PASS** — Both BC-2.20.005 tests green; purity invariant verified.
