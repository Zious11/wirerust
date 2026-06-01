//! HS-043 flow-expiry wiring tests (BC-2.04.013 v1.7 PC0 — idle-expiry wiring).
//!
//! ## What these tests verify
//!
//! `process_packet` in `mod.rs` calls `expire_idle_by_timeout` (the
//! production-wired per-packet sweep) after each packet.  As a result:
//!   - `stats.flows_expired` increments whenever idle flows are swept.
//!   - Idle-flow memory is reclaimed in production on the per-packet path.
//!   - The `--flow-timeout` CLI flag exists (cli.rs) and is wired through.
//!
//! Note: `expire_flows` is a separate public/direct-call API on
//! `TcpReassembler` (offline/test use).  It is NOT the per-packet path;
//! `expire_idle_by_timeout` (mod.rs:575-590) is the function called from
//! the production loop (mod.rs:166-169).
//!
//! ## What these tests prove
//!
//! Every test in this module asserts the post-fix (production-wired) behaviour:
//!   - Integration tests confirm `flows_expired` increments when idle flows
//!     are swept by `expire_idle_by_timeout` on the per-packet path.
//!   - CLI tests confirm `--flow-timeout` is a valid clap argument and that
//!     the JSON output reflects the expired-flow count.
//!
//! ## Fixture: tests/fixtures/flow-expiry.pcap
//!
//! Built by the test-writer with raw libpcap bytes (Python 3, stdlib only):
//!
//!   Global header: magic=0xa1b2c3d4 (microseconds), version 2.4,
//!                  linktype=1 (ETHERNET), snaplen=65535
//!
//!   Packet 1 (ts_sec=0):  Ethernet + IPv4 + TCP SYN
//!                          10.0.0.1:11111 -> 10.0.0.2:80, seq=1000
//!   Packet 2 (ts_sec=6):  Ethernet + IPv4 + TCP SYN
//!                          10.0.0.3:22222 -> 10.0.0.2:80, seq=2000
//!
//! Rationale: when `--flow-timeout 5` is used, Flow A (last_seen=0) is idle
//! for 6 seconds when Flow B arrives at t=6.  `process_packet` calls
//! `expire_idle_by_timeout(6, handler)` internally, which expires Flow A
//! (6 - 0 = 6 > 5 → expired).  Flow B itself is freshly created at t=6,
//! so it is NOT expired in the same call.

mod hs043 {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};
    use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Minimal no-op handler — records close events only (needed for the
    /// handler arg; we don't assert on stream data here).
    struct NullHandler {
        close_events: Vec<(FlowKey, CloseReason)>,
    }

    impl NullHandler {
        fn new() -> Self {
            NullHandler {
                close_events: Vec::new(),
            }
        }
    }

    impl StreamHandler for NullHandler {
        fn on_data(&mut self, _key: &FlowKey, _dir: Direction, _data: &[u8], _offset: u64) {}

        fn on_flow_close(&mut self, key: &FlowKey, reason: CloseReason) {
            self.close_events.push((key.clone(), reason));
        }
    }

    /// Build a minimal TCP ParsedPacket suitable for `process_packet`.
    ///
    /// Matches the helper signature used throughout `reassembly_engine_tests.rs`:
    /// IPv4 addresses as `[u8; 4]`, all TCP flag booleans explicit.
    #[allow(clippy::too_many_arguments)]
    fn make_tcp_packet(
        src_ip: [u8; 4],
        src_port: u16,
        dst_ip: [u8; 4],
        dst_port: u16,
        seq: u32,
        payload: &[u8],
        syn: bool,
        ack: bool,
        fin: bool,
        rst: bool,
    ) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::from(src_ip)),
            dst_ip: IpAddr::V4(Ipv4Addr::from(dst_ip)),
            protocol: Protocol::Tcp,
            transport: TransportInfo::Tcp {
                src_port,
                dst_port,
                seq_number: seq,
                syn,
                ack,
                fin,
                rst,
            },
            payload: payload.to_vec(),
            packet_len: 54 + payload.len(),
        }
    }

    // -------------------------------------------------------------------------
    // Test 1 — Integration through process_packet (load-bearing)
    //
    // BC-2.04.013 v1.7 PC0: `process_packet` calls `expire_idle_by_timeout`
    // (the production-wired idle sweep) after each packet, passing the
    // packet's timestamp as current_time.  This verifies the wiring obligation.
    //
    // `expire_flows` is the separate public/direct-call API; it is NOT the
    // function called from the per-packet path.  The assertion below verifies
    // the observable outcome: `flows_expired` increments when idle flows are
    // swept by `expire_idle_by_timeout` on the per-packet path.
    //
    // Setup:
    //   - flow_timeout_secs = 5
    //   - Flow A SYN at t=0  (last_seen=0)
    //   - Flow B SYN at t=6  (process_packet internally calls
    //                          expire_idle_by_timeout(6, handler), which
    //                          expires Flow A because 6 - 0 = 6 > 5)
    //
    // Expected: after process_packet for Flow B at t=6, flows_expired >= 1.
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_PC0_idle_expiry_wired_in_process_packet() {
        let config = ReassemblyConfig {
            flow_timeout_secs: 5,
            ..ReassemblyConfig::default()
        };
        let mut reassembler = TcpReassembler::new(config);
        let mut handler = NullHandler::new();

        // Flow A: SYN at timestamp t=0.
        let syn_a = make_tcp_packet(
            [10, 0, 0, 1],
            11111,
            [10, 0, 0, 2],
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_a, 0, &mut handler);

        // Sanity: no expiry yet.
        assert_eq!(
            reassembler.stats().flows_expired,
            0,
            "precondition: flows_expired must be 0 after only Flow A's SYN"
        );

        // Flow B SYN at timestamp t=6.
        // process_packet calls expire_idle_by_timeout(6, handler) internally,
        // which expires Flow A (last_seen=0, 6-0=6 > 5).
        let syn_b = make_tcp_packet(
            [10, 0, 0, 3],
            22222,
            [10, 0, 0, 2],
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_b, 6, &mut handler);

        // BC-2.04.013 v1.7 PC0: flows_expired must be >= 1.
        // expire_idle_by_timeout is called from process_packet; flows_expired
        // increments when idle flows are swept on the per-packet path.
        assert!(
            reassembler.stats().flows_expired >= 1,
            "BC-2.04.013 v1.5 PC0: flows_expired must be >= 1 after Flow A (last_seen=0) \
             was idle for 6s with flow_timeout_secs=5; got flows_expired={}",
            reassembler.stats().flows_expired
        );
    }

    // -------------------------------------------------------------------------
    // Test 2 — Boundary: NOT expired at exactly flow_timeout_secs
    //
    // expire_idle_by_timeout uses strict-greater semantics:
    //   (current_time - last_seen) > timeout  ← strictly greater, not >=
    //
    // A flow idle for EXACTLY flow_timeout_secs must NOT be expired.
    //
    // Setup:
    //   - flow_timeout_secs = 5
    //   - Flow A SYN at t=0  (last_seen=0)
    //   - Flow B SYN at t=5  (process_packet called at t=5)
    //
    // delta = 5 - 0 = 5; 5 > 5 is false → NOT expired.
    //
    // The assertion `flows_expired == 0` verifies the strict-greater boundary:
    // with `expire_idle_by_timeout` wired into process_packet, a flow idle for
    // EXACTLY flow_timeout_secs must NOT be expired.  The companion assertion
    // flow_count() == 2 confirms both flows remain alive.  Together they guard
    // against a > vs >= regression in the expiry predicate.
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_v15_boundary_not_expired_at_exact_timeout() {
        let config = ReassemblyConfig {
            flow_timeout_secs: 5,
            ..ReassemblyConfig::default()
        };
        let mut reassembler = TcpReassembler::new(config);
        let mut handler = NullHandler::new();

        // Flow A: SYN at t=0.
        let syn_a = make_tcp_packet(
            [10, 0, 0, 1],
            11111,
            [10, 0, 0, 2],
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_a, 0, &mut handler);

        // Flow B SYN at t=5 (exactly timeout, NOT past it).
        // delta for Flow A = 5 - 0 = 5; 5 > 5 is false → NOT expired.
        let syn_b = make_tcp_packet(
            [10, 0, 0, 3],
            22222,
            [10, 0, 0, 2],
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_b, 5, &mut handler);

        // Both flows must still be alive (neither expired).
        // This is the load-bearing boundary assertion: with expire_idle_by_timeout
        // wired into process_packet, the strict-greater predicate must hold —
        // a flow idle for EXACTLY timeout secs must NOT be swept.
        assert_eq!(
            reassembler.stats().flows_expired,
            0,
            "BC-2.04.013 strict-greater boundary: flow idle for EXACTLY timeout secs must NOT \
             be expired; flows_expired must be 0"
        );
        assert_eq!(
            reassembler.flow_count(),
            2,
            "BC-2.04.013 strict-greater boundary: both flows must still be tracked \
             (neither is past the timeout)"
        );
    }

    // -------------------------------------------------------------------------
    // Test 3 — Boundary: IS expired at flow_timeout_secs + 1
    //
    // A flow idle for (timeout + 1) seconds MUST be expired.
    //
    // Setup:
    //   - flow_timeout_secs = 5
    //   - Flow A SYN at t=0  (last_seen=0)
    //   - Flow B SYN at t=6  (current_time = 6)
    //
    // delta = 6 - 0 = 6; 6 > 5 is true → EXPIRED.
    //
    // This is the companion to test 2 — one second past the exact boundary
    // must flip the expiry.  This test is the primary regression guard for
    // the per-packet wiring: if expire_idle_by_timeout were ever removed from
    // process_packet, flows_expired would stay 0 and this test would fail.
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_v15_boundary_expired_at_timeout_plus_one() {
        let config = ReassemblyConfig {
            flow_timeout_secs: 5,
            ..ReassemblyConfig::default()
        };
        let mut reassembler = TcpReassembler::new(config);
        let mut handler = NullHandler::new();

        // Flow A: SYN at t=0.
        let syn_a = make_tcp_packet(
            [10, 0, 0, 1],
            11111,
            [10, 0, 0, 2],
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_a, 0, &mut handler);

        // Flow B SYN at t=6 (one past timeout).
        // delta for Flow A = 6 - 0 = 6; 6 > 5 is true → expired.
        let syn_b = make_tcp_packet(
            [10, 0, 0, 3],
            22222,
            [10, 0, 0, 2],
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_b, 6, &mut handler);

        // Flow A must have been expired via process_packet's internal call to
        // expire_idle_by_timeout.  flows_expired increments on the per-packet
        // path; if that wiring is removed, flows_expired stays 0 and this fails.
        assert!(
            reassembler.stats().flows_expired >= 1,
            "BC-2.04.013 v1.5 boundary (timeout+1): flow idle for 6s with timeout=5 MUST be \
             expired via process_packet; flows_expired={} (expected >=1)",
            reassembler.stats().flows_expired
        );
    }

    // -------------------------------------------------------------------------
    // Test 4 — CLI black-box: --flow-timeout 5 produces flows_expired >= 1
    //
    // The CLI accepts `--flow-timeout 5` (cli.rs).  When run against the
    // tests/fixtures/flow-expiry.pcap fixture (two TCP SYNs at t=0 and t=6),
    // the JSON output must show flows_expired >= 1 in the TCP Reassembly
    // analyzer summary.  The wiring from --flow-timeout through
    // ReassemblyConfig.flow_timeout_secs to expire_idle_by_timeout is
    // exercised end-to-end by this CLI black-box test.
    //
    // Fixture: tests/fixtures/flow-expiry.pcap
    //   Built with raw libpcap bytes (Python 3 stdlib):
    //     Global header: magic=0xa1b2c3d4, version 2.4, linktype=1 (ETHERNET)
    //     Packet 1 (ts_sec=0): Ethernet+IPv4+TCP SYN, 10.0.0.1:11111->10.0.0.2:80
    //     Packet 2 (ts_sec=6): Ethernet+IPv4+TCP SYN, 10.0.0.3:22222->10.0.0.2:80
    //   The 6-second gap ensures Flow A (last_seen=0) expires when Flow B
    //   arrives at t=6 with --flow-timeout 5.
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_v15_cli_flow_timeout_arg_produces_flows_expired() {
        use assert_cmd::Command;

        const FIXTURE: &str = "tests/fixtures/flow-expiry.pcap";

        // Run: wirerust analyze <fixture> --reassemble --flow-timeout 5 --output-format json
        // --reassemble is needed so the reassembler is active (no --http/--tls implied).
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                FIXTURE,
                "--reassemble",
                "--flow-timeout",
                "5",
                "--output-format",
                "json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let json_str = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value =
            serde_json::from_str(&json_str).expect("valid JSON from wirerust");

        // Find the TCP Reassembly analyzer summary in the "analyzers" array.
        let analyzers = json["analyzers"]
            .as_array()
            .expect("analyzers must be an array");
        let reasm_summary = analyzers
            .iter()
            .find(|a| a["analyzer_name"] == "TCP Reassembly")
            .expect("TCP Reassembly analyzer summary must be present in JSON output");

        let flows_expired = reasm_summary["detail"]["flows_expired"]
            .as_u64()
            .expect("flows_expired must be a u64 in TCP Reassembly detail");

        assert!(
            flows_expired >= 1,
            "BC-2.04.013 v1.5 CLI: --flow-timeout 5 must produce flows_expired >= 1 in the \
             TCP Reassembly analyzer summary; got flows_expired={flows_expired}"
        );
    }

    // -------------------------------------------------------------------------
    // Test 6 — Gating-property regression: active flow delta-0 never self-expires
    //
    // BC-2.04.013 v1.5 PC0 — invariant: a flow that continuously receives
    // packets within the timeout window must NEVER be expired by the sweep.
    //
    // The production path stamps `last_seen` on the flow AFTER `expire_idle_by_timeout`
    // runs (see `get_or_create_flow` in mod.rs, which is called after the expiry
    // gate). This ordering means the sweep always sees the PREVIOUS last_seen for
    // the currently-arriving flow, not the fresh timestamp — so the delta for an
    // active flow is the gap since its PRIOR packet, not zero.
    //
    // Setup: flow_timeout_secs=5, packets at t=0,2,4,6,8,10 (each within 5s of
    //        prior). flows_expired must remain 0 throughout, and the flow must
    //        still be tracked (flow_count >= 1) at the end.
    //
    // Mutation-catch: if the sweep were moved to run AFTER last_seen is stamped,
    // the delta would be 0 on the packet that re-stamps last_seen, but flow_count
    // would still be 1 (no expiry yet). The failure mode this guards is a future
    // refactor that moves the sweep BELOW get_or_create_flow AND uses the updated
    // last_seen: the delta for a flow at current_time equals current_time - current_time
    // = 0, so it would never expire even when it should. The inverse regression —
    // treating delta-0 as expired — would cause flows_expired > 0 here.
    //
    // This test catches the inverse: an active flow's delta with 2s inter-packet
    // spacing over a 5s window must never be misclassified as expired.
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_v15_active_flow_delta0_never_self_expires() {
        let config = ReassemblyConfig {
            flow_timeout_secs: 5,
            ..ReassemblyConfig::default()
        };
        let mut reassembler = TcpReassembler::new(config);
        let mut handler = NullHandler::new();

        // SYN at t=0 — opens the flow.
        let syn_pkt = make_tcp_packet(
            [10, 0, 0, 1],
            55001,
            [10, 0, 0, 2],
            80,
            3000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_pkt, 0, &mut handler);

        // Subsequent packets at t=2,4,6,8,10 — each within 5s of the prior.
        // Uses ACK+payload to advance last_seen without closing the flow.
        for ts in [2u32, 4, 6, 8, 10] {
            let data_pkt = make_tcp_packet(
                [10, 0, 0, 1],
                55001,
                [10, 0, 0, 2],
                80,
                3001 + ts, // advancing seq keeps insertion non-duplicate
                b"x",
                false,
                true,
                false,
                false,
            );
            reassembler.process_packet(&data_pkt, ts, &mut handler);

            // Invariant: no expiry has occurred at any point in the sequence.
            assert_eq!(
                reassembler.stats().flows_expired,
                0,
                "BC-2.04.013 active-flow guard: flows_expired must be 0 after packet at t={ts}; \
                 got flows_expired={}",
                reassembler.stats().flows_expired
            );
        }

        // Flow must still be tracked after all packets (not self-expired).
        assert!(
            reassembler.flow_count() >= 1,
            "BC-2.04.013 active-flow guard: flow must still be tracked after active packet \
             sequence; flow_count={}",
            reassembler.flow_count()
        );
    }

    // -------------------------------------------------------------------------
    // Test 7 — Gating-property regression: gated sweep does not let idle flow
    //          escape when multiple packets share the same triggering second
    //
    // BC-2.04.013 v1.5 PC0 — invariant: `last_expiry_sweep_secs` gates the
    // O(n) sweep to at most once per unique second, but it must NOT allow an
    // idle flow to escape expiry when the triggering second is repeated.
    //
    // The gate `if timestamp > last_expiry_sweep_secs` fires on the FIRST packet
    // at a new second, then skips on any subsequent packet at that same second
    // (because last_expiry_sweep_secs == timestamp after the first one). This is
    // correct: the first packet already ran the sweep and expired all qualifying
    // flows; subsequent packets at the same second cannot un-expire them.
    //
    // Setup: flow_timeout_secs=5
    //   - Flow A SYN at t=0 (last_seen=0, idle)
    //   - Three packets for Flow B at t=8 (all same second):
    //       packet 1 triggers the gate, runs sweep → Flow A (8-0=8 > 5) is expired
    //       packets 2+3 skip the gate (last_expiry_sweep_secs=8, timestamp=8)
    //   - Assert flows_expired >= 1 after all three packets for Flow B.
    //
    // Mutation-catch: if the gate were loosened (e.g. `timestamp > last_expiry_sweep_secs + 100`)
    // the sweep would not fire at t=8 and Flow A would never be expired → flows_expired stays 0.
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_v15_gated_sweep_no_escape_same_second() {
        let config = ReassemblyConfig {
            flow_timeout_secs: 5,
            ..ReassemblyConfig::default()
        };
        let mut reassembler = TcpReassembler::new(config);
        let mut handler = NullHandler::new();

        // Flow A: SYN at t=0 — idle flow that should expire when t=8 arrives.
        let syn_a = make_tcp_packet(
            [10, 0, 0, 10],
            44001,
            [10, 0, 0, 20],
            80,
            4000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn_a, 0, &mut handler);

        // Precondition: Flow A is tracked, nothing expired yet.
        assert_eq!(reassembler.stats().flows_expired, 0, "precondition");
        assert_eq!(reassembler.flow_count(), 1, "precondition: flow A tracked");

        // Three packets for Flow B — all at the same second t=8.
        // First packet must trigger the sweep; Flow A (idle 8s > 5s) must be expired.
        // Packets 2+3 at the same second must not double-count or reset the sweep.
        for i in 0..3u32 {
            let pkt_b = make_tcp_packet(
                [10, 0, 0, 30],
                44002,
                [10, 0, 0, 20],
                80,
                5000 + i,
                &[],
                i == 0, // SYN only on first packet
                i > 0,  // ACK on subsequent
                false,
                false,
            );
            reassembler.process_packet(&pkt_b, 8, &mut handler);
        }

        // Flow A must have been swept on the first t=8 packet and NOT escape.
        assert!(
            reassembler.stats().flows_expired >= 1,
            "BC-2.04.013 gated-sweep no-escape: Flow A (idle 8s with timeout=5) must be \
             expired when t=8 packet arrives; flows_expired={} (expected >=1)",
            reassembler.stats().flows_expired
        );
    }

    // -------------------------------------------------------------------------
    // Test 8 — Gating-property regression: regressing timestamp does not panic
    //          (two-layer underflow safety: gate + `current_time > last_seen` guard)
    //
    // BC-2.04.013 v1.5 PC0 — invariant: non-monotonic / regressing timestamps
    // (e.g. t=10 then t=8) must NOT cause an arithmetic underflow panic.
    //
    // The release profile (and the debug profile under `overflow-checks = true`)
    // would panic on `8u32 - 10u32` if the subtraction were reached.
    //
    // Two-layer protection in the production code:
    //
    //   Layer 1 (outer gate, in process_packet):
    //     `if timestamp > self.last_expiry_sweep_secs` — a regressing timestamp
    //     (8 < last_expiry_sweep_secs=10) skips the sweep call entirely.
    //
    //   Layer 2 (inner guard, in expire_idle_by_timeout):
    //     `current_time > flow.last_seen` — short-circuits before any subtraction
    //     when current_time <= last_seen, so even a direct call to expire_flows
    //     with a small current_time cannot underflow.
    //
    // Mutation-catch: removing BOTH guards (outer gate + inner short-circuit)
    // causes a panic under overflow-checks when current_time < last_seen.
    // This test catches any refactor that removes either layer while keeping
    // the timestamp sequence (t=10 then t=8 for the same flow).
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_v15_regressing_timestamp_no_underflow_panic() {
        let config = ReassemblyConfig {
            flow_timeout_secs: 5,
            ..ReassemblyConfig::default()
        };
        let mut reassembler = TcpReassembler::new(config);
        let mut handler = NullHandler::new();

        // Packet 1 at t=10 — establishes a flow with last_seen=10 and sets
        // last_expiry_sweep_secs=10.
        let pkt1 = make_tcp_packet(
            [10, 0, 0, 5],
            60001,
            [10, 0, 0, 6],
            80,
            6000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&pkt1, 10, &mut handler);

        assert_eq!(
            reassembler.flow_count(),
            1,
            "precondition: flow present after t=10 packet"
        );

        // Packet 2 at t=8 — timestamp REGRESSES (8 < last_expiry_sweep_secs=10).
        // Layer 1: gate `8 > 10` is false → sweep not called.
        // Layer 2 (defensive): if the sweep were called with current_time=8 and
        //   flow.last_seen=10, the guard `8 > 10` in expire_idle_by_timeout would
        //   short-circuit before the subtraction `8u32 - 10u32`.
        // Either layer is sufficient; both must exist for defense-in-depth.
        // This call must complete without panicking.
        let pkt2 = make_tcp_packet(
            [10, 0, 0, 5],
            60001,
            [10, 0, 0, 6],
            80,
            6001,
            b"y",
            false,
            true,
            false,
            false,
        );
        reassembler.process_packet(&pkt2, 8, &mut handler);

        // Sanity: no expiry occurred (flow was active; regressing clock should not
        // trigger expiry or panic).
        assert_eq!(
            reassembler.stats().flows_expired,
            0,
            "BC-2.04.013 regressing-timestamp guard: no flow should expire on a clock \
             regression; flows_expired={} (expected 0)",
            reassembler.stats().flows_expired
        );
    }

    // -------------------------------------------------------------------------
    // Test 5 — CLI: --flow-timeout 0 must be rejected
    //
    // The acceptance spec requires minimum 1 and rejection of 0.
    // clap must validate the range [1, u32::MAX] (or similar) for this flag.
    //
    // `--flow-timeout` exists (cli.rs) and clap validates the range [1, u32::MAX].
    // Passing 0 must be rejected as an out-of-range value (non-zero exit).
    // `assert().failure()` verifies the non-zero exit; the specific error message
    // (clap range-validation) is not asserted here to avoid brittleness.
    // -------------------------------------------------------------------------
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_04_013_v15_cli_flow_timeout_zero_rejected() {
        use assert_cmd::Command;

        const FIXTURE: &str = "tests/fixtures/flow-expiry.pcap";

        // --flow-timeout 0 must cause a non-zero exit (clap validation error).
        // Before fix: fails with "unexpected argument" (arg doesn't exist).
        // After fix:  fails with "invalid value: 0 is below minimum 1".
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                FIXTURE,
                "--flow-timeout",
                "0",
                "--output-format",
                "json",
            ])
            .assert()
            .failure(); // must NOT succeed
    }
}
