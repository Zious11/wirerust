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
//! NOTE: The fixture builder is defined here independently of the parallel
//! builder in `benches/tls_fragmented.rs`. Bench files use `harness = false`
//! and are not importable as library modules, so the test-facing builder
//! lives here. Both builders deliver byte-identical segment sequences
//! (STORY-149).
//!
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

use std::net::IpAddr;

use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};

/// Build a minimal TLS ClientHello handshake-message byte sequence (no TLS record
/// header; 4-byte handshake header + ClientHello body).
///
/// Produces a valid `parse_tls_message_handshake`-parseable ClientHello with a
/// single AES-128-CBC cipher suite and no extensions so the body is short and
/// the split arithmetic is simple.
///
/// Layout (bytes):
///   [0x01]          — msg_type = ClientHello
///   [0x00, 0x00, L] — body length (3 bytes, big-endian)
///   [0x03, 0x03]    — version TLS 1.2
///   [0u8 * 32]      — client random (32 zeros, deterministic)
///   [0x00]          — session_id length = 0
///   [0x00, 0x02]    — cipher_suites length = 2 (one suite)
///   [0x00, 0x2f]    — TLS_RSA_WITH_AES_128_CBC_SHA (0x002f)
///   [0x01]          — compression_methods length = 1
///   [0x00]          — null compression
///
/// Total: 4 + 41 = 45 bytes — compact enough for clean 3-way splitting.
fn build_client_hello_handshake_bytes() -> Vec<u8> {
    // Build ClientHello body.
    let mut body: Vec<u8> = Vec::with_capacity(41);
    body.extend_from_slice(&[0x03, 0x03]); // version TLS 1.2
    body.extend_from_slice(&[0u8; 32]); // random (32 deterministic zeros)
    body.push(0x00); // session_id_len = 0
    body.extend_from_slice(&[0x00, 0x02]); // cipher_suites_len = 2
    body.extend_from_slice(&[0x00, 0x2f]); // TLS_RSA_WITH_AES_128_CBC_SHA
    body.push(0x01); // compression_methods_len = 1
    body.push(0x00); // null compression
    // No extensions field — ch.ext will be None; handle_client_hello handles this.

    // Build 4-byte handshake header.
    let body_len = body.len() as u32;
    let mut hs: Vec<u8> = Vec::with_capacity(4 + body.len());
    hs.push(0x01); // msg_type = ClientHello
    hs.push((body_len >> 16) as u8); // length byte 0 (big-endian 3-byte)
    hs.push((body_len >> 8) as u8); // length byte 1
    hs.push(body_len as u8); // length byte 2
    hs.extend_from_slice(&body);
    hs
}

/// Wrap `payload` in a minimal 5-byte TLS record header.
///
/// Uses version bytes `[0x03, 0x03]` (TLS 1.2) — consistent with the VP-039
/// `wrap_as_tls_record` helper. `content_type` must be 0x16 for handshake records.
fn wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
    let len = payload.len();
    let mut record = vec![
        content_type,
        0x03,
        0x03,
        (len >> 8) as u8,
        (len & 0xff) as u8,
    ];
    record.extend_from_slice(payload);
    record
}

/// Build a synthetic sequence of TLS record byte segments that together
/// form a single fragmented TLS handshake spanning >= 3 TLS records.
///
/// Each `Vec<u8>` in the returned sequence is one TCP segment payload to
/// deliver to [`TlsAnalyzer::on_data`]. The carry-drain loop in
/// `try_parse_records` must execute >= 2 times per call sequence
/// (AC-149-002 / issue #360). The builder is deterministic: two calls
/// return byte-identical sequences.
///
/// Intentionally duplicated from `benches/tls_fragmented.rs` for
/// `cargo test --all-targets` visibility (bench files use `harness = false`
/// and cannot be imported as library modules — STORY-149 note).
fn build_fragmented_handshake_fixture() -> Vec<Vec<u8>> {
    let hs = build_client_hello_handshake_bytes();
    let n = hs.len(); // 45 bytes

    // Split into 3 approximately equal parts.
    // split1 = 15, split2 = 30, final = 15.
    // After segment 1: carry = 15 bytes < 45 = 4 + 41 → incomplete, no dispatch.
    // After segment 2: carry = 30 bytes < 45 → still incomplete, no dispatch.
    // After segment 3: carry = 45 bytes >= 45 → complete, dispatch ClientHello.
    let split1 = n / 3;
    let split2 = 2 * n / 3;

    vec![
        wrap_as_tls_record(0x16, &hs[..split1]),
        wrap_as_tls_record(0x16, &hs[split1..split2]),
        wrap_as_tls_record(0x16, &hs[split2..]),
    ]
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
