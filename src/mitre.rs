//! MITRE ATT&CK technique-ID → name / tactic lookup module.
//!
//! Backed by a single exhaustive `match` statement (see [`technique_info`]);
//! zero runtime dependencies. See the design spec under
//! `docs/superpowers/specs/` for the full rationale.
//!
//! ## ID format
//!
//! Callers pass technique IDs in MITRE's canonical form: `TXXXX` for parent
//! techniques (e.g., `T1046`) and `TXXXX.NNN` for sub-techniques (period
//! separator, three-digit suffix — e.g., `T1071.001`). This format is used
//! across ATT&CK matrices and STIX 2.1 bundles. Inputs that don't match a
//! seeded ID return `None` from the lookup functions.
//!
//! ## Catalogued vs. emitted techniques (staged entries)
//!
//! [`technique_info`] is a *catalogue*: it seeds every technique ID that
//! wirerust may attach to a [`crate::findings::Finding`]. Not every
//! catalogued ID is currently produced by an analyzer.
//!
//! Some entries are **staged** — present in the lookup table ahead of the
//! detection logic that will emit them. This is intentional:
//!
//! - The catalogue is the single place an ID's name and tactic are defined.
//!   Seeding an ID here first means the analyzer PR that starts emitting it
//!   only has to set `mitre_technique: Some("TXXXX")` — it does not also
//!   have to touch this module, keeping that change small and focused.
//! - The ICS techniques (`T0xxx`) in particular are seeded for the planned
//!   Modbus / DNP3 analyzers (see the README roadmap) but are not emitted
//!   until those analyzers land.
//!
//! A staged entry is therefore not dead code — it is a deliberate forward
//! declaration. The set of *emitted* IDs is whatever the analyzers in
//! `src/analyzer/` and `src/reassembly/` currently pass as
//! `mitre_technique`; `grep -rn 'mitre_technique: Some' src/` is the
//! authoritative way to see it. No invariant requires the catalogue and the
//! emitted set to match — the catalogue is intentionally the superset.

use std::fmt;

// MITRE ATT&CK is an evolving external standard — new tactics are added in
// new ATT&CK versions (e.g., v18 added Resource Development). Mark the enum
// `#[non_exhaustive]` so adding a variant later is non-breaking for any
// downstream crate that matches on `MitreTactic`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MitreTactic {
    // Enterprise canonical kill-chain order.
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

/// Returns all tactics in canonical kill-chain order, with ICS-unique
/// tactics appended last. Intended as a stable iteration order for any
/// consumer that needs to present findings grouped by tactic.
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

/// Resolves a technique ID to its `(name, tactic)` pair. The single source
/// of truth for every seeded technique — [`technique_name`] and
/// [`technique_tactic`] are thin projections over this function, which
/// makes it impossible to add one facet without the other.
///
/// Returns `None` for IDs not in the seeded set.
pub fn technique_info(id: &str) -> Option<(&'static str, MitreTactic)> {
    let info = match id {
        // Enterprise.
        "T1027" => (
            "Obfuscated Files or Information",
            MitreTactic::DefenseEvasion,
        ),
        "T1036" => ("Masquerading", MitreTactic::DefenseEvasion),
        "T1040" => ("Network Sniffing", MitreTactic::CredentialAccess),
        "T1046" => ("Network Service Discovery", MitreTactic::Discovery),
        "T1071" => ("Application Layer Protocol", MitreTactic::CommandAndControl),
        "T1071.001" => ("Web Protocols", MitreTactic::CommandAndControl),
        "T1071.004" => ("DNS", MitreTactic::CommandAndControl),
        "T1083" => ("File and Directory Discovery", MitreTactic::Discovery),
        "T1499.002" => ("Service Exhaustion Flood", MitreTactic::Impact),
        "T1505.003" => ("Web Shell", MitreTactic::Persistence),
        "T1573" => ("Encrypted Channel", MitreTactic::CommandAndControl),
        // ICS. MITRE assigns distinct TA-IDs per matrix (e.g., Enterprise
        // Discovery TA0007 vs ICS Discovery TA0111); we intentionally
        // merge by name so a single grouped report has one section per
        // tactic name regardless of source matrix.
        "T0846" => ("Remote System Discovery", MitreTactic::Discovery),
        "T0855" => (
            "Unauthorized Command Message",
            MitreTactic::IcsImpairProcessControl,
        ),
        "T0856" => (
            "Spoof Reporting Message",
            MitreTactic::IcsImpairProcessControl,
        ),
        "T0885" => ("Commonly Used Port", MitreTactic::CommandAndControl),
        _ => return None,
    };
    Some(info)
}

/// Resolves a technique ID to its human-readable name. Returns `None` for
/// unknown IDs.
pub fn technique_name(id: &str) -> Option<&'static str> {
    technique_info(id).map(|(name, _)| name)
}

/// Resolves a technique ID to its parent tactic. Returns `None` for
/// unknown IDs.
pub fn technique_tactic(id: &str) -> Option<MitreTactic> {
    technique_info(id).map(|(_, tactic)| tactic)
}
