---
document_type: design-authority-decision
cycle_id: feature-enip-v0.11.0
decision_id: BOUNDARY-131-132
status: authoritative
author: architect
date: 2026-06-25
subject: "STORY-131 vs STORY-132 on_data boundary — Wave 58 independence"
---

# STORY-131 / STORY-132 on_data Boundary Decision

## Problem Statement

STORY-131 (Wave 58, `depends_on: []`) must merge before STORY-132 (Wave 59) exists. The
stub-architect placed a `todo!("EnipAnalyzer::on_data — STORY-132 implements frame-walk and
CIP dispatch")` in the `DispatchTarget::Enip` arm of `StreamDispatcher::on_data`. The
test-writer wrote 14 dispatcher integration tests that call `StreamDispatcher::on_data` with
port-44818 flows. Three of those tests (`test_dispatcher_routes_port_44818`,
`test_dispatcher_does_not_route_other_ports`, `test_dispatcher_rule_order_dnp3_before_enip`)
would reach the `DispatchTarget::Enip` arm and panic at the `todo!()`.

BC-2.17.019 has two distinct postconditions:
- PC-1: `classify()` returns `DispatchTarget::Enip` for port-44818 flows
- PC-2: `EnipAnalyzer::on_data()` receives all subsequent TCP bytes for this flow (wiring
  guarantee)
- EC-007: `enip.is_none()` early-exit guard fires when no analyzer is configured

PC-1 and EC-007 are fully implementable and verifiable in Wave 58. PC-2 requires
`on_data` to do something non-panicking when called — but NOT frame-walk or CIP dispatch
(those are STORY-132).

---

## Decision 1: EnipAnalyzer::on_data Body for STORY-131

**Chosen approach: OPTION A — Minimal non-panicking on_data body in STORY-131, replaced by
STORY-132 with full frame-walk.**

The `DispatchTarget::Enip` arm in `StreamDispatcher::on_data` MUST NOT remain a `todo!()` in
STORY-131. Replace it with a call to `EnipAnalyzer::on_data` that has a minimal but
non-panicking body. STORY-132 replaces this body with the real CPF frame-walk.

**Rationale:**

Option B (leave `todo!()`, move routing tests to white-box `classify()` unit tests inside
`dispatcher.rs`) was considered and rejected. The DNP3 precedent (STORY-110) shows that the
dispatcher integration tests DO drive data through `on_data` to confirm PC-2 (wiring
guarantee). The DNP3 tests at `test_port_20000_dispatches_to_dnp3` confirm routing by
asserting `dnp3.flows.is_empty() == false` AFTER calling `dispatcher.on_data()` — i.e., they
verify the data reached the analyzer. This is the pattern we must mirror for ENIP. White-box
`classify()` tests alone verify only PC-1; they cannot verify PC-2 (that the dispatcher
actually calls the analyzer). BC-2.17.019 PC-2 is a wiring contract that requires end-to-end
integration verification.

Additionally, leaving `todo!()` in the `Enip` arm forces the test-writer to contort three
routing tests away from the integration-test file into a `#[cfg(test)]` mod inside
`dispatcher.rs`. That is a non-standard location that breaks the established test namespace
convention (`tests/bc_2_1N_NNN_*` files for story-level tests) and creates a DF-AC-TEST-NAME-SYNC
drift item.

**What STORY-131 must implement in `EnipAnalyzer::on_data`:**

```rust
/// Receive reassembled TCP bytes for a port-44818 flow.
///
/// STORY-131: minimal wiring body — accumulates total received byte count only.
/// Frame-walk, CPF parse, and CIP dispatch are added by STORY-132.
///
/// WIRING-EXEMPT: single field increment and Vec::extend. No branching on content;
/// no finding emission; no side effects beyond the two counter/buffer fields.
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], _ts: u32) {
    self.bytes_received = self.bytes_received.saturating_add(data.len() as u64);
}
```

Add a `bytes_received: u64` field to `EnipAnalyzer` (initialized to 0 in `::new()`).
This counter is:
- the observable that STORY-131 integration tests assert on to verify PC-2
- stable across the STORY-131 → STORY-132 transition (STORY-132 adds frame-walk alongside
  this counter; it does not remove it)
- analogous to `Dnp3Analyzer.flows` being non-empty after routing (different mechanism,
  same purpose: confirm the data reached the analyzer)

`FlowKey` must be passed by value or cloned, matching the existing `Dnp3Analyzer::on_data`
signature: `pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32)`.

The `DispatchTarget::Enip` arm in `StreamDispatcher::on_data` becomes:

```rust
DispatchTarget::Enip => {
    // BC-2.17.019 §P2: route port-44818-classified flow bytes to EnipAnalyzer.
    // Frame-walk and CIP dispatch added by STORY-132.
    if let Some(ref mut enip) = self.enip {
        enip.on_data(flow_key.clone(), data, timestamp);
    }
}
```

**Signature note:** The `flow_key` in `on_data(&mut self, flow_key: &FlowKey, ...)` is a
reference. The ENIP arm must clone it for the `EnipAnalyzer::on_data(flow_key: FlowKey, ...)`
call, matching the DNP3 pattern exactly (see dispatcher.rs line ~365:
`dnp3.on_data(flow_key.clone(), data, timestamp)`).

---

## Decision 2: Exact Observables for STORY-131 Routing Tests

STORY-131 routing tests must assert the following observables. These are achievable in Wave
58 alone and do not require CIP frame-walk.

### PC-1 tests (classify routing — DispatchTarget::Enip selected)

These tests verify that port-44818 data reaches the ENIP analyzer. The observable is
`enip_analyzer.bytes_received > 0` after calling `dispatcher.on_data()` with a port-44818
flow key and a non-TLS, non-HTTP payload.

**`dispatch::test_dispatcher_routes_port_44818`**
- Construct `StreamDispatcher::new(None, None, None, None, Some(EnipAnalyzer::new(50, 5)))`
- Call `dispatcher.on_data(&flow_key(12345, 44818), Direction::ClientToServer, &[0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 0, 1_700_000_000)`
- `let enip = dispatcher.take_enip_analyzer().unwrap()`
- Assert: `enip.bytes_received > 0`
- Rationale: `bytes_received > 0` proves PC-2 (data reached the analyzer); combined with the
  test not panicking, it proves PC-1 (Rule 7 fired and the `DispatchTarget::Enip` arm was
  taken rather than panicking at `todo!()`).

**`dispatch::test_dispatcher_does_not_route_other_ports`**
- Use port 9999 (not 44818, not any known port). Non-TLS, non-HTTP payload.
- Assert: `enip.bytes_received == 0`
- Rationale: confirm Rule 7 is not over-broad (non-44818 flows do NOT reach ENIP analyzer).

**`dispatch::test_dispatcher_rule_order_dnp3_before_enip`**
- Both DNP3 and ENIP analyzers active. Flow on port 20000 (DNP3 Rule 6). Non-TLS payload.
- Assert: `enip.bytes_received == 0` (Rule 6 fires before Rule 7; ENIP gets nothing).
- Optionally assert `dnp3.flows.is_empty() == false` to confirm DNP3 got it (mirrors
  `test_ec006_ports_502_and_20000_modbus_wins` from STORY-110 DNP3 tests).

### EC-007 test (enip=None early-exit)

**`dispatch::test_dispatcher_no_enip_analyzer_port_44818_is_noop`** (new name; see Decision 3)
- `StreamDispatcher::new(None, None, None, None, None)` — all analyzers None
- Call `dispatcher.on_data` with port-44818 flow and `[0x65, 0x00, ...]` payload
- No assertion needed beyond "no panic"
- This covers BC-2.17.019 Invariant 4 (early-exit guard) and EC-007 (enip=None no-op)

### take_enip_analyzer() tests (BC-2.17.019 structural — already green-by-design)

**`dispatch::test_take_enip_analyzer_transfers_ownership`**
- Construct with `Some(EnipAnalyzer::new(50, 5))`
- Assert `dispatcher.enip_analyzer().is_some()` before take
- First `take_enip_analyzer()` → `Some(...)`; accessor now `is_none()`
- Rationale: pure structural test; no `on_data` call needed; green immediately

**`dispatch::test_take_enip_analyzer_returns_none_when_not_set`**
- Construct with `None` for ENIP
- Assert first `take_enip_analyzer()` → `None`
- Rationale: pure structural; green immediately

### CLI flag tests (BC-2.17.020/023/026 — no on_data involvement)

All six CLI tests (`test_cli_enip_flag_constructs_analyzer`, `test_cli_no_enip_flag_no_analyzer`,
`test_cli_all_flag_includes_enip`, `test_enip_without_reassembly_warns_and_disables`,
`test_write_burst_threshold_custom`, `test_write_burst_threshold_default`,
`test_error_burst_threshold_custom`, `test_error_burst_threshold_default`,
`test_error_burst_threshold_zero_semantics`) are pure clap-parse + field-check tests that
never call `on_data`. They are green-by-design once the CLI flags and `EnipAnalyzer::new` are
implemented. No changes to their structure are needed.

---

## Decision 3: Test Location Assignment and DF-AC-TEST-NAME-SYNC Guidance

### Tests that STAY in `tests/enip_analyzer_tests.rs` `mod dispatch`

All 14 STORY-131 tests stay in `tests/enip_analyzer_tests.rs::dispatch`. No tests move to
white-box `#[cfg(test)]` blocks in `dispatcher.rs`. Rationale: the `bytes_received` counter
observable makes the routing tests fully expressible as black-box integration tests.

### Tests that MOVE (none — no migration needed)

No tests move to `dispatcher.rs #[cfg(test)]`. The DNP3 test file
(`tests/bc_2_15_110_dnp3_dispatcher_tests.rs`) confirms that full end-to-end dispatcher
integration tests are the correct home for routing verification.

### One test NAME change required

The current STORY-131 test plan does not include an explicit EC-007 test. The test-writer
should add (or rename one existing test to):

**`dispatch::test_dispatcher_no_enip_analyzer_port_44818_is_noop`**

This replaces no existing test; it is a new test. It verifies EC-007 (enip=None early-exit
on port-44818 flow) and BC-2.17.019 Invariant 4. Without it, the early-exit guard is only
implicitly tested by the other tests that construct with `None`.

**Total test count for `mod dispatch`: 15** (the original 14 + this one EC-007 explicit test).

### DF-AC-TEST-NAME-SYNC verdict

A story-writer DF-AC-TEST-NAME-SYNC sweep of STORY-131.md IS REQUIRED for the following
specific change:
- Add `dispatch::test_dispatcher_no_enip_analyzer_port_44818_is_noop` to the Test Plan
  table in STORY-131.md (new test, not a rename)
- Update the test count comment in STORY-131.md Tasks list from "14 tests" to "15 tests"
- Update AC-131-001 observable language: add "Assert `enip.bytes_received > 0` after routing
  to confirm PC-2 (wiring guarantee)" — the current AC text says only "DispatchTarget::Enip
  is selected" which is PC-1 only

No other sweep changes are required. The 14 original test names are preserved.

---

## Decision 4: What STORY-132 Replaces vs. Extends

STORY-132 (Wave 59) adds the CPF frame-walk and CIP dispatch to `EnipAnalyzer::on_data`. The
`bytes_received` counter from STORY-131 is NOT removed by STORY-132 — it remains as a
diagnostic field. STORY-132 adds the frame-walk body alongside the counter:

```rust
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32) {
    // STORY-131 counter preserved.
    self.bytes_received = self.bytes_received.saturating_add(data.len() as u64);
    // STORY-132: append to per-flow carry buffer, then frame-walk.
    // ... (full implementation per BC-2.17.016 / BC-2.17.005 / BC-2.17.006)
}
```

STORY-132 does NOT need to touch `StreamDispatcher` — the dispatcher arm written by
STORY-131 is already correct and forward-compatible. STORY-132's scope is
`src/analyzer/enip.rs` only (confirming the STORY-132 "Files NOT touched" list is correct).

---

## Summary for Test-Writer and Implementer

| Role | Action Required |
|------|----------------|
| **Implementer (STORY-131)** | Add `bytes_received: u64` field to `EnipAnalyzer`; implement `on_data` with single counter increment (no frame-walk); replace `todo!()` in dispatcher `Enip` arm with `enip.on_data(flow_key.clone(), data, timestamp)` |
| **Test-Writer (STORY-131)** | Change the three routing test assertions from "call on_data and it panics" to "call on_data and assert `enip.bytes_received > 0`"; add `test_dispatcher_no_enip_analyzer_port_44818_is_noop` for EC-007; update AC-131-001 observable language in STORY-131.md |
| **Story-Writer (sweep)** | DF-AC-TEST-NAME-SYNC: add EC-007 test name to Test Plan; update AC-131-001 assertion language; update task list test count to 15 |
| **Implementer (STORY-132)** | Extend `on_data` with carry-buffer append and CPF frame-walk; keep `bytes_received` counter; do NOT touch dispatcher |
