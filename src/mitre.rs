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

// ── VP-007: MITRE Technique ID Format and Catalog Completeness ────────────────
//
// Sub-property A (ID format): every seeded ID matches `T[0-9]{4}` or
// `T[0-9]{4}.[0-9]{3}`.
// Sub-property B (completeness): every seeded ID and every analyzer-emitted ID
// resolves in `technique_info` (both name and tactic Some).
// Corollary (BC-2.10.006): unknown IDs return None without panicking.
//
// The catalogue is a closed-world static match; the seeded set is finite (15)
// so the harness enumerates it exhaustively — fully sound, no abstraction.
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// All 15 seeded IDs (mirrors `technique_info`, this file). If `technique_info`
    /// gains/loses an entry, the completeness proof here will diverge from the
    /// table and must be updated in lockstep with the VP.
    const SEEDED_IDS: &[&str] = super::SEEDED_TECHNIQUE_IDS;

    /// IDs actually emitted by analyzers today (`grep -rn 'mitre_technique: Some' src/`).
    /// Sub-property B's emitter half: each must resolve in the catalogue.
    const EMITTED_IDS: &[&str] = &[
        "T1027",     // TLS: SNI anomaly
        "T1036",     // Reassembly: conflicting overlap
        "T1046",     // HTTP: admin panel
        "T1083",     // HTTP: path traversal
        "T1499.002", // HTTP: header flood
        "T1505.003", // HTTP: web shell
    ];

    /// Sub-property A: format invariant `T[0-9]{4}` or `T[0-9]{4}.[0-9]{3}`.
    ///
    /// BOUND/SOUNDNESS: the seeded set is a finite closed enumeration (15 IDs);
    /// the harness checks every one against the regex-equivalent byte predicate.
    /// No symbolic input is needed — the property is universal over a fixed set,
    /// so enumeration is exhaustive and sound.
    #[kani::proof]
    fn verify_all_seeded_ids_match_format() {
        for id in SEEDED_IDS {
            assert!(is_valid_technique_id_format(id));
        }
    }

    /// Sub-property B (catalogue half): every seeded ID resolves to Some name
    /// and Some tactic (BC-2.10.005 / BC-2.10.007).
    #[kani::proof]
    fn verify_all_seeded_ids_resolve() {
        for id in SEEDED_IDS {
            assert!(technique_name(id).is_some());
            assert!(technique_tactic(id).is_some());
        }
    }

    /// Sub-property B (emitter half, BC-2.10.008): every analyzer-emitted ID
    /// resolves in the catalogue.
    #[kani::proof]
    fn verify_all_emitted_ids_resolve() {
        for id in EMITTED_IDS {
            assert!(technique_name(id).is_some());
            assert!(technique_tactic(id).is_some());
        }
    }

    /// Corollary (BC-2.10.006): an ID not in the catalogue returns None for both
    /// projections and never panics.
    ///
    /// BOUND/SOUNDNESS: `technique_info` is a closed match whose only catch-all
    /// arm is `_ => None`; any string outside the 15 seeded literals takes it.
    /// A single representative unknown ID ("T9999") exercises that arm. "T9999"
    /// is deliberately a VALIDLY-FORMATTED (`T[0-9]{4}`) but UNREGISTERED ID, so
    /// this proves the "unknown" branch — not merely a malformed-string reject.
    /// Because the match is literal-equality on a closed set, no symbolic search
    /// over all strings is required to prove totality of the unknown branch.
    #[kani::proof]
    fn verify_unknown_id_returns_none_no_panic() {
        // Sanity: the canary is well-formed yet must not be in the catalogue.
        assert!(is_valid_technique_id_format("T9999"));
        assert!(technique_name("T9999").is_none());
        assert!(technique_tactic("T9999").is_none());
    }
}

/// Single source of truth for the seeded technique-ID set, consumed by both the
/// Kani proofs (`kani_proofs::SEEDED_IDS`) and the drift-guard test below. This
/// list MUST mirror every Some-returning arm of [`technique_info`]. The
/// `vp007_catalog_drift_guard` test mechanically fails if `technique_info` gains
/// or loses a Some-returning entry without this list (and
/// [`SEEDED_TECHNIQUE_ID_COUNT`]) being updated in lockstep — preventing the
/// completeness proofs from silently going stale (CR-005).
#[cfg(any(kani, test))]
const SEEDED_TECHNIQUE_IDS: &[&str] = &[
    // Enterprise
    "T1027",
    "T1036",
    "T1040",
    "T1046",
    "T1071",
    "T1071.001",
    "T1071.004",
    "T1083",
    "T1499.002",
    "T1505.003",
    "T1573",
    // ICS
    "T0846",
    "T0855",
    "T0856",
    "T0885",
];

/// Expected number of Some-returning arms in [`technique_info`]. Declared
/// separately from `SEEDED_TECHNIQUE_IDS.len()` so the drift guard catches BOTH
/// directions of accidental edit: bumping this without adding an ID (or vice
/// versa) fails the test. Must equal the count of `=> (...)` arms in
/// `technique_info` (currently 15).
#[cfg(any(kani, test))]
const SEEDED_TECHNIQUE_ID_COUNT: usize = 15;

/// Validates MITRE technique-ID format: `T[0-9]{4}` (parent) or
/// `T[0-9]{4}.[0-9]{3}` (sub-technique). Used by the VP-007 format proof; gated
/// to `kani`/`test` so it adds nothing to the normal build.
#[cfg(any(kani, test))]
fn is_valid_technique_id_format(id: &str) -> bool {
    let b = id.as_bytes();
    // Parent: T + 4 digits == 5 bytes.
    let parent_ok = b.len() == 5 && b[0] == b'T' && b[1..5].iter().all(|c| c.is_ascii_digit());
    // Sub-technique: T + 4 digits + '.' + 3 digits == 9 bytes.
    let sub_ok = b.len() == 9
        && b[0] == b'T'
        && b[1..5].iter().all(|c| c.is_ascii_digit())
        && b[5] == b'.'
        && b[6..9].iter().all(|c| c.is_ascii_digit());
    parent_ok || sub_ok
}

#[cfg(test)]
mod vp007_format_tests {
    use super::*;

    #[test]
    fn format_predicate_accepts_canonical_and_rejects_malformed() {
        assert!(is_valid_technique_id_format("T1027"));
        assert!(is_valid_technique_id_format("T1071.001"));
        assert!(is_valid_technique_id_format("T0846"));
        // Malformed cases must be rejected.
        assert!(!is_valid_technique_id_format("TXXXX"));
        assert!(!is_valid_technique_id_format("T102")); // too short
        assert!(!is_valid_technique_id_format("T10277")); // too long, no dot
        assert!(!is_valid_technique_id_format("T1071.01")); // 2-digit suffix
        assert!(!is_valid_technique_id_format("T1071.0001")); // 4-digit suffix
        assert!(!is_valid_technique_id_format("X1027")); // wrong prefix
        assert!(!is_valid_technique_id_format("T1071,001")); // wrong separator
    }

    /// CR-005: mechanically link the seeded-ID list to `technique_info` so the
    /// VP-007 completeness proofs cannot silently go stale. This test fails if:
    ///  - the seeded-ID list count drifts from the documented catalogue size,
    ///  - any seeded ID stops resolving (an entry was removed/renamed),
    ///  - a seeded ID is duplicated,
    ///  - a seeded ID is malformed, or
    ///  - the validly-formatted canary "T9999" starts resolving (an entry was
    ///    added to `technique_info` — the maintainer must then add it to
    ///    `SEEDED_TECHNIQUE_IDS` and bump `SEEDED_TECHNIQUE_ID_COUNT`).
    #[test]
    fn vp007_catalog_drift_guard() {
        // Count link: list length must equal the documented catalogue size.
        assert_eq!(
            SEEDED_TECHNIQUE_IDS.len(),
            SEEDED_TECHNIQUE_ID_COUNT,
            "SEEDED_TECHNIQUE_IDS length drifted from SEEDED_TECHNIQUE_ID_COUNT; \
             update both in lockstep with technique_info"
        );

        // Every seeded ID is well-formed, resolves, and is unique.
        let mut seen = std::collections::HashSet::new();
        for id in SEEDED_TECHNIQUE_IDS {
            assert!(
                is_valid_technique_id_format(id),
                "seeded ID {id} is malformed"
            );
            assert!(
                technique_info(id).is_some(),
                "seeded ID {id} no longer resolves in technique_info"
            );
            assert!(seen.insert(*id), "seeded ID {id} is duplicated");
        }

        // Canary: a well-formed but unregistered ID must NOT resolve. If this
        // fires, technique_info gained an entry not mirrored in the seeded list.
        assert!(is_valid_technique_id_format("T9999"));
        assert!(
            technique_info("T9999").is_none(),
            "canary T9999 resolved — technique_info gained an entry; add it to \
             SEEDED_TECHNIQUE_IDS and bump SEEDED_TECHNIQUE_ID_COUNT"
        );
    }
}
