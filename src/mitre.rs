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
//!   only has to set `mitre_techniques: vec!["TXXXX".to_string()]` — it does not also
//!   have to touch this module, keeping that change small and focused.
//! - The ICS techniques (`T0xxx`) in particular are seeded for the planned
//!   Modbus / DNP3 analyzers (see the README roadmap) but are not emitted
//!   until those analyzers land.
//!
//! A staged entry is therefore not dead code — it is a deliberate forward
//! declaration. The set of *emitted* IDs is whatever the analyzers in
//! `src/analyzer/` and `src/reassembly/` currently pass as
//! `mitre_techniques`; `grep -rn 'mitre_techniques: vec!' src/` is the
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
    /// ICS Impact tactic (TA0105) — T0827 "Loss of Control" and similar
    /// impact-category findings.  Distinct from the Enterprise `Impact` tactic.
    /// Added atomically with T0827 emission (STORY-109, VP-007 obligation).
    IcsImpact,
    /// ICS Discovery tactic (TA0102) — T0846 "Remote System Discovery" and
    /// T0888 "Remote System Information Discovery". Distinct from Enterprise
    /// Discovery (TA0007). Added in F5 to emit the authoritative ICS-matrix TA-id.
    IcsDiscovery,
    /// ICS Collection tactic (TA0100) — T0830 "Adversary-in-the-Middle".
    /// Distinct from Enterprise Collection (TA0009) and LateralMovement (TA0008).
    /// Added in F5 to emit the authoritative ICS-matrix TA-id.
    IcsCollection,
    /// ICS Command and Control tactic (TA0101) — T0885 "Commonly Used Port".
    /// Distinct from Enterprise CommandAndControl (TA0011). Added in F5 to emit
    /// the authoritative ICS-matrix TA-id.
    IcsCommandAndControl,
    /// ICS Execution tactic (TA0104) — T0858 "Change Operating Mode" and similar
    /// execution-category findings. Distinct from Enterprise Execution (TA0002).
    /// Added atomically with T0858 seeding (STORY-133, VP-007 obligation).
    IcsExecution,
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
            MitreTactic::IcsImpact => "Impact (ICS)",
            MitreTactic::IcsDiscovery => "Discovery (ICS)",
            MitreTactic::IcsCollection => "Collection (ICS)",
            MitreTactic::IcsCommandAndControl => "Command and Control (ICS)",
            MitreTactic::IcsExecution => "Execution (ICS)",
        };
        f.write_str(name)
    }
}

impl MitreTactic {
    /// Returns the canonical MITRE ATT&CK tactic TA-prefix ID string for this tactic
    /// (e.g., `IcsExecution` → `"TA0104"`). Used in tests via AC-133-004 / VP-007 Step 5.
    pub fn tactic_id(&self) -> &'static str {
        match self {
            MitreTactic::Reconnaissance => "TA0043",
            MitreTactic::ResourceDevelopment => "TA0042",
            MitreTactic::InitialAccess => "TA0001",
            MitreTactic::Execution => "TA0002",
            MitreTactic::Persistence => "TA0003",
            MitreTactic::PrivilegeEscalation => "TA0004",
            MitreTactic::DefenseEvasion => "TA0005",
            MitreTactic::CredentialAccess => "TA0006",
            MitreTactic::Discovery => "TA0007",
            MitreTactic::LateralMovement => "TA0008",
            MitreTactic::Collection => "TA0009",
            MitreTactic::CommandAndControl => "TA0011",
            MitreTactic::Exfiltration => "TA0010",
            MitreTactic::Impact => "TA0040",
            MitreTactic::IcsInhibitResponseFunction => "TA0107",
            MitreTactic::IcsImpairProcessControl => "TA0106",
            MitreTactic::IcsImpact => "TA0105",
            MitreTactic::IcsDiscovery => "TA0102",
            MitreTactic::IcsCollection => "TA0100",
            MitreTactic::IcsCommandAndControl => "TA0101",
            MitreTactic::IcsExecution => "TA0104",
        }
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
        MitreTactic::IcsImpact,
        MitreTactic::IcsDiscovery,
        MitreTactic::IcsCollection,
        MitreTactic::IcsCommandAndControl,
        MitreTactic::IcsExecution,
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
        // Discovery TA0007 vs ICS Discovery TA0102, Enterprise Collection
        // TA0009 vs ICS Collection TA0100, Enterprise C2 TA0011 vs ICS C2
        // TA0101). F5 uses dedicated ICS variants so the reporter emits the
        // authoritative ICS-matrix TA-id for each technique (f5-ics-technique-tactic-authoritative.md).
        "T0846" => ("Remote System Discovery", MitreTactic::IcsDiscovery),
        "T1692.001" => (
            "Unauthorized Message: Command Message",
            MitreTactic::IcsImpairProcessControl,
        ),
        "T1692.002" => (
            "Unauthorized Message: Reporting Message",
            MitreTactic::IcsImpairProcessControl,
        ),
        "T0885" => ("Commonly Used Port", MitreTactic::IcsCommandAndControl),
        // ICS — NEW F2 (STORY-100 / BC-2.10.005). Seeded for Modbus/DNP3 analyzers.
        "T0836" => ("Modify Parameter", MitreTactic::IcsImpairProcessControl),
        "T0814" => ("Denial of Service", MitreTactic::IcsInhibitResponseFunction),
        "T0806" => ("Brute Force I/O", MitreTactic::IcsImpairProcessControl),
        "T0835" => ("Manipulate I/O Image", MitreTactic::IcsImpairProcessControl),
        "T0831" => ("Manipulation of Control", MitreTactic::IcsImpact),
        "T0888" => (
            "Remote System Information Discovery",
            MitreTactic::IcsDiscovery,
        ),
        // STORY-109 / VP-007 atomic obligation — seeded together with the
        // T1691.001 and T0827 emission branches (BC-2.15.014 / BC-2.15.015).
        "T1691.001" => (
            "Block Operational Technology Message: Command Message",
            MitreTactic::IcsInhibitResponseFunction,
        ),
        "T0827" => ("Loss of Control", MitreTactic::IcsImpact),
        // STORY-114 / VP-007 atomic obligation — seeded together with the
        // T0830 and T1557.002 emission branches (D1/D12/GARP-conflict ARP spoof).
        "T0830" => ("Adversary-in-the-Middle", MitreTactic::IcsCollection),
        "T1557.002" => (
            "Adversary-in-the-Middle: ARP Cache Poisoning",
            MitreTactic::CredentialAccess,
        ),
        // STORY-133 / VP-007 atomic obligation — seeded together with T0858, T0816, T1693.001
        // EMITTED entries (T0858/T0816/T0846) and SEEDED array additions (Steps 2–4).
        "T0858" => ("Change Operating Mode", MitreTactic::IcsExecution),
        "T0816" => (
            "Device Restart/Shutdown",
            MitreTactic::IcsInhibitResponseFunction,
        ),
        "T1693.001" => (
            "Modify Firmware: System Firmware",
            MitreTactic::IcsInhibitResponseFunction,
        ),
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

/// Resolves a technique ID to the canonical MITRE ATT&CK tactic TA-prefix ID
/// string for the technique's parent tactic (BC-2.11.035 Catalog Extension).
///
/// Returns `None` when the technique ID does not resolve in the catalog.
/// Mapping is: each `MitreTactic` variant → its canonical TA-prefix string
/// (e.g., `Discovery` → `"TA0007"`, `IcsImpact` → `"TA0105"`).
///
/// The match on `MitreTactic` is exhaustive so a future variant added to the
/// enum without updating this table produces a compile error.
pub fn technique_tactic_id(id: &str) -> Option<&'static str> {
    let tactic = technique_tactic(id)?;
    // Exhaustive match over every current MitreTactic variant — a new variant
    // added to the enum WITHOUT updating this table produces a compile error,
    // enforcing the BC-2.11.035 invariant that the mapping cannot silently drift.
    // Note: `#[non_exhaustive]` on the enum applies only to external crates;
    // within this crate the match is fully exhaustive and no wildcard arm is needed.
    let ta_id = match tactic {
        MitreTactic::Reconnaissance => "TA0043",
        MitreTactic::ResourceDevelopment => "TA0042",
        MitreTactic::InitialAccess => "TA0001",
        MitreTactic::Execution => "TA0002",
        MitreTactic::Persistence => "TA0003",
        MitreTactic::PrivilegeEscalation => "TA0004",
        MitreTactic::DefenseEvasion => "TA0005",
        MitreTactic::CredentialAccess => "TA0006",
        MitreTactic::Discovery => "TA0007",
        MitreTactic::LateralMovement => "TA0008",
        MitreTactic::Collection => "TA0009",
        MitreTactic::CommandAndControl => "TA0011",
        MitreTactic::Exfiltration => "TA0010",
        MitreTactic::Impact => "TA0040",
        MitreTactic::IcsInhibitResponseFunction => "TA0107",
        MitreTactic::IcsImpairProcessControl => "TA0106",
        MitreTactic::IcsImpact => "TA0105",
        MitreTactic::IcsDiscovery => "TA0102",
        MitreTactic::IcsCollection => "TA0100",
        MitreTactic::IcsCommandAndControl => "TA0101",
        MitreTactic::IcsExecution => "TA0104",
    };
    Some(ta_id)
}

// ── VP-007: MITRE Technique ID Format and Catalog Completeness ────────────────
//
// Sub-property A (ID format): every seeded ID matches `T[0-9]{4}` or
// `T[0-9]{4}.[0-9]{3}`.
// Sub-property B (completeness): every seeded ID and every analyzer-emitted ID
// resolves in `technique_info` (both name and tactic Some).
// Corollary (BC-2.10.006): unknown IDs return None without panicking.
//
// The catalogue is a closed-world static match; the seeded set is finite (28)
// so the harness enumerates it exhaustively — fully sound, no abstraction.
//
// To audit the emitted IDs: `grep -rn 'mitre_techniques: vec!' src/`
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// All 28 seeded IDs (mirrors `technique_info`, this file). If `technique_info`
    /// gains/loses an entry, the completeness proof here will diverge from the
    /// table and must be updated in lockstep with the VP.
    const SEEDED_IDS: &[&str] = super::SEEDED_TECHNIQUE_IDS;

    /// IDs actually emitted by analyzers today (`grep -rn 'mitre_techniques: vec!' src/`).
    /// 6 Enterprise + 7 ICS + 2 STORY-109 + 2 ARP (STORY-114) + 3 ENIP (STORY-133) = 20 emitted IDs
    /// (BC-2.10.008 postcondition 1). T0846 promoted from seeded-only to emitted (STORY-133, Step 4).
    /// T0888 IS emitted (Modbus recon). T1693.001 is seeded-only (staged for v0.12.0).
    /// Sub-property B's emitter half: each must resolve in the catalogue.
    const EMITTED_IDS: &[&str] = &[
        // Enterprise (6)
        "T1027",     // TLS: SNI anomaly
        "T1036",     // Reassembly: conflicting overlap
        "T1046",     // HTTP: admin panel
        "T1083",     // HTTP: path traversal
        "T1499.002", // HTTP: header flood
        "T1505.003", // HTTP: web shell
        // ICS (7) — T1692.001 (remapped from T0855 per v19 remap) + 6 new F2 IDs
        "T1692.001", // ICS Impair Process Control (remapped from T0855, v19 remap issue #222)
        "T0836",     // Modify Parameter
        "T0814",     // Denial of Service
        "T0806",     // Brute Force I/O
        "T0835",     // Manipulate I/O Image
        "T0831",     // Manipulation of Control
        "T0888",     // Remote System Information Discovery
        // STORY-109 (2) — VP-007 atomic obligation; implemented in STORY-109.
        "T1691.001", // Block OT Message: Command Message (BC-2.15.014; IcsInhibitResponseFunction)
        "T0827",     // Loss of Control (BC-2.15.015; IcsImpact)
        // STORY-114 (2) — VP-007 atomic obligation; ARP D1/D12/GARP-conflict spoof detection.
        "T0830",     // Adversary-in-the-Middle (BC-2.16.004; IcsCollection/TA0100)
        "T1557.002", // ARP Cache Poisoning (BC-2.16.004; CredentialAccess)
        // STORY-133 (3) — VP-007 ENIP atomic obligation; T1693.001 NOT here (staged-only).
        "T0858", // Change Operating Mode (BC-2.17.011; IcsExecution/TA0104)
        "T0816", // Device Restart/Shutdown (BC-2.17.012; IcsInhibitResponseFunction/TA0107)
        "T0846", // Remote System Discovery (BC-2.17.010; IcsDiscovery/TA0102; promoted from seeded-only)
    ];

    /// Sub-property A: format invariant `T[0-9]{4}` or `T[0-9]{4}.[0-9]{3}`.
    ///
    /// BOUND/SOUNDNESS: the seeded set is a finite closed enumeration (28 IDs);
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
    /// arm is `_ => None`; any string outside the 25 seeded literals takes it.
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
///
/// Count history: Post-F2 (STORY-100) 11 Enterprise + 10 ICS = 21 total (pre-STORY-109 subtotal).
/// STORY-109 (VP-007 atomic obligation) +2 ICS (T1691.001, T0827) = 23 total.
/// STORY-114 (VP-007 ARP obligation) +2 ARP (T0830 ICS IcsCollection/TA0100, T1557.002 Enterprise CredentialAccess)
///   = 25 total (12 Enterprise + 13 ICS; normative split per VP-007 §CC-003).
/// STORY-133 (VP-007 ENIP obligation) +3 ENIP (T0858 IcsExecution/TA0104, T0816 IcsInhibitResponseFunction/TA0107,
///   T1693.001 staged IcsInhibitResponseFunction/TA0107) = 28 total.
/// ICS v19 remap (issue #222): T0855→T1692.001, T0856→T1692.002.
#[cfg(any(kani, test))]
const SEEDED_TECHNIQUE_IDS: &[&str] = &[
    // Enterprise (12 total: 11 below + T1557.002 in the ARP STORY-114 section)
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
    // ICS pre-F2 (4)
    "T0846",
    "T1692.001",
    "T1692.002",
    "T0885",
    // ICS new F2 (6) — STORY-100 additions
    "T0836",
    "T0814",
    "T0806",
    "T0835",
    "T0831",
    "T0888",
    // ICS STORY-109 (2) — VP-007 atomic obligation
    "T1691.001",
    "T0827",
    // ARP STORY-114 (2) — VP-007 atomic obligation
    "T0830",
    "T1557.002",
    // ENIP STORY-133 (3) — VP-007 atomic obligation
    "T0858",
    "T0816",
    "T1693.001",
];

/// Expected number of Some-returning arms in [`technique_info`]. Declared
/// separately from `SEEDED_TECHNIQUE_IDS.len()` so the drift guard catches BOTH
/// directions of accidental edit: bumping this without adding an ID (or vice
/// versa) fails the test. Must equal the count of `=> (...)` arms in
/// `technique_info` (currently 28: 21 post-F2/STORY-100 + 2 STORY-109 + 2 ARP/STORY-114
/// + 3 ENIP/STORY-133 additions).
#[cfg(any(kani, test))]
const SEEDED_TECHNIQUE_ID_COUNT: usize = 28;

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
        // ICS v19 sub-technique IDs (issue #222): must also be accepted.
        assert!(is_valid_technique_id_format("T1692.001"));
        assert!(is_valid_technique_id_format("T1692.002"));
        // Malformed cases must be rejected.
        assert!(!is_valid_technique_id_format("TXXXX"));
        assert!(!is_valid_technique_id_format("T102")); // too short
        assert!(!is_valid_technique_id_format("T10277")); // too long, no dot
        assert!(!is_valid_technique_id_format("T1071.01")); // 2-digit suffix
        assert!(!is_valid_technique_id_format("T1071.0001")); // 4-digit suffix
        assert!(!is_valid_technique_id_format("X1027")); // wrong prefix
        assert!(!is_valid_technique_id_format("T1071,001")); // wrong separator
    }

    /// CR-005 / CR-006: mechanically link `SEEDED_TECHNIQUE_IDS` to
    /// `technique_info` so the VP-007 completeness proofs cannot silently go
    /// stale — in EITHER direction. Rather than trust a hand-maintained count,
    /// this test DERIVES the true catalogue size by sweeping the entire finite
    /// technique-ID space and counting how many IDs `technique_info` resolves,
    /// then asserts that derived count equals `SEEDED_TECHNIQUE_IDS.len()`.
    ///
    /// The ID grammar (see `is_valid_technique_id_format`) is closed and finite:
    /// parent IDs `T[0-9]{4}` (10_000 of them) and sub-technique IDs
    /// `T[0-9]{4}.[0-9]{3}` (10_000 × 1_000).
    ///
    /// `technique_info`'s match is literal-equality on string keys, so any
    /// resolving key MUST be one of these well-formed shapes; sweeping both
    /// shapes therefore enumerates every key the catalogue could possibly hold.
    ///
    /// This closes the residual hole in the old count==const check: adding a new
    /// arm to `technique_info` (e.g. `T1999`) WITHOUT mirroring it in
    /// `SEEDED_TECHNIQUE_IDS` now makes the derived resolved-count exceed
    /// `SEEDED_TECHNIQUE_IDS.len()`, failing this test. Removing/renaming an arm
    /// makes it fall short. The test thus enforces FORWARD completeness, not just
    /// that the 15 known IDs still resolve.
    ///
    /// Retains the shrinkage / duplicate / malformed / resolve checks on the
    /// seeded list, plus the documented-count cross-check and the `T9999` canary.
    #[test]
    fn vp007_catalog_drift_guard() {
        // --- Seeded-list self-consistency (unchanged) -------------------------
        // Documented count cross-check (a second, independent tripwire).
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
        // Canary: a well-formed but unregistered ID must NOT resolve.
        assert!(is_valid_technique_id_format("T9999"));
        assert!(
            technique_info("T9999").is_none(),
            "canary T9999 resolved unexpectedly"
        );

        // --- CR-006: derive the catalogue size from the source of truth ------
        // Sweep the entire finite ID space and count resolutions. A reusable
        // String buffer avoids per-iteration allocation across the ~10.01M probes.
        let seeded: std::collections::HashSet<&str> =
            SEEDED_TECHNIQUE_IDS.iter().copied().collect();
        let mut resolved = 0usize;
        let mut buf = String::with_capacity(9);

        // Parent shape: T[0-9]{4} (10_000 IDs).
        for n in 0..10_000u32 {
            buf.clear();
            use std::fmt::Write as _;
            write!(buf, "T{n:04}").unwrap();
            if technique_info(&buf).is_some() {
                resolved += 1;
                assert!(
                    seeded.contains(buf.as_str()),
                    "technique_info resolves {buf} but it is missing from \
                     SEEDED_TECHNIQUE_IDS — mirror it (and bump \
                     SEEDED_TECHNIQUE_ID_COUNT)"
                );
            }
        }
        // Sub-technique shape: T[0-9]{4}.[0-9]{3} (10_000 × 1_000 IDs).
        for n in 0..10_000u32 {
            for s in 0..1_000u32 {
                buf.clear();
                use std::fmt::Write as _;
                write!(buf, "T{n:04}.{s:03}").unwrap();
                if technique_info(&buf).is_some() {
                    resolved += 1;
                    assert!(
                        seeded.contains(buf.as_str()),
                        "technique_info resolves {buf} but it is missing from \
                         SEEDED_TECHNIQUE_IDS — mirror it (and bump \
                         SEEDED_TECHNIQUE_ID_COUNT)"
                    );
                }
            }
        }

        // The derived catalogue size MUST equal the seeded-list size. This is
        // the forward-completeness guarantee: no unmirrored addition can hide.
        assert_eq!(
            resolved,
            SEEDED_TECHNIQUE_IDS.len(),
            "technique_info resolves {resolved} IDs but SEEDED_TECHNIQUE_IDS has \
             {} — the catalogue and the seeded list have drifted",
            SEEDED_TECHNIQUE_IDS.len()
        );

        // --- BC-2.11.035 Catalog Extension: technique_tactic_id drift guard ----
        // Every seeded ID that resolves in technique_info must also return Some
        // from technique_tactic_id — the TA-ID mapping table must be exhaustive.
        for id in SEEDED_TECHNIQUE_IDS {
            assert!(
                technique_tactic_id(id).is_some(),
                "seeded ID {id} resolves in technique_info but technique_tactic_id \
                 returns None — the MitreTactic→TA-ID mapping table is incomplete; \
                 add the new MitreTactic variant to technique_tactic_id"
            );
        }
        // Canary: unknown ID must also return None from technique_tactic_id.
        assert!(
            technique_tactic_id("T9999").is_none(),
            "canary T9999 must return None from technique_tactic_id"
        );
    }
}
