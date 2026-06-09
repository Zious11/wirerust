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

# BC-2.14.016: Coordinated Write Sequence to Holding Registers Within 5-Second Window Emits T0831 Manipulation of Control Finding

## Description

When two or more write FCs targeting holding registers (FC 0x06, 0x10, or 0x16) are observed
within the same flow within a 5-second pcap-timestamp window, a T0831 ("Manipulation of
Control") finding is emitted. T0831 represents a coordinated attack that drives the process
outside safe operating bounds — e.g., raising a setpoint while simultaneously lowering an
alarm threshold, or writing to both a process variable register and its safety interlock
register within a short window. The five-second window is fixed in v1 (not CLI-configurable).
The T0831 finding is emitted once per window overflow (not once per write), using a per-flow
`t0831_window_start_ts: u32` and `t0831_window_write_count: u32` in `ModbusFlowState`. This
BC fires in addition to the per-write T0855 and T0836 findings (BCs 013/014), not instead.

## Discriminator Rule (T0836 vs T0835 vs T0831)

This invariant is authoritative for all three write-technique BCs:

| Technique | FC subset | Firing condition | Granularity |
|-----------|-----------|-----------------|-------------|
| T0836 (BC-2.14.014) | 0x06, 0x10, 0x16 | Per-write (every qualifying FC) | One finding per PDU |
| T0835 (BC-2.14.015) | {0x05, 0x0F} (coil-only) | Per-write (every qualifying FC) | One finding per PDU |
| T0831 (this BC) | 0x06, 0x10, 0x16 | Sequence detector: ≥2 writes to holding registers within a 5-second window in the same flow | One finding per window overflow (not per PDU) |

**Why T0831 is distinct from T0836:** T0836 detects a single parameter write (precision:
one altered setpoint). T0831 detects a coordinated multi-write sequence that together
moves the process out of its safety envelope. The sequence aspect — multiple holding-register
writes within a short window — is the distinguishing signal. T0831 always co-occurs with
one or more T0836 findings (since each write in the sequence also fires T0836), but T0836
can occur without T0831 (single writes).

**v1 scope note (architecture-delta.md §12):** The v1 implementation uses the simpler
heuristic: "any two or more Write FCs to holding registers within a 5-second window". It
does NOT require that different register addresses be targeted, nor that the combination
include both setpoint and alarm registers. This heuristic has a higher false-positive rate
than a full semantic cross-register correlator, which is deferred to a future feature cycle.

## Preconditions

1. The MBAP ADU has passed the three-point validity gate.
2. The TCP direction is `Direction::ClientToServer`.
3. `function_code` is one of: `0x06`, `0x10`, `0x16`.
4. The window-update logic runs FIRST on every holding-register write (regardless of emission):
   see Invariant 2 for the canonical evaluation order. The T0831 finding is emitted only when
   the post-update `t0831_window_write_count >= 2` AND `t0831_burst_emitted == false`.
5. `self.all_findings.len() < MAX_FINDINGS`.
6. `flow.t0831_burst_emitted == false` (emission is once per window overflow, not once per
   subsequent write in the same window — prevents flooding from long burst sequences).

## Postconditions

1. A `Finding` is pushed with:
   - `category: ThreatCategory::Execution`
   - `verdict: Verdict::Malicious`
   - `confidence: Confidence::Medium`
   - `summary`: `"Modbus coordinated control manipulation: {n} holding-register writes in {elapsed_ms}ms window (unit {unit_id})"`
     where `{n}` is `flow.t0831_window_write_count + 1` (the total writes including this one)
     and `{elapsed_ms}` is `(now_ts - flow.t0831_window_start_ts) / 1000`.
   - `evidence`: one entry — `"FC=0x{fc:02X} TxnID={txn_id:#06X} UnitID={unit_id}; window_count={n} window_start_ts={start_ts}"`.
   - `mitre_technique: Some("T0831".to_string())`
   - `source_ip: Some(flow_key.client_ip())`
   - `timestamp: Some(...)` — pcap-relative capture timestamp per BC-2.09.007.
   - `direction: Some(Direction::ClientToServer)`
2. `flow.t0831_burst_emitted = true` (suppresses further T0831 findings in this window).
3. `flow.t0831_window_write_count` is incremented by 1 (the current write is counted).
4. T0855 and T0836 findings are ALSO emitted for this PDU (per BCs 013 and 014 — T0831 does
   not replace per-write findings; it supplements them). The priority rule applies to T0836 vs
   T0835 (T0836 takes priority for this FC subset), NOT to T0855 which always fires
   independently. T0835 is NOT emitted for FCs 0x06/0x10/0x16 (T0836 priority).
5. `flow.write_count` and `self.total_write_count` incremented once.

## Invariants

1. **Window state fields** (on `ModbusFlowState` — per architecture-delta.md §2.3):
   - `t0831_window_start_ts: u32` — timestamp of first holding-register write in current window.
   - `t0831_window_write_count: u32` — count of holding-register writes in current window.
   - `t0831_burst_emitted: bool` — true once T0831 has fired in the current window.
2. **Canonical evaluation ORDER** (authoritative — window-update FIRST, then emission check):
   On every holding-register write FC (0x06, 0x10, 0x16) in ClientToServer direction:
   ```
   // STEP 1: Window-update runs FIRST on every qualifying write, unconditionally.
   if now_ts - t0831_window_start_ts > T0831_WINDOW_SECS * 1_000_000:
       // Window expired: reset (this write starts a new window)
       t0831_window_start_ts = now_ts
       t0831_window_write_count = 1
       t0831_burst_emitted = false
   else:
       // Still in window: increment
       t0831_window_write_count += 1

   // STEP 2: Emission check runs AFTER the update, using the post-update count.
   if t0831_window_write_count >= 2 AND NOT t0831_burst_emitted:
       emit T0831 finding
       t0831_burst_emitted = true
   ```
   **Critical ordering rule**: the window-update (Step 1) ALWAYS runs before the emission
   check (Step 2). This ensures the count-establishing write (first write, count 0→1) is
   tracked even though it does NOT trigger emission. A Precondition that gates Step 1 on
   "count >= 1" would make the detector permanently dead (the first write would never be
   counted). The previous Precondition 4 wording is superseded by this authoritative order.
3. **T0831 fires at most once per 5-second window per flow.** Subsequent holding-register
   writes within the same window do not generate additional T0831 findings.
4. The `T0831_WINDOW_SECS = 5` constant is fixed in v1 (not CLI-configurable).
5. A single write FC from a fresh flow (no prior holds-register writes) never fires T0831.
   The minimum condition is two writes within the window.
6. T0831 applies to the same FC subset as T0836 (0x06, 0x10, 0x16) — writes to holding
   registers. Coil writes (T0835 subset 0x05, 0x0F) do NOT contribute to the T0831 window
   counter or trigger T0831 in v1.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First holding-register write in a flow | `t0831_window_write_count = 1`, `t0831_window_start_ts = now_ts`. T0831 NOT emitted (count < 2). T0855 + T0836 emitted. |
| EC-002 | Second holding-register write within 5 seconds of the first | T0831 emitted (count becomes 2). `t0831_burst_emitted = true`. T0855 + T0836 also emitted for this PDU. |
| EC-003 | Third write within the same window | T0831 NOT emitted (`t0831_burst_emitted == true`). T0855 + T0836 emitted. `t0831_window_write_count = 3`. |
| EC-004 | Write at exactly `t0831_window_start_ts + 5_000_000` microseconds (boundary) | Window NOT expired (condition is `>`, not `>=`). If count >= 1, this is within the window. Counted toward T0831. |
| EC-005 | Write at `t0831_window_start_ts + 5_000_001` microseconds (one us past boundary) | Window expired. Reset. This write starts a new window (`count=1`). T0831 NOT emitted yet. |
| EC-006 | Coil write (FC 0x05) between two holding-register writes | Coil writes do NOT increment `t0831_window_write_count`. Only FCs {0x06, 0x10, 0x16} count toward T0831. |
| EC-007 | `all_findings.len() == MAX_FINDINGS - 1` when T0831 would fire | T0855 fills the last slot (emitted first). T0836 and T0831 are both skipped. `t0831_burst_emitted` still set to true to prevent repeated failed attempts. |
| EC-008 | Two flows with overlapping timestamps; second flow gets two writes within 5 seconds | T0831 fires for the second flow; first flow is unaffected (per-flow state isolation). |
| EC-009 | First holding-register write on a fresh flow (t0831_window_write_count is 0) | Window-update runs: count becomes 1, window_start_ts = now_ts, burst_emitted = false. Emission check: count = 1 < 2 → T0831 NOT emitted. T0855 + T0836 are emitted for this PDU (per BCs 013/014). This is the count-establishing write. |
| EC-010 | now_ts < t0831_window_start_ts (timestamp wrap-around or out-of-order packet) | `now_ts - t0831_window_start_ts` wraps to a large u32 value (≫ 5_000_000). The window-expiry check fires (treating as "window expired"), resetting the window with count = 1. This is the evasion-resistant policy: a wrapped huge value resets the window rather than suppressing detection — it does NOT allow an attacker to prevent T0831 by injecting out-of-order packets (at worst, the attacker forces a window reset, meaning T0831 requires one more write to fire again, not that detection is permanently suppressed). |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Flow A: write 1 at ts=1000 (FC=0x06, UnitID=1) → write 2 at ts=2000 (FC=0x10, UnitID=1) | Write1: T0855+T0836 (T0835 suppressed — T0836 priority for 0x06); Write2: T0855+T0836+T0831 (T0831 fires on 2nd write, count=2, elapsed=1ms; T0835 suppressed for 0x10) | happy-path |
| Flow A: write 1 at ts=1000 → write 2 at ts=6_001_000 (6 seconds later) | Write1: no T0831; Write2: new window started (ts=6001000, count=1); no T0831 (count only 1 in new window) | edge-case (expired window) |
| Flow A: three writes at ts=1000, 2000, 3000 (all FC=0x06) | Write1: no T0831 (count=1); Write2: T0831 fired (count=2, burst_emitted=true); Write3: no T0831 (burst_emitted=true) | edge-case (once-per-window) |
| Flow A: FC=0x05 (coil) at ts=1000; FC=0x06 at ts=2000 | Coil write at ts=1000 does NOT start T0831 window. Holding-register write at ts=2000: count=1, no T0831 | edge-case (coil excluded) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | classify_fc total and Write-class completeness | Kani |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC implements the coordinated-sequence detector for process control manipulation, the highest-complexity ICS attack pattern in the approved v1 scope |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-14 (analyzer/modbus.rs, C-22; ModbusFlowState t0831_* fields) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0831 — Manipulation of Control (ATT&CK for ICS; IcsImpairProcessControl tactic) |

## Related BCs

- BC-2.14.013 — composes with (T0855 co-emitted for each write PDU in the sequence)
- BC-2.14.014 — composes with (T0836 co-emitted for each write PDU in the sequence)
- BC-2.14.017 — related to (T0806 uses a separate 1-second window; T0831 uses 5-second window; both run independently in the same flow)
- BC-2.14.022 — depends on (MAX_FINDINGS cap guard)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusFlowState` with `t0831_window_start_ts`, `t0831_window_write_count`, `t0831_burst_emitted`
- `src/analyzer/modbus.rs` — T0831 coordination detector in `on_data` holding-register branch
- `src/mitre.rs` — `technique_info("T0831")` arm (new per ADR-005 §4.2)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Kani: Write-class sub-property B

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §12 (T0831 scoping note: v1 uses simpler heuristic of two writes within 5-second window); architecture-delta.md §2.6 (T0831 detection: coordinated write sequence); modbus-tcp-research.md §5 (T0831 severity rationale) |
| **Confidence** | medium (v1 heuristic is intentionally loose per §12 simplification note) |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Deterministic** | yes (same PDU sequence + timestamps always produces same output) |
| **Overall classification** | effectful shell |
