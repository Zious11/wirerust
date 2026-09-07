# AC-185-003 — `parse_cotp_header` Recognizes Connect Request (CR) TPDU

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-003
**Traces to:** BC-2.20.007 postconditions 1–3
**Wave:** 88

---

## Acceptance Criterion

- Given `tpkt_payload[1] & 0xF0 == 0xE0` and the LI-truncation check has passed
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: ConnectRequest, protocol_id: None,
  payload_offset })` where `payload_offset == 1 + LI` (traces to BC-2.20.007
  postcondition 2)
- `protocol_id` is unconditionally `None` for CR, regardless of any bytes present beyond
  the fixed CR header (traces to BC-2.20.007 postcondition 3)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_007
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 3 tests
test story_185::test_BC_2_20_007_connect_request_protocol_id_none_even_with_trailing_bytes ... ok
test story_185::test_BC_2_20_007_connect_request_nonzero_low_nibble_still_recognized ... ok
test story_185::test_BC_2_20_007_connect_request_recognized ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 0.00s
```

Plus 2 independent RFC-905-derived holdout vectors (DF-CANONICAL-FRAME-HOLDOUT-001),
authored directly from the fetched ISO 8073 (RFC 905) specification text rather than
this project's own BC-2.20.007 vector text:

```
cargo test --test iso_on_tcp_tests test_iso8073_rfc905_table8_cr_cc_low_nibble_is_free_holdout
```
```
running 1 test
test story_185::test_iso8073_rfc905_table8_cr_cc_low_nibble_is_free_holdout ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

```
cargo test --test iso_on_tcp_tests test_iso8073_rfc905_s13_2_1_li_excludes_itself_holdout
```
```
running 1 test
test story_185::test_iso8073_rfc905_s13_2_1_li_excludes_itself_holdout ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

Result: **5/5 PASS** (3 BC-tagged tests + 2 RFC-905 holdouts)

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_007_connect_request_recognized` | `[0x06, 0xE0, 0x00, 0x00, 0x00, 0x01, 0x00]` | Minimal CR TPDU (LI=6) -> `ConnectRequest`, `protocol_id: None`, `payload_offset: 7` | PASS |
| `test_BC_2_20_007_connect_request_nonzero_low_nibble_still_recognized` | `[0x06, 0xE1, ...]` | EC-002: non-zero low nibble (`0xE1`) does not affect CR recognition — high-nibble-only discrimination | PASS |
| `test_BC_2_20_007_connect_request_protocol_id_none_even_with_trailing_bytes` | `[0x06, 0xE0, ..., 0xAB]` | `protocol_id` stays `None` for CR even with a trailing byte present beyond the fixed CR header | PASS |
| `test_iso8073_rfc905_table8_cr_cc_low_nibble_is_free_holdout` | `[0x06, 0xEF, ...]` (independent RFC 905 Table 8 vector) | Confirms — independently of this project's own BC citation — that CR's code is `1110 xxxx`, low nibble free (also exercises the CC half of Table 8; see `AC-004-connect-confirm-recognition.md`) | PASS |
| `test_iso8073_rfc905_s13_2_1_li_excludes_itself_holdout` | `[0x06, 0xE3, 0xAA, 0xBB, 0xCC, 0xDD, 0x00]` | RFC 905 §13.2.1: LI counts header octets *after* itself, so `payload_offset == 1 + LI == 7`, confirmed with DST-REF/SRC-REF values distinct from the BC-2.20.007 canonical vector | PASS |

---

## Success-Path Demonstration

Key behavioral assertions verified:
- Minimal CR TPDU (`LI=6`, code `0xE0`) -> `Some(CotpHeader { ConnectRequest, None, 7 })`.
- Only the high nibble (`& 0xF0`) discriminates TPDU type — the low nibble is free for
  CDT (credit) signaling per RFC 905 Table 8, confirmed with both `0xE1` (BC vector) and
  `0xEF` (independent RFC 905 holdout).
- `protocol_id` is unconditionally `None` for CR — no upper-layer payload is inspected,
  even when trailing bytes are present.
- `payload_offset == 1 + LI` arithmetic independently reconfirmed against RFC 905
  §13.2.1's own definition of the Length Indicator, using DST-REF/SRC-REF byte values
  never used by any BC-2.20.007 vector.

---

## Verdict

AC-185-003: **PASS** — All 3 BC-2.20.007 tests plus 2 independent RFC-905 holdouts
green; high-nibble-only discrimination and `payload_offset` arithmetic verified from an
independent specification source.
