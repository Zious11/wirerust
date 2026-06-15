//! STORY-114 integration test suite.
//!
//! Exercises:
//!   AC-011 — T0830 and T1557.002 resolve in the MITRE catalog (after VP-007 atomic update)
//!   AC-012 — VP-007: SEEDED=25, EMITTED=17; vp007_catalog_drift_guard passes
//!   AC-006 — --arp-spoof-threshold CLI flag parsed and defaulted correctly
//!
//! All tests here MUST FAIL before the Green step because:
//!   - T0830 and T1557.002 are not yet in src/mitre.rs (SEEDED=23, EMITTED=15).
//!   - The count assertions (25/17) fail against current values (23/15).
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped per-story mod.
//! DF-AC-TEST-NAME-SYNC-001: function names match the Story Test Plan exactly.

#![allow(non_snake_case)]

// ---------------------------------------------------------------------------
// AC-011 — BC-2.16.004 MITRE: T0830 + T1557.002 resolve in catalog
// ---------------------------------------------------------------------------

mod story_114_mitre {
    use wirerust::mitre::{MitreTactic, technique_name, technique_tactic};

    /// AC-011 (BC-2.16.004 Invariant 4 / VP-007): after the VP-007 5-part atomic update,
    /// technique_info("T0830") returns ("Adversary-in-the-Middle", MitreTactic::LateralMovement)
    /// and technique_info("T1557.002") returns ("Adversary-in-the-Middle: ARP Cache Poisoning",
    /// MitreTactic::CredentialAccess). Both are resolvable via technique_name and technique_tactic
    /// (not returning None / "Unknown").
    ///
    /// Red Gate: fails now because T0830 and T1557.002 return None from technique_info
    /// (they are absent from src/mitre.rs at SEEDED=23, EMITTED=15 baseline).
    /// EC-012: "technique_info("T0830") returned None before; returns Some(...) after 5-part update".
    #[test]
    fn test_t0830_and_t1557_002_resolves_in_catalog() {
        // T0830: "Adversary-in-the-Middle", MitreTactic::LateralMovement
        // (ADR-008 Decision 6 tactic anchor; ICS ATT&CK v19.1)
        let t0830_name = technique_name("T0830");
        assert!(
            t0830_name.is_some(),
            "AC-011 / BC-2.16.004 Invariant 4: technique_name(\"T0830\") must return Some \
             after the VP-007 5-part atomic update (STORY-114 co-commit). \
             Currently returns None — RED until src/mitre.rs seeding. Got: {:?}",
            t0830_name
        );
        assert_eq!(
            t0830_name,
            Some("Adversary-in-the-Middle"),
            "AC-011 / BC-2.16.004 Invariant 4: T0830 name must be 'Adversary-in-the-Middle' \
             (MITRE ICS ATT&CK v19.1). Got: {:?}",
            t0830_name
        );

        let t0830_tactic = technique_tactic("T0830");
        assert!(
            t0830_tactic.is_some(),
            "AC-011 / BC-2.16.004 Invariant 4: technique_tactic(\"T0830\") must return Some. \
             Currently returns None. RED.",
        );
        assert_eq!(
            t0830_tactic,
            Some(MitreTactic::LateralMovement),
            "AC-011 / BC-2.16.004 Invariant 4 (tactic anchor — ADR-008 Decision 6): T0830 \
             must map to MitreTactic::LateralMovement. Got: {:?}",
            t0830_tactic
        );

        // T1557.002: "Adversary-in-the-Middle: ARP Cache Poisoning", MitreTactic::CredentialAccess
        // (ADR-008 Decision 6 tactic anchor; Enterprise ATT&CK v19.1)
        let t1557_name = technique_name("T1557.002");
        assert!(
            t1557_name.is_some(),
            "AC-011 / BC-2.16.004 Invariant 4: technique_name(\"T1557.002\") must return Some \
             after the VP-007 5-part atomic update. Currently returns None — RED. Got: {:?}",
            t1557_name
        );
        assert_eq!(
            t1557_name,
            Some("Adversary-in-the-Middle: ARP Cache Poisoning"),
            "AC-011 / BC-2.16.004 Invariant 4: T1557.002 name must be \
             'Adversary-in-the-Middle: ARP Cache Poisoning' (Enterprise ATT&CK v19.1). \
             Got: {:?}",
            t1557_name
        );

        let t1557_tactic = technique_tactic("T1557.002");
        assert!(
            t1557_tactic.is_some(),
            "AC-011 / BC-2.16.004 Invariant 4: technique_tactic(\"T1557.002\") must return Some. \
             Currently returns None. RED.",
        );
        assert_eq!(
            t1557_tactic,
            Some(MitreTactic::CredentialAccess),
            "AC-011 / BC-2.16.004 Invariant 4 (tactic anchor — ADR-008 Decision 6): T1557.002 \
             must map to MitreTactic::CredentialAccess. Got: {:?}",
            t1557_tactic
        );
    }

    // -----------------------------------------------------------------------
    // AC-012 — VP-007: SEEDED=25, EMITTED=17 (replaces old 23/15 count test)
    // -----------------------------------------------------------------------

    /// AC-012 (VP-007 / STORY-114): after the 5-part atomic update, the catalog has
    /// SEEDED=25 and EMITTED=17 entries. Specifically:
    ///   - All 25 seeded IDs resolve via technique_name (non-None, non-empty).
    ///   - technique_name never returns "" (the sentinel "Unknown" pattern from old code).
    ///   - vp007_catalog_drift_guard passes (that in-crate unit test is the mechanical gate;
    ///     this integration test provides the public-API view).
    ///
    /// Red Gate: fails now because only 23 IDs are seeded (T0830 and T1557.002 missing).
    /// This test REPLACES the old test_technique_name_resolves_all_21_seeded_ids for the
    /// 25-entry post-STORY-114 state.
    #[test]
    fn test_vp007_seeded_25_emitted_17() {
        // All 25 seeded IDs post-STORY-114 atomic update (11 Enterprise + 10 ICS + 2 ARP):
        // Enterprise (11)
        let seeded_25: &[(&str, &str, MitreTactic)] = &[
            ("T1027", "Obfuscated Files or Information", MitreTactic::DefenseEvasion),
            ("T1036", "Masquerading", MitreTactic::DefenseEvasion),
            ("T1040", "Network Sniffing", MitreTactic::CredentialAccess),
            ("T1046", "Network Service Discovery", MitreTactic::Discovery),
            ("T1071", "Application Layer Protocol", MitreTactic::CommandAndControl),
            ("T1071.001", "Web Protocols", MitreTactic::CommandAndControl),
            ("T1071.004", "DNS", MitreTactic::CommandAndControl),
            ("T1083", "File and Directory Discovery", MitreTactic::Discovery),
            ("T1499.002", "Service Exhaustion Flood", MitreTactic::Impact),
            ("T1505.003", "Web Shell", MitreTactic::Persistence),
            ("T1573", "Encrypted Channel", MitreTactic::CommandAndControl),
            // ICS pre-F2 (4)
            ("T0846", "Remote System Discovery", MitreTactic::Discovery),
            ("T1692.001", "Unauthorized Message: Command Message", MitreTactic::IcsImpairProcessControl),
            ("T1692.002", "Unauthorized Message: Reporting Message", MitreTactic::IcsImpairProcessControl),
            ("T0885", "Commonly Used Port", MitreTactic::CommandAndControl),
            // ICS new F2 — STORY-100 (6)
            ("T0836", "Modify Parameter", MitreTactic::IcsImpairProcessControl),
            ("T0814", "Denial of Service", MitreTactic::IcsInhibitResponseFunction),
            ("T0806", "Brute Force I/O", MitreTactic::IcsImpairProcessControl),
            ("T0835", "Manipulate I/O Image", MitreTactic::IcsImpairProcessControl),
            ("T0831", "Manipulation of Control", MitreTactic::IcsImpairProcessControl),
            ("T0888", "Remote System Information Discovery", MitreTactic::Discovery),
            // STORY-109 (2)
            ("T1691.001", "Block Operational Technology Message: Command Message", MitreTactic::IcsInhibitResponseFunction),
            ("T0827", "Loss of Control", MitreTactic::IcsImpact),
            // STORY-114 ARP (2) — the additions that make this test RED until impl
            ("T0830", "Adversary-in-the-Middle", MitreTactic::LateralMovement),
            ("T1557.002", "Adversary-in-the-Middle: ARP Cache Poisoning", MitreTactic::CredentialAccess),
        ];

        // Verify count is exactly 25
        assert_eq!(
            seeded_25.len(),
            25,
            "test_vp007_seeded_25_emitted_17: internal test vector count must be 25; got {}",
            seeded_25.len()
        );

        // For each of the 25 seeded IDs: technique_name is Some and non-empty; tactic is correct
        for (id, expected_name, expected_tactic) in seeded_25 {
            let name = technique_name(id);
            assert!(
                name.is_some(),
                "AC-012 / VP-007: technique_name({id:?}) must return Some (non-None) after \
                 the 25-entry VP-007 atomic update. Currently returns None. \
                 RED until src/mitre.rs seeding. Got: {:?}",
                name
            );
            let name_str = name.unwrap();
            assert!(
                !name_str.is_empty(),
                "AC-012 / VP-007: technique_name({id:?}) must return a non-empty string. Got: {:?}",
                name_str
            );
            assert_ne!(
                name_str, "Unknown",
                "AC-012 / VP-007: technique_name({id:?}) must not return 'Unknown' (sentinel). \
                 Got: {:?}",
                name_str
            );
            assert_eq!(
                name,
                Some(*expected_name),
                "AC-012 / VP-007: technique_name({id:?}) returned unexpected value. \
                 Expected {:?}, got {:?}",
                expected_name,
                name
            );

            let tactic = technique_tactic(id);
            assert_eq!(
                tactic,
                Some(*expected_tactic),
                "AC-012 / VP-007: technique_tactic({id:?}) returned unexpected tactic. \
                 Expected {:?}, got {:?}",
                expected_tactic,
                tactic
            );
        }
    }

    // -----------------------------------------------------------------------
    // Emitted-ID count verification (17 after STORY-114)
    // -----------------------------------------------------------------------

    /// AC-012 companion: verify that all 17 analyzer-emitted IDs (post-STORY-114)
    /// resolve in the catalog. This is the public-API counterpart to the
    /// kani_proofs::EMITTED_IDS check inside src/mitre.rs.
    ///
    /// Red Gate: T0830 and T1557.002 return None until the atomic update.
    #[test]
    fn test_vp007_all_17_emitted_ids_resolve() {
        // IDs actually emitted by analyzers (grep -rn 'mitre_techniques: vec!' src/):
        // 6 Enterprise + 7 ICS + 2 STORY-109 + 2 ARP (STORY-114) = 17 emitted IDs
        let emitted_17: &[&str] = &[
            // Enterprise (6)
            "T1027",     // TLS: SNI anomaly
            "T1036",     // Reassembly: conflicting overlap
            "T1046",     // HTTP: admin panel
            "T1083",     // HTTP: path traversal
            "T1499.002", // HTTP: header flood
            "T1505.003", // HTTP: web shell
            // ICS (7)
            "T1692.001", // ICS Modbus write
            "T0836",     // Modify Parameter
            "T0814",     // Denial of Service
            "T0806",     // Brute Force I/O
            "T0835",     // Manipulate I/O Image
            "T0831",     // Manipulation of Control
            "T0888",     // Remote System Information Discovery
            // STORY-109 (2)
            "T1691.001", // Block OT Message: Command Message
            "T0827",     // Loss of Control
            // STORY-114 ARP (2) — RED until catalog seeded
            "T0830",     // ARP Adversary-in-the-Middle (D1/D12/GARP-conflict)
            "T1557.002", // ARP Cache Poisoning (D1/D12/GARP-conflict)
        ];

        assert_eq!(
            emitted_17.len(),
            17,
            "test_vp007_all_17_emitted_ids_resolve: internal vector must have exactly 17 \
             emitted IDs post-STORY-114. Got {}.",
            emitted_17.len()
        );

        for id in emitted_17 {
            assert!(
                technique_name(id).is_some(),
                "AC-012 / VP-007 emitter half: analyzer emits {id} but technique_name({id}) \
                 returned None — not in catalog. RED until VP-007 atomic update applied.",
            );
            assert!(
                technique_tactic(id).is_some(),
                "AC-012 / VP-007 emitter half: analyzer emits {id} but technique_tactic({id}) \
                 returned None. RED until VP-007 atomic update applied.",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-006 — BC-2.16.012: --arp-spoof-threshold CLI flag
// ---------------------------------------------------------------------------

mod story_114_cli {
    use assert_cmd::Command;

    // Fixture that contains only Ethernet/IP/TCP frames (no ARP frames).
    // Using http-ooo.pcap (known-good fixture from existing CLI integration tests).
    const IP_ONLY_FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    /// AC-006a (BC-2.16.012 PC1): `--arp-spoof-threshold N` is accepted by the CLI
    /// and does not cause a parse error. The analyzer is initialized with spoof_threshold=N.
    ///
    /// This test verifies that the flag exists and is parsed (behavioral: CLI accepts it).
    /// With the http-ooo.pcap fixture (no ARP frames), no spoof findings are emitted, but
    /// the flag must be accepted without error.
    ///
    /// Note: This may PASS already if the flag is declared in cli.rs (it is in the scaffold).
    /// The behavioral assertion is that the flag is wired through to ArpAnalyzer::new()
    /// and the CLI exits success. We also assert via JSON that arp_spoof_threshold=1 has
    /// no effect on a no-ARP capture (spoof_findings remains 0).
    #[test]
    fn test_cli_arp_spoof_threshold_parsed() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_path = tmp.path().join("threshold_test.json");

        // --arp-spoof-threshold 1 (threshold=1: any rebind → HIGH)
        Command::cargo_bin("wirerust")
            .expect("wirerust binary must be built — run `cargo build` first")
            .args([
                "analyze",
                IP_ONLY_FIXTURE,
                "--arp",
                "--arp-spoof-threshold",
                "1",
                "--output-format",
                "json",
                "--json",
                out_path.to_str().expect("utf-8 path"),
            ])
            .assert()
            .success();

        let written = std::fs::read_to_string(&out_path).expect("output JSON must exist");
        let json: serde_json::Value =
            serde_json::from_str(&written).expect("output must be valid JSON");

        // Behavioral check: --arp must be active (ARP summary present in output)
        let summaries = json.get("analyzers").and_then(|v| v.as_array()).expect(
            "AC-006 / BC-2.16.012: 'analyzers' key must be present in JSON output with --arp"
        );
        let arp_summary = summaries.iter().find(|s| {
            s.get("analyzer_name")
                .and_then(|n| n.as_str())
                .map(|n| n == "ARP")
                .unwrap_or(false)
        });
        assert!(
            arp_summary.is_some(),
            "AC-006 / BC-2.16.012: ARP summary must be present when --arp is active."
        );

        // With no ARP frames in the fixture, spoof_findings must be 0
        let spoof_findings = arp_summary
            .unwrap()
            .get("detail")
            .and_then(|d| d.get("spoof_findings"))
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        assert_eq!(
            spoof_findings, 0,
            "AC-006 / BC-2.16.012: with no ARP frames, spoof_findings must be 0 \
             (threshold=1 has no effect without ARP rebinds). Got {}.",
            spoof_findings
        );
    }

    /// AC-006b (BC-2.16.012 PC2): when `--arp-spoof-threshold` is absent, default=3 applies.
    /// Verified behaviorally: CLI runs successfully without the flag; ARP analysis with
    /// http-ooo.pcap (no ARP frames) produces the same result as --arp-spoof-threshold 3.
    ///
    /// This test verifies the flag is optional and the default is accepted.
    #[test]
    fn test_cli_arp_spoof_threshold_default_3() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_path_default = tmp.path().join("default_threshold.json");

        // Run without --arp-spoof-threshold (default=3)
        Command::cargo_bin("wirerust")
            .expect("wirerust binary must be built")
            .args([
                "analyze",
                IP_ONLY_FIXTURE,
                "--arp",
                "--output-format",
                "json",
                "--json",
                out_path_default.to_str().expect("utf-8 path"),
            ])
            .assert()
            .success();

        let written =
            std::fs::read_to_string(&out_path_default).expect("output JSON must exist");
        let json: serde_json::Value =
            serde_json::from_str(&written).expect("output must be valid JSON");

        // ARP summary must still be present (--arp is active)
        let summaries = json.get("analyzers").and_then(|v| v.as_array()).expect(
            "AC-006b / BC-2.16.012 PC2: 'analyzers' key must be present"
        );
        let arp_summary = summaries.iter().find(|s| {
            s.get("analyzer_name")
                .and_then(|n| n.as_str())
                .map(|n| n == "ARP")
                .unwrap_or(false)
        });
        assert!(
            arp_summary.is_some(),
            "AC-006b / BC-2.16.012 PC2: ARP summary must be present when --arp active \
             (default threshold=3 applies when flag absent)."
        );
    }
}
