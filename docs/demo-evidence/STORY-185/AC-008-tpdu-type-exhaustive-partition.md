# AC-185-008 — The Four-Way TPDU-Type Match Is Exhaustive and Non-Overlapping Over All 16 Nibble Values

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-008
**Traces to:** BC-2.20.011 invariant 3
**Wave:** 88

---

## Acceptance Criterion

- Given any `u8` value at `tpkt_payload[1] & 0xF0`
- When `parse_cotp_header` classifies it
- Then exactly one of CR (`0xE`), CC (`0xD`), DT-with-payload/DT-empty-payload (`0xF`),
  or the unrecognized-reject arm (the 13 remaining values) applies
- Unit-level spot check; full exhaustiveness is the VP-049 Kani obligation (see
  `AC-010-vp049-kani-skeleton.md`)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests test_BC_2_20_011_tpdu_type_match_is_exhaustive
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 1 test
test story_185::test_BC_2_20_011_tpdu_type_match_is_exhaustive ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

Result: **1/1 PASS**

---

## Test Coverage

| Test Name | Coverage | Result |
|-----------|----------|--------|
| `test_BC_2_20_011_tpdu_type_match_is_exhaustive` | Loops `nibble` over `0x0..=0xF` (all 16 high-nibble values) against a fixed 3-byte input shape (`LI=2`, so CR/CC/DT all reach a defined outcome at `payload_offset == 3`), asserting exactly one of `Some(ConnectRequest)`, `Some(ConnectConfirm)`, `Some(DataTransfer)`, or `None` per value | PASS |

---

## Partition Coverage Table

| High Nibble | Outcome | Class |
|-------------|---------|-------|
| `0x0`–`0xC` (13 values) | `None` | Reject (unrecognized) |
| `0xD` | `Some(ConnectConfirm)` | Accept |
| `0xE` | `Some(ConnectRequest)` | Accept |
| `0xF` | `Some(DataTransfer)` | Accept |

All 16 values are covered by exactly one arm — no value produces more than one outcome
and no value falls through undefined.

---

## Verdict

AC-185-008: **PASS** — Unit-level spot check across all 16 high-nibble values green for
one fixed input shape. Full formal exhaustiveness over every possible `&[u8]` input
shape and every LI value is the VP-049 Kani obligation, executed in STORY-194 (see
`AC-010-vp049-kani-skeleton.md` for the skeleton evidence anchored in this story).
