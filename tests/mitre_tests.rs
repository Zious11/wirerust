use wirerust::mitre::{MitreTactic, all_tactics_in_report_order, technique_name, technique_tactic};

#[test]
fn display_renders_enterprise_tactics_with_canonical_spacing() {
    assert_eq!(
        MitreTactic::CommandAndControl.to_string(),
        "Command and Control"
    );
    assert_eq!(MitreTactic::DefenseEvasion.to_string(), "Defense Evasion");
    assert_eq!(
        MitreTactic::CredentialAccess.to_string(),
        "Credential Access"
    );
    assert_eq!(MitreTactic::LateralMovement.to_string(), "Lateral Movement");
    assert_eq!(
        MitreTactic::PrivilegeEscalation.to_string(),
        "Privilege Escalation"
    );
    assert_eq!(MitreTactic::InitialAccess.to_string(), "Initial Access");
    assert_eq!(
        MitreTactic::ResourceDevelopment.to_string(),
        "Resource Development"
    );
    assert_eq!(MitreTactic::Reconnaissance.to_string(), "Reconnaissance");
    assert_eq!(MitreTactic::Execution.to_string(), "Execution");
    assert_eq!(MitreTactic::Persistence.to_string(), "Persistence");
    assert_eq!(MitreTactic::Discovery.to_string(), "Discovery");
    assert_eq!(MitreTactic::Collection.to_string(), "Collection");
    assert_eq!(MitreTactic::Exfiltration.to_string(), "Exfiltration");
    assert_eq!(MitreTactic::Impact.to_string(), "Impact");
}

#[test]
fn display_renders_ics_tactics_unprefixed() {
    assert_eq!(
        MitreTactic::IcsInhibitResponseFunction.to_string(),
        "Inhibit Response Function"
    );
    assert_eq!(
        MitreTactic::IcsImpairProcessControl.to_string(),
        "Impair Process Control"
    );
}

#[test]
fn report_order_starts_with_reconnaissance_and_ends_with_ics() {
    let tactics = all_tactics_in_report_order();
    assert_eq!(tactics.first(), Some(&MitreTactic::Reconnaissance));
    assert_eq!(tactics.last(), Some(&MitreTactic::IcsImpairProcessControl));
}

#[test]
fn report_order_contains_every_variant_exactly_once() {
    use std::collections::HashSet;
    let tactics = all_tactics_in_report_order();
    // HashSet on MitreTactic uses the derived Eq + Hash — robust against
    // any future change to the Debug impl.
    let unique: HashSet<MitreTactic> = tactics.iter().copied().collect();
    assert_eq!(
        unique.len(),
        tactics.len(),
        "duplicate variant in report order"
    );
    assert_eq!(
        tactics.len(),
        16,
        "expected 14 Enterprise + 2 ICS-unique = 16 variants"
    );
}

#[test]
fn report_order_matches_enterprise_kill_chain_for_first_14() {
    let tactics = all_tactics_in_report_order();
    let enterprise = [
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
    assert_eq!(&tactics[..14], &enterprise);
}

#[test]
fn technique_name_resolves_every_seeded_id() {
    assert_eq!(
        technique_name("T1027"),
        Some("Obfuscated Files or Information")
    );
    assert_eq!(technique_name("T1036"), Some("Masquerading"));
    assert_eq!(technique_name("T1040"), Some("Network Sniffing"));
    assert_eq!(technique_name("T1046"), Some("Network Service Discovery"));
    assert_eq!(technique_name("T1071"), Some("Application Layer Protocol"));
    assert_eq!(technique_name("T1071.001"), Some("Web Protocols"));
    assert_eq!(technique_name("T1071.004"), Some("DNS"));
    assert_eq!(
        technique_name("T1083"),
        Some("File and Directory Discovery")
    );
    assert_eq!(
        technique_name("T1499.002"),
        Some("Service Exhaustion Flood")
    );
    assert_eq!(technique_name("T1505.003"), Some("Web Shell"));
    assert_eq!(technique_name("T1573"), Some("Encrypted Channel"));
    assert_eq!(technique_name("T0846"), Some("Remote System Discovery"));
    assert_eq!(
        technique_name("T0855"),
        Some("Unauthorized Command Message")
    );
    assert_eq!(technique_name("T0856"), Some("Spoof Reporting Message"));
    assert_eq!(technique_name("T0885"), Some("Commonly Used Port"));
}

#[test]
fn technique_name_returns_none_for_unknown_ids() {
    assert_eq!(technique_name("T9999"), None);
    assert_eq!(technique_name(""), None);
    assert_eq!(technique_name("T1046.999"), None);
    assert_eq!(technique_name("garbage"), None);
}

#[test]
fn technique_tactic_matches_spec_table() {
    assert_eq!(technique_tactic("T1027"), Some(MitreTactic::DefenseEvasion));
    assert_eq!(technique_tactic("T1036"), Some(MitreTactic::DefenseEvasion));
    assert_eq!(
        technique_tactic("T1040"),
        Some(MitreTactic::CredentialAccess)
    );
    assert_eq!(technique_tactic("T1046"), Some(MitreTactic::Discovery));
    assert_eq!(
        technique_tactic("T1071"),
        Some(MitreTactic::CommandAndControl)
    );
    assert_eq!(
        technique_tactic("T1071.001"),
        Some(MitreTactic::CommandAndControl)
    );
    assert_eq!(
        technique_tactic("T1071.004"),
        Some(MitreTactic::CommandAndControl)
    );
    assert_eq!(technique_tactic("T1083"), Some(MitreTactic::Discovery));
    assert_eq!(technique_tactic("T1499.002"), Some(MitreTactic::Impact));
    assert_eq!(
        technique_tactic("T1505.003"),
        Some(MitreTactic::Persistence)
    );
    assert_eq!(
        technique_tactic("T1573"),
        Some(MitreTactic::CommandAndControl)
    );
    assert_eq!(technique_tactic("T0846"), Some(MitreTactic::Discovery));
    assert_eq!(
        technique_tactic("T0855"),
        Some(MitreTactic::IcsImpairProcessControl)
    );
    assert_eq!(
        technique_tactic("T0856"),
        Some(MitreTactic::IcsImpairProcessControl)
    );
    assert_eq!(
        technique_tactic("T0885"),
        Some(MitreTactic::CommandAndControl)
    );
}

#[test]
fn technique_tactic_returns_none_for_unknown_ids() {
    assert_eq!(technique_tactic("T9999"), None);
    assert_eq!(technique_tactic(""), None);
}

#[test]
fn known_emitted_technique_ids_resolve_in_lookup() {
    // Sanity check on a hand-curated list of the technique IDs the codebase
    // emits today via `mitre_technique: Some("...")`. This is not an
    // exhaustive cross-check — adding a new emission site without also
    // adding the ID here will not fail this test. See issue #67 for the
    // tracked discussion of the trade-off (the hand-curated approach is
    // the idiomatic Rust pattern at this scale; revisit when emission
    // sites grow > ~20 or a missed-update incident occurs). The
    // convention is to update this list in the same commit as a new
    // emission.
    let emitted_ids = [
        // src/analyzer/http.rs
        "T1083",
        "T1505.003",
        "T1046",
        "T1499.002",
        // src/analyzer/tls.rs (added in this feature)
        "T1027",
        // src/reassembly/mod.rs
        "T1036",
    ];

    for id in emitted_ids {
        assert!(
            technique_name(id).is_some(),
            "analyzer emits {id} but technique_name({id}) returned None",
        );
        assert!(
            technique_tactic(id).is_some(),
            "analyzer emits {id} but technique_tactic({id}) returned None",
        );
    }
}
