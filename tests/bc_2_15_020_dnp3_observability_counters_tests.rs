//! Failing tests (Red Gate) for three new observability counters on `Dnp3Analyzer`
//! (silent-limit audit, FIX-B maintenance run maint-2026-07-06).
//!
//! Behavioral contracts covered:
//!   BC-2.15.020 v1.5 — `dropped_findings`, `master_addrs_dropped`,
//!                       `pending_requests_evicted` present in `summarize()` detail map (8 keys)
//!   BC-2.15.016 v2.1 — `master_addrs_dropped` incremented on MAX_MASTER_ADDRS=64 cap;
//!                       `pending_requests_evicted` incremented on MAX_PENDING_REQUESTS=256 LRU eviction
//!   BC-2.15.022 v1.5 — `dropped_findings` incremented on MAX_FINDINGS=10_000 cap suppression
//!
//! GREEN: all 9 tests pass.  The three counter fields (`dropped_findings`,
//! `master_addrs_dropped`, `pending_requests_evicted`) are present on `Dnp3Analyzer`, and
//! `summarize()` emits the corresponding detail keys (maint-2026-07-06 FIX-B, commit 636c0d6).
//!
//! Assertion strategy: all assertions target `summarize().detail` map keys so tests
//! COMPILE even before the struct fields exist.  A missing key causes a runtime panic via
//! `.unwrap_or_else()` — not a compile error — which is the correct red-gate failure mode.
//!
//! Site references from df-validation-pc019-pc020-2026-07-06.md:
//!   - master_addrs cap/silent-ignore: dnp3.rs:146, 750-755
//!   - pending_requests LRU eviction helper: dnp3.rs:1799-1815
//!   - MAX_FINDINGS cap guards (11 sites): dnp3.rs:201, 987,1040,1093,1171,1292,1353,1416,1500,1569,1603,1666
//!
//! DF-TEST-NAMESPACE-001: all tests are wrapped in `mod bc_2_15_020_dnp3_observability_counters`.

#![allow(non_snake_case)]

mod bc_2_15_020_dnp3_observability_counters {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::dnp3::{
        Dnp3Analyzer, MAX_FINDINGS, MAX_MASTER_ADDRS, MAX_PENDING_REQUESTS,
    };
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    // -----------------------------------------------------------------------
    // Shared helpers
    // -----------------------------------------------------------------------

    fn dnp3_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            20000,
        )
    }

    /// Build a minimal valid DNP3 link frame (15 bytes) carrying one application FC.
    ///
    /// Layout (mirrors the helper in `dnp3_detection_tests.rs::story_108`):
    ///   [0..1]   0x05 0x64              (sync word)
    ///   [2]      0x08                   (LENGTH=8 → frame_len = 5+8+2×1 = 15)
    ///   [3]      0xC4                   (CTRL: DIR=1(0x80), PRM=1(0x40), FC-nibble=UNCONFIRMED_USER_DATA; has_user_data=true)
    ///   [4..5]   dest little-endian
    ///   [6..7]   src  little-endian
    ///   [8..9]   0x00 0x00              (header CRC placeholder)
    ///   [10]     0xC0                   (transport: FIR=1(0x40)|FIN=1(0x80))
    ///   [11]     0x00                   (app control; app_seq = 0x00 & 0x0F = 0)
    ///   [12]     app_fc
    ///   [13..14] 0x00 0x00              (data-block CRC placeholder)
    ///
    /// is_master_frame(0xC4) = (0xC4 & 0x80 != 0) = true — all frames are master-direction.
    fn build_dnp3_detection_frame(app_fc: u8, dest: u16, src: u16) -> Vec<u8> {
        let length_byte: u8 = 8;
        let u_bytes = (length_byte as usize) - 5; // 3
        let blocks = u_bytes.div_ceil(16); // 1
        let frame_len = 5 + (length_byte as usize) + 2 * blocks; // 15

        let mut frame = vec![0u8; frame_len];
        frame[0] = 0x05;
        frame[1] = 0x64;
        frame[2] = length_byte;
        frame[3] = 0xC4;
        let [dl, dh] = dest.to_le_bytes();
        frame[4] = dl;
        frame[5] = dh;
        let [sl, sh] = src.to_le_bytes();
        frame[6] = sl;
        frame[7] = sh;
        frame[10] = 0xC0;
        frame[11] = 0x00;
        frame[12] = app_fc;
        frame
    }

    /// Minimal dummy `Finding` for pre-filling `all_findings` without triggering logic.
    fn dummy_finding() -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Unlikely,
            confidence: Confidence::Low,
            summary: "pre-fill dummy".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    // -----------------------------------------------------------------------
    // KEY-PRESENCE tests (BC-2.15.020 v1.5 Postcondition 1, Invariant 1)
    //
    // Each of the three new counter keys must appear in `summarize().detail` with
    // value 0 on a freshly constructed (zero-input) `Dnp3Analyzer`.
    //
    // RED GATE: none of the three keys is currently emitted by `summarize()`.
    // Missing key → `.unwrap_or_else()` panics → test fails at runtime, not compile time.
    // -----------------------------------------------------------------------

    /// BC-2.15.020 v1.5 PC-1 / BC-2.15.022 v1.5 Inv-5:
    /// `dropped_findings` must always be present in `Dnp3Analyzer::summarize()` detail
    /// with value 0 when the MAX_FINDINGS cap has never been hit.
    ///
    /// RED GATE: key absent from current `summarize()` → panic on `.unwrap_or_else()`.
    #[test]
    fn test_BC_2_15_020_dropped_findings_key_present_zero_on_fresh_analyzer() {
        let analyzer = Dnp3Analyzer::new(10);
        let summary = analyzer.summarize();
        let val = summary.detail.get("dropped_findings").unwrap_or_else(|| {
            panic!(
                "BC-2.15.020 v1.5 / BC-2.15.022 v1.5: Dnp3Analyzer summarize() must contain \
                     key 'dropped_findings' (u64, always-present, 0 when cap never hit). \
                     Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                summary.detail.keys().collect::<Vec<_>>()
            )
        });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.15.022 v1.5: 'dropped_findings' must be 0 on a fresh analyzer, got: {val}"
        );
    }

    /// BC-2.15.020 v1.5 PC-1 / BC-2.15.016 v2.1 PC-6:
    /// `master_addrs_dropped` must always be present in `Dnp3Analyzer::summarize()` detail
    /// with value 0 when MAX_MASTER_ADDRS=64 has never been reached.
    ///
    /// RED GATE: key absent from current `summarize()` → panic on `.unwrap_or_else()`.
    #[test]
    fn test_BC_2_15_020_master_addrs_dropped_key_present_zero_on_fresh_analyzer() {
        let analyzer = Dnp3Analyzer::new(10);
        let summary = analyzer.summarize();
        let val = summary
            .detail
            .get("master_addrs_dropped")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.15.020 v1.5 / BC-2.15.016 v2.1 PC-6: Dnp3Analyzer summarize() must \
                     contain key 'master_addrs_dropped' (u64, always-present, 0 when cap never \
                     reached). Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    summary.detail.keys().collect::<Vec<_>>()
                )
            });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.15.016 v2.1: 'master_addrs_dropped' must be 0 on a fresh analyzer, got: {val}"
        );
    }

    /// BC-2.15.020 v1.5 PC-1 / BC-2.15.016 v2.1 PC-10:
    /// `pending_requests_evicted` must always be present in `Dnp3Analyzer::summarize()` detail
    /// with value 0 when MAX_PENDING_REQUESTS=256 has never been reached.
    ///
    /// RED GATE: key absent from current `summarize()` → panic on `.unwrap_or_else()`.
    #[test]
    fn test_BC_2_15_020_pending_requests_evicted_key_present_zero_on_fresh_analyzer() {
        let analyzer = Dnp3Analyzer::new(10);
        let summary = analyzer.summarize();
        let val = summary
            .detail
            .get("pending_requests_evicted")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.15.020 v1.5 / BC-2.15.016 v2.1 PC-10: Dnp3Analyzer summarize() must \
                     contain key 'pending_requests_evicted' (u64, always-present, 0 when cap \
                     never reached). Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    summary.detail.keys().collect::<Vec<_>>()
                )
            });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.15.016 v2.1: 'pending_requests_evicted' must be 0 on a fresh analyzer, got: {val}"
        );
    }

    // -----------------------------------------------------------------------
    // INCREMENT ON EVENT tests
    // -----------------------------------------------------------------------

    /// BC-2.15.022 v1.5 PC-5: `dropped_findings` increments by 1 for each finding
    /// suppressed because `all_findings.len() == MAX_FINDINGS = 10_000`.
    ///
    /// Mechanism: pre-fill `all_findings` to exactly MAX_FINDINGS via the public field
    /// (same pattern as `test_max_findings_cap_preserves_first_finding` in
    /// `dnp3_detection_tests.rs`). Then deliver a COLD_RESTART (FC=0x0D) frame — the
    /// detection logic would push a T0814 finding, but the cap is hit, so the push is
    /// suppressed and `dropped_findings` must be incremented by 1.
    ///
    /// Mirrors BC-2.15.022 EC-001 canonical test vector; mirrors BC-2.14.022 (Modbus)
    /// and BC-2.17.022 (ENIP) precedents.
    ///
    /// RED GATE: `dropped_findings` not yet a field on `Dnp3Analyzer`; `summarize()` does
    /// not emit it → `.unwrap_or_else()` panics.
    #[test]
    fn test_BC_2_15_022_dropped_findings_increments_when_all_findings_cap_hit() {
        let mut analyzer = Dnp3Analyzer::new(10);
        let key = dnp3_flow_key();

        // Pre-fill all_findings to MAX_FINDINGS using the public field.
        for _ in 0..MAX_FINDINGS {
            analyzer.all_findings.push(dummy_finding());
        }
        assert_eq!(
            analyzer.all_findings.len(),
            MAX_FINDINGS,
            "pre-condition: all_findings must be at MAX_FINDINGS={} before test",
            MAX_FINDINGS
        );

        // Seed the flow with a non-detection READ frame (no finding expected from READ).
        let read_frame = build_dnp3_detection_frame(0x01, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &read_frame, 0, Direction::ClientToServer);

        // COLD_RESTART (FC=0x0D) — detection pushes T0814, but cap is at MAX_FINDINGS.
        // Expectation: push suppressed, `dropped_findings` incremented by 1.
        let cold_restart = build_dnp3_detection_frame(0x0D, 0x0003, 0x0001);
        analyzer.on_data(key.clone(), &cold_restart, 100, Direction::ClientToServer);

        let summary = analyzer.summarize();
        let dropped = summary
            .detail
            .get("dropped_findings")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.15.022 v1.5 PC-5: summarize() must contain 'dropped_findings' after a \
                     cap-suppressed T0814 finding (all_findings was at MAX_FINDINGS={}). \
                     Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    MAX_FINDINGS,
                    summary.detail.keys().collect::<Vec<_>>()
                )
            })
            .as_u64()
            .expect("'dropped_findings' must be a u64");

        assert!(
            dropped >= 1,
            "BC-2.15.022 v1.5 PC-5: 'dropped_findings' must be >= 1 after a cap-suppressed \
             COLD_RESTART T0814 finding (all_findings at MAX_FINDINGS={}). Got 0.",
            MAX_FINDINGS
        );
    }

    /// BC-2.15.016 v2.1 PC-6 / BC-2.15.020 v1.5:
    /// `master_addrs_dropped` increments by 1 when the 65th unique master source address
    /// arrives at a flow whose `master_addrs_seen` is already at MAX_MASTER_ADDRS=64.
    ///
    /// Mechanism: feed 64 READ (FC=0x01) frames with distinct source addresses (src=0..63),
    /// each with CTRL=0xC4 (DIR=1 → master-direction). This fills `master_addrs_seen` to 64.
    /// Then feed a 65th frame with src=64 — the cap gate prevents the push and
    /// `master_addrs_dropped` must be incremented. Mirrors BC-2.15.016 EC-011.
    ///
    /// RED GATE: `master_addrs_dropped` not yet a field; key absent → panic.
    #[test]
    fn test_BC_2_15_016_master_addrs_dropped_increments_on_65th_unique_master_addr() {
        // High threshold to suppress T1692.001 burst findings across 65 frames.
        let mut analyzer = Dnp3Analyzer::new(1000);
        let key = dnp3_flow_key();

        // Feed MAX_MASTER_ADDRS=64 distinct master source addresses to one flow.
        // CTRL=0xC4: is_master_frame(0xC4) = (0xC4 & 0x80 != 0) = true.
        for i in 0u16..MAX_MASTER_ADDRS as u16 {
            let frame = build_dnp3_detection_frame(0x01, 0x0003, i);
            analyzer.on_data(key.clone(), &frame, i as u32, Direction::ClientToServer);
        }

        // Verify master_addrs_seen is exactly at cap before the overflow frame.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after 64 frames");
        assert_eq!(
            flow.master_addrs_seen.len(),
            MAX_MASTER_ADDRS,
            "pre-condition: master_addrs_seen must be at MAX_MASTER_ADDRS={} before overflow",
            MAX_MASTER_ADDRS
        );

        // 65th unique master address (src=64, not yet in master_addrs_seen) —
        // cap gate fires: `len < MAX_MASTER_ADDRS` is false → push skipped.
        // `master_addrs_dropped` must be incremented by 1 (EC-011, PC-6).
        let overflow_frame = build_dnp3_detection_frame(0x01, 0x0003, MAX_MASTER_ADDRS as u16);
        analyzer.on_data(
            key.clone(),
            &overflow_frame,
            MAX_MASTER_ADDRS as u32,
            Direction::ClientToServer,
        );

        let summary = analyzer.summarize();
        let dropped = summary
            .detail
            .get("master_addrs_dropped")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.15.016 v2.1 PC-6 / BC-2.15.020 v1.5: summarize() must contain \
                     'master_addrs_dropped' after a 65th unique master address is silently \
                     ignored at MAX_MASTER_ADDRS={}. \
                     Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    MAX_MASTER_ADDRS,
                    summary.detail.keys().collect::<Vec<_>>()
                )
            })
            .as_u64()
            .expect("'master_addrs_dropped' must be a u64");

        assert!(
            dropped >= 1,
            "BC-2.15.016 v2.1 EC-011: 'master_addrs_dropped' must be >= 1 after a 65th \
             unique master address arrives at MAX_MASTER_ADDRS={}. Got 0.",
            MAX_MASTER_ADDRS
        );
    }

    /// BC-2.15.016 v2.1 PC-6 / BC-2.15.020 v1.5 Invariant 5 (NEGATIVE):
    /// Re-seeing an already-known master source address when `master_addrs_seen` is at
    /// MAX_MASTER_ADDRS=64 MUST NOT increment `master_addrs_dropped`.
    ///
    /// Rationale: the drop counter must only fire when a NEW address is silently ignored
    /// due to cap pressure.  An address already in `master_addrs_seen` is a known address —
    /// the `contains()` check in the push gate short-circuits before the `len < cap` gate,
    /// so the counter is never reached (confirmed in df-validation-pc019-pc020-2026-07-06.md
    /// §PC-016: "A full `master_addrs_seen` does NOT cause `contains(new_src)` to return
    /// `true` — it returns `false` for any address not already present").
    ///
    /// Mirrors the HTTP-AC008-NEG-TEST-001 pattern (existing-key reuse must not bump counter).
    ///
    /// RED GATE: `master_addrs_dropped` key absent → panic before the no-increment assertion.
    #[test]
    fn test_BC_2_15_016_NEG_known_master_addr_at_cap_does_not_increment_master_addrs_dropped() {
        let mut analyzer = Dnp3Analyzer::new(1000);
        let key = dnp3_flow_key();

        // Fill master_addrs_seen to exactly MAX_MASTER_ADDRS=64 with src=0..63.
        for i in 0u16..MAX_MASTER_ADDRS as u16 {
            let frame = build_dnp3_detection_frame(0x01, 0x0003, i);
            analyzer.on_data(key.clone(), &frame, i as u32, Direction::ClientToServer);
        }

        // Snapshot counter before re-visiting a known address.
        // If the key is absent, this step panics — red gate achieved.
        let before_summary = analyzer.summarize();
        let before = before_summary
            .detail
            .get("master_addrs_dropped")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.15.016 v2.1 NEG: 'master_addrs_dropped' key must be present in \
                     summarize() at the start of the negative assertion. \
                     Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    before_summary.detail.keys().collect::<Vec<_>>()
                )
            })
            .as_u64()
            .expect("'master_addrs_dropped' must be a u64");

        // Re-visit src=0 (already in master_addrs_seen) when cap is full.
        // contains(0) = true → the cap gate never fires → counter must NOT increment.
        let repeat_frame = build_dnp3_detection_frame(0x01, 0x0003, 0u16);
        analyzer.on_data(
            key.clone(),
            &repeat_frame,
            MAX_MASTER_ADDRS as u32 + 1,
            Direction::ClientToServer,
        );

        let after_summary = analyzer.summarize();
        let after = after_summary
            .detail
            .get("master_addrs_dropped")
            .and_then(|v| v.as_u64())
            .expect("'master_addrs_dropped' key must still be present after re-visit");

        assert_eq!(
            after, before,
            "BC-2.15.016 v2.1 NEG (negative): 'master_addrs_dropped' must NOT increment when \
             a KNOWN master address (src=0, already in master_addrs_seen) is re-seen while \
             master_addrs_seen is at MAX_MASTER_ADDRS={}. \
             Before re-visit: {before}, After re-visit: {after}.",
            MAX_MASTER_ADDRS
        );
    }

    /// BC-2.15.016 v2.1 PC-10 / BC-2.15.020 v1.5:
    /// `pending_requests_evicted` increments by 1 on each LRU eviction from
    /// `pending_requests` at MAX_PENDING_REQUESTS=256.
    ///
    /// Mechanism: feed 256 DIRECT_OPERATE (FC=0x05) frames with distinct dest=0..255 to one
    /// flow (src=1 fixed, app_seq=0 from byte[11]=0x00). Each frame inserts key `(dest, 0)`
    /// into `pending_requests`. At MAX_PENDING_REQUESTS=256 entries, the 257th unique key
    /// (dest=256) triggers the LRU eviction in `insert_pending_request` — oldest entry is
    /// evicted, new entry inserted, `pending_requests_evicted` incremented. Mirrors EC-008.
    ///
    /// RED GATE: `pending_requests_evicted` not yet a field; key absent → panic.
    #[test]
    fn test_BC_2_15_016_pending_requests_evicted_increments_on_lru_eviction() {
        // High threshold to suppress T1692.001 burst findings across 257 frames.
        let mut analyzer = Dnp3Analyzer::new(1000);
        let key = dnp3_flow_key();

        // Fill pending_requests to MAX_PENDING_REQUESTS=256 with distinct (dest, 0) keys.
        // src=1 is fixed — it is added to master_addrs_seen on the first frame, so all
        // subsequent frames from src=1 have src_was_known=true (no unexpected-source findings).
        //
        // All frames use ts=1000 so scan_block_timeouts never removes entries: the timeout
        // check is `now_ts.saturating_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS (10s)`, which
        // evaluates to `1000 - 1000 = 0 > 10 = false` on every call.  Using monotonically
        // increasing timestamps (dest*2) would cause entries to time out once elapsed > 10s.
        for dest in 0u16..MAX_PENDING_REQUESTS as u16 {
            let frame = build_dnp3_detection_frame(0x05, dest, 1u16);
            analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);
        }

        // Verify pending_requests is at cap before the eviction frame.
        let flow = analyzer
            .flows
            .get(&key)
            .expect("flow must exist after 256 frames");
        assert_eq!(
            flow.pending_requests.len(),
            MAX_PENDING_REQUESTS,
            "pre-condition: pending_requests must be at MAX_PENDING_REQUESTS={} before overflow",
            MAX_PENDING_REQUESTS
        );

        // 257th unique key (dest=256, not yet in pending_requests) — triggers LRU eviction.
        // After: oldest entry evicted, new entry inserted, map still at 256,
        //        `pending_requests_evicted` incremented by 1.
        let eviction_frame = build_dnp3_detection_frame(0x05, MAX_PENDING_REQUESTS as u16, 1u16);
        analyzer.on_data(
            key.clone(),
            &eviction_frame,
            1000,
            Direction::ClientToServer,
        );

        let summary = analyzer.summarize();
        let evicted = summary
            .detail
            .get("pending_requests_evicted")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.15.016 v2.1 PC-10 / BC-2.15.020 v1.5: summarize() must contain \
                     'pending_requests_evicted' after a 257th unique DIRECT_OPERATE request \
                     triggers LRU eviction at MAX_PENDING_REQUESTS={}. \
                     Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    MAX_PENDING_REQUESTS,
                    summary.detail.keys().collect::<Vec<_>>()
                )
            })
            .as_u64()
            .expect("'pending_requests_evicted' must be a u64");

        assert!(
            evicted >= 1,
            "BC-2.15.016 v2.1 EC-008: 'pending_requests_evicted' must be >= 1 after a 257th \
             unique Control-class request triggers LRU eviction at MAX_PENDING_REQUESTS={}. \
             Got 0.",
            MAX_PENDING_REQUESTS
        );
    }

    /// BC-2.15.016 v2.1 PC-10 (NEGATIVE):
    /// Normal request/response completion MUST NOT increment `pending_requests_evicted`.
    ///
    /// When a RESPONSE (FC=0x81) arrives and `pending_requests.remove(&(src, app_seq))`
    /// succeeds, the entry is removed via normal completion — not LRU eviction.  The
    /// `pending_requests_evicted` counter must remain 0.
    ///
    /// Scenario: DIRECT_OPERATE request (dest=3, src=1, app_seq=0) inserts key (3, 0).
    /// Matching RESPONSE (src=3, dest=1, app_seq=0) removes (3, 0) via `pending_requests.remove`.
    /// No eviction helper involved; pending_requests stays well below MAX_PENDING_REQUESTS.
    ///
    /// RED GATE: key absent → panic before the zero assertion.
    #[test]
    fn test_BC_2_15_016_NEG_normal_request_response_completion_does_not_increment_pending_requests_evicted()
     {
        let mut analyzer = Dnp3Analyzer::new(1000);
        let key = dnp3_flow_key();

        // DIRECT_OPERATE request: dest=3 (outstation), src=1 (master), app_seq=0.
        // Inserts key (3, 0) into pending_requests.
        let request_frame = build_dnp3_detection_frame(0x05, 3u16, 1u16);
        analyzer.on_data(key.clone(), &request_frame, 100, Direction::ClientToServer);

        // Matching RESPONSE: FC=0x81, src=3 (outstation), dest=1 (master), app_seq=0.
        // Calls pending_requests.remove(&(3, 0)) — normal completion, not eviction.
        let response_frame = build_dnp3_detection_frame(0x81, 1u16, 3u16);
        analyzer.on_data(key.clone(), &response_frame, 105, Direction::ServerToClient);

        let summary = analyzer.summarize();
        let evicted = summary
            .detail
            .get("pending_requests_evicted")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.15.016 v2.1 NEG: 'pending_requests_evicted' key must be present in \
                     summarize() after a normal request/response completion. \
                     Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    summary.detail.keys().collect::<Vec<_>>()
                )
            })
            .as_u64()
            .expect("'pending_requests_evicted' must be a u64");

        assert_eq!(
            evicted, 0,
            "BC-2.15.016 v2.1 (negative): 'pending_requests_evicted' must be 0 after a normal \
             DIRECT_OPERATE (FC=0x05) / RESPONSE (FC=0x81) completion. Normal pending_requests \
             removal is NOT an LRU eviction. Got {evicted}."
        );
    }

    // -----------------------------------------------------------------------
    // NEGATIVE: observability counter events must NOT emit any Finding
    // (DNP3 counterpart of EVICTION-NO-FINDING-NEG-TEST-001 from
    // bc_silent_resource_caps_tests.rs; precedent: BC-2.16.008 Inv-5,
    // BC-2.14.012 v1.1, EVICTION-NO-FINDING-NEG-TEST-001)
    // -----------------------------------------------------------------------

    /// DNP3-EVICTION-NO-FINDING-NEG-TEST-001 / BC-2.15.016 v2.1 Inv-2,5 /
    /// BC-2.15.022 v1.5 Inv-5 (NEGATIVE):
    ///
    /// All three observability counter events are COUNTER-ONLY — none must emit a Finding:
    ///
    /// Part A — `master_addrs_dropped` (65 frames):
    ///   Fill master_addrs_seen to 64 via src=0..63, then send 65th new master address.
    ///   Counter increments; `all_findings` must NOT grow due to the drop event.
    ///
    /// Part B — `pending_requests_evicted` (257 frames):
    ///   Fill pending_requests to 256 via dest=0..255, then send 257th unique request.
    ///   Counter increments; `all_findings` must NOT grow due to the eviction event.
    ///
    /// Part C — `dropped_findings` (pre-fill + 1 frame):
    ///   Pre-fill all_findings to MAX_FINDINGS, then deliver COLD_RESTART.
    ///   Counter increments; `all_findings.len()` must stay at MAX_FINDINGS.
    ///
    /// RED GATE: each Part first asserts its counter >= 1 (via `.unwrap_or_else` on the
    /// missing key) — tests fail at that assertion before the no-Finding check runs.
    #[test]
    fn test_DNP3_EVICTION_NO_FINDING_NEG_TEST_001_observability_counters_emit_no_finding() {
        // ---- Part A: master_addrs_dropped is COUNTER-ONLY ----
        {
            let mut analyzer = Dnp3Analyzer::new(1000);
            let key = dnp3_flow_key();

            for i in 0u16..MAX_MASTER_ADDRS as u16 {
                let frame = build_dnp3_detection_frame(0x01, 0x0003, i);
                analyzer.on_data(key.clone(), &frame, i as u32, Direction::ClientToServer);
            }
            let findings_before_drop = analyzer.all_findings.len();

            let overflow_frame = build_dnp3_detection_frame(0x01, 0x0003, MAX_MASTER_ADDRS as u16);
            analyzer.on_data(
                key.clone(),
                &overflow_frame,
                MAX_MASTER_ADDRS as u32,
                Direction::ClientToServer,
            );

            // Assert counter was incremented (RED GATE: panics on missing key).
            let summary = analyzer.summarize();
            let dropped_addrs = summary
                .detail
                .get("master_addrs_dropped")
                .unwrap_or_else(|| {
                    panic!(
                        "DNP3-EVICTION-NO-FINDING-NEG-TEST-001 Part A: 'master_addrs_dropped' \
                         must be present after 65th unique master address arrives. \
                         Key MISSING — red gate expected. Keys: {:?}",
                        summary.detail.keys().collect::<Vec<_>>()
                    )
                })
                .as_u64()
                .expect("'master_addrs_dropped' must be u64");
            assert!(
                dropped_addrs >= 1,
                "Part A pre-condition: master_addrs_dropped must be >= 1 to confirm cap was hit. \
                 Got 0 — cannot verify no-Finding invariant without a confirmed cap event."
            );

            // Assert no new Finding from the master_addrs drop event itself.
            let findings_after_drop = analyzer.all_findings.len();
            assert_eq!(
                findings_after_drop, findings_before_drop,
                "DNP3-EVICTION-NO-FINDING-NEG-TEST-001 Part A / BC-2.15.016 v2.1 Inv-2 \
                 (negative): `all_findings` must not grow due to a master_addrs_dropped counter \
                 event. Before drop: {} finding(s). After drop: {} finding(s). \
                 A cap-triggered master-address ignore is COUNTER-ONLY — no Finding emitted.",
                findings_before_drop, findings_after_drop
            );
        }

        // ---- Part B: pending_requests_evicted is COUNTER-ONLY ----
        {
            let mut analyzer = Dnp3Analyzer::new(1000);
            let key = dnp3_flow_key();

            // ts=1000 fixed for all frames so scan_block_timeouts never removes entries:
            // `1000.saturating_sub(1000) = 0 > BLOCK_CMD_TIMEOUT_SECS (10)` → false.
            for dest in 0u16..MAX_PENDING_REQUESTS as u16 {
                let frame = build_dnp3_detection_frame(0x05, dest, 1u16);
                analyzer.on_data(key.clone(), &frame, 1000, Direction::ClientToServer);
            }
            let findings_before_evict = analyzer.all_findings.len();

            let eviction_frame =
                build_dnp3_detection_frame(0x05, MAX_PENDING_REQUESTS as u16, 1u16);
            analyzer.on_data(
                key.clone(),
                &eviction_frame,
                1000,
                Direction::ClientToServer,
            );

            // Assert counter was incremented (RED GATE: panics on missing key).
            let summary = analyzer.summarize();
            let evicted = summary
                .detail
                .get("pending_requests_evicted")
                .unwrap_or_else(|| {
                    panic!(
                        "DNP3-EVICTION-NO-FINDING-NEG-TEST-001 Part B: 'pending_requests_evicted' \
                         must be present after LRU eviction at MAX_PENDING_REQUESTS={}. \
                         Key MISSING — red gate expected. Keys: {:?}",
                        MAX_PENDING_REQUESTS,
                        summary.detail.keys().collect::<Vec<_>>()
                    )
                })
                .as_u64()
                .expect("'pending_requests_evicted' must be u64");
            assert!(
                evicted >= 1,
                "Part B pre-condition: pending_requests_evicted must be >= 1 to confirm eviction \
                 fired. Got 0 — cannot verify no-Finding invariant without a confirmed eviction."
            );

            // Assert no new Finding from the LRU eviction event itself.
            let findings_after_evict = analyzer.all_findings.len();
            assert_eq!(
                findings_after_evict, findings_before_evict,
                "DNP3-EVICTION-NO-FINDING-NEG-TEST-001 Part B / BC-2.15.016 v2.1 Inv-5 \
                 (negative): `all_findings` must not grow due to a pending_requests LRU \
                 eviction. Before eviction: {} finding(s). After eviction: {} finding(s). \
                 LRU eviction is COUNTER-ONLY — no Finding, no T1691.001 timeout event.",
                findings_before_evict, findings_after_evict
            );
        }

        // ---- Part C: dropped_findings cap-drop is COUNTER-ONLY ----
        {
            let mut analyzer = Dnp3Analyzer::new(10);
            let key = dnp3_flow_key();

            for _ in 0..MAX_FINDINGS {
                analyzer.all_findings.push(dummy_finding());
            }

            let read_frame = build_dnp3_detection_frame(0x01, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &read_frame, 0, Direction::ClientToServer);

            // COLD_RESTART would push T0814 but cap hit → dropped_findings incremented.
            let cold_restart = build_dnp3_detection_frame(0x0D, 0x0003, 0x0001);
            analyzer.on_data(key.clone(), &cold_restart, 100, Direction::ClientToServer);

            // Assert counter was incremented (RED GATE: panics on missing key).
            let summary = analyzer.summarize();
            let dropped_count = summary
                .detail
                .get("dropped_findings")
                .unwrap_or_else(|| {
                    panic!(
                        "DNP3-EVICTION-NO-FINDING-NEG-TEST-001 Part C: 'dropped_findings' must \
                         be present after cap-suppressed T0814 at MAX_FINDINGS={}. \
                         Key MISSING — red gate expected. Keys: {:?}",
                        MAX_FINDINGS,
                        summary.detail.keys().collect::<Vec<_>>()
                    )
                })
                .as_u64()
                .expect("'dropped_findings' must be u64");
            assert!(
                dropped_count >= 1,
                "Part C pre-condition: dropped_findings must be >= 1 to confirm cap was hit. \
                 Got 0 — cannot verify no-growth invariant without a confirmed cap event."
            );

            // Assert all_findings did NOT grow beyond MAX_FINDINGS.
            assert_eq!(
                analyzer.all_findings.len(),
                MAX_FINDINGS,
                "DNP3-EVICTION-NO-FINDING-NEG-TEST-001 Part C / BC-2.15.022 v1.5 Inv-5 \
                 (negative): `all_findings.len()` must stay at MAX_FINDINGS={} after a \
                 cap-suppressed finding event. MAX_FINDINGS cap-drop is COUNTER-ONLY — \
                 `all_findings` must NOT grow beyond the cap. Got {}.",
                MAX_FINDINGS,
                analyzer.all_findings.len()
            );
        }
    }
} // mod bc_2_15_020_dnp3_observability_counters
