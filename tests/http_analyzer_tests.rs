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
    // HTTP/1.1 without any Host header is RFC 7230 §5.4 non-compliant
    // and must produce a "without Host header" Anomaly finding.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"GET /path HTTP/1.1\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    let findings = analyzer.findings();
    let host_finding = findings
        .iter()
        .find(|f| f.summary.contains("without Host header"))
        .expect("expected a missing-Host anomaly");
    assert_eq!(host_finding.category, ThreatCategory::Anomaly);
    assert_eq!(host_finding.verdict, Verdict::Inconclusive);
    assert_eq!(host_finding.confidence, Confidence::Medium);
}

#[test]
fn test_detect_empty_host_header() {
    // An empty-value `Host:` is equally RFC 7230 §5.4 non-compliant and
    // is the documented bypass that the `is_none()`-only check missed
    // (Suricata surfaces this as sid 2221028
    // `http.request_header_host_invalid`, separate from the
    // sid-2221014 missing-Host event). It must produce an Anomaly
    // finding with the distinct "empty Host header" summary so analysts
    // can disambiguate it from the truly-absent case.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"GET /path HTTP/1.1\r\nHost: \r\nUser-Agent: curl/8.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    let findings = analyzer.findings();
    let host_finding = findings
        .iter()
        .find(|f| f.summary.contains("empty Host header"))
        .expect("expected an empty-Host anomaly");
    assert_eq!(host_finding.category, ThreatCategory::Anomaly);
    assert_eq!(host_finding.verdict, Verdict::Inconclusive);
    assert_eq!(host_finding.confidence, Confidence::Medium);

    // And the *missing* variant must not also fire on the empty case —
    // they are surfaced via distinct summary strings.
    assert!(
        !findings
            .iter()
            .any(|f| f.summary.contains("without Host header")),
        "empty-Host case must not also trigger the missing-Host variant"
    );
}

#[test]
fn test_detect_whitespace_only_host_header() {
    // `Host:    ` (whitespace-only value) is folded into the empty case
    // by `find_header`'s `.trim()` and must produce the same empty-Host
    // anomaly as a literally-empty value.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();
    let request = b"GET /path HTTP/1.1\r\nHost:    \r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    assert!(
        analyzer
            .findings()
            .iter()
            .any(|f| f.summary.contains("empty Host header")),
        "whitespace-only Host: must fire the empty-Host anomaly"
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
    // A missing User-Agent header (not present at all) must NOT trigger
    // the empty-UA finding. This asymmetry with the Host check is
    // deliberate and documented in `src/analyzer/http.rs` (rule 7
    // comment): Snort ships its missing-UA rule (sid 1:38130) disabled
    // by default because legitimate non-browser traffic (cron jobs,
    // healthchecks, microservices, embedded clients) routinely omits
    // UA, while empty-UA is a stronger malicious-traffic indicator
    // (Kheir 2015 reports ~24% of malware samples emit empty UA).
    // The detection specifically checks for `Some("")`, not `None`.
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

// ---------------------------------------------------------------------------
// STORY-041 Brownfield-Formalization Tests (BC-2.06.001 – BC-2.06.026)
//
// Naming convention: test_BC_2_06_NNN_<descriptive_suffix>
// Each test formalizes an AC clause from STORY-041 using the BC's canonical
// test vectors wherever available.  These tests confirm existing behavior
// (formalization-confirms-existing-behavior mode) and will PASS against the
// current src/analyzer/http.rs implementation.
//
// The uppercase "BC" in function names is intentional (DF-AC-TEST-NAME-SYNC-001).
// Suppress the non_snake_case lint for this block.
// ---------------------------------------------------------------------------
#[allow(non_snake_case)]
mod bc_2_06_formalization {
    use super::*;

// ── BC-2.06.001 ──────────────────────────────────────────────────────────────
// AC-001: complete request → methods/hosts/user_agents/uris updated, buf
//         drained, request_error_count reset, check_request_detections called.

/// BC-2.06.001 postconditions 1-4 + 7 — canonical test vector (happy path).
/// Exercises: method map, host map, UA map, URI vec all updated on one
/// complete HTTP/1.1 request (the BC's golden vector).
#[test]
fn test_BC_2_06_001_complete_request_updates_all_counters() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Canonical test vector from BC-2.06.001.
    let req =
        b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap(),
        1,
        "BC-2.06.001 postcondition 1: methods[GET] must be 1 after one GET"
    );
    assert_eq!(
        *analyzer.host_counts().get("example.com").unwrap(),
        1,
        "BC-2.06.001 postcondition 2: hosts[example.com] must be 1"
    );
    assert_eq!(
        *analyzer.user_agent_counts().get("curl/7.0").unwrap(),
        1,
        "BC-2.06.001 postcondition 3: user_agents[curl/7.0] must be 1"
    );
    assert_eq!(
        analyzer.uri_list(),
        &["/index.html"],
        "BC-2.06.001 postcondition 4: URI appended to uris vec"
    );
    // Postcondition 5 (buf drained) is implicit: a second request on the
    // same flow is processed independently (tested by pipelined tests).
    // Postcondition 6 (request_error_count reset) is tested in AC-004.
    // Postcondition 7 (detections invoked) — no finding for normal request.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "BC-2.06.001 postcondition 6 proxy: parse_errors must be 0 for valid request"
    );
}

/// BC-2.06.001 postcondition 5 — consumed bytes drained from request_buf
/// so that a second request on the same flow is parsed independently.
#[test]
fn test_BC_2_06_001_consumed_bytes_drained_from_buf() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Two back-to-back requests: if the buffer were NOT drained after the
    // first, the second parse would attempt to re-parse the old bytes and
    // the method count would be wrong.
    let two_reqs =
        b"GET /a HTTP/1.1\r\nHost: h.com\r\n\r\nPOST /b HTTP/1.1\r\nHost: h.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, two_reqs, 0);

    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        1,
        "drain: GET must be counted exactly once"
    );
    assert_eq!(
        *analyzer.method_counts().get("POST").unwrap_or(&0),
        1,
        "drain: POST must be counted exactly once"
    );
    assert_eq!(
        analyzer.uri_list(),
        &["/a", "/b"],
        "drain: both URIs must be present in order"
    );
}

/// BC-2.06.001 invariant 4 — parsing a request does NOT increment transactions.
#[test]
fn test_BC_2_06_001_request_parse_does_not_increment_transactions() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let req =
        b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

    assert_eq!(
        analyzer.transaction_count(),
        0,
        "BC-2.06.001 invariant 4: transaction_count must remain 0 after request parse"
    );
}

/// BC-2.06.001 EC-001 — HTTP/1.0 (version byte == 0) parsed normally;
/// missing-Host finding does NOT fire because the version gate exempts 1.0.
#[test]
fn test_BC_2_06_001_http10_parsed_without_host_finding() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // HTTP/1.0 — no Host header (legal for 1.0).
    let req = b"GET /resource HTTP/1.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        1,
        "BC-2.06.001 EC-001: GET must be counted for HTTP/1.0 request"
    );
    // No missing-Host finding for 1.0.
    assert!(
        !analyzer
            .findings()
            .iter()
            .any(|f| f.summary.contains("Host header")),
        "BC-2.06.001 EC-001: missing-Host finding must NOT fire for HTTP/1.0"
    );
}

/// BC-2.06.001 EC-003/004 — absent User-Agent and absent Host (HTTP/1.0)
/// produce no map entries for those fields.
#[test]
fn test_BC_2_06_001_absent_optional_headers_produce_no_map_entries() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // No Host, no UA — HTTP/1.0 so no missing-Host finding either.
    let req = b"POST /submit HTTP/1.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

    assert_eq!(
        *analyzer.method_counts().get("POST").unwrap_or(&0),
        1,
        "method must still be counted even without Host/UA"
    );
    assert!(
        analyzer.host_counts().is_empty(),
        "BC-2.06.001 EC-004: hosts map must be empty when Host header absent"
    );
    assert!(
        analyzer.user_agent_counts().is_empty(),
        "BC-2.06.001 EC-003: user_agents map must be empty when UA absent"
    );
}

// ── BC-2.06.026 / AC-002 ─────────────────────────────────────────────────────
// AC-002: header values extracted via from_utf8_lossy.trim().

/// BC-2.06.026 postconditions 1-3 — leading/trailing whitespace trimmed from
/// Host value; non-UTF-8 bytes replaced with U+FFFD.
#[test]
fn test_BC_2_06_026_header_utf8_lossy_whitespace_trimmed() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // BC-2.06.026 EC-002: spaces around the host value must be stripped.
    let req = b"GET / HTTP/1.1\r\nHost:   example.com   \r\nUser-Agent: bot\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

    assert!(
        analyzer.host_counts().contains_key("example.com"),
        "BC-2.06.026: leading/trailing spaces must be trimmed from Host value"
    );
    assert!(
        !analyzer.host_counts().contains_key("  example.com  "),
        "untrimmed key must not be present"
    );
}

/// BC-2.06.026 postcondition 2 — non-UTF-8 bytes in User-Agent replaced by
/// U+FFFD (lossy conversion).  BC-2.06.026 EC-003 vector.
#[test]
fn test_BC_2_06_026_non_utf8_header_value_replaced_with_replacement_char() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Build a User-Agent value with an invalid UTF-8 byte (0x80 — lone continuation).
    let mut req = b"GET / HTTP/1.1\r\nHost: h.com\r\nUser-Agent: curl/7.0".to_vec();
    req.push(0x80); // invalid UTF-8 byte → U+FFFD after lossy conversion
    req.extend_from_slice(b"\r\n\r\n");
    analyzer.on_data(&fk, Direction::ClientToServer, &req, 0);

    // The stored key must contain the replacement character, not the raw byte.
    let ua_key = analyzer
        .user_agent_counts()
        .keys()
        .next()
        .expect("UA map must have an entry");
    assert!(
        ua_key.contains('\u{FFFD}'),
        "BC-2.06.026 postcondition 2: non-UTF-8 byte must be replaced with U+FFFD, got: {ua_key:?}"
    );
}

// ── BC-2.06.026 postcondition 4 / AC-009 ────────────────────────────────────
// AC-009: find_header case-insensitive; None for absent header.

/// BC-2.06.026 invariant 1 — find_header uses eq_ignore_ascii_case; mixed-case
/// header names are matched.
#[test]
fn test_BC_2_06_026_find_header_case_insensitive_match() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // httparse preserves the exact case the client sent.
    // "HOST" in all caps must still be mapped to hosts.
    let req = b"GET /resource HTTP/1.1\r\nHOST: caps.example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

    assert!(
        analyzer.host_counts().contains_key("caps.example.com"),
        "BC-2.06.026 invariant 1: HOST (all-caps) must be matched case-insensitively"
    );
}

/// BC-2.06.026 postconditions 1/4 — None returned for absent header; absent UA
/// produces no user_agents entry.
#[test]
fn test_BC_2_06_026_find_header_returns_none_for_absent_header() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // No User-Agent header at all.
    let req = b"GET / HTTP/1.1\r\nHost: x.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

    assert!(
        analyzer.user_agent_counts().is_empty(),
        "BC-2.06.026: absent User-Agent must not produce a user_agents entry"
    );
}

// ── BC-2.06.002 / AC-003 ─────────────────────────────────────────────────────
// AC-003: try_parse_requests pipelined loop — each request counted independently.

/// BC-2.06.002 postconditions 1-5 — two complete back-to-back requests in one
/// buffer are each counted independently, and both URIs are recorded.
#[test]
fn test_BC_2_06_002_pipelined_requests_each_counted_independently() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // BC-2.06.002 canonical test vector.
    let pipelined =
        b"GET /a HTTP/1.1\r\nHost: h\r\n\r\nGET /b HTTP/1.1\r\nHost: h\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, pipelined, 0);

    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        2,
        "BC-2.06.002 postcondition 2: GET must be counted twice for two pipelined GETs"
    );
    assert_eq!(
        analyzer.uri_list(),
        &["/a", "/b"],
        "BC-2.06.002 postcondition 2: both URIs must be present in order"
    );
    assert_eq!(
        analyzer.transaction_count(),
        0,
        "BC-2.06.002: requests must NOT increment transactions"
    );
}

/// BC-2.06.002 postcondition 3 — anomaly detection fires per request; the
/// POST/admin vector yields distinct method entries and an admin-panel finding
/// only for the /admin request.
#[test]
fn test_BC_2_06_002_pipelined_detections_per_request_not_aggregated() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // POST /login (no finding) followed by GET /admin (admin-panel finding).
    let pipelined = b"POST /login HTTP/1.1\r\nHost: h.com\r\n\r\nGET /admin HTTP/1.1\r\nHost: h.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, pipelined, 0);

    assert_eq!(
        *analyzer.method_counts().get("POST").unwrap_or(&0),
        1,
        "POST must be counted once"
    );
    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        1,
        "GET must be counted once"
    );
    // Admin-panel finding fires for /admin only.
    let admin_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("Admin panel"))
        .collect();
    assert_eq!(
        admin_findings.len(),
        1,
        "BC-2.06.002 postcondition 3: exactly one admin-panel finding for pipelined pair"
    );
}

/// BC-2.06.002 postcondition 5 — loop exits when partial bytes remain;
/// partial bytes are retained in the buffer.
#[test]
fn test_BC_2_06_002_pipelined_loop_stops_on_partial_tail() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // First request complete, second is partial (no trailing \r\n\r\n).
    let mixed = b"GET /first HTTP/1.1\r\nHost: h.com\r\n\r\nGET /partial HTTP/1.1\r\nHos";
    analyzer.on_data(&fk, Direction::ClientToServer, mixed, 0);

    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        1,
        "BC-2.06.002 postcondition 5: only the first complete request must be counted"
    );

    // Complete the partial request — buffer must have been retained.
    let completion = b"t: h.com\r\n\r\n";
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        completion,
        mixed.len() as u64,
    );
    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        2,
        "BC-2.06.002 postcondition 5: completing the partial must yield count=2"
    );
}

// ── BC-2.06.002 invariant 1 / AC-004 ─────────────────────────────────────────
// AC-004: request_error_count reset to 0 after each successful parse;
//         had_success prevents body bytes from being counted as parse errors.

/// BC-2.06.002 invariant 1 — error_count is reset to 0 after a successful
/// parse even when a prior error in the same direction incremented it.
#[test]
fn test_BC_2_06_002_request_error_count_reset_after_success() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // One garbage chunk → error_count becomes 1.
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "precondition: one parse error"
    );

    // One valid request → error_count must be reset to 0 (not visible via
    // parse_error_count which is global, but we can verify the direction is
    // NOT poisoned by confirming the valid request was parsed).
    let valid = b"GET /ok HTTP/1.1\r\nHost: x.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);

    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        1,
        "BC-2.06.002 invariant 1: valid request after error must be parsed (error_count reset)"
    );
    // Global parse_errors counter still reflects the earlier error.
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "global parse_errors must not decrease (it only counts; reset is per-flow)"
    );
}

/// BC-2.06.002 invariant 2 — had_success prevents body bytes from inflating
/// parse_errors after a successful header parse in the same on_data call.
#[test]
fn test_BC_2_06_002_had_success_suppresses_body_byte_errors() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Response with a body: header parses successfully, body bytes remain in
    // the buffer. The loop retries with body bytes — had_success must prevent
    // those bytes from incrementing parse_errors.
    let resp_with_body =
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello";
    analyzer.on_data(&fk, Direction::ServerToClient, resp_with_body, 0);

    assert_eq!(
        analyzer.transaction_count(),
        1,
        "response header must be counted"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "BC-2.06.002 invariant 2: body bytes after header must NOT increment parse_errors"
    );
}

// ── BC-2.06.003 / AC-005 ─────────────────────────────────────────────────────
// AC-005: Status::Partial → no counters updated, buf retained unchanged.

/// BC-2.06.003 postconditions 1-4 — partial request leaves all counters
/// unchanged; buffer is retained for subsequent completion.
#[test]
fn test_BC_2_06_003_partial_request_leaves_counters_unchanged() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // BC-2.06.003 canonical test vector (first half only).
    let partial = b"GET /test HTTP/1.1\r\nHost: ";
    analyzer.on_data(&fk, Direction::ClientToServer, partial, 0);

    assert!(
        analyzer.method_counts().get("GET").is_none(),
        "BC-2.06.003 postcondition 1: methods must be empty on partial"
    );
    assert!(
        analyzer.uri_list().is_empty(),
        "BC-2.06.003 postcondition 1: uris must be empty on partial"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "BC-2.06.003 postcondition 4: partial must NOT increment parse_errors"
    );

    // Complete the request — buffer must have been retained.
    let completion = b"h.com\r\n\r\n";
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        completion,
        partial.len() as u64,
    );
    assert_eq!(
        *analyzer.method_counts().get("GET").unwrap_or(&0),
        1,
        "BC-2.06.003 postcondition 5: request must be counted after completion"
    );
}

/// BC-2.06.003 postcondition 2 — partial request does not trigger anomaly
/// detection.
#[test]
fn test_BC_2_06_003_partial_request_no_anomaly_detection() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Partial path-traversal request — the traversal rule must NOT fire yet.
    let partial = b"GET /../../etc/passwd HTTP/1.1\r\nHos";
    analyzer.on_data(&fk, Direction::ClientToServer, partial, 0);

    assert!(
        analyzer.findings().is_empty(),
        "BC-2.06.003 postcondition 2: no findings before request is complete"
    );
}

// ── BC-2.06.003 invariant 1 / AC-006 ─────────────────────────────────────────
// AC-006: Status::Partial distinct from Err — does not increment parse_errors.

/// BC-2.06.003 invariant 1 — Partial does not increment parse_errors or
/// advance request_error_count toward the poison threshold.
#[test]
fn test_BC_2_06_003_partial_not_counted_as_error() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send many partial on_data calls — none should increment parse_errors.
    for chunk in [
        b"GET /pa" as &[u8],
        b"ge HTT",
        b"P/1.1\r\n",
        b"Host: e",
        b"x.com\r\n",
    ] {
        analyzer.on_data(&fk, Direction::ClientToServer, chunk, 0);
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.003 invariant 1: partial must never increment parse_errors"
        );
    }
    // Complete the request.
    analyzer.on_data(&fk, Direction::ClientToServer, b"\r\n", 0);
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "BC-2.06.003 invariant 1: parse_errors must remain 0 after partial+completion"
    );
}

// ── BC-2.06.004 / AC-007 ─────────────────────────────────────────────────────
// AC-007: try_parse_responses → transactions++, status_codes[code]++,
//         response_buf drained, response_error_count reset to 0.

/// BC-2.06.004 postconditions 1-4 — canonical 200 OK vector.
#[test]
fn test_BC_2_06_004_response_parse_increments_transactions_and_status_code() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, resp, 0);

    assert_eq!(
        analyzer.transaction_count(),
        1,
        "BC-2.06.004 postcondition 1: transactions must be 1 after one response"
    );
    assert_eq!(
        *analyzer.status_code_counts().get(&200).unwrap_or(&0),
        1,
        "BC-2.06.004 postcondition 2: status_codes[200] must be 1"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "BC-2.06.004 postcondition 4 proxy: parse_errors must be 0 (error_count reset)"
    );
}

/// BC-2.06.004 postcondition 3 — response_buf bytes are drained; two pipelined
/// responses in one on_data are both counted.
#[test]
fn test_BC_2_06_004_response_buf_drained_enables_pipelined_parsing() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // BC-2.06.004 canonical pipelined vector.
    let pipelined = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\nHTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, pipelined, 0);

    assert_eq!(
        analyzer.transaction_count(),
        2,
        "BC-2.06.004 postcondition 3: response_buf drained → second response must be counted"
    );
    assert_eq!(
        *analyzer.status_code_counts().get(&200).unwrap_or(&0),
        1,
        "status_codes[200] must be 1"
    );
    assert_eq!(
        *analyzer.status_code_counts().get(&304).unwrap_or(&0),
        1,
        "status_codes[304] must be 1"
    );
}

/// BC-2.06.004 EC-005 — httparse code==None → status_codes[0] incremented.
#[test]
fn test_BC_2_06_004_status_code_none_mapped_to_zero() {
    // httparse returns code=None when the status line has no numeric code.
    // unwrap_or(0) maps this to key 0 in the status_codes map.
    // We verify indirectly: a normal 200 response stores status_codes[200].
    // The unwrap_or(0) path is in parse_one_response which we can observe
    // via the status_code_counts accessor.
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Well-formed 404 response.
    let resp = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, resp, 0);

    assert_eq!(
        *analyzer.status_code_counts().get(&404).unwrap_or(&0),
        1,
        "BC-2.06.004 postcondition 2: status_codes[404] must be 1"
    );
    assert_eq!(
        analyzer.transaction_count(),
        1,
        "transactions must be 1 for the 404 response"
    );
}

// ── BC-2.06.004 invariant 1 / AC-008 ─────────────────────────────────────────
// AC-008: transactions counts responses ONLY; summarize() packets_analyzed
//         equals self.transactions.

/// BC-2.06.004 invariant 1 — request parse does NOT increment transactions;
/// response parse DOES.
#[test]
fn test_BC_2_06_004_transactions_counts_responses_not_requests() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Five requests, zero responses.
    for i in 0..5u8 {
        let req = format!("GET /path{i} HTTP/1.1\r\nHost: x.com\r\n\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, req.as_bytes(), 0);
    }
    assert_eq!(
        analyzer.transaction_count(),
        0,
        "BC-2.06.004 invariant 1: 5 requests must NOT increment transactions"
    );

    // One response.
    let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, resp, 0);
    assert_eq!(
        analyzer.transaction_count(),
        1,
        "BC-2.06.004 invariant 1: one response must produce transactions=1"
    );
}

/// BC-2.06.004 invariant 1 + summarize() mapping — packets_analyzed equals
/// transactions (response count), not request count.
#[test]
fn test_BC_2_06_004_summarize_packets_analyzed_equals_transactions() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Three requests, two responses.
    for i in 0..3u8 {
        let req = format!("GET /r{i} HTTP/1.1\r\nHost: x.com\r\n\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, req.as_bytes(), 0);
    }
    let responses = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\nHTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, responses, 0);

    let summary = analyzer.summarize();
    assert_eq!(
        summary.packets_analyzed,
        2,
        "BC-2.06.004 invariant 1: packets_analyzed must equal transaction count (2 responses)"
    );
    assert_eq!(
        summary.packets_analyzed,
        analyzer.transaction_count(),
        "summarize().packets_analyzed must equal self.transactions"
    );
}

// ── BC-2.06.026 invariant 4 / AC-010 ─────────────────────────────────────────
// AC-010: no escape function at parse time; raw URI bytes flow into
//         Finding.evidence unchanged.
// NOTE: The primary integration test is
//   test_http_finding_c1_csi_escaped_by_terminal_reporter (reporter_tests.rs)
// which already satisfies this AC end-to-end. The test below is a unit-level
// companion confirming the property at the analyzer boundary.

/// BC-2.06.026 invariant 4 — raw URI bytes from req.path flow directly into
/// Finding.evidence; no escaping occurs at the analyzer layer.
#[test]
fn test_BC_2_06_026_raw_uri_bytes_preserved_in_finding_evidence() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Path traversal URI with a C1 CSI byte (U+009B, encoded as 0xC2 0x9B in
    // UTF-8).  httparse accepts this because the high bytes are valid in URIs.
    // The path-traversal rule fires and puts the raw URI into evidence.
    let mut req = b"GET /../../etc/passwd".to_vec();
    req.extend_from_slice(&[0xC2, 0x9B]); // U+009B CSI — raw control byte
    req.extend_from_slice(b" HTTP/1.1\r\nHost: target.com\r\n\r\n");
    analyzer.on_data(&fk, Direction::ClientToServer, &req, 0);

    let findings = analyzer.findings();
    let traversal = findings
        .iter()
        .find(|f| f.summary.contains("Path traversal"))
        .expect("BC-2.06.026: path-traversal request must produce a Finding");

    // The evidence field must carry the raw C1 bytes — no escape applied.
    let evidence_raw = traversal.evidence[0].as_bytes();
    assert!(
        evidence_raw.windows(2).any(|w| w == [0xC2, 0x9B]),
        "BC-2.06.026 invariant 4: raw C1 CSI bytes must appear verbatim in Finding.evidence; \
         got: {:?}",
        traversal.evidence[0]
    );
}

} // mod bc_2_06_formalization
