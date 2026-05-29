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
    // AC-005 / BC-2.06.014 invariant 4: evidence must be EXACTLY "Direction: request",
    // not derived from the Direction enum (which would print a variant name, not this string).
    assert_eq!(
        findings[0].evidence[0], "Direction: request",
        "AC-005 / BC-2.06.014 invariant 4: evidence must be exactly 'Direction: request'"
    );
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
    assert_eq!(
        long_uri_finding.summary, "Abnormally long URI (2101 chars)",
        "summary must be exactly 'Abnormally long URI (2101 chars)', got: {}",
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
// BC-prefixed test names use mixed case by convention (DF-AC-TEST-NAME-SYNC-001).
// Each test function carries its own #[allow(non_snake_case)] to keep the lint
// suppression narrow.  Non-test helpers inside this module must use snake_case;
// clippy enforces that in CI because the module-wide allow has been removed.
// ---------------------------------------------------------------------------
mod bc_2_06_formalization {
    use super::*;

    // ── BC-2.06.001 ──────────────────────────────────────────────────────────────
    // AC-001: complete request → methods/hosts/user_agents/uris updated, buf
    //         drained, request_error_count reset, check_request_detections called.

    /// BC-2.06.001 postconditions 1-4 + 7 — canonical test vector (happy path).
    /// Exercises: method map, host map, UA map, URI vec all updated on one
    /// complete HTTP/1.1 request (the BC's golden vector).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_001_complete_request_updates_all_counters() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Canonical test vector from BC-2.06.001.
        let req = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.0\r\n\r\n";
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
    ///
    /// Postcondition 5 (buf drained) is exercised INDIRECTLY by the back-to-back
    /// parse pattern — `request_buf.len()` is not publicly observable, so drain
    /// success is inferred via the absence of re-parsing artifacts.
    ///
    /// Mental-deletion verification: removing `drain()` at src/analyzer/http.rs:398
    /// would cause `method_counts` to show an extra GET (the first request
    /// re-parsed as part of the second call) and the POST would fail to parse
    /// (orphan bytes in the buffer), so `method_counts["POST"]` would be 0.
    /// Either deviation from {GET=1, POST=1} would fail the assertions below.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_001_consumed_bytes_drained_from_buf() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Two back-to-back requests fed in a single on_data call.
        // Postcondition 5 (buf drained) is proven INDIRECTLY: correct method counts
        // below are only achievable when each complete request is drained from the
        // buffer before the next parse iteration begins.
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
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_001_request_parse_does_not_increment_transactions() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        let req = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.0\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

        assert_eq!(
            analyzer.transaction_count(),
            0,
            "BC-2.06.001 invariant 4: transaction_count must remain 0 after request parse"
        );
    }

    /// BC-2.06.001 EC-001 — HTTP/1.0 (version byte == 0) parsed normally;
    /// missing-Host finding does NOT fire because the version gate exempts 1.0.
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    ///
    /// BC-2.06.026 invariant 3: `.trim()` is called after `from_utf8_lossy`, which
    /// removes all ASCII whitespace (spaces, tabs) from both ends of the header value.
    ///
    /// LF coverage note (F-W15P2-002): AC-002 narrative mentions LF but httparse
    /// rejects bare `\n` in header values with `Err(HeaderName)` — confirmed by
    /// probe test (bare LF: `Err(HeaderName)`, embedded LF: `Err(HeaderName)`).
    /// LF trimming is therefore not reachable via `on_data`; AC-002 coverage is
    /// narrowed to space + tab, which is what these assertions exercise.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_026_header_utf8_lossy_whitespace_trimmed() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();
        let fk2 = test_flow_key_b();

        // BC-2.06.026 EC-002 — space variant: spaces around the host value must be stripped.
        let req_space = b"GET / HTTP/1.1\r\nHost:   example.com   \r\nUser-Agent: bot\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, req_space, 0);

        assert!(
            analyzer.host_counts().contains_key("example.com"),
            "BC-2.06.026: leading/trailing spaces must be trimmed from Host value"
        );
        assert!(
            !analyzer.host_counts().contains_key("  example.com  "),
            "untrimmed space-padded key must not be present"
        );

        // BC-2.06.026 invariant 3 — tab variant: tabs are ASCII whitespace; `.trim()`
        // must remove them just as it removes spaces.
        // httparse accepts tab characters in header field values (they are valid obs-ws
        // per RFC 9110 §5.6.3). The stored key must be "tab.example.com" not
        // "\ttab.example.com\t".
        let req_tab = b"GET / HTTP/1.1\r\nHost:\ttab.example.com\t\r\nUser-Agent: bot\r\n\r\n";
        analyzer.on_data(&fk2, Direction::ClientToServer, req_tab, 0);

        assert!(
            analyzer.host_counts().contains_key("tab.example.com"),
            "BC-2.06.026 invariant 3: leading/trailing tabs must be trimmed from Host value"
        );
        assert!(
            !analyzer.host_counts().contains_key("\ttab.example.com\t"),
            "untrimmed tab-padded key must not be present"
        );
    }

    /// BC-2.06.026 postcondition 2 — non-UTF-8 bytes in User-Agent replaced by
    /// U+FFFD (lossy conversion).  BC-2.06.026 EC-003 vector.
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_002_pipelined_requests_each_counted_independently() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // BC-2.06.002 canonical test vector.
        let pipelined = b"GET /a HTTP/1.1\r\nHost: h\r\n\r\nGET /b HTTP/1.1\r\nHost: h\r\n\r\n";
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

    /// BC-2.06.002 postcondition 3 — anomaly detection fires PER REQUEST, not
    /// aggregated.  Two pipelined requests both matching the admin-panel pattern
    /// must produce TWO distinct findings.  If detection were aggregated (a
    /// hypothetical bug), only one finding would be emitted regardless of how
    /// many requests matched.
    ///
    /// Admin-panel src patterns (src/analyzer/http.rs:236):
    ///   ["/wp-admin", "/admin", "/phpmyadmin", "/manager"]
    ///
    /// Chosen test URIs:
    ///   1. `/admin/dashboard`   — matches via "/admin" substring
    ///   2. `/wp-admin/index.php` — matches via "/wp-admin" substring
    ///
    /// Aggregation-distinguishing guarantee: len() == 2 can ONLY pass if both
    /// requests individually triggered the detection loop.  Under aggregated
    /// emission (hypothetical) len() would be 1 even with two matching requests.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_002_pipelined_detections_per_request_not_aggregated() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Two pipelined requests that both match the admin-panel detection pattern.
        // "/admin/dashboard" matches "/admin"; "/wp-admin/index.php" matches "/wp-admin".
        let pipelined = b"GET /admin/dashboard HTTP/1.1\r\nHost: h.com\r\n\r\n\
GET /wp-admin/index.php HTTP/1.1\r\nHost: h.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, pipelined, 0);

        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            2,
            "both GET requests must be counted"
        );
        // Per-request emission: two distinct admin-panel findings, one per request.
        // Under aggregated emission (hypothetical bug) this would be 1, not 2.
        let admin_findings: Vec<_> = analyzer
            .findings()
            .into_iter()
            .filter(|f| f.summary.contains("Admin panel"))
            .collect();
        assert_eq!(
            admin_findings.len(),
            2,
            "BC-2.06.002 postcondition 3: two pipelined admin requests must produce \
             two separate Admin panel findings (per-request emission proven); \
             aggregated emission would yield len=1"
        );
        // Confirm the two findings reference distinct URIs.
        let uris: Vec<_> = admin_findings.iter().map(|f| f.summary.as_str()).collect();
        assert!(
            uris.iter().any(|s| s.contains("/admin/dashboard")),
            "first finding must reference /admin/dashboard"
        );
        assert!(
            uris.iter().any(|s| s.contains("/wp-admin/index.php")),
            "second finding must reference /wp-admin/index.php"
        );
    }

    /// BC-2.06.002 postcondition 5 — loop exits when partial bytes remain;
    /// partial bytes are retained in the buffer.
    #[allow(non_snake_case)]
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
    ///
    /// The reset is proven via POISON_THRESHOLD = 3: without a reset, two rounds
    /// of 2 garbage chunks each (4 total) would exceed the threshold and poison
    /// the direction, causing subsequent valid GETs to be skipped. With the reset,
    /// each pair of errors is cleared by the intervening GET, preventing poisoning.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_002_request_error_count_reset_after_success() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // POISON_THRESHOLD = 3. Two errors alone are not enough to poison, but
        // without a reset they accumulate — two rounds of 2 would give count=4
        // which exceeds the threshold of 3.

        // Round 1: 2 garbage chunks → error_count = 2 (below threshold).
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE1\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE2\r\n\r\n", 0);
        assert_eq!(
            analyzer.parse_error_count(),
            2,
            "precondition: two parse errors accumulated"
        );

        // Valid GET #1 — must reset error_count to 0.
        let valid1 = b"GET /first HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, valid1, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.002 invariant 1: first GET after 2 errors must be parsed (not poisoned)"
        );

        // Round 2: 2 more garbage chunks. If error_count was NOT reset to 0
        // after valid GET #1, the running total would now be 4 (≥ 3 = POISON_THRESHOLD)
        // and the direction would be poisoned. With the reset, it returns to 2 here.
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE3\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE4\r\n\r\n", 0);

        // Valid GET #2 — must also parse successfully because the reset kept
        // the per-flow error_count at 2, not at 4.
        let valid2 = b"GET /second HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, valid2, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            2,
            "BC-2.06.002 invariant 1: second GET after second error pair must be parsed; \
             without the reset, cumulative error_count (4) would exceed POISON_THRESHOLD (3) \
             and this GET would be skipped"
        );

        // Global parse_errors counter reflects all four garbage errors (never decrements).
        assert_eq!(
            analyzer.parse_error_count(),
            4,
            "global parse_errors must equal the total number of error events (4)"
        );

        // Now verify the threshold still fires correctly: send 3 more garbage chunks
        // after the last reset (error_count restarts at 0 after valid2, so 3 errors
        // is exactly POISON_THRESHOLD) → direction must be poisoned.
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK1\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK2\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK3\r\n\r\n", 0);

        // A third GET must be skipped — direction is poisoned.
        let valid3 = b"GET /third HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, valid3, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            2,
            "BC-2.06.002 invariant 1: after POISON_THRESHOLD errors the direction is poisoned; \
             third GET must be skipped (poisoned_bytes_skipped increases instead)"
        );
        assert!(
            analyzer.poisoned_bytes_skipped() > 0,
            "poisoned direction must cause poisoned_bytes_skipped to be non-zero"
        );
    }

    /// BC-2.06.002 invariant 2 (REQUEST side) — had_success prevents body bytes from
    /// inflating parse_errors after a successful request header parse in the same
    /// on_data call.
    ///
    /// The REQUEST-side guard is at src/analyzer/http.rs:404 (try_parse_requests).
    ///
    /// Loop iteration 1: parse the complete GET request header → Complete(n),
    ///   had_success = true, header bytes drained, request_buf now contains the
    ///   NUL-prefixed body bytes ("\x00\x01...").
    /// Loop iteration 2: parse "\x00\x01..." → Err(Token) because 0x00 is not a
    ///   legal HTTP method character.  Because had_success == true, parse_errors
    ///   must NOT be incremented.
    ///
    /// Mental-deletion verification: if the `if !had_success` guard at
    /// src/analyzer/http.rs:404 were removed, parse_error_count() would return 1
    /// and this assertion would fail — proving the guard is load-bearing.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_002_had_success_suppresses_request_body_byte_errors() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Build a complete HTTP/1.1 request header followed immediately by body
        // bytes that begin with NUL (0x00) and STX (0x01).  The header parses
        // successfully; the body remainder causes Err(Token) on the next loop
        // iteration because 0x00 is not a legal token start in an HTTP method.
        let mut req_with_body = b"GET /resource HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec();
        req_with_body.push(0x00); // NUL — Err(Token) on next iteration
        req_with_body.push(0x01); // additional non-HTTP byte
        analyzer.on_data(&fk, Direction::ClientToServer, &req_with_body, 0);

        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.002 request-side: the GET header must be counted before the body error"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.002 invariant 2 (request side): had_success guard at \
             src/analyzer/http.rs:404 must prevent parse_errors increment \
             when NUL body bytes cause Err after a successful request header parse"
        );
    }

    /// BC-2.06.004 invariant 4 — Response-side had_success guard prevents body bytes that
    /// follow a successfully parsed response header from inflating parse_errors. This is the
    /// response-side analog of BC-2.06.002 invariant 2 (request-side). Guard at
    /// src/analyzer/http.rs:462 (try_parse_responses).
    ///
    /// Loop iteration 1: parse "HTTP/1.1 200 OK\r\n...\r\n\r\n" →
    ///   Complete(n), had_success = true, header bytes drained, response buf
    ///   now contains "\x00body" only.
    /// Loop iteration 2: parse "\x00body" → Err(InvalidToken). Because
    ///   had_success == true, parse_errors must NOT be incremented.
    ///
    /// Mental-deletion verification: if the `if !had_success` guard at
    /// src/analyzer/http.rs:462 were deleted (or changed to `if true`),
    /// parse_error_count() would return 1 and this assertion would fail —
    /// proving the response-side guard is independently load-bearing.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_004_had_success_suppresses_response_body_byte_errors() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        let mut resp_with_body =
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\n".to_vec();
        resp_with_body.push(0x00); // NUL — causes Err(InvalidToken) in next iteration
        resp_with_body.extend_from_slice(b"body");
        analyzer.on_data(&fk, Direction::ServerToClient, &resp_with_body, 0);

        assert_eq!(
            analyzer.transaction_count(),
            1,
            "response header must be counted as one transaction"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.004 invariant 4: response-side had_success guard must prevent body bytes \
             (NUL-injected) from inflating parse_errors after successful header parse"
        );
    }

    // ── BC-2.06.003 / AC-005 ─────────────────────────────────────────────────────
    // AC-005: Status::Partial → no counters updated, buf retained unchanged.

    /// BC-2.06.003 postconditions 1-4 — partial request leaves all counters
    /// unchanged; buffer is retained for subsequent completion.
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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

    /// BC-2.06.004 postcondition 2 — well-formed response with numeric status code 404
    /// is stored at status_codes[404].
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_004_well_formed_404_response_status_code_counted() {
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
        // NOTE: BC-2.06.004 EC-005 (code==None → status_codes[0] via unwrap_or(0)) is NOT
        // exercised here. Empirically, httparse rejects status lines without a numeric code
        // via `Err(InvalidStatus)` rather than `Status::Complete { code: None, .. }` — so
        // EC-005 may be unreachable via the public `on_data` API. Deferred to research-agent
        // investigation per DF-VALIDATION-001 (filed as W15.D1 in STATE.md drift items).
    }

    // ── BC-2.06.004 invariant 1 / AC-008 ─────────────────────────────────────────
    // AC-008: transactions counts responses ONLY; summarize() packets_analyzed
    //         equals self.transactions.

    /// BC-2.06.004 invariant 1 — request parse does NOT increment transactions;
    /// response parse DOES.
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
            summary.packets_analyzed, 2,
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
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_026_raw_uri_bytes_preserved_in_finding_evidence() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Path traversal URI with a C1 CSI sequence: the bytes [0xC2, 0x9B] are
        // the valid UTF-8 encoding of U+009B (C1 CSI control character). httparse
        // accepts them into req.path: &str unchanged because they are well-formed
        // UTF-8. We verify they survive intact through find_header → uri →
        // Finding.evidence with NO escape, HTML-encode, or character-replacement
        // transformation. If the analyzer ever applied HTML-escape (e.g.,
        // U+009B → &#x9B;), the evidence bytes would differ from [0xC2, 0x9B]
        // and this test would fail.
        let mut req = b"GET /../../etc/passwd".to_vec();
        req.extend_from_slice(&[0xC2, 0x9B]); // valid UTF-8 for U+009B (C1 CSI)
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
            "BC-2.06.026 invariant 4: raw C1 CSI bytes (U+009B UTF-8 encoding) must appear \
             verbatim in Finding.evidence; got: {:?}",
            traversal.evidence[0]
        );
        // Anti-assertion: the HTML-escaped form must NOT appear.
        assert_ne!(
            traversal.evidence[0].as_bytes(),
            b"&#x9B;",
            "no HTML escape: analyzer must not apply HTML-encoding to URI bytes"
        );
    }
} // mod bc_2_06_formalization

// ---------------------------------------------------------------------------
// STORY-042 Brownfield-Formalization Tests
// (BC-2.06.005, BC-2.06.006, BC-2.06.007, BC-2.06.012)
//
// Naming convention: test_BC_2_06_NNN_<descriptive_suffix>
// Anchored to AC-001 through AC-010 and edge cases EC-001 through EC-010.
//
// These tests confirm existing brownfield behavior (formalization mode):
// they will PASS if the source already conforms to the BCs, FAIL if the
// source diverges from a BC postcondition, invariant, or edge case.
//
// The uppercase "BC" in function names follows DF-AC-TEST-NAME-SYNC-001.
// Each function carries its own #[allow(non_snake_case)].
// ---------------------------------------------------------------------------
mod bc_2_06_story042_formalization {
    use super::*;

    // ── BC-2.06.005 / AC-001 ─────────────────────────────────────────────────────
    // AC-001: path-traversal URI emits Reconnaissance/Likely/High, T1083,
    //         truncated summary, raw-URI evidence, direction=ClientToServer.

    /// BC-2.06.005 postcondition 1 (all fields) — canonical path-traversal vector.
    ///
    /// Exercises: category=Reconnaissance, verdict=Likely, confidence=High,
    /// mitre_technique=Some("T1083"), summary prefix "Path traversal in URI: ",
    /// evidence = ["URI: /../etc/passwd"], direction = Some(ClientToServer).
    ///
    /// EC-001: URI = "/../etc/passwd" → path-traversal finding emitted.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_005_path_traversal_all_fields() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Canonical BC-2.06.005 vector: URI contains "../"
        let request = b"GET /../etc/passwd HTTP/1.1\r\nHost: target.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        let findings = analyzer.findings();
        let traversal = findings
            .iter()
            .find(|f| f.summary.contains("Path traversal in URI"))
            .expect("BC-2.06.005 postcondition 1: path-traversal finding must be emitted");

        assert_eq!(
            traversal.category,
            ThreatCategory::Reconnaissance,
            "BC-2.06.005 pc-1: category must be Reconnaissance"
        );
        assert_eq!(
            traversal.verdict,
            Verdict::Likely,
            "BC-2.06.005 pc-1: verdict must be Likely"
        );
        assert_eq!(
            traversal.confidence,
            Confidence::High,
            "BC-2.06.005 pc-1: confidence must be High"
        );
        assert_eq!(
            traversal.mitre_technique.as_deref(),
            Some("T1083"),
            "BC-2.06.005 pc-1: mitre_technique must be T1083"
        );
        // Summary must contain the URI (truncated to 120 chars max).
        assert!(
            traversal.summary.starts_with("Path traversal in URI: "),
            "BC-2.06.005 pc-1: summary must start with 'Path traversal in URI: ', got: {}",
            traversal.summary
        );
        assert!(
            traversal.summary.contains("/../etc/passwd"),
            "BC-2.06.005 pc-1: summary must contain the URI, got: {}",
            traversal.summary
        );
        // Evidence must contain the full raw URI (no truncation in evidence field).
        assert_eq!(
            traversal.evidence.len(),
            1,
            "BC-2.06.005 pc-1: evidence must have exactly one entry"
        );
        assert!(
            traversal.evidence[0].starts_with("URI: "),
            "BC-2.06.005 pc-1: evidence[0] must start with 'URI: ', got: {}",
            traversal.evidence[0]
        );
        assert!(
            traversal.evidence[0].contains("/../etc/passwd"),
            "BC-2.06.005 pc-1: evidence must contain full raw URI, got: {}",
            traversal.evidence[0]
        );
        // Direction must be ClientToServer (path traversal is client-originated).
        assert_eq!(
            traversal.direction,
            Some(Direction::ClientToServer),
            "BC-2.06.005 pc-1: direction must be ClientToServer"
        );

        // BC-2.06.005 postcondition 1: summary uses truncate_uri(uri, 120) while
        // evidence retains the full raw URI (AC-001 truncation sub-test).
        // Use a path-traversal URI longer than 120 chars so truncation is non-trivial.
        {
            let mut a2 = HttpAnalyzer::new();
            let fk2 = test_flow_key();
            // 130-char URI: "/../" prefix (triggers detection) + 126 'a' chars.
            let long_path = format!("/../{}", "a".repeat(126));
            assert!(
                long_path.len() > 120,
                "test setup: long_path must exceed 120 chars (got {})",
                long_path.len()
            );
            let req = format!("GET {long_path} HTTP/1.1\r\nHost: h\r\n\r\n");
            a2.on_data(&fk2, Direction::ClientToServer, req.as_bytes(), 0);

            let findings2 = a2.findings();
            let t2 = findings2
                .iter()
                .find(|f| f.summary.contains("Path traversal in URI"))
                .expect(
                    "BC-2.06.005 pc-1 (truncation): path-traversal finding must be emitted for \
                     a long URI",
                );
            // Summary URI portion must be exactly 120 chars (the truncation limit).
            let summary_uri = t2
                .summary
                .strip_prefix("Path traversal in URI: ")
                .expect("summary must have expected prefix");
            assert_eq!(
                summary_uri.len(),
                120,
                "BC-2.06.005 pc-1 (truncation): summary URI portion must be exactly 120 chars \
                 when the URI exceeds 120 chars; got {} chars: {:?}",
                summary_uri.len(),
                summary_uri
            );
            assert!(
                long_path.starts_with(summary_uri),
                "BC-2.06.005 pc-1 (truncation): summary URI must be the first 120 chars of the \
                 full URI"
            );
            // Evidence must contain the FULL URI (no truncation).
            assert!(
                t2.evidence[0].contains(&long_path),
                "BC-2.06.005 pc-1 (truncation): evidence must contain the full raw URI (no \
                 truncation); evidence: {:?}",
                t2.evidence[0]
            );
        }
    }

    // ── BC-2.06.005 / AC-002 ─────────────────────────────────────────────────────
    // AC-002: exactly four traversal patterns (../  ..%2f  ..%252f  ....//);
    //         no backslash variant; URI is lowercased before match.

    /// BC-2.06.005 invariant 1 — all four canonical patterns trigger the finding;
    /// the backslash pattern ("..\\") does NOT; URI is lowercased before match.
    ///
    /// Exercises EC-001 (plain ../), EC-002 (URL-encoded ..%2f), EC-003 (double-encoded
    /// ..%252f), EC-004 (....//), and the invariant that there is no backslash variant.
    /// EC numbering matches STORY-042 Edge Cases table exactly.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_005_encoded_traversal_four_patterns() {
        // EC-001: "/../etc/passwd" plain "../" variant (lowercased input).
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /../etc/passwd HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            let findings = a.findings();
            let t = findings
                .iter()
                .find(|f| f.summary.contains("Path traversal"))
                .expect("EC-001 pattern '../': path-traversal finding must be emitted");
            assert_eq!(
                t.mitre_technique.as_deref(),
                Some("T1083"),
                "EC-001: mitre_technique must be T1083"
            );
        }

        // EC-002 (BC-2.06.005 invariant 1): URL-encoded "..%2f" variant.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /..%2fetc%2fpasswd HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings()
                    .iter()
                    .any(|f| f.summary.contains("Path traversal")),
                "BC-2.06.005 invariant 1: '..%2f' (URL-encoded slash) must trigger finding"
            );
        }

        // EC-003 (BC-2.06.005 invariant 1): double-encoded "..%252f" variant.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /..%252fetc HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings()
                    .iter()
                    .any(|f| f.summary.contains("Path traversal")),
                "BC-2.06.005 invariant 1: '..%252f' (double-encoded) must trigger finding"
            );
        }

        // EC-004 (BC-2.06.005 invariant 1): "....//etc/passwd" variant.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /....//etc/passwd HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings()
                    .iter()
                    .any(|f| f.summary.contains("Path traversal")),
                "BC-2.06.005 invariant 1: '....//' must trigger finding"
            );
        }

        // BC-2.06.005 invariant 1: case-insensitivity — uppercase pattern lowercased before match.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            // "..%2F" (capital F) must match because URI is lowercased before check.
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /..%2Fetc HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings()
                    .iter()
                    .any(|f| f.summary.contains("Path traversal")),
                "BC-2.06.005 invariant 1: '..%2F' (uppercase F) must match via lowercase"
            );
        }

        // BC-2.06.005 invariant 1: NO backslash variant — "..\" must NOT trigger
        // a path-traversal finding.
        // Guardedness: we also assert that the request actually parsed (GET count==1
        // and parse_error_count==0) so the negative assertion is meaningful — if
        // httparse rejected the backslash URI outright, the detection block would
        // never execute and the negative would be vacuously true.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /..\\etc\\passwd HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            // The request must have been parsed (not rejected) for the negative to hold.
            assert_eq!(
                *a.method_counts().get("GET").unwrap_or(&0),
                1,
                "BC-2.06.005 invariant 1 (guard): backslash URI must parse successfully \
                 so the detection-block is reached and the negative assertion is non-vacuous"
            );
            assert_eq!(
                a.parse_error_count(),
                0,
                "BC-2.06.005 invariant 1 (guard): parse_error_count must be 0 — \
                 the request was well-formed enough for httparse to accept it"
            );
            let has_traversal = a
                .findings()
                .iter()
                .any(|f| f.summary.contains("Path traversal"));
            assert!(
                !has_traversal,
                "BC-2.06.005 invariant 1: backslash variant '..\\' must NOT trigger \
                 path-traversal finding (no backslash pattern in source)"
            );
        }
    }

    // ── BC-2.06.005 / AC-003 ─────────────────────────────────────────────────────
    // AC-003: path-traversal detection fires per-request, not per-flow-once.

    /// BC-2.06.005 postcondition 2 — two pipelined requests each containing "../"
    /// each emit a separate path-traversal finding (per-request, not per-flow-once).
    #[allow(non_snake_case)]
    #[test]
    fn test_path_traversal_fires_per_request() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Two pipelined requests, both with path-traversal URIs.
        let pipelined = b"GET /../etc/passwd HTTP/1.1\r\nHost: target.com\r\n\r\n\
GET /../../boot.ini HTTP/1.1\r\nHost: target.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, pipelined, 0);

        let traversal_findings: Vec<_> = analyzer
            .findings()
            .into_iter()
            .filter(|f| f.summary.contains("Path traversal in URI"))
            .collect();

        assert_eq!(
            traversal_findings.len(),
            2,
            "BC-2.06.005 postcondition 2: two pipelined traversal requests must emit \
             two separate findings (per-request, not per-flow-once); \
             per-flow-once emission would yield len=1"
        );
        // Verify the two findings reference distinct URIs.
        assert!(
            traversal_findings
                .iter()
                .any(|f| f.summary.contains("/../etc/passwd")),
            "first traversal finding must reference /../etc/passwd"
        );
        assert!(
            traversal_findings
                .iter()
                .any(|f| f.summary.contains("/../../boot.ini")),
            "second traversal finding must reference /../../boot.ini"
        );
    }

    // ── BC-2.06.005 / EC-011 ─────────────────────────────────────────────────────
    // EC-011 (BC-2.06.005 EC-007): HTTP/1.0 request with path traversal →
    //         path-traversal finding still emitted; HTTP/1.0 is NOT exempt.

    /// BC-2.06.005 precondition 1 + EC-007 — path-traversal detection applies to
    /// HTTP/1.0 requests as well as HTTP/1.1.  The BC explicitly states that both
    /// "HTTP/1.1 or HTTP/1.0" requests are in scope.
    ///
    /// EC-011: GET /../etc/passwd HTTP/1.0 → T1083 finding emitted.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_005_http10_path_traversal_not_exempt() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // HTTP/1.0 request containing a "../" path-traversal URI.
        let request = b"GET /../etc/passwd HTTP/1.0\r\nHost: target.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        // The request must have been parsed (not silently dropped) — method count
        // confirms that the path through check_request_detections was executed.
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "EC-011 (guard): HTTP/1.0 request must be parsed and counted; if method_counts \
             is 0 the detection block was never reached"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "EC-011 (guard): parse_error_count must be 0 — HTTP/1.0 request must not be \
             rejected as malformed"
        );

        // The path-traversal finding must be emitted regardless of HTTP version.
        let findings = analyzer.findings();
        let traversal = findings
            .iter()
            .find(|f| f.summary.contains("Path traversal in URI"))
            .expect(
                "BC-2.06.005 EC-007 / EC-011: path-traversal finding (T1083) must be emitted \
                 for an HTTP/1.0 request; HTTP/1.0 is NOT exempt from detection",
            );

        assert_eq!(
            traversal.mitre_technique.as_deref(),
            Some("T1083"),
            "EC-011: mitre_technique must be T1083 for HTTP/1.0 path-traversal"
        );
        assert_eq!(
            traversal.category,
            ThreatCategory::Reconnaissance,
            "EC-011: category must be Reconnaissance for HTTP/1.0 path-traversal"
        );
        assert_eq!(
            traversal.verdict,
            Verdict::Likely,
            "EC-011: verdict must be Likely for HTTP/1.0 path-traversal"
        );
        assert_eq!(
            traversal.confidence,
            Confidence::High,
            "EC-011: confidence must be High for HTTP/1.0 path-traversal"
        );
        assert!(
            traversal.evidence[0].contains("/../etc/passwd"),
            "EC-011: evidence must contain the full raw URI, got: {}",
            traversal.evidence[0]
        );
        assert_eq!(
            traversal.direction,
            Some(Direction::ClientToServer),
            "EC-011: direction must be ClientToServer"
        );
    }

    // ── BC-2.06.006 / AC-004 ─────────────────────────────────────────────────────
    // AC-004: web-shell URI emits Execution/Likely/Medium, T1505.003,
    //         truncated summary, raw-URI evidence, direction=ClientToServer.

    /// BC-2.06.006 postcondition 1 (all fields) — canonical web-shell vector.
    ///
    /// Exercises: category=Execution, verdict=Likely, confidence=Medium,
    /// mitre_technique=Some("T1505.003"), summary prefix "Possible web shell access: ",
    /// evidence = ["URI: /uploads/c99.php"], direction = Some(ClientToServer).
    ///
    /// EC-005: URI = "/shell.php" → web-shell finding emitted (T1505.003).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_006_webshell_path_all_fields() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Canonical BC-2.06.006 vector: URI contains "/c99.php" (substring match).
        let request = b"GET /uploads/c99.php HTTP/1.1\r\nHost: target.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        let findings = analyzer.findings();
        let shell_finding = findings
            .iter()
            .find(|f| f.summary.contains("web shell"))
            .expect("BC-2.06.006 postcondition 1: web-shell finding must be emitted");

        assert_eq!(
            shell_finding.category,
            ThreatCategory::Execution,
            "BC-2.06.006 pc-1: category must be Execution"
        );
        assert_eq!(
            shell_finding.verdict,
            Verdict::Likely,
            "BC-2.06.006 pc-1: verdict must be Likely"
        );
        assert_eq!(
            shell_finding.confidence,
            Confidence::Medium,
            "BC-2.06.006 pc-1: confidence must be Medium"
        );
        assert_eq!(
            shell_finding.mitre_technique.as_deref(),
            Some("T1505.003"),
            "BC-2.06.006 pc-1: mitre_technique must be T1505.003"
        );
        assert!(
            shell_finding
                .summary
                .starts_with("Possible web shell access: "),
            "BC-2.06.006 pc-1: summary must start with 'Possible web shell access: ', got: {}",
            shell_finding.summary
        );
        assert!(
            shell_finding.summary.contains("/uploads/c99.php"),
            "BC-2.06.006 pc-1: summary must contain the URI, got: {}",
            shell_finding.summary
        );
        // Evidence: full raw URI, no truncation.
        assert_eq!(
            shell_finding.evidence.len(),
            1,
            "BC-2.06.006 pc-1: evidence must have exactly one entry"
        );
        assert!(
            shell_finding.evidence[0].starts_with("URI: "),
            "BC-2.06.006 pc-1: evidence[0] must start with 'URI: ', got: {}",
            shell_finding.evidence[0]
        );
        assert!(
            shell_finding.evidence[0].contains("/uploads/c99.php"),
            "BC-2.06.006 pc-1: evidence must contain full raw URI, got: {}",
            shell_finding.evidence[0]
        );
        assert_eq!(
            shell_finding.direction,
            Some(Direction::ClientToServer),
            "BC-2.06.006 pc-1: direction must be ClientToServer"
        );

        // BC-2.06.006 postcondition 1: summary uses truncate_uri(uri, 120) while
        // evidence retains the full raw URI (AC-004 truncation sub-test).
        // Use a web-shell URI longer than 120 chars so truncation is non-trivial.
        {
            let mut a2 = HttpAnalyzer::new();
            let fk2 = test_flow_key();
            // Long URI: "/shell.php" prefix (triggers detection) + 115 'b' chars = 125 chars total.
            let long_shell = format!("/shell.php{}", "b".repeat(115));
            assert!(
                long_shell.len() > 120,
                "test setup: long_shell must exceed 120 chars (got {})",
                long_shell.len()
            );
            let req = format!("GET {long_shell} HTTP/1.1\r\nHost: h\r\n\r\n");
            a2.on_data(&fk2, Direction::ClientToServer, req.as_bytes(), 0);

            let findings2 = a2.findings();
            let s2 = findings2
                .iter()
                .find(|f| f.summary.contains("web shell"))
                .expect(
                    "BC-2.06.006 pc-1 (truncation): web-shell finding must be emitted for a \
                     long URI",
                );
            let summary_uri = s2
                .summary
                .strip_prefix("Possible web shell access: ")
                .expect("summary must have expected prefix");
            assert_eq!(
                summary_uri.len(),
                120,
                "BC-2.06.006 pc-1 (truncation): summary URI portion must be exactly 120 chars \
                 when the URI exceeds 120 chars; got {} chars",
                summary_uri.len()
            );
            assert!(
                long_shell.starts_with(summary_uri),
                "BC-2.06.006 pc-1 (truncation): summary URI must be the first 120 chars of the \
                 full URI"
            );
            assert!(
                s2.evidence[0].contains(&long_shell),
                "BC-2.06.006 pc-1 (truncation): evidence must contain the full raw URI (no \
                 truncation); evidence: {:?}",
                s2.evidence[0]
            );
        }
    }

    // ── BC-2.06.006 / AC-005 ─────────────────────────────────────────────────────
    // AC-005: web-shell URI comparison is case-insensitive (lowercased) and
    //         substring-based.

    /// BC-2.06.006 invariant 1-2 — web-shell check is case-insensitive via lowercase
    /// and substring: "/uploads/SHELL.PHP" (uppercase) triggers the finding via
    /// the lowercased pattern "/shell.php".
    ///
    /// EC-006: URI = "/uploads/SHELL.PHP" → web-shell finding emitted.
    #[allow(non_snake_case)]
    #[test]
    fn test_webshell_case_insensitive() {
        // EC-006: uppercase URI must match via lowercase.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /uploads/SHELL.PHP HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings().iter().any(|f| f.summary.contains("web shell")),
                "BC-2.06.006 invariant 1: '/uploads/SHELL.PHP' must match via lowercase"
            );
        }

        // Substring match: "/uploads/c99.php?cmd=id" contains "/c99.php".
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /uploads/c99.php?cmd=id HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings().iter().any(|f| f.summary.contains("web shell")),
                "BC-2.06.006 invariant 2: '/uploads/c99.php?cmd=id' must match via substring"
            );
        }

        // All 10 web-shell patterns must individually trigger the finding.
        let shell_patterns = [
            "/shell.php",
            "/shell.asp",
            "/shell.jsp",
            "/cmd.php",
            "/cmd.asp",
            "/cmd.jsp",
            "/c99.php",
            "/r57.php",
            "/webshell",
            "/backdoor",
        ];
        for pattern in &shell_patterns {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            let request = format!("GET {pattern} HTTP/1.1\r\nHost: target.com\r\n\r\n");
            a.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);
            assert!(
                a.findings().iter().any(|f| f.summary.contains("web shell")),
                "BC-2.06.006: pattern '{pattern}' must trigger web-shell finding"
            );
        }
    }

    // ── BC-2.06.007 / AC-006 ─────────────────────────────────────────────────────
    // AC-006: admin-panel URI emits Reconnaissance/Inconclusive/Low, T1046,
    //         truncated summary, raw-URI evidence, direction=ClientToServer.

    /// BC-2.06.007 postcondition 1 (all fields) — all four admin-panel patterns.
    ///
    /// Exercises each of the 4 patterns with full field assertions:
    /// category=Reconnaissance, verdict=Inconclusive, confidence=Low, T1046.
    ///
    /// EC-008: URI = "/wp-admin/edit.php" → admin-panel finding emitted.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_007_admin_panel_all_fields() {
        let pattern_uris = [
            ("/wp-admin/edit.php", "/wp-admin"),
            ("/admin/dashboard", "/admin"),
            ("/phpmyadmin/", "/phpmyadmin"),
            ("/manager/html", "/manager"),
        ];

        for (uri, matched_pattern) in &pattern_uris {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();

            let request = format!("GET {uri} HTTP/1.1\r\nHost: target.com\r\n\r\n");
            analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

            let findings = analyzer.findings();
            let admin_finding = findings
                .iter()
                .find(|f| f.summary.contains("Admin panel"))
                .unwrap_or_else(|| {
                    panic!(
                        "BC-2.06.007 postcondition 1: admin-panel finding must be emitted \
                         for URI '{uri}' (pattern '{matched_pattern}')"
                    )
                });

            assert_eq!(
                admin_finding.category,
                ThreatCategory::Reconnaissance,
                "BC-2.06.007 pc-1: category must be Reconnaissance for '{uri}'"
            );
            assert_eq!(
                admin_finding.verdict,
                Verdict::Inconclusive,
                "BC-2.06.007 pc-1: verdict must be Inconclusive for '{uri}'"
            );
            assert_eq!(
                admin_finding.confidence,
                Confidence::Low,
                "BC-2.06.007 pc-1: confidence must be Low for '{uri}'"
            );
            assert_eq!(
                admin_finding.mitre_technique.as_deref(),
                Some("T1046"),
                "BC-2.06.007 pc-1: mitre_technique must be T1046 for '{uri}'"
            );
            assert!(
                admin_finding.summary.starts_with("Admin panel access: "),
                "BC-2.06.007 pc-1: summary must start with 'Admin panel access: ', got: {}",
                admin_finding.summary
            );
            assert!(
                admin_finding.summary.contains(*uri),
                "BC-2.06.007 pc-1: summary must contain the URI '{uri}', got: {}",
                admin_finding.summary
            );
            assert_eq!(
                admin_finding.evidence.len(),
                1,
                "BC-2.06.007 pc-1: evidence must have exactly one entry for '{uri}'"
            );
            assert!(
                admin_finding.evidence[0].starts_with("URI: "),
                "BC-2.06.007 pc-1: evidence[0] must start with 'URI: ', got: {}",
                admin_finding.evidence[0]
            );
            assert!(
                admin_finding.evidence[0].contains(*uri),
                "BC-2.06.007 pc-1: evidence must contain full raw URI '{uri}', got: {}",
                admin_finding.evidence[0]
            );
            assert_eq!(
                admin_finding.direction,
                Some(Direction::ClientToServer),
                "BC-2.06.007 pc-1: direction must be ClientToServer for '{uri}'"
            );
        }

        // BC-2.06.007 postcondition 1: summary uses truncate_uri(uri, 120) while
        // evidence retains the full raw URI (AC-006 truncation sub-test).
        // Use an admin-panel URI longer than 120 chars so truncation is non-trivial.
        {
            let mut a2 = HttpAnalyzer::new();
            let fk2 = test_flow_key();
            // Long URI: "/admin/" prefix (triggers detection) + 115 'c' chars = 122 chars total.
            let long_admin = format!("/admin/{}", "c".repeat(115));
            assert!(
                long_admin.len() > 120,
                "test setup: long_admin must exceed 120 chars (got {})",
                long_admin.len()
            );
            let req = format!("GET {long_admin} HTTP/1.1\r\nHost: h\r\n\r\n");
            a2.on_data(&fk2, Direction::ClientToServer, req.as_bytes(), 0);

            let findings2 = a2.findings();
            let a_f = findings2
                .iter()
                .find(|f| f.summary.contains("Admin panel"))
                .expect(
                    "BC-2.06.007 pc-1 (truncation): admin-panel finding must be emitted for a \
                     long URI",
                );
            let summary_uri = a_f
                .summary
                .strip_prefix("Admin panel access: ")
                .expect("summary must have expected prefix");
            assert_eq!(
                summary_uri.len(),
                120,
                "BC-2.06.007 pc-1 (truncation): summary URI portion must be exactly 120 chars \
                 when the URI exceeds 120 chars; got {} chars",
                summary_uri.len()
            );
            assert!(
                long_admin.starts_with(summary_uri),
                "BC-2.06.007 pc-1 (truncation): summary URI must be the first 120 chars of the \
                 full URI"
            );
            assert!(
                a_f.evidence[0].contains(&long_admin),
                "BC-2.06.007 pc-1 (truncation): evidence must contain the full raw URI (no \
                 truncation); evidence: {:?}",
                a_f.evidence[0]
            );
        }
    }

    // ── BC-2.06.007 / AC-007 ─────────────────────────────────────────────────────
    // AC-007: admin-panel URI comparison is case-insensitive and substring-based.

    /// BC-2.06.007 invariant 1-2 — admin-panel check is case-insensitive via
    /// lowercase and substring: "/ADMIN" (uppercase) triggers via "/admin";
    /// "/site/admin/settings" triggers via "/admin" substring.
    ///
    /// EC-009: URI = "/ADMIN" (uppercase) → admin-panel finding emitted.
    #[allow(non_snake_case)]
    #[test]
    fn test_admin_panel_case_insensitive() {
        // EC-009: uppercase "/ADMIN" must match via lowercase.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /ADMIN HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings()
                    .iter()
                    .any(|f| f.summary.contains("Admin panel")),
                "BC-2.06.007 invariant 1: '/ADMIN' (uppercase) must match via lowercase"
            );
        }

        // BC-2.06.007 invariant 2: substring match — "/site/admin/settings"
        // triggers via "/admin" substring.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /site/admin/settings HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings()
                    .iter()
                    .any(|f| f.summary.contains("Admin panel")),
                "BC-2.06.007 invariant 2: '/site/admin/settings' must match via '/admin' substring"
            );
        }

        // BC-2.06.007 invariant 1: mixed-case "/WP-Admin" must also match.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /WP-Admin/post.php HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );
            assert!(
                a.findings()
                    .iter()
                    .any(|f| f.summary.contains("Admin panel")),
                "BC-2.06.007 invariant 1: '/WP-Admin/post.php' must match via lowercase"
            );
        }
    }

    // ── BC-2.06.005 invariant 3 + BC-2.06.006 invariant 4 / AC-008 ───────────────
    // AC-008: all URI-based detections are independent; a URI matching multiple
    //         patterns emits multiple findings (no suppression).

    /// BC-2.06.005 invariant 3 + BC-2.06.006 invariant 4 — independent detections.
    ///
    /// EC-007: URI = "/cmd.php/../etc/passwd" → both web-shell (T1505.003) AND
    ///         path-traversal (T1083) findings emitted.
    ///
    /// The three detection blocks (path-traversal, web-shell, admin-panel) must all
    /// fire independently: a URI matching all three emits three separate findings.
    #[allow(non_snake_case)]
    #[test]
    fn test_multiple_detections_fire_independently() {
        // EC-007: URI matching both web-shell AND path-traversal patterns.
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            // "/cmd.php/../etc/passwd" contains "/cmd.php" (web-shell) AND "../" (path-traversal).
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /cmd.php/../etc/passwd HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );

            let findings = a.findings();
            let has_traversal = findings
                .iter()
                .any(|f| f.mitre_technique.as_deref() == Some("T1083"));
            let has_webshell = findings
                .iter()
                .any(|f| f.mitre_technique.as_deref() == Some("T1505.003"));

            assert!(
                has_traversal,
                "BC-2.06.005 invariant 3: path-traversal finding (T1083) must be emitted for \
                 '/cmd.php/../etc/passwd'"
            );
            assert!(
                has_webshell,
                "BC-2.06.006 invariant 4: web-shell finding (T1505.003) must be emitted for \
                 '/cmd.php/../etc/passwd'"
            );
        }

        // URI matching all three detection categories simultaneously:
        // "/wp-admin/../shell.php" → path-traversal (T1083) + web-shell (T1505.003)
        //                            + admin-panel (T1046)
        {
            let mut a = HttpAnalyzer::new();
            let fk = test_flow_key();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                b"GET /wp-admin/../shell.php HTTP/1.1\r\nHost: h\r\n\r\n",
                0,
            );

            let findings = a.findings();
            let traversal_count = findings
                .iter()
                .filter(|f| f.mitre_technique.as_deref() == Some("T1083"))
                .count();
            let webshell_count = findings
                .iter()
                .filter(|f| f.mitre_technique.as_deref() == Some("T1505.003"))
                .count();
            let admin_count = findings
                .iter()
                .filter(|f| f.mitre_technique.as_deref() == Some("T1046"))
                .count();

            assert_eq!(
                traversal_count, 1,
                "BC-2.06.005 invariant 3: path-traversal (T1083) must fire for \
                 '/wp-admin/../shell.php'"
            );
            assert_eq!(
                webshell_count, 1,
                "BC-2.06.006 invariant 4: web-shell (T1505.003) must fire for \
                 '/wp-admin/../shell.php'"
            );
            assert_eq!(
                admin_count, 1,
                "BC-2.06.007 invariant 4: admin-panel (T1046) must fire for \
                 '/wp-admin/../shell.php'"
            );
        }
    }

    // ── BC-2.06.012 / AC-009 ─────────────────────────────────────────────────────
    // AC-009: syntactically valid HTTP/1.1 GET with standard URI, non-empty Host,
    //         and non-empty User-Agent produces zero findings.

    /// BC-2.06.012 postcondition 1-3 — well-formed HTTP/1.1 request produces zero
    /// findings; method/host/UA counters updated normally; parse_errors unchanged.
    ///
    /// EC-010: URI = "/index.html" → zero findings.
    /// EC-012: GET /index.html HTTP/1.1 with Host and UA → zero findings.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_012_normal_request_zero_findings() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // BC-2.06.012 canonical test vector.
        let request =
            b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.0\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        // BC-2.06.012 postcondition 1: all_findings must gain zero new entries.
        assert!(
            analyzer.findings().is_empty(),
            "BC-2.06.012 postcondition 1: normal request must produce zero findings, \
             got: {:?}",
            analyzer.findings()
        );

        // BC-2.06.012 postcondition 2: method/host/UA counters updated normally.
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.012 postcondition 2: methods[GET] must be 1"
        );
        assert_eq!(
            *analyzer.host_counts().get("example.com").unwrap_or(&0),
            1,
            "BC-2.06.012 postcondition 2: hosts[example.com] must be 1"
        );
        assert_eq!(
            *analyzer.user_agent_counts().get("curl/7.0").unwrap_or(&0),
            1,
            "BC-2.06.012 postcondition 2: user_agents[curl/7.0] must be 1"
        );
        assert_eq!(
            analyzer.uri_list(),
            &["/index.html"],
            "BC-2.06.012 postcondition 2: uris must contain /index.html"
        );

        // BC-2.06.012 postcondition 3: parse_errors unchanged (zero).
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.012 postcondition 3: parse_errors must be 0 for well-formed request"
        );
    }

    // ── BC-2.06.012 / AC-010 ─────────────────────────────────────────────────────
    // AC-010: all anomaly detections are independently gated; none fires on clean
    //         input; zero findings is the expected steady state.

    /// BC-2.06.012 invariant 1 — each anomaly detection gate is independently
    /// inactive for clean input.
    ///
    /// This test exhaustively verifies that none of the individual detection
    /// branches fire on a normal /index.html request: no path-traversal (T1083),
    /// no web-shell (T1505.003), no admin-panel (T1046), no unusual-method,
    /// no missing/empty Host, no long-URI, no empty-UA finding.
    ///
    /// parse_errors must also remain 0 (no parse failure).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_012_normal_request_no_parse_errors() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        let request =
            b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        // Positive-parse anchor: the request must have been parsed and the method
        // counter incremented before any negative (absence) assertions are checked.
        // Without this, a silent parse failure would cause the detection block to
        // never execute, making all subsequent negative assertions vacuously true.
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "precondition: request must parse — methods[GET] must be 1 before testing \
             absence of findings"
        );

        // BC-2.06.012 postcondition 3: parse_errors must not increment.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.012 invariant 1: parse_errors must be 0 for well-formed request"
        );

        let findings = analyzer.findings();

        // BC-2.06.012 invariant 1: path-traversal gate must not fire.
        assert!(
            !findings
                .iter()
                .any(|f| f.mitre_technique.as_deref() == Some("T1083")),
            "BC-2.06.012 invariant 1: path-traversal (T1083) must not fire for '/index.html'"
        );

        // BC-2.06.012 invariant 1: web-shell gate must not fire.
        assert!(
            !findings
                .iter()
                .any(|f| f.mitre_technique.as_deref() == Some("T1505.003")),
            "BC-2.06.012 invariant 1: web-shell (T1505.003) must not fire for '/index.html'"
        );

        // BC-2.06.012 invariant 1: admin-panel gate must not fire.
        assert!(
            !findings
                .iter()
                .any(|f| f.mitre_technique.as_deref() == Some("T1046")),
            "BC-2.06.012 invariant 1: admin-panel (T1046) must not fire for '/index.html'"
        );

        // BC-2.06.012 invariant 1: zero findings overall — the complete steady-state check.
        assert!(
            findings.is_empty(),
            "BC-2.06.012 invariant 1: all anomaly gates are independently inactive for clean \
             input; zero findings is the expected steady state; got: {:?}",
            findings
        );
    }
} // mod bc_2_06_story042_formalization

// ---------------------------------------------------------------------------
// STORY-043 Brownfield-Formalization Tests
// BC-2.06.008 – BC-2.06.011: method, host, URI-length, user-agent anomalies
//
// Naming convention: test_BC_2_06_NNN_<descriptive_suffix>
// AC-named functions (e.g. test_detect_unusual_method) match the exact
// function names stated in STORY-043 Acceptance Criteria.
//
// All tests confirm existing src/analyzer/http.rs behaviour; they are
// expected to PASS (brownfield-formalization mode).  Any test that FAILS
// indicates a divergence between source and the behavioural contract and
// is documented as a source bug.
//
// The uppercase "BC" in function names is intentional per
// DF-AC-TEST-NAME-SYNC-001.  Each function carries its own
// #[allow(non_snake_case)] to keep the lint suppression narrow.
// ---------------------------------------------------------------------------
mod bc_2_06_043_formalization {
    use super::*;

    // ── BC-2.06.008 ───────────────────────────────────────────────────────────

    /// AC-001 (BC-2.06.008 postcondition 1) — CONNECT triggers
    /// Reconnaissance/Inconclusive/Medium, mitre=None, evidence="{method} {uri}",
    /// direction=ClientToServer.
    ///
    /// Covers BC-2.06.008 canonical test vector:
    ///   CONNECT proxy.example.com:443 HTTP/1.1 → Finding(Recon/Inconclusive/Medium)
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_008_detect_unusual_method() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // BC-2.06.008 canonical vector: CONNECT with a valid Host so the only
        // finding is the unusual-method one (no host-anomaly interference).
        let request = b"CONNECT proxy.example.com:443 HTTP/1.1\r\nHost: proxy.example.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        let findings = analyzer.findings();
        let method_finding = findings
            .iter()
            .find(|f| f.summary.contains("Unusual HTTP method"))
            .expect("BC-2.06.008 postcondition 1: CONNECT must produce an unusual-method finding");

        // Postcondition 1: category/verdict/confidence.
        assert_eq!(
            method_finding.category,
            ThreatCategory::Reconnaissance,
            "BC-2.06.008 postcondition 1: category must be Reconnaissance"
        );
        assert_eq!(
            method_finding.verdict,
            Verdict::Inconclusive,
            "BC-2.06.008 postcondition 1: verdict must be Inconclusive"
        );
        assert_eq!(
            method_finding.confidence,
            Confidence::Medium,
            "BC-2.06.008 postcondition 1: confidence must be Medium"
        );

        // Postcondition 1: mitre_technique is None (BC-2.06.008 invariant 3).
        assert_eq!(
            method_finding.mitre_technique, None,
            "BC-2.06.008 invariant 3: mitre_technique must be None for unusual-method finding"
        );

        // Postcondition 1: summary = "Unusual HTTP method: CONNECT".
        assert_eq!(
            method_finding.summary, "Unusual HTTP method: CONNECT",
            "BC-2.06.008 postcondition 1: summary must be 'Unusual HTTP method: CONNECT'"
        );

        // Postcondition 1: evidence = vec!["CONNECT proxy.example.com:443"].
        assert_eq!(
            method_finding.evidence,
            vec!["CONNECT proxy.example.com:443"],
            "BC-2.06.008 postcondition 1: evidence must be '<method> <uri>'"
        );

        // Postcondition 1: direction = Some(ClientToServer).
        assert_eq!(
            method_finding.direction,
            Some(Direction::ClientToServer),
            "BC-2.06.008 postcondition 1: direction must be Some(ClientToServer)"
        );
    }

    /// BC-2.06.008 invariants 1-2 + postcondition 3 (AC-002) — method matching
    /// is exact and case-sensitive; lowercase "delete" and standard methods
    /// do NOT trigger the detection.
    ///
    /// BC-2.06.008 EC-007: "delete" (lowercase) → no finding.
    /// BC-2.06.008 postcondition 3: GET, POST, PUT, PATCH, HEAD → no finding.
    #[allow(non_snake_case)]
    #[test]
    fn test_unusual_method_case_sensitive() {
        // ── Lowercase "delete" must NOT match uppercase "DELETE" ──────────────
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            // httparse silently accepts non-standard tokens; "delete" is a valid
            // method token but is not in the unusual_methods slice.
            // We use HTTP/1.0 to suppress the missing-Host finding so the
            // zero-findings assertion is unambiguous.
            let request = b"delete /resource HTTP/1.0\r\n\r\n";
            analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
            assert_eq!(
                *analyzer.method_counts().get("delete").unwrap_or(&0),
                1,
                "precondition: lowercase 'delete' must parse as a method (BC-2.06.008 invariant 2 anchor)"
            );
            assert!(
                !analyzer
                    .findings()
                    .iter()
                    .any(|f| f.summary.contains("Unusual HTTP method")),
                "BC-2.06.008 invariant 2: lowercase 'delete' must NOT trigger unusual-method finding"
            );
        }

        // ── All four unusual methods trigger on exact uppercase match ─────────
        for method in &["DELETE", "OPTIONS", "TRACE"] {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            // HTTP/1.0 to suppress missing-Host noise.
            let request = format!("{method} /resource HTTP/1.0\r\n\r\n");
            analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);
            assert!(
                analyzer
                    .findings()
                    .iter()
                    .any(|f| f.summary == format!("Unusual HTTP method: {method}")),
                "BC-2.06.008 invariant 1: {method} (uppercase) must trigger unusual-method finding"
            );
        }

        // ── Standard methods must NOT trigger ────────────────────────────────
        for method in &["GET", "POST", "PUT", "PATCH", "HEAD"] {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            let request = format!("{method} /resource HTTP/1.1\r\nHost: example.com\r\n\r\n");
            analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);
            assert_eq!(
                *analyzer.method_counts().get(*method).unwrap_or(&0),
                1,
                "precondition: standard method {method} must parse (BC-2.06.008 postcondition 3 anchor)"
            );
            assert!(
                !analyzer
                    .findings()
                    .iter()
                    .any(|f| f.summary.contains("Unusual HTTP method")),
                "BC-2.06.008 postcondition 3: standard method {method} must NOT trigger unusual-method finding"
            );
        }
    }

    // ── BC-2.06.009 ───────────────────────────────────────────────────────────

    /// AC-003 (BC-2.06.009 postcondition 1) — HTTP/1.1 with no Host header
    /// emits Anomaly/Inconclusive/Medium with summary "HTTP/1.1 request without
    /// Host header", evidence="{method} {uri}", direction=ClientToServer.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_009_detect_missing_host_header() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // BC-2.06.009 canonical vector: GET / HTTP/1.1 with no Host header.
        let request = b"GET /path HTTP/1.1\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        let findings = analyzer.findings();
        let host_finding = findings
            .iter()
            .find(|f| f.summary.contains("without Host header"))
            .expect(
                "BC-2.06.009 postcondition 1: HTTP/1.1 request without Host must produce finding",
            );

        // Postcondition 1: category/verdict/confidence.
        assert_eq!(
            host_finding.category,
            ThreatCategory::Anomaly,
            "BC-2.06.009 postcondition 1: category must be Anomaly"
        );
        assert_eq!(
            host_finding.verdict,
            Verdict::Inconclusive,
            "BC-2.06.009 postcondition 1: verdict must be Inconclusive"
        );
        assert_eq!(
            host_finding.confidence,
            Confidence::Medium,
            "BC-2.06.009 postcondition 1: confidence must be Medium"
        );

        // Postcondition 1: exact summary text (absent case).
        assert_eq!(
            host_finding.summary, "HTTP/1.1 request without Host header",
            "BC-2.06.009 postcondition 1: absent-Host summary must be exact"
        );

        // Postcondition 1: mitre_technique is None (BC-2.06.009 invariant 3).
        assert_eq!(
            host_finding.mitre_technique, None,
            "BC-2.06.009 invariant 3: mitre_technique must be None for host-anomaly finding"
        );

        // Postcondition 1: evidence = "{method} {uri}".
        assert_eq!(
            host_finding.evidence,
            vec!["GET /path"],
            "BC-2.06.009 postcondition 1: evidence must be '<method> <uri>'"
        );

        // Postcondition 1: direction = Some(ClientToServer).
        assert_eq!(
            host_finding.direction,
            Some(Direction::ClientToServer),
            "BC-2.06.009 postcondition 1: direction must be Some(ClientToServer)"
        );
    }

    /// AC-004 (BC-2.06.009 postcondition 1) — HTTP/1.1 with Host present but
    /// empty (after trim) emits a distinct summary "HTTP/1.1 request with empty
    /// Host header"; the absent-Host variant must NOT co-fire.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_009_detect_empty_host_header() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // BC-2.06.009 EC-003: Host header present with empty value.
        let request = b"GET /path HTTP/1.1\r\nHost: \r\nUser-Agent: curl/8.0\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        let findings = analyzer.findings();
        let host_finding = findings
            .iter()
            .find(|f| f.summary.contains("empty Host header"))
            .expect(
                "BC-2.06.009 postcondition 1: empty-Host must produce 'with empty Host header' finding",
            );

        // Postcondition 1: category/verdict/confidence.
        assert_eq!(
            host_finding.category,
            ThreatCategory::Anomaly,
            "BC-2.06.009 postcondition 1: category must be Anomaly"
        );
        assert_eq!(
            host_finding.verdict,
            Verdict::Inconclusive,
            "BC-2.06.009 postcondition 1: verdict must be Inconclusive"
        );
        assert_eq!(
            host_finding.confidence,
            Confidence::Medium,
            "BC-2.06.009 postcondition 1: confidence must be Medium"
        );

        // Postcondition 1: exact summary — distinct from the absent case.
        assert_eq!(
            host_finding.summary, "HTTP/1.1 request with empty Host header",
            "BC-2.06.009 postcondition 1: empty-Host summary must be exact and distinct"
        );

        // Absent-Host variant must NOT also fire (two distinct cases, not one).
        assert!(
            !findings
                .iter()
                .any(|f| f.summary.contains("without Host header")),
            "BC-2.06.009 postcondition 1: empty-Host must not also trigger absent-Host variant"
        );
    }

    /// AC-005 (BC-2.06.009 postcondition 3 + invariant 1) — HTTP/1.0 requests
    /// are completely exempt from the Host check.  Neither absent nor empty Host
    /// triggers the finding for version==0.
    ///
    /// BC-2.06.009 EC-005/006: HTTP/1.0 with no Host → no finding.
    /// BC-2.06.009 EC-006: HTTP/1.0 with empty Host → no finding.
    /// Whitespace-only `Host:   ` on HTTP/1.1 → Some("") after trim → finding.
    #[allow(non_snake_case)]
    #[test]
    fn test_http10_no_host_finding() {
        // ── HTTP/1.0 with no Host header ──────────────────────────────────────
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            let request = b"GET /resource HTTP/1.0\r\n\r\n";
            analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
            assert_eq!(
                *analyzer.method_counts().get("GET").unwrap_or(&0),
                1,
                "precondition: HTTP/1.0 request without Host must parse (BC-2.06.009 postcondition 3 anchor)"
            );
            assert!(
                !analyzer
                    .findings()
                    .iter()
                    .any(|f| f.summary.contains("Host header")),
                "BC-2.06.009 postcondition 3: HTTP/1.0 without Host must NOT produce host-anomaly finding"
            );
        }

        // ── HTTP/1.0 with empty Host header ───────────────────────────────────
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            let request = b"GET /resource HTTP/1.0\r\nHost: \r\n\r\n";
            analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
            assert_eq!(
                *analyzer.method_counts().get("GET").unwrap_or(&0),
                1,
                "precondition: HTTP/1.0 request with empty Host must parse (BC-2.06.009 EC-006 anchor)"
            );
            assert!(
                !analyzer
                    .findings()
                    .iter()
                    .any(|f| f.summary.contains("Host header")),
                "BC-2.06.009 EC-006: HTTP/1.0 with empty Host must NOT produce host-anomaly finding"
            );
        }

        // ── HTTP/1.1 with whitespace-only Host → Some("") after trim → finding ─
        // (BC-2.06.009 invariant 2: find_header trims; Host:   \r\n → Some(""))
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            let request = b"GET /path HTTP/1.1\r\nHost:   \r\n\r\n";
            analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
            assert_eq!(
                *analyzer.method_counts().get("GET").unwrap_or(&0),
                1,
                "precondition: HTTP/1.1 whitespace-only-Host request must parse (BC-2.06.009 invariant 2 anchor)"
            );
            assert!(
                analyzer
                    .findings()
                    .iter()
                    .any(|f| f.summary.contains("empty Host header")),
                "BC-2.06.009 invariant 2: whitespace-only Host on HTTP/1.1 must produce empty-Host finding"
            );
        }
    }

    // ── BC-2.06.010 ───────────────────────────────────────────────────────────

    /// AC-006 (BC-2.06.010 postcondition 1) — URI > 2048 bytes emits
    /// Execution/Likely/Medium with exact byte count in summary and truncated
    /// 200-char prefix in evidence.
    ///
    /// BC-2.06.010 canonical vector: GET /<2049 A chars> HTTP/1.1.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_010_detect_long_uri() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // URI of 2101 bytes total (1 leading "/" + 2100 "A"s).
        let long_path = "/".to_string() + &"A".repeat(2100);
        let uri_len = long_path.len(); // 2101
        let request = format!("GET {long_path} HTTP/1.1\r\nHost: target.com\r\n\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

        let findings = analyzer.findings();
        let long_uri_finding = findings
            .iter()
            .find(|f| f.summary.contains("Abnormally long URI"))
            .expect(
                "BC-2.06.010 postcondition 1: URI > 2048 must produce Abnormally-long-URI finding",
            );

        // Postcondition 1: category/verdict/confidence.
        assert_eq!(
            long_uri_finding.category,
            ThreatCategory::Execution,
            "BC-2.06.010 postcondition 1: category must be Execution"
        );
        assert_eq!(
            long_uri_finding.verdict,
            Verdict::Likely,
            "BC-2.06.010 postcondition 1: verdict must be Likely"
        );
        assert_eq!(
            long_uri_finding.confidence,
            Confidence::Medium,
            "BC-2.06.010 postcondition 1: confidence must be Medium"
        );

        // Postcondition 1: mitre_technique is None (BC-2.06.010 invariant 4).
        assert_eq!(
            long_uri_finding.mitre_technique, None,
            "BC-2.06.010 invariant 4: mitre_technique must be None for long-URI finding"
        );

        // Postcondition 1 + invariant 3: summary contains exact byte count.
        assert_eq!(
            long_uri_finding.summary,
            format!("Abnormally long URI ({uri_len} chars)"),
            "BC-2.06.010 postcondition 1: summary must include exact byte count"
        );

        // Postcondition 1 + invariant 2: evidence truncated to EXACTLY 200 chars via truncate_uri.
        assert!(
            long_uri_finding.evidence[0].starts_with("URI prefix:"),
            "BC-2.06.010 postcondition 1: evidence must start with 'URI prefix:'"
        );
        // The evidence must NOT contain the full 2101-char URI (it's truncated to 200).
        let prefix_value = long_uri_finding.evidence[0]
            .strip_prefix("URI prefix: ")
            .unwrap_or(&long_uri_finding.evidence[0]);
        // BC-2.06.010 invariant 2: truncation is to EXACTLY 200 chars (not merely <=200).
        // Input: "/" + "A"*2100 = 2101 chars; expected prefix: "/" + "A"*199 = 200 chars.
        assert_eq!(
            prefix_value.len(),
            200,
            "BC-2.06.010 invariant 2: evidence URI prefix must be exactly 200 chars, got {}",
            prefix_value.len()
        );
        assert_eq!(
            prefix_value,
            "/".to_string() + &"A".repeat(199),
            "BC-2.06.010 invariant 2: evidence URI prefix must be the first 200 chars of the input URI"
        );

        // Postcondition 1: direction = Some(ClientToServer).
        assert_eq!(
            long_uri_finding.direction,
            Some(Direction::ClientToServer),
            "BC-2.06.010 postcondition 1: direction must be Some(ClientToServer)"
        );
    }

    /// AC-007 (BC-2.06.010 invariants 1-3) — long-URI threshold is strictly
    /// greater-than 2048.  uri.len() == 2048 must NOT fire; uri.len() == 2049
    /// must fire.  Summary uses the exact byte count (not the truncated length).
    ///
    /// BC-2.06.010 EC-001: len=2048 → no finding.
    /// BC-2.06.010 EC-002: len=2049 → finding with "2049 chars" in summary.
    #[allow(non_snake_case)]
    #[test]
    fn test_long_uri_boundary_exactly_2048() {
        // ── URI of exactly 2048 bytes — must NOT fire ─────────────────────────
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            // "/" + 2047 "A"s = 2048 bytes total.
            let uri_2048 = "/".to_string() + &"A".repeat(2047);
            assert_eq!(
                uri_2048.len(),
                2048,
                "precondition: URI must be exactly 2048 bytes"
            );
            let request = format!("GET {uri_2048} HTTP/1.1\r\nHost: x.com\r\n\r\n");
            analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);
            assert_eq!(
                *analyzer.method_counts().get("GET").unwrap_or(&0),
                1,
                "precondition: 2048-byte-URI request must parse (BC-2.06.010 invariant 1 anchor)"
            );
            assert!(
                !analyzer
                    .findings()
                    .iter()
                    .any(|f| f.summary.contains("Abnormally long URI")),
                "BC-2.06.010 invariant 1: URI of exactly 2048 bytes must NOT trigger long-URI finding"
            );
        }

        // ── URI of exactly 2049 bytes — MUST fire ────────────────────────────
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            // "/" + 2048 "A"s = 2049 bytes total.
            let uri_2049 = "/".to_string() + &"A".repeat(2048);
            assert_eq!(
                uri_2049.len(),
                2049,
                "precondition: URI must be exactly 2049 bytes"
            );
            let request = format!("GET {uri_2049} HTTP/1.1\r\nHost: x.com\r\n\r\n");
            analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

            let findings = analyzer.findings();
            let long_uri_finding = findings
                .iter()
                .find(|f| f.summary.contains("Abnormally long URI"))
                .expect("BC-2.06.010 invariant 1: URI of 2049 bytes must trigger long-URI finding");

            // Invariant 3: summary contains the exact byte count (2049, not truncated).
            assert_eq!(
                long_uri_finding.summary, "Abnormally long URI (2049 chars)",
                "BC-2.06.010 invariant 3: summary must include exact byte count 2049"
            );
        }
    }

    // ── BC-2.06.011 ───────────────────────────────────────────────────────────

    /// AC-008 (BC-2.06.011 postcondition 1) — User-Agent present with empty
    /// value after trim emits Anomaly/Inconclusive/Low with summary "Empty
    /// User-Agent header", evidence="{method} {uri}", direction=ClientToServer.
    ///
    /// BC-2.06.011 canonical vector: GET / HTTP/1.1 with User-Agent: (empty).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_011_detect_empty_user_agent() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // BC-2.06.011 canonical vector: empty User-Agent (trailing space after colon).
        let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\nUser-Agent: \r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        let findings = analyzer.findings();
        let ua_finding = findings
            .iter()
            .find(|f| f.summary.contains("Empty User-Agent"))
            .expect("BC-2.06.011 postcondition 1: empty UA must produce Empty-User-Agent finding");

        // Postcondition 1: category/verdict/confidence.
        assert_eq!(
            ua_finding.category,
            ThreatCategory::Anomaly,
            "BC-2.06.011 postcondition 1: category must be Anomaly"
        );
        assert_eq!(
            ua_finding.verdict,
            Verdict::Inconclusive,
            "BC-2.06.011 postcondition 1: verdict must be Inconclusive"
        );
        assert_eq!(
            ua_finding.confidence,
            Confidence::Low,
            "BC-2.06.011 postcondition 1: confidence must be Low"
        );

        // Postcondition 1: exact summary.
        assert_eq!(
            ua_finding.summary, "Empty User-Agent header",
            "BC-2.06.011 postcondition 1: summary must be 'Empty User-Agent header'"
        );

        // Postcondition 1: mitre_technique is None (BC-2.06.011 invariant 3).
        assert_eq!(
            ua_finding.mitre_technique, None,
            "BC-2.06.011 invariant 3: mitre_technique must be None for empty-UA finding"
        );

        // Postcondition 1: evidence = "{method} {uri}".
        assert_eq!(
            ua_finding.evidence,
            vec!["GET /page"],
            "BC-2.06.011 postcondition 1: evidence must be '<method> <uri>'"
        );

        // Postcondition 1: direction = Some(ClientToServer).
        assert_eq!(
            ua_finding.direction,
            Some(Direction::ClientToServer),
            "BC-2.06.011 postcondition 1: direction must be Some(ClientToServer)"
        );
    }

    /// AC-009 (BC-2.06.011 postcondition 2 + invariant 2) — absent User-Agent
    /// (None) must NOT produce any finding.
    ///
    /// BC-2.06.011 EC-002: no User-Agent header present → no finding.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_011_missing_user_agent_no_finding() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // No User-Agent header at all.
        let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        // Positive-parse anchor: request must have been parsed before asserting absence.
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "precondition: absent-UA request must parse (BC-2.06.011 postcondition 2 anchor)"
        );

        // BC-2.06.011 postcondition 2: no finding when UA is absent (None).
        assert!(
            !analyzer
                .findings()
                .iter()
                .any(|f| f.summary.contains("User-Agent")),
            "BC-2.06.011 postcondition 2: absent User-Agent (None) must NOT trigger empty-UA finding"
        );

        // BC-2.06.011 invariant 2: the asymmetry is intentional — absent=no finding,
        // empty=finding.  Pin only the UA-specific behavior; do not couple to absence of
        // all detections (which would fail if another detection fires on the same input).
        // The targeted User-Agent assertion above already covers this contract.
        // If an unrelated finding fires here it is a separate BC's concern, not AC-009.
    }

    /// AC-010 (BC-2.06.011 invariant 1) — whitespace-only User-Agent value
    /// is folded to Some("") by find_header's .trim() and must trigger the
    /// empty-UA finding.
    ///
    /// BC-2.06.011 EC-004: User-Agent:   (spaces only) → Some("") after trim → finding.
    /// BC-2.06.011 invariant 1: find_header returns Some("") for User-Agent: \r\n.
    #[allow(non_snake_case)]
    #[test]
    fn test_whitespace_user_agent_triggers_empty_ua_finding() {
        // ── Space-only value ──────────────────────────────────────────────────
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\nUser-Agent:   \r\n\r\n";
            analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
            let findings = analyzer.findings();
            assert!(
                findings
                    .iter()
                    .any(|f| f.summary == "Empty User-Agent header"),
                "BC-2.06.011 invariant 1: space-only User-Agent must trigger empty-UA finding"
            );
        }

        // ── Header present with no value at all (bare colon + CRLF) ──────────
        // BC-2.06.011 invariant 1: User-Agent: \r\n → trim("") → Some("") → finding.
        {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\nUser-Agent:\r\n\r\n";
            analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
            let findings = analyzer.findings();
            assert!(
                findings
                    .iter()
                    .any(|f| f.summary == "Empty User-Agent header"),
                "BC-2.06.011 invariant 1: bare 'User-Agent:\\r\\n' (no space) must trigger empty-UA finding"
            );
        }
    }

    // ── Multi-anomaly co-occurrence tests ─────────────────────────────────────

    /// BC-2.06.011 EC-005 / STORY-043 EC-013 — empty UA + missing Host on
    /// HTTP/1.1 both fire independently on the same request.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_011_empty_ua_and_missing_host_both_fire_independently() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // HTTP/1.1, no Host, empty User-Agent — both detections should fire.
        let request = b"GET /resource HTTP/1.1\r\nUser-Agent: \r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

        // Positive-parse anchor: request must have been parsed before asserting findings.
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "precondition: dual-anomaly request must parse (BC-2.06.009/BC-2.06.011 parse anchor)"
        );

        let findings = analyzer.findings();
        assert!(
            findings
                .iter()
                .any(|f| f.summary.contains("without Host header")),
            "BC-2.06.009: missing-Host finding must fire"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.summary == "Empty User-Agent header"),
            "BC-2.06.011: empty-UA finding must fire"
        );
    }

    /// STORY-043 EC-014 — long URI + path traversal both fire independently.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_010_long_uri_and_path_traversal_both_fire_independently() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Build a URI that is >2048 chars AND contains "../" for path-traversal.
        let long_traversal = "/../../".to_string() + &"A".repeat(2048);
        let request = format!("GET {long_traversal} HTTP/1.1\r\nHost: target.com\r\n\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

        let findings = analyzer.findings();
        assert!(
            findings
                .iter()
                .any(|f| f.summary.contains("Abnormally long URI")),
            "BC-2.06.010: long-URI finding must fire even when path-traversal also matches"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.summary.contains("Path traversal")),
            "BC-2.06.005: path-traversal finding must fire even when long-URI also matches"
        );
    }

    /// BC-2.06.008 — all four unusual methods each fire their own finding.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_008_all_four_unusual_methods_emit_finding() {
        for method in &["CONNECT", "TRACE", "DELETE", "OPTIONS"] {
            let mut analyzer = HttpAnalyzer::new();
            let fk = test_flow_key();
            // HTTP/1.0 to suppress missing-Host noise.
            let request = format!("{method} /resource HTTP/1.0\r\n\r\n");
            analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

            let findings = analyzer.findings();
            assert!(
                findings
                    .iter()
                    .any(|f| f.summary == format!("Unusual HTTP method: {method}")),
                "BC-2.06.008: {method} must produce 'Unusual HTTP method: {method}' finding"
            );
            // Per-finding category/confidence spot-check.
            let f = findings
                .iter()
                .find(|f| f.summary.contains("Unusual HTTP method"))
                .unwrap();
            assert_eq!(f.category, ThreatCategory::Reconnaissance);
            assert_eq!(f.verdict, Verdict::Inconclusive);
            assert_eq!(f.confidence, Confidence::Medium);
            assert_eq!(f.mitre_technique, None);
        }
    }

    /// BC-2.06.010 — URI of 10000 chars fires; evidence truncated to 200 chars.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_010_very_long_uri_evidence_truncated_to_200() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        let uri = "/".to_string() + &"X".repeat(9999); // 10000 chars total
        let request = format!("GET {uri} HTTP/1.1\r\nHost: x.com\r\n\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, request.as_bytes(), 0);

        let findings = analyzer.findings();
        let f = findings
            .iter()
            .find(|f| f.summary.contains("Abnormally long URI"))
            .expect("10000-char URI must trigger long-URI finding");

        // Invariant 3: exact byte count in summary (10000, NOT 200).
        assert_eq!(
            f.summary, "Abnormally long URI (10000 chars)",
            "BC-2.06.010 invariant 3: summary must be exactly 'Abnormally long URI (10000 chars)'"
        );

        // Invariant 2: evidence truncated to EXACTLY 200 chars.
        // Input: "/" + "X"*9999 = 10000 chars; expected prefix: "/" + "X"*199 = 200 chars.
        let prefix = f.evidence[0]
            .strip_prefix("URI prefix: ")
            .unwrap_or(&f.evidence[0]);
        assert_eq!(
            prefix.len(),
            200,
            "BC-2.06.010 invariant 2: evidence URI prefix must be exactly 200 chars, got {}",
            prefix.len()
        );
        assert_eq!(
            prefix,
            "/".to_string() + &"X".repeat(199),
            "BC-2.06.010 invariant 2: evidence URI prefix must be the first 200 chars of the input URI"
        );
    }
} // mod bc_2_06_043_formalization

mod bc_2_06_044_formalization {
    use super::*;

    // ── BC-2.06.013 ───────────────────────────────────────────────────────────────
    // Non-HTTP Bytes Increment parse_errors; No Token-Error Findings

    /// BC-2.06.013 postconditions 1-5 — SSH-like bytes in request direction:
    /// parse_errors incremented by 1, no finding emitted, early return.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_013_non_http_bytes_increment_parse_errors_no_finding() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Canonical test vector: SSH-like bytes → httparse::Error::Token
        analyzer.on_data(
            &fk,
            Direction::ClientToServer,
            b"SSH-2.0-OpenSSH\r\n\r\n",
            0,
        );

        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "BC-2.06.013 postcondition 1: parse_errors must be 1 after one non-HTTP buffer"
        );
        assert!(
            analyzer.findings().is_empty(),
            "BC-2.06.013 postcondition 4: no finding must be emitted for a token error"
        );
        // Postcondition 3 (buf clear): no method counted — buffer was cleared.
        assert!(
            analyzer.method_counts().get("SSH-2.0-OpenSSH").is_none(),
            "BC-2.06.013 postcondition 3: request_buf must be cleared after error"
        );
    }

    /// BC-2.06.013 canonical test vector 2 — binary garbage bytes.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_013_binary_garbage_increments_parse_errors() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Binary garbage: 0xFF 0xFE = invalid UTF-8 and invalid HTTP token.
        analyzer.on_data(
            &fk,
            Direction::ClientToServer,
            b"\xff\xfe binary garbage",
            0,
        );

        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "BC-2.06.013 EC-001: binary garbage must increment parse_errors to 1"
        );
        assert!(
            analyzer.findings().is_empty(),
            "BC-2.06.013 EC-001: no finding for binary garbage (token error only)"
        );
    }

    /// BC-2.06.013 invariant 1 — had_success suppresses error counting for body bytes.
    /// A complete HTTP request header followed immediately by body bytes in the same
    /// on_data call must NOT increment parse_errors.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_013_invariant_had_success_suppresses_body_byte_errors() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Normal request + binary body bytes (NUL, which Err(Token) on re-parse).
        let mut req = b"GET /resource HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec();
        req.push(0x00); // NUL — causes parse error on next loop iteration
        analyzer.on_data(&fk, Direction::ClientToServer, &req, 0);

        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.013 invariant 1: body bytes after successful header must NOT increment parse_errors"
        );
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.013 invariant 1: GET must be counted despite body bytes following the header"
        );
    }

    /// BC-2.06.013 invariant 2 — TooManyHeaders is the only Err that also emits a finding;
    /// confirmed by verifying a token error does NOT emit a finding.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_013_invariant_token_error_does_not_emit_finding() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Any string that causes httparse::Error::Token (not TooManyHeaders).
        analyzer.on_data(&fk, Direction::ClientToServer, b"NOT_HTTP\r\n\r\n", 0);

        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "BC-2.06.013: token error must increment parse_errors"
        );
        let findings = analyzer.findings();
        assert!(
            findings.is_empty(),
            "BC-2.06.013 invariant 2: token error must NOT emit a finding; got: {:?}",
            findings
        );
    }

    // ── BC-2.06.014 ───────────────────────────────────────────────────────────────
    // Too Many Headers Emits Anomaly/Inconclusive/Medium Finding (T1499.002)

    /// BC-2.06.014 postconditions 1-5 — request with 97 headers (exceeds MAX_HEADERS=96)
    /// must emit exactly one finding with all required fields.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_014_too_many_headers_request_emits_anomaly_finding() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        let mut request = b"GET / HTTP/1.1\r\n".to_vec();
        for i in 0..97 {
            request.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
        }
        request.extend_from_slice(b"\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, &request, 0);

        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "BC-2.06.014 postcondition 2: TooManyHeaders must increment parse_errors"
        );
        let findings = analyzer.findings();
        assert_eq!(
            findings.len(),
            1,
            "BC-2.06.014 postcondition 1: exactly one finding must be emitted"
        );
        let f = &findings[0];
        assert_eq!(
            f.category,
            ThreatCategory::Anomaly,
            "BC-2.06.014 postcondition 1: category must be Anomaly"
        );
        assert_eq!(
            f.verdict,
            Verdict::Inconclusive,
            "BC-2.06.014 postcondition 1: verdict must be Inconclusive"
        );
        assert_eq!(
            f.confidence,
            Confidence::Medium,
            "BC-2.06.014 postcondition 1: confidence must be Medium"
        );
        assert_eq!(
            f.mitre_technique.as_deref(),
            Some("T1499.002"),
            "BC-2.06.014 postcondition 1: mitre_technique must be T1499.002"
        );
        assert_eq!(
            f.summary,
            "Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)",
            "BC-2.06.014 postcondition 1: summary text must match exactly"
        );
        assert_eq!(
            f.evidence,
            vec!["Direction: request".to_string()],
            "BC-2.06.014 postcondition 1 / invariant 4: evidence must be plain string 'Direction: request'"
        );
        assert_eq!(
            f.direction,
            Some(Direction::ClientToServer),
            "BC-2.06.014 postcondition 1: direction field must be ClientToServer"
        );
    }

    /// BC-2.06.014 postconditions 1-5 (response side) — response with 97 headers
    /// must emit a finding with "Direction: response" evidence.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_014_too_many_headers_response_emits_anomaly_finding() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        let mut response = b"HTTP/1.1 200 OK\r\n".to_vec();
        for i in 0..97 {
            response.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
        }
        response.extend_from_slice(b"\r\n");
        analyzer.on_data(&fk, Direction::ServerToClient, &response, 0);

        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "BC-2.06.014 postcondition 2 (response): parse_errors must be 1"
        );
        let findings = analyzer.findings();
        assert_eq!(
            findings.len(),
            1,
            "BC-2.06.014: exactly one finding for TooManyHeaders on response"
        );
        let f = &findings[0];
        assert_eq!(
            f.evidence,
            vec!["Direction: response".to_string()],
            "BC-2.06.014 invariant 4: response evidence must be plain string 'Direction: response'"
        );
        assert_eq!(
            f.direction,
            Some(Direction::ServerToClient),
            "BC-2.06.014 postcondition 1: direction field must be ServerToClient"
        );
        assert_eq!(
            f.mitre_technique.as_deref(),
            Some("T1499.002"),
            "BC-2.06.014: mitre_technique must be T1499.002 for response side"
        );
    }

    /// BC-2.06.014 invariant 3 — TooManyHeaders does NOT bypass the error-count path;
    /// repeated TooManyHeaders advances toward poisoning.  On the 3rd consecutive
    /// TooManyHeaders the direction is poisoned AND the finding is emitted.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_014_invariant_too_many_headers_contributes_to_poison_threshold() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Build a canonical too-many-headers request (97 headers).
        let build_tmh_request = || {
            let mut req = b"GET / HTTP/1.1\r\n".to_vec();
            for i in 0..97 {
                req.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
            }
            req.extend_from_slice(b"\r\n");
            req
        };

        let req = build_tmh_request();
        analyzer.on_data(&fk, Direction::ClientToServer, &req, 0);
        assert_eq!(analyzer.parse_error_count(), 1);
        assert_eq!(analyzer.findings().len(), 1);

        let req2 = build_tmh_request();
        analyzer.on_data(&fk, Direction::ClientToServer, &req2, 0);
        assert_eq!(analyzer.parse_error_count(), 2);
        assert_eq!(analyzer.findings().len(), 2);

        // Third: poisons the direction AND emits a finding.
        let req3 = build_tmh_request();
        analyzer.on_data(&fk, Direction::ClientToServer, &req3, 0);
        assert_eq!(
            analyzer.parse_error_count(),
            3,
            "BC-2.06.014 invariant 3: third TooManyHeaders must increment parse_errors to 3"
        );
        assert_eq!(
            analyzer.findings().len(),
            3,
            "BC-2.06.014 EC-003: third TooManyHeaders must emit a finding too"
        );

        // Fourth: direction is now poisoned; bytes should be skipped without parsing.
        let before = analyzer.poisoned_bytes_skipped();
        let extra = b"GET / HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, extra, 0);
        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            before + extra.len() as u64,
            "BC-2.06.014 invariant 3: direction must be poisoned after 3 TooManyHeaders errors"
        );
    }

    /// BC-2.06.014 invariant 4 — evidence text is a plain hardcoded string, not derived
    /// from the Direction enum.  Specifically "Direction: request" and "Direction: response".
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_014_invariant_evidence_is_plain_string_not_enum_derived() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();
        let fk2 = test_flow_key_b();

        let mut req = b"GET / HTTP/1.1\r\n".to_vec();
        for i in 0..97 {
            req.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
        }
        req.extend_from_slice(b"\r\n");
        analyzer.on_data(&fk, Direction::ClientToServer, &req, 0);

        let mut resp = b"HTTP/1.1 200 OK\r\n".to_vec();
        for i in 0..97 {
            resp.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
        }
        resp.extend_from_slice(b"\r\n");
        analyzer.on_data(&fk2, Direction::ServerToClient, &resp, 0);

        let findings = analyzer.findings();
        let req_finding = findings
            .iter()
            .find(|f| f.direction == Some(Direction::ClientToServer))
            .expect("must have request-direction finding");
        let resp_finding = findings
            .iter()
            .find(|f| f.direction == Some(Direction::ServerToClient))
            .expect("must have response-direction finding");

        assert_eq!(
            req_finding.evidence[0], "Direction: request",
            "BC-2.06.014 invariant 4: request evidence must be exactly 'Direction: request'"
        );
        assert_eq!(
            resp_finding.evidence[0], "Direction: response",
            "BC-2.06.014 invariant 4: response evidence must be exactly 'Direction: response'"
        );
    }

    // ── BC-2.06.015 ───────────────────────────────────────────────────────────────
    // After 3 Consecutive Parse Errors a Direction is Poisoned; Subsequent Bytes Skipped

    /// BC-2.06.015 postconditions 1-4 — 3 consecutive errors trigger poisoning;
    /// subsequent bytes counted in poisoned_bytes_skipped without parsing.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_015_three_consecutive_errors_trigger_poisoning() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Canonical test vector: 3 consecutive non-HTTP chunks.
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK1\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK2\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK3\r\n\r\n", 0);

        assert_eq!(
            analyzer.parse_error_count(),
            3,
            "BC-2.06.015 postcondition 1 precursor: parse_errors must be 3 at poison threshold"
        );

        // Postcondition 4: subsequent bytes skipped without parsing.
        let post_poison = b"GET /index.html HTTP/1.1\r\nHost: x.com\r\n\r\n";
        let before = analyzer.poisoned_bytes_skipped();
        analyzer.on_data(&fk, Direction::ClientToServer, post_poison, 0);

        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            before + post_poison.len() as u64,
            "BC-2.06.015 postcondition 4: subsequent bytes must be counted in poisoned_bytes_skipped"
        );
        assert!(
            analyzer.method_counts().get("GET").is_none(),
            "BC-2.06.015 postcondition 4: poisoned direction must NOT parse the request"
        );
    }

    /// BC-2.06.015 postcondition 2 — non_http_flows incremented on first direction poisoned.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_015_non_http_flows_incremented_on_first_poison() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        }

        let summary = analyzer.summarize();
        assert_eq!(
            summary.detail["non_http_flows"],
            serde_json::json!(1),
            "BC-2.06.015 postcondition 2: non_http_flows must be 1 after first direction poisoned"
        );
    }

    /// BC-2.06.015 invariant 2 — error counter is CONSECUTIVE, not cumulative.
    /// One successful parse resets it to 0.  Canonical test vector: 2 bad + 1 good + 2 bad.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_015_invariant_error_count_is_consecutive_not_cumulative() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // 2 errors (below threshold).
        analyzer.on_data(&fk, Direction::ClientToServer, b"BAD1\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"BAD2\r\n\r\n", 0);

        // 1 success — resets consecutive count to 0.
        let good = b"GET /ok HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, good, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.015 invariant 2: valid request must parse after 2 errors"
        );

        // 2 more errors — consecutive count is now 2, NOT 4 (reset happened).
        analyzer.on_data(&fk, Direction::ClientToServer, b"BAD3\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"BAD4\r\n\r\n", 0);

        // Another valid request must succeed — only 2 consecutive errors, not poisoned.
        let good2 = b"GET /ok2 HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, good2, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            2,
            "BC-2.06.015 invariant 2: reset on success prevents cumulative poisoning; \
             consecutive count is 2 not 4 — direction must not be poisoned"
        );
    }

    /// BC-2.06.015 invariant 3 — poisoning is irreversible within a flow lifetime.
    /// Once poisoned, the direction never un-poisons (except via on_flow_close).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_015_invariant_poisoning_is_irreversible() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Poison the direction.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK\r\n\r\n", 0);
        }

        // Send 1000 bytes — all skipped.
        let payload: Vec<u8> = vec![b'A'; 1000];
        analyzer.on_data(&fk, Direction::ClientToServer, &payload, 0);

        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            1000,
            "BC-2.06.015 invariant 3 / EC-004: 1000 bytes to poisoned direction must all be skipped"
        );

        // Send a valid HTTP request — must still be skipped (irreversible).
        let valid = b"GET /attempt HTTP/1.1\r\nHost: x.com\r\n\r\n";
        let before = analyzer.poisoned_bytes_skipped();
        analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);
        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            before + valid.len() as u64,
            "BC-2.06.015 invariant 3: poisoning is irreversible — valid bytes still skipped"
        );
        assert!(
            analyzer.method_counts().get("GET").is_none(),
            "BC-2.06.015 invariant 3: GET must never be parsed once direction is poisoned"
        );
    }

    // ── BC-2.06.016 ───────────────────────────────────────────────────────────────
    // Single Parse Error Does NOT Poison

    /// BC-2.06.016 postconditions 1-5 — single error increments counters but
    /// does NOT trigger poisoning; subsequent valid request parses normally.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_016_single_error_does_not_poison_direction() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);

        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "BC-2.06.016 postcondition 3: parse_errors must be 1"
        );

        // Postcondition 2: not poisoned.
        let valid = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.016 postcondition 5: subsequent valid request must parse after single error"
        );
        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            0,
            "BC-2.06.016 postcondition 2: no bytes should be skipped after a single error"
        );
    }

    /// BC-2.06.016 invariant 2 — error_count reset on success means threshold measures
    /// CONSECUTIVE errors.  EC-001: 1 error then valid request — error_count back to 0.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_016_invariant_single_error_then_success_resets_count() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // 1 error.
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);

        // Success — count reset.
        let valid = b"GET /first HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.016 EC-001: valid request after single error must parse"
        );

        // Now need 3 new consecutive errors to poison (the reset is proven by
        // the fact that 2 more errors + 1 good still parse).
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK1\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK2\r\n\r\n", 0);
        let valid2 = b"GET /second HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, valid2, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            2,
            "BC-2.06.016 invariant 2: 2 errors after a reset must not poison; second GET must parse"
        );
    }

    /// BC-2.06.016 EC-003 — 2 errors, then 1 error (not consecutive reset):
    /// NOT poisoned; count after the third call is 1 (because a success
    /// intervened... actually no — this EC says no success intervened but
    /// the pattern is 2 errors + 1 non-consecutive error).
    ///
    /// Re-reading BC-2.06.016 EC-003: "2 errors, then 1 error (not consecutive reset)"
    /// means 2 + 1 = 3 TOTAL but NOT 3 CONSECUTIVE because there was a reset in between.
    /// This effectively tests: 2 errors → success → 1 error → count=1, not poisoned.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_016_ec003_two_errors_success_one_error_count_one() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // 2 consecutive errors.
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK1\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK2\r\n\r\n", 0);

        // Success — resets count to 0.
        let good = b"GET /ok HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, good, 0);

        // 1 more error — consecutive count is 1 now (not 3).
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK3\r\n\r\n", 0);

        // Must not be poisoned — count is 1, below threshold.
        let good2 = b"GET /ok2 HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, good2, 0);
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            2,
            "BC-2.06.016 EC-003: after 2 errors + success + 1 error, count is 1 — must NOT be poisoned"
        );
    }

    // ── BC-2.06.017 ───────────────────────────────────────────────────────────────
    // Poisoning is Per-Direction; Poisoned Request Does Not Affect Response

    /// BC-2.06.017 postconditions 1-3 — request poisoned; response continues normally.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_017_poisoned_request_does_not_affect_response_parsing() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Poison request direction.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        }

        // Response direction: valid response must parse normally.
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        let before = analyzer.poisoned_bytes_skipped();
        analyzer.on_data(&fk, Direction::ServerToClient, response, 0);

        assert_eq!(
            analyzer.transaction_count(),
            1,
            "BC-2.06.017 postcondition 1: response must be counted as transaction"
        );
        assert_eq!(
            *analyzer.status_code_counts().get(&200).unwrap_or(&0),
            1,
            "BC-2.06.017 postcondition 1: status 200 must be recorded"
        );
        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            before,
            "BC-2.06.017 postcondition 2: valid response bytes must NOT be counted as skipped"
        );
    }

    /// BC-2.06.017 invariant 1 — request_poisoned only gates ClientToServer data.
    /// After request poisoning, further ClientToServer bytes must be skipped but
    /// ServerToClient bytes must NOT be skipped.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_017_invariant_request_poisoned_gates_only_client_to_server() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Poison request direction with 3 errors.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK\r\n\r\n", 0);
        }
        let skipped_after_poison = analyzer.poisoned_bytes_skipped();
        // Should be 0: nothing was sent post-poison yet.
        assert_eq!(
            skipped_after_poison, 0,
            "precondition: no bytes skipped yet at poison time"
        );

        // Send data on both directions.
        let req_bytes = b"GET / HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, req_bytes, 0);
        // Request direction is poisoned: bytes counted as skipped.
        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            req_bytes.len() as u64,
            "BC-2.06.017 invariant 1: ClientToServer bytes must be counted as skipped"
        );

        let resp_bytes = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        let before_resp = analyzer.poisoned_bytes_skipped();
        analyzer.on_data(&fk, Direction::ServerToClient, resp_bytes, 0);
        // Response direction is NOT poisoned: bytes must NOT be skipped.
        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            before_resp,
            "BC-2.06.017 invariant 1: ServerToClient bytes must NOT be counted as skipped"
        );
        assert_eq!(
            analyzer.transaction_count(),
            1,
            "BC-2.06.017: response must produce a transaction"
        );
    }

    /// BC-2.06.017 EC-003 — response poisoned; request receives valid HTTP — parses normally.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_017_ec003_poisoned_response_does_not_affect_request() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Poison response direction with 3 errors.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ServerToClient, b"GARBAGE\r\n\r\n", 0);
        }

        // Request direction is not poisoned; valid request must parse.
        let valid_req = b"GET /test HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, valid_req, 0);

        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.017 EC-003: request direction must parse normally when only response is poisoned"
        );
    }

    // ── BC-2.06.018 ───────────────────────────────────────────────────────────────
    // non_http_flows Counts Flow Once Even if Both Directions Poisoned

    /// BC-2.06.018 postconditions 1-3 — only request direction poisoned → non_http_flows=1.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_018_only_request_poisoned_counts_one_flow() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        }

        let summary = analyzer.summarize();
        assert_eq!(
            summary.detail["non_http_flows"],
            serde_json::json!(1),
            "BC-2.06.018 EC-001: one request-poisoned flow must contribute non_http_flows=1"
        );
    }

    /// BC-2.06.018 postconditions 1-3 — both directions poisoned → non_http_flows=1, NOT 2.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_018_both_directions_poisoned_counts_one_flow_not_two() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Poison request direction.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        }
        // Poison response direction on the same flow.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ServerToClient, b"GARBAGE\r\n\r\n", 0);
        }

        let summary = analyzer.summarize();
        assert_eq!(
            summary.detail["non_http_flows"],
            serde_json::json!(1),
            "BC-2.06.018 postcondition 3 / EC-002: both directions poisoned must count as 1 flow"
        );
    }

    /// BC-2.06.018 invariant 2 — non_http_flows counts flows, not directions.
    /// Two separate flows each having one direction poisoned → non_http_flows=2.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_018_invariant_two_separate_flows_count_two() {
        let mut analyzer = HttpAnalyzer::new();
        let flow_a = test_flow_key();
        let flow_b = test_flow_key_b();

        // Poison flow A request direction.
        for _ in 0..3 {
            analyzer.on_data(&flow_a, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        }
        // Poison flow B request direction.
        for _ in 0..3 {
            analyzer.on_data(&flow_b, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        }

        let summary = analyzer.summarize();
        assert_eq!(
            summary.detail["non_http_flows"],
            serde_json::json!(2),
            "BC-2.06.018 EC-003 / invariant 2: two separate poisoned flows must count as non_http_flows=2"
        );
    }

    /// BC-2.06.018 invariant 3 — counted_as_non_http latch is checked before incrementing.
    /// The second direction's poisoning does NOT increment non_http_flows because the latch
    /// is already true.  Proven by asserting summarize() shows 1, not 2, after both poison.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_018_invariant_counted_as_non_http_latch_prevents_double_count() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // First: poison response direction.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ServerToClient, b"GARBAGE\r\n\r\n", 0);
        }
        let after_resp = analyzer.summarize();
        assert_eq!(
            after_resp.detail["non_http_flows"],
            serde_json::json!(1),
            "BC-2.06.018: first poisoned direction (response) must set non_http_flows=1"
        );

        // Second: poison request direction on the same flow.
        for _ in 0..3 {
            analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        }
        let after_req = analyzer.summarize();
        assert_eq!(
            after_req.detail["non_http_flows"],
            serde_json::json!(1),
            "BC-2.06.018 invariant 3: counted_as_non_http latch must prevent second poison \
             from incrementing non_http_flows again (still 1, not 2)"
        );
    }

    // ── BC-2.06.020 ───────────────────────────────────────────────────────────────
    // HTTP Body Bytes After Header Completion Do Not Inflate parse_errors

    /// BC-2.06.020 postconditions 1-4 — POST with body: parse_errors=0,
    /// request_error_count not advanced, buf cleared.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_020_post_with_body_does_not_inflate_parse_errors() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Canonical test vector: POST / HTTP/1.1 with JSON body.
        // Header parses completely (had_success=true); JSON body bytes remain in buf
        // and cause Err(Token) on next loop iteration — must be suppressed.
        let req =
            b"POST / HTTP/1.1\r\nHost: x.com\r\nContent-Length: 17\r\n\r\n{\"json\":\"body\"}";
        analyzer.on_data(&fk, Direction::ClientToServer, req, 0);

        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.020 postcondition 1: body bytes after header must NOT inflate parse_errors"
        );
        assert_eq!(
            *analyzer.method_counts().get("POST").unwrap_or(&0),
            1,
            "BC-2.06.020: POST header must be counted"
        );
        assert_eq!(
            analyzer.poisoned_bytes_skipped(),
            0,
            "BC-2.06.020 postcondition 2: no body bytes should poison the direction"
        );
    }

    /// BC-2.06.020 invariant 1 — had_success is local per try_parse_requests call.
    /// Initialized to false; set to true when a complete request is parsed.
    /// Two separate on_data calls each start with had_success=false.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_020_invariant_had_success_is_local_per_call() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // on_data call 1: valid request — had_success=true by end.
        let req1 = b"GET /first HTTP/1.1\r\nHost: x.com\r\n\r\n";
        analyzer.on_data(&fk, Direction::ClientToServer, req1, 0);
        assert_eq!(analyzer.parse_error_count(), 0, "first on_data: no errors");

        // on_data call 2: garbage — had_success starts as false again, error counted.
        analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "BC-2.06.020 invariant 1: had_success is local per call; second on_data starts false"
        );
    }

    /// BC-2.06.020 EC-001 — response with body: body bytes remain in buf after header
    /// parse; had_success suppresses the resulting Err.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_020_response_with_body_does_not_inflate_parse_errors() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Response header + body in one chunk.
        let resp = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 5\r\n\r\nhello";
        analyzer.on_data(&fk, Direction::ServerToClient, resp, 0);

        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.020 EC-001: response body bytes must not inflate parse_errors"
        );
        assert_eq!(
            analyzer.transaction_count(),
            1,
            "BC-2.06.020 EC-001: response transaction must be counted"
        );
    }

    /// BC-2.06.020 invariant 3 — TooManyHeaders check is inside the `if !had_success` block.
    /// A TooManyHeaders on body bytes after a successful header parse must NOT emit a finding.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_020_invariant_too_many_headers_after_success_suppressed() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Build: one valid request header followed immediately by a byte sequence that
        // would, if reparsed, trigger TooManyHeaders.  In practice, body bytes cause
        // Err(Token) not TooManyHeaders, but the invariant is that ALL Err paths are
        // gated by `if !had_success`.  We test this with a response that has body bytes,
        // which exercises the response-side had_success guard at the TooManyHeaders check.
        //
        // To confirm invariant 3 directly: send a valid request, then body bytes with NUL.
        // Since there's no Content-Length tracking, the NUL bytes become Err(Token) which
        // is suppressed by had_success.  The finding count must remain 0.
        let mut req_with_body = b"GET /resource HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec();
        req_with_body.extend_from_slice(b"\x00\x01\x02"); // NUL bytes
        analyzer.on_data(&fk, Direction::ClientToServer, &req_with_body, 0);

        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.020 invariant 3: Err after had_success=true must NOT increment parse_errors"
        );
        assert!(
            analyzer.findings().is_empty(),
            "BC-2.06.020 invariant 3: no finding must be emitted for body-byte Err after success"
        );
    }

    /// BC-2.06.020 invariant 3 (real TooManyHeaders path) — A second request with 97+
    /// headers appended immediately after a first valid request in the SAME on_data call
    /// must NOT produce a TooManyHeaders finding.
    ///
    /// Construction: buf = [valid GET request][second GET with 97 X-Header-N lines].
    /// After the first request parses successfully, had_success=true and its bytes are
    /// drained from request_buf.  The loop continues and tries to parse the second
    /// request; httparse returns Err(TooManyHeaders) (MAX_HEADERS=96, so 97 headers
    /// triggers the limit).  Because had_success==true the `if !had_success` guard at
    /// src/analyzer/http.rs:404 skips both the error-counter increment AND the finding
    /// push at :416.  This is the positive coverage the NUL-byte variant above cannot
    /// provide: it exercises the actual TooManyHeaders branch inside the guard.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_020_invariant_real_too_many_headers_after_success_suppressed() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Request 1: syntactically valid — will parse completely and set had_success=true.
        let mut buf = b"GET /first HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec();

        // Request 2: 97 headers — exceeds MAX_HEADERS (96) and causes Err(TooManyHeaders).
        // Appended in the same buffer so the loop encounters it after draining request 1.
        buf.extend_from_slice(b"GET /second HTTP/1.1\r\n");
        for i in 0..97 {
            buf.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
        }
        buf.extend_from_slice(b"\r\n");

        analyzer.on_data(&fk, Direction::ClientToServer, &buf, 0);

        // Request 1 must be counted.
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.020 invariant 3 (real TMH): first valid request must be counted"
        );

        // The TooManyHeaders error on request 2 must be suppressed by had_success.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.020 invariant 3 (real TMH): had_success guard must suppress \
             parse_errors for TooManyHeaders after first success"
        );

        // No finding must be emitted — the guard at :416 is inside `if !had_success`.
        let all_findings = analyzer.findings();
        let tmh_findings: Vec<_> = all_findings
            .iter()
            .filter(|f| f.summary.contains("Excessive HTTP headers"))
            .collect();
        assert!(
            tmh_findings.is_empty(),
            "BC-2.06.020 invariant 3 (real TMH): TooManyHeaders finding MUST NOT be \
             emitted when had_success=true — guard at src/analyzer/http.rs:416 must gate \
             the finding push; got {} finding(s)",
            tmh_findings.len()
        );
    }

    /// BC-2.06.020 invariant 3 (real TooManyHeaders — RESPONSE arm) — Symmetric sibling
    /// of `test_BC_2_06_020_invariant_real_too_many_headers_after_success_suppressed`.
    ///
    /// BC-2.06.020 invariant 3 applies to BOTH arms of the parse loop.  The response-side
    /// guard is at `src/analyzer/http.rs:462` wrapping the TooManyHeaders finding push at
    /// ~475-487.  This test exercises that arm directly:
    ///
    /// Construction (ServerToClient direction):
    ///   buf = [valid HTTP/1.1 200 response with complete headers + body]
    ///         [second response with 97+ X-Header-N lines → Err(TooManyHeaders)]
    ///
    /// After the first response parses, `had_success = true` and `transactions` becomes 1.
    /// The loop continues, encounters the second (too-many-headers) response, gets
    /// `Err(TooManyHeaders)`, but the `if !had_success` guard at :462 prevents both the
    /// `parse_errors` increment and the finding push at :475.
    ///
    /// Assertions:
    ///   - `transaction_count() == 1`  (first response counted; second not reached)
    ///   - `parse_error_count() == 0`  (guard suppressed the error increment)
    ///   - No "Excessive HTTP headers" finding emitted  (guard suppressed the push)
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_020_invariant_real_too_many_headers_after_success_suppressed_response() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // Response 1: syntactically valid — will parse completely and set had_success=true.
        // Content-Length: 0 so there are no stray body bytes; the entire first response
        // fits cleanly and is drained before the loop re-iterates.
        let mut buf = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".to_vec();

        // Response 2: 97 X-Header-N lines — exceeds MAX_HEADERS (96) and causes
        // Err(TooManyHeaders) on the RESPONSE parse path.  Appended in the same buffer
        // so the loop encounters it after draining response 1.
        buf.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
        for i in 0..97 {
            buf.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
        }
        buf.extend_from_slice(b"\r\n");

        analyzer.on_data(&fk, Direction::ServerToClient, &buf, 0);

        // Response 1 must be counted as a transaction (response-side success counter).
        assert_eq!(
            analyzer.transaction_count(),
            1,
            "BC-2.06.020 invariant 3 (resp TMH): first valid response must increment transactions"
        );

        // The TooManyHeaders error on response 2 must be suppressed by had_success.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "BC-2.06.020 invariant 3 (resp TMH): had_success guard on response arm must suppress \
             parse_errors for TooManyHeaders after first success (guard at src/analyzer/http.rs:462)"
        );

        // No finding must be emitted — the push at ~:475 is inside `if !had_success`.
        let all_findings = analyzer.findings();
        let tmh_findings: Vec<_> = all_findings
            .iter()
            .filter(|f| f.summary.contains("Excessive HTTP headers"))
            .collect();
        assert!(
            tmh_findings.is_empty(),
            "BC-2.06.020 invariant 3 (resp TMH): TooManyHeaders finding MUST NOT be emitted when \
             had_success=true on the response arm — guard at src/analyzer/http.rs:462 must gate \
             the finding push at ~:475; got {} finding(s)",
            tmh_findings.len()
        );
    }

    /// BC-2.06.020 EC-002 — 2 error buffers before a valid header, then body.
    /// The two pre-success errors ARE counted (parse_errors=2), but the body error
    /// after the successful header is NOT counted.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_06_020_pre_success_errors_counted_body_errors_not() {
        let mut analyzer = HttpAnalyzer::new();
        let fk = test_flow_key();

        // 2 error buffers.
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK1\r\n\r\n", 0);
        analyzer.on_data(&fk, Direction::ClientToServer, b"JUNK2\r\n\r\n", 0);
        assert_eq!(
            analyzer.parse_error_count(),
            2,
            "BC-2.06.020 EC-002: 2 errors before success must be counted"
        );

        // Valid header + body in same on_data.
        let mut req = b"GET /valid HTTP/1.1\r\nHost: x.com\r\n\r\n".to_vec();
        req.push(0x00); // body byte → Err on next iteration (suppressed)
        analyzer.on_data(&fk, Direction::ClientToServer, &req, 0);

        assert_eq!(
            analyzer.parse_error_count(),
            2,
            "BC-2.06.020 EC-002: body byte error after success must NOT add to parse_errors; stays at 2"
        );
        assert_eq!(
            *analyzer.method_counts().get("GET").unwrap_or(&0),
            1,
            "BC-2.06.020 EC-002: GET must be counted from the successful header"
        );
    }
} // mod bc_2_06_044_formalization
