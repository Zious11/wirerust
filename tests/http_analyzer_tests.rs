use wirerust::analyzer::http::HttpAnalyzer;

#[test]
fn test_http_analyzer_construction() {
    let analyzer = HttpAnalyzer::new();
    assert_eq!(analyzer.transaction_count(), 0);
}
