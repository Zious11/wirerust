use std::net::IpAddr;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::findings::{Confidence, ThreatCategory, Verdict};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

fn test_flow_key() -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        49153,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        80,
    )
}

fn test_flow_key_b() -> FlowKey {
    FlowKey::new(
        "192.168.1.1".parse::<IpAddr>().unwrap(),
        55000,
        "192.168.1.2".parse::<IpAddr>().unwrap(),
        8080,
    )
}

#[test]
fn test_http_analyzer_construction() {
    let analyzer = HttpAnalyzer::new();
    assert_eq!(analyzer.transaction_count(), 0);
}

#[test]
fn test_parse_get_request() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request =
        b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
    assert_eq!(*analyzer.host_counts().get("example.com").unwrap(), 1);
    assert_eq!(*analyzer.user_agent_counts().get("Mozilla/5.0").unwrap(), 1);
    assert_eq!(analyzer.uri_list(), &["/index.html"]);
}

#[test]
fn test_parse_pipelined_requests() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let pipelined = b"GET /first HTTP/1.1\r\nHost: a.com\r\n\r\nPOST /second HTTP/1.1\r\nHost: b.com\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, pipelined, 0);

    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
    assert_eq!(*analyzer.method_counts().get("POST").unwrap(), 1);
    assert_eq!(analyzer.uri_list().len(), 2);
}

#[test]
fn test_parse_partial_request() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        b"GET /page HTTP/1.1\r\nHos",
        0,
    );
    assert_eq!(analyzer.method_counts().get("GET"), None);

    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        b"t: example.com\r\n\r\n",
        23,
    );
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
}

#[test]
fn test_parse_response() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send request first
    let request = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    // Then response
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 5\r\n\r\nhello";
    analyzer.on_data(&fk, Direction::ServerToClient, response, 0);

    assert_eq!(*analyzer.status_code_counts().get(&200).unwrap(), 1);
    assert_eq!(analyzer.transaction_count(), 1);
}

#[test]
fn test_parse_pipelined_responses() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let requests = b"GET /a HTTP/1.1\r\nHost: x.com\r\n\r\nGET /b HTTP/1.1\r\nHost: x.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, requests, 0);

    let responses = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\nHTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, responses, 0);

    assert_eq!(*analyzer.status_code_counts().get(&200).unwrap(), 1);
    assert_eq!(*analyzer.status_code_counts().get(&404).unwrap(), 1);
    assert_eq!(analyzer.transaction_count(), 2);
}

#[test]
fn test_detect_path_traversal() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"GET /../../etc/passwd HTTP/1.1\r\nHost: target.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Reconnaissance);
    assert_eq!(findings[0].verdict, Verdict::Likely);
    assert_eq!(findings[0].confidence, Confidence::High);
}

#[test]
fn test_detect_encoded_traversal() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"GET /..%2f..%2fetc/passwd HTTP/1.1\r\nHost: target.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    assert!(
        !analyzer.findings().is_empty(),
        "Should detect encoded path traversal"
    );
}

#[test]
fn test_detect_webshell_path() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"GET /uploads/c99.php HTTP/1.1\r\nHost: target.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Execution);
}

#[test]
fn test_detect_unusual_method() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"CONNECT proxy.example.com:443 HTTP/1.1\r\nHost: proxy.example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Reconnaissance);
}

#[test]
fn test_detect_missing_host_header() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"GET /path HTTP/1.1\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    let findings = analyzer.findings();
    assert!(
        findings
            .iter()
            .any(|f| f.category == ThreatCategory::Anomaly),
        "Should detect missing Host header"
    );
}

#[test]
fn test_no_findings_for_normal_request() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request =
        b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    assert!(
        analyzer.findings().is_empty(),
        "Normal request should produce no findings"
    );
}

#[test]
fn test_summarize_produces_complete_output() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\nUser-Agent: TestBot\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, response, 0);

    let summary = analyzer.summarize();
    assert_eq!(summary.analyzer_name, "HTTP");
    assert_eq!(summary.packets_analyzed, 1);

    let detail = &summary.detail;
    assert_eq!(detail["transactions"], 1);
    assert_eq!(detail["methods"]["GET"], 1);
    assert_eq!(detail["status_codes"]["200"], 1);
    assert!(
        detail["top_hosts"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("example.com"))
    );
    assert_eq!(detail["user_agents"]["TestBot"], 1);
}

#[test]
fn test_flow_close_cleans_up_state() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET / HTTP/1.1\r\nHost: x.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    analyzer.on_flow_close(&fk, CloseReason::Fin);

    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        b"GET /new HTTP/1.1\r\nHost: y.com\r\n\r\n",
        0,
    );
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 2);
    assert_eq!(*analyzer.host_counts().get("y.com").unwrap(), 1);
}

#[test]
fn test_parse_error_increments_counter() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // "NOT_HTTP\r\n\r\n" triggers httparse::Error::Token
    analyzer.on_data(&fk, Direction::ClientToServer, b"NOT_HTTP\r\n\r\n", 0);

    assert_eq!(analyzer.parse_error_count(), 1);
    // Token error should NOT generate a finding (only TooManyHeaders does)
    assert!(analyzer.findings().is_empty());
}

#[test]
fn test_parse_error_in_summarize() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    analyzer.on_data(&fk, Direction::ClientToServer, b"NOT_HTTP\r\n\r\n", 0);

    let summary = analyzer.summarize();
    assert_eq!(summary.detail["parse_errors"], 1);
}

#[test]
fn test_too_many_headers_generates_finding() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Build a request with 97 headers to exceed MAX_HEADERS (96)
    let mut request = b"GET / HTTP/1.1\r\n".to_vec();
    for i in 0..97 {
        request.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
    }
    request.extend_from_slice(b"\r\n");

    analyzer.on_data(&fk, Direction::ClientToServer, &request, 0);

    assert_eq!(analyzer.parse_error_count(), 1);
    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Anomaly);
    assert_eq!(findings[0].verdict, Verdict::Inconclusive);
    assert_eq!(findings[0].confidence, Confidence::Medium);
    assert_eq!(findings[0].mitre_technique.as_deref(), Some("T1499.002"));
    assert!(findings[0].summary.contains("Excessive HTTP headers"));
    assert!(findings[0].evidence[0].contains("request"));
}

#[test]
fn test_parse_error_in_response() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send valid request first
    let request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    // Send malformed response
    analyzer.on_data(&fk, Direction::ServerToClient, b"NOT_HTTP\r\n\r\n", 0);

    assert_eq!(analyzer.parse_error_count(), 1);
    // Token error on response should NOT generate a finding
    assert!(analyzer.findings().is_empty());
}

#[test]
fn test_parse_error_poisons_direction_after_threshold() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send 3 consecutive errors to reach POISON_THRESHOLD
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE1\r\n\r\n", 0);
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE2\r\n\r\n", 0);
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE3\r\n\r\n", 0);
    assert_eq!(analyzer.parse_error_count(), 3);

    // Fourth: valid request — skipped because direction is now poisoned
    let valid = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let skipped_before = analyzer.poisoned_bytes_skipped();
    analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);

    assert_eq!(analyzer.parse_error_count(), 3); // no new errors (poisoned, not retried)
    assert!(analyzer.method_counts().get("GET").is_none()); // never parsed
    assert_eq!(
        analyzer.poisoned_bytes_skipped(),
        skipped_before + valid.len() as u64
    );
}

#[test]
fn test_single_error_does_not_poison() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // One error is below threshold — should NOT poison
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    assert_eq!(analyzer.parse_error_count(), 1);

    // Valid request should still parse (direction not poisoned yet)
    let valid = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);

    assert_eq!(analyzer.parse_error_count(), 1);
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
}

#[test]
fn test_poison_request_does_not_affect_response() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Poison request direction (3 errors)
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE1\r\n\r\n", 0);
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE2\r\n\r\n", 0);
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE3\r\n\r\n", 0);
    assert_eq!(analyzer.parse_error_count(), 3);

    // Response direction should still work
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, response, 0);
    assert_eq!(analyzer.transaction_count(), 1);
    assert_eq!(*analyzer.status_code_counts().get(&200).unwrap(), 1);
}

#[test]
fn test_non_http_flows_counts_per_flow_not_direction() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Poison request direction (3 errors)
    for _ in 0..3 {
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    }
    // Poison response direction (3 errors)
    for _ in 0..3 {
        analyzer.on_data(&fk, Direction::ServerToClient, b"GARBAGE\r\n\r\n", 0);
    }

    // Both directions poisoned, but non_http_flows should count 1 flow, not 2
    let summary = analyzer.summarize();
    assert_eq!(summary.detail["non_http_flows"], serde_json::json!(1));
}

#[test]
fn test_poison_cleared_after_flow_close() {
    use wirerust::reassembly::handler::CloseReason;

    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Poison request direction (3 errors)
    for _ in 0..3 {
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    }

    // Close the flow
    analyzer.on_flow_close(&fk, CloseReason::Fin);

    // Reopen same 4-tuple — should NOT be poisoned
    let valid = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
}

#[test]
fn test_normal_request_no_parse_errors() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request =
        b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    assert_eq!(analyzer.parse_error_count(), 0);
    assert!(analyzer.findings().is_empty());
}

#[test]
fn test_too_many_headers_in_response_generates_finding() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Build a response with 97 headers to exceed MAX_HEADERS (96)
    let mut response = b"HTTP/1.1 200 OK\r\n".to_vec();
    for i in 0..97 {
        response.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
    }
    response.extend_from_slice(b"\r\n");

    analyzer.on_data(&fk, Direction::ServerToClient, &response, 0);

    assert_eq!(analyzer.parse_error_count(), 1);
    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Anomaly);
    assert!(findings[0].evidence[0].contains("response"));
}

#[test]
fn test_multiple_parse_errors_accumulate() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // First error: malformed request
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    assert_eq!(analyzer.parse_error_count(), 1);

    // Second error: malformed response
    analyzer.on_data(&fk, Direction::ServerToClient, b"ALSO_BAD\r\n\r\n", 0);
    assert_eq!(analyzer.parse_error_count(), 2);
}

#[test]
fn test_body_bytes_do_not_inflate_parse_errors() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Request with no body
    let request = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    // Response WITH body "hello" (Content-Length: 5)
    // Body bytes remain in buffer after header parse and would previously
    // be re-parsed as HTTP, triggering a false parse error.
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 5\r\n\r\nhello";
    analyzer.on_data(&fk, Direction::ServerToClient, response, 0);

    assert_eq!(analyzer.parse_error_count(), 0);
    assert!(analyzer.findings().is_empty());
}

#[test]
fn test_cross_flow_isolation_parse_errors() {
    let mut analyzer = HttpAnalyzer::new();
    let flow_a = test_flow_key();
    let flow_b = test_flow_key_b();

    // Send malformed data on flow A
    analyzer.on_data(&flow_a, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    assert_eq!(analyzer.parse_error_count(), 1);

    // Send valid request on flow B — should parse successfully
    let valid = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&flow_b, Direction::ClientToServer, valid, 0);

    assert_eq!(analyzer.parse_error_count(), 1); // only from flow A
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
}

#[test]
fn test_cross_flow_isolation_poisoning() {
    let mut analyzer = HttpAnalyzer::new();
    let flow_a = test_flow_key();
    let flow_b = test_flow_key_b();

    // Poison flow A (3 consecutive errors)
    for _ in 0..3 {
        analyzer.on_data(&flow_a, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    }
    assert_eq!(analyzer.parse_error_count(), 3);

    // Flow B should be completely unaffected
    let valid = b"GET /page HTTP/1.1\r\nHost: other.com\r\n\r\n";
    analyzer.on_data(&flow_b, Direction::ClientToServer, valid, 0);

    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);

    // Verify flow A is poisoned (data skipped, bytes counted)
    let skipped_before = analyzer.poisoned_bytes_skipped();
    analyzer.on_data(&flow_a, Direction::ClientToServer, b"more data", 0);
    assert_eq!(
        analyzer.poisoned_bytes_skipped(),
        skipped_before + b"more data".len() as u64
    );
}

// ---------------------------------------------------------------------------
// Issue #20: missing HTTP analyzer test coverage
// ---------------------------------------------------------------------------

#[test]
fn test_buffer_cap_no_panic_on_oversized_headers() {
    // MAX_HEADER_BUF is 65_536. Data beyond this limit is silently
    // truncated — the buffer must not grow unbounded. To prove the cap
    // is enforced, we first send an oversized partial header, then send
    // the missing terminator on the SAME flow. If the buffer had been
    // allowed to retain all bytes, the second chunk would complete the
    // request; with truncation, it must remain unparsed.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Use a single header with a massive value to exceed 64KB without
    // hitting MAX_HEADERS (96). The request line + Host: header + the
    // large X-Big header totals > 65536 bytes, so the buffer truncates
    // mid-value.
    let mut oversized = b"GET / HTTP/1.1\r\nHost: example.com\r\nX-Big: ".to_vec();
    oversized.extend_from_slice(&vec![b'A'; 70_000]);
    // No \r\n\r\n yet.

    analyzer.on_data(&fk, Direction::ClientToServer, &oversized, 0);

    // The oversized partial request should not parse.
    assert!(
        analyzer.method_counts().get("GET").is_none(),
        "oversized partial request should not be counted as parsed"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "partial header from buffer cap should not count as a parse error"
    );

    // Now try to complete the same request on the SAME flow. If the full
    // oversized buffer had been retained, this would complete parsing.
    // Because the buffer is capped/truncated, the terminator is silently
    // dropped (remaining capacity is 0), and the request stays unparsed.
    let completion = b"\r\n\r\n";
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        completion,
        oversized.len() as u64,
    );

    assert!(
        analyzer.method_counts().get("GET").is_none(),
        "same-flow completion after buffer-cap truncation must not produce a parsed request"
    );
    assert!(
        analyzer.findings().is_empty(),
        "truncated partial data should not produce findings"
    );

    // Subsequent valid data on a NEW flow should still work (analyzer not corrupted).
    let fk2 = test_flow_key_b();
    let valid = b"GET /ok HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk2, Direction::ClientToServer, valid, 0);
    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap(),
        1,
        "valid request on a different flow should parse after buffer-cap hit"
    );
}

#[test]
fn test_detect_long_uri() {
    // URIs > 2048 chars should trigger an Execution finding with the
    // URI length in the summary and a truncated prefix in evidence.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let long_path = "/".to_string() + &"A".repeat(2100);
    let request = format!("GET {long_path} HTTP/1.1\r\nHost: target.com\r\n\r\n");
    analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

    let findings = analyzer.findings();
    let long_uri_finding = findings
        .iter()
        .find(|f| f.summary.contains("Abnormally long URI"))
        .expect("expected a long-URI finding for URI > 2048 chars");
    assert_eq!(long_uri_finding.category, ThreatCategory::Execution);
    assert_eq!(long_uri_finding.verdict, Verdict::Likely);
    assert_eq!(long_uri_finding.confidence, Confidence::Medium);
    assert!(
        long_uri_finding.summary.contains("2101 chars"),
        "summary should include the URI length, got: {}",
        long_uri_finding.summary
    );
    assert!(
        long_uri_finding.evidence[0].starts_with("URI prefix:"),
        "evidence should contain truncated URI prefix, got: {}",
        long_uri_finding.evidence[0]
    );
}

#[test]
fn test_detect_empty_user_agent() {
    // An empty User-Agent header (present but "") should trigger an
    // Anomaly finding. This is more suspicious than a missing UA —
    // real browsers always populate it, and even common tools
    // (curl, wget, Python requests) send a default string.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\nUser-Agent: \r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let findings = analyzer.findings();
    let ua_finding = findings
        .iter()
        .find(|f| f.summary.contains("Empty User-Agent"))
        .expect("expected an empty-UA finding");
    assert_eq!(ua_finding.category, ThreatCategory::Anomaly);
    assert_eq!(ua_finding.verdict, Verdict::Inconclusive);
    assert_eq!(ua_finding.confidence, Confidence::Low);
}

#[test]
fn test_missing_user_agent_no_finding() {
    // A missing User-Agent header (not present at all) should NOT
    // trigger the empty-UA finding. The detection specifically checks
    // for Some(""), not None.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    assert!(
        !analyzer
            .findings()
            .iter()
            .any(|f| f.summary.contains("User-Agent")),
        "missing (absent) User-Agent should not trigger empty-UA finding"
    );
}

#[test]
fn test_detect_admin_panel_paths() {
    // Admin panel URIs should trigger Reconnaissance findings.
    let patterns = [
        "/wp-admin/index.php",
        "/admin/dashboard",
        "/phpmyadmin/",
        "/manager/html",
    ];

    for pattern in &patterns {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        let request = format!("GET {pattern} HTTP/1.1\r\nHost: target.com\r\n\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

        let findings = analyzer.findings();
        let admin_finding = findings
            .iter()
            .find(|f| f.summary.contains("Admin panel"))
            .unwrap_or_else(|| panic!("expected admin-panel finding for URI {pattern}"));
        assert_eq!(
            admin_finding.category,
            ThreatCategory::Reconnaissance,
            "admin panel finding for {pattern} should be Reconnaissance"
        );
        assert_eq!(
            admin_finding.verdict,
            Verdict::Inconclusive,
            "admin panel finding for {pattern} should be Inconclusive"
        );
        assert_eq!(
            admin_finding.confidence,
            Confidence::Low,
            "admin panel finding for {pattern} should be Low confidence"
        );
        assert_eq!(
            admin_finding.mitre_technique.as_deref(),
            Some("T1046"),
            "admin panel finding for {pattern} should map to T1046"
        );
    }
}

#[test]
fn test_partial_response_reassembly() {
    // Split a response header across two on_data calls. The parser
    // should buffer the first partial chunk and complete the parse
    // when the rest arrives.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send a request first so the response direction is active.
    let request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    // Split response across two chunks mid-header.
    let part1 = b"HTTP/1.1 200 OK\r\nContent-Len";
    let part2 = b"gth: 0\r\n\r\n";

    analyzer.on_data(&fk, Direction::ServerToClient, part1, 0);
    // After part1: should be Partial — no transaction yet.
    assert_eq!(
        analyzer.transaction_count(),
        0,
        "partial response should not complete a transaction"
    );

    analyzer.on_data(&fk, Direction::ServerToClient, part2, part1.len() as u64);
    // After part2: response fully assembled → transaction counted.
    assert_eq!(
        analyzer.transaction_count(),
        1,
        "completed response should count as a transaction"
    );
    assert_eq!(
        *analyzer.status_code_counts().get(&200).unwrap(),
        1,
        "status code 200 should be recorded"
    );
}
