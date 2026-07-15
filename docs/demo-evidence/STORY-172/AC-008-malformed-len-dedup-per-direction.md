# AC-172-008 — Malformed-LEN Dedup Per Direction: Concrete Test Expectations

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
**AC:** AC-172-008
**Traces to:** BC-2.19.026 invariant 5; EC-006/007/008
**Wave:** 81

---

## Acceptance Criterion

- First malformed-LEN frame (valid 0x68, LEN=3 which is outside [4, 253]) in C2S direction:
  cursor advances 2 bytes; exactly ONE T0814 Anomaly/Possible/Medium emitted;
  `malformed_len_reported_c2s` set to true
- Second malformed-LEN frame in same C2S direction (flag already set):
  cursor advances 2 bytes; NO finding; flag unchanged
- First S2C malformed-LEN frame after C2S flag already set:
  cursor advances 2 bytes; ONE T0814 emitted independently for S2C;
  `malformed_len_reported_s2c` set to true; `malformed_len_reported_c2s` unchanged

---

## Test Suite Execution — BC-2.19.026 malformed-LEN dedup

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_026_malformed"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_172::test_BC_2_19_026_malformed_len_second_c2s ... ok
test story_172::test_BC_2_19_026_malformed_len_first_s2c_after_c2s ... ok
test story_172::test_BC_2_19_026_malformed_len_first_c2s ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 189 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | EC | Scenario | Expected | Result |
|-----------|-----|----------|----------|--------|
| `test_BC_2_19_026_malformed_len_first_c2s` | EC-006 | First malformed-LEN (valid 0x68, LEN=3) in C2S direction | ONE T0814 (Anomaly/Possible/Medium); `malformed_len_reported_c2s=true` | PASS |
| `test_BC_2_19_026_malformed_len_second_c2s` | EC-007 | Second malformed-LEN in same C2S direction; flag already set | NO finding; cursor advances 2; flag remains true | PASS |
| `test_BC_2_19_026_malformed_len_first_s2c_after_c2s` | EC-008 | First S2C malformed-LEN; C2S flag already set | ONE T0814 independently for S2C; `malformed_len_reported_s2c=true`; `malformed_len_reported_c2s` unchanged | PASS |

---

## Finding Properties (First Occurrence)

| Property | Value |
|----------|-------|
| MITRE Technique | T0814 "Denial of Service" |
| ThreatCategory | Anomaly |
| Verdict | Possible |
| Confidence | Medium |
| Cursor advance | +2 (skip APCI stub: start byte + LEN byte) |

---

## Dedup Flag Independence

`Iec104FlowState` holds four independent dedup flags:

```rust
pub struct Iec104FlowState {
    pub carry_overflow_reported_c2s: bool,   // carry-overflow dedup (AC-172-002)
    pub carry_overflow_reported_s2c: bool,   // carry-overflow dedup (AC-172-002)
    pub malformed_len_reported_c2s: bool,    // malformed-LEN dedup (AC-172-008)
    pub malformed_len_reported_s2c: bool,    // malformed-LEN dedup (AC-172-008)
}
```

The `malformed_len_reported_*` flags are intentionally separate from
`carry_overflow_reported_*` (BC-2.19.025 Invariant 4 / BC-2.19.026 Invariant 5) so
that neither anomaly class can suppress the other. A flow that has seen a carry overflow
(C2S) and a malformed-LEN (C2S) independently emits T0814 for each — not a single
combined T0814.

---

## Dedup State Machine (Per Direction)

```
Initial state: malformed_len_reported_{dir} = false

First malformed-LEN in direction:
  ├── advance cursor +2
  ├── emit ONE T0814 (Anomaly/Possible/Medium)
  └── set flag = true

Subsequent malformed-LEN in same direction (flag = true):
  ├── advance cursor +2
  └── (no finding, no flag change)
```

The flag is never reset within a flow lifetime (BC-2.19.026 invariant 5). It is cleared
implicitly when the flow is closed via `on_flow_close` (the entire `Iec104FlowState` is
dropped), so a reconnected flow starts fresh.

---

## Verdict

AC-172-008: **PASS** — All three per-direction dedup concrete test expectations confirmed.
First C2S malformed-LEN emits exactly one T0814. Second C2S emits nothing. First S2C
malformed-LEN emits independently. C2S flag untouched by S2C events.
