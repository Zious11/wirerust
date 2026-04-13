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
