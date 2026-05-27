//! TLS handshake analyzer.
//!
//! Reads reassembled TCP-stream data (the dispatcher routes to here on the
//! `0x16 0x03` content fingerprint), parses ClientHello / ServerHello records
//! with `tls_parser`, and computes JA3 / JA3S fingerprints (MD5-hashed
//! field concatenations per the Salesforce JA3 spec).
//!
//! Counters surfaced via [`StreamAnalyzer::summarize`] cover SNI / JA3 / JA3S
//! / version / cipher distributions plus `parse_errors` (malformed records)
//! and `truncated_records` (DoS-protection drops, see LESSON-P1.05).
//!
//! Findings cover: SNI hostnames containing C0/DEL control bytes,
//! non-ASCII SNI (RFC 6066 violation), weak / deprecated cipher suites,
//! and SSL/3.0 handshakes (POODLE-class).

use std::collections::HashMap;

use md5::{Digest, Md5};
use tls_parser::{
    Err as NomErr, TlsCipherSuite, TlsCipherSuiteID, TlsExtension, TlsExtensionType, TlsMessage,
    TlsMessageHandshake, parse_tls_extensions, parse_tls_plaintext,
};

use crate::analyzer::AnalysisSummary;
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

const MAX_BUF: usize = 65_536;
const MAX_MAP_ENTRIES: usize = 50_000;
/// Max valid TLS record payload: 18,432 bytes (TLS 1.2 ciphertext max per RFC 5246).
/// TLS 1.3 max is 16,640 but we use the larger value as a safe upper bound.
const MAX_RECORD_PAYLOAD: usize = 18_432;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Returns true if `val` is a TLS GREASE value (RFC 8701).
///
/// This uses the well-known bitmask test `(val & 0x0F0F) == 0x0A0A`,
/// which is **deliberately broader than RFC 8701's strict definition**.
/// RFC 8701 reserves exactly 16 GREASE values — `0x0A0A`, `0x1A1A`, …,
/// `0xFAFA` (both bytes equal, of the form `0xNANA`). The bitmask also
/// matches the 240 non-canonical `0x_A_A` values (e.g. `0x0A1A`,
/// `0xCABA`). This is intentional and harmless: IANA has assigned no
/// real cipher-suite or extension ID of `0x_A_A` form outside the 16
/// GREASE values, so the wider mask never filters a value that carries
/// JA3-relevant signal. The cheaper single-mask test is preferred over
/// an explicit 16-value membership check. The JA3 property tests in
/// this module's `ja3_property_tests` submodule pin this contract.
fn is_grease_u16(val: u16) -> bool {
    (val & 0x0F0F) == 0x0A0A
}

/// Returns true if the cipher is considered weak for client-advertised suites
/// (NULL / ANON / EXPORT ciphers).
fn is_weak_cipher(id: TlsCipherSuiteID) -> bool {
    match TlsCipherSuite::from_id(id.0) {
        Some(cs) => {
            let n = cs.name.to_uppercase();
            n.contains("NULL") || n.contains("ANON") || n.contains("EXPORT")
        }
        None => false,
    }
}

/// Returns true if a cipher is weak when selected by the server (adds RC4).
fn is_weak_server_cipher(id: TlsCipherSuiteID) -> bool {
    if is_weak_cipher(id) {
        return true;
    }
    match TlsCipherSuite::from_id(id.0) {
        Some(cs) => cs.name.to_uppercase().contains("RC4"),
        None => false,
    }
}

/// Human-readable cipher name, falling back to hex for unknown IDs.
fn cipher_name(id: TlsCipherSuiteID) -> String {
    match TlsCipherSuite::from_id(id.0) {
        Some(cs) => cs.name.to_string(),
        None => format!("0x{:04x}", id.0),
    }
}

/// Convert a byte slice to a lowercase hex string.
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ── JA3 / JA3S ───────────────────────────────────────────────────────────────

/// Compute JA3 fingerprint from ClientHello fields.
///
/// Returns `(md5_hex, ja3_string)`.
fn compute_ja3(
    version: u16,
    ciphers: &[TlsCipherSuiteID],
    extensions: &[TlsExtension<'_>],
) -> (String, String) {
    // Ciphers — filter GREASE
    let cipher_str: String = ciphers
        .iter()
        .filter(|c| !is_grease_u16(c.0))
        .map(|c| c.0.to_string())
        .collect::<Vec<_>>()
        .join("-");

    // Extension type IDs — filter GREASE
    let ext_ids: String = extensions
        .iter()
        .filter_map(|e| {
            let t: TlsExtensionType = e.into();
            let v: u16 = t.into();
            if is_grease_u16(v) {
                None
            } else {
                Some(v.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("-");

    // Elliptic curves (named groups) — filter GREASE
    let mut curves: Vec<String> = Vec::new();
    let mut point_formats: Vec<String> = Vec::new();

    for ext in extensions {
        match ext {
            TlsExtension::EllipticCurves(groups) => {
                for g in groups {
                    if !is_grease_u16(g.0) {
                        curves.push(g.0.to_string());
                    }
                }
            }
            TlsExtension::EcPointFormats(fmts) => {
                for &b in *fmts {
                    point_formats.push(b.to_string());
                }
            }
            _ => {}
        }
    }

    let curves_str = curves.join("-");
    let pf_str = point_formats.join("-");

    let ja3_str = format!("{version},{cipher_str},{ext_ids},{curves_str},{pf_str}");
    let hash = bytes_to_hex(Md5::digest(ja3_str.as_bytes()).as_slice());
    (hash, ja3_str)
}

/// Compute JA3S fingerprint from ServerHello fields.
///
/// Returns the MD5 hex string.
fn compute_ja3s(version: u16, cipher: TlsCipherSuiteID, extensions: &[TlsExtension<'_>]) -> String {
    let ext_ids: String = extensions
        .iter()
        .filter_map(|e| {
            let t: TlsExtensionType = e.into();
            let v: u16 = t.into();
            if is_grease_u16(v) {
                None
            } else {
                Some(v.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("-");

    let ja3s_str = format!("{},{},{}", version, cipher.0, ext_ids);
    bytes_to_hex(Md5::digest(ja3s_str.as_bytes()).as_slice())
}

/// Result of decoding a TLS SNI hostname.
///
/// RFC 6066 §3 specifies that the `HostName` field is "represented as a byte
/// string using ASCII encoding". Internationalized names use A-labels (RFC 5890,
/// `xn--…` Punycode form), which are also ASCII. Three RFC violations are tracked
/// separately:
///
/// - `AsciiWithControl` — bytes are pure ASCII but contain at least one C0 control
///   byte (0x00–0x1F) or DEL (0x7F). RFC 6066 §3 requires ASCII; the DNS preferred
///   hostname syntax (RFC 952 / RFC 1123, inherited by RFC 5890 A-label
///   construction) restricts to letters, digits, and hyphens — no whitespace,
///   no control codes. An SNI like `"foo\x1b[31m.example"` is
///   simultaneously a protocol violation, a potential terminal-injection vector
///   (rendered via the terminal reporter, which escapes at display time per
///   ADR 0003), and a plausible evasion / log-poisoning / covert-channel signal.
/// - `NonAsciiUtf8` — bytes decode as valid UTF-8 but contain non-ASCII codepoints
///   (e.g. a raw U-label "café.example"). RFC 6066 says these MUST be Punycode-encoded
///   before transmission. Major TLS clients (rustls, Chrome/BoringSSL, Firefox/NSS,
///   curl/libcurl) all do this conversion automatically — so a non-ASCII SNI on the
///   wire indicates either a buggy custom client (often raw OpenSSL without IDNA prep)
///   or an attacker tool.
/// - `NonUtf8` — bytes can't decode as UTF-8 at all. Strictly malformed.
///
/// All three are surfaced as Anomaly findings; usually a client bug or adversarial
/// input, but worth forensic review.
enum SniValue {
    /// Hostname is pure ASCII with no C0 control bytes (0x00–0x1F) and no DEL
    /// (0x7F), so this classifier emits no finding. This arm is a "nothing to
    /// flag at the byte-control level" bucket, not a full RFC 6066 /
    /// DNS-preferred-hostname compliance check — empty hostnames (`""`),
    /// spaces, and other non-LDH printable ASCII still land here. May be a
    /// literal hostname or an A-label (Punycode) form like
    /// `xn--caf-dma.example`. Broader LDH compliance is out of scope (see
    /// issue #54's "Out of scope" section).
    Ascii(String),
    /// Hostname is pure ASCII but contains at least one C0 control byte
    /// (0x00–0x1F) or DEL (0x7F). `hostname` is the raw String (safe because
    /// pure ASCII is always valid UTF-8); `hex` is the lossless lowercase hex
    /// of the raw bytes for forensic evidence.
    AsciiWithControl { hostname: String, hex: String },
    /// Hostname decodes as valid UTF-8 but contains at least one byte ≥ 0x80.
    /// `hostname` is the decoded String (always valid UTF-8); `hex` is the lossless
    /// lowercase hex of the raw bytes for forensic evidence.
    NonAsciiUtf8 { hostname: String, hex: String },
    /// Hostname bytes failed UTF-8 decoding. `lossy` is the U+FFFD-replaced form
    /// for human display; `hex` is the lossless lowercase hex of the raw bytes.
    NonUtf8 { lossy: String, hex: String },
}

/// Returns true if any byte is a C0 control (0x00–0x1F) or DEL (0x7F).
///
/// Callers must ensure `s` is pure ASCII before calling — the byte-level check
/// is only meaningful for ASCII strings. For non-ASCII UTF-8 it would produce
/// false negatives on multi-byte codepoints (whose continuation bytes are all
/// ≥ 0x80) and is redundant since non-ASCII already signals a protocol violation
/// via `NonAsciiUtf8`.
fn contains_c0_or_del(s: &str) -> bool {
    debug_assert!(
        s.is_ascii(),
        "contains_c0_or_del requires ASCII input; non-ASCII UTF-8 \
         continuation bytes are ≥ 0x80 and would produce a false negative"
    );
    s.bytes().any(|b| b < 0x20 || b == 0x7f)
}

/// Extract SNI hostname from the parsed extension list.
///
/// Returns `None` if no SNI extension is present or the extension's hostname
/// list is empty. Otherwise returns an `SniValue` describing the first hostname
/// (we ignore additional entries — multi-name SNI is rare and treating only the
/// first matches the prior behavior).
fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<SniValue> {
    for ext in extensions {
        if let TlsExtension::SNI(list) = ext
            && let Some((_, hostname)) = list.first()
        {
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
        }
    }
    None
}

// ── per-flow state ────────────────────────────────────────────────────────────

struct TlsFlowState {
    client_buf: Vec<u8>,
    server_buf: Vec<u8>,
    client_hello_seen: bool,
    server_hello_seen: bool,
}

impl TlsFlowState {
    fn new() -> Self {
        TlsFlowState {
            client_buf: Vec::new(),
            server_buf: Vec::new(),
            client_hello_seen: false,
            server_hello_seen: false,
        }
    }

    /// Returns true once both hellos have been seen (no more buffering needed).
    fn done(&self) -> bool {
        self.client_hello_seen && self.server_hello_seen
    }
}

// ── analyzer ─────────────────────────────────────────────────────────────────

pub struct TlsAnalyzer {
    flows: HashMap<FlowKey, TlsFlowState>,
    sni_counts: HashMap<String, u64>,
    ja3_counts: HashMap<String, u64>,
    ja3s_counts: HashMap<String, u64>,
    version_counts: HashMap<u16, u64>,
    cipher_counts: HashMap<String, u64>,
    handshakes_seen: u64,
    parse_errors: u64,
    /// TLS records whose declared `payload_len` exceeded
    /// `MAX_RECORD_PAYLOAD` and were dropped before parsing. Counted
    /// separately from `parse_errors` so that DoS-protection drops are
    /// distinguishable from genuinely malformed records — see
    /// LESSON-P1.05 (CNV-PAT-002 follow-up).
    truncated_records: u64,
    all_findings: Vec<Finding>,
}

impl Default for TlsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsAnalyzer {
    pub fn new() -> Self {
        TlsAnalyzer {
            flows: HashMap::new(),
            sni_counts: HashMap::new(),
            ja3_counts: HashMap::new(),
            ja3s_counts: HashMap::new(),
            version_counts: HashMap::new(),
            cipher_counts: HashMap::new(),
            handshakes_seen: 0,
            parse_errors: 0,
            truncated_records: 0,
            all_findings: Vec::new(),
        }
    }

    // ── public accessors ──────────────────────────────────────────────────

    pub fn sni_counts(&self) -> &HashMap<String, u64> {
        &self.sni_counts
    }

    pub fn ja3_counts(&self) -> &HashMap<String, u64> {
        &self.ja3_counts
    }

    pub fn ja3s_counts(&self) -> &HashMap<String, u64> {
        &self.ja3s_counts
    }

    pub fn version_counts(&self) -> &HashMap<u16, u64> {
        &self.version_counts
    }

    pub fn parse_error_count(&self) -> u64 {
        self.parse_errors
    }

    /// TLS records dropped before parsing because their declared
    /// payload length exceeded `MAX_RECORD_PAYLOAD`. See LESSON-P1.05.
    pub fn truncated_record_count(&self) -> u64 {
        self.truncated_records
    }

    pub fn handshake_count(&self) -> u64 {
        self.handshakes_seen
    }

    // ── internal helpers ──────────────────────────────────────────────────

    fn increment<K: Eq + std::hash::Hash>(map: &mut HashMap<K, u64>, key: K, limit: usize) {
        if map.len() < limit || map.contains_key(&key) {
            *map.entry(key).or_insert(0) += 1;
        }
    }

    /// Process a single complete ClientHello.
    fn handle_client_hello(
        &mut self,
        ch: &tls_parser::TlsClientHelloContents<'_>,
        _flow_key: &FlowKey,
    ) {
        self.handshakes_seen += 1;

        let version = ch.version.0;
        Self::increment(&mut self.version_counts, version, MAX_MAP_ENTRIES);

        // Parse extensions (compute partial JA3 with empty fields on failure)
        let exts: Vec<TlsExtension<'_>> = match ch.ext {
            Some(raw) => match parse_tls_extensions(raw) {
                Ok((_, v)) => v,
                Err(_) => {
                    self.parse_errors += 1;
                    Vec::new()
                }
            },
            None => Vec::new(),
        };

        // SNI
        if let Some(sni) = extract_sni(&exts) {
            // Choose a map key that preserves uniqueness for non-UTF-8 cases.
            // Using `lossy` as a key would collapse distinct byte sequences whose
            // U+FFFD replacements happen to align — bad for forensic counting.
            // For Ascii, AsciiWithControl, and NonAsciiUtf8 the hostname is valid
            // UTF-8 with no collision risk, so use it directly. Terminal-safe
            // display of embedded control bytes is handled by the terminal
            // reporter per ADR 0003.
            let key = match &sni {
                SniValue::Ascii(s) => s.clone(),
                SniValue::AsciiWithControl { hostname, .. } => hostname.clone(),
                SniValue::NonAsciiUtf8 { hostname, .. } => hostname.clone(),
                SniValue::NonUtf8 { hex, .. } => format!("<non-utf8:{hex}>"),
            };
            Self::increment(&mut self.sni_counts, key, MAX_MAP_ENTRIES);

            // SNI encoding violations (control chars, non-ASCII UTF-8,
            // non-UTF-8 bytes) map to MITRE T1027 (Obfuscated Files or
            // Information): the technique is corrupting a protocol field
            // to evade inspection, not impersonating a legitimate
            // hostname (which would be T1036 Masquerading) or proving C2
            // abuse over the channel (T1071.001).
            match sni {
                SniValue::Ascii(_) => {} // No C0/DEL detected; no finding emitted at this layer.
                SniValue::AsciiWithControl { hostname, hex } => {
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Raw hostname interpolation — the data layer stores raw
                        // bytes per ADR 0003 (including embedded C0/DEL control
                        // codes). The terminal reporter escapes them at render
                        // time to prevent terminal-injection; JSON output is
                        // already safe via serde_json's RFC 8259 escaping.
                        summary: format!(
                            "TLS SNI contains ASCII control characters \
                             (RFC 6066 §3 requires ASCII; DNS preferred hostname \
                             syntax per RFC 952 / RFC 1123 restricts to letters, \
                             digits, and hyphens): {hostname}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_technique: Some("T1027".to_string()),
                        source_ip: None,
                        timestamp: None,
                        direction: Some(Direction::ClientToServer),
                    });
                }
                SniValue::NonAsciiUtf8 { hostname, hex } => {
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Raw hostname interpolation — the data layer stores raw
                        // bytes per ADR 0003. Terminal-safety (escaping control
                        // codes, etc.) is applied by the terminal reporter at
                        // render time, not here.
                        summary: format!(
                            "TLS SNI contains non-ASCII characters (RFC 6066 requires \
                             A-labels per RFC 5890): {hostname}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_technique: Some("T1027".to_string()),
                        source_ip: None,
                        timestamp: None,
                        direction: Some(Direction::ClientToServer),
                    });
                }
                SniValue::NonUtf8 { lossy, hex } => {
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Raw lossy interpolation — the data layer stores raw
                        // bytes (including any embedded ASCII control codes) per
                        // ADR 0003. The terminal reporter is responsible for
                        // escaping these for safe display; JSON output is already
                        // safe via serde_json's automatic RFC 8259 escaping.
                        summary: format!(
                            "TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_technique: Some("T1027".to_string()),
                        source_ip: None,
                        timestamp: None,
                        direction: Some(Direction::ClientToServer),
                    });
                }
            }
        }

        // JA3
        let (ja3_hash, _ja3_str) = compute_ja3(version, &ch.ciphers, &exts);
        Self::increment(&mut self.ja3_counts, ja3_hash, MAX_MAP_ENTRIES);

        // Weak cipher detection
        let weak: Vec<String> = ch
            .ciphers
            .iter()
            .filter(|&&id| is_weak_cipher(id))
            .map(|&id| cipher_name(id))
            .collect();

        if !weak.is_empty() {
            self.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "ClientHello offers weak cipher suites (NULL/anonymous/export)"
                    .to_string(),
                evidence: weak,
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
                direction: Some(Direction::ClientToServer),
            });
        }

        // Deprecated protocol version detection (SSLv2/SSLv3)
        if version <= 0x0300 {
            let version_name = match version {
                0x0200 => "SSL 2.0",
                0x0300 => "SSL 3.0",
                _ => "Unknown legacy SSL",
            };
            self.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: format!(
                    "ClientHello uses deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)"
                ),
                evidence: vec![format!("Version: 0x{version:04x} ({version_name})")],
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
            direction: Some(Direction::ClientToServer),
            });
        }
    }

    /// Process a single complete ServerHello.
    fn handle_server_hello(
        &mut self,
        sh: &tls_parser::TlsServerHelloContents<'_>,
        _flow_key: &FlowKey,
    ) {
        let version = sh.version.0;
        Self::increment(&mut self.version_counts, version, MAX_MAP_ENTRIES);

        let exts: Vec<TlsExtension<'_>> = match sh.ext {
            Some(raw) => match parse_tls_extensions(raw) {
                Ok((_, v)) => v,
                Err(_) => {
                    self.parse_errors += 1;
                    Vec::new()
                }
            },
            None => Vec::new(),
        };

        // JA3S
        let ja3s_hash = compute_ja3s(version, sh.cipher, &exts);
        Self::increment(&mut self.ja3s_counts, ja3s_hash, MAX_MAP_ENTRIES);

        // Cipher tracking
        let name = cipher_name(sh.cipher);
        Self::increment(&mut self.cipher_counts, name.clone(), MAX_MAP_ENTRIES);

        if is_weak_server_cipher(sh.cipher) {
            self.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::Medium,
                summary: format!("ServerHello selected weak cipher suite ({name})"),
                evidence: vec![format!("Selected cipher: {} (0x{:04x})", name, sh.cipher.0)],
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
                direction: Some(Direction::ServerToClient),
            });
        }

        // Deprecated protocol version — server selecting SSLv2/SSLv3 is critical
        if version <= 0x0300 {
            let version_name = match version {
                0x0200 => "SSL 2.0",
                0x0300 => "SSL 3.0",
                _ => "Unknown legacy SSL",
            };
            self.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: format!(
                    "ServerHello negotiated deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)"
                ),
                evidence: vec![format!("Version: 0x{version:04x} ({version_name})")],
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
            direction: Some(Direction::ServerToClient),
            });
        }
    }

    /// Extract complete TLS records from the given direction's buffer and process them.
    fn try_parse_records(&mut self, flow_key: &FlowKey, direction: Direction) {
        loop {
            // Need at least 5 bytes for a TLS record header.
            let buf_len = match self.flows.get(flow_key) {
                Some(s) => match direction {
                    Direction::ClientToServer => s.client_buf.len(),
                    Direction::ServerToClient => s.server_buf.len(),
                },
                None => return,
            };

            if buf_len < 5 {
                return;
            }

            // Peek at the length field without removing data yet.
            let (record_type, payload_len) = {
                let buf = match direction {
                    Direction::ClientToServer => &self.flows[flow_key].client_buf,
                    Direction::ServerToClient => &self.flows[flow_key].server_buf,
                };
                let record_type = buf[0];
                let payload_len = u16::from_be_bytes([buf[3], buf[4]]) as usize;
                (record_type, payload_len)
            };

            // Reject impossibly large records (DoS protection).
            //
            // LESSON-P1.05 / CNV-PAT-002 follow-up: bump the dedicated
            // `truncated_records` counter in addition to `parse_errors`
            // so JSON consumers can distinguish "record dropped because
            // its declared length blew the cap" (a capacity/DoS event)
            // from "record contents failed to parse" (a malformed-data
            // event). `parse_errors` is kept incremented to preserve
            // back-compatibility with existing dashboards.
            if payload_len > MAX_RECORD_PAYLOAD {
                self.parse_errors += 1;
                self.truncated_records += 1;
                if let Some(state) = self.flows.get_mut(flow_key) {
                    match direction {
                        Direction::ClientToServer => state.client_buf.clear(),
                        Direction::ServerToClient => state.server_buf.clear(),
                    };
                }
                return;
            }

            let total_record_len = 5 + payload_len;
            if buf_len < total_record_len {
                // Incomplete record — wait for more data.
                return;
            }

            // We have a complete record. Clone it out so we can parse without holding &self.
            let record_bytes: Vec<u8> = match direction {
                Direction::ClientToServer => {
                    self.flows[flow_key].client_buf[..total_record_len].to_vec()
                }
                Direction::ServerToClient => {
                    self.flows[flow_key].server_buf[..total_record_len].to_vec()
                }
            };

            // Drain consumed bytes from buffer.
            if let Some(state) = self.flows.get_mut(flow_key) {
                match direction {
                    Direction::ClientToServer => state.client_buf.drain(..total_record_len),
                    Direction::ServerToClient => state.server_buf.drain(..total_record_len),
                };
            }

            // Only process handshake records (0x16).
            if record_type != 0x16 {
                continue;
            }

            match parse_tls_plaintext(&record_bytes) {
                Ok((_rem, plaintext)) => {
                    for msg in &plaintext.msg {
                        match msg {
                            TlsMessage::Handshake(TlsMessageHandshake::ClientHello(ch)) => {
                                if let Some(state) = self.flows.get_mut(flow_key) {
                                    state.client_hello_seen = true;
                                }
                                self.handle_client_hello(ch, flow_key);
                            }
                            TlsMessage::Handshake(TlsMessageHandshake::ServerHello(sh)) => {
                                if let Some(state) = self.flows.get_mut(flow_key) {
                                    state.server_hello_seen = true;
                                }
                                self.handle_server_hello(sh, flow_key);
                            }
                            _ => {}
                        }
                    }
                }
                Err(NomErr::Incomplete(_)) => {
                    // Should not happen since we verified length — count as error if it does.
                    self.parse_errors += 1;
                }
                Err(_) => {
                    self.parse_errors += 1;
                }
            }
        }
    }
}

// ── StreamHandler ─────────────────────────────────────────────────────────────

impl StreamHandler for TlsAnalyzer {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], _offset: u64) {
        // Check whether this flow is already done before we get a mutable ref.
        let done = self.flows.get(flow_key).is_some_and(|s| s.done());
        if done {
            return;
        }

        {
            let state = self
                .flows
                .entry(flow_key.clone())
                .or_insert_with(TlsFlowState::new);
            match direction {
                Direction::ClientToServer => {
                    let remaining = MAX_BUF.saturating_sub(state.client_buf.len());
                    if remaining > 0 {
                        let to_copy = data.len().min(remaining);
                        state.client_buf.extend_from_slice(&data[..to_copy]);
                    }
                }
                Direction::ServerToClient => {
                    let remaining = MAX_BUF.saturating_sub(state.server_buf.len());
                    if remaining > 0 {
                        let to_copy = data.len().min(remaining);
                        state.server_buf.extend_from_slice(&data[..to_copy]);
                    }
                }
            }
        }

        self.try_parse_records(flow_key, direction);
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, _reason: CloseReason) {
        self.flows.remove(flow_key);
    }
}

// ── StreamAnalyzer ────────────────────────────────────────────────────────────

impl StreamAnalyzer for TlsAnalyzer {
    fn name(&self) -> &'static str {
        "TLS"
    }

    fn summarize(&self) -> AnalysisSummary {
        // LESSON-P2.09: BTreeMap so the JSON output keys are
        // alphabetically ordered and deterministic across runs.
        let mut detail: std::collections::BTreeMap<String, serde_json::Value> =
            std::collections::BTreeMap::new();

        // Top SNIs (top 20 by count)
        let mut top_snis: Vec<_> = self.sni_counts.iter().collect();
        top_snis.sort_by(|a, b| b.1.cmp(a.1));
        let top_snis: Vec<&str> = top_snis.iter().take(20).map(|(k, _)| k.as_str()).collect();
        detail.insert("top_snis".to_string(), serde_json::json!(top_snis));

        detail.insert("ja3_hashes".to_string(), serde_json::json!(self.ja3_counts));
        detail.insert(
            "ja3s_hashes".to_string(),
            serde_json::json!(self.ja3s_counts),
        );
        detail.insert(
            "tls_versions".to_string(),
            serde_json::json!(
                self.version_counts
                    .iter()
                    .map(|(k, v)| (k.to_string(), *v))
                    .collect::<HashMap<String, u64>>()
            ),
        );
        detail.insert(
            "cipher_suites".to_string(),
            serde_json::json!(self.cipher_counts),
        );
        detail.insert(
            "parse_errors".to_string(),
            serde_json::json!(self.parse_errors),
        );
        detail.insert(
            "truncated_records".to_string(),
            serde_json::json!(self.truncated_records),
        );

        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.handshakes_seen,
            detail,
        }
    }

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}

// ---- Test-only seams (STORY-021 adversarial-pass-2 remediation / F-W11P2-001) ----
//
// These seams expose `TlsAnalyzer`-internal state to integration tests so they
// can verify that `TlsAnalyzer` does NOT apply the engine-local `MAX_FINDINGS`
// cap (BC-2.04.024 invariant 4). All follow the `#[doc(hidden)] pub fn` append
// pattern established in `src/reassembly/mod.rs`. MUST NOT be called from
// production code.
impl TlsAnalyzer {
    /// Test-only accessor for the count of accumulated findings.
    ///
    /// Exposes `self.all_findings.len()` so integration tests can verify
    /// that `TlsAnalyzer` does NOT apply the `MAX_FINDINGS` cap used by
    /// `TcpReassembler` (BC-2.04.024 invariant 4 / AC-007b — analyzer non-cap).
    /// The analyzer pushes to `all_findings` unconditionally — there is no local cap.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn all_findings_len_for_testing(&self) -> usize {
        self.all_findings.len()
    }

    /// Test-only direct push of a [`Finding`] into the analyzer's findings vec.
    ///
    /// Bypasses the normal analyzer detection logic so tests can pre-fill
    /// `all_findings` to arbitrary lengths (e.g. > 10,000) to verify that
    /// `TlsAnalyzer` has NO local cap analogous to `TcpReassembler::MAX_FINDINGS`
    /// (BC-2.04.024 invariant 4 / AC-007b — analyzer non-cap companion test).
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn push_finding_for_testing(&mut self, finding: Finding) {
        self.all_findings.push(finding);
    }
}

// ── JA3 / JA3S property tests (LESSON-P2.04) ─────────────────────────────────
//
// Inline `#[cfg(test)]` module so the property tests can reach the private
// `compute_ja3`, `compute_ja3s`, `is_grease_u16`, and `bytes_to_hex`
// functions. The JA3 algorithm is a fingerprint: its core invariants
// (determinism, fixed output format, GREASE-invariance, order-sensitivity)
// hold over the entire input space, which is exactly what property-based
// testing exercises better than example-based tests.
#[cfg(test)]
mod ja3_property_tests {
    use super::*;
    use proptest::prelude::*;

    /// The 16 canonical TLS GREASE values (RFC 8701): both bytes are
    /// `0xNA` for N in 0x0..=0xF.
    const GREASE_VALUES: [u16; 16] = [
        0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a, 0x4a4a, 0x5a5a, 0x6a6a, 0x7a7a, 0x8a8a, 0x9a9a, 0xaaaa,
        0xbaba, 0xcaca, 0xdada, 0xeaea, 0xfafa,
    ];

    #[test]
    fn is_grease_u16_matches_all_canonical_grease_values() {
        // Every RFC 8701 GREASE value must be recognized.
        for &g in &GREASE_VALUES {
            assert!(is_grease_u16(g), "0x{g:04x} must be recognized as GREASE");
        }
    }

    proptest! {
        #[test]
        fn is_grease_u16_matches_nibble_bitmask_contract(v in any::<u16>()) {
            // `is_grease_u16` implements the *bitmask* GREASE test
            // `(v & 0x0F0F) == 0x0A0A` — i.e. "the low nibble of each
            // byte is 0xA". This is the form used widely in JA3 tooling.
            //
            // NOTE: the bitmask is broader than RFC 8701's strict
            // definition. RFC 8701 lists exactly 16 GREASE values
            // (0x0A0A, 0x1A1A … 0xFAFA — both *bytes* equal). The
            // bitmask additionally accepts 240 non-canonical values like
            // 0x0A1A and 0xCABA. In practice this is harmless: IANA has
            // assigned no real cipher / extension IDs of `0x_A_A` form
            // outside the 16 GREASE values, so nothing real is wrongly
            // filtered. A strict 16-value check is recorded as a P2
            // follow-up rather than fixed here (this is a test PR).
            //
            // The property re-derives the predicate via independent
            // nibble extraction so it is a genuine cross-check, not a
            // tautology against the implementation's own expression.
            let lo_nibble_hi_byte = (v >> 8) & 0x0F;
            let lo_nibble_lo_byte = v & 0x0F;
            let expected = lo_nibble_hi_byte == 0x0A && lo_nibble_lo_byte == 0x0A;
            prop_assert_eq!(is_grease_u16(v), expected);
        }

        #[test]
        fn compute_ja3s_is_deterministic_and_hex(
            version in any::<u16>(),
            cipher in any::<u16>(),
        ) {
            // A fingerprint must be reproducible: identical inputs yield
            // identical hashes.
            let a = compute_ja3s(version, TlsCipherSuiteID(cipher), &[]);
            let b = compute_ja3s(version, TlsCipherSuiteID(cipher), &[]);
            prop_assert_eq!(&a, &b);
            // MD5 hex is always 32 lowercase-hex characters.
            prop_assert_eq!(a.len(), 32);
            prop_assert!(a.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        }

        #[test]
        fn compute_ja3_has_five_fields_and_hex_hash(
            version in any::<u16>(),
            ciphers in prop::collection::vec(any::<u16>(), 0..20),
        ) {
            let cipher_ids: Vec<TlsCipherSuiteID> =
                ciphers.iter().map(|&c| TlsCipherSuiteID(c)).collect();
            let (hash, ja3_str) = compute_ja3(version, &cipher_ids, &[]);
            // The JA3 string is always exactly 5 comma-separated fields:
            // version,ciphers,extensions,curves,point_formats.
            prop_assert_eq!(ja3_str.matches(',').count(), 4);
            // Hash is a 32-char lowercase-hex MD5 digest.
            prop_assert_eq!(hash.len(), 32);
            prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
            // First field is always the version, verbatim.
            let version_prefix = format!("{version},");
            prop_assert!(ja3_str.starts_with(&version_prefix));
        }

        #[test]
        fn compute_ja3_is_grease_invariant(
            version in any::<u16>(),
            ciphers in prop::collection::vec(any::<u16>(), 0..15),
            grease_idx in 0usize..16,
            insert_pos in 0usize..16,
        ) {
            // GREASE values are filtered before hashing, so inserting one
            // anywhere into the cipher list must NOT change the JA3 hash.
            let base: Vec<TlsCipherSuiteID> =
                ciphers.iter().map(|&c| TlsCipherSuiteID(c)).collect();
            let (base_hash, _) = compute_ja3(version, &base, &[]);

            let mut with_grease = base.clone();
            let pos = insert_pos.min(with_grease.len());
            with_grease.insert(pos, TlsCipherSuiteID(GREASE_VALUES[grease_idx]));
            let (grease_hash, _) = compute_ja3(version, &with_grease, &[]);

            prop_assert_eq!(base_hash, grease_hash);
        }

        #[test]
        fn compute_ja3_is_order_sensitive(
            version in any::<u16>(),
            a in any::<u16>(),
            b in any::<u16>(),
        ) {
            // JA3 is order-sensitive by design: the cipher list order is
            // part of the fingerprint. Two distinct non-GREASE ciphers in
            // opposite order must produce different hashes.
            prop_assume!(a != b);
            prop_assume!(!is_grease_u16(a) && !is_grease_u16(b));
            let ab = [TlsCipherSuiteID(a), TlsCipherSuiteID(b)];
            let ba = [TlsCipherSuiteID(b), TlsCipherSuiteID(a)];
            let (hash_ab, _) = compute_ja3(version, &ab, &[]);
            let (hash_ba, _) = compute_ja3(version, &ba, &[]);
            prop_assert_ne!(hash_ab, hash_ba);
        }

        #[test]
        fn bytes_to_hex_roundtrips_length_and_alphabet(
            bytes in prop::collection::vec(any::<u8>(), 0..64),
        ) {
            // bytes_to_hex must always emit exactly 2 lowercase-hex chars
            // per input byte.
            let hex = bytes_to_hex(&bytes);
            prop_assert_eq!(hex.len(), bytes.len() * 2);
            prop_assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        }
    }
}
