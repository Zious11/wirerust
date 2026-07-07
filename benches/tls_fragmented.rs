//! Criterion benchmark for the TLS fragmented-handshake carry-drain path
//! (AC-149-002; closes issue #360).
//!
//! This fixture exercises the carry-drain loop introduced in STORY-144/145 by
//! delivering a synthetic TLS handshake message split across at least 3 TLS
//! records, forcing the carry-drain loop to execute at least twice per
//! iteration. Run with `cargo bench --bench tls_fragmented`.
//!
//! The synthetic-handshake builder (`build_fragmented_handshake`) is a stub
//! (`todo!()`) until STORY-149 implements it. Once implemented, this bench
//! establishes the regression baseline for future carry-path changes.

use std::hint::black_box;
use std::net::IpAddr;

use criterion::{Criterion, criterion_group, criterion_main};

use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};

/// Build a synthetic sequence of TLS record byte segments that together form
/// a single fragmented TLS handshake spanning at least 3 TLS records.
///
/// Each returned `Vec<u8>` represents one TCP segment payload to be delivered
/// to the TLS analyzer via [`StreamHandler::on_data`]. The carry-drain loop in
/// `try_parse_records` must execute at least twice per call sequence
/// (AC-149-002: carry-drain loop executes >= twice per synthetic handshake).
///
/// The handshake is deterministic and repeatable so Criterion produces
/// meaningful statistics.
///
/// STUB: body is `todo!()` per Red Gate discipline — implement in STORY-149.
fn build_fragmented_handshake() -> Vec<Vec<u8>> {
    todo!("STORY-149: implement synthetic >=3-record fragmented TLS handshake builder")
}

/// Benchmark the TLS carry-drain loop over a synthetic fragmented handshake.
///
/// Feeds each segment directly to [`TlsAnalyzer::on_data`] on a synthetic
/// flow key, isolating the carry-drain path from TCP-reassembly overhead.
fn bench_tls_fragmented(c: &mut Criterion) {
    let mut group = c.benchmark_group("tls_fragmented");

    let key = FlowKey::new(
        IpAddr::from([192u8, 168, 1, 1]),
        51234,
        IpAddr::from([10u8, 0, 0, 1]),
        443,
    );
    let segments = build_fragmented_handshake();

    group.bench_function("3-record-carry-drain", |b| {
        b.iter(|| {
            let mut analyzer = TlsAnalyzer::new();
            for segment in &segments {
                analyzer.on_data(&key, Direction::ClientToServer, segment, 0, 0);
            }
            black_box(analyzer.handshake_count())
        });
    });

    group.finish();
}

criterion_group!(benches, bench_tls_fragmented);
criterion_main!(benches);
