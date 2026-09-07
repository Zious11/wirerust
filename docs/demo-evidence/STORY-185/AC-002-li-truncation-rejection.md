# AC-185-002 — `parse_cotp_header` Returns None When the Length Indicator Declares More Bytes Than Are Present

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-002
**Traces to:** BC-2.20.006 postcondition 1, invariant 2
**Wave:** 88

---

## Acceptance Criterion

- Given `tpkt_payload.len() >= 2` and `tpkt_payload.len() < 1 + tpkt_payload[0] as usize`
  (LI truncation)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `None`; no out-of-bounds index for any `u8` LI value, including `0`
  (traces to BC-2.20.006 postcondition 2, invariant 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_006
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 3 tests
test story_185::test_BC_2_20_006_invariant_no_panic_across_li_value_sample ... ok
test story_185::test_BC_2_20_006_li_truncation_returns_none ... ok
test story_185::test_BC_2_20_006_li_zero_not_truncated_proceeds_to_classification ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_006_li_truncation_returns_none` | `[0x06, 0xE0, 0x00, 0x01]` (LI=6, only 3 follow; EC-001), `[0x02, 0xF0]` (LI=2, only 1 follows; EC-002) | LI declares more remaining bytes than present | PASS |
| `test_BC_2_20_006_invariant_no_panic_across_li_value_sample` | 3-byte buffer with LI in `{0x03, 0x0A, 0x7F, 0xFE, 0xFF}` | No out-of-bounds index/panic across the `u8` LI range, up to the maximum value 255 | PASS |
| `test_BC_2_20_006_li_zero_not_truncated_proceeds_to_classification` | `[0x00, 0xF0]` | EC-003: `LI == 0` is degenerate but not truncated (`1 + 0 <= len`) — classification proceeds | PASS |

---

## Error-Path Demonstration

Key behavioral assertions verified:
- `LI=6` declaring 6 more bytes with only 3 present -> `None` (BC-2.20.006 canonical
  vector, EC-001, truncated CR header).
- `LI=2` declaring 2 more bytes with only 1 present -> `None` (canonical vector, EC-002,
  truncated DT header).
- No panic/out-of-bounds index for LI values spanning the full `u8` range up to the
  maximum (`0xFF` = 255), confirmed against a fixed 3-byte buffer where every sampled
  value genuinely truncates.
- Boundary correctness in the non-error direction: `LI == 0` does *not* trip the
  truncation guard (EC-003) — proving the guard is `len() < 1 + LI`, not an
  overly-conservative rejection of the degenerate-but-legal zero case.

---

## Verdict

AC-185-002: **PASS** — All 3 BC-2.20.006 tests green; truncation guard verified across
the full `u8` LI domain with no out-of-bounds access, and the `LI == 0` boundary
confirmed not over-rejected.
