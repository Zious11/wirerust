---
document_type: bc-index
level: L3
version: "1.41"
status: draft
producer: product-owner
timestamp: 2026-06-17T00:00:00Z
phase: 1a
traces_to: .factory/specs/prd.md
---

# wirerust Behavioral Contracts Index

> **Navigation:** This file is the master index of all BC-S.SS.NNN contracts. Each entry
> links to the individual BC file. BCs are sharded into per-subsystem directories (ss-NN/).
>
> All BCs are marked [WRITTEN]. Body files have been verified on disk for all 288 entries (283 prior + 5 new BC-2.11.025–029 for issue #259).
> 218 draft ingestion BCs were produced; 6 were retired during the remediation cycle (BC-ABS-004
> through BC-ABS-009) leaving 212 active L3 BCs from ingestion. BC-2.11.020 through BC-2.11.024
> were added in adversarial-review pass-4 (finding H-1: CsvReporter coverage gap), bringing
> the total to 217 active L3 BCs. BC-2.04.055 and BC-2.09.007 were added in Feature Mode F2
> (issue #100 pcap-timestamps delta) bringing the total to 219 active L3 BCs. BC-2.14.001
> through BC-2.14.025 were added in Feature Mode F2 (issue #7 Modbus/ICS analyzer) bringing
> the total to 244 active L3 BCs. BC-2.15.001 through BC-2.15.022 were added in Feature Mode
> F2 (issue #8 DNP3/ICS analyzer) bringing the total to 266 active L3 BCs. BC-2.15.023 and
> BC-2.15.024 were added in Feature Mode F2 (issue #8 research must-adds: DISABLE_UNSOLICITED
> abuse + malformed/structural anomaly) bringing the total to 268 active L3 BCs. BC-2.02.009
> was revised to v1.6 in Feature Mode F2 (issue #9 ARP analyzer, ADR-008 Decision 1:
> three-way ARP/non-Ethernet-ARP/non-IP postcondition; not a new BC). BC-2.16.001 through
> BC-2.16.015 were added in Feature Mode F2 (issue #9 ARP security analyzer) bringing the
> total to 283 active L3 BCs.
>
> **Status as of Phase 1a (current):**
> - Fully written: 288 BCs (all body files verified on disk; 283 prior + 5 new BC-2.11.025–029 for issue #259 v0.8.0 collapse feature)
> - Remaining: 0 BCs
> - PRD index (prd.md): UPDATED -- all 288 L3 BC IDs are registered

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
| BC-2.02.009 | Non-IP Non-ARP Frames Return No-IP-Layer Error; ARP Frames Return DecodedFrame::Arp | P1 | [WRITTEN] | BC-DEC-009 | <!-- v1.6: F2 ARP delta (ADR-008 Decision 1): three-way postcondition; ARP→DecodedFrame::Arp, non-Ethernet/IPv4 ARP→Err("Non-Ethernet/IPv4 ARP frame"), non-IP non-ARP→Err("No IP layer found") -->
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
| BC-2.04.002 | Non-TCP Packets Skipped; packets_skipped_non_tcp Increments | P1 | [WRITTEN] | BC-RAS-002 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.003 | Canonical FlowKey Ordering Ensures A->B and B->A Produce Identical Key | P0 | [WRITTEN] | BC-RAS-003 |
| BC-2.04.004 | First SYN Sets Client ISN and Initiator | P0 | [WRITTEN] | BC-RAS-004 |
| BC-2.04.005 | SYN+ACK Marks Server as Responder; State Transitions to Established | P0 | [WRITTEN] | BC-RAS-005 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.006 | Bidirectional Data Delivered with Correct Direction Tag | P0 | [WRITTEN] | BC-RAS-006 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.007 | In-Order Data Flushes Contiguously to Handler | P0 | [WRITTEN] | BC-RAS-007 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.008 | Out-of-Order Segments Buffer Until Gap Filled Then Flush | P0 | [WRITTEN] | BC-RAS-008 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.009 | Mid-Stream Join Infers ISN from seq-1; Flow Marked Partial | P0 | [WRITTEN] | BC-RAS-009 |
| BC-2.04.010 | RST Closes Flow Immediately with CloseReason::Rst | P0 | [WRITTEN] | BC-RAS-010 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.011 | Both FINs Close Flow with CloseReason::Fin | P0 | [WRITTEN] | BC-RAS-011 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.012 | finalize Flushes All Remaining Flows; Idempotent | P0 | [WRITTEN] | BC-RAS-012 | <!-- PG-ARP-F2-007: ss-04 src re-anchor --> <!-- P20-B-01: v1.9→v2.0 latch line 618→647 -->
| BC-2.04.013 | expire_idle_by_timeout / expire_flows Closes Idle Flows Past flow_timeout_secs | P1 | [WRITTEN] | BC-RAS-013 | <!-- PG-ARP-F2-007: ss-04 src re-anchor --> <!-- P20-B-02: v1.8→v1.9 expire call-site :166-169→:168-171 -->
| BC-2.04.014 | total_memory Tracks Buffered Bytes; Decrements on Flush and Close | P1 | [WRITTEN] | BC-RAS-014 | <!-- PG-ARP-F2-007: ss-04 src re-anchor --> <!-- P20-B-03: v1.5→v1.6 lifecycle.rs:60→:66 close_flow -->
| BC-2.04.015 | Flow Eviction on max_flows Hit Uses LRU Non-Established-First | P1 | [WRITTEN] | BC-RAS-015 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.016 | Memory Pressure Eviction When total_memory Exceeds memcap | P1 | [WRITTEN] | BC-RAS-016 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.017 | Eviction Sort -- Non-Established First, Then Oldest-Last-Seen | P1 | [WRITTEN] | BC-RAS-017 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.018 | Conflicting Overlap Emits Anomaly/Likely/High Finding with MITRE T1036 | P0 | [WRITTEN] | BC-RAS-018 | <!-- v1.5: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B4 -->
| BC-2.04.019 | Excessive Overlaps Emit One-Shot T1036 Finding | P0 | [WRITTEN] | BC-RAS-019 | <!-- v1.7: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B4 -->
| BC-2.04.020 | Excessive Small Segments Emit One-Shot Finding | P1 | [WRITTEN] | BC-RAS-020 | <!-- v1.6: P19 B-09 anchor fix: mod.rs:486-517→:506-538; :385-399→:402-405 -->
| BC-2.04.021 | Excessive Out-of-Window Segments Emit One-Shot Low Finding | P1 | [WRITTEN] | BC-RAS-021 | <!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B4 -->
| BC-2.04.022 | Per-Direction Alert Fires At Most Once Per Flow (Sticky Latch) | P0 | [WRITTEN] | BC-RAS-022 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.023 | Truncated Segment Emits Anomaly/Inconclusive/Low Finding | P1 | [WRITTEN] | BC-RAS-023 | <!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B4 -->
| BC-2.04.024 | Total Findings Capped at MAX_FINDINGS=10000; Excess Silently Dropped | P0 | [WRITTEN] | BC-RAS-024 | <!-- v1.4: P19 B-09 anchor fix: MAX_FINDINGS mod.rs:54→:56; guards :461,495,524→:479,515,546 -->
| BC-2.04.025 | finalize Emits Segment-Limit Summary Finding When Segments Dropped | P0 | [WRITTEN] | BC-RAS-025 | <!-- v1.6: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B4 -->
| BC-2.04.026 | finalize Does NOT Emit Segment-Limit Finding When Counter is Zero | P0 | [WRITTEN] | BC-RAS-026 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.027 | segments_depth_exceeded Tracks Fully-Rejected Segments After Depth Hit | P1 | [WRITTEN] | BC-RAS-027 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.028 | summarize Returns AnalysisSummary with Reassembly Stats Detail Map | P1 | [WRITTEN] | BC-RAS-028 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.029 | close_flow for Missing Key Logs One-Shot Process-Wide Warning | P2 | [WRITTEN] | BC-RAS-029 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.030 | bytes_reassembled Equals Total Bytes Delivered to Handler | P1 | [WRITTEN] | BC-RAS-030 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.031 | ISN Set on First SYN; Inferred as seq-1 on Data-Without-SYN | P0 | [WRITTEN] | BC-RAS-031 |
| BC-2.04.032 | insert_segment With No ISN Returns IsnMissing; Inserts Nothing | P0 | [WRITTEN] | BC-RAS-032 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.033 | Single Segment Insertion Returns Inserted; Stored Under Offset Key | P0 | [WRITTEN] | BC-RAS-033 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.034 | flush_contiguous Consumes Segments from base_offset in Order | P0 | [WRITTEN] | BC-RAS-034 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.035 | Identical Retransmission Returns Duplicate; Does Not Double-Count | P0 | [WRITTEN] | BC-RAS-035 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.036 | First-Wins Overlap: Gap Bytes Added, Existing Bytes Preserved | P0 | [WRITTEN] | BC-RAS-036 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.037 | Same-Range Conflicting Overlap Returns ConflictingOverlap; Original Wins | P0 | [WRITTEN] | BC-RAS-037 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.038 | Multi-Segment Full Coverage Returns Duplicate or ConflictingOverlap | P0 | [WRITTEN] | BC-RAS-038 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.039 | TCP Sequence Wraparound Across 32-bit Boundary Reassembles Correctly | P0 | [WRITTEN] | BC-RAS-039 |
| BC-2.04.040 | Small-Segment Counter Increments Per Direction | P1 | [WRITTEN] | BC-RAS-040 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.041 | Depth Truncation: Segment Crossing max_depth is Truncated | P0 | [WRITTEN] | BC-RAS-041 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.042 | Segment Beyond max_receive_window Returns OutOfWindow | P1 | [WRITTEN] | BC-RAS-042 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.043 | Adjacent Segments at Exact Boundary Do Not Count as Overlap | P0 | [WRITTEN] | BC-RAS-043 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.044 | Segments Map Full: Non-Overlapping Insert Returns SegmentLimitReached | P0 | [WRITTEN] | BC-RAS-044 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.045 | Segments Map Full: Overlapping Insert Returns SegmentLimitReached | P0 | [WRITTEN] | BC-RAS-045 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.046 | Segments Map Fills Mid-Loop: Partial Insertion | P0 | [WRITTEN] | BC-RAS-046 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.047 | buffered_bytes Mirrors Segment Size Sum After All Operations | P0 | [WRITTEN] | BC-RAS-047 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.048 | ISN_MISSING_WARNED Atomic Prevents Repeated eprintln | P2 | [WRITTEN] | BC-RAS-048 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.049 | FlowKey::Display Uses U+2192 Arrow (Not ASCII ->) | P1 | [WRITTEN] | BC-RAS-049 |
| BC-2.04.050 | Flow State Machine: New->SynSent->Established->Closing->Closed | P0 | [WRITTEN] | BC-RAS-050 |
| BC-2.04.051 | RST Transitions State to Closed from Any Prior State | P0 | [WRITTEN] | BC-RAS-051 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.052 | on_data_without_syn: New->Established; partial=true | P0 | [WRITTEN] | BC-RAS-052 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.053 | TcpFlow::direction Returns ClientToServer When src Matches Initiator | P0 | [WRITTEN] | BC-RAS-053 |
| BC-2.04.054 | finalize Unconditionally Bypasses MAX_FINDINGS Cap for Segment-Limit Finding | P0 | [WRITTEN] | BC-RAS-054 | <!-- PG-ARP-F2-007: ss-04 src re-anchor -->
| BC-2.04.055 | StreamHandler::on_data Carries Capture-Relative Timestamp Parameter | P1 | [WRITTEN] | feature-100-F2 | <!-- v1.0.3: P19 B-07 anchor fix; http.rs:501→:524; tls.rs:771→:798 -->

## ss-05: Content-First Protocol Dispatch (CAP-05)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.05.001 | TLS Content Signature Routes Flow to TLS Regardless of Port | P0 | [WRITTEN] | BC-DSP-001 | <!-- v1.7: Pass-18 B-01/B-02 anchor re-sync; fn classify :90→:184, TLS check :92-94→:186-187 -->
| BC-2.05.002 | HTTP Method Prefix Routes Flow to HTTP | P0 | [WRITTEN] | BC-DSP-002 | <!-- v1.6: Pass-18 B-01/B-02 anchor re-sync; HTTP method block :95-107→:190-202 -->
| BC-2.05.003 | Port Fallback: 443/8443->TLS, 80/8080->HTTP When Content Insufficient | P0 | [WRITTEN] | BC-DSP-003 | <!-- v1.7: Pass-18 B-01/B-02 anchor re-sync; port fallback :108-116→:204-212 -->
| BC-2.05.004 | Unknown Content + Unknown Port Returns DispatchTarget::None | P1 | [WRITTEN] | BC-DSP-004 | <!-- v1.5: Pass-18 B-01/B-02 anchor re-sync; None return :116→:241, classify call :136→:272, None branch :137-148→:273-284, non-None :149-151→:286-287 -->
| BC-2.05.005 | Classification Cached Per FlowKey After First Non-None Result | P0 | [WRITTEN] | BC-DSP-005 | <!-- v1.5: Pass-18 B-01/B-02 anchor re-sync; cache block :133-154→:269-290, non-None insert :149-151→:286-287 -->
| BC-2.05.006 | DispatchTarget::None NOT Cached Until Retry Cap; Reclassification Retried Until Cap Then Cached Permanently | P0 | [WRITTEN] | BC-DSP-006 | <!-- v1.5: Pass-18 B-01/B-02 anchor re-sync (most heavily stale); MAX const :40→:58, cache+retry :133-154→:269-290, None branch :137-148→:273-284, perm-insert :146→:282, attempts-remove :147→:283, non-None :149-151→:286-287, flow-close removes :175-176→:326-327 -->
| BC-2.05.007 | unclassified_flows Increments Only at on_flow_close | P1 | [WRITTEN] | BC-DSP-007 | <!-- v1.4: Pass-18 B-01/B-02/B-03 anchor re-sync + four-analyzer guard prose; on_flow_close :171-194→:322-361, guard :188-191→:352-356; guard widened http/tls→http/tls/modbus/dnp3 -->
| BC-2.05.008 | No Analyzer Configured: Dispatcher Early-Returns | P1 | [WRITTEN] | BC-DSP-008 | <!-- v1.6: Pass-18 B-01/B-02/B-03 anchor re-sync + four-analyzer guard prose; early-return guard :121-123→:256-259; guard widened http/tls→http/tls/modbus/dnp3 -->
| BC-2.05.009 | on_flow_close Removes Route Entry and Forwards Close | P0 | [WRITTEN] | BC-DSP-009 | <!-- v1.4: Pass-18 B-01/B-02 anchor re-sync; on_flow_close :171-194→:322-361, removes :175-176→:326-327 -->

## ss-06: HTTP Traffic Analysis (CAP-06)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.06.001 | Parse Complete HTTP/1.1 Request with Method/URI/Version/Host/UA | P0 | [WRITTEN] | BC-HTTP-001 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.002 | Parse Pipelined Requests with Independent Per-Request Counting | P0 | [WRITTEN] | BC-HTTP-002 | <!-- v1.5: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.003 | Partial Requests Buffered Until Complete; Not Counted Until Full | P0 | [WRITTEN] | BC-HTTP-003 | <!-- v1.4: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.004 | Parse HTTP/1.1 Responses with Status Code Counting | P0 | [WRITTEN] | BC-HTTP-004 | <!-- v1.9: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.005 | Path Traversal in URI Emits Reconnaissance/Likely/High Finding Mapped to T1083 | P0 | [WRITTEN] | BC-HTTP-005 | <!-- v1.9: P19-B-08 ss-06 line-anchor re-sync (prev v1.8: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.006 | Web-Shell URI Patterns Emit Execution/Likely/Medium Finding (T1505.003) | P0 | [WRITTEN] | BC-HTTP-006 | <!-- v1.6: P19-B-08 ss-06 line-anchor re-sync (prev v1.5: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.007 | Admin Panel Paths Emit Reconnaissance/Inconclusive/Low Finding (T1046) | P1 | [WRITTEN] | BC-HTTP-007 | <!-- v1.7: P19-B-08 ss-06 line-anchor re-sync (prev v1.6: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.008 | Unusual HTTP Methods Emit Reconnaissance/Inconclusive/Medium Finding | P1 | [WRITTEN] | BC-HTTP-008 | <!-- v1.5: P19-B-08 ss-06 line-anchor re-sync (prev v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.009 | HTTP/1.1 Missing or Empty Host Emits Anomaly/Inconclusive/Medium Finding | P0 | [WRITTEN] | BC-HTTP-009 | <!-- v1.5: P19-B-08 ss-06 line-anchor re-sync (prev v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.010 | URI Greater Than 2048 Chars Emits Execution/Likely/Medium Finding | P1 | [WRITTEN] | BC-HTTP-010 | <!-- v1.5: P19-B-08 ss-06 line-anchor re-sync (prev v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.011 | Empty UA Emits Anomaly/Inconclusive/Low; Absent UA Does NOT | P1 | [WRITTEN] | BC-HTTP-011 | <!-- v1.5: P19-B-08 ss-06 line-anchor re-sync (prev v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.012 | Well-Formed HTTP Request Produces Zero Findings | P0 | [WRITTEN] | BC-HTTP-012 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.013 | Non-HTTP Bytes Increment parse_errors; No Token-Error Findings | P0 | [WRITTEN] | BC-HTTP-013 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.014 | Too Many Headers Emits Anomaly/Inconclusive/Medium Finding (T1499.002) | P0 | [WRITTEN] | BC-HTTP-014 | <!-- v1.4: P19-B-08 ss-06 line-anchor re-sync (prev v1.3: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B5) -->
| BC-2.06.015 | After 3 Consecutive Parse Errors a Direction is Poisoned; Subsequent Bytes Skipped | P0 | [WRITTEN] | BC-HTTP-015 | <!-- v1.4: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.016 | Single Parse Error Does NOT Poison | P0 | [WRITTEN] | BC-HTTP-016 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.017 | Poisoning is Per-Direction; Poisoned Request Does Not Affect Response | P0 | [WRITTEN] | BC-HTTP-017 | <!-- v1.4: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.018 | non_http_flows Counts Flow Once Even if Both Directions Poisoned | P1 | [WRITTEN] | BC-HTTP-018 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.019 | on_flow_close Removes Per-Flow State; Reopening Same Key Starts Fresh | P0 | [WRITTEN] | BC-HTTP-019 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.020 | HTTP Body Bytes After Header Completion Do Not Inflate parse_errors | P1 | [WRITTEN] | BC-HTTP-020 | <!-- v1.5: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.021 | Cross-Flow Isolation: Errors and Poisoning Do Not Leak | P0 | [WRITTEN] | BC-HTTP-021 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.022 | Per-Direction Header Buffer Capped at MAX_HEADER_BUF (65536) | P1 | [WRITTEN] | BC-HTTP-022 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.023 | summarize Emits AnalysisSummary with HTTP Stats Detail Map | P1 | [WRITTEN] | BC-HTTP-023 | <!-- v1.5: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.024 | Per-Map Cardinality Cap: New Keys Dropped Past MAX_MAP_ENTRIES | P2 | [WRITTEN] | BC-HTTP-024 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.025 | uris List Capped at MAX_URIS=10000 | P2 | [WRITTEN] | BC-HTTP-025 | <!-- v1.3: P19-B-08 ss-06 line-anchor re-sync -->
| BC-2.06.026 | Header Values Extracted via from_utf8_lossy.trim(); Raw Bytes Preserved | P0 | [WRITTEN] | BC-HTTP-026 | <!-- v1.4: P19-B-08 ss-06 line-anchor re-sync -->

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
| BC-2.07.008 | JA3S String Format: version,cipher,extensions; MD5 Hex | P0 | [WRITTEN] | BC-TLS-008 | <!-- v1.4: P19 B-10 anchor fix: format tls.rs:171→:172; digest :172→:173 -->
| BC-2.07.009 | Weak Client Cipher in ClientHello Emits Anomaly/Likely/High Finding | P0 | [WRITTEN] <!-- v1.3: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 --> | BC-TLS-009 |
| BC-2.07.010 | Weak Server Cipher Selected Emits Anomaly/Likely/Medium Finding | P0 | [WRITTEN] <!-- v1.3: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 --> | BC-TLS-010 |
| BC-2.07.011 | Deprecated Client Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding | P0 | [WRITTEN] <!-- v1.4: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 --> | BC-TLS-011 |
| BC-2.07.012 | Deprecated Server Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding | P0 | [WRITTEN] <!-- v1.5: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B6 --> | BC-TLS-012 |
| BC-2.07.013 | Clean ASCII SNI Produces No Finding | P0 | [WRITTEN] | BC-TLS-013 |
| BC-2.07.014 | SNI Containing C0/DEL Byte Emits Anomaly/Inconclusive/Low Finding Mapped to T1027 | P0 | [WRITTEN] <!-- v1.3: mitre_technique→mitre_techniques vec!["T1027"]; ARP-F2 P14 B6 --> | BC-TLS-014 |
| BC-2.07.015 | Multiple Control Bytes in One SNI Produce Exactly ONE Finding | P0 | [WRITTEN] | BC-TLS-015 |
| BC-2.07.016 | C0 Boundary: 0x1F Trips Finding; 0x20 (Space) Does NOT | P0 | [WRITTEN] | BC-TLS-016 | <!-- v1.3: P19 B-10 anchor fix: contains_c0_or_del tls.rs:231-238→:232-239 -->
| BC-2.07.017 | Non-ASCII UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027) | P0 | [WRITTEN] <!-- v1.4: mitre_technique→mitre_techniques vec!["T1027"]; ARP-F2 P14 B6 --> | BC-TLS-017 |
| BC-2.07.018 | Punycode A-label is Pure ASCII; Emits No SNI Finding | P1 | [WRITTEN] | BC-TLS-018 |
| BC-2.07.019 | Non-UTF-8 SNI Emits Anomaly/Inconclusive/Low Finding (T1027); Count Key Tagged | P0 | [WRITTEN] <!-- v1.4: mitre_technique→mitre_techniques vec!["T1027"]; ARP-F2 P14 B6 --> | BC-TLS-019 |
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
| BC-2.07.037 | SNI with Both Non-ASCII and C0 Control Bytes Fires Arm 3 (NonAsciiUtf8), Not Arm 2 | P0 | [WRITTEN] | BC-TLS-037 | <!-- v1.3: P19 B-10 anchor fix: extract_sni tls.rs:246→:247; match block :251-265→:252-269; v1.4: PG-ARP-F2-007 arm 2/3 emission :426→:437/:449→:461 -->

<!-- PG-ARP-F2-007 (2026-06-13): full ss-07 tls.rs re-anchor applied to ALL 37 BCs (001-037, except 016/030 already clean). Root cause: tls.rs shifted ~10-60 lines from F2 timestamp-wiring (STORY-097/098/099). BC versions bumped individually; all Architecture Module, Architecture Anchors, Source Evidence Path, and inline prose citations updated to HEAD. -->

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
| BC-2.09.001 | Finding Constructed with Required Fields and Optional Fields | P0 | [WRITTEN] | BC-FND-001 | <!-- v1.6: P19 B-01 anchor :119-146 → :135-162 -->
| BC-2.09.002 | Finding Display Renders [Category] VERDICT (CONFIDENCE) — summary | P1 | [WRITTEN] | BC-FND-002 | <!-- v1.5: P19 B-02 anchor :159-170 → :173-184 -->
| BC-2.09.003 | Verdict Display: Uppercase Tokens | P1 | [WRITTEN] | BC-FND-003 | <!-- v1.3: P19 B-03 anchor :43-50 → :48-57; Possible variant added -->
| BC-2.09.004 | Confidence Display: Uppercase Tokens | P1 | [WRITTEN] | BC-FND-004 | <!-- v1.3: P19 B-04 anchor :68-76 → :75-83 -->
| BC-2.09.005 | Finding.summary and Evidence Store RAW Post-from_utf8_lossy Bytes per ADR 0003 | P0 | [WRITTEN] | BC-FND-005 | <!-- v1.7: P19 B-05 struct :120 → :135; fields :124-125 → :140-141; doc-comment :150-158 → :164-172; ev call-site :223 → :224 -->
| BC-2.09.006 | Finding JSON Serialization: Empty Vec Fields Omitted; mitre_techniques Serialized as Array | P0 | [WRITTEN] | BC-FND-006 | <!-- v1.6: mitre_techniques Vec; skip_serializing_if Vec::is_empty; ADR-006 F2 revision; P19 confirmed clean -->
| BC-2.09.007 | Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site | P1 | [WRITTEN] | feature-100-F2 | <!-- v1.1.2: P19 B-06 anchor :119-146 → :135-162 -->

## ss-10: MITRE ATT&CK Mapping (CAP-10)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.10.001 | MitreTactic Display Renders Enterprise Tactics with Canonical Spacing | P0 | [WRITTEN] | BC-MIT-001 |
| BC-2.10.002 | ICS Tactics Render Without "ICS:" Prefix; IcsImpact Disambiguated as "Impact (ICS)" | P1 | [WRITTEN] | BC-MIT-002 | <!-- v1.5: D-069 — IcsImpact Display corrected to "Impact (ICS)" (H1 title updated per bc_h1_is_title_source_of_truth); code at src/mitre.rs:91 was already correct; spec-side corrected. Supersedes D-067. -->
| BC-2.10.003 | all_tactics_in_report_order Returns Kill-Chain Order First Then ICS | P0 | [WRITTEN] | BC-MIT-003 |
| BC-2.10.004 | all_tactics_in_report_order Contains Every Variant Exactly Once | P0 | [WRITTEN] | BC-MIT-004 |
| BC-2.10.005 | technique_name Returns Some for Every Seeded ID (25 Total) | P0 | [WRITTEN] | BC-MIT-005 | <!-- v1.10: count 23->25; T0830 (ICS)+T1557.002 (Enterprise) added (ARP F2); 12E+13I split; PLANNED forward-declaration added -->
| BC-2.10.006 | technique_name Returns None for Unknown IDs | P0 | [WRITTEN] | BC-MIT-006 |
| BC-2.10.007 | technique_tactic Returns Correct Tactic for Every Seeded ID | P0 | [WRITTEN] | BC-MIT-007 |
| BC-2.10.008 | All Emitted Technique IDs Resolve in Lookup | P0 | [WRITTEN] | BC-MIT-008 | <!-- v1.12: 17 emitted IDs; T0830 (ICS)+T1557.002 (Enterprise) added (ARP F2); 7E+10I split; PLANNED forward-declaration in STORY-114 -->
| BC-2.10.009 | MitreTactic is #[non_exhaustive] | P2 | [WRITTEN] | BC-MIT-009 |

## ss-11: Reporting and Output (CAP-11)

> 29 BCs total; 29 fully written; 0 planned.
> BCs 001-019: JsonReporter / TerminalReporter / MITRE grouping (brownfield ingestion).
> BCs 020-024: CsvReporter (added pass-4, adversarial finding H-1).
> BCs 025-029: terminal finding collapse (greenfield, issue #259, v0.8.0).

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.11.001 | JsonReporter Renders JSON Object with summary/findings/analyzers/mitre_domain/mitre_attack_version Keys | P0 | [WRITTEN] | BC-RPT-001 | <!-- v1.5: ADD-ON 1 — envelope fields mitre_domain + mitre_attack_version added; mitre_attack_version placeholder "ics-attack-v15" flagged for F4 to pin -->
| BC-2.11.002 | JsonReporter Includes skipped_packets in Summary | P1 | [WRITTEN] | BC-RPT-002 |
| BC-2.11.003 | JsonReporter Escapes C0 Control Bytes per RFC 8259 via serde | P0 | [WRITTEN] | BC-RPT-003 |
| BC-2.11.004 | JsonReporter Preserves Non-ASCII Unicode in Readable Form | P1 | [WRITTEN] | BC-RPT-004 |
| BC-2.11.005 | JsonReporter Passes C1 Codepoints Through as Raw UTF-8 | P1 | [WRITTEN] | BC-RPT-005 |
| BC-2.11.006 | TerminalReporter Shows Skipped: N Packets Only When N > 0 | P1 | [WRITTEN] | BC-RPT-006 |
| BC-2.11.007 | TerminalReporter Escapes C0+DEL+C1+Backslash in Finding Summary and Evidence | P0 | [WRITTEN] | BC-RPT-007 |
| BC-2.11.008 | TerminalReporter Escape Preserves Printable ASCII and UTF-8 | P0 | [WRITTEN] | BC-RPT-008 |
| BC-2.11.009 | TerminalReporter Escapes C1 Codepoints U+0080-U+009F; U+00A0 Preserved | P0 | [WRITTEN] | BC-RPT-009 | <!-- v1.5: PG-ARP-F2-007 — test fn anchors: escapes_c1_nel_and_csi :375→:544; escapes_c1_range_boundaries :388→:556 -->
| BC-2.11.010 | TerminalReporter Escapes Both Summary AND Each Evidence Line | P0 | [WRITTEN] | BC-RPT-010 | <!-- v1.5: issue-#259 F2 — Invariant 4 + EC-006/EC-007: collapse-interaction: evidence sampling bounded to K=3 per BC-2.11.027; escape_for_terminal invariant unchanged; cross-refs BC-2.11.025/027/029; v1.6 2026-06-17: adv-pass-3 — Invariant 4 + EC-007: change false 'same call site in render_finding_prefix' → 'same escape_for_terminal FUNCTION called directly by collapse wrapper' (F-F2X-01); v1.7 2026-06-17: adv-pass-4 — F-F2-O01: anchor :203-226 → :203-227; Source Evidence path updated; v1.8 2026-06-17: adv-pass-5 — F1: remove residual '(BC-2.11.026 path-(b))' label from Invariant 4 body; v1.9 2026-06-17: issue-#62 F2 BC re-anchor (fix-burst) — Invariant 4 + EC-006 + EC-007: collapse_findings=true → render=FindingsRender::FlatCollapsed -->
| BC-2.11.011 | TerminalReporter Escapes Analyzer-Summary Detail Values | P0 | [WRITTEN] | BC-RPT-011 |
| BC-2.11.012 | TerminalReporter End-to-End: C1 CSI in Path-Traversal Finding Escaped | P0 | [WRITTEN] | BC-RPT-012 | <!-- v1.6 2026-06-17: adv-pass-4 sibling sweep — F-F2-O01: anchor :203-226 → :203-227 -->
| BC-2.11.013 | MITRE Grouping Emits Tactic Headers in Canonical Order; Uncategorized Last | P0 | [WRITTEN] | BC-RPT-013 | <!-- v1.8: PG-ARP-F2-007 — render_findings_grouped anchor :260-304→:272-323; tactic loop :290→:309; v1.9: issue-#259 F2 — Invariant 4 + EC-007: show_mitre_grouping=true suppresses collapse pass; grouped-mode collapse deferred to STORY-119; cross-refs BC-2.11.025/028/029; v1.10 2026-06-17: adv-pass-2 — EC-007 strengthened: structural suffix-free guarantee via path-(b) (F-A03); v1.11 2026-06-17: adv-pass-4 — EC-007 STRUCTURAL guarantee converted to OBSERVABLE GUARANTEE form (F-F2-A01); v1.12 2026-06-17: issue-#62 F2 BC re-anchor — show_mitre_grouping=true → render=FindingsRender::Grouped; Invariant 4 + EC-007 updated for enum illegal-state elimination -->
| BC-2.11.014 | Within Tactic Bucket: Sort by Verdict, Confidence, Emission Order | P1 | [WRITTEN] | BC-RPT-014 | <!-- v1.6: PG-ARP-F2-007 — sort-closure anchors: verdict_rank :269-275→:287-293; confidence_rank :276-282→:295-301; sort_by_key :284-288→:303-307; bucket push line 266→284; v1.7 2026-06-17: issue-#62 F2 BC re-anchor — Precondition 1: show_mitre_grouping=true → render=FindingsRender::Grouped -->
| BC-2.11.015 | No-Technique or Unknown-ID Findings Land in Uncategorized | P0 | [WRITTEN] | BC-RPT-015 | <!-- v1.7: PG-ARP-F2-007 — render_finding_grouped :244-252→:247-263; Uncategorized bucket :298-303→:317-322; v1.8 2026-06-17: issue-#62 F2 BC re-anchor (fix-burst) — Precondition 1: show_mitre_grouping=true → render=FindingsRender::Grouped -->
| BC-2.11.016 | MITRE Grouping Expands Per-Finding Line with Em-Dash and Name | P1 | [WRITTEN] | BC-RPT-016 | <!-- v1.5: mitre_technique→mitre_techniques vec![]; ARP-F2 P14 B7; v1.6: PG-ARP-F2-007 — expansion range :246-251→:249-261; em-dash literal :248→:259; v1.7 2026-06-17: issue-#62 F2 BC re-anchor (fix-burst) — Precondition 1: show_mitre_grouping=true → render=FindingsRender::Grouped -->
| BC-2.11.017 | Default Rendering Emits MITRE: <id(s)> Only (No Em-Dash) | P1 | [WRITTEN] | BC-RPT-017 | <!-- v1.5: multi-ID rendering "MITRE: T0855, T0836"; ADR-006 F2 revision; v1.6: example updated to T1692.001 (v19 remap); v1.7: PG-ARP-F2-007 — render_finding_flat anchor :230-235→:232-238; v1.8: issue-#259 F2 — Invariant 5 + EC-007/EC-008: collapse path invokes render_finding_flat per group; (xN) suffix on header line; N=1 byte-identical to pre-v0.8.0; cross-refs BC-2.11.025/026/028; v1.9 2026-06-17: adv-pass-1 — Invariant 5: (xN) suffix colorized with header (F-259-02); v1.10 2026-06-17: adv-pass-2 — Invariant 5 aligned to path-(b) wrapper; render_finding_prefix unchanged (F-A03); v1.11 2026-06-17: adv-pass-4 — F-F2-A01: Invariant 5 + Description converted to observable-behavior; PC-6 added (MITRE line in collapse path); non-normative implementation note added; 'render_finding_flat called once per group' call-graph claim removed; v1.12 2026-06-17: adv-pass-9 — F-PA-01: Invariant 5 cross-references BC-2.11.026 PC-6 for full color-ladder requirement; v1.13 2026-06-17: adv-passes-12-14 — F-PA-A01: representative finding = group_members[0]; PC-6 + EC-007 updated to cite group_members[0]; divergent-mitre test vector added; v1.14 2026-06-17: issue-#62 F2 BC re-anchor — show_mitre_grouping=false/collapse_findings bools → FindingsRender enum in Precondition + Description + Postcondition 6 + Invariants 1/5 + EC-004/007/008 + test vectors -->
| BC-2.11.018 | TerminalReporter Colorization: Likely/High=Red Bold, etc. | P2 | [WRITTEN] | BC-RPT-018 | <!-- v1.4: PG-ARP-F2-007 — colorization block :209-220→:209-222 (if-else block closes at 222) -->
| BC-2.11.019 | TerminalReporter Renders Sections in Correct Order | P1 | [WRITTEN] | BC-RPT-019 | <!-- v1.5: issue-#259 F2 — Postcondition 9 + Invariant 7 + EC-008/EC-009: flat FINDINGS dispatch routes through collapse pass when collapse_findings=true; section ordering unchanged; cross-refs BC-2.11.025/026/027/028/029; v1.6 2026-06-17: adv-pass-2 — anchor 149-160→149-162 in Invariant 7 (F-A05); v1.7 2026-06-17: issue-#62 F2 BC re-anchor — collapse_findings/show_mitre_grouping bools → FindingsRender enum in Postcondition 9 + Invariant 7 + EC-008/EC-009 -->
| BC-2.11.020 | CsvReporter Emits Exactly Nine Columns in Fixed Header Order | P0 | [WRITTEN] | pass-4 H-1 | <!-- v1.5: column-6 header renamed mitre_technique->mitre_techniques; ADR-006 F2 revision -->
| BC-2.11.021 | CsvReporter Neutralizes CSV-Injection Trigger Characters with a Leading Single Quote | P0 | [WRITTEN] | pass-4 H-1 | <!-- v1.4: PG-ARP-F2-007 — neutralize application range :89-97→:92-103 (STORY-100 added mitre_techniques column) -->
| BC-2.11.022 | CsvReporter Joins Evidence Vec Elements with "; " into a Single Cell | P1 | [WRITTEN] | pass-4 H-1 | <!-- v1.4: PG-ARP-F2-007 — evidence neutralize call :93→:98 (shifted by mitre_techniques column addition) -->
| BC-2.11.023 | CsvReporter Implements Reporter Trait and Emits One Row per Finding; Summary and AnalysisSummary Are Ignored | P0 | [WRITTEN] | pass-4 H-1 |
| BC-2.11.024 | CsvReporter Encodes Optional Fields as Empty Strings and mitre_techniques as Semicolon-Joined String | P1 | [WRITTEN] | pass-4 H-1 | <!-- v1.5: ADD-ON 2 — EC-015 added (consumer split guard for empty-cell); EC-001 strengthened (empty string not null/[]/N/A); Inv 4 explicit empty-string wording; v1.7: Pass-15 D-01: Evidence Types Updated; v1.8: PG-ARP-F2-007 — neutralize anchor :94-97→:99-102; pc1 clarified join@:87 vs neutralize@:99 -->
| BC-2.11.025 | Flat-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key; First-Occurrence Order; Deterministic | P0 | [WRITTEN] | issue-#259 greenfield | <!-- v1.1 2026-06-17: adv-pass-1 — singleton immutability invariant (F-259-03), severity-agnostic PC (F-259-04), insertion-ordered grouping invariant (F-259-06), raw-key/escaped-display PC + test vectors (F-259-09); v1.2 2026-06-17: adv-pass-2 — Vec-accumulator canonical, IndexMap demoted (F-A01); flood test vector strengthened (F-A04); anchor 149-160→149-162 (F-A05); v1.3 2026-06-17: adv-pass-4 — F-F2-A01: Invariant 6 singleton converted to observable-behavior; F-F2-O01: anchor :203-226→:203-227; F-F2-O02: flood vector timestamp updated to 'differing per-request timestamps'; v1.4 2026-06-17: adv-pass-9 — F-PA-02: soften flood-vector timestamp claim to 'timestamps MAY differ'; v1.5 2026-06-17: adv-passes-12-14 — F-PA-A01: generalize Invariant 6 representative definition from N=1 singleton to all N≥1 (group_members[0] for all N); v1.6 2026-06-17: F-F3-001: fix flood-vector evidence format 'GET /a HTTP/1.1'→'GET /a' (method+URI only, per http.rs:365); v1.7 2026-06-17: issue-#62 F2 BC re-anchor — collapse_findings/show_mitre_grouping bools → FindingsRender enum: Preconditions 1-2 + Description + Invariant 5 + EC-011; v1.8 2026-06-18: F2 adv-pass-2 fix (F-6) — VP-table: 'show_mitre_grouping=true suppresses collapse' → 'render = FindingsRender::Grouped suppresses collapse' -->
| BC-2.11.026 | Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix | P0 | [WRITTEN] | issue-#259 greenfield | <!-- v1.1 2026-06-17: adv-pass-1 — (xN) suffix IS colorized with header line; Invariant 4 + PC-4 + EC-008 updated (F-259-02); v1.2 2026-06-17: adv-pass-2 — path-(b) wrapper canonical in PC-4 (F-A03); anchor 149-160→149-162 (F-A05); EC-005 test vector + EC-009 suffix-free grouped mode EC (F-A06, F-A03); v1.3 2026-06-17: adv-pass-3 — PC-4 evidence emission sentence added (escape_for_terminal DIRECTLY, NOT via render_finding_prefix) (F-F2X-03); EC row order fixed 009/008→008/009 monotonic (F-F2X-02); arch anchor stale 'appended here' text removed; v1.4 2026-06-17: adv-pass-4 — F-F2-A01: PC-4 converted to observable-behavior (CANONICAL LINE ORDER) + non-normative note; PC-2 singleton ref updated; Inv-4 render_finding_prefix implementation detail removed; EC-007 STRUCTURAL→OBSERVABLE; F-F2-O01: anchor :203-226→:203-227; v1.5 2026-06-17: adv-pass-5 — F1: remove residual 'path-(b) separation' label from EC-009 body; v1.6 2026-06-17: adv-passes-6-8 — LOW-1: add Likely/High red-bold canonical test vector (suffix inside red-bold span); v1.7 2026-06-17: adv-pass-9 — F-PA-01: add normative PC-6 color-ladder requirement; v1.8 2026-06-17: adv-passes-12-14 — F-PA-A01: add normative PC-7 representative = group_members[0] for all N; divergent-mitre canonical test vector added; v1.9 2026-06-17: issue-#62 F2 BC re-anchor — collapse_findings/show_mitre_grouping bools → FindingsRender enum in Preconditions 1-2 + EC-006/EC-007/EC-009 -->
| BC-2.11.027 | Collapsed Group Retains at Most K=3 Representative Evidence Lines; Remainder Elided from Terminal Display | P1 | [WRITTEN] | issue-#259 greenfield | <!-- v1.1 2026-06-17: fix N=1 singleton model — K-cap NOT applied to singletons; Invariant 6 + EC-001 + last test vector corrected per adjudicated model (consistency audit); v1.2 2026-06-17: adv-pass-1 CRITICAL F-259-01 — positional first-K-members model enforced throughout; EC-004 corrected to 2 lines not 3; N=3/N=4 boundary vectors added (F-259-07); v1.3 2026-06-17: adv-pass-3 — PC-1/PC-6/Invariant-5: false render_finding_prefix loop reuse claims corrected to escape_for_terminal function-level reuse (F-F2X-01); v1.4 2026-06-17: escape-notation accuracy — EC-007 + canonical test vector: \\x1b → \\u{1b} (char::escape_default; verified by terminal.rs escapes_esc_byte test); v1.5 2026-06-17: issue-#62 F2 BC re-anchor — collapse_findings/show_mitre_grouping bools → FindingsRender enum in Preconditions 1-2 + EC-008 -->
| BC-2.11.028 | --no-collapse Opt-Out Flag Disables Terminal Collapse and Restores One-Line-Per-Finding Rendering; JSON/CSV Unaffected | P0 | [WRITTEN] | issue-#259 greenfield | <!-- v1.1 2026-06-17: fix Related BCs stale cross-ref BC-2.13.001→BC-2.13.004 (consistency audit); v1.2 2026-06-17: adv-pass-1 — PC-3 imperative; arch anchors marked insertion targets (F-259-08); v1.3 2026-06-17: adv-pass-9 — F-PA-03: add EC-010 (--no-collapse absent, default --output terminal → collapse applies, default-on); v1.4 2026-06-17: adv-passes-12-14 — F-PB-01: drop '--no-color/--no-reassemble convention' citation (global flags, not subcommand-scoped); replace with subcommand-scoped boolean precedent (#[arg(long)] mitre/dns on Commands::Analyze, cli.rs:150-152); fix stale arch anchor cli.rs:151-153 no_reassemble → cli.rs:150-152 mitre: bool; v1.5 2026-06-17: issue-#62 F2 BC re-anchor — complete wiring note updated to FindingsRender three-way expression; Preconditions + Postconditions + Invariants 1-2 + EC-001..005 updated for enum -->
| BC-2.11.029 | Collapse is Display-Layer Only; JSON/CSV Reporters Receive Unmodified findings Slice; Non-Repeated Findings Individually Visible in All Outputs | P0 | [WRITTEN] | issue-#259 greenfield | <!-- v1.1 2026-06-17: fix PC-3 misleading N=1≤K=3 reasoning (consistency audit); v1.2 2026-06-17: adv-pass-1 — precise csv.rs:40/csv.rs:76 anchors; terminal.rs:63-75 marked insertion target (F-259-05, F-259-08); v1.3 2026-06-17: issue-#62 F2 BC re-anchor (fix-burst) — Precondition 4 + PC-1 inline qualifier + Architecture Anchors: collapse_findings bool → FindingsRender::FlatCollapsed enum -->

## ss-12: CLI and Entry Point (Cross-Cutting)

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.12.001 | analyze Subcommand Parses Positional Targets and All Flags | P0 | [WRITTEN] | BC-CLI-001 |
| BC-2.12.002 | summary Subcommand Parses Targets and --hosts Flag | P1 | [WRITTEN] | BC-CLI-002 |
| BC-2.12.003 | Global Flag --no-color Parsed and Stored | P1 | [WRITTEN] | BC-CLI-003 |
| BC-2.12.004 | --output-format json Parses to Some(OutputFormat::Json) | P0 | [WRITTEN] | BC-CLI-004 |
| BC-2.12.005 | Reassembly CLI Flags: --reassemble/--no-reassemble, depth, memcap, and five anomaly-threshold flags | P0 | [WRITTEN] | BC-CLI-005 | <!-- P20-B-04+B-05: v1.4→v1.5 main.rs:87-122→139-166, inv4 104-117→147-161, cli.rs:71-122→73-124 -->
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
> BCs 013-015: Finding Emission — Write-Class Events (Group D). **v2 co-emission model (ADR-006):** one multi-tag finding per write PDU; T1692.001 co-included in vec, not separate. No tag-suppression.
> BCs 016-017: Finding Emission — Coordinated Write (T0831 5s window, Group E) and **dual-window** Write-Burst Detection (T0806/T1692.001; burst 1s + sustained >=2s per Decision-11, Group E).
> BCs 018-019: Finding Emission — Diagnostic/DoS (T0814) (BC-018) and Exception Burst Anomaly (BC-019) (Group F).
> BCs 020-022: Anomaly/Recon (**T0888** for 0x11/0x2B/0x0E — Decision-12; T0846 NOT emitted by Modbus), Summary Stats (6 keys incl dropped_findings), and Bounded-Resource (Groups G + resource cap).
> BCs 023-025: Dispatcher and CLI Integration (Group H). **BC-024 v2:** two flags (--modbus-write-burst-threshold + --modbus-write-sustained-threshold); old --modbus-write-threshold removed.
> Feature: issue-007-modbus-analyzer; ADR-005; ADR-006; introduced v0.3.0-feature-007.
> **v2 revision (2026-06-09):** BCs 013-017, 020, 024 revised per f2-fix-directives.md v2 Decisions 11-13.

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.14.001 | MBAP Header Accepted for Well-Formed 8-Byte-Minimum ADU | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.002 | MBAP Header Rejected for ADU Shorter than 8 Bytes (Truncation Safety) | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.003 | MBAP ADU Rejected When Protocol ID Is Not 0x0000 (3-Point Gate: Protocol Check — Bail-Out Policy) | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.004 | MBAP ADU Rejected When Length Is Outside [2, 254] (3-Point Gate: Length Check) | P0 | [WRITTEN] | feature-007-F2 | <!-- v1.1: Pass-14 Burst-3 D-01: Source Evidence path annotation [2,253]→[2,254] (annotation only; H1 unchanged) -->
| BC-2.14.005 | classify_fc Is Total Over All 256 FC Values — Complete Classification Enum | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.006 | Exception Response Detection — FC High Bit Set Identifies Exception and Recovers Original FC | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.007 | Write-Class FC Classification — State-Changing Function Codes Identified as Elevated-Risk | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.008 | Diagnostic-Class FC Classification and Sub-Function Dispatch (0x08 and 0x2B) | P1 | [WRITTEN] | feature-007-F2 |
| BC-2.14.009 | Request PDU Inserted into Per-Flow Pending Table Keyed on (Transaction ID, Unit ID) | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.010 | Response PDU Matched Against Pending Table and Entry Removed on FC Echo Match | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.011 | Exception Response Attributed to Originating Request FC via (Transaction ID, Unit ID) Lookup | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.012 | Pending Table Bounded to MAX_PENDING_TRANSACTIONS=256; New Requests Dropped When Full | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.013 | Write-Class FC in Request Direction Emits Multi-Tag Finding Carrying T1692.001 and Applicable Technique Tags | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: co-emission model; T0855 co-included in multi-tag vec; ADR-006 Decision-13; v2.3: T0855→T1692.001 (v19 remap, issue #222) -->
| BC-2.14.014 | Write FC 0x06/0x10/0x16/0x17 in Request Direction Emits Finding Tagged ["T1692.001","T0836"] | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: single multi-tag finding replaces two separate findings; ADR-006 Decision-13; v2.1: 0x17 added per BC-DISCREPANCY-001; v2.3: T0855→T1692.001 (v19 remap, issue #222) -->
| BC-2.14.015 | Write FC to Coil Output Only ({0x05, 0x0F}) Emits Finding Tagged ["T1692.001","T0835"] | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: single multi-tag finding; no suppression; ADR-006 Decision-13; v2.3: T0855→T1692.001 (v19 remap, issue #222) -->
| BC-2.14.016 | Coordinated Write Sequence to Holding Registers Within 5-Second Window Tags the Per-PDU Finding with T0831 | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: T0831 is co-tagged inline on the per-PDU write finding (mitre_techniques: ["T1692.001","T0836","T0831"]); no separate T0831 Finding object; ADR-006 Decision-13 §13.5; v2.2: T0855→T1692.001 (v19 remap, issue #222) -->
| BC-2.14.017 | Write-Rate Exceeding Either Burst or Sustained Threshold Emits T0806 + T1692.001 Finding | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: dual-window (1s burst / >=2s sustained); each fires at most once per window; ADR-006 Decision-11; v2.4: T0855→T1692.001 (v19 remap, issue #222); v2.5: Pass-14 Burst-3 B-01: MITRE traceability T1692.001 display name corrected to "Unauthorized Message: Command Message"; v2.6: issue #220: burst summary changed from "writes in {elapsed_secs}s window" to "writes within {window_secs}s window" (constant WIDTH, not elapsed span) to fix 0s display on same-second writes -->
| BC-2.14.018 | Diagnostics FC 0x08 Sub-Function 0x0004 or 0x0001 Emits T0814 Denial of Service Finding | P0 | [WRITTEN] | feature-007-F2 | <!-- v1.3: Pass-30 B-03: source_ip flow_key.client_ip() (non-existent) → Direction-resolved endpoint --> |
| BC-2.14.019 | Exception Response Anomaly — Burst of Exception Codes Emits Anomaly Finding for Recon/Scanning | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.020 | Reconnaissance Function Codes (0x11, 0x2B/0x0E) Emit T0888 Remote System Information Discovery Finding | P1 | [WRITTEN] | feature-007-F2 | <!-- v2.0: T0846->T0888 correctness fix; 0x07 excluded; Decision-12; v2.2: Pass-14 Burst-3 B-03/B-04: Invariant-6 counts 21/13→25/17 (canonical); Source Evidence §4.3 annotated as Decision-12-era; v2.3: Pass-30 B-01/B-02: source_ip flow_key.client_ip()/server_ip() (non-existent) → Direction-resolved endpoint in postcondition + EC-010 -->
| BC-2.14.021 | summarize() Returns AnalysisSummary with Specified Per-Analyzer Summary Keys | P1 | [WRITTEN] | feature-007-F2 |
| BC-2.14.022 | MAX_FINDINGS Cap and Poison-Skip Behavior for ModbusAnalyzer | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.023 | --modbus CLI Flag Enables ModbusAnalyzer; --all Includes Modbus; Default-Off; Requires Stream Reassembly | P0 | [WRITTEN] | feature-007-F2 |
| BC-2.14.024 | --modbus-write-burst-threshold and --modbus-write-sustained-threshold Configure Dual-Window Burst Detection | P0 | [WRITTEN] | feature-007-F2 | <!-- v2.0: old --modbus-write-threshold removed; replaced by two flags; Decision-11; v2.2: Pass-14 Burst-3 B-02: MITRE traceability T1692.001 display name corrected to "Unauthorized Message: Command Message" -->
| BC-2.14.025 | StreamDispatcher Classifies Port-502 Flows to DispatchTarget::Modbus as Rule 5 (After Content and TLS/HTTP Port Rules); Routes on_data and on_flow_close to ModbusAnalyzer | P0 | [WRITTEN] | feature-007-F2 |

## ss-15: DNP3/ICS Analysis (CAP-15)

> 24 BCs total; 24 fully written; 0 planned.
> BCs 001-004: DL Header Parse and Validity Gate (Group A + C).
> BCs 005-007: Function-Code Classification (Group B).
> BCs 008-009: Transport Layer / Desync Safety (Group E + desync).
> BCs 010-013: Finding Emission — Detection (T1692.001 control threshold, T0814 restart/DoS, T0836 write, T0827 co-emission) (Group D).
> BCs 014-015: Derived / Correlated Findings (T1691.001 block-command inference, T0827 loss-of-control) (Group F).
> BCs 016-017: Bounded Resource and CLI Integration (Group G + H).
> BCs 018-019: Anomaly Detection (broadcast destination, unsolicited response) (Group I).
> BC 020: Summary Stats.
> BCs 021-022: Dispatcher Integration and MAX_FINDINGS DoS Bound.
> BC 023: Research must-add — DISABLE_UNSOLICITED/ENABLE_UNSOLICITED abuse → T0814 (alarm suppression).
> BC 024: Research must-add — malformed/structural DNP3 anomaly from malformed_in_window threshold → T0814 (Crain-Sistrunk coverage).
> Feature: issue-008-dnp3-analyzer; ADR-007; introduced v0.6.0-feature-008.
> **New MITRE techniques (F2 DNP3):** T1691.001 (IcsInhibitResponseFunction — inferred block command) + T0827 (IcsImpact — derived loss-of-control correlated finding).
> **Research must-adds (2026-06-10 post-gate):** BC-2.15.023 and BC-2.15.024 use existing T0814 — no catalog change; MITRE counts remain 23 seeded / 15 emitted / 8 catalogue-only (counts current as of issue #8 post-gate; raised to 25 seeded / 17 emitted by issue #9 ARP — see BC-2.10.005/008; PLANNED until STORY-114).

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.15.001 | DNP3 DL Header Accepted for Well-Formed 10-Byte-Minimum Frame | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.002 | DNP3 DL Header Rejected for Frame Shorter Than 10 Bytes (Truncation Safety) | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.003 | DEST/SOURCE Addresses Decoded Little-Endian from Fixed Offsets 4–7 | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.004 | Three-Point Validity Gate Returns True Iff Sync==0x0564 and LENGTH>=5 | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.005 | classify_dnp3_fc Is Total Over All 256 FC Values (No Gap, No Panic) | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.006 | FC Classification Correctness — Control {0x03,0x04,0x05,0x06}, Restart {0x0D,0x0E}, Write {0x02}, Read {0x01} | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.007 | compute_dnp3_frame_len Arithmetic Correct; Result in [10,292]; No Overflow | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.008 | Transport FIR=1 First-Fragment Gates Application-Layer FC Extraction | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.009 | is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync (One-Shot, First Delivery Only) | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.010 | Unauthorized Control Command — Unexpected Source (count=1) or Control-Class FC Exceeding Threshold Emits T1692.001 | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.011 | COLD_RESTART/WARM_RESTART Observed — Emits T0814 Per-Occurrence Finding | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.012 | WRITE FC Observed — Emits T0836 Modify-Parameter Finding Per-Occurrence | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.013 | Co-Emission Ordering — Direct Finding (T0814/T1692.001) Precedes Derived T0827 | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.014 | Inferred Block-Command — Control Request Without Response Within Window Emits T1691.001 | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.015 | Derived Loss-of-Control — N Restart/Block Events in Window Emits T0827 as Correlated Finding | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.016 | Per-Flow State Bounds — Carry Buffer ≤292 B, master_addrs ≤64, pending_requests ≤256 | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.017 | --dnp3-direct-operate-threshold CLI Flag Controls Control-Command Detection Window | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.018 | Broadcast Destination Anomaly — DEST in 0xFFFD/0xFFFE/0xFFFF Emits Anomaly Finding | P1 | [WRITTEN] | feature-008-F2 |
| BC-2.15.019 | Unsolicited Response Anomaly — UNS Bit Set or FC 0x82 From Unexpected Pattern | P1 | [WRITTEN] | feature-008-F2 |
| BC-2.15.020 | summarize() Emits Function-Code Distribution and Control-Operation Counts | P1 | [WRITTEN] | feature-008-F2 |
| BC-2.15.021 | Port-20000 Flow Dispatched to Dnp3Analyzer (DispatchTarget::Dnp3, Rule 6) | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.022 | MAX_FINDINGS DoS Bound — Finding Cap Prevents Unbounded all_findings Growth | P0 | [WRITTEN] | feature-008-F2 |
| BC-2.15.023 | Unsolicited-Response Enable/Disable Abuse — FC 0x15/0x14 Observed Emits T0814 | P1 | [WRITTEN] | feature-008-F2 |
| BC-2.15.024 | Malformed/Structural DNP3 Anomaly — malformed_in_window Threshold Emits T0814 | P1 | [WRITTEN] | feature-008-F2 |

## ss-16: ARP Security Analysis (CAP-16) [Feature #9 — ADR-008]

> 15 BCs total; 15 fully written; 0 planned.
> BCs 001-002: ARP frame extraction (Group A — parse).
> BCs 003-004: GARP detection and spoof detection / binding-table update (Group B — detection).
> BC-005: Binding-table update (last-seen MAC wins) (Group B — state).
> BC-006: Binding-table cap (MAX_ARP_BINDINGS=65,536 via LRU) (Group B — resource).
> BC-007: D12 L2/L3 sender mismatch (Group C — detection).
> BC-008: D3 ARP storm rate detection (Group D — detection).
> BC-009: D11 malformed ARP finding (Group E — detection).
> BC-010: summarize() stats (Group F).
> BCs 011-013: CLI integration — --arp flag, --arp-spoof-threshold, --arp-storm-rate (Group G).
> BC-014: GARP-that-conflicts upgrade to D1 spoof finding (Group H — escalation).
> BC-015: Decode-vs-analysis separation architectural invariant (Group I — invariant).
> Feature: issue-009-arp-security-analyzer; ADR-008; introduced v0.7.0-feature-arp.
> **MITRE techniques:** T0830 (Adversary-in-the-Middle, `MitreTactic::LateralMovement`),
> T1557.002 (ARP Cache Poisoning, `MitreTactic::CredentialAccess`). No new MitreTactic
> variants added (both variants already exist in mitre.rs per arp-architecture-delta.md §5).

| BC ID | Title | Priority | Status | Origin |
|-------|-------|----------|--------|--------|
| BC-2.16.001 | ARP Request Frame Correctly Parsed from ArpPacketSlice | P0 | [WRITTEN] | feature-009-F2 |
| BC-2.16.002 | ARP Reply Frame Correctly Parsed from ArpPacketSlice | P0 | [WRITTEN] | feature-009-F2 |
| BC-2.16.003 | Gratuitous ARP Detection — sender_ip == target_ip Classified as GARP | P0 | [WRITTEN] | feature-009-F2 | <!-- v1.6: D-068 — benign GARP now emits mitre_techniques=[] (no MITRE attribution); T0830+T1557.002 exclusively on GARP-that-conflicts path (BC-2.16.014). Description, PC5, Invariant 2, Invariant 3, EC-001, EC-002, EC-007, canonical vectors updated. --> <!-- v1.7: Pass-5 Architecture Anchors §3.3 conditional benign-GARP MITRE fix — unconditional form replaced with conditional; Architecture Anchor updated per D-068. --> <!-- v1.8: Pass-13 PC7 cross-story clarity note; no H1/title change -->
| BC-2.16.004 | ARP Spoof Detection — IP→MAC Rebind Emits MEDIUM then HIGH Finding | P0 | [WRITTEN] | feature-009-F2 |
| BC-2.16.005 | Binding-Table Update — Last-Seen MAC Wins for a Given IP | P0 | [WRITTEN] | feature-009-F2 | <!-- v1.4: F-B8-M01: PC1 tightened — sender_ip excludes both 0.0.0.0 and 255.255.255.255 per Invariant 5; test-infra note for VP-024 Sub-C (new_for_test, process_arp_for_test, bindings_snapshot) added -->
| BC-2.16.006 | Binding-Table Cap — Table Never Exceeds MAX_ARP_BINDINGS via LRU Eviction | P0 | [WRITTEN] | feature-009-F2 |
| BC-2.16.007 | D12 L2/L3 Sender Mismatch — Ethernet Src MAC != ARP Sender HW Addr | P0 | [WRITTEN] | feature-009-F2 |
| BC-2.16.008 | D3 ARP Storm Rate Detection — Source MAC Exceeds ARP_STORM_RATE_DEFAULT Frames/Sec | P1 | [WRITTEN] | feature-009-F2 |
| BC-2.16.009 | D11 Malformed ARP — Non-Ethernet/IPv4 HW/Proto Address Sizes Emit LOW Finding | P1 | [WRITTEN] | feature-009-F2 | <!-- v1.3: F-B8-L02: PC4 --arp-absent clause clarified — malformed_frames increments unconditionally outside the analysis gate; note distinguishes PC4's outer precondition scope from counter behavior -->
| BC-2.16.010 | ArpAnalyzer::summarize() Returns AnalysisSummary with Required Keys (11 Keys) | P1 | [WRITTEN] | feature-009-F2 | <!-- v1.6: corpus-consistency-audit-2026-06-13 PR-1a/PR-1b: H1 enriched with "(11 Keys)" per Criterion-75; version suffix "; v1.5" removed from title (version belongs in frontmatter only) -->
| BC-2.16.011 | --arp CLI Flag Gates ARP Security Analysis | P0 | [WRITTEN] | feature-009-F2 |
| BC-2.16.012 | --arp-spoof-threshold Overrides SPOOF_REBIND_ESCALATION_DEFAULT | P1 | [WRITTEN] | feature-009-F2 |
| BC-2.16.013 | --arp-storm-rate Overrides ARP_STORM_RATE_DEFAULT | P1 | [WRITTEN] | feature-009-F2 |
| BC-2.16.014 | GARP-That-Conflicts Upgrades to MEDIUM and Triggers D1 Spoof Finding | P0 | [WRITTEN] | feature-009-F2 |
| BC-2.16.015 | Decode-vs-Analysis Separation — DecodedFrame::Arp Always Produced; Analysis Gated on --arp | P0 | [WRITTEN] | feature-009-F2 |

---

## Ingestion-to-L3 Mapping Coverage

| Ingestion group | Count | Mapped to L3 |
|----------------|-------|--------------|
| BC-RDR-001..008 | 8 | BC-2.01.001..008 |
| BC-DEC-001..015 | 15 | BC-2.02.001..015 (BC-2.02.009 revised to v1.6 in F2 ARP delta) |
| BC-RAS-001..054 + issue-#100 F2 | 55 | BC-2.04.001..055 |
| BC-DSP-001..009 | 9 | BC-2.05.001..009 |
| BC-HTTP-001..026 | 26 | BC-2.06.001..026 |
| BC-TLS-001..037 | 37 | BC-2.07.001..037 |
| BC-DNS-001..004 | 4 | BC-2.08.001..004 |
| BC-FND-001..006 + issue-#100 F2 | 7 | BC-2.09.001..007 |
| BC-MIT-001..009 | 9 | BC-2.10.001..009 |
| BC-RPT-001..019 | 19 | BC-2.11.001..019 |
| pass-4 H-1 (CsvReporter) | 5 | BC-2.11.020..024 |
| feature-259-F2 collapse (greenfield) | 5 | BC-2.11.025..029 |
| BC-CLI-001..017 | 17 | BC-2.12.001..017 |
| BC-SUM-001..004 | 4 | BC-2.12.018..021 |
| BC-ABS-001..010 | 10 | BC-2.13.001..004 (6 ABS retired by remediation cycle) |
| feature-007-F2 Modbus/ICS (greenfield) | 25 | BC-2.14.001..025 |
| feature-008-F2 DNP3/ICS (greenfield) | 24 | BC-2.15.001..024 |
| feature-009-F2 ARP security (greenfield) | 15 | BC-2.16.001..015 |

**Total BCs: 288. Canonical derivation: 218 draft ingestion BCs produced − 6 retired (BC-ABS-004..009) = 212 active from ingestion; + 5 post-ingestion pass-4 additions (BC-2.11.020..024) = 217; + 2 Feature Mode F2 additions (BC-2.04.055, BC-2.09.007) for issue #100 = 219 active BCs; + 25 Feature Mode F2 additions (BC-2.14.001..025) for issue #7 Modbus/ICS analyzer = 244 active BCs; + 22 Feature Mode F2 additions (BC-2.15.001..022) for issue #8 DNP3/ICS analyzer = 266 active BCs; + 2 research must-add additions (BC-2.15.023..024) for issue #8 post-gate F2 scope validation = 268 active BCs; + 15 Feature Mode F2 additions (BC-2.16.001..015) for issue #9 ARP security analyzer = 283 active BCs; + 5 Feature Mode F2 additions (BC-2.11.025..029) for issue #259 terminal finding collapse (v0.8.0) = 288 active BCs. BC-2.02.009 was revised to v1.6 (ADR-008 Decision 1, three-way postcondition) — a revision, not a new BC; count unchanged at each prior step. The mapping table above has 223 physical rows (218 ingestion-batch rows + 5 pass-4 rows) for pre-Modbus BCs; SS-14 adds 25 greenfield rows not in the ingestion batch; SS-15 adds 24 greenfield rows; SS-16 adds 15 greenfield rows; issue-#259 adds 5 greenfield rows to SS-11.**

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
