---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified:
  - version: "1.1"
    date: 2026-06-09
    change: "ADR-006 migration: mitre_technique: None → mitre_techniques: vec![] (empty vec = no technique) in postconditions, invariants, and canonical vectors. No behavioral change."
  - version: "1.2"
    date: 2026-06-09
    change: "F5 spec defect fix: (1) timestamp units corrected microseconds→seconds to match the pipeline's timestamp_secs delivery (BC-2.09.007); window math now uses elapsed_secs = now_ts.wrapping_sub(start_ts) with expiry check elapsed_secs > 10 (not > 10_000_000). (2) source_ip postcondition Path A changed from flow_key.server_ip() (non-existent accessor) to Direction-resolved server/responder endpoint: exception responses are always ServerToClient; resolve server endpoint from flow_key lower_ip/upper_ip + direction. wrapping_sub retained for u32 second timestamps."
  - version: "1.3"
    date: 2026-06-09
    change: "Holdout blemish-1 fix (Feature #7 v0.4.0): exception-burst recon for exception codes 0x01 (Illegal Function = FC scanning) and 0x02 (Illegal Data Address = register-map enumeration) now maps to T0888 Remote System Information Discovery, consistent with the established recon→T0888 mapping for FCs 0x11/0x2B (BC-2.14.020 Decision 12). Other exception codes and the Clear Counters 0x000A anti-forensic path retain mitre_techniques: vec![]. Postcondition Path A mitre_techniques updated for 0x01/0x02; research-note updated; canonical test vectors updated; Traceability MITRE field updated."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.019: Exception Response Anomaly — Burst of Exception Codes Emits Anomaly Finding for Recon/Scanning

## Description

Modbus exception responses carry forensically significant signals: a burst of 0x01 "Illegal
Function" exceptions across a flow indicates function-code scanning (the attacker is probing
what FCs the device supports); a burst of 0x02 "Illegal Data Address" exceptions indicates
address-map enumeration (the attacker is mapping which registers/coils exist). These patterns
are detected by tracking per-flow exception counts and emitting an `Anomaly` finding when the
exception count within a 10-second window strictly exceeds 5 (i.e., the 6th exception of the
same code triggers the finding). The threshold check uses strict greater-than (`> 5`):
`EXCEPTION_RATE_THRESHOLD = 5` is the maximum NON-triggering count; triggering requires MORE
THAN 5 (i.e., at least 6) exceptions of the same code within the window. Additionally, FC 0x08
sub-function 0x000A (Clear Counters) is handled here as an anti-forensic indicator (no single
T0xxx ICS technique; categorized as Anomaly). This BC covers the response-side anomaly path;
the per-PDU exception classification is defined in BC-2.14.007 (Group B, classify_fc returning
Exception for FC >= 0x80).

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. `classify_fc(function_code)` returns `FunctionCodeClass::Exception`, i.e.,
   `function_code >= 0x80`. This is always a response direction PDU (see Invariant 3).
3. The exception code byte is present: `adu.len() >= 9` (7 MBAP + 1 exception FC + 1 exception code).
4. `exception_code = data[offset + 8]` (the byte immediately after the exception FC byte).
5. (For rate-based trigger) The per-flow same-exception-code count within the current 10-second
   window STRICTLY EXCEEDS `EXCEPTION_RATE_THRESHOLD = 5` (i.e., `count > 5`, meaning the 6th
   or later exception of the same code in the window triggers the finding).
6. `self.all_findings.len() < MAX_FINDINGS`.

**Clear Counters path (separate trigger):**
4b. `function_code == 0x08 AND sub_func == 0x000A` (ClientToServer direction).
    This path uses the same BC rather than a separate one because the Anomaly finding
    structure is identical and the anti-forensic concern is closely related to exception recon.

## Postconditions

### Path A: Exception burst (FC >= 0x80, exception_code in {0x01, 0x02})

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Inconclusive`
   - `confidence: Confidence::Medium`
   - `summary` (exception code 0x01): `"Modbus recon: {n} Illegal Function exceptions in window (unit {unit_id}) — possible FC scanning"`
   - `summary` (exception code 0x02): `"Modbus recon: {n} Illegal Data Address exceptions in window (unit {unit_id}) — possible register map enumeration"`
   - `summary` (other exception codes): `"Modbus exception anomaly: {n} exceptions code 0x{ec:02X} in window (unit {unit_id})"`
     where `{n}` is the window count and `{unit_id}` is from the matching pending-table entry
     (response PDU is correlated to request via BC-2.14.011).
   - `evidence`: one entry — `"exception_fc=0x{fc:02X} exception_code=0x{ec:02X} window_count={n} original_fc=0x{orig_fc:02X}"` where
     `orig_fc` is the original request FC recovered via `fc & 0x7F` from the exception FC byte.
   - `mitre_techniques`:
     - exception code 0x01: `vec!["T0888"]` — FC scanning discovers which function codes the
       device supports; this is Remote System Information Discovery (T0888, TA0102 Discovery),
       consistent with the recon FC 0x11/0x2B mapping (BC-2.14.020, Decision 12).
     - exception code 0x02: `vec!["T0888"]` — register-map enumeration discovers the device's
       address layout; this is Remote System Information Discovery (T0888, TA0102 Discovery),
       consistent with the recon FC 0x11/0x2B mapping (BC-2.14.020, Decision 12).
     - other exception codes (0x03, 0x04, 0x05, 0x06, etc.): `vec![]` (empty vec — no single
       clean ICS ATT&CK ID; Anomaly category is sufficient for forensic flagging; per ADR-006
       an empty Vec<String> replaces the old None sentinel).
   - `source_ip: Some(<server/responder endpoint>)` — the source of exception responses.
     Exception responses are always `Direction::ServerToClient`; `FlowKey` has no
     `server_ip()` accessor. Resolve the server endpoint from the `direction` arg and
     `flow_key.lower_ip()` / `flow_key.upper_ip()`: for `ServerToClient` direction, the
     server/responder endpoint is identifiable from the `Direction` value combined with
     the flow key's lower/upper address pair.
   - `timestamp: Some(...)` — pcap-relative capture timestamp.
   - `direction: Some(Direction::ServerToClient)`
2. `flow.exception_window_count` reset and `flow.exception_burst_emitted` set to true for
   this exception code.
3. `flow.exception_count` and `self.total_exception_count` incremented by 1.

### Path B: Clear Counters (FC=0x08, sub-func=0x000A, ClientToServer)

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Anomaly`
   - `verdict: Verdict::Inconclusive`
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus anti-forensic: Clear Counters (0x08/0x000A) sent to unit {unit_id}"`
   - `evidence`: one entry — `"FC=0x08 SubFunc=0x000A TxnID={txn_id:#06X} UnitID={unit_id}"`.
   - `mitre_techniques: vec![]` (empty vec — Clear Counters 0x000A is an anti-forensic
     indicator with no single clean ICS ATT&CK technique ID; treat as Evasion indicator;
     per ADR-006 an empty Vec<String> replaces the old None sentinel).
   - `source_ip: Some(<client/initiator endpoint>)` — resolved from `Direction::ClientToServer`
     and `flow_key.lower_ip()` / `flow_key.upper_ip()`. `FlowKey` has no `client_ip()`
     accessor; the client/initiator endpoint is determined from the `direction` arg.
   - `timestamp: Some(...)` — pcap-relative timestamp.
   - `direction: Some(Direction::ClientToServer)`

## Invariants

1. **Exception window model**: per-flow, per-exception-code 10-second window.
   State fields in `ModbusFlowState`:
   - `exception_window_counts: HashMap<u8, u32>` — keyed by exception code byte.
   - `exception_window_start_ts: HashMap<u8, u32>` — window start per exception code.
   - `exception_burst_emitted: HashMap<u8, bool>` — once per window per code.
   
   On each exception response:
   ```
   // now_ts is in SECONDS (timestamp_secs per BC-2.09.007; the pipeline delivers seconds).
   // wrapping_sub used for u32 second timestamps; wrap at ~136 years — policy kept.
   elapsed_secs = now_ts.wrapping_sub(exception_window_start_ts[ec])
   if elapsed_secs > EXCEPTION_WINDOW_SECS:  // 10-second window
       exception_window_counts[ec] = 1
       exception_window_start_ts[ec] = now_ts
       exception_burst_emitted[ec] = false
   else:
       exception_window_counts[ec] += 1
   // Strict greater-than: fires on the 6th (or later) exception, NOT the 5th.
   if exception_window_counts[ec] > EXCEPTION_RATE_THRESHOLD AND NOT exception_burst_emitted[ec]:
       emit Anomaly finding
       exception_burst_emitted[ec] = true
   ```
   `EXCEPTION_RATE_THRESHOLD = 5` (fixed, not CLI-configurable in v1).
   Semantics: `> 5` means the finding triggers when the count transitions from 5 to 6 (i.e.,
   the **6th exception** of the same code within the window). Exactly 5 exceptions: no finding.
   6 or more: finding emitted on the count that crosses the threshold (once per window).
   `EXCEPTION_WINDOW_SECS = 10` (fixed constant, SECONDS — matches pipeline timestamp units).

2. **Exception-code attribution**: the original request FC is recovered via `exception_fc & 0x7F`.
   If the (transaction_id, unit_id) pair is in `flow.pending`, the stored request FC is used
   for correlation evidence (per BC-2.14.011). If not found, `orig_fc` is derived from the
   exception FC byte alone.

3. **Exception responses are always ServerToClient direction** when correctly classified by
   the TCP direction signal. FC >= 0x80 may occasionally appear in request direction if a
   device is misconfigured; in that case, no exception finding is emitted (ambiguous; the
   three-point validity gate does not reject these but the response-path logic is gated on
   `direction == Direction::ServerToClient`).

4. **`flow.exception_count` is incremented for every exception-class PDU**, regardless of
   whether the rate threshold fires. The aggregate exception count is reported in `summarize()`
   per BC-2.14.021.

5. **The exception burst finding is `Verdict::Inconclusive` (not `Likely`)** because an
   exception burst alone is circumstantial: it could be caused by a misconfigured HMI or a
   compatibility probe rather than an active attack. Operators review inconclusive findings.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 5 Illegal Function (0x01) exceptions in 10 seconds | Count=5; `5 > 5` is false; no finding yet. The 5th exception does NOT trigger. |
| EC-002 | 6th Illegal Function exception within the same 10-second window | Count=6; `6 > 5` is true; Anomaly finding emitted (the 6th exception is the trigger). |
| EC-003 | Mix of 0x01 and 0x02 exception codes (3 each) in 10 seconds | Neither code exceeds 5 independently. No finding. (Codes are tracked separately.) |
| EC-004 | 6 exception code 0x02 within 10 seconds | Count=6 > 5; Anomaly emitted for register-map enumeration. |
| EC-005 | Exception burst window resets; new 6 exceptions in second window | `burst_emitted` was reset to false on window rollover; new finding emitted. |
| EC-006 | FC 0x08 sub-func 0x000A (Clear Counters) | Anomaly finding (anti-forensic indicator). No T0814 — that requires sub-func 0x0001/0x0004. |
| EC-007 | FC >= 0x80 but ADU is only 8 bytes (missing exception code byte) | No finding; malformed exception response; `parse_errors++`. |
| EC-008 | All 6 exception-burst finding slots taken (`all_findings.len() == MAX_FINDINGS`) | No finding pushed; `exception_count` still incremented. |
| EC-009 | now_ts < exception_window_start_ts[ec] (second-timestamp out-of-order or wrap) | `now_ts.wrapping_sub(exception_window_start_ts[ec])` yields a very large u32 value (≫ 10 seconds). Window-expiry fires: resets count=1, window_start_ts=now_ts, burst_emitted=false. Evasion-resistant: attacker cannot permanently suppress detection by injecting low-timestamp exception responses — at worst they force a window reset, requiring 6 more exceptions to retrigger. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 6 PDUs with FC=0x83 (exception for FC 0x03), exception_code=0x01, all within 8 seconds, UnitID=1, ServerToClient direction | After 6th PDU: count=6 > 5; Anomaly Finding{category=Anomaly, verdict=Inconclusive, confidence=Medium, summary="Modbus recon: 6 Illegal Function exceptions in window (unit 1) — possible FC scanning", mitre_techniques=vec!["T0888"]}. The 5th PDU: count=5, no finding. | happy-path (FC scanning; 6th triggers) |
| 6 PDUs with FC=0x83, exception_code=0x02, within 5 seconds | After 6th PDU: count=6 > 5; Anomaly Finding with register-map enumeration summary; mitre_techniques=vec!["T0888"] | happy-path (address recon; 6th triggers) |
| ADU: `00 01 00 00 00 06 01 08 00 0A 00 00` (FC=0x08, SubFunc=0x000A Clear Counters, UnitID=1) — ClientToServer | Anomaly Finding{summary="Modbus anti-forensic: Clear Counters (0x08/0x000A) sent to unit 1", mitre_techniques=vec![]} | happy-path (Clear Counters) |
| 5 exception_code=0x01 responses then 1 more after 11 seconds (window expired) | 6th exception starts new window (count=1); no finding (threshold not met in new window) | edge-case (window expiry) |
| 4 exception_code=0x01 + 4 exception_code=0x02 (8 total exceptions, 4 each) | No finding (each code tracked independently; neither exceeds threshold=5) | edge-case (mixed codes) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc(fc) == Exception iff fc >= 0x80 | Kani (sub-property C) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC covers exception-response anomaly detection and anti-forensic indicator detection, both of which are forensic signals unique to ICS protocol analysis |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; ModbusFlowState exception_window_counts) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0888 (exception codes 0x01 FC-scanning and 0x02 address-map enumeration); vec![] for other exception codes and Clear Counters 0x000A. FC-scanning (0x01) and address-map enumeration (0x02) map to T0888 Remote System Information Discovery, consistent with the recon FC mapping (BC-2.14.020 Decision 12); other exception codes remain untagged Anomalies. |

## Related BCs

- BC-2.14.007 — depends on (classify_fc Exception class: FC >= 0x80)
- BC-2.14.011 — depends on (exception FC attributed to originating request via pending table)
- BC-2.14.018 — related to (FC 0x08 sub-func 0x0001/0x0004 → T0814; this BC handles 0x000A)
- BC-2.14.020 — related to (unusual FC anomaly detection; exception bursts are a related anomaly signal)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — exception-response path in `on_data`
- `src/analyzer/modbus.rs` — `ModbusFlowState` exception_window_counts / exception_burst_emitted
- `src/analyzer/modbus.rs` — `classify_fc` pure function (VP-022 sub-property C)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Exception-class sub-property C (fc >= 0x80 iff Exception)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.6 (exception responses → T0855 attributed; anomaly context); modbus-tcp-research.md §3 (exception code table: 0x01 Illegal Function = FC scanning, 0x02 Illegal Data Address = address-map recon); modbus-tcp-research.md §7 (open-item: Clear Counters has no clean ATT&CK ID) |
| **Confidence** | medium (exception rate threshold is [JUDGMENT]) |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes |
| **Overall classification** | effectful shell |
