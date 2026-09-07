# AC-185-006 — `parse_cotp_header` Recognizes DT With Empty Payload — `protocol_id` Is None

**Story:** STORY-185: S7comm COTP TPDU-Type Parser
**AC:** AC-185-006
**Traces to:** BC-2.20.010 postconditions 1–2
**Wave:** 88

---

## Acceptance Criterion

- Given `tpkt_payload[1] & 0xF0 == 0xF0` and `tpkt_payload.len() == payload_offset`
  exactly (no trailing byte)
- When `parse_cotp_header(tpkt_payload)` is called
- Then returns `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: None,
  payload_offset })`; no out-of-bounds index at `tpkt_payload[payload_offset]` (traces
  to BC-2.20.010 postcondition 2)

---

## Test Suite Execution

Command:
```
cargo test --test iso_on_tcp_tests BC_2_20_010
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 1 test
test story_185::test_BC_2_20_010_dt_empty_payload_protocol_id_none ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

Result: **1/1 PASS**

---

## Test Coverage

| Test Name | Canonical Vector | Condition Exercised | Result |
|-----------|-------------------|----------------------|--------|
| `test_BC_2_20_010_dt_empty_payload_protocol_id_none` | `[0x02, 0xF0, 0x80]` | Minimal DT TPDU (LI=2), zero trailing payload bytes (`len() == payload_offset`) -> `DataTransfer`, `protocol_id: None`, `payload_offset: 3` | PASS |

---

## Boundary/Error-Adjacent Demonstration

This AC is the direct boundary companion to AC-185-005: identical DT TPDU-code and LI,
differing only in whether a trailing byte is present. Key assertion:
- `tpkt_payload.len() == payload_offset` exactly (no trailing byte, EC-001's
  "legitimately empty payload" for DT) -> `protocol_id: None`, with no out-of-bounds
  index attempted at `tpkt_payload[payload_offset]` (which does not exist in this
  input). This is the safety-critical half of the DT-recognition pair — it proves the
  implementation checks `len() > payload_offset` before indexing rather than indexing
  unconditionally and catching a panic.

---

## Verdict

AC-185-006: **PASS** — BC-2.20.010's single canonical test green; the empty-DT-payload
boundary (paired with AC-185-005's non-empty case) verified with no out-of-bounds access.
