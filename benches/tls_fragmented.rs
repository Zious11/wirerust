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

/// Wrap `payload` in a minimal 5-byte TLS record header (version TLS 1.2).
fn wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
    let len = payload.len();
    let mut record = vec![content_type, 0x03, 0x03, (len >> 8) as u8, (len & 0xff) as u8];
    record.extend_from_slice(payload);
    record
}

/// Build a synthetic sequence of TLS record byte segments that together form
/// a single fragmented TLS handshake spanning at least 3 TLS records.
///
/// Each returned `Vec<u8>` represents one TCP segment payload to be delivered
/// to the TLS analyzer via [`StreamHandler::on_data`]. The carry-drain loop in
/// `try_parse_records` executes at least twice per call sequence
/// (AC-149-002: carry-drain loop executes >= twice per synthetic handshake).
///
/// The handshake is deterministic and repeatable so Criterion produces
/// meaningful statistics.
///
/// Layout: a minimal TLS ClientHello (no extensions, 45 bytes total) split
/// into 3 TLS records of 15 bytes each so the carry accumulates across records
/// and dispatches only on the final fragment.
///
/// Intentionally duplicated from `tests/bc_149_fragmented_fixture_tests.rs`
/// (bench files use `harness = false` and cannot be imported as library
/// modules — STORY-149 note).
fn build_fragmented_handshake() -> Vec<Vec<u8>> {
    // Build minimal ClientHello handshake-message bytes (4-byte header + 41-byte body).
    let mut body: Vec<u8> = Vec::with_capacity(41);
    body.extend_from_slice(&[0x03, 0x03]); // version TLS 1.2
    body.extend_from_slice(&[0u8; 32]); // client random (32 deterministic zeros)
    body.push(0x00); // session_id_len = 0
    body.extend_from_slice(&[0x00, 0x02]); // cipher_suites_len = 2
    body.extend_from_slice(&[0x00, 0x2f]); // TLS_RSA_WITH_AES_128_CBC_SHA
    body.push(0x01); // compression_methods_len = 1
    body.push(0x00); // null compression

    let body_len = body.len() as u32;
    let mut hs: Vec<u8> = Vec::with_capacity(4 + body.len());
    hs.push(0x01); // msg_type = ClientHello
    hs.push((body_len >> 16) as u8);
    hs.push((body_len >> 8) as u8);
    hs.push(body_len as u8);
    hs.extend_from_slice(&body);

    // Split into 3 equal parts (15 bytes each).
    let n = hs.len(); // 45 bytes
    let split1 = n / 3; // 15
    let split2 = 2 * n / 3; // 30

    vec![
        wrap_as_tls_record(0x16, &hs[..split1]),
        wrap_as_tls_record(0x16, &hs[split1..split2]),
        wrap_as_tls_record(0x16, &hs[split2..]),
    ]
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
