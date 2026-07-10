// Shared TLS fragmented-handshake fixture builder.
//
// This file is NOT a Rust module — it is included verbatim via `include!` by:
//   tests/bc_149_fragmented_fixture_tests.rs  — include!("common/tls_fragmented_fixture.rs")
//   benches/tls_fragmented.rs                  — include!("../tests/common/tls_fragmented_fixture.rs")
//
// Bench files use `harness = false` and cannot be imported as library modules;
// `include!` sharing eliminates duplication drift at compile time without
// requiring a library module (STORY-149 / F-S149P1-004).

/// Wrap `payload` in a minimal 5-byte TLS record header.
///
/// Uses version bytes `[0x03, 0x03]` (TLS 1.2). `content_type` must be 0x16
/// for handshake records — consistent with the VP-039 `wrap_as_tls_record` helper.
fn wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
    let len = u16::try_from(payload.len())
        .expect("fixture payload exceeds u16 TLS record length");
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

/// Build a minimal TLS ClientHello handshake-message byte sequence (no TLS record
/// header; 4-byte handshake header + ClientHello body).
///
/// Produces a valid `parse_tls_message_handshake`-parseable ClientHello with a
/// single AES-128-CBC cipher suite and no extensions so the body is short and
/// the split arithmetic is simple.
///
/// Layout (bytes):
///   `[0x01]`          — msg_type = ClientHello
///   `[0x00, 0x00, L]` — body length (3 bytes, big-endian)
///   `[0x03, 0x03]`    — version TLS 1.2
///   `[0u8 * 32]`      — client random (32 zeros, deterministic)
///   `[0x00]`          — session_id length = 0
///   `[0x00, 0x02]`    — cipher_suites length = 2 (one suite)
///   `[0x00, 0x2f]`    — TLS_RSA_WITH_AES_128_CBC_SHA (0x002f)
///   `[0x01]`          — compression_methods length = 1
///   `[0x00]`          — null compression
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
    // SEC-010 disposition: accepted-as-bounded-by-construction — fixed 41-byte ClientHello body
    let body_len = body.len() as u32;
    let mut hs: Vec<u8> = Vec::with_capacity(4 + body.len());
    hs.push(0x01); // msg_type = ClientHello
    hs.push((body_len >> 16) as u8); // length byte 0 (big-endian 3-byte)
    hs.push((body_len >> 8) as u8); // length byte 1
    hs.push(body_len as u8); // length byte 2
    hs.extend_from_slice(&body);
    hs
}

/// Build a synthetic sequence of TLS record byte segments that together
/// form a single fragmented TLS handshake spanning >= 3 TLS records.
///
/// Each `Vec<u8>` in the returned sequence is one TCP segment payload to
/// deliver to `TlsAnalyzer::on_data`. The carry-drain loop in
/// `try_parse_records` must execute >= 2 times per call sequence
/// (AC-149-002 / issue #360). The builder is deterministic: two calls
/// return byte-identical sequences.
///
/// Shared via `include!` from both:
///   `tests/bc_149_fragmented_fixture_tests.rs` (test coverage) and
///   `benches/tls_fragmented.rs` (Criterion regression fixture).
/// Using `include!` eliminates duplication drift without requiring a library
/// module — bench files use `harness = false` and cannot be imported as
/// library modules (STORY-149 / F-S149P1-004).
fn build_fragmented_handshake_fixture() -> Vec<Vec<u8>> {
    let hs = build_client_hello_handshake_bytes();
    let n = hs.len(); // 45 bytes

    // Split into 3 approximately equal parts (15 / 15 / 15).
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
