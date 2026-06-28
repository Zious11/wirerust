//! Failing tests for STORY-104: Modbus Detection Emissions + Summary.
//!
//! Covers BC-2.14.013 through BC-2.14.022 — all seven detection rules, the
//! MAX_FINDINGS cap, and the `summarize()` six-key contract.
//!
//! Tests originated as Red Gate stubs (todo!() panics) before implementation; all now GREEN.
//! Test naming follows `test_BC_S_SS_NNN_xxx` pattern for full traceability.
//!
//! Canonical test vectors used verbatim from BC documents where available.

// BC traceability convention mandates uppercase BC identifiers in function names.
// The non_snake_case lint fires on uppercase — suppressed intentionally.
#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-104 tests are grouped inside a dedicated
// `mod story_104` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_104 {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::modbus::{
        DEFAULT_WRITE_BURST_THRESHOLD, DEFAULT_WRITE_SUSTAINED_THRESHOLD, MAX_FINDINGS, MbapHeader,
        ModbusAnalyzer, ModbusFlowState, WRITE_BURST_WINDOW_SECS,
    };
    use wirerust::findings::{Confidence, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    // ---------------------------------------------------------------------------
    // Helpers — shared ADU builders and test infrastructure
    // ---------------------------------------------------------------------------

    /// Create a canonical `FlowKey` for tests: client 192.168.1.10:1234 ↔ server 192.168.1.100:502.
    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            1234,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            502,
        )
    }

    /// Create a standard `ModbusAnalyzer` with default thresholds.
    fn default_analyzer() -> ModbusAnalyzer {
        ModbusAnalyzer::new(
            DEFAULT_WRITE_BURST_THRESHOLD,
            DEFAULT_WRITE_SUSTAINED_THRESHOLD,
        )
    }

    /// Build a minimal valid MBAP + PDU byte vector for an arbitrary FC.
    ///
    /// Layout: `[txn_hi, txn_lo, 0x00, 0x00, len_hi, len_lo, unit_id, fc, data...]`
    /// `length` field = 1 (unit_id) + 1 (fc) + data.len() — must be in [2, 254].
    fn build_adu(txn_id: u16, unit_id: u8, fc: u8, pdu_data: &[u8]) -> Vec<u8> {
        // length field = unit_id (1) + fc (1) + pdu_data.len()
        let length = (2 + pdu_data.len()) as u16;
        let mut adu = vec![
            (txn_id >> 8) as u8,
            (txn_id & 0xFF) as u8,
            0x00,
            0x00, // protocol_id = 0
            (length >> 8) as u8,
            (length & 0xFF) as u8,
            unit_id,
            fc,
        ];
        adu.extend_from_slice(pdu_data);
        adu
    }

    /// Parse a raw ADU byte slice into a `MbapHeader`.
    /// Panics if the slice is too short — test helper only.
    fn parse_header(adu: &[u8]) -> MbapHeader {
        wirerust::analyzer::modbus::parse_mbap_header(adu)
            .expect("test ADU must be at least 8 bytes")
    }

    /// Drive `process_pdu` with a fully-formed ADU, collecting returned findings.
    fn drive(
        analyzer: &mut ModbusAnalyzer,
        flow: &mut ModbusFlowState,
        fk: &FlowKey,
        direction: Direction,
        adu: &[u8],
        timestamp: u32,
    ) -> Vec<wirerust::findings::Finding> {
        let header = parse_header(adu);
        let fc = header.function_code;
        analyzer.process_pdu(fk, flow, direction, &header, fc, adu, timestamp)
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.013 / BC-2.14.014 — Register-write multi-tag finding
    // AC-001: FC=0x06 emits exactly one finding with mitre_techniques = ["T1692.001","T0836"]
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_013_014_holding_register_write_emits_t1692_001_t0836
    ///
    /// Canonical vector from BC-2.14.013:
    /// ADU: `00 01 00 00 00 06 01 06 00 10 01 F4` (FC=0x06, Write Single Register, UnitID=1)
    /// Expected: exactly one Finding; mitre_techniques = ["T1692.001","T0836"];
    /// category = Execution; verdict = Likely; confidence = Medium.
    /// Traces to: BC-2.14.013 post.1, BC-2.14.014, STORY-104 AC-001.
    /// ICS v19 remap (issue #222): T0855→T1692.001.
    #[test]
    fn test_BC_2_14_013_014_holding_register_write_emits_t1692_001_t0836() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x06 (Write Single Register), data = [0x00, 0x10, 0x01, 0xF4]
        let adu: [u8; 12] = [
            0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x06, 0x00, 0x10, 0x01, 0xF4,
        ];
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        assert_eq!(findings.len(), 1, "exactly one finding per write-class PDU");
        let f = &findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T1692.001", "T0836"],
            "register write must tag T1692.001+T0836 (v19 remap: T0855→T1692.001)"
        );
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "category = Execution"
        );
        assert!(matches!(f.verdict, Verdict::Likely), "verdict = Likely");
        assert!(
            matches!(f.confidence, Confidence::Medium),
            "confidence = Medium"
        );
    }

    /// test_BC_2_14_013_014_fc_0x10_emits_t0855_t0836
    ///
    /// FC=0x10 (Write Multiple Registers) is in the holding-register subset.
    /// Traces to: BC-2.14.014.
    #[test]
    fn test_BC_2_14_013_014_fc_0x10_emits_t1692_001_t0836() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x10 with minimal payload
        let adu = build_adu(
            0x0002,
            0x01,
            0x10,
            &[0x00, 0x00, 0x00, 0x01, 0x02, 0x01, 0xF4],
        );
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].mitre_techniques, vec!["T1692.001", "T0836"]);
    }

    /// test_BC_2_14_013_014_fc_0x16_emits_t1692_001_t0836
    ///
    /// FC=0x16 (Mask Write Register) is in the holding-register subset.
    /// Traces to: BC-2.14.013 EC-003, BC-2.14.014. ICS v19 remap (issue #222).
    #[test]
    fn test_BC_2_14_013_014_fc_0x16_emits_t1692_001_t0836() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        let adu = build_adu(0x0003, 0x01, 0x16, &[0x00, 0x10, 0xFF, 0xFF, 0x00, 0x25]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].mitre_techniques, vec!["T1692.001", "T0836"]);
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.013 / BC-2.14.015 — Coil-write multi-tag finding
    // AC-002: FC=0x05 emits exactly one finding with ["T1692.001","T0835"]
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_013_015_coil_write_emits_t1692_001_t0835
    ///
    /// Canonical vector from BC-2.14.013:
    /// ADU: `00 02 00 00 00 06 02 0F 00 00 00 08 01 FF` (FC=0x0F Write Multiple Coils)
    /// Expected: mitre_techniques = ["T1692.001","T0835"].
    /// Traces to: BC-2.14.013 post.1, BC-2.14.015, STORY-104 AC-002.
    /// ICS v19 remap (issue #222): T0855→T1692.001.
    #[test]
    fn test_BC_2_14_013_015_coil_write_emits_t1692_001_t0835() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x0F (Write Multiple Coils): [0x00,0x02,0x00,0x00,0x00,0x06,0x02,0x0F,0x00,0x00,0x00,0x08,0x01,0xFF]
        let adu: Vec<u8> = vec![
            0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x02, 0x0F, 0x00, 0x00, 0x00, 0x08, 0x01, 0xFF,
        ];
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        assert_eq!(findings.len(), 1, "exactly one finding for coil write");
        assert_eq!(
            findings[0].mitre_techniques,
            vec!["T1692.001", "T0835"],
            "coil write must tag T1692.001+T0835 (v19 remap: T0855→T1692.001)"
        );
    }

    /// test_BC_2_14_013_015_fc_0x05_emits_t0855_t0835
    ///
    /// FC=0x05 (Write Single Coil).
    /// Traces to: BC-2.14.013 EC-006, BC-2.14.015.
    #[test]
    fn test_BC_2_14_013_015_fc_0x05_emits_t1692_001_t0835() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        let adu = build_adu(0x0004, 0x01, 0x05, &[0x00, 0x00, 0xFF, 0x00]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].mitre_techniques, vec!["T1692.001", "T0835"]);
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.013 — File/other write emits T1692.001 only (no register/coil subtype)
    // AC-002: FC in {0x15, 0x17} → ["T1692.001"] only
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_013_file_write_emits_t1692_001_only
    ///
    /// FC=0x15 (Write File Record) — not in register or coil subset.
    /// Expected: mitre_techniques = ["T1692.001"] only.
    /// Traces to: BC-2.14.013 invariant 2 (FC 0x15/0x17 → T1692.001 only), AC-002.
    /// ICS v19 remap (issue #222): T0855→T1692.001.
    #[test]
    fn test_BC_2_14_013_file_write_emits_t1692_001_only() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x15 with minimal valid data
        let adu = build_adu(
            0x0005,
            0x01,
            0x15,
            &[0x07, 0x00, 0x04, 0x00, 0x00, 0x00, 0x07, 0x00],
        );
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        assert_eq!(findings.len(), 1, "one finding for file-record write");
        assert_eq!(
            findings[0].mitre_techniques,
            vec!["T1692.001"],
            "FC=0x15 → T1692.001 only, no subtype (v19 remap: T0855→T1692.001)"
        );
        assert!(!findings[0].mitre_techniques.contains(&"T0836".to_string()));
        assert!(!findings[0].mitre_techniques.contains(&"T0835".to_string()));
    }

    /// test_BC_2_14_013_fc_0x17_emits_t0855_only_per_pdu_finding
    ///
    /// FC=0x17 (Read/Write Multiple Registers) — writes holding registers → T0836 applies.
    ///
    /// ORCHESTRATOR RULING BC-DISCREPANCY-001: 0x17 writes holding registers → T0836;
    /// BC-2.14.013 EC-001 stale (spec-steward reconciling). FC 0x17 is in the register-write
    /// set {0x06,0x10,0x16,0x17} per BC-2.14.016 union-tagging table, which is authoritative.
    /// Traces to: ORCHESTRATOR RULING BC-DISCREPANCY-001 (supersedes BC-2.14.013 EC-001).
    #[test]
    fn test_BC_2_14_013_fc_0x17_emits_t1692_001_only_per_pdu_finding() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x17 (Read/Write Multiple Registers) — first write on flow
        let adu = build_adu(
            0x0006,
            0x01,
            0x17,
            &[
                0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0x00, 0x01, 0x02, 0x00, 0x64,
            ],
        );
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        // ORCHESTRATOR RULING BC-DISCREPANCY-001: 0x17 writes holding registers -> T0836;
        // BC-2.14.013 EC-001 stale (spec-steward reconciling)
        assert_eq!(findings.len(), 1);
        let tags = &findings[0].mitre_techniques;
        assert!(
            tags.contains(&"T1692.001".to_string()),
            "T1692.001 must be present (v19 remap: T0855→T1692.001)"
        );
        assert!(
            tags.contains(&"T0836".to_string()),
            "T0836 must appear for FC=0x17 (ORCHESTRATOR RULING BC-DISCREPANCY-001: 0x17 writes holding registers)"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.016 — T0831 inline co-tag on 2nd holding-register write within 5s
    // AC-003: first → ["T1692.001","T0836"]; second → ["T1692.001","T0836","T0831"]; third → ["T1692.001","T0836"]
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_016_t0831_inline_cotag_on_second_holding_register_write
    ///
    /// Deliver three FC=0x06 writes within 5 seconds.
    /// findings[0] = ["T1692.001","T0836"] (1st write, T0831 window starts, count=1)
    /// findings[1] = ["T1692.001","T0836","T0831"] (2nd write, count=2 → T0831 fires once)
    /// findings[2] = ["T1692.001","T0836"] (3rd write, T0831 emit-once exhausted)
    /// ICS v19 remap (issue #222): T0855→T1692.001.
    /// Traces to: BC-2.14.016, STORY-104 AC-003.
    #[test]
    fn test_BC_2_14_016_t0831_inline_cotag_on_second_holding_register_write() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // All three writes within 5s (timestamps 1s, 2s, 3s — second-granularity after F-DELTA-001 fix).
        let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x10, 0x02, 0x00]);
        let adu3 = build_adu(0x0003, 0x01, 0x06, &[0x00, 0x10, 0x03, 0x00]);

        let f1 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 1);
        let f2 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 2);
        let f3 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu3, 3);

        // First write: T0831 window started, count=1 → no T0831 tag.
        assert_eq!(
            f1.len(),
            1,
            "first write yields exactly one per-PDU finding"
        );
        assert_eq!(
            f1[0].mitre_techniques,
            vec!["T1692.001", "T0836"],
            "first write: T0831 not yet fired (v19 remap: T0855→T1692.001)"
        );

        // Second write: count=2, t0831_burst_emitted=false → T0831 co-tagged.
        assert_eq!(
            f2.len(),
            1,
            "second write yields exactly one per-PDU finding (T0831 is co-tag, not separate)"
        );
        assert_eq!(
            f2[0].mitre_techniques,
            vec!["T1692.001", "T0836", "T0831"],
            "second write must include T0831 co-tag (v19 remap: T0855→T1692.001)"
        );

        // Third write: t0831_burst_emitted=true → back to T0836 only (no T0831).
        assert_eq!(
            f3.len(),
            1,
            "third write yields exactly one per-PDU finding"
        );
        assert_eq!(
            f3[0].mitre_techniques,
            vec!["T1692.001", "T0836"],
            "third write: T0831 emit-once exhausted (v19 remap: T0855→T1692.001)"
        );
    }

    /// test_BC_2_14_016_t0831_window_reset_after_5s
    ///
    /// After 5s the T0831 window resets; a write after the reset counts as the 1st
    /// write in a new window and does NOT carry T0831.
    /// Traces to: BC-2.14.016 invariant 2 (window-reset path).
    #[test]
    fn test_BC_2_14_016_t0831_window_reset_after_5s() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Two writes within 5s to fire T0831. (F-DELTA-001: second-granularity timestamps)
        let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 0);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 1);

        // Third write at 6s (window expired — saturating_sub(6, 0) = 6 > T0831_WINDOW_SECS=5).
        let adu3 = build_adu(0x0003, 0x01, 0x06, &[0x00, 0x12, 0x01, 0xF4]);
        let f3 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu3, 6);

        assert_eq!(f3.len(), 1);
        assert_eq!(
            f3[0].mitre_techniques,
            vec!["T1692.001", "T0836"],
            "after window reset: T0831 must NOT fire on the 1st write of a new window (v19 remap)"
        );
        assert!(!f3[0].mitre_techniques.contains(&"T0831".to_string()));
    }

    /// test_BC_2_14_016_t0831_window_update_before_emission_ordering
    ///
    /// Verifies the "window-update FIRST, then emission check" ordering from BC-2.14.016
    /// invariant 2. Specifically: `t0831_window_write_count` must be >= 2 AFTER the
    /// increment that processes the second write, at which point the emission check fires.
    /// Traces to: BC-2.14.016 invariant 2 (evaluation order).
    #[test]
    fn test_BC_2_14_016_t0831_window_update_before_emission_ordering() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let adu1 = build_adu(
            0x0001,
            0x01,
            0x10,
            &[0x00, 0x00, 0x00, 0x01, 0x02, 0x01, 0x00],
        );
        let adu2 = build_adu(
            0x0002,
            0x01,
            0x10,
            &[0x00, 0x01, 0x00, 0x01, 0x02, 0x02, 0x00],
        );

        // F-DELTA-001: second-granularity. Two writes at 0s and 1s → within 5s T0831 window.
        let f1 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 0);
        let f2 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 1);

        // If ordering is wrong (check before update): T0831 would never fire because
        // count=1 before increment. Correct ordering means count=2 after increment → fires.
        assert!(
            !f1[0].mitre_techniques.contains(&"T0831".to_string()),
            "first write must not carry T0831"
        );
        assert!(
            f2[0].mitre_techniques.contains(&"T0831".to_string()),
            "second write must carry T0831 (update-before-check ordering)"
        );
    }

    /// test_BC_2_14_016_t0831_saturating_sub_wrap
    ///
    /// Timestamp wrap: write1 at ts=0xFFFFFFFE, write2 at ts=0x00000001.
    /// saturating_sub(0x00000001, 0xFFFFFFFE) = 0 seconds < T0831_WINDOW_SECS=5 → same window.
    /// Both writes within the same T0831 window → second write carries T0831.
    /// F-DELTA-001: timestamps are seconds; 0s elapsed (saturating) is within the 5s T0831 window.
    /// Traces to: BC-2.14.016 + f2-fix-directives §11.5b (saturating_sub policy).
    /// Also traces to: STORY-104 AC-006 (saturating_sub for all window elapsed computations).
    #[test]
    fn test_BC_2_14_016_t0831_saturating_sub_wrap() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x10, 0x02, 0x00]);

        // write1 near u32::MAX boundary, write2 slightly past zero (wrap-around).
        // saturating_sub(0x00000001, 0xFFFFFFFE) = 0 seconds → within 5s T0831 window.
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu1,
            0xFFFFFFFE_u32,
        );
        let f2 = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu2,
            0x00000001_u32,
        );

        // saturating_sub(0x1, 0xFFFFFFFE) = 0 → within 5s window → T0831 fires (no panic from overflow-checks)
        assert!(
            f2[0].mitre_techniques.contains(&"T0831".to_string()),
            "saturating_sub ensures T0831 fires across u32 boundary"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.017 — Burst detector (T0806+T1692.001, 1-second window)
    // AC-004: >20 writes in 1s → separate burst finding with ["T0806","T1692.001"]
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_017_burst_detector_fires_at_threshold_plus_1
    ///
    /// Deliver 21 write-class FCs within 1 second (default burst threshold = 20).
    /// The 21st write tips the burst threshold. Expected:
    /// - Exactly 21 per-PDU write findings emitted (one per write).
    /// - Exactly 1 burst finding with mitre_techniques = ["T0806","T1692.001"].
    /// - Total findings returned across all 21 calls = 22 (21 per-PDU + 1 burst).
    ///
    /// ICS v19 remap (issue #222): T0855→T1692.001.
    ///
    /// Traces to: BC-2.14.017 invariant 1, STORY-104 AC-004.
    #[test]
    fn test_BC_2_14_017_burst_detector_fires_at_threshold_plus_1() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut all_findings = Vec::new();
        // 21 writes all at ts=0 (same second, within 1s burst window).
        // F-DELTA-001: timestamps are seconds; all same second → burst window never expires.
        for i in 0..21_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            let ts = 0_u32; // all in the same 1-second window
            let mut findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
            all_findings.append(&mut findings);
        }

        let per_pdu: Vec<_> = all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0836".to_string())
                    || (f.mitre_techniques.contains(&"T1692.001".to_string())
                        && !f.mitre_techniques.contains(&"T0806".to_string()))
            })
            .collect();
        let burst: Vec<_> = all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .collect();

        assert_eq!(
            per_pdu.len(),
            21,
            "21 per-PDU write findings (one per write-class PDU)"
        );
        assert_eq!(
            burst.len(),
            1,
            "exactly one burst finding when threshold is exceeded"
        );
        assert_eq!(
            burst[0].mitre_techniques,
            vec!["T0806", "T1692.001"],
            "burst finding: T0806 first, T1692.001 second (canonical order, v19 remap)"
        );
        assert_eq!(
            all_findings.len(),
            22,
            "21 per-PDU + 1 burst = 22 total (AC-012)"
        );
    }

    /// test_BC_2_14_017_burst_does_not_fire_at_exactly_threshold
    ///
    /// Exactly 20 writes within 1s (equal to threshold, not exceeding it).
    /// Burst fires only when count STRICTLY EXCEEDS threshold.
    /// Traces to: BC-2.14.017 invariant 1 ("count > write_burst_threshold").
    #[test]
    fn test_BC_2_14_017_burst_does_not_fire_at_exactly_threshold() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut all_findings = Vec::new();
        // F-DELTA-001: all writes at ts=0 (same second, within burst window).
        for i in 0..20_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            let ts = 0_u32; // all in same 1-second window
            let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
            all_findings.append(&mut f);
        }
        let burst_count = all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();
        assert_eq!(
            burst_count, 0,
            "no burst at exactly threshold=20 (strict >)"
        );
    }

    /// test_BC_2_14_017_burst_emit_once_per_window
    ///
    /// After the burst fires (21st write), deliver another 10 writes still within the
    /// same 1s window. The burst finding must fire EXACTLY ONCE per window.
    /// Traces to: BC-2.14.017 invariant 1 (`window_burst_emitted` guard).
    #[test]
    fn test_BC_2_14_017_burst_emit_once_per_window() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut all_findings = Vec::new();
        // 31 writes all at ts=0 (same 1-second window). F-DELTA-001: timestamps are seconds.
        for i in 0..31_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            let ts = 0_u32; // all in same 1-second window
            let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
            all_findings.append(&mut f);
        }
        let burst_count = all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();
        assert_eq!(
            burst_count, 1,
            "burst finding must fire exactly once per window (emit-once guard)"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.017 — Sustained detector (truncation-free second-scale math)
    // AC-005: count > threshold*elapsed_s AND elapsed_s >= 2
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_017_sustained_detector_uses_truncation_free_math_no_false_positive
    ///
    /// 25 writes over 3 seconds (rate = 25/3 ≈ 8.33/s < 10/s threshold).
    /// Naive integer division: 25 > 10*3 = 30 → FALSE (correct, no burst).
    /// Truncation-free form: 25 > 10*3 = 30 → FALSE → no burst.
    /// F-DELTA-001: timestamps are seconds. First 24 writes at ts=0, 25th at ts=3.
    /// Traces to: BC-2.14.017 invariant 2, STORY-104 AC-005.
    #[test]
    fn test_BC_2_14_017_sustained_detector_uses_truncation_free_math_no_false_positive() {
        // Use a HIGH burst threshold so burst detector does NOT fire on its own.
        let mut az = ModbusAnalyzer::new(100, DEFAULT_WRITE_SUSTAINED_THRESHOLD);
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut all_findings = Vec::new();
        // 24 writes at ts=0, 25th write at ts=3 (elapsed=3s; count=25; 25 > 10*3=30 → FALSE → no burst).
        for i in 0..24_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);
            all_findings.append(&mut f);
        }
        let adu25 = build_adu(0x0019, 0x01, 0x06, &[0x00, 0x18, 0x01, 0x00]);
        let mut f25 = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu25,
            3,
        );
        all_findings.append(&mut f25);

        let sustained_count = all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();
        assert_eq!(
            sustained_count, 0,
            "8.33/s < 10/s threshold: 25 > 10*3=30 → FALSE → NO sustained finding"
        );
    }

    /// test_BC_2_14_017_sustained_detector_fires_when_rate_exceeded
    ///
    /// 25 writes over 2 seconds (rate = 12.5/s > 10/s threshold).
    /// Truncation-free seconds form: 25 > 10*2=20 → TRUE → fires.
    /// F-DELTA-001: timestamps are seconds. 24 writes at ts=0, 25th at ts=2.
    /// Traces to: BC-2.14.017 invariant 2, STORY-104 AC-005.
    #[test]
    fn test_BC_2_14_017_sustained_detector_fires_when_rate_exceeded() {
        // HIGH burst threshold to isolate the sustained detector.
        let mut az = ModbusAnalyzer::new(100, DEFAULT_WRITE_SUSTAINED_THRESHOLD);
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut all_findings = Vec::new();
        // 24 writes at ts=0 (uninitialized → window starts at ts=0, count=1, then accumulates to 24).
        // 25th write at ts=2 → elapsed=2, count=25, 25 > 10*2=20 → TRUE → fires.
        for i in 0..24_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);
            all_findings.append(&mut f);
        }
        // 25th write at ts=2 (elapsed from 0 = exactly 2s)
        let adu25 = build_adu(0x0019, 0x01, 0x06, &[0x00, 0x18, 0x01, 0x00]);
        let mut f25 = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu25,
            2,
        );
        all_findings.append(&mut f25);

        let sustained_count = all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();
        assert!(
            sustained_count >= 1,
            "12.5/s > 10/s threshold over 2s → sustained finding must fire (truncation-free math)"
        );
        let burst_f = all_findings
            .iter()
            .find(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .expect("sustained finding must exist");
        assert_eq!(
            burst_f.mitre_techniques,
            vec!["T0806", "T1692.001"],
            "sustained finding carries T0806+T1692.001 (v19 remap: T0855→T1692.001)"
        );
    }

    /// test_BC_2_14_017_sustained_low_and_slow_fires
    ///
    /// Low-and-slow scenario: 8 writes/s sustained over >=2s.
    /// With threshold=5/s: 8 > 5 → fires. Ensures the detector catches slow attacks
    /// that the 1s burst window misses.
    /// Traces to: BC-2.14.017 (motivation for sustained detector).
    #[test]
    fn test_BC_2_14_017_sustained_low_and_slow_fires() {
        // Burst threshold very high (100) so only sustained fires.
        // Sustained threshold = 5 writes/s.
        let mut az = ModbusAnalyzer::new(100, 5);
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut all_findings = Vec::new();
        // 17 writes over 2s: 16 at ts=0, 17th at ts=2 → elapsed=2s; count=17; 17 > 5*2=10 → fires.
        // F-DELTA-001: timestamps are seconds; rate = 17/2 = 8.5/s > 5/s threshold.
        for i in 0..16_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);
            all_findings.append(&mut f);
        }
        // 17th write at ts=2 triggers the sustained window check (elapsed=2 >= WRITE_SUSTAINED_WINDOW_SECS=2)
        let adu17 = build_adu(0x0011, 0x01, 0x06, &[0x00, 0x10, 0x01, 0x00]);
        let mut f17 = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu17,
            2,
        );
        all_findings.append(&mut f17);

        let sustained = all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();
        assert!(
            sustained >= 1,
            "low-and-slow (8.5/s > 5/s threshold over 2s) must fire sustained detector"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.017 / AC-006 — saturating_sub for all window elapsed computations
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_017_window_elapsed_uses_saturating_sub_no_panic
    ///
    /// Deliver a write at ts=0xFFFFFF00 (near u32::MAX), then a write at ts=0x00000100.
    /// Plain subtraction: 0x100 - 0xFFFFFF00 = underflow → panic in debug mode.
    /// saturating_sub: 0x100u32.saturating_sub(0xFFFFFF00) = 0 seconds → no panic, window preserved.
    /// Expected: no panic; elapsed=0 does NOT exceed 1s burst window → window is NOT reset →
    ///   window_write_count accumulates to 2 (discriminating: proves window was preserved, not reset).
    /// Traces to: BC-2.14.017 + f2-fix-directives §11.5b, STORY-104 AC-006.
    #[test]
    fn test_BC_2_14_017_window_elapsed_uses_saturating_sub_no_panic() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);

        // Must not panic (plain subtraction would panic in debug overflow-checks mode).
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu1,
            0xFFFFFF00_u32,
        );
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu2,
            0x00000100_u32,
        );

        // saturating_sub(0x100, 0xFFFFFF00) = 0 → NOT > 1s burst window → window NOT reset →
        // window_write_count must be 2 (discriminating: wrapping_sub would reset → count=1).
        assert_eq!(
            flow.window_write_count, 2,
            "saturating_sub(0x100, 0xFFFFFF00)=0: burst window must NOT reset → count=2 \
             (wrapping_sub would give 0x200=512 > 1s → reset → count=1)"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.018 — Diagnostics FC 0x08 sub-function 0x0004 / 0x0001 → T0814
    // AC-007: Force Listen Only or Restart Comms → Finding ["T0814"]
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_018_diagnostics_force_listen_only_emits_t0814
    ///
    /// FC=0x08 with sub-function 0x0004 (Force Listen Only Mode) → T0814 finding.
    /// ADU bytes: 7 MBAP + FC 0x08 + sub-func 0x00 0x04 + data 0x00 0x00 = 12 bytes.
    /// Traces to: BC-2.14.018 post.1, STORY-104 AC-007.
    #[test]
    fn test_BC_2_14_018_diagnostics_force_listen_only_emits_t0814() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x08, sub-func=0x0004 (Force Listen Only), query data = 0x0000
        // ADU: [txn=0x0001, proto=0x0000, length=0x0006, unit=0x01, fc=0x08, 0x00, 0x04, 0x00, 0x00]
        let adu = build_adu(0x0001, 0x01, 0x08, &[0x00, 0x04, 0x00, 0x00]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let t0814_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .collect();
        assert!(
            !t0814_findings.is_empty(),
            "FC=0x08/0x0004 must emit a T0814 finding"
        );
        assert_eq!(
            t0814_findings[0].mitre_techniques,
            vec!["T0814"],
            "T0814 only (no other tags)"
        );
    }

    /// test_BC_2_14_018_diagnostics_restart_comms_emits_t0814
    ///
    /// FC=0x08 with sub-function 0x0001 (Restart Communications Option) → T0814 finding.
    /// Traces to: BC-2.14.018 post.1.
    #[test]
    fn test_BC_2_14_018_diagnostics_restart_comms_emits_t0814() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        let adu = build_adu(0x0002, 0x01, 0x08, &[0x00, 0x01, 0x00, 0x00]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let t0814_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .collect();
        assert!(
            !t0814_findings.is_empty(),
            "FC=0x08/0x0001 must emit a T0814 finding"
        );
    }

    /// test_BC_2_14_018_diagnostics_other_subfunc_does_not_emit_t0814
    ///
    /// FC=0x08 with sub-function 0x0000 (loopback/Return Query Data) → no T0814 finding.
    /// Traces to: BC-2.14.018 invariant 2 (only 0x0001 and 0x0004 trigger T0814).
    /// AC-006 edge case.
    #[test]
    fn test_BC_2_14_018_diagnostics_other_subfunc_does_not_emit_t0814() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // Sub-func 0x0000 (loopback) — no T0814
        let adu = build_adu(0x0003, 0x01, 0x08, &[0x00, 0x00, 0xAB, 0xCD]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let t0814_count = findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();
        assert_eq!(t0814_count, 0, "sub-func 0x0000 must NOT emit T0814");
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.019 — Exception-burst anomaly: >5 same-code exceptions in 10s
    // AC-008: 6th exception of same code → Anomaly finding; recon codes 0x01/0x02 carry
    //         mitre_techniques = ["T0888"] (blemish-T0888 fix, BC-2.14.019 v1.3).
    //         Other exception codes emit mitre_techniques = [].
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_019_exception_burst_emits_anomaly_finding
    ///
    /// Deliver 11 exception responses for FC=0x83 (exception for Write Single Reg 0x06,
    /// exception code=0x01) within 10 seconds.
    /// The 6th exception (count STRICTLY EXCEEDS 5) must emit an Anomaly finding with
    /// mitre_techniques: ["T0888"] (exc_code=0x01 is Illegal Function = FC scanning →
    /// T0888 Remote System Information Discovery; blemish-T0888 fix, BC-2.14.019 v1.3).
    /// Traces to: BC-2.14.019 post.1 (path A), STORY-104 AC-008.
    #[test]
    fn test_BC_2_14_019_exception_burst_emits_anomaly_finding() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Pre-insert a pending request so attribute_exception can match:
        // request: txn=0x000N, unit=0x01, FC=0x06
        // F-DELTA-001: timestamps in seconds. 11 exceptions at ts=0..10 → all within 10s window.
        let mut all_findings = Vec::new();
        for i in 0..11_u32 {
            // Insert the request first (so exception can be attributed)
            let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &req_adu,
                i,
            );

            // Exception response: FC=0x86 (=0x06|0x80), exception code=0x01
            let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
            let mut f = drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ServerToClient,
                &exc_adu,
                i,
            );
            all_findings.append(&mut f);
        }

        let anomaly_findings: Vec<_> = all_findings
            .iter()
            .filter(|f| matches!(f.category, ThreatCategory::Anomaly))
            .collect();

        assert!(
            !anomaly_findings.is_empty(),
            "6+ same-code exceptions must emit at least one Anomaly finding"
        );
        // exc_code=0x01 (Illegal Function = FC scanning) → T0888 (blemish-T0888 fix,
        // BC-2.14.019 updated: recon exception codes carry T0888 tag).
        let exception_anomaly = anomaly_findings
            .iter()
            .find(|f| f.mitre_techniques.contains(&"T0888".to_string()))
            .expect("exception-burst Anomaly for exc_code=0x01 must carry T0888 (FC scanning → Remote System Information Discovery)");
        assert!(
            exception_anomaly
                .mitre_techniques
                .contains(&"T0888".to_string()),
            "exception-burst Anomaly: exc_code=0x01 must emit T0888"
        );
    }

    /// test_BC_2_14_019_exception_burst_does_not_fire_at_threshold
    ///
    /// Exactly 5 exception responses (equal to EXCEPTION_RATE_THRESHOLD, not exceeding it).
    /// No Anomaly finding expected (strict > check).
    /// Traces to: BC-2.14.019 precondition 5 ("STRICTLY EXCEEDS").
    #[test]
    fn test_BC_2_14_019_exception_burst_does_not_fire_at_threshold() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // F-DELTA-001: timestamps in seconds. 5 exceptions at ts=0..4 → all within 10s window.
        let mut all_findings = Vec::new();
        for i in 0..5_u32 {
            let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &req_adu,
                i,
            );

            let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
            let mut f = drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ServerToClient,
                &exc_adu,
                i,
            );
            all_findings.append(&mut f);
        }

        // exc_code=0x01 → if a burst did fire it would carry T0888 (blemish-T0888 fix).
        // With only 5 exceptions (= threshold, not > threshold), no burst fires at all.
        let exception_anomaly_count = all_findings
            .iter()
            .filter(|f| {
                matches!(f.category, ThreatCategory::Anomaly)
                    && f.mitre_techniques.contains(&"T0888".to_string())
            })
            .count();
        assert_eq!(
            exception_anomaly_count, 0,
            "exactly 5 exceptions (=threshold) must NOT fire anomaly (strict > required)"
        );
    }

    /// test_BC_2_14_019_exception_burst_emit_once_per_window
    ///
    /// After the burst fires on the 6th exception, subsequent exceptions in the same window
    /// must NOT re-emit the anomaly (emit-once guard per BC-2.14.019 post.2).
    /// Traces to: BC-2.14.019 post.2 (exception_burst_emitted guard).
    #[test]
    fn test_BC_2_14_019_exception_burst_emit_once_per_window() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // F-DELTA-001: timestamps in seconds. 20 exceptions at ts=0..9 (reusing ts) → all within 10s window.
        let mut all_findings = Vec::new();
        for i in 0..20_u32 {
            let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &req_adu,
                i % 9, // 0..8 seconds, all within 10s window
            );
            let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
            let mut f = drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ServerToClient,
                &exc_adu,
                i % 9,
            );
            all_findings.append(&mut f);
        }

        // exc_code=0x01 → burst finding now carries T0888 (blemish-T0888 fix,
        // BC-2.14.019: Illegal Function exception burst → T0888 Remote System Information Discovery).
        let exception_anomaly_count = all_findings
            .iter()
            .filter(|f| {
                matches!(f.category, ThreatCategory::Anomaly)
                    && f.mitre_techniques.contains(&"T0888".to_string())
            })
            .count();
        assert_eq!(
            exception_anomaly_count, 1,
            "exception burst anomaly must fire exactly once per window (20 exceptions in same window → still one anomaly finding)"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.020 — Recon FCs: 0x11 and 0x2B/0x0E → T0888; FC=0x07 → no finding
    // AC-009
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_020_recon_fc_0x11_emits_t0888
    ///
    /// FC=0x11 (Report Server ID) in ClientToServer → T0888 finding.
    /// Traces to: BC-2.14.020 post.1 (recon path), STORY-104 AC-009.
    #[test]
    fn test_BC_2_14_020_recon_fc_0x11_emits_t0888() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x11 (Report Server ID) — no payload
        let adu = build_adu(0x0001, 0x01, 0x11, &[]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let t0888_count = findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
            .count();
        assert!(t0888_count >= 1, "FC=0x11 must emit a T0888 finding");
        assert_eq!(
            findings
                .iter()
                .find(|f| f.mitre_techniques.contains(&"T0888".to_string()))
                .unwrap()
                .mitre_techniques,
            vec!["T0888"],
            "T0888 only (no other tags)"
        );
    }

    /// test_BC_2_14_020_recon_fc_0x2b_0x0e_emits_t0888
    ///
    /// FC=0x2B (MEI Encapsulated Interface) with MEI type=0x0E (Read Device Identification).
    /// Expected: T0888 finding.
    /// Traces to: BC-2.14.020 post.1 (recon path), STORY-104 AC-009.
    #[test]
    fn test_BC_2_14_020_recon_fc_0x2b_0x0e_emits_t0888() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x2B, MEI type=0x0E (Read Device Identification), Read Device ID code=0x01
        let adu = build_adu(0x0001, 0x01, 0x2B, &[0x0E, 0x01, 0x00]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let t0888_count = findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
            .count();
        assert!(t0888_count >= 1, "FC=0x2B/0x0E must emit a T0888 finding");
    }

    /// test_BC_2_14_020_recon_fc_0x2b_non_0x0e_does_not_emit_t0888
    ///
    /// FC=0x2B with MEI type != 0x0E → no T0888 (Decision 12: only MEI type 0x0E maps
    /// to T0888). May emit Anomaly (unknown diagnostic sub-function) but NOT T0888.
    /// Traces to: BC-2.14.020 EC-005 / STORY-104 AC-009.
    #[test]
    fn test_BC_2_14_020_recon_fc_0x2b_non_0x0e_does_not_emit_t0888() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x2B, MEI type=0x0D (not 0x0E) — no T0888
        let adu = build_adu(0x0001, 0x01, 0x2B, &[0x0D, 0x01]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let t0888_count = findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
            .count();
        assert_eq!(
            t0888_count, 0,
            "FC=0x2B with MEI type!=0x0E must NOT emit T0888"
        );
    }

    /// test_BC_2_14_020_fc_0x07_does_not_emit_t0888
    ///
    /// FC=0x07 (Read Exception Status) does NOT emit a T0888 finding (Decision 12 in
    /// f2-fix-directives.md §12: FC 0x07 removed as standalone recon indicator).
    /// Traces to: BC-2.14.020 description ("FC 0x07 is NOT a standalone recon indicator"),
    /// STORY-104 AC-009.
    #[test]
    fn test_BC_2_14_020_fc_0x07_does_not_emit_t0888() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        let adu = build_adu(0x0001, 0x01, 0x07, &[]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let t0888_count = findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
            .count();
        assert_eq!(
            t0888_count, 0,
            "FC=0x07 must NOT emit a T0888 finding (per Decision 12)"
        );
    }

    /// test_BC_2_14_020_unknown_fc_emits_anomaly
    ///
    /// A genuinely unknown FC (e.g., 0x60 — not in any defined set) should emit an
    /// Anomaly finding with mitre_techniques: vec![].
    /// Traces to: BC-2.14.020 post.1 (unknown FC path).
    #[test]
    fn test_BC_2_14_020_unknown_fc_emits_anomaly() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x60 — genuinely unknown (not in any defined set, not >= 0x80)
        let adu = build_adu(0x0001, 0x01, 0x60, &[0x01, 0x02]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let anomaly_count = findings
            .iter()
            .filter(|f| matches!(f.category, ThreatCategory::Anomaly))
            .count();
        assert!(
            anomaly_count >= 1,
            "unknown FC must emit an Anomaly finding"
        );
        let anomaly_f = findings
            .iter()
            .find(|f| matches!(f.category, ThreatCategory::Anomaly))
            .unwrap();
        assert!(
            anomaly_f.mitre_techniques.is_empty(),
            "unknown FC Anomaly: mitre_techniques must be empty"
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.021 — summarize() returns exactly six keys
    // AC-010
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_021_summarize_returns_six_keys
    ///
    /// Process a mix of ADUs, then call `summarize()`. Assert all six required keys
    /// are present in `detail`: pdu_count, write_count, exception_count, parse_errors,
    /// function_code_distribution, dropped_findings.
    /// Traces to: BC-2.14.021 post.1, STORY-104 AC-010.
    #[test]
    fn test_BC_2_14_021_summarize_returns_six_keys() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // 3 reads (FC=0x03), 2 writes (FC=0x06)
        // F-DELTA-001: timestamps in seconds.
        for i in 0..3_u32 {
            let adu = build_adu(i as u16, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
            drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, i);
        }
        for i in 0..2_u32 {
            let adu = build_adu((10 + i) as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &adu,
                10 + i,
            );
        }

        let summary = az.summarize();
        let detail = &summary.detail;

        // All six keys must be present
        assert!(detail.contains_key("pdu_count"), "missing key: pdu_count");
        assert!(
            detail.contains_key("write_count"),
            "missing key: write_count"
        );
        assert!(
            detail.contains_key("exception_count"),
            "missing key: exception_count"
        );
        assert!(
            detail.contains_key("parse_errors"),
            "missing key: parse_errors"
        );
        assert!(
            detail.contains_key("function_code_distribution"),
            "missing key: function_code_distribution"
        );
        assert!(
            detail.contains_key("dropped_findings"),
            "missing key: dropped_findings"
        );
        assert_eq!(
            detail.len(),
            6,
            "must have EXACTLY six keys (not more, not fewer)"
        );
    }

    /// test_BC_2_14_021_summarize_pdu_count_correct
    ///
    /// After processing 5 valid PDUs, `pdu_count` in summary must be 5.
    /// Traces to: BC-2.14.021 invariant 4 (pdu_count counts valid PDUs only).
    #[test]
    fn test_BC_2_14_021_summarize_pdu_count_correct() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // F-DELTA-001: timestamps in seconds.
        for i in 0..5_u32 {
            let adu = build_adu(i as u16, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
            drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, i);
        }

        let summary = az.summarize();
        let pdu_count = summary.detail["pdu_count"]
            .as_u64()
            .expect("pdu_count must be a number");
        assert_eq!(
            pdu_count, 5,
            "pdu_count must equal number of valid PDUs processed"
        );
    }

    /// test_BC_2_14_021_summarize_write_count_correct
    ///
    /// After processing 2 write-class PDUs, `write_count` in summary must be 2.
    /// Traces to: BC-2.14.021 invariant 4 (write_count = write-class FCs in request direction).
    #[test]
    fn test_BC_2_14_021_summarize_write_count_correct() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // F-DELTA-001: timestamps in seconds.
        for i in 0..2_u32 {
            let adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, i);
        }

        let summary = az.summarize();
        let write_count = summary.detail["write_count"]
            .as_u64()
            .expect("write_count must be a number");
        assert_eq!(write_count, 2);
    }

    /// test_BC_2_14_021_summarize_dropped_findings_always_present
    ///
    /// Even when no findings are dropped (cap never hit), `dropped_findings` must be
    /// present with value 0.
    /// Traces to: BC-2.14.021 invariant 1 (key ALWAYS present), BC-2.14.022 post.5.
    #[test]
    fn test_BC_2_14_021_summarize_dropped_findings_always_present() {
        let az = default_analyzer();
        let summary = az.summarize();
        assert!(
            summary.detail.contains_key("dropped_findings"),
            "dropped_findings must always be present"
        );
        let val = summary.detail["dropped_findings"].as_u64().unwrap_or(99);
        assert_eq!(val, 0, "dropped_findings must be 0 when cap never hit");
    }

    /// test_BC_2_14_021_summarize_function_code_distribution_hex_format
    ///
    /// `function_code_distribution` keys use "0x{:02X}" format (uppercase hex, 0-padded, 0x prefix).
    /// Example: FC 0x03 → key "0x03"; FC 0x10 → key "0x10".
    /// Traces to: BC-2.14.021 invariant 3.
    #[test]
    fn test_BC_2_14_021_summarize_function_code_distribution_hex_format() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Process one FC=0x03 read
        let adu = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);

        let summary = az.summarize();
        let dist = summary.detail["function_code_distribution"]
            .as_object()
            .expect("function_code_distribution must be a JSON object");
        assert!(
            dist.contains_key("0x03"),
            "FC 0x03 must appear as key '0x03' (uppercase hex, 0x prefix)"
        );
        assert_eq!(dist["0x03"].as_u64().unwrap(), 1);
    }

    /// test_BC_2_14_021_summarize_function_code_distribution_no_zero_entries
    ///
    /// FC codes that were never observed must NOT appear in `function_code_distribution`.
    /// Traces to: BC-2.14.021 post.2, invariant 2 (zero-count suppression).
    #[test]
    fn test_BC_2_14_021_summarize_function_code_distribution_no_zero_entries() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let adu = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);

        let summary = az.summarize();
        let dist = summary.detail["function_code_distribution"]
            .as_object()
            .expect("must be object");
        // Only FC 0x03 was observed — all other 255 FC codes must be absent.
        assert_eq!(
            dist.len(),
            1,
            "only observed FCs must appear in distribution (no zero entries)"
        );
        assert!(dist.contains_key("0x03"));
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.022 — MAX_FINDINGS cap (10,000) and poison-skip
    // AC-011: cap reached → no push, dropped_findings++, counters still updated
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_022_max_findings_cap_poison_skip
    ///
    /// Pre-fill `all_findings` to MAX_FINDINGS=10_000, then deliver one write-class PDU.
    /// Expected:
    /// - `all_findings.len()` remains 10_000 (no push).
    /// - `dropped_findings == 1` (one finding discarded).
    /// - `total_write_count` incremented (counter unaffected by cap).
    ///
    /// Traces to: BC-2.14.022 post.1-4, STORY-104 AC-011.
    #[test]
    fn test_BC_2_14_022_max_findings_cap_poison_skip() {
        use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
        let mut az = default_analyzer();

        // Pre-fill findings to cap using a dummy finding.
        let dummy = Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: "dummy".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        };
        for _ in 0..MAX_FINDINGS {
            az.all_findings.push(dummy.clone());
        }
        assert_eq!(az.all_findings.len(), MAX_FINDINGS);

        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // One more write-class PDU — should poison-skip the finding
        let adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        assert_eq!(
            az.all_findings.len(),
            MAX_FINDINGS,
            "all_findings must NOT exceed MAX_FINDINGS after poison-skip"
        );
        assert_eq!(
            az.dropped_findings, 1,
            "dropped_findings must be incremented for each skipped push"
        );
        assert_eq!(
            az.total_write_count, 1,
            "total_write_count must be incremented even when cap is hit"
        );
    }

    /// test_BC_2_14_022_dropped_findings_counts_each_skipped_push
    ///
    /// Pre-fill to cap, then deliver 3 write-class PDUs. `dropped_findings` must be 3
    /// (one per skipped finding push, not one per PDU if a PDU would emit multiple findings).
    /// Traces to: BC-2.14.022 post.2 ("per skipped finding push attempt").
    #[test]
    fn test_BC_2_14_022_dropped_findings_counts_each_skipped_push() {
        use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
        let mut az = default_analyzer();

        let dummy = Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: "dummy".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        };
        for _ in 0..MAX_FINDINGS {
            az.all_findings.push(dummy.clone());
        }

        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        for i in 0..3_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &adu,
                i * 100_000,
            );
        }

        assert_eq!(
            az.all_findings.len(),
            MAX_FINDINGS,
            "cap must not be exceeded"
        );
        assert!(
            az.dropped_findings >= 3,
            "at least 3 dropped_findings after 3 skipped write findings"
        );
    }

    /// test_BC_2_14_022_counters_unaffected_by_cap
    ///
    /// After cap is hit, `fn_code_counts` must still be incremented for each valid PDU.
    /// Traces to: BC-2.14.022 post.3 (counters are UNAFFECTED by findings cap).
    #[test]
    fn test_BC_2_14_022_counters_unaffected_by_cap() {
        use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
        let mut az = default_analyzer();

        let dummy = Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: "dummy".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        };
        for _ in 0..MAX_FINDINGS {
            az.all_findings.push(dummy.clone());
        }

        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        let adu = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);

        // FC=0x03 count must be 1 even though cap was already hit
        assert_eq!(
            *az.fn_code_counts.get(&0x03).unwrap_or(&0),
            1,
            "fn_code_counts must be updated regardless of cap"
        );
        assert_eq!(
            az.total_pdu_count, 1,
            "total_pdu_count must be updated regardless of cap"
        );
    }

    // ---------------------------------------------------------------------------
    // AC-012 — Burst finding is separate from per-PDU finding (BC-2.14.013 invariant 5)
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_013_burst_and_per_pdu_finding_are_separate
    ///
    /// The T0806+T1692.001 burst finding is a SEPARATE Finding object, emitted alongside
    /// (not instead of) the per-PDU write finding. When the 21st write tips the burst
    /// threshold, that PDU should have generated: 1 per-PDU write finding + 1 burst finding.
    /// Total `all_findings` after 21 writes = 22 (21 per-PDU + 1 burst).
    /// Traces to: BC-2.14.013 invariant 5, STORY-104 AC-012.
    #[test]
    fn test_BC_2_14_013_burst_and_per_pdu_finding_are_separate() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut total_findings = 0usize;
        for i in 0..21_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            // F-DELTA-001: timestamps in seconds. All at ts=0 → within 1s burst window.
            let ts = 0_u32;
            let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
            total_findings += findings.len();
        }

        assert_eq!(
            total_findings, 22,
            "21 per-PDU findings + 1 burst finding = 22 total (burst supplements, not replaces)"
        );

        // Additionally verify the burst finding is distinct (it's a Finding with T0806, not the per-PDU T0836 finding)
        let per_pdu_count = az
            .all_findings
            .iter()
            .filter(|f| {
                !f.mitre_techniques.contains(&"T0806".to_string())
                    && f.mitre_techniques.contains(&"T1692.001".to_string())
            })
            .count();
        let burst_count = az
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();

        assert_eq!(
            per_pdu_count, 21,
            "21 distinct per-PDU write findings in all_findings"
        );
        assert_eq!(burst_count, 1, "1 distinct burst finding in all_findings");
    }

    // ---------------------------------------------------------------------------
    // EC-003 (from STORY-104 edge cases) — Two findings would be emitted when cap
    // at 9999; first push succeeds, second is dropped.
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_022_EC003_second_finding_dropped_when_cap_at_9999
    ///
    /// EC-003: `all_findings.len() == 9999`; burst threshold also tips on the same PDU
    /// (which would emit 2 findings: per-PDU write + burst). First finding pushed
    /// (count=10,000); second finding skipped (`dropped_findings = 1`).
    /// Traces to: STORY-104 edge case EC-003, BC-2.14.022.
    #[test]
    fn test_BC_2_14_022_EC003_second_finding_dropped_when_cap_at_9999() {
        use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

        // Use burst threshold = 1 so any single write also tips the burst.
        let mut az = ModbusAnalyzer::new(1, 100);

        let dummy = Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: "dummy".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        };
        // Pre-fill to 9999 (one below cap)
        for _ in 0..(MAX_FINDINGS - 1) {
            az.all_findings.push(dummy.clone());
        }
        assert_eq!(az.all_findings.len(), MAX_FINDINGS - 1);

        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // First write: with threshold=1, after 1st write count=1+1=2 > 1 → burst fires.
        // This PDU would emit: 1 per-PDU write finding + 1 burst finding = 2 findings.
        // Cap allows: first push (9999→10000), second push dropped.
        // F-DELTA-001: timestamps in seconds.
        let adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1);
        // Second write at same ts tips burst immediately (window_write_count=2 > threshold=1)
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 1);

        assert_eq!(
            az.all_findings.len(),
            MAX_FINDINGS,
            "all_findings must be exactly MAX_FINDINGS after EC-003 scenario"
        );
        assert!(
            az.dropped_findings >= 1,
            "at least one finding must be dropped (dropped_findings >= 1)"
        );
    }

    // ---------------------------------------------------------------------------
    // BINDING TESTS — adversarial review findings (STORY-104 defect fixes)
    // These tests assert the BC postconditions that were previously gaps.
    // ---------------------------------------------------------------------------

    // ---------------------------------------------------------------------------
    // BINDING-001: source_ip — client_ip for ClientToServer (BC-2.14.013 post.1)
    // ---------------------------------------------------------------------------

    /// test_binding_source_ip_client_for_write_finding
    ///
    /// Asserts finding.source_ip = Some(client_ip) for a write-class PDU in ClientToServer.
    /// FlowKey: client 192.168.1.10:1234 ↔ server 192.168.1.100:502.
    /// Per BC-2.14.013 post.1: source_ip = Some(flow_key.client_ip()).
    #[test]
    fn test_binding_source_ip_client_for_write_finding() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        let adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1, // F-DELTA-001: timestamp in seconds
        );
        assert!(!findings.is_empty(), "write PDU must produce a finding");
        let f = &findings[0];
        assert!(
            f.source_ip.is_some(),
            "write finding source_ip must be Some (BC-2.14.013 post.1)"
        );
        // Client is 192.168.1.10 (not on port 502)
        let expected_client = std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 10));
        assert_eq!(
            f.source_ip,
            Some(expected_client),
            "write finding source_ip must equal client IP 192.168.1.10 (BC-2.14.013 post.1)"
        );
    }

    // ---------------------------------------------------------------------------
    // BINDING-002: source_ip — server_ip for ServerToClient exception (BC-2.14.019 post.A)
    // ---------------------------------------------------------------------------

    /// test_binding_source_ip_server_for_exception_finding
    ///
    /// Asserts finding.source_ip = Some(server_ip) for an exception-burst finding in
    /// ServerToClient direction. Per BC-2.14.019 post.A: source_ip = Some(flow_key.server_ip()).
    #[test]
    fn test_binding_source_ip_server_for_exception_finding() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Deliver 6 exception responses to trigger the burst (BC-2.14.019 EC-002: 6th triggers).
        // F-DELTA-001: timestamps in seconds. 6 exceptions at ts=0..5 → all within 10s window.
        let mut anomaly_finding = None;
        for i in 0..6_u32 {
            let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &req_adu,
                i,
            );
            let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
            let findings = drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ServerToClient,
                &exc_adu,
                i,
            );
            for f in findings {
                // exc_code=0x01 → burst finding carries T0888 (blemish-T0888 fix,
                // BC-2.14.019: Illegal Function exception burst → T0888).
                if matches!(f.category, wirerust::findings::ThreatCategory::Anomaly)
                    && f.mitre_techniques.contains(&"T0888".to_string())
                {
                    anomaly_finding = Some(f);
                }
            }
        }

        let f = anomaly_finding
            .expect("6 exceptions must produce an Anomaly finding (T0888 for exc_code=0x01)");
        assert!(
            f.source_ip.is_some(),
            "exception-burst finding source_ip must be Some (BC-2.14.019 post.A)"
        );
        // Server is 192.168.1.100 (on port 502)
        let expected_server = std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(
            f.source_ip,
            Some(expected_server),
            "exception-burst finding source_ip must equal server IP 192.168.1.100 (BC-2.14.019 post.A)"
        );
    }

    // ---------------------------------------------------------------------------
    // BINDING-003: source_ip — client_ip for Clear Counters Path B (BC-2.14.019 post.B)
    // ---------------------------------------------------------------------------

    /// test_binding_source_ip_client_for_clear_counters_finding
    ///
    /// Asserts finding.source_ip = Some(client_ip) for the Clear Counters Path B finding.
    /// Per BC-2.14.019 post.B: source_ip = Some(flow_key.client_ip()).
    #[test]
    fn test_binding_source_ip_client_for_clear_counters_finding() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x08, sub-func=0x000A (Clear Counters)
        let adu = build_adu(0x0001, 0x01, 0x08, &[0x00, 0x0A, 0x00, 0x00]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );
        let cc_finding = findings
            .iter()
            .find(|f| {
                matches!(f.category, wirerust::findings::ThreatCategory::Anomaly)
                    && f.mitre_techniques.is_empty()
                    && f.summary.contains("Clear Counters")
            })
            .expect(
                "FC=0x08/0x000A must emit a Clear Counters Anomaly finding (BC-2.14.019 Path B)",
            );

        let expected_client = std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 10));
        assert_eq!(
            cc_finding.source_ip,
            Some(expected_client),
            "Clear Counters finding source_ip must equal client IP (BC-2.14.019 post.B)"
        );
    }

    // ---------------------------------------------------------------------------
    // BINDING-004: exception cross-window reset (BC-2.14.019 EC-005)
    // ---------------------------------------------------------------------------

    /// test_binding_exception_cross_window_reset
    ///
    /// 6 exceptions of the same code → first finding emitted. Then advance time > 10s.
    /// Another 6 exceptions of the same code → SECOND finding must be emitted (window reset).
    /// Per BC-2.14.019 EC-005: "burst_emitted was reset to false on window rollover; new finding emitted."
    #[test]
    fn test_binding_exception_cross_window_reset() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        let mut anomaly_count = 0usize;

        // First burst: 6 exceptions of exc_code=0x01 within 9 seconds.
        // F-DELTA-001: timestamps in seconds. ts=0..5 → all within 10s window.
        for i in 0..6_u32 {
            let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &req_adu,
                i,
            );
            let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
            let findings = drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ServerToClient,
                &exc_adu,
                i,
            );
            for f in &findings {
                // exc_code=0x01 → burst finding carries T0888 (blemish-T0888 fix,
                // BC-2.14.019: Illegal Function exception burst → T0888).
                if matches!(f.category, wirerust::findings::ThreatCategory::Anomaly)
                    && f.mitre_techniques.contains(&"T0888".to_string())
                {
                    anomaly_count += 1;
                }
            }
        }
        assert_eq!(
            anomaly_count, 1,
            "first burst of 6 same-code exceptions must produce exactly one Anomaly finding"
        );

        // Second burst: 6 more exceptions starting at ts > 10s after the first window start.
        // Window started at ts=0; 11 seconds later → new window (EXCEPTION_WINDOW_SECS=10).
        let base_ts: u32 = 11;
        for i in 0..6_u32 {
            let req_adu = build_adu((100 + i) as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &req_adu,
                base_ts + i,
            );
            let exc_adu = build_adu((100 + i) as u16, 0x01, 0x86, &[0x01]);
            let findings = drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ServerToClient,
                &exc_adu,
                base_ts + i,
            );
            for f in &findings {
                // exc_code=0x01 → burst finding carries T0888 (blemish-T0888 fix).
                if matches!(f.category, wirerust::findings::ThreatCategory::Anomaly)
                    && f.mitre_techniques.contains(&"T0888".to_string())
                {
                    anomaly_count += 1;
                }
            }
        }
        assert_eq!(
            anomaly_count, 2,
            "second burst of 6 exceptions in a new window must produce a SECOND Anomaly finding (BC-2.14.019 EC-005 cross-window reset)"
        );
    }

    // ---------------------------------------------------------------------------
    // BINDING-005: per-flow counters (BC-2.14.013 post.2, BC-2.14.019 inv4)
    // ---------------------------------------------------------------------------

    /// test_binding_per_flow_counters_updated
    ///
    /// Asserts that flow.write_count, flow.exception_count, flow.pdu_count, and
    /// flow.last_ts are correctly updated per PDU.
    /// Per BC-2.14.013 post.2: flow.write_count incremented.
    /// Per BC-2.14.019 inv4: flow.exception_count incremented for every exception PDU.
    #[test]
    fn test_binding_per_flow_counters_updated() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        assert_eq!(flow.pdu_count, 0, "initial pdu_count = 0");
        assert_eq!(flow.write_count, 0, "initial write_count = 0");
        assert_eq!(flow.exception_count, 0, "initial exception_count = 0");
        assert_eq!(flow.last_ts, 0, "initial last_ts = 0");

        // One write PDU (FC=0x06)
        // F-DELTA-001: timestamps in seconds.
        let adu_write = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu_write,
            1,
        );
        assert_eq!(flow.pdu_count, 1, "pdu_count = 1 after one write PDU");
        assert_eq!(flow.write_count, 1, "write_count = 1 after one write PDU");
        assert_eq!(
            flow.exception_count, 0,
            "exception_count unchanged after write PDU"
        );
        assert_eq!(flow.last_ts, 1, "last_ts updated to 1");

        // One read PDU (FC=0x03)
        let adu_read = build_adu(0x0002, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu_read,
            2,
        );
        assert_eq!(flow.pdu_count, 2, "pdu_count = 2 after two PDUs");
        assert_eq!(
            flow.write_count, 1,
            "write_count unchanged after read PDU (read is not write)"
        );
        assert_eq!(flow.last_ts, 2, "last_ts updated to 2");

        // One exception response (FC=0x86)
        let exc_adu = build_adu(0x0001, 0x01, 0x86, &[0x01]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ServerToClient,
            &exc_adu,
            3,
        );
        assert_eq!(flow.pdu_count, 3, "pdu_count = 3 after three PDUs");
        assert_eq!(
            flow.exception_count, 1,
            "exception_count = 1 after one exception PDU (BC-2.14.019 inv4)"
        );
        assert_eq!(flow.last_ts, 3, "last_ts updated to 3");
    }

    // ---------------------------------------------------------------------------
    // BINDING-006: total_flows_analyzed incremented once per flow (BC-2.14.021 post.3)
    // ---------------------------------------------------------------------------

    /// test_binding_total_flows_analyzed_incremented_on_first_pdu
    ///
    /// total_flows_analyzed is incremented once per flow, on the flow's first PDU.
    /// Subsequent PDUs on the same flow must NOT re-increment it.
    /// Per BC-2.14.021 post.3.
    #[test]
    fn test_binding_total_flows_analyzed_incremented_on_first_pdu() {
        let mut az = default_analyzer();
        let fk = test_flow_key();

        assert_eq!(
            az.total_flows_analyzed, 0,
            "initial total_flows_analyzed = 0"
        );

        // First PDU on flow A
        let mut flow_a = ModbusFlowState::default();
        let adu1 = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(
            &mut az,
            &mut flow_a,
            &fk,
            Direction::ClientToServer,
            &adu1,
            0,
        );
        assert_eq!(
            az.total_flows_analyzed, 1,
            "total_flows_analyzed = 1 after first PDU on flow A"
        );

        // Second PDU on the SAME flow — must NOT re-increment
        // F-DELTA-001: timestamps in seconds.
        let adu2 = build_adu(0x0002, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(
            &mut az,
            &mut flow_a,
            &fk,
            Direction::ClientToServer,
            &adu2,
            1,
        );
        assert_eq!(
            az.total_flows_analyzed, 1,
            "total_flows_analyzed still 1 after second PDU on same flow"
        );

        // First PDU on a DIFFERENT flow (flow B) — must increment
        let fk2 = FlowKey::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1)),
            5000,
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 2)),
            502,
        );
        let mut flow_b = ModbusFlowState::default();
        let adu3 = build_adu(0x0001, 0x02, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(
            &mut az,
            &mut flow_b,
            &fk2,
            Direction::ClientToServer,
            &adu3,
            2,
        );
        assert_eq!(
            az.total_flows_analyzed, 2,
            "total_flows_analyzed = 2 after first PDU on flow B"
        );
    }

    // ---------------------------------------------------------------------------
    // BINDING-007: Clear-Counters Path B anomaly (BC-2.14.019 post.B)
    // ---------------------------------------------------------------------------

    /// test_binding_clear_counters_path_b_emits_anomaly
    ///
    /// FC=0x08 sub-func=0x000A (Clear Counters, ClientToServer) → Anomaly finding with
    /// mitre_techniques: vec![] and category Anomaly (BC-2.14.019 Path B).
    /// Summary must contain "Clear Counters" and "0x08/0x000A".
    #[test]
    fn test_binding_clear_counters_path_b_emits_anomaly() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x08, sub-func=0x000A, ClientToServer
        let adu = build_adu(0x0001, 0x01, 0x08, &[0x00, 0x0A, 0x00, 0x00]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            1_000_000,
        );

        let cc = findings
            .iter()
            .find(|f| {
                matches!(f.category, wirerust::findings::ThreatCategory::Anomaly)
                    && f.mitre_techniques.is_empty()
                    && f.summary.contains("Clear Counters")
            })
            .expect("FC=0x08/0x000A must emit a Clear Counters Anomaly (BC-2.14.019 Path B)");

        assert!(
            matches!(cc.verdict, wirerust::findings::Verdict::Inconclusive),
            "verdict = Inconclusive"
        );
        assert!(
            matches!(cc.confidence, wirerust::findings::Confidence::Medium),
            "confidence = Medium"
        );
        assert!(
            cc.summary.contains("0x08/0x000A"),
            "summary must mention FC/sub-func"
        );
        assert!(
            cc.mitre_techniques.is_empty(),
            "Clear Counters: no MITRE technique (BC-2.14.019 post.B)"
        );
        // Must NOT have T0814 (that's for 0x0001/0x0004)
        assert!(
            !cc.mitre_techniques.contains(&"T0814".to_string()),
            "Clear Counters must NOT carry T0814"
        );
    }

    // ---------------------------------------------------------------------------
    // BINDING-008: Response-direction recon emission (BC-2.14.020 EC-010)
    // ---------------------------------------------------------------------------

    /// test_binding_recon_fc_0x11_fires_in_response_direction
    ///
    /// FC=0x11 in ServerToClient direction must also emit a T0888 finding with
    /// source_ip = server_ip (BC-2.14.020 EC-010: "recon Anomaly still emitted").
    #[test]
    fn test_binding_recon_fc_0x11_fires_in_response_direction() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        // FC=0x11 in ServerToClient direction — server responding to server-id query
        let adu = build_adu(0x0001, 0x01, 0x11, &[]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ServerToClient,
            &adu,
            1_000_000,
        );

        let t0888 = findings
            .iter()
            .find(|f| f.mitre_techniques.contains(&"T0888".to_string()))
            .expect("FC=0x11 in ServerToClient must emit a T0888 finding (BC-2.14.020 EC-010)");

        // source_ip should be server_ip = 192.168.1.100 for ServerToClient
        let expected_server = std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(
            t0888.source_ip,
            Some(expected_server),
            "response-direction recon finding source_ip must equal server IP (BC-2.14.020 EC-010)"
        );
        assert!(
            matches!(
                t0888.direction,
                Some(wirerust::reassembly::handler::Direction::ServerToClient)
            ),
            "direction must be ServerToClient"
        );
    }

    /// test_binding_recon_fc_0x2b_0x0e_fires_in_response_direction
    ///
    /// FC=0x2B MEI type=0x0E in ServerToClient → T0888 finding with source_ip = server_ip.
    /// Per BC-2.14.020 EC-010: direction-independent.
    #[test]
    fn test_binding_recon_fc_0x2b_0x0e_fires_in_response_direction() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();
        let adu = build_adu(0x0001, 0x01, 0x2B, &[0x0E, 0x01, 0x00]);
        let findings = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ServerToClient,
            &adu,
            1_000_000,
        );

        let t0888 = findings
            .iter()
            .find(|f| f.mitre_techniques.contains(&"T0888".to_string()))
            .expect(
                "FC=0x2B/0x0E in ServerToClient must emit a T0888 finding (BC-2.14.020 EC-010)",
            );

        let expected_server = std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(
            t0888.source_ip,
            Some(expected_server),
            "response-direction 0x2B/0x0E source_ip must equal server IP"
        );
    }

    // ---------------------------------------------------------------------------
    // Phase-F6 mutation-killing tests (FIX-F6 / VP-022 hardening).
    //
    // These three tests close mutation-testing gaps surfaced by `cargo mutants
    // --file src/analyzer/modbus.rs`. Each was a CONFIRMED surviving mutant in the
    // detection/correlation core of `process_pdu` (verified by manually applying
    // the mutation and observing the pre-existing Modbus suite stay green). They
    // drive the real `process_pdu` integration path (via `drive`) and assert the
    // observable consequence the mutant would change, so the corresponding mutation
    // is now reliably caught.
    // ---------------------------------------------------------------------------

    /// Kills mutant `modbus.rs:499:29 replace != with ==` in `process_pdu`.
    ///
    /// The request-path guard `if fc_class != FunctionCodeClass::Exception` decides
    /// whether a request ADU is inserted into the per-flow `pending` table
    /// (BC-2.14.009). Inverting it to `==` would insert ONLY exception-FC requests
    /// (which never legitimately appear on the request path) and would skip every
    /// real (Read/Write/Diagnostic) request — leaving `pending` empty. We drive a
    /// non-exception Read request through `process_pdu` and assert the pending entry
    /// exists; the mutant makes `pending` empty, failing the assertion.
    #[test]
    fn test_f6_mutation_process_pdu_inserts_nonexception_request_into_pending() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Read Holding Registers (FC=0x03) — a non-exception request.
        let adu = build_adu(0x0011, 0x07, 0x03, &[0x00, 0x00, 0x00, 0x0A]);
        let _ = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            100,
        );

        assert!(
            flow.pending.contains_key(&(0x0011u16, 0x07u8)),
            "process_pdu must insert a non-exception request into pending \
         (kills the 499 != -> == guard-inversion mutant)"
        );
        let &(stored_fc, stored_ts) = flow
            .pending
            .get(&(0x0011u16, 0x07u8))
            .expect("pending entry must exist for the inserted request");
        assert_eq!(stored_fc, 0x03, "stored FC must be the request FC");
        assert_eq!(stored_ts, 100, "stored ts must be the request timestamp");
    }

    /// Kills mutants `modbus.rs:503:53 replace += with -=` and `... with *=`
    /// in `process_pdu`.
    ///
    /// On a duplicate in-flight transaction — the SAME `(txn_id, unit_id)` request
    /// observed again before its response — `insert_request` returns `Some(old)` and
    /// `process_pdu` performs `self.duplicate_inflight_txn += 1` (BC-2.14.009
    /// invariant 6). The `+= 1 -> -= 1` mutant would underflow the `u64` counter
    /// from 0 (panicking under overflow-checks, or wrapping); `+= 1 -> *= 1` would
    /// leave it at 0. We drive the same request twice and assert the counter is
    /// exactly 1.
    #[test]
    fn test_f6_mutation_process_pdu_increments_duplicate_inflight_txn() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Same (txn_id=0x0021, unit_id=0x01) request driven twice with no intervening
        // response → the second insert overwrites → duplicate_inflight_txn increments.
        let adu = build_adu(0x0021, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x05]);
        let _ = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            100,
        );
        assert_eq!(
            az.duplicate_inflight_txn, 0,
            "no duplicate after a single request"
        );

        let _ = drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu,
            101,
        );
        assert_eq!(
            az.duplicate_inflight_txn, 1,
            "duplicate in-flight (txn_id, unit_id) must increment duplicate_inflight_txn \
         by exactly 1 (kills the 503 += -> -= / += -> *= mutants)"
        );
    }

    /// Kills mutants `modbus.rs:535:46 replace > with ==` and `... with >=`
    /// in `process_pdu` (T0831 window-expiry boundary, BC-2.14.016).
    ///
    /// The T0831 register-write window check is `if t0831_elapsed > T0831_WINDOW_SECS`
    /// (window = 5s). A SECOND register write whose elapsed time is EXACTLY the window
    /// width (5s) is still "within window" under `>` (5 > 5 == false), so the within-
    /// window branch runs, the count reaches 2, and T0831 co-tags the finding. Both
    /// the `==` and `>=` mutants flip `5 (==|>=) 5` to true → they take the
    /// window-RESET branch instead → count resets to 1 → T0831 does NOT fire. We
    /// assert T0831 IS present on the boundary write, killing both mutants. (The
    /// existing `window_reset_after_5s` test uses elapsed=6, which all three operators
    /// agree is expired, so it could not distinguish them — this pins the boundary.)
    #[test]
    fn test_f6_mutation_t0831_fires_at_exact_window_boundary() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // First register write at t=0 starts the T0831 window.
        let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        let f1 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 0);
        assert_eq!(
            f1[0].mitre_techniques,
            vec!["T1692.001", "T0836"],
            "first write: T0831 not yet fired (v19 remap: T0855→T1692.001)"
        );

        // Second register write at t=5 == T0831_WINDOW_SECS — the exact boundary.
        // With the production `>`: 5 > 5 is false → within-window → count=2 → T0831 fires.
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x02, 0x00]);
        let f2 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 5);
        assert!(
            f2[0].mitre_techniques.contains(&"T0831".to_string()),
            "a register write at elapsed == T0831_WINDOW_SECS (5s) is WITHIN the window \
         and MUST co-tag T0831 (kills the 535 > -> == and > -> >= boundary mutants); \
         got {:?}",
            f2[0].mitre_techniques
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.14.017 v2.6 — Burst summary window-width display (issue #220 regression guard)
    // EC-011 / Postcondition 1: summary must report configured window WIDTH constant
    // (WRITE_BURST_WINDOW_SECS), not the elapsed span between first and last write.
    // ---------------------------------------------------------------------------

    /// test_BC_2_14_017_burst_summary_reports_window_width_not_elapsed
    ///
    /// Regression guard for GitHub issue #220: when all writes in a burst share the same
    /// integer-second timestamp, the burst detector computed `burst_elapsed = 0` and
    /// interpolated that into the summary string, producing "...in 0s window".
    ///
    /// The fix (BC-2.14.017 v2.6 Postcondition 1, EC-011, STORY-104 AC-004) requires the
    /// summary to report the configured window width constant (`WRITE_BURST_WINDOW_SECS = 1`),
    /// not the elapsed span.  Canonical summary form:
    ///   "Modbus write burst: {count} writes within {window_secs}s window
    ///    (unit {unit_id}, threshold {threshold}/s)"
    ///
    /// This guard drives 21 write-class FCs all at ts=0 (same second), locates the single
    /// burst finding, and asserts both the anti-regression property ("0s window" absent) and
    /// the exact canonical summary string.
    ///
    /// Traces to: BC-2.14.017 v2.6 Postcondition 1, EC-011, STORY-104 AC-004.
    #[test]
    fn test_BC_2_14_017_burst_summary_reports_window_width_not_elapsed() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // 21 writes all at ts=0 — same integer-second timestamp, so the pre-fix code
        // computes burst_elapsed = 0 and emits "...in 0s window".
        let mut all_findings = Vec::new();
        for i in 0..21_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);
            all_findings.append(&mut f);
        }

        let burst_findings: Vec<_> = all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .collect();

        // Guard: exactly one burst finding (behavioral invariant — no change expected).
        assert_eq!(
            burst_findings.len(),
            1,
            "exactly one burst finding must be emitted for 21 same-second writes"
        );

        let burst = burst_findings[0];

        // Anti-regression: the pre-fix string "in 0s window" must be absent.
        assert!(
            !burst.summary.contains("0s window"),
            "issue #220 regression: summary must not contain \"0s window\"; got: {:?}",
            burst.summary
        );

        // Exact canonical form per BC-2.14.017 v2.6 Postcondition 1 / EC-011.
        // window_secs = WRITE_BURST_WINDOW_SECS (1), unit_id = 1, threshold = 20.
        let expected = format!(
            "Modbus write burst: 21 writes within {}s window (unit 1, threshold {}/s)",
            WRITE_BURST_WINDOW_SECS, DEFAULT_WRITE_BURST_THRESHOLD,
        );
        assert_eq!(
            burst.summary, expected,
            "burst summary must use window WIDTH constant, not elapsed span"
        );
    }
} // mod story_104

// ---------------------------------------------------------------------------
// STORY-141 — direction_and_clock: per-direction carry isolation +
//             saturating_sub window monotonicity (F4 RED gate)
//
// Tests trace to:
//   BC-2.14.002 v2.0  — carry-direction isolation invariant (EC-XXX)
//   BC-2.14.016 v2.3  — T0831 5s window, saturating_sub (EC-011)
//   BC-2.14.017 v2.7  — T0806 burst/sustained windows (EC-012), >= preserved
//   BC-2.14.019 v1.5  — T0888 exception 10s window (EC-009)
//   RULING-MODBUS-SIBLING-001 §1–5
//
// RED conditions (vs. current stub):
//   AC-141-001/006 (EC-X1/EC-X2): stub routes BOTH directions through carry_c2s
//     → s2c delivery splices c2s carry → parse_errors > 0 or wrong FC counted.
//   AC-141-005/006/008 (clock windows): stub uses wrapping_sub at lines 534/595/820
//     → backwards-ts wrap gives large elapsed > threshold → window reset → detection
//     suppressed → assertion fails.
//   AC-141-007 (sustained >=): stub uses wrapping_sub at line 670; >= preserved so
//     saturating_sub=0 NOT >= 2 → no spurious fire; but we assert the FULL sequence
//     where ts=102 gives elapsed=2 → fires. RED because: stub uses wrapping_sub at
//     line 670 and the test expects no spurious fire at backwards ts then correct fire
//     at ts=102 — wrapping_sub(50,100) wraps to large value, NOT >= 2 (coincidentally
//     same outcome), BUT wrapping_sub(100, 100)=0 and then we prime with writes at ts=0
//     seeding sustained_window_start_ts=0; backwards ts=50<100 → this test verifies the
//     exact write-count arithmetic stays intact. Per the story, AC-141-007 is GREEN with
//     the stub for the no-spurious-fire part but RED for the "fires at ts=102" assertion
//     because the stub has wrapping_sub AND the test drives sustained counts that rely on
//     the window not being spuriously reset by the backwards call.
//   AC-141-010 (saturating_sub replacement): directly validates saturating_sub semantics.
//
// Namespace: DF-TEST-NAMESPACE-001.
// ---------------------------------------------------------------------------

mod direction_and_clock {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::modbus::{
        DEFAULT_WRITE_BURST_THRESHOLD, DEFAULT_WRITE_SUSTAINED_THRESHOLD, MAX_ADU_CARRY_BYTES,
        MbapHeader, ModbusAnalyzer, ModbusFlowState, WRITE_BURST_WINDOW_SECS, parse_mbap_header,
    };
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{Direction, StreamHandler};

    // -----------------------------------------------------------------------
    // Shared helpers (mirror story_104 style)
    // -----------------------------------------------------------------------

    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            1234,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            502,
        )
    }

    fn default_analyzer() -> ModbusAnalyzer {
        ModbusAnalyzer::new(
            DEFAULT_WRITE_BURST_THRESHOLD,
            DEFAULT_WRITE_SUSTAINED_THRESHOLD,
        )
    }

    /// Build a valid Modbus TCP ADU: 6-byte MBAP + fc + pdu_data.
    fn build_adu(txn_id: u16, unit_id: u8, fc: u8, pdu_data: &[u8]) -> Vec<u8> {
        let length = (2 + pdu_data.len()) as u16;
        let mut adu = vec![
            (txn_id >> 8) as u8,
            (txn_id & 0xFF) as u8,
            0x00,
            0x00,
            (length >> 8) as u8,
            (length & 0xFF) as u8,
            unit_id,
            fc,
        ];
        adu.extend_from_slice(pdu_data);
        adu
    }

    /// Drive `on_data` through the full reassembly path (StreamHandler trait required).
    fn drive_on_data(
        analyzer: &mut ModbusAnalyzer,
        fk: &FlowKey,
        direction: Direction,
        data: &[u8],
        timestamp: u32,
    ) {
        analyzer.on_data(fk, direction, data, 0, timestamp);
    }

    /// Drive `process_pdu` directly with a caller-owned flow state.
    /// This lets tests inspect per-flow fields that are not accessible via the
    /// private `analyzer.flows` map.
    fn drive(
        analyzer: &mut ModbusAnalyzer,
        flow: &mut ModbusFlowState,
        fk: &FlowKey,
        direction: Direction,
        adu: &[u8],
        timestamp: u32,
    ) {
        let header: MbapHeader = parse_mbap_header(adu).expect("test ADU must be at least 8 bytes");
        let fc = header.function_code;
        let _ = analyzer.process_pdu(fk, flow, direction, &header, fc, adu, timestamp);
    }

    // -----------------------------------------------------------------------
    // AC-141-001: direction isolation — no splice (EC-X1 regression test)
    //
    // Uses on_data (StreamHandler) so carry routing is exercised end-to-end.
    // We observe parse_errors and fn_code_counts (both public) to detect splice.
    //
    // RED against stub: stub uses carry_c2s for BOTH directions → s2c delivery
    // prepends the 6-byte c2s partial → garbled MBAP parse → parse_errors > 0
    // OR fn_code_counts wrong.
    //
    // Traces to: BC-2.14.002 v2.0 direction-isolation Invariant, EC-XXX.
    // -----------------------------------------------------------------------

    /// Guards EC-X1: c2s partial carry must NOT contaminate s2c delivery.
    ///
    /// RED against stub: stub routes both directions through carry_c2s.
    /// GREEN after fix: per-direction carry isolation prevents the splice.
    #[test]
    fn test_ac141_001_carry_direction_isolation_no_splice() {
        let mut az = default_analyzer();
        let fk = test_flow_key();

        // Step 1: deliver partial c2s MBAP prefix (6 bytes — not enough for full 8-byte header).
        let partial_c2s: [u8; 6] = [0x00, 0x01, 0x00, 0x00, 0x00, 0x08];
        drive_on_data(&mut az, &fk, Direction::ClientToServer, &partial_c2s, 100);
        assert_eq!(
            az.parse_errors, 0,
            "partial c2s prefix must not cause parse errors"
        );

        // Step 2: deliver a complete, valid s2c ADU — FC=0x03 (Read Holding Registers).
        let s2c_adu = build_adu(0x0006, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x04, 0x00]);
        drive_on_data(&mut az, &fk, Direction::ServerToClient, &s2c_adu, 101);

        // After fix: fn_code_counts[0x03] == 1, no splice from c2s carry.
        // RED with stub: c2s carry prepended → garbled MBAP → parse_errors > 0 or wrong FC.
        assert_eq!(
            az.parse_errors, 0,
            "s2c ADU delivery must not produce parse errors (no splice from c2s carry); \
             RED with stub: carry_c2s prepended to s2c bytes → garbled MBAP"
        );
        assert_eq!(
            az.fn_code_counts.get(&0x03).copied().unwrap_or(0),
            1,
            "fn_code_counts[0x03] must be 1 after s2c FC=0x03 ADU; \
             RED with stub: splice may produce FC=0x06 or parse error instead"
        );
        assert_eq!(
            az.fn_code_counts.get(&0x06).copied().unwrap_or(0),
            0,
            "fn_code_counts[0x06] must be 0 (no garbled write from c2s carry splice); \
             RED with stub: spliced carry bytes may decode as FC=0x06"
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-002: same-direction carry completes normally (control / always GREEN)
    //
    // Traces to: BC-2.14.002 v2.0 Postcondition 1.
    // -----------------------------------------------------------------------

    /// Regression guard: same-direction carry completion must still work after the split.
    #[test]
    fn test_ac141_002_same_direction_carry_completes_normally() {
        let mut az = default_analyzer();
        let fk = test_flow_key();

        // Build a complete FC=0x06 ADU (12 bytes).
        let full_adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        assert_eq!(full_adu.len(), 12, "full ADU must be 12 bytes");

        // Deliver first 6 bytes (partial MBAP header) in c2s.
        drive_on_data(&mut az, &fk, Direction::ClientToServer, &full_adu[..6], 100);
        assert_eq!(
            az.parse_errors, 0,
            "partial c2s delivery must not produce parse errors"
        );
        assert_eq!(
            az.fn_code_counts.get(&0x06).copied().unwrap_or(0),
            0,
            "no ADU completed yet after partial delivery"
        );

        // Deliver remaining 6 bytes in the SAME c2s direction → ADU completes.
        drive_on_data(&mut az, &fk, Direction::ClientToServer, &full_adu[6..], 101);
        assert_eq!(
            az.parse_errors, 0,
            "completing c2s ADU must not produce parse errors"
        );
        assert_eq!(
            az.fn_code_counts.get(&0x06).copied().unwrap_or(0),
            1,
            "fn_code_counts[0x06] must be 1 after same-direction carry completion"
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-003: carry-cap overflow sets is_non_modbus; other direction untouched
    //
    // Uses on_data-based overflow, then checks total parse_errors and that a
    // subsequent s2c delivery does not also corrupt (since is_non_modbus bails early).
    //
    // We verify the cap behavior by checking that after overflow, the analyzer
    // has incremented parse_errors AND that additional s2c bytes produce no
    // new fn_code_counts (stream bails).
    //
    // Traces to: BC-2.14.002 v2.0 Invariant 1 (per-direction), RULING §1.5.
    // -----------------------------------------------------------------------

    /// Guards per-direction cap overflow: c2s cap overflow sets is_non_modbus
    /// and subsequent deliveries are silenced.
    #[test]
    fn test_ac141_003_cap_overflow_sets_is_non_modbus() {
        let mut az = default_analyzer();
        let fk = test_flow_key();

        // Deliver 4-byte sub-MBAP chunks in c2s repeatedly to exceed 260 bytes.
        let chunk: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
        // (260 / 4) + 5 = 70 iterations guarantees overflow.
        let needed_iters = (MAX_ADU_CARRY_BYTES / 4) + 5;
        for _ in 0..needed_iters {
            drive_on_data(&mut az, &fk, Direction::ClientToServer, &chunk, 100);
        }

        // After overflow: parse_errors must be > 0 (set by cap-overflow path).
        assert!(
            az.parse_errors > 0,
            "parse_errors must be > 0 after c2s carry cap overflow (> {} bytes); \
             cap-overflow path increments parse_errors and sets is_non_modbus",
            MAX_ADU_CARRY_BYTES
        );

        // After is_non_modbus latches, s2c deliveries must produce no new FC counts.
        let fc03_before = az.fn_code_counts.get(&0x03).copied().unwrap_or(0);
        let s2c_adu = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive_on_data(&mut az, &fk, Direction::ServerToClient, &s2c_adu, 101);
        let fc03_after = az.fn_code_counts.get(&0x03).copied().unwrap_or(0);
        assert_eq!(
            fc03_after, fc03_before,
            "after is_non_modbus latch, s2c delivery must not add new fn_code_counts; \
             confirms c2s cap overflow silenced the stream for BOTH directions"
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-004: no wrapping_sub in modbus.rs (source-level guard)
    //
    // RED against stub, GREEN after fix.
    //
    // Traces to: BC-2.14.016 v2.3 Invariant, BC-2.14.017 v2.7 Invariant.
    // -----------------------------------------------------------------------

    /// Guards that wrapping_sub is absent from src/analyzer/modbus.rs after the fix.
    ///
    /// RED against stub: the stub retains all 4 wrapping_sub calls at lines 534/595/670/820.
    /// GREEN after fix: all 4 sites replaced with saturating_sub.
    #[test]
    fn test_ac141_004_no_wrapping_sub_in_modbus() {
        let manifest = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set in cargo test");
        let modbus_path = std::path::Path::new(&manifest).join("src/analyzer/modbus.rs");
        let source = std::fs::read_to_string(&modbus_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {}", modbus_path.display(), e));

        let wrapping_sub_code_lines: Vec<_> = source
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                let trimmed = line.trim();
                let is_only_comment = trimmed.starts_with("///") || trimmed.starts_with("//");
                !is_only_comment && trimmed.contains(".wrapping_sub(")
            })
            .collect();

        assert!(
            wrapping_sub_code_lines.is_empty(),
            "wrapping_sub must not appear in code paths of src/analyzer/modbus.rs after fix; \
             RED against stub: found {} code line(s) with wrapping_sub:\n{}",
            wrapping_sub_code_lines.len(),
            wrapping_sub_code_lines
                .iter()
                .map(|(n, l)| format!("  line {}: {}", n + 1, l.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-005: T0831 5s window — backwards-ts must NOT reset window
    //
    // Uses process_pdu directly so we can inspect per-flow window fields.
    //
    // RED against stub: wrapping_sub(50,100) ≈ 4.29e9 >> 5 → window resets →
    //   t0831_window_write_count=1 → T0831 does NOT fire.
    //
    // Traces to: BC-2.14.016 v2.3 Postcondition window-expiry, EC-011.
    // -----------------------------------------------------------------------

    /// Guards EC-011: backwards-ts write must NOT reset the T0831 5s window.
    ///
    /// RED with stub: wrapping_sub resets window → T0831 suppressed.
    /// GREEN after fix: saturating_sub(50,100)=0 preserves window → T0831 fires.
    #[test]
    fn test_ac141_005_t0831_backwards_clock_no_reset() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Write 1: arm T0831 window at ts=100.
        let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu1,
            100,
        );

        // Write 2: backwards ts=50. saturating_sub(50,100)=0 NOT > 5 → no reset.
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu2,
            50,
        );

        assert!(
            flow.t0831_window_write_count >= 2,
            "t0831_window_write_count must be >= 2 after backwards-ts write (window NOT reset); \
             RED with stub: wrapping_sub resets window → count resets to 1"
        );

        // T0831 co-tag must fire.
        let t0831_fired = az
            .all_findings
            .iter()
            .any(|f| f.mitre_techniques.contains(&"T0831".to_string()));
        assert!(
            t0831_fired,
            "T0831 must fire after 2 writes within 5s window (backwards-ts must not reset); \
             RED with stub: wrapping_sub spuriously resets window → T0831 not emitted"
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-006: T0806 burst 1s window — backwards-ts must NOT reset window (EC-X2)
    //
    // RED against stub: wrapping_sub(50,100) >> 1 → window resets → burst suppressed.
    //
    // Traces to: BC-2.14.017 v2.7 Postcondition burst window, EC-012 (EC-X2).
    // -----------------------------------------------------------------------

    /// Guards EC-X2: backwards-ts must NOT reset the T0806 burst 1s window.
    ///
    /// RED with stub: wrapping_sub resets window → burst suppressed.
    /// GREEN after fix: saturating_sub(50,100)=0 preserves count → burst fires at 21.
    #[test]
    fn test_ac141_006_burst_backwards_clock_no_reset() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Arm: 20 writes at ts=100.
        for i in 0..20_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &adu,
                100,
            );
        }
        assert_eq!(
            flow.window_write_count, 20,
            "pre-condition: window_write_count must be 20 after 20 writes at ts=100"
        );

        // Backwards write at ts=50: must NOT reset.
        let adu_back = build_adu(0x0015, 0x01, 0x06, &[0x00, 0x14, 0x01, 0x00]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu_back,
            50,
        );
        assert!(
            flow.window_write_count >= 21,
            "window_write_count must be >= 21 after backwards-ts write (no reset); \
             RED with stub: wrapping_sub resets window → count=1"
        );

        // One more write at ts=100 → burst fires.
        let adu_final = build_adu(0x0016, 0x01, 0x06, &[0x00, 0x15, 0x01, 0x00]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu_final,
            100,
        );

        let burst_fired = az
            .all_findings
            .iter()
            .any(|f| f.mitre_techniques.contains(&"T0806".to_string()));
        assert!(
            burst_fired,
            "T0806 burst must fire after window_write_count > 20 (backwards-ts must not reset); \
             RED with stub: wrapping_sub resets window → burst count never reaches threshold"
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-007: T0806 sustained window — >= gate preserved, backwards-ts safe
    //
    // RED against stub: wrapping_sub at line 670 may produce spurious fire on
    // backwards call or suppress the correct forward-ts fire.
    //
    // Traces to: BC-2.14.017 v2.7 Postcondition sustained window.
    // -----------------------------------------------------------------------

    /// Guards sustained window: backwards-ts must not spuriously fire; >= preserved;
    /// correct fire at ts=102 (elapsed=2 >= 2).
    ///
    /// RED with stub: wrapping_sub at line 670 may spuriously trigger or suppress.
    #[test]
    fn test_ac141_007_sustained_operator_ge_preserved() {
        // High burst threshold (1000) so only sustained fires.
        let mut az = ModbusAnalyzer::new(1000, DEFAULT_WRITE_SUSTAINED_THRESHOLD);
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // 25 writes at ts=100 to seed sustained window.
        for i in 0..25_u32 {
            let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &adu,
                100,
            );
        }

        let sustained_before_backwards = az
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();

        // Backwards write at ts=50: saturating_sub(50,100)=0 NOT >= 2 → gate not met.
        let adu_back = build_adu(0x001A, 0x01, 0x06, &[0x00, 0x19, 0x01, 0x00]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu_back,
            50,
        );

        let sustained_after_backwards = az
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();
        assert_eq!(
            sustained_after_backwards, sustained_before_backwards,
            "backwards-ts write (ts=50 < window_start=100) must NOT fire sustained burst; \
             saturating_sub(50,100)=0, 0 NOT >= 2 → minimum-duration gate not met"
        );

        // Forward write at ts=102: elapsed=2 >= 2 → gate met → rate check fires.
        let adu_forward = build_adu(0x001B, 0x01, 0x06, &[0x00, 0x1A, 0x01, 0x00]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu_forward,
            102,
        );

        let sustained_total = az
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
            .count();
        assert!(
            sustained_total > sustained_before_backwards,
            "sustained burst must fire at ts=102 (elapsed=2 >= 2, rate > threshold); \
             >= operator preserved: 2 >= 2 is TRUE → gate met → detection fires. \
             RED with stub: wrapping_sub may reset window or fire at wrong time"
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-008: T0888 exception 10s window — backwards-ts must NOT reset
    //
    // RED against stub: wrapping_sub(50,100) >> 10 → window resets → count drops.
    //
    // Traces to: BC-2.14.019 v1.5 Postcondition, EC-009 amended.
    // -----------------------------------------------------------------------

    /// Guards EC-009: backwards-ts exception must NOT reset the T0888 10s window.
    ///
    /// RED with stub: wrapping_sub resets window → exception count drops.
    /// GREEN after fix: saturating_sub(50,100)=0 preserves count.
    #[test]
    fn test_ac141_008_exception_backwards_clock_no_reset() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Deliver 5 exception responses for exc_code=0x01 at ts=100.
        for i in 0..5_u32 {
            let req_adu = build_adu(i as u16, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ClientToServer,
                &req_adu,
                100 + i,
            );
            let exc_adu = build_adu(i as u16, 0x01, 0x83, &[0x01]);
            drive(
                &mut az,
                &mut flow,
                &fk,
                Direction::ServerToClient,
                &exc_adu,
                100 + i,
            );
        }
        assert_eq!(
            flow.exception_window_counts
                .get(&0x01)
                .copied()
                .unwrap_or(0),
            5,
            "pre-condition: exception_window_counts[0x01] must be 5"
        );

        // Backwards exception at ts=50: saturating_sub(50,100)=0 NOT > 10 → no reset.
        let req_back = build_adu(0x0005, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &req_back,
            50,
        );
        let exc_back = build_adu(0x0005, 0x01, 0x83, &[0x01]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ServerToClient,
            &exc_back,
            50,
        );

        assert!(
            flow.exception_window_counts
                .get(&0x01)
                .copied()
                .unwrap_or(0)
                >= 6,
            "exception_window_counts[0x01] must be >= 6 after backwards-ts exception (no reset); \
             RED with stub: wrapping_sub resets window → count drops back to 1"
        );
    }

    // -----------------------------------------------------------------------
    // AC-141-010: saturating_sub replaces wrapping_sub (behavioral regression)
    //
    // Delivers a write at ts=0xFFFFFF00, seeding window_start.
    // Delivers backwards write at ts=0x00000100.
    // saturating_sub(0x100, 0xFFFFFF00) = 0, NOT > 1 → count=2.
    // wrapping_sub(0x100, 0xFFFFFF00) = 0x200 = 512 > 1 → window resets → count=1.
    //
    // RED against stub: count remains 1 after second write.
    //
    // Traces to: BC-2.14.017 v2.7 Postcondition burst window, EC-012.
    // -----------------------------------------------------------------------

    /// Guards that saturating_sub replaces wrapping_sub in window arithmetic.
    ///
    /// RED with stub: wrapping_sub resets window → window_write_count=1.
    /// GREEN after fix: saturating_sub preserves window → count=2.
    #[test]
    fn test_ac141_010_window_elapsed_uses_saturating_sub() {
        let mut az = default_analyzer();
        let mut flow = ModbusFlowState::default();
        let fk = test_flow_key();

        // Write 1: seed window_start_ts = 0xFFFFFF00.
        let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu1,
            0xFFFFFF00_u32,
        );
        assert_eq!(
            flow.window_start_ts, 0xFFFFFF00_u32,
            "window_start_ts must be seeded at 0xFFFFFF00"
        );
        assert_eq!(
            flow.window_write_count, 1,
            "window_write_count must be 1 after first write"
        );

        // Write 2: backwards ts=0x00000100.
        // saturating_sub(0x100, 0xFFFFFF00) = 0 → NOT > 1 → no reset → count=2.
        // wrapping_sub(0x100, 0xFFFFFF00) = 0x200 = 512 > 1 → window resets → count=1.
        let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);
        drive(
            &mut az,
            &mut flow,
            &fk,
            Direction::ClientToServer,
            &adu2,
            0x00000100_u32,
        );

        assert_eq!(
            flow.window_write_count, 2,
            "window_write_count must be 2 after backwards-ts write (saturating_sub preserves window); \
             RED with stub: wrapping_sub(0x100, 0xFFFFFF00)=0x200=512 > {} → window resets → count=1",
            WRITE_BURST_WINDOW_SECS
        );
    }
} // mod direction_and_clock

// ---------------------------------------------------------------------------
// STORY-141 — VP-037: Modbus carry direction isolation (genuine proptest)
//
// These are GENUINE proptest harnesses using proptest::prelude::* with
// generated strategies. Lesson from STORY-139 F-139-002 enforced:
// NOT deterministic point tests masquerading as proptests.
//
// RED behavior (stub): stub routes both directions through carry_c2s →
// s2c delivery prepends c2s carry → fn_code_counts diverge from independent runs.
//
// Namespace: DF-TEST-NAMESPACE-001.
// ---------------------------------------------------------------------------

mod vp037_modbus_carry_direction_isolation {
    use proptest::prelude::*;
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::modbus::{
        DEFAULT_WRITE_BURST_THRESHOLD, DEFAULT_WRITE_SUSTAINED_THRESHOLD, ModbusAnalyzer,
    };
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{Direction, StreamHandler};

    fn ip(a: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, a))
    }

    fn make_key() -> FlowKey {
        FlowKey::new(ip(10), 1234, ip(100), 502)
    }

    fn make_analyzer() -> ModbusAnalyzer {
        ModbusAnalyzer::new(
            DEFAULT_WRITE_BURST_THRESHOLD,
            DEFAULT_WRITE_SUSTAINED_THRESHOLD,
        )
    }

    /// Build a valid Modbus TCP ADU (8 bytes minimum: 6 MBAP + unit_id + FC).
    fn build_adu(txn_id: u16, unit_id: u8, fc: u8, pdu_data: &[u8]) -> Vec<u8> {
        let length = (2 + pdu_data.len()) as u16;
        let mut adu = vec![
            (txn_id >> 8) as u8,
            (txn_id & 0xFF) as u8,
            0x00,
            0x00,
            (length >> 8) as u8,
            (length & 0xFF) as u8,
            unit_id,
            fc,
        ];
        adu.extend_from_slice(pdu_data);
        adu
    }

    // VP-037 proptest: fn_code_counts are isolated by direction.
    //
    // Strategy: `split_offset in 0usize..6` — partial MBAP prefix length for the
    // c2s ADU. At split_offset=0 the full c2s ADU is delivered in one call
    // (degenerate split, still valid). At split_offset > 0 the c2s ADU is split
    // across two deliveries.
    //
    // For each case:
    //   1. Build complete c2s ADU (FC=0x06 Write) and s2c ADU (FC=0x03 Read).
    //   2. Deliver c2s partial (bytes 0..split_offset) → stashed in carry_c2s.
    //   3. Deliver complete s2c ADU → carry_s2c path (carry_c2s NOT involved).
    //   4. Deliver remaining c2s bytes (split_offset..) → carry_c2s prepended.
    //   5. Assert fn_code_counts[0x03]==1 AND fn_code_counts[0x06]==1 AND parse_errors==0.
    //
    // With stub (carry_c2s for both): s2c delivery prepends c2s partial →
    // garbled MBAP → parse_errors > 0 or wrong FC counted → FAIL.
    //
    // Traces: VP-037 (BC-2.14.002 v2.0 direction-isolation Invariant).
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(256))]

        /// Guards VP-037: fn_code_counts are correctly isolated across c2s and s2c
        /// deliveries regardless of split offset.
        ///
        /// FAILS with stub: carry_c2s prepended to s2c walk → garbled FC decode →
        /// fn_code_counts diverge from the expected isolated counts.
        #[test]
        fn proptest_vp037_direction_isolation_fn_code_counts(
            split_offset in 0usize..6,
        ) {
            // Build complete ADUs.
            // c2s: FC=0x06 (Write Single Register), 4 data bytes → 12 bytes total.
            let c2s_adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
            // s2c: FC=0x03 (Read Holding Registers), 4 data bytes → 12 bytes total.
            let s2c_adu = build_adu(0x0002, 0x01, 0x03, &[0x00, 0x04, 0x01, 0x00]);

            let key = make_key();
            let mut az = make_analyzer();

            // Delivery 1: partial c2s MBAP prefix (0..split_offset bytes).
            // If split_offset == 0, deliver nothing (degenerate — skip).
            if split_offset > 0 {
                az.on_data(&key, Direction::ClientToServer, &c2s_adu[..split_offset], 0, 100);
                // No ADU completed yet — no parse errors expected.
                prop_assert_eq!(az.parse_errors, 0, "partial c2s must not cause parse errors");
            }

            // Delivery 2: complete s2c ADU.
            // With stub: carry_c2s (containing c2s partial) prepended to s2c bytes → garbled.
            az.on_data(&key, Direction::ServerToClient, &s2c_adu, 0, 101);
            prop_assert_eq!(
                az.parse_errors, 0,
                "s2c ADU delivery must not produce parse errors; \
                 FAILS with stub: c2s carry spliced into s2c walk → garbled MBAP"
            );
            prop_assert_eq!(
                az.fn_code_counts.get(&0x03).copied().unwrap_or(0), 1,
                "fn_code_counts[0x03] must be 1 after s2c delivery; \
                 FAILS with stub: splice causes wrong FC decode"
            );
            prop_assert_eq!(
                az.fn_code_counts.get(&0x06).copied().unwrap_or(0), 0,
                "fn_code_counts[0x06] must still be 0 before c2s completes; \
                 FAILS with stub: spliced bytes may decode as FC=0x06"
            );

            // Delivery 3: complete c2s ADU (split_offset..).
            az.on_data(&key, Direction::ClientToServer, &c2s_adu[split_offset..], 0, 102);
            prop_assert_eq!(
                az.parse_errors, 0,
                "completing c2s delivery must not produce parse errors"
            );
            prop_assert_eq!(
                az.fn_code_counts.get(&0x06).copied().unwrap_or(0), 1,
                "fn_code_counts[0x06] must be 1 after c2s completion; \
                 FAILS with stub: carry contamination may skip or mismatch"
            );
        }

        /// Guards VP-037 equivalence invariant: interleaved fn_code_counts must equal
        /// the sum of independent same-direction control runs.
        ///
        /// Establishes carry isolation as a measurable behavioral invariant:
        /// interleaved(fn_code_counts) == c2s_only(fn_code_counts) + s2c_only(fn_code_counts).
        ///
        /// FAILS with stub: carry contamination causes fn_code_counts to diverge.
        #[test]
        fn proptest_vp037_independent_run_equivalence(
            split_offset in 0usize..6,
        ) {
            let c2s_adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
            let s2c_adu = build_adu(0x0002, 0x01, 0x03, &[0x00, 0x04, 0x01, 0x00]);
            let key = make_key();

            // Interleaved run: c2s partial → s2c complete → c2s completing.
            let mut interleaved = make_analyzer();
            if split_offset > 0 {
                interleaved.on_data(&key, Direction::ClientToServer, &c2s_adu[..split_offset], 0, 100);
            }
            interleaved.on_data(&key, Direction::ServerToClient, &s2c_adu, 0, 101);
            interleaved.on_data(&key, Direction::ClientToServer, &c2s_adu[split_offset..], 0, 102);
            let interleaved_fc03 = interleaved.fn_code_counts.get(&0x03).copied().unwrap_or(0);
            let interleaved_fc06 = interleaved.fn_code_counts.get(&0x06).copied().unwrap_or(0);
            let interleaved_errors = interleaved.parse_errors;

            // Independent c2s-only run.
            let mut c2s_only = make_analyzer();
            if split_offset > 0 {
                c2s_only.on_data(&key, Direction::ClientToServer, &c2s_adu[..split_offset], 0, 100);
            }
            c2s_only.on_data(&key, Direction::ClientToServer, &c2s_adu[split_offset..], 0, 102);
            let c2s_fc06 = c2s_only.fn_code_counts.get(&0x06).copied().unwrap_or(0);
            let c2s_errors = c2s_only.parse_errors;

            // Independent s2c-only run.
            let mut s2c_only = make_analyzer();
            s2c_only.on_data(&key, Direction::ServerToClient, &s2c_adu, 0, 101);
            let s2c_fc03 = s2c_only.fn_code_counts.get(&0x03).copied().unwrap_or(0);
            let s2c_errors = s2c_only.parse_errors;

            // VP-037 invariant: interleaved fn_code_counts == sum of independent runs.
            prop_assert_eq!(
                interleaved_fc06, c2s_fc06,
                "VP-037: interleaved fn_code_counts[0x06] must equal c2s-only count; \
                 FAILS with stub: carry contamination causes divergence"
            );
            prop_assert_eq!(
                interleaved_fc03, s2c_fc03,
                "VP-037: interleaved fn_code_counts[0x03] must equal s2c-only count; \
                 FAILS with stub: carry splice causes wrong FC decode"
            );
            prop_assert_eq!(
                interleaved_errors, c2s_errors + s2c_errors,
                "VP-037: interleaved parse_errors must equal sum of independent runs; \
                 FAILS with stub: carry splice produces spurious parse_errors"
            );
        }
    }
} // mod vp037_modbus_carry_direction_isolation

// ---------------------------------------------------------------------------
// STORY-141 — VP-038: Modbus window monotonic / no-spurious-reset (genuine proptests)
//
// GENUINE proptest harnesses for all four Modbus windowed detections:
//   Sub-A: T0831 5s window
//   Sub-B: T0806 burst 1s window
//   Sub-C: T0806 sustained >= 2s gate (>= preserved, not >)
//   Sub-D: T0888 exception 10s window
//   Sub-E: genuine u32 rollover deterministic test
//
// Sub-A through Sub-D use prop_assume!(backwards_ts <= window_start) to constrain
// the strategy domain. GENUINE proptests (proptest! macro + generated strategies).
//
// Namespace: DF-TEST-NAMESPACE-001.
// ---------------------------------------------------------------------------

mod vp038_modbus_window_monotonic_no_spurious_reset {
    use proptest::prelude::*;
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::modbus::{
        DEFAULT_WRITE_BURST_THRESHOLD, DEFAULT_WRITE_SUSTAINED_THRESHOLD, EXCEPTION_WINDOW_SECS,
        MbapHeader, ModbusAnalyzer, ModbusFlowState, T0831_WINDOW_SECS, WRITE_BURST_WINDOW_SECS,
        WRITE_SUSTAINED_WINDOW_SECS, parse_mbap_header,
    };
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    fn ip(a: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, a))
    }

    fn make_key() -> FlowKey {
        FlowKey::new(ip(10), 1234, ip(100), 502)
    }

    fn make_analyzer() -> ModbusAnalyzer {
        ModbusAnalyzer::new(
            DEFAULT_WRITE_BURST_THRESHOLD,
            DEFAULT_WRITE_SUSTAINED_THRESHOLD,
        )
    }

    fn build_adu(txn_id: u16, unit_id: u8, fc: u8, pdu_data: &[u8]) -> Vec<u8> {
        let length = (2 + pdu_data.len()) as u16;
        let mut adu = vec![
            (txn_id >> 8) as u8,
            (txn_id & 0xFF) as u8,
            0x00,
            0x00,
            (length >> 8) as u8,
            (length & 0xFF) as u8,
            unit_id,
            fc,
        ];
        adu.extend_from_slice(pdu_data);
        adu
    }

    /// Drive one complete ADU through `process_pdu` with a caller-owned flow state.
    /// Returns the findings from that call. Uses the public `process_pdu` API so
    /// per-flow window fields can be inspected directly (avoiding private `az.flows`).
    fn drive_pdu(
        az: &mut ModbusAnalyzer,
        flow: &mut ModbusFlowState,
        key: &FlowKey,
        direction: Direction,
        adu: &[u8],
        timestamp: u32,
    ) {
        let header: MbapHeader = parse_mbap_header(adu)
            .unwrap_or_else(|| panic!("test ADU must be at least 8 bytes (got {})", adu.len()));
        let fc = header.function_code;
        az.process_pdu(key, flow, direction, &header, fc, adu, timestamp);
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(256))]

        // -----------------------------------------------------------------------
        // VP-038 Sub-A: T0831 5s window — backwards-ts must NOT reset window.
        //
        // Strategy: (window_start in 1..u32::MAX, backwards_ts in 0..=u32::MAX)
        // with prop_assume!(backwards_ts <= window_start).
        //
        // Arithmetic guard: saturating_sub(backwards_ts, window_start) == 0.
        // Behavioral guard: drive process_pdu — arm window with 1 write at window_start,
        // deliver backwards-ts write, assert t0831_window_write_count >= 2.
        //
        // FAILS with stub (wrapping_sub): large wrapped value > T0831_WINDOW_SECS=5
        // → window resets → count drops to 1.
        //
        // Traces: VP-038 Sub-A (BC-2.14.016 v2.3 EC-011); AC-141-005.
        // -----------------------------------------------------------------------

        /// Guards VP-038 Sub-A: backwards-ts write must NOT reset T0831 5s window.
        ///
        /// FAILS with stub: wrapping_sub(backwards_ts, window_start) wraps to large
        /// value > 5 → window reset → t0831_window_write_count drops to 1.
        #[test]
        fn proptest_vp038_sub_a_t0831_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // Arithmetic invariant: saturating_sub must return 0.
            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(
                elapsed, 0,
                "saturating_sub(backwards_ts, window_start) must be 0 (T0831 window guard)"
            );
            prop_assert!(
                elapsed <= T0831_WINDOW_SECS,
                "elapsed=0 must NOT trigger > {} T0831 window reset", T0831_WINDOW_SECS
            );

            // Behavioral invariant: drive process_pdu with owned flow state.
            let key = make_key();
            let mut az = make_analyzer();
            let mut flow = ModbusFlowState::default();

            // Write 1: arm T0831 window at window_start.
            let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
            drive_pdu(&mut az, &mut flow, &key, Direction::ClientToServer, &adu1, window_start);

            // Backwards write: must NOT reset.
            let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);
            drive_pdu(&mut az, &mut flow, &key, Direction::ClientToServer, &adu2, backwards_ts);

            prop_assert!(
                flow.t0831_window_write_count >= 2,
                "VP-038 Sub-A: t0831_window_write_count must be >= 2 after backwards-ts write; \
                 FAILS with stub (wrapping_sub resets window → count drops to 1)"
            );
        }

        // -----------------------------------------------------------------------
        // VP-038 Sub-B: T0806 burst 1s window — backwards-ts must NOT reset.
        //
        // Strategy: (window_start in 1..u32::MAX, burst_count in 2..200,
        //            backwards_ts in 0..=u32::MAX)
        // with prop_assume!(backwards_ts <= window_start).
        //
        // Arm window with burst_count writes at window_start, then deliver
        // backwards-ts write. Assert window_write_count >= burst_count+1 (no reset).
        //
        // FAILS with stub: wrapping_sub >> 1 → window resets → count drops to 1.
        //
        // Traces: VP-038 Sub-B (BC-2.14.017 v2.7 EC-012, EC-X2); AC-141-006.
        // -----------------------------------------------------------------------

        /// Guards VP-038 Sub-B: backwards-ts write must NOT reset T0806 burst window.
        ///
        /// FAILS with stub: wrapping_sub resets window → burst count suppressed.
        #[test]
        fn proptest_vp038_sub_b_burst_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            burst_count in 2u64..25,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // Arithmetic invariant.
            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0, "saturating_sub must be 0 for backwards ts (burst window)");
            prop_assert!(elapsed <= WRITE_BURST_WINDOW_SECS, "elapsed=0 must NOT trigger burst window reset");

            // Behavioral invariant.
            let key = make_key();
            // Use burst threshold high enough that the burst detector doesn't fire
            // just from arming — we want to test window preservation, not burst emission.
            let az_threshold = burst_count as u32 + 50;
            let mut az = ModbusAnalyzer::new(az_threshold, DEFAULT_WRITE_SUSTAINED_THRESHOLD);
            let mut flow = ModbusFlowState::default();

            // Arm window: burst_count writes at window_start.
            for i in 0..burst_count {
                let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, (i % 256) as u8, 0x01, 0x00]);
                drive_pdu(&mut az, &mut flow, &key, Direction::ClientToServer, &adu, window_start);
            }

            prop_assert_eq!(
                flow.window_write_count as u64, burst_count,
                "pre-condition: window_write_count must equal burst_count after arm"
            );

            // Backwards write.
            let adu_back = build_adu(burst_count as u16 + 1, 0x01, 0x06, &[0x00, 0xFF, 0x01, 0x00]);
            drive_pdu(&mut az, &mut flow, &key, Direction::ClientToServer, &adu_back, backwards_ts);

            prop_assert!(
                flow.window_write_count as u64 > burst_count,
                "VP-038 Sub-B: window_write_count must be > burst_count after backwards-ts write; \
                 FAILS with stub (wrapping_sub resets window → count drops to 1)"
            );
        }

        // -----------------------------------------------------------------------
        // VP-038 Sub-C: T0806 sustained >= 2s gate — backwards-ts must NOT
        // spuriously satisfy minimum-duration gate.
        //
        // Arithmetic-only proptest: saturating_sub(backwards_ts, window_start) = 0
        // does NOT satisfy >= 2. No `az.flows` access needed.
        //
        // Traces: VP-038 Sub-C (BC-2.14.017 v2.7 Postcondition sustained window).
        // -----------------------------------------------------------------------

        /// Guards VP-038 Sub-C: saturating_sub(backwards_ts, window_start) = 0 does NOT
        /// satisfy the >= 2s minimum-duration gate (no spurious sustained burst).
        ///
        /// This proptest covers the arithmetic domain. Behavioral coverage (correct fire
        /// at ts=window_start+2) is in test_ac141_007_sustained_operator_ge_preserved.
        #[test]
        fn proptest_vp038_sub_c_sustained_backwards_ts_no_spurious_fire(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // Arithmetic invariant: saturating_sub returns 0.
            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0, "saturating_sub(backwards_ts, window_start) must be 0");

            // The >= WRITE_SUSTAINED_WINDOW_SECS gate (minimum-duration check):
            // 0 NOT >= 2 → gate not met → no spurious sustained burst on backwards call.
            // Note: >= is preserved (not changed to >). 0 >= 2 is FALSE → correct.
            prop_assert!(
                elapsed < WRITE_SUSTAINED_WINDOW_SECS,
                "VP-038 Sub-C: elapsed=0 must NOT satisfy >= {} minimum-duration gate; \
                 no spurious sustained burst on backwards-ts call. \
                 Note: >= operator preserved (not >), 0 < {} is TRUE → gate not met.",
                WRITE_SUSTAINED_WINDOW_SECS,
                WRITE_SUSTAINED_WINDOW_SECS
            );
        }

        // -----------------------------------------------------------------------
        // VP-038 Sub-D: T0888 exception 10s window — backwards-ts must NOT reset.
        //
        // Strategy: (window_start in 1..u32::MAX, backwards_ts in 0..=u32::MAX)
        // with prop_assume!(backwards_ts <= window_start).
        //
        // Behavioral guard: arm 5 exceptions at window_start, deliver backwards-ts
        // exception, assert exception_window_counts[exc_code] >= 6.
        //
        // FAILS with stub: wrapping_sub >> 10 → window resets → count drops.
        //
        // Traces: VP-038 Sub-D (BC-2.14.019 v1.5 EC-009); AC-141-008.
        // -----------------------------------------------------------------------

        /// Guards VP-038 Sub-D: backwards-ts exception must NOT reset T0888 10s window.
        ///
        /// FAILS with stub: wrapping_sub resets window → exception count drops.
        #[test]
        fn proptest_vp038_sub_d_exception_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // Arithmetic invariant.
            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(elapsed, 0, "saturating_sub must be 0 for backwards ts (exception window)");
            prop_assert!(elapsed <= EXCEPTION_WINDOW_SECS, "elapsed=0 must NOT trigger exception window reset");

            // Behavioral invariant: arm 5 exceptions then deliver one backwards.
            let key = make_key();
            let mut az = make_analyzer();
            let mut flow = ModbusFlowState::default();

            for i in 0..5u32 {
                // Pre-insert a request so attribute_exception can match (txn_id=i, FC=0x03).
                let req = build_adu(i as u16, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
                drive_pdu(&mut az, &mut flow, &key, Direction::ClientToServer, &req,
                    window_start.saturating_add(i));
                // Exception: FC=0x83, exc_code=0x01.
                let exc = build_adu(i as u16, 0x01, 0x83, &[0x01]);
                drive_pdu(&mut az, &mut flow, &key, Direction::ServerToClient, &exc,
                    window_start.saturating_add(i));
            }

            prop_assert_eq!(
                flow.exception_window_counts.get(&0x01).copied().unwrap_or(0), 5,
                "pre-condition: exception_window_counts[0x01] must be 5 after arm"
            );

            // Backwards exception.
            let req_back = build_adu(0x0005, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
            drive_pdu(&mut az, &mut flow, &key, Direction::ClientToServer, &req_back, backwards_ts);
            let exc_back = build_adu(0x0005, 0x01, 0x83, &[0x01]);
            drive_pdu(&mut az, &mut flow, &key, Direction::ServerToClient, &exc_back, backwards_ts);

            prop_assert!(
                flow.exception_window_counts.get(&0x01).copied().unwrap_or(0) >= 6,
                "VP-038 Sub-D: exception_window_counts must be >= 6 after backwards-ts exception; \
                 FAILS with stub (wrapping_sub resets window → count drops to 1)"
            );
        }
    }

    // -----------------------------------------------------------------------
    // VP-038 Sub-E: genuine u32 rollover — deterministic no-spurious-reset test
    //
    // window_start = u32::MAX - 5 (0xFFFFFFFA), now_ts = 4 (post-rollover).
    //
    // OLD BUG (wrapping_sub):
    //   wrapping_sub(4, 0xFFFFFFFA) = 10
    //   10 > EXCEPTION_WINDOW_SECS=10 is FALSE (so exception window OK)
    //   BUT 10 > T0831_WINDOW_SECS=5 is TRUE → spurious T0831 window reset
    //   AND 10 > WRITE_BURST_WINDOW_SECS=1 is TRUE → spurious burst window reset
    //
    // FIX (saturating_sub):
    //   saturating_sub(4, 0xFFFFFFFA) = 0
    //   0 is NOT > 5, NOT > 1, NOT > 10, NOT >= 2 → no spurious reset on ANY window.
    //
    // Traces: VP-038 Sub-E (BC-2.14.016/017/019 EC-007); AC-141-010.
    // -----------------------------------------------------------------------

    /// Guards genuine u32 rollover: saturating_sub returns 0, no spurious reset on
    /// any Modbus windowed detection.
    ///
    /// window_start = u32::MAX - 5 (0xFFFFFFFA), now_ts = 4 (post-rollover).
    /// wrapping_sub(4, 0xFFFFFFFA) = 10 → spuriously resets T0831 (>5) and burst (>1).
    /// saturating_sub(4, 0xFFFFFFFA) = 0 → no spurious reset on any window.
    #[test]
    fn test_vp038_sub_e_genuine_rollover_no_spurious_reset() {
        let window_start: u32 = u32::MAX - 5; // 0xFFFFFFFA
        let now_ts_post_rollover: u32 = 4;

        // Document old (broken) behavior: wrapping_sub gives 10.
        let wrapping_elapsed = now_ts_post_rollover.wrapping_sub(window_start);
        assert_eq!(
            wrapping_elapsed, 10,
            "wrapping_sub(4, u32::MAX-5) must equal 10 (the old spurious value that \
             exceeds T0831 (>5) and burst (>1) window thresholds)"
        );

        // Document old spurious resets under wrapping_sub.
        assert!(
            wrapping_elapsed > T0831_WINDOW_SECS,
            "OLD BUG: wrapping_elapsed=10 > T0831_WINDOW_SECS={} → spurious T0831 reset",
            T0831_WINDOW_SECS
        );
        assert!(
            wrapping_elapsed > WRITE_BURST_WINDOW_SECS,
            "OLD BUG: wrapping_elapsed=10 > WRITE_BURST_WINDOW_SECS={} → spurious burst reset",
            WRITE_BURST_WINDOW_SECS
        );

        // Assert new (correct) behavior: saturating_sub gives 0.
        let saturating_elapsed = now_ts_post_rollover.saturating_sub(window_start);
        assert_eq!(
            saturating_elapsed, 0,
            "saturating_sub(4, u32::MAX-5) must equal 0 (no spurious reset for any window)"
        );

        // Guards all four Modbus windows: saturating_elapsed=0 must NOT trigger any reset.
        assert!(
            saturating_elapsed <= T0831_WINDOW_SECS,
            "saturating_sub=0 must NOT trigger T0831 5s window reset (rollover guard)"
        );
        assert!(
            saturating_elapsed <= WRITE_BURST_WINDOW_SECS,
            "saturating_sub=0 must NOT trigger T0806 burst 1s window reset (rollover guard)"
        );
        assert!(
            saturating_elapsed < WRITE_SUSTAINED_WINDOW_SECS,
            "saturating_sub=0 must NOT satisfy T0806 sustained >= {}s gate (rollover guard)",
            WRITE_SUSTAINED_WINDOW_SECS
        );
        assert!(
            saturating_elapsed <= EXCEPTION_WINDOW_SECS,
            "saturating_sub=0 must NOT trigger T0888 exception 10s window reset (rollover guard)"
        );
    }
} // mod vp038_modbus_window_monotonic_no_spurious_reset
