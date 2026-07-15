# AC-171-007 — Directional Isolation: C2S and S2C Tracked Independently

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking  
**AC:** AC-171-007  
**Traces to:** BC-2.19.023 postcondition 3 (direction parameter selects field); VP-045  
**Wave:** 80

---

## Acceptance Criterion

- Given different N(S) sequences in C2S and S2C directions
- When I-frames arrive alternating directions
- Then `last_ns_c2s` and `last_ns_s2c` are updated independently; no cross-direction mixing
- VP-045 proptest verifies this directional isolation property

---

## Test Suite Execution — BC-2.19.024 ac171_007

Command:
```
cargo test --test iec104_analyzer_tests "ac171_007"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_171::test_BC_2_19_024_ac171_007_c2s_call_updates_c2s_not_s2c ... ok
test story_171::test_BC_2_19_024_ac171_007_s2c_call_updates_s2c_not_c2s ... ok
test story_171::test_BC_2_19_024_ac171_007_interleaved_c2s_s2c_independent_baselines_and_gaps ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 163 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | Assertion | Result |
|-----------|-----------|--------|
| `test_BC_2_19_024_ac171_007_c2s_call_updates_c2s_not_s2c` | C2S-direction call advances `last_ns_c2s`; `last_ns_s2c` remains unchanged | PASS |
| `test_BC_2_19_024_ac171_007_s2c_call_updates_s2c_not_c2s` | S2C-direction call advances `last_ns_s2c`; `last_ns_c2s` remains unchanged | PASS |
| `test_BC_2_19_024_ac171_007_interleaved_c2s_s2c_independent_baselines_and_gaps` | Interleaved C2S/S2C sequence with gap > 12 in only one direction → finding in that direction only; other direction independent | PASS |

---

## Field Isolation

`Iec104FlowState` holds two independent fields:

```rust
pub struct Iec104FlowState {
    // ... (session, frame format fields from STORY-168)
    pub last_ns_c2s: Option<u16>,   // client-to-server N(S) baseline
    pub last_ns_s2c: Option<u16>,   // server-to-client N(S) baseline
}
```

The gap check is parameterized by direction — a C2S frame reads/writes `last_ns_c2s`
only; an S2C frame reads/writes `last_ns_s2c` only.

This means:
- A large N(S) jump in C2S direction does NOT trigger a finding in S2C direction
- An attacker who injects S2C frames does not disturb the C2S sequence tracking
- Asymmetric flows (e.g., heartbeat-only S2C) do not cross-contaminate C2S gap checks

---

## Interleaved Sequence Scenario

The interleaved test (`ac171_007_interleaved`) exercises the most important scenario:

```
Frame 1: C2S N(S)=0    → C2S: None → Some(0), S2C unchanged
Frame 2: S2C N(S)=0    → S2C: None → Some(0), C2S unchanged
Frame 3: C2S N(S)=1    → C2S: gap=1 ≤ 12, no finding; S2C unchanged
Frame 4: S2C N(S)=100  → S2C: gap=100 > 12, T1692.001 Possible; C2S unchanged
```

Finding is emitted only for the S2C direction; C2S state remains `Some(1)`.

---

## Verdict

AC-171-007: **PASS** — All 3 directional isolation tests green. C2S and S2C fields
confirmed independent. Interleaved sequence with gap > 12 in S2C only → finding in S2C
only; C2S unaffected. No cross-direction contamination.
