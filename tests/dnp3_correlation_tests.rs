//! Failing tests for STORY-109: DNP3 Correlated/Derived + Anomaly Detections.
//!
//! Covers AC-001..AC-014 and edge cases EC-001..EC-010 from the STORY-109 spec.
//! Traces to behavioral contracts: BC-2.15.014, BC-2.15.015, BC-2.15.018,
//! BC-2.15.019, BC-2.15.023, BC-2.15.024.
//!
//! RED GATE: ALL tests in `mod story_109` MUST FAIL (todo!() panics or assertion
//! failures) before any production logic is added.  Tests compile clean.
//!
//! Test naming convention: exact names mandated by STORY-109 ACs plus
//! `test_EC_NNN_xxx` for edge cases, all in `mod story_109` per DF-TEST-NAMESPACE-001.

// BC-based test naming uses uppercase BC IDs.
#![allow(non_snake_case)]

mod story_109 {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::DateTime;
    use wirerust::analyzer::dnp3::{
        BLOCK_CMD_THRESHOLD, BLOCK_CMD_TIMEOUT_SECS, CORRELATION_WINDOW_SECS, Dnp3Analyzer,
        MALFORMED_ANOMALY_THRESHOLD, T0827_THRESHOLD,
    };
    use wirerust::findings::{Confidence, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

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
    /// Layout (15 bytes — LENGTH=8 → frame_len = 5+8+2*1 = 15):
    ///   [0..2]  0x05 0x64 0x08      (sync + LENGTH=8)
    ///   [3]     0xC4                (CTRL: DIR=1 PRM=1 UNCONFIRMED_USER_DATA)
    ///   [4..5]  dest_lo dest_hi     (little-endian DEST)
    ///   [6..7]  src_lo  src_hi      (little-endian SRC)
    ///   [8..9]  0x00 0x00           (header CRC placeholder)
    ///   [10]    0xC0                (transport: FIR=1, FIN=1)
    ///   [11]    app_seq & 0x0F      (app control with app_seq in lower nibble)
    ///   [12]    app_fc              (application function code)
    ///   [13..14] 0x00 0x00          (data-block CRC placeholder)
    fn build_detection_frame(app_fc: u8, dest: u16, src: u16) -> Vec<u8> {
        build_detection_frame_with_seq(app_fc, dest, src, 0)
    }

    /// Build a detection frame with a specific application sequence number (lower 4 bits).
    fn build_detection_frame_with_seq(app_fc: u8, dest: u16, src: u16, app_seq: u8) -> Vec<u8> {
        // LENGTH=8 → frame_len = 5+8+2*1 = 15 (U=3, blocks=1)
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
        // byte 11: app control — encode app_seq in lower nibble
        frame[11] = app_seq & 0x0F;
        // byte 12: application FC
        frame[12] = app_fc;
        // bytes 13-14: data-block CRC placeholder (0x00)
        frame
    }

    /// Build a minimal invalid DNP3 frame that fails the validity gate (LENGTH < 5).
    /// Used to trigger the malformed / parse_errors paths (BC-2.15.024 precondition 1).
    fn build_invalid_frame_length_too_short() -> Vec<u8> {
        // Start with valid sync, then LENGTH=2 (< 5 minimum) → compute_dnp3_frame_len returns None
        // The carry-buffer will drain this byte and increment parse_errors.
        // We send enough data that the carry-buffer processes the LENGTH=2 byte.
        // Use 10 bytes: sync + LENGTH=2 + rest, so the carry sees a full structure to reject.
        let mut frame = vec![0u8; 10];
        frame[0] = 0x05;
        frame[1] = 0x64;
        frame[2] = 2; // LENGTH=2 < 5 → validity gate REJECT → parse_errors += 1
        frame[3] = 0xC4;
        frame[4] = 0x03;
        frame[5] = 0x00;
        frame[6] = 0x01;
        frame[7] = 0x00;
        frame[8] = 0x00;
        frame[9] = 0x00;
        frame
    }

    // -------------------------------------------------------------------------
    // AC-001 (BC-2.15.014 postcondition 1)
    // test_block_event_count_increments_unconditionally
    // -------------------------------------------------------------------------

    /// AC-001: FC in {0x03, 0x04, 0x05} (NOT 0x06) that receives no RESPONSE within
    /// BLOCK_CMD_TIMEOUT_SECS=10s → block_event_count += 1 UNCONDITIONALLY.
    /// FC=0x06 (DIRECT_OPERATE_NR) is EXCLUDED — does NOT increment block_event_count.
    ///
    /// Strategy: send a DIRECT_OPERATE (0x05) at ts=0, then advance ts to 11
    /// (> 10s timeout) with another frame to trigger the scan. Verify
    /// block_event_count=1. Then send FC=0x06 with a 10s+ advance and verify
    /// block_event_count is NOT incremented.
    ///
    /// Traces to: BC-2.15.014 postcondition 1; STORY-109 AC-001.
    #[test]
    fn test_block_event_count_increments_unconditionally() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Send a DIRECT_OPERATE (FC=0x05) with app_seq=1 at ts=0
        let frame = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 1);
        analyzer.on_data(key.clone(), &frame, 0);

        // No response — advance past BLOCK_CMD_TIMEOUT_SECS (10s) with a new frame at ts=11
        // The block-timeout scan fires during this on_data call.
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001); // READ — just to advance ts
        analyzer.on_data(key.clone(), &trigger, 11);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.block_event_count, 1,
            "AC-001: DIRECT_OPERATE (0x05) timeout must set block_event_count=1 \
             (unconditional, regardless of T1691.001 threshold)"
        );

        // Now send FC=0x06 (DIRECT_OPERATE_NR) — should NOT be tracked in pending_requests,
        // so no block event when timeout fires.
        let frame_nr = build_detection_frame_with_seq(0x06, 0x0003, 0x0001, 2);
        analyzer.on_data(key.clone(), &frame_nr, 12);

        // Advance past 10s from ts=12 (so ts=23+): block_event_count must stay at 1
        let trigger2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger2, 23);

        let flow = analyzer.flows.get(&key).expect("flow must still exist");
        assert_eq!(
            flow.block_event_count, 1,
            "AC-001: FC=0x06 (DIRECT_OPERATE_NR) MUST NOT increment block_event_count \
             (excluded from block-command timeout tracking)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-002 (BC-2.15.014 postcondition 3)
    // test_t1691_001_emitted_at_threshold_3_of_300s
    // -------------------------------------------------------------------------

    /// AC-002: 3 DIRECT_OPERATE requests each without response within 10s →
    /// T1691.001 emitted at 3rd event; verdict Possible, confidence Low.
    ///
    /// Test sequence:
    ///   ts=0: send DIRECT_OPERATE (app_seq=1)
    ///   ts=11: advance (no response) → block_event_count=1
    ///   ts=22: advance (no response) → block_event_count=2
    ///   ts=33: advance (no response) → block_event_count=3 → T1691.001 emitted
    ///
    /// Traces to: BC-2.15.014 postcondition 3; Canonical Test Vector §1; STORY-109 AC-002.
    #[test]
    fn test_t1691_001_emitted_at_threshold_3_of_300s() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 3 separate DIRECT_OPERATE requests, each with distinct app_seq,
        // each timing out (no response within 10s).
        // Interleave with READ frames (ts+11 each) to trigger the scan.
        for i in 0u32..3 {
            let base_ts = i * 11; // ts = 0, 11, 22
            let frame = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, i as u8);
            analyzer.on_data(key.clone(), &frame, base_ts);
            // Advance 11 seconds: triggers block-timeout scan for all pending_requests
            // with wrapping_sub(base_ts + 11, base_ts) = 11 > BLOCK_CMD_TIMEOUT_SECS (10)
            let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &trigger, base_ts + 11);
        }

        // Exactly one T1691.001 finding must have been emitted
        let t1691_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1691.001".to_string()))
            .collect();

        assert_eq!(
            t1691_findings.len(),
            1,
            "AC-002: exactly ONE T1691.001 finding at 3rd block event (threshold=3); \
             got {} findings",
            t1691_findings.len()
        );

        let f = t1691_findings[0];
        assert!(
            matches!(f.verdict, Verdict::Possible),
            "AC-002: T1691.001 verdict must be Possible; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Low),
            "AC-002: T1691.001 confidence must be Low; got {:?}",
            f.confidence
        );
        assert_eq!(
            f.mitre_techniques,
            vec!["T1691.001"],
            "AC-002: mitre_techniques must be exactly [\"T1691.001\"]"
        );

        // Verify the one-shot guard is set
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.block_finding_emitted_this_window,
            "AC-002: block_finding_emitted_this_window must be true after emission"
        );
        assert_eq!(
            flow.block_event_count, 3,
            "AC-002: block_event_count must be 3 after 3 timeouts"
        );
    }

    // -------------------------------------------------------------------------
    // AC-003 (BC-2.15.014 postcondition 2 / invariant 7 — single shared 300s window)
    // test_block_events_not_reset_at_120s
    // -------------------------------------------------------------------------

    /// AC-003: Two block events spaced 150s apart, both within 300s window.
    /// Under the old 120s design, the first event would be lost at t=120s.
    /// This test verifies block_event_count=2 (NOT reset at 120s or 150s).
    ///
    /// Key trace (Trace B precursor): block at t=0, block at t=150s; both within 300s
    /// window. block_event_count must be 2 after both timeouts fire.
    ///
    /// Traces to: BC-2.15.014 postcondition 2/invariant 7; STORY-109 AC-003.
    #[test]
    fn test_block_events_not_reset_at_120s() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Block event #1: DIRECT_OPERATE at ts=0, timeout at ts=11
        let frame1 = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 0);
        analyzer.on_data(key.clone(), &frame1, 0);
        let trigger1 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger1, 11);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.block_event_count, 1,
                "AC-003: block_event_count must be 1 after first timeout"
            );
        }

        // Block event #2: DIRECT_OPERATE at ts=150, timeout at ts=161
        // 150s < 300s window — must NOT reset block_event_count.
        let frame2 = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 1);
        analyzer.on_data(key.clone(), &frame2, 150);
        let trigger2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger2, 161);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.block_event_count, 2,
            "AC-003: block_event_count must be 2 after two timeouts spaced 150s apart \
             (both within 300s; NOT reset at 120s — single shared 300s window)"
        );
        // No T1691.001 yet (only 2 events, threshold=3)
        let t1691_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1691.001".to_string()))
            .count();
        assert_eq!(
            t1691_count, 0,
            "AC-003: no T1691.001 at count=2 (below threshold of 3)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-004 (BC-2.15.015 postconditions 1/2 — T0827 Trace B)
    // test_t0827_emitted_at_combined_threshold
    // -------------------------------------------------------------------------

    /// AC-004: Trace B — 2 block events + 1 restart = combined 3 → T0827 emitted.
    ///
    /// Timeline:
    ///   ts=0:   block timeout #1 → block_event_count=1
    ///   ts=150: block timeout #2 → block_event_count=2
    ///   ts=200: COLD_RESTART → restart_event_count=1 + T0814 + T0827
    ///
    /// T0827 assertions:
    ///   - mitre_techniques: vec!["T0827"]
    ///   - category: Impact (ThreatCategory::Impact)
    ///   - tactic: IcsImpact (tested via technique_tactic lookup)
    ///   - loss_of_control_emitted = true
    ///   - T0827 appears AFTER the triggering T0814 in all_findings (BC-2.15.013 ordering)
    ///
    /// Traces to: BC-2.15.015 postconditions 1/2; Canonical Test Vector Trace B; STORY-109 AC-004.
    #[test]
    fn test_t0827_emitted_at_combined_threshold() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Block event #1: ts=0 → timeout at ts=11
        let frame1 = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 0);
        analyzer.on_data(key.clone(), &frame1, 0);
        let t1 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &t1, 11);

        // Block event #2: ts=150 → timeout at ts=161 (still within 300s window)
        let frame2 = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 1);
        analyzer.on_data(key.clone(), &frame2, 150);
        let t2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &t2, 161);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(
                flow.block_event_count, 2,
                "pre-condition: block_event_count=2"
            );
            assert_eq!(
                flow.restart_event_count, 0,
                "pre-condition: restart_event_count=0"
            );
        }
        assert_eq!(
            analyzer
                .all_findings
                .iter()
                .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
                .count(),
            0,
            "AC-004 pre-condition: no T0827 yet (combined=2 < threshold=3)"
        );

        // COLD_RESTART at ts=200: pushes T0814 then T0827 (combined=3 >= threshold=3)
        let restart = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &restart, 200);

        // Must have T0827
        let t0827_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .collect();
        assert_eq!(
            t0827_findings.len(),
            1,
            "AC-004: exactly ONE T0827 finding when combined=3 (2 block + 1 restart)"
        );

        let f = t0827_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0827"],
            "AC-004: T0827 finding must carry only [\"T0827\"]"
        );
        assert!(
            matches!(f.category, ThreatCategory::Impact),
            "AC-004: T0827 category must be Impact; got {:?}",
            f.category
        );

        // Verify tactic via the mitre catalog
        let tactic = wirerust::mitre::technique_tactic("T0827");
        assert_eq!(
            tactic,
            Some(wirerust::mitre::MitreTactic::IcsImpact),
            "AC-004: T0827 tactic must be IcsImpact (NEW variant)"
        );

        // BC-2.15.013 ordering: T0827 must appear AFTER T0814 in all_findings
        // within the same on_data call (STORY-108-deferred ordering now enforced).
        let t0814_pos = analyzer
            .all_findings
            .iter()
            .position(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .expect("AC-004: T0814 (COLD_RESTART) must be present before T0827");
        let t0827_pos = analyzer
            .all_findings
            .iter()
            .position(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .expect("AC-004: T0827 must be present");
        assert!(
            t0827_pos > t0814_pos,
            "AC-004 (BC-2.15.013 PC2): T0827 [{t0827_pos}] must appear AFTER T0814 [{t0814_pos}] \
             in all_findings (derived finding after triggering direct finding)"
        );

        // One-shot guard
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.loss_of_control_emitted,
            "AC-004: loss_of_control_emitted must be true after T0827 emission"
        );
    }

    // -------------------------------------------------------------------------
    // AC-005 (BC-2.15.015 postcondition 3 — window expiry resets SIX fields)
    // test_correlation_window_expiry_resets_six_fields
    // -------------------------------------------------------------------------

    /// AC-005: Advance past CORRELATION_WINDOW_SECS (300s) → all SIX windowed fields
    /// reset; parse_errors UNCHANGED.
    ///
    /// Pre-conditions: populate each of the six fields with non-default values by
    /// sending malformed frames and restart frames. Then advance past 300s.
    ///
    /// Six fields that MUST reset: restart_event_count, block_event_count,
    /// block_finding_emitted_this_window, loss_of_control_emitted,
    /// malformed_in_window, malformed_anomaly_emitted.
    ///
    /// parse_errors MUST NOT reset (lifetime counter).
    ///
    /// Traces to: BC-2.15.015 postcondition 3; BC-2.15.024 Invariant 1; STORY-109 AC-005.
    #[test]
    fn test_correlation_window_expiry_resets_six_fields() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Seed parse_errors via invalid-LENGTH frames.
        // Each LENGTH=2 frame causes parse_errors += 1 via the carry-buffer validity gate.
        // Deliver them as raw bytes (the carry-buffer processes them, drain+parse_error).
        for _ in 0..2u32 {
            let malformed = build_invalid_frame_length_too_short();
            analyzer.on_data(key.clone(), &malformed, 0);
        }

        // Seed restart_event_count (and loss_of_control_emitted via T0827)
        // by doing 3 COLD_RESTARTs at ts=0..2.
        for i in 0u32..3 {
            let restart = build_detection_frame(0x0D, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &restart, i);
        }

        // Verify pre-conditions: some windowed state is set.
        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert!(
                flow.restart_event_count >= 3,
                "AC-005 pre: restart_event_count must be >= 3"
            );
            assert!(
                flow.loss_of_control_emitted,
                "AC-005 pre: loss_of_control_emitted must be true"
            );
            // parse_errors must be at least 2 from the malformed frames
            let parse_errors_before = flow.parse_errors;
            assert!(
                parse_errors_before >= 2,
                "AC-005 pre: parse_errors must be >= 2 from malformed frames"
            );
        }

        // Now advance past the 300s correlation window.
        // correlation_window_start_ts was seeded at ts=0; now send a frame at ts=301
        // (wrapping_sub(301, 0) = 301 >= CORRELATION_WINDOW_SECS=300 → window expires).
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 301);

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        // All SIX windowed fields must be reset.
        assert_eq!(
            flow.restart_event_count, 0,
            "AC-005: restart_event_count must be 0 after window expiry"
        );
        assert_eq!(
            flow.block_event_count, 0,
            "AC-005: block_event_count must be 0 after window expiry"
        );
        assert!(
            !flow.block_finding_emitted_this_window,
            "AC-005: block_finding_emitted_this_window must be false after window expiry"
        );
        assert!(
            !flow.loss_of_control_emitted,
            "AC-005: loss_of_control_emitted must be false after window expiry"
        );
        assert_eq!(
            flow.malformed_in_window, 0,
            "AC-005: malformed_in_window must be 0 after window expiry"
        );
        assert!(
            !flow.malformed_anomaly_emitted,
            "AC-005: malformed_anomaly_emitted must be false after window expiry"
        );

        // parse_errors MUST NOT be reset (lifetime/monotonic counter).
        assert!(
            flow.parse_errors >= 2,
            "AC-005: parse_errors must STILL be >= 2 after window expiry (never reset) \
             — lifetime counter per BC-2.15.024 Invariant 1"
        );
    }

    // -------------------------------------------------------------------------
    // AC-006 (BC-2.15.015 invariant 7 — distinct-event guard)
    // test_t0827_requires_distinct_events
    // -------------------------------------------------------------------------

    /// AC-006: 2 restarts + 0 block events = combined 2 < threshold=3 → no T0827.
    ///
    /// Verifies that two restart events alone are insufficient for T0827 emission.
    ///
    /// Traces to: BC-2.15.015 invariant 7; BC-2.15.015 EC-001; STORY-109 AC-006.
    #[test]
    fn test_t0827_requires_distinct_events() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 2 COLD_RESTARTs — restart_event_count=2 but block_event_count=0
        let r1 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &r1, 0);
        let r2 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &r2, 10);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 2,
            "AC-006: restart_event_count must be 2 after 2 COLD_RESTARTs"
        );
        assert_eq!(
            flow.block_event_count, 0,
            "AC-006: block_event_count must be 0 (no block timeouts)"
        );

        let t0827_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .count();
        assert_eq!(
            t0827_count, 0,
            "AC-006: T0827 must NOT be emitted when combined=2 (< threshold=3); \
             2 restarts + 0 blocks is insufficient"
        );
        assert!(
            !flow.loss_of_control_emitted,
            "AC-006: loss_of_control_emitted must be false (T0827 not emitted)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-007 (BC-2.15.018 postcondition 1)
    // test_broadcast_control_anomaly_fires_for_dest_ffff
    // test_broadcast_read_no_anomaly
    // -------------------------------------------------------------------------

    /// AC-007a: Control-class FC to dest=0xFFFF (broadcast) → T1692.001 anomaly finding.
    ///
    /// Expected: category Suspicious, verdict Possible, confidence Medium.
    /// direct_operate_count ALSO incremented (broadcast Control feeds burst threshold).
    ///
    /// Traces to: BC-2.15.018 postcondition 1/2; Canonical Test Vector; STORY-109 AC-007.
    #[test]
    fn test_broadcast_control_anomaly_fires_for_dest_ffff() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // DIRECT_OPERATE (FC=0x05) to broadcast dest=0xFFFF
        let frame = build_detection_frame(0x05, 0xFFFF, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000);

        // Must have exactly one T1692.001 finding with Suspicious/Possible/Medium
        let broadcast_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T1692.001".to_string())
                    && matches!(f.category, ThreatCategory::Suspicious)
            })
            .collect();
        assert_eq!(
            broadcast_findings.len(),
            1,
            "AC-007: exactly ONE T1692.001 Suspicious finding for broadcast DIRECT_OPERATE \
             to 0xFFFF; got {}",
            broadcast_findings.len()
        );

        let f = broadcast_findings[0];
        assert!(
            matches!(f.verdict, Verdict::Possible),
            "AC-007: broadcast anomaly verdict must be Possible; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Medium),
            "AC-007: broadcast anomaly confidence must be Medium; got {:?}",
            f.confidence
        );
        assert_eq!(
            f.mitre_techniques,
            vec!["T1692.001"],
            "AC-007: broadcast anomaly must carry only [\"T1692.001\"]"
        );

        // source_ip must be Some (BC-2.15.018 postcondition 1)
        assert!(
            f.source_ip.is_some(),
            "AC-007: source_ip must be Some for broadcast anomaly finding"
        );
        // timestamp must be Some (BC-2.15.018 postcondition 1)
        assert!(
            f.timestamp.is_some(),
            "AC-007: timestamp must be Some for broadcast anomaly finding"
        );

        // direct_operate_count must ALSO be incremented (BC-2.15.018 postcondition 2)
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.direct_operate_count >= 1,
            "AC-007: direct_operate_count must be >= 1 (broadcast Control increments it)"
        );
    }

    /// AC-007b: READ (FC=0x01) to broadcast dest=0xFFFF → NO broadcast anomaly.
    ///
    /// READ to broadcast is legitimate (global Class 0 poll). Only Control-class FCs trigger
    /// BC-2.15.018.
    ///
    /// Traces to: BC-2.15.018 invariant 2; BC-2.15.018 EC-004; STORY-109 AC-007.
    #[test]
    fn test_broadcast_read_no_anomaly() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // READ (FC=0x01) to broadcast dest=0xFFFF — no anomaly expected
        let frame = build_detection_frame(0x01, 0xFFFF, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000);

        // No T1692.001 findings from a READ to broadcast
        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();
        assert_eq!(
            t1692_count, 0,
            "AC-007: READ (FC=0x01) to broadcast dest=0xFFFF must NOT emit T1692.001 \
             (broadcast anomaly only for Control-class FCs)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-008 (BC-2.15.018 invariant 4 — broadcast + burst both retained)
    // test_broadcast_and_burst_both_retained
    // -------------------------------------------------------------------------

    /// AC-008: 11 broadcast Control FCs → broadcast anomaly finding(s) PLUS a burst finding.
    /// Implementation must not deduplicate on technique ID alone — both findings retained.
    ///
    /// After 11 broadcast DIRECT_OPERATE (0x05) frames to dest=0xFFFF:
    ///   - Each frame emits a BC-2.15.018 broadcast anomaly (Suspicious/Possible/Medium T1692.001)
    ///   - direct_operate_count incremented for each → after 11th, burst threshold (>10) crossed
    ///   - BC-2.15.010 emits a SECOND T1692.001 (Execution/Likely/Medium burst finding)
    ///   - Total T1692.001 findings > 1
    ///
    /// Traces to: BC-2.15.018 invariant 4; BC-2.15.013 invariant 4; STORY-109 AC-008.
    #[test]
    fn test_broadcast_and_burst_both_retained() {
        let mut analyzer = Dnp3Analyzer::new(10); // threshold=10
        let key = test_flow_key();

        // 11 broadcast DIRECT_OPERATE frames to dest=0xFFFF
        for i in 0..11u32 {
            let frame = build_detection_frame(0x05, 0xFFFF, 0x0001);
            analyzer.on_data(key.clone(), &frame, i);
        }

        // Total T1692.001 findings: must be > 1 (both broadcast anomaly + burst retained)
        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();
        assert!(
            t1692_count > 1,
            "AC-008: > 1 T1692.001 findings expected after 11 broadcast Control FCs \
             (both broadcast anomaly and burst threshold findings retained; \
             no dedup by technique ID); got {}",
            t1692_count
        );

        // Verify that both finding subtypes are present:
        // - At least one Suspicious (broadcast anomaly)
        let has_suspicious = analyzer.all_findings.iter().any(|f| {
            f.mitre_techniques.contains(&"T1692.001".to_string())
                && matches!(f.category, ThreatCategory::Suspicious)
        });
        assert!(
            has_suspicious,
            "AC-008: must have at least one Suspicious T1692.001 (broadcast anomaly)"
        );

        // - At least one Execution (burst threshold, BC-2.15.010)
        let has_execution = analyzer.all_findings.iter().any(|f| {
            f.mitre_techniques.contains(&"T1692.001".to_string())
                && matches!(f.category, ThreatCategory::Execution)
        });
        assert!(
            has_execution,
            "AC-008: must have at least one Execution T1692.001 (burst threshold finding, BC-2.15.010)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-009 (BC-2.15.019 postconditions 1/2)
    // test_unsolicited_response_anomaly_no_prior_enable
    // test_unsolicited_response_no_anomaly_after_enable
    // -------------------------------------------------------------------------

    /// AC-009a: FC=0x82 (UNSOLICITED_RESPONSE) with no prior ENABLE_UNSOLICITED (FC=0x14)
    /// → T0814 anomaly finding, verdict Possible, confidence Low, one-shot.
    ///
    /// Traces to: BC-2.15.019 postcondition 1/2; Canonical Test Vector; STORY-109 AC-009.
    #[test]
    fn test_unsolicited_response_anomaly_no_prior_enable() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x82 (UNSOLICITED_RESPONSE) — no prior ENABLE_UNSOLICITED on this flow
        let frame = build_detection_frame(0x82, 0x0001, 0x0003);
        analyzer.on_data(key.clone(), &frame, 1000);

        // Must emit exactly ONE T0814 finding (unsolicited anomaly)
        let t0814_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .collect();
        assert_eq!(
            t0814_findings.len(),
            1,
            "AC-009: exactly ONE T0814 finding for UNSOLICITED_RESPONSE with no prior \
             ENABLE_UNSOLICITED; got {}",
            t0814_findings.len()
        );

        let f = t0814_findings[0];
        assert!(
            matches!(f.verdict, Verdict::Possible),
            "AC-009: unsolicited anomaly verdict must be Possible; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Low),
            "AC-009: unsolicited anomaly confidence must be Low; got {:?}",
            f.confidence
        );

        // One-shot guard: second UNSOLICITED_RESPONSE must NOT emit another finding
        let frame2 = build_detection_frame(0x82, 0x0001, 0x0003);
        analyzer.on_data(key.clone(), &frame2, 1001);

        let t0814_count_after = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();
        assert_eq!(
            t0814_count_after, 1,
            "AC-009: one-shot guard: second UNSOLICITED_RESPONSE must NOT emit another T0814"
        );

        // unsolicited_anomaly_emitted flag must be set
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.unsolicited_anomaly_emitted,
            "AC-009: unsolicited_anomaly_emitted must be true after emission"
        );
    }

    /// AC-009b: ENABLE_UNSOLICITED (FC=0x14) observed first → subsequent FC=0x82 NOT anomalous.
    ///
    /// Traces to: BC-2.15.019 postcondition 3; BC-2.15.019 EC-001; STORY-109 AC-009.
    #[test]
    fn test_unsolicited_response_no_anomaly_after_enable() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // First: ENABLE_UNSOLICITED (FC=0x14) — sets enable_unsolicited_seen=true
        let enable_frame = build_detection_frame(0x14, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &enable_frame, 100);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert!(
                flow.enable_unsolicited_seen,
                "AC-009b: enable_unsolicited_seen must be true after FC=0x14"
            );
        }

        // Then: UNSOLICITED_RESPONSE (FC=0x82) — must NOT emit anomaly
        let unsol_frame = build_detection_frame(0x82, 0x0001, 0x0003);
        analyzer.on_data(key.clone(), &unsol_frame, 200);

        // No T0814 anomaly from the unsolicited response (ENABLE_UNSOLICITED was seen)
        // (Note: the ENABLE_UNSOLICITED itself may emit a T0814 per BC-2.15.023 AC-011 —
        //  but NOT from the UNSOLICITED_RESPONSE following it)
        let unsolicited_anomaly_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("unexpected unsolicited response")
            })
            .collect();
        assert_eq!(
            unsolicited_anomaly_findings.len(),
            0,
            "AC-009b: no unsolicited-anomaly T0814 finding when ENABLE_UNSOLICITED was prior; \
             enable_unsolicited_seen=true suppresses the BC-2.15.019 anomaly"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            !flow.unsolicited_anomaly_emitted,
            "AC-009b: unsolicited_anomaly_emitted must remain false when anomaly was suppressed"
        );
    }

    // -------------------------------------------------------------------------
    // AC-010 (BC-2.15.023 postcondition 1 — DISABLE_UNSOLICITED T0814 Likely/Medium)
    // test_disable_unsolicited_emits_t0814_likely_medium
    // -------------------------------------------------------------------------

    /// AC-010: FC=0x15 (DISABLE_UNSOLICITED) on FIR=1 → ONE T0814 Finding,
    /// verdict Likely, confidence Medium, per-occurrence.
    ///
    /// Summary must match exactly (pinned format from BC-2.15.023 postcondition 1):
    /// "DNP3 DISABLE_UNSOLICITED observed: FC 0x15 from src=0x0001 to dest=0x0003 \
    ///  — alarm suppression / event-blinding primitive"
    ///
    /// Detection is on RAW app_fc == 0x15 (NOT via classify_dnp3_fc).
    ///
    /// Traces to: BC-2.15.023 postcondition 1; Canonical Test Vector; STORY-109 AC-010.
    #[test]
    fn test_disable_unsolicited_emits_t0814_likely_medium() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x15 (DISABLE_UNSOLICITED) from src=0x0001 to dest=0x0003
        let frame = build_detection_frame(0x15, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000);

        // Must have exactly one T0814 from DISABLE_UNSOLICITED
        let disable_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("DISABLE_UNSOLICITED")
            })
            .collect();
        assert_eq!(
            disable_findings.len(),
            1,
            "AC-010: exactly ONE T0814 finding for FC=0x15 (DISABLE_UNSOLICITED); got {}",
            disable_findings.len()
        );

        let f = disable_findings[0];
        assert!(
            matches!(f.verdict, Verdict::Likely),
            "AC-010: DISABLE_UNSOLICITED verdict must be Likely; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Medium),
            "AC-010: DISABLE_UNSOLICITED confidence must be Medium; got {:?}",
            f.confidence
        );
        assert_eq!(
            f.mitre_techniques,
            vec!["T0814"],
            "AC-010: DISABLE_UNSOLICITED must carry only [\"T0814\"]"
        );

        // Exact summary pinned to BC-2.15.023 postcondition 1 format.
        // src=0x0001, dest=0x0003 → {:#06X} → "0x0001", "0x0003"
        assert_eq!(
            f.summary,
            "DNP3 DISABLE_UNSOLICITED observed: FC 0x15 from src=0x0001 to dest=0x0003 \
             — alarm suppression / event-blinding primitive",
            "AC-010: DISABLE_UNSOLICITED summary must match exact BC-2.15.023 format string; \
             got: {:?}",
            f.summary
        );

        // source_ip must be Some
        assert!(
            f.source_ip.is_some(),
            "AC-010: source_ip must be Some for DISABLE_UNSOLICITED finding"
        );
        // timestamp must be Some (ts=1000)
        let expected_ts = DateTime::from_timestamp(1000i64, 0);
        assert_eq!(
            f.timestamp, expected_ts,
            "AC-010: timestamp must be Some(ts=1000); got {:?}",
            f.timestamp
        );

        // Per-occurrence: second DISABLE_UNSOLICITED emits another finding
        let frame2 = build_detection_frame(0x15, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame2, 1001);

        let count_after = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("DISABLE_UNSOLICITED")
            })
            .count();
        assert_eq!(
            count_after, 2,
            "AC-010: per-occurrence: 2nd DISABLE_UNSOLICITED must emit 2nd T0814 finding"
        );
    }

    // -------------------------------------------------------------------------
    // AC-011 (BC-2.15.023 postcondition 1 — ENABLE_UNSOLICITED T0814 Possible/Low)
    // test_enable_unsolicited_emits_t0814_possible_low
    // -------------------------------------------------------------------------

    /// AC-011: FC=0x14 (ENABLE_UNSOLICITED) on FIR=1 → ONE T0814 Finding,
    /// verdict Possible, confidence Low, per-occurrence.
    ///
    /// Traces to: BC-2.15.023 postcondition 1; Canonical Test Vector; STORY-109 AC-011.
    #[test]
    fn test_enable_unsolicited_emits_t0814_possible_low() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x14 (ENABLE_UNSOLICITED) from src=0x0001 to dest=0x0003
        let frame = build_detection_frame(0x14, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 500);

        let enable_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("ENABLE_UNSOLICITED")
            })
            .collect();
        assert_eq!(
            enable_findings.len(),
            1,
            "AC-011: exactly ONE T0814 finding for FC=0x14 (ENABLE_UNSOLICITED); got {}",
            enable_findings.len()
        );

        let f = enable_findings[0];
        assert!(
            matches!(f.verdict, Verdict::Possible),
            "AC-011: ENABLE_UNSOLICITED verdict must be Possible; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Low),
            "AC-011: ENABLE_UNSOLICITED confidence must be Low; got {:?}",
            f.confidence
        );
        assert_eq!(
            f.mitre_techniques,
            vec!["T0814"],
            "AC-011: ENABLE_UNSOLICITED must carry only [\"T0814\"]"
        );

        // Per-occurrence: second ENABLE_UNSOLICITED emits another finding
        let frame2 = build_detection_frame(0x14, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame2, 501);

        let count_after = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("ENABLE_UNSOLICITED")
            })
            .count();
        assert_eq!(
            count_after, 2,
            "AC-011: per-occurrence: 2nd ENABLE_UNSOLICITED must emit 2nd T0814 finding"
        );
    }

    // -------------------------------------------------------------------------
    // AC-012 (BC-2.15.024 postconditions 2/3 — malformed_in_window threshold T0814)
    // test_malformed_anomaly_at_threshold_3_of_300s
    // -------------------------------------------------------------------------

    /// AC-012: 3 malformed frames within 300s window → T0814 Possible/Low at 3rd;
    /// parse_errors=3 (lifetime); malformed_in_window=3 (windowed).
    ///
    /// Summary must contain "possible Crain-Sistrunk crash-probe" per BC-2.15.024 PC3.
    ///
    /// Malformed frames are triggered by invalid LENGTH bytes (< 5).
    ///
    /// Traces to: BC-2.15.024 postconditions 2/3; STORY-109 AC-012.
    #[test]
    fn test_malformed_anomaly_at_threshold_3_of_300s() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver 3 malformed (LENGTH=2 < 5) frames within 300s.
        // Each invalid LENGTH triggers parse_errors += 1 AND (new) malformed_in_window += 1.
        for _ in 0..3u32 {
            let malformed = build_invalid_frame_length_too_short();
            analyzer.on_data(key.clone(), &malformed, 0);
        }

        // Exactly one T0814 malformed-anomaly finding must be emitted
        let malformed_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("possible Crain-Sistrunk crash-probe")
            })
            .collect();
        assert_eq!(
            malformed_findings.len(),
            1,
            "AC-012: exactly ONE T0814 malformed-anomaly finding at malformed_in_window=3; \
             got {}",
            malformed_findings.len()
        );

        let f = malformed_findings[0];
        assert!(
            matches!(f.verdict, Verdict::Possible),
            "AC-012: malformed anomaly verdict must be Possible; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Low),
            "AC-012: malformed anomaly confidence must be Low; got {:?}",
            f.confidence
        );
        assert_eq!(
            f.mitre_techniques,
            vec!["T0814"],
            "AC-012: malformed anomaly must carry only [\"T0814\"]"
        );

        // Summary must contain the Crain-Sistrunk substring (exact BC-2.15.024 PC3 pin)
        assert!(
            f.summary.contains("possible Crain-Sistrunk crash-probe"),
            "AC-012: summary must contain \"possible Crain-Sistrunk crash-probe\"; \
             got: {:?}",
            f.summary
        );

        // parse_errors (lifetime) must be 3
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.parse_errors, 3,
            "AC-012: parse_errors must be 3 (3 malformed frames, lifetime counter)"
        );
        // malformed_in_window (windowed) must be 3
        assert_eq!(
            flow.malformed_in_window, 3,
            "AC-012: malformed_in_window must be 3 (windowed counter)"
        );
        // One-shot guard set
        assert!(
            flow.malformed_anomaly_emitted,
            "AC-012: malformed_anomaly_emitted must be true after emission"
        );
    }

    // -------------------------------------------------------------------------
    // AC-013 (BC-2.15.024 invariant 1 — parse_errors never reset)
    // test_parse_errors_not_reset_at_window_expiry
    // -------------------------------------------------------------------------

    /// AC-013: parse_errors is lifetime/monotonic. After 300s window expiry,
    /// parse_errors remains at its accumulated value; malformed_in_window IS reset to 0.
    ///
    /// Traces to: BC-2.15.024 invariant 1; BC-2.15.015 postcondition 3; STORY-109 AC-013.
    #[test]
    fn test_parse_errors_not_reset_at_window_expiry() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Deliver 3 malformed frames at ts=0 to accumulate parse_errors=3, malformed_in_window=3
        for _ in 0..3u32 {
            let malformed = build_invalid_frame_length_too_short();
            analyzer.on_data(key.clone(), &malformed, 0);
        }

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert_eq!(flow.parse_errors, 3, "AC-013 pre: parse_errors=3");
            assert_eq!(
                flow.malformed_in_window, 3,
                "AC-013 pre: malformed_in_window=3"
            );
        }

        // Advance past 300s window expiry (ts=301 from correlation_window_start_ts=0)
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 301);

        let flow = analyzer.flows.get(&key).expect("flow must exist");

        // parse_errors MUST NOT be reset (lifetime counter)
        assert_eq!(
            flow.parse_errors, 3,
            "AC-013: parse_errors must STILL be 3 after 300s window expiry \
             (lifetime monotonic counter — NEVER reset)"
        );

        // malformed_in_window MUST be reset to 0 at window expiry
        assert_eq!(
            flow.malformed_in_window, 0,
            "AC-013: malformed_in_window must be 0 after 300s window expiry (windowed counter)"
        );

        // malformed_anomaly_emitted MUST be reset to false at window expiry
        assert!(
            !flow.malformed_anomaly_emitted,
            "AC-013: malformed_anomaly_emitted must be false after window expiry"
        );
    }

    // -------------------------------------------------------------------------
    // AC-014 (BC-2.15.014 invariant 8 / BC-2.15.016 invariant 8)
    // test_pending_request_timeout_wrapping_sub
    // -------------------------------------------------------------------------

    /// AC-014: Block-timeout check uses wrapping_sub — no panic on backward timestamps.
    ///
    /// Scenario: insert a pending request at ts=u32::MAX - 5, then deliver a frame
    /// at ts=5. wrapping_sub(5, u32::MAX - 5) = 11 > BLOCK_CMD_TIMEOUT_SECS=10.
    /// This should fire the timeout without panicking (overflow-checks=true in release).
    ///
    /// Traces to: BC-2.15.014 invariant 8; BC-2.15.016 invariant 8; STORY-109 AC-014.
    #[test]
    fn test_pending_request_timeout_wrapping_sub() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Insert a pending request at ts = u32::MAX - 5 (near wrap boundary)
        let near_max_ts = u32::MAX - 5;
        let frame = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 0);
        analyzer.on_data(key.clone(), &frame, near_max_ts);

        // Advance to ts=5: wrapping_sub(5, u32::MAX - 5) = 5 + 6 = 11 > 10
        // This must trigger the block timeout without panicking.
        // (Plain subtraction 5 - (u32::MAX - 5) would overflow and panic under overflow-checks=true)
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 5);

        // Must not have panicked; block_event_count should be 1 (timeout fired)
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after wrapping_sub scenario");
        assert_eq!(
            flow.block_event_count, 1,
            "AC-014: wrapping_sub timeout must fire and increment block_event_count \
             (backward ts: request at u32::MAX-5, trigger at ts=5; no panic)"
        );
    }

    // =========================================================================
    // EDGE CASES EC-001..EC-010
    // =========================================================================

    // -------------------------------------------------------------------------
    // EC-001: DIRECT_OPERATE_NR (FC=0x06) — not tracked; block_event_count NOT
    //         incremented (mirrors AC-001 negative case, standalone test).
    // -------------------------------------------------------------------------

    /// EC-001: FC=0x06 (DIRECT_OPERATE_NR) expects no response by design.
    /// It MUST NOT be added to pending_requests.
    /// Its "missing response" MUST NOT increment block_event_count.
    ///
    /// Traces to: BC-2.15.014 invariant 1; BC-2.15.014 EC-001; STORY-109 EC-001.
    #[test]
    fn test_EC_001_direct_operate_nr_not_tracked_no_block_event() {
        let mut analyzer = Dnp3Analyzer::new(100); // high threshold so no burst fires
        let key = test_flow_key();

        // Deliver FC=0x06 (DIRECT_OPERATE_NR)
        let frame = build_detection_frame_with_seq(0x06, 0x0003, 0x0001, 0);
        analyzer.on_data(key.clone(), &frame, 0);

        // Advance past timeout
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 11);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.block_event_count, 0,
            "EC-001: FC=0x06 (DIRECT_OPERATE_NR) must NOT increment block_event_count"
        );

        // pending_requests must be empty (0x06 never inserted)
        assert!(
            flow.pending_requests.is_empty(),
            "EC-001: pending_requests must be empty after FC=0x06 (not tracked)"
        );

        // No T1691.001 from DIRECT_OPERATE_NR
        let t1691_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1691.001".to_string()))
            .count();
        assert_eq!(
            t1691_count, 0,
            "EC-001: no T1691.001 from FC=0x06 (DIRECT_OPERATE_NR)"
        );
    }

    // -------------------------------------------------------------------------
    // EC-002: SELECT → RESPONSE within 10s removes pending entry; no block event.
    // -------------------------------------------------------------------------

    /// EC-002: SELECT (FC=0x03) followed by matching RESPONSE (FC=0x81) within 10s
    /// → entry removed from pending_requests; block_event_count NOT incremented.
    ///
    /// Traces to: BC-2.15.014 EC-002; STORY-109 EC-002.
    #[test]
    fn test_EC_002_select_response_within_timeout_no_block_event() {
        let mut analyzer = Dnp3Analyzer::new(100); // high threshold
        let key = test_flow_key();

        // SELECT (FC=0x03) from master (src=0x0001) to outstation (dest=0x0003)
        let select_frame = build_detection_frame_with_seq(0x03, 0x0003, 0x0001, 5);
        analyzer.on_data(key.clone(), &select_frame, 0);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            // Entry should be in pending_requests (key = (0x0003, 5))
            assert!(
                flow.pending_requests.contains_key(&(0x0003, 5)),
                "EC-002 pre: pending_requests must contain the SELECT entry (0x0003, seq=5)"
            );
        }

        // RESPONSE (FC=0x81) within 5s — matching (dest=0x0001, app_seq=5) from outstation
        // Response: outstation (src=0x0003) replies to master (dest=0x0001), same app_seq
        let response_frame = build_detection_frame_with_seq(0x81, 0x0001, 0x0003, 5);
        analyzer.on_data(key.clone(), &response_frame, 5); // 5 < BLOCK_CMD_TIMEOUT_SECS=10

        // After receiving the response, the pending entry must be removed
        // (the block-timeout scan at next on_data should not find it)
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 15); // advance past 10s

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.block_event_count, 0,
            "EC-002: SELECT → RESPONSE within timeout → no block event; block_event_count=0"
        );
        assert!(
            !flow.pending_requests.contains_key(&(0x0003, 5)),
            "EC-002: pending_requests entry must be removed after matching RESPONSE"
        );
    }

    // -------------------------------------------------------------------------
    // EC-004: T0827 one-shot: 4th restart same window → no second T0827.
    // -------------------------------------------------------------------------

    /// EC-004: After T0827 is emitted, a 4th restart in the same 300s window
    /// does NOT trigger a second T0827 (one-shot guard: loss_of_control_emitted=true).
    ///
    /// Traces to: BC-2.15.015 Trace C; BC-2.15.015 EC-004; STORY-109 EC-004.
    #[test]
    fn test_EC_004_t0827_one_shot_fourth_restart_no_second() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 3 COLD_RESTARTs → combined=3 → T0827 emitted after 3rd
        for i in 0u32..3 {
            let r = build_detection_frame(0x0D, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &r, i * 10);
        }

        {
            let t0827_count = analyzer
                .all_findings
                .iter()
                .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
                .count();
            assert_eq!(
                t0827_count, 1,
                "EC-004 pre: exactly ONE T0827 after 3 restarts"
            );
        }

        // 4th COLD_RESTART in same window (ts=30, well within 300s from ts=0)
        let r4 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &r4, 30);

        let t0827_count_after = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .count();
        assert_eq!(
            t0827_count_after, 1,
            "EC-004: one-shot guard: 4th restart must NOT emit a second T0827 \
             (loss_of_control_emitted=true blocks second emission)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 4,
            "EC-004: restart_event_count must be 4 (4th restart counted even when guard active)"
        );
    }

    // -------------------------------------------------------------------------
    // EC-005: Broadcast READ (FC=0x01) to 0xFFFF → no broadcast anomaly.
    // -------------------------------------------------------------------------

    /// EC-005: READ (FC=0x01) to broadcast dest=0xFFFF — not anomalous.
    /// Only Control-class FCs trigger BC-2.15.018.
    ///
    /// Traces to: BC-2.15.018 invariant 2; BC-2.15.018 EC-004; STORY-109 EC-005.
    #[test]
    fn test_EC_005_broadcast_read_no_anomaly() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        let frame = build_detection_frame(0x01, 0xFFFF, 0x0001); // READ to broadcast
        analyzer.on_data(key.clone(), &frame, 1000);

        let t1692_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1692.001".to_string()))
            .count();
        assert_eq!(
            t1692_count, 0,
            "EC-005: READ (FC=0x01) to broadcast dest=0xFFFF must NOT emit T1692.001"
        );
    }

    // -------------------------------------------------------------------------
    // EC-007: ENABLE_UNSOLICITED before UNSOLICITED_RESPONSE suppresses anomaly.
    // -------------------------------------------------------------------------

    /// EC-007: ENABLE_UNSOLICITED (FC=0x14) observed first → enable_unsolicited_seen=true
    /// → subsequent FC=0x82 (UNSOLICITED_RESPONSE) does NOT emit unsolicited anomaly.
    ///
    /// This is a standalone verification of the state-machine ordering.
    /// (Also covered by AC-009b; this test focuses on the flag state.)
    ///
    /// Traces to: BC-2.15.019 invariant 2; BC-2.15.019 EC-001; STORY-109 EC-007.
    #[test]
    fn test_EC_007_enable_before_unsolicited_suppresses_anomaly() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // ENABLE_UNSOLICITED first
        let enable = build_detection_frame(0x14, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &enable, 100);

        let flow_after_enable = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow_after_enable.enable_unsolicited_seen,
            "EC-007: enable_unsolicited_seen must be true after FC=0x14"
        );

        // Then UNSOLICITED_RESPONSE — anomaly must be suppressed
        let unsol = build_detection_frame(0x82, 0x0001, 0x0003);
        analyzer.on_data(key.clone(), &unsol, 200);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            !flow.unsolicited_anomaly_emitted,
            "EC-007: unsolicited_anomaly_emitted must remain false when ENABLE was prior"
        );

        // No unsolicited-anomaly finding
        let unsolicited_anomaly_count = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("unexpected unsolicited response")
            })
            .count();
        assert_eq!(
            unsolicited_anomaly_count, 0,
            "EC-007: no unsolicited-anomaly T0814 when ENABLE_UNSOLICITED was prior"
        );
    }

    // -------------------------------------------------------------------------
    // EC-009: 4th malformed frame (same window) — one-shot guard; no 2nd T0814.
    // -------------------------------------------------------------------------

    /// EC-009: After malformed_anomaly_emitted=true (at 3rd frame), a 4th malformed
    /// frame increments both parse_errors and malformed_in_window but does NOT emit
    /// a second T0814 malformed-anomaly finding.
    ///
    /// Traces to: BC-2.15.024 EC-004; STORY-109 EC-009.
    #[test]
    fn test_EC_009_fourth_malformed_no_second_t0814() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 3 malformed frames → T0814 emitted at 3rd
        for _ in 0..3u32 {
            let malformed = build_invalid_frame_length_too_short();
            analyzer.on_data(key.clone(), &malformed, 0);
        }

        let t0814_malformed_count_before = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("possible Crain-Sistrunk crash-probe")
            })
            .count();
        assert_eq!(
            t0814_malformed_count_before, 1,
            "EC-009 pre: exactly ONE malformed T0814 after 3 frames"
        );

        // 4th malformed frame — guard must prevent second T0814
        let malformed4 = build_invalid_frame_length_too_short();
        analyzer.on_data(key.clone(), &malformed4, 0);

        let t0814_malformed_count_after = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("possible Crain-Sistrunk crash-probe")
            })
            .count();
        assert_eq!(
            t0814_malformed_count_after, 1,
            "EC-009: one-shot guard: 4th malformed frame must NOT emit second T0814"
        );

        // parse_errors must be 4 (lifetime — never capped by guard)
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.parse_errors, 4,
            "EC-009: parse_errors must be 4 after 4 malformed frames (lifetime, no guard)"
        );
        assert_eq!(
            flow.malformed_in_window, 4,
            "EC-009: malformed_in_window must be 4 after 4 frames"
        );
    }

    // -------------------------------------------------------------------------
    // EC-010: Single COLD_RESTART → T0814 emitted, T0827 NOT emitted.
    //         (T0827 requires >= 3 combined events.)
    // -------------------------------------------------------------------------

    /// EC-010: A single COLD_RESTART emits T0814 but must NOT emit T0827.
    /// T0827 requires restart_event_count + block_event_count >= T0827_THRESHOLD = 3.
    ///
    /// Traces to: BC-2.15.015 invariant 1; BC-2.15.015 EC-007; STORY-109 EC-010.
    #[test]
    fn test_EC_010_single_cold_restart_t0814_no_t0827() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Single COLD_RESTART
        let frame = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000);

        // T0814 must be emitted (COLD_RESTART is a per-occurrence detection)
        let t0814_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .count();
        assert!(
            t0814_count >= 1,
            "EC-010: COLD_RESTART must emit at least one T0814 finding"
        );

        // T0827 must NOT be emitted (only 1 event, threshold=3)
        let t0827_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .count();
        assert_eq!(
            t0827_count, 0,
            "EC-010: single COLD_RESTART must NOT emit T0827 \
             (invariant: T0827 requires restart_event_count + block_event_count >= 3)"
        );

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.restart_event_count, 1,
            "EC-010: restart_event_count must be 1 after single COLD_RESTART"
        );
        assert!(
            !flow.loss_of_control_emitted,
            "EC-010: loss_of_control_emitted must be false (T0827 not emitted)"
        );
    }

    // =========================================================================
    // Constant-value sanity tests
    // (These ensure the story-mandated constants are present and correct;
    //  they compile without production logic so always PASS — included to
    //  guard against stub regressions.)
    // =========================================================================

    /// Verify that the story-mandated constants have the correct values.
    /// These are pure-compile assertions — they do not require any production logic.
    ///
    /// Traces to: STORY-109 Architecture Mapping (constant table).
    #[test]
    fn test_story_109_constants_correct_values() {
        assert_eq!(
            CORRELATION_WINDOW_SECS, 300u32,
            "CORRELATION_WINDOW_SECS must be 300"
        );
        assert_eq!(
            BLOCK_CMD_TIMEOUT_SECS, 10u32,
            "BLOCK_CMD_TIMEOUT_SECS must be 10"
        );
        assert_eq!(BLOCK_CMD_THRESHOLD, 3u64, "BLOCK_CMD_THRESHOLD must be 3");
        assert_eq!(T0827_THRESHOLD, 3u64, "T0827_THRESHOLD must be 3");
        assert_eq!(
            MALFORMED_ANOMALY_THRESHOLD, 3u64,
            "MALFORMED_ANOMALY_THRESHOLD must be 3"
        );
    }

    /// Verify MitreTactic::IcsImpact variant exists and T0827 resolves to it.
    ///
    /// Traces to: STORY-109 VP-007 atomic obligation; BC-2.15.015 Invariant 2.
    #[test]
    fn test_story_109_ics_impact_tactic_seeded() {
        use wirerust::mitre::{MitreTactic, technique_info};

        // T0827 must be in the catalog with IcsImpact tactic
        let info = technique_info("T0827");
        assert!(info.is_some(), "T0827 must be seeded in technique_info");
        let (name, tactic) = info.unwrap();
        assert_eq!(name, "Loss of Control");
        assert_eq!(
            tactic,
            MitreTactic::IcsImpact,
            "T0827 must use IcsImpact tactic (NEW variant, distinct from enterprise Impact)"
        );

        // T1691.001 must be in the catalog with IcsInhibitResponseFunction tactic
        let info2 = technique_info("T1691.001");
        assert!(
            info2.is_some(),
            "T1691.001 must be seeded in technique_info"
        );
        let (name2, tactic2) = info2.unwrap();
        assert_eq!(
            name2,
            "Block Operational Technology Message: Command Message"
        );
        assert_eq!(
            tactic2,
            MitreTactic::IcsInhibitResponseFunction,
            "T1691.001 must use IcsInhibitResponseFunction tactic"
        );
    }

    /// Verify is_broadcast_destination helper: dest >= 0xFFFD → true, others false.
    ///
    /// Traces to: BC-2.15.018 invariant 1; STORY-109 AC-007.
    #[test]
    fn test_story_109_is_broadcast_destination_helper() {
        use wirerust::analyzer::dnp3::is_broadcast_destination;

        // Broadcast range: 0xFFFD, 0xFFFE, 0xFFFF
        assert!(is_broadcast_destination(0xFFFF), "0xFFFF must be broadcast");
        assert!(is_broadcast_destination(0xFFFE), "0xFFFE must be broadcast");
        assert!(is_broadcast_destination(0xFFFD), "0xFFFD must be broadcast");

        // Non-broadcast
        assert!(
            !is_broadcast_destination(0xFFFC),
            "0xFFFC must NOT be broadcast"
        );
        assert!(
            !is_broadcast_destination(0x0003),
            "0x0003 must NOT be broadcast"
        );
        assert!(
            !is_broadcast_destination(0x0000),
            "0x0000 must NOT be broadcast"
        );
    }
}
