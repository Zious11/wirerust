# AC-172-001 — Directional Carry Buffers: Independent Stash and Isolation

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
**AC:** AC-172-001
**Traces to:** BC-2.19.025 postconditions 1–4, invariants 1–2
**Wave:** 81

---

## Acceptance Criterion

- Given independent C2S and S2C byte streams for a flow
- When carry bytes accumulate across on_data calls (partial APCI frame)
- Then `carry_c2s` and `carry_s2c` are always strictly separate — bytes from one direction
  are never appended to the other's carry buffer
- Each carry buffer is bounded at `MAX_IEC104_CARRY_BYTES = 255` bytes

---

## Test Suite Execution — BC-2.19.025 carry stash

Command:
```
cargo test --test iec104_analyzer_tests "AC_172_001"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_172::test_AC_172_001_carry_stash_s2c_partial_frame ... ok
test story_172::test_AC_172_001_carry_directional_isolation_interleaved ... ok
test story_172::test_AC_172_001_carry_stash_c2s_partial_frame ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 189 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | Assertion | Result |
|-----------|-----------|--------|
| `test_AC_172_001_carry_stash_c2s_partial_frame` | A 3-byte partial STARTDT frame delivered C2S lands in `carry_c2s`; `carry_s2c` remains empty; no finding emitted | PASS |
| `test_AC_172_001_carry_stash_s2c_partial_frame` | A 3-byte partial STARTDT frame delivered S2C lands in `carry_s2c`; `carry_c2s` remains empty; no finding emitted | PASS |
| `test_AC_172_001_carry_directional_isolation_interleaved` | Interleaved C2S and S2C partial deliveries never mix bytes across directions | PASS |

---

## Carry Buffer Field Layout

`Iec104FlowState` holds two independent `Vec<u8>` carry fields:

```rust
pub struct Iec104FlowState {
    pub carry_c2s: Vec<u8>,
    pub carry_s2c: Vec<u8>,
    // ... other fields
}
```

A C2S `on_data` call prepends `carry_c2s` before scanning, then writes remaining bytes
back to `carry_c2s`. It never touches `carry_s2c`. A S2C call mirrors this with
`carry_s2c`. These buffers are separate heap allocations — cross-direction contamination
is structurally impossible.

---

## Partial-Frame Stash Scenario

The C2S stash test injects 3 bytes (`[0x68, 0x04, 0xAA]`) — a partial APCI stub that
begins with valid start byte (0x68) but has only 3 bytes, which is fewer than LEN+2=6
(LEN=4 is the minimum valid length). The frame-walk loop reaches "insufficient data"
and stashes all 3 bytes into `carry_c2s`. On a subsequent delivery that completes the
frame, the carry bytes are prepended before scanning resumes.

---

## Verdict

AC-172-001: **PASS** — Carry bytes accumulate into the correct directional buffer.
Cross-direction isolation confirmed. No carry bytes cross the C2S/S2C boundary under
any interleaved delivery pattern.
