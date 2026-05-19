//! Universal anomaly-finding type — the lingua franca of the pipeline.
//!
//! Every analyzer ([`crate::analyzer::dns`], [`crate::analyzer::http`],
//! [`crate::analyzer::tls`], [`crate::reassembly`]) emits findings as
//! [`Finding`]s; every reporter ([`crate::reporter::terminal`],
//! [`crate::reporter::json`]) consumes the same type. The three classifying
//! enums ([`ThreatCategory`], [`Verdict`], [`Confidence`]) keep the
//! cross-analyzer vocabulary bounded.
//!
//! See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`) for the
//! raw-data-vs-display-layer escaping contract that `Finding::Display`
//! intentionally does NOT enforce — the terminal reporter handles it
//! at render time so the same `Finding` value can flow safely to JSON,
//! logs, or a TTY without double-escaping.

use std::fmt;
use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Confidence that a [`Finding`] reflects a real threat.
///
/// LESSON-P2.10: marked `#[non_exhaustive]` so downstream consumers
/// must include a wildcard arm when matching on this enum. Lets us
/// add new verdicts (e.g. `Suspected`) in the future without breaking
/// SemVer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum Verdict {
    /// The finding is likely a real threat / anomaly.
    Likely,
    /// The finding is most likely benign noise; emitted for transparency.
    Unlikely,
    /// The finding cannot be classified as Likely or Unlikely from the
    /// available evidence.
    Inconclusive,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Likely => write!(f, "LIKELY"),
            Verdict::Unlikely => write!(f, "UNLIKELY"),
            Verdict::Inconclusive => write!(f, "INCONCLUSIVE"),
        }
    }
}

/// Confidence band assigned by the analyzer to its own finding.
///
/// LESSON-P2.10: `#[non_exhaustive]` so a future "VeryHigh" or
/// "Negligible" tier can be added without breaking downstream
/// pattern matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum Confidence {
    /// Strong signal — false-positive rate is expected to be low.
    High,
    /// Moderate signal — useful for triage but warrants human review.
    Medium,
    /// Weak signal — often noisy, included for completeness only.
    Low,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::High => write!(f, "HIGH"),
            Confidence::Medium => write!(f, "MEDIUM"),
            Confidence::Low => write!(f, "LOW"),
        }
    }
}

/// Top-level threat taxonomy assigned to each [`Finding`].
///
/// Roughly mirrors a subset of MITRE ATT&CK tactics (with `Anomaly`
/// covering unclassified / pre-classification signals) plus `C2`
/// (Command-and-Control) split out from the ATT&CK tactic of the
/// same name for analyst convenience.
///
/// LESSON-P2.10: `#[non_exhaustive]` so the enum can grow as new
/// detection families are added (Impact, Collection, Discovery,
/// etc.) without breaking SemVer on downstream pattern matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum ThreatCategory {
    /// Active or passive information gathering (port scans, host
    /// enumeration, banner grabbing).
    Reconnaissance,
    /// Movement between hosts after initial access.
    LateralMovement,
    /// Command-and-control communication patterns (beaconing,
    /// covert channels, suspicious DNS / TLS fingerprints).
    C2,
    /// Data being staged or moved off-host.
    Exfiltration,
    /// Credential theft / brute-force / spraying patterns.
    CredentialAccess,
    /// Suspicious code execution patterns (upload paths, traversal,
    /// malformed methods).
    Persistence,
    /// Execution-class signals (long URIs, unusual methods).
    Execution,
    /// Unclassified protocol-level anomalies (RFC violations, evasion
    /// indicators) that don't yet warrant a specific tactic tag.
    Anomaly,
}

impl fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub category: ThreatCategory,
    pub verdict: Verdict,
    pub confidence: Confidence,
    pub summary: String,
    pub evidence: Vec<String>,
    // LESSON-P1.02 / NFR OBS-010: all three `Option<_>` fields are
    // emitted symmetrically — absent values are omitted from the JSON
    // object rather than serialized as `null`. Previously only
    // `timestamp` carried this attribute, producing an asymmetric
    // mixed-shape output (`mitre_technique: null` vs no `timestamp`
    // key at all) that made JSON consumers harder to write.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitre_technique: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ip: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Produces the raw text representation of a finding for logging, debugging,
/// or machine-readable output. **Not safe for direct terminal display** — the
/// `summary` field may contain attacker-controlled bytes from packet payloads
/// (including ASCII control codes like ESC `0x1b`) that a terminal would
/// interpret as control sequences. For safe terminal rendering, use the
/// terminal reporter (`src/reporter/terminal.rs`), which escapes every
/// `summary` and `evidence` entry before writing to the output buffer.
/// See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`) for the
/// full layering principle.
impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{cat}] {verdict} ({conf}) — {summary}",
            cat = self.category,
            verdict = self.verdict,
            conf = self.confidence,
            summary = self.summary,
        )
    }
}
