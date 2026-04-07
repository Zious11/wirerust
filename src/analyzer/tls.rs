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

// ── helpers ──────────────────────────────────────────────────────────────────

/// Returns true if `val` is a TLS GREASE value (RFC 8701).
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
            if is_grease_u16(v) { None } else { Some(v.to_string()) }
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
            if is_grease_u16(v) { None } else { Some(v.to_string()) }
        })
        .collect::<Vec<_>>()
        .join("-");

    let ja3s_str = format!("{},{},{}", version, cipher.0, ext_ids);
    bytes_to_hex(Md5::digest(ja3s_str.as_bytes()).as_slice())
}

/// Extract SNI hostname from the parsed extension list.
fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<String> {
    for ext in extensions {
        if let TlsExtension::SNI(list) = ext
            && let Some((_, hostname)) = list.first()
        {
            return String::from_utf8(hostname.to_vec()).ok();
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
            Self::increment(&mut self.sni_counts, sni, MAX_MAP_ENTRIES);
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
                summary: "ClientHello offers weak cipher suites (NULL/anonymous/export)".to_string(),
                evidence: weak,
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
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
                summary: format!("ServerHello selected weak cipher suite ({})", name),
                evidence: vec![format!("Selected cipher: {} (0x{:04x})", name, sh.cipher.0)],
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
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
            let state = self.flows.entry(flow_key.clone()).or_insert_with(TlsFlowState::new);
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
        let mut detail = HashMap::new();

        // Top SNIs (top 20 by count)
        let mut top_snis: Vec<_> = self.sni_counts.iter().collect();
        top_snis.sort_by(|a, b| b.1.cmp(a.1));
        let top_snis: Vec<&str> = top_snis.iter().take(20).map(|(k, _)| k.as_str()).collect();
        detail.insert("top_snis".to_string(), serde_json::json!(top_snis));

        detail.insert("ja3_hashes".to_string(), serde_json::json!(self.ja3_counts));
        detail.insert("ja3s_hashes".to_string(), serde_json::json!(self.ja3s_counts));
        detail.insert(
            "tls_versions".to_string(),
            serde_json::json!(
                self.version_counts
                    .iter()
                    .map(|(k, v)| (k.to_string(), *v))
                    .collect::<HashMap<String, u64>>()
            ),
        );
        detail.insert("cipher_suites".to_string(), serde_json::json!(self.cipher_counts));
        detail.insert("parse_errors".to_string(), serde_json::json!(self.parse_errors));

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
