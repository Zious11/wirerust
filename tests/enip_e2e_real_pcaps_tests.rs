//! Full-pipeline end-to-end tests for the EtherNet/IP analyzer against real ENIP/CIP pcaps.
//!
//! Exercises the complete `PcapSource::from_file → decode_packet → TcpReassembler →
//! StreamDispatcher → EnipAnalyzer` pipeline using real-world captures and asserts that the
//! analyzer's outputs match the ground-truth outcomes established by the holdout evaluator
//! (`.factory/holdout-scenarios/eval-runs/`).
//!
//! ## Fixture management
//!
//! Captures live in `tests/fixtures/local-samples/` (gitignored — see E2E-PCAPS.md). When
//! that directory is absent or a specific fixture file is missing, the affected test prints a
//! skip notice and returns immediately. This keeps CI green without fixtures while still
//! failing loudly (assertion-level) when fixtures are present. `#[ignore]` is NOT used.
//!
//! To populate fixtures locally:
//!
//! ```bash
//! bin/fetch-e2e-pcaps
//! ```
//!
//! ## Test cases and pcap → holdout mapping
//!
//! | Test | Pcap | Holdout | What it asserts |
//! |------|------|---------|-----------------|
//! | `test_e2e_BC_2_17_enip_test_pcap_T0846_listidentity` | `enip_test.pcap` (ITI CC-BY-4.0) | HS-114 | Exactly 1 T0846 finding (ListIdentity, Reconnaissance/Likely/High), 2 PDUs, 0 parse_errors |
//! | `test_e2e_BC_2_17_enip_enum_attr_T0888_identity_reads` | `enip_enum_attr_PLC.pcapng` (MIT) | HS-115 | ≥ 200 T0888 findings (Reconnaissance/Likely/High); enip_summary has error_count = 190, total_pdu_count = 406, parse_errors = 0 |
//! | `test_e2e_BC_2_17_enip_connect_upload_forwardopen_anomaly_empty_mitre` | `enip_connect_to_plc1_and_upload.pcapng` (MIT) | HS-116 | ForwardOpen (0x54) findings present: Anomaly/Possible/Low with empty mitre_techniques |
//! | `test_e2e_BC_2_17_enip_read_tags_zero_findings_no_false_positives` | `enip_read_tags.pcapng` (MIT) | HS-122 Case A | 0 findings, 0 parse_errors, 8 PDUs — known-good no false positives |
//! | `test_e2e_BC_2_17_ethernet_ip_cip_large_clean_no_panic` | `EthernetIP-CIP.pcap` (ITI CC-BY-4.0) | HS-110 + HS-120 + HS-122 | 8799 PDUs dispatched to ENIP, 0 parse_errors, 0 findings, no panic |
//! | `test_e2e_BC_2_17_enip_metasploit_zero_T0858_T0816_correct_scope` | `enip_metasploit.pcapng` (MIT) | HS-119 + HS-111/112 determination | 0 T0858/T0816 findings (correct: non-standard framing); 13 PDUs, 0 parse_errors |
//!
//! ## Traces
//!
//! BC-2.17.019 (ENIP dispatch port 44818), BC-2.17.020 (--enip flag), BC-2.17.021 (enip_summary),
//! BC-2.17.023 (T0836 write-burst), BC-2.17.026 (T0888 error-burst), HS-110..HS-122.
//!
//! Per DF-TEST-NAMESPACE-001: all tests are wrapped in `mod enip_e2e_real_pcaps`.

#![allow(non_snake_case)]

mod enip_e2e_real_pcaps {
    use std::path::Path;

    use wirerust::analyzer::enip::EnipAnalyzer;
    use wirerust::decoder::{DecodedFrame, decode_packet};
    use wirerust::dispatcher::StreamDispatcher;
    use wirerust::reader::PcapSource;
    use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

    // -------------------------------------------------------------------------
    // Fixture root — relative to the crate root (same convention as other E2E tests)
    // -------------------------------------------------------------------------

    const LOCAL_SAMPLES: &str = "tests/fixtures/local-samples";

    // -------------------------------------------------------------------------
    // Skip-if-absent guard (mirrors e2e_corpus_smoke_tests.rs pattern)
    //
    // Returns true if the fixture is present, false if the test should be skipped.
    // -------------------------------------------------------------------------

    /// Check whether a fixture file is present in `tests/fixtures/local-samples/`.
    ///
    /// When the file is absent the caller prints a skip notice and returns early.
    /// This keeps CI green when the gitignored local-samples directory is not populated.
    fn fixture_present(filename: &str) -> bool {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(LOCAL_SAMPLES)
            .join(filename);
        if !path.exists() {
            eprintln!(
                "[enip-e2e] SKIP: fixture '{}' not found at {}. \
                 Run `bin/fetch-e2e-pcaps` to populate local-samples.",
                filename,
                path.display()
            );
            false
        } else {
            true
        }
    }

    // -------------------------------------------------------------------------
    // Pipeline helper — run the full reader → reassembler → dispatcher → ENIP
    // pipeline on `filename`, return the ENIP analyzer (which owns all_findings).
    //
    // Mirrors the `run_analyze` pipeline in `src/main.rs` and the pattern from
    // `tests/fixture_reassembly_tests.rs` and `tests/multi_analyzer_e2e_tests.rs`.
    // -------------------------------------------------------------------------

    /// Run the full ENIP analysis pipeline on a pcap/pcapng file.
    ///
    /// The file must exist under `tests/fixtures/local-samples/`. This function panics
    /// if `PcapSource::from_file` fails — the fixture was present but unreadable, which
    /// is a test infrastructure failure, not a skip condition.
    ///
    /// Returns the `EnipAnalyzer` after `reassembler.finalize(&mut dispatcher)` so that
    /// all per-flow state has been flushed and `on_flow_close` has been called for every
    /// completed stream, making `summarize()` return the full aggregate.
    fn run_enip_pipeline(filename: &str) -> EnipAnalyzer {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(LOCAL_SAMPLES)
            .join(filename);

        let source = PcapSource::from_file(&path)
            .unwrap_or_else(|e| panic!("[enip-e2e] failed to open {filename}: {e:#}"));

        let config = ReassemblyConfig::default();
        let mut reassembler = TcpReassembler::new(config);
        // EnipAnalyzer::new(write_burst_threshold, error_burst_threshold).
        // Use the same defaults as the CLI: write_burst=50, error_burst=5.
        let mut dispatcher =
            StreamDispatcher::new(None, None, None, None, Some(EnipAnalyzer::new(50, 5)));

        for raw in &source.packets {
            if let Ok(DecodedFrame::Ip(parsed)) = decode_packet(&raw.data, source.datalink) {
                reassembler.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher);
            }
        }

        // Flush any open flows (calls on_flow_close for each, folding per-flow counters
        // into the aggregate fields read by summarize()).
        reassembler.finalize(&mut dispatcher);

        // Take ownership of the ENIP analyzer out of the dispatcher.
        dispatcher
            .take_enip_analyzer()
            .expect("[enip-e2e] ENIP analyzer must be present after run_enip_pipeline")
    }

    // =========================================================================
    // Test 1 — enip_test.pcap (ITI CC-BY-4.0)
    //
    // Pcap: 2 ENIP PDUs (command 0x0063 ListIdentity), 1 flow, 0 errors.
    // Expected: exactly 1 T0846 finding, Reconnaissance/Likely/High.
    //
    // Holdout: HS-114 (ListIdentity / T0846, one-shot guard).
    // Traces: BC-2.17.019, BC-2.17.014 (T0846 one-shot), HS-114.
    // License: ITI/ICS-Security-Tools CC-BY-4.0 (see E2E-PCAPS.md attribution section).
    // =========================================================================

    /// test_e2e_BC_2_17_enip_test_pcap_T0846_listidentity
    ///
    /// enip_test.pcap contains 2 ENIP PDUs with command=0x0063 (ListIdentity) from
    /// src=10.1.1.167, dispatched over TCP/44818. The one-shot guard means exactly
    /// one T0846 finding fires despite two PDUs.
    ///
    /// Postconditions asserted (ground-truth from holdout evaluator run-manifest.md):
    /// - `enip.all_findings` has exactly 1 finding.
    /// - That finding's `mitre_techniques` contains "T0846".
    /// - That finding's `verdict` is `Likely`.
    /// - That finding's `confidence` is `High`.
    /// - That finding's `category` is `Reconnaissance`.
    /// - `enip_summary.total_pdu_count` == 2.
    /// - `enip_summary.flows_analyzed` == 1.
    /// - `enip_summary.parse_errors` == 0.
    /// - `enip_summary.error_count` == 0.
    /// - `enip_summary.command_distribution` contains "0x0063": 2.
    ///
    /// Traces: BC-2.17.019, BC-2.17.014 Pattern A (T0846 ListIdentity), HS-114.
    #[test]
    fn test_e2e_BC_2_17_enip_test_pcap_T0846_listidentity() {
        if !fixture_present("enip_test.pcap") {
            return;
        }

        let enip = run_enip_pipeline("enip_test.pcap");

        // ── Findings count ───────────────────────────────────────────────────
        assert_eq!(
            enip.all_findings.len(),
            1,
            "enip_test.pcap: expected exactly 1 finding (T0846 one-shot guard); \
             got {} findings: {:?}",
            enip.all_findings.len(),
            enip.all_findings
                .iter()
                .map(|f| f.mitre_techniques.as_slice())
                .collect::<Vec<_>>()
        );

        // ── Technique, verdict, confidence, category ─────────────────────────
        let f = &enip.all_findings[0];
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0846"),
            "enip_test.pcap finding[0]: must contain T0846 (ListIdentity); \
             got mitre_techniques={:?}",
            f.mitre_techniques
        );
        assert_eq!(
            format!("{}", f.verdict),
            "LIKELY",
            "enip_test.pcap finding[0]: expected verdict=Likely, got {:?}",
            f.verdict
        );
        assert_eq!(
            format!("{}", f.confidence),
            "HIGH",
            "enip_test.pcap finding[0]: expected confidence=High, got {:?}",
            f.confidence
        );
        assert_eq!(
            format!("{:?}", f.category),
            "Reconnaissance",
            "enip_test.pcap finding[0]: expected category=Reconnaissance, got {:?}",
            f.category
        );

        // ── enip_summary (via summarize()) ────────────────────────────────────
        let summary = enip.summarize();
        assert_eq!(
            summary.analyzer_name, "EtherNet/IP",
            "enip_test.pcap: analyzer_name must be 'EtherNet/IP'"
        );

        let enip_summary = &summary.detail["enip_summary"];

        assert_eq!(
            enip_summary["total_pdu_count"],
            serde_json::json!(2u64),
            "enip_test.pcap: total_pdu_count must be 2; got {:?}",
            enip_summary["total_pdu_count"]
        );
        assert_eq!(
            enip_summary["flows_analyzed"],
            serde_json::json!(1u64),
            "enip_test.pcap: flows_analyzed must be 1; got {:?}",
            enip_summary["flows_analyzed"]
        );
        assert_eq!(
            enip_summary["parse_errors"],
            serde_json::json!(0u64),
            "enip_test.pcap: parse_errors must be 0; got {:?}",
            enip_summary["parse_errors"]
        );
        assert_eq!(
            enip_summary["error_count"],
            serde_json::json!(0u64),
            "enip_test.pcap: error_count must be 0; got {:?}",
            enip_summary["error_count"]
        );
        assert_eq!(
            enip_summary["write_count"],
            serde_json::json!(0u64),
            "enip_test.pcap: write_count must be 0; got {:?}",
            enip_summary["write_count"]
        );
        // command_distribution must show 0x0063: 2 (two ListIdentity PDUs)
        assert_eq!(
            enip_summary["command_distribution"]["0x0063"],
            serde_json::json!(2u64),
            "enip_test.pcap: command_distribution[\"0x0063\"] must be 2; \
             got {:?}",
            enip_summary["command_distribution"]
        );
    }

    // =========================================================================
    // Test 2 — enip_enum_attr_PLC.pcapng (scy-phy/bro-cip-enip MIT)
    //
    // Pcap: 406 ENIP PDUs (2x RegisterSession + 404x SendRRData), 1 flow.
    // Expected: 202 T0888 findings (Identity class GetAttributesAll/Single reads),
    //           error_count = 190 (CIP error responses), parse_errors = 0.
    //
    // Holdout: HS-115 (T0888 Pattern A — Identity-read recon).
    // Traces: BC-2.17.019, BC-2.17.026 (T0888 detection), HS-115.
    // License: scy-phy/bro-cip-enip MIT.
    // =========================================================================

    /// test_e2e_BC_2_17_enip_enum_attr_T0888_identity_reads
    ///
    /// enip_enum_attr_PLC.pcapng contains GetAttributesAll (service=0x01) and
    /// GetAttributeSingle (service=0x0E) CIP requests targeting the Identity Object
    /// (class 0x01). This is the canonical T0888 (Reconnaissance) real-world corpus
    /// sample with the highest confirmed TP count: 202 findings.
    ///
    /// The 190 error_count represents CIP error responses (general_status != 0)
    /// from the PLC — the server returned error replies to many attribute reads.
    ///
    /// ## MANIFEST DISCREPANCY NOTE (HS-115 / run-manifest.md)
    ///
    /// run-manifest.md stated "T0888 (x202, verdict=Likely, confidence=High)" — this
    /// was imprecise. The actual JSON output from the holdout evaluator run contains:
    ///   - 201 × T0888 Pattern A findings: verdict=Likely, confidence=High
    ///     (GetAttributesAll/GetAttributeSingle Identity Object reads)
    ///   - 1 × T0888 Pattern B finding: verdict=Possible, confidence=Medium
    ///     (error-response burst: 6 error responses in 10s window)
    ///     → finding[15]: "CIP error-response burst: 6 error responses in 10s window
    ///     — possible service enumeration (T0888)"
    ///
    /// This test asserts the REAL ground truth (the JSON file), not the imprecise
    /// manifest summary. Pattern B fires the error-burst detector at a lower confidence
    /// than Pattern A (request-side identity reads), which is correct behavior.
    ///
    /// Postconditions asserted (ground-truth from eval-runs JSON output):
    /// - `enip.all_findings.len()` == 202.
    /// - Every finding has "T0888" in `mitre_techniques`.
    /// - 201 findings have verdict=Likely, confidence=High, category=Reconnaissance (Pattern A).
    /// - Exactly 1 finding has verdict=Possible, confidence=Medium, category=Reconnaissance
    ///   (Pattern B error-burst).
    /// - `enip_summary.total_pdu_count` == 406.
    /// - `enip_summary.flows_analyzed` == 1.
    /// - `enip_summary.parse_errors` == 0.
    /// - `enip_summary.error_count` == 190.
    /// - `enip_summary.command_distribution` contains "0x0065": 2, "0x006F": 404.
    ///
    /// Traces: BC-2.17.019, BC-2.17.026 (T0888), HS-115.
    #[test]
    fn test_e2e_BC_2_17_enip_enum_attr_T0888_identity_reads() {
        if !fixture_present("enip_enum_attr_PLC.pcapng") {
            return;
        }

        let enip = run_enip_pipeline("enip_enum_attr_PLC.pcapng");

        // ── Findings count (exact regression guard) ───────────────────────────
        assert_eq!(
            enip.all_findings.len(),
            202,
            "enip_enum_attr_PLC.pcapng: expected exactly 202 T0888 findings \
             (ground-truth from HS-115 eval-run JSON); got {}",
            enip.all_findings.len()
        );

        // ── Every finding must contain T0888 ──────────────────────────────────
        for (i, f) in enip.all_findings.iter().enumerate() {
            assert!(
                f.mitre_techniques.iter().any(|t| t == "T0888"),
                "enip_enum_attr_PLC.pcapng finding[{i}]: must contain T0888; \
                 got mitre_techniques={:?}",
                f.mitre_techniques
            );
            // Every finding is Reconnaissance regardless of Pattern A vs B.
            assert_eq!(
                format!("{:?}", f.category),
                "Reconnaissance",
                "enip_enum_attr_PLC.pcapng finding[{i}]: expected category=Reconnaissance"
            );
        }

        // ── Verdict/confidence distribution: 201 Likely/High + 1 Possible/Medium ─
        // (Pattern A = GetAttributesAll/Single on Identity class → Likely/High;
        //  Pattern B = error-burst in window → Possible/Medium per HS-115 determination.)
        let likely_high = enip
            .all_findings
            .iter()
            .filter(|f| {
                format!("{}", f.verdict) == "LIKELY" && format!("{}", f.confidence) == "HIGH"
            })
            .count();
        let possible_medium = enip
            .all_findings
            .iter()
            .filter(|f| {
                format!("{}", f.verdict) == "POSSIBLE" && format!("{}", f.confidence) == "MEDIUM"
            })
            .count();

        assert_eq!(
            likely_high, 201,
            "enip_enum_attr_PLC.pcapng: expected 201 Likely/High T0888 findings (Pattern A); \
             got {likely_high}"
        );
        assert_eq!(
            possible_medium, 1,
            "enip_enum_attr_PLC.pcapng: expected exactly 1 Possible/Medium T0888 finding \
             (Pattern B error-burst); got {possible_medium}"
        );

        // ── enip_summary ──────────────────────────────────────────────────────
        let summary = enip.summarize();
        let enip_summary = &summary.detail["enip_summary"];

        assert_eq!(
            enip_summary["total_pdu_count"],
            serde_json::json!(406u64),
            "enip_enum_attr_PLC.pcapng: total_pdu_count must be 406; got {:?}",
            enip_summary["total_pdu_count"]
        );
        assert_eq!(
            enip_summary["flows_analyzed"],
            serde_json::json!(1u64),
            "enip_enum_attr_PLC.pcapng: flows_analyzed must be 1"
        );
        assert_eq!(
            enip_summary["parse_errors"],
            serde_json::json!(0u64),
            "enip_enum_attr_PLC.pcapng: parse_errors must be 0"
        );
        assert_eq!(
            enip_summary["error_count"],
            serde_json::json!(190u64),
            "enip_enum_attr_PLC.pcapng: error_count must be 190 (CIP error responses); \
             got {:?}",
            enip_summary["error_count"]
        );
        assert_eq!(
            enip_summary["write_count"],
            serde_json::json!(0u64),
            "enip_enum_attr_PLC.pcapng: write_count must be 0"
        );
        assert_eq!(
            enip_summary["command_distribution"]["0x0065"],
            serde_json::json!(2u64),
            "enip_enum_attr_PLC.pcapng: command_distribution[\"0x0065\"] must be 2"
        );
        assert_eq!(
            enip_summary["command_distribution"]["0x006F"],
            serde_json::json!(404u64),
            "enip_enum_attr_PLC.pcapng: command_distribution[\"0x006F\"] must be 404"
        );
    }

    // =========================================================================
    // Test 3 — enip_connect_to_plc1_and_upload.pcapng (scy-phy/bro-cip-enip MIT)
    //
    // Pcap: 4094 ENIP PDUs (82x SendRRData + 4012x SendUnitData), 1 flow.
    // Expected: 17 ForwardOpen/ForwardClose findings, category=Anomaly,
    //           verdict=Possible, confidence=Low, no mitre_techniques (empty MITRE).
    //
    // Holdout: HS-116 (ForwardOpen lifecycle anomaly — empty MITRE per ADR-010 Decision 7).
    // Traces: BC-2.17.019, HS-116.
    // License: scy-phy/bro-cip-enip MIT.
    // =========================================================================

    /// test_e2e_BC_2_17_enip_connect_upload_forwardopen_anomaly_empty_mitre
    ///
    /// enip_connect_to_plc1_and_upload.pcapng contains a CIP connection lifecycle
    /// (ForwardOpen/ForwardClose service=0x54/0x4E) during a program upload session.
    /// v0.11.0 emits lifecycle anomaly findings for ForwardOpen (service=0x54) with
    /// empty `mitre_techniques` per ADR-010 Decision 7 (no dedicated MITRE ICS technique
    /// for CIP connection establishment anomaly at this confidence level).
    ///
    /// Postconditions asserted (ground-truth from run-manifest.md):
    /// - `enip.all_findings.len()` == 17.
    /// - Every finding has category=Anomaly, verdict=Possible, confidence=Low.
    /// - Every finding has an empty `mitre_techniques` list (HS-116 MITRE-empty assertion).
    /// - `enip_summary.total_pdu_count` == 4094.
    /// - `enip_summary.parse_errors` == 0.
    /// - `enip_summary.error_count` == 0.
    /// - `enip_summary.write_count` == 0.
    ///
    /// Traces: BC-2.17.019, HS-116.
    #[test]
    fn test_e2e_BC_2_17_enip_connect_upload_forwardopen_anomaly_empty_mitre() {
        if !fixture_present("enip_connect_to_plc1_and_upload.pcapng") {
            return;
        }

        let enip = run_enip_pipeline("enip_connect_to_plc1_and_upload.pcapng");

        // ── Findings count ───────────────────────────────────────────────────
        assert_eq!(
            enip.all_findings.len(),
            17,
            "enip_connect_to_plc1_and_upload.pcapng: expected exactly 17 findings \
             (HS-116 / run-manifest.md); got {}",
            enip.all_findings.len()
        );

        // ── Every finding: Anomaly / Possible / Low / empty mitre_techniques ─
        for (i, f) in enip.all_findings.iter().enumerate() {
            // HS-116: mitre_techniques must be empty (no MITRE technique mapped for
            // CIP connection lifecycle anomaly per ADR-010 Decision 7).
            assert!(
                f.mitre_techniques.is_empty(),
                "enip_connect_to_plc1_and_upload.pcapng finding[{i}]: \
                 mitre_techniques must be empty (HS-116 ADR-010 Decision 7); \
                 got {:?}",
                f.mitre_techniques
            );
            assert_eq!(
                format!("{:?}", f.category),
                "Anomaly",
                "enip_connect_to_plc1_and_upload.pcapng finding[{i}]: expected Anomaly"
            );
            assert_eq!(
                format!("{}", f.verdict),
                "POSSIBLE",
                "enip_connect_to_plc1_and_upload.pcapng finding[{i}]: expected verdict=Possible"
            );
            assert_eq!(
                format!("{}", f.confidence),
                "LOW",
                "enip_connect_to_plc1_and_upload.pcapng finding[{i}]: expected confidence=Low"
            );
        }

        // ── enip_summary ──────────────────────────────────────────────────────
        let summary = enip.summarize();
        let enip_summary = &summary.detail["enip_summary"];

        assert_eq!(
            enip_summary["total_pdu_count"],
            serde_json::json!(4094u64),
            "enip_connect_to_plc1_and_upload.pcapng: total_pdu_count must be 4094"
        );
        assert_eq!(
            enip_summary["parse_errors"],
            serde_json::json!(0u64),
            "enip_connect_to_plc1_and_upload.pcapng: parse_errors must be 0"
        );
        assert_eq!(
            enip_summary["error_count"],
            serde_json::json!(0u64),
            "enip_connect_to_plc1_and_upload.pcapng: error_count must be 0"
        );
        assert_eq!(
            enip_summary["write_count"],
            serde_json::json!(0u64),
            "enip_connect_to_plc1_and_upload.pcapng: write_count must be 0"
        );
    }

    // =========================================================================
    // Test 4 — enip_read_tags.pcapng (scy-phy/bro-cip-enip MIT)
    //
    // Pcap: 8 ENIP PDUs (4x RegisterSession + 4x SendRRData), 2 flows.
    // Expected: 0 findings (known-good: normal CIP read-tags — NO false positives).
    //
    // Holdout: HS-122 Case A (known-good arm — zero attack-technique false positives).
    // Traces: BC-2.17.019, HS-122.
    // License: scy-phy/bro-cip-enip MIT.
    // =========================================================================

    /// test_e2e_BC_2_17_enip_read_tags_zero_findings_no_false_positives
    ///
    /// enip_read_tags.pcapng contains normal SWaT CIP tag-read traffic. No attack
    /// techniques or anomalous behavior — this is the "known-good" benchmark.
    ///
    /// Postconditions asserted (ground-truth from run-manifest.md):
    /// - `enip.all_findings` is empty (0 findings — no false positives).
    /// - `enip_summary.total_pdu_count` == 8.
    /// - `enip_summary.flows_analyzed` == 2.
    /// - `enip_summary.parse_errors` == 0.
    /// - `enip_summary.error_count` == 0.
    /// - `enip_summary.write_count` == 0.
    ///
    /// Traces: BC-2.17.019, HS-122 (known-good arm).
    #[test]
    fn test_e2e_BC_2_17_enip_read_tags_zero_findings_no_false_positives() {
        if !fixture_present("enip_read_tags.pcapng") {
            return;
        }

        let enip = run_enip_pipeline("enip_read_tags.pcapng");

        // ── Zero findings (no false positives) ────────────────────────────────
        assert!(
            enip.all_findings.is_empty(),
            "enip_read_tags.pcapng: expected 0 findings (known-good traffic); \
             got {} findings: {:?}",
            enip.all_findings.len(),
            enip.all_findings
                .iter()
                .map(|f| f.mitre_techniques.as_slice())
                .collect::<Vec<_>>()
        );

        // ── enip_summary ──────────────────────────────────────────────────────
        let summary = enip.summarize();
        let enip_summary = &summary.detail["enip_summary"];

        assert_eq!(
            enip_summary["total_pdu_count"],
            serde_json::json!(8u64),
            "enip_read_tags.pcapng: total_pdu_count must be 8; got {:?}",
            enip_summary["total_pdu_count"]
        );
        assert_eq!(
            enip_summary["flows_analyzed"],
            serde_json::json!(2u64),
            "enip_read_tags.pcapng: flows_analyzed must be 2"
        );
        assert_eq!(
            enip_summary["parse_errors"],
            serde_json::json!(0u64),
            "enip_read_tags.pcapng: parse_errors must be 0"
        );
        assert_eq!(
            enip_summary["error_count"],
            serde_json::json!(0u64),
            "enip_read_tags.pcapng: error_count must be 0"
        );
        assert_eq!(
            enip_summary["write_count"],
            serde_json::json!(0u64),
            "enip_read_tags.pcapng: write_count must be 0"
        );
    }

    // =========================================================================
    // Test 5 — EthernetIP-CIP.pcap (ITI/ICS-Security-Tools CC-BY-4.0)
    //
    // Pcap: 8799 ENIP PDUs (438x SendRRData + 8361x SendUnitData), 4 flows.
    // Expected: 0 findings, 0 parse_errors, no panic. Largest clean dispatch run.
    //
    // Holdout: HS-110 (canonical LE-header decode), HS-120 (dispatch port 44818),
    //          HS-122 (real-world corpus — clean parse mandate).
    // Traces: BC-2.17.019, BC-2.17.021, HS-110, HS-120, HS-122.
    // License: ITI/ICS-Security-Tools CC-BY-4.0.
    // Attribution: ICS Security Tools, Illinois Institute of Technology (ITI). CC-BY-4.0.
    // =========================================================================

    /// test_e2e_BC_2_17_ethernet_ip_cip_large_clean_no_panic
    ///
    /// EthernetIP-CIP.pcap is the largest clean ENIP dispatch run in the corpus
    /// (8799 PDUs, sourced from CloudShark). It validates:
    /// (a) LE header decoding across 8799 frames (HS-110).
    /// (b) Port-44818 dispatch routing (HS-120) — all PDUs reach the ENIP analyzer.
    /// (c) Real-world corpus mandate (HS-122) — no panic, 0 parse_errors.
    ///
    /// Postconditions asserted (ground-truth from run-manifest.md):
    /// - No panic (the test completes; any panic is a test failure by definition).
    /// - `enip.all_findings` is empty (0 findings).
    /// - `enip_summary.total_pdu_count` == 8799.
    /// - `enip_summary.flows_analyzed` == 4.
    /// - `enip_summary.parse_errors` == 0.
    /// - `enip_summary.error_count` == 0.
    /// - `enip_summary.write_count` == 0.
    /// - `enip_summary.command_distribution` contains "0x006F": 438 and "0x0070": 8361.
    ///
    /// Traces: BC-2.17.019, HS-110, HS-120, HS-122.
    #[test]
    fn test_e2e_BC_2_17_ethernet_ip_cip_large_clean_no_panic() {
        if !fixture_present("EthernetIP-CIP.pcap") {
            return;
        }

        let enip = run_enip_pipeline("EthernetIP-CIP.pcap");

        // ── Zero findings ─────────────────────────────────────────────────────
        assert!(
            enip.all_findings.is_empty(),
            "EthernetIP-CIP.pcap: expected 0 findings (large clean capture — HS-122 Case A); \
             got {} findings: {:?}",
            enip.all_findings.len(),
            enip.all_findings
                .iter()
                .map(|f| f.mitre_techniques.as_slice())
                .collect::<Vec<_>>()
        );

        // ── enip_summary ──────────────────────────────────────────────────────
        let summary = enip.summarize();
        let enip_summary = &summary.detail["enip_summary"];

        assert_eq!(
            enip_summary["total_pdu_count"],
            serde_json::json!(8799u64),
            "EthernetIP-CIP.pcap: total_pdu_count must be 8799 (HS-120 dispatch corroboration); \
             got {:?}",
            enip_summary["total_pdu_count"]
        );
        assert_eq!(
            enip_summary["flows_analyzed"],
            serde_json::json!(4u64),
            "EthernetIP-CIP.pcap: flows_analyzed must be 4"
        );
        assert_eq!(
            enip_summary["parse_errors"],
            serde_json::json!(0u64),
            "EthernetIP-CIP.pcap: parse_errors must be 0 (HS-110 / HS-122 mandate)"
        );
        assert_eq!(
            enip_summary["error_count"],
            serde_json::json!(0u64),
            "EthernetIP-CIP.pcap: error_count must be 0"
        );
        assert_eq!(
            enip_summary["write_count"],
            serde_json::json!(0u64),
            "EthernetIP-CIP.pcap: write_count must be 0"
        );
        // HS-110 corroboration: LE-decoded command distribution is consistent.
        assert_eq!(
            enip_summary["command_distribution"]["0x006F"],
            serde_json::json!(438u64),
            "EthernetIP-CIP.pcap: command_distribution[\"0x006F\"] must be 438 (HS-110 LE decode)"
        );
        assert_eq!(
            enip_summary["command_distribution"]["0x0070"],
            serde_json::json!(8361u64),
            "EthernetIP-CIP.pcap: command_distribution[\"0x0070\"] must be 8361 (HS-110 LE decode)"
        );
    }

    // =========================================================================
    // Test 6 — enip_metasploit.pcapng (scy-phy/bro-cip-enip MIT)
    //
    // Pcap: 13 ENIP PDUs (8x RegisterSession + 5x SendRRData), 4 flows.
    // Expected: 0 findings (correct per holdout determination).
    //
    // Holdout: HS-119 (0x00B2-offset-0 scope model) + HS-111/112 determination (b/c).
    //
    // IMPORTANT — this test DOCUMENTS correct non-detection behavior. It is NOT a
    // bug. The Metasploit capture does not fire T0858/T0816 because the module uses
    // non-standard framing:
    //   - Frames 53860/42942/36768: raw-CIP / malformed-CPF (no 0x00B2 item at all,
    //     or item_count field is absurdly large) — determination (c).
    //   - Frame 51310: valid 0x00B2 CPF item, but offset-0 CIP service = 0x52
    //     (Unconnected_Send); Stop 0x07 is *embedded* inside Unconnected_Send and
    //     v0.11.0 does not unwrap embedded messages — determination (b).
    // This is consistent with the ADR-010 / HS-119 0x00B2-offset-0-only detection
    // scope. The test pins this behavior as a regression guard: if T0858/T0816
    // unexpectedly fire in a future version, the test fails and the change must
    // be evaluated against the HS-119 scope model before being accepted.
    //
    // Traces: BC-2.17.019, HS-119, HS-111, HS-112.
    // License: scy-phy/bro-cip-enip MIT.
    // =========================================================================

    /// test_e2e_BC_2_17_enip_metasploit_zero_T0858_T0816_correct_scope
    ///
    /// enip_metasploit.pcapng captures Metasploit `multi_cip_command` issuing
    /// STOPCPU/RESETETHER/CRASHETHER/CRASHCPU against an Allen-Bradley PLC.
    ///
    /// **Zero T0858/T0816 is the CORRECT behavior** (holdout evaluation determination (b)/(c));
    /// see the test comment and holdout-evaluation-report.md for the full byte-exact
    /// proof. The Metasploit module does NOT use spec-compliant 0x00B2 Unconnected
    /// Data Item framing at offset 0 for Stop/Reset services, so the v0.11.0
    /// 0x00B2-offset-0 detection path does not fire.
    ///
    /// Postconditions asserted:
    /// - `enip.all_findings` contains no T0858 or T0816 findings.
    /// - `enip_summary.total_pdu_count` == 13.
    /// - `enip_summary.flows_analyzed` == 4.
    /// - `enip_summary.parse_errors` == 0.
    /// - `enip_summary.error_count` == 0.
    ///
    /// Traces: BC-2.17.019, HS-119, HS-111 determination, HS-112 determination.
    #[test]
    fn test_e2e_BC_2_17_enip_metasploit_zero_T0858_T0816_correct_scope() {
        if !fixture_present("enip_metasploit.pcapng") {
            return;
        }

        let enip = run_enip_pipeline("enip_metasploit.pcapng");

        // ── No T0858 or T0816 findings (correct — HS-119 scope model) ─────────
        //
        // NOTE (HS-119 / determination b/c): If T0858 or T0816 appear here in a
        // future version, stop and evaluate before weakening this assertion.
        // The metasploit capture uses non-standard framing; if these techniques fire
        // it may indicate the detection scope has been expanded to cover Unconnected_Send
        // embedding (determination b, a v0.12.0-class capability) or non-CPF frames
        // (determination c). Document the change in ADR-010 before accepting.
        let t0858_count = enip
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T0858"))
            .count();
        let t0816_count = enip
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T0816"))
            .count();

        assert_eq!(
            t0858_count, 0,
            "enip_metasploit.pcapng: expected 0 T0858 findings (HS-119 / correct non-detection \
             — Metasploit uses non-standard 0x00B2/Unconnected_Send framing; see \
             holdout-evaluation-report.md determination (b)/(c)); got {t0858_count}"
        );
        assert_eq!(
            t0816_count, 0,
            "enip_metasploit.pcapng: expected 0 T0816 findings (HS-119 / correct non-detection \
             — Metasploit RESET frame has ENIP length=8, no valid CPF; see \
             holdout-evaluation-report.md determination (c)); got {t0816_count}"
        );

        // ── enip_summary ──────────────────────────────────────────────────────
        let summary = enip.summarize();
        let enip_summary = &summary.detail["enip_summary"];

        assert_eq!(
            enip_summary["total_pdu_count"],
            serde_json::json!(13u64),
            "enip_metasploit.pcapng: total_pdu_count must be 13 (8 RegisterSession + 5 SendRRData); \
             got {:?}",
            enip_summary["total_pdu_count"]
        );
        assert_eq!(
            enip_summary["flows_analyzed"],
            serde_json::json!(4u64),
            "enip_metasploit.pcapng: flows_analyzed must be 4; got {:?}",
            enip_summary["flows_analyzed"]
        );
        assert_eq!(
            enip_summary["parse_errors"],
            serde_json::json!(0u64),
            "enip_metasploit.pcapng: parse_errors must be 0 — malformed-CPF frames must NOT \
             increment parse_errors (they are valid ENIP commands; only the CIP payload is \
             non-standard); got {:?}",
            enip_summary["parse_errors"]
        );
        assert_eq!(
            enip_summary["error_count"],
            serde_json::json!(0u64),
            "enip_metasploit.pcapng: error_count must be 0"
        );
    }
} // mod enip_e2e_real_pcaps
