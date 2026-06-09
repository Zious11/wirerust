---
document_type: behavioral-contract
level: L3
version: "1.0"
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
modified: []
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

# BC-2.14.017: Write-Rate Burst Exceeding --modbus-write-threshold Emits T0806 Brute Force I/O and T0855 Findings

## Description

When the per-flow write-FC rate exceeds the configured threshold within a 1-second sliding
window, `ModbusAnalyzer` emits a T0806 ("Brute Force I/O") finding AND a companion T0855
("Unauthorized Command Message") finding to represent the burst event. The threshold is set
via `--modbus-write-threshold` (default 10 write FCs per 1-second window). The window model
uses `ModbusFlowState.window_write_count` and `window_start_ts` (1-second resolution in pcap
microsecond timestamps). A `window_burst_emitted` flag ensures the T0806 finding fires once
per window overflow rather than on every subsequent write in the same window.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `classify_fc(function_code)` returns `FunctionCodeClass::Write`.
4. The write-rate window update logic (see Invariants) has determined that
   `window_write_count > write_threshold` (after incrementing the counter for this write).
5. `flow.window_burst_emitted == false` (suppress re-emission within the same window).
6. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

1. A T0806 `Finding` is pushed with:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Malicious`
   - `confidence: Confidence::High` — burst patterns are high-confidence Brute Force I/O.
   - `summary`: `"Modbus write burst: {count} writes in {elapsed_ms}ms window (unit {unit_id}, threshold {threshold}/s)"`
     where `{count}` is `flow.window_write_count`, `{elapsed_ms}` is
     `(now_ts - flow.window_start_ts) / 1000`, `{unit_id}` is the MBAP Unit ID,
     and `{threshold}` is `self.write_threshold`.
   - `evidence`: one entry — `"window_write_count={count} window_start_ts={start_ts} threshold={threshold} FC=0x{fc:02X} UnitID={unit_id}"`.
   - `mitre_technique: Some("T0806".to_string())`
   - `source_ip: Some(flow_key.client_ip())`
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. A companion T0855 `Finding` is pushed immediately after the T0806 finding with:
   - Same `category`, `verdict`, `source_ip`, `timestamp`, `direction` as T0806.
   - `confidence: Confidence::High`
   - `summary`: `"Modbus unauthorized write burst: FC 0x{fc:02X} x{count} in window (unit {unit_id})"`
   - `evidence`: same evidence string as T0806 finding.
   - `mitre_technique: Some("T0855".to_string())`
3. `flow.window_burst_emitted = true`.
4. The per-PDU T0855 finding (BC-2.14.013) is ALSO emitted for this write PDU. The total
   number of findings emitted for a burst-triggering PDU is up to 4: per-PDU T0855 (BC-2.14.013)
   + any of T0836/T0835/T0831 (BCs 014/015/016) + T0806 (this BC postcondition 1)
   + burst T0855 (this BC postcondition 2). All are independent findings.
5. `flow.write_count`, `self.total_write_count`, and `fn_code_counts` are incremented normally.
   (Counter increments are not conditional on the findings cap — see BC-2.14.022.)

## Invariants

1. **Write-rate window model** (authoritative definition for `window_write_count`,
   `window_start_ts`, `window_burst_emitted` in `ModbusFlowState`):

   On every write-class FC in `ClientToServer` direction:
   ```
   if now_ts - window_start_ts > WRITE_RATE_WINDOW_SECS * 1_000_000:
       // Window expired: reset
       window_write_count = 1
       window_start_ts = now_ts
       window_burst_emitted = false
   else:
       // Still in window: increment
       window_write_count += 1

   if window_write_count > write_threshold AND NOT window_burst_emitted:
       emit T0806 finding
       emit burst T0855 finding
       window_burst_emitted = true
   ```
   Where `WRITE_RATE_WINDOW_SECS = 1` (constant, not configurable).

2. **Default threshold semantics**: `DEFAULT_MODBUS_WRITE_THRESHOLD = 10`.
   Semantics: more than 10 write FCs within any contiguous 1-second window to the same flow
   triggers the burst (Decision 5 — single configurable threshold over a 1s window). This is
   the sole criterion; there is no secondary ">20 writes" threshold. Callers may adjust the
   threshold via `--modbus-write-threshold`; the default of 10 is based on research §5.1.

3. **Per-flow isolation**: each flow's `ModbusFlowState` has independent `window_write_count`,
   `window_start_ts`, and `window_burst_emitted` fields. A burst on flow A does not affect
   flow B.

4. **Unit ID note**: the Unit ID in the MBAP header identifies the sub-device (slave address).
   In v1, the window is per-flow (TCP 5-tuple), NOT per-{flow+Unit-ID}. A burst of writes
   to different Unit IDs on the same TCP connection is still counted together. This is a
   known limitation noted in research §5.1: the approved scope says "to the same target" but
   wirerust cannot distinguish sub-devices without deeper state; v1 applies the threshold
   at the flow level.

5. **T0806 is a BURST-LEVEL finding** (once per window overflow); T0855 via BC-2.14.013 is
   a **PDU-LEVEL finding** (once per write PDU). They are independent detection paths that
   happen to both fire on the PDU that tips the threshold. The burst-T0855 finding from this
   BC (postcondition 2) is a SECOND T0855 finding for that PDU and is distinct from the
   per-PDU T0855 — it carries burst-specific summary and evidence text.

6. **`--modbus-write-threshold 0`** is not validated to be > 0 in v1 (no minimum enforced
   by this BC). If 0 is passed, every write FC triggers a burst finding. This is an
   operational misconfiguration concern, not a safety defect.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `write_threshold = 10`; exactly 11 write FCs in < 1 second | 11th write tips the threshold: T0806 + burst T0855 emitted. Writes 1–10: only per-PDU T0855 (and T0836/T0835 as applicable). |
| EC-002 | 12th write FC arrives in the same 1-second window | T0806 NOT re-emitted (`window_burst_emitted = true`). Per-PDU T0855 from BC-2.14.013 still emitted. |
| EC-003 | Window expires; 11th write in new window | Window resets (`burst_emitted = false`, `count = 1`). No T0806 yet (count = 1 in new window, threshold not exceeded). |
| EC-004 | New window after burst: first 11 writes re-exceed threshold | T0806 fires again (new window, `burst_emitted` was reset to false on window rollover). |
| EC-005 | `write_threshold = 1`; second write FC in window | Second write: `count = 2 > 1`; T0806 + burst T0855 emitted. |
| EC-006 | All write FCs are from different Unit IDs in the same TCP connection | Window counter accumulates across all Unit IDs. Burst fires after `write_threshold + 1` writes regardless of Unit ID diversity. |
| EC-007 | `all_findings.len() == MAX_FINDINGS - 1` when burst fires | Per-PDU T0855 (from BC-2.14.013, pushed first) fills last slot. T0806 and burst T0855 NOT pushed. `window_burst_emitted` still set to true (prevents future failed attempts). |
| EC-008 | Read FC (0x03) in high volume | Read FCs do NOT increment `window_write_count`. No T0806. The rate gate is write-class-only. |
| EC-009 | now_ts < window_start_ts (pcap timestamp wrap-around or out-of-order packets) | `now_ts - window_start_ts` wraps as u32 arithmetic to a large value (≫ 1_000_000). The window-expiry check fires (treating as "window expired"), resetting `window_write_count = 1`, `window_start_ts = now_ts`, `window_burst_emitted = false`. This is the evasion-resistant policy: a wrapped huge value resets the window, meaning an attacker who injects a low-timestamp packet at most forces a window reset (requiring threshold+1 more writes for a new burst detection) — they cannot permanently suppress T0806 detection. No crash or panic. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `write_threshold=10`; 10 write PDUs (FC=0x06) at ts=0,1,2,...,9 (μs) — same flow | No T0806 after 10 writes; `window_write_count=10`. Per-PDU T0855 for each. | edge-case (at threshold, not over) |
| Same as above + 11th write at ts=10 | T0806 Finding + burst T0855 Finding pushed after 11th write; `window_burst_emitted=true`; total T0855 findings = 12 (11 per-PDU + 1 burst) | happy-path (threshold crossed) |
| `write_threshold=10`; burst of 25 writes at ts=0,1,...,24 (all within 1s window) | T0806 fires on 11th write; writes 12–25: no additional T0806 (burst_emitted=true). 25 per-PDU T0855s + 1 T0806 + 1 burst T0855. | happy-path (sustained burst) |
| `write_threshold=10`; 5 reads (FC=0x03) then 11 writes | Reads: no window increment; writes 1–10: below threshold; write 11: T0806 + burst T0855 | edge-case (reads don't count) |
| Window expires between write 10 and write 11 (ts gap > 1,000,000 μs) | Window resets. Write 11 starts new window (count=1). No T0806 | edge-case (window expiry) |
| `write_threshold=3`; hex sequence ADU-A(FC=0x06 ts=0), ADU-B(FC=0x10 ts=100), ADU-C(FC=0x05 ts=200), ADU-D(FC=0x10 ts=300) | ADU-A: count=1; ADU-B: count=2; ADU-C: count=3 (0x05 is write-class); ADU-D: count=4 > 3 → T0806 + burst T0855 | happy-path (mixed write FCs) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc Write-class exhaustiveness | Kani (sub-property B) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC implements the write-burst rate detection path of the ICS analysis capability, which is the signal for Brute Force I/O attacks against I/O points |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; ModbusFlowState window_write_count/window_start_ts/window_burst_emitted) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Techniques | T0806 — Brute Force I/O (ATT&CK for ICS; IcsImpairProcessControl); T0855 — Unauthorized Command Message (co-emitted for burst event) |

## Related BCs

- BC-2.14.013 — composes with (per-PDU T0855 also emitted for the same PDU; independent)
- BC-2.14.016 — related to (T0831 5-second window runs independently; separate state fields)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusFlowState` write-rate window fields
- `src/analyzer/modbus.rs` — `ModbusAnalyzer.write_threshold` field; `DEFAULT_MODBUS_WRITE_THRESHOLD`
- `src/analyzer/modbus.rs` — burst-detection branch in `on_data` write-class path
- `src/mitre.rs` — `technique_info("T0806")` arm (new per ADR-005 §4.2)
- `src/cli.rs` — `--modbus-write-threshold` flag with `default_value_t = 10`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Write-class sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.3 (window model; WRITE_RATE_WINDOW_SECS; window_burst_emitted semantics); architecture-delta.md §2.6 (T0806 detection trigger); modbus-tcp-research.md §5.1 (rate threshold [JUDGMENT]: >10/s sustained or >20/s burst); ADR-005 §5.1 |
| **Confidence** | high (threshold value is [JUDGMENT] but the mechanism is well-specified) |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes (same PDU sequence + timestamps always produces same output) |
| **Overall classification** | effectful shell |
