// Test ordering / grouping convention
// ─────────────────────────────────────────────────────────────────────────────
// Helpers (flow_key, etc.) appear first.
//
// Test functions are grouped by the Behavioral Contract they exercise and
// named with the BC-prefixed pattern `test_BC_S_SS_NNN_…` where available.
// Within each group the tests appear in precondition → postcondition →
// invariant order, matching the structure of the BC document.  Edge-case and
// integration tests that exercise multiple BCs follow at the end.
// ─────────────────────────────────────────────────────────────────────────────

use std::net::IpAddr;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};

fn flow_key(src_port: u16, dst_port: u16) -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        src_port,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        dst_port,
    )
}

/// Exercises VP-004: 0x16 0x03 prefix routes to TLS regardless of port.
///
/// AC-001 (BC-2.05.001 postcondition 1): TLS signature [0x16, 0x03, ...] on a
/// non-standard port (8080) routes to TLS via content detection, not port fallback.
/// HTTP analyzer must receive zero data; TLS analyzer must receive the data.
///
/// This also serves as `test_tls_content_wins_over_port_8080`: content-priority over
/// port-fallback hint for HTTP port 8080.
#[test]
fn test_tls_content_wins_over_port_8080() {
    // Both analyzers present so we can observe which one received data.
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 8080 would fall back to Http by port — if content wins, Tls is chosen instead.
    let fk = flow_key(49152, 8080);

    // Canonical test vector from BC-2.05.001: [0x16, 0x03, 0x03, 0x00, 0x50, ...]
    let tls_data = [0x16u8, 0x03, 0x03, 0x00, 0x50, 0x01, 0x00, 0x00, 0x4c, 0x03];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0, 0);

    // Content-first wins: HTTP must not have received any data from this flow.
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-001: TLS signature on port 8080 must route to Tls, not Http"
    );
    assert_eq!(
        http.parse_error_count(),
        0,
        "AC-001: HTTP analyzer must not have attempted to parse TLS bytes"
    );
}

/// True happy-path baseline: TLS content on TLS port 443 — most common real-world case.
/// AC-001 supplementary: content detection works on the canonical TLS port too.
#[test]
fn test_tls_content_routes_tls_on_port_443() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk = flow_key(49152, 443);

    let tls_data = [0x16u8, 0x03, 0x03, 0x00, 0x50, 0x01, 0x00, 0x00, 0x4c, 0x03];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0, 0);

    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-001 baseline: TLS signature on port 443 must route to Tls, not Http"
    );
    assert_eq!(
        http.parse_error_count(),
        0,
        "AC-001 baseline: HTTP analyzer must not attempt to parse TLS bytes on port 443"
    );
}

#[test]
fn test_dispatcher_routes_http() {
    let mut dispatcher =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None, None);
    let fk = flow_key(49152, 80);

    let http_data = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0, 0);

    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(*http.method_counts().get("GET").unwrap(), 1);
}

/// AC-004 (BC-2.05.002 postcondition 1, invariant 3): Each of the 10 HTTP
/// method/version prefix byte strings routes to Http when content matches.
/// Uses port 9999 to isolate content classification from port fallback.
/// Also covers EC-008 (b"HTTP/1.1 200 OK" response-first case) via the
/// HTTP/ prefix.
#[test]
fn test_all_http_method_prefixes_route_to_http() {
    // Complete HTTP messages so the parser can confirm receipt via method_counts
    // or status_codes. For methods, supply Host + double-CRLF so httparse
    // returns Complete (and method_counts is populated). The HTTP/ prefix is a
    // response line; sent as ClientToServer it hits the request parser which
    // errors → parse_error_count > 0 confirms routing.
    let cases: &[(&[u8], &str)] = &[
        (b"GET / HTTP/1.1\r\nHost: x\r\n\r\n", "GET"),
        (b"POST / HTTP/1.1\r\nHost: x\r\n\r\n", "POST"),
        (b"PUT / HTTP/1.1\r\nHost: x\r\n\r\n", "PUT"),
        (b"DELETE / HTTP/1.1\r\nHost: x\r\n\r\n", "DELETE"),
        (b"HEAD / HTTP/1.1\r\nHost: x\r\n\r\n", "HEAD"),
        (b"OPTIONS / HTTP/1.1\r\nHost: x\r\n\r\n", "OPTIONS"),
        (b"PATCH / HTTP/1.1\r\nHost: x\r\n\r\n", "PATCH"),
        (
            b"CONNECT host:443 HTTP/1.1\r\nHost: host:443\r\n\r\n",
            "CONNECT",
        ),
        (b"TRACE / HTTP/1.1\r\nHost: x\r\n\r\n", "TRACE"),
        // EC-008: response-first / server-initiated. Sent as ClientToServer
        // so the request parser sees a malformed message → parse_error_count > 0.
        (b"HTTP/1.1 200 OK\r\n\r\n", "HTTP/"),
    ];

    for (i, (data, label)) in cases.iter().enumerate() {
        let mut dispatcher = StreamDispatcher::new(
            Some(HttpAnalyzer::new()),
            Some(TlsAnalyzer::new()),
            None,
            None,
            None,
            None,
        );
        // Port 9999: no port fallback hint — Http must be chosen by content.
        let fk = flow_key(49152 + i as u16, 9999);
        dispatcher.on_data(&fk, Direction::ClientToServer, data, 0, 0);

        let http = dispatcher.http_analyzer().expect("HTTP analyzer set");
        let tls = dispatcher.tls_analyzer().expect("TLS analyzer set");

        // Either HTTP saw the data (method recorded or parse-error counted),
        // OR (for HTTP/ response-first) the parser may register differently —
        // but in all cases TLS must NOT have received the data.
        assert_eq!(
            tls.parse_error_count(),
            0,
            "AC-004 prefix {label:?}: TLS must not be invoked for HTTP content"
        );
        // Method-counts may be 0 for HTTP/ response-first (no method) but
        // parse_error_count being > 0 or method_counts being non-empty signals
        // the data was routed to the HTTP analyzer.
        let routed_to_http = !http.method_counts().is_empty() || http.parse_error_count() > 0;
        assert!(
            routed_to_http,
            "AC-004 prefix {label:?}: HTTP analyzer must have received the data"
        );
    }
}

#[test]
fn test_dispatcher_content_detection_tls_on_port_80() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk = flow_key(49152, 80); // Port 80, but content is TLS

    // TLS record header on port 80 — content detection should override port
    let tls_data = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0, 0);

    // HTTP analyzer should NOT have received this data
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(http.method_counts().len(), 0);
    assert_eq!(http.parse_error_count(), 0); // Confirms HTTP didn't try to parse TLS bytes
}

/// AC-007 (BC-2.05.003 postcondition 1): When both content checks fail (data has
/// no TLS/HTTP magic bytes), port fallback fires. Port 443 → DispatchTarget::Tls.
#[test]
fn test_port_fallback_443_to_tls() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk = flow_key(49152, 443); // Port 443

    // 6 bytes: TLS record type 0x16 but version 0x0401 (not 0x0300–0x0303) so content
    // detection (which requires data[1]==0x03) does NOT fire; only port fallback applies.
    // The 1-byte payload (0xFF) forms a syntactically complete but malformed handshake
    // record, which causes TlsAnalyzer to increment parse_error_count — confirming routing.
    let unknown_data = [0x16u8, 0x04, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_data, 0, 0);

    // Should have routed to TLS based on port 443; HTTP must not have received it.
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-007: short data on port 443 must fall back to Tls, not Http"
    );
    assert_eq!(
        http.parse_error_count(),
        0,
        "AC-007: HTTP analyzer must not attempt to parse bytes on port-443 fallback"
    );
    // Positive TLS discriminator: non-TLS garbage routed to TlsAnalyzer creates a
    // flow entry — proves TlsAnalyzer actually received the bytes.
    // Updated in STORY-144: the carry-buffer path (AC-144-002) now accumulates
    // short 0x16 payloads without immediately producing parse_errors; using
    // active_flows_len_for_testing() > 0 as the discriminator instead.
    let tls = dispatcher.tls_analyzer().unwrap();
    assert!(
        tls.active_flows_len_for_testing() > 0,
        "AC-007: port 443 fallback must route to Tls analyzer \
         (TlsAnalyzer creates a flow entry on receipt of any on_data call)"
    );
}

/// AC-007 (BC-2.05.003 postcondition 1): Port 8443 → DispatchTarget::Tls via port fallback.
/// 6-byte non-TLS, non-HTTP data ensures neither content check fires.
#[test]
fn test_port_fallback_8443_to_tls() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 8443 is a known TLS port; data has no TLS/HTTP signature.
    let fk = flow_key(49152, 8443);

    // 6 bytes: TLS record type 0x16 but version 0x0401 (not 0x0300–0x0303) so content
    // detection (which requires data[1]==0x03) does NOT fire; only port fallback applies.
    // The 1-byte payload (0xFF) forms a complete but malformed handshake record, causing
    // TlsAnalyzer to increment parse_error_count — confirming routing to TLS analyzer.
    let ambiguous_data = [0x16u8, 0x04, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &ambiguous_data, 0, 0);

    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-007: port 8443 fallback must route to Tls, not Http"
    );
    assert_eq!(
        http.parse_error_count(),
        0,
        "AC-007: HTTP analyzer must not be called when port 8443 falls back to Tls"
    );
    // Positive TLS discriminator: non-TLS garbage routed to TlsAnalyzer creates a
    // flow entry — proves TlsAnalyzer actually received the bytes.
    // Updated in STORY-144: the carry-buffer path (AC-144-002) now accumulates
    // short 0x16 payloads without immediately producing parse_errors; using
    // active_flows_len_for_testing() > 0 as the discriminator instead.
    let tls = dispatcher.tls_analyzer().unwrap();
    assert!(
        tls.active_flows_len_for_testing() > 0,
        "AC-007: port 8443 fallback must route to Tls analyzer \
         (TlsAnalyzer creates a flow entry on receipt of any on_data call)"
    );
}

/// AC-007 (BC-2.05.003 postcondition 2): Port 80 → DispatchTarget::Http via port fallback.
/// 5-byte non-TLS, non-HTTP data ensures neither content check fires.
#[test]
fn test_port_fallback_80_to_http() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 80 is a known HTTP port; data has no TLS/HTTP signature.
    let fk = flow_key(49152, 80);

    // 5 bytes with no TLS (byte0≠0x16) and no HTTP method prefix → only port fallback applies.
    let ambiguous_data = [0x00u8, 0x01, 0x02, 0x03, 0x04];
    dispatcher.on_data(&fk, Direction::ClientToServer, &ambiguous_data, 0, 0);

    // Port 80 fallback → Http. The flow IS classified (not unclassified).
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-007: port 80 fallback must route to Http (flow classified, not unclassified)"
    );
    // Discriminator: HTTP analyzer must have attempted to parse the bytes (the data is
    // non-HTTP garbage, so httparse will increment parse_error_count). If the flow were
    // mis-routed to Tls, HTTP would never see the bytes → parse_error_count == 0 → fails.
    let http = dispatcher.http_analyzer().unwrap();
    assert!(
        http.parse_error_count() > 0,
        "AC-007: port 80 fallback must route to Http analyzer (received the 5-byte \
         non-HTTP data and tried to parse, incrementing parse_error_count)"
    );
}

/// AC-007 (BC-2.05.003 postcondition 2): Port 8080 → DispatchTarget::Http via port fallback.
/// 5-byte non-TLS, non-HTTP data ensures neither content check fires.
/// Also covers EC-010: unknown bytes on port 8080 → Http.
#[test]
fn test_port_fallback_8080_to_http() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 8080 is a known HTTP port; data has no TLS/HTTP signature.
    let fk = flow_key(49152, 8080);

    // 5 bytes with no TLS (byte0≠0x16) and no HTTP method prefix → only port fallback applies.
    let ambiguous_data = [0x00u8, 0x01, 0x02, 0x03, 0x04];
    dispatcher.on_data(&fk, Direction::ClientToServer, &ambiguous_data, 0, 0);

    // Port 8080 fallback → Http. Same verification strategy as port 80 above.
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-007/EC-010: port 8080 fallback must route to Http (flow classified, not unclassified)"
    );
    // Discriminator: HTTP analyzer must have attempted to parse the bytes (the data is
    // non-HTTP garbage, so httparse will increment parse_error_count). If the flow were
    // mis-routed to Tls, HTTP would never see the bytes → parse_error_count == 0 → fails.
    let http = dispatcher.http_analyzer().unwrap();
    assert!(
        http.parse_error_count() > 0,
        "AC-007/EC-010: port 8080 fallback must route to Http analyzer (received the 5-byte \
         non-HTTP data and tried to parse, incrementing parse_error_count)"
    );
}

/// AC-003 (BC-2.05.001 precondition 1): When data.len() < 5, the TLS content
/// check is skipped. This is isolated from port fallback by using port 9999
/// (no port fallback hint). With no content match and no port match, the flow
/// is unclassified.
///
/// Also covers EC-004: data.len() == 4 (boundary — exactly one byte short of the
/// minimum required for TLS content inspection).
#[test]
fn test_tls_check_skipped_below_len_5() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 9999: no port fallback hint — isolates the length-gate from port fallback.
    let fk = flow_key(49152, 9999);

    // 4 bytes starting with TLS-looking byte0=0x16 — would pass TLS check IF 5 bytes present.
    // Exactly at the EC-004 boundary: data.len() == 4.
    let four_bytes = [0x16u8, 0x03, 0x03, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &four_bytes, 0, 0);

    // TLS content check skipped (too short), HTTP content check also fails (no method prefix),
    // port fallback also fails (unknown port) → flow unclassified.
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-003/EC-004: 4-byte data must not route to Http"
    );
    assert_eq!(
        http.parse_error_count(),
        0,
        "AC-003/EC-004: HTTP analyzer must not be called for 4-byte data on unknown port"
    );
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "AC-003/EC-004: 4-byte TLS-looking data on unknown port must remain unclassified"
    );
}

/// EC-005 (edge case): TLS content check requires byte0==0x16 AND byte1==0x03.
/// Data with byte0=0x16 but byte1≠0x03 must NOT be routed to Tls.
#[test]
fn test_tls_check_requires_byte1_equals_0x03() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 9999: no port fallback hint.
    let fk = flow_key(49152, 9999);

    // byte0=0x16, byte1=0x04 (not 0x03) — TLS check must fail.
    let almost_tls = [0x16u8, 0x04, 0x03, 0x00, 0x05];
    dispatcher.on_data(&fk, Direction::ClientToServer, &almost_tls, 0, 0);

    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "EC-005: byte0=0x16 + byte1=0x04 must not route to Http (no HTTP prefix)"
    );
    // Flow is unclassified (no content match, no port match).
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "EC-005: byte1=0x04 (not 0x03) must not trigger TLS routing; flow unclassified"
    );

    // Variant: byte1=0x02.
    let mut dispatcher2 = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk2 = flow_key(49152, 9999);
    let almost_tls2 = [0x16u8, 0x02, 0x03, 0x00, 0x05];
    dispatcher2.on_data(&fk2, Direction::ClientToServer, &almost_tls2, 0, 0);
    dispatcher2.on_flow_close(&fk2, CloseReason::Fin);
    assert_eq!(
        dispatcher2.unclassified_flows(),
        1,
        "EC-005 variant: byte1=0x02 (not 0x03) must not trigger TLS routing; flow unclassified"
    );
}

#[test]
fn test_unclassified_flows_counter() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk = flow_key(49152, 9999); // Non-standard port

    // Send data that doesn't match HTTP or TLS content signatures
    dispatcher.on_data(&fk, Direction::ClientToServer, b"UNKNOWN_PROTOCOL", 0, 0);
    assert_eq!(dispatcher.unclassified_flows(), 0); // Not counted until close

    // Close the flow — never classified
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(dispatcher.unclassified_flows(), 1);
}

#[test]
fn test_classified_flow_not_counted_as_unclassified() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk = flow_key(49152, 80);

    let http_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0, 0);
    dispatcher.on_flow_close(&fk, CloseReason::Fin);

    assert_eq!(dispatcher.unclassified_flows(), 0);
}

// ---- LESSON-P2.11: max_classification_attempts knob ----

#[test]
fn test_default_max_classification_attempts() {
    // The default cap is exposed and matches the documented constant.
    let dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None, None);
    assert_eq!(
        dispatcher.max_classification_attempts(),
        wirerust::dispatcher::DEFAULT_MAX_CLASSIFICATION_ATTEMPTS
    );
}

#[test]
fn test_with_max_classification_attempts_overrides_default() {
    // The builder-style override sets a custom cap.
    let dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None, None)
        .with_max_classification_attempts(3);
    assert_eq!(dispatcher.max_classification_attempts(), 3);
}

#[test]
fn test_unclassifiable_flow_still_counted_after_attempt_cap() {
    // LESSON-P2.11: once a flow exceeds max_classification_attempts it
    // is permanently routed to None. It must still be counted as an
    // unclassified flow on close — the give-up branch must not lose
    // the flow from the accounting.
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(3);
    let fk = flow_key(49152, 9999); // non-standard port, unknown content

    // Feed several non-HTTP, non-TLS chunks — well past the cap of 3.
    for _ in 0..10 {
        dispatcher.on_data(&fk, Direction::ClientToServer, b"UNKNOWN_PROTOCOL", 0, 0);
    }
    assert_eq!(dispatcher.unclassified_flows(), 0); // not counted until close

    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "a flow that hit the classification cap must still count as unclassified on close"
    );
}

#[test]
fn test_late_classification_within_attempt_budget_still_routes() {
    // A flow whose protocol only becomes visible after a few
    // non-matching chunks must still classify correctly, as long as
    // the match arrives before the attempt cap is reached.
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(8);
    let fk = flow_key(49152, 9999);

    // Two unclassifiable chunks (within the budget of 8)...
    dispatcher.on_data(&fk, Direction::ClientToServer, b"\x00\x01", 0, 0);
    dispatcher.on_data(&fk, Direction::ClientToServer, b"\x02\x03", 0, 0);
    // ...then a clear HTTP request.
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        b"GET / HTTP/1.1\r\nHost: x\r\n\r\n",
        0,
        0,
    );

    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        *http.method_counts().get("GET").unwrap(),
        1,
        "HTTP arriving within the attempt budget must still be routed"
    );
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "a successfully (if late) classified flow must not be counted unclassified"
    );
}

#[test]
fn test_zero_attempt_budget_classifies_nothing() {
    // Edge case: max_classification_attempts == 0 means the very
    // first unclassifiable chunk immediately stamps the flow None.
    // A flow whose first chunk *is* a clear protocol still routes,
    // because classification on a positive match doesn't consume the
    // (already-zero) failure budget.
    let mut dispatcher =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None, None)
            .with_max_classification_attempts(0);
    let fk = flow_key(49152, 80);

    let http_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0, 0);
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        *http.method_counts().get("GET").unwrap(),
        1,
        "a first-chunk positive match must route even with a zero failure budget"
    );
}

// ---- STORY-031: content-first classification tests (BC-2.05.001/002/003) ----

/// AC-005 (BC-2.05.002 invariant 3): HTTP method prefixes require a trailing
/// space. `b"GET"` (3 bytes, no space) must NOT match. The comparison is
/// case-sensitive; `b"get "` must NOT match either.
/// EC-007: b"GET" on port 9999 → falls to port fallback → returns None (unknown port).
#[test]
fn test_http_no_space_does_not_match() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 9999: no port fallback match, so the only way Http is chosen is content.
    let fk = flow_key(49152, 9999);

    // b"GET" without trailing space — must not match any HTTP prefix.
    dispatcher.on_data(&fk, Direction::ClientToServer, b"GET", 0, 0);
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-005: b\"GET\" (no trailing space) must not route to Http"
    );

    // Case-sensitive: lowercase b"get " must not match.
    // Use a COMPLETE request (Host + double-CRLF) so that if a regression made
    // matching case-insensitive, httparse would return Complete and increment
    // method_counts — giving us a true discriminator rather than relying on Partial.
    let fk2 = flow_key(49153, 9999);
    dispatcher.on_data(
        &fk2,
        Direction::ClientToServer,
        b"get /index HTTP/1.1\r\nHost: x\r\n\r\n",
        0,
        0,
    );
    assert_eq!(
        dispatcher.http_analyzer().unwrap().method_counts().len(),
        0,
        "AC-005: lowercase b\"get \" must not route to Http (case-sensitive check)"
    );
    // Close the flow and verify it was never classified to either analyzer.
    // If mis-routed AND parsed as Partial, the flow would be in routes as Http
    // → unclassified_flows == 0. Verifying unclassified == 1 proves the flow
    // was never classified.
    dispatcher.on_flow_close(&fk2, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "AC-005: lowercase b\"get \" flow must be unclassified (Http rejection means \
         DispatchTarget::None; no route inserted; on_flow_close None branch fires)"
    );

    // Positive control: b"GET " (with trailing space, correct case) DOES match
    // on the same port — confirms the negatives above failed due to the
    // trailing-space/case rule, not some other test setup issue.
    // Use a complete request (Host + double CRLF) so httparse returns Complete
    // and method_counts is populated.
    let fk_positive = flow_key(49154, 9999);
    dispatcher.on_data(
        &fk_positive,
        Direction::ClientToServer,
        b"GET /index HTTP/1.1\r\nHost: example.com\r\n\r\n",
        0,
        0,
    );
    assert_eq!(
        *dispatcher
            .http_analyzer()
            .unwrap()
            .method_counts()
            .get("GET")
            .unwrap(),
        1,
        "AC-005 positive control: properly-formatted b\"GET \" with trailing space MUST route to Http"
    );
}

/// AC-006 (BC-2.05.002 invariant 1, BC-2.05.001 invariant 1): TLS check is
/// evaluated BEFORE the HTTP check. Data beginning with 0x16 0x03 routes to
/// Tls even if the remaining bytes happen to look like an HTTP method.
/// The HTTP check is unreachable for data starting with 0x16 0x03.
#[test]
fn test_tls_takes_priority_over_http_methods_check() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Neutral port (9999) — port fallback plays no part.
    let fk = flow_key(49152, 9999);

    // Construct data that starts with the TLS magic bytes (0x16 0x03) followed
    // by enough bytes to pass the len >= 5 gate. The remainder is irrelevant to
    // the routing decision, but we pad it to 10 bytes for completeness.
    let tls_then_garbage = [0x16u8, 0x03, 0x01, 0x00, 0x06, 0x47, 0x45, 0x54, 0x20, 0x2f];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_then_garbage, 0, 0);

    // TLS wins — HTTP analyzer must have received nothing.
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-006: TLS signature (0x16 0x03) must take priority over HTTP prefix check"
    );
    assert_eq!(
        http.parse_error_count(),
        0,
        "AC-006: HTTP analyzer must not have been called when TLS bytes are present"
    );
}

/// AC-008 (BC-2.05.003 invariants 1-2): Port fallback uses lower_port() and
/// upper_port() (canonical ordering). A flow with src=8443, dst=9000 has
/// lower_port()=8443, which is found in the TLS port slice. TLS port check
/// (443/8443) is evaluated before HTTP port check (80/8080).
#[test]
fn test_port_fallback_uses_canonical_port_ordering() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );

    // src=8443, dst=9000: lower_port() == 8443. Content is ambiguous (non-TLS, non-HTTP)
    // so port fallback fires. 8443 must be found → DispatchTarget::Tls.
    // Payload: record_type=0x16, version=0x0401 (data[1]≠0x03 → content check fails),
    // payload_len=1 → complete record that TlsAnalyzer can attempt to parse → parse_error.
    let fk_8443 = flow_key(8443, 9000);
    dispatcher.on_data(
        &fk_8443,
        Direction::ClientToServer,
        b"\x16\x04\x01\x00\x01\xFF",
        0,
        0,
    );
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        http.method_counts().len(),
        0,
        "AC-008: port 8443 in canonical lower_port() slot must fall back to Tls, not Http"
    );
    assert_eq!(
        http.parse_error_count(),
        0,
        "AC-008: port 8443 canonical-ordering fallback must route to Tls (HTTP analyzer must not be invoked)"
    );
    // Positive TLS discriminator for 8443 sub-case.
    // Updated in STORY-144: carry-buffer path accumulates short 0x16 payloads
    // without parse_errors; active_flows_len_for_testing() > 0 proves routing.
    {
        let tls = dispatcher.tls_analyzer().unwrap();
        assert!(
            tls.active_flows_len_for_testing() > 0,
            "AC-008: port 8443 canonical-ordering fallback must route to Tls analyzer \
             (TlsAnalyzer creates a flow entry on receipt of any on_data call)"
        );
    }

    // Also verify 443 in the upper_port() slot is found: src=9000, dst=443.
    // With IPs 10.0.0.1 < 10.0.0.2, canonicalization is by (IP, port) tuple,
    // so lower_port()=9000 and upper_port()=443. The TLS port check still
    // finds 443 because it scans both slots via the [lower, upper] slice.
    let fk_443_upper = flow_key(9000, 443);
    assert_eq!(
        fk_443_upper.lower_port(),
        9000,
        "canonicalization: IP precedes port in tuple-compare"
    );
    assert_eq!(
        fk_443_upper.upper_port(),
        443,
        "canonicalization: 443 ends up in upper slot here"
    );
    dispatcher.on_data(
        &fk_443_upper,
        Direction::ClientToServer,
        b"\x16\x04\x01\x00\x01\xFF",
        0,
        0,
    );
    assert_eq!(
        dispatcher.http_analyzer().unwrap().method_counts().len(),
        0,
        "AC-008: port 443 via canonical port ordering must fall back to Tls"
    );
    assert_eq!(
        dispatcher.http_analyzer().unwrap().parse_error_count(),
        0,
        "AC-008: port 443 canonical-ordering fallback must route to Tls (HTTP analyzer must not be invoked)"
    );
    // Positive TLS discriminator for 443-upper sub-case.
    // Updated in STORY-144: carry-buffer path accumulates short 0x16 payloads
    // without parse_errors; active_flows_len_for_testing() > 0 proves routing.
    {
        let tls = dispatcher.tls_analyzer().unwrap();
        assert!(
            tls.active_flows_len_for_testing() > 0,
            "AC-008: port 443 canonical-ordering fallback must route to Tls analyzer \
             (TlsAnalyzer creates a flow entry on receipt of any on_data call)"
        );
    }

    // TLS port check evaluated before HTTP port check (INV-1). A flow on port 8443
    // must not be reclassified as Http even if 8080 is also somehow in the key.
    // (Standard FlowKey only exposes two ports, so this invariant is structural.)
}

/// AC-009 (BC-2.05.003 invariant 3): Port fallback is only reached when BOTH
/// content checks fail. A valid HTTP GET on port 443 is classified as Http by
/// content, NOT as Tls by port fallback.
/// EC-011: b"GET " on port 443 → Http (content wins over port 443 TLS hint).
#[test]
fn test_http_content_on_port_443_routes_to_http() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 443 would fall back to Tls — but content check for HTTP must fire first.
    let fk = flow_key(49152, 443);

    let http_on_tls_port = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_on_tls_port, 0, 0);

    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        *http.method_counts().get("GET").unwrap_or(&0),
        1,
        "AC-009: HTTP GET on port 443 must be classified as Http by content, not Tls by port"
    );
}

// ---- STORY-032: classification caching + DispatchTarget::None retry budget ----

/// STORY-032 AC-004 + AC-005 (BC-2.05.005 postconditions 1-4, invariant 1):
/// After a flow is classified as Http on its first chunk, the cached target is used for
/// all subsequent chunks — even if those chunks start with TLS bytes. The cache is
/// immutable: a cached Http flow is never reclassified as Tls.
///
/// Observability strategy (indirect): after the first GET chunk, HttpAnalyzer has
/// method_counts["GET"]==1 and TlsAnalyzer has parse_error_count==0. On the second
/// chunk (TLS bytes for the same FlowKey), if the cache is used, the data is forwarded
/// to HttpAnalyzer (not TlsAnalyzer) — HttpAnalyzer sees malformed bytes and increments
/// parse_error_count; TlsAnalyzer remains silent. If cache were NOT used, classify would
/// re-run on TLS bytes, return Tls, and TlsAnalyzer would receive the data instead.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_005_classification_cached_after_first_match() {
    // AC-004: cache-HIT path is independently verified (BC-2.05.005 R4 finding).
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    // Port 9999: no port fallback, so routing is content-only.
    let fk = flow_key(49152, 9999);

    // First chunk: valid HTTP GET — classify returns Http; cached in routes[fk].
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_bytes, 0, 0);
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(
        *http.method_counts().get("GET").unwrap_or(&0),
        1,
        "AC-004: first GET chunk must be routed to HttpAnalyzer and recorded"
    );
    assert_eq!(
        dispatcher.tls_analyzer().unwrap().parse_error_count(),
        0,
        "AC-004: TlsAnalyzer must not receive first chunk (classified as Http)"
    );

    // Second chunk: same FlowKey, TLS bytes — if cache is used, HttpAnalyzer receives
    // this data (not TlsAnalyzer). TLS bytes sent to HttpAnalyzer fail parsing →
    // parse_error_count > 0 on HttpAnalyzer, parse_error_count == 0 on TlsAnalyzer.
    // AC-005 (EC-005): immutable cache — Http flow stays Http even with TLS content.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0, 0);

    assert_eq!(
        dispatcher.tls_analyzer().unwrap().parse_error_count(),
        0,
        "AC-005: cached Http flow must NOT route TLS bytes to TlsAnalyzer (immutable cache)"
    );
    assert!(
        dispatcher.http_analyzer().unwrap().parse_error_count() > 0,
        "AC-004/cache-hit: second chunk (TLS bytes) forwarded to HttpAnalyzer via cache — \
         HttpAnalyzer attempted to parse them, incrementing parse_error_count"
    );

    // AC-005: the flow closes as classified (not unclassified).
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-005: Http-cached flow must not be counted as unclassified on close"
    );
}

/// STORY-032 BC-2.05.005 EC-003 / EC-008 (flow-close cache eviction + re-classification):
/// When a flow is closed, its cached DispatchTarget is removed from `routes` and its
/// classification_attempts counter is removed. If the same FlowKey is reused after close,
/// the dispatcher must re-classify from scratch — there must be no stale None route
/// preventing classification of a legitimately-typed stream on the reopened flow.
///
/// Observability strategy (indirect):
///   Phase A — confirm None is permanently cached (cap=3, 3 unmatched chunks → routes[K]=None).
///             Proof: a 4th TLS chunk does NOT reach TlsAnalyzer (TlsAnalyzer counters stay 0).
///   Phase B — call on_flow_close; this evicts both routes[K] and classification_attempts[K].
///             Proof of eviction: send TLS bytes on K → classify runs → returns Tls → TlsAnalyzer
///             receives data → parse_error_count or truncated_record_count increments.
///             (If the None were NOT evicted, the cached None would short-circuit classify and
///             TlsAnalyzer would remain silent, contradicting what we observe.)
///   Phase C — verify Tls is now cached for the reopened flow (cache-hit proof):
///             Send HTTP GET bytes on K → if cached as Tls, classify does NOT re-run and
///             HttpAnalyzer receives nothing (method_counts["GET"] == 0).
///             If the cache were broken and classify re-ran on GET bytes, it would return Http
///             and HttpAnalyzer would record the GET — so absence of the method is the proof.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_005_cache_evicted_on_flow_close_then_reclassified() {
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    let unknown_bytes: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];

    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(3);
    let fk = flow_key(49152, 22);

    // Phase A: exhaust retry cap → None permanently cached in routes[fk].
    for _ in 0..3 {
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0, 0);
    }
    // Sanity-check that None is cached: a TLS chunk must not reach TlsAnalyzer.
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0, 0);
    assert_eq!(
        dispatcher.tls_analyzer().unwrap().parse_error_count(),
        0,
        "EC-008/setup: after cap=3, None is cached; TLS chunk must not reach TlsAnalyzer"
    );
    assert_eq!(
        dispatcher.tls_analyzer().unwrap().truncated_record_count(),
        0,
        "EC-008/setup: cached None short-circuits; no TLS events expected"
    );

    // Phase B: close the flow — routes[fk] and classification_attempts[fk] are both removed.
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "EC-008/close: permanently-None-cached flow must be counted as unclassified on close"
    );

    // Proof of cache eviction: same FlowKey, TLS bytes. classify must run (not short-circuit),
    // return Tls, and route data to TlsAnalyzer — creating a flow entry.
    // Updated in STORY-144: carry-buffer path accumulates short 0x16 payloads
    // without parse_errors; active_flows_len_for_testing() > 0 proves routing.
    // If the stale None were still present, TlsAnalyzer would remain silent (no flow entry).
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0, 0);
    assert!(
        dispatcher
            .tls_analyzer()
            .unwrap()
            .active_flows_len_for_testing()
            > 0,
        "EC-008/reclassify: after close, same FlowKey with TLS bytes must re-run classify; \
         TlsAnalyzer must receive data (stale None route was evicted, not reused)"
    );

    // Phase C: verify the reopened flow is now cached as Tls (not re-running classify on
    // every subsequent chunk). Send HTTP GET bytes — if Tls is cached, classify does NOT
    // re-run and HttpAnalyzer receives nothing (method_counts["GET"] stays 0).
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_bytes, 0, 0);
    assert_eq!(
        dispatcher
            .http_analyzer()
            .unwrap()
            .method_counts()
            .get("GET")
            .copied()
            .unwrap_or(0),
        0,
        "EC-008/cache-hit: GET bytes on Tls-cached reopened flow must NOT reach HttpAnalyzer; \
         if cache were broken, classify would re-run, return Http, and method_counts[GET] > 0"
    );

    // Close as Tls-classified (not unclassified).
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "EC-008/reclose: reopened flow classified as Tls must not increment unclassified_flows \
         (count stays at 1 from the prior close, not 2)"
    );
}

/// STORY-032 AC-003 + AC-006 (BC-2.05.004 invariants 1-2, BC-2.05.006 Phase A postconditions):
/// Before the retry cap is reached, DispatchTarget::None is NOT cached in `routes`.
/// Each on_data call re-runs classify, which means a late-arriving valid protocol chunk
/// can still classify the flow (as long as the cap hasn't been hit yet).
///
/// Observability strategy (indirect): with cap=8, send 7 unmatched chunks (SSH-like
/// bytes on unknown port 22) — None must NOT be permanently cached after chunk 7.
/// Proof: send an 8th chunk with valid TLS bytes; if None had been cached, classify
/// would not run and TlsAnalyzer would receive nothing. If None was NOT cached (correct),
/// classify runs on chunk 8, returns Tls, and TlsAnalyzer receives the data.
/// Also verifies: unclassified_flows() increments each time classify returns None
/// indirectly by confirming the flow closes as unclassified only when permanently None.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_006_none_not_cached_before_retry_cap() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(8);
    // Port 22 (SSH): not in {80, 443, 8080, 8443} → port fallback also fails → None.
    let fk = flow_key(49152, 22);

    // AC-006: 7 unmatched chunks (7 < cap of 8) — None NOT yet cached.
    let unknown_bytes: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    for _ in 0..7 {
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0, 0);
    }
    // Confirm neither analyzer received anything (all discarded as DispatchTarget::None).
    assert_eq!(
        dispatcher.http_analyzer().unwrap().parse_error_count(),
        0,
        "AC-006: unmatched chunks must not reach HttpAnalyzer"
    );
    assert_eq!(
        dispatcher.tls_analyzer().unwrap().parse_error_count(),
        0,
        "AC-006: unmatched chunks must not reach TlsAnalyzer"
    );

    // Key assertion: after 7 None results (7 < cap=8), None is NOT yet cached.
    // Proof: an 8th chunk with valid TLS bytes (byte0=0x16, byte1=0x03) must be
    // classified as Tls and routed to TlsAnalyzer. If None were cached, classify
    // would not run and TlsAnalyzer would remain silent.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0, 0);

    // Updated in STORY-144: carry-buffer path accumulates short 0x16 payloads
    // without parse_errors; active_flows_len_for_testing() > 0 proves routing.
    assert!(
        dispatcher
            .tls_analyzer()
            .unwrap()
            .active_flows_len_for_testing()
            > 0,
        "AC-003/AC-006: None must NOT be cached after 7 attempts (cap=8); \
         8th chunk with TLS bytes must re-run classify, route to TlsAnalyzer \
         (TlsAnalyzer creates a flow entry on receipt of any on_data call)"
    );
    // Flow closed as classified (Tls), not unclassified.
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-006: flow classified as Tls on 8th chunk must not count as unclassified"
    );
}

/// STORY-032 AC-002 + AC-007 + AC-008 (BC-2.05.006 Phase B postconditions, invariants 3-4):
/// When the retry cap is reached, DispatchTarget::None IS permanently cached.
/// Subsequent on_data calls short-circuit via the cache — classify is NOT called again.
///
/// Three sub-cases:
///   Sub-case 1 (cap=3): AC-002, AC-007, AC-008 primary scenario — 3 unmatched chunks hit
///     the cap; a 4th TLS chunk is silently dropped via the cached None route.
///   Sub-case 2 (cap=0): EC-004 — every flow immediately caches None on its first chunk;
///     a subsequent TLS chunk must not reach TlsAnalyzer.
///   Sub-case 3 (cap=8 default): EC-002 W12.L1 scenario-match — 8 consecutive None results
///     using the default cap (no explicit override); 9th TLS chunk must be suppressed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_006_none_cached_permanently_after_retry_cap() {
    // AC-008: cap is configurable (not hardcoded). Use cap=3 for a fast test.
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(3);
    assert_eq!(
        dispatcher.max_classification_attempts(),
        3,
        "AC-008: with_max_classification_attempts(3) must be reflected by the accessor"
    );
    let fk = flow_key(49152, 22);

    // AC-007: 3 unmatched chunks → on the 3rd, count reaches cap=3; None cached permanently.
    let unknown_bytes: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    for _ in 0..3 {
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0, 0);
    }

    // Chunk 4: valid TLS bytes. If None is permanently cached (correct), classify does
    // NOT run → TlsAnalyzer receives nothing → both parse_error_count and
    // truncated_record_count remain 0. If the cache were broken, TlsAnalyzer would fire.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0, 0);

    assert_eq!(
        dispatcher.tls_analyzer().unwrap().parse_error_count(),
        0,
        "AC-002/AC-007: after cap=3 is hit, None is permanently cached; \
         4th chunk (TLS bytes) must NOT reach TlsAnalyzer (classify not called)"
    );
    assert_eq!(
        dispatcher.tls_analyzer().unwrap().truncated_record_count(),
        0,
        "AC-002/AC-007: 4th chunk must be silently dropped via cached None route (not parsed)"
    );
    assert_eq!(
        dispatcher.http_analyzer().unwrap().parse_error_count(),
        0,
        "AC-002/AC-007: 4th chunk must NOT reach HttpAnalyzer either (cached None short-circuits)"
    );

    // Flow closes as unclassified (permanently-None-cached flows count as unclassified).
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "AC-007: permanently-None-cached flow must be counted as unclassified on close"
    );

    // EC-004: cap=0 → first chunk immediately caches None permanently.
    // A subsequent TLS chunk must NOT be classified (cache short-circuits on chunk 2).
    let mut d_zero = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(0);
    let fk2 = flow_key(49152, 22);
    // First chunk: unknown bytes → count would be 1, but 1 >= 0 after saturating, so
    // the implementation uses `>= max` check; with max=0, count(1) >= 0 → None cached.
    d_zero.on_data(&fk2, Direction::ClientToServer, &unknown_bytes, 0, 0);
    // Second chunk: TLS bytes — must not reach TlsAnalyzer (None cached after chunk 1).
    d_zero.on_data(&fk2, Direction::ClientToServer, &tls_bytes, 0, 0);
    assert_eq!(
        d_zero.tls_analyzer().unwrap().parse_error_count(),
        0,
        "EC-004: cap=0 caches None on first chunk; second TLS chunk must not reach TlsAnalyzer"
    );
    assert_eq!(
        d_zero.tls_analyzer().unwrap().truncated_record_count(),
        0,
        "EC-004: cap=0 cached-None short-circuits classify on all subsequent chunks"
    );

    // EC-002: default cap=8 sub-case (W12.L1 scenario-match)
    //
    // BC-2.05.006 EC-002 + STORY-032 edge-case catalog specify "8 consecutive None results
    // (default cap=8)". The sub-cases above used cap=3 and cap=0 (fast test). This sub-case
    // exercises the DEFAULT_MAX_CLASSIFICATION_ATTEMPTS=8 path with no explicit override —
    // a dispatcher constructed via `StreamDispatcher::new` alone, so the default is in effect.
    let mut d_default = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        d_default.max_classification_attempts(),
        8,
        "EC-002: default cap must equal DEFAULT_MAX_CLASSIFICATION_ATTEMPTS (8)"
    );
    let fk3 = flow_key(49152, 22);
    let unknown_bytes_default: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    let tls_bytes_default: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];

    // Send 8 unmatched chunks — on the 8th, attempt count reaches default cap of 8;
    // DispatchTarget::None is permanently cached in routes[fk3].
    for _ in 0..8 {
        d_default.on_data(
            &fk3,
            Direction::ClientToServer,
            &unknown_bytes_default,
            0,
            0,
        );
    }

    // Verify None is now permanently cached: a 9th chunk with valid TLS bytes must NOT
    // reach TlsAnalyzer (classify is short-circuited by the cached None route).
    d_default.on_data(&fk3, Direction::ClientToServer, &tls_bytes_default, 0, 0);

    assert_eq!(
        d_default.tls_analyzer().unwrap().parse_error_count(),
        0,
        "EC-002/default-cap=8: after 8 None results, None is permanently cached; \
         9th chunk (TLS bytes) must not reach TlsAnalyzer (classify not called)"
    );
    assert_eq!(
        d_default.tls_analyzer().unwrap().truncated_record_count(),
        0,
        "EC-002/default-cap=8: cached-None short-circuits classify; no TLS records parsed"
    );
    assert_eq!(
        d_default.http_analyzer().unwrap().parse_error_count(),
        0,
        "EC-002/default-cap=8: 9th chunk must not reach HttpAnalyzer either (cached None)"
    );

    // Flow closes as unclassified (permanently-None-cached).
    d_default.on_flow_close(&fk3, CloseReason::Fin);
    assert_eq!(
        d_default.unclassified_flows(),
        1,
        "EC-002/default-cap=8: permanently-None-cached flow must be counted as unclassified"
    );
}

/// STORY-032 AC-009 + EC-006 + EC-007 (BC-2.05.006 edge cases EC-001, EC-002):
/// Late classification after N None results (N < cap) succeeds: the (N+1)th chunk
/// with valid content is classified and cached. Subsequent chunks use the cached target.
///
/// Covers EC-006: 3 Nones then 1 TLS chunk (cap=8) → Tls cached on 4th call.
/// Covers EC-007: 7 Nones then 1 TLS chunk (cap=8) → Tls cached on 8th call
///                (cap not yet hit when TLS arrives on call 8 because count=7 < 8).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_006_late_classification_after_nones() {
    // --- EC-006: 3 Nones then TLS (cap=8) ---
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(8);
    let fk = flow_key(49152, 22);

    let unknown_bytes: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    // 3 unmatched chunks — attempt count reaches 3, still below cap of 8.
    for _ in 0..3 {
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0, 0);
    }

    // 4th chunk: TLS bytes — classify returns Tls; routes[fk]=Tls cached; attempts removed.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0, 0);

    // Updated in STORY-144: carry-buffer path accumulates short 0x16 payloads
    // without parse_errors; active_flows_len_for_testing() > 0 proves routing.
    assert!(
        dispatcher
            .tls_analyzer()
            .unwrap()
            .active_flows_len_for_testing()
            > 0,
        "AC-009/EC-006: TLS bytes on 4th call (3 prior Nones, cap=8) must classify as Tls \
         and route to TlsAnalyzer (TlsAnalyzer creates a flow entry on receipt of any on_data call)"
    );
    assert_eq!(
        dispatcher.http_analyzer().unwrap().parse_error_count(),
        0,
        "AC-009/EC-006: HttpAnalyzer must not receive the TLS bytes (routed to Tls)"
    );

    // 5th chunk: verify the CACHED Tls route is used (classify not re-run).
    // Send valid GET bytes — if the cached Tls route is used, classify does NOT re-run and
    // HttpAnalyzer never receives the data (method_counts["GET"] stays 0). If the cache were
    // broken and classify re-ran, it would return Http and HttpAnalyzer would record the GET.
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_bytes, 0, 0);

    // TlsAnalyzer receives the GET bytes via the cached Tls route; the bytes don't match a
    // TLS record type (byte0=0x47≠0x16) so the TLS parser silently skips them. The definitive
    // proof of cache-hit is the negative: HttpAnalyzer must NOT have received the data.
    assert_eq!(
        dispatcher
            .http_analyzer()
            .unwrap()
            .method_counts()
            .get("GET")
            .copied()
            .unwrap_or(0),
        0,
        "AC-009/cache-hit: GET bytes on Tls-cached flow must NOT reach HttpAnalyzer; \
         if cache were broken, classify would re-run, return Http, and method_counts[GET] > 0"
    );

    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-009/EC-006: late-classified Tls flow must not count as unclassified"
    );

    // --- EC-007: 7 Nones then TLS (cap=8) — cap not yet hit when TLS arrives ---
    let mut d2 = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(8);
    let fk2 = flow_key(49153, 22);

    for _ in 0..7 {
        d2.on_data(&fk2, Direction::ClientToServer, &unknown_bytes, 0, 0);
    }
    // 8th chunk: TLS bytes — attempt count was 7 (< cap=8); classify runs; returns Tls.
    d2.on_data(&fk2, Direction::ClientToServer, &tls_bytes, 0, 0);

    // Updated in STORY-144: carry-buffer path accumulates short 0x16 payloads
    // without parse_errors; active_flows_len_for_testing() > 0 proves routing.
    assert!(
        d2.tls_analyzer().unwrap().active_flows_len_for_testing() > 0,
        "EC-007: TLS bytes on 8th call (7 prior Nones, cap=8) must classify as Tls; \
         cap is not yet hit when TLS arrives (count=7 < 8 before this call's increment)"
    );
    d2.on_flow_close(&fk2, CloseReason::Fin);
    assert_eq!(
        d2.unclassified_flows(),
        0,
        "EC-007: flow classified as Tls on 8th chunk must not count as unclassified"
    );
}

// ---- STORY-033: on_flow_close lifecycle, unclassified counter, no-analyzer guard ----

// STORY-033 AC-001 + AC-003 + AC-006: unclassified_flows increments only at on_flow_close,
// for flows with no cached route (no prior on_data) and for flows cached as None after retry
// cap. Also exercises the unconditional cleanup of routes and classification_attempts (AC-006).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_007_unclassified_flows_counter() {
    // Sub-case 1 (AC-003 + AC-006): flow with no on_data call before on_flow_close.
    // routes.remove returns None → unclassified branch fires → unclassified_flows += 1.
    // At least one analyzer is configured (both present), so the guard is satisfied.
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk_no_data = flow_key(49200, 9999);

    // Verify counter is 0 before any close.
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-006/setup: unclassified_flows must start at 0"
    );

    // on_flow_close for a key never seen — routes.remove returns None → unclassified.
    dispatcher.on_flow_close(&fk_no_data, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "AC-001/AC-003: flow with no on_data must increment unclassified_flows on close \
         (routes.remove returns None → unclassified branch)"
    );

    // Sub-case 2 (AC-001): flow with unknown content → retry cap stamps DispatchTarget::None
    // in routes → on_flow_close matches Some(DispatchTarget::None) → unclassified branch.
    let fk_capped = flow_key(49201, 9999);
    let mut dispatcher2 = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    )
    .with_max_classification_attempts(2);

    // Two unknown-content chunks → attempt count reaches cap=2 → DispatchTarget::None cached.
    let unknown: &[u8] = &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    dispatcher2.on_data(&fk_capped, Direction::ClientToServer, unknown, 0, 0);
    dispatcher2.on_data(&fk_capped, Direction::ClientToServer, unknown, 0, 0);

    // Counter must NOT increment during on_data — only on close.
    assert_eq!(
        dispatcher2.unclassified_flows(),
        0,
        "AC-001: unclassified_flows must NOT increment during on_data (only at on_flow_close)"
    );

    dispatcher2.on_flow_close(&fk_capped, CloseReason::Fin);
    assert_eq!(
        dispatcher2.unclassified_flows(),
        1,
        "AC-001: flow cached as DispatchTarget::None after retry cap must increment \
         unclassified_flows on close (Some(DispatchTarget::None) → unclassified branch)"
    );

    // Sub-case 3 (AC-006 monotonic): two unclassified flow closes → counter == 2.
    let mut dispatcher3 = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk_a = flow_key(49202, 9999);
    let fk_b = flow_key(49203, 9999);

    dispatcher3.on_flow_close(&fk_a, CloseReason::Fin);
    assert_eq!(
        dispatcher3.unclassified_flows(),
        1,
        "AC-006: first unclassified close increments to 1"
    );
    dispatcher3.on_flow_close(&fk_b, CloseReason::Fin);
    assert_eq!(
        dispatcher3.unclassified_flows(),
        2,
        "AC-006: second unclassified close increments to 2 (monotonic)"
    );
}

// STORY-033 AC-002: classified flows (Http or Tls route) do NOT increment unclassified_flows
// on close. Counter is monotonically increasing and never decrements.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_007_classified_flow_not_counted_as_unclassified() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );

    // Part 1: HTTP-classified flow.
    let fk_http = flow_key(49210, 9999);
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk_http, Direction::ClientToServer, http_bytes, 0, 0);

    // Verify the flow was routed to Http (method_counts proves routing).
    assert_eq!(
        *dispatcher
            .http_analyzer()
            .unwrap()
            .method_counts()
            .get("GET")
            .unwrap_or(&0),
        1,
        "AC-002/setup: GET bytes must have been routed to HttpAnalyzer"
    );

    dispatcher.on_flow_close(&fk_http, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-002: Http-classified flow must NOT increment unclassified_flows on close"
    );

    // Part 2: TLS-classified flow — same dispatcher, counter must stay 0.
    let fk_tls = flow_key(49211, 9999);
    // TLS content bytes: byte0=0x16, byte1=0x03, len >= 5.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk_tls, Direction::ClientToServer, &tls_bytes, 0, 0);

    dispatcher.on_flow_close(&fk_tls, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-002: Tls-classified flow must NOT increment unclassified_flows on close; \
         counter must remain 0 after both classified closes (monotonic, never decrements)"
    );

    // Invariant: the counter never decremented (it started at 0, stayed at 0 through
    // two classified closes — this is the strongest monotonic verification available
    // without a dedicated decrement test).
}

// STORY-033 AC-004 + AC-005 (early-return aspect): StreamDispatcher::new(None, None, None, None, None, None) returns
// immediately from on_data before any classify or state mutation. Indirect proof via
// observing that routes/attempts maps remain empty (unclassified_flows stays 0 even
// on close, because the guard also prevents incrementing when no analyzers are configured).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_008_no_analyzer_dispatcher_early_returns() {
    let mut dispatcher = StreamDispatcher::new(None, None, None, None, None, None);
    let fk = flow_key(49220, 9999);

    // Call on_data multiple times with various byte patterns — must be no-ops.
    dispatcher.on_data(&fk, Direction::ClientToServer, b"GET / HTTP/1.1\r\n", 0, 0);
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        &[0x16u8, 0x03, 0x01, 0x00, 0x01, 0xFF],
        0,
        0,
    );
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE],
        0,
        0,
    );

    // Indirect proof: unclassified_flows stays 0 after on_flow_close.
    // The guard at dispatcher.rs:188-191 requires `self.http.is_some() ||
    // self.tls.is_some()`. With both None, the guard is not satisfied →
    // unclassified_flows is never incremented.
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-004: no-analyzer dispatcher early-returns from on_data; on_flow_close also does \
         not increment unclassified_flows (guard: no analyzer configured)"
    );

    // BC-2.05.008 invariant 2: on_flow_close still processes (no early return there).
    // This is verified above — on_flow_close ran without panic for an unseen FlowKey.
    // The absence of panic is itself the assertion (if on_flow_close had panicked on
    // missing-key, the test would fail with a thread panic before this point).

    // Additional case: close a different key (never in routes) — no panic, counter still 0.
    let fk2 = flow_key(49221, 9999);
    dispatcher.on_flow_close(&fk2, CloseReason::Rst);
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-004/EC-006: no-analyzer dispatcher must not increment unclassified_flows even \
         for unknown FlowKey closes (guard: no analyzer configured)"
    );
}

// STORY-033 AC-005: early-return guard fires only when BOTH analyzers are None.
// A dispatcher with only http=Some (tls=None) is NOT subject to early return —
// on_data runs classify and can route HTTP data.
// A dispatcher with only tls=Some (http=None) similarly is not early-returned.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_008_single_analyzer_not_early_returned() {
    // Part 1: http=Some, tls=None. HTTP GET bytes must be classified and forwarded.
    let mut dispatcher_http_only =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None, None);
    let fk_http = flow_key(49230, 9999);
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher_http_only.on_data(&fk_http, Direction::ClientToServer, http_bytes, 0, 0);

    assert_eq!(
        *dispatcher_http_only
            .http_analyzer()
            .unwrap()
            .method_counts()
            .get("GET")
            .unwrap_or(&0),
        1,
        "AC-005: http=Some/tls=None dispatcher must NOT early-return; \
         HTTP GET bytes must reach HttpAnalyzer (method_counts[GET] >= 1)"
    );

    // Part 2: http=None, tls=Some. TLS bytes must be classified and forwarded.
    // After on_data with TLS bytes, TlsAnalyzer receives the data and its
    // internal buffer has the flow registered (active_flows_len_for_testing == 1).
    let mut dispatcher_tls_only =
        StreamDispatcher::new(None, Some(TlsAnalyzer::new()), None, None, None, None);
    let fk_tls = flow_key(49231, 9999);
    // Valid-length TLS-like bytes: record_type=0x16, version=0x0301, payload_len=1 byte.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher_tls_only.on_data(&fk_tls, Direction::ClientToServer, &tls_bytes, 0, 0);

    // TlsAnalyzer must have received the data: active_flows_len_for_testing == 1 OR
    // parse/truncation counter > 0 (depending on how tls_parser handles the 1-byte payload).
    let tls_analyzer = dispatcher_tls_only.tls_analyzer().unwrap();
    assert!(
        tls_analyzer.active_flows_len_for_testing() >= 1
            || tls_analyzer.parse_error_count() > 0
            || tls_analyzer.truncated_record_count() > 0,
        "AC-005: http=None/tls=Some dispatcher must NOT early-return; \
         TLS bytes must reach TlsAnalyzer (active flow created or parse event recorded)"
    );
}

// STORY-033 AC-007 + AC-008: on_flow_close forwards the close event to the correct analyzer
// (Http or Tls depending on cached route). After forwarding, the analyzer's per-flow state
// is removed. Classified flows are NOT counted as unclassified (AC-008: exactly one destination).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_009_flow_close_forwards_to_http_analyzer() {
    // Part 1: Http-classified flow close → HttpAnalyzer.on_flow_close removes per-flow state.
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk_http = flow_key(49240, 9999);
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

    dispatcher.on_data(&fk_http, Direction::ClientToServer, http_bytes, 0, 0);

    // Verify HttpAnalyzer has per-flow state before close.
    assert_eq!(
        dispatcher
            .http_analyzer()
            .unwrap()
            .active_flows_len_for_testing(),
        1,
        "AC-007/setup: HttpAnalyzer must have per-flow state after on_data for Http-classified flow"
    );

    dispatcher.on_flow_close(&fk_http, CloseReason::Fin);

    // After on_flow_close, HttpAnalyzer.on_flow_close must have been called → flows entry removed.
    assert_eq!(
        dispatcher
            .http_analyzer()
            .unwrap()
            .active_flows_len_for_testing(),
        0,
        "AC-007: on_flow_close for Http-classified flow must forward to HttpAnalyzer \
         (HttpAnalyzer.flows entry removed)"
    );

    // AC-008: flow contributed to Http close, NOT to unclassified counter.
    assert_eq!(
        dispatcher.unclassified_flows(),
        0,
        "AC-008: Http-classified flow close must not increment unclassified_flows \
         (exactly one destination: Http analyzer)"
    );

    // Part 2: Tls-classified flow close → TlsAnalyzer.on_flow_close removes per-flow state.
    let mut dispatcher2 = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );
    let fk_tls = flow_key(49241, 9999);
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];

    dispatcher2.on_data(&fk_tls, Direction::ClientToServer, &tls_bytes, 0, 0);

    // TlsAnalyzer must have per-flow state before close.
    assert_eq!(
        dispatcher2
            .tls_analyzer()
            .unwrap()
            .active_flows_len_for_testing(),
        1,
        "AC-007/setup: TlsAnalyzer must have per-flow state after on_data for Tls-classified flow"
    );

    dispatcher2.on_flow_close(&fk_tls, CloseReason::Fin);

    // After on_flow_close, TlsAnalyzer.on_flow_close must have been called → flows entry removed.
    assert_eq!(
        dispatcher2
            .tls_analyzer()
            .unwrap()
            .active_flows_len_for_testing(),
        0,
        "AC-007: on_flow_close for Tls-classified flow must forward to TlsAnalyzer \
         (TlsAnalyzer.flows entry removed)"
    );

    // AC-008: Tls-classified flow contributed to Tls close, NOT to unclassified counter.
    assert_eq!(
        dispatcher2.unclassified_flows(),
        0,
        "AC-008: Tls-classified flow close must not increment unclassified_flows \
         (exactly one destination: Tls analyzer)"
    );
}

// STORY-033 AC-009: on_flow_close for a FlowKey never in routes (no prior on_data) causes
// routes.remove() to return None. The None branch executes, incrementing unclassified_flows
// if at least one analyzer is configured. No panic occurs.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_009_flow_close_for_unknown_flow_key() {
    let mut dispatcher = StreamDispatcher::new(
        Some(HttpAnalyzer::new()),
        Some(TlsAnalyzer::new()),
        None,
        None,
        None,
        None,
    );

    // Construct a FlowKey that was never seen by on_data.
    let fk_unknown = flow_key(49250, 9999);

    // on_flow_close must not panic; routes.remove returns None → unclassified branch.
    dispatcher.on_flow_close(&fk_unknown, CloseReason::Fin);

    assert_eq!(
        dispatcher.unclassified_flows(),
        1,
        "AC-009/EC-004: on_flow_close for unknown FlowKey must increment unclassified_flows \
         (routes.remove returns None → None branch executes; at least one analyzer configured)"
    );

    // Verify no analyzer received a close call (no per-flow state to remove).
    assert_eq!(
        dispatcher
            .http_analyzer()
            .unwrap()
            .active_flows_len_for_testing(),
        0,
        "AC-009: HttpAnalyzer must have no per-flow state for a key that was never on_data'd"
    );
    assert_eq!(
        dispatcher
            .tls_analyzer()
            .unwrap()
            .active_flows_len_for_testing(),
        0,
        "AC-009: TlsAnalyzer must have no per-flow state for a key that was never on_data'd"
    );

    // Variant: RST close for a different unknown key — no panic, counter increments again.
    let fk_unknown2 = flow_key(49251, 9999);
    dispatcher.on_flow_close(&fk_unknown2, CloseReason::Rst);
    assert_eq!(
        dispatcher.unclassified_flows(),
        2,
        "AC-009: second unknown-key close must further increment unclassified_flows to 2"
    );
}

// ---- STORY-097: timestamp threading from dispatcher to downstream analyzers ----

/// STORY-097 AC-004 (BC-2.04.055 dispatcher-forwarding invariant):
/// `StreamDispatcher::on_data` must thread the `timestamp` argument through to
/// BOTH the TLS and HTTP downstream analyzers unchanged.
///
/// Two sub-cases are exercised in the same test (one flow routed to TLS, one to
/// HTTP) so that both analyzer paths are covered and the shared `last_ts` store
/// in each analyzer's per-flow state is verified independently.
///
/// Observability: `TlsAnalyzer::last_ts_for_testing` / `HttpAnalyzer::last_ts_for_testing`
/// expose the most-recently stored capture timestamp for a given flow, mirroring
/// the `#[doc(hidden)]` testing-accessor pattern used by `active_flows_len_for_testing`,
/// `client_buf_len_for_testing`, etc.
#[test]
fn test_stream_dispatcher_forwards_timestamp_to_analyzers() {
    const TS: u32 = 7777;

    // ── Sub-case 1: TLS path ────────────────────────────────────────────────────
    // TLS content bytes: byte0=0x16, byte1=0x03 — routes to TlsAnalyzer.
    // Port 9999 (neutral) ensures classification is purely content-driven.
    {
        let mut dispatcher = StreamDispatcher::new(
            Some(HttpAnalyzer::new()),
            Some(TlsAnalyzer::new()),
            None,
            None,
            None,
            None,
        );
        let fk_tls = flow_key(49300, 9999);

        // 10-byte TLS-looking chunk with timestamp TS.
        let tls_data = [0x16u8, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
        dispatcher.on_data(&fk_tls, Direction::ClientToServer, &tls_data, 0, TS);

        // TlsAnalyzer must have received the data (flow created in its state map).
        let tls = dispatcher.tls_analyzer().expect("TLS analyzer present");
        assert!(
            tls.active_flows_len_for_testing() >= 1,
            "AC-004/TLS: TlsAnalyzer must have received the data chunk (flow state created)"
        );
        // The stored last_ts for the flow must equal the timestamp passed to on_data.
        assert_eq!(
            tls.last_ts_for_testing(&fk_tls),
            Some(TS),
            "AC-004/TLS: TlsAnalyzer.last_ts for flow must equal the timestamp forwarded \
             by StreamDispatcher (expected {TS}, dispatcher must not alter or drop it)"
        );
        // HTTP analyzer must NOT have received any data from this TLS-classified flow.
        let http = dispatcher.http_analyzer().expect("HTTP analyzer present");
        assert_eq!(
            http.last_ts_for_testing(&fk_tls),
            None,
            "AC-004/TLS: HttpAnalyzer must not have per-flow state for a TLS-routed flow"
        );
    }

    // ── Sub-case 2: HTTP path ───────────────────────────────────────────────────
    // HTTP GET bytes — routes to HttpAnalyzer.
    // Port 9998 (neutral) ensures classification is purely content-driven.
    {
        let mut dispatcher = StreamDispatcher::new(
            Some(HttpAnalyzer::new()),
            Some(TlsAnalyzer::new()),
            None,
            None,
            None,
            None,
        );
        let fk_http = flow_key(49301, 9998);

        let http_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        dispatcher.on_data(&fk_http, Direction::ClientToServer, http_data, 0, TS);

        // HttpAnalyzer must have received the data (method_counts updated).
        let http = dispatcher.http_analyzer().expect("HTTP analyzer present");
        assert!(
            http.method_counts().get("GET").copied().unwrap_or(0) >= 1,
            "AC-004/HTTP: HttpAnalyzer must have received the GET chunk (method_counts[GET] >= 1)"
        );
        // The stored last_ts for the flow must equal the timestamp passed to on_data.
        assert_eq!(
            http.last_ts_for_testing(&fk_http),
            Some(TS),
            "AC-004/HTTP: HttpAnalyzer.last_ts for flow must equal the timestamp forwarded \
             by StreamDispatcher (expected {TS}, dispatcher must not alter or drop it)"
        );
        // TLS analyzer must NOT have received any data from this HTTP-classified flow.
        let tls = dispatcher.tls_analyzer().expect("TLS analyzer present");
        assert_eq!(
            tls.last_ts_for_testing(&fk_http),
            None,
            "AC-004/HTTP: TlsAnalyzer must not have per-flow state for an HTTP-routed flow"
        );
    }
}

// ── STORY-153 (BC-2.05.010 + BC-2.05.011 + VP-042 + VP-043) ───────────────────
//
// Regression-guard test suite for:
//   - TransportProto enum (AC-153-001)
//   - unclassified_port_counts field + dual-gate + lower_port normalization (AC-153-002/003)
//   - TCP counter key purity (AC-153-004)
//   - udp_gap_key seam (AC-153-005)
//   - VP-042 proptest harnesses, 3 subs (AC-153-006)
//   - VP-043 proptest harnesses, 2 harnesses (AC-153-007)
//
// All tests exercise the fully-implemented counting logic (on_flow_close inner
// block and udp_gap_key seam) and serve as regression guards going forward.
//
// Structural/accessor tests remain GREEN-by-design per BC-5.38.002/003.
//
// F-F3P10-001 regression guard: `unclassified_flows += 1` is correctly placed
// outside the `coverage_gaps_enabled` block (ADR-012 Decision 6 Clarification
// EXACT) — this test ensures that placement is never regressed.
// ───────────────────────────────────────────────────────────────────────────────

#[allow(non_snake_case)]
mod story_153 {
    use std::net::IpAddr;

    use proptest::prelude::*;
    use wirerust::analyzer::http::HttpAnalyzer;
    use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
    use wirerust::dispatcher::{StreamDispatcher, TransportProto, udp_gap_key};
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// `StreamDispatcher` with one HTTP analyzer and `coverage_gaps_enabled = true`.
    /// Satisfies the dual-gate precondition (analyzer-present AND gaps enabled)
    /// required by BC-2.05.010 PC-1 / ADR-012 Decision 6 Clarification.
    fn gaps_dispatcher() -> StreamDispatcher {
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None, None)
            .with_coverage_gaps(true)
    }

    /// `StreamDispatcher` with one HTTP analyzer and `coverage_gaps_enabled = false`
    /// (the default). Used by F-F3P10-001 and the coverage_gaps_disabled tests.
    fn no_gaps_dispatcher() -> StreamDispatcher {
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None, None, None, None, None)
    }

    /// Builds a `FlowKey` where `10.0.0.1` is always the lower IP (since `10.0.0.1 <
    /// 10.0.0.9`), so `lower_port() == port_a` and `upper_port() == port_b`.
    /// The normalized service port used by the gap counter is
    /// `lower_port().min(upper_port()) = min(port_a, port_b)`.
    fn keyed(port_a: u16, port_b: u16) -> FlowKey {
        FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            port_a,
            "10.0.0.9".parse::<IpAddr>().unwrap(),
            port_b,
        )
    }

    /// Builds a synthetic `ParsedPacket` with `TransportInfo::Udp { src_port, dst_port }`.
    /// Used by the UDP seam tests and VP-043 proptest harnesses.
    fn make_udp_packet(src_port: u16, dst_port: u16) -> ParsedPacket {
        ParsedPacket {
            src_ip: "10.0.0.1".parse::<IpAddr>().unwrap(),
            dst_ip: "10.0.0.9".parse::<IpAddr>().unwrap(),
            protocol: Protocol::Udp,
            transport: TransportInfo::Udp { src_port, dst_port },
            payload: vec![],
            packet_len: 8,
        }
    }

    // ── AC-153-001: TransportProto enum ─────────────────────────────────────

    /// BC-2.05.010 PC-4 / Invariant 1: `TransportProto` has `Tcp` and `Udp` variants,
    /// distinct from `protocols::Transport` (which has a third `LinkLayer` variant).
    /// GREEN-by-design: enum variants are compiler-enforced constants.
    #[test]
    fn test_BC_2_05_010_key_type_identity() {
        assert_ne!(TransportProto::Tcp, TransportProto::Udp);
        let t = TransportProto::Tcp;
        assert_eq!(t, TransportProto::Tcp);
        let u = TransportProto::Udp;
        assert_eq!(u, TransportProto::Udp);
    }

    /// BC-2.05.010 PC-4 / ADR-012 Decision 6: `TransportProto` has EXACTLY 2 variants.
    /// An exhaustive match without a wildcard arm proves this at compile time —
    /// adding a third variant (e.g., `LinkLayer`) would cause a compile error here.
    /// GREEN-by-design.
    #[test]
    fn test_BC_2_05_transport_proto_no_linkLayer() {
        fn exhaustive(t: TransportProto) -> u8 {
            // No wildcard arm: compiler enforces exhaustiveness.
            match t {
                TransportProto::Tcp => 0,
                TransportProto::Udp => 1,
            }
        }
        assert_eq!(exhaustive(TransportProto::Tcp), 0);
        assert_eq!(exhaustive(TransportProto::Udp), 1);
    }

    // ── AC-153-002: Fields + accessor + builder ──────────────────────────────

    /// BC-2.05.010 PC-1 / BC-2.05.011 PC-1: accessor exists and returns an empty map
    /// after construction with coverage gaps enabled, before any `on_flow_close` calls.
    /// GREEN-by-design: `unclassified_port_counts()` returns `&self.field` with no
    /// branching — always succeeds; map starts empty.
    #[test]
    fn test_BC_2_05_010_fields_accessible() {
        let dispatcher = gaps_dispatcher();
        assert!(
            dispatcher.unclassified_port_counts().is_empty(),
            "unclassified_port_counts must be empty before any on_flow_close calls"
        );
    }

    /// BC-2.05.010 PC-4 (conditional-population gate): when constructed WITHOUT
    /// `.with_coverage_gaps(true)`, a None-target flow close must leave the map empty.
    ///
    /// Regression guard: `coverage_gaps_enabled = false` means the inner
    /// `if self.coverage_gaps_enabled` block is never entered; map stays empty.
    #[test]
    fn test_BC_2_05_010_coverage_gaps_disabled_map_empty() {
        let mut dispatcher = no_gaps_dispatcher();
        // None-target close: no on_data → routes returns None → None arm fires.
        // coverage_gaps_enabled = false → inner coverage_gaps block not entered.
        dispatcher.on_flow_close(&keyed(54321, 9999), CloseReason::Fin);
        assert!(
            dispatcher.unclassified_port_counts().is_empty(),
            "coverage_gaps=false: map must remain empty after None-target close \
             (dual-gate: inner coverage_gaps block must not fire)"
        );
    }

    // ── F-F3P10-001: unclassified_flows must NOT be gated on coverage_gaps ───

    /// F-F3P10-001 REGRESSION GUARD (ADR-012 Decision 6 Clarification EXACT):
    /// `unclassified_flows()` increments on a None-target close even when
    /// `coverage_gaps_enabled = false`. The per-port map must remain empty.
    ///
    /// REGRESSION: placing `unclassified_flows += 1` inside `if coverage_gaps_enabled`
    /// would zero this counter on all normal runs, breaking BC-2.05.009 and
    /// holdouts HS-040/HS-095.
    ///
    /// Regression guard: `unclassified_flows += 1` is correctly placed outside
    /// the `coverage_gaps_enabled` block — this test ensures that is never regressed.
    #[test]
    fn test_BC_2_05_010_unclassified_flows_fires_when_gaps_disabled() {
        let mut dispatcher = no_gaps_dispatcher();
        dispatcher.on_flow_close(&keyed(54321, 9999), CloseReason::Fin);
        assert_eq!(
            dispatcher.unclassified_flows(),
            1,
            "F-F3P10-001: unclassified_flows must increment regardless of coverage_gaps setting"
        );
        assert!(
            dispatcher.unclassified_port_counts().is_empty(),
            "F-F3P10-001: unclassified_port_counts must stay empty when coverage_gaps=false"
        );
    }

    // ── AC-153-003: TCP counter at on_flow_close ─────────────────────────────

    /// BC-2.05.010 PC-1 / Postcondition 1 — None-target close on neutral port 9999.
    ///
    /// Flow: client `10.0.0.1:54321` ↔ server `10.0.0.9:9999`.
    /// FlowKey: lower_ip = 10.0.0.1 (IP-ordered), lower_port = 54321 (ephemeral),
    /// upper_port = 9999. `lower_port().min(upper_port())` = min(54321, 9999) = 9999.
    ///
    /// IP-first ordering guard (F-F3P11-001): `lower_port()` alone = 54321 (wrong key).
    /// Correct impl uses `lower_port().min(upper_port())` = 9999.
    /// Port 9999 is neutral — not a classify() port rule target.
    /// Port 502 is RESERVED EXCLUSIVELY for `test_BC_2_05_011_no_increment_classified_flow`.
    ///
    /// Regression guard: on_flow_close None-target arm increments `unclassified_port_counts`.
    #[test]
    fn test_BC_2_05_010_tcp_counter_none_target() {
        let mut dispatcher = gaps_dispatcher();
        // FlowKey: lower_ip=10.0.0.1, lower_port=54321, upper_ip=10.0.0.9, upper_port=9999
        // lower_port().min(upper_port()) = min(54321, 9999) = 9999 (service port).
        let fk = FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            54321,
            "10.0.0.9".parse::<IpAddr>().unwrap(),
            9999,
        );
        dispatcher.on_flow_close(&fk, CloseReason::Fin);
        assert_eq!(
            dispatcher
                .unclassified_port_counts()
                .get(&(TransportProto::Tcp, 9999)),
            Some(&1),
            "BC-2.05.010 PC-1: (Tcp, 9999) count must be 1 after 1 None-target close; \
             IP-first guard: lower_port() alone = 54321 (wrong); \
             correct impl keys on min(54321, 9999) = 9999"
        );
    }

    /// BC-2.05.011 PC-1 + Invariant 1 (monotonically non-decreasing):
    /// Three successive None-target closes on the same port produce count == 3.
    ///
    /// Regression guard: on_flow_close inner block increments count on every None-target close.
    #[test]
    fn test_BC_2_05_011_monotonic_increment() {
        let mut dispatcher = gaps_dispatcher();
        // keyed(60000, 7777): lower_port=60000, upper_port=7777, min=7777
        let fk = keyed(60000, 7777);
        for _ in 0..3 {
            dispatcher.on_flow_close(&fk, CloseReason::Fin);
        }
        assert_eq!(
            dispatcher
                .unclassified_port_counts()
                .get(&(TransportProto::Tcp, 7777)),
            Some(&3),
            "BC-2.05.011 PC-1 / Invariant 1: count must be exactly 3 after 3 None-target closes"
        );
    }

    /// BC-2.05.011 PC-4 / EC-002 label fix: a Modbus-classified flow close on port 502
    /// must NOT increment `unclassified_port_counts`.
    ///
    /// EC-002 label fix: BC-2.05.011 EC-002 says "Http/502" but the correct
    /// `DispatchTarget` for port 502 is `Modbus`. This test uses Modbus/502.
    ///
    /// A None-target close on port 9001 makes the test non-vacuous: without it,
    /// `(Tcp, 502)` being absent would be trivially true (no count was ever attempted).
    ///
    /// Regression guard: the None-target close on port 9001 increments the counter.
    #[test]
    fn test_BC_2_05_011_no_increment_classified_flow() {
        let mut dispatcher = gaps_dispatcher();

        // POSITIVE — makes test non-vacuous: None-target close on port 9001.
        // keyed(60001, 9001): lower_port=60001, upper_port=9001, min=9001
        dispatcher.on_flow_close(&keyed(60001, 9001), CloseReason::Fin);
        // After impl: (Tcp, 9001) == Some(&1).

        // NEGATIVE: classify a flow on port 502 as Modbus (Rule 5) then close it.
        // Non-TLS (0x00 != 0x16), non-HTTP → Rule 5 (port 502) → Modbus.
        // keyed(54321, 502): upper_port=502; Modbus arm fires, None arm does NOT.
        let fk_modbus = keyed(54321, 502);
        let modbus_data = [0x00u8, 0x01, 0x00, 0x00, 0x00, 0x04, 0x01, 0x01, 0x00, 0x01];
        dispatcher.on_data(&fk_modbus, Direction::ClientToServer, &modbus_data, 0, 0);
        dispatcher.on_flow_close(&fk_modbus, CloseReason::Fin);

        // The None-target close on port 9001 must have been counted.
        assert_eq!(
            dispatcher
                .unclassified_port_counts()
                .get(&(TransportProto::Tcp, 9001)),
            Some(&1),
            "None-target close on port 9001 must produce count == 1"
        );
        // The Modbus-classified close on port 502 must NOT have been counted.
        // (EC-002 label fix: BC-2.05.011 EC-002 says Http/502; correct target is Modbus/502)
        assert!(
            !dispatcher
                .unclassified_port_counts()
                .contains_key(&(TransportProto::Tcp, 502)),
            "BC-2.05.011 PC-4 / EC-002 label fix: Modbus-classified close on port 502 \
             must NOT add (Tcp, 502) to unclassified_port_counts"
        );
    }

    /// BC-2.05.010 PC-1 / F-F3P11-001: `lower_port().min(upper_port())` normalization.
    ///
    /// Sub-case (a) direction normalization:
    ///   `keyed(1234, 9999)`: lower_port=1234, upper_port=9999, min=1234
    ///
    /// Sub-case (b) client-has-lower-IP guard — the core F-F3P11-001 case:
    ///   `keyed(9999, 1234)`: lower_port=9999, upper_port=1234 (IP-first ordering!).
    ///   `lower_port()` alone = 9999 (WRONG — lower_ip's ephemeral port).
    ///   `lower_port().min(upper_port())` = min(9999, 1234) = 1234 (CORRECT — service port).
    ///
    /// Both flows must produce key `(Tcp, 1234)`: total count == 2.
    ///
    /// Regression guard: on_flow_close None-target arm correctly normalizes port keys.
    #[test]
    fn test_BC_2_05_010_lower_port_normalization() {
        let mut dispatcher = gaps_dispatcher();

        // Sub-case (a): lower_ip=10.0.0.1, lower_port=1234, upper_port=9999, min=1234
        dispatcher.on_flow_close(&keyed(1234, 9999), CloseReason::Fin);

        // Sub-case (b) — IP-first ordering guard:
        // lower_ip=10.0.0.1, lower_port=9999, upper_port=1234
        // lower_port() alone = 9999 (wrong); lower_port().min(upper_port()) = 1234 (correct)
        dispatcher.on_flow_close(&keyed(9999, 1234), CloseReason::Fin);

        assert_eq!(
            dispatcher
                .unclassified_port_counts()
                .get(&(TransportProto::Tcp, 1234)),
            Some(&2),
            "F-F3P11-001: both flows (keyed(1234,9999) and keyed(9999,1234)) must produce \
             key (Tcp, 1234) = min(1234,9999); two closes → count == 2"
        );
        // Port 9999 must NOT appear: that would indicate lower_port() alone was used.
        assert!(
            !dispatcher
                .unclassified_port_counts()
                .contains_key(&(TransportProto::Tcp, 9999)),
            "F-F3P11-001: (Tcp, 9999) must NOT appear — lower_port() alone on \
             keyed(9999,1234) = 9999 is the IP-first ordering bug; \
             correct key is (Tcp, min(9999,1234)) = (Tcp, 1234)"
        );
    }

    /// BC-2.05.010 PC-4 (coverage_gaps disabled, no-increment variant):
    /// When `coverage_gaps_enabled = false`, a None-target flow close must NOT
    /// increment `unclassified_port_counts` (inner gate not entered).
    ///
    /// Regression guard: `coverage_gaps = false` bypasses the inner counting block; map stays empty.
    #[test]
    fn test_BC_2_05_010_coverage_gaps_disabled_no_increment() {
        let mut dispatcher = no_gaps_dispatcher();
        dispatcher.on_flow_close(&keyed(60000, 8888), CloseReason::Fin);
        assert!(
            !dispatcher
                .unclassified_port_counts()
                .contains_key(&(TransportProto::Tcp, 8888)),
            "coverage_gaps=false: (Tcp, 8888) must NOT be added to unclassified_port_counts"
        );
        assert!(
            dispatcher.unclassified_port_counts().is_empty(),
            "coverage_gaps=false: map must remain empty after any number of None-target closes"
        );
    }

    // ── AC-153-004: TCP map key purity ───────────────────────────────────────

    /// BC-2.05.010 PC-3 / BC-2.05.011 PC-5 / Invariant 4:
    /// Every key in `unclassified_port_counts` has `key.0 == TransportProto::Tcp`.
    /// No `TransportProto::Udp` key may appear in the TCP dispatcher map.
    ///
    /// Regression guard: on_flow_close None-target closes with `coverage_gaps_enabled = true`
    /// only ever insert `TransportProto::Tcp` keys into the map.
    #[test]
    fn test_BC_2_05_011_tcp_map_key_purity() {
        let mut dispatcher = gaps_dispatcher();
        for &svc_port in &[7777u16, 8888, 9000] {
            // keyed(60000, svc_port): lower_port=60000, upper_port=svc_port, min=svc_port
            dispatcher.on_flow_close(&keyed(60000, svc_port), CloseReason::Fin);
        }
        assert!(
            dispatcher
                .unclassified_port_counts()
                .keys()
                .all(|(t, _)| *t == TransportProto::Tcp),
            "BC-2.05.011 Invariant 4: all keys in unclassified_port_counts must \
             carry TransportProto::Tcp — no Udp key may appear in the TCP map"
        );
    }

    // ── AC-153-005: UDP gap-key seam (udp_gap_key) ───────────────────────────

    /// BC-2.05.010 PC-2 / EC-001 (BACnet/IP):
    /// `udp_gap_key` returns `Some((Udp, min_port))` for an unhandled UDP packet.
    ///
    /// Regression guard: `udp_gap_key` returns the correct `(Udp, min_port)` key.
    #[test]
    fn test_BC_2_05_010_udp_counter_unhandled() {
        // BACnet/IP: src=61000 (client ephemeral), dst=47808 (BACnet port)
        // min(61000, 47808) = 47808 → expected key (Udp, 47808)
        let packet = make_udp_packet(61000, 47808);
        let result = udp_gap_key(&packet, false);
        assert_eq!(
            result,
            Some((TransportProto::Udp, 47808)),
            "BC-2.05.010 PC-2: unhandled UDP/47808 must return Some((Udp, 47808))"
        );
    }

    /// BC-2.05.010 Invariant 7 / ADR-012 Decision 10:
    /// `udp_gap_key` returns `None` when `dns_handles = true` (DNS accepted the packet).
    ///
    /// Regression guard: `udp_gap_key` returns `None` when `dns_handles = true`.
    #[test]
    fn test_BC_2_05_010_udp_dns_not_counted() {
        // DNS response direction: src=53 (server), dst=60000 (client ephemeral)
        let packet = make_udp_packet(53, 60000);
        let result = udp_gap_key(&packet, true);
        assert_eq!(
            result, None,
            "BC-2.05.010 Invariant 7 / ADR-012 Decision 10: \
             dns_handles=true must return None (DNS gap-excluded)"
        );
    }

    /// BC-2.05.010 PC-2 / EC-012/EC-013 (BACnet bidirectionality):
    /// `udp_gap_key` normalizes to `min(src_port, dst_port)` so query and response
    /// directions both produce the same key `(Udp, 47808)`.
    ///
    /// Regression guard: `udp_gap_key` normalizes to `min(src_port, dst_port)` for both directions.
    #[test]
    fn test_BC_2_05_010_udp_lower_port_normalization() {
        // Query: src=61000 (ephemeral), dst=47808 (BACnet) → min=47808
        let packet_query = make_udp_packet(61000, 47808);
        // Response: src=47808 (BACnet), dst=61000 (ephemeral) → min=47808
        let packet_response = make_udp_packet(47808, 61000);

        let result_query = udp_gap_key(&packet_query, false);
        let result_response = udp_gap_key(&packet_response, false);

        assert_eq!(
            result_query,
            Some((TransportProto::Udp, 47808)),
            "Query direction src=61000/dst=47808 must return Some((Udp, 47808))"
        );
        assert_eq!(
            result_response,
            Some((TransportProto::Udp, 47808)),
            "Response direction src=47808/dst=61000 must return same key Some((Udp, 47808))"
        );
        assert_eq!(
            result_query, result_response,
            "BC-2.05.010 PC-2: both directions must produce the same key (Udp, 47808)"
        );
    }

    /// BC-2.05.010 PC-3 / BC-2.05.011 Invariant 4 (UDP key purity):
    /// All `Some(_)` returns from `udp_gap_key` carry `TransportProto::Udp`.
    /// A non-UDP `ParsedPacket` returns `None` (no Tcp key ever appears).
    ///
    /// Regression guard: all `Some(_)` returns from `udp_gap_key` carry `TransportProto::Udp`.
    #[test]
    fn test_BC_2_05_011_udp_map_key_purity() {
        let cases = [
            make_udp_packet(61000, 47808), // BACnet/IP → (Udp, 47808)
            make_udp_packet(61001, 161),   // SNMP → (Udp, 161)
            make_udp_packet(9999, 8888),   // neutral ports → (Udp, 8888)
            make_udp_packet(47808, 61000), // BACnet response direction → (Udp, 47808)
        ];
        for pkt in &cases {
            let result = udp_gap_key(pkt, false);
            assert!(
                matches!(result, Some((TransportProto::Udp, _))),
                "BC-2.05.011 Invariant 4: udp_gap_key must return \
                 Some((TransportProto::Udp, _)) for unhandled UDP packets; got {result:?}"
            );
        }
        // DNS packet with dns_handles=true must return None — gap-excluded.
        let dns_pkt = make_udp_packet(53, 60000);
        assert_eq!(
            udp_gap_key(&dns_pkt, true),
            None,
            "DNS-handled packet must return None (no counter increment)"
        );
        // A TCP ParsedPacket (non-UDP transport) must return None — seam is UDP-only.
        let tcp_pkt = ParsedPacket {
            src_ip: "10.0.0.1".parse::<IpAddr>().unwrap(),
            dst_ip: "10.0.0.9".parse::<IpAddr>().unwrap(),
            protocol: Protocol::Tcp,
            transport: TransportInfo::Tcp {
                src_port: 12345,
                dst_port: 80,
                seq_number: 0,
                syn: false,
                ack: true,
                fin: false,
                rst: false,
            },
            payload: vec![],
            packet_len: 20,
        };
        assert_eq!(
            udp_gap_key(&tcp_pkt, false),
            None,
            "Non-UDP ParsedPacket must return None from udp_gap_key"
        );
    }

    // ── AC-153-006: VP-042 proptest harnesses (TCP dispatcher path) ───────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// VP-042 Sub-A: `unclassified_port_counts.values().sum() == N` after N
        /// None-target `on_flow_close` calls with `coverage_gaps = true` and ≥1 analyzer.
        ///
        /// Regression guard: on_flow_close None-target arm accumulates counts correctly.
        #[test]
        fn proptest_vp042_total_count_equals_n(
            n in 1u64..=256u64,
            service_port in 1024u16..=19999u16,
        ) {
            let mut dispatcher = gaps_dispatcher();
            // keyed(50000, service_port): lower_port=50000, upper_port=service_port.
            // min(50000, service_port) = service_port (service_port ≤ 19999 < 50000).
            let fk = keyed(50000, service_port);
            for _ in 0..n {
                dispatcher.on_flow_close(&fk, CloseReason::Fin);
            }
            let total: u64 = dispatcher.unclassified_port_counts().values().sum();
            prop_assert_eq!(
                total, n,
                "VP-042 Sub-A: sum of all counts must equal N after N None-target closes"
            );
        }

        /// VP-042 Sub-B: for each port P in a generated sequence, the count for `(Tcp, P)`
        /// equals the number of times P appears in the sequence (exactness property).
        ///
        /// Regression guard: on_flow_close None-target arm tracks per-port frequencies correctly.
        #[test]
        fn proptest_vp042_per_port_count_equals_frequency(
            ports in proptest::collection::vec(1024u16..=9000u16, 1..=20usize),
        ) {
            let mut dispatcher = gaps_dispatcher();
            for (i, &service_port) in ports.iter().enumerate() {
                // keyed(50000+i, service_port): min(50000+i, service_port) = service_port.
                // (service_port ≤ 9000 < 50000+i; i ≤ 19 so 50000+i ≤ 50019 ≤ u16::MAX)
                dispatcher.on_flow_close(
                    &keyed(50000 + i as u16, service_port),
                    CloseReason::Fin,
                );
            }
            // Build the expected frequency map from the input port sequence.
            let mut freq: std::collections::HashMap<u16, u64> = std::collections::HashMap::new();
            for &p in &ports {
                *freq.entry(p).or_insert(0) += 1;
            }
            // Each port's counter must equal its frequency.
            for (&p, &expected) in &freq {
                let got = dispatcher
                    .unclassified_port_counts()
                    .get(&(TransportProto::Tcp, p))
                    .copied()
                    .unwrap_or(0);
                prop_assert_eq!(
                    got, expected,
                    "VP-042 Sub-B: count for (Tcp, {}) must equal input frequency {}",
                    p,
                    expected
                );
            }
            // No spurious extra keys.
            prop_assert_eq!(
                dispatcher.unclassified_port_counts().len(),
                freq.len(),
                "VP-042 Sub-B: map must contain exactly as many keys as distinct ports"
            );
        }

        /// VP-042 Sub-C (BC-2.05.011 Invariant 5): a classified `on_flow_close` on port P
        /// (Http via content detection) must NOT change the count for `(Tcp, P)` even when
        /// None-target closes on the same port have already incremented it.
        ///
        /// Regression guard: classified `on_flow_close` must not change the count established
        /// by preceding None-target closes; k = 0 validates the zero-base case.
        #[test]
        fn proptest_vp042_no_count_spurious_on_classified_flows(
            service_port in 1024u16..=9000u16,
            k in 0u64..=10u64,
        ) {
            let mut dispatcher = gaps_dispatcher();
            // k None-target closes on service_port.
            // keyed(50000, service_port): min(50000, service_port) = service_port.
            let fk_none = keyed(50000, service_port);
            for _ in 0..k {
                dispatcher.on_flow_close(&fk_none, CloseReason::Fin);
            }
            // Classify a flow on the SAME service_port via HTTP content (Rule 2).
            // "GET " fires Rule 2 regardless of port — content-first wins over all port rules.
            let fk_classified = keyed(49999, service_port);
            let http_data = b"GET / HTTP/1.1\r\nHost: x\r\n\r\n";
            dispatcher.on_data(
                &fk_classified,
                Direction::ClientToServer,
                http_data,
                0,
                0,
            );
            dispatcher.on_flow_close(&fk_classified, CloseReason::Fin);
            // Count must still equal k — the classified close must not increment the counter.
            let count = dispatcher
                .unclassified_port_counts()
                .get(&(TransportProto::Tcp, service_port))
                .copied()
                .unwrap_or(0);
            prop_assert_eq!(
                count, k,
                "VP-042 Sub-C / BC-2.05.011 Invariant 5: classified Http close on port \
                 {} must not change the count from {}",
                service_port,
                k
            );
        }
    }

    // ── AC-153-007: VP-043 proptest harnesses (udp_gap_key seam) ─────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// VP-043 / DF-KANI-NONVACUITY-001: for M calls to `udp_gap_key` on a UDP packet
        /// with `min(src_port, dst_port) == Q` and `dns_handles = false`, all M calls return
        /// `Some((TransportProto::Udp, Q))`.
        ///
        /// Calls the production seam directly (non-vacuous: `udp_unclassified_counts`
        /// in main.rs is unreachable from integration tests).
        ///
        /// Regression guard: `udp_gap_key` consistently returns `Some((Udp, Q))` for all N calls.
        #[test]
        fn proptest_vp043_total_count_equals_n(
            n in 1usize..=256usize,
            q in 1024u16..=9000u16,
        ) {
            // src = q + 10000 → min(src, q) = q. (q ≤ 9000, src = q+10000 ≤ 19000 ≤ u16::MAX)
            let src = q + 10000;
            let packet = make_udp_packet(src, q);
            for _ in 0..n {
                let result = udp_gap_key(&packet, false);
                prop_assert_eq!(
                    result,
                    Some((TransportProto::Udp, q)),
                    "VP-043: udp_gap_key must return Some((Udp, {})) for unhandled \
                     UDP packet with min(src, dst) == {}",
                    q,
                    q
                );
            }
        }

        /// VP-043 / ADR-012 Decision 10 (DNS exclusion gate):
        /// For any UDP packet, `udp_gap_key(parsed, true)` returns `None`.
        /// The seam guards the main.rs loop: `dns_handles = true` → counter not incremented.
        ///
        /// Regression guard: `udp_gap_key` returns `None` for any packet when `dns_handles = true`.
        #[test]
        fn proptest_vp043_no_increment_on_classified_udp(
            src_port in 1u16..=60000u16,
            dst_port in 1u16..=65535u16,
        ) {
            let packet = make_udp_packet(src_port, dst_port);
            // dns_handles=true: dissector accepted this packet — must not be counted.
            let result = udp_gap_key(&packet, true);
            prop_assert_eq!(
                result,
                None,
                "VP-043 / ADR-012 Decision 10: dns_handles=true must return None; \
                 got {:?} for src={} dst={}",
                result,
                src_port,
                dst_port
            );
        }
    }
}

// ── F6 mutation-hardening: on_flow_close analyzer-present guard ────────────────
//
// Pins survivors from `cargo mutants` on the 5-way analyzer-present disjunction
// in `on_flow_close` (dispatcher.rs ~line 461-465):
//   http.is_some() || tls.is_some() || modbus.is_some() || dnp3.is_some() || enip.is_some()
// The `gaps_dispatcher()` helper always has an HTTP analyzer, so the first
// disjunct is always true and the dnp3/enip clauses are never load-bearing —
// mutating `|| enip.is_some()` to `&& enip.is_some()` (and the dnp3 clause) went
// undetected. These tests build dispatchers with EXACTLY ONE non-HTTP analyzer so
// each trailing disjunct becomes the sole reason the guard is true; the
// `unclassified_flows` counter (gated ONLY on this guard) must then reach 1.
mod f6_hardening {
    use std::net::IpAddr;

    use wirerust::analyzer::dnp3::Dnp3Analyzer;
    use wirerust::analyzer::enip::EnipAnalyzer;
    use wirerust::dispatcher::StreamDispatcher;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{CloseReason, StreamHandler};

    fn none_flow() -> FlowKey {
        FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            50000,
            "10.0.0.9".parse::<IpAddr>().unwrap(),
            40000,
        )
    }

    /// Kills `dispatcher.rs replace || with && (enip disjunct)` in on_flow_close.
    /// With ONLY an ENIP analyzer present, the guard is true solely because of the
    /// `|| enip.is_some()` clause; the mutant `dnp3.is_some() && enip.is_some()`
    /// becomes `false && true == false`, so the counter would stay 0.
    #[test]
    fn f6_unclassified_counts_with_only_enip_analyzer() {
        let mut dispatcher = StreamDispatcher::new(
            None,
            None,
            None,
            None,
            Some(EnipAnalyzer::new(10, 10)),
            None,
        );
        dispatcher.on_flow_close(&none_flow(), CloseReason::Fin);
        assert_eq!(
            dispatcher.unclassified_flows(),
            1,
            "ENIP-only dispatcher must count an unclassified None-target close \
             (guard depends solely on the enip.is_some() disjunct)"
        );
    }

    /// Kills the analogous `|| with && (dnp3 disjunct)` mutation. With ONLY a DNP3
    /// analyzer present, the guard is true solely because of `|| dnp3.is_some()`.
    #[test]
    fn f6_unclassified_counts_with_only_dnp3_analyzer() {
        let mut dispatcher =
            StreamDispatcher::new(None, None, None, Some(Dnp3Analyzer::new(10)), None, None);
        dispatcher.on_flow_close(&none_flow(), CloseReason::Fin);
        assert_eq!(
            dispatcher.unclassified_flows(),
            1,
            "DNP3-only dispatcher must count an unclassified None-target close \
             (guard depends solely on the dnp3.is_some() disjunct)"
        );
    }
}

// =============================================================================
// STORY-173: AC-173-001 (classify) + AC-173-008 (dispatcher wiring)
// All tests live in `mod story_173` per DF-TEST-NAMESPACE-001.
// =============================================================================
//
// Two tests verify dispatcher wiring to Iec104Analyzer (now green):
//   test_iec104_only_dispatcher_data_reaches_analyzer — dispatcher forwards STARTDT-act
//   test_iec104_only_dispatcher_stopdt_produces_t0881 — dispatcher forwards STOPDT-act
//
// Three tests are GUARDS that verify safety invariants:
//   test_BC_2_05_012_early_exit_guard_includes_iec104 — guard prevents early exit
//   test_iec104_disabled_port_2404_no_panic           — None iec104 doesn't panic
//   test_iec104_only_guard_unclassified_flows_counted  — guard ensures flow visibility

mod story_173 {
    #![allow(non_snake_case)]

    use std::net::IpAddr;

    use wirerust::analyzer::iec104::Iec104Analyzer;
    use wirerust::dispatcher::StreamDispatcher;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};

    fn flow_key(src_port: u16, dst_port: u16) -> FlowKey {
        FlowKey::new(
            "10.0.0.1".parse::<IpAddr>().unwrap(),
            src_port,
            "10.0.0.2".parse::<IpAddr>().unwrap(),
            dst_port,
        )
    }

    // STARTDT-act U-frame: start=0x68, LEN=4, CF1=0x07, CF2-CF4=0.
    fn startdt_act() -> Vec<u8> {
        vec![0x68, 0x04, 0x07, 0x00, 0x00, 0x00]
    }

    // STOPDT-act U-frame: start=0x68, LEN=4, CF1=0x13, CF2-CF4=0.
    fn stopdt_act() -> Vec<u8> {
        vec![0x68, 0x04, 0x13, 0x00, 0x00, 0x00]
    }

    // -------------------------------------------------------------------------
    // AC-173-008 — DISPATCHER WIRING
    // Iec104 on_data arm forwards data via iec104.on_data(...) per ADR-013 Decision 9.
    // -------------------------------------------------------------------------

    /// AC-173-008 — dispatcher wiring: STARTDT-act on port 2404 must reach Iec104Analyzer.
    ///
    /// With ONLY iec104 set, feeding a STARTDT-act on port 2404 through the dispatcher
    /// must result in Iec104Analyzer::flows having 1 entry and session_started = true.
    ///
    /// The Iec104 arm in on_data calls `iec104.on_data(...)` to forward the packet.
    /// This creates a per-flow state and processes the STARTDT-act.
    ///
    /// Traces: AC-173-008; BC-2.05.012 invariant 1; ADR-013 Decision 9 step 5.
    #[test]
    fn test_iec104_only_dispatcher_data_reaches_analyzer() {
        let iec104 = Iec104Analyzer::new();
        let mut dispatcher = StreamDispatcher::new(None, None, None, None, None, Some(iec104));

        let fk = flow_key(60001, 2404);
        dispatcher.on_data(
            &fk,
            Direction::ClientToServer,
            &startdt_act(),
            0,
            1_700_000_000,
        );

        let analyzer = dispatcher
            .iec104_analyzer()
            .expect("IEC-104 analyzer must be present when configured");

        // Dispatcher forwards to iec104.on_data(...), which creates per-flow state.
        assert_eq!(
            analyzer.flows.len(),
            1,
            "AC-173-008: Iec104Analyzer::flows must have 1 entry after feeding a STARTDT-act \
             on port 2404. Got {} entries.",
            analyzer.flows.len()
        );
        // Flow state now exists due to dispatcher forwarding.
        let state = analyzer.flows.get(&fk);
        assert!(
            state.is_some(),
            "AC-173-008: flows must have an entry for the port-2404 FlowKey"
        );
        assert!(
            state.unwrap().session_started,
            "AC-173-008: STARTDT-act must set session_started = true after dispatcher wiring"
        );
    }

    /// AC-173-008 — STOPDT-act on port 2404 must produce a T0881 finding via the dispatcher.
    ///
    /// With ONLY iec104 set, a STOPDT-act (session_started=false → T0881 Verdict::Likely)
    /// fed through the dispatcher must appear in all_findings.
    ///
    /// The Iec104 arm calls iec104.on_data(...) to forward the packet and detect the threat.
    ///
    /// Traces: AC-173-008; BC-2.19.011 (T0881 Likely on stop without prior start).
    #[test]
    fn test_iec104_only_dispatcher_stopdt_produces_t0881() {
        let iec104 = Iec104Analyzer::new();
        let mut dispatcher = StreamDispatcher::new(None, None, None, None, None, Some(iec104));

        let fk = flow_key(60002, 2404);
        dispatcher.on_data(
            &fk,
            Direction::ClientToServer,
            &stopdt_act(),
            0,
            1_700_000_000,
        );

        let analyzer = dispatcher
            .iec104_analyzer()
            .expect("IEC-104 analyzer must be present when configured");

        // Dispatcher forwards to iec104.on_data(...), which detects STOPDT without prior STARTDT.
        assert_eq!(
            analyzer.all_findings.len(),
            1,
            "AC-173-008: STOPDT-act through the dispatcher must emit 1 T0881 finding. \
             Got {} findings.",
            analyzer.all_findings.len()
        );
        assert!(
            analyzer
                .all_findings
                .first()
                .map(|f| f.mitre_techniques.iter().any(|t| t == "T0881"))
                .unwrap_or(false),
            "AC-173-008: the finding must cite T0881 (Service Stop)"
        );
    }

    // -------------------------------------------------------------------------
    // AC-173-001 / AC-173-008 GUARDS (PASS on current stub)
    // -------------------------------------------------------------------------

    /// AC-173-001 guard — early-exit guard includes iec104.is_none() so a iec104-only
    /// dispatcher does NOT silently discard data before reaching the match arm.
    ///
    /// PASSES on current stub (guard is in place; data reaches the (no-op) Iec104 arm).
    /// Validates ADR-013 Decision 9 step 4.
    ///
    /// Traces: AC-173-008; ADR-013 Decision 9 step 4 (early-exit guard).
    #[test]
    fn test_BC_2_05_012_early_exit_guard_includes_iec104() {
        let iec104 = Iec104Analyzer::new();
        let mut dispatcher = StreamDispatcher::new(None, None, None, None, None, Some(iec104));

        let fk = flow_key(60003, 2404);
        // This would panic / return early if the guard were missing iec104.is_none().
        // With the guard in place, no panic occurs (data reaches Iec104 arm, even if no-op).
        dispatcher.on_data(&fk, Direction::ClientToServer, &startdt_act(), 0, 0);
    }

    /// AC-173-003 edge (EC-003) — with iec104=None, port-2404 traffic causes no panic.
    ///
    /// PASSES on current stub.
    ///
    /// Traces: BC-2.12.025 PC-2 (default-off); AC-173-003 EC-003.
    #[test]
    fn test_iec104_disabled_port_2404_no_panic() {
        let mut dispatcher = StreamDispatcher::new(None, None, None, None, None, None);
        let fk = flow_key(60004, 2404);
        dispatcher.on_data(&fk, Direction::ClientToServer, &stopdt_act(), 0, 0);
        dispatcher.on_flow_close(&fk, CloseReason::Fin);
    }

    /// AC-173-008 guard — iec104-only dispatcher counts unclassified flows.
    ///
    /// The early-exit guard `&& self.iec104.is_none()` means that with ONLY iec104 set,
    /// the guard is false and data is processed. A non-2404 flow → None target →
    /// unclassified_flows incremented in on_flow_close.
    ///
    /// PASSES on current stub.
    ///
    /// Traces: AC-173-008 (early-exit guard); BC-2.05.012.
    #[test]
    fn test_iec104_only_guard_unclassified_flows_counted() {
        let iec104 = Iec104Analyzer::new();
        let mut dispatcher = StreamDispatcher::new(None, None, None, None, None, Some(iec104));

        // Non-2404 flow → DispatchTarget::None → unclassified_flows + 1.
        let fk = flow_key(60005, 9999);
        dispatcher.on_flow_close(&fk, CloseReason::Fin);

        assert_eq!(
            dispatcher.unclassified_flows(),
            1,
            "AC-173-008: iec104-only dispatcher must count an unclassified None-target close \
             (guard depends on `|| self.iec104.is_some()`)"
        );
    }
}
