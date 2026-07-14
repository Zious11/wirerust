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
