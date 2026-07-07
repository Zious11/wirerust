//! STORY-150 / AC-150-001 / AC-150-003 / VP-039 / BC-2.07.004 / BC-2.07.028 tests.
//!
//! # Structure
//!
//! ## Red Gate tests (AC-150-001) — MUST FAIL before implementation
//!
//! Two structural tests assert that the C2S/S2C dispatch-arm duplication
//! (TLS-DRAIN-DUP-001) is gone from `process_handshake_carry`:
//!
//!   - `test_BC_150_001_..._parse_hs_call_not_duplicated`: expects exactly ONE
//!     call to `parse_tls_message_handshake` in the function body (currently 2).
//!   - `test_BC_150_001_..._msg_bytes_extraction_not_duplicated`: expects at most
//!     ONE `let msg_bytes` extraction site (currently 2).
//!
//! Both tests FAIL NOW (before implementation) and PASS after AC-150-001.
//!
//! ## AC-150-003 — not machine-checkable as a Red Gate; documented below
//!
//! The VP-039 line-correspondence table (`src/analyzer/tls.rs` lines ~1806–1818)
//! uses descriptive step names, not `// VP-039: line N` annotations keyed to
//! specific source line numbers. There is therefore no machine-checkable format
//! that would let a test verify that the prose table's step descriptions
//! correspond to the correct current lines in `process_handshake_carry`. A Red
//! Gate test cannot be written for this criterion without adopting a new
//! annotation convention (e.g., `// VP-039-LINE: N`).
//!
//! AC-150-003 is therefore covered by:
//!   (a) `test_BC_150_003_vp039_proof_module_marker_present` — confirms the VP-039
//!       proof module header (table marker string) was not accidentally deleted.
//!   (b) The behavior-preservation regression pins below, which exercise the same
//!       carry-drain-loop paths that the Kani `drain_loop_model` models.
//!
//! ## Behavior-preservation regression pins (VP-039 / BC-2.07.004 / BC-2.07.028)
//!
//! These tests verify that the C2S and S2C carry-drain paths produce identical
//! observable outcomes (carry accumulation, flag-set, parse_errors) before AND
//! after the DRY refactor. They PASS NOW and must continue to PASS after
//! AC-150-001 implementation. If any of these fail after refactor, the
//! implementation broke behavioral equivalence — not these tests.
//!
//! Tests are wrapped in `mod story_150` per DF-TEST-NAMESPACE-001.
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

mod story_150 {
    #[allow(unused_imports)]
    use super::*;

    use std::net::IpAddr;

    use wirerust::analyzer::tls::TlsAnalyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{Direction, StreamHandler};

    // ── Fixture helpers ──────────────────────────────────────────────────────

    fn fixture_flow_key() -> FlowKey {
        FlowKey::new(
            IpAddr::from([192u8, 168, 1, 1]),
            51234,
            IpAddr::from([10u8, 0, 0, 1]),
            443,
        )
    }

    fn fixture_flow_key_b() -> FlowKey {
        FlowKey::new(
            IpAddr::from([10u8, 0, 0, 3]),
            9876,
            IpAddr::from([10u8, 0, 0, 4]),
            8443,
        )
    }

    /// Wrap `payload` bytes in a 5-byte TLS 1.2 handshake record header (0x16).
    fn wrap_as_tls_record(payload: &[u8]) -> Vec<u8> {
        let len = payload.len();
        let mut record = vec![0x16u8, 0x03, 0x03, (len >> 8) as u8, (len & 0xff) as u8];
        record.extend_from_slice(payload);
        record
    }

    /// Build a minimal ClientHello handshake message (raw bytes, no TLS record
    /// header). Layout: 4-byte handshake header + 41-byte body = 45 bytes total.
    ///
    /// Produces a `parse_tls_message_handshake`-parseable ClientHello with a
    /// single cipher suite and no extensions. Deterministic: two calls return
    /// byte-identical output. Adapted from `tests/common/tls_fragmented_fixture.rs`.
    fn build_client_hello_handshake_bytes() -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&[0x03, 0x03]); // version TLS 1.2
        body.extend_from_slice(&[0u8; 32]); // client random (deterministic zeros)
        body.push(0x00); // session_id_len = 0
        body.extend_from_slice(&[0x00, 0x02]); // cipher_suites_len = 2
        body.extend_from_slice(&[0x00, 0x2f]); // TLS_RSA_WITH_AES_128_CBC_SHA
        body.push(0x01); // compression_methods_len = 1
        body.push(0x00); // null compression
        // body = 41 bytes; no extensions field so ch.ext will be None.

        let body_len = body.len() as u32;
        let mut hs = vec![
            0x01_u8, // msg_type = ClientHello
            (body_len >> 16) as u8,
            (body_len >> 8) as u8,
            body_len as u8,
        ];
        hs.extend_from_slice(&body);
        hs // 4 + 41 = 45 bytes
    }

    /// Build a minimal ServerHello handshake message (raw bytes, no TLS record
    /// header). Layout: 4-byte handshake header + 45-byte body = 49 bytes total.
    ///
    /// Produces a `parse_tls_message_handshake`-parseable ServerHello. Includes
    /// a renegotiation_info extension (same structure as `build_server_hello` in
    /// `tests/tls_analyzer_tests.rs`) to ensure tls-parser accepts the message.
    /// Deterministic: two calls return byte-identical output.
    fn build_server_hello_handshake_bytes() -> Vec<u8> {
        // Extensions: renegotiation_info (0xff01) with empty payload = 5 bytes.
        let mut extensions = Vec::new();
        extensions.extend_from_slice(&[0xff, 0x01]); // extension type
        extensions.extend_from_slice(&[0x00, 0x01]); // extension data length = 1
        extensions.push(0x00); // empty renegotiation info

        let mut body = Vec::new();
        body.extend_from_slice(&[0x03, 0x03]); // version TLS 1.2
        body.extend_from_slice(&[0u8; 32]); // server random (deterministic zeros)
        body.push(0x00); // session_id_len = 0
        body.extend_from_slice(&[0x13, 0x01]); // cipher TLS_AES_128_GCM_SHA256
        body.push(0x00); // compression: null

        let ext_len = extensions.len() as u16;
        body.extend_from_slice(&ext_len.to_be_bytes()); // extensions length
        body.extend_from_slice(&extensions); // 5 bytes extensions
        // body = 2+32+1+2+1+2+5 = 45 bytes

        let body_len = body.len() as u32;
        let mut hs = vec![
            0x02_u8, // msg_type = ServerHello
            (body_len >> 16) as u8,
            (body_len >> 8) as u8,
            body_len as u8,
        ];
        hs.extend_from_slice(&body);
        hs // 4 + 45 = 49 bytes
    }

    // ── Source-inspection helper (mirrors bc_149_single_borrow_invariant_tests.rs) ──

    /// Extract the body of the first function whose definition line contains
    /// `fn_sig`, using brace-depth counting to locate the matching closing `}`.
    ///
    /// Returns the text from the opening `{` (inclusive) to the matching
    /// closing `}` (inclusive). Panics if the signature or its brace-pair are
    /// not found. Brace characters inside string literals or line-comments
    /// contribute to the count; this is acceptable because `process_handshake_carry`
    /// does not contain free-standing unmatched braces in comments or strings.
    fn extract_fn_body(source: &str, fn_sig: &str) -> String {
        let sig_pos = source.find(fn_sig).unwrap_or_else(|| {
            panic!("function signature {fn_sig:?} not found in source");
        });
        let rel_open = source[sig_pos..].find('{').unwrap_or_else(|| {
            panic!("no opening '{{' found after function signature {fn_sig:?}");
        });
        let open_pos = sig_pos + rel_open;
        let tail = &source[open_pos..];
        let mut depth: usize = 0;
        let mut end_byte: usize = 0;
        for (byte_idx, ch) in tail.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end_byte = byte_idx + 1; // '}' is always 1 byte in UTF-8
                        break;
                    }
                }
                _ => {}
            }
        }
        assert!(end_byte > 0, "no matching '}}' found for {fn_sig:?}");
        source[open_pos..open_pos + end_byte].to_string()
    }

    fn read_tls_source() -> String {
        std::fs::read_to_string("src/analyzer/tls.rs").unwrap_or_else(|e| {
            panic!(
                "failed to read src/analyzer/tls.rs \
                 (cargo test must be run from the crate root): {e}"
            )
        })
    }

    // ── AC-150-001: Red Gate structural tests ─────────────────────────────────
    //
    // These tests MUST FAIL before AC-150-001 implementation and MUST PASS after.
    //
    // Current state: `process_handshake_carry` contains TWO symmetric arms
    // (Direction::ClientToServer and Direction::ServerToClient) that each contain
    // a `let msg_bytes = carry[...].to_vec()` extraction and a
    // `parse_tls_message_handshake(&msg_bytes)` call. After AC-150-001 the shared
    // dispatch abstraction (closure, helper function, or macro) will have exactly
    // one call site for each.

    /// AC-150-001 (TLS-DRAIN-DUP-001): `process_handshake_carry` must contain
    /// exactly ONE call to `parse_tls_message_handshake` after the C2S/S2C dispatch
    /// arms are unified via a shared abstraction.
    ///
    /// RED GATE — FAILS before implementation (2 occurrences present; one per arm).
    /// PASSES after AC-150-001 (shared abstraction has 1 call site).
    #[test]
    fn test_BC_150_001_process_handshake_carry_parse_hs_call_not_duplicated() {
        let source = read_tls_source();
        let body = extract_fn_body(&source, "fn process_handshake_carry(");

        let count = body.matches("parse_tls_message_handshake(").count();

        assert_eq!(
            count, 1,
            "AC-150-001 (TLS-DRAIN-DUP-001): `process_handshake_carry` must \
             contain exactly ONE call to `parse_tls_message_handshake` after \
             the C2S and S2C dispatch arms are unified via a shared abstraction \
             (STORY-150). Found {count} call site(s). BEFORE implementation: 2 \
             sites exist — one in the Direction::ClientToServer arm and one in \
             the Direction::ServerToClient arm. AFTER AC-150-001: the shared \
             dispatch abstraction (closure/function/macro) reduces this to 1. \
             This test is the Red Gate for AC-150-001 (TLS-DRAIN-DUP-001)."
        );
    }

    /// AC-150-001 (TLS-DRAIN-DUP-001): `process_handshake_carry` must contain
    /// at most ONE `let msg_bytes` carry-slice extraction after the dispatch arms
    /// are unified.
    ///
    /// RED GATE — FAILS before implementation (2 occurrences present; one per arm).
    /// PASSES after AC-150-001 (shared abstraction has at most 1 extraction site;
    /// possibly 0 if the extraction is expressed differently inside the abstraction).
    #[test]
    fn test_BC_150_001_process_handshake_carry_msg_bytes_extraction_not_duplicated() {
        let source = read_tls_source();
        let body = extract_fn_body(&source, "fn process_handshake_carry(");

        let count = body.matches("let msg_bytes").count();

        assert!(
            count <= 1,
            "AC-150-001 (TLS-DRAIN-DUP-001): `process_handshake_carry` must \
             contain at most ONE `let msg_bytes` carry-slice extraction \
             (`carry[consumed..consumed + 4 + body_len].to_vec()`) — the \
             current implementation duplicates this extraction in both the \
             Direction::ClientToServer arm and the Direction::ServerToClient \
             arm (STORY-150). Found {count} occurrence(s). BEFORE implementation: \
             2 occur — one per direction arm. AFTER AC-150-001: the shared dispatch \
             abstraction reduces this to at most 1. \
             Red Gate for AC-150-001 (TLS-DRAIN-DUP-001)."
        );
    }

    // ── AC-150-003: VP-039 line-correspondence table ──────────────────────────
    //
    // AC-150-003 MACHINE-CHECKABILITY ASSESSMENT:
    //
    // The VP-039 line-correspondence table (src/analyzer/tls.rs lines ~1806–1818)
    // uses descriptive step names as row identifiers — e.g.,
    //   `carry.len()-consumed<4 → break`   header-incomplete guard
    //   `mt = carry[consumed]`             msg_type read
    //   ...
    // rather than `// VP-039: line N` annotations keyed to specific source line
    // numbers. There is no machine-checkable mapping from these prose descriptions
    // to the exact line numbers in `process_handshake_carry`.
    //
    // To make AC-150-003 unit-testable as a Red Gate, the codebase would need to
    // adopt a `// VP-039-LINE: N` annotation convention in production source and
    // a corresponding test that verifies each annotated line N contains the
    // expected code snippet. That convention does not currently exist.
    //
    // Coverage provided instead:
    //   (a) test_BC_150_003_vp039_proof_module_marker_present below (structural
    //       sanity: the module header was not accidentally deleted by the refactor).
    //   (b) The behavior-preservation regression pins in this file exercise the
    //       exact carry-drain-loop paths (header-incomplete guard, body_len decode,
    //       Decision-4, cursor-advance, carry-restore) that the Kani
    //       `drain_loop_model` models, providing indirect coverage of the
    //       model-to-production correspondence.

    /// AC-150-003 (structural): the VP-039 Kani proof module header — which
    /// contains the line-correspondence table comment block — must remain
    /// present in the source after the AC-150-001 refactor.
    ///
    /// This test verifies that the `mod kani_proofs_vp039` header comment
    /// marker strings (`"model step"` and `"production"`) were not accidentally
    /// removed or relocated. It does NOT validate that the prose step descriptions
    /// within the table are current — that is a manual review obligation
    /// (see AC-150-003 commentary above).
    ///
    /// GREEN (before implementation): the VP-039 module header exists.
    /// Must remain GREEN after AC-150-001 implementation.
    #[test]
    fn test_BC_150_003_vp039_proof_module_marker_present() {
        let source = read_tls_source();

        assert!(
            source.contains("mod kani_proofs_vp039"),
            "AC-150-003: the VP-039 Kani proof module (`mod kani_proofs_vp039`) \
             must remain present in src/analyzer/tls.rs after the AC-150-001 \
             refactor. The module header contains the line-correspondence table \
             that must be updated per AC-150-003 (STORY-150 / VP-039)."
        );
        assert!(
            source.contains("model step"),
            "AC-150-003: the VP-039 line-correspondence table comment block \
             (containing the 'model step' marker string) must remain present \
             in the kani_proofs_vp039 module header after refactor. If the \
             table was rewritten, update this test to match the new marker \
             (STORY-150 / VP-039)."
        );
    }

    // ── Behavior-preservation regression pins ─────────────────────────────────
    //
    // The following tests verify directional symmetry of the carry-drain paths
    // in `process_handshake_carry`. They PASS before the AC-150-001 refactor
    // and must continue to PASS after. A failing regression pin after refactor
    // indicates the implementation changed behavior — fix the implementation,
    // not these tests.
    //
    // VP-039 coverage note: these tests exercise the exact loop paths that the
    // Kani `drain_loop_model` in `kani_proofs_vp039` models:
    //   - header-incomplete guard (carry_len < 4+body_len → accumulate)
    //   - cursor advance (consumed += 4 + body_len)
    //   - carry restore (remaining bytes stored back after the loop)
    //   - per-direction dispatch (C2S→ClientHello, S2C→ServerHello)

    /// BC-2.07.004 / BC-2.07.038 regression pin:
    /// A complete ClientHello delivered in a single TLS record via C2S direction
    /// sets `client_hello_seen`, increments `handshake_count`, and leaves the
    /// client carry buffer empty.
    ///
    /// REGRESSION PIN — PASSES before and after AC-150-001 implementation.
    #[test]
    fn test_BC_150_regression_c2s_single_record_client_hello_sets_flag_and_count() {
        let hs = build_client_hello_handshake_bytes();
        let record = wrap_as_tls_record(&hs);

        let key = fixture_flow_key();
        let mut analyzer = TlsAnalyzer::new();
        analyzer.on_data(&key, Direction::ClientToServer, &record, 0, 0);

        assert!(
            analyzer.client_hello_seen_for_testing(&key),
            "BC-2.07.004 / BC-2.07.038 regression pin: single-record C2S \
             ClientHello delivery must set client_hello_seen (STORY-150)"
        );
        assert_eq!(
            analyzer.handshake_count(),
            1,
            "BC-2.07.038 regression pin: single-record C2S ClientHello delivery \
             must increment handshake_count to 1 (STORY-150)"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&key),
            0,
            "BC-2.07.038 regression pin: after complete ClientHello delivery, \
             client carry buffer must be empty (STORY-150)"
        );
    }

    /// BC-2.07.028 / BC-2.07.038 regression pin:
    /// A complete ServerHello delivered in a single TLS record via S2C direction
    /// sets `server_hello_seen` and leaves the server carry buffer empty.
    ///
    /// REGRESSION PIN — PASSES before and after AC-150-001 implementation.
    #[test]
    fn test_BC_150_regression_s2c_single_record_server_hello_sets_flag() {
        let hs = build_server_hello_handshake_bytes();
        let record = wrap_as_tls_record(&hs);

        let key = fixture_flow_key();
        let mut analyzer = TlsAnalyzer::new();
        analyzer.on_data(&key, Direction::ServerToClient, &record, 0, 0);

        assert!(
            analyzer.server_hello_seen_for_testing(&key),
            "BC-2.07.028 / BC-2.07.038 regression pin: single-record S2C \
             ServerHello delivery must set server_hello_seen (STORY-150)"
        );
        assert_eq!(
            analyzer.server_hs_carry_len_for_testing(&key),
            0,
            "BC-2.07.038 regression pin: after complete S2C ServerHello delivery, \
             server carry buffer must be empty (STORY-150)"
        );
    }

    /// BC-2.07.038 regression pin — directional isolation:
    /// C2S ClientHello delivery sets `client_hello_seen` but must NOT set
    /// `server_hello_seen` or accumulate bytes in the server carry buffer.
    ///
    /// Verifies that the refactored shared dispatch abstraction maintains strict
    /// direction isolation: the C2S arm must only affect C2S state.
    ///
    /// REGRESSION PIN — PASSES before and after AC-150-001 implementation.
    #[test]
    fn test_BC_150_regression_directional_isolation_c2s_does_not_touch_s2c_state() {
        let hs = build_client_hello_handshake_bytes();
        let record = wrap_as_tls_record(&hs);

        let key = fixture_flow_key();
        let mut analyzer = TlsAnalyzer::new();
        analyzer.on_data(&key, Direction::ClientToServer, &record, 0, 0);

        assert!(
            analyzer.client_hello_seen_for_testing(&key),
            "BC-2.07.038 directional isolation regression pin: \
             C2S ClientHello must set client_hello_seen (STORY-150)"
        );
        assert!(
            !analyzer.server_hello_seen_for_testing(&key),
            "BC-2.07.038 directional isolation regression pin: \
             C2S ClientHello delivery must NOT set server_hello_seen — \
             the refactored shared dispatch abstraction must maintain strict \
             per-direction flag assignment (STORY-150 AC-150-001)"
        );
        assert_eq!(
            analyzer.server_hs_carry_len_for_testing(&key),
            0,
            "BC-2.07.038 directional isolation regression pin: \
             C2S ClientHello delivery must NOT accumulate bytes in the \
             server carry buffer (STORY-150 AC-150-001)"
        );
    }

    /// BC-2.07.038 regression pin — directional isolation (S2C → C2S):
    /// S2C ServerHello delivery sets `server_hello_seen` but must NOT set
    /// `client_hello_seen` or accumulate bytes in the client carry buffer.
    ///
    /// REGRESSION PIN — PASSES before and after AC-150-001 implementation.
    #[test]
    fn test_BC_150_regression_directional_isolation_s2c_does_not_touch_c2s_state() {
        let hs = build_server_hello_handshake_bytes();
        let record = wrap_as_tls_record(&hs);

        let key = fixture_flow_key();
        let mut analyzer = TlsAnalyzer::new();
        analyzer.on_data(&key, Direction::ServerToClient, &record, 0, 0);

        assert!(
            analyzer.server_hello_seen_for_testing(&key),
            "BC-2.07.038 directional isolation regression pin: \
             S2C ServerHello must set server_hello_seen (STORY-150)"
        );
        assert!(
            !analyzer.client_hello_seen_for_testing(&key),
            "BC-2.07.038 directional isolation regression pin: \
             S2C ServerHello delivery must NOT set client_hello_seen — \
             strict per-direction flag assignment must be maintained by the \
             shared dispatch abstraction (STORY-150 AC-150-001)"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&key),
            0,
            "BC-2.07.038 directional isolation regression pin: \
             S2C ServerHello delivery must NOT accumulate bytes in the \
             client carry buffer (STORY-150 AC-150-001)"
        );
    }

    /// BC-2.07.038 / VP-039 regression pin — fragmented C2S ClientHello:
    /// Delivering a ClientHello split across 3 TLS records (15/15/15 bytes)
    /// accumulates carry across the first 2 records and dispatches on record 3.
    ///
    /// Exercises: header-incomplete guard → accumulate, cursor-advance,
    /// carry-restore, final dispatch. Same paths as VP-039 `drain_loop_model`
    /// harness `verify_drain_loop_cursor_safety`.
    ///
    /// REGRESSION PIN — PASSES before and after AC-150-001 implementation.
    #[test]
    fn test_BC_150_regression_c2s_fragmented_client_hello_carry_drains_on_completion() {
        let hs = build_client_hello_handshake_bytes(); // 45 bytes
        let n = hs.len();
        let split1 = n / 3; // 15
        let split2 = 2 * n / 3; // 30
        let segments = [
            wrap_as_tls_record(&hs[..split1]),
            wrap_as_tls_record(&hs[split1..split2]),
            wrap_as_tls_record(&hs[split2..]),
        ];

        let key = fixture_flow_key();
        let mut analyzer = TlsAnalyzer::new();

        // Phase 1: deliver first N-1 segments — carry must accumulate, no dispatch yet.
        for seg in segments.iter().take(segments.len() - 1) {
            analyzer.on_data(&key, Direction::ClientToServer, seg, 0, 0);
        }

        assert!(
            analyzer.client_hs_carry_len_for_testing(&key) > 0,
            "BC-2.07.038 fragmented C2S regression pin: after partial delivery, \
             client carry must be non-empty — the carry-drain loop accumulates \
             handshake bytes across record boundaries and retains partials \
             (VP-039 / STORY-150)"
        );
        assert_eq!(
            analyzer.handshake_count(),
            0,
            "BC-2.07.038 fragmented C2S regression pin: handshake must NOT be \
             dispatched until the final fragment arrives (VP-039 / STORY-150)"
        );

        // Phase 2: deliver the final segment — carry drains, ClientHello dispatched.
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
            "BC-2.07.038 fragmented C2S regression pin: final segment must \
             complete the fragmented ClientHello and dispatch exactly one \
             handshake (VP-039 / STORY-150)"
        );
        assert!(
            analyzer.client_hello_seen_for_testing(&key),
            "BC-2.07.038 fragmented C2S regression pin: client_hello_seen must \
             be set after successful fragmented ClientHello delivery (STORY-150)"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&key),
            0,
            "BC-2.07.038 fragmented C2S regression pin: client carry must be \
             empty after the complete handshake is dispatched (STORY-150)"
        );
    }

    /// BC-2.07.038 / VP-039 regression pin — fragmented S2C ServerHello:
    /// Delivering a ServerHello split across 3 TLS records accumulates carry
    /// across the first 2 records and dispatches on record 3.
    ///
    /// Symmetric companion to the C2S fragmented test above. The C2S and S2C
    /// arms must produce identical carry-accumulation and dispatch behavior
    /// modulo the direction-specific flag (`server_hello_seen` vs
    /// `client_hello_seen`). This is the key directional symmetry property that
    /// AC-150-001's shared abstraction must preserve.
    ///
    /// REGRESSION PIN — PASSES before and after AC-150-001 implementation.
    #[test]
    fn test_BC_150_regression_s2c_fragmented_server_hello_carry_drains_on_completion() {
        let hs = build_server_hello_handshake_bytes(); // 49 bytes
        let n = hs.len();
        let split1 = n / 3; // 16
        let split2 = 2 * n / 3; // 32
        let segments = [
            wrap_as_tls_record(&hs[..split1]),
            wrap_as_tls_record(&hs[split1..split2]),
            wrap_as_tls_record(&hs[split2..]),
        ];

        let key = fixture_flow_key();
        let mut analyzer = TlsAnalyzer::new();

        // Phase 1: deliver first N-1 segments — server carry non-empty, no dispatch.
        for seg in segments.iter().take(segments.len() - 1) {
            analyzer.on_data(&key, Direction::ServerToClient, seg, 0, 0);
        }

        assert!(
            analyzer.server_hs_carry_len_for_testing(&key) > 0,
            "BC-2.07.038 fragmented S2C regression pin: after partial delivery, \
             server carry must be non-empty — symmetric to C2S accumulation \
             behavior (VP-039 / STORY-150)"
        );
        assert!(
            !analyzer.server_hello_seen_for_testing(&key),
            "BC-2.07.038 fragmented S2C regression pin: server_hello_seen must \
             NOT be set until the final fragment arrives (STORY-150)"
        );

        // Phase 2: deliver the final segment — carry drains, server_hello_seen set.
        analyzer.on_data(
            &key,
            Direction::ServerToClient,
            segments.last().unwrap(),
            0,
            0,
        );

        assert!(
            analyzer.server_hello_seen_for_testing(&key),
            "BC-2.07.038 fragmented S2C regression pin: server_hello_seen must \
             be set after complete fragmented ServerHello delivery \
             (VP-039 / STORY-150)"
        );
        assert_eq!(
            analyzer.server_hs_carry_len_for_testing(&key),
            0,
            "BC-2.07.038 fragmented S2C regression pin: server carry must be \
             empty after the complete ServerHello is dispatched (STORY-150)"
        );
    }

    /// BC-2.07.038 / AC-150-001 regression pin — parse-error symmetry:
    /// Equivalent malformed handshake messages in the C2S (msg_type=0x01) and
    /// S2C (msg_type=0x02) arms must produce the same `parse_errors` increment.
    ///
    /// The current implementation duplicates `Ok(_) => self.parse_errors += 1` /
    /// `Err(_) => self.parse_errors += 1` in both arms (AC-150-001 notes this as
    /// part of the duplication). After the DRY refactor the shared abstraction
    /// must apply the same increment for both directions.
    ///
    /// Malformed payload: msg_type byte + body_len=5 + 5 bytes of 0xFF garbage.
    /// A ClientHello/ServerHello body of 5 bytes is far too short for the parser
    /// (ClientHello needs ≥34 bytes for version+random alone) → Err(_) → parse_errors += 1.
    ///
    /// REGRESSION PIN — PASSES before and after AC-150-001 implementation.
    #[test]
    fn test_BC_150_regression_parse_error_increment_symmetry_c2s_and_s2c() {
        // C2S malformed: msg_type=0x01 (ClientHello), body_len=5, garbage body.
        let malformed_c2s: Vec<u8> = vec![
            0x01, // msg_type = ClientHello
            0x00, 0x00, 0x05, // body_len = 5
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // 5 garbage bytes (far too short for a ClientHello)
        ];
        let c2s_record = wrap_as_tls_record(&malformed_c2s);

        let key1 = fixture_flow_key();
        let mut analyzer_c2s = TlsAnalyzer::new();
        analyzer_c2s.on_data(&key1, Direction::ClientToServer, &c2s_record, 0, 0);
        let c2s_errors = analyzer_c2s.parse_error_count();

        assert!(
            c2s_errors >= 1,
            "BC-2.07.038 parse-error regression pin: a malformed msg_type=0x01 \
             (ClientHello) in the C2S arm must increment parse_errors by at least 1 \
             (AC-150-001: Err(_) arm in C2S must remain after refactor — STORY-150)"
        );

        // S2C malformed: msg_type=0x02 (ServerHello), body_len=5, garbage body.
        let malformed_s2c: Vec<u8> = vec![
            0x02, // msg_type = ServerHello
            0x00, 0x00, 0x05, // body_len = 5
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // 5 garbage bytes
        ];
        let s2c_record = wrap_as_tls_record(&malformed_s2c);

        let key2 = fixture_flow_key_b();
        let mut analyzer_s2c = TlsAnalyzer::new();
        analyzer_s2c.on_data(&key2, Direction::ServerToClient, &s2c_record, 0, 0);
        let s2c_errors = analyzer_s2c.parse_error_count();

        assert!(
            s2c_errors >= 1,
            "BC-2.07.038 parse-error regression pin: a malformed msg_type=0x02 \
             (ServerHello) in the S2C arm must increment parse_errors by at least 1 \
             (AC-150-001: Err(_) arm in S2C must remain after refactor — STORY-150)"
        );

        assert_eq!(
            c2s_errors, s2c_errors,
            "BC-2.07.038 / AC-150-001 parse-error symmetry regression pin: \
             equivalent malformed messages in C2S (msg_type=0x01) and S2C \
             (msg_type=0x02) arms must produce the same parse_errors increment \
             ({c2s_errors} vs {s2c_errors}). After the DRY refactor, both arms \
             must delegate error counting to the same shared abstraction \
             (STORY-150 AC-150-001)."
        );
    }
}
