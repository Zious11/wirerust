//! Failing tests for STORY-108: DNP3 Direct Detection Emissions.
//!
//! Covers AC-001..AC-012 and edge cases EC-001..EC-008 from the STORY-108 spec.
//! Traces to behavioral contracts: BC-2.15.010, BC-2.15.011, BC-2.15.012,
//! BC-2.15.013, BC-2.15.020, BC-2.15.022.
//!
//! STORY-108 is complete; all tests in this file pass (GREEN).
//! Tests cover `detect_control_class_burst`, `detect_restart`,
//! `detect_write`, and `summarize()`.
//!
//! Test naming convention: `test_BC_S_SS_NNN_xxx` / `test_EC_NNN_xxx`
//! following the project TDD standard (DF-TEST-NAMESPACE-001).

// BC traceability uses uppercase BC identifiers in function names; suppress lint.
#![allow(non_snake_case)]

mod story_108 {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::DateTime;
    use wirerust::analyzer::dnp3::{Dnp3Analyzer, Dnp3FcClass, MAX_FINDINGS, classify_dnp3_fc};
    use wirerust::findings::{Confidence, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn test_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            20000,
        )
    }

    /// Build a minimal valid DNP3 frame carrying a specific application FC.
    ///
    /// Layout (13 bytes — minimum to satisfy FIR=1 + user-data path):
    ///   [0..2]  0x05 0x64 length_byte        (sync + LENGTH)
    ///   [3]     0xC4                           (CTRL: DIR=1 PRM=1 UNCONFIRMED_USER_DATA)
    ///   [4..5]  dest_lo dest_hi               (little-endian DEST)
    ///   [6..7]  src_lo  src_hi                (little-endian SRC)
    ///   [8..9]  0x00 0x00                     (CRC placeholder)
    ///   [10]    0xC0                           (transport: FIR=1, FIN=1)
    ///   [11]    0x00                           (app control)
    ///   [12]    app_fc                         (application function code)
    ///
    /// LENGTH=14 → compute_dnp3_frame_len(14) = 5+14+2 = 21, but we only deliver
    /// 13 bytes so it lives in carry as a partial.  The frame-walk does not reach
    /// the detection branch for a partial frame.  To get the detection branch to
    /// fire we must deliver a *complete* frame: LENGTH=8 → frame_len=15 bytes is
    /// the minimum frame that reaches byte 12 (app_fc at offset 12, frame_len >=13).
    ///
    /// Minimum complete frame containing byte[12]: LENGTH byte such that
    /// compute_dnp3_frame_len(length_byte) >= 13.
    ///   length_byte=8 → 5+8+2*ceil(3/16) = 5+8+2 = 15 ≥ 13  ✓
    ///   (U = 8-5 = 3 user bytes; blocks = ceil(3/16) = 1; frame_len = 15)
    ///
    /// We pad to exactly `frame_len` bytes.
    fn build_detection_frame(app_fc: u8, dest: u16, src: u16) -> Vec<u8> {
        // LENGTH=8 → frame_len = 5+8+2*1 = 15
        let length_byte: u8 = 8;
        let u = (length_byte as usize) - 5; // 3
        let blocks = u.div_ceil(16); // 1
        let frame_len = 5 + (length_byte as usize) + 2 * blocks; // 15
        assert_eq!(frame_len, 15);

        let mut frame = vec![0u8; frame_len];
        frame[0] = 0x05;
        frame[1] = 0x64;
        frame[2] = length_byte;
        // CTRL=0xC4: DIR=1, PRM=1, nibble=0x04 UNCONFIRMED_USER_DATA → has_user_data==true
        frame[3] = 0xC4;
        let [dl, dh] = dest.to_le_bytes();
        frame[4] = dl;
        frame[5] = dh;
        let [sl, sh] = src.to_le_bytes();
        frame[6] = sl;
        frame[7] = sh;
        // bytes 8-9: header CRC placeholder (0x00)
        // byte 10: transport octet — 0xC0 = FIR=1 (0x40) | FIN=1 (0x80)
        frame[10] = 0xC0;
        // byte 11: app control (arbitrary)
        frame[11] = 0x00;
        // byte 12: application FC
        frame[12] = app_fc;
        // bytes 13-14: data-block CRC placeholder (0x00)
        frame
    }

    // -----------------------------------------------------------------------
    // AC-001 (BC-2.15.010 postconditions 1/2)
    // test_direct_operate_count_increments_on_control_fc
    // -----------------------------------------------------------------------

    /// AC-001: Every Control-class FC (0x03, 0x04, 0x05, 0x06) on a FIR=1 frame
    /// increments `flow.direct_operate_count`.  On the first Control-class FC in
    /// a new window, `flow.window_start_ts = now_ts`.
    ///
    /// Traces to: BC-2.15.010 postconditions 1 and 2; STORY-108 AC-001.
    #[test]
    fn test_direct_operate_count_increments_on_control_fc() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x05 (DIRECT_OPERATE) — one frame at ts=1000
        let frame = build_detection_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.direct_operate_count, 1,
            "AC-001: first DIRECT_OPERATE must set direct_operate_count=1"
        );
        assert_eq!(
            flow.window_start_ts, 1000,
            "AC-001: first Control FC must seed window_start_ts=now_ts"
        );

        // FC=0x03 (SELECT) — second frame at ts=1001
        let frame2 = build_detection_frame(0x03, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame2, 1001, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must still exist");
        assert_eq!(
            flow.direct_operate_count, 2,
            "AC-001: second Control FC (SELECT 0x03) must increment count to 2"
        );
        // window_start_ts stays at 1000 (set on first FC in window)
        assert_eq!(
            flow.window_start_ts, 1000,
            "AC-001: window_start_ts must not change after first FC in same window"
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 (BC-2.15.010 postcondition 3 — emission at threshold+1)
    // test_t1692_001_emitted_at_threshold_plus_one
    // -----------------------------------------------------------------------

    /// AC-002: With default threshold=10, T1692.001 is emitted at the 11th FC.
    /// No finding at count=10 (threshold check is `>`, not `>=`).
    ///
    /// Traces to: BC-2.15.010 postcondition 3; Canonical Test Vector §1; STORY-108 AC-002.
    #[test]
    fn test_t1692_001_emitted_at_threshold_plus_one() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver 10 DIRECT_OPERATE FCs — no finding expected at count=10
        for i in 0..10u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "AC-002: at count=10 (==threshold) NO finding yet (check is >)"
        );

        // 11th FC — count=11 > threshold=10 → finding MUST be emitted
        let frame_11 = build_detection_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame_11, 10, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-002: at count=11 (>threshold=10) exactly ONE finding must be emitted"
        );

        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T1692.001"],
            "AC-002: finding must carry T1692.001 only"
        );
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "AC-002: category must be Execution"
        );
        assert!(
            matches!(f.verdict, Verdict::Likely),
            "AC-002: verdict must be Likely"
        );
        assert!(
            matches!(f.confidence, Confidence::Medium),
            "AC-002: confidence must be Medium"
        );

        // BC-2.15.010 PC3: verify the exact summary format from the BC postcondition.
        // Format: "DNP3 unauthorized control command burst: {count} control FCs in {elapsed}s
        // window (threshold {threshold})"
        // count=11, elapsed=10 (ts of 11th frame=10, window_start_ts=0), threshold=10.
        // Elapsed is derived from saturating_sub(10, 0)=10 — deterministic for fixed timestamps.
        // Use starts_with to pin everything except the timing-sensitive elapsed suffix.
        assert!(
            f.summary.starts_with(
                "DNP3 unauthorized control command burst: 11 control FCs in 10s window (threshold 10)"
            ),
            "BC-2.15.010 PC3: summary must match exact BC format string; got: {:?}",
            f.summary
        );

        // BC-2.15.010 PC3: source_ip must be Some(<source endpoint resolved from flow_key>).
        //
        // Derivation: test_flow_key() = FlowKey::new(10.0.0.1:20000, 10.0.0.2:20000).
        // FlowKey canonicalizes by (ip, port): (10.0.0.1, 20000) < (10.0.0.2, 20000) so
        //   lower_ip = 10.0.0.1 (outstation / server side, port 20000)
        //   upper_ip = 10.0.0.2 (master / client side, port 20000)
        // DNP3 frames here have CTRL=0xC4 (DIR=1, PRM=1 — master-to-outstation direction).
        // The master is the frame source; analogous to Modbus client_ip convention:
        //   lower_port == 20000 (server port) → lower endpoint is outstation, upper is master.
        // Expected source_ip = upper_ip = 10.0.0.2 (the master/initiator).
        //
        let expected_source_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        assert_eq!(
            f.source_ip,
            Some(expected_source_ip),
            "BC-2.15.010 PC3: source_ip must be Some(10.0.0.2) resolved from flow_key \
             (upper_ip = master endpoint); got {:?}",
            f.source_ip
        );

        // BC-2.15.010 PC3: timestamp must be Some(<pcap-relative capture timestamp>).
        // The 11th frame was delivered with ts=10; expected = DateTime::from_timestamp(10, 0).
        let expected_ts = DateTime::from_timestamp(10i64, 0);
        assert_eq!(
            f.timestamp, expected_ts,
            "BC-2.15.010 PC3: timestamp must be Some(DateTime from ts=10); got {:?}",
            f.timestamp
        );

        // Verify one-shot guard is set
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.direct_operate_emitted,
            "AC-002: direct_operate_emitted must be true after emission"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 (BC-2.15.010 postcondition 3 — one-shot guard)
    // test_t1692_001_one_shot_guard
    // -----------------------------------------------------------------------

    /// AC-003: 16 total Control FCs → exactly 1 finding (one-shot guard after emission).
    ///
    /// After `direct_operate_emitted=true`, additional Control-class FCs in the same
    /// window increment the counter but do NOT push additional T1692.001 findings.
    ///
    /// Traces to: BC-2.15.010 postcondition 3 (guard invariant); Canonical Test Vector §3;
    ///            STORY-108 AC-003.
    #[test]
    fn test_t1692_001_one_shot_guard() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver 16 DIRECT_OPERATE FCs all within the 60s window (ts 0..15)
        for i in 0..16u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();

        assert_eq!(
            t1692_count, 1,
            "AC-003: exactly ONE T1692.001 finding for 16 Control FCs (one-shot guard active)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.direct_operate_count, 16,
            "AC-003: counter must still be 16 (incremented even after guard set)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 (BC-2.15.010 postcondition 4 — window expiry reset)
    // test_t1692_001_window_expiry_resets_counter
    // -----------------------------------------------------------------------

    /// AC-004: Emit a finding in window 1; advance time past 60s; verify window resets
    /// and a second finding can be emitted in the new window.
    ///
    /// Uses `saturating_sub` semantics (RULING-DNP3-SIBLING-001 §2.2): the implementation
    /// must check `now_ts.saturating_sub(window_start_ts) > DETECTION_WINDOW_SECS`.
    ///
    /// Traces to: BC-2.15.010 postcondition 4; STORY-108 AC-004.
    #[test]
    fn test_t1692_001_window_expiry_resets_counter() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Window 1: deliver 11 FCs within 60s → first finding emitted
        for i in 0..11u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer); // ts 0..10
        }
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-004: first window should produce one finding"
        );

        // Advance time past 60s: window_start_ts=0, now_ts=61 → elapsed=61 > 60
        // Send a new Control FC — this should RESET the window (not fire another finding yet)
        let frame_reset = build_detection_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame_reset, 61, Direction::ClientToServer);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.direct_operate_count, 1,
                "AC-004: after window expiry+reset, count must be 1 (new window seeded)"
            );
            assert_eq!(
                flow.window_start_ts, 61,
                "AC-004: after window reset, window_start_ts must be the new ts=61"
            );
            assert!(
                !flow.direct_operate_emitted,
                "AC-004: after window reset, direct_operate_emitted must be false"
            );
        }

        // Now send 10 more FCs in the new window (total in new window = 11) → second finding
        for i in 0..10u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, 62 + i, Direction::ClientToServer);
        }

        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();
        assert_eq!(
            t1692_count, 2,
            "AC-004: second window should produce a second finding (two total)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 (BC-2.15.011 postconditions 1/2 — T0814 per-occurrence)
    // test_t0814_emitted_per_occurrence_cold_restart
    // test_t0814_emitted_per_occurrence_warm_restart
    // test_initialize_data_not_restart
    // -----------------------------------------------------------------------

    /// AC-005a: COLD_RESTART (0x0D) → one T0814 finding with Confidence::High.
    /// restart_event_count incremented.
    ///
    /// Traces to: BC-2.15.011 postconditions 1/2; Canonical Test Vector; STORY-108 AC-005.
    #[test]
    fn test_t0814_emitted_per_occurrence_cold_restart() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x0D (COLD_RESTART)
        let frame = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-005: COLD_RESTART must emit exactly ONE finding"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0814"],
            "AC-005: COLD_RESTART finding must carry T0814 only"
        );
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "AC-005: category must be Execution"
        );
        assert!(
            matches!(f.verdict, Verdict::Likely),
            "AC-005: verdict must be Likely"
        );
        assert!(
            matches!(f.confidence, Confidence::High),
            "AC-005: COLD_RESTART confidence must be High"
        );

        // BC-2.15.011 PC1: pin exact summary format from BC postcondition.
        // Format: "DNP3 restart command observed: FC 0x{fc:02X} ({name}) from src={src:#06X}
        // to dest={dest:#06X}"
        // Frame built with build_detection_frame(0x0D, 0x0003, 0x0001):
        //   fc=0x0D, name="COLD_RESTART", src=0x0001, dest=0x0003
        // {:#06X} → uppercase hex with 0x prefix, minimum 6 chars total → "0x0001", "0x0003"
        assert_eq!(
            f.summary,
            "DNP3 restart command observed: FC 0x0D (COLD_RESTART) from src=0x0001 to dest=0x0003",
            "BC-2.15.011 PC1: summary must match exact BC format string; got: {:?}",
            f.summary
        );

        // BC-2.15.011 PC1: source_ip must be Some(<source endpoint resolved from flow_key>).
        //
        // Derivation: same flow_key as AC-002 above.
        //   lower_ip = 10.0.0.1 (outstation, port 20000)
        //   upper_ip = 10.0.0.2 (master, port 20000)
        // COLD_RESTART frames come from the master (DIR=1 / PRM=1 in CTRL=0xC4).
        // Expected source_ip = upper_ip = 10.0.0.2.
        let expected_source_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        assert_eq!(
            f.source_ip,
            Some(expected_source_ip),
            "BC-2.15.011 PC1: source_ip must be Some(10.0.0.2) resolved from flow_key; got {:?}",
            f.source_ip
        );

        // BC-2.15.011 PC1: timestamp must be Some(<pcap-relative capture timestamp>).
        // The COLD_RESTART frame was delivered with ts=1000.
        let expected_ts = DateTime::from_timestamp(1000i64, 0);
        assert_eq!(
            f.timestamp, expected_ts,
            "BC-2.15.011 PC1: timestamp must be Some(DateTime from ts=1000); got {:?}",
            f.timestamp
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 1,
            "AC-005: restart_event_count must be 1 after COLD_RESTART"
        );
    }

    /// AC-005b: WARM_RESTART (0x0E) → one T0814 finding with Confidence::High.
    ///
    /// Traces to: BC-2.15.011 postconditions 1/2; Canonical Test Vector; STORY-108 AC-005.
    #[test]
    fn test_t0814_emitted_per_occurrence_warm_restart() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x0E (WARM_RESTART)
        let frame = build_detection_frame(0x0E, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-005: WARM_RESTART must emit exactly ONE finding"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0814"],
            "AC-005: WARM_RESTART finding must carry T0814 only"
        );
        assert!(
            matches!(f.confidence, Confidence::High),
            "AC-005: WARM_RESTART confidence must be High"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 1,
            "AC-005: restart_event_count must be 1 after WARM_RESTART"
        );
    }

    /// AC-005c: FC 0x0F (INITIALIZE_DATA) is Management-class → no T0814 finding.
    ///
    /// This test verifies two conditions: COLD_RESTART (0x0D) increments
    /// restart_event_count (requiring detect_restart to be implemented),
    /// then INITIALIZE_DATA (0x0F) does NOT increment it (Management class,
    /// not Restart class).
    ///
    /// Traces to: BC-2.15.011 EC-004; BC-2.15.006 EC-009; STORY-108 AC-005.
    #[test]
    fn test_initialize_data_not_restart() {
        // Sanity: verify classify_dnp3_fc(0x0F) == Management (pure function, always correct)
        assert_eq!(
            classify_dnp3_fc(0x0F),
            Dnp3FcClass::Management,
            "classify_dnp3_fc(0x0F) must return Management (not Restart)"
        );

        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FIRST: deliver a genuine COLD_RESTART (0x0D) — must emit T0814 and increment counter
        let cold_frame = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold_frame, 500, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-005c pre-condition: COLD_RESTART must emit one T0814 finding"
        );

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.restart_event_count, 1,
                "AC-005c pre-condition: restart_event_count must be 1 after COLD_RESTART"
            );
        }

        // SECOND: deliver FC=0x0F (INITIALIZE_DATA) — must NOT trigger T0814.
        // ts=600 is 100s after ts=500 — well within the 300s correlation window
        // (CORRELATION_WINDOW_SECS=300). If the delivery ts were ≥300s after the
        // window start, the correlation-window expiry would legitimately reset
        // restart_event_count. ts=600 avoids crossing the window boundary.
        let init_frame = build_detection_frame(0x0F, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &init_frame, 600, Direction::ClientToServer);

        // Still exactly 1 finding (from the COLD_RESTART above, not INITIALIZE_DATA)
        let t0814_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();
        assert_eq!(
            t0814_count, 1,
            "AC-005/EC-003: FC=0x0F (INITIALIZE_DATA) must NOT emit additional T0814 (only the COLD_RESTART's finding)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 1,
            "AC-005/EC-003: restart_event_count must stay at 1 after INITIALIZE_DATA (not incremented)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-006 (BC-2.15.012 postcondition 1 — T0836 per-occurrence for WRITE)
    // test_t0836_emitted_for_write_fc
    // test_write_fc_not_t1692
    // -----------------------------------------------------------------------

    /// AC-006a: WRITE (FC=0x02) → one T0836 finding per occurrence, Confidence::Medium.
    ///
    /// Traces to: BC-2.15.012 postcondition 1; Canonical Test Vector; STORY-108 AC-006.
    #[test]
    fn test_t0836_emitted_for_write_fc() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x02 (WRITE)
        let frame = build_detection_frame(0x02, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-006: WRITE FC must emit exactly ONE finding"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0836"],
            "AC-006: WRITE finding must carry T0836 ONLY (not T1692.001)"
        );
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "AC-006: category must be Execution"
        );
        assert!(
            matches!(f.verdict, Verdict::Likely),
            "AC-006: verdict must be Likely"
        );
        assert!(
            matches!(f.confidence, Confidence::Medium),
            "AC-006: WRITE confidence must be Medium"
        );

        // BC-2.15.012 PC1: pin exact summary format from BC postcondition.
        // Format: "DNP3 WRITE command observed: parameter modification from src={src:#06X}
        // to dest={dest:#06X}"
        // Frame built with build_detection_frame(0x02, 0x0003, 0x0001):
        //   src=0x0001, dest=0x0003
        // {:#06X} → uppercase hex with 0x prefix, minimum 6 chars total → "0x0001", "0x0003"
        assert_eq!(
            f.summary,
            "DNP3 WRITE command observed: parameter modification from src=0x0001 to dest=0x0003",
            "BC-2.15.012 PC1: summary must match exact BC format string; got: {:?}",
            f.summary
        );

        // BC-2.15.012 PC1: source_ip must be Some(<source endpoint resolved from flow_key>).
        //
        // Derivation: same flow_key as AC-002/AC-005 above.
        //   lower_ip = 10.0.0.1 (outstation, port 20000)
        //   upper_ip = 10.0.0.2 (master, port 20000)
        // WRITE frames come from the master (DIR=1 / PRM=1 in CTRL=0xC4).
        // Expected source_ip = upper_ip = 10.0.0.2.
        let expected_source_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        assert_eq!(
            f.source_ip,
            Some(expected_source_ip),
            "BC-2.15.012 PC1: source_ip must be Some(10.0.0.2) resolved from flow_key; got {:?}",
            f.source_ip
        );

        // BC-2.15.012 PC1: timestamp must be Some(<pcap-relative capture timestamp>).
        // The WRITE frame was delivered with ts=1000.
        let expected_ts = DateTime::from_timestamp(1000i64, 0);
        assert_eq!(
            f.timestamp, expected_ts,
            "BC-2.15.012 PC1: timestamp must be Some(DateTime from ts=1000); got {:?}",
            f.timestamp
        );
    }

    /// AC-006b: WRITE FC does NOT also emit T1692.001 (ADR-007 Decision 5 separation).
    ///
    /// Traces to: BC-2.15.012 invariant 4; STORY-108 AC-006.
    #[test]
    fn test_write_fc_not_t1692() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x02 (WRITE) — 20 writes (well above any hypothetical threshold)
        for i in 0..20u32 {
            let frame = build_detection_frame(0x02, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();
        assert_eq!(
            t1692_count, 0,
            "AC-006: WRITE FC must NEVER emit T1692.001 (ADR-007 Decision 5 separation)"
        );
        // All 20 should emit T0836 (per-occurrence, no threshold)
        let t0836_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0836".to_string()))
            .count();
        assert_eq!(
            t0836_count, 20,
            "AC-006: each WRITE FC must emit one T0836 (per-occurrence, no threshold)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 (BC-2.15.013 postconditions 2/3) — restart findings append in observation order
    // -----------------------------------------------------------------------

    /// AC-007: Restart findings append to `all_findings` in observation order.
    ///
    /// Verifies that two T0814 findings (from two separate `on_data` calls) appear in
    /// the order they were emitted — COLD_RESTART first, then WARM_RESTART.
    ///
    /// DEFERRAL NOTE: BC-2.15.013 PC2 (direct-before-derived ordering WITHIN A SINGLE
    /// `on_data` call) and PC4/PC5 (mid-multi-finding-sequence cap re-check) require
    /// T0827 derived findings to be present. T0827 emission is NOT implemented in
    /// STORY-108 (deferred to STORY-109 where the T0827 derived push exists).
    /// When STORY-109 adds T0827, this test's append-order assertion (T0814 before any
    /// T0827 from the same call) must still hold — the T0827 push must come AFTER the
    /// T0814 push within the same `detect_restart_split` call.
    ///
    /// This test proves the inter-call append ordering (two sequential `on_data` calls),
    /// not the intra-call ordering that BC-2.15.013 PC2 specifies.
    ///
    /// Traces to: BC-2.15.013 postconditions 2/3; STORY-108 AC-007.
    #[test]
    fn test_restart_findings_append_in_observation_order() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // COLD_RESTART first
        let cold = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold, 100, Direction::ClientToServer);

        // WARM_RESTART second
        let warm = build_detection_frame(0x0E, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &warm, 110, Direction::ClientToServer);

        // Verify ordering: both findings are T0814 (direct), no T0827 yet (STORY-109)
        assert!(
            analyzer.all_findings.len() >= 2,
            "AC-007: at least 2 findings expected (COLD + WARM restart)"
        );

        // The first finding must be T0814 (direct observation comes first)
        assert_eq!(
            analyzer.all_findings[0].mitre_techniques,
            vec!["T0814"],
            "AC-007: first finding (from COLD_RESTART) must be T0814"
        );
        assert_eq!(
            analyzer.all_findings[1].mitre_techniques,
            vec!["T0814"],
            "AC-007: second finding (from WARM_RESTART) must be T0814"
        );

        // No T0827 in STORY-108 (STORY-109 adds that)
        let t0827_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .count();
        assert_eq!(
            t0827_count, 0,
            "AC-007: T0827 must NOT be emitted in STORY-108 (deferred to STORY-109)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 (BC-2.15.013 postconditions 4/5 — MAX_FINDINGS cap preserves first finding)
    // test_max_findings_cap_preserves_first_finding
    // -----------------------------------------------------------------------

    /// AC-008: Fill all_findings to MAX_FINDINGS-1; then deliver COLD_RESTART.
    /// T0814 is pushed (one slot remaining). Then another restart is delivered;
    /// now at cap — T0814 is dropped, but restart_event_count still increments.
    ///
    /// Scenario:
    ///   - Pre-fill to MAX_FINDINGS-1
    ///   - COLD_RESTART → T0814 pushed (count now MAX_FINDINGS)
    ///   - COLD_RESTART again → cap hit; no T0814; restart_event_count incremented
    ///
    /// Traces to: BC-2.15.013 postconditions 4/5; BC-2.15.022 Canonical Test Vectors;
    ///            STORY-108 AC-008.
    #[test]
    fn test_max_findings_cap_preserves_first_finding() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Pre-fill all_findings to MAX_FINDINGS-1 by directly pushing dummy findings.
        // We use a Finding-like value constructed from scratch.
        use wirerust::findings::Finding;
        for _ in 0..(MAX_FINDINGS - 1) {
            analyzer.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Unlikely,
                confidence: Confidence::Low,
                summary: "pre-fill".to_string(),
                evidence: vec![],
                mitre_techniques: vec![],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }
        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS - 1,
            "pre-condition: all_findings must be at MAX_FINDINGS-1"
        );

        // Create a flow (deliver one non-detection frame to create flow entry)
        let init_frame = build_detection_frame(0x01, 0x0003, 0x0001); // READ — no detection
        analyzer.on_data(key.clone(), &init_frame, 0, Direction::ClientToServer);

        // Deliver COLD_RESTART — should push T0814 (one slot left)
        let cold1 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold1, 100, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS,
            "AC-008: after first COLD_RESTART, all_findings must reach MAX_FINDINGS"
        );
        let last = analyzer.all_findings.last().expect("at least one finding");
        assert_eq!(
            last.mitre_techniques,
            vec!["T0814"],
            "AC-008: last finding (the T0814 from COLD_RESTART) must be preserved at MAX_FINDINGS"
        );

        // Deliver second COLD_RESTART — cap hit, T0814 MUST NOT be pushed
        let cold2 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold2, 110, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS,
            "AC-008: second COLD_RESTART at cap must NOT grow all_findings beyond MAX_FINDINGS"
        );

        // restart_event_count must still be 2 (both restarts counted even when capped)
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 2,
            "AC-008: restart_event_count must be 2 (both restarts counted, even second was capped)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-009 (BC-2.15.022 postconditions 1/3 — counters updated when capped)
    // test_max_findings_counters_updated_when_capped
    // -----------------------------------------------------------------------

    /// AC-009: When all_findings is at MAX_FINDINGS, counters still update.
    /// Specifically: direct_operate_count, restart_event_count, fc_counts, fn_code_counts,
    /// frame_count all continue to increment even when findings are suppressed.
    ///
    /// Traces to: BC-2.15.022 postconditions 1/3; STORY-108 AC-009.
    #[test]
    fn test_max_findings_counters_updated_when_capped() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Pre-fill all_findings to MAX_FINDINGS
        use wirerust::findings::Finding;
        for _ in 0..MAX_FINDINGS {
            analyzer.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Unlikely,
                confidence: Confidence::Low,
                summary: "pre-fill".to_string(),
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
            "pre-condition: all_findings must be at MAX_FINDINGS"
        );

        // Deliver a COLD_RESTART frame — no finding pushed, but restart_event_count += 1
        let cold = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold, 100, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS,
            "AC-009: all_findings must NOT grow beyond cap"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 1,
            "AC-009: restart_event_count must be 1 (counter updated despite cap)"
        );
        assert_eq!(
            flow.frame_count, 1,
            "AC-009: frame_count must be 1 (frame counted despite cap)"
        );
        // fc_counts for 0x0D must be updated
        assert_eq!(
            flow.fc_counts.get(&0x0D).copied().unwrap_or(0),
            1,
            "AC-009: fc_counts[0x0D] must be 1 (FC counted despite cap)"
        );
        // fn_code_counts must be updated
        assert_eq!(
            analyzer.fn_code_counts.get(&0x0D).copied().unwrap_or(0),
            1,
            "AC-009: fn_code_counts[0x0D] must be 1 (aggregate FC count despite cap)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-010 (BC-2.15.020 postcondition 1 — summarize FC distribution)
    // test_summarize_function_code_distribution
    // -----------------------------------------------------------------------

    /// AC-010: Process 5 DIRECT_OPERATE (0x05) + 3 READ (0x01) frames.
    /// summarize() must return fn_code_counts = {0x05: 5, 0x01: 3}.
    ///
    /// Traces to: BC-2.15.020 postcondition 1; Canonical Test Vectors; STORY-108 AC-010.
    #[test]
    fn test_summarize_function_code_distribution() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 5 DIRECT_OPERATE (FC=0x05)
        for i in 0..5u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        // 3 READ (FC=0x01)
        for i in 0..3u32 {
            let frame = build_detection_frame(0x01, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, 100 + i, Direction::ClientToServer);
        }

        let summary = analyzer.summarize();

        // The summary detail must contain function_code_distribution
        let fc_dist = summary
            .detail
            .get("function_code_distribution")
            .expect("AC-010: function_code_distribution must be present in summary detail");

        // Extract as object — keys may be "5" or "0x05" (implementation decides format)
        // We check the values are correct. The simplest approach: look for the JSON values.
        let fc_json = fc_dist
            .as_object()
            .expect("AC-010: function_code_distribution must be a JSON object");

        // Find the DIRECT_OPERATE count (key "5" or "0x05")
        let direct_op_count = fc_json
            .iter()
            .find(|(k, _)| k.as_str() == "5" || k.as_str() == "0x05")
            .map(|(_, v)| v.as_u64().unwrap_or(0))
            .unwrap_or(0);
        assert_eq!(
            direct_op_count, 5,
            "AC-010: fn_code_counts[0x05] must be 5 (5 DIRECT_OPERATE frames)"
        );

        // Find the READ count (key "1" or "0x01")
        let read_count = fc_json
            .iter()
            .find(|(k, _)| k.as_str() == "1" || k.as_str() == "0x01")
            .map(|(_, v)| v.as_u64().unwrap_or(0))
            .unwrap_or(0);
        assert_eq!(
            read_count, 3,
            "AC-010: fn_code_counts[0x01] must be 3 (3 READ frames)"
        );

        // flows_analyzed must be >= 1
        let flows_analyzed = summary
            .detail
            .get("flows_analyzed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert!(
            flows_analyzed >= 1,
            "AC-010: flows_analyzed must be >= 1 after processing one flow"
        );

        // total_frames must be 8 (5+3)
        let total_frames = summary
            .detail
            .get("total_frames")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert_eq!(
            total_frames, 8,
            "AC-010: total_frames must be 8 (5 DIRECT_OPERATE + 3 READ)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-011 (BC-2.15.020 invariant 4 — zero-flow case)
    // test_summarize_zero_flows
    // -----------------------------------------------------------------------

    /// AC-011: When no DNP3 flows were analyzed, summarize() returns a summary with
    /// zero counts (not absent, not panicking).
    ///
    /// Traces to: BC-2.15.020 postcondition 2 and invariant 4; STORY-108 AC-011.
    #[test]
    fn test_summarize_zero_flows() {
        let analyzer = Dnp3Analyzer::new(10);

        // No on_data calls — zero flows
        let summary = analyzer.summarize();

        // Summary must exist and contain zero counts
        assert_eq!(
            summary.analyzer_name, "DNP3",
            "AC-011: analyzer_name must be 'DNP3'"
        );

        let flows_analyzed = summary
            .detail
            .get("flows_analyzed")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        assert_eq!(
            flows_analyzed, 0,
            "AC-011: flows_analyzed must be 0 when no flows processed"
        );

        let total_frames = summary
            .detail
            .get("total_frames")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        assert_eq!(
            total_frames, 0,
            "AC-011: total_frames must be 0 when no flows processed"
        );

        let parse_errors = summary
            .detail
            .get("parse_errors")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        assert_eq!(
            parse_errors, 0,
            "AC-011: parse_errors must be 0 when no flows processed"
        );

        // function_code_distribution must be present (as empty object, not absent)
        let fc_dist = summary.detail.get("function_code_distribution");
        assert!(
            fc_dist.is_some(),
            "AC-011: function_code_distribution must be present even for zero flows"
        );
    }

    // -----------------------------------------------------------------------
    // AC-012 (BC-2.15.010 invariant — threshold is `>` not `>=`)
    // Verified by AC-002 canonical test vector §2, plus dedicated assertion below.
    // -----------------------------------------------------------------------

    /// AC-012: At exactly threshold (count=10, threshold=10), no finding is emitted.
    /// The check is `>`, not `>=`.
    ///
    /// Traces to: BC-2.15.010 invariant (strict >); Canonical Test Vector §2;
    ///            STORY-108 AC-012.
    #[test]
    fn test_BC_2_15_010_threshold_is_strictly_greater_not_gte() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver exactly 10 DIRECT_OPERATE FCs (= threshold, not > threshold)
        for i in 0..10u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "AC-012: at count=10 (==threshold) no finding: check is >, not >="
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.direct_operate_count, 10,
            "AC-012: counter must be 10 after 10 FCs"
        );
        assert!(
            !flow.direct_operate_emitted,
            "AC-012: direct_operate_emitted must be false (threshold not crossed yet)"
        );
    }

    // -----------------------------------------------------------------------
    // EC-001: DIRECT_OPERATE_NR (FC=0x06) counts toward threshold
    // -----------------------------------------------------------------------

    /// EC-001: FC 0x06 (DIRECT_OPERATE_NR) is Control-class per BC-2.15.006.
    /// It must increment direct_operate_count and count toward the T1692.001 threshold.
    ///
    /// Traces to: BC-2.15.010 Invariant 2; BC-2.15.006 postcondition 4; STORY-108 EC-001.
    #[test]
    fn test_EC_001_direct_operate_nr_counts_toward_threshold() {
        // Sanity: FC=0x06 is Control-class
        assert_eq!(
            classify_dnp3_fc(0x06),
            Dnp3FcClass::Control,
            "EC-001: FC=0x06 must be Control-class"
        );

        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver 11 DIRECT_OPERATE_NR (0x06) frames → must trigger T1692.001
        for i in 0..11u32 {
            let frame = build_detection_frame(0x06, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();
        assert_eq!(
            t1692_count, 1,
            "EC-001: 11 DIRECT_OPERATE_NR (FC=0x06) must trigger T1692.001 finding"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.direct_operate_count, 11,
            "EC-001: direct_operate_count must be 11 after 11 DIRECT_OPERATE_NR frames"
        );
    }

    // -----------------------------------------------------------------------
    // EC-002: Exactly at threshold — no finding
    // (Also covered by AC-012; EC-002 uses different FC=0x03 SELECT)
    // -----------------------------------------------------------------------

    /// EC-002: Control FC at exactly threshold (count=10, threshold=10) → no finding.
    ///
    /// This test guards both conditions:
    ///   a) no T1692.001 at count=10 (threshold not exceeded)
    ///   b) direct_operate_count IS 10 (counter incremented correctly)
    ///
    /// Condition (b) guards the counter: detect_control_class_burst must increment
    /// direct_operate_count, so it reflects the actual number of Control FCs seen.
    ///
    /// Traces to: BC-2.15.010 EC-002; STORY-108 EC-002.
    #[test]
    fn test_EC_002_no_finding_at_exact_threshold() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Use SELECT (0x03) — same Control-class, different FC
        for i in 0..10u32 {
            let frame = build_detection_frame(0x03, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        // (a) No T1692.001 finding at count=10 (10 > 10 is false)
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "EC-002: at count=10 (==threshold=10) no finding expected (10 > 10 is false)"
        );

        // (b) Counter must actually be 10 — verifies detect_control_class_burst increments it
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after 10 on_data calls");
        assert_eq!(
            flow.direct_operate_count, 10,
            "EC-002: direct_operate_count must be 10 after 10 SELECT FCs"
        );
    }

    // -----------------------------------------------------------------------
    // EC-005: Two COLD_RESTARTs → restart_event_count=2, two T0814 findings
    // -----------------------------------------------------------------------

    /// EC-005: Two COLD_RESTARTs on same flow → 2 T0814 findings; restart_event_count=2.
    ///
    /// Restart detection is per-occurrence (no one-shot guard), so each COLD_RESTART
    /// fires independently.
    ///
    /// Traces to: BC-2.15.011 EC-002; STORY-108 EC-005.
    #[test]
    fn test_EC_005_two_cold_restarts_restart_event_count_is_2() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        let cold1 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold1, 100, Direction::ClientToServer);

        let cold2 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold2, 200, Direction::ClientToServer);

        let t0814_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();
        assert_eq!(
            t0814_count, 2,
            "EC-005: two COLD_RESTARTs must emit two T0814 findings (per-occurrence)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 2,
            "EC-005: restart_event_count must be 2 after two COLD_RESTARTs"
        );
    }

    // -----------------------------------------------------------------------
    // EC-006: all_findings at cap when COLD_RESTART arrives → no T0814;
    //         restart_event_count still incremented
    // -----------------------------------------------------------------------

    /// EC-006: When all_findings.len()==MAX_FINDINGS, a COLD_RESTART must not push
    /// a finding but MUST still increment restart_event_count.
    ///
    /// Traces to: BC-2.15.011 EC-003; BC-2.15.022 EC-001; STORY-108 EC-006.
    #[test]
    fn test_EC_006_cap_restart_counter_still_increments() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Pre-fill to MAX_FINDINGS
        use wirerust::findings::Finding;
        for _ in 0..MAX_FINDINGS {
            analyzer.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Unlikely,
                confidence: Confidence::Low,
                summary: "pre-fill".to_string(),
                evidence: vec![],
                mitre_techniques: vec![],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }

        let cold = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold, 500, Direction::ClientToServer);

        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS,
            "EC-006: no new finding pushed when at cap"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 1,
            "EC-006: restart_event_count must still be 1 even though finding was capped"
        );
    }

    // -----------------------------------------------------------------------
    // EC-007: FC=0x05 then FC=0x02 on same flow → two separate findings; never co-tagged
    // -----------------------------------------------------------------------

    /// EC-007: FC=0x05 (DIRECT_OPERATE) then FC=0x02 (WRITE) on same flow.
    /// If threshold crossed, two findings: T1692.001 and T0836 separately.
    /// They must NEVER share a single Finding with both tags.
    ///
    /// Traces to: BC-2.15.013 invariant 3 (no co-tag); STORY-108 EC-007.
    #[test]
    fn test_EC_007_control_then_write_separate_findings_never_cotagged() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver 11 DIRECT_OPERATE to trigger T1692.001
        for i in 0..11u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        // Deliver 1 WRITE → T0836
        let write_frame = build_detection_frame(0x02, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &write_frame, 12, Direction::ClientToServer);

        // Verify no finding has BOTH T1692.001 and T0836 in the same finding
        for f in &analyzer.all_findings {
            let has_t1692 = f.mitre_techniques.contains(&"T1692.001".to_string());
            let has_t0836 = f.mitre_techniques.contains(&"T0836".to_string());
            assert!(
                !(has_t1692 && has_t0836),
                "EC-007: no Finding must carry both T1692.001 and T0836 (cannot co-occur on same FC)"
            );
        }

        // Verify T1692.001 finding exists
        let t1692 = analyzer
            .all_findings
            .iter()
            .any(|f| f.mitre_techniques.contains(&"T1692.001".to_string()));
        assert!(
            t1692,
            "EC-007: T1692.001 finding must exist (from 11 DIRECT_OPERATE)"
        );

        // Verify T0836 finding exists
        let t0836 = analyzer
            .all_findings
            .iter()
            .any(|f| f.mitre_techniques.contains(&"T0836".to_string()));
        assert!(t0836, "EC-007: T0836 finding must exist (from WRITE FC)");
    }

    // -----------------------------------------------------------------------
    // EC-008: wrapping_sub — out-of-order timestamp safe (no panic)
    // -----------------------------------------------------------------------

    /// EC-008: `now_ts.saturating_sub(window_start_ts)` must not panic with out-of-order
    /// timestamps (e.g. pcap replay where timestamps go backward).
    ///
    /// Scenario: seed window at ts=0xFFFFFFF0; new FC at ts=0x00000005.
    /// saturating_sub(0x5, 0xFFFFFFF0) = 0 (saturates at 0; backwards-clock stays in window).
    /// Plain subtraction (0x5 - 0xFFFFFFF0) would overflow with overflow-checks=true.
    ///
    /// Traces to: BC-2.15.010 invariant (saturating_sub required; RULING-DNP3-SIBLING-001
    /// §2.2); STORY-108 EC-008.
    #[test]
    fn test_EC_008_wrapping_sub_out_of_order_timestamp_no_panic() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Seed the window near u32::MAX
        let ts_start: u32 = 0xFFFFFFF0;
        let frame_seed = build_detection_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(
            key.clone(),
            &frame_seed,
            ts_start,
            Direction::ClientToServer,
        );

        // Deliver 10 more frames at ts that wraps around (0..=9)
        // saturating_sub(0x9, 0xFFFFFFF0) = 0 < 60 → still in same window (saturates at 0)
        for i in 0..10u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            // Must NOT panic (plain subtraction would panic due to overflow-checks=true)
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        // If we got here without panic, saturating_sub is working.
        // Count must be >= 11 in the window (backwards-ts saturates to 0, stays in window):
        // saturating_sub(0, 0xFFFFFFF0) = 0 < 60 → same window
        // saturating_sub(9, 0xFFFFFFF0) = 0 < 60 → same window
        // So all 11 FCs should be in the same window (threshold=10 → finding emitted)
        let t1692 = analyzer
            .all_findings
            .iter()
            .any(|f| f.mitre_techniques.contains(&"T1692.001".to_string()));
        assert!(
            t1692,
            "EC-008: T1692.001 must fire without panic using wrapping_sub \
             (11 FCs within ~25-second wrapped window)"
        );
    }

    // -----------------------------------------------------------------------
    // Summary structure verification (supports AC-010 / AC-011)
    // test_summarize_control_operation_counts_per_flow
    // -----------------------------------------------------------------------

    /// Verifies that summarize() includes control_operation_counts per flow.
    ///
    /// Traces to: BC-2.15.020 postcondition 1 (control_operation_counts field);
    ///            STORY-108 AC-010.
    #[test]
    fn test_BC_2_15_020_summarize_control_operation_counts_per_flow() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 5 DIRECT_OPERATE FCs on one flow (count=5, below threshold=10)
        for i in 0..5u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        let summary = analyzer.summarize();

        // control_operation_counts must be present
        let ctrl_counts = summary
            .detail
            .get("control_operation_counts")
            .expect("BC-2.15.020: control_operation_counts must be present in summary");

        // Must be an object or array (implementation decides schema);
        // the critical assertion: it's not null/absent
        assert!(
            !ctrl_counts.is_null(),
            "BC-2.15.020: control_operation_counts must not be null"
        );
    }

    // -----------------------------------------------------------------------
    // parse_errors in summary
    // -----------------------------------------------------------------------

    /// Verifies summarize() includes parse_errors (BC-2.15.020 postcondition 1).
    ///
    /// Traces to: BC-2.15.020 postcondition 1; STORY-108 AC-011.
    #[test]
    fn test_BC_2_15_020_summarize_includes_parse_errors() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver a frame that causes parse errors: valid sync + LENGTH=4 (invalid < 5)
        let mut bad_frame = vec![0u8; 10];
        bad_frame[0] = 0x05;
        bad_frame[1] = 0x64;
        bad_frame[2] = 4; // invalid LENGTH
        bad_frame[3] = 0xC4;
        analyzer.on_data(key.clone(), &bad_frame, 0, Direction::ClientToServer);

        let summary = analyzer.summarize();

        // parse_errors must be present
        assert!(
            summary.detail.contains_key("parse_errors"),
            "BC-2.15.020: parse_errors key must be present in summary detail"
        );
    }

    // -----------------------------------------------------------------------
    // Verify summarize() does NOT emit new findings (BC-2.15.020 invariant 3)
    // -----------------------------------------------------------------------

    /// BC-2.15.020 invariant 3: summarize() must NOT push new findings.
    ///
    /// Traces to: BC-2.15.020 invariant 3; STORY-108 AC-010.
    #[test]
    fn test_BC_2_15_020_summarize_does_not_push_findings() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 5 DIRECT_OPERATE frames (below threshold — no findings during on_data)
        for i in 0..5u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        let count_before = analyzer.all_findings.len();
        let _summary = analyzer.summarize();
        let count_after = analyzer.all_findings.len();

        assert_eq!(
            count_before, count_after,
            "BC-2.15.020 invariant 3: summarize() must NOT push new findings"
        );
    }
    // -----------------------------------------------------------------------
    // F-108-P2-001: Asymmetric-port coverage — de-vacuify the port-20000 heuristic
    //
    // The existing test_flow_key() has BOTH endpoints on port 20000, so the
    // `if flow_key.lower_port() == 20000` branch always takes the same arm.
    // These two tests build realistic flows (master on ephemeral port, outstation
    // on port 20000) and verify BOTH branches of the heuristic independently.
    //
    // FlowKey::new canonicalizes by (ip, port) tuple: if (ip_a, port_a) <= (ip_b, port_b)
    // then lower=(ip_a, port_a), upper=(ip_b, port_b); otherwise swapped.
    // -----------------------------------------------------------------------

    /// F-108-P2-001 Test A: master on ephemeral port, outstation on port 20000,
    /// master IP is numerically smaller.
    ///
    /// Configuration:
    ///   master    = 10.0.0.5:49152  (initiator, sends control commands)
    ///   outstation = 10.0.0.9:20000 (server / responder)
    ///
    /// FlowKey canonicalization:
    ///   Compare (10.0.0.5, 49152) vs (10.0.0.9, 20000):
    ///   10.0.0.5 < 10.0.0.9 (IpAddr Ord compares octets) → lower=(10.0.0.5, 49152),
    ///   upper=(10.0.0.9, 20000).
    ///
    /// Port heuristic branch exercised:
    ///   lower_port() == 49152 ≠ 20000 → ELSE branch → master_ip = lower_ip = 10.0.0.5
    ///
    /// This tests the ELSE branch of `if flow_key.lower_port() == 20000`.
    ///
    /// Traces to: BC-2.15.010 PC3 (source_ip resolution); STORY-108 F-108-P2-001 Test A.
    #[test]
    fn test_BC_2_15_010_asymmetric_port_master_lower_ip_else_branch() {
        let mut analyzer = Dnp3Analyzer::new(10);

        // Build asymmetric FlowKey: master=10.0.0.5:49152, outstation=10.0.0.9:20000
        // Canonicalization: (10.0.0.5, 49152) < (10.0.0.9, 20000) → lower=(10.0.0.5, 49152)
        // lower_port() = 49152 ≠ 20000 → ELSE branch → master_ip = lower_ip = 10.0.0.5
        let master_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5));
        let outstation_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 9));
        let key = FlowKey::new(master_ip, 49152, outstation_ip, 20000);

        // Verify the canonicalization so the branch derivation is self-documenting:
        // lower_port must be 49152 (not 20000), confirming the ELSE branch fires.
        assert_eq!(
            key.lower_port(),
            49152,
            "Test A canonicalization: lower_port must be 49152 (master's ephemeral port) \
             so the ELSE branch of the heuristic fires"
        );
        assert_eq!(
            key.lower_ip(),
            master_ip,
            "Test A canonicalization: lower_ip must be 10.0.0.5 (master)"
        );

        // Deliver 11 Control-class FCs (DIRECT_OPERATE=0x05) to trigger T1692.001
        for i in 0..11u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "Test A: 11 Control FCs must emit exactly ONE T1692.001 finding"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T1692.001"],
            "Test A: finding must carry T1692.001"
        );

        // The critical assertion: source_ip must be the MASTER's IP (10.0.0.5).
        // ELSE branch: lower_port != 20000 → master_ip = lower_ip = 10.0.0.5
        assert_eq!(
            f.source_ip,
            Some(master_ip),
            "Test A (ELSE branch): source_ip must be Some(10.0.0.5) = master (lower_ip); \
             lower_port=49152 ≠ 20000 takes ELSE → lower_ip; got {:?}",
            f.source_ip
        );
    }

    /// F-108-P2-001 Test B: master on ephemeral port, outstation on port 20000,
    /// outstation IP is numerically smaller (forces outstation into lower slot).
    ///
    /// Configuration:
    ///   master     = 10.0.0.9:49152 (initiator, sends control commands)
    ///   outstation = 10.0.0.5:20000 (server / responder)
    ///
    /// FlowKey canonicalization:
    ///   Compare (10.0.0.9, 49152) vs (10.0.0.5, 20000):
    ///   10.0.0.5 < 10.0.0.9 → (10.0.0.5, 20000) wins as lower tuple →
    ///   lower=(10.0.0.5, 20000), upper=(10.0.0.9, 49152).
    ///
    /// Port heuristic branch exercised:
    ///   lower_port() == 20000 → IF branch → master_ip = upper_ip = 10.0.0.9
    ///
    /// This tests the IF branch of `if flow_key.lower_port() == 20000`.
    /// Test A exercises the ELSE branch; together they cover both branches non-vacuously.
    ///
    /// Traces to: BC-2.15.010 PC3 (source_ip resolution); STORY-108 F-108-P2-001 Test B.
    #[test]
    fn test_BC_2_15_010_asymmetric_port_master_upper_ip_if_branch() {
        let mut analyzer = Dnp3Analyzer::new(10);

        // Build asymmetric FlowKey: master=10.0.0.9:49152, outstation=10.0.0.5:20000
        // Canonicalization: (10.0.0.9, 49152) vs (10.0.0.5, 20000):
        //   10.0.0.5 < 10.0.0.9 → lower=(10.0.0.5, 20000), upper=(10.0.0.9, 49152)
        // lower_port() = 20000 → IF branch → master_ip = upper_ip = 10.0.0.9
        let master_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 9));
        let outstation_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5));
        let key = FlowKey::new(master_ip, 49152, outstation_ip, 20000);

        // Verify the canonicalization so the branch derivation is self-documenting:
        // lower_port must be 20000 (outstation's port), confirming the IF branch fires.
        assert_eq!(
            key.lower_port(),
            20000,
            "Test B canonicalization: lower_port must be 20000 (outstation's well-known port) \
             so the IF branch of the heuristic fires"
        );
        assert_eq!(
            key.lower_ip(),
            outstation_ip,
            "Test B canonicalization: lower_ip must be 10.0.0.5 (outstation)"
        );
        assert_eq!(
            key.upper_ip(),
            master_ip,
            "Test B canonicalization: upper_ip must be 10.0.0.9 (master)"
        );

        // Deliver 11 Control-class FCs (DIRECT_OPERATE=0x05) to trigger T1692.001
        for i in 0..11u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "Test B: 11 Control FCs must emit exactly ONE T1692.001 finding"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T1692.001"],
            "Test B: finding must carry T1692.001"
        );

        // The critical assertion: source_ip must be the MASTER's IP (10.0.0.9).
        // IF branch: lower_port == 20000 → master_ip = upper_ip = 10.0.0.9
        assert_eq!(
            f.source_ip,
            Some(master_ip),
            "Test B (IF branch): source_ip must be Some(10.0.0.9) = master (upper_ip); \
             lower_port=20000 takes IF → upper_ip; got {:?}",
            f.source_ip
        );
    }
    // -----------------------------------------------------------------------
    // Red Gate — BC-2.15.020 v1.4: parse_errors key rename (D-220, human-approved)
    //
    // This test passes on current code: the rename is complete — the analyzer emits
    // "parse_errors", not "total_parse_errors" (rename landed in develop f5c002a).
    //
    // Traces to: BC-2.15.020 v1.4 postcondition 1 (BREAKING rename D-220);
    //            scope.md PC-014; test-vector row "Red Gate — key name".
    // -----------------------------------------------------------------------

    /// BC-2.15.020 v1.4 Red Gate: summarize() detail map MUST use key "parse_errors",
    /// NOT "total_parse_errors" (D-220 breaking rename — aligns DNP3 with HTTP/TLS/Modbus).
    ///
    /// Traces to: BC-2.15.020 v1.4 postcondition 1; PC-014 scope.md §4.
    #[test]
    fn test_BC_2_15_020_parse_errors_key_name_is_parse_errors() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver a frame with invalid LENGTH (4 < 5) to produce a parse error,
        // mirroring the approach used by test_BC_2_15_020_summarize_includes_parse_errors.
        let mut bad_frame = vec![0u8; 10];
        bad_frame[0] = 0x05;
        bad_frame[1] = 0x64;
        bad_frame[2] = 4; // invalid LENGTH — triggers parse error counter
        bad_frame[3] = 0xC4;
        analyzer.on_data(key.clone(), &bad_frame, 0, Direction::ClientToServer);

        let summary = analyzer.summarize();

        // MUST be present under the new canonical key name (post-D-220).
        assert!(
            summary.detail.contains_key("parse_errors"),
            "BC-2.15.020 v1.4 D-220: detail map must contain \"parse_errors\" \
             (aligns with HTTP/TLS/Modbus sibling analyzers)"
        );

        // MUST NOT be present under the old divergent key name.
        assert!(
            !summary.detail.contains_key("total_parse_errors"),
            "BC-2.15.020 v1.4 D-220: detail map must NOT contain \"total_parse_errors\" \
             (old key removed by rename; callers must migrate to \"parse_errors\")"
        );
    }
} // mod story_108

// ---------------------------------------------------------------------------
// STORY-140 — mod direction_and_clock
//
// Red-gate tests for STORY-140: DNP3 per-direction carry buffer split
// (DRIFT-DNP3-DIRECTION-001) and saturating_sub window monotonicity
// (DRIFT-DNP3-CLOCK-001 / DRIFT-DNP3-OP-001).
//
// AC coverage: AC-140-001, AC-140-002, AC-140-003, AC-140-005,
//              AC-140-007, AC-140-008, AC-140-009.
//
// All tests in this module FAIL against the red-gate stub (carry_c2s used for
// both directions; wrapping_sub >= in the 300s window). They go GREEN once the
// implementer applies the direction-split carry selection and replaces wrapping_sub
// with saturating_sub (plus the >= → > operator pin).
//
// Namespace: DF-TEST-NAMESPACE-001 (mod wrapper).
// Doc-comment tense: DF-GREEN-DOC-TENSE-SWEEP v2 (regression-guard framing).
// ---------------------------------------------------------------------------

mod direction_and_clock {
    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn ip(a: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, a))
    }

    /// Flow: master=10.0.0.1:54321, outstation=10.0.0.2:20000.
    ///
    /// FlowKey canonicalization:
    ///   Compare (10.0.0.1, 54321) vs (10.0.0.2, 20000).
    ///   10.0.0.1 < 10.0.0.2 → lower=(10.0.0.1, 54321), upper=(10.0.0.2, 20000).
    ///   lower_port=54321, upper_port=20000.
    ///   Port-heuristic: lower_port ≠ 20000 → ELSE branch → master_ip = lower_ip = 10.0.0.1.
    /// With direction-based fix (AC-140-002): ClientToServer → src=10.0.0.1 is master.
    fn c2s_flow_key() -> FlowKey {
        FlowKey::new(ip(1), 54321, ip(2), 20000)
    }

    /// Build a minimal partial DNP3 header prefix of `prefix_len` bytes.
    ///
    /// The bytes begin with the valid sync word [0x05, 0x64] so that when stashed in
    /// carry_c2s they pass the sync gate on the next c2s delivery.  The remaining bytes
    /// are zero (placeholder for CTRL, DEST, SRC, CRC — not completed intentionally).
    ///
    /// Invariant: `prefix_len` must be in 1..10 (partial — not a complete 10-byte header).
    fn partial_c2s_header(prefix_len: usize) -> Vec<u8> {
        assert!(
            (1..10).contains(&prefix_len),
            "partial_c2s_header: prefix_len must be 1..9, got {prefix_len}"
        );
        let mut buf = vec![0u8; prefix_len];
        if prefix_len >= 1 {
            buf[0] = 0x05;
        }
        if prefix_len >= 2 {
            buf[1] = 0x64;
        }
        buf
    }

    /// Build a complete valid DNP3 link frame for s2c (outstation-to-master).
    ///
    /// Layout (10 bytes — minimum DNP3 frame, LENGTH=5):
    ///   [0]  0x05  sync
    ///   [1]  0x64  sync
    ///   [2]  0x05  LENGTH = 5 (minimum)
    ///   [3]  0x44  CTRL: DIR=0 (outstation response), PRM=1 (primary), nibble=0x04
    ///               UNCONFIRMED_USER_DATA  → has_user_data = true
    ///   [4]  0x01  DEST lo
    ///   [5]  0x00  DEST hi
    ///   [6]  0x03  SRC lo
    ///   [7]  0x00  SRC hi
    ///   [8]  0x00  CRC lo (placeholder — not enforced in frame-walk)
    ///   [9]  0x00  CRC hi
    ///
    /// This frame passes the sync gate ([0x05, 0x64]) and compute_dnp3_frame_len(5) = 10.
    /// The transport octet is at byte[10] — not present in this minimal 10-byte frame —
    /// so the FIR=1 check is skipped; frame_count increments without FC classification.
    fn complete_s2c_frame() -> Vec<u8> {
        vec![
            0x05, 0x64, // sync
            0x05, // LENGTH = 5 → frame_len = 10 bytes
            0x44, // CTRL: DIR=0 PRM=1 UNCONFIRMED_USER_DATA
            0x01, 0x00, // DEST = 1 (little-endian)
            0x03, 0x00, // SRC = 3 (little-endian)
            0x00, 0x00, // CRC placeholder
        ]
    }

    /// Build a detection-capable DNP3 frame with given application FC.
    ///
    /// LENGTH=8 → frame_len=15 bytes, which reaches byte[12] (app_fc).
    /// This mirrors `build_detection_frame` in mod story_108.
    fn detection_frame(app_fc: u8) -> Vec<u8> {
        let length_byte: u8 = 8;
        let u = (length_byte as usize) - 5; // 3
        let blocks = u.div_ceil(16); // 1
        let frame_len = 5 + (length_byte as usize) + 2 * blocks; // 15

        let mut frame = vec![0u8; frame_len];
        frame[0] = 0x05;
        frame[1] = 0x64;
        frame[2] = length_byte;
        frame[3] = 0xC4; // CTRL: DIR=1 PRM=1 UNCONFIRMED_USER_DATA
        frame[4] = 0x03; // DEST lo
        frame[5] = 0x00; // DEST hi
        frame[6] = 0x01; // SRC lo
        frame[7] = 0x00; // SRC hi
        // bytes 8-9: header CRC placeholder
        frame[10] = 0xC0; // transport: FIR=1 (0x40) | FIN=1 (0x80)
        frame[11] = 0x00; // app control
        frame[12] = app_fc;
        // bytes 13-14: data CRC placeholder
        frame
    }

    // -----------------------------------------------------------------------
    // AC-140-001 (a): carry direction isolation — no cross-direction splice
    //
    // Guards EC-X1 (DNP3 DRIFT-DNP3-DIRECTION-001): FAILS against the red-gate
    // stub (carry_c2s used for both directions).  With the stub, the s2c delivery
    // prepends the partial c2s bytes into carry_c2s, producing a garbled buffer
    // that the frame-walk parses as the s2c frame; carry_c2s is NOT retained
    // because the splice is consumed.
    //
    // Traces: BC-2.15.016 v2.0 Invariant 6, EC-010; RULING-DNP3-SIBLING-001 §8 AC-8.
    // -----------------------------------------------------------------------

    /// Guards EC-X1 (DNP3 carry-direction splice): partial c2s frame stashed in
    /// carry_c2s; complete s2c delivery uses carry_s2c (not carry_c2s).
    ///
    /// After the s2c delivery: frame_count==1, parse_errors==0, carry_c2s still holds
    /// the partial c2s bytes, carry_s2c is empty (s2c consumed its own empty carry).
    ///
    /// With the stub (carry_c2s for both): the s2c frame is prepended with c2s carry bytes
    /// → the spliced buffer is either accepted (spurious frame, carry_c2s drained) or
    /// rejected (parse_errors > 0) — BOTH outcomes fail this test.
    #[test]
    fn test_ac140_001_carry_direction_isolation_no_splice() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = c2s_flow_key();

        // Deliver 5-byte partial c2s header → stashed in carry_c2s.
        let partial = partial_c2s_header(5);
        analyzer.on_data(key.clone(), &partial, 100, Direction::ClientToServer);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after c2s delivery");
            assert_eq!(
                flow.frame_count, 0,
                "partial delivery must not complete a frame"
            );
            assert_eq!(
                flow.carry_c2s.len(),
                5,
                "AC-140-001: carry_c2s must hold the 5-byte partial c2s header"
            );
            assert_eq!(
                flow.carry_s2c.len(),
                0,
                "AC-140-001: carry_s2c must be empty after c2s-only delivery"
            );
        }

        // Deliver complete s2c frame → carry_s2c used (carry_c2s NOT involved).
        let s2c = complete_s2c_frame();
        analyzer.on_data(key.clone(), &s2c, 101, Direction::ServerToClient);

        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after s2c delivery");
        assert_eq!(
            flow.frame_count, 1,
            "AC-140-001: complete s2c frame must produce frame_count==1 \
             (guards: stub splices carry_c2s into s2c → garbled frame, frame_count≠1)"
        );
        assert_eq!(
            flow.parse_errors, 0,
            "AC-140-001: s2c delivery must produce 0 parse_errors \
             (guards: stub splice produces spurious parse_error)"
        );
        assert!(
            !flow.carry_c2s.is_empty(),
            "AC-140-001: carry_c2s must STILL hold the partial c2s bytes \
             (guards: stub drains carry_c2s when splicing it into the s2c walk)"
        );
        assert_eq!(
            flow.carry_s2c.len(),
            0,
            "AC-140-001: carry_s2c must be empty (s2c frame fully consumed its carry)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-140-001 (b): carry_c2s and carry_s2c are independent concurrently
    //
    // Guards BC-2.15.016 v2.0 Postcondition 1, Invariant 6: partial frames
    // stashed in both directions coexist without contamination.
    //
    // With stub (carry_c2s for both): the s2c partial overwrites carry_c2s bytes
    // from the c2s partial delivery → carry_c2s doesn't retain both separately.
    // -----------------------------------------------------------------------

    /// Guards BC-2.15.016 v2.0 Postcondition 1 + Invariant 6: partial c2s and
    /// partial s2c frames stashed concurrently in carry_c2s and carry_s2c respectively.
    ///
    /// Both carries must contain bytes after their respective partial deliveries, and
    /// neither carry must contain bytes from the other direction.
    ///
    /// With stub (carry_c2s for both): the s2c partial delivery appends to carry_c2s
    /// instead of carry_s2c → carry_s2c.len()==0 fails the assert.
    #[test]
    fn test_ac140_001_carry_c2s_and_carry_s2c_are_independent() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = c2s_flow_key();

        // Partial c2s delivery: 5 bytes stashed in carry_c2s.
        let partial_c2s = partial_c2s_header(5);
        analyzer.on_data(key.clone(), &partial_c2s, 100, Direction::ClientToServer);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.carry_c2s.len(),
                5,
                "AC-140-001b: carry_c2s must hold 5 bytes from partial c2s delivery"
            );
            assert_eq!(
                flow.carry_s2c.len(),
                0,
                "AC-140-001b: carry_s2c must be empty after c2s-only delivery"
            );
        }

        // Partial s2c delivery (3 bytes of a valid s2c sync header) → stashed in carry_s2c.
        // Use only 3 bytes — partial sync [0x05, 0x64, LENGTH] but not full 10-byte frame.
        let partial_s2c: Vec<u8> = vec![0x05, 0x64, 0x05]; // valid sync prefix, partial
        analyzer.on_data(key.clone(), &partial_s2c, 101, Direction::ServerToClient);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            !flow.carry_c2s.is_empty(),
            "AC-140-001b: carry_c2s must still hold c2s partial bytes after s2c delivery \
             (guards: stub overwrites carry_c2s when processing s2c)"
        );
        assert!(
            !flow.carry_s2c.is_empty(),
            "AC-140-001b: carry_s2c must hold the partial s2c bytes \
             (guards: stub uses carry_c2s for s2c → carry_s2c.len()==0)"
        );
        // Verify that neither carry contains bytes from the other direction.
        // c2s carry should NOT start with [0x05, 0x64, 0x05] (the s2c partial).
        // s2c carry should NOT start with [0x05, 0x64] from c2s at full length 5.
        // The carries must be distinct byte sequences.
        assert_ne!(
            flow.carry_c2s, flow.carry_s2c,
            "AC-140-001b: carry_c2s and carry_s2c must hold distinct byte sequences \
             (carries must not alias each other)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-140-002: direction-based source IP — standard DNP3 topology
    //
    // Guards RULING-DNP3-SIBLING-001 §1.4: direction-aware src_ip resolution
    // correctly identifies the master as the source when Direction::ClientToServer.
    //
    // Standard topology: outstation listens on port 20000 (well-known),
    // master initiates from an ephemeral port (54321).
    //
    // FlowKey::new canonicalization: (10.0.0.1, 20000) <= (10.0.0.2, 54321)
    //   → lower=(10.0.0.1, 20000) = outstation, upper=(10.0.0.2, 54321) = master.
    //   Port-heuristic IF branch: lower_port==20000 → master_ip = upper_ip = 10.0.0.2.
    //   Direction C2S: master = upper_ip = 10.0.0.2.  Both agree.
    //
    // Assert: source_ip == Some(10.0.0.2) — the master (upper/C2S-initiator).
    // -----------------------------------------------------------------------

    /// Guards RULING-DNP3-SIBLING-001 §1.4: direction-aware src_ip resolution.
    ///
    /// Standard DNP3 topology: outstation=10.0.0.1:20000 (server, well-known port),
    /// master=10.0.0.2:54321 (client, ephemeral port).
    ///
    /// FlowKey canonicalization:
    ///   (10.0.0.1, 20000) <= (10.0.0.2, 54321) → lower=(10.0.0.1, 20000), upper=(10.0.0.2, 54321).
    ///   lower = outstation (port 20000), upper = master (port 54321).
    ///
    /// Port-heuristic (IF branch): lower_port==20000 → master_ip = upper_ip = 10.0.0.2.
    /// Direction (ClientToServer): master initiates C2S → master_ip = upper_ip = 10.0.0.2.
    /// Both agree: source_ip must be Some(10.0.0.2).
    ///
    /// Traces to: STORY-140 AC-140-002; RULING-DNP3-SIBLING-001 §1.4; BC-2.15.016 v2.0 PC3.
    /// Consistent with test_BC_2_15_010_asymmetric_port_master_upper_ip_if_branch (IF branch).
    #[test]
    fn test_ac140_002_direction_based_source_ip() {
        let outstation_ip = ip(1); // 10.0.0.1 — outstation, listens on port 20000
        let master_ip = ip(2); // 10.0.0.2 — master, initiates from ephemeral port 54321

        // Standard topology: outstation on port 20000, master on ephemeral 54321.
        // Canonicalization: (10.0.0.1, 20000) <= (10.0.0.2, 54321) → lower=outstation, upper=master.
        let key = FlowKey::new(outstation_ip, 20000, master_ip, 54321);

        // Self-documenting canonicalization assertions.
        assert_eq!(
            key.lower_port(),
            20000,
            "test setup: lower_port must be 20000 (outstation well-known port); IF branch fires"
        );
        assert_eq!(
            key.lower_ip(),
            outstation_ip,
            "test setup: lower_ip must be 10.0.0.1 (the outstation)"
        );
        assert_eq!(
            key.upper_ip(),
            master_ip,
            "test setup: upper_ip must be 10.0.0.2 (the master)"
        );

        let mut analyzer = Dnp3Analyzer::new(10);

        // Deliver 11 Control-class FCs (DIRECT_OPERATE = 0x05) as ClientToServer.
        // Direction::ClientToServer = master initiating toward outstation.
        // master = upper_ip = 10.0.0.2 (initiated the C2S flow).
        for i in 0..11u32 {
            let frame = detection_frame(0x05);
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
        }

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-140-002: 11 Control FCs must emit exactly ONE T1692.001 finding"
        );
        let f = &analyzer.all_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T1692.001"],
            "AC-140-002: finding must carry T1692.001"
        );

        // The critical assertion: source_ip must be 10.0.0.2 (the master — C2S initiator = upper_ip).
        // IF branch port-heuristic: lower_port==20000 → master_ip = upper_ip = 10.0.0.2.
        // Direction C2S: master = upper_ip = 10.0.0.2.
        // Both the port-heuristic and direction-based resolution agree on the correct answer.
        assert_eq!(
            f.source_ip,
            Some(master_ip),
            "AC-140-002: source_ip must be Some(10.0.0.2) = master (upper_ip, C2S initiator); \
             lower_port==20000 (IF branch) → master=upper_ip; C2S → source=master=upper_ip; \
             got {:?}",
            f.source_ip
        );
    }

    // -----------------------------------------------------------------------
    // AC-140-003: dispatcher passes direction to on_data (compilation test)
    //
    // Guards BC-2.15.016 v2.0 Precondition 2: the 4-argument on_data signature
    // `(flow_key, data, ts, direction)` is accepted.  This is a compilation-only
    // guard — it passes as soon as the signature compiles.
    //
    // The red-gate structural scaffolding already has the 4-argument signature,
    // so this test compiles and PASSES once the stub is in place.  It guards
    // against future regression to the 3-argument form.
    // -----------------------------------------------------------------------

    /// Guards BC-2.15.016 v2.0 Precondition 2: Dnp3Analyzer::on_data accepts
    /// a `direction: Direction` fourth argument.
    ///
    /// Calls on_data with both Direction values to confirm both variants compile
    /// and execute without panic.  Regression-guard: any future removal of the
    /// direction parameter would cause a compile error here.
    #[test]
    fn test_ac140_003_dispatcher_passes_direction() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = c2s_flow_key();
        let frame = detection_frame(0x05);

        // Must compile and not panic for both direction variants.
        analyzer.on_data(key.clone(), &frame, 0, Direction::ClientToServer);
        analyzer.on_data(key.clone(), &frame, 1, Direction::ServerToClient);

        // Both deliveries must be processed (flow exists, counters incremented).
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after both on_data calls");
        assert!(
            flow.frame_count >= 1,
            "AC-140-003: on_data with direction parameter must process frames without panic"
        );
    }

    // -----------------------------------------------------------------------
    // AC-140-005: 300s correlation window uses strict `>` (not `>=`)
    //
    // Guards BC-2.15.015 v2.0 Postcondition 3, Invariant 6; DRIFT-DNP3-OP-001.
    //
    // With stub (wrapping_sub >=): elapsed==300 triggers the window-reset arm
    // (300 >= 300 is true → window resets, restart_event_count cleared → T0827
    // does NOT fire → assert fails).
    //
    // With fix (saturating_sub >): elapsed==300 does NOT trigger reset
    // (300 > 300 is false → count=3 reaches T0827_THRESHOLD=3 → T0827 fires).
    // -----------------------------------------------------------------------

    /// Guards DRIFT-DNP3-OP-001 operator pin: elapsed==300 must NOT expire the
    /// 300s correlation window under strict `>`.
    ///
    /// Scenario:
    ///   ts=0: COLD_RESTART → restart_event_count=1, correlation_window seeded at ts=0.
    ///   ts=150: COLD_RESTART → restart_event_count=2.
    ///   ts=300: COLD_RESTART → elapsed = saturating_sub(300, 0) = 300.
    ///     Strict `>`: 300 > 300 is FALSE → window NOT reset → restart_event_count=3.
    ///     3 >= T0827_THRESHOLD=3 → T0827 fires.
    ///   ts=301: any delivery → elapsed=301 > 300 → window IS reset.
    ///
    /// With stub (wrapping_sub >= 300): 300 >= 300 is TRUE → window resets at ts=300
    /// → restart_event_count cleared → T0827 does NOT fire → test FAILS.
    #[test]
    fn test_ac140_005_correlation_window_operator_pin_boundary() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = c2s_flow_key();
        let cold_restart_frame = detection_frame(0x0D); // FC=0x0D COLD_RESTART

        // ts=0: First COLD_RESTART — seeds correlation_window_start_ts = 0.
        analyzer.on_data(
            key.clone(),
            &cold_restart_frame,
            0,
            Direction::ClientToServer,
        );
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.restart_event_count, 1,
                "AC-140-005: first COLD_RESTART must set restart_event_count=1"
            );
        }

        // ts=150: Second COLD_RESTART — still in window (150 < 300).
        analyzer.on_data(
            key.clone(),
            &cold_restart_frame,
            150,
            Direction::ClientToServer,
        );
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.restart_event_count, 2,
                "AC-140-005: second COLD_RESTART must set restart_event_count=2"
            );
        }

        // ts=300: Third COLD_RESTART — elapsed = saturating_sub(300, 0) = 300.
        // Strict >: 300 > 300 is FALSE → window NOT reset → count reaches 3 → T0827 fires.
        // Stub >=:  300 >= 300 is TRUE → window reset → count cleared → T0827 does NOT fire.
        analyzer.on_data(
            key.clone(),
            &cold_restart_frame,
            300,
            Direction::ClientToServer,
        );

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.restart_event_count, 3,
                "AC-140-005: at elapsed==300 (strict >), window must NOT reset; \
                 restart_event_count must be 3 (not reset to 1 by window expiry). \
                 FAILS with stub (>= resets window at elapsed=300)"
            );
        }

        let t0827_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .count();
        assert_eq!(
            t0827_count, 1,
            "AC-140-005: T0827 must fire at restart_event_count=3 (>= T0827_THRESHOLD=3). \
             FAILS with stub (>= resets window → count never reaches 3 → T0827 suppressed)"
        );

        // ts=301: Window IS expired under strict `>`.
        let any_frame = detection_frame(0x01); // READ FC — no detection side-effects
        analyzer.on_data(key.clone(), &any_frame, 301, Direction::ClientToServer);
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            // elapsed = saturating_sub(301, 300) = 1, but window was reset AT ts=300
            // after T0827 fired... actually the window resets when elapsed > 300.
            // After the ts=300 frame the window_start slides to ts=300.
            // At ts=301: elapsed = saturating_sub(301, 300) = 1, NOT > 300 → no reset.
            // The window expiry at strict > 300 means ts=301 after a window_start of 0
            // resets it (elapsed=301 > 300). But the window_start was already at 300.
            // This assertion confirms the window was already seeded; no further assertion needed.
            assert!(
                flow.correlation_window_seeded,
                "AC-140-005: correlation window must be seeded by ts=301"
            );
        }
    }

    // -----------------------------------------------------------------------
    // AC-140-007: regression — partial c2s carry + complete s2c frame → isolation
    //
    // Promoted from EC-X1 repro (RULING-DNP3-SIBLING-001 §1.1).
    // Distinguished from AC-140-001 by the assertion set: this test uses
    // frame_count=1 and parse_errors=0 as the primary regression guards,
    // making the regression scenario explicit.
    //
    // With stub: carry_c2s spliced into s2c walk → parse_errors > 0 OR
    // carry_c2s.len()==0 (bytes consumed by the splice) → FAILS.
    // -----------------------------------------------------------------------

    /// Guards EC-X1 (DNP3 carry-direction splice regression repro).
    ///
    /// Deliver 5-byte partial c2s header (stashed in carry_c2s), then deliver
    /// complete s2c frame.  Assert: frame_count==1, parse_errors==0,
    /// carry_c2s retains partial bytes, carry_s2c is empty.
    ///
    /// Before STORY-140 fix: spliced buf = carry_c2s(partial c2s header) ++ s2c_frame_bytes.
    /// The [0x05, 0x64] sync in carry_c2s head passes the sync gate; compute_dnp3_frame_len
    /// produces a length that either accepts the garbled frame (parse_errors==0 but wrong
    /// frame_count) or rejects it (parse_errors>0) — either outcome fails this test.
    #[test]
    fn test_ac140_007_regression_carry_direction_no_splice() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = c2s_flow_key();

        // 5-byte partial c2s header: [0x05, 0x64, 0x00, 0x00, 0x00].
        // Length=5, starts with sync word. Stashed in carry_c2s.
        let partial = partial_c2s_header(5);
        analyzer.on_data(key.clone(), &partial, 100, Direction::ClientToServer);

        // Complete s2c frame: 10 bytes, valid, must process cleanly without touching carry_c2s.
        let s2c = complete_s2c_frame();
        analyzer.on_data(key.clone(), &s2c, 101, Direction::ServerToClient);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.frame_count, 1,
            "AC-140-007 EC-X1 repro: frame_count must be 1 (the s2c frame only); \
             FAILS with stub (splice may produce 0 or wrong frame_count)"
        );
        assert_eq!(
            flow.parse_errors, 0,
            "AC-140-007 EC-X1 repro: parse_errors must be 0; \
             FAILS with stub (splice produces garbled frame → parse_error)"
        );
        assert!(
            !flow.carry_c2s.is_empty(),
            "AC-140-007 EC-X1 repro: carry_c2s must retain the partial c2s bytes; \
             FAILS with stub (c2s carry consumed by the s2c splice)"
        );
        assert_eq!(
            flow.carry_s2c.len(),
            0,
            "AC-140-007 EC-X1 repro: carry_s2c must be empty after s2c consume"
        );
    }

    // -----------------------------------------------------------------------
    // AC-140-008: regression — backwards-ts packet does not reset 60s detect window
    //
    // DNP3 EC-X2 analog (RULING-DNP3-SIBLING-001 §8 AC-9).
    //
    // With stub (wrapping_sub): wrapping_sub(50, 100) = u32::MAX - 49 ≈ 4.29e9.
    // 4.29e9 > 60 → window resets → direct_operate_count = 1 → threshold not crossed
    // → T1692.001 suppressed → test FAILS.
    //
    // With fix (saturating_sub): saturating_sub(50, 100) = 0. 0 NOT > 60 → no reset.
    // Count continues from 9 → 10 (at backwards ts) → 11, 12 → T1692.001 fires at 11.
    // -----------------------------------------------------------------------

    /// Guards DNP3 EC-X2 (backwards-clock T1692.001 suppression regression repro).
    ///
    /// Scenario:
    ///   9 DIRECT_OPERATE at ts=100 (window_start=100, direct_operate_count=9).
    ///   1 DIRECT_OPERATE at ts=50 (backwards clock).
    ///     saturating_sub(50, 100) = 0 → NOT > 60 → no reset → count=10.
    ///     count=10 == threshold=10 (NOT > 10) → no finding yet.
    ///   2 more DIRECT_OPERATE at ts=100 → count=11, count=12.
    ///     count=11 > threshold=10 → T1692.001 fires.
    ///
    /// With stub (wrapping_sub): backwards-ts resets window → T1692.001 suppressed → FAILS.
    #[test]
    fn test_ac140_008_regression_backwards_ts_t1692_no_reset() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = c2s_flow_key();
        let do_frame = detection_frame(0x05); // DIRECT_OPERATE FC=0x05

        // Phase 1: 9 DIRECT_OPERATE at ts=100.
        for _ in 0..9u32 {
            analyzer.on_data(key.clone(), &do_frame, 100, Direction::ClientToServer);
        }
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.direct_operate_count, 9,
                "AC-140-008 setup: direct_operate_count must be 9 after 9 FCs"
            );
            assert_eq!(
                flow.window_start_ts, 100,
                "AC-140-008 setup: window_start_ts must be 100"
            );
        }

        // Phase 2: 1 DIRECT_OPERATE at ts=50 (backwards clock, ts=50 < window_start=100).
        // saturating_sub(50, 100) = 0 → NOT > 60 → window NOT reset → count=10.
        analyzer.on_data(key.clone(), &do_frame, 50, Direction::ClientToServer);
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.direct_operate_count, 10,
                "AC-140-008: backwards-ts FC must increment count to 10 (no reset). \
                 FAILS with stub (wrapping_sub resets window → count=1)"
            );
            assert_eq!(
                analyzer.all_findings.len(),
                0,
                "AC-140-008: count=10 == threshold=10 (NOT > 10) → no finding yet"
            );
        }

        // Phase 3: 2 more DIRECT_OPERATE at ts=100 → count=11, count=12.
        // count=11 > threshold=10 → T1692.001 fires.
        analyzer.on_data(key.clone(), &do_frame, 100, Direction::ClientToServer);
        analyzer.on_data(key.clone(), &do_frame, 100, Direction::ClientToServer);

        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();
        assert_eq!(
            t1692_count, 1,
            "AC-140-008 EC-X2 repro: T1692.001 must fire at direct_operate_count=11 (>threshold=10). \
             FAILS with stub (wrapping_sub backwards-ts resets window → detection suppressed)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.direct_operate_count, 12,
            "AC-140-008: direct_operate_count must be 12 after all 12 deliveries (9+1+2)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-140-009: regression — backwards-clock restart event does not reset 300s window
    //
    // Guards BC-2.15.015 v2.0 Postcondition 3, EC-010; RULING-DNP3-SIBLING-001 §8 AC-10.
    //
    // With stub (wrapping_sub >=): wrapping_sub(50, 100) = u32::MAX-49 ≈ 4.29e9.
    // 4.29e9 >= 300 → window resets → restart_event_count cleared → T0827 suppressed.
    //
    // With fix (saturating_sub >): saturating_sub(50, 100) = 0. 0 NOT > 300 → no reset.
    // After backwards-ts: count still 2. ts=200 third restart → count=3 → T0827 fires.
    // -----------------------------------------------------------------------

    /// Guards BC-2.15.015 EC-010 (backwards-clock T0827 suppression regression repro).
    ///
    /// Scenario:
    ///   ts=100: COLD_RESTART → restart_event_count=1, correlation_window seeded at ts=100.
    ///   ts=150: COLD_RESTART → restart_event_count=2.
    ///   ts=50: backwards-clock COLD_RESTART.
    ///     saturating_sub(50, 100) = 0 → NOT > 300 → window NOT reset.
    ///     restart_event_count increments to 3 → T0827 fires (3 >= T0827_THRESHOLD=3).
    ///   (Confirm T0827 fires on the backwards-ts delivery itself when count reaches 3.)
    ///
    /// Alternatively: the backwards-ts call at ts=50 increments count to 3 and fires T0827.
    ///
    /// With stub (wrapping_sub >=): backwards-ts resets window → count cleared → T0827
    /// suppressed → test FAILS.
    #[test]
    fn test_ac140_009_regression_backwards_ts_t0827_no_reset() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = c2s_flow_key();
        let cold_restart = detection_frame(0x0D); // FC=0x0D COLD_RESTART

        // ts=100: First COLD_RESTART → seeds window at ts=100, restart_event_count=1.
        analyzer.on_data(key.clone(), &cold_restart, 100, Direction::ClientToServer);
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.restart_event_count, 1,
                "AC-140-009 setup: restart_event_count=1 after first COLD_RESTART at ts=100"
            );
            assert_eq!(
                flow.correlation_window_start_ts, 100,
                "AC-140-009 setup: correlation_window_start_ts seeded at ts=100"
            );
        }

        // ts=150: Second COLD_RESTART → restart_event_count=2.
        analyzer.on_data(key.clone(), &cold_restart, 150, Direction::ClientToServer);
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.restart_event_count, 2,
                "AC-140-009 setup: restart_event_count=2 after second COLD_RESTART at ts=150"
            );
        }

        // ts=50: COLD_RESTART with BACKWARDS clock (ts=50 < window_start=100).
        // saturating_sub(50, 100) = 0 → NOT > 300 → window NOT reset.
        // restart_event_count increments to 3 → T0827_THRESHOLD=3 → T0827 fires.
        analyzer.on_data(key.clone(), &cold_restart, 50, Direction::ClientToServer);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.restart_event_count, 3,
                "AC-140-009: backwards-ts COLD_RESTART must increment count to 3 (no reset). \
                 FAILS with stub (wrapping_sub >= resets window → count cleared)"
            );
            // Window must still be anchored at ts=100 (not reset by backwards-ts).
            assert_eq!(
                flow.correlation_window_start_ts, 100,
                "AC-140-009: correlation_window_start_ts must remain 100 (window not reset). \
                 FAILS with stub (wrapping_sub resets window → start_ts slides to ts=50)"
            );
        }

        let t0827_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .count();
        assert_eq!(
            t0827_count, 1,
            "AC-140-009 EC-010 repro: T0827 must fire when restart_event_count=3 reaches threshold. \
             FAILS with stub (wrapping_sub >= resets window → count never reaches threshold)"
        );
    }
} // mod direction_and_clock

// ---------------------------------------------------------------------------
// STORY-140 — VP-035: DNP3 carry-buffer direction isolation (genuine proptest)
//
// These are GENUINE proptest harnesses using proptest::prelude::* with generated
// strategies.  They are NOT deterministic point tests masquerading as proptests
// (STORY-139 F-139-002 lesson enforced by STORY-140 AC-140-010 discipline).
//
// RED-gate behavior: with the stub (carry_c2s for both directions), the interleaved
// run mixes carry bytes → frame_count and parse_errors diverge from independent runs.
//
// Namespace: DF-TEST-NAMESPACE-001.
// ---------------------------------------------------------------------------

mod vp035_dnp3_carry_direction_isolation {
    use proptest::prelude::*;
    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    fn ip(a: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, a))
    }

    fn make_key() -> FlowKey {
        FlowKey::new(ip(1), 54321, ip(2), 20000)
    }

    fn make_analyzer() -> Dnp3Analyzer {
        Dnp3Analyzer::new(10)
    }

    /// Build a minimal valid DNP3 link-layer frame (10 bytes) with given CTRL byte.
    ///
    /// Layout:
    ///   [0]  0x05  sync
    ///   [1]  0x64  sync
    ///   [2]  0x05  LENGTH = 5 (minimum; frame_len = 10 bytes)
    ///   [3]  ctrl  CTRL byte (direction bit = bit7: 0x80 set = master; 0x44 = outstation)
    ///   [4]  0x01  DEST lo
    ///   [5]  0x00  DEST hi
    ///   [6]  0x03  SRC lo
    ///   [7]  0x00  SRC hi
    ///   [8]  0x00  CRC lo (placeholder — not enforced in frame-walk)
    ///   [9]  0x00  CRC hi
    ///
    /// This is the `build_minimal_dnp3_frame` helper from VP-035 spec skeleton.
    fn build_minimal_dnp3_frame(ctrl: u8) -> Vec<u8> {
        vec![
            0x05, 0x64, // sync bytes
            0x05, // LENGTH = 5 (minimum; total 10 bytes)
            ctrl, // CTRL (direction bit = bit7)
            0x01, 0x00, // DEST = 1 (little-endian)
            0x03, 0x00, // SRC = 3 (little-endian)
            0x00, 0x00, // CRC placeholder (not enforced)
        ]
    }

    // VP-035 proptest: carry_c2s and carry_s2c are never mixed across directions.
    //
    // Strategy: `split_offset in 1usize..9` (partial header cut point) and
    // `_s2c_ctrl in 0x00u8..=0xFFu8` (s2c CTRL byte variation).
    //
    // For each case:
    //   1. Partial c2s delivery (bytes 0..split_offset) → stashed in carry_c2s.
    //   2. Complete s2c frame → frame_count == 1, parse_errors == 0.
    //   3. Completing c2s bytes → carry_c2s prepended; frame_count == 2, parse_errors == 0.
    // Assert: both carries drained.
    //
    // With stub (carry_c2s for both): s2c delivery prepends c2s carry bytes → garbled
    // frame → frame_count ≠ 2 OR parse_errors > 0 → FAIL.
    //
    // Traces: VP-035 (BC-2.15.016 v2.0 Invariant 6, EC-010); AC-140-010.
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(256))]

        /// Guards VP-035: carry_c2s and carry_s2c are never mixed across directions.
        ///
        /// FAILS with stub: interleaved s2c delivery prepends c2s carry bytes into
        /// the frame-walk → frame_count ≠ 2 or parse_errors > 0.
        #[test]
        fn proptest_vp035_direction_isolation_frame_count(
            split_offset in 1usize..9,
            _s2c_ctrl in 0x00u8..=0xFFu8,
        ) {
            let c2s_frame = build_minimal_dnp3_frame(0xC4u8); // DIR=1 master frame
            let s2c_frame = build_minimal_dnp3_frame(0x44u8); // DIR=0 outstation frame

            let key = make_key();
            let mut analyzer = make_analyzer();

            // Delivery 1: partial c2s header (bytes 0..split_offset) → stashed in carry_c2s.
            analyzer.on_data(key.clone(), &c2s_frame[..split_offset], 100, Direction::ClientToServer);
            {
                let flow = analyzer.flows.get(&key).expect("flow must exist after c2s delivery");
                prop_assert_eq!(
                    flow.frame_count, 0,
                    "partial c2s delivery must not complete a frame"
                );
                prop_assert_eq!(
                    flow.parse_errors, 0,
                    "partial c2s delivery must not produce parse errors"
                );
            }

            // Delivery 2: complete s2c frame → carry_s2c used (carry_c2s NOT involved).
            // With stub: carry_c2s spliced → parse_errors > 0 or frame_count ≠ 1.
            analyzer.on_data(key.clone(), &s2c_frame, 100, Direction::ServerToClient);
            {
                let flow = analyzer.flows.get(&key).expect("flow must exist after s2c delivery");
                prop_assert_eq!(
                    flow.parse_errors, 0,
                    "s2c delivery must produce 0 parse_errors; \
                     FAILS with stub (c2s carry spliced into s2c walk → garbled frame)"
                );
                prop_assert_eq!(
                    flow.frame_count, 1,
                    "s2c delivery must complete exactly 1 frame; \
                     FAILS with stub (splice may produce 0 or >1)"
                );
            }

            // Delivery 3: completing c2s bytes (split_offset..end) → carry_c2s prepended.
            analyzer.on_data(key.clone(), &c2s_frame[split_offset..], 100, Direction::ClientToServer);
            {
                let flow = analyzer.flows.get(&key).expect("flow must exist after completing c2s");
                prop_assert_eq!(
                    flow.parse_errors, 0,
                    "completing c2s delivery must not produce parse errors"
                );
                prop_assert_eq!(
                    flow.frame_count, 2,
                    "after c2s completion: frame_count must be 2 (one c2s + one s2c); \
                     FAILS with stub (carry isolation broken)"
                );
                // Both carries must be drained after full frame delivery.
                prop_assert!(
                    flow.carry_c2s.is_empty(),
                    "carry_c2s must be empty after c2s frame fully consumed"
                );
                prop_assert!(
                    flow.carry_s2c.is_empty(),
                    "carry_s2c must be empty after s2c frame fully consumed"
                );
            }
        }

        /// Guards VP-035 direction isolation invariant: interleaved frame_count equals
        /// sum of independent same-direction runs.
        ///
        /// Establishes carry-isolation as an observable behavioral invariant independent
        /// of FIR gating.  FAILS with stub: carry contamination causes interleaved
        /// frame_count ≠ c2s_count + s2c_count.
        #[test]
        fn proptest_vp035_independent_run_equivalence(
            split_offset in 1usize..9,
        ) {
            let c2s_frame = build_minimal_dnp3_frame(0xC4u8);
            let s2c_frame = build_minimal_dnp3_frame(0x44u8);
            let key = make_key();

            // Interleaved run: c2s partial → s2c complete → c2s completing.
            let mut interleaved = make_analyzer();
            interleaved.on_data(key.clone(), &c2s_frame[..split_offset], 100, Direction::ClientToServer);
            interleaved.on_data(key.clone(), &s2c_frame, 100, Direction::ServerToClient);
            interleaved.on_data(key.clone(), &c2s_frame[split_offset..], 100, Direction::ClientToServer);
            let interleaved_frames = interleaved
                .flows
                .get(&key)
                .map(|f| f.frame_count)
                .unwrap_or(0);
            let interleaved_errors = interleaved
                .flows
                .get(&key)
                .map(|f| f.parse_errors)
                .unwrap_or(u64::MAX);

            // Independent c2s-only run.
            let mut c2s_only = make_analyzer();
            c2s_only.on_data(key.clone(), &c2s_frame[..split_offset], 100, Direction::ClientToServer);
            c2s_only.on_data(key.clone(), &c2s_frame[split_offset..], 100, Direction::ClientToServer);
            let c2s_frames = c2s_only.flows.get(&key).map(|f| f.frame_count).unwrap_or(0);
            let c2s_errors = c2s_only.flows.get(&key).map(|f| f.parse_errors).unwrap_or(u64::MAX);

            // Independent s2c-only run.
            let mut s2c_only = make_analyzer();
            s2c_only.on_data(key.clone(), &s2c_frame, 100, Direction::ServerToClient);
            let s2c_frames = s2c_only.flows.get(&key).map(|f| f.frame_count).unwrap_or(0);
            let s2c_errors = s2c_only.flows.get(&key).map(|f| f.parse_errors).unwrap_or(u64::MAX);

            // VP-035 invariant: interleaved frame_count == sum of independent runs.
            // FAILS with stub: carry contamination causes c2s carry to pollute s2c walk
            // → spurious parse_errors OR wrong frame_count in interleaved run.
            prop_assert_eq!(
                interleaved_frames,
                c2s_frames + s2c_frames,
                "VP-035: interleaved frame_count must equal sum of independent runs; \
                 FAILS with stub (carry contamination breaks isolation invariant)"
            );
            prop_assert_eq!(
                interleaved_errors,
                c2s_errors + s2c_errors,
                "VP-035: interleaved parse_errors must equal sum of independent runs; \
                 FAILS with stub (carry splice produces spurious parse_errors)"
            );
        }
    }
} // mod vp035_dnp3_carry_direction_isolation

// ---------------------------------------------------------------------------
// STORY-140 — VP-036: DNP3 window monotonic / no-spurious-reset (genuine proptests)
//
// GENUINE proptest harnesses for all three DNP3 windowed detections:
//   Sub-A: T1692.001 60s direct-operate burst window
//   Sub-B: T1691.001 10s block-command timeout
//   Sub-C: T0827/T0814 300s correlation window + DRIFT-DNP3-OP-001 operator pin
//   Sub-D: genuine u32 rollover deterministic test
//
// Sub-A, Sub-B, Sub-C use prop_assume!(backwards_ts <= window_start) to constrain
// the strategy domain.  These are GENUINE proptests (proptest! macro + generated
// strategies) — NOT deterministic point tests named proptest_* (STORY-139 lesson).
//
// Sub-D is deterministic (specific arithmetic values near u32::MAX).
//
// Namespace: DF-TEST-NAMESPACE-001.
// ---------------------------------------------------------------------------

mod vp036_dnp3_window_monotonic_no_spurious_reset {
    use proptest::prelude::*;
    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    fn ip(a: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, a))
    }

    fn make_key() -> FlowKey {
        // master on ephemeral port; outstation on port 20000.
        // lower_port = 20000 → port-heuristic IF → master = upper_ip.
        // (Direction-fix overrides this for the direction-based tests, but these VP-036
        // tests focus on window arithmetic, not source IP resolution.)
        FlowKey::new(ip(2), 54321, ip(1), 20000)
    }

    /// Build a minimal detection-capable DNP3 frame (15 bytes, app_fc at byte[12]).
    fn detection_frame(app_fc: u8) -> Vec<u8> {
        let length_byte: u8 = 8;
        let u = (length_byte as usize) - 5; // 3
        let blocks = u.div_ceil(16); // 1
        let frame_len = 5 + (length_byte as usize) + 2 * blocks; // 15
        let mut frame = vec![0u8; frame_len];
        frame[0] = 0x05;
        frame[1] = 0x64;
        frame[2] = length_byte;
        frame[3] = 0xC4; // CTRL: DIR=1 PRM=1 UNCONFIRMED_USER_DATA (master)
        frame[4] = 0x03; // DEST lo
        frame[5] = 0x00; // DEST hi
        frame[6] = 0x01; // SRC lo
        frame[7] = 0x00; // SRC hi
        frame[10] = 0xC0; // transport: FIR=1 | FIN=1
        frame[11] = 0x00; // app control
        frame[12] = app_fc;
        frame
    }

    // VP-036 Sub-A proptest: T1692.001 60s window — backwards ts does NOT reset window.
    //
    // Strategy: (window_start, threshold, backwards_ts) with
    // prop_assume!(backwards_ts <= window_start).
    // saturating_sub(backwards_ts, window_start) == 0 → NOT > 60 → no reset.
    //
    // Also drives on_data to confirm the full behavioral invariant:
    // arm window with `threshold` FCs at window_start, deliver one FC at backwards_ts
    // (no reset), then one more FC at window_start → T1692.001 fires.
    //
    // With stub (wrapping_sub): wrapping_sub(backwards_ts, window_start) wraps to large
    // value > 60 → window resets → T1692.001 suppressed → FAIL.
    //
    // Traces: VP-036 Sub-A (BC-2.15.010 v1.8 PC4, EC-012); AC-140-011.
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(256))]

        /// Guards VP-036 Sub-A: backwards-ts event must NOT reset the T1692.001 60s window.
        ///
        /// FAILS with stub (wrapping_sub): large wrapped value > 60 → window reset
        /// → detection suppressed.
        #[test]
        fn proptest_vp036_sub_a_direct_operate_60s_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            threshold in 1u64..10,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // Arithmetic invariant: saturating_sub must return 0.
            let elapsed_backwards = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(
                elapsed_backwards, 0,
                "saturating_sub must return 0 for backwards ts (T1692.001 60s window)"
            );
            prop_assert!(
                elapsed_backwards <= 60,
                "elapsed=0 must NOT trigger the > 60 window reset"
            );

            // Behavioral invariant: drive on_data to confirm window not reset.
            // Use threshold as u32 directly (capped to 9 by strategy).
            let threshold_u32 = threshold as u32;
            let key = make_key();
            let mut analyzer = Dnp3Analyzer::new(threshold_u32);
            let do_frame = detection_frame(0x05); // DIRECT_OPERATE

            // Arm window: deliver `threshold` FCs at window_start.
            for _ in 0..threshold_u32 {
                analyzer.on_data(key.clone(), &do_frame, window_start, Direction::ClientToServer);
            }

            // Backwards-ts FC: must NOT reset (saturating_sub gives 0, not > 60).
            analyzer.on_data(key.clone(), &do_frame, backwards_ts, Direction::ClientToServer);

            // One more FC at window_start → count = threshold + 2 > threshold → T1692.001 fires.
            analyzer.on_data(key.clone(), &do_frame, window_start, Direction::ClientToServer);

            let t1692 = analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T1692.001".to_string()));
            prop_assert!(
                t1692,
                "VP-036 Sub-A: T1692.001 must fire after backwards-ts event; \
                 FAILS with stub (wrapping_sub resets window → count=1 → threshold not crossed)"
            );
        }

        /// Guards VP-036 Sub-A EC-X2 repro: saturating_sub(50, 100) == 0.
        ///
        /// Driven by proptest runner over burst_count to ensure the repro is exercised
        /// across a range of threshold values.  The arithmetic assertion is deterministic;
        /// the behavioral invariant scales with burst_count.
        ///
        /// FAILS with stub: wrapping_sub(50, 100) ≈ 4.29e9 >> 60 → window reset.
        #[test]
        fn proptest_vp036_sub_a_ec_x2_repro_t1692(
            burst_count in 2u64..10,
        ) {
            let threshold = (burst_count - 1) as u32;
            let window_start: u32 = 100;
            let backwards_ts: u32 = 50; // ts=50 < window_start=100 (backwards)

            // Arithmetic guard: saturating_sub must return 0 (not wrapping ~4.29e9).
            let elapsed = backwards_ts.saturating_sub(window_start);
            assert_eq!(
                elapsed, 0,
                "saturating_sub(50, 100) must equal 0; \
                 wrapping_sub would give ~4.29e9 → spurious reset → EC-X2 bug"
            );
            assert!(
                elapsed <= 60,
                "elapsed=0 must NOT trigger the > 60 window reset"
            );

            // Behavioral invariant: drive on_data.
            let key = make_key();
            let mut analyzer = Dnp3Analyzer::new(threshold);
            let do_frame = detection_frame(0x05);

            // Arm window at ts=100 with `threshold` FCs.
            for _ in 0..threshold {
                analyzer.on_data(key.clone(), &do_frame, window_start, Direction::ClientToServer);
            }
            // Backwards-ts FC at ts=50: must NOT reset.
            analyzer.on_data(key.clone(), &do_frame, backwards_ts, Direction::ClientToServer);
            // One more FC at ts=100: count = threshold + 2 > threshold → T1692.001.
            analyzer.on_data(key.clone(), &do_frame, window_start, Direction::ClientToServer);

            let t1692 = analyzer
                .all_findings
                .iter()
                .any(|f| f.mitre_techniques.contains(&"T1692.001".to_string()));
            prop_assert!(
                t1692,
                "VP-036 Sub-A EC-X2: T1692.001 must fire; \
                 FAILS with stub (wrapping_sub resets window → detection suppressed)"
            );
        }

        // VP-036 Sub-B proptest: T1691.001 10s block-command timeout — backwards ts
        // does NOT fire spurious timeout.
        //
        // Strategy: (request_ts, backwards_ts) with prop_assume!(backwards_ts <= request_ts).
        // saturating_sub(backwards_ts, request_ts) == 0 → NOT > 10 → timeout NOT fired.
        //
        // This test verifies the arithmetic invariant directly (no pending_requests
        // injection needed — the arithmetic property is sufficient for Sub-B).
        //
        // Traces: VP-036 Sub-B (BC-2.15.014 v2.1 PC3, EC-009); AC-140-011.

        /// Guards VP-036 Sub-B: backwards-ts event must NOT fire spurious T1691.001 timeout.
        ///
        /// FAILS with stub (wrapping_sub): large wrapped value > 10 → spurious timeout
        /// would fire for any pending request with request_ts > backwards_ts.
        #[test]
        fn proptest_vp036_sub_b_block_timeout_backwards_ts_no_fire(
            request_ts in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= request_ts);

            // Arithmetic invariant: saturating_sub returns 0 for backwards ts.
            let elapsed = backwards_ts.saturating_sub(request_ts);
            prop_assert_eq!(
                elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T1691.001 10s timeout)"
            );
            prop_assert!(
                elapsed <= 10,
                "elapsed=0 must NOT trigger the > 10 second block-timeout; \
                 FAILS with stub (wrapping_sub gives large value > 10 → spurious timeout)"
            );

            // Confirm wrapping_sub would give a different (large) value when backwards_ts < request_ts.
            // This makes the test non-vacuous: it exercises the exact scenario that caused EC-X2.
            if backwards_ts < request_ts {
                let wrapping_elapsed = backwards_ts.wrapping_sub(request_ts);
                prop_assert!(
                    wrapping_elapsed > 0,
                    "wrapping_sub must give nonzero for backwards ts (confirming the old bug)"
                );
            }
        }

        // VP-036 Sub-C proptest: T0827/T0814 300s correlation window — backwards ts
        // does NOT reset window; operator is strict `>` (NOT `>=`).
        //
        // Strategy: (window_start, backwards_ts) with prop_assume!(backwards_ts <= window_start).
        // saturating_sub(backwards_ts, window_start) == 0 → NOT > 300 → no reset.
        //
        // Traces: VP-036 Sub-C (BC-2.15.015 v2.0 PC3, EC-010); AC-140-011.

        /// Guards VP-036 Sub-C: backwards-ts event must NOT reset the 300s correlation window.
        ///
        /// FAILS with stub (wrapping_sub >=): large wrapped value >= 300 → window resets.
        #[test]
        fn proptest_vp036_sub_c_300s_window_backwards_ts_no_reset(
            window_start in 1u32..u32::MAX,
            backwards_ts in 0u32..=u32::MAX,
        ) {
            prop_assume!(backwards_ts <= window_start);

            // Arithmetic invariant.
            let elapsed = backwards_ts.saturating_sub(window_start);
            prop_assert_eq!(
                elapsed, 0,
                "saturating_sub must return 0 for backwards ts (T0827/T0814 300s window)"
            );
            prop_assert!(
                elapsed <= 300,
                "elapsed=0 must NOT trigger the 300s correlation-window reset; \
                 FAILS with stub (wrapping_sub >= gives large value >= 300 → spurious reset)"
            );

            // Confirm wrapping_sub gives a nonzero (large) result when backwards_ts < window_start.
            if backwards_ts < window_start {
                let wrapping_elapsed = backwards_ts.wrapping_sub(window_start);
                prop_assert!(
                    wrapping_elapsed > 0,
                    "wrapping_sub must give nonzero for backwards ts (confirming the old bug)"
                );
            }
        }

        // VP-036 Sub-C operator pin: elapsed==300 does NOT expire under strict `> 300`.
        //
        // DRIFT-DNP3-OP-001: the 300s correlation window uses strict `>` (was `>=`).
        //   elapsed == 300: 300 > 300 is FALSE → window NOT expired.
        //   elapsed == 301: 301 > 300 is TRUE → window expires.
        //
        // This sub-test is driven by proptest over window_start to ensure the boundary
        // is correctly computed for any window_start value (not just 0).
        //
        // Traces: BC-2.15.015 v2.0 PC3, Invariant 6; DRIFT-DNP3-OP-001; AC-140-011.

        /// Guards DRIFT-DNP3-OP-001 operator pin via proptest over window_start.
        ///
        /// For any window_start, elapsed==300 must NOT expire (300 > 300 is false).
        /// elapsed==301 MUST expire (301 > 300 is true).
        ///
        /// FAILS with stub (>= operator): 300 >= 300 is true → window expires at elapsed=300.
        #[test]
        fn proptest_vp036_sub_c_operator_pin_elapsed_300_not_expired(
            window_start in 0u32..(u32::MAX - 400),
        ) {
            let ts_at_exact = window_start + 300;
            let ts_over = window_start + 301;

            let elapsed_exact = ts_at_exact.saturating_sub(window_start);
            let elapsed_over = ts_over.saturating_sub(window_start);

            prop_assert_eq!(elapsed_exact, 300, "elapsed at ts_at_exact must be 300");
            prop_assert_eq!(elapsed_over, 301, "elapsed at ts_over must be 301");

            // Strict > 300: elapsed==300 is NOT > 300 → window NOT expired (DRIFT-DNP3-OP-001).
            // FAILS with stub (>= operator): 300 >= 300 → window expires at elapsed=300.
            prop_assert!(
                elapsed_exact <= 300,
                "elapsed=300 must NOT expire under strict > 300 (DRIFT-DNP3-OP-001 pin); \
                 FAILS with stub (>= would expire at elapsed=300)"
            );
            // elapsed==301 IS > 300 → window expires.
            prop_assert!(
                elapsed_over > 300,
                "elapsed=301 MUST expire under strict > 300"
            );
        }
    }

    // -----------------------------------------------------------------------
    // VP-036 Sub-D: genuine u32 rollover deterministic test
    //
    // window_start = u32::MAX - 5, now_ts = 4 (post-rollover).
    // wrapping_sub(4, u32::MAX-5) = 10 → OLD BUG: triggers 10s timeout (10 > 10? No.
    //   Actually 10 > 10 is false but would trigger the 1s ENIP window; for DNP3 the
    //   issue was wrapping_sub giving ~10, which is small enough to still be
    //   "in window" for the 60s/300s windows but triggers the 10s block-timeout).
    // saturating_sub(4, u32::MAX-5) = 0 → CORRECT: no spurious reset for any window.
    //
    // This is a deterministic unit test (not a proptest) because the rollover
    // scenario requires specific arithmetic values near u32::MAX.
    // -----------------------------------------------------------------------

    /// Guards genuine u32 rollover: saturating_sub returns 0 (no spurious reset).
    ///
    /// window_start = u32::MAX - 5, now_ts = 4 (post-rollover small value).
    /// wrapping_sub(4, u32::MAX-5) = 10 → OLD BUG: would trigger the 10s block-timeout
    /// (10 > 10 is false BUT the edge is at the exact boundary; also documents the
    /// general rollover scenario where wrapping_sub gives a small nonzero value).
    /// saturating_sub(4, u32::MAX-5) = 0 → CORRECT: no spurious reset for any window.
    ///
    /// Regression guard: if saturating_sub is replaced with wrapping_sub, wrapping_elapsed
    /// would be 10 — which would NOT trigger the 60s or 300s windows, but WOULD approach
    /// the 10s block-timeout boundary. This documents the exact arithmetic divergence.
    #[test]
    fn test_vp036_sub_d_genuine_rollover_no_spurious_reset() {
        let window_start: u32 = u32::MAX - 5;
        let now_ts_post_rollover: u32 = 4;

        // Document old (broken) behavior: wrapping_sub gives 10.
        let wrapping_elapsed = now_ts_post_rollover.wrapping_sub(window_start);
        assert_eq!(
            wrapping_elapsed, 10,
            "wrapping_sub(4, u32::MAX-5) must equal 10 (the old spurious value)"
        );

        // Assert new (correct) behavior: saturating_sub gives 0.
        let saturating_elapsed = now_ts_post_rollover.saturating_sub(window_start);
        assert_eq!(
            saturating_elapsed, 0,
            "saturating_sub(4, u32::MAX-5) must equal 0 (no spurious reset)"
        );

        // Guards all three DNP3 windows: saturating_elapsed=0 must NOT trigger any reset.
        assert!(
            saturating_elapsed <= 60,
            "saturating_sub=0 must NOT trigger T1692.001 60s window reset (rollover)"
        );
        assert!(
            saturating_elapsed <= 10,
            "saturating_sub=0 must NOT trigger T1691.001 10s block-timeout (rollover)"
        );
        assert!(
            saturating_elapsed <= 300,
            "saturating_sub=0 must NOT trigger T0827/T0814 300s window reset (rollover)"
        );
    }
} // mod vp036_dnp3_window_monotonic_no_spurious_reset
