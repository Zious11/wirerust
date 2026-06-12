---
document_type: adjudication
adjudication_id: ADJ-002
status: revised
date: 2026-06-11
revised: 2026-06-12
author: architect
finding: F-F5-003
feature: issue-008-dnp3-analyzer
traces_to:
  - ADJ-001
  - ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - BC-2.15.016
  - BC-2.15.024
  - VP-023
  - STORY-109-resync-adjudication.md
---

# ADJ-002: F-F5-003 Resync Accounting Gap — Junk-at-Clean-Boundary Evades Malformed Counters

## Finding Under Review

F-F5-003 (MAJOR): The byte-walk-forward resync arm at `src/analyzer/dnp3.rs` ~lines 378-397
silently drains non-sync bytes with no `parse_errors` or `malformed_in_window` increment when
the resync arm is entered after a CLEAN frame consume (line 591 `flow.carry.drain(..frame_len)`)
rather than after a LENGTH-gate or frame-header reject. ADJ-001 Decision 1's no-double-count
ruling justified zero-increment in the resync arm on the basis that "the malformed-frame reject
that incremented parse_errors + malformed_in_window already happened in the LENGTH-gate arm".
The adversary argues this justification is path-conditional and does not hold when the resync
arm is entered from the clean-consume path.

---

## IS-A-GAP: YES

### Code Path Analysis — Three Routes to the Resync Arm

Reading the frame-walk loop at lines 361-592:

**Path A: LENGTH-gate reject → resync arm (ADJ-001's justified case)**

1. Iteration N: sync gate passes (`carry[0]=0x05, carry[1]=0x64`).
2. `compute_dnp3_frame_len(carry[2])` returns None (LENGTH < 5).
3. `parse_errors += 1; malformed_in_window += 1; carry.drain(..1); check_malformed_anomaly(...);
   continue;`
4. Loop re-enters. New `carry[0]` is the byte that was at index 1 (e.g. `0x64`). This is not
   `0x05` so the sync gate fails.
5. Resync arm fires. Counters were already incremented in step 3. No double-count risk.
   ADJ-001 Decision 1 is precisely correct for this path.

**Path B: Clean frame consume → resync arm (THE GAP)**

1. Iteration N: sync gate passes. LENGTH valid. `frame_len` computed. `carry.len() >= frame_len`.
   Full frame parsed successfully. `frame_count += 1`. Detections run.
2. Line 591: `flow.carry.drain(..frame_len)`. This is the CLEAN consume — no error, no increment.
3. Loop re-enters (implicit, top of `loop` body). `carry[0..1]` are now whatever bytes
   immediately follow the consumed frame. If these bytes are non-sync junk (attacker injected
   arbitrary data precisely at a frame boundary, or a mid-session corruption event), the sync
   gate fails.
4. Resync arm fires. `parse_errors` and `malformed_in_window` have NOT been incremented for
   this event anywhere in this iteration. The resync arm does not increment them.
5. All non-sync junk bytes are drained silently. From a counter perspective, this event never
   happened.

No prior counting occurred in this iteration. Zero accounting for sync-loss at a clean
boundary is the gap.

**Path C: Frame-header mismatch reject → resync arm (analogous to Path A)**

The frame-header mismatch arm at lines 432-438 increments both counters and drains `frame_len`
bytes before continuing. The loop re-enters with the post-consume carry head, which may or may
not be a sync word. If it is not, the resync arm fires. Counters were already incremented in
the mismatch arm. No double-count risk. This path is equivalent to Path A in structure and is
not a gap.

### Conclusion on IS-A-GAP

The adversary's argument is correct and precise. Path B is a distinct control-flow path that
reaches the resync arm without any prior counter increment in the same iteration. The ADJ-001
justification was accurate for Paths A and C but was incomplete in its coverage: it implicitly
assumed the resync arm is only reachable after a counter-incrementing reject. The code as
implemented has no such structural guarantee — the loop re-enters from the clean consume
(line 591) and proceeds directly to the sync check.

This is a real behavioral gap: a Crain-Sistrunk-style probe injecting junk bytes precisely at
frame boundaries would traverse the resync arm on every frame boundary hit and accumulate
ZERO malformed accounting, defeating BC-2.15.024's threshold.

---

## COUNTING RULE

### Principle

The resync arm represents sync-loss: the carry head does not contain a valid DNP3 sync word.
Sync-loss is explicitly enumerated in BC-2.15.024 Precondition 1 as a malformed-reject path
that MUST increment both `parse_errors` and `malformed_in_window`. Whether that sync-loss was
triggered by a prior LENGTH-gate reject (Path A) or by junk appearing after a clean consume
(Path B) is irrelevant to the counting obligation: in BOTH cases, the carry head is corrupted
and the event is a structural anomaly on the flow.

However, in Path A (and C), the LENGTH-gate (or mismatch) arm has already incremented both
counters. The resync arm must NOT increment again for the same event. The invariant is:

    Exactly one (parse_errors++, malformed_in_window++) increment per sync-loss event.

### Mechanism to Distinguish Path A/C from Path B

The cleanest mechanism is a boolean flag `counted_this_iter` that is local to each frame-walk
loop iteration, initialized to `false` at the top of the loop, set to `true` by any
counter-incrementing arm (LENGTH-gate, mismatch), and consulted by the resync arm to decide
whether to increment. This is zero overhead (single bool on the stack per iteration), requires
no restructuring of the existing arms, and directly encodes the invariant.

### Exact Algorithm

```
loop {
    if carry.len() < 3 { break; }

    // Reset per-iteration "already counted this sync-loss" flag.
    let mut counted_this_iter = false;   // <-- NEW

    // SYNC GATE
    if carry[0] != 0x05 || carry[1] != 0x64 {
        // Sync-loss: this is a malformed/structural event on this flow.
        // Increment BOTH counters UNLESS a counter-incrementing reject arm
        // already fired in this same iteration (which would mean the carry[0]
        // misalignment was produced by a drain-1 from the LENGTH-gate arm).
        if !counted_this_iter {                  // <-- NEW GUARD
            parse_errors += 1;
            malformed_in_window += 1;
            check_malformed_anomaly(...);
        }
        // ... byte-walk-forward resync (unchanged) ...
        continue;
    }

    // VALIDITY GATE (LENGTH < 5)
    let frame_len = match compute_dnp3_frame_len(carry[2]) {
        Some(fl) => fl,
        None => {
            parse_errors += 1;
            malformed_in_window += 1;
            counted_this_iter = true;            // <-- NEW: mark counted
            carry.drain(..1);
            check_malformed_anomaly(...);
            continue;
        }
    };

    // FRAME-LENGTH GUARD
    if carry.len() < frame_len { break; }

    // HEADER MISMATCH
    let header = match parse_dnp3_dl_header(&carry[..frame_len]) {
        Some(h) if is_valid_dnp3_frame_header(&h) => h,
        _ => {
            parse_errors += 1;
            malformed_in_window += 1;
            counted_this_iter = true;            // <-- NEW: mark counted
            carry.drain(..frame_len);
            check_malformed_anomaly(...);
            continue;
        }
    };

    // ... (valid frame processing, detections) ...

    // CLEAN CONSUME (line 591 equivalent)
    carry.drain(..frame_len);
    // counted_this_iter remains false; loop re-enters; if next head is non-sync,
    // resync arm will increment (because !counted_this_iter is true).
}
```

### Why This Is Correct for Every Path

Path A (LENGTH-gate → resync arm):
- LENGTH-gate fires: `parse_errors++`, `malformed_in_window++`, `counted_this_iter = true`,
  `carry.drain(..1)`, `continue`.
- Loop top: `counted_this_iter` reset to `false` for the NEW iteration.

Wait — this requires care. After `continue`, the loop re-enters at the TOP where
`counted_this_iter` is re-initialized to `false`. So when the resync arm runs in the NEW
iteration, `counted_this_iter` is `false` again. That means the resync arm WOULD increment —
which would be a double-count.

This is incorrect. The `counted_this_iter` approach only works if the resync arm is entered in
the SAME iteration as the LENGTH-gate drain. Let's trace this precisely:

Iteration N:
- Sync gate: PASSES (carry[0]=0x05, carry[1]=0x64).
- LENGTH-gate fires: `counted_this_iter = true`. `carry.drain(..1)`. `continue`.

Iteration N+1 (new loop body, new `counted_this_iter = false`):
- Sync gate: FAILS (carry[0]=0x64 — the byte that was at index 1).
- `!counted_this_iter` is TRUE → resync arm would increment. DOUBLE-COUNT.

This confirms the `counted_this_iter` approach as described above IS INCORRECT for Path A.
The LENGTH-gate and the resync arm that follows it are in DIFFERENT iterations. The flag
reset at the loop top breaks the connection.

### Correct Algorithm — LENGTH-Gate Owns Full Resync Navigation

The root cause is that the LENGTH-gate arm drains 1 byte and then `continue`s, leaving the
resync to happen in the NEXT iteration. This means there is no way to communicate "this
sync-loss was already counted" across an iteration boundary using a per-iteration flag.

The correct fix restructures ownership: the LENGTH-gate arm should perform the byte-walk-
forward resync itself, not defer it to the next iteration's sync check. This way:

- Path A: LENGTH-gate increments counters ONCE and then immediately resyncs the carry in the
  same arm. The loop re-enters with a valid sync head (or empty carry). The resync arm is
  never reached after a LENGTH-gate reject.
- Path B: Clean consume. Loop re-enters. Carry head is non-sync junk. Sync check fails.
  Resync arm fires. This is the ONLY path that reaches the resync arm. Resync arm always
  increments. No conditional needed.
- Path C: Mismatch arm: increments counters, drains `frame_len` bytes (the mismatch frame).
  `continue`. Loop re-enters. If the subsequent carry head is non-sync, the resync arm fires
  and increments again. This would be correct because the mismatch arm counted ONE event (the
  mismatched frame), and the non-sync junk following it is a SEPARATE event.

Concretely, the LENGTH-gate arm becomes:

```rust
None => {
    // Invalid LENGTH (< 5): structural parse error. Count it.
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    // Immediately resync the carry so the next iteration starts at a valid head
    // (or with empty carry). This prevents the next iteration from entering the
    // sync-check arm for the same sync-loss event, which would be a double-count.
    // The resync is pure cursor movement — no second increment.
    flow.carry.drain(..1); // remove the invalid 0x05 head
    // Now byte-walk-forward from index 0 (the new head, formerly index 1):
    let next_sync = flow
        .carry
        .windows(2)
        .enumerate()
        .find(|(_, w)| w[0] == 0x05 && w[1] == 0x64)
        .map(|(i, _)| i);
    match next_sync {
        Some(i) => { flow.carry.drain(..i); }
        None    => { flow.carry.clear(); }
    }
    Self::check_malformed_anomaly(flow, &mut self.all_findings, ts, &flow_key);
    continue;
}
```

The sync-check arm (the resync arm ~378-397) is simplified: it always increments and then
resyncs. The sync-check arm is now ONLY reached from Path B (clean consume followed by junk)
and the top-of-loop entry with a freshly misaligned carry. The LENGTH-gate arm never leaves
a misaligned head for the next iteration.

### Final Exact Counting Rule

**Rule**: The resync arm (~lines 378-397) increments `parse_errors` and `malformed_in_window`
unconditionally (exactly once per entry into the arm). The LENGTH-gate arm performs its own
inline resync (as above) so that it never leaves a non-sync head that would cause the
NEXT iteration's sync check to fire for the same event. The mismatch arm's drain of
`frame_len` bytes makes it structurally identical to a clean consume: if the bytes following
the mismatch frame are also non-sync, the resync arm fires in the next iteration and counts
that as a separate event, which is correct.

**Which arm owns the increment:**
- Resync arm: owns its own `parse_errors++` and `malformed_in_window++` unconditionally.
- LENGTH-gate arm: owns its own `parse_errors++` and `malformed_in_window++`, AND owns its
  own inline resync so the resync arm is never entered as a consequence of its drain-1.
- Mismatch arm: owns its own `parse_errors++` and `malformed_in_window++`. Does NOT own
  inline resync (it drains the full frame, so the next carry head is the byte immediately
  after the malformed frame — a new event if non-sync).

**How the double-count is avoided:**
The LENGTH-gate arm's inline resync ensures the carry head is valid (or carry is empty) when
the `continue` re-enters the loop top. The sync check will pass (or the `carry.len() < 3`
guard will break the loop). The resync arm is structurally unreachable as a consequence of a
LENGTH-gate drain.

### Per-Iteration Double-Count Proof

After LENGTH-gate + inline resync:
- `carry` is either empty or starts with `[0x05, 0x64, ...]`.
- `carry.len() < 3` → break. OR sync gate passes → no resync arm entry.
- In neither case does the resync arm execute. QED: no double-count.

After clean consume (line 591):
- `carry` may be empty or start with any bytes.
- `carry.len() < 3` → break (no resync arm). OR `carry[0] != 0x05 || carry[1] != 0x64` →
  resync arm fires and increments once. OR sync gate passes → no resync arm entry.
- Exactly zero or one increments per clean-consume continuation. QED: correct.

---

## BC-CHANGE: YES — BC-2.15.024 Precondition 1 and BC-2.15.016 EC-007

### BC-2.15.024 Change Required

BC-2.15.024 Precondition 1 currently lists three existing structural-reject paths that
increment both counters:

> The three existing reject paths are:
> - BC-2.15.016 Postcondition 2 (carry overflow / LENGTH byte truncation), or
> - BC-2.15.004 validity gate reject (LENGTH<5 or sync!=0x0564), or
> - BC-2.15.009 sync-loss bail (no valid sync in first 16 bytes, is_non_dnp3 set).

The resync arm handling sync-loss at a clean boundary is a FOURTH path that must be added.
Product-owner must add to Precondition 1:

> - byte-walk-forward resync arm (sync-loss on an established flow after a clean frame
>   consume — non-sync bytes appear immediately after a valid frame; the resync arm
>   increments both parse_errors and malformed_in_window once per resync entry before
>   performing the byte-walk-forward navigation).

This is NOT a new mechanism being invented — it is correctly documenting that the resync
arm, which already existed as a structural sync-loss handler, must count the events it
handles. The BC text currently omits this path because ADJ-001 Decision 2 ruled no BC-2.15.024
change was required under the assumption that the resync arm was only reachable after a prior
counted reject. That assumption is now falsified by Path B analysis.

### BC-2.15.016 EC-007 Change Required

EC-007 currently specifies (post-ADJ-001 v1.2 update):

> Validity gate (BC-2.15.004) handles this; `parse_errors++` (lifetime) and
> `malformed_in_window++` (windowed, STORY-109); carry advanced via byte-walk-forward
> resync: scan carry from index 1 for the next `[0x05, 0x64]` sync word; drain all
> bytes before it if found; if no sync word found, clear carry entirely. No further
> `parse_errors` or `malformed_in_window` increment occurs during resync navigation —
> the error was already counted at the LENGTH gate.

The phrase "scan carry from index 1" and the structure of "no further increment during
resync navigation" remains accurate under the new algorithm BECAUSE the LENGTH-gate arm
now performs the resync inline (not in the next iteration). The resync navigation specified
is correct. However, EC-007 should note that the inline resync is performed within the
LENGTH-gate arm itself (not deferred to the next iteration's sync check).

Product-owner must update EC-007 to add one clarifying clause:

CURRENT end of EC-007:
> No further `parse_errors` or `malformed_in_window` increment occurs during resync
> navigation — the error was already counted at the LENGTH gate. The carry-clear on
> no-sync-found does NOT set `is_non_dnp3 = true`. Each non-break iteration drains
> ≥1 byte; carry bounded ≤292 bytes; loop terminates.

REPLACEMENT end of EC-007 (add one sentence before the termination line):
> The LENGTH-gate arm performs this resync navigation INLINE before `continue`, so
> the loop's next iteration begins with a valid sync head or an empty carry; the
> sync-check arm is not entered as a consequence of a LENGTH-gate drain (no
> double-count across iterations). The carry-clear on no-sync-found does NOT set
> `is_non_dnp3 = true`. Each non-break iteration drains ≥1 byte; carry bounded
> ≤292 bytes; loop terminates.

### BC-2.15.016 — Sync-Check Arm (new EC row optional but recommended)

The sync-check arm now unconditionally increments both counters when it fires. It is
reached exclusively when a clean frame consume leaves a non-sync head, or when carry
is initially misaligned. Product-owner should add or update the relevant EC to state:

> EC-009 (new): After a clean frame consume (`carry.drain(..frame_len)`), if the
> immediately following bytes do not begin with `[0x05, 0x64]` (junk at a frame
> boundary due to injection or corruption), the sync-check arm fires: `parse_errors++`
> (lifetime) and `malformed_in_window++` (windowed); byte-walk-forward resync then
> locates or clears the carry. This counts as one malformed/structural event.

Note: if an EC-009 already exists for other content, the product-owner should use the
next available ID. The content requirement is what matters.

---

## VP-023 IMPACT: NONE (panic-free / wrapping preserved)

VP-023 Sub-property D proves `compute_dnp3_frame_len` result is in [10, 292], guaranteeing
carry indexing is in-bounds. The proposed change does not touch `compute_dnp3_frame_len`.

The inline resync inside the LENGTH-gate arm uses the identical `windows(2).enumerate().find()`
iterator chain already proven safe in ADJ-001 Decision 4. Specifically:

- The inline resync fires AFTER `carry.drain(..1)`, so carry has had 1 byte removed.
  `carry.len() >= 2` at that point (it was >= 3 at the top-of-loop guard; drain-1 leaves >= 2).
  `windows(2)` on a 2-element slice yields one window — safe.
- `drain(..i)` where `i` is an iterator-returned index into `carry` is in-bounds by
  construction (the iterator cannot produce out-of-range indices).
- `carry.clear()` is unconditionally safe.
- No u32 arithmetic added. No wrapping subtraction added. `parse_errors` and
  `malformed_in_window` are `u64` — no overflow risk.

The resync arm increment (`parse_errors += 1`, `malformed_in_window += 1`) adds two `u64`
increments. With `overflow-checks = true` in the release profile, these would panic on
overflow. `u64::MAX` is ~1.8 × 10^19. Even at 10^9 malformed frames per second (physically
impossible on a 100 Mbps link), overflow would require ~585 years. No wrapping is needed.

VP-023 does not require re-running or modification.

---

## TESTS

### Required Tests

**(i) Junk-at-clean-boundary increments both counters exactly once**

Test name: `test_EC_junk_at_clean_boundary_increments_malformed_counters`

Scenario: Deliver one complete valid DNP3 frame followed IMMEDIATELY by non-sync junk bytes
in the same `on_data` call (or as a second call). After the frame-walk processes the valid
frame (clean consume), the carry head is `[0xAA, 0xBB, 0xCC, ...]` with no `[0x05, 0x64]`.

Assertions:
- `flow.parse_errors == 1` (one structural event from the junk)
- `flow.malformed_in_window == 1`
- `flow.frame_count == 1` (the valid frame was counted)
- `flow.carry.len() == 0` (resync cleared the junk carry)

Example input construction:
```
// Build a minimal valid DNP3 frame (10 bytes, frame_len=10):
//   05 64 05 44 03 00 01 00 <CRC1> <CRC2>  (LENGTH=5, compute_dnp3_frame_len(5)=10)
// NOTE: actual CRC bytes are irrelevant if parse_dnp3_dl_header / is_valid_dnp3_frame_header
// do not check CRC (ADR-007 Decision 3 — CRC deferred). Use 0x00 0x00 as placeholders.
// Append junk: AA BB CC (no 0x05 0x64 anywhere).
let data = [0x05u8, 0x64, 0x05, 0x44, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00,
            0xAA, 0xBB, 0xCC];
```

**(ii) LENGTH-gate-then-resync still increments exactly once (no double-count regression)**

Test name: `test_EC_length_gate_resync_no_double_count`

This is a STRENGTHENED version of `test_EC_006_invalid_length_byte_increments_parse_errors`.
Scenario: deliver one malformed frame with LENGTH<5 (e.g. LENGTH=2). The LENGTH-gate arm fires,
increments both counters, and performs inline resync. The carry is cleared (no sync found in
junk). Loop continues: carry.len() == 0 < 3, break.

Assertions:
- `flow.parse_errors == 1` (exactly one increment — no second increment from resync arm)
- `flow.malformed_in_window == 1` (exactly one)
- `flow.carry.len() == 0`
- The resync arm was NOT entered (this is verified structurally by the parse_errors count
  remaining at 1, not 2)

This is equivalent to the existing `test_EC_006` with an added assertion that counters are
exactly 1 (not 2), which would catch a double-count regression.

**(iii) Malformed flood via boundary-junk reaches BC-2.15.024 threshold**

Test name: `test_malformed_anomaly_boundary_junk_reaches_threshold`

Scenario: Three `on_data` calls, each delivering one valid DNP3 frame followed by non-sync
junk. Each call should produce one malformed accounting event (from the resync arm). On the
third call, `malformed_in_window` reaches 3 and T0814 is emitted.

Assertions after third call:
- `flow.parse_errors == 3`
- `flow.malformed_in_window == 3`
- `flow.frame_count == 3` (three valid frames were consumed)
- `flow.malformed_anomaly_emitted == true`
- `self.all_findings` contains exactly one T0814 finding with `mitre_techniques: ["T0814"]`

**(iv) Existing threshold test must remain green**

`test_malformed_anomaly_at_threshold_3_of_300s` (STORY-109 AC-012): verifies the LENGTH<5
flood path reaches threshold. Must continue to pass with `parse_errors == 3`,
`malformed_in_window == 3`, T0814 emitted. This test provides the regression guard for
Path A (no double-count).

**(v) Existing carry-cap tests must remain green**

`test_carry_buffer_cap_at_292` and `test_EC_003_carry_291_plus_2_overflow`: the overflow arm
increments both counters. The frame-walk then runs the resync arm on the junk carry. Under
the new rule, the resync arm unconditionally increments — but these tests pre-fill carry with
all-junk bytes (0xAA) and trigger the OVERFLOW arm (which increments), then the loop runs the
resync arm which would NOW ALSO increment. This introduces a double-count for the overflow path.

OVERFLOW PATH ANALYSIS: The overflow arm (lines 347-356) increments `parse_errors += 1` and
`malformed_in_window += 1`, then the frame-walk loop runs. After overflow, the carry head is
still junk (`0xAA` which is not `0x05`). The sync check fires and under the new rule would
increment AGAIN. This is a DOUBLE-COUNT for the overflow path.

Therefore: the overflow arm MUST ALSO perform inline resync (like the LENGTH-gate arm), OR
the overflow arm must clear the carry before the frame-walk begins. The simplest fix for
the overflow arm is to clear the carry immediately after the overflow discard:

```rust
if data.len() > remaining_capacity {
    flow.carry.extend_from_slice(&data[..remaining_capacity]);
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    Self::check_malformed_anomaly(flow, &mut self.all_findings, ts, &flow_key);
    flow.carry.clear(); // <-- NEW: overflow always leaves a junk carry; clear now
    return;             // <-- CHANGE: return instead of falling through to frame-walk
                        //     (the carry is empty; frame-walk would be a no-op anyway)
}
```

OR, equivalently, perform the inline resync in the overflow arm. The `carry.clear()` +
`return` form is simpler and safe: overflow means the carry was filled with data that could
not form a valid frame anyway (it was capped at 292 with excess discarded); there is nothing
useful in it. Clearing and returning is semantically correct and eliminates the frame-walk
entry on a known-junk carry.

If `carry.clear()` + `return` is used for the overflow arm, then `test_carry_buffer_cap_at_292`
and `test_EC_003_carry_291_plus_2_overflow` assertions of `carry.len() == 0` (per ADJ-001-A
Q2 replacements) remain valid, and `parse_errors == 1` (exactly one increment from the
overflow arm) also remains valid. The resync arm is never entered from the overflow path.

This makes the overflow arm structurally identical to the LENGTH-gate arm: count once, clear,
return/continue without entering the sync check.

---

## COMPLETE FIX SPECIFICATION

Three code changes are required. Zero conditional flags needed.

### Change 1: Resync arm — add unconditional increment

At `~lines 378-397`, BEFORE the byte-walk-forward navigation, add:

```rust
if flow.carry[0] != 0x05 || flow.carry[1] != 0x64 {
    // Sync-loss at an established flow head (Path B: junk after clean consume,
    // or Path C continuation where mismatch arm drained a full frame and the
    // next head is also junk).
    // COUNT this event: one structural sync-loss increment per entry.
    // The LENGTH-gate arm performs its own inline resync, so it never reaches
    // here — no double-count for Path A.
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    Self::check_malformed_anomaly(flow, &mut self.all_findings, ts, &flow_key);
    // Byte-walk-forward resync (unchanged):
    let next_sync = flow.carry.windows(2).enumerate().skip(1)
        .find(|(_, w)| w[0] == 0x05 && w[1] == 0x64)
        .map(|(i, _)| i);
    match next_sync {
        Some(i) => { flow.carry.drain(..i); }
        None    => { flow.carry.clear(); }
    }
    continue;
}
```

### Change 2: LENGTH-gate arm — add inline resync after drain-1

At `~lines 406-417`, after `carry.drain(..1)` and `check_malformed_anomaly`, add the
byte-walk-forward resync before `continue`:

```rust
None => {
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    flow.carry.drain(..1);
    // Inline resync: reposition carry to next [0x05,0x64] or clear.
    // This prevents the next iteration's sync check from firing for the
    // same sync-loss event (no double-count across iterations).
    let next_sync = flow.carry.windows(2).enumerate()
        .find(|(_, w)| w[0] == 0x05 && w[1] == 0x64)
        .map(|(i, _)| i);
    match next_sync {
        Some(i) => { flow.carry.drain(..i); }
        None    => { flow.carry.clear(); }
    }
    Self::check_malformed_anomaly(flow, &mut self.all_findings, ts, &flow_key);
    continue;
}
```

Note: `check_malformed_anomaly` is moved to AFTER the inline resync. This is safe because
`check_malformed_anomaly` reads `malformed_in_window` (already incremented) and `all_findings`
(immutable for this check). Order relative to the resync navigation does not matter for
correctness; placing it after the resync is cleaner.

### Change 3: Overflow arm — add carry.clear() + return

At `~lines 347-356`, after `check_malformed_anomaly`, add `flow.carry.clear()` and change
`else { ... }` to early-return:

```rust
let remaining_capacity = MAX_DNP3_FRAME_LEN - flow.carry.len();
if data.len() > remaining_capacity {
    flow.carry.extend_from_slice(&data[..remaining_capacity]);
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    Self::check_malformed_anomaly(flow, &mut self.all_findings, ts, &flow_key);
    flow.carry.clear(); // NEW: overflow leaves junk carry; clear to avoid resync-arm
                        //      double-count when frame-walk runs on the now-empty carry.
    return;             // NEW: nothing useful remains; skip the frame-walk entirely.
} else {
    flow.carry.extend_from_slice(data);
}
```

---

## REMEDIATION SEQUENCE

Owners and order are binding.

### Step 1 — Product-Owner: Update BC-2.15.024 Precondition 1
**Owner: Product-Owner**
**Blocking: Steps 2-5**

Add the resync arm as a fourth structural-reject path in BC-2.15.024 Precondition 1, per
the BC-CHANGE section above. This is a documentation-accuracy fix to match the behavior
being implemented.

### Step 2 — Product-Owner: Update BC-2.15.016 EC-007 and add EC-009
**Owner: Product-Owner**
**Blocking: Steps 3-5**
**Depends on: Step 1**

Update BC-2.15.016 EC-007 with the inline-resync clarification (the LENGTH-gate arm performs
resync before `continue`). Add EC-009 (or next available ID) specifying that the sync-check
arm fires and counts junk-at-clean-boundary events.

### Step 3 — Implementer: Apply the three code changes
**Owner: Implementer**
**Blocking: Steps 4-5**
**Depends on: Steps 1-2**

Location: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/F5-DNP3-FIX/src/analyzer/dnp3.rs`

Apply Changes 1, 2, and 3 in the complete fix specification above. Key constraints:
- Change 1 (resync arm): UNCONDITIONAL increment. No flag. `check_malformed_anomaly` called.
- Change 2 (LENGTH-gate arm): inline resync AFTER drain-1, BEFORE (or after — order flexible)
  `check_malformed_anomaly`. The `continue` is at the end.
- Change 3 (overflow arm): `carry.clear()` then `return`. The frame-walk is skipped.
- Do NOT add any `counted_this_iter` flag — the structural separation of code paths makes it
  unnecessary.

### Step 4 — Implementer/Test-Writer: Add and verify tests
**Owner: Implementer / Test-Writer**
**Depends on: Step 3**

New tests to add (per TESTS section):
1. `test_EC_junk_at_clean_boundary_increments_malformed_counters`
2. `test_EC_length_gate_resync_no_double_count` (strengthened EC-006)
3. `test_malformed_anomaly_boundary_junk_reaches_threshold`

Existing tests that must remain green (regression guard):
- `test_malformed_anomaly_at_threshold_3_of_300s` (AC-012)
- `test_parse_errors_not_reset_at_window_expiry` (AC-013)
- `test_EC_009_fourth_malformed_no_second_t0814` (EC-009 — note: this is a test, not the BC EC)
- `test_carry_buffer_cap_at_292` (ADJ-001-A)
- `test_EC_003_carry_291_plus_2_overflow` (ADJ-001-A)
- All STORY-107 and STORY-109 tests: `cargo test --all-targets`

### Step 5 — Adversarial Reviewer: Validate no triple-path coverage gap
**Owner: Adversarial reviewer**
**Depends on: Steps 3-4**

Verify:
1. Path A (LENGTH-gate → inline resync → loop top → either break or sync passes): carries
   `parse_errors == 1` and `malformed_in_window == 1` for one malformed LENGTH<5 event.
   Resync arm NOT entered.
2. Path B (clean consume → junk head): resync arm fires, `parse_errors == 1`,
   `malformed_in_window == 1`. Exactly one increment.
3. Path C (mismatch → drain frame_len → junk head): mismatch arm counts the mismatch event;
   if next head is also junk, resync arm counts it as a SECOND event (two separate structural
   events). Verify `parse_errors == 2`, `malformed_in_window == 2` for this scenario.
4. Overflow arm + frame-walk: carry is cleared before frame-walk. Resync arm NOT entered.
   `parse_errors == 1`, `malformed_in_window == 1`.
5. Flood via boundary-junk (three frames, each followed by junk): T0814 emitted at third event.

---

## Summary Table

| Item | Decision |
|------|----------|
| IS-A-GAP | YES. Path B (clean frame consume leaves non-sync head) reaches resync arm with zero prior counting in that iteration. ADJ-001's no-double-count reasoning was correct for Paths A and C but did not cover Path B. |
| COUNTING RULE | Resync arm increments unconditionally. LENGTH-gate arm adds inline resync so its drain-1 never causes the next iteration to enter the resync arm. Overflow arm clears carry and returns early, also never entering the resync arm. No boolean flag needed — structural code path separation enforces the invariant. Exactly one increment per sync-loss event on any path. |
| OVERFLOW ARM | Must add `carry.clear()` + `return` to prevent the overflow-increment from being followed by a resync-arm increment on the same junk carry (third double-count vector not mentioned in the original finding but discovered during full fix analysis). |
| BC-CHANGE | YES. BC-2.15.024 Precondition 1: add resync arm as fourth reject path. BC-2.15.016 EC-007: clarify LENGTH-gate arm performs inline resync. BC-2.15.016 EC-009 (new): document junk-at-clean-boundary as a counted structural event. |
| VP-023 | No impact. No panic vectors introduced. u64 counters, iterator-safe drain, `carry.clear()` all safe. No re-run required. |
| TESTS | Add: `test_EC_junk_at_clean_boundary_increments_malformed_counters`, `test_EC_length_gate_resync_no_double_count`, `test_malformed_anomaly_boundary_junk_reaches_threshold`. Existing: all prior STORY-107/109 tests must stay green. |
| SEQUENCE | PO: BC-2.15.024 PC1 update → BC-2.15.016 EC-007 + EC-009 → Implementer: three code changes → Test-Writer: new tests + regression → Adversarial: five-path verification. |

---

## Authority

This adjudication supersedes and extends ADJ-001 Decision 1's no-double-count ruling. ADJ-001
Decision 1 remains correct for Paths A and C; this adjudication adds Path B coverage and
identifies the overflow arm as a third path requiring structural isolation. ADJ-001's
algorithm (the byte-walk-forward resync code itself) is preserved verbatim — the change is
WHERE it appears (also in the LENGTH-gate arm inline) and WHAT it does (unconditional
increment before navigation in the sync-check arm, versus the prior zero-increment).

Issued by the Architect (ADR-007 owner, SS-15 architecture owner). Final as of REVISION 1.
REVISION 1 IS SUPERSEDED IN PART BY REVISION 2 BELOW — do not build from REVISION 1 Change 3
or from the PC1 "fourth path" text. Implementer and Product-Owner: read REVISION 2 in full
before acting on any code change or BC edit.

---

---

# REVISION 2 (post-slice-B) — 2026-06-12

## Status: SUPERSEDES REVISION 1 CHANGE 3 AND ALL BC-CHANGE TEXT

The Slice B agentic adversarial pass (findings F-B-001 through F-C-003) found three problems
in REVISION 1 that make it UNSAFE TO BUILD AS WRITTEN:

| Finding | Severity | Subject |
|---------|----------|---------|
| F-B-002 / F-B-003 | HIGH | Change 3 (`overflow clear()+return`) silently discards a potentially valid head frame — data loss + detection-evasion DoS |
| F-B-001 | CRITICAL | Semantics contradiction: prose says "one per sync-loss event" but the algorithm counts "one per counter-arm entry" — these diverge on attacker-salted fake-sync junk and the discrepancy was never reconciled |
| F-B-004 / F-C-003 | HIGH | BC-CHANGE text written against stale 3-path PC1 (REVISION 1 called the resync arm the "fourth path"); after F-F5-004 reconciliation committed in BC-2.15.024 v1.2, PC1 already lists only TWO paths — so the resync arm is the THIRD, and the bail must NOT be re-introduced; also 5-site propagation list was incomplete |

Findings F-B-008 and F-B-009 CONFIRMED the core of REVISION 1 is sound:
- Change 1 (resync arm unconditional increment) — VERIFIED CORRECT.
- Change 2 (LENGTH-gate inline resync) — VERIFIED CORRECT.

These two changes stand unchanged. Only Change 3 is rejected; the semantics and BC-change
sections are replaced in their entirety below.

---

## R2-SECTION 1: Overflow Arm — REJECTION OF Change 3 + Replacement Algorithm

### Why Change 3 Is Unsafe

REVISION 1 Change 3 specified:

```
flow.carry.clear();
return;
```

after the overflow arm's `check_malformed_anomaly` call. The rationale was "overflow carry has
nothing useful." This is FALSE.

The overflow arm fires when the TOTAL accumulated bytes (existing carry + new data) exceed 292.
It caps the carry at exactly 292 bytes by accepting `remaining_capacity` bytes from the new
data and discarding the rest. After this cap operation, `flow.carry` contains up to 292 bytes.
Those bytes may include a valid `[0x05, 0x64]` sync word at or near the head — either because:

(a) The carry was partially accumulated and the capped bytes happen to begin with a valid sync,
    or
(b) An attacker intentionally crafts the overflow by padding with bytes that place a real
    `[0x05, 0x64, LENGTH, ...]` structure at the cap boundary, making a valid frame available
    immediately after the overflow.

`carry.clear()` + `return` silently discards whatever is at the carry head, including a
potentially complete and parseable valid frame. This causes:

1. **Data loss**: a legitimate frame recoverable from the carry head is destroyed without
   being parsed. Any detection that frame would have triggered is permanently suppressed.

2. **Detection-evasion DoS via repeated overflow**: an attacker who forces one overflow per
   `on_data` call (e.g., by sending exactly `remaining_capacity + 1` bytes each call, where
   the single extra byte is immediately discarded) causes `parse_errors` to increment once per
   call (correct), but ALSO suppresses all frame parsing for that flow by clearing the carry
   on every call — even when valid frames are present after the overflow boundary.

3. **Test blindness**: the existing `test_carry_buffer_cap_at_292` and `test_EC_003_carry_291_plus_2_overflow`
   tests pre-fill the carry with all-0xAA junk bytes (no `[0x05, 0x64]` anywhere). They
   cannot detect this data-loss path because in those tests `carry.clear()` produces the same
   observable result as a correct inline resync that finds no sync word and clears.

### Replacement: Overflow Arm Does Inline Resync (Same Structure as Change 2)

The overflow arm must count once, then perform the identical inline-resync logic used by the
LENGTH-gate arm (Change 2). This:

(a) Preserves any valid head frame sitting in the carry after the overflow cap operation.
(b) Keeps `parse_errors == 1` for the all-junk case — `test_carry_buffer_cap_at_292` and
    `test_EC_003_carry_291_plus_2_overflow` remain green because no-sync-found clears carry,
    exactly as before.
(c) Eliminates the double-count that REVISION 1 was trying to fix: the overflow arm counts
    once (correct), then repositions carry to a valid sync head so the frame-walk loop's
    subsequent iterations see a valid head (or empty carry) — the sync-check arm is never
    entered as a consequence of the overflow-arm's side effects.

### Change 3 Replacement Algorithm (FINAL)

```
// OVERFLOW ARM (replaces REVISION 1 Change 3)
let remaining_capacity = MAX_DNP3_FRAME_LEN - flow.carry.len();
if data.len() > remaining_capacity {
    // Cap: accept up to remaining_capacity bytes; discard the rest.
    flow.carry.extend_from_slice(&data[..remaining_capacity]);
    // Count this structural event exactly once.
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    // Inline resync: reposition carry to next [0x05,0x64] or clear if none found.
    // Structurally identical to Change 2 (LENGTH-gate arm).
    // This prevents the frame-walk's sync-check arm from firing on the same
    // overflow event (no double-count across the overflow→frame-walk boundary).
    // If a valid [0x05,0x64,...] head exists in the carry, it is PRESERVED for
    // the frame-walk — not silently discarded.
    let next_sync = flow
        .carry
        .windows(2)
        .enumerate()
        .find(|(_, w)| w[0] == 0x05 && w[1] == 0x64)
        .map(|(i, _)| i);
    match next_sync {
        Some(i) => { flow.carry.drain(..i); }
        None    => { flow.carry.clear(); }
    }
    Self::check_malformed_anomaly(flow, &mut self.all_findings, ts, &flow_key);
    // Do NOT return early. Fall through to the frame-walk.
    // The frame-walk will either:
    //   (a) find a valid frame at the new carry head and parse it, or
    //   (b) find carry.len() < 3 and break immediately (empty or sub-header carry).
    // Either outcome is correct. The sync-check arm will NOT fire because the
    // carry head is now either [0x05,0x64,...] or empty.
} else {
    flow.carry.extend_from_slice(data);
}
// ... frame-walk loop follows (unchanged) ...
```

### Change 3 Replacement: VP-023 Impact

No new panic vectors. The `windows(2).enumerate().find(...)` chain is the same pattern used
in Changes 1 and 2 and already analyzed as safe in REVISION 1's VP-023 section. `carry.len()`
after `extend_from_slice(&data[..remaining_capacity])` is exactly 292 (it was
`292 - remaining_capacity` before; now it is 292). `windows(2)` on a 292-element Vec yields
290 windows — safe. `drain(..i)` with an iterator-returned index is in-bounds by construction.
`carry.clear()` is unconditionally safe. No arithmetic changes. VP-023 does not require
re-running or modification.

### Change 3 Replacement: Test Continuity

`test_carry_buffer_cap_at_292` and `test_EC_003_carry_291_plus_2_overflow`:
- These tests fill carry with all-0xAA bytes (no `[0x05, 0x64]`).
- The inline resync finds no sync word → `carry.clear()`.
- `parse_errors == 1`, `carry.len() == 0`.
- These assertions are IDENTICAL to what REVISION 1 Change 3's `carry.clear()+return` would
  have produced. Both tests remain green.

---

## R2-SECTION 2: Semantics — Chosen Principle + Alignment

### The Contradiction in REVISION 1

REVISION 1's COUNTING RULE section stated the invariant as:

> Exactly one (parse_errors++, malformed_in_window++) increment per **sync-loss event**.

The algorithm it specified, however, counts "one per counter-arm entry":

- Each entry into the LENGTH-gate arm (one `[0x05, 0x64, LENGTH<5]` triplet encountered)
  counts as one increment.
- Each entry into the mismatch arm counts as one increment.
- Each entry into the overflow arm counts as one increment.
- Each entry into the resync arm (sync-check arm) counts as one increment.

These two formulations diverge on attacker-salted fake-sync traffic. Consider a carry
containing: `[AA BB 05 64 02 AA BB 05 64 02 AA BB]` — three `[0x05, 0x64, 02]` triplets
embedded in junk. The algorithm will:
- Resync arm fires on `AA BB ...` → count 1 (drains to first `05 64`, carry = `[05 64 02 AA BB 05 64 02 AA BB]`)
- LENGTH-gate fires on `05 64 02` → count 2 (inline resync, drains to next `05 64`)
- LENGTH-gate fires on `05 64 02` → count 3 (inline resync, clears)

Three increments for what could be described as "one contiguous junk blob" or "three distinct
fake-frame attempts."

Separately, a carry containing `[AA BB CC DD EE FF ...]` (200 bytes of pure junk, no embedded
`05 64`) produces:
- Resync arm fires once → count 1 (carry.clear(), or drains to first real sync if present)

One increment for 200 junk bytes.

The "per sync-loss event" framing implies these should both produce 1 increment. The
"per counter-arm entry" framing counts 3 for the embedded-fake-sync case and 1 for the pure
junk case.

### Decision: Principle 1 — One Count Per Rejected Frame-Shaped Unit

**CHOSEN SEMANTICS: "one increment per counter-arm entry."**

Rationale:

1. Each embedded `[0x05, 0x64, invalid-LENGTH]` is a distinct frame-shaped structural unit
   that triggers a distinct parse attempt. Each one is independently a structural reject event.
   Counting each reject is the correct signal: an attacker embedding N fake-sync triplets is
   sending N fake frame attempts, each of which should register.

2. The "per sync-loss event" language was an imprecise summary of the no-double-count
   invariant: it meant "do not count the same reject twice (once in the LENGTH-gate arm and
   once in the resync arm)." It did NOT mean "collapse all junk into a single increment
   regardless of how many fake frames are embedded."

3. Principle 1 is simpler to specify and implement: no de-duplication logic, no contiguous-
   run detection, no carry-state lookahead before counting. The structural code path separation
   (Change 2: inline resync in LENGTH-gate arm) already guarantees no double-count without any
   explicit de-dup mechanism.

4. Attacker-salted fake-sync floods: an attacker embedding many `[0x05, 0x64, invalid-LENGTH]`
   triplets will accumulate multiple increments, causing `malformed_in_window` to reach
   `MALFORMED_ANOMALY_THRESHOLD` faster. This is the CORRECT and INTENDED response —
   T0814 (Possible/Low) should be triggered by this traffic pattern. This is NOT a false
   positive; it is exactly the Crain-Sistrunk probe class that BC-2.15.024 is designed to
   detect. Do NOT add de-duplication logic to suppress this signal.

5. Document explicitly: if an attacker sends a flood of `[0x05, 0x64, invalid-LENGTH]` triplets
   sufficient to cross the `malformed_in_window >= 3` threshold, T0814 is emitted. This is
   INTENDED anomaly detection behavior (T0814 Possible/Low). The threshold exists precisely
   to make this behavior detectable.

### Language Replacement

**DELETE** from all documents (BCs, ADJs, spec text): the phrase
"exactly one increment per sync-loss event" / "one per resync event" / "one per sync-loss."

**REPLACE WITH**: "exactly one increment per counter-arm entry: each entry into the resync
arm, LENGTH-gate arm, mismatch arm, or overflow arm increments both `parse_errors` and
`malformed_in_window` exactly once. The structural code-path separation (Change 2: inline
resync in the LENGTH-gate arm; Change 3 replacement: inline resync in the overflow arm)
ensures no arm is entered twice for the same underlying reject event."

**ADD** the following informational note wherever the semantics are discussed:

> Crafted traffic embedding multiple `[0x05, 0x64, invalid-LENGTH]` ("fake-sync") triplets
> within a single `on_data` payload will produce multiple increments — one per embedded
> triplet that triggers a LENGTH-gate entry. If sufficient triplets cross the
> `malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD` threshold, T0814 is emitted. This is
> INTENDED behavior: each fake-sync triplet is a distinct structural probe attempt. The
> per-arm counting rule correctly surfaces adversarial fake-sync floods as the Crain-Sistrunk
> probe class (T0814 Possible/Low). No de-duplication of contiguous-junk runs is needed or
> desired.

---

## R2-SECTION 3: BC-2.15.024 PC1 — Exact Wording + Five-Site Propagation

### The Stale-PC1 Problem

REVISION 1's BC-CHANGE section instructed the product-owner to add the resync arm as a
"FOURTH path" to BC-2.15.024 Precondition 1, stating:

> The three existing reject paths are:
> - BC-2.15.016 Postcondition 2 (carry overflow / LENGTH byte truncation), or
> - BC-2.15.004 validity gate reject (LENGTH<5 or sync!=0x0564), or
> - BC-2.15.009 sync-loss bail (no valid sync in first 16 bytes, is_non_dnp3 set).

This was written against the v1.1 PC1 text, which still included the BC-2.15.009 bail as a
path. The F-F5-004 reconciliation (BC-2.15.024 v1.2, committed 2026-06-11) REMOVED the bail
from PC1 and reduced to TWO paths:

> The two existing malformed-frame reject paths are:
> - BC-2.15.016 Postcondition 2 (carry overflow / LENGTH byte truncation), or
> - BC-2.15.004 validity gate reject (LENGTH<5 or sync!=0x0564).

The resync arm is therefore the THIRD path. The bail must NOT be re-introduced. REVISION 1's
"FOURTH path" framing and its implicit re-statement of the three-path list are both wrong
against v1.2.

### Exact Combined PC1 Text (Third Path — DEFINITIVE)

The product-owner must replace the existing Precondition 1 in BC-2.15.024 with the following
text in full. This is the authoritative post-REVISION-2 PC1:

```
1. One of the structural-reject paths for a malformed DNP3 frame has just fired, causing:
   - `flow.parse_errors += 1` (lifetime counter — already incremented by the reject path;
     this BC does not change that increment), AND
   - `flow.malformed_in_window += 1` (windowed counter — incremented HERE, in parallel).
   The three structural-reject paths are:
   - BC-2.15.016 Postcondition 2 (carry overflow / LENGTH byte truncation), **or**
   - BC-2.15.004 validity gate reject (LENGTH<5 or sync!=0x0564), **or**
   - byte-walk-forward resync arm (sync-loss on an established flow after a clean frame
     consume — non-sync bytes appear immediately after a valid frame boundary; the resync
     arm increments both parse_errors and malformed_in_window exactly once per arm entry
     before performing the byte-walk-forward navigation, as specified in BC-2.15.016 EC-009).
   Each arm entry counts as exactly one structural reject event, regardless of how many
   non-sync bytes are present in the carry at that point. An attacker embedding multiple
   `[0x05, 0x64, invalid-LENGTH]` ("fake-sync") triplets may trigger multiple LENGTH-gate
   entries and therefore multiple increments per `on_data` call — this is INTENDED and
   correct behavior; T0814 (Possible/Low) is the appropriate response when the threshold
   is crossed.
   **NOT INCLUDED:** BC-2.15.009 is_non_dnp3 desync bail. The desync bail is a flow-level
   early-exit for non-DNP3 traffic that fires BEFORE any frame parse begins. Per BC-2.15.009
   Postcondition 3, parse_errors is explicitly NOT incremented on the is_non_dnp3 bail path
   (incrementing it would produce misleading metrics — the flow is simply not DNP3, not a
   malformed-DNP3-frame condition). The is_non_dnp3 bail therefore does NOT satisfy this
   precondition and does NOT feed malformed_in_window. (See also Precondition 6: a bailed
   flow has is_non_dnp3=true; any subsequent on_data call is a no-op — this BC's counters
   are never reached after bail.)
```

### Five-Site Propagation List in BC-2.15.024

The parse_errors-source enumeration appears in five independent sites within BC-2.15.024.
ALL FIVE must be updated in the same commit to remain consistent with the new PC1. Updating
PC1 alone is insufficient — inconsistency between these sites is how the F-B-004 contradiction
arose in the first place.

**Site 1 — Precondition 1 (~line 105):**
Replace with the exact PC1 text above.

**Site 2 — Description "Two-counter model" bullet (~line 78–82):**
The current text reads:
> `parse_errors: u64` — **LIFETIME / monotonic counter**. Incremented on every structurally
> malformed DNP3 frame by the existing reject paths: BC-2.15.016 Postcondition 2 (carry
> overflow / LENGTH byte truncation) and BC-2.15.004 validity-gate reject (LENGTH<5 or
> sync!=0x0564). **NEVER reset at window expiry.**

Replace the reject-path enumeration clause with:
> Incremented on every structurally malformed DNP3 frame by the three structural-reject paths:
> BC-2.15.016 Postcondition 2 (carry overflow / LENGTH byte truncation), BC-2.15.004
> validity-gate reject (LENGTH<5 or sync!=0x0564), and the byte-walk-forward resync arm
> (sync-loss after a clean consume — BC-2.15.016 EC-009). **NEVER reset at window expiry.**

**Site 3 — Invariant 1 (~line 169):**
The current text reads:
> `parse_errors` is fed by BC-2.15.016 Postcondition 2 (carry overflow / LENGTH truncation)
> and BC-2.15.004 validity-gate reject (LENGTH<5 or sync!=0x0564). It is NOT incremented by
> BC-2.15.009 is_non_dnp3 desync bail ...

Replace the feed-list clause with:
> `parse_errors` is fed by three structural-reject paths: BC-2.15.016 Postcondition 2 (carry
> overflow / LENGTH truncation), BC-2.15.004 validity-gate reject (LENGTH<5 or sync!=0x0564),
> and the byte-walk-forward resync arm (sync-loss after a clean consume — BC-2.15.016 EC-009).
> It is NOT incremented by BC-2.15.009 is_non_dnp3 desync bail ...
> (remainder unchanged)

**Site 4 — Architecture Anchors (~line 306):**
The current text reads:
> `src/analyzer/dnp3.rs` — on each structural-reject path (BC-2.15.016 PC2 carry-overflow and
> BC-2.15.004 validity-gate): `flow.malformed_in_window += 1;` (NEW, added alongside existing
> `flow.parse_errors += 1`). The BC-2.15.009 is_non_dnp3 bail does NOT have a
> malformed_in_window increment ...

Replace "on each structural-reject path (BC-2.15.016 PC2 carry-overflow and BC-2.15.004 validity-gate)" with:
> on each of the three structural-reject paths (BC-2.15.016 PC2 carry-overflow,
> BC-2.15.004 validity-gate, and the byte-walk-forward resync arm / BC-2.15.016 EC-009):

**Site 5 — EC-006 (~line 228):**
The current EC-006 text reads:
> Immediate no-op per BC-2.15.009 PC5-6; no parse, no counter increments (parse_errors and
> malformed_in_window are NOT incremented — the bail fires before any frame-parse stage).

This text is correct as written — the bail precedes all three reject paths and does not
interact with them. No change required at EC-006. Confirm only.

### BC-2.15.016 Changes Required (Three Sites)

**BC-2.15.016 EC-007 — add inline-resync-location clarification:**

Current end of EC-007:
> No further `parse_errors` or `malformed_in_window` increment occurs during resync navigation
> — the error was already counted at the LENGTH gate. The carry-clear on no-sync-found does
> NOT set `is_non_dnp3 = true`. Each non-break iteration drains ≥1 byte; carry bounded ≤292
> bytes; loop terminates.

Replace with (adding one sentence before the termination line):
> No further `parse_errors` or `malformed_in_window` increment occurs during resync navigation
> — the error was already counted at the LENGTH gate. The LENGTH-gate arm performs this resync
> navigation INLINE before `continue`, so the loop's next iteration begins with a valid sync
> head or an empty carry; the sync-check arm is NOT entered as a consequence of a LENGTH-gate
> drain (no double-count across iterations). The carry-clear on no-sync-found does NOT set
> `is_non_dnp3 = true`. Each non-break iteration drains ≥1 byte; carry bounded ≤292 bytes;
> loop terminates.

**BC-2.15.016 EC-004 — update overflow carry-state wording:**

Current EC-004 reads (per ADJ-001-A clarification in v1.2):
> 1 byte accepted (total=292); 1 byte discarded; `parse_errors++`

The Canonical Test Vectors row "Carry overflow (adversarial)" currently reads (per ADJ-001-A v1.2 addition):
> `parse_errors++` (the frame-walk then runs: if the carry head is not a valid sync word and
> no `[0x05,0x64]` is found in the carry, byte-walk-forward resync CLEARS the carry — final
> carry may be empty; the 292-cap is proven by the parse_errors increment, not by residual
> carry length)

This wording implied the overflow arm fell through to the frame-walk and relied on the
frame-walk's resync arm to handle the carry — which is the REVISION 1 design that Slice B
rejected. Replace with:
> `parse_errors++`; `malformed_in_window++`; then INLINE resync (within the overflow arm,
> before falling through to the frame-walk): if a `[0x05,0x64]` sync word is found in the
> carry, bytes before it are drained (preserving the valid head frame); if no sync word found,
> carry is cleared. The frame-walk then runs on the repositioned carry. The sync-check arm is
> NOT entered as a consequence of the overflow (no double-count). If carry was all-junk with
> no sync word: final carry is empty; `parse_errors==1`, `malformed_in_window==1`.

Also add EC-004 row update in the Edge Cases table — current EC-004:
> Carry reaches 291 bytes (1 byte short of 292); on_data delivers 2 more bytes
> | 1 byte accepted (total=292); 1 byte discarded; `parse_errors++`

Update to:
> 1 byte accepted (total=292); 1 byte discarded; `parse_errors++`; `malformed_in_window++`;
> overflow arm performs inline resync: if carry head is `[0x05, 0x64, ...]`, carry preserved
> at head; if all-junk with no `[0x05, 0x64]`, carry cleared. Frame-walk runs on result.

**BC-2.15.016 EC-009 (new) — sync-check arm fires and counts junk-at-clean-boundary:**

Add a new row to the Edge Cases table:
> EC-009 | After a clean frame consume, carry head is immediately non-sync (junk injected at frame boundary) | `parse_errors++`; `malformed_in_window++`; byte-walk-forward resync locates next `[0x05,0x64]` or clears carry; if `malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD`, T0814 emitted (BC-2.15.024). This is one structural event. The sync-check arm is entered ONLY from Path B (clean consume → junk head) — it is NOT entered after a LENGTH-gate or overflow-arm reject (those arms perform inline resync that leaves a valid head or empty carry before `continue`).

If EC-009 already exists for other content, use the next available ID. The content is what
matters.

### BC-2.15.016 Should Be Added to STORY-109 Inputs (F-C-002)

`STORY-109.md` currently lists BC-2.15.024 in its inputs but NOT BC-2.15.016. The EC-009
addition to BC-2.15.016 (junk-at-clean-boundary counted by sync-check arm) directly affects
the behavior STORY-109 is implementing. BC-2.15.016 must be added to STORY-109's `inputs:`
list, and STORY-109's `input-hash:` must be regenerated:

```
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.014.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.015.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.016.md   # ADD THIS
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.018.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.019.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.023.md
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.024.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
```

After applying all BC-2.15.016 changes above AND the BC-2.15.024 PC1 change, regenerate
input-hashes for all three stories that list either BC as an input (F-C-001):

```bash
bin/compute-input-hash --write .factory/stories/STORY-107.md
bin/compute-input-hash --write .factory/stories/STORY-109.md
# STORY-108 does not list BC-2.15.016 or BC-2.15.024 in its inputs;
# confirm no change needed (STORY-108 inputs: BC-2.15.010/011/012/013/020/022 + ADR-007 + VP-023).
```

---

## R2-SECTION 4: Complete Fix Specification (Revised)

Three code changes are required. REVISION 1 Changes 1 and 2 are UNCHANGED. Change 3 is
REPLACED.

### Change 1 (UNCHANGED): Resync arm — unconditional increment

At `~lines 378-397`, BEFORE the byte-walk-forward navigation, add unconditional
`parse_errors += 1`, `malformed_in_window += 1`, `check_malformed_anomaly(...)`.

Exact algorithm: as specified in REVISION 1. No modification.

### Change 2 (UNCHANGED): LENGTH-gate arm — inline resync after drain-1

At `~lines 406-417`, after `carry.drain(..1)`, add the byte-walk-forward inline resync
before `continue`.

Exact algorithm: as specified in REVISION 1. No modification.

### Change 3 REPLACEMENT: Overflow arm — inline resync (NOT clear+return)

At `~lines 347-356`, replace the plain `extend_from_slice + parse_errors++ + check_malformed_anomaly`
block with:

```rust
let remaining_capacity = MAX_DNP3_FRAME_LEN - flow.carry.len();
if data.len() > remaining_capacity {
    flow.carry.extend_from_slice(&data[..remaining_capacity]);
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    // Inline resync — identical structure to Change 2 (LENGTH-gate arm).
    // Preserves a valid head frame if present; clears carry if all-junk.
    // Prevents sync-check arm from firing for the same overflow event.
    let next_sync = flow
        .carry
        .windows(2)
        .enumerate()
        .find(|(_, w)| w[0] == 0x05 && w[1] == 0x64)
        .map(|(i, _)| i);
    match next_sync {
        Some(i) => { flow.carry.drain(..i); }
        None    => { flow.carry.clear(); }
    }
    Self::check_malformed_anomaly(flow, &mut self.all_findings, ts, &flow_key);
    // Fall through to frame-walk. Do NOT return early.
} else {
    flow.carry.extend_from_slice(data);
}
```

Key difference from REVISION 1 Change 3: NO `carry.clear()` before `check_malformed_anomaly`.
NO `return`. Instead: inline resync (which clears only if no sync word found), then fall through.

---

## R2-SECTION 5: Revised Tests

### Tests (i) through (iv) from REVISION 1 — UNCHANGED

The first four required tests specified in REVISION 1 (junk-at-clean-boundary, no-double-count
regression, flood-reaches-threshold, and existing threshold test stays green) remain correct
and mandatory. No modification.

### Test (v) — REVISED: carry-cap tests now have a tighter behavioral invariant

`test_carry_buffer_cap_at_292` and `test_EC_003_carry_291_plus_2_overflow`:

Under the new Change 3 replacement: the overflow arm performs inline resync (clears for
all-junk carry). The observable `carry.len() == 0` and `parse_errors == 1` assertions
from ADJ-001-A remain valid. These tests are GREEN under REVISION 2.

However, the implementation team MUST verify that the frame-walk does NOT subsequently
enter the sync-check arm and produce a second increment. Since the overflow arm now clears
the carry (for all-junk input), the frame-walk will find `carry.len() < 3` and break
immediately — the sync-check arm is NOT reached. Assertion: `parse_errors == 1` (not 2).
This is the REVISED tightening for test (v): explicitly assert `parse_errors == 1` (not just
that the test completes) to catch any regression where the overflow arm fails to inline-resync
before falling through to the frame-walk.

### Test (vi) — NEW: overflow with valid head frame preserved

Test name: `test_overflow_arm_preserves_valid_head_frame`

Scenario: Construct a carry state where a valid `[0x05, 0x64, ...]` sync word exists at
some position N > 0 within the carry after the overflow cap. Specifically:

1. Pre-fill `flow.carry` with `[0xAA, 0xAA, ..., 0x05, 0x64, 0x05, 0x44, 0x03, 0x00, 0x01,
   0x00, 0x00, 0x00]` where the `0x05 0x64 ...` sequence begins at index K (e.g., K=2), and
   the total pre-filled length is (292 - 1) = 291 bytes.
2. Deliver `on_data` with 2 bytes of additional junk (`[0xBB, 0xCC]`). `remaining_capacity = 1`.
   Overflow fires: 1 byte accepted (total=292), 1 byte discarded. `parse_errors = 1`,
   `malformed_in_window = 1`.
3. Overflow arm inline resync: `[0x05, 0x64]` found at some position in the 292-byte carry.
   Bytes before it are drained. Carry now begins at `[0x05, 0x64, 0x05, ...]`.
4. Frame-walk runs. If the carry from `0x05 0x64 0x05 0x44 ...` is a complete valid frame
   (frame_len = 10, carry has >= 10 bytes from that point), `frame_count++`.

Assertions:
- `flow.parse_errors == 1` (overflow counted once)
- `flow.malformed_in_window == 1` (no double-count)
- `flow.frame_count >= 1` (valid head frame was PRESERVED and parsed — not silently discarded)
- `flow.carry` state consistent with having consumed the valid frame

This test is the direct regression guard for the REVISION 1 Change 3 data-loss defect. Under
REVISION 1's `carry.clear()+return`, this test would produce `frame_count == 0` (the valid
frame was destroyed). Under REVISION 2's inline-resync, `frame_count == 1`.

### Test (vii) — NEW: fake-sync flood crosses T0814 threshold (Principle 1 confirmation)

Test name: `test_fake_sync_flood_crosses_malformed_threshold`

Scenario: Deliver a single `on_data` payload containing exactly 3 embedded
`[0x05, 0x64, 0x02, 0xAA, 0xAA, 0xAA, ...]` triplets (each `LENGTH=2`, which is invalid,
< 5). No valid frames. Each triplet triggers a LENGTH-gate entry.

Assertions after the call:
- `flow.parse_errors == 3` (one per LENGTH-gate entry — Principle 1 semantics confirmed)
- `flow.malformed_in_window == 3`
- `flow.malformed_anomaly_emitted == true`
- `self.all_findings` contains exactly one T0814 finding with `mitre_techniques: ["T0814"]`
- `flow.frame_count == 0` (no valid frames)

This test confirms that:
(a) The Principle 1 semantics ("per arm entry") are implemented correctly.
(b) The fake-sync-flood T0814 detection path works end-to-end.
(c) De-duplication logic was NOT added (which would have produced `parse_errors == 1`).

---

## R2 Summary

| Item | REVISION 1 | REVISION 2 |
|------|-----------|-----------|
| Change 1 (resync arm increment) | VERIFIED SOUND (F-B-008) | UNCHANGED |
| Change 2 (LENGTH-gate inline resync) | VERIFIED SOUND (F-B-009) | UNCHANGED |
| Change 3 (overflow arm) | REJECTED (F-B-002/F-B-003): clear+return silently discards valid head frame → data loss + evasion DoS | REPLACED: inline resync identical to Change 2; preserves valid head frame; clear only if no sync word found; fall through to frame-walk |
| Semantics invariant | CONTRADICTION (F-B-001): "one per sync-loss event" vs. algorithm "one per arm entry" | RESOLVED: Principle 1 chosen ("one per arm entry"); fake-sync flood counting is INTENDED T0814 behavior; no de-dup |
| BC-2.15.024 PC1 | STALE (F-B-004): written against v1.1 three-path list; called resync arm "fourth path"; would re-introduce bail | CORRECTED: written against v1.2 two-path list; resync arm is THIRD path; bail not mentioned |
| BC propagation | INCOMPLETE (F-C-003): only PC1 + EC-007 + EC-009 | COMPLETE: 5 sites in BC-2.15.024 + 3 sites in BC-2.15.016 (EC-004, EC-007, EC-009) |
| STORY-109 inputs | BC-2.15.016 absent (F-C-002) | BC-2.15.016 added; input-hash regeneration required for STORY-107 and STORY-109 |
| Tests | 4 required + carry-cap regression | 7 required: original 4 + tightened (v) assertion + new (vi) overflow-preserves-head + new (vii) fake-sync-flood |

---

## R2 Product-Owner BC Change List (Complete)

The following changes are required to BC artifacts before the implementer writes any code.
All changes should be applied in a single commit (or burst) to maintain consistency.

**BC-2.15.024 — 5 changes:**

| # | Site | Action |
|---|------|--------|
| PO-1 | Precondition 1 | Replace entirely with the exact PC1 text in R2-SECTION 3 |
| PO-2 | Description "Two-counter model" parse_errors bullet (~line 78) | Replace reject-path enumeration with three-path list including resync arm |
| PO-3 | Invariant 1 "parse_errors is LIFETIME" (~line 169) | Replace feed-list with three-path list including resync arm |
| PO-4 | Architecture Anchors increment-site bullet (~line 306) | Replace "(BC-2.15.016 PC2 carry-overflow and BC-2.15.004 validity-gate)" with three-path list including resync arm |
| PO-5 | EC-006 | Confirm text is correct as written — no change needed |

**BC-2.15.016 — 3 changes:**

| # | Site | Action |
|---|------|--------|
| PO-6 | EC-007 (end of text) | Add inline-resync-location sentence (LENGTH-gate performs resync before `continue`; sync-check arm NOT entered after LENGTH-gate reject) |
| PO-7 | EC-004 (Edge Cases table row) + Canonical Test Vectors "Carry overflow" row | Update to reflect inline resync in overflow arm (not frame-walk resync arm); note valid-head-frame preservation |
| PO-8 | EC-009 (new row in Edge Cases table) | Add sync-check arm junk-at-clean-boundary specification |

**STORY-109.md — 1 change:**

| # | Site | Action |
|---|------|--------|
| PO-9 | `inputs:` frontmatter | Add `- .factory/specs/behavioral-contracts/ss-15/BC-2.15.016.md` |

**Input-hash regeneration — 2 stories:**

| # | Story | Action |
|---|-------|--------|
| PO-10 | STORY-107.md | `bin/compute-input-hash --write .factory/stories/STORY-107.md` (after BC-2.15.016 changes) |
| PO-11 | STORY-109.md | `bin/compute-input-hash --write .factory/stories/STORY-109.md` (after BC-2.15.016 + BC-2.15.024 changes + inputs list update) |

---

## R2 Implementer Code Change List (Complete)

| # | Location | Action |
|---|----------|--------|
| IMP-1 | `~lines 378-397` (sync-check / resync arm) | Unchanged from REVISION 1 Change 1: add unconditional `parse_errors += 1`, `malformed_in_window += 1`, `check_malformed_anomaly(...)` before the byte-walk-forward navigation |
| IMP-2 | `~lines 406-417` (LENGTH-gate arm) | Unchanged from REVISION 1 Change 2: add inline byte-walk-forward resync after `carry.drain(..1)`, before `continue` |
| IMP-3 | `~lines 347-356` (overflow arm) | REPLACE REVISION 1 Change 3: inline resync (windows(2).find) instead of carry.clear()+return; fall through to frame-walk |
| IMP-4 | Test suite | Add tests (i)-(iv) per REVISION 1 TESTS section (unchanged), PLUS tightened (v) assertion (`parse_errors == 1` explicitly), PLUS new (vi) `test_overflow_arm_preserves_valid_head_frame`, PLUS new (vii) `test_fake_sync_flood_crosses_malformed_threshold` |

**Constraint:** IMP-3 must NOT use `carry.clear()` followed by `return`. It must use the
inline resync pattern (windows/find/drain-or-clear) and fall through to the frame-walk.
This is the single most critical structural constraint of REVISION 2.

---

## R2 Authority

REVISION 2 supersedes REVISION 1 in the following respects:
- Change 3 specification (overflow arm algorithm): REVISION 2 governs.
- BC-2.15.024 PC1 wording and propagation site list: REVISION 2 governs.
- BC-2.15.016 EC-004 and EC-009: REVISION 2 governs.
- Semantics invariant language: REVISION 2 governs.
- Test list: REVISION 2 governs (superset of REVISION 1).

REVISION 1 remains authoritative for:
- Change 1 (resync arm increment): verified sound, no modification.
- Change 2 (LENGTH-gate inline resync): verified sound, no modification.
- VP-023 impact analysis: unchanged.
- Path A/B/C analysis and IS-A-GAP reasoning: unchanged and correct.

Issued by the Architect (ADR-007 owner, SS-15 architecture owner). REVISION 2 final.
Product-Owner must apply PO-1 through PO-11 before the implementer applies IMP-1 through IMP-4.
Tests IMP-4 (all seven) are mandatory before F5 convergence gate.
