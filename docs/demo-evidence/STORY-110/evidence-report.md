---
story_id: STORY-110
bc: BC-2.15.017, BC-2.15.021
date: 2026-06-11
recorder: demo-recorder
---

# STORY-110 Demo Evidence Report

DNP3 Dispatcher Integration + CLI Flag — Rule 6 Dispatch, Content-First Precedence, Early-Exit Guard, `take_dnp3_analyzer`, `--dnp3-direct-operate-threshold`, VP-007 Atomic-Update (wave 39 — FINAL E-15 story)

## AC Coverage (8/10 demonstrated; AC-005 + AC-011 F6-deferred)

| AC | BC | Test Name(s) | Assertion | Recording | Result |
|----|----|-------------|-----------|-----------|--------|
| AC-001 | BC-2.15.021 PC1 | `test_port_20000_dispatches_to_dnp3` | `classify(ports={20000}, data=[])` → `DispatchTarget::Dnp3` (Rule 6); Rules 1–5 not triggered (no TLS/HTTP content, no TLS/HTTP/Modbus port) | [GIF](AC-001-port-20000-dispatches-to-dnp3.gif) / [WEBM](AC-001-port-20000-dispatches-to-dnp3.webm) | PASS |
| AC-002 | BC-2.15.021 PC5/6 | `test_tls_on_port_20000_routes_to_tls`, `test_http_on_port_20000_routes_to_http` | TLS ClientHello on port 20000 → `DispatchTarget::Tls` (Rule 1, NOT Rule 6); HTTP GET on port 20000 → `DispatchTarget::Http` (Rule 2, NOT Rule 6). Content-first precedence (INV-2) preserved. | [GIF](AC-002-content-first-precedence.gif) / [WEBM](AC-002-content-first-precedence.webm) | PASS |
| AC-003 | BC-2.15.021 INV4 | `test_early_exit_guard_includes_dnp3` | Early-exit guard in `on_data` extended to `&& self.dnp3.is_none()`; data NOT silently dropped when only DNP3 analyzer is active | [GIF](AC-003-early-exit-guard.gif) / [WEBM](AC-003-early-exit-guard.webm) | PASS |
| AC-004 | BC-2.15.021 INV5 | `test_take_dnp3_analyzer_moves_out` | `dispatcher.take_dnp3_analyzer()` returns `Some(analyzer)` on first call, `None` on second call (moved out); mirrors `take_modbus_analyzer()` | [GIF](AC-004-take-dnp3-analyzer.gif) / [WEBM](AC-004-take-dnp3-analyzer.webm) | PASS |
| AC-005 | BC-2.15.021 VP-004 oracle | Kani: `verify_content_first_precedence_exhaustive` | `classify_oracle` gains port-20000→Dnp3 arm after port-502→Modbus arm; Kani proof asserts `got == want` for all 65536² combinations | **F6-DEFERRED** — Kani harness; not runnable at unit-test time |  |
| AC-006 | BC-2.15.017 PC1/2 | `test_cli_flag_dnp3_direct_operate_threshold_parsed` | `--dnp3-direct-operate-threshold N` parses N as `u32` and sets `Dnp3Analyzer.direct_operate_threshold = N`; omitted flag applies default 10 (`DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`) | [GIF](AC-006-cli-flag-threshold-parsed.gif) / [WEBM](AC-006-cli-flag-threshold-parsed.webm) | PASS |
| AC-007 | BC-2.15.017 PC3/4 | `test_threshold_0_fires_immediately`, `test_threshold_max_never_fires` | threshold=0 → T1692.001 on first Control-class FC (count=1 > 0); threshold=u32::MAX (4294967295) → T1692.001 never fires | [GIF](AC-007-threshold-edge-values.gif) / [WEBM](AC-007-threshold-edge-values.webm) | PASS |
| AC-008 | BC-2.15.017 PC5 | `test_threshold_echoed_in_t1692_summary` | T1692.001 finding summary contains `"(threshold {N})"` with the configured N; default (10) appears when flag omitted | [GIF](AC-008-threshold-echoed-in-summary.gif) / [WEBM](AC-008-threshold-echoed-in-summary.webm) | PASS |
| AC-009 | BC-2.15.021 INV1/2 | `test_port_502_and_20000_routes_to_modbus` | Rule 5 (port 502, Modbus) fires before Rule 6 (port 20000, DNP3); flow with both ports → `DispatchTarget::Modbus`; `DispatchTarget::None` is now Rule 7; no Modbus flows stolen | [GIF](AC-009-rule-ordering-no-stolen-flows.gif) / [WEBM](AC-009-rule-ordering-no-stolen-flows.webm) | PASS |
| AC-010 | BC-2.15.021 VP-007 | `test_vp007_seeded_23_emitted_15` (integration) + `vp007_catalog_drift_guard` (in-crate unit) | Integration: all 23 seeded IDs resolve to non-empty, non-"Unknown" name/tactic via public API. In-crate unit: `SEEDED_TECHNIQUE_IDS.len() == 23` AND `SEEDED_TECHNIQUE_ID_COUNT == 23`. Kani/EMITTED=15 check is F6-gate. | [GIF](AC-010-vp007-seeded-23-emitted-15.gif) / [WEBM](AC-010-vp007-seeded-23-emitted-15.webm) | PASS |
| AC-011 | BC-2.15.021 VP-023 | F6-gate state-manager task | VP-023 status draft→verified; VP-INDEX verified 22→23, draft 1→0; after all four Kani proofs in STORY-106 run green | **F6-DEFERRED** — not a unit-test obligation |  |

## F6-Deferred Items

| AC | Reason | Gate |
|----|--------|------|
| AC-005 | `verify_content_first_precedence_exhaustive` is a Kani harness; requires `cargo kani` (nightly + kani toolchain). Not runnable at unit-test time. Oracle arm (`classify_oracle` port-20000→Dnp3) is implemented in same commit as `classify()` per architecture compliance rule 2. | Wave 39 F6 |
| AC-011 | VP-023 status propagation is a factory-artifacts state-manager step, not a code test. Runs after all four STORY-106 Kani proofs report `VERIFICATION:- SUCCESSFUL`. | Wave 39 F6 |

## End-to-End CLI Demo

**Produced**: YES

The headline STORY-110 delivery is demonstrated end-to-end:

1. Release binary built (`cargo build --release`)
2. `wirerust analyze --help` shows `--dnp3-direct-operate-threshold` with default 10
3. Full 26-test dispatcher suite runs and passes

Recording: [GIF](E2E-cli-dnp3-dispatcher.gif) / [WEBM](E2E-cli-dnp3-dispatcher.webm)

The CLI demo shows real `--help` output (not fabricated) confirming the flag is live. A real pcap end-to-end is not demonstrated (no DNP3 pcap fixture in the test tree); the integration test suite for BC-2.15.021 / BC-2.15.017 covers all dispatch and detection paths via synthetic byte sequences.

## Full Test Run Output

```
running 26 tests
test story_110::test_AC_003_early_exit_guard_does_not_fire_when_dnp3_is_some ... ok
test story_110::test_BC_2_15_021_detect_control_burst_unit_threshold_0_fires_on_first ... ok
test story_110::test_BC_2_15_021_detect_control_burst_unit_fires_at_threshold_plus_1 ... ok
test story_110::test_BC_2_15_021_detect_control_burst_unit_threshold_echoed_in_summary ... ok
test story_110::test_BC_2_15_021_detect_control_burst_window_expiry_resets_counter ... ok
test story_110::test_BC_2_15_021_threshold_stored_in_dnp3_analyzer ... ok
test story_110::test_cli_flag_dnp3_direct_operate_threshold_parsed ... ok
test story_110::test_early_exit_guard_includes_dnp3 ... ok
test story_110::test_ec001_non_dnp3_content_on_port_20000_desync_bail ... ok
test story_110::test_ec002_multiple_frames_in_one_on_data_call ... ok
test story_110::test_ec003_partial_frame_split_across_two_on_data_calls ... ok
test story_110::test_ec005_unknown_port_routes_to_none ... ok
test story_110::test_ec006_ports_502_and_20000_modbus_wins ... ok
test story_110::test_ec007_dnp3_disabled_port_20000_flow_is_noop ... ok
test story_110::test_ec008_threshold_omitted_defaults_to_10 ... ok
test story_110::test_http_on_port_20000_routes_to_http ... ok
test story_110::test_none_is_rule_7_no_match ... ok
test story_110::test_port_20000_dispatches_to_dnp3 ... ok
test story_110::test_port_502_and_20000_routes_to_modbus ... ok
test story_110::test_take_dnp3_analyzer_moves_out ... ok
test story_110::test_threshold_0_fires_immediately ... ok
test story_110::test_threshold_default_10_echoed_in_t1692_summary ... ok
test story_110::test_threshold_echoed_in_t1692_summary ... ok
test story_110::test_threshold_max_never_fires ... ok
test story_110::test_tls_on_port_20000_routes_to_tls ... ok
test story_110::test_vp007_seeded_23_emitted_15 ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test bc_2_15_110_dnp3_dispatcher_tests`

## VP-007 Drift Guard (in-crate unit test)

```
running 1 test
test mitre::vp007_format_tests::vp007_catalog_drift_guard ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out; finished in 1.77s
```

Command: `cargo test vp007_catalog_drift_guard -- --nocapture`

## STORY-109 Regression — CLEAN (34/34)

```
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_correlation_tests`

## STORY-108 Regression — CLEAN (26/26)

```
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_detection_tests`

## STORY-107 Regression — CLEAN (14/14)

```
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_flow_state_tests`

## STORY-106 Regression — CLEAN (36/36)

```
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_parse_core_tests`

## Dispatch Architecture Narrative

### Rule Table (post-STORY-110)

| Rule | Condition | Target |
|------|-----------|--------|
| Rule 1 | `data[0]==0x16 && data[1]==0x03` (TLS ClientHello) | `DispatchTarget::Tls` |
| Rule 2 | `data` starts with HTTP verb prefix | `DispatchTarget::Http` |
| Rule 3 | `ports.contains(&443)` (TLS port) | `DispatchTarget::Tls` |
| Rule 4 | `ports.contains(&80)` or `ports.contains(&8080)` (HTTP port) | `DispatchTarget::Http` |
| Rule 5 | `ports.contains(&502)` (Modbus) | `DispatchTarget::Modbus` |
| **Rule 6** | **`ports.contains(&20000)` (DNP3)** | **`DispatchTarget::Dnp3`** |
| Rule 7 | No match | `DispatchTarget::None` |

Content rules (1–2) always win over port rules (3–7). Rule 6 was added in this story as the DNP3 port rule. `DispatchTarget::None` shifted from Rule 6 to Rule 7.

### Early-Exit Guard

Before this story the guard read: `if self.http.is_none() && self.tls.is_none() && self.modbus.is_none()`. A dispatcher with only a DNP3 analyzer would silently skip `on_data` (silent data drop, AC-003 invariant 4). The guard is now extended with `&& self.dnp3.is_none()`.

### CLI Wiring

`Commands::Analyze.dnp3_direct_operate_threshold: u32` (default `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT = 10`) is parsed by clap and passed to `Dnp3Analyzer::new(threshold)` in the `run_analyze` orchestrator. The flag echoes in every T1692.001 finding summary as `"(threshold {threshold})"`.

## Tape Sources

All VHS tape scripts are committed alongside recordings. Each tape covers one AC.

| Tape | AC |
|------|----|
| [AC-001-port-20000-dispatches-to-dnp3.tape](AC-001-port-20000-dispatches-to-dnp3.tape) | AC-001 |
| [AC-002-content-first-precedence.tape](AC-002-content-first-precedence.tape) | AC-002 |
| [AC-003-early-exit-guard.tape](AC-003-early-exit-guard.tape) | AC-003 |
| [AC-004-take-dnp3-analyzer.tape](AC-004-take-dnp3-analyzer.tape) | AC-004 |
| [AC-006-cli-flag-threshold-parsed.tape](AC-006-cli-flag-threshold-parsed.tape) | AC-006 |
| [AC-007-threshold-edge-values.tape](AC-007-threshold-edge-values.tape) | AC-007 |
| [AC-008-threshold-echoed-in-summary.tape](AC-008-threshold-echoed-in-summary.tape) | AC-008 |
| [AC-009-rule-ordering-no-stolen-flows.tape](AC-009-rule-ordering-no-stolen-flows.tape) | AC-009 |
| [AC-010-vp007-seeded-23-emitted-15.tape](AC-010-vp007-seeded-23-emitted-15.tape) | AC-010 |
| [E2E-cli-dnp3-dispatcher.tape](E2E-cli-dnp3-dispatcher.tape) | End-to-End CLI |
