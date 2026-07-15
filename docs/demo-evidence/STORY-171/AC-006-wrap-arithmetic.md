# AC-171-006 — 15-Bit Modular Arithmetic: wrapping_sub & 0x7FFF

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking  
**AC:** AC-171-006  
**Traces to:** BC-2.19.024 invariant 1 (15-bit modular arithmetic)  
**Wave:** 80

---

## Acceptance Criterion

- Given `last_ns_dir = Some(32767)` and `current_ns = 1`
- When gap is computed: `1u16.wrapping_sub(32767) & 0x7FFF`
  = `(1u16.wrapping_sub(32767)) & 0x7FFF`
  = `32770u16 & 0x7FFF` = `2`
- Then gap = 2 ≤ 12 → no finding; state → `Some(1)`
- MUST use `wrapping_sub` with `& 0x7FFF` mask; plain subtraction would overflow

---

## Test Suite Execution — BC-2.19.024 ac171_006

Command:
```
cargo test --test iec104_analyzer_tests "ac171_006"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 2 tests
test story_171::test_BC_2_19_024_ac171_006_wrap_32767_to_0_gap_1_no_finding ... ok
test story_171::test_BC_2_19_024_ac171_006_wrap_32767_to_1_gap_2_no_finding ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 164 filtered out; finished in 0.00s
```

Result: **2/2 PASS**

---

## Test Coverage

### AC canonical vector (BC-2.19.024 invariant 1)

| Test Name | prev | current_ns | Gap Arithmetic | Gap | Finding | Result |
|-----------|------|------------|---------------|-----|---------|--------|
| `test_BC_2_19_024_ac171_006_wrap_32767_to_1_gap_2_no_finding` | 32767 | 1 | `1u16.wrapping_sub(32767) & 0x7FFF` = `32770 & 0x7FFF` = 2 | 2 ≤ 12 | None | PASS |
| `test_BC_2_19_024_ac171_006_wrap_32767_to_0_gap_1_no_finding` | 32767 | 0 | `0u16.wrapping_sub(32767) & 0x7FFF` = `32769 & 0x7FFF` = 1 | 1 ≤ 12 | None | PASS |

---

## Why wrapping_sub + & 0x7FFF

IEC 60870-5-104 sequence numbers are 15-bit (range 0–32767). The standard defines
the "wrap" as modulo 32768. A correct gap calculation must account for this:

```rust
let gap = current_ns.wrapping_sub(prev) & 0x7FFF;
```

Step-by-step for prev=32767, current=1:

```
current_ns.wrapping_sub(prev)
= 1u16.wrapping_sub(32767u16)   // u16 modulo 65536
= (1 - 32767) mod 65536
= -32766 mod 65536
= 32770

32770 & 0x7FFF
= 0x7FFE & 0x7FFF (32770 = 0x8002, wait let me recalculate)

Actually: 32770 in binary = 1000 0000 0000 0010
          0x7FFF in binary = 0111 1111 1111 1111
          AND result       = 0000 0000 0000 0010 = 2
```

Gap = 2 ≤ 12 → Path B → no finding (EC-005, correct wrap behavior).

Without the `& 0x7FFF` mask, the u16 wrapping_sub result 32770 would be > 12 and
would incorrectly emit a T1692.001 finding for a normal sequence wrap.

---

## Verdict

AC-171-006: **PASS** — Both wrap arithmetic tests green. `Some(32767) → current=1`
computes gap=2 (no finding). `Some(32767) → current=0` computes gap=1 (no finding).
15-bit modular arithmetic via `wrapping_sub & 0x7FFF` confirmed correct; no false
positive on normal sequence number rollover.
