# AC-171-004 — Subsequent Frame with Gap ≤ k=12 Updates State with No Finding

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking  
**AC:** AC-171-004  
**Traces to:** BC-2.19.024 postcondition Path B  
**Wave:** 80

---

## Acceptance Criterion

- Given `last_ns_dir` is `Some(prev)` and `(current_ns.wrapping_sub(prev) & 0x7FFF) <= 12`
- When the next I-frame is processed
- Then the directional field is updated to `Some(current_ns)` and no finding is emitted
- Test vectors: prev=5000, current=5001 (gap=1) → no finding; gap=12 exactly → no finding

---

## Test Suite Execution — BC-2.19.024 Path B

Command:
```
cargo test --test iec104_analyzer_tests "path_b"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_171::test_BC_2_19_024_path_b_gap_0_same_ns_no_finding ... ok
test story_171::test_BC_2_19_024_path_b_gap_1_no_finding_state_updates_to_current_ns ... ok
test story_171::test_BC_2_19_024_path_b_gap_12_exactly_k_boundary_no_finding ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 163 filtered out; finished in 0.00s
```

Result: **3/3 PASS**

---

## Test Coverage

| Test Name | prev | current_ns | gap (wrapping) | Expected Findings | Result |
|-----------|------|------------|-----------------|-------------------|--------|
| `test_BC_2_19_024_path_b_gap_0_same_ns_no_finding` | prev | same prev | 0 | 0 findings | PASS |
| `test_BC_2_19_024_path_b_gap_1_no_finding_state_updates_to_current_ns` | 5000 | 5001 | 1 | 0 findings; state → Some(5001) | PASS |
| `test_BC_2_19_024_path_b_gap_12_exactly_k_boundary_no_finding` | prev | prev+12 | 12 | 0 findings (≤ k, boundary) | PASS |

---

## k=12 Window Boundary

The k=12 window is fixed for MVP (ADR-013 Decision 6). The gap check is strictly `> 12`:
gaps of 0 through 12 inclusive are all benign and produce no finding.

Gap=12 (EC-003 boundary case) is covered explicitly by
`test_BC_2_19_024_path_b_gap_12_exactly_k_boundary_no_finding`, confirming the ≤ boundary
condition is implemented as `>` (not `>=`).

---

## Verdict

AC-171-004: **PASS** — All 3 Path-B tests green. Gaps 0, 1, and 12 all produce no finding.
State correctly updates to `Some(current_ns)` on each call.
