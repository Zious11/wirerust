//! Criterion micro-benchmarks for the wirerust pcap-processing pipeline
//! (LESSON-P2.07).
//!
//! Three benchmark groups cover the genuinely hot paths:
//!
//!   - `decode`  — raw frame → `ParsedPacket` decoding throughput.
//!   - `summary` — capture-level `Summary::ingest` accumulation.
//!   - `reassembly` — the full TCP reassembly + content-dispatch +
//!     per-protocol analyzer path.
//!
//! Each benchmark loads its pcap fixture once, outside the timed
//! closure, so file I/O is not counted. Run with `cargo bench`;
//! results land in `target/criterion/`.
//!
//! Fixture choices favor the larger consumed fixtures so the
//! benchmark exercises a realistic packet count rather than a
//! handful of frames.

use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};

use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::decoder::{DecodedFrame, decode_packet};
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reader::PcapSource;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};
use wirerust::summary::Summary;

/// Load a fixture's raw packets + datalink, panicking on failure —
/// benchmarks have no meaningful error-recovery path.
fn load(fixture: &str) -> PcapSource {
    let path = format!("tests/fixtures/{fixture}");
    PcapSource::from_file(Path::new(&path))
        .unwrap_or_else(|e| panic!("failed to load benchmark fixture {path}: {e}"))
}

/// Benchmark: decode every frame in a fixture into a `ParsedPacket`.
fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    for fixture in ["segmented.pcap", "tls.pcap", "dns-remoteshell.pcap"] {
        let source = load(fixture);
        group.bench_function(fixture, |b| {
            b.iter(|| {
                for raw in &source.packets {
                    let _ = black_box(decode_packet(&raw.data, source.datalink));
                }
            });
        });
    }
    group.finish();
}

/// Benchmark: capture-level `Summary` accumulation over every
/// successfully-decoded packet.
fn bench_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("summary");
    for fixture in ["segmented.pcap", "dns-remoteshell.pcap"] {
        let source = load(fixture);
        // Pre-decode once so the benchmark isolates `Summary::ingest`.
        // STORY-111: decode_packet now returns Result<DecodedFrame>; extract
        // only IP frames for the summary benchmark (ARP frames have no IP stats).
        let parsed: Vec<_> = source
            .packets
            .iter()
            .filter_map(
                |raw| match decode_packet(&raw.data, source.datalink).ok()? {
                    DecodedFrame::Ip(p) => Some(p),
                    DecodedFrame::Arp(_) => None,
                },
            )
            .collect();
        group.bench_function(fixture, |b| {
            b.iter(|| {
                let mut summary = Summary::new();
                for p in &parsed {
                    summary.ingest(p);
                }
                black_box(summary.total_packets)
            });
        });
    }
    group.finish();
}

/// Benchmark: the full reassembly + dispatch + HTTP/TLS analysis path
/// over a fixture, including the end-of-capture `finalize`.
fn bench_reassembly(c: &mut Criterion) {
    let mut group = c.benchmark_group("reassembly");
    for fixture in ["segmented.pcap", "tls.pcap"] {
        let source = load(fixture);
        // STORY-111: filter to IP frames only; ARP frames are not reassembled.
        let parsed: Vec<_> = source
            .packets
            .iter()
            .filter_map(
                |raw| match decode_packet(&raw.data, source.datalink).ok()? {
                    DecodedFrame::Ip(p) => Some((p, raw.timestamp_secs)),
                    DecodedFrame::Arp(_) => None,
                },
            )
            .collect();
        group.bench_function(fixture, |b| {
            b.iter(|| {
                let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
                let mut dispatcher = StreamDispatcher::new(
                    Some(HttpAnalyzer::new()),
                    Some(TlsAnalyzer::new()),
                    None,
                    None,
                    None,
                );
                for (p, ts) in &parsed {
                    reassembler.process_packet(p, *ts, &mut dispatcher);
                }
                reassembler.finalize(&mut dispatcher);
                black_box(reassembler.findings().len())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_decode, bench_summary, bench_reassembly);
criterion_main!(benches);
