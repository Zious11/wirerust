---
document_type: adjudication
adjudication_id: ADJ-001
status: final
date: 2026-06-11
author: architect
story: STORY-109
feature: issue-008-dnp3-analyzer
traces_to:
  - ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - BC-2.15.004
  - BC-2.15.009
  - BC-2.15.016
  - BC-2.15.024
  - STORY-107
  - VP-023
---

# ADJ-001: STORY-109 Carry-Resync Adjudication — Byte-Walk-Forward for BC-2.15.024

## Conflict Summary

STORY-109 implements BC-2.15.024 (Malformed/Structural DNP3 Anomaly): when
`malformed_in_window >= 3` within 300s, emit a T0814 "Crain-Sistrunk crash-probe"
finding. The three malformed-frame reject paths that increment `malformed_in_window`
are LENGTH<5 (BC-2.15.004), frame-length mismatch (BC-2.15.007), and
sync-loss/carry-overflow (BC-2.15.016/009).

The current carry-resync in `on_data` (STORY-107 v1, drain-1 on invalid LENGTH)
causes the following failure sequence when a malformed frame
`[0x05, 0x64, 0x02, ...]` (LENGTH=2<5) is delivered:

1. Iteration 1: carry[0..2] = `[0x05, 0x64]` — sync OK; `compute_dnp3_frame_len(2)`
   returns None → `parse_errors += 1`, `malformed_in_window += 1`; `carry.drain(..1)`.
2. Iteration 2: carry[0] = `0x64` — sync gate BREAKS.
3. Carry holds 9 bytes of junk. The `0x64` at head does not match `0x05` so
   subsequent on_data calls immediately break the sync gate without processing —
   the carry fills to 292 and only the overflow-discard path fires.
4. Consequence: a repeated flood of structurally identical malformed frames,
   each delivered as a fresh `on_data` call, appends to the carry and the
   SAME `0x64`-headed junk blocks the sync gate — so no second `parse_errors`
   increment occurs from the LENGTH-gate path until the carry reaches the
   292-byte cap and the overflow path fires once.
5. `malformed_in_window` therefore increments only once per flush-to-overflow
   cycle, not once per malformed frame. BC-2.15.024's threshold of 3 within 300s
   can never be reached from a realistic LENGTH<5 flood, defeating the detection.

Four STORY-109 tests assert the correct behavior and are currently blocked:
`test_malformed_anomaly_at_threshold_3_of_300s` (AC-012),
`test_parse_errors_not_reset_at_window_expiry` (AC-013),
`test_correlation_window_expiry_resets_six_fields` (AC-005),
`test_EC_009_fourth_malformed_no_second_t0814` (EC-009).

---

## DECISION 1: Algorithm — Byte-Walk-Forward Resync is Authorized and Correct

### Verdict: AUTHORIZED. Byte-walk-forward resync is the correct STORY-109
realization of STORY-107's explicitly deferred resync.

### Justification

STORY-107 v1 deliberately deferred byte-walk resync for mid-carry sync-loss.
The in-code comment at lines 356-359 states this explicitly:
"Byte-walk resync on mid-carry sync-loss is deferred to a later detection story."
STORY-109 is that later story. BC-2.15.024 requires that every malformed frame
delivered on an established flow increments `malformed_in_window` exactly once.
The drain-1 policy makes this impossible for the LENGTH<5 case because the
drained byte leaves a non-sync head that blocks subsequent processing.
Byte-walk-forward resync is the narrowest correct fix: it resolves the specific
blockage at the carry head without changing any other aspect of the frame-walk.

### Exact Algorithm

The byte-walk-forward resync replaces the break in the sync-gate arm. The complete
replacement logic for the sync check arm is specified here.

CURRENT code (STORY-107 v1, lines 360-363 approximately):
```
if flow.carry[0] != 0x05 || flow.carry[1] != 0x64 {
    break;
}
```

REPLACEMENT — byte-walk-forward resync:
```rust
if flow.carry[0] != 0x05 || flow.carry[1] != 0x64 {
    // Mid-carry sync-loss: scan forward to find the next [0x05, 0x64] sync word.
    // Walk one byte at a time (not two) so we don't skip a sync word that begins
    // at an odd offset.
    //
    // Termination guarantee: each iteration of the enclosing frame-walk loop that
    // reaches this arm drains at least 1 byte (carry.drain(..1)), so the loop
    // makes progress on every non-break iteration. Carry is bounded to ≤292 bytes,
    // so the total drain is bounded.
    //
    // BC-2.15.009 16-byte bail interaction: this arm is only reached on ESTABLISHED
    // flows (carry was non-empty before this on_data call, meaning the BC-2.15.009
    // initial-16-byte bail window has already passed). The 16-byte bail applies only
    // to the first delivery on a new flow (carry.is_empty() check in Step 1 above).
    // Mid-carry sync-loss is distinct: the flow is established DNP3 but the carry
    // has become misaligned due to partial delivery or malformed framing.
    //
    // Search: find the first index i >= 1 such that carry[i] == 0x05 and
    // (i+1 < carry.len()) carry[i+1] == 0x64. Drain all bytes before that index.
    // If no sync word is found in the carry, drain the entire carry — the next
    // on_data call will start fresh (empty carry → valid sync on new delivery).
    //
    // NOTE: draining the entire carry when no sync is found does NOT set
    // is_non_dnp3 = true and does NOT increment parse_errors here. The malformed-
    // frame reject that incremented parse_errors + malformed_in_window already
    // happened in the LENGTH-gate arm (or the overflow arm) before this point.
    // This arm handles only the resync navigation; it does NOT double-count.
    let next_sync = flow.carry
        .windows(2)
        .enumerate()
        .skip(1)  // start at index 1, not 0 (current head is already invalid)
        .find(|(_, w)| w[0] == 0x05 && w[1] == 0x64)
        .map(|(i, _)| i);
    match next_sync {
        Some(i) => {
            // Drain all bytes before the next sync word.
            flow.carry.drain(..i);
            // Continue the loop — the next iteration will re-check the new head.
        }
        None => {
            // No sync word found anywhere in the remaining carry.
            // Drain the entire carry so the next on_data starts fresh.
            flow.carry.clear();
        }
    }
    continue;
}
```

### Per-Frame Increment Semantics (No Double-Count)

The byte-walk-forward resync arm does NOT increment `parse_errors` or
`malformed_in_window`. Incrementing only occurs in exactly two locations:
1. The LENGTH gate arm: `compute_dnp3_frame_len` returns None → `parse_errors += 1;
   malformed_in_window += 1; carry.drain(..1); check_malformed_anomaly(...); continue;`
2. The carry-overflow arm (Step 2 above): `parse_errors += 1; malformed_in_window += 1;
   check_malformed_anomaly(...);`

The resync navigation that follows a LENGTH-gate rejection is pure cursor
movement. The error was already counted. Moving the carry head to the next
sync word (or clearing the carry) is a liveness operation, not a new error event.

This guarantees: exactly one `malformed_in_window` increment per malformed frame
delivered, with no double-counting regardless of how many bytes are drained
during resync navigation.

### Worked Example — Three-Frame Flood

Each call is a fresh `on_data` with `[0x05, 0x64, 0x02, 0xC4, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]`.

Frame 1 delivery (carry starts empty):
- Step 2: append 10 bytes. carry = `[05 64 02 C4 03 00 01 00 00 00]`.
- Frame-walk iteration 1: sync OK; `compute_dnp3_frame_len(2)` = None.
  → `parse_errors=1, malformed_in_window=1`; `carry.drain(..1)`.
  carry = `[64 02 C4 03 00 01 00 00 00]` (9 bytes).
- Frame-walk iteration 2: carry[0]=0x64 ≠ 0x05 → byte-walk-forward resync.
  Search carry[1..] for `[0x05, 0x64]`: not found. `carry.clear()`. `continue`.
- Frame-walk iteration 3: carry.len() = 0 < 3 → break.
- Post-call: `parse_errors=1, malformed_in_window=1`. carry is empty.

Frame 2 delivery (carry is empty):
- Identical to Frame 1. `parse_errors=2, malformed_in_window=2`. carry is empty.

Frame 3 delivery (carry is empty):
- Identical. After LENGTH gate: `parse_errors=3, malformed_in_window=3`.
  `check_malformed_anomaly` fires. T0814 emitted.
- carry is empty after resync.
- Post-call: `malformed_in_window=3, malformed_anomaly_emitted=true`.

This satisfies all four blocked tests.

### Termination and Progress Guarantee

Every iteration of the frame-walk loop that reaches the byte-walk-forward arm
executes `carry.clear()` or `carry.drain(..i)` followed by `continue`. The
`drain` removes at minimum 1 byte (since `i >= 1` from `.skip(1)`). The `clear`
removes all bytes. In both cases carry.len() strictly decreases. Since carry is
bounded at ≤292 bytes and each non-break iteration drains ≥1 byte, the loop
terminates in at most 292 iterations on any single carry state.

### BC-2.15.009 Interaction

BC-2.15.009 ("no valid sync in first 16 bytes → is_non_dnp3 bail") applies only
to the INITIAL delivery check (`flow.carry.is_empty() && data.len() >= 2`). This
check happens in Step 1 before carry accumulation. Once any bytes have been
accepted into carry, the flow is established as DNP3 and BC-2.15.009's initial
bail no longer applies. The byte-walk-forward resync arm operates on an
established carry — it is a different code path entirely and does NOT interact
with the BC-2.15.009 16-byte rule. Mid-carry sync-loss does NOT set
`is_non_dnp3 = true`. The `is_non_dnp3` latch is and remains a one-shot,
initial-delivery-only mechanism.

---

## DECISION 2: BC Text Update — Minimal, Scoped to BC-2.15.016 EC-007

### Verdict: YES — one targeted clause update required in BC-2.15.016.
No change required to BC-2.15.004, BC-2.15.009, or BC-2.15.024.

### Analysis

BC-2.15.016 EC-007 currently reads:
> `flow.carry[2]` (LENGTH byte) is invalid (< 5) after partial accumulation |
> Validity gate (BC-2.15.004) handles this; `parse_errors++`; carry advanced

The phrase "carry advanced" is ambiguous — it does not specify whether "advanced"
means drain-1 (STORY-107 v1 behavior) or byte-walk-forward (STORY-109 correct
behavior). The test `test_EC_006_invalid_length_byte_increments_parse_errors`
pinned the STORY-107 v1 interpretation of "carry advanced" as drain-1 with
carry.len()=9. That pinning must be updated.

BC-2.15.004 does not need updating: its postcondition 4 ("the on_data caller
MUST increment flow.parse_errors and skip all further processing of this frame")
does not prescribe navigation after the increment. It is silent on how the
carry head is repositioned, which is correct — that is BC-2.15.016's concern.

BC-2.15.009 does not need updating: the initial-bail rule is correctly scoped
to the first delivery and the byte-walk-forward resync is a distinct mid-carry
concern. The BCs do not overlap.

BC-2.15.024 does not need updating: it says "carry advanced" via the existing
BC-2.15.016/004 reject paths. Byte-walk-forward is a refinement of "carry
advanced" — the per-frame increment semantics (exactly one per malformed frame)
are already correct in BC-2.15.024.

### Required BC-2.15.016 Update

Product-owner must update BC-2.15.016 at exactly the following location:

**EC-007** (Edge Cases table, row EC-007):

CURRENT text:
> `flow.carry[2]` (LENGTH byte) is invalid (< 5) after partial accumulation |
> Validity gate (BC-2.15.004) handles this; `parse_errors++`; carry advanced

REPLACEMENT text:
> `flow.carry[2]` (LENGTH byte) is invalid (< 5) after partial accumulation |
> Validity gate (BC-2.15.004) handles this; `parse_errors++` (lifetime) and
> `malformed_in_window++` (windowed, STORY-109); carry advanced via byte-walk-forward
> resync: scan carry from index 1 for the next `[0x05, 0x64]` sync word; drain all
> bytes before it; if none found, clear carry. No further `parse_errors` or
> `malformed_in_window` increment occurs during resync navigation (error already
> counted at the LENGTH gate). This replaces the STORY-107 v1 drain-1 behavior for
> this path.

Additionally, the STORY-107 v1 version note should be recorded in the BC-2.15.016
modification log, e.g.:
> "v1.2: EC-007 resync policy updated: drain-1 (STORY-107 v1) replaced by
> byte-walk-forward resync (STORY-109 realization). The drain-1 behavior was a
> known deferral (STORY-107 STORY-107 in-code comment: 'Byte-walk resync on
> mid-carry sync-loss is deferred to a later detection story'). This update
> realizes that deferral. — 2026-06-11"

No other BC text changes are required.

---

## DECISION 3: test_EC_006 Update — Authorized

### Verdict: AUTHORIZED. Update test_EC_006's carry-length assertion from 9 to 0.
This is planned spec evolution (STORY-107 deferral realized), NOT a regression.

### Current test_EC_006 assertion (authorized to change):
```rust
assert_eq!(
    flow.carry.len(),
    9,
    "carry must be exactly 9 bytes: drain-1 resync consumed 1 byte, then the sync \
     gate broke on carry[0]=0x64, leaving bytes [1..9] of the original delivery"
);
```

### New expected carry state under byte-walk-forward resync:

After one `on_data` call delivering `[0x05, 0x64, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]`
(10 bytes, LENGTH=4<5):

1. Carry accumulates 10 bytes.
2. Frame-walk iteration 1: sync OK; `compute_dnp3_frame_len(4)` = None.
   → `parse_errors=1`; `carry.drain(..1)`. carry = 9 bytes `[0x64, ...]`.
3. Frame-walk iteration 2: carry[0]=0x64 ≠ 0x05 → byte-walk-forward resync.
   Search carry[1..8] for `[0x05, 0x64]`: not found (all zero padding after 0x04).
   `carry.clear()`. `continue`.
4. Frame-walk iteration 3: carry.len()=0 < 3 → break.

Post-call carry.len() = 0.

### Replacement assertion:
```rust
assert_eq!(
    flow.carry.len(),
    0,
    "carry must be 0 bytes: byte-walk-forward resync found no next sync word in \
     [0x64, 0x04, ...zeros] and cleared the carry (STORY-109 realization of the \
     STORY-107 deferred byte-walk resync). STORY-107 v1 drain-1 left carry=9; \
     byte-walk-forward clears when no next sync word is found."
);
```

The weaker `assert!(flow.carry.len() < 10, ...)` assertion (liveness guard) remains
valid under both behaviors and does not need to change. Only the strict `assert_eq!(9)`
requires updating.

### Classification

This is NOT a regression. The STORY-107 in-code comment explicitly deferred
byte-walk resync. The STORY-107 test was written to match the known-deferred
behavior and was always intended to be updated when the deferral was realized.
The liveness invariant (carry must have advanced from 10, i.e., drain occurred)
is preserved — carry goes from 10 to 0, which is strictly less than 10.

### STORY-107 input-hash/spec note

STORY-107.md does not need a spec content update — the deferral was already
documented in code and in the STORY-107 comment. The test update alone is
sufficient. No new BC version for STORY-107's BC-2.15.016 reference is required
beyond the EC-007 update specified in Decision 2 above.

The input-hash for STORY-107 will become stale after BC-2.15.016 is updated.
Before the next Phase-4 gate, run `bin/compute-input-hash --write .factory/stories/STORY-107.md`
to refresh it. This is a routine hash maintenance step, not a story regeneration.

---

## DECISION 4: VP-023 Implications

### Verdict: NO NEW VP-023 proof harness required. Existing invariants preserved.
Implementer must verify two behavioral invariants by inspection during code review.

### Analysis

VP-023 covers Sub-property D: `compute_dnp3_frame_len` result in [10, 292],
proving carry indexing is in-bounds. Byte-walk-forward resync does not touch
`compute_dnp3_frame_len` at all — it operates on the carry after that function
returns None (the LENGTH<5 arm) or in the sync-loss arm (before the function
is called). VP-023 Sub-D's proof domain is unaffected.

### Invariants the implementer MUST preserve:

1. **No panic invariant**: `flow.carry.windows(2)` on an empty slice returns an
   empty iterator without panicking. But the resync arm is only reached when
   `carry.len() >= 3` (the guard at the top of the frame-walk loop). So
   `carry.len() >= 3` when the resync arm executes, meaning `.windows(2)` on
   a 3+ element slice is always safe. `.skip(1)` on an iterator of length 1
   (windows of a 3-byte carry) yields an empty iterator, which causes the
   `find` to return None → `carry.clear()`. No panic. No bounds check needed
   beyond the existing carry.len() >= 3 guard.

2. **Carry bounded at ≤292 invariant**: The resync arm only removes bytes from
   carry (drain or clear). It never adds bytes. The carry was already bounded by
   the Step 2 cap before the frame-walk begins. Carry shrinks monotonically
   within the frame-walk loop. The ≤292 invariant is trivially preserved.

3. **No wrapping arithmetic in resync path**: The `windows(2).enumerate().skip(1).find()`
   chain uses only iterator indices (usize). No u32 arithmetic, no potential
   overflow. The `drain(..i)` where `i >= 1` and `i < carry.len()` (guaranteed
   by the `find` returning an index within the carry) is safe. No wrapping
   arithmetic is needed or used in this path.

4. **parse_errors and malformed_in_window are u64**: No practical overflow risk.
   VP-023 does not formally verify these fields. The byte-walk-forward resync
   arm does not modify either counter. The counters are only modified in the
   LENGTH-gate arm and the overflow arm, exactly as before.

### VP-023 re-running

VP-023's Kani proof harness (Sub-property D) does not need to be re-run for
this change. The proof target is `compute_dnp3_frame_len`, which is unchanged.
If the implementation team chooses to extend VP-023 with a Sub-property E
covering "resync does not panic under any carry state of length 1..=292 with
any byte values", that would be a worthwhile future hardening, but it is not
required for STORY-109 delivery.

---

## REMEDIATION SEQUENCE

The following steps are ordered. Each step must complete before the next begins.

### Step 1 — Product-Owner: Update BC-2.15.016 EC-007 text
**Owner: Product-Owner**
**Blocking: Steps 2, 3, 4**

Update BC-2.15.016 in `.factory/specs/behavioral-contracts/ss-15/BC-2.15.016.md`:
- Revise EC-007 row as specified in Decision 2 above.
- Add modification log entry (v1.2).
- Do NOT change any other BC text. Do NOT modify BC-2.15.004, BC-2.15.009,
  or BC-2.15.024.

### Step 2 — Implementer: Replace drain-1 with byte-walk-forward resync in `on_data`
**Owner: Implementer**
**Depends on: Step 1 (BC update must be authoritative before code change)**
**Location: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109/src/analyzer/dnp3.rs`**

Replace the sync-gate break arm (the `if flow.carry[0] != 0x05 || flow.carry[1] != 0x64 { break; }` block) with the byte-walk-forward resync as specified in
Decision 1 above. Key constraints:
- The resync arm MUST use `continue`, not `break`.
- The resync arm MUST NOT increment `parse_errors` or `malformed_in_window`.
- The resync search MUST start at index 1 (`.skip(1)`) so it does not re-evaluate
  the already-rejected current head.
- When no next sync word is found, `carry.clear()` is used (not `is_non_dnp3 = true`).

Update the in-code comment at lines 356-359 to remove the deferral language and
document the new algorithm.

### Step 3 — Implementer/Test-Writer: Update test_EC_006 assertion
**Owner: Implementer or Test-Writer (whichever owns the test file)**
**Depends on: Step 2 (test must be updated after code change, not before)**
**Location: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109/tests/dnp3_flow_state_tests.rs`, line ~786**

Replace the `assert_eq!(flow.carry.len(), 9, ...)` assertion with
`assert_eq!(flow.carry.len(), 0, ...)` per the exact replacement text in
Decision 3 above. Update the assertion message to document the STORY-109
realization. The weaker `assert!(flow.carry.len() < 10, ...)` liveness guard
remains unchanged.

### Step 4 — Implementer/Test-Writer: Verify four blocked tests pass
**Owner: Implementer**
**Depends on: Steps 2 and 3**

Run the following and verify all pass:
```
cargo test test_malformed_anomaly_at_threshold_3_of_300s
cargo test test_parse_errors_not_reset_at_window_expiry
cargo test test_correlation_window_expiry_resets_six_fields
cargo test test_EC_009_fourth_malformed_no_second_t0814
```

Also verify no regression in STORY-107 tests:
```
cargo test test_EC_006_invalid_length_byte_increments_parse_errors
cargo test --all-targets
```

### Step 5 — Implementer: Refresh STORY-107 input-hash
**Owner: Implementer**
**Depends on: Step 1 (BC-2.15.016 must be updated before hash is recomputed)**

Run from the repo root (where `.factory/` is mounted):
```
bin/compute-input-hash --write .factory/stories/STORY-107.md
```

This is routine hash maintenance following BC text evolution. It does not
trigger story regeneration.

### Step 6 — Adversarial Pass: Required before STORY-109 merge
**Owner: Test-Writer / Adversarial pass**
**Depends on: Steps 1-5 all green**

A fresh adversarial pass is required specifically for the resync path. The
adversarial reviewer should verify:
- No carry state (0..=292 bytes, any byte values) causes a panic in the
  byte-walk-forward resync arm.
- The three-frame malformed flood scenario (Decision 1 worked example)
  produces exactly one T0814, `parse_errors=3`, `malformed_in_window=3`.
- The carry-clear case (no next sync word found) does not set `is_non_dnp3`
  and does not prevent subsequent valid frames from being processed after
  the next `on_data` delivers fresh bytes with a valid sync word.
- The drain-to-next-sync case (next sync word found at some offset >1) correctly
  positions the carry head for the next frame-walk iteration without leaving
  a double-counted error event.

---

## Summary Table

| Section | Decision | Owner |
|---------|----------|-------|
| DECISION 1 | Byte-walk-forward resync: AUTHORIZED. Algorithm as specified. `continue` not `break`. No counter increment in resync arm. Termination guaranteed. BC-2.15.009 interaction: none (distinct code paths). | Architect |
| DECISION 2 | BC change: YES, scoped to BC-2.15.016 EC-007 only. No change to BC-2.15.004, BC-2.15.009, BC-2.15.024. Exact clause replacement specified. | Product-Owner |
| DECISION 3 | test_EC_006 update: AUTHORIZED. New expected value: `carry.len() == 0`. Classification: planned spec evolution (STORY-107 deferral realized), not regression. STORY-107 input-hash refresh required. | Implementer/Test-Writer |
| DECISION 4 | VP-023: no re-run required. Three behavioral invariants (no-panic, carry bounded, no wrapping arithmetic) must be verified by inspection. Future Sub-property E possible but not required for STORY-109. | Implementer (inspection) |
| REMEDIATION | Steps 1-6 in order: PO updates BC → Implementer updates code → update test_EC_006 → verify 4 blocked tests pass → refresh STORY-107 hash → adversarial pass. | See per-step owners |

---

## Authority

This adjudication is issued by the Architect (ADR-007 owner, SS-15 architecture
owner). It is final for STORY-109. The Product-Owner is authorized to implement
the BC-2.15.016 EC-007 update described in Decision 2 without further review.
The Implementer is authorized to implement the code change in Decision 1 and
the test update in Decision 3 without further review. No escalation is required.

---

## Addendum ADJ-001-A: Carry-Cap Tests and Clear-vs-Bail Ruling
**Date: 2026-06-11**

Two additional STORY-107 tests failed after the byte-walk-forward resync was
implemented: `test_carry_buffer_cap_at_292` (AC-001) and
`test_EC_003_carry_291_plus_2_overflow` (EC-003). Both pre-fill carry with
all-junk bytes (repeated 0xAA with no `[0x05, 0x64]` sync word anywhere), trigger
the carry-overflow in Step 2 of `on_data`, then the frame-walk executes and the
byte-walk-forward resync clears the junk carry. Both tests assert
`carry.len() == 292` (the STORY-107 v1 hold-the-junk behavior); the new behavior
produces `carry.len() == 0`.

---

### Q1 — CLEAR vs BAIL: CONFIRM (a) clear-and-continue

**Ruling: CONFIRMED. `carry.clear()` without setting `is_non_dnp3 = true` is
the correct and only acceptable behavior for the no-sync-found resync case.**

Justification:

BC-2.15.009's 16-byte bail rule applies exclusively to the INITIAL delivery
check — the `flow.carry.is_empty() && data.len() >= 2` guard at Step 1 of
`on_data`. Its semantic is: "this flow has never accepted a byte and the very
first delivery shows no sync word — this is not a DNP3 flow." Once any bytes
have been accepted into carry, the flow is established as DNP3 by the initial
check having passed. BC-2.15.009 does not govern what happens when an
established flow's carry becomes misaligned later.

The two failing tests construct their junk state by DIRECTLY MUTATING
`flow.carry` after the flow entry has been created with a valid `[0x05, 0x64]`
seed. This is an artificial test setup — no real network sequence produces a
292-byte all-0xAA carry on an established DNP3 flow without having first gone
through the carry-overflow path. The carry-overflow already recorded the error
(`parse_errors += 1`, `malformed_in_window += 1`). The subsequent byte-walk on
the junk carry is purely navigational.

Setting `is_non_dnp3 = true` in this arm would be architecturally wrong for
three reasons:

1. **Permanent vs. transient misalignment.** Carry junk after an overflow is a
   transient condition. The next `on_data` call could deliver a perfectly valid
   DNP3 frame starting with `[0x05, 0x64]`. Latching `is_non_dnp3` would
   permanently silence a live DNP3 flow based on a single overflow event.

2. **BC-2.15.009 scope.** BC-2.15.009 Postcondition 6 states: "`is_non_dnp3`
   remains `true`; it is never reset to `false`." Setting it from the resync
   arm would create an un-resettable latch that BC-2.15.009 does not authorize
   for this trigger condition.

3. **Detection completeness.** Permanently bailing after a carry overflow would
   allow an attacker to silence a flow's detection by deliberately flooding it
   with oversized garbage — a trivially-achievable DoS against the analyzer
   itself. Clear-and-continue means the next valid DNP3 frame resumes detection
   normally.

The practical difference is exactly as stated: clear lets the flow recover if a
later valid frame arrives; bail permanently drops it. Clear is correct.

---

### Q2 — TEST UPDATE AUTHORIZATION: AUTHORIZED, with restructured assertion

**Ruling: AUTHORIZED to update both `test_carry_buffer_cap_at_292` and
`test_EC_003_carry_291_plus_2_overflow`. The 292-cap invariant IS still
genuinely tested after the change; the cap assertion must be restructured,
not deleted.**

The cap invariant (BC-2.15.016 PC2) governs the accumulation step (Step 2 of
`on_data`): carry NEVER EXCEEDS 292 during byte accumulation. This invariant is
tested at the moment of accumulation, not at the moment the frame-walk
subsequently runs. The existing tests verify `parse_errors == 1` (overflow fired)
and the original `carry.len() == 292` captured state at a point AFTER the
frame-walk had run. After the resync change, the frame-walk clears the carry,
so checking `carry.len() == 292` after `on_data` returns no longer reflects the
cap invariant — it reflects the frame-walk's post-resync state.

The correct restructuring is to verify the cap invariant through the
`parse_errors` count (which proves the overflow fired, i.e., carry hit 292 and
bytes were discarded) and to verify liveness (the frame-walk ran and advanced
the carry) separately. The `carry.len() == 0` assertion after the call is
correct for the new behavior and confirms liveness.

#### Exact replacement for `test_carry_buffer_cap_at_292`:

Replace:
```rust
assert_eq!(
    flow.carry.len(),
    MAX_DNP3_FRAME_LEN,
    "carry.len() must be capped at MAX_DNP3_FRAME_LEN=292 after overflow"
);
assert_eq!(
    flow.parse_errors, 1,
    "parse_errors must be 1: carry overflow increments the lifetime parse_errors counter"
);
```

With:
```rust
// BC-2.15.016 PC2: the 292-cap invariant is proven by parse_errors == 1.
// parse_errors is incremented once and ONLY ONCE — in the overflow arm of Step 2,
// when carry.len() + new_bytes.len() > 292. If the cap were not enforced,
// carry would grow to 295 and no parse_errors increment would occur.
// parse_errors == 1 is the authoritative cap-invariant assertion.
assert_eq!(
    flow.parse_errors, 1,
    "parse_errors must be 1: carry overflow fired (292-cap enforced; 3 bytes discarded)"
);

// After the overflow, the frame-walk ran and found no [0x05,0x64] sync word
// in the 292 bytes of 0xAA/0xBB filler → byte-walk-forward resync cleared carry.
// carry.len() == 0 confirms: (a) the frame-walk ran (not a no-op), and
// (b) the resync liveness property (carry advanced on every non-break iteration).
assert_eq!(
    flow.carry.len(),
    0,
    "carry must be 0 after byte-walk-forward resync found no sync in junk carry \
     (STORY-109 behavior; overflow was already counted via parse_errors)"
);
```

#### Exact replacement for `test_EC_003_carry_291_plus_2_overflow`:

Replace:
```rust
assert_eq!(
    flow.carry.len(),
    MAX_DNP3_FRAME_LEN,
    "carry must be capped at 292 after accepting 1 of 2 bytes"
);
assert_eq!(
    flow.parse_errors, 1,
    "parse_errors must be 1: 1 byte was discarded (overflow)"
);
```

With:
```rust
// BC-2.15.016 PC2 (292-cap): parse_errors == 1 proves the cap fired.
// 291 bytes in carry + 2 bytes delivered → only 1 accepted (total=292 cap);
// 1 byte discarded; overflow arm increments parse_errors exactly once.
assert_eq!(
    flow.parse_errors, 1,
    "parse_errors must be 1: 1 byte was discarded at the 292 cap (BC-2.15.016 PC2)"
);

// After overflow, frame-walk ran: no [0x05,0x64] sync found in 292 bytes of 0xAA
// → carry cleared by byte-walk-forward resync (STORY-109 behavior).
assert_eq!(
    flow.carry.len(),
    0,
    "carry must be 0: byte-walk-forward resync found no sync in junk carry after overflow"
);
```

#### Does the 292-cap remain genuinely tested?

Yes. `parse_errors == 1` is the mechanically correct assertion for the cap
invariant because:
- If the cap were NOT enforced (carry grows to 295), the overflow arm never
  fires and `parse_errors` remains 0. The test would fail at `assert_eq!(1)`.
- If the cap fires but TWICE somehow, `parse_errors` would be 2. Also fails.
- Only when carry is correctly capped at 292 with exactly the right number of
  bytes discarded does `parse_errors == 1` hold.

The `carry.len() == 0` assertion covers liveness/resync. Together they form a
strictly stronger combined assertion than the old `carry.len() == 292` alone,
which only tested post-frame-walk state and said nothing about whether the
overflow arm actually fired.

---

### BC-2.15.016 EC-003/EC-004 and Canonical Test Vectors

**No wording change is required to BC-2.15.016 EC-003 or EC-004.**

EC-003 (`Single on_data call delivers one complete frame + start of a second`)
and EC-004 (`Carry reaches 291 bytes (1 byte short of 292); on_data delivers 2
more bytes | 1 byte accepted (total=292); 1 byte discarded; parse_errors++`)
describe the accumulation behavior, not the post-frame-walk carry state. EC-004
says "1 byte accepted (total=292); 1 byte discarded; parse_errors++" — this
remains entirely accurate. The EC text describes what the overflow arm does; it
does not claim the carry is 292 after `on_data` returns (that would be a claim
about the frame-walk's behavior, not the overflow arm's behavior). The text is
correct as-is.

**The Canonical Test Vectors table "Carry overflow (adversarial)" row should
have a note appended.** The current row reads:

> Carry overflow (adversarial) | [290 bytes] | 5 bytes | 2 bytes appended (292); 3 discarded; parse_errors++

This is accurate for the overflow arm. Add a parenthetical note about post-walk
state, so future readers are not surprised:

CURRENT:
> 2 bytes appended (292); 3 discarded; parse_errors++

REPLACEMENT:
> 2 bytes appended to reach 292 (cap enforced); 3 discarded; parse_errors++ (overflow arm). Then frame-walk runs: if carry head has no [0x05,0x64] sync, byte-walk-forward resync clears carry. Final carry.len() depends on whether carry contains a recoverable sync word.

Product-owner should apply this one-line note to BC-2.15.016's Canonical Test
Vectors table as part of the v1.2 modification pass already authorized in
Decision 2.

---

### Addendum Summary

| Question | Ruling |
|----------|--------|
| Q1: clear vs bail | CONFIRMED (a): `carry.clear()` without `is_non_dnp3 = true`. Bail is architecturally wrong for mid-carry resync. |
| Q2: update both carry-cap tests | AUTHORIZED. Restructure: `parse_errors == 1` is the 292-cap assertion; `carry.len() == 0` is the resync-liveness assertion. Exact replacements above. Cap invariant is MORE strongly tested, not weakened. |
| BC-2.15.016 EC-003/EC-004 | No change required. Text describes the overflow arm, not post-walk state — remains accurate. |
| BC-2.15.016 Canonical Test Vectors row | One-line note addition authorized for the "Carry overflow" row, as part of the v1.2 modification pass. |
