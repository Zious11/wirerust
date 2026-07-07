//! AC-149-002 / PERF-002 fragmented-handshake fixture tests (closes issue #360).
//!
//! Verifies that the synthetic fragmented TLS handshake fixture:
//!   (a) returns >= 3 TLS record segments (carry-drain loop executes >= 2 times),
//!   (b) exercises the carry-drain loop across record boundaries, and
//!   (c) is deterministic (two builds are byte-identical).
//!
//! RED GATE — FAILS NOW: `build_fragmented_handshake_fixture()` is a stub
//! (`todo!()`) — all three tests panic with:
//!   "STORY-149: implement synthetic >=3-record fragmented TLS handshake builder"
//!
//! NOTE: The fixture builder is defined here independently of the parallel
//! stub in `benches/tls_fragmented.rs`. Bench files use `harness = false`
//! and are not importable as library modules, so the test-facing builder
//! lives here. Both stubs carry the same `todo!()` body; the implementer
//! updates them to matching implementations simultaneously (STORY-149).
//!
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

use std::net::IpAddr;

use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};

/// Build a synthetic sequence of TLS record byte segments that together
/// form a single fragmented TLS handshake spanning >= 3 TLS records.
///
/// Each `Vec<u8>` in the returned sequence is one TCP segment payload to
/// deliver to [`TlsAnalyzer::on_data`]. The carry-drain loop in
/// `try_parse_records` must execute >= 2 times per call sequence
/// (AC-149-002 / issue #360). The builder is deterministic: two calls
/// return byte-identical sequences.
///
/// STUB — `todo!()` until STORY-149 implements the builder.
/// Intentionally duplicated from `benches/tls_fragmented.rs` for
/// `cargo test --all-targets` visibility.
fn build_fragmented_handshake_fixture() -> Vec<Vec<u8>> {
    todo!("STORY-149: implement synthetic >=3-record fragmented TLS handshake builder")
}

/// Returns a synthetic [`FlowKey`] for use in fixture tests.
fn fixture_flow_key() -> FlowKey {
    FlowKey::new(
        IpAddr::from([192u8, 168, 1, 1]),
        51234,
        IpAddr::from([10u8, 0, 0, 1]),
        443,
    )
}

/// AC-149-002 (PERF-002): the fixture must return >= 3 segment entries so
/// the carry-drain loop executes >= 2 times per synthetic handshake.
///
/// Each segment wraps one complete TLS record (5-byte header + payload).
/// The handshake message body is split across >= 3 records so that
/// records 1 .. N-1 leave partial data in the carry buffer and only
/// record N completes the carry-drain.
///
/// RED GATE — FAILS NOW: `build_fragmented_handshake_fixture()` panics with
/// `todo!("STORY-149: implement synthetic >=3-record fragmented TLS handshake builder")`.
#[test]
fn test_BC_149_002_fixture_spans_at_least_3_records() {
    let segments = build_fragmented_handshake_fixture();

    assert!(
        segments.len() >= 3,
        "AC-149-002 (PERF-002): build_fragmented_handshake_fixture must \
         return >= 3 TLS record segments so the carry-drain loop executes \
         >= 2 times per synthetic handshake. \
         Got {} segment(s) (STORY-149 / issue #360).",
        segments.len()
    );
}

/// AC-149-002 (PERF-002): delivering all segments to `TlsAnalyzer` must
/// complete exactly one handshake; delivering only the first N-1 segments
/// must leave the carry buffer non-empty (proving the carry-drain loop is
/// traversed across record boundaries, not resolved within a single delivery).
///
/// RED GATE — FAILS NOW: `build_fragmented_handshake_fixture()` panics with
/// `todo!("STORY-149: implement synthetic >=3-record fragmented TLS handshake builder")`.
#[test]
fn test_BC_149_002_carry_drain_loop_exercised_across_records() {
    let segments = build_fragmented_handshake_fixture();
    let key = fixture_flow_key();

    // Phase 1: deliver the first N-1 segments.
    // The client-direction carry buffer must be non-empty (bytes have
    // accumulated across records but the handshake message is not yet
    // complete) and no handshake may have been dispatched yet.
    let mut analyzer = TlsAnalyzer::new();
    for segment in segments.iter().take(segments.len() - 1) {
        analyzer.on_data(&key, Direction::ClientToServer, segment, 0, 0);
    }

    let carry_after_partial = analyzer.client_hs_carry_len_for_testing(&key);
    assert!(
        carry_after_partial > 0,
        "AC-149-002: after delivering the first N-1 segments the \
         client-direction carry buffer must be non-empty — the handshake \
         spans >= 3 records and cannot complete until the final fragment \
         arrives (STORY-149 / issue #360). \
         carry_len after N-1 segments = {carry_after_partial}"
    );
    assert_eq!(
        analyzer.handshake_count(),
        0,
        "AC-149-002: the handshake must NOT be dispatched until the final \
         segment is delivered; the carry-drain loop must not resolve until \
         all fragments arrive (STORY-149 / issue #360)"
    );

    // Phase 2: deliver the final segment.
    // The carry-drain loop must now consume the accumulated bytes and
    // dispatch exactly one complete handshake.
    analyzer.on_data(
        &key,
        Direction::ClientToServer,
        segments.last().unwrap(),
        0,
        0,
    );
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-149-002: delivering the final segment must complete the \
         fragmented handshake via the carry-drain loop; \
         expected handshake_count == 1 after all segments are delivered \
         (STORY-149 / issue #360)"
    );
}

/// AC-149-002 (PERF-002): the fixture must be deterministic — two calls
/// must return byte-identical segment sequences so Criterion can produce
/// meaningful statistics across iterations.
///
/// RED GATE — FAILS NOW: `build_fragmented_handshake_fixture()` panics with
/// `todo!("STORY-149: implement synthetic >=3-record fragmented TLS handshake builder")`.
#[test]
fn test_BC_149_002_fixture_is_deterministic() {
    let first = build_fragmented_handshake_fixture();
    let second = build_fragmented_handshake_fixture();

    assert_eq!(
        first.len(),
        second.len(),
        "AC-149-002: build_fragmented_handshake_fixture must be \
         deterministic; two calls returned different segment counts \
         ({} vs {}) (STORY-149 / issue #360)",
        first.len(),
        second.len()
    );

    for (i, (a, b)) in first.iter().zip(second.iter()).enumerate() {
        assert_eq!(
            a, b,
            "AC-149-002: build_fragmented_handshake_fixture must be \
             deterministic; segment {i} differed between two calls \
             (STORY-149 / issue #360)"
        );
    }
}
