---
document_type: lessons-learned
cycle: feature-arp-v0.7.0
producer: state-manager
timestamp: 2026-06-15T00:00:00Z
---

# Lessons Learned — feature-arp-v0.7.0

Process-gap items recorded per S-7.02 codification discipline. Each item needs
a follow-up story OR a justified deferral entry. Items flagged `[process-gap]`
below are candidates for policy codification.

---

## [process-gap] PG-ARP-F4-REDBANNER-SWEEP

**Source:** STORY-112 Step-4.5 adversarial convergence (multiple comment-fix bursts)

**Observation:** The RED-gate banner sibling-sweep was missed across 3 successive
comment-fix bursts. The module docstring was fixed (F-2), but per-test section
banners inside `bc_2_16_story112_arp_tests.rs` were left stale — including the
AC-004 banner block, which a later fresh adversarial pass escalated from MEDIUM
to HIGH.

This is a recurrence of DF-SIBLING-SWEEP-001 in the doc-comment dimension.
When a module-level status changes (RED→GREEN), the sweep must enumerate:

1. Module-level docstrings
2. Per-test section banners (all of them, not just the first encountered)
3. Any inline `// RED GATE` / `// TODO` comments referencing the transitional error string
4. Story frontmatter changelog notes referencing old-state language

**Candidate policy extension:** Extend DF-SIBLING-SWEEP-001 (or create a new
sub-rule `DF-RED-BANNER-SWEEP-001`) to enumerate per-test section banners and
doc-comments as explicit sibling targets whenever a module-level implementation
status changes.

**Status:** DEFERRED — needs codification follow-up (next feature cycle or
dedicated housekeeping story).

---

## [process-gap] PG-ARP-F4-PRECLEAR-PROPAGATION

**Source:** STORY-112 Step-4.5 adversarial convergence (AC-004 banner re-escalation)

**Observation:** The orchestrator propagated a prior adversarial pass's
"acceptably-nuanced / leave as-is" pre-clearance into a fix dispatch for the
AC-004 banner. A later fresh-context pass overturned this pre-clearance as HIGH
(the region was still present-tense stale-RED, which the fresh context correctly
identified as a state contradiction).

**Root cause:** Fix dispatches must not inherit pre-clearances from an earlier
adversarial pass. Each fresh adversarial context examines the full perimeter
without inherited verdicts. Pre-clearance from pass N does not bind pass N+1.

**Candidate policy extension:** Add explicit language to DF-ADVERSARY-METHODOLOGY-001
or the per-story adversarial dispatch template: "Fix dispatches MUST NOT carry
forward 'leave as-is' verdicts from a prior adversarial pass. Each fresh
adversarial context is pre-clearance-free."

**Status:** DEFERRED — policy codification follow-up (next feature cycle).

---

## [process-gap] PG-ARP-F4-GUARD-WORDING

**Source:** STORY-112 Step-4.5 adversarial convergence (checkout-guard accuracy)

**Observation (F-4 in adversarial review):** An adversary checkout-guard
premise stated "main repo does NOT have this function" (referring to
`extract_arp_frame`). This was inaccurate: the main-repo `develop` branch at
`cced898` does carry `extract_arp_frame`, introduced by STORY-111's
non-panicking None placeholder. The function BODY, not the function PRESENCE,
is the distinguishing characteristic between STORY-111 stub state and
STORY-112 implementation state.

**Root cause:** The checkout guard keyed on function presence rather than body
content. The correct discriminator is:
- Stub state: `extract_arp_frame` body returns `None` unconditionally (placeholder)
- Implemented state: `extract_arp_frame` body performs hw/proto type + size
  validation and field copy

The transitional error string `"ARP extraction not yet implemented"` (in
`decode_packet`) is another valid body-content discriminator.

**Candidate policy extension:** Extend DF-ADVERSARY-CHECKOUT-GUARD-001 guidance:
"Checkout guards for stub-vs-implementation transitions MUST key on BODY CONTENT
(placeholder behavior vs real logic) rather than function presence. The presence
of a function is not a reliable discriminator after the stub commit."

**Status:** DEFERRED — extend DF-ADVERSARY-CHECKOUT-GUARD-001 (engine agent-prompt
note or policy codification follow-up).

---

## [process-gap] PG-ARP-F4-DEMO-LEAK

**Source:** STORY-112 pre-PR diff inspection (2026-06-15)

**Observation:** The demo-recorder agent committed 4 gif+webm+tape recording sets
(demo binaries) to the develop-bound worktree branch under `.factory-demos/STORY-112/`.
This dodged the `.factory/` worktree ignore rule because `.factory-demos/` is a
different directory name — not covered by the develop branch's `.gitignore` entry.

The leak was caught by a pre-PR diff inspection. Commit 76bdf16 (demo binary commit
on the worktree branch) was dropped, and `.factory-demos/` was added to `.gitignore`
via commit bec7a76, which shipped on develop in PR #238.

**Root cause:** The demo-recorder dispatch template targeted a develop-bound worktree
path (`.factory-demos/`) rather than the factory-artifacts branch (`.factory/demo-evidence/`).
Demo evidence is factory-artifacts-only content.

**Lesson:** Demo evidence belongs ONLY on the factory-artifacts branch under
`.factory/demo-evidence/`. The demo-recorder MUST NOT commit demo artifacts
(gif/webm/tape/binary) to develop-bound worktree branches.

**Candidate fix:**
1. Demo-recorder dispatch template must target `.factory/demo-evidence/` on the
   factory-artifacts worktree, OR commit evidence to a fully gitignored path that
   is never staged to a develop-bound branch.
2. The orchestrator MUST run a pre-PR diff check for binary/demo artifacts
   (gif, webm, tape, mp4, png above a threshold) before dispatching pr-manager.
   Any such artifact detected on a develop-bound branch is a hard STOP.

**Status:** DEFERRED — demo-recorder dispatch template update + orchestrator
pre-PR binary-leak check (candidate for next engine sprint).

---

## [process-gap] PG-ARP-F4-PRMGR-MERGE-SHORTSTOP (RECURRENCE #3)

**Source:** STORY-112 PR #238 delivery (2026-06-15). Third recurrence this feature cycle.

**Observation:** pr-manager again halted at step 6 (APPROVE) without executing steps
7-9 (merge + confirm + consolidated report). Required an orchestrator "merge NOW"
SendMessage to complete. The exact same pattern occurred at STORY-111 (PR #236)
and at the DNP3 F5 cycle.

**Root cause:** pr-manager interprets its mandate as obtaining approval rather than
driving the PR to a merged state. The 9-step protocol is not self-enforcing.

**Escalation note:** Three recurrences in one feature cycle (STORY-111, STORY-112,
and at least one DNP3 F5 PR). This has crossed the threshold for engine-level
escalation. DF-PR-MANAGER-COMPLETE-001 (HIGH) is already filed; this recurrence
should be referenced when that policy is enforced or escalated to CRITICAL.

**Candidate fix:** The pr-manager dispatch template must include an explicit
"DO NOT STOP AT APPROVE — execute steps 7-9 (merge, confirm CI green, consolidated
report) before returning" instruction, and the orchestrator should verify merge
completion before declaring the PR cycle closed.

**Status:** DEFERRED — engine dispatch template hardening; escalate
DF-PR-MANAGER-COMPLETE-001 recurrence count to 3 in policy registry.
