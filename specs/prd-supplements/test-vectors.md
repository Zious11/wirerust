---
document_type: prd-supplement-test-vectors
level: L3
version: "2.3"
status: draft
producer: product-owner
timestamp: 2026-06-12T02:00:00Z
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
input-hash: N/A
# input-hash rationale: src/analyzer/arp.rs was removed from inputs (forward-referenced;
# file does not exist in develop HEAD until STORY-111 lands). The hash computation tool
# (bin/compute-input-hash) errors on missing inputs, so this supplement's hash is deferred
# until STORY-111 merges and arp.rs is present. Re-add src/analyzer/arp.rs to inputs: and
# run `bin/compute-input-hash --write` after STORY-111 lands.
traces_to: .factory/specs/prd.md
modified:
  - "v1.6: Pass-7 remediation F-C-P7-002: separated BC-2.10.005/BC-2.10.004 attributions — 'per BC-2.10.005 v1.10 (25 seeded IDs) and BC-2.10.004 v1.4 (17 tactic variants)'; was incorrectly citing BC-2.10.004 as the seeded-count source. — 2026-06-12"
  - "v1.7: Pass-9 remediation F-C-P9-002: BC-2.10.004 version citation updated v1.4→v1.5 (file is at v1.5 per pass-7 F-C-P7-003 remediation). F-C-P9-004: SS-10 unknown-ID canary updated UNKNOWN999→T9999 and category happy-path→edge-case to match BC-2.10.005/BC-2.10.006 canonical canary and mitre.rs Kani verify_unknown_id_returns_none_no_panic. — 2026-06-12"
  - "v2.0: ARP-F2 Pass-14 remediation C-05 + 12 stale snippets: (C-05) removed src/analyzer/arp.rs from inputs (not in develop HEAD; forward-reference to STORY-111); set input-hash to N/A with deferred-hash rationale comment. (12 stale snippets) converted all mitre_technique:Some('X') / 'mitre_technique':'X' occurrences to mitre_techniques:vec!['X'] / 'mitre_techniques':['X'] form per STALE discrimination rule; updated skip-serialization/empty-semantics prose (lines ~279/280/342) to Vec/empty-vec/key-absent form. Version 1.9→2.0. — 2026-06-13"
  - "v2.1: P19 straggler anchor sweep — BC-2.06.005 Notes http.rs:193→:205 (path-traversal push); BC-2.07.014 Notes tls.rs:426-448→:437-460 (AsciiWithControl block); BC-2.07.017 Notes tls.rs:449-468→:461-493 (NonAsciiUtf8 block); BC-2.07.037 Notes tls.rs:251-258→:252-266 (extract_sni match block). Verified against src/analyzer/http.rs and src/analyzer/tls.rs. — 2026-06-13"
  - "v2.3: F5 ICS tactic-ID correctness fix (D-209, 2026-06-23, DF-SIBLING-SWEEP-001): BC-2.10.003/004 test vector updated — 17→20 MitreTactic variants; ICS order appended with IcsDiscovery [17], IcsCollection [18], IcsCommandAndControl [19]; Notes text updated."
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

#### ~~BC-2.01.004 -- Reject pcapng~~ [RETIRED — replaced by BC-2.01.009]

> **RETIRED 2026-06-19 (F2 pcapng-reader-support).** BC-2.01.004 is superseded by BC-2.01.009.
> The test vector below is INVERTED — pcapng now returns Ok, not Err. See BC-2.01.009 vectors below.

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| ~~file with pcapng magic bytes (0x0A0D0D0A)~~ | ~~Err("Failed to parse pcap header: ...")~~ | ~~error~~ | ~~E-INP-002; pcapng magic fails PcapReader::new~~ — **STALE: inverted by F2; pcapng now Ok via BC-2.01.009** |

#### BC-2.01.009 -- Accept pcapng via Magic-Byte Probe (F2)

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `tests/fixtures/smb3.pcapng` (former negative fixture) | `Ok(PcapSource)` with `packets.len() > 0` | happy-path | Behavioral inversion of BC-2.01.004; probe detects SHB magic 0x0A0D0D0A |
| Classic `tests/fixtures/*.pcap` files | `Ok(PcapSource)` via classic-pcap path unchanged | regression | Probe routes to classic path on non-pcapng magic |
| Stream of 2 bytes only | `Err` | error | Too short for probe |
| 4 bytes `[0xDE, 0xAD, 0xBE, 0xEF]` | `Err` containing "unrecognized pcap magic" or equivalent | error | E-INP-002 / unrecognized magic |

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
| Segment at offset 100, data `[0xAA, 0xBB]`; then segment at offset 100, data `[0xCC, 0xDD]` (conflicting overlap) | Finding { category: Anomaly, verdict: Likely, confidence: High, mitre_techniques: vec!["T1036"] } | happy-path | First-wins overlap with conflicting bytes |
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
| `GET /../etc/passwd HTTP/1.1\r\nHost: example.com\r\n\r\n` | Finding { category: Reconnaissance, verdict: Likely, confidence: High, mitre_techniques: vec!["T1083"], summary: contains "../" } | happy-path | Classic directory traversal; http.rs:205 uses ThreatCategory::Reconnaissance |
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
| HTTP request with 97 headers (MAX_HEADERS=96 exceeded) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Medium, mitre_techniques: vec!["T1499.002"] } | edge-case | httparse array size = 96; overflow triggers finding |
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
| TLS ClientHello with SNI = `evil\x1bhost.com` (ESC byte 0x1B, a C0 control) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_techniques: vec!["T1027"] } | happy-path | AsciiWithControl arm (tls.rs:437-460); verdict/confidence are Inconclusive/Low, not Likely/High |
| TLS ClientHello with SNI = `evil\x7fhost.com` (DEL 0x7F) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_techniques: vec!["T1027"] } | happy-path | DEL = AsciiWithControl arm |
| TLS ClientHello with SNI = `www.example.com` (clean ASCII) | No SNI-related finding | happy-path | BC-2.07.013 |
| TLS ClientHello with SNI = `www.ex\x20ample.com` (SPACE = 0x20) | No SNI C0 finding (0x20 is not C0/DEL) | edge-case | BC-2.07.016: 0x1F trips; 0x20 does NOT |

#### BC-2.07.017 -- Non-ASCII UTF-8 SNI

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| TLS ClientHello with SNI = `xn--\xC3\xA9vil.com` (contains U+00E9, valid UTF-8 but non-ASCII) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_techniques: vec!["T1027"] } | happy-path | NonAsciiUtf8 arm (tls.rs:461-493); verdict/confidence are Inconclusive/Low, not Likely/High |
| TLS ClientHello with SNI = `xn--test.com` (pure ASCII Punycode A-label) | No finding | happy-path | BC-2.07.018: A-labels are pure ASCII |

#### BC-2.07.037 -- Mixed Non-ASCII + C0 SNI Fires Arm 3

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| SNI bytes contain BOTH U+00E9 (non-ASCII UTF-8) AND 0x1B (C0 control) | Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, mitre_techniques: vec!["T1027"] } with arm = NonAsciiUtf8 (arm 3), not C0/DEL (arm 2) | edge-case | Arm 3 (NonAsciiUtf8) takes priority because `from_utf8` succeeds on the multi-byte U+00E9 sequence before `is_ascii()` is checked; tls.rs:252-266 `extract_sni` match order |

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

#### BC-2.09.006 -- JSON Serialization of Vec and Option Fields

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| Finding with mitre_techniques: vec![] (empty) | JSON object does NOT contain key "mitre_techniques" | happy-path | skip_serializing_if = Vec::is_empty; absent not null/[] |
| Finding with mitre_techniques: vec!["T1027"] | JSON object contains `"mitre_techniques": ["T1027"]` | happy-path | singleton vec → JSON array |
| Finding with mitre_techniques: vec!["T0830","T1557.002"] | JSON object contains `"mitre_techniques": ["T0830","T1557.002"]` | happy-path | multi-technique co-attribution (ADR-006 Decision 13) |
| Finding with source_ip: None | JSON object does NOT contain key "source_ip" | happy-path | skip_serializing_if = Option::is_none |
| Finding with timestamp: None | JSON object does NOT contain key "timestamp" | happy-path | absent not null; O-01 closed but None still valid for sites without timestamp |
| Finding with direction: None | JSON object does NOT contain key "direction" | happy-path | skip_serializing_if = Option::is_none |

---

### SS-10: MITRE ATT&CK Mapping (CAP-10)

#### BC-2.10.005 -- technique_name for All 25 Seeded IDs

> Representative subset (8 of 25 total seeded IDs shown; full 25-ID catalog in BC-2.10.005
> canonical test vectors). Added in F2 ARP feature: T0830 (ICS LateralMovement) and T1557.002
> (Enterprise CredentialAccess). Full seeded count is 25 (12 Enterprise + 13 ICS) per
> BC-2.10.005 v1.10 (25 seeded IDs) and BC-2.10.004 v1.6 (20 tactic variants). PLANNED — implemented in STORY-114.

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `technique_name("T1036")` | Some("Masquerading") | happy-path | Overlap evasion |
| `technique_name("T1027")` | Some("Obfuscated Files or Information") | happy-path | SNI anomaly |
| `technique_name("T1083")` | Some("File and Directory Discovery") | happy-path | Path traversal |
| `technique_name("T1499.002")` | Some("Service Exhaustion Flood") | happy-path | Too-many-headers |
| `technique_name("T1505.003")` | Some("Web Shell") | happy-path | Web-shell URI |
| `technique_name("T1046")` | Some("Network Service Discovery") | happy-path | Admin panel |
| `technique_name("T0830")` | Some("Adversary-in-the-Middle") | happy-path | ARP AiTM, ICS (new F2 ARP) |
| `technique_name("T1557.002")` | Some("Adversary-in-the-Middle: ARP Cache Poisoning") | happy-path | ARP Cache Poisoning, Enterprise (new F2 ARP) |
| `technique_name("T9999")` | None | edge-case | BC-2.10.006; canonical canary aligns to mitre.rs Kani verify_unknown_id_returns_none_no_panic |

#### BC-2.10.003/004 -- all_tactics_in_report_order

| Input | Expected Output | Category | Notes |
|-------|----------------|----------|-------|
| `all_tactics_in_report_order()` | Vec with exactly 20 MitreTactic variants; Reconnaissance first; ICS tactics last; no duplicates | happy-path | Kill-chain order: Recon[0], ResourceDev[1], InitAccess[2], Exec[3], Persist[4], PrivEsc[5], DefEvasion[6], CredAccess[7], Discovery[8], LateralMov[9], Collection[10], C2[11], Exfil[12], Impact[13] [14 Enterprise], then IcsInhibitResponseFunction[14], IcsImpairProcessControl[15], IcsImpact[16], IcsDiscovery[17], IcsCollection[18], IcsCommandAndControl[19] [6 ICS — IcsImpact added F2 DNP3; IcsDiscovery/IcsCollection/IcsCommandAndControl added F5 D-209] |

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
| All findings have mitre_techniques: vec![] (empty) | Single `Uncategorized` section; no tactic headers | edge-case | BC-2.11.015 |

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

### SS-16: ARP Security Analysis (CAP-16) [Feature #9 — v0.7.0]

> Edge cases and boundary conditions for the ARP security analyzer.
> Sources: BC-2.16.003 through BC-2.16.009 Edge Case tables (greenfield F2 contracts).
> These supplement the canonical test vectors embedded in each BC file.

#### ARP Storm Rate — Same-Second Denominator (BC-2.16.008 EC-002/EC-008)

| Source MAC | Frame count | Timing | storm_rate | Expected outcome |
|---|---|---|---|---|
| AA:BB:CC:DD:EE:FF | 50 frames, all `ts = 100` (same second) | 60s window (same integer second; ts==window_start_ts) | 50 | Storm finding emitted; rate = 50/1 = 50 = threshold; count/1 avoids divide-by-zero |
| AA:BB:CC:DD:EE:FF | 49 frames, all `ts = 100` | 60s window (same integer second; ts==window_start_ts) | 50 | No storm finding (49 < 50) |
| AA:BB:CC:DD:EE:FF | 51st frame, same source MAC, `storm_emitted=true` | 60s window (same integer second; ts==window_start_ts) | 50 | No additional finding; one-shot guard active |
| BB:CC:DD:EE:FF:00 | 50 frames, `ts = 100` | 60s window (same integer second; ts==window_start_ts) | 50 | Independent storm finding for distinct source MAC; per-MAC state |

**Rate formula (ARP-AMB-003 RESOLVED in F2):** `rate = count_in_window / max(1, ts - window_start_ts)`. Timestamps are integer seconds (u32); there is no sub-second ambiguity. When `ts == window_start_ts` (all frames in the same integer second), `max(1, 0) = 1` and rate = count_in_window. When frames span N elapsed seconds, rate = count / N. The table above is arithmetically consistent with this formula. See BC-2.16.008 for complete test vectors including the 2-second burst case.

#### GARP-That-Conflicts Upgrade Escalation (BC-2.16.014)

| Binding table state | Frame | Expected findings |
|---|---|---|
| Empty | op=2, sender_ip=10.0.0.1==target_ip, sender_mac=AA:AA | GARP finding LOW (no conflict; BC-2.16.003) |
| `{10.0.0.1 → BB:BB, rebind_count=0}` | op=2, sender_ip=10.0.0.1==target_ip, sender_mac=AA:AA | GARP finding MEDIUM + D1 spoof finding MEDIUM; T0830+T1557.002 on both |
| `{10.0.0.1 → BB:BB, rebind_count=2, first_rebind_ts=5}` | op=2, sender_ip=10.0.0.1==target_ip, sender_mac=AA:AA, ts=30 (within 60s) | GARP finding MEDIUM + D1 spoof finding HIGH (rebind_count→3 ≥ threshold=3) |
| `{10.0.0.1 → BB:BB}`, `spoof_high_emitted=true` | op=2 GARP conflict | GARP MEDIUM + D1 MEDIUM (HIGH one-shot guard active; BC-2.16.014 EC-005) |

**Edge case note:** GARP-that-conflicts escalation is the only path where two findings are emitted for a single frame. F3 test vectors must verify both findings in emission order: GARP finding first, D1 spoof finding second.

#### Binding-Table LRU Eviction Cap (BC-2.16.006)

| Sequence | Expected `bindings.len()` | Notes |
|---|---|---|
| 65,535 distinct IPs inserted | 65,535 | Below cap; no eviction |
| 65,536th distinct IP | 65,536 | Cap reached; no eviction yet |
| 65,537th distinct IP | 65,536 | LRU evicted; len stays at 65,536 |
| Evicted IP reappears | 65,536 | Treated as first-time; rebind_count=0; no spoof finding on re-appearance (BC-2.16.006 EC-005) |
| Kani scaled: TEST_MAX_ARP_BINDINGS=8; 9 distinct IPs | ≤ 8 after each step | VP-024 Sub-property D |

**Edge case note (ARP-AMB-001):** The LRU substrate (HashMap-ordered LRU vs BTreeMap vs custom) is an F3 implementation choice. The cap invariant (`len ≤ MAX_ARP_BINDINGS`) is specified; the substrate is not. Kani scaled proof uses `#[kani::unwind(12)]` for 9-iteration loop.

#### Malformed ARP Frame Detection (BC-2.16.009)

| ARP payload characteristics | `extract_arp_frame` result | D11 finding | Notes |
|---|---|---|---|
| hw_type=Ethernet, proto=IPv4, hlen=6, plen=4 (well-formed) | `Some(ArpFrame)` | None | Normal path (BC-2.16.001 / BC-2.16.002) |
| hw_type=Ethernet, proto=IPv4, hlen=8, plen=4 (non-standard MAC len) | `None` | LOW finding | E-DEC-004; ARP-AMB-002 re: call site |
| hw_type=0x0006 (IEEE 802), proto=IPv4, hlen=6, plen=4 | `None` | LOW finding | E-DEC-004 |
| hw_type=Ethernet, proto=0x86DD (IPv6), hlen=6, plen=16 | `None` | LOW finding | E-DEC-004 |
| hw_addr_size=0 | `None` | LOW finding | Zero-length HW address |
| proto_addr_size=0 | `None` | LOW finding | Zero-length protocol address |
| etherparse rejects frame entirely (bad EtherType, truncated) | Err before ArpPacketSlice | None (D11 not triggered) | BC-2.16.009 EC-007: decoder error path, not D11 |

**Edge case note (ARP-AMB-004 — RESOLVED in F2):** Malformed frames (returning `None` from `extract_arp_frame`) are excluded from `frames_analyzed`; they are counted separately in the `malformed_frames` key (BC-2.16.010 v1.5). This is no longer an F3 implementation choice — the decision was made in F2 Pass 1 remediation. See spec-changelog §[arp-f2-pass1-remediation-2026-06-12]. (Mirrors the ARP-AMB-003 RESOLVED note above.)

#### ARP Extraction — SLL Capture (outer_src_mac = None) (BC-2.16.001 EC-003)

| `outer_src_mac` | Frame | Expected `ArpFrame.outer_src_mac` | D12 check |
|---|---|---|---|
| `Some([0xAA,0xBB,0xCC,0xDD,0xEE,0xFF])` matching sender_mac | op=1 ARP Request | `Some([0xAA,...])` | D12 check passes; no mismatch finding |
| `Some([0x11,0x22,0x33,0x44,0x55,0x66])` ≠ sender_mac | op=1 ARP Request with sender_mac=AA:AA | `Some([0x11,...])` | D12 check fires MEDIUM finding (BC-2.16.007) |
| `None` (SLL capture) | op=1 ARP Request | `None` | D12 check silently skipped; no mismatch finding |

---

## Cross-Subsystem Integration Vectors

### Integration 1: HTTP Path Traversal End-to-End (SS-04 -> SS-05 -> SS-06 -> SS-09 -> SS-11)

| Scenario | Input | Step 1: Reassembly | Step 2: Dispatch | Step 3: HTTP Analysis | Final Output |
|----------|-------|-------------------|-----------------|----------------------|-------------|
| Attacker sends `GET /../etc/passwd HTTP/1.1\r\nHost: evil.com\r\n\r\n` over TCP flow | pcap with SYN, data, FIN sequence | TcpReassembler delivers data bytes to StreamDispatcher | Dispatcher classifies as HTTP (method prefix `GET `) | HttpAnalyzer detects traversal; emits Finding(Reconnaissance/Likely/High, mitre_techniques: vec!["T1083"]) | JSON: `{"category":"Reconnaissance","verdict":"Likely","confidence":"High","mitre_techniques":["T1083"]...}` |

### Integration 2: TLS SNI C0 Injection (SS-04 -> SS-05 -> SS-07 -> SS-11)

| Scenario | Input | Step 1 | Step 2 | Step 3 | Final Output |
|----------|-------|--------|--------|--------|-------------|
| TLS ClientHello with SNI = `evil\x00host.com` | pcap with TLS handshake | Reassembly delivers TLS record bytes | Dispatcher: first byte 0x16 -> TLS (content-first) | TlsAnalyzer: SNI AsciiWithControl arm; emits Finding(Anomaly/Inconclusive/Low, mitre_techniques: vec!["T1027"]) | JSON: `"mitre_techniques":["T1027"]`, summary contains raw SNI bytes |

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
