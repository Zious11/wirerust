# RETRANSMIT-NS-FALSEPOS-001 — TCP Retransmission False-Positive (Intentional, Fail-Closed)

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking  
**Edge Case:** RETRANSMIT-NS-FALSEPOS-001  
**Traces to:** BC-2.19.024 invariant 3 (fail-closed per INV-3); STORY-171 Edge Cases table  
**Wave:** 80

---

## Edge Case Definition

**RETRANSMIT-NS-FALSEPOS-001:** TCP retransmissions that re-deliver I-frames with a lower
N(S) than the last seen value will produce a false-positive T1692.001 Possible finding.

The analyzer cannot distinguish TCP retransmits from adversarial replays. This is the
expected fail-closed behavior per INV-3 (deny-by-default). The risk is documented here
for operator awareness; future mitigation via TCP deduplication is deferred.

---

## Test Suite Execution

Command:
```
cargo test --test iec104_analyzer_tests RETRANSMIT
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 1 test
test story_171::test_RETRANSMIT_NS_FALSEPOS_001_backwards_ns_yields_large_gap_emits_t1692_001_finding ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 165 filtered out; finished in 0.00s
```

Result: **1/1 PASS** (intentional false-positive behavior confirmed)

---

## Test Coverage

| Test Name | Scenario | Expected Behavior | Result |
|-----------|----------|-------------------|--------|
| `test_RETRANSMIT_NS_FALSEPOS_001_backwards_ns_yields_large_gap_emits_t1692_001_finding` | prev=100 (last seen), current=5 (retransmitted older N(S)) → wrapping gap = `5u16.wrapping_sub(100) & 0x7FFF` = 32677 > 12 | T1692.001 Possible emitted (intended false-positive) | PASS |

---

## Fail-Closed Rationale (INV-3)

The gap formula `current_ns.wrapping_sub(prev) & 0x7FFF` treats a backwards N(S) as
a very large gap (approaching 32767). This is correct under the fail-closed principle:

- **True adversarial replay / desync attack:** backwards N(S) with large gap → T1692.001
  Possible → correct detection
- **TCP retransmission delivering older I-frame:** same backwards N(S) → T1692.001
  Possible → false positive, but operator is alerted

The analyzer accepts the false-positive rate in exchange for zero false-negative risk on
genuine desync attacks. Suppressing the finding would require TCP deduplication upstream
(out of scope for STORY-171).

---

## EC-007 Summary

| Edge Case | Scenario | Behavior | Policy |
|-----------|----------|----------|--------|
| EC-007 (RETRANSMIT-NS-FALSEPOS-001) | TCP retransmission delivers I-frame with lower N(S) | T1692.001 Possible emitted (false positive) | Intentional fail-closed (INV-3). Not suppressed. |

**This is not a bug.** The test documents and confirms the intended behavior.

---

## Verdict

RETRANSMIT-NS-FALSEPOS-001: **PASS** — Backwards N(S) yields large wrapping gap and
correctly emits T1692.001 Possible per fail-closed policy. Behavior is intentional and
documented per STORY-171 Edge Case EC-007.
