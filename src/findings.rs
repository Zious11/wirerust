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
    pub mitre_technique: Option<String>,
    pub source_ip: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

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
