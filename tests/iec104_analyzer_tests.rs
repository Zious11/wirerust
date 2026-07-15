//! Tests for STORY-167: IEC-104 APCI Core Parser (pure-core free functions).
//!
//! Covers BC-2.19.001 through BC-2.19.006 and all edge cases from the BCs.
//! All tests are GREEN against the delivered parse_apci_header / is_valid_iec104_frame implementation.
//!
//! ## Contract coverage
//! - BC-2.19.001: `parse_apci_header` returns None for input shorter than 6 bytes.
//! - BC-2.19.002: `parse_apci_header` returns None for start byte != 0x68.
//! - BC-2.19.003: `parse_apci_header` returns None for LEN < 4.
//! - BC-2.19.004: `parse_apci_header` returns None for LEN > 253.
//! - BC-2.19.005: `parse_apci_header` returns Some(ApciHeader) for valid input; CF1-CF4 verbatim.
//! - BC-2.19.006: `is_valid_iec104_frame` post-classification validity gate.
//!
//! ## Test naming convention
//! Tests follow `test_BC_S_SS_NNN_xxx()` for BC-traceable tests.
//! The non_snake_case lint fires on uppercase BC IDs — suppressed intentionally.
//!
//! ## Provenance
//! Authored Red-first as TDD stubs (STORY-167 strict TDD mode); now GREEN against
//! the delivered parse_apci_header / is_valid_iec104_frame implementation.
//!
//! Canonical test vectors from BCs are used verbatim (DF-CANONICAL-FRAME-HOLDOUT-001).

#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-167 tests are grouped inside a dedicated
// `mod story_167` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_167 {
    use wirerust::analyzer::iec104::{ApciHeader, is_valid_iec104_frame, parse_apci_header};

    // =========================================================================
    // BC-2.19.001: parse_apci_header returns None for input shorter than 6 bytes
    // AC-167-001
    // =========================================================================

    /// BC-2.19.001 canonical vector: empty slice returns None.
    ///
    /// Canonical vector from BC-2.19.001: `[]` (0 bytes) → None.
    /// Precondition: data.len() == 0 (< 6).
    /// Postcondition 1: returns None. Postcondition 2: no bytes accessed. No panic.
    ///
    /// Traces: BC-2.19.001 postconditions 1-3; AC-167-001; EC-001.
    #[test]
    fn test_BC_2_19_001_returns_none_for_empty_slice() {
        let result = parse_apci_header(&[]);
        assert!(
            result.is_none(),
            "empty slice must return None (BC-2.19.001 postcondition 1)"
        );
    }

    /// BC-2.19.001: one-byte slice returns None.
    ///
    /// Canonical vector from BC-2.19.001 EC-002: `data.len() == 1` → None.
    /// Precondition: data.len() == 1 (< 6).
    ///
    /// Traces: BC-2.19.001 postconditions 1-3; AC-167-001; EC-002.
    #[test]
    fn test_BC_2_19_001_returns_none_for_one_byte() {
        let result = parse_apci_header(&[0x68]);
        assert!(
            result.is_none(),
            "1-byte slice must return None even if byte is 0x68 (BC-2.19.001 postcondition 1)"
        );
    }

    /// BC-2.19.001: five-byte slice (one byte short) returns None.
    ///
    /// Canonical vector from BC-2.19.001: `[0x68, 0x04, 0x07, 0x00, 0x00]` (5 bytes) → None.
    /// This slice looks like the start of a valid frame but is one byte short of the 6-byte
    /// APCI header minimum. The length guard fires before any field is inspected.
    ///
    /// Traces: BC-2.19.001 postconditions 1-3; AC-167-001; EC-003 (5 bytes, one short); EC-005.
    #[test]
    fn test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector() {
        let data: &[u8] = &[0x68, 0x04, 0x07, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "5-byte canonical vector must return None (BC-2.19.001 EC-003/EC-005)"
        );
    }

    /// BC-2.19.001: two-byte slice returns None.
    ///
    /// Exercises len=2 path — shorter than 5-byte canonical vector above.
    ///
    /// Traces: BC-2.19.001 postconditions 1-2; AC-167-001.
    #[test]
    fn test_BC_2_19_001_returns_none_for_two_bytes() {
        let result = parse_apci_header(&[0x68, 0x04]);
        assert!(
            result.is_none(),
            "2-byte slice must return None (BC-2.19.001 postcondition 1)"
        );
    }

    /// BC-2.19.001 invariant: no panic on any truncated or malformed slice.
    ///
    /// Exercises a sample of lengths 0-5 including all-zeros content to verify purity
    /// invariant 2 (no panic). If any call panics, the test infrastructure reports it.
    ///
    /// Traces: BC-2.19.001 invariants 1-3; AC-167-001; VP-047 (no-panic).
    #[test]
    fn test_BC_2_19_001_invariant_no_panic_on_truncated_inputs() {
        let inputs: &[&[u8]] = &[
            &[],
            &[0x00],
            &[0xFF],
            &[0x68],
            &[0x68, 0x04],
            &[0x68, 0x04, 0x07],
            &[0x68, 0x04, 0x07, 0x00],
            &[0x00, 0x00, 0x00, 0x00, 0x00],
        ];
        for &data in inputs {
            // Each call must either return None or return Some — neither panics.
            let result = parse_apci_header(data);
            assert!(
                result.is_none(),
                "input of len {} must return None for len < 6 (BC-2.19.001)",
                data.len()
            );
        }
    }

    // =========================================================================
    // BC-2.19.002: parse_apci_header returns None for start byte != 0x68
    // AC-167-002
    // =========================================================================

    /// BC-2.19.002 canonical vector: start byte 0x00 returns None.
    ///
    /// Canonical vector from BC-2.19.002: `[0x00, 0x04, 0x07, 0x00, 0x00, 0x00]` → None + T0814.
    /// data.len() == 6 (length guard passes); data[0] != 0x68 (start-byte guard fires).
    ///
    /// Traces: BC-2.19.002 postcondition 1; AC-167-002; EC-001.
    #[test]
    fn test_BC_2_19_002_returns_none_for_start_byte_0x00_canonical_vector() {
        let data: &[u8] = &[0x00, 0x04, 0x07, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "start byte 0x00 must return None (BC-2.19.002 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.19.002 canonical vector: start byte 0xFF returns None.
    ///
    /// Canonical vector from BC-2.19.002: `[0xFF, 0x04, 0x07, 0x00, 0x00, 0x00]` → None + T0814.
    ///
    /// Traces: BC-2.19.002 postcondition 1; AC-167-002.
    #[test]
    fn test_BC_2_19_002_returns_none_for_start_byte_0xFF_canonical_vector() {
        let data: &[u8] = &[0xFF, 0x04, 0x07, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "start byte 0xFF must return None (BC-2.19.002 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.19.002 edge case: off-by-one from valid start byte (0x69) returns None.
    ///
    /// Exercises EC-003 from STORY-167: `data[0] == 0x69` (off-by-one from 0x68).
    ///
    /// Traces: BC-2.19.002 postcondition 1; AC-167-002; STORY-167 EC-003.
    #[test]
    fn test_BC_2_19_002_returns_none_for_start_byte_0x69_off_by_one() {
        let data: &[u8] = &[0x69, 0x04, 0x07, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "start byte 0x69 (off-by-one from 0x68) must return None (BC-2.19.002, STORY-167 EC-003)"
        );
    }

    // =========================================================================
    // BC-2.19.003: parse_apci_header returns None for LEN < 4
    // AC-167-003
    // =========================================================================

    /// BC-2.19.003 canonical vector: LEN=0 returns None.
    ///
    /// Canonical vector from BC-2.19.003: `[0x68, 0x00, ...]` (LEN=0) → None + T0814.
    /// Preconditions: data.len() >= 6, data[0] == 0x68, data[1] == 0 (< 4).
    ///
    /// Traces: BC-2.19.003 postcondition 1; AC-167-003; EC-001.
    #[test]
    fn test_BC_2_19_003_returns_none_for_len_zero_canonical_vector() {
        let data: &[u8] = &[0x68, 0x00, 0x07, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "LEN=0 must return None (BC-2.19.003 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.19.003 canonical vector: LEN=3 (off-by-one boundary) returns None.
    ///
    /// Canonical vector from BC-2.19.003: `[0x68, 0x03, ...]` (LEN=3) → None + T0814.
    /// LEN=3 is one below the minimum valid value of 4.
    ///
    /// Traces: BC-2.19.003 postcondition 1; AC-167-003; EC-002 (LEN=3); STORY-167 EC-004.
    #[test]
    fn test_BC_2_19_003_returns_none_for_len_3_off_by_one_canonical_vector() {
        let data: &[u8] = &[0x68, 0x03, 0x07, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "LEN=3 (below minimum) must return None (BC-2.19.003 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.19.003 also covers LEN=1 and LEN=2 to exhaust all sub-minimum values.
    ///
    /// Traces: BC-2.19.003 postcondition 1; AC-167-003.
    #[test]
    fn test_BC_2_19_003_returns_none_for_len_1_and_len_2() {
        for len_byte in [0x01u8, 0x02] {
            let data: [u8; 6] = [0x68, len_byte, 0x07, 0x00, 0x00, 0x00];
            let result = parse_apci_header(&data);
            assert!(
                result.is_none(),
                "LEN={len_byte} must return None (BC-2.19.003 postcondition 1)"
            );
        }
    }

    // =========================================================================
    // BC-2.19.004: parse_apci_header returns None for LEN > 253
    // AC-167-004
    // =========================================================================

    /// BC-2.19.004 canonical vector: LEN=254 (0xFE) returns None.
    ///
    /// Canonical vector from BC-2.19.004: `[0x68, 0xFE, ...]` (LEN=254) → None + T0814.
    /// LEN=254 exceeds the protocol maximum of 253.
    ///
    /// Traces: BC-2.19.004 postcondition 1; AC-167-004; EC-001.
    #[test]
    fn test_BC_2_19_004_returns_none_for_len_254_canonical_vector() {
        let data: &[u8] = &[0x68, 0xFE, 0x01, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "LEN=254 (0xFE) must return None (BC-2.19.004 canonical vector, postcondition 1)"
        );
    }

    /// BC-2.19.004 canonical vector: LEN=255 (0xFF) returns None.
    ///
    /// Canonical vector from BC-2.19.004: `[0x68, 0xFF, ...]` (LEN=255) → None + T0814.
    ///
    /// Traces: BC-2.19.004 postcondition 1; AC-167-004; EC-002.
    #[test]
    fn test_BC_2_19_004_returns_none_for_len_255_canonical_vector() {
        let data: &[u8] = &[0x68, 0xFF, 0x01, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        assert!(
            result.is_none(),
            "LEN=255 (0xFF) must return None (BC-2.19.004 canonical vector, postcondition 1)"
        );
    }

    // =========================================================================
    // BC-2.19.005: parse_apci_header returns Some(ApciHeader) for valid input
    // AC-167-005
    // =========================================================================

    /// BC-2.19.005 canonical vector: U-frame STARTDT-act (LEN=4, minimum valid).
    ///
    /// Canonical vector from BC-2.19.005:
    ///   `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]`
    ///   → `Some(ApciHeader { start: 0x68, len: 4, cf1: 0x07, cf2: 0, cf3: 0, cf4: 0 })`
    ///
    /// All six fields are asserted individually (non-vacuous). CF1=0x07 encodes STARTDT-act
    /// (bits 1:0 = 0b11 = U-format, bits 7:2 = 0b000001 = STARTDT-act).
    ///
    /// Also exercises BC-2.19.001 (EC-004: exactly 6 bytes proceeds past length guard) and
    /// BC-2.19.003 (EC-003: LEN=4 is minimum valid, proceeds to accept path).
    ///
    /// Traces: BC-2.19.005 postconditions 1-6; AC-167-005; BC-2.19.001 EC-004;
    ///         BC-2.19.003 EC-003; STORY-167 EC-005.
    #[test]
    fn test_BC_2_19_005_u_frame_startdt_act_all_fields_correct_canonical_vector() {
        let data: &[u8] = &[0x68, 0x04, 0x07, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        let h = result
            .expect("U-frame canonical vector must return Some (BC-2.19.005 postcondition 1)");
        // BC-2.19.005 postcondition 1: start byte always 0x68
        assert_eq!(h.start, 0x68, "start must be 0x68");
        // BC-2.19.005 postcondition 1: len == data[1] == 4
        assert_eq!(h.len, 4, "len must be 4 (data[1])");
        // BC-2.19.005 postcondition 1: cf1 == data[2] == 0x07
        assert_eq!(h.cf1, 0x07, "cf1 must be 0x07 (data[2])");
        // BC-2.19.005 postcondition 1: cf2 == data[3] == 0x00
        assert_eq!(h.cf2, 0x00, "cf2 must be 0x00 (data[3])");
        // BC-2.19.005 postcondition 1: cf3 == data[4] == 0x00
        assert_eq!(h.cf3, 0x00, "cf3 must be 0x00 (data[4])");
        // BC-2.19.005 postcondition 1: cf4 == data[5] == 0x00
        assert_eq!(h.cf4, 0x00, "cf4 must be 0x00 (data[5])");
        // BC-2.19.005 postcondition 2: total frame size = len + 2 = 6
        assert_eq!(
            h.len as usize + 2,
            6,
            "total frame bytes must be 6 for LEN=4"
        );
    }

    /// BC-2.19.005 canonical vector: S-frame (CF1=0x01, LEN=4).
    ///
    /// Canonical vector from BC-2.19.005:
    ///   `[0x68, 0x04, 0x01, 0x00, 0x00, 0x00]`
    ///   → `Some(ApciHeader { start: 0x68, len: 4, cf1: 0x01, cf2: 0, cf3: 0, cf4: 0 })`
    ///
    /// CF1=0x01 encodes S-format (bits 1:0 = 0b01).
    ///
    /// Traces: BC-2.19.005 postconditions 1-6; AC-167-005.
    #[test]
    fn test_BC_2_19_005_s_frame_all_fields_correct_canonical_vector() {
        let data: &[u8] = &[0x68, 0x04, 0x01, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        let h = result
            .expect("S-frame canonical vector must return Some (BC-2.19.005 postcondition 1)");
        assert_eq!(h.start, 0x68, "start must be 0x68");
        assert_eq!(h.len, 4, "len must be 4");
        assert_eq!(h.cf1, 0x01, "cf1 must be 0x01 (S-frame)");
        assert_eq!(h.cf2, 0x00, "cf2 must be 0x00");
        assert_eq!(h.cf3, 0x00, "cf3 must be 0x00");
        assert_eq!(h.cf4, 0x00, "cf4 must be 0x00");
    }

    /// BC-2.19.005 canonical vector: I-frame (CF1=0x00, LEN=14).
    ///
    /// Canonical vector from BC-2.19.005:
    ///   `[0x68, 0x0E, 0x00, 0x00, 0x00, 0x00, ...]` (LEN=14)
    ///   → `Some(ApciHeader { start: 0x68, len: 14, cf1: 0x00, ... })`
    ///
    /// CF1=0x00 encodes I-format (bit 0 = 0). The parse function reads only the first
    /// 6 bytes; trailing bytes are not accessed (postcondition 6).
    ///
    /// Traces: BC-2.19.005 postconditions 1-6; AC-167-005.
    #[test]
    fn test_BC_2_19_005_i_frame_all_fields_correct_canonical_vector() {
        // Extend beyond 6 bytes to verify bytes beyond index 5 are not accessed
        let data: &[u8] = &[0x68, 0x0E, 0x00, 0x00, 0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
        let result = parse_apci_header(data);
        let h = result
            .expect("I-frame canonical vector must return Some (BC-2.19.005 postcondition 1)");
        assert_eq!(h.start, 0x68, "start must be 0x68");
        assert_eq!(h.len, 14, "len must be 14 (0x0E, data[1])");
        assert_eq!(h.cf1, 0x00, "cf1 must be 0x00 (I-frame)");
        assert_eq!(h.cf2, 0x00, "cf2 must be 0x00");
        assert_eq!(h.cf3, 0x00, "cf3 must be 0x00");
        assert_eq!(h.cf4, 0x00, "cf4 must be 0x00");
        // BC-2.19.005 postcondition 2: total frame bytes = 14 + 2 = 16
        assert_eq!(
            h.len as usize + 2,
            16,
            "total frame bytes must be 16 for LEN=14"
        );
    }

    /// BC-2.19.005 canonical vector: LEN=253 (maximum valid) returns Some.
    ///
    /// Canonical vector from BC-2.19.004:
    ///   `[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00]` (LEN=253)
    ///   → `Some(ApciHeader { start: 0x68, len: 253, cf1: 0x01, ... })`
    ///
    /// Also exercises BC-2.19.004 EC-003 (LEN=253 is maximum valid, proceeds).
    ///
    /// Traces: BC-2.19.005 postconditions 1-2; BC-2.19.004 EC-003; AC-167-005; STORY-167 EC-006.
    #[test]
    fn test_BC_2_19_005_returns_some_for_len_253_maximum_canonical_vector() {
        let data: &[u8] = &[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00];
        let result = parse_apci_header(data);
        let h = result.expect("LEN=253 canonical vector must return Some (BC-2.19.004 EC-003)");
        assert_eq!(h.start, 0x68, "start must be 0x68");
        assert_eq!(h.len, 253, "len must be 253 (maximum valid LEN)");
        assert_eq!(h.cf1, 0x01, "cf1 must be 0x01");
        // BC-2.19.005 invariant 1: h.len + 2 in [6, 255] — no overflow
        let total = h.len as usize + 2;
        assert!(
            (6..=255).contains(&total),
            "h.len ({}) + 2 = {total} must be in [6, 255] (BC-2.19.005 invariant 1)",
            h.len
        );
        assert_eq!(total, 255, "LEN=253: total must be 255");
    }

    /// BC-2.19.005 invariant 1: for any Some(h), h.len+2 is in [6, 255] — no overflow.
    ///
    /// Tests the complete valid LEN boundary sequence: 4 and 253 (canonical boundaries).
    /// Verifies VP-044 Kani property B arithmetically for boundary inputs.
    ///
    /// Traces: BC-2.19.005 invariants 1-2; AC-167-005; VP-044 property B.
    #[test]
    fn test_BC_2_19_005_invariant_len_plus_two_in_range_for_boundaries() {
        let boundary_cases: &[(&[u8], u8)] = &[
            (&[0x68, 0x04, 0xAA, 0xBB, 0xCC, 0xDD], 4),
            (&[0x68, 0xFD, 0xAA, 0xBB, 0xCC, 0xDD], 253),
        ];
        for &(data, expected_len) in boundary_cases {
            let h = parse_apci_header(data)
                .unwrap_or_else(|| panic!("LEN={expected_len} must return Some"));
            assert_eq!(h.len, expected_len, "h.len must equal data[1]");
            let total = h.len as usize + 2;
            assert!(
                (6..=255).contains(&total),
                "h.len ({}) + 2 = {total} must be in [6, 255] (BC-2.19.005 invariant 1)",
                h.len
            );
        }
    }

    /// BC-2.19.005 postcondition 6: bytes beyond index 5 are not accessed.
    ///
    /// Provides a 6-byte slice with CF1-CF4 values 0xAA/0xBB/0xCC/0xDD, then verifies
    /// the exact CF values are returned verbatim without being contaminated by anything
    /// beyond index 5 in a longer slice.
    ///
    /// Traces: BC-2.19.005 postcondition 6; AC-167-005.
    #[test]
    fn test_BC_2_19_005_cf_fields_verbatim_from_data_indices_2_through_5() {
        let data: &[u8] = &[0x68, 0x10, 0xAA, 0xBB, 0xCC, 0xDD, 0xFF, 0xFF];
        let h = parse_apci_header(data).expect("valid 8-byte input (LEN=16) must return Some");
        // CF1-CF4 must be verbatim copies of data[2..6]
        assert_eq!(h.cf1, 0xAA, "cf1 must equal data[2] = 0xAA");
        assert_eq!(h.cf2, 0xBB, "cf2 must equal data[3] = 0xBB");
        assert_eq!(h.cf3, 0xCC, "cf3 must equal data[4] = 0xCC");
        assert_eq!(h.cf4, 0xDD, "cf4 must equal data[5] = 0xDD");
    }

    // =========================================================================
    // BC-2.19.006: is_valid_iec104_frame post-classification validity gate
    // AC-167-006
    // =========================================================================

    /// BC-2.19.006 canonical vector: valid start byte (0x68) and LEN=4 returns true.
    ///
    /// Canonical vector from BC-2.19.006: `[0x68, 0x04, ...]` → true.
    ///
    /// Traces: BC-2.19.006 postcondition 1; AC-167-006; EC-001.
    #[test]
    fn test_BC_2_19_006_returns_true_for_valid_start_and_len_4_canonical_vector() {
        let data: &[u8] = &[0x68, 0x04, 0x07, 0x00, 0x00, 0x00];
        assert!(
            is_valid_iec104_frame(data),
            "start=0x68, LEN=4 must return true (BC-2.19.006 canonical vector)"
        );
    }

    /// BC-2.19.006 canonical vector: valid start byte and LEN=253 returns true.
    ///
    /// Exercises EC-002: `data[0] == 0x68`, `data[1] == 253` → true.
    ///
    /// Traces: BC-2.19.006 postcondition 1; AC-167-006; EC-002.
    #[test]
    fn test_BC_2_19_006_returns_true_for_valid_start_and_len_253() {
        let data: &[u8] = &[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00];
        assert!(
            is_valid_iec104_frame(data),
            "start=0x68, LEN=253 must return true (BC-2.19.006 EC-002)"
        );
    }

    /// BC-2.19.006 canonical vector: wrong start byte (0x48) returns false.
    ///
    /// Canonical vector from BC-2.19.006: `[0x48, 0x04, ...]` → false.
    ///
    /// Traces: BC-2.19.006 postcondition 2; AC-167-006; EC-003.
    #[test]
    fn test_BC_2_19_006_returns_false_for_wrong_start_byte_canonical_vector() {
        let data: &[u8] = &[0x48, 0x04, 0x07, 0x00, 0x00, 0x00];
        assert!(
            !is_valid_iec104_frame(data),
            "start=0x48 must return false (BC-2.19.006 canonical vector)"
        );
    }

    /// BC-2.19.006 canonical vector: LEN out of range (0xFF=255) returns false.
    ///
    /// Canonical vector from BC-2.19.006: `[0x68, 0xFF, ...]` → false.
    ///
    /// Traces: BC-2.19.006 postcondition 2; AC-167-006; EC-005.
    #[test]
    fn test_BC_2_19_006_returns_false_for_len_ff_out_of_range_canonical_vector() {
        let data: &[u8] = &[0x68, 0xFF, 0x01, 0x00, 0x00, 0x00];
        assert!(
            !is_valid_iec104_frame(data),
            "LEN=0xFF must return false (BC-2.19.006 canonical vector)"
        );
    }

    /// BC-2.19.006: empty slice returns false.
    ///
    /// A zero-length slice cannot provide the start byte.
    ///
    /// Traces: BC-2.19.006 postcondition 2; AC-167-006.
    #[test]
    fn test_BC_2_19_006_returns_false_for_empty_slice() {
        assert!(
            !is_valid_iec104_frame(&[]),
            "empty slice must return false (BC-2.19.006)"
        );
    }

    /// BC-2.19.006: one-byte slice returns false.
    ///
    /// Exercises STORY-167 EC-008 and BC-2.19.006 EC-006: `data.len() == 1` → false
    /// (cannot read LEN byte).
    ///
    /// Traces: BC-2.19.006 postcondition 2; AC-167-006; EC-006; STORY-167 EC-008.
    #[test]
    fn test_BC_2_19_006_returns_false_for_one_byte_slice() {
        assert!(
            !is_valid_iec104_frame(&[0x68]),
            "1-byte slice must return false (BC-2.19.006 EC-006 — cannot read LEN)"
        );
    }

    /// BC-2.19.006: LEN=3 (below minimum) returns false.
    ///
    /// Exercises BC-2.19.006 EC-004: `data[0] == 0x68`, `data[1] == 3` → false.
    ///
    /// Traces: BC-2.19.006 postcondition 2; AC-167-006; EC-004.
    #[test]
    fn test_BC_2_19_006_returns_false_for_len_3_below_minimum() {
        let data: &[u8] = &[0x68, 0x03, 0x00, 0x00, 0x00, 0x00];
        assert!(
            !is_valid_iec104_frame(data),
            "LEN=3 must return false (BC-2.19.006 EC-004)"
        );
    }

    /// BC-2.19.006: LEN=254 (above maximum) returns false.
    ///
    /// Exercises BC-2.19.006 EC-005: `data[0] == 0x68`, `data[1] == 254` → false.
    ///
    /// Traces: BC-2.19.006 postcondition 2; AC-167-006; EC-005.
    #[test]
    fn test_BC_2_19_006_returns_false_for_len_254_above_maximum() {
        let data: &[u8] = &[0x68, 0xFE, 0x00, 0x00, 0x00, 0x00];
        assert!(
            !is_valid_iec104_frame(data),
            "LEN=254 must return false (BC-2.19.006 EC-005)"
        );
    }

    /// BC-2.19.006 invariant 2: consistency with parse_apci_header.
    ///
    /// Any input where is_valid_iec104_frame returns true AND data.len() >= 6 must also
    /// produce Some from parse_apci_header. Tests this forward implication for two
    /// canonical boundary inputs.
    ///
    /// Traces: BC-2.19.006 invariant 2; AC-167-006 postcondition (last bullet).
    #[test]
    fn test_BC_2_19_006_invariant_consistency_with_parse_apci_header() {
        let valid_inputs: &[&[u8]] = &[
            &[0x68, 0x04, 0x07, 0x00, 0x00, 0x00], // LEN=4 minimum
            &[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00], // LEN=253 maximum
            &[0x68, 0x64, 0x00, 0x00, 0x00, 0x00], // LEN=100 mid-range
        ];
        for &data in valid_inputs {
            let gate_result = is_valid_iec104_frame(data);
            assert!(
                gate_result,
                "is_valid_iec104_frame must return true for valid input (BC-2.19.006 invariant 2)"
            );
            // If gate returns true and len >= 6, parse_apci_header must return Some.
            assert!(
                data.len() >= 6,
                "test invariant: all valid_inputs in this test have len >= 6"
            );
            let parse_result = parse_apci_header(data);
            assert!(
                parse_result.is_some(),
                "parse_apci_header must return Some when is_valid_iec104_frame returns true \
                 and data.len() >= 6 (BC-2.19.006 invariant 2)"
            );
        }
    }

    /// BC-2.19.006 invariant 2 negative: inputs where gate returns false should NOT
    /// produce Some from parse_apci_header on the reject paths covered by BC-2.19.002-004.
    ///
    /// Covers bad start byte and out-of-range LEN. (Len < 6 path requires is_valid to
    /// also return false, which it does since it needs len >= 2 for the LEN byte.)
    ///
    /// Traces: BC-2.19.006 invariant 2 (contrapositive); AC-167-006.
    #[test]
    fn test_BC_2_19_006_invariant_false_gate_implies_none_from_parse() {
        let invalid_inputs: &[&[u8]] = &[
            &[0x00, 0x04, 0x07, 0x00, 0x00, 0x00], // bad start byte
            &[0x68, 0x03, 0x07, 0x00, 0x00, 0x00], // LEN < 4
            &[0x68, 0xFE, 0x07, 0x00, 0x00, 0x00], // LEN > 253
        ];
        for &data in invalid_inputs {
            let gate_result = is_valid_iec104_frame(data);
            assert!(
                !gate_result,
                "is_valid_iec104_frame must return false for invalid input (BC-2.19.006 invariant 2 contrapositive)"
            );
            let parse_result = parse_apci_header(data);
            assert!(
                parse_result.is_none(),
                "parse_apci_header must return None when gate returns false (BC-2.19.006 invariant 2)"
            );
        }
    }

    // =========================================================================
    // ApciHeader struct correctness (AC-167-005 postcondition 1)
    // Verifies that ApciHeader fields have the documented types and that
    // PartialEq works correctly (required by BC-2.19.005).
    // =========================================================================

    /// AC-167-005: ApciHeader can be constructed and compared for equality.
    ///
    /// Verifies the struct layout matches the BC-2.19.005 postcondition 1 specification
    /// (fields: start, len, cf1, cf2, cf3, cf4; all u8). Non-vacuous: checks that two
    /// distinct structs compare as equal only when all fields match.
    ///
    /// Traces: BC-2.19.005 postcondition 1; AC-167-005 (struct layout).
    #[test]
    fn test_BC_2_19_005_apci_header_equality_and_field_layout() {
        let h1 = ApciHeader {
            start: 0x68,
            len: 4,
            cf1: 0x07,
            cf2: 0x00,
            cf3: 0x00,
            cf4: 0x00,
        };
        let h2 = ApciHeader {
            start: 0x68,
            len: 4,
            cf1: 0x07,
            cf2: 0x00,
            cf3: 0x00,
            cf4: 0x00,
        };
        let h3 = ApciHeader {
            start: 0x68,
            len: 4,
            cf1: 0x01,
            cf2: 0x00,
            cf3: 0x00,
            cf4: 0x00,
        };
        assert_eq!(h1, h2, "identical ApciHeaders must be equal");
        assert_ne!(h1, h3, "ApciHeaders differing in cf1 must not be equal");
    }
}

// =============================================================================
// STORY-168: IEC-104 Frame Format Discrimination + U-Format Session State Machine
//
// Covers BC-2.19.007–014 and VP-046 proptest skeleton.
// All tests in this module MUST FAIL (Red Gate) because classify_frame_format
// and process_u_frame are todo!() stubs. They pass after implementation.
//
// ## Contract coverage
// - BC-2.19.007: classify_frame_format returns IFormat when CF1 bit 0 = 0
// - BC-2.19.008: classify_frame_format returns SFormat when CF1 bits1:0 = 0b01
// - BC-2.19.009: classify_frame_format returns UFormat for all remaining CF1 + totality
// - BC-2.19.010: STARTDT-act (0x07) / STARTDT-con (0x0B) set session_started=true; no finding
// - BC-2.19.011: STOPDT-act (0x13) with session_started=true → T0881 Possible
// - BC-2.19.012: STOPDT-act (0x13) with session_started=false → T0881 Likely;
//                STOPDT-con (0x23) → session_started=false; no finding (ACT-only MVP)
// - BC-2.19.013: TESTFR-act (0x43) / TESTFR-con (0x83) → no finding; state unchanged
// - BC-2.19.014: non-canonical U-frame CF1 → T0814 Possible; state unchanged
// - VP-046: proptest skeleton — classify_frame_format totality over all 256 u8 values
//
// ## Canonical test vectors (DF-CANONICAL-FRAME-HOLDOUT-001)
// Used verbatim from BCs; no invented inputs where the BC provides them.
//
// ## Provenance
// Written Red-first as TDD stubs (STORY-168 strict TDD mode); GREEN after implementation.
// =============================================================================
mod story_168 {
    use proptest::prelude::*;
    use wirerust::analyzer::iec104::{
        FrameFormat, Iec104FlowState, classify_frame_format, process_u_frame,
    };
    use wirerust::findings::{ThreatCategory, Verdict};

    // =========================================================================
    // BC-2.19.007: classify_frame_format returns IFormat when CF1 bit 0 = 0
    // AC-168-001
    // =========================================================================

    /// BC-2.19.007 canonical vector: CF1=0x00 (all zeros) → IFormat.
    ///
    /// Canonical vector from BC-2.19.007: `cf1 == 0x00` → IFormat; N(S)=0.
    ///
    /// Traces: BC-2.19.007 postcondition 1; AC-168-001; EC-001.
    #[test]
    fn test_BC_2_19_007_returns_iformat_for_cf1_0x00_canonical_vector() {
        let fmt = classify_frame_format(0x00);
        assert_eq!(
            fmt,
            FrameFormat::IFormat,
            "CF1=0x00 (bit0=0) must return IFormat (BC-2.19.007 canonical vector)"
        );
    }

    /// BC-2.19.007 canonical vector: CF1=0x02 (bit0=0, bit1=1) → IFormat.
    ///
    /// Canonical vector from BC-2.19.007: `cf1 == 0x02` → IFormat.
    /// Verifies that the I-format guard is bit0 only (0x02 has bit1=1 but bit0=0).
    ///
    /// Traces: BC-2.19.007 postcondition 1; AC-168-001; EC-002.
    #[test]
    fn test_BC_2_19_007_returns_iformat_for_cf1_0x02_canonical_vector() {
        let fmt = classify_frame_format(0x02);
        assert_eq!(
            fmt,
            FrameFormat::IFormat,
            "CF1=0x02 (bit0=0, bit1=1) must return IFormat (BC-2.19.007 canonical vector EC-002)"
        );
    }

    /// BC-2.19.007 canonical vector: CF1=0x7E (all bits set except bit0) → IFormat.
    ///
    /// Canonical vector from BC-2.19.007: `cf1 == 0x7E` → IFormat.
    ///
    /// Traces: BC-2.19.007 postcondition 1; AC-168-001; EC-003.
    #[test]
    fn test_BC_2_19_007_returns_iformat_for_cf1_0x7E_canonical_vector() {
        let fmt = classify_frame_format(0x7E);
        assert_eq!(
            fmt,
            FrameFormat::IFormat,
            "CF1=0x7E (bit0=0) must return IFormat (BC-2.19.007 canonical vector EC-003)"
        );
    }

    /// BC-2.19.007 canonical vector: CF1=0xFE (largest even u8) → IFormat.
    ///
    /// Exercises EC-003 boundary: `cf1 == 0xFE` → IFormat.
    ///
    /// Traces: BC-2.19.007 postcondition 1; AC-168-001; EC-003.
    #[test]
    fn test_BC_2_19_007_returns_iformat_for_cf1_0xFE_all_even_bits_set() {
        let fmt = classify_frame_format(0xFE);
        assert_eq!(
            fmt,
            FrameFormat::IFormat,
            "CF1=0xFE (bit0=0) must return IFormat (BC-2.19.007 EC-003 boundary)"
        );
    }

    /// BC-2.19.007 invariant: all 128 even CF1 values (bit0=0) return IFormat.
    ///
    /// Exhaustive check for EC-005: all 128 values {0,2,4,...,254} → IFormat.
    /// Verifies the I-format partition of VP-046 totality.
    ///
    /// Traces: BC-2.19.007 invariant 1; AC-168-001; EC-005; VP-046 I-format partition.
    #[test]
    fn test_BC_2_19_007_invariant_all_128_even_cf1_values_return_iformat() {
        for cf1 in (0u8..=254).step_by(2) {
            let fmt = classify_frame_format(cf1);
            assert_eq!(
                fmt,
                FrameFormat::IFormat,
                "CF1=0x{cf1:02X} (even, bit0=0) must return IFormat (BC-2.19.007 invariant 1)"
            );
        }
    }

    // =========================================================================
    // BC-2.19.008: classify_frame_format returns SFormat when CF1 bits1:0 = 0b01
    // AC-168-002
    // =========================================================================

    /// BC-2.19.008 canonical vector: CF1=0x01 (minimal S-frame indicator) → SFormat.
    ///
    /// Canonical vector from BC-2.19.008: `cf1 == 0x01` → SFormat; N(R)=0.
    ///
    /// Traces: BC-2.19.008 postcondition 1; AC-168-002; EC-001.
    #[test]
    fn test_BC_2_19_008_returns_sformat_for_cf1_0x01_canonical_vector() {
        let fmt = classify_frame_format(0x01);
        assert_eq!(
            fmt,
            FrameFormat::SFormat,
            "CF1=0x01 (bits1:0=0b01) must return SFormat (BC-2.19.008 canonical vector)"
        );
    }

    /// BC-2.19.008 canonical vector: CF1=0x05 (bits1:0=0b01) → SFormat.
    ///
    /// Canonical vector from BC-2.19.008: `cf1 == 0x05` → SFormat.
    ///
    /// Traces: BC-2.19.008 postcondition 1; AC-168-002.
    #[test]
    fn test_BC_2_19_008_returns_sformat_for_cf1_0x05_canonical_vector() {
        let fmt = classify_frame_format(0x05);
        assert_eq!(
            fmt,
            FrameFormat::SFormat,
            "CF1=0x05 (bits1:0=0b01) must return SFormat (BC-2.19.008 canonical vector)"
        );
    }

    /// BC-2.19.008 negative canonical: CF1=0x03 (bits1:0=0b11) is NOT SFormat.
    ///
    /// Canonical vector from BC-2.19.008 EC-002: `cf1 == 0x03` → UFormat (BC-2.19.009), not SFormat.
    ///
    /// Traces: BC-2.19.008 EC-002; AC-168-002 (negative).
    #[test]
    fn test_BC_2_19_008_does_not_return_sformat_for_cf1_0x03_uformat() {
        let fmt = classify_frame_format(0x03);
        assert_ne!(
            fmt,
            FrameFormat::SFormat,
            "CF1=0x03 (bits1:0=0b11) must NOT return SFormat (BC-2.19.008 EC-002 — is UFormat)"
        );
        assert_eq!(
            fmt,
            FrameFormat::UFormat,
            "CF1=0x03 (bits1:0=0b11) must return UFormat (BC-2.19.009)"
        );
    }

    /// BC-2.19.008 invariant: all 64 CF1 values with bits1:0=0b01 return SFormat.
    ///
    /// Exhaustive check for EC-004: all 64 values {0x01,0x05,0x09,...,0xFD} → SFormat.
    /// Verifies the S-format partition of VP-046 totality.
    ///
    /// Traces: BC-2.19.008 invariant 3; AC-168-002; EC-004; VP-046 S-format partition.
    #[test]
    fn test_BC_2_19_008_invariant_all_64_cf1_values_bits1_0_0b01_return_sformat() {
        for n in 0u8..64 {
            // bits1:0 = 0b01, upper 6 bits vary: value = n*4 + 0x01
            let cf1 = n.wrapping_mul(4).wrapping_add(0x01);
            let fmt = classify_frame_format(cf1);
            assert_eq!(
                fmt,
                FrameFormat::SFormat,
                "CF1=0x{cf1:02X} (bits1:0=0b01) must return SFormat (BC-2.19.008 invariant 3)"
            );
        }
    }

    // =========================================================================
    // BC-2.19.009: classify_frame_format returns UFormat for remaining CF1 values
    //              + VP-046 totality
    // AC-168-003, AC-168-009
    // =========================================================================

    /// BC-2.19.009 canonical vector: CF1=0x07 (STARTDT-act) → UFormat.
    ///
    /// Canonical vector from BC-2.19.009: `cf1 == 0x07` → UFormat.
    ///
    /// Traces: BC-2.19.009 postcondition 1; AC-168-003; EC-001.
    #[test]
    fn test_BC_2_19_009_returns_uformat_for_cf1_0x07_startdt_act_canonical_vector() {
        let fmt = classify_frame_format(0x07);
        assert_eq!(
            fmt,
            FrameFormat::UFormat,
            "CF1=0x07 (STARTDT-act, bits1:0=0b11) must return UFormat (BC-2.19.009 canonical vector)"
        );
    }

    /// BC-2.19.009 canonical vector: CF1=0x0B (STARTDT-con) → UFormat.
    ///
    /// Canonical vector from BC-2.19.009: `cf1 == 0x0B` → UFormat.
    ///
    /// Traces: BC-2.19.009 postcondition 1; AC-168-003; EC-001.
    #[test]
    fn test_BC_2_19_009_returns_uformat_for_cf1_0x0B_startdt_con_canonical_vector() {
        let fmt = classify_frame_format(0x0B);
        assert_eq!(
            fmt,
            FrameFormat::UFormat,
            "CF1=0x0B (STARTDT-con, bits1:0=0b11) must return UFormat (BC-2.19.009)"
        );
    }

    /// BC-2.19.009 canonical vector: CF1=0x13 (STOPDT-act) → UFormat.
    ///
    /// Canonical vector from BC-2.19.009: `cf1 == 0x13` → UFormat.
    ///
    /// Traces: BC-2.19.009 postcondition 1; AC-168-003; EC-002.
    #[test]
    fn test_BC_2_19_009_returns_uformat_for_cf1_0x13_stopdt_act_canonical_vector() {
        let fmt = classify_frame_format(0x13);
        assert_eq!(
            fmt,
            FrameFormat::UFormat,
            "CF1=0x13 (STOPDT-act, bits1:0=0b11) must return UFormat (BC-2.19.009)"
        );
    }

    /// BC-2.19.009 canonical vector: CF1=0x03 (non-canonical U) → UFormat.
    ///
    /// Canonical vector from BC-2.19.009: `cf1 == 0x03` → UFormat (non-canonical).
    ///
    /// Traces: BC-2.19.009 postcondition 1; AC-168-003.
    #[test]
    fn test_BC_2_19_009_returns_uformat_for_cf1_0x03_non_canonical_canonical_vector() {
        let fmt = classify_frame_format(0x03);
        assert_eq!(
            fmt,
            FrameFormat::UFormat,
            "CF1=0x03 (bits1:0=0b11, non-canonical) must return UFormat (BC-2.19.009 canonical vector)"
        );
    }

    /// BC-2.19.009 canonical vector: CF1=0xFF (all bits set) → UFormat.
    ///
    /// Canonical vector from BC-2.19.009: `cf1 == 0xFF` → UFormat (non-canonical).
    ///
    /// Traces: BC-2.19.009 postcondition 1; AC-168-003; EC-004.
    #[test]
    fn test_BC_2_19_009_returns_uformat_for_cf1_0xFF_canonical_vector() {
        let fmt = classify_frame_format(0xFF);
        assert_eq!(
            fmt,
            FrameFormat::UFormat,
            "CF1=0xFF (bits1:0=0b11) must return UFormat (BC-2.19.009 canonical vector EC-004)"
        );
    }

    /// BC-2.19.009 invariant: all 64 CF1 values with bits1:0=0b11 return UFormat.
    ///
    /// Exhaustive check for EC-005: all 64 values {0x03,0x07,0x0B,...,0xFF} → UFormat.
    /// Verifies the U-format partition of VP-046 totality.
    ///
    /// Traces: BC-2.19.009 invariant 1; AC-168-003; EC-005; VP-046 U-format partition.
    #[test]
    fn test_BC_2_19_009_invariant_all_64_cf1_values_bits1_0_0b11_return_uformat() {
        for n in 0u8..64 {
            // bits1:0 = 0b11, upper 6 bits vary: value = n*4 + 0x03
            let cf1 = n.wrapping_mul(4).wrapping_add(0x03);
            let fmt = classify_frame_format(cf1);
            assert_eq!(
                fmt,
                FrameFormat::UFormat,
                "CF1=0x{cf1:02X} (bits1:0=0b11) must return UFormat (BC-2.19.009 invariant 1)"
            );
        }
    }

    /// BC-2.19.009 invariant totality: all 256 u8 CF1 values produce exactly one FrameFormat.
    ///
    /// Exhaustively tests VP-046 totality property: every u8 maps to exactly one of
    /// {IFormat, SFormat, UFormat} with no panic (covers all three partitions simultaneously).
    /// This unit test is a Red-Gate complement to the proptest below.
    ///
    /// Traces: BC-2.19.009 invariant 1; AC-168-003; VP-046 totality.
    #[test]
    fn test_BC_2_19_009_invariant_vp046_totality_exhaustive_all_256_values() {
        for cf1 in 0u8..=255 {
            let fmt = classify_frame_format(cf1);
            // Verify partition is total: each value maps to exactly one variant.
            match cf1 & 0x03 {
                0x00 | 0x02 => {
                    assert_eq!(
                        fmt,
                        FrameFormat::IFormat,
                        "CF1=0x{cf1:02X} (bit0=0) must return IFormat (VP-046 totality)"
                    );
                }
                0x01 => {
                    assert_eq!(
                        fmt,
                        FrameFormat::SFormat,
                        "CF1=0x{cf1:02X} (bits1:0=0b01) must return SFormat (VP-046 totality)"
                    );
                }
                0x03 => {
                    assert_eq!(
                        fmt,
                        FrameFormat::UFormat,
                        "CF1=0x{cf1:02X} (bits1:0=0b11) must return UFormat (VP-046 totality)"
                    );
                }
                _ => unreachable!("cf1 & 0x03 is always 0x00..=0x03"),
            }
        }
    }

    // =========================================================================
    // BC-2.19.010: STARTDT-act (0x07) and STARTDT-con (0x0B) set session_started=true
    // AC-168-004
    // =========================================================================

    /// BC-2.19.010 canonical vector: STARTDT-act on fresh flow → session_started true, no finding.
    ///
    /// Canonical vector from BC-2.19.010: CF1=0x07, prior false → true, no finding.
    ///
    /// Traces: BC-2.19.010 postconditions 1-3; AC-168-004; EC-001.
    #[test]
    fn test_BC_2_19_010_startdt_act_sets_session_started_true_on_fresh_flow() {
        let mut state = Iec104FlowState::default();
        assert!(
            !state.session_started,
            "precondition: session_started must be false initially"
        );
        let finding = process_u_frame(&mut state, 0x07);
        assert!(
            state.session_started,
            "STARTDT-act (0x07) must set session_started=true (BC-2.19.010 postcondition 1)"
        );
        assert!(
            finding.is_none(),
            "STARTDT-act must not emit a finding (BC-2.19.010 postcondition 3)"
        );
    }

    /// BC-2.19.010 canonical vector: duplicate STARTDT-act → session_started remains true.
    ///
    /// Canonical vector from BC-2.19.010: CF1=0x07, prior true → true (idempotent), no finding.
    ///
    /// Traces: BC-2.19.010 invariant 1; AC-168-004; EC-002.
    #[test]
    fn test_BC_2_19_010_startdt_act_idempotent_when_already_started() {
        let mut state = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let finding = process_u_frame(&mut state, 0x07);
        assert!(
            state.session_started,
            "STARTDT-act when already started must leave session_started=true (BC-2.19.010 invariant 1)"
        );
        assert!(
            finding.is_none(),
            "Duplicate STARTDT-act must not emit a finding (BC-2.19.010 invariant 1)"
        );
    }

    /// BC-2.19.010 canonical vector: STARTDT-con (0x0B) on fresh flow → session_started true, no finding.
    ///
    /// Canonical vector from BC-2.19.010: CF1=0x0B, prior false → true, no finding.
    ///
    /// Traces: BC-2.19.010 postcondition 4; AC-168-004; EC-003.
    #[test]
    fn test_BC_2_19_010_startdt_con_sets_session_started_true_on_fresh_flow() {
        let mut state = Iec104FlowState::default();
        assert!(
            !state.session_started,
            "precondition: session_started must be false initially"
        );
        let finding = process_u_frame(&mut state, 0x0B);
        assert!(
            state.session_started,
            "STARTDT-con (0x0B) without prior STARTDT-act must set session_started=true \
             (BC-2.19.010 postcondition 4; EC-003)"
        );
        assert!(
            finding.is_none(),
            "STARTDT-con must not emit a finding (BC-2.19.010 postcondition 3)"
        );
    }

    // =========================================================================
    // BC-2.19.011: STOPDT-act while session_started=true → T0881 Possible
    // AC-168-005
    // =========================================================================

    /// BC-2.19.011 canonical vector: STOPDT-act after STARTDT → T0881 Possible; session false.
    ///
    /// Canonical vector from BC-2.19.011: CF1=0x13, prior true → false, T0881 Possible.
    /// Asserts: finding emitted, verdict==Possible, mitre_techniques contains "T0881".
    ///
    /// Traces: BC-2.19.011 postconditions 1-2; AC-168-005; EC-001.
    #[test]
    fn test_BC_2_19_011_stopdt_act_after_startdt_emits_t0881_possible() {
        let mut state = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let finding = process_u_frame(&mut state, 0x13);
        assert!(
            !state.session_started,
            "STOPDT-act must set session_started=false (BC-2.19.011 postcondition 1)"
        );
        let f = finding.expect(
            "STOPDT-act when session active must emit T0881 finding (BC-2.19.011 postcondition 2)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "STOPDT-act after STARTDT must emit Verdict::Possible (BC-2.19.011 postcondition 2)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Impact,
            "T0881 finding must have category Impact (BC-2.19.011; L3 category regression guard)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0881"),
            "T0881 finding must have mitre_techniques containing \"T0881\" \
             (BC-2.19.011 MITRE Techniques field)"
        );
    }

    /// BC-2.19.011 EC-003: STOPDT-act followed by STARTDT-act — T0881 emitted then session restarts.
    ///
    /// Tests that STOPDT emits T0881 and STARTDT can restart the session afterward.
    ///
    /// Traces: BC-2.19.011 EC-003; AC-168-005.
    #[test]
    fn test_BC_2_19_011_stopdt_act_followed_by_startdt_act_restarts_session() {
        let mut state = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let stop_finding = process_u_frame(&mut state, 0x13);
        assert!(
            !state.session_started,
            "STOPDT must set session_started=false"
        );
        assert!(stop_finding.is_some(), "STOPDT must emit a finding");
        // Now STARTDT restarts the session
        let start_finding = process_u_frame(&mut state, 0x07);
        assert!(
            state.session_started,
            "STARTDT after STOPDT must set session_started=true"
        );
        assert!(start_finding.is_none(), "STARTDT must not emit a finding");
    }

    // =========================================================================
    // BC-2.19.012: STOPDT-act without prior STARTDT → T0881 Likely;
    //              STOPDT-con → session_started=false, no finding (ACT-only MVP)
    // AC-168-006
    // =========================================================================

    /// BC-2.19.012 canonical vector: STOPDT-act without prior STARTDT → T0881 Likely.
    ///
    /// Canonical vector from BC-2.19.012: CF1=0x13, prior false → false, T0881 Likely.
    /// The elevated confidence (Likely > Possible) distinguishes anomalous STOPDT.
    ///
    /// Traces: BC-2.19.012 postconditions 1-2; AC-168-006; EC-001.
    #[test]
    fn test_BC_2_19_012_stopdt_act_without_startdt_emits_t0881_likely() {
        let mut state = Iec104FlowState::default();
        assert!(
            !state.session_started,
            "precondition: session_started must be false"
        );
        let finding = process_u_frame(&mut state, 0x13);
        assert!(
            !state.session_started,
            "session_started must remain false after STOPDT without STARTDT \
             (BC-2.19.012 postcondition 1, invariant 3)"
        );
        let f = finding.expect(
            "STOPDT-act without STARTDT must emit T0881 finding (BC-2.19.012 postcondition 2)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "STOPDT without prior STARTDT must emit Verdict::Likely (BC-2.19.012 postcondition 2)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Impact,
            "T0881 finding must have category Impact (BC-2.19.012; L3 category regression guard)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0881"),
            "T0881 finding must have mitre_techniques containing \"T0881\" \
             (BC-2.19.012 MITRE Techniques field)"
        );
        // BC-2.19.012 postcondition 3 (M1): Likely-path finding MUST carry the cold-start
        // note so analysts can distinguish it from a Possible (session-active) stop.
        assert!(
            f.summary.contains("without prior STARTDT")
                || f.evidence
                    .iter()
                    .any(|e| e.contains("without prior STARTDT")),
            "STOPDT-act without STARTDT finding must contain the note \
             'without prior STARTDT' in summary or evidence (BC-2.19.012 PC3)"
        );
    }

    /// BC-2.19.012 confidence distinction: STOPDT-act without STARTDT is Likely; with STARTDT is Possible.
    ///
    /// Tests the confidence escalation invariant: Likely > Possible (BC-2.19.012 invariant 1).
    ///
    /// Traces: BC-2.19.012 invariant 1; AC-168-006 (confidence distinction); AC-168-005.
    #[test]
    fn test_BC_2_19_012_invariant_stopdt_confidence_escalation_likely_vs_possible() {
        // Path 1: session active → Possible
        let mut state_active = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let f_possible = process_u_frame(&mut state_active, 0x13)
            .expect("STOPDT after STARTDT must emit finding");
        assert_eq!(
            f_possible.verdict,
            Verdict::Possible,
            "STOPDT after STARTDT must emit Possible (BC-2.19.011)"
        );

        // Path 2: no prior STARTDT → Likely
        let mut state_cold = Iec104FlowState::default();
        let f_likely = process_u_frame(&mut state_cold, 0x13)
            .expect("STOPDT without STARTDT must emit finding");
        assert_eq!(
            f_likely.verdict,
            Verdict::Likely,
            "STOPDT without STARTDT must emit Likely (BC-2.19.012)"
        );

        // Verify the confidence escalation holds — not equal
        assert_ne!(
            f_possible.verdict, f_likely.verdict,
            "Possible and Likely must be different verdicts (BC-2.19.012 invariant 1)"
        );
    }

    /// BC-2.19.012: STOPDT-con (0x23) sets session_started=false with no finding (ACT-only MVP).
    ///
    /// Canonical vector from BC-2.19.011 EC-002: CF1=0x23, prior true → false, no finding.
    ///
    /// Traces: BC-2.19.011 EC-002; BC-2.19.012 postcondition note (ACT-only MVP); AC-168-006.
    #[test]
    fn test_BC_2_19_012_stopdt_con_sets_session_false_no_finding_act_only_mvp() {
        let mut state = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let finding = process_u_frame(&mut state, 0x23);
        assert!(
            !state.session_started,
            "STOPDT-con (0x23) must set session_started=false (BC-2.19.012 ACT-only MVP)"
        );
        assert!(
            finding.is_none(),
            "STOPDT-con must not emit a finding (BC-2.19.012 ACT-only MVP; ADR-013 Decision 5)"
        );
    }

    // =========================================================================
    // BC-2.19.013: TESTFR-act (0x43) and TESTFR-con (0x83) produce no finding
    // AC-168-007
    // =========================================================================

    /// BC-2.19.013 canonical vector: TESTFR-act (0x43) → no finding; state unchanged.
    ///
    /// Canonical vector from BC-2.19.013: CF1=0x43 → no finding; session state not modified.
    ///
    /// Traces: BC-2.19.013 postconditions 1-2; AC-168-007; EC-001.
    #[test]
    fn test_BC_2_19_013_testfr_act_emits_no_finding_canonical_vector() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0x43);
        assert!(
            finding.is_none(),
            "TESTFR-act (0x43) must not emit a finding (BC-2.19.013 postcondition 1)"
        );
    }

    /// BC-2.19.013 canonical vector: TESTFR-con (0x83) → no finding; state unchanged.
    ///
    /// Canonical vector from BC-2.19.013: CF1=0x83 → no finding; session state not modified.
    ///
    /// Traces: BC-2.19.013 postconditions 1-2; AC-168-007; EC-002.
    #[test]
    fn test_BC_2_19_013_testfr_con_emits_no_finding_canonical_vector() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0x83);
        assert!(
            finding.is_none(),
            "TESTFR-con (0x83) must not emit a finding (BC-2.19.013 postcondition 1)"
        );
    }

    /// BC-2.19.013 invariant: TESTFR does not modify session_started.
    ///
    /// Verifies BC-2.19.013 postcondition 2 (state unchanged) for both act and con.
    ///
    /// Traces: BC-2.19.013 postcondition 2; AC-168-007; EC-001/002.
    #[test]
    fn test_BC_2_19_013_invariant_testfr_does_not_modify_session_started() {
        // Case 1: session_started=false — should remain false after TESTFR
        let mut state_cold = Iec104FlowState::default();
        let _ = process_u_frame(&mut state_cold, 0x43);
        assert!(
            !state_cold.session_started,
            "TESTFR-act must not change session_started from false (BC-2.19.013 postcondition 2)"
        );
        let _ = process_u_frame(&mut state_cold, 0x83);
        assert!(
            !state_cold.session_started,
            "TESTFR-con must not change session_started from false (BC-2.19.013 postcondition 2)"
        );

        // Case 2: session_started=true — should remain true after TESTFR
        let mut state_active = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let _ = process_u_frame(&mut state_active, 0x43);
        assert!(
            state_active.session_started,
            "TESTFR-act must not change session_started from true (BC-2.19.013 postcondition 2)"
        );
        let _ = process_u_frame(&mut state_active, 0x83);
        assert!(
            state_active.session_started,
            "TESTFR-con must not change session_started from true (BC-2.19.013 postcondition 2)"
        );
    }

    // =========================================================================
    // BC-2.19.014: non-canonical U-frame CF1 → T0814 Possible (CVE-2026-1773)
    // AC-168-008
    // =========================================================================

    /// BC-2.19.014 canonical vector: CF1=0x03 (non-canonical U) → T0814 Possible.
    ///
    /// Canonical vector from BC-2.19.014: CF1=0x03 → T0814 Possible; state unchanged.
    ///
    /// Traces: BC-2.19.014 postconditions 1-3; AC-168-008; EC-001.
    #[test]
    fn test_BC_2_19_014_non_canonical_cf1_0x03_emits_t0814_possible() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0x03);
        let f = finding.expect(
            "Non-canonical U CF1=0x03 must emit T0814 finding (BC-2.19.014 postcondition 1)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "T0814 finding must have Verdict::Possible (BC-2.19.014 postcondition 1)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "T0814 finding must have category Anomaly (BC-2.19.014; L3 category regression guard)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "T0814 finding must have mitre_techniques containing \"T0814\" \
             (BC-2.19.014 MITRE Techniques field)"
        );
    }

    /// BC-2.19.014 canonical vector: CF1=0xFF (all bits set, non-canonical) → T0814 Possible.
    ///
    /// Canonical vector from BC-2.19.014: CF1=0xFF → T0814 Possible.
    ///
    /// Traces: BC-2.19.014 postconditions 1-2; AC-168-008; EC-002.
    #[test]
    fn test_BC_2_19_014_non_canonical_cf1_0xFF_emits_t0814_possible_canonical_vector() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0xFF);
        let f = finding.expect(
            "Non-canonical U CF1=0xFF must emit T0814 finding (BC-2.19.014 canonical vector EC-002)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "T0814 must have Verdict::Possible (BC-2.19.014 postcondition 1)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "T0814 finding must have category Anomaly (BC-2.19.014; L3 category regression guard)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "T0814 finding must contain \"T0814\" in mitre_techniques"
        );
    }

    /// BC-2.19.014 canonical vector: CF1=0x0F (bits1:0=0b11, non-canonical) → T0814 Possible.
    ///
    /// Canonical vector from BC-2.19.014: CF1=0x0F → T0814 Possible.
    ///
    /// Traces: BC-2.19.014 postcondition 1; AC-168-008; EC-003.
    #[test]
    fn test_BC_2_19_014_non_canonical_cf1_0x0F_emits_t0814_possible_canonical_vector() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0x0F);
        let f = finding.expect(
            "Non-canonical U CF1=0x0F must emit T0814 finding (BC-2.19.014 canonical vector EC-003)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "T0814 must be Possible (BC-2.19.014)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "T0814 finding must have category Anomaly (BC-2.19.014; L3 category regression guard)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "T0814 finding must contain \"T0814\" in mitre_techniques"
        );
    }

    /// BC-2.19.014 canonical vector: CF1=0x1B (bits1:0=0b11, non-canonical) → T0814 Possible.
    ///
    /// Additional non-canonical sample: 0x1B = 0b00011011 (bits1:0=0b11, not in canonical set).
    ///
    /// Traces: BC-2.19.014 postcondition 1; AC-168-008.
    #[test]
    fn test_BC_2_19_014_non_canonical_cf1_0x1B_emits_t0814_possible() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0x1B);
        let f = finding.expect("Non-canonical U CF1=0x1B must emit T0814 finding (BC-2.19.014)");
        assert_eq!(f.verdict, Verdict::Possible, "T0814 must be Possible");
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "T0814 finding must have category Anomaly (BC-2.19.014; L3 category regression guard)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "must contain T0814"
        );
    }

    /// BC-2.19.014 invariant: non-canonical U-frame does not advance session state.
    ///
    /// BC-2.19.014 invariant 1 (fail-closed): non-canonical CF1 must not mutate session_started.
    ///
    /// Traces: BC-2.19.014 postcondition 3; invariant 1; AC-168-008.
    #[test]
    fn test_BC_2_19_014_invariant_non_canonical_u_frame_does_not_advance_session_state() {
        // Case 1: session_started=false — must remain false after non-canonical U-frame
        let mut state_cold = Iec104FlowState::default();
        let _ = process_u_frame(&mut state_cold, 0x0F);
        assert!(
            !state_cold.session_started,
            "Non-canonical U-frame must not change session_started from false \
             (BC-2.19.014 invariant 1 fail-closed)"
        );

        // Case 2: session_started=true — must remain true after non-canonical U-frame
        let mut state_active = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let _ = process_u_frame(&mut state_active, 0xFF);
        assert!(
            state_active.session_started,
            "Non-canonical U-frame must not change session_started from true \
             (BC-2.19.014 invariant 1 fail-closed)"
        );
    }

    /// BC-2.19.014 negative: canonical CF1 values do NOT trigger T0814.
    ///
    /// Verifies the canonical set exclusivity (BC-2.19.014 invariant 3): the six canonical
    /// values must NOT produce T0814. Specifically tests TESTFR (no finding) and
    /// STARTDT-act (no finding). STOPDT-act produces a finding but it's T0881, not T0814.
    ///
    /// Traces: BC-2.19.014 canonical test vector rows 3-4; AC-168-008 negative; invariant 3.
    #[test]
    fn test_BC_2_19_014_negative_canonical_cf1_values_do_not_emit_t0814() {
        // Canonical values that produce no finding at all
        let no_finding_cases: &[u8] = &[0x07, 0x0B, 0x43, 0x83, 0x23];
        for &cf1 in no_finding_cases {
            let mut state = Iec104FlowState::default();
            let finding = process_u_frame(&mut state, cf1);
            assert!(
                finding.is_none(),
                "Canonical CF1=0x{cf1:02X} must not emit any finding (BC-2.19.014 negative)"
            );
        }
        // STOPDT-act produces T0881 (not T0814)
        let mut state = Iec104FlowState::default();
        let f = process_u_frame(&mut state, 0x13)
            .expect("STOPDT-act must emit a finding (T0881, not T0814)");
        assert!(
            !f.mitre_techniques.iter().any(|t| t == "T0814"),
            "STOPDT-act (0x13, canonical) must NOT emit T0814 (BC-2.19.014 negative)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0881"),
            "STOPDT-act must emit T0881, not T0814"
        );
    }

    // =========================================================================
    // VP-046 proptest skeleton: classify_frame_format totality
    // AC-168-009
    // =========================================================================

    // VP-046 proptest skeleton — classify_frame_format totality over all 256 u8 values.
    //
    // Per AC-168-009: this skeleton compiles and FAILS Red Gate because classify_frame_format
    // is a todo!(). Full proptest run (1000+ cases) is in STORY-174 (VP-046 full proof run).
    //
    // Per STORY-168 Architecture Compliance: classify_frame_format must be total over all
    // 256 u8 values with no unhandled case and no panic (BC-2.19.009 invariant 1).
    // The proptest oracle mirrors the bit-partition logic independently of the implementation.
    //
    // Traces: BC-2.19.009 invariant 1; AC-168-009; VP-046.
    proptest! {
        #[test]
        fn proptest_vp046_frame_format_totality(cf1 in 0u8..=255u8) {
            // Every u8 maps to exactly one FrameFormat — no unhandled case
            let fmt = classify_frame_format(cf1);
            // Partitioning assertion (independent oracle; not copied from implementation):
            match cf1 & 0x03 {
                0x00 | 0x02 => {
                    prop_assert!(
                        matches!(fmt, FrameFormat::IFormat),
                        "CF1=0x{:02X} (bit0=0) must be IFormat (VP-046)",
                        cf1
                    );
                }
                0x01 => {
                    prop_assert!(
                        matches!(fmt, FrameFormat::SFormat),
                        "CF1=0x{:02X} (bits1:0=0b01) must be SFormat (VP-046)",
                        cf1
                    );
                }
                0x03 => {
                    prop_assert!(
                        matches!(fmt, FrameFormat::UFormat),
                        "CF1=0x{:02X} (bits1:0=0b11) must be UFormat (VP-046)",
                        cf1
                    );
                }
                _ => unreachable!("cf1 & 0x03 is always 0x00..=0x03"),
            }
        }
    }
}

// =============================================================================
// STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields
//
// Covers BC-2.19.015–018 and all edge cases from the BCs and STORY-169 EC table.
// All tests in this module MUST FAIL (Red Gate) because parse_asdu is todo!().
// They pass after implementation.
//
// ## Contract coverage
// - BC-2.19.015: parse_asdu returns None for asdu_body.len() < 6; purity invariant (no panic).
// - BC-2.19.016: type_id = asdu_body[0]; sq = (asdu_body[1] & 0x80) != 0;
//                count = asdu_body[1] & 0x7F.
// - BC-2.19.017: cot_cause = asdu_body[2] & 0x3F; cot_pn = (asdu_body[2] & 0x40) != 0;
//                cot_test = (asdu_body[2] & 0x80) != 0; cot_originator = asdu_body[3].
// - BC-2.19.018: casdu = u16 LE from asdu_body[4..6];
//                first_ioa = Some(24-bit LE) when count>0 && len>=9, else None.
//
// ## STORY-169 EC table coverage
// - EC-001: 5B → None
// - EC-002: 6B → Some, first_ioa=None
// - EC-003: 6–8B, count>0 → first_ioa=None
// - EC-004: 9B, count>0 → first_ioa=Some
// - EC-005: count==0 → first_ioa=None regardless of body length
// - EC-006: IOA=[0xFF,0xFF,0xFF] → first_ioa=Some(0xFFFFFF)
// - EC-007: TypeID=0 (undefined) passes through without rejection
// - EC-008: cot_test=true is extracted; caller handles [TEST] tagging
//
// ## Canonical test vectors (DF-CANONICAL-FRAME-HOLDOUT-001)
// Used verbatim from BC-2.19.015–018; no invented inputs where BCs provide them.
//
// ## Provenance
// Written Red-first as TDD stubs (STORY-169 strict TDD mode);
// GREEN after implementer delivers parse_asdu.
// =============================================================================
mod story_169 {
    use wirerust::analyzer::iec104::{Asdu, parse_asdu};

    // -------------------------------------------------------------------------
    // Test helpers: construct canonical byte bodies without repetition.
    // Layout: [type_id, vsq, cot_byte2, cot_orig, casdu_lo, casdu_hi, ...]
    // -------------------------------------------------------------------------

    /// Build a 6-byte minimum-valid ASDU body (DUI only, no IOA bytes).
    fn body_6(
        type_id: u8,
        vsq: u8,
        cot_byte2: u8,
        cot_orig: u8,
        casdu_lo: u8,
        casdu_hi: u8,
    ) -> Vec<u8> {
        vec![type_id, vsq, cot_byte2, cot_orig, casdu_lo, casdu_hi]
    }

    // =========================================================================
    // BC-2.19.015: ASDU Minimum-Length Guard Rejects Body Shorter Than 6 Bytes
    // AC-169-001
    // =========================================================================

    /// BC-2.19.015 canonical vector: 0-byte ASDU body → None.
    ///
    /// Canonical vector from BC-2.19.015 table: `asdu_body.len() == 0` → None + T0814 (caller).
    /// parse_asdu must not access any byte and must not panic.
    ///
    /// Traces: BC-2.19.015 postconditions 1–3; AC-169-001; BC-2.19.015 EC-001.
    #[test]
    fn test_BC_2_19_015_returns_none_for_empty_body() {
        let result = parse_asdu(&[]);
        assert!(
            result.is_none(),
            "0-byte ASDU body must return None (BC-2.19.015 postcondition 1)"
        );
    }

    /// BC-2.19.015 canonical vector: 5-byte ASDU body → None (one byte short of DUI minimum).
    ///
    /// Canonical vector from BC-2.19.015 table: `asdu_body.len() == 5` → None + T0814 (caller).
    /// This is STORY-169 EC-001 (5B → None) and BC-2.19.015 EC-002.
    ///
    /// Traces: BC-2.19.015 postconditions 1–3; AC-169-001; EC-002 (BC-015); EC-001 (STORY-169).
    #[test]
    fn test_BC_2_19_015_returns_none_for_five_bytes_canonical_vector() {
        // Five bytes that look like a valid ASDU start but are one byte short of the 6-byte DUI.
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01];
        let result = parse_asdu(data);
        assert!(
            result.is_none(),
            "5-byte ASDU body must return None — one byte short of 6-byte DUI minimum \
             (BC-2.19.015 canonical vector / STORY-169 EC-001)"
        );
    }

    /// BC-2.19.015 canonical vector: exactly 6 bytes → Some(Asdu) with first_ioa=None.
    ///
    /// Canonical vector from BC-2.19.015 table: `asdu_body.len() == 6` → Some(Asdu{...}).
    /// Minimum valid DUI — no IOA bytes present, so first_ioa=None.
    /// STORY-169 EC-002 (6B → Some, first_ioa=None). All nine fields asserted individually.
    ///
    /// Traces: BC-2.19.015 postcondition 1 (accept); BC-2.19.016/017/018; AC-169-001/002/003/004/005;
    ///         EC-003 (BC-015); EC-002 (STORY-169).
    #[test]
    fn test_BC_2_19_015_returns_some_for_exactly_six_bytes_minimum_valid() {
        // Canonical 6-byte body: TypeID=45 (C_SC_NA_1), count=1, cause=6 (activation), orig=0, casdu=1
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data)
            .expect("6-byte ASDU body must return Some (BC-2.19.015 EC-003 / STORY-169 EC-002)");
        // BC-2.19.016: TypeID and VSQ fields
        assert_eq!(
            asdu.type_id, 0x2D,
            "type_id must be 0x2D (45) — BC-2.19.016 PC1"
        );
        assert!(!asdu.sq, "sq must be false for VSQ=0x01 — BC-2.19.016 PC2");
        assert_eq!(
            asdu.count, 1,
            "count must be 1 for VSQ=0x01 — BC-2.19.016 PC3"
        );
        // BC-2.19.017: COT fields
        assert_eq!(
            asdu.cot_cause, 6,
            "cot_cause must be 6 for byte2=0x06 — BC-2.19.017 PC1"
        );
        assert!(
            !asdu.cot_pn,
            "cot_pn must be false for byte2=0x06 — BC-2.19.017 PC2"
        );
        assert!(
            !asdu.cot_test,
            "cot_test must be false for byte2=0x06 — BC-2.19.017 PC3"
        );
        assert_eq!(
            asdu.cot_originator, 0,
            "cot_originator must be 0 — BC-2.19.017 PC4"
        );
        // BC-2.19.018: CASDU and first_ioa
        assert_eq!(
            asdu.casdu, 1,
            "casdu must be 1 for bytes [0x01,0x00] LE — BC-2.19.018 PC1"
        );
        assert_eq!(
            asdu.first_ioa, None,
            "first_ioa must be None for 6-byte body (insufficient IOA bytes) \
             — BC-2.19.018 PC3 / STORY-169 EC-002"
        );
    }

    /// BC-2.19.015 invariant 2: parse_asdu must not panic for any short input (len 0–5).
    ///
    /// Exercises every boundary length 0–5 inclusive. If any call panics, the test fails.
    /// Also verifies all return None (no vacuous pass).
    ///
    /// Traces: BC-2.19.015 invariants 1–2; AC-169-001; VP-047 (no-panic).
    #[test]
    fn test_BC_2_19_015_invariant_no_panic_on_all_short_lengths() {
        let short_inputs: &[&[u8]] = &[
            &[],
            &[0x2D],
            &[0x2D, 0x01],
            &[0x2D, 0x01, 0x06],
            &[0x2D, 0x01, 0x06, 0x00],
            &[0x2D, 0x01, 0x06, 0x00, 0x01],
        ];
        for &data in short_inputs {
            // Must return None without panicking (BC-2.19.015 postcondition 1; invariant 2).
            let result = parse_asdu(data);
            assert!(
                result.is_none(),
                "ASDU body of len {} must return None — min-length guard (BC-2.19.015 invariant 2)",
                data.len()
            );
        }
    }

    /// BC-2.19.015 invariant 2 / AC-169-006: parse_asdu is pure — same input produces same output.
    ///
    /// Calls parse_asdu three times on the same 9-byte input and verifies all results are
    /// structurally equal (PartialEq on Option<Asdu>). Also verifies the result is Some for
    /// a valid input, making the equality assertion non-vacuous.
    ///
    /// Traces: BC-2.19.015 invariant 2; AC-169-006 (purity — no side effects, deterministic).
    #[test]
    fn test_BC_2_19_015_invariant_parse_asdu_pure_deterministic() {
        // 9-byte body: TypeID=45, count=1, cause=6, orig=0, casdu=1, IOA=5
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00];
        let r1: Option<Asdu> = parse_asdu(data);
        let r2: Option<Asdu> = parse_asdu(data);
        let r3: Option<Asdu> = parse_asdu(data);
        // All three calls must produce structurally identical results (purity).
        assert_eq!(
            r1, r2,
            "parse_asdu must be deterministic: call 1 == call 2 (AC-169-006)"
        );
        assert_eq!(
            r2, r3,
            "parse_asdu must be deterministic: call 2 == call 3 (AC-169-006)"
        );
        // Verify the result is Some — ensures the equality assertion is non-vacuous
        // (None == None is trivially true but meaningless for purity verification).
        let asdu = r1.expect("9-byte body must return Some — purity test requires non-None result");
        assert_eq!(
            asdu.type_id, 0x2D,
            "type_id must be stable across all calls (AC-169-006)"
        );
        assert_eq!(
            asdu.first_ioa,
            Some(5),
            "first_ioa must be stable across all calls (AC-169-006)"
        );
    }

    // =========================================================================
    // BC-2.19.016: TypeID and VSQ Extraction from ASDU Bytes 0–1
    // AC-169-002
    // =========================================================================

    /// BC-2.19.016 postcondition 1: type_id equals asdu_body[0] verbatim for multiple values.
    ///
    /// Exercises TypeID values 1, 45 (C_SC_NA_1), 100 (C_IC_NA_1), and 255 to confirm
    /// the verbatim extraction rule holds regardless of the TypeID value.
    ///
    /// Traces: BC-2.19.016 postcondition 1; AC-169-002.
    #[test]
    fn test_BC_2_19_016_type_id_extracted_verbatim_from_byte_0() {
        let type_ids: &[u8] = &[0x01, 0x2D, 0x64, 0xFF];
        for &tid in type_ids {
            let body = body_6(tid, 0x01, 0x06, 0x00, 0x01, 0x00);
            let asdu = parse_asdu(&body).unwrap_or_else(|| {
                panic!("6-byte body with type_id=0x{tid:02X} must return Some (BC-2.19.016 PC1)")
            });
            assert_eq!(
                asdu.type_id, tid,
                "type_id must equal asdu_body[0]=0x{tid:02X} verbatim (BC-2.19.016 postcondition 1)"
            );
        }
    }

    /// BC-2.19.016 canonical vector: TypeID=45 (C_SC_NA_1), VSQ=0x01 → type_id=45, sq=false, count=1.
    ///
    /// Canonical vector from BC-2.19.016 table row 2:
    ///   `asdu_body[0]=0x2D, asdu_body[1]=0x01` → `type_id=45, sq=false, count=1`.
    ///
    /// Traces: BC-2.19.016 postconditions 1–3; AC-169-002; EC-001 (BC-016).
    #[test]
    fn test_BC_2_19_016_type_id_45_c_sc_na_1_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data)
            .expect("BC-2.19.016 canonical vector [0x2D, 0x01, ...] must return Some");
        assert_eq!(
            asdu.type_id, 45,
            "type_id must be 45 (0x2D = C_SC_NA_1) (BC-2.19.016 canonical vector)"
        );
        assert!(
            !asdu.sq,
            "sq must be false for VSQ=0x01 (bit7=0) (BC-2.19.016 PC2)"
        );
        assert_eq!(
            asdu.count, 1,
            "count must be 1 for VSQ=0x01 (bits6:0=1) (BC-2.19.016 PC3)"
        );
    }

    /// BC-2.19.016 canonical vector: TypeID=0 (undefined per spec) passes through without rejection.
    ///
    /// Per BC-2.19.016 invariant 1: TypeID=0 is undefined; parse_asdu extracts it verbatim and
    /// returns Some — the caller (STORY-170 effectful shell) handles anomaly detection via
    /// BC-2.19.022. STORY-169 EC-007.
    ///
    /// Traces: BC-2.19.016 postcondition 1; invariant 1; AC-169-002; EC-002 (BC-016); EC-007 (STORY-169).
    #[test]
    fn test_BC_2_19_016_type_id_0_undefined_passthrough_canonical_vector() {
        let data: &[u8] = &[0x00, 0x01, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data).expect(
            "TypeID=0 must not be rejected by parse_asdu — caller handles anomaly \
             (BC-2.19.016 invariant 1; STORY-169 EC-007)",
        );
        assert_eq!(
            asdu.type_id, 0,
            "type_id must be 0 verbatim — undefined TypeID passthrough (BC-2.19.016 PC1; EC-007)"
        );
    }

    /// BC-2.19.016: VSQ=0x81 → sq=true, count=1.
    ///
    /// 0x81 = 0b10000001: bit7=1 (SQ=true), bits6:0=0x01 (count=1).
    ///
    /// Traces: BC-2.19.016 postconditions 2–3; AC-169-002.
    #[test]
    fn test_BC_2_19_016_vsq_0x81_sq_true_count_1() {
        let data: &[u8] = &[0x2D, 0x81, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data).expect("6-byte body with VSQ=0x81 must return Some");
        assert!(
            asdu.sq,
            "sq must be true for VSQ=0x81 (bit7=1) (BC-2.19.016 postcondition 2)"
        );
        assert_eq!(
            asdu.count, 1,
            "count must be 1 for VSQ=0x81 (bits6:0=1) (BC-2.19.016 postcondition 3)"
        );
    }

    /// BC-2.19.016: VSQ=0x03 → sq=false, count=3.
    ///
    /// 0x03 = 0b00000011: bit7=0 (SQ=false), bits6:0=0x03 (count=3).
    ///
    /// Traces: BC-2.19.016 postconditions 2–3; AC-169-002.
    #[test]
    fn test_BC_2_19_016_vsq_0x03_sq_false_count_3() {
        let data: &[u8] = &[0x2D, 0x03, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data).expect("6-byte body with VSQ=0x03 must return Some");
        assert!(
            !asdu.sq,
            "sq must be false for VSQ=0x03 (bit7=0) (BC-2.19.016 postcondition 2)"
        );
        assert_eq!(
            asdu.count, 3,
            "count must be 3 for VSQ=0x03 (bits6:0=3) (BC-2.19.016 postcondition 3)"
        );
    }

    /// BC-2.19.016 canonical vector: TypeID=255, VSQ=0x80 → type_id=255, sq=true, count=0.
    ///
    /// Canonical vector from BC-2.19.016 table row 3:
    ///   `asdu_body[0]=0xFF, asdu_body[1]=0x80` → `type_id=255, sq=true, count=0`.
    /// Also exercises EC-003 from BC-016 (sq=true, count=0 → no IOA iteration).
    /// Also exercises STORY-169 EC-005 (count=0 → first_ioa=None, covered by BC-2.19.018).
    ///
    /// Traces: BC-2.19.016 postconditions 1–4; AC-169-002; EC-003 (BC-016).
    #[test]
    fn test_BC_2_19_016_type_id_255_vsq_0x80_sq_true_count_0_canonical_vector() {
        let data: &[u8] = &[0xFF, 0x80, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data)
            .expect("BC-2.19.016 canonical vector [0xFF, 0x80, ...] must return Some");
        assert_eq!(
            asdu.type_id, 255,
            "type_id must be 255 (BC-2.19.016 canonical vector row 3)"
        );
        assert!(
            asdu.sq,
            "sq must be true for VSQ=0x80 (bit7=1) (BC-2.19.016 canonical vector)"
        );
        assert_eq!(
            asdu.count, 0,
            "count must be 0 for VSQ=0x80 (bits6:0=0) (BC-2.19.016 canonical vector; EC-003)"
        );
    }

    /// BC-2.19.016 invariant 3: VSQ=0x7F → sq=false, count=127 (maximum count value).
    ///
    /// 0x7F = 0b01111111: bit7=0 (SQ=false), bits6:0=0x7F (count=127).
    /// Verifies the 7-bit count field boundary.
    ///
    /// Traces: BC-2.19.016 postconditions 2–3; invariant 3 (count bound 0–127); AC-169-002.
    #[test]
    fn test_BC_2_19_016_vsq_0x7F_sq_false_count_127_max() {
        let data: &[u8] = &[0x01, 0x7F, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data).expect("6-byte body with VSQ=0x7F must return Some");
        assert!(
            !asdu.sq,
            "sq must be false for VSQ=0x7F (bit7=0) (BC-2.19.016 postcondition 2)"
        );
        assert_eq!(
            asdu.count, 127,
            "count must be 127 for VSQ=0x7F — max 7-bit count (BC-2.19.016 invariant 3)"
        );
    }

    // =========================================================================
    // BC-2.19.017: COT Extraction (Cause of Transmission) from ASDU Bytes 2–3
    // AC-169-003
    // =========================================================================

    /// BC-2.19.017 canonical vector: byte2=0x06, byte3=0x00 → cause=6, pn=false, test=false, orig=0.
    ///
    /// Canonical vector from BC-2.19.017 table row 1:
    ///   `byte[2]=0x06, byte[3]=0x00` → `cause=6, P/N=false, T=false, originator=0`.
    ///
    /// Traces: BC-2.19.017 postconditions 1–4; AC-169-003; EC-003 (BC-017).
    #[test]
    fn test_BC_2_19_017_cot_cause_6_activation_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data)
            .expect("BC-2.19.017 canonical vector [byte2=0x06, byte3=0x00] must return Some");
        assert_eq!(
            asdu.cot_cause, 6,
            "cot_cause must be 6 for byte2=0x06 (0x06 & 0x3F = 6) (BC-2.19.017 canonical vector)"
        );
        assert!(
            !asdu.cot_pn,
            "cot_pn must be false for byte2=0x06 (bit6=0) (BC-2.19.017 canonical vector)"
        );
        assert!(
            !asdu.cot_test,
            "cot_test must be false for byte2=0x06 (bit7=0) (BC-2.19.017 canonical vector)"
        );
        assert_eq!(
            asdu.cot_originator, 0,
            "cot_originator must be 0 for byte3=0x00 (BC-2.19.017 canonical vector)"
        );
    }

    /// BC-2.19.017: byte2=0x46 → cot_cause=6, cot_pn=true, cot_test=false.
    ///
    /// 0x46 = 0b01000110: bits5:0=6 (cause=6), bit6=1 (pn=true), bit7=0 (test=false).
    ///
    /// Traces: BC-2.19.017 postconditions 1–2; AC-169-003.
    #[test]
    fn test_BC_2_19_017_cot_pn_true_byte2_0x46_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x46, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data).expect("6-byte body with byte2=0x46 must return Some");
        assert_eq!(
            asdu.cot_cause, 6,
            "cot_cause must be 6 for byte2=0x46 (0x46 & 0x3F = 6) (BC-2.19.017 PC1)"
        );
        assert!(
            asdu.cot_pn,
            "cot_pn must be true for byte2=0x46 (bit6=1) (BC-2.19.017 postcondition 2)"
        );
        assert!(
            !asdu.cot_test,
            "cot_test must be false for byte2=0x46 (bit7=0) (BC-2.19.017 postcondition 3)"
        );
    }

    /// BC-2.19.017: byte2=0x86 → cot_cause=6, cot_pn=false, cot_test=true.
    ///
    /// 0x86 = 0b10000110: bits5:0=6 (cause=6), bit6=0 (pn=false), bit7=1 (test=true).
    /// STORY-169 EC-008: T-bit set; extract normally; caller may suppress or tag [TEST].
    ///
    /// Traces: BC-2.19.017 postconditions 1, 3; invariant 1; AC-169-003; EC-001 (BC-017); EC-008 (STORY-169).
    #[test]
    fn test_BC_2_19_017_cot_test_true_byte2_0x86_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x86, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data).expect("6-byte body with byte2=0x86 must return Some");
        assert_eq!(
            asdu.cot_cause, 6,
            "cot_cause must be 6 for byte2=0x86 (0x86 & 0x3F = 6) (BC-2.19.017 PC1)"
        );
        assert!(
            !asdu.cot_pn,
            "cot_pn must be false for byte2=0x86 (bit6=0) (BC-2.19.017 postcondition 2)"
        );
        assert!(
            asdu.cot_test,
            "cot_test must be true for byte2=0x86 (bit7=1) \
             (BC-2.19.017 postcondition 3; STORY-169 EC-008)"
        );
    }

    /// BC-2.19.017 canonical vector: byte2=0xC6, byte3=0x01 → cause=6, pn=true, test=true, orig=1.
    ///
    /// Canonical vector from BC-2.19.017 table row 2:
    ///   `byte[2]=0xC6, byte[3]=0x01` → `cause=6, P/N=true, T=true, originator=1`.
    /// 0xC6 = 0b11000110: bits5:0=6, bit6=1 (pn=true), bit7=1 (test=true).
    ///
    /// Traces: BC-2.19.017 postconditions 1–4; AC-169-003; EC-001 (BC-017).
    #[test]
    fn test_BC_2_19_017_cot_all_bits_byte2_0xC6_byte3_0x01_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0xC6, 0x01, 0x01, 0x00];
        let asdu = parse_asdu(data)
            .expect("BC-2.19.017 canonical vector [byte2=0xC6, byte3=0x01] must return Some");
        assert_eq!(
            asdu.cot_cause, 6,
            "cot_cause must be 6 (0xC6 & 0x3F = 6) (BC-2.19.017 canonical vector)"
        );
        assert!(
            asdu.cot_pn,
            "cot_pn must be true (0xC6 bit6=1) (BC-2.19.017 canonical vector)"
        );
        assert!(
            asdu.cot_test,
            "cot_test must be true (0xC6 bit7=1) (BC-2.19.017 canonical vector)"
        );
        assert_eq!(
            asdu.cot_originator, 1,
            "cot_originator must be 1 (byte3=0x01) (BC-2.19.017 canonical vector)"
        );
    }

    /// BC-2.19.017 canonical vector: byte2=0x3F, byte3=0xFF → cause=63, pn=false, test=false, orig=255.
    ///
    /// Canonical vector from BC-2.19.017 table row 3:
    ///   `byte[2]=0x3F, byte[3]=0xFF` → `cause=63, P/N=false, T=false, originator=255`.
    /// 0x3F = 0b00111111: bits5:0=63 (max cause), bit6=0, bit7=0.
    ///
    /// Traces: BC-2.19.017 postconditions 1–4; invariant 2 (max cause=63); AC-169-003.
    #[test]
    fn test_BC_2_19_017_cot_cause_max_63_byte2_0x3F_byte3_0xFF_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x3F, 0xFF, 0x01, 0x00];
        let asdu = parse_asdu(data)
            .expect("BC-2.19.017 canonical vector [byte2=0x3F, byte3=0xFF] must return Some");
        assert_eq!(
            asdu.cot_cause, 63,
            "cot_cause must be 63 (0x3F & 0x3F = 63) — max cause (BC-2.19.017 canonical vector)"
        );
        assert!(
            !asdu.cot_pn,
            "cot_pn must be false (0x3F bit6=0) (BC-2.19.017 canonical vector)"
        );
        assert!(
            !asdu.cot_test,
            "cot_test must be false (0x3F bit7=0) (BC-2.19.017 canonical vector)"
        );
        assert_eq!(
            asdu.cot_originator, 255,
            "cot_originator must be 255 (byte3=0xFF) (BC-2.19.017 canonical vector)"
        );
    }

    /// BC-2.19.017 postcondition 4: cot_originator equals asdu_body[3] verbatim.
    ///
    /// Exercises originator values 0x00 (no originator), 0x01, and 0xAB to verify
    /// verbatim extraction from byte index 3.
    ///
    /// Traces: BC-2.19.017 postcondition 4; AC-169-003; EC-002 (BC-017).
    #[test]
    fn test_BC_2_19_017_cot_originator_verbatim_from_byte_3() {
        let originator_values: &[u8] = &[0x00, 0x01, 0xAB];
        for &orig in originator_values {
            let body = body_6(0x2D, 0x01, 0x06, orig, 0x01, 0x00);
            let asdu = parse_asdu(&body).unwrap_or_else(|| {
                panic!(
                    "6-byte body with originator=0x{orig:02X} must return Some (BC-2.19.017 PC4)"
                )
            });
            assert_eq!(
                asdu.cot_originator, orig,
                "cot_originator must equal asdu_body[3]=0x{orig:02X} verbatim \
                 (BC-2.19.017 postcondition 4)"
            );
        }
    }

    // =========================================================================
    // BC-2.19.018: CASDU and First IOA Extraction from ASDU Bytes 4–8
    // AC-169-004, AC-169-005
    // =========================================================================

    /// BC-2.19.018 canonical vector: CASDU bytes [0x01, 0x00] → casdu=1.
    ///
    /// Canonical vector from BC-2.19.018 table row 1:
    ///   `asdu_body[4..6] = [0x01, 0x00]` → `casdu = 1`.
    ///
    /// Traces: BC-2.19.018 postcondition 1; AC-169-004; EC-001 (BC-018).
    #[test]
    fn test_BC_2_19_018_casdu_little_endian_1_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00];
        let asdu = parse_asdu(data)
            .expect("BC-2.19.018 canonical vector [CASDU=0x01,0x00] must return Some");
        assert_eq!(
            asdu.casdu, 1,
            "casdu must be 1 for LE bytes [0x01, 0x00] (BC-2.19.018 postcondition 1 canonical vector)"
        );
    }

    /// BC-2.19.018 canonical vector: CASDU bytes [0xFF, 0xFF] → casdu=65535 (maximum).
    ///
    /// Canonical vector from BC-2.19.018 table row 2:
    ///   `asdu_body[4..6] = [0xFF, 0xFF]` → `casdu = 65535`.
    /// Also exercises BC-2.19.018 EC-004: maximum CASDU value.
    ///
    /// Traces: BC-2.19.018 postcondition 1; AC-169-004; EC-004 (BC-018).
    #[test]
    fn test_BC_2_19_018_casdu_max_65535_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0xFF, 0xFF];
        let asdu = parse_asdu(data)
            .expect("BC-2.19.018 canonical vector [CASDU=0xFF,0xFF] must return Some");
        assert_eq!(
            asdu.casdu, 65535,
            "casdu must be 65535 for LE bytes [0xFF, 0xFF] (BC-2.19.018 EC-004 max CASDU)"
        );
    }

    /// BC-2.19.018 invariant 1: CASDU=0 is extracted without rejection.
    ///
    /// BC-2.19.018 invariant 1 states CASDU=0 is undefined per IEC 60870-5-104 but
    /// is extracted without rejection (anomaly flagging is out of MVP scope).
    ///
    /// Traces: BC-2.19.018 postcondition 1; invariant 1; AC-169-004; EC-003 (BC-018).
    #[test]
    fn test_BC_2_19_018_casdu_0_undefined_extracted_without_rejection() {
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x00, 0x00];
        let asdu =
            parse_asdu(data).expect("CASDU=0 must not cause rejection (BC-2.19.018 invariant 1)");
        assert_eq!(
            asdu.casdu, 0,
            "casdu must be 0 — undefined value extracted without rejection (BC-2.19.018 invariant 1)"
        );
    }

    /// BC-2.19.018 canonical vector: 9-byte body, count=1 → first_ioa=Some(1).
    ///
    /// Canonical vector from BC-2.19.018 table row 1:
    ///   `IOA = [0x01, 0x00, 0x00]` → `first_ioa = Some(1)`.
    /// STORY-169 EC-004: 9B, count>0 → first_ioa=Some.
    ///
    /// Traces: BC-2.19.018 postcondition 2; AC-169-005; EC-001 (BC-018); EC-004 (STORY-169).
    #[test]
    fn test_BC_2_19_018_first_ioa_some_count_1_len_9_canonical_vector() {
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00];
        let asdu = parse_asdu(data).expect("BC-2.19.018 canonical 9-byte vector must return Some");
        assert_eq!(
            asdu.first_ioa,
            Some(1),
            "first_ioa must be Some(1) for IOA=[0x01,0x00,0x00], count=1 \
             (BC-2.19.018 postcondition 2 canonical vector; STORY-169 EC-004)"
        );
    }

    /// BC-2.19.018 canonical vector: IOA=[0xFF,0xFF,0xFF] → first_ioa=Some(0xFFFFFF = 16777215).
    ///
    /// Canonical vector from BC-2.19.018 table row 2:
    ///   `IOA = [0xFF, 0xFF, 0xFF]` → `first_ioa = Some(16777215)` (max 24-bit value).
    /// STORY-169 EC-006: IOA max 0xFFFFFF.
    ///
    /// Traces: BC-2.19.018 postcondition 2; invariant 2 (max 24-bit IOA); AC-169-005;
    ///         EC-004 (BC-018); EC-006 (STORY-169).
    #[test]
    fn test_BC_2_19_018_first_ioa_max_0xFFFFFF_canonical_vector() {
        // Canonical: CASDU=[0xFF,0xFF], IOA=[0xFF,0xFF,0xFF]
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let asdu = parse_asdu(data).expect("BC-2.19.018 max-IOA canonical vector must return Some");
        assert_eq!(
            asdu.first_ioa,
            Some(0x00FF_FFFF),
            "first_ioa must be Some(0xFFFFFF = 16777215) for IOA=[0xFF,0xFF,0xFF] \
             (BC-2.19.018 invariant 2 max 24-bit; STORY-169 EC-006)"
        );
    }

    /// BC-2.19.018 postcondition 3: first_ioa=None when count==0 regardless of body length.
    ///
    /// STORY-169 EC-005: count==0 → first_ioa=None even with 9+ byte body.
    /// Tests VSQ=0x00 (sq=false, count=0) and VSQ=0x80 (sq=true, count=0).
    ///
    /// Traces: BC-2.19.018 postcondition 3; AC-169-005; EC-001 (BC-018); EC-005 (STORY-169).
    #[test]
    fn test_BC_2_19_018_first_ioa_none_when_count_0_regardless_of_length() {
        // EC-005 (STORY-169): VSQ=0x00 (count=0), 9-byte body → first_ioa=None
        let data_vsq_0x00: &[u8] = &[0x2D, 0x00, 0x06, 0x00, 0x01, 0x00, 0x01, 0x02, 0x03];
        let asdu_0x00 = parse_asdu(data_vsq_0x00)
            .expect("9-byte body with count=0 (VSQ=0x00) must return Some");
        assert_eq!(
            asdu_0x00.count, 0,
            "count must be 0 for VSQ=0x00 (precondition for EC-005)"
        );
        assert_eq!(
            asdu_0x00.first_ioa, None,
            "first_ioa must be None when count==0 (VSQ=0x00) regardless of body length \
             (BC-2.19.018 postcondition 3; STORY-169 EC-005)"
        );

        // EC-003 (BC-016) + EC-005 (STORY-169): VSQ=0x80 (sq=true, count=0), 9-byte body → None
        let data_vsq_0x80: &[u8] = &[0x2D, 0x80, 0x06, 0x00, 0x01, 0x00, 0x01, 0x02, 0x03];
        let asdu_0x80 = parse_asdu(data_vsq_0x80)
            .expect("9-byte body with count=0 (VSQ=0x80, sq=true) must return Some");
        assert_eq!(
            asdu_0x80.first_ioa, None,
            "first_ioa must be None when count==0 (VSQ=0x80, sq=true) regardless of length \
             (BC-2.19.018 postcondition 3; EC-005 STORY-169)"
        );
    }

    /// BC-2.19.018 postcondition 3: first_ioa=None when exactly 6 bytes and count>0.
    ///
    /// BC-2.19.018 EC-002: 6-byte body, count=1 → IOA would need bytes 6–8 but body ends
    /// at byte 5 → first_ioa=None (truncated, insufficient bytes for 3-byte IOA).
    /// STORY-169 EC-002: 6B → first_ioa=None.
    ///
    /// Traces: BC-2.19.018 postcondition 3; AC-169-005; EC-002 (BC-018 and STORY-169).
    #[test]
    fn test_BC_2_19_018_first_ioa_none_when_exactly_6_bytes_count_gt_0() {
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00];
        let asdu =
            parse_asdu(data).expect("6-byte body with count=1 must return Some (STORY-169 EC-002)");
        assert_eq!(
            asdu.count, 1,
            "count must be 1 (precondition: count>0 for this IOA-absence check)"
        );
        assert_eq!(
            asdu.first_ioa, None,
            "first_ioa must be None for 6-byte body with count=1 — no IOA bytes available \
             (BC-2.19.018 postcondition 3; EC-002 BC-018 and STORY-169)"
        );
    }

    /// BC-2.19.018 postcondition 3: first_ioa=None for 7-byte and 8-byte bodies with count>0.
    ///
    /// STORY-169 EC-003: 6–8 byte body, count>0 → first_ioa=None (insufficient for 3-byte IOA).
    /// A 3-byte IOA requires asdu_body.len() >= 9; 7 or 8 bytes are still insufficient.
    ///
    /// Traces: BC-2.19.018 postcondition 3; AC-169-005; EC-003 (STORY-169).
    #[test]
    fn test_BC_2_19_018_first_ioa_none_when_7_or_8_bytes_count_gt_0() {
        // 7-byte body, count=1 → first_ioa=None (STORY-169 EC-003)
        let data_7: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, 0xAA];
        let asdu_7 = parse_asdu(data_7)
            .expect("7-byte body with count=1 must return Some (STORY-169 EC-003)");
        assert_eq!(
            asdu_7.first_ioa, None,
            "first_ioa must be None for 7-byte body with count=1 — still insufficient for 3-byte IOA \
             (BC-2.19.018 postcondition 3; STORY-169 EC-003)"
        );

        // 8-byte body, count=1 → first_ioa=None (STORY-169 EC-003)
        let data_8: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, 0xAA, 0xBB];
        let asdu_8 = parse_asdu(data_8)
            .expect("8-byte body with count=1 must return Some (STORY-169 EC-003)");
        assert_eq!(
            asdu_8.first_ioa, None,
            "first_ioa must be None for 8-byte body with count=1 — still insufficient for 3-byte IOA \
             (BC-2.19.018 postcondition 3; STORY-169 EC-003)"
        );
    }

    /// BC-2.19.018 postcondition 2: first_ioa is 24-bit LE zero-extended, verified with
    /// a non-trivial multi-byte IOA value.
    ///
    /// IOA=[0x34,0x12,0x00] → first_ioa=Some(0x001234=4660) verifies correct LE byte order
    /// (if bytes were reversed it would be Some(0x123400=1193984), catching a BE bug).
    ///
    /// Traces: BC-2.19.018 postcondition 2; invariant 2; AC-169-005.
    #[test]
    fn test_BC_2_19_018_first_ioa_le_byte_order_verified() {
        // IOA=[0x34, 0x12, 0x00] → 0x00001234 = 4660 in LE (would be 0x123400 in BE — catches swap)
        let data: &[u8] = &[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, 0x34, 0x12, 0x00];
        let asdu =
            parse_asdu(data).expect("9-byte body with IOA=[0x34,0x12,0x00] must return Some");
        assert_eq!(
            asdu.first_ioa,
            Some(0x0000_1234),
            "first_ioa must be Some(0x1234=4660) for IOA=[0x34,0x12,0x00] — \
             LE byte order: b0 is LSB (BC-2.19.018 postcondition 2; invariant 2)"
        );
    }
}

// =============================================================================
// STORY-170: IEC-104 Control Command Detection
//
// Covers BC-2.19.019–022 and BC-2.19.017 [TEST] tagging.
// All tests in this module MUST FAIL (Red Gate) because detect_iec104_threats
// is a todo!() stub. They pass after the implementer delivers the TypeID dispatch.
//
// ## Contract coverage
// - BC-2.19.019 (AC-170-001): TypeIDs 45–47 → T1692.001 Possible only (1 finding);
//                              TypeIDs 48–51 → T1692.001 + T0836 Possible (2 findings).
// - BC-2.19.020 (AC-170-002): TypeID 105 → T0827 Likely (1 finding; NOT Possible).
// - BC-2.19.021 (AC-170-003): TypeIDs 100, 101, 103 → no finding (benign admin commands).
// - BC-2.19.022 (AC-170-004): TypeID=0 or [128,255] → T0814 Possible Anomaly.
//   (AC-170-005): TypeIDs in defined-but-unhandled [1,127] → no finding.
// - BC-2.19.017 (AC-170-007): cot_test=true appends " [TEST]" to every finding summary.
//
// ## Canonical test vectors (DF-CANONICAL-FRAME-HOLDOUT-001)
// Used verbatim from BC-2.19.019–022 and BC-2.19.017 tables.
//
// ## Provenance
// Written Red-first as TDD stubs (STORY-170 strict TDD mode); GREEN after implementer
// delivers detect_iec104_threats TypeID dispatch table.
// =============================================================================
mod story_170 {
    use wirerust::analyzer::iec104::{Asdu, detect_iec104_threats};
    use wirerust::findings::{ThreatCategory, Verdict};

    // -------------------------------------------------------------------------
    // Test helper: build a minimal Asdu for detect_iec104_threats tests.
    //
    // All fields irrelevant to TypeID dispatch are set to sensible defaults.
    // `type_id` and `cot_test` are the two fields consumed by detect_iec104_threats.
    // -------------------------------------------------------------------------

    /// Construct a minimal Asdu with a given TypeID and cot_test flag.
    ///
    /// Fields not relevant to STORY-170 dispatch logic are set to typical defaults:
    /// - sq=false, count=1, cot_cause=6 (activation), cot_pn=false, cot_originator=0
    /// - casdu=1 (RTU address 1), first_ioa=None
    fn make_asdu(type_id: u8, cot_test: bool) -> Asdu {
        Asdu {
            type_id,
            sq: false,
            count: 1,
            cot_cause: 6,
            cot_pn: false,
            cot_test,
            cot_originator: 0,
            casdu: 1,
            first_ioa: None,
        }
    }

    // =========================================================================
    // BC-2.19.019: Control Command TypeIDs 45–51 Emit T1692.001;
    //              Set-Point + Bitstring TypeIDs 48–51 Also Emit T0836
    // AC-170-001
    // =========================================================================

    /// BC-2.19.019 canonical vector: TypeID=45 (C_SC_NA_1) → exactly 1 finding.
    ///
    /// Canonical vector from BC-2.19.019 table: TypeID=45, CASDU=1 → T1692.001 Possible only.
    /// Switching command: T1692.001 only; no T0836 (binary control, not parameter write).
    ///
    /// Traces: BC-2.19.019 postcondition 1; invariant 1; AC-170-001; EC-001 (BC-019); EC-003 (STORY-170).
    #[test]
    fn test_BC_2_19_019_type45_c_sc_na1_emits_exactly_one_finding() {
        let asdu = make_asdu(45, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=45 (C_SC_NA_1) must emit exactly 1 finding — T1692.001 only, no T0836 \
             (BC-2.19.019 postconditions 1–2; invariant 2)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=45 emits T1692.001 Possible with Impact category.
    ///
    /// Traces: BC-2.19.019 postcondition 1; AC-170-001; EC-001 (BC-019); EC-003 (STORY-170).
    #[test]
    fn test_BC_2_19_019_type45_emits_t1692001_possible_impact() {
        let asdu = make_asdu(45, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        let f = findings
            .first()
            .expect("TypeID=45 must emit at least one finding (BC-2.19.019 postcondition 1)");
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T1692.001"),
            "TypeID=45 finding must contain \"T1692.001\" in mitre_techniques \
             (BC-2.19.019 postcondition 1)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "TypeID=45 T1692.001 finding must have Verdict::Possible (BC-2.19.019 postcondition 1)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Impact,
            "TypeID=45 T1692.001 finding must have category Impact (BC-2.19.019; ICS command)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=45 does NOT emit T0836.
    ///
    /// Invariant 2: switching commands 45–47 emit T1692.001 only — no T0836.
    ///
    /// Traces: BC-2.19.019 invariant 2; AC-170-001 (negative); EC-003 (STORY-170).
    #[test]
    fn test_BC_2_19_019_type45_does_not_emit_t0836() {
        let asdu = make_asdu(45, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            !findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
            "TypeID=45 (switching command) must NOT emit T0836 — T0836 is for set-point/bitstring \
             only (BC-2.19.019 invariant 2)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=46 (C_DC_NA_1) → exactly 1 finding (T1692.001 only).
    ///
    /// Canonical vector from BC-2.19.019 table: TypeID=46 → T1692.001 Possible only.
    ///
    /// Traces: BC-2.19.019 postcondition 1; invariant 2; AC-170-001.
    #[test]
    fn test_BC_2_19_019_type46_c_dc_na1_emits_t1692001_only() {
        let asdu = make_asdu(46, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=46 (C_DC_NA_1) must emit exactly 1 finding — T1692.001 only \
             (BC-2.19.019 invariant 2)"
        );
        assert!(
            findings[0]
                .mitre_techniques
                .iter()
                .any(|t| t == "T1692.001"),
            "TypeID=46 finding must be T1692.001 (BC-2.19.019 postcondition 1)"
        );
        assert!(
            !findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
            "TypeID=46 must NOT emit T0836 (BC-2.19.019 invariant 2)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=47 (C_RC_NA_1) → exactly 1 finding (T1692.001 only).
    ///
    /// EC-007 from BC-2.19.019: TypeID=47 (C_RC_NA_1, regulating step) → T1692.001 Possible only.
    ///
    /// Traces: BC-2.19.019 postcondition 1; invariant 2; AC-170-001; EC-007 (BC-019).
    #[test]
    fn test_BC_2_19_019_type47_c_rc_na1_emits_t1692001_only() {
        let asdu = make_asdu(47, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=47 (C_RC_NA_1, regulating step) must emit exactly 1 finding \
             (BC-2.19.019 EC-007)"
        );
        assert!(
            findings[0]
                .mitre_techniques
                .iter()
                .any(|t| t == "T1692.001"),
            "TypeID=47 finding must be T1692.001 (BC-2.19.019 postcondition 1)"
        );
        assert!(
            !findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
            "TypeID=47 must NOT emit T0836 (BC-2.19.019 EC-007; invariant 2)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=48 (C_SE_NA_1) → exactly 2 findings.
    ///
    /// Set-point normalized value: T1692.001 + T0836, both Possible.
    /// EC-006 from BC-2.19.019: TypeID=48 → T1692.001 Possible + T0836 Possible.
    ///
    /// Traces: BC-2.19.019 postconditions 1–2; AC-170-001; EC-006 (BC-019).
    #[test]
    fn test_BC_2_19_019_type48_c_se_na1_emits_exactly_two_findings() {
        let asdu = make_asdu(48, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            2,
            "TypeID=48 (C_SE_NA_1, set-point normalized) must emit exactly 2 findings: \
             T1692.001 + T0836 (BC-2.19.019 postconditions 1–2; EC-006)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=48 emits T1692.001 Possible.
    ///
    /// Traces: BC-2.19.019 postcondition 1; AC-170-001; EC-006 (BC-019).
    #[test]
    fn test_BC_2_19_019_type48_emits_t1692001_possible() {
        let asdu = make_asdu(48, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        let has_t1692 = findings
            .iter()
            .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"));
        assert!(
            has_t1692,
            "TypeID=48 must emit a T1692.001 finding (BC-2.19.019 postcondition 1)"
        );
        let t1692_finding = findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"))
            .unwrap();
        assert_eq!(
            t1692_finding.verdict,
            Verdict::Possible,
            "TypeID=48 T1692.001 finding must have Verdict::Possible (BC-2.19.019 postcondition 1)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=48 emits T0836 Possible.
    ///
    /// Traces: BC-2.19.019 postcondition 2; AC-170-001; EC-006 (BC-019).
    #[test]
    fn test_BC_2_19_019_type48_emits_t0836_possible() {
        let asdu = make_asdu(48, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        let has_t0836 = findings
            .iter()
            .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836"));
        assert!(
            has_t0836,
            "TypeID=48 (C_SE_NA_1) must emit a T0836 finding (BC-2.19.019 postcondition 2)"
        );
        let t0836_finding = findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0836"))
            .unwrap();
        assert_eq!(
            t0836_finding.verdict,
            Verdict::Possible,
            "TypeID=48 T0836 finding must have Verdict::Possible (BC-2.19.019 postcondition 2)"
        );
    }

    /// BC-2.19.019: TypeID=49 (C_SE_NB_1) → exactly 2 findings (T1692.001 + T0836).
    ///
    /// Traces: BC-2.19.019 postconditions 1–2; AC-170-001.
    #[test]
    fn test_BC_2_19_019_type49_c_se_nb1_emits_t1692001_and_t0836() {
        let asdu = make_asdu(49, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            2,
            "TypeID=49 (C_SE_NB_1) must emit exactly 2 findings (BC-2.19.019 postconditions 1–2)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
            "TypeID=49 must emit T1692.001 (BC-2.19.019 postcondition 1)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
            "TypeID=49 must emit T0836 (BC-2.19.019 postcondition 2)"
        );
    }

    /// BC-2.19.019: TypeID=50 (C_SE_NC_1) → exactly 2 findings (T1692.001 + T0836).
    ///
    /// Traces: BC-2.19.019 postconditions 1–2; AC-170-001.
    #[test]
    fn test_BC_2_19_019_type50_c_se_nc1_emits_t1692001_and_t0836() {
        let asdu = make_asdu(50, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            2,
            "TypeID=50 (C_SE_NC_1) must emit exactly 2 findings (BC-2.19.019 postconditions 1–2)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
            "TypeID=50 must emit T1692.001 (BC-2.19.019 postcondition 1)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
            "TypeID=50 must emit T0836 (BC-2.19.019 postcondition 2)"
        );
    }

    /// BC-2.19.019 canonical vector: TypeID=51 (C_BO_NA_1) → exactly 2 findings.
    ///
    /// Canonical vector from BC-2.19.019 table row 4: TypeID=51, CASDU=100 → T1692.001 + T0836.
    /// Bitstring write — same as set-point regarding T0836 emission.
    /// EC-004 (STORY-170 edge case): TypeID=51 max control — bitstring.
    ///
    /// Traces: BC-2.19.019 postconditions 1–2; AC-170-001; EC-002 (BC-019); EC-004 (STORY-170).
    #[test]
    fn test_BC_2_19_019_type51_c_bo_na1_emits_exactly_two_findings_canonical_vector() {
        let asdu = Asdu {
            type_id: 51,
            sq: false,
            count: 1,
            cot_cause: 6,
            cot_pn: false,
            cot_test: false,
            cot_originator: 0,
            casdu: 100, // canonical vector CASDU=100
            first_ioa: None,
        };
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            2,
            "TypeID=51 (C_BO_NA_1, bitstring) must emit exactly 2 findings: T1692.001 + T0836 \
             (BC-2.19.019 canonical vector row 4; EC-004 STORY-170)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
            "TypeID=51 must emit T1692.001 (BC-2.19.019 postcondition 1)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
            "TypeID=51 must emit T0836 (BC-2.19.019 postcondition 2; EC-004 STORY-170)"
        );
    }

    /// BC-2.19.019 invariant: switching TypeIDs 45–47 each emit exactly 1 finding (T1692.001).
    ///
    /// Parameterized over all three switching command TypeIDs.
    ///
    /// Traces: BC-2.19.019 postcondition 1; invariant 2; AC-170-001.
    #[test]
    fn test_BC_2_19_019_invariant_switching_types_45_to_47_each_emit_one_finding() {
        for type_id in [45u8, 46, 47] {
            let asdu = make_asdu(type_id, false);
            let mut findings = Vec::new();
            detect_iec104_threats(&asdu, &mut findings);
            assert_eq!(
                findings.len(),
                1,
                "TypeID={type_id} (switching command) must emit exactly 1 finding \
                 (BC-2.19.019 postconditions 1–2; invariant 2)"
            );
            assert!(
                findings[0]
                    .mitre_techniques
                    .iter()
                    .any(|t| t == "T1692.001"),
                "TypeID={type_id} sole finding must be T1692.001 (BC-2.19.019 postcondition 1)"
            );
            assert!(
                !findings
                    .iter()
                    .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
                "TypeID={type_id} must NOT emit T0836 (BC-2.19.019 invariant 2)"
            );
        }
    }

    /// BC-2.19.019 invariant: set-point/bitstring TypeIDs 48–51 each emit exactly 2 findings.
    ///
    /// Parameterized over all four set-point/bitstring TypeIDs.
    ///
    /// Traces: BC-2.19.019 postconditions 1–2; AC-170-001.
    #[test]
    fn test_BC_2_19_019_invariant_setpoint_types_48_to_51_each_emit_two_findings() {
        for type_id in [48u8, 49, 50, 51] {
            let asdu = make_asdu(type_id, false);
            let mut findings = Vec::new();
            detect_iec104_threats(&asdu, &mut findings);
            assert_eq!(
                findings.len(),
                2,
                "TypeID={type_id} (set-point/bitstring) must emit exactly 2 findings: \
                 T1692.001 + T0836 (BC-2.19.019 postconditions 1–2)"
            );
            assert!(
                findings
                    .iter()
                    .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
                "TypeID={type_id} must contain T1692.001 (BC-2.19.019 postcondition 1)"
            );
            assert!(
                findings
                    .iter()
                    .any(|f| f.mitre_techniques.iter().any(|t| t == "T0836")),
                "TypeID={type_id} must contain T0836 (BC-2.19.019 postcondition 2)"
            );
        }
    }

    /// BC-2.19.019 invariant: T1692.001 is present for every control TypeID 45–51.
    ///
    /// Invariant 1: T1692.001 is emitted for ALL TypeIDs in 45–51 without exception.
    ///
    /// Traces: BC-2.19.019 invariant 1; AC-170-001.
    #[test]
    fn test_BC_2_19_019_invariant_t1692001_present_for_all_types_45_to_51() {
        for type_id in 45u8..=51 {
            let asdu = make_asdu(type_id, false);
            let mut findings = Vec::new();
            detect_iec104_threats(&asdu, &mut findings);
            assert!(
                findings
                    .iter()
                    .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
                "TypeID={type_id} must emit T1692.001 — invariant 1 holds for all 45–51 \
                 (BC-2.19.019 invariant 1)"
            );
        }
    }

    /// BC-2.19.019 invariant: all T1692.001 and T0836 findings have Verdict::Possible.
    ///
    /// Verifies that no finding from TypeIDs 45–51 is Likely (not an escalation scenario).
    ///
    /// Traces: BC-2.19.019 postconditions 1–2; AC-170-001.
    #[test]
    fn test_BC_2_19_019_invariant_all_findings_have_possible_verdict() {
        for type_id in 45u8..=51 {
            let asdu = make_asdu(type_id, false);
            let mut findings = Vec::new();
            detect_iec104_threats(&asdu, &mut findings);
            for f in &findings {
                assert_eq!(
                    f.verdict,
                    Verdict::Possible,
                    "TypeID={type_id} finding verdict must be Possible — not Likely \
                     (BC-2.19.019 postconditions 1–2)"
                );
            }
        }
    }

    // =========================================================================
    // BC-2.19.020: C_RP_NA_1 (TypeID 105) Emits T0827 "Loss of Control" Finding
    // AC-170-002
    // =========================================================================

    /// BC-2.19.020 canonical vector: TypeID=105 → exactly 1 finding.
    ///
    /// EC-006 (STORY-170): TypeID=105 specifically → T0827 Likely.
    ///
    /// Traces: BC-2.19.020 postconditions 1–2; AC-170-002; EC-006 (STORY-170).
    #[test]
    fn test_BC_2_19_020_type105_c_rp_na1_emits_exactly_one_finding() {
        let asdu = make_asdu(105, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=105 (C_RP_NA_1) must emit exactly 1 finding: T0827 only \
             (BC-2.19.020 postcondition 1; invariant 1)"
        );
    }

    /// BC-2.19.020 canonical vector: TypeID=105 emits T0827 Likely.
    ///
    /// Canonical vector from BC-2.19.020: TypeID=105 → T0827 Likely.
    /// Critical: verdict is Likely, NOT Possible (AC-170-002 traces BC-2.19.020 v1.1).
    ///
    /// Traces: BC-2.19.020 postcondition 1; AC-170-002; EC-006 (STORY-170).
    #[test]
    fn test_BC_2_19_020_type105_emits_t0827_likely_canonical_vector() {
        let asdu = make_asdu(105, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        let f = findings
            .first()
            .expect("TypeID=105 must emit a T0827 finding (BC-2.19.020 postcondition 1)");
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0827"),
            "TypeID=105 finding must contain \"T0827\" in mitre_techniques \
             (BC-2.19.020 canonical vector)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "TypeID=105 T0827 must have Verdict::Likely — NOT Possible \
             (BC-2.19.020 postcondition 1; v1.1 correction)"
        );
    }

    /// BC-2.19.020 negative: TypeID=105 verdict is Likely, not Possible.
    ///
    /// Explicitly guards against the v1.0 bug where T0827 was emitted as Possible.
    /// The v1.1 correction changed this to Likely (BC-2.19.020 modified note).
    ///
    /// Traces: BC-2.19.020 postcondition 1; AC-170-002 (v1.1 confidence correction).
    #[test]
    fn test_BC_2_19_020_type105_verdict_is_likely_not_possible() {
        let asdu = make_asdu(105, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        let f = findings
            .first()
            .expect("TypeID=105 must emit a finding (BC-2.19.020 postcondition 1)");
        assert_ne!(
            f.verdict,
            Verdict::Possible,
            "TypeID=105 T0827 must NOT be Possible — it must be Likely \
             (BC-2.19.020 v1.1 correction; AC-170-002)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "TypeID=105 T0827 must be Verdict::Likely (BC-2.19.020 postcondition 1)"
        );
    }

    /// BC-2.19.020 invariant 1: TypeID=105 does NOT emit T1692.001.
    ///
    /// Reset Process (C_RP) is a session management command, not a parameter change.
    /// Invariant 1: only T0827 is emitted for TypeID 105.
    ///
    /// Traces: BC-2.19.020 invariant 1; AC-170-002 (negative).
    #[test]
    fn test_BC_2_19_020_type105_does_not_emit_t1692001() {
        let asdu = make_asdu(105, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            !findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
            "TypeID=105 must NOT emit T1692.001 — only T0827 \
             (BC-2.19.020 invariant 1: reset is session management, not parameter change)"
        );
    }

    /// BC-2.19.020: TypeID=105 T0827 finding has Impact category.
    ///
    /// "Loss of Control" is an Impact-class technique at ICS level.
    ///
    /// Traces: BC-2.19.020 postcondition 1; AC-170-002.
    #[test]
    fn test_BC_2_19_020_type105_category_is_impact() {
        let asdu = make_asdu(105, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        let f = findings
            .first()
            .expect("TypeID=105 must emit a finding (BC-2.19.020)");
        assert_eq!(
            f.category,
            ThreatCategory::Impact,
            "TypeID=105 T0827 finding must have ThreatCategory::Impact (BC-2.19.020)"
        );
    }

    // =========================================================================
    // BC-2.19.021: Interrogation and Clock-Sync Commands (TypeIDs 100, 101, 103)
    //              Are Logged Without Findings
    // AC-170-003
    // =========================================================================

    /// BC-2.19.021 canonical vector: TypeID=100 (C_IC_NA_1) → no finding emitted.
    ///
    /// Canonical vector from BC-2.19.021 table: TypeID=100 → none.
    /// General Interrogation is benign: no security finding.
    ///
    /// Traces: BC-2.19.021 postcondition 1; invariant 1; AC-170-003; EC-001 (BC-021).
    #[test]
    fn test_BC_2_19_021_type100_c_ic_emits_no_finding_canonical_vector() {
        let asdu = make_asdu(100, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=100 (C_IC_NA_1 general interrogation) must produce no finding \
             (BC-2.19.021 postcondition 1; canonical vector)"
        );
    }

    /// BC-2.19.021 canonical vector: TypeID=101 (C_CI_NA_1) → no finding emitted.
    ///
    /// Canonical vector from BC-2.19.021 table: TypeID=101 → none.
    /// Counter Interrogation is benign: no security finding.
    ///
    /// Traces: BC-2.19.021 postcondition 1; invariant 1; AC-170-003; EC-002 (BC-021).
    #[test]
    fn test_BC_2_19_021_type101_c_ci_emits_no_finding_canonical_vector() {
        let asdu = make_asdu(101, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=101 (C_CI_NA_1 counter interrogation) must produce no finding \
             (BC-2.19.021 postcondition 1; canonical vector)"
        );
    }

    /// BC-2.19.021 canonical vector: TypeID=103 (C_CS_NA_1) → no finding emitted.
    ///
    /// Canonical vector from BC-2.19.021 table: TypeID=103 → none.
    /// Clock Synchronization is benign: no security finding.
    ///
    /// Traces: BC-2.19.021 postcondition 1; invariant 1; AC-170-003; EC-003 (BC-021).
    #[test]
    fn test_BC_2_19_021_type103_c_cs_emits_no_finding_canonical_vector() {
        let asdu = make_asdu(103, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=103 (C_CS_NA_1 clock sync) must produce no finding \
             (BC-2.19.021 postcondition 1; canonical vector)"
        );
    }

    /// BC-2.19.021 invariant: all three interrogation/clock-sync TypeIDs produce empty findings.
    ///
    /// Parameterized over {100, 101, 103} to verify no-finding invariant holds for all three.
    /// Critical: this tests the false-positive-prevention that was fixed in STORY-170 v2.0.
    /// The old (buggy) spec erroneously emitted T0827 for these TypeIDs.
    ///
    /// Traces: BC-2.19.021 postconditions 1–3; invariant 1; AC-170-003.
    #[test]
    fn test_BC_2_19_021_invariant_all_interrogation_types_emit_no_finding() {
        for type_id in [100u8, 101, 103] {
            let asdu = make_asdu(type_id, false);
            let mut findings = Vec::new();
            detect_iec104_threats(&asdu, &mut findings);
            assert!(
                findings.is_empty(),
                "TypeID={type_id} (interrogation/clock-sync) must emit no finding — \
                 benign administrative command (BC-2.19.021 postcondition 1; invariant 1)"
            );
        }
    }

    // =========================================================================
    // BC-2.19.022: Reserved or Invalid TypeID Emits T0814 Anomaly
    // AC-170-004 (TypeID=0 or [128,255]) and AC-170-005 ([1,127] defined unhandled)
    // =========================================================================

    /// BC-2.19.022 canonical vector: TypeID=0 → T0814 Possible Anomaly.
    ///
    /// Canonical vector from BC-2.19.022 table row 1: TypeID=0 → T0814 Possible.
    /// TypeID=0 is undefined by IEC 60870-5-104. EC-001 (STORY-170) and EC-001 (BC-022).
    ///
    /// Traces: BC-2.19.022 precondition 2; postcondition 1; invariant 1; AC-170-004;
    ///         EC-001 (BC-022); EC-001 (STORY-170).
    #[test]
    fn test_BC_2_19_022_type0_undefined_emits_t0814_anomaly_canonical_vector() {
        let asdu = make_asdu(0, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=0 must emit exactly 1 finding: T0814 Anomaly \
             (BC-2.19.022 canonical vector; EC-001 BC-022 and STORY-170)"
        );
        let f = &findings[0];
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "TypeID=0 finding must contain \"T0814\" in mitre_techniques \
             (BC-2.19.022 postcondition 1)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "TypeID=0 T0814 must have Verdict::Possible (BC-2.19.022 postcondition 1)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "TypeID=0 T0814 must have ThreatCategory::Anomaly (BC-2.19.022 title: 'Anomaly')"
        );
    }

    /// BC-2.19.022 canonical vector: TypeID=128 → T0814 Possible.
    ///
    /// Canonical vector from BC-2.19.022 table row 2: TypeID=128 → T0814 Possible.
    /// 128 is the start of the private-use/reserved range.
    ///
    /// Traces: BC-2.19.022 precondition 2; postcondition 1; AC-170-004; EC-002 (BC-022).
    #[test]
    fn test_BC_2_19_022_type128_emits_t0814_possible_canonical_vector() {
        let asdu = make_asdu(128, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=128 must emit exactly 1 finding: T0814 Possible \
             (BC-2.19.022 canonical vector EC-002)"
        );
        assert!(
            findings[0].mitre_techniques.iter().any(|t| t == "T0814"),
            "TypeID=128 must emit T0814 (BC-2.19.022 canonical vector)"
        );
        assert_eq!(
            findings[0].verdict,
            Verdict::Possible,
            "TypeID=128 T0814 must be Possible (BC-2.19.022 postcondition 1)"
        );
        assert_eq!(
            findings[0].category,
            ThreatCategory::Anomaly,
            "TypeID=128 T0814 must be Anomaly (BC-2.19.022)"
        );
    }

    /// BC-2.19.022 canonical vector: TypeID=255 → T0814 Possible.
    ///
    /// Canonical vector from BC-2.19.022 table row 3: TypeID=255 → T0814 Possible.
    /// Maximum TypeID value — private-use/reserved range.
    /// EC-008 (STORY-170): TypeID=255 (max, private-use/reserved) → T0814 Possible.
    ///
    /// Traces: BC-2.19.022 postcondition 1; AC-170-004; EC-003 (BC-022); EC-008 (STORY-170).
    #[test]
    fn test_BC_2_19_022_type255_emits_t0814_possible_canonical_vector() {
        let asdu = make_asdu(255, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=255 must emit exactly 1 finding: T0814 Possible \
             (BC-2.19.022 canonical vector EC-003; EC-008 STORY-170)"
        );
        assert!(
            findings[0].mitre_techniques.iter().any(|t| t == "T0814"),
            "TypeID=255 must emit T0814 (BC-2.19.022 canonical vector)"
        );
        assert_eq!(
            findings[0].verdict,
            Verdict::Possible,
            "TypeID=255 T0814 must be Possible (BC-2.19.022 postcondition 1)"
        );
    }

    /// BC-2.19.022: TypeID=200 (mid-range private-use) → T0814 Possible.
    ///
    /// Additional reserved TypeID in the middle of the 128–255 range.
    ///
    /// Traces: BC-2.19.022 postcondition 1; AC-170-004.
    #[test]
    fn test_BC_2_19_022_type200_private_use_emits_t0814_possible() {
        let asdu = make_asdu(200, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "TypeID=200 (private-use range) must emit 1 finding: T0814 Possible \
             (BC-2.19.022 postcondition 1)"
        );
        assert!(
            findings[0].mitre_techniques.iter().any(|t| t == "T0814"),
            "TypeID=200 must emit T0814 (BC-2.19.022 postcondition 1)"
        );
    }

    /// BC-2.19.022 canonical vector: TypeID=44 (max monitoring, defined-but-unhandled) → no finding.
    ///
    /// Canonical vector from BC-2.19.022 table row 4: TypeID=44 → no finding (silently logged).
    /// EC-002 (STORY-170): TypeID=44 (max monitoring, defined-but-unhandled) → no finding.
    /// TypeID=44 is defined by IEC 60870-5-104 as a monitoring-direction TypeID.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005; EC-004 (BC-022); EC-002 (STORY-170).
    #[test]
    fn test_BC_2_19_022_type44_max_monitoring_emits_no_finding_canonical_vector() {
        let asdu = make_asdu(44, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=44 (max monitoring TypeID, defined-but-unhandled) must produce no finding — \
             silently logged (BC-2.19.022 canonical vector EC-004; EC-002 STORY-170)"
        );
    }

    /// BC-2.19.022: TypeID=52 (RESERVED per IEC 60870-5-104, above control range) → no finding.
    ///
    /// TypeID=52 is RESERVED in IEC 60870-5-104; it is NOT in the control-command range [45–51].
    /// Per BC-2.19.022 invariant 1: defined-but-unhandled TypeIDs in [1,127] → silently logged.
    /// EC-005 (STORY-170): TypeID=52 → no finding.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005; EC-004 (BC-019); EC-005 (STORY-170).
    #[test]
    fn test_BC_2_19_022_type52_reserved_above_control_range_emits_no_finding() {
        let asdu = make_asdu(52, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=52 (RESERVED, above control range 45–51) must produce no finding — \
             silently logged per BC-2.19.022 invariant 1; EC-005 STORY-170"
        );
    }

    /// BC-2.19.022: TypeID=99 (defined range above control, below interrogation) → no finding.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005.
    #[test]
    fn test_BC_2_19_022_type99_defined_unhandled_emits_no_finding() {
        let asdu = make_asdu(99, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=99 (defined-but-unhandled) must produce no finding \
             (BC-2.19.022 invariant 1; AC-170-005)"
        );
    }

    /// BC-2.19.022: TypeID=102 (C_RD_NA_1 Read Command, defined but not in any detection set) → no finding.
    ///
    /// EC-007 (STORY-170): TypeID=102 (C_RD_NA_1) → no finding; silently logged.
    /// Also matches BC-2.19.021 invariant 2: TypeID 102 is explicitly NOT in {100,101,103}.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005; EC-004 (BC-021); EC-007 (STORY-170).
    #[test]
    fn test_BC_2_19_022_type102_c_rd_not_in_detection_set_emits_no_finding() {
        let asdu = make_asdu(102, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=102 (C_RD_NA_1, defined but not in detection set) must produce no finding — \
             silently logged (BC-2.19.022 invariant 1; EC-007 STORY-170)"
        );
    }

    /// BC-2.19.022: TypeID=104 (defined range, between C_RP and C_IC) → no finding.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005.
    #[test]
    fn test_BC_2_19_022_type104_defined_unhandled_emits_no_finding() {
        let asdu = make_asdu(104, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=104 (defined-but-unhandled, between C_RP and C_IC) must produce no finding \
             (BC-2.19.022 invariant 1; AC-170-005)"
        );
    }

    /// BC-2.19.022: TypeID=127 (max defined-but-unhandled in [1,127]) → no finding.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005.
    #[test]
    fn test_BC_2_19_022_type127_max_defined_unhandled_emits_no_finding() {
        let asdu = make_asdu(127, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=127 (maximum defined-but-unhandled value in [1,127]) must produce no finding \
             (BC-2.19.022 invariant 1; AC-170-005)"
        );
    }

    /// BC-2.19.022: TypeID=1 (minimum defined-but-unhandled monitoring TypeID) → no finding.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005.
    #[test]
    fn test_BC_2_19_022_type1_minimum_defined_unhandled_emits_no_finding() {
        let asdu = make_asdu(1, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            findings.is_empty(),
            "TypeID=1 (minimum defined-but-unhandled monitoring TypeID) must produce no finding \
             (BC-2.19.022 invariant 1; AC-170-005)"
        );
    }

    /// BC-2.19.022 invariant: a representative sample of defined-but-unhandled TypeIDs emit no finding.
    ///
    /// Tests {1, 30, 44, 52, 99, 102, 104, 127} — all silently logged with no findings.
    /// Ensures the silent-log path is exhaustive for a cross-section of the [1,127] range.
    ///
    /// Traces: BC-2.19.022 invariant 1; AC-170-005; AC-170-006.
    #[test]
    fn test_BC_2_19_022_invariant_silent_range_sample_emits_no_findings() {
        // Defined-but-unhandled TypeIDs that must produce no finding:
        // 1–44: monitoring direction; 52–99: above control range;
        // 102: C_RD (read, not detection set); 104: between C_RP and C_IC; 106–127: future
        let silent_type_ids: &[u8] = &[1, 30, 44, 52, 99, 102, 104, 106, 127];
        for &type_id in silent_type_ids {
            let asdu = make_asdu(type_id, false);
            let mut findings = Vec::new();
            detect_iec104_threats(&asdu, &mut findings);
            assert!(
                findings.is_empty(),
                "TypeID={type_id} (defined-but-unhandled in [1,127]) must produce no finding — \
                 silently logged (BC-2.19.022 invariant 1; AC-170-005)"
            );
        }
    }

    // =========================================================================
    // BC-2.19.017: cot_test=true Tags Finding Summaries with " [TEST]"
    // AC-170-007
    // =========================================================================

    /// BC-2.19.017 canonical vector: TypeID=45, cot_test=true → summary ends with " [TEST]".
    ///
    /// EC-009 (STORY-170): cot_test=true with TypeID=45 (control command test frame) →
    /// T1692.001 Possible emitted with ` [TEST]` appended to Finding::summary.
    ///
    /// Traces: BC-2.19.017 invariant 1; AC-170-007; EC-001 (BC-017); EC-009 (STORY-170).
    #[test]
    fn test_BC_2_19_017_cot_test_true_control_command_summary_has_test_tag() {
        let asdu = make_asdu(45, true);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            !findings.is_empty(),
            "TypeID=45 must emit at least one finding (precondition for [TEST] tagging test)"
        );
        for f in &findings {
            assert!(
                f.summary.contains(" [TEST]"),
                "TypeID=45 with cot_test=true: finding summary must contain \" [TEST]\" — \
                 (BC-2.19.017 invariant 1; AC-170-007; EC-009 STORY-170) \
                 actual summary: {:?}",
                f.summary
            );
        }
    }

    /// BC-2.19.017 negative: TypeID=45, cot_test=false → summary does NOT contain " [TEST]".
    ///
    /// The [TEST] tag must only appear when cot_test=true. Normal operational findings
    /// must not be polluted with the test marker.
    ///
    /// Traces: BC-2.19.017 invariant 1 (contrapositive); AC-170-007 (negative).
    #[test]
    fn test_BC_2_19_017_cot_test_false_control_command_summary_has_no_test_tag() {
        let asdu = make_asdu(45, false);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        for f in &findings {
            assert!(
                !f.summary.contains(" [TEST]"),
                "TypeID=45 with cot_test=false: finding summary must NOT contain \" [TEST]\" — \
                 test tag must only appear when cot_test=true \
                 (BC-2.19.017 invariant 1 contrapositive; AC-170-007) \
                 actual summary: {:?}",
                f.summary
            );
        }
    }

    /// BC-2.19.017: TypeID=105 (T0827), cot_test=true → summary contains " [TEST]".
    ///
    /// The [TEST] tag applies to ALL finding types, including T0827 Loss of Control.
    ///
    /// Traces: BC-2.19.017 invariant 1; AC-170-007.
    #[test]
    fn test_BC_2_19_017_cot_test_true_t0827_summary_has_test_tag() {
        let asdu = make_asdu(105, true);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            !findings.is_empty(),
            "TypeID=105 must emit a T0827 finding (precondition for [TEST] tag test)"
        );
        for f in &findings {
            assert!(
                f.summary.contains(" [TEST]"),
                "TypeID=105 (T0827) with cot_test=true: finding summary must contain \" [TEST]\" \
                 (BC-2.19.017 invariant 1; AC-170-007) actual summary: {:?}",
                f.summary
            );
        }
    }

    /// BC-2.19.017: TypeID=48 (T1692.001 + T0836), cot_test=true → both findings have " [TEST]".
    ///
    /// When cot_test=true on a set-point TypeID, BOTH co-emitted findings must be tagged.
    ///
    /// Traces: BC-2.19.017 invariant 1; AC-170-007; BC-2.19.019 postcondition 2.
    #[test]
    fn test_BC_2_19_017_cot_test_true_setpoint_both_findings_have_test_tag() {
        let asdu = make_asdu(48, true);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert_eq!(
            findings.len(),
            2,
            "TypeID=48 with cot_test=true must still emit 2 findings \
             (BC-2.19.019 postconditions 1–2; cot_test does not suppress findings)"
        );
        for f in &findings {
            assert!(
                f.summary.contains(" [TEST]"),
                "TypeID=48 with cot_test=true: ALL findings must have \" [TEST]\" in summary — \
                 found summary without tag: {:?} (BC-2.19.017 invariant 1; AC-170-007)",
                f.summary
            );
        }
    }

    /// BC-2.19.017: TypeID=128 (T0814 reserved), cot_test=true → summary contains " [TEST]".
    ///
    /// The [TEST] tag applies to T0814 anomaly findings as well.
    ///
    /// Traces: BC-2.19.017 invariant 1; AC-170-007; BC-2.19.022 postcondition 1.
    #[test]
    fn test_BC_2_19_017_cot_test_true_t0814_reserved_type_summary_has_test_tag() {
        let asdu = make_asdu(128, true);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            !findings.is_empty(),
            "TypeID=128 must emit a T0814 finding (precondition for [TEST] tag test)"
        );
        for f in &findings {
            assert!(
                f.summary.contains(" [TEST]"),
                "TypeID=128 (T0814 reserved) with cot_test=true: finding summary must contain \
                 \" [TEST]\" (BC-2.19.017 invariant 1; AC-170-007) actual summary: {:?}",
                f.summary
            );
        }
    }

    /// BC-2.19.017 invariant: cot_test=false never produces " [TEST]" in any finding.
    ///
    /// Tests a representative cross-section of TypeIDs that emit findings:
    /// {45 (T1692.001), 48 (T1692.001+T0836), 105 (T0827), 128 (T0814)}.
    /// For all, cot_test=false → no finding has " [TEST]" in summary.
    ///
    /// Traces: BC-2.19.017 invariant 1 (contrapositive); AC-170-007 (negative).
    #[test]
    fn test_BC_2_19_017_invariant_cot_test_false_never_adds_test_tag() {
        let detection_type_ids: &[u8] = &[45, 48, 105, 128];
        for &type_id in detection_type_ids {
            let asdu = make_asdu(type_id, false);
            let mut findings = Vec::new();
            detect_iec104_threats(&asdu, &mut findings);
            for f in &findings {
                assert!(
                    !f.summary.contains(" [TEST]"),
                    "TypeID={type_id} with cot_test=false must NOT have \" [TEST]\" in summary — \
                     found: {:?} (BC-2.19.017 invariant 1 contrapositive; AC-170-007)",
                    f.summary
                );
            }
        }
    }

    // =========================================================================
    // F-170-001: CASDU and first_ioa target-address context in findings
    // BC-2.19.019 postcondition 3, BC-2.19.020 postcondition 2
    // =========================================================================

    /// F-170-001: CASDU appears in finding evidence for a control command TypeID.
    ///
    /// TypeID=48 (C_SE_NA_1) with casdu=100 → at least one finding.evidence entry
    /// contains "CASDU=100". Verifies BC-2.19.019 postcondition 3 requirement that
    /// every control-command finding carries ASDU-layer target-address context.
    ///
    /// Traces: BC-2.19.019 postcondition 3; BC-2.19.020 postcondition 2; F-170-001.
    #[test]
    fn test_F_170_001_casdu_appears_in_finding_evidence_for_control_type() {
        let asdu = Asdu {
            type_id: 48,
            sq: false,
            count: 1,
            cot_cause: 6,
            cot_pn: false,
            cot_test: false,
            cot_originator: 0,
            casdu: 100,
            first_ioa: None,
        };
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            !findings.is_empty(),
            "TypeID=48 must emit findings (precondition for CASDU evidence check)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.evidence.iter().any(|e| e.contains("CASDU=100"))),
            "TypeID=48 with casdu=100: at least one finding.evidence entry must contain \
             \"CASDU=100\" (BC-2.19.019 postcondition 3; F-170-001)"
        );
    }

    /// F-170-001: first_ioa appears in finding evidence when asdu.first_ioa is Some.
    ///
    /// TypeID=48 with first_ioa=Some(0x1234)=4660 → at least one finding.evidence entry
    /// contains "first_ioa=4660". Verifies BC-2.19.019 postcondition 3 IOA field.
    ///
    /// Traces: BC-2.19.019 postcondition 3; BC-2.19.020 postcondition 2; F-170-001.
    #[test]
    fn test_F_170_001_first_ioa_appears_in_finding_evidence_when_some() {
        let asdu = Asdu {
            type_id: 48,
            sq: false,
            count: 1,
            cot_cause: 6,
            cot_pn: false,
            cot_test: false,
            cot_originator: 0,
            casdu: 1,
            first_ioa: Some(0x1234), // decimal 4660
        };
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings);
        assert!(
            !findings.is_empty(),
            "TypeID=48 must emit findings (precondition for first_ioa evidence check)"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.evidence.iter().any(|e| e.contains("first_ioa=4660"))),
            "TypeID=48 with first_ioa=Some(0x1234=4660): at least one finding.evidence entry \
             must contain \"first_ioa=4660\" (BC-2.19.019 postcondition 3; F-170-001)"
        );
    }

    /// BC-2.19.017 start_idx guard: pre-existing findings are NOT re-tagged with [TEST].
    ///
    /// Calls detect_iec104_threats with a non-empty findings Vec (pre-populated with a
    /// dummy Finding), then emits for TypeID=45 with cot_test=true. Asserts that the
    /// PRE-EXISTING finding is NOT re-tagged with " [TEST]" — only findings pushed
    /// during this call are tagged. Proves the start_idx shared-accumulator logic
    /// (STORY-173 path; BC-2.19.017 invariant 1 scoping).
    ///
    /// Traces: BC-2.19.017 invariant 1; AC-170-007; start_idx accumulator guard.
    #[test]
    fn test_BC_2_19_017_start_idx_guard_preexisting_finding_not_tagged() {
        use wirerust::findings::{Confidence, Finding};

        let dummy = Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Possible,
            confidence: Confidence::Low,
            summary: "pre-existing dummy finding from prior call".to_string(),
            evidence: vec!["dummy evidence".to_string()],
            mitre_techniques: vec!["T9999".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        };
        let mut findings = vec![dummy];

        let asdu = make_asdu(45, true); // cot_test=true → new findings get [TEST]
        detect_iec104_threats(&asdu, &mut findings);

        // The pre-existing finding (index 0) must NOT be tagged:
        assert!(
            !findings[0].summary.contains(" [TEST]"),
            "pre-existing finding must NOT be tagged with \" [TEST]\" — \
             start_idx logic must only tag findings emitted by this call \
             (BC-2.19.017 invariant 1; start_idx accumulator guard)"
        );
        // New findings (from index 1 onward) must have [TEST]:
        assert!(
            findings.len() > 1,
            "TypeID=45 with cot_test=true must emit at least one new finding \
             after the pre-existing one (precondition for new-findings [TEST] check)"
        );
        assert!(
            findings[1..].iter().all(|f| f.summary.contains(" [TEST]")),
            "all NEW findings emitted by this call must have \" [TEST]\" in summary \
             (BC-2.19.017 invariant 1; cot_test=true; start_idx accumulator guard)"
        );
    }
}
