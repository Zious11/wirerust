# AC-173-008: StreamDispatcher iec104 Field + on_data/on_flow_close Wiring

**AC:** AC-173-008
**BC:** BC-2.05.012 invariant 1 (VP-004 oracle), ADR-013 Decision 9 steps 4–5
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

`StreamDispatcher` gains the infrastructure to forward port-2404 flow data to `Iec104Analyzer`:

- `iec104: Option<Iec104Analyzer>` field added to `StreamDispatcher`.
- `new()` extended from 5 to 6 parameters (`iec104: Option<Iec104Analyzer>` as the last arg).
- `set_iec104_analyzer(&mut self, analyzer: Iec104Analyzer)` setter added.
- Early-exit guard in `on_data` extended with `&& self.iec104.is_none()` so a `--iec104`-only
  invocation does not silently discard all data (ADR-013 Decision 9 step 4).
- `on_data` gains a `DispatchTarget::Iec104` match arm calling
  `iec104.on_data(flow_key.clone(), data, timestamp, direction)`.
- `on_flow_close` gains a `DispatchTarget::Iec104` match arm calling
  `iec104.on_flow_close(flow_key.clone())`.

---

## Source confirmation

`src/dispatcher.rs` — `iec104` field (line 102):
```rust
/// port-2404 flows that do not match content rules 1–2 or port rules 3–7.
iec104: Option<Iec104Analyzer>,
```

`src/dispatcher.rs` — `new()` signature (line 131):
```rust
iec104: Option<Iec104Analyzer>,
```

`src/dispatcher.rs` — `on_data` Iec104 arm (lines 464–470):
```rust
DispatchTarget::Iec104 => {
    // BC-2.05.012 §P2 / AC-173-008: forward port-2404 flow data to Iec104Analyzer.
    if let Some(ref mut iec104) = self.iec104 {
        iec104.on_data(flow_key.clone(), data, timestamp, direction);
    }
}
```

---

## Test output

Command:
```
cargo test --test dispatcher_tests story_173
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.72s
     Running tests/dispatcher_tests.rs

running 5 tests
test story_173::test_iec104_disabled_port_2404_no_panic ... ok
test story_173::test_iec104_only_guard_unclassified_flows_counted ... ok
test story_173::test_BC_2_05_012_early_exit_guard_includes_iec104 ... ok
test story_173::test_iec104_only_dispatcher_data_reaches_analyzer ... ok
test story_173::test_iec104_only_dispatcher_stopdt_produces_t0881 ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 56 filtered out; finished in 0.00s
```

---

## Per-test behavior

**`test_iec104_only_dispatcher_data_reaches_analyzer` (primary wiring test — STARTDT-act):**
1. Constructs `StreamDispatcher::new(None, None, None, None, None, Some(iec104))` — only `iec104` set.
2. Sends a STARTDT-act (`0x68 0x04 0x07 0x00 0x00 0x00`) on `FlowKey(src=60001, dst=2404)`.
3. Asserts `analyzer.flows.len() == 1` — the `DispatchTarget::Iec104` arm called `on_data`, which
   created per-flow state.
4. Asserts `state.session_started == true` — STARTDT-act was processed by the analyzer.

**`test_iec104_only_dispatcher_stopdt_produces_t0881` (threat emission via dispatcher — STOPDT-act):**
1. Constructs same dispatcher (only `iec104` set).
2. Sends a STOPDT-act (`0x68 0x04 0x13 0x00 0x00 0x00`) on a new port-2404 FlowKey.
   `session_started=false` → `detect_iec104_threats` emits T0881 `Verdict::Likely`.
3. Asserts `analyzer.all_findings.len() == 1` and the finding cites `"T0881"`.

**`test_BC_2_05_012_early_exit_guard_includes_iec104` (early-exit guard — ADR-013 Decision 9 step 4):**
- Constructs a `iec104`-only dispatcher.
- Sends a STARTDT-act on port 2404.
- No panic — the early-exit guard correctly includes `&& self.iec104.is_none()`, so the guard
  is `false` (iec104 is Some) and data proceeds to the match arm.
  If the guard had been `&& self.http.is_none() && ... && self.enip.is_none()` (missing iec104),
  the guard would have been `true` and data would have been silently discarded — the test
  catches this bug by asserting the flow state was created.

**`test_iec104_only_guard_unclassified_flows_counted` (guard — non-2404 traffic):**
- With only `iec104` set, a non-2404 flow close is counted as `unclassified_flows` == 1.
- This confirms the early-exit guard is `false` for iec104-only dispatchers on all flows,
  not just port-2404 flows.

**`test_iec104_disabled_port_2404_no_panic` (EC-003 — no analyzer, no panic):**
- Constructs `StreamDispatcher::new(None, None, None, None, None, None)`.
- Sends STOPDT-act and calls `on_flow_close` on a port-2404 flow.
- No panic — the `DispatchTarget::Iec104` arm handles `None` gracefully.

---

## Verdict

PASS — `iec104` field present, `new()` accepts 6 params, early-exit guard extended to
include `iec104.is_none()`, `on_data` and `on_flow_close` both have `Iec104` arms,
and data genuinely reaches `Iec104Analyzer` (verified by state/finding assertions).
