//! Behavioral-contract test suite for STORY-071: MITRE ATT&CK Mapping.
//!
//! Test naming follows BC-based convention per TDD Iron Law:
//! `test_BC_S_SS_NNN_xxx()` where S=2, SS=10, NNN=001..009.
//!
//! Each test references its BC postcondition/invariant in its doc comment.
//! All tests verify the existing brownfield implementation in src/mitre.rs.

use std::collections::HashSet;

use wirerust::mitre::{MitreTactic, all_tactics_in_report_order, technique_name, technique_tactic};

// ---------------------------------------------------------------------------
// AC-001 | BC-2.10.001 postcondition 1
// All 14 Enterprise MitreTactic variants render with canonical ATT&CK names.
// ---------------------------------------------------------------------------
#[test]
fn test_all_enterprise_tactic_display_strings() {
    // BC-2.10.001 postcondition 1: 14 Enterprise tactic variants render as
    // canonical ATT&CK tactic name strings (spaces included in multi-word names).
    assert_eq!(MitreTactic::Reconnaissance.to_string(), "Reconnaissance");
    assert_eq!(
        MitreTactic::ResourceDevelopment.to_string(),
        "Resource Development"
    );
    assert_eq!(MitreTactic::InitialAccess.to_string(), "Initial Access");
    assert_eq!(MitreTactic::Execution.to_string(), "Execution");
    assert_eq!(MitreTactic::Persistence.to_string(), "Persistence");
    assert_eq!(
        MitreTactic::PrivilegeEscalation.to_string(),
        "Privilege Escalation"
    );
    assert_eq!(MitreTactic::DefenseEvasion.to_string(), "Defense Evasion");
    assert_eq!(
        MitreTactic::CredentialAccess.to_string(),
        "Credential Access"
    );
    assert_eq!(MitreTactic::Discovery.to_string(), "Discovery");
    assert_eq!(MitreTactic::LateralMovement.to_string(), "Lateral Movement");
    assert_eq!(MitreTactic::Collection.to_string(), "Collection");
    assert_eq!(
        MitreTactic::CommandAndControl.to_string(),
        "Command and Control"
    );
    assert_eq!(MitreTactic::Exfiltration.to_string(), "Exfiltration");
    assert_eq!(MitreTactic::Impact.to_string(), "Impact");
}

// ---------------------------------------------------------------------------
// AC-002 | BC-2.10.001 invariant 3
// "Command and Control" uses lowercase "and" (canonical ATT&CK form).
// (Part of AC-001 test; kept as standalone function per story spec W1.4.)
// ---------------------------------------------------------------------------
#[test]
fn test_command_and_control_lowercase_and() {
    // BC-2.10.001 invariant 3: "Command and Control" uses lowercase "and",
    // not "And". Exact string equality required.
    let display = MitreTactic::CommandAndControl.to_string();
    assert_eq!(display, "Command and Control");
    assert!(
        !display.contains("And"),
        "expected lowercase 'and', found 'And' in: {display:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-003 | BC-2.10.002 postcondition 1
// IcsInhibitResponseFunction displays as "Inhibit Response Function" (no prefix).
// ---------------------------------------------------------------------------
#[test]
fn test_ics_inhibit_response_function_display() {
    // BC-2.10.002 postcondition 1: IcsInhibitResponseFunction => "Inhibit Response Function"
    // No "ICS:" prefix or any other matrix qualifier.
    let display = MitreTactic::IcsInhibitResponseFunction.to_string();
    assert_eq!(display, "Inhibit Response Function");
    assert!(
        !display.starts_with("ICS:"),
        "ICS tactic must not have 'ICS:' prefix, got: {display:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-004 | BC-2.10.002 postcondition 2
// IcsImpairProcessControl displays as "Impair Process Control" (no prefix).
// ---------------------------------------------------------------------------
#[test]
fn test_ics_impair_process_control_display() {
    // BC-2.10.002 postcondition 2: IcsImpairProcessControl => "Impair Process Control"
    // No "ICS:" prefix or any other matrix qualifier.
    let display = MitreTactic::IcsImpairProcessControl.to_string();
    assert_eq!(display, "Impair Process Control");
    assert!(
        !display.starts_with("ICS:"),
        "ICS tactic must not have 'ICS:' prefix, got: {display:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-005 | BC-2.10.003 postcondition 1
// all_tactics_in_report_order().len() equals 17 (14 Enterprise + 3 ICS).
// STORY-109 adds MitreTactic::IcsImpact for T0827 (VP-007 atomic obligation),
// bringing the ICS-unique count from 2 to 3 and the total from 16 to 17.
// ---------------------------------------------------------------------------
#[test]
fn test_all_tactics_length_is_16() {
    // BC-2.10.003 postcondition 1 / invariant 2 (updated STORY-109):
    // 14 Enterprise + 3 ICS-unique = 17 variants.
    assert_eq!(
        all_tactics_in_report_order().len(),
        17,
        "expected 14 Enterprise + 3 ICS-unique = 17 variants (STORY-109 adds IcsImpact)"
    );
}

// ---------------------------------------------------------------------------
// AC-006 | BC-2.10.003 postcondition 2
// First 14 elements are Enterprise tactics in canonical kill-chain order.
// ---------------------------------------------------------------------------
#[test]
fn test_all_tactics_enterprise_kill_chain_order() {
    // BC-2.10.003 postcondition 2: Reconnaissance, ResourceDevelopment,
    // InitialAccess, Execution, Persistence, PrivilegeEscalation,
    // DefenseEvasion, CredentialAccess, Discovery, LateralMovement,
    // Collection, CommandAndControl, Exfiltration, Impact — in that order.
    let expected_enterprise: [MitreTactic; 14] = [
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
    ];
    assert_eq!(&all_tactics_in_report_order()[..14], &expected_enterprise);
}

// ---------------------------------------------------------------------------
// AC-007 | BC-2.10.003 postcondition 3
// Elements [14] and [15] are IcsInhibitResponseFunction and IcsImpairProcessControl.
// ---------------------------------------------------------------------------
#[test]
fn test_all_tactics_ics_at_end() {
    // BC-2.10.003 postcondition 3: ICS tactics appear after all 14 Enterprise
    // tactics at positions [14] and [15].
    let tactics = all_tactics_in_report_order();
    assert_eq!(
        tactics[14],
        MitreTactic::IcsInhibitResponseFunction,
        "position [14] must be IcsInhibitResponseFunction"
    );
    assert_eq!(
        tactics[15],
        MitreTactic::IcsImpairProcessControl,
        "position [15] must be IcsImpairProcessControl"
    );
}

// ---------------------------------------------------------------------------
// AC-008 | BC-2.10.004 postcondition 1 & 2
// Collecting all_tactics_in_report_order() into a HashSet gives size 17.
// STORY-109 adds MitreTactic::IcsImpact → total 17.
// ---------------------------------------------------------------------------
#[test]
fn test_all_tactics_no_duplicates() {
    // BC-2.10.004 postcondition 1: each variant appears exactly once.
    // BC-2.10.004 postcondition 2: no variant appears twice.
    // Verified by: HashSet deduplication produces same len as slice.
    let tactics = all_tactics_in_report_order();
    let unique: HashSet<MitreTactic> = tactics.iter().copied().collect();
    assert_eq!(
        unique.len(),
        tactics.len(),
        "duplicate variant detected in all_tactics_in_report_order()"
    );
    // STORY-109: IcsImpact added → 17 total (was 16 post-F2).
    assert_eq!(unique.len(), 17);
}

// ---------------------------------------------------------------------------
// AC-009 | BC-2.10.004 postcondition 3
// No variant omitted — all 17 variants appear in the slice.
// STORY-109 adds MitreTactic::IcsImpact → total 17.
// ---------------------------------------------------------------------------
#[test]
fn test_all_tactics_all_variants_present() {
    // BC-2.10.004 postcondition 3: no variant omitted.
    // Verified by comparing the collected HashSet against the full expected set.
    let actual: HashSet<MitreTactic> = all_tactics_in_report_order().iter().copied().collect();

    let expected: HashSet<MitreTactic> = [
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
        // STORY-109 (VP-007 atomic obligation) — IcsImpact for T0827
        MitreTactic::IcsImpact,
    ]
    .into_iter()
    .collect();

    assert_eq!(
        actual, expected,
        "all_tactics_in_report_order() is missing or has extra variants"
    );
}

// ---------------------------------------------------------------------------
// AC-010 + AC-011 | BC-2.10.005 postcondition 1
// All 21 seeded technique IDs resolve to Some(name). Includes T1027 check
// (AC-010) and exhaustive check of all 21 (AC-011) in the same function.
// ---------------------------------------------------------------------------
#[test]
fn test_technique_name_resolves_all_21_seeded_ids() {
    // BC-2.10.005 postcondition 1: technique_name returns Some for each of the
    // 21 seeded IDs. Catalog count is exactly 21 (post-F2 / STORY-100).
    // Seeded IDs (11 Enterprise + 10 ICS):
    //   T1027, T1036, T1040, T1046, T1071, T1071.001, T1071.004,
    //   T1083, T1499.002, T1505.003, T1573,
    //   T0846, T1692.001, T1692.002, T0885,
    //   T0836, T0814, T0806, T0835, T0831, T0888.

    // AC-010: spot-check on canonical test vector from BC-2.10.005.
    assert_eq!(
        technique_name("T1027"),
        Some("Obfuscated Files or Information")
    );

    // AC-011: exhaustive check of all 21 entries. Names are derived from
    // technique_info (src/mitre.rs) — asserted here to catch catalogue drift.
    let seeded: &[(&str, &str)] = &[
        // Enterprise (11)
        ("T1027", "Obfuscated Files or Information"),
        ("T1036", "Masquerading"),
        ("T1040", "Network Sniffing"),
        ("T1046", "Network Service Discovery"),
        ("T1071", "Application Layer Protocol"),
        ("T1071.001", "Web Protocols"),
        ("T1071.004", "DNS"),
        ("T1083", "File and Directory Discovery"),
        ("T1499.002", "Service Exhaustion Flood"),
        ("T1505.003", "Web Shell"),
        ("T1573", "Encrypted Channel"),
        // ICS pre-F2 (4) — T0855/T0856 remapped to T1692.001/T1692.002 per v19 (issue #222)
        ("T0846", "Remote System Discovery"),
        ("T1692.001", "Unauthorized Message: Command Message"),
        ("T1692.002", "Unauthorized Message: Reporting Message"),
        ("T0885", "Commonly Used Port"),
        // ICS new F2 — STORY-100 additions (6)
        ("T0836", "Modify Parameter"),
        ("T0814", "Denial of Service"),
        ("T0806", "Brute Force I/O"),
        ("T0835", "Manipulate I/O Image"),
        ("T0831", "Manipulation of Control"),
        ("T0888", "Remote System Information Discovery"),
    ];

    assert_eq!(
        seeded.len(),
        21,
        "catalog count must be exactly 21 (post-F2 / STORY-100)"
    );

    for (id, expected_name) in seeded {
        // Derive the live value from technique_info to avoid hardcoded
        // false-greens — if the catalogue name changes, this test fails.
        let live_name = technique_name(id);
        assert!(
            live_name.is_some(),
            "technique_name({id:?}) returned None — ID missing from catalogue"
        );
        assert_eq!(
            live_name,
            Some(*expected_name),
            "technique_name({id:?}) returned unexpected value (catalogue drift?)"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-012 | BC-2.10.006 postcondition 1
// technique_name returns None for "T9999", "", and "t1027" (lowercase).
// ---------------------------------------------------------------------------
#[test]
fn test_technique_name_returns_none_for_unknown_ids() {
    // BC-2.10.006 postcondition 1: returns None for any ID not in the 21-entry
    // static match table. No panic, no error, no default string.
    // BC-2.10.006 invariant 1: match is exact string equality (case-sensitive, no trim).
    assert_eq!(technique_name("T9999"), None);
    assert_eq!(technique_name(""), None);
    assert_eq!(
        technique_name("t1027"),
        None,
        "match must be case-sensitive"
    );
}

// ---------------------------------------------------------------------------
// AC-013 + AC-014 | BC-2.10.007 postcondition 2
// All 21 seeded technique-to-tactic assignments are correct.
// AC-013 spot-checks T1027 => DefenseEvasion; AC-014 is exhaustive.
// ---------------------------------------------------------------------------
#[test]
fn test_technique_tactic_correct_assignments() {
    // BC-2.10.007 postcondition 2: tactic assignments match ATT&CK matrix.
    // AC-013: spot-check canonical test vector.
    assert_eq!(
        technique_tactic("T1027"),
        Some(MitreTactic::DefenseEvasion),
        "T1027 must map to DefenseEvasion"
    );

    // AC-014: exhaustive table per BC-2.10.007 postcondition 2.
    // Includes all 21 seeded IDs (11 Enterprise + 10 ICS, post-F2 / STORY-100).
    let assignments: &[(&str, MitreTactic)] = &[
        // Enterprise (11)
        ("T1027", MitreTactic::DefenseEvasion),
        ("T1036", MitreTactic::DefenseEvasion),
        ("T1040", MitreTactic::CredentialAccess),
        ("T1046", MitreTactic::Discovery),
        ("T1071", MitreTactic::CommandAndControl),
        ("T1071.001", MitreTactic::CommandAndControl),
        ("T1071.004", MitreTactic::CommandAndControl),
        ("T1083", MitreTactic::Discovery),
        ("T1499.002", MitreTactic::Impact),
        ("T1505.003", MitreTactic::Persistence),
        ("T1573", MitreTactic::CommandAndControl),
        // ICS pre-F2 (4) — T0855/T0856 remapped to T1692.001/T1692.002 per v19 (issue #222)
        ("T0846", MitreTactic::Discovery),
        ("T1692.001", MitreTactic::IcsImpairProcessControl),
        ("T1692.002", MitreTactic::IcsImpairProcessControl),
        ("T0885", MitreTactic::CommandAndControl),
        // ICS new F2 — STORY-100 additions (6)
        ("T0836", MitreTactic::IcsImpairProcessControl),
        ("T0806", MitreTactic::IcsImpairProcessControl),
        ("T0835", MitreTactic::IcsImpairProcessControl),
        ("T0831", MitreTactic::IcsImpairProcessControl),
        ("T0814", MitreTactic::IcsInhibitResponseFunction),
        ("T0888", MitreTactic::Discovery),
    ];

    for (id, expected_tactic) in assignments {
        assert_eq!(
            technique_tactic(id),
            Some(*expected_tactic),
            "technique_tactic({id:?}) returned unexpected tactic"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-015 | BC-2.10.008 postcondition 1
// All 13 currently-emitted technique IDs resolve from both technique_name
// and technique_tactic (6 Enterprise + 7 ICS, post-F2 / STORY-100).
// ---------------------------------------------------------------------------
#[test]
fn test_all_emitted_ids_resolve() {
    // BC-2.10.008 postcondition 1: emitted set is a strict subset of the
    // catalogued 21 IDs. No emitted ID may return None from either lookup.
    // Sources (ground truth: `grep -rn 'mitre_techniques: vec!' src/`):
    //   src/analyzer/tls.rs          — T1027 (×3 sites)
    //   src/analyzer/http.rs         — T1083, T1505.003, T1046, T1499.002 (×2 sites)
    //   src/reassembly/mod.rs        — T1036
    //   src/reassembly/lifecycle.rs  — T1036
    //   ICS analyzers (Modbus/DNP3)  — T1692.001, T0836, T0814, T0806, T0835, T0831, T0888
    // Total: 6 Enterprise + 7 ICS = 13 unique emitted technique IDs.
    // Note: T0846 is seeded but NOT emitted; T0888 IS emitted (Modbus recon).
    // ICS v19 remap (issue #222): T0855→T1692.001 (emitted), T0856→T1692.002 (seeded-only).
    let emitted_ids = [
        // Enterprise (6)
        "T1027",     // src/analyzer/tls.rs
        "T1036",     // src/reassembly/mod.rs AND src/reassembly/lifecycle.rs
        "T1046",     // src/analyzer/http.rs
        "T1083",     // src/analyzer/http.rs
        "T1499.002", // src/analyzer/http.rs
        "T1505.003", // src/analyzer/http.rs
        // ICS (7) — T1692.001 (remapped from T0855 per v19 remap, issue #222) + 6 new F2 IDs (STORY-100)
        "T1692.001", // ICS Impair Process Control (remapped from T0855)
        "T0836",     // Modify Parameter
        "T0814",     // Denial of Service
        "T0806",     // Brute Force I/O
        "T0835",     // Manipulate I/O Image
        "T0831",     // Manipulation of Control
        "T0888",     // Remote System Information Discovery
    ];

    assert_eq!(
        emitted_ids.len(),
        13,
        "there are exactly 13 currently-emitted IDs per BC-2.10.008 (post-F2 / STORY-100)"
    );

    for id in &emitted_ids {
        assert!(
            technique_name(id).is_some(),
            "analyzer emits {id} but technique_name({id}) returned None"
        );
        assert!(
            technique_tactic(id).is_some(),
            "analyzer emits {id} but technique_tactic({id}) returned None"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-016 | BC-2.10.009 postcondition 1
// MitreTactic has #[non_exhaustive] attribute.
// Grep-based: reads src/mitre.rs and asserts the attribute is present.
// ---------------------------------------------------------------------------
#[test]
fn test_mitre_tactic_is_non_exhaustive() {
    // BC-2.10.009 invariant 1: #[non_exhaustive] must be placed directly on the
    // MitreTactic enum definition, enabling non-breaking variant additions when
    // new ATT&CK versions introduce new tactics.
    //
    // The assertion checks adjacency — that `#[non_exhaustive]` immediately
    // precedes `pub enum MitreTactic` in the source — rather than merely
    // checking that the attribute exists somewhere in the file. A weaker
    // `src.contains("#[non_exhaustive]")` check would pass even if the
    // attribute were removed from MitreTactic and placed on a different item.
    let src = std::fs::read_to_string("src/mitre.rs")
        .expect("src/mitre.rs must be readable from the worktree root");
    assert!(
        src.contains("#[non_exhaustive]\npub enum MitreTactic"),
        "src/mitre.rs must have '#[non_exhaustive]' immediately preceding \
         'pub enum MitreTactic' (BC-2.10.009 invariant 1)"
    );
}

// ---------------------------------------------------------------------------
// EC-001 | BC-2.10.006 invariant 3
// technique_name("T1059") — real ATT&CK but not seeded — returns None.
// ---------------------------------------------------------------------------
#[test]
fn test_ec_001_real_unseed_technique_returns_none() {
    // BC-2.10.006 invariant 3: "T1059" is a real ATT&CK technique but is
    // not present in the 21-entry seeded catalog. Must return None.
    assert_eq!(technique_name("T1059"), None);
}

// ---------------------------------------------------------------------------
// EC-002 | BC-2.10.006 invariant 1
// technique_name("T1046.001") — sub-technique of seeded parent, not itself
// seeded — returns None.
// ---------------------------------------------------------------------------
#[test]
fn test_ec_002_unseeded_sub_technique_returns_none() {
    // BC-2.10.006 invariant 1: "T1046.001" is a sub-technique of the seeded
    // parent T1046 but is not itself seeded. Exact string match — must return None.
    assert_eq!(technique_name("T1046.001"), None);
}

// ---------------------------------------------------------------------------
// EC-003 | BC-2.10.006 invariant 1 (no trimming)
// technique_name(" T1027") — leading space — returns None.
// ---------------------------------------------------------------------------
#[test]
fn test_ec_003_leading_space_returns_none() {
    // BC-2.10.006 invariant 1: the match is exact string equality with no
    // whitespace trimming. " T1027" (leading space) must return None.
    assert_eq!(technique_name(" T1027"), None);
}

// ---------------------------------------------------------------------------
// EC-004 | BC-2.10.007 postcondition 3
// technique_tactic("T9999") returns None.
// ---------------------------------------------------------------------------
#[test]
fn test_ec_004_unknown_id_tactic_returns_none() {
    // BC-2.10.007 postcondition 3: technique_tactic returns None for any ID
    // not in the seeded set. "T9999" is unknown.
    assert_eq!(technique_tactic("T9999"), None);
}

// ---------------------------------------------------------------------------
// ICS v19 remap (issue #222): explicit pin for T1692.001 emitted and sub-technique format
// ---------------------------------------------------------------------------

/// Explicit pin test: T1692.001 is the emitted ID for Modbus write findings (v19 remap).
///
/// Verifies:
/// 1. T1692.001 resolves in the catalog with the expected name and tactic.
/// 2. T1692.002 resolves in the catalog with the expected name and tactic (seeded-only).
/// 3. The sub-technique format validator accepts T1692.001 and T1692.002.
/// 4. The old revoked IDs T0855 and T0856 no longer resolve (removed from catalog).
#[test]
fn test_ics_v19_remap_t1692_sub_techniques_are_pinned() {
    // T1692.001 — emitted by Modbus analyzer (replaces T0855).
    assert_eq!(
        technique_name("T1692.001"),
        Some("Unauthorized Message: Command Message"),
        "T1692.001 must resolve to the v19 name"
    );
    assert_eq!(
        technique_tactic("T1692.001"),
        Some(MitreTactic::IcsImpairProcessControl),
        "T1692.001 tactic must be IcsImpairProcessControl (unchanged from T0855)"
    );

    // T1692.002 — seeded-only (replaces T0856, never emitted).
    assert_eq!(
        technique_name("T1692.002"),
        Some("Unauthorized Message: Reporting Message"),
        "T1692.002 must resolve to the v19 name"
    );
    assert_eq!(
        technique_tactic("T1692.002"),
        Some(MitreTactic::IcsImpairProcessControl),
        "T1692.002 tactic must be IcsImpairProcessControl (unchanged from T0856)"
    );

    // Revoked IDs must NO LONGER resolve in the catalog.
    assert_eq!(
        technique_name("T0855"),
        None,
        "T0855 is revoked in v19 — must not resolve"
    );
    assert_eq!(
        technique_name("T0856"),
        None,
        "T0856 is revoked in v19 — must not resolve"
    );
}

// ---------------------------------------------------------------------------
// EC-005 | BC-2.10.003 postcondition 2 (first element)
// all_tactics_in_report_order()[0] is MitreTactic::Reconnaissance.
// ---------------------------------------------------------------------------
#[test]
fn test_ec_005_first_tactic_is_reconnaissance() {
    // BC-2.10.003 postcondition 2: Reconnaissance is the first element of the
    // kill-chain-ordered slice (position [0]).
    assert_eq!(
        all_tactics_in_report_order()[0],
        MitreTactic::Reconnaissance,
        "first element of kill-chain order must be Reconnaissance"
    );
}
