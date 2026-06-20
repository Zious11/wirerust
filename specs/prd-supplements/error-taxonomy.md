---
document_type: prd-supplement-error-taxonomy
level: L3
version: "2.3"
status: draft
producer: product-owner
timestamp: 2026-06-12T02:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/prd.md
  - src/reader.rs
  - src/decoder.rs
  - src/main.rs
  - src/reassembly/mod.rs
  - src/reassembly/segment.rs
  - src/analyzer/http.rs
  - src/analyzer/tls.rs
  - src/analyzer/dns.rs
input-hash: N/A
# input-hash rationale: src/analyzer/arp.rs was removed from inputs (forward-referenced;
# file does not exist in develop HEAD until STORY-111 lands). The hash computation tool
# (bin/compute-input-hash) errors on missing inputs, so this supplement's hash is deferred
# until STORY-111 merges. Re-add src/analyzer/arp.rs to inputs: and run
# `bin/compute-input-hash --write` after STORY-111 lands.
traces_to: .factory/specs/prd.md
modified:
  - "v2.0: ARP-F2 Pass-14 remediation C-05 + C-07: (C-05) removed src/analyzer/arp.rs from inputs (not in develop HEAD; forward-reference to STORY-111); set input-hash to N/A with deferred-hash rationale comment. (C-07) E-ARP-002 Notes rewritten for clarity: 'within the average since window-start within the 60-second flap window' → explicit rate formula count/max(1,elapsed) prose; detector is average-rate not sliding-window; window semantics clarified. Version 1.9→2.0. — 2026-06-13"
  - "v2.2: D-068 remediation sweep — E-ARP-005 Notes corrected: 'MITRE techniques T0830 and T1557.002 attached to both forms' was unconditional and violated D-068. Replaced with conditional form: T0830 and T1557.002 attached ONLY on the GARP-that-conflicts path (BC-2.16.014); benign non-conflicting GARP emits mitre_techniques=[] per D-068. — 2026-06-14"
  - "v2.3: F2 pcapng-reader-support (FE-001, ADR-009) — (1) E-INP-008..011 added (pcapng block-level parse failures, EPB-before-IDB, EPB/SPB data truncation, multi-IDB linktype conflict). (2) E-INP-002 Notes revised: removed 'or pcapng format' trigger condition — pcapng files now route to E-INP-008..011 via BC-2.01.009 magic-byte probe, not E-INP-002. next_free_error_code = E-INP-012. — 2026-06-19"
  - "v2.1: P19 straggler anchor sweep — E-ANA-001 http.rs:405/:463 → :424/:484 (parse_errors increment); E-ANA-002 request block :406-415 → :424-434, response block :464-473 → :484-494; E-ANA-003 tls.rs:643-653 → :689-699; E-ANA-006 http.rs:375-389 → :390-394; E-ANA-007 tls.rs increment helper :372-375 → :379-384, call sites :387/:416/:494/:549/:564/:568 → :398/:427/:520/:593/:608/:612; E-ANA-008 http.rs:391-392 → :406; E-RAS-003 mod.rs :461/:495/:524 → :479/:515/:546, lifecycle.rs :101/:121 → :111/:141. Verified against src. — 2026-06-13"
---

# Error Taxonomy: wirerust

> PRD supplement -- extracted from PRD Section 5.
> Referenced by: implementer, test-writer.
> Brownfield: error paths catalogued from current develop HEAD source.

## Severity Definitions

| Severity | Meaning | Exit Code Impact | User Signal |
|----------|---------|-----------------|-------------|
| `broken` | Cannot continue; analysis aborts | Non-zero exit (1) | Error message via anyhow chain to stderr |
| `degraded` | Partial result possible; analysis continues | Exit 0 | Warning on stderr; `skipped_packets` or `parse_errors` counter incremented |
| `cosmetic` | Analyst-visible anomaly in output format or minor signal | Exit 0 | One-shot eprintln or finding emitted |

## Error Categories

| Category Code | Category | Description |
|--------------|----------|-------------|
| `INP` | Input / File | File access, pcap format validation, link-type rejection |
| `DEC` | Decoder | Packet-level decode failures (malformed L2/L3/L4 headers) |
| `RAS` | Reassembly | TCP stream reassembly state-machine edge cases and resource limits |
| `ANA` | Analyzer | Protocol-level parse failures (HTTP, TLS, DNS) |
| `ARP` | ARP Decoder | ARP frame decode and malformed-ARP detection signals |
| `OUT` | Output | File write failures for --json/--csv file paths |
| `CFG` | Configuration | Mutually exclusive flag combinations rejected by clap |


## Error Catalog

### INP: Input / File Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-INP-001 | Input | `broken` | 1 | `src/reader.rs:56-60` | `Unsupported pcap link type: <type>. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)` | BC-2.01.001, BC-2.02.008 | Surfaced via anyhow chain. `<type>` is `DataLink` Debug repr (e.g. `UNKNOWN(166)`) |
| E-INP-002 | Input | `broken` | 1 | `src/reader.rs:46` | `Failed to parse pcap header: <underlying>` | BC-2.01.006 | `pcap_file::pcap::PcapReader::new` failure; wrong magic number or truncated classic-pcap file. **F2 note:** pcapng files are NO LONGER a trigger for E-INP-002; the BC-2.01.009 magic-byte probe routes pcapng files to E-INP-008..011 before reaching this path. |
| E-INP-003 | Input | `broken` | 1 | `src/reader.rs:70` | `Failed to read packet: <underlying>` | BC-2.01.007 | Per-packet `next_raw_packet()` failure; corrupt or truncated payload |
| E-INP-004 | Input | `broken` | 1 | `src/reader.rs:86-87` | `Failed to open <path>: <os-error>` | BC-2.12.012 | `std::fs::File::open` failure; file not found, permission denied |
| E-INP-005 | Input | `broken` | 1 | `src/main.rs:147`, `src/main.rs:260` | `Failed to read <path>: <underlying>` | BC-2.12.012 | Wraps E-INP-001..003; `with_context` adds file path. Surfaced via `PcapSource::from_file` in the capture loop |
| E-INP-006 | Input | `broken` | 1 | `src/main.rs:363` | `Target not found: <target>` | BC-2.12.012 | `anyhow::bail!` when target path is neither file nor directory |
| E-INP-007 | Input | `degraded` | 0 | `src/main.rs:170-177` | `Warning: failed to decode packet (<error>). Further errors counted silently.` | BC-2.12.014 | Printed to stderr ONCE per run; subsequent decode errors are counted into `Summary.skipped_packets` silently. Only the first decode error per run produces a message. |
| E-INP-008 | Input | `broken` | 1 | `src/reader.rs` (pcapng SHB/IDB parse path) | `Failed to parse pcapng <block-type>: <underlying>` | BC-2.01.010, BC-2.01.011, BC-2.01.017 | Covers structural parse failures at the SHB or IDB level: truncated file, missing BOM, malformed block-total-length, unsupported major version. `<block-type>` is one of "Section Header Block", "Interface Description Block". `<underlying>` is the anyhow root cause. Surfaced via anyhow chain; ultimate wrapper is E-INP-005 ("Failed to read \<path\>: \<underlying\>"). |
| E-INP-009 | Input | `broken` | 1 | `src/reader.rs` (pcapng EPB parse path, pre-IDB guard) | `pcapng Enhanced Packet Block encountered before any Interface Description Block` | BC-2.01.012, BC-2.01.017 | Emitted when an EPB is encountered and the interface table is empty (no IDB has been seen in the current section). This is a pcapng structural violation. The file is broken — no safe way to interpret the packet's linktype or timestamp resolution. |
| E-INP-010 | Input | `broken` | 1 | `src/reader.rs` (pcapng EPB/SPB/unknown-block parse path) | `Failed to parse pcapng <block-type> (block #<seq>): <underlying>` | BC-2.01.012, BC-2.01.013, BC-2.01.015, BC-2.01.017 | Covers packet-data-level truncation in EPB and SPB (captured_length inconsistent with block_total_length), and block_total_length < 8 in unknown-block skip. `<block-type>` is "Enhanced Packet Block", "Simple Packet Block", or "unknown block (type=0x{N:08X})". `<seq>` is the 1-based block sequence number within the file for debuggability. |
| E-INP-011 | Input | `broken` | 1 | `src/reader.rs` (pcapng multi-IDB agreement check) | `pcapng multi-interface link-type conflict: interface 0 has <first:?>, interface <n> has <other:?>` | BC-2.01.018, BC-2.01.017 | Emitted when two or more IDBs in a section carry different `linktype` values. `<first:?>` and `<other:?>` are the `DataLink` Debug repr values. This reflects the fail-closed multi-IDB policy (ADR-009 Decision 3). Known limitation: rejects legitimate multi-NIC captures mixing Ethernet and Linux Cooked interfaces. |

### DEC: Decoder Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-DEC-001 | Decoder | `degraded` | 0 | `src/decoder.rs` (decode_packet) | (no message -- Result::Err returned to caller) | BC-2.02.007 | `etherparse::SlicedPacket::from_ethernet` / `from_ip` / `from_linux_sll` (selected by `datalink` match) fails for genuine structural corruption (bad header version, bad IHL, bad TCP data-offset). NOT triggered by snaplen-length truncation (see E-DEC-002). Propagates as anyhow::Error to caller (main.rs E-INP-007 path). |
| E-DEC-002 | Decoder | `degraded` | 0 | `src/decoder.rs` (lax fallback) | (no message -- continues with degraded ParsedPacket) | BC-2.02.003 | Strict parser returns `SliceError::Len` -> lax (`LaxSlicedPacket`) fallback triggered. Packet decoded with clamped lengths. This is NOT an error from the caller's perspective; it produces a valid ParsedPacket. |
| E-DEC-003 | Decoder | `degraded` | 0 | `src/decoder.rs` | `No IP layer found` | BC-2.02.009 (Path 3) | anyhow error returned when the frame is non-IP and non-ARP (e.g. LLDP, EtherType 0x9000). Counted as skipped. **PLANNED (STORY-111):** Since v0.7.0, ARP frames will no longer produce this error — they return Ok(DecodedFrame::Arp) or E-ARP-001 instead. This behavior change requires the decoder refactor in STORY-111; not yet present in develop HEAD. |
| E-DEC-004 | Decoder | `degraded` | 0 | `src/decoder.rs` | `Non-Ethernet/IPv4 ARP frame` | BC-2.02.009 (Path 2), BC-2.16.009 | **PLANNED (STORY-111):** anyhow error returned when an ARP frame is present but hw_addr_type != Ethernet (0x0001) or proto_addr_type != IPv4 (0x0800) or hw_addr_size != 6 or proto_addr_size != 4. Counted as skipped. The ArpAnalyzer's D11 malformed finding (E-ARP-001) is additionally emitted when --arp is active. Added in v0.7.0 (ADR-008 Decision 2); not present in develop HEAD until STORY-111 lands. |

### RAS: Reassembly Errors / Signals

These are not exit-code-1 errors; they are internal state signals that produce
findings or one-shot warnings. They are catalogued here for implementer completeness.

| Error Code | Category | Severity | Exit Code | Source Location | Signal Type | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|-------------|--------|-------|
| E-RAS-001 | Reassembly | `cosmetic` | 0 | `src/reassembly/segment.rs:16, 54-55` | One-shot stderr: `wirerust: insert_segment called with no ISN set` | BC-2.04.032, BC-2.04.048 | Guarded by `ISN_MISSING_WARNED: AtomicBool` (segment.rs:16). `eprintln!` fires at most ONCE per process (segment.rs:54-55). |
| E-RAS-002 | Reassembly | `cosmetic` | 0 | `src/reassembly/lifecycle.rs:31, 44-47` | One-shot stderr: `wirerust: close_flow called for non-existent key: <key> (reason: <reason>)` | BC-2.04.029 | Guarded by `CLOSE_FLOW_MISSING_WARNED: AtomicBool` (lifecycle.rs:31). `eprintln!` fires at most ONCE (lifecycle.rs:44-47). `<reason>` is the `CloseReason` Debug repr. Indicates a structural invariant violation (flow table inconsistency). |
| E-RAS-003 | Reassembly | `degraded` | 0 | `src/reassembly/mod.rs:479, 515, 546`; `src/reassembly/lifecycle.rs:111, 141` | Silent drop + counter increment | BC-2.04.024 | When `self.findings.len() >= MAX_FINDINGS (10,000)`, further per-flow findings are silently dropped. The `finalize()` summary finding unconditionally bypasses this cap. |
| E-RAS-004 | Reassembly | `cosmetic` | 0 | `src/reassembly/mod.rs:115-125` | `assert!` panic (programmer error only; unreachable in production via CLI path post-FIX-P5-002) | BC-2.04.001 | `TcpReassembler::new` panics if any config field is 0 or invalid. **FIX-P5-002 (ADV-IMPL-P04-MED-001, 2026-06-01):** `--reassembly-depth` and `--reassembly-memcap` now use a `parse_nonzero_usize` custom clap `value_parser` that rejects 0 at parse time (exit 2, `ValueValidation`, message `"0 is not in 1.."`), preventing 0 from ever reaching this assert on the operator-input path. The assert is retained as a backstop for programmer-error scenarios (e.g., direct Rust API construction with a zero-valued config). The prior panic on user-supplied `--reassembly-depth 0` / `--reassembly-memcap 0` is now fully prevented; that class of input is handled at the CLI layer as a usage error (E-CFG-007 below). |
| E-RAS-005 | Reassembly | `degraded` | 0 | `src/reassembly/segment.rs` | `InsertResult::SegmentLimitReached` returned | BC-2.04.044..046 | When `max_segments_per_direction` (default 10,000) is reached, new segments return `SegmentLimitReached`. Tracked via `segments_segment_limit`. A summary finding is emitted by `finalize()`. |

### ANA: Analyzer Parse Errors

| Error Code | Category | Severity | Exit Code | Source Location | Signal Type | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|-------------|--------|-------|
| E-ANA-001 | Analyzer | `degraded` | 0 | `src/analyzer/http.rs:424, 484` | `parse_errors` counter incremented | BC-2.06.013, BC-2.06.015 | Non-HTTP bytes or incomplete HTTP headers that fail `httparse`. No finding emitted for individual parse errors. After `POISON_THRESHOLD=3` consecutive errors in one direction, that direction is "poisoned" (E-ANA-002). |
| E-ANA-002 | Analyzer | `degraded` | 0 | `src/analyzer/http.rs:424-434` (request), `484-494` (response) | Direction poisoned; `poisoned_bytes_skipped` counter incremented | BC-2.06.015..017 | HTTP direction poisoning: after 3 consecutive parse errors, subsequent bytes in that direction are skipped. Per-direction and per-flow. Cleared on `on_flow_close`. |
| E-ANA-003 | Analyzer | `degraded` | 0 | `src/analyzer/tls.rs:689-699` | `parse_errors` incremented; per-direction buffer cleared | BC-2.07.004, BC-2.07.029 | TLS record payload exceeds `MAX_RECORD_PAYLOAD=18,432` bytes or body parse fails. Buffer is cleared; analysis continues on next record. |
| E-ANA-004 | Analyzer | `degraded` | 0 | `src/analyzer/tls.rs` | `parse_errors` incremented | BC-2.07.029 | TLS record body parsing failure (bad handshake structure, truncated extension). Buffer continues. |
| E-ANA-005 | Analyzer | `cosmetic` | 0 | `src/analyzer/dns.rs` | (none -- counts as query or response) | BC-2.08.002 | DNS parse error is implicit: malformed DNS is silently counted as a query or response based on the QR-bit only; no per-packet parse error counter exists for DNS. |
| E-ANA-006 | Analyzer | `cosmetic` | 0 | `src/analyzer/http.rs:390-394` | New map key silently dropped | BC-2.06.024 | HTTP per-map cardinality (`MAX_MAP_ENTRIES=50,000`): new keys past the cap are dropped; existing keys still increment. Affects: `methods`, `hosts`, `user_agents`, `status_codes`. |
| E-ANA-007 | Analyzer | `cosmetic` | 0 | `src/analyzer/tls.rs:379-384` (increment helper), `398, 427, 520, 593, 608, 612` (call sites) | New map key silently dropped | BC-2.07.028 | TLS per-map cardinality (`MAX_MAP_ENTRIES=50,000`): same behavior. Affects: `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`. SNI anomaly findings still fire even when `sni_counts` is at capacity. |
| E-ANA-008 | Analyzer | `cosmetic` | 0 | `src/analyzer/http.rs:406` | URI silently dropped | BC-2.06.025 | HTTP URI list cap: `MAX_URIS=10,000`; further URIs silently dropped from the `uris` list. Detection rules continue to run on dropped URIs. |

### ARP: ARP Decoder Signals

> NOTE: ARP decode + analyzer behavior and T0830/T1557.002 MITRE arms are PLANNED in
> STORY-111..115 (v0.7.0); not present in current develop HEAD. `technique_name` returns
> `None` for T0830/T1557.002 until STORY-114. Rows E-DEC-004, E-ARP-001..005 describe
> the post-STORY-111..115 target behavior.

These are not exit-code-1 errors; they are either degraded-result conditions that increment
skipped-packet counters or findings emitted by the ArpAnalyzer. They are catalogued here
for implementer and test-writer completeness. All require `--arp` to be active unless noted.

| Error Code | Category | Severity | Exit Code | Source Location | Signal Type | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|-------------|--------|-------|
| E-ARP-001 | ARP | `degraded` | 0 | `src/analyzer/arp.rs` (D11 malformed path) | Finding emitted: Anomaly/LOW; message format: `Malformed ARP: hw_addr_size=<n>, proto_addr_size=<n>, hw_type=<hex>` | BC-2.16.009 | Emitted when `extract_arp_frame` returns `None` (non-Ethernet/IPv4 hw/proto sizes), indicating a non-standard or malformed ARP frame. Verdict triple: `finding_type: Anomaly`, `confidence: LOW` (per BC-2.16.009 Postcondition 3 — no Inconclusive field; LOW reflects the high FP rate in ICS environments with legacy protocol converters). The frame is counted in `malformed_frames` (not `frames_analyzed`) per BC-2.16.010. Also counted as a skipped packet via E-DEC-004. Requires `--arp`. |
| E-ARP-002 | ARP | `cosmetic` | 0 | `src/analyzer/arp.rs` (D3 storm path) | Finding emitted: Anomaly/MEDIUM; message format: `ARP storm from <mac>: <rate> frames/sec (threshold: <n>)` | BC-2.16.008 | Emitted when a source MAC's computed rate meets or exceeds `ARP_STORM_RATE_DEFAULT` (default 50) ARP frames per second. Rate is `count_in_window / max(1, ts - window_start_ts)` — average over the elapsed seconds since window-start, not a sliding-window detector. One-shot per source MAC per 60-second window (ARP_FLAP_WINDOW_SECS=60). Counter state stored per-MAC in a bounded map (MAX_STORM_COUNTERS=4,096; LRU eviction at cap). Requires `--arp`. |
| E-ARP-003 | ARP | `cosmetic` | 0 | `src/analyzer/arp.rs` (D12 mismatch path) | Finding emitted: Anomaly/MEDIUM; message format: `ARP sender/Ethernet MAC mismatch: sender_mac=<mac>, outer_src_mac=<mac>` | BC-2.16.007 | Emitted when the Ethernet frame's outer source MAC differs from the ARP sender HW address field (D12). Requires Ethernet link type (`outer_src_mac` is `Some`); silently skipped for SLL captures where `outer_src_mac` is `None`. Requires `--arp`. MITRE techniques T0830 (Adversary-in-the-Middle, ICS) and T1557.002 (ARP Cache Poisoning, Enterprise) attached (per BC-2.16.007 PC1). |
| E-ARP-004 | ARP | `cosmetic` | 0 | `src/analyzer/arp.rs` (D1 spoof path) | Finding emitted: Anomaly/MEDIUM or Anomaly/HIGH; message format: `ARP spoof: IP <ip> changed MAC from <old_mac> to <new_mac> (rebind <n>)` | BC-2.16.004 | Emitted when `ArpAnalyzer::process_arp` detects an IP→MAC rebind (sender_ip already in binding table with a different sender_mac). Exactly one Finding per rebind event. Severity = HIGH iff `rebind_count >= spoof_threshold AND (timestamp_secs - first_rebind_ts <= ARP_FLAP_WINDOW_SECS) AND !spoof_high_emitted`, else MEDIUM (per BC-2.16.004 PC1.c). `spoof_high_emitted` one-shot guard prevents repeated HIGH findings per flap window. MITRE techniques T0830 (LateralMovement) and T1557.002 (CredentialAccess) attached. Requires `--arp`. |
| E-ARP-005 | ARP | `cosmetic` | 0 | `src/analyzer/arp.rs` (D2 GARP path) | Finding emitted: Anomaly/LOW (benign GARP) or Anomaly/MEDIUM (GARP-that-conflicts); message format: `Gratuitous ARP from <ip> (sender_mac=<mac>)` | BC-2.16.003, BC-2.16.014 | Emitted when `is_gratuitous_arp(frame)` returns `true` (sender_ip == target_ip, any opcode). Confidence = LOW when no binding conflict exists; MEDIUM when the same frame also triggers a D1 binding conflict (GARP-that-conflicts escalation, BC-2.16.014). MITRE techniques T0830 and T1557.002 attached ONLY on the GARP-that-conflicts path (BC-2.16.014); benign non-conflicting GARP emits mitre_techniques=[] per D-068. Requires `--arp`. |

### OUT: Output Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-OUT-001 | Output | `broken` | 1 | `src/main.rs:333-334` | `Failed to write JSON output to <path>: <os-error>` | BC-2.12.017 | `std::fs::write` failure when `--json <FILE>` specifies a path (permission denied, disk full, bad path). |
| E-OUT-002 | Output | `broken` | 1 | `src/main.rs:335-336` | `Failed to write CSV output to <path>: <os-error>` | BC-2.12.017 | Same as E-OUT-001 for `--csv <FILE>`. |

### CFG: Configuration Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-CFG-001 | Config | `broken` | 2 | clap (src/cli.rs:72) | `error: the argument '--reassemble' cannot be used with '--no-reassemble'` | BC-2.12.007 | clap enforces `conflicts_with`; prints to stderr with usage hint; exits with code 2 (clap's standard argument error code, NOT exit code 1). |
| E-CFG-002 | Config | `broken` | 2 | clap (src/cli.rs:53) | `error: the argument '--json...' cannot be used with '--csv...'` | BC-2.12.017 | clap `conflicts_with = "csv"` on `--json`. Same exit code 2. |
| E-CFG-003 | Config | `broken` | 2 | clap | `error: invalid value '<VAL>' for '--overlap-threshold <OVERLAP_THRESHOLD>': <VAL> is not in 0..=255` | BC-2.12.005 | clap range validator `value_parser(clap::value_parser!(u32).range(0..=255))`. |
| E-CFG-004 | Config | `broken` | 2 | clap | `error: invalid value '<VAL>' for '--small-segment-threshold <SMALL_SEGMENT_THRESHOLD>': <VAL> is not in 0..=2048` | BC-2.12.005 | clap range validator `value_parser(clap::value_parser!(u32).range(0..=2048))`. |
| E-CFG-005 | Config | `broken` | 2 | clap | `error: invalid value '<VAL>' for '--small-segment-max-bytes <SMALL_SEGMENT_MAX_BYTES>': <VAL> is not in 0..=2048` | BC-2.12.005 | clap range validator `value_parser(clap::value_parser!(u16).range(0..=2048))`. |
| E-CFG-006 | Config | `degraded` | 0 | `src/main.rs:90-93` | `Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. Stream analysis will be skipped.` | BC-2.12.009 | Semantic conflict not enforced by clap; wirerust continues without stream analyzers. Not exit-code 1. |
| E-CFG-007 | Config | `broken` | 2 | clap (`src/cli.rs`, `parse_nonzero_usize` value_parser on `--reassembly-depth`) | `error: invalid value '0' for '--reassembly-depth <REASSEMBLY_DEPTH>': 0 is not in 1..` | BC-2.12.005 EC-006 | **FIX-P5-002 (2026-06-01).** Custom `parse_nonzero_usize` clap `value_parser` enforces >= 1 at parse time. Observable as a clap `ValueValidation` error; exit code 2. Prevents 0 from reaching `TcpReassembler::new`'s assert (see E-RAS-004). |
| E-CFG-008 | Config | `broken` | 2 | clap (`src/cli.rs`, `parse_nonzero_usize` value_parser on `--reassembly-memcap`) | `error: invalid value '0' for '--reassembly-memcap <REASSEMBLY_MEMCAP>': 0 is not in 1..` | BC-2.12.005 EC-007 | **FIX-P5-002 (2026-06-01).** Same `parse_nonzero_usize` value_parser as E-CFG-007; applied to `--reassembly-memcap`. Exit code 2. Prevents memcap=0 from reaching `TcpReassembler::new`'s assert. |

## Error Handling Strategy Summary

| Layer | Strategy | Rationale |
|-------|----------|-----------|
| File/pcap open | `anyhow::bail!` / `?` propagation -- abort | A missing or unreadable pcap means no analysis is possible for that target. |
| Per-packet decode | Count to `skipped_packets`; continue loop | Single bad packet must not abort the analysis of a valid pcap. |
| Snaplen truncation | Lax fallback -- degrade gracefully | Common in real-world forensic captures; strict rejection would miss valid packets. |
| HTTP parse error | Count to `parse_errors`; optionally poison after threshold | Mid-stream joins produce transient errors; 3 consecutive errors indicate non-HTTP stream. |
| TLS parse / buffer overflow | Count to `parse_errors`; clear buffer; continue | Malformed TLS records should not kill handshake fingerprinting for subsequent records. |
| DNS parse | Implicit QR-bit dispatch; no error counter | DNS is statistics-only; failure to parse one record only affects that record's counter. |
| ARP non-Ethernet/IPv4 | E-DEC-004 degraded result (skipped packet) + optional D11 finding via E-ARP-001 | Non-standard ARP frames are rare but valid in OT/ICS networks; analyst signal emitted rather than silent skip. |
| ARP storm | One-shot finding per source MAC per 60s window (E-ARP-002) | Rate-based: avoids finding spam on legitimate ARP floods; one-shot guard resets at window expiry. |
| Findings cap | Silent drop after MAX_FINDINGS=10,000 | Adversarial input could generate unbounded findings; cap prevents memory exhaustion. |
| Output write | `anyhow` `?` -- abort with message | If the analyst-requested output cannot be written, the run is a failure. |
| Clap arg parse | clap exits 2 | Standard UX contract; invalid flags are user errors, not internal errors. |
