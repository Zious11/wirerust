//! Multi-analyzer end-to-end smoke test (CR-011).
//!
//! Exercises HTTP + TLS + DNS + TCP reassembly + reporter in a single synthetic
//! packet stream, asserting on the final aggregated reporter output.
//!
//! Design principles:
//! - Uses only the crate's legitimate public API (no `_for_testing` symbols).
//! - Reuses patterns from `dispatcher_tests.rs`, `tls_analyzer_tests.rs`,
//!   `http_integration_tests.rs`, and `dns_tests.rs`.
//! - Mirrors the `run_analyze` pipeline from `src/main.rs`:
//!   - `summary.ingest()` on every decoded packet, in the same position as production.
//!   - ParsedPacket → DnsAnalyzer (packet-level, UDP port 53)
//!   - ParsedPacket → TcpReassembler → StreamDispatcher → HttpAnalyzer / TlsAnalyzer
//!   - All analyzer summaries are then collected and rendered by JsonReporter.
//! - Reassembly is exercised by splitting the HTTP request across two TCP
//!   segments with consecutive sequence numbers.
//! - All packets use compact monotonic timestamps (1, 2, 3, …) to guarantee
//!   they stay within the 300-second `flow_timeout_secs` default, so flows
//!   close via FIN (not idle-timeout).
//! - CR-001 accessors (`http_analyzer()`, `tls_analyzer()`, `take_tls_analyzer()`)
//!   are used; `take_tls_analyzer()` exercises the ownership-transfer path while
//!   `http_analyzer()` / `tls_analyzer()` exercise the immutable-borrow path.

use std::net::{IpAddr, Ipv4Addr};

use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reassembly::handler::StreamAnalyzer;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};
use wirerust::reporter::Reporter;
use wirerust::reporter::json::JsonReporter;
use wirerust::summary::Summary;

// ---------------------------------------------------------------------------
// Packet builders (reusing patterns from tls_analyzer_tests.rs / dns_tests.rs)
// ---------------------------------------------------------------------------

const CLIENT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
const SERVER_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

/// Build a TCP ParsedPacket for use with TcpReassembler.
///
/// `payload` is the application-layer bytes. `seq_number` is the TCP sequence
/// number so the reassembler can order segments. `syn` / `ack` / `fin` / `rst`
/// control the TCP flag bits.
#[allow(clippy::too_many_arguments)]
fn make_tcp_packet(
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: u16,
    dst_port: u16,
    seq_number: u32,
    syn: bool,
    ack: bool,
    fin: bool,
    rst: bool,
    payload: Vec<u8>,
) -> ParsedPacket {
    let packet_len = payload.len() + 40; // approximate (IP + TCP headers)
    ParsedPacket {
        src_ip,
        dst_ip,
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port,
            dst_port,
            seq_number,
            syn,
            ack,
            fin,
            rst,
        },
        payload,
        packet_len,
    }
}

/// Build a UDP ParsedPacket (for DNS, which is handled at the packet level).
fn make_udp_packet(
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: u16,
    dst_port: u16,
    payload: Vec<u8>,
) -> ParsedPacket {
    let packet_len = payload.len() + 28; // approximate (IP + UDP headers)
    ParsedPacket {
        src_ip,
        dst_ip,
        protocol: Protocol::Udp,
        transport: TransportInfo::Udp { src_port, dst_port },
        payload,
        packet_len,
    }
}

/// Build a minimal 12-byte DNS query payload (QR=0: byte[2] bit-7 == 0).
/// Sufficient for DnsAnalyzer.analyze() to classify it as a query.
fn dns_query_payload() -> Vec<u8> {
    let mut p = vec![0u8; 12];
    p[0] = 0xAB; // transaction ID high
    p[1] = 0xCD; // transaction ID low
    p[2] = 0x01; // QR=0 (query), RD=1 (recursion desired)
    p[3] = 0x00; // RA=0, Z=0, RCODE=0
    p[4] = 0x00; // QDCOUNT high
    p[5] = 0x01; // QDCOUNT low = 1 question
    // Remaining bytes (ANCOUNT, NSCOUNT, ARCOUNT) are zero.
    p
}

/// Build a minimal TLS ClientHello record with the given SNI.
///
/// This is a self-contained copy of the builder pattern from
/// `tls_analyzer_tests.rs`, inlined here so `multi_analyzer_e2e_tests.rs`
/// has no inter-test-file dependency.
fn build_tls_client_hello(sni: &str) -> Vec<u8> {
    let sni_bytes = sni.as_bytes();

    // SNI extension (type 0x0000)
    let mut sni_list_data = Vec::new();
    sni_list_data.push(0x00u8); // NameType: host_name
    let name_len = u16::try_from(sni_bytes.len()).expect("SNI too long");
    sni_list_data.extend_from_slice(&name_len.to_be_bytes());
    sni_list_data.extend_from_slice(sni_bytes);
    let sni_list_len = u16::try_from(sni_list_data.len()).expect("SNI list too long");
    let sni_ext_len = sni_list_len.checked_add(2).expect("SNI ext too long");

    let mut extensions = Vec::new();
    extensions.extend_from_slice(&[0x00, 0x00]); // extension type: server_name
    extensions.extend_from_slice(&sni_ext_len.to_be_bytes());
    extensions.extend_from_slice(&sni_list_len.to_be_bytes());
    extensions.extend_from_slice(&sni_list_data);

    // Supported Groups extension
    extensions.extend_from_slice(&[0x00, 0x0a, 0x00, 0x06, 0x00, 0x04, 0x00, 0x1d, 0x00, 0x17]);
    // EC Point Formats extension
    extensions.extend_from_slice(&[0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);

    // ClientHello body
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&[0x03, 0x03]); // version TLS 1.2
    ch_body.extend_from_slice(&[0u8; 32]); // random
    ch_body.push(0x00); // session_id length: 0
    // Two cipher suites: TLS_AES_128_GCM_SHA256 (0x1301) + TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 (0xc02b)
    ch_body.extend_from_slice(&[0x00, 0x04]); // cipher suites length = 4 bytes (2 suites)
    ch_body.extend_from_slice(&[0x13, 0x01]); // TLS_AES_128_GCM_SHA256
    ch_body.extend_from_slice(&[0xc0, 0x2b]); // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression
    let ext_len = u16::try_from(extensions.len()).expect("extensions too long");
    ch_body.extend_from_slice(&ext_len.to_be_bytes());
    ch_body.extend_from_slice(&extensions);

    // Handshake header
    let mut handshake = Vec::new();
    handshake.push(0x01); // ClientHello
    let ch_len = ch_body.len() as u32;
    handshake.push((ch_len >> 16) as u8);
    handshake.push((ch_len >> 8) as u8);
    handshake.push(ch_len as u8);
    handshake.extend_from_slice(&ch_body);

    // TLS record header
    let mut record = Vec::new();
    record.push(0x16); // content type: handshake
    record.extend_from_slice(&[0x03, 0x01]); // record version: TLS 1.0
    let hs_len = u16::try_from(handshake.len()).expect("handshake too long");
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

// ---------------------------------------------------------------------------
// The multi-analyzer end-to-end smoke test (CR-011)
// ---------------------------------------------------------------------------

/// CR-011: Multi-analyzer end-to-end smoke test.
///
/// Feeds a single synthetic packet stream through the full pipeline:
///
///   HTTP analyzer:
///     TCP flow (client:49100 → server:80), HTTP GET split across TWO segments
///     to exercise reassembly. Each segment is individually handed to the
///     reassembler; the reassembler flushes each contiguous prefix immediately
///     to HttpAnalyzer (which accumulates bytes until httparse returns Complete).
///     Asserts: reporter JSON shows exactly 1 GET in HTTP analyzer summary.
///
///   TLS analyzer:
///     TCP flow (client:49101 → server:443), one TLS ClientHello record with
///     SNI "e2e.smoke.test.local" sent in a single segment.
///     Asserts: reporter JSON shows the exact SNI in TLS analyzer summary.
///
///   DNS analyzer:
///     UDP packet (client:12345 → server:53), minimal 12-byte DNS query (QR=0).
///     Asserts: reporter JSON shows dns_queries == 1.
///
///   Reporter:
///     All analyzer summaries collected and rendered via JsonReporter.
///     All assertions target the final JSON output — the public output surface.
///
/// Timing: all packets use compact monotonic timestamps (1, 2, 3, …) so all
/// flows stay within the 300-second idle-timeout window of ReassemblyConfig::default().
/// Both flows close via FIN (not via idle-timeout).
///
/// This test uses only the crate's public API:
///   - `dispatcher.http_analyzer()` / `dispatcher.tls_analyzer()`: immutable-borrow accessors
///     (CR-001) used to collect findings and summaries.
///   - `dispatcher.take_tls_analyzer()`: ownership-transfer accessor (CR-001) exercised
///     to verify the take path compiles and works end-to-end.
///   - No `_for_testing` symbols added to `src/`.
#[test]
fn test_cr011_multi_analyzer_http_tls_dns_reassembly_reporter_e2e() {
    // -----------------------------------------------------------------------
    // Pipeline setup — mirrors run_analyze() in src/main.rs
    // -----------------------------------------------------------------------
    let mut summary = Summary::new();
    let mut all_findings = Vec::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut dispatcher =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()), None);

    // Compact monotonic clock: all packets within a single 300-second window so
    // the 5-minute idle-timeout (ReassemblyConfig::default().flow_timeout_secs = 300)
    // never fires. Flows close via FIN, not via CloseReason::Timeout.
    let mut ts: u32 = 1;
    let mut next_ts = || {
        let t = ts;
        ts += 1;
        t
    };

    // -----------------------------------------------------------------------
    // Flow 1: HTTP — TCP client:49100 → server:80
    // The GET request is split across two TCP segments to exercise reassembly.
    //
    // Segment A: "GET /smoke-test HTTP/1.1\r\nHost: "   (incomplete — no CRLF pair)
    // Segment B: "e2e.example.com\r\n\r\n"              (completes the message)
    //
    // The reassembler delivers each contiguous prefix to HttpAnalyzer immediately;
    // httparse returns Partial on segment A and Complete on segment B, at which
    // point HttpAnalyzer records the transaction and method count.
    // The flow is closed by the client-side FIN below (CloseReason::Fin).
    // -----------------------------------------------------------------------
    const HTTP_SRC_PORT: u16 = 49100;
    const HTTP_DST_PORT: u16 = 80;

    let seg_a: Vec<u8> = b"GET /smoke-test HTTP/1.1\r\nHost: ".to_vec();
    let seg_b: Vec<u8> = b"e2e.example.com\r\n\r\n".to_vec();
    let seg_a_len = seg_a.len() as u32;
    let seg_b_len = seg_b.len() as u32;

    // SYN — opens the flow in the reassembler
    let syn_pkt = make_tcp_packet(
        CLIENT_IP,
        SERVER_IP,
        HTTP_SRC_PORT,
        HTTP_DST_PORT,
        /*seq=*/ 999,
        /*syn=*/ true,
        /*ack=*/ false,
        /*fin=*/ false,
        /*rst=*/ false,
        vec![],
    );
    summary.ingest(&syn_pkt);
    reassembler.process_packet(&syn_pkt, next_ts(), &mut dispatcher);

    // SYN-ACK — server acknowledges
    let syn_ack_pkt = make_tcp_packet(
        SERVER_IP,
        CLIENT_IP,
        HTTP_DST_PORT,
        HTTP_SRC_PORT,
        /*seq=*/ 1999,
        /*syn=*/ true,
        /*ack=*/ true,
        /*fin=*/ false,
        /*rst=*/ false,
        vec![],
    );
    summary.ingest(&syn_ack_pkt);
    reassembler.process_packet(&syn_ack_pkt, next_ts(), &mut dispatcher);

    // Segment A: first half of the HTTP request (seq = ISN + 1 post-SYN)
    let seg_a_pkt = make_tcp_packet(
        CLIENT_IP,
        SERVER_IP,
        HTTP_SRC_PORT,
        HTTP_DST_PORT,
        /*seq=*/ 1000,
        /*syn=*/ false,
        /*ack=*/ true,
        /*fin=*/ false,
        /*rst=*/ false,
        seg_a,
    );
    summary.ingest(&seg_a_pkt);
    reassembler.process_packet(&seg_a_pkt, next_ts(), &mut dispatcher);

    // Segment B: second half of the HTTP request (seq advances past segment A)
    let seg_b_pkt = make_tcp_packet(
        CLIENT_IP,
        SERVER_IP,
        HTTP_SRC_PORT,
        HTTP_DST_PORT,
        /*seq=*/ 1000 + seg_a_len,
        /*syn=*/ false,
        /*ack=*/ true,
        /*fin=*/ false,
        /*rst=*/ false,
        seg_b,
    );
    summary.ingest(&seg_b_pkt);
    reassembler.process_packet(&seg_b_pkt, next_ts(), &mut dispatcher);

    // FIN — close the HTTP flow via CloseReason::Fin (not idle-timeout)
    let fin_pkt = make_tcp_packet(
        CLIENT_IP,
        SERVER_IP,
        HTTP_SRC_PORT,
        HTTP_DST_PORT,
        /*seq=*/ 1000 + seg_a_len + seg_b_len,
        /*syn=*/ false,
        /*ack=*/ true,
        /*fin=*/ true,
        /*rst=*/ false,
        vec![],
    );
    summary.ingest(&fin_pkt);
    reassembler.process_packet(&fin_pkt, next_ts(), &mut dispatcher);

    // -----------------------------------------------------------------------
    // Flow 2: TLS — TCP client:49101 → server:443
    // A single TLS ClientHello record with SNI "e2e.smoke.test.local".
    // -----------------------------------------------------------------------
    const SNI: &str = "e2e.smoke.test.local";
    const TLS_SRC_PORT: u16 = 49101;
    const TLS_DST_PORT: u16 = 443;

    let tls_hello = build_tls_client_hello(SNI);
    let tls_hello_len = tls_hello.len() as u32;

    // SYN for TLS flow
    let tls_syn = make_tcp_packet(
        CLIENT_IP,
        SERVER_IP,
        TLS_SRC_PORT,
        TLS_DST_PORT,
        /*seq=*/ 4999,
        /*syn=*/ true,
        /*ack=*/ false,
        /*fin=*/ false,
        /*rst=*/ false,
        vec![],
    );
    summary.ingest(&tls_syn);
    reassembler.process_packet(&tls_syn, next_ts(), &mut dispatcher);

    // TLS ClientHello — one complete segment
    let tls_data_pkt = make_tcp_packet(
        CLIENT_IP,
        SERVER_IP,
        TLS_SRC_PORT,
        TLS_DST_PORT,
        /*seq=*/ 5000,
        /*syn=*/ false,
        /*ack=*/ true,
        /*fin=*/ false,
        /*rst=*/ false,
        tls_hello,
    );
    summary.ingest(&tls_data_pkt);
    reassembler.process_packet(&tls_data_pkt, next_ts(), &mut dispatcher);

    // FIN for TLS flow — closes via CloseReason::Fin (not idle-timeout)
    let tls_fin = make_tcp_packet(
        CLIENT_IP,
        SERVER_IP,
        TLS_SRC_PORT,
        TLS_DST_PORT,
        /*seq=*/ 5000 + tls_hello_len,
        /*syn=*/ false,
        /*ack=*/ true,
        /*fin=*/ true,
        /*rst=*/ false,
        vec![],
    );
    summary.ingest(&tls_fin);
    reassembler.process_packet(&tls_fin, next_ts(), &mut dispatcher);

    // -----------------------------------------------------------------------
    // DNS: UDP client:12345 → server:53
    // Handled at the packet level (not via the TCP reassembler).
    // DnsAnalyzer never emits findings (BC-2.08.004).
    // -----------------------------------------------------------------------
    let dns_pkt = make_udp_packet(
        CLIENT_IP,
        SERVER_IP,
        /*src=*/ 12345,
        /*dst=*/ 53,
        dns_query_payload(),
    );
    summary.ingest(&dns_pkt);
    assert!(
        dns_analyzer.can_decode(&dns_pkt),
        "CR-011 setup: DnsAnalyzer must accept UDP dst=53 packet"
    );
    let dns_findings = dns_analyzer.analyze(&dns_pkt);
    assert!(
        dns_findings.is_empty(),
        "CR-011 setup: DnsAnalyzer must produce no findings (BC-2.08.004)"
    );
    // Extend all_findings with DNS findings immediately, mirroring run_analyze() in
    // src/main.rs which calls `all_findings.extend(findings)` after the can_decode gate.
    // This is a no-op today (guaranteed empty by the assert above), but keeps the test
    // pipeline isomorphic with production so a future DNS finding would not be silently
    // dropped from the reporter's input.
    all_findings.extend(dns_findings);

    // -----------------------------------------------------------------------
    // Finalize reassembler — flushes any buffered partial streams
    // -----------------------------------------------------------------------
    reassembler.finalize(&mut dispatcher);

    // -----------------------------------------------------------------------
    // Sanity: the benign stream must not produce reassembly anomaly findings.
    // Assert before collecting all_findings so a false-positive here is clearly
    // identified as a reassembly interaction issue (CR-006).
    // -----------------------------------------------------------------------
    assert!(
        reassembler.findings().is_empty(),
        "CR-011: benign stream must produce zero reassembly anomaly findings; \
         got: {:?}",
        reassembler.findings()
    );

    // -----------------------------------------------------------------------
    // Collect findings + analyzer summaries — mirrors run_analyze() in main.rs
    // -----------------------------------------------------------------------
    // DNS findings were already extended above (at the packet-processing site).
    all_findings.extend(reassembler.findings().to_vec());

    // HTTP findings via immutable-borrow accessor (CR-001)
    if let Some(http) = dispatcher.http_analyzer() {
        all_findings.extend(http.findings());
    }
    // TLS findings via immutable-borrow accessor (CR-001)
    if let Some(tls) = dispatcher.tls_analyzer() {
        all_findings.extend(tls.findings());
    }

    // Analyzer summaries in the same order as run_analyze()
    let mut analyzer_summaries = Vec::new();
    {
        let mut reasm_summary = reassembler.summarize();
        reasm_summary.detail.insert(
            "unclassified_flows".to_string(),
            serde_json::json!(dispatcher.unclassified_flows()),
        );
        analyzer_summaries.push(reasm_summary);
    }
    analyzer_summaries.push(dns_analyzer.summarize());
    if let Some(http) = dispatcher.http_analyzer() {
        analyzer_summaries.push(http.summarize());
    }
    // Use take_tls_analyzer() (CR-001 ownership-transfer accessor) to exercise
    // the take path; this moves the TLS analyzer out of the dispatcher.
    if let Some(tls) = dispatcher.take_tls_analyzer() {
        analyzer_summaries.push(tls.summarize());
    }

    // -----------------------------------------------------------------------
    // Render via JsonReporter — the public output surface
    // -----------------------------------------------------------------------
    let reporter = JsonReporter;
    let json_output = reporter.render(&summary, &all_findings, &analyzer_summaries);

    // JSON must be valid
    let parsed: serde_json::Value =
        serde_json::from_str(&json_output).expect("CR-011: reporter must produce valid JSON");

    // -----------------------------------------------------------------------
    // Assertions on the aggregated JSON output
    // -----------------------------------------------------------------------

    // Locate each analyzer's summary block by name.
    // JsonReporter emits the analyzer summaries under the key "analyzers"
    // (see src/reporter/json.rs).
    let analyzer_arr = parsed["analyzers"]
        .as_array()
        .expect("CR-011: 'analyzers' must be a JSON array in reporter output");

    let find_summary = |name: &str| -> &serde_json::Value {
        analyzer_arr
            .iter()
            .find(|a| a["analyzer"].as_str() == Some(name))
            .unwrap_or_else(|| panic!("CR-011: missing analyzer summary for '{name}'"))
    };

    // --- DNS assertion ---
    // DnsAnalyzer must have seen exactly 1 query (the single packet we fed it).
    let dns_summary = find_summary("DNS");
    assert_eq!(
        dns_summary["detail"]["dns_queries"],
        serde_json::json!(1u64),
        "CR-011 DNS: dns_queries must be 1 after one UDP DNS query packet"
    );
    assert_eq!(
        dns_summary["detail"]["dns_responses"],
        serde_json::json!(0u64),
        "CR-011 DNS: dns_responses must be 0 (only a query was sent)"
    );

    // --- HTTP assertion ---
    // HttpAnalyzer must have seen exactly one GET from the reassembled request.
    // detail["methods"] is a JSON object mapping method name → count.
    let http_summary = find_summary("HTTP");
    let methods_obj = http_summary["detail"]["methods"]
        .as_object()
        .expect("CR-011 HTTP: 'methods' must be a JSON object in HTTP summary");
    let get_count = methods_obj.get("GET").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(
        get_count, 1,
        "CR-011 HTTP: methods['GET'] must be exactly 1 after reassembly of the split \
         HTTP request (>= 1 would mask double-count regressions); got methods: {methods_obj:?}"
    );

    // --- TLS assertion ---
    // TlsAnalyzer must have extracted the exact SNI from the ClientHello.
    // top_snis entries are plain hostname strings — assert exact equality,
    // not substring match, to catch wrong-SNI bugs.
    let tls_summary = find_summary("TLS");
    let top_snis = tls_summary["detail"]["top_snis"]
        .as_array()
        .expect("CR-011 TLS: 'top_snis' must be a JSON array in TLS summary");
    let saw_sni = top_snis
        .iter()
        .any(|entry| entry.as_str().map(|s| s == SNI).unwrap_or(false));
    assert!(
        saw_sni,
        "CR-011 TLS: top_snis must contain the exact SNI '{SNI}' after processing ClientHello; \
         got: {top_snis:?}"
    );

    // --- Cross-cutting: all four analyzer names appear in the output ---
    let analyzer_names: Vec<&str> = analyzer_arr
        .iter()
        .filter_map(|a| a["analyzer"].as_str())
        .collect();
    assert!(
        analyzer_names.contains(&"DNS"),
        "CR-011: 'DNS' analyzer must appear in analyzers; got: {analyzer_names:?}"
    );
    assert!(
        analyzer_names.contains(&"HTTP"),
        "CR-011: 'HTTP' analyzer must appear in analyzers; got: {analyzer_names:?}"
    );
    assert!(
        analyzer_names.contains(&"TLS"),
        "CR-011: 'TLS' analyzer must appear in analyzers; got: {analyzer_names:?}"
    );
    // Reassembly summary is also present (identified by the unclassified_flows key
    // inserted by the test — same as run_analyze() in src/main.rs).
    let has_reasm = analyzer_arr.iter().any(|a| {
        a["detail"]
            .as_object()
            .map(|d| d.contains_key("unclassified_flows"))
            .unwrap_or(false)
    });
    assert!(
        has_reasm,
        "CR-011: reassembly summary with 'unclassified_flows' key must appear in output"
    );

    // --- Reassembly liveness: bytes_reassembled > 0 ---
    // Proves the TCP segments were actually handed to the reassembler and
    // not silently dropped (e.g. by timestamp-triggered idle-timeout).
    let reasm_entry = analyzer_arr
        .iter()
        .find(|a| {
            a["detail"]
                .as_object()
                .map(|d| d.contains_key("unclassified_flows"))
                .unwrap_or(false)
        })
        .expect("CR-011: reassembly entry must exist in analyzers array");
    let bytes_reasm = reasm_entry["detail"]["bytes_reassembled"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        bytes_reasm > 0,
        "CR-011: bytes_reassembled must be > 0 (HTTP and TLS TCP segments must reach \
         the reassembler and not be silently dropped by idle-timeout)"
    );
}
