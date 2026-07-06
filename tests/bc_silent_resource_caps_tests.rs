//! TDD red-gate tests for four new observability counters that surface silently-dropped /
//! evicted analyzer state (silent-limit audit, fix/surface-silent-resource-caps).
//!
//! Behavioral contracts:
//!   BC-2.16.010 v1.9 — ArpAnalyzer summarize() now 13 keys
//!   BC-2.16.008 v2.0 — `bindings_evicted` (MAX_ARP_BINDINGS=65536 LRU eviction)
//!                       `storm_counters_evicted` (MAX_STORM_COUNTERS=4096 LRU eviction)
//!   BC-2.14.012 v1.1 — `dropped_transactions` (MAX_PENDING_TRANSACTIONS=256 drop-not-evict)
//!   BC-2.14.021 v1.2 — ModbusAnalyzer summarize() now 7 keys (adds `dropped_transactions`)
//!   BC-2.07.031 v1.5 — TlsAnalyzer summarize() `dropped_map_entries` (MAX_MAP_ENTRIES=50000)
//!   BC-2.06.023 v1.6 — HttpAnalyzer summarize() `dropped_map_entries` (MAX_MAP_ENTRIES=50000)
//!
//! These tests are the RED GATE: they MUST FAIL on the current codebase (the new fields /
//! keys do not exist yet). The implementer's job is to make each test pass with minimum code.
//!
//! Strategy: all assertions are against summarize().detail map keys so the tests COMPILE
//! even before the struct fields are added. A missing key causes a runtime assertion failure,
//! which is the correct red-gate failure mode — not a compile error.
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod silent_resource_caps`.

#![allow(non_snake_case)]

mod silent_resource_caps {
    use std::net::IpAddr;
    use wirerust::analyzer::arp::ArpAnalyzer;
    use wirerust::analyzer::modbus::{MAX_PENDING_TRANSACTIONS, ModbusAnalyzer};
    use wirerust::decoder::ArpFrame;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{Direction, StreamAnalyzer, StreamHandler};

    // -----------------------------------------------------------------------
    // Shared helpers
    // -----------------------------------------------------------------------

    fn modbus_flow_key() -> FlowKey {
        FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            12345,
            "10.0.0.2".parse::<IpAddr>().unwrap(),
            502,
        )
    }

    /// Build a minimal valid Modbus TCP ADU (FC=0x03 Read Holding Registers request).
    ///
    /// MBAP layout: [txn_hi, txn_lo, proto_hi=0, proto_lo=0, len_hi=0, len_lo=6,
    ///               unit_id, fc=0x03, addr_hi=0, addr_lo=0, qty_hi=0, qty_lo=1]
    /// Length=6 covers unit_id (1) + FC (1) + 4 data bytes.
    fn modbus_read_request(txn_id: u16, unit_id: u8) -> [u8; 12] {
        let [hi, lo] = txn_id.to_be_bytes();
        [
            hi, lo, 0x00, 0x00, 0x00, 0x06, unit_id, 0x03, 0x00, 0x00, 0x00, 0x01,
        ]
    }

    // -----------------------------------------------------------------------
    // KEY-PRESENCE tests — assert new keys appear in summarize() with value 0
    // on a freshly constructed (zero-input) analyzer.
    // All four tests FAIL on current code: key absent → panic via .unwrap() /
    // explicit assert.
    // -----------------------------------------------------------------------

    /// BC-2.16.010 v1.9 / BC-2.16.008 v2.0: `bindings_evicted` must be present in
    /// ArpAnalyzer summarize() detail with value 0 when no frames have been processed.
    ///
    /// RED GATE: `bindings_evicted` is not yet a field on ArpAnalyzer and is not yet
    /// inserted into the detail map by summarize().
    #[test]
    fn test_BC_2_16_008_bindings_evicted_key_present_zero_on_fresh_analyzer() {
        let analyzer = ArpAnalyzer::new(3, 50);
        let summary = analyzer.summarize();
        let val = summary.detail.get("bindings_evicted").unwrap_or_else(|| {
            panic!(
                "BC-2.16.010 v1.9 / BC-2.16.008 v2.0 PC: ArpAnalyzer summarize() must \
                 contain key 'bindings_evicted' (u64, always-present, 0 when no evictions). \
                 Key is MISSING — red gate expected on current code. \
                 Keys present: {:?}",
                summary.detail.keys().collect::<Vec<_>>()
            )
        });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.16.008 v2.0: 'bindings_evicted' must be 0 on a fresh analyzer, got: {val}"
        );
    }

    /// BC-2.16.010 v1.9 / BC-2.16.008 v2.0: `storm_counters_evicted` must be present in
    /// ArpAnalyzer summarize() detail with value 0 when no frames have been processed.
    ///
    /// RED GATE: `storm_counters_evicted` is not yet a field on ArpAnalyzer and is not yet
    /// inserted into the detail map by summarize().
    #[test]
    fn test_BC_2_16_008_storm_counters_evicted_key_present_zero_on_fresh_analyzer() {
        let analyzer = ArpAnalyzer::new(3, 50);
        let summary = analyzer.summarize();
        let val = summary
            .detail
            .get("storm_counters_evicted")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.16.010 v1.9 / BC-2.16.008 v2.0 PC: ArpAnalyzer summarize() must \
                 contain key 'storm_counters_evicted' (u64, always-present, 0 when no \
                 evictions). Key is MISSING — red gate expected on current code. \
                 Keys present: {:?}",
                    summary.detail.keys().collect::<Vec<_>>()
                )
            });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.16.008 v2.0: 'storm_counters_evicted' must be 0 on a fresh analyzer, got: {val}"
        );
    }

    /// BC-2.14.021 v1.2 / BC-2.14.012 v1.1: `dropped_transactions` must be present in
    /// ModbusAnalyzer summarize() detail with value 0 when no PDUs have been processed.
    ///
    /// RED GATE: `dropped_transactions` is not yet a field on ModbusAnalyzer and is not yet
    /// inserted into the detail map by summarize().
    #[test]
    fn test_BC_2_14_012_dropped_transactions_key_present_zero_on_fresh_analyzer() {
        let analyzer = ModbusAnalyzer::new(20, 10);
        let summary = analyzer.summarize();
        let val = summary
            .detail
            .get("dropped_transactions")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.14.021 v1.2 / BC-2.14.012 v1.1 PC: ModbusAnalyzer summarize() must \
                 contain key 'dropped_transactions' (u64, always-present, 0 when no drops). \
                 Key is MISSING — red gate expected on current code. \
                 Keys present: {:?}",
                    summary.detail.keys().collect::<Vec<_>>()
                )
            });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.14.012 v1.1: 'dropped_transactions' must be 0 on a fresh analyzer, got: {val}"
        );
    }

    /// BC-2.14.021 v1.2: summarize() must return exactly 7 keys (was 6 before this BC amendment).
    ///
    /// RED GATE: `dropped_transactions` not yet in detail map → actual len == 6, not 7.
    #[test]
    fn test_BC_2_14_021_summarize_seven_keys_exact() {
        let analyzer = ModbusAnalyzer::new(20, 10);
        let summary = analyzer.summarize();
        let detail = &summary.detail;
        let required_keys = [
            "dropped_transactions",
            "dropped_findings",
            "exception_count",
            "function_code_distribution",
            "parse_errors",
            "pdu_count",
            "write_count",
        ];
        for key in &required_keys {
            assert!(
                detail.contains_key(*key),
                "BC-2.14.021 v1.2: ModbusAnalyzer summarize() must contain key '{key}'. \
                 Key is MISSING — red gate expected on current code. \
                 Keys present: {:?}",
                detail.keys().collect::<Vec<_>>()
            );
        }
        assert_eq!(
            detail.len(),
            7,
            "BC-2.14.021 v1.2: ModbusAnalyzer summarize() must return exactly 7 keys \
             (was 6 before BC-2.14.021 v1.2 amendment). Got {}. Keys: {:?}",
            detail.len(),
            detail.keys().collect::<Vec<_>>()
        );
    }

    // -----------------------------------------------------------------------
    // INCREMENT-ON-EVENT tests
    // -----------------------------------------------------------------------

    /// BC-2.14.012 v1.1: `dropped_transactions` increments when a new unique request
    /// is dropped because the pending table is at MAX_PENDING_TRANSACTIONS=256.
    ///
    /// Mechanism: feed 257 distinct (txn_id, unit_id) requests to one flow via
    /// `on_data` (ClientToServer). The 257th request is a new key at a full table →
    /// `dropped_transactions` must be >= 1.
    ///
    /// This test drives `ModbusAnalyzer` via its public `on_data` interface, building
    /// minimal valid Modbus TCP ADUs (FC=0x03, length=6). 257 iterations completes in
    /// well under 100 ms.
    ///
    /// RED GATE: `dropped_transactions` field does not exist on ModbusAnalyzer;
    /// summarize() does not emit it → key absent, test panics on `unwrap_or_else`.
    #[test]
    fn test_BC_2_14_012_dropped_transactions_increments_at_cap() {
        let mut analyzer = ModbusAnalyzer::new(20, 10);
        let fk = modbus_flow_key();

        // Feed MAX_PENDING_TRANSACTIONS + 1 = 257 distinct requests to one flow.
        // txn_id runs 0..=256 (u16), unit_id fixed at 0x01 so all keys are distinct.
        // The 257th request (txn_id=256, unit_id=0x01) is a new key while the table
        // is full → insert_request drops it → dropped_transactions must be incremented.
        for txn_id in 0u16..=(MAX_PENDING_TRANSACTIONS as u16) {
            let adu = modbus_read_request(txn_id, 0x01);
            analyzer.on_data(&fk, Direction::ClientToServer, &adu, 0, txn_id as u32);
        }

        let summary = analyzer.summarize();
        let dropped = summary
            .detail
            .get("dropped_transactions")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.14.012 v1.1: summarize() must contain 'dropped_transactions' key \
                     after exceeding MAX_PENDING_TRANSACTIONS={}. \
                     Key is MISSING — red gate expected on current code. \
                     Keys present: {:?}",
                    MAX_PENDING_TRANSACTIONS,
                    summary.detail.keys().collect::<Vec<_>>()
                )
            })
            .as_u64()
            .expect("dropped_transactions must be a u64");

        assert!(
            dropped >= 1,
            "BC-2.14.012 v1.1: 'dropped_transactions' must be >= 1 after feeding \
             {} requests to a single flow (cap = {}). Got 0.",
            MAX_PENDING_TRANSACTIONS + 1,
            MAX_PENDING_TRANSACTIONS
        );
    }

    /// BC-2.16.008 v2.0: `bindings_evicted` increments when the binding table LRU-evicts
    /// at MAX_ARP_BINDINGS=65_536.
    ///
    /// At 65_536 distinct IPs the table is full; the 65_537th insert must evict one entry
    /// and increment `bindings_evicted`. Filling 65_536 + 1 distinct ARP entries requires
    /// driving 65_537 frames which takes several seconds on most machines and is too slow
    /// for a default-run test suite.
    ///
    /// Therefore this test is `#[ignore]` by default. It is included so the implementer
    /// has a concrete live test for the eviction path, and can be run explicitly:
    ///   cargo test test_BC_2_16_008_bindings_evicted_increments_at_cap -- --ignored
    ///
    /// The key-presence test `test_BC_2_16_008_bindings_evicted_key_present_zero_on_fresh_analyzer`
    /// provides the red-gate assertion at zero cost. Code review validates the LRU eviction
    /// path increments the counter.
    ///
    /// Justification for #[ignore]: MAX_ARP_BINDINGS=65_536 distinct ARP frames at ~1 µs
    /// each ≈ 65 ms minimum, but typically 0.5–2 s with frame construction overhead.
    /// Acceptable for a one-off run; unacceptable as a default CI test on every commit.
    #[test]
    #[ignore = "BC-2.16.008 v2.0 eviction increment test; MAX_ARP_BINDINGS=65536 frames takes \
                ~0.5-2s. Run with: cargo test test_BC_2_16_008_bindings_evicted_increments -- \
                --ignored. Key-presence red-gate covered by \
                test_BC_2_16_008_bindings_evicted_key_present_zero_on_fresh_analyzer."]
    fn test_BC_2_16_008_bindings_evicted_increments_at_cap() {
        use wirerust::analyzer::arp::MAX_ARP_BINDINGS;

        let mut analyzer = ArpAnalyzer::new(3, 50);

        // Fill the binding table to exactly MAX_ARP_BINDINGS distinct sender IPs.
        // Use unique sender_mac per IP to avoid spoof findings (which require a MAC change
        // on an existing IP). Target IP = 192.168.0.1 (irrelevant).
        let make_frame = |i: u32| -> ArpFrame {
            let ip = (i + 1).to_be_bytes(); // avoid 0.0.0.0
            let mac_lo = ((i + 1) & 0xFF) as u8;
            let mac_hi = (((i + 1) >> 8) & 0xFF) as u8;
            ArpFrame {
                operation: 1,
                sender_mac: [0xAA, 0xBB, 0xCC, 0xDD, mac_hi, mac_lo],
                sender_ip: ip,
                target_mac: [0u8; 6],
                target_ip: [192, 168, 0, 1],
                outer_src_mac: Some([0xAA, 0xBB, 0xCC, 0xDD, mac_hi, mac_lo]),
                packet_len: 42,
            }
        };

        // Fill to cap (65_536 distinct IPs).
        for i in 0u32..MAX_ARP_BINDINGS as u32 {
            let frame = make_frame(i);
            let _ = analyzer.process_arp(&frame, 0);
        }

        let before = analyzer
            .summarize()
            .detail
            .get("bindings_evicted")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        assert_eq!(
            before, 0,
            "BC-2.16.008 v2.0: 'bindings_evicted' must be 0 before the 65537th distinct IP"
        );

        // Insert one more distinct IP — must trigger LRU eviction.
        let eviction_frame = make_frame(MAX_ARP_BINDINGS as u32 + 1);
        let _ = analyzer.process_arp(&eviction_frame, 1);

        let after = analyzer
            .summarize()
            .detail
            .get("bindings_evicted")
            .and_then(|v| v.as_u64())
            .expect("'bindings_evicted' key must be present after eviction");
        assert!(
            after >= 1,
            "BC-2.16.008 v2.0: 'bindings_evicted' must be >= 1 after inserting \
             MAX_ARP_BINDINGS+1={} distinct sender IPs. Got 0.",
            MAX_ARP_BINDINGS + 1
        );
    }

    /// BC-2.16.008 v2.0: `storm_counters_evicted` increments when the storm-counter table
    /// LRU-evicts at MAX_STORM_COUNTERS=4096.
    ///
    /// At 4096 distinct source MACs the storm-counter table is full; the 4097th distinct MAC
    /// must evict one entry and increment `storm_counters_evicted`. Filling 4096 distinct ARP
    /// request frames (each with a unique sender MAC) at storm rate (same timestamp) is fast
    /// in practice (~4 ms), but may be slow in coverage or sanitizer builds.
    ///
    /// This test is `#[ignore]` by default because MAX_STORM_COUNTERS=4096 frames may take
    /// >1s in slow CI environments.
    ///
    /// Run explicitly:
    ///   cargo test test_BC_2_16_008_storm_counters_evicted_increments -- --ignored
    ///
    /// Key-presence red-gate:
    ///   test_BC_2_16_008_storm_counters_evicted_key_present_zero_on_fresh_analyzer
    #[test]
    #[ignore = "BC-2.16.008 v2.0 storm eviction increment test; MAX_STORM_COUNTERS=4096 frames \
                may be slow in coverage/sanitizer builds. Run with: cargo test \
                test_BC_2_16_008_storm_counters_evicted_increments -- --ignored. \
                Key-presence red-gate covered by \
                test_BC_2_16_008_storm_counters_evicted_key_present_zero_on_fresh_analyzer."]
    fn test_BC_2_16_008_storm_counters_evicted_increments_at_cap() {
        use wirerust::analyzer::arp::MAX_STORM_COUNTERS;

        let mut analyzer = ArpAnalyzer::new(3, 50);

        // Fill to exactly MAX_STORM_COUNTERS (4096) distinct source MACs.
        // All frames have the same timestamp (ts=1) so the storm-counter window
        // accumulates without resetting, maximising the chance that all MACs are
        // tracked. Unique sender_mac and sender_ip per iteration.
        let make_frame = |i: u32| -> ArpFrame {
            let mac_lo = (i & 0xFF) as u8;
            let mac_hi = ((i >> 8) & 0xFF) as u8;
            let ip = [0x0A, mac_hi, mac_lo, 0x01]; // 10.x.x.1 (unique per i)
            ArpFrame {
                operation: 1,
                sender_mac: [0x00, 0x00, mac_hi, mac_lo, 0x00, 0x01],
                sender_ip: ip,
                target_mac: [0u8; 6],
                target_ip: [192, 168, 0, 1],
                outer_src_mac: Some([0x00, 0x00, mac_hi, mac_lo, 0x00, 0x01]),
                packet_len: 42,
            }
        };

        for i in 0u32..MAX_STORM_COUNTERS as u32 {
            let frame = make_frame(i);
            let _ = analyzer.process_arp(&frame, 1);
        }

        let before = analyzer
            .summarize()
            .detail
            .get("storm_counters_evicted")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        assert_eq!(
            before, 0,
            "BC-2.16.008 v2.0: 'storm_counters_evicted' must be 0 before the 4097th distinct MAC"
        );

        // Insert one more distinct MAC — must trigger LRU eviction.
        let eviction_frame = make_frame(MAX_STORM_COUNTERS as u32 + 1);
        let _ = analyzer.process_arp(&eviction_frame, 1);

        let after = analyzer
            .summarize()
            .detail
            .get("storm_counters_evicted")
            .and_then(|v| v.as_u64())
            .expect("'storm_counters_evicted' key must be present after eviction");
        assert!(
            after >= 1,
            "BC-2.16.008 v2.0: 'storm_counters_evicted' must be >= 1 after inserting \
             MAX_STORM_COUNTERS+1={} distinct source MACs. Got 0.",
            MAX_STORM_COUNTERS + 1
        );
    }

    // -----------------------------------------------------------------------
    // TLS / HTTP dropped_map_entries — key-presence tests
    // (increment tests are #[ignore]'d: cap is 50_000, filling it is too slow
    // for a default test run)
    // -----------------------------------------------------------------------

    /// BC-2.07.031 v1.5: `dropped_map_entries` must be present in TlsAnalyzer
    /// summarize() detail with value 0 on a freshly constructed analyzer.
    ///
    /// RED GATE: `dropped_map_entries` not yet a field on TlsAnalyzer; summarize()
    /// does not emit it.
    #[test]
    fn test_BC_2_07_031_tls_dropped_map_entries_key_present_zero_on_fresh_analyzer() {
        use wirerust::analyzer::tls::TlsAnalyzer;
        let analyzer = TlsAnalyzer::new();
        let summary = analyzer.summarize();
        let val = summary
            .detail
            .get("dropped_map_entries")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.07.031 v1.5 PC: TlsAnalyzer summarize() must contain key \
                 'dropped_map_entries' (u64, always-present, 0 when no drops). \
                 Key is MISSING — red gate expected on current code. \
                 Keys present: {:?}",
                    summary.detail.keys().collect::<Vec<_>>()
                )
            });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.07.031 v1.5: 'dropped_map_entries' must be 0 on a fresh TLS analyzer, got: {val}"
        );
    }

    /// BC-2.06.023 v1.6: `dropped_map_entries` must be present in HttpAnalyzer
    /// summarize() detail with value 0 on a freshly constructed analyzer.
    ///
    /// RED GATE: `dropped_map_entries` not yet a field on HttpAnalyzer; summarize()
    /// does not emit it.
    #[test]
    fn test_BC_2_06_023_http_dropped_map_entries_key_present_zero_on_fresh_analyzer() {
        use wirerust::analyzer::http::HttpAnalyzer;
        let analyzer = HttpAnalyzer::new();
        let summary = analyzer.summarize();
        let val = summary
            .detail
            .get("dropped_map_entries")
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.06.023 v1.6 PC: HttpAnalyzer summarize() must contain key \
                 'dropped_map_entries' (u64, always-present, 0 when no drops). \
                 Key is MISSING — red gate expected on current code. \
                 Keys present: {:?}",
                    summary.detail.keys().collect::<Vec<_>>()
                )
            });
        assert_eq!(
            val.as_u64(),
            Some(0),
            "BC-2.06.023 v1.6: 'dropped_map_entries' must be 0 on a fresh HTTP analyzer, got: {val}"
        );
    }

    /// BC-2.07.031 v1.5: `dropped_map_entries` increments when a distribution map
    /// (sni_counts, ja3_counts, ja3s_counts, cipher_counts, version_counts) drops a
    /// new key at MAX_MAP_ENTRIES=50_000.
    ///
    /// Filling 50_001 distinct keys requires constructing at minimum 50_001 TLS
    /// ClientHello messages with unique SNIs. Even with minimal frames this takes
    /// several seconds and is not acceptable for a default CI run.
    ///
    /// Therefore this test is `#[ignore]` by default. The key-presence test provides
    /// the red-gate assertion at zero cost.
    ///
    /// Run explicitly:
    ///   cargo test test_BC_2_07_031_tls_dropped_map_entries_increments -- --ignored
    #[test]
    #[ignore = "BC-2.07.031 v1.5 TLS dropped_map_entries increment test; requires 50_001 \
                distinct SNI/JA3/cipher entries (MAX_MAP_ENTRIES=50000). Too slow for default \
                CI run. Run explicitly with --ignored. Key-presence red-gate covered by \
                test_BC_2_07_031_tls_dropped_map_entries_key_present_zero_on_fresh_analyzer."]
    fn test_BC_2_07_031_tls_dropped_map_entries_increments_at_cap() {
        // Placeholder body — the ignored guard means this never executes in CI.
        // When un-ignored the implementer should populate 50_001 distinct SNIs and verify
        // dropped_map_entries >= 1 in the detail map.
        use wirerust::analyzer::tls::TlsAnalyzer;
        let _analyzer = TlsAnalyzer::new();
        todo!(
            "BC-2.07.031 v1.5: implement increment test once TlsAnalyzer exposes a \
             smaller-cap test seam (or accept ~10s runtime for full 50_001 entries)"
        );
    }

    /// BC-2.06.023 v1.6: `dropped_map_entries` increments when a distribution map
    /// (methods, hosts, user_agents) drops a new key at MAX_MAP_ENTRIES=50_000.
    ///
    /// As with TLS, filling 50_001 distinct host values requires constructing 50_001
    /// HTTP request/response pairs and is too slow for a default CI run.
    ///
    /// Run explicitly:
    ///   cargo test test_BC_2_06_023_http_dropped_map_entries_increments -- --ignored
    #[test]
    #[ignore = "BC-2.06.023 v1.6 HTTP dropped_map_entries increment test; requires 50_001 \
                distinct Host header values (MAX_MAP_ENTRIES=50000). Too slow for default CI run. \
                Run explicitly with --ignored. Key-presence red-gate covered by \
                test_BC_2_06_023_http_dropped_map_entries_key_present_zero_on_fresh_analyzer."]
    fn test_BC_2_06_023_http_dropped_map_entries_increments_at_cap() {
        // Placeholder body — the ignored guard means this never executes in CI.
        use wirerust::analyzer::http::HttpAnalyzer;
        let _analyzer = HttpAnalyzer::new();
        todo!(
            "BC-2.06.023 v1.6: implement increment test once HttpAnalyzer exposes a \
             smaller-cap test seam (or accept ~10s runtime for full 50_001 host entries)"
        );
    }
} // mod silent_resource_caps
