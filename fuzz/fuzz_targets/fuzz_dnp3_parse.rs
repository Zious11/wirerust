//! VP-023 fuzz harness: DNP3 parse / carry-walk path never panics on arbitrary input.
//!
//! The DNP3 analyzer consumes attacker-controlled pcap bytes via two surfaces:
//!
//!   1. `parse_dnp3_dl_header(&[u8])` — the pure-core fixed-offset DNP3 data-link
//!      header decoder. VP-023 (`verify_parse_dnp3_dl_header_safety`) proves
//!      panic/OOB freedom under Kani for bounded slices; this fuzz target
//!      exercises the SAME function over arbitrary-length attacker bytes (no
//!      length bound) as an independent, unbounded cross-check of the Kani proof.
//!
//!   2. `Dnp3Analyzer::on_data(flow_key, data, ts)` — the effectful per-flow shell
//!      (ADR-007 Decision 2; BC-2.15.016). This path includes:
//!        - the desync latch / sync-word `[0x05, 0x64]` bail (BC-2.15.009),
//!        - the 292-byte carry buffer with overflow discard (AC-001/EC-003),
//!        - the `while`-loop frame-walk consuming complete frames from the head
//!          of `flow.carry` (EC-002),
//!        - the gate-before-count validity gate (SEC-106-001; BC-2.15.004),
//!        - the FIR=1 + user-data application-FC extraction (BC-2.15.008),
//!        - the 300s correlation-window expiry / block-timeout scans
//!          (BC-2.15.014/015), and
//!        - the full detection engine (master-IP resolution, unexpected-source
//!          split, malformed-anomaly, direct-operate threshold).
//!      Kani cannot drive `on_data` directly (its per-flow `HashMap` state uses
//!      `RandomState`, an FFI the model checker cannot symbolically execute), so
//!      fuzzing is the primary dynamic safety check for this shell.
//!
//! The fuzzer splits the input into TWO chunks and feeds them as two successive
//! `on_data` calls on the SAME flow, deliberately exercising the cross-segment
//! carry-buffer frame-reassembly path (a partial DNP3 frame split across TCP
//! segments). The flow key is on port 20000 so the master-IP resolution
//! heuristic (`resolve_master_ip`, port-20000 outstation arm) runs its full
//! code path, and the unexpected-source-split detector is reachable.
//!
//! A panic anywhere in these paths is a fuzz finding. Every short/oversized/
//! invalid frame must be handled gracefully (None / gate-reject / desync-latch /
//! carry-stash / overflow-discard) — consistent with VP-023.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_dnp3_parse -- -max_total_time=120 -rss_limit_mb=4096

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::net::{IpAddr, Ipv4Addr};
use wirerust::analyzer::dnp3::{parse_dnp3_dl_header, Dnp3Analyzer};
use wirerust::reassembly::flow::FlowKey;

fuzz_target!(|data: &[u8]| {
    // --- Surface 1: direct pure-core DL-header parse over UNBOUNDED attacker bytes ---
    // Unbounded cross-check of VP-023 `verify_parse_dnp3_dl_header_safety`.
    let _ = parse_dnp3_dl_header(data);

    // --- Surface 2: full on_data carry-walk, two-segment cross-boundary path ---
    // Port-20000 flow so the master-IP resolution + unexpected-source detector run.
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let key = FlowKey::new(ip_a, 50000, ip_b, 20000);

    let mut analyzer = Dnp3Analyzer::new(10);

    // Split the input so a partial DNP3 frame can straddle the segment boundary,
    // forcing the carry buffer to buffer-and-complete across calls.
    let split = data.len() / 2;
    let (first, second) = data.split_at(split);

    // First segment establishes the flow (sync-word check / desync latch).
    analyzer.on_data(key.clone(), first, 1_700_000_000);
    // Second segment with a later timestamp exercises the carry cross-boundary
    // walk and the time-windowed correlation-window / block-timeout scans.
    analyzer.on_data(key.clone(), second, 1_700_000_005);
    // Feed the whole buffer once more, far in the future, to drive the 300s
    // correlation-window expiry (BC-2.15.015) and overflow-discard accounting.
    analyzer.on_data(key.clone(), data, 1_700_000_400);

    // A second flow on a non-standard port pair (NEITHER endpoint on 20000)
    // exercises the resolve_master_ip ambiguous arm.
    let ip_c = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ip_d = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));
    let key2 = FlowKey::new(ip_c, 40000, ip_d, 41000);
    let mut analyzer2 = Dnp3Analyzer::new(1);
    analyzer2.on_data(key2.clone(), data, 1_700_000_000);
    analyzer2.on_data(key2, second, 1_700_000_010);
});
