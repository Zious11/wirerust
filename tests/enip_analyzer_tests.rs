//! Integration tests for the EtherNet/IP pure-core parse module (SS-17, STORY-130).
//!
//! Traces to: BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004.
//!
//! Test namespace: `mod parse_header` (DF-TEST-NAMESPACE-001).
//! Test naming: `test_BC_S_SS_NNN_xxx` or AC-cited name from STORY-130 **Test:** fields
//! (DF-AC-TEST-NAME-SYNC-001).
//!
//! All functions under test use `todo!()` bodies — every test MUST fail (Red Gate).
//! Tests MUST compile. Tests MUST NOT pass before implementation.

use wirerust::analyzer::enip::{
    EnipCommandClass, EnipHeader, classify_enip_command, is_valid_enip_frame, parse_enip_header,
};

mod parse_header {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Construct a minimal all-zero EnipHeader for use in validity-gate tests.
    /// Avoids repetition; fields are overridden per-test as needed.
    fn zero_header() -> EnipHeader {
        EnipHeader {
            command: 0x0000,
            length: 0,
            session_handle: 0,
            status: 0,
            sender_context: [0u8; 8],
            options: 0,
        }
    }

    /// Canonical BC-2.17.002 SendRRData test vector (24 bytes, little-endian).
    ///
    /// Byte breakdown:
    ///   [0..2]   6F 00   command = 0x006F (SendRRData, LE)
    ///   [2..4]   20 00   length  = 32 (LE)
    ///   [4..8]   04 03 02 01  session_handle = 0x01020304 (LE)
    ///   [8..12]  00 00 00 00  status = 0
    ///   [12..20] AA BB CC DD EE FF 00 11  sender_context (verbatim [u8;8])
    ///   [20..24] 00 00 00 00  options = 0
    ///
    /// Source: BC-2.17.002 §Canonical Test Vectors, "SendRRData with session" row.
    /// Aligns with holdout HS-110.
    fn canonical_sendrr_vector() -> [u8; 24] {
        [
            0x6F, 0x00, // command = 0x006F
            0x20, 0x00, // length  = 32
            0x04, 0x03, 0x02, 0x01, // session_handle = 0x01020304 (LE)
            0x00, 0x00, 0x00, 0x00, // status = 0
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, // sender_context
            0x00, 0x00, 0x00, 0x00, // options = 0
        ]
    }

    // -----------------------------------------------------------------------
    // AC-130-001 — parse_enip_header accept path
    // Cited tests: test_parse_enip_header_valid, test_parse_enip_header_too_short,
    //              test_parse_enip_header_exactly_24
    // Traces: BC-2.17.002 postconditions 1–8
    // -----------------------------------------------------------------------

    /// AC-130-001 — canonical SendRRData 24-byte vector: all fields decoded correctly (LE).
    ///
    /// Uses the exact BC-2.17.002 canonical vector. Asserts all six fields individually:
    /// command, length, session_handle, status, sender_context, options.
    /// Non-vacuous: every assertion binds to the actual vector byte values.
    ///
    /// Traces: BC-2.17.002 postconditions 2–7. VP-032 Sub-A field assertion target.
    #[test]
    fn test_parse_enip_header_valid() {
        let bytes = canonical_sendrr_vector();
        let result = parse_enip_header(&bytes);
        let h = result.expect("must return Some for 24-byte canonical vector");
        // BC-2.17.002 postcondition 2: command LE bytes [0,1]
        assert_eq!(h.command, 0x006F, "command: expected 0x006F (SendRRData)");
        // BC-2.17.002 postcondition 3: length LE bytes [2,3]
        assert_eq!(h.length, 32, "length: expected 32");
        // BC-2.17.002 postcondition 4: session_handle LE bytes [4..8] = 04 03 02 01 → 0x01020304
        assert_eq!(
            h.session_handle, 0x01020304,
            "session_handle: LE bytes 04 03 02 01 must decode to 0x01020304"
        );
        // BC-2.17.002 postcondition 5: status LE bytes [8..12]
        assert_eq!(h.status, 0, "status: expected 0");
        // BC-2.17.002 postcondition 6: sender_context verbatim [u8;8]
        assert_eq!(
            h.sender_context,
            [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11],
            "sender_context: expected verbatim copy of bytes [12..20]"
        );
        // BC-2.17.002 postcondition 7: options LE bytes [20..24]
        assert_eq!(h.options, 0, "options: expected 0");
    }

    /// AC-130-001 — 23-byte input returns None (one byte short of minimum).
    ///
    /// Traces: BC-2.17.001 postcondition 1; EC-002 (one byte short).
    #[test]
    fn test_parse_enip_header_too_short() {
        let bytes = [0u8; 23];
        assert!(
            parse_enip_header(&bytes).is_none(),
            "23-byte slice must return None"
        );
    }

    /// AC-130-001 — exactly 24 bytes: returns Some with correct all-zero fields.
    ///
    /// Non-vacuous: verifies all fields are zero for all-zero input.
    /// Traces: BC-2.17.002 EC-001 (exact minimum); BC-2.17.001 EC-004.
    #[test]
    fn test_parse_enip_header_exactly_24() {
        let bytes = [0u8; 24];
        let h = parse_enip_header(&bytes).expect("exactly 24 zero bytes must return Some");
        assert_eq!(h.command, 0x0000, "command must be 0 for all-zero input");
        assert_eq!(h.length, 0, "length must be 0 for all-zero input");
        assert_eq!(h.session_handle, 0, "session_handle must be 0");
        assert_eq!(h.status, 0, "status must be 0");
        assert_eq!(h.sender_context, [0u8; 8], "sender_context must be [0;8]");
        assert_eq!(h.options, 0, "options must be 0");
    }

    // -----------------------------------------------------------------------
    // AC-130-002 — parse_enip_header reject path
    // Cited tests: test_parse_enip_header_no_panic_empty,
    //              test_parse_enip_header_no_panic_23_bytes
    // Traces: BC-2.17.001 postconditions 1–3; EC-001, EC-003
    // -----------------------------------------------------------------------

    /// AC-130-002 — empty slice: returns None without panic.
    ///
    /// Traces: BC-2.17.001 EC-001 (0 bytes, no access, no panic).
    #[test]
    fn test_parse_enip_header_no_panic_empty() {
        assert!(
            parse_enip_header(&[]).is_none(),
            "empty slice must return None"
        );
    }

    /// AC-130-002 — 23-byte slice (all 0xFF): returns None without panic.
    ///
    /// Using 0xFF pattern ensures no bytes are accidentally read: if the function
    /// incorrectly reads bytes[0..1] = [0xFF, 0xFF], the decoded command would be 0xFFFF.
    /// The test exclusively asserts None — command must not be decoded from short input
    /// (BC-2.17.001 postcondition 2, invariant 3).
    ///
    /// Traces: BC-2.17.001 EC-003 (one byte short, none accessed).
    #[test]
    fn test_parse_enip_header_no_panic_23_bytes() {
        let bytes = [0xFFu8; 23];
        assert!(
            parse_enip_header(&bytes).is_none(),
            "23-byte 0xFF slice must return None without reading any bytes"
        );
    }

    // -----------------------------------------------------------------------
    // Additional field-offset correctness (BC-2.17.002 postconditions 4–7)
    // These tests verify each field independently at its canonical LE offset.
    // Not AC-cited by name but required by BC-2.17.002 and STORY-130 coverage spec.
    // -----------------------------------------------------------------------

    /// BC-2.17.002 postcondition 4 — session_handle at bytes [4..8], LE.
    ///
    /// Uses distinct byte values to prove LE decode: [01 02 03 04] → 0x04030201.
    /// Verifies bytes-beyond-23 are ignored (slice is 30 bytes).
    ///
    /// Traces: BC-2.17.002 postcondition 4, postcondition 8.
    #[test]
    fn test_enip_header_fields_session_handle_le() {
        let mut buf = [0u8; 30];
        buf[4] = 0x01;
        buf[5] = 0x02;
        buf[6] = 0x03;
        buf[7] = 0x04; // LE: 0x04030201
        // Poison bytes beyond 23 to verify they are not read
        buf[24] = 0xFF;
        buf[25] = 0xFF;
        let h = parse_enip_header(&buf).expect("30-byte slice must return Some");
        assert_eq!(
            h.session_handle, 0x04030201,
            "session_handle: bytes [01 02 03 04] LE must decode to 0x04030201"
        );
    }

    /// BC-2.17.002 postcondition 6 — sender_context at bytes [12..20], verbatim [u8;8].
    ///
    /// Non-trivial pattern verifies offset correctness: [12..20] = [A1..A8].
    /// Traces: BC-2.17.002 postcondition 6 (opaque 8-byte verbatim copy, invariant 3).
    #[test]
    fn test_enip_header_fields_sender_context_verbatim() {
        let mut buf = [0u8; 24];
        let ctx: [u8; 8] = [0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8];
        buf[12..20].copy_from_slice(&ctx);
        let h = parse_enip_header(&buf).expect("24-byte slice must return Some");
        assert_eq!(
            h.sender_context, ctx,
            "sender_context must be verbatim copy of bytes [12..20]"
        );
    }

    /// BC-2.17.002 postcondition 7 — options at bytes [20..24], LE.
    ///
    /// Traces: BC-2.17.002 postcondition 7.
    #[test]
    fn test_enip_header_fields_options_le() {
        let mut buf = [0u8; 24];
        buf[20] = 0x78;
        buf[21] = 0x56;
        buf[22] = 0x34;
        buf[23] = 0x12; // LE: 0x12345678
        let h = parse_enip_header(&buf).expect("24-byte slice must return Some");
        assert_eq!(
            h.options, 0x1234_5678,
            "options: bytes [78 56 34 12] LE must decode to 0x12345678"
        );
    }

    /// BC-2.17.002 postcondition 8 — bytes beyond index 23 are NOT read.
    ///
    /// A 48-byte slice is provided. Bytes [24..48] are set to 0xFF. The header
    /// fields must still decode from [0..24] only. Checks command specifically.
    ///
    /// Traces: BC-2.17.002 postcondition 8 (trailing bytes ignored).
    #[test]
    fn test_enip_header_fields_trailing_bytes_ignored() {
        let mut buf = [0u8; 48];
        buf[0] = 0x6F;
        buf[1] = 0x00; // command = 0x006F
        // Poison trailing bytes
        for b in &mut buf[24..] {
            *b = 0xFF;
        }
        let h = parse_enip_header(&buf).expect("48-byte slice must return Some");
        assert_eq!(
            h.command, 0x006F,
            "command must be decoded from bytes [0..2] only; trailing 0xFF bytes must not affect result"
        );
    }

    // -----------------------------------------------------------------------
    // AC-130-003 — classify_enip_command named variants and Unknown arm
    // Cited tests: test_classify_enip_command_known, test_classify_enip_command_unknown
    // Traces: BC-2.17.004 postcondition 2 (all 9 named mappings); postcondition 4 (Unknown)
    // -----------------------------------------------------------------------

    /// AC-130-003 — all 9 named ODVA command codes map to their named variants.
    ///
    /// Non-vacuous: each of the 9 named commands is asserted individually.
    /// Order follows BC-2.17.004 postcondition 2 table.
    ///
    /// Traces: BC-2.17.004 postcondition 2; VP-032 Sub-B named-variant reachability.
    #[test]
    fn test_classify_enip_command_known() {
        assert_eq!(
            classify_enip_command(0x0004),
            EnipCommandClass::ListServices,
            "0x0004 must map to ListServices"
        );
        assert_eq!(
            classify_enip_command(0x0063),
            EnipCommandClass::ListIdentity,
            "0x0063 must map to ListIdentity"
        );
        assert_eq!(
            classify_enip_command(0x0064),
            EnipCommandClass::ListInterfaces,
            "0x0064 must map to ListInterfaces"
        );
        assert_eq!(
            classify_enip_command(0x0065),
            EnipCommandClass::RegisterSession,
            "0x0065 must map to RegisterSession"
        );
        assert_eq!(
            classify_enip_command(0x0066),
            EnipCommandClass::UnRegisterSession,
            "0x0066 must map to UnRegisterSession"
        );
        assert_eq!(
            classify_enip_command(0x006F),
            EnipCommandClass::SendRRData,
            "0x006F must map to SendRRData (canonical LE command byte: 6F 00)"
        );
        assert_eq!(
            classify_enip_command(0x0070),
            EnipCommandClass::SendUnitData,
            "0x0070 must map to SendUnitData"
        );
        assert_eq!(
            classify_enip_command(0x0072),
            EnipCommandClass::IndicateStatus,
            "0x0072 must map to IndicateStatus"
        );
        assert_eq!(
            classify_enip_command(0x0075),
            EnipCommandClass::Cancel,
            "0x0075 must map to Cancel (highest named value)"
        );
    }

    /// AC-130-003 — an unassigned value (0x0001) maps to Unknown.
    ///
    /// Traces: BC-2.17.004 postcondition 4 (Unknown arm reachable).
    #[test]
    fn test_classify_enip_command_unknown() {
        assert_eq!(
            classify_enip_command(0x0001),
            EnipCommandClass::Unknown,
            "0x0001 is not an ODVA-assigned command; must map to Unknown"
        );
    }

    // -----------------------------------------------------------------------
    // AC-130-005 — Unknown arm reachability at 0x0000, 0xFFFF, gap values
    // Cited tests: test_classify_enip_command_unknown_zero,
    //              test_classify_enip_command_unknown_ffff,
    //              test_classify_enip_command_unknown_gap
    // Traces: BC-2.17.004 postcondition 4; EC-003, EC-004, EC-005
    // -----------------------------------------------------------------------

    /// AC-130-005 — 0x0000 maps to Unknown.
    ///
    /// Traces: BC-2.17.004 EC-003; VP-032 Sub-B non-vacuity (Unknown arm proven reachable).
    #[test]
    fn test_classify_enip_command_unknown_zero() {
        assert_eq!(
            classify_enip_command(0x0000),
            EnipCommandClass::Unknown,
            "0x0000 is not ODVA-assigned; must map to Unknown"
        );
    }

    /// AC-130-005 — 0xFFFF maps to Unknown.
    ///
    /// Traces: BC-2.17.004 EC-004 (max u16, not ODVA-assigned).
    #[test]
    fn test_classify_enip_command_unknown_ffff() {
        assert_eq!(
            classify_enip_command(0xFFFF),
            EnipCommandClass::Unknown,
            "0xFFFF is not ODVA-assigned; must map to Unknown"
        );
    }

    /// AC-130-005 — 0x0067 (gap between UnRegisterSession 0x0066 and SendRRData 0x006F)
    /// maps to Unknown.
    ///
    /// Traces: BC-2.17.004 EC-005; BC-2.17.003 EC-005 (gap in ODVA command table).
    #[test]
    fn test_classify_enip_command_unknown_gap() {
        assert_eq!(
            classify_enip_command(0x0067),
            EnipCommandClass::Unknown,
            "0x0067 is a gap in the ODVA command table; must map to Unknown"
        );
    }

    // -----------------------------------------------------------------------
    // AC-130-004 — is_valid_enip_frame biconditional gate
    // Cited tests: test_is_valid_enip_frame_known_commands_true,
    //              test_is_valid_enip_frame_unknown_command_false,
    //              test_is_valid_enip_frame_boundary_commands,
    //              test_is_valid_enip_frame_all_fields_zeroed
    // Traces: BC-2.17.003 postconditions 1–4; invariant 1
    // -----------------------------------------------------------------------

    /// AC-130-004 — all 9 known ODVA commands return true.
    ///
    /// Each header is constructed with only `command` set; other fields zeroed.
    /// Verifies the biconditional holds for the full known-command set.
    ///
    /// Traces: BC-2.17.003 postcondition 1; VP-032 Sub-C biconditional.
    #[test]
    fn test_is_valid_enip_frame_known_commands_true() {
        let known: &[u16] = &[
            0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075,
        ];
        for &cmd in known {
            let mut h = zero_header();
            h.command = cmd;
            assert!(
                is_valid_enip_frame(&h),
                "known command 0x{cmd:04X} must return true from is_valid_enip_frame"
            );
        }
    }

    /// AC-130-004 — unknown command 0x0000 returns false.
    ///
    /// Traces: BC-2.17.003 postcondition 2; EC-003.
    #[test]
    fn test_is_valid_enip_frame_unknown_command_false() {
        let mut h = zero_header();
        h.command = 0x0000;
        assert!(
            !is_valid_enip_frame(&h),
            "command 0x0000 is not in the known-command set; must return false"
        );
    }

    /// AC-130-004 — boundary commands: Cancel (0x0075, highest known) = true;
    /// 0x0076 (one above Cancel) = false; 0xFFFF = false.
    ///
    /// Covers the upper boundary and two representative unknowns above it.
    /// Traces: BC-2.17.003 EC-006 (Cancel included), EC-007 (0x0076 excluded), EC-004 (0xFFFF).
    #[test]
    fn test_is_valid_enip_frame_boundary_commands() {
        let mut h = zero_header();

        // 0x0075 — Cancel, highest member of known-command set
        h.command = 0x0075;
        assert!(
            is_valid_enip_frame(&h),
            "0x0075 (Cancel) is in the known-command set; must return true"
        );

        // 0x0076 — one above Cancel, not in ODVA table
        h.command = 0x0076;
        assert!(
            !is_valid_enip_frame(&h),
            "0x0076 is above Cancel; must return false"
        );

        // 0xFFFF — max u16, not in ODVA table
        h.command = 0xFFFF;
        assert!(
            !is_valid_enip_frame(&h),
            "0xFFFF is not in the known-command set; must return false"
        );
    }

    /// AC-130-004 — all-zeroed header: command=0x0000 returns false; demonstrates
    /// command-only gate (length, status, options do NOT affect the result).
    ///
    /// Non-vacuous: if the gate checked other fields, changing length/status/options
    /// to 0 might make it pass. This test confirms only command matters.
    ///
    /// Traces: BC-2.17.003 postcondition 3 (command-only gate); EC-003; invariant 3.
    #[test]
    fn test_is_valid_enip_frame_all_fields_zeroed() {
        let h = zero_header(); // command = 0x0000, all other fields zero
        assert!(
            !is_valid_enip_frame(&h),
            "all-zero header: command 0x0000 is not in the known-command set; must return false"
        );
    }

    // -----------------------------------------------------------------------
    // Additional is_valid_enip_frame tests (BC-2.17.003 coverage)
    // Covers EC-006 (command-only gate with valid command + zeroed other fields),
    // EC-005 (gap in ODVA table), and 0x0062 (below ListIdentity).
    // -----------------------------------------------------------------------

    /// BC-2.17.003 EC-008 — valid command with all other fields zeroed returns true.
    ///
    /// Specifically tests ListIdentity (0x0063) with status=0, options=0.
    /// Proves the gate is command-only: other field values do not affect the result.
    ///
    /// Traces: BC-2.17.003 postcondition 3; EC-008.
    #[test]
    fn test_is_valid_enip_frame_command_only_gate() {
        let h = EnipHeader {
            command: 0x0063, // ListIdentity — known command
            length: 0,
            session_handle: 0,
            status: 0,
            sender_context: [0u8; 8],
            options: 0,
        };
        assert!(
            is_valid_enip_frame(&h),
            "0x0063 (ListIdentity) with all other fields zeroed must return true — command-only gate"
        );
    }

    /// BC-2.17.003 EC-005 — gap value 0x0071 (between SendUnitData 0x0070 and
    /// IndicateStatus 0x0072) returns false.
    ///
    /// Traces: BC-2.17.003 EC-005 (gap in ODVA command table).
    #[test]
    fn test_is_valid_enip_frame_gap_value() {
        let mut h = zero_header();
        h.command = 0x0071; // gap: not in ODVA table
        assert!(
            !is_valid_enip_frame(&h),
            "0x0071 is a gap in the ODVA command table; must return false"
        );
    }

    /// BC-2.17.003 EC near-boundary — 0x0062 (below ListIdentity 0x0063) returns false.
    ///
    /// Verifies the lower boundary of the ListIdentity cluster.
    /// Traces: BC-2.17.003 §Canonical Test Vectors row "0x0062: false".
    #[test]
    fn test_is_valid_enip_frame_below_list_identity() {
        let mut h = zero_header();
        h.command = 0x0062; // one below ListIdentity
        assert!(
            !is_valid_enip_frame(&h),
            "0x0062 is below ListIdentity; must return false"
        );
    }
}
