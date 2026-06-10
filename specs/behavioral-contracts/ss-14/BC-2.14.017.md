---
document_type: behavioral-contract
level: L3
version: "2.3"
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
  - version: "2.0"
    date: 2026-06-09
    change: "UPDATED (v2.0 — Decision 11, f2-fix-directives.md §11): Dual-window detection replaces single-window. BURST detector: >N writes in any 1s window (unchanged mechanism, threshold renamed to write_burst_threshold, default 20). SUSTAINED detector: NEW — >M avg writes/sec averaged over >=2s window (write_sustained_threshold, default 10). Both detectors fire at most once per their respective window. Both emit mitre_techniques: [\"T0806\",\"T0855\"] per Decision 13. Added sustained_window_start_ts / sustained_window_write_count / sustained_burst_emitted fields to ModbusFlowState. Detection math: sustained_window_write_count > write_sustained_threshold * elapsed_secs AND elapsed_secs>=2 AND NOT sustained_burst_emitted. Low-and-slow test vector added: 8 writes/s for 30s FIRES sustained. Targets v0.3.0."
  - version: "2.1"
    date: 2026-06-09
    change: "Adversarial review fix (Gemini cross-model review): replace integer-division sustained math with truncation-free microsecond-scale formula: (count as u64)*1_000_000 > (threshold as u64)*(elapsed_us as u64) AND elapsed_us >= 2_000_000 AND NOT sustained_burst_emitted. Use wrapping_sub for both burst and sustained window elapsed computation to handle u32 pcap-timestamp rollover. EC-004/EC-004b updated; EC-010 updated with wrapping_sub semantics; canonical vectors updated to use elapsed_us arithmetic."
  - version: "2.2"
    date: 2026-06-09
    change: "F5 spec defect fix: timestamp units corrected microseconds→seconds to match the pipeline's timestamp_secs delivery (BC-2.09.007; StreamHandler::on_data passes seconds, not microseconds). The f2 microsecond-scale assumption (*1_000_000, elapsed_us, 2_000_000) was wrong. Both burst and sustained window math now uses elapsed_secs = now_ts.wrapping_sub(window_start_ts) where now_ts is in SECONDS. Burst expiry: elapsed_secs > WRITE_BURST_WINDOW_SECS (1). Sustained minimum: elapsed_secs >= WRITE_SUSTAINED_WINDOW_SECS (2); rate check: count > threshold * elapsed_secs (seconds form, no *1_000_000). wrapping_sub retained (u32 second timestamps wrap at ~136 years — never in practice, policy kept for correctness). source_ip fields changed from flow_key.client_ip() (non-existent accessor) to Direction-resolved endpoint: ClientToServer direction → client/initiator endpoint. Canonical test vectors updated to second-scale timestamps. Note: sub-second rate precision is a future enhancement."
  - version: "2.3"
    date: 2026-06-09
    change: "F7 consistency finding F5: burst postcondition 1 summary string corrected to match code emission. Previous spec said '{count} writes in {elapsed_ms}ms window' with {elapsed_ms} = (now_ts - window_start_ts)/1000 — the /1000 formula is dead-letter (now_ts is already in seconds, not microseconds; /1000 would produce sub-second fractions). The code correctly emits '{elapsed_secs}s window'. Fixed: renamed elapsed_ms→elapsed_secs, removed /1000 formula, changed 'ms' unit to 's'. The burst finding summary is now '{count} writes in {elapsed_secs}s window (unit {unit_id}, threshold {threshold}/s)'."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
  - .factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md
input-hash: TBD
---

# BC-2.14.017: Write-Rate Exceeding Either Burst or Sustained Threshold Emits T0806 + T0855 Finding

<!-- Previous version (v1.0): "Write-Rate Burst Exceeding --modbus-write-threshold Emits T0806 Brute Force I/O and T0855 Findings"
     v1.0 model: single 1-second window, --modbus-write-threshold (default 10), write_threshold: u32.
       A single T0806 finding + a separate T0855 finding emitted as two objects per burst event.
     v2.0 model (Decision 11 + Decision 13):
       TWO independent detectors — burst (>N writes in 1s) and sustained (>M avg over >=2s).
       Each emits ONE finding with mitre_techniques: ["T0806","T0855"] (co-tagged, one finding per event).
       Burst threshold: write_burst_threshold (default 20). Sustained threshold: write_sustained_threshold (default 10).
       Prior --modbus-write-threshold flag is REMOVED; see BC-2.14.024 v2.0.
       Targets v0.3.0.
-->

## Description

`ModbusAnalyzer` implements two independent write-rate detectors per flow, each emitting a
single Finding with `mitre_techniques: vec!["T0806", "T0855"]` when triggered:

1. **Burst detector** (1-second window): fires when more than `write_burst_threshold` write-class
   FCs are observed within any single 1-second window on the flow. Default threshold: 20 writes/s.
   Guards the fast-attack case (e.g., flooding a PLC with register writes to force a fail-safe).

2. **Sustained detector** (≥2-second rolling window): fires when the average write rate exceeds
   `write_sustained_threshold` writes/second sustained over ≥2 consecutive seconds. Default
   threshold: 10 writes/s averaged. Guards the low-and-slow case (e.g., 8–12 writes/s maintained
   for 30+ seconds — missed by the burst detector but anomalous relative to legitimate SCADA
   baseline of 0.1–5 writes/s).

Each detector fires at most once per its respective window. The burst detector uses
`window_burst_emitted`; the sustained detector uses `sustained_burst_emitted`. Both flags are
reset on window expiry.

Per Decision 13 (ADR-006), each burst event emits ONE Finding with `mitre_techniques:
vec!["T0806", "T0855"]` (co-tagged, not two separate findings as in v1.0). The burst
finding is emitted alongside (not instead of) the per-PDU write finding from BC-2.14.013.

Targets v0.3.0.

## Preconditions

### Burst detector preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. TCP direction is `Direction::ClientToServer`.
3. `classify_fc(function_code)` returns `FunctionCodeClass::Write`.
4. The burst window update has determined `window_write_count > write_burst_threshold`
   (after incrementing the counter for this write).
5. `flow.window_burst_emitted == false`.
6. `self.all_findings.len() < MAX_FINDINGS`.

### Sustained detector preconditions

1–3 as above.
4. `elapsed_secs = now_ts.wrapping_sub(sustained_window_start_ts) >= WRITE_SUSTAINED_WINDOW_SECS` (2)
   AND `sustained_window_write_count > write_sustained_threshold * elapsed_secs`
   where `now_ts` is in SECONDS per BC-2.09.007 (see Invariant 2 for full rationale).
5. `flow.sustained_burst_emitted == false`.
6. `self.all_findings.len() < MAX_FINDINGS`.

## Postconditions

### Burst finding postcondition

1. ONE `Finding` is pushed:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High` — burst patterns are high-confidence Brute Force I/O.
   - `summary`: `"Modbus write burst: {count} writes in {elapsed_secs}s window (unit {unit_id}, threshold {threshold}/s)"`
     where `{count}` is `flow.window_write_count`, `{elapsed_secs}` is
     `now_ts.wrapping_sub(flow.window_start_ts)` (seconds, per BC-2.09.007 — no /1000),
     `{unit_id}` is the MBAP Unit ID, and `{threshold}` is `self.write_burst_threshold`.
   - `evidence`: one entry — `"Burst threshold exceeded: {count} write FCs in 1s window; window_write_count={count} window_start_ts={start_ts} threshold={threshold} FC=0x{fc:02X} UnitID={unit_id}"`.
   - `mitre_techniques: vec!["T0806", "T0855"]`
   - `source_ip: Some(<client/initiator endpoint>)` — resolved from the TCP `direction` arg
     passed to `on_data`: `Direction::ClientToServer` means the write-class PDU originates from
     the client (initiator) side. `FlowKey` has no `client_ip()` accessor; resolve via
     `flow_key.lower_ip()` / `flow_key.upper_ip()` combined with the `direction` field.
     For write-rate findings the direction is always `ClientToServer`, so `source_ip` is
     the client/initiator endpoint. Concrete resolution: consult `Direction` to determine
     which of `lower_ip` / `upper_ip` is the initiator for this flow.
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. `flow.window_burst_emitted = true`.
3. `flow.write_count`, `self.total_write_count`, and `fn_code_counts` are incremented normally.

### Sustained finding postcondition

1. ONE `Finding` is pushed:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Likely`
   - `confidence: Confidence::High`
   - `summary`: `"Modbus write burst: {count} writes over {elapsed_s}s window (unit {unit_id}, >{threshold}/s avg)"`
     where `{count}` is `flow.sustained_window_write_count`, `{elapsed_s}` is `elapsed_secs`,
     and `{threshold}` is `self.write_sustained_threshold`.
   - `evidence`: one entry — `"Sustained write rate exceeded: {count} writes over {elapsed_s} seconds (>{threshold}/s average); sustained_window_start_ts={start_ts} FC=0x{fc:02X} UnitID={unit_id}"`.
   - `mitre_techniques: vec!["T0806", "T0855"]`
   - `source_ip: Some(<client/initiator endpoint>)` — resolved from the TCP `direction` arg
     as described in the burst finding postcondition above (same Direction::ClientToServer logic;
     write-rate detectors always fire on ClientToServer PDUs).
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. `flow.sustained_burst_emitted = true`.
3. Counters incremented as above.

## Invariants

### 1. Burst window model (1-second window)

`window_write_count`, `window_start_ts`, `window_burst_emitted` in `ModbusFlowState`:

```
On every write-class FC in ClientToServer direction:

// now_ts is in SECONDS (timestamp_secs per BC-2.09.007; the pipeline delivers seconds).
// wrapping_sub used for u32 seconds; wrap at ~136 years — effectively never, policy kept.
elapsed_secs = now_ts.wrapping_sub(window_start_ts)

if elapsed_secs > WRITE_BURST_WINDOW_SECS:
    // Window expired: reset
    window_write_count = 1
    window_start_ts = now_ts
    window_burst_emitted = false
else:
    window_write_count += 1

if window_write_count > write_burst_threshold AND NOT window_burst_emitted:
    emit ONE burst Finding { mitre_techniques: vec!["T0806","T0855"], evidence="Burst threshold exceeded..." }
    window_burst_emitted = true
```

`WRITE_BURST_WINDOW_SECS = 1` (constant, seconds). `DEFAULT_WRITE_BURST_THRESHOLD = 20`.

### 2. Sustained window model (≥2-second rolling window)

`sustained_window_start_ts`, `sustained_window_write_count`, `sustained_burst_emitted` in `ModbusFlowState`:

```
On every write-class FC in ClientToServer direction (AFTER burst update):

// now_ts is in SECONDS (timestamp_secs per BC-2.09.007; the pipeline delivers seconds).
// wrapping_sub used for u32 seconds; wrap at ~136 years — effectively never, policy kept.

if sustained_window_start_ts == 0:
    // Initial state: start the window
    sustained_window_start_ts = now_ts
    sustained_window_write_count = 1
else:
    sustained_window_write_count += 1
    elapsed_secs = now_ts.wrapping_sub(sustained_window_start_ts)

    if elapsed_secs >= WRITE_SUSTAINED_WINDOW_SECS:
        // Detection trigger — seconds-scale integer math:
        trigger := sustained_window_write_count > write_sustained_threshold * elapsed_secs
                   AND NOT sustained_burst_emitted

        if trigger:
            emit ONE sustained Finding { mitre_techniques: vec!["T0806","T0855"], evidence="Sustained write rate exceeded..." }
            sustained_burst_emitted = true

        // Window reset (non-overlapping; always reset after >=2s elapses):
        sustained_window_start_ts = now_ts
        sustained_window_write_count = 1
        sustained_burst_emitted = false
```

`WRITE_SUSTAINED_WINDOW_SECS = 2` (minimum window duration, seconds). `DEFAULT_WRITE_SUSTAINED_THRESHOLD = 10`.

**Detection math (seconds-scale, integer-only):**
```
elapsed_secs := now_ts.wrapping_sub(sustained_window_start_ts)   // u32 wrapping sub (seconds)

trigger := elapsed_secs >= WRITE_SUSTAINED_WINDOW_SECS
         AND sustained_window_write_count > write_sustained_threshold * elapsed_secs
         AND NOT sustained_burst_emitted
```
Note: The v2.1 microsecond-scale formula (`count * 1_000_000 > threshold * elapsed_us`) is
SUPERSEDED by this seconds form (F5 correction). The pipeline's `on_data` delivers
`timestamp_secs` (seconds, per BC-2.09.007); the microsecond assumption was wrong.
`wrapping_sub` handles u32 second-timestamp rollover (rolls over at ~136 years of capture time;
effectively never in practice, but the policy is retained for correctness).
At defaults: fires when count > 10 writes/s average over the elapsed window.
At exactly 2s: fires if count > 10*2 = 20 writes (i.e., ≥ 21 writes in 2 seconds).
At 30s: fires if count > 10*30 = 300 writes.
Note: sub-second rate precision is a future enhancement that would require threading
`timestamp_usecs` through `on_data` — not in v1 scope.

### 3. Burst vs sustained finding distinction

The evidence string distinguishes the two emission paths:
- Burst: `"Burst threshold exceeded: N write FCs in 1s window"`
- Sustained: `"Sustained write rate exceeded: N writes over E seconds (>T/s average)"`

Both carry `mitre_techniques: vec!["T0806","T0855"]` per Decision 13.

### 4. Per-flow isolation

Each flow's `ModbusFlowState` has independent window fields. A burst or sustained event on
flow A does not affect flow B.

### 5. Constants (v2, all four)

```rust
const WRITE_BURST_WINDOW_SECS: u32 = 1;           // fixed 1-second burst window (seconds)
const DEFAULT_WRITE_BURST_THRESHOLD: u32 = 20;     // --modbus-write-burst-threshold default
const WRITE_SUSTAINED_WINDOW_SECS: u32 = 2;        // minimum sustained window duration (seconds)
const DEFAULT_WRITE_SUSTAINED_THRESHOLD: u32 = 10; // --modbus-write-sustained-threshold default
```

These constants are in SECONDS (the pipeline delivers `timestamp_secs` per BC-2.09.007;
the v2.1 microsecond interpretation was wrong and is corrected in v2.2).
The prior `WRITE_RATE_WINDOW_SECS = 1` and `DEFAULT_MODBUS_WRITE_THRESHOLD = 10` are REMOVED.

### 6. Finding count per PDU (v2 vs v1 comparison)

| Scenario | v2.0 findings | v1.0 findings |
|----------|---------------|---------------|
| Register write (mid-burst, no T0831, no threshold tip) | 1 (per-PDU: `["T0855","T0836"]`) | 2 (T0855 + T0836 separate) |
| Register write tipping burst threshold | 2 (per-PDU: `["T0855","T0836"]` + burst: `["T0806","T0855"]`) | 4 (T0855+T0836+T0806+burst-T0855) |
| Register write tipping sustained threshold | 2 (per-PDU + sustained: `["T0806","T0855"]`) | N/A (v1.0 had no sustained detector) |
| Register write tipping both burst AND sustained | 3 (per-PDU + burst + sustained) | N/A |

V2 is strictly fewer findings per PDU than v1 while preserving all technique attribution.

### 7. `mitre_techniques` field per Decision 13 (ADR-006)

Both the burst and sustained findings use `mitre_techniques: Vec<String>` (not
`mitre_technique: Option<String>`). The field is `vec!["T0806", "T0855"]` for both.
JSON output: `"mitre_techniques": ["T0806","T0855"]`. CSV column 6: `"T0806;T0855"`.

### 8. Zero threshold rejection (see BC-2.14.024)

If either threshold is 0, the CLI rejects the value before `ModbusAnalyzer::new` is called
(per BC-2.14.024 v2.0 postconditions P3a/P3b). The invariant `write_burst_threshold >= 1`
and `write_sustained_threshold >= 1` holds at the analyzer struct level.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `write_burst_threshold=20`; exactly 21 write FCs in < 1 second | 21st write tips the burst threshold: ONE burst Finding `["T0806","T0855"]` emitted alongside per-PDU finding. Writes 1–20: only per-PDU findings. |
| EC-002 | 22nd write FC in same 1-second window | Burst NOT re-emitted (`window_burst_emitted = true`). Per-PDU finding still emitted. |
| EC-003 | Burst window expires; 21st write in new window | Window resets. No burst finding yet (count = 1). |
| EC-004 | `write_sustained_threshold=10`; 8 writes in 2s (elapsed_secs=2) | 8 > 10*2=20? No. Does NOT fire at default threshold=10. To fire with 8 writes/s avg, use `--modbus-write-sustained-threshold 3` (8 > 3*2=6 → fires). |
| EC-004b | `write_sustained_threshold=10`; elapsed_secs = 1 (1 second elapsed), count = 21 | elapsed_secs=1 < WRITE_SUSTAINED_WINDOW_SECS=2 → NOT yet at window minimum; check skipped. No detection. (With seconds-scale timestamps, sub-second precision is not available; the window minimum check uses whole seconds.) |
| EC-005 | `write_sustained_threshold=10`; 22 writes in 2s (elapsed_secs=2; 11 writes/s avg) | elapsed_secs=2 >= 2; check: 22 > 10*2=20 → FIRES. Sustained finding emitted. Window resets. |
| EC-006 | `write_sustained_threshold=10`; 20 writes in 2s (exactly at threshold; elapsed_secs=2) | elapsed_secs=2 >= 2; check: 20 > 10*2=20? No (strict `>`). Does NOT fire. |
| EC-007 | Both burst and sustained fire on the same PDU | Per-PDU finding + burst Finding + sustained Finding (3 findings total). Each fires at most once per their respective window overlap. |
| EC-008 | `all_findings.len() == MAX_FINDINGS - 1` when burst fires | Per-PDU finding fills last slot (pushed first). Burst finding NOT pushed. `window_burst_emitted` still set to true. |
| EC-009 | Read FC (0x03) in high volume | Read FCs do NOT increment `window_write_count` or `sustained_window_write_count`. No T0806. Rate gates are write-class-only. |
| EC-010 | now_ts < window_start_ts (u32 second-timestamp out-of-order or wrap) | `now_ts.wrapping_sub(window_start_ts)` gives a large u32 value (≫ any window threshold). Both burst and sustained detectors treat this as window-expired: reset. Rollover at ~136 years — effectively never in practice. Correct and evasion-resistant. |

## Canonical Test Vectors

All timestamps are in SECONDS (timestamp_secs per BC-2.09.007; the pipeline delivers seconds, not microseconds).

| Input | Expected Output | Category |
|-------|----------------|----------|
| `write_burst_threshold=20`; 20 write PDUs (FC=0x06) at ts=1000s..1000s (same second) — same flow | No burst finding after 20 writes; `window_write_count=20`. Per-PDU findings with `["T0855","T0836"]` each. | edge-case (at burst threshold, not over) |
| Same + 21st write at ts=1000s (still within 1-second burst window) | ONE burst Finding `{mitre_techniques=["T0806","T0855"], evidence contains "Burst threshold exceeded"}` emitted; `window_burst_emitted=true` | happy-path (burst threshold crossed) |
| `write_burst_threshold=20`; 25 writes within elapsed_secs=0 (same second) | Burst fires on 21st write; writes 22–25: no additional burst finding (`burst_emitted=true`). 25 per-PDU findings + 1 burst finding. | happy-path (burst caps at once) |
| `write_sustained_threshold=7`; 16 writes accumulated; elapsed_secs=2 (window boundary hit) | elapsed_secs=2 >= 2; check: 16 > 7*2=14 → FIRES. ONE sustained Finding `{mitre_techniques=["T0806","T0855"], evidence="Sustained write rate exceeded: 16 writes over 2 seconds (>7/s average)"}` | happy-path (low-and-slow sustained detection) |
| `write_sustained_threshold=10`; 22 writes accumulated at elapsed_secs=2 | Check: 22 > 10*2=20 → FIRES sustained finding. | happy-path (sustained at default threshold) |
| `write_sustained_threshold=10`; 20 writes at elapsed_secs=2 (exactly at threshold) | Check: 20 > 10*2=20? No (strict >). NOT fired. | edge-case (exactly at threshold; strict > required) |
| Window expires between write 20 and write 21 (ts gap > WRITE_BURST_WINDOW_SECS=1 second) for burst detector | Burst window resets. Write 21 starts new window (count=1). No burst finding. | edge-case (burst window expiry) |
| `write_burst_threshold=3`; ADU-A(FC=0x06 ts=1000s), ADU-B(FC=0x10 ts=1000s), ADU-C(FC=0x05 ts=1000s), ADU-D(FC=0x10 ts=1000s) | ADU-D: count=4 > 3 → burst fires: ONE Finding `["T0806","T0855"]` | happy-path (mixed write FCs, burst) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc Write-class exhaustiveness | Kani (sub-property B) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC implements the dual-window write-rate detection path of the ICS analysis capability, which covers both fast-burst (T0806 Brute Force I/O) and low-and-slow (sustained rate) attacks against I/O points |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; ModbusFlowState burst+sustained window fields; ModbusAnalyzer write_burst_threshold + write_sustained_threshold) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Techniques | T0806 — Brute Force I/O (ATT&CK for ICS; IcsImpairProcessControl); T0855 — Unauthorized Command Message (co-tagged on burst/sustained events) |

## Related BCs

- BC-2.14.013 — composes with (per-PDU T0855+T0836/T0835 finding also emitted for the same PDU; independent from burst)
- BC-2.14.016 — related to (T0831 5-second window runs independently; separate state fields)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)
- BC-2.14.024 — depends on (dual CLI flags configure write_burst_threshold and write_sustained_threshold)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusFlowState` burst window fields (`window_write_count`, `window_start_ts`, `window_burst_emitted`)
- `src/analyzer/modbus.rs` — `ModbusFlowState` sustained window fields (`sustained_window_start_ts`, `sustained_window_write_count`, `sustained_burst_emitted`)
- `src/analyzer/modbus.rs` — `ModbusAnalyzer.write_burst_threshold: u32` and `write_sustained_threshold: u32`; `DEFAULT_WRITE_BURST_THRESHOLD = 20`; `DEFAULT_WRITE_SUSTAINED_THRESHOLD = 10`
- `src/analyzer/modbus.rs` — burst-detection branch and sustained-detection branch in `on_data` write-class path
- `src/mitre.rs` — `technique_info("T0806")` arm (new per ADR-005 §4.2)
- `src/cli.rs` — `--modbus-write-burst-threshold` and `--modbus-write-sustained-threshold` flags (BC-2.14.024)
- `.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Write-class sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | f2-fix-directives.md §11 (Decision 11: dual-window model; §11.1 CLI flags; §11.4 sustained fields; §11.5 detection math; §11.6 finding distinction); ADR-006 (mitre_techniques: Vec<String>); architecture-delta.md §2.3 (sustained window fields; complete ModbusFlowState field list v2) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes (same PDU sequence + timestamps always produces same output) |
| **Overall classification** | effectful shell |
