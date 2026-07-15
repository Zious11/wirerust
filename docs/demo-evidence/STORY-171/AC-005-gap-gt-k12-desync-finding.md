# AC-171-005 — Subsequent Frame with Gap > k=12 Emits T1692.001 Possible

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking  
**AC:** AC-171-005  
**Traces to:** BC-2.19.024 postcondition Path C; invariant 1 (fail-closed)  
**Wave:** 80

---

## Acceptance Criterion

- Given `last_ns_dir` is `Some(prev)` and `(current_ns.wrapping_sub(prev) & 0x7FFF) > 12`
- When the next I-frame is processed
- Then T1692.001 "Unauthorized Message: Command Message" finding is emitted with confidence Possible
- The finding message includes: current N(S), previous N(S) (prev), and the gap value
- The directional field is updated to `Some(current_ns)`
- Test vectors: prev=5001, current=5020 (gap=19) → T1692.001 Possible

---

## Test Suite Execution — BC-2.19.024 Path C

Command:
```
cargo test --test iec104_analyzer_tests "path_c"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 5 tests
test story_171::test_BC_2_19_024_path_c_canonical_table_row8_prev_100_current_114_gap_14_emits_finding ... ok
test story_171::test_BC_2_19_024_path_c_ec005_gap_32767_massive_jump_emits_t1692_001_possible ... ok
test story_171::test_BC_2_19_024_path_c_gap_13_k_plus_1_emits_t1692_001_possible ... ok
test story_171::test_BC_2_19_024_path_c_gap_19_canonical_vector_prev_5001_current_5020_emits_t1692_001 ... ok
test story_171::test_BC_2_19_024_path_c_state_updates_to_current_ns_after_finding_emitted ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.00s
```

Result: **5/5 PASS**

---

## Test Coverage

| Test Name | prev | current_ns | gap | Finding | Verdict | Result |
|-----------|------|------------|-----|---------|---------|--------|
| `test_BC_2_19_024_path_c_gap_13_k_plus_1_emits_t1692_001_possible` | prev | prev+13 | 13 (k+1) | T1692.001 | Possible | PASS |
| `test_BC_2_19_024_path_c_gap_19_canonical_vector_prev_5001_current_5020_emits_t1692_001` | 5001 | 5020 | 19 | T1692.001 | Possible | PASS |
| `test_BC_2_19_024_path_c_canonical_table_row8_prev_100_current_114_gap_14_emits_finding` | 100 | 114 | 14 | T1692.001 | Possible | PASS |
| `test_BC_2_19_024_path_c_ec005_gap_32767_massive_jump_emits_t1692_001_possible` | prev | prev+32767 (15-bit) | 32767 (EC-006) | T1692.001 | Possible | PASS |
| `test_BC_2_19_024_path_c_state_updates_to_current_ns_after_finding_emitted` | prev | prev+gap>12 | >12 | state → Some(current_ns) after finding emitted | — | PASS |

---

## Finding Properties (BC-2.19.024 Path C)

The emitted finding carries:

| Property | Value |
|----------|-------|
| MITRE Technique | T1692.001 (Unauthorized Message: Command Message) |
| Verdict | Possible |
| Category | Impact |
| Evidence | current N(S), previous N(S), gap value |

---

## Boundary Between Path B and Path C

| gap value | Path | Finding |
|-----------|------|---------|
| 0 | B | None |
| 1 | B | None |
| 12 | B | None (≤ k — boundary) |
| 13 | C | T1692.001 Possible (k+1 — just over boundary) |
| 19 | C | T1692.001 Possible (BC canonical vector) |
| 32767 | C | T1692.001 Possible (EC-006 massive jump) |

---

## Verdict

AC-171-005: **PASS** — All 5 Path-C tests green. Gap=13 (k+1 boundary), gap=19 (canonical
vector), gap=14 (BC table row 8), and gap=32767 (EC-006 massive jump) all emit T1692.001
Possible. State correctly advances to `Some(current_ns)` after finding emission.
