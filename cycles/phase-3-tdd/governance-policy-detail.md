# Governance Policy Detail — Phase 3 TDD (v0.1.0-greenfield-spec)

## Extracted from STATE.md on 2026-05-29

This file contains the full prose for the `## Governance Policy` section that was
in STATE.md. The canonical machine-readable source is `.factory/policies.yaml`.
This prose was moved here to keep STATE.md under 200 lines (content-routing rule S-7.02).

---

**DF-VALIDATION-001** (commit 9b6efd3, `.factory/policies.yaml`): every
deferred/open finding must be research-agent validated before filing as a
GitHub issue. Pointer in `CLAUDE.md` on `develop` via PR #99 (0082a0c).

**DF-SIBLING-SWEEP-001** (added 2026-05-26, `.factory/policies.yaml`): every
remediation dispatch to story-writer, test-writer, or product-owner MUST
include an explicit sibling-sweep checklist. Orchestrator MUST inject the
checklist under "## Sibling-Sweep Checklist (MANDATORY per DF-SIBLING-SWEEP-001)"
into every dispatch prompt. Derived from W9-D8 (6 consecutive recurrences in
Wave 9). Severity: CRITICAL.

**DF-PR-MANAGER-COMPLETE-001** (added 2026-05-26, `.factory/policies.yaml`):
pr-manager MUST complete steps 7-9 (handle approval, squash merge, post-merge
cleanup) before reporting back to the orchestrator. APPROVE verdict is step 6
of 9 — NOT the stopping point. Orchestrator MUST inject this policy with the
concrete `gh pr merge <#> --squash --admin --delete-branch` command template
under "## PR Completion Policy (MANDATORY per DF-PR-MANAGER-COMPLETE-001)"
into every pr-manager dispatch. Derived from W9.L3 (7 consecutive PRs
#122/123/126/127/128/129/130 across Waves 8-9). Severity: HIGH.

**DF-ADVERSARY-METHODOLOGY-001** (added 2026-05-27, `.factory/policies.yaml`):
All file operations in adversary dispatch prompts MUST use absolute paths.
The `cd <path> && ...` pattern is FORBIDDEN in adversary dispatches because
the cd does not persist across Bash invocations in some agent profiles,
causing the adversary to query the wrong filesystem. Git operations must use
`git -C <absolute-path> ...`. Derived from W11.L2 (pass-5 adversary grepped
main repo instead of worktree; produced 2 FALSE-CRITICAL findings).
Severity: HIGH.

**DF-AC-TEST-NAME-SYNC-001 v2** (extended 2026-05-29, `.factory/policies.yaml`):
v1 (added 2026-05-28): AC "**Test:**" citations must match `fn test_*` definitions.
v2 (added 2026-05-29): Citations MUST also uniquely resolve — a bare test name matching
two functions across module boundaries does NOT satisfy the policy. Module qualifier or
fully-qualified path required when ambiguity exists. Derived from F-W16-WAVE-P1-003
(PG-W16-001 codification). Severity: MEDIUM.

**DF-CONVERGENCE-BEFORE-MERGE-001** (added 2026-05-29, `.factory/policies.yaml`):
Per-story delivery flow MUST NOT merge a story PR before Step-4.5 adversarial
convergence gate is ACHIEVED (3 consecutive clean passes per BC-5.39.001). Derived
from PG-W16-005 / W16.L1 (4 stories merged without convergence; STATE.md left stale).
Severity: CRITICAL.

**DF-DEVELOP-FRESHNESS-001** (added 2026-05-29, `.factory/policies.yaml`):
Orchestrator MUST `git pull origin develop --ff-only` before every adversarial
dispatch (per-story and wave-level). Derived from W1.1 + W8.1 (stale develop
produced 2 FALSE-CRITICAL findings in wave-level pass-3). Severity: HIGH.

**DF-ADVERSARY-TOOLCHAIN-PAIRING-001** (added 2026-05-29, `.factory/policies.yaml`):
Adversary dispatches that require build/test verification MUST be paired with a
toolchain-runner sub-agent. Adversary read-only profile cannot execute `cargo test`,
`cargo clippy`, or `compute-input-hash --check`. Orchestrator must run toolchain
checks before dispatching adversary. Derived from W11-D3/D4/D5 (F-W11P4-010/011).
Severity: HIGH.

**DF-SIBLING-SWEEP-001 v4** (extended 2026-05-29, `.factory/policies.yaml`):
New bullet group added: when a BC source-evidence anchor changes, sweep ALL consuming
STORY Architecture-Mapping bodies that cite the same anchor — not only sibling BCs and
test files. Derived from PG-W16-003 / W16.L3 / F-W16-S044-P3-001.
Severity: CRITICAL (inherits from v1).

**DF-SIBLING-SWEEP-001 v3** (extended 2026-05-27, `.factory/policies.yaml`):
Two new bullet groups added to the existing policy:
(1) EC-scenario-match sub-rule (W12.L1, F-W12P6-001): when an EC row includes a
`covered by <test>` citation, the cited test MUST exercise the SPECIFIC scenario
described (specific port, byte value, length boundary) — not just the parent BC
capability. Adversaries flag mismatches as MEDIUM mis-anchor.
(2) Single-burst-all-BCs rule (W12.L2): when anchor-completeness applies to ONE
BC in a story, orchestrator MUST sweep ALL BCs in the story `bcs:` frontmatter in
the SAME burst. Derived from Wave 12 iterative cascade (passes 3/4/6 each caught
the same gap in a different BC). Severity: CRITICAL (inherits from v1).
