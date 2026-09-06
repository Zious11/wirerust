# AC-181-002: All Existing ENIP Tests Pass Unchanged

**AC:** AC-181-002  
**Story:** STORY-181 (SEC-001 ENIP unsafe split-borrow refactor)  
**Date:** 2026-07-24  
**Branch:** feature/STORY-181-enip-sec001-split-borrow

---

## Verdict: PASS — 184/184 ENIP tests, zero failures

---

## ENIP Integration Test Suite

Command:
```
cargo test --test enip_analyzer_tests
```

Output (tail):
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/enip_analyzer_tests.rs (target/debug/deps/enip_analyzer_tests-831d4c1100defc5d)

test result: ok. 184 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

184 tests pass, 0 failures.

---

## Carry-Path Regression Witnesses (BC-2.17.016)

The three tests named in the story AC are confirmed passing:

### test_carry_buffer_partial_header
```
cargo test --test enip_analyzer_tests "test_carry_buffer_partial_header"

running 1 test
test frame_walk::test_carry_buffer_partial_header ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 183 filtered out; finished in 0.00s
```

### test_carry_buffer_two_frames_one_segment
```
cargo test --test enip_analyzer_tests "test_carry_buffer_two_frames_one_segment"

running 1 test
test frame_walk::test_carry_buffer_two_frames_one_segment ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 183 filtered out; finished in 0.00s
```

### test_ec_x1_cross_direction_no_splice
```
cargo test --test enip_analyzer_tests "test_ec_x1_cross_direction_no_splice"

running 1 test
test direction_and_clock::test_ec_x1_cross_direction_no_splice ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 183 filtered out; finished in 0.01s
```

All three carry-path tests pass. BC-2.17.016 regression guard is satisfied.

---

## Full Suite Summary (cargo test --all-targets)

Command:
```
cargo test --all-targets
```

Selected result lines (all targets, zero failures):
```
test result: ok. 184 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s  (enip)
test result: ok. 248 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s  (iec104)
test result: ok. 229 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s  (dnp3)
test result: ok. 160 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.94s (tls)
```

Every test-result line in the full suite reads `0 failed`. No test assertion was modified.
