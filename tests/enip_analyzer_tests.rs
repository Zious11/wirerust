//! Integration tests for the EtherNet/IP pure-core parse module (SS-17, STORY-130).
//!
//! Traces to: BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004.
//! All tests reference `wirerust::analyzer::enip::*` public surface.

use wirerust::analyzer::enip::{
    classify_enip_command, is_valid_enip_frame, EnipCommandClass, EnipHeader,
    parse_enip_header,
};

mod parse_header {
    use super::*;

    // -----------------------------------------------------------------------
    // AC-130-001 / AC-130-002 — parse_enip_header accept and reject paths
    // -----------------------------------------------------------------------

    /// AC-130-001 — valid 25-byte input: parse returns Some with correct LE fields.
    /// Traces: BC-2.17.002 postconditions 2–7.
    #[test]
    fn test_parse_enip_header_valid() {
        // SendRRData header with session handle = 0x01020304 (LE bytes: 04 03 02 01)
        let mut bytes = [0u8; 25];
        bytes[0] = 0x6F;
        bytes[1] = 0x00; // command = 0x006F (SendRRData, LE)
        bytes[2] = 0x20;
        bytes[3] = 0x00; // length = 32
        bytes[4] = 0x04;
        bytes[5] = 0x03;
        bytes[6] = 0x02;
        bytes[7] = 0x01; // session_handle = 0x01020304 (LE)
        // status = 0, sender_context = 0, options = 0 (remaining zero)
        let result = parse_enip_header(&bytes);
        let h = result.expect("must return Some for len=25");
        assert_eq!(h.command, 0x006F);
        assert_eq!(h.length, 32);
        assert_eq!(h.session_handle, 0x01020304);
        assert_eq!(h.status, 0);
        assert_eq!(h.sender_context, [0u8; 8]);
        assert_eq!(h.options, 0);
    }

    /// AC-130-001 — 23-byte input: returns None (one byte short of minimum).
    /// Traces: BC-2.17.001 postcondition 1; EC-002.
    #[test]
    fn test_parse_enip_header_too_short() {
        let bytes = [0u8; 23];
        assert!(parse_enip_header(&bytes).is_none());
    }

    /// AC-130-001 — exactly 24 bytes: returns Some.
    /// Traces: BC-2.17.002 EC-001 (exact minimum).
    #[test]
    fn test_parse_enip_header_exactly_24() {
        let bytes = [0u8; 24];
        assert!(parse_enip_header(&bytes).is_some());
    }

    /// AC-130-002 — empty slice: returns None, no panic.
    /// Traces: BC-2.17.001 EC-001.
    #[test]
    fn test_parse_enip_header_no_panic_empty() {
        assert!(parse_enip_header(&[]).is_none());
    }

    /// AC-130-002 — 23-byte slice: returns None, no panic.
    /// Traces: BC-2.17.001 EC-003.
    #[test]
    fn test_parse_enip_header_no_panic_23_bytes() {
        let bytes = [0xFFu8; 23];
        assert!(parse_enip_header(&bytes).is_none());
    }

    // -----------------------------------------------------------------------
    // AC-130-003 — classify_enip_command named variants and Unknown arm
    // -----------------------------------------------------------------------

    /// AC-130-003 — all 9 named ODVA command codes map to their named variants.
    /// Traces: BC-2.17.004 postcondition 2.
    #[test]
    fn test_classify_enip_command_known() {
        assert_eq!(classify_enip_command(0x0004), EnipCommandClass::ListServices);
        assert_eq!(classify_enip_command(0x0063), EnipCommandClass::ListIdentity);
        assert_eq!(classify_enip_command(0x0064), EnipCommandClass::ListInterfaces);
        assert_eq!(classify_enip_command(0x0065), EnipCommandClass::RegisterSession);
        assert_eq!(classify_enip_command(0x0066), EnipCommandClass::UnRegisterSession);
        assert_eq!(classify_enip_command(0x006F), EnipCommandClass::SendRRData);
        assert_eq!(classify_enip_command(0x0070), EnipCommandClass::SendUnitData);
        assert_eq!(classify_enip_command(0x0072), EnipCommandClass::IndicateStatus);
        assert_eq!(classify_enip_command(0x0075), EnipCommandClass::Cancel);
    }

    /// AC-130-003 — an unassigned value maps to Unknown.
    /// Traces: BC-2.17.004 postcondition 4; EC-003.
    #[test]
    fn test_classify_enip_command_unknown() {
        assert_eq!(classify_enip_command(0x0001), EnipCommandClass::Unknown);
    }

    // -----------------------------------------------------------------------
    // AC-130-005 — Unknown arm reachability at 0x0000, 0xFFFF, gap values
    // -----------------------------------------------------------------------

    /// AC-130-005 — 0x0000 maps to Unknown (BC-2.17.004 EC-003).
    #[test]
    fn test_classify_enip_command_unknown_zero() {
        assert_eq!(classify_enip_command(0x0000), EnipCommandClass::Unknown);
    }

    /// AC-130-005 — 0xFFFF maps to Unknown (BC-2.17.004 EC-004).
    #[test]
    fn test_classify_enip_command_unknown_ffff() {
        assert_eq!(classify_enip_command(0xFFFF), EnipCommandClass::Unknown);
    }

    /// AC-130-005 — 0x0067 (gap between UnRegisterSession and SendRRData) maps to Unknown.
    /// Traces: BC-2.17.004 EC-005.
    #[test]
    fn test_classify_enip_command_unknown_gap() {
        assert_eq!(classify_enip_command(0x0067), EnipCommandClass::Unknown);
    }

    // -----------------------------------------------------------------------
    // AC-130-004 — is_valid_enip_frame biconditional gate
    // -----------------------------------------------------------------------

    /// AC-130-004 — all 9 known commands return true.
    /// Traces: BC-2.17.003 postcondition 1.
    #[test]
    fn test_is_valid_enip_frame_known_commands_true() {
        for cmd in [
            0x0004u16, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075,
        ] {
            let h = EnipHeader {
                command: cmd,
                length: 0,
                session_handle: 0,
                status: 0,
                sender_context: [0u8; 8],
                options: 0,
            };
            assert!(
                is_valid_enip_frame(&h),
                "expected true for known command 0x{cmd:04X}"
            );
        }
    }

    /// AC-130-004 — unknown command 0x0000 returns false.
    /// Traces: BC-2.17.003 postcondition 2; EC-003.
    #[test]
    fn test_is_valid_enip_frame_unknown_command_false() {
        let h = EnipHeader {
            command: 0x0000,
            length: 0,
            session_handle: 0,
            status: 0,
            sender_context: [0u8; 8],
            options: 0,
        };
        assert!(!is_valid_enip_frame(&h));
    }

    /// AC-130-004 — boundary commands: 0x0075 (Cancel, highest known) = true;
    /// 0x0076 (one above Cancel) = false.
    /// Traces: BC-2.17.003 EC-006 and EC-007.
    #[test]
    fn test_is_valid_enip_frame_boundary_commands() {
        let make_header = |cmd: u16| EnipHeader {
            command: cmd,
            length: 0,
            session_handle: 0,
            status: 0,
            sender_context: [0u8; 8],
            options: 0,
        };
        assert!(is_valid_enip_frame(&make_header(0x0075))); // Cancel — included
        assert!(!is_valid_enip_frame(&make_header(0x0076))); // above Cancel — excluded
    }

    /// AC-130-004 — all-zeroed header: command=0x0000 → false; gate is command-only.
    /// Traces: BC-2.17.003 postcondition 3; EC-003.
    #[test]
    fn test_is_valid_enip_frame_all_fields_zeroed() {
        let h = EnipHeader {
            command: 0x0000,
            length: 0,
            session_handle: 0,
            status: 0,
            sender_context: [0u8; 8],
            options: 0,
        };
        assert!(!is_valid_enip_frame(&h));
    }
}
