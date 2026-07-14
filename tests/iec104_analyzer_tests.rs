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
    use wirerust::findings::Verdict;

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
        assert!(
            f.mitre_techniques.iter().any(|t| t == "T0881"),
            "T0881 finding must have mitre_techniques containing \"T0881\" \
             (BC-2.19.012 MITRE Techniques field)"
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
