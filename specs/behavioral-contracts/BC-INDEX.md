---
document_type: bc-index
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
traces_to: .factory/specs/prd.md
---

# wirerust Behavioral Contracts Index

> **Navigation:** This file is the master index of all BC-S.SS.NNN contracts. Each entry
> links to the individual BC file. BCs are sharded into per-subsystem directories (ss-NN/).
>
> All BCs are marked [WRITTEN]. Body files have been verified on disk for all 244 entries.
> 218 draft ingestion BCs were produced; 6 were retired during the remediation cycle (BC-ABS-004
> through BC-ABS-009) leaving 212 active L3 BCs from ingestion. BC-2.11.020 through BC-2.11.024
> were added in adversarial-review pass-4 (finding H-1: CsvReporter coverage gap), bringing
> the total to 217 active L3 BCs. BC-2.04.055 and BC-2.09.007 were added in Feature Mode F2
> (issue #100 pcap-timestamps delta) bringing the total to 219 active L3 BCs. BC-2.14.001
> through BC-2.14.025 were added in Feature Mode F2 (issue #7 Modbus/ICS analyzer) bringing
> the total to 244 active L3 BCs.
>
> **Status as of Phase 1a (current):**
> - Fully written: 244 BCs (all body files verified on disk)
> - Remaining: 0 BCs
> - PRD index (prd.md): UPDATED (v1.1) -- all 244 L3 BC IDs are registered

## ss-01: PCAP File Ingestion (CAP-01)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.01.001 | Accept Supported Link Types and Reject Unsupported at File Open | P0 | [WRITTEN] | BC-RDR-001 |
| BC-2.01.002 | Read All Packets from PCAP as Vec<RawPacket> Preserving Timestamps | P0 | [WRITTEN] | BC-RDR-002 |
| BC-2.01.003 | Accept PCAP with Zero Packets Without Error | P1 | [WRITTEN] | BC-RDR-003 |
| BC-2.01.004 | Reject pcapng-Format Input at Reader Level | P0 | [WRITTEN] | BC-RDR-004 |
| BC-2.01.005 | Convert PCAP Record Timestamp to (timestamp_secs: u32, timestamp_usecs: u32) | P1 | [WRITTEN] | BC-RDR-005 |
| BC-2.01.006 | Surface PCAP Header Parse Errors with Anyhow Context | P1 | [WRITTEN] | BC-RDR-006 |
| BC-2.01.007 | Surface Per-Packet Read Errors with Anyhow Context | P1 | [WRITTEN] | BC-RDR-007 |
| BC-2.01.008 | from_file Opens via BufReader and Delegates to from_pcap_reader | P2 | [WRITTEN] | BC-RDR-008 |

## ss-02: Link-Type Gating / Packet Decoding (CAP-02 + CAP-03)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.02.001 | Decode Ethernet-framed IPv4 TCP Packet to ParsedPacket | P0 | [WRITTEN] | BC-DEC-001 |
| BC-2.02.002 | Decode Ethernet-framed IPv4 UDP Packet with DNS Port Hint | P0 | [WRITTEN] | BC-DEC-002 |
| BC-2.02.003 | Decode RAW Link-Layer IPv4 TCP Packet via from_ip | P0 | [WRITTEN] | BC-DEC-003 |
| BC-2.02.004 | DataLink::IPV4 Decodes Identically to DataLink::RAW | P1 | [WRITTEN] | BC-DEC-004 |
| BC-2.02.005 | Decode RAW IPv6 TCP Packet Surfacing IPv6 Addresses | P0 | [WRITTEN] | BC-DEC-005 |
| BC-2.02.006 | Decode Linux SLL (Cooked) TCP Packets | P0 | [WRITTEN] | BC-DEC-006 |
| BC-2.02.007 | Reject Malformed Input Bytes with anyhow Error (No Panic) | P0 | [WRITTEN] | BC-DEC-007 |
| BC-2.02.008 | Reject Unsupported Link Types in decode_packet | P1 | [WRITTEN] | BC-DEC-008 |
| BC-2.02.009 | Surface No IP Layer Found Error for Non-IP Frames | P1 | [WRITTEN] | BC-DEC-009 |
| BC-2.02.010 | Classify ICMP as Protocol::Icmp with TransportInfo::None | P1 | [WRITTEN] | BC-DEC-010 |
| BC-2.02.011 | Classify Other IP Protocols as Protocol::Other(byte) | P1 | [WRITTEN] | BC-DEC-011 |
| BC-2.02.012 | app_protocol_hint Returns Service Strings from Port Number | P1 | [WRITTEN] | BC-DEC-012 |
| BC-2.02.013 | app_protocol_hint Returns None When TransportInfo is None | P2 | [WRITTEN] | BC-DEC-013 |
| BC-2.02.014 | packet_len is Set to Total Frame Length, Not Just Payload Length | P1 | [WRITTEN] | BC-DEC-014 |
| BC-2.02.015 | Extract TCP Control Flags and Sequence Number into TransportInfo::Tcp | P0 | [WRITTEN] | BC-DEC-015 |

## ss-04: TCP Stream Reassembly (CAP-04)

> 55 BCs total; 55 fully written; 0 planned.

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.04.001 | TcpReassembler::new Panics on Invalid Config | P1 | [WRITTEN] | BC-RAS-001 |
| BC-2.04.002 | Non-TCP Packets Skipped; packets_skipped_non_tcp Increments | P1 | [WRITTEN] | BC-RAS-002 |
| BC-2.04.003 | Canonical FlowKey Ordering Ensures A->B and B->A Produce Identical Key | P0 | [WRITTEN] | BC-RAS-003 |
| BC-2.04.004 | First SYN Sets Client ISN and Initiator | P0 | [WRITTEN] | BC-RAS-004 |
| BC-2.04.005 | SYN+ACK Marks Server as Responder; State Transitions to Established | P0 | [WRITTEN] | BC-RAS-005 |
| BC-2.04.006 | Bidirectional Data Delivered with Correct Direction Tag | P0 | [WRITTEN] | BC-RAS-006 |
| BC-2.04.007 | In-Order Data Flushes Contiguously to Handler | P0 | [WRITTEN] | BC-RAS-007 |
| BC-2.04.008 | Out-of-Order Segments Buffer Until Gap Filled Then Flush | P0 | [WRITTEN] | BC-RAS-008 |
| BC-2.04.009 | Mid-Stream Join Infers ISN from seq-1; Flow Marked Partial | P0 | [WRITTEN] | BC-RAS-009 |
| BC-2.04.010 | RST Closes Flow Immediately with CloseReason::Rst | P0 | [WRITTEN] | BC-RAS-010 |
| BC-2.04.011 | Both FINs Close Flow with CloseReason::Fin | P0 | [WRITTEN] | BC-RAS-011 |
| BC-2.04.012 | finalize Flushes All Remaining Flows; Idempotent | P0 | [WRITTEN] | BC-RAS-012 |
| BC-2.04.013 | expire_idle_by_timeout / expire_flows Closes Idle Flows Past flow_timeout_secs | P1 | [WRITTEN] | BC-RAS-013 |
| BC-2.04.014 | total_memory Tracks Buffered Bytes; Decrements on Flush and Close | P1 | [WRITTEN] | BC-RAS-014 |
| BC-2.04.015 | Flow Eviction on max_flows Hit Uses LRU Non-Established-First | P1 | [WRITTEN] | BC-RAS-015 |
| BC-2.04.016 | Memory Pressure Eviction When total_memory Exceeds memcap | P1 | [WRITTEN] | BC-RAS-016 |
| BC-2.04.017 | Eviction Sort -- Non-Established First, Then Oldest-Last-Seen | P1 | [WRITTEN] | BC-RAS-017 |
| BC-2.04.018 | Conflicting Overlap Emits Anomaly/Likely/High Finding with MITRE T1036 | P0 | [WRITTEN] | BC-RAS-018 |
| BC-2.04.019 | Excessive Overlaps Emit One-Shot T1036 Finding | P0 | [WRITTEN] | BC-RAS-019 |
| BC-2.04.020 | Excessive Small Segments Emit One-Shot Finding | P1 | [WRITTEN] | BC-RAS-020 |
| BC-2.04.021 | Excessive Out-of-Window Segments Emit One-Shot Low Finding | P1 | [WRITTEN] | BC-RAS-021 |
| BC-2.04.022 | Per-Direction Alert Fires At Most Once Per Flow (Sticky Latch) | P0 | [WRITTEN] | BC-RAS-022 |
| BC-2.04.023 | Truncated Segment Emits Anomaly/Inconclusive/Low Finding | P1 | [WRITTEN] | BC-RAS-023 |
| BC-2.04.024 | Total Findings Capped at MAX_FINDINGS=10000; Excess Silently Dropped | P0 | [WRITTEN] | BC-RAS-024 |
| BC-2.04.025 | finalize Emits Segment-Limit Summary Finding When Segments Dropped | P0 | [WRITTEN] | BC-RAS-025 |
| BC-2.04.026 | finalize Does NOT Emit Segment-Limit Finding When Counter is Zero | P0 | [WRITTEN] | BC-RAS-026 |
| BC-2.04.027 | segments_depth_exceeded Tracks Fully-Rejected Segments After Depth Hit | P1 | [WRITTEN] | BC-RAS-027 |
| BC-2.04.028 | summarize Returns AnalysisSummary with Reassembly Stats Detail Map | P1 | [WRITTEN] | BC-RAS-028 |
| BC-2.04.029 | close_flow for Missing Key Logs One-Shot Process-Wide Warning | P2 | [WRITTEN] | BC-RAS-029 |
| BC-2.04.030 | bytes_reassembled Equals Total Bytes Delivered to Handler | P1 | [WRITTEN] | BC-RAS-030 |
| BC-2.04.031 | ISN Set on First SYN; Inferred as seq-1 on Data-Without-SYN | P0 | [WRITTEN] | BC-RAS-031 |
| BC-2.04.032 | insert_segment With No ISN Returns IsnMissing; Inserts Nothing | P0 | [WRITTEN] | BC-RAS-032 |
| BC-2.04.033 | Single Segment Insertion Returns Inserted; Stored Under Offset Key | P0 | [WRITTEN] | BC-RAS-033 |
| BC-2.04.034 | flush_contiguous Consumes Segments from base_offset in Order | P0 | [WRITTEN] | BC-RAS-034 |
| BC-2.04.035 | Identical Retransmission Returns Duplicate; Does Not Double-Count | P0 | [WRITTEN] | BC-RAS-035 |
| BC-2.04.036 | First-Wins Overlap: Gap Bytes Added, Existing Bytes Preserved | P0 | [WRITTEN] | BC-RAS-036 |
| BC-2.04.037 | Same-Range Conflicting Overlap Returns ConflictingOverlap; Original Wins | P0 | [WRITTEN] | BC-RAS-037 |
| BC-2.04.038 | Multi-Segment Full Coverage Returns Duplicate or ConflictingOverlap | P0 | [WRITTEN] | BC-RAS-038 |
| BC-2.04.039 | TCP Sequence Wraparound Across 32-bit Boundary Reassembles Correctly | P0 | [WRITTEN] | BC-RAS-039 |
| BC-2.04.040 | Small-Segment Counter Increments Per Direction | P1 | [WRITTEN] | BC-RAS-040 |
| BC-2.04.041 | Depth Truncation: Segment Crossing max_depth is Truncated | P0 | [WRITTEN] | BC-RAS-041 |
| BC-2.04.042 | Segment Beyond max_receive_window Returns OutOfWindow | P1 | [WRITTEN] | BC-RAS-042 |
| BC-2.04.043 | Adjacent Segments at Exact Boundary Do Not Count as Overlap | P0 | [WRITTEN] | BC-RAS-043 |
| BC-2.04.044 | Segments Map Full: Non-Overlapping Insert Returns SegmentLimitReached | P0 | [WRITTEN] | BC-RAS-044 |
| BC-2.04.045 | Segments Map Full: Overlapping Insert Returns SegmentLimitReached | P0 | [WRITTEN] | BC-RAS-045 |
| BC-2.04.046 | Segments Map Fills Mid-Loop: Partial Insertion | P0 | [WRITTEN] | BC-RAS-046 |
| BC-2.04.047 | buffered_bytes Mirrors Segment Size Sum After All Operations | P0 | [WRITTEN] | BC-RAS-047 |
| BC-2.04.048 | ISN_MISSING_WARNED Atomic Prevents Repeated eprintln | P2 | [WRITTEN] | BC-RAS-048 |
| BC-2.04.049 | FlowKey::Display Uses U+2192 Arrow (Not ASCII ->) | P1 | [WRITTEN] | BC-RAS-049 |
| BC-2.04.050 | Flow State Machine: New->SynSent->Established->Closing->Closed | P0 | [WRITTEN] | BC-RAS-050 |
| BC-2.04.051 | RST Transitions State to Closed from Any Prior State | P0 | [WRITTEN] | BC-RAS-051 |
| BC-2.04.052 | on_data_without_syn: New->Established; partial=true | P0 | [WRITTEN] | BC-RAS-052 |
| BC-2.04.053 | TcpFlow::direction Returns ClientToServer When src Matches Initiator | P0 | [WRITTEN] | BC-RAS-053 |
| BC-2.04.054 | finalize Unconditionally Bypasses MAX_FINDINGS Cap for Segment-Limit Finding | P0 | [WRITTEN] | BC-RAS-054 |
| BC-2.04.055 | StreamHandler::on_data Carries Capture-Relative Timestamp Parameter | P1 | [WRITTEN] | feature-100-F2 |

## ss-05: Content-First Protocol Dispatch (CAP-05)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.05.001 | TLS Content Signature Routes Flow to TLS Regardless of Port | P0 | [WRITTEN] | BC-DSP-001 |
| BC-2.05.002 | HTTP Method Prefix Routes Flow to HTTP | P0 | [WRITTEN] | BC-DSP-002 |
| BC-2.05.003 | Port Fallback: 443/8443->TLS, 80/8080->HTTP When Content Insufficient | P0 | [WRITTEN] | BC-DSP-003 |
| BC-2.05.004 | Unknown Content + Unknown Port Returns DispatchTarget::None | P1 | [WRITTEN] | BC-DSP-004 |
| BC-2.05.005 | Classification Cached Per FlowKey After First Non-None Result | P0 | [WRITTEN] | BC-DSP-005 |
| BC-2.05.006 | DispatchTarget::None NOT Cached Until Retry Cap; Reclassification Retried Until Cap Then Cached Permanently | P0 | [WRITTEN] | BC-DSP-006 |
| BC-2.05.007 | unclassified_flows Increments Only at on_flow_close | P1 | [WRITTEN] | BC-DSP-007 |
| BC-2.05.008 | No Analyzer Configured: Dispatcher Early-Returns | P1 | [WRITTEN] | BC-DSP-008 |
| BC-2.05.009 | on_flow_close Removes Route Entry and Forwards Close | P0 | [WRITTEN] | BC-DSP-009 |

## ss-06: HTTP Traffic Analysis (CAP-06)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.06.001 | Parse Complete HTTP/1.1 Request with Method/URI/Version/Host/UA | P0 | [WRITTEN] | BC-HTTP-001 |
| BC-2.06.002 | Parse Pipelined Requests with Independent Per-Request Counting | P0 | [WRITTEN] | BC-HTTP-002 |
| BC-2.06.003 | Partial Requests Buffered Until Complete; Not Counted Until Full | P0 | [WRITTEN] | BC-HTTP-003 |
| BC-2.06.004 | Parse HTTP/1.1 Responses with Status Code Counting | P0 | [WRITTEN] | BC-HTTP-004 |
| BC-2.06.005 | Path Traversal in URI Emits Reconnaissance/Likely/High Finding Mapped to T1083 | P0 | [WRITTEN] | BC-HTTP-005 |
| BC-2.06.006 | Web-Shell URI Patterns Emit Execution/Likely/Medium Finding (T1505.003) | P0 | [WRITTEN] | BC-HTTP-006 |
| BC-2.06.007 | Admin Panel Paths Emit Reconnaissance/Inconclusive/Low Finding (T1046) | P1 | [WRITTEN] | BC-HTTP-007 |
| BC-2.06.008 | Unusual HTTP Methods Emit Reconnaissance/Inconclusive/Medium Finding | P1 | [WRITTEN] | BC-HTTP-008 |
| BC-2.06.009 | HTTP/1.1 Missing or Empty Host Emits Anomaly/Inconclusive/Medium Finding | P0 | [WRITTEN] | BC-HTTP-009 |
| BC-2.06.010 | URI Greater Than 2048 Chars Emits Execution/Likely/Medium Finding | P1 | [WRITTEN] | BC-HTTP-010 |
| BC-2.06.011 | Empty UA Emits Anomaly/Inconclusive/Low; Absent UA Does NOT | P1 | [WRITTEN] | BC-HTTP-011 |
| BC-2.06.012 | Well-Formed HTTP Request Produces Zero Findings | P0 | [WRITTEN] | BC-HTTP-012 |
| BC-2.06.013 | Non-HTTP Bytes Increment parse_errors; No Token-Error Findings | P0 | [WRITTEN] | BC-HTTP-013 |
| BC-2.06.014 | Too Many Headers Emits Anomaly/Inconclusive/Medium Finding (T1499.002) | P0 | [WRITTEN] | BC-HTTP-014 |
| BC-2.06.015 | After 3 Consecutive Parse Errors a Direction is Poisoned; Subsequent Bytes Skipped | P0 | [WRITTEN] | BC-HTTP-015 |
| BC-2.06.016 | Single Parse Error Does NOT Poison | P0 | [WRITTEN] | BC-HTTP-016 |
| BC-2.06.017 | Poisoning is Per-Direction; Poisoned Request Does Not Affect Response | P0 | [WRITTEN] | BC-HTTP-017 |
| BC-2.06.018 | non_http_flows Counts Flow Once Even if Both Directions Poisoned | P1 | [WRITTEN] | BC-HTTP-018 |
| BC-2.06.019 | on_flow_close Removes Per-Flow State; Reopening Same Key Starts Fresh | P0 | [WRITTEN] | BC-HTTP-019 |
| BC-2.06.020 | HTTP Body Bytes After Header Completion Do Not Inflate parse_errors | P1 | [WRITTEN] | BC-HTTP-020 |
| BC-2.06.021 | Cross-Flow Isolation: Errors and Poisoning Do Not Leak | P0 | [WRITTEN] | BC-HTTP-021 |
| BC-2.06.022 | Per-Direction Header Buffer Capped at MAX_HEADER_BUF (65536) | P1 | [WRITTEN] | BC-HTTP-022 |
| BC-2.06.023 | summarize Emits AnalysisSummary with HTTP Stats Detail Map | P1 | [WRITTEN] | BC-HTTP-023 |
| BC-2.06.024 | Per-Map Cardinality Cap: New Keys Dropped Past MAX_MAP_ENTRIES | P2 | [WRITTEN] | BC-HTTP-024 |
| BC-2.06.025 | uris List Capped at MAX_URIS=10000 | P2 | [WRITTEN] | BC-HTTP-025 |
| BC-2.06.026 | Header Values Extracted via from_utf8_lossy.trim(); Raw Bytes Preserved | P0 | [WRITTEN] | BC-HTTP-026 |

## ss-07: TLS Traffic Analysis (CAP-07)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.07.001 | Parse Complete TLS ClientHello: Version, Ciphers, Extensions, SNI, JA3 | P0 | [WRITTEN] | BC-TLS-001 |
| BC-2.07.002 | Parse Complete TLS ServerHello: JA3S Fingerprint Computed | P0 | [WRITTEN] | BC-TLS-002 |
| BC-2.07.003 | After Both Hellos Seen, Subsequent Records Are Silently Skipped | P0 | [WRITTEN] | BC-TLS-003 |
| BC-2.07.004 | TLS Record Payload > MAX_RECORD_PAYLOAD Increments parse_errors and truncated_records | P0 | [WRITTEN] | BC-TLS-004 |
| BC-2.07.005 | Per-Direction Buffer Capped at MAX_BUF = 65536 Bytes | P1 | [WRITTEN] | BC-TLS-005 |
| BC-2.07.006 | JA3 Computation Filters GREASE Values per RFC 8701 | P0 | [WRITTEN] | BC-TLS-006 |
| BC-2.07.007 | JA3 String Format: version,ciphers,...; MD5 Hex | P0 | [WRITTEN] | BC-TLS-007 |
| BC-2.07.008 | JA3S String Format: version,cipher,extensions; MD5 Hex | P0 | [WRITTEN] | BC-TLS-008 |
| BC-2.07.009 | Weak Client Cipher in ClientHello Emits Anomaly/Likely/High Finding | P0 | [WRITTEN] | BC-TLS-009 |
| BC-2.07.010 | Weak Server Cipher Selected Emits Anomaly/Likely/Medium Finding | P0 | [WRITTEN] | BC-TLS-010 |
| BC-2.07.011 | Deprecated Client Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding | P0 | [WRITTEN] | BC-TLS-011 |
| BC-2.07.012 | Deprecated Server Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding | P0 | [WRITTEN] | BC-TLS-012 |
| BC-2.07.013 | Clean ASCII SNI Produces No Finding | P0 | [WRITTEN] | BC-TLS-013 |
| BC-2.07.014 | SNI Containing C0/DEL Byte Emits Anomaly/Inconclusive/Low Finding Mapped to T1027 | P0 | [WRITTEN] | BC-TLS-014 |
| BC-2.07.015 | Multiple Control Bytes in One SNI Produce Exactly ONE Finding | P0 | [WRITTEN] | BC-TLS-015 |
| BC-2.07.016 | C0 Boundary: 0x1F Trips Finding; 0x20 (Space) Does NOT | P0 | [WRITTEN] | BC-TLS-016 |
| BC-2.07.017 | Non-ASCII UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027) | P0 | [WRITTEN] | BC-TLS-017 |
| BC-2.07.018 | Punycode A-label is Pure ASCII; Emits No SNI Finding | P1 | [WRITTEN] | BC-TLS-018 |
| BC-2.07.019 | Non-UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027); Count Key Tagged | P0 | [WRITTEN] | BC-TLS-019 |
| BC-2.07.020 | Non-UTF-8 SNI Preserves Raw Bytes per ADR 0003 | P0 | [WRITTEN] | BC-TLS-020 |
| BC-2.07.021 | Non-ASCII UTF-8 SNI Preserves Raw Bytes per ADR 0003 | P0 | [WRITTEN] | BC-TLS-021 |
| BC-2.07.022 | Empty SNI ServerNameList: No Count, No Finding, Handshake Counted | P1 | [WRITTEN] | BC-TLS-022 |
| BC-2.07.023 | Empty SNI Hostname Bytes Counted Under "" Key; No Finding | P2 | [WRITTEN] | BC-TLS-023 |
| BC-2.07.024 | Only FIRST ServerName Entry Processed | P1 | [WRITTEN] | BC-TLS-024 |
| BC-2.07.025 | Non-Zero NameType Entries Treated as Hostnames | P2 | [WRITTEN] | BC-TLS-025 |
| BC-2.07.026 | Trailing Bytes in ServerNameList Tolerated | P2 | [WRITTEN] | BC-TLS-026 |
| BC-2.07.027 | Large SNI (16 KB) Under MAX_RECORD_PAYLOAD Parses Successfully | P1 | [WRITTEN] | BC-TLS-027 |
| BC-2.07.028 | sni_counts Cap: Finding Still Fires When Map at Capacity | P0 | [WRITTEN] | BC-TLS-028 |
| BC-2.07.029 | Bad TLS Record Body Increments parse_errors; No Panic | P0 | [WRITTEN] | BC-TLS-029 |
| BC-2.07.030 | Normal Handshake with Strong Cipher Produces Zero Findings | P0 | [WRITTEN] | BC-TLS-030 |
| BC-2.07.031 | summarize Emits AnalysisSummary with TLS Stats Detail Map | P1 | [WRITTEN] | BC-TLS-031 |
| BC-2.07.032 | TLS 1.3 ClientHello legacy_version Recorded as 0x0303 | P1 | [WRITTEN] | BC-TLS-032 |
| BC-2.07.033 | TLS Analyzer Ignores Non-Handshake Records | P1 | [WRITTEN] | BC-TLS-033 |
| BC-2.07.034 | After Both Hellos Seen, on_data Short-Circuits | P0 | [WRITTEN] | BC-TLS-034 |
| BC-2.07.035 | on_flow_close Drops Per-Flow TlsFlowState | P1 | [WRITTEN] | BC-TLS-035 |
| BC-2.07.036 | Unknown Cipher IDs Render as Hex 0xNNNN Lowercase | P2 | [WRITTEN] | BC-TLS-036 |
| BC-2.07.037 | SNI with Both Non-ASCII and C0 Control Bytes Fires Arm 3 (NonAsciiUtf8), Not Arm 2 | P0 | [WRITTEN] | BC-TLS-037 |

## ss-08: DNS Traffic Analysis (CAP-08)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.08.001 | DnsAnalyzer Matches Packets Where Port == 53 (TCP or UDP) | P0 | [WRITTEN] | BC-DNS-001 |
| BC-2.08.002 | DNS QR-Bit Dispatch: response_count Incremented if Set; query_count Otherwise | P0 | [WRITTEN] | BC-DNS-002 |
| BC-2.08.003 | summarize Emits AnalysisSummary with dns_queries and dns_responses | P1 | [WRITTEN] | BC-DNS-003 |
| BC-2.08.004 | DnsAnalyzer NEVER Emits Findings (Statistics-Only by Design) | P0 | [WRITTEN] | BC-DNS-004 |

## ss-09: Forensic Finding Emission (CAP-09)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.09.001 | Finding Constructed with Required Fields and Optional Fields | P0 | [WRITTEN] | BC-FND-001 | <!-- v1.4: mitre_technique->mitre_techniques Vec<String>; ADR-006 F2 revision -->
| BC-2.09.002 | Finding Display Renders [Category] VERDICT (CONFIDENCE) — summary | P1 | [WRITTEN] | BC-FND-002 |
| BC-2.09.003 | Verdict Display: Uppercase Tokens | P1 | [WRITTEN] | BC-FND-003 |
| BC-2.09.004 | Confidence Display: Uppercase Tokens | P1 | [WRITTEN] | BC-FND-004 |
| BC-2.09.005 | Finding.summary and Evidence Store RAW Post-from_utf8_lossy Bytes per ADR 0003 | P0 | [WRITTEN] | BC-FND-005 |
| BC-2.09.006 | Finding JSON Serialization: Empty Vec Fields Omitted; mitre_techniques Serialized as Array | P0 | [WRITTEN] | BC-FND-006 | <!-- v1.5: mitre_techniques Vec; skip_serializing_if Vec::is_empty; ADR-006 F2 revision -->
| BC-2.09.007 | Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site | P1 | [WRITTEN] | feature-100-F2 |

## ss-10: MITRE ATT&CK Mapping (CAP-10)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.10.001 | MitreTactic Display Renders Enterprise Tactics with Canonical Spacing | P0 | [WRITTEN] | BC-MIT-001 |
| BC-2.10.002 | ICS Tactics Render Unprefixed | P1 | [WRITTEN] | BC-MIT-002 |
| BC-2.10.003 | all_tactics_in_report_order Returns Kill-Chain Order First Then ICS | P0 | [WRITTEN] | BC-MIT-003 |
| BC-2.10.004 | all_tactics_in_report_order Contains Every Variant Exactly Once | P0 | [WRITTEN] | BC-MIT-004 |
| BC-2.10.005 | technique_name Returns Some for Every Seeded ID (21 Total) | P0 | [WRITTEN] | BC-MIT-005 | <!-- v1.4: count 15->21; T0888+5 new ICS added; ADR-006 / Decision-12 F2 revision -->
| BC-2.10.006 | technique_name Returns None for Unknown IDs | P0 | [WRITTEN] | BC-MIT-006 |
| BC-2.10.007 | technique_tactic Returns Correct Tactic for Every Seeded ID | P0 | [WRITTEN] | BC-MIT-007 |
| BC-2.10.008 | All Emitted Technique IDs Resolve in Lookup | P0 | [WRITTEN] | BC-MIT-008 | <!-- v1.3: grep pattern mitre_technique:Some->mitre_techniques:vec!; T0888 replaces T0846 in emitted list; 13 total emitted; ADR-006 / Decision-12 F2 revision -->
| BC-2.10.009 | MitreTactic is #[non_exhaustive] | P2 | [WRITTEN] | BC-MIT-009 |

## ss-11: Reporting and Output (CAP-11)

> 24 BCs total; 24 fully written; 0 planned.
> BCs 001-019: JsonReporter / TerminalReporter / MITRE grouping (brownfield ingestion).
> BCs 020-024: CsvReporter (added pass-4, adversarial finding H-1).

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.11.001 | JsonReporter Renders JSON Object with summary/findings/analyzers Keys | P0 | [WRITTEN] | BC-RPT-001 |
| BC-2.11.002 | JsonReporter Includes skipped_packets in Summary | P1 | [WRITTEN] | BC-RPT-002 |
| BC-2.11.003 | JsonReporter Escapes C0 Control Bytes per RFC 8259 via serde | P0 | [WRITTEN] | BC-RPT-003 |
| BC-2.11.004 | JsonReporter Preserves Non-ASCII Unicode in Readable Form | P1 | [WRITTEN] | BC-RPT-004 |
| BC-2.11.005 | JsonReporter Passes C1 Codepoints Through as Raw UTF-8 | P1 | [WRITTEN] | BC-RPT-005 |
| BC-2.11.006 | TerminalReporter Shows Skipped: N Packets Only When N > 0 | P1 | [WRITTEN] | BC-RPT-006 |
| BC-2.11.007 | TerminalReporter Escapes C0+DEL+C1+Backslash in Finding Summary and Evidence | P0 | [WRITTEN] | BC-RPT-007 |
| BC-2.11.008 | TerminalReporter Escape Preserves Printable ASCII and UTF-8 | P0 | [WRITTEN] | BC-RPT-008 |
| BC-2.11.009 | TerminalReporter Escapes C1 Codepoints U+0080-U+009F; U+00A0 Preserved | P0 | [WRITTEN] | BC-RPT-009 |
| BC-2.11.010 | TerminalReporter Escapes Both Summary AND Each Evidence Line | P0 | [WRITTEN] | BC-RPT-010 |
| BC-2.11.011 | TerminalReporter Escapes Analyzer-Summary Detail Values | P0 | [WRITTEN] | BC-RPT-011 |
| BC-2.11.012 | TerminalReporter End-to-End: C1 CSI in Path-Traversal Finding Escaped | P0 | [WRITTEN] | BC-RPT-012 |
| BC-2.11.013 | MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last | P0 | [WRITTEN] | BC-RPT-013 |
| BC-2.11.014 | Within Tactic Bucket: Sort by Verdict, Confidence, Emission Order | P1 | [WRITTEN] | BC-RPT-014 |
| BC-2.11.015 | No-Technique or Unknown-ID Findings Land in Uncategorized | P0 | [WRITTEN] | BC-RPT-015 |
| BC-2.11.016 | MITRE Grouping Expands Per-Finding Line with Em-Dash and Name | P1 | [WRITTEN] | BC-RPT-016 |
| BC-2.11.017 | Default Rendering Emits MITRE: <id(s)> Only (No Em-Dash) | P1 | [WRITTEN] | BC-RPT-017 | <!-- v1.5: multi-ID rendering "MITRE: T0855, T0836"; ADR-006 F2 revision -->
| BC-2.11.018 | TerminalReporter Colorization: Likely/High=Red Bold, etc. | P2 | [WRITTEN] | BC-RPT-018 |
| BC-2.11.019 | TerminalReporter Renders Sections in Correct Order | P1 | [WRITTEN] | BC-RPT-019 |
| BC-2.11.020 | CsvReporter Emits Exactly Nine Columns in Fixed Header Order | P0 | [WRITTEN] | pass-4 H-1 | <!-- v1.5: column-6 header renamed mitre_technique->mitre_techniques; ADR-006 F2 revision -->
| BC-2.11.021 | CsvReporter Neutralizes CSV-Injection Trigger Characters with a Leading Single Quote | P0 | [WRITTEN] | pass-4 H-1 |
| BC-2.11.022 | CsvReporter Joins Evidence Vec Elements with "; " into a Single Cell | P1 | [WRITTEN] | pass-4 H-1 |
| BC-2.11.023 | CsvReporter Implements Reporter Trait and Emits One Row per Finding; Summary and AnalysisSummary Are Ignored | P0 | [WRITTEN] | pass-4 H-1 |
| BC-2.11.024 | CsvReporter Encodes Optional Fields as Empty Strings and mitre_techniques as Semicolon-Joined String | P1 | [WRITTEN] | pass-4 H-1 | <!-- v1.4: mitre_technique None->mitre_techniques vec![]; semicolon-join for multi-tag; ADR-006 F2 revision -->

## ss-12: CLI and Entry Point (Cross-Cutting)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.12.001 | analyze Subcommand Parses Positional Targets and All Flags | P0 | [WRITTEN] | BC-CLI-001 |
| BC-2.12.002 | summary Subcommand Parses Targets and --hosts Flag | P1 | [WRITTEN] | BC-CLI-002 |
| BC-2.12.003 | Global Flag --no-color Parsed and Stored | P1 | [WRITTEN] | BC-CLI-003 |
| BC-2.12.004 | --output-format json Parses to Some(OutputFormat::Json) | P0 | [WRITTEN] | BC-CLI-004 |
| BC-2.12.005 | Reassembly CLI Flags: --reassemble/--no-reassemble, depth, memcap, and five anomaly-threshold flags | P0 | [WRITTEN] | BC-CLI-005 |
| BC-2.12.006 | Multiple Positional Targets Accepted in analyze | P1 | [WRITTEN] | BC-CLI-006 |
| BC-2.12.007 | --reassemble and --no-reassemble are Mutually Exclusive (clap conflicts_with) | P0 | [WRITTEN] | BC-CLI-007 |
| BC-2.12.008 | --all Enables dns/http/tls Together | P1 | [WRITTEN] | BC-CLI-008 |
| BC-2.12.009 | needs_reassembly Logic; --no-reassemble Forces Off with Warning | P0 | [WRITTEN] | BC-CLI-009 |
| BC-2.12.010 | NO_COLOR Env Var Disables Color | P2 | [WRITTEN] | BC-CLI-010 |
| BC-2.12.011 | Directory Target Expands to *.pcap Sorted; *.pcapng Excluded | P1 | [WRITTEN] | BC-CLI-011 |
| BC-2.12.012 | Non-Existent Target Yields bail! with Target Not Found | P1 | [WRITTEN] | BC-CLI-012 |
| BC-2.12.013 | Per-Target Progress Bar on stderr via indicatif | P2 | [WRITTEN] | BC-CLI-013 |
| BC-2.12.014 | Per-Target Decode Errors Counted into skipped_packets | P1 | [WRITTEN] | BC-CLI-014 |
| BC-2.12.015 | dispatcher.unclassified_flows() Injected into Reassembly Summary | P1 | [WRITTEN] | BC-CLI-015 |
| BC-2.12.016 | Output Format Selection: json->JsonReporter, csv->CsvReporter, else Terminal | P0 | [WRITTEN] | BC-CLI-016 |
| BC-2.12.017 | Output Routed to File if --json/--csv <FILE>; Stdout Otherwise | P0 | [WRITTEN] | BC-CLI-017 |
| BC-2.12.018 | Summary::ingest Increments total_packets, total_bytes, hosts, protocols | P0 | [WRITTEN] | BC-SUM-001 |
| BC-2.12.019 | Summary::ingest Derives Service Name from app_protocol_hint | P1 | [WRITTEN] | BC-SUM-002 |
| BC-2.12.020 | Summary::unique_hosts Returns Sorted Deduplicated Vec<IpAddr> | P1 | [WRITTEN] | BC-SUM-003 |
| BC-2.12.021 | Summary Serializes with total_packets/total_bytes/skipped_packets Fields | P1 | [WRITTEN] | BC-SUM-004 |

## ss-13: Absent / Unwired Feature Contracts

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.13.001 | --threats Flag Does Not Exist; clap Rejects It as Unknown Argument | P0 | [WRITTEN] | BC-ABS-001 |
| BC-2.13.002 | --beacon Flag Does Not Exist; No C2 Beacon Analyzer Exists | P0 | [WRITTEN] | BC-ABS-002 |
| BC-2.13.003 | --filter <BPF> Flag Does Not Exist; No BPF Filter Applied | P0 | [WRITTEN] | BC-ABS-003 |
| BC-2.13.004 | --verbose Flag Does Not Exist; No Verbose Logging Mode | P2 | [WRITTEN] | BC-ABS-010 |

## ss-14: Modbus/ICS Analysis (CAP-14)

> 25 BCs total; 25 fully written; 0 planned.
> BCs 001-004: MBAP Parse and Validity Gate (Group A).
> BCs 005-008: Function-Code Classification (Group B). BC-005 covers ALL 256 FC values (totality).
> BCs 009-012: Transaction Correlation (Group C).
> BCs 013-015: Finding Emission — Write-Class Events (Group D). **v2 co-emission model (ADR-006):** one multi-tag finding per write PDU; T0855 co-included in vec, not separate. No tag-suppression.
> BCs 016-017: Finding Emission — Coordinated Write (T0831 5s window, Group E) and **dual-window** Write-Burst Detection (T0806/T0855; burst 1s + sustained >=2s per Decision-11, Group E).
> BCs 018-019: Finding Emission — Diagnostic/DoS (T0814) (BC-018) and Exception Burst Anomaly (BC-019) (Group F).
> BCs 020-022: Anomaly/Recon (**T0888** for 0x11/0x2B/0x0E — Decision-12; T0846 NOT emitted by Modbus), Summary Stats (6 keys incl dropped_findings), and Bounded-Resource (Groups G + resource cap).
> BCs 023-025: Dispatcher and CLI Integration (Group H). **BC-024 v2:** two flags (--modbus-write-burst-threshold + --modbus-write-sustained-threshold); old --modbus-write-threshold removed.
> Feature: issue-007-modbus-analyzer; ADR-005; ADR-006; introduced v0.3.0-feature-007.
> **v2 revision (2026-06-09):** BCs 013-017, 020, 024 revised per f2-fix-directives.md v2 Decisions 11-13.

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.14.001 | MBAP Header Accepted for Well-Formed 8-Byte-Minimum ADU | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.002 | MBAP Header Rejected for ADU Shorter Than 8 Bytes | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.003 | MBAP Header Rejected When Protocol ID is Not 0x0000 | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.004 | MBAP Header Rejected When Length is Outside [2, 253] | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.005 | classify_fc Is Total Over All 256 FC Values (Covers Read, Write, Diagnostic, Exception, and Unknown Classes) | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.006 | Exception Response Detection — FC High Bit Set Identifies Exception and Recovers Original FC | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.007 | Write-Class FC Classification — State-Changing Function Codes Identified as Elevated-Risk | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.008 | Diagnostic-Class FC Classification and Sub-Function Dispatch (0x08 and 0x2B) | P1 | [WRITTEN] | feature-007-F2 |
| BC-2.14.009 | Request PDU Inserted into Per-Flow Pending Table Keyed on (Transaction ID, Unit ID) | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.010 | Response PDU Matched Against Pending Table and Entry Removed on FC Echo Match | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.011 | Exception Response PDU Attributed to Originating Request FC via Pending Table Lookup | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.012 | Pending Table Bounded to MAX_PENDING_TRANSACTIONS=256; New Requests Dropped (Not Evicting) When Full | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.013 | Write-Class FC in Request Direction Emits Multi-Tag Finding Carrying T0855 and Applicable Technique Tags | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: co-emission model; T0855 co-included in multi-tag vec; ADR-006 Decision-13 -->
| BC-2.14.014 | Write FC 0x06/0x10/0x16 in Request Direction Emits Finding Tagged ["T0855","T0836"] | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: single multi-tag finding replaces two separate findings; ADR-006 Decision-13 -->
| BC-2.14.015 | Write FC to Coil Output Only ({0x05, 0x0F}) Emits Finding Tagged ["T0855","T0835"] | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: single multi-tag finding; no suppression; ADR-006 Decision-13 -->
| BC-2.14.016 | Coordinated Write Sequence to Holding Registers Within 5-Second Window Tags the Per-PDU Finding with T0831 | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: T0831 is co-tagged inline on the per-PDU write finding (mitre_techniques: ["T0855","T0836","T0831"]); no separate T0831 Finding object; ADR-006 Decision-13 §13.5 -->
| BC-2.14.017 | Write-Rate Exceeding Either Burst or Sustained Threshold Emits T0806 + T0855 Finding | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: dual-window (1s burst / >=2s sustained); each fires at most once per window; ADR-006 Decision-11 -->
| BC-2.14.018 | Diagnostics FC 0x08 Sub-Function 0x0004 or 0x0001 Emits T0814 Denial of Service Finding | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.019 | Exception Response Anomaly — Burst of Exception Codes Emits Anomaly Finding for Recon/Scanning | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.020 | Reconnaissance Function Codes (0x11, 0x2B/0x0E) Emit T0888 Remote System Information Discovery Finding | P1 | [WRITTEN] | feature-007-F2 | <!-- v2.0: T0846->T0888 correctness fix; 0x07 excluded; Decision-12 -->
| BC-2.14.021 | summarize() Returns AnalysisSummary with Specified Per-Analyzer Summary Keys | P1 | [WRITTEN] | feature-007-F2 |
| BC-2.14.022 | MAX_FINDINGS Cap and Poison-Skip Behavior for ModbusAnalyzer | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.023 | --modbus CLI Flag Enables ModbusAnalyzer; --all Includes Modbus; Default-Off; Requires Stream Reassembly | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.024 | --modbus-write-burst-threshold and --modbus-write-sustained-threshold Configure Dual-Window Burst Detection | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: old --modbus-write-threshold removed; replaced by two flags; Decision-11 -->
| BC-2.14.025 | StreamDispatcher Classifies Port-502 Flows to DispatchTarget::Modbus as Rule 5 (After Content and TLS/HTTP Port Rules); Routes on_data and on_flow_close to ModbusAnalyzer | P0 | [WRITTEN] | feature-007-F2 |

---

## Ingestion-to-L3 Mapping Coverage

| Ingestion group | Count | Mapped to L3 |
|----------------|-------|--------------|
| BC-RDR-001..008 | 8 | BC-2.01.001..008 |
| BC-DEC-001..015 | 15 | BC-2.02.001..015 |
| BC-RAS-001..054 | 54 | BC-2.04.001..054 |
| BC-DSP-001..009 | 9 | BC-2.05.001..009 |
| BC-HTTP-001..026 | 26 | BC-2.06.001..026 |
| BC-TLS-001..037 | 37 | BC-2.07.001..037 |
| BC-DNS-001..004 | 4 | BC-2.08.001..004 |
| BC-FND-001..006 | 6 | BC-2.09.001..006 |
| BC-MIT-001..009 | 9 | BC-2.10.001..009 |
| BC-RPT-001..019 | 19 | BC-2.11.001..019 |
| pass-4 H-1 (CsvReporter) | 5 | BC-2.11.020..024 |
| BC-CLI-001..017 | 17 | BC-2.12.001..017 |
| BC-SUM-001..004 | 4 | BC-2.12.018..021 |
| BC-ABS-001..010 | 10 | BC-2.13.001..004 (6 ABS retired by remediation cycle) |
| feature-007-F2 Modbus/ICS (greenfield) | 25 | BC-2.14.001..025 |

**Total BCs: 244. Canonical derivation: 218 draft ingestion BCs produced − 6 retired (BC-ABS-004..009) = 212 active from ingestion; + 5 post-ingestion pass-4 additions (BC-2.11.020..024) = 217; + 2 Feature Mode F2 additions (BC-2.04.055, BC-2.09.007) for issue #100 = 219 active BCs; + 25 Feature Mode F2 additions (BC-2.14.001..025) for issue #7 Modbus/ICS analyzer = 244 active BCs. The mapping table above has 223 physical rows (218 ingestion-batch rows + 5 pass-4 rows) for pre-Modbus BCs; SS-14 adds 25 greenfield rows not in the ingestion batch.**

Note: BC-ABS-004 (--hosts unwired), BC-ABS-005 (--services unwired), BC-ABS-006 (--json
file unwired), BC-ABS-007 (CSV unwired), BC-ABS-009 (no e2e CLI tests) are RETIRED --
these were fixed by PRs #70, #74, #84, and others in the remediation cycle. They are not
represented as L3 BCs because they are no longer absent behaviors.

BC-ABS-008 (rayon declared but unused) is RETIRED as an absent-behavior contract because
the concern is reclassified as open dependency debt, not a behavioral gap. As of develop
HEAD, `rayon = "1"` remains in Cargo.toml:28 with zero call sites in src/. This is tracked
as domain-debt item O-07 and is not a behavioral contract violation. BC-ABS-008 is not
represented as an active L3 BC; O-07 is the living tracker for this item.

The remaining 4 absent behaviors (BC-2.13.001..004) describe features that remain
intentionally unwired.
