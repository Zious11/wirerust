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
// Module-level constant
// ---------------------------------------------------------------------------

/// Maximum number of findings accumulated per `EnipAnalyzer` instance.
///
/// Mirrors `dnp3::MAX_FINDINGS` (10_000) — consistent DoS cap across analyzers
/// (BC-2.17.022; ADR-010 Decision 4). Every `all_findings.push` is gated on
/// `all_findings.len() < MAX_FINDINGS`.
pub const MAX_FINDINGS: usize = 10_000;

/// Maximum number of bytes the carry buffer may hold after any `on_data` call.
///
/// When `flow.carry.len() > MAX_ENIP_CARRY_BYTES` after the frame-walk loop stashes
/// remaining bytes, the carry-overflow path fires: `parse_errors += 1`,
/// `malformed_in_window += 1`, `check_t0814()` (before latch), `is_non_enip = true`,
/// `carry.clear()` (BC-2.17.016 Invariant 1 / Postcondition 4 / Invariant 4).
///
/// NOT configurable — hard cap per ADR-010 Decision 3/4.
///
/// Traces: BC-2.17.016 Invariant 1 / Invariant 4; ADR-010 Decision 3.
pub const MAX_ENIP_CARRY_BYTES: usize = 600;

/// Number of malformed frames within the 300-second window that triggers a T0814 finding.
///
/// Compile-time constant — NOT CLI-configurable (ADR-010 Decision 5).
/// The windowed `malformed_in_window` counter (reset every 300 s) is compared against
/// this threshold by `check_t0814`. Once the threshold is reached and
/// `malformed_anomaly_emitted == false`, a single T0814 Anomaly/Possible/Low finding
/// is emitted and `malformed_anomaly_emitted` is latched true for the remainder of the window.
///
/// Traces: BC-2.17.018 Invariant 1 / Postcondition 3; ADR-010 Decision 5.
pub const MALFORMED_ANOMALY_THRESHOLD: u64 = 3;

use std::collections::HashMap;
use std::net::IpAddr;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;

// ---------------------------------------------------------------------------
// Per-flow state (STORY-134 — BC-2.17.008/010/014/016)
// ---------------------------------------------------------------------------

/// Per-flow mutable state for the EtherNet/IP analyzer.
///
/// Carries the carry buffer, error window fields, per-command counts, and
/// detection guards used across all EtherNet/IP detection stories.
///
/// **Field names are normative** — BCs reference these exact identifiers:
/// - `error_counts_in_window` — BC-2.17.008 invariant; must NOT be aliased.
/// - `error_window_start_ts`  — BC-2.17.008 postcondition 2 canonical field name.
/// - `error_rate_emitted`     — BC-2.17.008 invariant; T0888 Pattern B one-shot guard.
/// - `list_identity_emitted`  — BC-2.17.010 invariant 1; T0846 per-flow one-shot guard.
/// - `is_non_enip`            — BC-2.17.016 carry-buffer overflow latch; set by STORY-137.
///
/// # Architecture
/// Allocated in `EnipAnalyzer::flows: HashMap<FlowKey, EnipFlowState>` (STORY-137).
/// For STORY-134 tests, construct directly to verify detection helpers without
/// depending on the frame-walk wiring (STORY-137 scope).
///
/// Traces: BC-2.17.008, BC-2.17.010, BC-2.17.014, BC-2.17.016; ADR-010 Decision 4.
pub struct EnipFlowState {
    /// Per-status CIP error counts within the current 10-second window.
    ///
    /// Keyed by `general_status` byte (byte 2 of 0x00B2 CIP response).
    /// Cleared on window expiry (BC-2.17.008 postcondition 4).
    /// Field name is normative per BC-2.17.008 invariant.
    pub error_counts_in_window: HashMap<u8, u64>,

    /// Timestamp (pcap-relative seconds, u32) of the first error in the current window.
    ///
    /// Seeded on the first error response in a new window (BC-2.17.008 postcondition 2).
    /// Valid only when `error_window_active` is `true`.
    /// Field name is normative per BC-2.17.008 postcondition 2.
    pub error_window_start_ts: u32,

    /// Active flag for the 10-second error window (BC-2.17.008).
    ///
    /// `false` until the first qualifying CIP error response arrives; thereafter `true`.
    /// Guards the window-expiry branch so that timestamp==0 is a valid window-start
    /// value, not a sentinel for "no error seen yet" (fixes F-134-001).
    /// Reset to `false` is NOT performed on expiry — the window is reseeded, not closed.
    pub error_window_active: bool,

    /// One-shot guard for T0888 Pattern B (error-burst finding per window).
    ///
    /// Set to `true` when the Pattern B finding is emitted for the current window;
    /// reset to `false` on window expiry (BC-2.17.008 postcondition 4).
    /// Field name is normative per BC-2.17.008 invariant.
    pub error_rate_emitted: bool,

    /// SetAttribute request count in the current 1-second write-burst window.
    ///
    /// Incremented on every SetAttributesAll (0x02), SetAttributeList (0x04), or
    /// SetAttributeSingle (0x10) request via a 0x00B2 item (BC-2.17.012 postcondition 1).
    /// Reset to 1 (seeding the new window with the current write) on 1s window expiry
    /// (BC-2.17.012 postcondition 4). Field name is normative per BC-2.17.012 Architecture Anchors.
    pub write_count_in_window: u64,

    /// One-shot guard for T0836 write-burst finding (per window).
    ///
    /// Set to `true` when the T0836 finding is emitted for the current 1s window;
    /// reset to `false` on window expiry (BC-2.17.012 postcondition 4).
    /// Field name is normative per BC-2.17.012 Architecture Anchors.
    pub write_burst_emitted: bool,

    /// Timestamp (pcap-relative seconds, u32) of the start of the current 1s write-burst window.
    ///
    /// Seeded on the first write-class request in a new window (BC-2.17.012 postcondition 3).
    /// Uses u32 wrapping_sub arithmetic — same pattern as `error_window_start_ts`.
    /// Field name is normative per BC-2.17.012 Architecture Anchors / postcondition 3.
    pub write_window_start_ts: u32,

    /// One-shot guard for T0846 ListIdentity finding (per-flow).
    ///
    /// Set to `true` on the first ListIdentity frame per flow; subsequent frames
    /// on the same flow increment `command_counts[0x0063]` but do NOT emit additional
    /// T0846 findings (BC-2.17.010 invariant 1).
    pub list_identity_emitted: bool,

    /// Non-ENIP latch: when `true`, all `on_data` calls for this flow are immediate no-ops.
    ///
    /// Set by the carry-buffer overflow logic in STORY-137 (BC-2.17.016 postcondition 4).
    /// In STORY-134 tests, construct flows with `is_non_enip: true` directly to verify
    /// detection suppression without running the frame-walk.
    pub is_non_enip: bool,

    /// Per-command ENIP packet counts for this flow.
    ///
    /// Incremented in the frame-walk loop (`on_data`, STORY-137) at PC-0 (BC-2.17.016)
    /// immediately after `parse_enip_header` returns `Some`, before `is_valid_enip_frame`.
    /// STORY-134's `process_pdu` MUST NOT increment this counter (single-increment rule
    /// BC-2.17.024/025 / BC-2.17.016 invariant 6).
    pub command_counts: HashMap<u16, u64>,

    /// Number of successfully dispatched PDUs for this flow.
    ///
    /// Incremented inside `process_pdu` only (BC-2.17.024). Distinct from `command_counts`
    /// which is incremented in the frame-walk regardless of frame validity.
    pub pdu_count: u64,

    /// Per-flow structural parse error count (lifetime — never reset at window expiry).
    ///
    /// Incremented on invalid/oversized frames and carry-buffer overflow (BC-2.17.016).
    pub parse_errors: u64,

    /// Windowed malformed frame counter (reset at window expiry per BC-2.17.018).
    pub malformed_in_window: u64,

    /// Carry buffer for partial ENIP frames (BC-2.17.016).
    ///
    /// Bounded to `MAX_ENIP_CARRY_BYTES = 600`. Overflow triggers `is_non_enip = true`.
    /// Managed exclusively by `on_data` (STORY-137).
    pub carry: Vec<u8>,

    /// ForwardOpen + LargeForwardOpen request count (per-flow, lifetime).
    ///
    /// Incremented on every `CipServiceClass::ForwardOpen` or `CipServiceClass::LargeForwardOpen`
    /// request via a 0x00B2 item and `!is_non_enip`, regardless of the MAX_FINDINGS cap
    /// (EC-008 / BC-2.17.015 Architecture Rule 4). Read by STORY-138 session summary.
    /// Field name is normative per BC-2.17.015 Architecture Mapping.
    pub open_connection_count: u32,

    /// ForwardClose request count (per-flow, lifetime).
    ///
    /// Incremented on every `CipServiceClass::ForwardClose` request via a 0x00B2 item
    /// and `!is_non_enip`, regardless of the MAX_FINDINGS cap (EC-008 / BC-2.17.015
    /// Architecture Rule 4). Read by STORY-138 session summary.
    /// Field name is normative per BC-2.17.015 Architecture Mapping.
    pub close_connection_count: u32,

    /// One-shot guard for T0814 per-window emission (BC-2.17.018 Postcondition 4).
    ///
    /// Set to `true` when the T0814 finding is emitted for the current 300-second window;
    /// reset to `false` on window expiry along with `malformed_in_window`. Prevents a second
    /// T0814 finding from firing within the same window when the threshold is exceeded again.
    /// Field name is normative per BC-2.17.018 Architecture Mapping.
    pub malformed_anomaly_emitted: bool,

    /// Timestamp (pcap-relative seconds, u32) of the start of the current 300-second malformed
    /// frame detection window (BC-2.17.018 Postcondition 5).
    ///
    /// Seeded to `now_ts` on the first `on_data` call for a flow (initialized to 0 in
    /// `EnipFlowState::new()`). Window expiry is evaluated at the top of each `on_data` call:
    /// when `now_ts - malformed_window_start >= 300`, the window resets:
    /// `malformed_in_window = 0`, `malformed_anomaly_emitted = false`,
    /// `malformed_window_start = now_ts`.
    /// Field name is normative per BC-2.17.018 Architecture Mapping.
    pub malformed_window_start: u32,
}

impl EnipFlowState {
    /// Construct a new default `EnipFlowState` with all counters zeroed and maps empty.
    ///
    /// WIRING-EXEMPT: zero-initialiser constructor — no branching, no I/O, no helpers,
    /// ≤ 3 lines of struct init. All fields initialise to their zero/empty values.
    pub fn new() -> Self {
        Self {
            error_counts_in_window: HashMap::new(),
            error_window_start_ts: 0,
            error_window_active: false,
            error_rate_emitted: false,
            list_identity_emitted: false,
            is_non_enip: false,
            command_counts: HashMap::new(),
            pdu_count: 0,
            parse_errors: 0,
            malformed_in_window: 0,
            carry: Vec::new(),
            write_count_in_window: 0,
            write_burst_emitted: false,
            write_window_start_ts: 0,
            open_connection_count: 0,
            close_connection_count: 0,
            malformed_anomaly_emitted: false,
            malformed_window_start: 0,
        }
    }
}

impl Default for EnipFlowState {
    /// Delegates to `EnipFlowState::new()`.
    ///
    /// WIRING-EXEMPT: single delegation call, no branching, no I/O.
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// T0814 windowed DoS detection helper (STORY-137 — BC-2.17.018)
// ---------------------------------------------------------------------------

/// Check whether the windowed malformed-frame threshold has been crossed and, if so,
/// emit a single T0814 Anomaly/Possible/Low finding into `all_findings`.
///
/// Called from the frame-walk loop in `on_data` after every structural reject (invalid
/// command, oversized declared frame, carry-buffer overflow) and at the carry-cap check.
///
/// **Conditional emission (BC-2.17.018 Postconditions 3–4):** fires when ALL of:
/// - `flow.malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD` (= 3)
/// - `flow.malformed_anomaly_emitted == false`
/// - `flow.is_non_enip == false`  — MUST be evaluated BEFORE `is_non_enip` is latched
///   on carry-overflow (BC-2.17.018 Precondition 6 / EC-007; ordering enforced in `on_data`)
/// - `all_findings.len() < MAX_FINDINGS`
///
/// Sets `flow.malformed_anomaly_emitted = true` after emission (one-shot per window).
/// Does NOT set `flow.is_non_enip` — that is the exclusive domain of the carry-cap check
/// in `on_data` (BC-2.17.016 Invariant 4).
///
/// # Parameters
/// - `flow` — mutable per-flow state; reads `malformed_in_window`,
///   `malformed_anomaly_emitted`, `is_non_enip`; writes `malformed_anomaly_emitted`.
/// - `all_findings` — analyzer-level findings accumulator; length compared against `MAX_FINDINGS`.
/// - `now_ts` — pcap-relative capture timestamp (seconds, u32); included in the finding.
/// - `src_ip` — source IP of the offending flow; included in the finding summary and field.
/// - `dest_ip` — destination IP; included in the finding summary for flow identification.
///
/// # Traces
/// BC-2.17.018 Postconditions 3–4; Invariant 1; Precondition 6; EC-007.
pub fn check_t0814(
    _flow: &mut EnipFlowState,
    _all_findings: &mut Vec<crate::findings::Finding>,
    _now_ts: u32,
    _src_ip: std::net::IpAddr,
    _dest_ip: std::net::IpAddr,
) {
    todo!("STORY-137: T0814 windowed DoS detection [BC-2.17.018]");
}

// ---------------------------------------------------------------------------
// Aggregate analyzer struct (STORY-131 — BC-2.17.019/020/023/026)
// ---------------------------------------------------------------------------

/// EtherNet/IP stream analyzer aggregate.
///
/// Receives reassembled TCP bytes for port-44818 flows (via `StreamDispatcher`
/// Rule 7, wired in STORY-131) and accumulates detection findings.
///
/// Threshold fields are populated from CLI flags (BC-2.17.023 / BC-2.17.026):
/// - `enip_write_burst_threshold` — T0836 write-burst threshold (default 50).
/// - `enip_error_burst_threshold` — T0888 error-burst threshold (default 5).
///
/// Detection logic (CIP parse, MITRE detections) is added by STORY-134+.
/// Frame-walk wiring in `on_data` is added by STORY-137 (BC-2.17.016).
///
/// BC-2.17.019 §P2–P3 / BC-2.17.020 §P1 / BC-2.17.023 §P1 / BC-2.17.026 §P1.
pub struct EnipAnalyzer {
    /// Write-burst threshold for T0836 detection (BC-2.17.023 / OA-001 RESOLVED=50).
    pub enip_write_burst_threshold: u32,
    /// Error-burst threshold for T0888 Pattern B detection (BC-2.17.026 default=5).
    pub enip_error_burst_threshold: u32,
    /// Accumulated findings — populated by detection logic (STORY-134+).
    pub all_findings: Vec<Finding>,
    /// Total reassembled TCP bytes received across all port-44818 flows.
    ///
    /// Observable for BC-2.17.019 PC-2 integration tests (STORY-131 boundary decision):
    /// `bytes_received > 0` after `dispatcher.on_data()` confirms the wiring arm fired.
    /// Incremented by `on_data` via saturating_add.
    pub bytes_received: u64,
    /// Aggregate lifetime count of CIP error responses (general_status != 0x00).
    ///
    /// Incremented by `process_pdu` on every qualifying error response (BC-2.17.008
    /// Postcondition 2b / Invariant 2). Never reset across flows or windows.
    /// Read by `summarize()` (BC-2.17.021 postcondition 1 `error_count` field).
    pub error_count: u64,

    /// Aggregate lifetime count of CIP write-class service requests (SetAttribute*).
    ///
    /// Incremented on every qualifying SetAttributesAll (0x02), SetAttributeList (0x04),
    /// or SetAttributeSingle (0x10) request via a 0x00B2 item (BC-2.17.012 postcondition 2).
    /// Never reset. Read by `summarize()` per BC-2.17.021 postcondition 1 `write_count` field.
    /// Field name is normative per BC-2.17.012 Architecture Anchors.
    pub write_count: u64,
}

impl EnipAnalyzer {
    /// Construct a new `EnipAnalyzer` with the given threshold values.
    ///
    /// `write_burst_threshold` — T0836 write-burst cap (CLI `--enip-write-burst-threshold`,
    /// default 50, BC-2.17.023 Invariant 1).
    /// `error_burst_threshold` — T0888 error-burst cap (CLI `--enip-error-burst-threshold`,
    /// default 5, BC-2.17.026 Invariant 1).
    ///
    /// WIRING-EXEMPT: constructor assigns scalar fields and initialises collections to empty.
    /// Zero branching; no I/O; no non-trivial helpers; ≤ 3 meaningful lines of struct-init.
    pub fn new(write_burst_threshold: u32, error_burst_threshold: u32) -> Self {
        Self {
            enip_write_burst_threshold: write_burst_threshold,
            enip_error_burst_threshold: error_burst_threshold,
            all_findings: Vec::new(),
            bytes_received: 0,
            error_count: 0,
            write_count: 0,
        }
    }

    /// Receive reassembled TCP bytes for a port-44818 flow and run the ENIP frame-walk loop.
    ///
    /// Entry point for all per-flow stream data. Implements:
    /// 1. `bytes_received` increment (WIRING-EXEMPT routing confirmation observable,
    ///    BC-2.17.019 PC-2; dispatcher wiring contract from STORY-131).
    /// 2. `is_non_enip` early-exit guard — flows permanently quarantined by carry-buffer
    ///    overflow are immediate no-ops after the bytes_received increment (BC-2.17.016 Inv 4).
    /// 3. 300-second malformed window expiry check and reset (BC-2.17.018 Postcondition 5).
    /// 4. Carry + new-data concatenation (BC-2.17.016 Postcondition 2).
    /// 5. Frame-walk loop (BC-2.17.016 Postcondition 1 / ADR-010 Decision 4):
    ///    - `parse_enip_header` + `command_counts` increment (PC-0, canonical single site)
    ///    - `is_valid_enip_frame` gate → byte-walk resync on unknown command (cursor += 1)
    ///    - Oversized declared frame → frame-skip path (cursor += min(total, remaining))
    ///    - Partial frame → stash into carry; break
    ///    - Valid complete frame → `process_pdu`; cursor += total_frame_len
    /// 6. Carry stash after loop (BC-2.17.016 Postcondition 3).
    /// 7. Carry-cap check (BC-2.17.016 Postcondition 4 / Invariant 4):
    ///    - `parse_errors += 1`; `malformed_in_window += 1`
    ///    - `check_t0814()` ← MUST run while `is_non_enip == false` (BC-2.17.018 Precond 6)
    ///    - `is_non_enip = true`; `carry.clear()`
    ///
    /// WIRING-EXEMPT (bytes_received line only): single saturating_add, no branching, no I/O.
    /// The remainder of this function is a STORY-137 todo!() stub that establishes the Red Gate.
    ///
    /// # Parameters
    /// - `flow_key`  — TCP flow identifier; used to look up / insert `EnipFlowState`.
    /// - `data`      — reassembled TCP bytes for this flow segment.
    /// - `timestamp` — pcap-relative capture timestamp (seconds, u32).
    ///
    /// # Traces
    /// BC-2.17.016 (frame-walk algorithm); BC-2.17.018 (T0814 windowed detection);
    /// BC-2.17.004 Inv-3 (command_counts single increment site);
    /// BC-2.17.019 §P2 (routing confirmation); AC-131-001 (bytes_received observable).
    pub fn on_data(
        &mut self,
        _flow_key: crate::reassembly::flow::FlowKey,
        data: &[u8],
        _timestamp: u32,
    ) {
        // WIRING-EXEMPT: bytes_received increment is the routing-confirmation observable for
        // STORY-131 dispatcher tests (BC-2.17.019 PC-2). Single saturating_add; no branching;
        // no I/O; no non-trivial helpers; 1 line. GREEN-BY-DESIGN for existing dispatch tests.
        self.bytes_received = self.bytes_received.saturating_add(data.len() as u64);

        // BC-5.38.005 self-check invariant 1:
        // "If I include this real implementation, will the test for this function pass trivially
        // without any implementer work?"
        // Answer for the bytes_received line: YES, but only for dispatch routing tests (AC-131-*),
        // not for any frame_walk tests. The dispatch tests assert bytes_received > 0 after
        // on_data() — that assertion is satisfied by the saturating_add above alone, independent
        // of the frame-walk implementation. The frame_walk tests assert carry-buffer state,
        // parse_errors, malformed_in_window, command_counts, pdu_count, and findings — none of
        // which are touched by the saturating_add. All 21 frame_walk tests will hit the todo!()
        // below and FAIL (Red Gate satisfied). WIRING-EXEMPT classification: bytes_received line
        // is pure routing-confirmation wiring — identical to take_enip_analyzer() and other
        // STORY-131 WIRING-EXEMPT bodies; it is not business logic.

        // STORY-137 frame-walk loop — todo!() body; Red Gate per BC-5.38.001.
        // The implementer replaces this todo!() with:
        //   1. is_non_enip guard (return immediately if true)
        //   2. 300-second window expiry check/reset
        //   3. buf = carry.clone() + data
        //   4. while buf.len() - cursor >= 24 { ... }
        //   5. carry stash after loop
        //   6. carry-cap check (parse_errors, malformed_in_window, check_t0814, is_non_enip, carry.clear())
        // Per BC-2.17.016 pseudocode (STORY-137 lines 199-271).
        todo!("STORY-137: frame-walk carry buffer [BC-2.17.016]");
    }

    /// Main per-PDU detection dispatch for a single validated ENIP frame.
    ///
    /// Called by the frame-walk loop in `on_data` (STORY-137 / BC-2.17.016) for every
    /// frame that passes `is_valid_enip_frame`. This function owns all MITRE detection
    /// logic for STORY-134:
    ///
    /// 1. `is_non_enip` gate — exit immediately if `flow.is_non_enip == true`.
    /// 2. `classify_enip_command` → ListIdentity → T0846 (BC-2.17.010).
    /// 3. For SendRRData/SendUnitData: walk CPF items; for `type_id == 0x00B2`:
    ///    a. `parse_cip_header` + `classify_cip_service`.
    ///    b. If request + GetAttribute + Class(0x01) in path → T0888 Pattern A (BC-2.17.014).
    ///    c. If response + `general_status != 0x00` → accumulate error; check Pattern B
    ///    (BC-2.17.008 + BC-2.17.014 Pattern B).
    ///
    /// **SINGLE-INCREMENT NOTE (BC-2.17.024/025):** `process_pdu` MUST NOT touch
    /// `flow.command_counts`. The sole canonical `command_counts` increment site is
    /// the frame-walk loop in `on_data` (BC-2.17.016 PC-0, STORY-137).
    ///
    /// # Parameters
    /// - `flow_key` — identifies the TCP flow; used to look up `EnipFlowState`.
    /// - `pdu`      — complete ENIP frame bytes (header + payload, 24 + header.length bytes).
    /// - `timestamp` — pcap-relative capture timestamp (seconds, u32).
    /// - `src_ip`   — source IP address of the sending endpoint.
    ///
    /// # Traces
    /// BC-2.17.008, BC-2.17.010, BC-2.17.014; ADR-010 Decision 4 (frame-walk / detection order).
    pub fn process_pdu(
        &mut self,
        flow: &mut EnipFlowState,
        pdu: &[u8],
        timestamp: u32,
        src_ip: IpAddr,
    ) {
        // ADR-010 Decision 4, step 1: is_non_enip gate — suppress all detection on
        // flows flagged as non-ENIP (BC-2.17.010 precondition 2; BC-2.17.014 precondition 5).
        if flow.is_non_enip {
            return;
        }

        // ADR-010 Decision 4, step 1: parse ENIP header; silently drop frames < 24 bytes.
        let header = match parse_enip_header(pdu) {
            Some(h) => h,
            None => return,
        };

        // ADR-010 Decision 4, step 2: classify command; T0846 ListIdentity detection
        // (BC-2.17.010). SINGLE-INCREMENT NOTE: command_counts is NOT touched here;
        // that increment belongs to the frame-walk in on_data (STORY-137, BC-2.17.016 PC-0).
        let cmd_class = classify_enip_command(header.command);
        if matches!(cmd_class, EnipCommandClass::ListIdentity) {
            // BC-2.17.010 postcondition 2: one-shot guard + MAX_FINDINGS gate.
            if !flow.list_identity_emitted && self.all_findings.len() < MAX_FINDINGS {
                self.all_findings.push(Finding {
                    category: crate::findings::ThreatCategory::Reconnaissance,
                    verdict: crate::findings::Verdict::Likely,
                    confidence: crate::findings::Confidence::High,
                    summary: "EtherNet/IP ListIdentity broadcast observed: \
                              network-wide device enumeration (T0846)"
                        .to_string(),
                    evidence: vec![format!(
                        "ENIP command=0x0063 (ListIdentity) src={src_ip} \
                         session={session}",
                        session = header.session_handle
                    )],
                    mitre_techniques: vec!["T0846".to_string()],
                    source_ip: Some(src_ip),
                    timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0),
                    direction: None,
                });
                // BC-2.17.010 postcondition 2 last line: set one-shot guard only after
                // successful push.
                flow.list_identity_emitted = true;
            }
            // BC-2.17.010 postcondition 3: ListIdentity frames after guard set produce no
            // additional finding. Return after ENIP-layer detection; no CPF parse needed.
            return;
        }

        // ADR-010 Decision 4, steps 3–6: for SendRRData/SendUnitData, walk CPF items.
        // CPF data starts at pdu[30..]: ENIP header (24) + Interface Handle (4) + Timeout (2).
        if !matches!(
            cmd_class,
            EnipCommandClass::SendRRData | EnipCommandClass::SendUnitData
        ) {
            return;
        }

        // BC-2.17.005: CPF payload begins after the 6-byte SendRRData-specific header
        // (Interface Handle u32 + Timeout u16) that follows the 24-byte ENIP header.
        const CPF_OFFSET: usize = 24 + 4 + 2; // 30
        if pdu.len() <= CPF_OFFSET {
            return;
        }
        let cpf_data = &pdu[CPF_OFFSET..];
        let items = parse_cpf_items(cpf_data);

        for item in &items {
            // F-P9-001 gate: only 0x00B2 (Unconnected Data Item) in v0.11.0.
            if item.type_id != 0x00B2 {
                continue;
            }

            let item_data = &item.data;

            // ADR-010 Decision 4, step 4: parse CIP header.
            let cip_hdr = match parse_cip_header(item_data) {
                Some(h) => h,
                None => continue,
            };

            let service_class = classify_cip_service(cip_hdr.service);

            if matches!(service_class, CipServiceClass::Response) {
                // BC-2.17.008: error accumulation for CIP responses.
                // Precondition 3: need at least 4 bytes to read general_status at byte 2.
                if item_data.len() < 4 {
                    continue;
                }
                let general_status = item_data[2];

                if general_status != 0x00 {
                    // BC-2.17.008 postcondition 4: check for window expiry BEFORE updating
                    // counters. Only applicable when an active window exists. Uses
                    // error_window_active (not error_window_start_ts == 0) so that a
                    // legitimate timestamp of 0 is not mistaken for "unseeded" (F-134-001).
                    if flow.error_window_active
                        && timestamp.wrapping_sub(flow.error_window_start_ts) > 10
                    {
                        flow.error_counts_in_window.clear();
                        flow.error_window_start_ts = timestamp;
                        flow.error_rate_emitted = false;
                    }

                    // BC-2.17.008 postcondition 2: accumulate error.
                    *flow
                        .error_counts_in_window
                        .entry(general_status)
                        .or_insert(0) += 1;
                    // BC-2.17.008 postcondition 2: seed window timestamp on first error.
                    if !flow.error_window_active {
                        flow.error_window_start_ts = timestamp;
                        flow.error_window_active = true;
                    }
                    // BC-2.17.008 postcondition 2b: aggregate lifetime counter.
                    self.error_count = self.error_count.saturating_add(1);

                    // BC-2.17.014 Pattern B: burst threshold check (strict >).
                    let total: u64 = flow.error_counts_in_window.values().sum();
                    if total > self.enip_error_burst_threshold as u64
                        && !flow.error_rate_emitted
                        && self.all_findings.len() < MAX_FINDINGS
                    {
                        self.all_findings.push(Finding {
                            category: crate::findings::ThreatCategory::Reconnaissance,
                            verdict: crate::findings::Verdict::Possible,
                            confidence: crate::findings::Confidence::Medium,
                            summary: format!(
                                "CIP error-response burst: {total} error responses in 10s window \
                                 — possible service enumeration (T0888)"
                            ),
                            evidence: vec![format!(
                                "error_counts_in_window={:?} within 10s; possible service probe",
                                flow.error_counts_in_window
                            )],
                            mitre_techniques: vec!["T0888".to_string()],
                            source_ip: Some(src_ip),
                            timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0),
                            direction: None,
                        });
                        // BC-2.17.014 Pattern B postcondition 2: one-shot guard.
                        flow.error_rate_emitted = true;
                    }
                }
            } else {
                // ADR-010 Decision 4, step 5: T0888 Pattern A — GetAttribute to Identity Object.
                // BC-2.17.014 Pattern A preconditions 1–3: GetAttribute service, request (high
                // bit clear), and Class(0x01) in path.
                if matches!(
                    service_class,
                    CipServiceClass::GetAttributesAll
                        | CipServiceClass::GetAttributeList
                        | CipServiceClass::GetAttributeSingle
                ) && cip_hdr.service & 0x80 == 0
                {
                    let path_segments = parse_cip_request_path(&cip_hdr.request_path);
                    let targets_identity = path_segments
                        .iter()
                        .any(|seg| matches!(seg, CipPathSegment::Class(0x01)));

                    if targets_identity && self.all_findings.len() < MAX_FINDINGS {
                        self.all_findings.push(Finding {
                            category: crate::findings::ThreatCategory::Reconnaissance,
                            verdict: crate::findings::Verdict::Likely,
                            confidence: crate::findings::Confidence::High,
                            summary: "CIP Identity Object attribute read: \
                                      single-device reconnaissance (T0888)"
                                .to_string(),
                            evidence: vec![format!(
                                "CIP service=0x{service:02X} ({name}) path targets Identity \
                                 Object (class 0x01) src={src_ip}",
                                service = cip_hdr.service,
                                name = service_class_name(service_class.clone()),
                            )],
                            mitre_techniques: vec!["T0888".to_string()],
                            source_ip: Some(src_ip),
                            timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0),
                            direction: None,
                        });
                    }
                }

                // T0858 detection — CIP Stop service (0x07) request.
                // BC-2.17.011 preconditions 1–5: classify_cip_service returns Stop,
                // service & 0x80 == 0 (request), type_id == 0x00B2 (F-P9-001 gate above),
                // !is_non_enip (gate at function entry), len < MAX_FINDINGS.
                // Per-occurrence, no one-shot guard (BC-2.17.011 postcondition 2).
                if matches!(service_class, CipServiceClass::Stop)
                    && cip_hdr.service & 0x80 == 0
                    && self.all_findings.len() < MAX_FINDINGS
                {
                    // BC-2.17.011 postcondition 1: emit T0858 finding per occurrence.
                    self.all_findings.push(crate::findings::Finding {
                        category: crate::findings::ThreatCategory::Execution,
                        verdict: crate::findings::Verdict::Likely,
                        confidence: crate::findings::Confidence::High,
                        summary:
                            "CIP Stop service observed: controller run\u{2192}stop transition \
                             command (T0858)"
                                .to_string(),
                        evidence: vec![format!(
                            "CIP service=0x07 (Stop) from src={src_ip} \
                             ENIP cmd={cmd:#06X} session={session}",
                            cmd = header.command,
                            session = header.session_handle,
                        )],
                        mitre_techniques: vec!["T0858".to_string()],
                        source_ip: Some(src_ip),
                        timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0),
                        direction: None,
                    });
                }

                // T0816 detection — CIP Reset service (0x05) request.
                // BC-2.17.013 preconditions 1–5: classify_cip_service returns Reset,
                // type_id == 0x00B2, !is_non_enip, len < MAX_FINDINGS.
                // Per-occurrence, no one-shot guard (BC-2.17.013 postcondition 2).
                // Uses classify_cip_service result — NOT raw `service & 0x7F == 0x05`
                // (BC-2.17.007 invariant 1, Architecture Rule 2).
                if matches!(service_class, CipServiceClass::Reset)
                    && cip_hdr.service & 0x80 == 0
                    && self.all_findings.len() < MAX_FINDINGS
                {
                    // BC-2.17.013 postcondition 1: emit T0816 finding per occurrence.
                    self.all_findings.push(crate::findings::Finding {
                        category: crate::findings::ThreatCategory::Execution,
                        verdict: crate::findings::Verdict::Likely,
                        confidence: crate::findings::Confidence::High,
                        summary: "CIP Reset service observed: adversary-triggered device restart \
                             (T0816)"
                            .to_string(),
                        evidence: vec![format!(
                            "CIP service=0x05 (Reset) from src={src_ip} \
                             ENIP cmd={cmd:#06X} session={session}",
                            cmd = header.command,
                            session = header.session_handle,
                        )],
                        mitre_techniques: vec!["T0816".to_string()],
                        source_ip: Some(src_ip),
                        timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0),
                        direction: None,
                    });
                }

                // T0836 detection — SetAttribute write-burst within 1s window.
                // BC-2.17.012 preconditions 1–5: classify_cip_service returns a write-class
                // variant (SetAttributesAll/SetAttributeList/SetAttributeSingle), request
                // (high bit clear), type_id == 0x00B2, !is_non_enip.
                // Window: write_window_start_ts (u32 seconds), write_count_in_window (u64),
                // write_burst_emitted (bool one-shot guard per window).
                // Both flow.write_count_in_window AND self.write_count incremented on every
                // qualifying write (BC-2.17.012 postconditions 1–2).
                if matches!(
                    service_class,
                    CipServiceClass::SetAttributesAll
                        | CipServiceClass::SetAttributeList
                        | CipServiceClass::SetAttributeSingle
                ) && cip_hdr.service & 0x80 == 0
                {
                    // BC-2.17.012 postcondition 4: check 1s window expiry BEFORE incrementing.
                    // Uses wrapping_sub arithmetic (same pattern as error window).
                    // Only applicable when the window is seeded (write_count_in_window > 0).
                    if flow.write_count_in_window > 0
                        && timestamp.wrapping_sub(flow.write_window_start_ts) > 1
                    {
                        // Window expired: reseed with the current write as the first of new window.
                        flow.write_count_in_window = 1;
                        flow.write_window_start_ts = timestamp;
                        flow.write_burst_emitted = false;
                    } else {
                        // BC-2.17.012 postcondition 1: increment per-flow window counter.
                        flow.write_count_in_window += 1;
                        // BC-2.17.012 postcondition 3: seed window timestamp on first write.
                        if flow.write_count_in_window == 1 {
                            flow.write_window_start_ts = timestamp;
                        }
                    }
                    // BC-2.17.012 postcondition 2: increment aggregate lifetime counter.
                    self.write_count = self.write_count.saturating_add(1);

                    // BC-2.17.012 postcondition 5: emit T0836 when count strictly exceeds
                    // threshold AND one-shot guard not set AND MAX_FINDINGS not reached.
                    if flow.write_count_in_window > self.enip_write_burst_threshold as u64
                        && !flow.write_burst_emitted
                        && self.all_findings.len() < MAX_FINDINGS
                    {
                        let svc_name = service_class_name(service_class.clone());
                        self.all_findings.push(crate::findings::Finding {
                            category: crate::findings::ThreatCategory::Execution,
                            verdict: crate::findings::Verdict::Likely,
                            confidence: crate::findings::Confidence::Medium,
                            summary: format!(
                                "CIP write-class service burst: {} SetAttribute operations in \
                                 1s window (threshold {}) \u{2014} possible parameter \
                                 modification attack (T0836)",
                                flow.write_count_in_window, self.enip_write_burst_threshold,
                            ),
                            evidence: vec![format!(
                                "CIP service=0x{svc:02X} ({svc_name}) src={src_ip} \
                                 ENIP session={session}",
                                svc = cip_hdr.service,
                                session = header.session_handle,
                            )],
                            mitre_techniques: vec!["T0836".to_string()],
                            source_ip: Some(src_ip),
                            timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0),
                            direction: None,
                        });
                        // BC-2.17.012 postcondition 5: set one-shot guard after emission.
                        flow.write_burst_emitted = true;
                    }
                }

                // STORY-136 — BC-2.17.015: CIP connection-lifecycle detection.
                // ForwardOpen (0x54), LargeForwardOpen (0x5B), ForwardClose (0x4E) requests
                // via 0x00B2 items emit Anomaly/Possible/Low findings with mitre_techniques:
                // vec![] (no ATT&CK technique for CIP connection establishment anomaly per
                // ADR-010 Decision 7). Counts increment BEFORE the MAX_FINDINGS gate so that
                // open_connection_count and close_connection_count are accurate even when the
                // findings cap is reached (EC-008 / BC-2.17.015 Architecture Rule 4).
                // Detection keys SOLELY on classify_cip_service result — NOT raw & 0x80 == 0
                // (BC-2.17.007 Invariant 1 already guarantees response bytes return Response).
                // Architecture Rule 6 / AC-136-003: do NOT hand-roll & 0x80 == 0 predicate.
                if matches!(
                    service_class,
                    CipServiceClass::ForwardOpen
                        | CipServiceClass::LargeForwardOpen
                        | CipServiceClass::ForwardClose
                ) {
                    // BC-2.17.015: increment counts BEFORE the MAX_FINDINGS gate (EC-008 /
                    // Architecture Rule 4) so session summary (STORY-138) is accurate even when
                    // the cap is reached.
                    let (summary, evidence) = if matches!(
                        service_class,
                        CipServiceClass::ForwardOpen | CipServiceClass::LargeForwardOpen
                    ) {
                        flow.open_connection_count = flow.open_connection_count.saturating_add(1);
                        let name = service_class_name(service_class);
                        let service_byte = cip_hdr.service;
                        (
                            format!(
                                "CIP ForwardOpen connection establishment observed from \
                                 src={src_ip}: connection lifecycle anomaly"
                            ),
                            vec![format!(
                                "CIP service=0x{service_byte:02X} ({name}) from src={src_ip} \
                                 session={session}. No dedicated MITRE ICS technique for CIP \
                                 connection establishment anomaly; T1692.001 applies only when \
                                 connection demonstrably carries unauthorized command \
                                 (ADR-010 Decision 7)",
                                session = header.session_handle,
                            )],
                        )
                    } else {
                        flow.close_connection_count = flow.close_connection_count.saturating_add(1);
                        (
                            format!(
                                "CIP ForwardClose connection teardown observed from \
                                 src={src_ip}: connection lifecycle closed"
                            ),
                            vec![format!(
                                "CIP service=0x4E (ForwardClose) from src={src_ip} \
                                 session={session}. Connection lifecycle closed; no dedicated \
                                 MITRE ICS technique (ADR-010 Decision 7)",
                                session = header.session_handle,
                            )],
                        )
                    };
                    if self.all_findings.len() < MAX_FINDINGS {
                        self.all_findings.push(crate::findings::Finding {
                            category: crate::findings::ThreatCategory::Anomaly,
                            verdict: crate::findings::Verdict::Possible,
                            confidence: crate::findings::Confidence::Low,
                            summary,
                            evidence,
                            mitre_techniques: vec![],
                            source_ip: Some(src_ip),
                            timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0),
                            direction: None,
                        });
                    }
                }
            }
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
// Private helper — CIP service class display name (STORY-134)
// ---------------------------------------------------------------------------

/// Map a `CipServiceClass` to a short human-readable name string for evidence fields.
///
/// Used by `process_pdu` to format T0888 Pattern A evidence.
/// Returns a static string — no allocation on the common path.
///
/// Traces: BC-2.17.014 Pattern A postcondition 1 (evidence field).
fn service_class_name(class: CipServiceClass) -> &'static str {
    match class {
        CipServiceClass::GetAttributesAll => "GetAttributesAll",
        CipServiceClass::GetAttributeList => "GetAttributeList",
        CipServiceClass::GetAttributeSingle => "GetAttributeSingle",
        CipServiceClass::SetAttributesAll => "SetAttributesAll",
        CipServiceClass::SetAttributeList => "SetAttributeList",
        CipServiceClass::SetAttributeSingle => "SetAttributeSingle",
        CipServiceClass::Reset => "Reset",
        CipServiceClass::Stop => "Stop",
        CipServiceClass::MultipleServicePacket => "MultipleServicePacket",
        CipServiceClass::GetAndClear => "GetAndClear",
        CipServiceClass::ForwardClose => "ForwardClose",
        CipServiceClass::ForwardOpen => "ForwardOpen",
        CipServiceClass::LargeForwardOpen => "LargeForwardOpen",
        CipServiceClass::Response => "Response",
        CipServiceClass::Unknown => "Unknown",
    }
}

// ---------------------------------------------------------------------------
// CPF item walk types and parser (STORY-132 — BC-2.17.005)
// ---------------------------------------------------------------------------

/// CPF item parsed from the Common Packet Format item list.
///
/// `type_id: u16` — little-endian type_id from the CPF item header (bytes 0–1 of each item).
/// `data: Vec<u8>` — item payload bytes (CPF item header bytes excluded).
///
/// **Architecture anchor (BC-2.17.005):** exactly 2 fields. The `length` field from the wire
/// is a **transient parse local** inside `parse_cpf_items` — it is NOT a struct field.
/// `data.len()` recovers the length at any point after construction.
///
/// Recognized type_ids: 0x0000 (NullAddress), 0x00A1 (ConnectedAddress), 0x00B1
/// (ConnectedData), 0x00B2 (UnconnectedData). All others are parsed into `CpfItem` with
/// their data bytes; call-site dispatch is the caller's responsibility.
#[derive(Debug, Clone, PartialEq)]
pub struct CpfItem {
    /// CPF item type identifier (little-endian wire field, 2 bytes).
    pub type_id: u16,
    /// Item payload bytes (excludes the 4-byte CPF item header).
    pub data: Vec<u8>,
}

/// Walk the CPF item list from a CPF payload byte slice.
///
/// Reads a 2-byte LE `item_count`, then iterates over each declared item. Each item
/// has a 4-byte header: `type_id` (LE u16) + `length` (LE u16, transient parse local).
/// Iteration stops early on any bounds violation — a declared `item_count` larger than
/// available bytes will not cause a panic or out-of-bounds read.
///
/// Returns `vec![]` if `cpf_data.len() < 2`.
///
/// # Fuzz Obligation (F-P9-002)
///
/// This function carries an F6 cargo-fuzz no-panic / bounds-safety obligation
/// (see VP-032 "Out-of-scope note" and ADR-010 Decision 8 DEFERRED list).
/// A fuzz harness targeting this function MUST be added in the F6 formal-hardening phase.
/// TODO: F-P9-002 — add `fuzz_target!(|data: &[u8]| { parse_cpf_items(data); })` in fuzz/.
///
/// # Panics
/// Never panics for any input (pure-core obligation, BC-2.17.005 postcondition 5).
///
/// # Traces
/// BC-2.17.005; pure-core free fn (ADR-010 Decision 2).
pub fn parse_cpf_items(cpf_data: &[u8]) -> Vec<CpfItem> {
    // BC-2.17.005 postcondition 1: < 2 bytes → cannot read item_count.
    if cpf_data.len() < 2 {
        return vec![];
    }
    // BC-2.17.005 postcondition 2: item_count is LE u16 at [0..2].
    let item_count = u16::from_le_bytes([cpf_data[0], cpf_data[1]]) as usize;
    let mut items = Vec::with_capacity(item_count.min((cpf_data.len().saturating_sub(2)) / 4));
    let mut cursor = 2usize;

    for _ in 0..item_count {
        // BC-2.17.005 postcondition 3 (first bound): need 4 bytes for item header.
        if cursor + 4 > cpf_data.len() {
            break;
        }
        // BC-2.17.005 postcondition 3: type_id and transient length are LE u16.
        let type_id = u16::from_le_bytes([cpf_data[cursor], cpf_data[cursor + 1]]);
        let length = u16::from_le_bytes([cpf_data[cursor + 2], cpf_data[cursor + 3]]) as usize;
        cursor += 4;
        // BC-2.17.005 postcondition 3 (second bound): data must fit.
        if cursor + length > cpf_data.len() {
            break;
        }
        let data = cpf_data[cursor..cursor + length].to_vec();
        cursor += length;
        items.push(CpfItem { type_id, data });
    }
    items
}

// ---------------------------------------------------------------------------
// CIP header types and parser (STORY-132 — BC-2.17.006)
// ---------------------------------------------------------------------------

/// Parsed CIP message header extracted from a `CpfItem` with `type_id == 0x00B2`.
///
/// **Architecture anchor (BC-2.17.006):** exactly 2 fields:
/// - `service: u8` — raw CIP service byte (high bit 0x80 = response; low 7 bits = service ID).
/// - `request_path: Vec<u8>` — raw path bytes (length = `request_path_size * 2`).
///
/// `request_path_size` (the wire field at `item_data[1]`) is a **transient parse local** —
/// NOT a struct field. `general_status` is also NOT a struct field: it is extracted at the
/// response call site (byte 2 of the 0x00B2 item_data, gated `len >= 4`) per BC-2.17.008.
///
/// **v0.11.0 caller contract (F-P9-001):** this function MUST be called ONLY for items with
/// `type_id == 0x00B2`. Passing a 0x00B1 Connected Data Item (which has a 2-byte
/// CIP sequence-count prefix) will misparse the sequence count as the service byte.
/// The call-site gate (`item.type_id == 0x00B2`) lives in `EnipAnalyzer::process_pdu`.
#[derive(Debug, Clone, PartialEq)]
pub struct CipHeader {
    /// Raw CIP service byte: bit 7 set = response; bits 0–6 = service code.
    pub service: u8,
    /// Raw request path bytes (length = `request_path_size * 2` words).
    pub request_path: Vec<u8>,
}

/// Parse a CIP header from the data bytes of an Unconnected Data Item (0x00B2).
///
/// Returns `Some(CipHeader)` when `item_data.len() >= 2` and the path bytes fit within
/// the slice. Returns `None` if the data is too short for the declared path.
///
/// **Call-site contract (F-P9-001):** call ONLY for `type_id == 0x00B2` items in v0.11.0.
///
/// # Fuzz Obligation (F-P9-002)
///
/// This function carries an F6 cargo-fuzz no-panic / bounds-safety obligation
/// (see VP-032 "Out-of-scope note" and ADR-010 Decision 8 DEFERRED list).
/// A fuzz harness targeting this function MUST be added in the F6 formal-hardening phase.
/// TODO: F-P9-002 — add `fuzz_target!(|data: &[u8]| { parse_cip_header(data); })` in fuzz/.
///
/// # Panics
/// Never panics for any input (pure-core obligation, BC-2.17.006 postcondition 8).
///
/// # Traces
/// BC-2.17.006; pure-core free fn (ADR-010 Decision 2); F-P9-001 call-site gate.
pub fn parse_cip_header(item_data: &[u8]) -> Option<CipHeader> {
    // BC-2.17.006 postcondition 1: < 2 bytes → None.
    if item_data.len() < 2 {
        return None;
    }
    // BC-2.17.006 postconditions 2–3: service byte and transient path_size.
    let service = item_data[0];
    let request_path_size = item_data[1] as usize;
    let path_byte_count = request_path_size * 2;
    // BC-2.17.006 postcondition 5: truncated path → None.
    if item_data.len() < 2 + path_byte_count {
        return None;
    }
    // BC-2.17.006 postcondition 6: extract path bytes.
    let request_path = item_data[2..2 + path_byte_count].to_vec();
    Some(CipHeader {
        service,
        request_path,
    })
}

// ---------------------------------------------------------------------------
// CIP service classification (STORY-132 — BC-2.17.007; VP-032 Sub-D Kani target)
// ---------------------------------------------------------------------------

/// CIP service classification over all 256 possible `u8` service byte values.
///
/// Exactly 15 variants: 13 named request services + `Response` + `Unknown`.
/// The `Response` variant covers all 128 values in range 0x80–0xFF (high bit set).
/// The `Unknown` variant covers request-range values (0x00–0x7F) not in the named set.
///
/// VP-032 Sub-D Kani target: both response-bit totality and request-range partition are
/// formally verified by `vp032_cip_service_classification_totality` and
/// `vp032_cip_service_request_partition` in `#[cfg(kani)] mod kani_proofs`.
///
/// # Traces
/// BC-2.17.007; VP-032 Sub-D.
#[derive(Debug, Clone, PartialEq)]
pub enum CipServiceClass {
    /// 0x01 — Get all attributes of a CIP object instance.
    GetAttributesAll,
    /// 0x02 — Set all attributes of a CIP object instance (T0836 write trigger).
    SetAttributesAll,
    /// 0x03 — Get a list of attribute values by attribute ID list.
    GetAttributeList,
    /// 0x04 — Set a list of attribute values by attribute ID list (T0836 write trigger).
    SetAttributeList,
    /// 0x05 — Reset a CIP object (T0816 detection trigger).
    Reset,
    /// 0x07 — Change Operating Mode / Stop (T0858 detection trigger).
    Stop,
    /// 0x0A — Send multiple CIP services in one request (per ODVA CIP Vol 1 §3-5.5).
    MultipleServicePacket,
    /// 0x0E — Get a single attribute value by attribute ID (T0888 identity-read trigger).
    GetAttributeSingle,
    /// 0x10 — Set a single attribute value by attribute ID (T0836 write trigger).
    SetAttributeSingle,
    /// 0x4B — Get-and-clear (wirerust convention for staged T1693.001 firmware marker;
    /// not emitted in v0.11.0 per ADR-010 Decision 8 deferred list).
    GetAndClear,
    /// 0x4E — Forward Close (connection lifecycle, BC-2.17.015).
    ForwardClose,
    /// 0x54 — Forward Open (connection lifecycle, BC-2.17.015).
    ForwardOpen,
    /// 0x5B — Large Forward Open (connection lifecycle, BC-2.17.015).
    LargeForwardOpen,
    /// Any service byte with high bit set (0x80–0xFF): CIP response message.
    /// The response-bit invariant (BC-2.17.007 postcondition 2) is checked first.
    Response,
    /// Any request-range service byte (0x00–0x7F) not in the 13-variant named set.
    Unknown,
}

/// Classify a CIP service byte into a `CipServiceClass` variant.
///
/// This function is **total** — every possible `u8` input maps to exactly one variant
/// without panicking (BC-2.17.007 postcondition 1; VP-032 Sub-D).
///
/// **Response-bit invariant (BC-2.17.007 invariant 1):** `service & 0x80 != 0` is checked
/// FIRST; matching values return `CipServiceClass::Response` regardless of the lower 7 bits.
///
/// For request-range values (high bit clear), 13 named service codes map to named variants;
/// all other values map to `CipServiceClass::Unknown`.
///
/// # Panics
/// Never panics for any input (pure-core obligation; VP-032 Sub-D safety contract).
///
/// # Traces
/// BC-2.17.007; VP-032 Sub-D primary + partition Kani targets.
pub fn classify_cip_service(service: u8) -> CipServiceClass {
    // BC-2.17.007 invariant 1: response-bit check FIRST (applies to 0x80–0xFF range).
    if service & 0x80 != 0 {
        return CipServiceClass::Response;
    }
    // BC-2.17.007 postcondition 3: 13 named request service codes.
    match service {
        0x01 => CipServiceClass::GetAttributesAll,
        0x02 => CipServiceClass::SetAttributesAll,
        0x03 => CipServiceClass::GetAttributeList,
        0x04 => CipServiceClass::SetAttributeList,
        0x05 => CipServiceClass::Reset,
        0x07 => CipServiceClass::Stop,
        0x0A => CipServiceClass::MultipleServicePacket,
        0x0E => CipServiceClass::GetAttributeSingle,
        0x10 => CipServiceClass::SetAttributeSingle,
        0x4B => CipServiceClass::GetAndClear,
        0x4E => CipServiceClass::ForwardClose,
        0x54 => CipServiceClass::ForwardOpen,
        0x5B => CipServiceClass::LargeForwardOpen,
        // BC-2.17.007 postcondition 4: all other request-range values → Unknown.
        _ => CipServiceClass::Unknown,
    }
}

// ---------------------------------------------------------------------------
// CIP request path types and parser (STORY-132 — BC-2.17.009)
// ---------------------------------------------------------------------------

/// A single CIP logical path segment (8-bit format, v0.11.0 scope).
///
/// Three variants in scope for v0.11.0 per ADR-010 Decision 8:
/// - `Class(u8)` — segment type 0x20; value = CIP class ID.
/// - `Instance(u8)` — segment type 0x24; value = instance number.
/// - `Attribute(u8)` — segment type 0x30; value = attribute ID.
///
/// 16-bit extended variants (0x21, 0x25, 0x31) and Electronic Key segments are deferred
/// to v0.12.0. Unrecognized segment types are silently skipped in `parse_cip_request_path`.
///
/// # Traces
/// BC-2.17.009; ADR-010 Decision 8 (8-bit logical segments only, v0.11.0).
#[derive(Debug, Clone, PartialEq)]
pub enum CipPathSegment {
    /// CIP Class segment (type byte 0x20): identifies the target CIP object class.
    /// `Class(0x01)` = Identity Object (T0888 recon trigger, BC-2.17.014).
    Class(u8),
    /// CIP Instance segment (type byte 0x24): identifies the target object instance.
    Instance(u8),
    /// CIP Attribute segment (type byte 0x30): identifies the target attribute.
    Attribute(u8),
}

/// Extract Class, Instance, and Attribute segments from a CIP request path byte slice.
///
/// Walks the path 2 bytes at a time. For each pair: exact-match on segment type byte
/// (0x20 = Class, 0x24 = Instance, 0x30 = Attribute). Unrecognized segment types advance
/// the cursor by 2 and are silently skipped. Stops at any bounds violation.
///
/// Returns `vec![]` for an empty or 1-byte path.
///
/// **Architecture constraint (ADR-010 Decision 8 / Architecture Rule 2):** use exact-match
/// (== 0x20 / == 0x24 / == 0x30) — do NOT use `& 0xE0` mask (would misclassify 0x24 as
/// Class). 16-bit extended segments are deferred to v0.12.0.
///
/// # Fuzz Obligation (F-P9-002)
///
/// TODO: F-P9-002 — add `fuzz_target!(|data: &[u8]| { parse_cip_request_path(data); })`
/// in fuzz/ during F6 formal-hardening phase.
///
/// # Panics
/// Never panics for any input (pure-core obligation, BC-2.17.009 postcondition 4).
///
/// # Traces
/// BC-2.17.009; ADR-010 Decision 8; pure-core free fn.
pub fn parse_cip_request_path(path: &[u8]) -> Vec<CipPathSegment> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    // BC-2.17.009 postcondition 2: walk 2 bytes at a time; break on bounds violation.
    while cursor + 2 <= path.len() {
        let segment_type = path[cursor];
        let value = path[cursor + 1];
        // BC-2.17.009 postcondition 2: exact-match only (Architecture Rule 2 — no &0xE0 mask).
        match segment_type {
            0x20 => segments.push(CipPathSegment::Class(value)),
            0x24 => segments.push(CipPathSegment::Instance(value)),
            0x30 => segments.push(CipPathSegment::Attribute(value)),
            // Other segment types: skip; advance by 2.
            _ => {}
        }
        cursor += 2;
    }
    segments
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

    // -----------------------------------------------------------------------
    // VP-032 Sub-D harnesses (STORY-132 — BC-2.17.007)
    // Appended to the existing kani_proofs block opened by STORY-130.
    // DF-KANI-NONVACUITY-001: both harnesses call the production
    // classify_cip_service by name — no re-implementation.
    // -----------------------------------------------------------------------

    /// VP-032 Sub-D (primary): classify_cip_service is total over all 256 u8 values;
    /// `service & 0x80 != 0` iff result is `CipServiceClass::Response`.
    ///
    /// Biconditional simultaneously proves response-arm totality, Response-variant
    /// reachability, and non-Response reachability (DF-KANI-NONVACUITY-001).
    /// No kani::assume — both arms are reachable in the full symbolic u8 domain.
    ///
    /// BOUND/SOUNDNESS: symbolic u8 covers all 256 possible CIP service byte values.
    /// No loops → no unwind annotation needed.
    ///
    /// Traces: BC-2.17.007 postconditions 1–2, invariant 1; VP-032 Sub-D primary.
    #[kani::proof]
    fn vp032_cip_service_classification_totality() {
        let service: u8 = kani::any();
        let class = classify_cip_service(service);
        // Response-bit biconditional: Response iff high bit set.
        // Proven for all 256 symbolic u8 values.
        let is_response = matches!(class, CipServiceClass::Response);
        assert_eq!(is_response, service & 0x80 != 0);
    }

    /// VP-032 Sub-D (partition): over the request range 0x00..=0x7F, every service byte
    /// maps to either a named CIP service variant or `Unknown` — the partition is exhaustive.
    /// Proves `Unknown` is reachable (non-vacuous) and that no named variant leaks into
    /// the Unknown arm (DF-KANI-NONVACUITY-001).
    ///
    /// BOUND/SOUNDNESS: constrained to 0x00..=0x7F (request range only; response arm
    /// covered by the primary harness above). No loops → no unwind annotation needed.
    ///
    /// Traces: BC-2.17.007 postconditions 3–5, invariant 2; VP-032 Sub-D partition.
    #[kani::proof]
    fn vp032_cip_service_request_partition() {
        const NAMED_SERVICES: &[u8] = &[
            0x01, // GetAttributesAll
            0x02, // SetAttributesAll
            0x03, // GetAttributeList
            0x04, // SetAttributeList
            0x05, // Reset
            0x07, // Stop
            0x0A, // MultipleServicePacket
            0x0E, // GetAttributeSingle
            0x10, // SetAttributeSingle
            0x4B, // GetAndClear
            0x4E, // ForwardClose
            0x54, // ForwardOpen
            0x5B, // LargeForwardOpen
        ];
        let service: u8 = kani::any();
        // Restrict to request range (high bit clear).
        kani::assume(service & 0x80 == 0);
        let class = classify_cip_service(service);
        // Must NOT be Response (high bit clear means request; BC-2.17.007 invariant 1).
        assert!(!matches!(class, CipServiceClass::Response));
        // Exhaustive named-vs-Unknown partition: named iff not Unknown.
        let is_named = NAMED_SERVICES.contains(&service);
        let is_unknown = matches!(class, CipServiceClass::Unknown);
        assert_eq!(is_named, !is_unknown);
    }
}
