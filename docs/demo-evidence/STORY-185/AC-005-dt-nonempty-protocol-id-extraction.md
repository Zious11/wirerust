# AC-185-005 — `parse_cotp_header` Recognizes DT With Non-Empty Payload and Extracts `protocol_id`

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-005
**Traces to:** BC-2.20.009 postconditions 1–3, edge case EC-004
**Wave:** 88

---

## Acceptance Criterion

- Given `tpkt_payload[1] & 0xF0 == 0xF0` and `tpkt_payload.len() > payload_offset` where
  `payload_offset = 1 + LI`
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id:
  Some(tpkt_payload[payload_offset]), payload_offset })` (traces to BC-2.20.009
  postcondition 1)
- `protocol_id` is the trailing byte verbatim for every `u8` value (`0x32`, `0x72`, or
  any other byte) — never coerced or force-fit (traces to BC-2.20.009 edge case EC-004)

**Literal-avoidance note:** per this story's `test_BC_2_20_012_static_regression_guard_no_hardcoded_protocol_literals`
guard, `src/analyzer/iso_on_tcp.rs` must contain zero occurrences of the literals
`0x32`/`0x72`. This evidence file follows the same discipline as the test source and
does not reproduce those two byte values as literal tokens; the tests below use `0x01`
and other bytes to demonstrate identical, uninterpreted extraction behavior.

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_009
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 3 tests
test story_185::test_BC_2_20_009_dt_nonempty_payload_extracts_protocol_id ... ok
test story_185::test_BC_2_20_009_dt_protocol_id_is_first_trailing_byte_only ... ok
test story_185::test_BC_2_20_009_dt_protocol_id_extracted_for_boundary_byte_values ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 0.00s
```

Plus 1 independent RFC-905-derived holdout vector (DF-CANONICAL-FRAME-HOLDOUT-001):

```
cargo test --test iso_on_tcp_tests test_iso8073_rfc905_s13_7_1_dt_class0_normal_format_holdout
```
```
running 1 test
test story_185::test_iso8073_rfc905_s13_7_1_dt_class0_normal_format_holdout ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

Result: **4/4 PASS** (3 BC-tagged tests + 1 RFC-905 holdout)

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_009_dt_nonempty_payload_extracts_protocol_id` | `[0x02, 0xF0, 0x80, 0x01]` | Minimal DT TPDU (LI=2) with 1 trailing byte -> `DataTransfer`, `protocol_id: Some(0x01)`, `payload_offset: 3` | PASS |
| `test_BC_2_20_009_dt_protocol_id_is_first_trailing_byte_only` | `[0x02, 0xF0, 0x80, 0x01, 0x02, 0x03, 0x04]` | `protocol_id` is exactly `tpkt_payload[payload_offset]` — the *first* trailing byte only, never a later one | PASS |
| `test_BC_2_20_009_dt_protocol_id_extracted_for_boundary_byte_values` | `[..., 0x00]` and `[..., 0xFF]` | Both `u8` extremes (0x00 min, 0xFF max) extracted verbatim | PASS |
| `test_iso8073_rfc905_s13_7_1_dt_class0_normal_format_holdout` | `[0x02, 0xF0, 0xC0, 0x99]` | RFC 905 §13.7.1 format (a): class-0 DT fixed part is exactly 2 octets, so `payload_offset == 1 + LI == 3`; independent TPDU-NR/EOT and user-data byte values | PASS |

---

## Success-Path Demonstration

Key behavioral assertions verified:
- Minimal DT TPDU with non-empty payload -> `Some(CotpHeader { DataTransfer,
  Some(<trailing byte>), payload_offset })`.
- Only the byte at `payload_offset` is ever inspected — additional trailing bytes past
  it are ignored.
- Verbatim, uninterpreted extraction confirmed at both `u8` boundary values (`0x00`,
  `0xFF`) and at the independent RFC-905 holdout's `0x99` value — no branch inside
  `parse_cotp_header` ever special-cases a specific byte.
- `payload_offset == 1 + LI` arithmetic reconfirmed against RFC 905 §13.7.1's own DT
  fixed-part-length definition, independent of any BC-2.20.009 vector byte pattern.

---

## Verdict

AC-185-005: **PASS** — All 3 BC-2.20.009 tests plus 1 independent RFC-905 holdout
green; verbatim protocol-ID extraction verified across boundary values without any
value-specific interpretation inside the parser.
