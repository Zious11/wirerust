---
document_type: architecture-delta
feature_id: issue-008-dnp3-analyzer
github_issue: 8
title: "F2 Architecture Delta — DNP3 TCP Analyzer (SS-15)"
status: draft
version: "1.1"
producer: architect
created: 2026-06-10
modified:
  - "v1.1 (2026-06-10, Pass-2 remediation): Added 6 correlation-state fields to Dnp3FlowState (HIGH-1/HIGH-2): restart_event_count, block_event_count, pending_requests, block_finding_emitted_this_window, loss_of_control_emitted, correlation_window_start_ts. Added MAX_PENDING_REQUESTS bound constant. Added correlation-window reset note. Fields required by BC-2.15.011/014/015 for T1691.001 and T0827 detection."
base_commit: fb2c875
branch: develop
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/phase-f1-delta-analysis/dnp3-delta-analysis.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
---

# F2 Architecture Delta — Issue #8: DNP3 TCP Protocol Analyzer

## 1. Overview

This document is the complete architecture delta for Feature #8 (DNP3 analyzer, Issue #8).
It defines all new types, state structures, integration contracts, and spec-update obligations
needed before F3 story decomposition begins. It does NOT author behavioral contracts
(BC-2.15.NNN) — those are the product-owner's responsibility. It defines the structures and
anchors those contracts depend on.

**Approved F1 scope (locked — do not re-litigate):**
- TCP-only, port 20000, `StreamHandler` + `StreamDispatcher` integration (DispatchTarget::Dnp3,
  port-20000 Rule 6)
- CRC-16/DNP: structural skip only (strip per-block, no CRC validation in v1)
- Application layer: parse FC from FIR=1 first fragments only
- CLI: `--dnp3-direct-operate-threshold` (mirrors `--modbus-write-burst-threshold`)
- MITRE technique set (v19.1-corrected): T1692.001, T1691.001 (NEW), T0827 (NEW), T0814, T0836

**ADR produced:** ADR-007
(`.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md`)

---

## 2. SS-15 "DNP3/ICS Analysis" Subsystem Shard

### 2.1 Component Definition: C-23 Dnp3Analyzer

| Field | Value |
|-------|-------|
| Component ID | C-23 |
| File | `src/analyzer/dnp3.rs` |
| Subsystem | SS-15 |
| Traits | `StreamHandler` + `StreamAnalyzer` (from `src/reassembly/handler.rs`) |
| Purity | Split: pure-core parse functions (VP-023 target) + effectful shell (`on_data`, flow state HashMap) |

`Dnp3Analyzer` follows the same internal pattern established by `ModbusAnalyzer` (C-22)
per ADR-0002 §Internal Structure Pattern.

### 2.2 Top-Level Struct Layout

```rust
pub struct Dnp3Analyzer {
    /// Per-flow state, keyed by FlowKey.
    flows: HashMap<FlowKey, Dnp3FlowState>,

    /// Threshold for DIRECT_OPERATE (0x05) + DIRECT_OPERATE_NR (0x06) detection window.
    /// Fires T1692.001 (unauthorized control) when exceeded.
    /// CLI: --dnp3-direct-operate-threshold. Default: TBD in F3.
    direct_operate_threshold: u32,

    /// Aggregate function-code distribution across all flows: FC byte → count.
    fn_code_counts: HashMap<u8, u64>,
}
```

### 2.3 Per-Flow State Layout

```rust
pub struct Dnp3FlowState {
    /// Partial frame carry buffer. Max 292 bytes (max DNP3 link frame size).
    carry: Vec<u8>,

    /// Desync guard: set true when the flow passes port-20000 but no valid
    /// DNP3 sync word is observed in the first 16 bytes. All subsequent
    /// on_data calls for this flow are no-ops when is_non_dnp3 = true.
    is_non_dnp3: bool,

    /// Per-flow application FC distribution: FC byte → count.
    fc_counts: HashMap<u8, u64>,

    /// Direct-operate count within the current detection window.
    direct_operate_count: u32,

    /// Timestamp (secs) of the first direct-operate in the current window.
    window_start_ts: u32,

    /// Guard: T1692.001 already emitted in this window (one-shot per window).
    direct_operate_emitted: bool,

    /// Source link addresses observed with DIR=1 (master-direction bit).
    /// Bounded to MAX_MASTER_ADDRS to prevent unbounded growth on adversarial
    /// traffic spoofing many source addresses.
    master_addrs_seen: Vec<u16>,

    /// Parse errors (invalid frames, sync failures, CRC-block boundary overruns).
    parse_errors: u64,

    /// Total frames processed.
    frame_count: u64,

    // ---- Correlation-window state (BC-2.15.011 / BC-2.15.014 / BC-2.15.015) ----
    // All six fields below reset together at correlation-window expiry (single window,
    // default 300s [F2-GATE]).  They are distinct from the direct-operate burst
    // detector window above (60s, controlled by window_start_ts / direct_operate_count).

    /// COLD_RESTART (0x0D) + WARM_RESTART (0x0E) event accumulator within the
    /// correlation window.  Contributes to the T0827 (Loss of Control) derived
    /// finding threshold (BC-2.15.011).  Feeds T0827; does NOT feed T1691.001.
    restart_event_count: u64,

    /// Block-timeout accumulator: control requests without a corresponding
    /// outstation RESPONSE within the T1691.001 inference timeout.  Feeds both
    /// the T1691.001 per-window threshold guard AND the T0827 derivation
    /// threshold (BC-2.15.014).
    block_event_count: u64,

    /// Outstanding control requests awaiting an outstation RESPONSE, keyed by
    /// (destination_addr: u16, app_seq: u8) → observation_timestamp_secs: u32.
    /// Used for passive T1691.001 request/response correlation and timeout
    /// inference (BC-2.15.014).
    ///
    /// BOUNDED: entries are evicted on matching RESPONSE or on correlation-window
    /// expiry.  At-insert, if the table would exceed MAX_PENDING_REQUESTS the
    /// oldest entry is dropped (mirrors the Modbus pending-table DoS bound
    /// established in architecture-delta.md §2 / ADR-005 Decision 2).
    pending_requests: HashMap<(u16, u8), u32>,

    /// One-shot guard: T1691.001 finding already emitted this correlation window
    /// (BC-2.15.014).  Prevents duplicate T1691.001 emissions within a single
    /// window.  Reset at correlation-window expiry.
    block_finding_emitted_this_window: bool,

    /// One-shot guard: T0827 "Loss of Control" finding already emitted this
    /// correlation window (BC-2.15.015).  Prevents duplicate T0827 emissions
    /// within a single window.  Reset at correlation-window expiry.
    loss_of_control_emitted: bool,

    /// Timestamp (seconds) of the start of the current correlation window.
    /// Window default: 300 seconds [F2-GATE]; exact threshold pinned in F3 BCs.
    /// When `now - correlation_window_start_ts >= window_duration`, all six
    /// correlation-state fields reset and this field is set to `now`.
    correlation_window_start_ts: u32,
}
```

**Constants:**
```rust
/// Maximum DNP3 link frame size in bytes (IEEE 1815: LENGTH=255 → 292).
const MAX_DNP3_FRAME_LEN: usize = 292;

/// Maximum master-source addresses tracked per flow.
const MAX_MASTER_ADDRS: usize = 64;

/// Maximum outstanding pending control requests tracked per flow for
/// T1691.001 request/response correlation.  Entries beyond this limit
/// cause the oldest entry to be evicted (DoS-safe bounded table,
/// mirrors Modbus pending-table bound).
const MAX_PENDING_REQUESTS: usize = 256;
```

---

## 3. Pure-Core Function Signatures (VP-023 Targets)

These four functions form the pure core. They take data by value/reference, perform no
I/O, and construct only stack structures. They are the Kani verification targets for VP-023.

```rust
/// Data-link header struct (parsed from first 10 bytes of a link frame).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dnp3DlHeader {
    pub start1:      u8,   // byte 0 — must be 0x05
    pub start2:      u8,   // byte 1 — must be 0x64
    pub length:      u8,   // byte 2 — user data count (CONTROL+DEST+SOURCE+payload), min 5
    pub control:     u8,   // byte 3 — DIR/PRM/FCB/FCV/link-FC bitfield
    pub destination: u16,  // bytes 4-5, little-endian
    pub source:      u16,  // bytes 6-7, little-endian
    // bytes 8-9: header CRC (not a struct field; skipped structurally)
}

/// Application-layer function-code classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dnp3FcClass {
    Read,        // 0x01 READ
    Write,       // 0x02 WRITE → T0836
    Control,     // 0x03 SELECT, 0x04 OPERATE, 0x05 DIRECT_OPERATE, 0x06 DIRECT_OPERATE_NR → T1692.001
    Restart,     // 0x0D COLD_RESTART, 0x0E WARM_RESTART → T0814
    Management,  // all other management FCs (freeze, config, time, file, auth)
    Response,    // 0x81 RESPONSE, 0x82 UNSOLICITED_RESPONSE, 0x83 AUTHENTICATE_RESP
    Unknown,     // everything else (wildcard arm)
}

/// Parse the 10-byte DNP3 data-link header (8 header bytes + 2 header CRC).
/// Returns None if data.len() < 10 (truncated frame).
/// Never panics for any input. (VP-023 Sub-property A)
pub fn parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>;

/// Three-point validity gate: true iff start1==0x05 && start2==0x64 && length>=5.
/// Never panics; reads only struct fields. (VP-023 Sub-property C)
pub fn is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool;

/// Classify an application function code byte into a Dnp3FcClass.
/// Total over all 256 u8 values (VP-023 Sub-property B).
pub fn classify_dnp3_fc(fc: u8) -> Dnp3FcClass;

/// Compute the total on-wire frame length for a given LENGTH field value.
/// Returns None if length < 5 (invalid). For length in 5..=255, returns
/// frame_len = 5 + length + 2 * ceil((length - 5) / 16).
/// Result is always in [10, 292]. Never panics. (VP-023 Sub-property D)
pub fn compute_dnp3_frame_len(length: u8) -> Option<usize>;
```

---

## 4. StreamDispatcher Integration Contracts

### 4.1 DispatchTarget Extension

```rust
// BEFORE (4 variants):
pub enum DispatchTarget {
    Http, Tls, Modbus, None,
}

// AFTER (5 variants):
pub enum DispatchTarget {
    Http, Tls, Modbus, Dnp3, None,   // Dnp3 added (Rule 6, port 20000)
}
```

### 4.2 Rule-6 classify() Addition

Position: after port-502/Modbus Rule 5, before the DispatchTarget::None fallback.
The fallback becomes Rule 7 after this change (ADR-007 Decision 1 Rule Table).

```rust
// Rule 6: DNP3 port (20000 — IANA-registered, ADR-007). Fires AFTER all
// content rules and all earlier port-fallback rules. TLS/HTTP on port 20000
// will have matched Rules 1 or 2 above (INV-2 / VP-004 precedence).
if ports.contains(&20000) {
    return DispatchTarget::Dnp3;
}
// Rule 7: no match (was Rule 6 before this ADR).
DispatchTarget::None
```

### 4.3 StreamDispatcher Struct Field

```rust
pub struct StreamDispatcher {
    // existing fields ...
    dnp3: Option<Dnp3Analyzer>,   // NEW
}
```

### 4.4 Early-Exit Guard Extension

```rust
// BEFORE:
if self.http.is_none() && self.tls.is_none() && self.modbus.is_none() {
    return;
}
// AFTER:
if self.http.is_none() && self.tls.is_none() && self.modbus.is_none() && self.dnp3.is_none() {
    return;
}
```

### 4.5 take_dnp3_analyzer() Accessor

```rust
/// Moves the DNP3 analyzer out of the dispatcher, consuming the slot.
/// Mirror of take_modbus_analyzer(). Call ONCE, post-reassembler.finalize().
pub fn take_dnp3_analyzer(&mut self) -> Option<Dnp3Analyzer> {
    self.dnp3.take()
}
```

### 4.6 VP-004 classify_oracle Obligation

The `classify_oracle` function in `dispatcher.rs`'s `#[cfg(kani)] mod kani_proofs` must
gain the port-20000 → Dnp3 arm immediately after the port-502 → Modbus arm:

```rust
// In kani_proofs::classify_oracle (after Modbus arm):
if lower_port == 20000 || upper_port == 20000 {
    return DispatchTarget::Dnp3;
}
```

This is the CRITICAL obligation from the F1 delta (dnp3-delta-analysis.md §5.3 "Highest-Risk
Shared Touchpoints"). The `verify_content_first_precedence_exhaustive` Kani proof asserts
`got == want` across all 65 536^2 port combinations; oracle/production divergence causes
proof failure at F6.

---

## 5. CONTROL Octet Helpers (Link-Layer Direction)

The CONTROL octet (data-link header byte 3) carries the DIR flag (bit 7). The analyzer
uses this to attribute findings directionally:

```rust
/// Returns true if the frame's CONTROL byte has DIR=1 (master direction).
/// DIR=1 means "sent by the master station."
#[inline]
fn is_master_frame(control: u8) -> bool {
    control & 0x80 != 0
}

/// Returns true if the frame carries user data (link FC 0x03 or 0x04).
/// Only CONFIRMED_USER_DATA (0x03) and UNCONFIRMED_USER_DATA (0x04) carry
/// transport+application layers; all other link FCs are link-control only.
#[inline]
fn has_user_data(control: u8) -> bool {
    let link_fc = control & 0x0F;
    link_fc == 0x03 || link_fc == 0x04
}
```

---

## 6. Transport Octet Helpers

After the 10-byte header, the first user octet in the payload buffer is the transport
octet. The analyzer checks FIR to gate application-layer parsing:

```rust
/// Returns true if this transport segment is the FIRST segment of an application fragment.
/// FIR=1 means the following bytes are App-Control + App-FC (safe to parse as a new request).
#[inline]
fn transport_is_fir(transport_octet: u8) -> bool {
    transport_octet & 0x40 != 0
}
```

---

## 7. Addressing Helpers (Broadcast Detection)

```rust
/// Broadcast destination addresses: 0xFFFD, 0xFFFE, 0xFFFF.
/// Treating any destination in 0xFFFD..=0xFFFF as broadcast is safe and
/// spec-supported regardless of exact per-address confirm-semantics (which are
/// [UNVERIFIED] — see §9 open questions). [SPEC: three broadcast addrs confirmed]
#[inline]
fn is_broadcast_destination(dest: u16) -> bool {
    dest >= 0xFFFD
}
```

---

## 8. Detection Logic Summary (Pure Detection Spec — no code here)

| Detection | FC/condition | MITRE technique | Notes |
|-----------|-------------|-----------------|-------|
| Unauthorized control | SELECT/OPERATE/DIRECT_OPERATE/DIRECT_OPERATE_NR (0x03–0x06) from any source when threshold exceeded OR from non-allowlisted source (master_addrs heuristic) | T1692.001 | One finding per window-overflow (direct_operate_emitted guard) |
| Modify parameter | WRITE (0x02) | T0836 | Per-frame detection; always emitted when WRITE observed |
| DoS via restart | COLD_RESTART (0x0D) / WARM_RESTART (0x0E) | T0814 | Per-occurrence; threshold for T0827 derivation |
| Block command (inferred) | Control FC observed, no corresponding outstation RESPONSE (0x81) within timeout | T1691.001 | Passive inference only; requires request/response correlation |
| Loss of control (derived) | Sustained T0814 or T1691.001 events exceeding correlated-impact threshold | T0827 | Multi-event correlated finding; NOT per-packet |
| Broadcast control | Control FC to destination 0xFFFD–0xFFFF | T1692.001 co-emitted | Anomaly note added to finding |

**T0827 emission guard (F3 decision):** The threshold for deriving the T0827 Impact
finding from accumulated T0814 / T1691.001 events is an F3 story decision. F2 specifies
that T0827 MUST NOT be emitted from a single packet and MUST require a multi-event
correlation window.

---

## 9. MITRE Catalog Obligations (VP-007 Atomic Update)

This is the most mechanically-enforced change in this delta. The full 5-part obligation
is specified in ADR-007 Decision 5. Summary of what changes in `src/mitre.rs`:

### 9.1 New technique_info Arms

```rust
// In technique_info match:
"T1691.001" => (
    "Block Operational Technology Message: Command Message",
    MitreTactic::IcsInhibitResponseFunction,
),
"T0827" => (
    "Loss of Control",
    MitreTactic::IcsImpact,
),
```

### 9.2 New MitreTactic Variant

```rust
// In MitreTactic enum (after IcsImpairProcessControl):
IcsImpact,   // ICS Impact tactic (TA0105) — T0827 Loss of Control
```

With matching Display arm:
```rust
MitreTactic::IcsImpact => "Impact",
```

And appended to all_tactics_in_report_order():
```rust
MitreTactic::IcsImpact,
```

### 9.3 SEEDED_TECHNIQUE_IDS and Count

Add to the ICS section of `SEEDED_TECHNIQUE_IDS`:
```rust
"T1691.001",    // ICS new F3 (issue #8) — Block OT Message: Command Message
"T0827",        // ICS new F3 (issue #8) — Loss of Control (Impact)
```

Bump: `SEEDED_TECHNIQUE_ID_COUNT: usize = 21` → `23`.

### 9.4 EMITTED_IDS Extension

Add to `kani_proofs::EMITTED_IDS`:
```rust
"T1691.001",    // DNP3: request-without-response inference
"T0827",        // DNP3: derived loss-of-control impact finding
```

### 9.5 Count Verification

After the update:
- `SEEDED_TECHNIQUE_IDS.len()` = 23
- `SEEDED_TECHNIQUE_ID_COUNT` = 23
- `EMITTED_IDS.len()` = 15 (13 prior + T1691.001 + T0827)
- `cargo test mitre` must pass before PR merge

**Note on T0803 (revoked):** Do NOT add T0803. The F1 delta erroneously listed T0803 as
the new catalog entry. T0803 is REVOKED in ics-attack-19.1; T1691.001 is the replacement.
See dnp3-research.md §6 and ADR-007 Decision 5.

---

## 10. CLI Delta

```rust
// In Commands::Analyze struct (cli.rs):
/// Threshold for DIRECT_OPERATE/DIRECT_OPERATE_NR FC burst detection.
/// Fires T1692.001 when count in the detection window exceeds this value.
#[arg(long, default_value_t = DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT)]
dnp3_direct_operate_threshold: u32,
```

The default value (`DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`) is an F3 decision.
It must be added to `main.rs`'s `Dnp3Analyzer::new()` constructor call.

---

## 11. Pure-Core vs Effectful-Shell Boundary (VP-023 Purity Map)

| Function | Layer | Pure/Effectful | VP-023 target? |
|----------|-------|----------------|----------------|
| `parse_dnp3_dl_header` | Pure core | Pure | YES (Sub-A) |
| `is_valid_dnp3_frame_header` | Pure core | Pure | YES (Sub-C) |
| `classify_dnp3_fc` | Pure core | Pure | YES (Sub-B) |
| `compute_dnp3_frame_len` | Pure core | Pure | YES (Sub-D) |
| `is_master_frame` | Pure core | Pure | No (trivial 1-liner) |
| `transport_is_fir` | Pure core | Pure | No (trivial 1-liner) |
| `is_broadcast_destination` | Pure core | Pure | No (trivial 1-liner) |
| `Dnp3Analyzer::on_data` | Effectful shell | Effectful | No (HashMap, flow state) |
| `Dnp3Analyzer::on_flow_close` | Effectful shell | Effectful | No |
| `Dnp3Analyzer::finalize` | Effectful shell | Effectful | No |

The pure-core functions are free `fn`s (not `impl Dnp3Analyzer` methods), following the
VP-022 Modbus precedent that proved essential for Kani harness targeting.

---

## 12. Regression Risk Reference

The four highest-risk shared touchpoints from dnp3-delta-analysis.md §9:

| Touchpoint | Risk | Delta obligation |
|-----------|------|-----------------|
| `dispatcher.rs` VP-004 classify_oracle | CRITICAL | Gain port-20000 → Dnp3 arm (§4.6 above) |
| `mitre.rs` VP-007 drift guard | CRITICAL | 5-part atomic update (§9 above) |
| `dispatcher.rs` early-exit guard | HIGH | Extend to `&& self.dnp3.is_none()` (§4.4 above) |
| `main.rs` needs_reassembly predicate | HIGH | Add `enable_dnp3` to the predicate |

---

## 13. Architecture Spec-Update Obligations

Files updated in this F2 burst (all already updated by architect):

| Artifact | Change |
|---------|--------|
| `.factory/specs/architecture/ARCH-INDEX.md` | SS-15 added to Subsystem Registry; ADR-007 added to ADR table; SS-15 bounded-resource note |
| `.factory/specs/verification-properties/VP-INDEX.md` | VP-023 added; counts bumped (total 22→23, P1 8→9, Kani 9→10, draft 0→1) |
| `.factory/specs/architecture/verification-architecture.md` | VP-023 added to Should Prove table; P1 list updated |
| `.factory/specs/architecture/verification-coverage-matrix.md` | VP-023 row added; analyzer/dnp3.rs row added; Totals updated (Kani 9→10, total 22→23) |

Files to be updated in F3/F4 stories:

| Artifact | Change |
|---------|--------|
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | Add SS-15 section (BC-2.15.001..NNN rows) |
| `module-criticality.md` | Add `Dnp3Analyzer` with criticality classification |
