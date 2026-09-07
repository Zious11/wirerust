# AC-184-003 — `parse_tpkt_header` Returns None for Decoded Length < 7 (RFC 1006 §6 Minimum)

**Story:** STORY-184: S7comm TPKT Core Parser
**AC:** AC-184-003
**Traces to:** BC-2.20.003 postcondition 1, invariant 2
**Wave:** 87

---

## Acceptance Criterion

- Given `data.len() >= 4`, `data[0] == 0x03`, and
  `u16::from_be_bytes([data[2], data[3]]) < 7` (RFC 1006 §6 minimum)
- When `parse_tpkt_header(data)` is called
- Then returns `None`; no panic or overflow for any `u16` length value, including `0`
  (traces to BC-2.20.003 invariant 2)

RFC 1006 §6 states the minimum legal TPKT packet length is 7 (4-byte TPKT header +
3-byte minimum COTP unit that must follow it). This AC covers the full sub-minimum
range `[0, 6]`, including the genuine 6-vs-7 accept-floor boundary (paired with
AC-184-004's `length == 7` accept case).

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_003
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 8 tests
test story_184::test_BC_2_20_003_invariant_no_panic_across_sub_minimum_lengths ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_five_below_rfc_minimum ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_four_below_rfc_minimum ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_one_canonical_vector ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_six_boundary_below_rfc_minimum ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_three_off_by_one_canonical_vector ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_two ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_zero_canonical_vector ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
```

Also within scope of this AC — the independent RFC-1006-derived holdout vector
(DF-CANONICAL-FRAME-HOLDOUT-001) for the length=4 header-only case:

```
cargo test --test iso_on_tcp_tests test_rfc1006_s6_length_four_below_minimum_returns_none
```
```
running 1 test
test story_184::test_rfc1006_s6_length_four_below_minimum_returns_none ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.00s
```

Result: **9/9 PASS** (8 BC-tagged tests + 1 RFC-1006-derived holdout)

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_003_returns_none_for_length_zero_canonical_vector` | `[0x03, 0x00, 0x00, 0x00]` | length=0 (most degenerate case) | PASS |
| `test_BC_2_20_003_returns_none_for_length_one_canonical_vector` | `[0x03, 0x00, 0x00, 0x01]` | length=1 | PASS |
| `test_BC_2_20_003_returns_none_for_length_two` | `[0x03, 0x00, 0x00, 0x02]` | length=2 | PASS |
| `test_BC_2_20_003_returns_none_for_length_three_off_by_one_canonical_vector` | `[0x03, 0x00, 0x00, 0x03]` | length=3 | PASS |
| `test_BC_2_20_003_returns_none_for_length_four_below_rfc_minimum` | `[0x03, 0x00, 0x00, 0x04]` | length=4 (the TPKT header's own structural floor, still below the RFC §6 floor of 7) | PASS |
| `test_BC_2_20_003_returns_none_for_length_five_below_rfc_minimum` | `[0x03, 0x00, 0x00, 0x05]` | length=5 | PASS |
| `test_BC_2_20_003_returns_none_for_length_six_boundary_below_rfc_minimum` | `[0x03, 0x00, 0x00, 0x06]` | length=6 (one below the RFC §6 minimum; the genuine 6-vs-7 boundary) | PASS |
| `test_BC_2_20_003_invariant_no_panic_across_sub_minimum_lengths` | 7 length-byte pairs, decoded 0–6 | No overflow/panic for any sub-minimum `u16` length (invariant 2) | PASS |
| `test_rfc1006_s6_length_four_below_minimum_returns_none` | `[0x03, 0x00, 0x00, 0x04]` | RFC-1006-derived independent holdout (DF-CANONICAL-FRAME-HOLDOUT-001), authored independently of the BC text | PASS |

---

## Error-Path Demonstration

Key behavioral assertions verified:
- `length == 0` (all-zero length field) -> `None` (EC-005, most degenerate case).
- `length == 6` (one below the RFC 1006 §6 minimum of 7) -> `None` (EC-006) — this is
  the genuine accept-floor boundary; `length == 7` is the paired accept case
  demonstrated in `AC-004-valid-accept-path.md`.
- Purity/overflow invariant: no panic or overflow across all sub-minimum `u16` length
  values sampled (0–6), including the all-zero length-field byte pattern.
- Spec-independent grounding: `test_rfc1006_s6_length_four_below_minimum_returns_none`
  is derived directly from RFC 1006 §6 rather than reusing BC-2.20.003 vector text
  verbatim (DF-CANONICAL-FRAME-HOLDOUT-001), reducing the risk that a shared
  spec-transcription error in the BC would go undetected.

---

## Verdict

AC-184-003: **PASS** — All 8 BC-2.20.003 tests plus the 1 independent RFC-1006 holdout
green; the 6-vs-7 accept-floor boundary and overflow-safety invariant verified.
