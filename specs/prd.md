---
document_type: prd
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/domain/domain-spec.md
  - .factory/specs/domain/domain-debt.md
  - .factory/specs/domain/invariants/inv-01-core-invariants.md
  - .factory/specs/domain/capabilities/cap-01-pcap-ingestion.md
  - .factory/specs/domain/capabilities/cap-02-link-type-gating.md
  - .factory/specs/domain/capabilities/cap-03-packet-decoding.md
  - .factory/specs/domain/capabilities/cap-04-tcp-reassembly.md
  - .factory/specs/domain/capabilities/cap-05-content-first-dispatch.md
  - .factory/specs/domain/capabilities/cap-06-http-analysis.md
  - .factory/specs/domain/capabilities/cap-07-tls-analysis.md
  - .factory/specs/domain/capabilities/cap-08-dns-analysis.md
  - .factory/specs/domain/capabilities/cap-09-finding-emission.md
  - .factory/specs/domain/capabilities/cap-10-mitre-mapping.md
  - .factory/specs/domain/capabilities/cap-11-reporting-output.md
  - .factory/semport/wirerust/wirerust-pass-3-behavioral-contracts.md
  - .factory/semport/wirerust/wirerust-pass-3-deep-behavioral-contracts-r4.md
input-hash: "[md5-TBD]"
traces_to: .factory/specs/domain/domain-spec.md
supplements:
  - prd-supplements/interface-definitions.md
  - prd-supplements/error-taxonomy.md
  - prd-supplements/test-vectors.md
  - prd-supplements/nfr-catalog.md
---

# Product Requirements Document: wirerust

> **Brownfield Mode:** This PRD is DESCRIPTIVE of the shipped system as of develop HEAD (post
> remediation-cycle PRs #69-#98, reconciled against 0082a0c). Every requirement is grounded in
> verified source evidence. Known gaps are recorded as debt (O-01..O-08), not silently omitted.
> Do NOT treat this document as aspirational -- it specifies what the system does today.

> **BC Index Model:** This PRD is an index document. Each Behavioral Contract (BC) lives in its
> own file under `behavioral-contracts/ss-NN/`. The tables below provide one-line summaries
> linking to individual BC files. Full contract details are NOT inlined here.

> **Supplement Model:** Sections 3-5 reference extracted supplement files under
> `prd-supplements/`. These supplements are produced in a SEPARATE burst (Phase 1b).
> Entries in those sections are summary stubs until the supplement burst completes.


## 1. Product Overview

### 1.1 Problem Statement

Network security analysts and incident responders must triage captured network traffic for
indicators of compromise. Existing tools (Wireshark, Zeek, Suricata) require network
connectivity, complex configuration, or ongoing daemon processes. Analysts working on isolated
forensic workstations need a single-binary tool that produces structured, machine-readable
findings from pcap captures without any runtime infrastructure.

Additionally, existing tools often sanitize or alter attacker-controlled data during analysis,
destroying forensic fidelity. A raw HTTP URI containing C0 control bytes looks different after
being processed by a display-layer renderer -- yet the raw bytes are the evidence.

### 1.2 Solution Vision

wirerust is an offline, single-binary, single-pass forensic triage CLI that ingests classic-pcap
captures and emits structured findings about HTTP, TLS, and DNS traffic plus TCP stream-reassembly
anomalies. It has no network I/O, no async runtime, no unsafe blocks, and no process-to-process
state. The binary is the complete deployment unit.

The core design principle is "trustworthy forensic data preservation plus display-layer safety":
raw attacker-controlled bytes survive intact through every layer to JSON output; the terminal
renderer is the sole owner of escape logic. This ensures SIEM consumers see unaltered forensic
data while terminal operators are protected from terminal injection attacks.

Architecture: 5-layer synchronous pipeline (Entry -> Ingest -> Stream -> Domain -> Output), 24
Rust source files, 3,868 source LOC, 282 tests, single crate, Rust 2024 edition, MSRV 1.91.

### 1.3 Key Differentiators

| ID | Differentiator | Description |
|----|---------------|-------------|
| KD-001 | Offline single-binary deployment | No daemon, no network I/O, no runtime dependencies. Suitable for air-gapped forensic workstations. |
| KD-002 | Forensic-fidelity raw-data contract | Attacker-controlled bytes (URIs, SNI hostnames, payloads) pass through unmodified to JSON output; escape runs only at terminal display (ADR 0003). |
| KD-003 | Content-first protocol identification | Protocol dispatch inspects TCP payload bytes before port numbers, defeating port-obfuscation attacks (ADR 0001). |
| KD-004 | First-wins TCP overlap forensics | Conflicting retransmissions are detected and emitted as findings; attackers cannot silently insert alternate bytes (INV-3). |
| KD-005 | MITRE ATT&CK tactic-grouped output | Findings carry structured MITRE technique IDs; terminal output can group by tactic for kill-chain analysis. |
| KD-006 | SNI anomaly detection with 4-way classification | TLS SNI hostnames are classified into four categories (clean ASCII, C0/DEL-containing, non-ASCII UTF-8, non-UTF-8 bytes) each triggering distinct findings. |
| KD-007 | Bounded-resource design | MAX_FINDINGS cap (10,000), per-direction buffer caps (65 KB), configurable reassembly thresholds with CLI override, no unbounded accumulation paths (except O-06). |

### 1.4 Target Users

| Persona | Description | Volume | Pain Level |
|---------|-------------|--------|------------|
| Forensic analyst | Processes pcap captures from incident response collections on isolated workstations | Low volume, high frequency during IR | High -- needs structured output fast, cannot install complex tooling |
| SOC operator | Bulk-processes pcap archives for indicator extraction, feeds output into SIEM | Medium volume, batch mode | High -- JSON output must be machine-parseable, not display-oriented |
| Malware researcher | Analyzes C2 traffic patterns, TLS fingerprinting, HTTP evasion techniques | Low volume, deep analysis | Medium -- needs JA3/JA3S and SNI anomaly details |
| Security toolchain integrator | Uses wirerust as a preprocessing stage in a pipeline (jq, grep, awk on JSON output) | High volume, automated | Medium -- needs deterministic JSON key order, stable exit codes |

### 1.5 Out of Scope

> Machine-consumed constraint list. The adversary and consistency-validator check that no story
> AC implements any feature listed here. Be explicit and unambiguous.

- pcapng format support (wirerust reads classic pcap ONLY; pcapng files are rejected at the reader boundary)
- Live network capture / sniffing (no network I/O of any kind; offline pcap files only)
- HTTP/2 or HTTP/3 analysis (HTTP/1.x only; H2 frames will be parsed as unknown bytes)
- DNS-based detection findings (DNS is statistics-only: query/response counts only; no NXDOMAIN flood, no tunneling detection)
- TLS decryption or certificate validation (SNI and cipher fingerprinting only; no key material involved)
- BPF filtering (--filter flag removed by PR #74; clap rejects --filter as unknown argument; out of scope for current release)
- C2 beacon detection (--beacon flag removed by PR #74; clap rejects --beacon as unknown argument; no beacon analyzer exists)
- --threats flag behavior (flag removed by PR #74; clap rejects --threats as unknown argument; no corresponding analyzer)
- --verbose flag (removed by PR #74 alongside --filter/--beacon/--threats; clap rejects --verbose as unknown argument; no verbosity levels defined)
- --services flag on summary subcommand (removed by PR #74; clap rejects --services as unknown argument; per-service breakdown is out of scope for current release)
- Parallel file processing (rayon = "1" is a declared production dependency but is entirely unused in src/; single-threaded only)
- Streaming / lazy-read pcap processing (entire file loaded into RAM before processing)
- Per-packet timestamp in findings (Finding.timestamp is always None; O-01)
- Empirically-calibrated anomaly thresholds (defaults are research-documented but not validated against labelled traffic; O-03)
- MITRE techniques T1040, T1071, T1071.001, T1071.004, T1573, T0846, T0855, T0856, T0885 (catalogued but never emitted; O-04)


## 2. Behavioral Contracts Index

> BCs are organized by L2 domain capability (CAP-NN). BC numbering: BC-2.NN.NNN where
> 2 = PRD section, NN = capability number, NNN = sequential within capability.
> Files live in `behavioral-contracts/ss-NN/BC-2.NN.NNN.md`.

### 2.1 PCAP File Ingestion (CAP-01)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.01.001 | Accept supported link types and reject unsupported at file open | P0 | BC-RDR-001 |
| BC-2.01.002 | Read all packets from pcap as Vec<RawPacket> preserving timestamps | P0 | BC-RDR-002 |
| BC-2.01.003 | Accept pcap with zero packets (header-only) without error | P1 | BC-RDR-003 |
| BC-2.01.004 | Reject pcapng-format input at reader level | P0 | BC-RDR-004 |
| BC-2.01.005 | Convert pcap record timestamp to (timestamp_secs: u32, timestamp_usecs: u32) | P1 | BC-RDR-005 |
| BC-2.01.006 | Surface pcap header parse errors with anyhow context | P1 | BC-RDR-006 |
| BC-2.01.007 | Surface per-packet read errors with anyhow context | P1 | BC-RDR-007 |
| BC-2.01.008 | from_file opens via BufReader and delegates to from_pcap_reader | P2 | BC-RDR-008 |

> Full contracts: `behavioral-contracts/ss-01/BC-2.01.001.md` through `BC-2.01.008.md`

### 2.2 Link-Type Gating (CAP-02)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.02.001 | Decode Ethernet-framed IPv4 TCP packet to ParsedPacket | P0 | BC-DEC-001 |
| BC-2.02.002 | Decode Ethernet-framed IPv4 UDP packet with DNS hint | P0 | BC-DEC-002 |
| BC-2.02.003 | Decode RAW link-layer IPv4 TCP packet via from_ip | P0 | BC-DEC-003 |
| BC-2.02.004 | DataLink::IPV4 decodes identically to DataLink::RAW | P1 | BC-DEC-004 |
| BC-2.02.005 | Decode RAW IPv6 TCP packet surfacing IPv6 addresses | P0 | BC-DEC-005 |
| BC-2.02.006 | Decode Linux SLL (cooked) TCP packets | P0 | BC-DEC-006 |
| BC-2.02.007 | Reject malformed input bytes with anyhow error (no panic) | P0 | BC-DEC-007 |
| BC-2.02.008 | Reject unsupported link types in decode_packet | P1 | BC-DEC-008 |
| BC-2.02.009 | Surface No IP layer found error | P1 | BC-DEC-009 |
| BC-2.02.010 | Classify ICMP as Protocol::Icmp with TransportInfo::None | P1 | BC-DEC-010 |
| BC-2.02.011 | Classify other IP protocols as Protocol::Other(byte) | P1 | BC-DEC-011 |
| BC-2.02.012 | app_protocol_hint returns service strings from port number | P1 | BC-DEC-012 |
| BC-2.02.013 | app_protocol_hint returns None when TransportInfo is None | P2 | BC-DEC-013 |
| BC-2.02.014 | packet_len is set to total frame length not just payload | P1 | BC-DEC-014 |
| BC-2.02.015 | Extract TCP control flags and sequence number into TransportInfo::Tcp | P0 | BC-DEC-015 |

> Full contracts: `behavioral-contracts/ss-02/BC-2.02.001.md` through `BC-2.02.015.md`

### 2.3 Packet Decoding (CAP-03)

> CAP-03 BCs are co-located with CAP-02 in ss-02 because the decoder is the single component
> (C-5) implementing both capabilities. The BC-DEC-NNN ingestion IDs map to BC-2.02.NNN above.
> No separate ss-03 directory is required for this capability.

### 2.4 TCP Stream Reassembly (CAP-04)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.04.001 | TcpReassembler::new panics on invalid config (defensive assert) | P1 | BC-RAS-001 |
| BC-2.04.002 | Non-TCP packets are skipped and packets_skipped_non_tcp increments | P1 | BC-RAS-002 |
| BC-2.04.003 | Canonical FlowKey ordering ensures A->B and B->A produce identical key | P0 | BC-RAS-003 |
| BC-2.04.004 | First SYN sets client ISN and initiator | P0 | BC-RAS-004 |
| BC-2.04.005 | SYN+ACK marks server as responder and transitions state to Established | P0 | BC-RAS-005 |
| BC-2.04.006 | Bidirectional data delivered with correct Direction tag | P0 | BC-RAS-006 |
| BC-2.04.007 | In-order data flushes contiguously to handler in segment order | P0 | BC-RAS-007 |
| BC-2.04.008 | Out-of-order segments buffer until gap filled then flush contiguously | P0 | BC-RAS-008 |
| BC-2.04.009 | Mid-stream join infers ISN from first-data seq-1 and marks flow partial | P0 | BC-RAS-009 |
| BC-2.04.010 | RST closes flow immediately with CloseReason::Rst and zeroes total_memory | P0 | BC-RAS-010 |
| BC-2.04.011 | Both FINs close flow with CloseReason::Fin and remove from table | P0 | BC-RAS-011 |
| BC-2.04.012 | finalize flushes all remaining flows with Timeout and is idempotent | P0 | BC-RAS-012 |
| BC-2.04.013 | expire_flows closes flows idle past flow_timeout_secs with Timeout | P1 | BC-RAS-013 |
| BC-2.04.014 | total_memory tracks buffered bytes and decrements on flush and close | P1 | BC-RAS-014 |
| BC-2.04.015 | Flow eviction on max_flows hit uses LRU non-established-first policy | P1 | BC-RAS-015 |
| BC-2.04.016 | Memory pressure eviction when total_memory exceeds memcap | P1 | BC-RAS-016 |
| BC-2.04.017 | Eviction sort: non-established first, then oldest-last-seen within band | P1 | BC-RAS-017 |
| BC-2.04.018 | Conflicting overlap emits Anomaly/Likely/High finding with MITRE T1036 | P0 | BC-RAS-018 |
| BC-2.04.019 | Excessive overlaps (>threshold) emit one-shot T1036 finding | P0 | BC-RAS-019 |
| BC-2.04.020 | Excessive small segments (>threshold) emit one-shot finding | P1 | BC-RAS-020 |
| BC-2.04.021 | Excessive out-of-window segments (>threshold) emit one-shot Low finding | P1 | BC-RAS-021 |
| BC-2.04.022 | Per-direction alert fires at most once per flow (sticky latch) | P0 | BC-RAS-022 |
| BC-2.04.023 | Truncated segment emits Anomaly/Inconclusive/Low finding (no MITRE) | P1 | BC-RAS-023 |
| BC-2.04.024 | Total findings capped at MAX_FINDINGS=10000; excess silently dropped | P0 | BC-RAS-024 |
| BC-2.04.025 | finalize emits segment-limit summary finding when segments dropped (with pluralization) | P0 | BC-RAS-025 |
| BC-2.04.026 | finalize does NOT emit segment-limit finding when counter is zero | P0 | BC-RAS-026 |
| BC-2.04.027 | segments_depth_exceeded tracks fully-rejected segments after depth hit | P1 | BC-RAS-027 |
| BC-2.04.028 | summarize returns AnalysisSummary with reassembly stats detail map | P1 | BC-RAS-028 |
| BC-2.04.029 | close_flow for missing key logs one-shot process-wide warning | P2 | BC-RAS-029 |
| BC-2.04.030 | bytes_reassembled equals total bytes delivered to handler at end | P1 | BC-RAS-030 |
| BC-2.04.031 | ISN set on first SYN; inferred as seq-1 on data-without-SYN | P0 | BC-RAS-031 |
| BC-2.04.032 | insert_segment with no ISN returns IsnMissing and inserts nothing | P0 | BC-RAS-032 |
| BC-2.04.033 | Single segment insertion returns Inserted and stores under offset key | P0 | BC-RAS-033 |
| BC-2.04.034 | flush_contiguous consumes segments from base_offset in order | P0 | BC-RAS-034 |
| BC-2.04.035 | Identical retransmission returns Duplicate and does not double-count bytes | P0 | BC-RAS-035 |
| BC-2.04.036 | First-wins overlap: gap bytes added, existing bytes preserved | P0 | BC-RAS-036 |
| BC-2.04.037 | Same-range conflicting overlap returns ConflictingOverlap, preserves original | P0 | BC-RAS-037 |
| BC-2.04.038 | Multi-segment full coverage returns Duplicate or ConflictingOverlap as appropriate | P0 | BC-RAS-038 |
| BC-2.04.039 | TCP sequence wraparound across 32-bit boundary reassembles correctly | P0 | BC-RAS-039 |
| BC-2.04.040 | Small-segment counter increments per direction for segments under threshold | P1 | BC-RAS-040 |
| BC-2.04.041 | Depth truncation: segment crossing max_depth is truncated to remaining capacity | P0 | BC-RAS-041 |
| BC-2.04.042 | Segment beyond max_receive_window returns OutOfWindow; boundary segment accepted | P1 | BC-RAS-042 |
| BC-2.04.043 | Adjacent segments at exact boundary do not count as overlap | P0 | BC-RAS-043 |
| BC-2.04.044 | Segments map full: non-overlapping insert returns SegmentLimitReached | P0 | BC-RAS-044 |
| BC-2.04.045 | Segments map full: overlapping insert needing gap insertion returns SegmentLimitReached | P0 | BC-RAS-045 |
| BC-2.04.046 | Segments map fills mid-loop: partial insertion with later gaps dropped | P0 | BC-RAS-046 |
| BC-2.04.047 | buffered_bytes mirrors segment size sum after all insert/overlap/flush ops | P0 | BC-RAS-047 |
| BC-2.04.048 | ISN_MISSING_WARNED atomic prevents repeated eprintln on missing-ISN errors | P2 | BC-RAS-048 |
| BC-2.04.049 | FlowKey::Display formats as lower_ip:lower_port -> upper_ip:upper_port with U+2192 | P1 | BC-RAS-049 |
| BC-2.04.050 | Flow state machine: New->SynSent->Established->Closing->Closed transitions | P0 | BC-RAS-050 |
| BC-2.04.051 | RST transitions state to Closed from any prior state | P0 | BC-RAS-051 |
| BC-2.04.052 | on_data_without_syn transitions New->Established and sets partial=true | P0 | BC-RAS-052 |
| BC-2.04.053 | TcpFlow::direction returns ClientToServer when src matches initiator | P0 | BC-RAS-053 |
| BC-2.04.054 | finalize unconditionally bypasses MAX_FINDINGS cap for segment-limit finding | P0 | BC-RAS-054 |

> Full contracts: `behavioral-contracts/ss-04/BC-2.04.001.md` through `BC-2.04.054.md`

### 2.5 Content-First Protocol Dispatch (CAP-05)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.05.001 | TLS content signature routes flow to TLS regardless of port | P0 | BC-DSP-001 |
| BC-2.05.002 | HTTP method prefix routes flow to HTTP | P0 | BC-DSP-002 |
| BC-2.05.003 | Port fallback: 443/8443->TLS, 80/8080->HTTP when content insufficient | P0 | BC-DSP-003 |
| BC-2.05.004 | Unknown content and unknown port returns DispatchTarget::None | P1 | BC-DSP-004 |
| BC-2.05.005 | Classification cached per FlowKey after first non-None result | P0 | BC-DSP-005 |
| BC-2.05.006 | DispatchTarget::None NOT cached until retry cap (default 8); reclassification retried per on_data until cap, then None cached permanently | P0 | BC-DSP-006 |
| BC-2.05.007 | unclassified_flows increments only at on_flow_close for never-classified flows | P1 | BC-DSP-007 |
| BC-2.05.008 | No analyzer configured: dispatcher early-returns from on_data | P1 | BC-DSP-008 |
| BC-2.05.009 | on_flow_close removes route entry and forwards close to analyzer | P0 | BC-DSP-009 |

> Full contracts: `behavioral-contracts/ss-05/BC-2.05.001.md` through `BC-2.05.009.md`

### 2.6 HTTP Traffic Analysis (CAP-06)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.06.001 | Parse complete HTTP/1.1 request extracting method, URI, version, Host, User-Agent | P0 | BC-HTTP-001 |
| BC-2.06.002 | Parse pipelined requests with independent per-request method/uri counting | P0 | BC-HTTP-002 |
| BC-2.06.003 | Partial requests buffered until complete; not counted until full | P0 | BC-HTTP-003 |
| BC-2.06.004 | Parse HTTP/1.1 responses with status code counting and transaction advance | P0 | BC-HTTP-004 |
| BC-2.06.005 | Path traversal in URI emits Reconnaissance/Likely/High finding mapped to T1083 | P0 | BC-HTTP-005 |
| BC-2.06.006 | Web-shell URI patterns emit Execution/Likely/Medium finding mapped to T1505.003 | P0 | BC-HTTP-006 |
| BC-2.06.007 | Admin panel paths emit Reconnaissance/Inconclusive/Low finding mapped to T1046 | P1 | BC-HTTP-007 |
| BC-2.06.008 | Unusual HTTP methods emit Reconnaissance/Inconclusive/Medium finding (no MITRE) | P1 | BC-HTTP-008 |
| BC-2.06.009 | HTTP/1.1 request without Host header emits Anomaly/Inconclusive/Medium finding | P0 | BC-HTTP-009 |
| BC-2.06.010 | URI longer than 2048 chars emits Execution/Likely/Medium finding with char count | P1 | BC-HTTP-010 |
| BC-2.06.011 | Empty (present-but-blank) User-Agent emits Anomaly/Inconclusive/Low finding; absent UA does NOT | P1 | BC-HTTP-011 |
| BC-2.06.012 | Well-formed HTTP request produces zero findings | P0 | BC-HTTP-012 |
| BC-2.06.013 | Non-HTTP bytes increment parse_errors but do not emit Token-error findings | P0 | BC-HTTP-013 |
| BC-2.06.014 | Too many headers (>96) emits Anomaly/Inconclusive/Medium finding mapped to T1499.002 | P0 | BC-HTTP-014 |
| BC-2.06.015 | After 3 consecutive parse errors a direction is poisoned; subsequent bytes skipped | P0 | BC-HTTP-015 |
| BC-2.06.016 | Single parse error does not poison; next valid request parses normally | P0 | BC-HTTP-016 |
| BC-2.06.017 | Poisoning is per-direction: poisoned request does not affect response | P0 | BC-HTTP-017 |
| BC-2.06.018 | non_http_flows counts a flow once even if both directions get poisoned | P1 | BC-HTTP-018 |
| BC-2.06.019 | on_flow_close removes per-flow state; reopening same FlowKey starts fresh | P0 | BC-HTTP-019 |
| BC-2.06.020 | HTTP body bytes after header completion do not inflate parse_errors | P1 | BC-HTTP-020 |
| BC-2.06.021 | Cross-flow isolation: parse errors and poisoning in one flow do not leak | P0 | BC-HTTP-021 |
| BC-2.06.022 | Per-direction header buffer capped at MAX_HEADER_BUF (65536 bytes) | P1 | BC-HTTP-022 |
| BC-2.06.023 | summarize emits AnalysisSummary with HTTP stats detail map | P1 | BC-HTTP-023 |
| BC-2.06.024 | Per-map cardinality cap: new keys dropped past MAX_MAP_ENTRIES (50000) | P2 | BC-HTTP-024 |
| BC-2.06.025 | uris list capped at MAX_URIS=10000; further URIs silently dropped | P2 | BC-HTTP-025 |
| BC-2.06.026 | Header value extraction uses from_utf8_lossy.trim(); raw bytes preserved per ADR 0003 | P0 | BC-HTTP-026 |

> Full contracts: `behavioral-contracts/ss-06/BC-2.06.001.md` through `BC-2.06.026.md`

### 2.7 TLS Traffic Analysis (CAP-07)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.07.001 | Parse complete TLS ClientHello: version, ciphers, extensions, SNI, JA3 | P0 | BC-TLS-001 |
| BC-2.07.002 | Parse complete TLS ServerHello: JA3S fingerprint computed | P0 | BC-TLS-002 |
| BC-2.07.003 | After both hellos seen, subsequent records silently skipped | P0 | BC-TLS-003 |
| BC-2.07.004 | TLS record payload > MAX_RECORD_PAYLOAD (18432) increments parse_errors and truncated_records | P0 | BC-TLS-004 |
| BC-2.07.005 | Per-direction buffer capped at MAX_BUF=65536 bytes | P1 | BC-TLS-005 |
| BC-2.07.006 | JA3 computation filters GREASE values per RFC 8701 | P0 | BC-TLS-006 |
| BC-2.07.007 | JA3 string format: version,ciphers,extensions,curves,pointfmts hyphen-joined; MD5 hex | P0 | BC-TLS-007 |
| BC-2.07.008 | JA3S string format: version,cipher,extensions hyphen-joined; MD5 hex | P0 | BC-TLS-008 |
| BC-2.07.009 | Weak client cipher (NULL/ANON/EXPORT in ClientHello) emits Anomaly/Likely/High finding | P0 | BC-TLS-009 |
| BC-2.07.010 | Weak server cipher selected (NULL/ANON/EXPORT/RC4) emits Anomaly/Likely/Medium finding | P0 | BC-TLS-010 |
| BC-2.07.011 | Deprecated client protocol (<=SSLv3) emits Anomaly/Likely/High finding citing RFC 7568 | P0 | BC-TLS-011 |
| BC-2.07.012 | Deprecated server protocol (<=SSLv3) emits Anomaly/Likely/High finding | P0 | BC-TLS-012 |
| BC-2.07.013 | Clean ASCII SNI without C0/DEL bytes produces no SNI-related finding | P0 | BC-TLS-013 |
| BC-2.07.014 | SNI containing C0/DEL byte emits Anomaly/Inconclusive/Low finding mapped to T1027 | P0 | BC-TLS-014 |
| BC-2.07.015 | Multiple control bytes in one SNI produce exactly ONE finding | P0 | BC-TLS-015 |
| BC-2.07.016 | C0 boundary: 0x1F trips the finding; 0x20 (space) does not | P0 | BC-TLS-016 |
| BC-2.07.017 | Non-ASCII but valid UTF-8 SNI emits Anomaly/Inconclusive/Low finding mapped to T1027 | P0 | BC-TLS-017 |
| BC-2.07.018 | Punycode A-label (xn--...) is pure ASCII and emits no SNI finding | P1 | BC-TLS-018 |
| BC-2.07.019 | Non-UTF-8 SNI bytes emit Anomaly/Inconclusive/Low finding mapped to T1027; count key tagged | P0 | BC-TLS-019 |
| BC-2.07.020 | Non-UTF-8 SNI summary preserves raw bytes (no Debug-format escaping per ADR 0003) | P0 | BC-TLS-020 |
| BC-2.07.021 | Non-ASCII UTF-8 SNI summary preserves raw bytes per ADR 0003 | P0 | BC-TLS-021 |
| BC-2.07.022 | SNI extension with empty ServerNameList: no count, no finding, handshake still counted | P1 | BC-TLS-022 |
| BC-2.07.023 | SNI with empty hostname bytes counts under "" key; no non-UTF-8 finding | P2 | BC-TLS-023 |
| BC-2.07.024 | Only FIRST ServerName entry in multi-name SNI list is processed | P1 | BC-TLS-024 |
| BC-2.07.025 | Non-zero NameType entries are passed through as hostnames (current tls-parser behavior) | P2 | BC-TLS-025 |
| BC-2.07.026 | Trailing bytes in ServerNameList tolerated; first hostname still extracted | P2 | BC-TLS-026 |
| BC-2.07.027 | Large SNI (16 KB) under MAX_RECORD_PAYLOAD parses successfully | P1 | BC-TLS-027 |
| BC-2.07.028 | sni_counts cap at MAX_MAP_ENTRIES silently drops keys; SNI anomaly finding still fires | P0 | BC-TLS-028 |
| BC-2.07.029 | Bad TLS record body increments parse_errors and does not panic | P0 | BC-TLS-029 |
| BC-2.07.030 | Normal handshake (strong cipher) produces zero findings | P0 | BC-TLS-030 |
| BC-2.07.031 | summarize emits AnalysisSummary with TLS stats detail map | P1 | BC-TLS-031 |
| BC-2.07.032 | TLS 1.3 ClientHello legacy_version recorded as 0x0303 per JA3 spec | P1 | BC-TLS-032 |
| BC-2.07.033 | TLS analyzer ignores non-handshake records (record_type != 0x16) | P1 | BC-TLS-033 |
| BC-2.07.034 | After both hellos seen for flow, on_data short-circuits without buffering | P0 | BC-TLS-034 |
| BC-2.07.035 | on_flow_close drops per-flow TlsFlowState | P1 | BC-TLS-035 |
| BC-2.07.036 | Unknown cipher IDs render as hex 0xNNNN lowercase | P2 | BC-TLS-036 |
| BC-2.07.037 | SNI with both non-ASCII and C0 control bytes fires arm 3 (NonAsciiUtf8), not arm 2 | P0 | BC-TLS-037 |

> Full contracts: `behavioral-contracts/ss-07/BC-2.07.001.md` through `BC-2.07.037.md`

### 2.8 DNS Traffic Analysis (CAP-08)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.08.001 | DnsAnalyzer matches packets where src or dst port == 53 (TCP or UDP) | P0 | BC-DNS-001 |
| BC-2.08.002 | DNS QR-bit dispatch: response_count++ if bit set; query_count++ otherwise; returns empty findings | P0 | BC-DNS-002 |
| BC-2.08.003 | summarize emits AnalysisSummary with dns_queries and dns_responses counts | P1 | BC-DNS-003 |
| BC-2.08.004 | DnsAnalyzer NEVER emits findings (statistics-only by design) | P0 | BC-DNS-004 |

> Full contracts: `behavioral-contracts/ss-08/BC-2.08.001.md` through `BC-2.08.004.md`

### 2.9 Forensic Finding Emission (CAP-09)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.09.001 | Finding is constructed with required and optional fields as specified | P0 | BC-FND-001 |
| BC-2.09.002 | Finding Display renders [Category] VERDICT (CONFIDENCE) -- summary (raw text) | P1 | BC-FND-002 |
| BC-2.09.003 | Verdict Display: Likely/Unlikely/Inconclusive render as uppercase tokens | P1 | BC-FND-003 |
| BC-2.09.004 | Confidence Display: High/Medium/Low render as uppercase tokens | P1 | BC-FND-004 |
| BC-2.09.005 | Finding.summary and evidence store RAW post-from_utf8_lossy bytes per ADR 0003 | P0 | BC-FND-005 |
| BC-2.09.006 | Finding JSON serialization: timestamp and all None Option fields omitted via skip_serializing_if | P0 | BC-FND-006 |

> Full contracts: `behavioral-contracts/ss-09/BC-2.09.001.md` through `BC-2.09.006.md`
>
> Known limitation: All 22 emission sites set timestamp: None (domain-debt O-01). This is
> described by BC-2.09.001 as current behavior. Finding.timestamp field exists but is never populated.

### 2.10 MITRE ATT&CK Mapping (CAP-10)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.10.001 | MitreTactic Display renders Enterprise tactics with canonical spacing | P0 | BC-MIT-001 |
| BC-2.10.002 | ICS tactics render unprefixed (no ICS: prefix) | P1 | BC-MIT-002 |
| BC-2.10.003 | all_tactics_in_report_order returns kill-chain order first then ICS-unique | P0 | BC-MIT-003 |
| BC-2.10.004 | all_tactics_in_report_order contains every variant exactly once (16 total) | P0 | BC-MIT-004 |
| BC-2.10.005 | technique_name returns Some for every seeded ID (15 total) | P0 | BC-MIT-005 |
| BC-2.10.006 | technique_name returns None for unknown IDs | P0 | BC-MIT-006 |
| BC-2.10.007 | technique_tactic returns correct tactic for every seeded ID | P0 | BC-MIT-007 |
| BC-2.10.008 | All technique IDs currently emitted by analyzers resolve in lookup | P0 | BC-MIT-008 |
| BC-2.10.009 | MitreTactic is #[non_exhaustive] (adding variants is non-breaking) | P2 | BC-MIT-009 |

> Full contracts: `behavioral-contracts/ss-10/BC-2.10.001.md` through `BC-2.10.009.md`
>
> Domain debt O-04: 9 techniques are catalogued but never emitted (T1040, T1071, T1071.001,
> T1071.004, T1573, T0846, T0855, T0856, T0885). These are staged for future analyzers.
> BC-2.10.005 documents all 15 as present in the catalog; their emission status is out of scope.

### 2.11 Reporting and Output (CAP-11)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.11.001 | JsonReporter renders JSON object with summary, findings, analyzers keys | P0 | BC-RPT-001 |
| BC-2.11.002 | JsonReporter includes skipped_packets in summary (zero when unset) | P1 | BC-RPT-002 |
| BC-2.11.003 | JsonReporter escapes C0 control bytes per RFC 8259 via serde | P0 | BC-RPT-003 |
| BC-2.11.004 | JsonReporter preserves non-ASCII Unicode in readable form (no unnecessary \uNNNN) | P1 | BC-RPT-004 |
| BC-2.11.005 | JsonReporter passes C1 codepoints through as raw UTF-8 (serde_json does not escape them) | P1 | BC-RPT-005 |
| BC-2.11.006 | TerminalReporter shows Skipped: N packets only when N > 0 | P1 | BC-RPT-006 |
| BC-2.11.007 | TerminalReporter escapes C0+DEL+C1+backslash in finding summary and evidence | P0 | BC-RPT-007 |
| BC-2.11.008 | TerminalReporter escape preserves printable ASCII, Cyrillic, emoji, mixed Unicode | P0 | BC-RPT-008 |
| BC-2.11.009 | TerminalReporter escapes C1 codepoints U+0080-U+009F; U+00A0 is preserved | P0 | BC-RPT-009 |
| BC-2.11.010 | TerminalReporter escapes both Finding.summary AND each evidence line | P0 | BC-RPT-010 |
| BC-2.11.011 | TerminalReporter escapes analyzer-summary detail values (closes C1 gap) | P0 | BC-RPT-011 |
| BC-2.11.012 | TerminalReporter end-to-end: C1 CSI in path-traversal finding is escaped | P0 | BC-RPT-012 |
| BC-2.11.013 | MITRE grouping emits tactic headers in all_tactics_in_report_order; Uncategorized last | P0 | BC-RPT-013 |
| BC-2.11.014 | Within tactic bucket findings sort by verdict then confidence then emission order | P1 | BC-RPT-014 |
| BC-2.11.015 | No-technique or unknown-ID findings land in Uncategorized; unknown IDs get (unknown) label | P0 | BC-RPT-015 |
| BC-2.11.016 | MITRE grouping expands per-finding line with em-dash and technique name for known IDs | P1 | BC-RPT-016 |
| BC-2.11.017 | Default (flag-off) rendering emits MITRE: <id> only with no em-dash | P1 | BC-RPT-017 |
| BC-2.11.018 | TerminalReporter colorization: Likely/High=red bold, Likely/other=yellow, Inconclusive=cyan, Unlikely=dimmed | P2 | BC-RPT-018 |
| BC-2.11.019 | TerminalReporter renders sections in order: header, PROTOCOLS, SERVICES, FINDINGS, ANALYZER summaries | P1 | BC-RPT-019 |

> Full contracts: `behavioral-contracts/ss-11/BC-2.11.001.md` through `BC-2.11.024.md`
> (BC-2.11.020–024 added adversarial-review pass-4: CsvReporter coverage gap H-1)

### 2.12 CLI and Entry Point (CAP-01 / Cross-cutting)

> CLI BCs are cross-cutting: they describe the entry point (C-1..C-3) that wires all capabilities
> together. Numbered under ss-12 for organizational clarity.

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.12.001 | analyze subcommand parses positional targets and all analysis flags | P0 | BC-CLI-001 |
| BC-2.12.002 | summary subcommand parses positional targets and --hosts flag | P1 | BC-CLI-002 |
| BC-2.12.003 | Global flag --no-color is parsed and stored | P1 | BC-CLI-003 |
| BC-2.12.004 | Global flag --output-format json parses to Some(OutputFormat::Json); default is None | P0 | BC-CLI-004 |
| BC-2.12.005 | Reassembly CLI flags: --reassemble/--no-reassemble, depth, memcap, and five anomaly-threshold flags | P0 | BC-CLI-005 |
| BC-2.12.006 | Multiple positional targets accepted in analyze | P1 | BC-CLI-006 |
| BC-2.12.007 | --reassemble and --no-reassemble are mutually exclusive (clap conflicts_with) | P0 | BC-CLI-007 |
| BC-2.12.008 | --all enables dns/http/tls together (boolean OR semantics) | P1 | BC-CLI-008 |
| BC-2.12.009 | needs_reassembly = (--reassemble OR --http OR --tls); --no-reassemble forces off with warning | P0 | BC-CLI-009 |
| BC-2.12.010 | NO_COLOR env var disables color even without --no-color flag | P2 | BC-CLI-010 |
| BC-2.12.011 | Directory target expands to all *.pcap files sorted; *.pcapng excluded from glob | P1 | BC-CLI-011 |
| BC-2.12.012 | Non-existent target yields bail! with Target not found message | P1 | BC-CLI-012 |
| BC-2.12.013 | Per-target progress bar on stderr using indicatif | P2 | BC-CLI-013 |
| BC-2.12.014 | Per-target decode errors counted into skipped_packets; only first error printed to stderr | P1 | BC-CLI-014 |
| BC-2.12.015 | dispatcher.unclassified_flows() injected into reassembly AnalysisSummary detail | P1 | BC-CLI-015 |
| BC-2.12.016 | --output-format json picks JsonReporter; --output-format csv picks CsvReporter; default terminal | P0 | BC-CLI-016 |
| BC-2.12.017 | Output routed: file path if --json <FILE> or --csv <FILE> given; stdout otherwise | P0 | BC-CLI-017 |
| BC-2.12.018 | Summary::ingest increments total_packets, total_bytes, hosts, protocol counters | P0 | BC-SUM-001 |
| BC-2.12.019 | Summary::ingest derives service name from app_protocol_hint and increments service counter | P1 | BC-SUM-002 |
| BC-2.12.020 | Summary::unique_hosts returns sorted deduplicated Vec<IpAddr> | P1 | BC-SUM-003 |
| BC-2.12.021 | Summary serializes with total_packets, total_bytes, skipped_packets fields | P1 | BC-SUM-004 |

> Full contracts: `behavioral-contracts/ss-12/BC-2.12.001.md` through `BC-2.12.021.md`

### 2.13 Absent / Unwired Feature Contracts (Documented Current Behavior)

> These BCs document flags or behaviors that do not exist in the current codebase (removed by
> PR #74). clap rejects all four as unknown arguments; there is no runtime behavior for any of
> them. They are HIGH-confidence absent contracts verified against src/cli.rs.

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.13.001 | --threats flag does not exist; clap rejects it as unknown argument | P0 (absent) | BC-ABS-001 |
| BC-2.13.002 | --beacon flag does not exist; no C2 beacon analyzer exists | P0 (absent) | BC-ABS-002 |
| BC-2.13.003 | --filter <BPF> flag does not exist; no BPF filter applied | P0 (absent) | BC-ABS-003 |
| BC-2.13.004 | --verbose flag does not exist; no verbose logging mode | P2 (absent) | BC-ABS-010 |

> Full contracts: `behavioral-contracts/ss-13/BC-2.13.001.md` through `BC-2.13.004.md`


## 3. Interface Definition

> **Supplement:** Full interface definitions are in `prd-supplements/interface-definitions.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

Summary: wirerust exposes a single CLI binary. Subcommands: `analyze` (produces findings),
`summary` (produces protocol/host overview). Global flags include `--output-format`,
`--no-color`, `--reassemble`, `--no-reassemble`, reassembly threshold overrides, and file
output paths (`--json <FILE>`, `--csv <FILE>`). Exit codes: 0=success, 1=fatal error.
See `prd-supplements/interface-definitions.md` for the complete flag reference, exit code
semantics, JSON output schema, and flag interaction rules.


## 4. Non-Functional Requirements

> **Supplement:** Full NFR catalog is in `prd-supplements/nfr-catalog.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

The NFR catalog (79 entries from pass-4) covers categories: PERF (throughput and latency),
SEC (memory safety, no unsafe, injection prevention), REL (overflow checks, saturating
arithmetic), OBS (counters for dropped findings, truncated records, poisoned bytes),
RES (MAX_FINDINGS cap, buffer caps, map cardinality caps), MNT (MSRV, test coverage),
PORT (Rust 2024 edition), SUP (MITRE version), COMPAT (pcap classic only).
See `prd-supplements/nfr-catalog.md` for NFR-NNN entries with numerical targets.

Known NFR violation: NFR-VIO-001 -- README's "multi-GB captures" claim is only accurate
under matching RAM constraints (eager full-file load; O-01 context).


## 5. Error Taxonomy

> **Supplement:** Full error taxonomy is in `prd-supplements/error-taxonomy.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

Errors follow anyhow chaining patterns. Key categories:
- E-INP-NNN: Input / File errors (header parse failure, unsupported link type, file open failure, packet read failure)
- E-DEC-NNN: Decoder errors (unsupported link type, no IP layer, etherparse parse failure)
- E-RAS-NNN: Reassembly errors (lifecycle state-machine edge cases and resource limits)
- E-ANA-NNN: Analyzer errors (HTTP, TLS, DNS protocol-level parse failures)
- E-OUT-NNN: Output errors (file write failures for --json/--csv paths)
- E-CFG-NNN: Configuration errors (mutually exclusive flag combinations rejected by clap)
See `prd-supplements/error-taxonomy.md` for the complete E-xxx-NNN catalog.


## 6. Competitive Differentiator Traceability

> Maps each key differentiator (Section 1.3) to the behavioral contracts that implement it.

### 6.1 KD-001: Offline Single-Binary Deployment

| BC ID | Contribution |
|-------|-------------|
| BC-2.01.001 | Link-type gating at read time: no network call needed |
| BC-2.01.002 | Eager full-file load into memory: no streaming or daemon state |
| BC-2.12.016 | All three output reporters (terminal, JSON, CSV) are self-contained |

### 6.2 KD-002: Forensic-Fidelity Raw-Data Contract

| BC ID | Contribution |
|-------|-------------|
| BC-2.09.005 | Finding.summary and evidence carry RAW post-from_utf8_lossy bytes (ADR 0003) |
| BC-2.11.003 | JsonReporter uses serde RFC 8259 escaping; does NOT call escape_for_terminal |
| BC-2.11.007 | TerminalReporter is the SOLE caller of escape_for_terminal |
| BC-2.07.020 | TLS SNI non-UTF-8 bytes preserved raw in Finding.summary |
| BC-2.07.021 | TLS SNI non-ASCII UTF-8 bytes preserved raw in Finding.summary |
| BC-2.06.026 | HTTP header bytes preserved raw at analyzer layer |

### 6.3 KD-003: Content-First Protocol Identification

| BC ID | Contribution |
|-------|-------------|
| BC-2.05.001 | 0x16 0x03 content signature routes to TLS regardless of port |
| BC-2.05.002 | HTTP method prefix routes to HTTP regardless of port |
| BC-2.05.003 | Port fallback only when content is insufficient (5 bytes minimum) |
| BC-2.05.005 | Classification cached per flow for efficiency |
| BC-2.05.006 | DispatchTarget::None not cached until retry cap (default 8); late protocol identification retried until cap, then permanently cached as None |

### 6.4 KD-004: First-Wins TCP Overlap Forensics

| BC ID | Contribution |
|-------|-------------|
| BC-2.04.036 | First-wins: gap bytes added; existing bytes preserved on partial overlap |
| BC-2.04.037 | Same-range conflicting overlap returns ConflictingOverlap; original data wins |
| BC-2.04.018 | ConflictingOverlap emits Anomaly/Likely/High finding with T1036 (Masquerading) |
| BC-2.04.019 | Excessive overlap threshold emits one-shot T1036 alert finding |

### 6.5 KD-005: MITRE ATT&CK Tactic-Grouped Output

| BC ID | Contribution |
|-------|-------------|
| BC-2.10.003 | all_tactics_in_report_order returns kill-chain order for deterministic grouping |
| BC-2.10.005 | technique_name lookup for all 15 seeded IDs |
| BC-2.11.013 | TerminalReporter MITRE grouping with tactic headers in canonical order |
| BC-2.11.015 | Uncategorized bucket for no-MITRE and unknown-MITRE findings |
| BC-2.11.016 | Per-finding MITRE expansion with em-dash and name |

### 6.6 KD-006: SNI Anomaly Detection with 4-Way Classification

| BC ID | Contribution |
|-------|-------------|
| BC-2.07.013 | Clean ASCII SNI: silent, no finding |
| BC-2.07.014 | AsciiWithControl SNI: C0/DEL bytes detected, T1027 finding |
| BC-2.07.017 | NonAsciiUtf8 SNI: non-ASCII chars detected, T1027 finding |
| BC-2.07.019 | NonUtf8 SNI: invalid UTF-8 bytes detected, T1027 finding |
| BC-2.07.037 | Disambiguation: mixed non-ASCII+control fires arm 3 (NonAsciiUtf8) not arm 2 |

### 6.7 KD-007: Bounded-Resource Design

| BC ID | Contribution |
|-------|-------------|
| BC-2.04.024 | MAX_FINDINGS=10000 cap on reassembly engine findings |
| BC-2.04.025 | finalize bypass is the ONLY unconditional push past MAX_FINDINGS |
| BC-2.07.004 | MAX_RECORD_PAYLOAD=18432 cap on TLS record parsing |
| BC-2.07.005 | MAX_BUF=65536 per-direction buffer cap in TLS |
| BC-2.06.022 | MAX_HEADER_BUF=65536 per-direction buffer cap in HTTP |
| BC-2.04.041 | max_depth truncation prevents unbounded stream accumulation |
| BC-2.04.042 | max_receive_window rejects out-of-window segments |


## 7. Requirements Traceability Matrix

> Module column reflects subsystem IDs from ARCH-INDEX (ARCH-INDEX.md Subsystem Registry, Phase 1c). Priority is from Section 2.
> Test type is from BC source evidence (HIGH confidence = test exists; MEDIUM = code-only;
> LOW = ADR/comment-only).

| BC ID | Source (L2 CAP) | Module(s) | Priority | Test Type |
|-------|----------------|-----------|----------|-----------|
| BC-2.01.001 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.002 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.003 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.004 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.005 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.006 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.007 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.008 | CAP-01 | SS-01 (reader.rs) | P2 | inferred |
| BC-2.02.001 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.002 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.003 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.004 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.005 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.006 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.007 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.008 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.009 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.010 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.011 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.012 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.013 | CAP-02 | SS-02 (decoder.rs) | P2 | inferred |
| BC-2.02.014 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.015 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.04.001 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.002 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.003 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.004 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.005 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.006 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.007 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.008 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.009 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.010 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.011 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.012 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.013 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.014 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.015 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.016 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.017 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.018 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.019 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.020 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.021 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.022 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.023 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.024 | CAP-04 | SS-04 (reassembly/) | P0 | inferred |
| BC-2.04.025 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.026 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.027 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.028 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.029 | CAP-04 | SS-04 (reassembly/) | P2 | low |
| BC-2.04.030 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.031 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.032 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.033 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.034 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.035 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.036 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.037 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.038 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.039 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.040 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.041 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.042 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.043 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.044 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.045 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.046 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.047 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.048 | CAP-04 | SS-04 (reassembly/) | P2 | low |
| BC-2.04.049 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.050 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.051 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.052 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.053 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.054 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.05.001 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.002 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.003 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.004 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.005 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.05.006 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.05.007 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.008 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.009 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.06.001 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.002 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.003 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.004 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.005 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.006 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.007 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.008 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.009 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.010 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.011 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.012 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.013 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.014 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.015 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.016 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.017 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.018 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.019 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.020 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.021 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.022 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.023 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.024 | CAP-06 | SS-06 (analyzer/http.rs) | P2 | inferred |
| BC-2.06.025 | CAP-06 | SS-06 (analyzer/http.rs) | P2 | inferred |
| BC-2.06.026 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.07.001 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.002 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.003 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.004 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.005 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.006 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.007 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.008 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.009 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit+integration |
| BC-2.07.010 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.011 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | integration |
| BC-2.07.012 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.013 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.014 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.015 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.016 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.017 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.018 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.019 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.020 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.021 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.022 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.023 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.024 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.025 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.026 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.027 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.028 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.029 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.030 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.031 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit+integration |
| BC-2.07.032 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | integration |
| BC-2.07.033 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.034 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.035 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.036 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | inferred |
| BC-2.07.037 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.08.001 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.08.002 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.08.003 | CAP-08 | SS-08 (analyzer/dns.rs) | P1 | unit |
| BC-2.08.004 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.09.001 | CAP-09 | SS-09 (findings.rs) | P0 | unit |
| BC-2.09.002 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.003 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.004 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.005 | CAP-09 | SS-09 (findings.rs) | P0 | unit+integration |
| BC-2.09.006 | CAP-09 | SS-09 (findings.rs) | P0 | unit |
| BC-2.10.001 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.002 | CAP-10 | SS-10 (mitre.rs) | P1 | unit |
| BC-2.10.003 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.004 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.005 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.006 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.007 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.008 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.009 | CAP-10 | SS-10 (mitre.rs) | P2 | low |
| BC-2.11.001 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.002 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.003 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.004 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.005 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.006 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.007 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.008 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.009 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.010 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.011 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.012 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.013 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.014 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.015 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.016 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.017 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.018 | CAP-11 | SS-11 (reporter/) | P2 | inferred |
| BC-2.11.019 | CAP-11 | SS-11 (reporter/) | P1 | inferred |
| BC-2.12.001 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.002 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.003 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.004 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.005 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.006 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.007 | CAP-12 | SS-12 (cli.rs) | P0 | inferred |
| BC-2.12.008 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.009 | CAP-12 | SS-12 (main.rs) | P0 | inferred |
| BC-2.12.010 | CAP-12 | SS-12 (main.rs) | P2 | inferred |
| BC-2.12.011 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.012 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.013 | CAP-12 | SS-12 (main.rs) | P2 | low |
| BC-2.12.014 | CAP-12 | SS-12 (main.rs) | P1 | unit |
| BC-2.12.015 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.016 | CAP-12 | SS-12 (main.rs) | P0 | unit |
| BC-2.12.017 | CAP-12 | SS-12 (main.rs) | P0 | unit |
| BC-2.12.018 | CAP-12 | SS-12 (summary.rs) | P0 | unit |
| BC-2.12.019 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.12.020 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.12.021 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.13.001 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.002 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.003 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.004 | CAP-12 | SS-13 (cli.rs) | P2 | unit |


## 8. Domain Debt Index

> These open items from domain-debt.md are cross-referenced here for quick lookup.
> They describe CURRENT BEHAVIOR as of develop HEAD, not future requirements.

| Item | Description | Affected BCs |
|------|-------------|--------------|
| O-01 | Finding.timestamp always None; RawPacket timestamps never threaded to Finding constructors | BC-2.09.001, BC-2.09.006 |
| O-02 | Absent User-Agent (None) intentionally not detected; only Some("") fires | BC-2.06.011 |
| O-03 | Anomaly thresholds not empirically calibrated against labelled traffic | BC-2.04.019, BC-2.04.020, BC-2.04.021 |
| O-04 | 9 MITRE techniques catalogued but never emitted (T1040, T1071, T1071.001, T1071.004, T1573, T0846, T0855, T0856, T0885) | BC-2.10.005 |
| O-05 | reassembly/mod.rs still 691 LOC after partial split (#85) | BC-2.04.* (reassembly module group) |
| O-06 | Weak-cipher Finding evidence Vec has unbounded cardinality (up to ~9216 cipher names) | BC-2.07.009 |
| O-07 | rayon declared in Cargo.toml but never imported; unused transitive dependency | (none -- build/dep debt only) |
| O-08 | dns.rs module doc-comment (lines 1-7) describes DGA/entropy/NXDOMAIN/rare-TLD detection not implemented; DnsAnalyzer is statistics-only (QR-bit counters, always returns empty findings Vec) | BC-2.08.001-004 |
