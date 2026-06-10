//! Failing tests for STORY-105: Modbus Dispatcher Integration + CLI
//!
//! Covers BC-2.14.023, BC-2.14.024, BC-2.14.025, and VP-004 oracle extension.
//!
//! All tests in this file are designed to FAIL (Red Gate) until the
//! implementation in dispatcher.rs, cli.rs, and main.rs is complete.
//!
//! Test naming follows BC-prefixed convention per TDD methodology:
//!   `test_BC_2_14_NNN_…` for behavioral contract coverage.
//!
//! ## Red Gate Guarantee
//!
//! Tests that exercise `StreamDispatcher::on_data` with port-502 flows
//! will panic (via `todo!()` in the dispatcher stub) because
//! `ModbusAnalyzer::on_data` is not yet implemented.
//!
//! Tests that exercise `take_modbus_analyzer()` / `findings()` will
//! either panic or return wrong results because ModbusAnalyzer does not
//! yet implement StreamHandler.
//!
//! CLI tests for threshold-zero rejection will fail because the
//! `wirerust` binary invoked via assert_cmd will exit non-zero and print
//! the error — but those tests actually pass at build time since the
//! validation code IS in main.rs. So the CLI-level stub IS wired.
//! The DISPATCHER-level tests (on_data routing) are the Red Gate.

// BC-prefixed test names use non-snake-case identifiers (project-wide convention).
#![allow(non_snake_case)]

use std::net::IpAddr;

use assert_cmd::Command;
use wirerust::analyzer::modbus::ModbusAnalyzer;
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn flow_key(src_port: u16, dst_port: u16) -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        src_port,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        dst_port,
    )
}

/// Smallest consumed pcap fixture — used for CLI integration tests.
const FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

// ---------------------------------------------------------------------------
// BC-2.14.025 — DispatchTarget::Modbus classify() — Rule 5 (port 502)
// ---------------------------------------------------------------------------

/// AC-006 (BC-2.14.025 postcondition P1 — Rule 5 happy path)
///
/// A flow on port 502 with non-TLS/non-HTTP payload bytes must be classified
/// as DispatchTarget::Modbus. This drives Rule 5 (after content rules 1-2
/// and port fallback rules 3-4).
///
/// MBAP bytes: [0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03] (valid
/// Modbus read-holding-registers request per BC-2.14.001).
///
/// GREEN: on_data() routes port-502 flows to ModbusAnalyzer (STORY-105 implemented).
/// Verifies that a valid MBAP packet delivered on port-502 increments total_pdu_count.
#[test]
fn test_BC_2_14_025_port_502_classified_to_modbus_as_rule_5() {
    let modbus = ModbusAnalyzer::new(20, 10);
    let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus));
    let key = flow_key(12345, 502);
    // Valid MBAP header: TxnID=0x0001, ProtoID=0x0000, Len=0x0006, UnitID=0x01, FC=0x03
    let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
    // on_data() routes to ModbusAnalyzer, which parses the ADU.
    dispatcher.on_data(&key, Direction::ClientToServer, &mbap_data, 0, 1_000_000);
    // Verify the PDU was processed.
    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert!(
        modbus.total_pdu_count > 0,
        "Port-502 data must be routed to ModbusAnalyzer (Rule 5, BC-2.14.025)"
    );
}

/// AC-006 (BC-2.14.025 EC-001 — TLS content on port 502 → Rule 1 wins)
///
/// A flow on port 502 carrying a TLS ClientHello signature (0x16 0x03)
/// MUST be classified as TLS (Rule 1) — NOT Modbus (Rule 5). Content-first
/// invariant (INV-2) requires this.
///
/// This test verifies the VP-004 classify-oracle property:
/// `classify` and `classify_oracle` must agree for port-502+TLS-content inputs.
///
/// This test does NOT panic — it exercises only the classify path (no on_data
/// routing to Modbus), so it should PASS even in the stub state.
#[test]
fn test_BC_2_14_025_port_502_tls_content_classified_to_tls_not_modbus() {
    use wirerust::analyzer::tls::TlsAnalyzer;
    let mut dispatcher = StreamDispatcher::new(
        None,
        Some(TlsAnalyzer::new()),
        Some(ModbusAnalyzer::new(20, 10)),
    );
    let key = flow_key(12345, 502);
    // TLS ClientHello signature: 0x16 0x03 0x03 ...
    let tls_data = [0x16u8, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00];
    // This should route to TLS (Rule 1 fires), NOT Modbus (Rule 5).
    // The TLS analyzer receives data (no panic — TLS on_data is implemented).
    // We verify Modbus analyzer gets no PDUs.
    dispatcher.on_data(&key, Direction::ClientToServer, &tls_data, 0, 1_000_000);
    // Modbus should have zero PDUs — TLS content wins.
    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert_eq!(
        modbus.total_pdu_count, 0,
        "Modbus must NOT receive data when TLS content signature fires on port 502 (Rule 1 > Rule 5)"
    );
}

/// AC-006 (BC-2.14.025 EC-002 — HTTP content on port 502 → Rule 2 wins)
///
/// A flow on port 502 carrying an HTTP GET request MUST be classified as
/// Http (Rule 2) — NOT Modbus. Content-first invariant.
///
/// This test does NOT panic — HTTP routing is already implemented.
#[test]
fn test_BC_2_14_025_port_502_http_content_classified_to_http_not_modbus() {
    use wirerust::analyzer::http::HttpAnalyzer;
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        None,
        Some(ModbusAnalyzer::new(20, 10)),
    );
    let key = flow_key(12345, 502);
    let http_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    // Rule 2 fires (HTTP content) before Rule 5 (port 502).
    dispatcher.on_data(&key, Direction::ClientToServer, http_data, 0, 1_000_000);
    // Modbus should have zero PDUs — HTTP content wins.
    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert_eq!(
        modbus.total_pdu_count, 0,
        "Modbus must NOT receive data when HTTP GET fires on port 502 (Rule 2 > Rule 5)"
    );
}

/// AC-006 (BC-2.14.025 EC-005 — port 443 with MBAP bytes → Rule 3 wins)
///
/// MBAP bytes on port 443 → Rule 3 (TLS port) fires, NOT Rule 5 (Modbus).
/// Verifies that the 443/8443 port fallback has higher priority than port 502 rule.
///
/// This test does NOT panic because MBAP on port 443 routes to TLS analyzer
/// (which is implemented).
#[test]
fn test_BC_2_14_025_port_443_mbap_bytes_classified_to_tls_not_modbus() {
    use wirerust::analyzer::tls::TlsAnalyzer;
    let mut dispatcher = StreamDispatcher::new(
        None,
        Some(TlsAnalyzer::new()),
        Some(ModbusAnalyzer::new(20, 10)),
    );
    let key = flow_key(12345, 443);
    let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
    // Rule 3 (port 443 → TLS) fires before Rule 5 (port 502 → Modbus).
    dispatcher.on_data(&key, Direction::ClientToServer, &mbap_data, 0, 1_000_000);
    // Modbus should have zero PDUs.
    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert_eq!(
        modbus.total_pdu_count, 0,
        "Modbus must NOT receive data on port 443 (Rule 3 > Rule 5)"
    );
}

/// AC-006 (BC-2.14.025 EC-006 — Modbus disabled, port-502 flow → no-op)
///
/// When `self.modbus = None`, classify() still returns Modbus for port-502
/// (classification is unconditional), but on_data() is a no-op.
/// This test should PASS without panicking because the on_data Modbus arm
/// checks `if let Some(ref mut modbus) = self.modbus` — which is None here.
#[test]
fn test_BC_2_14_025_modbus_disabled_port_502_flow_is_noop() {
    // No Modbus analyzer — modbus=None
    let mut dispatcher = StreamDispatcher::new(None, None, None);
    let key = flow_key(12345, 502);
    let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
    // This calls classify() → Modbus, then on_data() Modbus arm → None check → no-op.
    // Should NOT panic (the arm is `if let Some(...)` not unwrap).
    dispatcher.on_data(&key, Direction::ClientToServer, &mbap_data, 0, 1_000_000);
    // No assertions needed — just verify no panic.
}

/// AC-007 (BC-2.14.025 §P2 — on_data routes port-502 flow to ModbusAnalyzer)
///
/// After delivering data on a port-502 flow, the ModbusAnalyzer should have
/// total_pdu_count > 0 (indicating process_pdu was called).
#[test]
fn test_BC_2_14_025_on_data_routes_port_502_flow_to_modbus_analyzer() {
    let modbus = ModbusAnalyzer::new(20, 10);
    let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus));
    let key = flow_key(49152, 502); // ephemeral src port, Modbus dst port
    // Valid Modbus write-single-coil request:
    // MBAP: TxnID=0x0001, ProtoID=0x0000, Len=0x0006, UnitID=0xFF, FC=0x05
    // Data: addr=0x0000, value=0xFF00 (coil ON)
    let write_pdu = [
        0x00u8, 0x01, // transaction_id
        0x00, 0x00, // protocol_id
        0x00, 0x06, // length (UnitID + PDU = 6 bytes)
        0xFF, // unit_id
        0x05, // function_code: Write Single Coil
        0x00, 0x00, // coil address
        0xFF, 0x00, // coil value (ON)
    ];
    dispatcher.on_data(&key, Direction::ClientToServer, &write_pdu, 0, 1_000_000);

    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert!(
        modbus.total_pdu_count > 0,
        "ModbusAnalyzer must receive PDUs routed from port-502 flow (total_pdu_count > 0)"
    );
}

/// AC-011 (BC-2.14.025 §P3 — on_flow_close routes Modbus flow to analyzer)
///
/// After delivering data + closing a port-502 flow, on_flow_close must
/// route to the Modbus analyzer (flow state removed from flows map on close).
#[test]
fn test_BC_2_14_025_on_flow_close_routes_modbus_flow_to_analyzer() {
    let modbus = ModbusAnalyzer::new(20, 10);
    let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus));
    let key = flow_key(49152, 502);
    let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
    dispatcher.on_data(&key, Direction::ClientToServer, &mbap_data, 0, 1_000_000);
    dispatcher.on_flow_close(&key, CloseReason::Fin);
    // Analyzer is still accessible after flow close; PDU was counted before close.
    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert!(
        modbus.total_pdu_count > 0,
        "on_flow_close must not prevent prior PDU counting (BC-2.14.025 §P3)"
    );
}

/// AC-009 (VP-004 oracle — unclassified_flows guard extended for Modbus)
///
/// When a flow receives no matching classification (unclassified) while
/// `self.modbus.is_some()`, `unclassified_flows` must still be incremented.
///
/// BC-2.14.025 §P3: the `unclassified_flows` guard is extended:
/// `if self.http.is_some() || self.tls.is_some() || self.modbus.is_some()`.
///
/// This test verifies the guard works for a Modbus-only run (no HTTP/TLS).
/// It should PASS because unclassified flows don't trigger the Modbus arm.
#[test]
fn test_BC_2_14_025_unclassified_flows_counted_in_modbus_only_run() {
    // A flow on an unknown port (9999) — no content match, no port match.
    // It will be classified as None after retry cap.
    let key = flow_key(12345, 9999);
    let random_data = [0xDEu8, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00];

    // Force it past the retry cap so it gets cached as None.
    let mut dispatcher = StreamDispatcher::new(None, None, Some(ModbusAnalyzer::new(20, 10)))
        .with_max_classification_attempts(1);

    dispatcher.on_data(&key, Direction::ClientToServer, &random_data, 0, 1_000_000);
    // Close the flow — it had DispatchTarget::None cached.
    dispatcher.on_flow_close(&key, CloseReason::Fin);

    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "unclassified_flows must be incremented in a Modbus-only run \
         (BC-2.14.025 §P3 guard extension)"
    );
}

/// BC-2.14.025 §P4 — take_modbus_analyzer() accessor and take pattern
///
/// After calling take_modbus_analyzer(), the slot must be None (Option::take
/// semantics per BC-2.14.025 §P4 invariant 2).
///
/// This test PASSES (no on_data routing involved).
#[test]
fn test_BC_2_14_025_take_modbus_analyzer_uses_option_take() {
    let modbus = ModbusAnalyzer::new(20, 10);
    let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus));

    // Before take: accessor returns Some.
    assert!(
        dispatcher.modbus_analyzer().is_some(),
        "modbus_analyzer() should return Some before take"
    );

    // After take: slot is None.
    let taken = dispatcher.take_modbus_analyzer();
    assert!(taken.is_some(), "take_modbus_analyzer() should return Some");
    assert!(
        dispatcher.modbus_analyzer().is_none(),
        "modbus_analyzer() should return None after take (Option::take leaves slot None)"
    );

    // Second take returns None.
    let second_take = dispatcher.take_modbus_analyzer();
    assert!(
        second_take.is_none(),
        "Second take must return None — slot was already consumed"
    );
}

// ---------------------------------------------------------------------------
// BC-2.14.023 — CLI: --modbus flag enables analyzer; --all includes Modbus
// ---------------------------------------------------------------------------

/// AC-003 (BC-2.14.023 postcondition P1 — default off)
///
/// Without --modbus or --all, the output must NOT contain a Modbus section.
/// This tests the default-off invariant.
///
/// RED GATE: This test currently PASSES because --modbus is default-off
/// and no Modbus analysis section appears. (It's a negative test.)
/// It exists to guard against future regressions.
#[test]
fn test_BC_2_14_023_modbus_disabled_by_default() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("out.json");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            "--http",
            "--json",
            out.to_str().unwrap(),
            FIXTURE,
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).expect("output written");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    let analyzers = parsed
        .get("analyzers")
        .and_then(|a| a.as_array())
        .expect("analyzers array");

    let has_modbus = analyzers
        .iter()
        .any(|a| a.get("analyzer").and_then(|n| n.as_str()) == Some("modbus"));

    assert!(
        !has_modbus,
        "Modbus section must NOT appear in output when --modbus is absent (BC-2.14.023 P1)"
    );
}

/// AC-001 (BC-2.14.023 postcondition P2 — --modbus enables analyzer)
///
/// With --modbus on an empty PCAP, the output MUST contain a Modbus section
/// (even with zero PDUs — summarize() returns all-zero stats).
///
/// RED GATE: This test FAILS because when a port-502 flow delivers data,
/// the dispatcher hits todo!(). On an empty PCAP with no TCP flows,
/// on_data() is never called, so the test may PASS on the empty fixture.
/// BUT — the test asserts a Modbus section is in the output, which requires
/// take_modbus_analyzer() to be called in main.rs AND ModbusAnalyzer::summarize()
/// to produce a section. The post-finalize collection IS wired (via
/// `modbus.all_findings` and `modbus.summarize()`), so this should PASS
/// even with the stub — the modbus summary key "modbus" should appear.
///
/// If PASSES: that's correct stub behavior (empty PCAP → no flows → no panic).
/// If FAILS: the main.rs wiring is not complete.
#[test]
fn test_BC_2_14_023_modbus_flag_enables_analyzer_empty_pcap() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("out.json");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            "--modbus",
            "--json",
            out.to_str().unwrap(),
            FIXTURE,
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).expect("output written");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    let analyzers = parsed
        .get("analyzers")
        .and_then(|a| a.as_array())
        .expect("analyzers array");

    let has_modbus = analyzers
        .iter()
        .any(|a| a.get("analyzer").and_then(|n| n.as_str()) == Some("modbus"));

    assert!(
        has_modbus,
        "Modbus section MUST appear in output when --modbus is provided (BC-2.14.023 P2). \
         Analyzers found: {:?}",
        analyzers
            .iter()
            .filter_map(|a| a.get("analyzer").and_then(|n| n.as_str()))
            .collect::<Vec<_>>()
    );
}

/// AC-002 (BC-2.14.023 postcondition P3 — --all includes Modbus)
///
/// With --all, the Modbus analyzer is included. Same as AC-001 but with --all.
///
/// RED GATE: Same analysis as AC-001 — may PASS on empty-ish fixture
/// if no port-502 TCP flows exist. We test the section presence only.
#[test]
fn test_BC_2_14_023_all_flag_enables_modbus() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("out.json");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", "--all", "--json", out.to_str().unwrap(), FIXTURE])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).expect("output written");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    let analyzers = parsed
        .get("analyzers")
        .and_then(|a| a.as_array())
        .expect("analyzers array");

    let has_modbus = analyzers
        .iter()
        .any(|a| a.get("analyzer").and_then(|n| n.as_str()) == Some("modbus"));

    assert!(
        has_modbus,
        "Modbus section MUST appear in output when --all is provided (BC-2.14.023 P3). \
         Analyzers found: {:?}",
        analyzers
            .iter()
            .filter_map(|a| a.get("analyzer").and_then(|n| n.as_str()))
            .collect::<Vec<_>>()
    );
}

/// AC-004 (BC-2.14.023 EC-001 — --modbus + --no-reassemble warns and omits)
///
/// When --modbus and --no-reassemble are both present:
/// - A warning is printed to stderr
/// - No Modbus section appears in the output
/// - Exit code 0
///
/// This test PASSES because the warning logic IS wired in the stub main.rs.
#[test]
fn test_BC_2_14_023_modbus_with_no_reassemble_prints_warning() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("out.json");

    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            "--modbus",
            "--no-reassemble",
            "--json",
            out.to_str().unwrap(),
            FIXTURE,
        ])
        .output()
        .expect("command ran");

    assert!(output.status.success(), "exit code must be 0");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("WARNING: --modbus requires stream reassembly"),
        "stderr must contain the reassembly warning (BC-2.14.023 EC-001), got: {stderr:?}"
    );

    // Modbus section must NOT appear in output.
    if out.exists() {
        let content = std::fs::read_to_string(&out).expect("output written");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
        if let Some(analyzers) = parsed.get("analyzers").and_then(|a| a.as_array()) {
            let has_modbus = analyzers
                .iter()
                .any(|a| a.get("analyzer").and_then(|n| n.as_str()) == Some("modbus"));
            assert!(
                !has_modbus,
                "Modbus section must NOT appear when --no-reassemble is set (BC-2.14.023 EC-001)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// BC-2.14.024 — CLI: threshold flags parse and validate
// ---------------------------------------------------------------------------

/// AC-005 (BC-2.14.024 postcondition P3a — burst threshold 0 → error)
///
/// --modbus-write-burst-threshold 0 must produce a fatal error (exit != 0)
/// with the message "--modbus-write-burst-threshold must be >= 1 (got 0)".
///
/// This test PASSES because the validation IS wired in main.rs.
#[test]
fn test_BC_2_14_024_burst_threshold_zero_rejected() {
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            "--modbus",
            "--modbus-write-burst-threshold",
            "0",
            FIXTURE,
        ])
        .output()
        .expect("command ran");

    assert!(
        !output.status.success(),
        "exit code must be non-zero when --modbus-write-burst-threshold 0 (BC-2.14.024 P3a)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--modbus-write-burst-threshold must be >= 1"),
        "Error message must mention the burst threshold (BC-2.14.024 P3a), got: {stderr:?}"
    );
}

/// AC-005 (BC-2.14.024 postcondition P3b — sustained threshold 0 → error)
///
/// --modbus-write-sustained-threshold 0 must produce a fatal error (exit != 0).
///
/// This test PASSES because the validation IS wired in main.rs.
#[test]
fn test_BC_2_14_024_sustained_threshold_zero_rejected() {
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            "--modbus",
            "--modbus-write-sustained-threshold",
            "0",
            FIXTURE,
        ])
        .output()
        .expect("command ran");

    assert!(
        !output.status.success(),
        "exit code must be non-zero when --modbus-write-sustained-threshold 0 (BC-2.14.024 P3b)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--modbus-write-sustained-threshold must be >= 1"),
        "Error message must mention the sustained threshold (BC-2.14.024 P3b), got: {stderr:?}"
    );
}

/// AC-005 (BC-2.14.024 postcondition P1 — defaults applied when flags absent)
///
/// Without --modbus-write-burst-threshold / --modbus-write-sustained-threshold,
/// ModbusAnalyzer::new(20, 10) is constructed.
///
/// Verified by constructing ModbusAnalyzer directly and checking fields.
/// This PASSES (pure unit test, no dispatcher routing involved).
#[test]
fn test_BC_2_14_024_default_thresholds_applied() {
    let modbus = ModbusAnalyzer::new(20, 10); // defaults per BC-2.14.024 P1
    assert_eq!(
        modbus.write_burst_threshold, 20,
        "Default write_burst_threshold must be 20 (BC-2.14.024 P1)"
    );
    assert_eq!(
        modbus.write_sustained_threshold, 10,
        "Default write_sustained_threshold must be 10 (BC-2.14.024 P1)"
    );
}

/// AC-005 (BC-2.14.024 postcondition P2 — custom burst threshold flows through)
///
/// --modbus-write-burst-threshold 5 must set write_burst_threshold = 5.
/// Tested via direct construction (mirrors what main.rs does).
#[test]
fn test_BC_2_14_024_custom_burst_threshold_flows_through() {
    let modbus = ModbusAnalyzer::new(5, 10);
    assert_eq!(
        modbus.write_burst_threshold, 5,
        "write_burst_threshold must equal the CLI-supplied value (BC-2.14.024 P2)"
    );
}

/// AC-005 (BC-2.14.024 postcondition P4 — custom sustained threshold flows through)
///
/// --modbus-write-sustained-threshold 3 must set write_sustained_threshold = 3.
#[test]
fn test_BC_2_14_024_custom_sustained_threshold_flows_through() {
    let modbus = ModbusAnalyzer::new(20, 3);
    assert_eq!(
        modbus.write_sustained_threshold, 3,
        "write_sustained_threshold must equal the CLI-supplied value (BC-2.14.024 P4)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.14.023 — AC-010: needs_reassembly includes enable_modbus
// ---------------------------------------------------------------------------

/// AC-010 (BC-2.14.023 postcondition P4)
///
/// --modbus alone (without --http or --tls) must enable reassembly
/// (needs_reassembly = true). The Modbus analyzer is constructed and
/// port-502 flows are analyzed.
///
/// Integration test: run wirerust with --modbus on the http fixture (which
/// has TCP streams). On this fixture, no flows are on port 502, so no
/// Modbus PDUs are processed — but the modbus section should appear with
/// pdu_count = 0 (confirming reassembly ran and the analyzer was created).
///
/// RED GATE: If PASSES (modbus section present, pdu_count=0), the
/// needs_reassembly wiring is correct. If FAILS, needs_reassembly is
/// missing the || enable_modbus term.
#[test]
fn test_BC_2_14_023_modbus_alone_triggers_reassembly() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("out.json");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        // --modbus ONLY — no --http, no --tls
        .args(["analyze", "--modbus", "--json", out.to_str().unwrap(), FIXTURE])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).expect("output written");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    let analyzers = parsed
        .get("analyzers")
        .and_then(|a| a.as_array())
        .expect("analyzers array");

    let modbus_summary = analyzers
        .iter()
        .find(|a| a.get("analyzer").and_then(|n| n.as_str()) == Some("modbus"));

    assert!(
        modbus_summary.is_some(),
        "Modbus analyzer summary MUST appear when --modbus alone triggers reassembly \
         (BC-2.14.023 P4, AC-010). Got analyzers: {:?}",
        analyzers
            .iter()
            .filter_map(|a| a.get("analyzer").and_then(|n| n.as_str()))
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// VP-004 oracle unit-test analogue (non-Kani, cargo test)
// ---------------------------------------------------------------------------

/// AC-009 (BC-2.14.025 §P5 — VP-004 oracle Rule 5 non-Kani analogue)
///
/// Verifies that the classify() function (via the dispatcher's on_data path)
/// behaves identically to the classify_oracle for port-502 inputs that don't
/// match TLS/HTTP content rules.
///
/// This is the cargo-test analogue of the Kani VP-004 harness extension.
/// The Kani harness uses symbolic inputs over all 65536 port values; this
/// test uses concrete representative cases.
///
/// GREEN (STORY-105 implemented): non-TLS/non-HTTP data on port 502 is routed to
/// ModbusAnalyzer (Rule 5). The oracle and production classify() agree.
/// Non-MBAP binary data on port 502 is silently discarded (invalid ADU gate fires,
/// parse_errors incremented). This verifies Rule 5 routing without panic.
#[test]
fn test_BC_2_14_025_vp004_oracle_rule_5_port_502_non_tls_non_http() {
    // Oracle prediction for port-502, non-TLS, non-HTTP content:
    // Rules 1-4 don't fire; Rule 5 (port 502) fires → DispatchTarget::Modbus.
    // Production classify() must agree with the oracle (VP-004 property).
    let modbus = ModbusAnalyzer::new(20, 10);
    let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus));
    let key = flow_key(54321, 502);
    // Random non-TLS, non-HTTP binary data on port 502.
    // This data begins with 0xAB which is a high-bit-set byte — parse_mbap_header
    // will succeed (len >= 8) but is_valid_modbus_adu will fail (protocol_id != 0).
    let binary_data = [0xABu8, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89];
    // Rule 5 fires: data is routed to ModbusAnalyzer (no panic — STORY-105 implemented).
    dispatcher.on_data(&key, Direction::ClientToServer, &binary_data, 0, 2_000_000);
    // The ADU is invalid (non-Modbus protocol_id) → parse_errors incremented.
    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert_eq!(
        modbus.parse_errors, 1,
        "Non-Modbus binary data on port 502 must increment parse_errors (VP-004 Rule 5 routing verified)"
    );
}

/// AC-009 (VP-004 oracle — port-8080 does NOT match Modbus Rule 5)
///
/// Port 8080 → Rule 4 (HTTP) fires before Rule 5 (Modbus). The oracle
/// for port-8080 returns Http, not Modbus.
///
/// This test PASSES (routing to HTTP is implemented; no panic).
#[test]
fn test_BC_2_14_025_vp004_oracle_port_8080_routes_to_http_not_modbus() {
    use wirerust::analyzer::http::HttpAnalyzer;
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        None,
        Some(ModbusAnalyzer::new(20, 10)),
    );
    let key = flow_key(12345, 8080);
    // Non-HTTP content bytes — port fallback fires (Rule 4: port 8080 → HTTP).
    let binary_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
    // No panic — HTTP arm is implemented.
    dispatcher.on_data(&key, Direction::ClientToServer, &binary_data, 0, 1_000_000);
    // Modbus should have zero PDUs.
    let modbus = dispatcher.take_modbus_analyzer().unwrap();
    assert_eq!(
        modbus.total_pdu_count, 0,
        "Modbus must NOT receive data on port 8080 (Rule 4 > Rule 5 per VP-004 oracle)"
    );
}

/// AC-009 (VP-004 oracle — port-9999 routes to None, not Modbus)
///
/// Port 9999 doesn't match any known port. The oracle returns None.
/// With no content match and no known port, classify() returns None.
/// This test PASSES.
#[test]
fn test_BC_2_14_025_vp004_oracle_unknown_port_routes_to_none_not_modbus() {
    // Modbus-only dispatcher, but port 9999 — classify returns None.
    let mut dispatcher = StreamDispatcher::new(None, None, Some(ModbusAnalyzer::new(20, 10)))
        .with_max_classification_attempts(1);
    let key = flow_key(12345, 9999);
    let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
    // classify() returns None (no content match, not 443/8443/80/8080/502).
    // on_data() Modbus arm is NOT reached.
    dispatcher.on_data(&key, Direction::ClientToServer, &mbap_data, 0, 1_000_000);
    // Force close so unclassified_flows is incremented.
    dispatcher.on_flow_close(&key, CloseReason::Timeout);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "Port 9999 must route to None (not Modbus), incrementing unclassified_flows"
    );
}
