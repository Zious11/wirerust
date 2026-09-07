# AC-185-004 — `parse_cotp_header` Recognizes Connect Confirm (CC) TPDU

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-004
**Traces to:** BC-2.20.008 postconditions 1–3
**Wave:** 88

---

## Acceptance Criterion

- Given `tpkt_payload[1] & 0xF0 == 0xD0` and the LI-truncation check has passed
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: ConnectConfirm, protocol_id: None,
  payload_offset })` with `payload_offset == 1 + LI` (traces to BC-2.20.008
  postcondition 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_008
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 2 tests
test story_185::test_BC_2_20_008_connect_confirm_nonzero_low_nibble_still_recognized ... ok
test story_185::test_BC_2_20_008_connect_confirm_recognized ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.00s
```

Result: **2/2 PASS**

The independent RFC-905 Table 8 holdout `test_iso8073_rfc905_table8_cr_cc_low_nibble_is_free_holdout`
also exercises CC recognition (code `0xDA`, low nibble `0xA`) in the same assertion that
covers CR; its test-count contribution is attributed to `AC-003-connect-request-recognition.md`
to avoid double-counting in the story-level 22-test tally, but its CC-half assertion is
reproduced below for completeness.

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_008_connect_confirm_recognized` | `[0x06, 0xD0, 0x00, 0x01, 0x00, 0x00, 0x00]` | Minimal CC TPDU (LI=6) -> `ConnectConfirm`, `protocol_id: None`, `payload_offset: 7` | PASS |
| `test_BC_2_20_008_connect_confirm_nonzero_low_nibble_still_recognized` | `[0x06, 0xD1, ...]` | EC-002: non-zero low nibble (`0xD1`) does not affect CC recognition | PASS |
| `test_iso8073_rfc905_table8_cr_cc_low_nibble_is_free_holdout` (CC-half; counted under AC-003) | `[0x06, 0xDA, 0x00, 0x01, 0x00, 0x00, 0x00]` | RFC 905 Table 8: CC code is `1101 xxxx`, low nibble `0xA` must not prevent recognition | PASS |

---

## Success-Path Demonstration

Key behavioral assertions verified:
- Minimal CC TPDU (`LI=6`, code `0xD0`) -> `Some(CotpHeader { ConnectConfirm, None, 7 })`.
- Only the high nibble (`& 0xF0`) discriminates CC — the low nibble is free, confirmed
  with both `0xD1` (BC vector) and `0xDA` (independent RFC 905 Table 8 holdout).
- `protocol_id` is `None` for CC, mirroring CR (no upper-layer payload has been
  established yet at the connect-confirm stage).

---

## Verdict

AC-185-004: **PASS** — Both BC-2.20.008 tests green; CC high-nibble discrimination
cross-checked against the independent RFC 905 Table 8 holdout (attributed to AC-003 in
the story-level tally).
