---
document_type: arch-ruling
ruling_id: RULING-DNP3-SIBLING-001
cycle: feature-enip-v0.11.0
author: architect
date: 2026-06-27
status: authoritative
supersedes: []
addendum_to: RULING-EDGECASE-001
bcs_amended:
  - BC-2.15.016
  - BC-2.15.010
  - BC-2.15.014
  - BC-2.15.015
adrs_amended:
  - ADR-007
vps_recommended:
  - VP-NEW-C: dnp3-carry-direction-isolation
  - VP-NEW-D: dnp3-window-monotonic-no-spurious-reset
release_blocker: true
release_held: v0.11.0
human_approved: true
---

# RULING-DNP3-SIBLING-001: DNP3 Direction Carry Splice and Clock-Backwards Window Reset

## 0. Executive Summary

Human approved atomic fix: DNP3 (`src/analyzer/dnp3.rs`) shares BOTH bug patterns confirmed
in RULING-EDGECASE-001 for ENIP. This ruling adjudicates the analogous fix for DNP3, to be
delivered atomically with the ENIP fix in v0.11.0.

Two confirmed bugs:

- **DRIFT-DNP3-DIRECTION-001** — cross-direction carry-buffer splice: `Dnp3FlowState` owns a
  single `carry: Vec<u8>` (line 197); `on_data` takes no `Direction` parameter (line 313).
  Both TCP directions share one carry buffer. A partial master-to-outstation frame in carry
  corrupts the next outstation-to-master delivery.
- **DRIFT-DNP3-CLOCK-001** — clock-backwards / out-of-order timestamp window reset:
  `wrapping_sub` is used across all windowed comparisons in `on_data` and helper methods.
  A backwards-timestamp packet wraps to ~4.29e9, exceeding any threshold, causing spurious
  window resets that suppress burst detections (T1692.001, T1691.001, T0814).

Additionally:

- **DRIFT-DNP3-OP-001** — operator inconsistency: the 300s correlation-window expiry at
  line 984 uses `>= CORRELATION_WINDOW_SECS` while all other DNP3 window checks use `>`.
  Pinned to strict `>` to match the established pattern, analogous to EC-X4 for ENIP.

All three defects are **v0.11.0 release blockers** per this ruling. The fix is one atomic
story (STORY-140), mirroring the STORY-139 scope for ENIP.

---

## 1. DRIFT-DNP3-DIRECTION-001 — Cross-Direction Carry-Buffer Splice

### 1.1 Root Cause

`FlowKey` canonicalizes both TCP directions to a single key (VP-001, verified and immutable).
`Dnp3FlowState` owns ONE `carry: Vec<u8>` (line 197). `Dnp3Analyzer::on_data` takes no
`Direction` parameter (line 313 signature: `fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32)`).
Consequently, both TCP directions (master-to-outstation and outstation-to-master) share
the same carry buffer. A partial master frame stashed into carry is prepended to the
next outstation delivery:

```
buf = carry(master_partial_header) ++ outstation_bytes
```

The frame-walk loop reads `carry[0..10]` as a DNP3 link-layer header. If the spliced
buffer passes the sync gate (`[0x05, 0x64]` already present in carry head) and the
`compute_dnp3_frame_len` check, the loop may commit the splice as a valid PDU — dispatching
FC classification against the wrong application bytes.

**FIR=1 severity modulation (ADR-007 Decision 3):** DNP3's application FC is extracted
only from FIR=1 transport segments (`transport_is_fir`, line 554). This gate operates on
`flow.carry[10]` (transport octet) after the frame passes the sync and length gates. A
spliced buffer from a partial request stashed in carry may present a transport octet whose
bit 6 (`0x40`) is coincidentally set, causing a spurious FC classification. Alternatively,
if bit 6 is NOT set (FIR=0), the FC extraction is skipped entirely, causing a genuine
first-fragment FC to be suppressed. The FIR=1 gate reduces the probability of a detection
false-positive relative to ENIP (where every PDU header is inspected), but it does NOT
eliminate the splice risk. Cross-direction carry corruption produces:
- Spurious `parse_errors` increments when the spliced frame fails the validity gate.
- Spurious `malformed_in_window` increments and premature T0814 findings.
- Missed FC classification (FIR=0 on a spliced transport octet suppresses a genuine FIR=1
  detection from the other direction's frame).

The ENIP ruling (RULING-EDGECASE-001 §1.6) characterized the DNP3 splice severity as
"lower" because the FIR=1 gate makes false detection harder. That assessment holds for
**false-positive detections**. However, the **false-negative** path (spurious parse_errors
consuming legitimate frames, desync-latch propagation, missed FIR=1 FCs) is equally
dangerous and makes this a confirmed correctness defect rather than a theoretical one.

### 1.2 Decision: Two Carry Buffers per FlowState

**Adopted: replace `Dnp3FlowState.carry: Vec<u8>` with `carry_c2s: Vec<u8>` (master-to-outstation)
and `carry_s2c: Vec<u8>` (outstation-to-master). Thread `direction: Direction` into `on_data`.**

This is the same structural fix as RULING-EDGECASE-001 Option (b). All rationale from
§1.2 of that ruling applies identically. The fix isolates only the carry buffer (the
genuine per-direction state) while leaving all detection counters as bidirectional per-flow
aggregates.

**Why not full per-direction sub-state?** For the same reasons as ENIP: windowed
correlation fields (`restart_event_count`, `block_event_count`, `correlation_window_start_ts`)
are flow-level aggregates across both directions by spec design (BC-2.15.014, BC-2.15.015).
Direction-bifurcation of all detection state is a v0.12.0 scope item.

### 1.3 Per-Direction vs. Per-Flow State Classification

| State field | Classification | Rationale |
|-------------|---------------|-----------|
| `carry_c2s: Vec<u8>` | PER-DIRECTION (new) | Partial-frame accumulation for master-to-outstation stream only. A partial request frame is meaningless in the outstation stream. |
| `carry_s2c: Vec<u8>` | PER-DIRECTION (new) | Symmetric for outstation-to-master stream. |
| `is_non_dnp3: bool` | PER-FLOW (keep shared) | Stream-level desync applies to the whole flow; both directions share the classification. |
| `fc_counts: HashMap<u8, u64>` | PER-FLOW (keep shared) | Flow-level FC distribution for reporting (BC-2.15.020). |
| `frame_count: u64` | PER-FLOW (keep shared) | Total frames across both directions. |
| `parse_errors: u64` | PER-FLOW (keep shared) | Lifetime structural-error counter; not direction-specific for reporting (BC-2.15.024 Invariant 1). |
| `direct_operate_count: u32` | PER-FLOW (keep shared) | Burst detection counts Control-class FCs flow-wide (BC-2.15.010 Invariant 2). |
| `window_start_ts: u32` | PER-FLOW (keep shared) | Detection window is flow-level. |
| `direct_operate_emitted: bool` | PER-FLOW (keep shared) | One-shot guard, flow-level. |
| `master_addrs_seen: Vec<u16>` | PER-FLOW (keep shared) | Master address tracking aggregates both directions (BC-2.15.016 PC5). |
| `restart_event_count: u64` | PER-FLOW (keep shared) | T0827 accumulator; shared 300s window (BC-2.15.015). |
| `block_event_count: u64` | PER-FLOW (keep shared) | T0827 accumulator; shared 300s window (BC-2.15.015). |
| `pending_requests: HashMap<...>` | PER-FLOW (keep shared) | Request/response correlation is across both directions (BC-2.15.014). |
| `block_finding_emitted_this_window: bool` | PER-FLOW (keep shared) | One-shot guard; flow-level. |
| `loss_of_control_emitted: bool` | PER-FLOW (keep shared) | T0827 one-shot; flow-level. |
| `correlation_window_start_ts: u32` | PER-FLOW (keep shared) | Single shared 300s window (BC-2.15.015 single reset owner). |
| `correlation_window_seeded: bool` | PER-FLOW (keep shared) | Window-seed flag; flow-level. |
| `malformed_in_window: u64` | PER-FLOW (keep shared) | T0814 windowed counter; flow-level. |
| `malformed_anomaly_emitted: bool` | PER-FLOW (keep shared) | T0814 one-shot; flow-level. |
| `enable_unsolicited_seen: bool` | PER-FLOW (keep shared) | Context flag; flow-level. |
| `response_seen: bool` | PER-FLOW (keep shared) | Context flag; flow-level. |
| `unsolicited_anomaly_emitted: bool` | PER-FLOW (keep shared) | One-shot guard; flow-level. |
| `unexpected_source_emitted: bool` | PER-FLOW (keep shared) | One-shot guard; flow-level (BC-2.15.010 Invariant 5). |

**Summary:** `Dnp3FlowState.carry: Vec<u8>` is removed and replaced with `carry_c2s: Vec<u8>`
and `carry_s2c: Vec<u8>`. All other fields stay per-flow. `on_data` receives `direction: Direction`
and uses it ONLY to select which carry buffer to operate on.

### 1.4 DRIFT-DNP3-DIRECTION-001 — Source IP Resolution Fix-Along

`resolve_master_ip` (line 1463) uses a port-20000 heuristic to identify the master endpoint.
The doc-comment at line 1456 explicitly marks this as a **direction-deferral**:

> "Direction-aware resolution — analogous to `src/analyzer/modbus.rs` ~355–382 — is deferred
> to a post-v0.6.0 follow-up chore."

Threading `Direction` into `on_data` as required by the carry split also enables fixing
this deferral. The port-20000 heuristic fails when neither endpoint is on port 20000
(non-standard outstation port, proxied capture). The direction-aware fix replaces it with:

```rust
// Direction::ClientToServer = master (initiates connections to port 20000)
// Direction::ServerToClient = outstation (listens on port 20000)
let master_ip = match direction {
    Direction::ClientToServer => flow_key.src_ip_of(direction),
    Direction::ServerToClient => flow_key.dst_ip_of(direction),
};
```

Exact implementation mirrors the Modbus pattern (`src/analyzer/modbus.rs` ~355-382).
This MUST be included in STORY-140 (same no-cost fix-along reasoning as RULING-EDGECASE-001 §1.4).

### 1.5 on_data Signature Change

**Before** (line 313):
```rust
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32)
```

**After:**
```rust
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction)
```

`Direction` lives at `crate::reassembly::handler::Direction`. It is already imported in
`modbus.rs` and, after STORY-139, in `enip.rs`. All `Dnp3Analyzer::on_data` call sites
in `src/dispatcher.rs` must be updated to pass `Direction` from the `StreamHandler`
context — matching exactly the STORY-139 pattern.

---

## 2. DRIFT-DNP3-CLOCK-001 — Clock-Backwards Window Reset

### 2.1 Root Cause

All windowed timestamp comparisons in `dnp3.rs` use `wrapping_sub`. The complete list:

| File:Line | Window | Threshold | Operator | Detection |
|-----------|--------|-----------|----------|-----------|
| `dnp3.rs:745` | 60s detect window | `DETECTION_WINDOW_SECS` | `>` | T1692.001 expiry check (reset arm) |
| `dnp3.rs:765` | 60s detect window | `DETECTION_WINDOW_SECS` | `<=` | T1692.001 in-window emit guard |
| `dnp3.rs:769` | 60s detect window | — | (elapsed) | T1692.001 elapsed display |
| `dnp3.rs:895` | 10s block timeout | `BLOCK_CMD_TIMEOUT_SECS` | `>` | T1691.001 pending-request timeout |
| `dnp3.rs:984` | 300s correlation window | `CORRELATION_WINDOW_SECS` | `>=` | All six-field window reset (BC-2.15.015) |
| `dnp3.rs:1025` | 300s correlation window | — | (elapsed) | T0827 elapsed display |
| `dnp3.rs:1335` | 300s correlation window | `CORRELATION_WINDOW_SECS` | `<` | T0814 in-window guard (`check_malformed_anomaly`) |
| `dnp3.rs:1341` | 300s correlation window | — | (elapsed) | T0814 elapsed display |

`wrapping_sub` was introduced (BC-2.15.014 Inv 3 / BC-2.15.015 v1.3) to prevent panic
under `overflow-checks=true` when timestamps go backwards (out-of-order pcap replay). However,
the same backwards-clock adversarial evasion described in RULING-EDGECASE-001 §2.1 applies:
when `now_ts < window_start_ts`, `wrapping_sub` wraps to ~4.29e9, exceeding every threshold,
causing spurious window resets that discard burst accumulation in progress.

Adversarial scenario for DNP3: send 9 Control-class FCs to build `direct_operate_count=9`
(one below the threshold of 10), then inject one rogue packet with a stale timestamp
(e.g., `ts = window_start_ts - 1`). `wrapping_sub` wraps to ~4.29e9 >> 60, the 60s detect
window resets, `direct_operate_count` falls to 1, and T1692.001 is suppressed. Repeat
for T1691.001 (block-timeout window) and T0814 (300s malformed window).

### 2.2 Decision: Option (a) — saturating_sub

**Adopted: replace `wrapping_sub` with `saturating_sub` for all window-expiry comparison
sites in `dnp3.rs`.**

Same decision and reasoning as RULING-EDGECASE-001 §2.2. Under `saturating_sub`:
- Backwards clock (`now_ts < window_start`): result = 0; elapsed = 0; window NOT reset;
  burst accumulation preserved. Adversarially injected stale-timestamp packet cannot abort
  detection.
- Genuine u32 rollover (`now_ts` post-wrap, `window_start` pre-wrap): result = 0; window
  also NOT reset. Acceptable trade-off (genuine rollover is ~136-year event; security cost
  of rollover-non-reset is negligible).

**Sites to change** (8 code sites → 5 computation sites; elapsed-display sites inherit):

| dnp3.rs line | Before | After |
|-------------|--------|-------|
| 745 | `now_ts.wrapping_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS` | `now_ts.saturating_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS` |
| 765 | `now_ts.wrapping_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS` | `now_ts.saturating_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS` |
| 769 | `now_ts.wrapping_sub(flow.window_start_ts)` (elapsed) | `now_ts.saturating_sub(flow.window_start_ts)` |
| 895 | `now_ts.wrapping_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS` | `now_ts.saturating_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS` |
| 984 | `now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS` | `now_ts.saturating_sub(flow.correlation_window_start_ts) > CORRELATION_WINDOW_SECS` |
| 1025 | `now_ts.wrapping_sub(flow.correlation_window_start_ts)` (elapsed) | `now_ts.saturating_sub(flow.correlation_window_start_ts)` |
| 1335 | `now_ts.wrapping_sub(flow.correlation_window_start_ts) < CORRELATION_WINDOW_SECS` | `now_ts.saturating_sub(flow.correlation_window_start_ts) < CORRELATION_WINDOW_SECS` |
| 1341 | `now_ts.wrapping_sub(flow.correlation_window_start_ts)` (elapsed) | `now_ts.saturating_sub(flow.correlation_window_start_ts)` |

Note also that line 984's operator changes from `>=` to `>` (see §3 below).

The elapsed-display sites (769, 1025, 1341) are also updated for consistency; under
`saturating_sub` a backwards-clock packet displays elapsed=0 rather than ~4.29e9, which
is the correct behavior in findings and summary output.

### 2.3 Operator Pinning (DRIFT-DNP3-OP-001)

Line 984 uses `>= CORRELATION_WINDOW_SECS` while all other window expiry checks in
`dnp3.rs` (and in the now-fixed `enip.rs`) use strict `>`. This is an operator inconsistency
analogous to EC-X4 for ENIP.

**Ruling: change line 984 from `>=` to `>`**, consistent with:
- `dnp3.rs:745` T1692.001 window: `> DETECTION_WINDOW_SECS`
- `dnp3.rs:895` T1691.001 timeout: `> BLOCK_CMD_TIMEOUT_SECS`
- All three ENIP windows after STORY-139 fix

Semantic: `> CORRELATION_WINDOW_SECS` means the packet at exactly elapsed=300s is the
last packet of the current window, not the first of the new one. This is more intuitive
for sustained-pattern correlation: a restart event at exactly the 300-second mark is not
spuriously moved into a fresh window.

Note: BC-2.15.015 Postcondition 3 and Invariant 6 currently specify `>= CORRELATION_WINDOW_SECS`
(carried from BC v1.3 which introduced `wrapping_sub`). This BC must be amended to `>` in F2.

---

## 3. Release Blocker Confirmation

**DRIFT-DNP3-DIRECTION-001 — CONFIRMED RELEASE BLOCKER.** The cross-direction carry splice
produces spurious `parse_errors` and `malformed_in_window` increments on any DNP3 TCP flow
that experiences a segment boundary at a direction-crossing point. This is a common TCP
condition in pcap analysis. The FIR=1 gate reduces false-positive detection artifacts but
does not prevent false-negative suppression of legitimate FIR=1 frames or spurious T0814
findings from inflated `malformed_in_window`. A confirmed correctness defect affecting
detection quality.

**DRIFT-DNP3-CLOCK-001 — CONFIRMED RELEASE BLOCKER.** The backwards-clock window reset
is an adversarial evasion path identical to ENIP EC-X2: a single injected stale-timestamp
packet can suppress T1692.001 burst detection (60s window), T1691.001 block-inference
(10s timeout), and T0814 malformed-anomaly (300s window) simultaneously. This violates
the threat-detection intent of all three BCs. v0.11.0 MUST NOT ship this vulnerability
in the DNP3 analyzer after explicitly fixing the same pattern in ENIP.

**DRIFT-DNP3-OP-001 — NOT a standalone blocker.** Operator inconsistency folded into
DRIFT-DNP3-CLOCK-001 fix atomically.

---

## 4. Carry-Cap Reachability Assessment (Analog to RULING-137-002)

RULING-137-002 assessed ENIP carry-cap overflow reachability for BC-2.17.016. The DNP3
analog is BC-2.15.016.

**DNP3 carry-cap at 292 bytes (MAX_DNP3_FRAME_LEN).** The overflow path in `on_data` is
at lines 371-400: `if data.len() > remaining_capacity`. This fires when `carry.len() + data.len() > 292`.
Since carry is bounded to ≤292 by construction (it can only grow to 292 before the overflow
arm fires), the overflow arm can be reached via the standard accumulation path:
repeated sub-frame deliveries accumulate carry until a new delivery would push it past 292.

**Unlike the ENIP situation, there is no "exactly-cap is unreachable" ambiguity here.**
The ENIP carry-cap was 600 bytes with a specific "Path B (frame-skip)" reachability
question. DNP3's overflow arm simply guards `carry.len() + new_bytes > MAX_DNP3_FRAME_LEN`.
This is reachable via any adversarial partial-frame flood delivering sub-292-byte chunks
until carry is full, then delivering one more byte. BC-2.15.016 EC-004 correctly
characterizes this; there is no RULING-137-002-style unreachability issue for DNP3.

After the carry split, the bound becomes:
- `carry_c2s.len() <= 292` (capped per direction)
- `carry_s2c.len() <= 292` (capped per direction)

Each directional carry has its own 292-byte cap. The overflow arm must check and handle
each carry independently. There is no interaction across directions for the cap.

---

## 5. Binding BC and ADR Amendments

### 5.1 BC-2.15.016 Amendments

**Version bump: 1.6 → 2.0** (breaking structural change to Dnp3FlowState).

1. **Description (carry field):** Replace:
   > `flow.carry: Vec<u8>` accumulates partial TCP segments until a complete DNP3 link frame boundary is available

   with:
   > `flow.carry_c2s: Vec<u8>` (master-to-outstation) and `flow.carry_s2c: Vec<u8>` (outstation-to-master) accumulate partial TCP segments per direction until a complete DNP3 link frame boundary is available. RULING-DNP3-SIBLING-001 §1.3: carry is split per-direction to prevent cross-direction splice.

2. **Postcondition 1 (carry prepend):** Change from:
   > Incoming bytes are appended to `flow.carry` on each `on_data` call.

   to:
   > Incoming bytes are appended to `(match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c })` on each `on_data` call.

3. **Postcondition 2 (cap check):** Update both the field name (`carry_c2s` / `carry_s2c`) and
   clarify that each directional carry is independently bounded at 292 bytes:
   > `flow.carry_c2s.len() <= 292` AND `flow.carry_s2c.len() <= 292`. Overflow checked and capped per direction independently.

4. **Postcondition 3 (frame consume):** Update to reference the active directional carry:
   > `(active_carry).drain(..frame_len)` where `active_carry` is `carry_c2s` or `carry_s2c` per direction.

5. **Postcondition 4 (residual stash):** Rename `flow.carry` → directional carry.

6. **Invariant 1 (carry bounded):** Change to:
   > `flow.carry_c2s.len() <= 292` AND `flow.carry_s2c.len() <= 292`

7. **Add new Invariant 6 (direction isolation):**
   > `carry_c2s` and `carry_s2c` are NEVER mixed. `on_data` selects exactly one of the two
   > buffers based on the `direction` argument on every call. No frame-walk loop ever
   > prepends bytes from one direction into the other. This invariant prevents the
   > cross-direction splice documented in DRIFT-DNP3-DIRECTION-001 (RULING-DNP3-SIBLING-001).

8. **Add new Edge Case EC-010 (direction isolation):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-010 | Partial master-to-outstation frame stashed in carry_c2s; next call is outstation-to-master direction | `carry_s2c` is prepended to s2c data (`carry_c2s` NOT involved); s2c frame processes cleanly; `carry_c2s` retains partial c2s bytes |

9. **Architecture Anchors:** Replace `Dnp3FlowState.carry: Vec<u8>` with:
   - `Dnp3FlowState.carry_c2s: Vec<u8>`
   - `Dnp3FlowState.carry_s2c: Vec<u8>`

10. **on_data signature update:** Reflect new `direction: Direction` parameter in Precondition 2.

### 5.2 BC-2.15.010 Amendments

**Version bump: 1.7 → 1.8** (window operator fix, minor).

1. **Postcondition 4 (window expiry):** Change:
   > `now_ts.wrapping_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS`

   to:
   > `now_ts.saturating_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS`

2. **Add new Edge Case EC-012 (backwards timestamp, analogous to ENIP BC-2.17.008 EC-009):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-012 | 9 Control-class FCs at ts=100 (window_start=100), then 1 FC at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; elapsed=0, NOT > 60 → window NOT reset; `direct_operate_count=10`; threshold not yet exceeded at count=10 (uses strict `>`). One more FC at ts=100 yields count=11 > 10 → T1692.001 fires |

3. **Postcondition 3 in-window emit guard:** Change:
   > `now_ts.wrapping_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS`

   to:
   > `now_ts.saturating_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS`

### 5.3 BC-2.15.014 Amendments

**Version bump: 2.0 → 2.1** (timeout operator fix, minor).

1. **Precondition 3 (timeout check):** Change:
   > `now_ts.wrapping_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS`

   to:
   > `now_ts.saturating_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS`

   The Invariant 2 rationale note ("wrapping_sub used ... wrap at ~136 years — effectively
   never, policy kept") must be updated to reference `saturating_sub` with the same
   backwards-clock safety justification from RULING-DNP3-SIBLING-001 §2.2.

2. **Add new Edge Case EC-009 (backwards timestamp on pending-request timeout):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-009 | Control request at ts=100 inserted in pending_requests; subsequent on_data at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; elapsed=0, NOT > 10 → timeout NOT fired; request remains pending. Subsequent on_data at ts=111: `saturating_sub(111, 100) = 11 > 10` → timeout fires, `block_event_count` incremented |

### 5.4 BC-2.15.015 Amendments

**Version bump: 1.9 → 2.0** (window operator fix `>=` → `>`, plus backwards-clock).

1. **Description (window-expiry handler):** Change:
   > `now_ts.wrapping_sub(flow.correlation_window_start_ts) >= CORRELATION_WINDOW_SECS`

   to:
   > `now_ts.saturating_sub(flow.correlation_window_start_ts) > CORRELATION_WINDOW_SECS`

   Note: BOTH the `wrapping` → `saturating` change AND the `>=` → `>` operator pin are applied here.

2. **Postcondition 3 (window reset condition):** Update the condition expression to:
   > `now_ts.saturating_sub(flow.correlation_window_start_ts) > CORRELATION_WINDOW_SECS`

3. **Invariant 6 (single reset owner):** Update the window-expiry expression from:
   > `now_ts.wrapping_sub(correlation_window_start_ts) >= CORRELATION_WINDOW_SECS`

   to:
   > `now_ts.saturating_sub(correlation_window_start_ts) > CORRELATION_WINDOW_SECS`

4. **Add new Edge Case EC-010 (backwards timestamp on 300s window, analogous to ENIP BC-2.17.018 EC-008):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-010 | 2 restarts at ts=100 (`restart_event_count=2`, `correlation_window_start_ts=100`), then one event at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; NOT > 300 → window NOT reset; `restart_event_count` still 2 on the backwards-ts call. Next event at ts=101 with another restart: `restart_event_count=3`, T0827 fires |

---

## 6. ADR-007 Amendment

**ADR-007 Decision 2 (DNP3 carry-buffer pattern)** must be updated to reflect the carry split:

Replace the single `carry: Vec<u8>` reference in the struct layout with:

```rust
/// Partial-frame accumulation buffer — MASTER-TO-OUTSTATION direction only.
/// Max 292 bytes (MAX_DNP3_FRAME_LEN). Bounded DoS guard.
/// RULING-DNP3-SIBLING-001 §1.3: carry is split per-direction.
carry_c2s: Vec<u8>,

/// Partial-frame accumulation buffer — OUTSTATION-TO-MASTER direction only.
/// Max 292 bytes (MAX_DNP3_FRAME_LEN). Bounded DoS guard.
/// RULING-DNP3-SIBLING-001 §1.3: carry is split per-direction.
carry_s2c: Vec<u8>,
```

Update `on_data` pseudocode in Decision 2 to:
```
let active_carry = match direction {
    ClientToServer => &mut flow.carry_c2s,
    ServerToClient => &mut flow.carry_s2c,
};
```

Update all `wrapping_sub` references in Decision 2 / Decision 4 prose to `saturating_sub`.

Add `modified:` entry: `"RULING-DNP3-SIBLING-001 (2026-06-27): carry split per-direction, Direction threading, saturating_sub window expiry, operator pin >= → >"`.

---

## 7. VP Recommendations

### VP-NEW-C: DNP3 Carry-Direction Isolation Invariant

**Recommendation: ADD as a proptest VP.** Analogous to VP-033 (ENIP).

Property: for any sequence of `on_data` calls with alternating `Direction::ClientToServer`
and `Direction::ServerToClient`, `carry_c2s` is NEVER read during a `ServerToClient` call,
and `carry_s2c` is NEVER read during a `ClientToServer` call. Validated by a proptest
strategy generating random (direction, partial_bytes, full_bytes) sequences and asserting
that per-direction `frame_count` matches independent single-direction control runs.

**Proof method: proptest.** Mirrors VP-033 exactly.

**VP number:** to be assigned by spec-steward. Trace to BC-2.15.016 Invariant 6 (new).

### VP-NEW-D: DNP3 Window Monotonic No-Spurious-Reset

**Recommendation: ADD as a proptest VP.** Analogous to VP-034 (ENIP).

Property: for all three windowed detections (T1692.001 / 60s, T1691.001 / 10s, T0814 /
300s), a single event with `now_ts < window_start_ts` does NOT reset the window. Formally:
if burst of N events has accumulated and one backwards-ts event arrives, then one more
forward-ts event must still trigger the detection.

**Proof method: proptest over (window_start, burst_count, backwards_ts, threshold) triples.**
Mirrors VP-034 exactly.

**VP number:** to be assigned by spec-steward. Traces to BC-2.15.010 PC4 (amended),
BC-2.15.014 PC3 (amended), BC-2.15.015 PC3 (amended).

---

## 8. STORY-140 Acceptance Criteria

**Story title:** "Fix DNP3 DRIFT-DNP3-DIRECTION-001 carry-direction splice + DRIFT-DNP3-CLOCK-001 clock-backwards window reset"

**Scope:** Single atomic story. Implements all changes from §1–§3 of this ruling.

**Acceptance Criteria:**

1. `Dnp3FlowState` has `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`; `carry: Vec<u8>` is removed.
2. `Dnp3Analyzer::on_data` takes a `direction: Direction` parameter (same import as STORY-139: `crate::reassembly::handler::Direction`); the frame-walk loop selects `carry_c2s` or `carry_s2c` by direction.
3. `resolve_master_ip` is replaced with direction-aware source resolution (DRIFT-DNP3-DIRECTION-001 fix-along — Modbus pattern at `modbus.rs` ~355-382).
4. All `on_data` call sites for `Dnp3Analyzer` in `src/dispatcher.rs` pass `direction`.
5. All windowed timestamp comparisons in `dnp3.rs` use `saturating_sub` (not `wrapping_sub`): lines 745, 765, 769, 895, 984, 1025, 1335, 1341.
6. The 300s correlation-window expiry check (line 984) uses `> CORRELATION_WINDOW_SECS` (strict greater-than, not `>=`).
7. All existing DNP3 tests pass with the new `on_data` signature.
8. **New regression test (carry direction isolation):** Deliver partial master-to-outstation frame (bytes that do NOT form a complete frame), then deliver a complete outstation-to-master frame with a valid FIR=1 Response (FC=0x81). Assert: `frame_count == 1`, `parse_errors == 0`, the partial c2s frame remains in `carry_c2s`, `carry_s2c` is empty after the s2c consume. This is the EC-X1 analog for DNP3.
9. **New regression test (backwards-clock window no-reset):** Deliver 9 Control-class FCs at ts=100 (`window_start_ts=100`, `direct_operate_count=9`), then 1 FC at ts=50 (backwards), then 2 FCs at ts=100 again. Assert: `direct_operate_count=12`, T1692.001 fires at count=11. This is the EC-X2 analog for DNP3.
10. **New regression test (correlation-window backwards-clock):** Deliver 2 COLD_RESTART events at ts=0 and ts=150 (`restart_event_count=2`, `correlation_window_start_ts=0`), then one event at ts=50 (backwards), then one COLD_RESTART at ts=200. Assert: `restart_event_count=3` at ts=200 (window NOT reset by the backwards-ts call), T0827 fires.
11. VP-NEW-C proptest (carry direction isolation) is drafted in the same story.
12. VP-NEW-D proptest (window monotonic no-spurious-reset) is drafted in the same story.

---

## 9. Spec-Steward Actions Required

The following BCs are amended by this ruling and require version bumps:

| BC | Old version | New version | Input-hash impact |
|----|------------|------------|-------------------|
| BC-2.15.016 | 1.6 | 2.0 | STALE on version bump |
| BC-2.15.010 | 1.7 | 1.8 | STALE on version bump |
| BC-2.15.014 | 2.0 | 2.1 | STALE on version bump |
| BC-2.15.015 | 1.9 | 2.0 | STALE on version bump |

**Actions:**

1. Bump versions in frontmatter of all four BCs.
2. Add `modified:` entry to each BC frontmatter citing this ruling.
3. Run `bin/compute-input-hash --scan` after BC edits. Any story whose `inputs:` list
   includes any of the four BCs above will report STALE and must be regenerated before
   Phase 4 entry.
4. Assign VP numbers for VP-NEW-C and VP-NEW-D and register in VP-INDEX.md.
5. Update VP-INDEX counts and propagate to `verification-architecture.md` and
   `verification-coverage-matrix.md` per the VP-index propagation obligation.
6. Add this ruling to `decisions-archive.md` under the `feature-enip-v0.11.0` cycle.
7. ADR-007 updated in place per §6 above.

---

## 10. Summary for Test-Writer and Implementer

**Implementer (carry fix — DRIFT-DNP3-DIRECTION-001):**
- Replace `Dnp3FlowState.carry: Vec<u8>` (line 197) with `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`.
- Add `direction: Direction` to `on_data` (line 313). Import: `crate::reassembly::handler::Direction`.
- In the frame-walk: `let active_carry = if direction == Direction::ClientToServer { &mut flow.carry_c2s } else { &mut flow.carry_s2c };`
- Use `active_carry` for all carry operations: accumulate, cap-check, inline-resync, frame-walk, drain.
- Replace `resolve_master_ip` heuristic with direction-based IP selection (Modbus pattern).
- Update all `Dnp3Analyzer::on_data` call sites in `src/dispatcher.rs`.

**Implementer (window fix — DRIFT-DNP3-CLOCK-001 + DRIFT-DNP3-OP-001):**
- Replace every `now_ts.wrapping_sub(...)` at lines 745, 765, 769, 895, 984, 1025, 1335, 1341 with `now_ts.saturating_sub(...)`.
- Change `>=` at line 984 to `>`.
- No other logic changes.

**Test-writer:**
- Write AC-8 regression test (carry direction isolation — DNP3 EC-X1 analog).
- Write AC-9 regression test (60s window backwards-clock — T1692.001).
- Write AC-10 regression test (300s window backwards-clock — T0827).
- Write VP-NEW-C proptest: random (direction, partial_bytes) sequences; assert carry isolation.
- Write VP-NEW-D proptest: random (window_start, burst_count, backwards_ts, threshold); assert no spurious reset.
