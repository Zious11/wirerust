---
document_type: arch-ruling
ruling_id: RULING-DNP3-DESYNC-001
cycle: feature-enip-v0.11.0
wave: Wave-64
author: architect
date: 2026-06-28
status: authoritative
supersedes: []
addendum_to: RULING-DNP3-SIBLING-001
bcs_amended:
  - BC-2.15.009
adrs_amended: []
vps_recommended: []
release_blocker: true
release_held: v0.11.0
human_approved: false
last_amended: 2026-06-28
amendment_ref: ADDENDUM-2026-06-28-frame-count-guard
---

# RULING-DNP3-DESYNC-001: DNP3 is_non_dnp3 Desync-Latch Direction-Contamination Bug

## 0. Executive Summary

After STORY-140 splits `Dnp3FlowState.carry` into `carry_c2s` / `carry_s2c` and introduces
`direction: Direction` into `Dnp3Analyzer::on_data`, a latent correctness bug in the
first-delivery desync check at `dnp3.rs:363` becomes a confirmed defect. The check:

```rust
if active_carry!(flow, direction).is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

tests ONLY the carry of the CURRENT delivery's direction. After the STORY-140 carry
split, this means a junk first delivery in the `ServerToClient` direction on a flow that
has already accepted legitimate `ClientToServer` DNP3 frames will latch `is_non_dnp3 = true`,
permanently silencing the established c2s direction.

This is documented as a finding in DESIGN-CROSS-DIRECTION-STATE §2.1-§2.2. This ruling
adjudicates the binding fix and the BC amendment.

The fix is one-line and is delivered as STORY-142 (small: one code change + one regression
test + BC amendment).

---

## 1. Root Cause

### 1.1 The Desync Latch Condition (Post-STORY-140)

BC-2.15.009 specifies: "the bail fires only when `flow.carry.is_empty() && data.len() >= 2
&& data[0..2] != [0x05, 0x64]`." The intent is: only fire on the genuinely first-ever
delivery to an unestablished flow.

Before STORY-140, `flow.carry` was a single shared buffer. `carry.is_empty()` meant
"no bytes have been accepted into carry from either direction." This correctly proxied
"the flow is unestablished."

After STORY-140, `active_carry!(flow, direction)` selects `carry_c2s` or `carry_s2c` by
direction. `active_carry!(...).is_empty()` now means "no bytes have been accepted in THIS
direction's carry." This is a different question.

**Concrete failure scenario** (from DESIGN-CROSS-DIRECTION-STATE §2.1):

1. Flow established: first delivery `direction=ClientToServer`, data=valid DNP3 sync word.
   - `carry_c2s.is_empty()` = true; data[0..2] == `[0x05, 0x64]` → latch check does NOT fire.
   - Frame walk runs; `carry_c2s` accumulates bytes; `frame_count += 1`.
   - `carry_s2c` remains empty.

2. Second delivery: `direction=ServerToClient`, data = non-DNP3 junk (port-reuse event in
   a merged pcap, or a TLS ClientHello from a different session sharing the same 4-tuple FlowKey).
   - `active_carry!(flow, ServerToClient)` = `carry_s2c.is_empty()` = **true** (never written).
   - `data[0..2] != [0x05, 0x64]` (junk bytes).
   - **`flow.is_non_dnp3 = true`** — the entire flow is permanently silenced.

3. All subsequent `on_data` calls (including further legitimate c2s DNP3 frames) return
   immediately at the `if flow.is_non_dnp3 { return; }` bail at the top of `on_data`.
   The established c2s direction is dead.

**Impact:** Any capture environment where:
- FlowKey aliases two different physical sessions (merged pcap, multi-interface capture), OR
- A DNP3-over-non-standard-port flow receives a single non-DNP3 response packet first

will produce a permanently silenced flow, suppressing ALL DNP3 detection for that flow.

### 1.2 Why RULING-EDGECASE-001 and RULING-DNP3-SIBLING-001 Did Not Catch This

RULING-EDGECASE-001 §1.3 classification for `is_non_enip: PER-FLOW (keep shared)` states:
> "If the flow is not ENIP, both directions are not ENIP. A per-direction latch would require
> both to fire before the flow is abandoned, which is incorrect."

RULING-DNP3-SIBLING-001 §1.3 analogously classified `is_non_dnp3: PER-FLOW (keep shared)`.

This reasoning was correct for ENIP (`is_non_enip` fires on carry-cap overflow, not a
sync-word check — the ENIP failure mode does not exist; see DESIGN-CROSS-DIRECTION-STATE §3.3).

For DNP3, the reasoning is also correct in the abstract ("if the flow is not DNP3, both
directions are not DNP3") — but the IMPLEMENTATION of the desync check uses the directional
carry as a proxy for "flow is unestablished." After the carry split, that proxy breaks.
The latch condition must be widened to ask "are BOTH directional carries empty?" to correctly
proxy the original intent.

---

## 2. Binding Fix

### 2.1 Complete Desync-Latch Predicate (Amended 2026-06-28)

**Adopted: DESIGN-CROSS-DIRECTION-STATE §2.2 Option A, amended to include `frame_count == 0` guard.**

Change `dnp3.rs:363` from:

```rust
if active_carry!(flow, direction).is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

to:

```rust
if flow.frame_count == 0
    && flow.carry_c2s.is_empty()
    && flow.carry_s2c.is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

**Complete predicate (canonical):**
`flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`

**What changed from the initial ruling:** The initial adjudication adopted only the
both-carries-empty condition (replacing the single `active_carry!` check). Sub-case ii
analysis (see §2.3 and ADDENDUM-2026-06-28-frame-count-guard) revealed that this is still
INCOMPLETE: a completed c2s frame drains `carry_c2s` to empty, so a subsequent junk s2c
delivery finds both carries empty and still latches `is_non_dnp3`, silencing the established
c2s direction. The `frame_count == 0` guard is the correct and complete fix.

**Why `frame_count == 0` is the right proxy:** `frame_count` is incremented exactly once
per successfully parsed complete DNP3 frame in any direction. Once `frame_count >= 1`, the
flow has unconditionally established DNP3 parsing in at least one direction. The latch must
never fire on an established flow regardless of carry state. `frame_count == 0` precisely
captures "the flow has NEVER successfully parsed a frame in either direction" — i.e., the
flow is genuinely unestablished. The both-carries-empty condition alone cannot make this
determination because carries are transient (drained to empty after each complete frame).

### 2.2 Why Not Option B (Per-Direction is_non_dnp3 flags)

DESIGN-CROSS-DIRECTION-STATE §2.2 Option B would split `is_non_dnp3: bool` into
`is_non_dnp3_c2s: bool` and `is_non_dnp3_s2c: bool`. This is more complex (two fields,
two check paths, two latch paths) and is semantically wrong for the "if the flow is not
DNP3, both directions are not" principle. Option A achieves the fix with a single
predicate change, no new fields, and no BC-level structural changes beyond the
first-delivery condition text.

### 2.3 Correctness Analysis of the Fix

**Case 1: Genuine non-DNP3 flow (c2s first, junk bytes).**
- First delivery: `direction=ClientToServer`, `carry_c2s.is_empty()` = true,
  `carry_s2c.is_empty()` = true, data[0..2] != `[0x05,0x64]` → condition fires →
  `is_non_dnp3 = true`. Correct — the flow is immediately latched.

**Case 2: Established c2s DNP3 flow, junk s2c delivery.**
- After c2s frames: `carry_c2s` may be empty (if frames were complete and drained)
  OR non-empty (partial frame stashed). `carry_s2c.is_empty()` = true.
- `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()` = depends on whether c2s
  carry is empty.
  - If `carry_c2s` is non-empty (partial c2s frame in flight): BOTH are NOT empty →
    condition does not fire → flow NOT latched. Correct.
  - If `carry_c2s` is empty (complete c2s frames were consumed): BOTH are empty →
    condition DOES fire on the junk s2c delivery.

**Edge case in Case 2 / carry_c2s empty after clean c2s frames:** This is the
scenario where Option A's widened condition still fires. Is this acceptable?

Analysis: if `carry_c2s` is empty after successful c2s parsing, the c2s stream has
cleanly consumed all buffered bytes. The flow has had legitimate c2s traffic. A junk
s2c delivery while carry_c2s is empty would still latch the flow.

However, this is the SAME behavior as the pre-STORY-140 single-carry model: with a
single shared carry, once a complete c2s frame is consumed, carry is empty — and any
junk s2c delivery on an empty carry would trigger the latch under the original condition.

**Sub-case ii (the common request→response lifecycle) — root cause of the initial
incomplete fix:**

A normal DNP3 session proceeds: c2s request frame → s2c response. After the c2s request
frame is fully parsed and consumed, `carry_c2s` is drained to empty. When the s2c
response arrives, if it is non-DNP3 (e.g., port-reuse junk, TLS ClientHello aliased on
the same 4-tuple), the both-carries-empty condition fires:
- `flow.frame_count >= 1` (c2s frame was successfully parsed) — BUT the initial
  both-carries-empty-only fix has no guard on this.
- `carry_c2s.is_empty()` = true (drained after clean parse).
- `carry_s2c.is_empty()` = true (never written).
- Both-carries-empty alone → latch fires → established c2s direction silenced.

This is the **sub-case ii** failure: the both-carries-empty condition was INCOMPLETE.
It correctly widened the original bug (which only tested the active direction's carry)
but left this transient-carry-state gap. Sub-case ii is the COMMON case in real
request→response traffic.

**Complete analysis with the `frame_count == 0` guard:**

- **Case 1: Genuine non-DNP3 flow (c2s first, junk bytes).**
  `frame_count == 0` (no frames ever parsed), `carry_c2s.is_empty()` = true,
  `carry_s2c.is_empty()` = true, `data[0..2] != [0x05, 0x64]` → all four conditions
  true → latch fires → `is_non_dnp3 = true`. Correct.

- **Case 2 (sub-case i): Established c2s DNP3 flow, junk s2c, partial c2s in carry.**
  `frame_count >= 1`, `carry_c2s` non-empty (partial c2s frame buffered).
  `frame_count == 0` is FALSE → latch does not fire. Correct.

- **Case 2 (sub-case ii): Established c2s DNP3 flow, junk s2c, carry_c2s drained.**
  `frame_count >= 1` (c2s frame was successfully parsed). `carry_c2s.is_empty()` = true.
  `carry_s2c.is_empty()` = true. BUT `frame_count == 0` is FALSE → latch does not fire.
  Correct. This is the case the initial both-carries-empty-only fix missed.

**Conclusion:** The `frame_count == 0` guard is load-bearing and not redundant with the
both-carries-empty check. Once `frame_count >= 1` the flow is unconditionally established
and the latch never fires regardless of carry state. This eliminates all sub-cases of
direction contamination for established flows.

**Enhancement path (v0.12.0 — now lower priority):** The `frame_count == 0` guard makes
the "unestablished" proxy precise. The previously noted `first_c2s_frame_seen` /
`first_s2c_frame_seen` flag approach is superseded by the `frame_count` guard and is no
longer needed.

---

## 3. BC-2.15.009 Amendment

**BC-2.15.009: is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync**

**Version bump: 1.6 → 2.0** (semantic change to the bail precondition).

1. **Precondition 3 (bail condition):** Change from:
   > `flow.carry.is_empty() && data.len() >= 2 && data[0] != 0x05 || data[1] != 0x64`

   to (complete predicate — incorporates `frame_count == 0` guard, amended 2026-06-28):
   > `flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() &&`
   > `data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`

   Rationale: after the carry split (STORY-140 / RULING-DNP3-SIBLING-001), the bail
   condition must check BOTH directional carries AND `frame_count == 0` to correctly proxy
   "the flow has never successfully parsed a frame in either direction." The both-carries-empty
   condition alone is INCOMPLETE: carries are transiently empty after each complete frame
   is consumed, so a subsequent junk delivery in the other direction (sub-case ii — the
   common request→response lifecycle) would still latch `is_non_dnp3` and silence the
   established direction. The `frame_count == 0` guard ensures the latch fires only on a
   genuinely unestablished flow. See RULING-DNP3-DESYNC-001 §2.1 and
   ADDENDUM-2026-06-28-frame-count-guard.

2. **Description paragraph 2:** Add after "Once any bytes have been accepted into carry
   the flow is established":
   > After the STORY-140 carry split and the sub-case ii correction
   > (ADDENDUM-2026-06-28-frame-count-guard), the bail fires only when ALL of the
   > following are true: `flow.frame_count == 0` (no frame has ever been successfully
   > parsed in any direction), `carry_c2s.is_empty()` AND `carry_s2c.is_empty()` (no
   > in-flight carry bytes in either direction), `data.len() >= 2`, and the data does
   > not begin with the DNP3 sync word. Once `frame_count >= 1` the flow is
   > unconditionally established and the latch never fires regardless of carry state.
   > RULING-DNP3-DESYNC-001.

3. **Add new Edge Cases (direction-isolation desync) — amended 2026-06-28 to add EC-012:**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-010 | First delivery: `direction=ClientToServer`, valid DNP3 sync, `carry_c2s` accumulates 6 bytes (partial frame). `frame_count=0`. Second delivery: `direction=ServerToClient`, non-DNP3 junk. | `frame_count==0` but `carry_c2s.is_empty()=false` → bail condition does NOT fire → `is_non_dnp3` remains false → c2s stream continues processing. |
   | EC-011 | First delivery: `direction=ClientToServer`, non-DNP3 junk. `frame_count=0`, both carries empty. | `frame_count==0`, `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true`, junk bytes → bail fires → `is_non_dnp3=true`. Correct: genuinely unestablished flow. |
   | EC-012 | First delivery: `direction=ClientToServer`, valid DNP3 sync, complete frame consumed, `carry_c2s` drained to empty, `frame_count=1`. Second delivery: `direction=ServerToClient`, non-DNP3 junk. | `frame_count==1` (≥1) → bail condition does NOT fire → `is_non_dnp3` remains false → c2s stream continues. **Sub-case ii — requires `frame_count==0` guard. Would latch under both-carries-empty-only condition.** |

4. **Architecture Anchors:** Update the carry field reference from `flow.carry` to
   `flow.carry_c2s` / `flow.carry_s2c` (consistent with RULING-DNP3-SIBLING-001 §5.1
   BC-2.15.016 amendments).

---

## 4. STORY-142 Scope and Acceptance Criteria

**Story title:** "Fix DNP3 is_non_dnp3 desync-latch direction-contamination (RULING-DNP3-DESYNC-001)"

**Scope:** Small story. One-line code change in `dnp3.rs`. One regression test. BC-2.15.009
amendment.

**Precondition:** STORY-140 must be merged (the `active_carry!` macro and `carry_c2s`/
`carry_s2c` fields must exist). STORY-142 is a follow-on to STORY-140.

**Acceptance Criteria (amended 2026-06-28 — `frame_count==0` guard + `test_ac142_004`):**

1. `dnp3.rs:363` desync-latch condition is changed to the COMPLETE predicate:
   `flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`.
   The `frame_count == 0` guard is added alongside the both-carries-empty check; the rest
   of the condition (`data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`) is unchanged.

2. **`test_ac142_001` — partial-c2s-carry preserves direction on junk-s2c (sub-case i):**
   - Step 1: Deliver partial c2s DNP3 frame (valid sync bytes `[0x05, 0x64, ...]` but incomplete). `carry_c2s` is non-empty, `frame_count=0`. `is_non_dnp3` remains false.
   - Step 2: Deliver non-DNP3 junk bytes in `direction=ServerToClient` (e.g., `[0xFF, 0xFE, 0x00]`). Assert: `flow.is_non_dnp3 == false` (latch did NOT fire because `carry_c2s` is non-empty).
   - Step 3: Complete the c2s partial frame in `direction=ClientToServer`. Assert: `frame_count == 1`, `parse_errors == 0`.
   This test is RED against the buggy post-STORY-140 code and GREEN after the fix.

3. **`test_ac142_002` — true non-DNP3 flow latches immediately:**
   First delivery in c2s direction with non-DNP3 junk (`frame_count=0`, both carries empty):
   latch fires, `is_non_dnp3 = true`. No regression from the fix.

4. **`test_ac142_003` — established c2s flow with established non-DNP3 s2c:**
   Not a new regression: existing tests covering `frame_count >= 1` behavior continue to
   pass (the `frame_count == 0` guard changes nothing for flows that have already parsed
   at least one frame).

5. **`test_ac142_004` — complete-frame then junk-s2c, sub-case ii (NEW — requires `frame_count==0` guard):**
   - Step 1: Deliver a complete c2s DNP3 frame (valid sync bytes, full frame). `carry_c2s`
     is drained to empty after parse. `frame_count == 1`.
   - Step 2: Deliver non-DNP3 junk bytes in `direction=ServerToClient` (e.g., `[0xFF, 0xFE, 0x00]`).
     At this point: `frame_count=1`, `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true`.
   - Assert: `flow.is_non_dnp3 == false` (latch did NOT fire because `frame_count >= 1`).
   - Assert: `frame_count == 1` (unchanged — junk s2c delivery did not decrement it).
   This test is RED against the both-carries-empty-only fix and GREEN only with the
   `frame_count == 0` guard. This is the sub-case ii regression test.

6. BC-2.15.009 is amended per §3 above (version bump 1.6 → 2.0, precondition 3 updated to
   include `frame_count == 0` guard, EC-010/EC-011/EC-012 added).

5. All existing DNP3 tests pass.

---

## 5. Spec-Steward Actions Required

| BC | Old version | New version | Input-hash impact |
|----|------------|------------|-------------------|
| BC-2.15.009 | 1.6 | 2.0 | STALE on version bump |

**Actions:**

1. Bump version in frontmatter of BC-2.15.009 from 1.6 to 2.0.
2. Add `modified:` entry citing this ruling.
3. Run `bin/compute-input-hash --scan` after BC edit.
4. Add this ruling to `decisions-archive.md` under the `feature-enip-v0.11.0` cycle.

Note: no new VPs are recommended for this fix (the existing VP-NEW-C from RULING-DNP3-SIBLING-001
covers carry-direction isolation; the desync-latch fix is a one-line predicate change
with a targeted regression test, not a new property class).

---

## 6. Summary for Implementer and Test-Writer

**Implementer (amended 2026-06-28 — complete predicate):**
- In `dnp3.rs`, find the desync-latch block (currently at line 363, post-STORY-140).
- Replace the entire condition with the complete predicate:
  `flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`
- No other logic changes. The bail body (`flow.is_non_dnp3 = true; return;`) is unchanged.
- The `frame_count` field must already exist on `Dnp3FlowState` after STORY-140; confirm
  it is incremented on each successful complete-frame parse.

**Test-writer (amended 2026-06-28 — add `test_ac142_004`):**
- `test_ac142_001`: partial c2s sync bytes → non-DNP3 junk s2c → assert `is_non_dnp3 == false`, c2s processing continues (sub-case i, partial-carry guard).
- `test_ac142_002`: true non-DNP3 flow — first delivery junk c2s, both carries empty, `frame_count=0` → latch fires.
- `test_ac142_003`: existing established-flow tests continue to pass (no regression).
- `test_ac142_004` (NEW — sub-case ii): complete c2s frame consumed → `carry_c2s` drained, `frame_count=1` → non-DNP3 junk s2c → assert `is_non_dnp3 == false`. This test is RED under the both-carries-empty-only condition and GREEN only with the `frame_count == 0` guard. Name: `test_ac142_004` (or `test_dnp3_desync_latch_complete_frame_then_junk_s2c`).

---

## ADDENDUM-2026-06-28: frame_count==0 Guard — Sub-case ii Correction

**Date:** 2026-06-28
**Author:** architect
**Supersedes:** The §2.1 both-carries-empty-only binding decision from the initial ruling.

### What Changed

The initial adjudication of RULING-DNP3-DESYNC-001 adopted the both-carries-empty condition
(`flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`) as the complete fix. This
addendum supersedes that decision. The both-carries-empty-only condition is INCOMPLETE.

### Root Cause of the Incomplete Fix

Carries are transient buffers: they accumulate bytes for a partial frame and are drained to
empty when a complete frame is consumed. After a successful c2s frame parse, `carry_c2s` is
empty again. This is the normal, correct behavior — not an error condition. The consequence
is that the both-carries-empty condition fires not only on a genuinely unestablished flow,
but also on an established flow at the moment when all frames are cleanly consumed.

Sub-case ii: c2s request → s2c junk (the COMMON request→response lifecycle pattern)
1. c2s request frame delivered and fully parsed. `frame_count` incremented to 1. `carry_c2s`
   drained to empty.
2. s2c junk delivery arrives (e.g., port-reuse, protocol mismatch on a shared 4-tuple).
   State at latch check: `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true`.
3. Both-carries-empty-only condition: TRUE → `is_non_dnp3 = true` → established c2s direction
   silenced. WRONG.

This is not a corner case. It is the normal pattern for any DNP3 flow that processes at
least one complete request before receiving an anomalous response.

### The Correct Fix

Add `flow.frame_count == 0` as a required precondition. `frame_count` is incremented on every
successful complete-frame parse in any direction. Once `frame_count >= 1`, the flow is
unconditionally established and the latch must never fire.

**Complete canonical predicate (this supersedes all prior versions):**
`flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`

### BC-2.15.009 Amendment Required

BC-2.15.009 must add `frame_count == 0` as a latch precondition. The product owner will
amend BC-2.15.009 to reflect this. The architect flags this as a required PO action before
STORY-142 can be marked implementation-ready. The BC version target remains 2.0 (this
addendum does not change the version number; the semantic scope of the version bump is
already correct — it covers the full predicate change).

### Impact on STORY-142 Acceptance Criteria

AC-1 is updated to require the complete predicate. A new test `test_ac142_004` is added
to cover sub-case ii as a regression test (RED under both-carries-empty-only, GREEN under
the complete predicate). See §4 for the full updated AC list.
