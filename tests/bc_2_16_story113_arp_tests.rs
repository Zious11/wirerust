//! STORY-113 CLI integration tests (GREEN).
//!
//! Exercises CLI-level behavioral contracts for:
//!   BC-2.16.011 — --arp CLI Flag Gates ARP Security Analysis
//!
//! Acceptance criteria tested:
//!   AC-015  test_BC_2_16_011_main_arp_flag_absent_no_findings_no_summary
//!   AC-016  test_BC_2_16_011_main_arp_flag_present_summarize_appended
//!
//! Both tests invoke `wirerust analyze` on a real PCAP (http-ooo.pcap, which
//! contains only Ethernet/IP/TCP frames — no ARP frames). The ArpAnalyzer is
//! fully implemented as of STORY-113:
//!   - AC-015: with --arp absent, no ARP code is invoked and no ARP summary
//!     appears in the JSON output (behavioral gate: "ARP" entry absent from
//!     the "analyzers" array).
//!   - AC-016: with --arp present, summarize() is called at end of capture
//!     and the JSON output contains an "ARP" entry in the "analyzers" array
//!     with all eleven canonical keys.
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
    // summarize() IS called and returns an eleven-key AnalysisSummary (GREEN).
    const IP_ONLY_FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    // ---------------------------------------------------------------------------
    // AC-015 — BC-2.16.011 PC1-4: --arp absent → no ARP analysis, no summary
    // ---------------------------------------------------------------------------

    /// AC-015 (BC-2.16.011 PC1-4): when --arp is absent, process_arp is NOT called,
    /// no ARP findings are emitted, and no ARP AnalysisSummary is appended.
    ///
    /// With --arp absent the binary runs without invoking any arp_analyzer methods.
    /// The behavioral gate is that the JSON output's "analyzers" array must NOT
    /// contain an entry with analyzer_name == "ARP".
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
    /// With --arp present, summarize() is called at end of capture and the JSON output's
    /// "analyzers" array (BC-2.11.001 — the existing top-level key) must contain an entry
    /// with analyzer_name="ARP" and all eleven canonical keys. The in-memory Rust collection
    /// is named `analyzer_summaries`; the JSON key it serializes under is "analyzers"
    /// (BC-2.16.010 Invariant 4).
    #[test]
    fn test_BC_2_16_011_main_arp_flag_present_summarize_appended() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_path = tmp.path().join("with_arp.json");

        // Run WITH --arp flag. summarize() is called and returns the eleven-key summary.
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
            .success();

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
