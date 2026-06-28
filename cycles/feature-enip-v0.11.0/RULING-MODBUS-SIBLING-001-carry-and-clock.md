---
document_type: arch-ruling
ruling_id: RULING-MODBUS-SIBLING-001
cycle: feature-enip-v0.11.0
wave: Wave-64
author: architect
date: 2026-06-28
status: authoritative
supersedes: []
addendum_to: RULING-EDGECASE-001
corrects: RULING-EDGECASE-001-§1.6
bcs_amended:
  - BC-2.14.002
  - BC-2.14.016
  - BC-2.14.017
  - BC-2.14.019
adrs_amended: []
story_104_acs_amended:
  - AC-006
vps_recommended:
  - VP-NEW-E: modbus-carry-direction-isolation
  - VP-NEW-F: modbus-window-monotonic-no-spurious-reset
release_blocker: true
release_held: v0.11.0
human_approved: false
sibling_sweep_tag: DF-SIBLING-SWEEP-001
---

# RULING-MODBUS-SIBLING-001: Modbus Carry-Direction Splice (EC-X1) and Clock-Backwards Window Reset (EC-X2)

## 0. Executive Summary

RULING-EDGECASE-001 §1.6 stated: "Modbus already has direction threading and is NOT
affected." This claim is **RETRACTED**. The scratch repro at
`.worktrees/modbus-ecx-verify/tests/scratch_modbus_ecx_repro.rs` (commit 74f2913) provides
empirical, test-passing confirmation that BOTH EC-X1 and EC-X2 are present in the Modbus
analyzer (`src/analyzer/modbus.rs`) as of develop @ b6d7a01. The full error analysis
for the retraction is in §5 (DF-SIBLING-SWEEP-001 lesson).

Two confirmed bugs:

- **DRIFT-MODBUS-DIRECTION-001** — cross-direction carry-buffer splice:
  `ModbusFlowState.carry: Vec<u8>` (line 170) is single and direction-shared.
  `on_data` (line 1018) receives `direction: Direction` and correctly dispatches
  detection logic by direction — but `flow.carry` is accumulated and prepended without
  regard to direction. A partial c2s ADU stashed in `carry` is prepended to the next
  s2c delivery, producing a garbled combined buffer.
  EMPIRICALLY CONFIRMED: `scratch_EC_X1_splice_confirmed_garbled_write_fires_on_s2c_direction`
  asserts `fn_code_counts[0x03]` diverges between treatment and control (treatment sees
  the garbled FC=0x06 from the spliced carry; control sees the correct FC=0x03).

- **DRIFT-MODBUS-CLOCK-001** — clock-backwards window reset:
  All four windowed timestamp comparisons use `wrapping_sub`. When `now_ts < window_start_ts`
  (backwards or out-of-order packet), `wrapping_sub` produces ~4.29e9, exceeding every
  window threshold and triggering a spurious reset that discards burst accumulation.
  EMPIRICALLY CONFIRMED: `scratch_EC_X2_explicit_window_state_via_process_pdu` asserts
  `flow.window_write_count == 1` after the backwards-ts delivery, proving the reset path
  fired and detection was suppressed.

Both are **v0.11.0 release blockers** per this ruling. The fix is one atomic story
(STORY-141), analogous to STORY-140 for DNP3 and STORY-139 for ENIP.

---

## 1. DRIFT-MODBUS-DIRECTION-001 — Cross-Direction Carry-Buffer Splice

### 1.1 Root Cause

`FlowKey` canonicalizes both TCP directions to a single key (VP-001). `ModbusFlowState`
owns ONE `carry: Vec<u8>` (line 170). `ModbusAnalyzer::on_data` already receives
`direction: Direction` (line 1018-1025 signature):

```rust
fn on_data(
    &mut self,
    flow_key: &FlowKey,
    direction: Direction,
    data: &[u8],
    _offset: u64,
    timestamp: u32,
)
```

However, `direction` is used ONLY for detection dispatch (the `match direction` blocks
in `process_pdu`) — it is NOT used to select which carry buffer to accumulate into or
prepend from. The carry operations at lines 1043-1056 (prepend), 1080-1085 (stash —
MBAP partial), and 1120-1125 (stash — ADU incomplete) all reference `flow.carry`
unconditionally regardless of direction.

**Repro mechanism** (`scratch_EC_X1_splice_confirmed_garbled_write_fires_on_s2c_direction`):

Step 1: deliver 6-byte partial c2s ADU (`[0x00,0x01,0x00,0x00,0x00,0x06]`) →
`parse_mbap_header` returns `None` (< 8 bytes) → stashed into `flow.carry`.

Step 2: deliver complete s2c ADU (TxnID=0x0006, FC=0x03, 13 bytes) on SAME FlowKey →
`on_data` builds `combined = [0x00,0x01,0x00,0x00,0x00,0x06] ++ [0x00,0x06,0x00,0x00,...]`
= 19 bytes. `parse_mbap_header(&combined)` succeeds: reads bytes 0-7 as MBAP header →
TxnID=0x0001, proto=0, length=6, unit_id=0x00, fc=combined[7]=0x06 (garbled Write).
`adu_len = 12`. `process_pdu` is called with `direction=ServerToClient` and FC=0x06 →
`fn_code_counts[0x06]` incremented in s2c direction. The true s2c FC=0x03 ADU is never
correctly parsed (its remaining 7 bytes are stashed back into carry).

The test assertion `assert_ne!(treatment_fc03, control_fc03)` passes: control sees
`fn_code_counts[0x03]=1`, treatment sees `fn_code_counts[0x06]=1` and
`fn_code_counts[0x03]=0`. This is the definitive splice proof.

### 1.2 Decision: Split carry into carry_c2s / carry_s2c

**Adopted: replace `ModbusFlowState.carry: Vec<u8>` with `carry_c2s: Vec<u8>` (client-to-server)
and `carry_s2c: Vec<u8>` (server-to-client). Route by the existing `direction` parameter
in `on_data` for all carry operations.**

This is structurally identical to RULING-EDGECASE-001 Option (b) for ENIP and
RULING-DNP3-SIBLING-001 §1.2 for DNP3.

**Why this is simpler than ENIP/DNP3:** Modbus already has `direction: Direction` in the
`on_data` signature (lines 1018-1025). No signature change is needed; no call-site sweep
of `dispatcher.rs` is required. The fix is purely additive within `modbus.rs` and
`ModbusFlowState`.

**Why not full per-direction sub-state keying?** Same reasoning as RULING-EDGECASE-001
§1.2 / RULING-DNP3-SIBLING-001 §1.2: detection counters are flow-level aggregates; only
the carry buffer is genuinely per-direction.

### 1.3 Per-Direction vs. Per-Flow State Classification

| State field | Classification | Rationale |
|-------------|---------------|-----------|
| `carry_c2s: Vec<u8>` | PER-DIRECTION (new) | Partial-ADU accumulation for master/client stream only |
| `carry_s2c: Vec<u8>` | PER-DIRECTION (new) | Symmetric for server/outstation response stream |
| `is_non_modbus: bool` | PER-FLOW (keep shared) | See §1.4 below — no change needed after carry split |
| `pending: HashMap<(u16, u8), (u8, u32)>` | PER-FLOW (keep shared) | Request/response correlation is cross-direction by definition |
| `write_count: u64` | PER-FLOW (keep shared) | Flow-level lifetime aggregate |
| `exception_count: u64` | PER-FLOW (keep shared) | Flow-level lifetime aggregate |
| `pdu_count: u64` | PER-FLOW (keep shared) | Flow-level aggregate; BC-2.14.021 scoped |
| `last_ts: u32` | PER-FLOW (keep shared) | Last-seen timestamp; not direction-specific |
| `window_write_count: u32` | PER-FLOW (keep shared) | Write burst is request-direction only by protocol; CIP gate already enforces direction selectivity |
| `window_start_ts: u32` | PER-FLOW (keep shared) | Window is flow-level |
| `window_burst_emitted: bool` | PER-FLOW (keep shared) | One-shot guard |
| `sustained_window_start_ts: u32` | PER-FLOW (keep shared) | Flow-level |
| `sustained_window_write_count: u32` | PER-FLOW (keep shared) | Flow-level |
| `sustained_burst_emitted: bool` | PER-FLOW (keep shared) | One-shot guard |
| `t0831_window_start_ts: u32` | PER-FLOW (keep shared) | Flow-level |
| `t0831_window_write_count: u32` | PER-FLOW (keep shared) | Flow-level |
| `t0831_burst_emitted: bool` | PER-FLOW (keep shared) | One-shot guard |
| `exception_window_counts: HashMap<u8, u32>` | PER-FLOW (keep shared) | Exception tracking is response-direction; MBAP gate already enforces |
| `exception_window_start_ts: HashMap<u8, u32>` | PER-FLOW (keep shared) | Flow-level |
| `exception_burst_emitted: HashMap<u8, bool>` | PER-FLOW (keep shared) | One-shot guards per exception code |

**Summary:** `ModbusFlowState.carry: Vec<u8>` is removed and replaced with `carry_c2s: Vec<u8>`
and `carry_s2c: Vec<u8>`. All other fields stay per-flow. `on_data` uses the existing
`direction` parameter ONLY to select which carry buffer to operate on.

### 1.4 is_non_modbus Latch — No Change Needed

DESIGN-CROSS-DIRECTION-STATE §4.2 analyzed this: `is_non_modbus` is set in two paths:
1. Carry-cap overflow (lines 1081, 1121) — analogous to `is_non_enip` cap. PER-FLOW correct.
2. Invalid MBAP protocol_id or out-of-range length (lines 1101-1104).

After the carry split, Path 2 can only fire on a direction-correct combined buffer
(no splice contamination). A c2s partial can no longer produce an invalid MBAP header
when the s2c delivery arrives, so the splice-driven false-desync path is eliminated.
`is_non_modbus` can remain PER-FLOW with no additional change.

This is unlike the DNP3 `is_non_dnp3` latch (which has a separate first-delivery
sync-word check that DOES have the desync-latch failure mode analyzed in
DESIGN-CROSS-DIRECTION-STATE §2 and adjudicated in RULING-DNP3-DESYNC-001).

### 1.5 Carry-Cap Reachability Assessment

**Modbus carry cap is 260 bytes (MAX_ADU_CARRY_BYTES), one max ADU.**

Two stash points in `on_data`:
- Line 1080-1085: partial MBAP header (< 8 bytes remain) stash.
- Line 1120-1125: partial ADU body (remaining.len() < adu_len) stash.

Both stash points check `flow.carry.len() + remaining.len() > MAX_ADU_CARRY_BYTES` before
stashing. Since `adu_len` is bounded at 260 by `is_valid_modbus_adu` (and the max partial
stash is `remaining.len()` which must satisfy `remaining.len() < adu_len <= 260`), a single
partial ADU is always within cap. The cap overflow path fires only via repeated adversarial
sub-8-byte deliveries that accumulate carry until a new delivery pushes the cumulative total
past 260.

**Verdict: the carry-cap overflow path IS reachable** via repeated partial-MBAP-header
deliveries (< 8 bytes each), accumulating until `carry.len() + new_chunk.len() > 260`. This
path sets `is_non_modbus = true` and terminates the flow — correct DoS-guard behavior.

After the carry split:
- `carry_c2s.len() <= 260` (capped per direction independently)
- `carry_s2c.len() <= 260` (capped per direction independently)

The overflow check at each stash point must be updated to reference the active directional
carry. Each direction has its own 260-byte DoS guard. There is no interaction across
directions for the cap.

**Unlike RULING-137-002 (ENIP overflow), there is no "exact-cap-unreachable" question
here.** The Modbus cumulative guard checks `carry.len() + remaining.len() > MAX_ADU_CARRY_BYTES`;
a cumulative total of exactly 260 does NOT exceed 260, so the guard fires at 261. This
boundary condition matches the intent and is not a spec errata.

---

## 2. DRIFT-MODBUS-CLOCK-001 — Clock-Backwards Window Reset

### 2.1 Root Cause

All four windowed timestamp comparisons in `modbus.rs` use `wrapping_sub`. The complete
list of sites:

| modbus.rs line | Window | Threshold | Operator | Detection |
|---------------|--------|-----------|----------|-----------|
| 534 | T0831 5s window | `T0831_WINDOW_SECS` (5) | `>` | Coordinated write window expiry (BC-2.14.016) |
| 595 | Burst 1s window | `WRITE_BURST_WINDOW_SECS` (1) | `>` | T0806 burst expiry (BC-2.14.017) |
| 670 | Sustained >=2s window | `WRITE_SUSTAINED_WINDOW_SECS` (2) | `>=` | T0806 sustained expiry (BC-2.14.017) |
| 820 | Exception 10s window | `EXCEPTION_WINDOW_SECS` (10) | `>` | T0888 exception burst expiry (BC-2.14.019) |

`wrapping_sub` was introduced by f2-fix-directives §11.5b with the rationale "u32 second
timestamps wrap at ~136 years — never in practice, policy kept for correctness." The
module-level doc-comment at line 112 also prescribes it: "All window-duration arithmetic
MUST use `wrapping_sub` on the u32 timestamps." **This comment is STALE and must be
corrected** (see §2.3 below).

**Adversarial evasion for Modbus burst detection:**

The repro (`scratch_EC_X2_explicit_window_state_via_process_pdu`) confirms the exact path:
deliver 20 write FCs at ts=100 (`window_write_count=20`, `window_start_ts=100`). Deliver
write 21 at ts=50 (backwards). `wrapping_sub(50, 100) = 4294967246 >> WRITE_BURST_WINDOW_SECS(1)`:
window-expired branch fires, resets `window_write_count = 1`, `window_start_ts = 50`.
The burst at count=21 never fires. The test assert `flow.window_write_count == 1` passes
(proving reset occurred), confirming EC-X2 is CONFIRMED in Modbus.

Same evasion applies to the sustained detector (T0806/BC-2.14.017 Postcondition 2), the
T0831 coordinated-write detector (BC-2.14.016), and the exception-burst detector
(BC-2.14.019).

### 2.2 Decision: Option (a) — saturating_sub

**Adopted: replace `wrapping_sub` with `saturating_sub` for all window-expiry comparison
sites in `modbus.rs`.**

Same decision and reasoning as RULING-EDGECASE-001 §2.2 and RULING-DNP3-SIBLING-001 §2.2.

| Scenario | saturating_sub result | Window action |
|----------|-----------------------|---------------|
| Forward, in-window: now=200, start=100 | 100 | No reset if 100 <= threshold |
| Forward, expired: now=150, start=100 | 50 | Reset if 50 > threshold |
| Backwards: now=50, start=100 | 0 | No reset; burst accumulation preserved |
| Genuine u32 rollover | 0 | No reset; acceptable trade-off |

**Sites to change** (4 computation sites):

| modbus.rs line | Before | After | Operator unchanged? |
|---------------|--------|-------|---------------------|
| 534 | `timestamp.wrapping_sub(flow.t0831_window_start_ts) > T0831_WINDOW_SECS` | `timestamp.saturating_sub(flow.t0831_window_start_ts) > T0831_WINDOW_SECS` | Yes (`>`) |
| 595 | `timestamp.wrapping_sub(flow.window_start_ts) > WRITE_BURST_WINDOW_SECS` | `timestamp.saturating_sub(flow.window_start_ts) > WRITE_BURST_WINDOW_SECS` | Yes (`>`) |
| 670 | `timestamp.wrapping_sub(flow.sustained_window_start_ts) >= WRITE_SUSTAINED_WINDOW_SECS` | `timestamp.saturating_sub(flow.sustained_window_start_ts) >= WRITE_SUSTAINED_WINDOW_SECS` | **Keep `>=`** (see §2.3) |
| 820 | `timestamp.wrapping_sub(*flow.exception_window_start_ts.get(&exc_code).unwrap_or(&timestamp)) > EXCEPTION_WINDOW_SECS` | `timestamp.saturating_sub(*flow.exception_window_start_ts.get(&exc_code).unwrap_or(&timestamp)) > EXCEPTION_WINDOW_SECS` | Yes (`>`) |

### 2.3 Operator Consistency Analysis

Unlike ENIP (where line 821 used `>=` against `> threshold` on the other windows) and DNP3
(where line 984 used `>=`), the Modbus windows have the following operator profile:

| Window | Operator | Consistent with ENIP/DNP3 precedent? |
|--------|----------|---------------------------------------|
| T0831 (5s) | `>` | Yes — consistent |
| Burst (1s) | `>` | Yes — consistent |
| Sustained (>=2s) | `>=` | Intentional — sustained fires ON the 2-second mark |
| Exception (10s) | `>` | Yes — consistent |

The sustained detector's `>=` is INTENTIONAL and correct: `BC-2.14.017 Postcondition 2`
specifies `elapsed_secs >= WRITE_SUSTAINED_WINDOW_SECS` as the minimum-duration gate for
the sustained detector. This is not an inconsistency — it means "the window must have
run for AT LEAST 2 seconds before the rate check applies." This is semantically different
from the expiry-reset operators (which use `>` to mean "window has expired").

**Ruling: do NOT change the `>=` at line 670.** It is semantically correct for the
sustained-window minimum-duration check.

### 2.4 Stale Module-Level Comment at Line 112

The module-level doc-comment at `modbus.rs:112` prescribes `wrapping_sub`:

```rust
/// All window-duration arithmetic MUST use `wrapping_sub` on the u32 timestamps
/// (f2-fix-directives §11.5b) — even though no window timers fire in STORY-103...
```

This comment is STALE after the EC-X2 fix. It must be changed to:

```rust
/// All window-duration arithmetic uses `saturating_sub` on the u32 timestamps
/// (RULING-MODBUS-SIBLING-001 §2.3 — replaces wrapping_sub per f2-fix-directives §11.5b).
/// Under saturating_sub, backwards-clock packets (out-of-order pcap delivery or
/// adversarial injection) produce elapsed=0, preserving burst accumulation rather than
/// triggering a spurious window reset. See RULING-MODBUS-SIBLING-001 §2.2.
```

---

## 3. Release Blocker Confirmation

**DRIFT-MODBUS-DIRECTION-001 — CONFIRMED RELEASE BLOCKER.** The carry-direction splice
produces incorrect FC classifications on any Modbus TCP flow with a TCP segment boundary
at a direction-crossing point (partial c2s ADU stashed → s2c delivery garbled). The
`scratch_EC_X1_splice_confirmed_garbled_write_fires_on_s2c_direction` test provides
definitive proof via `fn_code_counts` divergence. v0.11.0 MUST NOT ship with this bug
after explicitly fixing the identical pattern in ENIP (STORY-139) and DNP3 (STORY-140).

**DRIFT-MODBUS-CLOCK-001 — CONFIRMED RELEASE BLOCKER.** The backwards-clock window reset
suppresses burst detection (T0806), coordinated-write co-tagging (T0831), and exception
burst detection (T0888). An adversary delivering a single stale-timestamp packet can
abort any in-progress Modbus burst detection window. The `scratch_EC_X2_explicit_window_state_via_process_pdu`
test asserts `window_write_count == 1` after the backwards delivery, proving the reset.
v0.11.0 MUST NOT ship with this vulnerability after fixing the same pattern in ENIP and DNP3.

---

## 4. Binding BC and Story Amendments

### 4.1 BC-2.14.002 Amendments

**BC-2.14.002: MBAP Header Rejected for ADU Shorter than 8 Bytes (Truncation Safety)**

**Version bump: 1.0 → 2.0** (breaking structural change to ModbusFlowState carry).

1. **Precondition 3 (carry prepend):** Change from:
   > `buf = flow.carry ++ data`

   to:
   > `buf = (match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c }) ++ data`

2. **Postcondition 1 (carry stash):** Change from:
   > `flow.carry = remaining`

   to:
   > `match direction { ClientToServer => flow.carry_c2s = remaining, ServerToClient => flow.carry_s2c = remaining }`

3. **Invariant 1 (carry bounded):** Change from:
   > `flow.carry.len() <= MAX_ADU_CARRY_BYTES = 260`

   to:
   > `flow.carry_c2s.len() <= MAX_ADU_CARRY_BYTES = 260` AND `flow.carry_s2c.len() <= MAX_ADU_CARRY_BYTES = 260`

4. **Add new Invariant (direction isolation):**
   > `carry_c2s` and `carry_s2c` are NEVER mixed. `on_data` selects exactly one buffer
   > per call based on the `direction` argument. No frame-walk loop ever prepends bytes from
   > one direction into the other. Prevents DRIFT-MODBUS-DIRECTION-001 (RULING-MODBUS-SIBLING-001).

5. **Add new Edge Case EC-XXX (direction isolation):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-XXX | Partial c2s MBAP (< 8 bytes) stashed in carry_c2s; next call is s2c direction | `carry_s2c` (empty) prepended to s2c data → clean s2c parse; `carry_c2s` retains c2s partial |

6. **Architecture Anchors:** Replace `ModbusFlowState.carry: Vec<u8>` with:
   - `ModbusFlowState.carry_c2s: Vec<u8>`
   - `ModbusFlowState.carry_s2c: Vec<u8>`

### 4.2 BC-2.14.016 Amendments

**BC-2.14.016: Coordinated Write Sequence (T0831, 5-second window)**

**Version bump: 2.2 → 2.3** (window arithmetic fix, minor).

1. **Pseudocode window-expiry expression (line 130):** Change:
   > `elapsed_secs = now_ts.wrapping_sub(t0831_window_start_ts)`

   to:
   > `elapsed_secs = now_ts.saturating_sub(t0831_window_start_ts)`

2. **Invariant / rationale note (line ~157):** Change any prose citing `wrapping_sub` to
   `saturating_sub` with rationale from RULING-MODBUS-SIBLING-001 §2.2.

3. **EC-010 (backwards timestamp):** Change expected behavior from:
   > "`now_ts.wrapping_sub(t0831_window_start_ts)` yields a very large u32 value (≫ 5 seconds).
   > Window-expiry check fires (resets window). ... Evasion-resistant: attacker forcing a window
   > reset at most delays T0831 by one write."

   to:
   > "`now_ts.saturating_sub(t0831_window_start_ts)` = 0. Elapsed=0, NOT > T0831_WINDOW_SECS(5).
   > Window NOT reset. T0831 accumulation preserved. Adversarially injected stale-timestamp
   > packets cannot abort coordinated-write detection. (RULING-MODBUS-SIBLING-001 §2.2)"

4. **Add new Edge Case (T0831 backwards-clock no-reset):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-011 | First holding-register write at ts=100 (window seeded); second holding-register write at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; NOT > 5 → window NOT reset; `t0831_window_write_count` incremented to 2; T0831 co-tag fires on the second write |

### 4.3 BC-2.14.017 Amendments

**BC-2.14.017: Write-Rate Exceeding Either Burst or Sustained Threshold (T0806)**

**Version bump: 2.6 → 2.7** (window arithmetic fix, minor).

1. **Pseudocode burst window (line 171):** Change:
   > `elapsed_secs = now_ts.wrapping_sub(window_start_ts)`

   to:
   > `elapsed_secs = now_ts.saturating_sub(window_start_ts)`

2. **Pseudocode sustained window (line 204):** Change:
   > `elapsed_secs = now_ts.wrapping_sub(sustained_window_start_ts)`

   to:
   > `elapsed_secs = now_ts.saturating_sub(sustained_window_start_ts)`

3. **Invariant rationale (line ~234):** Change the `wrapping_sub` rationale note from:
   > "`wrapping_sub` handles u32 second-timestamp rollover (rolls over at ~136 years ...)"

   to:
   > "`saturating_sub` handles both backwards-clock (out-of-order or adversarial) packets and
   > genuine u32 rollover by returning 0 — preserving burst accumulation in both cases.
   > RULING-MODBUS-SIBLING-001 §2.2."

4. **EC-010 (backwards timestamp):** Change expected behavior from:
   > "`now_ts.wrapping_sub(window_start_ts)` gives a large u32 value (≫ any window threshold).
   > Both burst and sustained detectors treat this as window-expired: reset. Rollover at ~136
   > years — effectively never in practice. Correct and evasion-resistant."

   to:
   > "`now_ts.saturating_sub(window_start_ts) = 0`. Burst: NOT > WRITE_BURST_WINDOW_SECS(1) →
   > window NOT reset; `window_write_count` incremented. Sustained: NOT >= WRITE_SUSTAINED_WINDOW_SECS(2)
   > → minimum-duration gate not met on this backwards-ts call (correct). Burst accumulation
   > preserved. (RULING-MODBUS-SIBLING-001 §2.2)"

5. **Add new Edge Case (burst backwards-clock no-reset):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-012 | 20 writes at ts=100 (`window_write_count=20`, `window_start_ts=100`), then 1 write at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; NOT > 1 → window NOT reset; `window_write_count=21`; burst fires (21 > threshold=20) |

### 4.4 BC-2.14.019 Amendments

**BC-2.14.019: Exception Response Anomaly (T0888, 10-second window)**

**Version bump: 1.4 → 1.5** (window arithmetic fix, minor).

1. **Pseudocode (line 143):** Change:
   > `elapsed_secs = now_ts.wrapping_sub(exception_window_start_ts[ec])`

   to:
   > `elapsed_secs = now_ts.saturating_sub(exception_window_start_ts[ec])`

2. **EC-009 (backwards timestamp):** Change expected behavior from:
   > "`now_ts.wrapping_sub(exception_window_start_ts[ec])` yields a very large u32 value (≫ 10
   > seconds). Window-expiry fires: resets count=1... Evasion-resistant: attacker cannot
   > permanently suppress detection..."

   to:
   > "`now_ts.saturating_sub(exception_window_start_ts[ec]) = 0`. Elapsed=0, NOT > EXCEPTION_WINDOW_SECS(10)
   > → window NOT reset. Exception count incremented. Burst accumulation preserved. Adversarially
   > injected stale-timestamp exception responses cannot abort detection. (RULING-MODBUS-SIBLING-001 §2.2)"

### 4.5 STORY-104 AC-006 Correction

STORY-104 AC-006 mandates `wrapping_sub` and includes a test `test_window_elapsed_uses_wrapping_sub`:

> "AC-006 (traces to BC-2.14.017 — wrapping_sub for all window elapsed computations)
> All four window-duration computations use `now_ts.wrapping_sub(window_start_ts)` (not
> plain subtraction)."

And the test:
> "Test: `test_window_elapsed_uses_wrapping_sub()` — deliver a write at `ts=0xFFFFFF00`
> (near u32::MAX) followed by a write at `ts=0x00000100` (wrapped); assert
> `wrapping_sub(0x00000100, 0xFFFFFF00) = 0x00000200` (512 µs); assert no panic."

**Correction in STORY-141:** AC-006 must be amended to prescribe `saturating_sub`.
The existing regression test must be rewritten to test `saturating_sub` semantics
(saturating_sub of a genuine rollover returns 0, not 0x200 — the test assertion changes).
The new test must also add the backwards-clock scenario from the repro.

---

## 5. VP Recommendations

### VP-NEW-E: Modbus Carry-Direction Isolation Invariant

**Recommendation: ADD as a proptest VP.** Analog of VP-033 (ENIP) and VP-NEW-C (DNP3).

Property: for any sequence of `on_data` calls with alternating `Direction::ClientToServer`
and `Direction::ServerToClient` delivering partial ADUs, `carry_c2s` is NEVER read during
a `ServerToClient` call, and `carry_s2c` is NEVER read during a `ClientToServer` call.
Validated by a proptest strategy generating random (direction, partial_bytes, full_bytes)
sequences and asserting that per-direction `fn_code_counts` (or total `pdu_count`) match
independent single-direction control runs.

**Proof method: proptest.** Mirrors VP-NEW-C for DNP3 exactly.

**VP number:** to be assigned by spec-steward. Traces to BC-2.14.002 (new direction-isolation
invariant).

### VP-NEW-F: Modbus Window Monotonic No-Spurious-Reset

**Recommendation: ADD as a proptest VP.** Analog of VP-034 (ENIP) and VP-NEW-D (DNP3).

Property: for all four windowed detections (T0806 burst/1s, T0806 sustained/>=2s, T0831/5s,
T0888/10s), a single event with `now_ts < window_start_ts` does NOT reset the window.
Formally: if burst of N events has accumulated and one backwards-ts event arrives, then
one more forward-ts event must still trigger the detection (for the burst/T0831/T0888
windows; sustained window behavior under backwards-ts tested separately).

**Proof method: proptest over (window_start, burst_count, backwards_ts, threshold) triples.**
Mirrors VP-NEW-D for DNP3 exactly.

**VP number:** to be assigned by spec-steward. Traces to BC-2.14.016 (new EC-011),
BC-2.14.017 (new EC-012), BC-2.14.019 (amended EC-009).

---

## 6. STORY-141 Scope and Acceptance Criteria

**Story title:** "Fix Modbus DRIFT-MODBUS-DIRECTION-001 carry-direction splice + DRIFT-MODBUS-CLOCK-001 clock-backwards window reset"

**Scope:** Single atomic story. Implements all changes from §1-§2 of this ruling.
The `direction: Direction` parameter is already available in `on_data` — no signature
change, no dispatcher call-site sweep.

**Acceptance Criteria:**

1. `ModbusFlowState` has `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`; `carry: Vec<u8>` is removed.
2. `on_data` uses `direction` to select the active carry: `let active_carry = match direction { ClientToServer => &mut flow.carry_c2s, ServerToClient => &mut flow.carry_s2c };`. This active carry is used for ALL carry operations: prepend (lines 1043-1053), MBAP-partial stash (line 1084), ADU-partial stash (line 1124), and the cap-check guards (lines 1080, 1120). No signature change to `on_data`.
3. The carry-cap DoS guard is applied to the active directional carry independently on each stash path. `is_non_modbus` is set if EITHER directional carry cap overflows (per §1.4 above, single shared latch is correct).
4. The stale module-level doc-comment at line 112 prescribing `wrapping_sub` is corrected to prescribe `saturating_sub` (per §2.4).
5. All four window-expiry sites use `saturating_sub` instead of `wrapping_sub`:
   - Line 534 (`t0831_window_start_ts`)
   - Line 595 (`window_start_ts` — burst)
   - Line 670 (`sustained_window_start_ts`)
   - Line 820 (`exception_window_start_ts[ec]`)
6. The sustained-window `>=` operator at line 670 is KEPT (intentional; see §2.3).
7. All existing Modbus tests pass.
8. **New regression test (carry direction isolation):** Deliver partial c2s ADU (6 bytes of a valid MBAP prefix, not a complete 8-byte-min ADU), then deliver a complete s2c FC=0x03 read response ADU on the SAME FlowKey. Assert: `fn_code_counts[0x03] == 1` (not 0), `fn_code_counts[0x06] == 0` (no garbled write), `parse_errors == 0`. This is the EC-X1 formalized regression (based on `scratch_EC_X1_splice_confirmed_garbled_write_fires_on_s2c_direction`).
9. **New regression test (same-direction carry completes normally):** Deliver partial c2s ADU (first 6 bytes), then deliver the remaining bytes of the same c2s ADU on the SAME FlowKey, SAME direction. Assert: `total_pdu_count == 1`, `total_write_count == 1` (FC=0x06), `parse_errors == 0`.
10. **New regression test (backwards-clock burst no-reset):** Deliver 20 write FCs at ts=100 (`window_write_count` reaches 20, threshold=20), then 1 write FC at ts=50 (backwards), then 1 write FC at ts=100. Assert: `window_write_count >= 21` (no reset on backwards-ts call); T0806 burst finding fires (count > threshold=20). Based on `scratch_EC_X2_treatment_backwards_timestamp_suppresses_burst`.
11. **AC-006 corrected:** the existing `test_window_elapsed_uses_wrapping_sub` test is replaced with a test that verifies `saturating_sub` semantics — specifically that a backwards-ts write does NOT reset `window_write_count` (see §4.5).
12. VP-NEW-E proptest (carry direction isolation) is drafted in the same story.
13. VP-NEW-F proptest (window monotonic no-spurious-reset) is drafted in the same story.

---

## 7. Spec-Steward Actions Required

| BC | Old version | New version | Input-hash impact |
|----|------------|------------|-------------------|
| BC-2.14.002 | 1.0 | 2.0 | STALE on version bump |
| BC-2.14.016 | 2.2 | 2.3 | STALE on version bump |
| BC-2.14.017 | 2.6 | 2.7 | STALE on version bump |
| BC-2.14.019 | 1.4 | 1.5 | STALE on version bump |

**Actions:**

1. Bump versions in frontmatter of all four BCs per the table above.
2. Add `modified:` entry to each BC frontmatter citing this ruling.
3. Amend STORY-104 AC-006 to prescribe `saturating_sub` and update the regression test assertion (see §4.5).
4. Run `bin/compute-input-hash --scan` after BC edits. Any story whose `inputs:` list includes
   the four BCs above will report STALE and must be regenerated before Phase 4 entry.
5. Assign VP numbers for VP-NEW-E and VP-NEW-F and register in VP-INDEX.md.
6. Update VP-INDEX counts and propagate to `verification-architecture.md` and
   `verification-coverage-matrix.md` per the VP-index propagation obligation.
7. Add this ruling to `decisions-archive.md` under the `feature-enip-v0.11.0` cycle.

---

## 8. DF-SIBLING-SWEEP-001 Lesson

This finding is a direct consequence of the DF-SIBLING-SWEEP-001 failure documented in
RULING-EDGECASE-001 §1.6 CORRECTION (see the addendum). The correction to §1.6 explains
why "direction threading in the on_data signature" is NOT equivalent to "per-direction
carry." This ruling is the downstream fix.

**Key lesson:** Direction threading (i.e., `on_data` receives `direction: Direction` as
a parameter and uses it for detection dispatch) does NOT automatically mean carries are
per-direction. A codebase audit for EC-X1 analog must inspect the carry ACCUMULATION and
PREPEND paths, not just the detection dispatch path. Modbus had direction threading in its
on_data signature but shared a single carry — exactly the category error that §1.6 missed.

---

## 9. Summary for Test-Writer and Implementer

**Implementer (carry fix — DRIFT-MODBUS-DIRECTION-001):**
- Replace `ModbusFlowState.carry: Vec<u8>` (line 170) with `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`.
- In `on_data` (line 1043): add `let active_carry = match direction { Direction::ClientToServer => &mut flow.carry_c2s, Direction::ServerToClient => &mut flow.carry_s2c };`
- Replace all `flow.carry` references in `on_data` with `active_carry`.
- Cap check at lines 1080 and 1120: check `active_carry.len() + remaining.len() > MAX_ADU_CARRY_BYTES`.
- Clear active carry at the start: replace `flow.carry.clear()` (line 1056) with `active_carry.clear()`.
- Stash at line 1084: `active_carry.extend_from_slice(remaining)`.
- Stash at line 1124: `active_carry.extend_from_slice(remaining)`.
- Update module-level doc-comment at line 112 per §2.4.
- No changes to `on_data` signature, `process_pdu` signature, or `dispatcher.rs`.

**Implementer (window fix — DRIFT-MODBUS-CLOCK-001):**
- Replace every `now_ts.wrapping_sub(...)` / `timestamp.wrapping_sub(...)` at lines 534, 595, 670, 820 with the saturating variant.
- Keep the `>=` operator at line 670 (sustained window minimum-duration gate — intentional).
- Correct the stale doc-comment at line 112.

**Test-writer:**
- AC-8 regression test: carry direction isolation (partial c2s → complete s2c; assert correct FC parse).
- AC-9 regression test: same-direction carry completes normally (2-part c2s; assert pdu_count=1).
- AC-10 regression test: backwards-clock burst no-reset (20 writes at ts=100 → 1 at ts=50 → 1 at ts=100; assert burst fires).
- Replace AC-006 test per §4.5.
- VP-NEW-E proptest: random (direction, partial_bytes) sequences; assert carry isolation via fn_code_counts.
- VP-NEW-F proptest: random (window_start, burst_count, backwards_ts, threshold); assert no spurious reset.
