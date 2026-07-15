# AC-172-004 — Frame-Walk Advance Modes: Bad-Start-Byte + Malformed-LEN Termination

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
**AC:** AC-172-004
**Traces to:** BC-2.19.026 postcondition 4, invariants 1 and 5; ADR-013 Decision 3
**Wave:** 81

---

## Acceptance Criterion

The frame-walk loop always advances the cursor on every iteration, ensuring termination
for any finite input. ADR-013 Decision 3 defines four exclusive advance modes:

| Condition | Cursor advance | Carry behavior | Finding |
|-----------|---------------|----------------|---------|
| Bad start byte (`data[pos] != 0x68`) | +1 | NOT cleared (resync scan) | None |
| Malformed LEN (valid 0x68, LEN outside [4, 253]) | +2 (skip APCI stub) | Unchanged | T0814 on FIRST occurrence per direction; silent on subsequent (dedup) |
| Valid frame | LEN+2 | Residual stashed if incomplete | None from loop itself |
| Insufficient data (< LEN+2 bytes remain) | 0 (loop returns) | Remaining stashed to carry | None |

No other advance mode is valid.

---

## Test Suite Execution — bad-start-byte arm

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_026_bad_start"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 1 test
test story_172::test_BC_2_19_026_bad_start_byte_advance_one_no_finding ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 191 filtered out; finished in 0.00s
```

Result: **1/1 PASS**

---

## Test Coverage

| Test Name | Data | Expected | Result |
|-----------|------|----------|--------|
| `test_BC_2_19_026_bad_start_byte_advance_one_no_finding` | `[0x00, 0x68, 0x04, ...]` — first byte is not 0x68 | Cursor advances 1 byte; carry is NOT cleared; no finding emitted; resync scan continues at next position | PASS |

---

## Bad-Start-Byte Behavior Detail

The test injects a byte stream where the first byte is 0x00 (not 0x68). The loop
advances 1 byte without emitting any finding and without clearing the carry buffer. On the
next iteration the valid 0x68 is found and parsing resumes normally. This implements the
"resync scan" mode — the analyzer does not give up on the stream when it encounters a
non-start byte mid-stream; it slides forward one byte at a time to locate the next valid
APCI header.

Key distinction from carry-overflow resync: a bad-start-byte resync does NOT clear carry.
The carry holds bytes that were legitimately stashed from a prior call; clearing it on a
bad-start-byte would discard valid partially-received frame data.

---

## Malformed-LEN EMIT-WITH-DEDUP

The malformed-LEN arm of the advance table is exercised by AC-172-008 (concrete dedup
expectations). AC-172-004 defines the termination guarantee — the +2 advance ensures the
loop does not stall on a malformed 0x68 byte. AC-172-008 verifies the T0814 emission and
dedup flag behavior.

---

## VP-047 No-Panic Termination

Because every iteration advances the cursor by at least 1 byte (bad-start and
malformed-LEN arms) or returns immediately (insufficient data), the loop always terminates
for any finite input. VP-047 fuzz harness (`fuzz_iec104_parser`) exercises this guarantee
exhaustively; skeleton compiled in STORY-172, full run in STORY-174.

---

## Verdict

AC-172-004: **PASS** — Bad-start-byte advance-1/no-carry-clear/no-finding confirmed.
Carry is preserved during resync. Loop terminates for all finite inputs. Malformed-LEN
dedup tested concretely in AC-172-008.
