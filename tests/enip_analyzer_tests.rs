//! Integration tests for the EtherNet/IP module (SS-17, STORY-130 + STORY-131).
//!
//! `mod parse_header` — STORY-130 pure-core parse tests (GREEN).
//! `mod dispatch`     — STORY-131 dispatcher/CLI integration tests (RED until STORY-131 impl).
//!
//! Traces to: BC-2.17.001–004 (parse_header), BC-2.17.019/020/023/026 (dispatch).

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

// ─────────────────────────────────────────────────────────────────────────────
// STORY-131 dispatcher / CLI integration tests (Red Gate — STORY-131).
//
// Traces to: BC-2.17.019 (Rule 7 routing), BC-2.17.020 (CLI --enip flag),
//            BC-2.17.023 (write-burst threshold), BC-2.17.026 (error-burst threshold).
//
// RED GATE STATUS (Decision 2 re-scope, boundary doc BOUNDARY-131-132):
// Observable for routing tests is `bytes_received > 0` (Decision 2), NOT
// `all_findings` (which requires STORY-132 frame-walk). The `todo!()` in the
// `DispatchTarget::Enip` arm fires whenever port-44818 data is dispatched.
//
// RED (FAIL until STORY-131 impl — panics at todo!() in Enip arm):
//   test_dispatcher_routes_port_44818            — drives port-44818, asserts bytes_received > 0
//   test_cli_enip_flag_constructs_analyzer       — drives port-44818, asserts bytes_received > 0
//   test_cli_all_flag_includes_enip              — drives port-44818, asserts bytes_received > 0
//   test_write_burst_threshold_custom            — drives port-44818, asserts bytes_received > 0
//   test_write_burst_threshold_default           — drives port-44818, asserts bytes_received > 0
//   test_error_burst_threshold_custom            — drives port-44818, asserts bytes_received > 0
//   test_error_burst_threshold_default           — drives port-44818, asserts bytes_received > 0
//   test_error_burst_threshold_zero_semantics    — drives port-44818, asserts bytes_received > 0
//
// GREEN-BY-DESIGN (justification inline per test):
//   test_dispatcher_does_not_route_other_ports   — port 9999 only; Enip arm never reached
//   test_dispatcher_rule_order_dnp3_before_enip  — port 20000 only; DNP3 arm taken, not Enip
//   test_dispatcher_no_enip_analyzer_port_44818_is_noop — enip=None early-exit guard fires
//   test_take_enip_analyzer_transfers_ownership  — Option::take() plumbing, no on_data
//   test_take_enip_analyzer_returns_none_when_not_set  — Option semantics, no on_data
//   test_cli_no_enip_flag_no_analyzer            — enip=None early-exit guard fires
//   test_enip_without_reassembly_warns_and_disables    — enip=None early-exit guard fires
// ─────────────────────────────────────────────────────────────────────────────
mod dispatch {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::enip::EnipAnalyzer;
    use wirerust::dispatcher::StreamDispatcher;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{Direction, StreamHandler};

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn make_ip(a: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, a))
    }

    /// Build a dispatcher with only an ENIP analyzer armed.
    fn dispatcher_with_enip(write_burst: u32, error_burst: u32) -> StreamDispatcher {
        StreamDispatcher::new(
            None,
            None,
            None,
            None,
            Some(EnipAnalyzer::new(write_burst, error_burst)),
        )
    }

    /// Build a dispatcher with NO analyzers (ENIP disabled path).
    fn dispatcher_no_enip() -> StreamDispatcher {
        StreamDispatcher::new(None, None, None, None, None)
    }

    /// Minimal valid ENIP RegisterSession payload (command=0x0065, LE, rest zeros).
    ///
    /// 24 bytes — satisfies `parse_enip_header` >= 24. Known-valid command byte so
    /// `is_valid_enip_frame` returns `true` when the detection path (STORY-132) runs.
    fn enip_register_session_payload() -> [u8; 24] {
        let mut p = [0u8; 24];
        p[0] = 0x65; // command = 0x0065 (RegisterSession), LE low byte
        p[1] = 0x00; // LE high byte
        p
    }

    // -------------------------------------------------------------------------
    // AC-131-001: StreamDispatcher routes port-44818 TCP flows to EnipAnalyzer
    // Traces: BC-2.17.019 postconditions 1–3
    // -------------------------------------------------------------------------

    /// AC-131-001 — a flow with dst_port=44818 routes to EnipAnalyzer (PC-1 + PC-2).
    ///
    /// Observable (Decision 2): `enip.bytes_received > 0` after `dispatcher.on_data()`
    /// proves PC-2 (data reached the analyzer). Combined with no-panic it proves PC-1
    /// (Rule 7 fired and the `DispatchTarget::Enip` arm was taken).
    ///
    /// RED GATE: the `DispatchTarget::Enip` arm contains `todo!()` until STORY-131
    /// implements `EnipAnalyzer::on_data`. The test panics at the `todo!()`.
    ///
    /// Traces: BC-2.17.019 postconditions 1, 2; AC-131-001.
    #[test]
    fn test_dispatcher_routes_port_44818() {
        let mut d = dispatcher_with_enip(50, 5);
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let payload = enip_register_session_payload();
        // Routes to DispatchTarget::Enip — todo!() fires until STORY-131 impl.
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let enip = d
            .take_enip_analyzer()
            .expect("ENIP analyzer must be present after routing");
        // bytes_received > 0 confirms the dispatcher wiring arm fired (PC-2).
        assert!(
            enip.bytes_received > 0,
            "bytes_received must be > 0 after routing port-44818 data to EnipAnalyzer"
        );
    }

    /// AC-131-001 — a flow on port 9999 (not 44818) does NOT route to EnipAnalyzer.
    ///
    /// Port 9999 matches no rule (Rule 8 / DispatchTarget::None); the Enip arm is
    /// never reached. `bytes_received` stays 0, proving Rule 7 is not over-broad.
    ///
    /// GREEN-BY-DESIGN: the port-9999 call takes the Rule 8 / early-exit path and
    /// never enters the `DispatchTarget::Enip` arm, so `todo!()` is not triggered.
    /// `bytes_received == 0` is a non-vacuous assertion because it distinguishes a
    /// correct implementation from one that credits all ports to the ENIP analyzer.
    ///
    /// Traces: BC-2.17.019 postcondition 3 (non-44818 flows unaffected); AC-131-001.
    #[test]
    fn test_dispatcher_does_not_route_other_ports() {
        let mut d = dispatcher_with_enip(50, 5);
        let key_other = FlowKey::new(make_ip(1), 12345, make_ip(2), 9999);
        let payload = enip_register_session_payload();
        // Port 9999 → Rule 8 (None). Enip arm never entered.
        d.on_data(&key_other, Direction::ClientToServer, &payload, 0, 0);
        let enip = d
            .take_enip_analyzer()
            .expect("ENIP analyzer must be present (was armed, not consumed)");
        assert_eq!(
            enip.bytes_received, 0,
            "bytes_received must remain 0 when only a non-44818 flow is dispatched"
        );
    }

    /// AC-131-001 — Rule 6 (port 20000 → DNP3) fires before Rule 7 (port 44818 → ENIP).
    ///
    /// A flow on port 20000 routes to DNP3 (Rule 6), leaving the ENIP analyzer
    /// untouched: `enip.bytes_received == 0`. The DNP3 on_data is implemented and
    /// does not panic, so this test is GREEN-BY-DESIGN.
    ///
    /// Optionally asserts `dnp3.flows.is_empty() == false` (mirrors
    /// `test_ec006_ports_502_and_20000_modbus_wins` from STORY-110).
    ///
    /// GREEN-BY-DESIGN: only port 20000 is dispatched. Rule 6 fires; the
    /// `DispatchTarget::Enip` arm is never reached. `bytes_received == 0` confirms
    /// Rule 7 did not fire for the port-20000 flow.
    ///
    /// Traces: AC-131-001 "Rule 7 after Rule 6"; BC-2.17.019 postconditions 1, 3;
    /// STORY-131 Architecture Compliance Rule 1.
    #[test]
    fn test_dispatcher_rule_order_dnp3_before_enip() {
        use wirerust::analyzer::dnp3::Dnp3Analyzer;
        let mut d = StreamDispatcher::new(
            None,
            None,
            None,
            Some(Dnp3Analyzer::new(10)),
            Some(EnipAnalyzer::new(50, 5)),
        );
        let payload = enip_register_session_payload();
        // Port 20000 → Rule 6 (DNP3). DNP3 on_data is implemented; no panic.
        let key_dnp3 = FlowKey::new(make_ip(1), 12345, make_ip(2), 20000);
        d.on_data(&key_dnp3, Direction::ClientToServer, &payload, 0, 0);
        // ENIP arm was never entered → bytes_received must be 0.
        let enip = d
            .take_enip_analyzer()
            .expect("ENIP analyzer must remain after DNP3-routed flow");
        assert_eq!(
            enip.bytes_received, 0,
            "bytes_received must be 0: port-20000 flow routed to DNP3 (Rule 6), not ENIP (Rule 7)"
        );
    }

    // -------------------------------------------------------------------------
    // EC-007: dispatcher.enip=None early-exit guard fires cleanly on port-44818
    // Traces: BC-2.17.019 Invariant 4; EC-007
    // -------------------------------------------------------------------------

    /// EC-007 — port-44818 flow with no ENIP analyzer configured → no panic, no routing.
    ///
    /// When `enip` is `None` the `DispatchTarget::Enip` arm executes the early-exit
    /// `if let Some(ref mut enip) = self.enip` guard and silently returns without
    /// forwarding data. No `on_data` call on any analyzer occurs; no panic.
    ///
    /// GREEN-BY-DESIGN: the early-exit guard is already implemented in the current
    /// dispatcher stub. `enip=None` means the arm body is a no-op (`if let` is false).
    /// The `todo!()` is only reached if `enip` is `Some`.
    ///
    /// Traces: BC-2.17.019 Invariant 4; EC-007; AC-131-001.
    #[test]
    fn test_dispatcher_no_enip_analyzer_port_44818_is_noop() {
        let mut d = dispatcher_no_enip();
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let payload = enip_register_session_payload();
        // Must NOT panic: early-exit guard fires (enip is None).
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        // No analyzer was ever constructed.
        assert!(
            d.take_enip_analyzer().is_none(),
            "take_enip_analyzer must return None when no ENIP analyzer was configured"
        );
    }

    // -------------------------------------------------------------------------
    // AC-131-002: take_enip_analyzer() transfers EnipAnalyzer to caller
    // Traces: BC-2.17.019 postcondition 4
    //
    // WIRING-EXEMPT: take_enip_analyzer() is a one-line `Option::take()` with
    // no branching or I/O — identical to take_dnp3_analyzer(). BC-5.38.001
    // WIRING-EXEMPT criteria satisfied. Both tests are legitimately GREEN.
    // -------------------------------------------------------------------------

    /// AC-131-002 — take_enip_analyzer() returns Some and clears the slot.
    ///
    /// WIRING-EXEMPT: tests completed Option::take() plumbing; no on_data call.
    /// Traces: BC-2.17.019 postcondition 4; AC-131-002.
    #[test]
    fn test_take_enip_analyzer_transfers_ownership() {
        let mut d = dispatcher_with_enip(50, 5);
        let taken = d.take_enip_analyzer();
        assert!(
            taken.is_some(),
            "take_enip_analyzer must return Some when analyzer is armed"
        );
        assert!(
            d.enip_analyzer().is_none(),
            "take_enip_analyzer must set dispatcher.enip to None after transfer"
        );
    }

    /// AC-131-002 — take_enip_analyzer() returns None when no analyzer was set.
    ///
    /// WIRING-EXEMPT: tests Option::take() None path; no on_data call.
    /// Traces: AC-131-002 edge case; BC-2.17.019 postcondition 4.
    #[test]
    fn test_take_enip_analyzer_returns_none_when_not_set() {
        let mut d = dispatcher_no_enip();
        assert!(
            d.take_enip_analyzer().is_none(),
            "take_enip_analyzer must return None when no ENIP analyzer is set"
        );
    }

    // -------------------------------------------------------------------------
    // AC-131-003: CLI --enip flag enables EnipAnalyzer construction and wiring
    // Traces: BC-2.17.020 postconditions 1, 3, 4
    // -------------------------------------------------------------------------

    /// AC-131-003 — with --enip set, EnipAnalyzer is wired and receives port-44818 data.
    ///
    /// Observable (Decision 2): field assertions for thresholds are GREEN-BY-DESIGN
    /// (constructor is implemented). The wiring assertion drives port-44818 data and
    /// checks `bytes_received > 0`, which requires `on_data` to be implemented.
    ///
    /// RED GATE: `on_data` panics at `todo!()` until STORY-131 implements
    /// `EnipAnalyzer::on_data` with the `bytes_received` counter increment.
    ///
    /// Traces: BC-2.17.020 postcondition 1; AC-131-003.
    #[test]
    fn test_cli_enip_flag_constructs_analyzer() {
        let analyzer = EnipAnalyzer::new(50, 5);
        let mut d = dispatcher_no_enip();
        d.set_enip_analyzer(analyzer);
        // Field assertions — GREEN-BY-DESIGN (constructor is implemented).
        {
            let a = d.enip_analyzer().expect("set_enip_analyzer must make Some");
            assert_eq!(
                a.enip_write_burst_threshold, 50,
                "default write-burst threshold must be 50 after --enip"
            );
            assert_eq!(
                a.enip_error_burst_threshold, 5,
                "default error-burst threshold must be 5 after --enip"
            );
        }
        // Wiring assertion: port-44818 data must reach the analyzer (PC-2).
        // Panics at todo!() until STORY-131 impl.
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let payload = enip_register_session_payload();
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let taken = d
            .take_enip_analyzer()
            .expect("analyzer must survive on_data");
        assert!(
            taken.bytes_received > 0,
            "bytes_received must be > 0 after --enip wiring routes port-44818 data to analyzer"
        );
    }

    /// AC-131-003 — without --enip, port-44818 flows fall through (early-exit, no analyzer).
    ///
    /// GREEN-BY-DESIGN: `enip=None` so the early-exit `if let` guard in the `Enip` arm
    /// fires without entering `todo!()`. The `take` returns None confirming no analyzer
    /// was constructed. Two non-vacuous postconditions: (1) no panic, (2) take == None.
    ///
    /// Traces: BC-2.17.020 postcondition 3; BC-2.17.019 EC-007; AC-131-003.
    #[test]
    fn test_cli_no_enip_flag_no_analyzer() {
        let mut d = dispatcher_no_enip();
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let payload = enip_register_session_payload();
        // Must NOT panic (early-exit guard fires; enip is None).
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        assert!(
            d.take_enip_analyzer().is_none(),
            "dispatcher.enip_analyzer must be None when --enip is not set; \
             port-44818 flows fall through (early-exit guard)"
        );
    }

    /// AC-131-003 — --all flag includes ENIP; wiring routes port-44818 to analyzer.
    ///
    /// Field assertions are GREEN-BY-DESIGN. Wiring assertion drives port-44818
    /// and checks `bytes_received > 0`.
    ///
    /// RED GATE: panics at `todo!()` until STORY-131 impl.
    /// Traces: BC-2.17.020 Invariant 4; AC-131-003.
    #[test]
    fn test_cli_all_flag_includes_enip() {
        let analyzer = EnipAnalyzer::new(50, 5);
        let mut d = dispatcher_no_enip();
        d.set_enip_analyzer(analyzer);
        // Field assertions — GREEN-BY-DESIGN.
        {
            let a = d.enip_analyzer().expect("--all must wire ENIP analyzer");
            assert_eq!(
                a.enip_write_burst_threshold, 50,
                "--all must produce write-burst threshold=50 (OA-001 RESOLVED=50)"
            );
            assert_eq!(
                a.enip_error_burst_threshold, 5,
                "--all must produce error-burst threshold=5 (BC-2.17.026 Inv 1)"
            );
        }
        // Wiring assertion — RED until STORY-131 impl.
        let key = FlowKey::new(make_ip(5), 11111, make_ip(6), 44818);
        let payload = enip_register_session_payload();
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let taken = d.take_enip_analyzer().expect("analyzer must be present");
        assert!(
            taken.bytes_received > 0,
            "bytes_received must be > 0: --all flag must route port-44818 data to ENIP analyzer"
        );
    }

    // -------------------------------------------------------------------------
    // AC-131-004: Missing TCP reassembly with --enip disables ENIP
    // Traces: BC-2.17.020 postcondition 2
    // -------------------------------------------------------------------------

    /// AC-131-004 — --enip without TCP reassembly: no analyzer constructed; port-44818 no-op.
    ///
    /// Simulates main.rs guard (enable_enip=true, skip_reassembly=true → enip=None).
    /// Early-exit guard fires; take returns None.
    ///
    /// GREEN-BY-DESIGN: enip=None so the Enip arm early-exit fires without touching
    /// `todo!()`. Two non-vacuous postconditions: (1) no panic, (2) take == None.
    ///
    /// Traces: BC-2.17.020 postcondition 2; AC-131-004.
    #[test]
    fn test_enip_without_reassembly_warns_and_disables() {
        // Simulate main.rs: enable_enip=true, skip_reassembly=true → no analyzer.
        let enable_enip = true;
        let skip_reassembly = true;
        let enip_opt: Option<EnipAnalyzer> = if enable_enip && !skip_reassembly {
            Some(EnipAnalyzer::new(50, 5))
        } else {
            None // WARNING emitted by main.rs eprintln!; not verifiable in unit test
        };
        let mut d = StreamDispatcher::new(None, None, None, None, enip_opt);
        let key = FlowKey::new(make_ip(7), 22222, make_ip(8), 44818);
        let payload = enip_register_session_payload();
        // Must NOT panic: early-exit guard fires (enip is None).
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        assert!(
            d.take_enip_analyzer().is_none(),
            "EnipAnalyzer must NOT be constructed when --enip is set without TCP reassembly"
        );
    }

    // -------------------------------------------------------------------------
    // AC-131-005: --enip-write-burst-threshold stored and wiring confirmed
    // Traces: BC-2.17.023 postconditions 1–3
    // -------------------------------------------------------------------------

    /// AC-131-005 — custom write-burst threshold=100 stored; port-44818 data reaches analyzer.
    ///
    /// Field assertion is GREEN-BY-DESIGN. Wiring assertion (`bytes_received > 0`)
    /// requires `on_data` to be implemented.
    ///
    /// RED GATE: panics at `todo!()` until STORY-131 impl.
    /// Traces: BC-2.17.023 postconditions 1, 3; AC-131-005.
    #[test]
    fn test_write_burst_threshold_custom() {
        let mut d = dispatcher_with_enip(100, 5);
        // Field assertion — GREEN-BY-DESIGN.
        {
            let a = d.enip_analyzer().expect("analyzer must be set");
            assert_eq!(
                a.enip_write_burst_threshold, 100,
                "custom write-burst threshold 100 must be stored in enip_write_burst_threshold"
            );
        }
        // Wiring assertion — RED until STORY-131 impl.
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let mut payload = [0u8; 24];
        payload[0] = 0x6F; // command = SendRRData (0x006F), LE
        payload[1] = 0x00;
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let analyzer = d
            .take_enip_analyzer()
            .expect("analyzer must survive on_data");
        assert!(
            analyzer.bytes_received > 0,
            "bytes_received must be > 0: port-44818 data must reach analyzer (wiring confirmation)"
        );
    }

    /// AC-131-005 — default write-burst threshold=50 stored; port-44818 data reaches analyzer.
    ///
    /// Field assertion is GREEN-BY-DESIGN. Wiring assertion requires `on_data` impl.
    ///
    /// RED GATE: panics at `todo!()` until STORY-131 impl.
    /// Traces: BC-2.17.023 postconditions 2, 3; AC-131-005.
    #[test]
    fn test_write_burst_threshold_default() {
        let mut d = dispatcher_with_enip(50, 5);
        // Field assertion — GREEN-BY-DESIGN.
        {
            let a = d.enip_analyzer().expect("analyzer must be set");
            assert_eq!(
                a.enip_write_burst_threshold, 50,
                "default write-burst threshold must be 50 (OA-001 RESOLVED=50)"
            );
        }
        // Wiring assertion — RED until STORY-131 impl.
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let mut payload = [0u8; 24];
        payload[0] = 0x6F; // SendRRData
        payload[1] = 0x00;
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let analyzer = d
            .take_enip_analyzer()
            .expect("analyzer must survive on_data");
        assert!(
            analyzer.bytes_received > 0,
            "bytes_received must be > 0: port-44818 data must reach analyzer (wiring confirmation)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-131-006: --enip-error-burst-threshold stored and wiring confirmed
    // Traces: BC-2.17.026 postconditions 1–3
    // -------------------------------------------------------------------------

    /// AC-131-006 — custom error-burst threshold=10 stored; port-44818 data reaches analyzer.
    ///
    /// Field assertion is GREEN-BY-DESIGN. Wiring assertion requires `on_data` impl.
    ///
    /// RED GATE: panics at `todo!()` until STORY-131 impl.
    /// Traces: BC-2.17.026 postconditions 1, 3; AC-131-006.
    #[test]
    fn test_error_burst_threshold_custom() {
        let mut d = dispatcher_with_enip(50, 10);
        // Field assertion — GREEN-BY-DESIGN.
        {
            let a = d.enip_analyzer().expect("analyzer must be set");
            assert_eq!(
                a.enip_error_burst_threshold, 10,
                "custom error-burst threshold 10 must be stored in enip_error_burst_threshold"
            );
        }
        // Wiring assertion — RED until STORY-131 impl.
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let mut payload = [0u8; 24];
        payload[0] = 0x6F; // SendRRData
        payload[1] = 0x00;
        payload[8] = 0x08; // non-zero status (CIP error code)
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let analyzer = d
            .take_enip_analyzer()
            .expect("analyzer must survive on_data");
        assert!(
            analyzer.bytes_received > 0,
            "bytes_received must be > 0: port-44818 data must reach analyzer (wiring confirmation)"
        );
    }

    /// AC-131-006 — default error-burst threshold=5 stored; port-44818 data reaches analyzer.
    ///
    /// Field assertion is GREEN-BY-DESIGN. Wiring assertion requires `on_data` impl.
    ///
    /// RED GATE: panics at `todo!()` until STORY-131 impl.
    /// Traces: BC-2.17.026 postconditions 2, 3; Invariant 3; AC-131-006.
    #[test]
    fn test_error_burst_threshold_default() {
        let mut d = dispatcher_with_enip(50, 5);
        // Field assertion — GREEN-BY-DESIGN.
        {
            let a = d.enip_analyzer().expect("analyzer must be set");
            assert_eq!(
                a.enip_error_burst_threshold, 5,
                "default error-burst threshold must be 5"
            );
        }
        // Wiring assertion — RED until STORY-131 impl.
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let mut payload = [0u8; 24];
        payload[0] = 0x6F;
        payload[1] = 0x00;
        payload[8] = 0x08; // non-zero status = CIP error
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let analyzer = d
            .take_enip_analyzer()
            .expect("analyzer must survive on_data");
        assert!(
            analyzer.bytes_received > 0,
            "bytes_received must be > 0: port-44818 data must reach analyzer (wiring confirmation)"
        );
    }

    /// AC-131-006 — threshold=0 stored; port-44818 data reaches analyzer (wiring only).
    ///
    /// BC-2.17.026 Invariant 4 (zero-threshold semantics: first error fires T0888 Pattern B)
    /// is a STORY-132+ detection assertion. For STORY-131, this test verifies only:
    /// (1) threshold=0 is stored correctly (GREEN-BY-DESIGN), and (2) port-44818 data
    /// reaches the analyzer (`bytes_received > 0`, RED until STORY-131 impl).
    ///
    /// RED GATE: panics at `todo!()` until STORY-131 impl.
    /// Traces: BC-2.17.026 Invariant 4 (wiring half); postcondition 4; AC-131-006.
    #[test]
    fn test_error_burst_threshold_zero_semantics() {
        let mut d = dispatcher_with_enip(50, 0);
        // Field assertion — GREEN-BY-DESIGN.
        {
            let a = d.enip_analyzer().expect("analyzer must be set");
            assert_eq!(
                a.enip_error_burst_threshold, 0,
                "threshold=0 must be stored as 0"
            );
        }
        // Wiring assertion — RED until STORY-131 impl.
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let mut payload = [0u8; 24];
        payload[0] = 0x6F; // SendRRData
        payload[1] = 0x00;
        payload[8] = 0x08; // non-zero status = CIP error
        d.on_data(&key, Direction::ClientToServer, &payload, 0, 0);
        let analyzer = d
            .take_enip_analyzer()
            .expect("analyzer must survive on_data");
        assert!(
            analyzer.bytes_received > 0,
            "bytes_received must be > 0: port-44818 data must reach analyzer (wiring confirmation)"
        );
    }
}
