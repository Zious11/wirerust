//! EtherNet/IP (ENIP) pure-core parser, command classifier, and frame validity gate.
//!
//! Subsystem SS-17, CAP-17 — `analyzer/enip.rs`.
//!
//! ## Architecture (ADR-010 Decision 2)
//!
//! All functions in this module are **pure-core free `fn`s** — no `self`, no I/O, no global
//! state mutation. This is a hard constraint for VP-032 Kani formal verification validity.
//!
//! - `parse_enip_header` — 24-byte LE header parse; None for <24 bytes (BC-2.17.001/002)
//! - `classify_enip_command` — total classification over all 65,536 u16 command values
//!   (BC-2.17.004; VP-032 Sub-B)
//! - `is_valid_enip_frame` — biconditional gate against 9-value ODVA known-command set
//!   (BC-2.17.003; VP-032 Sub-C)
//!
//! ## Byte order
//! All multi-byte fields in the ENIP encapsulation header are **little-endian** per ODVA
//! EtherNet/IP specification and ADR-010 Decision 1. Use `u16::from_le_bytes` /
//! `u32::from_le_bytes` — never `from_be_bytes`.
//!
//! ## VP-032 Kani harnesses
//! Sub-A through Sub-C harnesses live in `#[cfg(kani)] mod kani_proofs` below.
//! Sub-D (`vp032_cip_service_classification_totality`) is in scope for STORY-132.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Data types — pure-core (STORY-130)
// ---------------------------------------------------------------------------

/// Parsed EtherNet/IP encapsulation header (fixed 24-byte layout, all LE fields).
///
/// Field offsets per ODVA EtherNet/IP Specification Table 2-4 and ADR-010 Decision 2:
/// - `command`        bytes  0–1   `u16::from_le_bytes` (BC-2.17.002 postcondition 2)
/// - `length`         bytes  2–3   `u16::from_le_bytes` (BC-2.17.002 postcondition 3)
/// - `session_handle` bytes  4–7   `u32::from_le_bytes` (BC-2.17.002 postcondition 4)
/// - `status`         bytes  8–11  `u32::from_le_bytes` (BC-2.17.002 postcondition 5)
/// - `sender_context` bytes 12–19  `[u8; 8]` verbatim copy (BC-2.17.002 postcondition 6)
/// - `options`        bytes 20–23  `u32::from_le_bytes` (BC-2.17.002 postcondition 7)
///
/// `sender_context` is opaque — copied verbatim as `[u8; 8]`, NOT decoded as a number.
/// (ADR-010 Decision 1; BC-2.17.002 invariant 3)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnipHeader {
    /// ENIP encapsulation command code (bytes 0–1, LE).
    pub command: u16,
    /// Payload byte count after the 24-byte header (bytes 2–3, LE).
    pub length: u16,
    /// Session handle, 0 for commands that do not require registration (bytes 4–7, LE).
    pub session_handle: u32,
    /// Encapsulation status; 0x00000000 = success (bytes 8–11, LE).
    pub status: u32,
    /// 8-byte opaque sender context, copied verbatim (bytes 12–19).
    pub sender_context: [u8; 8],
    /// Options field; must be 0x00000000 in standard implementations (bytes 20–23, LE).
    pub options: u32,
}

/// EtherNet/IP encapsulation command classification.
///
/// Exactly 10 variants: 9 named ODVA commands + `Unknown` catch-all.
/// (BC-2.17.004 invariant 1; VP-032 Sub-B totality target)
///
/// The set of non-Unknown variants is identical to the known-command set used by
/// `is_valid_enip_frame` (BC-2.17.004 invariant 2 — these two must stay in sync).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnipCommandClass {
    /// 0x0004 — List services available from the target.
    ListServices,
    /// 0x0063 — List identity objects (recon; T0846 detection path).
    ListIdentity,
    /// 0x0064 — List available interfaces.
    ListInterfaces,
    /// 0x0065 — Register a new session with the target.
    RegisterSession,
    /// 0x0066 — Close/unregister a session.
    UnRegisterSession,
    /// 0x006F — Send explicit messaging request/reply (primary CIP path).
    SendRRData,
    /// 0x0070 — Send implicit/connected data.
    SendUnitData,
    /// 0x0072 — Indicate status (I/O scanner heartbeat).
    IndicateStatus,
    /// 0x0075 — Cancel pending request.
    Cancel,
    /// All other u16 values — non-ODVA or unassigned command codes.
    Unknown,
}

// ---------------------------------------------------------------------------
// Pure-core free functions (VP-032 Kani targets)
// ---------------------------------------------------------------------------

/// Parse the 24-byte EtherNet/IP encapsulation header from a byte slice.
///
/// Returns `Some(EnipHeader)` if `data.len() >= 24`; returns `None` for any shorter
/// input without accessing any bytes (BC-2.17.001 postcondition 1–3).
///
/// All multi-byte fields are decoded little-endian per ODVA spec (ADR-010 Decision 2).
/// Bytes beyond index 23 are not read — the caller (frame-walk loop) handles them
/// (BC-2.17.002 postcondition 8).
///
/// # Panics
/// Never panics for any input (VP-032 Sub-A safety contract).
///
/// # Traces
/// BC-2.17.001, BC-2.17.002; VP-032 Sub-A Kani target.
pub fn parse_enip_header(data: &[u8]) -> Option<EnipHeader> {
    if data.len() < 24 {
        return None;
    }
    Some(EnipHeader {
        command: u16::from_le_bytes([data[0], data[1]]),
        length: u16::from_le_bytes([data[2], data[3]]),
        session_handle: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        status: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        sender_context: [
            data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19],
        ],
        options: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
    })
}

/// Classify a u16 EtherNet/IP command code into an `EnipCommandClass` variant.
///
/// This function is **total** — it returns a valid variant for every possible `u16`
/// input and never panics (BC-2.17.004 postcondition 3; VP-032 Sub-B).
///
/// The `Unknown` arm is reachable and non-vacuous: `classify_enip_command(0x0000)`
/// returns `EnipCommandClass::Unknown` (BC-2.17.004 postcondition 4; DF-KANI-NONVACUITY-001).
///
/// # Traces
/// BC-2.17.004; VP-032 Sub-B Kani target.
pub fn classify_enip_command(command: u16) -> EnipCommandClass {
    match command {
        0x0004 => EnipCommandClass::ListServices,
        0x0063 => EnipCommandClass::ListIdentity,
        0x0064 => EnipCommandClass::ListInterfaces,
        0x0065 => EnipCommandClass::RegisterSession,
        0x0066 => EnipCommandClass::UnRegisterSession,
        0x006F => EnipCommandClass::SendRRData,
        0x0070 => EnipCommandClass::SendUnitData,
        0x0072 => EnipCommandClass::IndicateStatus,
        0x0075 => EnipCommandClass::Cancel,
        _ => EnipCommandClass::Unknown,
    }
}

/// Validity gate: returns `true` iff `h.command` is in the 9-value ODVA known-command set.
///
/// The biconditional holds for all 65,536 possible `u16` command values (BC-2.17.003 invariant 1).
/// Only `h.command` is inspected — `h.length`, `h.status`, `h.session_handle`, and `h.options`
/// are NOT gate criteria (BC-2.17.003 postcondition 3).
///
/// Known-command set: {0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075}
///
/// # Panics
/// Never panics for any input (VP-032 Sub-C safety contract).
///
/// # Traces
/// BC-2.17.003; VP-032 Sub-C Kani target.
pub fn is_valid_enip_frame(h: &EnipHeader) -> bool {
    matches!(
        h.command,
        0x0004 | 0x0063 | 0x0064 | 0x0065 | 0x0066 | 0x006F | 0x0070 | 0x0072 | 0x0075
    )
}

// ---------------------------------------------------------------------------
// Aggregate analyzer struct (STORY-131 — BC-2.17.019/020/023/026)
// ---------------------------------------------------------------------------

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;

/// EtherNet/IP stream analyzer aggregate.
///
/// Receives reassembled TCP bytes for port-44818 flows (via `StreamDispatcher`
/// Rule 7 after STORY-131) and accumulates detection findings.
///
/// Threshold fields are populated from CLI flags (BC-2.17.023 / BC-2.17.026):
/// - `enip_write_burst_threshold` — T0836 write-burst threshold (default 50).
/// - `enip_error_burst_threshold` — T0888 error-burst threshold (default 5).
///
/// Detection logic (frame-walk, CIP parse, MITRE detections) is added by
/// STORY-132–137. This stub carries the structural skeleton only.
///
/// BC-2.17.019 §P2–P3 / BC-2.17.020 §P1 / BC-2.17.023 §P1 / BC-2.17.026 §P1.
pub struct EnipAnalyzer {
    /// Write-burst threshold for T0836 detection (BC-2.17.023 / OA-001 RESOLVED=50).
    pub enip_write_burst_threshold: u32,
    /// Error-burst threshold for T0888 Pattern B detection (BC-2.17.026 default=5).
    pub enip_error_burst_threshold: u32,
    /// Accumulated findings — populated by detection logic (STORY-132+).
    pub all_findings: Vec<Finding>,
    /// Total reassembled TCP bytes received across all port-44818 flows.
    ///
    /// Observable for BC-2.17.019 PC-2 integration tests (STORY-131 boundary decision):
    /// `bytes_received > 0` after `dispatcher.on_data()` confirms the wiring arm fired.
    /// Incremented by `on_data` (STORY-131 implementer wires this). Stable across
    /// STORY-131 → STORY-132: STORY-132 adds frame-walk alongside this counter.
    // STORY-131 implementer wires this
    #[allow(dead_code)]
    pub bytes_received: u64,
}

impl EnipAnalyzer {
    /// Construct a new `EnipAnalyzer` with the given threshold values.
    ///
    /// `write_burst_threshold` — T0836 write-burst cap (CLI `--enip-write-burst-threshold`,
    /// default 50, BC-2.17.023 Invariant 1).
    /// `error_burst_threshold` — T0888 error-burst cap (CLI `--enip-error-burst-threshold`,
    /// default 5, BC-2.17.026 Invariant 1).
    ///
    /// WIRING-EXEMPT: constructor assigns two scalar fields and initialises one Vec to empty.
    /// Zero branching; no I/O; no non-trivial helpers; ≤ 3 meaningful lines of struct-init.
    pub fn new(write_burst_threshold: u32, error_burst_threshold: u32) -> Self {
        Self {
            enip_write_burst_threshold: write_burst_threshold,
            enip_error_burst_threshold: error_burst_threshold,
            all_findings: Vec::new(),
            bytes_received: 0,
        }
    }

    /// Produce an end-of-capture summary for the ENIP analyzer.
    ///
    /// Real metrics (frames parsed, detection counts, per-flow stats) are
    /// populated by STORY-132+. This stub returns a minimal shell that
    /// prevents the reporter pipeline from panicking when --enip/--all is used
    /// before the detection logic lands.
    ///
    /// WIRING-EXEMPT: required by the reporter pipeline contract. Without a
    /// non-panicking summarize(), the existing `--all` CLI test regresses.
    /// Body constructs one AnalysisSummary with zero fields — no branching,
    /// no I/O, no non-trivial helpers, ≤ 3 lines.
    pub fn summarize(&self) -> AnalysisSummary {
        use std::collections::BTreeMap;
        AnalysisSummary {
            analyzer_name: "EtherNet/IP".to_string(),
            packets_analyzed: 0,
            detail: BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// VP-032 Kani formal verification harnesses (Sub-A, Sub-B, Sub-C)
// Sub-D (vp032_cip_service_classification_totality) is added by STORY-132.
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-032 Sub-A: parse_enip_header never panics; returns None for <24 bytes;
    /// returns Some with correct field layout for >=24 bytes.
    ///
    /// BOUND/SOUNDNESS: 48-byte bound (2x minimum header) covers all length
    /// conditions; behavior is identical for any longer slice (fixed 24-byte read).
    /// Non-vacuity: both Some and None branches are reachable in the symbolic range.
    #[kani::proof]
    #[kani::unwind(49)]
    fn vp032_enip_header_parse_safety() {
        const BOUND: usize = 48;
        let data: [u8; BOUND] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= BOUND);
        let slice = &data[..len];
        let result = parse_enip_header(slice);
        if len < 24 {
            // BC-2.17.001 postcondition 1: must return None for any len < 24
            assert!(result.is_none());
        } else {
            // BC-2.17.002 postconditions 2/3/5: field offsets at fixed LE positions
            let h = result.expect("must be Some for len >= 24");
            let expected_cmd = u16::from_le_bytes([slice[0], slice[1]]);
            assert_eq!(h.command, expected_cmd);
            let expected_len = u16::from_le_bytes([slice[2], slice[3]]);
            assert_eq!(h.length, expected_len);
            let expected_status = u32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]);
            assert_eq!(h.status, expected_status);
        }
    }

    /// VP-032 Sub-B: classify_enip_command(cmd) == Unknown iff cmd is not in KNOWN_COMMANDS.
    /// Biconditional simultaneously proves totality, Unknown reachability, and named-variant
    /// reachability (DF-KANI-NONVACUITY-001). No kani::assume on cmd.
    #[kani::proof]
    fn vp032_enip_command_classification_biconditional() {
        const KNOWN_COMMANDS: &[u16] = &[
            0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075,
        ];
        let cmd: u16 = kani::any();
        let is_unknown = matches!(classify_enip_command(cmd), EnipCommandClass::Unknown);
        let not_in_known = !KNOWN_COMMANDS.contains(&cmd);
        assert_eq!(is_unknown, not_in_known);
    }

    /// VP-032 Sub-C: is_valid_enip_frame iff h.command is in the known-command set.
    /// Biconditional proven for all 65,536 u16 command values.
    #[kani::proof]
    fn vp032_enip_validity_gate_biconditional() {
        let cmd: u16 = kani::any();
        let h = EnipHeader {
            command: cmd,
            length: 0,
            session_handle: 0,
            status: 0,
            sender_context: [0u8; 8],
            options: 0,
        };
        let known_cmds: &[u16] = &[
            0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075,
        ];
        let is_known = known_cmds.contains(&cmd);
        let gate_result = is_valid_enip_frame(&h);
        assert_eq!(gate_result, is_known);
    }
}
