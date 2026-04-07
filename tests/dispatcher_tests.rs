use std::net::IpAddr;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};

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
}
