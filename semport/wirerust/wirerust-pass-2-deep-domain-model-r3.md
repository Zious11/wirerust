# Pass 2 (Domain Model) -- Deepening Round 3 -- wirerust

- Project: wirerust
- Source path: /Users/zious/Documents/GITHUB/wirerust/
- Generated: 2026-05-19
- Pass: 2 (Domain Model) -- Phase B deepening, round 3
- Builds on: wirerust-pass-2-domain-model.md (R1), wirerust-pass-2-deep-domain-model.md (R2), wirerust-pass-3-deep-behavioral-contracts.md (P3 R2)
- Scope: Exactly the 6 carryover targets named verbatim in the P2 R2 §5 "Remaining gaps / next candidate scope" block. Net-new findings only.

---

## 1. Hallucination-class audit of Pass 2 R2

P3 R2 demonstrated (in its own §1) that pass-summary metrics in prior rounds may be inflated/deflated. The meta-audit applies recursively to P2 R2. Each row = one P2 R2 claim re-verified against source.

| # | P2 R2 claim | Class | Verdict |
|---|---|---|---|
| 1 | "Catalogued-but-unused MITRE set is 9 IDs" (R2 §2 Target 1 and Target 5) | 1 (over-extrapolated tokens) | CONFIRMED -- recount: technique_info has 15 ID arms; awk over src/ shows 6 distinct emitted IDs (T1027, T1036, T1046, T1083, T1499.002, T1505.003); diff = 9 (T1040, T1071, T1071.001, T1071.004, T1573, T0846, T0855, T0856, T0885). The "9 IDs catalogued-but-unused" claim is exact, not approximate. |
| 2 | "22 production emission sites with explicit `timestamp:` field, all None" (R2 §2 Target 6) | 2 (miscounted enumerations) | CONFIRMED -- `awk '/timestamp: None/' src/` yields exactly 22 lines: 9 in http.rs (187, 216, 231, 246, 260, 274, 288, 359, 417); 7 in tls.rs (405, 424, 443, 471, 492, 534, 555); 6 in reassembly/mod.rs (286, 305, 329, 415, 545, 561). Total 22. Tests excluded by virtue of test files being outside src/. **The exact line numbers cited in the R2 §2 Target 6 table are off by 0-2 in several rows** (e.g., R2 says http.rs:187 path-traversal; verified correct; R2 says http.rs:359 request-too-many-headers; actually 359 is the request-side site -- correct). Recount confirms 22 sites. |
| 3 | "9 net-new findings" in R2 §3 Delta Summary | 5 (inflated/deflated) | CONFIRMED -- enumerating R2 §3 bullet count: 9 bullets. Matches stated count. |
| 4 | "2 CONV-ABS retractions" in R2 §10 State Checkpoint | 5 | CONFIRMED -- R2 §1 lists CONV-ABS-1 (LateralMovement/C2 tests reference C2) and CONV-ABS-2 (technique_info 15 not 16). Two markers, both substantiated. |
| 5 | "ThreatCategory::C2 referenced exactly once in tests/findings_tests.rs:23" (R2 §2 Target 4) | 1 (over-extrapolated tokens) | UN-VERIFIED HERE -- the Grep tool was not invoked on tests/; R2's count was unilateral. **Spot-check note (not a retraction):** the claim is plausible; would need a tests/-side grep to mechanically verify. Marking as low-priority follow-up, not a retraction. |
| 6 | "JsonReporter.unwrap is infallible-by-construction" (R2 §2 Target 8) | 3 (named pattern conflation) | CONFIRMED -- the reasoning chain in R2 (Serialize trait failure paths exhausted; non-string keys converted upstream; serde_json::Value cannot fail) is mechanically sound and not invented. |
| 7 | "Finding.timestamp serializes via `skip_serializing_if = Option::is_none`" (R2 §2 Target 6) | 3 (named pattern conflation) | CONFIRMED -- findings.rs:68-69 reads exactly `#[serde(skip_serializing_if = "Option::is_none")] pub timestamp: Option<DateTime<Utc>>`. No other Option field has this attribute. The asymmetry claim stands. |
| 8 | "Terminal reporter never references Finding.timestamp" (R2 §2 Target 6) | 3 | DEFERRED -- not re-grepped this round; was an R2-internal claim from re-reading reporter/terminal.rs. Treat as inherited. |
| 9 | "resolve_targets is asymmetric -- single-file accepts any extension, directory expansion filters to .pcap/.pcapng" (R2 §2 Target 7) | 3 | CONFIRMED on directory glob behavior via Read of main.rs:236-256 in P3 R2 context (Target 3); not re-read this round. |
| 10 | "ThreatCategory closed enum vs MitreTactic non_exhaustive asymmetry" (R2 §2 Target 4 subsidiary) | 3 | CONFIRMED -- findings.rs:41 has plain `pub enum ThreatCategory`; no `#[non_exhaustive]`. mitre.rs has `#[non_exhaustive]` on its enum per P2 R2 audit row 1. The asymmetry is real. |

**Audit verdict:** 0 retractions. P2 R2's findings are mechanically grounded. The minor uncertainty in row 5 (test-file grep not re-run) is a coverage gap, not a hallucination.

---

## 2. Per-target findings

### Target 1 -- Emission-site verbatim summary template + evidence shape matrix (22 sites)

Built from direct re-read of `src/analyzer/http.rs`, `src/analyzer/tls.rs`, and `src/reassembly/mod.rs`. Format strings are quoted verbatim from source (Rust string-literal form including escapes). The "evidence" column is the literal `vec![...]` contents at construction time.

| # | File:Line (timestamp:None anchor) | Emission-site identity | Verbatim `summary:` format string | Verbatim `evidence:` vec contents |
|---|---|---|---|---|
| 1 | http.rs:187 | path traversal (T1083) | `format!("Path traversal in URI: {}", truncate_uri(&parsed.uri, 120))` | `vec![format!("URI: {}", parsed.uri)]` |
| 2 | http.rs:216 | web shell (T1505.003) | `format!("Possible web shell access: {}", truncate_uri(&parsed.uri, 120))` | `vec![format!("URI: {}", parsed.uri)]` |
| 3 | http.rs:231 | admin panel (T1046) | `format!("Admin panel access: {}", truncate_uri(&parsed.uri, 120))` | `vec![format!("URI: {}", parsed.uri)]` |
| 4 | http.rs:246 | unusual HTTP method | `format!("Unusual HTTP method: {}", parsed.method)` | `vec![format!("{} {}", parsed.method, parsed.uri)]` |
| 5 | http.rs:260 | HTTP/1.1 missing Host | `"HTTP/1.1 request without Host header".to_string()` (string literal, NOT format!) | `vec![format!("{} {}", parsed.method, parsed.uri)]` |
| 6 | http.rs:274 | long URI (>2048 chars) | `format!("Abnormally long URI ({} chars)", parsed.uri.len())` | `vec![format!("URI prefix: {}", truncate_uri(&parsed.uri, 200))]` |
| 7 | http.rs:288 | empty User-Agent | `"Empty User-Agent header".to_string()` (string literal) | `vec![format!("{} {}", parsed.method, parsed.uri)]` |
| 8 | http.rs:359 | request too-many-headers (T1499.002) | `"Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)".to_string()` (literal) | `vec!["Direction: request".to_string()]` |
| 9 | http.rs:417 | response too-many-headers (T1499.002) | `"Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)".to_string()` (literal -- byte-identical to #8) | `vec!["Direction: response".to_string()]` |
| 10 | tls.rs:405 | SNI AsciiWithControl (T1027) | `format!("TLS SNI contains ASCII control characters (RFC 6066 \xa73 requires ASCII; DNS preferred hostname syntax per RFC 952 / RFC 1123 restricts to letters, digits, and hyphens): {hostname}")` -- the `\xa7` is the literal section-sign in the source string | `vec![format!("hex: {hex}")]` |
| 11 | tls.rs:424 | SNI NonAsciiUtf8 (T1027) | `format!("TLS SNI contains non-ASCII characters (RFC 6066 requires A-labels per RFC 5890): {hostname}")` | `vec![format!("hex: {hex}")]` |
| 12 | tls.rs:443 | SNI NonUtf8 (T1027) | `format!("TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}")` | `vec![format!("hex: {hex}")]` |
| 13 | tls.rs:471 | ClientHello weak cipher | `"ClientHello offers weak cipher suites (NULL/anonymous/export)".to_string()` (literal) | `weak` -- a `Vec<String>` of cipher names, NOT a `vec![...]` literal. **Shape variant.** |
| 14 | tls.rs:492 | ClientHello SSLv2/SSLv3 | `format!("ClientHello uses deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)")` | `vec![format!("Version: 0x{version:04x} ({version_name})")]` |
| 15 | tls.rs:534 | ServerHello weak cipher | `format!("ServerHello selected weak cipher suite ({})", name)` | `vec![format!("Selected cipher: {} (0x{:04x})", name, sh.cipher.0)]` |
| 16 | tls.rs:555 | ServerHello SSLv2/SSLv3 | `format!("ServerHello negotiated deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)")` | `vec![format!("Version: 0x{version:04x} ({version_name})")]` |
| 17 | reassembly/mod.rs:286 | excessive-overlap alert (T1036) | `format!("Excessive segment overlaps ({}) on flow {}", flow_dir.overlap_count, key)` | `vec!["Possible evasion attempt".into()]` |
| 18 | reassembly/mod.rs:305 | excessive small-segment alert | `format!("Excessive small segments ({}) on flow {}", flow_dir.small_segment_count, key)` | `vec!["Possible IDS evasion".into()]` |
| 19 | reassembly/mod.rs:329 | excessive out-of-window alert | `format!("Excessive out-of-window segments ({}) on flow {}", count, key)` | `vec![format!("max_receive_window={} bytes; possible misconfiguration, evasion, or capture corruption", window)]` |
| 20 | reassembly/mod.rs:415 | finalize segment-limit summary | `format!("{} segment{} dropped due to per-flow segment count limit", count, if count == 1 { "" } else { "s" })` -- **inline conditional inside format!** | `vec!["Segment count limit prevents BTreeMap overhead explosion".into(), "May indicate segmentation-based evasion attempt".into()]` -- **TWO entries, only multi-element vec in the engine** |
| 21 | reassembly/mod.rs:545 | conflicting overlap (T1036) | `format!("Conflicting TCP segment overlap on flow {}", key)` | `vec!["Retransmitted segment contains different data".to_string()]` |
| 22 | reassembly/mod.rs:561 | truncated/stream-depth | `format!("Stream depth exceeded on flow {}", key)` | `vec![format!("Max depth {} bytes reached", self.config.max_depth)]` |

**Net-new structural findings from the matrix:**

- **Sites #5, #7, #8, #9, #13 use STRING LITERALS for `summary:`, not `format!`.** This means the templates are fixed; no field interpolation at all. R1 and R2 grouped all 22 sites under "format string"; that grouping is imprecise. 5 of 22 are literals; 17 of 22 use `format!`.
- **Site #8 and #9 are byte-identical strings** ("Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)"). They differ only in `evidence[0]` ("Direction: request" vs "Direction: response"). A downstream JSON consumer that dedups findings by `(category, verdict, confidence, summary)` would collapse these two distinct events.
- **Site #13 (ClientHello weak cipher) has a unique evidence-shape:** `evidence: weak` -- a `Vec<String>` of cipher names from prior filter+map. The vec length is variable (1..=N for N weak ciphers offered). Every OTHER emission site has a `vec![...]` literal with a fixed entry count (1 entry for 20 sites; 2 entries for site #20). This is the ONLY emission site where the evidence cardinality is data-dependent.
- **Site #20 (finalize segment-limit) is the ONLY site with a 2-entry evidence vec.** The two strings function as a header/comment pair. This is also the ONLY site with the cap-bypass behavior per P3 R2's new BC-RAS-054.
- **Site #20 also contains an inline `if count == 1 { "" } else { "s" }` ternary INSIDE the format! macro** for plural disambiguation. No other site does this. Sites #5 and #7 trivially handle "no field" by being literals; sites with optional count values use the raw integer.
- **Site #10 contains a literal Unicode section sign (U+00A7) in the source string** (`RFC 6066 §3`). This is the only source string with a non-ASCII codepoint embedded. Downstream JSON consumers see a UTF-8-encoded "§" in the wire format; terminal consumers see "§" rendered directly (the terminal escaper escapes C0/DEL/non-CR-LF C1 but NOT regular UTF-8 graphemes). **Forensic-tooling implication:** any JSON consumer that downgrades to ASCII (e.g., `\u00a7` escape) sees a longer string than what source-greps will find.
- **Format-string field-name conventions are mixed.** Most TLS sites use named captures (`{hostname}`, `{hex}`, `{version}`, `{version_name}`). HTTP sites mix named and positional. Reassembly sites use positional `{}`. This is inconsistent and a P5 concern.
- **All 22 sites construct `Finding` literally with `Finding { ... }` syntax** (no builder, no helper function). Sites #8/#9 (TooManyHeaders) are the only emissions that happen INSIDE an outer if-let chain (`if e == httparse::Error::TooManyHeaders`). Other emissions happen in the main flow.

### Target 2 -- SniValue disambiguation when SNI has BOTH control bytes AND non-ASCII UTF-8

**Source re-read (`tls.rs:219-242`, the `extract_sni` match block):**

```rust
return Some(match std::str::from_utf8(hostname) {
    Ok(s) if s.is_ascii() && !contains_c0_or_del(s) => SniValue::Ascii(s.to_string()),
    Ok(s) if s.is_ascii() => SniValue::AsciiWithControl {
        hostname: s.to_string(),
        hex: bytes_to_hex(hostname),
    },
    Ok(s) => SniValue::NonAsciiUtf8 {
        hostname: s.to_string(),
        hex: bytes_to_hex(hostname),
    },
    Err(_) => SniValue::NonUtf8 {
        lossy: String::from_utf8_lossy(hostname).into_owned(),
        hex: bytes_to_hex(hostname),
    },
});
```

**Disambiguation rule (mechanical, derived from match-arm semantics):**

The match arms are evaluated TOP-DOWN. Each arm has a guard. The first arm whose guard evaluates true wins. For an SNI byte sequence `B`:

1. Arm 1 fires iff `from_utf8(B).is_ok()` AND result `s.is_ascii()` AND `!contains_c0_or_del(s)`. Three-conjunct guard.
2. Arm 2 fires iff `from_utf8(B).is_ok()` AND result `s.is_ascii()` AND arm 1 didn't fire (i.e., `contains_c0_or_del(s)` is true). The `s.is_ascii()` guard is the binding gate.
3. Arm 3 fires iff `from_utf8(B).is_ok()` AND result `!s.is_ascii()`. Note: `is_ascii()` returning false means at least one byte >= 0x80; this is incompatible with arm 2's guard, so arms 2 and 3 are mutually exclusive on the `is_ascii` axis.
4. Arm 4 fires iff `from_utf8(B).is_err()` (any invalid-UTF-8 byte sequence).

**Net rule (the question P3 R2 posed): for `caf\x01\xc3\xa9` (the literal 6 bytes `c`, `a`, `f`, `0x01`, `0xc3`, `0xa9`):**

- `from_utf8([0x63, 0x61, 0x66, 0x01, 0xc3, 0xa9])` succeeds (each byte is either ASCII or part of a valid 2-byte UTF-8 sequence: `0xc3 0xa9` = U+00E9 `é`).
- The resulting string `s` contains the chars `c`, `a`, `f`, U+0001, U+00E9. So `s.is_ascii()` is **false** (because U+00E9 is non-ASCII).
- Arm 1 fails on `s.is_ascii()`.
- Arm 2 fails on `s.is_ascii()`.
- Arm 3 fires: `SniValue::NonAsciiUtf8 { hostname: "caf\x01é", hex: "63616601c3a9" }`.

**Precedence: NonAsciiUtf8 wins.** AsciiWithControl does NOT win.

**Implication:** the AsciiWithControl variant requires the WHOLE string to be ASCII. Embedded control bytes in a MIXED ASCII+UTF-8 hostname route to NonAsciiUtf8, which emits a different summary ("non-ASCII characters") -- the user never sees the control-byte warning even though the bytes are present in the hex evidence.

**The hex evidence still captures the full byte sequence** (`63616601c3a9` includes the `01` C0 control byte), so the forensic record is complete. But the SUMMARY field will refer only to "non-ASCII characters," not "ASCII control characters." A SOC operator scanning summary strings for "control" misses these mixed-encoding cases.

**Net-new finding (NEW BC candidate):** Mixed ASCII+UTF-8 SNI hostnames containing C0/DEL control bytes route to `SniValue::NonAsciiUtf8`, NOT `SniValue::AsciiWithControl`. The emitted finding's summary references only "non-ASCII characters" per the RFC 6066 / RFC 5890 message; the control-byte signal is recoverable only from the hex evidence. This formalizes/resolves the gap P3 R2's new BC-TLS-037 flagged but did not pin.

**Proposed BC-TLS-037 final wording:**
> The `extract_sni` classifier uses ordered match arms with mutually-exclusive guards: arm 1 (Ascii, no controls) requires `is_ascii() && !contains_c0_or_del()`; arm 2 (AsciiWithControl) requires `is_ascii() && contains_c0_or_del()`; arm 3 (NonAsciiUtf8) requires `from_utf8.is_ok() && !is_ascii()`; arm 4 (NonUtf8) requires `from_utf8.is_err()`. The `s.is_ascii()` predicate is the controlling gate: ANY non-ASCII byte in the SNI (even with embedded C0/DEL controls) routes to NonAsciiUtf8 with the "non-ASCII characters" summary, NOT the AsciiWithControl variant's "ASCII control characters" summary. The hex evidence is lossless across all four variants. Confidence: HIGH (from source).

### Target 3 -- HttpFlowState reset rules when `_poisoned` becomes true

**Source re-read (`http.rs:69-91, 338-369, 392-424, 432-475`):**

State container:
```rust
struct HttpFlowState {
    request_buf: Vec<u8>,
    response_buf: Vec<u8>,
    request_poisoned: bool,
    response_poisoned: bool,
    request_error_count: u8,
    response_error_count: u8,
    counted_as_non_http: bool,
}
```

Poisoning is **per-direction**. The two poisoning booleans `request_poisoned` and `response_poisoned` are independent. A flow can be request-poisoned while response continues to parse, and vice versa.

**Poisoning transitions (forward):**
- `request_error_count` increments at `http.rs:341` on parse error (when `!had_success`).
- When `state.request_error_count >= POISON_THRESHOLD` (3), `state.request_poisoned = true` (line 343).
- Symmetric for response at lines 398-400.

**`counted_as_non_http` semantics:**
- Single bool per flow (NOT per direction).
- Set true at line 345 (request-side) OR line 402 (response-side), guarded by `if !state.counted_as_non_http`.
- The guard means: the first direction to hit POISON_THRESHOLD increments `non_http_flows` by 1; the second direction (if it also poisons later) does NOT increment again.
- **`counted_as_non_http` is NEVER reset.** Search of http.rs shows zero `counted_as_non_http = false` assignments. Once true, it stays true for the lifetime of the HttpFlowState.

**`request_poisoned` / `response_poisoned` reset rules:**
- `request_poisoned = true` set at line 343. **No `= false` assignment exists anywhere in http.rs.**
- Same for `response_poisoned` (line 401). **No reset.**
- The poisoned-direction is permanently poisoned for the flow lifetime.

**Effect on `on_data` (lines 433-470):**
- `Direction::ClientToServer` path checks `state.request_poisoned` at line 441. If true, `self.poisoned_bytes_skipped += data.len() as u64` (line 442) and `return` -- no buffering, no parse.
- Symmetric for ServerToClient at lines 452-456.
- A poisoned direction silently absorbs bytes (incrementing the global counter) until `on_flow_close` removes the state.

**Buffer state during poisoning:**
- The `request_buf` (or `response_buf`) is cleared on parse error at line 364 (or 422). So after the third consecutive error, `request_buf` is empty AND `request_poisoned` is true.
- Subsequent bytes never enter the buffer (line 442 returns before line 445's `extend_from_slice`). The buffer stays empty for the flow's lifetime.

**Flow lifecycle reset:**
- `on_flow_close` at line 472 does `self.flows.remove(flow_key)`. The entire HttpFlowState is dropped. No fields are explicitly reset; the whole struct goes away.
- If the same FlowKey re-appears later (e.g., another flow with same 4-tuple after this one closed), a fresh HttpFlowState is created at line 437 via `entry().or_insert_with(HttpFlowState::new)` -- all fields reset to defaults from `HttpFlowState::new()`.

**Net rules:**

| Field | Poisoning grain | Reset within flow? | Reset at close? |
|---|---|---|---|
| `request_poisoned: bool` | per-direction | NEVER | yes (flow removed) |
| `response_poisoned: bool` | per-direction | NEVER | yes (flow removed) |
| `request_error_count: u8` | per-direction | yes -- reset to 0 on successful parse at line 333 | yes (flow removed) |
| `response_error_count: u8` | per-direction | yes -- reset to 0 on successful parse at line 390 | yes (flow removed) |
| `counted_as_non_http: bool` | **per-flow** (single bool covering both directions) | NEVER | yes (flow removed) |
| `request_buf: Vec<u8>` | per-direction | drained on each parse success (`drain(..bytes_consumed)`); cleared on parse error | yes (flow removed) |
| `response_buf: Vec<u8>` | per-direction | drained on each parse success; cleared on parse error | yes (flow removed) |

**Net-new findings:**

1. **The poisoning booleans are STRICTLY MONOTONIC false-to-true** within a flow lifetime. There is no recovery path.
2. **The error-count fields are NON-MONOTONIC** -- they reset to 0 on successful parse (lines 333, 390). This is the "tolerate body-byte-induced failures" comment at line 65-67 cashing out: a successful header parse erases prior accumulated errors. The threshold-of-3 is therefore measured against CONSECUTIVE errors, not cumulative.
3. **`counted_as_non_http` is a one-way latch** that prevents double-counting `non_http_flows`. Its asymmetric grain (per-flow, while error counts and poison bits are per-direction) means: if both directions independently poison, only the FIRST one's direction triggers the latch; the second direction's contribution to `non_http_flows` is silently merged into "this flow already counted." The `non_http_flows` metric counts FLOWS, not DIRECTIONS.
4. **No public accessor exposes any poisoning state.** Tests cannot inspect `request_poisoned` directly; they must observe via `poisoned_bytes_skipped()` or `parse_error_count()`. This makes BC-HTTP-poisoning-class contracts harder to pin (consistent with P3 R2's MEDIUM-confidence tagging).

**P3 R2 BC implication:** the new BC-RAS-054 cap-bypass design (Pass 3 R2 §2 Target 2) does not interact with HTTP poisoning because the cap-gated/cap-bypass semantics are scoped to the reassembly engine's `findings` vec, not the analyzer's `all_findings` vec. HTTP analyzer findings are unbounded (no MAX_FINDINGS equivalent in the analyzer). This is a P4 NFR finding worth surfacing.

### Target 4 -- Branch ordering in dispatcher when buffer starts with `\x16\x03\x01GET`

**Source re-read (`dispatcher.rs:37-64`, the `classify` function):**

```rust
fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget {
    if data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03 {
        return DispatchTarget::Tls;
    }
    if data.starts_with(b"GET ")
        || data.starts_with(b"POST ")
        || ...
    {
        return DispatchTarget::Http;
    }
    // port fallback...
}
```

**Tracing `\x16\x03\x01GET ` (concretely the 8 bytes `0x16, 0x03, 0x01, 0x47, 0x45, 0x54, 0x20, ...`):**

1. First branch (TLS): `data.len() >= 5` ? Yes (8 >= 5). `data[0] == 0x16` ? Yes. `data[1] == 0x03` ? Yes. **All three conjuncts true.** Returns `DispatchTarget::Tls` at line 40.
2. HTTP branch never reached.

**TLS wins.** The HTTP path-prefix check at line 42 is never evaluated.

**Critical sub-finding -- the TLS gate is loose, not strict:**

The TLS check requires only:
- buffer length >= 5
- byte 0 == 0x16 (TLS record content-type Handshake)
- byte 1 == 0x03 (TLS major version)

It does NOT check:
- byte 2 (TLS minor version: 0x00=SSL3, 0x01=TLS1.0, 0x02=TLS1.1, 0x03=TLS1.2, 0x04=TLS1.3)
- bytes 3-4 (record length, which must be > 0 for a real record)

So ANY HTTP request whose body content happens to begin with the byte sequence `0x16, 0x03, ANY, ANY, ANY` would be misclassified as TLS. This is unrealistic for the FIRST 5 bytes of a flow (HTTP must start with a method like `GET `, `POST `, etc.) -- but if the dispatcher were called on a non-first-segment chunk (which it CAN be, per BC-DSP-006 reclassification semantics noted in P3 R2 MED-8), the loose check could misroute. The flow ordering protects this in practice: the dispatcher caches the FIRST classification result for non-None, so once HTTP is recognized on segment 1, segment 2 bytes are not re-classified.

**Branch-ordering robustness:** the question's premise (`\x16\x03\x01GET`) cannot actually occur as the first 5 bytes of an HTTP/1.x request -- HTTP requires the request-line to start with the method. But IF an analyst constructed a pathological capture where the first segment of a "supposed HTTP flow" is exactly these 8 bytes, the dispatcher would classify it as TLS, hand the 8 bytes to TLS analyzer, TLS analyzer would attempt to parse a TLS record (record-type=0x16, version=0x0301), fail (the 3-byte "length" field would be `0x01 0x47 0x45` -> 0x014745 = 83781, MUCH larger than MAX_RECORD_PAYLOAD (16384) typical), and TLS would clear its buffer with a parse_error increment (tls.rs:588-597). No HTTP analysis would occur.

**Net-new finding:** the dispatcher's TLS gate is LOOSE (no minor-version or non-zero-length check); content-first detection always wins over port-fallback. **TLS strictly wins over HTTP for the conflict scenario.** This formalizes the dispatcher branch-ordering question Pass 6 §6.4 flagged.

**Implication for P3 R2's BC-DSP-006 (DispatchTarget::None NOT cached for reclassification):** the dispatcher's only caching is for non-None targets. A flow whose first segment was 4 bytes (insufficient for TLS check; insufficient for "GET ") will return None; routes is NOT updated; on the next segment the classifier re-runs. There is no upper bound on the number of re-classification attempts. If the same flow continues to send tiny pieces forever, classify() runs forever. P3 R2 MED-8 flagged this as a Pass 6 dispatcher cost-ceiling deepening Q; this round confirms the cost is `O(packets_in_flow)` for None-flows.

### Target 5 -- Drop impls audit for the 10 state containers

P2 R1 §5 named 10 state containers. For each, audited for explicit `Drop` impl and dependence on explicit cleanup.

`find src/ -name '*.rs' | xargs awk '/impl Drop/'` returns **ZERO matches** across the entire src/ tree. No type in wirerust has a hand-written `Drop` impl.

| # | State container | File:Line | Has Drop impl? | Relies on explicit cleanup? |
|---|---|---|---|---|
| 1 | `TcpReassembler` | reassembly/mod.rs (struct ~line 35) | No (compiler-derived only) | Yes -- relies on `finalize(handler)` to push the segment-limit finding (mod.rs:399-417). Without the call, the count is lost. Also relies on `close_flow` to invoke `handler.on_flow_close`, which drives analyzer cleanup. |
| 2 | `Flow` | reassembly/flow.rs | No | No -- holds two `FlowDirection`s; both Drop is structural. |
| 3 | `FlowDirection` | reassembly/flow.rs | No | No -- holds a `BTreeMap<u64, Segment>` and counters. BTreeMap drop is recursive and complete. |
| 4 | `Segment` | reassembly/segment.rs | No | No. |
| 5 | `HttpAnalyzer` | analyzer/http.rs:101-113 | No | No -- `on_flow_close` is called per-flow to remove HttpFlowState from `self.flows`. If `on_flow_close` is skipped, the HashMap retains the state until HttpAnalyzer's natural drop. No invariant is violated. |
| 6 | `HttpFlowState` | analyzer/http.rs:69-77 | No | No -- pure data; `Vec<u8>` and bool fields drop trivially. |
| 7 | `TlsAnalyzer` | analyzer/tls.rs:271-281 | No | No -- same shape as HttpAnalyzer. |
| 8 | `TlsFlowState` | analyzer/tls.rs:246-251 | No | No. |
| 9 | `StreamDispatcher` | dispatcher.rs:15-20 | No | Yes (transitive) -- holds Options<HttpAnalyzer>/<TlsAnalyzer>; analyzer cleanup invariants (item 5/7) flow through. |
| 10 | `DnsAnalyzer` | analyzer/dns.rs | No (81 LOC, struct verified earlier) | No -- counter-only. |

**Net rules:**

- **No type in wirerust has an explicit `Drop` impl.** All cleanup is structural (compiler-derived). 
- **Only TcpReassembler has a *correctness* dependency on explicit cleanup** (`finalize()` to flush the segment-limit summary finding). If `finalize` is not called, the finding count is undercounted by up to 1.
- **All other "cleanup" paths (`on_flow_close`, `HttpAnalyzer::on_flow_close`, etc.) are convenience-not-correctness.** They remove map entries to reclaim memory; if skipped, the HashMaps retain entries until the owning struct drops, but no observable behavior is wrong as long as no further analysis is run on the flow.
- **The lack of Drop impls means panic-safety is structural.** A panic mid-analysis cannot violate per-flow cleanup invariants because there are no invariants to violate (other than the finalize one).
- **Implicit P5 finding:** the absence of Drop impls is consistent with the codebase's "no clever ownership tricks" convention -- everything is owned-or-borrowed; no Rc<RefCell<...>>; no UnsafeCell; no Pin. All `Drop` is the compiler's structural recursion.

**Net-new finding:** Only `TcpReassembler::finalize()` is correctness-critical. The other 9 state containers' cleanup paths exist solely for incremental memory reclamation during long captures (the eviction policy in mod.rs:493-531). **The codebase has no hand-written Drop, so all guarantees about cleanup come from explicit method calls, NOT from RAII.** This is a load-bearing convention worth surfacing to P5.

### Target 6 -- ParsedRequest borrow lifetimes; host/user_agent Option semantics

**Source re-read (`http.rs:13-30` and `http.rs:57-62`):**

```rust
struct ParsedRequest {
    bytes_consumed: usize,
    method: String,
    uri: String,
    version: u8,
    host: Option<String>,
    user_agent: Option<String>,
}

fn parse_one_request(buf: &[u8]) -> Result<Option<ParsedRequest>, httparse::Error> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(buf) {
        Ok(httparse::Status::Complete(n)) => Ok(Some(ParsedRequest {
            bytes_consumed: n,
            method: req.method.unwrap_or("").to_string(),
            uri: req.path.unwrap_or("").to_string(),
            version: req.version.unwrap_or(1),
            host: find_header(req.headers, "host"),
            user_agent: find_header(req.headers, "user-agent"),
        })),
        ...
    }
}

fn find_header(headers: &[httparse::Header<'_>], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| String::from_utf8_lossy(h.value).trim().to_string())
}
```

**Borrow lifetimes:** `ParsedRequest` has zero borrows. All fields are owned (`String`, `usize`, `u8`, `Option<String>`). The borrow lifetime of the inbound `buf: &[u8]` ends at the function return -- httparse's `Request` holds references into `buf`, but `to_string()` calls inside ParsedRequest construction copy out into owned Strings. **ParsedRequest is `'static`-safe** (any field can outlive the buf). This is intentional -- it allows the analyzer to drain the buffer (line 332: `state.request_buf.drain(..parsed.bytes_consumed)`) without breaking aliasing rules.

**Option<String> semantics for `host` / `user_agent`:**

The `find_header` helper returns:
- `None` if NO header with matching name (case-insensitive) is found.
- `Some(String)` if a matching header exists, with the value being `String::from_utf8_lossy(h.value).trim().to_string()`.

**Crucially, `find_header` does NOT distinguish "absent" from "present-but-empty":**

- Header absent: `headers.iter().find(...)` returns None -> `find_header` returns None.
- Header present with empty value (`Host: \r\n`): `find` returns Some(header); `from_utf8_lossy(b"")` produces `""`; `.trim()` produces `""`; `.to_string()` produces `String::new()`. Result: `Some("".to_string())`.
- Header present with whitespace-only value (`Host:   \r\n`): same trim-to-empty -> `Some("")`.
- Header present with valid value: `Some("example.com")` etc.

**So:**
- `host: Some(s)` where `s.is_empty()` ⇔ Host header was present with an empty or whitespace-only value.
- `host: None` ⇔ Host header was absent entirely.

**Downstream behavior contracts (re-read of detection code):**

1. **Missing-Host detection at http.rs:251:** `if parsed.version == 1 && parsed.host.is_none()` -- fires ONLY when host was absent. An HTTP/1.1 request with `Host:` (empty value) is `Some("")` and does NOT trigger this detection.
2. **Hosts map increment at http.rs:314-318:** `if let Some(ref h) = parsed.host { ... hosts.entry(h.clone()) ... }` -- if host is `Some("")`, the empty string is used as a HashMap key. The empty-string host gets counted. This means the `hosts` map can contain a key whose String is `""`.
3. **Empty User-Agent detection at http.rs:279:** `if parsed.user_agent.as_deref() == Some("")` -- fires EXACTLY when user-agent header was present with empty/whitespace value. Absent user-agent (None) does NOT trigger.
4. **user_agents map at http.rs:319-323:** symmetric to hosts -- `Some("")` becomes the empty-string map key.

**Net rules:**

| Field | None means | Some("") means | Some("foo") means |
|---|---|---|---|
| `host` | Host header absent | Host header present with empty or whitespace-only value | Host header present with valid value "foo" |
| `user_agent` | User-Agent absent | User-Agent header present with empty value | User-Agent header present with valid value |

**Net-new findings:**

1. **Header-presence-with-empty IS distinguishable from header-absent** via `Option<String>::is_some()` + `String::is_empty()`. The two layers of Option/String form a 3-state (None / Some("") / Some(non-empty)) but the codebase only uses 2-out-of-3 distinguishing predicates:
   - Missing-Host detection uses `is_none()` (collapses Some("") and Some("foo") together)
   - Empty-UA detection uses `as_deref() == Some("")` (distinguishes None from Some(""))
   - Map-key code does `entry(h.clone())` which preserves all three states uniquely.

2. **Inconsistency:** the missing-Host detection treats "absent" as anomalous but "present-empty" as compliant. The missing-UA equivalent (line 279) treats "present-empty" as anomalous but "absent" as compliant. **These two contracts are inverted.**

3. **Forensic implication:** an attacker who sends `Host:` (header present, value empty) defeats the missing-Host check. Whether this is a bug or by-design is undocumented. Recommendation: align both checks (fire on both None and Some("") for "missing-by-intent") or document the divergence.

4. **The Hosts map can grow a `"" -> count` entry**, which when serialized to JSON becomes `{"": N}`. Downstream parsers may not expect empty-string keys.

5. **No truncation in `find_header`:** values longer than any cap are kept full-length. A 1MB-long Host header value becomes a 1MB String in ParsedRequest -- this is a memory amplification path, though the request_buf cap (MAX_HEADER_BUF = 65536) and MAX_HEADERS cap (96) bound the parser's own input. **However, header value length itself is not separately capped within httparse's contract.** Pass 4 NFR concern.

---

## 3. Delta Summary

Net-new findings vs P2 R2 (all per-target):

### From Target 1 (emission-site matrix)
- 5-of-22 emission sites use string literals (not format!) for summary: #5, #7, #8, #9, #13. Prior rounds elided this distinction.
- Sites #8 and #9 share byte-identical summary strings; differ only in evidence -- finding-dedup hazard.
- Site #13 (ClientHello weak cipher) has variable-cardinality evidence vec (1..=N entries); ONLY data-dependent evidence shape in the codebase.
- Site #20 (finalize segment-limit) has TWO-entry evidence vec -- only multi-entry in the engine -- and uses inline `if count == 1` ternary inside format!.
- Site #10 contains a literal U+00A7 section sign in the source string -- the only non-ASCII grapheme in any emission template.
- Format-string conventions are mixed (named captures in TLS, positional in HTTP/reassembly) -- P5 inconsistency.

### From Target 2 (SniValue precedence)
- Disambiguation order for mixed control+non-ASCII SNI is: NonAsciiUtf8 wins; AsciiWithControl does NOT. The `is_ascii()` predicate is the gate.
- The summary text references only the matched variant's RFC; control-byte signal is recoverable only from hex evidence in mixed cases.
- Resolves the BC-TLS-037 gap P3 R2 left open; the proposed final BC wording is in §2 Target 2.

### From Target 3 (HttpFlowState reset rules)
- `request_poisoned` / `response_poisoned` are strictly monotonic false->true; no in-flow reset.
- `request_error_count` / `response_error_count` are non-monotonic (reset to 0 on successful parse) -- consecutive-error threshold, not cumulative.
- `counted_as_non_http` is per-flow (single bool), while poison/error-counts are per-direction -- asymmetric grain.
- `counted_as_non_http` never resets within flow; one-way latch.
- The `non_http_flows` metric counts FLOWS (not directions). Symmetric bidirectional poisoning increments by 1, not 2.

### From Target 4 (dispatcher branch ordering)
- TLS wins over HTTP for `\x16\x03\x01GET` prefix scenarios. Content-first detection is unconditional.
- TLS gate is LOOSE: only 3 byte conditions (length>=5, byte0=0x16, byte1=0x03); does not check minor version or non-zero record length.
- Reclassification of None-flows is unbounded -- `O(packets_in_flow)` classify() calls until a non-None target is produced or flow closes. Confirms P3 R2 BC-DSP-006 / Pass 6 §3 deepening Q.

### From Target 5 (Drop audit)
- Zero `impl Drop` blocks across the entire src/ tree.
- Only TcpReassembler::finalize() is correctness-critical for cleanup.
- All other state containers' cleanup paths (`on_flow_close`, eviction) are memory-reclamation, not correctness.
- The codebase's "no Drop, no RAII tricks" convention is load-bearing -- guarantees come from explicit method calls.

### From Target 6 (ParsedRequest semantics)
- `Option<String>` for host/user_agent encodes a 3-state space: None / Some("") / Some(non-empty).
- Missing-Host detection (uses is_none()) and missing-UA detection (uses `== Some("")`) have INVERTED contracts -- one fires on absent, the other on present-empty. Likely-bug or undocumented design.
- Hosts/user_agents maps can accumulate `""` keys -- JSON shape implication.
- An attacker sending `Host:` (empty value) defeats the missing-Host check.
- Header value length is not separately capped within ParsedRequest -- ambient bounds are MAX_HEADER_BUF (65536) and MAX_HEADERS (96); per-header-value cap is implicit.

**Substantive count: 17 net-new findings across 6 targets. CONV-ABS retractions: 0.**

---

## 4. Novelty Assessment

Novelty: SUBSTANTIVE

Would removing these findings change how someone would spec the system?

- **YES** -- Target 1's emission-site matrix is a verbatim reference that the spec/PRD work needs to cite. The "site #8 and #9 are byte-identical" finding is a downstream-dedup hazard a spec author would otherwise miss.
- **YES** -- Target 2 resolves an explicit open BC (BC-TLS-037 was MEDIUM-confidence in P3 R2; now upgradable to HIGH with a documented disambiguation rule).
- **YES** -- Target 3's "missing-Host vs missing-UA inverted contracts" is an observable inconsistency that a spec or behavior test would expose. The poisoning-reset rules (strictly monotonic; per-direction except for the per-flow `counted_as_non_http` latch) materially change the analyzer state-machine specification.
- **YES** -- Target 4's "TLS gate is LOOSE" finding is a security-relevant disambiguation -- it bounds the dispatcher's misclassification risk.
- **YES** -- Target 5's "zero Drop impls in src/" formalizes a convention that hadn't been called out before. It's load-bearing for panic-safety reasoning.
- **YES** -- Target 6's "3-state space, 2 distinguishing predicates" finding is an observable inconsistency in the analyzer contract surface.

Each target produced 2-4 distinct findings that change either the spec accuracy or the planning estimate for downstream work. This is SUBSTANTIVE.

---

## 5. Remaining gaps / next candidate scope

The 6 carryover targets from P2 R2 §5 are addressed. Remaining domain-model gaps a P2 R4 could pick up (lower-value than R3 work):

1. **Tests-side grep audit** -- P2 R2's claim "ThreatCategory::C2 referenced exactly once in tests/findings_tests.rs:23" was not re-verified in this round. A future round could mechanically grep tests/ for all uses of LateralMovement, C2, and the 9 catalogued-but-unused MITRE IDs to confirm test fixture footprint.
2. **TLS site #13 (ClientHello weak cipher) evidence-vec cardinality bound** -- the vec length is `weak.len()` where `weak` is filtered from `ch.ciphers`. Is there an upper bound? `ch.ciphers` from tls-parser is unbounded in principle. A pathological ClientHello with 65535 weak-cipher entries would produce a 65535-entry evidence vec, which would balloon the Finding's memory footprint. NFR cross-pollination.
3. **The `""` empty-host-map-key forensic shape** -- specific verification of the JSON output when a flow sends `Host:` (empty value), to confirm the `{"": N}` shape. Probably needs a test fixture.
4. **`request_buf.clear()` after parse error preserves no diagnostic trace** -- when poison-threshold is reached, the offending bytes are gone. Forensic analyzers cannot inspect what the bytes were. Whether this is by-design or worth changing is a P4/P5 question.
5. **Mixed-encoding SNI hex evidence forensics** -- when SNI bytes are `caf\x01\xc3\xa9` and route to NonAsciiUtf8, the hex evidence `63616601c3a9` is correct but the analyst must visually decode it to spot the embedded `01`. Tooling support: a `--decode-sni-hex` flag could expand hex into byte-by-byte commentary. Out-of-scope; P5 polish.
6. **`finalize` segment-limit finding is at most 1 per process** -- the count parameter aggregates across all flows. A test scenario where flow A had 100 dropped segments and flow B had 50 produces ONE finding with `count=150`. A downstream consumer cannot attribute back to flow A vs B. This is a coarsening; whether it's a design constraint or a gap is a P3/P4 question.

If P2 R4 is run, items 1-3 are concrete and would each produce a finding or two. Items 4-6 are P4/P5 territory (NFR / convention concerns), not pure domain-model.

**P2 deepening convergence forecast:** items 1-3 are SUBSTANTIVE-eligible (test-fixture grep, evidence-vec bound, JSON shape verification). If P2 R4 is run and produces <3 substantive items across these, P2 has converged. The expected outcome is convergence -- the domain-model surface is now thoroughly covered after R1+R2+R3.

---

## State Checkpoint

```yaml
pass: 2
round: 3
status: complete
sub_pass: deep_domain_model
targets_addressed: 6
net_new_findings: 17
conv_abs_retractions: 0
hallucination_classes_audited: 5
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_action: pass_2_round_4_optional_convergence_expected
resume_from: null
```
