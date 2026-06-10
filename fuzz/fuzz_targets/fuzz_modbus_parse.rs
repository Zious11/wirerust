//! VP-022 fuzz harness: Modbus MBAP parse path never panics on arbitrary input.
//!
//! The Modbus analyzer consumes attacker-controlled pcap bytes via two surfaces:
//!
//!   1. `parse_mbap_header(&[u8])` — the pure-core fixed-offset MBAP decoder.
//!      VP-022 sub-property A proves panic/OOB freedom under Kani for slices of
//!      length 0..=12; this fuzz target exercises the SAME function over
//!      arbitrary-length attacker bytes (no length bound) as an independent,
//!      unbounded cross-check of the Kani proof.
//!
//!   2. `ModbusAnalyzer::on_data(...)` — the effectful StreamHandler shell that
//!      walks a TCP byte stream, parsing and dispatching every ADU. This path
//!      includes the F-105-001 partial-ADU carry buffer (260-byte cap), the
//!      pending-transaction table (256-cap), the 3-point validity gate, the
//!      desync latch, and the full seven-detector detection engine
//!      (`process_pdu`). Kani cannot drive `on_data` directly (its HashMap state
//!      uses `RandomState`, an FFI the model checker cannot symbolically
//!      execute), so fuzzing is the primary dynamic safety check for this shell.
//!
//! The fuzzer splits the input into TWO chunks and feeds them as two successive
//! `on_data` calls on the SAME flow, deliberately exercising the carry-buffer
//! cross-segment ADU-reassembly path (a partial ADU split across TCP segments).
//! It also alternates direction so request-insert and response-match paths both
//! run.
//!
//! A panic anywhere in these paths is a fuzz finding. No `unwrap()` on attacker
//! bytes is expected to fire; every short/oversized/invalid ADU must be handled
//! gracefully (None / gate-reject / desync-latch / carry-stash).
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_modbus_parse -- -max_total_time=300

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::net::{IpAddr, Ipv4Addr};
use wirerust::analyzer::modbus::{parse_mbap_header, ModbusAnalyzer};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};

fuzz_target!(|data: &[u8]| {
    // --- Surface 1: direct pure-core MBAP parse over UNBOUNDED attacker bytes ---
    // Unbounded cross-check of VP-022 sub-property A (Kani bounds len <= 12).
    let _ = parse_mbap_header(data);

    // --- Surface 2: full on_data ADU walk, two-segment carry-buffer path ---
    // Use a port-502 flow key so the Modbus client/server IP resolution and the
    // detection engine run their full code paths.
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let key = FlowKey::new(ip_a, 50000, ip_b, 502);

    let mut analyzer = ModbusAnalyzer::new(20, 100);

    // Split the input so a partial ADU can straddle the segment boundary,
    // forcing the carry buffer to buffer-and-complete across calls.
    let split = data.len() / 2;
    let (first, second) = data.split_at(split);

    // First segment: client -> server (request-insert path).
    analyzer.on_data(&key, Direction::ClientToServer, first, 0, 1_700_000_000);
    // Second segment: server -> client (response-match / exception path), with a
    // later timestamp to exercise the time-windowed burst/sustained detectors.
    analyzer.on_data(
        &key,
        Direction::ServerToClient,
        second,
        first.len() as u64,
        1_700_000_005,
    );
    // Feed the whole buffer once more, same direction, to drive duplicate-inflight
    // and pending-table-cap (256) accounting on a longer stream.
    analyzer.on_data(
        &key,
        Direction::ClientToServer,
        data,
        0,
        1_700_000_010,
    );

    // Flow close: drains any per-flow finalization without panic.
    analyzer.on_flow_close(&key, CloseReason::Fin);
});
