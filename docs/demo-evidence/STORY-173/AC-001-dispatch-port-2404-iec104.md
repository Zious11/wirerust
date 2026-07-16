# AC-173-001: DispatchTarget::Iec104 Variant + Rule 8 Port-2404 Dispatch

**AC:** AC-173-001
**BC:** BC-2.05.012 postconditions 1–3
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

`DispatchTarget::Iec104` is added as a new enum variant in `src/dispatcher.rs`. Rule 8 in
`classify(data: &[u8], flow_key: &FlowKey)` routes TCP flows on port 2404 to the new variant by
matching via `[flow_key.lower_port(), flow_key.upper_port()].contains(&2404)`. This fires after
Rules 1–7 (content-first TLS/HTTP, then port-based Modbus/DNP3/ENIP) so that explicit content
signatures always win. Rule 9 (`DispatchTarget::None`) follows as the no-match fallback.

---

## Source confirmation

`src/dispatcher.rs` module doc comment (lines 22–30):
```
//!  1. TLS content signature → DispatchTarget::Tls
//!  2. HTTP method token     → DispatchTarget::Http
//!  3. Port 443/8443         → DispatchTarget::Tls
//!  4. Port 80/8080          → DispatchTarget::Http
//!  5. Port 502              → DispatchTarget::Modbus  ← Rule 5 (ADR-005)
//!  6. Port 20000            → DispatchTarget::Dnp3   ← Rule 6 (ADR-007)
//!  7. Port 44818            → DispatchTarget::Enip   ← Rule 7 (ADR-010)
//!  8. Port 2404             → DispatchTarget::Iec104 ← Rule 8 (STORY-173, ADR-013)
//!  9. No match              → DispatchTarget::None
```

`DispatchTarget::Iec104` variant declaration (line 64):
```rust
/// Port-2404 IEC 60870-5-104 TCP flows (Rule 8, BC-2.05.012). Added in STORY-173.
Iec104,
```

Rule 8 classify arm (lines 364–370):
```rust
// Rule 8: IEC-104 port (2404 — IANA-assigned, ADR-013 Decision 1). Fires AFTER Rule 7
// ... VP-004 oracle obligation: classify_oracle gains the port-2404 → Iec104 arm.
if ports.contains(&2404) {
    return DispatchTarget::Iec104;
}
// Rule 9: no match.
DispatchTarget::None
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

The `test_iec104_only_dispatcher_data_reaches_analyzer` test (AC-173-008 primary, which
necessarily exercises the `DispatchTarget::Iec104` match arm and Rule 8 classify):

1. Constructs `StreamDispatcher::new(None, None, None, None, None, Some(iec104))`.
2. Calls `on_data` with a port-2404 FlowKey and a STARTDT-act frame (`0x68 0x04 0x07 ...`).
3. Asserts `analyzer.flows.len() == 1` — data reached `Iec104Analyzer::on_data`, confirming
   Rule 8 classified the flow as `DispatchTarget::Iec104`.

The `test_iec104_disabled_port_2404_no_panic` test (EC-003 guard):
- Constructs dispatcher with `iec104=None`; sends a STOPDT-act on port 2404 and calls
  `on_flow_close`; no panic occurs — `DispatchTarget::Iec104` arm handles `None` gracefully.

---

## Verdict

PASS — `DispatchTarget::Iec104` variant present, Rule 8 routes port 2404, both directions
(lower/upper port) match, and the VP-004 oracle update (AC-173-006) was co-delivered.
