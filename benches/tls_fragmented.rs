//! Criterion benchmark for the TLS fragmented-handshake carry-drain path
//! (AC-149-002; closes issue #360).
//!
//! This fixture exercises the carry-drain loop introduced in STORY-144/145 by
//! delivering a synthetic TLS handshake message split across at least 3 TLS
//! records, forcing the carry-drain loop to execute at least twice per
//! iteration. Run with `cargo bench --bench tls_fragmented`.
//!
//! The shared fixture builder (`build_fragmented_handshake_fixture`) lives in
//! `tests/common/tls_fragmented_fixture.rs` and is included via `include!`.
//! Bench files use `harness = false` and cannot be imported as library modules;
//! `include!` sharing eliminates the previous duplication drift at compile time
//! (STORY-149 / F-S149P1-004).

use std::hint::black_box;
use std::net::IpAddr;

use criterion::{Criterion, criterion_group, criterion_main};

use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};

// Shared builder: wrap_as_tls_record, build_client_hello_handshake_bytes,
// build_fragmented_handshake_fixture. Path is relative to this source file's
// directory (benches/), so "../tests/common/..." resolves to
// tests/common/tls_fragmented_fixture.rs.
include!("../tests/common/tls_fragmented_fixture.rs");

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
    let segments = build_fragmented_handshake_fixture();

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
