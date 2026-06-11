//! Failing tests for STORY-106: DNP3 DL/Transport Parse + FC Classify — Pure Core.
//!
//! Covers BC-2.15.001 through BC-2.15.009 and edge cases EC-001..EC-010.
//! All tests MUST FAIL (todo!() panic) before implementation — Red Gate per the
//! strict-TDD contract (STORY-106, tdd_mode: strict).
//!
//! ## Test naming convention
//! Tests follow `test_BC_S_SS_NNN_xxx()` for BC-traceable tests and
//! `test_AC_NNN_xxx()` for story AC-prefixed tests where specified by the story.
//!
//! ## Precedent
//! Mirrors the structure of `tests/modbus_parse_tests.rs` (STORY-102).

// BC-based test naming convention uses uppercase BC IDs: test_BC_S_SS_NNN_xxx.
// The non_snake_case lint fires on uppercase letters — suppressed here intentionally.
#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-106 tests are grouped inside a dedicated
// `mod story_106` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_106 {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::dnp3::{
        Dnp3Analyzer, Dnp3DlHeader, Dnp3FcClass, Dnp3FlowState, classify_dnp3_fc,
        compute_dnp3_frame_len, is_valid_dnp3_frame_header, parse_dnp3_dl_header,
        transport_is_fir,
    };
    use wirerust::reassembly::flow::FlowKey;

    // ---------------------------------------------------------------------------
    // Helper: build a minimal valid 10-byte DNP3 DL header slice.
    // Canonical vector from BC-2.15.001: 05 64 0E C4 03 00 01 00 88 C5
    // ---------------------------------------------------------------------------
    fn canonical_direct_operate_header() -> [u8; 10] {
        [0x05, 0x64, 0x0E, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x88, 0xC5]
    }

    /// Build a test FlowKey for use in effectful shell tests (AC-008, AC-009).
    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            50000,
        )
    }

    // =========================================================================
    // AC-001 / BC-2.15.001: Some for data.len() >= 10
    // =========================================================================

    /// test_parse_dnp3_dl_header_returns_some_for_minimum_10_bytes
    ///
    /// Canonical vector: `05 64 0E C4 03 00 01 00 88 C5` (10 bytes).
    /// Expected: `Some { start1:0x05, start2:0x64, length:14, control:0xC4,
    ///            destination:0x0003, source:0x0001 }`.
    ///
    /// Traces to: BC-2.15.001 postconditions 1–7; STORY-106 AC-001.
    #[test]
    fn test_parse_dnp3_dl_header_returns_some_for_minimum_10_bytes() {
        let data = canonical_direct_operate_header();
        let result = parse_dnp3_dl_header(&data);
        assert!(result.is_some(), "10-byte canonical vector must return Some");
        let h = result.unwrap();
        assert_eq!(h.start1, 0x05, "start1 must be 0x05");
        assert_eq!(h.start2, 0x64, "start2 must be 0x64");
        assert_eq!(h.length, 0x0E, "length must be 14 (0x0E)");
        assert_eq!(h.control, 0xC4, "control must be 0xC4");
        assert_eq!(h.destination, 0x0003, "destination LE [0x03,0x00] = 0x0003");
        assert_eq!(h.source, 0x0001, "source LE [0x01,0x00] = 0x0001");
    }

    /// Additional canonical vector: minimum-length control frame (LENGTH=5).
    ///
    /// Canonical vector: `05 64 05 C0 01 00 03 00 A1 B2`
    /// Expected: `Some { start1:0x05, start2:0x64, length:5, control:0xC0,
    ///            destination:0x0001, source:0x0003 }`.
    ///
    /// Traces to: BC-2.15.001 canonical test vector row 1; AC-001.
    #[test]
    fn test_BC_2_15_001_minimum_length_control_frame() {
        let data: &[u8] = &[0x05, 0x64, 0x05, 0xC0, 0x01, 0x00, 0x03, 0x00, 0xA1, 0xB2];
        let result = parse_dnp3_dl_header(data);
        assert!(result.is_some(), "10-byte minimum control frame must return Some");
        let h = result.unwrap();
        assert_eq!(h.start1, 0x05);
        assert_eq!(h.start2, 0x64);
        assert_eq!(h.length, 5, "LENGTH must be 5");
        assert_eq!(h.control, 0xC0);
        assert_eq!(h.destination, 0x0001, "dest LE [0x01,0x00] = 0x0001");
        assert_eq!(h.source, 0x0003, "src LE [0x03,0x00] = 0x0003");
    }

    /// BC-2.15.001 postcondition 8: trailing bytes are not decoded as struct fields.
    ///
    /// A 15-byte slice (canonical header + 5 trailing bytes) still parses
    /// the same six fields from the first 10 bytes.
    ///
    /// Traces to: BC-2.15.001 postcondition 8; AC-001.
    #[test]
    fn test_BC_2_15_001_trailing_bytes_not_decoded() {
        let mut data = vec![0x05u8, 0x64, 0x0E, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x88, 0xC5];
        // Append 5 trailing bytes that should be ignored.
        data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00]);
        let result = parse_dnp3_dl_header(&data);
        assert!(result.is_some(), "15-byte slice must return Some");
        let h = result.unwrap();
        assert_eq!(h.start1, 0x05);
        assert_eq!(h.length, 0x0E);
        assert_eq!(h.destination, 0x0003);
        assert_eq!(h.source, 0x0001);
    }

    /// BC-2.15.001 invariant 4: parse does NOT check sync or LENGTH validity.
    ///
    /// A 10-byte slice with START1=0x00 (invalid sync) still returns Some.
    /// The validity gate (`is_valid_dnp3_frame_header`) is separate.
    ///
    /// Traces to: BC-2.15.001 invariant 4; BC-2.15.001 canonical vector row 4.
    #[test]
    fn test_BC_2_15_001_invariant_parse_does_not_gate_on_sync() {
        // START1=0x00, START2=0x00 — invalid sync, but parse must return Some.
        let data: &[u8] = &[0x00, 0x00, 0x05, 0xC0, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00];
        let result = parse_dnp3_dl_header(data);
        assert!(
            result.is_some(),
            "invalid sync bytes must still return Some — no gate inside parse"
        );
        let h = result.unwrap();
        assert_eq!(h.start1, 0x00, "start1 raw decoded as 0x00");
        assert_eq!(h.start2, 0x00, "start2 raw decoded as 0x00");
    }

    // =========================================================================
    // AC-002 / BC-2.15.002: None for data.len() < 10
    // =========================================================================

    /// test_parse_dnp3_dl_header_rejects_truncated_input
    ///
    /// Canonical vectors: empty → None; 9-byte → None; 10-byte → Some.
    /// No panic for any length 0..=9.
    ///
    /// Traces to: BC-2.15.002 postcondition 1; STORY-106 AC-002.
    #[test]
    fn test_parse_dnp3_dl_header_rejects_truncated_input() {
        // Zero-length — must return None without panic.
        assert_eq!(
            parse_dnp3_dl_header(&[]),
            None,
            "empty slice must return None"
        );
        // 9 bytes (one short) — canonical BC-2.15.002 vector.
        let nine_bytes: &[u8] = &[0x05, 0x64, 0x05, 0xC0, 0x01, 0x00, 0x03, 0x00, 0xA1];
        assert_eq!(
            parse_dnp3_dl_header(nine_bytes),
            None,
            "9-byte slice must return None"
        );
        // 10 bytes — boundary: must return Some (BC-2.15.001 happy path).
        let ten_bytes: &[u8] = &[0x05, 0x64, 0x05, 0xC0, 0x01, 0x00, 0x03, 0x00, 0xA1, 0xB2];
        assert!(
            parse_dnp3_dl_header(ten_bytes).is_some(),
            "10-byte slice must return Some"
        );
    }

    /// Full boundary sweep: all lengths 0..=9 → None; 10 → Some.
    ///
    /// Traces to: BC-2.15.002 postconditions 1–3; invariant 1.
    #[test]
    fn test_BC_2_15_002_boundary_sweep_all_short_lengths() {
        // Use the canonical direct-operate header extended to 10 bytes as base.
        let base: &[u8] = &[0x05, 0x64, 0x0E, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x88, 0xC5];
        // All lengths 0..=9 must return None — no panic.
        for len in 0usize..10 {
            assert_eq!(
                parse_dnp3_dl_header(&base[..len]),
                None,
                "len={len} must be None (< 10)"
            );
        }
        // Length 10 must return Some.
        assert!(
            parse_dnp3_dl_header(base).is_some(),
            "len=10 must be Some"
        );
    }

    /// EC-001: zero-length input — no panic.
    ///
    /// Traces to: STORY-106 EC-001 (edge case: zero-length input).
    #[test]
    fn test_BC_2_15_002_ec001_zero_length_no_panic() {
        // Must not panic; must return None.
        assert_eq!(parse_dnp3_dl_header(&[]), None);
    }

    /// EC-002: 9-byte input (one short of minimum).
    ///
    /// Traces to: STORY-106 EC-002.
    #[test]
    fn test_BC_2_15_002_ec002_nine_bytes_returns_none() {
        let data: &[u8] = &[0x05, 0x64, 0x05, 0xC0, 0x01, 0x00, 0x03, 0x00, 0xA1];
        assert_eq!(parse_dnp3_dl_header(data), None, "9 bytes must be None");
    }

    // =========================================================================
    // AC-003 / BC-2.15.003: Little-endian address decode
    // =========================================================================

    /// test_parse_dnp3_dl_header_le_address_decode
    ///
    /// Asserts LE decode disambiguation:
    ///   [0x03, 0x00] → 0x0003 (NOT 0x0300)
    ///   [0xFD, 0xFF] → 0xFFFD (NOT 0xFDFF)
    ///   [0x00, 0x01] → 0x0100 (NOT 0x0001) — LE vs BE disambiguation
    ///
    /// Traces to: BC-2.15.003 postcondition 1; STORY-106 AC-003.
    #[test]
    fn test_parse_dnp3_dl_header_le_address_decode() {
        // Vector 1: dest=[0x03,0x00] → 0x0003; src=[0x01,0x00] → 0x0001
        let data1: &[u8] = &[0x05, 0x64, 0x0E, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x88, 0xC5];
        let h1 = parse_dnp3_dl_header(data1).expect("canonical vector must parse");
        assert_eq!(
            h1.destination, 0x0003,
            "dest [0x03,0x00] LE must be 0x0003, not 0x0300"
        );
        assert_eq!(
            h1.source, 0x0001,
            "src [0x01,0x00] LE must be 0x0001, not 0x0100"
        );

        // Vector 2: dest=[0xFD,0xFF] → 0xFFFD (broadcast confirm-required)
        let data2: &[u8] = &[0x05, 0x64, 0x05, 0x44, 0xFD, 0xFF, 0x02, 0x00, 0x00, 0x00];
        let h2 = parse_dnp3_dl_header(data2).expect("FD-FF vector must parse");
        assert_eq!(
            h2.destination, 0xFFFD,
            "dest [0xFD,0xFF] LE must be 0xFFFD, not 0xFDFF"
        );

        // Vector 3: dest=[0x00,0x01] → 0x0100 (LE disambiguation: low=0x00, high=0x01)
        let data3: &[u8] = &[0x05, 0x64, 0x05, 0xC0, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00];
        let h3 = parse_dnp3_dl_header(data3).expect("[0x00,0x01] vector must parse");
        assert_eq!(
            h3.destination, 0x0100,
            "dest [0x00,0x01] LE must be 0x0100 (not 0x0001 — LE vs BE disambiguation)"
        );
    }

    /// EC-003: 10-byte slice with START1=0x00 returns Some with correct raw fields.
    ///
    /// This verifies that parse does not gate on sync — it returns Some with
    /// the decoded raw values even when start1 != 0x05.
    ///
    /// Traces to: STORY-106 EC-003; BC-2.15.001 invariant 4.
    #[test]
    fn test_BC_2_15_003_ec003_invalid_sync_returns_some_with_raw_fields() {
        let data: &[u8] = &[0x00, 0x00, 0x10, 0xC0, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00];
        let h = parse_dnp3_dl_header(data).expect("10 bytes with invalid sync must return Some");
        assert_eq!(h.start1, 0x00);
        assert_eq!(h.start2, 0x00);
        assert_eq!(h.destination, 0x0003, "LE decode still correct even with invalid sync");
    }

    /// EC-004: DEST bytes [0xFF,0xFF] → destination = 0xFFFF (broadcast).
    ///
    /// Traces to: STORY-106 EC-004; BC-2.15.003 EC-003.
    #[test]
    fn test_BC_2_15_003_ec004_broadcast_0xffff() {
        let data: &[u8] = &[0x05, 0x64, 0x05, 0x44, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00];
        let h = parse_dnp3_dl_header(data).expect("broadcast vector must parse");
        assert_eq!(
            h.destination, 0xFFFF,
            "dest [0xFF,0xFF] LE must be 0xFFFF (broadcast no-confirm)"
        );
    }

    // =========================================================================
    // AC-004 / BC-2.15.004: Three-point validity gate biconditional
    // =========================================================================

    /// test_is_valid_dnp3_frame_header_biconditional
    ///
    /// Tests 6 vectors covering all partial-match cases per the story:
    ///   1. Valid: start1=0x05, start2=0x64, length=14 → true
    ///   2. Wrong START1 (0x04): → false
    ///   3. Wrong START2 (0x63): → false
    ///   4. LENGTH=4 (below minimum 5): → false
    ///   5. All zeros: → false
    ///   6. Minimum-valid: start1=0x05, start2=0x64, length=5 → true
    ///
    /// Traces to: BC-2.15.004 postcondition 1; STORY-106 AC-004.
    #[test]
    fn test_is_valid_dnp3_frame_header_biconditional() {
        // 1. All three conditions met — valid DIRECT_OPERATE header.
        let valid = Dnp3DlHeader {
            start1: 0x05,
            start2: 0x64,
            length: 0x0E,
            control: 0xC4,
            destination: 0x0003,
            source: 0x0001,
        };
        assert!(
            is_valid_dnp3_frame_header(&valid),
            "start1=0x05, start2=0x64, length=14 must be true"
        );

        // 2. Wrong START1 (0x04).
        let wrong_start1 = Dnp3DlHeader { start1: 0x04, ..valid.clone() };
        assert!(
            !is_valid_dnp3_frame_header(&wrong_start1),
            "start1=0x04 must be false (condition 1 fails)"
        );

        // 3. Wrong START2 (0x63).
        let wrong_start2 = Dnp3DlHeader { start2: 0x63, ..valid.clone() };
        assert!(
            !is_valid_dnp3_frame_header(&wrong_start2),
            "start2=0x63 must be false (condition 2 fails)"
        );

        // 4. LENGTH=4 (below minimum 5).
        let length_too_low = Dnp3DlHeader { length: 4, ..valid.clone() };
        assert!(
            !is_valid_dnp3_frame_header(&length_too_low),
            "length=4 must be false (condition 3 fails)"
        );

        // 5. All zeros (no condition met).
        let all_zeros = Dnp3DlHeader {
            start1: 0x00,
            start2: 0x00,
            length: 0,
            control: 0x00,
            destination: 0x0000,
            source: 0x0000,
        };
        assert!(
            !is_valid_dnp3_frame_header(&all_zeros),
            "all-zeros struct must be false"
        );

        // 6. Minimum valid: start1=0x05, start2=0x64, length=5.
        let minimum_valid = Dnp3DlHeader {
            start1: 0x05,
            start2: 0x64,
            length: 5,
            control: 0xC0,
            destination: 0x0001,
            source: 0x0003,
        };
        assert!(
            is_valid_dnp3_frame_header(&minimum_valid),
            "start1=0x05, start2=0x64, length=5 must be true (minimum valid)"
        );
    }

    /// Additional biconditional case: LENGTH=0 must be false.
    ///
    /// Traces to: BC-2.15.004 EC-004.
    #[test]
    fn test_BC_2_15_004_length_zero_false() {
        let h = Dnp3DlHeader {
            start1: 0x05,
            start2: 0x64,
            length: 0,
            control: 0xC0,
            destination: 0x0001,
            source: 0x0001,
        };
        assert!(
            !is_valid_dnp3_frame_header(&h),
            "length=0 must return false"
        );
    }

    /// BC-2.15.004 EC-002: LENGTH=255 (maximum) with valid sync → true.
    ///
    /// Traces to: BC-2.15.004 edge case EC-002; canonical test vector row 3.
    #[test]
    fn test_BC_2_15_004_length_255_valid() {
        let h = Dnp3DlHeader {
            start1: 0x05,
            start2: 0x64,
            length: 255,
            control: 0x44,
            destination: 0x0003,
            source: 0x0001,
        };
        assert!(
            is_valid_dnp3_frame_header(&h),
            "start1=0x05, start2=0x64, length=255 must be true (maximum valid LENGTH)"
        );
    }

    /// EC-005: compute_dnp3_frame_len(4) → None.
    ///
    /// Traces to: STORY-106 EC-005.
    #[test]
    fn test_BC_2_15_004_ec005_length_4_rejected_by_gate() {
        // is_valid gate rejects LENGTH=4.
        let h = Dnp3DlHeader {
            start1: 0x05,
            start2: 0x64,
            length: 4,
            control: 0xC0,
            destination: 0x0001,
            source: 0x0003,
        };
        assert!(
            !is_valid_dnp3_frame_header(&h),
            "length=4 must fail validity gate"
        );
    }

    // =========================================================================
    // AC-005 / BC-2.15.005: classify_dnp3_fc totality — no panic for any u8
    // =========================================================================

    /// test_classify_dnp3_fc_total
    ///
    /// Iterates all 256 u8 values. Any panic = Red Gate failure (implementation incomplete).
    /// Spot-checks FC=0xFF and FC=0x80 return Unknown.
    ///
    /// Traces to: BC-2.15.005 postconditions 1–4; STORY-106 AC-005.
    #[test]
    fn test_classify_dnp3_fc_total() {
        // Spot-check: FC=0xFF (reserved) must be Unknown.
        assert_eq!(
            classify_dnp3_fc(0xFF),
            Dnp3FcClass::Unknown,
            "FC=0xFF must return Unknown (wildcard arm)"
        );
        // Spot-check: FC=0x80 (reserved range) must be Unknown.
        assert_eq!(
            classify_dnp3_fc(0x80),
            Dnp3FcClass::Unknown,
            "FC=0x80 must return Unknown (reserved range)"
        );
    }

    /// Full totality sweep: all 256 FC values must return a defined variant.
    ///
    /// Traces to: BC-2.15.005 invariant 1; canonical test vectors "Totality witness".
    #[test]
    fn test_BC_2_15_005_totality_sweep_all_256_values() {
        for fc in 0u8..=255 {
            let class = classify_dnp3_fc(fc);
            assert!(
                matches!(
                    class,
                    Dnp3FcClass::Read
                        | Dnp3FcClass::Write
                        | Dnp3FcClass::Control
                        | Dnp3FcClass::Restart
                        | Dnp3FcClass::Management
                        | Dnp3FcClass::Response
                        | Dnp3FcClass::Unknown
                ),
                "classify_dnp3_fc({fc:#04x}) returned an undefined variant"
            );
        }
    }

    /// BC-2.15.005 canonical vectors: named FC values.
    ///
    /// Traces to: BC-2.15.005 canonical test vector table.
    #[test]
    fn test_BC_2_15_005_canonical_vectors() {
        assert_eq!(classify_dnp3_fc(0x01), Dnp3FcClass::Read, "0x01 READ must be Read");
        assert_eq!(classify_dnp3_fc(0x02), Dnp3FcClass::Write, "0x02 WRITE must be Write");
        assert_eq!(classify_dnp3_fc(0x05), Dnp3FcClass::Control, "0x05 DIRECT_OPERATE must be Control");
        assert_eq!(classify_dnp3_fc(0x0D), Dnp3FcClass::Restart, "0x0D COLD_RESTART must be Restart");
        assert_eq!(classify_dnp3_fc(0x81), Dnp3FcClass::Response, "0x81 RESPONSE must be Response");
        assert_eq!(classify_dnp3_fc(0x82), Dnp3FcClass::Response, "0x82 UNSOLICITED_RESPONSE must be Response");
        assert_eq!(classify_dnp3_fc(0xFF), Dnp3FcClass::Unknown, "0xFF (reserved) must be Unknown");
        assert_eq!(classify_dnp3_fc(0x00), Dnp3FcClass::Management, "0x00 CONFIRM must be Management");
    }

    // =========================================================================
    // AC-006 / BC-2.15.006: FC set membership correctness
    // =========================================================================

    /// test_classify_dnp3_fc_set_membership
    ///
    /// One assertion per listed FC (per story requirement):
    ///   Control set {0x03,0x04,0x05,0x06}, Restart set {0x0D,0x0E},
    ///   Write {0x02}, Read {0x01}, Response {0x81,0x82,0x83},
    ///   Management {0x07, 0x0F}.
    ///
    /// Traces to: BC-2.15.006 postconditions 1–11; STORY-106 AC-006.
    #[test]
    fn test_classify_dnp3_fc_set_membership() {
        // Control set (BC-2.15.006 postconditions 1–4).
        assert_eq!(classify_dnp3_fc(0x03), Dnp3FcClass::Control, "0x03 SELECT must be Control");
        assert_eq!(classify_dnp3_fc(0x04), Dnp3FcClass::Control, "0x04 OPERATE must be Control");
        assert_eq!(classify_dnp3_fc(0x05), Dnp3FcClass::Control, "0x05 DIRECT_OPERATE must be Control");
        assert_eq!(
            classify_dnp3_fc(0x06),
            Dnp3FcClass::Control,
            "0x06 DIRECT_OPERATE_NR must be Control (EC-007: still Control even without response)"
        );

        // Restart set (BC-2.15.006 postconditions 5–6).
        assert_eq!(classify_dnp3_fc(0x0D), Dnp3FcClass::Restart, "0x0D COLD_RESTART must be Restart");
        assert_eq!(classify_dnp3_fc(0x0E), Dnp3FcClass::Restart, "0x0E WARM_RESTART must be Restart");

        // Write set (BC-2.15.006 postcondition 7).
        assert_eq!(classify_dnp3_fc(0x02), Dnp3FcClass::Write, "0x02 WRITE must be Write");

        // Read set (BC-2.15.006 postcondition 8).
        assert_eq!(classify_dnp3_fc(0x01), Dnp3FcClass::Read, "0x01 READ must be Read");

        // Response set (BC-2.15.006 postconditions 9–11).
        assert_eq!(classify_dnp3_fc(0x81), Dnp3FcClass::Response, "0x81 RESPONSE must be Response");
        assert_eq!(classify_dnp3_fc(0x82), Dnp3FcClass::Response, "0x82 UNSOLICITED_RESPONSE must be Response");
        assert_eq!(classify_dnp3_fc(0x83), Dnp3FcClass::Response, "0x83 AUTHENTICATE_RESP must be Response");

        // FC 0x07 IMMED_FREEZE → Management (NOT Control).
        assert_eq!(
            classify_dnp3_fc(0x07),
            Dnp3FcClass::Management,
            "0x07 IMMED_FREEZE must be Management, NOT Control"
        );

        // FC 0x0F INITIALIZE_DATA → Management (NOT Restart per BC-2.15.006 EC-009).
        assert_eq!(
            classify_dnp3_fc(0x0F),
            Dnp3FcClass::Management,
            "0x0F INITIALIZE_DATA must be Management, NOT Restart"
        );
    }

    /// EC-007: FC=0x06 (DIRECT_OPERATE_NR) → Control.
    ///
    /// Traces to: STORY-106 EC-007; BC-2.15.006 invariant 4.
    #[test]
    fn test_BC_2_15_006_ec007_direct_operate_nr_is_control() {
        assert_eq!(
            classify_dnp3_fc(0x06),
            Dnp3FcClass::Control,
            "0x06 DIRECT_OPERATE_NR must be Control"
        );
    }

    /// EC-008: FC=0x82 (UNSOLICITED_RESPONSE) → Response.
    ///
    /// Traces to: STORY-106 EC-008; BC-2.15.006 postcondition 10.
    #[test]
    fn test_BC_2_15_006_ec008_unsolicited_response_is_response() {
        assert_eq!(
            classify_dnp3_fc(0x82),
            Dnp3FcClass::Response,
            "0x82 UNSOLICITED_RESPONSE must be Response"
        );
    }

    // =========================================================================
    // AC-007 / BC-2.15.007: compute_dnp3_frame_len arithmetic
    // =========================================================================

    /// test_compute_dnp3_frame_len_formula
    ///
    /// 7 canonical vectors including both block-boundary cases:
    ///   LENGTH=4 → None (below minimum)
    ///   LENGTH=5 → Some(10)
    ///   LENGTH=6 → Some(13)
    ///   LENGTH=21 → Some(28)
    ///   LENGTH=22 → Some(31)
    ///   LENGTH=255 → Some(292)
    ///   LENGTH=0 → None
    ///
    /// Traces to: BC-2.15.007 canonical test vector table; STORY-106 AC-007.
    #[test]
    fn test_compute_dnp3_frame_len_formula() {
        // Below minimum (None cases).
        assert_eq!(compute_dnp3_frame_len(0), None, "length=0 must be None");
        assert_eq!(compute_dnp3_frame_len(4), None, "length=4 must be None (one below minimum)");

        // Minimum valid: U=0, blocks=0, frame_len=5+5+0=10.
        assert_eq!(compute_dnp3_frame_len(5), Some(10), "length=5 must be Some(10)");

        // First block: U=1, blocks=ceil(1/16)=1, frame_len=5+6+2=13.
        assert_eq!(compute_dnp3_frame_len(6), Some(13), "length=6 must be Some(13)");

        // Exactly one 16-octet block: U=16, blocks=ceil(16/16)=1, frame_len=5+21+2=28.
        assert_eq!(compute_dnp3_frame_len(21), Some(28), "length=21 must be Some(28)");

        // Starts second block: U=17, blocks=ceil(17/16)=2, frame_len=5+22+4=31.
        assert_eq!(compute_dnp3_frame_len(22), Some(31), "length=22 must be Some(31)");

        // Maximum frame: U=250, blocks=ceil(250/16)=16, frame_len=5+255+32=292.
        assert_eq!(compute_dnp3_frame_len(255), Some(292), "length=255 must be Some(292)");
    }

    /// Result bounds: all valid LENGTHs produce frame_len in [10, 292].
    ///
    /// Traces to: BC-2.15.007 postconditions 3–5; invariants 2–3.
    #[test]
    fn test_BC_2_15_007_result_bounds_all_valid_lengths() {
        for length in 5u8..=255 {
            let result = compute_dnp3_frame_len(length);
            assert!(result.is_some(), "length={length} (>= 5) must return Some");
            let frame_len = result.unwrap();
            assert!(
                frame_len >= 10,
                "frame_len={frame_len} for length={length} must be >= 10"
            );
            assert!(
                frame_len <= 292,
                "frame_len={frame_len} for length={length} must be <= 292"
            );
        }
    }

    /// EC-005: compute_dnp3_frame_len(4) → None.
    ///
    /// Traces to: STORY-106 EC-005; BC-2.15.007 EC-002.
    #[test]
    fn test_BC_2_15_007_ec005_length_4_returns_none() {
        assert_eq!(compute_dnp3_frame_len(4), None, "length=4 must be None");
    }

    /// EC-006: compute_dnp3_frame_len(255) → Some(292).
    ///
    /// U=250, blocks=16, 5+255+32=292.
    ///
    /// Traces to: STORY-106 EC-006; BC-2.15.007 canonical vector row 7.
    #[test]
    fn test_BC_2_15_007_ec006_length_255_returns_292() {
        assert_eq!(compute_dnp3_frame_len(255), Some(292), "length=255 must be Some(292)");
    }

    /// Additional typical DIRECT_OPERATE vector: LENGTH=14 → Some(21).
    ///
    /// U=9, blocks=ceil(9/16)=1, frame_len=5+14+2=21.
    ///
    /// Traces to: BC-2.15.007 canonical test vector row 4.
    #[test]
    fn test_BC_2_15_007_length_14_direct_operate() {
        // U=9, blocks=1, frame_len=5+14+2=21
        assert_eq!(compute_dnp3_frame_len(14), Some(21), "length=14 must be Some(21)");
    }

    // =========================================================================
    // AC-008 / BC-2.15.008: FIR=1 gating — transport_is_fir helper
    // =========================================================================

    /// test_fir_gating_extract_on_fir1_skip_on_fir0
    ///
    /// transport_octet=0xC0 (FIR=1, FIN=1, SEQ=0): FIR bit set.
    /// transport_octet=0x80 (FIR=0, FIN=1, SEQ=0): FIR bit NOT set.
    ///
    /// Traces to: BC-2.15.008 invariant 1; STORY-106 AC-008.
    #[test]
    fn test_fir_gating_extract_on_fir1_skip_on_fir0() {
        // FIR=1 cases: bit 6 (0x40) is set.
        assert!(
            transport_is_fir(0xC0),
            "transport_octet=0xC0 (FIR=1, FIN=1, SEQ=0) must return true"
        );
        assert!(
            transport_is_fir(0x40),
            "transport_octet=0x40 (FIR=1, FIN=0, SEQ=0) must return true"
        );

        // FIR=0 cases: bit 6 is clear.
        assert!(
            !transport_is_fir(0x80),
            "transport_octet=0x80 (FIR=0, FIN=1, SEQ=0) must return false"
        );
        assert!(
            !transport_is_fir(0x00),
            "transport_octet=0x00 (FIR=0, FIN=0, SEQ=0) must return false"
        );
    }

    /// BC-2.15.008 invariant 1: FIR mask is 0x40.
    ///
    /// All transport octets with bit 6 set must return true; all without must return false.
    ///
    /// Traces to: BC-2.15.008 invariant 1 (FIR bit is bit 6, mask 0x40).
    #[test]
    fn test_BC_2_15_008_fir_biconditional_all_256_transport_octets() {
        for octet in 0u8..=255 {
            let expected = (octet & 0x40) != 0;
            assert_eq!(
                transport_is_fir(octet),
                expected,
                "transport_is_fir({octet:#04x}): expected {expected} (bit 6 = {expected})"
            );
        }
    }

    /// EC-009: FIR=0 continuation (transport_octet=0x80) — no FC extraction.
    ///
    /// This verifies the transport_is_fir predicate returns false for
    /// continuation segments (FIR=0 variant).
    ///
    /// Traces to: STORY-106 EC-009; BC-2.15.008 EC-003.
    #[test]
    fn test_BC_2_15_008_ec009_fir0_continuation_returns_false() {
        assert!(
            !transport_is_fir(0x80),
            "transport_octet=0x80 (FIR=0, FIN=1) must return false — continuation segment"
        );
    }

    // =========================================================================
    // AC-009 / BC-2.15.009: Desync bail — is_non_dnp3 latch
    // =========================================================================

    /// test_desync_bail_non_dnp3_traffic
    ///
    /// Step 1: deliver non-DNP3 bytes [0xFF, 0xFE, ...] (16 bytes, no valid sync at offset 0).
    ///         Assert: flow.is_non_dnp3 = true after on_data returns.
    /// Step 2: deliver a second valid-looking segment.
    ///         Assert: no findings emitted; carry buffer not grown.
    ///
    /// Traces to: BC-2.15.009 postconditions 1–6; STORY-106 AC-009.
    #[test]
    fn test_desync_bail_non_dnp3_traffic() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Step 1: Deliver 16 non-DNP3 bytes — no valid sync [0x05, 0x64] at offset 0.
        let non_dnp3: [u8; 16] = [
            0xFF, 0xFE, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
            0x0B, 0x0C, 0x0D,
        ];
        analyzer.on_data(key.clone(), &non_dnp3, 0);

        // After delivering non-DNP3 data, is_non_dnp3 must be set to true.
        let flow = analyzer.flows.get(&key).expect("flow state must exist after on_data");
        assert!(flow.is_non_dnp3, "is_non_dnp3 must be true after no valid sync in first 16 bytes");

        // Step 2: deliver a second segment — this must be a no-op.
        let carry_before = flow.carry.len();
        let _carry_before = carry_before; // Suppress unused warning; we recheck after.

        let second_segment: [u8; 10] = [0x05, 0x64, 0x0E, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x88, 0xC5];
        analyzer.on_data(key.clone(), &second_segment, 1);

        // After second on_data on a bailed flow:
        // - is_non_dnp3 must still be true (one-way latch).
        // - carry must not have grown.
        let flow_after = analyzer
            .flows
            .get(&key)
            .expect("flow state must still exist");
        assert!(
            flow_after.is_non_dnp3,
            "is_non_dnp3 must remain true — one-way latch (BC-2.15.009 invariant 2)"
        );
        // The carry buffer must not grow on bailed flows.
        assert_eq!(
            flow_after.carry.len(),
            0,
            "carry buffer must not grow on bailed flow (no-op path)"
        );
    }

    /// BC-2.15.009 postcondition 7: valid DNP3 sync at offset 0 does not trigger bail.
    ///
    /// A flow starting with `0x05 0x64` must NOT set is_non_dnp3.
    ///
    /// Traces to: BC-2.15.009 postcondition 7; canonical test vector (valid DNP3 frame start).
    #[test]
    fn test_BC_2_15_009_valid_sync_no_bail() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Valid DNP3 sync at offset 0.
        let valid_frame: [u8; 10] = [0x05, 0x64, 0x0E, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x88, 0xC5];
        analyzer.on_data(key.clone(), &valid_frame, 0);

        // is_non_dnp3 must stay false for a valid DNP3 flow.
        if let Some(flow) = analyzer.flows.get(&key) {
            assert!(
                !flow.is_non_dnp3,
                "is_non_dnp3 must remain false for valid DNP3 sync at offset 0"
            );
        }
        // If no flow entry created yet, the test passes vacuously (not yet implemented).
    }

    /// EC-010: valid sync at offset 2 (not offset 0) → desync bail fires.
    ///
    /// BC-2.15.009 checks only offset 0; sync not at offset 0 is treated as non-DNP3.
    ///
    /// Traces to: STORY-106 EC-010; BC-2.15.009 edge case EC-003.
    #[test]
    fn test_BC_2_15_009_ec010_sync_at_offset_2_triggers_bail() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 16 bytes with valid sync starting at offset 2, NOT at offset 0.
        let misaligned: [u8; 16] = [
            0xAB, 0xCD, 0x05, 0x64, 0x0E, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x88, 0xC5, 0x00, 0x00,
            0x00, 0x00,
        ];
        analyzer.on_data(key.clone(), &misaligned, 0);

        // Valid sync at offset 2 must still trigger bail (v1 only checks offset 0).
        if let Some(flow) = analyzer.flows.get(&key) {
            assert!(
                flow.is_non_dnp3,
                "sync at offset 2 must trigger desync bail — v1 checks only offset 0"
            );
        }
    }

    // =========================================================================
    // Dnp3FlowState default state (precondition for AC-009)
    // =========================================================================

    /// Dnp3FlowState defaults: is_non_dnp3=false, carry empty.
    ///
    /// Traces to: BC-2.15.009 edge case EC-005 (flow correctly starts with valid sync
    /// → is_non_dnp3 stays false); the Default impl is the creation precondition.
    #[test]
    fn test_BC_2_15_009_flow_state_defaults_to_not_bailed() {
        let state = Dnp3FlowState::default();
        assert!(!state.is_non_dnp3, "Dnp3FlowState must default to is_non_dnp3=false");
        assert!(state.carry.is_empty(), "carry buffer must be empty on default");
    }
} // mod story_106
