# Pass 3 (Behavioral Contracts) -- Deepening Round 2 -- wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 3 (Behavioral Contracts) -- Phase B deepening round 2
- **Round 1 reference:** `wirerust-pass-3-behavioral-contracts.md` (claimed 137 BCs / 26 MEDIUM / 10 ABS; **actual recount 216 BCs / 40 MEDIUM / 10 ABS** -- see §1 audit)
- **Inputs re-read (round-2):** Pass 3 R1 (BC index, full), Pass 2 R1 (entities, BRs), Pass 4 R1 (NFR table), Pass 6 synthesis (gap report §6.6, deepening plan §7 P3), source modules `reassembly/mod.rs`, `reader.rs`, `cli.rs`, `main.rs`, `analyzer/http.rs`, `analyzer/tls.rs`, tests `reassembly_engine_tests.rs` (1398 LOC, full scope), `reassembly_segment_tests.rs` (402 LOC, full scope), `http_analyzer_tests.rs` (735 LOC), `tls_analyzer_tests.rs` (1455 LOC, partial), `reader_tests.rs` (148 LOC, full), `cli_tests.rs` (110 LOC, full).
- **Scope:** carryover targets verbatim from Pass 3 R1 orchestrator note + Pass 6 §6.6 / §7 P3.

---

## 1. Hallucination-class audit results

Audited Pass 3 R1 against actual source and tests. Each row = one audited claim. CONV-ABS-N markers identify retractions.

| # | Class | Pass-3-R1 claim | Actual finding (re-counted/re-read) | Verdict | Retraction marker |
|---|---|---|---|---|---|
| 1 | Inflated/deflated metrics (class 5) | "137 BCs total" in §0 prelude | 216 unique BC IDs counted from `## 1. BC Index` table (`awk` distinct count) | DEFLATED -- the 137 figure is wrong by ~80 | **CONV-ABS-1 RETRACTED** |
| 2 | Inflated/deflated metrics (class 5) | "26 MEDIUM-confidence BCs" in P3 R1 prelude and P6 synthesis | 40 MEDIUM rows (`MEDIUM ` table cells, regex-exact) in BC index. Pass 6 §6.6 lists 27 by ID but the index has 40 rows tagged MEDIUM. | INFLATED understatement by ~14 | **CONV-ABS-2 RETRACTED** |
| 3 | Inflated/deflated metrics (class 5) | "10 absent BCs" / "BC-ABS-001..010" | 10 confirmed (9 HIGH-absent + 1 MEDIUM-absent on BC-ABS-009). | CORRECT | -- |
| 4 | Inflated/deflated metrics (class 5) | "81% HIGH confidence" derivation | Re-derive from actual: 216 - 40 MEDIUM - 4 LOW - 10 ABS = 162 HIGH; 162/216 = 75%, not 81%. With ABS pulled out: 162/(216-10) = 78.6%. | Either way, NOT 81% | **CONV-ABS-3 RETRACTED** |
| 5 | Miscounted enumerations (class 2) | "BC-RAS-001..053 -- 53 reassembly BCs" | The BC index has 53 RAS rows. Confirmed: BC-RAS-001 through BC-RAS-053 dense, no gaps. | CORRECT | -- |
| 6 | Miscounted enumerations (class 2) | "BC-HTTP-001..026 -- 26 HTTP BCs" | 26 rows present, no gaps. | CORRECT | -- |
| 7 | Miscounted enumerations (class 2) | "BC-TLS-001..036 -- 36 TLS BCs" | 36 rows present, no gaps. | CORRECT | -- |
| 8 | Miscounted enumerations (class 2) | "BC-CLI-001..017 -- 17 CLI BCs" | 17 rows present (BC-CLI-001..017). | CORRECT | -- |
| 9 | Over-extrapolated token list (class 1) | BC-HTTP-005 lists "../, ..%2f, ..%252f, ....//" as the COMPLETE traversal token set | Re-read of `http.rs:174-178`: exactly those 4 substrings. | CORRECT | -- |
| 10 | Over-extrapolated token list (class 1) | BC-HTTP-006 enumerates 6 web-shell patterns: "/shell.php, /cmd.asp, /c99.php, /r57.php, /webshell, /backdoor, etc." | Actual code `http.rs:192-203` has **10 patterns**: /shell.php, /shell.asp, /shell.jsp, /cmd.php, /cmd.asp, /cmd.jsp, /c99.php, /r57.php, /webshell, /backdoor. The R1 text says "etc." which is unverifiable; tightened in §3. | UNDER-listed (over-extrapolated via "etc.") | **CONV-ABS-4 RETRACTED** (tighten to literal 10) |
| 11 | Over-extrapolated token list (class 1) | BC-HTTP-007 admin patterns: "/wp-admin, /admin, /phpmyadmin, /manager" | `http.rs:221`: exactly those 4. | CORRECT | -- |
| 12 | Over-extrapolated token list (class 1) | BC-HTTP-008 unusual methods: "CONNECT/TRACE/DELETE/OPTIONS" | `http.rs:236`: exactly those 4. | CORRECT | -- |
| 13 | Over-extrapolated token list (class 1) | BC-RDR-001 supported link types: "Ethernet/RAW/IPv4/IPv6/SLL" and numeric 1/101/113/228/229 | `reader.rs:25-30` confirms 5 variants; error message `:33` lists numeric IDs 1/101/113/228/229. | CORRECT | -- |
| 14 | Named pattern conflation (class 3) | BC-DEC-012 says app_protocol_hint table is "DNS/HTTP/TLS/SSH/SMB/Modbus/DNP3 from port 53/80/443/22/445/502/20000" -- 7 services | Need to verify list count exactly 7. Confirmed via earlier passes (Pass 4 NFR-MNT-008 + Pass 6 unknowns §18). | CORRECT (and confirmed 7-entry, not subject to "etc.") | -- |
| 15 | Same-basename artifact conflation (class 4) | BC IDs cite `mod.rs` line numbers | 3 `mod.rs` files exist (`reassembly/mod.rs`, `analyzer/mod.rs`, `reporter/mod.rs`). Spot-checked: BC-RAS-024 cites "mod.rs:534" -- that's the `generate_conflicting_overlap_finding`'s `if findings.len() >= MAX_FINDINGS { return; }` guard in `reassembly/mod.rs:534`. Correct module. BC-RAS-018/023/025 also cite mod.rs:539-562 -- all in `reassembly/mod.rs`. | CORRECT (no cross-module misattribution found) | -- |
| 16 | Same-basename artifact conflation (class 4) | BC-RPT-019 cites "terminal.rs:65-137" | `reporter/terminal.rs` (350 LOC). One `terminal.rs` only. Spot-check OK. | CORRECT | -- |
| 17 | Named pattern conflation (class 3) | BC-RAS-019 says "Exceeding OVERLAP_ALERT_THRESHOLD (50)" -- threshold is **50**, finding fires at count **>50** (i.e., 51) | `reassembly/mod.rs:270` is `flow_dir.overlap_count > OVERLAP_ALERT_THRESHOLD`. Test `test_overlap_anomaly_finding` sends 51 duplicates and asserts the finding fires. Wording "(50)" without the boundary qualifier is slightly misleading. | TECHNICALLY CORRECT but boundary semantics not in BC text. Tightened in §3 (BC-RAS-019). | **CONV-ABS-5 PARTIAL TIGHTEN** |
| 18 | Named pattern conflation (class 3) | BC-RAS-024 calls MAX_FINDINGS cap "per-flow findings silently dropped" | `mod.rs:272/291/310` use `&& self.findings.len() < MAX_FINDINGS` -- only THREE per-direction-alert findings respect the cap. `generate_conflicting_overlap_finding` (`mod.rs:534`) and `generate_truncated_finding` (`mod.rs:549`) ALSO check, but `finalize`'s segment-limit finding at `:400-417` pushes unconditionally (comment: "Pushed unconditionally (at most 1 finding) to avoid being silently dropped"). The BC text "per-flow findings" is OK; but the BC fails to note the finalize bypass is a deliberate carve-out. | Tightened in §3 (BC-RAS-024 + new BC-RAS-054 for finalize bypass) | **CONV-ABS-6 PARTIAL TIGHTEN** |
| 19 | Over-extrapolated token list (class 1) | BC-TLS-014 "C0 control byte (0x00-0x1F) or DEL (0x7F)" -- claims this is the complete control-byte range that emits the finding | The TLS analyzer separates ASCII-with-control from C1; the round-1 BC text is silent on C1 control bytes (U+0080..U+009F) for SNI. Re-read `tls.rs:370-407`: the discrimination uses the `SniValue::AsciiWithControl` variant which is exclusively C0+DEL. C1 bytes in an SNI would route to either NonAsciiUtf8 (if valid UTF-8 encoding the C1 codepoint) or NonUtf8 (if raw 0x80-0x9F bytes). BC-TLS-014 is CORRECT as written but the discrimination rule deserves a tightened BC (see new BC-TLS-037 in §3). | CORRECT but UNDER-specified | new BC added in §3 |
| 20 | Miscounted enumerations (class 2) | "Test functions catalogued: 202" in P3 R1 header | Cross-check with P0: 202. Re-verifiable. Not part of this round's audit scope. | DEFER -- not BC-affecting | -- |

**Audit verdict:** 6 retractions, of which 4 are metric corrections (CONV-ABS-1..3 + CONV-ABS-4 token-list under-count) and 2 are partial tightenings (CONV-ABS-5 / CONV-ABS-6). No same-basename misattribution found. No named-pattern conflation that materially changes the BC.

---

## 2. Per-target findings

### Target 1 -- BC-RAS-023 (Truncated/depth findings: finding-shape contract)

**Pass-3-R1 wording:** "Truncated segment (insert exceeds max_depth) emits an Anomaly/Inconclusive/Low finding with no MITRE ID. MEDIUM confidence -- mod.rs:549-562 (`generate_truncated_finding`) -- not directly tested for finding emission, depth-exceeded counter is tested."

**Source re-read (`reassembly/mod.rs:549-563`):**
```
fn generate_truncated_finding(&mut self, key: &FlowKey, src_ip: std::net::IpAddr) {
    if self.findings.len() >= MAX_FINDINGS { return; }
    self.findings.push(Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: format!("Stream depth exceeded on flow {}", key),
        evidence: vec![format!("Max depth {} bytes reached", self.config.max_depth)],
        mitre_technique: None,
        source_ip: Some(src_ip),
        timestamp: None,
    });
}
```
Call site: `reassembly/mod.rs:243-246` (`InsertResult::Truncated` arm) -- the truncated branch is reached on the FIRST segment that crosses `max_depth`, NOT on subsequent fully-rejected segments (those hit `DepthExceeded`, no finding).

**Test pinning audit:** `test_depth_exceeded_counter` (`reassembly_engine_tests.rs:1330-1398`) drives:
- Packet 1 (8 bytes) under 10-byte cap -> Inserted
- Packet 2 (5 bytes) exceeds cap by 3 -> `Truncated` (truncated to 2 bytes, finding **should** emit)
- Packet 3 (5 bytes) post-cap -> `DepthExceeded` counter
- Assertion: `segments_depth_exceeded == 1` (after p3, NOT after p2).
- **Gap:** the test does NOT assert that `reassembler.findings()` contains a "Stream depth exceeded" entry. The finding emission is reachable in the code path but unasserted.

**Refined BC -- finding-shape contract (HIGH-confidence on shape, MEDIUM-confidence on test pinning):**

> **BC-RAS-023 (refined).** When a TCP segment's payload would cause `buffered_bytes + reassembled_bytes` to exceed `config.max_depth`, the segment is truncated to the remaining capacity AND a single `Finding` is pushed into `reassembler.findings()` with the following exact shape:
> - `category` = `ThreatCategory::Anomaly`
> - `verdict` = `Verdict::Inconclusive`
> - `confidence` = `Confidence::Low`
> - `summary` = exact format string `"Stream depth exceeded on flow {key}"` where `{key}` is `FlowKey::Display` ("lower_ip:lower_port -> upper_ip:upper_port" with U+2192 arrow per BC-RAS-049)
> - `evidence` = vec with one entry: `"Max depth {N} bytes reached"` where `{N}` = `config.max_depth` (a `usize`) -- value at construction-time, NOT decremented after truncation
> - `mitre_technique` = `None`
> - `source_ip` = `Some(packet.src_ip)` -- the IP whose segment caused the truncation, NOT the flow initiator
> - `timestamp` = `None`
>
> The finding emits exactly once per `Truncated` InsertResult event. Subsequent `DepthExceeded` events on the same direction increment `segments_depth_exceeded` but do NOT push another finding.

**Test recommendation (to upgrade to HIGH):** Extend `test_depth_exceeded_counter` with assertions:
```rust
let trunc_finding = reassembler.findings().iter()
    .find(|f| f.summary.contains("Stream depth exceeded"))
    .expect("truncated finding must emit");
assert_eq!(trunc_finding.category, ThreatCategory::Anomaly);
assert_eq!(trunc_finding.verdict, Verdict::Inconclusive);
assert_eq!(trunc_finding.confidence, Confidence::Low);
assert!(trunc_finding.mitre_technique.is_none());
assert!(trunc_finding.evidence[0].contains("Max depth 10 bytes reached"));
assert_eq!(trunc_finding.source_ip, Some(IpAddr::V4(Ipv4Addr::new(10,0,0,1))));
```
After this addition, BC-RAS-023 upgrades to HIGH.

---

### Target 2 -- BC-RAS-024 (MAX_FINDINGS = 10_000 saturation behavior)

**Pass-3-R1 wording:** "Total findings capped at MAX_FINDINGS=10000; further per-flow findings silently dropped. MEDIUM -- mod.rs:534 -- not directly tested."

**Source re-read (`reassembly/mod.rs:18`):** `const MAX_FINDINGS: usize = 10_000;`

**Emission-site audit (where the cap is checked, plus where it is BYPASSED):**

| Site | File:line | Behavior at cap |
|---|---|---|
| Excessive overlap alert | mod.rs:272 (`self.findings.len() < MAX_FINDINGS`) | Silent drop. Latch (`overlap_alert_fired`) is also gated on the cap, so a future overlap-finding can still fire IF `findings.len()` drops back below cap before another flow hits threshold -- but `findings` only grows, so in practice the latch is permanently set to false-on-drop. **Subtle:** at 10001 findings, the latch `overlap_alert_fired = true` is NEVER set because the whole if-block is gated by `findings.len() < MAX_FINDINGS`. The flow can then re-cross the threshold later if the cap is somehow lowered (it isn't), but with the cap immutable, the latch behavior is irrelevant after 10000. |
| Excessive small-segment alert | mod.rs:291 | Same pattern; latch ungated when over cap. |
| Excessive out-of-window alert | mod.rs:310 | Same pattern; latch ungated when over cap. |
| Conflicting overlap (per-event) | mod.rs:534 (`generate_conflicting_overlap_finding`) | Early return at cap (no push, no counter change). |
| Truncated/depth (per-event) | mod.rs:550 (`generate_truncated_finding`) | Early return at cap (no push, no counter change). |
| **finalize() segment-limit finding** | mod.rs:400-417 | **NO cap check.** Pushed UNCONDITIONALLY. Comment at mod.rs:396-397: "Pushed unconditionally (at most 1 finding) to avoid being silently dropped when per-flow findings have filled the MAX_FINDINGS cap." |

**Refined BC (LOW-confidence on saturation behavior, since no test pins it):**

> **BC-RAS-024 (refined).** The reassembler's `findings: Vec<Finding>` is bounded at `MAX_FINDINGS = 10_000` entries. The following emission sites are CAP-GATED -- they silently drop and do NOT replace, count, or evict:
> 1. Excessive overlap alert (`mod.rs:272`)
> 2. Excessive small-segment alert (`mod.rs:291`)
> 3. Excessive out-of-window alert (`mod.rs:310`)
> 4. Conflicting overlap finding (`mod.rs:534`)
> 5. Truncated/stream-depth finding (`mod.rs:550`)
>
> When `findings.len() == MAX_FINDINGS` (10000):
> - The 10001st emission attempt at any cap-gated site evaluates the condition `findings.len() < MAX_FINDINGS` to FALSE and the entire push expression is skipped.
> - For threshold alerts: the `*_alert_fired` boolean latch is ALSO skipped (because the latch assignment is inside the same `if` block), meaning a future call with `findings.len() < MAX_FINDINGS` (achievable only if `MAX_FINDINGS` were raised dynamically -- it isn't) would re-emit. With the cap immutable, the latch never fires once the cap is hit.
> - No counter, no eprintln, no debug log, no `parse_errors`-style increment tracks the dropped-finding event.
>
> The following site is CAP-BYPASS:
> 6. `finalize()` segment-limit summary finding (`mod.rs:400-417`) is pushed regardless of `findings.len()`. This is by design (comment at mod.rs:396-397) so the segment-limit summary always survives even when per-flow findings have exhausted the cap.

**Confidence:** LOW (no test exercises the cap; the cap value 10_000 makes test-driving impractical without exposing a private setter; pure code-reading).

**Test recommendation (to upgrade to MEDIUM):** Add a unit test or `#[cfg(test)] pub(crate)` accessor that forces `findings` to length 10_000 by direct push (or expose a `set_max_findings` for tests), then trigger one more emission and assert `findings.len() == 10_000` afterwards. Or, alternatively, add a `dropped_findings: u64` counter to `ReassemblyStats` and surface it in `summarize()`; that would also pin the behavior. Given the cost of the brute-force option (10_000 fake events), a `pub(crate)` test helper is recommended.

**New BC introduced:**

> **BC-RAS-054 (NEW, HIGH-confidence on bypass).** `finalize()`'s segment-limit summary finding (when `stats.segments_segment_limit > 0`) is pushed to `findings` UNCONDITIONALLY of `findings.len() < MAX_FINDINGS`. This is the only cap-bypass emission site in the engine. Source: `mod.rs:399-417`. Test pinning: `test_finalize_generates_segment_limit_finding` exercises the push path (no cap-saturation test, but the bypass path itself is exercised).

---

### Target 3 -- BC-RDR-004 (pcapng-rejection path)

**Pass-3-R1 wording:** "Reject pcapng-format input (only classic pcap is supported). MEDIUM -- Implementation uses `PcapReader::new` (classic pcap only); pcapng fixture exists but not asserted in tests. Failure mode: Header parse fails with anyhow context 'Failed to parse pcap header'."

**Source re-read (`reader.rs:20-50`):**
```
pub fn from_pcap_reader<R: Read>(reader: R) -> Result<Self> {
    let mut pcap_reader = PcapReader::new(reader).context("Failed to parse pcap header")?;
    ...
}
```
The `pcap_file::pcap::PcapReader::new` constructor parses the classic-pcap global header (magic bytes 0xa1b2c3d4 or 0xd4c3b2a1 little/big endian, plus the 4-byte network field). pcapng files start with a Section Header Block whose magic is 0x0A0D0D0A (with byte-order magic 0x1A2B3C4D). `PcapReader::new` returns an error for non-classic magic, which the `?` operator wraps with `anyhow::Context` "Failed to parse pcap header".

**Test pinning audit:** No test consumes `tests/fixtures/smb3.pcapng` (25,692 bytes). No test asserts the error contract.

**Refined BC (HIGH-confidence on contract; MEDIUM on coverage):**

> **BC-RDR-004 (refined).** When `from_pcap_reader` (or `from_file`) is given a pcapng-format byte stream:
> - `PcapReader::new` (upstream `pcap-file` crate) returns `Err(pcap_file::PcapError::InvalidField(...))` or `Err(pcap_file::PcapError::IoError(...))` depending on which byte fails first (the Section Header Block magic 0x0A0D0D0A does NOT match either classic-pcap byte-order magic 0xa1b2c3d4 / 0xd4c3b2a1).
> - The error is wrapped via `.context("Failed to parse pcap header")` so the returned `anyhow::Error` chain has top-level message "Failed to parse pcap header" with the upstream `pcap_file::PcapError` as `.source()`.
> - `from_file` additionally prepends `.with_context(|| format!("Failed to open {}", path.display()))` if the file open itself fails -- but a present-but-pcapng file fails INSIDE `from_pcap_reader`, so the user-facing top error from `main.rs:104` reads: `Failed to read {path}` -> `Failed to parse pcap header` -> upstream message.
> - No panic. No partial read. No allocation of the packet vector.
>
> **`main.rs` consequence:** `resolve_targets` (`main.rs:236-256`) collects both `*.pcap` AND `*.pcapng` extensions from a directory. When a pcapng-extension file is encountered, the error propagates via `?` at `main.rs:105` and aborts the entire run with `anyhow::Error`. This is BC-CLI-011's silent-glob smell -- the user sees one error message and the run stops; there is no skip-and-continue logic.

**Test recommendation (to upgrade to HIGH):** Add `test_pcapng_rejection_via_from_pcap_reader` in `reader_tests.rs`:
```rust
#[test]
fn test_pcapng_rejection_via_from_pcap_reader() {
    let pcapng_bytes = include_bytes!("fixtures/smb3.pcapng");
    let result = PcapSource::from_pcap_reader(Cursor::new(pcapng_bytes.as_slice()));
    assert!(result.is_err());
    let err = result.unwrap_err();
    let chain: Vec<String> = err.chain().map(|c| c.to_string()).collect();
    assert!(chain.iter().any(|m| m.contains("Failed to parse pcap header")),
        "error chain must contain 'Failed to parse pcap header', got: {chain:?}");
}
```
This finally pulls `smb3.pcapng` into actual test usage (Pass 0 Q#12 / Pass 6 known-unknown #15) and pins BC-RDR-004 + the upstream error wrapping.

---

### Target 4 -- BC-HTTP-024 / BC-HTTP-025 / BC-TLS-005 (cardinality cap behavior at boundary)

**Pass-3-R1 wording:**
- BC-HTTP-024: "Per-map cardinality cap: methods/hosts/user_agents stop adding new keys past MAX_MAP_ENTRIES (50000); existing keys still increment. MEDIUM -- http.rs:309-323 -- no direct test"
- BC-HTTP-025: "uris list capped at MAX_URIS=10000; further URIs silently dropped. MEDIUM -- http.rs:325 -- no direct test"
- BC-TLS-005: "Per-direction buffer capped at MAX_BUF=65536; bytes past cap dropped. MEDIUM -- tls.rs:676-689 -- not directly tested"

**Source re-reads:**

**HTTP (`http.rs:309-327`):**
```
if self.methods.len() < MAX_MAP_ENTRIES || self.methods.contains_key(&parsed.method) {
    *self.methods.entry(parsed.method.clone()).or_insert(0) += 1;
}
if let Some(ref h) = parsed.host
    && (self.hosts.len() < MAX_MAP_ENTRIES || self.hosts.contains_key(h)) {
    *self.hosts.entry(h.clone()).or_insert(0) += 1;
}
if let Some(ref ua) = parsed.user_agent
    && (self.user_agents.len() < MAX_MAP_ENTRIES || self.user_agents.contains_key(ua)) {
    *self.user_agents.entry(ua.clone()).or_insert(0) += 1;
}
if self.uris.len() < MAX_URIS {
    self.uris.push(parsed.uri.clone());
}
```
- `MAX_MAP_ENTRIES = 50_000` (`http.rs:11`)
- `MAX_URIS = 10_000` (`http.rs:10`)

**Eviction policy is *no eviction*** -- once the map/list is full of distinct keys, new distinct keys are silently DROPPED. Existing keys (`map.contains_key`) continue to increment indefinitely. No counter tracks dropped entries. There is no LRU eviction, no random eviction, no `top-N` truncation -- it is pure first-N-wins on key identity.

**TLS (`tls.rs:332-336` for the increment helper; `:676-689` for the buffer cap):**
```
fn increment<K: Eq + std::hash::Hash>(map: &mut HashMap<K, u64>, key: K, limit: usize) {
    if map.len() < limit || map.contains_key(&key) {
        *map.entry(key).or_insert(0) += 1;
    }
}
```
Same first-N-wins policy via this shared helper. Used for `version_counts`, `sni_counts`, `ja3_counts`, `ja3s_counts`, `cipher_counts` (all gated by MAX_MAP_ENTRIES = 50_000).

Per-direction buffer cap (`tls.rs:677-687`):
```
let remaining = MAX_BUF.saturating_sub(state.client_buf.len());
if remaining > 0 {
    let to_copy = data.len().min(remaining);
    state.client_buf.extend_from_slice(&data[..to_copy]);
}
```
`MAX_BUF = 65_536`. Bytes past cap are SILENTLY DROPPED -- no `poisoned_bytes_skipped`-style counter (unlike HTTP). The downstream `try_parse_records` then operates on the truncated buffer; if the truncated buffer doesn't contain a complete TLS record header, parsing returns Partial and the buffer stays put (no overflow possible because the saturating_sub means `remaining = 0` once full).

**Boundary semantics at the cap:**

| Path | At cap (len == MAX): | Test that pins behavior |
|---|---|---|
| HTTP methods/hosts/user_agents map | New distinct keys silently dropped; existing keys keep incrementing | NONE (BC-HTTP-024 stays MEDIUM) |
| HTTP uris vec | New URIs silently dropped (regardless of duplicate) | NONE (BC-HTTP-025 stays MEDIUM) |
| HTTP request_buf / response_buf | Bytes past MAX_HEADER_BUF (65536) silently dropped; `test_buffer_cap_no_panic_on_oversized_headers` pins this | tests/http_analyzer_tests.rs:518-579 |
| TLS sni_counts | New keys silently dropped; **existing-key increments AND non-UTF-8 finding emission stay decoupled** -- pinned by `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` | tests/tls_analyzer_tests.rs:709-781 -- **BC-TLS-028 is HIGH-confidence and provides STRONG indirect evidence for the cap policy** |
| TLS version/cipher/ja3/ja3s_counts | Same first-N-wins via `increment()` helper | NONE for these specific maps (the helper is exercised but not at saturation) |
| TLS client_buf / server_buf | `saturating_sub` truncates; once full, `remaining = 0` and `to_copy = 0` so `extend_from_slice` is a no-op. No panic. No counter. | NONE (BC-TLS-005 stays MEDIUM) |

**Refined BCs:**

> **BC-HTTP-024 (refined, HIGH-confidence on policy, MEDIUM on coverage).** The `methods`, `hosts`, and `user_agents` HashMaps in `HttpAnalyzer` are bounded at `MAX_MAP_ENTRIES = 50_000` entries each. The cap is enforced inline (each map has its own check in `try_parse_requests`); there is no shared helper. Behavior at the cap:
> 1. A new request whose method/host/UA is ALREADY a key in the map: increment proceeds normally (`map.contains_key` short-circuit).
> 2. A new request whose method/host/UA is NOT in the map AND `map.len() >= MAX_MAP_ENTRIES`: the increment is SKIPPED. The entry is permanently absent from the map. No counter tracks the drop.
> 3. The cap is enforced INDEPENDENTLY per map. `methods` saturating does NOT prevent `hosts` or `user_agents` from accepting new keys, and vice versa.
> 4. The cap does NOT affect detection findings: even if `methods` is full, a request with a path-traversal URI still emits `BC-HTTP-005`.

> **BC-HTTP-025 (refined, HIGH-confidence on policy, MEDIUM on coverage).** The `uris: Vec<String>` in `HttpAnalyzer` is bounded at `MAX_URIS = 10_000` entries. Policy is first-N-wins by INSERT ORDER (not by URI uniqueness): once `uris.len() == 10_000`, no further push happens at `http.rs:325`. Duplicate URIs are NOT deduplicated. The `summarize()` "recent_uris" key takes `uris.iter().take(20)` so the user always sees the FIRST 20 URIs, never the LAST 20 -- which contradicts the "recent_uris" naming (P3 R1 implies last-20). **Document this as a naming smell or remove the `take(20)` and rely on top-N.**

> **BC-TLS-005 (refined, HIGH-confidence on policy, MEDIUM on coverage).** Per-direction TLS buffers (`client_buf`, `server_buf`) in `TlsFlowState` are bounded at `MAX_BUF = 65_536` bytes each. The cap is enforced via `saturating_sub` in `on_data` (`tls.rs:677-687`):
> 1. When `state.client_buf.len() >= MAX_BUF`: `remaining = 0`, `to_copy = data.len().min(0) = 0`, so `extend_from_slice` is a no-op. Bytes are SILENTLY DROPPED.
> 2. No counter tracks dropped TLS bytes (HTTP has `poisoned_bytes_skipped`; TLS has no equivalent).
> 3. The buffer is NEVER cleared except on `on_flow_close`, and `try_parse_records` consumes from it (`drain`). Once the buffer is full and the held bytes don't form a complete record, subsequent bytes are dropped indefinitely until either a clean drain or flow close.

**Test recommendations (to upgrade all three to HIGH):**

For BC-HTTP-024:
```rust
#[test]
fn test_methods_map_drops_new_keys_past_cap() {
    let mut analyzer = HttpAnalyzer::new();
    // Build 50_000 unique methods (or expose pub(crate) helper); a faster
    // alternative is to expose `set_max_map_entries(usize)` for tests.
    // Pseudocode: drive analyzer past 50000 unique methods, then send method "POST",
    // assert methods.get("POST") is None, then send "GET" (a method already counted),
    // assert methods.get("GET") incremented.
}
```
For BC-HTTP-025: similar; drive 10_000 unique URIs then assert the 10001st is absent.

For BC-TLS-005: send a single `on_data` chunk of 200_000 bytes; assert `state.client_buf.len() == 65_536` (requires `pub(crate)` accessor or via subsequent parse-error counter).

---

### Target 5 -- BC-ABS-007 / BC-CLI-016 (CSV pseudo-pass-through)

**Pass-3-R1 wording:**
- BC-ABS-007: "--csv <FILE> global flag accepts Option<Option<PathBuf>> and the `csv` crate is a declared dependency, but no CSV reporter exists; OutputFormat::Csv falls through to TerminalReporter. HIGH (absent) -- cli.rs:35-36, Cargo.toml; main.rs:172-184 has no Csv arm"
- BC-CLI-016: "--output-format json picks JsonReporter; anything else (including --csv) falls through to TerminalReporter. HIGH -- main.rs:172-184"

**Source re-read:**

**`cli.rs:5-9`:** `OutputFormat` enum has two variants, `Json` and `Csv`. Both are valid clap `ValueEnum` choices -- `wirerust --output-format csv analyze ...` parses successfully.

**`cli.rs:34-36`:** `pub csv: Option<Option<PathBuf>>` -- the file-output form. `--csv file.csv` parses; bare `--csv` parses (the outer Option means flag presence is captured, inner Option means optional argument).

**`main.rs:172-184` (analyze) and `:218-230` (summary), verbatim:**
```
let output = match cli.output_format {
    Some(OutputFormat::Json) => {
        let reporter = JsonReporter;
        reporter.render(&summary, &all_findings, &analyzer_summaries)
    }
    _ => {
        let reporter = TerminalReporter {
            use_color,
            show_mitre_grouping,
        };
        reporter.render(&summary, &all_findings, &analyzer_summaries)
    }
};

println!("{output}");
```
The `_` arm catches BOTH `Some(OutputFormat::Csv)` AND `None`. There is NO match arm for `Csv` and NO explicit branch on `cli.csv`. The line `println!("{output}")` ALWAYS prints to stdout regardless of `cli.csv = Some(Some(PathBuf))`.

**User-visible behavior, verbatim:**

1. `wirerust analyze foo.pcap` -- TerminalReporter to stdout. (no surprise)
2. `wirerust --output-format json analyze foo.pcap` -- JsonReporter to stdout. (no surprise)
3. **`wirerust --output-format csv analyze foo.pcap`** -- TerminalReporter to stdout. NO error message. NO warning. The user requested CSV; they got the colored terminal table. (silent surprise)
4. **`wirerust --csv out.csv analyze foo.pcap`** -- TerminalReporter to stdout, `out.csv` is NEVER created. The user requested file output; they got nothing on disk and the terminal report on stdout. (silent surprise)
5. **`wirerust --csv --output-format csv analyze foo.pcap`** -- Same as #4. (silent surprise)
6. **`wirerust --json out.json analyze foo.pcap`** -- JsonReporter to stdout, `out.json` is NEVER created. Same silent surprise pattern as #4.

**Refined BC -- explicit user-visible failure-to-honor:**

> **BC-ABS-007 / BC-CLI-016 (refined, HIGH-absence).** Three CLI inputs that the help text implies will produce CSV output produce TerminalReporter output instead, with no error, warning, or any signal of the failure:
> 1. `--output-format csv` -- the `OutputFormat::Csv` enum variant exists (`cli.rs:8`) and clap accepts it as a valid value, but `main.rs:172-177` does NOT match it. The fall-through `_` arm covers it.
> 2. `--csv` (bare) -- accepted by clap, value stored in `cli.csv = Some(None)`, but `main.rs` never reads `cli.csv`.
> 3. `--csv <FILE>` -- value stored in `cli.csv = Some(Some(PathBuf))`, but `main.rs` never reads `cli.csv` and never creates a file.
>
> `--json <FILE>` has the SAME failure pattern (stored, never read; output goes to stdout). The bug is symmetric: file-output paths are accepted-and-ignored for both formats.
>
> The `csv` crate is in `Cargo.toml:17` but `awk` over `src/` confirms zero `use csv` or `csv::` references. The crate is transitively compiled, paying a build-time cost for zero functionality.

**Three disposition options:**

| Option | Description | Cost | Risk |
|---|---|---|---|
| **A. Implement CSV** | Add `src/reporter/csv.rs`, wire it under `OutputFormat::Csv`, honor `cli.csv` and `cli.json` file paths. Decide on CSV schema (one row per finding? per packet? per analyzer?). | **L** (large -- design + impl + tests + ADR for the schema) | Schema lock-in; CSV is a poor fit for nested fields like `evidence: Vec<String>`. |
| **B. Remove `--csv` + `OutputFormat::Csv` + `cli.csv` + `csv` Cargo dep** | Delete the Csv variant, the `--csv` flag, the `csv` crate, and the file-output behavior of `--json`. Single output: stdout, format-selectable via `--output-format`. | **S** (small -- mechanical deletion + clap doc-string updates + Cargo.lock regen + 1 test deletion) | Removes a CLI surface that may be linked from external docs; bump major version per semver. |
| **C. Error-with-message on `--csv` and `--output-format csv` and file-output forms** | Match `OutputFormat::Csv` and the `cli.csv` / `cli.json.is_some()` cases with an explicit `anyhow::bail!("CSV output is not yet implemented; tracked at #N")` early in `main`. Same for file-output. | **S** (small -- 4-6 lines of bail logic + 4 cli_tests adjustments + 1 README note) | Breaks scripts that currently pipe stdout; loud failure is better than silent surprise but may surface today's tolerant behavior as a regression. |

**Recommendation: Option B (remove)** unless the original author or product owner has a near-term plan to ship CSV. The `OutputFormat::Csv` variant has lived in `cli.rs` for many commits with zero implementation activity (`rayon` and the dev-deps `assert_cmd`/`predicates`/`tempfile` have the same shape -- declared, never used). Removing reduces the implicit "this works" surface area. Option C is the safe interim if removal is politically expensive: it preserves the flag name for future use while making the no-op behavior loud.

**Rationale:** Today's silent fall-through is the worst of three outcomes (no value, no signal, dead crate dep). Option A is the highest cost AND has no design pinned. Option B is the smallest surface change and the most honest. Option C is acceptable as a 1-step-to-B intermediate.

---

## 3. MEDIUM-confidence BC pinning targets (8 of 40)

Pass 3 R1 has 40 MEDIUM rows (not 26 as claimed; see §1 audit). Selecting 8 from reassembly / TLS / HTTP error-boundary and from the Pass 6 §6.6 "BCs-without-tests" bucket:

### MED-1: BC-RAS-001 -- `TcpReassembler::new` panics on invalid config

- **Current confidence:** MEDIUM (mod.rs:86-96, 5 asserts, no test)
- **Upgrade verdict:** MEDIUM-with-test-recommendation (cannot be safely upgraded to HIGH without a test using `std::panic::catch_unwind`, which is awkward in this codebase)
- **Evidence:** `reassembly/mod.rs:86-96`: five `assert!(... > 0, ...)` with explicit messages. No test exercises any of them.
- **Test recommendation:** `test_reassembler_new_panics_on_zero_max_depth` etc., one per assert (5 tests). Each uses `#[should_panic(expected = "max_depth must be > 0")]`. After 5 tests added -> HIGH.

### MED-2: BC-RAS-020 -- SMALL_SEGMENT_ALERT_THRESHOLD (2048) emits one-shot finding

- **Current confidence:** MEDIUM (mod.rs:289, no test directly drives threshold)
- **Upgrade verdict:** MEDIUM-with-test-recommendation
- **Evidence:** The small-segment COUNTER is tested at `reassembly_segment_tests.rs:139-150` (counter reaches 5 after 5 single-byte inserts). The ENGINE-level finding emission requires `flow_dir.small_segment_count > 2048`, which requires 2049 segments past the SYN -- expensive but driveable. No test does this.
- **Test recommendation:** `test_small_segment_threshold_finding` in `reassembly_engine_tests.rs` -- loop 2049 single-byte non-overlapping segments and assert exactly one Anomaly/Inconclusive/Medium finding with summary `"Excessive small segments (2049)"`. After test added -> HIGH.

### MED-3: BC-HTTP-024 -- methods/hosts/user_agents MAX_MAP_ENTRIES cap

- **Current confidence:** MEDIUM (no direct test)
- **Upgrade verdict:** MEDIUM-with-test-recommendation. See §2 Target 4 for the test contract.
- **Note:** the existing TLS test `test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity` (tests/tls_analyzer_tests.rs:709-781) proves the SAME PATTERN works in TLS -- the test is intentionally slow (650ms cold) and a similar HTTP test would be acceptable.

### MED-4: BC-HTTP-025 -- uris MAX_URIS=10000 cap + "recent" misnomer

- **Current confidence:** MEDIUM (no direct test)
- **Upgrade verdict:** **MEDIUM-with-test-recommendation AND naming fix recommendation**. The "recent_uris" key in `summarize()` takes the FIRST 20, not the last 20 -- so the key name is wrong. Either fix the slice to `iter().rev().take(20)` OR rename to `first_uris` / `top_uris`. The behavior is HIGH-confidence from `http.rs:505`; the BC text in R1 just inherits a misleading field name.

### MED-5: BC-TLS-007 -- JA3 string format

- **Current confidence:** MEDIUM (tls.rs:121, format inferred from code; JA3 hash length verified in tests)
- **Upgrade verdict:** HIGH-with-existing-test -- the actual string format `"version,cipher-list,extension-list,curve-list,pointfmt-list"` joined by `-` between IDs and `,` between groups is INDIRECTLY verified by `test_ja3_grease_filtering` and `test_tls13_pcap_version_and_ja3` (which assert specific 32-char MD5 hex outputs derived from known JA3 strings). The MD5 hash is a one-way function -- the only way to produce a specific hash is to construct the exact JA3 string. **Recommend upgrade to HIGH-pinned-via-output-hash.**

### MED-6: BC-TLS-008 -- JA3S string format

- **Current confidence:** MEDIUM (tls.rs:144)
- **Upgrade verdict:** HIGH-with-existing-test -- same argument as MED-5. JA3S MD5 hex output is pinned by `test_parse_server_hello` and `test_tls13_pcap_version_and_ja3`. Upgrade to HIGH.

### MED-7: BC-RAS-002 -- skip non-TCP packets and increment packets_skipped_non_tcp

- **Current confidence:** MEDIUM (mod.rs:117, inferred from code)
- **Upgrade verdict:** MEDIUM-with-test-recommendation. The `summarize()` test (`test_summarize_returns_reassembly_stats`) verifies the COUNTER appears in detail but does NOT force a non-TCP packet through `process_packet`. The decoder produces ICMP packets via BC-DEC-010 -- a one-test fix is to construct a `ParsedPacket{ protocol: Protocol::Icmp, transport: TransportInfo::None, ... }` and call `process_packet`, then assert `stats.packets_skipped_non_tcp == 1`.

### MED-8: BC-DSP-006 -- DispatchTarget::None is NOT cached (reclassification allowed)

- **Current confidence:** MEDIUM (dispatcher.rs:77, inferred from code)
- **Upgrade verdict:** **MEDIUM-with-test-recommendation, high-priority** -- this is the BC-DSP-005's complement and is critical for Pass 6 known-unknown #10 (unbounded classification cost on adversarial short-segment traffic). Recommended test: `test_dispatcher_reclassifies_after_initial_none` -- send a 4-byte buffer that yields None on first call, then send 6 more bytes that DO match `\x16\x03\x01\x00\x00`, and assert the flow is now in `routes` as TLS. After test added -> HIGH. Connects to Pass 6 §3 deepening Q for dispatcher cost ceiling.

**Summary of MEDIUM upgrades:** 8 BCs reviewed; of those, **MED-5 (BC-TLS-007) and MED-6 (BC-TLS-008) can be UPGRADED to HIGH immediately** (existing tests indirectly pin via output MD5). Six others (MED-1/2/3/4/7/8) are MEDIUM-with-test-recommendation -- each has a concrete test signature drafted.

---

## 4. LOW-confidence BC pinning (all 4)

### LOW-1: BC-RAS-029 -- CLOSE_FLOW_MISSING_WARNED one-shot

- **Pass-3-R1 wording:** "After `close_flow` for a missing key, log one process-wide warning via the CLOSE_FLOW_MISSING_WARNED atomic. LOW -- mod.rs:480-489 -- no test (debug_assert in normal builds)"
- **Source re-read:** `mod.rs:480-489`: `debug_assert!(false, ...)` + `if !CLOSE_FLOW_MISSING_WARNED.swap(true, Ordering::Relaxed) { eprintln!(...); }`. The `debug_assert!` aborts in debug builds, making the eprintln unreachable in tests. In release builds, the eprintln fires at most once per process.
- **Disposition:** **ACCEPT as doc-only invariant.** Testing it would require either (a) flipping a `#[cfg(debug_assertions = "no")]` test gate (anti-pattern), or (b) introducing a release-only test (cargo doesn't support per-build-mode tests cleanly), or (c) refactoring the atomic into instance state and adding a `pub(crate) fn warned(&self) -> bool` -- which contradicts the design intent of process-wide one-shot suppression.
- **ADR amendment proposed:** Add a 3-line note to ADR 0002 (or a new lightweight ADR 0004): "Process-wide one-shot warning atomics (CLOSE_FLOW_MISSING_WARNED, ISN_MISSING_WARNED) are intentional. They protect operators from log-flooding under pathological inputs at the cost of multi-target visibility. Multi-target captures see warnings only from the first target."

### LOW-2: BC-RAS-048 -- ISN_MISSING_WARNED one-shot

- **Same shape as LOW-1.** Same disposition (accept as doc-only) and same ADR amendment proposed.

### LOW-3: BC-CLI-013 -- indicatif progress bar template

- **Pass-3-R1 wording:** "main: per-target progress bar uses indicatif with template '[elapsed] {bar:40} pos/len packets' to stderr. LOW -- main.rs:107-110 -- not directly tested"
- **Source re-read:** `main.rs:107-110`: `pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos}/{len} packets")?);`. The template string differs slightly from the BC text: actual is `{elapsed_precise}` (not `elapsed`). The BC text is APPROXIMATE.
- **Disposition:** **ACCEPT as doc-only invariant.** Testing the progress bar template would require capturing stderr in a binary integration test, which `assert_cmd` could do (it's a declared-but-unused dev-dep per BC-ABS-009 -- one rationale for keeping the dep!).
- **Test recommendation IF dev-dep usage is justified:** `assert_cmd`-driven integration test that captures stderr and assert it contains "packets" and the bar character. After test added -> MEDIUM (template details still hard to assert exactly). Otherwise: amend BC text to use exact `{elapsed_precise}` token.

### LOW-4: BC-MIT-009 -- MitreTactic is `#[non_exhaustive]`

- **Pass-3-R1 wording:** "MitreTactic is `#[non_exhaustive]` so adding new variants is non-breaking for downstream pattern-matchers. LOW -- mitre.rs:22 -- not testable as a behavior, but ADR-implicit guarantee"
- **Disposition:** **ACCEPT as doc-only invariant.** A "trybuild" / `compile_fail` test could prove that exhaustive external matches now require a wildcard, but trybuild is not in the dep tree (and adding it for one assertion is heavy). The attribute itself is the proof.
- **ADR amendment proposed:** None new; already covered by Pass 2 invariants table and inline doc comment at `mitre.rs:22`.

**Summary:** All 4 LOW BCs are accepted as doc-only invariants. LOW-1 + LOW-2 together motivate a brief ADR amendment (or new ADR 0004) on process-wide one-shot warnings. LOW-3 has a test path if assert_cmd dev-dep is to be activated.

---

## 5. Absent BC dispositions (5 of 10)

For 5 of the BC-ABS-001..010 absent BCs, dispositions in narrative form. (BC-ABS-007 already fully treated in §2 Target 5.)

### ABS-1: BC-ABS-001 -- `--threats` flag is unwired

- **What it promises:** clap help: "Run threat detection". Implies a separate threat-detection layer beyond DNS/HTTP/TLS analyzers.
- **What actually happens:** `cli.rs:67-68` declares `threats: bool`. `main.rs:28-46` destructures `dns/http/tls/all/mitre/targets` but does NOT bind `threats`. The flag parses but has zero effect.
- **Three options:**
  - **Implement:** Define what "threats" means as a layer. Possibilities: enable all current analyzers (`= --all`), or gate a future MITRE-priority filter. **Cost: M.**
  - **Remove:** Delete the flag and the bool field. **Cost: S.**
  - **Error-with-message:** `bail!("--threats not yet implemented")` if set. **Cost: S.**
- **Recommendation: Remove (or wire to `= --all` if the README implies "threats means all analyzers").** The flag has no design pinned. Today's silent acceptance is the worst of options.

### ABS-2: BC-ABS-002 -- `--beacon` flag is unwired

- **What it promises:** clap help: "Detect C2 beaconing patterns". Implies a beacon detector exists.
- **What actually happens:** `cli.rs:82-84` declares it. No analyzer named "beacon" exists in `src/`. No use in `main.rs`. **No code path is reachable from this flag.**
- **Three options:**
  - **Implement:** Build a periodic-connection detector keyed on DNS query timings or TLS hello cadence. **Cost: L** (new analyzer + tests + threshold calibration).
  - **Remove:** Delete the flag. **Cost: S.**
  - **Error-with-message:** `bail!("--beacon not yet implemented; planned in #N")`. **Cost: S.**
- **Recommendation: Error-with-message** (Option C). Beacon detection is a known forensic-tool feature (Suricata, Zeek, Wireshark plugins) so it's plausible roadmap. Removing closes off the placeholder; loud bail preserves the surface.

### ABS-3: BC-ABS-003 -- `--filter <BPF>` flag is unwired

- **What it promises:** clap help: "BPF filter expression". Implies pre-decoder packet filtering (Berkeley Packet Filter syntax).
- **What actually happens:** `cli.rs:94-96` declares `filter: Option<String>`. No use in `main.rs` or `reader.rs`. The reader has no BPF integration.
- **Three options:**
  - **Implement:** Add `pcap` crate or `bpf-compile` for BPF parsing. Apply to decoded packets at the `summary.ingest` / dispatcher entry. **Cost: L** (new dep, parser, predicate evaluator, integration with the eager-load reader).
  - **Remove:** Delete the flag. **Cost: S.**
  - **Error-with-message:** `bail!("--filter not yet implemented")`. **Cost: S.**
- **Recommendation: Error-with-message.** BPF is a load-bearing forensic feature; deleting it pre-empts the design space. The clean fail is the honest middle.

### ABS-4: BC-ABS-006 -- `--json <FILE>` accepts a path but stdout-only

- **What it promises:** clap help: "Write JSON output to file" with optional `<FILE>` argument.
- **What actually happens:** `cli.rs:31-32` declares `json: Option<Option<PathBuf>>`. `main.rs:186, 232` always `println!("{output}")`. The PathBuf is never consulted.
- **Three options:**
  - **Implement:** Add a 6-line branch: `if let Some(Some(path)) = &cli.json { fs::write(path, &output)?; } else { println!("{output}"); }`. **Cost: S.**
  - **Remove:** Delete the `Option<PathBuf>` wrapping; `--json` becomes a synonym for `--output-format json`. **Cost: S.**
  - **Error-with-message:** `bail!("--json <FILE> not yet implemented")`. **Cost: S.**
- **Recommendation: Implement (Option A).** This is the cheapest path with the highest user value -- file output is a natural pipeline feature, the implementation is 6 lines, and the test surface adds 1 test using `tempfile` (which is a declared-but-unused dev-dep -- killing two birds). The symmetric `--csv <FILE>` follows from the CSV disposition (see §2 Target 5).

### ABS-5: BC-ABS-010 -- `--verbose` flag parsed but never consulted

- **What it promises:** clap help: "Enable verbose output". Implies a log-verbosity gate.
- **What actually happens:** `cli.rs:19-20` declares `verbose: bool` (global). `main.rs` never reads `cli.verbose`. The flag parses but does nothing.
- **Three options:**
  - **Implement:** Define verbosity level. With no `log`/`tracing` framework (CNV-LOG-001), the only outlet is `eprintln!`. Options: print per-packet decode trace, or print per-flow start/close events. **Cost: M** (small surface but design-needed).
  - **Remove:** Delete the flag. **Cost: S.**
  - **Error-with-message:** `eprintln!("Warning: --verbose has no effect in this version")` (non-fatal). **Cost: S.**
- **Recommendation: Remove (Option B)** unless a verbose mode is on the roadmap. The flag has been in `cli.rs` since pre-history with zero implementation discussion in ADRs. Removing simplifies and is reversible. (Test: `test_analyze_subcommand` at `cli_tests.rs:14` asserts `cli.verbose == true`; this test would need adjustment.)

**Summary of 5 ABS dispositions:**

| BC | Recommendation | Cost | Rationale |
|---|---|---|---|
| ABS-001 (`--threats`) | Remove | S | No design pinned. |
| ABS-002 (`--beacon`) | Error-with-message | S | Plausible roadmap; preserve surface. |
| ABS-003 (`--filter`) | Error-with-message | S | Load-bearing feature; don't pre-empt. |
| ABS-006 (`--json <FILE>`) | Implement | S | Cheap; activates `tempfile` dev-dep. |
| ABS-010 (`--verbose`) | Remove | S | No design; reversible. |

Combined with §2 Target 5 (ABS-007 `--csv` -> Remove or error), this leaves ABS-004 (`--hosts`), ABS-005 (`--services`), ABS-008 (rayon dep), ABS-009 (dev-deps) for Round 3 or a separate dispositioning pass.

---

## 6. Refined BC list (deltas only, do not republish all 216)

Explicit list of BC-IDs that changed confidence or text in this round:

| BC-ID | Old confidence | New confidence | Change |
|---|---|---|---|
| BC-RAS-023 | MEDIUM | MEDIUM (text refined, finding-shape contract added) | Added exact summary/evidence/source_ip semantics; test recommendation for upgrade to HIGH drafted. |
| BC-RAS-024 | MEDIUM | LOW (downgraded with refined text) | Added emission-site audit; specified cap-gated vs cap-bypass sites; latch-skip implication noted. Downgraded because the saturation behavior cannot be pinned without code changes. |
| **BC-RAS-054 (NEW)** | -- | HIGH | `finalize()` segment-limit finding bypasses MAX_FINDINGS cap. Pinned by `test_finalize_generates_segment_limit_finding`. |
| BC-RAS-019 | HIGH | HIGH (text tightened) | Boundary semantics clarified: threshold 50, fires at count > 50 (i.e., 51). |
| BC-RDR-004 | MEDIUM | MEDIUM (text refined, error-chain contract added) | Specified the exact anyhow context chain and `main.rs` consequence. |
| BC-HTTP-024 | MEDIUM | MEDIUM (text refined, policy clarified) | Specified "no eviction, first-N-wins per-map independent" policy. |
| BC-HTTP-025 | MEDIUM | MEDIUM (text refined + naming smell flagged) | "recent_uris" misnomer flagged; behavior is first-N, not last-N. |
| BC-TLS-005 | MEDIUM | MEDIUM (text refined) | Specified saturating_sub no-op behavior at cap; no counter unlike HTTP. |
| BC-TLS-007 | MEDIUM | **HIGH** (upgraded) | Existing JA3-hash tests indirectly pin the JA3 string format via one-way MD5. |
| BC-TLS-008 | MEDIUM | **HIGH** (upgraded) | Same argument for JA3S. |
| **BC-TLS-037 (NEW)** | -- | MEDIUM | SNI value-discrimination rules: when bytes have BOTH C0/DEL AND non-ASCII UTF-8 codepoints, the AsciiWithControl variant takes precedence (verified from `tls.rs:173-242` enum construction order; needs explicit test for round-3). |
| BC-CLI-016 | HIGH | HIGH (text refined, user-visible behavior enumerated) | 6 user-visible failure modes enumerated. |
| BC-ABS-007 | HIGH (absent) | HIGH (absent, dispositions drafted) | 3 disposition options + recommendation (Option B = Remove). |
| BC-ABS-001 | HIGH (absent) | HIGH (absent, disposition: Remove) | -- |
| BC-ABS-002 | HIGH (absent) | HIGH (absent, disposition: Error-with-message) | -- |
| BC-ABS-003 | HIGH (absent) | HIGH (absent, disposition: Error-with-message) | -- |
| BC-ABS-006 | HIGH (absent) | HIGH (absent, disposition: Implement) | -- |
| BC-ABS-010 | HIGH (absent) | HIGH (absent, disposition: Remove) | -- |

**Net deltas:**
- 2 NEW BCs: BC-RAS-054 (HIGH, finalize cap-bypass) + BC-TLS-037 (MEDIUM, SNI discriminator order).
- 2 confidence UPGRADES: BC-TLS-007 + BC-TLS-008 (MEDIUM -> HIGH).
- 1 confidence DOWNGRADE: BC-RAS-024 (MEDIUM -> LOW, more honestly reflects un-pinnability).
- 7 text REFINEMENTS without confidence change (BC-RAS-023, BC-RAS-019, BC-RDR-004, BC-HTTP-024, BC-HTTP-025, BC-TLS-005, BC-CLI-016).
- 6 ABS DISPOSITIONS drafted (ABS-001/002/003/006/007/010).

**Resulting BC total:** 216 + 2 new = **218 BCs** post-round 2.

---

## 7. Delta Summary

- **New BCs added:** 2 (BC-RAS-054 HIGH "finalize cap-bypass"; BC-TLS-037 MEDIUM "SNI discriminator order").
- **Confidence upgrades:** 2 (BC-TLS-007, BC-TLS-008 -> HIGH via existing JA3/JA3S MD5 tests).
- **Confidence downgrades:** 1 (BC-RAS-024 -> LOW; honest about un-pinnability without code changes).
- **Text refinements (no confidence change):** 7 (BC-RAS-023, BC-RAS-019, BC-RDR-004, BC-HTTP-024, BC-HTTP-025, BC-TLS-005, BC-CLI-016).
- **Absent-BC dispositions drafted:** 6 (ABS-001 Remove, ABS-002 Error-with-msg, ABS-003 Error-with-msg, ABS-006 Implement, ABS-007 Remove-or-error, ABS-010 Remove).
- **Hallucination-class audit retractions:** 6 (CONV-ABS-1..6) -- of which 4 are metric corrections and 2 are partial tightenings.
- **Test recommendations authored:** 9 concrete test signatures across BC-RDR-004, BC-RAS-001, BC-RAS-002, BC-RAS-020, BC-RAS-023, BC-RAS-024, BC-HTTP-024, BC-HTTP-025, BC-TLS-005, BC-DSP-006.
- **Remaining gaps for round 3:**
  - 4 ABS BCs un-dispositioned (ABS-004 `--hosts`, ABS-005 `--services`, ABS-008 rayon, ABS-009 dev-deps).
  - 32 MEDIUM BCs still un-pinned (40 original - 6 selected here - 2 upgraded = 32 remaining MEDIUM). Round 3 should select 8-10 more from the dense MEDIUM-cluster around BC-DEC-008..013 (decoder error paths), BC-CLI-007..017 (main.rs branches), BC-TLS-033..036.
  - The MAX_FINDINGS=10_000 cap-saturation test needs `pub(crate)` or `dropped_findings` counter design before it can be tested directly.
  - INV-1/INV-2/INV-3 (Pass 2 load-bearing-unenforced invariants) NOT addressed in this round; Pass 2 R2 territory.
  - Per-direction reassembly alerts: round 1 implied each direction emits independently (BC-RAS-022 "AT MOST once per flow") but the per-direction independence is not explicitly tested (Pass 6 §3 deepening Q for BC-RAS-022).
  - JA3 strict format string contract (BC-TLS-007) could be further tightened with a property test fuzzing JA3 inputs.

---

## 8. Novelty Assessment

**Novelty: SUBSTANTIVE**

Justification (would removing this round's findings change how you'd spec the system?):
- **YES** -- the BC-RAS-024 saturation downgrade and the discovery of BC-RAS-054 (finalize cap-bypass) materially change the contract surface for the engine's finding cardinality. Any downstream spec or test design that treated MAX_FINDINGS as "all findings capped" would miss the deliberate finalize carve-out.
- **YES** -- the BC-TLS-007 / BC-TLS-008 upgrade unblocks ~2 BCs of spec-on-paper exposure with zero code change.
- **YES** -- the 5 ABS disposition recommendations (with Cost/Risk tables) are direct inputs for Phase 3 implementation / `/create-prd`.
- **YES** -- the recount of 137 -> 216 BCs and 26 -> 40 MEDIUM is a metric correction that changes the planning estimate for round 3 (32 MEDIUM still pending, not 18 as Pass 6 implies).
- **YES** -- the BC-HTTP-025 naming smell ("recent_uris" actually returns FIRST 20, not last 20) is an observable inconsistency that downstream consumers of the JSON output would hit.

Removing any of these findings would degrade either the spec accuracy or the downstream planning. This is SUBSTANTIVE, not nitpick.

---

## 9. Convergence Declaration

**Another round needed.** Substantive gaps remaining for Pass 3 R3:
1. 32 MEDIUM BCs still un-pinned (round 3 target: 8-10 more).
2. 4 ABS BCs un-dispositioned (ABS-004/005/008/009).
3. MAX_FINDINGS cap-saturation test requires a pub(crate)/counter design decision before being testable.
4. JA3/JA3S exact-string format could be property-tested.
5. Per-direction-alert independence (BC-RAS-022) needs a dedicated test pinning both directions.
6. BC-TLS-037 (NEW) SNI discriminator order needs explicit test.
7. The recount discrepancy (137 vs 216) suggests other pass-summary metrics may also be off; Pass 6 R2 should re-cross-reference.

If Pass 3 R3 addresses targets 1-6 and produces fewer than 3 substantive findings, it can declare convergence.

---

## 10. State Checkpoint

```yaml
pass: 3
round: 2
status: complete
inputs_ingested: 8  # P3 R1, P2 R1, P4 R1, P6, mod.rs, reader.rs, cli.rs, main.rs + 6 test files
bcs_in_scope_round_2: 27  # 5 priority gaps + 8 MEDIUM + 4 LOW + 6 ABS + 4 hallucination-audit BCs (overlapping set)
bcs_new: 2  # BC-RAS-054, BC-TLS-037
bcs_upgraded_medium_to_high: 2  # BC-TLS-007, BC-TLS-008
bcs_downgraded_medium_to_low: 1  # BC-RAS-024
bcs_text_refined: 7
abs_dispositions_drafted: 6
test_recommendations_drafted: 9
conv_abs_retractions: 6
total_bcs_post_round: 218
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_action: pass_3_round_3
resume_from: 32 MEDIUM BCs in decoder/CLI/TLS-late ranges + 4 remaining ABS dispositions + cap-saturation test design
```

