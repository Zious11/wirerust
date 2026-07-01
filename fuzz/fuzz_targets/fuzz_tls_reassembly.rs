//! F6 fuzz harness: TLS handshake carry-reassembly path never panics on arbitrary input.
//!
//! Exercises the STORY-144/145/146 delta in `src/analyzer/tls.rs`:
//!
//!   * `TlsAnalyzer::on_data(...)` — the StreamHandler shell that appends to the
//!     per-direction TCP segment buffer (`client_buf` / `server_buf`, MAX_BUF cap),
//!     drives the buffer-saturation tail-drop detector (STORY-146,
//!     `buffer_saturation_drops`), and calls `try_parse_records`.
//!
//!   * `try_parse_records` — the TLS record framer plus the cursor-based handshake
//!     carry-drain loop (STORY-144 ClientToServer / STORY-145 ServerToClient). This
//!     is the core delta: Step-1 pre-append overflow guard (Decision 5), the
//!     `consumed`-cursor drain loop, the Decision-4 body_len-spoof guard, and the
//!     single post-loop `drain(..consumed)`.
//!
//!   * `summarize()` — the BTreeMap key insertions (including the always-present
//!     `handshake_reassembly_overflows` / `buffer_saturation_drops` counters).
//!
//! The fuzzer slices the arbitrary input into a sequence of variable-length
//! segments and feeds them via `on_data`, ALTERNATING direction so BOTH the
//! ClientToServer (ClientHello, msg_type 0x01) and ServerToClient (ServerHello,
//! msg_type 0x02) carry paths run, deliberately straddling TLS records and
//! handshake messages across segment boundaries to force cross-segment carry
//! reassembly. It then calls `on_flow_close` and `summarize()` / `findings()`.
//!
//! A panic / OOB index / arithmetic-overflow / OOM anywhere in these paths is a
//! fuzz finding. Every malformed / oversized / fragmented record must be handled
//! gracefully (drop / overflow-counter-bump / carry-stash) without panic.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_tls_reassembly -- -max_total_time=180 -rss_limit_mb=2048

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::net::{IpAddr, Ipv4Addr};
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

fuzz_target!(|data: &[u8]| {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    // Port 443 flow so this reads as a TLS flow to any upstream routing.
    let key = FlowKey::new(ip_a, 50000, ip_b, 443);

    let mut analyzer = TlsAnalyzer::new();

    // Use the first byte as a directional/segmentation seed; the remainder is the
    // byte stream that gets carved into segments. An empty input still drives the
    // close + summarize paths.
    let (seed, body) = match data.split_first() {
        Some((s, rest)) => (*s, rest),
        None => (0u8, &[][..]),
    };

    // Carve `body` into variable-length segments. Segment sizes are derived from
    // the stream bytes themselves so the fuzzer can evolve toward boundaries that
    // split a TLS record header (5 bytes) or a handshake header (4 bytes) across
    // `on_data` calls — exercising cross-segment carry reassembly.
    let mut offset: usize = 0;
    let mut idx: u64 = 0;
    let mut pos: u64 = 0;
    while pos < body.len() as u64 {
        let p = pos as usize;
        // Chunk length 1..=512, derived from the current byte + the rotating seed.
        let chunk = 1usize + ((body[p] as usize ^ (seed as usize)) % 512);
        let end = (p + chunk).min(body.len());
        let segment = &body[p..end];

        // Alternate direction each segment so both ClientHello (0x01, C2S) and
        // ServerHello (0x02, S2C) carry-drain paths run on the same flow.
        let direction = if idx & 1 == 0 {
            Direction::ClientToServer
        } else {
            Direction::ServerToClient
        };

        analyzer.on_data(&key, direction, segment, offset as u64, 1_700_000_000 + idx as u32);

        offset += segment.len();
        idx += 1;
        pos = end as u64;

        // Bound the number of segments so a tiny input cannot blow up iteration
        // count; libFuzzer inputs are small but this keeps each exec fast.
        if idx > 4096 {
            break;
        }
    }

    // Also feed the entire body once in each direction to drive longer single
    // segments through the saturation tail-drop + record-framer paths.
    analyzer.on_data(&key, Direction::ClientToServer, body, 0, 1_700_001_000);
    analyzer.on_data(&key, Direction::ServerToClient, body, 0, 1_700_001_001);

    // Drain summarize() (BTreeMap key insertions + counter surfacing) and findings.
    let _ = analyzer.summarize();
    let _ = analyzer.findings();

    // Close the flow (drops per-flow carry state without panic).
    analyzer.on_flow_close(&key, CloseReason::Fin);
});
