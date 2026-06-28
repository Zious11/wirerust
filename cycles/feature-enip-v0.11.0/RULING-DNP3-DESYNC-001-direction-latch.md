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

### 2.1 Option A: Both-Carries-Empty Condition

**Adopted: DESIGN-CROSS-DIRECTION-STATE §2.2 Option A.**

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
if flow.carry_c2s.is_empty()
    && flow.carry_s2c.is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

**Exact change:** Replace `active_carry!(flow, direction).is_empty()` with
`flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`.

This is a one-line change to the condition. The latch now fires only when BOTH direction
carries are empty — i.e., when this is the genuinely first-ever delivery to an unestablished
flow in any direction.

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

**Conclusion:** Option A preserves the same semantics as the pre-STORY-140 single-carry
model for the case where carry_c2s is drained. The additional protection it provides
over the current (post-STORY-140 buggy) behavior is specifically for the scenario where
a junk s2c delivery arrives WHILE a partial c2s frame is in carry_c2s. In that case:
- Buggy (post-STORY-140): carry_s2c.is_empty() → latch fires → established c2s silenced.
- Fixed (Option A): carry_c2s.is_non_empty() → BOTH not empty → latch does not fire.

This is the material correctness improvement: a mid-session junk s2c packet cannot veto
an established c2s stream that currently has a partial frame buffered.

For scenarios where carry_c2s is transiently empty between clean c2s frames, Option A
behaves identically to the pre-split single-carry model. This is an acceptable
residual limitation and matches the existing BC-2.15.009 semantics.

**Enhancement path (v0.12.0 follow-up):** Track `first_c2s_frame_seen: bool` and
`first_s2c_frame_seen: bool` flags to make the "unestablished" proxy more precise.
This is out of scope for the Wave 64 one-line fix.

---

## 3. BC-2.15.009 Amendment

**BC-2.15.009: is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync**

**Version bump: 1.6 → 2.0** (semantic change to the bail precondition).

1. **Precondition 3 (bail condition):** Change from:
   > `flow.carry.is_empty() && data.len() >= 2 && data[0] != 0x05 || data[1] != 0x64`
   
   to:
   > `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 &&`
   > `(data[0] != 0x05 || data[1] != 0x64)`

   Rationale: after the carry split (STORY-140 / RULING-DNP3-SIBLING-001), the bail
   condition must check BOTH directional carries to correctly proxy "the flow is
   unestablished in any direction." Checking only the active direction's carry allows a
   junk delivery in one direction to latch the flow even when the other direction has
   established carry buffering. See RULING-DNP3-DESYNC-001 §1.1.

2. **Description paragraph 2:** Add after "Once any bytes have been accepted into carry
   the flow is established":
   > After the STORY-140 carry split, "accepted into carry" means bytes accepted into
   > EITHER `carry_c2s` OR `carry_s2c`. The bail fires only when BOTH are empty AND
   > the current delivery has no sync word. If `carry_c2s` is non-empty (partial c2s
   > frame in flight), a junk s2c delivery does NOT latch `is_non_dnp3`.
   > RULING-DNP3-DESYNC-001.

3. **Add new Edge Case (direction-isolation desync):**

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-010 | First delivery: `direction=ClientToServer`, valid DNP3 sync, `carry_c2s` accumulates 6 bytes (partial frame). Second delivery: `direction=ServerToClient`, non-DNP3 junk. | `carry_c2s.is_empty()=false` → bail condition does NOT fire → `is_non_dnp3` remains false → c2s stream continues processing |
   | EC-011 | First delivery: `direction=ClientToServer`, valid DNP3 sync, complete frame consumed, `carry_c2s` drained to empty. Second delivery: `direction=ServerToClient`, non-DNP3 junk. | `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true` → bail fires → `is_non_dnp3=true`. Same behavior as pre-STORY-140 single-carry model. |

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

**Acceptance Criteria:**

1. `dnp3.rs:363` desync-latch condition is changed from `active_carry!(flow, direction).is_empty()` to `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`. The rest of the condition (`data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`) is unchanged.

2. **New regression test (established-c2s-direction preserved on junk-s2c):**
   - Step 1: Deliver partial c2s DNP3 frame (valid sync bytes `[0x05, 0x64, ...]` but incomplete). `carry_c2s` is non-empty. `is_non_dnp3` remains false.
   - Step 2: Deliver non-DNP3 junk bytes in `direction=ServerToClient` (e.g., `[0xFF, 0xFE, 0x00]`). Assert: `flow.is_non_dnp3 == false` (latch did NOT fire because carry_c2s is non-empty).
   - Step 3: Complete the c2s partial frame with the remaining bytes in `direction=ClientToServer`. Assert: `frame_count == 1`, `parse_errors == 0`.
   This test is RED against the buggy post-STORY-140 code and GREEN after the fix.

3. **Existing regression test (true non-DNP3 flow latches immediately) still passes:**
   First delivery in c2s direction with non-DNP3 junk (both carries empty): latch fires, `is_non_dnp3 = true`. No regression from the fix.

4. BC-2.15.009 is amended per §3 above (version bump 1.6 → 2.0, precondition 3 updated, EC-010/EC-011 added).

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

**Implementer:**
- In `dnp3.rs`, find the desync-latch block (currently at line 363, post-STORY-140).
- Change `active_carry!(flow, direction).is_empty()` to `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`.
- No other logic changes. The rest of the bail body (`flow.is_non_dnp3 = true; return;`) is unchanged.

**Test-writer:**
- Write AC-2 regression test: partial c2s sync bytes → non-DNP3 junk s2c → assert `is_non_dnp3 == false` and c2s processing continues.
- Verify the existing "true non-DNP3 flow" test (AC-3) still passes (both carries empty → latch fires).
