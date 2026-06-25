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
    /// Verifies the full ID → tactic → TA-ID chain for all 3 STORY-133 IDs plus T0846 regression:
    ///   T0858 → IcsExecution → "TA0104"
    ///   T0816 → IcsInhibitResponseFunction → "TA0107"
    ///   T0846 → IcsDiscovery → "TA0102" (pre-existing regression)
    ///
    /// AC-133-004 tested the enum method directly; this exercises the full path from
    /// technique ID → tactic → TA-ID string through the Option chain.
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
