//! D-074 / F-ARP-F4P1-001: ARP threshold-zero rejection guard tests.
//!
//! Guards that `--arp-storm-rate 0` and `--arp-spoof-threshold 0` are
//! rejected by the CLI with a fatal error (non-zero exit, stderr message
//! indicating the value must be >= 1) before any packet processing begins.
//!
//! Behavioral contracts covered:
//!   BC-2.16.008 EC-006  --arp-storm-rate must be >= 1
//!   BC-2.16.012 EC-004  --arp-spoof-threshold must be >= 1
//!   BC-2.16.013 EC-004  --arp-storm-rate must be >= 1 (D3 storm flag)
//!   Decision D-074       Fail-fast validation: value 0 rejected before any
//!                        packet processing, same as --modbus-write-burst-threshold
//!                        and --modbus-write-sustained-threshold (BC-2.14.024 P3a/P3b).
//!
//! These tests are analogous to `test_BC_2_14_024_burst_threshold_zero_rejected`
//! and `test_BC_2_14_024_sustained_threshold_zero_rejected` in
//! `bc_2_14_105_modbus_dispatch_tests.rs`.  Mirror that exact style:
//!   - assert_cmd::Command::cargo_bin("wirerust")
//!   - check !output.status.success() (non-zero exit)
//!   - check stderr.contains("must be >= 1")
//!
//! DF-TEST-NAMESPACE-001: all tests are wrapped in `mod d074_arp_threshold_zero`.
//! DF-AC-TEST-NAME-SYNC-001: function names embed the BC and EC identifiers.

#![allow(non_snake_case)]

mod d074_arp_threshold_zero {
    use assert_cmd::Command;

    // Smallest fixture available for CLI integration tests — contains only
    // Ethernet/IP/TCP frames (no ARP frames), so no analyzer side-effects
    // interfere with the CLI-rejection assertion.  Mirrors the fixture
    // choice made by bc_2_14_105_modbus_dispatch_tests.rs and
    // bc_2_16_story114_arp_tests.rs.
    const FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    // -----------------------------------------------------------------------
    // BC-2.16.008 EC-006 / BC-2.16.013 EC-004 / D-074
    // --arp-storm-rate 0 → CLI fatal error before packet processing
    // -----------------------------------------------------------------------

    /// Guards that `--arp-storm-rate 0` produces a fatal error (non-zero exit)
    /// with a stderr message indicating the value must be >= 1.
    ///
    /// BC-2.16.008 EC-006 / BC-2.16.013 EC-004 / Decision D-074:
    /// A storm-rate of 0 would cause divide-by-zero or degenerate behaviour
    /// in the D3 storm detector (rate = frames / max(1, window)).  The CLI
    /// must reject 0 before any packet is read, mirroring the validation
    /// already applied to --modbus-write-burst-threshold (BC-2.14.024 P3a).
    ///
    /// Assertions:
    ///   Positive: exit status is non-zero (failure).
    ///   Positive: stderr contains "--arp-storm-rate must be >= 1".
    ///   Negative: exit status is NOT 0 (must not succeed).
    ///   Negative: stderr does NOT need to mention "panicked"
    ///             (must be a controlled error, not a crash).
    #[test]
    fn test_BC_2_16_008_EC_006_arp_storm_rate_zero_rejected() {
        let output = Command::cargo_bin("wirerust")
            .expect(
                "wirerust binary must be built — run `cargo build` first \
                 (D-074 / BC-2.16.008 EC-006 test requires the compiled binary)",
            )
            .args(["analyze", "--arp", "--arp-storm-rate", "0", FIXTURE])
            .output()
            .expect("D-074: wirerust command must run without OS error");

        assert!(
            !output.status.success(),
            "D-074 / BC-2.16.008 EC-006 / BC-2.16.013 EC-004: exit status must be non-zero \
             when --arp-storm-rate 0 (value 0 must be rejected before packet processing). \
             Got status: {:?}",
            output.status
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("--arp-storm-rate must be >= 1"),
            "D-074 / BC-2.16.008 EC-006: stderr must contain \
             \"--arp-storm-rate must be >= 1\" when value 0 is supplied. \
             Got stderr: {stderr:?}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.16.012 EC-004 / D-074
    // --arp-spoof-threshold 0 → CLI fatal error before packet processing
    // -----------------------------------------------------------------------

    /// Guards that `--arp-spoof-threshold 0` produces a fatal error (non-zero
    /// exit) with a stderr message indicating the value must be >= 1.
    ///
    /// BC-2.16.012 EC-004 / Decision D-074:
    /// A spoof-threshold of 0 means "fire HIGH on every ARP frame that is
    /// seen at all", which is never the intended behaviour and would flood
    /// the findings list.  The CLI must reject 0 before any packet is read,
    /// mirroring the validation applied to --modbus-write-sustained-threshold
    /// (BC-2.14.024 P3b).
    ///
    /// Assertions:
    ///   Positive: exit status is non-zero (failure).
    ///   Positive: stderr contains "--arp-spoof-threshold must be >= 1".
    ///   Negative: exit status is NOT 0 (must not succeed).
    #[test]
    fn test_BC_2_16_012_EC_004_arp_spoof_threshold_zero_rejected() {
        let output = Command::cargo_bin("wirerust")
            .expect(
                "wirerust binary must be built — run `cargo build` first \
                 (D-074 / BC-2.16.012 EC-004 test requires the compiled binary)",
            )
            .args(["analyze", "--arp", "--arp-spoof-threshold", "0", FIXTURE])
            .output()
            .expect("D-074: wirerust command must run without OS error");

        assert!(
            !output.status.success(),
            "D-074 / BC-2.16.012 EC-004: exit status must be non-zero when \
             --arp-spoof-threshold 0 (value 0 must be rejected before packet processing). \
             Got status: {:?}",
            output.status
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("--arp-spoof-threshold must be >= 1"),
            "D-074 / BC-2.16.012 EC-004: stderr must contain \
             \"--arp-spoof-threshold must be >= 1\" when value 0 is supplied. \
             Got stderr: {stderr:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Positive boundary: value 1 is accepted for both flags
    // -----------------------------------------------------------------------

    /// Guards that `--arp-storm-rate 1` (the minimum valid value) is accepted
    /// by the CLI without error (BC-2.16.013 / D-074 positive boundary).
    ///
    /// This test verifies that the zero-rejection gate does NOT incorrectly
    /// block the minimum legal value.
    ///
    /// Assertions:
    ///   Positive: exit status is 0 (success).
    ///   Negative: exit status is NOT non-zero for value 1.
    #[test]
    fn test_BC_2_16_013_arp_storm_rate_one_accepted() {
        let output = Command::cargo_bin("wirerust")
            .expect(
                "wirerust binary must be built — run `cargo build` first \
                 (D-074 / BC-2.16.013 boundary test requires the compiled binary)",
            )
            .args(["analyze", "--arp", "--arp-storm-rate", "1", FIXTURE])
            .output()
            .expect("D-074: wirerust command must run without OS error");

        assert!(
            output.status.success(),
            "D-074 / BC-2.16.013: --arp-storm-rate 1 must be accepted (minimum valid value). \
             Exit status: {:?}. stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Guards that `--arp-spoof-threshold 1` (the minimum valid value) is
    /// accepted by the CLI without error (BC-2.16.012 / D-074 positive boundary).
    ///
    /// This test verifies that the zero-rejection gate does NOT incorrectly
    /// block the minimum legal value.
    ///
    /// Assertions:
    ///   Positive: exit status is 0 (success).
    ///   Negative: exit status is NOT non-zero for value 1.
    #[test]
    fn test_BC_2_16_012_arp_spoof_threshold_one_accepted() {
        let output = Command::cargo_bin("wirerust")
            .expect(
                "wirerust binary must be built — run `cargo build` first \
                 (D-074 / BC-2.16.012 boundary test requires the compiled binary)",
            )
            .args(["analyze", "--arp", "--arp-spoof-threshold", "1", FIXTURE])
            .output()
            .expect("D-074: wirerust command must run without OS error");

        assert!(
            output.status.success(),
            "D-074 / BC-2.16.012: --arp-spoof-threshold 1 must be accepted (minimum valid value). \
             Exit status: {:?}. stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
