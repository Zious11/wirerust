use wirerust::mitre::{MitreTactic, all_tactics_in_report_order};

#[test]
fn display_renders_enterprise_tactics_with_canonical_spacing() {
    assert_eq!(MitreTactic::CommandAndControl.to_string(), "Command and Control");
    assert_eq!(MitreTactic::DefenseEvasion.to_string(), "Defense Evasion");
    assert_eq!(MitreTactic::CredentialAccess.to_string(), "Credential Access");
    assert_eq!(MitreTactic::LateralMovement.to_string(), "Lateral Movement");
    assert_eq!(MitreTactic::PrivilegeEscalation.to_string(), "Privilege Escalation");
    assert_eq!(MitreTactic::InitialAccess.to_string(), "Initial Access");
    assert_eq!(MitreTactic::ResourceDevelopment.to_string(), "Resource Development");
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
    assert_eq!(
        tactics.last(),
        Some(&MitreTactic::IcsImpairProcessControl)
    );
}

#[test]
fn report_order_contains_every_variant_exactly_once() {
    let tactics = all_tactics_in_report_order();
    let mut seen: Vec<String> = tactics.iter().map(|t| format!("{t:?}")).collect();
    seen.sort();
    let before = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), before, "duplicate variant in report order");
    assert_eq!(before, 16, "expected 14 Enterprise + 2 ICS-unique = 16 variants");
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
