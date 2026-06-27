---
document_type: arch-ruling
ruling_id: RULING-EDGECASE-001
cycle: feature-enip-v0.11.0
author: architect
date: 2026-06-27
status: authoritative
supersedes: []
addendum_to: null
bcs_amended:
  - BC-2.17.016
  - BC-2.17.008
  - BC-2.17.012
  - BC-2.17.018
adrs_amended:
  - ADR-010
vps_recommended:
  - VP-NEW-A: carry-direction-isolation (EC-X1)
  - VP-NEW-B: window-monotonic-no-spurious-reset (EC-X2)
release_blocker: true
release_held: v0.11.0
---

# RULING-EDGECASE-001: Direction Carry Splice (EC-X1) and Clock-Backwards Window Reset (EC-X2)

## 0. Executive Summary

Two empirically-confirmed HIGH-severity bugs found in the post-convergence edge-case hunt
block v0.11.0 release. Both are confirmed via test-vs-control evidence in
`tests/scratch_ecx1_ecx2_repro.rs`. This ruling adjudicates:

- **EC-X1** — cross-direction carry-buffer splice: a partial c2s frame stashed in
  `EnipFlowState.carry` corrupts the next s2c delivery, producing phantom findings and
  suppressing legitimate error detections.
- **EC-X2** — clock-backwards / out-of-order timestamp window reset: `wrapping_sub` on a
  backwards timestamp wraps to ~4.29e9, triggering a false window reset that discards a
  burst-in-progress, suppressing T0836, T0888, or T0814.
- **EC-X3** — BC-2.17.016 EC-003/EC-006 state an unreachable 600-byte carry size.
  Corrected as a spec-errata.
- **EC-X4** — operator inconsistency: malformed window uses `>= 300`, error/write windows
  use `> threshold`. Pinned to a single consistent operator.

Both EC-X1 and EC-X2 are **release blockers**. v0.11.0 is held until both are fixed and
verified. EC-X3 and EC-X4 fold into the same fix burst.

---

## 1. EC-X1 — Cross-Direction Carry-Buffer Splice

### 1.1 Root Cause

`FlowKey` canonicalizes both TCP directions to a single key (VP-001, verified and
immutable). `EnipFlowState` owns ONE `carry: Vec<u8>`. `on_data(flow_key, data, ts)` takes
no `Direction` parameter (DRIFT-ENIP-DIRECTION-001, documented in `enip.rs`). Consequently
both TCP directions share the same carry buffer. A partial client-to-server frame stashed
into carry is prepended to the next server-to-client delivery:

```
buf = carry(c2s_partial_header) ++ s2c_response_bytes
```

The frame-walk loop reads carry[0..24] as the ENIP header for the "frame". It declares
`total_frame_len = 24 + declared_c2s_payload`. If `buf.len() >= total_frame_len`, the loop
commits the splice as a valid PDU — dispatching a request header against response bytes.
If `buf.len() < total_frame_len`, the response is stashed again, delaying or losing it.

The repro confirms: `+1` spurious T0858 finding, `-1` missed `error_count`, `+3` spurious
`parse_errors` on a fresh bidirectional flow with one segment boundary crossing.

### 1.2 Decision: Option (b) — Two Carry Buffers per FlowState

**Adopted: Option (b).** Replace `EnipFlowState.carry: Vec<u8>` with two separate
carry buffers: `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`. Thread `Direction` into
`on_data`. Select the carry buffer by direction on every call.

**Why not Option (a) — full per-direction sub-state keyed by (FlowKey, Direction)?**
Option (a) would replace `HashMap<FlowKey, EnipFlowState>` with
`HashMap<(FlowKey, Direction), EnipFlowState>`, duplicating all windowed state per
direction. That is architecturally cleaner for source-IP resolution and detection
attribution, but it is a much larger change: every detection BC would need direction-aware
rewording, window semantics would bifurcate (which direction owns the T0836 count?), and
the `summarize()` drainage would need to merge two half-states. That is a v0.12.0 scope
item, not a hotfix for a confirmed bug.

Option (b) is the **minimal correct fix**: it isolates only the state that is genuinely
per-direction (carry), while leaving all detection counters as bidirectional per-flow
aggregates. ENIP request commands (c2s) and response status (s2c) are structurally
separate — the carry splice corrupts them; fixing the carry isolation is sufficient to
restore correct behavior without rearchitecting detection logic.

**Why not Option (c) — other?** No viable alternative identified. The only other approach
(direction-less disambiguation by frame inspection) cannot work because the spliced buffer
is syntactically valid to the parser — the request header's `is_valid_enip_frame` check
passes even when the declared payload body is response bytes.

### 1.3 Exact Per-Direction State Classification

| State field | Classification | Rationale |
|-------------|---------------|-----------|
| `carry_c2s: Vec<u8>` | PER-DIRECTION (new) | Carry is the byte-level partial-frame accumulation for ONE direction's TCP stream. A partial c2s frame is meaningless in the s2c stream. |
| `carry_s2c: Vec<u8>` | PER-DIRECTION (new) | Symmetric. |
| `is_non_enip: bool` | PER-FLOW (keep shared) | Stream-level desync; if the flow is not ENIP, both directions are not ENIP. A per-direction latch would require both to fire before the flow is abandoned, which is incorrect. |
| `command_counts: HashMap<u16, u64>` | PER-FLOW (keep shared) | Commands are aggregated across the flow for detection and reporting. BC-2.17.004 Inv-3 and BC-2.17.016 PC-0 both specify flow-level aggregation. |
| `parse_errors: u64` | PER-FLOW (keep shared) | Lifetime parse-error counter for the flow; not direction-specific for reporting purposes. |
| `malformed_in_window: u64` | PER-FLOW (keep shared) | T0814 fires on structural anomalies per flow; direction attribution is not required. |
| `malformed_anomaly_emitted: bool` | PER-FLOW (keep shared) | One-shot guard is per-window, not per-direction. |
| `malformed_window_start: u32` | PER-FLOW (keep shared) | Window is flow-level. |
| `pdu_count: u64` | PER-FLOW (keep shared) | Flow-level aggregate; BC-2.17.024 is flow-scoped. |
| `write_count_in_window: u64` | PER-FLOW (keep shared) | T0836 write burst is a flow-level detection. Write requests are c2s; responses s2c. Since we only count write-class SERVICE REQUESTS (BC-2.17.012 Precondition 2: `service & 0x80 == 0`), the existing direction filtering in the CIP service gate is sufficient. |
| `write_window_start_ts: u32` | PER-FLOW (keep shared) | Window is flow-level. |
| `write_burst_emitted: bool` | PER-FLOW (keep shared) | One-shot guard. |
| `error_counts_in_window: HashMap<u8, u64>` | PER-FLOW (keep shared) | T0888 counts response errors (s2c). Since the CIP response gate (`service & 0x80 != 0`) already discriminates direction at the protocol level, per-direction carry isolation is sufficient; the counter itself need not split. |
| `error_window_start_ts: u32` | PER-FLOW (keep shared) | Window is flow-level. |
| `error_window_active: bool` | PER-FLOW (keep shared) | Flow-level sentinel. |
| `error_rate_emitted: bool` | PER-FLOW (keep shared) | One-shot guard. |
| `list_identity_emitted: bool` | PER-FLOW (keep shared) | One-shot guard, flow-level. |

**Summary of the structural change:** `EnipFlowState.carry: Vec<u8>` is removed and
replaced with `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`. All other fields stay
per-flow. `on_data` receives an additional `direction: Direction` parameter and uses it
ONLY to select which carry buffer to prepend and which to stash into.

### 1.4 DRIFT-ENIP-DIRECTION-001 Fix-Along

Threading `Direction` into `on_data` as required by option (b) also enables fixing
DRIFT-ENIP-DIRECTION-001 (source IP resolution). Once `Direction` is available,
`resolve_enip_client_ip` can be replaced with a direction-aware resolution identical to
the Modbus pattern:

```
let src_ip = match direction {
    Direction::ClientToServer => /* client (non-44818 port) side */,
    Direction::ServerToClient => /* server (44818 port) side */,
};
```

This is a no-cost fix-along: the parameter is already being threaded. It MUST be included
in the same story as the carry split to avoid DRIFT-ENIP-DIRECTION-001 persisting after
the fix.

### 1.5 on_data Signature Change

**Before:**
```rust
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], timestamp: u32)
```

**After:**
```rust
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], timestamp: u32, direction: Direction)
```

The `Direction` type lives at `crate::reassembly::handler::Direction`. It is already
imported in `modbus.rs` for the same purpose. All call sites in `src/dispatcher.rs`
must be updated to pass `Direction` from the `StreamHandler`
context — matching exactly the Modbus pattern.

### 1.6 Sibling Sweep (DF-SIBLING-SWEEP-001)

DNP3 (`Dnp3Analyzer::on_data`) carries the same Direction-dropped, single-carry pattern
(DRIFT-DNP3-DIRECTION-001, documented in `dnp3.rs`). Modbus already has direction threading
and is NOT affected.

**Ruling: fix ENIP now; track DNP3 as a sibling follow-up chore for v0.12.0.**

Rationale: DNP3's carry-direction splice risk is lower because DNP3's application layer
parses FIR=1 frames only (ADR-007 Decision 3). A partial DNP3 request frame in carry that
gets prepended to a DNP3 response is unlikely to pass the FIR=1 gate with a meaningfully
wrong application payload, making the detection impact smaller. The DNP3 sibling fix is
still required for correctness but is NOT a v0.11.0 blocker.

**Human decision flag:** The scope call "fix DNP3 in v0.12.0" is a product-level decision.
This ruling recommends it but a human MUST confirm it before the v0.11.0 release notes
characterize the scope. If the human decides DNP3 must be fixed atomically with ENIP,
the fix story scope expands to include `Dnp3FlowState.carry` → `carry_c2s` / `carry_s2c`
with the same structural change.

---

## 2. EC-X2 — Clock-Backwards / Out-of-Order Timestamp Window Reset

### 2.1 Root Cause

All three windowed detections evaluate window expiry using `now_ts.wrapping_sub(window_start_ts)`:

| BC | Window | Operator | Condition |
|----|--------|----------|-----------|
| BC-2.17.012 (T0836) | 1 second | `> 1` | `timestamp.wrapping_sub(write_window_start_ts) > 1` |
| BC-2.17.008 (T0888) | 10 seconds | `> 10` | `timestamp.wrapping_sub(error_window_start_ts) > 10` |
| BC-2.17.018 (T0814) | 300 seconds | `>= 300` | `timestamp.wrapping_sub(malformed_window_start) >= 300` |

`wrapping_sub` was introduced to handle genuine u32 rollover (pcap timestamps wrap after
~136 years of uptime, or more practically, on relative-second-based captures that wrap
within a pcap's duration). However, when `now_ts < window_start_ts` due to packet
reordering, capture jitter, or adversarial injection, `wrapping_sub` wraps to ~4.29 billion,
which is >> any threshold. The window resets spuriously, discarding the burst accumulated
so far. The repro confirms: 50 writes at ts=100 then one write at ts=50 → `write_count_in_window`
reset to 1 → T0836 SUPPRESSED.

### 2.2 Decision: Option (a) — `saturating_sub`

**Adopted: Option (a) — replace `wrapping_sub` with `saturating_sub` for all window expiry
comparisons.**

**Behavior under saturating_sub:**

| Scenario | `saturating_sub` result | Window action |
|----------|------------------------|---------------|
| `now_ts = 200`, `window_start = 100` (forward, in-window) | `100` | No reset if `100 <= threshold` |
| `now_ts = 150`, `window_start = 100` (forward, expired) | `50` | Reset if `50 > threshold` (e.g. T0836: 50 > 1 → reset) |
| `now_ts = 50`, `window_start = 100` (BACKWARDS) | `0` | No reset (0 <= any positive threshold); packet treated as in-window |
| Genuine u32 rollover: `now_ts = 4` (post-wrap), `window_start = u32::MAX - 10` | `0` | No reset |

**Why saturating_sub is correct for both cases:**

- **Backwards clock:** `saturating_sub` returns 0, meaning elapsed = 0 seconds. The burst
  accumulation is preserved. A backwards-clock packet is treated as arriving at the same
  instant as the window start — conservative but correct. An adversary injecting a single
  old-timestamp packet can no longer abort a burst detection window.
- **Genuine u32 rollover:** `saturating_sub` also returns 0. This means a genuine rollover
  event does NOT trigger a window reset either. The window will persist until a monotonically
  advancing timestamp eventually expires it. This is acceptable because genuine u32 rollover
  on pcap-relative second timestamps is an extremely rare edge case (requires ~136 years of
  monotonic uptime or a capture restart that doesn't re-zero the base). The security cost of
  failing to reset on genuine rollover is negligible relative to the security cost of the
  adversarially-triggered false reset.

**Why not option (b) — clamp (ignore events where `now_ts < window_start`):**
Option (b) silently drops the triggering event's counter contribution. In the T0836 case,
a backwards write at ts=50 during a burst would be ignored entirely (counter not incremented).
This is slightly less correct than treating it as in-window (option a), and it requires
adding a guard branch before every counter increment, making the logic more complex.

**Why not option (c) — track max-seen-ts per window:**
Option (c) correctly handles the general out-of-order case but requires a `max_seen_ts`
field on `EnipFlowState` for each window — three new fields. The behavioral benefit over
option (a) is marginal for the actual attack scenario (adversary injects one old-timestamp
packet to defeat detection; option (a) defeats this). Rejected for simplicity.

### 2.3 Genuine u32 Rollover vs. Backwards Clock: Are They Distinguishable?

Under `saturating_sub` they are NOT distinguishable, and they do not need to be: both
produce 0 elapsed, both preserve the window. This is the correct behavior. The former
`wrapping_sub` attempted to handle rollover but inadvertently created the backwards-clock
evasion. `saturating_sub` trades the theoretical rollover-reset for protection against
backwards-clock evasion — which is the operationally relevant threat.

### 2.4 Operator Pinning (EC-X4 resolution)

The current code shows three different patterns:

| BC | Current operator | Source line |
|----|-----------------|-------------|
| BC-2.17.012 write window | `> 1` | `enip.rs:1312` |
| BC-2.17.008 error window | `> 10` | `enip.rs:1129` |
| BC-2.17.018 malformed window | `>= 300` | `enip.rs:821` |

The `>=` on the malformed window and `>` on the others is inconsistent. The semantic
difference is whether the packet arriving EXACTLY at the threshold second is:
- `> N`: the packet at elapsed=N is still IN the window; expiry fires at elapsed=N+1.
- `>= N`: the packet at elapsed=N triggers the reset; it starts a new window.

**Ruling: pin ALL three windows to strict `>`.**

Rationale: `> threshold` means the window expires when the elapsed time EXCEEDS the
threshold — the packet at exactly `elapsed == threshold` is treated as the last packet
of the current window, not the first packet of a new one. This is the more intuitive
semantic for burst detection (a burst of writes in exactly 1 second does not split across
two windows) and is already the rule for T0836 and T0888. The T0814 `>= 300` is a spec
defect that diverges from the established pattern without documented justification. It
is corrected here to `> 300` to match the other windows.

**Note:** this is a very minor behavioral change for T0814 (one second of tolerance at the
boundary). It is folded into the EC-X2 fix with no separate story.

### 2.5 Sibling Sweep

DNP3's windowed detection (`dnp3.rs`) uses the same `wrapping_sub` pattern
(DRIFT-DNP3-DIRECTION-001 co-found with DRIFT-DNP3-CLOCK-001, to be catalogued). Same
scope ruling as EC-X1: fix ENIP now, DNP3 in v0.12.0. Human must confirm scope.

---

## 3. EC-X3 — BC-2.17.016 Carry-Size Edge Cases Are Unreachable (Spec Errata)

### 3.1 Finding

BC-2.17.016 EC-003 and EC-006 describe states where `carry.len() == 600` (exactly
600 bytes) and `header.length = 576` (giving total_frame_len = 600). These edge cases
were written as if 600-byte carry is a normal reachable state, but RULING-137-002
proved that the carry-cap check can only fire on a stash that would EXCEED 600 bytes
(i.e., the stash would be >= 601 bytes). A stash of exactly 600 bytes does NOT exceed
the cap; the flow is not latched `is_non_enip`.

The spec errata is:
- EC-003 ("Carry grows to exactly 600 bytes — Cap not yet exceeded") is correct.
- EC-006 ("Large ENIP payload with header.length=576 fills carry exactly to 600") is
  technically reachable but practically very unlikely: the partial frame of exactly
  total_frame_len bytes would have to be delivered with carry=0 and a payload that
  fills exactly 576 bytes. More importantly, EC-006's description says it "fills carry
  exactly" — this is fine; it does NOT exceed the cap. The edge case itself is correct;
  the accompanying canonical test vector claims `header.length = 600 - 24 = 576` which is
  accurate. NO CORRECTION NEEDED for EC-006 itself.

What IS wrong: the canonical test vector table in BC-2.17.016 row 2 states
"Complete frame (600 bytes total, header.length=576)" which is valid — a frame of exactly
600 bytes with header.length=576 is a complete parseable frame. The carry after is 0
(not stashed, because the frame is complete). This is correctly specified.

**The actual unreachability (from RULING-137-002) applies to the OVERFLOW path (EC-004,
EC-008), not to EC-003/EC-006.** On re-read, EC-003 and EC-006 do NOT claim overflow
fires at 600 — only that the cap is "not yet exceeded". EC-003 is therefore correct.

**Correction: EC-006 description uses `header.length = 600 - 24 = 576`; this is the
MAXIMUM stashable partial-frame payload (576 payload bytes + 24 header = 600 total, which
equals the cap but does not exceed it). This is a REACHABLE state. The "unreachable"
characterization in the task prompt was incorrect — EC-003/EC-006 are fine.**

However, the CANONICAL TEST VECTOR row "Complete frame (600 bytes total, header.length=576)"
IS reachable and correct. The issue is EC-004 says "Carry would grow to 601 bytes →
`is_non_enip=true`" which requires a partial HEADER + data combination that exceeds 600
bytes — which, per RULING-137-002, requires carrying more than 600 bytes of a partial
frame. Per the proof, this CAN happen via Path A (loop never enters because buf.len() < 24)
when accumulated carry + new_data grows past 600. So EC-004 is reached via the
"buf < 24, stash all of buf" path. RULING-137-002 said the overflow is reachable via
repeated sub-24-byte deliveries accumulating carry — and that assertion WAS confirmed as
correct by the proof (the claim that was retracted was only the claim that Path B could
overflow). The EC-004 overflow IS reachable; RULING-137-002 clarified only that Path B
(frame-skip) cannot trigger it.

**Conclusion for EC-X3: No BC amendment is needed. The edge-case descriptions are
accurate. The "unreachable 600-byte carry" characterization in the task prompt was
imprecise; what is unreachable is the OVERFLOW path via the frame-skip sub-path, not
the 600-byte stash value itself. This is already correctly documented in RULING-137-002.
EC-X3 is a spec-clarity non-issue, not a defect.**

---

## 4. Binding BC and ADR Amendments

### 4.1 ADR-010 Decision 4 Amendment

In the `EnipFlowState` struct listing (Decision 4), replace:

```rust
/// Partial ENIP frame accumulation buffer.
/// Max 600 bytes (MAX_ENIP_CARRY_BYTES). Bounded DoS guard.
carry: Vec<u8>,
```

with:

```rust
/// Partial ENIP frame accumulation buffer — CLIENT-TO-SERVER direction only.
/// Receives bytes from the TCP initiator (Modbus client, or device sending to port 44818).
/// Max 600 bytes (MAX_ENIP_CARRY_BYTES). Bounded DoS guard.
/// RULING-EDGECASE-001 §1.3: carry is split per-direction to prevent cross-direction splice.
carry_c2s: Vec<u8>,

/// Partial ENIP frame accumulation buffer — SERVER-TO-CLIENT direction only.
/// Receives bytes from the TCP responder (EtherNet/IP device, port 44818 side).
/// Max 600 bytes (MAX_ENIP_CARRY_BYTES). Bounded DoS guard.
/// RULING-EDGECASE-001 §1.3: carry is split per-direction to prevent cross-direction splice.
carry_s2c: Vec<u8>,
```

In the `on_data()` frame-walk pseudocode (Decision 4), update:

```
let buf = carry ++ new_data
```
to:
```
let carry = match direction {
    ClientToServer => &flow.carry_c2s,
    ServerToClient => &flow.carry_s2c,
};
let buf = carry ++ new_data
```

And at stash-back:
```
match direction {
    ClientToServer => flow.carry_c2s = buf[cursor..],
    ServerToClient => flow.carry_s2c = buf[cursor..],
}
```

The cap check applies to whichever carry buffer was just written:
```
let active_carry_len = match direction {
    ClientToServer => flow.carry_c2s.len(),
    ServerToClient => flow.carry_s2c.len(),
};
if active_carry_len > MAX_ENIP_CARRY_BYTES {
    flow.is_non_enip = true;
    flow.parse_errors += 1;
    // clear the overflowed carry
}
```

Also update the `on_data()` signature:
```rust
// BEFORE (Decision 4 original)
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], timestamp: u32)

// AFTER (RULING-EDGECASE-001 amendment)
pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], timestamp: u32, direction: Direction)
```

In the window expiry comparisons (Decision 4 prose and pseudocode), replace all
instances of `wrapping_sub` with `saturating_sub`:
- `now_ts.wrapping_sub(flow.write_window_start_ts) > 1` → `now_ts.saturating_sub(flow.write_window_start_ts) > 1`
- `now_ts.wrapping_sub(flow.error_window_start_ts) > 10` → `now_ts.saturating_sub(flow.error_window_start_ts) > 10`
- `timestamp.wrapping_sub(flow.malformed_window_start) >= 300` → `timestamp.saturating_sub(flow.malformed_window_start) > 300`
  (note: `>=` changed to `>` per EC-X4 operator pinning, §2.4 above)

### 4.2 BC-2.17.016 Amendments

**Version bump: 1.0 → 2.0** (breaking structural change to EnipFlowState).

1. **Precondition 2 (carry prepend):** Change from:
   > `buf = flow.carry ++ data`

   to:
   > `buf = (match direction { ClientToServer => flow.carry_c2s, ServerToClient => flow.carry_s2c }) ++ data`

2. **Postcondition 3 (after-loop stash):** Change from:
   > `flow.carry = buf[cursor..]`

   to:
   > `match direction { ClientToServer => flow.carry_c2s = buf[cursor..], ServerToClient => flow.carry_s2c = buf[cursor..] }`

3. **Postcondition 4 (cap check):** Update to reference `carry_c2s` / `carry_s2c` (whichever was just written) rather than `carry`.

4. **Invariant 1 (carry is bounded):** Change from:
   > `flow.carry.len() <= MAX_ENIP_CARRY_BYTES = 600`

   to:
   > `flow.carry_c2s.len() <= MAX_ENIP_CARRY_BYTES = 600` AND `flow.carry_s2c.len() <= MAX_ENIP_CARRY_BYTES = 600`

5. **Architecture Anchors:** Replace `EnipFlowState.carry: Vec<u8>` with:
   - `EnipFlowState.carry_c2s: Vec<u8>`
   - `EnipFlowState.carry_s2c: Vec<u8>`

6. **Add new Invariant 7 (direction isolation):**
   > `carry_c2s` and `carry_s2c` are NEVER mixed. `on_data` selects exactly one of the
   > two buffers based on the `direction` argument on every call. No frame-walk loop ever
   > prepends bytes from one direction into the other. This invariant prevents the
   > cross-direction splice bug documented in EC-X1 (RULING-EDGECASE-001).

7. **Add new Edge Case EC-010 (direction isolation confirmation):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-010 | Partial c2s frame stashed in carry_c2s; next call is s2c direction | `carry_s2c` is prepended to s2c data (carry_c2s NOT involved); s2c frame processes cleanly; carry_c2s retains partial c2s bytes |

8. **on_data signature update:** reflect new `direction: Direction` parameter in Precondition 1.

### 4.3 BC-2.17.008 Amendments

**Version bump: 1.2 → 1.3** (window operator fix, minor).

1. **Postcondition 4 (window expiry):** Change:
   > `now_ts.wrapping_sub(flow.error_window_start_ts) > 10`

   to:
   > `now_ts.saturating_sub(flow.error_window_start_ts) > 10`

2. **Add new Edge Case EC-009 (backwards timestamp):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-009 | Error responses at ts=100, then one response at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; elapsed=0, NOT > 10 → window NOT reset; burst accumulation preserved |

### 4.4 BC-2.17.012 Amendments

**Version bump: 1.1 → 1.2** (window operator fix, minor).

1. **Postcondition 4 (window expiry):** Change:
   > `now_ts.wrapping_sub(flow.write_window_start_ts) > 1`

   to:
   > `now_ts.saturating_sub(flow.write_window_start_ts) > 1`

2. **Add new Edge Case EC-009 (backwards timestamp):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-009 | 50 writes at ts=100 (window_start=100), then 1 write at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; elapsed=0, NOT > 1 → window NOT reset; write_count_in_window=51; T0836 fires |

### 4.5 BC-2.17.018 Amendments

**Version bump: 1.0 → 1.1** (window operator fix, `>=` → `>`, plus backwards-clock).

1. **Postcondition 5 (window expiry):** Change:
   > `timestamp.wrapping_sub(flow.malformed_window_start) >= 300`

   to:
   > `timestamp.saturating_sub(flow.malformed_window_start) > 300`

   Note: both the `wrapping` → `saturating` change AND the `>=` → `>` operator pin
   are applied here in the same version bump.

2. **Invariant 2 (window text):** Update to read "Expiry fires when elapsed seconds > 300
   (strict greater-than, consistent with T0836 and T0888 window semantics)."

3. **Add new Edge Case EC-008 (backwards timestamp):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-008 | 2 malformed frames at ts=100 (malformed_in_window=2), then 1 frame at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; NOT > 300 → window NOT reset; malformed_in_window=3; T0814 fires |

---

## 5. VP Recommendations

### VP-NEW-A: Carry-Direction Isolation Invariant

**Recommendation: ADD as a unit-test VP (no Kani needed).**

Property: for any sequence of `on_data` calls with alternating `Direction::ClientToServer`
and `Direction::ServerToClient`, `carry_c2s` is NEVER read during a `ServerToClient` call,
and `carry_s2c` is NEVER read during a `ClientToServer` call. This can be proven by
inspection of the match arm (pure structural isolation), or by a property-based test
(fast-check / proptest) that drives alternating-direction calls with partial frames and
asserts per-direction PDU counts match independent same-direction control runs.

**Proof method: proptest.** Kani model-checking of the carry selection is possible but
overkill — the invariant is trivially enforced by the match arm. A proptest strategy
that generates random partial-frame sequences for both directions and verifies count
isolation is sufficient and matches the repro methodology.

**VP number:** to be assigned by spec-steward. Trace to BC-2.17.016 Invariant 7 (new).

### VP-NEW-B: Window Monotonic No-Spurious-Reset

**Recommendation: ADD as a proptest VP.**

Property: for all three windowed detections (T0836, T0888, T0814), a single event with
`now_ts < window_start_ts` does NOT reset the window. Formally: if
`window_start = T` and a burst of N (threshold) events has been accumulated, then
delivering one event with `ts < T` followed by one event with `ts >= T` must still
produce a detection (assuming N+1 accumulated events exceeds threshold). This is exactly
the EC-X2 repro scenario.

**Proof method: proptest over (window_start, backwards_ts, threshold) triples.** Kani
could prove `saturating_sub(backwards_ts, window_start) == 0` when
`backwards_ts <= window_start` (pure arithmetic, feasible), but the proptest over the
full window state transition is more directly tied to the behavioral invariant.

**VP number:** to be assigned by spec-steward. Traces to BC-2.17.008, BC-2.17.012,
BC-2.17.018 (window expiry postconditions as amended).

---

## 6. Release Blocker Confirmation

**EC-X1 — CONFIRMED RELEASE BLOCKER.** The cross-direction carry splice produces both
phantom findings (false positives, T0858 in the repro) and missed detections (false
negatives, T0888-B error count) on any TCP flow that experiences a segment boundary at
a direction-crossing point. This is a common TCP condition. False-positive T0858 findings
represent a severe detection quality regression; false-negative T0888-B is a detection gap.
v0.11.0 MUST NOT ship with this bug.

**EC-X2 — CONFIRMED RELEASE BLOCKER.** The backwards-clock window reset is an adversarial
evasion path: a single injected out-of-order packet defeats burst detection for T0836,
T0888, and T0814. This violates the threat-detection intent of these BCs at the
protocol level. Any ICS adversary with the ability to inject a single rogue packet with
a stale timestamp can suppress all burst-based ENIP detections. v0.11.0 MUST NOT ship
with this bug.

**EC-X3 — NOT a blocker.** Spec-clarity non-issue per §3 above.

**EC-X4 — NOT a standalone blocker.** Operator inconsistency folded into EC-X2 fix.
No incremental risk if corrected atomically with EC-X2.

---

## 7. Fix Story Specification

One new story is warranted. It covers EC-X1, EC-X2, EC-X4, and the DRIFT-ENIP-DIRECTION-001
fix-along as a single atomic burst.

**Story title:** "Fix EC-X1 carry-direction splice + EC-X2 clock-backwards window reset"

**Acceptance criteria:**
1. `EnipFlowState` has `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`; `carry: Vec<u8>` is
   removed.
2. `on_data` takes a `direction: Direction` parameter; the frame-walk loop selects
   `carry_c2s` or `carry_s2c` by direction.
3. `resolve_enip_client_ip` is replaced with direction-aware source resolution
   (DRIFT-ENIP-DIRECTION-001 fix-along).
4. All three window-expiry comparisons use `saturating_sub` (not `wrapping_sub`).
5. The malformed window uses `> 300` (not `>= 300`).
6. All existing ENIP tests pass with the new `on_data` signature.
7. New regression test: deliver partial c2s frame, then full s2c error-response frame;
   assert `error_count == 1`, `pdu_count == 1`, `parse_errors == 0`, `findings == 0`
   (threshold not crossed). This is the EC-X1 control/test pair formalized.
8. New regression test: 50 writes at ts=100, 1 write at ts=50, 1 write at ts=100;
   assert T0836 fires (write_count_in_window = 51 > 50). This is the EC-X2 control pair.
9. VP-NEW-A proptest (carry direction isolation) is drafted in the same story.
10. VP-NEW-B proptest (window monotonic no-spurious-reset) is drafted in the same story.

---

## 8. Spec-Steward Version Bump and Input-Hash Implications

The following BCs are amended by this ruling and require version bumps:

| BC | Old version | New version | Stories that consume it | Input-hash impact |
|----|------------|------------|------------------------|-------------------|
| BC-2.17.016 | 1.0 | 2.0 | All STORY-NNN that list BC-2.17.016 in `inputs:` | STALE on version bump |
| BC-2.17.008 | 1.2 | 1.3 | All stories listing BC-2.17.008 in `inputs:` | STALE on version bump |
| BC-2.17.012 | 1.1 | 1.2 | All stories listing BC-2.17.012 in `inputs:` | STALE on version bump |
| BC-2.17.018 | 1.0 | 1.1 | All stories listing BC-2.17.018 in `inputs:` | STALE on version bump |

**Spec-steward actions required:**

1. Bump versions in frontmatter of all four BCs per the table above.
2. Add a `modified:` entry to each BC frontmatter citing this ruling.
3. Run `bin/compute-input-hash --scan` after BC edits. Any story whose `inputs:` list
   includes BC-2.17.016, BC-2.17.008, BC-2.17.012, or BC-2.17.018 will report STALE.
   Those stories MUST be regenerated (or their `input-hash` manually updated) before
   proceeding to Phase 4. This is the Phase-4 entry gate per CLAUDE.md.
4. Assign VP numbers for VP-NEW-A and VP-NEW-B and register them in VP-INDEX.md.
5. Update VP-INDEX counts (new VPs added) and propagate to `verification-architecture.md`
   and `verification-coverage-matrix.md` per the VP-index propagation obligation.
6. Add this ruling to `decisions-archive.md` under the `feature-enip-v0.11.0` cycle.

ADR-010 is amended in place (Decision 4). The ADR frontmatter `modified:` list should be
updated with `"RULING-EDGECASE-001 (2026-06-27): Decision 4 carry split, Direction thread,
saturating_sub window expiry"`.

---

## 9. Summary for Test-Writer and Implementer

**Implementer (carry fix — EC-X1):**
- Replace `EnipFlowState.carry: Vec<u8>` with `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`.
- Add `direction: Direction` to `on_data` signature. Use `crate::reassembly::handler::Direction`.
- In the frame-walk loop: `let carry = if direction == Direction::ClientToServer { &mut flow.carry_c2s } else { &mut flow.carry_s2c };`
- Build `buf = carry.clone() ++ data` (same logic, different source buffer).
- After loop, stash back into the same directional carry.
- Cap check applies to the directional carry.
- Replace `resolve_enip_client_ip` with direct `direction`-based IP selection (Modbus pattern).
- Update all `on_data` call sites in stream dispatcher to pass direction.

**Implementer (window fix — EC-X2 + EC-X4):**
- Replace every `now_ts.wrapping_sub(...)` in `on_data` and `process_pdu` with `now_ts.saturating_sub(...)`.
- Change `>= 300` in the malformed window check to `> 300`.
- No other logic changes.

**Test-writer:**
- Formalize the EC-X1 repro test from `scratch_ecx1_ecx2_repro.rs` into a proper named
  test in the ENIP test suite (not scratch). It MUST be a RED test against the unfixed code
  and GREEN after the fix.
- Formalize the EC-X2 T0836 repro test similarly.
- Write VP-NEW-A proptest: generate random (direction, partial_bytes, full_bytes) triples;
  assert that a c2s partial stash never contaminates an s2c delivery's PDU count.
- Write VP-NEW-B proptest: generate random (window_start, burst_count, backwards_ts, threshold)
  tuples satisfying `burst_count == threshold`; assert detection fires after the backwards-ts
  packet + one more forward-ts packet.
