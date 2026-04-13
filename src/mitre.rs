//! MITRE ATT&CK technique-ID → name / tactic lookup module.
//!
//! Backed by exhaustive `match` statements; zero runtime dependencies.
//! See `docs/superpowers/specs/2026-04-13-mitre-attack-mapping-design.md`
//! for the full design rationale.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MitreTactic {
    // Enterprise canonical order (MITRE ATT&CK v18, 14 tactics).
    Reconnaissance,
    ResourceDevelopment,
    InitialAccess,
    Execution,
    Persistence,
    PrivilegeEscalation,
    DefenseEvasion,
    CredentialAccess,
    Discovery,
    LateralMovement,
    Collection,
    CommandAndControl,
    Exfiltration,
    Impact,
    // ICS-unique tactics (names that don't collide with Enterprise).
    IcsInhibitResponseFunction,
    IcsImpairProcessControl,
}

impl fmt::Display for MitreTactic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MitreTactic::Reconnaissance => "Reconnaissance",
            MitreTactic::ResourceDevelopment => "Resource Development",
            MitreTactic::InitialAccess => "Initial Access",
            MitreTactic::Execution => "Execution",
            MitreTactic::Persistence => "Persistence",
            MitreTactic::PrivilegeEscalation => "Privilege Escalation",
            MitreTactic::DefenseEvasion => "Defense Evasion",
            MitreTactic::CredentialAccess => "Credential Access",
            MitreTactic::Discovery => "Discovery",
            MitreTactic::LateralMovement => "Lateral Movement",
            MitreTactic::Collection => "Collection",
            MitreTactic::CommandAndControl => "Command and Control",
            MitreTactic::Exfiltration => "Exfiltration",
            MitreTactic::Impact => "Impact",
            MitreTactic::IcsInhibitResponseFunction => "Inhibit Response Function",
            MitreTactic::IcsImpairProcessControl => "Impair Process Control",
        };
        f.write_str(name)
    }
}

/// Returns all tactics in MITRE canonical kill-chain order, with ICS-unique
/// tactics appended last. Used by the terminal reporter to produce a stable
/// section order when grouping findings by tactic.
pub fn all_tactics_in_report_order() -> &'static [MitreTactic] {
    &[
        MitreTactic::Reconnaissance,
        MitreTactic::ResourceDevelopment,
        MitreTactic::InitialAccess,
        MitreTactic::Execution,
        MitreTactic::Persistence,
        MitreTactic::PrivilegeEscalation,
        MitreTactic::DefenseEvasion,
        MitreTactic::CredentialAccess,
        MitreTactic::Discovery,
        MitreTactic::LateralMovement,
        MitreTactic::Collection,
        MitreTactic::CommandAndControl,
        MitreTactic::Exfiltration,
        MitreTactic::Impact,
        MitreTactic::IcsInhibitResponseFunction,
        MitreTactic::IcsImpairProcessControl,
    ]
}

/// Resolves a MITRE ATT&CK technique ID to its human-readable name.
///
/// Returns `None` for unknown IDs; callers that treat unknowns as
/// programming errors should `debug_assert!` at their call site.
/// The canonical ID format is `TXXXX` for parent techniques and
/// `TXXXX.NNN` for sub-techniques (period separator, three-digit
/// suffix), used consistently across Enterprise, ICS, and Mobile
/// matrices and in STIX 2.1 bundles.
pub fn technique_name(id: &str) -> Option<&'static str> {
    let name = match id {
        // Enterprise.
        "T1027" => "Obfuscated Files or Information",
        "T1036" => "Masquerading",
        "T1040" => "Network Sniffing",
        "T1046" => "Network Service Discovery",
        "T1071" => "Application Layer Protocol",
        "T1071.001" => "Web Protocols",
        "T1071.004" => "DNS",
        "T1083" => "File and Directory Discovery",
        "T1499.002" => "Service Exhaustion Flood",
        "T1505.003" => "Web Shell",
        "T1573" => "Encrypted Channel",
        // ICS.
        "T0846" => "Remote System Discovery",
        "T0855" => "Unauthorized Command Message",
        "T0856" => "Spoof Reporting Message",
        "T0885" => "Commonly Used Port",
        _ => return None,
    };
    Some(name)
}

/// Resolves a MITRE ATT&CK technique ID to its parent tactic.
///
/// For IDs shared in name between Enterprise and ICS (Discovery,
/// Command and Control, etc.) this returns the unified variant — see
/// the spec for the v1 limitation rationale.
pub fn technique_tactic(id: &str) -> Option<MitreTactic> {
    let tactic = match id {
        // Enterprise.
        "T1027" | "T1036" => MitreTactic::DefenseEvasion,
        "T1040" => MitreTactic::CredentialAccess,
        "T1046" | "T1083" => MitreTactic::Discovery,
        "T1071" | "T1071.001" | "T1071.004" | "T1573" => MitreTactic::CommandAndControl,
        "T1499.002" => MitreTactic::Impact,
        "T1505.003" => MitreTactic::Persistence,
        // ICS.
        "T0846" => MitreTactic::Discovery,
        "T0855" | "T0856" => MitreTactic::IcsImpairProcessControl,
        "T0885" => MitreTactic::CommandAndControl,
        _ => return None,
    };
    Some(tactic)
}
