//! Failing tests for STORY-109: DNP3 Correlated/Derived + Anomaly Detections.
//!
//! Covers AC-001..AC-014 and edge cases EC-001..EC-010 from the STORY-109 spec.
//! Traces to behavioral contracts: BC-2.15.014, BC-2.15.015, BC-2.15.018,
//! BC-2.15.019, BC-2.15.023, BC-2.15.024.
//!
//! These tests were authored RED-first (TDD) against todo!() stubs; production logic
//! has since landed and all 34 tests in `mod story_109` pass.
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
    use wirerust::reassembly::handler::Direction;

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
        analyzer.on_data(key.clone(), &frame, 0, Direction::ClientToServer);

        // No response — advance past BLOCK_CMD_TIMEOUT_SECS (10s) with a new frame at ts=11
        // The block-timeout scan fires during this on_data call.
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001); // READ — just to advance ts
        analyzer.on_data(key.clone(), &trigger, 11, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert_eq!(
            flow.block_event_count, 1,
            "AC-001: DIRECT_OPERATE (0x05) timeout must set block_event_count=1 \
             (unconditional, regardless of T1691.001 threshold)"
        );

        // Now send FC=0x06 (DIRECT_OPERATE_NR) — should NOT be tracked in pending_requests,
        // so no block event when timeout fires.
        let frame_nr = build_detection_frame_with_seq(0x06, 0x0003, 0x0001, 2);
        analyzer.on_data(key.clone(), &frame_nr, 12, Direction::ClientToServer);

        // Advance past 10s from ts=12 (so ts=23+): block_event_count must stay at 1
        let trigger2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger2, 23, Direction::ClientToServer);

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
    /// F-109-P1-003: BC-2.15.014 PC3 pins the exact summary format:
    ///   "DNP3 inferred blocked command: {count} requests without response within {window}s
    ///    (dest={dest:#06X})"
    /// where {window} = BLOCK_CMD_TIMEOUT_SECS (10), {dest} = 0x0003, {count} = 3.
    /// Expected exact summary: "DNP3 inferred blocked command: 3 requests without response
    /// within 10s (dest=0x0003)"
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
            analyzer.on_data(key.clone(), &frame, base_ts, Direction::ClientToServer);
            // Advance 11 seconds: triggers block-timeout scan for all pending_requests
            // with saturating_sub(base_ts + 11, base_ts) = 11 > BLOCK_CMD_TIMEOUT_SECS (10)
            let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
            analyzer.on_data(
                key.clone(),
                &trigger,
                base_ts + 11,
                Direction::ClientToServer,
            );
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

        // F-109-P1-003: BC-2.15.014 PC3 exact summary pin.
        // Format: "DNP3 inferred blocked command: {count} requests without response
        //          within {window}s (dest={dest:#06X})"
        // With the test's concrete values: count=3, window=BLOCK_CMD_TIMEOUT_SECS=10,
        // dest=0x0003 → dest formatted as {:#06X} = "0x0003".
        // {window} per BC-2.15.014 is the per-request timeout (BLOCK_CMD_TIMEOUT_SECS),
        // NOT the correlation window, per BC description: "10 seconds. Rationale: DNP3
        // SBO select-to-operate timeout is typically 3–10 seconds".
        let expected_t1691_summary =
            "DNP3 inferred blocked command: 3 requests without response within 10s (dest=0x0003)";
        assert_eq!(
            f.summary, expected_t1691_summary,
            "AC-002 (F-109-P1-003): T1691.001 summary must match exact BC-2.15.014 PC3 format; \
             BC-expected: {:?}; impl-actual: {:?}",
            expected_t1691_summary, f.summary
        );

        // BC-2.15.014 PC3 — category must be ThreatCategory::Execution.
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "AC-002: T1691.001 category must be ThreatCategory::Execution \
             (BC-2.15.014 PC3); got {:?}",
            f.category
        );

        // BC-2.15.014 PC3 v1.6 — evidence is a SUMMARY string (not per-request).
        // Format: "block_event_count={count} in correlation window; threshold={threshold}"
        // For this scenario: count=3, threshold=BLOCK_CMD_THRESHOLD=3.
        // This assertion pins the BC-2.15.014 v1.6 evidence format (including the
        // " in correlation window; " segment required by v1.6).
        let expected_evidence = vec!["block_event_count=3 in correlation window; threshold=3"];
        assert_eq!(
            f.evidence, expected_evidence,
            "AC-002 (BC-2.15.014 PC3 v1.6): T1691.001 evidence must be the SUMMARY format \
             \"block_event_count={{count}} in correlation window; threshold={{threshold}}\";\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_evidence, f.evidence
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
        analyzer.on_data(key.clone(), &frame1, 0, Direction::ClientToServer);
        let trigger1 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger1, 11, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &frame2, 150, Direction::ClientToServer);
        let trigger2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger2, 161, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &frame1, 0, Direction::ClientToServer);
        let t1 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &t1, 11, Direction::ClientToServer);

        // Block event #2: ts=150 → timeout at ts=161 (still within 300s window)
        let frame2 = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 1);
        analyzer.on_data(key.clone(), &frame2, 150, Direction::ClientToServer);
        let t2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &t2, 161, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &restart, 200, Direction::ClientToServer);

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
    // F-P9-001 regression tests (BC-2.15.015 PC5 / EC-002)
    // Expose the gap: T0827 must fire when the BLOCK-TIMEOUT path crosses the
    // combined threshold, even when block_event_count < BLOCK_CMD_THRESHOLD (3).
    //
    // The impl calls maybe_emit_t0827 on the block-timeout path regardless of
    // block_event_count, so when restarts supply 2 of the 3 events and 1 block
    // timeout supplies the remainder, T0827 is correctly emitted.  Both tests
    // below verify this combined-threshold path (F-P9-001 fix).
    // -------------------------------------------------------------------------

    /// F-P9-001 regression test 1 (BC-2.15.015 PC5 / EC-002):
    /// Trace C-rev — 2 restarts first, then 1 block timeout crosses combined threshold.
    ///
    /// Arrival order:
    ///   ts=0:   COLD_RESTART #1 → restart_event_count=1, T0814 emitted
    ///   ts=10:  COLD_RESTART #2 → restart_event_count=2, T0814 emitted; combined=2 < 3
    ///   ts=20:  SELECT (FC=0x03, app_seq=7, dest=0x0003) delivered — pending_requests entry
    ///   ts=31:  READ frame (no RESPONSE for seq=7) — block-timeout scan fires;
    ///           saturating_sub(31, 20) = 11 > BLOCK_CMD_TIMEOUT_SECS=10;
    ///           block_event_count becomes 1; combined = 2+1 = 3 >= T0827_THRESHOLD=3.
    ///           T1691.001 is NOT emitted (block_event_count=1 < BLOCK_CMD_THRESHOLD=3).
    ///           T0827 MUST be emitted (combined >= 3, !loss_of_control_emitted).
    ///
    /// Expected T0827 fields (BC-2.15.015 PC1):
    ///   mitre_techniques: vec!["T0827"]
    ///   category: ThreatCategory::Impact
    ///   verdict: Verdict::Likely
    ///   confidence: Confidence::Medium
    ///
    /// Authored RED (F-P9-001): the fix unconditionally calls maybe_emit_t0827 on the
    /// block-timeout path so T0827 fires when combined >= threshold regardless of
    /// block_event_count.  This test now passes.
    ///
    /// Traces to: BC-2.15.015 PC5; EC-002; F-P9-001; STORY-109 AC-004 (Trace C-rev).
    #[test]
    fn test_t0827_emitted_when_block_crosses_threshold_after_restarts() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // --- Step 1: 2 COLD_RESTARTs → restart_event_count=2, combined=2 ---
        // ts=0: COLD_RESTART #1 → T0814; restart_event_count=1
        let r1 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &r1, 0, Direction::ClientToServer);

        // ts=10: COLD_RESTART #2 → T0814; restart_event_count=2, combined=2 < 3
        let r2 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &r2, 10, Direction::ClientToServer);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after 2 restarts");
            assert_eq!(
                flow.restart_event_count, 2,
                "F-P9-001/test1 pre: restart_event_count must be 2"
            );
            assert_eq!(
                flow.block_event_count, 0,
                "F-P9-001/test1 pre: block_event_count must be 0"
            );
        }
        // No T0827 yet (combined=2 < threshold=3)
        assert_eq!(
            analyzer
                .all_findings
                .iter()
                .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
                .count(),
            0,
            "F-P9-001/test1 pre: no T0827 at combined=2 (< threshold=3)"
        );

        // --- Step 2: SELECT (FC=0x03) at ts=20 — enters pending_requests ---
        // Uses app_seq=7 so it does not collide with any prior request.
        // dest=0x0003, src=0x0001.
        let select = build_detection_frame_with_seq(0x03, 0x0003, 0x0001, 7);
        analyzer.on_data(key.clone(), &select, 20, Direction::ClientToServer);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after SELECT");
            assert!(
                flow.pending_requests.contains_key(&(0x0003, 7)),
                "F-P9-001/test1 pre: pending_requests must contain (dest=0x0003, seq=7) \
                 after SELECT"
            );
        }

        // --- Step 3: advance to ts=31 with no RESPONSE for seq=7 ---
        // saturating_sub(31, 20) = 11 > BLOCK_CMD_TIMEOUT_SECS=10 → block timeout fires.
        // block_event_count → 1; combined = 2+1 = 3.
        // T1691.001 must NOT fire (block_event_count=1 < BLOCK_CMD_THRESHOLD=3).
        // T0827 MUST fire (combined=3 >= T0827_THRESHOLD=3).
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 31, Direction::ClientToServer);

        // Verify block_event_count reached 1
        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after block timeout");
            assert_eq!(
                flow.block_event_count, 1,
                "F-P9-001/test1: block_event_count must be 1 after SELECT timeout"
            );
        }

        // T1691.001 must NOT have fired (block_event_count=1 < BLOCK_CMD_THRESHOLD=3)
        let t1691_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1691.001".to_string()))
            .count();
        assert_eq!(
            t1691_count, 0,
            "F-P9-001/test1: T1691.001 must NOT fire when block_event_count=1 \
             (< BLOCK_CMD_THRESHOLD=3); got {t1691_count}"
        );

        // T0827 MUST have fired exactly once (combined=3 >= T0827_THRESHOLD=3).
        // Verifies BC-2.15.015 PC5: the block-timeout path calls maybe_emit_t0827
        // regardless of block_event_count (F-P9-001 fix).
        let t0827_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .collect();

        // Diagnostic: print what IS present so the failure message is informative.
        let present_techniques: Vec<Vec<String>> = analyzer
            .all_findings
            .iter()
            .map(|f| f.mitre_techniques.clone())
            .collect();

        assert_eq!(
            t0827_findings.len(),
            1,
            "F-P9-001/test1 (BC-2.15.015 PC5 / EC-002 REGRESSION): exactly ONE T0827 must be \
             emitted when block-timeout path crosses combined threshold \
             (2 restarts + 1 block = 3); impl currently skips maybe_emit_t0827 from \
             block-timeout when block_event_count < BLOCK_CMD_THRESHOLD=3. \
             Techniques actually present: {present_techniques:?}"
        );

        let f = t0827_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0827"],
            "F-P9-001/test1: T0827 finding must carry only [\"T0827\"]"
        );
        assert!(
            matches!(f.category, ThreatCategory::Impact),
            "F-P9-001/test1: T0827 category must be Impact; got {:?}",
            f.category
        );
        assert!(
            matches!(f.verdict, Verdict::Likely),
            "F-P9-001/test1: T0827 verdict must be Likely; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Medium),
            "F-P9-001/test1: T0827 confidence must be Medium; got {:?}",
            f.confidence
        );

        // BC-2.15.013 ordering: T0827 must appear after at least one T0814
        // (the COLD_RESTART T0814s were pushed before the block scan ran).
        let last_t0814_pos = analyzer
            .all_findings
            .iter()
            .rposition(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .expect("F-P9-001/test1: at least one T0814 must exist (from COLD_RESTARTs)");
        let t0827_pos = analyzer
            .all_findings
            .iter()
            .position(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .expect("F-P9-001/test1: T0827 must be present");
        assert!(
            t0827_pos > last_t0814_pos,
            "F-P9-001/test1 (BC-2.15.013 PC2): T0827 [{t0827_pos}] must appear AFTER \
             last T0814 [{last_t0814_pos}] in all_findings"
        );

        // One-shot guard
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.loss_of_control_emitted,
            "F-P9-001/test1: loss_of_control_emitted must be true after T0827 emission"
        );
    }

    /// F-P9-001 regression test 2 (BC-2.15.015 PC5 / EC-002):
    /// Trace D — 1 restart, then 2 block timeouts; T0827 fires on the 2nd block timeout.
    ///
    /// Arrival order:
    ///   ts=0:   COLD_RESTART → restart_event_count=1, T0814 emitted; combined=1
    ///   ts=10:  SELECT #1 (FC=0x03, app_seq=1, dest=0x0003) → pending_requests entry
    ///   ts=21:  READ frame (no RESPONSE for seq=1) → block timeout fires;
    ///           block_event_count=1; combined=2 < 3.  No T0827 yet.
    ///   ts=30:  SELECT #2 (FC=0x03, app_seq=2, dest=0x0003) → pending_requests entry
    ///   ts=41:  READ frame (no RESPONSE for seq=2) → block timeout fires;
    ///           block_event_count=2; combined = 1+2 = 3 >= T0827_THRESHOLD=3.
    ///           T1691.001 still NOT emitted (block_event_count=2 < BLOCK_CMD_THRESHOLD=3).
    ///           T0827 MUST be emitted on THIS scan.
    ///
    /// Expected T0827 fields: same as test 1 (Impact / Likely / Medium).
    ///
    /// Authored RED for the same F-P9-001 reason: the fix removes the
    /// block_event_count >= 3 guard from the maybe_emit_t0827 call site on the
    /// block-timeout path.  This test now passes.
    ///
    /// Traces to: BC-2.15.015 PC5; EC-002; F-P9-001; STORY-109 AC-004 (Trace D).
    #[test]
    fn test_t0827_emitted_when_second_block_crosses_threshold_after_one_restart() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // --- Step 1: 1 COLD_RESTART → restart_event_count=1, combined=1 ---
        let r1 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &r1, 0, Direction::ClientToServer);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after restart");
            assert_eq!(
                flow.restart_event_count, 1,
                "F-P9-001/test2 pre: restart_event_count must be 1"
            );
            assert_eq!(
                flow.block_event_count, 0,
                "F-P9-001/test2 pre: block_event_count must be 0"
            );
        }

        // --- Step 2: SELECT #1 at ts=10 → pending_requests entry (seq=1) ---
        let select1 = build_detection_frame_with_seq(0x03, 0x0003, 0x0001, 1);
        analyzer.on_data(key.clone(), &select1, 10, Direction::ClientToServer);

        // --- Step 3: advance to ts=21 with no RESPONSE for seq=1 ---
        // saturating_sub(21, 10) = 11 > 10 → block timeout fires; block_event_count=1.
        // combined = 1+1 = 2 < 3 → no T0827 yet.
        let trigger1 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger1, 21, Direction::ClientToServer);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after first block timeout");
            assert_eq!(
                flow.block_event_count, 1,
                "F-P9-001/test2: block_event_count must be 1 after first SELECT timeout"
            );
        }
        assert_eq!(
            analyzer
                .all_findings
                .iter()
                .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
                .count(),
            0,
            "F-P9-001/test2: no T0827 at combined=2 (< threshold=3) after 1 restart + 1 block"
        );

        // --- Step 4: SELECT #2 at ts=30 → pending_requests entry (seq=2) ---
        let select2 = build_detection_frame_with_seq(0x03, 0x0003, 0x0001, 2);
        analyzer.on_data(key.clone(), &select2, 30, Direction::ClientToServer);

        // --- Step 5: advance to ts=41 with no RESPONSE for seq=2 ---
        // saturating_sub(41, 30) = 11 > 10 → block timeout fires; block_event_count=2.
        // combined = 1+2 = 3 >= T0827_THRESHOLD=3.
        // T1691.001 must NOT fire (block_event_count=2 < BLOCK_CMD_THRESHOLD=3).
        // T0827 MUST fire (combined=3 >= 3, !loss_of_control_emitted).
        let trigger2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger2, 41, Direction::ClientToServer);

        // Verify block_event_count reached 2
        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after second block timeout");
            assert_eq!(
                flow.block_event_count, 2,
                "F-P9-001/test2: block_event_count must be 2 after second SELECT timeout"
            );
        }

        // T1691.001 must NOT have fired (block_event_count=2 < BLOCK_CMD_THRESHOLD=3)
        let t1691_count = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T1691.001".to_string()))
            .count();
        assert_eq!(
            t1691_count, 0,
            "F-P9-001/test2: T1691.001 must NOT fire when block_event_count=2 \
             (< BLOCK_CMD_THRESHOLD=3); got {t1691_count}"
        );

        // T0827 MUST have fired exactly once on the scan where the 2nd block crossed
        // the combined threshold.  Verifies BC-2.15.015 PC5 (F-P9-001 fix).
        let t0827_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .collect();

        // Diagnostic: print what IS present to aid failure triage.
        let present_techniques: Vec<Vec<String>> = analyzer
            .all_findings
            .iter()
            .map(|f| f.mitre_techniques.clone())
            .collect();

        assert_eq!(
            t0827_findings.len(),
            1,
            "F-P9-001/test2 (BC-2.15.015 PC5 / EC-002 REGRESSION): exactly ONE T0827 must be \
             emitted when 2nd block timeout crosses combined threshold \
             (1 restart + 2 blocks = 3); impl currently skips maybe_emit_t0827 from \
             block-timeout when block_event_count < BLOCK_CMD_THRESHOLD=3. \
             Techniques actually present: {present_techniques:?}"
        );

        let f = t0827_findings[0];
        assert_eq!(
            f.mitre_techniques,
            vec!["T0827"],
            "F-P9-001/test2: T0827 finding must carry only [\"T0827\"]"
        );
        assert!(
            matches!(f.category, ThreatCategory::Impact),
            "F-P9-001/test2: T0827 category must be Impact; got {:?}",
            f.category
        );
        assert!(
            matches!(f.verdict, Verdict::Likely),
            "F-P9-001/test2: T0827 verdict must be Likely; got {:?}",
            f.verdict
        );
        assert!(
            matches!(f.confidence, Confidence::Medium),
            "F-P9-001/test2: T0827 confidence must be Medium; got {:?}",
            f.confidence
        );

        // BC-2.15.013 ordering: T0827 must appear after the T0814 from the restart.
        let t0814_pos = analyzer
            .all_findings
            .iter()
            .position(|f| f.mitre_techniques.contains(&"T0814".to_string()))
            .expect("F-P9-001/test2: T0814 must exist (from COLD_RESTART at ts=0)");
        let t0827_pos = analyzer
            .all_findings
            .iter()
            .position(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .expect("F-P9-001/test2: T0827 must be present");
        assert!(
            t0827_pos > t0814_pos,
            "F-P9-001/test2 (BC-2.15.013 PC2): T0827 [{t0827_pos}] must appear AFTER \
             T0814 [{t0814_pos}] in all_findings"
        );

        // One-shot guard
        let flow = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow.loss_of_control_emitted,
            "F-P9-001/test2: loss_of_control_emitted must be true after T0827 emission"
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
            analyzer.on_data(key.clone(), &malformed, 0, Direction::ClientToServer);
        }

        // Seed restart_event_count (and loss_of_control_emitted via T0827)
        // by doing 3 COLD_RESTARTs at ts=0..2.
        for i in 0u32..3 {
            let restart = build_detection_frame(0x0D, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &restart, i, Direction::ClientToServer);
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
        // (saturating_sub(301, 0) = 301 > CORRELATION_WINDOW_SECS=300 → window expires).
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 301, Direction::ClientToServer);

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
    // BC-2.15.015 first-window-seeding edge (mutation survivor #6 kill)
    // test_BC_2_15_015_first_window_seed_sets_anchor_not_expiry
    // -------------------------------------------------------------------------

    /// First-window-seeding: on a brand-new flow (`correlation_window_seeded == false`),
    /// the very first `maybe_expire_correlation_window` call (via `on_data`) at a
    /// high timestamp (ts=1000) MUST:
    ///   1. Set `correlation_window_start_ts = 1000` (anchored to the first frame).
    ///   2. Set `correlation_window_seeded = true`.
    ///   3. NOT reset the six windowed counters (no spurious expiry — it is a seed,
    ///      not an expiry).
    ///
    /// If the `!` in `if !flow.correlation_window_seeded` is deleted (mutation survivor #6),
    /// the seed branch is SKIPPED for unseeded flows.  The expiry branch then evaluates
    /// `wrapping_sub(1000, 0) = 1000 >= 300` → spurious expiry fires, AND
    /// `correlation_window_seeded` remains `false` forever, so the window is re-seeded on
    /// EVERY subsequent call instead of only once.  Both observable effects kill the mutant:
    ///   - Under the mutant, `correlation_window_seeded` stays `false` after the first call.
    ///   - Under the mutant, a second frame at ts=1000 (same window) would again "expire"
    ///     the window, resetting counters that were just accumulated — the distinct-source
    ///     assertion below catches this via `restart_event_count`.
    ///
    /// Traces to: BC-2.15.015 seeding invariant (STORY-109); mutation survivor #6.
    #[test]
    fn test_BC_2_15_015_first_window_seed_sets_anchor_not_expiry() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Fresh flow: before any on_data call, no flow state exists yet.
        // Deliver the FIRST frame at ts=1000.  This is far above
        // CORRELATION_WINDOW_SECS (300), so without the seed guard it would
        // trigger a spurious window expiry (wrapping_sub(1000, 0) >= 300).
        let first_frame = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &first_frame, 1000, Direction::ClientToServer);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after first frame");

            // Assertion 1 (kills mutant): the window anchor must be the first frame's ts,
            // not 0 (which would mean the seed branch was skipped).
            assert_eq!(
                flow.correlation_window_start_ts, 1000,
                "BC-2.15.015 seed: correlation_window_start_ts must be anchored to the \
                 first frame's ts=1000, not 0. \
                 Mutant (deleted `!`): seed branch is skipped, anchor stays 0."
            );

            // Assertion 2 (kills mutant): the seeded flag must be true after the first call.
            assert!(
                flow.correlation_window_seeded,
                "BC-2.15.015 seed: correlation_window_seeded must be true after the first \
                 frame. \
                 Mutant (deleted `!`): seeded flag is never set on an unseeded flow."
            );

            // Assertion 3: no spurious expiry — windowed counters must not have been touched.
            // (They start at 0 and a pure seed must not reset them — a reset is harmless for
            // zero values, but combined with assertions 1 & 2 this pins the code path.)
            assert_eq!(
                flow.restart_event_count, 0,
                "BC-2.15.015 seed: restart_event_count must be 0 — seed must not touch \
                 windowed counters"
            );
        }

        // Assertion 4: deliver a COLD_RESTART (FC=0x0D) at the SAME ts=1000 (still inside
        // the seeded window).  restart_event_count must become 1.
        // Under the mutant, `correlation_window_seeded` is still false after the first call,
        // so `maybe_expire_correlation_window` treats this second call as ALSO unseeded:
        // it seeds again (anchor=1000) and returns — preventing the expiry check from running.
        // That means restart_event_count can still increment here (same outcome as correct
        // code for this step).  The decisive observable difference was already in assertions
        // 1 & 2 above.  This step confirms the counter accumulates in the seeded window.
        let restart_frame = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &restart_frame, 1000, Direction::ClientToServer);

        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after restart frame");
            assert_eq!(
                flow.restart_event_count, 1,
                "BC-2.15.015 seed: restart_event_count must be 1 after one COLD_RESTART \
                 in the seeded window"
            );

            // Assertion 5 (secondary mutant kill): deliver a third frame still at ts=1000.
            // Under the correct code: seeded=true, so the expiry check runs but
            // saturating_sub(1000, 1000) = 0 < 300 → no expiry → counters intact.
            // Under the mutant: seeded is still false → seed branch fires again → return
            // immediately WITHOUT running the expiry check.  Either way restart_event_count
            // must still be 1 here (no expiry occurred), but `correlation_window_seeded`
            // being false (mutant) was already caught in assertion 2.
            assert!(
                flow.correlation_window_seeded,
                "BC-2.15.015 seed: correlation_window_seeded must remain true on subsequent \
                 same-window frames"
            );
        }
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
        analyzer.on_data(key.clone(), &r1, 0, Direction::ClientToServer);
        let r2 = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &r2, 10, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

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
        // F-109-P1-001 closure: BC-2.15.018 PC1 pins category: ThreatCategory::Suspicious.
        assert!(
            matches!(f.category, ThreatCategory::Suspicious),
            "AC-007 (F-109-P1-001 category closure): broadcast anomaly category must be \
             ThreatCategory::Suspicious (BC-2.15.018 PC1); got {:?}",
            f.category
        );
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
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

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
            analyzer.on_data(key.clone(), &frame, i, Direction::ClientToServer);
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
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

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
        // F-109-P1-001 closure: BC-2.15.019 PC1 pins category: ThreatCategory::Suspicious.
        assert!(
            matches!(f.category, ThreatCategory::Suspicious),
            "AC-009 (F-109-P1-001 category closure): unsolicited anomaly category must be \
             ThreatCategory::Suspicious (BC-2.15.019 PC1); got {:?}",
            f.category
        );
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
        analyzer.on_data(key.clone(), &frame2, 1001, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &enable_frame, 100, Direction::ClientToServer);

        {
            let flow = analyzer.flows.get(&key).expect("flow must exist");
            assert!(
                flow.enable_unsolicited_seen,
                "AC-009b: enable_unsolicited_seen must be true after FC=0x14"
            );
        }

        // Then: UNSOLICITED_RESPONSE (FC=0x82) — must NOT emit anomaly
        let unsol_frame = build_detection_frame(0x82, 0x0001, 0x0003);
        analyzer.on_data(key.clone(), &unsol_frame, 200, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

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
        // F-109-P1-001 closure: BC-2.15.023 PC1 pins category: ThreatCategory::Execution.
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "AC-010 (F-109-P1-001 category closure): DISABLE_UNSOLICITED category must be \
             ThreatCategory::Execution (BC-2.15.023 PC1); got {:?}",
            f.category
        );
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

        // F-109-P6-001 (resolved GREEN): BC-2.15.023 PC1 pins evidence format:
        //   "FC=0x{fc:02X} dest={dest:#06X} src={src:#06X}"
        // — dest BEFORE src, NO parenthetical FC name.
        // frame: dest=0x0003, src=0x0001, fc=0x15 → "FC=0x15 dest=0x0003 src=0x0001".
        // The impl now emits exactly this format; the assertion passes.
        assert_eq!(
            f.evidence,
            vec!["FC=0x15 dest=0x0003 src=0x0001"],
            "AC-010 (F-109-P6-001): DISABLE_UNSOLICITED evidence must match exact \
             BC-2.15.023 PC1 format \"FC=0x{{fc:02X}} dest={{dest:#06X}} src={{src:#06X}}\"; \
             got: {:?}",
            f.evidence
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
        analyzer.on_data(key.clone(), &frame2, 1001, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &frame, 500, Direction::ClientToServer);

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
        // F-109-P1-001 closure: BC-2.15.023 PC1 pins category: ThreatCategory::Execution.
        assert!(
            matches!(f.category, ThreatCategory::Execution),
            "AC-011 (F-109-P1-001 category closure): ENABLE_UNSOLICITED category must be \
             ThreatCategory::Execution (BC-2.15.023 PC1); got {:?}",
            f.category
        );
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

        // F-109-P6-001 (MAJOR): BC-2.15.023 PC1 pins evidence format:
        //   "FC=0x{fc:02X} dest={dest:#06X} src={src:#06X}"
        // — dest BEFORE src, NO parenthetical FC name.
        // frame: dest=0x0003, src=0x0001, fc=0x14 → "FC=0x14 dest=0x0003 src=0x0001".
        // Authored RED (F-109-P6-001): the impl was corrected to omit the FC-name
        // parenthetical and to emit dest before src.  This assertion now passes.
        assert_eq!(
            f.evidence,
            vec!["FC=0x14 dest=0x0003 src=0x0001"],
            "AC-011 (F-109-P6-001): ENABLE_UNSOLICITED evidence must match exact \
             BC-2.15.023 PC1 format \"FC=0x{{fc:02X}} dest={{dest:#06X}} src={{src:#06X}}\"; \
             got: {:?}",
            f.evidence
        );

        // Per-occurrence: second ENABLE_UNSOLICITED emits another finding
        let frame2 = build_detection_frame(0x14, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame2, 501, Direction::ClientToServer);

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
            analyzer.on_data(key.clone(), &malformed, 0, Direction::ClientToServer);
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

        // F-109-P1-001 closure: BC-2.15.024 PC3 pins category: ThreatCategory::Anomaly.
        // This assertion verifies the impl emits Anomaly (not Suspicious) per BC-2.15.024 PC3.
        assert!(
            matches!(f.category, ThreatCategory::Anomaly),
            "AC-012 (F-109-P1-001 MAJOR): malformed anomaly category must be \
             ThreatCategory::Anomaly (BC-2.15.024 PC3); got {:?}",
            f.category
        );

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

        // F-109-P1-004: BC-2.15.024 PC3 summary format must include the flow fragment
        // "(flow {src_ip}→{dest_ip})". The test_flow_key() uses:
        //   ip_a = 10.0.0.1:20000, ip_b = 10.0.0.2:20000
        // FlowKey canonicalizes: lower_ip=10.0.0.1 (port 20000), upper_ip=10.0.0.2 (port 20000).
        // The flow fragment in the summary uses lower_ip as src and upper_ip as dest
        // (source-endpoint convention: the flow's originating/lower-IP endpoint → remote).
        // Expected full BC-2.15.024 PC3 summary (count=3, elapsed=0s since all frames at ts=0):
        //   "DNP3 structural anomaly: 3 malformed frames in 0s window
        //    (flow 10.0.0.1→10.0.0.2) — possible Crain-Sistrunk crash-probe"
        // This assertion pins the BC-2.15.024 PC3 flow fragment requirement:
        // the summary must contain "(flow {src_ip}→{dest_ip})" per the v1.6 format.
        assert!(
            f.summary.contains("(flow 10.0.0.1→10.0.0.2)"),
            "AC-012 (F-109-P1-004): summary must contain flow fragment \
             \"(flow 10.0.0.1→10.0.0.2)\" per BC-2.15.024 PC3; \
             got: {:?}",
            f.summary
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
            analyzer.on_data(key.clone(), &malformed, 0, Direction::ClientToServer);
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
        analyzer.on_data(key.clone(), &trigger, 301, Direction::ClientToServer);

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
    // AC-014 superseded — BC-2.15.014 v2.1 EC-009 / RULING-DNP3-SIBLING-001 §2.2
    // test_pending_request_timeout_no_spurious_fire_on_rollover_or_backwards_ts
    //
    // Regression-guard: the old wrapping_sub(5, u32::MAX-5) = 11 > 10 semantics
    // (STORY-109 AC-014) have been superseded by saturating_sub per BC-2.15.014 v2.1
    // EC-009 and RULING-DNP3-SIBLING-001 §2.2.  Under saturating_sub, the rollover
    // scenario (request_ts=u32::MAX-5, now_ts=5) yields saturating_sub(5, u32::MAX-5) = 0,
    // which is NOT > BLOCK_CMD_TIMEOUT_SECS=10, so no spurious block timeout fires.
    // Forward-clock STILL fires: saturating_sub(11, 0) = 11 > 10.
    // -------------------------------------------------------------------------

    /// Regression-guard: rollover/backwards-ts does NOT spuriously fire a block timeout.
    ///
    /// Past semantics (STORY-109 AC-014): wrapping_sub(5, u32::MAX-5) = 11 → fired.
    /// Current semantics (BC-2.15.014 v2.1 EC-009; RULING-DNP3-SIBLING-001 §2.2):
    ///   saturating_sub(5, u32::MAX-5) = 0 → NOT > 10 → no spurious fire.
    ///
    /// Also guards forward-clock path: saturating_sub(11, 0) = 11 > 10 → fires normally.
    ///
    /// Supersedes: STORY-109 AC-014 (wrapping_sub semantics).
    /// Traces to: BC-2.15.014 v2.1 EC-009; RULING-DNP3-SIBLING-001 §2.2.
    #[test]
    fn test_pending_request_timeout_no_spurious_fire_on_rollover_or_backwards_ts() {
        // ---- Part A: rollover/backwards-ts must NOT fire a block timeout ----
        {
            let mut analyzer = Dnp3Analyzer::new(10);
            let key = test_flow_key();

            // Insert a pending request at ts = u32::MAX - 5 (near wrap boundary).
            let near_max_ts = u32::MAX - 5;
            let frame = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 0);
            analyzer.on_data(key.clone(), &frame, near_max_ts, Direction::ClientToServer);

            // Advance to ts=5: saturating_sub(5, u32::MAX-5) = 0, NOT > 10.
            // Under old wrapping_sub this was 11 > 10 → spuriously fired.
            // Under saturating_sub (BC-2.15.014 v2.1 EC-009) this is 0 → no fire.
            let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &trigger, 5, Direction::ClientToServer);

            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after rollover scenario");
            assert_eq!(
                flow.block_event_count, 0,
                "BC-2.15.014 v2.1 EC-009: rollover/backwards-ts (request at u32::MAX-5, \
                 trigger at ts=5) must NOT spuriously fire a block timeout; \
                 saturating_sub(5, u32::MAX-5)=0, not > 10; \
                 supersedes STORY-109 AC-014 wrapping_sub semantics"
            );
        }

        // ---- Part B: forward-clock STILL fires a block timeout ----
        {
            let mut analyzer = Dnp3Analyzer::new(10);
            let key = test_flow_key();

            // Insert a pending request at ts=0.
            let frame = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 0);
            analyzer.on_data(key.clone(), &frame, 0, Direction::ClientToServer);

            // Advance to ts=11: saturating_sub(11, 0) = 11 > 10 → block timeout fires.
            let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &trigger, 11, Direction::ClientToServer);

            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after forward-clock scenario");
            assert_eq!(
                flow.block_event_count, 1,
                "BC-2.15.014 v2.1: forward-clock (request at ts=0, trigger at ts=11) \
                 MUST fire a block timeout; saturating_sub(11, 0)=11 > 10"
            );
        }
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
        analyzer.on_data(key.clone(), &frame, 0, Direction::ClientToServer);

        // Advance past timeout
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 11, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &select_frame, 0, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &response_frame, 5, Direction::ClientToServer); // 5 < BLOCK_CMD_TIMEOUT_SECS=10

        // After receiving the response, the pending entry must be removed
        // (the block-timeout scan at next on_data should not find it)
        let trigger = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &trigger, 15, Direction::ClientToServer); // advance past 10s

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
            analyzer.on_data(key.clone(), &r, i * 10, Direction::ClientToServer);
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
        analyzer.on_data(key.clone(), &r4, 30, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &enable, 100, Direction::ClientToServer);

        let flow_after_enable = analyzer.flows.get(&key).expect("flow must exist");
        assert!(
            flow_after_enable.enable_unsolicited_seen,
            "EC-007: enable_unsolicited_seen must be true after FC=0x14"
        );

        // Then UNSOLICITED_RESPONSE — anomaly must be suppressed
        let unsol = build_detection_frame(0x82, 0x0001, 0x0003);
        analyzer.on_data(key.clone(), &unsol, 200, Direction::ClientToServer);

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
            analyzer.on_data(key.clone(), &malformed, 0, Direction::ClientToServer);
        }

        let t0814_malformed_findings_before: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("possible Crain-Sistrunk crash-probe")
            })
            .collect();
        let t0814_malformed_count_before = t0814_malformed_findings_before.len();
        assert_eq!(
            t0814_malformed_count_before, 1,
            "EC-009 pre: exactly ONE malformed T0814 after 3 frames"
        );

        // F-109-P1-001 closure: category must be ThreatCategory::Anomaly per BC-2.15.024 PC3.
        // This assertion verifies the impl emits Anomaly (not Suspicious) per BC-2.15.024 PC3.
        let f_pre = t0814_malformed_findings_before[0];
        assert!(
            matches!(f_pre.category, ThreatCategory::Anomaly),
            "EC-009 (F-109-P1-001 MAJOR): malformed anomaly category must be \
             ThreatCategory::Anomaly (BC-2.15.024 PC3); got {:?}",
            f_pre.category
        );

        // 4th malformed frame — guard must prevent second T0814
        let malformed4 = build_invalid_frame_length_too_short();
        analyzer.on_data(key.clone(), &malformed4, 0, Direction::ClientToServer);

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
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

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

    // =========================================================================
    // ADVERSARIAL PASS-2 EXACT-EQUALITY PINS (F-P2-001..005 + OBS-3/EC-006)
    // These pin every BC-mandated summary and evidence string to an exact
    // assert_eq! so that format drift from the BC is caught immediately.
    // =========================================================================

    // -------------------------------------------------------------------------
    // F-P2-001 — exact T0827 summary pin (BC-2.15.015 PC1)
    // -------------------------------------------------------------------------

    /// F-P2-001: Pin `f.summary` for the T0827 finding in
    /// `test_t0827_emitted_at_combined_threshold` (AC-004) to the EXACT BC-2.15.015
    /// PC1 format string:
    ///   "DNP3 sustained loss-of-control pattern: {restart_count} restart events +
    ///    {block_count} blocked commands within {elapsed}s on flow (dest={dest:#06X})"
    ///
    /// Concrete values (Trace B test):
    ///   restart_count=1, block_count=2, elapsed=200s
    ///   (correlation_window_start=0, T0827-triggering COLD_RESTART at ts=200)
    ///   dest=0x0003 (outstation address from the COLD_RESTART frame)
    ///   Formatted: "0x0003" ({:#06X} = "0x" + zero-pad to 4 hex digits, uppercase).
    ///
    /// Traces to: BC-2.15.015 PC1; STORY-109 AC-004 (Trace B); F-P2-001.
    #[test]
    fn test_BC_2_15_015_t0827_summary_exact_pin() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // Replicate AC-004 Trace B setup:
        // Block event #1: ts=0 → timeout at ts=11
        let frame1 = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 0);
        analyzer.on_data(key.clone(), &frame1, 0, Direction::ClientToServer);
        let t1 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &t1, 11, Direction::ClientToServer);

        // Block event #2: ts=150 → timeout at ts=161
        let frame2 = build_detection_frame_with_seq(0x05, 0x0003, 0x0001, 1);
        analyzer.on_data(key.clone(), &frame2, 150, Direction::ClientToServer);
        let t2 = build_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &t2, 161, Direction::ClientToServer);

        // COLD_RESTART at ts=200 → combined=3 → T0827
        let restart = build_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &restart, 200, Direction::ClientToServer);

        let t0827_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| f.mitre_techniques.contains(&"T0827".to_string()))
            .collect();
        assert_eq!(
            t0827_findings.len(),
            1,
            "F-P2-001 pre: exactly ONE T0827 finding must exist"
        );

        let f = t0827_findings[0];

        // BC-2.15.015 PC1 exact summary format:
        //   restart_count=1, block_count=2, elapsed=200s, dest=0x0003
        // saturating_sub(200, 0) = 200; {0x0003:#06X} = "0x0003"
        let expected_summary = "DNP3 sustained loss-of-control pattern: \
            1 restart events + 2 blocked commands within 200s on flow (dest=0x0003)";
        assert_eq!(
            f.summary, expected_summary,
            "F-P2-001 (BC-2.15.015 PC1): T0827 summary must match exact BC format;\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_summary, f.summary
        );
    }

    // -------------------------------------------------------------------------
    // F-P2-002 — exact broadcast summary + evidence pin (BC-2.15.018 PC1)
    // -------------------------------------------------------------------------

    /// F-P2-002: Pin both `f.summary` and `f.evidence` for the broadcast anomaly
    /// finding in `test_broadcast_control_anomaly_fires_for_dest_ffff` (AC-007)
    /// to the EXACT BC-2.15.018 PC1 format strings:
    ///   summary:  "DNP3 broadcast control command: Control FC 0x{fc:02X} sent to
    ///              broadcast destination {dest:#06X}"
    ///   evidence: "FC=0x{fc:02X} dest={dest:#06X} (broadcast) src={src:#06X}"
    ///
    /// Concrete values:
    ///   fc=0x05 (DIRECT_OPERATE), dest=0xFFFF, src=0x0001
    ///   fc formatted as 02X (no # prefix in summary/evidence): "05"
    ///   dest as {:#06X}: "0xFFFF"  (2-char prefix + 4-char hex = 6 total)
    ///   src  as {:#06X}: "0x0001"
    ///
    /// Traces to: BC-2.15.018 PC1; STORY-109 AC-007; F-P2-002.
    #[test]
    fn test_BC_2_15_018_broadcast_summary_and_evidence_exact_pin() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // DIRECT_OPERATE (FC=0x05) to broadcast dest=0xFFFF from src=0x0001
        let frame = build_detection_frame(0x05, 0xFFFF, 0x0001);
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

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
            "F-P2-002 pre: exactly ONE Suspicious T1692.001 broadcast finding"
        );

        let f = broadcast_findings[0];

        // BC-2.15.018 PC1 exact summary format:
        let expected_summary =
            "DNP3 broadcast control command: Control FC 0x05 sent to broadcast destination 0xFFFF";
        assert_eq!(
            f.summary, expected_summary,
            "F-P2-002 (BC-2.15.018 PC1 summary): broadcast summary must match exact BC format;\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_summary, f.summary
        );

        // BC-2.15.018 PC1 exact evidence format (single entry in Vec<String>):
        let expected_evidence = vec!["FC=0x05 dest=0xFFFF (broadcast) src=0x0001".to_string()];
        assert_eq!(
            f.evidence, expected_evidence,
            "F-P2-002 (BC-2.15.018 PC1 evidence): broadcast evidence must match exact BC format;\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_evidence, f.evidence
        );
    }

    // -------------------------------------------------------------------------
    // F-P2-003 — exact unsolicited summary + evidence pin (BC-2.15.019 PC1)
    // -------------------------------------------------------------------------

    /// F-P2-003: Pin both `f.summary` and `f.evidence` for the unsolicited-anomaly
    /// finding in `test_unsolicited_response_anomaly_no_prior_enable` (AC-009a)
    /// to the EXACT BC-2.15.019 PC1 format strings:
    ///   summary:  "DNP3 unexpected unsolicited response: UNSOLICITED_RESPONSE from
    ///              src={src:#06X} with no prior ENABLE_UNSOLICITED or solicited
    ///              exchange on this flow"
    ///   evidence: "FC=0x82 src={src:#06X} dest={dest:#06X} UNS_bit={uns_bit}"
    ///
    /// Concrete values from the test frame `build_detection_frame(0x82, 0x0001, 0x0003)`:
    ///   app_fc=0x82, dest=0x0001, src=0x0003  (outstation→master direction)
    ///   app_ctrl byte (frame[11]) = 0x00; UNS bit = (0x00 & 0x10 != 0) = false
    ///   src  as {:#06X}: "0x0003"
    ///   dest as {:#06X}: "0x0001"
    ///   uns_bit: false
    ///
    /// Traces to: BC-2.15.019 PC1; STORY-109 AC-009; F-P2-003.
    #[test]
    fn test_BC_2_15_019_unsolicited_summary_and_evidence_exact_pin() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x82 (UNSOLICITED_RESPONSE) — outstation (src=0x0003) to master (dest=0x0001)
        // app_ctrl byte = 0x00 (app_seq=0, no UNS bit set)
        let frame = build_detection_frame(0x82, 0x0001, 0x0003);
        analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);

        let t0814_findings: Vec<_> = analyzer
            .all_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0814".to_string())
                    && f.summary.contains("unexpected unsolicited response")
            })
            .collect();
        assert_eq!(
            t0814_findings.len(),
            1,
            "F-P2-003 pre: exactly ONE unsolicited-anomaly T0814 finding"
        );

        let f = t0814_findings[0];

        // BC-2.15.019 PC1 exact summary format:
        let expected_summary = "DNP3 unexpected unsolicited response: \
            UNSOLICITED_RESPONSE from src=0x0003 with no prior ENABLE_UNSOLICITED \
            or solicited exchange on this flow";
        assert_eq!(
            f.summary, expected_summary,
            "F-P2-003 (BC-2.15.019 PC1 summary): unsolicited summary must match exact BC format;\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_summary, f.summary
        );

        // BC-2.15.019 PC1 exact evidence format (single entry in Vec<String>):
        //   app_ctrl=0x00 → UNS_bit = (0x00 & 0x10 != 0) = false
        let expected_evidence = vec!["FC=0x82 src=0x0003 dest=0x0001 UNS_bit=false".to_string()];
        assert_eq!(
            f.evidence, expected_evidence,
            "F-P2-003 (BC-2.15.019 PC1 evidence): unsolicited evidence must match exact BC format;\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_evidence, f.evidence
        );
    }

    // -------------------------------------------------------------------------
    // F-P2-004 — exact ENABLE_UNSOLICITED summary pin (BC-2.15.023 PC1)
    // -------------------------------------------------------------------------

    /// F-P2-004: Pin `f.summary` for the ENABLE_UNSOLICITED (FC=0x14) finding in
    /// `test_enable_unsolicited_emits_t0814_possible_low` (AC-011) to the EXACT
    /// BC-2.15.023 PC1 ENABLE format string:
    ///   "DNP3 ENABLE_UNSOLICITED observed: FC 0x14 from src={src:#06X} to
    ///    dest={dest:#06X} — unsolicited reporting control"
    ///
    /// Concrete values (frame built with dest=0x0003, src=0x0001):
    ///   src=0x0001, dest=0x0003
    ///   Expected: "DNP3 ENABLE_UNSOLICITED observed: FC 0x14 from src=0x0001
    ///              to dest=0x0003 — unsolicited reporting control"
    ///
    /// The DISABLE sibling (AC-010) is already pinned in
    /// `test_disable_unsolicited_emits_t0814_likely_medium` — this test confirms
    /// the ENABLE sibling is equally pinned. Both must pass or fail together.
    ///
    /// Traces to: BC-2.15.023 PC1 (ENABLE variant); STORY-109 AC-011; F-P2-004.
    #[test]
    fn test_BC_2_15_023_enable_unsolicited_summary_exact_pin() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // FC=0x14 (ENABLE_UNSOLICITED) from src=0x0001 to dest=0x0003
        let frame = build_detection_frame(0x14, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &frame, 500, Direction::ClientToServer);

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
            "F-P2-004 pre: exactly ONE ENABLE_UNSOLICITED T0814 finding"
        );

        let f = enable_findings[0];

        // BC-2.15.023 PC1 (ENABLE variant) exact summary:
        let expected_summary = "DNP3 ENABLE_UNSOLICITED observed: FC 0x14 from src=0x0001 \
            to dest=0x0003 — unsolicited reporting control";
        assert_eq!(
            f.summary, expected_summary,
            "F-P2-004 (BC-2.15.023 PC1 ENABLE): ENABLE_UNSOLICITED summary must match exact \
             BC format;\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_summary, f.summary
        );
    }

    // -------------------------------------------------------------------------
    // F-P2-005 — exact malformed evidence pin (BC-2.15.024 PC3)
    // -------------------------------------------------------------------------

    /// F-P2-005: Add an exact `f.evidence` assertion to the malformed-anomaly
    /// finding in `test_malformed_anomaly_at_threshold_3_of_300s` (AC-012),
    /// pinned to BC-2.15.024 PC3 evidence format:
    ///   "malformed_in_window={count} in correlation window; threshold={threshold}"
    ///
    /// Concrete values:
    ///   count=3 (3rd malformed frame crosses threshold)
    ///   threshold=MALFORMED_ANOMALY_THRESHOLD=3
    ///   Expected: "malformed_in_window=3 in correlation window; threshold=3"
    ///
    /// Traces to: BC-2.15.024 PC3; STORY-109 AC-012; F-P2-005.
    #[test]
    fn test_BC_2_15_024_malformed_evidence_exact_pin() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // 3 malformed (LENGTH=2) frames within 300s → T0814 emitted at 3rd
        for _ in 0..3u32 {
            let malformed = build_invalid_frame_length_too_short();
            analyzer.on_data(key.clone(), &malformed, 0, Direction::ClientToServer);
        }

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
            "F-P2-005 pre: exactly ONE T0814 malformed-anomaly finding"
        );

        let f = malformed_findings[0];

        // BC-2.15.024 PC3 exact evidence format (single entry in Vec<String>):
        let expected_evidence =
            vec!["malformed_in_window=3 in correlation window; threshold=3".to_string()];
        assert_eq!(
            f.evidence, expected_evidence,
            "F-P2-005 (BC-2.15.024 PC3 evidence): malformed evidence must match exact BC format;\n\
             BC-expected: {:?}\n\
             impl-actual:  {:?}",
            expected_evidence, f.evidence
        );
    }

    // -------------------------------------------------------------------------
    // OBS-3 / EC-006 — bailed flow is permanent no-op (BC-2.15.009 PC5)
    // test_EC_006_bailed_flow_disable_unsolicited_no_op
    // -------------------------------------------------------------------------

    /// OBS-3 / EC-006: On a flow already bailed (`is_non_dnp3=true` from a prior
    /// sync-loss), delivering a DISABLE_UNSOLICITED (FC=0x15) frame is an immediate
    /// NO-OP per BC-2.15.009 PC5:
    ///   - `parse_errors` does NOT increment (no new parse attempted)
    ///   - `malformed_in_window` does NOT increment
    ///   - No Finding is pushed (no detection on bailed flow)
    ///   - carry is NOT touched
    ///
    /// Setup: deliver a frame whose first bytes are NOT the DNP3 sync word [0x05, 0x64]
    /// so the flow latches `is_non_dnp3=true`. Then deliver a well-formed
    /// DISABLE_UNSOLICITED (FC=0x15) frame and verify ALL counts are unchanged.
    ///
    /// This codifies that the implementation's immediate-bail after non-DNP3 sync is
    /// correct and that EC-006 ("subsequent on_data on bailed flow → immediate no-op")
    /// matches BC-2.15.009 PC5.
    ///
    /// Traces to: BC-2.15.009 PC5; BC-2.15.009 EC-006; OBS-3; STORY-109.
    #[test]
    fn test_EC_006_bailed_flow_disable_unsolicited_no_op() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = test_flow_key();

        // --- Step 1: trigger the desync bail ---
        // Deliver bytes that do NOT start with [0x05, 0x64] so the analyzer
        // latches is_non_dnp3=true on this flow.
        // Use 16+ bytes of non-DNP3 data (e.g. 0xFF-filled buffer) so the
        // 16-byte bail window is satisfied.
        let non_dnp3_bytes: Vec<u8> = vec![0xFF; 20];
        analyzer.on_data(key.clone(), &non_dnp3_bytes, 0, Direction::ClientToServer);

        // Verify bail was latched
        {
            let flow = analyzer
                .flows
                .get(&key)
                .expect("flow must exist after bail");
            assert!(
                flow.is_non_dnp3,
                "OBS-3/EC-006 setup: is_non_dnp3 must be true after non-DNP3 data"
            );
        }

        // Snapshot counters before the bailed-flow call
        let (parse_errors_before, malformed_in_window_before, findings_before) = {
            let flow = analyzer.flows.get(&key).unwrap();
            (
                flow.parse_errors,
                flow.malformed_in_window,
                analyzer.all_findings.len(),
            )
        };

        // --- Step 2: deliver a well-formed DISABLE_UNSOLICITED (FC=0x15) to the bailed flow ---
        // This MUST be an immediate no-op per BC-2.15.009 PC5.
        let disable_frame = build_detection_frame(0x15, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &disable_frame, 100, Direction::ClientToServer);

        let flow = analyzer.flows.get(&key).expect("flow must still exist");

        // BC-2.15.009 PC5: parse_errors must NOT increment on bailed flow
        assert_eq!(
            flow.parse_errors, parse_errors_before,
            "OBS-3/EC-006 (BC-2.15.009 PC5): parse_errors must NOT increment on bailed flow; \
             before={parse_errors_before} after={}",
            flow.parse_errors
        );

        // BC-2.15.009 PC5: malformed_in_window must NOT increment on bailed flow
        assert_eq!(
            flow.malformed_in_window, malformed_in_window_before,
            "OBS-3/EC-006 (BC-2.15.009 PC5): malformed_in_window must NOT increment on \
             bailed flow; before={malformed_in_window_before} after={}",
            flow.malformed_in_window
        );

        // BC-2.15.009 PC5: no Finding must be pushed on bailed flow
        assert_eq!(
            analyzer.all_findings.len(),
            findings_before,
            "OBS-3/EC-006 (BC-2.15.009 PC5): no Finding must be pushed on bailed flow; \
             findings before={findings_before} after={}",
            analyzer.all_findings.len()
        );

        // is_non_dnp3 must remain latched (one-way latch per BC-2.15.009 invariant 2)
        assert!(
            flow.is_non_dnp3,
            "OBS-3/EC-006 (BC-2.15.009 invariant 2): is_non_dnp3 must remain true \
             (one-way latch — never reset to false)"
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
