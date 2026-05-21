---
document_type: prd-supplement-nfr-catalog
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/prd.md
  - .factory/semport/wirerust/wirerust-pass-4-nfr-catalog.md
  - .factory/semport/wirerust/wirerust-pass-4-deep-nfr-catalog.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - src/
input-hash: "592d3cb"
traces_to: .factory/specs/prd.md
---

# Non-Functional Requirements Catalog: wirerust

> PRD supplement -- extracted from PRD Section 4.
> Referenced by: architect, performance-engineer, formal-verifier.
> Brownfield: built from the 79-NFR ingestion corpus (pass-4 R1 + R2).
> Status reconciled against develop HEAD (post remediation-cycle PRs #69-#98).

## Status Legend

| Status | Meaning |
|--------|---------|
| OPEN | Not yet addressed; gap or violation remains in current source |
| CLOSED | Addressed by remediation cycle; confirmed in current source |
| CLOSED (PR#N) | Closed by a specific PR in the remediation cycle |
| OPEN-DEBT | Open and registered as domain-debt (O-NN) or NFR-VIO; remediation tracked separately |
| N/A | Not a violation; NFR describes a design property that is currently correctly implemented |


## NFR Categories

| Category | Description | Validation Agent |
|----------|-------------|-----------------|
| Performance | Throughput, algorithmic choices, single-pass policy, zero-copy | performance-engineer |
| Security | Escape policy (ADR 0003), no unsafe, no shell-out, no network egress | security-reviewer |
| Reliability | Overflow-checks, panic-freedom, saturating arithmetic, finalize idempotency | formal-verifier |
| Observability | Counters, eprintln channels, AnalysisSummary.detail, MITRE tagging | performance-engineer |
| Resource | Buffer caps, finding caps, map cardinality caps, flow table limits | formal-verifier |
| Maintainability | Clippy, rustfmt, #[non_exhaustive], single-crate, test organization | code-reviewer |
| Portability | MSRV, edition, platform cfg, env vars | performance-engineer |
| Supply Chain | Dependency hygiene, Cargo.lock, no build.rs | security-reviewer |
| Compatibility | Input formats, output formats | test-writer |


## NFR Registry

### Performance (NFR-PERF)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-PERF-001 | Performance | L2-L4 packet decoding uses zero-copy parsing via `etherparse::SlicedPacket`; only L4 payload cloned into `ParsedPacket.payload` | One allocation per packet (payload clone only); zero allocations in the L2-L3 slice path | Code review: confirm only `.to_vec()` in payload extraction | P0 | N/A | N/A -- implemented correctly; grounded in src/decoder.rs:288-291 (`tcp.payload().to_vec()` / `udp.payload().to_vec()` in `build_parsed`) |
| NFR-PERF-002 | Performance | Single eager pass: full pcap loaded into `Vec<RawPacket>` before analysis begins; NOT streaming | RAM usage <= pcap_file_size * ~1.5 (Vec header overhead) | Load test with 1 GB pcap; measure RSS | P1 | NFR-VIO-001 | OPEN-DEBT -- README claim "multi-GB captures" overstates capability if RAM is constrained; documented as NFR-VIO-001 |
| NFR-PERF-003 | Performance | Content-first dispatch classifies each flow exactly ONCE (or zero if data < 5 bytes and port unknown); result cached in `HashMap<FlowKey, DispatchTarget>` | O(1) per subsequent on_data call per flow | Benchmark: 10,000-flow pcap; confirm cache hit rate = 100% after first classification | P0 | N/A | N/A -- implemented correctly; `routes: HashMap<FlowKey, DispatchTarget>` at src/dispatcher.rs:43; cache lookup in `on_data` at src/dispatcher.rs:133-154 |
| NFR-PERF-004 | Performance | Overlap detection uses SIMD-friendly slice equality (`segment_data[..] != existing_data[..]`) rather than byte-by-byte comparison | No benchmark yet; autovectorization confirmed by LLVM IR inspection or `cargo asm` | CI benchmark (not yet present) | P1 | N/A | OPEN-DEBT -- no benchmarks exist (see NFR-MNT gap); confirmed as design intent via code comment at segment.rs:124 |

### Security (NFR-SEC)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-SEC-001 | Security | Raw post-`from_utf8_lossy` bytes stored in `Finding.summary` and `Finding.evidence`; sanitization exclusively at reporter boundary (ADR 0003) | Zero escape calls in analyzer layer; all escape logic in reporter/terminal.rs | Test: `test_output_sanitization_layering_contract` | P0 | N/A | N/A -- ADR 0003 implemented; tests pass |
| NFR-SEC-002 | Security | Terminal reporter escapes C0 (0x00-0x1F), DEL (0x7F), C1 (U+0080-U+009F), and backslash before writing to TTY; preserves printable ASCII and valid non-ASCII Unicode | Zero terminal-injection vulnerabilities; `escape_for_terminal` tests pass | Tests in terminal.rs #[cfg(test)] block + reporter_tests.rs BC-RPT-007..012 | P0 | N/A | N/A -- implemented in src/reporter/terminal.rs:44-61 (`escape_for_terminal` function); all 7 inline tests pass |
| NFR-SEC-003 | Security | Terminal reporter ALSO escapes analyzer-summary detail values (closes C1 gap that serde_json display leaves open) | No C1 codepoints in terminal output path | Test: `test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries` | P0 | N/A | N/A -- src/reporter/terminal.rs:172 (`escape_for_terminal(&val.to_string())` applied to each analyzer detail entry) |
| NFR-SEC-004 | Security | JSON reporter uses `serde_json::to_string_pretty` for RFC 8259 C0 escaping; C1 codepoints pass through as raw UTF-8 (valid for machine consumers) | RFC 8259-compliant output; C0 bytes always \uXXXX escaped | Test: `test_json_reporter_produces_valid_json`; `test_http_finding_c1_csi_in_json_reporter` | P0 | N/A | N/A -- implemented via serde_json delegation |
| NFR-SEC-005 | Security | TLS SNI with embedded C0/DEL bytes emits Anomaly/Inconclusive/Low finding tagged T1027; non-ASCII UTF-8 SNI also emits T1027 Anomaly/Inconclusive/Low | All 4 SniValue arms produce correct findings/no-findings per BC-2.07.013..019 | Tests: `ascii_control_sni_finding_sets_mitre_t1027`; SNI boundary tests BC-TLS-016 | P0 | N/A | N/A -- `SniValue` enum and `extract_sni` at src/analyzer/tls.rs:200-269; finding emission in `handle_client_hello` at src/analyzer/tls.rs:424-489; all SNI tests pass |
| NFR-SEC-006 | Security | No shell-out: zero uses of `std::process::Command`, `Command::new`, or FFI throughout `src/` | 0 shell-out sites | Grep audit: `grep -r "Command::new\|std::process" src/` = 0 | P0 | N/A | N/A -- confirmed by pass-4 cross-cutting audit |
| NFR-SEC-007 | Security | No network I/O: zero uses of `TcpStream`, `UdpSocket`, `tokio::net`, `reqwest`, or any socket API; only `std::net::IpAddr` (value type) used | 0 socket creation sites | Grep audit: `grep -r "TcpStream\|UdpSocket\|reqwest" src/` = 0 | P0 | N/A | N/A -- confirmed; IpAddr only |
| NFR-SEC-008 | Security | TLS record payload validated against `MAX_RECORD_PAYLOAD = 18,432` before allocation; oversized records clear buffer and increment `parse_errors` (DoS protection) | Oversized records never OOM-allocated | Test: `test_oversized_sni_exceeds_record_payload_limit` | P0 | N/A | N/A -- src/analyzer/tls.rs:33 (`MAX_RECORD_PAYLOAD` const), 643-653 (oversized-record guard in `try_parse_records`) |

### Reliability (NFR-REL)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-REL-001 | Reliability | Release builds use `overflow-checks = true` (in `[profile.release]`); arithmetic overflow panics in release, not just debug | 0 silent wrapping overflows in release | Cargo.toml inspection: `overflow-checks = true` at line 31 | P0 | N/A | N/A -- Cargo.toml confirmed |
| NFR-REL-002 | Reliability | TCP sequence-number arithmetic uses `wrapping_sub` to handle 32-bit modular wrap-around correctly | Wraparound test passes: `test_sequence_wraparound` | Test: `tests/reassembly_segment_tests.rs::test_sequence_wraparound` | P0 | N/A | N/A -- src/reassembly/segment.rs:32-34 (`seq_offset` uses `seq.wrapping_sub(isn) as u64`) |
| NFR-REL-003 | Reliability | Saturating arithmetic used at 12 sites to prevent overflow panics on adversarial inputs (u32/u8 counters, buffer depth subtraction, stream offset window check) | 12 sites confirmed; all necessary or defensive | Grep: `grep -rn "saturating_" src/ | wc -l` = 12 | P0 | N/A | N/A -- 12 sites confirmed (pass-4 R2 corrected from 13); all necessary |
| NFR-REL-004 | Reliability | `TcpReassembler::new` defensively asserts all 5 config bounds > 0; panics on invalid config at construction (programmer error only) | Panic only fires on zero-value config; CLI range validators prevent 0 from reaching constructor | Code inspection: mod.rs:107-118 | P1 | N/A | N/A -- src/reassembly/mod.rs:107-118 (5 `assert!` calls in `TcpReassembler::new`) |
| NFR-REL-005 | Reliability | `TcpReassembler::finalize` is idempotent: guarded by `self.finalized: bool`; second call is no-op | No double-flush or duplicate summary findings on double-finalize | Code inspection: mod.rs:103 (`finalized` field), mod.rs:557-561 (guard at top of `finalize`) | P0 | N/A | N/A -- implemented; no explicit double-call test yet |
| NFR-REL-006 | Reliability | `close_flow` on missing key emits ONE process-wide warning via `CLOSE_FLOW_MISSING_WARNED: AtomicBool`; subsequent missing-key closes are silent | Exactly 1 stderr warning per process; no repeated noise | Grep: `CLOSE_FLOW_MISSING_WARNED` in src/reassembly/lifecycle.rs:31, 44-49 (extracted to lifecycle.rs per LESSON-P2.01) | P1 | N/A | N/A -- AtomicBool guard implemented |
| NFR-REL-007 | Reliability | `insert_segment` with no ISN emits ONE process-wide warning via `ISN_MISSING_WARNED: AtomicBool`; returns `InsertResult::IsnMissing`; no panic | Test: `test_isn_missing_returns_isn_missing` passes | Test: segment_tests.rs::test_isn_missing | P0 | N/A | N/A -- AtomicBool guard in segment.rs:16 (`ISN_MISSING_WARNED`), 51-58 (IsnMissing arm with one-shot eprintln) |
| NFR-REL-008 | Reliability | Errors propagate via `anyhow::Result` from all file/reader/decoder paths with context strings; no `panic!` on bad input | Single bad packet does not abort analysis run | Tests: `test_decode_invalid_packet`; `test_unsupported_link_type_rejected` | P0 | N/A | N/A -- anyhow throughout; first-error warning on stderr; count-and-continue |
| NFR-REL-009 | Reliability | Decode errors counted into `Summary.skipped_packets`; FIRST error message only printed to stderr | 1 error message per run; no flooding | Code inspection: main.rs:166-173 (decode error branch in capture loop) | P1 | N/A | N/A -- implemented; no test for suppression specifically |
| NFR-REL-010 | Reliability | TLS oversized record clears per-direction buffer and continues; does not abort analyzer | Test: `test_oversized_sni_exceeds_record_payload_limit` passes | Test: tls_analyzer_tests.rs | P0 | N/A | N/A -- src/analyzer/tls.rs:643-653 |
| NFR-REL-011 | Reliability | HTTP per-direction poison mechanism: `POISON_THRESHOLD = 3` consecutive errors poisons direction; empirically calibrated against `tests/fixtures/http-full.cap` (threshold reduced false-positives from 14 to 3) | Poison fires after exactly 3 consecutive errors; not after 1 or 2 | Tests: BC-HTTP-015..017; `test_parse_error_poisons_direction_after_threshold` | P0 | N/A | N/A -- POISON_THRESHOLD=3 empirically calibrated (pass-4 R2 confirmed) |

### Observability (NFR-OBS)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-OBS-001 | Observability | `ReassemblyStats` exposes 16 counters surfaced via `summarize()` into `AnalysisSummary.detail` | All 16 counter fields populated and accessible | Test: `test_summarize_returns_reassembly_stats` | P1 | N/A | N/A -- `ReassemblyStats` struct in src/reassembly/stats.rs:10-32 (extracted per LESSON-P2.01); `summarize()` in src/reassembly/mod.rs:620-658 |
| NFR-OBS-002 | Observability | Per-analyzer `summarize()` returns uniform `AnalysisSummary{analyzer_name, packets_analyzed, detail: HashMap}` shape | All 3 analyzers + reassembler produce this shape | Tests: per-analyzer summarize tests | P1 | N/A | N/A -- trait enforced at compile time |
| NFR-OBS-003 | Observability | Parse errors counted (not swallowed): `HttpAnalyzer.parse_errors`, `TlsAnalyzer.parse_errors` | Counters increment correctly on malformed input | Tests: `test_parse_error_increments_counter` (HTTP, TLS) | P0 | N/A | N/A -- http.rs:405, 463 (`parse_errors += 1` in request/response error branches); tls.rs:394, 555, 644 (extension-parse error, server-hello extension error, oversized-record guard) |
| NFR-OBS-004 | Observability | Findings carry MITRE ATT&CK technique IDs where a clean mapping exists; `None` preferred over a forced fit | All 15 seeded technique IDs resolve; no force-fit | Test: `known_emitted_technique_ids_resolve_in_lookup` | P1 | N/A | N/A -- src/mitre.rs |
| NFR-OBS-005 | Observability | `unclassified_flows` counter from dispatcher injected into reassembly `AnalysisSummary.detail` | Counter visible in JSON/terminal output | Tests: `test_unclassified_flows_counter` | P1 | N/A | N/A -- counter field at src/dispatcher.rs:53; accessor at 80-81; incremented at 188-191 (`on_flow_close` None arm); injected into summary at src/main.rs:205-209 |
| NFR-OBS-006 | Observability | One-shot one-line stderr warnings for: ISN-missing insert, close-flow missing key, first decode error, --no-reassemble conflict with --http/--tls | Exactly 1 message per event category per process | Code inspection + manual testing | P1 | N/A | N/A -- AtomicBool guards + early-return pattern |
| NFR-OBS-007 | Observability | Per-target progress bar via `indicatif` written to stderr; template `[elapsed] {bar:40} pos/len packets` | Progress visible on stderr without polluting stdout | Manual test with large pcap | P2 | N/A | N/A -- src/main.rs:149-152 (ProgressBar creation and style template) |
| NFR-OBS-008 | Observability | `--mitre` flag groups findings under tactic headers in kill-chain order; `Uncategorized` bucket last | 16 tactic headers + Uncategorized; stable order per `all_tactics_in_report_order` | Tests: `mitre_grouping_*` (BC-RPT-013..017) | P0 | N/A | N/A -- src/reporter/terminal.rs:253-310 (`render_findings_grouped`) |
| NFR-OBS-009 | Observability | `Skipped: N packets` line suppressed when N = 0; only shown when N > 0 | No spurious "Skipped: 0" in clean-capture output | Tests: `test_terminal_reporter_shows_skipped_when_nonzero`, `test_terminal_reporter_hides_skipped_when_zero` | P1 | N/A | N/A -- src/reporter/terminal.rs:94-105 (`if summary.skipped_packets > 0` guard) |
| NFR-OBS-010 | Observability | `Finding` JSON schema uses symmetric `skip_serializing_if = "Option::is_none"` on ALL four Option fields: `mitre_technique`, `source_ip`, `timestamp`, `direction` | All None Option fields omitted (not serialized as null); downstream consumers must handle key-absent | Test: BC-2.09.006; `test_finding_serializes_with_or_without_options` (proposed) | P0 | NFR-VIO previous | CLOSED (LESSON-P1.02) -- src/findings.rs:132-145 shows all four Option fields now carry `skip_serializing_if = "Option::is_none"`; the previous asymmetry where only `timestamp` had the attribute is corrected |

### Resource Bounds (NFR-RES)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-RES-001 | Resource | Max engine-level findings: `MAX_FINDINGS = 10,000`; further per-flow findings silently dropped; `finalize()` summary finding unconditionally bypasses cap | findings.len() <= 10,001 (10,000 + 1 finalize finding) | Test: BC-2.04.024 / BC-2.04.054 | P0 | ADR 0002 | N/A -- src/reassembly/mod.rs:54 (`const MAX_FINDINGS: usize = 10_000`) |
| NFR-RES-002 | Resource | Per-flow-direction overlap-anomaly alert fires when `overlap_count > overlap_alert_threshold` (default 50); exactly ONCE per direction (sticky latch). Worst-case 6 findings per bidirectional flow (3 alert types x 2 directions) | At most 6 findings per flow from threshold alerts; at most 10,000 total via MAX_FINDINGS | Tests: `test_overlap_anomaly_finding`; latch tests | P0 | N/A | N/A -- default in config.rs:125 (`overlap_alert_threshold: 50`); alert logic in mod.rs:430-449; threshold moved from module const to ReassemblyConfig field per LESSON-P2.05 |
| NFR-RES-003 | Resource | Per-flow-direction small-segment alert fires when `small_segment_run > small_segment_alert_threshold` (default 100); exactly ONCE per direction | At most 1 alert per direction per flow | (No direct test -- BC-RAS-020 P1 planned) | P1 | N/A | N/A -- default in config.rs:126 (`small_segment_alert_threshold: 100`); alert logic in mod.rs:457-488; threshold moved to ReassemblyConfig per LESSON-P2.05 |
| NFR-RES-004 | Resource | Per-flow-direction out-of-window alert fires when `out_of_window_count > out_of_window_alert_threshold` (default 100); exactly ONCE per direction | At most 1 alert per direction per flow | Tests: `test_out_of_window_threshold_alert`; `test_out_of_window_alert_fires_only_once` | P1 | N/A | N/A -- default in config.rs:129 (`out_of_window_alert_threshold: 100`); alert logic in mod.rs:489-512; threshold moved to ReassemblyConfig per LESSON-P2.05 |
| NFR-RES-005 | Resource | Default per-direction reassembly depth: `max_depth = 10 MB` (10 * 1024 * 1024); CLI override: `--reassembly-depth <MB>` | Depth truncation at 10 MB per direction by default | Tests: `test_depth_limit_truncation`; `test_reassembly_flags` | P0 | N/A | N/A -- config.rs:119 (`max_depth: 10 * 1024 * 1024`); cli.rs:70-71 (`reassembly_depth` flag with default_value_t = 10) |
| NFR-RES-006 | Resource | Default global reassembly memcap: `memcap = 1 GB`; CLI override: `--reassembly-memcap <MB>` | Eviction triggers above 1 GB total buffered bytes | Tests: `test_memcap_eviction` (uses small value) | P0 | N/A | N/A -- config.rs:120 (`memcap: 1024 * 1024 * 1024`); cli.rs:73-75 (`reassembly_memcap` flag with default_value_t = 1024) |
| NFR-RES-007 | Resource | Default flow timeout: `flow_timeout_secs = 300` (5 min) | Idle flows evicted after 300 seconds | Tests: `test_flow_timeout_expiration` | P1 | N/A | N/A -- config.rs:121 (`flow_timeout_secs: 300`) |
| NFR-RES-008 | Resource | Default max concurrent tracked flows: `max_flows = 100,000` | LRU-style eviction triggers above 100,000 concurrent flows | Tests: `test_max_flows_eviction` (uses small value) | P1 | N/A | N/A -- config.rs:122 (`max_flows: 100_000`) |
| NFR-RES-009 | Resource | Default max segments per direction: `max_segments_per_direction = 10,000` | Segment-limit finding emitted via finalize() when exceeded | Tests: `test_max_segments_per_direction`; `test_segment_limit_*` | P1 | N/A | N/A -- config.rs:123 (`max_segments_per_direction: 10_000`) |
| NFR-RES-010 | Resource | Default max receive window: `max_receive_window = 1 MB` (1,048,576 bytes); matches Suricata/Zeek/Snort default | Out-of-window segments rejected above 1 MB offset | Tests: `test_out_of_window_segment_rejected_by_engine` | P0 | industry | N/A -- config.rs:124 (`max_receive_window: 1_048_576`) |
| NFR-RES-011 | Resource | HTTP per-direction header buffer cap: `MAX_HEADER_BUF = 65,536` bytes; bytes past cap silently dropped | No OOM on HTTP request with header larger than 64 KB | Test: `test_buffer_cap_no_panic_on_oversized_headers` | P0 | N/A | N/A -- http.rs:21 (`const MAX_HEADER_BUF`); cap enforced in `on_data` at http.rs:513, 525 |
| NFR-RES-012 | Resource | HTTP max headers per message: `MAX_HEADERS = 96`; exceeding emits Anomaly/Inconclusive/Medium (T1499.002) | Exactly 1 finding when > 96 headers; 0 findings at exactly 96 | Tests: `test_too_many_headers_generates_finding` | P0 | N/A | N/A -- http.rs:22 (`const MAX_HEADERS: usize = 96`); TooManyHeaders finding emitted at http.rs:416-428 (request) and 475-487 (response) |
| NFR-RES-013 | Resource | HTTP URI list cap: `MAX_URIS = 10,000`; further URIs silently dropped from list (detection still runs) | `uris` list never exceeds 10,000 entries | (No direct test -- BC-HTTP-025 P2 planned) | P2 | N/A | N/A -- http.rs:23 (`const MAX_URIS: usize = 10_000`); cap enforced at http.rs:391-393 (`if self.uris.len() < MAX_URIS`) |
| NFR-RES-014 | Resource | HTTP/TLS per-map cardinality cap: `MAX_MAP_ENTRIES = 50,000`; new keys silently dropped past cap; existing keys still increment | All 8 aggregate maps bounded at 50,000 keys | Test: `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` (TLS); HTTP analogous | P0 | ADR 0002 | N/A -- http.rs:24 (`const MAX_MAP_ENTRIES: usize = 50_000`); tls.rs:30 (same const) |
| NFR-RES-015 | Resource | TLS per-direction record-assembly buffer cap: `MAX_BUF = 65,536` bytes | No OOM on large TLS record stream | (No direct test -- BC-TLS-005 P1 planned) | P1 | N/A | N/A -- tls.rs:29 (`const MAX_BUF: usize = 65_536`); cap enforced in `on_data` at tls.rs:733, 740 |
| NFR-RES-016 | Resource | TLS record payload sanity cap: `MAX_RECORD_PAYLOAD = 18,432`; oversized records increment `parse_errors` and clear buffer (RFC 5246: TLS 1.2 ciphertext max; 18,432 >= TLS 1.3 max of 16,640) | Oversized records never allocated | Test: `test_oversized_sni_exceeds_record_payload_limit` | P0 | RFC 5246/8446 | N/A -- tls.rs:31-33 (`MAX_RECORD_PAYLOAD` const with RFC comment); guard at tls.rs:643-653 |
| NFR-RES-017 | Resource | HTTP poison threshold: `POISON_THRESHOLD = 3`; empirically calibrated against `http-full.cap` fixture | Exactly 3 consecutive errors required to poison; not 2 | Tests: `test_parse_error_poisons_direction_after_threshold`; `test_single_error_does_not_poison` | P0 | empirical (PR #42) | N/A -- http.rs:80 (`const POISON_THRESHOLD: u8 = 3`) |
| NFR-RES-018 | Resource | HTTP URI truncation in findings: summary cap = 120 chars; evidence cap = 200 chars (display-layer wart per ADR 0003) | URIs truncated in finding strings but detection threshold (2048) uses full URI | Test: `test_detect_long_uri` | P1 | ADR 0003 wart | N/A -- http.rs:196, 225 (summary truncated at 120 via `truncate_uri`); http.rs:311 (evidence truncated at 200); http.rs:305 (detection uses full `uri.len()`); acknowledged ADR wart |
| NFR-RES-019 | Resource | HTTP "abnormally long URI" finding threshold: URIs longer than 2,048 chars emit Execution/Likely/Medium finding | Finding emitted for URI.len() > 2048; not for URI.len() == 2048 | Test: `test_detect_long_uri` | P1 | N/A | N/A -- http.rs:305-316 (`if parsed.uri.len() > 2048`) |
| NFR-RES-020 | Resource | `summarize()` truncates to top-20 hosts / SNIs / recent URIs in output; bounded JSON regardless of cardinality | Top-20 truncation in all three fields | Tests: `test_summarize_produces_complete_output` (HTTP); `test_summarize_output` (TLS) | P1 | N/A | N/A -- http.rs:573 (`top_hosts.iter().take(20)`); http.rs:576 (`uris.iter().take(20)`); tls.rs:773 (`top_snis.iter().take(20)`) |
| NFR-RES-021 | Resource | TLS handshake-only short-circuit: once both `client_hello_seen` and `server_hello_seen` are true, `on_data` short-circuits; no further buffering per flow | Memory bounded per flow after handshake completes | Test: `test_stop_after_handshake` | P0 | N/A | N/A -- `done()` method at tls.rs:291-293; `on_data` early return at tls.rs:721-723 |
| NFR-RES-022 | Resource | (PROPOSED) When MAX_FINDINGS is reached, further pushes are silently dropped; a `dropped_findings: u64` counter on `ReassemblyStats` makes the drop observable via `summarize().detail` | `dropped_findings` counter visible in AnalysisSummary; non-zero when cap was hit | Implement counter in reassembly/mod.rs; test: `test_dropped_findings_counter_increments_when_cap_reached` | P1 | N/A | OPEN -- counter not yet implemented; recommended by pass-4 R2; GitHub issue pending |
| NFR-RES-023 | Resource | ClientHello weak-cipher Finding evidence vec is data-dependent-cardinality: upper bound ~9,216 entries (MAX_RECORD_PAYLOAD / 2-bytes-per-cipher); worst-case Finding heap ~270-500 KB; no per-cipher cap yet | Recommend `MAX_WEAK_CIPHER_EVIDENCE = 64` truncation cap with "+N more" entry | Stress test: Finding.evidence.len() <= 9,216 | P1 | O-06 | OPEN -- no per-cipher truncation cap; worst-case heap unbounded at data layer; weak-cipher `evidence` built at tls.rs:497-516; GitHub issue #102 |

### Maintainability (NFR-MNT)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-MNT-001 | Maintainability | `RUSTFLAGS=-Dwarnings` in CI; zero `#[allow(...)]` clusters in `src/` | 0 compiler warnings in CI | CI: `cargo test` with `-Dwarnings` flag | P0 | N/A | N/A -- .github/workflows/ci.yml:10-12, 58 |
| NFR-MNT-002 | Maintainability | `cargo clippy --all-targets -- -D warnings` separate CI job | 0 clippy warnings | CI job "clippy" | P0 | N/A | N/A -- ci.yml:49-58 |
| NFR-MNT-003 | Maintainability | `cargo fmt --all --check`; `rustfmt.toml` pins edition=2024, max_width=100, field-init shorthand, try shorthand | 0 formatting violations in CI | CI job "fmt" | P0 | N/A | N/A -- ci.yml:60-68; rustfmt.toml:1-5 |
| NFR-MNT-004 | Maintainability | `MitreTactic` is `#[non_exhaustive]` so adding new ATT&CK variants is non-breaking for downstream | Downstream crates must use `_` arm in matches | Compile-time (non_exhaustive attribute) | P1 | N/A | N/A -- src/mitre.rs:45-47 (`#[non_exhaustive]` attribute on `MitreTactic` enum) |
| NFR-MNT-005 | Maintainability | All tests under `tests/`; zero inline `#[test]` modules in `src/` except `terminal.rs` (private helper tested inline per convention) | 1 allowed inline test module (terminal.rs escape helper) | Grep: `grep -rn "#\[test\]" src/` | P1 | N/A | N/A -- confirmed by pass-4 |
| NFR-MNT-006 | Maintainability | No per-file SPDX license headers; license declared once at `Cargo.toml:6` (MIT) and `LICENSE` | 0 SPDX comments in src/*.rs | Grep: `grep -rn "SPDX" src/` = 0 | P2 | N/A | N/A -- project-wide convention |
| NFR-MNT-007 | Maintainability | Semantic-PR titles enforced by `amannn/action-semantic-pull-request@v6`; types: feat/fix/docs/style/refactor/perf/test/build/ci/chore/revert | 0 PRs merged with non-semantic title | CI gate (PR check) | P1 | N/A | N/A -- ci.yml:14-38 |
| NFR-MNT-008 | Maintainability | Threshold-fired alerts use sticky boolean fields (`overlap_alert_fired`, `small_segment_alert_fired`, `out_of_window_alert_fired`) -- not ad-hoc counters compared on each call site | Single pattern for all 3 alert types; easier to audit | Code review | P1 | N/A | N/A -- src/reassembly/flow.rs:93, 102, 104 (three alert_fired fields on `FlowDirection`) |
| NFR-MNT-009 | Maintainability | `technique_info` is single source of truth for `(name, tactic)` per MITRE ID; `technique_name` and `technique_tactic` are thin projections | Adding a technique requires one change in one function | Tests: `technique_tactic_matches_spec_table`; `technique_name_resolves_every_seeded_id` | P1 | N/A | N/A -- src/mitre.rs:122-156 (`technique_info` match); projections at 160-167 |
| NFR-MNT-010 | Maintainability | Four ADRs (`docs/adr/0001..0004`) document content-first dispatch, modular analyzers, reporting pipeline layering, process-wide warning atomics; referenced from doc comments | 4 ADRs present and referenced | File existence check + grep for ADR references | P1 | N/A | N/A -- ADR 0004 added in remediation cycle |
| NFR-MNT-011 | Maintainability | Effective MSRV declared via `rust-version = "1.91"` in Cargo.toml; `floor_char_boundary` requires >= 1.86 (stabilized Rust 1.86, 2025-04-03); declared value 1.91 is correct and conservative (1.91 >= 1.86) | User with Rust < 1.91 gets clear MSRV-mismatch diagnostic; floor_char_boundary guaranteed present | Cargo.toml:5 inspection; `cargo build` on Rust 1.91+ | P1 | NFR-VIO-009 | CLOSED -- `rust-version = "1.91"` declared at Cargo.toml:5; value is correct (higher than the 1.86 minimum; may be constrained by other features). `floor_char_boundary` call at src/analyzer/http.rs:110. NFR-VIO-009 is resolved. |

### Portability (NFR-PORT)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-PORT-001 | Portability | CI runs only on `ubuntu-latest`; no macOS/Windows job | Known: macOS/Windows regressions can land silently | CI matrix review | P2 | NFR-VIO-010 | OPEN-DEBT -- single-platform CI accepted for now (cost-vs-benefit decision); document Linux-only stance in README if macOS install path is advertised |
| NFR-PORT-002 | Portability | Zero platform-specific `cfg` in `src/`; code portable across any libc supporting etherparse/pcap-file | 0 `#[cfg(target_os = ...)]` or `#[cfg(target_arch = ...)]` attributes | Grep: `grep -rn "cfg.*target" src/` = 0 | P1 | N/A | N/A -- confirmed by pass-4 audit |
| NFR-PORT-003 | Portability | No `build.rs` and no `rust-toolchain.toml`; effective MSRV tracks `dtolnay/rust-toolchain@stable` + 1.86 minimum | Binary compiles on stable Rust >= 1.86 | CI build on stable | P1 | N/A | N/A -- no build.rs confirmed |
| NFR-PORT-004 | Portability | Only env var read at runtime: `NO_COLOR` (no-color.org convention); no other env-var config | 1 env var consumed; no `$WIRERUST_*` or similar config vars | Grep: `grep -rn "env::var" src/` | P1 | N/A | N/A -- src/main.rs:43 |
| NFR-PORT-005 | Portability | Rust 2024 edition (`Cargo.toml:4`; `rustfmt.toml:1`); uses let-chains and other 2024 stabilized features | Edition 2024 declared and enforced | CI build; rustfmt edition check | P0 | N/A | N/A -- Cargo.toml:4 |

### Supply Chain (NFR-SUP)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-SUP-001 | Supply chain | Direct production dependencies: current count per Cargo.toml; unused `rayon` dep declared but not used in any `src/` file | Minimal transitive surface; no unused prod deps | `cargo tree --depth 1 --edges normal`; `grep -r "rayon" src/` | P1 | NFR-VIO-006 | OPEN -- `rayon = "1"` present at Cargo.toml:28 but zero uses in src/; NFR-VIO-006 firm-fix disposition (cost: S; remove line); GitHub issue pending |
| NFR-SUP-002 | Supply chain | Dev dependencies `assert_cmd`, `predicates`, `tempfile` are used (post-remediation integration tests wired) | All dev-deps have at least one test consumer | `grep -r "assert_cmd\|predicates\|tempfile" tests/` | P1 | NFR-VIO-007 | CLOSED -- integration tests using `assert_cmd` were added in remediation cycle per BC-2.13 cleanup |
| NFR-SUP-003 | Supply chain | `Cargo.lock` checked in; appropriate for a binary crate | Reproducible builds | File presence: `git ls-files Cargo.lock` | P0 | N/A | N/A -- Cargo.lock present |
| NFR-SUP-004 | Supply chain | Dependency version requirements use minor-version pins (caret SemVer); no exact-version pins, no git deps, no path deps | All deps use `"major"` or `"major.minor"` form | Cargo.toml review | P1 | N/A | N/A -- all caret pins confirmed |
| NFR-SUP-005 | Supply chain | No `[build-dependencies]`, no `build.rs`, no procedural macros beyond `clap`/`serde` derive | Zero build-script attack surface | File absence: `find . -name "build.rs"` | P0 | N/A | N/A -- confirmed |

### Compatibility (NFR-COMPAT)

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source | Status |
|----|----------|-------------|--------|------------------|----------|-------------|--------|
| NFR-COMPAT-001 | Compatibility | Only classic pcap accepted; 5 link types: Ethernet (1), RAW (101), Linux SLL (113), IPv4 (228), IPv6 (229); all others rejected with message listing supported types | 0 unsupported-link-type pcaps silently accepted | Tests: `test_unsupported_link_type_rejected`; `test_pcapng_rejected` | P0 | N/A | N/A -- src/reader.rs:50-61 |
| NFR-COMPAT-002 | Compatibility | Output formats: terminal (default) and JSON (`--output-format json` / `--json`). CSV declared via `OutputFormat::Csv` and `CsvReporter` is wired (remediation cycle) | All three output paths produce non-empty output | Tests: reporter tests for json, terminal, csv | P0 | NFR-VIO-005 | CLOSED (remediation cycle) -- `CsvReporter` wired; `--output-format csv` and `--csv <FILE>` now produce CSV output |


## NFR-to-Module Mapping

| NFR ID | Affected Subsystem / Source Files | Architectural Impact |
|--------|----------------------------------|---------------------|
| NFR-PERF-001 | SS-02 (decoder.rs) | Zero-copy slice API must not be broken by any decoder refactor |
| NFR-PERF-002 | SS-01 (reader.rs) | All-in-memory design constraint; streaming refactor is O-01 class debt |
| NFR-PERF-003 | SS-05 (dispatcher.rs) | Cache must be preserved; no per-packet re-classification |
| NFR-SEC-001..003 | SS-06, SS-07, SS-09, SS-11 | ADR 0003 must be maintained across all new analyzers and reporters |
| NFR-SEC-008 | SS-07 (tls.rs) | MAX_RECORD_PAYLOAD must not be raised without RFC justification |
| NFR-REL-001 | All (Cargo.toml) | overflow-checks=true must not be disabled in [profile.release] |
| NFR-REL-003 | SS-04, SS-06, SS-07 | Saturating arithmetic must be preserved on any refactor of counter/buffer math |
| NFR-RES-001 | SS-04 (reassembly/mod.rs) | MAX_FINDINGS cap must be preserved; finalize bypass is intentional design |
| NFR-RES-005..010 | SS-04 (reassembly/config.rs) | ReassemblyConfig defaults are CLI-documented; changing defaults is a breaking change |
| NFR-RES-011..012 | SS-06 (analyzer/http.rs) | MAX_HEADER_BUF and MAX_HEADERS are correctness constants, not tuning knobs |
| NFR-RES-014..016 | SS-07 (analyzer/tls.rs) | MAX_MAP_ENTRIES / MAX_RECORD_PAYLOAD / MAX_BUF constrain adversarial input bounds |
| NFR-RES-023 | SS-07 (analyzer/tls.rs:497-516) | Weak-cipher evidence vec is the only data-dependent-cardinality evidence vector |
| NFR-OBS-010 | SS-09 (findings.rs) | JSON schema Option symmetry must be preserved across all Finding fields |
| NFR-MNT-011 | SS-06 (analyzer/http.rs:110) | MSRV 1.86 constraint; floor_char_boundary must not be replaced with a 1.85-compatible workaround without updating Cargo.toml |


## Open NFR Summary

| Count | Status | Severity | Details |
|-------|--------|----------|---------|
| 70 | N/A (correctly implemented) | -- | No violation; describes correct current behavior |
| 4 | CLOSED by remediation cycle | -- | NFR-OBS-010 (Option symmetry, LESSON-P1.02), NFR-COMPAT-002 (CSV wired), NFR-SUP-002 (dev deps used), NFR-MNT-011 (rust-version=1.91 declared, NFR-VIO-009 resolved) |
| 5 | OPEN | High/Medium | NFR-RES-022 (dropped_findings counter not yet implemented), NFR-RES-023 (weak-cipher heap bound, GitHub #102), NFR-SUP-001 (rayon dep still present), NFR-PERF-004 (no benchmarks), NFR-PORT-001 (single-platform CI) |

Total NFRs: 79 (4 PERF + 8 SEC + 11 REL + 10 OBS + 23 RES + 11 MNT + 5 PORT + 5 SUP + 2 COMPAT)


## NFR Violation Dispositions (from Pass-4 R2)

The following 10 NFR violations were identified in the ingestion corpus. Dispositions
are recorded here for traceability. Closed items were addressed in PRs #69-#98.

| VIO ID | Description | Disposition | Status |
|--------|------------|-------------|--------|
| NFR-VIO-001 | "Multi-GB captures" README claim vs. eager `Vec<RawPacket>` load | document-and-accept | OPEN-DEBT (O-01 class) |
| NFR-VIO-002 | `resolve_targets` glob included `*.pcapng` but reader rejects pcapng | fix (S) | CLOSED -- pcapng extension excluded from glob in remediation cycle |
| NFR-VIO-003 | 8 unwired CLI flags (`--threats`, `--beacon`, `--filter`, `--verbose`, `--hosts`, `--services`, `--json <FILE>`, `--csv <FILE>`) | fix (mixed) | CLOSED -- unwired flags removed; `--json`/`--csv` wired; `--hosts` wired (LESSON-P1.03) |
| NFR-VIO-004 | `--json/--csv <FILE>` wrote to stdout, not the specified file | fix (S) | CLOSED -- `std::fs::write` wired in `write_output()` |
| NFR-VIO-005 | `OutputFormat::Csv` declared but dispatch fell through to terminal | fix (M) | CLOSED -- CsvReporter implemented and wired |
| NFR-VIO-006 | `rayon` declared, zero uses | fix (S) | OPEN -- `rayon = "1"` still present at Cargo.toml:28 as of develop HEAD; zero src/ uses confirmed; single-line removal pending |
| NFR-VIO-007 | `assert_cmd`, `predicates`, `tempfile` dev-deps unused | fix (S) | CLOSED -- integration tests added using these deps |
| NFR-VIO-008 | `serde_json::to_string_pretty(&output).unwrap()` in JsonReporter | document-and-accept | CLOSED (LESSON cosmetic) -- infallible by construction; acceptable as-is per pass-4 R2 disposition |
| NFR-VIO-009 | Effective MSRV 1.86 undeclared in Cargo.toml | fix (S) | CLOSED -- `rust-version = "1.91"` declared at Cargo.toml:5. The declared value deliberately exceeds the computed minimum: 1.91 was chosen after clippy reported `incompatible_msrv` against a lower pin (LESSON-P0.01). This is an intentionally conservative declaration; it may be higher than the true minimum required to compile the crate, and that is accepted. |
| NFR-VIO-010 | CI only on `ubuntu-latest` | document-and-accept with caveat | OPEN-DEBT -- macOS/Windows matrix expansion deferred; NFR-PORT-001 |
