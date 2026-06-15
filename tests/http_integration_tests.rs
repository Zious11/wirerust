use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::decoder::{DecodedFrame, decode_packet};
use wirerust::reader::PcapSource;
use wirerust::reassembly::handler::StreamAnalyzer;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

#[test]
fn test_http_analysis_with_fixture() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/http-full.cap")).unwrap();

    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut http_analyzer = HttpAnalyzer::new();

    for raw in &source.packets {
        if let Ok(DecodedFrame::Ip(parsed)) = decode_packet(&raw.data, source.datalink) {
            reassembler.process_packet(&parsed, raw.timestamp_secs, &mut http_analyzer);
        }
    }
    reassembler.finalize(&mut http_analyzer);

    let summary = http_analyzer.summarize();
    assert_eq!(summary.analyzer_name, "HTTP");

    assert!(
        http_analyzer.transaction_count() > 0,
        "Expected HTTP transactions from http-full.cap, got 0"
    );

    assert!(
        http_analyzer.method_counts().contains_key("GET"),
        "Expected GET requests in http-full.cap"
    );
}
