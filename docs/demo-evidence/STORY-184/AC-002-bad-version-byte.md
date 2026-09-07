# AC-184-002 — `parse_tpkt_header` Returns None for Version Byte != 0x03

**Story:** STORY-184: S7comm TPKT Core Parser
**AC:** AC-184-002
**Traces to:** BC-2.20.002 postconditions 1–2, invariant 2
**Wave:** 87

---

## Acceptance Criterion

- Given `data.len() >= 4` and `data[0] != 0x03`
- When `parse_tpkt_header(data)` is called
- Then returns `None`; the length field (`data[2..4]`) is never decoded
  (traces to BC-2.20.002 postcondition 2)
- No panic for any `u8` value of `data[0]` (traces to BC-2.20.002 invariant 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_002
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 5 tests
test story_184::test_BC_2_20_002_bad_version_short_circuits_before_length_decode ... ok
test story_184::test_BC_2_20_002_invariant_no_panic_across_version_byte_sample ... ok
test story_184::test_BC_2_20_002_returns_none_for_version_0x00_canonical_vector ... ok
test story_184::test_BC_2_20_002_returns_none_for_version_0x04_off_by_one_canonical_vector ... ok
test story_184::test_BC_2_20_002_returns_none_for_version_0xFF_canonical_vector ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.00s
```

Result: **5/5 PASS**

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_002_returns_none_for_version_0x00_canonical_vector` | `[0x00, 0x00, 0x00, 0x04]` | version=0x00 | PASS |
| `test_BC_2_20_002_returns_none_for_version_0x04_off_by_one_canonical_vector` | `[0x04, 0x00, 0x00, 0x04]` | version=0x04 (off-by-one, no leniency) | PASS |
| `test_BC_2_20_002_returns_none_for_version_0xFF_canonical_vector` | `[0xFF, 0x00, 0x00, 0x04]` | version=0xFF | PASS |
| `test_BC_2_20_002_bad_version_short_circuits_before_length_decode` | `[0x02, 0x00, 0xFF, 0xFF]` | bad version with a length field that would otherwise decode to the maximally-legal 65535 — proves the version check short-circuits before length decode (postcondition 2) | PASS |
| `test_BC_2_20_002_invariant_no_panic_across_version_byte_sample` | 8 sampled `u8` values (0x01, 0x02, 0x05, 0x10, 0x7F, 0x80, 0xFE, 0xFF) | No panic for any `data[0] != 0x03` (invariant 2 spot check) | PASS |

---

## Error-Path Demonstration

Key behavioral assertions verified:
- `data[0] == 0x00` -> `None`.
- `data[0] == 0x04` (adjacent to the valid `0x03`) -> `None` — no off-by-one leniency.
- `data[0] == 0xFF` -> `None`.
- Version-check-before-length-decode ordering: `[0x02, 0x00, 0xFF, 0xFF]` (length bytes
  decode to 65535, the maximum legal length) still returns `None` — the length field is
  never inspected once the version guard fails (BC-2.20.002 postcondition 2).
- Purity invariant: no panic across an 8-value sample of the `u8` domain excluding
  `0x03` (full 256-value totality is the VP-048 Kani obligation — see AC-184-006).

This AC also anchors SS-20's resync behavior: a non-`0x03` version byte is the signal
the frame-walk loop (STORY-186) uses to attempt byte-at-a-time resynchronization on a
malformed TPKT stream.

---

## Verdict

AC-184-002: **PASS** — All 5 BC-2.20.002 tests green; version-before-length guard
ordering and purity invariant verified.
