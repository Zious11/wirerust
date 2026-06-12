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

// Per DF-TEST-NAMESPACE-001: all STORY-105 tests are grouped inside a dedicated
// `mod story_105` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_105 {
    use std::net::IpAddr;

    use assert_cmd::Command;
    use wirerust::analyzer::modbus::{MAX_ADU_CARRY_BYTES, ModbusAnalyzer};
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
    /// Complete ADU: TxnID=0x0001, ProtoID=0x0000, Len=0x0006, UnitID=0x01, FC=0x03,
    /// starting-address=0x0000, quantity=0x0001 (12 bytes total, Len=6 → adu_len=12).
    ///
    /// GREEN: on_data() routes port-502 flows to ModbusAnalyzer (STORY-105 implemented).
    /// Verifies that a valid MBAP packet delivered on port-502 increments total_pdu_count.
    /// Updated for F-105-001 carry-buffer fix: complete ADUs are required (partial ADUs
    /// are now stashed in carry and not processed until the full ADU arrives).
    #[test]
    fn test_BC_2_14_025_port_502_classified_to_modbus_as_rule_5() {
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(12345, 502);
        // Complete Modbus read-holding-registers request: 12 bytes total.
        // TxnID=0x0001, ProtoID=0x0000, Len=0x0006, UnitID=0x01, FC=0x03,
        // starting-address=0x0000, quantity=0x0001.
        let complete_adu = [
            0x00u8, 0x01, // transaction_id
            0x00, 0x00, // protocol_id
            0x00, 0x06, // length = 6 (UnitID + FC + 4 bytes data)
            0x01, // unit_id
            0x03, // function_code: Read Holding Registers
            0x00, 0x00, // starting address
            0x00, 0x01, // quantity of registers
        ];
        // on_data() routes to ModbusAnalyzer, which parses the complete ADU.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &complete_adu,
            0,
            1_700_000_000,
        );
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
            None,
        );
        let key = flow_key(12345, 502);
        // TLS ClientHello signature: 0x16 0x03 0x03 ...
        let tls_data = [0x16u8, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00];
        // This should route to TLS (Rule 1 fires), NOT Modbus (Rule 5).
        // The TLS analyzer receives data (no panic — TLS on_data is implemented).
        // We verify Modbus analyzer gets no PDUs.
        dispatcher.on_data(&key, Direction::ClientToServer, &tls_data, 0, 1_700_000_000);
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
            None,
        );
        let key = flow_key(12345, 502);
        let http_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        // Rule 2 fires (HTTP content) before Rule 5 (port 502).
        dispatcher.on_data(&key, Direction::ClientToServer, http_data, 0, 1_700_000_000);
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
            None,
        );
        let key = flow_key(12345, 443);
        let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
        // Rule 3 (port 443 → TLS) fires before Rule 5 (port 502 → Modbus).
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &mbap_data,
            0,
            1_700_000_000,
        );
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
        let mut dispatcher = StreamDispatcher::new(None, None, None, None);
        let key = flow_key(12345, 502);
        let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
        // This calls classify() → Modbus, then on_data() Modbus arm → None check → no-op.
        // Should NOT panic (the arm is `if let Some(...)` not unwrap).
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &mbap_data,
            0,
            1_700_000_000,
        );
        // No assertions needed — just verify no panic.
    }

    /// AC-007 (BC-2.14.025 §P2 — on_data routes port-502 flow to ModbusAnalyzer)
    ///
    /// After delivering data on a port-502 flow, the ModbusAnalyzer should have
    /// total_pdu_count > 0 (indicating process_pdu was called).
    #[test]
    fn test_BC_2_14_025_on_data_routes_port_502_flow_to_modbus_analyzer() {
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
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
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &write_pdu,
            0,
            1_700_000_000,
        );

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
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502);
        // Complete ADU: TxnID=0x0001, ProtoID=0x0000, Len=0x0006, UnitID=0x01, FC=0x03,
        // starting-address=0x0000, quantity=0x0001 (12 bytes, F-105-001: full ADU needed).
        let complete_adu = [
            0x00u8, 0x01, // transaction_id
            0x00, 0x00, // protocol_id
            0x00, 0x06, // length = 6 (UnitID + FC + 4 bytes data)
            0x01, // unit_id
            0x03, // function_code: Read Holding Registers
            0x00, 0x00, // starting address
            0x00, 0x01, // quantity of registers
        ];
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &complete_adu,
            0,
            1_700_000_000,
        );
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
        let mut dispatcher =
            StreamDispatcher::new(None, None, Some(ModbusAnalyzer::new(20, 10)), None)
                .with_max_classification_attempts(1);

        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &random_data,
            0,
            1_700_000_000,
        );
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
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);

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
            .any(|a| a.get("analyzer_name").and_then(|n| n.as_str()) == Some("modbus"));

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
            .any(|a| a.get("analyzer_name").and_then(|n| n.as_str()) == Some("modbus"));

        assert!(
            has_modbus,
            "Modbus section MUST appear in output when --modbus is provided (BC-2.14.023 P2). \
         Analyzers found: {:?}",
            analyzers
                .iter()
                .filter_map(|a| a.get("analyzer_name").and_then(|n| n.as_str()))
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
            .any(|a| a.get("analyzer_name").and_then(|n| n.as_str()) == Some("modbus"));

        assert!(
            has_modbus,
            "Modbus section MUST appear in output when --all is provided (BC-2.14.023 P3). \
         Analyzers found: {:?}",
            analyzers
                .iter()
                .filter_map(|a| a.get("analyzer_name").and_then(|n| n.as_str()))
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
                    .any(|a| a.get("analyzer_name").and_then(|n| n.as_str()) == Some("modbus"));
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
            .find(|a| a.get("analyzer_name").and_then(|n| n.as_str()) == Some("modbus"));

        assert!(
            modbus_summary.is_some(),
            "Modbus analyzer summary MUST appear when --modbus alone triggers reassembly \
         (BC-2.14.023 P4, AC-010). Got analyzers: {:?}",
            analyzers
                .iter()
                .filter_map(|a| a.get("analyzer_name").and_then(|n| n.as_str()))
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
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
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
            None,
        );
        let key = flow_key(12345, 8080);
        // Non-HTTP content bytes — port fallback fires (Rule 4: port 8080 → HTTP).
        let binary_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
        // No panic — HTTP arm is implemented.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &binary_data,
            0,
            1_700_000_000,
        );
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
        let mut dispatcher =
            StreamDispatcher::new(None, None, Some(ModbusAnalyzer::new(20, 10)), None)
                .with_max_classification_attempts(1);
        let key = flow_key(12345, 9999);
        let mbap_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
        // classify() returns None (no content match, not 443/8443/80/8080/502).
        // on_data() Modbus arm is NOT reached.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &mbap_data,
            0,
            1_700_000_000,
        );
        // Force close so unclassified_flows is incremented.
        dispatcher.on_flow_close(&key, CloseReason::Timeout);
        assert_eq!(
            dispatcher.unclassified_flows(),
            1,
            "Port 9999 must route to None (not Modbus), incrementing unclassified_flows"
        );
    }

    // ---------------------------------------------------------------------------
    // F-105-002 PINNING TESTS — carry-buffer correctness (partial-ADU buffering)
    // ---------------------------------------------------------------------------

    /// F-105-002 pin-1: TWO complete ADUs concatenated in ONE on_data call.
    ///
    /// Verifies that the walk loop processes multiple ADUs per on_data invocation.
    /// Two FC=0x05 write-single-coil ADUs (12 bytes each) concatenated → 24 bytes.
    /// Expected: total_pdu_count == 2.
    ///
    /// This would have worked before F-105-001 as well (both ADUs were complete),
    /// but it is included here as a regression guard for the multi-ADU loop.
    #[test]
    fn test_F_105_002_pin1_two_complete_adus_in_one_on_data_counted_correctly() {
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502);

        // ADU 1: TxnID=0x0001, Len=6, UnitID=0x01, FC=0x05, addr=0x0000, value=0xFF00
        let adu1 = [
            0x00u8, 0x01, // transaction_id
            0x00, 0x00, // protocol_id
            0x00, 0x06, // length = 6 (UnitID + FC + 4 bytes)
            0x01, // unit_id
            0x05, // FC: Write Single Coil
            0x00, 0x00, // coil address
            0xFF, 0x00, // coil value ON
        ];
        // ADU 2: TxnID=0x0002, same structure
        let adu2 = [
            0x00u8, 0x02, // transaction_id
            0x00, 0x00, // protocol_id
            0x00, 0x06, // length = 6
            0x01, // unit_id
            0x05, // FC: Write Single Coil
            0x00, 0x01, // coil address
            0xFF, 0x00, // coil value ON
        ];

        // Concatenate both ADUs into one chunk and deliver in a single on_data call.
        let mut combined = Vec::with_capacity(24);
        combined.extend_from_slice(&adu1);
        combined.extend_from_slice(&adu2);

        dispatcher.on_data(&key, Direction::ClientToServer, &combined, 0, 1_700_000_000);

        let modbus = dispatcher.take_modbus_analyzer().unwrap();
        assert_eq!(
            modbus.total_pdu_count, 2,
            "F-105-002 pin-1: two concatenated complete ADUs in one on_data call must yield \
         total_pdu_count == 2 (multi-ADU walk loop)"
        );
        assert_eq!(
            modbus.parse_errors, 0,
            "F-105-002 pin-1: no parse errors for well-formed ADUs"
        );
    }

    /// F-105-002 pin-2: ONE write-class ADU SPLIT across two on_data calls.
    ///
    /// This test pins F-105-001 (the partial-ADU buffering bug). It FAILS against
    /// code without the carry buffer and PASSES with the fix.
    ///
    /// A FC=0x05 Write Single Coil ADU is 12 bytes total (Len=6).
    /// First call delivers only bytes 0..5 (5 bytes — MBAP prefix incomplete).
    /// Second call delivers the remaining bytes 5..12 (7 bytes — completes the ADU).
    ///
    /// Expected:
    /// - total_pdu_count == 1 (ONE PDU, not zero or two)
    /// - total_write_count == 1 (ONE write finding)
    /// - parse_errors == 0 (NO spurious parse errors)
    /// - is_non_modbus == false on the flow (not incorrectly disabled)
    ///
    /// Without carry buffer: first call gets 5 bytes → parse_mbap_header returns None
    /// (< 8 bytes) → break without processing. Second call gets 7 bytes → parse_mbap_header
    /// returns None (< 8 bytes) → break again. Result: 0 PDUs, NOT the correct behavior.
    /// (Or alternatively: parse_mbap_header succeeds on the second segment's bytes if
    /// they happen to contain a valid-looking header at offset 0, leading to misparse.)
    #[test]
    fn test_F_105_002_pin2_write_adu_split_across_two_on_data_calls() {
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502);

        // Complete FC=0x05 Write Single Coil ADU (12 bytes):
        // TxnID=0x0001, ProtoID=0x0000, Len=0x0006, UnitID=0x01, FC=0x05,
        // addr=0x0000, value=0xFF00
        let full_adu = [
            0x00u8, 0x01, // transaction_id
            0x00, 0x00, // protocol_id
            0x00, 0x06, // length = 6 (UnitID + FC + 4 data bytes)
            0x01, // unit_id
            0x05, // FC: Write Single Coil
            0x00, 0x00, // coil address
            0xFF, 0x00, // coil value ON
        ];

        // Split after 5 bytes (incomplete MBAP prefix — parse_mbap_header needs >= 8).
        let first_chunk = &full_adu[..5];
        let second_chunk = &full_adu[5..];

        // First on_data: only 5 bytes — not enough for even the MBAP header.
        // With carry buffer: stashed in carry, no PDU processed yet.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            first_chunk,
            0,
            1_700_000_000,
        );

        // Second on_data: remaining 7 bytes — carry (5) + new (7) = 12 bytes → full ADU.
        // With carry buffer: carry prepended, full 12-byte ADU parsed, process_pdu called.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            second_chunk,
            5,
            1_700_000_000,
        );

        let modbus = dispatcher.take_modbus_analyzer().unwrap();
        assert_eq!(
            modbus.total_pdu_count, 1,
            "F-105-002 pin-2: split write ADU must yield exactly ONE PDU \
         (F-105-001 carry buffer fix pins this)"
        );
        assert_eq!(
            modbus.total_write_count, 1,
            "F-105-002 pin-2: split write ADU must emit exactly ONE write finding"
        );
        assert_eq!(
            modbus.parse_errors, 0,
            "F-105-002 pin-2: split write ADU must produce ZERO parse errors \
         (no spurious desync from partial data)"
        );
    }

    /// F-105-002 pin-3: partial MBAP header split (first 3 bytes, then the rest).
    ///
    /// First call delivers only 3 bytes (bytes 0..3 of the MBAP header —
    /// transaction_id only). Second call delivers the remaining 9 bytes
    /// (protocol_id + length + unit_id + FC + 2 bytes data).
    ///
    /// Expected: ONE PDU processed, parse_errors == 0.
    /// This exercises the `parse_mbap_header returns None` carry-stash branch
    /// with a very small initial chunk.
    #[test]
    fn test_F_105_002_pin3_partial_mbap_header_3_bytes_then_rest() {
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502);

        // Complete FC=0x03 Read Holding Registers ADU (12 bytes, Len=6):
        let full_adu = [
            0x00u8, 0x01, // transaction_id
            0x00, 0x00, // protocol_id
            0x00, 0x06, // length = 6
            0x01, // unit_id
            0x03, // FC: Read Holding Registers
            0x00, 0x00, // starting address
            0x00, 0x01, // quantity = 1
        ];

        // Split after just 3 bytes: [0x00, 0x01, 0x00] — partial transaction_id+protocol
        let first_chunk = &full_adu[..3];
        let second_chunk = &full_adu[3..];

        // First on_data: 3 bytes → parse_mbap_header returns None (< 8) → stash in carry.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            first_chunk,
            0,
            1_700_000_000,
        );

        // Second on_data: carry (3) + new (9) = 12 bytes → full ADU → process_pdu called.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            second_chunk,
            3,
            1_700_000_000,
        );

        let modbus = dispatcher.take_modbus_analyzer().unwrap();
        assert_eq!(
            modbus.total_pdu_count, 1,
            "F-105-002 pin-3: partial MBAP header split (3+9 bytes) must yield ONE PDU"
        );
        assert_eq!(
            modbus.parse_errors, 0,
            "F-105-002 pin-3: partial MBAP header split must produce ZERO parse errors"
        );
    }

    // ---------------------------------------------------------------------------
    // F-105-003 CARRY-CAP TESTS — DoS guard for cumulative carry bounding
    // ---------------------------------------------------------------------------

    /// F-105-003 pin-1: carry-cap guard (cumulative) — near-limit safe case.
    ///
    /// Verifies that a partial ADU of exactly MAX_ADU_CARRY_BYTES - 1 bytes (259 bytes)
    /// is stored in carry without triggering the DoS guard, and that the subsequent
    /// call completing the ADU processes it correctly.
    ///
    /// ADU: TxnID=0x0003, ProtoID=0x0000, Len=0x00FE (254 → adu_len=260), UnitID=0x01,
    /// FC=0x01 (Read Coils), followed by 252 bytes of coil data.
    /// Total ADU = 6 + 254 = 260 bytes (max valid ADU).
    ///
    /// Delivery:
    ///   Call 1: bytes 0..259 (259 bytes, one short of complete) → stashed in carry.
    ///   Call 2: byte 259 (final byte) → carry(259) + data(1) = 260 bytes → complete ADU.
    ///
    /// Expected: total_pdu_count == 1, parse_errors == 0 (cap NOT tripped — 259 <= 260).
    /// This is the boundary-safe case: carry stays within the allowed limit.
    #[test]
    fn test_F_105_003_pin1_carry_cap_near_limit_safe_case() {
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502);

        // Build a max-size valid ADU: length=254 → adu_len=260.
        // MBAP: TxnID=0x0003, ProtoID=0x0000, Len=0x00FE, UnitID=0x01, FC=0x01.
        // PDU payload: 252 bytes of 0xAB to fill to exactly adu_len=260.
        let mut full_adu = Vec::with_capacity(260);
        full_adu.extend_from_slice(&[
            0x00u8, 0x03, // transaction_id
            0x00, 0x00, // protocol_id = 0 (valid)
            0x00, 0xFE, // length = 254 (max valid; adu_len = 6 + 254 = 260)
            0x01, // unit_id
            0x01, // FC: Read Coils (request)
        ]);
        // Fill remaining 252 bytes (length=254 covers UnitID + FC + 252 data bytes).
        full_adu.extend(std::iter::repeat_n(0xABu8, 252));
        assert_eq!(
            full_adu.len(),
            260,
            "sanity: full ADU must be exactly 260 bytes"
        );

        // Call 1: deliver 259 bytes — one short of complete.
        // Guard check: flow.carry.len()(0) + remaining.len()(259) = 259 <= 260. NOT tripped.
        // Carry holds 259 bytes.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &full_adu[..259],
            0,
            1_700_000_000,
        );

        // Call 2: deliver final 1 byte — combined buf = carry(259) + data(1) = 260 bytes.
        // adu_len = 260. remaining.len() = 260 >= 260 → complete ADU, processed normally.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &full_adu[259..],
            259,
            1_700_000_001,
        );

        let modbus = dispatcher.take_modbus_analyzer().unwrap();
        assert_eq!(
            modbus.total_pdu_count, 1,
            "F-105-003 pin-1: near-limit partial ADU (259 bytes) + 1-byte completion must yield \
         ONE PDU (carry cap NOT tripped at 259 bytes)"
        );
        assert_eq!(
            modbus.parse_errors, 0,
            "F-105-003 pin-1: near-limit partial must produce ZERO parse errors"
        );
    }

    /// F-105-003 pin-2: carry-cap guard (cumulative) — dribble never-completing stream.
    ///
    /// Verifies that a malicious stream dribbling bytes that accumulate in carry
    /// but can never form a valid Modbus ADU eventually triggers the non-Modbus
    /// desync flag and stops processing — bounding the carry before it could grow
    /// unboundedly.
    ///
    /// Attack scenario: attacker sends bytes 0..6 with protocol_id=0x0100 (non-zero,
    /// invalid Modbus) dribbled one byte per call. For the first 7 calls, fewer than
    /// 8 bytes are in the combined buffer, so parse_mbap_header returns None and the
    /// tail is stashed. On the 8th call (8 bytes total), parse_mbap_header returns
    /// Some, is_valid_modbus_adu fails (protocol_id=0x0100 ≠ 0x0000), and the desync
    /// policy fires: is_non_modbus=true, parse_errors=1.
    ///
    /// After is_non_modbus is set, a subsequent valid complete ADU on the same flow
    /// must produce ZERO new PDUs — verifying the carry is bounded (the flow is
    /// discarded, not accumulated forever).
    ///
    /// The cumulative carry guard `flow.carry.len() + remaining.len() > MAX_ADU_CARRY_BYTES`
    /// is the safety net that prevents this carry accumulation from exceeding one max ADU
    /// (260 bytes) before the desync or guard fires.
    #[test]
    fn test_F_105_003_pin2_dribble_never_completing_stream_is_non_modbus_set() {
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502);

        // Crafted 8-byte sequence: valid enough to parse (>= 8 bytes) but protocol_id != 0.
        // Bytes: TxnID=0x0001, ProtoID=0x0100 (INVALID — not 0x0000), Len=0x0006, UnitID=0x01,
        //        FC=0x03. is_valid_modbus_adu will reject this and set is_non_modbus=true.
        let crafted = [
            0x00u8, 0x01, // transaction_id
            0x01, 0x00, // protocol_id = 0x0100 (non-zero → desync)
            0x00, 0x06, // length = 6
            0x01, // unit_id
            0x03, // function_code
        ];

        // Dribble one byte per call. For calls 1-7: buf < 8 bytes → stash in carry.
        // The carry accumulates 1, 2, 3, …, 7 bytes across calls.
        // Guard check per call: flow.carry.len()(0, because cleared) + remaining.len() <= 260.
        // None of these exceed MAX_ADU_CARRY_BYTES, so carry grows safely (≤ 7 bytes).
        for i in 0..7 {
            dispatcher.on_data(
                &key,
                Direction::ClientToServer,
                &crafted[i..=i],
                i as u64,
                1_700_000_000,
            );
        }

        // Call 8: 8th byte arrives. carry(7) + data(1) = 8 bytes in combined buf.
        // parse_mbap_header returns Some. is_valid_modbus_adu fails (protocol_id != 0).
        // Desync fires: is_non_modbus = true, parse_errors = 1.
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &crafted[7..=7],
            7,
            1_700_000_007,
        );

        // Verify: parse_errors == 1 (desync fired exactly once).
        // total_pdu_count == 0 (no valid ADU was ever processed).
        {
            // Peek at the analyzer without consuming it.
            let modbus_ref = dispatcher.take_modbus_analyzer().unwrap();
            assert_eq!(
                modbus_ref.parse_errors, 1,
                "F-105-003 pin-2: dribble desync must set parse_errors=1"
            );
            assert_eq!(
                modbus_ref.total_pdu_count, 0,
                "F-105-003 pin-2: no valid PDU must be counted on the crafted invalid stream"
            );
            // Re-insert the analyzer so we can test subsequent behavior.
            let mut dispatcher2 = StreamDispatcher::new(None, None, Some(modbus_ref), None);

            // Now send a valid complete ADU on the same flow key.
            // Because is_non_modbus == true, on_data bails immediately without processing.
            let valid_adu = [
                0x00u8, 0x04, // transaction_id
                0x00, 0x00, // protocol_id = 0 (valid)
                0x00, 0x06, // length = 6
                0x01, // unit_id
                0x03, // FC: Read Holding Registers
                0x00, 0x00, // starting address
                0x00, 0x01, // quantity
            ];
            dispatcher2.on_data(
                &key,
                Direction::ClientToServer,
                &valid_adu,
                8,
                1_700_000_001,
            );

            let modbus_final = dispatcher2.take_modbus_analyzer().unwrap();
            assert_eq!(
                modbus_final.total_pdu_count, 0,
                "F-105-003 pin-2: after is_non_modbus is set, subsequent valid ADUs must NOT \
             be processed — carry is bounded (flow discarded, not accumulated)"
            );
            assert_eq!(
                modbus_final.parse_errors, 1,
                "F-105-003 pin-2: parse_errors must stay at 1 after is_non_modbus bails"
            );
        }
    }

    /// F-105-003 pin-3: carry-cap guard cumulative check — MAX_ADU_CARRY_BYTES constant sanity.
    ///
    /// Verifies that MAX_ADU_CARRY_BYTES equals 260 (6 MBAP prefix + 254 max PDU length),
    /// which is exactly one maximum-size Modbus ADU. The cumulative carry guard uses this
    /// bound to ensure carry can hold at most one ADU before firing. This is a structural
    /// pin test: if the constant changes, the carry semantics change and all carry-cap
    /// invariants need re-review.
    #[test]
    fn test_F_105_003_pin3_max_adu_carry_bytes_equals_one_max_adu() {
        // One max ADU = 6 bytes MBAP prefix + 254 bytes (max valid length field).
        // If this fails, update carry-cap comments and re-review the DoS guard threshold.
        assert_eq!(
            MAX_ADU_CARRY_BYTES, 260,
            "MAX_ADU_CARRY_BYTES must equal 260 (6-byte MBAP prefix + max length=254); \
         carry-cap invariant depends on this value"
        );
    }

    // ---------------------------------------------------------------------------
    // F-DELTA-001 mandatory E2E test — timestamp units (seconds) verified end-to-end
    // ---------------------------------------------------------------------------

    /// E2E test: port-502 Modbus flow through StreamDispatcher::on_data with second-scale
    /// timestamps (1_700_000_000 ≈ 2023-11-14).
    ///
    /// Asserts:
    ///   1. Finding.timestamp year is 2023 (confirms pipeline delivers SECONDS, not microseconds).
    ///   2. Burst detector fires when 21 writes are delivered within the 1-second burst window.
    ///
    /// This is the mandatory end-to-end validation for F-DELTA-001: the timestamp unit fix
    /// that corrected modbus.rs from treating timestamp as microseconds to treating it as
    /// seconds (consistent with all other analyzers and the reassembly pipeline).
    ///
    /// Using StreamDispatcher::on_data (not process_pdu directly) ensures the full pipeline
    /// path — dispatcher → ModbusHandler::on_data → process_pdu — is exercised.
    #[test]
    fn test_f_delta_001_e2e_second_scale_timestamps_through_dispatcher() {
        use chrono::Datelike;

        // 1_700_000_000 seconds = 2023-11-14T22:13:20Z (UTC)
        // This is a second-scale UNIX timestamp, matching what the pipeline delivers.
        const TS_2023: u32 = 1_700_000_000;

        // Use a low burst threshold (1) so 2 writes in the same second triggers the burst detector.
        let modbus = ModbusAnalyzer::new(1, 100);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502); // client:49152 → server:502

        // Helper: build a complete Modbus write-single-register ADU (FC=0x06).
        // MBAP: TxnID, ProtoID=0x0000, Len=0x0006, UnitID=0x01, FC=0x06, addr, value.
        let make_write_adu = |txn: u16, addr: u16, value: u16| -> [u8; 12] {
            [
                (txn >> 8) as u8,
                (txn & 0xFF) as u8, // transaction_id
                0x00,
                0x00, // protocol_id = 0
                0x00,
                0x06, // length = 6 (UnitID + FC + 4 bytes data)
                0x01, // unit_id
                0x06, // function_code: Write Single Register
                (addr >> 8) as u8,
                (addr & 0xFF) as u8, // register address
                (value >> 8) as u8,
                (value & 0xFF) as u8, // register value
            ]
        };

        // --- Part 1: Verify timestamp year is 2023 ---
        // Deliver a single write at TS_2023. The per-PDU write finding should have year=2023.
        let adu1 = make_write_adu(0x0001, 0x0010, 0x01F4);
        dispatcher.on_data(&key, Direction::ClientToServer, &adu1, 0, TS_2023);

        let modbus_ref = dispatcher.take_modbus_analyzer().unwrap();
        let write_findings: Vec<_> = modbus_ref
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .collect();

        assert!(
            !write_findings.is_empty(),
            "F-DELTA-001 E2E: at least one write finding must be emitted for FC=0x06"
        );
        let f = &write_findings[0];
        let ts = f
            .timestamp
            .expect("F-DELTA-001 E2E: write finding must have a timestamp (not None)");
        assert_eq!(
            ts.year(),
            2023,
            "F-DELTA-001 E2E: finding timestamp year must be 2023 (confirming SECONDS pipeline, \
         not microseconds). timestamp={ts:?}"
        );

        // --- Part 2: Burst detector fires with second-scale timestamps ---
        // Use threshold=1: 2nd write in same 1-second window fires the burst detector.
        // Deliver 21 writes all at TS_2023 (same second → same burst window).
        let modbus2 = ModbusAnalyzer::new(1, 100);
        let mut dispatcher2 = StreamDispatcher::new(None, None, Some(modbus2), None);

        for txn in 0..21_u16 {
            let adu = make_write_adu(txn + 2, txn, 1);
            dispatcher2.on_data(&key, Direction::ClientToServer, &adu, 0, TS_2023);
        }

        let modbus2_ref = dispatcher2.take_modbus_analyzer().unwrap();
        let burst_findings: Vec<_> = modbus2_ref
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .collect();

        assert_eq!(
            burst_findings.len(),
            1,
            "F-DELTA-001 E2E: burst detector must fire exactly once when 21 writes arrive \
         within a 1-second window (threshold=1). burst_findings={:?}",
            burst_findings.len()
        );
        // Burst finding also has correct year.
        let burst_ts = burst_findings[0]
            .timestamp
            .expect("F-DELTA-001 E2E: burst finding must have a timestamp");
        assert_eq!(
            burst_ts.year(),
            2023,
            "F-DELTA-001 E2E: burst finding timestamp must also be year 2023"
        );
    }

    // ---------------------------------------------------------------------------
    // F-DELTA-003 — proto-id=0 + length-invalid latch test
    // ---------------------------------------------------------------------------

    /// F-DELTA-003 latch: proto-id=0 ADU with length OUT OF [2, 254] latches is_non_modbus.
    ///
    /// A Modbus/TCP ADU with protocol_id=0x0000 (valid Modbus identifier) but length=255
    /// (out of the valid range [2, 254]) fails `is_valid_modbus_adu` and must:
    ///   (a) increment parse_errors to 1
    ///   (b) latch the flow as is_non_modbus (clearing carry)
    ///   (c) cause all subsequent ADUs on the SAME flow to be silently ignored
    ///       (total_pdu_count stays at 0 even after a structurally-valid follow-up ADU)
    ///
    /// This pins the F-DELTA-003 length-invalid branch of the desync policy. The
    /// protocol_id!=0 branch was already tested (F-105-003 pin-2). This test covers the
    /// symmetric proto-id=0 / bad-length path which was previously untested.
    ///
    /// Traces to: BC-2.14.003/004 (is_valid_modbus_adu gate), F-DELTA-003 fix directive.
    #[test]
    fn test_f_delta_003_proto_id_0_length_invalid_latches_is_non_modbus_and_bails() {
        // --- Part (a) + (b): bad ADU causes parse_errors==1 and latches the flow ---
        let modbus = ModbusAnalyzer::new(20, 10);
        let mut dispatcher = StreamDispatcher::new(None, None, Some(modbus), None);
        let key = flow_key(49152, 502);

        // ADU: protocol_id=0x0000 (valid Modbus), length=0x00FF=255 (OUT OF [2, 254]).
        // This passes parse_mbap_header (>= 8 bytes) but fails is_valid_modbus_adu.
        // F-DELTA-003: the invalid-length branch must latch is_non_modbus and increment
        // parse_errors, symmetrically to the protocol_id!=0 branch.
        let bad_length_adu = [
            0x00u8, 0x01, // transaction_id = 1
            0x00, 0x00, // protocol_id = 0x0000 (Modbus — but length is invalid)
            0x00, 0xFF, // length = 255 (INVALID: outside [2, 254])
            0x01, // unit_id
            0x03, // function_code: Read Holding Registers
        ];
        dispatcher.on_data(
            &key,
            Direction::ClientToServer,
            &bad_length_adu,
            0,
            1_700_000_000,
        );

        let modbus_ref = dispatcher.take_modbus_analyzer().unwrap();
        assert_eq!(
            modbus_ref.parse_errors, 1,
            "F-DELTA-003: proto-id=0 + length=255 (invalid) must increment parse_errors to 1 \
         (is_valid_modbus_adu gate fires on the length-out-of-range path)"
        );

        // --- Part (c): subsequent valid ADU on the SAME flow is ignored (latch holds) ---
        // Re-insert the analyzer (with parse_errors=1 and the flow latched) into a new
        // dispatcher and deliver a structurally-valid Modbus ADU on the SAME flow key.
        // Because is_non_modbus was set on the flow, on_data must bail immediately —
        // total_pdu_count must stay at 0.
        let mut dispatcher2 = StreamDispatcher::new(None, None, Some(modbus_ref), None);

        // Valid Read Holding Registers request: proto-id=0, length=6 (in range [2, 254]).
        let valid_adu = [
            0x00u8, 0x02, // transaction_id = 2
            0x00, 0x00, // protocol_id = 0x0000
            0x00, 0x06, // length = 6 (valid)
            0x01, // unit_id
            0x03, // function_code: Read Holding Registers
            0x00, 0x00, // starting address
            0x00, 0x01, // quantity = 1
        ];
        dispatcher2.on_data(
            &key,
            Direction::ClientToServer,
            &valid_adu,
            8,
            1_700_000_001,
        );

        let modbus_final = dispatcher2.take_modbus_analyzer().unwrap();
        assert_eq!(
            modbus_final.total_pdu_count, 0,
            "F-DELTA-003: after length-invalid latch, a subsequent valid ADU on the same flow \
         must NOT be processed — total_pdu_count must stay at 0 (is_non_modbus bail fires)"
        );
        assert_eq!(
            modbus_final.parse_errors, 1,
            "F-DELTA-003: parse_errors must remain at 1 after the latch-bail \
         (no additional parse_errors incremented for latched flows)"
        );
    }
} // mod story_105
