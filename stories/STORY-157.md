---
document_type: story
story_id: STORY-157
epic_id: E-11
version: "1.7"
status: draft
producer: story-writer
timestamp: 2026-07-07T00:00:00Z
phase: f7
level: feature
cycle: wave-70-story-149
points: 5
priority: P3
# v1.7 (2026-07-08): Pass-2 remediation F-157-P2-001/002/003 — hash chain note rewritten (no "current" claims), DF-PR-MANAGER-COMPLETE-001 converted to last-extended/version/derived-from convention (v2), CLASSIFIER-001 CLEAN note simplified; re-hash 7d287cc→4ca0ad4.
# v1.6 (2026-07-08): Input re-hash 9f2eb1e→7d287cc after policies.yaml amendments for Pass-1 remediation F-157-P1-002/003 (DF-PR-MANAGER-COMPLETE-001 HALT terminal state + DF-MERGE-AUTH-CLASSIFIER-001 cross-ref).
# v1.5 (2026-07-08): Pass-1 remediation F-157-P1-001 — Notes input-hash stale/hallucinated value corrected: v1.2 canonical 357bca5, v1.4 re-hash 9f2eb1e explicitly named.
# v1.4 (2026-07-08): Input re-hash after policies.yaml amendments (DF-ADVERSARY-CHECKOUT-GUARD-002 + DF-MERGE-AUTH-CLASSIFIER-001 added per AC-157-001/007 implementation); no scope impact.
# v1.3 (2026-07-07): Sibling fix — body Wave header synced to frontmatter wave 71 (wave-TBD drift class, scheduling-burst propagation gap).
# v1.2 (2026-07-07): input-hash gate (wave-71 planning): declared spec inputs (policies.yaml,
#   wave-70 gate-summary); canonical hash 357bca5 computed. Folded PG-HASH-HOOK-DIVERGENCE and
#   PG-HASH-INLINE-COMMENT into PG-HASH codification scope: AC-157-009/010 added.
# v1.1 (2026-07-07): Human decision (pipeline resume) — fold wave-70 retrospective open
#   question into scope: PG-W70-MERGE-AUTH merge-authorization procedure codification
#   added as fourth item. Points raised 3→5; reasoning: merge-authorization requires a
#   substantive policy classifier design (decision boundary for clause (b) vs. per-PR
#   human authorization), not merely a checklist entry; adds ~2 pts of specification
#   and validation work over the original three documentation/tooling items.
depends_on: []
blocks: []
# BC status: pending PO authorship
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: bin/compute-input-hash
subsystems: []
estimated_days: 1
wave: "71"
traces_to:
  - .factory/policies.yaml
  - .factory/STATE.md
  - bin/compute-input-hash
  - bin/test_compute_input_hash.py
input-hash: "4ca0ad4"
inputs:
  - .factory/policies.yaml
  - .factory/cycles/wave-70-story-149/wave-gate/gate-summary.md
---

# STORY-157: Wave-70 process-gap codifications: adversary attestation preamble, demo-evidence scrub gate, input-hash empty-inputs handling, merge-authorization procedure

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** 71
**Points:** 5
**Priority:** P3

## Narrative

- **As a** factory orchestrator and developer on the wirerust project
- **I want** four wave-70 process gaps and two additional tool process-gaps (discovered
  during wave-71 input-hash drift resolution) codified into durable project artifacts
  (policy amendments, CLAUDE.md guidance, Python tool fixes, and a merge-authorization
  procedure)
- **So that** adversary checkout-guard omissions are structurally prevented, absolute
  host paths cannot be reintroduced into committed demo-evidence,
  `bin/compute-input-hash` correctly handles E-11 stories with `inputs: []`, and
  the pr-manager merge-authorization procedure under DF-PR-MANAGER-COMPLETE-001
  clause (b) is clearly defined for future orchestrator execution

## Behavioral Contracts

_(none — E-11 convention: no BCs authored yet; status: draft, pending PO authorship)_

## Background

Wave-70 (STORY-149 delivery, cycle wave-70-story-149, 2026-07-07) surfaced three
process gaps recorded in STATE.md (D-395) and the wave-70 adversarial convergence
state. S-7.02 (cycle-close requirement) mandates codification of recurring process
gaps as follow-up stories rather than hand-fixing each cycle.

### PG-S149-001 — Adversary attestation preamble omission

During the 8-pass adversarial convergence for STORY-149, the adversary agent omitted
the mandatory checkout-guard/freshness attestation block from its initial reply in 4
of the 8 per-story passes, requiring retroactive orchestrator requests each time.
DF-ADVERSARY-CHECKOUT-GUARD-001 (policies.yaml) requires the attestation to appear as
a hard preamble before any findings. The enforcement clause states: "A findings report
that lacks the checkout-guard attestation MUST be treated as methodology-incomplete."
Root cause: the dispatch template does not structurally block findings output until
attestation is confirmed; the mandate is prose-only.

### PG-W70-DEMO-SCRUB — Demo-evidence absolute host path leakage

The demo-recorder for STORY-149 committed evidence transcripts containing absolute
host paths (`/Users/zious/...`). Wave-70 Phase-2 gate finding F-W70P2-002 (MEDIUM,
privacy/process-gap) identified the scope as repo-wide: 196 substitutions across 193
files in 31 directories, remediated in PR #376 (`docs: scrub absolute host paths from
committed demo evidence`). The demo-recording checklist had no path-scrub step, and
no CI grep guard existed to prevent reintroduction.

### PG-HASH-EMPTY-INPUTS — bin/compute-input-hash fails on `inputs: []`

E-11 stories use `inputs: []` because they have no source spec files. The canonical
hash for an empty inputs list is `d41d8cd` (first 7 hex chars of MD5 of an empty
byte string). However, `bin/compute-input-hash` raises `SystemExit` ("No 'inputs:'
block found" or "'inputs:' block is empty") on both the inline compact form
`inputs: []` and on an empty multiline block. Root cause: `_INPUTS_RE` (line 99)
only matches non-empty item lists; `parse_inputs` has no short-circuit for the empty
case. This means `--scan` fails on E-11 stories instead of reporting MATCH.

### PG-HASH-HOOK-DIVERGENCE — validate-input-hash hook vs. canonical algorithm divergence

The plugin's `validate-input-hash` hook uses a bash implementation of the hash algorithm
that computes MD5 via `$(cat file)` concatenation. The bash `$()` subshell strips all
trailing newlines from each file's content before concatenation, producing a different
hash than the canonical Python tool (`bin/compute-input-hash`), which reads raw bytes
including trailing newlines. This causes canonical Python hashes to be flagged as
"drift" by the hook on every story edit.

Evidence (wave-71 input-hash drift resolution, 2026-07-07):
- STORY-156: Python=`ce96d86`, hook=`7b7dc6b`
- STORY-150: Python=`c5acbe4`, hook=`26416e1`
- STORY-157: Python=`357bca5`, hook=`4a47ab6`

Root cause: the plugin tool at `{PLUGIN_ROOT}/bin/compute-input-hash` uses
`CONCAT="${CONCAT}$(cat "$RESOLVED")"` — the `$()` subshell strips trailing newlines
from every file. The canonical Python tool reads raw bytes with no stripping.

Resolution required: `CLAUDE.md` must document that `input-hash:` values are set using
the Python canonical tool only; the hook's divergent compute is a known false-positive
that must be treated as advisory-only until the plugin is reconciled to the canonical
algorithm.

### PG-HASH-INLINE-COMMENT — inline comment corruption in inputs parser

When an `inputs:` list entry contains an inline `# comment` suffix (e.g.,
`.factory/specs/behavioral-contracts/ss-01/BC-2.01.004.md  # RETIRED 2026-06-19: ...`),
the canonical Python tool's `parse_inputs` function includes the comment text as part of
the file path, causing a file-not-found error instead of resolving to the actual file.

Evidence: `bin/compute-input-hash --scan` reports:
`ERROR: Input file missing: ...BC-2.01.004.md  # RETIRED 2026-06-19: superseded by
BC-2.01.009 (behavioral inversion); file retained per append-only-numbering policy`

Root cause: `_INPUTS_RE` (line ~99) captures the full list-item text including inline
comments. `parse_inputs` does not strip ` # …` suffixes before passing paths to file
resolution.

Fix: Strip everything from ` #` (space-hash) onward when parsing each input path.
Self-test coverage required in `bin/test_compute_input_hash.py`.

### PG-W70-MERGE-AUTH — Merge-authorization procedure under DF-PR-MANAGER-COMPLETE-001 clause (b)

During the wave-70 retrospective (2026-07-07 pipeline resume), the orchestrator's
execution of wave-70 PR merges was reviewed. The orchestrator acted under
DF-PR-MANAGER-COMPLETE-001 clause (b), which authorizes orchestrator-executed
merges when CI is green and the human has pre-authorized the wave. However, the
exact decision boundary for when clause (b) applies — versus when each individual
PR merge requires fresh per-PR human authorization — was never formally codified.
This left the authorization scope open to retrospective interpretation: was each
wave-70 merge individually authorized, or was wave-level pre-authorization
sufficient under clause (b)?

The human decision (2026-07-07) resolved this open question: the wave-70 merges
were correctly executed under clause (b), and the procedure must be codified to
prevent future ambiguity. The codification must define: (a) the conditions under
which clause (b) wave-level authorization is sufficient (wave pre-authorized by
human, CI green, no blocking review findings); (b) the conditions that require
fresh per-PR human authorization even within a pre-authorized wave (unexpected CI
failure, blocking findings discovered post-review, scope change surfaced during
delivery); and (c) how pr-manager step-8 should communicate authorization status
to the orchestrator. Root cause: pr-manager step-8 guidance describes the merge
action but does not encode the authorization decision classifier, leaving orchestrator
judgment as the only gate.

## Acceptance Criteria

### AC-157-001 (traces to PG-S149-001 — dispatch template structural enforcement)
A policy amendment or new policy entry in `.factory/policies.yaml` encodes the
PG-S149-001 gap: adversary dispatch MUST structurally ensure the checkout-guard
attestation block precedes any findings output. The enforcement clause must include
an explicit orchestrator VOID instruction for non-compliant reports (reports lacking
Phase-A attestation output are voided and re-dispatched, not triaged).

### AC-157-002 (traces to PG-W70-DEMO-SCRUB — demo-recording path-scrub gate)
`CLAUDE.md` or a factory runbook artifact contains a "Demo-evidence path-scrub gate"
step (within the demo-recording checklist or as a dedicated subsection) that:
(a) requires verifying zero occurrences of `/Users/` and `/home/` in all
    demo-evidence files before pushing,
(b) optionally specifies a CI guard command (`grep -rE '/Users/|/home/'
    docs/demo-evidence/`) that fails if non-zero results are found,
(c) references PG-W70-DEMO-SCRUB, F-W70P2-002, and PR #376.

### AC-157-003 (traces to PG-HASH-EMPTY-INPUTS — inline compact form)
`bin/compute-input-hash` handles `inputs: []` (inline compact YAML form) without
raising `SystemExit` — it emits `d41d8cd` as the hash and exits 0.

### AC-157-004 (traces to PG-HASH-EMPTY-INPUTS — empty multiline block)
`bin/compute-input-hash` handles an empty multiline `inputs:` block (no list items)
without raising `SystemExit` — it emits `d41d8cd` as the hash and exits 0.

### AC-157-005 (traces to PG-HASH-EMPTY-INPUTS — self-test coverage)
`bin/test_compute_input_hash.py` includes at least one test verifying a story with
`inputs: []` produces `d41d8cd`, and at least one test verifying the empty multiline
form produces the same hash.

### AC-157-006 (traces to PG-HASH-EMPTY-INPUTS — scan gate)
After the fix, `bin/compute-input-hash --scan` completes without error on the
`.factory/stories/` directory and reports MATCH (not error) for all STORY-NNN files
with `inputs: []` and `input-hash: d41d8cd`.

### AC-157-007 (traces to PG-W70-MERGE-AUTH — merge-authorization policy entry)
A policy entry in `.factory/policies.yaml` (new policy or amendment to
DF-PR-MANAGER-COMPLETE-001) codifies the PG-W70-MERGE-AUTH classifier: explicitly
defines the conditions under which orchestrator may execute PR merges under clause
(b) without per-PR human authorization (wave pre-authorized by human, CI green,
no blocking review findings), and the conditions requiring fresh per-PR human
authorization (unexpected CI failure, blocking findings discovered, scope change
surfaced during delivery). The policy entry must be mechanically applicable — the
orchestrator must be able to evaluate each condition without human interpretation.

### AC-157-008 (traces to PG-W70-MERGE-AUTH — pr-manager step-8 guidance update)
The pr-manager step-8 guidance artifact (factory runbook or orchestrator sequence
document) is updated to reference the PG-W70-MERGE-AUTH policy entry by ID and
summarize the merge-authorization decision classifier, so future orchestrator runs
apply the procedure without requiring retrospective clarification. The update must
clarify the expected communication pattern: when step-8 encounters a condition
requiring fresh per-PR authorization, it must surface this to the human explicitly
rather than defaulting to autonomous merge.

### AC-157-009 (traces to PG-HASH-HOOK-DIVERGENCE — CLAUDE.md documentation)
`CLAUDE.md` documents the PG-HASH-HOOK-DIVERGENCE discrepancy with three required elements:
(a) names `bin/compute-input-hash` (Python, repo root) as the canonical algorithm for all
`input-hash:` values; (b) explicitly notes that the plugin's `validate-input-hash` hook
uses a divergent bash computation (trailing-newline stripping via `$(cat file)`) and will
report false-positive drift against canonical hashes; (c) states that `input-hash:` values
MUST be set using the canonical Python tool only, and hook validation errors citing a
divergent hash must be treated as advisory-only until the plugin is reconciled to the
canonical algorithm. The documentation must include at least one concrete evidence example
(e.g., STORY-156 Python=`ce96d86` hook=`7b7dc6b`).

### AC-157-010 (traces to PG-HASH-INLINE-COMMENT — inline comment stripping in inputs parser)
`bin/compute-input-hash` strips inline `# comment` suffixes from `inputs:` list entries
before file resolution. Specifically: each input path entry is stripped of everything from
` #` (space followed by `#`) onward before the path is resolved against the repo root.
After the fix, `bin/compute-input-hash --scan` no longer reports ERROR for STORY-001's
retired-BC entry; instead it resolves the stripped path and reports MATCH or STALE based
on the actual file content. `bin/test_compute_input_hash.py` includes at least one test
verifying that a path with an inline comment is resolved correctly (the comment is stripped,
the base path is used for file lookup).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `parse_inputs` empty-inputs fix | `bin/compute-input-hash` | Pure (Python function) |
| Empty-inputs self-test cases | `bin/test_compute_input_hash.py` | Test (pure) |
| PG-S149-001 policy entry | `.factory/policies.yaml` | Configuration artifact |
| Demo-evidence scrub gate | `CLAUDE.md` or factory runbook | Documentation artifact |
| PG-W70-MERGE-AUTH policy entry | `.factory/policies.yaml` | Configuration artifact |
| PR-manager step-8 guidance update | Factory runbook / orchestrator sequence doc | Documentation artifact |
| PG-HASH-HOOK-DIVERGENCE documentation | `CLAUDE.md` | Documentation artifact |
| Inline-comment stripping fix | `bin/compute-input-hash` | Effectful (I/O) |
| Inline-comment self-test cases | `bin/test_compute_input_hash.py` | Pure (test-only) |

## Purity Classification

All deliverables are outside the production Rust codebase:

| File | Classification | Reason |
|------|---------------|--------|
| `bin/compute-input-hash` | Effectful (I/O) | Reads filesystem, writes frontmatter |
| `bin/test_compute_input_hash.py` | Pure (test-only) | In-memory computation only |
| `.factory/policies.yaml` | Configuration artifact | No code, no side effects |
| `CLAUDE.md` | Documentation artifact | No code, no side effects |
| Factory runbook / orchestrator sequence doc | Documentation artifact | No code, no side effects |

No production Rust modules are modified. No pure-core / effectful-shell boundary
analysis is required for this story. The `tdd_mode: strict` default applies to the
Python tool fix — the self-test cases serve as the Red Gate.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `inputs: []` (inline compact YAML) | `bin/compute-input-hash` emits `d41d8cd`, exit 0 |
| EC-002 | `inputs:` with empty multiline block (no items) | `bin/compute-input-hash` emits `d41d8cd`, exit 0 |
| EC-003 | `inputs:` field entirely absent | Existing `SystemExit` "No 'inputs:' block found" retained (no change) |
| EC-004 | Non-empty `inputs:` list | Existing algorithm unchanged; only the empty case is short-circuited |
| EC-005 | `bin/compute-input-hash --scan` on a directory with mix of `inputs: []` and non-empty inputs stories | MATCH reported for `d41d8cd` stories; hash computed normally for others; no errors |
| EC-006 | `validate-input-hash` hook reports drift against a canonical Python hash | Treat as advisory-only; canonical Python tool value is authoritative; document per AC-157-009 |
| EC-007 | `inputs:` list entry with inline `# comment` suffix (e.g., `BC-2.01.004.md  # RETIRED ...`) | Comment stripped; base path resolved; MATCH or STALE reported normally |

## Tasks

1. **Policy amendment (AC-157-001):** Read DF-ADVERSARY-CHECKOUT-GUARD-001 enforcement
   clause; draft amendment or new policy `DF-ADVERSARY-CHECKOUT-GUARD-002` encoding
   two-phase dispatch and VOID instruction for attestation-lacking reports. Add to
   `.factory/policies.yaml` (via `/vsdd-factory:policy-add` at implementation time).
2. **Demo-evidence path-scrub gate (AC-157-002):** Add a "Demo-evidence path-scrub
   gate" section to `CLAUDE.md` (within the demo-recording section or as a standalone
   subsection). Include the grep command and references to PG-W70-DEMO-SCRUB, F-W70P2-002,
   PR #376.
3. **Fix `parse_inputs` for `inputs: []` (AC-157-003/004):** In `bin/compute-input-hash`,
   modify `parse_inputs` to detect the inline compact form `inputs: []` and the empty
   multiline block; return `[]` in both cases. In `compute_hash`, short-circuit to
   `hashlib.md5(b"").hexdigest()[:7]` when `input_paths` is empty.
4. **Add self-test cases (AC-157-005):** In `bin/test_compute_input_hash.py`, add two
   tests: one for inline `inputs: []`, one for empty multiline block; both assert
   `d41d8cd`.
5. **Verify scan gate (AC-157-006):** Run `bin/compute-input-hash --scan` and confirm
   MATCH for all seven E-11 stories in scope: STORY-091, STORY-121, STORY-143, STORY-147,
   STORY-155 (wave-TBD) and STORY-150, STORY-157 (wave 71).
6. **Research-agent validation:** All gap codifications require research-agent
   validation per DF-VALIDATION-001 before any GitHub issue is filed.
7. **Merge-authorization procedure (AC-157-007/008):** Read DF-PR-MANAGER-COMPLETE-001
   in `.factory/policies.yaml`; draft PG-W70-MERGE-AUTH policy entry (or amendment)
   encoding the authorization classifier: conditions for clause (b) sufficiency
   (wave pre-authorized, CI green, no blocking findings) vs. conditions requiring
   fresh per-PR human authorization (CI failure, blocking findings, scope change).
   Update pr-manager step-8 guidance artifact to reference the policy by ID and
   summarize the classifier. Add via `/vsdd-factory:policy-add` at implementation time.
8. **Hook divergence documentation (AC-157-009):** Add a note to `CLAUDE.md` (in the
   Input Hash Computation section or a new "Known Tool Divergences" subsection) documenting
   PG-HASH-HOOK-DIVERGENCE: canonical algorithm is `bin/compute-input-hash` (Python);
   plugin hook diverges due to trailing-newline stripping; hook validation errors against
   canonical hashes are advisory-only. Include one concrete evidence example.
9. **Inline comment fix (AC-157-010):** In `bin/compute-input-hash`, modify `parse_inputs`
   to strip inline `# comment` suffixes from each path entry (strip everything from ` #`
   onward before file resolution). Add a self-test in `bin/test_compute_input_hash.py`
   verifying that an inline comment suffix is ignored and the base path is used.

## Previous Story Intelligence

N/A — first story in E-11 covering these three specific process gaps.

Lessons from closest analogues:
- **STORY-147 (PG-MUTANTS-JOBS-001, 3 pts):** Config + documentation deliverable;
  ≤ 15 total lines. Kept scope tight: one `mutants.toml` entry, one CLAUDE.md note.
- **STORY-155 (PG-INDEX-DRIFT-001, 3 pts):** Workflow change + optional policy/docs.
  Root-cause identification, standing obligation, idempotency verification.

## Architecture Compliance Rules

- This story modifies ONLY: `bin/compute-input-hash`, `bin/test_compute_input_hash.py`,
  `.factory/policies.yaml`, and `CLAUDE.md`. No production Rust is touched.
- The empty-inputs fix MUST NOT change behavior for stories with non-empty `inputs:`.
- The short-circuit value MUST be computed as `hashlib.md5(b"").hexdigest()[:7]`
  (not a hard-coded string), so the derivation is self-documenting.
- `bin/compute-input-hash` forbids third-party dependencies — Python stdlib only.

## Library & Framework Requirements

- Python 3 standard library (`hashlib`, `re`, `pathlib`) — no version change required.
- No new Rust dependencies.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `bin/compute-input-hash` | Modify | Fix `parse_inputs` + `compute_hash` for empty inputs |
| `bin/test_compute_input_hash.py` | Modify | Add 2 empty-inputs test cases |
| `.factory/policies.yaml` | Modify | Add PG-S149-001 policy or amend DF-ADVERSARY-CHECKOUT-GUARD-001; add PG-W70-MERGE-AUTH merge-authorization classifier (new policy or amendment to DF-PR-MANAGER-COMPLETE-001) |
| `CLAUDE.md` | Modify | Add demo-evidence path-scrub gate note |
| pr-manager step-8 guidance (factory runbook or orchestrator sequence doc) | Modify | Update step-8 to reference PG-W70-MERGE-AUTH by ID and summarize the authorization classifier |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~5 k |
| `bin/compute-input-hash` (~200 lines) | ~2.5 k |
| `bin/test_compute_input_hash.py` (~150 lines) | ~2 k |
| `.factory/policies.yaml` (relevant sections) | ~2.5 k |
| `CLAUDE.md` (relevant sections) | ~1 k |
| pr-manager step-8 guidance artifact | ~1 k |
| **Total** | **~14 k** |

Well within the 20–30% context-window threshold. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** All four gaps require research-agent validation before
  any GitHub issue is filed. See CLAUDE.md: "Deferred or open findings ... MUST be
  validated by the research agent (`vsdd-factory:research-agent`) before being filed
  as GitHub issues."
- Source process-gaps: PG-S149-001 (STATE.md D-395, wave-70-story-149, 2026-07-07);
  PG-W70-DEMO-SCRUB (F-W70P2-002, wave-70 Phase-2 gate, MEDIUM, fixed PR #376);
  PG-HASH-EMPTY-INPUTS (`bin/compute-input-hash` `_INPUTS_RE` line 99, empty `inputs:`
  not handled); PG-W70-MERGE-AUTH (2026-07-07 pipeline resume retrospective, wave-70
  orchestrator merges executed under DF-PR-MANAGER-COMPLETE-001 clause (b) — boundary
  conditions for per-PR vs. wave-level authorization were never codified);
  PG-HASH-HOOK-DIVERGENCE (2026-07-07 wave-71 input-hash drift resolution: plugin bash
  hook uses `$(cat file)` stripping trailing newlines — diverges from canonical Python
  tool; STORY-156/150/157 evidence); PG-HASH-INLINE-COMMENT (2026-07-07 wave-71
  drift resolution: STORY-001 `inputs:` inline `# RETIRED` comment corrupts path
  resolution in `parse_inputs`).
- S-7.02 disposition: creating/amending this story at draft status codifies four wave-70
  PG-* open items plus two wave-71 tool process-gaps for S-7.02 wave-70/71 cycle-close
  purposes.
- No behavioral contract required: E-11 convention (epics.md E-11: "BCs: none
  authored yet — status: draft; pending PO authorship").
- input-hash note: v1.2 declares real spec inputs (policies.yaml, wave-70 gate-summary);
  canonical Python hash chain: `357bca5` (v1.2 initial declared-inputs hash) → `9f2eb1e`
  (v1.4 re-hash after AC-157-001/007 policy additions) → `7d287cc` (v1.6 re-hash after
  Pass-1 remediation F-157-P1-002/003 policy amendments). The frontmatter input-hash field
  is always the authoritative current value. The plugin bash hook computed a divergent value
  (`4a47ab6`) against the v1.2 hash due to PG-HASH-HOOK-DIVERGENCE (trailing-newline
  stripping); this is the documented false-positive that AC-157-009 addresses. The original
  `d41d8cd` placeholder (MD5 of empty string) was stored at v1.0/v1.1 when `inputs: []`
  was used; AC-157-003 will fix the scanner to handle `inputs: []` without error for other
  E-11 stories that legitimately have no spec inputs.
- Precedent: STORY-147 (PG-MUTANTS-JOBS-001, 2026-07-01) and STORY-155
  (PG-INDEX-DRIFT-001, 2026-07-05) — same E-11 pattern: cycle process-gap follow-up
  encoding lessons into project workflow/tooling/docs.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.7 | 2026-07-08 | state-manager | Pass-2 remediation F-157-P2-001/002/003 — Notes hash chain rewritten as version-of-origin chain (no "current" claims); DF-PR-MANAGER-COMPLETE-001 converted to last-extended/version/derived-from amendment convention (v2); CLASSIFIER-001 CLEAN note simplified to "CLEAN is stricter than NITPICK_ONLY; both are in the allowed set"; re-hash 7d287cc→4ca0ad4 after policies.yaml amendments. |
| 1.6 | 2026-07-08 | state-manager | Input re-hash 9f2eb1e→7d287cc after policies.yaml amendments for Pass-1 remediation F-157-P1-002/003 (DF-PR-MANAGER-COMPLETE-001 HALT terminal state + DF-MERGE-AUTH-CLASSIFIER-001 CLEAN/NITPICK_ONLY + cross-ref); no scope impact. |
| 1.5 | 2026-07-08 | state-manager | Pass-1 remediation F-157-P1-001 — Notes input-hash hallucinated value corrected to v1.2 canonical `357bca5` and v1.4 re-hash `9f2eb1e` explicitly named; no scope impact. |
| 1.4 | 2026-07-08 | state-manager | Input re-hash after policies.yaml amendments (DF-ADVERSARY-CHECKOUT-GUARD-002 + DF-MERGE-AUTH-CLASSIFIER-001 added per AC-157-001/007 implementation); no scope impact. |
| 1.3 | 2026-07-07 | state-manager | Sibling fix — body Wave header synced to frontmatter wave 71 (wave-TBD drift class, scheduling-burst propagation gap); corrected Task step 5 stale E-11 story list: split into wave-TBD (STORY-091/121/143/147/155) and wave-71 (STORY-150/157). |
| 1.2 | 2026-07-07 | story-writer | Input-hash gate (wave-71 planning) — declared spec inputs (policies.yaml, wave-70 gate-summary); canonical hash 357bca5 computed. Folded PG-HASH-HOOK-DIVERGENCE and PG-HASH-INLINE-COMMENT into PG-HASH codification scope: AC-157-009/010 added. |
| 1.1 | 2026-07-07 | story-writer | Human decision (pipeline resume) — fold wave-70 retrospective open question into scope: PG-W70-MERGE-AUTH merge-authorization procedure codification added as fourth item. Points raised 3→5. |
| 1.0 | 2026-07-07 | story-writer | Initial authorship — wave-70 process-gap codifications: adversary attestation preamble, demo-evidence scrub gate, input-hash empty-inputs handling. |
