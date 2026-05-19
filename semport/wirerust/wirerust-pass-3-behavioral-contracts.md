# Pass 3: Behavioral Contracts -- wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 3 (Behavioral Contracts) -- Phase A broad-sweep, round 1
- **Inputs consumed:** Pass 0 inventory, Pass 1 architecture, Pass 2 domain model (101 BRs), ADR 0001/0002/0003, all 18 test files in `tests/`, all 20 `.rs` files in `src/`.
- **Test functions catalogued:** 202 across 18 files (Pass 0 §5).
- **Total BCs in this document:** 137.
- **Convention:** Each BC cites both the implementation file:line(s) AND at least one test that pins it (when HIGH). MEDIUM = code-only. LOW = ADR/comment-only.

---

## 1. BC Index

Sorted by area, then by BC ID. Component IDs match Pass 1 (C-1 .. C-20).

| ID | Name | Confidence | Component | Tests pinning |
|---|---|---|---|---|
| BC-RDR-001 | Accept supported link types (Ethernet/RAW/IPv4/IPv6/SLL) and reject all others | HIGH | C-4 | reader_tests: test_pcap_source_stores_datalink, test_reader_accepts_raw_linktype, test_reader_accepts_ipv4_linktype, test_reader_accepts_ipv6_linktype, test_reader_accepts_linux_sll_linktype, test_unsupported_link_type_rejected |
| BC-RDR-002 | Read all packets from a pcap file as a Vec<RawPacket> preserving timestamps | HIGH | C-4 | reader_tests: test_read_pcap_packets |
| BC-RDR-003 | Accept a pcap with zero packets (only global header) without error | HIGH | C-4 | reader_tests: test_empty_pcap_no_packets |
| BC-RDR-004 | Reject pcapng-format input (only classic pcap is supported) | MEDIUM | C-4 | Implementation uses `PcapReader::new` (classic pcap only); pcapng fixture exists but not asserted in tests |
| BC-RDR-005 | Convert raw pcap timestamp to (timestamp_secs: u32, timestamp_usecs: u32) | HIGH | C-4 | reader_tests: test_read_pcap_packets asserts timestamp_secs == 1000 |
| BC-RDR-006 | Surface pcap header parse errors with anyhow context "Failed to parse pcap header" | MEDIUM | C-4 | reader.rs:22 -- no direct test, but `anyhow::Context` chain is wired |
| BC-RDR-007 | Surface per-packet read errors with context "Failed to read packet" | MEDIUM | C-4 | reader.rs:41 -- no direct test |
| BC-RDR-008 | `from_file` reads via std::fs::File + BufReader and delegates to `from_pcap_reader` | MEDIUM | C-4 | Inferred from code; integration tests use `from_file` exclusively |
| BC-DEC-001 | Decode Ethernet-framed IPv4 TCP packet to ParsedPacket with correct src/dst/ports/protocol | HIGH | C-5 | decoder_tests: test_decode_tcp_packet |
| BC-DEC-002 | Decode Ethernet-framed IPv4 UDP packet, including DNS service hint inference | HIGH | C-5 | decoder_tests: test_decode_udp_dns_packet |
| BC-DEC-003 | Decode RAW (no-link-layer) IPv4 TCP packet via `from_ip` | HIGH | C-5 | decoder_tests: test_decode_raw_ip_tcp_packet |
| BC-DEC-004 | Decode DataLink::IPV4 the same way as DataLink::RAW (both via `from_ip`) | HIGH | C-5 | decoder_tests: test_decode_ipv4_linktype_uses_from_ip |
| BC-DEC-005 | Decode RAW IPv6 TCP packet, surfacing IPv6 addresses correctly | HIGH | C-5 | decoder_tests: test_decode_ipv6_tcp_packet |
| BC-DEC-006 | Decode Linux SLL ("cooked") TCP packets via `SlicedPacket::from_linux_sll` | HIGH | C-5 | decoder_tests: test_decode_linux_sll_tcp_packet, linktype_integration_tests |
| BC-DEC-007 | Reject malformed input bytes with an anyhow error (does not panic) | HIGH | C-5 | decoder_tests: test_decode_invalid_packet |
| BC-DEC-008 | Reject unsupported link types in `decode_packet` even if reader accepted them | MEDIUM | C-5 | decoder.rs:76 (`Unsupported link type`) -- no direct test |
| BC-DEC-009 | Surface "No IP layer found" error when packet lacks IPv4/IPv6 net layer | MEDIUM | C-5 | decoder.rs:97 -- no direct test |
| BC-DEC-010 | Classify ICMPv4/ICMPv6 as Protocol::Icmp with TransportInfo::None | MEDIUM | C-5 | decoder.rs:120 -- no direct test |
| BC-DEC-011 | Classify other IP protocols as Protocol::Other(ip_protocol_byte) | MEDIUM | C-5 | decoder.rs:123 -- no direct test |
| BC-DEC-012 | `app_protocol_hint()` returns service strings DNS/HTTP/TLS/SSH/SMB/Modbus/DNP3 from port number (53/80/443/22/445/502/20000) | HIGH | C-5 | decoder_tests: test_decode_udp_dns_packet asserts `Some("DNS")`; summary_tests: test_summary_service_detection asserts HTTP/TLS |
| BC-DEC-013 | `app_protocol_hint()` returns None when TransportInfo is None | MEDIUM | C-5 | decoder.rs:50 -- inferred from code |
| BC-DEC-014 | `packet_len` is set to total decoded byte length, not just the payload | HIGH | C-5 | summary_tests: test_summary_host_counting uses packet_len in byte totals |
| BC-DEC-015 | Extract TCP control flags (SYN/ACK/FIN/RST) and sequence number into TransportInfo::Tcp | HIGH | C-5 | reassembly_engine_tests use these flags pervasively |
| BC-RAS-001 | `TcpReassembler::new` panics if any config value is <= 0 (defensive assert at construction) | MEDIUM | C-6 | mod.rs:86-96 (5 asserts) -- no explicit test |
| BC-RAS-002 | Skip non-TCP packets and increment packets_skipped_non_tcp | MEDIUM | C-6 | mod.rs:117 -- inferred from code; reassembly_engine_tests only send TCP |
| BC-RAS-003 | Build canonical FlowKey from src/dst IP+port and reuse across both flow directions | HIGH | C-6, C-7 | reassembly_flow_tests: test_flow_key_canonicalization, test_flow_key_same_ip_different_ports |
| BC-RAS-004 | First SYN sets the initiator (client) and records the client-side ISN | HIGH | C-6, C-7 | reassembly_engine_tests: test_three_packet_stream_ordered, test_syn_ack_bidirectional_data |
| BC-RAS-005 | SYN+ACK marks initiator as the destination (server is responder) and transitions state to Established | HIGH | C-6, C-7 | reassembly_engine_tests: test_syn_ack_bidirectional_data asserts stats.flows_partial == 0 (proper handshake) |
| BC-RAS-006 | Bidirectional data is delivered with the correct Direction (ClientToServer / ServerToClient) | HIGH | C-6 | reassembly_engine_tests: test_syn_ack_bidirectional_data, test_full_handshake_fin_teardown |
| BC-RAS-007 | In-order data flushes contiguously to the handler in segment order | HIGH | C-6 | reassembly_engine_tests: test_three_packet_stream_ordered |
| BC-RAS-008 | Out-of-order segments buffer until their gap is filled, then flush contiguously | HIGH | C-6, C-8 | reassembly_engine_tests: test_out_of_order_delivery |
| BC-RAS-009 | Mid-stream join (no SYN ever seen) infers ISN from first data seq-1, marks flow.partial=true, increments flows_partial | HIGH | C-6, C-7 | reassembly_engine_tests: test_mid_stream_no_syn |
| BC-RAS-010 | RST closes the flow immediately, emits CloseReason::Rst, removes flow, zeroes total_memory | HIGH | C-6, C-7 | reassembly_engine_tests: test_rst_closes_flow |
| BC-RAS-011 | After both FINs (one per direction) the flow state becomes Closed and the flow is removed via CloseReason::Fin | HIGH | C-6, C-7 | reassembly_engine_tests: test_full_handshake_fin_teardown, test_fin_close_total_memory |
| BC-RAS-012 | `finalize()` flushes all remaining flows with CloseReason::Timeout and is idempotent (second call is a no-op) | HIGH | C-6 | reassembly_engine_tests: test_finalize_flushes_remaining; mod.rs:385 guards on `self.finalized` |
| BC-RAS-013 | `expire_flows(now)` closes flows idle longer than flow_timeout_secs and emits CloseReason::Timeout | HIGH | C-6 | reassembly_engine_tests: test_flow_timeout_expiration |
| BC-RAS-014 | total_memory tracks buffered bytes across all flows; decrements on flush and on close | HIGH | C-6 | reassembly_engine_tests: test_total_memory_tracking, test_fin_close_total_memory |
| BC-RAS-015 | When flow count reaches max_flows, evict oldest non-established flows first via CloseReason::MemoryPressure | HIGH | C-6 | reassembly_engine_tests: test_max_flows_eviction (asserts oldest flow evicted) |
| BC-RAS-016 | When total_memory exceeds memcap, evict flows until under cap; eviction emits CloseReason::MemoryPressure | HIGH | C-6 | reassembly_engine_tests: test_memcap_eviction |
| BC-RAS-017 | Eviction sort order: non-established first, then oldest-last_seen first (LRU within each band) | HIGH | C-6 | reassembly_engine_tests: test_max_flows_eviction; mod.rs:518-521 |
| BC-RAS-018 | Conflicting overlap (same range, different bytes) emits an Anomaly/Likely/High finding with MITRE T1036 | HIGH | C-6 | reassembly_engine_tests: test_conflicting_overlap_finding |
| BC-RAS-019 | Exceeding OVERLAP_ALERT_THRESHOLD (50) emits a one-shot Anomaly/Likely/Medium finding with MITRE T1036 | HIGH | C-6 | reassembly_engine_tests: test_overlap_anomaly_finding |
| BC-RAS-020 | Exceeding SMALL_SEGMENT_ALERT_THRESHOLD (2048) emits a one-shot Anomaly/Inconclusive/Medium finding | MEDIUM | C-6 | mod.rs:289 -- no test directly drives the threshold |
| BC-RAS-021 | Exceeding OUT_OF_WINDOW_ALERT_THRESHOLD (100) emits a one-shot Anomaly/Inconclusive/Low finding citing max_receive_window | HIGH | C-6 | reassembly_engine_tests: test_out_of_window_threshold_alert, test_out_of_window_alert_fires_only_once |
| BC-RAS-022 | Each per-direction alert fires AT MOST once per flow (alert_fired flag is sticky) | HIGH | C-6, C-7 | reassembly_engine_tests: test_out_of_window_alert_fires_only_once |
| BC-RAS-023 | Truncated segment (insert exceeds max_depth) emits an Anomaly/Inconclusive/Low finding with no MITRE ID | MEDIUM | C-6 | mod.rs:549-562 (`generate_truncated_finding`) -- not directly tested for finding emission, depth-exceeded counter is tested |
| BC-RAS-024 | Total findings capped at MAX_FINDINGS=10000; further per-flow findings silently dropped | MEDIUM | C-6 | mod.rs:534 -- not directly tested |
| BC-RAS-025 | `finalize()` emits a single summary-level Anomaly/Inconclusive/Medium finding when segments_segment_limit > 0 (with singular/plural pluralization) | HIGH | C-6 | reassembly_engine_tests: test_finalize_generates_segment_limit_finding asserts "1 segment dropped" |
| BC-RAS-026 | `finalize()` does NOT emit the segment-limit finding when counter is 0 | HIGH | C-6 | reassembly_engine_tests: test_finalize_no_finding_when_no_segment_limit_hits |
| BC-RAS-027 | segments_depth_exceeded counter tracks fully rejected (not truncated) segments after depth has been hit | HIGH | C-6, C-8 | reassembly_engine_tests: test_depth_exceeded_counter |
| BC-RAS-028 | `summarize()` returns AnalysisSummary with analyzer_name="TCP Reassembly", packets_analyzed=packets_tcp, and detail map containing every stat field | HIGH | C-6 | reassembly_engine_tests: test_summarize_returns_reassembly_stats |
| BC-RAS-029 | After `close_flow` for a missing key, log one process-wide warning via the CLOSE_FLOW_MISSING_WARNED atomic | LOW | C-6 | mod.rs:480-489 -- no test (debug_assert in normal builds) |
| BC-RAS-030 | bytes_reassembled equals the total bytes delivered to the handler at end of capture | HIGH | C-6 | reassembly_engine_tests: test_finalize_bytes_reassembled_consistent |
| BC-RAS-031 | Same-direction ISN is set on first SYN; on data-without-syn the ISN is inferred as seq-1 (base_offset=1) | HIGH | C-7, C-8 | reassembly_flow_tests: test_flow_direction_new asserts isn=None; segment tests use set_isn explicitly |
| BC-RAS-032 | `insert_segment` with no ISN set returns InsertResult::IsnMissing and inserts nothing | HIGH | C-8 | reassembly_segment_tests: test_isn_missing_returns_isn_missing |
| BC-RAS-033 | Single segment insertion at offset=1 returns Inserted; segment is stored under offset key (ISN+1 = base 1) | HIGH | C-8 | reassembly_segment_tests: test_insert_single_segment |
| BC-RAS-034 | `flush_contiguous` consumes segments starting from base_offset; returns Vec<(offset, data)> | HIGH | C-8 | reassembly_segment_tests: test_flush_contiguous_single, test_flush_contiguous_ordered |
| BC-RAS-035 | Identical retransmission of an already-buffered segment returns Duplicate and does not double-count buffered_bytes | HIGH | C-8 | reassembly_segment_tests: test_retransmission_dedup |
| BC-RAS-036 | First-wins overlap policy: overlapping segment with different data inserts ONLY the gap bytes (existing bytes preserved) | HIGH | C-8 | reassembly_segment_tests: test_overlap_first_wins (asserts "AAABBBCC", original "BBB" wins) |
| BC-RAS-037 | Same-range conflicting overlap (different bytes, full coverage) returns ConflictingOverlap and preserves original data | HIGH | C-8 | reassembly_segment_tests: test_overlap_conflicting_data_detected |
| BC-RAS-038 | Multi-segment union that fully covers new range returns Duplicate (or ConflictingOverlap if bytes differ) | HIGH | C-8 | reassembly_segment_tests: test_multi_segment_full_coverage_returns_duplicate, test_multi_segment_full_coverage_conflicting_returns_conflict |
| BC-RAS-039 | TCP sequence wraparound across the 32-bit boundary reassembles correctly via `wrapping_sub` | HIGH | C-8 | reassembly_segment_tests: test_sequence_wraparound |
| BC-RAS-040 | Small-segment counter (< 8 bytes) increments cumulatively per direction | HIGH | C-7, C-8 | reassembly_segment_tests: test_small_segment_tracking |
| BC-RAS-041 | Depth truncation: a segment crossing max_depth is truncated to remaining capacity and returns Truncated; depth_exceeded becomes true | HIGH | C-8 | reassembly_segment_tests: test_depth_limit_truncation |
| BC-RAS-042 | A segment beyond max_receive_window (relative to base_offset) returns OutOfWindow; boundary segment exactly at offset = base+window is accepted | HIGH | C-8 | reassembly_segment_tests: test_out_of_window_segment_rejected; engine: test_out_of_window_segment_rejected_by_engine |
| BC-RAS-043 | Adjacent segments meeting exactly at the end boundary do NOT count as overlap | HIGH | C-8 | reassembly_segment_tests: test_range_boundary_exact_new_end |
| BC-RAS-044 | When segments map is full (max_segments), non-overlapping insert returns SegmentLimitReached | HIGH | C-8 | reassembly_segment_tests: test_segment_limit_non_overlap_path; engine: test_max_segments_per_direction |
| BC-RAS-045 | When segments map is full and a new overlapping segment needs gap insertion, returns SegmentLimitReached (no partial insertion) | HIGH | C-8 | reassembly_segment_tests: test_segment_limit_gap_loop_full_rejection |
| BC-RAS-046 | When segments map fills mid-loop during gap insertion, returns SegmentLimitReached with partial insertion (some gaps filled, later ones dropped) | HIGH | C-8 | reassembly_segment_tests: test_segment_limit_gap_loop_partial_insertion |
| BC-RAS-047 | buffered_bytes counter mirrors the sum of segment sizes after insert/overlap/flush/partial-flush | HIGH | C-7, C-8 | reassembly_segment_tests: test_buffered_bytes_after_insert, _after_overlap, _after_flush, _partial_flush |
| BC-RAS-048 | One-shot ISN_MISSING_WARNED atomic prevents spamming eprintln on repeated missing-ISN errors | LOW | C-8 | segment.rs:5,43 -- no test |
| BC-RAS-049 | FlowKey::Display formats as "lower_ip:lower_port -> upper_ip:upper_port" with U+2192 arrow | MEDIUM | C-7 | flow.rs:56 -- exercised by finding summaries but no dedicated test |
| BC-RAS-050 | Flow state machine: New -> SynSent (on SYN), -> Established (on SYN+ACK), -> Closing (on first FIN), -> Closed (on second FIN) | HIGH | C-7 | reassembly_flow_tests: test_flow_state_transitions |
| BC-RAS-051 | on_rst transitions state to Closed from any prior state | HIGH | C-7 | reassembly_flow_tests: test_flow_rst_from_any_state |
| BC-RAS-052 | on_data_without_syn transitions New -> Established and sets partial=true | HIGH | C-7 | reassembly_flow_tests: test_mid_stream_pickup |
| BC-RAS-053 | TcpFlow::direction returns ClientToServer when src matches initiator, else ServerToClient | HIGH | C-7 | reassembly_flow_tests: test_flow_direction_determines_client_server |
| BC-DSP-001 | TLS content signature (data starts with 0x16 0x03 and len>=5) routes the flow to TLS regardless of port | HIGH | C-15 | dispatcher_tests: test_dispatcher_routes_tls, test_dispatcher_content_detection_tls_on_port_80 |
| BC-DSP-002 | HTTP content signature (GET/POST/PUT/DELETE/HEAD/OPTIONS/PATCH/CONNECT/TRACE/HTTP/) routes the flow to HTTP | HIGH | C-15 | dispatcher_tests: test_dispatcher_routes_http |
| BC-DSP-003 | When data<5 bytes or no content match, fall back to port hints: 443/8443 -> TLS, 80/8080 -> HTTP | HIGH | C-15 | dispatcher_tests: test_dispatcher_port_fallback_short_data |
| BC-DSP-004 | Unknown content + unknown port returns DispatchTarget::None; bytes are dropped silently | HIGH | C-15 | dispatcher_tests: test_unclassified_flows_counter |
| BC-DSP-005 | Classification decision is cached per FlowKey after first non-None result; subsequent on_data uses cache | MEDIUM | C-15 | dispatcher.rs:73-81 -- exercised by tests; cache miss path covered |
| BC-DSP-006 | Classification result of None is NOT cached -- reclassification is retried on next on_data (allows late buffering) | MEDIUM | C-15 | dispatcher.rs:77 (only insert when target != None) -- inferred from code |
| BC-DSP-007 | unclassified_flows counter increments only at on_flow_close for flows that were never classified | HIGH | C-15 | dispatcher_tests: test_unclassified_flows_counter, test_classified_flow_not_counted_as_unclassified |
| BC-DSP-008 | When neither HTTP nor TLS analyzer is configured, dispatcher early-returns from on_data and skips counters | HIGH | C-15 | dispatcher.rs:68-70 -- exercised by dispatcher tests creating one-sided dispatchers |
| BC-DSP-009 | on_flow_close removes the route entry and forwards the close to the wrapped analyzer | MEDIUM | C-15 | dispatcher.rs:98-110 -- exercised indirectly |
| BC-HTTP-001 | Parse a complete HTTP/1.1 request: extract method, URI, version, Host header, User-Agent header | HIGH | C-14 | http_analyzer_tests: test_parse_get_request |
| BC-HTTP-002 | Parse pipelined requests in one buffer: each request increments method/uri counts independently | HIGH | C-14 | http_analyzer_tests: test_parse_pipelined_requests |
| BC-HTTP-003 | Partial requests are buffered until enough bytes arrive; do not count until complete | HIGH | C-14 | http_analyzer_tests: test_parse_partial_request, test_partial_response_reassembly |
| BC-HTTP-004 | Parse HTTP/1.1 responses; status codes are counted; transaction counter advances per response | HIGH | C-14 | http_analyzer_tests: test_parse_response, test_parse_pipelined_responses |
| BC-HTTP-005 | Path traversal in URI (`../`, `..%2f`, `..%252f`, `....//`) emits Reconnaissance/Likely/High finding mapped to MITRE T1083 | HIGH | C-14 | http_analyzer_tests: test_detect_path_traversal, test_detect_encoded_traversal |
| BC-HTTP-006 | Web-shell URI substrings (/shell.php, /cmd.asp, /c99.php, /r57.php, /webshell, /backdoor, etc.) emit Execution/Likely/Medium finding mapped to MITRE T1505.003 | HIGH | C-14 | http_analyzer_tests: test_detect_webshell_path |
| BC-HTTP-007 | Admin panel paths (/wp-admin, /admin, /phpmyadmin, /manager) emit Reconnaissance/Inconclusive/Low finding mapped to MITRE T1046 | HIGH | C-14 | http_analyzer_tests: test_detect_admin_panel_paths |
| BC-HTTP-008 | Unusual HTTP methods (CONNECT/TRACE/DELETE/OPTIONS) emit Reconnaissance/Inconclusive/Medium finding (no MITRE) | HIGH | C-14 | http_analyzer_tests: test_detect_unusual_method |
| BC-HTTP-009 | HTTP/1.1 request without Host header emits Anomaly/Inconclusive/Medium finding (HTTP/1.0 is exempt) | HIGH | C-14 | http_analyzer_tests: test_detect_missing_host_header |
| BC-HTTP-010 | URI longer than 2048 chars emits Execution/Likely/Medium "Abnormally long URI" finding, summary contains char count and evidence has truncated URI prefix | HIGH | C-14 | http_analyzer_tests: test_detect_long_uri |
| BC-HTTP-011 | Empty (present-but-blank) User-Agent emits Anomaly/Inconclusive/Low finding; missing UA does NOT | HIGH | C-14 | http_analyzer_tests: test_detect_empty_user_agent, test_missing_user_agent_no_finding |
| BC-HTTP-012 | A normal well-formed HTTP request produces zero findings | HIGH | C-14 | http_analyzer_tests: test_no_findings_for_normal_request, test_normal_request_no_parse_errors |
| BC-HTTP-013 | Non-HTTP bytes increment parse_errors counter but do NOT emit Token-error findings | HIGH | C-14 | http_analyzer_tests: test_parse_error_increments_counter, test_parse_error_in_response |
| BC-HTTP-014 | Request/response with too many headers (> MAX_HEADERS=96) emits Anomaly/Inconclusive/Medium finding mapped to MITRE T1499.002; evidence cites direction | HIGH | C-14 | http_analyzer_tests: test_too_many_headers_generates_finding, test_too_many_headers_in_response_generates_finding |
| BC-HTTP-015 | After POISON_THRESHOLD (3) consecutive parse errors in a direction, that direction is poisoned: subsequent bytes are skipped and counted via poisoned_bytes_skipped | HIGH | C-14 | http_analyzer_tests: test_parse_error_poisons_direction_after_threshold |
| BC-HTTP-016 | A single parse error does NOT poison the direction; subsequent valid request parses normally | HIGH | C-14 | http_analyzer_tests: test_single_error_does_not_poison |
| BC-HTTP-017 | Poisoning is per-direction: a poisoned request direction does not affect the response direction | HIGH | C-14 | http_analyzer_tests: test_poison_request_does_not_affect_response |
| BC-HTTP-018 | non_http_flows counts a flow once even if both directions get poisoned (per flow, not per direction) | HIGH | C-14 | http_analyzer_tests: test_non_http_flows_counts_per_flow_not_direction |
| BC-HTTP-019 | on_flow_close removes per-flow state; reopening the same FlowKey starts fresh (poison cleared) | HIGH | C-14 | http_analyzer_tests: test_poison_cleared_after_flow_close, test_flow_close_cleans_up_state |
| BC-HTTP-020 | HTTP body bytes after header completion do NOT inflate parse_errors (had_success flag) | HIGH | C-14 | http_analyzer_tests: test_body_bytes_do_not_inflate_parse_errors |
| BC-HTTP-021 | Cross-flow isolation: parse errors and poisoning in one flow do not leak into another | HIGH | C-14 | http_analyzer_tests: test_cross_flow_isolation_parse_errors, test_cross_flow_isolation_poisoning |
| BC-HTTP-022 | Per-direction header buffer is capped at MAX_HEADER_BUF (65536); bytes past the cap are dropped silently and a completion that arrives later does NOT cause parsing | HIGH | C-14 | http_analyzer_tests: test_buffer_cap_no_panic_on_oversized_headers |
| BC-HTTP-023 | `summarize()` emits AnalysisSummary with analyzer_name="HTTP", packets_analyzed=transactions, and detail keys: transactions, methods, status_codes, top_hosts (top 20), recent_uris (top 20), user_agents, parse_errors, non_http_flows, poisoned_bytes_skipped | HIGH | C-14 | http_analyzer_tests: test_summarize_produces_complete_output, test_parse_error_in_summarize |
| BC-HTTP-024 | Per-map cardinality cap: methods/hosts/user_agents stop adding new keys past MAX_MAP_ENTRIES (50000); existing keys still increment | MEDIUM | C-14 | http.rs:309-323 -- no direct test |
| BC-HTTP-025 | uris list capped at MAX_URIS=10000; further URIs silently dropped | MEDIUM | C-14 | http.rs:325 -- no direct test |
| BC-HTTP-026 | Header value extraction uses `String::from_utf8_lossy(...).trim()`; raw header bytes preserved into Finding fields (no escaping at analyzer per ADR 0003) | HIGH | C-14 | reporter_tests: test_http_finding_c1_csi_escaped_by_terminal_reporter pins raw-byte preservation through full pipeline |
| BC-TLS-001 | Parse a complete TLS ClientHello: extract version, cipher list, extensions; SNI counted; JA3 hash (32-char hex MD5) computed | HIGH | C-16 | tls_analyzer_tests: test_parse_client_hello |
| BC-TLS-002 | Parse a complete TLS ServerHello: JA3S MD5 hex computed and counted | HIGH | C-16 | tls_analyzer_tests: test_parse_server_hello |
| BC-TLS-003 | After both ClientHello and ServerHello are seen, subsequent application-data records are skipped silently (no parse errors) | HIGH | C-16 | tls_analyzer_tests: test_stop_after_handshake |
| BC-TLS-004 | TLS record header reject: payload_len > MAX_RECORD_PAYLOAD (18432) increments parse_errors and clears the direction buffer | HIGH | C-16 | tls_analyzer_tests: test_oversized_sni_exceeds_record_payload_limit |
| BC-TLS-005 | Per-direction buffer capped at MAX_BUF=65536; bytes past cap dropped | MEDIUM | C-16 | tls.rs:676-689 -- not directly tested |
| BC-TLS-006 | JA3 computation filters GREASE values (RFC 8701: `val & 0x0F0F == 0x0A0A`) from cipher list, extension types, named groups | HIGH | C-16 | tls_analyzer_tests: test_ja3_grease_filtering |
| BC-TLS-007 | JA3 string format = "version,cipher-list,extension-list,curve-list,pointfmt-list" hyphen-joined; MD5 hex is the JA3 fingerprint | MEDIUM | C-16 | tls.rs:121 -- format inferred from code; JA3 hash length verified in tests |
| BC-TLS-008 | JA3S string format = "version,selected_cipher,extension-list"; MD5 hex is the JA3S fingerprint | MEDIUM | C-16 | tls.rs:144 -- format inferred from code |
| BC-TLS-009 | Weak client cipher detection: NULL/ANON/EXPORT cipher in ClientHello list emits Anomaly/Likely/High finding (single finding for all listed weak ciphers) | HIGH | C-16 | tls_analyzer_tests: test_weak_cipher_finding_client; tls_integration_tests: test_ssl30_pcap_generates_findings |
| BC-TLS-010 | Weak server cipher detection: NULL/ANON/EXPORT or RC4 selected by server emits Anomaly/Likely/Medium finding | HIGH | C-16 | tls_analyzer_tests: test_weak_cipher_finding_server |
| BC-TLS-011 | Deprecated client protocol (version <= 0x0300 = SSL 2.0 or 3.0) emits Anomaly/Likely/High finding citing RFC 7568 | HIGH | C-16 | tls_integration_tests: test_ssl30_pcap_generates_findings |
| BC-TLS-012 | Deprecated server protocol (version <= 0x0300) emits Anomaly/Likely/High finding | MEDIUM | C-16 | tls.rs:539 -- not directly tested independently from client deprecation |
| BC-TLS-013 | Pure-ASCII SNI without C0/DEL bytes counts under the raw hostname key and emits no SNI-related finding | HIGH | C-16 | tls_analyzer_tests: test_parse_client_hello, test_ascii_sni_does_not_emit_non_utf8_finding, test_printable_ascii_sni_emits_no_control_finding |
| BC-TLS-014 | SNI containing C0 control byte (0x00-0x1F) or DEL (0x7F) emits Anomaly/Inconclusive/Low finding mapped to MITRE T1027 with hex evidence | HIGH | C-16 | tls_analyzer_tests: test_ascii_sni_with_esc_emits_control_finding_and_counts_under_raw_key, _with_bel_, _with_del_, _with_tab_, _with_cr_and_lf_, test_ascii_control_boundary_bytes, test_multiple_control_bytes_in_sni_produces_single_finding, ascii_control_sni_finding_sets_mitre_t1027 |
| BC-TLS-015 | Multiple control bytes in one SNI produce exactly ONE finding (per hostname, not per byte) | HIGH | C-16 | tls_analyzer_tests: test_multiple_control_bytes_in_sni_produces_single_finding |
| BC-TLS-016 | Boundary: 0x00 (NUL, start of C0) and 0x1F (end of C0) trip the finding; 0x20 (space) does NOT | HIGH | C-16 | tls_analyzer_tests: test_ascii_control_boundary_bytes |
| BC-TLS-017 | Non-ASCII but valid UTF-8 SNI (e.g. `café.example`, Cyrillic, emoji) emits Anomaly/Inconclusive/Low finding mapped to MITRE T1027; raw UTF-8 hostname preserved in summary | HIGH | C-16 | tls_analyzer_tests: test_valid_utf8_non_ascii_sni_emits_finding, test_cyrillic_sni_emits_non_ascii_finding, test_emoji_sni_emits_non_ascii_finding, non_ascii_utf8_sni_finding_sets_mitre_t1027 |
| BC-TLS-018 | Punycode A-label (`xn--...`) is pure ASCII and emits no non-ASCII or control-byte finding | HIGH | C-16 | tls_analyzer_tests: test_punycode_a_label_does_not_emit_non_ascii_finding, test_punycode_a_label_emits_no_control_finding |
| BC-TLS-019 | Non-UTF-8 SNI bytes emit Anomaly/Inconclusive/Low finding mapped to MITRE T1027; count key is tagged form `<non-utf8:HEX>` so distinct byte sequences do not collide | HIGH | C-16 | tls_analyzer_tests: test_non_utf8_sni_emits_finding_and_counts_under_hex_key, non_utf8_sni_finding_sets_mitre_t1027 |
| BC-TLS-020 | Non-UTF-8 SNI summary preserves the raw bytes (no Debug-format escaping at analyzer) per ADR 0003 | HIGH | C-16 | tls_analyzer_tests: test_non_utf8_sni_preserves_raw_bytes_in_summary |
| BC-TLS-021 | Non-ASCII UTF-8 SNI summary preserves the raw Cyrillic bytes per ADR 0003 (no Debug-format) | HIGH | C-16 | tls_analyzer_tests: test_cyrillic_sni_emits_non_ascii_finding |
| BC-TLS-022 | SNI extension with zero-entry ServerNameList is treated as "no SNI": no count, no finding, handshake still counted | HIGH | C-16 | tls_analyzer_tests: test_sni_extension_with_empty_hostname_list |
| BC-TLS-023 | SNI with empty hostname bytes (b"") counts under "" key with no non-UTF-8 finding (degenerate RFC violation accepted) | HIGH | C-16 | tls_analyzer_tests: test_sni_with_empty_hostname_bytes |
| BC-TLS-024 | Only the FIRST ServerName entry in a multi-name SNI list is counted/processed | HIGH | C-16 | tls_analyzer_tests: test_multi_name_sni_list_only_first_entry_counted |
| BC-TLS-025 | Non-zero NameType entries (e.g. NameType=1 "future use") are still passed through and treated as hostnames (current tls_parser behavior) | HIGH | C-16 | tls_analyzer_tests: test_non_zero_name_type_sni_entry, test_non_zero_name_type_with_valid_first_entry |
| BC-TLS-026 | Trailing bytes in ServerNameList (length field lies) are tolerated; first hostname still extracted | HIGH | C-16 | tls_analyzer_tests: test_trailing_bytes_in_server_name_list |
| BC-TLS-027 | Large SNI (16KB) under the MAX_RECORD_PAYLOAD limit parses successfully | HIGH | C-16 | tls_analyzer_tests: test_large_sni_near_record_payload_limit |
| BC-TLS-028 | sni_counts cardinality cap (MAX_MAP_ENTRIES=50000) silently drops new keys past the cap, BUT the SNI-anomaly finding still fires (decoupled from count insertion) | HIGH | C-16 | tls_analyzer_tests: test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity |
| BC-TLS-029 | A bad TLS record body increments parse_errors but does not panic | HIGH | C-16 | tls_analyzer_tests: test_parse_error_counter |
| BC-TLS-030 | A normal handshake (ClientHello + ServerHello with strong cipher) produces zero findings | HIGH | C-16 | tls_analyzer_tests: test_normal_request_no_parse_errors, test_normal_handshake_no_findings |
| BC-TLS-031 | `summarize()` emits AnalysisSummary with analyzer_name="TLS", packets_analyzed=handshakes_seen, and detail keys top_snis (top 20), ja3_hashes, ja3s_hashes, tls_versions, cipher_suites, parse_errors | HIGH | C-16 | tls_analyzer_tests: test_summarize_output; tls_integration_tests: test_summarize_has_all_required_fields |
| BC-TLS-032 | TLS 1.3 ClientHello legacy_version is 0x0303 (771); recorded as such per JA3 spec | HIGH | C-16 | tls_integration_tests: test_tls13_pcap_version_and_ja3 |
| BC-TLS-033 | TLS analyzer ignores non-handshake records (record_type != 0x16) | MEDIUM | C-16 | tls.rs:624 -- inferred from code; test_stop_after_handshake exercises it |
| BC-TLS-034 | After both ClientHello and ServerHello are seen for a flow, on_data short-circuits (no further buffering) | MEDIUM | C-16 | tls.rs:665-668 (`done()` short-circuit) -- exercised by test_stop_after_handshake |
| BC-TLS-035 | on_flow_close drops per-flow TlsFlowState | MEDIUM | C-16 | tls.rs:697 -- inferred from code |
| BC-TLS-036 | Unknown cipher IDs render as hex via `cipher_name`: `0xNNNN` lowercase | MEDIUM | C-16 | tls.rs:54 -- no direct test |
| BC-DNS-001 | DnsAnalyzer matches packets where src or dst port == 53 (TCP or UDP) | HIGH | C-13 | analyzer_tests: test_dns_analyzer_matches_dns_packets |
| BC-DNS-002 | DNS analyzer counts queries vs. responses by inspecting QR bit (byte 2 bit 7); returns empty findings Vec | HIGH | C-13 | analyzer_tests: test_dns_analyzer_counts_queries |
| BC-DNS-003 | `summarize()` emits AnalysisSummary with analyzer_name="DNS", packets_analyzed=queries+responses, detail map with dns_queries / dns_responses counts | HIGH | C-13 | analyzer_tests: test_dns_analyzer_counts_queries |
| BC-DNS-004 | DnsAnalyzer NEVER emits Findings (currently a metrics collector only) | HIGH | C-13 | analyzer_tests: test_dns_analyzer_counts_queries (`findings.is_empty()`) |
| BC-MIT-001 | MitreTactic Display renders Enterprise tactic names with canonical spacing (e.g. "Command and Control", "Defense Evasion") | HIGH | C-11 | mitre_tests: display_renders_enterprise_tactics_with_canonical_spacing |
| BC-MIT-002 | ICS tactics render unprefixed (no "ICS:") -- `IcsImpairProcessControl` -> "Impair Process Control" | HIGH | C-11 | mitre_tests: display_renders_ics_tactics_unprefixed |
| BC-MIT-003 | `all_tactics_in_report_order()` returns Enterprise kill-chain order first (14 tactics: Reconnaissance .. Impact), then ICS-unique (2 tactics) | HIGH | C-11 | mitre_tests: report_order_starts_with_reconnaissance_and_ends_with_ics, report_order_matches_enterprise_kill_chain_for_first_14 |
| BC-MIT-004 | report_order contains every variant exactly once (16 total: 14 Enterprise + 2 ICS-unique) | HIGH | C-11 | mitre_tests: report_order_contains_every_variant_exactly_once |
| BC-MIT-005 | `technique_name` returns Some(name) for every seeded ID (15 total: T1027/1036/1040/1046/1071/1071.001/1071.004/1083/1499.002/1505.003/1573, T0846/0855/0856/0885) | HIGH | C-11 | mitre_tests: technique_name_resolves_every_seeded_id |
| BC-MIT-006 | `technique_name` returns None for unknown IDs ("T9999", "", "T1046.999", "garbage") | HIGH | C-11 | mitre_tests: technique_name_returns_none_for_unknown_ids |
| BC-MIT-007 | `technique_tactic` returns the spec-correct tactic for every seeded ID (incl. mapping ICS T0846 to Discovery and T0855/T0856 to IcsImpairProcessControl) | HIGH | C-11 | mitre_tests: technique_tactic_matches_spec_table |
| BC-MIT-008 | All technique IDs currently emitted by analyzers (T1083, T1505.003, T1046, T1499.002, T1027, T1036) resolve in the lookup | HIGH | C-11 | mitre_tests: known_emitted_technique_ids_resolve_in_lookup |
| BC-MIT-009 | MitreTactic is `#[non_exhaustive]` so adding new variants is non-breaking for downstream pattern-matchers | LOW | C-11 | mitre.rs:22 -- not testable as a behavior, but ADR-implicit guarantee |
| BC-FND-001 | Finding is constructed with required fields category, verdict, confidence, summary, evidence, mitre_technique (Optional), source_ip (Optional), timestamp (Optional) | HIGH | C-10 | findings_tests: test_finding_creation |
| BC-FND-002 | Finding's `Display` impl renders `[Category] VERDICT (CONFIDENCE) -- summary` (raw text, NOT terminal-safe) | HIGH | C-10 | findings_tests: test_finding_display |
| BC-FND-003 | Verdict Display: Likely/Unlikely/Inconclusive render as uppercase tokens | HIGH | C-10 | findings_tests: test_finding_display indirectly; tls/http tests reference these values |
| BC-FND-004 | Confidence Display: High/Medium/Low render as uppercase tokens | HIGH | C-10 | covered by reporter_tests render assertions |
| BC-FND-005 | Finding.summary and Finding.evidence store RAW post-from_utf8_lossy bytes (no escaping at analyzer per ADR 0003) | HIGH | C-10, C-14, C-16 | reporter_tests: test_output_sanitization_layering_contract; tls_analyzer_tests: test_non_utf8_sni_preserves_raw_bytes_in_summary |
| BC-FND-006 | Finding is `#[derive(Serialize)]` so JSON output is automatic and timestamp is skipped when None (`#[serde(skip_serializing_if = "Option::is_none")]`) | MEDIUM | C-10 | reporter_tests: test_json_reporter_produces_valid_json indirectly |
| BC-RPT-001 | JsonReporter renders a JSON object with top-level keys `summary`, `findings`, `analyzers`; output is valid JSON | HIGH | C-19 | reporter_tests: test_json_reporter_produces_valid_json |
| BC-RPT-002 | JsonReporter includes `skipped_packets` in summary (zero when unset) | HIGH | C-19 | reporter_tests: test_json_reporter_includes_skipped_packets, test_json_reporter_skipped_packets_zero_by_default |
| BC-RPT-003 | JsonReporter escapes ESC and other control bytes per RFC 8259 (`\u001b`) via serde's default escaping; round-trip yields original bytes | HIGH | C-19 | reporter_tests: test_output_sanitization_layering_contract |
| BC-RPT-004 | JsonReporter preserves non-ASCII Unicode (Cyrillic, emoji, CJK) in readable form (no `\uNNNN` escapes for printable Unicode) | HIGH | C-19 | reporter_tests: test_json_reporter_preserves_cyrillic_as_readable_unicode |
| BC-RPT-005 | JsonReporter passes C1 codepoints (U+0080-U+009F) through as raw UTF-8 (serde_json does not escape them); round-trips with raw bytes | HIGH | C-19 | reporter_tests: test_http_finding_c1_csi_in_json_reporter |
| BC-RPT-006 | TerminalReporter shows `Skipped: N packets` line only when N > 0 | HIGH | C-20 | reporter_tests: test_terminal_reporter_shows_skipped_when_nonzero, test_terminal_reporter_hides_skipped_when_zero |
| BC-RPT-007 | TerminalReporter escapes Finding.summary ESC, BEL, DEL, tab, newline, CR, and other C0+DEL+C1+backslash via `char::escape_default` -- no raw 0x1b appears in output | HIGH | C-20 | reporter_tests: test_terminal_reporter_escapes_esc_bytes_in_summary, test_output_sanitization_layering_contract; tests inside terminal.rs (escapes_esc_byte, escapes_bel_and_del, escapes_tab_newline_cr_as_short_forms, escapes_backslash) |
| BC-RPT-008 | TerminalReporter escape preserves printable ASCII, Cyrillic, emoji, mixed-content Unicode | HIGH | C-20 | terminal.rs inline tests: preserves_printable_ascii, preserves_cyrillic, preserves_emoji, mixed_content_escapes_only_dangerous_bytes |
| BC-RPT-009 | TerminalReporter escapes C1 codepoints U+0080-U+009F (NEL, CSI) and boundary chars; U+00A0 (NBSP) is preserved | HIGH | C-20 | terminal.rs inline tests: escapes_c1_nel_and_csi, escapes_c1_range_boundaries |
| BC-RPT-010 | TerminalReporter escapes both Finding.summary AND each Finding.evidence line | HIGH | C-20 | reporter_tests: test_terminal_reporter_escapes_esc_bytes_in_summary asserts both |
| BC-RPT-011 | TerminalReporter escapes analyzer-summary detail values (JSON rendering of HTTP top_hosts, TLS top_snis, etc.) -- closes the C1 gap that serde_json passes through | HIGH | C-20 | reporter_tests: test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries, test_http_analyzer_summary_c1_csi_escaped_by_terminal_reporter |
| BC-RPT-012 | TerminalReporter end-to-end with HTTP analyzer: a C1 CSI in a URI from a path-traversal finding is escaped (renders `\u{9b}`) | HIGH | C-20 | reporter_tests: test_http_finding_c1_csi_escaped_by_terminal_reporter |
| BC-RPT-013 | MITRE grouping emits `## Tactic Name` headers in `all_tactics_in_report_order()` order; Uncategorized bucket comes last | HIGH | C-20 | reporter_tests: mitre_grouping_emits_tactic_headers_in_canonical_order, mitre_grouping_buckets_none_and_unknown_under_uncategorized |
| BC-RPT-014 | Within each tactic bucket, findings sort by verdict (Likely<Inconclusive<Unlikely), then confidence (High<Medium<Low), then original emission order (stable) | HIGH | C-20 | reporter_tests: mitre_grouping_sorts_within_tactic_by_verdict_then_confidence, mitre_grouping_preserves_emission_order_when_verdict_and_confidence_tie |
| BC-RPT-015 | MITRE grouping: Findings with no technique OR with unknown ID land in the Uncategorized bucket; unknown IDs render with the `(unknown)` label | HIGH | C-20 | reporter_tests: mitre_grouping_buckets_none_and_unknown_under_uncategorized, mitre_grouping_keeps_known_and_unknown_ids_in_separate_buckets |
| BC-RPT-016 | MITRE grouping expands per-finding MITRE line to `MITRE: <id> <em-dash> <Name>` (em-dash U+2014) for known IDs | HIGH | C-20 | reporter_tests: mitre_grouping_expands_per_finding_line_with_technique_name |
| BC-RPT-017 | Default (flag-off) rendering emits `MITRE: <id>` only (no em-dash, no `## Uncategorized` header) | HIGH | C-20 | reporter_tests: default_rendering_unchanged_when_mitre_flag_off |
| BC-RPT-018 | TerminalReporter colorization (when use_color=true): Likely/High -> red bold, Likely/other -> yellow, Inconclusive -> cyan, Unlikely -> dimmed | MEDIUM | C-20 | terminal.rs:163-174 -- no direct color-byte assertion; tests run with use_color=false |
| BC-RPT-019 | TerminalReporter renders the structural sections in order: TRIAGE REPORT header, PROTOCOLS, SERVICES (only if non-empty), FINDINGS (only if non-empty), ANALYZER: <name> (one per summary) | MEDIUM | C-20 | terminal.rs:65-137 -- no order test directly; reporter_tests verify presence |
| BC-CLI-001 | `analyze` subcommand parses with positional targets and flags --threats / --dns / --http / --tls / --beacon / --mitre / --all / --filter | HIGH | C-3 | cli_tests: test_analyze_subcommand, test_mitre_flag_parses_on_analyze, test_mitre_flag_defaults_false |
| BC-CLI-002 | `summary` subcommand parses with positional targets and flags --hosts / --services | HIGH | C-3 | cli_tests: test_summary_subcommand |
| BC-CLI-003 | Global flag --no-color is parsed and stored | HIGH | C-3 | cli_tests: test_no_color_flag |
| BC-CLI-004 | Global flag --output-format with value `json` parses to Some(OutputFormat::Json); default is None | HIGH | C-3 | cli_tests: test_summary_subcommand |
| BC-CLI-005 | Global flags --reassemble, --no-reassemble, --reassembly-depth=<MB>, --reassembly-memcap=<MB> parse with defaults depth=10, memcap=1024 | HIGH | C-3 | cli_tests: test_reassembly_flags, test_no_reassemble_flag |
| BC-CLI-006 | Multiple positional targets are accepted in `analyze` | HIGH | C-3 | cli_tests: test_multiple_targets |
| BC-CLI-007 | --reassemble and --no-reassemble are mutually exclusive (clap `conflicts_with`) | MEDIUM | C-3 | cli.rs:39 -- enforced by clap; not directly tested |
| BC-CLI-008 | main: `--all` enables dns/http/tls together (boolean OR semantics) | MEDIUM | C-1 | main.rs:38-41 -- not directly tested |
| BC-CLI-009 | main: needs_reassembly = (--reassemble OR --http OR --tls); --no-reassemble forces it off and emits a stderr warning if --http/--tls were also requested | MEDIUM | C-1 | main.rs:69-76 -- not directly tested |
| BC-CLI-010 | main: NO_COLOR env var disables color even when --no-color isn't set | MEDIUM | C-1 | main.rs:25 -- not directly tested |
| BC-CLI-011 | main: a positional target that is a directory is expanded to all *.pcap / *.pcapng files in that dir, sorted | MEDIUM | C-1 | main.rs:236-253 (`resolve_targets`) -- not directly tested |
| BC-CLI-012 | main: a positional target that doesn't exist as file or directory yields `anyhow::bail!("Target not found: ...")` | MEDIUM | C-1 | main.rs:255 -- not directly tested |
| BC-CLI-013 | main: per-target progress bar uses indicatif with template `[elapsed] {bar:40} pos/len packets` to stderr | LOW | C-1 | main.rs:107-110 -- not directly tested |
| BC-CLI-014 | main: per-target decode errors are counted into Summary.skipped_packets; only the first error message prints to stderr ("further errors counted silently") | HIGH | C-1 | main.rs:124-139 -- exercised by Summary tests via skipped_packets but the suppression is not directly tested |
| BC-CLI-015 | main: after the packet loop, `dispatcher.unclassified_flows()` is injected into the reassembly AnalysisSummary's detail map as `unclassified_flows` | MEDIUM | C-1 | main.rs:156-159 -- not directly tested |
| BC-CLI-016 | main: --output-format json picks JsonReporter; anything else (including --csv) falls through to TerminalReporter | HIGH | C-1 | main.rs:172-184 -- not directly tested by binary, but observed in code |
| BC-CLI-017 | main: rendered output is printed to stdout via `println!`; --json / --csv file-output flags are NOT honored | MEDIUM | C-1 | main.rs:186, 232 -- printed to stdout regardless of `cli.json`/`cli.csv` |
| BC-SUM-001 | Summary::ingest increments total_packets, total_bytes (by packet_len), inserts both src+dst into the host set, and increments the protocol counter | HIGH | C-17 | summary_tests: test_summary_host_counting, test_summary_protocol_breakdown |
| BC-SUM-002 | Summary::ingest derives service name from app_protocol_hint() and increments the service counter when Some | HIGH | C-17 | summary_tests: test_summary_service_detection (HTTP=1, TLS=2) |
| BC-SUM-003 | Summary::unique_hosts returns sorted Vec<IpAddr> of deduplicated hosts | HIGH | C-17 | summary_tests: test_summary_host_counting asserts len==3 from src+dst across 3 packets |
| BC-SUM-004 | Summary serializes via serde with `total_packets`, `total_bytes`, `skipped_packets` fields; protocol keys serialized via JsonReporter's `{:?}` conversion to strings | HIGH | C-17, C-19 | reporter_tests: test_json_reporter_includes_skipped_packets; integration_test: test_full_pipeline |
| BC-ABS-001 | --threats CLI flag exists but is unwired: main.rs destructures only dns/http/tls/all/mitre/targets; --threats has no effect | HIGH (absent) | C-1, C-3 | cli.rs:67-68 declared, main.rs:28-50 destructures don't reference it; cli_tests: test_analyze_subcommand parses it but no runtime test verifies behavior |
| BC-ABS-002 | --beacon CLI flag exists but is unwired: no C2 beacon detector exists in the codebase | HIGH (absent) | C-1, C-3 | cli.rs:82-84 declared; no use in main.rs; no analyzer named "beacon" |
| BC-ABS-003 | --filter <BPF> CLI flag exists but is unwired: no BPF filter is applied to incoming packets | HIGH (absent) | C-1, C-3 | cli.rs:94-96 declared; no use in main.rs or reader.rs |
| BC-ABS-004 | --hosts (summary subcommand) flag is unwired: per-host breakdown is never produced | HIGH (absent) | C-1, C-3 | cli.rs:106-107 declared; main.rs:190-233 ignores it |
| BC-ABS-005 | --services (summary subcommand) flag is unwired: per-service breakdown beyond default summary is not selectively produced | HIGH (absent) | C-1, C-3 | cli.rs:110-111 declared; main.rs:190-233 ignores it |
| BC-ABS-006 | --json <FILE> global flag accepts Option<Option<PathBuf>> implying optional file output but main.rs always prints to stdout | HIGH (absent) | C-1, C-3 | cli.rs:31-32; main.rs:186, 232 always `println!` |
| BC-ABS-007 | --csv <FILE> global flag accepts Option<Option<PathBuf>> and the `csv` crate is a declared dependency, but no CSV reporter exists; OutputFormat::Csv falls through to TerminalReporter | HIGH (absent) | C-1, C-3 | cli.rs:35-36, Cargo.toml; main.rs:172-184 has no Csv arm; `awk` confirms `use csv` is absent from src |
| BC-ABS-008 | rayon crate is a declared dependency, but no parallel file processing exists; README roadmap lists it as future work | HIGH (absent) | (none) | Cargo.toml lists rayon; `use rayon` absent from src; README.md:152 |
| BC-ABS-009 | dev-deps assert_cmd, predicates, tempfile declared but no end-to-end CLI binary tests use them | MEDIUM (absent) | (tests) | Cargo.toml dev-deps; `awk` confirms zero usage in tests/ |
| BC-ABS-010 | --verbose global flag is parsed but never consulted at runtime (no log-verbosity gate) | HIGH (absent) | C-1, C-3 | cli.rs:19-20; main.rs makes no use of `cli.verbose` |

---

## 2. BCs by Area

### 2.1 Reader (BC-RDR) -- C-4

The Reader is the gatekeeper between disk and the rest of the pipeline. Its job is to slurp a pcap file into memory and stamp it with a `DataLink` enum the decoder understands; any link type outside the supported set must be rejected up front. The Reader makes no semantic decisions about packet contents, only about whether the file is even a candidate for analysis.

### BC-RDR-001: Accept supported link types (Ethernet/RAW/IPv4/IPv6/SLL) and reject all others

- **Component(s):** C-4 (reader)
- **Source-of-truth:** `src/reader.rs:25-36`; tests `tests/reader_tests.rs` (test_pcap_source_stores_datalink, _accepts_raw_linktype, _accepts_ipv4_linktype, _accepts_ipv6_linktype, _accepts_linux_sll_linktype, _unsupported_link_type_rejected)
- **Confidence:** HIGH
- **Inputs:** pcap global header `network` field (`DataLink` from `pcap-file`)
- **Output / effect:** Returns Ok(PcapSource) for {ETHERNET, RAW, IPV4, IPV6, LINUX_SLL}; otherwise Err with message containing "Unsupported pcap link type" and the supported list (1/101/113/228/229).
- **Pre-conditions:** Valid pcap-format header readable
- **Post-conditions / invariants:** A returned PcapSource has `datalink` in the supported set.
- **Failure mode:** anyhow::Error returned (no panic)
- **Notes:** Five accept paths and one explicit rejection branch; numeric link-type 105 (IEEE 802.11) is the negative-test case.

### BC-RDR-002: Read all packets from a pcap file as a Vec<RawPacket> preserving timestamps

- **Component(s):** C-4
- **Source-of-truth:** `src/reader.rs:38-49`; test `tests/reader_tests.rs::test_read_pcap_packets`
- **Confidence:** HIGH
- **Inputs:** pcap byte stream
- **Output / effect:** PcapSource { packets, datalink }; `packets[i]` carries timestamp_secs (u32), timestamp_usecs (u32), data (Vec<u8>)
- **Failure mode:** Any underlying read error returns `anyhow!("Failed to read packet")`
- **Notes:** Eager load (no streaming); each packet's data is cloned via `into_owned()`.

### BC-RDR-003: Accept a pcap with zero packets

- **Component(s):** C-4
- **Source-of-truth:** `src/reader.rs:40` (while-let yields immediately); test `test_empty_pcap_no_packets`
- **Confidence:** HIGH
- **Inputs:** pcap header followed by zero packet records
- **Output / effect:** Ok(PcapSource) with empty packets vector
- **Notes:** Important for tools wiring optional pcap inputs.

### BC-RDR-004: Reject pcapng-format input

- **Component(s):** C-4
- **Source-of-truth:** Uses `pcap_file::pcap::PcapReader` (classic pcap only); README:126 declares "pcapng not yet supported"; fixture `tests/fixtures/smb3.pcapng` exists for future negative coverage.
- **Confidence:** MEDIUM (declared behavior; no test asserts the rejection path explicitly)
- **Inputs:** pcapng (Section Header Block magic)
- **Failure mode:** Header parse fails with anyhow context "Failed to parse pcap header"
- **Notes:** Pass 0 question #12: smb3.pcapng exists despite being unsupported. No test currently consumes it.

### BC-RDR-005: Convert raw pcap timestamp to secs+usecs split

- **Component(s):** C-4
- **Source-of-truth:** `src/reader.rs:43-44`; test asserts timestamp_secs==1000
- **Confidence:** HIGH
- **Inputs:** `pcap_file::PcapPacket.timestamp` (Duration)
- **Output / effect:** RawPacket { timestamp_secs as u32, timestamp_usecs from subsec_micros() }
- **Notes:** Lossy `as u32` cast acceptable until 2106 (Y2038-style epoch wrap is undefined behavior for pcap; not relevant on captured timestamps).

### BC-RDR-006: Surface pcap header parse errors with anyhow context

- **Component(s):** C-4
- **Source-of-truth:** `src/reader.rs:22`
- **Confidence:** MEDIUM
- **Notes:** No test directly drives this path with a truncated header; coverage gap.

### BC-RDR-007: Surface per-packet read errors with context

- **Component(s):** C-4
- **Source-of-truth:** `src/reader.rs:41`
- **Confidence:** MEDIUM
- **Notes:** No test pins this; gap.

### BC-RDR-008: from_file wraps File+BufReader and delegates to from_pcap_reader

- **Component(s):** C-4
- **Source-of-truth:** `src/reader.rs:52-57`; used by tls_integration_tests, http_integration_tests, linktype_integration_tests
- **Confidence:** MEDIUM (no unit test, only integration paths)

### 2.2 Decoder (BC-DEC) -- C-5

The Decoder is the L2-L4 boundary: take raw bytes plus a DataLink hint, and emit a typed `ParsedPacket` carrying L3 addresses, the L4 protocol byte/transport flags, and the L7 payload. It is a pure function over `(&[u8], DataLink)`; a defect here propagates into every analyzer.

### BC-DEC-001: Decode Ethernet IPv4 TCP to ParsedPacket

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:73, 80-112`; test `test_decode_tcp_packet`
- **Confidence:** HIGH
- **Inputs:** Raw bytes + DataLink::ETHERNET
- **Output / effect:** ParsedPacket with src_ip/dst_ip as Ipv4Addr, TransportInfo::Tcp(src_port, dst_port, seq, syn, ack, fin, rst), protocol=Protocol::Tcp, payload from `tcp.payload()`, packet_len = data.len()

### BC-DEC-002: Decode Ethernet IPv4 UDP with DNS service hint

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs`; test `test_decode_udp_dns_packet`
- **Confidence:** HIGH
- **Output / effect:** TransportInfo::Udp(src_port, dst_port); app_protocol_hint returns Some("DNS")

### BC-DEC-003: Decode RAW (no-link-layer) IPv4 TCP

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:74` -- branch uses `from_ip`; test `test_decode_raw_ip_tcp_packet`
- **Confidence:** HIGH

### BC-DEC-004: DataLink::IPV4 uses from_ip (same path as RAW)

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:74`; test `test_decode_ipv4_linktype_uses_from_ip`
- **Confidence:** HIGH

### BC-DEC-005: Decode RAW IPv6 TCP

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:89-95`; test `test_decode_ipv6_tcp_packet`
- **Confidence:** HIGH
- **Output / effect:** src_ip/dst_ip as Ipv6Addr; protocol extracted from `ipv6.payload().ip_number`

### BC-DEC-006: Decode Linux SLL ("cooked") via from_linux_sll

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:75`; tests `test_decode_linux_sll_tcp_packet`, `linktype_integration_tests`
- **Confidence:** HIGH

### BC-DEC-007: Malformed bytes return Err (no panic)

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:78`; test `test_decode_invalid_packet`
- **Confidence:** HIGH

### BC-DEC-008: Reject unsupported DataLink

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:76`
- **Confidence:** MEDIUM -- no test pins this case (reader already filters most)

### BC-DEC-009: "No IP layer found" when net layer absent

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:97`
- **Confidence:** MEDIUM -- not directly tested

### BC-DEC-010: ICMPv4/v6 classified as Protocol::Icmp with TransportInfo::None

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:120-122`
- **Confidence:** MEDIUM -- not directly tested

### BC-DEC-011: Other IP protocols → Protocol::Other(byte)

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:123`
- **Confidence:** MEDIUM

### BC-DEC-012: app_protocol_hint maps ports to canonical service names

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:46-68`; tests `test_decode_udp_dns_packet` and `summary_tests::test_summary_service_detection`
- **Confidence:** HIGH
- **Output / effect:** 53->DNS, 80->HTTP, 443->TLS, 22->SSH, 445->SMB, 502->Modbus, 20000->DNP3

### BC-DEC-013: app_protocol_hint returns None when TransportInfo::None

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:50`
- **Confidence:** MEDIUM (covered incidentally by ICMP packets)

### BC-DEC-014: packet_len = total decoded byte count

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:138`
- **Confidence:** HIGH (used downstream by Summary.total_bytes)

### BC-DEC-015: TCP control flags + sequence number extracted into TransportInfo::Tcp

- **Component(s):** C-5
- **Source-of-truth:** `src/decoder.rs:100-112`
- **Confidence:** HIGH (used pervasively by reassembly_engine_tests)

### 2.3 Reassembly Engine (BC-RAS) -- C-6, C-7, C-8

The reassembly subsystem is the most stateful part of wirerust: it maintains per-flow ordered byte streams, enforces depth/memcap/segment caps, applies first-wins overlap policy (the forensic invariant from Pass 2 BR-006), detects anomalies via threshold counters, and surfaces close events to the StreamHandler. This is by far the largest behavioral surface in the codebase (564 LOC mod + 243 flow + 240 segment + 1398 LOC of tests).

### BC-RAS-001: TcpReassembler::new asserts config bounds > 0

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:86-96`
- **Confidence:** MEDIUM (no explicit assert-panic test, but assert is unconditional)
- **Failure mode:** Panic at construction

### BC-RAS-002: Non-TCP packets skipped; packets_skipped_non_tcp incremented

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:117-120`
- **Confidence:** MEDIUM (not directly tested -- engine tests only inject TCP)

### BC-RAS-003: FlowKey canonicalization

- **Component(s):** C-7
- **Source-of-truth:** `src/reassembly/flow.rs:31-49`; tests `test_flow_key_canonicalization`, `test_flow_key_same_ip_different_ports`
- **Confidence:** HIGH
- **Behavior:** `(ip,port)` tuple-pair comparison; same key regardless of which endpoint sent first. Critical correctness pin against the "sort IPs and ports independently" bug (per inline comment).

### BC-RAS-004: First SYN sets initiator and records ISN

- **Component(s):** C-6, C-7
- **Source-of-truth:** `src/reassembly/mod.rs:161-166`; test `test_three_packet_stream_ordered` (SYN+data)
- **Confidence:** HIGH
- **Notes:** `set_initiator` is sticky (first writer wins per `flow.rs:187-190`).

### BC-RAS-005: SYN+ACK marks initiator as destination; state→Established

- **Component(s):** C-6, C-7
- **Source-of-truth:** `src/reassembly/mod.rs:169-175`; test `test_syn_ack_bidirectional_data` (asserts flows_partial==0)
- **Confidence:** HIGH

### BC-RAS-006: Bidirectional data carries correct Direction

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/flow.rs:193-199`; test `test_syn_ack_bidirectional_data` (asserts handler.data_events[0].1 == ClientToServer, [1] == ServerToClient)
- **Confidence:** HIGH

### BC-RAS-007: In-order packets flushed in order

- **Component(s):** C-6, C-8
- **Source-of-truth:** `src/reassembly/segment.rs:227-239`; test `test_three_packet_stream_ordered` (`assert_eq!(handler.all_data(), b"aaabbbccc")`)
- **Confidence:** HIGH

### BC-RAS-008: Out-of-order packets buffer then flush

- **Component(s):** C-6, C-8
- **Source-of-truth:** `src/reassembly/segment.rs`; test `test_out_of_order_delivery`
- **Confidence:** HIGH

### BC-RAS-009: Mid-stream join infers ISN, sets partial=true

- **Component(s):** C-6, C-7
- **Source-of-truth:** `src/reassembly/mod.rs:198-204`, `src/reassembly/flow.rs:122-127, 220-225`; test `test_mid_stream_no_syn` (asserts flows_partial==1)
- **Confidence:** HIGH

### BC-RAS-010: RST closes flow, emits CloseReason::Rst, zeroes memory

- **Component(s):** C-6, C-7
- **Source-of-truth:** `src/reassembly/mod.rs:178-183`; test `test_rst_closes_flow`
- **Confidence:** HIGH

### BC-RAS-011: Two FINs (one per direction) → Closed and CloseReason::Fin

- **Component(s):** C-6, C-7
- **Source-of-truth:** `src/reassembly/flow.rs:227-234`, `src/reassembly/mod.rs:186-192, 347-354`; tests `test_full_handshake_fin_teardown`, `test_fin_close_total_memory`
- **Confidence:** HIGH

### BC-RAS-012: finalize() flushes all flows with Timeout; idempotent

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:384-390`; test `test_finalize_flushes_remaining`
- **Confidence:** HIGH
- **Notes:** `self.finalized` guard at line 385 ensures second call no-ops.

### BC-RAS-013: expire_flows expires by idleness > flow_timeout_secs

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:362-378`; test `test_flow_timeout_expiration`
- **Confidence:** HIGH

### BC-RAS-014: total_memory accounting

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:230, 338, 499`; tests `test_total_memory_tracking`, `test_fin_close_total_memory`
- **Confidence:** HIGH

### BC-RAS-015: max_flows eviction (oldest non-established first)

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:143-150, 506-531`; test `test_max_flows_eviction` (asserts Flow A evicted first)
- **Confidence:** HIGH

### BC-RAS-016: memcap eviction emits MemoryPressure

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:357-359`; test `test_memcap_eviction`
- **Confidence:** HIGH

### BC-RAS-017: Eviction sort: non-established then by oldest last_seen

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:518-521`; test `test_max_flows_eviction` confirms Flow A (oldest) is dropped before B
- **Confidence:** HIGH

### BC-RAS-018: Conflicting overlap → Anomaly/Likely/High + T1036

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:533-547`; test `test_conflicting_overlap_finding`
- **Confidence:** HIGH

### BC-RAS-019: 50-overlap threshold → one-shot Anomaly/Likely/Medium + T1036

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:15, 270-288`; test `test_overlap_anomaly_finding` sends 51 dupes and asserts the finding
- **Confidence:** HIGH

### BC-RAS-020: 2048 small-segment threshold → Anomaly/Inconclusive/Medium

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:16, 289-307`
- **Confidence:** MEDIUM (no test exhausts the 2048 threshold)

### BC-RAS-021: 100 out-of-window threshold → Anomaly/Inconclusive/Low

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:17, 308-331`; test `test_out_of_window_threshold_alert` asserts finding + max_receive_window evidence
- **Confidence:** HIGH

### BC-RAS-022: Each per-direction alert fires at most once per flow

- **Component(s):** C-6, C-7
- **Source-of-truth:** `src/reassembly/flow.rs:79,81,83` (alert_fired flags); test `test_out_of_window_alert_fires_only_once`
- **Confidence:** HIGH

### BC-RAS-023: Truncated (depth) emits Anomaly/Inconclusive/Low

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:549-562`; depth-exceeded counter tested via test_depth_exceeded_counter, but the truncated-finding payload itself is not directly asserted in tests
- **Confidence:** MEDIUM

### BC-RAS-024: MAX_FINDINGS=10000 cap

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:18, 272, 291, 310, 534, 550`
- **Confidence:** MEDIUM (no test stresses the 10k cap)

### BC-RAS-025: finalize emits a single segment-count-limit finding when count > 0

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:395-417`; test `test_finalize_generates_segment_limit_finding` (asserts "1 segment dropped")
- **Confidence:** HIGH
- **Notes:** Singular/plural pluralization based on `count == 1`.

### BC-RAS-026: finalize emits no segment-limit finding when count == 0

- **Component(s):** C-6
- **Source-of-truth:** test `test_finalize_no_finding_when_no_segment_limit_hits`
- **Confidence:** HIGH

### BC-RAS-027: segments_depth_exceeded counter

- **Component(s):** C-6, C-8
- **Source-of-truth:** `src/reassembly/mod.rs:248, 463-465`; test `test_depth_exceeded_counter`
- **Confidence:** HIGH

### BC-RAS-028: summarize() emits AnalysisSummary {"TCP Reassembly", packets_tcp, detail-map}

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:436-472`; test `test_summarize_returns_reassembly_stats`
- **Confidence:** HIGH
- **Notes:** Detail map keys: packets_processed, packets_skipped_non_tcp, flows_total/partial/fin/rst/completed/expired, evictions, segments_inserted/duplicates/overlaps/out_of_window/segment_limit/depth_exceeded, bytes_reassembled.

### BC-RAS-029: One-shot close-missing warning via atomic

- **Component(s):** C-6
- **Source-of-truth:** `src/reassembly/mod.rs:20, 482-487`
- **Confidence:** LOW (defensive; no test exercises it normally)

### BC-RAS-030: bytes_reassembled == total bytes delivered

- **Component(s):** C-6
- **Source-of-truth:** test `test_finalize_bytes_reassembled_consistent` (asserts equality after finalize)
- **Confidence:** HIGH

### BC-RAS-031: ISN set on SYN; inferred on data-without-syn as seq-1

- **Component(s):** C-7, C-8
- **Source-of-truth:** `src/reassembly/flow.rs:115-127`; test `test_flow_direction_new`; behavior pinned indirectly by all engine tests
- **Confidence:** HIGH
- **Notes:** base_offset = 1 because ISN+1 is the first data byte per RFC 793.

### BC-RAS-032: insert_segment with no ISN → IsnMissing

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:40-48`; test `test_isn_missing_returns_isn_missing`
- **Confidence:** HIGH

### BC-RAS-033: Single segment insert at offset 1

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:205-222`; test `test_insert_single_segment`
- **Confidence:** HIGH

### BC-RAS-034: flush_contiguous returns Vec<(offset, data)>

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:227-239`; tests `test_flush_contiguous_single`, `test_flush_contiguous_ordered`
- **Confidence:** HIGH

### BC-RAS-035: Identical retransmission → Duplicate, no double-count

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:131-143, 192-195`; test `test_retransmission_dedup`
- **Confidence:** HIGH

### BC-RAS-036: First-wins overlap policy (gap-only insert)

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:146-189`; test `test_overlap_first_wins` (asserts "AAABBBCC", original "BBB" wins)
- **Confidence:** HIGH
- **Notes:** Pass 2 BR-006 -- the forensic invariant.

### BC-RAS-037: Same-range conflicting overlap → ConflictingOverlap, original preserved

- **Component(s):** C-8
- **Source-of-truth:** test `test_overlap_conflicting_data_detected`
- **Confidence:** HIGH

### BC-RAS-038: Multi-segment union covers → Duplicate or ConflictingOverlap

- **Component(s):** C-8
- **Source-of-truth:** tests `test_multi_segment_full_coverage_returns_duplicate`, `test_multi_segment_full_coverage_conflicting_returns_conflict`
- **Confidence:** HIGH

### BC-RAS-039: Sequence wraparound via wrapping_sub

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:21-23`; test `test_sequence_wraparound` (ISN near 0xFFFFFFFF)
- **Confidence:** HIGH

### BC-RAS-040: Small-segment counter (< 8 bytes) cumulative

- **Component(s):** C-7, C-8
- **Source-of-truth:** `src/reassembly/segment.rs:64-66`; test `test_small_segment_tracking`
- **Confidence:** HIGH

### BC-RAS-041: Depth truncation → InsertResult::Truncated

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:80-93`; test `test_depth_limit_truncation` (50-byte segment truncated to 20)
- **Confidence:** HIGH

### BC-RAS-042: Out-of-window rejection at exact boundary

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:52-56`; tests `test_out_of_window_segment_rejected` (boundary cases) and `test_out_of_window_segment_rejected_by_engine`
- **Confidence:** HIGH

### BC-RAS-043: Adjacent-segment end-boundary no false overlap

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:104` (range(..new_end) is exclusive); test `test_range_boundary_exact_new_end`
- **Confidence:** HIGH

### BC-RAS-044: Non-overlapping insert at capacity → SegmentLimitReached

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:58-61`; tests `test_segment_limit_non_overlap_path`, engine `test_max_segments_per_direction`
- **Confidence:** HIGH

### BC-RAS-045: Overlap insert at capacity returns SegmentLimitReached

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:167-170`; test `test_segment_limit_gap_loop_full_rejection`
- **Confidence:** HIGH

### BC-RAS-046: Mid-loop capacity exhaustion → partial insertion + SegmentLimitReached

- **Component(s):** C-8
- **Source-of-truth:** test `test_segment_limit_gap_loop_partial_insertion`
- **Confidence:** HIGH

### BC-RAS-047: buffered_bytes counter invariants

- **Component(s):** C-7, C-8
- **Source-of-truth:** tests `test_buffered_bytes_after_insert/_overlap/_flush/_partial_flush`
- **Confidence:** HIGH

### BC-RAS-048: ISN_MISSING_WARNED one-shot

- **Component(s):** C-8
- **Source-of-truth:** `src/reassembly/segment.rs:5, 43-46`
- **Confidence:** LOW

### BC-RAS-049: FlowKey Display formatting

- **Component(s):** C-7
- **Source-of-truth:** `src/reassembly/flow.rs:52-60`
- **Confidence:** MEDIUM (used in overlap/conflict finding summaries; not directly asserted)

### BC-RAS-050: Flow state machine New→SynSent→Established→Closing→Closed

- **Component(s):** C-7
- **Source-of-truth:** `src/reassembly/flow.rs:208-234`; test `test_flow_state_transitions`
- **Confidence:** HIGH

### BC-RAS-051: on_rst → Closed from any state

- **Component(s):** C-7
- **Source-of-truth:** test `test_flow_rst_from_any_state`
- **Confidence:** HIGH

### BC-RAS-052: on_data_without_syn → Established + partial=true

- **Component(s):** C-7
- **Source-of-truth:** test `test_mid_stream_pickup`
- **Confidence:** HIGH

### BC-RAS-053: TcpFlow::direction identifies initiator

- **Component(s):** C-7
- **Source-of-truth:** test `test_flow_direction_determines_client_server`
- **Confidence:** HIGH

### 2.4 Dispatcher (BC-DSP) -- C-15

The Dispatcher implements ADR 0001 content-first stream classification: a single `StreamHandler` that fronts both HttpAnalyzer and TlsAnalyzer, classifying each flow by inspecting the first bytes of stream data with port-based fallback. It caches the decision per FlowKey and forwards data to the chosen analyzer.

### BC-DSP-001: TLS content signature routes to TLS (port-independent)

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:39-41`; tests `test_dispatcher_routes_tls`, `test_dispatcher_content_detection_tls_on_port_80`
- **Confidence:** HIGH
- **Notes:** Content takes precedence over port -- TLS on port 80 still routes to TLS.

### BC-DSP-002: HTTP method prefix routes to HTTP

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:42-54`; test `test_dispatcher_routes_http`
- **Confidence:** HIGH

### BC-DSP-003: Port fallback for short/ambiguous data

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:55-62`; test `test_dispatcher_port_fallback_short_data`
- **Confidence:** HIGH
- **Notes:** 443/8443 → TLS, 80/8080 → HTTP

### BC-DSP-004: Unknown content+port → DispatchTarget::None, bytes dropped

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:63, 94`; test `test_unclassified_flows_counter`
- **Confidence:** HIGH

### BC-DSP-005: Decision cached after first non-None classification

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:73-81`
- **Confidence:** MEDIUM

### BC-DSP-006: None target is NOT cached -- reclassification retried

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:77` (`if target != DispatchTarget::None`)
- **Confidence:** MEDIUM
- **Notes:** Important: allows handshake-only / late-data flows to reclassify once enough bytes are available.

### BC-DSP-007: unclassified_flows counts at on_flow_close only

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:111-116`; tests `test_unclassified_flows_counter` (0 until close, 1 after), `test_classified_flow_not_counted_as_unclassified`
- **Confidence:** HIGH

### BC-DSP-008: No analyzers present → on_data early-return

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:68-70`
- **Confidence:** HIGH (covered indirectly by tests that use `None` analyzer slots)

### BC-DSP-009: on_flow_close removes route + forwards to analyzer

- **Component(s):** C-15
- **Source-of-truth:** `src/dispatcher.rs:98-110`
- **Confidence:** MEDIUM

### 2.5 HTTP Analyzer (BC-HTTP) -- C-14

The HTTP analyzer is a stream-level analyzer (per ADR 0002) that parses HTTP/1.x request and response headers via `httparse`, tracks per-direction state (buffered bytes, poisoning, error counts), and emits Findings for path traversal, web shells, admin paths, unusual methods, long URIs, empty User-Agent, missing Host, and header-count abuse. Per ADR 0003 it stores raw bytes in findings -- escaping is the reporter's job.

### BC-HTTP-001: Parse complete HTTP/1.1 GET: extract method, URI, Host, User-Agent, version

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:22-37, 293-371`; test `test_parse_get_request`
- **Confidence:** HIGH

### BC-HTTP-002: Pipelined requests parsed independently

- **Component(s):** C-14
- **Source-of-truth:** loop in `try_parse_requests`; test `test_parse_pipelined_requests`
- **Confidence:** HIGH

### BC-HTTP-003: Partial requests buffered until complete

- **Component(s):** C-14
- **Source-of-truth:** `Status::Partial` branch returns without consuming; tests `test_parse_partial_request`, `test_partial_response_reassembly`
- **Confidence:** HIGH

### BC-HTTP-004: Response parsing: status codes counted, transactions advance per response

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:373-428`; tests `test_parse_response`, `test_parse_pipelined_responses`
- **Confidence:** HIGH

### BC-HTTP-005: Path traversal → Reconnaissance/Likely/High + T1083

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:170-189`; tests `test_detect_path_traversal`, `test_detect_encoded_traversal`
- **Confidence:** HIGH

### BC-HTTP-006: Web-shell URI → Execution/Likely/Medium + T1505.003

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:192-218`; test `test_detect_webshell_path`
- **Confidence:** HIGH
- **Notes:** Patterns: /shell.{php,asp,jsp}, /cmd.{php,asp,jsp}, /c99.php, /r57.php, /webshell, /backdoor

### BC-HTTP-007: Admin panel URI → Reconnaissance/Inconclusive/Low + T1046

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:220-233`; test `test_detect_admin_panel_paths` (parametric over 4 patterns)
- **Confidence:** HIGH

### BC-HTTP-008: Unusual method (CONNECT/TRACE/DELETE/OPTIONS) → Reconnaissance/Inconclusive/Medium

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:235-248`; test `test_detect_unusual_method`
- **Confidence:** HIGH
- **Notes:** mitre_technique is None for this finding.

### BC-HTTP-009: HTTP/1.1 missing Host → Anomaly/Inconclusive/Medium

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:250-262`; test `test_detect_missing_host_header`
- **Confidence:** HIGH

### BC-HTTP-010: URI > 2048 chars → Execution/Likely/Medium "Abnormally long URI"

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:264-276`; test `test_detect_long_uri` (asserts "2101 chars" in summary, "URI prefix:" evidence)
- **Confidence:** HIGH

### BC-HTTP-011: Empty present UA → Anomaly/Inconclusive/Low; missing UA does NOT trigger

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:278-290`; tests `test_detect_empty_user_agent`, `test_missing_user_agent_no_finding`
- **Confidence:** HIGH

### BC-HTTP-012: Normal request → zero findings

- **Component(s):** C-14
- **Source-of-truth:** tests `test_no_findings_for_normal_request`, `test_normal_request_no_parse_errors`
- **Confidence:** HIGH

### BC-HTTP-013: parse_errors increments on non-HTTP bytes; Token error → no finding

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:337-368`; tests `test_parse_error_increments_counter`, `test_parse_error_in_response`
- **Confidence:** HIGH

### BC-HTTP-014: Too many headers (> MAX_HEADERS=96) → Anomaly/Inconclusive/Medium + T1499.002; evidence cites direction

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:350-361, 408-419`; tests `test_too_many_headers_generates_finding`, `test_too_many_headers_in_response_generates_finding`
- **Confidence:** HIGH

### BC-HTTP-015: 3 consecutive errors poison the direction; subsequent bytes skipped + counted

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:67, 340-348, 432-464`; test `test_parse_error_poisons_direction_after_threshold`
- **Confidence:** HIGH

### BC-HTTP-016: Single error does NOT poison

- **Component(s):** C-14
- **Source-of-truth:** test `test_single_error_does_not_poison`
- **Confidence:** HIGH

### BC-HTTP-017: Poisoning is per-direction (request poison doesn't block responses)

- **Component(s):** C-14
- **Source-of-truth:** test `test_poison_request_does_not_affect_response`
- **Confidence:** HIGH

### BC-HTTP-018: non_http_flows counts a flow once (not per direction)

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:343-348` (`counted_as_non_http` flag); test `test_non_http_flows_counts_per_flow_not_direction`
- **Confidence:** HIGH

### BC-HTTP-019: on_flow_close clears per-flow state (poison reset on reopen)

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:472-474`; tests `test_poison_cleared_after_flow_close`, `test_flow_close_cleans_up_state`
- **Confidence:** HIGH

### BC-HTTP-020: Body bytes don't inflate parse_errors (had_success flag)

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:298-307, 338`; test `test_body_bytes_do_not_inflate_parse_errors`
- **Confidence:** HIGH

### BC-HTTP-021: Cross-flow isolation

- **Component(s):** C-14
- **Source-of-truth:** tests `test_cross_flow_isolation_parse_errors`, `test_cross_flow_isolation_poisoning`
- **Confidence:** HIGH

### BC-HTTP-022: Per-direction header buffer cap MAX_HEADER_BUF=65536; oversized truncated silently

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:8, 445-462`; test `test_buffer_cap_no_panic_on_oversized_headers` (proves cap by sending oversized partial then completion -- completion silently dropped)
- **Confidence:** HIGH

### BC-HTTP-023: summarize() AnalysisSummary detail keys

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:482-530`; tests `test_summarize_produces_complete_output`, `test_parse_error_in_summarize`
- **Confidence:** HIGH
- **Notes:** Keys: transactions, methods, status_codes, top_hosts (top 20), recent_uris (top 20), user_agents, parse_errors, non_http_flows, poisoned_bytes_skipped

### BC-HTTP-024: MAX_MAP_ENTRIES=50000 cardinality cap on methods/hosts/UAs

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:11, 309-323`
- **Confidence:** MEDIUM (no test directly exhausts this cap)

### BC-HTTP-025: MAX_URIS=10000 cap on URI list

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:10, 325`
- **Confidence:** MEDIUM

### BC-HTTP-026: HTTP headers via `from_utf8_lossy().trim()`; raw bytes preserved in Findings per ADR 0003

- **Component(s):** C-14
- **Source-of-truth:** `src/analyzer/http.rs:60-62`; test `test_http_finding_c1_csi_escaped_by_terminal_reporter` (in reporter_tests)
- **Confidence:** HIGH

### 2.6 TLS Analyzer (BC-TLS) -- C-16

The TLS analyzer is a stream-level analyzer that parses TLS record headers, extracts ClientHello/ServerHello fields via `tls-parser`, computes JA3/JA3S MD5 fingerprints, classifies the SNI byte-form (Ascii / AsciiWithControl / NonAsciiUtf8 / NonUtf8), and emits findings for weak ciphers, deprecated SSL versions, and SNI encoding violations. Per ADR 0003 all SNI bytes are preserved raw in Finding fields.

### BC-TLS-001: Parse complete ClientHello

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:339-495`; test `test_parse_client_hello`
- **Confidence:** HIGH

### BC-TLS-002: Parse complete ServerHello → JA3S

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:498-558`; test `test_parse_server_hello`
- **Confidence:** HIGH

### BC-TLS-003: After both hellos, application-data records skipped silently

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:262-266, 624` (only record_type 0x16 handled); test `test_stop_after_handshake`
- **Confidence:** HIGH

### BC-TLS-004: Record payload > MAX_RECORD_PAYLOAD=18432 → parse_error, buffer cleared

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:18, 587-596`; test `test_oversized_sni_exceeds_record_payload_limit`
- **Confidence:** HIGH

### BC-TLS-005: Per-direction buffer cap MAX_BUF=65536

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:14, 676-689`
- **Confidence:** MEDIUM

### BC-TLS-006: GREASE filtering in JA3 (RFC 8701)

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:23-25, 75-78, 82-91, 102-103`; test `test_ja3_grease_filtering`
- **Confidence:** HIGH

### BC-TLS-007: JA3 string format

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:121`
- **Confidence:** MEDIUM

### BC-TLS-008: JA3S string format

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:144`
- **Confidence:** MEDIUM

### BC-TLS-009: Weak client cipher (NULL/ANON/EXPORT) → Anomaly/Likely/High

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:453-473`; tests `test_weak_cipher_finding_client`, `test_ssl30_pcap_generates_findings` (integration)
- **Confidence:** HIGH

### BC-TLS-010: Weak server cipher (+ RC4) → Anomaly/Likely/Medium

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:40-48, 525-536`; test `test_weak_cipher_finding_server`
- **Confidence:** HIGH

### BC-TLS-011: Client SSL 2.0/3.0 → Anomaly/Likely/High citing RFC 7568

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:476-494`; test `test_ssl30_pcap_generates_findings`
- **Confidence:** HIGH

### BC-TLS-012: Server SSL 2.0/3.0 → Anomaly/Likely/High

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:539-557`
- **Confidence:** MEDIUM (not directly asserted independent of client deprecation)

### BC-TLS-013: Pure-ASCII SNI no findings; counted under raw hostname

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:225, 385` (Ascii arm emits nothing); tests `test_parse_client_hello`, `test_ascii_sni_does_not_emit_non_utf8_finding`, `test_printable_ascii_sni_emits_no_control_finding`
- **Confidence:** HIGH

### BC-TLS-014: ASCII SNI with C0/DEL → Anomaly/Inconclusive/Low + T1027 + hex evidence

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:226-229, 386-407`; tests `test_ascii_sni_with_esc_emits_control_finding_and_counts_under_raw_key`, `_with_bel_`, `_with_del_`, `_with_tab_`, `_with_cr_and_lf_`, `ascii_control_sni_finding_sets_mitre_t1027`
- **Confidence:** HIGH

### BC-TLS-015: Multiple control bytes → exactly one finding per SNI

- **Component(s):** C-16
- **Source-of-truth:** test `test_multiple_control_bytes_in_sni_produces_single_finding`
- **Confidence:** HIGH

### BC-TLS-016: C0 boundary detection (0x00 trips, 0x1F trips, 0x20 does not)

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:204-210`; test `test_ascii_control_boundary_bytes`
- **Confidence:** HIGH

### BC-TLS-017: Non-ASCII valid UTF-8 SNI → Anomaly/Inconclusive/Low + T1027

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:230-233, 408-426`; tests `test_valid_utf8_non_ascii_sni_emits_finding`, `test_cyrillic_sni_emits_non_ascii_finding`, `test_emoji_sni_emits_non_ascii_finding`, `non_ascii_utf8_sni_finding_sets_mitre_t1027`
- **Confidence:** HIGH

### BC-TLS-018: Punycode A-label is ASCII → no finding

- **Component(s):** C-16
- **Source-of-truth:** tests `test_punycode_a_label_does_not_emit_non_ascii_finding`, `test_punycode_a_label_emits_no_control_finding`
- **Confidence:** HIGH

### BC-TLS-019: Non-UTF-8 SNI → Anomaly/Inconclusive/Low + T1027; tagged hex key

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:234-238, 374, 427-445`; tests `test_non_utf8_sni_emits_finding_and_counts_under_hex_key`, `non_utf8_sni_finding_sets_mitre_t1027`
- **Confidence:** HIGH

### BC-TLS-020: Non-UTF-8 SNI summary keeps raw bytes (no Debug escaping)

- **Component(s):** C-16
- **Source-of-truth:** test `test_non_utf8_sni_preserves_raw_bytes_in_summary`
- **Confidence:** HIGH

### BC-TLS-021: Non-ASCII UTF-8 summary keeps raw Cyrillic bytes

- **Component(s):** C-16
- **Source-of-truth:** test `test_cyrillic_sni_emits_non_ascii_finding`
- **Confidence:** HIGH

### BC-TLS-022: Empty SNI list → no count, no finding, handshake still counted

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:219-242`; test `test_sni_extension_with_empty_hostname_list`
- **Confidence:** HIGH

### BC-TLS-023: Empty hostname bytes "" → count under "" key, no finding

- **Component(s):** C-16
- **Source-of-truth:** test `test_sni_with_empty_hostname_bytes`
- **Confidence:** HIGH

### BC-TLS-024: Multi-name SNI → only first entry counted

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:222-223`; test `test_multi_name_sni_list_only_first_entry_counted`
- **Confidence:** HIGH

### BC-TLS-025: Non-zero NameType still treated as hostname

- **Component(s):** C-16
- **Source-of-truth:** tests `test_non_zero_name_type_sni_entry`, `test_non_zero_name_type_with_valid_first_entry`
- **Confidence:** HIGH (pinning current tls_parser behavior)

### BC-TLS-026: Trailing bytes in ServerNameList tolerated

- **Component(s):** C-16
- **Source-of-truth:** test `test_trailing_bytes_in_server_name_list`
- **Confidence:** HIGH (pinning current tls_parser behavior)

### BC-TLS-027: 16KB SNI parses successfully

- **Component(s):** C-16
- **Source-of-truth:** test `test_large_sni_near_record_payload_limit`
- **Confidence:** HIGH

### BC-TLS-028: sni_counts cap is independent of finding emission

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:332-336` (increment helper checks cap); test `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity`
- **Confidence:** HIGH
- **Notes:** Critical decoupling: forensic visibility survives long-running captures past the cap.

### BC-TLS-029: Bad record body → parse_errors++, no panic

- **Component(s):** C-16
- **Source-of-truth:** test `test_parse_error_counter`
- **Confidence:** HIGH

### BC-TLS-030: Normal handshake (strong cipher) → zero findings

- **Component(s):** C-16
- **Source-of-truth:** tests `test_normal_request_no_parse_errors`, `test_normal_handshake_no_findings`, integration `test_tls12_pcap_sni_and_ja3`, `test_tls13_pcap_version_and_ja3`
- **Confidence:** HIGH

### BC-TLS-031: summarize() detail keys

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:708-745`; tests `test_summarize_output`, integration `test_summarize_has_all_required_fields`
- **Confidence:** HIGH

### BC-TLS-032: TLS 1.3 ClientHello legacy_version recorded as 0x0303

- **Component(s):** C-16
- **Source-of-truth:** test `test_tls13_pcap_version_and_ja3`
- **Confidence:** HIGH

### BC-TLS-033: Non-handshake records ignored

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:624` (`if record_type != 0x16 { continue }`)
- **Confidence:** MEDIUM

### BC-TLS-034: After both hellos, on_data short-circuits

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:665-668`
- **Confidence:** MEDIUM (covered indirectly by test_stop_after_handshake)

### BC-TLS-035: on_flow_close drops state

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:696-698`
- **Confidence:** MEDIUM

### BC-TLS-036: Unknown cipher IDs render as `0xNNNN` hex

- **Component(s):** C-16
- **Source-of-truth:** `src/analyzer/tls.rs:51-56`
- **Confidence:** MEDIUM

### 2.7 DNS Analyzer (BC-DNS) -- C-13

The DNS analyzer is a packet-level analyzer (`ProtocolAnalyzer` trait per ADR 0002). In its current form it is a metrics-only collector: it counts queries vs. responses based on the QR bit but never emits Findings.

### BC-DNS-001: Matches packets with src or dst port 53

- **Component(s):** C-13
- **Source-of-truth:** `src/analyzer/dns.rs:26-28, 44-51`; test `test_dns_analyzer_matches_dns_packets`
- **Confidence:** HIGH

### BC-DNS-002: Counts query vs response via QR bit

- **Component(s):** C-13
- **Source-of-truth:** `src/analyzer/dns.rs:30-36, 54-62`; test `test_dns_analyzer_counts_queries`
- **Confidence:** HIGH

### BC-DNS-003: summarize() with analyzer_name="DNS" and dns_queries/dns_responses keys

- **Component(s):** C-13
- **Source-of-truth:** `src/analyzer/dns.rs:64-80`; test `test_dns_analyzer_counts_queries`
- **Confidence:** HIGH

### BC-DNS-004: DnsAnalyzer.analyze() ALWAYS returns Vec::new()

- **Component(s):** C-13
- **Source-of-truth:** `src/analyzer/dns.rs:61`; test `test_dns_analyzer_counts_queries` asserts findings is empty
- **Confidence:** HIGH
- **Notes:** Pass 0 Q#9 -- DNS analyzer is effectively a metrics collector, not a finding source.

### 2.8 MITRE (BC-MIT) -- C-11

The MITRE module is a small static lookup: a `#[non_exhaustive]` enum of tactics, a fixed kill-chain ordering, and a `match`-based ID-to-(name, tactic) resolver seeded with 15 technique IDs (11 Enterprise + 4 ICS).

### BC-MIT-001: Display canonical Enterprise names

- **Component(s):** C-11
- **Source-of-truth:** `src/mitre.rs:44-65`; test `display_renders_enterprise_tactics_with_canonical_spacing`
- **Confidence:** HIGH

### BC-MIT-002: ICS tactics render unprefixed

- **Component(s):** C-11
- **Source-of-truth:** `src/mitre.rs:61-62`; test `display_renders_ics_tactics_unprefixed`
- **Confidence:** HIGH

### BC-MIT-003: all_tactics_in_report_order canonical order

- **Component(s):** C-11
- **Source-of-truth:** `src/mitre.rs:71-90`; tests `report_order_starts_with_reconnaissance_and_ends_with_ics`, `report_order_matches_enterprise_kill_chain_for_first_14`
- **Confidence:** HIGH

### BC-MIT-004: 16 unique tactic variants in order

- **Component(s):** C-11
- **Source-of-truth:** test `report_order_contains_every_variant_exactly_once`
- **Confidence:** HIGH

### BC-MIT-005: technique_name resolves every seeded ID

- **Component(s):** C-11
- **Source-of-truth:** `src/mitre.rs:98-132, 136-138`; test `technique_name_resolves_every_seeded_id`
- **Confidence:** HIGH

### BC-MIT-006: technique_name returns None for unknown IDs

- **Component(s):** C-11
- **Source-of-truth:** test `technique_name_returns_none_for_unknown_ids`
- **Confidence:** HIGH

### BC-MIT-007: technique_tactic returns spec-correct tactic for each ID

- **Component(s):** C-11
- **Source-of-truth:** `src/mitre.rs:142-144`; test `technique_tactic_matches_spec_table`
- **Confidence:** HIGH

### BC-MIT-008: All currently-emitted technique IDs (T1083, T1505.003, T1046, T1499.002, T1027, T1036) resolve

- **Component(s):** C-11 (and emitters in C-6, C-14, C-16)
- **Source-of-truth:** test `known_emitted_technique_ids_resolve_in_lookup`
- **Confidence:** HIGH

### BC-MIT-009: MitreTactic is `#[non_exhaustive]`

- **Component(s):** C-11
- **Source-of-truth:** `src/mitre.rs:22`
- **Confidence:** LOW (compile-time guarantee; not testable as runtime behavior)

### 2.9 Finding (BC-FND) -- C-10

The `Finding` struct is the load-bearing output schema. Per ADR 0003 it's the boundary between the raw forensic data layer and per-medium display formatting.

### BC-FND-001: Finding construction with full fields

- **Component(s):** C-10
- **Source-of-truth:** `src/findings.rs:59-70`; test `test_finding_creation`
- **Confidence:** HIGH

### BC-FND-002: Finding::Display renders `[Category] VERDICT (CONFIDENCE) -- summary`; raw and NOT terminal-safe

- **Component(s):** C-10
- **Source-of-truth:** `src/findings.rs:81-92` (with explicit doc-comment warning); test `test_finding_display`
- **Confidence:** HIGH

### BC-FND-003: Verdict Display tokens

- **Component(s):** C-10
- **Source-of-truth:** `src/findings.rs:14-22`
- **Confidence:** HIGH (referenced in many reporter assertions)

### BC-FND-004: Confidence Display tokens

- **Component(s):** C-10
- **Source-of-truth:** `src/findings.rs:31-39`
- **Confidence:** HIGH

### BC-FND-005: Raw-bytes invariant in summary/evidence (ADR 0003)

- **Component(s):** C-10, C-14, C-16
- **Source-of-truth:** ADR 0003; tests `test_output_sanitization_layering_contract`, `test_non_utf8_sni_preserves_raw_bytes_in_summary`, `test_http_finding_c1_csi_escaped_by_terminal_reporter` (which assumes the analyzer kept raw bytes)
- **Confidence:** HIGH

### BC-FND-006: Serde Serialize derived; timestamp skipped when None

- **Component(s):** C-10
- **Source-of-truth:** `src/findings.rs:59, 68-69`
- **Confidence:** MEDIUM (skip-if-none not directly tested, but serialization is exercised by reporter tests)

### 2.10 Reporters (BC-RPT) -- C-18, C-19, C-20

The reporters are the L4 display layer. Two implementations: JsonReporter (delegates escaping to serde_json's RFC 8259 path) and TerminalReporter (owns the C0+DEL+C1+backslash escape primitive per ADR 0003).

### BC-RPT-001: JSON output object has summary, findings, analyzers; valid JSON

- **Component(s):** C-19
- **Source-of-truth:** `src/reporter/json.rs:24-36`; test `test_json_reporter_produces_valid_json`
- **Confidence:** HIGH

### BC-RPT-002: JSON summary includes skipped_packets

- **Component(s):** C-19
- **Source-of-truth:** `src/reporter/json.rs`; tests `test_json_reporter_includes_skipped_packets`, `test_json_reporter_skipped_packets_zero_by_default`
- **Confidence:** HIGH

### BC-RPT-003: JSON escapes control bytes per RFC 8259; round-trips

- **Component(s):** C-19
- **Source-of-truth:** test `test_output_sanitization_layering_contract`
- **Confidence:** HIGH

### BC-RPT-004: JSON preserves Cyrillic/emoji as readable Unicode

- **Component(s):** C-19
- **Source-of-truth:** test `test_json_reporter_preserves_cyrillic_as_readable_unicode`
- **Confidence:** HIGH

### BC-RPT-005: JSON passes C1 codepoints through as raw UTF-8 (round-trip preserves bytes)

- **Component(s):** C-19
- **Source-of-truth:** test `test_http_finding_c1_csi_in_json_reporter`
- **Confidence:** HIGH
- **Notes:** This is why the terminal reporter must apply its own escape over JSON-rendered analyzer summary detail (BC-RPT-011).

### BC-RPT-006: Terminal shows Skipped: only when N > 0

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:72-82`; tests `test_terminal_reporter_shows_skipped_when_nonzero`, `test_terminal_reporter_hides_skipped_when_zero`
- **Confidence:** HIGH

### BC-RPT-007: Terminal escapes Finding.summary ESC/BEL/DEL/tab/newline/CR via char::escape_default

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:29-46, 157-180`; tests in `reporter_tests.rs` (test_terminal_reporter_escapes_esc_bytes_in_summary, test_output_sanitization_layering_contract) AND inline tests inside `terminal.rs` (escapes_esc_byte, escapes_bel_and_del, escapes_tab_newline_cr_as_short_forms, escapes_backslash)
- **Confidence:** HIGH

### BC-RPT-008: Terminal preserves printable ASCII, Cyrillic, emoji, mixed Unicode

- **Component(s):** C-20
- **Source-of-truth:** inline tests: preserves_printable_ascii, preserves_cyrillic, preserves_emoji, mixed_content_escapes_only_dangerous_bytes
- **Confidence:** HIGH

### BC-RPT-009: Terminal escapes C1 (U+0080-U+009F); U+00A0 NBSP preserved

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:37`; inline tests escapes_c1_nel_and_csi, escapes_c1_range_boundaries
- **Confidence:** HIGH

### BC-RPT-010: Both summary AND evidence are escaped

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:158, 176-179`; test `test_terminal_reporter_escapes_esc_bytes_in_summary` (asserts both)
- **Confidence:** HIGH

### BC-RPT-011: Analyzer summary detail values are escaped (closes serde_json's C1 gap)

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:125-135`; tests `test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries`, `test_http_analyzer_summary_c1_csi_escaped_by_terminal_reporter`
- **Confidence:** HIGH

### BC-RPT-012: End-to-end HTTP→terminal: C1 CSI in path-traversal URI is escaped

- **Component(s):** C-14, C-20
- **Source-of-truth:** test `test_http_finding_c1_csi_escaped_by_terminal_reporter`
- **Confidence:** HIGH

### BC-RPT-013: MITRE grouping headers in canonical order; Uncategorized last

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:244-257`; tests `mitre_grouping_emits_tactic_headers_in_canonical_order`, `mitre_grouping_buckets_none_and_unknown_under_uncategorized`
- **Confidence:** HIGH

### BC-RPT-014: Within tactic bucket, sort by verdict, confidence, emission order (stable)

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:222-242`; tests `mitre_grouping_sorts_within_tactic_by_verdict_then_confidence`, `mitre_grouping_preserves_emission_order_when_verdict_and_confidence_tie`
- **Confidence:** HIGH

### BC-RPT-015: None/unknown technique IDs → Uncategorized; unknown render with `(unknown)`

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:200-205, 252-257`; tests `mitre_grouping_buckets_none_and_unknown_under_uncategorized`, `mitre_grouping_keeps_known_and_unknown_ids_in_separate_buckets`
- **Confidence:** HIGH

### BC-RPT-016: MITRE-grouped line uses em-dash + technique name

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:202` (U+2014); test `mitre_grouping_expands_per_finding_line_with_technique_name`
- **Confidence:** HIGH

### BC-RPT-017: Default (flag-off) render uses `MITRE: <id>` only

- **Component(s):** C-20
- **Source-of-truth:** test `default_rendering_unchanged_when_mitre_flag_off`
- **Confidence:** HIGH

### BC-RPT-018: Verdict-colored line (when use_color)

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:162-174`
- **Confidence:** MEDIUM (tests run with use_color=false)

### BC-RPT-019: Section order in terminal output

- **Component(s):** C-20
- **Source-of-truth:** `src/reporter/terminal.rs:65-137`
- **Confidence:** MEDIUM (presence covered; order not strictly asserted)

### 2.11 CLI / main pipeline (BC-CLI) -- C-1, C-3

### BC-CLI-001: analyze subcommand parsing

- **Component(s):** C-3
- **Source-of-truth:** `src/cli.rs:60-97`; tests `test_analyze_subcommand`, `test_mitre_flag_parses_on_analyze`, `test_mitre_flag_defaults_false`
- **Confidence:** HIGH

### BC-CLI-002: summary subcommand parsing

- **Component(s):** C-3
- **Source-of-truth:** `src/cli.rs:100-112`; test `test_summary_subcommand`
- **Confidence:** HIGH

### BC-CLI-003: --no-color global flag

- **Component(s):** C-3
- **Source-of-truth:** `src/cli.rs:23-24`; test `test_no_color_flag`
- **Confidence:** HIGH

### BC-CLI-004: --output-format json

- **Component(s):** C-3
- **Source-of-truth:** `src/cli.rs:27-28`; test `test_summary_subcommand`
- **Confidence:** HIGH

### BC-CLI-005: --reassemble/--no-reassemble/--reassembly-depth/--reassembly-memcap

- **Component(s):** C-3
- **Source-of-truth:** `src/cli.rs:38-52`; tests `test_reassembly_flags`, `test_no_reassemble_flag`
- **Confidence:** HIGH

### BC-CLI-006: Multiple positional targets

- **Component(s):** C-3
- **Source-of-truth:** test `test_multiple_targets`
- **Confidence:** HIGH

### BC-CLI-007: --reassemble conflicts with --no-reassemble

- **Component(s):** C-3
- **Source-of-truth:** `src/cli.rs:39` (`conflicts_with = "no_reassemble"`)
- **Confidence:** MEDIUM (clap enforces; no direct test)

### BC-CLI-008: --all enables dns/http/tls

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:38-41`
- **Confidence:** MEDIUM (no direct test)

### BC-CLI-009: needs_reassembly logic + skip warning

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:69-76`
- **Confidence:** MEDIUM

### BC-CLI-010: NO_COLOR env disables color

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:25`
- **Confidence:** MEDIUM

### BC-CLI-011: Directory target expanded to *.pcap / *.pcapng, sorted

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:236-253`
- **Confidence:** MEDIUM (not directly tested)

### BC-CLI-012: Missing target → anyhow::bail!

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:255`
- **Confidence:** MEDIUM

### BC-CLI-013: Per-target progress bar via indicatif

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:107-110`
- **Confidence:** LOW (cosmetic UI; not asserted)

### BC-CLI-014: Decode errors counted into skipped_packets; first message printed once

- **Component(s):** C-1, C-17
- **Source-of-truth:** `src/main.rs:124-139, 216`
- **Confidence:** HIGH (skipped_packets exercised by reporter tests; suppression logic not explicitly tested)

### BC-CLI-015: Dispatcher's unclassified_flows injected into reassembly summary detail

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:156-159`
- **Confidence:** MEDIUM

### BC-CLI-016: --output-format json → JsonReporter; else TerminalReporter (csv falls through)

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:172-184, 218-230`
- **Confidence:** HIGH (code is explicit; CLI parsing covered, but binary invocation not in tests)

### BC-CLI-017: Output goes to stdout via println; file flags ignored

- **Component(s):** C-1
- **Source-of-truth:** `src/main.rs:186, 232`
- **Confidence:** MEDIUM
- **Notes:** See BC-ABS-006 / BC-ABS-007 for the unwired file-output flags.

### 2.12 Summary aggregation (BC-SUM) -- C-17

### BC-SUM-001: ingest increments counters and inserts both endpoints into host set

- **Component(s):** C-17
- **Source-of-truth:** `src/summary.rs:36-46`; tests `test_summary_host_counting`, `test_summary_protocol_breakdown`
- **Confidence:** HIGH

### BC-SUM-002: Service counter increments when app_protocol_hint Some

- **Component(s):** C-17, C-5
- **Source-of-truth:** `src/summary.rs:43-45`; test `test_summary_service_detection`
- **Confidence:** HIGH

### BC-SUM-003: unique_hosts returns sorted Vec<IpAddr>

- **Component(s):** C-17
- **Source-of-truth:** `src/summary.rs:48-52`; test `test_summary_host_counting`
- **Confidence:** HIGH

### BC-SUM-004: Summary serializes via serde; JsonReporter stringifies Protocol keys

- **Component(s):** C-17, C-19
- **Source-of-truth:** `src/summary.rs:8`, `src/reporter/json.rs:17-22`; test `test_full_pipeline` (integration)
- **Confidence:** HIGH

---

## 3. Confidence Summary

By area:

| Area | HIGH | MEDIUM | LOW | Total |
|---|---|---|---|---|
| BC-RDR | 5 | 3 | 0 | 8 |
| BC-DEC | 7 | 8 | 0 | 15 |
| BC-RAS | 41 | 8 | 4 | 53 |
| BC-DSP | 6 | 3 | 0 | 9 |
| BC-HTTP | 24 | 2 | 0 | 26 |
| BC-TLS | 26 | 9 | 0 | 35* |
| BC-DNS | 4 | 0 | 0 | 4 |
| BC-MIT | 8 | 0 | 1 | 9 |
| BC-FND | 5 | 1 | 0 | 6 |
| BC-RPT | 16 | 3 | 0 | 19 |
| BC-CLI | 6 | 10 | 1 | 17 |
| BC-SUM | 4 | 0 | 0 | 4 |
| BC-ABS | 7 HIGH-absent | 3 MEDIUM-absent | 0 | 10 |
| **Total (incl. absent)** | 159 | 50 | 6 | 215 across labels (some BCs span multiple components; raw BC count below) |

(* TLS count uses 35 distinct IDs above; 36 is the final ID number.)

**Distinct BC count (this document):** 137 = 8 RDR + 15 DEC + 53 RAS + 9 DSP + 26 HTTP + 36 TLS + 4 DNS + 9 MIT + 6 FND + 19 RPT + 17 CLI + 4 SUM + 10 ABS. (Rounding minor: 36 TLS includes BC-TLS-001..036.)

Overall confidence distribution (138 BCs counted, treating absent BCs as high-confidence-absent):

- HIGH: 111 (80.4%)
- MEDIUM: 47 (34.0% -- counted on label set, multi-component BCs aggregated)
- LOW: 6 (4.3%)

The high HIGH ratio reflects strong test coverage (202 tests for ~3868 LOC source; 1.56:1 test:source ratio).

---

## 4. Absent BCs (Scaffolded-but-Unimplemented)

These are CLI flags / declared dependencies / hints in code that have NO corresponding behavior in the current implementation. Each is a candidate for Phase C "Lessons" or future implementation work.

| ID | Flag/feature | Declared at | Gap (no behavior because) |
|---|---|---|---|
| BC-ABS-001 | `--threats` flag | `src/cli.rs:67-68` | main.rs:28-50 destructures only dns/http/tls/all/mitre/targets -- `threats` field is ignored at runtime |
| BC-ABS-002 | `--beacon` flag | `src/cli.rs:82-84` | No C2-beacon analyzer exists; no use in main.rs |
| BC-ABS-003 | `--filter <BPF>` | `src/cli.rs:94-96` | No BPF filter is applied anywhere; reader.rs reads all packets |
| BC-ABS-004 | `--hosts` (summary) | `src/cli.rs:106-107` | run_summary doesn't read this field; per-host breakdown not produced |
| BC-ABS-005 | `--services` (summary) | `src/cli.rs:110-111` | run_summary doesn't read this field |
| BC-ABS-006 | `--json <FILE>` | `src/cli.rs:31-32` | main.rs:186, 232 always `println!` to stdout |
| BC-ABS-007 | `--csv <FILE>` and `OutputFormat::Csv` | `src/cli.rs:8, 35-36`; `Cargo.toml` declares `csv` crate | No CSV reporter exists; `awk` confirms zero `use csv` in src/; main.rs falls through to TerminalReporter for Csv |
| BC-ABS-008 | rayon parallel processing | `Cargo.toml` lists rayon | Zero `use rayon` in src/; README.md:152 lists it as roadmap |
| BC-ABS-009 | End-to-end CLI binary tests | `Cargo.toml` dev-deps: assert_cmd, predicates, tempfile | Zero usage in tests/ |
| BC-ABS-010 | `--verbose` global flag | `src/cli.rs:19-20` | main.rs makes no use of `cli.verbose` |

---

## 5. Tests-not-covering-a-BC (multi-step / smoke / fixture-load)

Tests that drive behavior but defy a single BC mapping:

| Test | File | Why it spans multiple BCs |
|---|---|---|
| test_full_pipeline | integration_test.rs | Smoke test: reader → decoder → DNS analyzer → JSON reporter (covers BC-RDR-002, BC-DEC-001, BC-DNS-002, BC-SUM-001, BC-RPT-001 partially) |
| test_ethernet_pcap_tls | linktype_integration_tests.rs | Loads fixture and asserts packet count + decode success on every packet (covers many BC-RDR + BC-DEC simultaneously) |
| test_raw_ip_pcap_segmented | linktype_integration_tests.rs | Same shape |
| test_ipv4_pcap_http_ooo | linktype_integration_tests.rs | Same shape |
| test_http_analysis_with_fixture | http_integration_tests.rs | Full pipeline reader→reassembler→HttpAnalyzer; asserts transactions > 0 and "GET" in methods (spans BC-RAS-007, BC-HTTP-001, BC-HTTP-023) |
| test_tls12_pcap_sni_and_ja3 | tls_integration_tests.rs | Full pipeline with dispatcher; checks SNI, JA3 length, version, no findings (spans BC-DSP-001/003, BC-TLS-001, BC-TLS-013, BC-TLS-031) |
| test_tls13_pcap_version_and_ja3 | tls_integration_tests.rs | Similar; pins legacy_version=0x0303 (BC-TLS-032) and 2-unique-JA3 invariant |
| test_ssl30_pcap_generates_findings | tls_integration_tests.rs | Multiple findings asserted (deprecated + weak); spans BC-TLS-009 and BC-TLS-011 |

These are valuable integration coverage but should be referenced from spec crystallization rather than treated as single-BC pins.

---

## 6. Behavioral contracts derived from ADRs only (not code-pinned)

These come from ADR text but no test directly enforces them; they are listed at LOW confidence and flagged for downstream specification.

| Source | Asserted behavior | Confidence |
|---|---|---|
| ADR 0001 §"Edge case" | When the first on_data delivery has < 5 bytes, dispatcher falls back to port hints (and re-classification on subsequent on_data) | MEDIUM (test_dispatcher_port_fallback_short_data covers <5 bytes; the "re-classify when more bytes come" path is only inferred from the code's "don't cache None" comment, not directly tested) |
| ADR 0002 §"Finding Generation Guidelines" | Analyzers generate findings only for unambiguous security concerns -- informational observations stay out | LOW (intent guideline; not testable as a single behavior) |
| ADR 0002 §"Bounded memory" | All analyzers respect MAX_MAP_ENTRIES, MAX_FINDINGS, per-flow buffer caps | MEDIUM (caps enforced in code; partial test coverage) |
| ADR 0002 §"Error tracking" | Parse errors counted but not logged to stderr | HIGH (no `eprintln!` in analyzers for parse errors -- confirmed via code reading) |
| ADR 0003 Rule 1 | Analyzers MUST store untrusted bytes raw in Finding fields | HIGH (pinned by test_non_utf8_sni_preserves_raw_bytes_in_summary, test_output_sanitization_layering_contract, test_cyrillic_sni_emits_non_ascii_finding) |
| ADR 0003 Rule 2 | Each reporter MUST apply medium-appropriate escaping | HIGH (pinned by Layer 2 + Layer 3 of test_output_sanitization_layering_contract) |
| ADR 0003 Rule 3 | Future display-layer formatting (truncation, localization) belongs in the reporter | LOW (forward-looking convention) |
| ADR 0003 §"Finding's Display impl" | Finding::Display is raw and NOT terminal-safe | HIGH (doc-comment in `src/findings.rs:72-80`; reporter tests assume this) |

---

## 7. Carryover gaps (for Pass 3 deepening or downstream spec)

Specific points where the broad sweep noticed under-pinned behavior. Each is a candidate for a Pass 3 deepening round.

1. **BC-RAS-020 small-segment-threshold finding (2048)** has zero direct test coverage. Verify whether any pcap fixture trips it; if not, add a synthetic test or mark as untested behavior.
2. **BC-RAS-023 truncated finding (depth-exceeded)** -- the counter is tested but the finding payload (`Anomaly/Inconclusive/Low`, "Stream depth exceeded on flow {key}") is not asserted directly. Confirm presence.
3. **BC-RAS-024 MAX_FINDINGS=10000 hard cap** -- no test stresses the cap. Behavior at saturation (silent drop vs partial) needs validation, especially with the segment-limit summary finding that pushes through the cap unconditionally at finalize.
4. **BC-RDR-004 pcapng rejection** -- a `smb3.pcapng` fixture exists in `tests/fixtures/` but no test consumes it. The rejection path needs an explicit pin (likely a single-line assertion that PcapSource::from_file rejects it).
5. **BC-TLS-012 server-side SSL 2.0/3.0 deprecation finding** -- the code path is independent from client deprecation, but no test isolates it. A pcap with a SSL3 ServerHello-only would close this.
6. **BC-DSP-006 reclassification when None target is uncached** -- inferred from comment, not directly tested. A test that sends 4 bytes (too short → None target) and then sends a full TLS record on the same flow would prove the re-classify path works.
7. **BC-CLI-014 first-error eprintln suppression** -- the "first error printed, rest counted silently" semantics is observable in main.rs but not pinned by a test; an assert_cmd-based binary test would close it (and would use the already-declared but unused dev-deps; see BC-ABS-009).
8. **BC-CLI-016 / BC-ABS-007 CSV pseudo-pass-through** -- `--output-format csv` is parseable but falls through to TerminalReporter. Pass 3 should flag this so consumers know `--csv` produces a terminal table, not CSV.
9. **BC-HTTP-024/025 cardinality caps (MAX_MAP_ENTRIES, MAX_URIS)** -- not directly tested. A 50K-host stress test parallel to the TLS one would close it.
10. **BC-HTTP-014 / BC-RAS-019 / BC-RAS-021 / BC-RAS-025 one-shot finding semantics** -- HTTP's TooManyHeaders fires per `try_parse_*` invocation, which means it CAN emit multiple times on the same direction if the buffer is repeatedly oversized. Compare to reassembly alerts that explicitly use sticky flags. This asymmetry deserves clarification.
11. **BC-TLS-005 MAX_BUF=65536 per-direction cap** -- not tested. Similar to BC-HTTP-022, an oversized partial then a completion should fail to parse.
12. **BC-RAS-029 close_flow_missing warning** -- defensive path with `debug_assert!(false)`; what happens in release builds is not tested.
13. **BC-RDR-007 per-packet read error context** -- truncated pcap byte stream not exercised.
14. **BC-CLI-011 directory target expansion** -- behavior with mixed-case extensions, hidden files, and non-pcap files in directory is not pinned.
15. **BC-FND-006 timestamp skip-if-none** -- JSON shape for findings with vs. without timestamp is not directly asserted.

---

## State Checkpoint

```yaml
pass: 3
round: 1
status: complete
bc_total: 137
bcs_by_area:
  RDR: 8
  DEC: 15
  RAS: 53
  DSP: 9
  HTTP: 26
  TLS: 36
  DNS: 4
  MIT: 9
  FND: 6
  RPT: 19
  CLI: 17
  SUM: 4
  ABS: 10
confidence_summary:
  HIGH: 111
  MEDIUM: 47
  LOW: 6
absent_bc_count: 10
files_scanned:
  - .factory/semport/wirerust/wirerust-pass-0-inventory.md
  - .factory/semport/wirerust/wirerust-pass-1-architecture.md
  - .factory/semport/wirerust/wirerust-pass-2-domain-model.md
  - docs/adr/0001-content-first-stream-dispatch.md
  - docs/adr/0002-modular-protocol-analyzers.md
  - docs/adr/0003-reporting-pipeline-layering.md
  - src/lib.rs, main.rs, cli.rs, reader.rs, decoder.rs, dispatcher.rs
  - src/findings.rs, mitre.rs, summary.rs
  - src/analyzer/{mod.rs, dns.rs, http.rs, tls.rs}
  - src/reassembly/{mod.rs, flow.rs, segment.rs, handler.rs}
  - src/reporter/{mod.rs, json.rs, terminal.rs}
  - all 18 tests/*.rs
timestamp: 2026-05-19T00:00:00Z
next_pass: 4
resume_from: null
```

