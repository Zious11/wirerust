//! VP-047 fuzz harness skeleton: IEC-104 on_data never panics on arbitrary input.
//!
//! `Iec104Analyzer::on_data` is the VP-047 cargo-fuzz target (ADR-013 Decision 8;
//! BC-2.19.026 postcondition 5). This harness feeds two successive on_data calls on
//! the same flow — deliberately exercising the cross-segment carry-buffer reassembly
//! path (a partial APCI frame split across TCP segments).
//!
//! The fuzzer splits the input at a midpoint and delivers both halves as separate
//! on_data calls on an established flow, exercising:
//!   - carry-append and carry-overflow detection (BC-2.19.025)
//!   - frame-walk loop advance modes: bad-start-byte (+1), malformed-LEN (+2 +
//!     EMIT-WITH-DEDUP), valid-frame (+LEN+2), insufficient (stash/return)
//!     (BC-2.19.026 / ADR-013 Decision 3)
//!   - VP-047 no-panic obligation for any byte sequence (BC-2.19.026 postcondition 5)
//!
//! Full fuzz run is in STORY-174. This skeleton establishes the harness seam.
//!
//! ## Architecture compliance (ADR-013 Decision 7 — licensing)
//! Forbidden dependencies (BANNED): `iec60870-5`, Wireshark `packet-104.c`, lib60870.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::net::IpAddr;
use wirerust::analyzer::iec104::Iec104Analyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::Direction;

fuzz_target!(|data: &[u8]| {
    let mut analyzer = Iec104Analyzer::new();

    let flow_key = FlowKey::new(
        IpAddr::from([10, 0, 0, 1]),
        1234,
        IpAddr::from([10, 0, 0, 2]),
        2404,
    );

    // Split at midpoint to exercise cross-segment carry reassembly.
    let mid = data.len() / 2;
    let (first, second) = data.split_at(mid);

    // First delivery: C2S direction.
    analyzer.on_data(flow_key.clone(), first, 0, Direction::ClientToServer);
    // Second delivery: C2S direction (same flow, continued).
    analyzer.on_data(flow_key.clone(), second, 1, Direction::ClientToServer);

    // Third delivery: S2C direction — verify directional isolation (BC-2.19.025).
    analyzer.on_data(flow_key.clone(), data, 2, Direction::ServerToClient);

    // Flow close: verify no-panic on teardown with non-empty carry (BC-2.19.027).
    analyzer.on_flow_close(flow_key.clone());

    // Double close: verify no-panic on unknown flow_key (BC-2.19.027 postcondition 4).
    analyzer.on_flow_close(flow_key);
});
