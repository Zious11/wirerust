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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 8080 would fall back to Http by port — if content wins, Tls is chosen instead.
    let fk = flow_key(49152, 8080);

    // Canonical test vector from BC-2.05.001: [0x16, 0x03, 0x03, 0x00, 0x50, ...]
    let tls_data = [0x16u8, 0x03, 0x03, 0x00, 0x50, 0x01, 0x00, 0x00, 0x4c, 0x03];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 443);

    let tls_data = [0x16u8, 0x03, 0x03, 0x00, 0x50, 0x01, 0x00, 0x00, 0x4c, 0x03];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), None);
    let fk = flow_key(49152, 80);

    let http_data = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0);

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
        let mut dispatcher =
            StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
        // Port 9999: no port fallback hint — Http must be chosen by content.
        let fk = flow_key(49152 + i as u16, 9999);
        dispatcher.on_data(&fk, Direction::ClientToServer, data, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 80); // Port 80, but content is TLS

    // TLS record header on port 80 — content detection should override port
    let tls_data = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0);

    // HTTP analyzer should NOT have received this data
    let http = dispatcher.http_analyzer().unwrap();
    assert_eq!(http.method_counts().len(), 0);
    assert_eq!(http.parse_error_count(), 0); // Confirms HTTP didn't try to parse TLS bytes
}

/// AC-007 (BC-2.05.003 postcondition 1): When both content checks fail (data has
/// no TLS/HTTP magic bytes), port fallback fires. Port 443 → DispatchTarget::Tls.
#[test]
fn test_port_fallback_443_to_tls() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 443); // Port 443

    // 6 bytes: TLS record type 0x16 but version 0x0401 (not 0x0300–0x0303) so content
    // detection (which requires data[1]==0x03) does NOT fire; only port fallback applies.
    // The 1-byte payload (0xFF) forms a syntactically complete but malformed handshake
    // record, which causes TlsAnalyzer to increment parse_error_count — confirming routing.
    let unknown_data = [0x16u8, 0x04, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_data, 0);

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
    // Positive TLS discriminator: non-TLS garbage routed to TlsAnalyzer triggers a
    // parse/truncation event — proves TlsAnalyzer actually received the bytes.
    let tls = dispatcher.tls_analyzer().unwrap();
    assert!(
        tls.parse_error_count() > 0 || tls.truncated_record_count() > 0,
        "AC-007: port 443 fallback must route to Tls analyzer \
         (6-byte non-TLS garbage triggers TlsAnalyzer parse/truncation event)"
    );
}

/// AC-007 (BC-2.05.003 postcondition 1): Port 8443 → DispatchTarget::Tls via port fallback.
/// 6-byte non-TLS, non-HTTP data ensures neither content check fires.
#[test]
fn test_port_fallback_8443_to_tls() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 8443 is a known TLS port; data has no TLS/HTTP signature.
    let fk = flow_key(49152, 8443);

    // 6 bytes: TLS record type 0x16 but version 0x0401 (not 0x0300–0x0303) so content
    // detection (which requires data[1]==0x03) does NOT fire; only port fallback applies.
    // The 1-byte payload (0xFF) forms a complete but malformed handshake record, causing
    // TlsAnalyzer to increment parse_error_count — confirming routing to TLS analyzer.
    let ambiguous_data = [0x16u8, 0x04, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &ambiguous_data, 0);

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
    // Positive TLS discriminator: non-TLS garbage routed to TlsAnalyzer triggers a
    // parse/truncation event — proves TlsAnalyzer actually received the bytes.
    let tls = dispatcher.tls_analyzer().unwrap();
    assert!(
        tls.parse_error_count() > 0 || tls.truncated_record_count() > 0,
        "AC-007: port 8443 fallback must route to Tls analyzer \
         (6-byte non-TLS garbage triggers TlsAnalyzer parse/truncation event)"
    );
}

/// AC-007 (BC-2.05.003 postcondition 2): Port 80 → DispatchTarget::Http via port fallback.
/// 5-byte non-TLS, non-HTTP data ensures neither content check fires.
#[test]
fn test_port_fallback_80_to_http() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 80 is a known HTTP port; data has no TLS/HTTP signature.
    let fk = flow_key(49152, 80);

    // 5 bytes with no TLS (byte0≠0x16) and no HTTP method prefix → only port fallback applies.
    let ambiguous_data = [0x00u8, 0x01, 0x02, 0x03, 0x04];
    dispatcher.on_data(&fk, Direction::ClientToServer, &ambiguous_data, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 8080 is a known HTTP port; data has no TLS/HTTP signature.
    let fk = flow_key(49152, 8080);

    // 5 bytes with no TLS (byte0≠0x16) and no HTTP method prefix → only port fallback applies.
    let ambiguous_data = [0x00u8, 0x01, 0x02, 0x03, 0x04];
    dispatcher.on_data(&fk, Direction::ClientToServer, &ambiguous_data, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 9999: no port fallback hint — isolates the length-gate from port fallback.
    let fk = flow_key(49152, 9999);

    // 4 bytes starting with TLS-looking byte0=0x16 — would pass TLS check IF 5 bytes present.
    // Exactly at the EC-004 boundary: data.len() == 4.
    let four_bytes = [0x16u8, 0x03, 0x03, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &four_bytes, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 9999: no port fallback hint.
    let fk = flow_key(49152, 9999);

    // byte0=0x16, byte1=0x04 (not 0x03) — TLS check must fail.
    let almost_tls = [0x16u8, 0x04, 0x03, 0x00, 0x05];
    dispatcher.on_data(&fk, Direction::ClientToServer, &almost_tls, 0);

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
    let mut dispatcher2 =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk2 = flow_key(49152, 9999);
    let almost_tls2 = [0x16u8, 0x02, 0x03, 0x00, 0x05];
    dispatcher2.on_data(&fk2, Direction::ClientToServer, &almost_tls2, 0);
    dispatcher2.on_flow_close(&fk2, CloseReason::Fin);
    assert_eq!(
        dispatcher2.unclassified_flows(),
        1,
        "EC-005 variant: byte1=0x02 (not 0x03) must not trigger TLS routing; flow unclassified"
    );
}

#[test]
fn test_unclassified_flows_counter() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 9999); // Non-standard port

    // Send data that doesn't match HTTP or TLS content signatures
    dispatcher.on_data(&fk, Direction::ClientToServer, b"UNKNOWN_PROTOCOL", 0);
    assert_eq!(dispatcher.unclassified_flows(), 0); // Not counted until close

    // Close the flow — never classified
    dispatcher.on_flow_close(&fk, CloseReason::Fin);
    assert_eq!(dispatcher.unclassified_flows(), 1);
}

#[test]
fn test_classified_flow_not_counted_as_unclassified() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 80);

    let http_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0);
    dispatcher.on_flow_close(&fk, CloseReason::Fin);

    assert_eq!(dispatcher.unclassified_flows(), 0);
}

// ---- LESSON-P2.11: max_classification_attempts knob ----

#[test]
fn test_default_max_classification_attempts() {
    // The default cap is exposed and matches the documented constant.
    let dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), None);
    assert_eq!(
        dispatcher.max_classification_attempts(),
        wirerust::dispatcher::DEFAULT_MAX_CLASSIFICATION_ATTEMPTS
    );
}

#[test]
fn test_with_max_classification_attempts_overrides_default() {
    // The builder-style override sets a custom cap.
    let dispatcher =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None).with_max_classification_attempts(3);
    assert_eq!(dispatcher.max_classification_attempts(), 3);
}

#[test]
fn test_unclassifiable_flow_still_counted_after_attempt_cap() {
    // LESSON-P2.11: once a flow exceeds max_classification_attempts it
    // is permanently routed to None. It must still be counted as an
    // unclassified flow on close — the give-up branch must not lose
    // the flow from the accounting.
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
        .with_max_classification_attempts(3);
    let fk = flow_key(49152, 9999); // non-standard port, unknown content

    // Feed several non-HTTP, non-TLS chunks — well past the cap of 3.
    for _ in 0..10 {
        dispatcher.on_data(&fk, Direction::ClientToServer, b"UNKNOWN_PROTOCOL", 0);
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
        .with_max_classification_attempts(8);
    let fk = flow_key(49152, 9999);

    // Two unclassifiable chunks (within the budget of 8)...
    dispatcher.on_data(&fk, Direction::ClientToServer, b"\x00\x01", 0);
    dispatcher.on_data(&fk, Direction::ClientToServer, b"\x02\x03", 0);
    // ...then a clear HTTP request.
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        b"GET / HTTP/1.1\r\nHost: x\r\n\r\n",
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
        StreamDispatcher::new(Some(HttpAnalyzer::new()), None).with_max_classification_attempts(0);
    let fk = flow_key(49152, 80);

    let http_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0);
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 9999: no port fallback match, so the only way Http is chosen is content.
    let fk = flow_key(49152, 9999);

    // b"GET" without trailing space — must not match any HTTP prefix.
    dispatcher.on_data(&fk, Direction::ClientToServer, b"GET", 0);
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Neutral port (9999) — port fallback plays no part.
    let fk = flow_key(49152, 9999);

    // Construct data that starts with the TLS magic bytes (0x16 0x03) followed
    // by enough bytes to pass the len >= 5 gate. The remainder is irrelevant to
    // the routing decision, but we pad it to 10 bytes for completeness.
    let tls_then_garbage = [0x16u8, 0x03, 0x01, 0x00, 0x06, 0x47, 0x45, 0x54, 0x20, 0x2f];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_then_garbage, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));

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
    {
        let tls = dispatcher.tls_analyzer().unwrap();
        assert!(
            tls.parse_error_count() > 0 || tls.truncated_record_count() > 0,
            "AC-008: port 8443 canonical-ordering fallback must route to Tls analyzer \
             (6-byte non-TLS garbage triggers TlsAnalyzer parse/truncation event)"
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
    {
        let tls = dispatcher.tls_analyzer().unwrap();
        assert!(
            tls.parse_error_count() > 0 || tls.truncated_record_count() > 0,
            "AC-008: port 443 canonical-ordering fallback must route to Tls analyzer \
             (6-byte non-TLS garbage triggers TlsAnalyzer parse/truncation event)"
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 443 would fall back to Tls — but content check for HTTP must fire first.
    let fk = flow_key(49152, 443);

    let http_on_tls_port = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_on_tls_port, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    // Port 9999: no port fallback, so routing is content-only.
    let fk = flow_key(49152, 9999);

    // First chunk: valid HTTP GET — classify returns Http; cached in routes[fk].
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_bytes, 0);
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
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0);

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

    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
        .with_max_classification_attempts(3);
    let fk = flow_key(49152, 22);

    // Phase A: exhaust retry cap → None permanently cached in routes[fk].
    for _ in 0..3 {
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0);
    }
    // Sanity-check that None is cached: a TLS chunk must not reach TlsAnalyzer.
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0);
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
    // return Tls, and route data to TlsAnalyzer — producing at least one parse or truncation
    // event. If the stale None were still present, TlsAnalyzer would remain silent.
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0);
    assert!(
        dispatcher.tls_analyzer().unwrap().parse_error_count() > 0
            || dispatcher.tls_analyzer().unwrap().truncated_record_count() > 0,
        "EC-008/reclassify: after close, same FlowKey with TLS bytes must re-run classify; \
         TlsAnalyzer must receive data (stale None route was evicted, not reused)"
    );

    // Phase C: verify the reopened flow is now cached as Tls (not re-running classify on
    // every subsequent chunk). Send HTTP GET bytes — if Tls is cached, classify does NOT
    // re-run and HttpAnalyzer receives nothing (method_counts["GET"] stays 0).
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_bytes, 0);
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
        .with_max_classification_attempts(8);
    // Port 22 (SSH): not in {80, 443, 8080, 8443} → port fallback also fails → None.
    let fk = flow_key(49152, 22);

    // AC-006: 7 unmatched chunks (7 < cap of 8) — None NOT yet cached.
    let unknown_bytes: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    for _ in 0..7 {
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0);
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
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0);

    assert!(
        dispatcher.tls_analyzer().unwrap().parse_error_count() > 0
            || dispatcher.tls_analyzer().unwrap().truncated_record_count() > 0,
        "AC-003/AC-006: None must NOT be cached after 7 attempts (cap=8); \
         8th chunk with TLS bytes must re-run classify, route to TlsAnalyzer, \
         and produce a parse/truncation event"
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
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
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0);
    }

    // Chunk 4: valid TLS bytes. If None is permanently cached (correct), classify does
    // NOT run → TlsAnalyzer receives nothing → both parse_error_count and
    // truncated_record_count remain 0. If the cache were broken, TlsAnalyzer would fire.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0);

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
    let mut d_zero = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
        .with_max_classification_attempts(0);
    let fk2 = flow_key(49152, 22);
    // First chunk: unknown bytes → count would be 1, but 1 >= 0 after saturating, so
    // the implementation uses `>= max` check; with max=0, count(1) >= 0 → None cached.
    d_zero.on_data(&fk2, Direction::ClientToServer, &unknown_bytes, 0);
    // Second chunk: TLS bytes — must not reach TlsAnalyzer (None cached after chunk 1).
    d_zero.on_data(&fk2, Direction::ClientToServer, &tls_bytes, 0);
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
    let mut d_default = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
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
        d_default.on_data(&fk3, Direction::ClientToServer, &unknown_bytes_default, 0);
    }

    // Verify None is now permanently cached: a 9th chunk with valid TLS bytes must NOT
    // reach TlsAnalyzer (classify is short-circuited by the cached None route).
    d_default.on_data(&fk3, Direction::ClientToServer, &tls_bytes_default, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
        .with_max_classification_attempts(8);
    let fk = flow_key(49152, 22);

    let unknown_bytes: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    // 3 unmatched chunks — attempt count reaches 3, still below cap of 8.
    for _ in 0..3 {
        dispatcher.on_data(&fk, Direction::ClientToServer, &unknown_bytes, 0);
    }

    // 4th chunk: TLS bytes — classify returns Tls; routes[fk]=Tls cached; attempts removed.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_bytes, 0);

    assert!(
        dispatcher.tls_analyzer().unwrap().parse_error_count() > 0
            || dispatcher.tls_analyzer().unwrap().truncated_record_count() > 0,
        "AC-009/EC-006: TLS bytes on 4th call (3 prior Nones, cap=8) must classify as Tls \
         and route to TlsAnalyzer"
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
    dispatcher.on_data(&fk, Direction::ClientToServer, http_bytes, 0);

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
    let mut d2 = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
        .with_max_classification_attempts(8);
    let fk2 = flow_key(49153, 22);

    for _ in 0..7 {
        d2.on_data(&fk2, Direction::ClientToServer, &unknown_bytes, 0);
    }
    // 8th chunk: TLS bytes — attempt count was 7 (< cap=8); classify runs; returns Tls.
    d2.on_data(&fk2, Direction::ClientToServer, &tls_bytes, 0);

    assert!(
        d2.tls_analyzer().unwrap().parse_error_count() > 0
            || d2.tls_analyzer().unwrap().truncated_record_count() > 0,
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
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
    let mut dispatcher2 =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()))
            .with_max_classification_attempts(2);

    // Two unknown-content chunks → attempt count reaches cap=2 → DispatchTarget::None cached.
    let unknown: &[u8] = &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    dispatcher2.on_data(&fk_capped, Direction::ClientToServer, unknown, 0);
    dispatcher2.on_data(&fk_capped, Direction::ClientToServer, unknown, 0);

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
    let mut dispatcher3 =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));

    // Part 1: HTTP-classified flow.
    let fk_http = flow_key(49210, 9999);
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk_http, Direction::ClientToServer, http_bytes, 0);

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
    dispatcher.on_data(&fk_tls, Direction::ClientToServer, &tls_bytes, 0);

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

// STORY-033 AC-004 + AC-005 (early-return aspect): StreamDispatcher::new(None, None) returns
// immediately from on_data before any classify or state mutation. Indirect proof via
// observing that routes/attempts maps remain empty (unclassified_flows stays 0 even
// on close, because the guard also prevents incrementing when no analyzers are configured).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_05_008_no_analyzer_dispatcher_early_returns() {
    let mut dispatcher = StreamDispatcher::new(None, None);
    let fk = flow_key(49220, 9999);

    // Call on_data multiple times with various byte patterns — must be no-ops.
    dispatcher.on_data(&fk, Direction::ClientToServer, b"GET / HTTP/1.1\r\n", 0);
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        &[0x16u8, 0x03, 0x01, 0x00, 0x01, 0xFF],
        0,
    );
    dispatcher.on_data(
        &fk,
        Direction::ClientToServer,
        &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE],
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
    let mut dispatcher_http_only = StreamDispatcher::new(Some(HttpAnalyzer::new()), None);
    let fk_http = flow_key(49230, 9999);
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher_http_only.on_data(&fk_http, Direction::ClientToServer, http_bytes, 0);

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
    let mut dispatcher_tls_only = StreamDispatcher::new(None, Some(TlsAnalyzer::new()));
    let fk_tls = flow_key(49231, 9999);
    // Valid-length TLS-like bytes: record_type=0x16, version=0x0301, payload_len=1 byte.
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];
    dispatcher_tls_only.on_data(&fk_tls, Direction::ClientToServer, &tls_bytes, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk_http = flow_key(49240, 9999);
    let http_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

    dispatcher.on_data(&fk_http, Direction::ClientToServer, http_bytes, 0);

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
    let mut dispatcher2 =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk_tls = flow_key(49241, 9999);
    let tls_bytes: [u8; 6] = [0x16, 0x03, 0x01, 0x00, 0x01, 0xFF];

    dispatcher2.on_data(&fk_tls, Direction::ClientToServer, &tls_bytes, 0);

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
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));

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
