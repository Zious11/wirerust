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
    /// Value 0 indicates no error has been seen in the current window yet.
    /// Field name is normative per BC-2.17.008 postcondition 2.
    pub error_window_start_ts: u32,

    /// One-shot guard for T0888 Pattern B (error-burst finding per window).
    ///
    /// Set to `true` when the Pattern B finding is emitted for the current window;
    /// reset to `false` on window expiry (BC-2.17.008 postcondition 4).
    /// Field name is normative per BC-2.17.008 invariant.
    pub error_rate_emitted: bool,

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
            error_rate_emitted: false,
            list_identity_emitted: false,
            is_non_enip: false,
            command_counts: HashMap::new(),
            pdu_count: 0,
            parse_errors: 0,
            malformed_in_window: 0,
            carry: Vec::new(),
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
        }
    }

    /// Receive reassembled TCP bytes for a port-44818 flow.
    ///
    /// Increments `bytes_received` by `data.len()` (saturating add) to evidence
    /// BC-2.17.019 PC-2 routing correctness. Full CIP frame-walk and detection
    /// logic are added by STORY-132+; this stub satisfies the dispatcher wiring
    /// contract for STORY-131.
    ///
    /// `flow_key` and `timestamp` are accepted to match the DNP3 `on_data`
    /// signature (ADR-010 Decision 9; STORY-131 boundary decision), but are
    /// not consumed until STORY-132 adds the frame-walk loop.
    ///
    /// WIRING-EXEMPT: single saturating_add with no I/O; ≤ 3 lines.
    ///
    /// # Traces
    /// BC-2.17.019 §P2 (routing confirmation); AC-131-001 observable.
    pub fn on_data(
        &mut self,
        _flow_key: crate::reassembly::flow::FlowKey,
        data: &[u8],
        _timestamp: u32,
    ) {
        self.bytes_received = self.bytes_received.saturating_add(data.len() as u64);
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
    /// BC-2.17.008, BC-2.17.010, BC-2.17.014; ADR-010 Decision 6 (detection order).
    pub fn process_pdu(
        &mut self,
        _flow: &mut EnipFlowState,
        _pdu: &[u8],
        _timestamp: u32,
        _src_ip: IpAddr,
    ) {
        todo!(
            "STORY-134: implement T0846/T0888 detection dispatch \
             (BC-2.17.010 ListIdentity, BC-2.17.008 error accumulation, \
             BC-2.17.014 Pattern A Identity read + Pattern B error burst)"
        )
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
