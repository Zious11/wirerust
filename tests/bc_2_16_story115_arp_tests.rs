//! STORY-115 integration and CLI test suite (Red Gate — tests written before D3 implementation).
//!
//! Exercises:
//!   AC-011 — --arp-storm-rate CLI flag parsed correctly and defaults to 50
//!   AC-012 — --arp-storm-rate accepted without --arp (no parse error)
//!   AC-015 — integration test: analyze --arp --arp-storm-rate 10 produces D3 storm finding
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped per-story mod.
//! DF-AC-TEST-NAME-SYNC-001: function names match the Story Test Plan exactly.
//! DF-CANONICAL-FRAME-HOLDOUT-001: RFC 826 ARP frame values (op 1, htype=0x0001, etc.)
//!   apply to synthesized pcap bytes in the integration test.

#![allow(non_snake_case)]

// ---------------------------------------------------------------------------
// AC-011 — BC-2.16.013 PC1/2: --arp-storm-rate CLI flag parsed and defaulted
// ---------------------------------------------------------------------------

mod story_115_cli {
    use clap::Parser;
    use wirerust::cli::{Cli, Commands};

    /// Verifies that --arp-storm-rate 10 is parsed correctly from the CLI surface,
    /// producing arp_storm_rate=10 in the parsed Commands::Analyze variant
    /// (BC-2.16.013 PC1; src/cli.rs `#[arg(long, default_value_t = 50)] arp_storm_rate: u32`).
    ///
    /// Turns GREEN when the flag is declared in src/cli.rs and consumed in src/main.rs.
    #[test]
    fn test_cli_arp_storm_rate_parsed() {
        // AC-011: parse --arp-storm-rate 10 and assert the value is 10
        let cli = Cli::try_parse_from([
            "wirerust",
            "analyze",
            "test.pcap",
            "--arp",
            "--arp-storm-rate",
            "10",
        ])
        .expect(
            "AC-011 / BC-2.16.013 PC1: CLI must accept --arp-storm-rate 10 without error. \
             Turns GREEN when arp_storm_rate flag is declared in src/cli.rs.",
        );

        match cli.command {
            Commands::Analyze { arp_storm_rate, .. } => {
                assert_eq!(
                    arp_storm_rate, 10,
                    "AC-011 / BC-2.16.013 PC1: --arp-storm-rate 10 must parse to \
                     arp_storm_rate=10. Got {}.",
                    arp_storm_rate
                );
            }
            other => panic!(
                "AC-011: expected Commands::Analyze but got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    /// Verifies that when --arp-storm-rate is absent, the default value is 50
    /// (BC-2.16.013 PC2; `default_value_t = 50`; ARP_STORM_RATE_DEFAULT).
    ///
    /// Turns GREEN when the flag is declared with default_value_t = 50 in src/cli.rs.
    #[test]
    fn test_cli_arp_storm_rate_default_50() {
        // AC-011: omit --arp-storm-rate; default must be 50
        let cli = Cli::try_parse_from(["wirerust", "analyze", "test.pcap", "--arp"]).expect(
            "AC-011 / BC-2.16.013 PC2: CLI must accept --arp without --arp-storm-rate. \
                 Default of 50 must apply.",
        );

        match cli.command {
            Commands::Analyze { arp_storm_rate, .. } => {
                assert_eq!(
                    arp_storm_rate, 50,
                    "AC-011 / BC-2.16.013 PC2: when --arp-storm-rate is absent, default must \
                     be 50 (ARP_STORM_RATE_DEFAULT). Got {}.",
                    arp_storm_rate
                );
            }
            other => panic!(
                "AC-011: expected Commands::Analyze but got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    // ---------------------------------------------------------------------------
    // AC-012 — BC-2.16.013 EC-006: flag accepted without --arp
    // ---------------------------------------------------------------------------

    /// Verifies that --arp-storm-rate N is accepted by the CLI without --arp present
    /// (no parse error), because the flag is an independent option on Commands::Analyze
    /// (BC-2.16.013 EC-006: flag accepted, has no effect when --arp absent).
    ///
    /// This test verifies CLI-level acceptance only; behavioral no-op with --arp absent
    /// is enforced by the --arp gate in main.rs (which never calls process_arp).
    ///
    /// Turns GREEN when arp_storm_rate is declared without requires("arp") in src/cli.rs.
    #[test]
    fn test_storm_rate_flag_accepted_without_arp_flag() {
        // AC-012: --arp-storm-rate without --arp must not produce a parse error
        let result =
            Cli::try_parse_from(["wirerust", "analyze", "test.pcap", "--arp-storm-rate", "25"]);

        assert!(
            result.is_ok(),
            "AC-012 / BC-2.16.013 EC-006: --arp-storm-rate 25 must be accepted by the CLI \
             even when --arp is absent (no parse error). Got: {:?}. \
             Turns GREEN when arp_storm_rate has no requires() constraint.",
            result.err()
        );

        if let Ok(cli) = result {
            match cli.command {
                Commands::Analyze { arp_storm_rate, .. } => {
                    assert_eq!(
                        arp_storm_rate, 25,
                        "AC-012 / BC-2.16.013 EC-006: --arp-storm-rate 25 without --arp \
                         must parse arp_storm_rate=25. Got {}.",
                        arp_storm_rate
                    );
                }
                other => panic!(
                    "AC-012: expected Commands::Analyze but got {:?}",
                    std::mem::discriminant(&other)
                ),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AC-015 — BC-2.16.008: integration test — full CLI pipeline with D3 storm finding
// ---------------------------------------------------------------------------

mod story_115_integration {
    use assert_cmd::Command;
    use serde_json::Value;
    use std::io::Write;

    /// Build a minimal valid Ethernet + ARP frame in raw bytes (RFC 826).
    ///
    /// Ethernet header (14 bytes): dst_mac, src_mac, EtherType 0x0806 (ARP).
    /// ARP payload (28 bytes): htype=0x0001 (Ethernet), ptype=0x0800 (IPv4),
    ///   hlen=6, plen=4, op=1 (Request), sender_hw (6), sender_ip (4),
    ///   target_hw (6), target_ip (4).
    ///
    /// DF-CANONICAL-FRAME-HOLDOUT-001: op=1 (Request), htype=0x0001, ptype=0x0800,
    ///   hlen=6, plen=4.
    fn build_arp_packet_bytes(
        src_mac: [u8; 6],
        sender_ip: [u8; 4],
        target_ip: [u8; 4],
    ) -> [u8; 42] {
        let mut pkt = [0u8; 42];
        // Ethernet dst (broadcast)
        pkt[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        // Ethernet src
        pkt[6..12].copy_from_slice(&src_mac);
        // EtherType ARP = 0x0806
        pkt[12] = 0x08;
        pkt[13] = 0x06;
        // ARP htype = 0x0001 (Ethernet)
        pkt[14] = 0x00;
        pkt[15] = 0x01;
        // ARP ptype = 0x0800 (IPv4)
        pkt[16] = 0x08;
        pkt[17] = 0x00;
        // ARP hlen = 6, plen = 4
        pkt[18] = 6;
        pkt[19] = 4;
        // ARP op = 1 (Request)
        pkt[20] = 0x00;
        pkt[21] = 0x01;
        // Sender HW addr
        pkt[22..28].copy_from_slice(&src_mac);
        // Sender protocol addr
        pkt[28..32].copy_from_slice(&sender_ip);
        // Target HW addr (zero for request)
        pkt[32..38].copy_from_slice(&[0u8; 6]);
        // Target protocol addr
        pkt[38..42].copy_from_slice(&target_ip);
        pkt
    }

    /// Write a minimal libpcap file (global header + N identical ARP packet records)
    /// to a temporary path, with all records sharing `timestamp_secs`.
    ///
    /// pcap global header (24 bytes): magic_number=0xa1b2c3d4 (native endian),
    /// version_major=2, version_minor=4, thiszone=0, sigfigs=0, snaplen=65535,
    /// network=1 (LINKTYPE_ETHERNET).
    fn write_pcap<W: Write>(
        writer: &mut W,
        pkt_bytes: &[u8],
        count: u32,
        timestamp_secs: u32,
    ) -> std::io::Result<()> {
        // Global header (little-endian)
        writer.write_all(&0xa1b2c3d4u32.to_le_bytes())?; // magic
        writer.write_all(&2u16.to_le_bytes())?; // version_major
        writer.write_all(&4u16.to_le_bytes())?; // version_minor
        writer.write_all(&0i32.to_le_bytes())?; // thiszone
        writer.write_all(&0u32.to_le_bytes())?; // sigfigs
        writer.write_all(&65535u32.to_le_bytes())?; // snaplen
        writer.write_all(&1u32.to_le_bytes())?; // network (LINKTYPE_ETHERNET)

        let incl_len = pkt_bytes.len() as u32;
        let orig_len = pkt_bytes.len() as u32;

        for _ in 0..count {
            // Packet record header
            writer.write_all(&timestamp_secs.to_le_bytes())?; // ts_sec
            writer.write_all(&0u32.to_le_bytes())?; // ts_usec
            writer.write_all(&incl_len.to_le_bytes())?;
            writer.write_all(&orig_len.to_le_bytes())?;
            // Packet data
            writer.write_all(pkt_bytes)?;
        }
        Ok(())
    }

    /// Verifies the full CLI pipeline: `wirerust analyze --arp --arp-storm-rate 10 <pcap>`
    /// produces a D3 storm finding with confidence=MEDIUM and mitre_techniques=[]
    /// (BC-2.16.008, BC-2.16.013; AC-015).
    ///
    /// The synthetic pcap contains 10 identical ARP Request frames from one source MAC
    /// (AA:BB:CC:DD:EE:FF) all at timestamp_secs=100. With storm_rate=10:
    /// rate = 10 / max(1, 100-100) = 10/1 = 10 >= 10 → D3 storm finding emitted.
    ///
    /// Turns GREEN when D3 storm detection is wired into the full analyze pipeline
    /// and --arp-storm-rate is consumed by main.rs.
    #[test]
    fn test_integration_arp_storm_end_to_end() {
        // Build synthetic pcap: 10 ARP Request frames from STORM_MAC at ts=100
        let storm_mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let sender_ip: [u8; 4] = [10, 0, 0, 1];
        let target_ip: [u8; 4] = [10, 0, 0, 2];
        let pkt = build_arp_packet_bytes(storm_mac, sender_ip, target_ip);

        let tmp = tempfile::tempdir().expect("tempdir");
        let pcap_path = tmp.path().join("storm_test.pcap");
        {
            let mut f = std::fs::File::create(&pcap_path).expect("create pcap");
            write_pcap(&mut f, &pkt, 10, 100).expect("write pcap");
        }

        // Run: wirerust analyze --arp --arp-storm-rate 10 <pcap> --output-format json
        let cmd_output = Command::cargo_bin("wirerust")
            .expect(
                "wirerust binary must be built — run `cargo build` first. \
                 AC-015 integration test requires the compiled binary.",
            )
            .args([
                "analyze",
                pcap_path.to_str().expect("pcap path"),
                "--arp",
                "--arp-storm-rate",
                "10",
                "--output-format",
                "json",
            ])
            .output()
            .expect("AC-015: wirerust command must run without OS error");

        assert!(
            cmd_output.status.success(),
            "AC-015 / BC-2.16.008: wirerust analyze must exit 0. \
             stderr: {}",
            String::from_utf8_lossy(&cmd_output.stderr)
        );

        let output_str = String::from_utf8_lossy(&cmd_output.stdout).into_owned();

        // Parse JSON and look for a D3 storm finding
        let parsed: Value = serde_json::from_str(&output_str).unwrap_or_else(|e| {
            panic!(
                "AC-015 / BC-2.16.008: wirerust output must be valid JSON. \
                 Parse error: {e}. Output: {output_str}"
            )
        });

        // Findings are in the "findings" array of the JSON output
        let findings = parsed
            .get("findings")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| {
                panic!(
                    "AC-015 / BC-2.16.008: JSON output must have a 'findings' array. \
                     Got: {output_str}"
                )
            });

        let d3_finding = findings.iter().find(|f| {
            let summary = f
                .get("summary")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_lowercase();
            let confidence = f
                .get("confidence")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_lowercase();
            confidence == "medium" && (summary.contains("storm") || summary.contains("d3"))
        });

        let f = d3_finding.unwrap_or_else(|| {
            panic!(
                "AC-015 / BC-2.16.008: a D3 storm finding with confidence=MEDIUM must appear \
                 in the JSON output after processing 10 ARP frames at ts=100 with \
                 --arp-storm-rate 10 (rate=10/1=10 >= 10). Got findings: {findings:?}. \
                 Turns GREEN when detect_storm is wired into the analyze pipeline."
            )
        });

        // Verify confidence == MEDIUM
        assert_eq!(
            f.get("confidence").and_then(|c| c.as_str()),
            Some("MEDIUM"),
            "AC-015 / BC-2.16.008: D3 storm finding must have confidence=MEDIUM. Got: {:?}",
            f.get("confidence")
        );

        // Verify mitre_techniques is absent or empty (DF-VALIDATION-001)
        let mitre = f.get("mitre_techniques");
        let mitre_empty = mitre
            .map(|m| m.as_array().map(|a| a.is_empty()).unwrap_or(true))
            .unwrap_or(true);
        assert!(
            mitre_empty,
            "AC-015 / BC-2.16.008 Invariant 3 / DF-VALIDATION-001: D3 storm finding must have \
             mitre_techniques=[] (empty; T0814 withheld). Got mitre_techniques: {:?}",
            mitre
        );

        // Verify T0814 is not present
        if let Some(techs) = mitre.and_then(|m| m.as_array()) {
            let has_t0814 = techs
                .iter()
                .any(|t| t.as_str().map(|s| s == "T0814").unwrap_or(false));
            assert!(
                !has_t0814,
                "AC-015 / BC-2.16.008 Invariant 3 / DF-VALIDATION-001: T0814 must NOT \
                 be present in any D3 storm finding's mitre_techniques. Got: {:?}",
                techs
            );
        }
    }
}
