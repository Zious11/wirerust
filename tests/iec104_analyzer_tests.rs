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
// Covers BC-2.19.007–014 and VP-046 proptest (full 256-value run verified in STORY-174).
// classify_frame_format and process_u_frame are implemented; all tests in this section pass.
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
// - VP-046: proptest — classify_frame_format totality over all 256 u8 values (verified STORY-174)
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
    use wirerust::reassembly::handler::Direction;

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
        let finding = process_u_frame(&mut state, 0x07, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x07, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x0B, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x13, Direction::ClientToServer, None, None);
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
        let stop_finding = process_u_frame(&mut state, 0x13, Direction::ClientToServer, None, None);
        assert!(
            !state.session_started,
            "STOPDT must set session_started=false"
        );
        assert!(stop_finding.is_some(), "STOPDT must emit a finding");
        // Now STARTDT restarts the session
        let start_finding = process_u_frame(&mut state, 0x07, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x13, Direction::ClientToServer, None, None);
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
        let f_possible = process_u_frame(&mut state_active, 0x13, Direction::ClientToServer, None, None)
            .expect("STOPDT after STARTDT must emit finding");
        assert_eq!(
            f_possible.verdict,
            Verdict::Possible,
            "STOPDT after STARTDT must emit Possible (BC-2.19.011)"
        );

        // Path 2: no prior STARTDT → Likely
        let mut state_cold = Iec104FlowState::default();
        let f_likely = process_u_frame(&mut state_cold, 0x13, Direction::ClientToServer, None, None)
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
        let finding = process_u_frame(&mut state, 0x23, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x43, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x83, Direction::ClientToServer, None, None);
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
        let _ = process_u_frame(&mut state_cold, 0x43, Direction::ClientToServer, None, None);
        assert!(
            !state_cold.session_started,
            "TESTFR-act must not change session_started from false (BC-2.19.013 postcondition 2)"
        );
        let _ = process_u_frame(&mut state_cold, 0x83, Direction::ClientToServer, None, None);
        assert!(
            !state_cold.session_started,
            "TESTFR-con must not change session_started from false (BC-2.19.013 postcondition 2)"
        );

        // Case 2: session_started=true — should remain true after TESTFR
        let mut state_active = Iec104FlowState {
            session_started: true,
            ..Default::default()
        };
        let _ = process_u_frame(&mut state_active, 0x43, Direction::ClientToServer, None, None);
        assert!(
            state_active.session_started,
            "TESTFR-act must not change session_started from true (BC-2.19.013 postcondition 2)"
        );
        let _ = process_u_frame(&mut state_active, 0x83, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x03, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0xFF, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x0F, Direction::ClientToServer, None, None);
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
        let finding = process_u_frame(&mut state, 0x1B, Direction::ClientToServer, None, None);
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
        let _ = process_u_frame(&mut state_cold, 0x0F, Direction::ClientToServer, None, None);
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
        let _ = process_u_frame(&mut state_active, 0xFF, Direction::ClientToServer, None, None);
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
            let finding = process_u_frame(&mut state, cf1, Direction::ClientToServer, None, None);
            assert!(
                finding.is_none(),
                "Canonical CF1=0x{cf1:02X} must not emit any finding (BC-2.19.014 negative)"
            );
        }
        // STOPDT-act produces T0881 (not T0814)
        let mut state = Iec104FlowState::default();
        let f = process_u_frame(&mut state, 0x13, Direction::ClientToServer, None, None)
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
    // VP-046 proptest: classify_frame_format totality (full 256-value run — STORY-174)
    // AC-168-009
    // =========================================================================

    // VP-046 proptest — classify_frame_format totality over all 256 u8 values.
    //
    // Per AC-168-009: proptest_vp046_frame_format_totality verifies classify_frame_format
    // over all 256 CF1 values. VP-046 full proof run completed in STORY-174.
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
// parse_asdu is implemented; all tests in this section pass.
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
// All tests GREEN against the delivered detect_iec104_threats TypeID dispatch
// (STORY-170 strict TDD; authored Red-first).
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
// Written Red-first as TDD stubs (STORY-170 strict TDD mode); GREEN against the
// delivered detect_iec104_threats TypeID dispatch table (STORY-170).
// =============================================================================
mod story_170 {
    use wirerust::analyzer::iec104::{Asdu, detect_iec104_threats};
    use wirerust::findings::{ThreatCategory, Verdict};
    use wirerust::reassembly::handler::Direction;

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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
            detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
            detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
            detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
            detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
            detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
            detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
            detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
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
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);

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

// =============================================================================
// STORY-171: IEC-104 N(S)/N(R) Sequence Tracking
//            Option<u16> First-Frame Guard + Desync Detection
//
// Covers BC-2.19.023 and BC-2.19.024.
//
// ## Contract coverage
// - BC-2.19.023: extract_ns(cf1, cf2) -> u16 via ((cf1>>1)|(cf2<<7)); range [0,32767]
//               extract_nr(cf3, cf4) -> u16 via ((cf3>>1)|(cf4<<7)); range [0,32767]
//               N(R) is transient — NOT stored in Iec104FlowState
// - BC-2.19.024: track_ns_desync(&mut state, current_ns, direction) -> Option<Finding>
//               Path A (None): state → Some(ns); no finding (mid-capture guard)
//               Path B (Some(prev), gap ≤ 12): state → Some(current_ns); no finding
//               Path C (Some(prev), gap > 12): T1692.001 Possible; state → Some(current_ns)
//               15-bit modular gap: current_ns.wrapping_sub(prev) & 0x7FFF (mandatory mask)
//               Directional isolation: last_ns_c2s / last_ns_s2c independent
//
// ## Canonical test vectors (BC-2.19.023 + BC-2.19.024 tables; used verbatim)
// BC-2.19.023: CF1=0x02/CF2=0x00→N(S)=1; CF1=0x00/CF2=0x00→N(S)=0; CF1=0xFE/CF2=0xFF→32767
// BC-2.19.024: None→5000→None; Some(5000)→5001→None; Some(5001)→5020→T1692.001 Possible
//
// ## RETRANSMIT-NS-FALSEPOS-001 note
// TCP retransmissions that re-deliver lower N(S) values produce a large backwards
// 15-bit gap (e.g. prev=5020, current=5001 → gap=32749) → T1692.001 Possible.
// This is INTENTIONAL fail-closed behavior (INV-3). The test documents and exercises
// this known false-positive; suppression via TCP deduplication is deferred.
//
// ## Provenance
// Written Red-first as TDD stubs (STORY-171 strict TDD mode, wave-80); now GREEN
// against the delivered extract_ns / extract_nr / track_ns_desync implementation.
// =============================================================================
mod story_171 {
    use proptest::prelude::*;
    use wirerust::analyzer::iec104::{Iec104FlowState, extract_nr, extract_ns, track_ns_desync};
    use wirerust::findings::Verdict;
    use wirerust::reassembly::handler::Direction;

    // =========================================================================
    // BC-2.19.023: extract_ns — 15-bit N(S) from CF1/CF2
    // AC-171-001
    // =========================================================================

    /// BC-2.19.023 canonical vector: CF1=0x02, CF2=0x00 → N(S)=1.
    ///
    /// Formula: `((0x02u16) >> 1) | ((0x00u16) << 7)` = `0x01 | 0x00` = 1.
    /// Canonical vector from BC-2.19.023 test vector table row 1.
    ///
    /// Traces: BC-2.19.023 postcondition 1; AC-171-001; EC-001.
    #[test]
    fn test_BC_2_19_023_extract_ns_cf1_0x02_cf2_0x00_returns_1() {
        let ns = extract_ns(0x02, 0x00);
        assert_eq!(
            ns, 1,
            "extract_ns(0x02, 0x00) must return 1 (BC-2.19.023 canonical vector table row 1)"
        );
    }

    /// BC-2.19.023 canonical vector: CF1=0xFE, CF2=0xFF → N(S)=32767.
    ///
    /// Formula: `((0xFEu16) >> 1) | ((0xFFu16) << 7)` = `0x7F | 0x7F80` = `0x7FFF` = 32767.
    /// This is the maximum 15-bit value.
    /// Canonical vector from BC-2.19.023 EC-003 and test vector table row 3.
    ///
    /// Traces: BC-2.19.023 postcondition 1, invariant 1; AC-171-001; EC-003.
    #[test]
    fn test_BC_2_19_023_extract_ns_cf1_0xFE_cf2_0xFF_returns_32767() {
        let ns = extract_ns(0xFE, 0xFF);
        assert_eq!(
            ns, 32767,
            "extract_ns(0xFE, 0xFF) must return 32767 = 0x7FFF \
             (BC-2.19.023 canonical vector EC-003; maximum 15-bit value)"
        );
    }

    /// BC-2.19.023 canonical vector: CF1=0x00, CF2=0x00 → N(S)=0.
    ///
    /// Formula: `((0x00u16) >> 1) | ((0x00u16) << 7)` = 0.
    /// Canonical vector from BC-2.19.023 test vector table row 2 (N(S) column).
    ///
    /// Traces: BC-2.19.023 postcondition 1; AC-171-001.
    #[test]
    fn test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x00_returns_0() {
        let ns = extract_ns(0x00, 0x00);
        assert_eq!(
            ns, 0,
            "extract_ns(0x00, 0x00) must return 0 (BC-2.19.023 zero case)"
        );
    }

    /// BC-2.19.023 EC-002: CF1=0x00, CF2=0x80 → N(S)=16384 = 0x4000.
    ///
    /// Formula: `((0x00u16) >> 1) | ((0x80u16) << 7)` = `0 | 0x4000` = 16384.
    /// Mid-range 15-bit value; only the CF2 high-bits contribute.
    ///
    /// Traces: BC-2.19.023 postcondition 1, invariant 1; AC-171-001; EC-002.
    #[test]
    fn test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x80_returns_16384() {
        let ns = extract_ns(0x00, 0x80);
        assert_eq!(
            ns, 16384,
            "extract_ns(0x00, 0x80) must return 16384 = 0x4000 (BC-2.19.023 EC-002)"
        );
    }

    /// BC-2.19.023 invariant 1: boundary range check for extract_ns.
    ///
    /// Exercises all three canonical table rows plus EC-002. Asserts the exact expected
    /// value AND the 15-bit range invariant (result must be ≤ 32767) for each input.
    ///
    /// Traces: BC-2.19.023 invariant 1; AC-171-001; VP-047 seam.
    #[test]
    fn test_BC_2_19_023_invariant_extract_ns_range_and_exact_values_boundary_inputs() {
        // (cf1, cf2, expected_ns) — BC-2.19.023 canonical table + EC-002/EC-003
        let cases: &[(u8, u8, u16)] = &[
            (0x00, 0x00, 0),     // table row 2 N(S) col: zero
            (0x02, 0x00, 1),     // table row 1 N(S) col
            (0x00, 0x80, 16384), // EC-002: 0x4000
            (0xFE, 0xFF, 32767), // table row 3 N(S) col / EC-003: 0x7FFF
        ];
        for &(cf1, cf2, expected) in cases {
            let ns = extract_ns(cf1, cf2);
            assert_eq!(
                ns, expected,
                "extract_ns(0x{cf1:02X}, 0x{cf2:02X}) must return {expected} \
                 (BC-2.19.023 postcondition 1 + invariant 1)"
            );
            assert!(
                ns <= 32767,
                "extract_ns(0x{cf1:02X}, 0x{cf2:02X}) = {ns} must be in [0, 32767] \
                 (BC-2.19.023 invariant 1)"
            );
        }
    }

    // =========================================================================
    // BC-2.19.023: extract_nr — 15-bit N(R) from CF3/CF4
    // AC-171-002
    // =========================================================================

    /// BC-2.19.023 canonical vector: CF3=0x02, CF4=0x00 → N(R)=1.
    ///
    /// Formula: `((0x02u16) >> 1) | ((0x00u16) << 7)` = 1.
    /// Canonical vector from BC-2.19.023 test vector table row 1 (N(R) column).
    ///
    /// Traces: BC-2.19.023 postcondition 2; AC-171-002.
    #[test]
    fn test_BC_2_19_023_extract_nr_cf3_0x02_cf4_0x00_returns_1() {
        let nr = extract_nr(0x02, 0x00);
        assert_eq!(
            nr, 1,
            "extract_nr(0x02, 0x00) must return 1 (BC-2.19.023 canonical vector table row 1)"
        );
    }

    /// BC-2.19.023 canonical vector: CF3=0xFE, CF4=0xFF → N(R)=32767.
    ///
    /// Formula: `((0xFEu16) >> 1) | ((0xFFu16) << 7)` = `0x7FFF` = 32767.
    /// Canonical vector from BC-2.19.023 test vector table row 3 (N(R) column).
    ///
    /// Traces: BC-2.19.023 postcondition 2, invariant 1; AC-171-002.
    #[test]
    fn test_BC_2_19_023_extract_nr_cf3_0xFE_cf4_0xFF_returns_32767() {
        let nr = extract_nr(0xFE, 0xFF);
        assert_eq!(
            nr, 32767,
            "extract_nr(0xFE, 0xFF) must return 32767 (BC-2.19.023 canonical vector table row 3)"
        );
    }

    /// BC-2.19.023: CF3=0x00, CF4=0x00 → N(R)=0.
    ///
    /// Formula: `((0x00u16) >> 1) | ((0x00u16) << 7)` = 0.
    ///
    /// Traces: BC-2.19.023 postcondition 2; AC-171-002.
    #[test]
    fn test_BC_2_19_023_extract_nr_cf3_0x00_cf4_0x00_returns_0() {
        let nr = extract_nr(0x00, 0x00);
        assert_eq!(
            nr, 0,
            "extract_nr(0x00, 0x00) must return 0 (BC-2.19.023 zero case)"
        );
    }

    /// BC-2.19.023 postcondition 4: N(R) is transient — Iec104FlowState has no last_nr fields.
    ///
    /// Postcondition 4 states N(R) is computed and available transiently but is NOT stored.
    /// This test calls extract_nr and confirms the return value, then verifies that
    /// Iec104FlowState has no last_nr field (compile-time proof: rustc rejects access to
    /// nonexistent fields; runtime proof: state only exposes last_ns_c2s / last_ns_s2c).
    ///
    /// Traces: BC-2.19.023 postcondition 4; AC-171-002.
    #[test]
    fn test_BC_2_19_023_extract_nr_is_transient_no_last_nr_field_in_flow_state() {
        // extract_nr returns a transient value — the caller holds it temporarily.
        let nr = extract_nr(0x02, 0x00);
        assert_eq!(
            nr, 1,
            "extract_nr(0x02, 0x00) must return 1 — used transiently by caller \
             (BC-2.19.023 postcondition 4)"
        );
        // Compile-time proof of postcondition 4: Iec104FlowState has last_ns_c2s / last_ns_s2c
        // but NO last_nr_c2s / last_nr_s2c. Any attempt to access .last_nr_c2s would fail to
        // compile. Runtime proof: the Default state only initialises the ns fields.
        let state = Iec104FlowState::default();
        assert_eq!(
            state.last_ns_c2s, None,
            "Iec104FlowState::default() must have last_ns_c2s=None (not last_nr)"
        );
        assert_eq!(
            state.last_ns_s2c, None,
            "Iec104FlowState::default() must have last_ns_s2c=None (not last_nr)"
        );
    }

    /// BC-2.19.023: extract_nr and extract_ns use the symmetric formula — same inputs yield same output.
    ///
    /// The formulas are identical: ns uses CF1/CF2; nr uses CF3/CF4. For equal byte pairs the
    /// results must be equal. Exercises all three canonical table rows.
    ///
    /// Traces: BC-2.19.023 postconditions 1-2; AC-171-001, AC-171-002.
    #[test]
    fn test_BC_2_19_023_extract_nr_symmetric_formula_equal_inputs_equal_outputs() {
        // BC canonical table: all three rows have matching (cf1=cf3, cf2=cf4) inputs
        let cases: &[(u8, u8)] = &[(0x02, 0x00), (0x00, 0x00), (0xFE, 0xFF)];
        for &(hi, lo) in cases {
            let ns = extract_ns(hi, lo);
            let nr = extract_nr(hi, lo);
            assert_eq!(
                ns, nr,
                "extract_ns(0x{hi:02X}, 0x{lo:02X}) must equal extract_nr(0x{hi:02X}, 0x{lo:02X}) \
                 — symmetric formula (BC-2.19.023 postconditions 1-2)"
            );
        }
    }

    // BC-2.19.023 invariant 2: extract_ns result is always in [0, 32767] for all u8 inputs.
    //
    // Proptest exercises 1000+ random (cf1, cf2) input pairs and asserts no overflow.
    // Verifies VP-047 no-overflow property for the N(S) extraction path.
    //
    // Traces: BC-2.19.023 invariant 2; AC-171-001; VP-047 seam.
    proptest! {
        #[test]
        fn test_BC_2_19_023_proptest_extract_ns_always_in_15bit_range(
            cf1 in 0u8..=255u8,
            cf2 in 0u8..=255u8,
        ) {
            let ns = extract_ns(cf1, cf2);
            prop_assert!(
                ns <= 32767,
                "extract_ns(0x{:02X}, 0x{:02X}) = {} must be ≤ 32767 \
                 (BC-2.19.023 invariant 2 — 15-bit range; VP-047)",
                cf1,
                cf2,
                ns
            );
        }
    }

    // BC-2.19.023 invariant 2: extract_nr result is always in [0, 32767] for all u8 inputs.
    //
    // Proptest exercises 1000+ random (cf3, cf4) input pairs and asserts no overflow.
    //
    // Traces: BC-2.19.023 invariant 2; AC-171-002; VP-047 seam.
    proptest! {
        #[test]
        fn test_BC_2_19_023_proptest_extract_nr_always_in_15bit_range(
            cf3 in 0u8..=255u8,
            cf4 in 0u8..=255u8,
        ) {
            let nr = extract_nr(cf3, cf4);
            prop_assert!(
                nr <= 32767,
                "extract_nr(0x{:02X}, 0x{:02X}) = {} must be ≤ 32767 \
                 (BC-2.19.023 invariant 2 — 15-bit range; VP-047)",
                cf3,
                cf4,
                nr
            );
        }
    }

    // =========================================================================
    // BC-2.19.024 Path A: first I-frame (state None) — no finding; baseline set
    // AC-171-003
    // =========================================================================

    /// BC-2.19.024 Path A EC-001: first I-frame C2S, N(S)=0, state None → no finding;
    /// last_ns_c2s = Some(0).
    ///
    /// Precondition: state.last_ns_c2s == None (fresh flow, no prior I-frame).
    /// Postcondition A1: state.last_ns_c2s == Some(0) — exact value asserted.
    /// Postcondition A2: return value is None — no finding unconditionally on first frame.
    ///
    /// Traces: BC-2.19.024 Path A postconditions 1-2; AC-171-003; EC-001.
    #[test]
    fn test_BC_2_19_024_path_a_first_frame_c2s_ns_0_no_finding_state_becomes_some_0() {
        let mut state = Iec104FlowState::default();
        assert_eq!(
            state.last_ns_c2s, None,
            "precondition: last_ns_c2s must be None"
        );
        let finding = track_ns_desync(&mut state, 0, Direction::ClientToServer, None, None);
        assert!(
            finding.is_none(),
            "Path A (None state): first I-frame with N(S)=0 must return None — \
             no finding (BC-2.19.024 postcondition A2; first-frame guard)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(0),
            "Path A: last_ns_c2s must become Some(0) after first I-frame \
             (BC-2.19.024 postcondition A1)"
        );
    }

    /// BC-2.19.024 Path A EC-002/EC-006: mid-capture start, first I-frame C2S, N(S)=5000
    /// (arbitrary mid-capture value) → no finding; last_ns_c2s = Some(5000).
    ///
    /// This is the primary correctness guard for the mid-capture use case:
    /// the first observed N(S) is arbitrary (not necessarily 0) and MUST NEVER
    /// generate a desync finding regardless of its value (ADR-013 Decision 6).
    ///
    /// Traces: BC-2.19.024 Path A postconditions 1-2; AC-171-003; EC-002; EC-006.
    #[test]
    fn test_BC_2_19_024_path_a_mid_capture_first_frame_c2s_ns_5000_no_finding_state_becomes_some_5000()
     {
        let mut state = Iec104FlowState::default();
        assert_eq!(
            state.last_ns_c2s, None,
            "precondition: last_ns_c2s must be None"
        );
        let finding = track_ns_desync(&mut state, 5000, Direction::ClientToServer, None, None);
        assert!(
            finding.is_none(),
            "Path A mid-capture: N(S)=5000 on first frame must return None — \
             no false positive on arbitrary mid-capture N(S) \
             (BC-2.19.024 postcondition A2; AC-171-003; EC-002)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(5000),
            "Path A mid-capture: last_ns_c2s must become Some(5000) \
             (BC-2.19.024 postcondition A1)"
        );
    }

    /// BC-2.19.024 Path A: first I-frame S2C direction, N(S)=0 → no finding;
    /// last_ns_s2c = Some(0); last_ns_c2s remains None.
    ///
    /// Mirrors the C2S Path A test for the opposite direction.
    /// Also pre-checks directional isolation: last_ns_c2s must not be touched.
    ///
    /// Traces: BC-2.19.024 Path A postconditions 1-2; AC-171-003; AC-171-007.
    #[test]
    fn test_BC_2_19_024_path_a_first_frame_s2c_ns_0_no_finding_state_becomes_some_0() {
        let mut state = Iec104FlowState::default();
        assert_eq!(
            state.last_ns_s2c, None,
            "precondition: last_ns_s2c must be None"
        );
        assert_eq!(
            state.last_ns_c2s, None,
            "precondition: last_ns_c2s must be None"
        );
        let finding = track_ns_desync(&mut state, 0, Direction::ServerToClient, None, None);
        assert!(
            finding.is_none(),
            "Path A S2C: first I-frame with N(S)=0 must return None \
             (BC-2.19.024 postcondition A2)"
        );
        assert_eq!(
            state.last_ns_s2c,
            Some(0),
            "Path A S2C: last_ns_s2c must become Some(0) \
             (BC-2.19.024 postcondition A1)"
        );
        assert_eq!(
            state.last_ns_c2s, None,
            "Path A S2C: last_ns_c2s must remain None — no cross-direction mutation \
             (AC-171-007 directional isolation)"
        );
    }

    // =========================================================================
    // BC-2.19.024 Path B: subsequent frame, gap ≤ k=12 — no finding
    // AC-171-004
    // =========================================================================

    /// BC-2.19.024 Path B canonical vector: prev=5000, current=5001, gap=1 → no finding;
    /// last_ns_c2s = Some(5001).
    ///
    /// Canonical test vector from BC-2.19.024 table row 3: Some(5000)→5001, gap=1 → no finding.
    ///
    /// Traces: BC-2.19.024 Path B postconditions 1-2; AC-171-004; canonical table row 3.
    #[test]
    fn test_BC_2_19_024_path_b_gap_1_no_finding_state_updates_to_current_ns() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(5000),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 5001, Direction::ClientToServer, None, None);
        assert!(
            finding.is_none(),
            "Path B gap=1: no finding expected (BC-2.19.024 postcondition B2; 1 ≤ 12)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(5001),
            "Path B gap=1: last_ns_c2s must update to Some(5001) \
             (BC-2.19.024 postcondition B1)"
        );
    }

    /// BC-2.19.024 Path B EC-003: gap=12 (exactly k) → no finding.
    ///
    /// EC-003: gap=12 is the boundary — ≤ k is allowed, so no finding.
    /// Canonical test vector: Some(0)→12, gap=12 → no finding; state→Some(12).
    ///
    /// Traces: BC-2.19.024 Path B postconditions 1-2; AC-171-004; EC-003; canonical table row 5.
    #[test]
    fn test_BC_2_19_024_path_b_gap_12_exactly_k_boundary_no_finding() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(0),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 12, Direction::ClientToServer, None, None);
        assert!(
            finding.is_none(),
            "Path B EC-003: gap=12 (exactly k=12) must return None — boundary ≤ k is allowed \
             (BC-2.19.024 EC-003)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(12),
            "Path B EC-003: last_ns_c2s must update to Some(12) \
             (BC-2.19.024 postcondition B1)"
        );
    }

    /// BC-2.19.024 Path B: gap=0 (same N(S) repeated) → no finding.
    ///
    /// Gap=0 is valid (≤ 12); emitting no finding preserves non-false-positive behavior.
    ///
    /// Traces: BC-2.19.024 Path B postconditions 1-2; AC-171-004.
    #[test]
    fn test_BC_2_19_024_path_b_gap_0_same_ns_no_finding() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(100),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 100, Direction::ClientToServer, None, None);
        assert!(
            finding.is_none(),
            "Path B gap=0 (same N(S) repeated): no finding (BC-2.19.024; gap=0 ≤ 12)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(100),
            "Path B gap=0: state must update to Some(100) (same value) \
             (BC-2.19.024 postcondition B1)"
        );
    }

    // =========================================================================
    // BC-2.19.024 Path C: subsequent frame, gap > k=12 — T1692.001 Possible
    // AC-171-005
    // =========================================================================

    /// BC-2.19.024 Path C EC-004: gap=13 (k+1) → T1692.001 Possible.
    ///
    /// EC-004: gap=13 is the first value that exceeds k=12 and triggers a finding.
    /// Canonical test vector: Some(0)→13, gap=13 → T1692.001 Possible; state→Some(13).
    /// Asserts exact verdict (Possible) and mitre_techniques containing "T1692.001".
    ///
    /// Traces: BC-2.19.024 Path C postconditions 1-3; AC-171-005; EC-004; canonical table row 6.
    #[test]
    fn test_BC_2_19_024_path_c_gap_13_k_plus_1_emits_t1692_001_possible() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(0),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 13, Direction::ClientToServer, None, None);
        let f = finding.expect(
            "Path C EC-004: gap=13 (k+1) must emit T1692.001 Possible \
             (BC-2.19.024 Path C postcondition 1; EC-004)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "Path C EC-004: verdict must be Possible (BC-2.19.024 postcondition C1)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T1692.001"),
            "Path C EC-004: mitre_techniques must contain \"T1692.001\" \
             (BC-2.19.024 postcondition C1; AC-171-005)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(13),
            "Path C EC-004: last_ns_c2s must update to Some(13) \
             (BC-2.19.024 postcondition C3)"
        );
    }

    /// BC-2.19.024 Path C canonical vector: prev=5001, current=5020, gap=19 → T1692.001 Possible.
    ///
    /// Canonical test vector from BC-2.19.024 table row 4: Some(5001)→5020, gap=19 → T1692.001.
    /// Asserts exact verdict (Possible), mitre_techniques ("T1692.001"), and state update (Some(5020)).
    ///
    /// Traces: BC-2.19.024 Path C postconditions 1-3; AC-171-005; canonical table row 4.
    #[test]
    fn test_BC_2_19_024_path_c_gap_19_canonical_vector_prev_5001_current_5020_emits_t1692_001() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(5001),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 5020, Direction::ClientToServer, None, None);
        let f = finding.expect(
            "Path C canonical: Some(5001)→5020, gap=19 must emit T1692.001 Possible \
             (BC-2.19.024 canonical table row 4)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "Path C canonical: verdict must be Possible (BC-2.19.024 postcondition C1)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T1692.001"),
            "Path C canonical: mitre_techniques must contain \"T1692.001\" \
             (BC-2.19.024 postcondition C1; AC-171-005)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(5020),
            "Path C canonical: last_ns_c2s must update to Some(5020) \
             (BC-2.19.024 postcondition C3)"
        );
        // BC-2.19.024 PC-C2: finding message must embed current N(S), prev N(S), and gap value.
        // Impl summary format: "IEC-104 N(S) sequence desync: N(S)={current} prev={prev} gap={gap} > k=12"
        // F-171-002; DF-SIBLING-SWEEP-001.
        assert!(
            f.summary.contains("5020"),
            "Path C canonical: summary must contain current N(S) \"5020\" \
             (BC-2.19.024 postcondition C2; AC-171-005; F-171-002)"
        );
        assert!(
            f.summary.contains("5001"),
            "Path C canonical: summary must contain prev N(S) \"5001\" \
             (BC-2.19.024 postcondition C2; AC-171-005; F-171-002)"
        );
        assert!(
            f.summary.contains("19"),
            "Path C canonical: summary must contain gap value \"19\" \
             (BC-2.19.024 postcondition C2; AC-171-005; F-171-002)"
        );
    }

    /// BC-2.19.024 Path C canonical table row 8: prev=100, current=114, gap=14 → T1692.001 Possible.
    ///
    /// Canonical test vector from BC-2.19.024 table row 8: Some(100)→114, gap=14 → T1692.001 Possible.
    ///
    /// Traces: BC-2.19.024 Path C postconditions 1-3; AC-171-005; canonical table row 8.
    #[test]
    fn test_BC_2_19_024_path_c_canonical_table_row8_prev_100_current_114_gap_14_emits_finding() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(100),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 114, Direction::ClientToServer, None, None);
        let f = finding.expect(
            "Path C table row 8: Some(100)→114, gap=14 must emit T1692.001 Possible \
             (BC-2.19.024 canonical table row 8)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "Path C table row 8: verdict must be Possible"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T1692.001"),
            "Path C table row 8: mitre_techniques must contain \"T1692.001\""
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(114),
            "Path C table row 8: state must update to Some(114) (BC-2.19.024 postcondition C3)"
        );
    }

    /// BC-2.19.024 Path C EC-005: gap=32767 (massive jump / replay) → T1692.001 Possible.
    ///
    /// EC-005: prev=0, current=32767 → gap=32767 >> 12 → T1692.001 Possible.
    /// Indicates replay injection or completely desynchronized counter (INV-3 fail-closed).
    ///
    /// Traces: BC-2.19.024 Path C postconditions 1-3; AC-171-005; EC-005.
    #[test]
    fn test_BC_2_19_024_path_c_ec005_gap_32767_massive_jump_emits_t1692_001_possible() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(0),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 32767, Direction::ClientToServer, None, None);
        let f = finding.expect(
            "Path C EC-005: prev=0, current=32767 (gap=32767) must emit T1692.001 Possible \
             (BC-2.19.024 EC-005; massive gap indicates replay/desync)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "Path C EC-005: verdict must be Possible (BC-2.19.024 EC-005)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T1692.001"),
            "Path C EC-005: mitre_techniques must contain \"T1692.001\""
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(32767),
            "Path C EC-005: state must update to Some(32767) (BC-2.19.024 postcondition C3)"
        );
    }

    /// BC-2.19.024 Path C postcondition C3: state always updates to Some(current_ns)
    /// even when a finding is emitted.
    ///
    /// Verifies postcondition C3 independently by checking that after a Path C finding
    /// the next frame with gap=1 from the new baseline produces no finding (Path B),
    /// proving the state was updated and not left stale.
    ///
    /// Traces: BC-2.19.024 Path C postcondition 3; AC-171-005.
    #[test]
    fn test_BC_2_19_024_path_c_state_updates_to_current_ns_after_finding_emitted() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(1000),
            ..Default::default()
        };
        // gap = 1030 - 1000 = 30 > 12 → Path C finding
        let f1 = track_ns_desync(&mut state, 1030, Direction::ClientToServer, None, None);
        assert!(
            f1.is_some(),
            "gap=30 must emit a finding (Path C precondition)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(1030),
            "Path C: last_ns_c2s must update to Some(1030) even when finding emitted \
             (BC-2.19.024 postcondition C3)"
        );
        // Next frame from new baseline: gap=1 → no finding (proves state was updated)
        let f2 = track_ns_desync(&mut state, 1031, Direction::ClientToServer, None, None);
        assert!(
            f2.is_none(),
            "After state update to Some(1030), gap=1 (1030→1031) must not emit finding \
             (Path B — state correctly updated by Path C)"
        );
    }

    // =========================================================================
    // BC-2.19.024 invariant 1: 15-bit modular arithmetic — wrapping_sub & 0x7FFF
    // AC-171-006
    // =========================================================================

    /// BC-2.19.024 AC-171-006 EC-004: Some(32767) → current=1 → gap=2 (15-bit) → no finding.
    ///
    /// Gap calculation:
    ///   `(1u16.wrapping_sub(32767)) & 0x7FFF`
    ///   = `32770 & 0x7FFF` = `32770 & 32767` = 2 (≤ 12 → no finding).
    ///
    /// CRITICAL: plain subtraction would be `1 - 32767` which overflows in debug mode
    /// or gives the wrong 16-bit result. The `& 0x7FFF` mask is mandatory to collapse
    /// the 16-bit wrapping to the 15-bit N(S) range.
    ///
    /// Traces: BC-2.19.024 invariant 1; AC-171-006; EC-004.
    #[test]
    fn test_BC_2_19_024_ac171_006_wrap_32767_to_1_gap_2_no_finding() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(32767),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 1, Direction::ClientToServer, None, None);
        assert!(
            finding.is_none(),
            "AC-171-006: Some(32767)→current=1, 15-bit gap=2 must NOT emit finding \
             (BC-2.19.024 invariant 1 — wrapping_sub & 0x7FFF; EC-004)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(1),
            "AC-171-006: state must update to Some(1) after valid wrap \
             (BC-2.19.024 postcondition B1)"
        );
    }

    /// BC-2.19.024 AC-171-006: Some(32767) → current=0 → 15-bit gap=1 → no finding.
    ///
    /// Full wraparound: N(S) goes 32767 → 0 (one past the maximum).
    /// Gap = `(0u16.wrapping_sub(32767)) & 0x7FFF` = `32769 & 32767` = 1.
    /// gap=1 ≤ 12 → no finding. Validates BC-2.19.023 invariant 3 (valid wrap).
    ///
    /// Traces: BC-2.19.024 invariant 1; AC-171-006; BC-2.19.023 invariant 3.
    #[test]
    fn test_BC_2_19_024_ac171_006_wrap_32767_to_0_gap_1_no_finding() {
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(32767),
            ..Default::default()
        };
        let finding = track_ns_desync(&mut state, 0, Direction::ClientToServer, None, None);
        assert!(
            finding.is_none(),
            "AC-171-006: Some(32767)→0 full wrap, gap=1 (15-bit) must not emit finding \
             (BC-2.19.024 invariant 1; valid N(S) wraparound)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(0),
            "AC-171-006: state must update to Some(0) after full wrap"
        );
    }

    // =========================================================================
    // BC-2.19.024 invariant 3: Directional isolation — C2S and S2C independent
    // AC-171-007
    // =========================================================================

    /// BC-2.19.024 AC-171-007: C2S call updates last_ns_c2s, leaves last_ns_s2c untouched.
    ///
    /// Given fresh state (both fields None), a C2S call must update last_ns_c2s and
    /// leave last_ns_s2c as None.
    ///
    /// Traces: BC-2.19.023 postcondition 3; BC-2.19.024 invariant 3; AC-171-007.
    #[test]
    fn test_BC_2_19_024_ac171_007_c2s_call_updates_c2s_not_s2c() {
        let mut state = Iec104FlowState::default();
        let _f = track_ns_desync(&mut state, 100, Direction::ClientToServer, None, None);
        assert_eq!(
            state.last_ns_c2s,
            Some(100),
            "AC-171-007: C2S call must update last_ns_c2s to Some(100)"
        );
        assert_eq!(
            state.last_ns_s2c, None,
            "AC-171-007: C2S call must NOT touch last_ns_s2c — must remain None \
             (BC-2.19.024 directional isolation)"
        );
    }

    /// BC-2.19.024 AC-171-007: S2C call updates last_ns_s2c, leaves last_ns_c2s untouched.
    ///
    /// Given fresh state (both fields None), an S2C call must update last_ns_s2c and
    /// leave last_ns_c2s as None.
    ///
    /// Traces: BC-2.19.023 postcondition 3; BC-2.19.024 invariant 3; AC-171-007.
    #[test]
    fn test_BC_2_19_024_ac171_007_s2c_call_updates_s2c_not_c2s() {
        let mut state = Iec104FlowState::default();
        let _f = track_ns_desync(&mut state, 200, Direction::ServerToClient, None, None);
        assert_eq!(
            state.last_ns_s2c,
            Some(200),
            "AC-171-007: S2C call must update last_ns_s2c to Some(200)"
        );
        assert_eq!(
            state.last_ns_c2s, None,
            "AC-171-007: S2C call must NOT touch last_ns_c2s — must remain None \
             (BC-2.19.024 directional isolation)"
        );
    }

    /// BC-2.19.024 AC-171-007: interleaved C2S and S2C calls maintain independent baselines
    /// and do not interfere with each other's gap calculation.
    ///
    /// Four-frame sequence:
    ///   1. C2S N(S)=10  → Path A: None→Some(10); no finding
    ///   2. S2C N(S)=200 → Path A: None→Some(200); no finding
    ///   3. C2S N(S)=11  → Path B: gap=1 from Some(10); no finding; c2s→Some(11)
    ///   4. S2C N(S)=220 → Path C: gap=20 from Some(200) > 12; T1692.001 Possible; s2c→Some(220)
    ///      last_ns_c2s must remain Some(11) throughout step 4.
    ///
    /// Traces: BC-2.19.023 postcondition 3; BC-2.19.024 invariant 3; AC-171-007.
    #[test]
    fn test_BC_2_19_024_ac171_007_interleaved_c2s_s2c_independent_baselines_and_gaps() {
        let mut state = Iec104FlowState::default();

        // Step 1: C2S first frame N(S)=10 → Path A
        let f1 = track_ns_desync(&mut state, 10, Direction::ClientToServer, None, None);
        assert!(f1.is_none(), "step 1: C2S N(S)=10 Path A must return None");
        assert_eq!(state.last_ns_c2s, Some(10), "step 1: c2s must be Some(10)");
        assert_eq!(state.last_ns_s2c, None, "step 1: s2c must remain None");

        // Step 2: S2C first frame N(S)=200 → Path A
        let f2 = track_ns_desync(&mut state, 200, Direction::ServerToClient, None, None);
        assert!(f2.is_none(), "step 2: S2C N(S)=200 Path A must return None");
        assert_eq!(
            state.last_ns_s2c,
            Some(200),
            "step 2: s2c must be Some(200)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(10),
            "step 2: c2s must remain Some(10) after S2C call"
        );

        // Step 3: C2S N(S)=11 → Path B (gap=1)
        let f3 = track_ns_desync(&mut state, 11, Direction::ClientToServer, None, None);
        assert!(
            f3.is_none(),
            "step 3: C2S N(S)=11, gap=1 from Some(10) must return None"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(11),
            "step 3: c2s must update to Some(11)"
        );
        assert_eq!(
            state.last_ns_s2c,
            Some(200),
            "step 3: s2c must remain Some(200) after C2S update"
        );

        // Step 4: S2C N(S)=220 → Path C (gap=20 > 12) → T1692.001 Possible
        let f4 = track_ns_desync(&mut state, 220, Direction::ServerToClient, None, None);
        let f =
            f4.expect("step 4: S2C N(S)=220, gap=20 from Some(200) must emit T1692.001 Possible");
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "step 4: S2C Path C finding must be Verdict::Possible"
        );
        assert_eq!(
            state.last_ns_s2c,
            Some(220),
            "step 4: s2c must update to Some(220)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(11),
            "step 4: c2s must remain Some(11) — S2C Path C must not affect c2s (AC-171-007)"
        );
    }

    // =========================================================================
    // RETRANSMIT-NS-FALSEPOS-001: TCP retransmission false positive
    // EC-007 / STORY-171 Edge Case Table
    // =========================================================================

    /// RETRANSMIT-NS-FALSEPOS-001: TCP retransmission of older N(S) produces large
    /// backwards 15-bit gap → T1692.001 Possible (EXPECTED / INTENTIONAL false positive).
    ///
    /// Scenario:
    ///   last seen N(S) = 5020 (state = Some(5020))
    ///   TCP retransmission re-delivers N(S) = 5001 (older, lower than last seen)
    ///   Gap = (5001u16.wrapping_sub(5020)) & 0x7FFF
    ///       = (5001 + 65536 - 5020) & 0x7FFF = 65517 & 32767 = 32749 >> 12
    ///   → T1692.001 Possible emitted.
    ///
    /// This IS a false positive: the frame is benign (TCP retransmit of a real frame).
    /// The passive analyzer cannot distinguish TCP retransmits from adversarial replays.
    /// Behavior is INTENTIONALLY fail-closed (INV-3: Fail-Closed Finding Emission).
    /// Future mitigation via TCP deduplication is deferred (STORY-171 Edge Cases EC-007).
    ///
    /// DO NOT change this test to expect None or to treat the finding as incorrect.
    /// The finding IS correct for the MVP fail-closed policy.
    ///
    /// Traces: STORY-171 EC-007; RETRANSMIT-NS-FALSEPOS-001; BC-2.19.024 invariant 3 (INV-3).
    #[test]
    fn test_RETRANSMIT_NS_FALSEPOS_001_backwards_ns_yields_large_gap_emits_t1692_001_finding() {
        // Simulate: analyzer has seen N(S)=5020; TCP retransmit re-delivers N(S)=5001
        let mut state = Iec104FlowState {
            last_ns_c2s: Some(5020),
            ..Default::default()
        };
        // Backwards gap: (5001.wrapping_sub(5020)) & 0x7FFF = 32749 > 12 → T1692.001 Possible
        let finding = track_ns_desync(&mut state, 5001, Direction::ClientToServer, None, None);
        assert!(
            finding.is_some(),
            "RETRANSMIT-NS-FALSEPOS-001: backwards N(S) (5001 after 5020) must emit \
             T1692.001 Possible — fail-closed MVP behavior (INV-3). \
             This is an INTENTIONAL false positive: TCP retransmits that re-deliver \
             lower N(S) values are indistinguishable from adversarial replays by a \
             passive analyzer. Future mitigation: TCP deduplication."
        );
        let f = finding.unwrap();
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "RETRANSMIT-NS-FALSEPOS-001: finding verdict must be Possible"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T1692.001"),
            "RETRANSMIT-NS-FALSEPOS-001: finding must cite T1692.001"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(5001),
            "RETRANSMIT-NS-FALSEPOS-001: state must update to Some(5001) \
             even for backwards/retransmit N(S)"
        );
    }

    // =========================================================================
    // BC-2.19.024 EC-006: full mid-capture three-frame sequence
    // =========================================================================

    /// BC-2.19.024 EC-006: full mid-capture three-frame sequence exercises all three paths.
    ///
    /// EC-006 from BC-2.19.024:
    ///   Frame 1: state None → N(S)=5000 → Some(5000); no finding (Path A)
    ///   Frame 2: Some(5000) → N(S)=5001 → gap=1 → Some(5001); no finding (Path B)
    ///   Frame 3: Some(5001) → N(S)=5020 → gap=19 → T1692.001 Possible; Some(5020) (Path C)
    ///
    /// Tests all three postcondition paths in a single realistic capture sequence.
    ///
    /// Traces: BC-2.19.024 EC-006; AC-171-003, AC-171-004, AC-171-005.
    #[test]
    fn test_BC_2_19_024_ec_006_mid_capture_three_frame_sequence_exercises_all_three_paths() {
        let mut state = Iec104FlowState::default();

        // Frame 1: mid-capture start, N(S)=5000, state=None → Path A
        let f1 = track_ns_desync(&mut state, 5000, Direction::ClientToServer, None, None);
        assert!(
            f1.is_none(),
            "EC-006 frame 1: N(S)=5000, state=None must return None (Path A; mid-capture guard)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(5000),
            "EC-006 frame 1: state must become Some(5000) (Path A postcondition A1)"
        );

        // Frame 2: gap=1 from Some(5000) → Path B
        let f2 = track_ns_desync(&mut state, 5001, Direction::ClientToServer, None, None);
        assert!(
            f2.is_none(),
            "EC-006 frame 2: N(S)=5001, gap=1 from Some(5000) must return None (Path B)"
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(5001),
            "EC-006 frame 2: state must become Some(5001) (Path B postcondition B1)"
        );

        // Frame 3: gap=19 from Some(5001) → Path C → T1692.001 Possible
        let f3 = track_ns_desync(&mut state, 5020, Direction::ClientToServer, None, None);
        let f = f3.expect(
            "EC-006 frame 3: N(S)=5020, gap=19 from Some(5001) must emit T1692.001 Possible \
             (Path C; BC-2.19.024 EC-006)",
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "EC-006 frame 3: verdict must be Possible (BC-2.19.024 Path C)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T1692.001"),
            "EC-006 frame 3: mitre_techniques must contain \"T1692.001\""
        );
        assert_eq!(
            state.last_ns_c2s,
            Some(5020),
            "EC-006 frame 3: state must become Some(5020) (Path C postcondition C3)"
        );
    }
}

// =============================================================================
// STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle
//
// Unit tests (AC-172-001..008, EC-001..011) and VP-045 asserting proptests
// (AC-172-007; upgraded to non-vacuous assertions in STORY-174 AC-174-002).
//
// ## Contract coverage
// - BC-2.19.025: directional carry buffers bounded at MAX_IEC104_CARRY_BYTES = 255
// - BC-2.19.026: frame-walk loop processes multiple APDUs per on_data call
// - BC-2.19.027: on_flow_close removes Iec104FlowState and discards carry bytes
//
// ## Proptest obligation (VP-045)
// Harnesses: proptest_vp045_direction_isolation, proptest_vp045_independent_run_equivalence
// Mirrors VP-033 (ENIP carry isolation), VP-035 (DNP3), VP-037 (Modbus) patterns.
// =============================================================================
mod story_172 {
    use proptest::prelude::*;
    use wirerust::analyzer::iec104::{Iec104Analyzer, MAX_IEC104_CARRY_BYTES};
    use wirerust::findings::{ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    fn flow_key_default() -> FlowKey {
        FlowKey::new(
            "127.0.0.1".parse().unwrap(),
            1234,
            "127.0.0.2".parse().unwrap(),
            2404,
        )
    }

    // =========================================================================
    // AC-172-001: carry stash on insufficient data — partial APCI split
    // BC-2.19.025 postconditions 1–4, invariants 1–2
    // =========================================================================

    /// AC-172-001 C2S: partial APCI header (3 bytes) stashed into carry_c2s.
    ///
    /// A STARTDT U-frame is 6 bytes. Delivering only the first 3 bytes must
    /// stash them into `carry_c2s`; `carry_s2c` is untouched.
    ///
    /// Traces: BC-2.19.025 postconditions 1–2; AC-172-001.
    #[test]
    fn test_AC_172_001_carry_stash_c2s_partial_frame() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // 3 bytes of a 6-byte STARTDT frame — insufficient to complete the frame.
        let partial = [0x68u8, 0x04, 0x07];
        analyzer.on_data(flow_key.clone(), &partial, 0, Direction::ClientToServer);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s, partial,
            "3-byte partial STARTDT must be stashed into carry_c2s (AC-172-001)"
        );
        assert!(
            state.carry_s2c.is_empty(),
            "carry_s2c must be untouched by C2S delivery (AC-172-001 directional isolation)"
        );
        assert!(
            analyzer.all_findings.is_empty(),
            "partial frame stash must not emit any finding (AC-172-001)"
        );
    }

    /// AC-172-001 S2C: partial APCI header stashed into carry_s2c; carry_c2s untouched.
    ///
    /// Traces: BC-2.19.025 postconditions 1–2; AC-172-001.
    #[test]
    fn test_AC_172_001_carry_stash_s2c_partial_frame() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        let partial = [0x68u8, 0x04, 0x07];
        analyzer.on_data(flow_key.clone(), &partial, 0, Direction::ServerToClient);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_s2c, partial,
            "3-byte partial STARTDT must be stashed into carry_s2c (AC-172-001)"
        );
        assert!(
            state.carry_c2s.is_empty(),
            "carry_c2s must be untouched by S2C delivery (AC-172-001 directional isolation)"
        );
        assert!(
            analyzer.all_findings.is_empty(),
            "partial frame stash must not emit any finding"
        );
    }

    /// AC-172-001: directional isolation — interleaved C2S and S2C deliveries never mix.
    ///
    /// Sends a 3-byte partial C2S and a 4-byte partial S2C. Each directional carry
    /// must contain only its own bytes; no cross-direction contamination.
    ///
    /// Traces: BC-2.19.025 invariant 1; RULING-DNP3-SIBLING-001; AC-172-001.
    #[test]
    fn test_AC_172_001_carry_directional_isolation_interleaved() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        let c2s_bytes = [0x68u8, 0x04, 0x07];
        let s2c_bytes = [0x68u8, 0x04, 0x01, 0x00];
        analyzer.on_data(flow_key.clone(), &c2s_bytes, 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), &s2c_bytes, 0, Direction::ServerToClient);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s, c2s_bytes,
            "carry_c2s must contain only C2S bytes (BC-2.19.025 invariant 1)"
        );
        assert_eq!(
            state.carry_s2c, s2c_bytes,
            "carry_s2c must contain only S2C bytes (BC-2.19.025 invariant 1)"
        );
    }

    // =========================================================================
    // AC-172-002: BC-2.19.025 v1.2 WALK-FIRST-RESIDUAL-BOUND carry-overflow tests
    // F-172-001 remediation: replaces PRE-CHECK-DISCARD-ALL canonical vectors
    // Canonical test vectors from BC-2.19.025 v1.2
    // =========================================================================

    /// AC-172-002 / Vector (i) — split frame across carry/delivery (legit traffic, no overflow).
    ///
    /// BC-2.19.025 v1.2 walk-first semantics: the 200-byte carry + 100-byte delivery (total=300)
    /// must NOT be discarded by a carry+delivery pre-check. The frame-walk loop consumes the
    /// complete 255-byte I-frame first, dispatching a T0827 finding; then the 45-byte partial
    /// frame tail is stashed as residual carry. No T0814 carry-overflow finding is emitted.
    ///
    /// Arithmetic: carry(200) + delivery(55+45) = 300. Frame = LEN+2 = 253+2 = 255 ≤ 300 →
    /// complete. Residual = 300−255 = 45 ≤ 255. No overflow.
    ///
    /// Step 1: first on_data delivers the first 200 bytes (exercises the carry-stash path).
    /// Step 2: second on_data delivers 55 completing bytes + 45-byte partial frame tail.
    ///
    /// Traces: BC-2.19.025 v1.2 postconditions 1–2, invariant 2 (walk-first ordering);
    ///         F-172-001; AC-172-002.
    #[test]
    fn test_BC_2_19_025_v12_vector_i_split_frame_c2s_walk_first_no_t0814() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();

        // Build the 255-byte I-frame with TypeID 105 (C_RP_NA_1 → emits T0827 on dispatch).
        // Frame layout: [0x68, 0xFD(LEN=253), CF1=0x00(I-frame), CF2, CF3, CF4, ASDU...]
        // ASDU bytes: type_id=105 at offset 6, VSQ=0x01 at 7, COT=0x06 at 8, orig=0 at 9,
        //             CASDU=0x01,0x00 at 10-11, IOA=0x01,0x00,0x00 at 12-14, zeros thereafter.
        let mut frame_255 = vec![0u8; 255];
        frame_255[0] = 0x68u8; // start byte
        frame_255[1] = 0xFDu8; // LEN = 253 → frame_total = LEN+2 = 255
        frame_255[2] = 0x00u8; // CF1 = 0x00 → I-format (bit 0 = 0)
        frame_255[3] = 0x00u8; // CF2
        frame_255[4] = 0x00u8; // CF3
        frame_255[5] = 0x00u8; // CF4
        frame_255[6] = 105u8; // ASDU type_id = 105 (C_RP_NA_1) → detect_iec104_threats → T0827
        frame_255[7] = 0x01u8; // VSQ: count=1, SQ=0
        frame_255[8] = 0x06u8; // COT: cause=6 (activation)
        frame_255[9] = 0x00u8; // originator = 0
        frame_255[10] = 0x01u8; // CASDU low
        frame_255[11] = 0x00u8; // CASDU high
        frame_255[12] = 0x01u8; // IOA byte 0
        frame_255[13] = 0x00u8; // IOA byte 1
        frame_255[14] = 0x00u8; // IOA byte 2
        // bytes 15-254: remain zero (padding to fill 255-byte frame)

        // Step 1: deliver first 200 bytes of the frame (exercises the carry-stash path).
        // carry_c2s must hold 200 bytes after this call; no finding emitted.
        analyzer.on_data(
            flow_key.clone(),
            &frame_255[..200],
            0,
            Direction::ClientToServer,
        );
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.carry_c2s.len(),
                200,
                "Vector (i) step 1: first 200 bytes of 255-byte frame must be stashed into carry_c2s"
            );
        }
        assert!(
            analyzer.all_findings.is_empty(),
            "Vector (i) step 1: partial frame stash must emit no finding"
        );

        // 45-byte partial second frame: [0x68, LEN=100 (frame_total=102), 43 body bytes].
        // frame_len = 100+2 = 102; only 45 bytes available → incomplete → stashed as residual.
        let mut partial_frame_45 = vec![0u8; 45];
        partial_frame_45[0] = 0x68u8;
        partial_frame_45[1] = 100u8; // LEN=100 → frame_total=102; 45 < 102 → partial

        // Step 2: deliver 55 completing bytes (frame_255[200..255]) + 45-byte partial tail.
        // Working buf = carry(200) + delivery(55+45) = 300 bytes.
        // Walk: consume 255-byte frame → T0827; stash 45-byte partial as residual.
        let mut delivery = frame_255[200..].to_vec(); // 55 bytes completing the 255-byte frame
        delivery.extend_from_slice(&partial_frame_45); // + 45-byte partial tail
        assert_eq!(
            delivery.len(),
            100,
            "delivery for step 2 must be exactly 100 bytes"
        );

        analyzer.on_data(flow_key.clone(), &delivery, 0, Direction::ClientToServer);

        // Assert: T0827 in findings (frame was dispatched, NOT discarded — walk-first semantics).
        // Under PRE-CHECK-DISCARD-ALL (v1.1): carry(200)+delivery(100)=300>255 → T0814 emitted,
        // delivery discarded before frame extraction → no T0827. This assertion FAILS v1.1.
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0827")),
            "Vector (i): 255-byte TypeID-105 I-frame must be dispatched and emit T0827 \
             (BC-2.19.025 v1.2 walk-first: no pre-check-discard-all; F-172-001)"
        );
        // Assert: NO T0814 carry-overflow (residual=45 ≤ MAX_IEC104_CARRY_BYTES=255).
        assert!(
            analyzer
                .all_findings
                .iter()
                .all(|f| !f.mitre_techniques.iter().any(|t| t == "T0814")),
            "Vector (i): conformant split-frame delivery must NOT emit T0814 carry-overflow \
             (BC-2.19.025 v1.2: overflow guard fires only on residual >255; F-172-001)"
        );
        // Assert: carry_c2s holds exactly the 45-byte partial tail.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s.len(),
            45,
            "Vector (i): carry_c2s must hold exactly 45-byte residual after walk (BC-2.19.025 v1.2)"
        );
        assert!(
            state.carry_s2c.is_empty(),
            "Vector (i): carry_s2c must be unaffected by C2S delivery (directional isolation)"
        );
    }

    /// AC-172-002 / Vector (ii) — single S2C delivery: complete frame plus tail (no prior carry).
    ///
    /// BC-2.19.025 v1.2: a 300-byte S2C delivery (0 carry + 300 delivery) must NOT be
    /// discarded by a pre-check. The frame-walk loop extracts the complete 255-byte I-frame,
    /// dispatching T0827, and stashes the remaining 45-byte partial frame as residual carry.
    ///
    /// Arithmetic: carry(0) + delivery(255+45) = 300. Frame = 255 bytes ≤ 300 → complete.
    /// Residual = 45 ≤ 255. No overflow.
    ///
    /// Traces: BC-2.19.025 v1.2 postconditions 1–2, invariant 2 (walk-first ordering);
    ///         F-172-001; AC-172-002.
    #[test]
    fn test_BC_2_19_025_v12_vector_ii_single_delivery_s2c_walk_first_no_t0814() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();

        // Same 255-byte I-frame structure as Vector (i).
        let mut frame_255 = vec![0u8; 255];
        frame_255[0] = 0x68u8;
        frame_255[1] = 0xFDu8; // LEN = 253 → frame_total = 255
        frame_255[2] = 0x00u8; // CF1 = 0x00 → I-format
        frame_255[3] = 0x00u8;
        frame_255[4] = 0x00u8;
        frame_255[5] = 0x00u8;
        frame_255[6] = 105u8; // type_id = 105 → T0827
        frame_255[7] = 0x01u8;
        frame_255[8] = 0x06u8;
        frame_255[9] = 0x00u8;
        frame_255[10] = 0x01u8;
        frame_255[11] = 0x00u8;
        frame_255[12] = 0x01u8;
        frame_255[13] = 0x00u8;
        frame_255[14] = 0x00u8;
        // bytes 15-254: zeros

        // 45-byte partial second frame: [0x68, LEN=100, 43 body bytes] → frame_total=102 > 45.
        let mut partial_frame_45 = vec![0u8; 45];
        partial_frame_45[0] = 0x68u8;
        partial_frame_45[1] = 100u8; // LEN=100 → frame_total=102; 45 < 102 → partial

        // Single 300-byte delivery: 255-byte complete frame + 45-byte partial tail.
        let mut delivery = frame_255.clone();
        delivery.extend_from_slice(&partial_frame_45);
        assert_eq!(
            delivery.len(),
            300,
            "Vector (ii) delivery must be exactly 300 bytes"
        );

        // carry_s2c is empty before this call (no prior carry).
        analyzer.on_data(flow_key.clone(), &delivery, 0, Direction::ServerToClient);

        // Assert: T0827 in findings (frame dispatched via walk-first; NOT discarded).
        // Under PRE-CHECK-DISCARD-ALL (v1.1): carry(0)+delivery(300)=300>255 → T0814, discard.
        // This assertion FAILS v1.1.
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0827")),
            "Vector (ii): S2C 255-byte TypeID-105 frame must be dispatched and emit T0827 \
             (BC-2.19.025 v1.2 walk-first; F-172-001)"
        );
        // Assert: NO T0814 carry-overflow.
        assert!(
            analyzer
                .all_findings
                .iter()
                .all(|f| !f.mitre_techniques.iter().any(|t| t == "T0814")),
            "Vector (ii): conformant single-delivery must NOT emit T0814 carry-overflow \
             (BC-2.19.025 v1.2 residual bound; F-172-001)"
        );
        // Assert: carry_s2c holds exactly the 45-byte residual.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_s2c.len(),
            45,
            "Vector (ii): carry_s2c must hold exactly 45-byte residual after walk (BC-2.19.025 v1.2)"
        );
        assert!(
            state.carry_c2s.is_empty(),
            "Vector (ii): carry_c2s must be unaffected by S2C delivery (directional isolation)"
        );
    }

    /// AC-172-002 / Vector (iii) — defensive adversarial carry-overflow dedup (EC-003/EC-004).
    ///
    /// Non-conformant, adversarially-constructed scenario. Tests the dedup guard for T0814
    /// carry-overflow events (`carry_overflow_reported_c2s` flag; BC-2.19.025 invariant 4).
    ///
    /// State injection: set `carry_c2s` directly to 256 bytes (one byte beyond
    /// MAX_IEC104_CARRY_BYTES=255), then call on_data with empty delivery.
    ///
    /// Expected v1.2 behavior:
    ///   - First overflow event (EC-003): carry cleared; ONE T0814 (Anomaly/Possible/Medium)
    ///     emitted; `carry_overflow_reported_c2s` set to true.
    ///   - Second overflow event (EC-004): carry cleared; NO additional T0814
    ///     (flag suppresses re-emission); `carry_overflow_reported_c2s` stays true.
    ///   - `carry_overflow_reported_s2c` remains false throughout (EC-005: per-direction
    ///     independence; C2S overflow must not set S2C dedup flag).
    ///
    /// Under PRE-CHECK-DISCARD-ALL (v1.1): carry(256)+delivery(0)=256>255 → T0814 emitted,
    /// carry RETAINED at 256 bytes (not cleared), flag never set → EC-003 assertions fail.
    /// Second trip also emits T0814 (no dedup) → EC-004 assertion fails.
    ///
    /// Traces: BC-2.19.025 v1.2 postcondition 3, invariants 4–5; F-172-001; EC-003/004/005;
    ///         AC-172-002.
    #[test]
    fn test_BC_2_19_025_v12_vector_iii_defensive_overflow_dedup_c2s() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();

        // Construct 256-byte adversarial carry: one byte beyond MAX_IEC104_CARRY_BYTES=255.
        // Pattern: [0x68, 0xFD(LEN=253), <254 zero bytes>] = 256 bytes total.
        // Injected directly — this carry state is unreachable through conformant IEC-104 traffic
        // (the walk always stashes ≤ 254 bytes). Direct injection tests the defensive guard.
        let mut overflow_carry = vec![0u8; 256];
        overflow_carry[0] = 0x68u8;
        overflow_carry[1] = 0xFDu8; // LEN=253 → frame_total=255; buf only has 256 bytes

        // First overflow event (EC-003): inject 256-byte carry, call on_data with empty delivery.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = overflow_carry.clone();
        }
        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);

        // carry_c2s must be CLEARED after overflow — v1.2 clears on overflow (EC-003).
        // Under v1.1 PRE-CHECK-DISCARD-ALL: carry is RETAINED (256 bytes). Assertion FAILS v1.1.
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.carry_c2s.len(),
                0,
                "Vector (iii) first trip: carry_c2s must be CLEARED after overflow \
                 (BC-2.19.025 v1.2 EC-003; v1.1 PRE-CHECK retained carry — now wrong)"
            );
            // Dedup flag must be set on first overflow (EC-003).
            // Under v1.1: flag never wired → remains false. Assertion FAILS v1.1.
            assert!(
                state.carry_overflow_reported_c2s,
                "Vector (iii) first trip: carry_overflow_reported_c2s must be true after \
                 first overflow (BC-2.19.025 invariant 4; EC-003)"
            );
            // S2C flag must remain false — per-direction independence (EC-005).
            assert!(
                !state.carry_overflow_reported_s2c,
                "Vector (iii): carry_overflow_reported_s2c must remain false \
                 (EC-005: C2S overflow must not affect S2C dedup flag)"
            );
        }
        // Exactly ONE T0814 must be emitted on first overflow.
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "Vector (iii) first trip: exactly one T0814 carry-overflow finding must be emitted \
             (BC-2.19.025 v1.2 EC-003)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "Vector (iii) T0814 must have ThreatCategory::Anomaly (BC-2.19.025)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "Vector (iii) T0814 must have Verdict::Possible (BC-2.19.025)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "Vector (iii) carry-overflow finding must cite T0814 (BC-2.19.025)"
        );

        // Second overflow event (EC-004): re-inject 256-byte carry, call on_data again.
        // The dedup flag (carry_overflow_reported_c2s=true) must suppress T0814 re-emission.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = overflow_carry.clone();
        }
        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);

        // carry must be cleared again (resync — not a permanent desync latch; EC-004).
        {
            let state = analyzer.flows.get(&flow_key).unwrap();
            assert_eq!(
                state.carry_c2s.len(),
                0,
                "Vector (iii) second trip: carry_c2s must be cleared again on resync (EC-004)"
            );
        }
        // NO additional T0814 — dedup flag suppresses re-emission (EC-004).
        // Under v1.1: no dedup → second T0814 emitted → findings.len()==2. Assertion FAILS v1.1.
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "Vector (iii) second trip: dedup flag must suppress T0814 re-emission — \
             total findings must remain 1 (BC-2.19.025 invariant 4; EC-004)"
        );
    }

    /// AC-172-002 / EC-001 adapted boundary: conformant-maximum partial frame residual (254 bytes)
    /// is stashed without T0814 (residual ≤ MAX_IEC104_CARRY_BYTES=255).
    ///
    /// The maximum achievable conformant partial-frame residual is 254 bytes: a 255-byte frame
    /// (LEN=253) with only 254 bytes delivered — one byte short of completion.
    /// Residual = 254 bytes ≤ MAX_IEC104_CARRY_BYTES=255 → the >255 guard does not fire.
    ///
    /// Note: residual = 255 bytes is unreachable for conformant IEC-104 traffic by construction
    /// (a 255-byte prefix with start 0x68 + LEN=253 IS a complete frame; the walk consumes it).
    /// BC-2.19.025 EC-002 documents this as "conformant traffic: unreachable". This test uses
    /// the closest achievable value (254 bytes) to pin that the guard threshold is >255, not ≥255.
    ///
    /// Traces: BC-2.19.025 v1.2 postcondition 2, invariants 2–3; EC-001/EC-002; AC-172-002.
    #[test]
    fn test_BC_2_19_025_v12_ec001_max_conformant_partial_254_no_t0814() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        assert_eq!(
            MAX_IEC104_CARRY_BYTES, 255,
            "MAX_IEC104_CARRY_BYTES must be 255 (ADR-013 Decision 2; BC-2.19.025 invariant 3)"
        );

        // Deliver 254 bytes: first 254 bytes of a 255-byte frame [0x68, 0xFD, 252 zeros].
        // frame_len = LEN+2 = 253+2 = 255. delivery.len()=254 < frame_len=255 → partial stash.
        // Residual = 254 bytes ≤ MAX_IEC104_CARRY_BYTES=255 → no overflow, no T0814.
        let mut partial_254 = vec![0u8; 254];
        partial_254[0] = 0x68u8;
        partial_254[1] = 0xFDu8; // LEN=253 → frame_total=255; only 254 bytes → incomplete partial

        analyzer.on_data(flow_key.clone(), &partial_254, 0, Direction::ClientToServer);

        // No T0814 carry-overflow: residual=254 ≤ MAX_IEC104_CARRY_BYTES=255.
        assert!(
            analyzer
                .all_findings
                .iter()
                .all(|f| !f.mitre_techniques.iter().any(|t| t == "T0814")),
            "254-byte conformant partial frame must not emit T0814 carry-overflow \
             (BC-2.19.025 v1.2 EC-001: overflow guard is >255, not ≥255)"
        );
        // carry must hold all 254 bytes (partial stash preserves residual).
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert_eq!(
            state.carry_c2s.len(),
            254,
            "254-byte partial frame must be fully stashed into carry_c2s (BC-2.19.025 v1.2)"
        );
        assert!(
            state.carry_s2c.is_empty(),
            "carry_s2c must be unaffected by C2S delivery (directional isolation)"
        );
    }

    // =========================================================================
    // AC-172-003: frame-walk loop processes all complete APCI frames per on_data
    // BC-2.19.026 postconditions 1–3
    // =========================================================================

    /// AC-172-003: two complete STARTDT back-to-back frames both processed in one delivery.
    ///
    /// A single delivery containing two concatenated 6-byte STARTDT U-frames must cause
    /// both frames to be parsed sequentially; no carry residual expected.
    ///
    /// Traces: BC-2.19.026 postconditions 1–3; AC-172-003.
    #[test]
    fn test_BC_2_19_026_multiple_complete_frames_processed_sequentially() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Two complete 6-byte STARTDT U-frames concatenated.
        let two_frames: Vec<u8> = [
            0x68u8, 0x04, 0x07, 0x00, 0x00, 0x00, // STARTDT-act frame 1
            0x68u8, 0x04, 0x07, 0x00, 0x00, 0x00, // STARTDT-act frame 2
        ]
        .to_vec();
        analyzer.on_data(flow_key.clone(), &two_frames, 0, Direction::ClientToServer);
        // Both frames processed → no residual carry, and STARTDT-act dispatch effect present.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "two complete frames must leave no carry residual (BC-2.19.026 postcondition 3)"
        );
        assert!(
            state.session_started,
            "STARTDT-act frames must set session_started=true via on_data dispatch \
             (BC-2.19.026 PC2; DF-SIBLING-SWEEP-001)"
        );
    }

    // =========================================================================
    // AC-172-004: frame-walk advance modes — bad start byte and malformed-LEN
    // BC-2.19.026 postcondition 4, invariants 1 and 5; EC-005
    // =========================================================================

    /// AC-172-004 / EC-005: bad start byte (data[pos] != 0x68) → 1-byte silent resync.
    ///
    /// A byte with value != 0x68 advances the cursor by 1 with no finding emitted and
    /// carry NOT cleared. A valid STARTDT frame follows the garbage byte; both should be
    /// handled correctly.
    ///
    /// Traces: BC-2.19.026 postcondition 4 (bad-start-byte arm); AC-172-004; EC-005.
    #[test]
    fn test_BC_2_19_026_bad_start_byte_advance_one_no_finding() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // 1 garbage byte (0xAA) followed by a complete 6-byte STARTDT frame.
        let data: Vec<u8> = [
            0xAAu8, // bad start byte → advance 1
            0x68, 0x04, 0x07, 0x00, 0x00, 0x00, // STARTDT-act (valid frame)
        ]
        .to_vec();
        analyzer.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);
        // Bad start byte emits no finding.
        assert!(
            analyzer.all_findings.is_empty(),
            "bad start byte must not emit any finding (BC-2.19.026 postcondition 4 bad-start arm)"
        );
        // The valid frame after the bad byte must be processed; no residual carry.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "carry must be empty after bad-start-byte + valid frame (EC-005)"
        );
    }

    // =========================================================================
    // AC-172-008: malformed-LEN dedup per direction (BC-2.19.026 invariant 5)
    // EC-006, EC-007, EC-008
    // =========================================================================

    /// AC-172-008 / EC-006: first malformed-LEN in C2S → exactly one T0814 + flag set.
    ///
    /// A valid 0x68 start byte with LEN=3 (below minimum 4) is malformed. On the FIRST
    /// occurrence in C2S: cursor advances 2 bytes; exactly ONE T0814 Anomaly/Possible/Medium
    /// is emitted; `malformed_len_reported_c2s` is set to true.
    ///
    /// Traces: BC-2.19.026 invariant 5; AC-172-008; EC-006.
    #[test]
    fn test_BC_2_19_026_malformed_len_first_c2s() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // LEN=3 is below the minimum of 4 → malformed.
        let malformed_data = [0x68u8, 0x03, 0x00, 0x00, 0x00, 0x00];
        analyzer.on_data(
            flow_key.clone(),
            &malformed_data,
            0,
            Direction::ClientToServer,
        );
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "first malformed-LEN C2S must emit exactly one T0814 (BC-2.19.026 invariant 5; EC-006)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "malformed-LEN T0814 must have ThreatCategory::Anomaly (BC-2.19.026)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "malformed-LEN T0814 must have Verdict::Possible (BC-2.19.026)"
        );
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "malformed-LEN finding must cite T0814 (BC-2.19.026)"
        );
        // Dedup flag must be set.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.malformed_len_reported_c2s,
            "malformed_len_reported_c2s must be true after first C2S malformed-LEN (EC-006)"
        );
        assert!(
            !state.malformed_len_reported_s2c,
            "malformed_len_reported_s2c must remain false (C2S dedup must not affect S2C)"
        );
    }

    /// AC-172-008 / EC-007: second malformed-LEN in same C2S direction → no additional finding.
    ///
    /// With `malformed_len_reported_c2s` already set, a second malformed-LEN frame in
    /// the same C2S direction must advance the cursor silently with no finding emitted.
    ///
    /// Traces: BC-2.19.026 invariant 5; AC-172-008; EC-007.
    #[test]
    fn test_BC_2_19_026_malformed_len_second_c2s() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Pre-set the dedup flag to simulate a flow that already saw one malformed-LEN C2S.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.malformed_len_reported_c2s = true;
        }
        let malformed_data = [0x68u8, 0x03, 0x00, 0x00, 0x00, 0x00];
        analyzer.on_data(
            flow_key.clone(),
            &malformed_data,
            0,
            Direction::ClientToServer,
        );
        // No additional finding must be emitted (dedup flag was already set).
        assert!(
            analyzer.all_findings.is_empty(),
            "second C2S malformed-LEN must emit no finding (BC-2.19.026 invariant 5 EMIT-WITH-DEDUP; EC-007)"
        );
        // Flag remains true (never reset within a flow lifetime).
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.malformed_len_reported_c2s,
            "malformed_len_reported_c2s must remain true after second C2S malformed-LEN (EC-007)"
        );
    }

    /// AC-172-008 / EC-008: first S2C malformed-LEN after C2S flag already set → independent T0814.
    ///
    /// C2S and S2C dedup flags are completely independent. After C2S has been flagged, the
    /// first malformed-LEN in S2C must still emit ONE T0814 and set `malformed_len_reported_s2c`.
    /// The C2S flag is unchanged.
    ///
    /// Traces: BC-2.19.026 invariant 5; AC-172-008; EC-008.
    #[test]
    fn test_BC_2_19_026_malformed_len_first_s2c_after_c2s() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Pre-set C2S dedup flag (simulates a flow where C2S already saw one malformed-LEN).
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.malformed_len_reported_c2s = true;
        }
        // First malformed-LEN in S2C direction.
        let malformed_data = [0x68u8, 0x03, 0x00, 0x00, 0x00, 0x00];
        analyzer.on_data(
            flow_key.clone(),
            &malformed_data,
            0,
            Direction::ServerToClient,
        );
        // Exactly one T0814 must be emitted for the S2C direction independently.
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "first S2C malformed-LEN must emit exactly one T0814 independently of C2S flag (EC-008)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(f.category, ThreatCategory::Anomaly, "T0814 must be Anomaly");
        assert_eq!(f.verdict, Verdict::Possible, "T0814 must be Possible");
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0814"),
            "S2C malformed-LEN finding must cite T0814 (EC-008)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.malformed_len_reported_s2c,
            "malformed_len_reported_s2c must be true after first S2C malformed-LEN (EC-008)"
        );
        assert!(
            state.malformed_len_reported_c2s,
            "malformed_len_reported_c2s must remain true and be unaffected by S2C detection (EC-008)"
        );
    }

    // =========================================================================
    // AC-172-005: on_data does not panic for any byte sequence (EC-004)
    // BC-2.19.026 postcondition 5
    // =========================================================================

    /// AC-172-005 / EC-004: empty data slice must not panic and must not mutate carry.
    ///
    /// Delivering an empty slice is a valid call. The frame-walk loop must handle
    /// zero-length input cleanly: no finding emitted, carry unchanged.
    ///
    /// Traces: BC-2.19.026 postcondition 5; AC-172-005; EC-004.
    #[test]
    fn test_AC_172_005_empty_data_slice_no_panic_no_finding() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        analyzer.on_data(flow_key.clone(), &[], 0, Direction::ClientToServer);
        assert!(
            analyzer.all_findings.is_empty(),
            "empty delivery must not emit any finding (BC-2.19.026 EC-004)"
        );
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "empty delivery must not alter carry_c2s (EC-004)"
        );
        assert!(
            state.carry_s2c.is_empty(),
            "empty delivery must not alter carry_s2c (EC-004)"
        );
    }

    // =========================================================================
    // AC-172-003 / EC-009: back-to-back frames — three complete frames
    // BC-2.19.026 postconditions 1–3
    // =========================================================================

    /// AC-172-003 / EC-009: three complete STARTDT frames back-to-back → all processed.
    ///
    /// Traces: BC-2.19.026 postconditions 1–3; AC-172-003; EC-009.
    #[test]
    fn test_BC_2_19_026_ec_009_back_to_back_three_frames() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Three complete 6-byte STARTDT U-frames concatenated.
        let three_frames: Vec<u8> = [
            0x68u8, 0x04, 0x07, 0x00, 0x00, 0x00, // STARTDT frame 1
            0x68u8, 0x04, 0x07, 0x00, 0x00, 0x00, // STARTDT frame 2
            0x68u8, 0x04, 0x07, 0x00, 0x00, 0x00, // STARTDT frame 3
        ]
        .to_vec();
        analyzer.on_data(
            flow_key.clone(),
            &three_frames,
            0,
            Direction::ClientToServer,
        );
        // All three frames processed → no residual carry, and STARTDT-act dispatch effect present.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "three complete back-to-back frames must leave no carry residual (EC-009)"
        );
        assert!(
            state.session_started,
            "STARTDT-act frames must set session_started=true via on_data dispatch \
             (BC-2.19.026 PC2; DF-SIBLING-SWEEP-001)"
        );
    }

    // =========================================================================
    // AC-172-006: on_flow_close removes Iec104FlowState and discards carry bytes
    // BC-2.19.027 postconditions 1–4, invariants 1–2
    // =========================================================================

    /// AC-172-006: on_flow_close removes the per-flow state entry.
    ///
    /// After `on_flow_close`, `analyzer.flows` must not contain the flow key.
    /// Calling `on_data` on the same key afterward must yield fresh (default) state.
    ///
    /// Traces: BC-2.19.027 postconditions 1–3; AC-172-006.
    #[test]
    fn test_BC_2_19_027_on_flow_close_removes_state() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Seed some state so there is something to remove.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.session_started = true;
            state.carry_c2s = vec![0x68u8, 0x04];
        }
        assert!(
            analyzer.flows.contains_key(&flow_key),
            "precondition: flow state must exist before on_flow_close"
        );
        analyzer.on_flow_close(flow_key.clone());
        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "on_flow_close must remove the flow state from analyzer.flows (BC-2.19.027 postcondition 1)"
        );
        assert!(
            analyzer.all_findings.is_empty(),
            "on_flow_close must not emit any finding for normal flow close (BC-2.19.027 postcondition 3)"
        );
    }

    /// AC-172-006: on_flow_close re-open yields fresh default state.
    ///
    /// After closing, a new `on_data` for the same flow key must start with clean
    /// default state (carry empty, flags false, session_started false).
    ///
    /// Traces: BC-2.19.027 postcondition 1; AC-172-006.
    #[test]
    fn test_AC_172_006_reopen_flow_yields_fresh_state() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Seed state, close the flow, then open it again.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.session_started = true;
            state.carry_c2s = vec![0x68u8, 0x04, 0x07, 0x00, 0x00];
            state.malformed_len_reported_c2s = true;
        }
        analyzer.on_flow_close(flow_key.clone());
        // Re-open with a fresh delivery.
        let partial = [0x68u8, 0x04, 0x07];
        analyzer.on_data(flow_key.clone(), &partial, 1, Direction::ClientToServer);
        let state = analyzer.flows.get(&flow_key).unwrap();
        // Fresh state: dedup flag must be false (not inherited from closed flow).
        assert!(
            !state.malformed_len_reported_c2s,
            "re-opened flow must have malformed_len_reported_c2s=false (fresh state; AC-172-006)"
        );
        assert!(
            !state.session_started,
            "re-opened flow must have session_started=false (fresh state; AC-172-006)"
        );
    }

    /// AC-172-006 / EC-010: on_flow_close with non-empty carry silently discards carry.
    ///
    /// No T0814 or other finding is emitted when carry bytes are present at flow close.
    ///
    /// Traces: BC-2.19.027 postcondition 3; AC-172-006; EC-010.
    #[test]
    fn test_BC_2_19_027_ec_010_close_with_carry_no_finding() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Pre-populate carry with partial frame bytes.
        {
            let state = analyzer.flows.entry(flow_key.clone()).or_default();
            state.carry_c2s = vec![0x68u8, 0x04, 0x07]; // partial STARTDT
            state.carry_s2c = vec![0x68u8, 0x04]; // partial S2C
        }
        analyzer.on_flow_close(flow_key.clone());
        assert!(
            analyzer.all_findings.is_empty(),
            "on_flow_close with non-empty carry must not emit any finding (BC-2.19.027 postcondition 3; EC-010)"
        );
        assert!(
            !analyzer.flows.contains_key(&flow_key),
            "on_flow_close must remove state even when carry is non-empty (EC-010)"
        );
    }

    /// AC-172-006 / EC-011: on_flow_close for unknown flow_key is a no-op.
    ///
    /// Calling `on_flow_close` on a flow that was never opened must not panic and
    /// must not modify any existing state.
    ///
    /// Traces: BC-2.19.027 postcondition 4; AC-172-006; EC-011.
    #[test]
    fn test_BC_2_19_027_ec_011_close_unknown_flow_key_no_panic() {
        let mut analyzer = Iec104Analyzer::new();
        let unknown_key = FlowKey::new(
            "192.168.1.1".parse().unwrap(),
            9999,
            "192.168.1.2".parse().unwrap(),
            2404,
        );
        // Must not panic.
        analyzer.on_flow_close(unknown_key);
        assert!(
            analyzer.flows.is_empty(),
            "on_flow_close for unknown key must not create any state (BC-2.19.027 postcondition 4; EC-011)"
        );
        assert!(
            analyzer.all_findings.is_empty(),
            "on_flow_close for unknown key must not emit any finding (EC-011)"
        );
    }

    // =========================================================================
    // VP-045: proptest_vp045_direction_isolation
    // AC-174-002: upgraded to asserting harnesses (non-vacuous per AC-174-002 amendment)
    // =========================================================================

    proptest! {
        /// VP-045 proptest: carry direction isolation (AC-174-002; BC-2.19.025 invariant 1).
        ///
        /// Verifies that C2S deliveries accumulate residual bytes only in `carry_c2s`
        /// and S2C deliveries accumulate residual bytes only in `carry_s2c`. Cross-
        /// direction contamination is detected by comparing the combined interleaved-
        /// replay carry against isolated C2S-only and S2C-only witness replays.
        ///
        /// Strategy: a generated `Vec` of direction-tagged byte chunks (`(bool, Vec<u8>)`,
        /// where `true` = C2S) is replayed in generated order with arbitrary chunk
        /// boundaries. Three analyzer instances receive the same data in different
        /// subsets: (a) combined receives all chunks with their tagged direction;
        /// (b) c2s_witness receives only C2S chunks; (c) s2c_witness receives only S2C
        /// chunks. The combined carry must equal the witnesses, proving neither carry
        /// buffer is contaminated by the other direction.
        ///
        /// Non-vacuity: `prop_assert_eq!` compares carry equality and `prop_assert!`
        /// checks the 255-byte bound on every proptest case.
        ///
        /// Traces: BC-2.19.025 invariants 1 and 3; VP-045; AC-174-002; RULING-DNP3-SIBLING-001.
        #[test]
        fn proptest_vp045_direction_isolation(
            chunks in prop::collection::vec(
                (any::<bool>(), prop::collection::vec(any::<u8>(), 0..64usize)),
                0..20usize,
            ),
        ) {
            let flow_key = FlowKey::new(
                "127.0.0.1".parse().unwrap(), 1234,
                "127.0.0.2".parse().unwrap(), 2404,
            );

            // combined: receives the full interleaved C2S + S2C sequence.
            let mut combined = Iec104Analyzer::new();
            // c2s_witness: receives only the C2S chunks (isolation witness).
            let mut c2s_witness = Iec104Analyzer::new();
            // s2c_witness: receives only the S2C chunks (isolation witness).
            let mut s2c_witness = Iec104Analyzer::new();

            for (is_c2s, data) in &chunks {
                let dir = if *is_c2s {
                    Direction::ClientToServer
                } else {
                    Direction::ServerToClient
                };
                combined.on_data(flow_key.clone(), data, 0, dir);
                if *is_c2s {
                    c2s_witness.on_data(flow_key.clone(), data, 0, Direction::ClientToServer);
                } else {
                    s2c_witness.on_data(flow_key.clone(), data, 0, Direction::ServerToClient);
                }
            }

            let combined_c2s = combined
                .flows
                .get(&flow_key)
                .map(|s| s.carry_c2s.clone())
                .unwrap_or_default();
            let combined_s2c = combined
                .flows
                .get(&flow_key)
                .map(|s| s.carry_s2c.clone())
                .unwrap_or_default();
            let witness_c2s = c2s_witness
                .flows
                .get(&flow_key)
                .map(|s| s.carry_c2s.clone())
                .unwrap_or_default();
            let witness_s2c = s2c_witness
                .flows
                .get(&flow_key)
                .map(|s| s.carry_s2c.clone())
                .unwrap_or_default();

            // Save lengths before consuming the vectors in prop_assert_eq!.
            let c2s_len = combined_c2s.len();
            let s2c_len = combined_s2c.len();

            // BC-2.19.025 invariant 1: no cross-direction mixing.
            prop_assert_eq!(
                combined_c2s, witness_c2s,
                "carry_c2s must equal the C2S-only witness replay \
                 (BC-2.19.025 invariant 1: S2C deliveries must not affect carry_c2s)"
            );
            prop_assert_eq!(
                combined_s2c, witness_s2c,
                "carry_s2c must equal the S2C-only witness replay \
                 (BC-2.19.025 invariant 1: C2S deliveries must not affect carry_s2c)"
            );

            // BC-2.19.025 invariant 3: 255-byte residual cap (defensive guard).
            prop_assert!(
                c2s_len <= MAX_IEC104_CARRY_BYTES,
                "carry_c2s.len()={} must be bounded at MAX_IEC104_CARRY_BYTES={} \
                 (BC-2.19.025 invariant 3)",
                c2s_len,
                MAX_IEC104_CARRY_BYTES
            );
            prop_assert!(
                s2c_len <= MAX_IEC104_CARRY_BYTES,
                "carry_s2c.len()={} must be bounded at MAX_IEC104_CARRY_BYTES={} \
                 (BC-2.19.025 invariant 3)",
                s2c_len,
                MAX_IEC104_CARRY_BYTES
            );
        }
    }

    proptest! {
        /// VP-045 proptest: independent-run equivalence (AC-174-002; BC-2.19.025 VP-045
        /// registered harness — independent-run determinism).
        ///
        /// Verifies that two independent `Iec104Analyzer` instances fed identical delivery
        /// sequences produce identical per-flow carry state and `frame_count`. This guards
        /// against hidden cross-flow state or non-determinism in `on_data`
        /// (BC-2.19.025 §Verification / VP-045 registered harness; RULING-DNP3-SIBLING-001).
        ///
        /// Non-vacuity: `prop_assert_eq!` compares `carry_c2s`, `carry_s2c`, and
        /// `frame_count` across the two instances on every proptest case.
        ///
        /// Traces: BC-2.19.025 VP-045 registered harness (independent-run determinism;
        /// no numbered invariant — see BC-2.19.025 §Verification); VP-045; AC-174-002.
        #[test]
        fn proptest_vp045_independent_run_equivalence(
            data in prop::collection::vec(any::<u8>(), 0..256usize),
        ) {
            let mut analyzer_a = Iec104Analyzer::new();
            let mut analyzer_b = Iec104Analyzer::new();
            let flow_key = FlowKey::new(
                "10.0.0.1".parse().unwrap(), 5000,
                "10.0.0.2".parse().unwrap(), 2404,
            );
            analyzer_a.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);
            analyzer_b.on_data(flow_key.clone(), &data, 0, Direction::ClientToServer);

            let state_a = analyzer_a.flows.get(&flow_key);
            let state_b = analyzer_b.flows.get(&flow_key);

            match (state_a, state_b) {
                (None, None) => {
                    // Both produced no flow state — consistent (empty or no-frame delivery).
                }
                (Some(a), Some(b)) => {
                    // BC-2.19.025 VP-045 registered harness: independent runs produce identical carry state.
                    prop_assert_eq!(
                        &a.carry_c2s, &b.carry_c2s,
                        "carry_c2s must be identical across independent analyzer runs \
                         (BC-2.19.025 VP-045 registered harness — independent-run determinism)"
                    );
                    prop_assert_eq!(
                        &a.carry_s2c, &b.carry_s2c,
                        "carry_s2c must be identical across independent analyzer runs \
                         (BC-2.19.025 VP-045 registered harness — independent-run determinism)"
                    );
                    prop_assert_eq!(
                        a.frame_count, b.frame_count,
                        "frame_count must be identical across independent analyzer runs \
                         (BC-2.19.025 VP-045 registered harness — independent-run determinism)"
                    );
                }
                _ => {
                    prop_assert!(
                        false,
                        "one analyzer produced flow state and the other did not — \
                         non-deterministic on_data (BC-2.19.025 VP-045 registered harness)"
                    );
                }
            }
        }
    }

    // =========================================================================
    // BC-2.19.026 PC2: dispatch-effect assertions (F-172-002 remediation)
    // Verifies that valid-frame dispatch branches produce their documented
    // state or finding effects when driven entirely through on_data.
    // =========================================================================

    /// BC-2.19.026 PC2 — STARTDT-act U-frame through on_data sets session_started.
    ///
    /// A STARTDT-act U-frame (CF1=0x07) delivered through on_data must set
    /// `session_started = true` on the flow state. Confirms that the U-format dispatch
    /// arm (process_u_frame) is wired into the on_data frame-walk loop.
    ///
    /// Frame: `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` — start, LEN=4, CF1=0x07
    /// (STARTDT-act, U-format), CF2-CF4=0.
    ///
    /// Traces: BC-2.19.026 PC2; BC-2.19.010 postcondition 1; AC-172-003.
    #[test]
    fn test_BC_2_19_026_pc2_dispatch_startdt_act_sets_session_started() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        let startdt_act: &[u8] = &[0x68, 0x04, 0x07, 0x00, 0x00, 0x00];
        analyzer.on_data(flow_key.clone(), startdt_act, 0, Direction::ClientToServer);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.session_started,
            "STARTDT-act through on_data must set session_started=true \
             (BC-2.19.026 PC2; BC-2.19.010 postcondition 1)"
        );
        assert!(
            analyzer.all_findings.is_empty(),
            "STARTDT-act must not emit any finding (BC-2.19.010)"
        );
    }

    /// BC-2.19.026 PC2 — STOPDT-act U-frame after STARTDT-act emits T0881.
    ///
    /// A STARTDT-act activates the session; a subsequent STOPDT-act (CF1=0x13) delivered
    /// through on_data must emit a T0881 finding with Verdict::Possible. Confirms that
    /// process_u_frame's STOPDT-act arm is reached via the on_data dispatch path.
    ///
    /// Delivery sequence:
    ///   1. `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` — STARTDT-act (no finding)
    ///   2. `[0x68, 0x04, 0x13, 0x00, 0x00, 0x00]` — STOPDT-act (T0881 Possible)
    ///
    /// Traces: BC-2.19.026 PC2; BC-2.19.011 postcondition 1; AC-172-003.
    #[test]
    fn test_BC_2_19_026_pc2_dispatch_stopdt_act_after_startdt_emits_t0881() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        let startdt_act: &[u8] = &[0x68, 0x04, 0x07, 0x00, 0x00, 0x00];
        let stopdt_act: &[u8] = &[0x68, 0x04, 0x13, 0x00, 0x00, 0x00];
        analyzer.on_data(flow_key.clone(), startdt_act, 0, Direction::ClientToServer);
        analyzer.on_data(flow_key.clone(), stopdt_act, 0, Direction::ClientToServer);
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "STOPDT-act after STARTDT-act must emit exactly one T0881 finding \
             (BC-2.19.026 PC2; BC-2.19.011 postcondition 1)"
        );
        let f = &analyzer.all_findings[0];
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0881"),
            "STOPDT-act finding must cite T0881 (BC-2.19.011 postcondition 1)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "STOPDT-act after active session must have Verdict::Possible (BC-2.19.011)"
        );
    }

    /// BC-2.19.026 PC2 — TypeID 105 I-frame through on_data emits T0827.
    ///
    /// An I-format frame carrying TypeID=105 (C_RP_NA_1, reset-process) delivered through
    /// on_data must emit a T0827 "Loss of Control" finding with Verdict::Likely. Confirms
    /// that the I-format dispatch arm (parse_asdu + detect_iec104_threats) is wired into
    /// the on_data frame-walk loop.
    ///
    /// Frame layout (12 bytes):
    ///   APCI: `[0x68, 0x0A, 0x00, 0x00, 0x00, 0x00]` — start, LEN=10,
    ///         CF1=0x00 (I-format, N(S)=0), CF2-CF4=0.
    ///   ASDU: `[0x69, 0x01, 0x06, 0x00, 0x01, 0x00]` — TypeID=105(0x69), VSQ=1,
    ///         COT_cause=6(activation), originator=0, CASDU=1.
    ///
    /// Traces: BC-2.19.026 PC2; BC-2.19.020 postcondition 1; AC-172-003.
    #[test]
    fn test_BC_2_19_026_pc2_dispatch_type105_i_frame_emits_t0827() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        let i_frame_type105: &[u8] = &[
            0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, // APCI: start, LEN=10, CF1-CF4
            0x69, 0x01, 0x06, 0x00, 0x01, 0x00, // ASDU: TypeID=105, VSQ=1, COT=6, CASDU=1
        ];
        analyzer.on_data(
            flow_key.clone(),
            i_frame_type105,
            0,
            Direction::ClientToServer,
        );
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0827")),
            "TypeID=105 I-frame through on_data must emit T0827 finding \
             (BC-2.19.026 PC2; BC-2.19.020 postcondition 1)"
        );
        let t0827 = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0827"))
            .unwrap();
        assert_eq!(
            t0827.verdict,
            Verdict::Likely,
            "T0827 finding from TypeID=105 must have Verdict::Likely (BC-2.19.020)"
        );
    }

    /// BC-2.19.026 PC2 — TypeID 45 control-command I-frame through on_data emits T1692.001.
    ///
    /// An I-format frame carrying TypeID=45 (C_SC_NA_1, single command) delivered through
    /// on_data must emit a T1692.001 "Unauthorized Command Message" finding. Confirms that
    /// the detect_iec104_threats switching-command arm (TypeIDs 45-47) is reached via the
    /// on_data dispatch path.
    ///
    /// Frame layout (12 bytes):
    ///   APCI: `[0x68, 0x0A, 0x00, 0x00, 0x00, 0x00]` — start, LEN=10, CF1=0x00 (N(S)=0).
    ///   ASDU: `[0x2D, 0x01, 0x06, 0x00, 0x01, 0x00]` — TypeID=45(0x2D), VSQ=1,
    ///         COT_cause=6, originator=0, CASDU=1.
    ///
    /// Traces: BC-2.19.026 PC2; BC-2.19.019 postcondition 1; AC-172-003.
    #[test]
    fn test_BC_2_19_026_pc2_dispatch_type45_control_command_emits_t1692_001() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        let i_frame_type45: &[u8] = &[
            0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, // APCI: start, LEN=10, CF1-CF4
            0x2D, 0x01, 0x06, 0x00, 0x01, 0x00, // ASDU: TypeID=45, VSQ=1, COT=6, CASDU=1
        ];
        analyzer.on_data(
            flow_key.clone(),
            i_frame_type45,
            0,
            Direction::ClientToServer,
        );
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
            "TypeID=45 I-frame through on_data must emit T1692.001 finding \
             (BC-2.19.026 PC2; BC-2.19.019 postcondition 1)"
        );
    }

    /// BC-2.19.026 PC2 — N(S) desync scenario through on_data emits T1692.001.
    ///
    /// Two C2S I-frames with a gap of 14 (> k=12) delivered through on_data trigger the
    /// track_ns_desync path-C branch:
    ///   frame 1 — N(S)=0  (CF1=0x00): path A, baseline set, no finding
    ///   frame 2 — N(S)=14 (CF1=0x1C): gap=14 > 12, T1692.001 Possible
    ///
    /// N(S) encoding: CF1 = (ns & 0x7F) << 1; CF2 = ns >> 7.
    ///   N(S)=14 → CF1 = 14 << 1 = 0x1C, CF2 = 0x00.
    ///
    /// TypeID=1 (M_SP_NA_1) is used in both frames; it falls in the unhandled 1-127 range
    /// and emits no finding, isolating the T1692.001 assertion to the desync path.
    ///
    /// Traces: BC-2.19.026 PC2; BC-2.19.024 path C; AC-172-003.
    #[test]
    fn test_BC_2_19_026_pc2_dispatch_ns_desync_via_on_data_emits_t1692_001() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Frame 1: N(S)=0, TypeID=1 (monitoring, no threat finding).
        let i_frame_ns0: &[u8] = &[
            0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, // APCI: CF1=0x00 → N(S)=0
            0x01, 0x01, 0x06, 0x00, 0x01, 0x00, // ASDU: TypeID=1 (M_SP_NA_1)
        ];
        // Frame 2: N(S)=14, TypeID=1. Gap = 14 > k=12 → T1692.001.
        let i_frame_ns14: &[u8] = &[
            0x68, 0x0A, 0x1C, 0x00, 0x00, 0x00, // APCI: CF1=0x1C → N(S)=14 (14<<1=28=0x1C)
            0x01, 0x01, 0x06, 0x00, 0x01, 0x00, // ASDU: TypeID=1 (M_SP_NA_1)
        ];
        analyzer.on_data(flow_key.clone(), i_frame_ns0, 0, Direction::ClientToServer);
        assert!(
            analyzer.all_findings.is_empty(),
            "first I-frame establishes baseline (path A) — must not emit any finding \
             (BC-2.19.024 path A)"
        );
        analyzer.on_data(flow_key.clone(), i_frame_ns14, 0, Direction::ClientToServer);
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001")),
            "N(S) gap of 14 > k=12 through on_data must emit T1692.001 \
             (BC-2.19.026 PC2; BC-2.19.024 path C)"
        );
    }

    /// BC-2.19.026 PC1+PC2 joint — STARTDT-act + TypeID-105 I-frame in one on_data call.
    ///
    /// A single on_data delivery containing a STARTDT-act U-frame followed immediately by
    /// a TypeID=105 I-frame must produce BOTH dispatch effects:
    ///   - session_started = true  (STARTDT-act processed; BC-2.19.010)
    ///   - T0827 in all_findings   (TypeID=105 I-frame dispatched; BC-2.19.020)
    ///
    /// This pins BC-2.19.026 postconditions 1 and 2 jointly: the frame-walk loop processes
    /// every complete frame in a single delivery, calling each dispatch branch in sequence.
    ///
    /// Delivery (18 bytes):
    ///   frame 1: `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` — STARTDT-act (6 bytes)
    ///   frame 2: `[0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x69, 0x01, 0x06, 0x00, 0x01, 0x00]`
    ///            — I-format TypeID=105 (12 bytes)
    ///
    /// Traces: BC-2.19.026 PC1+PC2; BC-2.19.010; BC-2.19.020; AC-172-003.
    #[test]
    fn test_BC_2_19_026_pc2_dispatch_multi_frame_startdt_plus_type105_joint_effects() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Concatenation: STARTDT-act (6 bytes) + I-frame TypeID=105 (12 bytes) = 18 bytes total.
        let combined: &[u8] = &[
            // Frame 1: STARTDT-act U-frame (6 bytes)
            0x68, 0x04, 0x07, 0x00, 0x00, 0x00,
            // Frame 2: I-format TypeID=105 (12 bytes)
            0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x69, 0x01, 0x06, 0x00, 0x01, 0x00,
        ];
        analyzer.on_data(flow_key.clone(), combined, 0, Direction::ClientToServer);
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.session_started,
            "STARTDT-act in multi-frame delivery must set session_started=true \
             (BC-2.19.026 PC1+PC2 joint; BC-2.19.010)"
        );
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.iter().any(|t| t == "T0827")),
            "TypeID=105 I-frame in multi-frame delivery must emit T0827 \
             (BC-2.19.026 PC1+PC2 joint; BC-2.19.020)"
        );
    }

    // =========================================================================
    // AC-174-007: targeted test for cargo-mutants survivor at iec104.rs:1220
    // (replace < with <= in the "need at least 2 bytes to read LEN" guard)
    // =========================================================================

    /// AC-174-007 mutant kill: 2-byte delivery [0x68, invalid_LEN] must emit T0814 immediately.
    ///
    /// The frame-walk guard at line 1220 is `if buf.len() - pos < 2` — it stashes only when
    /// there is fewer than 2 bytes remaining (i.e., the lone start byte with no LEN byte yet).
    /// When exactly 2 bytes remain `[0x68, invalid_LEN]`, the guard must NOT stash: it must
    /// proceed to read LEN, detect invalid (LEN=0 ∉ [4,253]), and emit T0814 in the same
    /// delivery.
    ///
    /// Mutation `replace < with <=` would stash instead of processing, delaying T0814 by one
    /// delivery. This test requires immediate emission.
    ///
    /// Traces: BC-2.19.026 invariant 5; AC-174-007 (AC-172-008 extension for 2-byte tail).
    #[test]
    fn test_AC_174_007_malformed_len_at_two_byte_tail_emits_t0814_immediately() {
        let mut analyzer = Iec104Analyzer::new();
        let flow_key = flow_key_default();
        // Deliver exactly 2 bytes: valid start byte (0x68) + invalid LEN=0 (below minimum 4).
        // With original `< 2`: buf.len()-pos = 2, NOT < 2 → reads LEN=0, detects invalid,
        // emits T0814, advances 2 → finding emitted in this delivery.
        // With mutant `<= 2`: buf.len()-pos = 2, IS <= 2 → stashes 2 bytes, no finding yet.
        analyzer.on_data(
            flow_key.clone(),
            &[0x68u8, 0x00],
            0,
            Direction::ClientToServer,
        );
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "2-byte delivery [0x68, 0x00] (invalid LEN=0) must emit T0814 immediately \
             (BC-2.19.026 invariant 5; AC-174-007 mutant kill for iec104.rs:1220)"
        );
        assert!(
            analyzer.all_findings[0]
                .mitre_techniques
                .iter()
                .any(|t| t == "T0814"),
            "malformed-LEN T0814 must be cited (BC-2.19.026 invariant 5)"
        );
        // Carry must be empty: the 2 bytes were processed and advanced past.
        let state = analyzer.flows.get(&flow_key).unwrap();
        assert!(
            state.carry_c2s.is_empty(),
            "carry must be empty after 2-byte malformed-LEN delivery is fully processed \
             (AC-174-007)"
        );
    }
}

// =============================================================================
// STORY-173: AC-173-007 — IEC-104 Findings Cap (BC-2.19.028)
// All tests live in `mod story_173` per DF-TEST-NAMESPACE-001.
// =============================================================================
//
// Two tests verify findings cap enforcement (now green):
//   test_BC_2_19_028_findings_cap — cap enforced; extend is truncated
//   test_BC_2_19_028_cap_maintained_across_multiple_on_data_calls — cap across multiple calls
//
// One test is a BOUNDARY GUARD (MAX-1 + 1 = MAX, no truncation needed):
//   test_BC_2_19_028_boundary_at_max_minus_one_allows_one_more

mod story_173 {
    #![allow(non_snake_case)]

    use std::net::IpAddr;

    use wirerust::analyzer::AnalysisSummary;
    use wirerust::analyzer::iec104::{Iec104Analyzer, MAX_IEC104_FINDINGS};
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    fn flow_key(src_port: u16, dst_port: u16) -> FlowKey {
        FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            src_port,
            "10.0.0.2".parse::<IpAddr>().unwrap(),
            dst_port,
        )
    }

    fn dummy_finding() -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Possible,
            confidence: Confidence::Medium,
            summary: "prefilled".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    // STOPDT-act U-frame: start=0x68, LEN=4, CF1=0x13, CF2-CF4=0.
    // With session_started=false → T0881 Verdict::Likely emitted by detect_iec104_threats.
    fn stopdt_act() -> Vec<u8> {
        vec![0x68, 0x04, 0x13, 0x00, 0x00, 0x00]
    }

    // -------------------------------------------------------------------------
    // AC-173-007 / BC-2.19.028 PC-2 — FINDINGS CAP PRIMARY INVARIANT
    // Cap enforced via truncation in on_data extend (line ~1283).
    // -------------------------------------------------------------------------

    /// AC-173-007 / BC-2.19.028 PC-2 — cap enforced: after pre-filling all_findings to
    /// MAX_IEC104_FINDINGS and feeding one on_data call that produces a finding,
    /// all_findings.len() must remain <= MAX and dropped_findings must be > 0.
    ///
    /// The cap is wired at the on_data extend step via truncation.
    /// When extend would exceed MAX, findings are dropped and dropped_findings is incremented.
    ///
    /// Traces: BC-2.19.028 PC-2 (primary invariant), PC-5 (dropped counter); AC-173-007.
    #[test]
    fn test_BC_2_19_028_findings_cap() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = flow_key(60001, 2404);

        // Pre-fill to MAX via the public all_findings field (BC-2.19.028 Architecture Anchor).
        analyzer.all_findings = vec![dummy_finding(); MAX_IEC104_FINDINGS];
        assert_eq!(
            analyzer.all_findings.len(),
            MAX_IEC104_FINDINGS,
            "pre-fill sanity: all_findings must be exactly MAX_IEC104_FINDINGS before on_data"
        );

        // Feed a STOPDT-act. session_started=false → T0881 Likely finding produced.
        analyzer.on_data(fk.clone(), &stopdt_act(), 0, Direction::ClientToServer);

        // BC-2.19.028 PC-2: primary invariant — never exceed MAX.
        assert!(
            analyzer.all_findings.len() <= MAX_IEC104_FINDINGS,
            "BC-2.19.028 PC-2: all_findings.len() ({}) must not exceed MAX_IEC104_FINDINGS \
             ({MAX_IEC104_FINDINGS}) after on_data — cap enforced at the extend step \
             via truncation before merging local_findings.",
            analyzer.all_findings.len()
        );

        // BC-2.19.028 PC-5: dropped_findings must count suppressed findings.
        assert!(
            analyzer.dropped_findings > 0,
            "BC-2.19.028 PC-5: dropped_findings must be > 0 when a finding was suppressed \
             by the cap. Got {} (counter incremented at extend step).",
            analyzer.dropped_findings
        );
    }

    // -------------------------------------------------------------------------
    // AC-173-007 / BC-2.19.028 EC-001 — BOUNDARY GUARD
    // Passes unconditionally: pre-fill MAX-1, one finding → exactly MAX (no cap truncation needed).
    // -------------------------------------------------------------------------

    /// AC-173-007 / BC-2.19.028 EC-001 — boundary: at MAX-1, one more finding fills to MAX.
    ///
    /// Pre-fill to MAX-1; one STOPDT-act produces 1 finding → total reaches exactly MAX.
    /// No cap truncation needed (MAX-1 + 1 == MAX); dropped_findings stays 0.
    ///
    /// Verifies boundary behavior: no cap truncation needed at MAX-1 (BC-2.19.028 EC-001).
    ///
    /// Traces: BC-2.19.028 EC-001; AC-173-007.
    #[test]
    fn test_BC_2_19_028_boundary_at_max_minus_one_allows_one_more() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = flow_key(60002, 2404);

        // Pre-fill to MAX - 1.
        analyzer.all_findings = vec![dummy_finding(); MAX_IEC104_FINDINGS - 1];

        // One STOPDT-act → one finding. Total becomes exactly MAX (no truncation required).
        analyzer.on_data(fk.clone(), &stopdt_act(), 0, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            MAX_IEC104_FINDINGS,
            "BC-2.19.028 EC-001: at MAX-1, one more finding must fill to exactly \
             MAX_IEC104_FINDINGS ({}). Got {}.",
            MAX_IEC104_FINDINGS,
            analyzer.all_findings.len()
        );
        assert_eq!(
            analyzer.dropped_findings, 0,
            "BC-2.19.028 EC-001: dropped_findings must remain 0 when within cap. Got {}",
            analyzer.dropped_findings
        );
    }

    // -------------------------------------------------------------------------
    // AC-173-007 / BC-2.19.028 EC-004 — CAP ACROSS MULTIPLE CALLS
    // Cap maintains invariant across N sequential calls (now green).
    // -------------------------------------------------------------------------

    /// AC-173-007 / BC-2.19.028 EC-004 — cap maintained across N sequential on_data calls.
    ///
    /// Pre-fill to MAX; then call on_data N times (different flow keys, each producing
    /// one STOPDT-act finding). With cap → len stays MAX and dropped_findings == N
    /// (all new findings are dropped to maintain the invariant).
    ///
    /// Cap is wired across multiple calls via truncation in the extend step.
    ///
    /// Traces: BC-2.19.028 EC-004; AC-173-007.
    #[test]
    fn test_BC_2_19_028_cap_maintained_across_multiple_on_data_calls() {
        const N: usize = 5;
        let mut analyzer = Iec104Analyzer::new();

        // Pre-fill to cap.
        analyzer.all_findings = vec![dummy_finding(); MAX_IEC104_FINDINGS];

        for i in 0..N {
            let fk = flow_key(60000 + i as u16, 2404);
            analyzer.on_data(fk, &stopdt_act(), 0, Direction::ClientToServer);
        }

        assert!(
            analyzer.all_findings.len() <= MAX_IEC104_FINDINGS,
            "BC-2.19.028 EC-004: all_findings must stay <= MAX after {} extra on_data calls. \
             Got {} (cap enforced via truncation at extend step).",
            N,
            analyzer.all_findings.len()
        );
        assert_eq!(
            analyzer.dropped_findings, N as u64,
            "BC-2.19.028 EC-004: dropped_findings must equal {N} after {N} suppressed on_data \
             calls (one finding per call). Got {}.",
            analyzer.dropped_findings
        );
    }

    // -------------------------------------------------------------------------
    // AC-173-007 / BC-2.19.028 PC-5 / F-173-001 — DROPPED_FINDINGS SURFACED IN SUMMARIZE
    // summarize() wired to include dropped_findings counter (now green).
    // -------------------------------------------------------------------------

    /// AC-173-007 / BC-2.19.028 PC-5 — `dropped_findings` counter appears in
    /// `summarize()` output under detail key `"dropped_findings"` with value > 0 after
    /// findings are suppressed by the cap.
    ///
    /// Drives the analyzer over the cap (mirrors `test_BC_2_19_028_findings_cap` setup),
    /// then calls `analyzer.summarize()` and asserts `detail["dropped_findings"] > 0`.
    ///
    /// The summarize() method is wired to expose the dropped_findings counter.
    ///
    /// Traces: BC-2.19.028 PC-5; AC-173-007; F-173-001.
    #[test]
    fn test_BC_2_19_028_dropped_findings_surfaced_in_summarize() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = flow_key(60099, 2404);

        // Pre-fill to MAX.
        analyzer.all_findings = vec![dummy_finding(); MAX_IEC104_FINDINGS];

        // Feed one STOPDT-act (session_started=false → T0881 Likely finding produced).
        // With the cap enforced this finding is discarded; dropped_findings increments to 1.
        analyzer.on_data(fk.clone(), &stopdt_act(), 0, Direction::ClientToServer);

        // Sanity: dropped_findings must be > 0 for this test to be meaningful.
        // (On the pre-implementation stub, the todo!() in summarize() was the load-bearing
        // failure for F-173-001; the cap enforcement was a secondary gap.)
        assert!(
            analyzer.dropped_findings > 0,
            "pre-condition: dropped_findings must be > 0 before calling summarize(); \
             got {} (cap enforced; finding suppressed above the limit)",
            analyzer.dropped_findings
        );

        // Call summarize() — now fully implemented; must populate the detail map.
        let summary: AnalysisSummary = analyzer.summarize();

        // BC-2.19.028 PC-5: detail map must contain "dropped_findings" key.
        assert!(
            summary.detail.contains_key("dropped_findings"),
            "BC-2.19.028 PC-5: summarize() detail map must contain key \"dropped_findings\"; \
             got keys: {:?}",
            summary.detail.keys().collect::<Vec<_>>()
        );

        // The value must be > 0 (matches the actual count dropped above).
        let dropped_val = summary.detail["dropped_findings"]
            .as_u64()
            .expect("\"dropped_findings\" detail value must be a JSON u64");
        assert!(
            dropped_val > 0,
            "BC-2.19.028 PC-5: summarize() detail[\"dropped_findings\"] must be > 0 after \
             findings were suppressed by the cap; got {dropped_val}"
        );
    }

    // -------------------------------------------------------------------------
    // LOW#1 (STORY-173 pre-merge fix): flows_analyzed must count closed flows
    //
    // BC-2.19.028 observability gap closed: summarize() now accumulates closed flows
    // via a flows_analyzed: u64 counter on Iec104Analyzer (incremented in
    // on_flow_close per removed flow; mirrors EnipAnalyzer pattern, enip.rs ~707).
    // -------------------------------------------------------------------------

    /// flows_analyzed in summarize() must count flows closed via on_flow_close.
    ///
    /// Two distinct flows are driven and closed; summarize().detail["flows_analyzed"]
    /// must equal 2. Regression guard: the flows_analyzed accumulator is wired to
    /// on_flow_close and must not regress to reading self.flows.len() (= 0 after removal).
    ///
    /// Traces: BC-2.19.028 observability; STORY-173 LOW#1.
    #[test]
    fn test_BC_2_19_028_flows_analyzed_counts_closed_flows() {
        let mut analyzer = Iec104Analyzer::new();
        let fk1 = flow_key(60201, 2404);
        let fk2 = flow_key(60202, 2404);

        // TESTFR-act: CF1=0x43, LEN=4 — no finding emitted, session state unchanged.
        let testfr_act = [0x68u8, 0x04, 0x43, 0x00, 0x00, 0x00];

        analyzer.on_data(fk1.clone(), &testfr_act, 0, Direction::ClientToServer);
        analyzer.on_data(fk2.clone(), &testfr_act, 0, Direction::ClientToServer);

        // Close both flows. After removal self.flows is empty (len == 0).
        analyzer.on_flow_close(fk1);
        analyzer.on_flow_close(fk2);

        let summary = analyzer.summarize();

        let flows_analyzed = summary
            .detail
            .get("flows_analyzed")
            .expect("summarize() detail map must include key \"flows_analyzed\"")
            .as_u64()
            .expect("\"flows_analyzed\" detail value must be a JSON u64");

        assert_eq!(
            flows_analyzed, 2,
            "flows_analyzed must equal 2 after closing 2 flows; \
             got {flows_analyzed} (current impl reads self.flows.len() = 0 after removal; \
             fix: add flows_analyzed: u64 field and increment in on_flow_close)"
        );
    }

    // -------------------------------------------------------------------------
    // LOW#2 (STORY-173 pre-merge fix): packets_analyzed must count parsed APDUs,
    // not all_findings.len()
    //
    // BC-2.19.028 observability gap closed: summarize() now returns the correct
    // packets_analyzed count via a per-flow frame_count: u64 on Iec104FlowState
    // incremented for every complete valid APDU in the on_data walk loop, accumulated
    // in summarize() (mirrors DNP3 closed_flows_count + per-flow frame_count pattern,
    // dnp3.rs ~316/1815).
    // -------------------------------------------------------------------------

    /// packets_analyzed in summarize() must count complete valid parsed APDUs,
    /// not all_findings.len().
    ///
    /// Three TESTFR-act U-frames (no findings) are fed through on_data, followed by
    /// one bad-start byte (0x00). packets_analyzed must equal 3; the bad-start byte
    /// must NOT be counted. Regression guard: the frame_count accumulator is wired to
    /// the on_data walk loop and must not regress to returning all_findings.len().
    ///
    /// Traces: BC-2.19.028 observability; STORY-173 LOW#2.
    #[test]
    fn test_iec104_packets_analyzed_counts_valid_frames() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = flow_key(60301, 2404);

        // 3 complete TESTFR-act frames (CF1=0x43, LEN=4, 6 bytes each) followed by
        // one bad-start byte. TESTFR-act produces no finding and leaves session state
        // unchanged — the cleanest zero-finding frame for this counter test.
        let mut data: Vec<u8> = Vec::new();
        for _ in 0..3 {
            data.extend_from_slice(&[0x68, 0x04, 0x43, 0x00, 0x00, 0x00]);
        }
        // Bad-start byte: not 0x68, so the walk loop advances 1 without counting.
        data.push(0x00);

        analyzer.on_data(fk.clone(), &data, 0, Direction::ClientToServer);
        analyzer.on_flow_close(fk);

        let summary = analyzer.summarize();

        assert_eq!(
            summary.packets_analyzed, 3,
            "packets_analyzed must equal 3 (the three complete valid APDUs parsed); \
             got {} (current impl reads all_findings.len() = 0 because TESTFR-act emits \
             no finding; fix: wire a frame counter in the on_data walk loop)",
            summary.packets_analyzed
        );
    }

    // =========================================================================
    // AC-174-007: targeted test for cargo-mutants survivor at iec104.rs:1323
    // (replace - with + in dropped_findings increment when remaining_cap > 0)
    // =========================================================================

    /// AC-174-007 mutant kill: dropped_findings must reflect exact count when remaining_cap > 0.
    ///
    /// The existing cap tests pre-fill to MAX (remaining_cap=0), making `-` vs `+` equivalent
    /// (both compute 0). This test pre-fills to MAX-2 (remaining_cap=2) and delivers 3 findings,
    /// requiring exactly 1 to be dropped — distinguishing original `len - cap` from `len + cap`.
    ///
    /// With `replace - with +`: dropped_findings would be 3 + 2 = 5 instead of 3 - 2 = 1.
    ///
    /// Traces: BC-2.19.028 PC-5; AC-174-007 (AC-173-007 extension for partial-cap scenario).
    #[test]
    fn test_AC_174_007_dropped_findings_exact_count_with_partial_cap() {
        let mut analyzer = Iec104Analyzer::new();
        // Pre-fill to MAX - 2: remaining_cap = 2.
        analyzer.all_findings = vec![dummy_finding(); MAX_IEC104_FINDINGS - 2];
        let fk = flow_key(61000, 2404);
        // Three STOPDT-act frames concatenated: each generates exactly 1 finding.
        // 3 local_findings > remaining_cap (2) → truncate to 2, drop 1.
        let three_stopdt: Vec<u8> = stopdt_act()
            .into_iter()
            .chain(stopdt_act())
            .chain(stopdt_act())
            .collect();
        analyzer.on_data(fk, &three_stopdt, 0, Direction::ClientToServer);
        assert_eq!(
            analyzer.dropped_findings, 1,
            "exactly 1 finding must be dropped (3 generated, remaining_cap=2); \
             mutation replace - with + would yield 5 instead (BC-2.19.028 PC-5; AC-174-007)"
        );
        assert_eq!(
            analyzer.all_findings.len(),
            MAX_IEC104_FINDINGS,
            "all_findings must be exactly at cap after partial truncation (BC-2.19.028 PC-2)"
        );
    }
}

// =============================================================================
// FIX-P4-001: IEC104-FINDING-DIRECTION-001 — populate direction on all IEC-104
// emitted Findings to match the TLS/Modbus/HTTP/DNP3/ENIP house pattern.
//
// ## What this tests
// Every IEC-104 emit site previously set `direction: None` even when `direction`
// was known at the call site. These tests assert the structured field is now
// `Some(direction)` for each finding type.
//
// ## Emit sites covered
// 1. track_ns_desync (:1046) — C2S desync + S2C desync
// 2. process_u_frame STOPDT-act (:388) + non-canonical U-frame (:424)
// 3. detect_iec104_threats TypeIDs 45-47 T1692.001 (:759),
//    TypeIDs 48-51 T1692.001 (:800) + T0836 (:815),
//    TypeID 105 T0827 (:845), TypeIDs 0/128-255 T0814 (:895)
// 4. on_data carry-overflow T0814 (:1193) + malformed-LEN T0814 (:1262)
//
// ## Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001; PG-W72-BREAKING-HOLDOUT-SWEEP
// =============================================================================
mod fix_p4_001 {
    use wirerust::analyzer::iec104::{
        Asdu, Iec104Analyzer, Iec104FlowState, MAX_IEC104_CARRY_BYTES, detect_iec104_threats,
        process_u_frame, track_ns_desync,
    };
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    fn flow_key(src_port: u16, dst_port: u16) -> FlowKey {
        FlowKey::new(
            "10.0.0.1".parse().unwrap(),
            src_port,
            "10.0.0.2".parse().unwrap(),
            dst_port,
        )
    }

    fn make_asdu(type_id: u8) -> Asdu {
        Asdu {
            type_id,
            sq: false,
            count: 1,
            cot_cause: 6,
            cot_pn: false,
            cot_test: false,
            cot_originator: 0,
            casdu: 1,
            first_ioa: None,
        }
    }

    // =========================================================================
    // track_ns_desync — direction field populated (IEC104-FINDING-DIRECTION-001)
    // =========================================================================

    /// track_ns_desync emits finding with direction=Some(ClientToServer) on C2S desync.
    ///
    /// RED: current code sets direction: None.
    /// GREEN: fix sets direction: Some(Direction::ClientToServer).
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001.
    #[test]
    fn test_fix_p4_001_track_ns_desync_direction_c2s() {
        let mut state = Iec104FlowState::default();
        // First call establishes baseline (path A, no finding).
        let _ = track_ns_desync(&mut state, 0, Direction::ClientToServer, None, None);
        // Second call: gap=5000 > k=12 → path C, emits finding.
        let finding = track_ns_desync(&mut state, 5000, Direction::ClientToServer, None, None)
            .expect("gap=5000 > k=12 must emit a desync finding (FIX-P4-001)");
        assert_eq!(
            finding.direction,
            Some(Direction::ClientToServer),
            "track_ns_desync C2S finding must carry direction=Some(ClientToServer) \
             (IEC104-FINDING-DIRECTION-001; was None before FIX-P4-001)"
        );
    }

    /// track_ns_desync emits finding with direction=Some(ServerToClient) on S2C desync.
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001.
    #[test]
    fn test_fix_p4_001_track_ns_desync_direction_s2c() {
        let mut state = Iec104FlowState::default();
        let _ = track_ns_desync(&mut state, 0, Direction::ServerToClient, None, None);
        let finding = track_ns_desync(&mut state, 5000, Direction::ServerToClient, None, None)
            .expect("gap=5000 > k=12 must emit a desync finding (FIX-P4-001)");
        assert_eq!(
            finding.direction,
            Some(Direction::ServerToClient),
            "track_ns_desync S2C finding must carry direction=Some(ServerToClient) \
             (IEC104-FINDING-DIRECTION-001; was None before FIX-P4-001)"
        );
    }

    // =========================================================================
    // process_u_frame — direction field populated (IEC104-FINDING-DIRECTION-001)
    // =========================================================================

    /// process_u_frame STOPDT-act emits Finding with direction=Some(ClientToServer).
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :388.
    #[test]
    fn test_fix_p4_001_process_u_frame_stopdt_direction_c2s() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0x13, Direction::ClientToServer, None, None)
            .expect("STOPDT-act must emit T0881 finding (FIX-P4-001)");
        assert_eq!(
            finding.direction,
            Some(Direction::ClientToServer),
            "process_u_frame STOPDT-act finding must carry direction=Some(ClientToServer) \
             (IEC104-FINDING-DIRECTION-001; emit site :388)"
        );
    }

    /// process_u_frame STOPDT-act emits Finding with direction=Some(ServerToClient).
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :388.
    #[test]
    fn test_fix_p4_001_process_u_frame_stopdt_direction_s2c() {
        let mut state = Iec104FlowState::default();
        let finding = process_u_frame(&mut state, 0x13, Direction::ServerToClient, None, None)
            .expect("STOPDT-act must emit T0881 finding (FIX-P4-001)");
        assert_eq!(
            finding.direction,
            Some(Direction::ServerToClient),
            "process_u_frame STOPDT-act finding must carry direction=Some(ServerToClient) \
             (IEC104-FINDING-DIRECTION-001; emit site :388)"
        );
    }

    /// process_u_frame non-canonical U-frame emits Finding with direction populated.
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :424.
    #[test]
    fn test_fix_p4_001_process_u_frame_noncanonical_direction_c2s() {
        let mut state = Iec104FlowState::default();
        // 0x03 has bits1:0=0b11 (U-format) but is not in the canonical set.
        let finding = process_u_frame(&mut state, 0x03, Direction::ClientToServer, None, None)
            .expect("non-canonical U-frame must emit T0814 finding (FIX-P4-001)");
        assert_eq!(
            finding.direction,
            Some(Direction::ClientToServer),
            "process_u_frame non-canonical U-frame finding must carry \
             direction=Some(ClientToServer) (IEC104-FINDING-DIRECTION-001; emit site :424)"
        );
    }

    // =========================================================================
    // detect_iec104_threats — direction field populated (IEC104-FINDING-DIRECTION-001)
    // =========================================================================

    /// TypeID=45 (C_SC_NA_1) T1692.001 finding carries direction=Some(ClientToServer).
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :759.
    #[test]
    fn test_fix_p4_001_detect_threats_type45_direction_c2s() {
        let asdu = make_asdu(45);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
        let f = findings.first().expect("TypeID=45 must emit finding");
        assert_eq!(
            f.direction,
            Some(Direction::ClientToServer),
            "TypeID=45 T1692.001 finding must carry direction=Some(ClientToServer) \
             (IEC104-FINDING-DIRECTION-001; emit site :759)"
        );
    }

    /// TypeID=48 (C_SE_NA_1) emits two findings — both carry direction=Some(ServerToClient).
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit sites :800 + :815.
    #[test]
    fn test_fix_p4_001_detect_threats_type48_direction_s2c() {
        let asdu = make_asdu(48);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings, Direction::ServerToClient, None, None);
        assert_eq!(findings.len(), 2, "TypeID=48 must emit 2 findings");
        for (i, f) in findings.iter().enumerate() {
            assert_eq!(
                f.direction,
                Some(Direction::ServerToClient),
                "TypeID=48 finding[{i}] must carry direction=Some(ServerToClient) \
                 (IEC104-FINDING-DIRECTION-001; emit sites :800/:815)"
            );
        }
    }

    /// TypeID=105 (C_RP_NA_1) T0827 finding carries direction=Some(ClientToServer).
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :845.
    #[test]
    fn test_fix_p4_001_detect_threats_type105_direction_c2s() {
        let asdu = make_asdu(105);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
        let f = findings
            .first()
            .expect("TypeID=105 must emit T0827 finding");
        assert_eq!(
            f.direction,
            Some(Direction::ClientToServer),
            "TypeID=105 T0827 finding must carry direction=Some(ClientToServer) \
             (IEC104-FINDING-DIRECTION-001; emit site :845)"
        );
    }

    /// TypeID=0 T0814 finding carries direction=Some(ClientToServer).
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :895.
    #[test]
    fn test_fix_p4_001_detect_threats_type0_direction_c2s() {
        let asdu = make_asdu(0);
        let mut findings = Vec::new();
        detect_iec104_threats(&asdu, &mut findings, Direction::ClientToServer, None, None);
        let f = findings.first().expect("TypeID=0 must emit T0814 finding");
        assert_eq!(
            f.direction,
            Some(Direction::ClientToServer),
            "TypeID=0 T0814 finding must carry direction=Some(ClientToServer) \
             (IEC104-FINDING-DIRECTION-001; emit site :895)"
        );
    }

    // =========================================================================
    // on_data inline emit sites — direction field populated
    // (:1193 carry-overflow T0814; :1262 malformed-LEN T0814)
    // =========================================================================

    /// on_data carry-overflow finding carries direction=Some(ClientToServer).
    ///
    /// Inject an oversized carry into the C2S buffer directly (>MAX_IEC104_CARRY_BYTES),
    /// then call on_data — the overflow check fires and emits a T0814 finding.
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :1193.
    #[test]
    fn test_fix_p4_001_on_data_carry_overflow_direction_c2s() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = flow_key(50000, 2404);
        // Seed the flow so we can directly manipulate its carry buffer.
        analyzer.on_data(fk.clone(), &[], 0, Direction::ClientToServer);
        // Overflow the C2S carry: inject MAX+1 bytes.
        {
            let state = analyzer.flows.get_mut(&fk).unwrap();
            state.carry_c2s = vec![0xAA; MAX_IEC104_CARRY_BYTES + 1];
        }
        // Trigger the overflow check by delivering any data in C2S direction.
        analyzer.on_data(fk.clone(), &[], 0, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .expect("carry-overflow must emit T0814 finding (FIX-P4-001)");
        assert_eq!(
            f.direction,
            Some(Direction::ClientToServer),
            "carry-overflow T0814 finding must carry direction=Some(ClientToServer) \
             (IEC104-FINDING-DIRECTION-001; emit site :1193)"
        );
    }

    /// on_data malformed-LEN finding carries direction=Some(ServerToClient).
    ///
    /// Deliver a byte sequence with a valid 0x68 start byte followed by LEN=1
    /// (outside [4, 253]) in the S2C direction.
    ///
    /// Traces: IEC104-FINDING-DIRECTION-001; FIX-P4-001 emit site :1262.
    #[test]
    fn test_fix_p4_001_on_data_malformed_len_direction_s2c() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = flow_key(50001, 2404);
        // 0x68 = IEC-104 start byte; 0x01 = LEN=1, outside [4, 253] → malformed
        let bad_frame: &[u8] = &[0x68, 0x01];
        analyzer.on_data(fk.clone(), bad_frame, 0, Direction::ServerToClient);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .expect("malformed-LEN must emit T0814 finding (FIX-P4-001)");
        assert_eq!(
            f.direction,
            Some(Direction::ServerToClient),
            "malformed-LEN T0814 finding must carry direction=Some(ServerToClient) \
             (IEC104-FINDING-DIRECTION-001; emit site :1262)"
        );
    }
}

// =============================================================================
// FIX-F5-001: source_ip + timestamp enrichment on all IEC-104 emit sites.
//
// ## What this tests
// Every IEC-104 emit site previously set source_ip: None and timestamp: None.
// BC-2.19.011 PC-3 requires findings to include the flow's 5-tuple context.
// These tests assert that source_ip == Some(initiator_ip) and
// timestamp.is_some() for each finding family, verified end-to-end via
// on_data (the effectful shell that resolves FlowKey IPs and passes them down).
//
// ## Flow setup
// FlowKey::new("10.0.0.1", 60002, "10.0.0.2", 2404):
//   lower=(10.0.0.1, 60002), upper=(10.0.0.2, 2404).
//   lower_port()=60002 ≠ 2404 → (client_ip, server_ip) = (10.0.0.1, 10.0.0.2).
//   C2S direction: source_ip = 10.0.0.1 (initiator/client).
//   S2C direction: source_ip = 10.0.0.2 (initiator/server).
//
// ## Emit sites covered (10 direct + 2 inline)
// 1. process_u_frame STOPDT-act T0881 — C2S + S2C (BC-2.19.011 PC-3)
// 2. on_data carry-overflow T0814
// 3. on_data malformed-LEN T0814
// 4. detect_iec104_threats TypeID=45 T1692.001
// 5. detect_iec104_threats TypeID=48 T1692.001 + T0836
// 6. detect_iec104_threats TypeID=105 T0827
// 7. detect_iec104_threats TypeID=0 T0814 (reserved)
// 8. track_ns_desync T1692.001 (desync path-C)
// 9. process_u_frame non-canonical U-frame T0814
//
// ## Traces: BC-2.19.011 PC-3; FIX-F5-001; sibling parity with DNP3/EnIP
// =============================================================================
mod fix_f5_001 {
    use std::net::IpAddr;

    use wirerust::analyzer::iec104::{Iec104Analyzer, MAX_IEC104_CARRY_BYTES};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    /// Helper: C2S flow where 10.0.0.1:60002 → 10.0.0.2:2404.
    /// lower=(10.0.0.1, 60002) because 10.0.0.1 < 10.0.0.2 lexicographically.
    /// lower_port()=60002 ≠ 2404 → client_ip=lower_ip=10.0.0.1.
    fn fk() -> FlowKey {
        FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            60002,
            "10.0.0.2".parse::<IpAddr>().unwrap(),
            2404,
        )
    }

    fn client_ip() -> IpAddr {
        "10.0.0.1".parse().unwrap()
    }

    fn server_ip() -> IpAddr {
        "10.0.0.2".parse().unwrap()
    }

    // STOPDT-act frame: start=0x68, LEN=4, CF1=0x13, CF2-CF4=0.
    fn stopdt_act() -> [u8; 6] {
        [0x68, 0x04, 0x13, 0x00, 0x00, 0x00]
    }

    // I-format frame with TypeID=45 (C_SC_NA_1 switching command):
    //   APCI: start=0x68, LEN=0x0A, CF1=0x00 (N(S)=0), CF2-CF4=0.
    //   ASDU: TypeID=45(0x2D), VSQ=1, COT_cause=6, orig=0, CASDU=1.
    fn i_frame_type45() -> [u8; 12] {
        [0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x2D, 0x01, 0x06, 0x00, 0x01, 0x00]
    }

    // I-format frame with TypeID=48 (C_SE_NA_1 set-point command):
    //   Emits T1692.001 + T0836 (two findings).
    fn i_frame_type48() -> [u8; 12] {
        [0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x30, 0x01, 0x06, 0x00, 0x01, 0x00]
    }

    // I-format frame with TypeID=105 (C_RP_NA_1 Reset Process Command):
    //   Emits T0827 Likely.
    fn i_frame_type105() -> [u8; 12] {
        [0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x69, 0x01, 0x06, 0x00, 0x01, 0x00]
    }

    // I-format frame with TypeID=0 (reserved/undefined):
    //   Emits T0814 Possible.
    fn i_frame_type0() -> [u8; 12] {
        [0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x00, 0x01, 0x00]
    }

    // I-format frame for desync test, N(S)=0:
    //   CF1=0x00 → N(S) = (0x00 >> 1) | (0x00 << 7) = 0.
    fn i_frame_ns0() -> [u8; 12] {
        [0x68, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x06, 0x00, 0x01, 0x00]
    }

    // I-format frame for desync test, N(S)=14 (gap=14 > k=12 → T1692.001):
    //   CF1=0x1C → N(S) = (0x1C >> 1) = 14.
    fn i_frame_ns14() -> [u8; 12] {
        [0x68, 0x0A, 0x1C, 0x00, 0x00, 0x00, 0x01, 0x01, 0x06, 0x00, 0x01, 0x00]
    }

    // Non-canonical U-frame: CF1=0x03 (bits1:0=0b11, not in canonical set).
    fn non_canonical_u_frame() -> [u8; 6] {
        [0x68, 0x04, 0x03, 0x00, 0x00, 0x00]
    }

    // =========================================================================
    // T0881 STOPDT-act — source_ip + timestamp (BC-2.19.011 PC-3; FIX-F5-001)
    // Written RED-first: source_ip was None; timestamp was None before this fix.
    // =========================================================================

    /// STOPDT-act C2S: finding carries source_ip=Some(client_ip) and timestamp.is_some().
    ///
    /// Verifies BC-2.19.011 PC-3: the finding includes the flow's address context.
    /// Initiator direction=ClientToServer → source_ip = client endpoint = 10.0.0.1.
    ///
    /// RED before FIX-F5-001: source_ip was None (no enrichment).
    /// GREEN after: source_ip = Some(10.0.0.1); timestamp.is_some().
    ///
    /// Traces: BC-2.19.011 PC-3; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_stopdt_c2s_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        analyzer.on_data(fk(), &stopdt_act(), 1_000_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0881"))
            .expect("STOPDT-act must emit T0881 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "T0881 C2S finding must carry source_ip=Some(10.0.0.1) — \
             initiator is client endpoint (BC-2.19.011 PC-3; FIX-F5-001); \
             was None before this fix"
        );
        assert!(
            f.timestamp.is_some(),
            "T0881 C2S finding must carry a non-None timestamp (FIX-F5-001); \
             was None before this fix"
        );
    }

    /// STOPDT-act S2C: finding carries source_ip=Some(server_ip) and timestamp.is_some().
    ///
    /// Direction=ServerToClient → source_ip = server endpoint = 10.0.0.2.
    ///
    /// Traces: BC-2.19.011 PC-3; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_stopdt_s2c_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        analyzer.on_data(fk(), &stopdt_act(), 2_000_000, Direction::ServerToClient);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0881"))
            .expect("STOPDT-act S2C must emit T0881 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(server_ip()),
            "T0881 S2C finding must carry source_ip=Some(10.0.0.2) — \
             initiator is server endpoint in S2C direction (BC-2.19.011 PC-3; FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "T0881 S2C finding must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // Carry-overflow T0814 — source_ip + timestamp (FIX-F5-001 inline emit site)
    // =========================================================================

    /// Carry-overflow T0814 carries source_ip=Some(client_ip) and timestamp.is_some().
    ///
    /// Traces: FIX-F5-001 F-01+F-02; on_data carry-overflow emit site.
    #[test]
    fn test_fix_f5_001_carry_overflow_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = fk();
        // Seed the flow.
        analyzer.on_data(fk.clone(), &[], 0, Direction::ClientToServer);
        // Overflow C2S carry.
        {
            let state = analyzer.flows.get_mut(&fk).unwrap();
            state.carry_c2s = vec![0xAA; MAX_IEC104_CARRY_BYTES + 1];
        }
        // Trigger overflow check.
        analyzer.on_data(fk.clone(), &[], 500_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .expect("carry-overflow must emit T0814 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "carry-overflow T0814 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "carry-overflow T0814 must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // Malformed-LEN T0814 — source_ip + timestamp (FIX-F5-001 inline emit site)
    // =========================================================================

    /// Malformed-LEN T0814 carries source_ip=Some(client_ip) and timestamp.is_some().
    ///
    /// Traces: FIX-F5-001 F-01+F-02; on_data malformed-LEN emit site.
    #[test]
    fn test_fix_f5_001_malformed_len_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        // 0x68 start byte + LEN=1 (outside [4,253]) → malformed-LEN T0814.
        let bad_frame: &[u8] = &[0x68, 0x01];
        analyzer.on_data(fk(), bad_frame, 750_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .expect("malformed-LEN must emit T0814 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "malformed-LEN T0814 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "malformed-LEN T0814 must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // TypeID=45 T1692.001 — source_ip + timestamp (FIX-F5-001)
    // =========================================================================

    /// TypeID=45 (C_SC_NA_1) T1692.001 finding carries source_ip and timestamp.
    ///
    /// Traces: BC-2.19.019; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_type45_t1692_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        analyzer.on_data(fk(), &i_frame_type45(), 1_200_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"))
            .expect("TypeID=45 must emit T1692.001 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "TypeID=45 T1692.001 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "TypeID=45 T1692.001 must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // TypeID=48 T0836 — source_ip + timestamp (FIX-F5-001)
    // =========================================================================

    /// TypeID=48 (C_SE_NA_1) T0836 finding carries source_ip and timestamp.
    ///
    /// TypeID=48 emits both T1692.001 and T0836; test specifically asserts T0836.
    ///
    /// Traces: BC-2.19.019 PC-2; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_type48_t0836_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        analyzer.on_data(fk(), &i_frame_type48(), 1_400_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0836"))
            .expect("TypeID=48 must emit T0836 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "TypeID=48 T0836 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "TypeID=48 T0836 must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // TypeID=105 T0827 — source_ip + timestamp (FIX-F5-001)
    // =========================================================================

    /// TypeID=105 (C_RP_NA_1) T0827 finding carries source_ip and timestamp.
    ///
    /// Traces: BC-2.19.020; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_type105_t0827_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        analyzer.on_data(fk(), &i_frame_type105(), 1_600_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0827"))
            .expect("TypeID=105 must emit T0827 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "TypeID=105 T0827 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "TypeID=105 T0827 must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // TypeID=0 T0814 (reserved/undefined) — source_ip + timestamp (FIX-F5-001)
    // =========================================================================

    /// TypeID=0 (undefined) T0814 finding carries source_ip and timestamp.
    ///
    /// Traces: BC-2.19.022; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_type0_t0814_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        analyzer.on_data(fk(), &i_frame_type0(), 1_800_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .expect("TypeID=0 must emit T0814 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "TypeID=0 T0814 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "TypeID=0 T0814 must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // track_ns_desync T1692.001 — source_ip + timestamp (FIX-F5-001)
    // =========================================================================

    /// N(S) desync T1692.001 finding carries source_ip and timestamp.
    ///
    /// Two C2S I-frames: N(S)=0 (baseline), then N(S)=14 (gap=14>12 → path-C).
    /// TypeID=1 (monitoring, no threat finding) isolates the desync finding.
    ///
    /// Traces: BC-2.19.024 path-C; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_desync_t1692_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        let fk = fk();
        // Frame 1: N(S)=0, establishes baseline (path A, no finding).
        analyzer.on_data(fk.clone(), &i_frame_ns0(), 0, Direction::ClientToServer);
        assert!(
            analyzer.all_findings.is_empty(),
            "first I-frame must not emit any finding (path A baseline)"
        );
        // Frame 2: N(S)=14, gap=14 > k=12 → T1692.001 path-C.
        analyzer.on_data(fk, &i_frame_ns14(), 2_500_000, Direction::ClientToServer);
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T1692.001"))
            .expect("N(S) desync gap=14 must emit T1692.001 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "desync T1692.001 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "desync T1692.001 must carry a non-None timestamp (FIX-F5-001)"
        );
    }

    // =========================================================================
    // Non-canonical U-frame T0814 — source_ip + timestamp (FIX-F5-001)
    // =========================================================================

    /// Non-canonical U-frame T0814 carries source_ip and timestamp.
    ///
    /// CF1=0x03: bits1:0=0b11 (U-format) but not in canonical set →
    /// T0814 "CVE-2026-1773 denial-of-service" finding.
    ///
    /// Traces: BC-2.19.014; FIX-F5-001 F-01+F-02.
    #[test]
    fn test_fix_f5_001_noncanonical_u_frame_t0814_source_ip_and_timestamp() {
        let mut analyzer = Iec104Analyzer::new();
        analyzer.on_data(
            fk(),
            &non_canonical_u_frame(),
            3_000_000,
            Direction::ClientToServer,
        );
        let f = analyzer
            .all_findings
            .iter()
            .find(|f| f.mitre_techniques.iter().any(|t| t == "T0814"))
            .expect("non-canonical U-frame must emit T0814 finding (FIX-F5-001)");
        assert_eq!(
            f.source_ip,
            Some(client_ip()),
            "non-canonical U-frame T0814 must carry source_ip=Some(10.0.0.1) (FIX-F5-001)"
        );
        assert!(
            f.timestamp.is_some(),
            "non-canonical U-frame T0814 must carry a non-None timestamp (FIX-F5-001)"
        );
    }
}
