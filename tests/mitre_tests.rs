//! Behavioral-contract test suite for STORY-071: MITRE ATT&CK Mapping.
//!
//! Test naming follows BC-based convention per TDD Iron Law:
//! `test_BC_S_SS_NNN_xxx()` where S=2, SS=10, NNN=001..009.
//!
//! Each test references its BC postcondition/invariant in its doc comment.
//! All tests verify the existing brownfield implementation in src/mitre.rs.

use std::collections::HashSet;

use wirerust::mitre::{
    MitreTactic, all_tactics_in_report_order, technique_name, technique_tactic, technique_tactic_id,
};

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
// all_tactics_in_report_order().len() equals 21 (14 Enterprise + 7 ICS).
// F5 adds MitreTactic::IcsDiscovery, IcsCollection, IcsCommandAndControl,
// bringing the ICS-unique count from 3 to 6 and the total from 17 to 20.
// STORY-133 adds MitreTactic::IcsExecution, bringing ICS-unique to 7 and total to 21.
// ---------------------------------------------------------------------------
#[test]
fn test_all_tactics_length_is_21() {
    // BC-2.10.003 postcondition 1 / invariant 2 (updated STORY-133):
    // 14 Enterprise + 7 ICS-unique = 21 variants.
    assert_eq!(
        all_tactics_in_report_order().len(),
        21,
        "expected 14 Enterprise + 7 ICS-unique = 21 variants (STORY-133 adds IcsExecution)"
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
// AC-007 | BC-2.10.003 postconditions 3 and 4
// Elements [14]-[16] are the first three ICS variants; [17]-[19] are the
// three new F5 ICS variants: IcsDiscovery, IcsCollection, IcsCommandAndControl.
// [20] is IcsExecution added in STORY-133 (VP-007 ENIP atomic burst).
// ---------------------------------------------------------------------------
#[test]
fn test_all_tactics_ics_at_end() {
    // BC-2.10.003 postcondition 3: ICS tactics at positions [14]-[16].
    // BC-2.10.003 postcondition 4: F5 ICS tactics at positions [17]-[19].
    // STORY-133: IcsExecution at position [20].
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
    assert_eq!(
        tactics[16],
        MitreTactic::IcsImpact,
        "position [16] must be IcsImpact (added F2 DNP3, STORY-109)"
    );
    // BC-2.10.003 postcondition 4 / EC-007/EC-008/EC-009 (F5 additions):
    assert_eq!(
        tactics[17],
        MitreTactic::IcsDiscovery,
        "position [17] must be IcsDiscovery (added F5)"
    );
    assert_eq!(
        tactics[18],
        MitreTactic::IcsCollection,
        "position [18] must be IcsCollection (added F5)"
    );
    assert_eq!(
        tactics[19],
        MitreTactic::IcsCommandAndControl,
        "position [19] must be IcsCommandAndControl (added F5)"
    );
    // STORY-133 (VP-007 ENIP atomic obligation):
    assert_eq!(
        tactics[20],
        MitreTactic::IcsExecution,
        "position [20] must be IcsExecution (added STORY-133)"
    );
}

// ---------------------------------------------------------------------------
// AC-008 | BC-2.10.004 postcondition 1 & 2
// Collecting all_tactics_in_report_order() into a HashSet gives size 21.
// F5 adds IcsDiscovery, IcsCollection, IcsCommandAndControl → total 20.
// STORY-133 adds IcsExecution → total 21.
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
    // STORY-133: IcsExecution added → 21 total (was 20 after F5).
    assert_eq!(unique.len(), 21);
}

// ---------------------------------------------------------------------------
// AC-009 | BC-2.10.004 postcondition 3
// No variant omitted — all 21 variants appear in the slice.
// F5 adds IcsDiscovery, IcsCollection, IcsCommandAndControl → total 20.
// STORY-133 adds IcsExecution → total 21.
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
        // F5 — 3 new ICS variants for correct ICS-matrix TA-IDs
        MitreTactic::IcsDiscovery,
        MitreTactic::IcsCollection,
        MitreTactic::IcsCommandAndControl,
        // STORY-133 (VP-007 ENIP atomic obligation) — IcsExecution for T0858
        MitreTactic::IcsExecution,
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
// All 21 STORY-100-era seeded technique IDs resolve to Some(name). Includes T1027 check
// (AC-010) and exhaustive check of all 21 (AC-011) in the same function.
// ---------------------------------------------------------------------------
#[test]
fn test_technique_name_resolves_all_21_seeded_ids() {
    // BC-2.10.005 postcondition 1: technique_name returns Some for each of the
    // 21 STORY-100-era seeded IDs (a stable subset of the 28-entry catalog).
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
    // BC-2.10.006 postcondition 1: returns None for any ID not in the 28-entry
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
// All seeded technique-to-tactic assignments are correct per MITRE ICS ATT&CK.
// AC-013 spot-checks T1027 => DefenseEvasion; AC-014 is exhaustive.
// F5 correctness fix: T0846/T0888 → IcsDiscovery, T0885 → IcsCommandAndControl,
// T0830 → IcsCollection, T0831 → IcsImpact.
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
    // F5 correctness fix applied: ICS techniques now use correct ICS-matrix variants.
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
        // ICS pre-F2 — T0855/T0856 remapped to T1692.001/T1692.002 per v19 (issue #222)
        // T0846: F5 fix — was Discovery (Enterprise TA0007), now IcsDiscovery (ICS TA0102)
        ("T0846", MitreTactic::IcsDiscovery),
        ("T1692.001", MitreTactic::IcsImpairProcessControl),
        ("T1692.002", MitreTactic::IcsImpairProcessControl),
        // T0885: F5 fix — was CommandAndControl (Enterprise TA0011), now IcsCommandAndControl (ICS TA0101)
        ("T0885", MitreTactic::IcsCommandAndControl),
        // ICS new F2 — STORY-100 additions (6)
        ("T0836", MitreTactic::IcsImpairProcessControl),
        ("T0806", MitreTactic::IcsImpairProcessControl),
        // T0835: no change — still IcsImpairProcessControl (TA0106) — confirmed correct
        ("T0835", MitreTactic::IcsImpairProcessControl),
        // T0831: F5 fix — was IcsImpairProcessControl (TA0106), now IcsImpact (ICS TA0105)
        ("T0831", MitreTactic::IcsImpact),
        ("T0814", MitreTactic::IcsInhibitResponseFunction),
        // T0888: F5 fix — was Discovery (Enterprise TA0007), now IcsDiscovery (ICS TA0102)
        ("T0888", MitreTactic::IcsDiscovery),
        // STORY-109 additions
        ("T1691.001", MitreTactic::IcsInhibitResponseFunction),
        ("T0827", MitreTactic::IcsImpact),
        // STORY-114 ARP additions
        // T0830: F5 fix — was LateralMovement (Enterprise TA0008), now IcsCollection (ICS TA0100)
        ("T0830", MitreTactic::IcsCollection),
        ("T1557.002", MitreTactic::CredentialAccess),
    ];

    for (id, expected_tactic) in assignments {
        assert_eq!(
            technique_tactic(id),
            Some(*expected_tactic),
            "technique_tactic({id:?}) returned unexpected tactic"
        );
    }

    // Explicit technique_tactic_id assertions for the F5-corrected techniques:
    // T0888 → IcsDiscovery → TA0102
    assert_eq!(
        technique_tactic_id("T0888"),
        Some("TA0102"),
        "T0888 must resolve to tactic_id TA0102 (IcsDiscovery, ICS ATT&CK)"
    );
    // T0846 → IcsDiscovery → TA0102
    assert_eq!(
        technique_tactic_id("T0846"),
        Some("TA0102"),
        "T0846 must resolve to tactic_id TA0102 (IcsDiscovery, ICS ATT&CK)"
    );
    // T0885 → IcsCommandAndControl → TA0101
    assert_eq!(
        technique_tactic_id("T0885"),
        Some("TA0101"),
        "T0885 must resolve to tactic_id TA0101 (IcsCommandAndControl, ICS ATT&CK)"
    );
    // T0830 → IcsCollection → TA0100
    assert_eq!(
        technique_tactic_id("T0830"),
        Some("TA0100"),
        "T0830 must resolve to tactic_id TA0100 (IcsCollection, ICS ATT&CK)"
    );
    // T0831 → IcsImpact → TA0105
    assert_eq!(
        technique_tactic_id("T0831"),
        Some("TA0105"),
        "T0831 must resolve to tactic_id TA0105 (IcsImpact, ICS ATT&CK)"
    );
}

// ---------------------------------------------------------------------------
// AC-015 | BC-2.10.008 postcondition 1
// All 13 currently-emitted technique IDs resolve from both technique_name
// and technique_tactic (6 Enterprise + 7 ICS, post-F2 / STORY-100).
// ---------------------------------------------------------------------------
#[test]
fn test_all_emitted_ids_resolve() {
    // BC-2.10.008 postcondition 1: emitted set is a strict subset of the
    // catalogued 28 IDs. No emitted ID may return None from either lookup.
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
    // not present in the 28-entry seeded catalog. Must return None.
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

// ---------------------------------------------------------------------------
// F5 Pass-1 process-gap hardening | authoritative ICS TA-id table
//
// Consolidated value-correctness pin for every ICS technique →
// technique_tactic_id() mapping, sourced from the F5 authoritative table:
//   .factory/cycles/feature-mitre-json-names/f5-ics-technique-tactic-authoritative.md
// (MITRE ATT&CK for ICS v19 / ics-attack-19.1, released 2026-04-20).
//
// Process gap from Pass-1: prior tests asserted MitreTactic variants but not
// the final TA-id string emitted by technique_tactic_id(). A future edit that
// swaps two valid ICS variants (e.g. IcsCollection↔IcsDiscovery) would produce
// the correct tactic name but the WRONG TA-id, silently passing tactic-variant
// tests while breaking TA-id correctness. This test closes that gap by asserting
// the exact (technique_id → tactic_id string) pair for every ICS technique.
//
// F-133-004 (STORY-133 adversarial High): extended to cover the 3 new ENIP ICS IDs
// per ADR-010 Decision 7: T0858→TA0104, T0816→TA0107, T1693.001→TA0107.
// This is the executable correctness gate that was missing from the initial
// STORY-133 test suite.
// ---------------------------------------------------------------------------
#[test]
fn test_ics_techniques_resolve_authoritative_tactic_ids() {
    // Authoritative ICS technique → tactic-id table (MITRE ATT&CK for ICS v19).
    // Source: f5-ics-technique-tactic-authoritative.md (verified against
    // attack.mitre.org technique pages via WebFetch during F5 research pass).
    //
    // ICS Discovery (TA0102):
    //   T0888 Remote System Information Discovery → TA0102
    //   T0846 Remote System Discovery             → TA0102
    // ICS Command and Control (TA0101):
    //   T0885 Commonly Used Port                  → TA0101
    // ICS Collection (TA0100):
    //   T0830 Adversary-in-the-Middle             → TA0100
    // ICS Impact (TA0105):
    //   T0831 Manipulation of Control             → TA0105  (F5 fix: was TA0106)
    //   T0827 Loss of Control                     → TA0105
    // ICS Impair Process Control (TA0106):
    //   T0836 Modify Parameter                    → TA0106
    //   T0806 Brute Force I/O                     → TA0106
    //   T0835 Manipulate I/O Image                → TA0106
    //   T1692.001 Unauthorized Message: Command   → TA0106
    // ICS Inhibit Response Function (TA0107):
    //   T0814 Denial of Service                   → TA0107
    //   T1691.001 Block OT Message: Command       → TA0107
    let authoritative: &[(&str, &str)] = &[
        // ICS Discovery — TA0102
        ("T0888", "TA0102"),
        ("T0846", "TA0102"),
        // ICS Command and Control — TA0101
        ("T0885", "TA0101"),
        // ICS Collection — TA0100 (F5 remap: T0830 moved from LateralMovement/TA0008)
        ("T0830", "TA0100"),
        // ICS Impact — TA0105
        ("T0831", "TA0105"), // F5 remap: moved from ImpairProcessControl/TA0106
        ("T0827", "TA0105"),
        // ICS Impair Process Control — TA0106
        ("T0836", "TA0106"),
        ("T0806", "TA0106"),
        ("T0835", "TA0106"),
        ("T1692.001", "TA0106"),
        // ICS Inhibit Response Function — TA0107
        ("T0814", "TA0107"),
        ("T1691.001", "TA0107"),
        // STORY-133 (F-133-004) — 3 new ENIP ICS IDs per ADR-010 Decision 7 (ics-attack-19.1)
        ("T0816", "TA0107"), // Device Restart/Shutdown → IcsInhibitResponseFunction
        ("T1693.001", "TA0107"), // Modify Firmware: System Firmware → IcsInhibitResponseFunction (staged)
        // ICS Execution — TA0104 (new variant added STORY-133)
        ("T0858", "TA0104"), // Change Operating Mode → IcsExecution
    ];

    for (id, expected_ta_id) in authoritative {
        assert_eq!(
            technique_tactic_id(id),
            Some(*expected_ta_id),
            "technique_tactic_id({id:?}) must return {expected_ta_id:?} \
             per MITRE ATT&CK for ICS v19 authoritative table \
             (f5-ics-technique-tactic-authoritative.md / ADR-010 Decision 7)"
        );
    }
}
