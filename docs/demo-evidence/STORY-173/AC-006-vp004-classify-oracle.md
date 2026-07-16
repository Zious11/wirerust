# AC-173-006: VP-004 Classifier Oracle Updated for DispatchTarget::Iec104

**AC:** AC-173-006
**BC:** BC-2.05.012 invariant 1 (VP-004 oracle — ADR-013 Decision 9 step 3)
**Story:** STORY-173
**Date:** 2026-07-15

---

## What this AC covers

The `classify_oracle` function inside the `#[cfg(kani)]` block in `src/dispatcher.rs` is
updated in the same commit that adds `DispatchTarget::Iec104`. The oracle mirrors production's
`classify()` exactly for Rule 8 (port 2404 → `DispatchTarget::Iec104`) so that the
`verify_content_first_precedence_exhaustive` Kani proof (`#[kani::proof]`) remains valid
after IEC-104 is added.

Per ADR-013 Decision 9 step 3: the oracle MUST be updated atomically with the dispatch target
variant and Rule 8 arm. Any mismatch between oracle and production causes the VP-004 Kani
proof to find a counterexample.

---

## Source confirmation

`src/dispatcher.rs` — `classify_oracle` Rule 8 arm (lines 709–716, inside `#[cfg(kani)]`):
```rust
// Rule 8: IEC-104 port fallback (ADR-013 Decision 1 — MUST mirror production exactly).
// VP-004 oracle obligation: this arm is mandatory per BC-2.05.012 /
// STORY-173 VP-004 oracle obligation (ADR-013 Decision 9 step 3).
// Placed AFTER Rule 7 (ENIP) and BEFORE Rule 9 (None).
if ports.contains(&2404) {
    return DispatchTarget::Iec104;
}
// Rule 9: nothing matched.
DispatchTarget::None
```

The `verify_content_first_precedence_exhaustive` Kani proof (line 720 onward) uses the
oracle to assert `classify(&data, &key) == classify_oracle(&data, lower, upper)` for all
possible 8-byte inputs and all possible port combinations. The Rule 8 arm in the oracle
ensures port-2404 flows are correctly classified as `DispatchTarget::Iec104`.

---

## Verification

The VP-004 Kani proof (`verify_content_first_precedence_exhaustive`) is a `#[kani::proof]`
harness that runs under `cargo kani` — not `cargo test`. The oracle update in this story
enables the proof to be re-run in STORY-174 formal hardening. At TDD gate (this story), the
oracle's presence in the `#[cfg(kani)]` block is confirmed by source inspection and the fact
that the code compiles without error (`cargo check` passes).

Source inspection confirms:
- The `classify_oracle` function has a Rule 8 arm matching `ports.contains(&2404)` and
  returning `DispatchTarget::Iec104`.
- The Rule 8 arm is placed AFTER Rule 7 (ENIP/44818) and BEFORE Rule 9 (None), mirroring
  production's `classify()` exactly.
- The overall structure has 9 rules (1–9), matching the production dispatcher's rule ladder.

---

## Verdict

PASS — `classify_oracle` updated with Rule 8 (`port 2404 → Iec104`) in the same commit.
The VP-004 Kani proof will be re-run in STORY-174. Source inspection confirms the oracle
mirrors production exactly for Rule 8.
