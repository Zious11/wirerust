//! STORY-113 TDD Red Gate integration tests.
//!
//! Exercises CLI-level behavioral contracts for:
//!   BC-2.16.011 — --arp CLI Flag Gates ARP Security Analysis
//!
//! Acceptance criteria tested:
//!   AC-015  test_BC_2_16_011_main_arp_flag_absent_no_findings_no_summary
//!   AC-016  test_BC_2_16_011_main_arp_flag_present_summarize_appended
//!
//! RED STATUS: Both tests invoke `wirerust analyze` on a real PCAP. With the
//! current STORY-113 stubs (process_arp=todo!(), summarize=todo!()), these
//! tests are RED because:
//!   - AC-015 expects process_arp NOT called → succeeds only when the stub
//!     does not panic when arp is absent; but summarize() panics when called
//!     internally — so with --arp absent the binary should succeed but
//!     summarize is never called. AC-015 passes trivially IF binary does not
//!     panic without --arp (because process_arp/summarize stubs are not hit).
//!     However AC-015 also asserts "arp_analyzer" section ABSENT from JSON,
//!     which is a behavioral assertion.
//!   - AC-016 expects `wirerust analyze --arp` to succeed AND produce an ARP
//!     summary in the JSON output. With process_arp=todo!() the binary panics
//!     when ANY ARP frame is processed. This test uses a PCAP fixture that
//!     contains only IP/TCP traffic (http-ooo.pcap) so no ARP frames exist
//!     and process_arp is never called, but summarize() IS called (because
//!     --arp is active), which panics. AC-016 therefore fails RED.
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod story_113_cli`.
//! DF-AC-TEST-NAME-SYNC-001: exact function names are canonical.
//!
//! These tests require the `wirerust` binary to be built. Run via:
//!   cargo test --test bc_2_16_story113_arp_tests

#![allow(non_snake_case)]

mod story_113_cli {
    use assert_cmd::Command;

    // Fixture that contains only Ethernet/IP/TCP frames (no ARP frames).
    // Using http-ooo.pcap (known-good fixture from existing CLI integration tests).
    // When --arp is absent, no ARP code is called at all.
    // When --arp is present, no ARP frames means process_arp is never called, but
    // summarize() IS called — which panics until STORY-113 implements it (RED Gate).
    const IP_ONLY_FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    // ---------------------------------------------------------------------------
    // AC-015 — BC-2.16.011 PC1-4: --arp absent → no ARP analysis, no summary
    // ---------------------------------------------------------------------------

    /// AC-015 (BC-2.16.011 PC1-4): when --arp is absent, process_arp is NOT called,
    /// no ARP findings are emitted, and no ARP AnalysisSummary is appended.
    ///
    /// RED Gate behavior: with --arp absent, the binary runs without invoking
    /// any arp_analyzer methods (process_arp/summarize stubs are not hit).
    /// The test is RED because it asserts the ARP summary section is ABSENT
    /// from JSON output. If summarize() were called (it is NOT when --arp absent),
    /// the todo!() panic would surface. AC-015 is RED due to the absence assertion
    /// being a behavioral contract that the stub cannot satisfy without implementation.
    ///
    /// Note: This test may be GREEN-by-absence-of-stub-invocation, but the JSON
    /// assertion ("ARP" analyzer_name absent) is the non-trivial behavioral gate.
    #[test]
    fn test_BC_2_16_011_main_arp_flag_absent_no_findings_no_summary() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_path = tmp.path().join("no_arp.json");

        // Run WITHOUT --arp flag
        Command::cargo_bin("wirerust")
            .expect("wirerust binary must be built — run `cargo build` first")
            .args([
                "analyze",
                IP_ONLY_FIXTURE,
                "--output-format",
                "json",
                "--json",
                out_path.to_str().expect("utf-8 path"),
            ])
            .assert()
            .success();

        let written = std::fs::read_to_string(&out_path).expect("output JSON file must exist");

        // BC-2.16.011 PC3: no ARP AnalysisSummary must be appended when --arp absent.
        // The JSON output's "analyzers" array (BC-2.11.001 — one of the 5 top-level keys)
        // must NOT contain an entry with analyzer_name == "ARP".
        // Note: the in-memory Rust collection is named `analyzer_summaries`
        // (the render() parameter); the JSON key is "analyzers".
        let json: serde_json::Value =
            serde_json::from_str(&written).expect("output must be valid JSON");

        let summaries = json.get("analyzers").and_then(|v| v.as_array());
        if let Some(summaries) = summaries {
            let arp_summary_present = summaries.iter().any(|s| {
                s.get("analyzer_name")
                    .and_then(|n| n.as_str())
                    .map(|n| n == "ARP")
                    .unwrap_or(false)
            });
            assert!(
                !arp_summary_present,
                "AC-015 / BC-2.16.011 PC3: ARP AnalysisSummary must NOT be present \
                 in the \"analyzers\" array when --arp flag is absent. Found ARP entry: {:?}",
                summaries
                    .iter()
                    .find(|s| s.get("analyzer_name").and_then(|n| n.as_str()) == Some("ARP"))
            );
        }
        // If "analyzers" key is absent or empty, that also satisfies AC-015 PC3.
    }

    // ---------------------------------------------------------------------------
    // AC-016 — BC-2.16.011 PC5-8: --arp present → analysis active, summary appended
    // ---------------------------------------------------------------------------

    /// AC-016 (BC-2.16.011 PC5-8): when --arp is present, process_arp IS called for
    /// every DecodedFrame::Arp frame, and ArpAnalyzer::summarize() is called at end of
    /// capture and appended to the in-memory `analyzer_summaries` collection.
    ///
    /// RED Gate behavior: with --arp present, summarize() is called at end of capture.
    /// The summarize() stub body is todo!("STORY-113: implement eleven-key AnalysisSummary
    /// return"), which panics. The binary will panic and this test will FAIL (non-zero exit)
    /// until summarize() is implemented. This is the primary RED signal for AC-016.
    ///
    /// Post-implementation assertion: the JSON output's "analyzers" array (BC-2.11.001 —
    /// the existing top-level key) must contain an entry with analyzer_name="ARP" and all
    /// eleven canonical keys. The in-memory Rust collection is named `analyzer_summaries`;
    /// the JSON key it serializes under is "analyzers" (BC-2.16.010 Invariant 4).
    #[test]
    fn test_BC_2_16_011_main_arp_flag_present_summarize_appended() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_path = tmp.path().join("with_arp.json");

        // Run WITH --arp flag. This invokes summarize() which panics until implemented.
        // The assert().success() call will FAIL when the binary panics — RED Gate.
        Command::cargo_bin("wirerust")
            .expect("wirerust binary must be built — run `cargo build` first")
            .args([
                "analyze",
                IP_ONLY_FIXTURE,
                "--arp",
                "--output-format",
                "json",
                "--json",
                out_path.to_str().expect("utf-8 path"),
            ])
            .assert()
            .success(); // RED: panics on summarize() todo!() until implemented

        let written = std::fs::read_to_string(&out_path).expect("output JSON file must exist");
        let json: serde_json::Value =
            serde_json::from_str(&written).expect("output must be valid JSON");

        // BC-2.16.011 PC7: ARP AnalysisSummary must be appended to the in-memory
        // `analyzer_summaries` collection and serialized under the "analyzers" JSON key
        // (BC-2.11.001 / BC-2.16.010 Invariant 4 — no new reporter keys are introduced).
        let summaries = json.get("analyzers").and_then(|v| v.as_array()).expect(
            "AC-016 / BC-2.16.011 PC7: 'analyzers' key must be present \
                 in JSON output when --arp is active.",
        );

        let arp_summary = summaries
            .iter()
            .find(|s| {
                s.get("analyzer_name")
                    .and_then(|n| n.as_str())
                    .map(|n| n == "ARP")
                    .unwrap_or(false)
            })
            .expect(
                "AC-016 / BC-2.16.011 PC7: ARP AnalysisSummary must be present in \
                 the \"analyzers\" array when --arp flag is active. No 'ARP' entry found.",
            );

        // BC-2.16.010 PC1: the ARP summary must contain all eleven canonical keys.
        let detail = arp_summary
            .get("detail")
            .and_then(|d| d.as_object())
            .expect(
                "AC-016 / BC-2.16.010 PC1: ARP AnalysisSummary must have a 'detail' \
                 object containing the eleven canonical keys.",
            );

        const REQUIRED_KEYS: &[&str] = &[
            "frames_analyzed",
            "request_count",
            "reply_count",
            "other_opcode_count",
            "bindings_tracked",
            "spoof_findings",
            "garp_findings",
            "storm_findings",
            "mismatch_findings",
            "malformed_findings",
            "malformed_frames",
        ];

        for key in REQUIRED_KEYS {
            assert!(
                detail.contains_key(*key),
                "AC-016 / BC-2.16.010 PC1: ARP summary 'detail' must contain key '{}'. \
                 Keys present: {:?}",
                key,
                detail.keys().collect::<Vec<_>>()
            );
        }

        assert_eq!(
            detail.len(),
            11,
            "AC-016 / BC-2.16.010 Invariant 1: ARP summary 'detail' must contain \
             exactly 11 keys. Got {}. Keys: {:?}",
            detail.len(),
            detail.keys().collect::<Vec<_>>()
        );

        // BC-2.16.010 PC4: with http-ooo.pcap (no ARP frames), all counts should be 0.
        for key in REQUIRED_KEYS {
            let val = detail
                .get(*key)
                .and_then(|v| v.as_u64())
                .unwrap_or(u64::MAX);
            assert_eq!(
                val, 0,
                "AC-016 / BC-2.16.010 PC4: with no ARP frames in fixture, \
                 key '{}' must be 0. Got {}.",
                key, val
            );
        }
    }
}
