# Lessons Learned — maint-2026-07-11

**Run ID:** maint-2026-07-11
**Date:** 2026-07-11
**Author:** session-reviewer (state-manager record)
**Format:** [codified] = AC added to a story; [register] = tech-debt row; [observation] = noted but not yet codified

---

## Lessons

### L-1 — BREAKING-Change Holdout Sweep Obligation (PG-W72-BREAKING-HOLDOUT-SWEEP)

**Status:** [codified] → STORY-164 AC-164-005

**Observation:** Wave-72 Lesson-2 (PROP-V0.12.0-01, wave-72/lessons.md) established that before any PR modifying a CLI flag, serialization format, or exit code is merged, all holdout scenarios exercising that surface MUST be swept for expectation staleness. This obligation — "BREAKING-change holdout sweep" — was identified as a process gap (no story AC enforced it) after a wave-72 PR was merged without discovering that downstream holdout scenarios expected the old behavior. The gap was captured as a lesson in wave-72 but no AC had been authored to formally encode it in the pipeline.

**Root cause:** Wave-72 lessons.md noted the gap, but the lesson did not have a corresponding behavioral contract AC in any story, leaving it as advisory prose only. Without a concrete deliverable, the obligation could be silently omitted in future feature waves.

**Codified:** STORY-164 v1.1 AC-164-005 — author `.factory/maintenance/breaking-change-delivery-protocol.md` (scope: any PR touching CLI args, JSON schema, or exit codes; mandate: sweep all holdout scenarios for that surface before merge; CLAUDE.md reference row added). Wave-72 Lesson-2 formally codified. Points 3→4.

---

### L-2 — Register Symbol-Grep Error: Proposed vs. Shipped Identifier

**Status:** [observation] — no codification required (1st occurrence, correction logged in register)

**Observation:** ISSUE-102-PREMATURE-CLOSE-001 was filed during maint-2026-07-09 backfill triage as a P2 finding because the triage grep searched for `MAX_WEAK_CIPHER_EVIDENCE` (the identifier name used in the GitHub #102 mitigation proposal) but the shipped implementation used `WEAK_CIPHER_EVIDENCE_CAP`. The grep returned zero results, leading to the incorrect conclusion that no cap existed and that #102 had been prematurely closed. DF-VALIDATION-001 research-agent pass (maint-2026-07-11) confirmed the cap is present at `tls.rs:635` as `WEAK_CIPHER_EVIDENCE_CAP: usize = 64` with `.take(64)` and "+N more" elision. Root cause: backfill triage grepped for a proposed symbol name (from the issue body) rather than verifying existence via `grep -r "cap\|limit\|max"` pattern scans across the file. This is the first occurrence of this class of error.

**Lesson:** When searching for a missing feature or cap in the source tree during triage, do not grep for the proposed identifier from the issue/mitigation text — grep for the semantic pattern (e.g., `usize = NN`, `take(`, bounded-output patterns) across the relevant file. Proposed names frequently differ from shipped names. Register row ISSUE-102-PREMATURE-CLOSE-001 updated to CLOSED-REFUTED.

**No story needed:** First occurrence, correction fully logged in register. If a second occurrence happens, promote to a formal triage-checklist AC.

---

### L-3 — Engine Recurrences: Relay Unreliable + Checkout-Guard (ADVERSARY-RELAY-UNRELIABLE-001, DRIFT-ENGINE-CHECKOUT-GUARD-001)

**Status:** [register] — both rows updated with recurrence notes; no new codification (prior stories covered the engine fixes)

**Observation (relay):** ADVERSARY-RELAY-UNRELIABLE-001 recurred a 3rd time during this run: the sweep-4 holdout-freshness agent wrote its sweep artifact to disk but went idle without relaying findings back to the orchestrator. Required synchronous re-dispatch. Prior workaround (documented in register) remains in effect: run adversary/holdout agents with `run_in_background: false` during maintenance passes. No new engine fix has shipped.

**Observation (checkout-guard):** DRIFT-ENGINE-CHECKOUT-GUARD-001 recurred during PR #396 adversary Pass 1: the adversary agent reviewed `develop` HEAD instead of the PR branch, producing a VOID pass. The pass was voided, checkout-guard applied, and the adversary re-dispatched. This is the 2nd occurrence of this class. The engine prompt fix (embed DF-ADVERSARY-CHECKOUT-GUARD-001 verbatim as first instruction block) has still not shipped.

**Lesson:** Both recurrences were handled correctly by the orchestrator — VOID declared, re-dispatch executed, no false findings accepted. The engine fixes remain outstanding dark-factory items. Register rows updated with recurrence counts. No new wirerust story needed; these are engine-level (dark-factory) fixes, not product changes. The pattern suggests the engine fix priority should be raised before the next feature wave where adversary dispatch will be frequent.

---

### L-4 — Single-Account Merge Constraint: Admin-Bypass Declined in Auto-Mode

**Status:** [observation] — friction on interim merge path; no new codification (PG-MERGE-AUTH interim path documented by prior stories); carry to session review

**Observation:** PR #396 required human merge because the harness permission classifier declined `gh pr merge --admin` when executed by the orchestrator subagent in auto-mode. The classifier correctly refused: required-review bypass via `--admin` requires explicit human authorization that must be visible in the calling agent's conversation thread, not relayed from a teammate message. This is a known single-account limitation (required-review rule is unsatisfiable in a single-account repo; the only bypass is admin, which the classifier declines in auto-mode).

**Friction:** The interim path is: (1) orchestrator requests human merge, (2) human manually runs `gh pr merge` or uses the GitHub web UI. This was the path used for PR #393 (maint-2026-07-09) and PR #396 (this run). It works but adds latency (human availability) and breaks fully-automated pipeline runs.

**Lesson:** The PG-MERGE-AUTH interim path (STORY-163 AC-163-002, DF-MERGE-AUTH-CLASSIFIER-001) is working as designed — classifier denial is the correct behavior. The friction is inherent to the single-account setup. Future options: (a) add a second GitHub account as a reviewer collaborator (medium effort, permanent fix); (b) add an explicit machine-readable AUTHORIZE_MERGE=yes directive that the classifier accepts as a main-conversation signal (low effort, config change). Option (a) was discussed during wave-71 and deferred. Carry this observation to the next session review for human disposition.

---

## Summary Table

| ID | Type | Codified? | Story/AC |
|----|------|-----------|----------|
| L-1 | BREAKING-change holdout sweep obligation | YES | STORY-164 AC-164-005 |
| L-2 | Register symbol-grep error (proposed vs. shipped identifier) | Observation (no story) | Register ISSUE-102-PREMATURE-CLOSE-001 CLOSED-REFUTED |
| L-3 | Engine recurrences: relay + checkout-guard | Register (no new story) | Register ADVERSARY-RELAY-UNRELIABLE-001 + DRIFT-ENGINE-CHECKOUT-GUARD-001 |
| L-4 | Single-account merge constraint: admin-bypass declined | Observation (carry to session review) | — |
