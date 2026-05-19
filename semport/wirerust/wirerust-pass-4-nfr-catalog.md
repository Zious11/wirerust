# Pass 4: NFR Catalog -- wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 4 (NFR Catalog) -- Phase A broad-sweep, round 1
- **Inputs consumed:** Pass 0 inventory, Pass 1 architecture, Pass 2 domain model, Pass 3 behavioral contracts (137 BCs), ADR 0001/0002/0003, `Cargo.toml`, `rustfmt.toml`, `.github/workflows/ci.yml`, and all 20 `.rs` files under `src/`.
- **NFRs in this document:** 76 (NFR-PERF 4 + NFR-SEC 8 + NFR-REL 11 + NFR-OBS 9 + NFR-RES 21 + NFR-MNT 11 + NFR-PORT 5 + NFR-SUP 5 + NFR-COMPAT 2). Numeric-value index lists 28 magic-number constants/defaults.
- **Confidence (overall):** HIGH for resource-bound NFRs (all magic numbers grounded by file:line; many pinned by tests), HIGH for security-escaping NFRs (ADR 0003 + inline tests), HIGH for portability/supply-chain (manifests + CI YAML directly read), MEDIUM for performance (no benchmarks in repo; "fast" is a marketing claim not a measured threshold), MEDIUM for observability (eprintln + counters in lieu of structured logs).

---

## 1. NFR Taxonomy in Use

Pass 4 catalogues NFRs under nine categories. NFR-IDs are stable across deepening rounds.

| Prefix | Category | Definition for wirerust |
|---|---|---|
| `NFR-PERF` | Performance (latency / throughput) | Algorithmic choices, single-pass eager-vs-streaming policy, zero-copy claims, content-first dispatch overhead bounds. |
| `NFR-SEC` | Security | Defensive treatment of attacker-controlled bytes (escape policy per ADR 0003), input validation, no shell-out, no network egress, no `unsafe`. |
| `NFR-REL` | Reliability | Panic-freedom in `src/`, error propagation via `anyhow::Result`, one-shot warn guards, finalize-idempotency, saturating/wrapping arithmetic. |
| `NFR-OBS` | Observability | `eprintln!` warning channels, `ReassemblyStats` counters, `AnalysisSummary.detail` maps, indicatif progress bar, MITRE technique tagging on findings. |
| `NFR-RES` | Resource bounds | Memcap, max-flows, per-flow depth, segments-per-direction, max-receive-window, per-direction header/record buffer caps, finding caps, cardinality caps. |
| `NFR-MNT` | Maintainability | `clippy -D warnings`, rustfmt 100-col edition 2024, single-crate, `#[non_exhaustive]` on `MitreTactic`, zero `#[allow]` clusters, no per-file license headers, tests in `tests/` only. |
| `NFR-PORT` | Portability | Linux-only CI matrix, no platform `cfg`, stable toolchain, edition 2024 (MSRV 1.85+), no `build.rs`. |
| `NFR-SUP` | Supply chain | 14 direct prod deps + 3 dev deps (3 prod + 3 dev unused), `Cargo.lock` checked in, minor-version pins like `clap = "4"`. |
| `NFR-COMPAT` | Input/output compatibility | Classic pcap only (pcapng rejected), 5 supported link types, supported output formats (terminal + JSON; CSV declared but unwired). |

Cross-cutting audit results (panic-freedom, no-`unsafe`, no-net, no-env, time policy, MSRV, lockfile policy) are recorded in §5.

---

## 2. NFR Catalog -- Per-Category Tables

### 2.1 Performance (NFR-PERF)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value (rationale) | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-PERF-001 | Performance | L2-L4 packet decoding uses zero-copy parsing via `etherparse::SlicedPacket`; only the L4 payload is cloned via `.to_vec()` into `ParsedPacket.payload`. | `src/decoder.rs:72-130` | -- | README L17 ("zero-copy packet parsing", "built for multi-GB captures"); avoids per-packet allocation in the slicing path. | Implicit (library choice) | All decoder_tests indirectly; no perf assertion |
| NFR-PERF-002 | Performance | Single eager pass: `PcapSource::from_pcap_reader` slurps the entire pcap into a `Vec<RawPacket>` before analysis begins. Not streaming. | `src/reader.rs:38-48` | -- | Simplicity over streaming; pcap files are bounded inputs. Note: contradicts a literal reading of "streaming" -- README says "one-pass triage", not "streaming". See §7 NFR-VIO-001. | Implicit | `tests/reader_tests.rs::test_read_pcap_packets` |
| NFR-PERF-003 | Performance | Content-first dispatch performs exactly one classification per flow on first delivery (or zero if data<5 bytes and port is unknown), then caches the decision in `HashMap<FlowKey, DispatchTarget>`. | `src/dispatcher.rs:72-81`, ADR 0001 §Rationale | -- | ADR 0001: "Classification requires reading 5 bytes on the first data delivery per flow -- negligible compared to reassembly and parsing costs." | Runtime cache | `tests/dispatcher_tests.rs` (cache hit/miss exercised) |
| NFR-PERF-004 | Performance | Overlap-detection uses SIMD-friendly slice equality (`segment_data[..] != existing_data[..]`) rather than byte-by-byte. | `src/reassembly/segment.rs:113-125` (`// Use slice comparison (SIMD-optimized)`) | -- | Comment in code; relies on Rust stdlib `PartialEq<[u8]>` autovectorization. | Compile-time (codegen) | `tests/reassembly_segment_tests.rs::test_overlap_first_wins` and conflict tests |

### 2.2 Security (NFR-SEC)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value (rationale) | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-SEC-001 | Security | The data layer (`Finding.summary`, `Finding.evidence`) holds raw post-`from_utf8_lossy` bytes; sanitization is the reporter's responsibility. | `src/findings.rs:71-92` doc comment; ADR 0003 §Decision; `src/analyzer/tls.rs:393-446` comments | -- | ADR 0003 §Problem ("Tribal-knowledge enforcement"); preserves forensic data for JSON/SIEM consumers while making display safety a reporter contract. | Documented contract + tests | `tests/reporter_tests.rs::test_output_sanitization_layering_contract`; `test_non_utf8_sni_preserves_raw_bytes_in_summary` |
| NFR-SEC-002 | Security | Terminal reporter escapes C0 controls (0x00-0x1F), DEL (0x7F), C1 controls (U+0080-U+009F), and backslash before writing to TTY; passes through all printable ASCII and valid non-ASCII Unicode (Cyrillic/CJK/emoji). | `src/reporter/terminal.rs:29-46` (`escape_for_terminal`) | C1 range = U+0080..=U+009F | ADR 0003 §"Why C1?"; closes CWE-117 terminal-injection vector; `char::escape_default` chosen over `str::escape_default` because the latter mangles non-ASCII Unicode. | Runtime | terminal.rs inline tests (`escapes_esc_byte`, `escapes_bel_and_del`, `escapes_c1_nel_and_csi`, `escapes_c1_range_boundaries`, `preserves_cyrillic`, `preserves_emoji`, `mixed_content_escapes_only_dangerous_bytes`); reporter_tests `test_terminal_reporter_escapes_*` (BC-RPT-007..012) |
| NFR-SEC-003 | Security | Terminal reporter ALSO escapes analyzer-summary detail values (e.g. `top_hosts`, `top_snis`, `recent_uris`) because `serde_json`'s display impl passes C1 codepoints through unescaped. | `src/reporter/terminal.rs:125-135` | -- | ADR 0003 §"The layering rule" (terminal layer escapes per terminal-safety); closes the C1 (U+009B = CSI) gap left by RFC 8259's relaxed encoding. | Runtime | `tests/reporter_tests.rs::test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries`, `test_http_analyzer_summary_c1_csi_escaped_by_terminal_reporter` |
| NFR-SEC-004 | Security | JSON reporter relies on `serde_json::to_string_pretty` for RFC 8259 escaping of C0+DEL; this is sufficient for machine consumers. | `src/reporter/json.rs:36` | -- | ADR 0003 §"Immediate scope: terminal-safe escaping" -- JSON is escaped automatically; C1 codepoints pass through but are valid UTF-8 and round-trip cleanly for downstream tools. | Runtime (delegated) | `tests/reporter_tests.rs::test_json_reporter_produces_valid_json`, `test_json_reporter_preserves_cyrillic_as_readable_unicode`, `test_http_finding_c1_csi_in_json_reporter` |
| NFR-SEC-005 | Security | TLS SNI hostnames with embedded ASCII control bytes (C0/DEL) are flagged as `Anomaly/Inconclusive/Low` findings tagged with MITRE T1027 (Obfuscated Files or Information). | `src/analyzer/tls.rs:204-238` (`SniValue` discriminant), `tls.rs:384-446` (finding emission) | C0 range 0x00..=0x1F, DEL 0x7F | RFC 6066 §3 + RFC 952/1123 hostname syntax; the comment at `tls.rs:155-172` cites adversary log-poisoning / covert-channel signal. | Runtime | `tests/tls_analyzer_tests.rs::ascii_control_sni_finding_sets_mitre_t1027` (and many BC-TLS-014..020 tests) |
| NFR-SEC-006 | Security | No shell-out, no `Command::new`, no `process::Command`, no FFI: the binary is a pure pcap analyzer with no subprocess surface. | Whole `src/` tree (verified via grep) | -- | Inherent to a forensic tool: no need to invoke external programs. | Implicit | None (negative property) |
| NFR-SEC-007 | Security | No network I/O: there is **zero** use of `std::net::TcpStream`, `UdpSocket`, `tokio::net`, `reqwest`, or any other socket API. The only `std::net` usage is the `IpAddr` value type for representing parsed packet IPs. | Whole `src/` tree (`std::net::IpAddr` is the only `std::net` symbol referenced) | -- | Pcap analysis is offline by design; eliminates network attack surface and exfiltration risk. | Implicit | None (negative property) |
| NFR-SEC-008 | Security | TLS record-payload header is validated against `MAX_RECORD_PAYLOAD = 18_432` before allocating; oversized records clear the buffer and increment `parse_errors` (DoS protection). | `src/analyzer/tls.rs:18, 587-597` | 18432 | TLS 1.2 ciphertext max per RFC 5246 (TLS 1.3 max is 16640; the larger value is a safe upper bound). | Runtime | `tests/tls_analyzer_tests.rs::test_oversized_sni_exceeds_record_payload_limit` |

### 2.3 Reliability (NFR-REL)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-REL-001 | Reliability | Release builds use `overflow-checks = true`: arithmetic overflow panics in release (not just debug). | `Cargo.toml:24-25` | -- | Defensive: catches integer-overflow bugs that would otherwise wrap silently and corrupt forensic counters. | Compile-time (codegen) | None directly |
| NFR-REL-002 | Reliability | Sequence-number arithmetic uses `wrapping_sub` to handle the TCP 32-bit sequence-space wraparound. | `src/reassembly/segment.rs:21-23, 50`; `src/reassembly/flow.rs:124` (`infer_isn`) | -- | TCP RFC 793; sequence numbers are mod-2^32 cyclic; subtraction must wrap. | Runtime | `tests/reassembly_segment_tests.rs::test_sequence_wraparound` |
| NFR-REL-003 | Reliability | Saturating arithmetic used in 13 sites to prevent overflow panics on adversarial inputs: window check, depth check, request buffer, response buffer, TLS buffer, error counters, FIN counter. | `src/reassembly/segment.rs:53,54,69,83`; `src/reassembly/mod.rs:229`; `src/reassembly/flow.rs:228`; `src/analyzer/http.rs:341,399,445,457`; `src/analyzer/tls.rs:677,684` | -- | Counter / buffer arithmetic on `u32`/`usize` with attacker-controlled inputs; saturate rather than panic in release (which would otherwise via NFR-REL-001). | Compile-time + runtime | Stress-test paths in `tests/reassembly_*` |
| NFR-REL-004 | Reliability | `TcpReassembler::new` defensively asserts that all five config bounds are > 0; panics on invalid config at construction. | `src/reassembly/mod.rs:86-96` | 0 (lower bound) | Fail-fast on misconfiguration -- 0 would mean "drop everything", a silent misbehavior. | Runtime (assert) | None directly (BC-RAS-001 notes no explicit test) |
| NFR-REL-005 | Reliability | `TcpReassembler::finalize` is idempotent (guarded by `self.finalized: bool`); second call is a no-op. | `src/reassembly/mod.rs:81, 384-389` | -- | Prevents double-flushing flows or duplicating summary findings if callers (e.g. test harnesses) call finalize twice. | Runtime | `tests/reassembly_engine_tests.rs::test_finalize_flushes_remaining` exercises one call; no explicit double-call test |
| NFR-REL-006 | Reliability | `close_flow` on a missing key emits ONE process-wide warning via `CLOSE_FLOW_MISSING_WARNED: AtomicBool`; subsequent missing-key closes are silent. | `src/reassembly/mod.rs:20, 480-489` | -- | Avoid spamming stderr if a structural invariant breaks; the `debug_assert!` already fires in debug builds. | Compile-time + runtime atomic | None |
| NFR-REL-007 | Reliability | `insert_segment` called with no ISN set emits ONE process-wide warning via `ISN_MISSING_WARNED: AtomicBool` and returns `InsertResult::IsnMissing`; no panic. | `src/reassembly/segment.rs:5, 40-48` | -- | Programming error category; visible once for diagnosis without breaking the analysis run. | Compile-time + runtime atomic | `tests/reassembly_segment_tests.rs::test_isn_missing_returns_isn_missing` |
| NFR-REL-008 | Reliability | Errors propagate via `anyhow::Result` from `PcapSource::from_file`, `from_pcap_reader`, `decode_packet`, `Cli::parse`, with context strings. No `panic!` paths on bad input. | `src/reader.rs:21-22, 41`; `src/decoder.rs:76-98`; `src/main.rs:255` | -- | Forensic tool must keep running through malformed input -- a single bad packet must not abort the analysis. | Runtime | `tests/decoder_tests.rs::test_decode_invalid_packet`; `tests/reader_tests.rs::test_unsupported_link_type_rejected` |
| NFR-REL-009 | Reliability | Decode errors in the per-packet loop are counted into `Summary.skipped_packets`; the FIRST error message is printed to stderr, subsequent errors are counted silently. | `src/main.rs:124-139, 204-211` | 1 (first error only) | Avoid stderr flooding on pcaps with widespread corruption; the counter is the persistent signal. | Runtime | `tests/summary_tests.rs` indirectly via `skipped_packets`; no test directly verifies the suppression |
| NFR-REL-010 | Reliability | TLS oversized record clears the per-direction buffer rather than aborting the analyzer or crashing; analysis continues with subsequent records. | `src/analyzer/tls.rs:587-597` | -- | Resilience: one malformed record on a stream must not kill the analyzer. | Runtime | `tests/tls_analyzer_tests.rs::test_oversized_sni_exceeds_record_payload_limit` |
| NFR-REL-011 | Reliability | HTTP per-direction "poison" mechanism stops processing after `POISON_THRESHOLD=3` consecutive parse errors; per-direction, per-flow, NOT global; cleared on `on_flow_close`. | `src/analyzer/http.rs:67, 342-348, 400-406` | 3 | Comment at `http.rs:64-66`: "Set > 1 to tolerate mid-stream joins where the first segment(s) are body data from a transfer that started before the capture." | Runtime | `tests/http_analyzer_tests.rs::test_parse_error_poisons_direction_after_threshold`, `test_single_error_does_not_poison`, `test_poison_request_does_not_affect_response`, `test_poison_cleared_after_flow_close`, `test_cross_flow_isolation_poisoning` (BC-HTTP-015..019, 021) |

### 2.4 Observability (NFR-OBS)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-OBS-001 | Observability | `ReassemblyStats` exposes 16 counters (packets, flows, segments, bytes, evictions); surfaced via `summarize()` into `AnalysisSummary.detail`. | `src/reassembly/mod.rs:54-72, 436-472` | 16 fields | ADR 0002 §"Error tracking": "Not logged to stderr -- the counter is the signal." | Runtime | `tests/reassembly_engine_tests.rs::test_summarize_returns_reassembly_stats` |
| NFR-OBS-002 | Observability | Per-analyzer `summarize()` returns `AnalysisSummary{analyzer_name, packets_analyzed, detail: HashMap<String, serde_json::Value>}` -- a uniform output shape every reporter can consume. | `src/analyzer/mod.rs:12-17`; `http.rs:482-530`, `tls.rs:708-745`, `dns.rs:64-80` | -- | ADR 0002 §"AnalysisSummary Format". | Compile-time (trait) | `tests/http_analyzer_tests.rs::test_summarize_produces_complete_output`, `tests/tls_analyzer_tests.rs::test_summarize_output`, `tests/analyzer_tests.rs::test_dns_analyzer_counts_queries` |
| NFR-OBS-003 | Observability | Parse errors are counted (never silently swallowed): `HttpAnalyzer.parse_errors`, `TlsAnalyzer.parse_errors`, `DnsAnalyzer` (implicit by query/response split). | `src/analyzer/http.rs:110, 339, 396`; `src/analyzer/tls.rs:279, 354, 510, 589, 650, 653` | -- | ADR 0002 §"Error tracking": "The counter is the signal." | Runtime | `tests/http_analyzer_tests.rs::test_parse_error_increments_counter`; `tests/tls_analyzer_tests.rs::test_parse_error_counter` |
| NFR-OBS-004 | Observability | Findings carry MITRE ATT&CK technique IDs where a clean mapping exists; `mitre_technique: Option<String>` -- `None` is preferred to a forced fit. | `src/findings.rs:66`; ADR 0002 §"Finding Generation Guidelines" | -- | ADR 0002 explicit guideline; `None` is meaningful (Uncategorized bucket in MITRE-grouped output). | Compile-time (Optional) | `tests/mitre_tests.rs::known_emitted_technique_ids_resolve_in_lookup`; finding-emitting analyzer tests |
| NFR-OBS-005 | Observability | The reassembler reports a `unclassified_flows` counter (flows the dispatcher could not route) by injecting it into the reassembly `AnalysisSummary.detail` map. | `src/dispatcher.rs:32-34, 99-115`; `src/main.rs:153-159` | -- | Lets the operator see how many flows fell through both content-detection AND port fallback -- a signal of unusual traffic. | Runtime | `tests/dispatcher_tests.rs::test_unclassified_flows_counter`, `test_classified_flow_not_counted_as_unclassified` |
| NFR-OBS-006 | Observability | One-shot one-line eprintln stderr warnings for: missing-ISN insert (`ISN_MISSING_WARNED`), close-flow missing key (`CLOSE_FLOW_MISSING_WARNED`), first decode error per run, `--no-reassemble` conflict with `--http/--tls`. | `src/reassembly/segment.rs:42-46`; `src/reassembly/mod.rs:482-487`; `src/main.rs:73-76, 124-129, 204-209` | -- | Lightweight signal for operators without adopting a full logging framework. | Runtime | None directly |
| NFR-OBS-007 | Observability | Per-target progress bar via `indicatif::ProgressBar` written to stderr with template `[elapsed] {bar:40} pos/len packets`. | `src/main.rs:107-110` | 40 (bar width) | UX: long pcaps should show progress; stderr keeps stdout clean for `--output-format json` piping. | Runtime | None (BC-CLI-013 is LOW conf) |
| NFR-OBS-008 | Observability | `--mitre` flag groups findings under tactic headers (`## Tactic Name`) in canonical kill-chain order, with an `Uncategorized` bucket last for findings without a mapping. | `src/reporter/terminal.rs:208-258`; `src/mitre.rs:71-90` | 14 enterprise + 2 ICS-unique = 16 tactic headers | Operator-friendly grouping; `all_tactics_in_report_order` provides a stable iteration order. | Runtime (CLI-gated) | `tests/reporter_tests.rs::mitre_grouping_*` (BC-RPT-013..017) |
| NFR-OBS-009 | Observability | `Skipped: N packets (decode errors)` line is only emitted when `N > 0`; suppresses noise on clean captures. | `src/reporter/terminal.rs:72-82` | 0 | Cosmetic but consistent: zeros are silent. | Runtime | `tests/reporter_tests.rs::test_terminal_reporter_shows_skipped_when_nonzero`, `test_terminal_reporter_hides_skipped_when_zero` |

### 2.5 Resource Bounds (NFR-RES)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value (rationale) | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-RES-001 | Resource bounds | Max engine-level findings across all flows: `MAX_FINDINGS = 10_000`; further per-flow findings are silently dropped. The `finalize()` summary-level segment-limit finding is pushed unconditionally to dodge the cap. | `src/reassembly/mod.rs:18, 272, 291, 310, 396-417, 534, 550` | 10_000 | ADR 0002 §"Finding Generation Guidelines": "Cap findings with `MAX_FINDINGS` to prevent memory exhaustion on adversarial input." Comment at `mod.rs:395-397` documents finalize-bypass. | Runtime (guard) | None directly (BC-RAS-024 is MEDIUM conf) |
| NFR-RES-002 | Resource bounds | Per-flow-direction overlap-anomaly alert fires when `overlap_count > OVERLAP_ALERT_THRESHOLD = 50` and exactly ONCE (sticky `overlap_alert_fired`). | `src/reassembly/mod.rs:15, 270-288` | 50 | Empirical / inferred (no explicit comment); intended to fire only on attacker-driven overlap floods, not naturally retransmit-heavy flows. | Runtime | `tests/reassembly_engine_tests.rs::test_overlap_anomaly_finding` |
| NFR-RES-003 | Resource bounds | Per-flow-direction small-segment alert fires when `small_segment_count > SMALL_SEGMENT_ALERT_THRESHOLD = 2048` and exactly once. | `src/reassembly/mod.rs:16, 289-307`; `src/reassembly/segment.rs:64-66` | 2048 | "Small" defined as `data.len() < 8` bytes; 2048 small segments in one direction is a strong IDS-evasion signal. Inferred -- no comment. | Runtime | None directly (BC-RAS-020 MEDIUM) |
| NFR-RES-004 | Resource bounds | Per-flow-direction out-of-window alert fires when `out_of_window_count > OUT_OF_WINDOW_ALERT_THRESHOLD = 100` and exactly once. | `src/reassembly/mod.rs:17, 308-331` | 100 | Inferred -- no comment. A small number of out-of-window segments is normal noise; ~100 is "something is wrong" (misconfig, evasion, capture corruption). | Runtime | `tests/reassembly_engine_tests.rs::test_out_of_window_threshold_alert`, `test_out_of_window_alert_fires_only_once` |
| NFR-RES-005 | Resource bounds | Default per-direction reassembly depth: `max_depth = 10 * 1024 * 1024` (10 MB). CLI: `--reassembly-depth=<MB>`. | `src/reassembly/mod.rs:43`; `src/cli.rs:46-48`; `src/main.rs:80` | 10 MB | Comment: "10 MB per direction". Balances memory vs. forensic depth on multi-GB captures. | Runtime (config) | `tests/reassembly_segment_tests.rs::test_depth_limit_truncation` (uses smaller value); `tests/cli_tests.rs::test_reassembly_flags` |
| NFR-RES-006 | Resource bounds | Default global reassembly memcap: `memcap = 1024 * 1024 * 1024` (1 GB). CLI: `--reassembly-memcap=<MB>`, default 1024. | `src/reassembly/mod.rs:44`; `src/cli.rs:50-52` | 1 GB | Comment: "1 GB total". Sized for a developer workstation; eviction triggers above this. | Runtime (config) | `tests/reassembly_engine_tests.rs::test_memcap_eviction` (small value); `tests/cli_tests.rs::test_reassembly_flags` |
| NFR-RES-007 | Resource bounds | Default flow timeout: `flow_timeout_secs = 300` (5 min). | `src/reassembly/mod.rs:45` | 300 sec | Comment: "5 minutes". Matches typical TCP idle timeouts. Inferred -- no further justification. | Runtime (config) | `tests/reassembly_engine_tests.rs::test_flow_timeout_expiration` |
| NFR-RES-008 | Resource bounds | Default max concurrent tracked flows: `max_flows = 100_000`. | `src/reassembly/mod.rs:46` | 100_000 | Comment: "100K concurrent flows. Prevents flow table flooding." Inferred sizing. | Runtime (config) | `tests/reassembly_engine_tests.rs::test_max_flows_eviction` (small value used) |
| NFR-RES-009 | Resource bounds | Default max segments per direction: `max_segments_per_direction = 10_000`. | `src/reassembly/mod.rs:47` | 10_000 | Comment: "10K segments per direction. Prevents BTreeMap overhead explosion." | Runtime (config) | `tests/reassembly_engine_tests.rs::test_max_segments_per_direction` (uses 5); `tests/reassembly_segment_tests.rs::test_segment_limit_*` |
| NFR-RES-010 | Resource bounds | Default max receive window: `max_receive_window = 1_048_576` (1 MB). | `src/reassembly/mod.rs:48` | 1 MB | Comment at `mod.rs:35-37`: "Default 1MB matches Suricata/Zeek/Snort." | Runtime (config) | `tests/reassembly_engine_tests.rs::test_out_of_window_segment_rejected_by_engine`, `test_out_of_window_threshold_alert` (use 1000) |
| NFR-RES-011 | Resource bounds | HTTP per-direction header buffer cap: `MAX_HEADER_BUF = 65_536` (64 KB). Bytes past the cap are dropped silently; if completion arrives later, parsing does NOT proceed (the truncated buffer never satisfies httparse). | `src/analyzer/http.rs:8, 445-462` | 65536 | Inferred -- no comment, but documented at ADR 0001 §"Broadcast to All Analyzers" alternative ("HTTP already buffers up to 64KB per flow direction"). | Runtime | `tests/http_analyzer_tests.rs::test_buffer_cap_no_panic_on_oversized_headers` (BC-HTTP-022) |
| NFR-RES-012 | Resource bounds | HTTP max headers per request/response: `MAX_HEADERS = 96`. Exceeding emits Anomaly/Inconclusive/Medium finding with MITRE T1499.002. | `src/analyzer/http.rs:9, 23, 45, 350-360, 408-418` | 96 | Inferred -- no comment. 96 covers realistic legitimate HTTP traffic with margin; httparse needs a fixed array. | Compile-time (array size) + runtime | `tests/http_analyzer_tests.rs::test_too_many_headers_generates_finding`, `test_too_many_headers_in_response_generates_finding` |
| NFR-RES-013 | Resource bounds | HTTP URI list cap: `MAX_URIS = 10_000`; further URIs silently dropped. | `src/analyzer/http.rs:10, 325-327` | 10_000 | Inferred -- no comment. Sample-cap to prevent unbounded growth on long captures. | Runtime | None directly (BC-HTTP-025 MEDIUM) |
| NFR-RES-014 | Resource bounds | HTTP/TLS per-map cardinality cap: `MAX_MAP_ENTRIES = 50_000` applies to `methods`, `hosts`, `user_agents`, `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`. New keys past the cap are silently dropped; existing keys still increment. | `src/analyzer/http.rs:11, 309-323`; `src/analyzer/tls.rs:15, 332-336, 347, 376, 451, 504, 519, 523` | 50_000 | ADR 0002 §"Aggregate counters": "Bounded by `MAX_MAP_ENTRIES` to prevent memory exhaustion from cardinality explosion." | Runtime | `tests/tls_analyzer_tests.rs::test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` (uses literal `50_000`) |
| NFR-RES-015 | Resource bounds | TLS per-direction record-assembly buffer cap: `MAX_BUF = 65_536` (64 KB). | `src/analyzer/tls.rs:14, 677, 684` | 65536 | Inferred -- no comment. Matches HTTP buffer size. | Runtime | None directly (BC-TLS-005 MEDIUM) |
| NFR-RES-016 | Resource bounds | TLS record payload sanity cap: `MAX_RECORD_PAYLOAD = 18_432`; oversized records are rejected and parse_errors incremented. | `src/analyzer/tls.rs:16-18, 587-597` | 18432 | Comment: "TLS 1.2 ciphertext max per RFC 5246. TLS 1.3 max is 16,640 but we use the larger value as a safe upper bound." | Runtime | `tests/tls_analyzer_tests.rs::test_oversized_sni_exceeds_record_payload_limit` |
| NFR-RES-017 | Resource bounds | HTTP poison threshold: `POISON_THRESHOLD = 3`; after 3 consecutive parse errors in a direction, subsequent bytes are skipped and tracked via `poisoned_bytes_skipped`. | `src/analyzer/http.rs:64-67, 342-348, 400-406` | 3 | Comment: "Set > 1 to tolerate mid-stream joins where the first segment(s) are body data from a transfer that started before the capture." | Runtime | `tests/http_analyzer_tests.rs::test_parse_error_poisons_direction_after_threshold`, `test_single_error_does_not_poison` |
| NFR-RES-018 | Resource bounds | HTTP URI display-truncation cap (data layer): 120 chars for findings summaries, 200 chars for "long URI" evidence. | `src/analyzer/http.rs:183, 211, 227, 271`; `src/analyzer/http.rs:93-99` (`truncate_uri`) | 120, 200 | ADR 0003 §"Other formatting concerns" flags this as a layering wart -- truncation belongs at display, not at construction; left as-is per YAGNI. | Runtime | `tests/http_analyzer_tests.rs::test_detect_long_uri` |
| NFR-RES-019 | Resource bounds | HTTP "abnormally long URI" finding threshold: URIs longer than 2048 chars emit Execution/Likely/Medium finding. | `src/analyzer/http.rs:265-276` | 2048 | Inferred -- no comment. 2048 is a common reference for "too long" URI heuristics (e.g., IIS default). | Runtime | `tests/http_analyzer_tests.rs::test_detect_long_uri` (BC-HTTP-010) |
| NFR-RES-020 | Resource bounds | TLS / HTTP `summarize()` returns only the top 20 hosts and top 20 SNIs / recent URIs by count -- bounded JSON output regardless of cardinality cap. | `src/analyzer/http.rs:502, 505`; `src/analyzer/tls.rs:712-714` | 20 | Inferred -- no comment. Operator-friendly truncation for terminal display. | Runtime | `tests/http_analyzer_tests.rs::test_summarize_produces_complete_output` (BC-HTTP-023), `tests/tls_analyzer_tests.rs::test_summarize_output` (BC-TLS-031) |
| NFR-RES-021 | Resource bounds | TLS handshake-only short-circuit: once both `client_hello_seen` and `server_hello_seen` are true (`TlsFlowState::done()`), subsequent `on_data` short-circuits and stops buffering -- bounds work per flow regardless of stream length. | `src/analyzer/tls.rs:262-266, 665-668` | -- | Performance + memory bound; an analyzer that only cares about the handshake doesn't need to keep parsing application data. | Runtime | `tests/tls_analyzer_tests.rs::test_stop_after_handshake` (BC-TLS-003, 034) |

### 2.6 Maintainability (NFR-MNT)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-MNT-001 | Maintainability | `RUSTFLAGS=-Dwarnings` in CI: any warning fails the build. Zero `#[allow(...)]` clusters anywhere in `src/`. | `.github/workflows/ci.yml:10-12, 58`; `find src -name '*.rs' -exec awk /#\\[allow/` returns 0 matches | -- | Enforces a clean baseline; warning suppression must be an explicit deliberate act. | CI gate | All CI runs |
| NFR-MNT-002 | Maintainability | `cargo clippy --all-targets -- -D warnings` is a separate CI job that must pass. | `.github/workflows/ci.yml:49-58` | -- | Catches lint regressions independently of `RUSTFLAGS`. | CI gate | All CI runs |
| NFR-MNT-003 | Maintainability | Rustfmt is enforced via `cargo fmt --all --check`; config pins edition 2024, `max_width = 100`, `use_field_init_shorthand`, `use_try_shorthand`. | `.github/workflows/ci.yml:60-68`; `rustfmt.toml:1-5` | 100 cols | Locks formatting policy; rejects unformatted PRs. | CI gate | All CI runs |
| NFR-MNT-004 | Maintainability | `MitreTactic` is `#[non_exhaustive]` so adding new variants (e.g. when MITRE adds a new tactic) is a non-breaking change for downstream crates. | `src/mitre.rs:21-22` | -- | Comment at `mitre.rs:17-20` cites ATT&CK v18's addition of Resource Development as the reason. | Compile-time | None directly (BC-MIT-009 LOW conf) |
| NFR-MNT-005 | Maintainability | All tests live under `tests/`; **zero** inline `#[test]` modules under `src/` except `src/reporter/terminal.rs:261` which contains a `#[cfg(test)] mod tests` block for the `escape_for_terminal` helper. | `find src -name '*.rs'` for `#[test]` | -- | Pass 0 §5 confirmed; project-wide convention. Internal helpers may be tested inline only when the helper is private (escape_for_terminal). | Implicit | n/a |
| NFR-MNT-006 | Maintainability | No per-file license header SPDX comments in any source file. License declared once at `Cargo.toml:6` (`MIT`) and in top-level `LICENSE`. | `Cargo.toml:6`; absence in every `src/*.rs` | -- | Project-wide convention (consistent across all 20 files). | Implicit | n/a |
| NFR-MNT-007 | Maintainability | Semantic-PR titles enforced via `amannn/action-semantic-pull-request@v6`; allowed types: feat/fix/docs/style/refactor/perf/test/build/ci/chore/revert. | `.github/workflows/ci.yml:14-38` | -- | Encodes git-flow + conventional-commit messaging policy in CI. | CI gate (PR only) | All PRs |
| NFR-MNT-008 | Maintainability | Domain rules implemented as boolean flag fields with sticky alert state (`overlap_alert_fired`, `small_segment_alert_fired`, `out_of_window_alert_fired`) rather than ad-hoc counters compared on each call site -- reduces drift. | `src/reassembly/flow.rs:79, 81, 83`; `src/reassembly/mod.rs:271, 290, 309` | -- | Pattern repeated 3x in the same module; tests pin the "fires only once" semantics. | Runtime | `tests/reassembly_engine_tests.rs::test_out_of_window_alert_fires_only_once` |
| NFR-MNT-009 | Maintainability | `technique_info` is the single source of truth for the `(name, tactic)` pair of each MITRE technique ID; `technique_name` and `technique_tactic` are thin projections. Makes it impossible to add one facet without the other. | `src/mitre.rs:92-145` | -- | Doc comment at `mitre.rs:93-96` explicit. | Compile-time | `tests/mitre_tests.rs::technique_tactic_matches_spec_table`, `technique_name_resolves_every_seeded_id` |
| NFR-MNT-010 | Maintainability | Three accepted Architecture Decision Records (`docs/adr/0001`..`0003`) covering content-first dispatch, modular analyzer pattern, and reporting pipeline layering. ADRs are referenced from doc comments to keep code grounded in rationale. | `docs/adr/000{1,2,3}-*.md`; doc-comments in `src/findings.rs:78`, `src/reporter/terminal.rs:27`, `src/analyzer/tls.rs:393-446` | 3 ADRs | Persistent design-rationale store. | Convention | n/a |
| NFR-MNT-011 | Maintainability | `floor_char_boundary` from std is used at `src/analyzer/http.rs:97`. This was stabilized in Rust 1.86 (2025-04-03); it is the load-bearing argument that the effective MSRV is 1.86+, not 1.85 (which would only satisfy edition 2024). | `src/analyzer/http.rs:97` | -- | Without it, multi-byte UTF-8 URIs would panic on substring boundary mismatch. Pass 0 inferred MSRV 1.85 from edition 2024 alone; in practice it is 1.86+. | Compile-time | Implicit -- `cargo build` fails on <1.86 |

### 2.7 Portability (NFR-PORT)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-PORT-001 | Portability | CI runs only on `ubuntu-latest`. There is no macOS / Windows / nightly job. | `.github/workflows/ci.yml:18, 42, 51, 62` | 1 platform | Inferred -- presumably to keep CI cost low. Practical effect: macOS/Windows regressions can land. | CI policy | None |
| NFR-PORT-002 | Portability | No platform-specific `cfg`. `find src -name '*.rs' -exec awk /#\[cfg.*target/` returns no matches. | All of `src/` | 0 platform branches | Code is portable across any libc that supports `pcap-file`, `etherparse`, etc. | Implicit | None |
| NFR-PORT-003 | Portability | No `build.rs` and no `rust-toolchain.toml`. Effective MSRV is "whatever `dtolnay/rust-toolchain@stable` resolves to today". | Repo root; `.github/workflows/ci.yml:45,54,65` | -- | Pass 0 §1 confirms; relies on edition 2024 (>=1.85) and `floor_char_boundary` (>=1.86). | Implicit | n/a |
| NFR-PORT-004 | Portability | The CLI parses argv only; the only env-var read at runtime is `NO_COLOR` (de-facto cross-platform "disable ANSI" convention) at `src/main.rs:25`. | `src/main.rs:25` | 1 env var | Conforms to `https://no-color.org/`. | Runtime | None (BC-CLI-010 MEDIUM) |
| NFR-PORT-005 | Portability | Edition 2024 (`Cargo.toml:4`); rustfmt.toml pins `edition = "2024"`. | `Cargo.toml:4`; `rustfmt.toml:1` | -- | Adopts let-chains, lifetime-capture, `if let` temp scoping, and other 2024 features (used heavily in `src/main.rs:245-248` `if let && (ext == "pcap" || ext == "pcapng")`). | Compile-time | CI |

### 2.8 Supply Chain (NFR-SUP)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-SUP-001 | Supply chain | 14 direct prod dependencies. Of those, 3 are declared but unused in current sources: `csv`, `rayon`, and (effectively) the `output_format` Csv arm of `clap`. | `Cargo.toml:8-22`; Pass 0 §2 | 14 (3 unused) | Unused deps inflate the transitive surface; should be removed or wired (Pass 0 questions #1, #2). | Manifest | n/a |
| NFR-SUP-002 | Supply chain | 3 dev-dependencies declared (`assert_cmd`, `predicates`, `tempfile`), all currently unused in `tests/`. | `Cargo.toml:27-30`; Pass 0 §2 | 3 (3 unused) | Pass 0 question #3; intended for binary-spawn integration tests that don't yet exist. | Manifest | n/a |
| NFR-SUP-003 | Supply chain | `Cargo.lock` is checked in (38,291 bytes); appropriate for a binary crate. | Repo root; Pass 0 §1 | -- | Reproducible builds across contributors / CI / `cargo install --path .`. | Convention | n/a |
| NFR-SUP-004 | Supply chain | Dependency version requirements are minor-version pins (e.g. `clap = "4"`, `pcap-file = "2"`, `tls-parser = "0.12"`, `etherparse = "0.16"`, `httparse = "1"`, `chrono = "0.4"`, `serde = "1"`); no exact-version pins, no git deps, no path deps. | `Cargo.toml:8-30` | -- | Standard SemVer caret behavior; `Cargo.lock` provides the exact resolution. | Manifest | n/a |
| NFR-SUP-005 | Supply chain | No `[build-dependencies]`, no `build.rs`, no procedural macros beyond what `clap`/`serde` derive bring transitively. | `Cargo.toml`; absence of `build.rs` | -- | Smaller compile-time attack surface; no custom build scripts that could be poisoned. | Manifest | n/a |

### 2.9 Input/Output Compatibility (NFR-COMPAT)

| ID | Category | NFR statement | Where encoded | Numeric value | Why this value | Enforcement | Tests pinning it |
|---|---|---|---|---|---|---|---|
| NFR-COMPAT-001 | Compatibility | Only classic pcap is accepted -- pcapng is rejected at header-parse time. 5 link types accepted: Ethernet (1), RAW (101), Linux SLL (113), IPv4 (228), IPv6 (229); all others rejected with anyhow error listing the supported set. | `src/reader.rs:22-36` (uses `pcap_file::pcap::PcapReader`); `src/decoder.rs:73-76` | 5 link types | README §"Supported Link Types" matches exactly. pcapng appears in roadmap. | Runtime | `tests/reader_tests.rs::test_unsupported_link_type_rejected` (BC-RDR-001, BC-RDR-004) |
| NFR-COMPAT-002 | Compatibility | Output formats: terminal (default) and JSON (`--output-format json`). CSV is declared via `OutputFormat::Csv` but the dispatch arm at `src/main.rs:172-184` only matches `Some(OutputFormat::Json)` and falls through to `TerminalReporter` for anything else. | `src/cli.rs:5-9, 31-36`; `src/main.rs:172-184, 218-230`; `Cargo.toml:17` (csv dep) | 2 implemented of 2 advertised + 1 declared-unused | Pass 3 BC-ABS-007 and Pass 0 question #1 confirm; README "Multiple outputs" only lists "colored terminal, JSON export". | Runtime (limited) | None directly (BC-CLI-016 MEDIUM) |

---

## 3. Numeric-Value Index

Every magic number / default in source order. Source-of-value: `ADR`, `comment`, `inferred` (no documented rationale), `RFC` (cites an RFC), or `industry` (matches Suricata/Zeek/Snort default).

| Name | Value | Location (file:line) | NFR-ID | Source of value |
|---|---|---|---|---|
| `OVERLAP_ALERT_THRESHOLD` | 50 | `src/reassembly/mod.rs:15` | NFR-RES-002 | inferred |
| `SMALL_SEGMENT_ALERT_THRESHOLD` | 2048 | `src/reassembly/mod.rs:16` | NFR-RES-003 | inferred |
| `OUT_OF_WINDOW_ALERT_THRESHOLD` | 100 | `src/reassembly/mod.rs:17` | NFR-RES-004 | inferred |
| `MAX_FINDINGS` | 10_000 | `src/reassembly/mod.rs:18` | NFR-RES-001 | ADR 0002 |
| `ReassemblyConfig::default().max_depth` | 10 MB (10*1024*1024) | `src/reassembly/mod.rs:43` | NFR-RES-005 | comment ("10 MB per direction") |
| `ReassemblyConfig::default().memcap` | 1 GB (1024^3) | `src/reassembly/mod.rs:44` | NFR-RES-006 | comment ("1 GB total") |
| `ReassemblyConfig::default().flow_timeout_secs` | 300 | `src/reassembly/mod.rs:45` | NFR-RES-007 | comment ("5 minutes") |
| `ReassemblyConfig::default().max_flows` | 100_000 | `src/reassembly/mod.rs:46` | NFR-RES-008 | comment ("Prevents flow table flooding") |
| `ReassemblyConfig::default().max_segments_per_direction` | 10_000 | `src/reassembly/mod.rs:47` | NFR-RES-009 | comment ("Prevents BTreeMap overhead explosion") |
| `ReassemblyConfig::default().max_receive_window` | 1 MB (1_048_576) | `src/reassembly/mod.rs:48` | NFR-RES-010 | industry ("matches Suricata/Zeek/Snort") |
| `Cli.reassembly_depth` default | 10 (MB) | `src/cli.rs:47-48` | NFR-RES-005 | matches NFR-RES-005 |
| `Cli.reassembly_memcap` default | 1024 (MB) | `src/cli.rs:51-52` | NFR-RES-006 | matches NFR-RES-006 |
| `MAX_HEADER_BUF` (HTTP) | 65_536 | `src/analyzer/http.rs:8` | NFR-RES-011 | inferred (ADR 0001 mentions "64KB per flow direction" descriptively) |
| `MAX_HEADERS` (HTTP) | 96 | `src/analyzer/http.rs:9` | NFR-RES-012 | inferred |
| `MAX_URIS` (HTTP) | 10_000 | `src/analyzer/http.rs:10` | NFR-RES-013 | inferred |
| `MAX_MAP_ENTRIES` (HTTP) | 50_000 | `src/analyzer/http.rs:11` | NFR-RES-014 | ADR 0002 ("Bounded by `MAX_MAP_ENTRIES`") |
| `POISON_THRESHOLD` (HTTP) | 3 | `src/analyzer/http.rs:67` | NFR-RES-017 / NFR-REL-011 | comment (tolerates mid-stream joins) |
| `truncate_uri` summary cap | 120 chars | `src/analyzer/http.rs:183, 211, 227` | NFR-RES-018 | inferred (ADR 0003 flags layering wart) |
| `truncate_uri` evidence cap | 200 chars | `src/analyzer/http.rs:271` | NFR-RES-018 | inferred |
| "Abnormally long URI" threshold | 2048 chars | `src/analyzer/http.rs:265` | NFR-RES-019 | inferred |
| `MAX_BUF` (TLS) | 65_536 | `src/analyzer/tls.rs:14` | NFR-RES-015 | inferred |
| `MAX_MAP_ENTRIES` (TLS) | 50_000 | `src/analyzer/tls.rs:15` | NFR-RES-014 | ADR 0002 |
| `MAX_RECORD_PAYLOAD` (TLS) | 18_432 | `src/analyzer/tls.rs:18` | NFR-RES-016 / NFR-SEC-008 | comment ("TLS 1.2 ciphertext max per RFC 5246; TLS 1.3 max is 16,640") |
| `is_grease_u16` GREASE mask | `(val & 0x0F0F) == 0x0A0A` | `src/analyzer/tls.rs:23-24` | NFR-OBS-003 (parse hygiene) | RFC 8701 |
| Deprecated-SSL version threshold | `<= 0x0300` (SSL 2.0 / 3.0) | `src/analyzer/tls.rs:476, 539` | NFR-SEC-005 (adjacent) | RFC 7568 |
| C0 control range (SNI / terminal escape) | 0x00..=0x1F + 0x7F (DEL) | `src/analyzer/tls.rs:210`; `src/reporter/terminal.rs:37` | NFR-SEC-002, NFR-SEC-005 | RFC 6066 §3 + ADR 0003 |
| C1 control range (terminal escape) | U+0080..=U+009F | `src/reporter/terminal.rs:37` | NFR-SEC-002 | ADR 0003 §"Why C1?" |
| `summarize()` top-N truncation (hosts/snis/uris) | 20 | `src/analyzer/http.rs:502, 505`; `src/analyzer/tls.rs:714` | NFR-RES-020 | inferred |
| ProgressBar template width | 40 | `src/main.rs:109` | NFR-OBS-007 | inferred |
| Supported pcap link-type set | {1, 101, 113, 228, 229} | `src/reader.rs:26-30` | NFR-COMPAT-001 | matches README §"Supported Link Types" |
| Dispatcher TLS port-fallback set | {443, 8443} | `src/dispatcher.rs:57` | NFR-OBS-005 / NFR-COMPAT (adjacent) | ADR 0001 |
| Dispatcher HTTP port-fallback set | {80, 8080} | `src/dispatcher.rs:60` | NFR-OBS-005 | ADR 0001 |
| HTTP method signature set | {GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH, CONNECT, TRACE, HTTP/} | `src/dispatcher.rs:42-52` | NFR-OBS-005 | ADR 0001 |
| TLS content signature | first byte 0x16, second 0x03, len>=5 | `src/dispatcher.rs:39` | NFR-OBS-005 | ADR 0001 |
| `app_protocol_hint` port→service table | {53:DNS, 80:HTTP, 443:TLS, 22:SSH, 445:SMB, 502:Modbus, 20000:DNP3} | `src/decoder.rs:58-66` | NFR-OBS (telemetry) | industry well-known ports |
| Pcap timestamp split width | u32 secs + u32 usecs | `src/reader.rs:43-44` | NFR-COMPAT-001 | inferred (Y2106 wraparound noted in Pass 3 BC-RDR-005) |
| DNS query/response QR-bit position | byte 2 bit 7 (0x80) | `src/analyzer/dns.rs:30-36` | NFR-OBS-003 | RFC 1035 |
| Small-segment threshold (segment.rs) | `data.len() < 8` | `src/reassembly/segment.rs:64` | NFR-RES-003 | inferred |

28 distinct numeric / set constants; several constants appear in more than one row (e.g. `MAX_MAP_ENTRIES` shared by HTTP & TLS) but are counted once.

---

## 4. CLI defaults & policy constants

| Setting | Value / behavior | Where | NFR ref |
|---|---|---|---|
| `--reassembly-depth` default | 10 (MB) | `src/cli.rs:47-48` | NFR-RES-005 |
| `--reassembly-memcap` default | 1024 (MB) | `src/cli.rs:51-52` | NFR-RES-006 |
| `--reassemble` vs `--no-reassemble` | `conflicts_with` | `src/cli.rs:39` | NFR-REL (clap validation) |
| `--no-reassemble` warn-once interaction with `--http/--tls` | one stderr eprintln, analysis continues | `src/main.rs:72-76` | NFR-OBS-006 |
| `NO_COLOR` env var | disables color | `src/main.rs:25` | NFR-PORT-004 |
| `--no-color` flag | disables color | `src/cli.rs:23-24` | NFR-PORT-004 |
| `--output-format` accepted values | `json`, `csv` (csv unwired -> falls through to terminal) | `src/cli.rs:5-9`; `src/main.rs:172-184` | NFR-COMPAT-002 |
| Multi-target globbing | directory expanded to `*.pcap` + `*.pcapng`, sorted | `src/main.rs:236-253` | NFR-COMPAT-001 (note: `*.pcapng` is collected even though reader rejects it -- §7 NFR-VIO-002) |
| `--threats`, `--beacon`, `--filter`, `--verbose`, `--hosts`, `--services`, `--json <FILE>`, `--csv <FILE>` | parsed but unwired (no runtime behavior) | `src/cli.rs:19,67,82,94,106,110, 31, 35` | NFR-VIO-003 (see §7) |

---

## 5. Cross-Cutting NFR Observations

### 5.1 Panic-freedom audit

Grep of `src/` for `panic!`, `.unwrap()`, `.expect(`, `unreachable!`, `todo!`, `unimplemented!`:

| Site | Verdict | Reason |
|---|---|---|
| `src/reporter/json.rs:36` -- `serde_json::to_string_pretty(&output).unwrap()` | Could panic on malformed input -- **risk LOW**. `output` is a `serde_json::Value` constructed via `json!{...}` from `Summary`, `Vec<Finding>`, and `Vec<AnalysisSummary>` -- all of which are derive-Serialize types with no failing serialize paths. `to_string_pretty` only fails on serializer I/O errors or on `Serialize` impls that fail; neither applies here. In practice infallible by construction. Still, this is a paper-cut: would be cleaner as `.expect("Finding/Summary serialize is infallible")` or `?` with the outer signature returning `Result<String, ...>`. Pass 3 BC-RPT-001 noted this. | infallible by construction |
| `src/reassembly/mod.rs:157` -- `self.flows.get_mut(&key).unwrap()` (after `contains_key`/`insert`) | The flow was inserted on the immediately preceding line (or already present). | infallible by construction |
| `src/reassembly/mod.rs:268` -- `self.flows.get_mut(&key).unwrap()` (after the dispatch and overlap counter handling) | Same flow; not removed by `process_packet` until step 10. | infallible by construction |
| `src/reassembly/mod.rs:334` -- `self.flows.get_mut(&key).unwrap()` (after the alert blocks) | Same. | infallible by construction |
| `debug_assert!` / `debug_assert_eq!` at 5 sites: `src/reassembly/mod.rs:223, 481, 1890`; `src/reassembly/flow.rs:150`; `src/reassembly/segment.rs:178, 209`; `src/analyzer/tls.rs:205-209` | Strip in release; defensive invariant checks. NFR-REL-001 (`overflow-checks=true`) preserves arithmetic-overflow panics in release but does NOT preserve `debug_assert!`. | test-only path |
| `assert!` at 5 sites: `src/reassembly/mod.rs:86-96` (constructor preconditions) | Defensive fail-fast in `TcpReassembler::new`. Only fires on programmer error (calling with config<=0). | test-only path (in practice) |
| Inline `assert_eq!` in `src/reporter/terminal.rs:265-348` | All inside `#[cfg(test)] mod tests`. | test-only path |

**Audit verdict:** the codebase is effectively panic-free under non-adversarial inputs. The 4 `.unwrap()`s in `src/` are all paired with immediate prior insert/contains-check or trivial Serialize-infallibility -- HIGH confidence of no panics on malformed pcap input. The `assert!`s in `TcpReassembler::new` are programmer-error-only (`max_depth > 0` etc.) and would only fire if a future contributor wired the CLI flag in a way that allowed 0 -- worth a regression test (recommendation R6 below).

### 5.2 No-`unsafe` audit

`find src -name '*.rs' -exec awk /unsafe/` returns **zero** matches. The codebase contains no `unsafe` blocks, no `unsafe fn`, no `unsafe impl`. Pass 1 §10 already noted this; this audit confirms. NFR-SEC by construction.

### 5.3 No-network / no-fs-write side-effect audit

Searching `src/` for `std::net::`, `tokio::net`, `reqwest`, `TcpStream`, `UdpSocket`, `File::create`, `fs::write`, `OpenOptions`:

- **Only** `std::net::IpAddr` (the value type) is used. No socket creation, no listener, no connector. NFR-SEC-007 confirmed.
- **Only** `std::fs::File::open` (read-only) at `src/reader.rs:53` and `std::fs::read_dir` at `src/main.rs:242`. No write side. The product writes ONLY to stdout (via `println!`) and stderr (via `eprintln!` and `indicatif::ProgressBar`). NFR-VIO-004 (§7) tracks the gap between CLI `--json <FILE>` / `--csv <FILE>` advertising file output and the actual stdout-only implementation.

### 5.4 No-env-var-config audit

`find src -name '*.rs' -exec awk /std::env|env::var/` returns **one** match: `src/main.rs:25` -- `std::env::var("NO_COLOR").is_err()`. This is the only env var consulted at runtime. The `RUSTFLAGS` mentioned in `.github/workflows/ci.yml:12` is a build-time gate, not a runtime config. **NFR-PORT-004 confirmed:** wirerust has effectively no env-var configuration surface.

### 5.5 Time policy

`find src -name '*.rs' -exec awk /SystemTime|Instant::now|chrono::Utc::now|DateTime/` returns:

- `src/findings.rs:4` -- `use chrono::{DateTime, Utc};`
- `src/findings.rs:69` -- `pub timestamp: Option<DateTime<Utc>>` on `Finding`.

There is **zero** call to `Utc::now()`, `SystemTime::now()`, or `Instant::now()` anywhere in `src/`. Every `Finding` in current emission sites pushes `timestamp: None` (Pass 2 observation). Pcap-derived timestamps (`raw.timestamp_secs`) are stored separately on `RawPacket` and consumed by reassembly as a `u32` (seconds-since-epoch); they never flow into `Finding.timestamp`. **NFR-OBS verdict:** timestamps in findings are not yet populated; chrono is wired but only used as a value type. The Y2106 caveat already noted in Pass 3 BC-RDR-005 is a latent issue.

### 5.6 MSRV / edition enforcement

- `Cargo.toml:4` -- `edition = "2024"` -> requires rustc >= 1.85.
- `rustfmt.toml:1` -- `edition = "2024"` matches.
- CI uses `dtolnay/rust-toolchain@stable` -- whatever stable resolves to today.
- Actual MSRV is **1.86+** because of `floor_char_boundary` at `src/analyzer/http.rs:97` (stabilized Rust 1.86, 2025-04-03). This is NFR-MNT-011 -- a hidden dependency the manifest doesn't document. A user with Rust 1.85 would `cargo build` and get a compile error pointing at `http.rs:97`.

### 5.7 Lockfile policy

- `Cargo.lock` is checked in (Pass 0 §7). Appropriate for a binary crate.
- No `--locked` flag in CI; `cargo test --all-targets` uses whatever resolution the lockfile produces.
- No `cargo audit` / `cargo deny` job. Supply-chain auditing is not automated.

---

## 6. Implicit-vs-Explicit NFR Comparison

For each NFR in §2, whether it is documented somewhere (ADR / doc-comment / test name) or only inferable from code:

| NFR ID | Explicit (ADR / comment / test name) | Implicit (only inferable from code) |
|---|---|---|
| NFR-PERF-001 zero-copy | README L17 ("zero-copy packet parsing") | -- |
| NFR-PERF-002 eager-not-streaming | -- (README L9 "one-pass triage" is ambiguous) | code at `reader.rs:38-48` |
| NFR-PERF-003 content-first cache | ADR 0001 §Rationale | -- |
| NFR-PERF-004 SIMD slice compare | comment at `segment.rs:113` | -- |
| NFR-SEC-001 raw-data layer | ADR 0003; doc comment on `Finding::Display` | -- |
| NFR-SEC-002 terminal escape | ADR 0003; doc comment at `terminal.rs:9-28` | -- |
| NFR-SEC-003 analyzer-summary escape | ADR 0003; comment at `terminal.rs:125-132` | -- |
| NFR-SEC-004 JSON RFC 8259 | ADR 0003 §"Where escaping does NOT happen"; test names | -- |
| NFR-SEC-005 SNI control flagging | doc comment at `tls.rs:155-172` | -- |
| NFR-SEC-006 no shell-out | -- | code-only |
| NFR-SEC-007 no network | README L17 ("multi-GB captures"; doesn't say no-net) | mostly inferable |
| NFR-SEC-008 TLS record cap | comment at `tls.rs:16-18` (RFC 5246/8446) | -- |
| NFR-REL-001 overflow-checks | -- | `Cargo.toml:24` is explicit but not commented |
| NFR-REL-002 wrapping_sub | test name `test_sequence_wraparound`; ADR-free | code-only otherwise |
| NFR-REL-003 saturating arithmetic | -- | code-only (13 sites) |
| NFR-REL-004 ctor asserts | -- | code-only |
| NFR-REL-005 finalize idempotent | doc comment at `mod.rs:383-384` | -- |
| NFR-REL-006 close_flow warn-once | -- | code-only |
| NFR-REL-007 ISN missing warn-once | -- | code-only |
| NFR-REL-008 anyhow propagation | -- | inferable from signatures |
| NFR-REL-009 first-error-only print | comment at `main.rs:127-128` ("Further errors counted silently") | -- |
| NFR-REL-010 TLS bad record resilience | -- | code-only |
| NFR-REL-011 HTTP poison | comment at `http.rs:64-66` | -- |
| NFR-OBS-001..009 | mostly explicit via test names + ADR 0002 §"Error tracking" | -- |
| NFR-RES-001 MAX_FINDINGS | ADR 0002 | numeric value not commented |
| NFR-RES-002..004 alert thresholds | -- | code-only -- VALUE RATIONALE MISSING |
| NFR-RES-005..010 ReassemblyConfig defaults | comments next to literals (5 of 6) | one inferred (max_segments=10K) |
| NFR-RES-011..018 HTTP/TLS caps | one (`MAX_RECORD_PAYLOAD`) commented; others bare | most magic numbers have no comment |
| NFR-RES-019 long URI | -- | code-only |
| NFR-RES-020 top-20 truncation | -- | code-only |
| NFR-RES-021 TLS short-circuit | doc comment at `tls.rs:263-265` | -- |
| NFR-MNT-001..003 CI lint/fmt/clippy | CI YAML explicit | -- |
| NFR-MNT-004 non_exhaustive | doc comment at `mitre.rs:17-20` | -- |
| NFR-MNT-005 tests-only-in-tests/ | -- | project-wide convention; not documented |
| NFR-MNT-006 no SPDX headers | -- | convention |
| NFR-MNT-007 semantic PR | CI YAML | -- |
| NFR-MNT-008 sticky alert flags | -- | code-only |
| NFR-MNT-009 technique_info SSOT | doc comment at `mitre.rs:93-96` | -- |
| NFR-MNT-010 3 ADRs | files exist | -- |
| NFR-MNT-011 effective MSRV 1.86 | -- | code-only -- HIDDEN |
| NFR-PORT-001 Linux-only CI | CI YAML | -- |
| NFR-PORT-002 no platform cfg | -- | inferable |
| NFR-PORT-003 no toolchain pin | -- | inferable |
| NFR-PORT-004 NO_COLOR | -- | code-only (and matches the no-color.org convention) |
| NFR-PORT-005 edition 2024 | manifest + rustfmt | -- |
| NFR-SUP-001..005 | manifest | -- |
| NFR-COMPAT-001 pcap-only / 5 link types | README §"Supported Link Types" | -- |
| NFR-COMPAT-002 terminal+json out | README §Features | -- |

**Headline:** the **highest-signal NFRs (security, finding-cap, dispatch policy) are well-documented** via ADRs 0001/0002/0003. The **reliability and performance NFRs are mostly tacit** (saturating arithmetic, wrapping arithmetic, panic-free constructor design). The **resource-bound magic numbers are documented inconsistently** -- defaults in `ReassemblyConfig` mostly have inline comments, but `OVERLAP_ALERT_THRESHOLD = 50`, `SMALL_SEGMENT_ALERT_THRESHOLD = 2048`, `OUT_OF_WINDOW_ALERT_THRESHOLD = 100`, `MAX_HEADERS = 96`, `MAX_BUF = 65_536`, "long URI" 2048 chars, top-20 truncation, and the 120/200 truncate_uri caps are all bare numeric literals with no comment explaining where the value came from.

---

## 7. NFR Violations (code-as-written contradicts a stated NFR)

| ID | Violation | Cited NFR / claim | Evidence |
|---|---|---|---|
| NFR-VIO-001 | README L17 calls wirerust "built for multi-GB captures" but the reader eagerly loads the entire pcap into a `Vec<RawPacket>` before any analysis begins. A 4 GB pcap implies at least 4 GB of resident memory just to hold the packets, plus per-packet `RawPacket` overhead (Vec capacity, struct fields). This is acceptable on a modern workstation but contradicts a strict reading of "streaming" -- the term "streaming" does not appear in the README; "one-pass triage" does, which is technically still true. | NFR-PERF-002 vs. README L17 | `src/reader.rs:38-48` (`Vec::new()` + `packets.push(...)` loop) |
| NFR-VIO-002 | `resolve_targets` collects `*.pcap` AND `*.pcapng` files from a directory (`src/main.rs:245-247`), but `PcapSource::from_pcap_reader` rejects pcapng at header-parse time (`src/reader.rs:22` returns "Failed to parse pcap header"). The result: directory mode silently swallows pcapng files into the "could not parse" path, polluting the first-error-only stderr message. Reader and CLI should agree. | NFR-COMPAT-001 vs. NFR-COMPAT-001's own README claim | `src/main.rs:245-247` collects `pcapng`; `src/reader.rs:22` rejects |
| NFR-VIO-003 | Seven CLI flags are advertised by `clap` but unwired in `main.rs`: `--threats`, `--beacon`, `--filter <BPF>`, `--verbose`, `--hosts`, `--services`, `--json <FILE>`, `--csv <FILE>`. They parse successfully and produce no behavior. Pass 3 BC-ABS-001..010 already enumerated them; for NFR purposes this is a violation of "the CLI surface should reflect actual behavior" (an implicit but conventional NFR). | NFR-OBS / NFR-MNT (CLI integrity) | `src/cli.rs:19,67,82,94,106,110,31,35`; `src/main.rs` destructures only a subset |
| NFR-VIO-004 | `--json <FILE>` and `--csv <FILE>` accept `Option<Option<PathBuf>>` suggesting "optional file output destination", but `src/main.rs:186, 232` unconditionally `println!` the rendered output to stdout. A user passing `--json out.json` sees nothing written to `out.json` and JSON dumped to terminal. This is a silent feature gap, worse than rejecting the flag. | NFR-COMPAT-002 vs. flag schema | `src/cli.rs:31-36`; `src/main.rs:186, 232` |
| NFR-VIO-005 | `csv` crate is declared in `Cargo.toml:17` and `OutputFormat::Csv` exists at `src/cli.rs:8`, but no CSV reporter is implemented. `src/main.rs:172-184` only matches `Some(OutputFormat::Json)` and falls through to `TerminalReporter` for `--output-format csv` -- the user gets terminal output, not CSV, with no error. | NFR-COMPAT-002 vs. supply chain | Pass 0 question #1 + Pass 3 BC-ABS-007 |
| NFR-VIO-006 | `rayon` declared in `Cargo.toml:22` for parallelism; **zero** uses in `src/` (README L152 lists it as roadmap). Unused prod dep. | NFR-SUP-001 | Pass 0 §2 confirmed |
| NFR-VIO-007 | `assert_cmd`, `predicates`, `tempfile` declared in dev-dependencies; **zero** uses in `tests/`. The `tests/cli_tests.rs` uses pure `Cli::parse_from` instead of spawning the binary -- no end-to-end binary test exists. | NFR-SUP-002 | Pass 0 §2 confirmed |
| NFR-VIO-008 | `JsonReporter::render` calls `serde_json::to_string_pretty(&output).unwrap()` at `src/reporter/json.rs:36`. Although the call is infallible by construction (see §5.1), the `unwrap()` is a clippy-friendly paper-cut in a codebase that is otherwise extremely careful about error propagation. Recommendation: convert to `.expect("Finding serialization is infallible")` for documentation, OR make `Reporter::render -> Result<String>`. | NFR-REL-008 (anyhow propagation) | `src/reporter/json.rs:36` |
| NFR-VIO-009 | Effective MSRV is 1.86 (because of `floor_char_boundary`), but `Cargo.toml` says nothing about MSRV and CI uses `stable`. A future stable downgrade or a user with rustc 1.85 would hit a confusing compile error. Recommendation: add `rust-version = "1.86"` to `Cargo.toml`. | NFR-MNT-011 / NFR-PORT-003 | `src/analyzer/http.rs:97`; absence of `rust-version` in `Cargo.toml` |
| NFR-VIO-010 | CI matrix runs only on `ubuntu-latest` -- macOS and Windows are not covered despite `cargo install --path .` being the documented install path (README L21-23) and no platform `cfg` gates anywhere. A platform-specific regression in `etherparse` or `pcap-file` could land undetected. | NFR-PORT-001 vs. install instructions | `.github/workflows/ci.yml:18,42,51,62`; README L21-23 |

---

## 8. Recommendations for Pass 4 Deepening Rounds

Specific gaps to investigate in deepening. Each cites a file:line and a concrete question.

| # | Gap | Where | Hypothesis to verify |
|---|---|---|---|
| R1 | `MAX_FINDINGS = 10_000` truncation contract | `src/reassembly/mod.rs:18, 272, 291, 310, 534, 550` and the `finalize()` bypass at 395-417 | Is the intent **truncation** ("first 10K findings win, the rest are dropped silently") or a **soft cap** that should emit an eprintln stat? The current behavior is silent truncation in `generate_*_finding` guards but the `finalize()` segment-limit finding deliberately bypasses the cap; that asymmetry deserves a test or a comment. |
| R2 | `OVERLAP_ALERT_THRESHOLD = 50`, `SMALL_SEGMENT_ALERT_THRESHOLD = 2048`, `OUT_OF_WINDOW_ALERT_THRESHOLD = 100` provenance | `src/reassembly/mod.rs:15-17` | Are these chosen empirically (against captured benign traffic) or are they "just a round number"? No comment, no test pins the cardinality directly; deepening should grep PR history or ADR drafts for justification, OR mark them all "inferred, calibrate against benign traffic" in a follow-up. |
| R3 | `max_segments_per_direction = 10_000` ceiling | `src/reassembly/mod.rs:47` | Comment cites "BTreeMap overhead explosion" but no benchmark exists in the repo. Is 10K the inflection point in BTreeMap's perf curve, or is it a guess? Deepening should look for a benchmark or document it as inferred. |
| R4 | TLS `MAX_BUF = 65_536` and HTTP `MAX_HEADER_BUF = 65_536` redundancy | `src/analyzer/tls.rs:14`; `src/analyzer/http.rs:8` | Same value, two constants, two analyzers, no shared definition. Is this a deliberate decoupling (so future TLS could differ) or accidental drift? Pass 4 deepening should consider whether to consolidate or document. |
| R5 | `MAX_HEADERS = 96` (HTTP) calibration | `src/analyzer/http.rs:9` | 96 covers the realistic worst case for legitimate traffic with margin, but where does that number come from (RFC, Suricata's default, IIS's limit)? No citation. Deepening should compare against Suricata/Zeek defaults. |
| R6 | Constructor preconditions not regression-tested | `src/reassembly/mod.rs:86-96` (5 `assert!`s) | No `#[should_panic]` test asserts the asserts. If a future refactor removes one, no CI signal. Recommend adding panic-tests OR converting to `Result<TcpReassembler, anyhow::Error>`. |
| R7 | "Long URI" finding 2048-char threshold | `src/analyzer/http.rs:265` | Is this an industry value (IIS, nginx default) or arbitrary? RFC 7230 §3.1.1 recommends "at least 8000 octets"; 2048 is more aggressive. Worth documenting OR raising to 8000. |
| R8 | `truncate_uri` 120-char summary cap consistency | `src/analyzer/http.rs:183, 211, 227` | Each call site repeats `120` as a literal -- magic number duplication. ADR 0003 §"Other formatting concerns" already flagged this as belonging at display layer; deepening should decide whether to extract a constant. |
| R9 | `summarize()` top-20 truncation | `src/analyzer/http.rs:502, 505`; `src/analyzer/tls.rs:714` | 20 is hard-coded in 3 places. Should be `const TOP_N_REPORT: usize = 20` and shared. Also: should this be operator-tunable via CLI? |
| R10 | Hidden MSRV 1.86 due to `floor_char_boundary` | `src/analyzer/http.rs:97`; `Cargo.toml` (absence of `rust-version`) | Add `rust-version = "1.86"` to `Cargo.toml` so cargo gives a clear error to users on older toolchains; also add a CI matrix dimension that builds against MSRV. |
| R11 | Eviction LRU semantics | `src/reassembly/mod.rs:506-531` (`evict_flows`) | Eviction is "non-established first, then oldest-`last_seen` first". Is `last_seen` monotonic (it's the pcap timestamp, NOT wall-clock) -- and what about the `u32` rollover at 2106? Deepening: confirm timestamp source and document boundary. |

---

## Pass 4 State Checkpoint

```yaml
pass: 4
round: 1
status: complete
files_scanned_src: 20
files_scanned_tests: 18 (cross-referenced via grep, not re-read)
adrs_consumed: 3
nfrs_catalogued: 76
magic_numbers_indexed: 28
nfr_violations_recorded: 10
recommendations: 11
confidence_overall: HIGH (resource-bound + security) / MEDIUM (performance + observability)
timestamp: 2026-05-19T00:00:00Z
next_pass: 5
resume_from: null
```

