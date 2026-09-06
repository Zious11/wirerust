//! Full-pipeline end-to-end tests for the IEC-104 analyzer against real IEC-104 pcaps.
//!
//! Exercises the complete `PcapSource::from_file → decode_packet → TcpReassembler →
//! StreamDispatcher → Iec104Analyzer` pipeline using real-world captures and asserts that
//! the analyzer's outputs match the ground-truth outcomes established during
//! analyzer-level validation (STORY-167..174 e2e coverage pass).
//!
//! ## Fixture management
//!
//! Fixtures resolve from two locations, checked in order: `tests/fixtures/` (committed —
//! always present in any checkout) then `tests/fixtures/local-samples/` (gitignored corpus
//! — see E2E-PCAPS.md). The committed ITI capture (`iec104-iti-diverse.pcap`) is always
//! present and its test always runs in CI; it never takes the skip path. When a gitignored
//! corpus fixture is absent, the affected test prints a skip notice and returns immediately
//! while `test_fixture_manifest_report()` hard-fails if a *committed* fixture goes missing
//! (broken checkout) — see AC-182-005. `#[ignore]` is NOT used.
//!
//! To populate fixtures locally:
//!
//! ```bash
//! bin/fetch-e2e-pcaps
//! ```
//!
//! ## Test cases and pcap mapping
//!
//! | Test | Pcap | What it asserts |
//! |------|------|-----------------|
//! | `test_e2e_BC_2_19_iec104_pcap_T0836_T1692_001_interrogation` | `iec104.pcap` (Wireshark Foundation) | T0836 ×24 + T1692.001 ×42 = 66 total; flows_analyzed=1; dropped_findings=0 |
//! | `test_e2e_BC_2_19_iec104_sq_pcapng_zero_findings_benign_uframes` | `iec104-sq.pcapng` (Wireshark Foundation) | 0 findings (benign STARTDT/TESTFR-only SQ-bit fixture); flows_analyzed=1; dropped_findings=0 |
//! | `test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu` | `iec104-iti-diverse.pcap` (ITI CC-BY-4.0; committed at `tests/fixtures/`) | T0836 ×20 + T1692.001 ×46 = 66 total; flows_analyzed=1; dropped_findings=0 |
//! | `test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage` | `iec104-iti-dissect.pcap` (ITI CC-BY-4.0) | T0814 ×2 + T1692.001 ×9 = 11 total; flows_analyzed=6; dropped_findings=0 |
//!
//! ## Traces
//!
//! BC-2.19 (IEC-104 analyzer pipeline, STORY-167..174); dispatch port 2404 (BC-2.05.012 §P2).
//!
//! Per DF-TEST-NAMESPACE-001: all tests are wrapped in `mod iec104_e2e_real_pcaps`.

#![allow(non_snake_case)]

mod iec104_e2e_real_pcaps {
    use std::path::Path;

    use wirerust::analyzer::iec104::Iec104Analyzer;
    use wirerust::decoder::{DecodedFrame, decode_packet};
    use wirerust::dispatcher::StreamDispatcher;
    use wirerust::reader::PcapSource;
    use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

    // -------------------------------------------------------------------------
    // Fixture roots — committed captures resolve from `tests/fixtures/` first (always
    // present in any checkout); the gitignored corpus resolves from
    // `tests/fixtures/local-samples/` as the secondary search location.
    // -------------------------------------------------------------------------

    const LOCAL_SAMPLES: &str = "tests/fixtures/local-samples";
    const COMMITTED_SAMPLES: &str = "tests/fixtures";

    // -------------------------------------------------------------------------
    // STORY-182 fixture manifest (AC-182-001/002/003/005) — dual-location scheme:
    // `tests/fixtures/` for committed captures (always present in CI), and
    // `tests/fixtures/local-samples/` for the gitignored corpus (absent in clean
    // checkouts). See `fixture_path()` below for the shared resolver.
    // -------------------------------------------------------------------------

    /// All 4 fixture filenames used by the fixture-gated tests in this module.
    const FIXTURE_MANIFEST: &[&str] = &[
        "iec104.pcap",
        "iec104-sq.pcapng",
        "iec104-iti-diverse.pcap",
        "iec104-iti-dissect.pcap",
    ];

    /// Fixtures committed directly to `tests/fixtures/` (redistributable license,
    /// no positive evidence of third-party-of-ITI origin — F-009 D-524 ruling).
    /// `iec104-iti-dissect.pcap` is NOT committed — POSITIVE EVIDENCE OF
    /// UPSTREAM-OF-ITI ORIGIN (upstream filename `TestDissectIec104.pcap` +
    /// E2E-PCAPS.md "Wireshark-dissector test capture"); stays gitignored.
    const COMMITTED_FIXTURES: &[&str] = &["iec104-iti-diverse.pcap"];

    /// Maps every fixture-gated test function to its fixture filename. New tests
    /// calling `fixture_present()` MUST register here.
    const FIXTURE_GATED_TESTS: &[(&str, &str)] = &[
        (
            "test_e2e_BC_2_19_iec104_pcap_T0836_T1692_001_interrogation",
            "iec104.pcap",
        ),
        (
            "test_e2e_BC_2_19_iec104_sq_pcapng_zero_findings_benign_uframes",
            "iec104-sq.pcapng",
        ),
        (
            "test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu",
            "iec104-iti-diverse.pcap",
        ),
        (
            "test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage",
            "iec104-iti-dissect.pcap",
        ),
    ];

    /// Resolve the path of a fixture file, checking `tests/fixtures/` (committed) before
    /// `tests/fixtures/local-samples/` (gitignored corpus).
    ///
    /// Returns `Some(path)` if the file exists in either location, `None` if absent in both.
    fn fixture_path(filename: &str) -> Option<std::path::PathBuf> {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"));
        let committed = base.join(COMMITTED_SAMPLES).join(filename);
        if committed.exists() {
            return Some(committed);
        }
        let local = base.join(LOCAL_SAMPLES).join(filename);
        if local.exists() {
            return Some(local);
        }
        None
    }

    // -------------------------------------------------------------------------
    // Skip-if-absent guard (mirrors enip_e2e_real_pcaps_tests.rs pattern)
    //
    // Returns true if the fixture is present, false if the test should be skipped.
    // Uses the shared `fixture_path()` resolver: committed captures in
    // `tests/fixtures/` (always present in CI) are checked before the gitignored
    // corpus in `tests/fixtures/local-samples/` (absent in clean checkouts).
    // -------------------------------------------------------------------------

    /// Check whether a fixture file is present via the shared `fixture_path()` resolver.
    ///
    /// When the file is absent from both search locations, the caller prints a skip
    /// notice and returns early. Committed-eligible fixtures never take this path in a
    /// clean checkout (they always resolve from `tests/fixtures/`); only the gitignored
    /// corpus fixtures can legitimately be missing.
    fn fixture_present(filename: &str) -> bool {
        match fixture_path(filename) {
            Some(_) => true,
            None => {
                let base = Path::new(env!("CARGO_MANIFEST_DIR"));
                if COMMITTED_FIXTURES.contains(&filename) {
                    // Committed-eligible fixture: show the committed path in the diagnostic.
                    // Absence here means a broken checkout — the hard-assert in
                    // test_fixture_manifest_report() catches this at test time.
                    let committed_path = base.join(COMMITTED_SAMPLES).join(filename);
                    eprintln!(
                        "[iec104-e2e] SKIP: fixture '{}' not found at {} (or in local-samples). \
                         Run `bin/fetch-e2e-pcaps` to populate local-samples.",
                        filename,
                        committed_path.display()
                    );
                } else {
                    // Gitignored corpus fixture (Wireshark "not redistributed" or
                    // origin-unclear — see COMMITTED_FIXTURES and module docs above).
                    // Show local-samples path; do NOT suggest tests/fixtures/ as a target.
                    let local_path = base.join(LOCAL_SAMPLES).join(filename);
                    eprintln!(
                        "[iec104-e2e] SKIP: fixture '{}' not found at {} \
                         (do not commit to tests/fixtures/ — licensing/redistribution \
                          constraint; run `bin/fetch-e2e-pcaps` to populate local-samples).",
                        filename,
                        local_path.display()
                    );
                }
                false
            }
        }
    }

    // -------------------------------------------------------------------------
    // Pipeline helper — run the full reader → reassembler → dispatcher → IEC-104
    // pipeline on `filename`, return the Iec104Analyzer (which owns all_findings).
    //
    // Mirrors the `run_analyze` pipeline in `src/main.rs` and the pattern from
    // `tests/enip_e2e_real_pcaps_tests.rs`.
    // -------------------------------------------------------------------------

    /// Run the full IEC-104 analysis pipeline on a pcap/pcapng file.
    ///
    /// The file is resolved via the shared `fixture_path()` resolver — committed captures
    /// under `tests/fixtures/` or gitignored corpus under `tests/fixtures/local-samples/`.
    /// Panics if the resolver returns `None` (a broken pre-condition: the caller must have
    /// verified presence via `fixture_present()` first — this is not a skip condition) or if
    /// `PcapSource::from_file` fails (the fixture was present but unreadable, which is a
    /// test infrastructure failure, not a skip condition).
    ///
    /// Returns the `Iec104Analyzer` after `reassembler.finalize(&mut dispatcher)` so that
    /// all per-flow state has been flushed and `on_flow_close` has been called for every
    /// completed stream, making `summarize()` return the full aggregate.
    fn run_iec104_pipeline(filename: &str) -> Iec104Analyzer {
        let path = fixture_path(filename).unwrap_or_else(|| {
            panic!(
                "[iec104-e2e] fixture_path returned None for '{}' — \
                 fixture_present() must be called before run_iec104_pipeline()",
                filename
            )
        });

        let source = PcapSource::from_file(&path)
            .unwrap_or_else(|e| panic!("[iec104-e2e] failed to open {filename}: {e:#}"));

        let config = ReassemblyConfig::default();
        let mut reassembler = TcpReassembler::new(config);
        // IEC-104 is the 6th (last) slot in StreamDispatcher::new.
        // All other analyzers are None — this test exercises ONLY the IEC-104 path.
        let mut dispatcher =
            StreamDispatcher::new(None, None, None, None, None, Some(Iec104Analyzer::new()));

        for raw in &source.packets {
            if let Ok(DecodedFrame::Ip(parsed)) = decode_packet(&raw.data, source.datalink) {
                reassembler.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher);
            }
        }

        // Flush any open flows (calls on_flow_close for each, folding per-flow frame counts
        // into the aggregate fields read by summarize()).
        reassembler.finalize(&mut dispatcher);

        // Take ownership of the IEC-104 analyzer out of the dispatcher.
        dispatcher
            .take_iec104_analyzer()
            .expect("[iec104-e2e] IEC-104 analyzer must be present after run_iec104_pipeline")
    }

    // =========================================================================
    // Test 1 — iec104.pcap (Wireshark Foundation; public sample; not redistributed)
    //
    // Pcap: canonical Wireshark IEC-104 reference. U-frames (STARTDT/STOPDT/TESTFR) +
    //       I-frame ASDUs + C_IC general interrogation (TypeID 100, COT 6/7/20/10).
    //       105 reader packets; 1 TCP flow on port 2404.
    // Expected: 66 findings = T0836 ×24 + T1692.001 ×42. All Impact/Possible/Medium.
    //           flows_analyzed=1, total_findings=66, dropped_findings=0.
    //
    // Traces: BC-2.19 (STORY-167..174 IEC-104 analyzer pipeline).
    // License: Wireshark Foundation public sample; no per-file license; not redistributed.
    // =========================================================================

    /// test_e2e_BC_2_19_iec104_pcap_T0836_T1692_001_interrogation
    ///
    /// iec104.pcap contains a real IEC-104 session: link-management U-frames
    /// (STARTDT/STOPDT/TESTFR) followed by I-frame ASDUs carrying control commands
    /// and a general-interrogation (C_IC TypeID 100) request/response lifecycle.
    ///
    /// The IEC-104 analyzer fires:
    /// - T1692.001 ×42: control-command TypeID/COT-flagged I-frames
    /// - T0836 ×24: parameter-modification / data-historian events
    ///
    /// All 66 findings are Impact/Possible/Medium (consistent with passive-monitor
    /// detection of command traffic without out-of-band confirmation).
    ///
    /// Postconditions asserted (ground-truth from analyzer-level validation run):
    /// - `iec104.all_findings.len()` == 66.
    /// - T0836 count == 24.
    /// - T1692.001 count == 42.
    /// - Every finding is Impact / Possible / Medium.
    /// - `iec104_summary.flows_analyzed` == 1.
    /// - `iec104_summary.total_findings` == 66.
    /// - `iec104_summary.dropped_findings` == 0.
    ///
    /// Traces: BC-2.19 (IEC-104 analyzer pipeline).
    #[test]
    fn test_e2e_BC_2_19_iec104_pcap_T0836_T1692_001_interrogation() {
        if !fixture_present("iec104.pcap") {
            return;
        }

        let iec104 = run_iec104_pipeline("iec104.pcap");

        // ── Total findings count ──────────────────────────────────────────────
        assert_eq!(
            iec104.all_findings.len(),
            66,
            "iec104.pcap: expected exactly 66 findings (T0836 ×24 + T1692.001 ×42); \
             got {} findings: {:?}",
            iec104.all_findings.len(),
            iec104
                .all_findings
                .iter()
                .map(|f| f.mitre_techniques.as_slice())
                .collect::<Vec<_>>()
        );

        // ── Per-technique counts ──────────────────────────────────────────────
        let t0836_count = iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T0836"))
            .count();
        let t1692_001_count = iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"))
            .count();

        assert_eq!(
            t0836_count, 24,
            "iec104.pcap: expected 24 T0836 findings; got {t0836_count}"
        );
        assert_eq!(
            t1692_001_count, 42,
            "iec104.pcap: expected 42 T1692.001 findings; got {t1692_001_count}"
        );

        // ── Category / verdict / confidence — all Impact/Possible/Medium ──────
        for (i, f) in iec104.all_findings.iter().enumerate() {
            assert_eq!(
                format!("{:?}", f.category),
                "Impact",
                "iec104.pcap finding[{i}]: expected category=Impact; got {:?}",
                f.category
            );
            assert_eq!(
                format!("{}", f.verdict),
                "POSSIBLE",
                "iec104.pcap finding[{i}]: expected verdict=Possible; got {:?}",
                f.verdict
            );
            assert_eq!(
                format!("{}", f.confidence),
                "MEDIUM",
                "iec104.pcap finding[{i}]: expected confidence=Medium; got {:?}",
                f.confidence
            );
        }

        // ── Analyzer summary ──────────────────────────────────────────────────
        let summary = iec104.summarize();
        assert_eq!(
            summary.analyzer_name, "IEC-104",
            "iec104.pcap: analyzer_name must be 'IEC-104'"
        );

        let detail = &summary.detail;

        assert_eq!(
            detail["flows_analyzed"],
            serde_json::json!(1u64),
            "iec104.pcap: flows_analyzed must be 1; got {:?}",
            detail["flows_analyzed"]
        );
        assert_eq!(
            detail["total_findings"],
            serde_json::json!(66u64),
            "iec104.pcap: total_findings must be 66; got {:?}",
            detail["total_findings"]
        );
        assert_eq!(
            detail["dropped_findings"],
            serde_json::json!(0u64),
            "iec104.pcap: dropped_findings must be 0 (no cap overflow); got {:?}",
            detail["dropped_findings"]
        );
    }

    // =========================================================================
    // Test 2 — iec104-sq.pcapng (Wireshark Foundation; public sample; not redistributed)
    //
    // Pcap: native pcapng (SHB magic 0x0A0D0D0A); SQ-bit set in ASDU variable-structure
    //       qualifier (sequence-of-information-objects encoding). 1 reader packet; 1 flow.
    // Expected: 0 findings (benign link-management traffic only — no attack techniques).
    //           flows_analyzed=1, total_findings=0, dropped_findings=0.
    //
    // This fixture exercises:
    //   (a) the pcapng reader with a native IEC-104 pcapng file, and
    //   (b) the SQ-bit ASDU parsing path in the IEC-104 analyzer.
    //
    // Zero findings confirms no false positives on STARTDT/TESTFR control frames.
    //
    // Traces: BC-2.19 (IEC-104 analyzer pipeline).
    // License: Wireshark Foundation public sample; no per-file license; not redistributed.
    // =========================================================================

    /// test_e2e_BC_2_19_iec104_sq_pcapng_zero_findings_benign_uframes
    ///
    /// iec104-sq.pcapng is a 584-byte native pcapng file containing a single IEC-104
    /// session packet with the SQ bit set. The content is benign link-management traffic
    /// (STARTDT/TESTFR U-frames or their acknowledgements) — no control commands, no
    /// parameter modifications, no data injection.
    ///
    /// Postconditions asserted:
    /// - `iec104.all_findings` is empty (0 findings — no false positives).
    /// - `iec104_summary.flows_analyzed` == 1.
    /// - `iec104_summary.total_findings` == 0.
    /// - `iec104_summary.dropped_findings` == 0.
    ///
    /// Traces: BC-2.19 (IEC-104 analyzer pipeline).
    #[test]
    fn test_e2e_BC_2_19_iec104_sq_pcapng_zero_findings_benign_uframes() {
        if !fixture_present("iec104-sq.pcapng") {
            return;
        }

        let iec104 = run_iec104_pipeline("iec104-sq.pcapng");

        // ── Zero findings (no false positives on benign SQ-bit traffic) ───────
        assert!(
            iec104.all_findings.is_empty(),
            "iec104-sq.pcapng: expected 0 findings (benign STARTDT/TESTFR/SQ-bit only); \
             got {} findings: {:?}",
            iec104.all_findings.len(),
            iec104
                .all_findings
                .iter()
                .map(|f| f.mitre_techniques.as_slice())
                .collect::<Vec<_>>()
        );

        // ── Analyzer summary ──────────────────────────────────────────────────
        let summary = iec104.summarize();
        let detail = &summary.detail;

        assert_eq!(
            detail["flows_analyzed"],
            serde_json::json!(1u64),
            "iec104-sq.pcapng: flows_analyzed must be 1; got {:?}",
            detail["flows_analyzed"]
        );
        assert_eq!(
            detail["total_findings"],
            serde_json::json!(0u64),
            "iec104-sq.pcapng: total_findings must be 0; got {:?}",
            detail["total_findings"]
        );
        assert_eq!(
            detail["dropped_findings"],
            serde_json::json!(0u64),
            "iec104-sq.pcapng: dropped_findings must be 0; got {:?}",
            detail["dropped_findings"]
        );
    }

    // =========================================================================
    // Test 3 — iec104-iti-diverse.pcap (ITI/ICS-Security-Tools CC-BY-4.0)
    //
    // Pcap: IEC-104 traffic with a diverse mix of ASDU Type IDs from the ITI ICS corpus.
    //       173 reader packets; 1 TCP flow on port 2404.
    // Expected: 66 findings = T0836 ×20 + T1692.001 ×46. All Impact/Possible/Medium.
    //           flows_analyzed=1, total_findings=66, dropped_findings=0.
    //
    // Wave-85 change (STORY-180, BC-2.19.029/030): TypeIDs 58–64 (time-tagged control
    // commands) present in this capture were silently ignored before wave-85. They are now
    // detected, raising the total from 31 to 66 (+35 findings). The untimed contribution
    // (31) is unchanged: TypeID=45→5×T1692.001, TypeID=46→6×T1692.001,
    // TypeID=50→10×T0836+10×T1692.001.
    // Timed contribution (+35): x=15 timed-switching ASDUs (TypeID=58→5, TypeID=59→10,
    // each 1 finding → +15×T1692.001); y=10 timed-setpoint ASDUs (TypeID=61→5,
    // TypeID=63→5, each 2 findings → +10×T1692.001 + +10×T0836); x+2y=35. ✓
    //
    // Traces: BC-2.19 (IEC-104 analyzer pipeline).
    // License: ITI/ICS-Security-Tools CC-BY-4.0. Attribution: ICS Security Tools,
    //          Illinois Institute of Technology (ITI).
    // Capture committed to tests/fixtures/ — always available in CI (STORY-182).
    // =========================================================================

    /// test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu
    ///
    /// iec104-iti-diverse.pcap is from the same ITI ICS Security Tools corpus as the ENIP
    /// fixtures. It contains a diverse ASDU Type ID mix representing realistic IEC-104
    /// traffic from a SCADA deployment, including time-tagged control commands.
    ///
    /// Wave-85 (STORY-180, BC-2.19.029/030): TypeIDs 58–64 (time-tagged control commands)
    /// present in this capture were silently ignored before wave-85; they are now detected,
    /// raising the expectation from 31 to 66. The +35 timed findings decompose as:
    ///   x=15 from timed-switching TypeIDs 58–59 (1 finding each, T1692.001 only)
    ///   2y=20 from timed-setpoint TypeIDs 61+63 (2 findings each: T1692.001 + T0836)
    ///   T0836 delta=y=10; T1692.001 delta=x+y=25
    ///
    /// Postconditions asserted (ground-truth from wave-85 validation run):
    /// - `iec104.all_findings.len()` == 66.
    /// - T0836 count == 20.
    /// - T1692.001 count == 46.
    /// - Count of findings whose summary contains "time-tagged" == 35 (= x + 2y).
    /// - Every finding is Impact / Possible / Medium.
    /// - `iec104_summary.flows_analyzed` == 1.
    /// - `iec104_summary.total_findings` == 66.
    /// - `iec104_summary.dropped_findings` == 0.
    ///
    /// Traces: BC-2.19 (IEC-104 analyzer pipeline).
    #[test]
    fn test_e2e_BC_2_19_iec104_iti_diverse_T0836_T1692_001_mixed_asdu() {
        if !fixture_present("iec104-iti-diverse.pcap") {
            return;
        }

        let iec104 = run_iec104_pipeline("iec104-iti-diverse.pcap");

        // ── Total findings count ──────────────────────────────────────────────
        assert_eq!(
            iec104.all_findings.len(),
            66,
            "iec104-iti-diverse.pcap: expected exactly 66 findings \
             (T0836 ×20 + T1692.001 ×46, incl. 35 time-tagged from wave-85); \
             got {} findings: {:?}",
            iec104.all_findings.len(),
            iec104
                .all_findings
                .iter()
                .map(|f| f.mitre_techniques.as_slice())
                .collect::<Vec<_>>()
        );

        // ── Per-technique counts ──────────────────────────────────────────────
        let t0836_count = iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T0836"))
            .count();
        let t1692_001_count = iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"))
            .count();

        assert_eq!(
            t0836_count, 20,
            "iec104-iti-diverse.pcap: expected 20 T0836 findings \
             (10 untimed from TypeID=50 + 10 timed from TypeIDs 61+63); got {t0836_count}"
        );
        assert_eq!(
            t1692_001_count, 46,
            "iec104-iti-diverse.pcap: expected 46 T1692.001 findings \
             (21 untimed + 25 timed: x=15 from TypeIDs 58-59, y=10 from TypeIDs 61+63); \
             got {t1692_001_count}"
        );

        // ── Time-tagged timed-arm marker (wave-85 STORY-180 guard) ───────────
        // x=15 timed-switching findings (TypeIDs 58-59) + 2y=20 timed-setpoint findings
        // (TypeIDs 61+63) = 35 findings whose summary contains "time-tagged".
        let time_tagged_count = iec104
            .all_findings
            .iter()
            .filter(|f| f.summary.contains("time-tagged"))
            .count();
        assert_eq!(
            time_tagged_count, 35,
            "iec104-iti-diverse.pcap: expected 35 time-tagged findings (x+2y = 15+20, \
             TypeIDs 58–59 + 61+63; BC-2.19.029/030 timed-command detection from wave-85); \
             got {time_tagged_count}"
        );

        // ── Category / verdict / confidence — all Impact/Possible/Medium ──────
        for (i, f) in iec104.all_findings.iter().enumerate() {
            assert_eq!(
                format!("{:?}", f.category),
                "Impact",
                "iec104-iti-diverse.pcap finding[{i}]: expected category=Impact; got {:?}",
                f.category
            );
            assert_eq!(
                format!("{}", f.verdict),
                "POSSIBLE",
                "iec104-iti-diverse.pcap finding[{i}]: expected verdict=Possible; got {:?}",
                f.verdict
            );
            assert_eq!(
                format!("{}", f.confidence),
                "MEDIUM",
                "iec104-iti-diverse.pcap finding[{i}]: expected confidence=Medium; got {:?}",
                f.confidence
            );
        }

        // ── Analyzer summary ──────────────────────────────────────────────────
        let summary = iec104.summarize();
        let detail = &summary.detail;

        assert_eq!(
            detail["flows_analyzed"],
            serde_json::json!(1u64),
            "iec104-iti-diverse.pcap: flows_analyzed must be 1; got {:?}",
            detail["flows_analyzed"]
        );
        assert_eq!(
            detail["total_findings"],
            serde_json::json!(66u64),
            "iec104-iti-diverse.pcap: total_findings must be 66; got {:?}",
            detail["total_findings"]
        );
        assert_eq!(
            detail["dropped_findings"],
            serde_json::json!(0u64),
            "iec104-iti-diverse.pcap: dropped_findings must be 0; got {:?}",
            detail["dropped_findings"]
        );
    }

    // =========================================================================
    // Test 4 — iec104-iti-dissect.pcap (ITI/ICS-Security-Tools CC-BY-4.0)
    //
    // Pcap: Wireshark-dissector test capture — constructed to exercise broad Type ID /
    //       COT coverage incl. control commands (C_SC/C_DC/C_SE). 147 reader packets;
    //       6 TCP flows on port 2404.
    // Expected: 11 findings = T0814 ×2 + T1692.001 ×9.
    //           T0814: Anomaly/Possible/Medium; T1692.001: Impact/Possible/Medium.
    //           flows_analyzed=6, total_findings=11, dropped_findings=0.
    //
    // The 6 flows reflect multiple independent IEC-104 sessions used to cover different
    // Type ID families in a single dissector-test capture.
    //
    // Traces: BC-2.19 (IEC-104 analyzer pipeline).
    // License: ITI/ICS-Security-Tools CC-BY-4.0. Attribution: ICS Security Tools,
    //          Illinois Institute of Technology (ITI).
    // =========================================================================

    /// test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage
    ///
    /// iec104-iti-dissect.pcap is a Wireshark-dissector test capture deliberately
    /// constructed to exercise a broad sweep of IEC-104 Type IDs and Causes of
    /// Transmission, including control commands (C_SC/C_DC/C_SE). It spans 6 TCP
    /// flows, reflecting multi-session coverage of distinct Type ID families.
    ///
    /// The IEC-104 analyzer fires:
    /// - T1692.001 ×9: control-command TypeID detections (Impact/Possible/Medium)
    /// - T0814 ×2: unexpected/spoofed message detections (Anomaly/Possible/Medium)
    ///
    /// Postconditions asserted (ground-truth from analyzer-level validation run):
    /// - `iec104.all_findings.len()` == 11.
    /// - T0814 count == 2; every T0814 finding is Anomaly / Possible / Medium.
    /// - T1692.001 count == 9; every T1692.001 finding is Impact / Possible / Medium.
    /// - `iec104_summary.flows_analyzed` == 6.
    /// - `iec104_summary.total_findings` == 11.
    /// - `iec104_summary.dropped_findings` == 0.
    ///
    /// Traces: BC-2.19 (IEC-104 analyzer pipeline).
    #[test]
    fn test_e2e_BC_2_19_iec104_iti_dissect_T0814_T1692_001_control_coverage() {
        if !fixture_present("iec104-iti-dissect.pcap") {
            return;
        }

        let iec104 = run_iec104_pipeline("iec104-iti-dissect.pcap");

        // ── Total findings count ──────────────────────────────────────────────
        assert_eq!(
            iec104.all_findings.len(),
            11,
            "iec104-iti-dissect.pcap: expected exactly 11 findings \
             (T0814 ×2 + T1692.001 ×9); got {} findings: {:?}",
            iec104.all_findings.len(),
            iec104
                .all_findings
                .iter()
                .map(|f| f.mitre_techniques.as_slice())
                .collect::<Vec<_>>()
        );

        // ── Per-technique counts ──────────────────────────────────────────────
        let t0814_count = iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .count();
        let t1692_001_count = iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"))
            .count();

        assert_eq!(
            t0814_count, 2,
            "iec104-iti-dissect.pcap: expected 2 T0814 findings; got {t0814_count}"
        );
        assert_eq!(
            t1692_001_count, 9,
            "iec104-iti-dissect.pcap: expected 9 T1692.001 findings; got {t1692_001_count}"
        );

        // ── T0814 findings: Anomaly / Possible / Medium ───────────────────────
        for (i, f) in iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .enumerate()
        {
            assert_eq!(
                format!("{:?}", f.category),
                "Anomaly",
                "iec104-iti-dissect.pcap T0814 finding[{i}]: expected category=Anomaly; \
                 got {:?}",
                f.category
            );
            assert_eq!(
                format!("{}", f.verdict),
                "POSSIBLE",
                "iec104-iti-dissect.pcap T0814 finding[{i}]: expected verdict=Possible; \
                 got {:?}",
                f.verdict
            );
            assert_eq!(
                format!("{}", f.confidence),
                "MEDIUM",
                "iec104-iti-dissect.pcap T0814 finding[{i}]: expected confidence=Medium; \
                 got {:?}",
                f.confidence
            );
        }

        // ── T1692.001 findings: Impact / Possible / Medium ────────────────────
        for (i, f) in iec104
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"))
            .enumerate()
        {
            assert_eq!(
                format!("{:?}", f.category),
                "Impact",
                "iec104-iti-dissect.pcap T1692.001 finding[{i}]: expected category=Impact; \
                 got {:?}",
                f.category
            );
            assert_eq!(
                format!("{}", f.verdict),
                "POSSIBLE",
                "iec104-iti-dissect.pcap T1692.001 finding[{i}]: expected verdict=Possible; \
                 got {:?}",
                f.verdict
            );
            assert_eq!(
                format!("{}", f.confidence),
                "MEDIUM",
                "iec104-iti-dissect.pcap T1692.001 finding[{i}]: expected confidence=Medium; \
                 got {:?}",
                f.confidence
            );
        }

        // ── Analyzer summary ──────────────────────────────────────────────────
        let summary = iec104.summarize();
        let detail = &summary.detail;

        assert_eq!(
            detail["flows_analyzed"],
            serde_json::json!(6u64),
            "iec104-iti-dissect.pcap: flows_analyzed must be 6 (6 TCP sessions); \
             got {:?}",
            detail["flows_analyzed"]
        );
        assert_eq!(
            detail["total_findings"],
            serde_json::json!(11u64),
            "iec104-iti-dissect.pcap: total_findings must be 11; got {:?}",
            detail["total_findings"]
        );
        assert_eq!(
            detail["dropped_findings"],
            serde_json::json!(0u64),
            "iec104-iti-dissect.pcap: dropped_findings must be 0; got {:?}",
            detail["dropped_findings"]
        );
    }

    // =========================================================================
    // Test 5 — test_fixture_manifest_report (STORY-182, AC-182-001 + AC-182-005)
    //
    // FIXTURE_MANIFEST / COMMITTED_FIXTURES / FIXTURE_GATED_TESTS are populated per
    // AC-182-001/005, and `tests/fixtures/iec104-iti-diverse.pcap` is committed per
    // AC-182-002. In a clean checkout this test PASSES (committed fixture present,
    // manifest/registry consistent); it FAILS via hard-assert if the committed
    // fixture is ever absent (broken checkout — see AC-182-005).
    // =========================================================================

    /// test_fixture_manifest_report
    ///
    /// Combines the AC-182-001 skip-reporting half (advisory `println!()` coverage summary
    /// and per-fixture `FIXTURE-SKIPPED` notices, visible only with `--nocapture`) with the
    /// AC-182-005 hard-assert half (committed-fixture presence, manifest/registry
    /// consistency, resolver coupling, and the forbidden-committed negative guard — all of
    /// which ARE visible in standard CI output because panics bypass stdout capture).
    ///
    /// Traces: PG-W85-005 / AC-182-001 / AC-182-005.
    #[test]
    fn test_fixture_manifest_report() {
        // ---------------------------------------------------------------------
        // AC-182-001: skip-reporting half (advisory; --nocapture only)
        // ---------------------------------------------------------------------
        let present: Vec<&str> = FIXTURE_MANIFEST
            .iter()
            .copied()
            .filter(|n| fixture_path(n).is_some())
            .collect();
        let absent: Vec<&str> = FIXTURE_MANIFEST
            .iter()
            .copied()
            .filter(|n| fixture_path(n).is_none())
            .collect();
        // Advisory stdout: visible with --nocapture only; not visible in standard CI output
        println!(
            "Fixture coverage: {}/{} fixtures present ({} fixture-gated tests will be skipped)",
            present.len(),
            FIXTURE_MANIFEST.len(),
            absent.len()
        );
        for name in &absent {
            println!(
                "FIXTURE-SKIPPED: '{}' absent — corpus test will not run \
                 (check tests/fixtures/ for committed or tests/fixtures/local-samples/ for corpus)",
                name
            );
        }

        // ---------------------------------------------------------------------
        // AC-182-005: hard-assert partition (committed/tracked fixtures MUST be present)
        // ---------------------------------------------------------------------

        // Hard assert: committed (tracked) fixtures MUST be present in tests/fixtures/ directly.
        // Using Path::exists() on the committed path, NOT fixture_path() which also checks
        // local-samples/ — the direct check works correctly regardless of local-samples presence.
        // This panic IS always visible in CI output regardless of --nocapture (assertion failure).
        for name in COMMITTED_FIXTURES {
            assert!(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures")
                    .join(name)
                    .exists(),
                "[iec104-e2e] REGRESSION: committed fixture '{}' is absent from \
                 tests/fixtures/ — this is a broken checkout. \
                 Run `git checkout tests/fixtures/` to restore.",
                name
            );
        }

        // FIXTURE_MANIFEST superset check (F-013 canonical location — was in AC-182-001):
        // Guards against a committed fixture being removed from the manifest (drift).
        for name in COMMITTED_FIXTURES {
            assert!(
                FIXTURE_MANIFEST.contains(name),
                "FIXTURE_MANIFEST does not contain committed fixture '{}' — \
                 update FIXTURE_MANIFEST to include all entries from COMMITTED_FIXTURES",
                name
            );
        }

        // Manifest-size drift pin (fires on manifest growth/shrink only):
        // This len() check fires when FIXTURE_MANIFEST gains or loses entries.
        // It does NOT detect an unregistered test silently using a fixture not in the manifest.
        // FIXTURE_GATED_TESTS registry (module-level) catches renames of registered tests;
        // the fixture_present() call-site count assertion below catches unregistered additions.
        // Co-update loci when fixtures are added: FIXTURE_MANIFEST, this assertion,
        // ci.yml "Fixture coverage: [1-9][0-9]*/[0-9]+" step, Task 9 Env A 4/4 expected value,
        // and the "1/4" expected-value literals (Env B blocks, Task 8 obligation, EC rows).
        assert_eq!(
            FIXTURE_MANIFEST.len(),
            4,
            "FIXTURE_MANIFEST.len() must equal the count of distinct fixture names used by \
             fixture-gated tests (currently 4: 1 committed ITI + 3 gitignored (2 Wireshark + 1 ITI)); \
             update FIXTURE_MANIFEST, this assertion, ci.yml coverage step, and Task 9 Env A \
             4/4 expected value together when new fixtures are added"
        );

        // Fixture-gated test registry assertion — BIDIRECTIONAL set equality:
        // Direction 1 (gated ⊆ manifest): every FIXTURE_GATED_TESTS entry's fixture_name
        // must be in FIXTURE_MANIFEST. Registered entries with wrong names fail here.
        for (_, fixture_name) in FIXTURE_GATED_TESTS {
            assert!(
                FIXTURE_MANIFEST.contains(fixture_name),
                "FIXTURE_GATED_TESTS entry '{}' is not in FIXTURE_MANIFEST — \
                 update FIXTURE_MANIFEST to include it, or correct this registry entry",
                fixture_name
            );
        }
        // Direction 2 (manifest ⊆ gated): every FIXTURE_MANIFEST entry must be exercised
        // by at least one FIXTURE_GATED_TESTS entry. Catches fixtures added to the manifest
        // that no test actually gates on — a manifest entry with no gated test is dead weight.
        for manifest_name in FIXTURE_MANIFEST {
            assert!(
                FIXTURE_GATED_TESTS.iter().any(|(_, f)| f == manifest_name),
                "FIXTURE_MANIFEST entry '{}' is not exercised by any FIXTURE_GATED_TESTS entry — \
                 add a registry entry for the test that uses it, or remove it from FIXTURE_MANIFEST",
                manifest_name
            );
        }

        // FIXTURE_GATED_TESTS count pin (update when a new fixture-gated test is added):
        assert_eq!(
            FIXTURE_GATED_TESTS.len(),
            4,
            "FIXTURE_GATED_TESTS.len() must equal the count of fixture-gated tests (currently 4); \
             update FIXTURE_GATED_TESTS and this assertion together when tests are added or removed"
        );

        // Per-test function-name coupling: reads the harness source file at test time and asserts
        // each registered name exists as `fn <name>` in the source.
        // NON-SELF-REFERENTIAL: the predicate checks for `fn test_name` (present only at the
        // function-definition site), not merely `test_name` (which also appears inside the
        // FIXTURE_GATED_TESTS string literal). If a test is renamed but FIXTURE_GATED_TESTS is
        // not updated, `fn <old_name>` will not be found in source → assertion fails.
        // This predicate CAN fail: the fn-definition span and the FIXTURE_GATED_TESTS string
        // literal are different text, so renaming a test without updating the registry produces
        // a genuine test failure (not a vacuous pass).
        let harness_src = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/iec104_e2e_real_pcaps_tests.rs"),
        )
        .expect(
            "[iec104-e2e] failed to read harness source for FIXTURE_GATED_TESTS coupling check",
        );
        for (test_name, _) in FIXTURE_GATED_TESTS {
            assert!(
                harness_src.contains(&format!("fn {}", test_name)),
                "FIXTURE_GATED_TESTS entry '{}' has no matching `fn {}` definition in \
                 tests/iec104_e2e_real_pcaps_tests.rs — the test was renamed or removed; \
                 update FIXTURE_GATED_TESTS accordingly",
                test_name,
                test_name
            );
        }

        // fixture_path() resolver coupling (F-001):
        // A mistyped COMMITTED_SAMPLES const causes this check to fail in CI; an inverted
        // ordering in fixture_path() is caught only on a fixture-bearing host (Task 9 Env A),
        // because local-samples/ is absent in CI.
        // Uses parent() equality (NOT starts_with) to catch local-samples/ as a subdir:
        // starts_with(tests/fixtures/) would pass for tests/fixtures/local-samples/foo.pcap
        // because local-samples/ is a subdirectory of tests/fixtures/. parent() equality
        // asserts the file is DIRECTLY in tests/fixtures/, not in any subdirectory.
        for name in COMMITTED_FIXTURES {
            let resolved = fixture_path(name).unwrap_or_else(|| {
                panic!(
                    "[iec104-e2e] fixture_path('{}') returned None for a COMMITTED_FIXTURES entry — \
                     COMMITTED_SAMPLES resolver is broken or the ordering in fixture_path() is inverted",
                    name
                )
            });
            let committed_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
            assert_eq!(
                resolved.parent(),
                Some(committed_dir.as_path()),
                "[iec104-e2e] fixture_path('{}') resolved to {:?} — parent dir must be \
                 tests/fixtures/ exactly (not tests/fixtures/local-samples/ or any other \
                 subdirectory); COMMITTED_SAMPLES ordering may be inverted or const is wrong",
                name,
                resolved
            );
        }

        // Forbidden-committed negative guard (F-P10-005):
        // For every FIXTURE_MANIFEST entry NOT in COMMITTED_FIXTURES, assert it is absent
        // from tests/fixtures/ (the committed path). Catches accidental commits of
        // non-redistributable or origin-unclear captures (Wireshark "not redistributed",
        // iec104-iti-dissect.pcap Wireshark dissector test suite origin — F-009 D-524 ruling).
        // Fails exactly when a forbidden capture is dropped in tests/fixtures/.
        for name in FIXTURE_MANIFEST {
            if !COMMITTED_FIXTURES.contains(name) {
                let forbidden_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures")
                    .join(name);
                assert!(
                    !forbidden_path.exists(),
                    "[iec104-e2e] LICENSING/REDISTRIBUTION VIOLATION: '{}' is present in \
                     tests/fixtures/ but is NOT in COMMITTED_FIXTURES — this file MUST NOT \
                     be committed; Wireshark captures and origin-unclear files are prohibited \
                     from redistribution (see Background §Must NOT commit). Remove the file \
                     from tests/fixtures/ and place it only in tests/fixtures/local-samples/.",
                    name
                );
            }
        }

        // Call-site count assertion (F-P10-009 / F-P11-001):
        // Counts call sites for the fixture-gating function. The needle is built via concat!
        // so this file's own prose CANNOT match it (source-self-scanning guard: a literal
        // needle appearing in comments or assertions inside the scanned file inflates the count,
        // causing FALSE FAILURES — the concat! split prevents this by ensuring the contiguous
        // needle never appears in this file's source text).
        let needle = concat!("fixture_present", "(\"");
        assert_eq!(
            harness_src.matches(needle).count(),
            FIXTURE_GATED_TESTS.len(),
            "Call-site count for the fixture-gating function != FIXTURE_GATED_TESTS.len() — \
             a new fixture-gated test was added without a FIXTURE_GATED_TESTS registry entry, \
             or a registered test's call was removed; update FIXTURE_GATED_TESTS accordingly"
        );
    }
} // mod iec104_e2e_real_pcaps
