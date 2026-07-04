//! F6 fuzz harness for the feature-protocol-coverage (E-21) delta.
//!
//! Exercises the delta's library-visible classification / coverage-gap
//! accumulation surfaces over unbounded attacker-controlled bytes, as an
//! independent dynamic cross-check of the VP-041 / VP-042 / VP-043 harnesses:
//!
//!   1. `protocols::{all,supported,unsupported}_protocols()` (VP-041) — the pure
//!      partition over the constant `KNOWN_PROTOCOLS` catalog. Must never panic;
//!      the completeness invariant `|sup| + |unsup| == |all|` is asserted as a
//!      fuzz oracle (also validated under the ASan/UBSan-instrumented build).
//!
//!   2. `StreamDispatcher` coverage-gap accumulation (VP-042) — the fuzzer derives
//!      arbitrary `(port_a, port_b)` flow keys and payload chunks from the input
//!      and drives `on_data` + `on_flow_close` on a dispatcher built with an HTTP
//!      analyzer and `.with_coverage_gaps(true)` (the dual-gate precondition), so
//!      the None-target `unclassified_port_counts` increment (saturating_add) and
//!      the min-port key normalization run over unbounded input. A panic anywhere
//!      is a finding.
//!
//!   3. `udp_gap_key` (VP-043) — the pure UDP gap-key seam is called on a
//!      constructed UDP `ParsedPacket` for every derived port pair, with the DNS
//!      gate toggled. The gate + key + transport oracle mirrors the VP-043 Kani
//!      proof and is asserted for every input.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_coverage_gap_classify -- -max_total_time=300

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::net::{IpAddr, Ipv4Addr};
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::dispatcher::{StreamDispatcher, TransportProto, udp_gap_key};
use wirerust::protocols::{all_protocols, supported_protocols, unsupported_protocols};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};

fuzz_target!(|data: &[u8]| {
    // --- VP-041: partition functions never panic; completeness holds. ---
    let n_all = all_protocols().len();
    let n_sup = supported_protocols().len();
    let n_unsup = unsupported_protocols().len();
    assert_eq!(
        n_sup + n_unsup,
        n_all,
        "VP-041: |supported| + |unsupported| must equal |KNOWN_PROTOCOLS|"
    );

    // --- VP-042: coverage-gap accumulation over arbitrary ports/segments. ---
    let mut dispatcher =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None)
            .with_coverage_gaps(true);
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 9));

    // Consume the input as a sequence of (port_a, port_b, payload-len, payload) ops.
    let mut i = 0usize;
    let mut ts: u32 = 0;
    while i + 4 <= data.len() {
        let port_a = u16::from_le_bytes([data[i], data[i + 1]]);
        let port_b = u16::from_le_bytes([data[i + 2], data[i + 3]]);
        i += 4;
        let key = FlowKey::new(ip_a, port_a, ip_b, port_b);

        // Optional payload chunk from remaining bytes. Clamp the cursor to the
        // slice length before every index so the harness itself never panics
        // (start <= end <= len is maintained explicitly).
        let plen = data.get(i).copied().unwrap_or(0) as usize;
        let start = i.saturating_add(1).min(data.len());
        let end = start.saturating_add(plen).min(data.len());
        let payload = &data[start..end];
        i = end;

        let dir = if ts & 1 == 0 {
            Direction::ClientToServer
        } else {
            Direction::ServerToClient
        };
        dispatcher.on_data(&key, dir, payload, 0, ts);
        dispatcher.on_flow_close(&key, CloseReason::Fin);
        ts = ts.wrapping_add(1);

        // --- VP-043: udp_gap_key over the same arbitrary ports, both gates. ---
        let udp = ParsedPacket {
            src_ip: ip_a,
            dst_ip: ip_b,
            protocol: Protocol::Udp,
            transport: TransportInfo::Udp {
                src_port: port_a,
                dst_port: port_b,
            },
            payload: Vec::new(),
            packet_len: 0,
        };
        let dns_handles = (port_a & 1) == 1;
        match udp_gap_key(&udp, dns_handles) {
            Some((proto, gap_port)) => {
                assert!(!dns_handles, "VP-043: DNS-accepted UDP must yield None");
                assert!(matches!(proto, TransportProto::Udp));
                assert_eq!(gap_port, port_a.min(port_b), "VP-043: key = min(src,dst)");
            }
            None => assert!(dns_handles, "VP-043: unhandled UDP must yield Some(..)"),
        }
    }

    // Map access must not panic; total is bounded.
    let _total: u64 = dispatcher.unclassified_port_counts().values().sum();
});
