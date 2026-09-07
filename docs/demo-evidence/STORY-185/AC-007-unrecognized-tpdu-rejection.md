# AC-185-007 — `parse_cotp_header` Returns None for an Unrecognized TPDU-Type Code

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-007
**Traces to:** BC-2.20.011 postconditions 1–2
**Wave:** 88

---

## Acceptance Criterion

- Given `tpkt_payload[1] & 0xF0` is none of `0xE0` (CR), `0xD0` (CC), `0xF0` (DT) — i.e.
  one of the 13 remaining nibble values (DR, DC, ED, AK, EA, RJ, ER, and others)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `None`; no panic for any of the 13 remaining nibble values; the frame is
  never force-fit into CR, CC, or DT (traces to BC-2.20.011 postcondition 2, invariant 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests test_BC_2_20_011_unrecognized_tpdu_type_returns_none
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 1 test
test story_185::test_BC_2_20_011_unrecognized_tpdu_type_returns_none ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

Plus 1 independent RFC-905-derived holdout vector (DF-CANONICAL-FRAME-HOLDOUT-001):

```
cargo test --test iso_on_tcp_tests test_iso8073_rfc905_table8_dr_code_not_modeled_holdout
```
```
running 1 test
test story_185::test_iso8073_rfc905_table8_dr_code_not_modeled_holdout ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

Result: **2/2 PASS** (1 BC-tagged test + 1 RFC-905 holdout)

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_011_unrecognized_tpdu_type_returns_none` | `[0x02, 0x80, 0x00]` (DR-shaped), `[0x02, 0xC0, 0x00]` (DC-shaped), `[0x02, 0x70, 0x00]` (ER-shaped), `[0x02, 0x00, 0x00]` (all-zero) | 4 unrecognized high-nibble shapes never force-fit into CR/CC/DT | PASS |
| `test_iso8073_rfc905_table8_dr_code_not_modeled_holdout` | `[0x02, 0x80, 0x00]` | RFC 905 Table 8 clause 13.5: DR code `1000 0000` is not one of the 3 frozen `CotpTpduType` variants and must reject | PASS |

---

## Error-Path Demonstration

Key behavioral assertions verified:
- All 4 canonical unrecognized-shape vectors (DR `0x8_`, DC `0xC_`, ER `0x7_`, all-zero
  `0x0_`) return `None` — none are coerced into CR, CC, or DT.
- Independent RFC 905 Table 8 confirmation: Disconnect Request (DR, clause 13.5, fixed
  octet `1000 0000`) is deliberately not one of `CotpTpduType`'s 3 frozen variants
  (ADR-014 Decision 1) and is correctly rejected rather than approximated to the
  "closest" recognized type.

---

## Verdict

AC-185-007: **PASS** — The BC-2.20.011 unrecognized-shape test plus the independent
RFC-905 DR holdout both green; no force-fitting of unmodeled TPDU types observed.
