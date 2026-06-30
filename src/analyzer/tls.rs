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

use chrono::DateTime;
use md5::{Digest, Md5};
// `parse_tls_message_handshake` is used by both direction carry drain loops
// (AC-144-002 / AC-145-001).  ClientToServer dispatches ClientHello (0x01);
// ServerToClient dispatches ServerHello (0x02) (ADR-011 Decision 4).
use tls_parser::{
    TlsCipherSuite, TlsCipherSuiteID, TlsExtension, TlsExtensionType, TlsMessage,
    TlsMessageHandshake, parse_tls_extensions, parse_tls_message_handshake,
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
    /// Most-recent on_data capture timestamp for this flow; used at Finding
    /// emission sites to attach capture-relative pcap provenance.
    /// Updated on every `on_data` call; keyed per-flow (VP-014 cross-flow
    /// isolation invariant).
    last_ts: u32,
    /// Handshake-message carry buffer for the ClientToServer direction.
    ///
    /// Accumulates 0x16 record payloads across multiple `try_parse_records`
    /// calls until a complete handshake message (4-byte header + body_len
    /// body bytes) is available. Initialized to `Vec::new()` in
    /// `TlsFlowState::new()`; dropped automatically when `on_flow_close`
    /// removes the `TlsFlowState` from the `flows` map (BC-2.07.040).
    /// NO `hs_carry_abandoned` flag exists anywhere (BC-2.07.039 Invariant 4).
    /// AC-144-001 / ADR-011 Decision 1.
    client_hs_carry: Vec<u8>,
    /// Handshake-message carry buffer for the ServerToClient direction.
    ///
    /// Symmetric companion to `client_hs_carry`. Initialized to `Vec::new()`.
    /// AC-144-001 / ADR-011 Decision 1.
    server_hs_carry: Vec<u8>,
}

impl TlsFlowState {
    fn new() -> Self {
        TlsFlowState {
            client_buf: Vec::new(),
            server_buf: Vec::new(),
            client_hello_seen: false,
            server_hello_seen: false,
            last_ts: 0,
            client_hs_carry: Vec::new(),
            server_hs_carry: Vec::new(),
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
    /// Aggregate count of handshake carry buffer overflow events.
    ///
    /// Incremented each time a 0x16 record payload would push a direction's
    /// carry buffer past `MAX_BUF` (Decision 5 buffer-fill guard) OR when
    /// `body_len > MAX_BUF` inside the drain loop (Decision 4 body_len-spoof
    /// guard). This is an AGGREGATE counter on `TlsAnalyzer` — NOT on
    /// `TlsFlowState` — and is NOT reset at `on_flow_close`. Mirrors the
    /// existing `truncated_records` aggregate counter pattern.
    /// AC-144-001 / ADR-011 Decision 1 / BC-2.07.039 Invariant 5.
    handshake_reassembly_overflows: u64,
    /// Aggregate count of per-direction TCP-segment buffer tail-drop events.
    ///
    /// Incremented each time `on_data` detects that the incoming data length
    /// exceeds the remaining capacity of `client_buf` or `server_buf`
    /// (`data.len() > remaining` where `remaining = MAX_BUF.saturating_sub(buf.len())`).
    /// This is an AGGREGATE counter on `TlsAnalyzer` — NOT on `TlsFlowState` — and is
    /// NOT reset at `on_flow_close`. Mirrors the `truncated_records` / `handshake_reassembly_overflows`
    /// aggregate counter pattern.
    /// AC-146-001 / BC-2.07.043 Invariants 1–3.
    buffer_saturation_drops: u64,
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
            handshake_reassembly_overflows: 0,
            buffer_saturation_drops: 0,
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
    ///
    /// `last_ts` is the per-flow capture timestamp (BC-2.09.007): attached as
    /// `Some(DateTime<Utc>)` to every Finding emitted from this handshake.
    fn handle_client_hello(
        &mut self,
        ch: &tls_parser::TlsClientHelloContents<'_>,
        _flow_key: &FlowKey,
        last_ts: u32,
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
                        mitre_techniques: vec!["T1027".to_string()],
                        source_ip: None,
                        // BC-2.09.007 post-1: capture-relative pcap timestamp.
                        timestamp: DateTime::from_timestamp(last_ts as i64, 0),
                        direction: Some(Direction::ClientToServer),
                    });
                }
                SniValue::NonAsciiUtf8 { hostname, hex } => {
                    // Scan for embedded C0/DEL control bytes. Valid UTF-8 continuation
                    // bytes are 0x80–0xBF, so scanning `hostname.bytes()` for
                    // `b < 0x20 || b == 0x7f` is safe — no false positives from
                    // multi-byte codepoints. When present, the summary is enriched so
                    // a SOC analyst grepping for "control" finds this case too
                    // (BC-TLS-037 / issue #104).
                    let has_control = hostname.bytes().any(|b| b < 0x20 || b == 0x7f);
                    let control_clause = if has_control {
                        " and ASCII control bytes"
                    } else {
                        ""
                    };
                    self.all_findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Low,
                        // Raw hostname interpolation — the data layer stores raw
                        // bytes per ADR 0003. Terminal-safety (escaping control
                        // codes, etc.) is applied by the terminal reporter at
                        // render time, not here.
                        summary: format!(
                            "TLS SNI contains non-ASCII characters{control_clause} \
                             (RFC 6066 requires A-labels per RFC 5890): {hostname}"
                        ),
                        evidence: vec![format!("hex: {hex}")],
                        mitre_techniques: vec!["T1027".to_string()],
                        source_ip: None,
                        // BC-2.09.007 post-1: capture-relative pcap timestamp.
                        timestamp: DateTime::from_timestamp(last_ts as i64, 0),
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
                        mitre_techniques: vec!["T1027".to_string()],
                        source_ip: None,
                        // BC-2.09.007 post-1: capture-relative pcap timestamp.
                        timestamp: DateTime::from_timestamp(last_ts as i64, 0),
                        direction: Some(Direction::ClientToServer),
                    });
                }
            }
        }

        // JA3
        let (ja3_hash, _ja3_str) = compute_ja3(version, &ch.ciphers, &exts);
        Self::increment(&mut self.ja3_counts, ja3_hash, MAX_MAP_ENTRIES);

        // Weak cipher detection
        //
        // Cap the evidence vec at WEAK_CIPHER_EVIDENCE_CAP entries to bound the
        // transient String allocation.  Input is already bounded by
        // MAX_RECORD_PAYLOAD (18,432 bytes → ≤9,216 cipher IDs), so this is
        // LOW-SEVERITY HARDENING — not a security/DoS fix (CWE-405 requires
        // asymmetric amplification; this allocation is linear and bounded).
        const WEAK_CIPHER_EVIDENCE_CAP: usize = 64;
        let total_weak = ch.ciphers.iter().filter(|&&id| is_weak_cipher(id)).count();
        let mut weak: Vec<String> = ch
            .ciphers
            .iter()
            .filter(|&&id| is_weak_cipher(id))
            .take(WEAK_CIPHER_EVIDENCE_CAP)
            .map(|&id| cipher_name(id))
            .collect();
        if total_weak > WEAK_CIPHER_EVIDENCE_CAP {
            weak.push(format!("(+{} more)", total_weak - WEAK_CIPHER_EVIDENCE_CAP));
        }

        if !weak.is_empty() {
            self.all_findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "ClientHello offers weak cipher suites (NULL/anonymous/export)"
                    .to_string(),
                evidence: weak,
                mitre_techniques: vec![],
                source_ip: None,
                // BC-2.09.007 post-1: capture-relative pcap timestamp.
                timestamp: DateTime::from_timestamp(last_ts as i64, 0),
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
                mitre_techniques: vec![],
                source_ip: None,
                // BC-2.09.007 post-1: capture-relative pcap timestamp.
                timestamp: DateTime::from_timestamp(last_ts as i64, 0),
            direction: Some(Direction::ClientToServer),
            });
        }
    }

    /// Process a single complete ServerHello.
    ///
    /// `last_ts` is the per-flow capture timestamp (BC-2.09.007): attached as
    /// `Some(DateTime<Utc>)` to every Finding emitted from this handshake.
    fn handle_server_hello(
        &mut self,
        sh: &tls_parser::TlsServerHelloContents<'_>,
        _flow_key: &FlowKey,
        last_ts: u32,
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
                mitre_techniques: vec![],
                source_ip: None,
                // BC-2.09.007 post-1: capture-relative pcap timestamp.
                timestamp: DateTime::from_timestamp(last_ts as i64, 0),
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
                mitre_techniques: vec![],
                source_ip: None,
                // BC-2.09.007 post-1: capture-relative pcap timestamp.
                timestamp: DateTime::from_timestamp(last_ts as i64, 0),
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

            // Guard-before-allocate (CR-010): skip the heap allocation for
            // non-handshake records. Only 0x16 (Handshake) records need to
            // be cloned for parsing; all other content types (0x14
            // ChangeCipherSpec, 0x15 Alert, 0x17 ApplicationData, etc.) are
            // drained and discarded without any per-record Vec allocation.
            //
            // NOTE: this guard must remain AFTER the `buf_len < total_record_len`
            // check above. That check guarantees the buffer holds at least
            // `total_record_len` bytes, making the `drain(..total_record_len)`
            // range valid. Hoisting the guard before that check would allow
            // a partial non-handshake record to be drained into a panic.
            if record_type != 0x16 {
                // Drain the non-handshake record from the buffer and loop.
                // `Drain`'s `Drop` impl performs the removal; the trailing
                // semicolon discards the value, consistent with the drain
                // statement on the 0x16 path below.
                match self.flows.get_mut(flow_key) {
                    Some(state) => {
                        match direction {
                            Direction::ClientToServer => state.client_buf.drain(..total_record_len),
                            Direction::ServerToClient => state.server_buf.drain(..total_record_len),
                        };
                    }
                    // Flow absent — evicted by an earlier on_flow_close on this
                    // same thread before we reached this point. Return rather than
                    // continue to avoid looping on a non-advancing buffer; mirrors
                    // the `None => return` contract at the buf_len read site above.
                    None => return,
                }
                continue;
            }

            // We have a complete handshake record. Clone it out so we can
            // parse without holding &self.
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

            // BC-2.09.007: retrieve per-flow last_ts before parsing the record.
            // Both handle_client_hello and handle_server_hello need it to attach
            // the capture-relative timestamp to any emitted Findings.
            let last_ts = self.flows.get(flow_key).map(|s| s.last_ts).unwrap_or(0);

            // AC-144-002 / AC-145-001 / BC-2.07.038 / BC-2.07.041: direction-parameterized
            // handshake carry path.
            //
            // Both ClientToServer and ServerToClient use the same cursor-based drain loop
            // design (SEC-001 O(carry_len) guarantee).  ClientToServer accumulates into
            // `client_hs_carry` and dispatches ClientHello (msg_type 0x01); ServerToClient
            // accumulates into `server_hs_carry` and dispatches ServerHello (msg_type 0x02).
            // Overflow/spoof-guard invariants are identical for both directions
            // (BC-2.07.041 v1.2 Invariant 2; ADR-011 Decision 4/5).
            match direction {
                Direction::ClientToServer => {
                    // ── ClientToServer carry path (AC-144-002) ──────────────────────
                    let record_payload = &record_bytes[5..];

                    // Step 1: Overflow check BEFORE append (BC-2.07.039 Invariant 3 /
                    // ADR-011 Decision 5). If carry + payload would exceed MAX_BUF,
                    // clear carry and increment the aggregate overflow counter; do NOT
                    // increment parse_errors; continue to the next record.
                    let carry_len_before = self
                        .flows
                        .get(flow_key)
                        .map(|s| s.client_hs_carry.len())
                        .unwrap_or(0);

                    if carry_len_before + record_payload.len() > MAX_BUF {
                        if let Some(state) = self.flows.get_mut(flow_key) {
                            state.client_hs_carry.clear();
                        }
                        // SEC-003: saturating_add to avoid theoretical overflow-check panic
                        // under `overflow-checks = true` (release profile). This counter is
                        // an aggregate diagnostic; saturation at u64::MAX is safe and intentional.
                        self.handshake_reassembly_overflows =
                            self.handshake_reassembly_overflows.saturating_add(1);
                        continue;
                    }

                    // Step 2: Append payload to client_hs_carry.
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.client_hs_carry.extend_from_slice(record_payload);
                    }

                    // Step 3: Drain loop — consume complete handshake messages from carry.
                    //
                    // SEC-001 (CWE-400/834): CURSOR-BASED drain to prevent quadratic CPU
                    // amplification. The previous approach called `carry.drain(..k)` once
                    // PER message, which is O(remaining-after-k) per call — O(N·L) total
                    // for N coalesced messages in a carry of L bytes. An attacker packing
                    // thousands of zero-body-length messages into a single MAX_RECORD_PAYLOAD
                    // record could cause ~40 MB of memmove per `on_data` call.
                    //
                    // Fix: advance a local `consumed` cursor per message; perform EXACTLY
                    // ONE `carry.drain(..consumed)` after the loop exits (O(carry_len) total).
                    //
                    // All slice reads use `&carry[consumed..]` during the loop; the single
                    // drain is issued via a separate `get_mut` borrow after the loop to
                    // satisfy Rust's borrow rules (immutable loop reads → mutable post-loop
                    // drain).
                    //
                    // Semantics preserved:
                    //   - Each message advances cursor by exactly 4 + body_len
                    //     (BC-2.07.038 Postcondition 4 / Invariant 2).
                    //   - Decision-4 body_len-spoof guard: carry.clear() + overflows+1 + break.
                    //   - Decision-5 overflow guard (Step 1, before append): unchanged.
                    //   - Partial trailing messages are NOT consumed (cursor not advanced);
                    //     they remain in carry for the next on_data call.
                    //   - Clone for dispatch: only msg_type==0x01 clones the message bytes
                    //     (4 + body_len, bounded ≤ 65,540). Non-dispatched types advance the
                    //     cursor without any heap allocation.
                    //
                    // BC-2.07.042: back-to-back coalesced messages are each dispatched.
                    let mut consumed: usize = 0;
                    // Track whether Decision-4 fired (body_len spoof) so we can clear the
                    // carry and skip the normal single-drain after the loop.
                    let mut decision4_fired = false;
                    loop {
                        // Read current carry state at the cursor position.
                        let (carry_len, msg_type, body_len) = {
                            let state = match self.flows.get(flow_key) {
                                Some(s) => s,
                                None => break,
                            };
                            let carry = &state.client_hs_carry;
                            if carry.len() - consumed < 4 {
                                break;
                            }
                            let mt = carry[consumed];
                            let bl = ((carry[consumed + 1] as usize) << 16)
                                | ((carry[consumed + 2] as usize) << 8)
                                | (carry[consumed + 3] as usize);
                            (carry.len(), mt, bl)
                        };

                        // Decision-4: body_len > MAX_BUF → body_len-spoof guard.
                        // Clear carry, increment overflow counter, break (ADR-011 Decision 4).
                        // Note: consumed bytes up to this point are discarded by the clear().
                        if body_len > MAX_BUF {
                            if let Some(state) = self.flows.get_mut(flow_key) {
                                state.client_hs_carry.clear();
                            }
                            self.handshake_reassembly_overflows =
                                self.handshake_reassembly_overflows.saturating_add(1);
                            decision4_fired = true;
                            break;
                        }

                        // Incomplete: body not yet fully arrived — wait for next record.
                        if carry_len - consumed < 4 + body_len {
                            break;
                        }

                        // Dispatch on msg_type:
                        // 0x01 → ClientHello via parse_tls_message_handshake.
                        //   Ok(ClientHello): set client_hello_seen, call handle_client_hello.
                        //   Err or Ok(non-CH): parse_errors+1, no finding (PC-9).
                        // 0x02 → STORY-145 scope (ServerHello on server direction).
                        //   Not reachable here (ClientToServer direction).
                        // Other: consume silently (BC-2.07.038 Inv-1; BC-2.07.042 EC-002).
                        // Clone only for msg_type==0x01 (the dispatch path). Non-dispatched
                        // types advance the cursor without any heap allocation.
                        match msg_type {
                            0x01 => {
                                // Clone the complete message bytes for parsing so we can
                                // dispatch (which takes &mut self) without holding a borrow.
                                let msg_bytes: Vec<u8> = {
                                    let state = match self.flows.get(flow_key) {
                                        Some(s) => s,
                                        None => break,
                                    };
                                    state.client_hs_carry[consumed..consumed + 4 + body_len]
                                        .to_vec()
                                };
                                match parse_tls_message_handshake(&msg_bytes) {
                                    Ok((
                                        _rem,
                                        TlsMessage::Handshake(TlsMessageHandshake::ClientHello(
                                            ref ch,
                                        )),
                                    )) => {
                                        if let Some(state) = self.flows.get_mut(flow_key) {
                                            state.client_hello_seen = true;
                                        }
                                        self.handle_client_hello(ch, flow_key, last_ts);
                                    }
                                    Ok(_) => {
                                        // Unexpected non-ClientHello for msg_type 0x01.
                                        self.parse_errors += 1;
                                    }
                                    Err(_) => {
                                        // Malformed assembled body (PC-9).
                                        self.parse_errors += 1;
                                    }
                                }
                            }
                            _ => {
                                // Other handshake types (Certificate=0x0b, etc.):
                                // consume silently; parse_errors NOT incremented
                                // (BC-2.07.038 Invariant 1; BC-2.07.042 EC-002).
                            }
                        }

                        // Advance cursor by exactly 4 + body_len regardless of parse outcome
                        // (BC-2.07.038 Postcondition 4 / Invariant 2).
                        consumed += 4 + body_len;
                    }

                    // Single drain after the loop: O(carry_len) total, not O(carry_len²).
                    // Skipped when Decision-4 fired (carry was already cleared above).
                    if !decision4_fired
                        && consumed > 0
                        && let Some(state) = self.flows.get_mut(flow_key)
                    {
                        state.client_hs_carry.drain(..consumed);
                    }
                }

                Direction::ServerToClient => {
                    // ── ServerToClient carry path (AC-145-001) ──────────────────────
                    //
                    // Symmetric to the ClientToServer carry path above.  The record
                    // payload is appended to `server_hs_carry`; the drain loop
                    // dispatches complete ServerHello messages (msg_type 0x02) via
                    // `parse_tls_message_handshake` (ADR-011 Decision 4).  All
                    // overflow and spoof-guard invariants are identical to the
                    // ClientToServer direction (BC-2.07.041 v1.2 Invariant 2).
                    let record_payload = &record_bytes[5..];

                    // Step 1: Overflow check BEFORE append.
                    let carry_len_before = self
                        .flows
                        .get(flow_key)
                        .map(|s| s.server_hs_carry.len())
                        .unwrap_or(0);

                    if carry_len_before + record_payload.len() > MAX_BUF {
                        if let Some(state) = self.flows.get_mut(flow_key) {
                            state.server_hs_carry.clear();
                        }
                        self.handshake_reassembly_overflows =
                            self.handshake_reassembly_overflows.saturating_add(1);
                        continue;
                    }

                    // Step 2: Append payload to server_hs_carry.
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.server_hs_carry.extend_from_slice(record_payload);
                    }

                    // Step 3: Drain loop — consume complete handshake messages.
                    //
                    // Same SEC-001 cursor-based O(carry_len) design as the
                    // ClientToServer direction above.
                    let mut consumed: usize = 0;
                    let mut decision4_fired = false;
                    loop {
                        let (carry_len, msg_type, body_len) = {
                            let state = match self.flows.get(flow_key) {
                                Some(s) => s,
                                None => break,
                            };
                            let carry = &state.server_hs_carry;
                            if carry.len() - consumed < 4 {
                                break;
                            }
                            let mt = carry[consumed];
                            let bl = ((carry[consumed + 1] as usize) << 16)
                                | ((carry[consumed + 2] as usize) << 8)
                                | (carry[consumed + 3] as usize);
                            (carry.len(), mt, bl)
                        };

                        // Decision-4: body_len > MAX_BUF → body_len-spoof guard.
                        if body_len > MAX_BUF {
                            if let Some(state) = self.flows.get_mut(flow_key) {
                                state.server_hs_carry.clear();
                            }
                            self.handshake_reassembly_overflows =
                                self.handshake_reassembly_overflows.saturating_add(1);
                            decision4_fired = true;
                            break;
                        }

                        // Incomplete: body not yet fully arrived.
                        if carry_len - consumed < 4 + body_len {
                            break;
                        }

                        // Dispatch on msg_type:
                        // 0x02 → ServerHello via parse_tls_message_handshake.
                        //   Ok(ServerHello): set server_hello_seen, call handle_server_hello.
                        //   Err or Ok(non-SH): parse_errors+1.
                        // Other: consume silently (BC-2.07.038 Inv-1).
                        match msg_type {
                            0x02 => {
                                let msg_bytes: Vec<u8> = {
                                    let state = match self.flows.get(flow_key) {
                                        Some(s) => s,
                                        None => break,
                                    };
                                    state.server_hs_carry[consumed..consumed + 4 + body_len]
                                        .to_vec()
                                };
                                match parse_tls_message_handshake(&msg_bytes) {
                                    Ok((
                                        _rem,
                                        TlsMessage::Handshake(TlsMessageHandshake::ServerHello(
                                            ref sh,
                                        )),
                                    )) => {
                                        if let Some(state) = self.flows.get_mut(flow_key) {
                                            state.server_hello_seen = true;
                                        }
                                        self.handle_server_hello(sh, flow_key, last_ts);
                                    }
                                    Ok(_) => {
                                        self.parse_errors += 1;
                                    }
                                    Err(_) => {
                                        self.parse_errors += 1;
                                    }
                                }
                            }
                            _ => {
                                // Other handshake types: consume silently
                                // (BC-2.07.038 Invariant 1).
                            }
                        }

                        consumed += 4 + body_len;
                    }

                    // Single drain after the loop: O(carry_len) total.
                    if !decision4_fired
                        && consumed > 0
                        && let Some(state) = self.flows.get_mut(flow_key)
                    {
                        state.server_hs_carry.drain(..consumed);
                    }
                }
            }
        }
    }
}

// ── StreamHandler ─────────────────────────────────────────────────────────────

impl StreamHandler for TlsAnalyzer {
    fn on_data(
        &mut self,
        flow_key: &FlowKey,
        direction: Direction,
        data: &[u8],
        _offset: u64,
        timestamp: u32,
    ) {
        // Check whether this flow is already done before we get a mutable ref.
        let done = self.flows.get(flow_key).is_some_and(|s| s.done());
        if done {
            return;
        }

        // AC-146-002 / BC-2.07.043 Invariant 4: compute did_drop INSIDE the &mut state
        // block (borrow-constraint mandated); increment self.buffer_saturation_drops
        // AFTER the block closes (mutable borrow on self.flows released).
        let did_drop;
        {
            let state = self
                .flows
                .entry(flow_key.clone())
                .or_insert_with(TlsFlowState::new);
            // BC-2.04.055 postcondition 3: update per-flow last-seen timestamp on
            // every on_data call.  Keyed by FlowKey (VP-014 cross-flow isolation).
            state.last_ts = timestamp;
            match direction {
                Direction::ClientToServer => {
                    let remaining = MAX_BUF.saturating_sub(state.client_buf.len());
                    // AC-146-002: detect tail-drop condition (strictly greater, NOT >=).
                    // data.len() > remaining covers both partial-drop (remaining>0, 1+
                    // bytes truncated) and full-drop (remaining==0) paths. The form
                    // `to_copy < data.len()` would miss the full-drop case because to_copy
                    // is computed only inside the `if remaining > 0` arm (ADR-011 C-3).
                    did_drop = data.len() > remaining;
                    if remaining > 0 {
                        let to_copy = data.len().min(remaining);
                        state.client_buf.extend_from_slice(&data[..to_copy]);
                    }
                }
                Direction::ServerToClient => {
                    let remaining = MAX_BUF.saturating_sub(state.server_buf.len());
                    // AC-146-002: symmetric drop detection for the ServerToClient arm.
                    // Same aggregate counter — BC-2.07.043 Postcondition 3.
                    did_drop = data.len() > remaining;
                    if remaining > 0 {
                        let to_copy = data.len().min(remaining);
                        state.server_buf.extend_from_slice(&data[..to_copy]);
                    }
                }
            }
        }
        // AC-146-002: increment AFTER the &mut state block closes (borrow released).
        // Placement is between the buffer-append block and the try_parse_records call
        // (ADR-011 Decision 1 / BC-2.07.043 Invariant 4). Byte-drop semantics unchanged.
        if did_drop {
            self.buffer_saturation_drops += 1;
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
        top_snis.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
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
        // AC-144-003: surface `handshake_reassembly_overflows` in the detail map.
        // Mirrors `truncated_records` above (BC-2.07.039 Postcondition 7).
        detail.insert(
            "handshake_reassembly_overflows".to_string(),
            serde_json::json!(self.handshake_reassembly_overflows),
        );
        // AC-146-005: surface `buffer_saturation_drops` in the detail map.
        // Key ALWAYS present even when count==0 (EC-008 / BC-2.07.043 Postcondition 4).
        // Mirrors `truncated_records` and `handshake_reassembly_overflows` pattern above.
        detail.insert(
            "buffer_saturation_drops".to_string(),
            serde_json::json!(self.buffer_saturation_drops),
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

    /// Test-only accessor: number of active per-flow state entries.
    ///
    /// Exposes `self.flows.len()` so integration tests can verify that
    /// `TlsAnalyzer::on_flow_close` removes the per-flow state (BC-2.05.009
    /// analyzer-forward side effect / STORY-033 AC-007). A flow that has been
    /// closed must no longer appear in the `flows` map.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn active_flows_len_for_testing(&self) -> usize {
        self.flows.len()
    }

    /// Test-only accessor: byte length of `client_buf` for the given flow.
    ///
    /// Exposes the post-parse drain observable so tests can assert that
    /// `try_parse_records` drains consumed record bytes from `client_buf`
    /// (BC-2.07.001 postcondition 8 / STORY-052 AC-005). Returns 0 if the
    /// flow is not yet in the `flows` map (never received data) or after the
    /// buf has been fully drained.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn client_buf_len_for_testing(&self, flow_key: &FlowKey) -> usize {
        self.flows
            .get(flow_key)
            .map(|s| s.client_buf.len())
            .unwrap_or(0)
    }

    /// Test-only accessor: byte length of `server_buf` for the given flow.
    ///
    /// Symmetric companion to `client_buf_len_for_testing` for the
    /// `ServerToClient` direction. Exposes the post-parse drain observable
    /// so tests can assert that `try_parse_records` drains consumed record
    /// bytes from `server_buf` (CR-010 guard-before-allocate coverage).
    /// Returns 0 if the flow is absent or after the buf has been fully drained.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn server_buf_len_for_testing(&self, flow_key: &FlowKey) -> usize {
        self.flows
            .get(flow_key)
            .map(|s| s.server_buf.len())
            .unwrap_or(0)
    }

    /// Test-only accessor: whether `server_hello_seen` is set for the given flow.
    ///
    /// Exposes `flow.server_hello_seen` so tests can directly verify
    /// BC-2.07.002 postcondition 1 ("flow.server_hello_seen is set to true")
    /// without having to rely on the done()-short-circuit proxy
    /// (STORY-053 AC-001). Returns `false` for absent flows (flow not yet
    /// in the `flows` map) — callers should assert the flow exists via
    /// `active_flows_len_for_testing` before relying on `false` as proof
    /// of a cleared flag.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn server_hello_seen_for_testing(&self, flow_key: &FlowKey) -> bool {
        self.flows
            .get(flow_key)
            .map(|s| s.server_hello_seen)
            .unwrap_or(false)
    }

    /// Test-only accessor: the most-recently-stored capture timestamp for the
    /// given flow.
    ///
    /// Exposes `flow_state.last_ts` so tests can assert that the dispatcher
    /// threads the correct `timestamp` argument through to the TLS analyzer
    /// (STORY-097 AC-004 / BC-2.04.055 dispatcher-forwarding invariant). Returns
    /// `None` when the flow has no live state (never received data or already
    /// closed).
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn last_ts_for_testing(&self, flow_key: &FlowKey) -> Option<u32> {
        self.flows.get(flow_key).map(|s| s.last_ts)
    }

    // ── STORY-144 test seams (AC-144-001) ─────────────────────────────────────
    //
    // These seams expose the new STORY-144 carry-buffer fields and aggregate
    // counter so that the VP-039 Red-Gate test suite can verify carry-buffer
    // state directly.  All follow the `#[doc(hidden)] pub fn` convention
    // established by the existing seams above.  MUST NOT be called from
    // production code.

    /// Test-only accessor: whether `client_hello_seen` is set for the given flow.
    ///
    /// Symmetric companion to the EXISTING `server_hello_seen_for_testing`
    /// (tls.rs:991). Exposes `flow.client_hello_seen` so tests can directly
    /// verify BC-2.07.001 postcondition 1 ("flow.client_hello_seen is set to
    /// true") for the carry-reassembly path (AC-144-001 / STORY-144).
    /// Returns `false` for absent flows.
    /// MUST NOT be called from production code.
    ///
    /// Self-check (BC-5.38.005 invariant 1):
    /// "If I include this real implementation, will the test for this function
    /// pass trivially without any implementer work?"
    /// — No: the test for this seam (`test_BC_2_07_038_canonical_frame_rfc8446_s4`
    /// and others) asserts `client_hello_seen == true` only AFTER a fragmented
    /// ClientHello is reassembled by the carry drain loop. The drain loop is not
    /// yet implemented, so `client_hello_seen` is never set via the carry path;
    /// the Red Gate holds.
    #[doc(hidden)]
    pub fn client_hello_seen_for_testing(&self, flow_key: &FlowKey) -> bool {
        self.flows
            .get(flow_key)
            .map(|s| s.client_hello_seen)
            .unwrap_or(false)
    }

    /// Test-only accessor: byte length of `client_hs_carry` for the given flow.
    ///
    /// Exposes the carry-buffer length so tests can assert that the handshake
    /// carry accumulates and drains correctly (AC-144-001 / BC-2.07.038 PC-6).
    /// Returns 0 if the flow is absent or if the carry is empty.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn client_hs_carry_len_for_testing(&self, flow_key: &FlowKey) -> usize {
        self.flows
            .get(flow_key)
            .map(|s| s.client_hs_carry.len())
            .unwrap_or(0)
    }

    /// Test-only accessor: byte length of `server_hs_carry` for the given flow.
    ///
    /// Symmetric companion to `client_hs_carry_len_for_testing` for the
    /// ServerToClient direction (AC-144-001 / BC-2.07.038 PC-6).
    /// Returns 0 if the flow is absent or if the carry is empty.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn server_hs_carry_len_for_testing(&self, flow_key: &FlowKey) -> usize {
        self.flows
            .get(flow_key)
            .map(|s| s.server_hs_carry.len())
            .unwrap_or(0)
    }

    /// Public accessor: aggregate count of handshake carry overflow events.
    ///
    /// Reads `self.handshake_reassembly_overflows` directly. Mirrors the existing
    /// `truncated_record_count()` public accessor pattern (AC-144-001 / ADR-011
    /// Decision 1). The counter is incremented by the carry drain loop whenever
    /// a record payload would push the carry buffer past MAX_BUF (Step 1 overflow
    /// guard) or when a body_len > MAX_BUF header is detected (Decision-4 spoof
    /// guard). Read-only; do not use for overflow-prevention decisions.
    pub fn handshake_reassembly_overflow_count(&self) -> u64 {
        self.handshake_reassembly_overflows
    }

    // ── STORY-146 accessor + test seam (AC-146-001) ───────────────────────────
    //
    // During the RED gate, `buffer_saturation_drop_count` and `fill_buf_for_testing`
    // carried `todo!()` bodies to enforce test failures before implementation.
    // Both are now fully implemented: `buffer_saturation_drop_count` reads
    // `self.buffer_saturation_drops` directly (mirroring the
    // `truncated_record_count` / `handshake_reassembly_overflow_count` pattern),
    // and `fill_buf_for_testing` fills the per-direction TCP-segment buffer to an
    // exact byte count so tests can drive the full-drop path without live traffic.

    /// Public accessor: aggregate count of per-direction buffer saturation tail-drop events.
    ///
    /// Reads `self.buffer_saturation_drops` directly. Mirrors the existing
    /// `truncated_record_count()` and `handshake_reassembly_overflow_count()` public
    /// accessor pattern (AC-146-001 / BC-2.07.043 Invariants 1–3). Read-only; do not
    /// use for drop-prevention decisions.
    #[doc(hidden)]
    pub fn buffer_saturation_drop_count(&self) -> u64 {
        self.buffer_saturation_drops
    }

    /// Test-only seam: fill the per-direction TCP-segment buffer to exactly `n` bytes.
    ///
    /// Fills `client_buf` (for `Direction::ClientToServer`) or `server_buf`
    /// (for `Direction::ServerToClient`) of the given flow to exactly `n` bytes,
    /// creating the flow state entry if it does not yet exist.
    ///
    /// Precondition: `n <= MAX_BUF`. Required to exercise the full-drop path
    /// (`remaining==0`, EC-002) since that state is not reachable via the public
    /// `on_data` API alone without first filling the buffer with real TLS data.
    ///
    /// Signature uses `flow_key: &FlowKey` by reference, matching the convention
    /// of all five sibling TLS test seams. AC-146-001 / BC-2.07.043 Architecture Anchor.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn fill_buf_for_testing(&mut self, flow_key: &FlowKey, direction: Direction, n: usize) {
        debug_assert!(
            n <= MAX_BUF,
            "fill_buf_for_testing: n={n} exceeds MAX_BUF={MAX_BUF}"
        );
        let state = self
            .flows
            .entry(flow_key.clone())
            .or_insert_with(TlsFlowState::new);
        let buf = match direction {
            Direction::ClientToServer => &mut state.client_buf,
            Direction::ServerToClient => &mut state.server_buf,
        };
        buf.clear();
        buf.resize(n, 0u8);
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

// ── VP-005: SNI 4-Way Ordered Classification ──────────────────────────────────
//
// Faithful re-statement of the 4-way match in `extract_sni` (this file,
// lines 251–265). Because `extract_sni` takes a parsed `&[TlsExtension]` that
// Kani cannot symbolically synthesize (tls-parser borrows), the harness lifts
// the classification arms onto a raw hostname byte slice. The arm guards and
// their TOP-DOWN ORDER mirror production EXACTLY, and the model reuses the very
// same `contains_c0_or_del` helper that production calls, so there is no
// divergence between this oracle and the shipped logic. Codes are 0-based and
// follow the SniValue discriminant order (see ARM-NUMBERING LEGEND in the VP).
//
//   code 0 = Ascii            (prose arm 1, no finding)
//   code 1 = AsciiWithControl (prose arm 2, T1027)
//   code 2 = NonAsciiUtf8     (prose arm 3, T1027)
//   code 3 = NonUtf8          (prose arm 4, T1027)
#[cfg(any(kani, test))]
fn classify_hostname_vp005(hostname: &[u8]) -> u8 {
    // EXACT mirror of extract_sni's arm ordering and guards. `contains_c0_or_del`
    // is the production helper (debug_asserts is_ascii — only reached under the
    // is_ascii guard, identical to production).
    match std::str::from_utf8(hostname) {
        Ok(s) if s.is_ascii() && !contains_c0_or_del(s) => 0,
        Ok(s) if s.is_ascii() => 1,
        Ok(_) => 2,
        Err(_) => 3,
    }
}

// TOOLING NOTE (Kani 0.67.0 + CBMC): feeding SYMBOLIC bytes into
// `std::str::from_utf8` is intractable here — `core::str::run_utf8_validation`
// becomes a data-dependent loop CBMC cannot bound, and harnesses with even one
// symbolic byte through `from_utf8` either time out (>100 s) or report
// "CBMC failed". The same input shapes verify in milliseconds when the bytes
// are CONCRETE (CBMC constant-folds `from_utf8`). The VP-005 harnesses below are
// therefore structured around this reality WITHOUT weakening any assertion:
//
//  (A) The symbolic totality + arm-correctness proof runs against an EXPLICIT
//      single-byte arm model `single_byte_arm` (no `from_utf8`), over all 256
//      byte values.
//  (B) `verify_single_byte_model_matches_production` ANCHORS that explicit model
//      to the real `from_utf8`-based `classify_hostname_vp005` by a CONCRETE
//      exhaustive sweep of all 256 byte values (each iteration constant-folds),
//      proving the model and production agree on every single-byte input.
//  (C) Concrete multi-byte proofs cover the NonAsciiUtf8 (code 2) arm and the
//      BC-2.07.037 non-ASCII+control priority case over the real production
//      function. Control bytes are enumerated with a CONCRETE loop (not a
//      symbolic byte) so `from_utf8` stays constant-folded yet coverage is
//      exhaustive over the C0/DEL set.
//
// (A)+(B) together are equivalent to running the symbolic proof directly against
// production, but tractable. (C) covers the arms a single byte cannot form.
#[cfg(kani)]
mod kani_proofs_vp005 {
    use super::*;

    /// Explicit single-byte arm model, derived from `extract_sni`'s semantics for
    /// a 1-byte hostname (no `std::str::from_utf8` call, so it is symbolically
    /// tractable). A lone byte `b`:
    ///   - `b >= 0x80`            => invalid UTF-8                  => code 3 (NonUtf8)
    ///   - `b < 0x20 || b == 0x7f`=> valid ASCII control           => code 1 (AsciiWithControl)
    ///   - otherwise (0x20..=0x7e)=> clean printable ASCII         => code 0 (Ascii)
    /// (code 2 / NonAsciiUtf8 is unreachable for a single byte — a non-ASCII
    /// codepoint needs >= 2 bytes — which proof (C) covers concretely.)
    /// Anchored to production by `verify_single_byte_model_matches_production`.
    fn single_byte_arm(b: u8) -> u8 {
        if b >= 0x80 {
            3
        } else if b < 0x20 || b == 0x7f {
            1
        } else {
            0
        }
    }

    /// (A) Core VP-005 property over a FULLY-SYMBOLIC byte (all 256 values),
    /// against the explicit `single_byte_arm` model: classification is TOTAL
    /// (result in {0,1,3}) and the ASCII / C0 / DEL boundary is exactly right —
    /// the precise subtlety VP-005 is about (clean ASCII vs control, incl. the
    /// 0x1F/0x20/0x7f edges) — for every possible byte.
    ///
    /// BOUND/SOUNDNESS: one symbolic `u8` is the entire single-byte input space
    /// (256 values, exhaustive). The explicit model is sound for production by
    /// the concrete anchor proof (B). The proof terminates in milliseconds.
    #[kani::proof]
    fn verify_sni_exactly_one_arm_fires_kani() {
        let b: u8 = kani::any();
        let arm = single_byte_arm(b);

        // Totality: always a valid arm code.
        assert!(arm <= 3);

        // Exact boundary map — covers 0x1F (->1), 0x20 (->0), 0x7e (->0),
        // 0x7f (->1), and >=0x80 (->3) for every byte value at once.
        if b >= 0x80 {
            assert!(arm == 3);
        } else if b < 0x20 || b == 0x7f {
            assert!(arm == 1);
        } else {
            assert!(arm == 0);
        }
        // AsciiWithControl (code 1) is reachable only for valid ASCII (< 0x80).
        if arm == 1 {
            assert!(b < 0x80);
        }
    }

    /// (B) Anchor: the explicit `single_byte_arm` model agrees with the REAL
    /// `from_utf8`-based production classifier on EVERY single-byte input. This
    /// is what lets proof (A) stand in for the production logic.
    ///
    /// BOUND/SOUNDNESS: the loop is a CONCRETE sweep over all 256 byte values
    /// (`0u8..=255`), so each `classify_hostname_vp005([b])` call has a concrete
    /// argument and CBMC constant-folds `from_utf8` — exhaustive over the full
    /// single-byte domain, and fast. `#[kani::unwind(257)]` fully unrolls the
    /// 256-iteration loop.
    #[kani::proof]
    #[kani::unwind(257)]
    fn verify_single_byte_model_matches_production() {
        for b in 0u8..=255 {
            assert!(single_byte_arm(b) == classify_hostname_vp005(&[b]));
        }
    }

    /// (C1) Arm code 2 (NonAsciiUtf8): a valid 2-byte non-ASCII codepoint with NO
    /// control byte classifies as NonAsciiUtf8 (code 2), against the REAL
    /// production function. Covers the "plain non-ASCII" half of arm 3.
    ///
    /// BOUND/SOUNDNESS: 0xC2 0xA0 = U+00A0, the smallest non-ASCII codepoint; the
    /// property does not depend on which non-ASCII codepoint, only that one is
    /// present (`is_ascii()` false). Fully concrete => `from_utf8` constant-folds.
    #[kani::proof]
    fn verify_nonascii_utf8_yields_code2() {
        let hostname: [u8; 2] = [0xC2, 0xA0]; // U+00A0, valid non-ASCII UTF-8
        assert!(classify_hostname_vp005(&hostname) == 2);
    }

    /// (C1b) Arm code 2 over a 4-byte non-ASCII codepoint (CR-001): closes the
    /// formal gap where no Kani harness exercised a 4-byte UTF-8 sequence. A
    /// valid 4-byte codepoint is non-ASCII, so it must classify as NonAsciiUtf8
    /// (code 2).
    ///
    /// BOUND/SOUNDNESS: 0xF0 0x9F 0x98 0x80 = U+1F600 (GRINNING FACE), a valid
    /// 4-byte UTF-8 codepoint and the maximum encoded length. Fully concrete =>
    /// `from_utf8` constant-folds; proof is instant.
    #[kani::proof]
    fn verify_4byte_utf8_yields_code2() {
        let hostname: [u8; 4] = [0xF0, 0x9F, 0x98, 0x80]; // U+1F600, valid 4-byte UTF-8
        assert!(classify_hostname_vp005(&hostname) == 2);
    }

    /// (C2) BC-2.07.037 / INV-5 arm-3-priority boundary, against the REAL
    /// production function: a valid non-ASCII codepoint FOLLOWED BY a C0/DEL
    /// control byte still classifies as NonAsciiUtf8 (code 2), NOT
    /// AsciiWithControl (code 1) — `is_ascii()` is false once a multi-byte
    /// codepoint is present, so arm 2's guard never fires.
    ///
    /// BOUND/SOUNDNESS: the trailing control byte is enumerated by a CONCRETE
    /// loop over the ENTIRE C0/DEL set (0x00..=0x1F and 0x7F) — every control
    /// value that could erroneously trip arm 2 is checked — while keeping each
    /// `classify_hostname_vp005` argument concrete so `from_utf8` constant-folds.
    /// `#[kani::unwind(34)]` covers the <= 33-iteration loop.
    #[kani::proof]
    #[kani::unwind(34)]
    fn verify_arm3_priority_nonascii_plus_control() {
        // C0 controls 0x00..=0x1F.
        for ctrl in 0x00u8..=0x1F {
            let hostname: [u8; 3] = [0xC2, 0xA0, ctrl]; // U+00A0 then a C0 byte
            assert!(classify_hostname_vp005(&hostname) == 2);
        }
        // DEL 0x7F.
        let hostname: [u8; 3] = [0xC2, 0xA0, 0x7F];
        assert!(classify_hostname_vp005(&hostname) == 2);
    }

    /// (C3) C0 boundary: 0x1F (last C0 control byte) trips AsciiWithControl
    /// (code 1) in the REAL production function.
    /// BOUND/SOUNDNESS: single concrete boundary byte.
    #[kani::proof]
    fn verify_c0_boundary_0x1f_triggers_ascii_with_control_code1() {
        assert!(classify_hostname_vp005(&[0x1Fu8]) == 1);
    }

    /// (C4) 0x20 (space) is the first printable ASCII byte; yields Ascii (code 0)
    /// in the REAL production function.
    /// BOUND/SOUNDNESS: single concrete boundary byte (other side of the edge).
    #[kani::proof]
    fn verify_0x20_space_yields_arm0() {
        assert!(classify_hostname_vp005(&[0x20u8]) == 0);
    }

    /// (C5) 0x7f (DEL) — the lone non-C0 control byte the scan catches — trips
    /// AsciiWithControl (code 1), distinguishing the `b == 0x7f` clause from the
    /// `b < 0x20` clause, in the REAL production function.
    /// BOUND/SOUNDNESS: single concrete boundary byte.
    #[kani::proof]
    fn verify_del_0x7f_triggers_ascii_with_control_code1() {
        assert!(classify_hostname_vp005(&[0x7fu8]) == 1);
    }
}

#[cfg(test)]
mod proptest_proofs_vp005 {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Supplemental unbounded check of the same INV-5 invariants the Kani
        /// proofs cover. Kani coverage is: a symbolic SINGLE byte (totality +
        /// the full ASCII/C0/DEL boundary, via the explicit model anchored to
        /// production over all 256 bytes) plus CONCRETE 2-, 3-, and 4-byte
        /// codepoint cases (NonAsciiUtf8 arm and the BC-2.07.037 non-ASCII+
        /// control priority case). No Kani harness takes a symbolic multi-byte
        /// input (symbolic `from_utf8` is intractable under CBMC — see the
        /// TOOLING NOTE above). This proptest fills that gap: it exercises
        /// arbitrary-LENGTH `Vec<u8>` — including 3- and 4-byte codepoints and
        /// mixed sequences — over the real `from_utf8`-based classifier.
        #[test]
        fn prop_sni_arm3_priority_and_arm1_ascii_only(hostname: Vec<u8>) {
            let arm = classify_hostname_vp005(&hostname);
            prop_assert!(arm <= 3);
            match std::str::from_utf8(&hostname) {
                Ok(s) if !s.is_ascii() => prop_assert_eq!(arm, 2), // NonAsciiUtf8 only
                Ok(_) => prop_assert!(arm == 0 || arm == 1),       // some ASCII arm
                Err(_) => prop_assert_eq!(arm, 3),                 // NonUtf8 only
            }
            // AsciiWithControl (code 1) fires only for all-ASCII inputs.
            if arm == 1 {
                prop_assert!(std::str::from_utf8(&hostname).map(|s| s.is_ascii()).unwrap_or(false));
            }
        }
    }
}
