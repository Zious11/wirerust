---
document_type: f5-lessons
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-06-12T15:23:29Z
cycle: "feature-8-dnp3-v0.5.0"
phase: feature-f5
---

# F5 Remediation Lessons — Feature #8 DNP3

---

## (a) DIR-bit Bug Survived ~30 Per-Story Passes — Holdout/Canonical-Frame Cross-Validation

**Tag:** `[process-gap]` PG-F5-DIRBIT-001

**What happened:** The `is_master_frame` DIR-bit mask bug (0x10 instead of 0x80) was
latent since STORY-107 (wave 36). It survived approximately 30 per-story adversarial
passes because the tests and BC-2.15.016 PC5 both used the same wrong mask. The error
was internally self-consistent — every per-story pass reviewed the code and the BC as a
unit and found them agreeing with each other.

The F5 holistic review + the agentic-sliced pre-implementation adversarial review
(fresh context, different model family, cross-validated against the DNP3 Layer-2 spec)
found it in one pass.

**Lesson:** Per-story adversarial passes with self-consistent (wrong) tests and BCs
cannot detect spec-level errors. Holdout scenarios with canonical protocol frames
(e.g., CONTROL=0xC4 master frame from the DNP3 spec) and cross-validated F5 holistic
reviews ARE the mechanism for catching this class of defect.

**Rule:** For any protocol-framing invariant (direction bits, function codes, frame
types), at least one holdout scenario MUST use a canonical byte sequence from the
protocol spec — not a test-derived value. This cross-validates the implementation
against the spec, not just against itself.

---

## (b) Adversary Checkout-Guard Failure (P9)

**Tag:** `[process-gap]` PG-F5-CHECKOUT-001

**What happened:** Convergence pass P9 reviewed develop HEAD instead of the feature
worktree. Its BLOCKER findings were invalid (they described code that the worktree had
already fixed). This wasted one convergence pass and required explicit re-dispatch on
the correct worktree (P10), which returned CLEAN.

**Lesson:** Convergence passes on a worktree MUST attest the worktree HEAD (not develop)
and include a fix-content check (confirm that fixes from prior passes are present in the
reviewed code) before trusting findings.

**Rule:** Mandatory checkout-guard protocol for adversary dispatch on worktree reviews:
1. Adversary MUST log: `git -C <worktree> log -1 --format='%h %s'` at pass start.
2. Adversary MUST confirm that fixes from the prior BLOCKER pass are visible in the
   reviewed file (content spot-check).
3. If either check fails, the pass is INVALID and must be re-run. The invalid pass is
   recorded as [process-gap] in convergence-trajectory, not as a valid BLOCKED finding.

---

## (c) Pre-Implementation Agentic-Sliced Adversarial Review Caught 2 Design BLOCKERs Before Coding

**Tag:** `[pattern-confirmation]` PG-F5-PREIMPL-001

**What happened:** Before TDD implementation, an agentic-sliced adversarial review was
run in 3 parallel slices (F-001 design, F-003 design, spec-consistency). The review
found 2 design BLOCKERs — the DIR-bit bug (F-A-001) and the overflow data-loss
accounting gap (F-F5-003) — BEFORE any code was written.

Both findings required ARCHITECT REVISION-2 directives (the original designs were
unsound). Catching them pre-coding saved rework: no broken code was written, no
test suite had to be un-done, and the implementation converged faster (10 passes vs
an estimated 15+ if the design flaws had been discovered post-coding).

**Lesson:** For high-risk or complex designs (multi-pass byte walks, detection logic
with security invariants), running an adversarial design review BEFORE TDD is high
leverage. The cost is 1 extra pass; the benefit is avoiding design-rework mid-convergence.

**Rule:** When a new F5 scoped adversarial review identifies multiple potential design
issues, dispatch a parallel agentic-sliced pre-implementation review (2-4 slices by
concern area) before TDD authoring begins. Make this a default step for F5 when
detected issues touch: protocol framing invariants, detection logic, or security-
invariant BCs.
