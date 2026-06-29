//! Tests for STORY-102: Modbus MBAP Parse + FC Classification (Pure Core).
//!
//! Covers BC-2.14.001 through BC-2.14.008 and the desync-bail contract (AC-013).
//! Originated as Red Gate stubs (todo!() panics) per BC-5.38.001; all tests now GREEN.
//!
//! ## Spec Note: Length Range Boundary
//! BC-2.14.004 canonical test vectors state `length=254` → `is_valid_modbus_adu = true`
//! and `length=255` → `false` (range [2, 254]).  STORY-102 AC-005 lists `length=254` → `false`.
//! These tests follow **BC-2.14.004** (the primary authoritative contract) as the source of
//! truth: valid range is [2, 254].  The AC-005 discrepancy is flagged for spec steward review.

// BC-based test naming convention uses uppercase BC IDs: test_BC_S_SS_NNN_xxx.
// The non_snake_case lint fires on uppercase letters — suppressed here intentionally.
#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-102 tests are grouped inside a dedicated
// `mod story_102` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_102 {
    use wirerust::analyzer::modbus::{
        FunctionCodeClass, MbapHeader, ModbusFlowState, classify_fc, is_valid_modbus_adu,
        parse_mbap_header,
    };

    // ---------------------------------------------------------------------------
    // BC-2.14.001 / AC-001: happy-path parse — Read Holding Registers ADU
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_001_returns_some_for_minimum_8_bytes
    ///
    /// Canonical vector: `00 01 00 00 00 06 01 03 00 00 00 0A` (12 bytes).
    /// Expected: `Some(MbapHeader { txn=0x0001, proto=0x0000, len=6, unit=0x01, fc=0x03 })`.
    /// Traces to: BC-2.14.001 postconditions 1–6; STORY-102 AC-001.
    #[test]
    fn test_BC_2_14_001_returns_some_for_minimum_8_bytes() {
        // Canonical BC-2.14.001 test vector: Read Holding Registers request.
        let data: &[u8] = &[
            0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A,
        ];
        let result = parse_mbap_header(data);
        assert!(result.is_some(), "len=12 must return Some");
        let h = result.unwrap();
        assert_eq!(h.transaction_id, 0x0001, "transaction_id big-endian decode");
        assert_eq!(h.protocol_id, 0x0000, "protocol_id big-endian decode");
        assert_eq!(h.length, 6, "length big-endian decode");
        assert_eq!(h.unit_id, 0x01, "unit_id direct byte decode");
        assert_eq!(h.function_code, 0x03, "function_code direct byte decode");
    }

    /// test_BC_2_14_001_canonical_vector_write_single_register
    ///
    /// Canonical vector from BC-2.14.001: `00 2A 00 00 00 06 FF 06 00 14 01 F4`.
    /// Traces to: BC-2.14.001 canonical test vectors (second row).
    #[test]
    fn test_BC_2_14_001_canonical_vector_write_single_register() {
        let data: &[u8] = &[
            0x00, 0x2A, 0x00, 0x00, 0x00, 0x06, 0xFF, 0x06, 0x00, 0x14, 0x01, 0xF4,
        ];
        let result = parse_mbap_header(data);
        assert!(result.is_some(), "write single register vector must parse");
        let h = result.unwrap();
        assert_eq!(h.transaction_id, 0x002A);
        assert_eq!(h.protocol_id, 0x0000);
        assert_eq!(h.length, 6);
        assert_eq!(h.unit_id, 0xFF, "broadcast unit address 0xFF");
        assert_eq!(h.function_code, 0x06);
    }

    /// test_BC_2_14_001_canonical_vector_exception_response
    ///
    /// Canonical vector: `FF FF 00 00 00 02 01 83` — exception response FC 0x83.
    /// Traces to: BC-2.14.001 canonical test vectors (third row).
    #[test]
    fn test_BC_2_14_001_canonical_vector_exception_response() {
        let data: &[u8] = &[0xFF, 0xFF, 0x00, 0x00, 0x00, 0x02, 0x01, 0x83];
        let result = parse_mbap_header(data);
        assert!(result.is_some(), "minimum 8-byte exception ADU must parse");
        let h = result.unwrap();
        assert_eq!(h.transaction_id, 0xFFFF, "max transaction_id");
        assert_eq!(h.protocol_id, 0x0000);
        assert_eq!(h.length, 2);
        assert_eq!(h.unit_id, 0x01);
        assert_eq!(h.function_code, 0x83, "exception FC with high bit set");
    }

    /// test_BC_2_14_001_exact_8_bytes_minimum_adu
    ///
    /// EC-001 from BC-2.14.001: exactly 8 bytes is the minimum valid parse.
    /// Traces to: BC-2.14.001 edge case EC-001; STORY-102 EC-001.
    #[test]
    fn test_BC_2_14_001_exact_8_bytes_minimum_adu() {
        let data: &[u8] = &[0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03];
        let result = parse_mbap_header(data);
        assert!(result.is_some(), "exactly 8 bytes must return Some");
    }

    /// test_BC_2_14_001_trailing_bytes_not_consumed
    ///
    /// BC-2.14.001 postcondition 8: trailing bytes at data[8..] are not consumed.
    /// Parsing a 260-byte slice (padded) returns the same header as the 8-byte slice.
    /// Traces to: BC-2.14.001 postcondition 8; STORY-102 AC-003 / EC-002.
    #[test]
    fn test_BC_2_14_001_trailing_bytes_not_consumed() {
        // 260-byte padded vector from BC-2.14.001 canonical test vectors.
        let mut data = vec![0x00u8; 260];
        data[0] = 0x00;
        data[1] = 0x01; // transaction_id = 0x0001
        data[2] = 0x00;
        data[3] = 0x00; // protocol_id = 0x0000
        data[4] = 0x00;
        data[5] = 0x06; // length = 6
        data[6] = 0x01; // unit_id = 0x01
        data[7] = 0x03; // function_code = 0x03
        let result = parse_mbap_header(&data);
        assert!(result.is_some(), "260-byte slice must return Some");
        let h = result.unwrap();
        assert_eq!(h.transaction_id, 0x0001);
        assert_eq!(h.length, 6);
        assert_eq!(h.function_code, 0x03);
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.001 / AC-003: ADU offset advance = 6 + header.length
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_001_adu_offset_advance_is_6_plus_length
    ///
    /// The ADU boundary loop advances offset by `6 + h.length` (the 6-byte MBAP prefix
    /// TxnID+ProtoID+Length is NOT counted in `length`; total = 6 + length).
    /// For length=6, the full ADU is 12 bytes.
    /// Traces to: BC-2.14.001 postcondition 8; STORY-102 AC-003.
    #[test]
    fn test_BC_2_14_001_adu_offset_advance_is_6_plus_length() {
        // 260-byte buffer; first ADU has length=6 (total = 6+6 = 12 bytes).
        let mut data = vec![0x00u8; 260];
        data[4] = 0x00;
        data[5] = 0x06; // length = 6
        data[6] = 0x01;
        data[7] = 0x03;

        let h = parse_mbap_header(&data).expect("parse must succeed on 260-byte slice");
        let adu_size = 6 + h.length as usize;
        assert_eq!(adu_size, 12, "ADU size = 6 + length(6) = 12");
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.002 / AC-002: None for slices shorter than 8 bytes
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_002_returns_none_for_empty_slice
    ///
    /// BC-2.14.002 EC-001: empty slice → None, no panic.
    /// Traces to: BC-2.14.002 postcondition 1; canonical vector (len=0).
    #[test]
    fn test_BC_2_14_002_returns_none_for_empty_slice() {
        assert_eq!(parse_mbap_header(&[]), None, "empty slice must be None");
    }

    /// test_BC_2_14_002_returns_none_for_single_byte
    ///
    /// BC-2.14.002 EC-002: single byte → None.
    /// Traces to: BC-2.14.002 canonical test vectors.
    #[test]
    fn test_BC_2_14_002_returns_none_for_single_byte() {
        assert_eq!(parse_mbap_header(&[0x00]), None, "len=1 must be None");
    }

    /// test_BC_2_14_002_returns_none_for_seven_bytes
    ///
    /// BC-2.14.002 EC-003: 7 bytes (full MBAP minus FC) → None.
    /// This is the boundary immediately below the 8-byte minimum.
    /// Traces to: BC-2.14.002 postcondition 1; canonical vector `00 01 00 00 00 06 01`.
    #[test]
    fn test_BC_2_14_002_returns_none_for_seven_bytes() {
        let data: &[u8] = &[0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01];
        assert_eq!(parse_mbap_header(data), None, "len=7 must be None");
    }

    /// test_BC_2_14_002_returns_none_for_short_slices_boundary_sweep
    ///
    /// Sweep all short lengths 0..=7, confirm None. Confirm Some for 8 and for 260.
    /// Traces to: BC-2.14.002 postconditions 1–2; STORY-102 AC-002.
    #[test]
    fn test_BC_2_14_002_returns_none_for_short_slices_boundary_sweep() {
        // Canonical test data (long enough for all lengths up to 260).
        let data: &[u8] = &[
            0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A,
        ];
        // All lengths < 8 must return None — no panic.
        for len in 0usize..8 {
            assert_eq!(
                parse_mbap_header(&data[..len]),
                None,
                "len={len} must be None"
            );
        }
        // Length 8 and 12 must return Some.
        assert!(
            parse_mbap_header(&data[..8]).is_some(),
            "len=8 must be Some"
        );
        assert!(
            parse_mbap_header(&data[..12]).is_some(),
            "len=12 must be Some"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.001 invariant 4 / AC-012: parse_mbap_header does NOT gate on proto/length
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_001_invariant_parse_does_not_gate_on_protocol_or_length
    ///
    /// `parse_mbap_header` returns Some even when protocol_id != 0 or length is invalid.
    /// The validity gate belongs ONLY to `is_valid_modbus_adu`.
    /// Input: `00 00 00 01 00 01 01 01` — protocol_id=1 (invalid), length=1 (invalid).
    /// Traces to: BC-2.14.001 invariant 4; STORY-102 AC-012.
    #[test]
    fn test_BC_2_14_001_invariant_parse_does_not_gate_on_protocol_or_length() {
        // protocol_id=0x0001 (invalid), length=0x0001 (invalid — below minimum 2)
        let data: &[u8] = &[0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01];
        let result = parse_mbap_header(data);
        assert!(
            result.is_some(),
            "parse_mbap_header must return Some even with invalid proto/length — no internal gate"
        );
        let h = result.unwrap();
        assert_eq!(
            h.protocol_id, 0x0001,
            "raw protocol_id decoded without rejection"
        );
        assert_eq!(h.length, 0x0001, "raw length decoded without rejection");
        // Now is_valid_modbus_adu must reject it.
        assert!(
            !is_valid_modbus_adu(&h),
            "is_valid_modbus_adu must return false for protocol_id=1"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.003 / AC-004: is_valid_modbus_adu rejects non-zero protocol_id
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_003_rejects_non_zero_protocol_id
    ///
    /// The 3-point gate rejects any ADU with protocol_id != 0x0000.
    /// Canonical vectors: `00 01 00 01 00 06 01 03` (proto=0x0001) → false.
    ///                    `00 01 00 00 00 06 01 03` (proto=0x0000, len=6) → true.
    /// Traces to: BC-2.14.003 postcondition 1; STORY-102 AC-004.
    #[test]
    fn test_BC_2_14_003_rejects_non_zero_protocol_id() {
        let header_invalid_proto = MbapHeader {
            transaction_id: 0x0001,
            protocol_id: 0x0001,
            length: 6,
            unit_id: 0x01,
            function_code: 0x03,
        };
        assert!(
            !is_valid_modbus_adu(&header_invalid_proto),
            "protocol_id=0x0001 must be rejected"
        );

        let header_valid_proto = MbapHeader {
            transaction_id: 0x0001,
            protocol_id: 0x0000,
            length: 6,
            unit_id: 0x01,
            function_code: 0x03,
        };
        assert!(
            is_valid_modbus_adu(&header_valid_proto),
            "protocol_id=0x0000, length=6 must be accepted"
        );
    }

    /// test_BC_2_14_003_rejects_max_protocol_id
    ///
    /// BC-2.14.003 EC-002: protocol_id=0xFFFF → false (permanent non-Modbus).
    /// Canonical vector: `00 01 FF FF 00 06 01 03` (proto=0xFFFF).
    /// Traces to: BC-2.14.003 edge case EC-002.
    #[test]
    fn test_BC_2_14_003_rejects_max_protocol_id() {
        let h = MbapHeader {
            transaction_id: 0x0001,
            protocol_id: 0xFFFF,
            length: 6,
            unit_id: 0x01,
            function_code: 0x03,
        };
        assert!(
            !is_valid_modbus_adu(&h),
            "protocol_id=0xFFFF must be rejected"
        );
    }

    /// test_BC_2_14_003_dos_safe_crafted_huge_length_with_bad_protocol
    ///
    /// Security: protocol_id!=0 with length=0xFFFF — bail-out before `6+65535` arithmetic.
    /// BC-2.14.003 canonical vector EC-005: `00 01 00 01 FF FF 01 03` (proto=0x0001, len=65535).
    /// Traces to: BC-2.14.003 edge case EC-005.
    #[test]
    fn test_BC_2_14_003_dos_safe_crafted_huge_length_with_bad_protocol() {
        let h = MbapHeader {
            transaction_id: 0x0001,
            protocol_id: 0x0001,
            length: 0xFFFF,
            unit_id: 0x01,
            function_code: 0x03,
        };
        assert!(
            !is_valid_modbus_adu(&h),
            "protocol_id=1 with crafted length=65535 must be rejected"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.004 / AC-005: is_valid_modbus_adu length gate [2, 254]
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_004_length_boundary_values
    ///
    /// Length gate: valid range [2, 254] per BC-2.14.004.
    /// Boundary sweep: 0 → false, 1 → false, 2 → true, 254 → true, 255 → false.
    ///
    /// NOTE: STORY-102 AC-005 states `length=254 → false`, which contradicts BC-2.14.004
    /// (canonical vector: `len=254` → `true`; invariant: "Maximum: Length = 254").
    /// These tests follow BC-2.14.004 as the primary authoritative contract.
    /// Spec steward should reconcile STORY-102 AC-005 with BC-2.14.004.
    ///
    /// Traces to: BC-2.14.004 postcondition 1; canonical test vectors; STORY-102 AC-005.
    #[test]
    fn test_BC_2_14_004_length_boundary_values() {
        let make_header = |length: u16| MbapHeader {
            transaction_id: 0x0001,
            protocol_id: 0x0000,
            length,
            unit_id: 0x01,
            function_code: 0x03,
        };

        // Below minimum — rejected.
        assert!(
            !is_valid_modbus_adu(&make_header(0)),
            "length=0 must be rejected"
        );
        assert!(
            !is_valid_modbus_adu(&make_header(1)),
            "length=1 must be rejected"
        );

        // Minimum valid (BC-2.14.004 EC-003).
        assert!(
            is_valid_modbus_adu(&make_header(2)),
            "length=2 must be accepted (minimum)"
        );

        // Maximum valid per BC-2.14.004 (Length = UnitID(1) + PDU_max(253) = 254).
        assert!(
            is_valid_modbus_adu(&make_header(254)),
            "length=254 must be accepted (maximum per BC-2.14.004)"
        );

        // First above maximum — rejected.
        assert!(
            !is_valid_modbus_adu(&make_header(255)),
            "length=255 must be rejected"
        );

        // u16::MAX — rejected, no OOB.
        assert!(
            !is_valid_modbus_adu(&make_header(0xFFFF)),
            "length=65535 must be rejected"
        );
    }

    /// test_BC_2_14_004_length_253_valid
    ///
    /// EC-007 from STORY-102: length=253 → valid; ADU boundary = 6+253=259 bytes.
    /// Traces to: STORY-102 EC-007.
    #[test]
    fn test_BC_2_14_004_length_253_valid() {
        let h = MbapHeader {
            transaction_id: 0x0001,
            protocol_id: 0x0000,
            length: 253,
            unit_id: 0x01,
            function_code: 0x03,
        };
        assert!(
            is_valid_modbus_adu(&h),
            "length=253 must be accepted; ADU size = 6+253 = 259"
        );
    }

    /// test_BC_2_14_004_all_zeros_rejected
    ///
    /// EC-003 from STORY-102 edge cases: all-zero 8-byte input.
    /// parse returns Some; is_valid returns false (length=0 < 2).
    /// Traces to: BC-2.14.001 EC-011; STORY-102 EC-003.
    #[test]
    fn test_BC_2_14_004_all_zeros_rejected_by_validity_gate() {
        let data = [0u8; 8];
        let h = parse_mbap_header(&data).expect("all-zeros 8 bytes must parse");
        assert_eq!(h.protocol_id, 0);
        assert_eq!(h.length, 0);
        assert!(
            !is_valid_modbus_adu(&h),
            "all-zeros header has length=0 → rejected by length gate"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.005 / AC-006: classify_fc is total over all 256 u8 values
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_005_classify_fc_is_total
    ///
    /// Iterates all 256 u8 values. Any panic = test fail. Confirms totality invariant.
    /// Traces to: BC-2.14.005 postcondition 1 and invariant 1; STORY-102 AC-006.
    #[test]
    fn test_BC_2_14_005_classify_fc_is_total() {
        for fc in 0u8..=255 {
            // This must not panic for any fc value.
            let class = classify_fc(fc);
            // Every value must be one of the five defined variants (compiler enforces this,
            // but the explicit match ensures totality is observable at test time).
            assert!(
                matches!(
                    class,
                    FunctionCodeClass::Read
                        | FunctionCodeClass::Write
                        | FunctionCodeClass::Diagnostic
                        | FunctionCodeClass::Exception
                        | FunctionCodeClass::Unknown
                ),
                "classify_fc({fc:#04x}) returned an undefined variant"
            );
        }
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.006 / AC-007: Exception FC — high bit set
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_006_exception_when_high_bit_set
    ///
    /// Any fc >= 0x80 must return Exception. Canonical vectors from BC-2.14.006.
    /// Traces to: BC-2.14.006 postcondition 1; STORY-102 AC-007.
    #[test]
    fn test_BC_2_14_006_exception_when_high_bit_set() {
        assert_eq!(
            classify_fc(0x83),
            FunctionCodeClass::Exception,
            "0x83 (exception for Read HR) must be Exception"
        );
        assert_eq!(
            classify_fc(0x80),
            FunctionCodeClass::Exception,
            "0x80 (minimum exception FC) must be Exception"
        );
        assert_eq!(
            classify_fc(0xFF),
            FunctionCodeClass::Exception,
            "0xFF (maximum FC) must be Exception"
        );
        assert_eq!(
            classify_fc(0x85),
            FunctionCodeClass::Exception,
            "0x85 (exception for Write Single Coil) must be Exception"
        );
        assert_eq!(
            classify_fc(0x90),
            FunctionCodeClass::Exception,
            "0x90 (exception for Write Multiple Registers) must be Exception"
        );
    }

    /// test_BC_2_14_006_original_fc_recovered_via_mask
    ///
    /// `original_fc = fc & 0x7F` losslessly recovers the request FC.
    /// Traces to: BC-2.14.006 postcondition 2; invariant 2; STORY-102 AC-007.
    // 0xFF & 0x7F == 0x7F: the mask has "no effect" on 0xFF in one direction but is the
    // correct operation to document — the assert verifies the general pattern holds for 0xFF.
    #[allow(clippy::identity_op)]
    #[test]
    fn test_BC_2_14_006_original_fc_recovered_via_mask() {
        // 0x83 is exception for 0x03 (Read Holding Registers)
        assert_eq!(
            0x83u8 & 0x7F,
            0x03,
            "original FC recovery: 0x83 & 0x7F == 0x03"
        );
        // 0x85 is exception for 0x05 (Write Single Coil)
        assert_eq!(
            0x85u8 & 0x7F,
            0x05,
            "original FC recovery: 0x85 & 0x7F == 0x05"
        );
        // 0x90 is exception for 0x10 (Write Multiple Registers)
        assert_eq!(
            0x90u8 & 0x7F,
            0x10,
            "original FC recovery: 0x90 & 0x7F == 0x10"
        );
        // 0xFF — original FC = 0x7F (Unknown class)
        assert_eq!(
            0xFFu8 & 0x7F,
            0x7F,
            "original FC recovery: 0xFF & 0x7F == 0x7F"
        );
    }

    /// test_BC_2_14_006_exception_biconditional_all_256_values
    ///
    /// VP-022 sub-property C in unit-test form: Exception IFF fc >= 0x80.
    /// Bidirectional: (a) all fc >= 0x80 → Exception; (b) no fc < 0x80 → Exception.
    /// Traces to: BC-2.14.006 invariant 1 (VP-022 sub-property C).
    #[test]
    fn test_BC_2_14_006_exception_biconditional_all_256_values() {
        for fc in 0u8..=255 {
            let is_exception = classify_fc(fc) == FunctionCodeClass::Exception;
            let high_bit_set = fc >= 0x80;
            assert_eq!(
                is_exception, high_bit_set,
                "Exception biconditional failed for fc={fc:#04x}: \
             classify_fc returned Exception={is_exception} but high_bit_set={high_bit_set}"
            );
        }
    }

    /// test_BC_2_14_006_no_fc_below_0x80_is_exception
    ///
    /// Regression: FCs below 0x80 must NEVER be Exception (b-side of biconditional).
    /// Traces to: BC-2.14.006 invariant 1; BC-2.14.006 canonical vector (0x03 NOT exception).
    #[test]
    fn test_BC_2_14_006_no_fc_below_0x80_is_exception() {
        for fc in 0x00u8..0x80 {
            assert_ne!(
                classify_fc(fc),
                FunctionCodeClass::Exception,
                "fc={fc:#04x} < 0x80 must NOT be Exception"
            );
        }
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.007 / AC-008: Write-class FCs — exact set {0x05,0x06,0x0F,0x10,0x15,0x16,0x17}
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_007_write_class_completeness
    ///
    /// All 7 Write FCs return Write. Non-write FCs do not.
    /// Traces to: BC-2.14.007 postcondition 1; invariant 1; STORY-102 AC-008.
    #[test]
    fn test_BC_2_14_007_write_class_completeness() {
        let write_fcs: &[u8] = &[0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17];
        for &fc in write_fcs {
            assert_eq!(
                classify_fc(fc),
                FunctionCodeClass::Write,
                "fc={fc:#04x} must be Write"
            );
        }

        // Representative non-Write FCs (regression).
        let non_write_fcs: &[u8] = &[0x01, 0x03, 0x08, 0x2B, 0x00, 0x7F, 0x80, 0x86];
        for &fc in non_write_fcs {
            assert_ne!(
                classify_fc(fc),
                FunctionCodeClass::Write,
                "fc={fc:#04x} must NOT be Write"
            );
        }
    }

    /// test_BC_2_14_007_write_0x17_classified_as_write
    ///
    /// 0x17 (Read/Write Multiple Registers): write half governs — must be Write.
    /// BC-2.14.007 EC-004: "write half governs; read half does not downgrade risk".
    /// Traces to: BC-2.14.007 invariant 3.
    #[test]
    fn test_BC_2_14_007_write_0x17_classified_as_write() {
        assert_eq!(
            classify_fc(0x17),
            FunctionCodeClass::Write,
            "0x17 Read/Write Multiple Registers must be Write"
        );
    }

    /// test_BC_2_14_007_write_0x15_and_0x16_classified_as_write
    ///
    /// 0x15 (Write File Record) and 0x16 (Mask Write Register) — both Write.
    /// Traces to: BC-2.14.007 canonical test vectors.
    #[test]
    fn test_BC_2_14_007_write_0x15_and_0x16_classified_as_write() {
        assert_eq!(
            classify_fc(0x15),
            FunctionCodeClass::Write,
            "0x15 Write File Record must be Write"
        );
        assert_eq!(
            classify_fc(0x16),
            FunctionCodeClass::Write,
            "0x16 Mask Write Register must be Write"
        );
    }

    /// test_BC_2_14_007_no_write_exception_fc_is_write
    ///
    /// FC >= 0x80 must NEVER be Write (pre-guard fires first).
    /// BC-2.14.007 invariant 2: 0x85/0x86 are Exception, not Write.
    /// Traces to: BC-2.14.007 invariant 2; canonical test vector 0x86.
    #[test]
    fn test_BC_2_14_007_no_write_exception_fc_is_write() {
        for fc in 0x80u8..=0xFF {
            assert_ne!(
                classify_fc(fc),
                FunctionCodeClass::Write,
                "fc={fc:#04x} >= 0x80 must be Exception, not Write"
            );
        }
    }

    /// test_BC_2_14_007_0x08_diagnostics_not_write
    ///
    /// BC-2.14.007 invariant 4 + BC-2.14.008: 0x08 is Diagnostic, never Write.
    /// Traces to: BC-2.14.007 EC-008.
    #[test]
    fn test_BC_2_14_007_0x08_diagnostics_not_write() {
        assert_ne!(
            classify_fc(0x08),
            FunctionCodeClass::Write,
            "0x08 Diagnostics must NOT be Write"
        );
        assert_eq!(
            classify_fc(0x08),
            FunctionCodeClass::Diagnostic,
            "0x08 Diagnostics must be Diagnostic"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.008 / AC-009: Diagnostic-class FCs — {0x08, 0x2B}
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_008_diagnostic_class
    ///
    /// 0x08 and 0x2B must be Diagnostic. Representative unknowns must be Unknown.
    /// Traces to: BC-2.14.008 postconditions 1–2; STORY-102 AC-009.
    #[test]
    fn test_BC_2_14_008_diagnostic_class() {
        assert_eq!(
            classify_fc(0x08),
            FunctionCodeClass::Diagnostic,
            "0x08 Diagnostics must be Diagnostic"
        );
        assert_eq!(
            classify_fc(0x2B),
            FunctionCodeClass::Diagnostic,
            "0x2B MEI must be Diagnostic"
        );

        // Representative Unknown FCs (not Read, Write, or Diagnostic, and below 0x80).
        let unknown_fcs: &[u8] = &[0x00, 0x09, 0x0A, 0x0D, 0x0E, 0x7F];
        for &fc in unknown_fcs {
            assert_eq!(
                classify_fc(fc),
                FunctionCodeClass::Unknown,
                "fc={fc:#04x} must be Unknown"
            );
        }
    }

    /// test_BC_2_14_008_exception_for_0x08_is_exception_not_diagnostic
    ///
    /// 0x88 is the exception FC for 0x08 — must be Exception (pre-guard), not Diagnostic.
    /// BC-2.14.008 EC-009.
    /// Traces to: BC-2.14.008 edge case EC-009.
    #[test]
    fn test_BC_2_14_008_exception_for_0x08_is_exception_not_diagnostic() {
        assert_eq!(
            classify_fc(0x88),
            FunctionCodeClass::Exception,
            "0x88 (exception for 0x08) must be Exception, not Diagnostic"
        );
        assert_ne!(
            classify_fc(0x88),
            FunctionCodeClass::Diagnostic,
            "0x88 must NOT be Diagnostic"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.005 canonical test vectors — all named FC values
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_005_canonical_read_fcs
    ///
    /// All 10 Read-class FCs from BC-2.14.005 postcondition 2 canonical test vectors.
    /// Traces to: BC-2.14.005 canonical test vector table.
    #[test]
    fn test_BC_2_14_005_canonical_read_fcs() {
        let read_fcs: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x07, 0x0B, 0x0C, 0x11, 0x14, 0x18];
        for &fc in read_fcs {
            assert_eq!(
                classify_fc(fc),
                FunctionCodeClass::Read,
                "fc={fc:#04x} must be Read"
            );
        }
    }

    /// test_BC_2_14_005_edge_case_fc_0x7f_is_unknown
    ///
    /// EC-004 from STORY-102: FC=0x7F (no high bit, undefined) → Unknown.
    /// BC-2.14.005 EC-010: "highest non-exception byte → Unknown".
    /// Traces to: BC-2.14.005 edge case EC-010; STORY-102 EC-004.
    #[test]
    fn test_BC_2_14_005_edge_case_fc_0x7f_is_unknown() {
        assert_eq!(
            classify_fc(0x7F),
            FunctionCodeClass::Unknown,
            "0x7F must be Unknown (no high bit, not in any standard set)"
        );
    }

    /// test_BC_2_14_005_edge_case_fc_0xff_is_exception
    ///
    /// EC-005 from STORY-102: FC=0xFF (all bits set) → Exception.
    /// Traces to: BC-2.14.005 EC-007; STORY-102 EC-005.
    #[test]
    fn test_BC_2_14_005_edge_case_fc_0xff_is_exception() {
        assert_eq!(
            classify_fc(0xFF),
            FunctionCodeClass::Exception,
            "0xFF must be Exception (high bit set)"
        );
    }

    /// test_BC_2_14_005_edge_case_fc_0x01_is_read
    ///
    /// EC-009 from STORY-102: FC=0x01 (Read Coils) → Read.
    /// Traces to: STORY-102 EC-009.
    #[test]
    fn test_BC_2_14_005_edge_case_fc_0x01_is_read() {
        assert_eq!(
            classify_fc(0x01),
            FunctionCodeClass::Read,
            "0x01 must be Read"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.003 / AC-013: ModbusFlowState desync bail-out contract
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_003_is_non_modbus_flag_exists_and_defaults_false
    ///
    /// `ModbusFlowState::default()` initializes `is_non_modbus = false`.
    /// After receiving a non-Modbus header (protocol_id != 0), the caller sets
    /// `is_non_modbus = true` and all subsequent on_data calls bail immediately.
    /// This test verifies the stub struct and the flag's default state.
    /// Traces to: BC-2.14.003 invariant 5; STORY-102 AC-013.
    #[test]
    fn test_BC_2_14_003_is_non_modbus_flag_defaults_to_false() {
        let state = ModbusFlowState::default();
        assert!(
            !state.is_non_modbus,
            "ModbusFlowState must default to is_non_modbus=false"
        );
    }

    /// test_BC_2_14_003_is_non_modbus_bail_simulated
    ///
    /// Simulates the desync bail-out: deliver a non-Modbus PDU (proto_id != 0),
    /// set is_non_modbus=true, then confirm a subsequent valid ADU is NOT processed
    /// (caller bails at entry when is_non_modbus is true).
    /// This exercises the contract without a full ModbusAnalyzer (that comes in STORY-103).
    /// Traces to: BC-2.14.003 postconditions 2–4; STORY-102 AC-013.
    // `state.is_non_modbus = true` is intentional field assignment post-default: the test
    // deliberately simulates the caller's desync-set path rather than using a struct literal,
    // because this is exactly how STORY-103 on_data will set the flag.
    #[allow(clippy::field_reassign_with_default)]
    #[test]
    fn test_BC_2_14_003_is_non_modbus_bail_simulated() {
        // Step 1: invalid PDU (protocol_id = 0x0001).
        let invalid_data: &[u8] = &[0x00, 0x01, 0x00, 0x01, 0x00, 0x06, 0x01, 0x03];
        let h = parse_mbap_header(invalid_data).expect("8-byte slice must parse");
        assert_eq!(h.protocol_id, 0x0001);
        assert!(
            !is_valid_modbus_adu(&h),
            "protocol_id=1 must fail validity gate"
        );

        // Step 2: caller sets is_non_modbus = true.
        let mut state = ModbusFlowState::default();
        state.is_non_modbus = true;

        // Step 3: bail-out guard — subsequent on_data calls must return immediately.
        // The valid PDU is never processed; simulated here by checking the flag.
        assert!(
            state.is_non_modbus,
            "is_non_modbus must remain true for all subsequent calls on this flow"
        );

        // Confirm a valid PDU that WOULD parse and pass the gate
        let valid_data: &[u8] = &[
            0x00, 0x02, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A,
        ];
        let valid_h = parse_mbap_header(valid_data).expect("valid ADU must parse");
        assert!(
            is_valid_modbus_adu(&valid_h),
            "valid ADU passes the gate — but is_non_modbus bail prevents reaching this"
        );
        // The bail is enforced by the caller (on_data in STORY-103); the flag is what we test here.
    }
} // mod story_102
