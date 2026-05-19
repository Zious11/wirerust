use std::fmt;
use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Likely,
    Unlikely,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Confidence {
    High,
    Medium,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ThreatCategory {
    Reconnaissance,
    LateralMovement,
    C2,
    Exfiltration,
    CredentialAccess,
    Execution,
    Persistence,
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
