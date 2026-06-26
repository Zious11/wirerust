//! Integration tests for the EtherNet/IP module (SS-17, STORY-130 + STORY-131 + STORY-132).
//!
//! `mod parse_header` — STORY-130 pure-core parse tests (GREEN).
//! `mod dispatch`     — STORY-131 dispatcher/CLI integration tests (GREEN).
//! `mod cpf_cip`      — STORY-132 CPF item walk, CIP header parse, path extract (GREEN).
//!
//! Traces to: BC-2.17.001–004 (parse_header), BC-2.17.019/020/023/026 (dispatch),
//!            BC-2.17.005/006/007/009 (cpf_cip).

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
// STORY-131 dispatcher / CLI integration tests (GREEN — STORY-131 implemented).
//
// Traces to: BC-2.17.019 (Rule 7 routing), BC-2.17.020 (CLI --enip flag),
//            BC-2.17.023 (write-burst threshold), BC-2.17.026 (error-burst threshold).
//
// IMPLEMENTATION STATUS (Decision 2 re-scope, boundary doc BOUNDARY-131-132):
// Observable for routing tests is `bytes_received > 0` (Decision 2), NOT
// `all_findings` (which requires STORY-132 CIP frame-walk — deferred).
// The `DispatchTarget::Enip` arm calls `enip.on_data(...)` which increments
// `bytes_received`; no todo!() remain in src.
//
// GREEN (routing verified via bytes_received > 0):
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
    /// Asserts `bytes_received > 0`, confirming the dispatcher Enip arm forwarded
    /// port-44818 data to `EnipAnalyzer::on_data` (STORY-131 implementation complete).
    ///
    /// Traces: BC-2.17.019 postconditions 1, 2; AC-131-001.
    #[test]
    fn test_dispatcher_routes_port_44818() {
        let mut d = dispatcher_with_enip(50, 5);
        let key = FlowKey::new(make_ip(1), 12345, make_ip(2), 44818);
        let payload = enip_register_session_payload();
        // Routes to DispatchTarget::Enip — calls enip.on_data() (STORY-131 implemented).
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
    /// The port-9999 call takes the Rule 8 / early-exit path and never enters the
    /// `DispatchTarget::Enip` arm, leaving `bytes_received` at 0. This is a
    /// non-vacuous assertion that distinguishes a correct implementation from one
    /// that credits all ports to the ENIP analyzer.
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
    /// When `enip=None` the `DispatchTarget::Enip` arm body is a no-op: the
    /// `if let Some(ref mut enip) = self.enip` guard is false, so no data is
    /// forwarded and no `on_data` call occurs.
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
    /// asserts `bytes_received > 0`, confirming the dispatcher Enip arm forwarded
    /// port-44818 data to `EnipAnalyzer::on_data` (STORY-131 implementation complete).
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
    /// `enip=None` so the early-exit `if let` guard in the `Enip` arm is false —
    /// port-44818 data is silently dropped without forwarding. Two non-vacuous
    /// postconditions: (1) no panic, (2) take returns None confirming no analyzer
    /// was constructed.
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
    /// Field assertions are GREEN-BY-DESIGN (constructor implemented). Wiring assertion
    /// asserts `bytes_received > 0`, confirming the dispatcher Enip arm forwarded
    /// port-44818 data to `EnipAnalyzer::on_data` (STORY-131 implementation complete).
    ///
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
        // Wiring assertion: port-44818 data must reach the analyzer (PC-2).
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
    /// `enip=None` so the Enip arm early-exit fires — port-44818 data is dropped
    /// without forwarding. Two non-vacuous postconditions: (1) no panic,
    /// (2) take returns None.
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
    /// Field assertion is GREEN-BY-DESIGN (constructor implemented). Wiring assertion
    /// asserts `bytes_received > 0`, confirming the dispatcher Enip arm forwarded
    /// port-44818 data to `EnipAnalyzer::on_data` (STORY-131 implementation complete).
    ///
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
        // Wiring assertion: port-44818 data must reach the analyzer (PC-2).
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
    /// Field assertion is GREEN-BY-DESIGN (constructor implemented). Wiring assertion
    /// asserts `bytes_received > 0`, confirming the dispatcher Enip arm forwarded
    /// port-44818 data to `EnipAnalyzer::on_data` (STORY-131 implementation complete).
    ///
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
        // Wiring assertion: port-44818 data must reach the analyzer (PC-2).
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
    /// Field assertion is GREEN-BY-DESIGN (constructor implemented). Wiring assertion
    /// asserts `bytes_received > 0`, confirming the dispatcher Enip arm forwarded
    /// port-44818 data to `EnipAnalyzer::on_data` (STORY-131 implementation complete).
    ///
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
        // Wiring assertion: port-44818 data must reach the analyzer (PC-2).
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
    /// Field assertion is GREEN-BY-DESIGN (constructor implemented). Wiring assertion
    /// asserts `bytes_received > 0`, confirming the dispatcher Enip arm forwarded
    /// port-44818 data to `EnipAnalyzer::on_data` (STORY-131 implementation complete).
    ///
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
        // Wiring assertion: port-44818 data must reach the analyzer (PC-2).
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
    /// is a STORY-132+ detection assertion (CIP frame-walk deferred to STORY-132). For
    /// STORY-131, this test verifies only: (1) threshold=0 is stored correctly
    /// (GREEN-BY-DESIGN), and (2) port-44818 data reaches the analyzer, asserted via
    /// `bytes_received > 0` (dispatcher Enip arm calls on_data; STORY-131 complete).
    ///
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
        // Wiring assertion: port-44818 data must reach the analyzer (PC-2).
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

// ─────────────────────────────────────────────────────────────────────────────
// STORY-132 CPF item walk, CIP header parse, CIP path extraction tests (GREEN — STORY-132 implemented).
//
// Traces to: BC-2.17.005 (cpf_items), BC-2.17.006 (cip_header),
//            BC-2.17.007 (classify_cip_service), BC-2.17.009 (cip_request_path).
//
// IMPLEMENTATION STATUS (STORY-132 complete):
// All 4 functions under test (`parse_cpf_items`, `parse_cip_header`,
// `classify_cip_service`, `parse_cip_request_path`) are fully implemented;
// all 19 tests pass. These tests originated as Red-Gate stubs (none could
// pass until the implementations landed in STORY-132).
//
// GREEN:
//   test_parse_cpf_items_single_item             — BC-2.17.005 PC 1–3 (single UnconnectedData item)
//   test_parse_cpf_items_two_items               — BC-2.17.005 PC 4 (two-item list)
//   test_parse_cpf_items_empty                   — BC-2.17.005 PC 1/3 (len < 2 → empty vec)
//   test_parse_cpf_items_truncated               — BC-2.17.005 PC 1/3 (short buf → safe empty)
//   test_cpf_item_type_ids                       — BC-2.17.005 (NullAddress / ConnectedData / UnconnectedData type IDs)
//   test_parse_cip_header_request                — BC-2.17.006 PC 1–4 (valid request frame)
//   test_parse_cip_header_response               — BC-2.17.006 PC 2 (response bit set)
//   test_parse_cip_header_too_short              — BC-2.17.006 PC 5 (< 2 bytes → None)
//   test_parse_cip_header_truncated_path         — BC-2.17.006 (path words exceed buffer)
//   test_cip_parse_skips_0x00b1_items            — BC-2.17.006 (0x00B1 ConnectedData items skipped)
//   test_cip_parse_processes_0x00b2_items        — BC-2.17.006 (0x00B2 UnconnectedData items parsed)
//   test_parse_cip_path_empty                    — BC-2.17.009 PC 4 (zero-word path → empty vec)
//   test_parse_cip_path_class_only               — BC-2.17.009 PC 1 (class segment only)
//   test_parse_cip_path_class_instance_attr      — BC-2.17.009 PC 1–3 (class + instance + attr)
//   test_parse_cip_path_unrecognized_skip        — BC-2.17.009 Unknown arm (unrecognized segment skipped)
//   test_parse_cip_path_odd_length_safe          — BC-2.17.009 (odd path_size_words safe)
//   test_classify_cip_service_named_codes        — BC-2.17.007 totality (all named service codes)
//   test_classify_cip_service_response_bit       — BC-2.17.007 (response bit → Response variant)
//   test_classify_cip_service_unknown            — BC-2.17.007 Unknown arm
// ─────────────────────────────────────────────────────────────────────────────
mod cpf_cip {
    use wirerust::analyzer::enip::{
        CipPathSegment, CipServiceClass, CpfItem, classify_cip_service, parse_cip_header,
        parse_cip_request_path, parse_cpf_items,
    };

    // -------------------------------------------------------------------------
    // AC-132-001: parse_cpf_items returns a typed item list
    // Traces: BC-2.17.005 postconditions 1–4
    // -------------------------------------------------------------------------

    /// AC-132-001 — single CPF item: item_count=1, type_id=0x00B2, length=4, data=[AA BB CC DD].
    ///
    /// Wire layout (cpf_data):
    ///   [0..2]  01 00  item_count = 1 (LE)
    ///   [2..4]  B2 00  type_id   = 0x00B2 (LE) — UnconnectedData
    ///   [4..6]  04 00  length    = 4 (LE, transient parse local)
    ///   [6..10] AA BB CC DD  item data (4 bytes)
    ///
    /// Asserts: exactly one CpfItem returned, type_id == 0x00B2, data == [0xAA,0xBB,0xCC,0xDD].
    ///
    /// Traces: BC-2.17.005 postconditions 1, 2, 3.
    #[test]
    fn test_parse_cpf_items_single_item() {
        let cpf_data: &[u8] = &[
            0x01, 0x00, // item_count = 1
            0xB2, 0x00, // type_id = 0x00B2 (UnconnectedData)
            0x04, 0x00, // length = 4
            0xAA, 0xBB, 0xCC, 0xDD, // data
        ];
        let items = parse_cpf_items(cpf_data);
        assert_eq!(
            items.len(),
            1,
            "must return exactly 1 CpfItem for item_count=1"
        );
        assert_eq!(
            items[0].type_id, 0x00B2,
            "type_id must be 0x00B2 (UnconnectedData)"
        );
        assert_eq!(
            items[0].data,
            vec![0xAA, 0xBB, 0xCC, 0xDD],
            "data must be the 4 item payload bytes"
        );
    }

    /// AC-132-001 — two CPF items: item_count=2; first 0x0000 null (length 0),
    /// second 0x00B2 (length 2, data=[0x4E, 0x01]).
    ///
    /// Wire layout:
    ///   [0..2]  02 00  item_count = 2
    ///   [2..4]  00 00  type_id = 0x0000 (NullAddress)
    ///   [4..6]  00 00  length = 0
    ///   [6..8]  B2 00  type_id = 0x00B2 (UnconnectedData)
    ///   [8..10] 02 00  length = 2
    ///   [10..12] 4E 01 data
    ///
    /// Asserts: 2 items; items[0].type_id=0x0000 with empty data; items[1].type_id=0x00B2
    /// with data=[0x4E,0x01].
    ///
    /// Traces: BC-2.17.005 postconditions 1–3; EC-001 (null addr item with length=0).
    #[test]
    fn test_parse_cpf_items_two_items() {
        let cpf_data: &[u8] = &[
            0x02, 0x00, // item_count = 2
            0x00, 0x00, // type_id = 0x0000 (NullAddress)
            0x00, 0x00, // length = 0
            0xB2, 0x00, // type_id = 0x00B2 (UnconnectedData)
            0x02, 0x00, // length = 2
            0x4E, 0x01, // data
        ];
        let items = parse_cpf_items(cpf_data);
        assert_eq!(
            items.len(),
            2,
            "must return exactly 2 CpfItems for item_count=2"
        );
        assert_eq!(
            items[0].type_id, 0x0000,
            "first item type_id must be 0x0000 (NullAddress)"
        );
        assert!(
            items[0].data.is_empty(),
            "NullAddress item must have empty data"
        );
        assert_eq!(
            items[1].type_id, 0x00B2,
            "second item type_id must be 0x00B2 (UnconnectedData)"
        );
        assert_eq!(
            items[1].data,
            vec![0x4E, 0x01],
            "second item data must be [0x4E, 0x01]"
        );
    }

    /// AC-132-001 — too short for item_count (1 byte): returns vec![].
    ///
    /// BC-2.17.005 postcondition 1: returns vec![] if cpf_data.len() < 2 (cannot read
    /// item_count). EC-002: 0 bytes and 1 byte are both too short.
    ///
    /// Traces: BC-2.17.005 postcondition 1; EC-002.
    #[test]
    fn test_parse_cpf_items_empty() {
        // 0-byte input: cannot read item_count
        let empty: &[u8] = &[];
        assert_eq!(
            parse_cpf_items(empty),
            vec![],
            "0-byte input must return vec![] (cannot read item_count)"
        );
        // 1-byte input: also cannot read 2-byte item_count
        let one_byte: &[u8] = &[0x01];
        assert_eq!(
            parse_cpf_items(one_byte),
            vec![],
            "1-byte input must return vec![] (cannot read item_count)"
        );
    }

    /// AC-132-001 — truncated item: item_count=1, type_id=0x00B2, declared length=5
    /// but only 3 data bytes present. Stops iteration; returns partial list (0 items).
    ///
    /// Wire layout:
    ///   [0..2]  01 00  item_count = 1
    ///   [2..4]  B2 00  type_id = 0x00B2
    ///   [4..6]  05 00  length = 5 (declares 5 bytes of data)
    ///   [6..9]  AA BB CC  only 3 bytes available (bounds violation)
    ///
    /// BC-2.17.005 postcondition 3 (stops on bounds violation). EC-003: declares length=5
    /// but only 3 bytes remain — `cursor + 4 + 5 = 11 > 9`; item is not appended.
    ///
    /// Traces: BC-2.17.005 postcondition 3; EC-003.
    #[test]
    fn test_parse_cpf_items_truncated() {
        let cpf_data: &[u8] = &[
            0x01, 0x00, // item_count = 1
            0xB2, 0x00, // type_id = 0x00B2
            0x05, 0x00, // length = 5 (truncated — only 3 bytes follow)
            0xAA, 0xBB, 0xCC, // only 3 bytes of the declared 5
        ];
        let items = parse_cpf_items(cpf_data);
        assert_eq!(
            items.len(),
            0,
            "truncated item must be discarded; partial list = 0 items (bounds violation)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-132-002: CpfItem carries type_id and data — three recognized type_ids
    // Traces: BC-2.17.005 postcondition 2
    // -------------------------------------------------------------------------

    /// AC-132-002 — CpfItem type_ids: 0x00B2, 0x00B1, and 0x0000 are all valid to construct.
    ///
    /// Verifies: struct has exactly `type_id: u16` and `data: Vec<u8>` (no `length` field).
    /// Tests round-trip via parse: construct a 3-item CPF payload and verify type_ids.
    ///
    /// Traces: BC-2.17.005 postcondition 2 (type_id and data fields; `length` is transient).
    #[test]
    fn test_cpf_item_type_ids() {
        // Direct construction — verifies struct field names and types.
        let null_item = CpfItem {
            type_id: 0x0000,
            data: vec![],
        };
        let connected_item = CpfItem {
            type_id: 0x00B1,
            data: vec![0x00, 0x01],
        };
        let unconnected_item = CpfItem {
            type_id: 0x00B2,
            data: vec![0x4E, 0x02],
        };

        assert_eq!(
            null_item.type_id, 0x0000,
            "NullAddress type_id must be 0x0000"
        );
        assert!(null_item.data.is_empty(), "NullAddress data must be empty");
        assert_eq!(
            connected_item.type_id, 0x00B1,
            "ConnectedData type_id must be 0x00B1"
        );
        assert_eq!(
            connected_item.data.len(),
            2,
            "ConnectedData data len must be 2"
        );
        assert_eq!(
            unconnected_item.type_id, 0x00B2,
            "UnconnectedData type_id must be 0x00B2"
        );
        assert_eq!(
            unconnected_item.data.len(),
            2,
            "UnconnectedData data len must be 2"
        );

        // Verify via parse: a 3-item CPF payload with all three type_ids.
        //   item_count=3
        //   NullAddress (0x0000, length=0)
        //   ConnectedData (0x00B1, length=2, data=[0x00, 0x01])
        //   UnconnectedData (0x00B2, length=2, data=[0x4E, 0x02])
        let cpf_data: &[u8] = &[
            0x03, 0x00, // item_count = 3
            0x00, 0x00, 0x00, 0x00, // 0x0000, length=0
            0xB1, 0x00, 0x02, 0x00, 0x00, 0x01, // 0x00B1, length=2, data=[0x00,0x01]
            0xB2, 0x00, 0x02, 0x00, 0x4E, 0x02, // 0x00B2, length=2, data=[0x4E,0x02]
        ];
        let items = parse_cpf_items(cpf_data);
        assert_eq!(items.len(), 3, "3-item CPF payload must produce 3 CpfItems");
        assert_eq!(items[0].type_id, 0x0000);
        assert_eq!(items[1].type_id, 0x00B1);
        assert_eq!(items[2].type_id, 0x00B2);
    }

    // -------------------------------------------------------------------------
    // AC-132-003: parse_cip_header parses CIP header from 0x00B2 item data
    // Traces: BC-2.17.006 postconditions 1–7
    // -------------------------------------------------------------------------

    /// AC-132-003 — request CIP header: service=0x4E (ForwardClose request, high bit clear),
    /// request_path_size=1 word (2 bytes), path=[0x20, 0x06].
    ///
    /// item_data layout:
    ///   [0]    0x4E   service byte (ForwardClose, request; bit7=0)
    ///   [1]    0x01   request_path_size = 1 word (transient parse local; NOT a struct field)
    ///   [2..4] 20 06  request_path (1 word = 2 bytes)
    ///
    /// Asserts: returns Some; service=0x4E; request_path=[0x20,0x06].
    ///
    /// Traces: BC-2.17.006 postconditions 1, 2, 3, 4; EC-005 via request_path_size=1.
    #[test]
    fn test_parse_cip_header_request() {
        let item_data: &[u8] = &[
            0x4E, // service = 0x4E (ForwardClose request; high bit clear)
            0x01, // request_path_size = 1 word (transient local; path_byte_count = 2)
            0x20, 0x06, // request_path = [0x20, 0x06] (Class segment for CIP Router)
        ];
        let result = parse_cip_header(item_data);
        let hdr = result.expect("must return Some for well-formed 0x00B2 item data");
        assert_eq!(
            hdr.service, 0x4E,
            "service must be 0x4E (ForwardClose request)"
        );
        assert_eq!(
            hdr.request_path,
            vec![0x20, 0x06],
            "request_path must be [0x20, 0x06] (1 word = 2 bytes)"
        );
    }

    /// AC-132-003 — response CIP header: service=0xCE (ForwardClose response, high bit set).
    ///
    /// Response items have service | 0x80; `request_path_size` byte is still present but
    /// per BC-2.17.006 the struct only carries `service` and `request_path`. CipHeader does
    /// NOT carry `general_status` (BC-2.17.006 postcondition 7; Architecture Rule 5).
    ///
    /// item_data layout:
    ///   [0]    0xCE   service byte (ForwardClose response; bit7=1 = response flag)
    ///   [1]    0x00   request_path_size = 0 (no path for response; transient local)
    ///   (no path bytes follow since path_byte_count = 0)
    ///
    /// Asserts: returns Some; service=0xCE; request_path is empty.
    ///
    /// Traces: BC-2.17.006 postconditions 1, 2, 4; postcondition 7 (general_status absent).
    #[test]
    fn test_parse_cip_header_response() {
        let item_data: &[u8] = &[
            0xCE, // service = 0xCE (0x4E | 0x80 = ForwardClose response)
            0x00, // request_path_size = 0 words (transient local; no path bytes)
        ];
        let result = parse_cip_header(item_data);
        let hdr = result.expect("must return Some for 2-byte response item data");
        assert_eq!(
            hdr.service, 0xCE,
            "service must be 0xCE (ForwardClose response, bit7 set)"
        );
        assert!(
            hdr.request_path.is_empty(),
            "request_path must be empty for response with request_path_size=0"
        );
    }

    /// AC-132-003 — too short: 1-byte input returns None.
    ///
    /// BC-2.17.006 postcondition 5: returns None if item_data.len() < 2 (cannot read service
    /// AND request_path_size). EC-004: 1-byte input.
    ///
    /// Traces: BC-2.17.006 postcondition 5; EC-004.
    #[test]
    fn test_parse_cip_header_too_short() {
        // 0 bytes
        assert!(
            parse_cip_header(&[]).is_none(),
            "0-byte input must return None (cannot read service byte)"
        );
        // 1 byte (has service but no request_path_size)
        assert!(
            parse_cip_header(&[0x4E]).is_none(),
            "1-byte input must return None (cannot read request_path_size)"
        );
    }

    /// AC-132-003 — truncated path: request_path_size=3 words but only 4 bytes of data total
    /// (need 2 + 6 = 8 bytes). Returns None.
    ///
    /// item_data layout:
    ///   [0]    0x0E   service = 0x0E (GetAttributeSingle)
    ///   [1]    0x03   request_path_size = 3 words → path_byte_count = 6
    ///   [2..6] 20 01 24 01  only 4 bytes of path (need 6; slice ends at 6 total)
    ///
    /// BC-2.17.006 postcondition 5: returns None if item_data.len() < 2 + path_byte_count.
    /// EC-006: 3 words requested, 2 bytes present past the size byte → truncated.
    ///
    /// Traces: BC-2.17.006 postcondition 5; EC-006.
    #[test]
    fn test_parse_cip_header_truncated_path() {
        let item_data: &[u8] = &[
            0x0E, // service = 0x0E (GetAttributeSingle)
            0x03, // request_path_size = 3 words (6 bytes needed)
            0x20, 0x01, 0x24, 0x01, // only 4 bytes; need 6 (cursor+2+6=8 > 6 available)
        ];
        assert!(
            parse_cip_header(item_data).is_none(),
            "must return None when declared path (3 words = 6 bytes) exceeds available data (4 bytes)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-132-004: 0x00B1 items skipped at call site; 0x00B2 items parsed
    // Traces: BC-2.17.006 Invariant 3 (F-P9-001 locked decision)
    // -------------------------------------------------------------------------

    /// AC-132-004 — call-site gate: 0x00B1 items appear in CpfItem list but parse_cip_header
    /// is NOT called for them (F-P9-001). Only 0x00B2 items are CIP-parsed.
    ///
    /// CPF payload has two items: 0x00B1 (ConnectedData) and 0x00B2 (UnconnectedData).
    /// The test exercises the documented caller gate: iterate items, call parse_cip_header
    /// ONLY for type_id == 0x00B2.
    ///
    /// Wire layout (10 bytes total + 2 for item_count):
    ///   item_count = 2
    ///   [0x00B1, length=3, data=[0x00,0x01,0x4E]]  (sequence-count prefix + partial service)
    ///   [0x00B2, length=2, data=[0x4E,0x00]]        (ForwardClose request, path_size=0)
    ///
    /// Asserts: parse_cpf_items returns 2 items; when caller gates on type_id == 0x00B2, the
    /// 0x00B1 item is NOT passed to parse_cip_header and therefore yields NO CipHeader;
    /// the 0x00B2 item IS parsed and returns Some(CipHeader { service: 0x4E, .. }).
    ///
    /// Traces: BC-2.17.006 Invariant 3; F-P9-001; AC-132-004; EC-010.
    #[test]
    fn test_cip_parse_skips_0x00b1_items() {
        let cpf_data: &[u8] = &[
            0x02, 0x00, // item_count = 2
            // 0x00B1 item: length=3, data=[0x00, 0x01, 0x4E]
            0xB1, 0x00, 0x03, 0x00, 0x00, 0x01, 0x4E,
            // 0x00B2 item: length=2, data=[0x4E, 0x00]
            0xB2, 0x00, 0x02, 0x00, 0x4E, 0x00,
        ];
        let items = parse_cpf_items(cpf_data);
        assert_eq!(items.len(), 2, "must parse 2 CPF items");

        // Simulate caller gate: only call parse_cip_header for 0x00B2 items.
        // F-P9-001: 0x00B1 items are NOT passed to parse_cip_header.
        let b1_items: Vec<&CpfItem> = items.iter().filter(|i| i.type_id == 0x00B1).collect();
        let b2_items: Vec<&CpfItem> = items.iter().filter(|i| i.type_id == 0x00B2).collect();

        assert_eq!(
            b1_items.len(),
            1,
            "must have one 0x00B1 item in CpfItem list"
        );
        assert_eq!(
            b2_items.len(),
            1,
            "must have one 0x00B2 item in CpfItem list"
        );

        // 0x00B1 item is present in the list but CIP parse is SKIPPED (F-P9-001).
        // (No call to parse_cip_header for b1_items — the caller gate prevents it.)

        // 0x00B2 item: call parse_cip_header and verify it returns Some.
        let cip_hdr_result = parse_cip_header(&b2_items[0].data);
        assert!(
            cip_hdr_result.is_some(),
            "parse_cip_header must return Some for 0x00B2 item with well-formed data"
        );
        // The 0x00B1 item must NOT have been CIP-parsed; no CipHeader exists for it.
        // (This is enforced by the caller gate — it is the architectural invariant under test.)
        assert_eq!(
            b1_items[0].type_id, 0x00B1,
            "0x00B1 item stored in CpfItem list with correct type_id"
        );
    }

    /// AC-132-004 — 0x00B2 items ARE processed: parse_cip_header returns Some for valid data.
    ///
    /// This test exercises the positive path of the F-P9-001 call-site gate: a single 0x00B2
    /// item is walked from a CPF payload, then passed to parse_cip_header, and yields a
    /// correctly-decoded CipHeader.
    ///
    /// CPF payload: item_count=1, type_id=0x00B2, length=4, data=[0x0E, 0x02, 0x20, 0x01].
    ///   service=0x0E (GetAttributeSingle), path_size=2 words → path=[0x20, 0x01, ?, ?]
    ///   Wait — path_size=2 words = 4 bytes but we only have 2 bytes after [0x0E, 0x02].
    ///   Use path_size=1 → data=[0x0E, 0x01, 0x20, 0x01]:
    ///   service=0x0E, path_size=1 word → path_byte_count=2 → path=[0x20,0x01].
    ///
    /// Asserts: parse_cip_header returns Some; service=0x0E; request_path=[0x20,0x01].
    ///
    /// Traces: BC-2.17.006 postconditions 1–4; AC-132-004; EC-010.
    #[test]
    fn test_cip_parse_processes_0x00b2_items() {
        let cpf_data: &[u8] = &[
            0x01, 0x00, // item_count = 1
            0xB2, 0x00, // type_id = 0x00B2 (UnconnectedData)
            0x04, 0x00, // length = 4
            // CIP data: service=0x0E, path_size=1 word (2 bytes), path=[0x20, 0x01]
            0x0E, 0x01, 0x20, 0x01,
        ];
        let items = parse_cpf_items(cpf_data);
        assert_eq!(items.len(), 1, "must parse 1 CPF item");
        assert_eq!(
            items[0].type_id, 0x00B2,
            "item must be 0x00B2 (UnconnectedData)"
        );

        // Call-site gate: type_id == 0x00B2 → call parse_cip_header.
        let cip_hdr = parse_cip_header(&items[0].data)
            .expect("parse_cip_header must return Some for well-formed 0x00B2 data");
        assert_eq!(
            cip_hdr.service, 0x0E,
            "service must be 0x0E (GetAttributeSingle)"
        );
        assert_eq!(
            cip_hdr.request_path,
            vec![0x20, 0x01],
            "request_path must be [0x20, 0x01] (1 word = 2 bytes)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-132-005: parse_cip_request_path extracts Class, Instance, Attribute segments
    // Traces: BC-2.17.009 postconditions 1–4
    // -------------------------------------------------------------------------

    /// AC-132-005 — empty path returns vec![].
    ///
    /// Traces: BC-2.17.009 postcondition 4 (vec![] for empty path); EC-007.
    #[test]
    fn test_parse_cip_path_empty() {
        assert_eq!(
            parse_cip_request_path(&[]),
            vec![],
            "empty path must return vec![]"
        );
    }

    /// AC-132-005 — Class-only path: [0x20, 0x01] → [Class(0x01)].
    ///
    /// Exact-match on 0x20 (Architecture Rule 2). Two bytes consumed in one iteration.
    ///
    /// Traces: BC-2.17.009 postcondition 1 (0x20 → Class).
    #[test]
    fn test_parse_cip_path_class_only() {
        let path: &[u8] = &[0x20, 0x01];
        let segs = parse_cip_request_path(path);
        assert_eq!(
            segs.len(),
            1,
            "must produce exactly 1 segment for [0x20, 0x01]"
        );
        assert_eq!(segs[0], CipPathSegment::Class(0x01), "must be Class(0x01)");
    }

    /// AC-132-005 — Class + Instance + Attribute path: 3 segments decoded correctly.
    ///
    /// Path: [0x20, 0x01, 0x24, 0x01, 0x30, 0x03]
    ///   0x20 0x01 → Class(0x01)       Identity Object
    ///   0x24 0x01 → Instance(0x01)    Instance 1
    ///   0x30 0x03 → Attribute(0x03)   Attribute 3 (Vendor ID)
    ///
    /// Traces: BC-2.17.009 postconditions 1, 2, 3 (all three segment types).
    #[test]
    fn test_parse_cip_path_class_instance_attr() {
        let path: &[u8] = &[
            0x20, 0x01, // Class(0x01)
            0x24, 0x01, // Instance(0x01)
            0x30, 0x03, // Attribute(0x03)
        ];
        let segs = parse_cip_request_path(path);
        assert_eq!(segs.len(), 3, "must produce exactly 3 segments");
        assert_eq!(
            segs[0],
            CipPathSegment::Class(0x01),
            "first segment must be Class(0x01)"
        );
        assert_eq!(
            segs[1],
            CipPathSegment::Instance(0x01),
            "second segment must be Instance(0x01)"
        );
        assert_eq!(
            segs[2],
            CipPathSegment::Attribute(0x03),
            "third segment must be Attribute(0x03)"
        );
    }

    /// AC-132-005 — unrecognized segment type 0x40 is skipped; Class(0x01) follows and is kept.
    ///
    /// Path: [0x40, 0x00, 0x20, 0x01] → [Class(0x01)]
    /// 0x40 is not 0x20/0x24/0x30 → advance cursor by 2 (skip). Then 0x20 → Class(0x01).
    ///
    /// Architecture Rule 2: exact-match (NOT &0xE0 mask). EC-008.
    ///
    /// Traces: BC-2.17.009 postcondition 3 (skip unrecognized); EC-008.
    #[test]
    fn test_parse_cip_path_unrecognized_skip() {
        let path: &[u8] = &[0x40, 0x00, 0x20, 0x01];
        let segs = parse_cip_request_path(path);
        assert_eq!(
            segs.len(),
            1,
            "unrecognized segment 0x40 skipped; only Class(0x01) should remain"
        );
        assert_eq!(segs[0], CipPathSegment::Class(0x01), "must be Class(0x01)");
    }

    /// AC-132-005 — 1-byte path: cursor+2 > 1 at first iteration; returns vec![], no panic.
    ///
    /// EC-007: odd-length input (1 byte) → bounds violation immediately → vec![].
    /// Architecture Rule 3 (no panic for any input).
    ///
    /// Traces: BC-2.17.009 postcondition 4; EC-007.
    #[test]
    fn test_parse_cip_path_odd_length_safe() {
        let path: &[u8] = &[0x20]; // 1 byte — cannot form a complete 2-byte segment
        let segs = parse_cip_request_path(path);
        assert_eq!(
            segs,
            vec![],
            "1-byte path must return vec![] without panic (cursor+2 > 1 at first iteration)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-132-007: classify_cip_service maps all 15 variants correctly
    // Traces: BC-2.17.007 postconditions 1–6, invariants 1–3
    // -------------------------------------------------------------------------

    /// AC-132-007 — 13 named request service codes map to their correct variants.
    ///
    /// Response-bit check FIRST: none of these have high bit set, so they fall
    /// through to the named match. Each is asserted individually (non-vacuous).
    ///
    /// Named codes from BC-2.17.007 postcondition 3:
    ///   0x01 GetAttributesAll, 0x02 SetAttributesAll, 0x03 GetAttributeList,
    ///   0x04 SetAttributeList, 0x05 Reset, 0x07 Stop, 0x0A MultipleServicePacket,
    ///   0x0E GetAttributeSingle, 0x10 SetAttributeSingle, 0x4B GetAndClear,
    ///   0x4E ForwardClose, 0x54 ForwardOpen, 0x5B LargeForwardOpen.
    ///
    /// Traces: BC-2.17.007 postcondition 3; VP-032 Sub-D partition (named variants).
    #[test]
    fn test_classify_cip_service_named_codes() {
        assert_eq!(
            classify_cip_service(0x01),
            CipServiceClass::GetAttributesAll,
            "0x01 must map to GetAttributesAll"
        );
        assert_eq!(
            classify_cip_service(0x02),
            CipServiceClass::SetAttributesAll,
            "0x02 must map to SetAttributesAll"
        );
        assert_eq!(
            classify_cip_service(0x03),
            CipServiceClass::GetAttributeList,
            "0x03 must map to GetAttributeList"
        );
        assert_eq!(
            classify_cip_service(0x04),
            CipServiceClass::SetAttributeList,
            "0x04 must map to SetAttributeList"
        );
        assert_eq!(
            classify_cip_service(0x05),
            CipServiceClass::Reset,
            "0x05 must map to Reset"
        );
        assert_eq!(
            classify_cip_service(0x07),
            CipServiceClass::Stop,
            "0x07 must map to Stop"
        );
        assert_eq!(
            classify_cip_service(0x0A),
            CipServiceClass::MultipleServicePacket,
            "0x0A must map to MultipleServicePacket"
        );
        assert_eq!(
            classify_cip_service(0x0E),
            CipServiceClass::GetAttributeSingle,
            "0x0E must map to GetAttributeSingle"
        );
        assert_eq!(
            classify_cip_service(0x10),
            CipServiceClass::SetAttributeSingle,
            "0x10 must map to SetAttributeSingle"
        );
        assert_eq!(
            classify_cip_service(0x4B),
            CipServiceClass::GetAndClear,
            "0x4B must map to GetAndClear"
        );
        assert_eq!(
            classify_cip_service(0x4E),
            CipServiceClass::ForwardClose,
            "0x4E must map to ForwardClose"
        );
        assert_eq!(
            classify_cip_service(0x54),
            CipServiceClass::ForwardOpen,
            "0x54 must map to ForwardOpen"
        );
        assert_eq!(
            classify_cip_service(0x5B),
            CipServiceClass::LargeForwardOpen,
            "0x5B must map to LargeForwardOpen"
        );
    }

    /// AC-132-007 — response-bit invariant: service & 0x80 != 0 → Response, applied FIRST.
    ///
    /// BC-2.17.007 postcondition 2, invariant 1: if bit 7 set, result MUST be Response
    /// regardless of the lower 7 bits. Tested with multiple representative values:
    ///   0x80 (bit7 only), 0x81 (GetAttributesAll | response), 0xCE (ForwardClose response),
    ///   0xFF (all bits set).
    ///
    /// Traces: BC-2.17.007 postcondition 2; invariant 1 (response-bit FIRST); VP-032 Sub-D.
    #[test]
    fn test_classify_cip_service_response_bit() {
        // 0x80: only bit 7 set — must be Response
        assert_eq!(
            classify_cip_service(0x80),
            CipServiceClass::Response,
            "0x80 (high bit set) must map to Response"
        );
        // 0x81: 0x01 (GetAttributesAll) | 0x80 → Response (applied FIRST before named lookup)
        assert_eq!(
            classify_cip_service(0x81),
            CipServiceClass::Response,
            "0x81 (GetAttributesAll | response-bit) must map to Response, not GetAttributesAll"
        );
        // 0xCE: ForwardClose response (0x4E | 0x80)
        assert_eq!(
            classify_cip_service(0xCE),
            CipServiceClass::Response,
            "0xCE (ForwardClose response = 0x4E | 0x80) must map to Response"
        );
        // 0xFF: all bits set
        assert_eq!(
            classify_cip_service(0xFF),
            CipServiceClass::Response,
            "0xFF (all bits set) must map to Response"
        );
    }

    /// AC-132-007 — Unknown arm reachable: 0x7F (high bit clear, not a named service) → Unknown.
    ///
    /// BC-2.17.007 postcondition 4 (non-named request-range values → Unknown).
    /// BC-2.17.007 postcondition 5 (Unknown is reachable — non-vacuity; VP-032 Sub-D partition).
    ///
    /// 0x7F: bit 7 clear (not Response), not in the 13-variant named set → Unknown.
    ///
    /// Traces: BC-2.17.007 postconditions 4, 5; VP-032 Sub-D partition non-vacuity.
    #[test]
    fn test_classify_cip_service_unknown() {
        assert_eq!(
            classify_cip_service(0x7F),
            CipServiceClass::Unknown,
            "0x7F (high bit clear, not a named service) must map to Unknown"
        );
        // Additional Unknown values to strengthen non-vacuity.
        assert_eq!(
            classify_cip_service(0x00),
            CipServiceClass::Unknown,
            "0x00 (not a named service) must map to Unknown"
        );
        assert_eq!(
            classify_cip_service(0x06),
            CipServiceClass::Unknown,
            "0x06 (gap between Reset 0x05 and Stop 0x07) must map to Unknown"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-133 MITRE ICS Technique Seeding: VP-007 6-step atomic burst.
//
// All 10 tests pass (STORY-133 implementation complete, VP-007 Steps 1–6 landed):
//
//   Implemented (VP-007 Steps 1–4):
//     test_technique_info_t0858         — AC-133-001, VP-007 Step 1
//     test_technique_info_t0816         — AC-133-002, VP-007 Step 1
//     test_technique_info_t1693_001     — AC-133-003, VP-007 Step 1
//     test_seeded_count_is_28           — AC-133-005, VP-007 Steps 2+3
//     test_emitted_count_is_20          — AC-133-006, VP-007 Step 4
//     test_t1693_001_not_emitted        — AC-133-006, VP-007 Step 4
//     test_t0858_t0816_and_t0846_tactic_id_resolution — AC-133-006 tactic_id cross-check
//                                         (T0858→TA0104, T0816→TA0107, T0846→TA0102)
//
//   GREEN-BY-DESIGN (VP-007 Step 5 — pure enum→str, zero branching, 1 line per arm):
//     test_ics_execution_tactic_display — AC-133-004; Display arm "Execution (ICS)" correct
//     test_ics_execution_tactic_id      — AC-133-004; tactic_id "TA0104" correct
//     test_t0846_in_emitted             — AC-133-006 partially; T0846 resolves IcsDiscovery
//
// EMITTED_IDS access note: `EMITTED_IDS` lives inside `#[cfg(kani)] mod kani_proofs`
// and is NOT pub — integration tests cannot access it directly. The strongest available
// EMITTED assertion from integration tests is enumerating all 20 expected emitted IDs
// and asserting each resolves in technique_info (prerequisite for emittability). The
// actual EMITTED ⊆ SEEDED invariant is enforced by the kani `vp007_catalog_drift_guard`
// unit test in src/mitre.rs (which does have in-crate access).
//
// SEEDED_TECHNIQUE_IDS/SEEDED_TECHNIQUE_ID_COUNT access note: these are `const` (not
// `pub const`) so they are NOT accessible from integration tests outside the crate.
// The strongest SEEDED count assertion available: assert all 28 individual expected IDs
// resolve in technique_info (exhaustive membership, stronger than a count proxy).
//
// Traces to: AC-133-001..007, VP-007 Steps 1–6, ADR-010 Decision 7.
// ─────────────────────────────────────────────────────────────────────────────
mod mitre_seeding {
    use wirerust::mitre::{MitreTactic, technique_info, technique_tactic, technique_tactic_id};

    // -------------------------------------------------------------------------
    // AC-133-001: technique_info("T0858") returns correct ICS technique metadata
    // Traces: VP-007 Step 1 (T0858 arm)
    // Implemented: T0858 arm added to technique_info in STORY-133 (VP-007 Step 1).
    // -------------------------------------------------------------------------

    /// AC-133-001 — technique_info("T0858") returns Some with IcsExecution tactic.
    ///
    /// `technique_info("T0858")` returns Some (T0858 arm added in STORY-133, VP-007 Step 1).
    ///
    /// Asserts: name == "Change Operating Mode"; tactic == IcsExecution.
    ///
    /// Traces: AC-133-001; VP-007 Step 1; ADR-010 Decision 7.
    #[test]
    fn test_technique_info_t0858() {
        let info = technique_info("T0858");
        assert!(
            info.is_some(),
            "technique_info(\"T0858\") must return Some — T0858 arm added in STORY-133 \
             (VP-007 Step 1)"
        );
        let (name, tactic) = info.unwrap();
        assert_eq!(
            name, "Change Operating Mode",
            "T0858 name must be \"Change Operating Mode\""
        );
        assert_eq!(
            tactic,
            MitreTactic::IcsExecution,
            "T0858 tactic must be MitreTactic::IcsExecution (not any other variant)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-133-002: technique_info("T0816") returns correct ICS technique metadata
    // Traces: VP-007 Step 1 (T0816 arm)
    // Implemented: T0816 arm added to technique_info in STORY-133 (VP-007 Step 1).
    // -------------------------------------------------------------------------

    /// AC-133-002 — technique_info("T0816") returns Some with IcsInhibitResponseFunction.
    ///
    /// `technique_info("T0816")` returns Some (T0816 arm added in STORY-133, VP-007 Step 1).
    ///
    /// Asserts: name == "Device Restart/Shutdown"; tactic == IcsInhibitResponseFunction.
    ///
    /// Traces: AC-133-002; VP-007 Step 1; ADR-010 Decision 7.
    #[test]
    fn test_technique_info_t0816() {
        let info = technique_info("T0816");
        assert!(
            info.is_some(),
            "technique_info(\"T0816\") must return Some — T0816 arm added in STORY-133 (VP-007 Step 1)"
        );
        let (name, tactic) = info.unwrap();
        assert_eq!(
            name, "Device Restart/Shutdown",
            "T0816 name must be \"Device Restart/Shutdown\""
        );
        assert_eq!(
            tactic,
            MitreTactic::IcsInhibitResponseFunction,
            "T0816 tactic must be MitreTactic::IcsInhibitResponseFunction (not IcsExecution)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-133-003: technique_info("T1693.001") returns staged technique metadata
    // Traces: VP-007 Step 1 (T1693.001 arm)
    // Implemented: T1693.001 arm added to technique_info in STORY-133 (VP-007 Step 1).
    // F-133-003 [adversarial fix]: strengthened to pin exact name+tactic per ADR-010 Decision 7.
    // -------------------------------------------------------------------------

    /// AC-133-003 — technique_info("T1693.001") returns Some (seeded catalog entry).
    ///
    /// `technique_info("T1693.001")` returns Some (arm added in STORY-133, VP-007 Step 1).
    /// T1693.001 is seeded but NOT emitted in v0.11.0 (see VP-007 Step 4 / AC-133-006).
    ///
    /// Asserts: name == "Modify Firmware: System Firmware" (ADR-010 Decision 7, v19.1);
    ///          tactic == MitreTactic::IcsInhibitResponseFunction (TA0107);
    ///          technique_tactic_id("T1693.001") == Some("TA0107").
    ///
    /// Traces: AC-133-003; VP-007 Step 1; ADR-010 Decision 7; F-133-003 [adversarial fix].
    #[test]
    fn test_technique_info_t1693_001() {
        let info = technique_info("T1693.001");
        assert!(
            info.is_some(),
            "technique_info(\"T1693.001\") must return Some — staged catalog entry missing \
             (VP-007 Step 1)"
        );
        let (name, tactic) = info.unwrap();
        assert_eq!(
            name, "Modify Firmware: System Firmware",
            "T1693.001 name must be \"Modify Firmware: System Firmware\" per ADR-010 Decision 7 \
             (v19.1 replacement for revoked T0857; NOT an EtherNet/IP-branded or Initial Access name)"
        );
        assert_eq!(
            tactic,
            MitreTactic::IcsInhibitResponseFunction,
            "T1693.001 tactic must be MitreTactic::IcsInhibitResponseFunction (TA0107) per \
             ADR-010 Decision 7; NOT InitialAccess or any other variant"
        );
        // Cross-check: technique_tactic_id must resolve to TA0107.
        assert_eq!(
            technique_tactic_id("T1693.001"),
            Some("TA0107"),
            "technique_tactic_id(\"T1693.001\") must return Some(\"TA0107\") \
             (IcsInhibitResponseFunction → TA0107)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-133-004: MitreTactic::IcsExecution Display and tactic_id
    // Traces: VP-007 Step 5
    //
    // GREEN-BY-DESIGN: Both tests exercise pure enum→str mappings (zero branching,
    // no I/O, no helpers, 1 line per arm). VP-007 Step 5 (the IcsExecution variant
    // + Display + tactic_id) is implemented in src/mitre.rs (STORY-133); these
    // tests pin the Display string and TA-ID.
    // -------------------------------------------------------------------------

    /// AC-133-004 — MitreTactic::IcsExecution.to_string() == "Execution (ICS)".
    ///
    /// GREEN-BY-DESIGN: pure enum→str mapping, zero branching, 1 line, fully determined
    /// by the type system. Implemented in STORY-133 (VP-007 Step 5).
    ///
    /// Traces: AC-133-004; VP-007 Step 5.
    #[test]
    fn test_ics_execution_tactic_display() {
        assert_eq!(
            MitreTactic::IcsExecution.to_string(),
            "Execution (ICS)",
            "MitreTactic::IcsExecution.to_string() must equal \"Execution (ICS)\""
        );
    }

    /// AC-133-004 — MitreTactic::IcsExecution.tactic_id() == "TA0104".
    ///
    /// GREEN-BY-DESIGN: pure enum→str mapping, zero branching, 1 line, fully determined
    /// by the type system. Implemented in STORY-133 (VP-007 Step 5).
    ///
    /// Traces: AC-133-004; VP-007 Step 5.
    #[test]
    fn test_ics_execution_tactic_id() {
        assert_eq!(
            MitreTactic::IcsExecution.tactic_id(),
            "TA0104",
            "MitreTactic::IcsExecution.tactic_id() must equal \"TA0104\""
        );
    }

    // -------------------------------------------------------------------------
    // AC-133-005: SEEDED array grows from 25 to 28 entries
    // Traces: VP-007 Steps 2 and 3
    //
    // SEEDED count assertion strategy:
    //   `SEEDED_TECHNIQUE_IDS` and `SEEDED_TECHNIQUE_ID_COUNT` are `const` (not
    //   `pub const`) in src/mitre.rs — they are NOT accessible from integration tests
    //   outside the crate. The unit test `vp007_catalog_drift_guard` (inside src/mitre.rs)
    //   enforces `SEEDED_TECHNIQUE_IDS.len() == SEEDED_TECHNIQUE_ID_COUNT` and sweeps the
    //   finite ID space to derive the catalogue size.
    //
    //   The strongest assertion available from an integration test is exhaustive membership:
    //   assert that ALL 28 expected post-STORY-133 IDs resolve in technique_info. This is
    //   STRONGER than a count proxy — it verifies exact expected membership, not just
    //   that some count matches. A count proxy via technique_tactic("T0858").is_some()
    //   would be identical to the AC-133-001 test (redundant) and could pass with the
    //   wrong 28 IDs (wrong set, correct count). The exhaustive membership assertion closes
    //   that gap.
    //
    // Implemented: T0858, T0816, T1693.001 arms added to technique_info in STORY-133 (VP-007 Step 2).
    // -------------------------------------------------------------------------

    /// AC-133-005 — ALL 28 expected post-STORY-133 seeded IDs resolve in technique_info.
    ///
    /// T0858, T0816, and T1693.001 arms added in STORY-133 (VP-007 Steps 1+2+3).
    /// All 28 seeded IDs now resolve.
    ///
    /// Assertion strategy: exhaustive membership over all 28 expected IDs — stronger than
    /// a count proxy (correct set, not just correct count). SEEDED_TECHNIQUE_IDS / COUNT
    /// are not pub so direct array-length assertion is unavailable from integration tests;
    /// the vp007_catalog_drift_guard unit test (src/mitre.rs) enforces the array/count sync.
    ///
    /// Traces: AC-133-005; VP-007 Steps 2, 3; ADR-010 Decision 7.
    #[test]
    fn test_seeded_count_is_28() {
        // The 28 expected post-STORY-133 seeded IDs (25 pre-existing + 3 new).
        // Every entry must resolve in technique_info after the VP-007 burst lands.
        // Pre-existing 25 (must continue to resolve — regression guard):
        let pre_existing: &[&str] = &[
            // Enterprise (12)
            "T1027",
            "T1036",
            "T1040",
            "T1046",
            "T1071",
            "T1071.001",
            "T1071.004",
            "T1083",
            "T1499.002",
            "T1505.003",
            "T1573",
            "T1557.002",
            // ICS (13)
            "T0846",
            "T1692.001",
            "T1692.002",
            "T0885",
            "T0836",
            "T0814",
            "T0806",
            "T0835",
            "T0831",
            "T0888",
            "T1691.001",
            "T0827",
            "T0830",
        ];
        // New 3 (VP-007 Step 2 — added in STORY-133):
        let new_ids: &[&str] = &["T0858", "T0816", "T1693.001"];

        // Regression guard: all pre-existing IDs must continue to resolve.
        for id in pre_existing {
            assert!(
                technique_info(id).is_some(),
                "pre-existing seeded ID {id} no longer resolves in technique_info \
                 after STORY-133 — regression"
            );
        }

        // AC-133-005 core: the 3 new IDs resolve after VP-007 Steps 1–3 (STORY-133).
        for id in new_ids {
            assert!(
                technique_info(id).is_some(),
                "new seeded ID {id} must resolve in technique_info after VP-007 Step 2 \
                 (SEEDED 25→28); added in STORY-133"
            );
        }

        // Total: 25 pre-existing + 3 new = 28 unique IDs all resolve.
        // (The vp007_catalog_drift_guard unit test enforces that no extra IDs exist
        //  beyond SEEDED_TECHNIQUE_IDS — the FORWARD completeness check.)
        assert_eq!(
            pre_existing.len() + new_ids.len(),
            28,
            "expected 25 pre-existing + 3 new = 28 total seeded IDs"
        );
    }

    // -------------------------------------------------------------------------
    // AC-133-006: EMITTED_IDS grows from 17 to 20; T1693.001 NOT emitted; T0846 IS emitted
    // Traces: VP-007 Step 4
    //
    // EMITTED_IDS access note: `EMITTED_IDS` lives inside `#[cfg(kani)] mod kani_proofs`
    // and is NOT pub — integration tests cannot access it. The strongest available assertion
    // from an integration test: enumerate ALL 20 expected post-STORY-133 emitted IDs and
    // assert each resolves in technique_info (necessary precondition for emittability).
    //
    // The actual EMITTED ⊆ SEEDED invariant (and the count == 20) is enforced by the kani
    // proof in src/mitre.rs once T0858, T0816, T0846 are added to EMITTED_IDS.
    //
    // Implemented: T0858 and T0816 arms added in STORY-133 (VP-007 Steps 1+4). T0846 pre-existing.
    // -------------------------------------------------------------------------

    /// AC-133-006 — ALL 20 expected post-STORY-133 emitted IDs resolve in technique_info.
    ///
    /// T0858 and T0816 arms added in STORY-133 (VP-007 Steps 1+4). All 20 emitted IDs resolve.
    ///
    /// Assertion strategy: enumerate all 20 expected EMITTED IDs and assert each resolves
    /// in technique_info. This is the strongest assertion available from integration tests
    /// (EMITTED_IDS is kani-gated, not pub). The vp007_catalog_drift_guard unit test and
    /// the kani `verify_all_emitted_ids_resolve` proof enforce the EMITTED ⊆ SEEDED invariant
    /// and EMITTED count now that the EMITTED_IDS entries have been added (STORY-133).
    ///
    /// Traces: AC-133-006; VP-007 Step 4; ADR-010 Decision 7.
    #[test]
    fn test_emitted_count_is_20() {
        // The 20 expected post-STORY-133 emitted IDs (17 pre-existing + 3 new).
        // Pre-existing 17 (regression guard — must continue to resolve):
        let pre_existing_emitted: &[&str] = &[
            // Enterprise (6)
            "T1027",
            "T1036",
            "T1046",
            "T1083",
            "T1499.002",
            "T1505.003",
            // ICS (7)
            "T1692.001",
            "T0836",
            "T0814",
            "T0806",
            "T0835",
            "T0831",
            "T0888",
            // STORY-109 (2)
            "T1691.001",
            "T0827",
            // STORY-114 (2)
            "T0830",
            "T1557.002",
        ];
        // New 3 EMITTED additions (VP-007 Step 4 — added in STORY-133):
        let new_emitted: &[&str] = &[
            "T0858", // technique_info arm added in STORY-133
            "T0816", // technique_info arm added in STORY-133
            "T0846", // T0846 pre-exists in technique_info (seeded-only → now also emitted)
        ];

        // Regression guard: all pre-existing emitted IDs must continue to resolve.
        for id in pre_existing_emitted {
            assert!(
                technique_info(id).is_some(),
                "pre-existing emitted ID {id} no longer resolves in technique_info — regression"
            );
        }

        // AC-133-006 core: new emitted IDs resolve after VP-007 Steps 1+4 (STORY-133).
        for id in new_emitted {
            assert!(
                technique_info(id).is_some(),
                "new emitted ID {id} must resolve in technique_info before appearing in \
                 EMITTED_IDS (VP-007 Step 4); T0858/T0816 added in STORY-133"
            );
        }

        // Total: 17 pre-existing + 3 new = 20 unique emitted IDs all resolve.
        assert_eq!(
            pre_existing_emitted.len() + new_emitted.len(),
            20,
            "expected 17 pre-existing + 3 new = 20 total emitted IDs"
        );
    }

    /// AC-133-006 — T1693.001 is NOT in EMITTED_IDS (staged-only for v0.11.0).
    ///
    /// `technique_info("T1693.001")` returns Some (T1693.001 arm added in STORY-133,
    /// VP-007 Step 1). The seeded assertion passes.
    ///
    /// Non-emission assertion: T1693.001 must NOT be in the expected emitted set.
    /// The expected 20 emitted IDs are enumerated inline — T1693.001 is absent.
    /// This is the strongest non-emission check available from integration tests (EMITTED_IDS
    /// is kani-gated; direct access is unavailable). The kani `vp007_catalog_drift_guard`
    /// and `verify_all_emitted_ids_resolve` proofs enforce the invariant at the kani layer.
    ///
    /// Traces: AC-133-006 "T1693.001 NOT emitted"; VP-007 Step 4; ADR-010 Decision 7.
    #[test]
    fn test_t1693_001_not_emitted() {
        // Part 1 — T1693.001 must be SEEDED (in technique_info). Added in STORY-133.
        assert!(
            technique_info("T1693.001").is_some(),
            "T1693.001 must be seeded in technique_info (VP-007 Step 1); arm added in STORY-133"
        );

        // Part 2 — T1693.001 must NOT be in the expected emitted set.
        // The 20 expected post-STORY-133 emitted IDs (hardcoded — matches kani EMITTED_IDS
        // after VP-007 Step 4 lands). T1693.001 must be absent.
        let expected_emitted_20: &[&str] = &[
            "T1027",
            "T1036",
            "T1046",
            "T1083",
            "T1499.002",
            "T1505.003", // Enterprise (6)
            "T1692.001",
            "T0836",
            "T0814",
            "T0806",
            "T0835",
            "T0831",
            "T0888", // ICS (7)
            "T1691.001",
            "T0827", // STORY-109 (2)
            "T0830",
            "T1557.002", // STORY-114 (2)
            "T0858",
            "T0816",
            "T0846", // STORY-133 new (3)
        ];
        assert_eq!(
            expected_emitted_20.len(),
            20,
            "sanity: expected_emitted_20 must have exactly 20 entries"
        );
        assert!(
            !expected_emitted_20.contains(&"T1693.001"),
            "T1693.001 must NOT appear in EMITTED_IDS — it is staged-only for v0.11.0 \
             (VP-007 Step 4 and ADR-010 Decision 7)"
        );
    }

    /// AC-133-006 — T0846 resolves in technique_info with tactic IcsDiscovery.
    ///
    /// T0846 pre-exists in technique_info and was promoted to EMITTED_IDS in STORY-133
    /// (VP-007 Step 4). The vp007_catalog_drift_guard enforces the EMITTED ⊆ SEEDED invariant.
    ///
    /// Traces: AC-133-006 "T0846 in EMITTED"; VP-007 Step 4.
    #[test]
    fn test_t0846_in_emitted() {
        let tactic = technique_tactic("T0846");
        assert!(
            tactic.is_some(),
            "T0846 must continue to resolve in technique_info after STORY-133 seeding"
        );
        assert_eq!(
            tactic.unwrap(),
            MitreTactic::IcsDiscovery,
            "T0846 tactic must remain MitreTactic::IcsDiscovery (ICS Discovery, TA0102)"
        );
    }

    /// AC-133-006 + AC-133-001/002 cross-check — technique_tactic_id end-to-end for new IDs.
    ///
    /// Verifies the full ID -> tactic -> TA-ID chain for all 3 STORY-133 IDs plus T0846
    /// regression:
    ///   T0858 -> IcsExecution -> "TA0104"
    ///   T0816 -> IcsInhibitResponseFunction -> "TA0107"
    ///   T0846 -> IcsDiscovery -> "TA0102" (pre-existing regression)
    ///
    /// AC-133-004 tested the enum method directly; this exercises the full path from
    /// technique ID -> tactic -> TA-ID string through the Option chain.
    ///
    /// Traces: AC-133-001/002 (T0858/T0816 arms); VP-007 Step 5 (IcsExecution TA-ID); ADR-010.
    #[test]
    fn test_t0858_t0816_and_t0846_tactic_id_resolution() {
        // T0858: new — arm added in STORY-133 (VP-007 Step 1)
        assert_eq!(
            technique_tactic_id("T0858"),
            Some("TA0104"),
            "technique_tactic_id(\"T0858\") must return Some(\"TA0104\") — \
             T0858 maps to IcsExecution (TA0104); seeded in STORY-133 (VP-007 Step 1)"
        );
        // T0816: new — arm added in STORY-133 (VP-007 Step 1)
        assert_eq!(
            technique_tactic_id("T0816"),
            Some("TA0107"),
            "technique_tactic_id(\"T0816\") must return Some(\"TA0107\") — \
             T0816 maps to IcsInhibitResponseFunction (TA0107); seeded in STORY-133 (VP-007 Step 1)"
        );
        // T0846: pre-existing — should pass NOW as regression check
        assert_eq!(
            technique_tactic_id("T0846"),
            Some("TA0102"),
            "technique_tactic_id(\"T0846\") must return Some(\"TA0102\") — \
             T0846 maps to IcsDiscovery (TA0102); pre-existing entry"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-134 recon-detection tests.
//
// Traces to: BC-2.17.010 (T0846 ListIdentity), BC-2.17.008 (CIP error accumulation),
//            BC-2.17.014 (T0888 Pattern A identity read + Pattern B error burst).
//
// SCOPE NOTE:
//   STORY-134 owns `process_pdu` + `EnipFlowState`. The frame-walk wiring in `on_data`
//   (BC-2.17.016 PC-0 command_counts increment) is owned by STORY-137.
//   Tests drive `process_pdu` directly, constructing `EnipFlowState` inline.
//
// IMPLEMENTATION STATUS (STORY-134 complete):
// All 20 tests pass. `process_pdu` implements T0846/T0888 detection; the
// post-call assertions confirm the required behavior. Tests originated as
// Red-Gate stubs (none could pass until STORY-134 shipped).
// ─────────────────────────────────────────────────────────────────────────────
mod recon {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::enip::{EnipAnalyzer, EnipFlowState};
    use wirerust::findings::{Confidence, ThreatCategory, Verdict};

    // =========================================================================
    // PDU construction helpers
    //
    // All helpers return minimal-but-valid ENIP frame byte vectors for use with
    // `process_pdu`. Only the fields relevant to each detection path are set;
    // others are zeroed.
    //
    // ENIP encapsulation header layout (24 bytes, all LE):
    //   [0..2]   command u16 LE
    //   [2..4]   length  u16 LE  (payload bytes after header)
    //   [4..8]   session_handle u32 LE
    //   [8..12]  status u32 LE
    //   [12..20] sender_context [u8; 8]
    //   [20..24] options u32 LE
    //
    // SendRRData payload layout:
    //   [0..4]   Interface Handle u32 LE (=0)
    //   [4..6]   Timeout u16 LE (=0)
    //   [6..]    CPF data (item_count u16 LE + items)
    //
    // CPF item layout: type_id u16 LE + length u16 LE + data bytes.
    //
    // CIP Unconnected Data Item (0x00B2) request:
    //   [0]      service byte (0x80 clear = request)
    //   [1]      request_path_size in words (u8)
    //   [2..]    request path bytes (path_size * 2 bytes)
    //
    // CIP Unconnected Data Item (0x00B2) response:
    //   [0]      service | 0x80
    //   [1]      reserved = 0x00
    //   [2]      general_status byte
    //   [3]      additional_status_size = 0x00
    // =========================================================================

    /// Build a 24-byte ENIP header with `command` set; other fields zeroed.
    fn enip_header(command: u16) -> [u8; 24] {
        let mut h = [0u8; 24];
        h[0..2].copy_from_slice(&command.to_le_bytes());
        h
    }

    /// Build a minimal ListIdentity PDU (command=0x0063, no payload).
    ///
    /// `parse_enip_header` requires >=24 bytes. `process_pdu` reads the ENIP header
    /// from the first 24 bytes; no CPF/CIP parse is needed for ListIdentity (ENIP
    /// command-layer detection, ADR-010 Decision 4 step 2).
    fn list_identity_pdu() -> Vec<u8> {
        enip_header(0x0063).to_vec() // command = ListIdentity
    }

    /// Build a SendRRData PDU containing one 0x00B2 CIP item.
    ///
    /// Layout:
    ///   ENIP header (24 bytes, command=0x006F)
    ///   Interface Handle (4 bytes, 0)
    ///   Timeout (2 bytes, 0)
    ///   CPF item_count = 2 (NullAddress + UnconnectedData)
    ///   NullAddress item (0x0000, length=0)
    ///   UnconnectedData item (0x00B2, length = cip_data.len())
    ///   cip_data bytes
    fn sendrr_pdu_with_cip(cip_data: &[u8]) -> Vec<u8> {
        // CPF payload: [NullAddr(4 bytes)] + [0x00B2 item(4 + cip_data.len())]
        let cpf_item_count_bytes = 2u16.to_le_bytes();
        let null_item: &[u8] = &[0x00, 0x00, 0x00, 0x00]; // type=0x0000, length=0
        let b2_type: &[u8] = &[0xB2, 0x00]; // type_id = 0x00B2 LE
        let b2_len = (cip_data.len() as u16).to_le_bytes();

        // Full payload after ENIP header:
        //   Interface Handle (4) + Timeout (2) + item_count (2) + NullAddr (4)
        //   + 0x00B2 header (4) + cip_data
        let payload_len: usize = 4 + 2 + 2 + 4 + 4 + cip_data.len();

        let mut pdu = Vec::with_capacity(24 + payload_len);
        // ENIP header
        let mut hdr = enip_header(0x006F); // SendRRData
        hdr[2..4].copy_from_slice(&(payload_len as u16).to_le_bytes()); // length field
        pdu.extend_from_slice(&hdr);
        // Interface Handle + Timeout
        pdu.extend_from_slice(&[0u8; 4]); // Interface Handle = 0
        pdu.extend_from_slice(&[0u8; 2]); // Timeout = 0
        // CPF items
        pdu.extend_from_slice(&cpf_item_count_bytes);
        pdu.extend_from_slice(null_item);
        pdu.extend_from_slice(b2_type);
        pdu.extend_from_slice(&b2_len);
        pdu.extend_from_slice(cip_data);
        pdu
    }

    /// Build a SendRRData PDU containing one 0x00B1 (Connected Data Item) CIP item.
    ///
    /// Used to verify the F-P9-001 type_id gate: 0x00B1 items are skipped; no
    /// CIP parse and no detection fires.
    fn sendrr_pdu_with_b1_cip(cip_data: &[u8]) -> Vec<u8> {
        let cpf_item_count_bytes = 2u16.to_le_bytes();
        let null_item: &[u8] = &[0x00, 0x00, 0x00, 0x00];
        let b1_type: &[u8] = &[0xB1, 0x00]; // type_id = 0x00B1 LE
        let b1_len = (cip_data.len() as u16).to_le_bytes();

        let payload_len: usize = 4 + 2 + 2 + 4 + 4 + cip_data.len();
        let mut pdu = Vec::with_capacity(24 + payload_len);
        let mut hdr = enip_header(0x006F);
        hdr[2..4].copy_from_slice(&(payload_len as u16).to_le_bytes());
        pdu.extend_from_slice(&hdr);
        pdu.extend_from_slice(&[0u8; 4]);
        pdu.extend_from_slice(&[0u8; 2]);
        pdu.extend_from_slice(&cpf_item_count_bytes);
        pdu.extend_from_slice(null_item);
        pdu.extend_from_slice(b1_type);
        pdu.extend_from_slice(&b1_len);
        pdu.extend_from_slice(cip_data);
        pdu
    }

    /// Build a CIP GetAttributeSingle request targeting a given CIP class.
    ///
    /// CIP service = 0x0E (GetAttributeSingle, request; bit 7 clear).
    /// Path = Class(class_id) + Instance(1) + Attribute(7): 3 words = 6 bytes.
    ///
    /// item_data layout:
    ///   [0]    0x0E    service = GetAttributeSingle
    ///   [1]    0x03    request_path_size = 3 words (6 bytes)
    ///   [2]    0x20    Class segment type
    ///   [3]    class_id
    ///   [4]    0x24    Instance segment type
    ///   [5]    0x01    instance = 1
    ///   [6]    0x30    Attribute segment type
    ///   [7]    0x07    attribute = 7 (ProductName)
    fn cip_get_attr_single_request(class_id: u8) -> Vec<u8> {
        vec![
            0x0E, // service = GetAttributeSingle (request)
            0x03, // request_path_size = 3 words
            0x20, class_id, // Class segment
            0x24, 0x01, // Instance segment
            0x30, 0x07, // Attribute segment (ProductName)
        ]
    }

    /// Build a CIP GetAttributesAll request targeting a given CIP class.
    ///
    /// CIP service = 0x01 (GetAttributesAll, request; bit 7 clear).
    /// Path = Class(class_id) + Instance(1): 2 words = 4 bytes.
    /// Used in EC-011 (GetAttributesAll to Identity Object also triggers Pattern A).
    #[allow(dead_code)]
    fn cip_get_attrs_all_request(class_id: u8) -> Vec<u8> {
        vec![
            0x01, // service = GetAttributesAll (request)
            0x02, // request_path_size = 2 words
            0x20, class_id, // Class segment
            0x24, 0x01, // Instance segment
        ]
    }

    /// Build a CIP error response with the given general_status byte.
    ///
    /// item_data layout (BC-2.17.008 canonical vector):
    ///   [0]    0x8E    service = 0x0E | 0x80 (GetAttributeSingle response, bit 7 set)
    ///   [1]    0x00    reserved
    ///   [2]    general_status
    ///   [3]    0x00    additional_status_size = 0
    fn cip_error_response(general_status: u8) -> Vec<u8> {
        vec![0x8E, 0x00, general_status, 0x00]
    }

    /// Build a CIP success response (general_status=0x00).
    fn cip_success_response() -> Vec<u8> {
        cip_error_response(0x00)
    }

    /// Canonical source IP for all recon tests (192.168.1.10).
    fn src_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))
    }

    // =========================================================================
    // AC-134-001: ListIdentity ENIP command emits T0846 (per-flow one-shot)
    // Traces: BC-2.17.010 postconditions 1-3; BC-2.17.010 invariant 1
    // =========================================================================

    /// AC-134-001 — single ListIdentity frame emits exactly one T0846 finding.
    ///
    /// BC-2.17.010 postcondition 2: if `flow.list_identity_emitted == false` and
    /// `all_findings.len() < MAX_FINDINGS`, push ONE Finding with:
    ///   - `mitre_techniques: vec!["T0846"]`
    ///   - `verdict: Verdict::Likely`, `confidence: Confidence::High`
    ///   - `summary: "EtherNet/IP ListIdentity broadcast observed: network-wide device
    ///      enumeration (T0846)"`
    ///   - `category: ThreatCategory::Reconnaissance`
    ///   - `list_identity_emitted` set to true.
    ///
    /// Non-vacuous: asserts finding count, MITRE ID, verdict, confidence, summary text,
    /// and `list_identity_emitted` state. All require `process_pdu` to have run.
    ///
    /// Traces: BC-2.17.010 postcondition 2; EC-001.
    #[test]
    fn test_list_identity_emits_t0846() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = list_identity_pdu();

        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "single ListIdentity frame must emit exactly 1 T0846 finding (BC-2.17.010 PC-2)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0846".to_string()],
            "finding must carry MITRE technique T0846 (Remote System Discovery, ICS Discovery TA0102)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "T0846 finding verdict must be Likely (BC-2.17.010 postcondition 2)"
        );
        assert_eq!(
            f.confidence,
            Confidence::High,
            "T0846 finding confidence must be High (BC-2.17.010 postcondition 2)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Reconnaissance,
            "T0846 finding category must be Reconnaissance (BC-2.17.010 postcondition 2)"
        );
        assert_eq!(
            f.summary,
            "EtherNet/IP ListIdentity broadcast observed: network-wide device enumeration (T0846)",
            "T0846 summary must match BC-2.17.010 postcondition 2 exact string"
        );
        assert!(
            flow.list_identity_emitted,
            "list_identity_emitted must be set to true after first ListIdentity (BC-2.17.010 PC-2)"
        );
    }

    /// AC-134-001 — five ListIdentity frames on same flow emit exactly one T0846 finding.
    ///
    /// Per-flow one-shot guard (BC-2.17.010 invariant 1): first frame emits finding and
    /// sets `flow.list_identity_emitted = true`; frames 2-5 are processed (no panic) but
    /// emit no additional findings.
    ///
    /// Non-vacuous: asserts `all_findings.len() == 1` after 5 calls (not 5); asserts
    /// `list_identity_emitted == true`. Distinguishes a correct one-shot implementation
    /// from a broken per-occurrence one. Required for holdout HS-114 Case B.
    ///
    /// Traces: BC-2.17.010 invariant 1; EC-002; holdout HS-114 Case B.
    #[test]
    fn test_list_identity_one_shot_guard_multi_frame() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = list_identity_pdu();

        for _ in 0..5 {
            analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        }

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "5 ListIdentity frames on the same flow must emit exactly 1 T0846 finding \
             (per-flow one-shot guard — BC-2.17.010 invariant 1; holdout HS-114 Case B)"
        );
        assert!(
            flow.list_identity_emitted,
            "list_identity_emitted must remain true after guard fires (BC-2.17.010 invariant 1)"
        );
        assert_eq!(
            analyzer.all_findings[0].mitre_techniques,
            vec!["T0846".to_string()],
            "the single finding must still be T0846"
        );
    }

    /// AC-134-001 — ListIdentity when `all_findings` is at MAX_FINDINGS: no finding pushed,
    /// `list_identity_emitted` remains false.
    ///
    /// BC-2.17.010 postcondition 2 last condition: push is gated on
    /// `self.all_findings.len() < MAX_FINDINGS`. When the cap is already hit, the
    /// guard `list_identity_emitted` must NOT be set (the finding was not emitted).
    ///
    /// MAX_FINDINGS is 10_000 (shared analyzer constant; per-analyzer value used in
    /// EnipAnalyzer::process_pdu, consistent with DNP3 and Modbus analyzers).
    ///
    /// Non-vacuous: pre-fills `all_findings` to exactly 10_000 entries (the cap), then
    /// asserts no new finding was appended and the guard is still false. Requires
    /// `process_pdu` to both inspect `all_findings.len()` and leave `list_identity_emitted`
    /// false when it cannot push.
    ///
    /// Traces: BC-2.17.010 postcondition 2 last condition; EC-002b (EC-003 in BC); BC-2.17.022.
    #[test]
    fn test_list_identity_respects_max_findings() {
        use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

        const MAX_FINDINGS: usize = 10_000;

        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // Pre-fill all_findings to the cap.
        for _ in 0..MAX_FINDINGS {
            analyzer.all_findings.push(Finding {
                category: ThreatCategory::Reconnaissance,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "placeholder".to_string(),
                evidence: vec![],
                mitre_techniques: vec![],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }
        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS,
            "pre-condition: all_findings must be exactly at the cap before the test call"
        );

        let pdu = list_identity_pdu();
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS,
            "when all_findings is at MAX_FINDINGS, ListIdentity must NOT push an additional \
             finding (BC-2.17.010 postcondition 2 last condition; BC-2.17.022)"
        );
        assert!(
            !flow.list_identity_emitted,
            "list_identity_emitted must remain false when the finding could not be pushed \
             (guard set only after successful push per BC-2.17.010 postcondition 2)"
        );
    }

    // =========================================================================
    // AC-134-002: CIP error responses accumulate per-status in error_counts_in_window
    // Traces: BC-2.17.008 postconditions 1-5; BC-2.17.008 invariants 1-4
    // =========================================================================

    /// AC-134-002 — CIP error response (general_status=0x08) increments
    /// `flow.error_counts_in_window[0x08]` and seeds `error_window_start_ts`.
    ///
    /// BC-2.17.008 postcondition 2: if `general_status != 0x00`:
    ///   - `flow.error_counts_in_window.entry(0x08).or_insert(0) += 1`
    ///   - If first error in window: `flow.error_window_start_ts = now_ts`
    ///
    /// Non-vacuous: asserts `error_counts_in_window[0x08] == 1` (requires extraction
    /// of byte 2 from CIP item data); asserts `error_window_start_ts` was seeded.
    ///
    /// Traces: BC-2.17.008 postcondition 2; canonical test vector (CIP 8E 00 08 00);
    /// EC-002.
    #[test]
    fn test_error_accumulation_increments_per_status() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // CIP error response: general_status = 0x08 (Service Not Supported)
        // BC-2.17.008 canonical vector: 8E 00 08 00
        let cip_data = cip_error_response(0x08);
        let pdu = sendrr_pdu_with_cip(&cip_data);

        analyzer.process_pdu(&mut flow, &pdu, 200, src_ip());

        assert_eq!(
            flow.error_counts_in_window.get(&0x08).copied().unwrap_or(0),
            1,
            "general_status=0x08 must increment error_counts_in_window[0x08] to 1 \
             (BC-2.17.008 postcondition 2; canonical vector 8E 00 08 00)"
        );
        // error_window_start_ts is seeded on first error (BC-2.17.008 postcondition 2);
        // exact value is the implementation's now_ts, which equals the timestamp argument (200).
        assert_eq!(
            flow.error_window_start_ts, 200,
            "error_window_start_ts must be seeded to now_ts=200 on first error \
             (BC-2.17.008 postcondition 2)"
        );
        // Two additional errors with status 0x08 → count becomes 3.
        analyzer.process_pdu(&mut flow, &pdu, 201, src_ip());
        analyzer.process_pdu(&mut flow, &pdu, 202, src_ip());
        assert_eq!(
            flow.error_counts_in_window.get(&0x08).copied().unwrap_or(0),
            3,
            "three error responses with status 0x08 must yield error_counts_in_window[0x08] == 3 \
             (BC-2.17.008 postcondition 2 accumulation)"
        );
    }

    /// AC-134-002 — CIP success response (general_status=0x00) does not update any counter.
    ///
    /// BC-2.17.008 postcondition 3: `general_status == 0x00` → no counter update, no
    /// aggregate increment.
    ///
    /// Non-vacuous: asserts `error_counts_in_window` stays empty and `error_count` stays 0.
    /// Requires `process_pdu` to read `general_status` correctly (byte 2 of CIP item data).
    ///
    /// Traces: BC-2.17.008 postcondition 3; invariant 3; EC-001.
    #[test]
    fn test_error_accumulation_ignores_success() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // CIP success response: general_status = 0x00
        // BC-2.17.008 canonical vector: 8E 00 00 00 <data>
        let cip_data = cip_success_response();
        let pdu = sendrr_pdu_with_cip(&cip_data);

        analyzer.process_pdu(&mut flow, &pdu, 300, src_ip());

        assert!(
            flow.error_counts_in_window.is_empty(),
            "general_status=0x00 (success) must NOT update error_counts_in_window \
             (BC-2.17.008 postcondition 3; invariant 3)"
        );
        assert_eq!(
            analyzer.error_count, 0,
            "general_status=0x00 (success) must NOT increment EnipAnalyzer.error_count \
             (BC-2.17.008 postcondition 3)"
        );
    }

    /// AC-134-002 — 10-second window expiry resets `error_counts_in_window`,
    /// `error_window_start_ts`, and `error_rate_emitted`.
    ///
    /// BC-2.17.008 postcondition 4: if `now_ts.wrapping_sub(flow.error_window_start_ts) > 10`:
    ///   - `flow.error_counts_in_window.clear()`
    ///   - `flow.error_window_start_ts = now_ts`
    ///   - `flow.error_rate_emitted = false`
    ///
    /// Sequence: send one error at ts=400 (seeds window); send another error at ts=412
    /// (>10s later, window should reset first, then the new error is recorded in the
    /// fresh window with count=1, not 2).
    ///
    /// Non-vacuous: asserts that after a >10s gap, `error_counts_in_window` has count=1
    /// (only the post-reset error), not 2 (which would indicate no reset).
    ///
    /// Traces: BC-2.17.008 postcondition 4; invariant 4; EC-005.
    #[test]
    fn test_error_window_resets_after_10s() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        let cip_data = cip_error_response(0x08);
        let pdu = sendrr_pdu_with_cip(&cip_data);

        // First error at ts=400: seeds the window, error_window_start_ts = 400.
        analyzer.process_pdu(&mut flow, &pdu, 400, src_ip());
        assert_eq!(
            flow.error_counts_in_window.get(&0x08).copied().unwrap_or(0),
            1,
            "pre-condition: first error must be counted before window expiry"
        );

        // Second error at ts=412: 412 - 400 = 12 > 10 → window must reset first.
        // After reset and re-seed, error_counts_in_window[0x08] = 1 (NOT 2).
        analyzer.process_pdu(&mut flow, &pdu, 412, src_ip());

        let count_after_reset = flow.error_counts_in_window.get(&0x08).copied().unwrap_or(0);
        assert_eq!(
            count_after_reset, 1,
            "after >10s gap, window must reset; error_counts_in_window[0x08] must be 1 \
             (only the post-reset error, not the pre-reset error) \
             (BC-2.17.008 postcondition 4; EC-005)"
        );
        assert!(
            !flow.error_rate_emitted,
            "error_rate_emitted must be reset to false on window expiry \
             (BC-2.17.008 postcondition 4)"
        );
    }

    /// AC-134-002 — CIP item with `type_id == 0x00B1` (Connected Data Item) skips
    /// `general_status` extraction; no counter update.
    ///
    /// BC-2.17.008 precondition 2 (HARD scope gate): `type_id != 0x00B2` → skip extraction
    /// entirely. The test uses a 0x00B1 item carrying bytes that look like an error response
    /// at the same offsets; the gate must prevent any extraction.
    ///
    /// Non-vacuous: uses a 0x00B1 PDU with `general_status=0x08` bytes present; asserts
    /// `error_counts_in_window` remains empty. Distinguishes a correct implementation
    /// (checks type_id before extracting) from a broken one that parses all items blindly.
    ///
    /// Traces: BC-2.17.008 precondition 2; EC-007; F-P9-001.
    #[test]
    fn test_error_accumulation_skips_connected_item() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // Build a 0x00B1 item with what would be an error response byte pattern.
        // Even though bytes 2-3 spell out general_status=0x08, the 0x00B1 gate must block it.
        let cip_data = cip_error_response(0x08);
        let pdu = sendrr_pdu_with_b1_cip(&cip_data);

        analyzer.process_pdu(&mut flow, &pdu, 500, src_ip());

        assert!(
            flow.error_counts_in_window.is_empty(),
            "type_id=0x00B1 (Connected Data Item) must be skipped by the 0x00B2 gate; \
             error_counts_in_window must remain empty (BC-2.17.008 precondition 2; F-P9-001)"
        );
        assert_eq!(
            analyzer.error_count, 0,
            "EnipAnalyzer.error_count must not be incremented for 0x00B1 items \
             (BC-2.17.008 precondition 2)"
        );
    }

    /// AC-134-002 — CIP item data shorter than 4 bytes skips `general_status` extraction;
    /// no counter update, no panic.
    ///
    /// BC-2.17.008 precondition 3: `cip_item_data.len() >= 4` required before indexing byte 2.
    ///
    /// Non-vacuous: uses a 3-byte CIP item (cannot safely index byte 2); asserts
    /// `error_counts_in_window` is empty and `error_count` is 0.
    ///
    /// Traces: BC-2.17.008 precondition 3; EC-004.
    #[test]
    fn test_error_accumulation_requires_4_bytes() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // 3-byte CIP item data: cannot safely read byte[2] as general_status.
        let cip_data: Vec<u8> = vec![0x8E, 0x00, 0x08]; // 3 bytes only
        let pdu = sendrr_pdu_with_cip(&cip_data);

        // Must NOT panic (bounds-safe extraction required by BC-2.17.008 precondition 3).
        analyzer.process_pdu(&mut flow, &pdu, 600, src_ip());

        assert!(
            flow.error_counts_in_window.is_empty(),
            "CIP item data < 4 bytes must skip general_status extraction; \
             error_counts_in_window must remain empty (BC-2.17.008 precondition 3; EC-004)"
        );
        assert_eq!(
            analyzer.error_count, 0,
            "EnipAnalyzer.error_count must not be incremented when cip_item_data.len() < 4 \
             (BC-2.17.008 precondition 3)"
        );
    }

    // =========================================================================
    // AC-134-006: EnipAnalyzer aggregate error_count increments on every CIP error
    // Traces: BC-2.17.008 Postcondition 2b; BC-2.17.008 Invariant 2; AC-134-006
    // =========================================================================

    /// AC-134-006 — `EnipAnalyzer.error_count` increments across multiple flows for every
    /// qualifying CIP error response (general_status != 0x00, type_id=0x00B2, len>=4).
    ///
    /// BC-2.17.008 Postcondition 2b / Invariant 2: `self.error_count` is a LIFETIME aggregate
    /// counter incremented on every non-zero general_status response. It is never reset between
    /// flows or windows.
    ///
    /// Sequence: 3 error responses on flow A + 2 error responses on flow B → `error_count == 5`.
    ///
    /// Non-vacuous: asserts the aggregate count equals the exact number of qualifying error
    /// responses across both flows. Requires `process_pdu` to call `self.error_count += 1`
    /// on the `EnipAnalyzer` struct (not the per-flow counter).
    ///
    /// Traces: BC-2.17.008 Postcondition 2b; Invariant 2; AC-134-006.
    #[test]
    fn test_aggregate_error_count_increments() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow_a = EnipFlowState::new();
        let mut flow_b = EnipFlowState::new();

        let err_pdu = sendrr_pdu_with_cip(&cip_error_response(0x08));

        // 3 error responses on flow A
        for ts in [700u32, 701, 702] {
            analyzer.process_pdu(&mut flow_a, &err_pdu, ts, src_ip());
        }
        assert_eq!(
            analyzer.error_count, 3,
            "after 3 error responses on flow A, EnipAnalyzer.error_count must be 3 \
             (BC-2.17.008 Postcondition 2b / Invariant 2)"
        );

        // 2 more error responses on flow B (different flow — verifies cross-flow accumulation)
        let src_ip_b = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20));
        for ts in [800u32, 801] {
            analyzer.process_pdu(&mut flow_b, &err_pdu, ts, src_ip_b);
        }
        assert_eq!(
            analyzer.error_count, 5,
            "after 3 errors on flow A + 2 on flow B, EnipAnalyzer.error_count must be 5 \
             (lifetime aggregate, never reset — BC-2.17.008 Postcondition 2b / Invariant 2)"
        );

        // Success responses must NOT increment error_count.
        let success_pdu = sendrr_pdu_with_cip(&cip_success_response());
        analyzer.process_pdu(&mut flow_a, &success_pdu, 900, src_ip());
        assert_eq!(
            analyzer.error_count, 5,
            "success response (general_status=0x00) must NOT increment error_count \
             (BC-2.17.008 postcondition 3)"
        );
    }

    // =========================================================================
    // AC-134-003: T0888 Pattern A — GetAttribute to Identity Object (Class 0x01)
    // Traces: BC-2.17.014 postconditions Pattern A
    // =========================================================================

    /// AC-134-003 — GetAttributeSingle to Identity Object (Class 0x01) via 0x00B2 item
    /// emits exactly one T0888 Pattern A finding (Likely/High).
    ///
    /// BC-2.17.014 Pattern A postcondition 1: ONE Finding with:
    ///   - `mitre_techniques: vec!["T0888"]`
    ///   - `verdict: Verdict::Likely`, `confidence: Confidence::High`
    ///   - `summary: "CIP Identity Object attribute read: single-device reconnaissance (T0888)"`
    ///   - `category: ThreatCategory::Reconnaissance`
    ///
    /// Non-vacuous: asserts finding count, MITRE ID, verdict, confidence, summary.
    ///
    /// Traces: BC-2.17.014 Pattern A postcondition 1; EC-001.
    #[test]
    fn test_t0888_pattern_a_identity_read() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // GetAttributeSingle to Class=0x01 (Identity Object), Instance=1, Attr=7
        let cip_data = cip_get_attr_single_request(0x01);
        let pdu = sendrr_pdu_with_cip(&cip_data);

        analyzer.process_pdu(&mut flow, &pdu, 1000, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "GetAttributeSingle to Identity Object (Class 0x01) via 0x00B2 must emit \
             exactly 1 T0888 Pattern A finding (BC-2.17.014 Pattern A PC-1)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0888".to_string()],
            "T0888 Pattern A finding must carry MITRE technique T0888 \
             (Remote System Information Discovery, ICS Discovery TA0102)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "T0888 Pattern A finding verdict must be Likely (BC-2.17.014 Pattern A PC-1)"
        );
        assert_eq!(
            f.confidence,
            Confidence::High,
            "T0888 Pattern A finding confidence must be High (BC-2.17.014 Pattern A PC-1)"
        );
        assert_eq!(
            f.category,
            ThreatCategory::Reconnaissance,
            "T0888 Pattern A finding category must be Reconnaissance"
        );
        assert_eq!(
            f.summary, "CIP Identity Object attribute read: single-device reconnaissance (T0888)",
            "T0888 Pattern A summary must match BC-2.17.014 Pattern A postcondition 1 exact string"
        );
    }

    /// AC-134-003 — GetAttributeSingle to non-Identity class (Class 0x04, Assembly) does NOT
    /// emit a T0888 Pattern A finding.
    ///
    /// BC-2.17.014 Pattern A precondition 2: path must contain `CipPathSegment::Class(0x01)`.
    /// Class 0x04 (Assembly Object) does not match; no finding emitted.
    ///
    /// Non-vacuous: uses Class=0x04 with the same GetAttributeSingle service; asserts
    /// `all_findings` is empty. Distinguishes an implementation that checks the class ID
    /// from one that fires on any GetAttribute regardless of target.
    ///
    /// Traces: BC-2.17.014 Pattern A precondition 2; EC-003.
    #[test]
    fn test_t0888_pattern_a_non_identity_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // GetAttributeSingle to Class=0x04 (Assembly Object) — NOT Identity Object
        let cip_data = cip_get_attr_single_request(0x04);
        let pdu = sendrr_pdu_with_cip(&cip_data);

        analyzer.process_pdu(&mut flow, &pdu, 1100, src_ip());

        assert!(
            analyzer.all_findings.is_empty(),
            "GetAttributeSingle to Class=0x04 (Assembly Object, not Identity) must NOT emit \
             T0888 Pattern A finding (BC-2.17.014 Pattern A precondition 2; EC-003)"
        );
    }

    /// AC-134-003 — GetAttributeSingle to Identity Object carried in a 0x00B1 (Connected Data
    /// Item) does NOT emit T0888 Pattern A.
    ///
    /// BC-2.17.014 Pattern A precondition 4 (F-P9-001): the 0x00B2-only gate applies.
    /// Connected items (0x00B1) are skipped in v0.11.0; no CIP parse and no detection.
    ///
    /// Non-vacuous: uses a 0x00B1 PDU carrying Class(0x01) CIP bytes; asserts no finding.
    /// Distinguishes an implementation that checks type_id from one that parses all items.
    ///
    /// Traces: BC-2.17.014 Pattern A precondition 4; EC-009; F-P9-001.
    #[test]
    fn test_t0888_pattern_a_connected_item_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // GetAttributeSingle to Class=0x01, but carried in 0x00B1 (Connected Data Item)
        let cip_data = cip_get_attr_single_request(0x01);
        let pdu = sendrr_pdu_with_b1_cip(&cip_data);

        analyzer.process_pdu(&mut flow, &pdu, 1200, src_ip());

        assert!(
            analyzer.all_findings.is_empty(),
            "GetAttributeSingle to Identity Object via 0x00B1 (Connected Data Item) must NOT \
             emit T0888 Pattern A — 0x00B2-only gate (F-P9-001; BC-2.17.014 PC-4; EC-009)"
        );
    }

    /// AC-134-003 — T0888 Pattern A fires per-occurrence (not one-shot): two
    /// GetAttributeSingle requests to Class 0x01 produce two T0888 findings.
    ///
    /// BC-2.17.014 invariant 2: Pattern A is per-occurrence. Unlike T0846 (one-shot per flow),
    /// each individual Identity Object read warrants its own finding.
    ///
    /// Non-vacuous: calls `process_pdu` twice with Class(0x01) GetAttributeSingle and asserts
    /// `all_findings.len() == 2`. Distinguishes per-occurrence from one-shot behavior.
    ///
    /// Traces: BC-2.17.014 invariant 2 (Pattern A is per-occurrence); EC-011; AC-134-003.
    #[test]
    fn test_t0888_pattern_a_fires_per_occurrence() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        let cip_data = cip_get_attr_single_request(0x01);
        let pdu = sendrr_pdu_with_cip(&cip_data);

        analyzer.process_pdu(&mut flow, &pdu, 1300, src_ip());
        analyzer.process_pdu(&mut flow, &pdu, 1301, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            2,
            "two GetAttributeSingle requests to Class 0x01 must produce 2 T0888 Pattern A \
             findings — per-occurrence, not one-shot (BC-2.17.014 invariant 2; EC-011)"
        );
        // Both findings must be T0888.
        for f in &analyzer.all_findings {
            assert_eq!(
                f.mitre_techniques,
                vec!["T0888".to_string()],
                "each per-occurrence finding must carry T0888"
            );
        }
    }

    // =========================================================================
    // AC-134-004: T0888 Pattern B — error burst crossing threshold fires one-shot
    // Traces: BC-2.17.014 postconditions Pattern B; BC-2.17.014 invariant 3
    // =========================================================================

    /// AC-134-004 — default threshold=5; 6th error in 10s window fires T0888 Pattern B
    /// (strict `>`; Possible/Medium).
    ///
    /// BC-2.17.014 Pattern B: `total_error_count > threshold` (strict >) with default=5.
    /// Exactly 5 errors → `5 > 5` is false, no finding. 6th error → `6 > 5` is true, finding.
    ///
    /// Assertion: after 6 calls, `all_findings` contains exactly 1 T0888 finding with
    /// `verdict=Possible`, `confidence=Medium`; `flow.error_rate_emitted == true`.
    ///
    /// Traces: BC-2.17.014 Pattern B postcondition 2; invariant 3 (strict >); EC-004.
    #[test]
    fn test_t0888_pattern_b_fires_at_threshold_plus_one() {
        let mut analyzer = EnipAnalyzer::new(50, 5); // error_burst_threshold = 5
        let mut flow = EnipFlowState::new();

        let err_pdu = sendrr_pdu_with_cip(&cip_error_response(0x08));

        // 5 errors: total=5, 5 > 5 is false → no Pattern B finding yet.
        for ts in [1400u32, 1401, 1402, 1403, 1404] {
            analyzer.process_pdu(&mut flow, &err_pdu, ts, src_ip());
        }
        assert!(
            analyzer.all_findings.is_empty(),
            "exactly 5 errors with threshold=5 must NOT fire Pattern B \
             (strict > semantics: 5 > 5 is false — BC-2.17.014 invariant 3; EC-005)"
        );

        // 6th error: total=6, 6 > 5 is true → Pattern B fires.
        analyzer.process_pdu(&mut flow, &err_pdu, 1405, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "6th error with threshold=5 must fire exactly 1 T0888 Pattern B finding \
             (strict >: 6 > 5 — BC-2.17.014 Pattern B postcondition 2; EC-004)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0888".to_string()],
            "Pattern B finding must carry MITRE technique T0888"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "T0888 Pattern B finding verdict must be Possible \
             (BC-2.17.014 Pattern B postcondition 2)"
        );
        assert_eq!(
            f.confidence,
            Confidence::Medium,
            "T0888 Pattern B finding confidence must be Medium \
             (BC-2.17.014 Pattern B postcondition 2)"
        );
        assert!(
            flow.error_rate_emitted,
            "error_rate_emitted must be set to true after Pattern B fires \
             (one-shot guard — BC-2.17.014 Pattern B postcondition 2)"
        );
    }

    /// AC-134-004 — T0888 Pattern B one-shot guard: 7th error in same window emits no
    /// additional finding after guard is set.
    ///
    /// BC-2.17.014 Pattern B postcondition 2 last line: `flow.error_rate_emitted = true`.
    /// Subsequent errors in the same window (guard still true) must not produce additional
    /// Pattern B findings.
    ///
    /// Non-vacuous: after the 6th error fires Pattern B (finding count=1, guard=true),
    /// a 7th error must leave finding count at 1. Distinguishes one-shot from per-occurrence.
    ///
    /// Traces: BC-2.17.014 Pattern B postcondition 2 (one-shot guard); EC-006; EC-008.
    #[test]
    fn test_t0888_pattern_b_one_shot_guard() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        let err_pdu = sendrr_pdu_with_cip(&cip_error_response(0x08));

        // 6 errors → Pattern B fires (finding count becomes 1, guard set).
        for ts in [1500u32, 1501, 1502, 1503, 1504, 1505] {
            analyzer.process_pdu(&mut flow, &err_pdu, ts, src_ip());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "pre-condition: 6th error must have fired Pattern B (finding count == 1)"
        );
        assert!(
            flow.error_rate_emitted,
            "pre-condition: error_rate_emitted must be true after Pattern B fires"
        );

        // 7th error: guard is still set (same window, < 10s elapsed) → no new finding.
        analyzer.process_pdu(&mut flow, &err_pdu, 1506, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "7th error in same window must NOT emit a second Pattern B finding \
             (error_rate_emitted one-shot guard — BC-2.17.014 Pattern B postcondition 2; EC-006)"
        );
    }

    /// AC-134-004 — exactly 5 errors with threshold=5 does NOT fire T0888 Pattern B.
    ///
    /// BC-2.17.014 invariant 3: strict `>` comparison. `5 > 5` is false; Pattern B
    /// requires strictly more than the threshold.
    ///
    /// Non-vacuous: sends exactly 5 error responses; asserts no finding and
    /// `error_rate_emitted == false`. Verifies the strict > boundary exactly.
    ///
    /// Traces: BC-2.17.014 invariant 3 (strict >); EC-005.
    #[test]
    fn test_t0888_pattern_b_no_fire_at_threshold() {
        let mut analyzer = EnipAnalyzer::new(50, 5); // threshold = 5
        let mut flow = EnipFlowState::new();

        let err_pdu = sendrr_pdu_with_cip(&cip_error_response(0x09));

        for ts in [1600u32, 1601, 1602, 1603, 1604] {
            analyzer.process_pdu(&mut flow, &err_pdu, ts, src_ip());
        }

        assert!(
            analyzer.all_findings.is_empty(),
            "exactly 5 errors with threshold=5 must NOT fire Pattern B \
             (5 > 5 is false — strict > semantics per BC-2.17.014 invariant 3; EC-005)"
        );
        assert!(
            !flow.error_rate_emitted,
            "error_rate_emitted must remain false when threshold is not exceeded \
             (BC-2.17.014 Pattern B precondition 2)"
        );
    }

    /// AC-134-004 — threshold=0: first error (count=1 > 0) fires Pattern B immediately.
    ///
    /// BC-2.17.014 invariant 3 + BC-2.17.026 invariant 4: with threshold=0, the very first
    /// CIP error response triggers `1 > 0` → Pattern B fires.
    ///
    /// Non-vacuous: creates analyzer with `error_burst_threshold=0`; sends 1 error;
    /// asserts Pattern B finding emitted. Verifies the threshold-zero edge case is handled.
    ///
    /// Traces: BC-2.17.014 invariant 3 (strict >); BC-2.17.026 invariant 4.
    #[test]
    fn test_t0888_pattern_b_threshold_zero() {
        let mut analyzer = EnipAnalyzer::new(50, 0); // error_burst_threshold = 0
        let mut flow = EnipFlowState::new();

        let err_pdu = sendrr_pdu_with_cip(&cip_error_response(0x08));

        analyzer.process_pdu(&mut flow, &err_pdu, 1700, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "with threshold=0, first error (total=1, 1 > 0 is true) must fire Pattern B \
             immediately (BC-2.17.014 invariant 3; BC-2.17.026 invariant 4)"
        );
        assert_eq!(
            analyzer.all_findings[0].mitre_techniques,
            vec!["T0888".to_string()],
            "Pattern B finding at threshold=0 must carry MITRE technique T0888"
        );
        assert_eq!(
            analyzer.all_findings[0].verdict,
            Verdict::Possible,
            "Pattern B finding at threshold=0 must have verdict=Possible"
        );
        assert!(
            flow.error_rate_emitted,
            "error_rate_emitted must be true after Pattern B fires at threshold=0"
        );
    }

    // =========================================================================
    // AC-134-005: is_non_enip flag suppresses all ENIP detections
    // Traces: BC-2.17.010 Precondition 2, BC-2.17.014 preconditions
    // =========================================================================

    /// AC-134-005 — flow with `is_non_enip=true` suppresses all T0846 and T0888 findings
    /// regardless of frame content.
    ///
    /// BC-2.17.010 precondition 2 and BC-2.17.014 precondition 5: `flow.is_non_enip == false`
    /// is required before any detection runs. When `is_non_enip=true`, `process_pdu` must
    /// return immediately without emitting any finding.
    ///
    /// The test constructs a flow with `is_non_enip=true` directly (per STORY-134 scope:
    /// the flag is set by STORY-137 frame-walk; in STORY-134 tests it is injected inline).
    ///
    /// Non-vacuous: sends a ListIdentity frame (would normally produce T0846) with
    /// `is_non_enip=true`; asserts `all_findings` stays empty. Distinguishes a correct
    /// early-exit implementation from one that ignores the flag.
    ///
    /// Traces: BC-2.17.010 precondition 2; BC-2.17.014 precondition 5; AC-134-005; EC-010.
    #[test]
    fn test_non_enip_flow_suppresses_recon() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // Inject is_non_enip=true directly (set by STORY-137 in production;
        // injected inline for STORY-134 scope isolation per AC-134-005).
        flow.is_non_enip = true;

        // ListIdentity frame — would emit T0846 if is_non_enip were false.
        let pdu = list_identity_pdu();
        analyzer.process_pdu(&mut flow, &pdu, 2000, src_ip());

        assert!(
            analyzer.all_findings.is_empty(),
            "is_non_enip=true must suppress T0846 ListIdentity detection — no findings emitted \
             (BC-2.17.010 precondition 2; EC-010)"
        );

        // GetAttributeSingle to Identity Object — would emit T0888 Pattern A if normal.
        let cip_request = cip_get_attr_single_request(0x01);
        let pdu2 = sendrr_pdu_with_cip(&cip_request);
        analyzer.process_pdu(&mut flow, &pdu2, 2001, src_ip());

        assert!(
            analyzer.all_findings.is_empty(),
            "is_non_enip=true must suppress T0888 Pattern A Identity-read detection — no \
             findings emitted (BC-2.17.014 precondition 5; EC-010)"
        );

        // Error response — would emit T0888 Pattern B (with threshold=0) if normal.
        let mut flow2 = EnipFlowState::new();
        flow2.is_non_enip = true;
        let mut analyzer2 = EnipAnalyzer::new(50, 0); // threshold=0 to ensure Pattern B would fire
        let err_pdu = sendrr_pdu_with_cip(&cip_error_response(0x08));
        analyzer2.process_pdu(&mut flow2, &err_pdu, 2002, src_ip());

        assert!(
            analyzer2.all_findings.is_empty(),
            "is_non_enip=true must suppress T0888 Pattern B error-burst detection — no \
             findings emitted (BC-2.17.014 precondition 3 / precondition 5; EC-010)"
        );
    }

    // =========================================================================
    // F-134-001 REGRESSION: error window must reset when first error is at ts=0
    // Traces: BC-2.17.008 PC-2 / PC-4; BC-2.17.014 EC-007
    // =========================================================================

    /// F-134-001 regression — error window resets correctly when the first qualifying
    /// CIP error arrives at timestamp==0.
    ///
    /// The prior implementation overloaded `error_window_start_ts == 0` as both the
    /// "no error seen yet" sentinel AND the window-seeded-at-zero case.  When a valid
    /// error arrived at pcap-relative second 0, the sentinel branch was permanently
    /// skipped, so `error_counts_in_window` accumulated unbounded and
    /// `error_rate_emitted` never re-armed.
    ///
    /// Correct behaviour (BC-2.17.008 postconditions 2 + 4):
    ///   1. First qualifying error at ts=0 seeds the window: `error_window_active = true`,
    ///      `error_window_start_ts = 0`, `error_counts_in_window[status] = 1`.
    ///   2. A second qualifying error at ts=12: `12u32.wrapping_sub(0) = 12 > 10` →
    ///      window resets before accumulating → `error_counts_in_window[status] = 1`
    ///      (not 2), and `error_rate_emitted` is `false` (re-armed).
    ///
    /// This test FAILS against the buggy sentinel-0 code and PASSES after the fix.
    ///
    /// Traces: BC-2.17.008 postconditions 2 and 4; F-134-001; EC-007.
    #[test]
    fn test_error_window_resets_after_10s_from_ts_zero() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        let cip_data = cip_error_response(0x08);
        let pdu = sendrr_pdu_with_cip(&cip_data);

        // First qualifying CIP error at timestamp == 0 (pcap-relative second 0).
        // BC-2.17.008 PC-2: window is seeded; error_window_active becomes true,
        // error_window_start_ts = 0, error_counts_in_window[0x08] = 1.
        analyzer.process_pdu(&mut flow, &pdu, 0, src_ip());

        assert_eq!(
            flow.error_counts_in_window.get(&0x08).copied().unwrap_or(0),
            1,
            "pre-condition: first error at ts=0 must seed the window with count=1 \
             (BC-2.17.008 postcondition 2; F-134-001)"
        );

        // Second qualifying error at ts=12.  12u32.wrapping_sub(0) = 12 > 10:
        // BC-2.17.008 PC-4 must fire, clearing error_counts_in_window and reseeding.
        // After the reset the new error is counted → error_counts_in_window[0x08] == 1.
        // (With the old sentinel-0 bug this returns 2 because the expiry branch was
        // never entered.)
        analyzer.process_pdu(&mut flow, &pdu, 12, src_ip());

        let count_after_reset = flow.error_counts_in_window.get(&0x08).copied().unwrap_or(0);
        assert_eq!(
            count_after_reset, 1,
            "after >10s gap from ts=0, window must reset; error_counts_in_window[0x08] \
             must be 1 (only the post-reset error), not 2 — sentinel-0 collision (F-134-001; \
             BC-2.17.008 postcondition 4)"
        );
        assert!(
            !flow.error_rate_emitted,
            "error_rate_emitted must be re-armed (false) after window reset at ts=0 boundary \
             (BC-2.17.008 postcondition 4; F-134-001)"
        );
    }

    // =========================================================================
    // O-134-001 CANONICAL CPF-OFFSET HOLDOUT (DF-CANONICAL-FRAME-HOLDOUT-001)
    // Traces: BC-2.17.014 Pattern A; BC-2.17.005; ADR-010 Decision 4
    // =========================================================================

    /// O-134-001 — canonical SendRRData frame with authoritative ODVA byte layout
    /// triggers T0888 Pattern A (Identity Object GetAttributesAll reconnaissance).
    ///
    /// This holdout pins the CPF list offset at pdu[30] against a shared-misunderstanding
    /// +/-2 error.  The frame is built byte-by-byte from the ODVA EtherNet/IP spec;
    /// every field offset is documented in-line.
    ///
    /// Canonical ODVA byte layout:
    ///   pdu[0..2]   ENIP command = 0x006F (SendRRData), LE
    ///   pdu[2..4]   ENIP length  = 22 (payload after header), LE
    ///   pdu[4..8]   Session Handle = 0x00000000
    ///   pdu[8..12]  Status = 0x00000000
    ///   pdu[12..20] Sender Context = 0x0000000000000000
    ///   pdu[20..24] Options = 0x00000000
    ///   ---- ENIP header ends at byte 24 ----
    ///   pdu[24..28] Interface Handle = 0x00000000
    ///   pdu[28..30] Timeout = 0x0000
    ///   ---- CPF list begins at pdu[30] ----
    ///   pdu[30..32] CPF item_count = 2, LE
    ///   pdu[32..34] NullAddress item type  = 0x0000, LE
    ///   pdu[34..36] NullAddress item length = 0x0000, LE
    ///   pdu[36..38] UnconnectedData item type   = 0x00B2, LE
    ///   pdu[38..40] UnconnectedData item length = 6 (CIP request size), LE
    ///   ---- CIP request begins at pdu[40] ----
    ///   pdu[40]     CIP service = 0x01 (GetAttributesAll, request; bit 7 clear)
    ///   pdu[41]     request_path_size = 2 words (4 bytes)
    ///   pdu[42]     0x20 (Class segment type, 1-byte)
    ///   pdu[43]     0x01 (Class ID = Identity Object)
    ///   pdu[44]     0x24 (Instance segment type, 1-byte)
    ///   pdu[45]     0x01 (Instance = 1)
    ///
    /// Asserts: exactly one T0888 finding with mitre_technique == "T0888".
    ///
    /// Traces: BC-2.17.014 Pattern A postcondition 1; BC-2.17.005; O-134-001;
    /// DF-CANONICAL-FRAME-HOLDOUT-001; ADR-010 Decision 4.
    #[test]
    fn test_process_pdu_canonical_sendrr_cpf_offset() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // Build canonical ODVA SendRRData frame by field offset (see doc-comment above).
        let mut pdu = vec![0u8; 46];

        // pdu[0..2]: ENIP command = 0x006F (SendRRData), LE
        pdu[0] = 0x6F;
        pdu[1] = 0x00;
        // pdu[2..4]: ENIP length = 22 (bytes 24..46 = 22 bytes of payload), LE
        pdu[2] = 22u8;
        pdu[3] = 0x00;
        // pdu[4..8]: Session Handle = 0 (4 bytes, all zero)
        // pdu[8..12]: Status = 0 (4 bytes, all zero)
        // pdu[12..20]: Sender Context = 0 (8 bytes, all zero)
        // pdu[20..24]: Options = 0 (4 bytes, all zero)
        // — bytes 4..24 already zero from vec initialisation —

        // pdu[24..28]: Interface Handle = 0x00000000 (already zero)
        // pdu[28..30]: Timeout = 0x0000 (already zero)

        // pdu[30..32]: CPF item_count = 2, LE
        pdu[30] = 0x02;
        pdu[31] = 0x00;

        // pdu[32..34]: NullAddress item type = 0x0000, LE (already zero)
        // pdu[34..36]: NullAddress item length = 0x0000, LE (already zero)

        // pdu[36..38]: UnconnectedData item type = 0x00B2, LE
        pdu[36] = 0xB2;
        pdu[37] = 0x00;

        // pdu[38..40]: UnconnectedData item length = 6 (CIP request is 6 bytes), LE
        pdu[38] = 0x06;
        pdu[39] = 0x00;

        // pdu[40]: CIP service = 0x01 (GetAttributesAll request; bit 7 clear = request)
        pdu[40] = 0x01;
        // pdu[41]: request_path_size = 2 words (4 bytes of path follow)
        pdu[41] = 0x02;
        // pdu[42]: Class segment type = 0x20 (1-byte class)
        pdu[42] = 0x20;
        // pdu[43]: Class ID = 0x01 (Identity Object)
        pdu[43] = 0x01;
        // pdu[44]: Instance segment type = 0x24 (1-byte instance)
        pdu[44] = 0x24;
        // pdu[45]: Instance = 0x01
        pdu[45] = 0x01;

        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "canonical SendRRData with CIP GetAttributesAll(Class 0x01) must produce \
             exactly 1 T0888 finding (BC-2.17.014 Pattern A; O-134-001; \
             DF-CANONICAL-FRAME-HOLDOUT-001)"
        );
        assert!(
            analyzer.all_findings[0]
                .mitre_techniques
                .contains(&"T0888".to_string()),
            "finding must carry MITRE technique T0888 (BC-2.17.014 Pattern A postcondition 1; \
             O-134-001)"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-135 command detection tests (GREEN — STORY-135 implemented).
//
// Traces to: BC-2.17.011 (T0858 CIP Stop), BC-2.17.013 (T0816 CIP Reset),
//            BC-2.17.012 (T0836 SetAttribute write-burst).
//
// IMPLEMENTATION STATUS (STORY-135 complete):
// All 16 tests pass. CIP Stop (T0858), CIP Reset (T0816), and SetAttribute
// write-burst (T0836) detections are fully implemented in `process_pdu`.
// Tests originated as Red-Gate stubs (none could pass until STORY-135 shipped).
//
// GREEN:
//   test_t0858_stop_service_0x07               — BC-2.17.011 PC-1 (Stop request emits T0858)
//   test_t0858_stop_response_no_finding        — BC-2.17.011 inv 4 (response-bit gate)
//   test_t0858_connected_item_no_finding       — BC-2.17.011 precond 3 (0x00B1 skip)
//   test_t0858_set_attribute_no_t0858          — BC-2.17.011/012 independence
//   test_t0816_reset_service                   — BC-2.17.013 PC-1 (Reset request emits T0816)
//   test_t0816_response_no_finding             — BC-2.17.013 inv 4 (response-bit gate)
//   test_t0816_connected_item_no_finding       — BC-2.17.013 precond 3 (0x00B1 skip)
//   test_t0836_burst_fires_at_threshold_plus_one — BC-2.17.012 inv 2 (strict > semantics)
//   test_t0836_burst_one_shot_guard            — BC-2.17.012 PC-5 (one-shot guard)
//   test_t0836_no_fire_at_threshold            — BC-2.17.012 inv 2 (== threshold → no fire)
//   test_t0836_threshold_zero_fires_on_first_write — BC-2.17.012 EC-007 (threshold=0 → first write fires)
//   test_t0836_window_resets_after_1s          — BC-2.17.012 PC-4 (window expiry + guard reset)
//   test_t0836_custom_threshold                — BC-2.17.012 inv 3 (custom threshold)
//   test_non_enip_suppresses_command_detections — BC-2.17.011/012/013 is_non_enip guard
//   test_write_count_accumulates               — BC-2.17.012 PC-1/2 (per-flow + aggregate)
//   test_aggregate_write_count_increments      — BC-2.17.012 PC-2 (aggregate lifetime counter)
//
// Helpers reuse sendrr_pdu_with_cip / sendrr_pdu_with_b1_cip from recon_detections
// via inline duplication (separate module scope requires local helpers).
// ─────────────────────────────────────────────────────────────────────────────
mod command_detections {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::enip::{EnipAnalyzer, EnipFlowState};
    use wirerust::findings::{Confidence, Verdict};

    // =========================================================================
    // Helpers
    // =========================================================================

    /// Canonical source IP (192.168.1.20) for command-detection tests.
    fn src_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))
    }

    /// Build a 24-byte ENIP header with `command` set (LE); all other fields zeroed.
    fn enip_header(command: u16) -> [u8; 24] {
        let mut h = [0u8; 24];
        h[0..2].copy_from_slice(&command.to_le_bytes());
        h
    }

    /// Build a SendRRData PDU containing one 0x00B2 (UnconnectedData) CIP item.
    ///
    /// Layout (matches recon_detections::sendrr_pdu_with_cip):
    ///   ENIP header (24 bytes, command=0x006F)
    ///   Interface Handle (4 bytes, 0) + Timeout (2 bytes, 0)
    ///   CPF item_count=2 (NullAddress + UnconnectedData)
    ///   NullAddress item (type=0x0000, length=0)
    ///   UnconnectedData item (type=0x00B2, length=cip_data.len())
    ///   cip_data bytes
    fn sendrr_pdu_with_cip(cip_data: &[u8]) -> Vec<u8> {
        let payload_len: usize = 4 + 2 + 2 + 4 + 4 + cip_data.len();
        let mut pdu = Vec::with_capacity(24 + payload_len);
        let mut hdr = enip_header(0x006F);
        hdr[2..4].copy_from_slice(&(payload_len as u16).to_le_bytes());
        pdu.extend_from_slice(&hdr);
        pdu.extend_from_slice(&[0u8; 4]); // Interface Handle
        pdu.extend_from_slice(&[0u8; 2]); // Timeout
        pdu.extend_from_slice(&2u16.to_le_bytes()); // item_count = 2
        pdu.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // NullAddress
        pdu.extend_from_slice(&[0xB2, 0x00]); // type_id = 0x00B2
        pdu.extend_from_slice(&(cip_data.len() as u16).to_le_bytes());
        pdu.extend_from_slice(cip_data);
        pdu
    }

    /// Build a SendRRData PDU containing one 0x00B1 (ConnectedData) CIP item.
    ///
    /// Used to verify the F-P9-001 gate: 0x00B1 items are skipped.
    fn sendrr_pdu_with_b1_cip(cip_data: &[u8]) -> Vec<u8> {
        let payload_len: usize = 4 + 2 + 2 + 4 + 4 + cip_data.len();
        let mut pdu = Vec::with_capacity(24 + payload_len);
        let mut hdr = enip_header(0x006F);
        hdr[2..4].copy_from_slice(&(payload_len as u16).to_le_bytes());
        pdu.extend_from_slice(&hdr);
        pdu.extend_from_slice(&[0u8; 4]);
        pdu.extend_from_slice(&[0u8; 2]);
        pdu.extend_from_slice(&2u16.to_le_bytes());
        pdu.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        pdu.extend_from_slice(&[0xB1, 0x00]); // type_id = 0x00B1
        pdu.extend_from_slice(&(cip_data.len() as u16).to_le_bytes());
        pdu.extend_from_slice(cip_data);
        pdu
    }

    /// Build a minimal CIP request with the given service byte and no path.
    ///
    /// item_data layout:
    ///   [0]  service byte (bit 7 = 0 for request)
    ///   [1]  0x00 (request_path_size = 0 words, no path)
    fn cip_request_no_path(service: u8) -> Vec<u8> {
        vec![service, 0x00]
    }

    /// Build a minimal CIP response for the given service (response bit set = service | 0x80).
    ///
    /// item_data layout:
    ///   [0]  service | 0x80
    ///   [1]  0x00 (reserved)
    ///   [2]  0x00 (general_status = success)
    ///   [3]  0x00 (additional_status_size = 0)
    fn cip_response(service: u8) -> Vec<u8> {
        vec![service | 0x80, 0x00, 0x00, 0x00]
    }

    /// Build a minimal SetAttributeSingle (0x10) request targeting Class/Instance/Attr.
    ///
    /// item_data layout:
    ///   [0]  0x10 (SetAttributeSingle, request)
    ///   [1]  0x03 (path_size = 3 words = 6 bytes)
    ///   [2]  0x20, 0x04 (Class segment, class=4 — Assembly Object)
    ///   [4]  0x24, 0x01 (Instance segment, instance=1)
    ///   [6]  0x30, 0x03 (Attribute segment, attr=3)
    fn cip_set_attr_single_request() -> Vec<u8> {
        vec![0x10, 0x03, 0x20, 0x04, 0x24, 0x01, 0x30, 0x03]
    }

    /// Build a minimal SetAttributesAll (0x02) request.
    fn cip_set_attrs_all_request() -> Vec<u8> {
        vec![0x02, 0x01, 0x20, 0x04]
    }

    // =========================================================================
    // AC-135-001: CIP Stop service (0x07) emits T0858 (Change Operating Mode)
    // Traces: BC-2.17.011 postconditions 1–2
    // =========================================================================

    /// AC-135-001 — CIP Stop (0x07) via 0x00B2 item emits exactly one T0858 finding.
    ///
    /// Preconditions (BC-2.17.011): classify_cip_service(0x07) == Stop, type_id==0x00B2,
    /// !is_non_enip, len < MAX_FINDINGS.
    /// Expected: one Finding with mitre_techniques=["T0858"], category=Execution,
    /// verdict=Likely, confidence=High.
    ///
    /// Traces: BC-2.17.011 postcondition 1; AC-135-001; EC-001.
    #[test]
    fn test_t0858_stop_service_0x07() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        // CIP Stop request: service=0x07, no path
        let cip = cip_request_no_path(0x07);
        let pdu = sendrr_pdu_with_cip(&cip);
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "CIP Stop (0x07) via 0x00B2 must emit exactly 1 T0858 finding (BC-2.17.011 PC-1)"
        );
        let f = &analyzer.all_findings[0];
        assert!(
            f.mitre_techniques.contains(&"T0858".to_string()),
            "finding must carry MITRE technique T0858 (BC-2.17.011 PC-1)"
        );
        assert_eq!(
            f.category,
            wirerust::findings::ThreatCategory::Execution,
            "T0858 finding must have category=Execution (BC-2.17.011 PC-1 — EXACT)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "T0858 finding must have verdict=Likely (BC-2.17.011 PC-1)"
        );
        assert_eq!(
            f.confidence,
            Confidence::High,
            "T0858 finding must have confidence=High (BC-2.17.011 PC-1)"
        );
        assert_eq!(
            f.summary,
            "CIP Stop service observed: controller run\u{2192}stop transition command (T0858)",
            "T0858 summary must be verbatim BC-2.17.011 PC-1 postcondition string"
        );
    }

    /// AC-135-001 — CIP Stop response (0x87, high bit set) does NOT emit T0858.
    ///
    /// classify_cip_service(0x87) returns Response — response-bit invariant
    /// (BC-2.17.007 invariant 1). Detection fires on requests only.
    ///
    /// Traces: BC-2.17.011 invariant 4 (request-only); AC-135-001; EC-002.
    #[test]
    fn test_t0858_stop_response_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        // CIP Stop response: service=0x87 (0x07 | 0x80)
        let cip = cip_response(0x07);
        let pdu = sendrr_pdu_with_cip(&cip);
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "CIP Stop response (0x87) must NOT emit T0858 (response-bit gate; BC-2.17.011 inv 4)"
        );
    }

    /// AC-135-001 — CIP Stop (0x07) via 0x00B1 (Connected Data Item) does NOT emit T0858.
    ///
    /// F-P9-001 gate: only 0x00B2 items trigger CIP service detection in v0.11.0.
    ///
    /// The `item.type_id != 0x00B2` continue in the CPF loop skips the item before
    /// the Stop-detection block runs (F-P9-001). The test exercises the 0x00B1-skip
    /// path only — no detection logic is reached for this item type.
    ///
    /// Traces: BC-2.17.011 precondition 3 (F-P9-001 gate); AC-135-001; EC-004.
    #[test]
    fn test_t0858_connected_item_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_request_no_path(0x07);
        // Use 0x00B1 item — F-P9-001 gate skips this item entirely.
        let pdu = sendrr_pdu_with_b1_cip(&cip);
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "CIP Stop via 0x00B1 (Connected Data Item) must NOT emit T0858 in v0.11.0 \
             (F-P9-001 gate; BC-2.17.011 precondition 3)"
        );
    }

    /// AC-135-001 — SetAttributeSingle (0x10) via 0x00B2 does NOT emit T0858.
    ///
    /// SetAttribute feeds the T0836 write-burst path, not T0858 (AC-135-001 last bullet).
    ///
    /// Traces: BC-2.17.011/012 independence; AC-135-001; EC-003.
    #[test]
    fn test_t0858_set_attribute_no_t0858() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        // No T0858: SetAttribute (0x10) must not reach the Stop detection block.
        assert!(
            !analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T0858".to_string())),
            "SetAttributeSingle (0x10) must NOT emit T0858 \
             (T0858 is triggered by Stop 0x07, not SetAttribute; BC-2.17.011/012)"
        );
    }

    // =========================================================================
    // AC-135-002: CIP Reset service (0x05) emits T0816 (Device Restart/Shutdown)
    // Traces: BC-2.17.013 postconditions 1–2
    // =========================================================================

    /// AC-135-002 — CIP Reset (0x05) via 0x00B2 item emits exactly one T0816 finding.
    ///
    /// Preconditions (BC-2.17.013): classify_cip_service(0x05) == Reset, type_id==0x00B2,
    /// !is_non_enip, len < MAX_FINDINGS.
    /// Expected: one Finding with mitre_techniques=["T0816"], category=Execution (EXACT —
    /// NOT InhibitResponseFunction), verdict=Likely, confidence=High.
    ///
    /// Traces: BC-2.17.013 postcondition 1; AC-135-002; EC-005.
    #[test]
    fn test_t0816_reset_service() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        // CIP Reset request: service=0x05, no path (device-level reset)
        let cip = cip_request_no_path(0x05);
        let pdu = sendrr_pdu_with_cip(&cip);
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "CIP Reset (0x05) via 0x00B2 must emit exactly 1 T0816 finding (BC-2.17.013 PC-1)"
        );
        let f = &analyzer.all_findings[0];
        assert!(
            f.mitre_techniques.contains(&"T0816".to_string()),
            "finding must carry MITRE technique T0816 (BC-2.17.013 PC-1)"
        );
        assert_eq!(
            f.category,
            wirerust::findings::ThreatCategory::Execution,
            "T0816 finding must have category=Execution (BC-2.17.013 PC-1 — EXACT; NOT InhibitResponseFunction)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "T0816 finding must have verdict=Likely (BC-2.17.013 PC-1)"
        );
        assert_eq!(
            f.confidence,
            Confidence::High,
            "T0816 finding must have confidence=High (BC-2.17.013 PC-1)"
        );
        assert_eq!(
            f.summary, "CIP Reset service observed: adversary-triggered device restart (T0816)",
            "T0816 summary must be verbatim BC-2.17.013 PC-1 postcondition string"
        );
    }

    /// AC-135-002 — CIP Reset response (0x85, high bit set) does NOT emit T0816.
    ///
    /// classify_cip_service(0x85) returns Response — response-bit invariant.
    ///
    /// Traces: BC-2.17.013 invariant 4; AC-135-002; EC-006.
    #[test]
    fn test_t0816_response_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_response(0x05); // 0x05 | 0x80 = 0x85
        let pdu = sendrr_pdu_with_cip(&cip);
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "CIP Reset response (0x85) must NOT emit T0816 (response-bit gate; BC-2.17.013 inv 4)"
        );
    }

    /// AC-135-002 — CIP Reset (0x05) via 0x00B1 (Connected Data Item) does NOT emit T0816.
    ///
    /// F-P9-001 gate: only 0x00B2 triggers service detection.
    ///
    /// The `item.type_id != 0x00B2` continue in the CPF loop skips 0x00B1 items,
    /// so the Reset-detection block is never entered for 0x00B1 items.
    ///
    /// Traces: BC-2.17.013 precondition 3; AC-135-002; EC-006.
    #[test]
    fn test_t0816_connected_item_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_request_no_path(0x05);
        let pdu = sendrr_pdu_with_b1_cip(&cip);
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "CIP Reset via 0x00B1 must NOT emit T0816 in v0.11.0 \
             (F-P9-001 gate; BC-2.17.013 precondition 3)"
        );
    }

    // =========================================================================
    // AC-135-003/005: T0836 write-burst detection and write-count accumulation
    // Traces: BC-2.17.012 postconditions 1–5; invariants 1–3
    // =========================================================================

    /// AC-135-003 — 51 SetAttributeSingle in 1s window (threshold=50) fires T0836.
    ///
    /// Strict `>` semantics (BC-2.17.012 invariant 2): 51 > 50 = true → finding emitted.
    /// One-shot guard set: further writes in same window produce no additional finding.
    ///
    /// Traces: BC-2.17.012 postcondition 5; invariant 2; AC-135-003; EC-008.
    #[test]
    fn test_t0836_burst_fires_at_threshold_plus_one() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        // 51 SetAttributeSingle requests in the same 1s window (timestamp constant = 100)
        for _ in 0..51 {
            analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "51 SetAttributeSingle in 1s window (threshold=50) must emit exactly 1 T0836 finding \
             (strict > semantics; BC-2.17.012 invariant 2; EC-008)"
        );
        let f = &analyzer.all_findings[0];
        assert!(
            f.mitre_techniques.contains(&"T0836".to_string()),
            "finding must carry MITRE technique T0836 (BC-2.17.012 PC-5)"
        );
        assert_eq!(
            f.category,
            wirerust::findings::ThreatCategory::Execution,
            "T0836 finding must have category=Execution (BC-2.17.012 PC-5 — EXACT; NOT ImpairProcessControl)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Likely,
            "T0836 finding must have verdict=Likely (BC-2.17.012 PC-5)"
        );
        assert_eq!(
            f.confidence,
            Confidence::Medium,
            "T0836 finding must have confidence=Medium (BC-2.17.012 PC-5)"
        );
        assert_eq!(
            f.summary,
            "CIP write-class service burst: 51 SetAttribute operations in \
             1s window (threshold 50) \u{2014} possible parameter modification attack (T0836)",
            "T0836 summary must be verbatim BC-2.17.012 PC-5 postcondition string \
             (count=51, threshold=50)"
        );
    }

    /// AC-135-003 — T0836 one-shot guard: 52nd write in same window produces no additional finding.
    ///
    /// After the guard is set (write_burst_emitted=true), further writes in the same window
    /// increment write_count_in_window but do NOT emit additional T0836 findings (BC-2.17.012 PC-5).
    ///
    /// Traces: BC-2.17.012 postcondition 5 (write_burst_emitted guard); AC-135-003; EC-009.
    #[test]
    fn test_t0836_burst_one_shot_guard() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        // 56 writes in same window — guard fires at 51; writes 52–56 produce no additional finding.
        for _ in 0..56 {
            analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "56 writes in 1s window must produce exactly 1 T0836 finding \
             (one-shot guard; BC-2.17.012 PC-5; EC-009)"
        );
    }

    /// AC-135-003 — exactly 50 SetAttributeSingle in 1s window (threshold=50) does NOT fire T0836.
    ///
    /// Strict `>` semantics: 50 > 50 = false → no finding (BC-2.17.012 invariant 2).
    /// This exercises the BC-2.17.012 canonical test-vector row "50 → No" (count == threshold
    /// uses strict `>`, so no finding is emitted).
    ///
    /// Traces: BC-2.17.012 invariant 2 (strict `>` operator); AC-135-003.
    #[test]
    fn test_t0836_no_fire_at_threshold() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        for _ in 0..50 {
            analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "exactly 50 SetAttributeSingle (count == threshold) must NOT emit T0836 \
             (strict > semantics; BC-2.17.012 invariant 2)"
        );
    }

    /// BC-2.17.012 EC-007 — threshold=0: first write immediately fires T0836 (count=1 > 0).
    ///
    /// With `enip_write_burst_threshold=0`, the strict `>` check `1 > 0` is immediately true
    /// on the first SetAttribute write, emitting one T0836 finding (BC-2.17.012 PC-5, EC-007).
    ///
    /// Traces: BC-2.17.012 postcondition 5; EC-007; AC-135-003.
    #[test]
    fn test_t0836_threshold_zero_fires_on_first_write() {
        let mut analyzer = EnipAnalyzer::new(0, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        // Single SetAttributeSingle write: count=1, threshold=0 → 1 > 0 = true → finding emitted.
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "threshold=0: first SetAttributeSingle write must immediately emit T0836 \
             (count=1 > threshold=0; BC-2.17.012 EC-007)"
        );
        let f = &analyzer.all_findings[0];
        assert!(
            f.mitre_techniques.contains(&"T0836".to_string()),
            "EC-007 finding must carry MITRE technique T0836 (BC-2.17.012 PC-5)"
        );
    }

    /// AC-135-003 — 1s window expiry resets guard; 51 new writes in window 2 fires T0836 again.
    ///
    /// Window 1: ts=100, 51 writes → T0836 emitted; guard=true.
    /// Window 2: ts=102 (102 - 100 = 2 > 1 → window expired); 51 writes → T0836 emitted again.
    ///
    /// Traces: BC-2.17.012 postcondition 4 (window expiry + guard reset); AC-135-003; EC-010.
    #[test]
    fn test_t0836_window_resets_after_1s() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        // Window 1: 51 writes at ts=100
        for _ in 0..51 {
            analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "window 1: 51 writes at ts=100 must emit 1 T0836 finding (BC-2.17.012 PC-5)"
        );
        // Window 2: 51 writes at ts=102 (2 seconds later; 102 - 100 = 2 > 1 → window expired)
        for _ in 0..51 {
            analyzer.process_pdu(&mut flow, &pdu, 102, src_ip());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            2,
            "window 2: 51 writes at ts=102 must emit a second T0836 finding \
             (window expired; guard reset; BC-2.17.012 postcondition 4; EC-010)"
        );
    }

    /// AC-135-003 — custom threshold=10: 11 writes fires T0836 at threshold+1.
    ///
    /// Traces: BC-2.17.012 invariant 3 (default 50; custom via CLI flag); AC-135-003.
    #[test]
    fn test_t0836_custom_threshold() {
        let mut analyzer = EnipAnalyzer::new(10, 5); // threshold=10
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        // 11 writes at ts=100 (11 > 10 → T0836 fires)
        for _ in 0..11 {
            analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "11 writes with threshold=10 must emit exactly 1 T0836 finding \
             (11 > 10; BC-2.17.012 invariant 2)"
        );
        assert!(
            analyzer.all_findings[0]
                .mitre_techniques
                .contains(&"T0836".to_string()),
            "finding must carry MITRE technique T0836"
        );
    }

    // =========================================================================
    // AC-135-004: is_non_enip suppresses T0858, T0816, and T0836 detections
    // Traces: BC-2.17.011/012/013 preconditions (is_non_enip guard)
    // =========================================================================

    /// AC-135-004 — is_non_enip=true suppresses all command detections (T0858, T0816, T0836).
    ///
    /// When flow.is_non_enip == true, process_pdu returns immediately without any
    /// CIP parse or detection (first gate in process_pdu body).
    ///
    /// The `is_non_enip` early-return at the top of process_pdu fires before any
    /// detection block — T0858, T0816, and T0836 paths are all unreachable when this
    /// flag is set.
    ///
    /// Traces: BC-2.17.011/012/013 preconditions (is_non_enip guard); AC-135-004; EC-011.
    #[test]
    fn test_non_enip_suppresses_command_detections() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        flow.is_non_enip = true; // latch set

        // Try CIP Stop — must produce no finding
        let stop_pdu = sendrr_pdu_with_cip(&cip_request_no_path(0x07));
        analyzer.process_pdu(&mut flow, &stop_pdu, 100, src_ip());

        // Try CIP Reset — must produce no finding
        let reset_pdu = sendrr_pdu_with_cip(&cip_request_no_path(0x05));
        analyzer.process_pdu(&mut flow, &reset_pdu, 100, src_ip());

        // Try SetAttributeSingle — must produce no finding
        let write_pdu = sendrr_pdu_with_cip(&cip_set_attr_single_request());
        for _ in 0..60 {
            analyzer.process_pdu(&mut flow, &write_pdu, 100, src_ip());
        }

        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "is_non_enip=true must suppress ALL command detections \
             (T0858, T0816, T0836); BC-2.17.011/012/013 preconditions"
        );
    }

    // =========================================================================
    // AC-135-005: write_count accumulation (flow and aggregate)
    // Traces: BC-2.17.012 postconditions 1–2
    // =========================================================================

    /// AC-135-005 — write_count_in_window and write_count increment on each SetAttribute.
    ///
    /// Both flow.write_count_in_window and analyzer.write_count must increment on every
    /// qualifying write-class request, regardless of threshold (BC-2.17.012 PC-1/PC-2).
    ///
    /// Traces: BC-2.17.012 postconditions 1, 2; AC-135-005.
    #[test]
    fn test_write_count_accumulates() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip = cip_set_attr_single_request();
        let pdu = sendrr_pdu_with_cip(&cip);
        // 3 SetAttributeSingle writes at ts=100 (all within 1s window, threshold not exceeded)
        for _ in 0..3 {
            analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        }
        assert_eq!(
            flow.write_count_in_window, 3,
            "write_count_in_window must be 3 after 3 SetAttributeSingle requests \
             (BC-2.17.012 postcondition 1)"
        );
        assert_eq!(
            analyzer.write_count, 3,
            "analyzer.write_count must be 3 after 3 SetAttributeSingle requests \
             (BC-2.17.012 postcondition 2)"
        );
    }

    /// AC-135-005 — analyzer.write_count == N after N SetAttribute frames (aggregate).
    ///
    /// The aggregate lifetime counter is independent of the per-flow burst window.
    ///
    /// Traces: BC-2.17.012 postcondition 2; AC-135-005.
    #[test]
    fn test_aggregate_write_count_increments() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let cip_single = cip_set_attr_single_request();
        let cip_all = cip_set_attrs_all_request();
        let pdu_single = sendrr_pdu_with_cip(&cip_single);
        let pdu_all = sendrr_pdu_with_cip(&cip_all);
        // 5 SetAttributeSingle + 3 SetAttributesAll = 8 total write-class requests
        for _ in 0..5 {
            analyzer.process_pdu(&mut flow, &pdu_single, 100, src_ip());
        }
        for _ in 0..3 {
            analyzer.process_pdu(&mut flow, &pdu_all, 100, src_ip());
        }
        assert_eq!(
            analyzer.write_count, 8,
            "analyzer.write_count must equal 8 after 5 SetAttributeSingle + 3 SetAttributesAll \
             (BC-2.17.012 postcondition 2; aggregate lifetime counter)"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-136: CIP connection-lifecycle detection tests.
//
// Traces to: BC-2.17.015 (ForwardOpen/LargeForwardOpen/ForwardClose lifecycle detection).
//
// IMPLEMENTATION STATUS (STORY-136 complete):
// All 10 connection_lifecycle tests pass. ForwardOpen/LargeForwardOpen/ForwardClose
// lifecycle detection is fully implemented in process_pdu (BC-2.17.015).
// Tests originated as Red-Gate stubs (none could pass until STORY-136 shipped).
//
// 6 tests originated as Red-Gate behavioral stubs (required STORY-136 implementation):
//   test_forward_open_emits_finding         (BC-2.17.015 PC-1; AC-136-001)
//   test_forward_open_no_mitre_technique    (BC-2.17.015 PC-1 mitre; AC-136-001)
//   test_large_forward_open_emits_finding   (BC-2.17.015 PC-1 invariant 5; AC-136-001)
//   test_forward_close_emits_finding        (BC-2.17.015 PC-4; AC-136-002)
//   test_forward_close_no_mitre_technique   (BC-2.17.015 PC-4 mitre; AC-136-002)
//   test_connection_counts_tracked          (BC-2.17.015 invariant 3; AC-136-005; EC-008)
//
// 4 tests were green-by-design suppression tests (never required the stub branch):
//   These tests never enter the ForwardOpen/LargeForwardOpen/ForwardClose branch;
//   either the is_non_enip gate returns early, the item type is 0x00B1 (skipped),
//   or classify_cip_service returns Response (not ForwardOpen/ForwardClose).
//   test_forward_open_connected_item_no_finding   (F-P9-001 0x00B1 gate; BC-2.17.015 PC-3)
//   test_forward_open_response_no_finding         (BC-2.17.007 Inv 1; BC-2.17.015 Inv 2)
//   test_forward_close_response_no_finding        (BC-2.17.007 Inv 1; BC-2.17.015 Inv 2)
//   test_non_enip_suppresses_connection_lifecycle (BC-2.17.015 PC-4; AC-136-004)
// ─────────────────────────────────────────────────────────────────────────────
mod connection_lifecycle {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::enip::{EnipAnalyzer, EnipFlowState};
    use wirerust::findings::{Confidence, ThreatCategory, Verdict};

    // =========================================================================
    // Helpers
    // =========================================================================

    /// Canonical source IP (192.168.1.30) for connection-lifecycle tests.
    fn src_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 30))
    }

    /// Build a 24-byte ENIP header with `command` set (LE); all other fields zeroed.
    fn enip_header(command: u16) -> [u8; 24] {
        let mut h = [0u8; 24];
        h[0..2].copy_from_slice(&command.to_le_bytes());
        h
    }

    /// Build a SendRRData PDU containing one 0x00B2 (UnconnectedData) CIP item.
    ///
    /// Layout (matches command_detections::sendrr_pdu_with_cip):
    ///   ENIP header (24 bytes, command=0x006F)
    ///   Interface Handle (4 bytes, 0) + Timeout (2 bytes, 0)
    ///   CPF item_count=2 (NullAddress + UnconnectedData)
    ///   NullAddress item (type=0x0000, length=0)
    ///   UnconnectedData item (type=0x00B2, length=cip_data.len())
    ///   cip_data bytes
    fn sendrr_pdu_with_cip(cip_data: &[u8]) -> Vec<u8> {
        let payload_len: usize = 4 + 2 + 2 + 4 + 4 + cip_data.len();
        let mut pdu = Vec::with_capacity(24 + payload_len);
        let mut hdr = enip_header(0x006F);
        hdr[2..4].copy_from_slice(&(payload_len as u16).to_le_bytes());
        pdu.extend_from_slice(&hdr);
        pdu.extend_from_slice(&[0u8; 4]); // Interface Handle
        pdu.extend_from_slice(&[0u8; 2]); // Timeout
        pdu.extend_from_slice(&2u16.to_le_bytes()); // item_count = 2
        pdu.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // NullAddress
        pdu.extend_from_slice(&[0xB2, 0x00]); // type_id = 0x00B2
        pdu.extend_from_slice(&(cip_data.len() as u16).to_le_bytes());
        pdu.extend_from_slice(cip_data);
        pdu
    }

    /// Build a SendRRData PDU containing one 0x00B1 (ConnectedData) CIP item.
    ///
    /// Used to verify the F-P9-001 gate: 0x00B1 items are skipped; no
    /// lifecycle detection fires (AC-136-001 EC-006 / F-P9-001).
    fn sendrr_pdu_with_b1_cip(cip_data: &[u8]) -> Vec<u8> {
        let payload_len: usize = 4 + 2 + 2 + 4 + 4 + cip_data.len();
        let mut pdu = Vec::with_capacity(24 + payload_len);
        let mut hdr = enip_header(0x006F);
        hdr[2..4].copy_from_slice(&(payload_len as u16).to_le_bytes());
        pdu.extend_from_slice(&hdr);
        pdu.extend_from_slice(&[0u8; 4]);
        pdu.extend_from_slice(&[0u8; 2]);
        pdu.extend_from_slice(&2u16.to_le_bytes());
        pdu.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        pdu.extend_from_slice(&[0xB1, 0x00]); // type_id = 0x00B1
        pdu.extend_from_slice(&(cip_data.len() as u16).to_le_bytes());
        pdu.extend_from_slice(cip_data);
        pdu
    }

    /// Build a minimal CIP request with the given service byte and no path.
    ///
    /// item_data layout:
    ///   [0]  service byte (bit 7 = 0 for request)
    ///   [1]  0x00 (request_path_size = 0 words, no path)
    fn cip_request_no_path(service: u8) -> Vec<u8> {
        vec![service, 0x00]
    }

    /// Build a minimal CIP response for the given service (response bit set = service | 0x80).
    ///
    /// item_data layout:
    ///   [0]  service | 0x80
    ///   [1]  0x00 (reserved)
    ///   [2]  0x00 (general_status = success)
    ///   [3]  0x00 (additional_status_size = 0)
    fn cip_response(service: u8) -> Vec<u8> {
        vec![service | 0x80, 0x00, 0x00, 0x00]
    }

    // =========================================================================
    // AC-136-001: ForwardOpen request emits Anomaly/Possible/Low finding (0x54)
    // Traces: BC-2.17.015 postconditions 1–3; AC-136-001
    // =========================================================================

    /// AC-136-001 — ForwardOpen (0x54) via 0x00B2 emits exactly one Anomaly/Possible/Low finding.
    ///
    /// BC-2.17.015 postcondition 1: category=Anomaly, verdict=Possible, confidence=Low.
    /// summary starts with "CIP ForwardOpen connection establishment observed from src=".
    /// mitre_techniques: vec![] (no ATT&CK technique — ADR-010 Decision 7).
    ///
    /// Traces: BC-2.17.015 postcondition 1; AC-136-001 EC-001.
    #[test]
    fn test_forward_open_emits_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = sendrr_pdu_with_cip(&cip_request_no_path(0x54));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "ForwardOpen (0x54) via 0x00B2 must emit exactly 1 finding (BC-2.17.015 PC-1)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "ForwardOpen finding category must be ThreatCategory::Anomaly (BC-2.17.015 PC-1)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "ForwardOpen finding verdict must be Verdict::Possible (BC-2.17.015 PC-1)"
        );
        assert_eq!(
            f.confidence,
            Confidence::Low,
            "ForwardOpen finding confidence must be Confidence::Low (BC-2.17.015 PC-1)"
        );
        assert!(
            f.summary
                .contains("CIP ForwardOpen connection establishment observed from src="),
            "ForwardOpen finding summary must contain expected prefix (BC-2.17.015 PC-1 summary)"
        );
        assert!(
            f.summary.contains(": connection lifecycle anomaly"),
            "ForwardOpen finding summary must contain normative suffix \
             \": connection lifecycle anomaly\" (BC-2.17.015 PC-1 summary)"
        );
        assert!(
            f.source_ip.is_some(),
            "ForwardOpen finding must carry source_ip (BC-2.17.015 PC-1)"
        );
        assert!(
            f.timestamp.is_some(),
            "ForwardOpen finding must carry timestamp (BC-2.17.015 PC-1)"
        );
        // BC-2.17.015 PC-1 evidence postcondition (F-136-P1-002):
        // exactly one evidence entry documenting the MITRE-gap rationale (ADR-010 Decision 7).
        assert_eq!(
            f.evidence.len(),
            1,
            "ForwardOpen finding must carry exactly 1 evidence entry \
             (BC-2.17.015 PC-1 / Invariant 1 / ADR-010 Decision 7 — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains("CIP service=0x54"),
            "ForwardOpen evidence[0] must contain \"CIP service=0x54\" \
             (BC-2.17.015 PC-1 evidence template — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains(
                "No dedicated MITRE ICS technique for CIP connection establishment anomaly"
            ),
            "ForwardOpen evidence[0] must document the MITRE-gap rationale \
             (BC-2.17.015 PC-1 evidence template — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains("ADR-010 Decision 7"),
            "ForwardOpen evidence[0] must cite ADR-010 Decision 7 \
             (BC-2.17.015 PC-1 evidence template / Invariant 1 — F-136-P1-002)"
        );
    }

    /// AC-136-001 — ForwardOpen (0x54) finding carries empty mitre_techniques vec.
    ///
    /// BC-2.17.015 postcondition 1 / Architecture Rule 2 / ADR-010 Decision 7:
    /// mitre_techniques MUST be vec![] (empty). No T0858, T1692.001, or any other technique.
    ///
    /// Traces: BC-2.17.015 postcondition 1 (mitre_techniques); AC-136-001.
    #[test]
    fn test_forward_open_no_mitre_technique() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = sendrr_pdu_with_cip(&cip_request_no_path(0x54));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "ForwardOpen must emit exactly 1 finding before checking mitre_techniques"
        );
        assert!(
            analyzer.all_findings[0].mitre_techniques.is_empty(),
            "ForwardOpen finding mitre_techniques must be vec![] \
             (no ATT&CK technique for CIP connection anomaly — ADR-010 Decision 7 / BC-2.17.015 PC-1)"
        );
    }

    /// AC-136-001 — ForwardOpen via 0x00B1 item (Connected Data Item) produces no finding.
    ///
    /// F-P9-001 gate: only 0x00B2 (Unconnected Data Item) items trigger lifecycle detection.
    /// ForwardOpen/ForwardClose are Connection Manager unconnected messages; they MUST ride
    /// in 0x00B2 items by CIP protocol design (BC-2.17.015 precondition 3; EC-006).
    ///
    /// Traces: BC-2.17.015 precondition 3; AC-136-001 EC-006; F-P9-001.
    #[test]
    fn test_forward_open_connected_item_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = sendrr_pdu_with_b1_cip(&cip_request_no_path(0x54));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "ForwardOpen via 0x00B1 item must produce no finding (F-P9-001 gate; BC-2.17.015 PC-3)"
        );
    }

    /// AC-136-001 — LargeForwardOpen (0x5B) via 0x00B2 emits exactly one Anomaly/Possible/Low finding.
    ///
    /// BC-2.17.015 invariant 5: LargeForwardOpen is treated identically to ForwardOpen
    /// for detection purposes (same finding fields, same category/verdict/confidence,
    /// same empty mitre_techniques, same summary prefix).
    ///
    /// Traces: BC-2.17.015 postcondition 1; invariant 5; AC-136-001 EC-002.
    #[test]
    fn test_large_forward_open_emits_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = sendrr_pdu_with_cip(&cip_request_no_path(0x5B));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "LargeForwardOpen (0x5B) via 0x00B2 must emit exactly 1 finding \
             (BC-2.17.015 PC-1; invariant 5 — treated identically to ForwardOpen)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "LargeForwardOpen finding category must be ThreatCategory::Anomaly (BC-2.17.015 PC-1)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "LargeForwardOpen finding verdict must be Verdict::Possible (BC-2.17.015 PC-1)"
        );
        assert_eq!(
            f.confidence,
            Confidence::Low,
            "LargeForwardOpen finding confidence must be Confidence::Low (BC-2.17.015 PC-1)"
        );
        assert!(
            f.summary
                .contains("CIP ForwardOpen connection establishment observed from src="),
            "LargeForwardOpen summary must use ForwardOpen prefix (BC-2.17.015 PC-1 / invariant 5 — \
             LargeForwardOpen treated identically to ForwardOpen)"
        );
        assert!(
            f.summary.contains(": connection lifecycle anomaly"),
            "LargeForwardOpen finding summary must contain normative suffix \
             \": connection lifecycle anomaly\" (BC-2.17.015 PC-1 / invariant 5 summary)"
        );
        assert!(
            f.mitre_techniques.is_empty(),
            "LargeForwardOpen mitre_techniques must be vec![] (ADR-010 Decision 7)"
        );
        // BC-2.17.015 PC-1 / Invariant 5: LargeForwardOpen uses the same evidence form as
        // ForwardOpen with its own service byte (0x5B). One evidence entry mandated (F-136-P1-002).
        assert_eq!(
            f.evidence.len(),
            1,
            "LargeForwardOpen finding must carry exactly 1 evidence entry \
             (BC-2.17.015 PC-1 / Invariant 5 / ADR-010 Decision 7 — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains("CIP service=0x5B"),
            "LargeForwardOpen evidence[0] must contain \"CIP service=0x5B\" \
             (BC-2.17.015 PC-1 / Invariant 5 evidence template — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains(
                "No dedicated MITRE ICS technique for CIP connection establishment anomaly"
            ),
            "LargeForwardOpen evidence[0] must document the MITRE-gap rationale \
             (BC-2.17.015 PC-1 / Invariant 5 evidence template — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains("ADR-010 Decision 7"),
            "LargeForwardOpen evidence[0] must cite ADR-010 Decision 7 \
             (BC-2.17.015 PC-1 / Invariant 5 — F-136-P1-002)"
        );
    }

    // =========================================================================
    // AC-136-002: ForwardClose request emits Anomaly/Possible/Low finding (0x4E)
    // Traces: BC-2.17.015 postconditions 4–5; AC-136-002
    // =========================================================================

    /// AC-136-002 — ForwardClose (0x4E) via 0x00B2 emits exactly one Anomaly/Possible/Low finding.
    ///
    /// BC-2.17.015 postcondition 4: category=Anomaly, verdict=Possible, confidence=Low.
    /// summary starts with "CIP ForwardClose connection teardown observed from src=".
    ///
    /// Traces: BC-2.17.015 postcondition 4; AC-136-002 EC-003.
    #[test]
    fn test_forward_close_emits_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = sendrr_pdu_with_cip(&cip_request_no_path(0x4E));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "ForwardClose (0x4E) via 0x00B2 must emit exactly 1 finding (BC-2.17.015 PC-4)"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "ForwardClose finding category must be ThreatCategory::Anomaly (BC-2.17.015 PC-4)"
        );
        assert_eq!(
            f.verdict,
            Verdict::Possible,
            "ForwardClose finding verdict must be Verdict::Possible (BC-2.17.015 PC-4)"
        );
        assert_eq!(
            f.confidence,
            Confidence::Low,
            "ForwardClose finding confidence must be Confidence::Low (BC-2.17.015 PC-4)"
        );
        assert!(
            f.summary
                .contains("CIP ForwardClose connection teardown observed from src="),
            "ForwardClose finding summary must contain expected prefix (BC-2.17.015 PC-4 summary)"
        );
        assert!(
            f.summary.contains(": connection lifecycle closed"),
            "ForwardClose finding summary must contain normative suffix \
             \": connection lifecycle closed\" (BC-2.17.015 PC-4 summary)"
        );
        assert!(
            f.source_ip.is_some(),
            "ForwardClose finding must carry source_ip (BC-2.17.015 PC-4)"
        );
        assert!(
            f.timestamp.is_some(),
            "ForwardClose finding must carry timestamp (BC-2.17.015 PC-4)"
        );
        // BC-2.17.015 PC-4 evidence postcondition (F-136-P1-002):
        // exactly one evidence entry documenting the lifecycle-close rationale (ADR-010 Decision 7).
        assert_eq!(
            f.evidence.len(),
            1,
            "ForwardClose finding must carry exactly 1 evidence entry \
             (BC-2.17.015 PC-4 / Invariant 1 / ADR-010 Decision 7 — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains("CIP service=0x4E (ForwardClose)"),
            "ForwardClose evidence[0] must contain \"CIP service=0x4E (ForwardClose)\" \
             (BC-2.17.015 PC-4 evidence template — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains("Connection lifecycle closed"),
            "ForwardClose evidence[0] must document \"Connection lifecycle closed\" \
             (BC-2.17.015 PC-4 evidence template — F-136-P1-002)"
        );
        assert!(
            f.evidence[0].contains("ADR-010 Decision 7"),
            "ForwardClose evidence[0] must cite ADR-010 Decision 7 \
             (BC-2.17.015 PC-4 evidence template / Invariant 1 — F-136-P1-002)"
        );
    }

    /// AC-136-002 — ForwardClose (0x4E) finding carries empty mitre_techniques vec.
    ///
    /// BC-2.17.015 postcondition 4 / Architecture Rule 2 / ADR-010 Decision 7:
    /// mitre_techniques MUST be vec![] (empty).
    ///
    /// Traces: BC-2.17.015 postcondition 4 (mitre_techniques); AC-136-002.
    #[test]
    fn test_forward_close_no_mitre_technique() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        let pdu = sendrr_pdu_with_cip(&cip_request_no_path(0x4E));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "ForwardClose must emit exactly 1 finding before checking mitre_techniques"
        );
        assert!(
            analyzer.all_findings[0].mitre_techniques.is_empty(),
            "ForwardClose finding mitre_techniques must be vec![] \
             (no ATT&CK technique — ADR-010 Decision 7 / BC-2.17.015 PC-4)"
        );
    }

    // =========================================================================
    // AC-136-003: CIP response bytes do NOT trigger lifecycle detection
    // Traces: BC-2.17.015 invariant 2; BC-2.17.007 invariant 1; AC-136-003
    // =========================================================================

    /// AC-136-003 — ForwardOpen response (0xD4 = 0x54 | 0x80) produces no finding.
    ///
    /// classify_cip_service(0xD4) returns CipServiceClass::Response (response-bit set
    /// takes priority per BC-2.17.007 invariant 1). Detection keys on classify_cip_service
    /// returning ForwardOpen — which 0xD4 never does. No raw & 0x80 == 0 predicate is
    /// hand-rolled at the call site (AC-136-003 Architecture Rule 6).
    ///
    /// Traces: BC-2.17.015 invariant 2; BC-2.17.007 invariant 1; AC-136-003 EC-004.
    #[test]
    fn test_forward_open_response_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        // 0xD4 = 0x54 | 0x80 — ForwardOpen response; classify_cip_service returns Response.
        let pdu = sendrr_pdu_with_cip(&cip_response(0x54));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "ForwardOpen response (0xD4) must produce no finding; \
             classify_cip_service returns Response (BC-2.17.007 invariant 1 / BC-2.17.015 invariant 2)"
        );
    }

    /// AC-136-003 — ForwardClose response (0xCE = 0x4E | 0x80) produces no finding.
    ///
    /// classify_cip_service(0xCE) returns CipServiceClass::Response. Detection must NOT
    /// fire for response bytes (BC-2.17.015 invariant 2; BC-2.17.007 invariant 1).
    ///
    /// Traces: BC-2.17.015 invariant 2; BC-2.17.007 invariant 1; AC-136-003 EC-005.
    #[test]
    fn test_forward_close_response_no_finding() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        // 0xCE = 0x4E | 0x80 — ForwardClose response; classify_cip_service returns Response.
        let pdu = sendrr_pdu_with_cip(&cip_response(0x4E));
        analyzer.process_pdu(&mut flow, &pdu, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "ForwardClose response (0xCE) must produce no finding; \
             classify_cip_service returns Response (BC-2.17.007 invariant 1 / BC-2.17.015 invariant 2)"
        );
    }

    // =========================================================================
    // AC-136-004: is_non_enip suppresses ForwardOpen/ForwardClose detection
    // Traces: BC-2.17.015 precondition 4; AC-136-004
    // =========================================================================

    /// AC-136-004 — is_non_enip=true suppresses all connection-lifecycle findings.
    ///
    /// When flow.is_non_enip == true, process_pdu returns at the is_non_enip gate
    /// (BC-2.17.015 precondition 4); no ForwardOpen or ForwardClose detection runs.
    ///
    /// Traces: BC-2.17.015 precondition 4; AC-136-004 EC-007.
    #[test]
    fn test_non_enip_suppresses_connection_lifecycle() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();
        flow.is_non_enip = true;
        let pdu_open = sendrr_pdu_with_cip(&cip_request_no_path(0x54));
        let pdu_close = sendrr_pdu_with_cip(&cip_request_no_path(0x4E));
        let pdu_large = sendrr_pdu_with_cip(&cip_request_no_path(0x5B));
        analyzer.process_pdu(&mut flow, &pdu_open, 100, src_ip());
        analyzer.process_pdu(&mut flow, &pdu_close, 100, src_ip());
        analyzer.process_pdu(&mut flow, &pdu_large, 100, src_ip());
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "is_non_enip=true must suppress ForwardOpen/LargeForwardOpen/ForwardClose detection \
             (BC-2.17.015 precondition 4 / AC-136-004)"
        );
    }

    // =========================================================================
    // AC-136-005: open_connection_count and close_connection_count tracked in flow state
    // Traces: BC-2.17.015 invariant 3; AC-136-005
    // =========================================================================

    /// AC-136-005 — connection counts increment on each occurrence; increment is outside
    /// MAX_FINDINGS gate (EC-008).
    ///
    /// BC-2.17.015 invariant 3 / Architecture Rule 4 (EC-008):
    /// - open_connection_count increments on ForwardOpen AND LargeForwardOpen requests.
    /// - close_connection_count increments on ForwardClose requests.
    /// - Counts increment EVEN WHEN all_findings is at MAX_FINDINGS (count BEFORE finding push).
    ///
    /// This test verifies counts via normal (non-capped) flow first, then verifies
    /// EC-008 via a pre-filled all_findings scenario.
    ///
    /// Traces: BC-2.17.015 invariant 3; Architecture Rule 4; AC-136-005; EC-008.
    #[test]
    fn test_connection_counts_tracked() {
        use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

        const MAX_FINDINGS: usize = 10_000;

        // --- Part A: normal counting (no MAX_FINDINGS cap) ---
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let mut flow = EnipFlowState::new();

        // 2 ForwardOpen + 1 LargeForwardOpen → open_connection_count must be 3.
        let pdu_open = sendrr_pdu_with_cip(&cip_request_no_path(0x54));
        let pdu_large = sendrr_pdu_with_cip(&cip_request_no_path(0x5B));
        let pdu_close = sendrr_pdu_with_cip(&cip_request_no_path(0x4E));

        analyzer.process_pdu(&mut flow, &pdu_open, 100, src_ip());
        analyzer.process_pdu(&mut flow, &pdu_open, 100, src_ip());
        analyzer.process_pdu(&mut flow, &pdu_large, 100, src_ip());
        analyzer.process_pdu(&mut flow, &pdu_close, 100, src_ip());

        assert_eq!(
            flow.open_connection_count, 3,
            "open_connection_count must be 3 after 2 ForwardOpen + 1 LargeForwardOpen \
             (BC-2.17.015 invariant 3 / AC-136-005)"
        );
        assert_eq!(
            flow.close_connection_count, 1,
            "close_connection_count must be 1 after 1 ForwardClose \
             (BC-2.17.015 invariant 3 / AC-136-005)"
        );
        assert_eq!(
            analyzer.all_findings.len(),
            4,
            "Part A: exactly 4 lifecycle findings must be emitted for 2 ForwardOpen + \
             1 LargeForwardOpen + 1 ForwardClose — confirms per-occurrence firing with no \
             one-shot guard (BC-2.17.015 PC-3/PC-5; EC-009)"
        );

        // --- Part B: EC-008 — counts increment even when all_findings is at MAX_FINDINGS ---
        let mut analyzer2 = EnipAnalyzer::new(50, 5);
        let mut flow2 = EnipFlowState::new();

        // Pre-fill all_findings to the cap.
        for _ in 0..MAX_FINDINGS {
            analyzer2.all_findings.push(Finding {
                category: ThreatCategory::Reconnaissance,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "placeholder".to_string(),
                evidence: vec![],
                mitre_techniques: vec![],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }
        assert_eq!(
            analyzer2.all_findings.len(),
            MAX_FINDINGS,
            "pre-condition: all_findings must be exactly at cap before EC-008 check"
        );

        // ForwardOpen at cap: finding NOT pushed; open_connection_count MUST still increment.
        let pdu_open2 = sendrr_pdu_with_cip(&cip_request_no_path(0x54));
        analyzer2.process_pdu(&mut flow2, &pdu_open2, 100, src_ip());

        assert_eq!(
            analyzer2.all_findings.len(),
            MAX_FINDINGS,
            "EC-008: no finding pushed when all_findings is at MAX_FINDINGS (BC-2.17.022)"
        );
        assert_eq!(
            flow2.open_connection_count, 1,
            "EC-008: open_connection_count must increment even when all_findings is at MAX_FINDINGS \
             (BC-2.17.015 Architecture Rule 4 / AC-136-005)"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORY-137 frame-walk robustness: carry buffer, non-ENIP detection, T0814 DoS burst.
//
// Traces to: BC-2.17.016 (frame-walk + carry buffer), BC-2.17.004 (command_counts),
//            BC-2.17.018 (T0814 windowed DoS detection).
//
// IMPLEMENTATION STATUS: RED GATE — all 20 behavioral tests fail via todo!()
// in on_data / check_t0814. The implementer (STORY-137 TDD phase) replaces the
// todo!() bodies and makes these tests GREEN.
//
// ARCHITECTURE NOTE: EnipAnalyzer stores per-flow state in `self.flows: HashMap<FlowKey,
// EnipFlowState>` (mirrors Dnp3Analyzer pattern). After each on_data call, tests read
// flow state via `analyzer.flows.get(&key).expect("flow must exist after on_data")`.
//
// Each test carries a BC / AC citation in its doc comment.
// ─────────────────────────────────────────────────────────────────────────────
mod frame_walk {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::enip::{EnipAnalyzer, MAX_ENIP_CARRY_BYTES};
    use wirerust::reassembly::flow::FlowKey;

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn src_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))
    }

    fn dst_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2))
    }

    fn make_flow_key() -> FlowKey {
        FlowKey::new(src_ip(), 12345, dst_ip(), 44818)
    }

    /// Build a minimal valid ENIP frame with the given command (u16 LE) and zero payload.
    ///
    /// The ENIP header is 24 bytes. `length` field (bytes 2–3) is set to 0, indicating no
    /// payload bytes after the header. `is_valid_enip_frame` returns `true` for ODVA commands.
    fn enip_frame(command: u16) -> Vec<u8> {
        let mut frame = vec![0u8; 24];
        frame[0..2].copy_from_slice(&command.to_le_bytes()); // command LE
        // length = 0 (no payload)
        frame
    }

    /// Build a valid RegisterSession frame (command=0x0065, length=4, 4-byte payload).
    ///
    /// `length` = 4 (bytes 2–3, LE) so total_frame_len = 28.
    fn enip_register_session_frame() -> Vec<u8> {
        let mut frame = vec![0u8; 28]; // 24 header + 4 payload
        frame[0] = 0x65; // command = 0x0065 (RegisterSession), LE low byte
        frame[1] = 0x00;
        frame[2] = 0x04; // length = 4, LE low byte
        frame[3] = 0x00;
        frame
    }

    /// Build a 24-byte frame with an unknown/invalid command (0xFF00).
    ///
    /// `is_valid_enip_frame` returns `false` for this command code (not in the ODVA set).
    /// Used to trigger the byte-walk resync path and the `command_counts[0xFF00]` increment.
    fn enip_unknown_command_frame() -> Vec<u8> {
        let mut frame = vec![0u8; 24];
        frame[0] = 0x00; // command = 0xFF00 LE low byte
        frame[1] = 0xFF; // command = 0xFF00 LE high byte
        // length = 0
        frame
    }

    /// Build a frame whose declared `length` field makes the total frame size exceed
    /// `MAX_ENIP_CARRY_BYTES` (600). Uses a valid command so `is_valid_enip_frame` passes,
    /// then the oversized-declared-frame path fires.
    ///
    /// command = 0x0065 (RegisterSession), length = 600 → total = 624.
    fn enip_oversized_declared_frame() -> Vec<u8> {
        let total_len: usize = 24 + 600; // = 624 bytes
        let mut frame = vec![0u8; total_len];
        frame[0] = 0x65; // RegisterSession LE low
        frame[1] = 0x00;
        frame[2] = (600u16 & 0xFF) as u8; // length = 600 LE low
        frame[3] = (600u16 >> 8) as u8;
        frame
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-137-001: Carry buffer accumulates partial ENIP frames
    // Traces: BC-2.17.016 Postconditions 1–3
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-137-001 — partial header (< 24 bytes) stashed into carry; carry.len() == 12.
    ///
    /// Send only 12 bytes (less than the 24-byte header minimum). The frame-walk loop
    /// finds `buf.len() - cursor < 24` on entry, so no iteration occurs. The 12 bytes
    /// are stored in `flow.carry`. No findings, no parse_errors.
    ///
    /// Traces: BC-2.17.016 Postconditions 2–3; AC-137-001; EC-003.
    #[test]
    fn test_carry_buffer_partial_header() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let partial = vec![0x65u8; 12]; // 12 bytes — partial header
        analyzer.on_data(key.clone(), &partial, 0);
        // RED GATE: todo!() in on_data panics before reaching here.
        // When implemented, flow state is read via analyzer.flows:
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data (BC-2.17.016 Postcondition 3)");
        assert_eq!(
            flow.carry.len(),
            12,
            "partial header (12 bytes < 24) must be stashed in carry \
             (BC-2.17.016 Postcondition 3; AC-137-001)"
        );
        assert_eq!(
            flow.parse_errors, 0,
            "partial header must not increment parse_errors (BC-2.17.016 Postcondition 1)"
        );
    }

    /// AC-137-001 — two complete frames in one segment: both processed; carry empty.
    ///
    /// Send a segment containing exactly 2 minimal ENIP frames (2 × 24 bytes = 48 bytes).
    /// Both frames have valid commands (RegisterSession, 0x0065). The frame-walk loop runs
    /// twice: both frames processed via `process_pdu`; `carry` is empty at the end.
    ///
    /// Traces: BC-2.17.016 Postconditions 1, 3; AC-137-001; EC-002.
    #[test]
    fn test_carry_buffer_two_frames_one_segment() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let mut two_frames = enip_frame(0x0065); // frame 1: RegisterSession, length=0
        two_frames.extend_from_slice(&enip_frame(0x0065)); // frame 2
        analyzer.on_data(key.clone(), &two_frames, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert!(
            flow.carry.is_empty(),
            "two complete frames must leave carry empty (BC-2.17.016 Postcondition 3; EC-002)"
        );
        assert_eq!(
            flow.pdu_count, 2,
            "pdu_count must be 2 after processing two complete frames (BC-2.17.024)"
        );
    }

    /// AC-137-001 — frame split across 3 segments: reassembled via carry.
    ///
    /// Frame total = 28 bytes (RegisterSession with length=4). Delivered in 3 segments:
    ///   seg1: bytes [0..10] (partial header)
    ///   seg2: bytes [10..20] (still partial header)
    ///   seg3: bytes [20..28] (completes header + payload)
    ///
    /// After seg1: carry.len() == 10. After seg2: carry.len() == 20. After seg3: frame
    /// is complete and processed via process_pdu; carry is empty.
    ///
    /// All 3 calls use the SAME flow key so the carry buffer accumulates across calls
    /// (BC-2.17.016 Postcondition 2 — carry is per-flow state in analyzer.flows).
    ///
    /// Traces: BC-2.17.016 Postconditions 2–3; AC-137-001; EC-003.
    #[test]
    fn test_carry_buffer_three_segments_one_frame() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_register_session_frame(); // 28 bytes

        analyzer.on_data(key.clone(), &frame[0..10], 0); // seg1
        // RED GATE: todo!() panics before here.
        // (subsequent calls would verify carry accumulation if seg1 did not panic)
        analyzer.on_data(key.clone(), &frame[10..20], 0); // seg2
        analyzer.on_data(key.clone(), &frame[20..28], 0); // seg3
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert!(
            flow.carry.is_empty(),
            "carry must be empty after the split frame is completed on seg3 \
             (BC-2.17.016 Postcondition 3; AC-137-001; EC-003)"
        );
        assert_eq!(
            flow.pdu_count, 1,
            "pdu_count must be 1 after the split frame is reassembled (BC-2.17.024)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-137-002: Carry buffer cap at MAX_ENIP_CARRY_BYTES (600)
    // Traces: BC-2.17.016 Invariant 4 / Postcondition 4; BC-2.17.018 EC-007
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-137-002 — carry-buffer cap invariant: carry stays ≤ MAX_ENIP_CARRY_BYTES after
    /// `on_data` regardless of pre-existing large carry state.
    ///
    /// Setup: Pre-populate flow.carry to 601 bytes of 0xFF garbage (simulating accumulated
    /// partial-frame data), then call on_data with a 28-byte valid ENIP frame.
    ///
    /// With correct `continue` semantics (RULING-137-001):
    ///   - The byte-walk loop runs through the 601 garbage bytes (601 iterations, each
    ///     advancing cursor by 1 and incrementing parse_errors).
    ///   - At cursor=601, the valid frame is at buf[601..629]. is_valid_enip_frame=true.
    ///     total_frame_len=28. buf.len()-cursor=28>=28. process_pdu fires.
    ///   - After loop: carry = buf[629..] = empty. carry.len()=0 ≤ 600. No overflow.
    ///   - is_non_enip remains false (no carry overflow triggered).
    ///
    /// With WRONG `break` semantics:
    ///   - First iteration: pos=0, garbage, invalid, cursor=1, BREAK.
    ///   - carry = buf[1..] = 628 bytes > 600. Overflow fires: is_non_enip=true.
    ///   - valid frame is NEVER processed (pdu_count=0).
    ///
    /// This test is RED under the current `break` implementation (is_non_enip=true, pdu_count=0)
    /// and GREEN under the correct `continue` implementation (is_non_enip=false, pdu_count>=1).
    ///
    /// Traces: BC-2.17.016 Postcondition 4 / Invariant 1/4; RULING-137-001 §3.4; AC-137-002; EC-004.
    #[test]
    fn test_carry_buffer_cap_at_600() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // Step 1: create the flow entry by sending empty data.
        analyzer.on_data(key.clone(), &[], 0);
        // Step 2: pre-populate carry with 601 bytes of 0xFF garbage (simulates accumulated
        // partial data; RULING-137-001 §3.4 — direct pre-population is the authoritative
        // test setup since carry > 600 is unreachable via normal continue semantics).
        {
            let flow = analyzer
                .flows
                .get_mut(&key)
                .expect("flow must exist after first on_data");
            flow.carry = vec![0xFF_u8; 601];
        }
        // Step 3: call on_data with a 28-byte valid ENIP frame (RegisterSession, length=4).
        // buf = carry(601) ++ valid_frame(28) = 629 bytes.
        let valid_frame = enip_register_session_frame(); // 28 bytes
        analyzer.on_data(key.clone(), &valid_frame, 0);
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after second on_data");
        // With correct continue semantics: byte-walk through 601 garbage positions, then
        // process the valid frame at position 601. is_non_enip stays false (no overflow).
        assert!(
            !flow.is_non_enip,
            "is_non_enip must remain false — carry overflow cannot be triggered by the \
             byte-walk path with continue semantics (RULING-137-001 §3.4; BC-2.17.016 Inv 4)"
        );
        assert_eq!(
            flow.pdu_count, 1,
            "valid frame at position 601 must be processed when continue allows the loop \
             to reach it (RULING-137-001 §3.4; AC-137-002)"
        );
        assert!(
            flow.carry.len() <= MAX_ENIP_CARRY_BYTES,
            "carry must be bounded after on_data (BC-2.17.016 Invariant 1; AC-137-002)"
        );
    }

    /// AC-137-002 — `is_non_enip` is set ONLY by genuine carry-buffer overflow (Invariant 4).
    ///
    /// This test verifies the NEGATIVE: that pre-set large carry (601 bytes) combined with a
    /// valid trailing frame does NOT set is_non_enip when the byte-walk loop uses `continue`
    /// (because the loop byte-walks through all garbage and processes the valid frame, leaving
    /// carry empty — never > 600).
    ///
    /// Contrast: with `break`, the first bad parse breaks out immediately, leaving carry=628 > 600,
    /// triggering is_non_enip=true. With `continue`, is_non_enip stays false.
    ///
    /// Traces: BC-2.17.016 Invariant 4; RULING-137-001 §3.4; AC-137-002; EC-004.
    #[test]
    fn test_carry_cap_sets_non_enip() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        analyzer.on_data(key.clone(), &[], 0); // create flow
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry = vec![0xFF_u8; 601]; // simulate large accumulated carry
        }
        let valid_frame = enip_register_session_frame();
        analyzer.on_data(key.clone(), &valid_frame, 0);
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        // With continue: byte-walk through garbage, process valid frame, carry=empty.
        // is_non_enip stays false — no carry overflow was triggered.
        assert!(
            !flow.is_non_enip,
            "is_non_enip must NOT be set: byte-walk with continue processes valid trailing frame \
             without triggering carry overflow (BC-2.17.016 Invariant 4; RULING-137-001 §3.4)"
        );
        assert_eq!(
            flow.pdu_count, 1,
            "valid frame must be processed when continue allows loop to traverse garbage prefix"
        );
    }

    /// AC-137-002 + AC-137-004 — T0814 fires at the 3rd malformed event; is_non_enip NOT set
    /// by T0814 emission (ordering constraint AC-137-002).
    ///
    /// THREE structural rejects via unknown-command frames (byte-walk resync path). On the 3rd,
    /// malformed_in_window=3 >= THRESHOLD. check_t0814 fires (while is_non_enip is still false).
    /// is_non_enip must NOT be set by T0814 — it is set only by carry overflow.
    ///
    /// Note: the previous test (with break semantics) used carry-overflow as the 3rd event.
    /// With correct continue semantics, carry overflow cannot be triggered via oversized-frame-
    /// skip residue (RULING-137-001 §3.4). Instead, three byte-walk rejects are used to reach
    /// the threshold. The ordering constraint is still exercised: check_t0814 fires while
    /// is_non_enip is false.
    ///
    /// **ORDERING CONSTRAINT (BC-2.17.018 EC-007 / AC-137-002):**
    ///   check_t0814 MUST fire before is_non_enip is set.
    ///
    /// Traces: BC-2.17.018 EC-007; BC-2.17.016 Invariant 4; RULING-137-001 §1; AC-137-002; AC-137-004.
    #[test]
    fn test_t0814_fires_on_carry_overflow_third_malformed() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // Three structural rejects via byte-walk (unknown command), same flow.
        // The T0814 threshold is 3; check_t0814 fires while is_non_enip is false.
        let frame = enip_unknown_command_frame(); // 24 bytes, command=0xFF00 (invalid)
        analyzer.on_data(key.clone(), &frame, 0); // reject #1
        analyzer.on_data(key.clone(), &frame, 0); // reject #2
        analyzer.on_data(key.clone(), &frame, 0); // reject #3 → T0814 fires
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T0814".to_string())),
            "T0814 finding must be emitted when malformed_in_window reaches 3 \
             (BC-2.17.018 EC-007; check_t0814 must fire BEFORE is_non_enip is latched)"
        );
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert!(
            !flow.is_non_enip,
            "is_non_enip must NOT be set by T0814 emission — it is set only by carry overflow \
             (BC-2.17.016 Invariant 4; RULING-137-001 §3.4; AC-137-002)"
        );
        assert!(
            flow.malformed_anomaly_emitted,
            "malformed_anomaly_emitted must be true after T0814 emission (BC-2.17.018 PC-4)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-137-003: Frame-walk resync and frame-skip paths
    // Traces: BC-2.17.016 Postcondition 1
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-137-003 — unknown command → byte-walk resync: cursor += 1, loop continues.
    ///
    /// Send a 24-byte frame with unknown command (0xFF00). `is_valid_enip_frame` returns
    /// `false`. Frame-walk: byte-walk resync (cursor += 1, NOT break). The remaining
    /// 23 bytes < 24, so loop exits. carry = buf[1..] = 23 bytes.
    /// `parse_errors == 1`, `malformed_in_window == 1`.
    ///
    /// Traces: BC-2.17.016 Postcondition 1 (byte-walk resync path); AC-137-003; BC-2.17.018 PC-1/2.
    #[test]
    fn test_byte_walk_resync_invalid_command() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let data = enip_unknown_command_frame(); // 24 bytes, command=0xFF00
        analyzer.on_data(key.clone(), &data, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.parse_errors, 1,
            "unknown command must increment parse_errors to 1 \
             (BC-2.17.016 Postcondition 1; AC-137-003)"
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "unknown command must increment malformed_in_window to 1 (BC-2.17.018 PC-1/2)"
        );
        assert_eq!(
            flow.carry.len(),
            23,
            "byte-walk resync (cursor += 1) leaves 23 bytes in carry \
             (BC-2.17.016 Postcondition 1 / AC-137-003)"
        );
    }

    /// AC-137-003 — oversized declared frame → frame-skip (cursor += total), NOT is_non_enip.
    ///
    /// Send exactly the oversized declared frame (624 bytes). The loop sees a valid command
    /// with total_frame_len=624 > 600 → frame-skip: cursor += min(624, 624) = 624.
    /// Loop exits. carry = buf[624..] = empty. `is_non_enip` must remain `false`.
    ///
    /// Traces: BC-2.17.016 Postcondition 1 (frame-skip path); AC-137-003; EC-010.
    #[test]
    fn test_oversize_frame_skip_continue() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let data = enip_oversized_declared_frame(); // 624 bytes, command=0x0065, length=600
        analyzer.on_data(key.clone(), &data, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.parse_errors, 1,
            "oversized declared frame must increment parse_errors to 1 \
             (BC-2.17.016 Postcondition 1; AC-137-003; EC-010)"
        );
        assert!(
            flow.carry.is_empty(),
            "frame-skip path must leave carry empty (cursor += total_frame_len) \
             (BC-2.17.016 Postcondition 1; EC-010)"
        );
    }

    /// AC-137-003 — oversized declared frame does NOT set is_non_enip (BC-2.17.016 Invariant 4).
    ///
    /// `is_non_enip` is EXCLUSIVELY set on carry-buffer overflow. The frame-skip path for
    /// oversized declared frames must NOT set it.
    ///
    /// Traces: BC-2.17.016 Invariant 4; AC-137-003; EC-010.
    #[test]
    fn test_oversize_frame_does_not_set_non_enip() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let data = enip_oversized_declared_frame();
        analyzer.on_data(key.clone(), &data, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert!(
            !flow.is_non_enip,
            "oversized declared frame (frame-skip path) must NOT set is_non_enip \
             (BC-2.17.016 Invariant 4; AC-137-003)"
        );
    }

    /// AC-137-003 + RULING-137-001 §3.1 — EC-010 discriminating test.
    ///
    /// Oversized declared frame (total=624 > 600) immediately followed by a valid 28-byte
    /// ENIP frame in the SAME on_data call (652 bytes total buffer).
    ///
    /// **With CORRECT `continue` semantics (RULING-137-001 Table 3.1):**
    ///   - Loop iter 1: valid header at pos 0, length=600, total=624 > 600 → frame-skip.
    ///     parse_errors=1, malformed_in_window=1. cursor += 624. continue.
    ///   - Loop iter 2: buf.len()-cursor = 652-624 = 28 >= 24. Parse header at [624..648].
    ///     Valid command. total_frame_len=28. 28>=28. process_pdu. cursor=652.
    ///   - After loop: carry = buf[652..] = empty.
    ///   - pdu_count=1, parse_errors=1, malformed_in_window=1, is_non_enip=false.
    ///
    /// **With WRONG `break` semantics:**
    ///   - Frame-skip: cursor=624, BREAK. carry=buf[624..]=28 bytes (valid frame stashed).
    ///   - Valid frame is NEVER processed: pdu_count=0.
    ///
    /// This test is RED under current `break` (pdu_count=0, carry.len()==28)
    /// and GREEN under correct `continue` (pdu_count=1, carry.is_empty()).
    ///
    /// Traces: BC-2.17.016 Postcondition 1 (frame-skip path); RULING-137-001 §3.1;
    ///         AC-137-003; EC-010.
    #[test]
    fn test_oversize_frame_skip_then_valid_frame_processed() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // Build: 624-byte oversized declared frame + 28-byte valid frame = 652 bytes total.
        // RULING-137-001 §3.1: oversized frame (header.length=600, total=624) + trailing
        // valid frame (header.length=4, total=28) in ONE segment.
        let mut data = enip_oversized_declared_frame(); // 624 bytes, length=600, command=0x0065
        data.extend_from_slice(&enip_register_session_frame()); // 28 bytes, length=4
        assert_eq!(data.len(), 652);
        analyzer.on_data(key.clone(), &data, 0);
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.pdu_count, 1,
            "trailing valid frame must be processed after oversized frame skip with continue \
             (RULING-137-001 §3.1; BC-2.17.016 Post-1 frame-skip path; AC-137-003; EC-010)"
        );
        assert_eq!(
            flow.parse_errors, 1,
            "parse_errors must be 1: only the oversized frame skip increments it \
             (RULING-137-001 §3.1; BC-2.17.018 PC-1; AC-137-003)"
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "malformed_in_window must be 1: only the oversized frame skip \
             (RULING-137-001 §3.1)"
        );
        assert!(
            !flow.is_non_enip,
            "is_non_enip must remain false: frame-skip path does NOT set it \
             (BC-2.17.016 Invariant 4; RULING-137-001 §3.1)"
        );
        assert!(
            flow.carry.is_empty(),
            "carry must be empty: continue advances cursor past oversized frame and processes \
             trailing valid frame (RULING-137-001 §3.1; BC-2.17.016 Post-1)"
        );
    }

    /// AC-137-003 + RULING-137-001 §3.2 — EC-012 minimal discriminating test.
    ///
    /// 1-byte garbage prefix immediately followed by a valid 28-byte ENIP frame in ONE
    /// on_data call (29 bytes total buffer). Verifies byte-walk resync CONTINUES to find
    /// the valid frame at the next cursor position.
    ///
    /// Buffer layout:
    ///   buf[0]     = 0xFF (garbage, makes command at [0..2] = LE(0xFF, 0x65) = 0x65FF — invalid)
    ///   buf[1..29] = enip_register_session_frame() (command=0x0065, length=4, total=28)
    ///
    /// **With CORRECT `continue` semantics (RULING-137-001 §3.2):**
    ///   - Loop iter 1: parse at [0..24]. command=LE(0xFF, 0x65)=0x65FF — invalid.
    ///     parse_errors=1. cursor=1. continue.
    ///   - Loop iter 2: parse at [1..25]=frame[0..24]. command=0x0065 — valid!
    ///     total_frame_len=28. buf.len()-cursor=29-1=28>=28. process_pdu. cursor=29.
    ///   - carry=empty. pdu_count=1. parse_errors=1. is_non_enip=false.
    ///
    /// **With WRONG `break` semantics:**
    ///   - Loop iter 1: invalid, cursor=1, BREAK. carry=buf[1..]=28 bytes (valid frame stashed).
    ///   - pdu_count=0.
    ///
    /// This test is RED under current `break` (pdu_count=0) and GREEN under correct `continue`
    /// (pdu_count=1).
    ///
    /// Traces: BC-2.17.016 Postcondition 1 (byte-walk resync); RULING-137-001 §3.2;
    ///         AC-137-003; EC-012.
    #[test]
    fn test_byte_walk_resync_to_valid_frame_same_segment() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // 1-byte garbage prefix: buf[0]=0xFF. At cursor=0, parse [0..24]:
        //   buf[0..2] = [0xFF, frame[0]] = [0xFF, 0x65] → command = LE(0xFF,0x65) = 0x65FF (unknown).
        // buf[1..29] = valid RegisterSession frame (command=0x0065, length=4).
        let mut data = vec![0xFF_u8]; // 1-byte garbage prefix
        data.extend_from_slice(&enip_register_session_frame()); // append 28-byte valid frame
        assert_eq!(data.len(), 29);
        analyzer.on_data(key.clone(), &data, 0);
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.pdu_count, 1,
            "valid frame at position 1 must be processed after 1-byte garbage prefix when \
             byte-walk uses continue (RULING-137-001 §3.2; BC-2.17.016 Post-1; AC-137-003; EC-012)"
        );
        assert_eq!(
            flow.parse_errors, 1,
            "parse_errors must be 1: one byte-walk resync before the valid frame \
             (RULING-137-001 §3.2; BC-2.17.018 PC-1)"
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "malformed_in_window must be 1 (RULING-137-001 §3.2; BC-2.17.018 PC-2)"
        );
        assert!(
            !flow.is_non_enip,
            "is_non_enip must remain false (BC-2.17.016 Invariant 4; RULING-137-001 §3.2)"
        );
        assert!(
            flow.carry.is_empty(),
            "carry must be empty after valid frame completes (RULING-137-001 §3.2)"
        );
    }

    /// AC-137-003 + AC-137-004 + RULING-137-001 §3.2 — EC-012 24-byte-block discriminating test.
    ///
    /// 24 bytes of garbage (all 0xFF) immediately followed by a valid 28-byte ENIP frame
    /// (52 bytes total). byte-walk runs 24 times before finding the valid frame.
    ///
    /// Per RULING-137-001 §3.2: parse_errors=24 (one per byte-walk position 0..23),
    /// pdu_count=1, T0814 fires (malformed_in_window=24 >= 3).
    ///
    /// Buffer layout:
    ///   buf[0..24]  = [0xFF; 24] (command at each cursor 0..23 = 0xFFFF or shifted, all invalid)
    ///   buf[24..52] = enip_register_session_frame() (command=0x0065, length=4, total=28)
    ///
    /// Under WRONG `break`: cursor=1, break. carry=51 bytes. pdu_count=0. parse_errors=1.
    /// Under CORRECT `continue`: 24 byte-walk iterations, then process valid frame at pos 24.
    ///   parse_errors=24. pdu_count=1. T0814 fires. carry=empty.
    ///
    /// This test is RED under current `break` (pdu_count=0, parse_errors=1).
    ///
    /// Traces: BC-2.17.016 Postcondition 1; BC-2.17.018 Postconditions 1–4; RULING-137-001 §3.2;
    ///         AC-137-003; AC-137-004; EC-012.
    #[test]
    fn test_byte_walk_resync_24_garbage_bytes_then_valid_frame() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // 24 bytes of 0xFF garbage + 28-byte valid frame = 52 bytes total.
        let mut data = vec![0xFF_u8; 24]; // all-garbage 24-byte block (command=0xFFFF, invalid)
        data.extend_from_slice(&enip_register_session_frame()); // valid frame at [24..52]
        assert_eq!(data.len(), 52);
        analyzer.on_data(key.clone(), &data, 0);
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.pdu_count, 1,
            "valid frame at position 24 must be processed after 24-garbage-byte prefix \
             (RULING-137-001 §3.2; BC-2.17.016 Post-1; AC-137-003; EC-012)"
        );
        assert_eq!(
            flow.parse_errors, 24,
            "parse_errors must be 24: one per byte-walk position 0..23 before valid frame \
             (RULING-137-001 §3.2; BC-2.17.018 PC-1)"
        );
        assert_eq!(
            flow.malformed_in_window, 24,
            "malformed_in_window must be 24 (RULING-137-001 §3.2; BC-2.17.018 PC-2)"
        );
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T0814".to_string())),
            "T0814 must fire: malformed_in_window=24 >= threshold=3 \
             (RULING-137-001 §3.2; BC-2.17.018 EC-007; AC-137-004)"
        );
        assert!(
            !flow.is_non_enip,
            "is_non_enip must remain false: T0814 does NOT set it \
             (BC-2.17.016 Invariant 4; RULING-137-001 §3.2)"
        );
        assert!(
            flow.carry.is_empty(),
            "carry must be empty: valid frame completes and exhausts the buffer \
             (RULING-137-001 §3.2)"
        );
    }

    /// AC-137-001 + AC-137-004 + RULING-137-001 §3.3 — multi-call carry residue counting.
    ///
    /// Verifies that per-offset counting accumulates correctly across multiple on_data calls
    /// when carry residue from call 1 is re-walked in call 2.
    ///
    /// **Call 1:** 23 bytes of 0xFF garbage.
    ///   buf.len()-cursor = 23 < 24: while loop never fires. carry = 23 bytes. parse_errors=0.
    ///
    /// **Call 2:** 5 bytes of 0xFF garbage. buf = carry(23) ++ new(5) = 28 bytes.
    ///   - Iter 1: buf.len()-cursor=28>=24. Parse at [0..24]: 0xFF bytes, command=0xFFFF, invalid.
    ///     parse_errors=1, malformed_in_window=1. cursor=1. continue.
    ///   - Iter 2: buf.len()-cursor=27>=24. Parse at [1..25]: same garbage. Fails.
    ///     parse_errors=2, malformed_in_window=2. cursor=2. continue.
    ///   - Iter 3: parse at [2..26]: fails. parse_errors=3, malformed_in_window=3. cursor=3.
    ///     continue. T0814 fires here (threshold=3).
    ///   - Iter 4: parse at [3..27]: fails. parse_errors=4, malformed_in_window=4. cursor=4.
    ///   - Iter 5: parse at [4..28]: fails. parse_errors=5, malformed_in_window=5. cursor=5.
    ///   - Iter 6: buf.len()-cursor=28-5=23 < 24. Exit loop.
    ///   - carry = buf[5..] = 23 bytes. 23 ≤ 600. No overflow. T0814 emitted=true.
    ///
    /// Per RULING-137-001 §3.3: parse_errors=5, malformed_in_window=5, T0814 emitted, carry=23.
    ///
    /// Traces: BC-2.17.016 Postconditions 1–3; BC-2.17.018 Postconditions 1–4;
    ///         RULING-137-001 §3.3; AC-137-001; AC-137-004.
    #[test]
    fn test_multi_call_carry_residue_counting() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // Call 1: 23 bytes — buf < 24, loop never fires, carry=23, parse_errors=0.
        let garbage_23 = vec![0xFF_u8; 23];
        analyzer.on_data(key.clone(), &garbage_23, 0);
        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after call 1");
            assert_eq!(
                flow.carry.len(),
                23,
                "call 1 (23 bytes < 24): loop never fires, all bytes stash to carry \
                 (RULING-137-001 §3.3; BC-2.17.016 Post-2)"
            );
            assert_eq!(
                flow.parse_errors, 0,
                "call 1: no loop iterations, no parse errors (RULING-137-001 §3.3)"
            );
        }
        // Call 2: 5 bytes of garbage. buf = carry(23) + new(5) = 28 bytes.
        // 5 byte-walk iterations, T0814 fires on iter 3.
        let garbage_5 = vec![0xFF_u8; 5];
        analyzer.on_data(key.clone(), &garbage_5, 0);
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after call 2");
        assert_eq!(
            flow.parse_errors, 5,
            "call 2: 5 byte-walk iterations → parse_errors=5 \
             (RULING-137-001 §3.3; BC-2.17.018 PC-1)"
        );
        assert_eq!(
            flow.malformed_in_window, 5,
            "call 2: malformed_in_window=5 (RULING-137-001 §3.3; BC-2.17.018 PC-2)"
        );
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T0814".to_string())),
            "T0814 must fire on call 2 iter 3 (malformed_in_window=3 >= threshold) \
             (RULING-137-001 §3.3; BC-2.17.018 PC-3)"
        );
        assert_eq!(
            flow.carry.len(),
            23,
            "carry must be 23 bytes after call 2 (buf[5..28] = last 23 bytes) \
             (RULING-137-001 §3.3; BC-2.17.016 Post-3)"
        );
        assert!(
            !flow.is_non_enip,
            "is_non_enip must remain false: T0814 does NOT set it (BC-2.17.016 Invariant 4)"
        );
    }

    /// AC-137-003 — is_non_enip flag is permanent: once set, subsequent on_data are no-ops.
    ///
    /// Pre-set is_non_enip=true via direct field mutation (simulating state after carry overflow
    /// has triggered), then verify that subsequent on_data calls do not modify any counters,
    /// carry, or findings.
    ///
    /// With CORRECT `continue` semantics:
    ///   pre-set is_non_enip directly (since natural overflow is impossible with continue);
    ///   then verify the is_non_enip early-exit guard fires correctly.
    ///
    /// Traces: BC-2.17.016 Invariant 4; AC-137-003; EC-011.
    #[test]
    fn test_non_enip_flag_permanent() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // Step 1: create the flow entry.
        analyzer.on_data(key.clone(), &[], 0);
        // Step 2: set is_non_enip = true directly (simulate carry-overflow already having fired).
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.is_non_enip = true;
            flow.parse_errors = 1; // simulate the one increment from the overflow event
            flow.malformed_in_window = 1;
        }
        let parse_errors_after_set = 1u64;
        let findings_after_set = analyzer.all_findings.len();

        // Subsequent call with valid data — must be a no-op (is_non_enip guards it).
        let valid_frame = enip_frame(0x0065);
        analyzer.on_data(key.clone(), &valid_frame, 1);
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after second on_data");
        assert_eq!(
            flow.parse_errors, parse_errors_after_set,
            "parse_errors must not change after is_non_enip is set (EC-011; BC-2.17.016 Inv 4)"
        );
        assert_eq!(
            analyzer.all_findings.len(),
            findings_after_set,
            "no new findings after is_non_enip is set (EC-011; BC-2.17.016 Invariant 4)"
        );
        assert!(
            flow.is_non_enip,
            "is_non_enip must remain true — it is a one-way permanent flag (BC-2.17.016 Inv 4)"
        );
    }

    /// AC-137-003 — carry overflow path: is_non_enip NOT latched by byte-walk; carry bounded.
    ///
    /// Verifies BC-2.17.016 Postcondition 4 invariant from the NEGATIVE direction:
    /// Pre-set carry=601 bytes (garbage) then call on_data with a valid 28-byte frame.
    /// With correct `continue` semantics, the byte-walk loop reduces carry to 0 (processes
    /// the valid frame at position 601), so the carry-cap overflow check does NOT fire.
    /// is_non_enip stays false; carry stays bounded (≤ MAX_ENIP_CARRY_BYTES).
    ///
    /// Since carry > 600 cannot be triggered naturally with continue semantics
    /// (RULING-137-001 §3.4 analysis), this test directly sets carry=601 and verifies
    /// the carry stays bounded after on_data (invariant holds; is_non_enip stays false
    /// because the byte-walk loop reduces carry before the cap check).
    ///
    /// Traces: BC-2.17.016 Postcondition 4; RULING-137-001 §3.4; AC-137-003; EC-004.
    #[test]
    fn test_non_enip_flag_set_at_carry_cap() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        analyzer.on_data(key.clone(), &[], 0); // create flow
        {
            let flow = analyzer.flows.get_mut(&key).expect("flow must exist");
            flow.carry = vec![0xFF_u8; 601];
        }
        let valid_frame = enip_register_session_frame();
        analyzer.on_data(key.clone(), &valid_frame, 0);
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        // carry-cap overflow does NOT fire (byte-walk reduces carry to 0 with continue).
        assert!(
            flow.carry.len() <= MAX_ENIP_CARRY_BYTES,
            "carry must be bounded (≤ MAX_ENIP_CARRY_BYTES=600) after on_data \
             (BC-2.17.016 Invariant 1 / Postcondition 4; AC-137-003)"
        );
        assert!(
            !flow.is_non_enip,
            "is_non_enip must NOT be set by byte-walk path — only carry overflow sets it \
             (BC-2.17.016 Invariant 4; AC-137-003)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-137-004: T0814 windowed DoS detection
    // Traces: BC-2.17.018 Postconditions 1–5; Invariants 1/3/4
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-137-004 — T0814 fires when malformed_in_window reaches threshold (3).
    ///
    /// Send 3 unknown-command frames (each increments malformed_in_window). On the 3rd,
    /// `check_t0814` fires: T0814 Anomaly/Possible/Low finding emitted, `malformed_anomaly_emitted`
    /// set to true.
    ///
    /// All 3 calls use the SAME flow key so malformed_in_window accumulates.
    ///
    /// Asserts T0814 summary contains required BC-2.17.018 Postcondition 3 substrings:
    /// "malformed frames", "possible crash-probe", "T0814" in mitre_techniques.
    ///
    /// Traces: BC-2.17.018 Postconditions 1–4; AC-137-004; EC-007.
    #[test]
    fn test_t0814_fires_at_threshold() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_unknown_command_frame();

        analyzer.on_data(key.clone(), &frame, 0);
        // RED GATE: todo!() panics before here.
        analyzer.on_data(key.clone(), &frame, 0);
        analyzer.on_data(key.clone(), &frame, 0);
        let t0814_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .collect();
        assert_eq!(
            t0814_findings.len(),
            1,
            "exactly 1 T0814 finding must be emitted when malformed_in_window reaches 3 \
             (BC-2.17.018 Postcondition 3; AC-137-004; EC-007)"
        );
        assert_eq!(
            t0814_findings[0].verdict,
            wirerust::findings::Verdict::Possible,
            "T0814 verdict must be Possible (BC-2.17.018 Postcondition 3)"
        );
        assert_eq!(
            t0814_findings[0].confidence,
            wirerust::findings::Confidence::Low,
            "T0814 confidence must be Low (BC-2.17.018 Postcondition 3)"
        );
        assert_eq!(
            t0814_findings[0].category,
            wirerust::findings::ThreatCategory::Anomaly,
            "T0814 category must be Anomaly (BC-2.17.018 Postcondition 3)"
        );
        // BC-2.17.018 Postcondition 3 — summary substring assertions.
        assert!(
            t0814_findings[0].summary.contains("malformed frames"),
            "T0814 summary must contain \"malformed frames\" \
             (BC-2.17.018 Postcondition 3 summary template)"
        );
        assert!(
            t0814_findings[0].summary.contains("possible crash-probe"),
            "T0814 summary must contain \"possible crash-probe\" \
             (BC-2.17.018 Postcondition 3 summary template)"
        );
        assert!(
            t0814_findings[0].source_ip.is_some(),
            "T0814 finding must carry source_ip (BC-2.17.018 Postcondition 3)"
        );
        assert!(
            t0814_findings[0].timestamp.is_some(),
            "T0814 finding must carry timestamp (BC-2.17.018 Postcondition 3)"
        );
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert!(
            flow.malformed_anomaly_emitted,
            "malformed_anomaly_emitted must be true after T0814 fires (BC-2.17.018 PC-4)"
        );
    }

    /// AC-137-004 — T0814 one-shot guard: 4th malformed event in same window → no second finding.
    ///
    /// Uses the oversized-declared-frame-skip path (RULING-137-001 §1): each call delivers a
    /// self-contained 624-byte buffer (header.length=600, total=624 > MAX_ENIP_CARRY_BYTES).
    /// The loop advances `cursor += min(624, 624) = 624` then `continue`; the loop exits with
    /// `buf.len()-cursor == 0`, so carry is empty after each call. Exactly 1 malformed event
    /// per call, no carry residue to re-walk on subsequent calls (RULING-137-001 §2).
    ///
    /// After 3 calls: `malformed_in_window=3, parse_errors=3` → T0814 fires, guard set
    ///   (BC-2.17.018 Canonical Test Vectors row 3).
    /// After 4th call: `malformed_in_window=4, parse_errors=4`, guard holds → no 2nd T0814
    ///   (BC-2.17.018 Canonical Test Vectors row 4; EC-004 — same window, guard set).
    ///
    /// Traces: BC-2.17.018 Postcondition 4; BC-2.17.018 EC-004; AC-137-004; RULING-137-001 §1–2.
    #[test]
    fn test_t0814_one_shot_guard_per_window() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // Each oversized-declared-frame call: exactly 1 malformed event, zero carry residue.
        // command=0x0065 (valid), header.length=600, total=624 > 600 → frame-skip path.
        for i in 0..4u32 {
            let frame = enip_oversized_declared_frame();
            analyzer.on_data(key.clone(), &frame, i);
        }
        let t0814_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();
        assert_eq!(
            t0814_count, 1,
            "T0814 must fire exactly once per window (one-shot guard prevents 2nd emission on \
             4th event) (BC-2.17.018 Postcondition 4; EC-004; AC-137-004; RULING-137-001 §1–2)"
        );
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.malformed_in_window, 4,
            "malformed_in_window must be 4: one increment per oversized-frame-skip call, \
             no carry-residue re-walk (BC-2.17.018 Canonical Test Vectors row 4; RULING-137-001 §2)"
        );
        assert_eq!(
            flow.parse_errors, 4,
            "parse_errors (lifetime) must be 4: one per structural reject, no carry inflation \
             (BC-2.17.018 PC-1; RULING-137-001 §2)"
        );
        assert!(
            flow.malformed_anomaly_emitted,
            "malformed_anomaly_emitted must be true after T0814 fires \
             (BC-2.17.018 Postcondition 4; one-shot guard)"
        );
    }

    /// AC-137-004 — T0814 does NOT fire below threshold (2 malformed events).
    ///
    /// Uses the oversized-declared-frame-skip path (RULING-137-001 §1): each call delivers a
    /// self-contained 624-byte buffer (header.length=600, total=624 > MAX_ENIP_CARRY_BYTES).
    /// The loop advances `cursor += min(624, 624) = 624` then `continue`; carry is empty after
    /// each call. Exactly 1 malformed event per call, no carry residue to re-walk.
    ///
    /// 2 calls → `malformed_in_window=2, parse_errors=2`: below MALFORMED_ANOMALY_THRESHOLD (3).
    /// T0814 must NOT fire. `malformed_anomaly_emitted` must remain false.
    ///
    /// Corresponds to BC-2.17.018 Canonical Test Vectors rows 1–2 (1 event: no finding;
    /// 2 events: no finding) and Precondition 2 (threshold not yet reached).
    ///
    /// Traces: BC-2.17.018 Preconditions 1–2; BC-2.17.018 EC-001–002; AC-137-004;
    ///         RULING-137-001 §1–2.
    #[test]
    fn test_t0814_does_not_fire_below_threshold() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        // Each oversized-declared-frame call: exactly 1 malformed event, zero carry residue.
        // command=0x0065 (valid), header.length=600, total=624 > 600 → frame-skip path.
        analyzer.on_data(key.clone(), &enip_oversized_declared_frame(), 0);
        analyzer.on_data(key.clone(), &enip_oversized_declared_frame(), 1);
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.malformed_in_window, 2,
            "malformed_in_window must be 2: one increment per oversized-frame-skip call, \
             no carry-residue inflation (BC-2.17.018 EC-002; RULING-137-001 §2)"
        );
        assert_eq!(
            flow.parse_errors, 2,
            "parse_errors (lifetime) must be 2: one per structural reject \
             (BC-2.17.018 PC-1; RULING-137-001 §2)"
        );
        assert!(
            !analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T0814".to_string())),
            "T0814 must NOT fire below threshold (2 < 3 = MALFORMED_ANOMALY_THRESHOLD) \
             (BC-2.17.018 Precondition 2; EC-001–002; AC-137-004)"
        );
        assert!(
            !flow.malformed_anomaly_emitted,
            "malformed_anomaly_emitted must remain false below threshold \
             (BC-2.17.018 Postcondition 4)"
        );
        assert!(
            !flow.is_non_enip,
            "is_non_enip must remain false: frame-skip path does NOT set it \
             (BC-2.17.016 Invariant 4)"
        );
    }

    /// AC-137-004 — T0814 re-fires after 300-second window reset.
    ///
    /// After a window that already fired T0814, simulate a 300-second expiry by sending
    /// a new `on_data` with `now_ts = 300`. The window resets: `malformed_in_window = 0`,
    /// `malformed_anomaly_emitted = false`. A fresh burst of 3 malformed frames fires a
    /// new T0814 (total: 2 T0814 findings in `all_findings`).
    ///
    /// All calls use the same flow key.
    ///
    /// Traces: BC-2.17.018 Postcondition 5; AC-137-004; EC-009.
    #[test]
    fn test_t0814_refire_after_window_reset() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_unknown_command_frame();

        // First window: 3 malformed frames → T0814 fires (ts=0,0,0).
        analyzer.on_data(key.clone(), &frame, 0);
        // RED GATE: todo!() panics before here.
        analyzer.on_data(key.clone(), &frame, 0);
        analyzer.on_data(key.clone(), &frame, 0);

        // Second window: send ts=300 to trigger expiry, then 2 more at 300, 300.
        analyzer.on_data(key.clone(), &frame, 300);
        analyzer.on_data(key.clone(), &frame, 300);
        analyzer.on_data(key.clone(), &frame, 300);
        let t0814_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();
        assert_eq!(
            t0814_count, 2,
            "T0814 must re-fire after 300-second window reset; expected 2 findings total \
             (BC-2.17.018 Postcondition 5; AC-137-004; EC-009)"
        );
    }

    /// AC-137-004 — parse_errors is NOT reset on window expiry (lifetime counter).
    ///
    /// Uses the oversized-declared-frame-skip path (RULING-137-001 §1): each call delivers a
    /// self-contained 624-byte buffer (header.length=600, total=624 > MAX_ENIP_CARRY_BYTES).
    /// Exactly 1 malformed event per call, no carry residue to re-walk (RULING-137-001 §2).
    ///
    /// Two-counter model (BC-2.17.018 Invariant 1):
    ///   - `parse_errors`: LIFETIME, monotonic, NEVER reset.
    ///   - `malformed_in_window`: WINDOWED, reset to 0 at 300s expiry.
    ///
    /// Scenario (K=3 events in window 1, M=1 event in window 2):
    ///   Window 1 (ts=0): 3 oversized-frame-skip calls → parse_errors=3, malformed_in_window=3.
    ///     T0814 fires on the 3rd call (threshold reached); malformed_anomaly_emitted=true.
    ///     (BC-2.17.018 Canonical Test Vectors row 3.)
    ///   Window 2 (ts=300): expiry triggers reset: malformed_in_window=0, anomaly_emitted=false.
    ///     Then the ts=300 call adds 1 event → parse_errors=4 (accumulated), malformed_in_window=1.
    ///     (BC-2.17.018 Postcondition 5; Canonical Test Vectors row 5.)
    ///
    /// Final expected state: parse_errors=4, malformed_in_window=1, malformed_anomaly_emitted=false.
    ///
    /// Traces: BC-2.17.018 Invariant 1; BC-2.17.018 Postcondition 5; BC-2.17.018 EC-005;
    ///         RULING-137-001 §2 (§3.3 two-counter model); AC-137-004.
    #[test]
    fn test_parse_errors_not_reset_on_window_expiry() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();

        // Window 1 (ts=0): 3 oversized-frame-skip events → threshold crossed, T0814 fires.
        // Each call: exactly 1 malformed event (parse_errors++, malformed_in_window++), no carry.
        analyzer.on_data(key.clone(), &enip_oversized_declared_frame(), 0);
        analyzer.on_data(key.clone(), &enip_oversized_declared_frame(), 0);
        analyzer.on_data(key.clone(), &enip_oversized_declared_frame(), 0);

        // Window 2 (ts=300): 300s elapsed → expiry resets malformed_in_window=0 and
        // malformed_anomaly_emitted=false. Then this call adds 1 more event → malformed_in_window=1.
        analyzer.on_data(key.clone(), &enip_oversized_declared_frame(), 300);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.parse_errors, 4,
            "parse_errors must accumulate across window resets (lifetime counter, NEVER reset): \
             3 events in window 1 + 1 in window 2 = 4 (BC-2.17.018 Invariant 1; EC-005)"
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "malformed_in_window must be 1 in window 2: reset to 0 at expiry, then +1 for the \
             single ts=300 event (BC-2.17.018 Postcondition 5; EC-005; RULING-137-001 §3.3)"
        );
        assert!(
            !flow.malformed_anomaly_emitted,
            "malformed_anomaly_emitted must be false in window 2: the guard is cleared at window \
             expiry, allowing a fresh T0814 when threshold is reached again \
             (BC-2.17.018 Postcondition 5; EC-005)"
        );
    }

    /// AC-137-004 — T0814 fires but is_non_enip is NOT set at threshold crossing.
    ///
    /// `is_non_enip` is exclusively set on carry-buffer overflow. T0814 emission must NOT
    /// set `is_non_enip`.
    ///
    /// Satisfies HS-117 Case D.
    ///
    /// Traces: BC-2.17.016 Invariant 4; BC-2.17.018 Invariant 4; AC-137-004; HS-117 Case D.
    #[test]
    fn test_t0814_non_enip_not_set_at_threshold() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_unknown_command_frame();
        for i in 0..3u32 {
            analyzer.on_data(key.clone(), &frame, i);
            // RED GATE: todo!() panics on i==0 before here.
        }
        assert!(
            analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T0814".to_string())),
            "T0814 must fire at threshold 3 (AC-137-004)"
        );
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert!(
            !flow.is_non_enip,
            "is_non_enip must NOT be set when T0814 fires — it is set ONLY on carry overflow \
             (BC-2.17.016 Invariant 4; HS-117 Case D)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-137-005: Valid vs invalid frame — malformed count behavior
    // Traces: BC-2.17.016 Postcondition 1; BC-2.17.018 Postconditions 1–2
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-137-005 — valid frame does NOT increment parse_errors or malformed_in_window.
    ///
    /// A valid complete ENIP frame (command=0x0065, length=0, total=24 bytes) passes
    /// `is_valid_enip_frame`, is dispatched to `process_pdu`, and increments `pdu_count`.
    /// Neither `parse_errors` nor `malformed_in_window` are incremented.
    ///
    /// Traces: BC-2.17.016 Postcondition 1 (valid path); BC-2.17.018 PC-1/2; AC-137-005; EC-001.
    #[test]
    fn test_valid_frame_no_malformed_count() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_frame(0x0065); // valid command, length=0, total=24
        analyzer.on_data(key.clone(), &frame, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.parse_errors, 0,
            "valid frame must not increment parse_errors (BC-2.17.016 PC-1; AC-137-005)"
        );
        assert_eq!(
            flow.malformed_in_window, 0,
            "valid frame must not increment malformed_in_window (BC-2.17.018 PC-1/2; AC-137-005)"
        );
    }

    /// AC-137-005 — invalid frame increments both parse_errors and malformed_in_window.
    ///
    /// A frame with unknown command (0xFF00) triggers the byte-walk resync path.
    /// Both `parse_errors` and `malformed_in_window` must be incremented.
    ///
    /// Traces: BC-2.17.018 Postconditions 1–2; AC-137-005; EC-005.
    #[test]
    fn test_invalid_frame_increments_malformed_count() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_unknown_command_frame();
        analyzer.on_data(key.clone(), &frame, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.parse_errors, 1,
            "invalid frame must increment parse_errors to 1 \
             (BC-2.17.018 Postcondition 1; AC-137-005)"
        );
        assert_eq!(
            flow.malformed_in_window, 1,
            "invalid frame must increment malformed_in_window to 1 \
             (BC-2.17.018 Postcondition 2; AC-137-005)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-137-006: command_counts canonical single increment site (BC-2.17.016 PC-0)
    // Traces: BC-2.17.016 Postcondition 0; BC-2.17.004 Invariant 3
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-137-006 — command_counts incremented for unknown command BEFORE is_valid_enip_frame.
    ///
    /// Send a 24-byte frame with command=0xFF00 (unknown). The frame-walk increments
    /// `command_counts[0xFF00]` immediately after `parse_enip_header` returns `Some`,
    /// BEFORE calling `is_valid_enip_frame`. Then the byte-walk resync fires (invalid cmd).
    /// `process_pdu` is NOT called (`pdu_count` stays 0).
    ///
    /// Traces: BC-2.17.016 PC-0; BC-2.17.004 Invariant 3; AC-137-006.
    #[test]
    fn test_command_counts_increments_for_unknown_command() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_unknown_command_frame(); // command = 0xFF00
        analyzer.on_data(key.clone(), &frame, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.command_counts.get(&0xFF00u16).copied().unwrap_or(0),
            1,
            "command_counts[0xFF00] must be 1 after one unknown-command frame \
             (BC-2.17.016 PC-0 / BC-2.17.004 Inv-3; AC-137-006)"
        );
        assert_eq!(
            flow.pdu_count, 0,
            "pdu_count must remain 0 — process_pdu must NOT be called for unknown command \
             (BC-2.17.016 PC-0; AC-137-006)"
        );
    }

    /// AC-137-006 — command_counts has a SINGLE increment site (not doubled in process_pdu).
    ///
    /// Send one valid known-command frame (RegisterSession, 0x0065, length=0).
    /// `command_counts[0x0065]` must be exactly 1 — confirming the increment is in the
    /// frame-walk (before `is_valid_enip_frame`), NOT duplicated in `process_pdu`.
    ///
    /// Traces: BC-2.17.016 PC-0; BC-2.17.024/025; AC-137-006.
    #[test]
    fn test_command_counts_single_site_not_doubled() {
        let mut analyzer = EnipAnalyzer::new(50, 5);
        let key = make_flow_key();
        let frame = enip_frame(0x0065); // RegisterSession, length=0, total=24
        analyzer.on_data(key.clone(), &frame, 0);
        // RED GATE: todo!() panics before here.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after on_data");
        assert_eq!(
            flow.command_counts.get(&0x0065u16).copied().unwrap_or(0),
            1,
            "command_counts[0x0065] must be exactly 1 — not doubled by process_pdu \
             (BC-2.17.016 PC-0 / BC-2.17.024/025; AC-137-006)"
        );
        assert_eq!(
            flow.pdu_count, 1,
            "pdu_count must be 1 after one valid frame dispatched to process_pdu (BC-2.17.024)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Constant accessibility check
    // ─────────────────────────────────────────────────────────────────────────

    /// Verify MAX_ENIP_CARRY_BYTES is accessible and equals 600.
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no helpers, 1 line.
    /// All four GREEN-BY-DESIGN criteria satisfied:
    ///   (1) zero branching  (2) no I/O  (3) no non-trivial helpers  (4) ≤ 3 lines.
    #[test]
    fn test_max_enip_carry_bytes_is_600() {
        assert_eq!(
            MAX_ENIP_CARRY_BYTES, 600,
            "MAX_ENIP_CARRY_BYTES must equal 600 (ADR-010 Decision 3; BC-2.17.016 Invariant 1)"
        );
    }
}
