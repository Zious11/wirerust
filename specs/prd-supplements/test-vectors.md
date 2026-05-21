---
document_type: prd-supplement-test-vectors
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - src/reader.rs
  - src/decoder.rs
  - src/reassembly/
  - src/analyzer/http.rs
  - src/analyzer/tls.rs
  - src/analyzer/dns.rs
  - src/findings.rs
  - src/reporter/terminal.rs
  - src/reporter/json.rs
  - src/mitre.rs
input-hash: "592d3cb"
traces_to: .factory/specs/prd.md
---

# Canonical Test Vectors: wirerust

> PRD supplement -- extracted from PRD Section 7 / BC test vector sections.
> Referenced by: test-writer, implementer, holdout-evaluator.
> Brownfield: vectors grounded in verified source behavior as of develop HEAD.

## Per-Subsystem Test Vectors

---

### SS-01: PCAP File Ingestion (CAP-01)

#### BC-2.01.001 -- Accept Supported Link Types; Reject Unsupported

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| pcap file with DataLink::ETHERNET (1) | Ok(PcapSource { datalink: ETHERNET }) | happy-path | Standard Ethernet pcap |
| pcap file with DataLink::RAW (101) | Ok(PcapSource { datalink: RAW }) | happy-path | Raw IP link |
| pcap file with DataLink::LINUX_SLL (113) | Ok(PcapSource { datalink: LINUX_SLL }) | happy-path | Linux cooked capture |
| pcap file with DataLink::IPV4 (228) | Ok(PcapSource { datalink: IPV4 }) | happy-path | Layer-3 IPv4 |
| pcap file with DataLink::IPV6 (229) | Ok(PcapSource { datalink: IPV6 }) | happy-path | Layer-3 IPv6 |
| pcap file with DataLink::UNKNOWN(166) | Err("Unsupported pcap link type: UNKNOWN(166). Supported: ...") | error | E-INP-001 |
| pcap file with DataLink::PPP or any other | Err("Unsupported pcap link type: ...") | error | All non-{1,101,113,228,229} types rejected |

#### BC-2.01.002 -- Read All Packets Preserving Timestamps

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| pcap with 3 packets, timestamps 1_000_000 / 1_000_001 / 1_000_002 sec | Vec with 3 RawPacket entries; each has timestamp_secs matching | happy-path | Timestamp preserved per packet |
| pcap nanosecond resolution (TsResolution::NanoSecond) | timestamp_usecs = ts_frac / 1_000 (microseconds) | happy-path | Nanosecond-to-microsecond conversion |

#### BC-2.01.003 -- Zero-Packet PCAP

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| pcap with valid header but zero data records | Ok(PcapSource { packets: [] }) | happy-path | Header-only file accepted without error |

#### BC-2.01.004 -- Reject pcapng

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| file with pcapng magic bytes (0x0A0D0D0A) | Err("Failed to parse pcap header: ...") | error | E-INP-002; pcapng magic fails PcapReader::new |

#### BC-2.01.006 / 007 -- Error Surface

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| truncated pcap (header missing last 4 bytes) | Err("Failed to parse pcap header: ...") | error | E-INP-002 |
| pcap where 2nd packet record is truncated | Err("Failed to read packet: ...") | error | E-INP-003 |

---

### SS-02: Packet Decoding (CAP-02 + CAP-03)

#### BC-2.02.001 -- Ethernet IPv4 TCP Decode

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| 14-byte Ethernet header + IPv4 + TCP payload `GET / HTTP/1.1\r\n` | ParsedPacket { src_ip: IPv4, dst_ip: IPv4, protocol: Tcp, transport: Tcp { syn: false, seq_number: N, ... }, payload: b"GET / HTTP/1.1\r\n..." } | happy-path | Standard Ethernet frame |

#### BC-2.02.005 -- IPv6 TCP Decode

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| RAW link + IPv6 + TCP | ParsedPacket { src_ip: IPv6, dst_ip: IPv6, protocol: Tcp } | happy-path | IPv6 addresses extracted |

#### BC-2.02.007 -- Reject Malformed (No Panic)

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| 4 random bytes [0xDE, 0xAD, 0xBE, 0xEF] with DataLink::ETHERNET | Err("...") -- no panic | error | Structural corruption rejected via strict parser |
| Snaplen-truncated frame (incl_len < ip.total_length, but IP header intact) | Ok(ParsedPacket with protocol: Tcp, payload: Vec<u8>) | happy-path (degraded) | Lax fallback triggered by SliceError::Len only |
| Frame with bad IHL (IP version nibble = 0) | Err("...") -- NOT lax-recovered | error | Non-Len SliceError; strict rejection only |

#### BC-2.02.015 -- TCP Flags

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| TCP SYN packet | TransportInfo::Tcp { syn: true, ack: false, fin: false, rst: false } | happy-path | |
| TCP SYN+ACK | TransportInfo::Tcp { syn: true, ack: true } | happy-path | |
| TCP FIN+ACK | TransportInfo::Tcp { fin: true, ack: true } | happy-path | |
| TCP RST | TransportInfo::Tcp { rst: true } | happy-path | |

---

### SS-04: TCP Stream Reassembly (CAP-04)

#### BC-2.04.003 -- Canonical FlowKey Ordering

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Packet: src=10.0.0.1:1234, dst=10.0.0.2:80 | FlowKey { lower_ip: 10.0.0.1, lower_port: 1234, upper_ip: 10.0.0.2, upper_port: 80 } | happy-path | lower = lexicographically smaller IP |
| Reverse: src=10.0.0.2:80, dst=10.0.0.1:1234 | Same FlowKey as above | happy-path | Bidirectional flows produce identical key |
| src_ip == dst_ip, src_port=80, dst_port=1234 | FlowKey with lower_port=80 | edge-case | Same IP, port ordering decides |

#### BC-2.04.018 -- Conflicting Overlap Finding

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Segment at offset 100, data `[0xAA, 0xBB]`; then segment at offset 100, data `[0xCC, 0xDD]` (conflicting overlap) | Finding { category: Anomaly, verdict: Likely, confidence: High, mitre_technique: Some("T1036") } | happy-path | First-wins overlap with conflicting bytes |
| Segment at offset 100, data `[0xAA]`; then segment at offset 100, data `[0xAA]` (exact retransmission) | InsertResult::Duplicate; zero findings for this insert | happy-path | Identical retransmission = no conflict finding |

#### BC-2.04.024 -- MAX_FINDINGS Cap

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| 10,001 distinct conflicting overlap events | First 10,000 produce findings; finding at index 10,001 silently dropped; findings.len() == 10,000 | edge-case | finalize() summary finding is NOT subject to cap (BC-2.04.054) |

#### BC-2.04.054 -- finalize Bypasses Cap

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| 10,000 findings already; segments_dropped > 0 at finalize() | findings.len() == 10,001; last finding is segment-limit summary | edge-case | Finalize unconditionally bypasses MAX_FINDINGS |

---

### SS-05: Content-First Protocol Dispatch (CAP-05)

#### BC-2.05.001 -- TLS Content Signature

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| TCP flow; first payload bytes: `[0x16, 0x03, 0x01, 0x00, 0x5A, ...]` (TLS record); dst_port=9999 | DispatchTarget::Tls (content-first, regardless of port 9999) | happy-path | ADR 0001: content beats port |
| TCP flow; first payload bytes: `[0x16, 0x03, 0x01, ...]`; dst_port=443 | DispatchTarget::Tls | happy-path | Redundant confirmation: content match + port match |

#### BC-2.05.002 / 003 -- HTTP Detection and Port Fallback

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| TCP flow; payload starts with `GET ` (4 bytes) | DispatchTarget::Http | happy-path | HTTP method prefix match |
| TCP flow; payload starts with `POST` | DispatchTarget::Http | happy-path | |
| TCP flow; payload = `[0x00, 0x01, 0x02]` (unknown); dst_port=443 | DispatchTarget::Tls (port fallback) | happy-path | Content insufficient; port 443 -> TLS fallback |
| TCP flow; payload = `[0x00, 0x01, 0x02]`; dst_port=80 | DispatchTarget::Http (port fallback) | happy-path | Port 80 -> HTTP fallback |
| TCP flow; payload = `[0x00, 0x01, 0x02]`; dst_port=9999 | DispatchTarget::None | happy-path | Unknown content + unknown port |

#### BC-2.05.005 -- Classification Cached

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Flow classified as Tls on first on_data; second on_data arrives | Second delivery routes to TLS without re-inspecting content | happy-path | Cache hit on subsequent data |
| DispatchTarget::None on first delivery; second on_data arrives | Reclassification attempted on second delivery (None not cached) | edge-case | BC-2.05.006 |

---

### SS-06: HTTP Traffic Analysis (CAP-06)

#### BC-2.06.005 -- Path Traversal Finding

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `GET /../etc/passwd HTTP/1.1\r\nHost: example.com\r\n\r\n` | Finding { category: Reconnaissance, verdict: Likely, confidence: High, mitre_technique: Some("T1083"), summary: contains "../" } | happy-path | Classic directory traversal; http.rs:193 uses ThreatCategory::Reconnaissance |
| `GET /static/file.css HTTP/1.1\r\nHost: example.com\r\n\r\n` | No finding | happy-path | Clean path; no traversal |

#### BC-2.06.009 -- Missing Host Header

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `GET / HTTP/1.1\r\n\r\n` (no Host header) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Medium } | happy-path | RFC 7230 requires Host for HTTP/1.1 |
| `GET / HTTP/1.0\r\n\r\n` | No finding | happy-path | HTTP/1.0 does not require Host |

#### BC-2.06.011 -- Empty User-Agent

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `GET / HTTP/1.1\r\nHost: x.com\r\nUser-Agent: \r\n\r\n` (present but blank) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low } | happy-path | Empty UA = finding |
| `GET / HTTP/1.1\r\nHost: x.com\r\n\r\n` (no User-Agent header at all) | No finding | happy-path | Absent UA = no finding (BC-2.06.011 explicitly: "absent UA does NOT") |

#### BC-2.06.014 -- Too Many Headers

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| HTTP request with 97 headers (MAX_HEADERS=96 exceeded) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Medium, mitre_technique: Some("T1499.002") } | edge-case | httparse array size = 96; overflow triggers finding |
| HTTP request with exactly 96 headers | No finding from header-count check | happy-path | At-cap, not over-cap |

#### BC-2.06.015 -- Poison After 3 Consecutive Errors

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| 3 consecutive non-HTTP byte chunks delivered to request direction | Direction poisoned; `non_http_flows` incremented; subsequent bytes skipped | edge-case | POISON_THRESHOLD=3 |
| 2 consecutive parse errors, then valid HTTP request | Not poisoned; valid request parsed normally | edge-case | BC-2.06.016 |
| Request direction poisoned; valid HTTP response arrives on response direction | Response parsed normally | edge-case | BC-2.06.017: per-direction isolation |

---

### SS-07: TLS Traffic Analysis (CAP-07)

#### BC-2.07.014 -- SNI C0/DEL Anomaly Finding

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| TLS ClientHello with SNI = `evil\x1bhost.com` (ESC byte 0x1B, a C0 control) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_technique: Some("T1027") } | happy-path | AsciiWithControl arm (tls.rs:426-448); verdict/confidence are Inconclusive/Low, not Likely/High |
| TLS ClientHello with SNI = `evil\x7fhost.com` (DEL 0x7F) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_technique: Some("T1027") } | happy-path | DEL = AsciiWithControl arm |
| TLS ClientHello with SNI = `www.example.com` (clean ASCII) | No SNI-related finding | happy-path | BC-2.07.013 |
| TLS ClientHello with SNI = `www.ex\x20ample.com` (SPACE = 0x20) | No SNI C0 finding (0x20 is not C0/DEL) | edge-case | BC-2.07.016: 0x1F trips; 0x20 does NOT |

#### BC-2.07.017 -- Non-ASCII UTF-8 SNI

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| TLS ClientHello with SNI = `xn--\xC3\xA9vil.com` (contains U+00E9, valid UTF-8 but non-ASCII) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_technique: Some("T1027") } | happy-path | NonAsciiUtf8 arm (tls.rs:449-468); verdict/confidence are Inconclusive/Low, not Likely/High |
| TLS ClientHello with SNI = `xn--test.com` (pure ASCII Punycode A-label) | No finding | happy-path | BC-2.07.018: A-labels are pure ASCII |

#### BC-2.07.037 -- Mixed Non-ASCII + C0 SNI Fires Arm 3

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| SNI bytes contain BOTH U+00E9 (non-ASCII UTF-8) AND 0x1B (C0 control) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_technique: Some("T1027") } with arm = NonAsciiUtf8 (arm 3), not C0/DEL (arm 2) | edge-case | Arm 3 (NonAsciiUtf8) takes priority because `from_utf8` succeeds on the multi-byte U+00E9 sequence before `is_ascii()` is checked; tls.rs:251-258 `extract_sni` match order |

#### BC-2.07.009 -- Weak Cipher Finding

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| ClientHello with cipher suite `TLS_NULL_WITH_NULL_NULL (0x0000)` | Finding { category: Anomaly, verdict: Likely, confidence: High } | happy-path | NULL cipher is weak |
| ClientHello with cipher suite `TLS_AES_256_GCM_SHA384 (0x1302)` | No weak-cipher finding | happy-path | Strong cipher |

#### BC-2.07.007 -- JA3 String Format

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| ClientHello: version=0x0303, ciphers=[0x1301, 0x002F], no extensions (GREASE filtered) | JA3 string = "771,4865-47," with the remainder of the format (curves, pointfmts empty); MD5 of the string | happy-path | 0x0303=771 decimal; GREASE values filtered per RFC 8701 |
| ClientHello with GREASE cipher 0x0A0A in the cipher list | GREASE cipher excluded from JA3 computation | edge-case | BC-2.07.006 |

---

### SS-08: DNS Traffic Analysis (CAP-08)

#### BC-2.08.001 -- Port 53 Matching

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| UDP packet with dst_port=53, DNS query payload | DnsAnalyzer.can_decode() = true; analyze() increments query_count | happy-path | |
| TCP packet with dst_port=53 | DnsAnalyzer.can_decode() = true | happy-path | DNS over TCP supported |
| UDP packet with dst_port=80 | DnsAnalyzer.can_decode() = false | happy-path | Not DNS port |

#### BC-2.08.004 -- No Findings Ever

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Any DNS packet (query or response, well-formed or malformed) | analyze() returns Vec::new() | happy-path | DNS is statistics-only; NEVER emits findings |

---

### SS-09: Forensic Finding Emission (CAP-09)

#### BC-2.09.006 -- JSON Serialization of Option Fields

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Finding with mitre_technique: None | JSON object does NOT contain key "mitre_technique" | happy-path | skip_serializing_if |
| Finding with mitre_technique: Some("T1027") | JSON object contains `"mitre_technique": "T1027"` | happy-path | |
| Finding with source_ip: None | JSON object does NOT contain key "source_ip" | happy-path | skip_serializing_if |
| Finding with timestamp: None | JSON object does NOT contain key "timestamp" | happy-path | Always None per O-01; must be absent not null |
| Finding with direction: None | JSON object does NOT contain key "direction" | happy-path | |

---

### SS-10: MITRE ATT&CK Mapping (CAP-10)

#### BC-2.10.005 -- technique_name for All 15 Seeded IDs

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `technique_name("T1036")` | Some("Masquerading") | happy-path | Overlap evasion |
| `technique_name("T1027")` | Some("Obfuscated Files or Information") | happy-path | SNI anomaly |
| `technique_name("T1083")` | Some("File and Directory Discovery") | happy-path | Path traversal |
| `technique_name("T1499.002")` | Some("Service Exhaustion Flood") | happy-path | Too-many-headers |
| `technique_name("T1505.003")` | Some("Web Shell") | happy-path | Web-shell URI |
| `technique_name("T1046")` | Some("Network Service Discovery") | happy-path | Admin panel |
| `technique_name("UNKNOWN999")` | None | happy-path | BC-2.10.006 |

#### BC-2.10.003/004 -- all_tactics_in_report_order

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `all_tactics_in_report_order()` | Vec with exactly 16 MitreTactic variants; Reconnaissance first; ICS tactics last; no duplicates | happy-path | Kill-chain order: Recon, ResourceDev, InitAccess, Exec, Persist, PrivEsc, DefEvasion, CredAccess, Discovery, LateralMov, Collection, C2, Exfil, Impact, then ICS |

---

### SS-11: Reporting and Output (CAP-11)

#### BC-2.11.003 -- JSON C0 Escaping

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Finding with summary = `"path\x00injection"` (null byte 0x00) | JSON: `"summary": "path\u0000injection"` | happy-path | serde_json RFC 8259: C0 bytes escaped as \uXXXX; null byte -> \u0000 |
| Finding with summary = `"evil\x1bpath"` (ESC 0x1B) | JSON: `"summary": "evil\u001bpath"` | happy-path | serde_json escapes ESC as \u001b |
| Finding with summary = `"Кириллица"` (Cyrillic, valid UTF-8) | JSON: `"summary": "Кириллица"` (readable, not \uXXXX-escaped) | happy-path | BC-2.11.004: non-ASCII Unicode preserved readable |

#### BC-2.11.007 -- Terminal C0/C1/DEL/Backslash Escaping

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Finding.summary = `"path\x1b[31mevil\x1b[0m"` (ANSI CSI sequence) | Rendered: `"path\e[31mevil\e[0m"` (ESC escaped as `\e`, brackets preserved) | happy-path | BC-RPT-007: terminal injection prevented |
| Finding.summary = `"test\x7fpath"` (DEL 0x7F) | Rendered with DEL escaped as `\x7f` or similar escape form | happy-path | DEL is C0-equivalent in terminal escape rules |
| Finding.summary = `"csi\xc2\x9bpayload"` (U+009B = C1 CSI, 2-byte UTF-8) | Rendered: CSI escaped (e.g. `\u{9b}`) | happy-path | BC-2.11.009: C1 range U+0080..=U+009F escaped |
| Finding.summary = `"Кириллица"` | Rendered verbatim: `"Кириллица"` | happy-path | BC-2.11.008: non-ASCII Unicode preserved |
| Finding.summary = `"path\\back"` (literal backslash) | Rendered: `"path\\\\back"` or equivalent escaped form | happy-path | Backslash escaped per escape_for_terminal |
| Finding.summary contains U+00A0 (non-breaking space) | Preserved verbatim (U+00A0 is not C1) | edge-case | BC-2.11.009: U+00A0 is outside C1 range |

#### BC-2.11.013 -- MITRE Tactic Grouping

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| 2 findings with T1036 (DefEvasion), 1 with T1083 (Discovery), 1 with no technique | Terminal output contains `## Defense Evasion` header with 2 findings; `## Discovery` with 1; `Uncategorized` with 1 (last) | happy-path | `--mitre` flag required |
| All findings have no mitre_technique | Single `Uncategorized` section; no tactic headers | edge-case | BC-2.11.015 |

---

### SS-12: CLI and Entry Point (Cross-Cutting)

#### BC-2.12.007 -- Mutually Exclusive Flags

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `wirerust analyze --reassemble --no-reassemble sample.pcap` | Exit code 2; clap error: "the argument '--reassemble' cannot be used with '--no-reassemble'" | error | E-CFG-001 |
| `wirerust analyze --json --csv sample.pcap` | Exit code 2; clap error about mutually exclusive flags | error | E-CFG-002 |

#### BC-2.12.011 -- Directory Expansion

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Target is directory containing `a.pcap`, `b.pcap`, `c.pcapng`, `d.txt` | Expands to `[a.pcap, b.pcap]` sorted; `c.pcapng` and `d.txt` excluded | happy-path | Only `.pcap` extension included; sorted lexicographically |

#### BC-2.12.012 -- Non-Existent Target

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `wirerust analyze /does/not/exist.pcap` | Exit code 1; error: "Target not found: /does/not/exist.pcap" | error | E-INP-006; anyhow::bail! |

---

## Cross-Subsystem Integration Vectors

### Integration 1: HTTP Path Traversal End-to-End (SS-04 -> SS-05 -> SS-06 -> SS-09 -> SS-11)

| Scenario | Input | Step 1: Reassembly | Step 2: Dispatch | Step 3: HTTP Analysis | Final Output |
|----------|-------|-------------------|-----------------|----------------------|-------------|
| Attacker sends `GET /../etc/passwd HTTP/1.1\r\nHost: evil.com\r\n\r\n` over TCP flow | pcap with SYN, data, FIN sequence | TcpReassembler delivers data bytes to StreamDispatcher | Dispatcher classifies as HTTP (method prefix `GET `) | HttpAnalyzer detects traversal; emits Finding(Reconnaissance/Likely/High, T1083) | JSON: `{"category":"Reconnaissance","verdict":"Likely","confidence":"High","mitre_technique":"T1083"...}` |

### Integration 2: TLS SNI C0 Injection (SS-04 -> SS-05 -> SS-07 -> SS-11)

| Scenario | Input | Step 1 | Step 2 | Step 3 | Final Output |
|----------|-------|--------|--------|--------|-------------|
| TLS ClientHello with SNI = `evil\x00host.com` | pcap with TLS handshake | Reassembly delivers TLS record bytes | Dispatcher: first byte 0x16 -> TLS (content-first) | TlsAnalyzer: SNI AsciiWithControl arm; emits Finding(Anomaly/Inconclusive/Low, T1027) | JSON: `"mitre_technique":"T1027"`, summary contains raw SNI bytes |

### Integration 3: Terminal Injection Prevention (SS-07 -> SS-11 terminal path)

| Scenario | Input | Analyzer Output | Terminal Reporter Output |
|----------|-------|----------------|--------------------------|
| TLS SNI = `\x1b[31m` (ANSI red sequence) | pcap with crafted TLS | Finding.summary = `"SNI: \x1b[31m"` (raw bytes per ADR 0003) | Rendered: `"SNI: \e[31m"` (ESC escaped; terminal cannot interpret as color code) |
| Same SNI via JSON reporter | Same Finding | `"summary": "SNI: \u001b[31m"` (serde_json RFC 8259 escaping: ESC -> \u001b) | JSON consumer gets escaped C0; C0 never reaches terminal directly |

### Integration 4: MAX_FINDINGS Cap (SS-04 -> SS-09)

| Scenario | Input | Reassembly Behavior | Final Output |
|----------|-------|---------------------|-------------|
| pcap constructed to generate 10,001 conflicting overlaps | adversarial pcap with many conflicting retransmissions | First 10,000 go into findings Vec; 10,001st is silently dropped; finalize() summary finding is appended unconditionally (findings.len() = 10,001) | JSON findings array has 10,001 entries; last entry is segment-limit summary finding |

---

## Golden File References

| Vector Set | File | Format | BC Coverage | Description |
|-----------|------|--------|------------|-------------|
| HTTP analysis regression | `tests/fixtures/http-full.cap` | classic pcap | BC-2.06.001..026 | Full HTTP/1.1 session with mixed request types; used to calibrate POISON_THRESHOLD=3 |
| TLS handshake (SNI/JA3) | `tests/fixtures/` (tls fixture files) | classic pcap | BC-2.07.001..037 | Standard TLS 1.2/1.3 handshakes |
| TCP reassembly | `tests/fixtures/` (reassembly fixture files) | classic pcap | BC-2.04.003..054 | Overlapping segments, OOO delivery |
| DNS traffic | `tests/fixtures/` (dns fixture files) | classic pcap | BC-2.08.001..004 | Query/response counting |

> Note: exact fixture file paths for all sets are confirmed present under `tests/fixtures/`
> via the brownfield ingestion. The `http-full.cap` fixture is explicitly documented as the
> empirical source for POISON_THRESHOLD=3 calibration (see NFR-RES-017 and semport Pass 4 R2).


## Real-World Corpus Scenarios

### Corpus 1: Known-Good (False Positive Rate Test)

| Corpus | Source | Expected Result | Rationale |
|--------|--------|----------------|-----------|
| Small Business HTTP traffic (benign) | Publicly available PCAP from Wireshark wiki: `http.cap` (7 KB, HTTP GET, clean traffic) | Zero or minimal findings; no path traversal, no SNI anomalies, no TCP overlap findings | A well-maintained production HTTP session should produce no threat-level findings |
| DNS resolution trace | Wireshark wiki `dns.cap` | `dns_queries` > 0; `dns_responses` > 0; zero findings (DNS is statistics-only) | Validates BC-2.08.004 at corpus scale |

### Corpus 2: Known-Problematic (False Negative Rate Test)

| Corpus | Source | Expected Result | Rationale |
|--------|--------|----------------|-----------|
| Malware traffic sample with path traversal | PCAP corpus from malware-traffic-analysis.net or equivalent CTF pcap with known web shell / path traversal | At least one Anomaly/Likely/High finding with T1083 or T1505.003 | Validates detection rules fire on real attacker traffic |
| TLS with SSLv3 fallback (POODLE-era capture) | Legacy TLS capture with SSLv3 negotiation | Finding with deprecated-protocol detection (BC-2.07.011/012) | Validates TLS version anomaly detection |
