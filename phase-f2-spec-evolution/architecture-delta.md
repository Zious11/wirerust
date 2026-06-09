---
document_type: architecture-delta
feature_id: issue-007-modbus-analyzer
github_issue: 7
title: "F2 Architecture Delta — Modbus TCP Analyzer (SS-14)"
status: draft
producer: architect
created: 2026-06-09
base_commit: 4cfc4c4
branch: develop
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/phase-f1-delta-analysis/delta-analysis.md
  - .factory/research/modbus-tcp-research.md
---

# F2 Architecture Delta — Issue #7: Modbus TCP Protocol Analyzer

## 1. Overview

This document is the complete architecture delta for Feature #7. It defines all new
types, state structures, integration contracts, and spec-update obligations needed before
F3 story decomposition begins. It does NOT author behavioral contracts (BC-2.14.NNN) or
VP-022 — those are product-owner and formal-verifier responsibilities respectively.
It defines the structures those artifacts depend on.

Approved scope (human decision):
- 7 MITRE ATT&CK for ICS techniques: T0855, T0836, T0814, T0806, T0835, T0831, T0846
- FULL transaction correlation: per-connection (Transaction-ID, Unit-ID, FC) table
- CLI-configurable `--modbus-write-threshold` (default 10 writes per 1-second window per flow)

ADR produced: **ADR-005** (`.factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md`)

---

## 2. SS-14 "Modbus/ICS Analysis" Subsystem Shard

### 2.1 Component Definition: C-22 ModbusAnalyzer

| Field | Value |
|-------|-------|
| Component ID | C-22 |
| File | `src/analyzer/modbus.rs` |
| Subsystem | SS-14 |
| Traits | `StreamHandler` + `StreamAnalyzer` (from `src/reassembly/handler.rs`) |
| Purity | Pure core (see §2.5) |

`ModbusAnalyzer` follows the same internal pattern established by `HttpAnalyzer` (C-12)
and `TlsAnalyzer` (C-13) per ADR-0002 §Internal Structure Pattern.

### 2.2 Top-Level Struct Layout

```rust
pub struct ModbusAnalyzer {
    /// Per-flow state, keyed by FlowKey.
    flows: HashMap<FlowKey, ModbusFlowState>,

    /// Global write-burst rate threshold (writes per second, sustained window).
    /// Exposed via CLI --modbus-write-threshold. Default = 10.
    write_threshold: u32,

    /// Aggregate function-code distribution across all flows: FC byte → count.
    fn_code_counts: HashMap<u8, u64>,

    /// Aggregate write-operation count across all flows.
    total_write_count: u64,

    /// Aggregate exception-response count across all flows.
    total_exception_count: u64,

    /// Total PDU count processed across all flows.
    total_pdu_count: u64,

    /// All findings emitted across all flows. Bounded by MAX_FINDINGS.
    all_findings: Vec<Finding>,

    /// MBAP parse error count (PDUs that failed the 3-point validity gate).
    parse_errors: u64,

    /// Count of findings silently dropped because all_findings.len() >= MAX_FINDINGS.
    /// Present in summarize() output ALWAYS (even when 0). See §2.7.
    dropped_findings: u64,

    /// Monotonic count of flows for which at least one PDU was processed.
    /// Incremented on first PDU per flow (HashMap insertion). NOT derived from
    /// self.flows.len() — on_flow_close removes entries, so len() → 0 at end.
    total_flows_analyzed: u64,

    /// Internal diagnostic counter: number of pending-table entries that were
    /// overwritten before a response was received (Transaction ID reuse / pipeline
    /// exhaustion). Incremented when `ModbusFlowState.pending.insert((txn_id, unit_id), ...)`
    /// returns `Some(_)` (i.e., a previous request was displaced without a matching response).
    /// NOT exposed as a summarize() key in v1 (BC-2.14.021 six-key contract is unchanged).
    duplicate_inflight_txn: u64,
}

impl ModbusAnalyzer {
    pub fn new(write_threshold: u32) -> Self { ... }
    pub fn parse_error_count(&self) -> u64 { self.parse_errors }
}
```

**MAX_FINDINGS cap:** `ModbusAnalyzer` uses the same `MAX_FINDINGS = 10_000` cap as the
reassembly engine and existing analyzers. When `all_findings.len() >= MAX_FINDINGS`, no new
findings are pushed (poison-skip model consistent with `HttpAnalyzer` and `TlsAnalyzer`).
The cap is applied per-finding push site: each finding emission point guards with
`if self.all_findings.len() < MAX_FINDINGS { self.all_findings.push(...); }`.

### 2.3 Per-Flow State: `ModbusFlowState`

```rust
struct ModbusFlowState {
    // --- Transaction correlation ---
    /// (transaction_id, unit_id) → (request_fc: u8, timestamp: u32).
    /// Bounded to MAX_PENDING_TRANSACTIONS = 256 entries.
    pending: HashMap<(u16, u8), (u8, u32)>,

    // --- Per-flow aggregate counters (all-time) ---
    /// Total write-class FCs seen in this flow (request direction, all time).
    write_count: u64,
    /// Total exception-response PDUs (FC >= 0x80) seen in this flow (all time).
    exception_count: u64,
    /// Total PDUs processed in this flow (valid ADUs past the 3-point gate).
    pdu_count: u64,
    /// Timestamp of the last PDU processed (pcap-relative u32 microseconds).
    last_ts: u32,

    // --- T0806/T0855 write-burst rate window (configurable threshold, 1-second fixed) ---
    /// Write-class FC count within the current 1-second rate-detection window.
    window_write_count: u32,
    /// Start timestamp of the current 1-second write-rate window (pcap-relative u32).
    window_start_ts: u32,
    /// True once T0806/T0855 burst has fired in the current window; reset on window expiry.
    window_burst_emitted: bool,

    // --- T0831 coordinated-write window (5-second fixed, not CLI-configurable) ---
    /// Start timestamp of the current T0831 5-second window (pcap-relative u32).
    t0831_window_start_ts: u32,
    /// Holding-register write count in the current T0831 5-second window.
    t0831_window_write_count: u32,
    /// True once T0831 has fired in the current window; reset on window expiry.
    t0831_burst_emitted: bool,

    // --- BC-2.14.019 exception-burst windows (per exception code byte) ---
    /// Per-exception-code count within the current 10-second window.
    exception_window_counts: HashMap<u8, u32>,
    /// Per-exception-code window start timestamp (pcap-relative u32).
    exception_window_start_ts: HashMap<u8, u32>,
    /// Per-exception-code burst-emitted flag; reset on window expiry for that code.
    exception_burst_emitted: HashMap<u8, bool>,

    // --- Desync safety (Decision 6) ---
    /// True if a protocol_id != 0x0000 failure was seen; flow is silently ignored thereafter.
    is_non_modbus: bool,
}
```

**`MAX_PENDING_TRANSACTIONS = 256`**: when the `pending` table reaches 256 entries (indicating
heavy pipelining or a pathological capture), new request entries are NOT inserted (preventing
unbounded growth). Existing entries continue to be matched and removed on response.

**Write-rate window model**: the window-based rate detector uses a 1-second sliding window
(fixed: `WRITE_RATE_WINDOW_SECS = 1`). On each write-class FC:
1. If `now_ts - window_start_ts > WRITE_RATE_WINDOW_SECS * 1_000_000` (microseconds), reset
   `window_write_count = 1`, `window_start_ts = now_ts`, and `window_burst_emitted = false`.
2. Otherwise increment `window_write_count`.
3. If `window_write_count > write_threshold` AND `!window_burst_emitted`, emit T0806 + T0855
   burst findings and set `window_burst_emitted = true`.

The `u32` timestamp is the pcap-relative capture timestamp in microseconds, matching the
existing `timestamp: u32` parameter in `StreamHandler::on_data`.

### 2.4 MBAP Parse Model

**Wire layout** (Modbus.org spec V1.1b3, big-endian):

```
Offset  Size  Field              Value / Semantics
------  ----  -----------------  ------------------------------------
0–1     2     Transaction ID     Session counter echoed in response
2–3     2     Protocol ID        ALWAYS 0x0000 for Modbus
4–5     2     Length             Byte count from Unit ID onward (= 1 + PDU len)
6       1     Unit ID            Slave/sub-unit address
7+      1+    PDU                Function Code (1 byte) + data
```

**Minimum ADU size**: 8 bytes (7-byte MBAP header + 1-byte Function Code minimum PDU).
**Maximum ADU size**: 260 bytes (7-byte MBAP + 253-byte PDU maximum per spec V1.1b3).
**Valid Length range**: 2 ≤ Length ≤ 253 (Unit ID byte + 1..252 PDU bytes).

**`parse_mbap_header` — pure core function** (VP-022 Kani target):

```rust
/// Parses a Modbus MBAP header from a byte slice. Returns None if the
/// slice is shorter than 8 bytes (minimum valid ADU). Does NOT validate
/// Protocol ID or Length range — caller applies the 3-point gate.
///
/// Pure: no I/O, no global state. Kani-provable for all inputs.
fn parse_mbap_header(data: &[u8]) -> Option<MbapHeader> {
    if data.len() < 8 { return None; }
    Some(MbapHeader {
        transaction_id: u16::from_be_bytes([data[0], data[1]]),
        protocol_id:    u16::from_be_bytes([data[2], data[3]]),
        length:         u16::from_be_bytes([data[4], data[5]]),
        unit_id:        data[6],
        function_code:  data[7],
    })
}

struct MbapHeader {
    transaction_id: u16,
    protocol_id:    u16,
    length:         u16,
    unit_id:        u8,
    function_code:  u8,
}
```

**Three-point validity gate** (applied in `on_data` after `parse_mbap_header` returns `Some`):

```rust
fn is_valid_modbus_adu(h: &MbapHeader) -> bool {
    h.protocol_id == 0x0000
        && h.length >= 2
        && h.length <= 253
}
```

**Desync / DoS safety policy** (Decision 6 — applied PER GATE FAILURE TYPE):

- **Protocol-ID failure** (`h.protocol_id != 0x0000`): set `flow.is_non_modbus = true` and
  return immediately from `on_data`. Do NOT advance by `6 + h.length` (attacker-controlled).
  All subsequent `on_data` calls for this flow key bail out at entry if `is_non_modbus`. This
  handles non-Modbus binary traffic misrouted to port 502 without DoS from a crafted Length.

- **Length-range failure** (`h.length < 2 || h.length > 253`): increment `parse_errors` and
  `break` the parsing loop. Do not advance by the malformed Length value. Discard the rest of
  the current segment (wait for next `on_data`). This is the safe default that prevents
  attacker-controlled offset arithmetic.

```rust
// After parse_mbap_header returns Some(h):
if h.protocol_id != 0x0000 {
    flow.is_non_modbus = true;
    return;  // bail — entire flow is non-Modbus
}
if h.length < 2 || h.length > 253 {
    self.parse_errors += 1;
    break;   // discard rest of segment — no attacker-controlled advance
}
```

**PDU boundary advancement** (in the `on_data` parsing loop, after gate passes):

```rust
// Total ADU size: 6-byte MBAP prefix (before Length field) + Length value
let adu_size = 6usize + (h.length as usize);
if offset + adu_size > data.len() {
    break; // incomplete ADU — wait for more data
}
offset += adu_size;
```

This is the same offset-advancing pattern used by `TlsAnalyzer`. Multiple ADUs per TCP
segment are handled by the loop; partial trailing ADUs are left for the next `on_data` call
(the reassembler delivers contiguous in-order bytes). Offset advance by `6 + length` is
only reached for ADUs that PASSED the three-point gate, bounding the `length` value to [2, 253]
before it is used in arithmetic.

### 2.5 Function-Code Classification

**`classify_fc` — pure core function** (VP-022 Kani target):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCodeClass {
    /// Read operations: 0x01, 0x02, 0x03, 0x04, 0x07, 0x0B, 0x0C, 0x11, 0x14, 0x18
    Read,
    /// Write operations: 0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17
    Write,
    /// Management/diagnostic: 0x08 (Diagnostics), 0x2B (MEI)
    Diagnostic,
    /// Exception response: fc >= 0x80 (high bit set). Original FC = fc & 0x7F.
    Exception,
    /// Unrecognized FC: not in the standard set and not an exception.
    Unknown,
}

/// Pure total function over all 256 FC values. Kani-provable exhaustive.
fn classify_fc(fc: u8) -> FunctionCodeClass {
    if fc >= 0x80 { return FunctionCodeClass::Exception; }
    match fc {
        0x01 | 0x02 | 0x03 | 0x04 | 0x07 | 0x0B | 0x0C | 0x11 | 0x14 | 0x18 => {
            FunctionCodeClass::Read
        }
        0x05 | 0x06 | 0x0F | 0x10 | 0x15 | 0x16 | 0x17 => {
            FunctionCodeClass::Write
        }
        0x08 | 0x2B => FunctionCodeClass::Diagnostic,
        _ => FunctionCodeClass::Unknown,
    }
}
```

**Exception detection invariant**: `classify_fc(fc) == Exception` if and only if `fc >= 0x80`.
This is VP-022 sub-property C and is directly Kani-provable.

### 2.6 Finding Emission Points — MITRE Technique Mapping

Each detection maps to one or more approved MITRE ATT&CK for ICS techniques:

| Detection trigger | Modbus signal | MITRE ID | Technique name | Tactic |
|-------------------|---------------|----------|----------------|--------|
| Write FC from request direction (any Write-class FC) | FC in {0x05, 0x06, 0x0F, 0x10, 0x15, 0x16, 0x17} | **T0855** | Unauthorized Command Message | IcsImpairProcessControl |
| Write FC to holding registers only | FC in {0x06, 0x10, 0x16}; T0836 takes priority over T0835 for these FCs | **T0836** | Modify Parameter | IcsImpairProcessControl |
| Write FC to coils/I/O (NOT in T0836 subset) | FC in {0x05, 0x0F} — coil writes only (T0836 not applicable) | **T0835** | Manipulate I/O Image | IcsImpairProcessControl |
| Coordinated write sequence to holding registers within 5-second window | ≥2 FCs in {0x06, 0x10, 0x16} within 5s in same flow | **T0831** | Manipulation of Control | IcsImpairProcessControl |
| Write burst exceeds rate threshold in 1-second window | >N write FCs in 1s to same flow; N = `write_threshold` | **T0806** | Brute Force I/O | IcsImpairProcessControl |
| Burst T0855 companion (once per burst event, not per PDU) | Same burst event as T0806; `window_burst_emitted` guard | **T0855** | Unauthorized Command Message (burst-level) | IcsImpairProcessControl |
| Diagnostics FC 0x08 sub-func 0x0004 (Force Listen Only) | Single-packet near-zero-FP signal | **T0814** | Denial of Service | IcsInhibitResponseFunction |
| Diagnostics FC 0x08 sub-func 0x0001 (Restart Communications) | Single-packet near-zero-FP signal | **T0814** | Denial of Service | IcsInhibitResponseFunction |
| Recon FCs: 0x11 Report Server ID or 0x2B/0x0E Read Device ID | Reconnaissance in steady-state SCADA environment | **T0846** | Remote System Discovery | IcsDiscovery |

**Diagnostic sub-function parsing**: when `fc == 0x08` and the PDU has at least 2 more data
bytes, read `sub_func = u16::from_be_bytes([data[8], data[9]])`. Sub-functions 0x0004 and
0x0001 emit T0814. Sub-function 0x000A (Clear Counters) emits an `Anomaly`-category finding
(no clean single ICS ATT&CK ID per research §5, flagged as anti-forensic indicator).

**`ThreatCategory` usage**: no new `ThreatCategory` variant is required for v1.
- T0855, T0836, T0806, T0835, T0831 → `ThreatCategory::Execution`
- T0814 (DoS) → `ThreatCategory::Anomaly`
- T0846 (recon) → `ThreatCategory::Anomaly`
- Exception burst → `ThreatCategory::Anomaly`
- Unknown FC / Clear Counters → `ThreatCategory::Anomaly`

The `#[non_exhaustive]` attribute on `ThreatCategory` ensures future ICS-specific variants
can be added without breaking existing match arms.

**Per-PDU Multi-Technique Co-Emission Policy (Decision 7 — v1 detection policy):**

A write-class PDU emits findings using a priority/selection rule to bound amplification:

1. **T0836 vs T0835 priority:** T0836 (holding-register writes: FC 0x06/0x10/0x16) takes
   priority. When T0836 fires, T0835 does NOT fire for the same PDU. T0835 fires ONLY
   for coil-only writes (FC 0x05, 0x0F) where T0836 does not apply.

2. **T0855 (per-write):** always emitted once per write-class PDU, independent of T0836/T0835.

3. **T0806 + burst T0855:** emitted at most ONCE per 1-second window overflow (burst event),
   not per write PDU within the burst. Guards: `window_burst_emitted` flag.

4. **T0831:** emitted at most ONCE per 5-second window overflow per flow. Guard:
   `t0831_burst_emitted` flag.

**Maximum findings for a single write PDU (worst case — triggers both burst events):**

| Scenario | Findings emitted | Count |
|----------|------------------|-------|
| Holding-register write (FC=0x06) that tips T0806 threshold AND completes T0831 window | T0836 + T0855 (per-write) + T0806 + T0855 (burst companion) + T0831 | 5 |
| Holding-register write (FC=0x06), mid-burst (both burst flags already set) | T0836 + T0855 | 2 |
| Coil write (FC=0x05) that tips T0806 threshold | T0835 + T0855 (per-write) + T0806 + T0855 (burst companion) | 4 |

This policy prevents a write flood from producing N × 3 or N × 4 findings per PDU and
exhausting MAX_FINDINGS. It is a deliberate v1 detection policy scoping decision.

### 2.7 on_data and on_flow_close Implementation Contract

**`on_data` signature** (implements `StreamHandler`):
```rust
fn on_data(
    &mut self,
    flow_key: &FlowKey,
    direction: Direction,
    data: &[u8],
    offset: u64,
    timestamp: u32,
)
```

**on_data logic sketch**:
1. Look up or insert `ModbusFlowState` for `flow_key` (effectful: HashMap mutation).
2. Parse all complete ADUs from `data` using the offset-advancing loop.
3. For each ADU: apply 3-point validity gate; skip invalid ADUs.
4. Infer direction: `direction == Direction::ClientToServer` → request; else response.
   (The `Direction` enum from `handler.rs` uses TCP connection direction, not port analysis.)
5. For requests: insert into `pending` table (if table is not full), update write counters,
   run rate-burst detection, emit findings.
6. For responses: look up `pending` by `(transaction_id, unit_id)`, validate FC echo,
   remove entry, emit attribution findings (e.g. exception on write FC → T0855 evidence).
7. Update aggregate counters (`fn_code_counts`, `total_write_count`, etc.).
8. Update `flow.last_ts = timestamp`.

**`on_flow_close` logic**: remove the flow's entry from `self.flows`. No findings are emitted
on close (unlike `HttpAnalyzer` which emits on-close findings for some patterns). The
`pending` table is dropped with the flow state — orphaned transactions at close are not
flagged (they can legitimately occur on half-captured connections).

**`summarize` output** (implements `StreamAnalyzer`) — SIX keys:
```rust
// Returns AnalysisSummary with detail keys:
// "pdu_count"                  → Value::Number(total_pdu_count)
// "write_count"                → Value::Number(total_write_count)
// "exception_count"            → Value::Number(total_exception_count)
// "function_code_distribution" → Value::Object { "0x03": count, "0x10": count, ... }
//                                (only FCs with count > 0; hex-string keys)
// "parse_errors"               → Value::Number(parse_errors)
// "dropped_findings"           → Value::Number(dropped_findings)
//                                (ALWAYS present; 0 when MAX_FINDINGS cap not reached)
//
// NOTE: "flows_analyzed" is NOT derived from self.flows.len(). on_flow_close removes
// entries from self.flows (bounded memory). summarize() uses self.total_flows_analyzed
// (monotonic counter, incremented on first PDU per flow). BC-2.14.021 does not currently
// expose total_flows_analyzed as a summary key; it is available internally.
```

**`findings` output**: returns `all_findings.clone()` (mirrors `HttpAnalyzer::findings`).

### 2.8 Purity Boundary for ModbusAnalyzer

| Function / component | Classification | Rationale |
|----------------------|---------------|-----------|
| `parse_mbap_header(data: &[u8]) -> Option<MbapHeader>` | **Pure core** | Deterministic on `data`; no I/O; VP-022 Kani target sub-property A |
| `is_valid_modbus_adu(h: &MbapHeader) -> bool` | **Pure core** | Boolean predicate on struct fields; no I/O |
| `classify_fc(fc: u8) -> FunctionCodeClass` | **Pure core** | Total function over u8; VP-022 Kani target sub-property B/C |
| Diagnostic sub-function extraction | **Pure core** | Slice index + BE decode; no I/O |
| `on_data` loop (HashMap mutation, counter increment) | **Effectful shell** | Mutates `self.flows`, `self.all_findings`; not formally provable |
| `on_flow_close` (HashMap remove) | **Effectful shell** | Mutates `self.flows` |
| `summarize` / `findings` | **Effectful shell** | Reads self state; returns owned data |

The pure core functions are extracted as `fn` (not `impl ModbusAnalyzer`) so they can be
called from Kani harnesses without constructing the full analyzer struct.

---

## 3. Dispatcher Integration Design

### 3.1 DispatchTarget::Modbus Variant

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchTarget {
    Http,
    Tls,
    Modbus,   // NEW — ADR-005
    None,
}
```

This enum is private to `dispatcher.rs`. All exhaustive match arms must be updated.

### 3.2 StreamDispatcher Struct Changes

```rust
pub struct StreamDispatcher {
    routes: HashMap<FlowKey, DispatchTarget>,
    classification_attempts: HashMap<FlowKey, u32>,
    max_classification_attempts: u32,
    http: Option<HttpAnalyzer>,
    tls: Option<TlsAnalyzer>,
    modbus: Option<ModbusAnalyzer>,   // NEW
    unclassified_flows: u64,
}

impl StreamDispatcher {
    pub fn new(
        http: Option<HttpAnalyzer>,
        tls: Option<TlsAnalyzer>,
        modbus: Option<ModbusAnalyzer>,  // NEW param
    ) -> Self { ... }

    /// Returns a reference to the Modbus analyzer, if configured.
    pub fn modbus_analyzer(&self) -> Option<&ModbusAnalyzer> {
        self.modbus.as_ref()
    }

    /// Moves the Modbus analyzer out of the dispatcher (post-finalize collection).
    pub fn take_modbus_analyzer(&mut self) -> Option<ModbusAnalyzer> {
        self.modbus.take()
    }
}
```

The `take_*` / `*_analyzer` accessor pattern matches the existing `take_tls_analyzer()` /
`tls_analyzer()` pair exactly. The `http_analyzer()` / `take_http_analyzer()` pair (if it
exists) should also be verified for consistency.

### 3.3 classify() Port-502 Branch — Placement and Precedence

**Production `classify` function** — new arm placement:

```rust
fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget {
    // Rule 1: TLS content signature (content-first, wins over any port).
    if data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03 {
        return DispatchTarget::Tls;
    }
    // Rule 2: HTTP method/response prefix (content-first).
    if data.starts_with(b"GET ")  || data.starts_with(b"POST ")
    || data.starts_with(b"PUT ")  || data.starts_with(b"DELETE ")
    || data.starts_with(b"HEAD ") || data.starts_with(b"OPTIONS ")
    || data.starts_with(b"PATCH ")|| data.starts_with(b"CONNECT ")
    || data.starts_with(b"TRACE ")|| data.starts_with(b"HTTP/") {
        return DispatchTarget::Http;
    }
    // Rule 3: Port fallback — TLS standard ports.
    let ports = [flow_key.lower_port(), flow_key.upper_port()];
    if ports.contains(&443) || ports.contains(&8443) {
        return DispatchTarget::Tls;
    }
    // Rule 4: Port fallback — HTTP standard ports.
    if ports.contains(&80) || ports.contains(&8080) {
        return DispatchTarget::Http;
    }
    // Rule 5: Port fallback — Modbus TCP IANA port (ADR-005 exception to ADR-0001).
    // AFTER all content and TLS/HTTP port fallbacks so no existing flow can be stolen.
    if ports.contains(&502) {
        return DispatchTarget::Modbus;
    }
    // Rule 6: Nothing matched.
    DispatchTarget::None
}
```

**Precedence guarantee**: port 502 is checked only after content rules (rules 1–2) and after
the established 443/8443/80/8080 port fallbacks (rules 3–4). TLS or HTTP content on port 502
would be classified as TLS/HTTP respectively by rules 1–2 before rule 5 is reached. This
preserves INV-2 for existing analyzers.

### 3.4 on_data Routing

**Early-exit guard change** (dispatcher.rs ~line 152):

```rust
// BEFORE:
if self.http.is_none() && self.tls.is_none() {
    return;
}
// AFTER:
if self.http.is_none() && self.tls.is_none() && self.modbus.is_none() {
    return;
}
```

**on_data match arm addition**:

```rust
match target {
    DispatchTarget::Http => {
        if let Some(ref mut http) = self.http {
            http.on_data(flow_key, direction, data, offset, timestamp);
        }
    }
    DispatchTarget::Tls => {
        if let Some(ref mut tls) = self.tls {
            tls.on_data(flow_key, direction, data, offset, timestamp);
        }
    }
    DispatchTarget::Modbus => {          // NEW
        if let Some(ref mut modbus) = self.modbus {
            modbus.on_data(flow_key, direction, data, offset, timestamp);
        }
    }
    DispatchTarget::None => {}
}
```

### 3.5 on_flow_close Routing

```rust
match target {
    Some(DispatchTarget::Http) => {
        if let Some(ref mut http) = self.http {
            http.on_flow_close(flow_key, reason);
        }
    }
    Some(DispatchTarget::Tls) => {
        if let Some(ref mut tls) = self.tls {
            tls.on_flow_close(flow_key, reason);
        }
    }
    Some(DispatchTarget::Modbus) => {   // NEW
        if let Some(ref mut modbus) = self.modbus {
            modbus.on_flow_close(flow_key, reason);
        }
    }
    Some(DispatchTarget::None) | None => {
        if self.http.is_some() || self.tls.is_some() || self.modbus.is_some() {
            self.unclassified_flows += 1;
        }
    }
}
```

**Note**: the `unclassified_flows` guard must also include `self.modbus.is_some()` so that
a Modbus-only run (`--modbus` without `--http` or `--tls`) correctly counts unclassified flows.

### 3.6 VP-004 Kani Harness Extension — CRITICAL

The `classify_oracle` at `dispatcher.rs` ~line 283 must mirror the production `classify`
function EXACTLY, including the new port-502 arm. Failing to add this arm would cause
`verify_content_first_precedence_exhaustive` to fail because `got` (from production) would
return `Modbus` for port-502 flows while `want` (from the oracle) would return `None`.

**Extended `classify_oracle`**:

```rust
fn classify_oracle(data: &[u8; 8], lower: u16, upper: u16) -> DispatchTarget {
    // Rule 1: TLS content signature.
    if data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03 {
        return DispatchTarget::Tls;
    }
    // Rule 2: HTTP method tokens (identical set/order to production).
    if data.starts_with(b"GET ")     || data.starts_with(b"POST ")
    || data.starts_with(b"PUT ")     || data.starts_with(b"DELETE ")
    || data.starts_with(b"HEAD ")    || data.starts_with(b"OPTIONS ")
    || data.starts_with(b"PATCH ")   || data.starts_with(b"CONNECT ")
    || data.starts_with(b"TRACE ")   || data.starts_with(b"HTTP/") {
        return DispatchTarget::Http;
    }
    // Rule 3: TLS port fallback.
    let ports = [lower, upper];
    if ports.contains(&443) || ports.contains(&8443) {
        return DispatchTarget::Tls;
    }
    // Rule 4: HTTP port fallback.
    if ports.contains(&80) || ports.contains(&8080) {
        return DispatchTarget::Http;
    }
    // Rule 5: Modbus port fallback (ADR-005 — MUST mirror production exactly).
    if ports.contains(&502) {
        return DispatchTarget::Modbus;
    }
    // Rule 6: None.
    DispatchTarget::None
}
```

**`verify_content_first_precedence_exhaustive` interaction**: this proof uses a symbolic 8-byte
`data` array. Port 502 is now reachable via symbolic `port_a` or `port_b`. The proof will
cover the port-502 → Modbus branch for all inputs that don't match rules 1–4, confirming
that Modbus only fires when no content rule applies and no TLS/HTTP port is present.

**Additional VP-004 proof suggested** (not mandatory for P0 re-verification, but recommended):

```rust
#[kani::proof]
fn verify_modbus_port_beats_none_not_http_or_tls() {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let key = FlowKey::new(ip, 502, ip, 12345); // lower_port or upper_port == 502
    // Data that matches no content rule (not 0x16 0x03, not HTTP method).
    let b0: u8 = kani::any();
    let b1: u8 = kani::any();
    kani::assume(!(b0 == 0x16 && b1 == 0x03)); // not TLS
    let data: [u8; 8] = [b0, b1, 0, 0, 0, 0, 0, 0];
    kani::assume(!data.starts_with(b"GET ") && !data.starts_with(b"POST "));
    // etc. for remaining method tokens
    assert!(matches!(classify(&data, &key), DispatchTarget::Modbus));
}
```

The exact assume-list must cover all HTTP method tokens that fit in 8 bytes.

---

## 4. MITRE Type Design

### 4.1 Matrix Discriminator Decision

**Decision**: Use technique-ID namespace as the implicit matrix discriminator, plus a
`technique_matrix(id: &str) -> Option<Matrix>` pure lookup function. No new field is added
to `Finding` or `MbapHeader`. This is the minimal-change approach consistent with the existing
`technique_info` pattern.

```rust
/// MITRE ATT&CK matrix identifier — distinguishes ICS from Enterprise technique IDs.
/// ICS technique IDs use the T0xxx namespace (e.g. T0855, T0836).
/// Enterprise technique IDs use the T1xxx–T9xxx namespace (e.g. T1027, T1071.001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MitreMatrix {
    Enterprise,
    Ics,
}

/// Resolves a technique ID to its ATT&CK matrix.
/// ICS: T0[0-9]{3} (5-char IDs with second char '0').
/// Enterprise: all other valid T[0-9]{4}[.[0-9]{3}] IDs.
/// Returns None for malformed or unknown IDs.
pub fn technique_matrix(id: &str) -> Option<MitreMatrix> {
    let b = id.as_bytes();
    // Must be a valid technique ID format first.
    if !is_valid_technique_id_format(id) {
        return None;
    }
    // ICS: 5-char ID, T0[0-9]{3} — second byte is b'0'.
    if b.len() == 5 && b[1] == b'0' {
        return Some(MitreMatrix::Ics);
    }
    // Enterprise: T[1-9][0-9]{3} or T[1-9][0-9]{3}.[0-9]{3}.
    Some(MitreMatrix::Enterprise)
}
```

**Rationale for namespace rule**: MITRE ATT&CK for ICS uses `T0xxx` (T + 0 + 3 digits)
precisely to avoid collision with Enterprise `T1xxx`–`T9xxx`. The rule `b[1] == b'0'` maps
the ID prefix to the matrix without any table lookup. It is provable: `classify_matrix(fc)`
is a pure function with no state. `is_valid_technique_id_format` already exists in `mitre.rs`
(gated `#[cfg(any(kani, test))` — if needed publicly, its gate must be widened or a parallel
public version added).

**Alternative considered and rejected**: adding `matrix: MitreMatrix` as a third element of
the `technique_info` return tuple `(name, tactic, matrix)`. This would require updating all
callers of `technique_info`. The namespace-rule approach adds zero call-site churn.

### 4.2 New technique_info Arms — T0836, T0814, T0806, T0835, T0831

The following arms must be added to `technique_info` in `src/mitre.rs`. They follow the
existing ICS arm style (line 143–152):

```rust
// ICS — existing arms (no change):
"T0846" => ("Remote System Discovery",   MitreTactic::Discovery),
"T0855" => ("Unauthorized Command Message", MitreTactic::IcsImpairProcessControl),
"T0856" => ("Spoof Reporting Message",   MitreTactic::IcsImpairProcessControl),
"T0885" => ("Commonly Used Port",        MitreTactic::CommandAndControl),

// ICS — new arms for Modbus feature (ADR-005); 5 new seeded + T0846 now emitted:
"T0836" => ("Modify Parameter",          MitreTactic::IcsImpairProcessControl),
"T0814" => ("Denial of Service",         MitreTactic::IcsInhibitResponseFunction),
"T0806" => ("Brute Force I/O",           MitreTactic::IcsImpairProcessControl),
"T0835" => ("Manipulate I/O Image",      MitreTactic::IcsImpairProcessControl),
"T0831" => ("Manipulation of Control",   MitreTactic::IcsImpairProcessControl),
// T0846 arm above covers recon findings from BC-2.14.020 (already seeded, now emitted).
```

### 4.3 VP-007 Atomic Update Set

Every change to `technique_info` requires the following five constants/arrays to be updated
in the SAME commit (enforced by `vp007_catalog_drift_guard`):

| Constant / Array | Current value | New value after Modbus feature |
|-----------------|--------------|-------------------------------|
| `SEEDED_TECHNIQUE_IDS` | 15 entries (T1027..T0885) | 20 entries — add T0836, T0814, T0806, T0835, T0831 (T0846 already seeded) |
| `SEEDED_TECHNIQUE_ID_COUNT` | `15` | `20` |
| `EMITTED_IDS` in `kani_proofs` | 6 Enterprise IDs | 6 Enterprise IDs + 7 ICS IDs = **13 total** |

**`EMITTED_IDS` after update** (all IDs that appear in `mitre_technique: Some(...)` calls
across `src/`, after Modbus is implemented):

```rust
const EMITTED_IDS: &[&str] = &[
    // Enterprise (6, existing — no change):
    "T1027",      // TLS: SNI anomaly
    "T1036",      // Reassembly: conflicting overlap
    "T1046",      // HTTP: admin panel
    "T1083",      // HTTP: path traversal
    "T1499.002",  // HTTP: header flood
    "T1505.003",  // HTTP: web shell
    // ICS — 7 new (Modbus analyzer, ADR-005):
    "T0855",      // Modbus: write-class FC / unauthorized command + burst companion
    "T0836",      // Modbus: 0x06/0x10/0x16 parameter writes (holding registers)
    "T0814",      // Modbus: 0x08 Force Listen Only / Restart Comms
    "T0806",      // Modbus: write burst rate exceeded (1-second window)
    "T0835",      // Modbus: I/O image manipulation writes (coil-only)
    "T0831",      // Modbus: coordinated write sequence (5-second window)
    "T0846",      // Modbus: recon FCs (0x11 Report Server ID, 0x2B/0x0E Read Device ID)
];
// Total: 6 Enterprise + 7 ICS = 13
```

**Note on T0855 and T0846 pre-existing gaps**: T0855 and T0846 are both seeded in
`SEEDED_TECHNIQUE_IDS` but are NOT currently in `EMITTED_IDS` (confirmed at `mitre.rs`
lines 191–198). Both gaps are fixed in the same Modbus feature commit: T0855 is added because
the Modbus analyzer emits it; T0846 is added because the Modbus analyzer emits it for recon
FCs (0x11 / 0x2B/0x0E) per BC-2.14.020 (post-decision-8 fix). This keeps VP-007
sub-property B (emitter half) sound.

**Updated `SEEDED_TECHNIQUE_IDS`**:
```rust
const SEEDED_TECHNIQUE_IDS: &[&str] = &[
    // Enterprise (11, unchanged)
    "T1027", "T1036", "T1040", "T1046", "T1071",
    "T1071.001", "T1071.004", "T1083", "T1499.002", "T1505.003", "T1573",
    // ICS (9 total — was 4, add 5 new)
    "T0846", "T0855", "T0856", "T0885",        // existing
    "T0836", "T0814", "T0806", "T0835", "T0831", // new
];
// Total: 11 + 9 = 20
```

**`SEEDED_TECHNIQUE_ID_COUNT`**: change from `15` to `20`.

### 4.4 all_tactics_in_report_order Impact

No change required. `MitreTactic::IcsInhibitResponseFunction` (for T0814) and
`MitreTactic::IcsImpairProcessControl` (for T0836/T0806/T0835/T0831/T0855) are already
present in `all_tactics_in_report_order()`. The terminal reporter's MITRE grouping loop
will automatically include them as Modbus findings are emitted.

---

## 5. CLI Integration

### 5.1 New CLI Flags

**`src/cli.rs`** — additions to `Commands::Analyze` variant:

```rust
/// Enable Modbus TCP protocol analyzer (TCP port 502).
#[arg(long)]
modbus: bool,

/// Per-flow write-burst rate threshold (write FCs per second).
/// Fires T0806/T0855 finding when sustained write rate exceeds this value.
/// Default: 10. CLI-configurable per approved F2 scope.
#[arg(long, default_value_t = 10)]
modbus_write_threshold: u32,
```

The `--all` flag expansion in `run_analyze` must include `modbus`:
```rust
let enable_modbus = *modbus || *all;
```

### 5.2 main.rs Four-Step Wiring Pattern

```rust
// Step 1: Construct ModbusAnalyzer (mirrors HttpAnalyzer / TlsAnalyzer construction).
let modbus_analyzer = if enable_modbus && !skip_reassembly {
    Some(ModbusAnalyzer::new(modbus_write_threshold))
} else {
    None
};

// Step 2: Extend needs_reassembly (must include Modbus or --modbus alone is silent).
let needs_reassembly = enable_http || enable_tls || enable_modbus;

// Step 3: Pass to StreamDispatcher::new (signature changes to 3 params).
let dispatcher = StreamDispatcher::new(http_analyzer, tls_analyzer, modbus_analyzer);

// Step 4 (post-finalize): Take Modbus analyzer and collect findings + summary.
if let Some(modbus) = dispatcher.take_modbus_analyzer() {
    all_findings.extend(modbus.findings());
    analyzer_summaries.push(modbus.summarize());
}
```

Omitting step 2 is the "silent analysis" regression risk flagged in the F1 delta analysis §4.
The BC-2.14.NNN integration contracts will include a postcondition verifying that `--modbus`
alone triggers reassembly.

---

## 6. Dependency Graph — Acyclicity Verification

`ModbusAnalyzer` (C-22) has the same dependency shape as `HttpAnalyzer` (C-12) and
`TlsAnalyzer` (C-13):

```
src/analyzer/modbus.rs
  imports:
    crate::analyzer::AnalysisSummary    (L3 — analyzer/mod.rs)
    crate::findings::{Finding, ...}     (L3 — findings.rs)
    crate::mitre::{technique_info, ...} (L3 — mitre.rs)
    crate::reassembly::flow::FlowKey    (L2 — reassembly/flow.rs)
    crate::reassembly::handler::{...}   (L2 — reassembly/handler.rs)
    std::collections::HashMap
```

No new edges are introduced that differ from the existing `analyzer/http.rs` or
`analyzer/tls.rs` import sets. `ModbusAnalyzer` does NOT import:
- `src/dispatcher.rs` (no back-reference to the dispatcher)
- `src/reassembly/mod.rs` (no reference to TcpReassembler internals)
- Any L4 reporter module

The L2←→L3 accepted cycle (handler.rs defines traits; analyzers implement them;
dispatcher.rs holds concrete analyzers) is unchanged in kind. Adding a third concrete
analyzer (modbus.rs) to the cycle does not create a new cycle — it adds one more node
to the existing accepted cycle per ADR-0002 §Consequences.

**DAG update** (addition to dependency-graph.md):

```
src/dispatcher.rs
  |-- reassembly/handler.rs
  |-- analyzer/http.rs
  |-- analyzer/tls.rs
  |-- analyzer/modbus.rs    [NEW — C-22, same import pattern as http/tls]
```

The graph remains acyclic at the file level (modbus.rs does not import dispatcher.rs).

---

## 7. Purity Boundary Map — ModbusAnalyzer Addition

Addition to purity-boundary-map.md §Per-Module Classification:

| Module | Classification | Rationale |
|--------|---------------|-----------|
| src/analyzer/modbus.rs (C-22) | **Pure core** | Stream-level; all state per-instance `HashMap<FlowKey, ModbusFlowState>`; pure core functions (`parse_mbap_header`, `classify_fc`, `is_valid_modbus_adu`) are formally verifiable via Kani (VP-022); no global side effects. Mirrors HttpAnalyzer and TlsAnalyzer purity classification. |

Addition to the purity boundary diagram — L3 Domain layer pure core column:
```
| src/analyzer/modbus.rs (C-22) |
```

---

## 8. Module Criticality Classification — ModbusAnalyzer

Addition to `.factory/specs/module-criticality.md` §HIGH Modules:

| Module | File | Rationale |
|--------|------|-----------|
| Modbus TCP analyzer | src/analyzer/modbus.rs | ICS/OT threat detection. Bugs in MBAP parsing or function-code classification produce incorrect findings or miss attack signals. The pure core functions (parse_mbap_header, classify_fc) are verified by VP-022. The finding-emission logic (write-burst detection, T0814 Diagnostics detection) is HIGH-criticality per the same reasoning as HttpAnalyzer and TlsAnalyzer. Target kill rate ≥ 90%. |

Note: `module-criticality.md` is currently frozen at 2026-06-02 per Phase-5 gate rule. The
feature cycle lifecycle allows additions of new modules; this addition is authorized per
"Module-Criticality Lifecycle" in ARCH-INDEX §Architecture Evolution (DF-030): new modules
added by feature cycles get criticality classification.

---

## 9. VP-022 Design for Formal Verifier

**This section designs the VP for the formal verifier to author — the architect does NOT
write the VP document itself.**

| Field | Value |
|-------|-------|
| Proposed ID | VP-022 |
| Title | Modbus MBAP Parse Safety and Function-Code Boundary Classification |
| Module | analyzer/modbus.rs |
| Tool | Kani |
| Phase assignment | P1 |
| Traces to BCs | BC-2.14.001 (MBAP accept), BC-2.14.002 (MBAP reject short), BC-2.14.003 (MBAP reject Protocol-ID), BC-2.14.004 (MBAP reject Length), BC-2.14.005 (classify_fc totality), BC-2.14.006 (FC Exception detection), BC-2.14.007 (FC Write-class), BC-2.14.008 (FC Diagnostic) |

**Sub-properties**:

- **Sub-property A**: `parse_mbap_header(data)` returns `None` for any `data` shorter than 8
  bytes; returns `Some(_)` for any `data` of length ≥ 8. Never panics.
  Anchors: BC-2.14.001 (accept), BC-2.14.002 (reject short), BC-2.14.003 (Protocol-ID gate),
  BC-2.14.004 (Length gate).
  Kani harness: symbolic `&[u8]` of all lengths 0..16; assert no panic, assert return
  matches length predicate.

- **Sub-property B**: `classify_fc` is total over all 256 FC values. Every `u8` input returns
  one of `{Read, Write, Diagnostic, Exception, Unknown}`. No gaps, no panics.
  Anchors: BC-2.14.005 (totality), BC-2.14.007 (Write-class set), BC-2.14.008 (Diagnostic set).
  Kani harness: `fc: u8 = kani::any()`;  `let _ = classify_fc(fc)` — no assertions needed
  beyond "does not panic" (the match is exhaustive by construction if it covers `_`).

- **Sub-property C**: `classify_fc(fc) == FunctionCodeClass::Exception` if and only if
  `fc >= 0x80` (high bit set).
  Anchors: BC-2.14.006 (Exception detection — FC high-bit set).
  Kani harness: `fc: u8 = kani::any()`;
  `assert_eq!(classify_fc(fc) == FunctionCodeClass::Exception, fc >= 0x80)`.

**Feasibility**: HIGH. All three sub-properties operate on inputs ≤16 bytes or a single `u8`.
Kani state space is trivially bounded. Sub-property C is a simple biconditional on a u8
comparison. Estimated Kani runtime: <1 second (analogous to VP-005 SNI classification proof).

**VP-INDEX propagation obligation** (to be executed by formal-verifier when VP-022 is authored):
- VP-INDEX total: 21 → 22
- VP-INDEX Kani count: 8 → 9
- VP-INDEX P1 count: 7 → 8
- verification-architecture.md: add VP-022 row to "Should Prove" table; add to P1 list
- verification-coverage-matrix.md: add VP-022 row; update analyzer/modbus.rs row (Kani 0→1,
  Total 0→1); update Totals row (Kani 8→9, Total 21→22)

---

## 10. Updated Architecture Section Files — Change Log

The following sharded architecture files require updates (in addition to new SS-14 content
defined in this delta):

| File | Change required |
|------|----------------|
| `ARCH-INDEX.md` | Add SS-14 row to Subsystem Registry; add ADR 0005 row to Architecture Decision Records; update Document Map token estimates; update Bounded-Resource Design note with ModbusAnalyzer constants |
| `module-decomposition.md` | Add C-22 (ModbusAnalyzer) to L3 Domain Layer table under SS-14 |
| `dependency-graph.md` | Add `analyzer/modbus.rs` under `dispatcher.rs` in the DAG |
| `purity-boundary-map.md` | Add modbus.rs row to Per-Module Classification; update diagram |
| `verification-architecture.md` | Add VP-022 to "Should Prove" table; add to P1 list; update Tooling Selection Kani count |
| `verification-coverage-matrix.md` | Add VP-022 row; add analyzer/modbus.rs module row; update Totals |
| `module-criticality.md` | Add ModbusAnalyzer to HIGH Modules (lifecycle allows additions for new feature modules) |

---

## 11. Canonical BC Map for SS-14 (BC-2.14.001–025)

This section records the final 25-BC layout as authored (all BC body files written).
The BC FILE H1 headings are authoritative (Decision 1). This table supersedes any
prior §11 draft. Product-owner must update BC-INDEX to match these titles.

### Group A — MBAP Parse and Validity Gate (4 BCs)

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.001 | MBAP Header Accepted for Well-Formed 8-Byte-Minimum ADU | VP-022 sub-prop A |
| BC-2.14.002 | MBAP Header Rejected for ADU Shorter Than 8 Bytes | VP-022 sub-prop A |
| BC-2.14.003 | MBAP Header Rejected When Protocol ID is Not 0x0000 | 3-point gate; desync bail-out (Decision 6) |
| BC-2.14.004 | MBAP Header Rejected When Length is Outside [2, 253] | 3-point gate; break-loop policy (Decision 6) |

### Group B — Function-Code Classification (4 BCs)

**Canonical concept → BC-ID (Decision 1):**

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.005 | classify_fc Is Total Over All 256 FC Values — Complete Classification Enum | Covers ALL classes (Read, Write, Diagnostic, Exception, Unknown); VP-022 sub-prop B |
| BC-2.14.006 | Exception Response Detection — FC High Bit Set Identifies Exception and Recovers Original FC | VP-022 sub-prop C |
| BC-2.14.007 | Write-Class FC Classification — State-Changing Function Codes Identified as Elevated-Risk | VP-022 sub-prop B |
| BC-2.14.008 | Diagnostic-Class FC Classification and Sub-Function Dispatch (0x08 and 0x2B) | VP-022 sub-prop B |

### Group C — Transaction Correlation (4 BCs)

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.009 | Request PDU Inserted into Per-Flow Pending Table Keyed on (Transaction ID, Unit ID) | |
| BC-2.14.010 | Response PDU Matched Against Pending Table and Entry Removed on FC Echo Match | |
| BC-2.14.011 | Exception Response PDU Attributed to Originating Request FC via Pending Table Lookup | |
| BC-2.14.012 | Pending Table Bounded to MAX_PENDING_TRANSACTIONS=256; New Requests Dropped (Not Evicting) When Full | |

### Group D — Finding Emission: Write-Class Events (3 BCs)

**Priority/selection rule applies (Decision 7): T0836 > T0835 for same PDU; T0855 always fires.**

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.013 | Write-Class FC in Request Direction Emits T0855 (Unauthorized Command Message) Finding | Always fires for any write-class FC |
| BC-2.14.014 | Write FC 0x06/0x10/0x16 in Request Direction Emits T0836 (Modify Parameter) Finding | T0836 takes priority; T0835 NOT emitted for same PDU |
| BC-2.14.015 | Write FC to Coil Output Only ({0x05, 0x0F}) Emits T0835 (Manipulate I/O Image) Finding | T0835 fires only for coil-only FCs {0x05, 0x0F}; suppressed for register FCs {0x06, 0x10, 0x16} where T0836 takes priority |

### Group E — Finding Emission: Coordinated Write and Burst Detection (2 BCs)

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.016 | Coordinated Write Sequence to Holding Registers Within 5-Second Window Emits T0831 Manipulation of Control Finding | FC subset {0x06, 0x10, 0x16}; `T0831_WINDOW_SECS = 5`; once per window |
| BC-2.14.017 | Write-Rate Burst Exceeding --modbus-write-threshold Emits T0806 Brute Force I/O and T0855 Findings | 1-second window; `window_burst_emitted` guard; T0806 + burst-T0855 companion |

### Group F — Finding Emission: Diagnostic/DoS and Anomaly (3 BCs)

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.018 | Diagnostics FC 0x08 Sub-Function 0x0004 or 0x0001 Emits T0814 Denial of Service Finding | Both sub-funcs in ONE BC; per-PDU, near-zero-FP |
| BC-2.14.019 | Exception Response Anomaly — Burst of Exception Codes Emits Anomaly Finding for Recon/Scanning | 10s window per exception code; `exception_window_*` fields |
| BC-2.14.020 | Unusual or Unknown Function Code Observed Emits Anomaly Finding | Unknown-FC path: no MITRE; Recon-FC path (0x11/0x2B/0x0E): T0846 (Decision 8) |

### Group G — Summary and Statistics (2 BCs)

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.021 | summarize() Returns AnalysisSummary with Specified Per-Analyzer Summary Keys | **SIX keys** (Decision 3): pdu_count, write_count, exception_count, parse_errors, function_code_distribution, dropped_findings |
| BC-2.14.022 | MAX_FINDINGS Cap and Poison-Skip Behavior for ModbusAnalyzer | dropped_findings counter ties to BC-2.14.021 sixth key |

### Group H — Dispatcher and CLI Integration (3 BCs)

| BC-ID | H1 Title (Authoritative) | Notes |
|-------|--------------------------|-------|
| BC-2.14.023 | --modbus CLI Flag Enables ModbusAnalyzer; --all Includes Modbus; Default-Off; Requires Stream Reassembly | |
| BC-2.14.024 | --modbus-write-threshold Configures Per-Flow Write-Burst Rate Threshold Consumed by Burst Detector | Single 1s-window threshold; default 10 (Decision 5) |
| BC-2.14.025 | StreamDispatcher Classifies Port-502 Flows to DispatchTarget::Modbus as Rule 5 (After Content and TLS/HTTP Port Rules) | VP-004 extension |

**Final SS-14 BC count: 25** (BC-2.14.001 through BC-2.14.025, all fully written).
ARCH-INDEX SS-14 row BC Count updated from 0 to 25.

---

## 12. T0831 Manipulation of Control — Implementation Specification

T0831 (Manipulation of Control) detects coordinated write sequences that drive the process
outside safe bounds. BC-2.14.016 (authoritative) defines the canonical v1 trigger:

**Canonical FC subset for T0831: {0x06, 0x10, 0x16}** — holding-register write FCs only.
(Previously §12 said {0x10} only — that was a simplification error. BC-2.14.016 body is
authoritative and specifies all three holding-register write FCs.)

**Trigger:** two or more write FCs in the canonical subset within the same flow within a
5-second pcap-timestamp window. This is a looser heuristic (does not require different
registers or setpoint+alarm correlation), accepted as the v1 implementation per BC-2.14.016.

**Constants:**
- `T0831_WINDOW_SECS: u32 = 5` — fixed; not CLI-configurable in v1.
- FC subset: `{0x06, 0x10, 0x16}` — same as T0836 (holding registers), NOT coil writes.

**State fields** (on `ModbusFlowState` — see §2.3 for complete list):
- `t0831_window_start_ts: u32` — start of current 5s window.
- `t0831_window_write_count: u32` — holding-register write count in window.
- `t0831_burst_emitted: bool` — emission guard; reset on window expiry.

**Emission policy:** T0831 fires ONCE per window overflow (not once per write). Subsequent
holding-register writes within the same window do not generate additional T0831 findings.
T0831 co-occurs with T0836 (and optionally T0855) per the per-PDU emission policy (§2.6).

BC-2.14.016 is fully written. No deferral to v1.1 is required.

---

## Appendix: Key Constant and Type Names

For implementers cross-referencing F3 stories to this delta:

| Name | Kind | Location | Value |
|------|------|----------|-------|
| `MAX_PENDING_TRANSACTIONS` | `const usize` | `src/analyzer/modbus.rs` | 256 |
| `MAX_FINDINGS` | `const usize` | `src/analyzer/modbus.rs` (local import or re-use from reassembly) | 10_000 |
| `WRITE_RATE_WINDOW_SECS` | `const u32` | `src/analyzer/modbus.rs` | 1 |
| `DEFAULT_MODBUS_WRITE_THRESHOLD` | `const u32` | `src/analyzer/modbus.rs` or `src/cli.rs` | 10 |
| `MbapHeader` | `struct` | `src/analyzer/modbus.rs` | fields: transaction_id, protocol_id, length, unit_id, function_code |
| `FunctionCodeClass` | `enum` | `src/analyzer/modbus.rs` | Read, Write, Diagnostic, Exception, Unknown |
| `ModbusAnalyzer` | `struct` (pub) | `src/analyzer/modbus.rs` | flows, write_threshold, fn_code_counts, total_write_count, total_exception_count, total_pdu_count, all_findings, parse_errors, dropped_findings, total_flows_analyzed, **duplicate_inflight_txn** (internal diagnostic — NOT a summarize() key; see BC-2.14.009 Invariant 6) |
| `ModbusFlowState` | `struct` (private) | `src/analyzer/modbus.rs` | pending, write_count, exception_count, pdu_count, last_ts, window_write_count, window_start_ts, window_burst_emitted, t0831_window_start_ts, t0831_window_write_count, t0831_burst_emitted, exception_window_counts, exception_window_start_ts, exception_burst_emitted, is_non_modbus |
| `MitreMatrix` | `enum` | `src/mitre.rs` | Enterprise, Ics |
| `technique_matrix` | `pub fn` | `src/mitre.rs` | id -> Option<MitreMatrix> |
| `SEEDED_TECHNIQUE_ID_COUNT` | `const usize` | `src/mitre.rs` | 15 → 20 (11 Enterprise + 9 ICS) |
| `SEEDED_TECHNIQUE_IDS` | `const &[&str]` | `src/mitre.rs` | 15 → 20 entries |
| `EMITTED_IDS` | `const &[&str]` (kani_proofs) | `src/mitre.rs` | 6 → 13 entries |
| `DispatchTarget::Modbus` | enum variant | `src/dispatcher.rs` | new fourth variant |
