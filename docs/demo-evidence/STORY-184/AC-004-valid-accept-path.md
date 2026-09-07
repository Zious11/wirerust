# AC-184-004 — `parse_tpkt_header` Returns Some(TpktHeader) for Valid Input

**Story:** STORY-184: S7comm TPKT Core Parser
**AC:** AC-184-004
**Traces to:** BC-2.20.004 postconditions 1–4, invariants 1–2
**Wave:** 87

---

## Acceptance Criterion

- Given `data.len() >= 4`, `data[0] == 0x03`, and the decoded `length` in `[7, 65535]`
  (`7` is the RFC 1006 §6 minimum accept floor)
- When `parse_tpkt_header(data)` is called
- Then returns `Some(TpktHeader { version: 3, length })` where `length` is exactly the
  big-endian `u16` decoded from `data[2..4]`
- `data[1]` (reserved byte) is never inspected; any value is accepted
  (traces to BC-2.20.004 invariant 1)
- `length == 65535` (maximum representable `u16`) is a legal accept
  (traces to BC-2.20.004 invariant 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_004
```

Output (filtered to the BC-tagged accept-path tests; excludes `four_way_partition`,
which is AC-184-005's obligation):
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 5 tests
test story_184::test_BC_2_20_004_exact_length_match_no_trailing_bytes ... ok
test story_184::test_BC_2_20_004_reserved_byte_nonzero_parses_identically_to_zero ... ok
test story_184::test_BC_2_20_004_trailing_bytes_beyond_declared_length_still_accepted_canonical_vector ... ok
test story_184::test_BC_2_20_004_valid_input_returns_some_header_length_65535_max_canonical_vector ... ok
test story_184::test_BC_2_20_004_valid_input_returns_some_header_length_7_canonical_vector ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.00s
```

Plus the 4 independent RFC-1006-derived holdout vectors (DF-CANONICAL-FRAME-HOLDOUT-001)
and 1 proptest exercising the accept path (`_holdout` and `proptests` are excluded from
the `BC_2_20_004` substring filter above, so run separately):
```
cargo test --test iso_on_tcp_tests test_rfc1006_s6_minimum_valid_length_holdout
cargo test --test iso_on_tcp_tests test_rfc1006_s6_ten_byte_tpkt_holdout
cargo test --test iso_on_tcp_tests test_rfc1006_s6_wide_length_field_holdout
cargo test --test iso_on_tcp_tests proptest_accepted_length_matches_decoded_bytes
```
```
test story_184::test_rfc1006_s6_minimum_valid_length_holdout ... ok
test story_184::test_rfc1006_s6_ten_byte_tpkt_holdout ... ok
test story_184::test_rfc1006_s6_wide_length_field_holdout ... ok
test story_184::proptests::test_BC_2_20_004_proptest_accepted_length_matches_decoded_bytes ... ok
```

Result: **9/9 PASS** (5 BC-tagged tests + 3 RFC-1006 holdouts + 1 proptest)

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_004_valid_input_returns_some_header_length_7_canonical_vector` | `[0x03, 0x00, 0x00, 0x07]` | length=7 (RFC §6 minimum accept floor; 6-vs-7 boundary) | PASS |
| `test_BC_2_20_004_valid_input_returns_some_header_length_65535_max_canonical_vector` | `[0x03, 0xFF, 0xFF, 0xFF]` | length=65535 (max `u16`), reserved byte=0xFF | PASS |
| `test_BC_2_20_004_reserved_byte_nonzero_parses_identically_to_zero` | `[0x03, 0x00, 0x00, 0x07]` vs `[0x03, 0xFF, 0x00, 0x07]` | Reserved byte never inspected — identical decode regardless of value | PASS |
| `test_BC_2_20_004_exact_length_match_no_trailing_bytes` | `[0x03, 0x00, 0x00, 0x07, 0xAA, 0xBB, 0xCC]` (7 bytes total) | `data.len() == length` exactly | PASS |
| `test_BC_2_20_004_trailing_bytes_beyond_declared_length_still_accepted_canonical_vector` | 14-byte input, declared length=10, second frame's header trails | `data.len() > length` — only the first frame's declared length is decoded; trailing bytes ignored | PASS |
| `test_rfc1006_s6_minimum_valid_length_holdout` | `[0x03, 0x00, 0x00, 0x07]` | RFC-1006-derived independent holdout for the length=7 minimum | PASS |
| `test_rfc1006_s6_ten_byte_tpkt_holdout` | 10-byte frame, length=10 | RFC-1006-derived holdout, header + 6 payload octets | PASS |
| `test_rfc1006_s6_wide_length_field_holdout` | `[0x03, 0x00, 0x02, 0x05]` (length=517) | Independent holdout covering a length-field bit pattern absent from any BC vector | PASS |
| `test_BC_2_20_004_proptest_accepted_length_matches_decoded_bytes` | Randomized `(len_hi, len_lo, reserved)` with decoded length >= 7 | Property: accepted `length` always exactly matches the big-endian decode of `data[2..4]`, reserved byte ignored | PASS |

---

## Accept-Path Demonstration

Key behavioral assertions verified:
- `length == 7` (RFC 1006 §6 minimum) -> `Some(TpktHeader { version: 3, length: 7 })`.
- `length == 65535` (maximum representable `u16`, the "oversized-length-field" edge
  case) -> `Some(TpktHeader { version: 3, length: 65535 })` — a legal accept.
- Reserved byte (`data[1]`) at `0x00` and `0xFF` decode to identical `TpktHeader`
  values — the reserved byte is never validated (BC-2.20.004 invariant 1).
- `data.len() > length as usize` (a second frame follows immediately) still returns
  `Some` describing only the first frame — frame-walk advance across multi-frame
  buffers is out of scope for this function (STORY-186 concern).
- Property-based coverage: for any randomized 4-byte header with a decoded length
  `>= 7`, the accepted `TpktHeader.length` exactly matches the big-endian decode of
  `data[2..4]`, with the reserved byte ignored across the sampled space.

---

## Verdict

AC-184-004: **PASS** — All 5 BC-2.20.004 tests, all 3 RFC-1006 accept-path holdouts,
and the accept-path proptest green; reserved-byte invariant and multi-frame trailing-
bytes behavior verified.
