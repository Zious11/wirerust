# AC-172-006 — on_flow_close Removes State and Discards Carry Bytes

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
**AC:** AC-172-006
**Traces to:** BC-2.19.027 postconditions 1–4, invariants 1–2
**Wave:** 81

---

## Acceptance Criterion

- Given a flow with active `Iec104FlowState` (possibly with non-empty carry buffers)
- When `on_flow_close(flow_key)` is called
- Then `Iec104FlowState` for that flow is removed from the state map
- `carry_c2s` and `carry_s2c` are dropped (memory freed)
- No finding is emitted for normal flow close
- Calling `on_flow_close` for an already-removed flow_key is a no-op (no panic)

---

## Test Suite Execution — BC-2.19.027 on_flow_close

Command:
```
cargo test --test iec104_analyzer_tests "BC_2_19_027"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 3 tests
test story_172::test_BC_2_19_027_ec_011_close_unknown_flow_key_no_panic ... ok
test story_172::test_BC_2_19_027_ec_010_close_with_carry_no_finding ... ok
test story_172::test_BC_2_19_027_on_flow_close_removes_state ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 189 filtered out; finished in 0.00s
```

Command (reopen fresh state):
```
cargo test --test iec104_analyzer_tests "AC_172_006"
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 1 test
test story_172::test_AC_172_006_reopen_flow_yields_fresh_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 191 filtered out; finished in 0.00s
```

Result: **4/4 PASS**

---

## Test Coverage

| Test Name | Scenario | Assertion | Result |
|-----------|----------|-----------|--------|
| `test_BC_2_19_027_on_flow_close_removes_state` | Flow with active state (session_started, non-empty carry); `on_flow_close` called | Flow key absent from `flows` map afterward; no finding emitted | PASS |
| `test_BC_2_19_027_ec_010_close_with_carry_no_finding` | Flow with non-empty carry buffers; `on_flow_close` called | Carry silently discarded; no finding; EC-010 | PASS |
| `test_BC_2_19_027_ec_011_close_unknown_flow_key_no_panic` | Unknown flow_key not in `flows` map; `on_flow_close` called | No panic; no-op; EC-011 | PASS |
| `test_AC_172_006_reopen_flow_yields_fresh_state` | Flow closed; same flow_key used in subsequent `on_data` | New `Iec104FlowState` created with all default values (`malformed_len_reported_c2s=false`, `session_started=false`, empty carries) | PASS |

---

## Lifecycle Sequence

```
on_data(flow_key, ...) ──► creates Iec104FlowState (lazy init on first call)
                              carry_c2s, carry_s2c accumulated
on_flow_close(flow_key) ──► flows.remove(&flow_key)
                              Iec104FlowState dropped; carry Vecs freed
on_data(flow_key, ...) ──► new Iec104FlowState created (fresh; no memory of prior flow)
```

The reopen-fresh test confirms that a flow_key reused after close behaves identically to
a brand-new flow: `malformed_len_reported_c2s`, `carry_overflow_reported_c2s`, and
`session_started` all start at their default values. Prior dedup flags from the old
connection do not carry over.

---

## No-Panic for Unknown Key (EC-011)

`on_flow_close` is implemented as `self.flows.remove(&flow_key)` which returns an
`Option` that is silently discarded. If the key is not present, `HashMap::remove`
returns `None` and is a no-op — no panic, no side effect.

---

## Verdict

AC-172-006: **PASS** — State removal confirmed. Non-empty carry silently discarded on
close. Unknown flow_key is a no-op. Reopen yields fresh default state with no
contamination from the previous connection's dedup flags or carry bytes.
