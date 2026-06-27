//! Regression tests for DNP3 `control_operation_counts` determinism.
//!
//! Traces to: BC-2.15.020 postcondition 1 (`control_operation_counts` must be
//! byte-identical across runs regardless of HashMap iteration order).
//!
//! Bug fixed in v0.9.2: `summarize()` previously called
//! `self.flows.values().enumerate()` over a `HashMap`, whose iteration order is
//! randomized per-process. Flow index `i` therefore mapped to different flows
//! each run, producing non-deterministic `control_operation_counts` values even
//! though the BTreeMap key-sort masked the issue at the map-key level.
//!
//! Fix: sort `flows.iter()` by `FlowKey` (which now derives `Ord` via
//! lexicographic `(lower_ip, lower_port, upper_ip, upper_port)`) before
//! `enumerate()`, making index→value assignment insertion-order-independent.
//!
//! ## Test naming convention
//! `test_BC_S_SS_NNN_xxx` following the project TDD standard (DF-TEST-NAMESPACE-001).
//!
//! ## Why no pcap fixture?
//! A within-process re-run cannot catch per-process HashMap randomization — both
//! runs would use the same seed and produce the same (possibly wrong) order.
//! Instead, these tests control insertion order directly by feeding two analyzers
//! with the same flows in opposite orders and asserting byte-identical output.
//!
//! ## Failure proof
//! With the pre-fix `self.flows.values().enumerate()` code, the
//! `test_BC_2_15_020_control_operation_counts_sorted_by_flow_key` test FAILS
//! because the index-to-value mapping changes with insertion order. After the
//! fix the test passes deterministically.

// BC traceability uses uppercase BC identifiers in function names; suppress lint.
#![allow(non_snake_case)]

mod dnp3_determinism {
    use std::net::{IpAddr, Ipv4Addr};

    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::Direction;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build a minimal valid DNP3 frame delivering application FC `app_fc`.
    ///
    /// LENGTH=8 → frame_len = 5+8+2*ceil(3/16) = 5+8+2 = 15 bytes.
    /// CTRL=0xC4: DIR=1, PRM=1, UNCONFIRMED_USER_DATA → has_user_data=true,
    /// frame reaches byte 12 (app_fc) so detection branches fire.
    fn build_frame(app_fc: u8, dest: u16, src: u16) -> Vec<u8> {
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
        frame[10] = 0xC0; // transport: FIR=1, FIN=1
        frame[11] = 0x00; // app control
        frame[12] = app_fc;
        frame
    }

    /// Three distinct FlowKeys with a known, total sort order:
    ///   flow_key_a < flow_key_b < flow_key_c
    ///
    /// FlowKey sorts by (lower_ip, lower_port, upper_ip, upper_port).
    /// All use lower_ip=10.0.0.1 and the upper_ip distinguishes them:
    ///   a: (10.0.0.1:20000, 10.0.0.2:20000)
    ///   b: (10.0.0.1:20000, 10.0.0.3:20000)
    ///   c: (10.0.0.1:20000, 10.0.0.4:20000)
    fn flow_key_a() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            20000,
        )
    }

    fn flow_key_b() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3)),
            20000,
        )
    }

    fn flow_key_c() -> FlowKey {
        FlowKey::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            20000,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 4)),
            20000,
        )
    }

    // -----------------------------------------------------------------------
    // FlowKey sort-order sanity check
    // -----------------------------------------------------------------------

    /// Verifies that the three test FlowKeys have the expected total order
    /// (flow_key_a < flow_key_b < flow_key_c) so the determinism assertions
    /// below have a known ground truth.
    #[test]
    fn test_flow_key_sort_order_sanity() {
        assert!(flow_key_a() < flow_key_b(), "expected key_a < key_b");
        assert!(flow_key_b() < flow_key_c(), "expected key_b < key_c");
        assert!(flow_key_a() < flow_key_c(), "expected key_a < key_c");
    }

    // -----------------------------------------------------------------------
    // BC-2.15.020 determinism regression: insertion-order independence
    // -----------------------------------------------------------------------

    /// Regression test for v0.9.2 determinism fix (BC-2.15.020 postcondition 1).
    ///
    /// Two analyzers receive the SAME three flows (A, B, C) in OPPOSITE insertion
    /// orders (A→B→C vs C→B→A). After `summarize()`, the serialized
    /// `control_operation_counts` maps must be BYTE-IDENTICAL.
    ///
    /// This proves insertion / HashMap-iteration order does not affect output.
    /// The test WOULD FAIL against the pre-fix `self.flows.values().enumerate()`
    /// code because the HashMap's per-call iteration order differs between the
    /// two instances, causing index "0" to map to different counts.
    ///
    /// Traces to: BC-2.15.020 postcondition 1; v0.9.2 determinism fix.
    #[test]
    fn test_BC_2_15_020_control_operation_counts_insertion_order_independent() {
        // Flow A: 1 DIRECT_OPERATE (FC=0x05).
        // Flow B: 3 DIRECT_OPERATE.
        // Flow C: 7 DIRECT_OPERATE.
        // Distinct counts make any index-permutation immediately visible.
        let frame_a = build_frame(0x05, 0x0002, 0x0001); // dest=2, src=1
        let frame_b = build_frame(0x05, 0x0003, 0x0001); // dest=3, src=1
        let frame_c = build_frame(0x05, 0x0004, 0x0001); // dest=4, src=1

        // Analyzer 1: insert A first, then B, then C.
        let mut analyzer1 = Dnp3Analyzer::new(100);
        analyzer1.on_data(flow_key_a(), &frame_a, 1, Direction::ClientToServer);
        analyzer1.on_data(flow_key_b(), &frame_b, 2, Direction::ClientToServer);
        analyzer1.on_data(flow_key_b(), &frame_b, 3, Direction::ClientToServer);
        analyzer1.on_data(flow_key_b(), &frame_b, 4, Direction::ClientToServer);
        analyzer1.on_data(flow_key_c(), &frame_c, 5, Direction::ClientToServer);
        analyzer1.on_data(flow_key_c(), &frame_c, 6, Direction::ClientToServer);
        analyzer1.on_data(flow_key_c(), &frame_c, 7, Direction::ClientToServer);
        analyzer1.on_data(flow_key_c(), &frame_c, 8, Direction::ClientToServer);
        analyzer1.on_data(flow_key_c(), &frame_c, 9, Direction::ClientToServer);
        analyzer1.on_data(flow_key_c(), &frame_c, 10, Direction::ClientToServer);
        analyzer1.on_data(flow_key_c(), &frame_c, 11, Direction::ClientToServer);

        // Analyzer 2: insert C first, then B, then A (reverse order).
        let mut analyzer2 = Dnp3Analyzer::new(100);
        analyzer2.on_data(flow_key_c(), &frame_c, 1, Direction::ClientToServer);
        analyzer2.on_data(flow_key_c(), &frame_c, 2, Direction::ClientToServer);
        analyzer2.on_data(flow_key_c(), &frame_c, 3, Direction::ClientToServer);
        analyzer2.on_data(flow_key_c(), &frame_c, 4, Direction::ClientToServer);
        analyzer2.on_data(flow_key_c(), &frame_c, 5, Direction::ClientToServer);
        analyzer2.on_data(flow_key_c(), &frame_c, 6, Direction::ClientToServer);
        analyzer2.on_data(flow_key_c(), &frame_c, 7, Direction::ClientToServer);
        analyzer2.on_data(flow_key_b(), &frame_b, 8, Direction::ClientToServer);
        analyzer2.on_data(flow_key_b(), &frame_b, 9, Direction::ClientToServer);
        analyzer2.on_data(flow_key_b(), &frame_b, 10, Direction::ClientToServer);
        analyzer2.on_data(flow_key_a(), &frame_a, 11, Direction::ClientToServer);

        let summary1 = analyzer1.summarize();
        let summary2 = analyzer2.summarize();

        let counts1 = summary1
            .detail
            .get("control_operation_counts")
            .expect("control_operation_counts must be present");
        let counts2 = summary2
            .detail
            .get("control_operation_counts")
            .expect("control_operation_counts must be present");

        let json1 = serde_json::to_string(counts1).expect("serialize counts1");
        let json2 = serde_json::to_string(counts2).expect("serialize counts2");

        assert_eq!(
            json1, json2,
            "BC-2.15.020: control_operation_counts must be byte-identical regardless \
             of insertion order (v0.9.2 determinism fix); \
             analyzer1={json1}, analyzer2={json2}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.15.020 determinism: index→value follows FlowKey sort order
    // -----------------------------------------------------------------------

    /// Verifies that `control_operation_counts` index→value assignment follows
    /// ascending FlowKey order (flow_key_a → index "0", flow_key_b → index "1",
    /// flow_key_c → index "2").
    ///
    /// This pins the semantic contract: the lowest FlowKey always gets index "0",
    /// even if that flow was inserted last into the HashMap.
    ///
    /// Traces to: BC-2.15.020 postcondition 1; v0.9.2 determinism fix.
    #[test]
    fn test_BC_2_15_020_control_operation_counts_sorted_by_flow_key() {
        // Known counts per flow (distinct to make any permutation visible):
        //   flow_key_a (lowest sort) → 1 DIRECT_OPERATE
        //   flow_key_b              → 3 DIRECT_OPERATE
        //   flow_key_c (highest)    → 7 DIRECT_OPERATE
        let frame_a = build_frame(0x05, 0x0002, 0x0001);
        let frame_b = build_frame(0x05, 0x0003, 0x0001);
        let frame_c = build_frame(0x05, 0x0004, 0x0001);

        // Insert in reverse sort order to force non-trivial HashMap ordering.
        let mut analyzer = Dnp3Analyzer::new(100);
        analyzer.on_data(flow_key_c(), &frame_c, 1, Direction::ClientToServer);
        analyzer.on_data(flow_key_c(), &frame_c, 2, Direction::ClientToServer);
        analyzer.on_data(flow_key_c(), &frame_c, 3, Direction::ClientToServer);
        analyzer.on_data(flow_key_c(), &frame_c, 4, Direction::ClientToServer);
        analyzer.on_data(flow_key_c(), &frame_c, 5, Direction::ClientToServer);
        analyzer.on_data(flow_key_c(), &frame_c, 6, Direction::ClientToServer);
        analyzer.on_data(flow_key_c(), &frame_c, 7, Direction::ClientToServer);
        analyzer.on_data(flow_key_b(), &frame_b, 8, Direction::ClientToServer);
        analyzer.on_data(flow_key_b(), &frame_b, 9, Direction::ClientToServer);
        analyzer.on_data(flow_key_b(), &frame_b, 10, Direction::ClientToServer);
        analyzer.on_data(flow_key_a(), &frame_a, 11, Direction::ClientToServer);

        let summary = analyzer.summarize();

        let counts = summary
            .detail
            .get("control_operation_counts")
            .expect("control_operation_counts must be present")
            .as_object()
            .expect("control_operation_counts must be a JSON object");

        // Index "0" must map to flow_key_a's count (1).
        let count_0 = counts
            .get("0")
            .and_then(|v| v.as_u64())
            .expect("key '0' must be present and a u64");
        assert_eq!(
            count_0, 1,
            "BC-2.15.020: index '0' must map to flow_key_a (lowest FlowKey) \
             with direct_operate_count=1, got {count_0}"
        );

        // Index "1" must map to flow_key_b's count (3).
        let count_1 = counts
            .get("1")
            .and_then(|v| v.as_u64())
            .expect("key '1' must be present and a u64");
        assert_eq!(
            count_1, 3,
            "BC-2.15.020: index '1' must map to flow_key_b with direct_operate_count=3, \
             got {count_1}"
        );

        // Index "2" must map to flow_key_c's count (7).
        let count_2 = counts
            .get("2")
            .and_then(|v| v.as_u64())
            .expect("key '2' must be present and a u64");
        assert_eq!(
            count_2, 7,
            "BC-2.15.020: index '2' must map to flow_key_c (highest FlowKey) \
             with direct_operate_count=7, got {count_2}"
        );
    }
}
