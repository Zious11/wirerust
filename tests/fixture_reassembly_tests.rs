//! Reassembly-engine smoke tests over the Wireshark-wiki TCP fixtures
//! added toward the P2.05 calibration corpus.
//!
//! Both captures are reassembly-heavy *benign baselines*: each drives
//! 100+ segments and 80 KB+ of reassembled stream data through the
//! engine while producing ZERO anomaly findings. That is the useful
//! property for threshold calibration — it confirms the overlap /
//! small-segment / out-of-window thresholds do not false-positive on
//! normal high-volume TCP traffic.
//!
//! Fixture provenance and licensing: see `tests/fixtures/README.md`.

use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

/// Drive every decodable packet of a fixture through the reassembly
/// engine and return the finalized reassembler for assertions.
fn reassemble_fixture(path: &str) -> TcpReassembler {
    let source = PcapSource::from_file(std::path::Path::new(path))
        .unwrap_or_else(|e| panic!("failed to load fixture {path}: {e}"));
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    // `HttpAnalyzer` doubles as a `StreamHandler` sink here; these
    // tests assert on the reassembler's own stats, not analyzer output.
    let mut sink = HttpAnalyzer::new();
    for raw in &source.packets {
        if let Ok(parsed) = decode_packet(&raw.data, source.datalink) {
            reassembler.process_packet(&parsed, raw.timestamp_secs, &mut sink);
        }
    }
    reassembler.finalize(&mut sink);
    reassembler
}

#[test]
fn test_tcp_ecn_sample_reassembles_as_benign_baseline() {
    let reasm = reassemble_fixture("tests/fixtures/tcp-ecn-sample.pcap");
    let s = reasm.stats();
    assert!(reasm.is_finalized());
    assert_eq!(
        s.packets_tcp, 479,
        "tcp-ecn-sample.pcap has 479 TCP packets"
    );
    assert_eq!(s.flows_total, 2, "expected 2 flows");
    assert!(
        s.bytes_reassembled > 0,
        "expected reassembled stream bytes, got 0"
    );
    assert!(s.segments_inserted > 0, "expected inserted segments");
    // Calibration baseline: a normal HTTP/ECN transfer must NOT trip
    // any reassembly anomaly threshold (overlap / small-seg / OOW).
    assert_eq!(
        reasm.findings().len(),
        0,
        "benign capture must produce no reassembly anomaly findings; got {:?}",
        reasm.findings()
    );
}

#[test]
fn test_tcp_ethereal_file1_reassembles_as_benign_baseline() {
    let reasm = reassemble_fixture("tests/fixtures/tcp-ethereal-file1.trace");
    let s = reasm.stats();
    assert!(reasm.is_finalized());
    assert_eq!(
        s.packets_tcp, 218,
        "tcp-ethereal-file1.trace has 218 TCP packets"
    );
    assert_eq!(s.flows_total, 1, "expected a single flow");
    assert!(
        s.bytes_reassembled > 100_000,
        "tcp-ethereal-file1 is a large multi-segment transfer; expected >100 KB reassembled, got {}",
        s.bytes_reassembled
    );
    // Calibration baseline: a large benign transfer must NOT trip any
    // reassembly anomaly threshold.
    assert_eq!(
        reasm.findings().len(),
        0,
        "benign capture must produce no reassembly anomaly findings; got {:?}",
        reasm.findings()
    );
}
