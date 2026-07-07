//! AC-149-002 / PERF-002 fragmented-handshake fixture tests (closes issue #360).
//!
//! Verifies that the synthetic fragmented TLS handshake fixture:
//!   (a) returns >= 3 TLS record segments (carry-drain loop executes >= 2 times),
//!   (b) exercises the carry-drain loop across record boundaries, and
//!   (c) is deterministic (two builds are byte-identical).
//!
//! GREEN (STORY-149): `build_fragmented_handshake_fixture()` is fully
//! implemented — a deterministic 3-record ClientHello (45 bytes, 15 bytes/record)
//! that exercises the carry-drain loop >= 2 times. All three tests pass.
//!
//! The shared fixture builder lives in `tests/common/tls_fragmented_fixture.rs`
//! and is included via `include!` into both this file and `benches/tls_fragmented.rs`.
//! Bench files use `harness = false` and cannot be imported as library modules;
//! `include!` sharing eliminates duplication drift at compile time without requiring
//! a library module (STORY-149 / F-S149P1-004).
//!
//! Tests are wrapped in `mod bc_149_fragmented_fixture` per DF-TEST-NAMESPACE-001.
//!
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

mod bc_149_fragmented_fixture {
    #[allow(unused_imports)]
    use super::*;

    use std::net::IpAddr;

    use wirerust::analyzer::tls::TlsAnalyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{Direction, StreamHandler};

    // Shared builder: wrap_as_tls_record, build_client_hello_handshake_bytes,
    // build_fragmented_handshake_fixture. Path is relative to this source file's
    // directory (tests/), so "common/tls_fragmented_fixture.rs" resolves to
    // tests/common/tls_fragmented_fixture.rs.
    include!("common/tls_fragmented_fixture.rs");

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
    /// GREEN (STORY-149): `build_fragmented_handshake_fixture()` returns a
    /// deterministic 3-record sequence; this test passes.
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
    /// GREEN (STORY-149): `build_fragmented_handshake_fixture()` returns a
    /// deterministic 3-record sequence; this test passes.
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
    /// GREEN (STORY-149): `build_fragmented_handshake_fixture()` returns a
    /// deterministic 3-record sequence; this test passes.
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
}
