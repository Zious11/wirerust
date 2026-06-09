//! Failing tests for STORY-104: Modbus Detection Emissions + Summary.
//!
//! Covers BC-2.14.013 through BC-2.14.022 — all seven detection rules, the
//! MAX_FINDINGS cap, and the `summarize()` six-key contract.
//!
//! RED GATE: ALL tests must fail (todo!() panics) before implementation begins.
//! Test naming follows `test_BC_S_SS_NNN_xxx` pattern for full traceability.
//!
//! Canonical test vectors used verbatim from BC documents where available.

// BC traceability convention mandates uppercase BC identifiers in function names.
// The non_snake_case lint fires on uppercase — suppressed intentionally.
#![allow(non_snake_case)]

use std::net::{IpAddr, Ipv4Addr};

use wirerust::analyzer::modbus::{
    DEFAULT_WRITE_BURST_THRESHOLD, DEFAULT_WRITE_SUSTAINED_THRESHOLD, MAX_FINDINGS,
    MbapHeader, ModbusAnalyzer, ModbusFlowState,
};
use wirerust::findings::{Confidence, ThreatCategory, Verdict};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::Direction;

// ---------------------------------------------------------------------------
// Helpers — shared ADU builders and test infrastructure
// ---------------------------------------------------------------------------

/// Create a canonical `FlowKey` for tests: client 192.168.1.10:1234 ↔ server 192.168.1.100:502.
fn test_flow_key() -> FlowKey {
    FlowKey::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
        1234,
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
        502,
    )
}

/// Create a standard `ModbusAnalyzer` with default thresholds.
fn default_analyzer() -> ModbusAnalyzer {
    ModbusAnalyzer::new(DEFAULT_WRITE_BURST_THRESHOLD, DEFAULT_WRITE_SUSTAINED_THRESHOLD)
}

/// Build a minimal valid MBAP + PDU byte vector for an arbitrary FC.
///
/// Layout: `[txn_hi, txn_lo, 0x00, 0x00, len_hi, len_lo, unit_id, fc, data...]`
/// `length` field = 1 (unit_id) + 1 (fc) + data.len() — must be in [2, 254].
fn build_adu(txn_id: u16, unit_id: u8, fc: u8, pdu_data: &[u8]) -> Vec<u8> {
    // length field = unit_id (1) + fc (1) + pdu_data.len()
    let length = (2 + pdu_data.len()) as u16;
    let mut adu = vec![
        (txn_id >> 8) as u8,
        (txn_id & 0xFF) as u8,
        0x00,
        0x00, // protocol_id = 0
        (length >> 8) as u8,
        (length & 0xFF) as u8,
        unit_id,
        fc,
    ];
    adu.extend_from_slice(pdu_data);
    adu
}

/// Parse a raw ADU byte slice into a `MbapHeader`.
/// Panics if the slice is too short — test helper only.
fn parse_header(adu: &[u8]) -> MbapHeader {
    wirerust::analyzer::modbus::parse_mbap_header(adu)
        .expect("test ADU must be at least 8 bytes")
}

/// Drive `process_pdu` with a fully-formed ADU, collecting returned findings.
fn drive(
    analyzer: &mut ModbusAnalyzer,
    flow: &mut ModbusFlowState,
    fk: &FlowKey,
    direction: Direction,
    adu: &[u8],
    timestamp: u32,
) -> Vec<wirerust::findings::Finding> {
    let header = parse_header(adu);
    let fc = header.function_code;
    analyzer.process_pdu(fk, flow, direction, &header, fc, adu, timestamp)
}

// ---------------------------------------------------------------------------
// BC-2.14.013 / BC-2.14.014 — Register-write multi-tag finding
// AC-001: FC=0x06 emits exactly one finding with mitre_techniques = ["T0855","T0836"]
// ---------------------------------------------------------------------------

/// test_BC_2_14_013_014_holding_register_write_emits_t0855_t0836
///
/// Canonical vector from BC-2.14.013:
/// ADU: `00 01 00 00 00 06 01 06 00 10 01 F4` (FC=0x06, Write Single Register, UnitID=1)
/// Expected: exactly one Finding; mitre_techniques = ["T0855","T0836"];
/// category = Execution; verdict = Likely; confidence = Medium.
/// Traces to: BC-2.14.013 post.1, BC-2.14.014, STORY-104 AC-001.
#[test]
fn test_BC_2_14_013_014_holding_register_write_emits_t0855_t0836() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x06 (Write Single Register), data = [0x00, 0x10, 0x01, 0xF4]
    let adu: [u8; 12] = [0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x06, 0x00, 0x10, 0x01, 0xF4];
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    assert_eq!(findings.len(), 1, "exactly one finding per write-class PDU");
    let f = &findings[0];
    assert_eq!(f.mitre_techniques, vec!["T0855", "T0836"], "register write must tag T0855+T0836");
    assert!(matches!(f.category, ThreatCategory::Execution), "category = Execution");
    assert!(matches!(f.verdict, Verdict::Likely), "verdict = Likely");
    assert!(matches!(f.confidence, Confidence::Medium), "confidence = Medium");
}

/// test_BC_2_14_013_014_fc_0x10_emits_t0855_t0836
///
/// FC=0x10 (Write Multiple Registers) is in the holding-register subset.
/// Traces to: BC-2.14.014.
#[test]
fn test_BC_2_14_013_014_fc_0x10_emits_t0855_t0836() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x10 with minimal payload
    let adu = build_adu(0x0002, 0x01, 0x10, &[0x00, 0x00, 0x00, 0x01, 0x02, 0x01, 0xF4]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].mitre_techniques, vec!["T0855", "T0836"]);
}

/// test_BC_2_14_013_014_fc_0x16_emits_t0855_t0836
///
/// FC=0x16 (Mask Write Register) is in the holding-register subset.
/// Traces to: BC-2.14.013 EC-003, BC-2.14.014.
#[test]
fn test_BC_2_14_013_014_fc_0x16_emits_t0855_t0836() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    let adu = build_adu(0x0003, 0x01, 0x16, &[0x00, 0x10, 0xFF, 0xFF, 0x00, 0x25]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].mitre_techniques, vec!["T0855", "T0836"]);
}

// ---------------------------------------------------------------------------
// BC-2.14.013 / BC-2.14.015 — Coil-write multi-tag finding
// AC-002: FC=0x05 emits exactly one finding with ["T0855","T0835"]
// ---------------------------------------------------------------------------

/// test_BC_2_14_013_015_coil_write_emits_t0855_t0835
///
/// Canonical vector from BC-2.14.013:
/// ADU: `00 02 00 00 00 06 02 0F 00 00 00 08 01 FF` (FC=0x0F Write Multiple Coils)
/// Expected: mitre_techniques = ["T0855","T0835"].
/// Traces to: BC-2.14.013 post.1, BC-2.14.015, STORY-104 AC-002.
#[test]
fn test_BC_2_14_013_015_coil_write_emits_t0855_t0835() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x0F (Write Multiple Coils): [0x00,0x02,0x00,0x00,0x00,0x06,0x02,0x0F,0x00,0x00,0x00,0x08,0x01,0xFF]
    let adu: Vec<u8> =
        vec![0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x02, 0x0F, 0x00, 0x00, 0x00, 0x08, 0x01, 0xFF];
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    assert_eq!(findings.len(), 1, "exactly one finding for coil write");
    assert_eq!(
        findings[0].mitre_techniques,
        vec!["T0855", "T0835"],
        "coil write must tag T0855+T0835"
    );
}

/// test_BC_2_14_013_015_fc_0x05_emits_t0855_t0835
///
/// FC=0x05 (Write Single Coil).
/// Traces to: BC-2.14.013 EC-006, BC-2.14.015.
#[test]
fn test_BC_2_14_013_015_fc_0x05_emits_t0855_t0835() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    let adu = build_adu(0x0004, 0x01, 0x05, &[0x00, 0x00, 0xFF, 0x00]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].mitre_techniques, vec!["T0855", "T0835"]);
}

// ---------------------------------------------------------------------------
// BC-2.14.013 — File/other write emits T0855 only (no register/coil subtype)
// AC-002: FC in {0x15, 0x17} → ["T0855"] only
// ---------------------------------------------------------------------------

/// test_BC_2_14_013_file_write_emits_t0855_only
///
/// FC=0x15 (Write File Record) — not in register or coil subset.
/// Expected: mitre_techniques = ["T0855"] only.
/// Traces to: BC-2.14.013 invariant 2 (FC 0x15/0x17 → T0855 only), AC-002.
#[test]
fn test_BC_2_14_013_file_write_emits_t0855_only() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x15 with minimal valid data
    let adu = build_adu(0x0005, 0x01, 0x15, &[0x07, 0x00, 0x04, 0x00, 0x00, 0x00, 0x07, 0x00]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    assert_eq!(findings.len(), 1, "one finding for file-record write");
    assert_eq!(findings[0].mitre_techniques, vec!["T0855"], "FC=0x15 → T0855 only, no subtype");
    assert!(!findings[0].mitre_techniques.contains(&"T0836".to_string()));
    assert!(!findings[0].mitre_techniques.contains(&"T0835".to_string()));
}

/// test_BC_2_14_013_fc_0x17_emits_t0855_only_per_pdu_finding
///
/// FC=0x17 (Read/Write Multiple Registers) — in T0831 window set but NOT in the
/// "register write subtype" for a standalone T0836 tag. Per BC-2.14.013 invariant 2:
/// FC 0x17 → T0855 only for the per-PDU non-T0831 finding.
/// (Note: FC 0x17 IS included in the T0831 window per BC-2.14.016, so a 2nd 0x17
/// within 5s yields ["T0855","T0836","T0831"] — but the first write is ["T0855","T0836"]
/// per the T0831 window union-tagging rule table.)
/// Traces to: BC-2.14.013 EC-001.
///
/// DISCREPANCY NOTE: BC-2.14.013 EC-001 states FC=0x17 → ["T0855"] only (not T0836).
/// BC-2.14.016 union-tagging table row 1 states FC {0x06,0x10,0x16,0x17} → ["T0855","T0836"]
/// for the 1st write in a window. This test follows BC-2.14.013 EC-001 (the write-class
/// BC is the primary authority for tag composition). Filed as BC-DISCREPANCY-001 for
/// spec-steward review.
#[test]
fn test_BC_2_14_013_fc_0x17_emits_t0855_only_per_pdu_finding() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x17 (Read/Write Multiple Registers) — first write on flow
    let adu = build_adu(
        0x0006,
        0x01,
        0x17,
        &[0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0x00, 0x01, 0x02, 0x00, 0x64],
    );
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    // Per BC-2.14.013 EC-001: FC=0x17 → ["T0855"] only.
    assert_eq!(findings.len(), 1);
    let tags = &findings[0].mitre_techniques;
    assert!(tags.contains(&"T0855".to_string()), "T0855 must be present");
    assert!(!tags.contains(&"T0836".to_string()), "T0836 must NOT appear for FC=0x17 per BC-2.14.013 EC-001");
}

// ---------------------------------------------------------------------------
// BC-2.14.016 — T0831 inline co-tag on 2nd holding-register write within 5s
// AC-003: first → ["T0855","T0836"]; second → ["T0855","T0836","T0831"]; third → ["T0855","T0836"]
// ---------------------------------------------------------------------------

/// test_BC_2_14_016_t0831_inline_cotag_on_second_holding_register_write
///
/// Deliver three FC=0x06 writes within 5 seconds.
/// findings[0] = ["T0855","T0836"] (1st write, T0831 window starts, count=1)
/// findings[1] = ["T0855","T0836","T0831"] (2nd write, count=2 → T0831 fires once)
/// findings[2] = ["T0855","T0836"] (3rd write, T0831 emit-once exhausted)
/// Traces to: BC-2.14.016, STORY-104 AC-003.
#[test]
fn test_BC_2_14_016_t0831_inline_cotag_on_second_holding_register_write() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    // All three writes within 5s (timestamps 1.0s, 2.0s, 3.0s in microseconds).
    let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
    let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x10, 0x02, 0x00]);
    let adu3 = build_adu(0x0003, 0x01, 0x06, &[0x00, 0x10, 0x03, 0x00]);

    let f1 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 1_000_000);
    let f2 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 2_000_000);
    let f3 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu3, 3_000_000);

    // First write: T0831 window started, count=1 → no T0831 tag.
    assert_eq!(f1.len(), 1, "first write yields exactly one per-PDU finding");
    assert_eq!(
        f1[0].mitre_techniques,
        vec!["T0855", "T0836"],
        "first write: T0831 not yet fired"
    );

    // Second write: count=2, t0831_burst_emitted=false → T0831 co-tagged.
    assert_eq!(f2.len(), 1, "second write yields exactly one per-PDU finding (T0831 is co-tag, not separate)");
    assert_eq!(
        f2[0].mitre_techniques,
        vec!["T0855", "T0836", "T0831"],
        "second write must include T0831 co-tag"
    );

    // Third write: t0831_burst_emitted=true → back to T0836 only (no T0831).
    assert_eq!(f3.len(), 1, "third write yields exactly one per-PDU finding");
    assert_eq!(
        f3[0].mitre_techniques,
        vec!["T0855", "T0836"],
        "third write: T0831 emit-once exhausted"
    );
}

/// test_BC_2_14_016_t0831_window_reset_after_5s
///
/// After 5s the T0831 window resets; a write after the reset counts as the 1st
/// write in a new window and does NOT carry T0831.
/// Traces to: BC-2.14.016 invariant 2 (window-reset path).
#[test]
fn test_BC_2_14_016_t0831_window_reset_after_5s() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    // Two writes within 5s to fire T0831.
    let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
    let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 0);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 1_000_000);

    // Third write at 6s (window expired — wrapping_sub(6_000_000, 0) = 6_000_000 > 5_000_000).
    let adu3 = build_adu(0x0003, 0x01, 0x06, &[0x00, 0x12, 0x01, 0xF4]);
    let f3 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu3, 6_000_000);

    assert_eq!(f3.len(), 1);
    assert_eq!(
        f3[0].mitre_techniques,
        vec!["T0855", "T0836"],
        "after window reset: T0831 must NOT fire on the 1st write of a new window"
    );
    assert!(!f3[0].mitre_techniques.contains(&"T0831".to_string()));
}

/// test_BC_2_14_016_t0831_window_update_before_emission_ordering
///
/// Verifies the "window-update FIRST, then emission check" ordering from BC-2.14.016
/// invariant 2. Specifically: `t0831_window_write_count` must be >= 2 AFTER the
/// increment that processes the second write, at which point the emission check fires.
/// Traces to: BC-2.14.016 invariant 2 (evaluation order).
#[test]
fn test_BC_2_14_016_t0831_window_update_before_emission_ordering() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let adu1 = build_adu(0x0001, 0x01, 0x10, &[0x00, 0x00, 0x00, 0x01, 0x02, 0x01, 0x00]);
    let adu2 = build_adu(0x0002, 0x01, 0x10, &[0x00, 0x01, 0x00, 0x01, 0x02, 0x02, 0x00]);

    let f1 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 500_000);
    let f2 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 1_500_000);

    // If ordering is wrong (check before update): T0831 would never fire because
    // count=1 before increment. Correct ordering means count=2 after increment → fires.
    assert!(!f1[0].mitre_techniques.contains(&"T0831".to_string()), "first write must not carry T0831");
    assert!(f2[0].mitre_techniques.contains(&"T0831".to_string()), "second write must carry T0831 (update-before-check ordering)");
}

/// test_BC_2_14_016_t0831_wrapping_sub_wrap
///
/// Timestamp wrap: write1 at ts=0xFFFFFF00, write2 at ts=0x00000100.
/// wrapping_sub(0x00000100, 0xFFFFFF00) = 0x00000200 = 512 µs << 5_000_000 µs → same window.
/// Both writes within the same T0831 window → second write carries T0831.
/// Traces to: BC-2.14.016 + f2-fix-directives §11.5b (wrapping_sub policy).
/// Also traces to: STORY-104 AC-006 (wrapping_sub for all window elapsed computations).
#[test]
fn test_BC_2_14_016_t0831_wrapping_sub_wrap() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
    let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x10, 0x02, 0x00]);

    // write1 near u32::MAX boundary, write2 slightly past zero (wrap-around)
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 0xFFFFFF00_u32);
    let f2 = drive(
        &mut az,
        &mut flow,
        &fk,
        Direction::ClientToServer,
        &adu2,
        0x00000100_u32,
    );

    // wrapping_sub(0x100, 0xFFFFFF00) = 0x200 = 512 µs → within 5s window → T0831 fires
    // (no panic from overflow-checks either)
    assert!(f2[0].mitre_techniques.contains(&"T0831".to_string()), "wrapping_sub ensures T0831 fires across u32 boundary");
}

// ---------------------------------------------------------------------------
// BC-2.14.017 — Burst detector (T0806+T0855, 1-second window)
// AC-004: >20 writes in 1s → separate burst finding with ["T0806","T0855"]
// ---------------------------------------------------------------------------

/// test_BC_2_14_017_burst_detector_fires_at_threshold_plus_1
///
/// Deliver 21 write-class FCs within 1 second (default burst threshold = 20).
/// The 21st write tips the burst threshold. Expected:
/// - Exactly 21 per-PDU write findings emitted (one per write).
/// - Exactly 1 burst finding with mitre_techniques = ["T0806","T0855"].
/// - Total findings returned across all 21 calls = 22 (21 per-PDU + 1 burst).
/// Traces to: BC-2.14.017 invariant 1, STORY-104 AC-004.
#[test]
fn test_BC_2_14_017_burst_detector_fires_at_threshold_plus_1() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    // 21 writes, each 10ms apart (all within 1s window = 200ms total << 1_000_000µs)
    for i in 0..21_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        let ts = i * 10_000; // 10ms intervals → 200ms total
        let mut findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
        all_findings.append(&mut findings);
    }

    let per_pdu: Vec<_> = all_findings
        .iter()
        .filter(|f| {
            f.mitre_techniques.contains(&"T0836".to_string())
                || (f.mitre_techniques.contains(&"T0855".to_string())
                    && !f.mitre_techniques.contains(&"T0806".to_string()))
        })
        .collect();
    let burst: Vec<_> = all_findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
        .collect();

    assert_eq!(per_pdu.len(), 21, "21 per-PDU write findings (one per write-class PDU)");
    assert_eq!(burst.len(), 1, "exactly one burst finding when threshold is exceeded");
    assert_eq!(
        burst[0].mitre_techniques,
        vec!["T0806", "T0855"],
        "burst finding: T0806 first, T0855 second (canonical order)"
    );
    assert_eq!(all_findings.len(), 22, "21 per-PDU + 1 burst = 22 total (AC-012)");
}

/// test_BC_2_14_017_burst_does_not_fire_at_exactly_threshold
///
/// Exactly 20 writes within 1s (equal to threshold, not exceeding it).
/// Burst fires only when count STRICTLY EXCEEDS threshold.
/// Traces to: BC-2.14.017 invariant 1 ("count > write_burst_threshold").
#[test]
fn test_BC_2_14_017_burst_does_not_fire_at_exactly_threshold() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    for i in 0..20_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        let ts = i * 10_000;
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
        all_findings.append(&mut f);
    }
    let burst_count = all_findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
        .count();
    assert_eq!(burst_count, 0, "no burst at exactly threshold=20 (strict >)");
}

/// test_BC_2_14_017_burst_emit_once_per_window
///
/// After the burst fires (21st write), deliver another 10 writes still within the
/// same 1s window. The burst finding must fire EXACTLY ONCE per window.
/// Traces to: BC-2.14.017 invariant 1 (`window_burst_emitted` guard).
#[test]
fn test_BC_2_14_017_burst_emit_once_per_window() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    // 31 writes within 300ms (all same window)
    for i in 0..31_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        let ts = i * 10_000;
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
        all_findings.append(&mut f);
    }
    let burst_count = all_findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
        .count();
    assert_eq!(burst_count, 1, "burst finding must fire exactly once per window (emit-once guard)");
}

// ---------------------------------------------------------------------------
// BC-2.14.017 — Sustained detector (truncation-free microsecond math)
// AC-005: (count*1_000_000) > (threshold*elapsed_us) AND elapsed_us >= 2_000_000
// ---------------------------------------------------------------------------

/// test_BC_2_14_017_sustained_detector_uses_truncation_free_math_no_false_positive
///
/// 25 writes over 2.9s (elapsed_us = 2_900_000; rate = 8.62/s < 10/s threshold).
/// Naive integer-division: 25 > 10*2 = 20 → TRUE (FALSE POSITIVE).
/// Correct formula: 25*1_000_000=25_000_000 > 10*2_900_000=29_000_000 → FALSE → no burst.
/// Traces to: BC-2.14.017 invariant 2, f2-fix-directives §11.5a, STORY-104 AC-005.
#[test]
fn test_BC_2_14_017_sustained_detector_uses_truncation_free_math_no_false_positive() {
    // Use a HIGH burst threshold so burst detector does NOT fire on its own.
    let mut az = ModbusAnalyzer::new(100, DEFAULT_WRITE_SUSTAINED_THRESHOLD);
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    // 25 writes; first at ts=0, last at ts=2_900_000 (even spacing within 2.9s).
    // Each write ~116_000µs apart.
    for i in 0..25_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        let ts = (i * 116_000) as u32;
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
        all_findings.append(&mut f);
    }

    let sustained_count = all_findings
        .iter()
        .filter(|f| {
            f.mitre_techniques.contains(&"T0806".to_string())
        })
        .count();
    assert_eq!(
        sustained_count, 0,
        "truncation-free math: 8.62/s < 10/s threshold → NO sustained finding (would be false positive with naive integer division)"
    );
}

/// test_BC_2_14_017_sustained_detector_fires_when_rate_exceeded
///
/// 25 writes over 2.0s (elapsed_us = 2_000_000; rate = 12.5/s > 10/s threshold).
/// Correct formula: 25*1_000_000=25_000_000 > 10*2_000_000=20_000_000 → TRUE → fires.
/// Traces to: BC-2.14.017 invariant 2, STORY-104 AC-005.
#[test]
fn test_BC_2_14_017_sustained_detector_fires_when_rate_exceeded() {
    // HIGH burst threshold to isolate the sustained detector.
    let mut az = ModbusAnalyzer::new(100, DEFAULT_WRITE_SUSTAINED_THRESHOLD);
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    // 25 writes over 2s: each ~80_000µs apart so last ts ≈ 1_920_000µs ≈ 2s.
    // Deliver 24 writes to accumulate; 25th at exactly elapsed=2_000_000.
    for i in 0..24_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        let ts = i * 80_000;
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
        all_findings.append(&mut f);
    }
    // 25th write at ts = 2_000_000 (elapsed from 0 = exactly 2s)
    let adu25 = build_adu(0x0019, 0x01, 0x06, &[0x00, 0x18, 0x01, 0x00]);
    let mut f25 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu25, 2_000_000);
    all_findings.append(&mut f25);

    let sustained_count = all_findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
        .count();
    assert!(
        sustained_count >= 1,
        "12.5/s > 10/s threshold over 2s → sustained finding must fire (truncation-free math)"
    );
    let burst_f = all_findings
        .iter()
        .find(|f| f.mitre_techniques.contains(&"T0806".to_string()))
        .expect("sustained finding must exist");
    assert_eq!(
        burst_f.mitre_techniques,
        vec!["T0806", "T0855"],
        "sustained finding carries T0806+T0855"
    );
}

/// test_BC_2_14_017_sustained_low_and_slow_fires
///
/// Low-and-slow scenario: 8 writes/s sustained over >=2s.
/// With threshold=5/s: 8 > 5 → fires. Ensures the detector catches slow attacks
/// that the 1s burst window misses.
/// Traces to: BC-2.14.017 (motivation for sustained detector).
#[test]
fn test_BC_2_14_017_sustained_low_and_slow_fires() {
    // Burst threshold very high (100) so only sustained fires.
    // Sustained threshold = 5 writes/s.
    let mut az = ModbusAnalyzer::new(100, 5);
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    // 17 writes over 2s (17/2s = 8.5/s > 5/s threshold).
    // Space them at 125_000µs intervals (8/s). 16 intervals = 2_000_000µs elapsed.
    for i in 0..16_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        let ts = i * 125_000;
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
        all_findings.append(&mut f);
    }
    // 17th write at exactly ts=2_000_000 triggers the window check
    let adu17 = build_adu(0x0011, 0x01, 0x06, &[0x00, 0x10, 0x01, 0x00]);
    let mut f17 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu17, 2_000_000);
    all_findings.append(&mut f17);

    let sustained = all_findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0806".to_string()))
        .count();
    assert!(sustained >= 1, "low-and-slow (8.5/s > 5/s threshold over 2s) must fire sustained detector");
}

// ---------------------------------------------------------------------------
// BC-2.14.017 / AC-006 — wrapping_sub for all window elapsed computations
// ---------------------------------------------------------------------------

/// test_BC_2_14_017_window_elapsed_uses_wrapping_sub_no_panic
///
/// Deliver a write at ts=0xFFFFFF00 (near u32::MAX), then a write at ts=0x00000100.
/// Plain subtraction: 0x100 - 0xFFFFFF00 = underflow → panic in debug mode.
/// wrapping_sub: 0x100u32.wrapping_sub(0xFFFFFF00) = 0x00000200 = 512 µs → no panic.
/// Expected: no panic; 512µs elapsed << 1_000_000µs → still in burst window.
/// Traces to: BC-2.14.017 + f2-fix-directives §11.5b, STORY-104 AC-006.
#[test]
fn test_BC_2_14_017_window_elapsed_uses_wrapping_sub_no_panic() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let adu1 = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
    let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);

    // Must not panic (plain subtraction would panic in debug overflow-checks mode).
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu1, 0xFFFFFF00_u32);
    let f2 = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 0x00000100_u32);

    // wrapping_sub = 0x200 = 512µs → WITHIN 1s burst window → window_write_count = 2
    // (No burst yet since threshold = 20; just verifying no panic and correct window state.)
    assert!(!f2.is_empty(), "second write must return a finding without panic");
    assert!(
        !f2.iter().any(|f| f.mitre_techniques.contains(&"T0806".to_string())),
        "512µs elapsed with 2 writes: no burst (threshold=20)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.14.018 — Diagnostics FC 0x08 sub-function 0x0004 / 0x0001 → T0814
// AC-007: Force Listen Only or Restart Comms → Finding ["T0814"]
// ---------------------------------------------------------------------------

/// test_BC_2_14_018_diagnostics_force_listen_only_emits_t0814
///
/// FC=0x08 with sub-function 0x0004 (Force Listen Only Mode) → T0814 finding.
/// ADU bytes: 7 MBAP + FC 0x08 + sub-func 0x00 0x04 + data 0x00 0x00 = 12 bytes.
/// Traces to: BC-2.14.018 post.1, STORY-104 AC-007.
#[test]
fn test_BC_2_14_018_diagnostics_force_listen_only_emits_t0814() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x08, sub-func=0x0004 (Force Listen Only), query data = 0x0000
    // ADU: [txn=0x0001, proto=0x0000, length=0x0006, unit=0x01, fc=0x08, 0x00, 0x04, 0x00, 0x00]
    let adu = build_adu(0x0001, 0x01, 0x08, &[0x00, 0x04, 0x00, 0x00]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let t0814_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
        .collect();
    assert!(!t0814_findings.is_empty(), "FC=0x08/0x0004 must emit a T0814 finding");
    assert_eq!(t0814_findings[0].mitre_techniques, vec!["T0814"], "T0814 only (no other tags)");
}

/// test_BC_2_14_018_diagnostics_restart_comms_emits_t0814
///
/// FC=0x08 with sub-function 0x0001 (Restart Communications Option) → T0814 finding.
/// Traces to: BC-2.14.018 post.1.
#[test]
fn test_BC_2_14_018_diagnostics_restart_comms_emits_t0814() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    let adu = build_adu(0x0002, 0x01, 0x08, &[0x00, 0x01, 0x00, 0x00]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let t0814_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
        .collect();
    assert!(!t0814_findings.is_empty(), "FC=0x08/0x0001 must emit a T0814 finding");
}

/// test_BC_2_14_018_diagnostics_other_subfunc_does_not_emit_t0814
///
/// FC=0x08 with sub-function 0x0000 (loopback/Return Query Data) → no T0814 finding.
/// Traces to: BC-2.14.018 invariant 2 (only 0x0001 and 0x0004 trigger T0814).
/// AC-006 edge case.
#[test]
fn test_BC_2_14_018_diagnostics_other_subfunc_does_not_emit_t0814() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // Sub-func 0x0000 (loopback) — no T0814
    let adu = build_adu(0x0003, 0x01, 0x08, &[0x00, 0x00, 0xAB, 0xCD]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let t0814_count = findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0814".to_string()))
        .count();
    assert_eq!(t0814_count, 0, "sub-func 0x0000 must NOT emit T0814");
}

// ---------------------------------------------------------------------------
// BC-2.14.019 — Exception-burst anomaly: >5 same-code exceptions in 10s
// AC-008: 6th exception of same code → Anomaly finding with mitre_techniques = []
// ---------------------------------------------------------------------------

/// test_BC_2_14_019_exception_burst_emits_anomaly_finding
///
/// Deliver 11 exception responses for FC=0x83 (exception for Write Single Reg 0x06,
/// exception code=0x01) within 10 seconds.
/// The 6th exception (count STRICTLY EXCEEDS 5) must emit an Anomaly finding with
/// mitre_techniques: vec![].
/// Traces to: BC-2.14.019 post.1 (path A), STORY-104 AC-008.
#[test]
fn test_BC_2_14_019_exception_burst_emits_anomaly_finding() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    // Pre-insert a pending request so attribute_exception can match:
    // request: txn=0x000N, unit=0x01, FC=0x06
    let mut all_findings = Vec::new();
    for i in 0..11_u32 {
        // Insert the request first (so exception can be attributed)
        let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &req_adu, i * 100_000);

        // Exception response: FC=0x86 (=0x06|0x80), exception code=0x01
        // ADU: [txn, 0x00, 0x00, length=0x03, unit=0x01, FC=0x86, code=0x01]
        let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ServerToClient, &exc_adu, i * 100_000 + 50_000);
        all_findings.append(&mut f);
    }

    let anomaly_findings: Vec<_> = all_findings
        .iter()
        .filter(|f| matches!(f.category, ThreatCategory::Anomaly))
        .collect();

    assert!(!anomaly_findings.is_empty(), "6+ same-code exceptions must emit at least one Anomaly finding");
    // Anomaly finding from BC-2.14.019 has empty mitre_techniques
    let exception_anomaly = anomaly_findings
        .iter()
        .find(|f| f.mitre_techniques.is_empty())
        .expect("exception-burst Anomaly must have mitre_techniques: vec![]");
    assert!(exception_anomaly.mitre_techniques.is_empty(), "exception-burst Anomaly: no MITRE technique");
}

/// test_BC_2_14_019_exception_burst_does_not_fire_at_threshold
///
/// Exactly 5 exception responses (equal to EXCEPTION_RATE_THRESHOLD, not exceeding it).
/// No Anomaly finding expected (strict > check).
/// Traces to: BC-2.14.019 precondition 5 ("STRICTLY EXCEEDS").
#[test]
fn test_BC_2_14_019_exception_burst_does_not_fire_at_threshold() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    for i in 0..5_u32 {
        let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &req_adu, i * 100_000);

        let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ServerToClient, &exc_adu, i * 100_000 + 50_000);
        all_findings.append(&mut f);
    }

    let exception_anomaly_count = all_findings
        .iter()
        .filter(|f| matches!(f.category, ThreatCategory::Anomaly) && f.mitre_techniques.is_empty())
        .count();
    assert_eq!(exception_anomaly_count, 0, "exactly 5 exceptions (=threshold) must NOT fire anomaly (strict > required)");
}

/// test_BC_2_14_019_exception_burst_emit_once_per_window
///
/// After the burst fires on the 6th exception, subsequent exceptions in the same window
/// must NOT re-emit the anomaly (emit-once guard per BC-2.14.019 post.2).
/// Traces to: BC-2.14.019 post.2 (exception_burst_emitted guard).
#[test]
fn test_BC_2_14_019_exception_burst_emit_once_per_window() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut all_findings = Vec::new();
    for i in 0..20_u32 {
        let req_adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &req_adu, i * 100_000);
        let exc_adu = build_adu(i as u16, 0x01, 0x86, &[0x01]);
        let mut f = drive(&mut az, &mut flow, &fk, Direction::ServerToClient, &exc_adu, i * 100_000 + 50_000);
        all_findings.append(&mut f);
    }

    let exception_anomaly_count = all_findings
        .iter()
        .filter(|f| matches!(f.category, ThreatCategory::Anomaly) && f.mitre_techniques.is_empty())
        .count();
    assert_eq!(exception_anomaly_count, 1, "exception burst anomaly must fire exactly once per window (20 exceptions in same window → still one anomaly finding)");
}

// ---------------------------------------------------------------------------
// BC-2.14.020 — Recon FCs: 0x11 and 0x2B/0x0E → T0888; FC=0x07 → no finding
// AC-009
// ---------------------------------------------------------------------------

/// test_BC_2_14_020_recon_fc_0x11_emits_t0888
///
/// FC=0x11 (Report Server ID) in ClientToServer → T0888 finding.
/// Traces to: BC-2.14.020 post.1 (recon path), STORY-104 AC-009.
#[test]
fn test_BC_2_14_020_recon_fc_0x11_emits_t0888() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x11 (Report Server ID) — no payload
    let adu = build_adu(0x0001, 0x01, 0x11, &[]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let t0888_count = findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
        .count();
    assert!(t0888_count >= 1, "FC=0x11 must emit a T0888 finding");
    assert_eq!(
        findings.iter().find(|f| f.mitre_techniques.contains(&"T0888".to_string())).unwrap().mitre_techniques,
        vec!["T0888"],
        "T0888 only (no other tags)"
    );
}

/// test_BC_2_14_020_recon_fc_0x2b_0x0e_emits_t0888
///
/// FC=0x2B (MEI Encapsulated Interface) with MEI type=0x0E (Read Device Identification).
/// Expected: T0888 finding.
/// Traces to: BC-2.14.020 post.1 (recon path), STORY-104 AC-009.
#[test]
fn test_BC_2_14_020_recon_fc_0x2b_0x0e_emits_t0888() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x2B, MEI type=0x0E (Read Device Identification), Read Device ID code=0x01
    let adu = build_adu(0x0001, 0x01, 0x2B, &[0x0E, 0x01, 0x00]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let t0888_count = findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
        .count();
    assert!(t0888_count >= 1, "FC=0x2B/0x0E must emit a T0888 finding");
}

/// test_BC_2_14_020_recon_fc_0x2b_non_0x0e_does_not_emit_t0888
///
/// FC=0x2B with MEI type != 0x0E → no T0888 (Decision 12: only MEI type 0x0E maps
/// to T0888). May emit Anomaly (unknown diagnostic sub-function) but NOT T0888.
/// Traces to: BC-2.14.020 EC-005 / STORY-104 AC-009.
#[test]
fn test_BC_2_14_020_recon_fc_0x2b_non_0x0e_does_not_emit_t0888() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x2B, MEI type=0x0D (not 0x0E) — no T0888
    let adu = build_adu(0x0001, 0x01, 0x2B, &[0x0D, 0x01]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let t0888_count = findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
        .count();
    assert_eq!(t0888_count, 0, "FC=0x2B with MEI type!=0x0E must NOT emit T0888");
}

/// test_BC_2_14_020_fc_0x07_does_not_emit_t0888
///
/// FC=0x07 (Read Exception Status) does NOT emit a T0888 finding (Decision 12 in
/// f2-fix-directives.md §12: FC 0x07 removed as standalone recon indicator).
/// Traces to: BC-2.14.020 description ("FC 0x07 is NOT a standalone recon indicator"),
/// STORY-104 AC-009.
#[test]
fn test_BC_2_14_020_fc_0x07_does_not_emit_t0888() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    let adu = build_adu(0x0001, 0x01, 0x07, &[]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let t0888_count = findings
        .iter()
        .filter(|f| f.mitre_techniques.contains(&"T0888".to_string()))
        .count();
    assert_eq!(t0888_count, 0, "FC=0x07 must NOT emit a T0888 finding (per Decision 12)");
}

/// test_BC_2_14_020_unknown_fc_emits_anomaly
///
/// A genuinely unknown FC (e.g., 0x60 — not in any defined set) should emit an
/// Anomaly finding with mitre_techniques: vec![].
/// Traces to: BC-2.14.020 post.1 (unknown FC path).
#[test]
fn test_BC_2_14_020_unknown_fc_emits_anomaly() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // FC=0x60 — genuinely unknown (not in any defined set, not >= 0x80)
    let adu = build_adu(0x0001, 0x01, 0x60, &[0x01, 0x02]);
    let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    let anomaly_count = findings
        .iter()
        .filter(|f| matches!(f.category, ThreatCategory::Anomaly))
        .count();
    assert!(anomaly_count >= 1, "unknown FC must emit an Anomaly finding");
    let anomaly_f = findings
        .iter()
        .find(|f| matches!(f.category, ThreatCategory::Anomaly))
        .unwrap();
    assert!(anomaly_f.mitre_techniques.is_empty(), "unknown FC Anomaly: mitre_techniques must be empty");
}

// ---------------------------------------------------------------------------
// BC-2.14.021 — summarize() returns exactly six keys
// AC-010
// ---------------------------------------------------------------------------

/// test_BC_2_14_021_summarize_returns_six_keys
///
/// Process a mix of ADUs, then call `summarize()`. Assert all six required keys
/// are present in `detail`: pdu_count, write_count, exception_count, parse_errors,
/// function_code_distribution, dropped_findings.
/// Traces to: BC-2.14.021 post.1, STORY-104 AC-010.
#[test]
fn test_BC_2_14_021_summarize_returns_six_keys() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    // 3 reads (FC=0x03), 2 writes (FC=0x06)
    for i in 0..3_u32 {
        let adu = build_adu(i as u16, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, i * 100_000);
    }
    for i in 0..2_u32 {
        let adu = build_adu((10 + i) as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000 + i * 100_000);
    }

    let summary = az.summarize();
    let detail = &summary.detail;

    // All six keys must be present
    assert!(detail.contains_key("pdu_count"), "missing key: pdu_count");
    assert!(detail.contains_key("write_count"), "missing key: write_count");
    assert!(detail.contains_key("exception_count"), "missing key: exception_count");
    assert!(detail.contains_key("parse_errors"), "missing key: parse_errors");
    assert!(detail.contains_key("function_code_distribution"), "missing key: function_code_distribution");
    assert!(detail.contains_key("dropped_findings"), "missing key: dropped_findings");
    assert_eq!(detail.len(), 6, "must have EXACTLY six keys (not more, not fewer)");
}

/// test_BC_2_14_021_summarize_pdu_count_correct
///
/// After processing 5 valid PDUs, `pdu_count` in summary must be 5.
/// Traces to: BC-2.14.021 invariant 4 (pdu_count counts valid PDUs only).
#[test]
fn test_BC_2_14_021_summarize_pdu_count_correct() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    for i in 0..5_u32 {
        let adu = build_adu(i as u16, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, i * 100_000);
    }

    let summary = az.summarize();
    let pdu_count = summary.detail["pdu_count"].as_u64().expect("pdu_count must be a number");
    assert_eq!(pdu_count, 5, "pdu_count must equal number of valid PDUs processed");
}

/// test_BC_2_14_021_summarize_write_count_correct
///
/// After processing 2 write-class PDUs, `write_count` in summary must be 2.
/// Traces to: BC-2.14.021 invariant 4 (write_count = write-class FCs in request direction).
#[test]
fn test_BC_2_14_021_summarize_write_count_correct() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    for i in 0..2_u32 {
        let adu = build_adu(i as u16, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, i * 100_000);
    }

    let summary = az.summarize();
    let write_count = summary.detail["write_count"].as_u64().expect("write_count must be a number");
    assert_eq!(write_count, 2);
}

/// test_BC_2_14_021_summarize_dropped_findings_always_present
///
/// Even when no findings are dropped (cap never hit), `dropped_findings` must be
/// present with value 0.
/// Traces to: BC-2.14.021 invariant 1 (key ALWAYS present), BC-2.14.022 post.5.
#[test]
fn test_BC_2_14_021_summarize_dropped_findings_always_present() {
    let az = default_analyzer();
    let summary = az.summarize();
    assert!(summary.detail.contains_key("dropped_findings"), "dropped_findings must always be present");
    let val = summary.detail["dropped_findings"].as_u64().unwrap_or(99);
    assert_eq!(val, 0, "dropped_findings must be 0 when cap never hit");
}

/// test_BC_2_14_021_summarize_function_code_distribution_hex_format
///
/// `function_code_distribution` keys use "0x{:02X}" format (uppercase hex, 0-padded, 0x prefix).
/// Example: FC 0x03 → key "0x03"; FC 0x10 → key "0x10".
/// Traces to: BC-2.14.021 invariant 3.
#[test]
fn test_BC_2_14_021_summarize_function_code_distribution_hex_format() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    // Process one FC=0x03 read
    let adu = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);

    let summary = az.summarize();
    let dist = summary.detail["function_code_distribution"]
        .as_object()
        .expect("function_code_distribution must be a JSON object");
    assert!(dist.contains_key("0x03"), "FC 0x03 must appear as key '0x03' (uppercase hex, 0x prefix)");
    assert_eq!(dist["0x03"].as_u64().unwrap(), 1);
}

/// test_BC_2_14_021_summarize_function_code_distribution_no_zero_entries
///
/// FC codes that were never observed must NOT appear in `function_code_distribution`.
/// Traces to: BC-2.14.021 post.2, invariant 2 (zero-count suppression).
#[test]
fn test_BC_2_14_021_summarize_function_code_distribution_no_zero_entries() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let adu = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);

    let summary = az.summarize();
    let dist = summary.detail["function_code_distribution"]
        .as_object()
        .expect("must be object");
    // Only FC 0x03 was observed — all other 255 FC codes must be absent.
    assert_eq!(dist.len(), 1, "only observed FCs must appear in distribution (no zero entries)");
    assert!(dist.contains_key("0x03"));
}

// ---------------------------------------------------------------------------
// BC-2.14.022 — MAX_FINDINGS cap (10,000) and poison-skip
// AC-011: cap reached → no push, dropped_findings++, counters still updated
// ---------------------------------------------------------------------------

/// test_BC_2_14_022_max_findings_cap_poison_skip
///
/// Pre-fill `all_findings` to MAX_FINDINGS=10_000, then deliver one write-class PDU.
/// Expected:
/// - `all_findings.len()` remains 10_000 (no push).
/// - `dropped_findings == 1` (one finding discarded).
/// - `total_write_count` incremented (counter unaffected by cap).
/// Traces to: BC-2.14.022 post.1-4, STORY-104 AC-011.
#[test]
fn test_BC_2_14_022_max_findings_cap_poison_skip() {
    use wirerust::findings::{Finding, ThreatCategory, Verdict, Confidence};
    let mut az = default_analyzer();

    // Pre-fill findings to cap using a dummy finding.
    let dummy = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "dummy".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };
    for _ in 0..MAX_FINDINGS {
        az.all_findings.push(dummy.clone());
    }
    assert_eq!(az.all_findings.len(), MAX_FINDINGS);

    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // One more write-class PDU — should poison-skip the finding
    let adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);

    assert_eq!(
        az.all_findings.len(),
        MAX_FINDINGS,
        "all_findings must NOT exceed MAX_FINDINGS after poison-skip"
    );
    assert_eq!(az.dropped_findings, 1, "dropped_findings must be incremented for each skipped push");
    assert_eq!(az.total_write_count, 1, "total_write_count must be incremented even when cap is hit");
}

/// test_BC_2_14_022_dropped_findings_counts_each_skipped_push
///
/// Pre-fill to cap, then deliver 3 write-class PDUs. `dropped_findings` must be 3
/// (one per skipped finding push, not one per PDU if a PDU would emit multiple findings).
/// Traces to: BC-2.14.022 post.2 ("per skipped finding push attempt").
#[test]
fn test_BC_2_14_022_dropped_findings_counts_each_skipped_push() {
    use wirerust::findings::{Finding, ThreatCategory, Verdict, Confidence};
    let mut az = default_analyzer();

    let dummy = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "dummy".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };
    for _ in 0..MAX_FINDINGS {
        az.all_findings.push(dummy.clone());
    }

    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    for i in 0..3_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, i * 100_000);
    }

    assert_eq!(az.all_findings.len(), MAX_FINDINGS, "cap must not be exceeded");
    assert!(az.dropped_findings >= 3, "at least 3 dropped_findings after 3 skipped write findings");
}

/// test_BC_2_14_022_counters_unaffected_by_cap
///
/// After cap is hit, `fn_code_counts` must still be incremented for each valid PDU.
/// Traces to: BC-2.14.022 post.3 (counters are UNAFFECTED by findings cap).
#[test]
fn test_BC_2_14_022_counters_unaffected_by_cap() {
    use wirerust::findings::{Finding, ThreatCategory, Verdict, Confidence};
    let mut az = default_analyzer();

    let dummy = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "dummy".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };
    for _ in 0..MAX_FINDINGS {
        az.all_findings.push(dummy.clone());
    }

    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    let adu = build_adu(0x0001, 0x01, 0x03, &[0x00, 0x00, 0x00, 0x01]);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 0);

    // FC=0x03 count must be 1 even though cap was already hit
    assert_eq!(*az.fn_code_counts.get(&0x03).unwrap_or(&0), 1, "fn_code_counts must be updated regardless of cap");
    assert_eq!(az.total_pdu_count, 1, "total_pdu_count must be updated regardless of cap");
}

// ---------------------------------------------------------------------------
// AC-012 — Burst finding is separate from per-PDU finding (BC-2.14.013 invariant 5)
// ---------------------------------------------------------------------------

/// test_BC_2_14_013_burst_and_per_pdu_finding_are_separate
///
/// The T0806+T0855 burst finding is a SEPARATE Finding object, emitted alongside
/// (not instead of) the per-PDU write finding. When the 21st write tips the burst
/// threshold, that PDU should have generated: 1 per-PDU write finding + 1 burst finding.
/// Total `all_findings` after 21 writes = 22 (21 per-PDU + 1 burst).
/// Traces to: BC-2.14.013 invariant 5, STORY-104 AC-012.
#[test]
fn test_BC_2_14_013_burst_and_per_pdu_finding_are_separate() {
    let mut az = default_analyzer();
    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();

    let mut total_findings = 0usize;
    for i in 0..21_u32 {
        let adu = build_adu(i as u16 + 1, 0x01, 0x06, &[0x00, i as u8, 0x01, 0x00]);
        let ts = i * 10_000; // 10ms intervals — all within 1s window
        let findings = drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, ts);
        total_findings += findings.len();
    }

    assert_eq!(total_findings, 22, "21 per-PDU findings + 1 burst finding = 22 total (burst supplements, not replaces)");

    // Additionally verify the burst finding is distinct (it's a Finding with T0806, not the per-PDU T0836 finding)
    let per_pdu_count = az.all_findings.iter().filter(|f| {
        !f.mitre_techniques.contains(&"T0806".to_string())
            && f.mitre_techniques.contains(&"T0855".to_string())
    }).count();
    let burst_count = az.all_findings.iter().filter(|f| {
        f.mitre_techniques.contains(&"T0806".to_string())
    }).count();

    assert_eq!(per_pdu_count, 21, "21 distinct per-PDU write findings in all_findings");
    assert_eq!(burst_count, 1, "1 distinct burst finding in all_findings");
}

// ---------------------------------------------------------------------------
// EC-003 (from STORY-104 edge cases) — Two findings would be emitted when cap
// at 9999; first push succeeds, second is dropped.
// ---------------------------------------------------------------------------

/// test_BC_2_14_022_EC003_second_finding_dropped_when_cap_at_9999
///
/// EC-003: `all_findings.len() == 9999`; burst threshold also tips on the same PDU
/// (which would emit 2 findings: per-PDU write + burst). First finding pushed
/// (count=10,000); second finding skipped (`dropped_findings = 1`).
/// Traces to: STORY-104 edge case EC-003, BC-2.14.022.
#[test]
fn test_BC_2_14_022_EC003_second_finding_dropped_when_cap_at_9999() {
    use wirerust::findings::{Finding, ThreatCategory, Verdict, Confidence};

    // Use burst threshold = 1 so any single write also tips the burst.
    let mut az = ModbusAnalyzer::new(1, 100);

    let dummy = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "dummy".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };
    // Pre-fill to 9999 (one below cap)
    for _ in 0..(MAX_FINDINGS - 1) {
        az.all_findings.push(dummy.clone());
    }
    assert_eq!(az.all_findings.len(), MAX_FINDINGS - 1);

    let mut flow = ModbusFlowState::default();
    let fk = test_flow_key();
    // First write: with threshold=1, after 1st write count=1+1=2 > 1 → burst fires.
    // This PDU would emit: 1 per-PDU write finding + 1 burst finding = 2 findings.
    // Cap allows: first push (9999→10000), second push dropped.
    let adu = build_adu(0x0001, 0x01, 0x06, &[0x00, 0x10, 0x01, 0xF4]);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu, 1_000_000);
    // Second write at same ts tips burst immediately (window_write_count=2 > threshold=1)
    let adu2 = build_adu(0x0002, 0x01, 0x06, &[0x00, 0x11, 0x01, 0xF4]);
    drive(&mut az, &mut flow, &fk, Direction::ClientToServer, &adu2, 1_000_100);

    assert_eq!(
        az.all_findings.len(),
        MAX_FINDINGS,
        "all_findings must be exactly MAX_FINDINGS after EC-003 scenario"
    );
    assert!(az.dropped_findings >= 1, "at least one finding must be dropped (dropped_findings >= 1)");
}
