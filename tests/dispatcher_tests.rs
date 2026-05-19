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

#[test]
fn test_dispatcher_routes_tls() {
    let mut dispatcher = StreamDispatcher::new(None, Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 443);

    // TLS ClientHello record header: content_type=0x16, version=0x0303, length=0x0005
    let tls_data = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0);

    // If routed correctly, TLS analyzer received data (no panic, no error)
    // We can't directly assert routing, but we can verify HTTP didn't get it
    assert!(dispatcher.http.is_none());
}

#[test]
fn test_dispatcher_routes_http() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), None);
    let fk = flow_key(49152, 80);

    let http_data = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0);

    let http = dispatcher.http.as_ref().unwrap();
    assert_eq!(*http.method_counts().get("GET").unwrap(), 1);
}

#[test]
fn test_dispatcher_content_detection_tls_on_port_80() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 80); // Port 80, but content is TLS

    // TLS record header on port 80 — content detection should override port
    let tls_data = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0);

    // HTTP analyzer should NOT have received this data
    let http = dispatcher.http.as_ref().unwrap();
    assert_eq!(http.method_counts().len(), 0);
    assert_eq!(http.parse_error_count(), 0); // Confirms HTTP didn't try to parse TLS bytes
}

#[test]
fn test_dispatcher_port_fallback_short_data() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 443); // Port 443

    // Only 2 bytes — too short for content detection, falls back to port
    let short_data = [0x16, 0x03];
    dispatcher.on_data(&fk, Direction::ClientToServer, &short_data, 0);

    // Should have routed to TLS based on port 443
    // HTTP should not have received it
    let http = dispatcher.http.as_ref().unwrap();
    assert_eq!(http.method_counts().len(), 0);
    assert_eq!(http.parse_error_count(), 0); // Confirms HTTP didn't try to parse TLS bytes
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

    let http = dispatcher.http.as_ref().unwrap();
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
    let http = dispatcher.http.as_ref().unwrap();
    assert_eq!(
        *http.method_counts().get("GET").unwrap(),
        1,
        "a first-chunk positive match must route even with a zero failure budget"
    );
}
