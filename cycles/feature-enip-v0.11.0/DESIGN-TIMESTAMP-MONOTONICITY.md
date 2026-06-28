---
document_type: design-proposal
title: "Timestamp Monotonicity: Shared Cross-Analyzer Design"
cycle: feature-enip-v0.11.0
author: architect
date: 2026-06-28
status: proposal
informs: v0.12.0-planning
release_held: null
related_rulings:
  - RULING-EDGECASE-001
---

# DESIGN PROPOSAL: Timestamp Monotonicity — Shared Cross-Analyzer Design

**Status: DESIGN PROPOSAL — for human/architect review. Not a ruling. No code changes.**

This note surfaces a systemic pattern found during the post-convergence edge-case hunt
for v0.11.0. RULING-EDGECASE-001 §2 fixed ENIP's `wrapping_sub` → `saturating_sub` and
DNP3's analogous sites were noted as v0.12.0 sibling work. This document widens the lens:
all four analyzers (enip.rs, dnp3.rs, modbus.rs, arp.rs) have windowed detections, and
`saturating_sub` — the fix that was shipped — is not a complete solution to
non-monotonic timestamps. It merely fails differently than `wrapping_sub` in certain
non-obvious ways. Understanding the full failure taxonomy is prerequisite to designing
a durable shared solution.

---

## 1. Enumeration of All Timestamp-Delta / Window Sites

### 1.1 enip.rs — FIXED (STORY-139, RULING-EDGECASE-001 §2.2)

| Site | Line | Detection | Current arithmetic |
|------|------|-----------|--------------------|
| Error-rate window expiry | enip.rs:1154 | T0888 (BC-2.17.008) | `saturating_sub` |
| Write-burst window expiry | enip.rs:1338 | T0836 (BC-2.17.012) | `saturating_sub` |
| Malformed window expiry | enip.rs:813 | T0814 (BC-2.17.018) | `saturating_sub` |

ENIP is fully swept. All three sites use `saturating_sub` as of STORY-139.

### 1.2 dnp3.rs — PARTIALLY FIXED (STORY-140 carry only; window arithmetic NOT yet swept)

| Site | Line | Detection | Current arithmetic |
|------|------|-----------|--------------------|
| Direct-operate window expiry | dnp3.rs:901 | T1692.001 (BC-2.15.010) | `saturating_sub` |
| Direct-operate in-window rate | dnp3.rs:921,925 | T1692.001 | `saturating_sub` |
| Block-command timeout scan | dnp3.rs:1083 | T1691.001 (BC-2.15.014) | `saturating_sub` |
| Correlation window expiry | dnp3.rs:1188 | BC-2.15.015 multi-field reset | `saturating_sub` |
| Correlation in-window elapsed | dnp3.rs:1238,1597,1603 | BC-2.15.015/024 | `saturating_sub` |

DNP3 has already been converted entirely to `saturating_sub` (all sites show
`saturating_sub` in grep output). The comment at dnp3.rs:164-166 documents this
explicitly ("saturating_sub used for all comparisons ... prevents panic under
overflow-checks=true on out-of-order pcap replay"). DNP3's window arithmetic is
consistent with the post-RULING-EDGECASE-001 ENIP state.

### 1.3 modbus.rs — NOT YET SWEPT (uses `wrapping_sub` throughout)

| Site | Line | Detection | Current arithmetic |
|------|------|-----------|--------------------|
| T0831 coordinated-write window expiry | modbus.rs:534 | T0831 (BC-2.14.016) | `wrapping_sub` |
| Write-burst 1s window expiry | modbus.rs:595 | T0806/T1692.001 (BC-2.14.017) | `wrapping_sub` |
| Sustained write window elapsed | modbus.rs:670 | T0806/T1692.001 (BC-2.14.017) | `wrapping_sub` |
| Exception-burst window expiry | modbus.rs:820 | BC-2.14.019 | `wrapping_sub` |

The comment at modbus.rs:112-113 explicitly mandates `wrapping_sub` ("All window-duration
arithmetic MUST use `wrapping_sub`") — this was the pre-RULING-EDGECASE-001 house style.
The mandate comment has not been updated since RULING-EDGECASE-001 changed the preference
to `saturating_sub`. All four Modbus window sites remain on `wrapping_sub`.

### 1.4 arp.rs — USES `saturating_sub` BUT FAILS DIFFERENTLY

| Site | Line | Detection | Current arithmetic |
|------|------|-----------|--------------------|
| D1 flap-window expiry check | arp.rs:810 | ARP spoof HIGH escalation (BC-2.16.004) | `saturating_sub` |
| D1 in-window elapsed for HIGH gate | arp.rs:828 | ARP spoof HIGH escalation | `saturating_sub` |
| D3 storm window-expiry (in_window check) | arp.rs:981 | ARP storm (BC-2.16.008) | `saturating_sub` |
| D3 storm rate-denominator elapsed | arp.rs:1000 | ARP storm rate | `saturating_sub` |

ARP uses `saturating_sub` everywhere. However `saturating_sub` introduces a
**distinct** failure mode that is NOT present in the ENIP/DNP3 context and is not
addressed by RULING-EDGECASE-001.

---

## 2. Failure Mode Classification per Site

### 2.1 `wrapping_sub` failure modes (Modbus)

**Evasion via backwards timestamp:**
A packet with `now_ts < window_start_ts` produces `wrapping_sub(now_ts, window_start_ts)`
≈ 2^32 − delta, which is ≫ any threshold. The window resets spuriously, discarding the
accumulated burst count. This is the exact EC-X2 failure confirmed in ENIP.

Modbus sites affected (backwards-ts → window reset → detection suppressed):
- modbus.rs:534 — T0831 coordinated-write window reset (5s window)
- modbus.rs:595 — write-burst 1s window reset
- modbus.rs:670 — sustained write window elapsed (rate denominator blows up to 2^32 → rate
  calculation becomes `count / 2^32 ≈ 0` → sustained trigger never fires)
- modbus.rs:820 — exception-burst window reset (10s window)

**Genuine u32 rollover:** `wrapping_sub` was intended to handle rollover. As RULING-EDGECASE-001
§2.3 notes, genuine pcap-relative second-timestamp rollover requires ~136 years of monotonic
uptime or a capture restart that re-zeros; this is operationally negligible.

### 2.2 `saturating_sub` failure modes (ENIP, DNP3, ARP)

**The shipped ENIP/DNP3 fix is correct for the detection-evasion threat model:**
`saturating_sub(backwards_ts, window_start_ts)` = 0. Window does not reset. Burst
accumulates. This is the intended behavior per RULING-EDGECASE-001 §2.2.

**BUT: ARP storm `detect_storm` has a false-positive path under backwards timestamps:**

Scenario: MAC A sends 60 ARP frames at ts=100 (window_start_ts=100, count_in_window=60).
Then a single backwards-clock frame arrives at ts=50.

1. `elapsed = saturating_sub(50, 100) = 0` → `in_window = (0 <= ARP_FLAP_WINDOW_SECS=60)` → `true`.
2. `count_in_window` increments to 61.
3. Step 3 (rate evaluation): `elapsed = saturating_sub(50, 100) = 0`. `denominator = max(1, 0) = 1`.
4. `rate = 61 / 1 = 61`. If `61 >= storm_rate (default 50)`: storm finding fires.

**The false-positive arises from the rate denominator, not the window-expiry check.**
The `max(1, elapsed)` guard at arp.rs:1001 was designed to avoid division by zero when
all frames arrive in the same second (elapsed=0 legitimately), but it also applies when
a backwards-timestamp packet forces `elapsed=0` artificially. With `saturating_sub`,
a legitimate 60-frame burst spread over 60 seconds (rate ≈ 1 fps) becomes a computed
rate of 61/1 = 61 fps after one backwards packet — triggering a storm false positive.

**ARP D1 flap-window: backwards timestamp pins window open (no false positive, but incorrect)**

At arp.rs:811: `elapsed > ARP_FLAP_WINDOW_SECS` is the window-expiry check. With
`saturating_sub`, a backwards packet gives `elapsed=0`, which is NOT `> 60`, so the
window is NOT reset. This is the correct security behavior (same reasoning as EC-X2:
a backwards packet should not abort a detection window). However: if the window is
genuinely stale (60+ real seconds have passed since `first_rebind_ts`), a backwards
out-of-order packet can PREVENT the legitimate window reset that SHOULD occur when the
next in-order packet arrives. The window stays pinned open, counting a new rebind in an
expired window as if it were in the original window — producing a false HIGH-confidence
finding when only one rebind occurred in the current window.

This is the inverse of the ENIP/Modbus problem: instead of a false negative (detection
suppressed), it produces a false positive (detection escalated when count should have
reset).

### 2.3 `saturating_sub` + frozen/identical timestamps

When pcap timestamps have second granularity and multiple packets arrive in the same
capture second (common in reassembly flushes), `elapsed = saturating_sub(ts, ts) = 0`.
This is intentional and documented (modbus.rs:651-658 "Known v1 limitation F-DELTA-004").
No additional fix needed here — it is an inherent property of second-granularity timestamps.

### 2.4 Summary table

| Analyzer | Site | Arithmetic | Backwards-ts failure mode |
|----------|------|-----------|--------------------------|
| enip.rs | error/write/malformed windows | `saturating_sub` | None (fixed by EC-X2) |
| dnp3.rs | all window sites | `saturating_sub` | None (already correct) |
| modbus.rs | all 4 window sites | `wrapping_sub` | FALSE NEGATIVE: window reset, burst suppressed |
| arp.rs D3 storm | rate-denominator | `saturating_sub` | FALSE POSITIVE: rate=count/1 when backwards ts; storm fires spuriously |
| arp.rs D1 flap | window-expiry | `saturating_sub` | INCORRECT WINDOW: stale window pinned open, false HIGH escalation possible |

---

## 3. Candidate Shared Designs

### Option A — `saturating_sub` uniformly (minimal, current ENIP/DNP3 approach)

Apply the RULING-EDGECASE-001 pattern to all remaining `wrapping_sub` sites (Modbus).
Leave `saturating_sub` on ARP unchanged.

**For Modbus:** Straightforward. Replace the four `wrapping_sub` calls with `saturating_sub`.
This stops the detection-evasion path for all Modbus windows. The semantics are identical
to the ENIP fix: backwards packet → elapsed=0 → window does not reset → burst preserved.

**For ARP D3 storm:** `saturating_sub` is already present. The false-positive path from
the `max(1, elapsed)` denominator is a SEPARATE issue not addressed by choosing between
`wrapping_sub` and `saturating_sub`. It requires denominator policy (see Option B/C).

**For ARP D1 flap:** `saturating_sub` is already present. The window-pinning issue is
a SEPARATE issue from the arithmetic choice.

Tradeoffs:
- Pros: minimal diff, consistent with shipped ENIP/DNP3 approach, does not introduce new
  shared infrastructure.
- Cons: Does NOT address the ARP storm false-positive or ARP flap-window pin-open issue.
  Those require orthogonal fixes regardless of which option is chosen for the global policy.

### Option B — per-window `max_seen_ts` monotonic clamp

Add a `max_seen_ts: u32` field to each window's state. On every packet delivery,
compute `effective_ts = ts.max(max_seen_ts)` before all elapsed calculations. Update
`max_seen_ts = effective_ts`. All window arithmetic uses `effective_ts`, which is
guaranteed monotonically non-decreasing.

For ARP storm specifically: `effective_ts` would be max(50, 100) = 100, so
`elapsed = saturating_sub(100, 100) = 0`, `denominator = max(1, 0) = 1`, rate = 61/1 = 61.
This does NOT fix the D3 false positive: the denominator problem remains because the
elapsed is legitimately 0 when all traffic occurs in the same second. However, it
prevents the elapsed from being artificially forced to 0 by a backwards-clock packet
when real elapsed has grown: a packet at ts=50 after packets at ts=200 would use
`effective_ts=200`, so elapsed would reflect real time.

Tradeoffs:
- Pros: correctly distinguishes "all traffic same-second (elapsed=0 real)" from "backwards
  packet artificially zeroing elapsed." Eliminates the ARP storm false-positive in the
  described scenario (60 frames at ts=100, then one at ts=50: effective_ts=100 throughout,
  elapsed=0 real, denominator=1, rate=61 — the false positive is NOT eliminated because
  the real scenario is the same as legitimate same-second traffic).
- Cons: adds one u32 field per window per flow. More state. Does NOT fully fix the D3
  storm denominator problem because the problem is identical to legitimate same-second
  traffic. Requires touching all window-state structs.

### Option C — `WindowClock` shared helper

Extract a small `WindowClock` struct that encapsulates the window-start timestamp, a
`max_seen_ts` field, and the arithmetic policy. All analyzers instantiate `WindowClock`
per window instead of raw `u32` fields.

```rust
pub struct WindowClock {
    window_start_ts: u32,
    max_seen_ts: u32,
}

impl WindowClock {
    /// Returns (elapsed_secs, is_expired).
    /// elapsed uses max_seen_ts as the reference to enforce monotonicity.
    /// Returns (0, false) if ts <= self.max_seen_ts (backwards-clock).
    pub fn tick(&mut self, ts: u32, threshold_secs: u32) -> (u32, bool) {
        let effective_ts = ts.max(self.max_seen_ts);
        self.max_seen_ts = effective_ts;
        let elapsed = effective_ts.saturating_sub(self.window_start_ts);
        (elapsed, elapsed > threshold_secs)
    }

    pub fn reset(&mut self, ts: u32) {
        let effective_ts = ts.max(self.max_seen_ts);
        self.window_start_ts = effective_ts;
        self.max_seen_ts = effective_ts;
    }
}
```

All window-expiry checks become a single `clock.tick(ts, threshold)` call. The `max_seen_ts`
monotonic clamp is embedded in the helper, not scattered across analyzers.

For ARP D3 storm false positive: the `max_seen_ts` clamp prevents the backwards-ts from
zeroing elapsed artificially, but when all frames arrive in the same second legitimately
(elapsed=0, denominator=1), the behavior is unchanged. The denominator policy itself
(`max(1, elapsed)`) is a separate design question orthogonal to `WindowClock`.

Tradeoffs:
- Pros: single canonical implementation of window clock arithmetic; eliminates per-analyzer
  drift (Modbus still uses `wrapping_sub` because it was not in scope for RULING-EDGECASE-001);
  makes future fixes trivial (change `WindowClock`, all analyzers benefit). Testable as a
  pure-core unit (no side effects).
- Cons: significant struct migration for `Dnp3FlowState`, `EnipFlowState`, `ModbusFlowState`,
  `StormCounter`, `BindingEntry`. Each per-flow state field set that groups window fields would
  need refactoring. This is a v0.12.0 architectural scope item, not a patch. It also changes
  public field names on structs that have tests referencing fields directly — a large test
  surface update.

---

## 4. Recommendation

**Recommended approach: two-phase.**

**Phase 1 (v0.12.0 Modbus sibling fix, low risk):**
Apply Option A — replace `wrapping_sub` with `saturating_sub` at the four Modbus sites
(modbus.rs:534, 595, 670, 820). Also update the mandatory comment at modbus.rs:112
("MUST use `wrapping_sub`") to align with the post-RULING-EDGECASE-001 policy. This
is a mechanical change identical in structure to STORY-139's ENIP fix. Risk: minimal.

**Phase 2 (v0.12.0 ARP denominator policy, medium risk):**
Address the ARP D3 storm false-positive separately. The `max(1, elapsed)` denominator
at arp.rs:1001 is the root of the false positive. The correct fix depends on a product
decision about the desired behavior when all frames in a burst share the same timestamp:

- Option: require a minimum elapsed threshold before rate-based emission (e.g., suppress
  storm emission until `elapsed >= 2`). This eliminates same-second false positives.
- Option: change the denominator to `elapsed.max(2)` (minimum 2s window for rate
  calculation). This dampens false positives at the cost of slightly increasing the burst
  count needed to trigger within a 2-second window.
- Option: track frame count only (no rate denominator at all) when elapsed < threshold.
  Emit purely on frame count when elapsed < 1, on rate when elapsed >= 1.

This denominator policy is a BC-level decision (BC-2.16.008 ARP-AMB-003 was previously
resolved), not just an arithmetic choice. **Flag for human/architect review before
implementing.**

**Option C (`WindowClock`) is deferred to v0.13.0 or later.** The struct migration cost is
high and the benefit over the two-phase approach is primarily ergonomic (unified struct).
It does not change the semantic behavior of Option A+B phase fixes; it merely consolidates
them. Defer until the number of windowed detections grows large enough to justify the
migration cost.

---

## 5. Interaction with EC-X2 (RULING-EDGECASE-001 §2)

The ENIP and DNP3 EC-X2 fix (STORY-139, STORY-140) is NOT regressed by any option above.
All three options preserve `saturating_sub` semantics for already-fixed sites. The
recommendation for Modbus (Option A, Phase 1) uses the same `saturating_sub` pattern
as the shipped ENIP fix — no new semantics are introduced.

The ARP denominator fix (Phase 2) is additive to the existing `saturating_sub` at
arp.rs:981/1000 — it changes the denominator policy, not the elapsed arithmetic.

---

## 6. Interaction with Attacker/Capture-Controlled Timestamps

`raw.timestamp_secs` is derived from the pcap packet header, which is:
1. Attacker-controlled (adversary can craft pcap or inject packets with stale timestamps).
2. Capture-environment-controlled (merged captures, pcapng interface clocks, NTP drift).

Neither `wrapping_sub` nor `saturating_sub` nor `max_seen_ts` clamping can distinguish
(1) from (2). All three operate on whatever timestamp the pcap delivers. The operational
implication:

- `saturating_sub` (Option A / current ENIP/DNP3 policy): an attacker injecting one
  backwards packet cannot abort a detection window. Cost: a legitimate clock rollback
  (e.g., merged captures from different epoch bases) will pin the window open indefinitely
  until forward-progressing timestamps exceed the stale anchor. This is acceptable for
  ICS PCAP analysis where capture time bases are generally controlled.
- `max_seen_ts` clamping (Options B/C): same adversarial property as `saturating_sub`,
  slightly better behavior for legitimate partial reorder (a few out-of-order packets
  within a pcap due to interface jitter), same caveat for epoch-base mismatches.

Neither approach validates timestamp monotonicity against wall-clock time. The system
trusts the pcap timestamp as ground truth for ordering, which is the correct assumption
for offline PCAP analysis tools.

---

## 7. Open Questions for Human Review

1. **ARP storm denominator policy** (Phase 2): should the storm detector suppress
   same-second rate-based emission, require a minimum 2s window before computing rate,
   or change to a pure count-based threshold? This determines which BC-2.16.008 amendment
   is needed. Current ARP-AMB-003 RESOLVED disposition may need reopening.

2. **Modbus comment update**: the mandate "MUST use `wrapping_sub`" at modbus.rs:112 was
   correct under f2-fix-directives §11.5b (the old policy). After the EC-X2 ruling,
   `saturating_sub` is the new standard. Should the Modbus comment simply be updated to
   match, or is there a Modbus-specific reason to keep `wrapping_sub` not captured in the
   existing comment? (No such reason was found during this hunt.)

3. **`WindowClock` timing**: if v0.12.0 scope expands to include DNP3 carry fix AND Modbus
   arithmetic fix, the combined struct-touching work may justify pulling `WindowClock` into
   v0.12.0 rather than deferring. Human/architect should decide scope before story
   decomposition.
