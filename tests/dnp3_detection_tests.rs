//! Failing tests for STORY-108: DNP3 Direct Detection Emissions.
//!
//! Covers AC-001..AC-012 and edge cases EC-001..EC-008 from the STORY-108 spec.
//! Traces to behavioral contracts: BC-2.15.010, BC-2.15.011, BC-2.15.012,
//! BC-2.15.013, BC-2.15.020, BC-2.15.022.
//!
//! RED GATE: ALL tests in this file MUST FAIL (todo!() panics) before
//! any production logic is added.  Tests compile clean and panic only on
//! the `todo!()` stubs in `detect_control_class_burst`, `detect_restart`,
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
        analyzer.on_data(key.clone(), &frame, 1000);

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
        analyzer.on_data(key.clone(), &frame2, 1001);

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
            analyzer.on_data(key.clone(), &frame, i);
        }
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "AC-002: at count=10 (==threshold) NO finding yet (check is >)"
        );

        // 11th FC — count=11 > threshold=10 → finding MUST be emitted
        let frame_11 = build_detection_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame_11, 10);

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
        // Elapsed is derived from wrapping_sub(10, 0)=10 — deterministic for fixed timestamps.
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
            analyzer.on_data(key.clone(), &frame, i);
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
    /// Uses `wrapping_sub` semantics: the implementation must check
    /// `now_ts.wrapping_sub(window_start_ts) > DETECTION_WINDOW_SECS`.
    ///
    /// Traces to: BC-2.15.010 postcondition 4; STORY-108 AC-004.
    #[test]
    fn test_t1692_001_window_expiry_resets_counter() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Window 1: deliver 11 FCs within 60s → first finding emitted
        for i in 0..11u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i); // ts 0..10
        }
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-004: first window should produce one finding"
        );

        // Advance time past 60s: window_start_ts=0, now_ts=61 → elapsed=61 > 60
        // Send a new Control FC — this should RESET the window (not fire another finding yet)
        let frame_reset = build_detection_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame_reset, 61);

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
            analyzer.on_data(key.clone(), &frame, 62 + i);
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
        analyzer.on_data(key.clone(), &frame, 1000);

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
        analyzer.on_data(key.clone(), &frame, 1000);

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
    /// This test has a RED GATE anchor: it first verifies that COLD_RESTART (0x0D)
    /// DOES increment restart_event_count (requiring detect_restart to be implemented),
    /// then verifies INITIALIZE_DATA (0x0F) does NOT.  Without the implementation,
    /// the COLD_RESTART counter assertion fails first (count stays 0), anchoring
    /// the Red Gate.
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
        analyzer.on_data(key.clone(), &cold_frame, 500);

        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-005c pre-condition: COLD_RESTART must emit one T0814 finding (Red Gate: stub panics)"
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
        analyzer.on_data(key.clone(), &init_frame, 600);

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
        analyzer.on_data(key.clone(), &frame, 1000);

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
            analyzer.on_data(key.clone(), &frame, i);
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
        analyzer.on_data(key.clone(), &cold, 100);

        // WARM_RESTART second
        let warm = build_detection_frame(0x0E, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &warm, 110);

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
        analyzer.on_data(key.clone(), &init_frame, 0);

        // Deliver COLD_RESTART — should push T0814 (one slot left)
        let cold1 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold1, 100);

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
        analyzer.on_data(key.clone(), &cold2, 110);

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
        analyzer.on_data(key.clone(), &cold, 100);

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
            analyzer.on_data(key.clone(), &frame, i);
        }

        // 3 READ (FC=0x01)
        for i in 0..3u32 {
            let frame = build_detection_frame(0x01, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, 100 + i);
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

        let total_parse_errors = summary
            .detail
            .get("total_parse_errors")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        assert_eq!(
            total_parse_errors, 0,
            "AC-011: total_parse_errors must be 0 when no flows processed"
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
            analyzer.on_data(key.clone(), &frame, i);
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
            analyzer.on_data(key.clone(), &frame, i);
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
    /// Condition (b) is the Red Gate anchor: until detect_control_class_burst is
    /// implemented, direct_operate_count will be 0 (stub does not mutate state),
    /// causing this test to fail on the counter assertion.
    ///
    /// Traces to: BC-2.15.010 EC-002; STORY-108 EC-002.
    #[test]
    fn test_EC_002_no_finding_at_exact_threshold() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Use SELECT (0x03) — same Control-class, different FC
        for i in 0..10u32 {
            let frame = build_detection_frame(0x03, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &frame, i);
        }

        // (a) No T1692.001 finding at count=10 (10 > 10 is false)
        assert_eq!(
            analyzer.all_findings.len(),
            0,
            "EC-002: at count=10 (==threshold=10) no finding expected (10 > 10 is false)"
        );

        // (b) Counter must actually be 10 — RED GATE: todo!() stub leaves count=0
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after 10 on_data calls");
        assert_eq!(
            flow.direct_operate_count, 10,
            "EC-002: direct_operate_count must be 10 after 10 SELECT FCs (Red Gate: stub leaves 0)"
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
        analyzer.on_data(key.clone(), &cold1, 100);

        let cold2 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold2, 200);

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
        analyzer.on_data(key.clone(), &cold, 500);

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
            analyzer.on_data(key.clone(), &frame, i);
        }

        // Deliver 1 WRITE → T0836
        let write_frame = build_detection_frame(0x02, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &write_frame, 12);

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

    /// EC-008: `now_ts.wrapping_sub(window_start_ts)` must not panic with out-of-order
    /// timestamps (e.g. pcap replay where timestamps go backward).
    ///
    /// Scenario: seed window at ts=0xFFFFFFF0; new FC at ts=0x00000005.
    /// wrapping_sub(0x5, 0xFFFFFFF0) = 0x00000015 = 21 seconds (within 60s window).
    /// Plain subtraction (0x5 - 0xFFFFFFF0) would overflow with overflow-checks=true.
    ///
    /// Traces to: BC-2.15.010 invariant (wrapping_sub required); STORY-108 EC-008.
    #[test]
    fn test_EC_008_wrapping_sub_out_of_order_timestamp_no_panic() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Seed the window near u32::MAX
        let ts_start: u32 = 0xFFFFFFF0;
        let frame_seed = build_detection_frame(0x05, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame_seed, ts_start);

        // Deliver 10 more frames at ts that wraps around (0..=9)
        // wrapping_sub(0x9, 0xFFFFFFF0) = 0x19 = 25 < 60 → still in same window
        for i in 0..10u32 {
            let frame = build_detection_frame(0x05, 0x0003, 0x0001);
            // Must NOT panic (plain subtraction would panic due to overflow-checks=true)
            analyzer.on_data(key.clone(), &frame, i);
        }

        // If we got here without panic, wrapping_sub is working.
        // Count must be >= 11 in the window (or reset depending on wrapping delta):
        // wrapping_sub(0, 0xFFFFFFF0) = 0x10 = 16 < 60 → same window
        // wrapping_sub(9, 0xFFFFFFF0) = 0x19 = 25 < 60 → same window
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
            analyzer.on_data(key.clone(), &frame, i);
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
    // total_parse_errors in summary
    // -----------------------------------------------------------------------

    /// Verifies summarize() includes total_parse_errors (BC-2.15.020 postcondition 1).
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
        analyzer.on_data(key.clone(), &bad_frame, 0);

        let summary = analyzer.summarize();

        // total_parse_errors must be present
        assert!(
            summary.detail.contains_key("total_parse_errors"),
            "BC-2.15.020: total_parse_errors key must be present in summary detail"
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
            analyzer.on_data(key.clone(), &frame, i);
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
            analyzer.on_data(key.clone(), &frame, i);
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
            analyzer.on_data(key.clone(), &frame, i);
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
} // mod story_108
