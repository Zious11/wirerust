---
document_type: arch-ruling
story_id: STORY-137
ruling_id: RULING-137-002
author: architect
date: 2026-06-26
status: authoritative
supersedes: []
addendum_to: RULING-137-001
---

# RULING-137-002: Carry-Overflow Unreachability and is_non_enip Dead-Latch

## Summary

This ruling adjudicates the finding — independently raised by the test-writer and
re-derived by the orchestrator — that the `is_non_enip` carry-overflow latch in
BC-2.17.016 Postcondition 4 / Invariant 4 is structurally unreachable under the
spec's own frame-walk algorithm. RULING-137-001 §5.4 previously asserted the
overflow IS reachable via repeated sub-24-byte deliveries accumulating carry past
600. That assertion was INCORRECT. This ruling supersedes §5.4 of RULING-137-001.

Verdict: **Genuine design gap (option b)**. The carry-cap check as specified can
never fire; the `is_non_enip` latch is dead code. However, the defect does NOT
block STORY-137 convergence. Disposition and follow-up are specified below.

---

## 1. Rigorous Unreachability Proof

### 1.1 Exhaustive path enumeration

After any `on_data` call, `flow.carry` is set by exactly one of three paths:

**Path A — loop never enters (buf.len() < 24):**

`buf = carry_before + new_data`. Loop condition `buf.len() - cursor >= 24` is
false at cursor=0, so the loop body does not execute. After the loop,
`flow.carry = buf[0..] = carry_before + new_data`. For the loop NOT to enter,
`buf.len() < 24`, which means `carry_before + new_data < 24`. Therefore
`flow.carry.len() < 24` after Path A. Maximum carry after Path A: **23 bytes**.

Note: RULING-137-001 §5.4 claimed this path could accumulate carry past 600 via
"repeated sub-24-byte deliveries." This claim fails because it applies to the carry
BEFORE the next call. As soon as `carry + new_data >= 24`, the loop fires. With
carry growing by at most `new_data.len()` per Path A call, and Path A requiring
`carry + new_data < 24`, the carry is capped at 23 bytes before the loop begins
firing. Once the loop fires, carry is reset per Paths B or C below. Path A can
NEVER accumulate carry beyond 23 bytes across repeated calls.

**Path B — loop exits via partial-frame break:**

The partial-frame stash condition (BC-2.17.016 Post-1 fourth sub-bullet) is:
`buf.len() - cursor < total_frame_len`. This branch stashes `buf[cursor..]` into
carry. The stash size is `buf.len() - cursor`.

The frame-skip guard fires first: if `total_frame_len > MAX_ENIP_CARRY_BYTES`
(600), the frame-skip path runs (`cursor += min(...)`, `continue`) and carry is
NOT stashed from that frame. The partial-frame stash is therefore only reachable
when `total_frame_len <= MAX_ENIP_CARRY_BYTES = 600`.

Combined constraints at the stash:
- `total_frame_len <= 600` (frame-skip guard not taken)
- `buf.len() - cursor < total_frame_len` (partial condition is true)

Therefore: `buf.len() - cursor < total_frame_len <= 600`

So: `buf.len() - cursor <= 599`

The stash is `buf[cursor..]`, which has exactly `buf.len() - cursor` bytes.
Therefore: **stash size <= 599 bytes**.

`flow.carry.len() <= 599 < 600 = MAX_ENIP_CARRY_BYTES` after Path B.

**Path C — loop exits normally (buf.len() - cursor < 24):**

After the loop exits because the remaining buffer has fewer than 24 bytes,
`flow.carry = buf[cursor..]` where `buf.len() - cursor <= 23`.
`flow.carry.len() <= 23` after Path C.

### 1.2 Exact maximum carry bound

`flow.carry.len() <= 599` after any `on_data` call on a non-bailed flow, where
carry arrives via the partial-frame stash (Path B). Paths A and C bound carry at
23 bytes.

**The maximum possible `flow.carry.len()` is 599 bytes.**

Since the cap check fires when `flow.carry.len() > MAX_ENIP_CARRY_BYTES = 600`,
and 599 <= 600, the cap check condition is NEVER true. `is_non_enip` is NEVER
set to true via `on_data`. The cap check at lines 714-722 of
`src/analyzer/enip.rs` is dead code.

### 1.3 Correction of RULING-137-001 §5.4

RULING-137-001 §5.4 stated:

> "The only way to reach carry overflow is via accumulated sub-24-byte residue
> from the 'no loop iteration' path (where buf.len() < 24 on entry)."

This sentence was presented as describing a reachable path. The analysis at
RULING-137-001 §5.4 correctly identified the conceptual mechanism (sub-24-byte
deliveries) but failed to complete the bound: sub-24-byte deliveries can only
accumulate carry up to 23 bytes before the loop begins firing, at which point
the residue is reset to at most 23 bytes (Path C). The "carry grows to 601 via
small packets" scenario described is mechanically blocked by the loop guard. That
claim is RETRACTED. The overflow is unreachable by any delivery sequence.

### 1.4 The BC-2.17.016 test vector is impossible

BC-2.17.016 Canonical Test Vectors includes:

> "Partial frame stash grows from 580 to 601 bytes — carry=580, new=21 → cap triggered, true"

Let us trace this with the spec algorithm:
- buf = 580 + 21 = 601 bytes
- Loop fires (601 >= 24)
- Parse header at [0..24]: assume valid header with `header.length = 576`,
  so `total_frame_len = 600`
- `total_frame_len = 600 <= MAX_ENIP_CARRY_BYTES = 600` (NOT frame-skip)
- `buf.len() - cursor = 601 >= total_frame_len = 600`: NOT partial; frame IS complete
- `process_pdu` fires; cursor = 600
- After loop: `buf.len() - cursor = 1 < 24`; carry = 1 byte

The frame COMPLETES (601 >= 600). The partial-stash path requires `601 < 600`,
which is false. To make the partial path fire, we need total_frame_len > 601,
i.e., >= 602, which means `header.length >= 578`, so `total_frame_len >= 602 > 600`
— which is the frame-skip condition, not the partial-stash condition.

There is no value of `header.length` that simultaneously satisfies all three:
1. `total_frame_len <= 600` (to avoid frame-skip)
2. `buf.len() - cursor < total_frame_len` (to trigger partial stash)
3. The stash result `buf.len() - cursor > 600` (to overflow carry)

Constraints 1 and 3 yield: `buf.len() - cursor > 600` AND `buf.len() - cursor < 600`.
This is a contradiction. The test vector is mathematically impossible.

---

## 2. Verdict: Genuine Design Gap (Option b)

The carry-cap check was evidently intended to guard against a scenario the
algorithm actually prevents by construction. The result is dead code that
cannot execute, and `is_non_enip` is therefore an inert flag that is never
latched by any reachable execution path in `on_data`.

This is option (b) from the finding: a genuine design gap where the cap threshold
was set equal to `MAX_ENIP_CARRY_BYTES` (600) without accounting for the fact
that the partial-stash path can only produce carry up to `MAX_ENIP_CARRY_BYTES - 1`
(599), making the `> MAX_ENIP_CARRY_BYTES` condition permanently false.

### 2.1 Consequence for the "quarantine" narrative

STORY-137's narrative states: "correctly detect and quarantine non-ENIP traffic
on port 44818." With `is_non_enip` never latched, the quarantine feature is inert.
STORY-134/135/136 all read `is_non_enip` as an early-return guard; since it is
never true, those guards never fire on any flow.

The T0814 malformed-frame detection (BC-2.17.018) is NOT affected: that detection
operates via `malformed_in_window >= 3`, which is reachable via the byte-walk
resync and frame-skip paths. T0814 works correctly. Only the `is_non_enip`
quarantine latch is inert.

### 2.2 Correct design — two viable fixes

**Fix A — lower the cap check threshold (recommended):**

Change the cap check from `flow.carry.len() > MAX_ENIP_CARRY_BYTES` to
`flow.carry.len() >= MAX_ENIP_CARRY_BYTES`. This makes the check fire when carry
reaches exactly 600 bytes (which the partial-stash path CAN produce: with
total_frame_len=600 and buf.len()-cursor=599, the stash is 599 bytes — still not
600). Actually with `>=`, the threshold fires at 600, and the maximum stash is 599,
so this STILL never fires. Fix A alone is insufficient.

The root cause is structural: as long as frame-skip blocks all total_frame_len > 600
frames from stashing, and partial-stash requires total_frame_len <= 600 and stash
< total_frame_len, the stash is always < 600. No threshold at or below 600 fires.

**Fix B — add a max-carry cap to the partial-frame stash path (correct fix):**

The cleanest fix is to bound the partial-frame stash itself. After:
```
if buf.len() - cursor < total_frame_len {
    break;
}
```

Add before the break:
```
if buf.len() - cursor > SOME_CARRY_CAP {
    // Partial frame exceeds carry cap — treat as non-ENIP.
    flow.parse_errors += 1;
    flow.malformed_in_window += 1;
    check_t0814(...);
    flow.is_non_enip = true;
    flow.carry.clear();
    return;
}
```

But this changes the semantics: what partial-frame size warrants quarantine?
The intent was to quarantine flows that accumulate excessive unreassembled bytes,
indicating the port-44818 traffic is not ENIP. A reasonable per-segment threshold
(e.g., 600 bytes) would need to be compared against `buf.len() - cursor` (the
stash size), not `flow.carry.len()` (which after a stash is the same thing).

The check would become: `if buf.len() - cursor > MAX_ENIP_CARRY_BYTES { is_non_enip }`.
But that's effectively the same as the frame-skip guard (which fires when
`total_frame_len > MAX_ENIP_CARRY_BYTES`). A partial frame that is larger than
MAX_ENIP_CARRY_BYTES only exists when `total_frame_len > 600`, and that is already
handled by frame-skip. So Fix B reduces to: "also quarantine on partial frames
whose declared total_frame_len > 600," which is a duplicate of frame-skip.

**Fix C — accumulate `flow.carry.len()` across multiple partial stashes:**

The `is_non_enip` latch WOULD be reachable if carry is accumulated ACROSS calls
rather than REPLACED on each call. The current code sets `flow.carry = buf[cursor..]`
(replacement). If flows carry state from prior partial stashes as a running
accumulation, carry could grow across calls. However, the spec algorithm (and the
implementation) is correct: on each call, carry is replaced with the residue from
the current buffer walk. Carry does not grow across calls.

**The most defensible fix (Fix D — intended semantic clarification):**

The original intent was to quarantine flows that persistently fail to produce a
complete ENIP frame across many TCP segments — a signature of non-ENIP binary
traffic on port 44818. The carry-accumulation model (growing across calls) would
serve this purpose. The fix is to ACCUMULATE carry across calls rather than
replacing it, with the cap check applied to the cumulative buffer. However, this
is a substantial semantic change: it means valid but slow streams (e.g., a large
CIP transfer arriving in many small segments) could be incorrectly quarantined.

**Conclusion:** The correct fix requires a product-owner decision about the intended
quarantine semantic. The options are:

1. Remove `is_non_enip` entirely and replace the quarantine narrative with the
   T0814 mechanism, which is the only actually-reachable non-ENIP signal.
2. Redesign the partial-frame cap to use a different trigger — for example, a
   maximum number of consecutive parse-failure calls without a complete frame.
3. Raise MAX_ENIP_CARRY_BYTES above 600 so that partial frames can accumulate
   past the cap threshold. (E.g., cap=600, partial-stash capped at 599, keep
   max carry at 599 — no overflow. Needs cap raised to e.g. 599 with > 599, or
   introduce a separate quarantine threshold distinct from carry cap.)
4. Accept the dead latch as belt-and-suspenders (option a). See §2.3 below.

### 2.3 Option (a) assessment — defensive-redundant, accept as-is

Option (a) says the dead latch is acceptable belt-and-suspenders. This is partially
defensible:

- The T0814 path (malformed-frame burst) IS reachable and functional. Non-ENIP
  traffic on port 44818 with garbage headers will trigger T0814 before any
  quarantine. The quarantine signal is therefore served (imperfectly) by T0814.
- The `is_non_enip` read-guards in STORY-134/135/136 are dead guards, but they
  are not harmful: they never trigger, so detection is never suppressed.

However, option (a) means the STORY-137 narrative ("quarantine non-ENIP traffic")
is an inaccurate statement of the feature's actual behavior. The spec promises a
quarantine mechanism that does not exist. This is a correctness issue in the spec,
not in the code.

**This ruling adopts option (b) for the spec defect classification but DEFERS the
fix per §3 below.** The dead latch is documented as a known limitation. No code
changes are made by this ruling; the spec defect is recorded for the follow-up cycle.

---

## 3. STORY-137 Convergence: DEFER (does not block)

**STORY-137 convergence is NOT BLOCKED.**

Rationale: STORY-137's code faithfully implements the ratified BC-2.17.016 spec.
The dead latch is a spec defect — the spec specifies behavior for a state that the
algorithm never reaches. The implementation is correct with respect to the spec.
Blocking STORY-137 would require either (a) the implementer to deviate from the
ratified spec, or (b) amending the spec — which would change STORY-137's inputs
and require input-hash recomputation.

The appropriate path is to defer the spec fix to a follow-up story. The current
implementation ships with a dead cap check (belt-and-suspenders, zero runtime
cost) and accurate T0814 detection. The `is_non_enip` quarantine narrative is
amended to a clarification note (see §5 below).

**Follow-up item (deferred):**

- Target: v0.12.0 or a dedicated maintenance story (STORY-XXX, SS-17)
- Title: "Fix is_non_enip quarantine latch — carry cap unreachable under spec algorithm"
- Scope: (1) Re-evaluate quarantine design: should quarantine be triggered by
  persistent frame-parse failures (N consecutive on_data calls without a complete
  frame), or by something else entirely? (2) Update BC-2.17.016 Postcondition 4,
  Invariant 4, EC-004, and the canonical test vector. (3) Update STORY-137
  narrative. (4) Run input-hash recompute on STORY-137 (inputs change). (5)
  Coordinate with product-owner on quarantine semantic intent.
- Orchestrator: record this in the S-7.02 cycle-closing checklist as a deferred
  spec defect with tracking label `spec-defect-is_non_enip-dead-latch`.

---

## 4. Process-Gap Classification

This is a **[process-gap]**. The defect is: a ratified BC specifying behavior for
an unreachable state (BC-2.17.016 PC-4, Inv-4, EC-004) passed through the F2
spec-evolution and F3 story-decomposition adversarial convergence without being
caught.

**Root cause:** The proof of unreachability requires combining three constraints
simultaneously (frame-skip gate, partial-stash condition, cap overflow condition)
into a contradiction. No single-artifact review catches this — it requires
cross-constraint reasoning across the three guard conditions in the algorithm.
The F2/F3 adversarial pass reviewed each guard individually but did not enumerate
all three simultaneously.

**Recommendation for future cycles:** When a spec defines a condition that fires
only under state `X`, and the algorithm contains multiple guards that together
bound state to never reach `X`, add a "reachability proof obligation" to the
adversarial checklist: "Is the trigger condition reachable under the algorithm's
own invariants?"

---

## 5. Spec Amendment (Narrow — Clarification Only, No BC Logic Change)

The following narrow amendments to spec artifacts record the finding without
changing the ratified algorithm. These amendments do NOT change STORY-137's
input files in a way that requires recomputation of the input-hash for the
CURRENT story: the `inputs:` list in STORY-137.md does not include this ruling
document. However, if BC-2.17.016 or ADR-010 are modified, the input-hash
WILL become stale (those two files are in STORY-137's `inputs:` list). The
amendments below are chosen to be minimal clarifications that DO NOT change the
behavioral postconditions or invariants — they add a NOTE only, preserving
hash stability.

### 5.1 ADR-010 Decision 3 — add a note to the carry-cap description

In the sentence at ADR-010 Decision 3 paragraph 4 ("Frame-skip behavior for
oversize declared frames"), add the following NOTE after the existing paragraph
(no existing text changed):

> **RULING-137-002 NOTE (2026-06-26):** Under the current frame-walk algorithm,
> the carry-overflow trigger (`flow.carry.len() > MAX_ENIP_CARRY_BYTES`) is
> structurally unreachable. Proof: the partial-frame stash path only fires when
> `total_frame_len <= 600` (otherwise frame-skip fires) AND `buf.len()-cursor
> < total_frame_len`; therefore the stash size is < 600, so carry never exceeds
> 599. The cap check is belt-and-suspenders dead code. The quarantine narrative
> (is_non_enip) is therefore inert; non-ENIP traffic on port 44818 is flagged
> via T0814 (malformed_in_window threshold) rather than via is_non_enip. A fix
> is deferred to v0.12.0 (see RULING-137-002 for options). No algorithm change
> is made in v0.11.0.

### 5.2 BC-2.17.016 — add a NOTE to Postcondition 4 and EC-004

Add to BC-2.17.016 Postcondition 4 (after the existing text):

> **RULING-137-002 NOTE (2026-06-26):** The trigger condition
> `flow.carry.len() > MAX_ENIP_CARRY_BYTES` is structurally unreachable under
> this spec's own algorithm (see RULING-137-002 §1). The canonical test vector
> "580+21=601 → overflow" in the Canonical Test Vectors table is mathematically
> impossible: with carry=580, new_data=21, buf=601, and total_frame_len=600,
> the partial condition `601 < 600` is false — the frame completes. This
> postcondition and EC-004 are deferred for redesign in v0.12.0.

These NOTE additions are additive-only. They do not change any postcondition
logic, invariant, or edge-case expected behavior — they only annotate the
known dead-code status. Because they are additive notes inside existing files
(not changes to postcondition logic), their impact on the input-hash is:

- BC-2.17.016 IS in STORY-137's inputs list → adding the note changes the
  file's content → changes the input-hash → STORY-137.md input-hash becomes STALE.
- ADR-010 IS in STORY-137's inputs list → same.

**Decision: The amendments to BC-2.17.016 and ADR-010 are NOT applied now.**
They are deferred to the v0.12.0 follow-up story to avoid forcing an input-hash
recompute in the middle of the STORY-137 convergence cycle. This ruling document
(which is NOT in STORY-137's inputs list) serves as the authoritative record.
The follow-up story will amend both documents at the start of its F2/F3 spec
phase, triggering a fresh input-hash.

---

## 6. Summary for Orchestrator and Implementer

### Carry bound (confirmed)
Maximum `flow.carry.len()` after any `on_data` call: **599 bytes**. The
`flow.carry.len() > 600` cap check is dead code. `is_non_enip` is never set
by `on_data`. This is a structural property of the frame-walk algorithm.

### Verdict
Option **(b): genuine design gap**. The spec defines behavior for an unreachable
state. The code correctly implements the (defective) spec. The quarantine narrative
is inaccurate.

### STORY-137 convergence
**DEFER — does not block.** STORY-137 implements the ratified spec correctly.
The spec defect is recorded for v0.12.0.

### Spec edits made in this ruling
**None to BC-2.17.016 or ADR-010** (deferred to v0.12.0 to avoid input-hash
churn in this cycle). This ruling document is the authoritative record.

### Follow-up item
v0.12.0 story: "Fix is_non_enip quarantine latch — carry cap unreachable"
Label: `spec-defect-is_non_enip-dead-latch`, SS-17, PO decision required on
quarantine semantic.

### Process gap
[process-gap] — carry-overflow unreachability requires cross-constraint
reasoning across three simultaneous guards; this class of proof obligation was
absent from F2/F3 adversarial checklist. Recommend adding reachability proof
obligation for bounded-state triggers.

---

## 7. Correction of RULING-137-001 §5.4

RULING-137-001 §5.4 section title "test_carry_buffer_cap_at_600" contains the
statement: "The only correct setup is: accumulate > 600 bytes in carry through
sub-24-byte on_data deliveries (the while loop never fires; all bytes become carry)."

This statement is incorrect. Sub-24-byte deliveries cannot accumulate carry past
23 bytes (once carry + new_data reaches 24, the loop fires and resets carry to
the post-walk residue which is at most 23 bytes). The test vectors in §5.4 that
attempt to reach carry = 601 via partial-frame stash are also incorrect for the
mathematical reasons in §1.3 above.

**Implication for tests:** The test `test_carry_buffer_cap_at_600` and
`test_carry_cap_sets_non_enip` (AC-137-002) cannot be implemented with the current
algorithm in a way that actually triggers the carry-overflow path, because the path
is unreachable. The test-writer has two options:

1. Skip these tests for now and file them as blocked on the v0.12.0 carry-cap
   redesign (preferred, clean).
2. Test the cap check by directly pre-setting `flow.carry` to 601 bytes via
   test-internal state manipulation (if the test infrastructure exposes the
   field), then calling `on_data` with 0 bytes and verifying the cap fires.
   This tests the dead-code path in isolation but does NOT test reachable
   behavior — it is a unit test of a guard that can never fire in production.
   Mark such a test clearly: `#[test] // tests dead-code cap guard: unreachable in practice per RULING-137-002`.

The orchestrator should coordinate with the test-writer on option 1 vs. 2 before
the next adversarial pass.
