---
document_type: design-proposal
title: "Cross-Direction Shared Per-Flow State: Systemic Pattern Review"
cycle: feature-enip-v0.11.0
author: architect
date: 2026-06-28
status: proposal
informs: v0.12.0-planning, modbus-ec-x1-sibling-fix
related_rulings:
  - RULING-EDGECASE-001
  - RULING-DNP3-SIBLING-001
---

# DESIGN PROPOSAL: Cross-Direction Shared Per-Flow State

**Status: DESIGN PROPOSAL — for human/architect review. Not a ruling. No code changes.**

RULING-EDGECASE-001 §1.2 adopted Option (b): split only the carry buffer per-direction
(`carry_c2s` / `carry_s2c`), keeping all detection counters and window state as shared
per-flow aggregates. Section 1.3 provides an explicit per-field classification table
ratifying "PER-FLOW keep shared" for all non-carry state in ENIP.

This document re-examines that classification at the codebase-wide level. The edge-case
hunt surfaced a concrete DNP3 desync-latch failure mode that the §1.3 "PER-FLOW keep
shared" classification for `is_non_*` flags does not survive intact. It also provides a
full inventory of per-flow state across all three TCP-based analyzers (ENIP, DNP3, Modbus)
to inform the pending Modbus EC-X1 sibling-fix scope decision.

---

## 1. Per-Flow State Inventory and Classification

### 1.1 ENIP — EnipFlowState (enip.rs:231-401)

| Field | Current classification (RULING §1.3) | Re-evaluated classification | Rationale |
|-------|-------------------------------------|-----------------------------|-----------|
| `carry_c2s: Vec<u8>` | PER-DIRECTION (split) | PER-DIRECTION — correct | Partial frame accumulation, direction-specific by definition |
| `carry_s2c: Vec<u8>` | PER-DIRECTION (split) | PER-DIRECTION — correct | Symmetric |
| `is_non_enip: bool` | PER-FLOW (keep shared) | PER-FLOW — correct | See §2 below for detailed analysis |
| `command_counts: HashMap<u16, u64>` | PER-FLOW (keep shared) | PER-FLOW — correct | Flow-level aggregate; BC-2.17.004 Inv-3 and BC-2.17.016 PC-0 specify flow-level |
| `parse_errors: u64` | PER-FLOW (keep shared) | PER-FLOW — correct | Lifetime error counter; direction not semantically meaningful |
| `malformed_in_window: u64` | PER-FLOW (keep shared) | PER-FLOW — correct | T0814 fires on structural anomalies per flow |
| `malformed_anomaly_emitted: bool` | PER-FLOW (keep shared) | PER-FLOW — correct | One-shot guard is per-window per-flow |
| `malformed_window_start_ts: u32` | PER-FLOW (keep shared) | PER-FLOW — correct | Window is flow-level; malformed frames can appear in either direction |
| `pdu_count: u64` | PER-FLOW (keep shared) | PER-FLOW — correct | BC-2.17.024 is flow-scoped |
| `write_count_in_window: u64` | PER-FLOW (keep shared) | PER-FLOW — correct | T0836; write requests are c2s, but per §1.3 the CIP service gate already discriminates |
| `write_window_start_ts: u32` | PER-FLOW (keep shared) | PER-FLOW — correct | Window is flow-level |
| `write_burst_emitted: bool` | PER-FLOW (keep shared) | PER-FLOW — correct | One-shot guard |
| `error_counts_in_window: HashMap<u8, u64>` | PER-FLOW (keep shared) | PER-FLOW — correct | T0888; response errors are s2c; CIP response gate already discriminates |
| `error_window_start_ts: u32` | PER-FLOW (keep shared) | PER-FLOW — correct | Window is flow-level |
| `error_window_active: bool` | PER-FLOW (keep shared) | PER-FLOW — correct | Sentinel for "any error seen" on the flow |
| `error_rate_emitted: bool` | PER-FLOW (keep shared) | PER-FLOW — correct | One-shot guard |
| `list_identity_emitted: bool` | PER-FLOW (keep shared) | PER-FLOW — correct | One-shot guard, flow-level |

**ENIP verdict: the §1.3 classification is fully correct. No re-evaluation needed.**

### 1.2 DNP3 — Dnp3FlowState (dnp3.rs:217-288)

| Field | Prior classification (analogous to RULING §1.3) | Re-evaluated classification | Notes |
|-------|--------------------------------------------------|----------------------------|-------|
| `carry_c2s: Vec<u8>` | PER-DIRECTION (split, STORY-140) | PER-DIRECTION — correct | |
| `carry_s2c: Vec<u8>` | PER-DIRECTION (split, STORY-140) | PER-DIRECTION — correct | |
| `is_non_dnp3: bool` | PER-FLOW (analogous to ENIP §1.3) | **AMBIGUOUS — see §2** | Critical failure mode identified |
| `fc_counts: HashMap<u8, u64>` | PER-FLOW | PER-FLOW — correct | Aggregate FC distribution across flow |
| `frame_count: u64` | PER-FLOW | PER-FLOW — correct | Total frames, flow-level |
| `parse_errors: u64` | PER-FLOW | PER-FLOW — correct | Lifetime, never reset |
| `direct_operate_count: u32` | PER-FLOW | PER-FLOW — correct | Control-class burst; FC 0x03/0x04 are c2s requests, but counter is flow-aggregate |
| `window_start_ts: u32` | PER-FLOW | PER-FLOW — correct | DO burst window is flow-level |
| `direct_operate_emitted: bool` | PER-FLOW | PER-FLOW — correct | One-shot guard |
| `master_addrs_seen: Vec<u16>` | PER-FLOW | PER-FLOW — correct | Set of masters seen on this flow |
| `restart_event_count: u64` | PER-FLOW | PER-FLOW — correct | BC-2.15.015 correlation window |
| `block_event_count: u64` | PER-FLOW | PER-FLOW — correct | BC-2.15.014 correlation window |
| `pending_requests: HashMap<(u16, u8), u32>` | PER-FLOW | PER-FLOW — correct | Request→response correlation; by definition cross-direction |
| `block_finding_emitted_this_window: bool` | PER-FLOW | PER-FLOW — correct | One-shot per window |
| `loss_of_control_emitted: bool` | PER-FLOW | PER-FLOW — correct | One-shot guard |
| `correlation_window_start_ts: u32` | PER-FLOW | PER-FLOW — correct | 300s window, flow-level |
| `correlation_window_seeded: bool` | PER-FLOW | PER-FLOW — correct | Initialization flag |
| `malformed_in_window: u64` | PER-FLOW | PER-FLOW — correct | T0814 windowed count, flow-level |
| `malformed_anomaly_emitted: bool` | PER-FLOW | PER-FLOW — correct | One-shot guard |
| `enable_unsolicited_seen: bool` | PER-FLOW | PER-FLOW — correct | Flow-lifetime protocol-state context |
| `response_seen: bool` | PER-FLOW | PER-FLOW — correct | Flow-lifetime protocol-state context |
| `unsolicited_anomaly_emitted: bool` | PER-FLOW | PER-FLOW — correct | One-shot guard |
| `unexpected_source_emitted: bool` | PER-FLOW | PER-FLOW — correct | One-shot guard |

**DNP3 verdict: all fields except `is_non_dnp3` are correctly per-flow. See §2 for the
latch issue.**

### 1.3 Modbus — ModbusFlowState (modbus.rs:119-171)

| Field | Classification | Notes |
|-------|---------------|-------|
| `pending: HashMap<(u16, u8), (u8, u32)>` | PER-FLOW — correct | Request→response correlation; by definition cross-direction |
| `write_count: u64` | PER-FLOW — correct | Lifetime write-class FC count |
| `exception_count: u64` | PER-FLOW — correct | Lifetime exception count |
| `pdu_count: u64` | PER-FLOW — correct | Lifetime PDU count |
| `last_ts: u32` | PER-FLOW — correct | Last-seen timestamp; used for correlation, not direction-specific |
| `window_write_count: u32` | PER-FLOW — correct | 1s burst window count |
| `window_start_ts: u32` | PER-FLOW — correct | 1s window anchor |
| `window_burst_emitted: bool` | PER-FLOW — correct | One-shot guard |
| `sustained_window_start_ts: u32` | PER-FLOW — correct | >=2s window anchor |
| `sustained_window_write_count: u32` | PER-FLOW — correct | >=2s window count |
| `sustained_burst_emitted: bool` | PER-FLOW — correct | One-shot guard |
| `t0831_window_start_ts: u32` | PER-FLOW — correct | 5s T0831 window anchor |
| `t0831_window_write_count: u32` | PER-FLOW — correct | 5s window count |
| `t0831_burst_emitted: bool` | PER-FLOW — correct | One-shot guard |
| `exception_window_counts: HashMap<u8, u32>` | PER-FLOW — correct | Per-exception-code burst window |
| `exception_window_start_ts: HashMap<u8, u32>` | PER-FLOW — correct | Per-exception-code anchors |
| `exception_burst_emitted: HashMap<u8, bool>` | PER-FLOW — correct | One-shot guards per exception code |
| `is_non_modbus: bool` | PER-FLOW — see §3 | Analogous to `is_non_dnp3`; Modbus EC-X1 scope discussion in §3 |
| `carry: Vec<u8>` | **SINGLE — NOT YET SPLIT** | Modbus has one carry buffer; EC-X1 sibling fix required for v0.12.0 |

**Modbus verdict: all detection state is correctly per-flow. The sole structural issue is
`carry: Vec<u8>` (not yet split per-direction — this is the Modbus EC-X1 sibling fix).
The `is_non_modbus` latch shares the same structural pattern as `is_non_dnp3` (see §3).**

---

## 2. The DNP3 `is_non_dnp3` Desync-Latch Finding

### 2.1 The Bug

The desync latch check in dnp3.rs:363-370 is:

```rust
if active_carry!(flow, direction).is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

After STORY-140, the `active_carry!(flow, direction)` macro correctly selects
`carry_c2s` or `carry_s2c` by direction. **This means the condition `active_carry!(...).is_empty()`
checks ONLY the carry for the CURRENT delivery's direction.** It does not check whether
the OTHER direction's carry is also empty.

**Concrete failure scenario:**

1. TCP connection establishes between master (A) and outstation (B).
2. First `on_data` call: `direction=ClientToServer`, data=valid DNP3 sync word `[0x05, 0x64, ...]`.
   - `carry_c2s.is_empty()` = true; data[0..2] == `[0x05, 0x64]` → DNP3 accepted.
   - `carry_c2s` gets data appended; frame walk runs; `frame_count += 1`.
   - `carry_s2c` remains empty.
3. Second `on_data` call: `direction=ServerToClient`, data = non-DNP3 junk (e.g., TLS
   ClientHello or HTTP/2 preface on the same port due to a port-reuse event on a
   capture-merged trace).
   - `active_carry!(flow, ServerToClient)` = `carry_s2c.is_empty()` = **true** (never written).
   - data[0..2] != `[0x05, 0x64]` → condition fires.
   - `flow.is_non_dnp3 = true` — **the entire flow is latched as non-DNP3.**
4. All subsequent `on_data` calls (including further legitimate c2s DNP3 frames) are
   no-ops. The established c2s direction is silenced permanently.

**Why this was not caught by EC-X1:** RULING-EDGECASE-001 §1.2 Option (b) rationale
notes: "if the flow is not ENIP, both directions are not ENIP. A per-direction latch
would require both to fire before the flow is abandoned, which is incorrect." This
reasoning is valid for ENIP — ENIP uses TCP port 44818 and a well-known session
registration structure, so non-ENIP bytes in either direction are a reliable indicator
the entire flow is not ENIP.

For DNP3, the situation is more nuanced. DNP3 masters and outstations can share
connections with non-DNP3 traffic on the same port in some environments (DNP3 uses
TCP port 20000 but is also tunneled over other ports). More critically, in
merged pcap captures covering multiple sessions, the FlowKey canonicalizes by
(src_ip, dst_ip, src_port, dst_port), which may alias sessions on different
physical interfaces that happen to share a 4-tuple. In such a case, legitimate DNP3
c2s traffic followed by non-DNP3 s2c traffic from a different session silences the
legitimate DNP3 flow.

### 2.2 Proposed Fix for DNP3 `is_non_dnp3`

Two targeted options:

**Option A — "both carries must be empty" latch condition:**

```rust
if carry_c2s.is_empty() && carry_s2c.is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

This makes the desync latch fire only when BOTH directions have no established carry —
i.e., when this is the genuine first-ever delivery to an unestablished flow. Once either
direction has accepted data, a junk delivery in the other direction does NOT latch the
flow as non-DNP3.

Tradeoff: if both carries are empty (second delivery to an unestablished flow that had
a c2s non-DNP3 first delivery followed by a same-second s2c non-DNP3 delivery), the
latch would NOT fire. However, the c2s first-delivery non-DNP3 check fires first
(carry_c2s was empty, data was non-DNP3 → is_non_dnp3 set). The c2s check already
handles the "truly non-DNP3 flow" case.

**This option appears correct.** The desync latch is intended to identify flows that
are NOT DNP3. Such flows will always fail the sync check on the first c2s delivery
(since the master initiates), setting is_non_dnp3 immediately regardless of s2c state.
The s2c case only matters when the c2s direction was already established — and in that
case, a single junk s2c delivery should not veto the established c2s direction.

**Option B — per-direction `is_non_dnp3` tracking:**

Replace `is_non_dnp3: bool` with `is_non_dnp3_c2s: bool` and `is_non_dnp3_s2c: bool`.
The bail check at the top of `on_data` becomes:

```rust
let active_non = match direction {
    ClientToServer => flow.is_non_dnp3_c2s,
    ServerToClient => flow.is_non_dnp3_s2c,
};
if active_non { return; }
```

The latch assignment becomes:

```rust
match direction {
    ClientToServer => flow.is_non_dnp3_c2s = true,
    ServerToClient => flow.is_non_dnp3_s2c = true,
}
```

Tradeoff: this allows one direction to be active (processing DNP3) while the other is
latched. This is semantically wrong for the RULING-EDGECASE-001 §1.3 rationale ("if
the flow is not ENIP/DNP3, both directions are not") — but the finding above shows that
the per-flow rationale breaks down once carry is per-direction. The ENIP §1.3 rationale
("a per-direction latch would require both to fire") applies differently when the
FIRST delivery semantics are now per-direction.

Option B is more complex (two bool fields, two check paths, two latch paths) and would
affect the ENIP `is_non_enip` field analogously (same structural argument applies, though
ENIP's latch fires on carry-cap overflow, not sync-word check, so the failure mode is less
likely). Option A is preferred for its minimal surface area.

**Recommendation for DNP3: Option A.** Change the desync-latch condition to check
`carry_c2s.is_empty() && carry_s2c.is_empty()` before latching. This is a one-line
change to dnp3.rs:363.

---

## 3. Broader Pattern: Full Per-Direction Keying vs Selective Per-Direction Fields

### 3.1 Full `HashMap<(FlowKey, Direction), State>` keying

Replace `flows: HashMap<FlowKey, Dnp3FlowState>` with
`flows: HashMap<(FlowKey, Direction), Dnp3FlowState>` (and analogously for ENIP and Modbus).

This is the architecture that RULING-EDGECASE-001 §1.2 explicitly evaluated and rejected
as Option (a) for v0.11.0:

> "Option (a) would replace `HashMap<FlowKey, EnipFlowState>` with
> `HashMap<(FlowKey, Direction), EnipFlowState>`, duplicating all windowed state per
> direction. That is architecturally cleaner for source-IP resolution and detection
> attribution, but it is a much larger change: every detection BC would need
> direction-aware rewording, window semantics would bifurcate (which direction owns
> the T0836 count?), and the `summarize()` drainage would need to merge two half-states."

This assessment remains correct. Full per-direction keying would:
1. Split detection counters per-direction. Burst detection requires aggregating both
   directions (a T0836 write burst is identified by the total write count on the flow,
   regardless of which direction the writes arrived from in a hypothetical bidirectional
   scenario). Bifurcating counters would require a merge step at `summarize()` time.
2. Double the number of state entries in `flows` for every active flow.
3. Require BC-level amendments to specify direction-aware semantics for every detection.

**Full per-direction keying is not recommended** for v0.12.0.

### 3.2 Selective per-direction fields (current approach + targeted extensions)

The current approach — per-flow keying with selective per-direction fields for carry
buffers only — is the correct long-term architecture for the detection patterns in
wirerust. The §1.3 classification table correctly identifies that detection counters are
aggregated at flow level. The carry buffer is the only state that is genuinely
per-direction because TCP stream reassembly is inherently per-direction: a partial frame
from direction A cannot be semantically combined with bytes from direction B.

The exception is the desync-latch bug (§2), which is not a carry issue but a CONDITION
issue: the first-delivery check uses the per-direction carry as a proxy for "is this
the first-ever byte for this flow?" After the carry split, that proxy answers "is this
the first-ever byte for THIS DIRECTION?" — which is a different question. The fix
(Option A) addresses this by widening the check back to "are BOTH direction carries
empty?" without introducing per-direction flags.

### 3.3 Current ENIP `is_non_enip` — is the §1.3 rationale still correct?

RULING-EDGECASE-001 §1.3 rationale for `is_non_enip: PER-FLOW (keep shared)`:
> "if the flow is not ENIP, both directions are not ENIP. A per-direction latch would
> require both to fire before the flow is abandoned, which is incorrect."

ENIP's `is_non_enip` is set by carry-cap overflow (enip.rs:183-184, 833), NOT by
sync-word check. The carry-cap path fires when either `carry_c2s` or `carry_s2c`
exceeds 600 bytes. At that point, the flow is correctly latched as non-ENIP because a
legitimate ENIP frame can never require accumulating more than 600 bytes of carry
(MAX frame = 600 bytes, so a partial frame can be at most 599 bytes).

The first-delivery sync-word check does not exist in ENIP (ENIP uses `is_valid_enip_frame`
on parsed frames, not a prefix check on raw bytes). Therefore the DNP3 desync-latch
failure mode does NOT apply to ENIP. The §1.3 "PER-FLOW keep shared" classification
for `is_non_enip` remains correct.

---

## 4. Modbus Carry Split: Scope for EC-X1 Sibling Fix

Modbus (`ModbusFlowState`) retains `carry: Vec<u8>` — a single shared carry buffer. The
RULING-EDGECASE-001 §1.6 sibling sweep ruling says: "fix ENIP now; track DNP3 as a
sibling follow-up chore for v0.12.0." DNP3's carry was fixed in STORY-140. Modbus is
the remaining sibling.

### 4.1 Does Modbus have the same cross-direction splice risk as ENIP?

Yes. Modbus `on_data` (modbus.rs:1043-1056) prepends `flow.carry` to incoming data
regardless of direction. If a partial c2s ADU (e.g., 4 bytes of a 12-byte request) is
stashed in `carry` and the next `on_data` call is an s2c response, the s2c response bytes
are prepended with the c2s partial. The 3-point validity gate at modbus.rs:1091
(`is_valid_modbus_adu`) may or may not pass the splice depending on byte values:

- If the spliced buffer looks like a valid MBAP header (protocol_id=0x0000, length 2-254),
  the splice commits as a "valid" ADU whose function code comes from the s2c response body.
- If the splice fails the gate (protocol_id != 0x0000 or length out of range), `is_non_modbus`
  is latched — silencing the flow. This is a false desync.

Both outcomes are incorrect. The Modbus carry split (EC-X1 sibling fix) is required.

### 4.2 Does the `is_non_modbus` latch have the same desync issue as `is_non_dnp3`?

The `is_non_modbus` latch is set in two places:
1. Carry-cap overflow (modbus.rs:1081, 1121) — analogous to `is_non_enip` carry-cap.
2. Invalid MBAP protocol_id or out-of-range length (modbus.rs:1101-1104).

Case (1): Same as ENIP — cap overflow is a legitimate "not Modbus" signal regardless of
direction. PER-FLOW is correct.

Case (2): After the carry split, if a c2s partial stash in `carry_c2s` gets prepended
to an s2c delivery and produces an invalid MBAP header, `is_non_modbus` fires. This is
the splice-driven false-desync path. The fix is the carry split (§4.1 above) — once
carries are per-direction, the splice cannot occur, and `is_non_modbus` from the gate
check can only fire on genuinely invalid Modbus traffic.

Unlike DNP3, there is no sync-word prefix check in Modbus; the first-delivery check is
the MBAP gate at modbus.rs:1091. After the carry split, the gate only operates on
direction-correct combined buffers, so the false-desync path is eliminated. No additional
change to `is_non_modbus` semantics is needed — it can remain PER-FLOW.

### 4.3 Recommended Modbus EC-X1 sibling fix scope

The Modbus EC-X1 sibling fix should:
1. Replace `carry: Vec<u8>` with `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`.
2. Thread `Direction` into `on_data` if not already present.
3. In the walk loop, select the active carry by direction for prepend and stash-back.
4. Apply the same carry-cap DoS guard to both directional carries independently.
5. Do NOT add per-direction `is_non_modbus` flags — per §4.2, PER-FLOW is correct
   after the carry split.

The fix does NOT need to address window arithmetic (that is the DESIGN-TIMESTAMP-MONOTONICITY
scope).

---

## 5. Recommendation Summary

| Issue | Affected file(s) | Recommended action |
|-------|-----------------|-------------------|
| DNP3 `is_non_dnp3` desync-latch desync | dnp3.rs:363 | Option A: widen latch condition to `carry_c2s.is_empty() && carry_s2c.is_empty()` |
| Modbus carry not yet split | modbus.rs | EC-X1 sibling: add `carry_c2s`/`carry_s2c`, thread `Direction` into `on_data` |
| ENIP `is_non_enip` | enip.rs | No change needed; §3.3 confirms PER-FLOW is correct |
| Full per-direction keying | all | Rejected for v0.12.0; revisit if detection attribution requirements change |

**This note informs whether the pending Modbus EC-X1 fix should also address
cross-direction state.** The answer is: the Modbus EC-X1 carry split is sufficient.
No Modbus detection state fields require per-direction treatment after the carry fix.
The `is_non_modbus` latch can remain per-flow after the carry split eliminates the
splice-driven false-desync path.

---

## 6. Open Questions for Human Review

1. **DNP3 desync-latch fix (Option A)**: the proposed one-line change to dnp3.rs:363
   weakens the latch condition from "this direction's carry is empty" to "both carries
   are empty." Is there a legitimate scenario where an s2c non-DNP3 delivery on an
   established c2s DNP3 flow should ALSO latch `is_non_dnp3` (i.e., is Option B
   preferred)? The answer depends on whether the product should treat a mixed-protocol
   flow as "not DNP3" or "partially DNP3."

2. **Modbus EC-X1 scope**: should the Modbus carry split be done atomically with the
   Modbus timestamp fix (DESIGN-TIMESTAMP-MONOTONICITY §4 Phase 1), or as separate
   stories? The changes are mechanically independent but both touch `ModbusFlowState`.

3. **RULING-EDGECASE-001 §1.3 re-confirmation for ENIP**: the analysis in §3.3 above
   confirms that the §1.3 "PER-FLOW keep shared" classification for `is_non_enip` is
   correct after STORY-139. This note does NOT amend the ruling; it confirms it. No
   spec change needed.
