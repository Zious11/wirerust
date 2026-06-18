//! Regression tests for F5-remediation fix set — REVISION 2 (canonical 0xC4 frames).
//!
//! ## Part A — F-A-001: DIR-bit correctness (BC-2.15.016 PC5)
//!
//! The DNP3 DIR bit is bit 7 (mask 0x80), NOT bit 4 (mask 0x10 = FCV/DFC).
//! `is_master_frame` tests `control & 0x80 != 0` — this is the correct mask.
//! Tests assert the correct behavior; all pass with the fixed implementation.
//!
//! ### IEEE 1815 Provenance for the 0xC4 Canonical Control Byte
//!
//! Per IEEE 1815-2012 §9.2.4.1 (data-link fixed-frame header; CONTROL field
//! validity per §9.2.4.1.3 and Annex B "Valid Data Link Layer Control Codes"),
//! the data-link CONTROL octet places DIR at bit 7 (0x80) and PRM at bit 6
//! (0x40), with the function code in the low nibble. A canonical
//! master-to-outstation UNCONFIRMED_USER_DATA primary frame has:
//!
//!   CONTROL = DIR(0x80) | PRM(0x40) | FC(0x04) = 0xC4
//!
//! This 0xC4 value is sourced from the authoritative IEEE 1815 standard,
//! cross-validated independently of the project's BC-2.15.016 PC5 /
//! BC-2.15.010 (the independent external cross-check required by
//! DF-CANONICAL-FRAME-HOLDOUT-001).
//!
//! ## Part B — F-F5-001: Unexpected-source detection (BC-2.15.010 Invariant 5)
//!
//! All frames use the canonical `build_canonical_master_control_frame` helper
//! (CTRL=0xC4). The suite uses CTRL=0xC4 throughout so that:
//!   - 0xC4 & 0x80 = 0x80 → is_master_frame=true (correct mask)
//!   - master_addrs_seen is populated correctly
//!   - detect_unexpected_source_split fires as expected
//!
//! ## Part C — F-F5-002: MitreTactic::IcsImpact / Impact Display collision
//!
//! MitreTactic::IcsImpact displays as "Impact (ICS)", distinct from
//! MitreTactic::Impact ("Impact"). Tests assert the display strings are distinct.
//!
//! ## Part D — F-F5-003: Resync accounting (BC-2.15.024, BC-2.15.016 EC-009)
//!
//! Tests for the three resync-arm counting changes (REVISION 2 IMP-1/IMP-2/IMP-3):
//! junk-at-clean-boundary, no-double-count, flood threshold, overflow-head-preserved,
//! fake-sync-flood.
//!
//! Traces to:
//!   BC-2.15.010 Invariant 5 (unexpected-source check)
//!   BC-2.15.016 PC5 (DIR bit = bit 7, mask 0x80)
//!   BC-2.15.024 (resync counting, three-path rule)
//!   HS-W37-002 (P0 holdout — two-frame sequence, canonical 0xC4 frames)
//!   F-F5-001-unexpected-source-adjudication.md REVISION 2
//!   F-F5-003-resync-accounting-adjudication.md REVISION 2

// BC traceability uses uppercase BC identifiers; suppress lint.
#![allow(non_snake_case)]

// ---------------------------------------------------------------------------
// Part A — F-A-001: DIR-bit correctness (canonical mask 0x80)
// ---------------------------------------------------------------------------

mod f5_dir_bit_fix {
    use wirerust::analyzer::dnp3::is_master_frame;

    // -----------------------------------------------------------------------
    // A-1 — test_canonical_master_frame_is_master_frame
    //
    // The canonical DNP3 master frame control byte is 0xC4.
    //
    //   0xC4 = 1100 0100
    //     bit7=1 → DIR=1 (master direction) ← the correct DIR bit per IEEE 1815
    //     bit6=1 → PRM=1
    //     bit5=0 → FCB=0
    //     bit4=0 → FCV=0  (bit 4 is FCV/DFC, NOT DIR)
    //     nibble=0100 → FC=UNCONFIRMED_USER_DATA
    //
    // Under the (fixed) correct mask (0x80): 0xC4 & 0x80 = 0x80 → is_master_frame=true. CORRECT.
    // Under the former buggy mask (0x10): 0xC4 & 0x10 = 0x00 → is_master_frame=false. WRONG.
    //
    // Traces to: BC-2.15.016 PC5 (corrected); F-A-001 REVISION 2 §R2-1.
    // -----------------------------------------------------------------------

    /// is_master_frame(0xC4) must return true: 0xC4 is the canonical master frame
    /// control byte (DIR=1, PRM=1, FCV=0, FC=UNCONF_USER_DATA). Correct mask 0x80:
    /// 0xC4 & 0x80 = 0x80 → true.
    ///
    /// IEEE 1815 citation (primary provenance for 0xC4):
    /// Per IEEE 1815-2012 §9.2.4.1 (data-link fixed-frame header; CONTROL field
    /// validity per §9.2.4.1.3 and Annex B "Valid Data Link Layer Control Codes"),
    /// the data-link CONTROL octet places DIR at bit 7 (0x80) and PRM at bit 6
    /// (0x40), function code in the low nibble. A canonical master-to-outstation
    /// UNCONFIRMED_USER_DATA primary frame has CONTROL = DIR(0x80) | PRM(0x40) |
    /// FC(0x04) = 0xC4. This 0xC4 value is sourced from the authoritative IEEE
    /// 1815 standard, cross-validated independently of the project's BC-2.15.016
    /// PC5 / BC-2.15.010 (the independent external cross-check required by
    /// DF-CANONICAL-FRAME-HOLDOUT-001).
    #[test]
    fn test_canonical_master_frame_is_master_frame() {
        // 0xC4 & 0x80 = 0x80 != 0 → must return true under corrected mask.
        // 0xC4 & 0x10 = 0x00 == 0 → returns false under BUGGY mask (this is the bug).
        assert!(
            is_master_frame(0xC4),
            "is_master_frame(0xC4) must return true: canonical master frame \
             (DIR=1 bit7 set, 0xC4 & 0x80 = 0x80)"
        );
    }

    // -----------------------------------------------------------------------
    // A-2 — test_dir_bit_is_bit7_not_bit4
    //
    // Asserts:
    //   is_master_frame(0x10) == false  (bit4=FCV only; bit7=DIR is clear)
    //   is_master_frame(0x80) == true   (bit7=DIR set; this is the pure DIR mask)
    //
    // Under the correct mask (0x80):
    //   is_master_frame(0x10) = (0x10 & 0x80 = 0x00 == 0) = false → CORRECT
    //   is_master_frame(0x80) = (0x80 & 0x80 = 0x80 != 0) = true  → CORRECT
    //
    // Traces to: BC-2.15.016 PC5 (corrected); F-A-001 REVISION 2 §R2-1.
    // -----------------------------------------------------------------------

    /// is_master_frame(0x10) must return false (0x10 = FCV bit only; DIR=bit7 is clear).
    /// is_master_frame(0x80) must return true  (0x80 = pure DIR bit only; DIR=1).
    /// Verifies the correct 0x80 mask distinguishes DIR from FCV.
    #[test]
    fn test_dir_bit_is_bit7_not_bit4() {
        // 0x10 = 0001 0000: bit4=FCV(=1), bit7=DIR(=0). DIR is CLEAR.
        // Correct mask (0x80): 0x10 & 0x80 = 0 → false.
        // Buggy mask (0x10): 0x10 & 0x10 = 0x10 → true. WRONG.
        assert!(
            !is_master_frame(0x10),
            "is_master_frame(0x10) must return false: 0x10 sets FCV bit (bit4) only; \
             DIR bit (bit7, mask 0x80) is clear"
        );

        // 0x80 = 1000 0000: bit7=DIR(=1), all other bits clear. DIR is SET.
        // Correct mask (0x80): 0x80 & 0x80 = 0x80 → true.
        // Buggy mask (0x10): 0x80 & 0x10 = 0x00 → false. WRONG.
        assert!(
            is_master_frame(0x80),
            "is_master_frame(0x80) must return true: 0x80 sets DIR bit (bit7) only"
        );
    }

    // -----------------------------------------------------------------------
    // A-3 — test_EF_is_master_frame_under_corrected_mask
    //
    // 0xEF = 1110 1111: bit7=DIR=1 → is_master_frame should be TRUE.
    //
    // The STORY-107 test previously asserted is_master_frame(0xEF)==false with
    // comment "0xEF & 0x10 == 0" — that was correct under the BUGGY mask but
    // wrong under the correct mask (0xEF & 0x80 = 0x80 != 0 → true).
    //
    // This test asserts the CORRECT behavior (must be true). Confirms that 0xEF
    // being DIR=1 is not a surprise — bit7 is set.
    //
    // 0xEF & 0x80 = 0x80 → is_master_frame=true (correct). Regression guard for the
    // former STORY-107 test that had 0xEF wrong (it used the buggy mask).
    //
    // Traces to: BC-2.15.016 PC5; F-A-001 REVISION 2 §R2-1 (0xEF correction).
    // -----------------------------------------------------------------------

    /// is_master_frame(0xEF) must return true: 0xEF has DIR bit (bit7) set.
    /// 0xEF = 1110 1111: bit7=1 (DIR=1). 0xEF & 0x80 = 0x80 != 0 → true.
    /// The old STORY-107 test had this wrong; see F-A-001 REVISION 2.
    #[test]
    fn test_EF_is_master_frame_under_corrected_mask() {
        // 0xEF = 1110 1111: bit7(DIR)=1. With corrected mask 0x80: true.
        assert!(
            is_master_frame(0xEF),
            "is_master_frame(0xEF) must return true: 0xEF & 0x80 = 0x80 != 0 (DIR=1)"
        );
    }

    // -----------------------------------------------------------------------
    // A-4 — test_canonical_mask_sanity_master_addrs_seen_populated
    //
    // Delivers a canonical 0xC4 non-Control master frame and asserts that
    // master_addrs_seen is populated (i.e., is_master_frame returned true for 0xC4).
    //
    // Under the BUGGY mask: is_master_frame(0xC4)=false → master_addrs_seen is empty
    // after the frame is delivered → assertion fails. This is the RED condition.
    //
    // Under the CORRECT mask: is_master_frame(0xC4)=true → master_addrs_seen=[0x0001].
    //
    // Uses a non-Control FC (0x00, READ — not a Control-class FC per BC-2.15.010)
    // to avoid triggering unexpected-source detection (which requires master_addrs_seen
    // to be non-empty AND a Control-class FC from a new source — here master_addrs_seen
    // starts empty so no unexpected-source check fires regardless).
    //
    // Traces to: BC-2.15.016 PC5; F-A-001 REVISION 2 §R2-1.
    // -----------------------------------------------------------------------

    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::reassembly::flow::FlowKey;

    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            20000,
        )
    }

    /// Delivers a canonical CTRL=0xC4 master-direction frame with a non-Control FC
    /// (FC=0x01, READ) and asserts master_addrs_seen is populated with the source
    /// address after the call. The correct 0x80 mask recognizes 0xC4 as a master frame
    /// and populates master_addrs_seen.
    #[test]
    fn test_canonical_mask_sanity_master_addrs_seen_populated() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Build a 15-byte frame with CTRL=0xC4 (canonical master) and FC=0x01 (READ).
        // LENGTH=8 → frame_len=15. Transport=0xC0 (FIR=1, FIN=1).
        // FC=0x01 (READ) is not a Control-class FC → no burst detection, no
        // unexpected-source detection. We only test master_addrs_seen population.
        let mut frame = vec![0u8; 15];
        frame[0] = 0x05; // START1
        frame[1] = 0x64; // START2
        frame[2] = 8; // LENGTH (5 header + 3 user data bytes = frame_len=15 with CRC)
        frame[3] = 0xC4; // CTRL: canonical master (DIR=1 bit7, PRM=1 bit6, FC=4)
        let [dl, dh] = 0x0003u16.to_le_bytes();
        frame[4] = dl;
        frame[5] = dh; // dest=0x0003
        let [sl, sh] = 0x0001u16.to_le_bytes();
        frame[6] = sl;
        frame[7] = sh; // src=0x0001
        // bytes 8-9: header CRC placeholder (0x00)
        frame[10] = 0xC0; // transport: FIR=1 (bit6), FIN=1 (bit7)
        frame[11] = 0x00; // app control
        frame[12] = 0x01; // FC=0x01 (READ — not Control-class)
        // bytes 13-14: data-block CRC placeholder

        analyzer.on_data(key.clone(), &frame, 0);

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        assert!(
            flow.master_addrs_seen.contains(&0x0001),
            "master_addrs_seen must contain 0x0001 after delivering a canonical CTRL=0xC4 \
             master frame (0xC4 & 0x80 = 0x80 → is_master_frame=true)"
        );
    }
}

// ---------------------------------------------------------------------------
// Part B — F-F5-001: Unexpected-source detection (canonical 0xC4 frames)
// ---------------------------------------------------------------------------

mod f5_unexpected_source {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::findings::{Confidence, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;

    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            20000,
        )
    }

    // -----------------------------------------------------------------------
    // CANONICAL HELPER — build_canonical_master_control_frame
    //
    // Builds a 15-byte DNP3 master Control-class frame with CTRL=0xC4.
    //
    // CTRL=0xC4:
    //   bit7=1 → DIR=1 (master direction) — tests bit 7 (mask 0x80, CORRECT)
    //   bit6=1 → PRM=1
    //   bit5=0 → FCB=0
    //   bit4=0 → FCV=0  (bit 4 is FCV, NOT DIR — the formerly buggy bit)
    //   nibble=0100 → FC=UNCONFIRMED_USER_DATA → has_user_data=true
    //
    // 0xC4 & 0x80 = 0x80 != 0 → is_master_frame=true (CORRECT mask).
    // 0xC4 & 0x10 = 0x00 == 0 → is_master_frame=false (BUGGY mask) — RED condition.
    //
    // This is the canonical byte vector from BC-2.15.010 Canonical Test Vectors
    // and HS-W37-002 (annotated: "CTRL=0xC4 (DIR=1, PRM=1, FCV=0, link-FC=4)").
    // Using it here makes the entire F-001 test suite red under the buggy mask
    // (because master_addrs_seen would never be populated — is_master_frame=false
    // for all 0xC4 frames), AND it makes them the correct anchor for post-fix green.
    //
    // Per F-A-001 REVISION 2 §R2-6: this helper is mandatory in this file.
    // -----------------------------------------------------------------------

    /// Build a canonical DNP3 master Control-class frame matching the BC-2.15.010
    /// Canonical Test Vector byte sequence.
    ///
    /// CTRL=0xC4: DIR=1 (bit7, mask 0x80), PRM=1 (bit6), FCB=0 (bit5), FCV=0 (bit4),
    ///            FC=0x04 (UNCONFIRMED_USER_DATA, lower nibble).
    /// 0xC4 & 0x80 = 0x80 != 0 → is_master_frame=true (corrected mask).
    /// 0xC4 & 0x0F = 0x04 → has_user_data=true.
    ///
    /// This matches the holdout HS-W37-002 annotated frame:
    ///   "CTRL=0xC4 (DIR=1, PRM=1, FCV=0, link-FC=4)"
    fn build_canonical_master_control_frame(app_fc: u8, dest: u16, src: u16) -> Vec<u8> {
        // LENGTH=8 → frame_len=15 (5 header bytes + 8 data bytes; data occupies
        // 1 data block of up to 16 payload bytes + 2 CRC bytes = 3 payload + 2 CRC = 5,
        // but here: LENGTH counts header fields only after START; frame_len formula:
        // frame_len = 5 + LENGTH + 2*ceil((LENGTH-5)/16). LENGTH=8 → data=3 bytes,
        // 1 block → frame_len = 5+8+2 = 15.)
        let mut frame = vec![0u8; 15];
        frame[0] = 0x05; // START1
        frame[1] = 0x64; // START2
        frame[2] = 8; // LENGTH=8
        frame[3] = 0xC4; // CTRL: DIR=1(bit7), PRM=1(bit6), FCV=0(bit4), FC=4(UNCONF_USER_DATA)
        let [dl, dh] = dest.to_le_bytes();
        frame[4] = dl;
        frame[5] = dh;
        let [sl, sh] = src.to_le_bytes();
        frame[6] = sl;
        frame[7] = sh;
        // bytes 8–9: header CRC placeholder (0x00 — CRC deferred per ADR-007 Decision 3)
        frame[10] = 0xC0; // transport: FIR=1 (bit6=0x40), FIN=1 (bit7=0x80) → 0xC0
        frame[11] = 0x00; // app control byte
        frame[12] = app_fc; // application function code
        // bytes 13–14: data-block CRC placeholder (0x00)
        frame
    }

    // -----------------------------------------------------------------------
    // B-0 — test_canonical_master_frame_helper_satisfies_is_master_frame
    //
    // Correctness anchor: documents the corrected 0x80 mask behavior and catches
    // any regression if someone reverts the mask fix.
    //
    // is_master_frame(0xC4) returns true under the correct 0x80 mask (regression guard).
    // is_master_frame(0x04) returns false under both masks (no DIR bit — expected).
    //
    // Traces to: F-A-001 REVISION 2 §R2-6 "test_canonical_master_frame_helper_...".
    // -----------------------------------------------------------------------

    /// Correctness anchor for the build_canonical_master_control_frame helper.
    /// Asserts the correct 0x80-mask behavior for key control bytes.
    /// Regression guard: 0xC4 must return true (0xC4 & 0x80 = 0x80 != 0).
    #[test]
    fn test_canonical_master_frame_helper_satisfies_is_master_frame() {
        use wirerust::analyzer::dnp3::is_master_frame;

        // Canonical master (CTRL=0xC4): DIR=1 bit7 set → must be true.
        // 0xC4 & 0x80 = 0x80 != 0 → true.
        assert!(
            is_master_frame(0xC4),
            "is_master_frame(0xC4) must return true: 0xC4 & 0x80 = 0x80 != 0 (DIR=1 bit7 set)"
        );

        // 0xD4: DIR=1 bit7 set (0xD4 & 0x80 = 0x80) → must be true.
        // Correct under BOTH masks (0xD4 & 0x10 = 0x10 also non-zero).
        assert!(
            is_master_frame(0xD4),
            "is_master_frame(0xD4) must return true: bit7 (DIR) is set"
        );

        // 0x00: all bits clear → must be false.
        assert!(
            !is_master_frame(0x00),
            "is_master_frame(0x00) must return false: no bits set"
        );

        // 0x04: UNCONF_USER_DATA with DIR=0 → must be false.
        assert!(
            !is_master_frame(0x04),
            "is_master_frame(0x04) must return false: DIR=0 (bit7 clear)"
        );
    }

    // -----------------------------------------------------------------------
    // B-1 — test_unexpected_source_fires_at_count_1
    //
    // HS-W37-002 (AMENDED, two-frame sequence) primary coverage.
    //
    // Setup:  fresh Dnp3Analyzer, threshold=10.
    //         Frame 1: build_canonical_master_control_frame(0x05, 0x0003, 0x0001)
    //           → establishes expected set: master_addrs_seen=[0x0001].
    //           No finding after frame 1.
    //         Frame 2: build_canonical_master_control_frame(0x05, 0x0003, 0x0099)
    //           → src=0x0099 NOT in master_addrs_seen → unexpected-source finding.
    //
    // Expected:
    //   all_findings.len() == 1
    //   finding[0].mitre_techniques == ["T1692.001"]
    //   finding[0].category == ThreatCategory::Execution
    //   finding[0].verdict == Verdict::Likely
    //   finding[0].confidence == Confidence::High   (NOT Medium)
    //   finding[0].summary == EXACT pinned string (see below)
    //   flow.direct_operate_count == 2  (both FCs counted — fall-through invariant)
    //   flow.direct_operate_emitted == false  (2 > 10 = false)
    //   flow.unexpected_source_emitted == true
    //
    // EXACT pinned summary string (per F-A-001 REVISION 2 §R2-6):
    //   "DNP3 unauthorized control command from unexpected source: \
    //    src=0x0099 is not in expected master set [0x0001] on dest=0x0003"
    //
    // Traces to: BC-2.15.010 Invariant 5; HS-W37-002 (amended);
    //            F-A-001 REVISION 2 §§R2-5, R2-6.
    // -----------------------------------------------------------------------

    /// BC-2.15.010 Invariant 5: unexpected source fires at count=1, independent of
    /// the burst threshold. Asserts the EXACT pinned summary string (full equality).
    /// Traces to: BC-2.15.010 Invariant 5; F-A-001 REVISION 2 §R2-5.
    #[test]
    fn test_unexpected_source_fires_at_count_1() {
        let mut analyzer = Dnp3Analyzer::new(10); // threshold=10
        let key = test_flow_key();

        // Frame 1: src=0x0001 establishes the expected master set.
        // master_addrs_seen empty BEFORE this frame; expected_set_established=false;
        // no unexpected-source check fires. After frame 1: master_addrs_seen=[0x0001].
        let frame1 = build_canonical_master_control_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame1, 0);

        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "first master Control FC must NOT emit a finding (establishes expected set)"
        );

        // Frame 2: src=0x0099 NOT in master_addrs_seen=[0x0001] → unexpected source.
        // direct_operate_count=2 after this frame; 2>10=false → burst guard NOT set.
        let frame2 = build_canonical_master_control_frame(0x05, 0x0003, 0x0099);
        analyzer.on_data(key.clone(), &frame2, 1);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "test_unexpected_source_fires_at_count_1: expected exactly ONE T1692.001 finding \
             for unexpected source 0x0099 at count=1"
        );

        let f = &analyzer.all_findings[0];

        // Technique tag
        assert_eq!(
            f.mitre_techniques,
            vec!["T1692.001"],
            "finding must carry T1692.001"
        );

        // Category, verdict, confidence
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "category must be ThreatCategory::Execution; got {:?}",
            f.category
        );
        assert!(
            matches!(f.verdict, Verdict::Likely),
            "verdict must be Verdict::Likely; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::High),
            "confidence must be Confidence::High (unauthorized source = high confidence, \
             distinct from burst's Medium); got {:?}",
            f.confidence
        );

        // EXACT pinned summary string (per F-A-001 REVISION 2 §R2-6 "Pinned Summary String"):
        //   src=0x0099 (formatted as {:#06X} → "0x0099")
        //   master_set=[0x0001] (single entry, formatted as "[0x0001]")
        //   dest=0x0003 (formatted as {:#06X} → "0x0003")
        let expected_summary = "DNP3 unauthorized control command from unexpected source: \
             src=0x0099 is not in expected master set [0x0001] on dest=0x0003";
        assert_eq!(
            f.summary, expected_summary,
            "summary must be exact pinned string per F-A-001 REVISION 2 §R2-6; \
             got: {:?}",
            f.summary
        );

        // F-P4-001: evidence field must carry TWO entries per F-F5-001 REVISION 2 §2:
        //   entry[0] = "FC=0x{app_fc:02X} dest={dest:#06X} src={src:#06X}"
        //   entry[1] = "expected_masters={master_set}"
        // For this test: app_fc=0x05 (DIRECT_OPERATE), dest=0x0003, src=0x0099,
        //   master_set=[0x0001] (single entry formatted {:#06X}).
        assert_eq!(
            f.evidence,
            vec![
                "FC=0x05 dest=0x0003 src=0x0099".to_string(),
                "expected_masters=[0x0001]".to_string(),
            ],
            "evidence must be TWO entries per F-F5-001 REVISION 2 §2: \
             entry[0]='FC=0x{{app_fc:02X}} dest={{dest:#06X}} src={{src:#06X}}', \
             entry[1]='expected_masters={{master_set}}'; \
             got: {:?}",
            f.evidence
        );

        // Fall-through invariant: both FCs (0x0001 and 0x0099) counted.
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.direct_operate_count, 2,
            "direct_operate_count must be 2 (both FCs counted — fall-through invariant R2-4)"
        );
        assert!(
            !flow.direct_operate_emitted,
            "direct_operate_emitted must be false (count=2 not > threshold=10)"
        );
        assert!(
            flow.unexpected_source_emitted,
            "unexpected_source_emitted must be true after emission"
        );
    }

    // -----------------------------------------------------------------------
    // B-2 — test_unexpected_source_independent_of_threshold
    //
    // Setup:  threshold=10. 9 Control FCs from src=0x0001. 1 FC from src=0x0099.
    //         direct_operate_count=10 after all 10 FCs.
    //         count=10 > threshold=10 → FALSE → burst guard does NOT fire.
    //         Only the unexpected-source finding should fire.
    //
    // Expected: exactly ONE finding; summary contains "unexpected source" and NOT "burst".
    //           flow.direct_operate_count == 10. flow.direct_operate_emitted == false.
    //
    // Traces to: BC-2.15.010 Invariant 5; HS-W37-002 evaluator note.
    // -----------------------------------------------------------------------

    /// Unexpected-source check is independent of the volumetric burst threshold.
    /// 9 FCs from established master (count=9) + 1 FC from unexpected source
    /// (count=10; 10 > 10 is false) → only unexpected-source finding fires.
    #[test]
    fn test_unexpected_source_independent_of_threshold() {
        let mut analyzer = Dnp3Analyzer::new(10); // threshold=10
        let key = test_flow_key();

        // 9 Control FCs from src=0x0001 — establishes set, count=9, no burst yet.
        for ts in 0u32..9 {
            let frame = build_canonical_master_control_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, ts);
        }
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "no finding after 9 FCs from established master"
        );

        // 1 Control FC from src=0x0099 (unexpected); count becomes 10.
        // 10 > 10 = false → burst guard does NOT fire.
        let frame_unexpected = build_canonical_master_control_frame(0x05, 0x0003, 0x0099);
        analyzer.on_data(key.clone(), &frame_unexpected, 9);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "test_unexpected_source_independent_of_threshold: expected exactly ONE finding \
             (unexpected-source, not burst)"
        );

        let f = &analyzer.all_findings[0];

        // Must be unexpected-source finding, NOT burst finding.
        assert!(
            f.summary.contains("unexpected source"),
            "summary must contain 'unexpected source'; got: {:?}",
            f.summary
        );
        assert!(
            !f.summary.contains("burst"),
            "summary must NOT contain 'burst' (must be unexpected-source, not burst finding); \
             got: {:?}",
            f.summary
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.direct_operate_count, 10,
            "direct_operate_count must be 10 after 10 total Control FCs"
        );
        assert!(
            !flow.direct_operate_emitted,
            "burst guard must NOT be set (count=10 is not > threshold=10)"
        );
    }

    // -----------------------------------------------------------------------
    // B-3 — test_unexpected_source_one_shot_guard
    //
    // Setup:  1 FC from src=0x0001 (establishes set). 3 FCs from src=0x0099.
    //
    // Expected: exactly ONE unexpected-source finding total (first FC from 0x0099
    //           triggers; subsequent two suppressed by unexpected_source_emitted=true).
    //           flow.unexpected_source_emitted == true.
    //
    // Traces to: F-A-001 REVISION 2 §2 "One-shot guard: T1692.001 NEVER reset".
    // -----------------------------------------------------------------------

    /// Flow-lifetime one-shot guard suppresses repeated unexpected-source findings.
    /// First FC from 0x0099 triggers; second and third suppressed by the guard.
    #[test]
    fn test_unexpected_source_one_shot_guard() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Establish expected set: src=0x0001.
        let frame_expected = build_canonical_master_control_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame_expected, 0);

        // 3 Control FCs from src=0x0099 — all unexpected.
        for ts in 1u32..4 {
            let frame = build_canonical_master_control_frame(0x05, 0x0003, 0x0099);
            analyzer.on_data(key.clone(), &frame, ts);
        }

        let unexpected_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.summary.contains("unexpected source"))
            .count();

        assert_eq!(
            unexpected_count, 1,
            "test_unexpected_source_one_shot_guard: expected exactly ONE unexpected-source \
             finding (first FC triggers; subsequent suppressed by unexpected_source_emitted)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.unexpected_source_emitted,
            "unexpected_source_emitted must be true after first emission"
        );
    }

    // -----------------------------------------------------------------------
    // B-4 — test_first_master_is_expected
    //
    // Setup:  fresh flow (master_addrs_seen is empty). First Control FC from src=0x0001.
    //
    // Expected: NO finding. master_addrs_seen == [0x0001].
    //           unexpected_source_emitted == false. direct_operate_count == 1.
    //
    // This is a GREEN test under both masks once the impl is correct: the first
    // master does not trigger an unexpected-source finding regardless.
    // Under the BUGGY mask with 0xC4 frames: master_addrs_seen stays empty
    // (is_master_frame=false) → still no finding (but for the wrong reason).
    // After fix: no finding because expected_set_established=false at step A.
    //
    // Traces to: F-A-001 REVISION 2 §1 "first master is NOT unexpected".
    // -----------------------------------------------------------------------

    /// The first master Control FC on a fresh flow establishes the expected set.
    /// No unexpected-source finding fires for the establishing frame.
    #[test]
    fn test_first_master_is_expected() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // First-ever Control FC on a fresh flow.
        let frame = build_canonical_master_control_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 0);

        // No finding — first master is always expected.
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "test_first_master_is_expected: first Control FC from the first master must NOT \
             emit a finding; got {} findings",
            analyzer.all_findings.len()
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        assert!(
            flow.master_addrs_seen.contains(&0x0001),
            "master_addrs_seen must contain 0x0001 after first master frame; \
             got {:?}",
            flow.master_addrs_seen
        );
        assert!(
            !flow.unexpected_source_emitted,
            "unexpected_source_emitted must be false for the establishing master"
        );
        assert_eq!(
            flow.direct_operate_count, 1,
            "direct_operate_count must be 1 (fall-through: burst counter incremented)"
        );
    }

    // -----------------------------------------------------------------------
    // B-5 — test_unexpected_source_and_burst_both_fire
    //
    // FALL-THROUGH INVARIANT (F-A-004): the unexpected-source detection must NOT
    // short-circuit the burst detection. Both findings can fire independently.
    //
    // Setup:  threshold=3.
    //         Frame 1: src=0x0001 (count=1; establishes set).
    //         Frames 2-5: src=0x0099 (count=2,3,4,5):
    //           - Frame 2 (count=2): unexpected-source fires (0x0099 not in set).
    //           - Frame 3 (count=3): 3>3=false → no burst.
    //           - Frame 4 (count=4): 4>3=true → burst fires.
    //           - Frame 5 (count=5): both guards set; no additional findings.
    //
    // Expected:
    //   all_findings.len() == 2 (one unexpected-source + one burst).
    //   Both carry T1692.001.
    //   One summary contains "unexpected source".
    //   One summary contains "threshold 3".
    //   flow.unexpected_source_emitted == true.
    //   flow.direct_operate_emitted == true.
    //
    // Traces to: F-A-001 REVISION 2 §2 "Interaction with burst guard";
    //            F-A-004 fall-through invariant.
    // -----------------------------------------------------------------------

    /// Both unexpected-source and burst-threshold T1692.001 findings fire on the
    /// same flow with independent one-shot guards. Verifies fall-through invariant.
    #[test]
    fn test_unexpected_source_and_burst_both_fire() {
        let mut analyzer = Dnp3Analyzer::new(3); // threshold=3
        let key = test_flow_key();

        // Frame 1: src=0x0001 establishes expected set; count=1.
        let frame1 = build_canonical_master_control_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame1, 0);

        // Frames 2-5: src=0x0099 (unexpected), count goes 2→5.
        for ts in 1u32..5 {
            let frame = build_canonical_master_control_frame(0x05, 0x0003, 0x0099);
            analyzer.on_data(key.clone(), &frame, ts);
        }

        assert_eq!(
            analyzer.all_findings.len(),
            2,
            "test_unexpected_source_and_burst_both_fire: expected 2 T1692.001 findings \
             (one unexpected-source + one burst)"
        );

        // Both findings carry T1692.001.
        for f in &analyzer.all_findings {
            assert_eq!(
                f.mitre_techniques,
                vec!["T1692.001"],
                "every finding must carry T1692.001; got {:?}",
                f.mitre_techniques
            );
        }

        // One finding is the unexpected-source finding.
        let has_unexpected = analyzer
            .all_findings
            .iter()
            .any(|f| f.summary.contains("unexpected source"));
        assert!(
            has_unexpected,
            "all_findings must contain an unexpected-source finding \
             (summary contains 'unexpected source')"
        );

        // One finding is the burst finding (contains "threshold 3").
        let has_burst = analyzer
            .all_findings
            .iter()
            .any(|f| f.summary.contains("threshold 3"));
        assert!(
            has_burst,
            "all_findings must contain a burst finding (summary contains 'threshold 3')"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.unexpected_source_emitted,
            "unexpected_source_emitted must be true"
        );
        assert!(
            flow.direct_operate_emitted,
            "direct_operate_emitted must be true"
        );
    }

    // -----------------------------------------------------------------------
    // B-6 — test_unexpected_source_max_master_addrs_full
    //
    // Setup:  fill master_addrs_seen to MAX_MASTER_ADDRS (64) by delivering 64
    //         non-Control master frames from src=1..=64 (each is a READ FC=0x01,
    //         not Control-class → no burst detection). Then deliver a Control FC
    //         from src=0x0099 (not in set; cap is full so 0x0099 cannot be added).
    //
    // Expected: ONE unexpected-source finding (cap saturation does NOT suppress detection).
    //           flow.master_addrs_seen.len() == 64 (0x0099 NOT added — cap).
    //
    // Traces to: F-A-001 REVISION 2 §2 "Interaction with MAX_MASTER_ADDRS".
    // -----------------------------------------------------------------------

    /// MAX_MASTER_ADDRS full case: attacker saturating the master-address table
    /// cannot use cap-fullness to suppress unexpected-source detection.
    #[test]
    fn test_unexpected_source_max_master_addrs_full() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Pre-fill master_addrs_seen to 64 (MAX_MASTER_ADDRS) using non-Control FCs.
        // FC=0x01 (READ) is not Control-class → no burst detection triggered.
        // Each src 1..=64 is distinct → all 64 get added to master_addrs_seen.
        for src in 1u16..=64 {
            let mut frame = vec![0u8; 15];
            frame[0] = 0x05;
            frame[1] = 0x64;
            frame[2] = 8;
            frame[3] = 0xC4; // canonical master CTRL
            let [dl, dh] = 0x0003u16.to_le_bytes();
            frame[4] = dl;
            frame[5] = dh;
            let [sl, sh] = src.to_le_bytes();
            frame[6] = sl;
            frame[7] = sh;
            frame[10] = 0xC0; // FIR=1, FIN=1
            frame[11] = 0x00;
            frame[12] = 0x01; // FC=READ — non-Control
            analyzer.on_data(key.clone(), &frame, src as u32);
        }

        // Verify master_addrs_seen is at capacity.
        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after setup");
            assert_eq!(
                flow.master_addrs_seen.len(),
                64,
                "pre-condition: master_addrs_seen must be at MAX_MASTER_ADDRS=64"
            );
        }

        // Deliver Control FC from src=0x0099 (not in the 64-entry set).
        // Since master_addrs_seen is full, 0x0099 cannot be added.
        // But the unexpected-source check must still fire.
        let frame_unexpected = build_canonical_master_control_frame(0x05, 0x0003, 0x0099);
        analyzer.on_data(key.clone(), &frame_unexpected, 65);

        // Filter: unexpected-source findings only.
        let unexpected_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.summary.contains("unexpected source"))
            .count();

        assert_eq!(
            unexpected_count, 1,
            "test_unexpected_source_max_master_addrs_full: expected ONE unexpected-source \
             finding even when master_addrs_seen is at cap (64 entries)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.master_addrs_seen.len(),
            64,
            "master_addrs_seen must remain at 64 (0x0099 not added due to cap)"
        );
    }

    // -----------------------------------------------------------------------
    // B-7 — test_unexpected_source_second_distinct_unexpected_source_suppressed
    //
    // Rotation case: one-shot guard prevents a second distinct unexpected source
    // from generating a second finding (prevents finding-flood from address rotation).
    //
    // Setup:  1 FC from 0x0001 (establishes set).
    //         1 FC from 0x0099 (unexpected, guard fires → unexpected_source_emitted=true).
    //         1 FC from 0x00AA (SECOND unexpected source → suppressed by one-shot guard).
    //
    // Expected: exactly ONE unexpected-source finding total.
    //           flow.master_addrs_seen contains 0x0001, 0x0099, 0x00AA (all pushed —
    //           source-address push and finding emission are independent).
    //
    // Traces to: F-A-001 REVISION 2 §R2-6 "test_unexpected_source_rotation_...".
    // -----------------------------------------------------------------------

    /// One-shot flow-lifetime guard suppresses second distinct unexpected source address.
    /// Prevents an attacker rotating source addresses from flooding findings.
    #[test]
    fn test_unexpected_source_second_distinct_unexpected_source_suppressed() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Frame 1: establish expected set with src=0x0001.
        let frame1 = build_canonical_master_control_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame1, 0);

        // Frame 2: unexpected src=0x0099 → guard fires.
        let frame2 = build_canonical_master_control_frame(0x05, 0x0003, 0x0099);
        analyzer.on_data(key.clone(), &frame2, 1);

        // Frame 3: SECOND unexpected src=0x00AA → suppressed by one-shot guard.
        let frame3 = build_canonical_master_control_frame(0x05, 0x0003, 0x00AA);
        analyzer.on_data(key.clone(), &frame3, 2);

        let unexpected_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.summary.contains("unexpected source"))
            .count();

        assert_eq!(
            unexpected_count, 1,
            "test_unexpected_source_second_distinct_unexpected_source_suppressed: \
             expected exactly ONE unexpected-source finding total (one-shot guard \
             suppresses second distinct unexpected address)"
        );

        // All three source addresses should have been pushed to master_addrs_seen
        // (source-address push is independent of finding emission).
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.master_addrs_seen.contains(&0x0001),
            "master_addrs_seen must contain 0x0001"
        );
        assert!(
            flow.master_addrs_seen.contains(&0x0099),
            "master_addrs_seen must contain 0x0099 (pushed independently of finding emission)"
        );
        assert!(
            flow.master_addrs_seen.contains(&0x00AA),
            "master_addrs_seen must contain 0x00AA (pushed independently of finding emission)"
        );
    }

    // -----------------------------------------------------------------------
    // B-8 — test_unexpected_source_skipped_on_is_non_dnp3
    //
    // Bailed flow (is_non_dnp3=true) receives a Control FC → on_data is a no-op
    // (BC-2.15.009 PC5-6); the unexpected-source check is never reached.
    //
    // Setup:  trigger the bail by delivering non-DNP3 bytes (no valid sync at head
    //         within first 16 bytes). Then deliver a canonical Control FC from 0x0099.
    //
    // Expected: NO finding (bail fires before any frame-parse stage).
    //           flow.is_non_dnp3 == true.
    //           flow.unexpected_source_emitted == false.
    //
    // Traces to: BC-2.15.009 PC5-6; F-A-001 REVISION 2 §R2-6
    //            "test_unexpected_source_skipped_on_non_dnp3_flow".
    // -----------------------------------------------------------------------

    /// Bailed flow (is_non_dnp3=true) is a no-op for all on_data calls;
    /// unexpected-source detection is never reached.
    #[test]
    fn test_unexpected_source_skipped_on_is_non_dnp3() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Trigger the desync bail: 16 bytes with no valid [0x05, 0x64] sync pair.
        // BC-2.15.009: if first 16 bytes contain no sync → is_non_dnp3=true.
        let non_dnp3 = [
            0xFF, 0xFE, 0x00, 0x01, 0x02, 0x03, 0x04, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
            0x0D, 0x0E,
        ];
        analyzer.on_data(key.clone(), &non_dnp3, 0);

        // Verify bail was triggered.
        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after bail trigger");
            assert!(
                flow.is_non_dnp3,
                "pre-condition: is_non_dnp3 must be true after non-DNP3 payload delivery"
            );
        }

        // Now deliver a Control FC from src=0x0099 — must be a no-op.
        let frame = build_canonical_master_control_frame(0x05, 0x0003, 0x0099);
        analyzer.on_data(key.clone(), &frame, 1);

        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "test_unexpected_source_skipped_on_is_non_dnp3: bailed flow must produce \
             NO finding for any subsequent on_data call (is_non_dnp3 bail is a no-op)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.is_non_dnp3,
            "is_non_dnp3 must remain true after the no-op call"
        );
        assert!(
            !flow.unexpected_source_emitted,
            "unexpected_source_emitted must be false (detection never reached on bailed flow)"
        );
    }
}

// ---------------------------------------------------------------------------
// Part C — F-F5-002: MitreTactic::IcsImpact / Impact Display collision
// ---------------------------------------------------------------------------

mod f5_ics_impact_display {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::mitre::{MitreTactic, all_tactics_in_report_order};
    use wirerust::reporter::Reporter;
    use wirerust::reporter::terminal::TerminalReporter;
    use wirerust::summary::Summary;

    // -----------------------------------------------------------------------
    // C-1 — test_ics_impact_display_distinct_from_impact
    //
    // MitreTactic::IcsImpact currently displays as "Impact" (same as Impact).
    // After the fix, IcsImpact must have a distinct non-empty display string
    // (e.g., "Impact (ICS)").
    //
    // MitreTactic::IcsImpact displays as "Impact (ICS)"; MitreTactic::Impact as "Impact".
    //
    // Traces to: F-F5-002.
    // -----------------------------------------------------------------------

    /// MitreTactic::IcsImpact must display as a distinct string from Impact.
    #[test]
    fn test_ics_impact_display_distinct_from_impact() {
        let impact_str = MitreTactic::Impact.to_string();
        let ics_impact_str = MitreTactic::IcsImpact.to_string();

        assert_ne!(
            ics_impact_str, impact_str,
            "MitreTactic::IcsImpact must display as a DISTINCT string from \
             MitreTactic::Impact; got both as {:?}",
            impact_str
        );

        assert!(
            !ics_impact_str.is_empty(),
            "IcsImpact display must be a non-empty string"
        );

        // all_tactics_in_report_order must produce distinct display strings.
        let tactics = all_tactics_in_report_order();
        let impact_display: Vec<String> = tactics
            .iter()
            .filter(|t| matches!(t, MitreTactic::Impact))
            .map(|t| t.to_string())
            .collect();
        let ics_impact_display: Vec<String> = tactics
            .iter()
            .filter(|t| matches!(t, MitreTactic::IcsImpact))
            .map(|t| t.to_string())
            .collect();

        assert_eq!(
            impact_display.len(),
            1,
            "exactly one Impact entry in all_tactics_in_report_order"
        );
        assert_eq!(
            ics_impact_display.len(),
            1,
            "exactly one IcsImpact entry in all_tactics_in_report_order"
        );
        assert_ne!(
            impact_display[0], ics_impact_display[0],
            "Impact and IcsImpact must have distinct display strings in all_tactics_in_report_order"
        );
    }

    // -----------------------------------------------------------------------
    // C-2 — test_reporter_renders_distinct_impact_sections
    //
    // Grouped terminal reporter must render two distinct tactic section headers
    // for Impact and IcsImpact.
    //
    // Verifies the grouped reporter renders two distinct tactic headers for Impact and IcsImpact.
    //
    // Traces to: F-F5-002; terminal.rs render_findings_grouped.
    // -----------------------------------------------------------------------

    fn mitre_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_mitre_grouping: true,
            show_hosts_breakdown: false,
            // STORY-118: new field; false here — grouped mode does not apply collapse.
            collapse_findings: false,
        }
    }

    /// Grouped terminal reporter renders distinct tactic sections for Impact and
    /// IcsImpact when findings belong to both tactics.
    #[test]
    fn test_reporter_renders_distinct_impact_sections() {
        let enterprise_impact_finding = Finding {
            category: ThreatCategory::Execution,
            verdict: Verdict::Likely,
            confidence: Confidence::Medium,
            summary: "Service Exhaustion Flood detected".to_string(),
            evidence: vec![],
            mitre_techniques: vec!["T1499.002".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        };

        let ics_impact_finding = Finding {
            category: ThreatCategory::Execution,
            verdict: Verdict::Likely,
            confidence: Confidence::Medium,
            summary: "DNP3 loss of control: combined restart+block events threshold crossed"
                .to_string(),
            evidence: vec![],
            mitre_techniques: vec!["T0827".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        };

        let findings = vec![enterprise_impact_finding, ics_impact_finding];
        let out = mitre_reporter().render(&Summary::new(), &findings, &[]);

        let impact_section_headers: Vec<&str> = out
            .lines()
            .filter(|line| line.trim_start().starts_with("## ") && line.contains("Impact"))
            .collect();

        let unique_headers: std::collections::HashSet<&str> =
            impact_section_headers.iter().copied().collect();

        assert_eq!(
            unique_headers.len(),
            2,
            "expected 2 DISTINCT tactic section headers for Impact and IcsImpact; \
             got {} unique header(s): {:?}.\nOutput:\n{out}",
            unique_headers.len(),
            unique_headers
        );
    }
}

// ---------------------------------------------------------------------------
// Part D — F-F5-003: Resync accounting (REVISION 2)
// ---------------------------------------------------------------------------

mod f5_resync_accounting {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::reassembly::flow::FlowKey;

    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            20000,
        )
    }

    /// Build a minimal valid 10-byte DNP3 frame (LENGTH=5 → frame_len=10).
    /// CTRL=0x44 (DIR=0, PRM=1, FC=4=UNCONF_USER_DATA) — outstation direction.
    /// CRC placeholders are 0x00 (CRC checking deferred per ADR-007 Decision 3).
    fn build_minimal_valid_frame(dest: u16, src: u16) -> Vec<u8> {
        // LENGTH=5 → compute_dnp3_frame_len(5) = Some(5+5+0) = Some(10).
        // Wait: frame_len = 5 + LENGTH + 2*blocks where blocks=ceil((LENGTH-5)/16).
        // LENGTH=5 → data bytes = 0 → blocks=0 → frame_len = 5+5+0 = 10.
        // Hmm: that's wrong. Let's use the standard: frame_len = 5 + LENGTH + 2*(data/16 rounded up).
        // But the frame itself is: [START START LENGTH CTRL DEST_L DEST_H SRC_L SRC_H CRC_L CRC_H].
        // For LENGTH=5: the frame has only the 10-byte header block (no user data block).
        // compute_dnp3_frame_len(5) = Some(10) — this is the minimum valid frame.
        let mut frame = vec![0u8; 10];
        frame[0] = 0x05; // START1
        frame[1] = 0x64; // START2
        frame[2] = 5; // LENGTH=5 (minimum valid per BC-2.15.004)
        frame[3] = 0x44; // CTRL=0x44: DIR=0, PRM=1, FC=4 (outstation direction)
        let [dl, dh] = dest.to_le_bytes();
        frame[4] = dl;
        frame[5] = dh;
        let [sl, sh] = src.to_le_bytes();
        frame[6] = sl;
        frame[7] = sh;
        // bytes 8-9: header CRC placeholder (0x00)
        frame
    }

    // -----------------------------------------------------------------------
    // D-1 — test_EC_junk_at_clean_boundary_increments_malformed_counters
    //
    // Path B (F-F5-003 REVISION 2): after a clean frame consume, the carry head
    // is immediately non-sync junk. The sync-check (resync) arm must fire and
    // unconditionally increment parse_errors and malformed_in_window.
    //
    // Setup: one complete valid 10-byte frame + 3 bytes of non-sync junk
    //        [0xAA, 0xBB, 0xCC] in the same on_data call.
    //
    // Expected:
    //   flow.parse_errors == 1     (one structural event from the junk)
    //   flow.malformed_in_window == 1
    //   flow.frame_count == 1      (the valid frame was consumed)
    //   flow.carry.len() == 0      (resync arm cleared the junk carry)
    //
    // Traces to: F-F5-003 REVISION 2 §R2-SECTION 1 Change 1 (resync arm unconditional
    //            increment); BC-2.15.016 EC-009 (new).
    // -----------------------------------------------------------------------

    /// Path B: junk at a clean frame boundary. Resync arm increments both
    /// counters unconditionally (F-F5-003 Change 1).
    #[test]
    fn test_EC_junk_at_clean_boundary_increments_malformed_counters() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Build: one complete valid 10-byte frame + 3 bytes of non-sync junk.
        // The valid frame will be consumed cleanly. The junk bytes [0xAA, 0xBB, 0xCC]
        // follow immediately after the frame boundary. No [0x05, 0x64] in the junk.
        let valid_frame = build_minimal_valid_frame(0x0003, 0x0001);
        assert_eq!(valid_frame.len(), 10, "valid frame must be 10 bytes");

        let mut data = valid_frame;
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC]); // 3 bytes of non-sync junk

        analyzer.on_data(key.clone(), &data, 0);

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        // Valid frame was consumed.
        assert_eq!(
            flow.frame_count, 1,
            "frame_count must be 1: the valid frame was consumed before the junk"
        );

        assert_eq!(
            flow.parse_errors, 1,
            "test_EC_junk_at_clean_boundary_increments_malformed_counters: \
             parse_errors must be 1 (one structural event from non-sync junk at clean boundary)"
        );

        assert_eq!(
            flow.malformed_in_window, 1,
            "malformed_in_window must be 1 (one sync-loss event from junk at clean boundary)"
        );

        // Resync arm cleared the junk (no [0x05, 0x64] in [0xAA, 0xBB, 0xCC]).
        assert_eq!(
            flow.carry.len(),
            0,
            "carry must be empty: resync arm cleared the non-sync junk carry"
        );
    }

    // -----------------------------------------------------------------------
    // D-2 — test_EC_length_gate_resync_no_double_count
    //
    // Strengthened version of test_EC_006: LENGTH<5 frame → LENGTH-gate arm fires
    // (parse_errors=1, malformed_in_window=1) + LENGTH-gate arm performs inline
    // resync (Change 2). The resync arm is NOT entered after a LENGTH-gate drain.
    // Final parse_errors must be exactly 1, NOT 2.
    //
    // Input: [0x05, 0x64, 0x02, ...zeros...] (10 bytes; LENGTH=2 < 5).
    //
    // Expected:
    //   flow.parse_errors == 1     (exactly one increment — no double-count)
    //   flow.malformed_in_window == 1
    //   flow.carry.len() == 0
    //
    // Change 2: LENGTH-gate arm performs inline resync → loop enters with
    //   empty or valid carry → resync arm NOT entered → parse_errors stays at 1.
    //
    // Traces to: F-F5-003 REVISION 2 Change 2 (LENGTH-gate inline resync);
    //            BC-2.15.016 EC-007 (updated).
    // -----------------------------------------------------------------------

    /// LENGTH-gate arm + inline resync: parse_errors must be exactly 1 (not 2).
    /// Catches any double-count regression from the resync arm firing after a
    /// LENGTH-gate drain (F-F5-003 Change 2 no-double-count invariant).
    #[test]
    fn test_EC_length_gate_resync_no_double_count() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Build a 10-byte frame with valid sync but LENGTH=2 (< 5 → validity gate REJECT).
        // Remaining bytes: 0x00 (no [0x05, 0x64] after the drain-1 of the 0x05 head).
        let mut bad_frame = vec![0u8; 10];
        bad_frame[0] = 0x05;
        bad_frame[1] = 0x64;
        bad_frame[2] = 2; // LENGTH=2 < 5 → compute_dnp3_frame_len returns None

        analyzer.on_data(key.clone(), &bad_frame, 0);

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        // LENGTH-gate arm performs inline resync (Change 2); parse_errors stays at 1.
        assert_eq!(
            flow.parse_errors, 1,
            "test_EC_length_gate_resync_no_double_count: parse_errors must be exactly 1 \
             (LENGTH-gate counts once; inline resync prevents resync arm from firing again)"
        );

        assert_eq!(
            flow.malformed_in_window, 1,
            "malformed_in_window must be exactly 1 (no double-count)"
        );

        assert_eq!(
            flow.carry.len(),
            0,
            "carry must be empty: inline resync found no sync in remaining bytes"
        );

        assert_eq!(
            flow.frame_count, 0,
            "frame_count must be 0: no valid frame was consumed"
        );
    }

    // -----------------------------------------------------------------------
    // D-3 — test_malformed_anomaly_boundary_junk_reaches_threshold
    //
    // Three on_data calls, each delivering one valid frame + junk bytes.
    // Each call produces one malformed accounting event (from the resync arm at
    // the junk-at-clean-boundary path). On the third call, malformed_in_window
    // reaches 3 (MALFORMED_ANOMALY_THRESHOLD=3) → T0814 emitted.
    //
    // Expected after third call:
    //   flow.parse_errors == 3
    //   flow.malformed_in_window == 3 (or 0 if window reset occurred — none here)
    //   flow.frame_count == 3
    //   flow.malformed_anomaly_emitted == true
    //   all_findings contains exactly one T0814 finding
    //
    // Traces to: F-F5-003 REVISION 2 Change 1 (resync arm unconditional increment);
    //            BC-2.15.024 PC3 (T0814 emission at threshold).
    // -----------------------------------------------------------------------

    /// Three valid-frame + junk calls accumulate malformed_in_window=3 → T0814.
    /// Each call triggers the resync arm (Change 1) which increments both counters.
    #[test]
    fn test_malformed_anomaly_boundary_junk_reaches_threshold() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Each call: valid 10-byte frame + 3 bytes of non-sync junk.
        // Timestamp=0 for all calls → within the same 300s window.
        for _ in 0..3 {
            let valid_frame = build_minimal_valid_frame(0x0003, 0x0001);
            let mut data = valid_frame;
            data.extend_from_slice(&[0xAA, 0xBB, 0xCC]);
            analyzer.on_data(key.clone(), &data, 0);
        }

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        assert_eq!(
            flow.parse_errors, 3,
            "test_malformed_anomaly_boundary_junk_reaches_threshold: \
             parse_errors must be 3 (one per junk-at-clean-boundary event)"
        );

        assert_eq!(
            flow.malformed_in_window, 3,
            "malformed_in_window must be 3 (three windowed events)"
        );

        assert_eq!(
            flow.frame_count, 3,
            "frame_count must be 3 (three valid frames consumed)"
        );

        assert!(
            flow.malformed_anomaly_emitted,
            "malformed_anomaly_emitted must be true after T0814 emission at threshold"
        );

        // Exactly one T0814 finding.
        let t0814_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();

        assert_eq!(
            t0814_count, 1,
            "all_findings must contain exactly ONE T0814 finding; got {t0814_count}"
        );
    }

    // -----------------------------------------------------------------------
    // D-4 — test_overflow_arm_preserves_valid_head_frame
    //
    // Data-loss guard + structural separation (REVISION 2 Change 3 replacement +
    // F-F5-003 REV 2 §R2-SECTION 4 / IMP-3):
    //   - The overflow arm must perform inline resync (NOT carry.clear()+return) so that
    //     a valid [0x05,0x64] head frame sitting in the carry after the cap is preserved
    //     and parsed, not silently discarded.
    //   - DISTINCT trailing junk bytes left in the carry AFTER the valid frame is consumed
    //     constitute a NEW independent sync-loss event. The unconditional resync arm
    //     (Change 1) increments parse_errors and malformed_in_window a SECOND time for
    //     those trailing bytes. No `overflow_counted_this_call`-style flag is used —
    //     structural path separation is the sole mechanism.
    //
    // Setup:
    //   1. Create flow entry. Pre-fill carry with 291 bytes:
    //      2 bytes 0xAA + valid 10-byte frame + 279 bytes 0xAA, total=291.
    //      The valid frame [0x05,0x64,0x05,0x44,...] is at offset 2.
    //   2. Deliver 2 bytes [0xBB, 0xCC]. remaining_capacity=1; overflow fires:
    //      1 byte (0xBB) accepted (carry becomes 292); 0xCC discarded.
    //      Event 1: parse_errors=1, malformed_in_window=1.
    //   3. Overflow arm inline resync: finds [0x05, 0x64] at offset 2 in the 292-byte carry.
    //      Drains bytes 0..2 → carry now starts at [0x05, 0x64, 0x05, 0x44, ...] + 280 bytes.
    //   4. Frame-walk: valid 10-byte frame at carry head → consumed → frame_count=1.
    //      Remaining carry: 280 bytes of 0xAA (279 original post-junk + 1 accepted 0xBB).
    //   5. Next frame-walk iteration: carry head is [0xAA, 0xAA] — not [0x05, 0x64].
    //      Resync arm (Change 1, unconditional): Event 2: parse_errors=2, malformed_in_window=2.
    //      Byte-walk finds no sync in 280 bytes of 0xAA → carry cleared → loop exits.
    //
    // Expected (REVISION 2 with flag removed — structural separation):
    //   flow.parse_errors == 2      (Event 1: overflow; Event 2: distinct trailing junk)
    //   flow.malformed_in_window == 2
    //   flow.frame_count >= 1       (valid head frame was PRESERVED and parsed)
    //
    // Contrast with all-junk carry-cap tests (test_carry_buffer_cap_at_292,
    // test_EC_003_carry_291_plus_2_overflow): those have NO valid frame in the carry, so
    // the overflow arm's inline resync clears the entire carry — the frame-walk loop exits
    // immediately (carry.len() < 3) — parse_errors stays at 1. The resync arm is NOT
    // entered for those traces.
    //
    // Traces to: F-F5-003 REVISION 2 §R2-SECTION 1 "Replacement: Overflow Arm
    //            Does Inline Resync"; REVISION 2 test (vi); REV 2 §R2-SECTION 4 / IMP-3.
    // -----------------------------------------------------------------------

    /// Overflow arm inline resync preserves a valid head frame; trailing junk after the
    /// consumed frame is counted as a distinct event (structural separation, no flag).
    #[test]
    fn test_overflow_arm_preserves_valid_head_frame() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Create flow entry.
        analyzer.on_data(key.clone(), &[0x05, 0x64], 0);

        // Pre-fill carry to 291 bytes:
        //   2 bytes 0xAA (pre-junk) + valid 10-byte frame + 279 bytes 0xAA (post-junk).
        // The valid frame [0x05,0x64,0x05,0x44,0x03,0x00,0x01,0x00,0x00,0x00] is at offset 2.
        let valid_frame_bytes: Vec<u8> = {
            let f = build_minimal_valid_frame(0x0003, 0x0001);
            assert_eq!(f.len(), 10, "minimal valid frame must be 10 bytes");
            f
        };

        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry.clear();
            // 2 pre-junk bytes (no sync pair before the valid frame)
            flow.carry.extend_from_slice(&[0xAA, 0xAA]);
            // valid frame at offset 2
            flow.carry.extend_from_slice(&valid_frame_bytes);
            // fill remainder to 291 with 0xAA
            let remaining = 291 - 2 - 10;
            flow.carry.extend(std::iter::repeat_n(0xAA, remaining));
            assert_eq!(
                flow.carry.len(),
                291,
                "pre-condition: carry must be 291 bytes"
            );
            // Reset parse_errors so only the overflow arm's increment counts.
            flow.parse_errors = 0;
            flow.malformed_in_window = 0;
            flow.frame_count = 0;
        }

        // Deliver 2 bytes → only 1 fits (292-291=1); 1 discarded. Overflow fires.
        // parse_errors=1, malformed_in_window=1.
        // Inline resync: [0x05,0x64] found at offset 2 in the 292-byte carry
        //   → drain 2 bytes → carry starts at [0x05,0x64,0x05,...].
        // Frame-walk: valid 10-byte frame → frame_count=1.
        analyzer.on_data(key.clone(), &[0xBB, 0xCC], 1);

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        // Event 1 (overflow) + Event 2 (distinct trailing junk after valid frame consumed):
        // structural path separation — no overflow_counted_this_call flag — each event counted.
        assert_eq!(
            flow.parse_errors, 2,
            "parse_errors must be 2: Event 1=overflow counted in overflow arm; \
             Event 2=distinct trailing 0xAA junk after valid frame consumed, counted by \
             unconditional resync arm (Change 1). No flag suppresses the second increment."
        );

        assert_eq!(
            flow.malformed_in_window, 2,
            "malformed_in_window must be 2 (two distinct sync-loss events, one per structural arm)"
        );

        // Core invariant: valid head frame was preserved and parsed (not discarded by clear+return).
        assert!(
            flow.frame_count >= 1,
            "test_overflow_arm_preserves_valid_head_frame: frame_count must be >= 1 \
             (overflow arm inline resync preserves valid head frame sitting in carry)"
        );
    }

    // -----------------------------------------------------------------------
    // D-5 — test_fake_sync_flood_crosses_malformed_threshold
    //
    // Principle 1 confirmation (REVISION 2 §R2-SECTION 2): one increment per
    // counter-arm entry. An attacker embedding N [0x05,0x64,invalid-LENGTH] triplets
    // triggers N LENGTH-gate entries → N increments. If N >= threshold → T0814.
    //
    // Setup: single on_data call with exactly 3 embedded [0x05,0x64,0x02] triplets
    //        (LENGTH=2 < 5) separated by padding bytes, such that each triplet
    //        triggers a distinct LENGTH-gate entry.
    //
    //   Payload: [0x05, 0x64, 0x02, 0xAA, 0xAA,
    //             0x05, 0x64, 0x02, 0xAA, 0xAA,
    //             0x05, 0x64, 0x02, 0xAA, 0xAA]
    //   (15 bytes; three fake-sync triplets; no valid sync between them after drain)
    //
    //   Walk:
    //     Iter 1: sync OK → LENGTH=2 → LENGTH-gate: parse_errors=1, drain(..1), inline resync.
    //             Inline resync: scans [0x64,0x02,0xAA,0xAA,0x05,0x64,...] for next [0x05,0x64].
    //             Found at offset 4 → drain(..4) → carry=[0x05,0x64,0x02,0xAA,0xAA,0x05,0x64,0x02,0xAA,0xAA].
    //     Iter 2: sync OK → LENGTH=2 → LENGTH-gate: parse_errors=2, drain(..1), inline resync.
    //             Inline resync: scans remainder, finds next [0x05,0x64] at offset 4 → drain(..4).
    //             carry=[0x05,0x64,0x02,0xAA,0xAA].
    //     Iter 3: sync OK → LENGTH=2 → LENGTH-gate: parse_errors=3, drain(..1), inline resync.
    //             Inline resync: scans [0x64,0x02,0xAA,0xAA] → no [0x05,0x64] → carry.clear().
    //     Iter 4: carry.len()==0 < 3 → break.
    //
    // Expected:
    //   flow.parse_errors == 3
    //   flow.malformed_in_window == 3
    //   flow.malformed_anomaly_emitted == true
    //   all_findings contains exactly one T0814 finding
    //   flow.frame_count == 0
    //
    // Traces to: F-F5-003 REVISION 2 §R2-SECTION 2 Principle 1;
    //            REVISION 2 test (vii).
    // -----------------------------------------------------------------------

    /// Principle 1: three embedded fake-sync [0x05,0x64,invalid-LENGTH] triplets
    /// → parse_errors=3 → T0814. Confirms intended detection of Crain-Sistrunk probes.
    /// Each triplet triggers exactly one LENGTH-gate entry (no double-count with Change 2).
    #[test]
    fn test_fake_sync_flood_crosses_malformed_threshold() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Three [0x05, 0x64, 0x02] triplets with 0xAA padding.
        // LENGTH=0x02 < 5 → each triggers a LENGTH-gate entry (parse_errors++).
        // With Change 2 (inline resync): after each LENGTH-gate drain+resync,
        // the carry is repositioned to the next [0x05,0x64] without entering the resync arm.
        // Exactly 3 LENGTH-gate entries → parse_errors=3.
        let data: Vec<u8> = vec![
            0x05, 0x64, 0x02, 0xAA, 0xAA, // triplet 1 + padding
            0x05, 0x64, 0x02, 0xAA, 0xAA, // triplet 2 + padding
            0x05, 0x64, 0x02, 0xAA, 0xAA, // triplet 3 + padding
        ];

        analyzer.on_data(key.clone(), &data, 0);

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        // With Change 2 (inline resync after each LENGTH-gate drain), resync arm
        // does NOT fire again → parse_errors stays at exactly 3.
        assert_eq!(
            flow.parse_errors, 3,
            "test_fake_sync_flood_crosses_malformed_threshold: \
             parse_errors must be 3 (one per LENGTH-gate entry — Principle 1 semantics)"
        );

        assert_eq!(
            flow.malformed_in_window, 3,
            "malformed_in_window must be 3 (three windowed LENGTH-gate events)"
        );

        assert!(
            flow.malformed_anomaly_emitted,
            "malformed_anomaly_emitted must be true after T0814 emission at threshold=3"
        );

        let t0814_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();

        assert_eq!(
            t0814_count, 1,
            "all_findings must contain exactly ONE T0814 finding; got {t0814_count}"
        );

        assert_eq!(
            flow.frame_count, 0,
            "frame_count must be 0 (no valid frames in the fake-sync flood payload)"
        );
    }
}
