# AC-172-007 — VP-045 Proptest Skeletons Compile: Carry Direction Isolation

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
**AC:** AC-172-007
**Traces to:** BC-2.19.025 invariant 1 (VP-045 proptest obligation)
**Wave:** 81

---

## Acceptance Criterion

- Given the `proptest_vp045_direction_isolation` and `proptest_vp045_independent_run_equivalence`
  harnesses anchored in this story
- When the proptest skeletons are compiled
- Then they compile without error and the bounded proptest runs pass
- Full extended proptest runs are executed in STORY-174
- Mirrors VP-033 (ENIP), VP-035 (DNP3), VP-037 (Modbus) pattern

---

## Test Suite Execution — VP-045 proptest skeletons

Command:
```
cargo test --test iec104_analyzer_tests "proptest_vp045"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 2 tests
test story_172::proptest_vp045_independent_run_equivalence ... ok
test story_172::proptest_vp045_direction_isolation ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 190 filtered out; finished in 0.04s
```

Result: **2/2 PASS**

---

## Test Coverage

| Harness | VP | Property Verified | Result |
|---------|----|-------------------|--------|
| `proptest_vp045_direction_isolation` | VP-045 | Interleaved C2S and S2C deliveries never mix carry bytes across directions; `carry_c2s` bytes come only from C2S path; `carry_s2c` bytes come only from S2C path | PASS (bounded runs) |
| `proptest_vp045_independent_run_equivalence` | VP-045 | Running C2S data alone (no S2C) produces the same C2S carry as running them interleaved — S2C data does not perturb C2S carry | PASS (bounded runs) |

---

## Proptest Harness Structure

Both harnesses use proptest's `prop::collection::vec(any::<u8>(), 0..256)` strategy to
generate arbitrary byte vectors for C2S and S2C independently. They exercise the
`Iec104Analyzer::on_data` API with interleaved calls, then assert the direction isolation
invariant. The harnesses are structured as proptest `#[test]` functions within the
`story_172` module, following the same skeleton pattern as VP-033 (ENIP carry isolation)
from STORY-130.

Full proptest settings (100K+ cases, reduced shrinking, CI gate) are wired in STORY-174.
The bounded variant used here runs the default proptest case count (256 cases) sufficient
to confirm compilation and basic property holds.

---

## Isolation Invariant (BC-2.19.025 Invariant 1)

The proptest harness verifies that for any interleaved sequence of C2S and S2C deliveries:

```
carry_c2s ⊂ bytes that entered via C2S on_data calls only
carry_s2c ⊂ bytes that entered via S2C on_data calls only
```

This is structurally enforced by `Iec104FlowState` having two separate `Vec<u8>` fields
(`carry_c2s`, `carry_s2c`) that are written exclusively by their respective direction arms
in `on_data`. The proptest provides an additional behavioral check across arbitrary input
combinations.

---

## Verdict

AC-172-007: **PASS** — Both VP-045 proptest skeleton harnesses compile and pass bounded
runs. Direction isolation invariant holds for all generated input combinations. Full
extended proptest execution deferred to STORY-174 per the standard proptest-skeleton
pattern.
