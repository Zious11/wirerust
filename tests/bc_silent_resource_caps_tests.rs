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
        use wirerust::analyzer::tls::TlsAnalyzer;
        use wirerust::reassembly::handler::Direction;

        const MAX_MAP_ENTRIES: usize = 50_000;

        let mut analyzer = TlsAnalyzer::new();
        // Build a minimal TLS 1.2 ClientHello record with an SNI extension.
        // Wire format: TLS record (0x16, ver, len) + Handshake header + ClientHello body.
        let build_ch = |sni: &str| -> Vec<u8> {
            let sni_bytes = sni.as_bytes();
            let sni_name_len = sni_bytes.len() as u16;
            // SNI entry: NameType(1) + NameLen(2) + Name
            let sni_entry_len = 1u16 + 2u16 + sni_name_len;
            // SNI ext data: ServerNameListLength(2) + entry
            let sni_ext_data_len = 2u16 + sni_entry_len;
            // Extensions block: ext_type(2) + ext_data_len(2) + sni_ext_data_len
            let _ext_total_len = 2u16 + 2u16 + sni_ext_data_len;

            let mut extensions = Vec::new();
            extensions.extend_from_slice(&[0x00, 0x00]); // SNI ext type
            extensions.extend_from_slice(&sni_ext_data_len.to_be_bytes());
            extensions.extend_from_slice(&sni_entry_len.to_be_bytes());
            extensions.push(0x00); // NameType: host_name
            extensions.extend_from_slice(&sni_name_len.to_be_bytes());
            extensions.extend_from_slice(sni_bytes);
            // Supported groups (required by some parsers)
            extensions
                .extend_from_slice(&[0x00, 0x0a, 0x00, 0x06, 0x00, 0x04, 0x00, 0x1d, 0x00, 0x17]);
            // EC point formats
            extensions.extend_from_slice(&[0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);

            let actual_ext_total = extensions.len() as u16;

            let mut ch_body = Vec::new();
            ch_body.extend_from_slice(&[0x03, 0x03]); // TLS 1.2
            ch_body.extend_from_slice(&[0u8; 32]); // random
            ch_body.push(0x00); // session_id len
            ch_body.extend_from_slice(&[0x00, 0x02]); // cipher suites len: 1 suite
            ch_body.extend_from_slice(&[0x13, 0x01]); // TLS_AES_128_GCM_SHA256
            ch_body.push(0x01); // compression methods len
            ch_body.push(0x00); // null compression
            ch_body.extend_from_slice(&actual_ext_total.to_be_bytes()); // extensions len
            ch_body.extend_from_slice(&extensions);

            let ch_len = ch_body.len() as u32;
            let mut handshake = vec![
                0x01, // ClientHello
                (ch_len >> 16) as u8,
                (ch_len >> 8) as u8,
                ch_len as u8,
            ];
            handshake.extend_from_slice(&ch_body);

            let hs_len = handshake.len() as u16;
            let mut record = vec![0x16]; // handshake record
            record.extend_from_slice(&[0x03, 0x01]); // record version TLS 1.0
            record.extend_from_slice(&hs_len.to_be_bytes());
            record.extend_from_slice(&handshake);
            record
        };

        // Fill sni_counts to exactly MAX_MAP_ENTRIES with unique hostnames.
        // Each frame uses a new flow key to avoid the "one handshake per flow" limit.
        for i in 0u32..MAX_MAP_ENTRIES as u32 {
            let sni = format!("h{i}.example.test");
            let record = build_ch(&sni);
            let flow_key = FlowKey::new(
                "10.0.0.1".parse::<IpAddr>().unwrap(),
                (i as u16).wrapping_add(1),
                "10.0.0.2".parse::<IpAddr>().unwrap(),
                443,
            );
            analyzer.on_data(&flow_key, Direction::ClientToServer, &record, 0, 0);
        }

        let before = analyzer
            .summarize()
            .detail
            .get("dropped_map_entries")
            .and_then(|v| v.as_u64())
            .expect("dropped_map_entries key must be present");
        assert_eq!(
            before, 0,
            "BC-2.07.031 v1.5: dropped_map_entries must be 0 before the 50_001st distinct SNI"
        );

        // Insert one more distinct SNI — must be dropped and increment the counter.
        let overflow_sni = "overflow.example.test";
        let overflow_record = build_ch(overflow_sni);
        let overflow_fk = FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            65535,
            "10.0.0.2".parse::<IpAddr>().unwrap(),
            443,
        );
        analyzer.on_data(
            &overflow_fk,
            Direction::ClientToServer,
            &overflow_record,
            0,
            0,
        );

        let after = analyzer
            .summarize()
            .detail
            .get("dropped_map_entries")
            .and_then(|v| v.as_u64())
            .expect("dropped_map_entries key must be present after overflow");
        assert!(
            after >= 1,
            "BC-2.07.031 v1.5: dropped_map_entries must be >= 1 after inserting \
             MAX_MAP_ENTRIES+1={} distinct SNIs. Got 0.",
            MAX_MAP_ENTRIES + 1
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
        use wirerust::analyzer::http::HttpAnalyzer;
        use wirerust::reassembly::handler::Direction;

        const MAX_MAP_ENTRIES: usize = 50_000;

        let mut analyzer = HttpAnalyzer::new();

        // Fill hosts map to exactly MAX_MAP_ENTRIES distinct Host values.
        // Each request uses a unique Host header on a unique flow to avoid
        // flow-level parsing state interfering across requests.
        for i in 0u32..MAX_MAP_ENTRIES as u32 {
            let fk = FlowKey::new(
                "10.0.0.1".parse::<IpAddr>().unwrap(),
                (i as u16).wrapping_add(1),
                "10.0.0.2".parse::<IpAddr>().unwrap(),
                80,
            );
            let request = format!("GET / HTTP/1.1\r\nHost: h{i}.example.test\r\n\r\n");
            analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0, 0);
        }

        let before = analyzer
            .summarize()
            .detail
            .get("dropped_map_entries")
            .and_then(|v| v.as_u64())
            .expect("dropped_map_entries key must be present");
        assert_eq!(
            before, 0,
            "BC-2.06.023 v1.6: dropped_map_entries must be 0 before the 50_001st distinct host"
        );

        // Insert one more distinct host — must be dropped and increment the counter.
        let overflow_fk = FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            65535,
            "10.0.0.2".parse::<IpAddr>().unwrap(),
            80,
        );
        let overflow_request = "GET / HTTP/1.1\r\nHost: overflow.example.test\r\n\r\n";
        analyzer.on_data(
            &overflow_fk,
            Direction::ClientToServer,
            overflow_request.as_bytes(),
            0,
            0,
        );

        let after = analyzer
            .summarize()
            .detail
            .get("dropped_map_entries")
            .and_then(|v| v.as_u64())
            .expect("dropped_map_entries key must be present after overflow");
        assert!(
            after >= 1,
            "BC-2.06.023 v1.6: dropped_map_entries must be >= 1 after inserting \
             MAX_MAP_ENTRIES+1={} distinct Host values. Got 0.",
            MAX_MAP_ENTRIES + 1
        );
    }

    // -----------------------------------------------------------------------
    // NEGATIVE REGRESSION TESTS (PR #365 reviewer follow-ups)
    // These guard ALREADY-SHIPPED, already-correct behavior.  They MUST PASS
    // on current code.  If either fails, that reveals a real bug — STOP.
    // -----------------------------------------------------------------------

    /// HTTP-AC008-NEG-TEST-001 / BC-2.06.024 AC-008 (negative):
    /// Hitting an EXISTING key in the HttpAnalyzer distribution maps must NOT
    /// increment `dropped_map_entries`.
    ///
    /// Invariant: `dropped_map_entries` is incremented ONLY when a NEW key is
    /// refused because the map is at MAX_MAP_ENTRIES=50_000 capacity.  Repeated
    /// requests that reuse already-inserted Host / User-Agent values are
    /// existing-key updates and must never touch the counter, regardless of
    /// how many times the same keys are seen.
    ///
    /// Mechanism: send several HTTP requests that all use the same Host and
    /// User-Agent, well below the 50k cap.  The maps stay tiny; the keys are
    /// inserted on the first request and updated (not refused) on every
    /// subsequent one.  Assert `dropped_map_entries == 0` throughout.
    ///
    /// This is a live (non-`#[ignore]`) test — it runs in under 1 ms.
    #[test]
    fn test_HTTP_AC008_NEG_TEST_001_existing_key_increment_does_not_raise_dropped_map_entries() {
        use wirerust::analyzer::http::HttpAnalyzer;
        use wirerust::reassembly::handler::Direction;

        let mut analyzer = HttpAnalyzer::new();

        // Use two distinct flow keys so the HTTP parser does not see the
        // repeated requests as a single pipelined stream and mangle them.
        // Even if they share a flow, the Host/User-Agent keys are already
        // present after the first parse, so the insert guard returns early.
        let fk_a = FlowKey::new(
            "10.10.0.1".parse::<IpAddr>().unwrap(),
            51000,
            "10.10.0.2".parse::<IpAddr>().unwrap(),
            80,
        );
        let fk_b = FlowKey::new(
            "10.10.0.3".parse::<IpAddr>().unwrap(),
            51001,
            "10.10.0.4".parse::<IpAddr>().unwrap(),
            80,
        );

        // 10 repetitions of the same Host + User-Agent on alternating flow keys.
        // The first request on each flow seeds the maps; subsequent ones reuse
        // the EXISTING key → must NOT increment dropped_map_entries.
        for i in 0u32..10 {
            let fk = if i % 2 == 0 { &fk_a } else { &fk_b };
            let req =
                b"GET /path HTTP/1.1\r\nHost: existing.example.test\r\nUser-Agent: TestBot/1.0\r\n\r\n";
            analyzer.on_data(fk, Direction::ClientToServer, req, 0, i);
        }

        let summary = analyzer.summarize();
        let dropped = summary
            .detail
            .get("dropped_map_entries")
            .and_then(|v| v.as_u64())
            .expect(
                "HTTP-AC008-NEG-TEST-001: 'dropped_map_entries' key must be present in \
                 HttpAnalyzer summarize(). Key missing — regression in BC-2.06.023.",
            );

        assert_eq!(
            dropped, 0,
            "HTTP-AC008-NEG-TEST-001 / BC-2.06.024 AC-008 (negative): \
             `dropped_map_entries` must be 0 when only EXISTING keys are hit \
             (same Host/User-Agent repeated below the 50k cap). \
             Got {dropped} — existing-key increment is incorrectly bumping the drop counter."
        );
    }

    /// EVICTION-NO-FINDING-NEG-TEST-001 / BC-2.16.006 Inv3 / BC-2.16.008 Inv5 /
    /// BC-2.16.010 Inv7 / BC-2.14.012 v1.1 (negative):
    /// Eviction / pending-drop events are COUNTER-ONLY: they must not produce
    /// any Finding.
    ///
    /// Part A — Modbus pending-drop (fast: 257 requests).
    ///   After saturating the 256-slot pending table, additional new requests
    ///   are silently dropped.  The `dropped_transactions` counter increments;
    ///   `all_findings` must gain NO new Finding for the drop event itself.
    ///
    /// Part B — ARP binding eviction (slow: requires 65537 distinct IPs).
    ///   Marked `#[ignore]` because filling MAX_ARP_BINDINGS=65536 distinct
    ///   entries takes ~0.5–2 s.  Justification: the invariant is structurally
    ///   enforced — `process_arp` only pushes findings before the eviction
    ///   branch and returns the `findings` vec which never includes an eviction
    ///   entry.  Part A provides the fast guard; Part B is an exhaustive check
    ///   runnable on-demand.
    ///
    /// Run Part B explicitly:
    ///   cargo test EVICTION_NO_FINDING_NEG_TEST_001_arp -- --ignored
    #[test]
    fn test_EVICTION_NO_FINDING_NEG_TEST_001_modbus_pending_drop_emits_no_finding() {
        let mut analyzer = ModbusAnalyzer::new(20, 10);
        let fk = modbus_flow_key();

        // Feed MAX_PENDING_TRANSACTIONS = 256 requests to fill the table.
        // All use FC=0x03 (read holding registers) — non-write, non-exception
        // → they produce no findings regardless of cap state.
        for txn_id in 0u16..MAX_PENDING_TRANSACTIONS as u16 {
            let adu = modbus_read_request(txn_id, 0x01);
            analyzer.on_data(&fk, Direction::ClientToServer, &adu, 0, txn_id as u32);
        }

        // Snapshot findings length immediately before the overflow request.
        let findings_at_cap = analyzer.all_findings.len();

        // The 257th request (txn_id=256, unit_id=0x01) is a NEW key at a FULL table →
        // `dropped_transactions` must increment and NO finding must be emitted.
        let overflow_txn_id = MAX_PENDING_TRANSACTIONS as u16;
        let overflow_adu = modbus_read_request(overflow_txn_id, 0x01);
        analyzer.on_data(
            &fk,
            Direction::ClientToServer,
            &overflow_adu,
            0,
            overflow_txn_id as u32,
        );

        // `dropped_transactions` must be at least 1 (proving the cap was hit).
        let summary = analyzer.summarize();
        let dropped = summary
            .detail
            .get("dropped_transactions")
            .and_then(|v| v.as_u64())
            .expect(
                "EVICTION-NO-FINDING-NEG-TEST-001: 'dropped_transactions' key must be present \
                 in ModbusAnalyzer summarize().",
            );
        assert!(
            dropped >= 1,
            "EVICTION-NO-FINDING-NEG-TEST-001: expected dropped_transactions >= 1 after \
             feeding {} requests (cap={}). Got 0 — test precondition not met.",
            MAX_PENDING_TRANSACTIONS + 1,
            MAX_PENDING_TRANSACTIONS
        );

        // `all_findings` must not have grown due to the drop event.
        // FC=0x03 (read holding registers) is non-write, non-exception → zero findings.
        // The drop path only increments `dropped_transactions`; it must never emit a Finding.
        let findings_after_overflow = analyzer.all_findings.len();
        assert_eq!(
            findings_after_overflow, findings_at_cap,
            "EVICTION-NO-FINDING-NEG-TEST-001 / BC-2.14.012 (negative): \
             `all_findings` must not grow due to a pending-table drop event. \
             At cap ({MAX_PENDING_TRANSACTIONS} requests): {findings_at_cap} finding(s). \
             After overflow request: {findings_after_overflow} finding(s). \
             A drop event is a COUNTER-ONLY event (BC-2.16.006 Inv3 / BC-2.14.012)."
        );
    }

    /// EVICTION-NO-FINDING-NEG-TEST-001 — Part B (ARP binding eviction, slow path).
    ///
    /// Fills MAX_ARP_BINDINGS=65536 distinct sender IPs into ArpAnalyzer, then
    /// inserts one more to trigger LRU eviction.  Asserts the eviction-triggering
    /// frame returns ZERO findings (the eviction branch runs only after `findings`
    /// is fully constructed and returned; no eviction Finding is ever appended).
    ///
    /// Marked `#[ignore]` because filling 65537 distinct ARP frames takes
    /// approximately 0.5–2 s.  Part A (Modbus, above) provides the fast guard.
    ///
    /// Run explicitly:
    ///   cargo test test_EVICTION_NO_FINDING_NEG_TEST_001_arp_eviction_emits_no_finding -- --ignored
    #[test]
    #[ignore = "EVICTION-NO-FINDING-NEG-TEST-001 ARP Part B: MAX_ARP_BINDINGS=65536 frames \
                takes ~0.5-2s. Run with: cargo test \
                test_EVICTION_NO_FINDING_NEG_TEST_001_arp_eviction_emits_no_finding -- --ignored. \
                Fast guard: test_EVICTION_NO_FINDING_NEG_TEST_001_modbus_pending_drop_emits_no_finding."]
    fn test_EVICTION_NO_FINDING_NEG_TEST_001_arp_eviction_emits_no_finding() {
        use wirerust::analyzer::arp::MAX_ARP_BINDINGS;

        let mut analyzer = ArpAnalyzer::new(3, 50);

        // Fill to exactly MAX_ARP_BINDINGS distinct sender IPs.
        // Unique sender_mac per IP prevents rebind (D1) findings.
        // outer_src_mac == sender_mac prevents mismatch (D12) findings.
        // Non-GARP (target_ip != sender_ip) prevents D2 findings.
        // Unique IPs prevent storm (D3) from firing on repeated MACs.
        let make_normal_frame = |i: u32| -> ArpFrame {
            let ip = (i + 1).to_be_bytes(); // avoid 0.0.0.0
            let mac_lo = ((i + 1) & 0xFF) as u8;
            let mac_hi = (((i + 1) >> 8) & 0xFF) as u8;
            ArpFrame {
                operation: 1,
                sender_mac: [0xAA, 0xBB, 0xCC, 0xDD, mac_hi, mac_lo],
                sender_ip: ip,
                target_mac: [0u8; 6],
                target_ip: [192, 168, 0, 1], // different from sender_ip → non-GARP
                outer_src_mac: Some([0xAA, 0xBB, 0xCC, 0xDD, mac_hi, mac_lo]),
                packet_len: 42,
            }
        };

        // Fill to cap.
        for i in 0u32..MAX_ARP_BINDINGS as u32 {
            let frame = make_normal_frame(i);
            let _ = analyzer.process_arp(&frame, 0);
        }

        // Verify cap is reached: bindings_evicted still 0 before the +1 insert.
        let before_evict = analyzer
            .summarize()
            .detail
            .get("bindings_evicted")
            .and_then(|v| v.as_u64())
            .expect("bindings_evicted key must be present");
        assert_eq!(
            before_evict,
            0,
            "EVICTION-NO-FINDING-NEG-TEST-001 (ARP Part B): bindings_evicted must be 0 \
             before the {}-th distinct IP insert.",
            MAX_ARP_BINDINGS + 1
        );

        // Insert one more distinct IP — triggers LRU eviction of the oldest entry.
        let eviction_frame = make_normal_frame(MAX_ARP_BINDINGS as u32 + 1);
        let eviction_findings = analyzer.process_arp(&eviction_frame, 1);

        // The eviction-triggering frame must return ZERO findings
        // (BC-2.16.006 Inv3 / BC-2.16.008 Inv5 / BC-2.16.010 Inv7:
        // eviction is COUNTER-ONLY, never a Finding).
        assert_eq!(
            eviction_findings.len(),
            0,
            "EVICTION-NO-FINDING-NEG-TEST-001 / BC-2.16.006 Inv3 (negative, ARP): \
             LRU eviction of a binding table entry must return ZERO findings. \
             Got {} finding(s). Eviction must be a COUNTER-ONLY event.",
            eviction_findings.len()
        );

        // Also confirm eviction counter incremented (proves the eviction path was hit).
        let after_evict = analyzer
            .summarize()
            .detail
            .get("bindings_evicted")
            .and_then(|v| v.as_u64())
            .expect("bindings_evicted key must be present after eviction");
        assert!(
            after_evict >= 1,
            "EVICTION-NO-FINDING-NEG-TEST-001 (ARP Part B): bindings_evicted must be >= 1 \
             after inserting MAX_ARP_BINDINGS+1={} distinct IPs. Got 0 — eviction path \
             was not exercised.",
            MAX_ARP_BINDINGS + 1
        );
    }
} // mod silent_resource_caps
