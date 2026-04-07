# TLS ClientHello + ServerHello Analyzer Design

**Issue:** #2 — Add TLS ClientHello analyzer  
**Scope:** `src/analyzer/tls.rs`, `src/dispatcher.rs`, changes to `src/main.rs`, new tests in `tests/tls_analyzer_tests.rs`.  
**Dependencies:** `tls-parser` 0.12, `md-5` 0.11  
**Related:** ADR 0001 (content-first stream dispatch)

## Problem

wirerust has no TLS analysis capability. The `--tls` CLI flag exists but is unused. TLS handshake metadata (SNI, cipher suites, JA3 fingerprints) is critical for network forensics and threat detection, yet the current architecture only supports a single `StreamHandler` (HTTP), making it impossible to add stream-level analyzers without a routing mechanism.

## Approach

### 1. StreamDispatcher (ADR 0001)

A content-first dispatcher that routes reassembled TCP stream data to the correct protocol analyzer. Implements `StreamHandler` and wraps `Option<HttpAnalyzer>` + `Option<TlsAnalyzer>`.

**Classification on first `on_data` per flow:**

1. If `data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03` → TLS (record header: Handshake + SSL/TLS 3.x)
2. If first bytes match HTTP method (`GET `, `POST `, `PUT `, `DELETE `, `HEAD `, `OPTIONS `, `PATCH `, `CONNECT `, `TRACE `) or `HTTP/` → HTTP
3. If data too short (< 5 bytes) → fall back to port hints: 443/8443 → TLS, 80/8080 → HTTP
4. No match → `None` (data dropped)

Decision cached per flow in `HashMap<FlowKey, DispatchTarget>`. See ADR 0001 for full rationale.

### 2. TlsAnalyzer

Implements `StreamAnalyzer` (same trait as `HttpAnalyzer`). Parses ClientHello and ServerHello from reassembled TCP streams.

**Per-flow state:**

```rust
struct TlsFlowState {
    buf: Vec<u8>,           // Accumulates stream bytes for TLS record extraction
    client_hello_seen: bool,
    server_hello_seen: bool,
}
```

**TLS record extraction loop:**

```
on_data(flow_key, direction, data, offset):
  1. If both client_hello_seen and server_hello_seen for this flow → return early
  2. Append data to flow's buf (capped at 64KB)
  3. Loop:
     a. buf.len() < 5 → break (need TLS record header)
     b. content_type = buf[0]
     c. record_len = u16::from_be_bytes(buf[3..5]) as usize
     d. buf.len() < 5 + record_len → break (incomplete record)
     e. If content_type != 0x16 (not handshake) → drain 5+record_len, continue
     f. parse_tls_plaintext(&buf[..5+record_len])
        → Ok: iterate TlsMessage list:
            - Handshake(ClientHello(ch)) → process_client_hello(ch, flow_key)
            - Handshake(ServerHello(sh)) → process_server_hello(sh, flow_key)
            - Other → ignore
        → Err(Incomplete) → increment parse_errors, drain record, break
        → Err(other) → increment parse_errors, clear buf, break
     g. Drain 5+record_len from buf
  4. Stop buffering after both CH + SH seen for this flow
```

### 3. ClientHello Processing

Extract and aggregate:
- **SNI**: From `TlsExtension::SNI` — hostname string. Counted in `HashMap<String, u64>`.
- **Cipher suites**: `ch.ciphers` as `Vec<TlsCipherSuiteID>`. Counted by selected cipher (from ServerHello).
- **TLS version**: `ch.version.0` as u16. Counted in `HashMap<u16, u64>`.
- **Extensions**: Parsed via `parse_tls_extensions(ch.ext)`.

**JA3 computation:**

```
JA3_string = "{version},{ciphers},{extensions},{elliptic_curves},{ec_point_formats}"

Where:
  version          = ch.version.0 as decimal string (e.g., "771")
  ciphers          = ch.ciphers, filtered GREASE, dash-separated decimals
  extensions       = extension type IDs in wire order, filtered GREASE, dash-separated
  elliptic_curves  = from TlsExtension::EllipticCurves, filtered GREASE, dash-separated
  ec_point_formats = from TlsExtension::EcPointFormats, dash-separated

GREASE filter: (id & 0x0F0F) == 0x0A0A (for u16 values)
              TlsExtension::Grease(..) variant (for extensions)

JA3 = hex(MD5(JA3_string))
```

**Empty fields:** When EllipticCurves or EcPointFormats extensions are absent, those fields are empty strings (not omitted). This produces trailing commas, e.g., `771,ciphers,extensions,,`. The trailing commas are part of the string and contribute to the MD5 hash. This is the official JA3 specification.

**TLS 1.3 version:** Use `ch.version.0` which gives the `legacy_version` field value (0x0303 = 771 for TLS 1.3). Do NOT use the `supported_versions` extension value (0x0304 = 772). This matches Wireshark's implementation and the principle of capturing literal packet contents. The `tls-parser` crate's `TlsClientHelloContents.version` already provides the legacy_version value.

JA3 hashes counted in `HashMap<String, u64>`.

**Weak cipher finding:**

Scan `ch.ciphers` for NULL, anonymous, or export cipher suites. If found:

- **Category:** `ThreatCategory::Anomaly`
- **Verdict:** `Verdict::Likely`
- **Confidence:** `Confidence::High`
- **MITRE:** `None` (no clean mapping for weak cipher negotiation)
- **Summary:** `"ClientHello offers weak cipher suites (NULL/anonymous/export)"`
- **Evidence:** List of weak cipher IDs found

**Weak cipher identification:**

A cipher suite is weak if any of:
- Name contains `NULL` (no encryption)
- Name contains `anon` (anonymous key exchange, no authentication)
- Name contains `EXPORT` (deliberately weakened)
- `TlsCipherSuiteID` matches known weak IDs: any suite with `WITH_NULL`, `_anon_`, or `EXPORT` in the IANA registry name

The `tls-parser` crate provides cipher suite lookup via `TlsCipherSuite::from_id(id)` which returns an `Option<&TlsCipherSuite>` with a `name` field. For unknown cipher IDs (returns `None`), skip the weak check — only flag ciphers we can positively identify as weak.

### 4. ServerHello Processing

Extract:
- **Selected cipher**: `sh.cipher` (single `TlsCipherSuiteID`)
- **TLS version**: `sh.version.0`
- **Extensions**: Parsed via `parse_tls_extensions(sh.ext)`

**JA3S computation:**

```
JA3S_string = "{version},{cipher},{extensions}"

Where:
  version    = sh.version.0 as decimal string
  cipher     = sh.cipher.0 as decimal string (single value)
  extensions = extension type IDs, dash-separated decimals

JA3S = hex(MD5(JA3S_string))
```

**TLS 1.3 version:** Same as JA3 — use `sh.version.0` (legacy_version = 771), not supported_versions extension.

JA3S hashes counted in `HashMap<String, u64>`.

**Weak cipher selection finding:**

If `sh.cipher` is RC4, NULL, anonymous, or export:

- **Category:** `ThreatCategory::Anomaly`
- **Verdict:** `Verdict::Likely`
- **Confidence:** `Confidence::Medium`
- **MITRE:** `None`
- **Summary:** `"ServerHello selected weak cipher suite ({name})"`
- **Evidence:** `"Selected cipher: {name} (0x{id:04x})"`

Medium confidence (not High) because the server may be misconfigured rather than under attack.

### 5. Aggregate State

```rust
pub struct TlsAnalyzer {
    flows: HashMap<FlowKey, TlsFlowState>,
    sni_counts: HashMap<String, u64>,
    ja3_counts: HashMap<String, u64>,
    ja3s_counts: HashMap<String, u64>,
    version_counts: HashMap<u16, u64>,
    cipher_counts: HashMap<String, u64>,  // Keyed by IANA name from ServerHello
    handshakes_seen: u64,
    parse_errors: u64,
    all_findings: Vec<Finding>,
}
```

### 6. `summarize()` Output

```json
{
  "analyzer_name": "TLS",
  "packets_analyzed": "<handshakes_seen>",
  "detail": {
    "top_snis": ["example.com", "api.github.com", ...],
    "ja3_hashes": {"e7d705a3...": 5, "abc123...": 2},
    "ja3s_hashes": {"def456...": 3},
    "tls_versions": {"771": 10, "772": 5},
    "cipher_suites": {"TLS_AES_128_GCM_SHA256": 8, ...},
    "parse_errors": 0
  }
}
```

`top_snis` is the top 20 SNIs by count (same pattern as HTTP's `top_hosts`).

### 7. Public Accessors

- `pub fn sni_counts(&self) -> &HashMap<String, u64>`
- `pub fn ja3_counts(&self) -> &HashMap<String, u64>`
- `pub fn ja3s_counts(&self) -> &HashMap<String, u64>`
- `pub fn version_counts(&self) -> &HashMap<u16, u64>`
- `pub fn parse_error_count(&self) -> u64`
- `pub fn handshake_count(&self) -> u64`

### 8. CLI Integration

`src/main.rs` changes:
- Replace `HttpAnalyzer` + `NullHandler` with `StreamDispatcher`
- `--tls` or `--all` enables TLS in dispatcher
- `--tls` auto-enables reassembly (same pattern as `--http`)
- Collect `tls.findings()` and `tls.summarize()` in report output

`src/cli.rs`: No changes (the `--tls` flag already exists at line 79).

## Changes

### New Files

| File | Purpose |
|------|---------|
| `src/analyzer/tls.rs` | `TlsAnalyzer` implementing `StreamAnalyzer` |
| `src/dispatcher.rs` | `StreamDispatcher` implementing `StreamHandler` |
| `tests/tls_analyzer_tests.rs` | Unit tests with crafted TLS bytes |

### Modified Files

| File | Change |
|------|--------|
| `src/main.rs` | Use `StreamDispatcher` instead of direct analyzer, wire up `--tls` |
| `src/analyzer/mod.rs` | Add `pub mod tls` |
| `src/lib.rs` | Add `pub mod dispatcher` |
| `Cargo.toml` | Add `tls-parser = "0.12"`, `md-5 = "0.11"` |

### No Changes To

- Reassembly engine (`src/reassembly/`)
- HTTP analyzer (`src/analyzer/http.rs`)
- Finding struct (`src/findings.rs`)
- Reporter (`src/reporter/`)

## Tests

| Test | Description |
|------|-------------|
| `test_parse_client_hello` | Craft a minimal ClientHello, assert SNI extracted, JA3 computed |
| `test_parse_server_hello` | Craft a ServerHello, assert cipher and JA3S computed |
| `test_ja3_grease_filtering` | ClientHello with GREASE cipher suites, assert they're excluded from JA3 |
| `test_ja3_known_fingerprint` | Use a known ClientHello → verify JA3 hash matches published value |
| `test_weak_cipher_finding_client` | ClientHello with NULL cipher, assert finding generated |
| `test_weak_cipher_finding_server` | ServerHello selects RC4, assert finding generated |
| `test_normal_handshake_no_findings` | Valid modern handshake, assert no findings |
| `test_parse_error_counter` | Malformed TLS record, assert `parse_error_count() == 1` |
| `test_summarize_output` | Full handshake, assert `summarize()` contains SNI, JA3, version |
| `test_dispatcher_routes_tls` | Send TLS bytes through dispatcher, assert TLS analyzer receives them |
| `test_dispatcher_routes_http` | Send HTTP bytes through dispatcher, assert HTTP analyzer receives them |
| `test_dispatcher_content_detection` | TLS on port 80, assert dispatcher routes to TLS (not HTTP) |
| `test_stop_after_handshake` | Send handshake + application data, assert no parse errors from encrypted bytes |

## Known Limitations (v1)

- **No handshake message fragmentation across TLS records.** If a ClientHello/ServerHello spans multiple TLS records, it's counted as a parse error. This is vanishingly rare for these messages (both are well under 16KB record max). File follow-up issue.
- **No Certificate message parsing.** Self-signed detection requires Certificate message parsing (TLS 1.2 only). Out of scope per design decision — file as separate issue.
- **No JA4.** JA3 is the current standard. JA4 (sorts extensions deterministically) can be added as enhancement.
- **Port fallback is limited.** Only 443/8443 and 80/8080 in fallback list. Content detection handles all other cases.

## Not In Scope

- Changes to reassembly engine
- Changes to Finding struct
- Certificate parsing / self-signed detection
- JA4 fingerprinting
- Known malicious JA3 hash matching (requires threat intel feed)
- TLS decryption
