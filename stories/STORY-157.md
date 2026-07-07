---
document_type: story
story_id: STORY-157
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-07T00:00:00Z
phase: f7
level: feature
cycle: wave-70-story-149
points: 3
priority: P3
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
wave: "~"
traces_to:
  - .factory/policies.yaml
  - .factory/STATE.md
  - bin/compute-input-hash
  - bin/test_compute_input_hash.py
input-hash: d41d8cd
inputs: []
---

# STORY-157: Wave-70 process-gap codifications: adversary attestation preamble, demo-evidence scrub gate, input-hash empty-inputs handling

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 3
**Priority:** P3

## Narrative

- **As a** factory orchestrator and developer on the wirerust project
- **I want** three wave-70 process gaps codified into durable project artifacts (policy
  amendments, CLAUDE.md guidance, and a Python tool fix)
- **So that** adversary checkout-guard omissions are structurally prevented, absolute
  host paths cannot be reintroduced into committed demo-evidence, and
  `bin/compute-input-hash` correctly handles E-11 stories with `inputs: []`

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

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `parse_inputs` empty-inputs fix | `bin/compute-input-hash` | Pure (Python function) |
| Empty-inputs self-test cases | `bin/test_compute_input_hash.py` | Test (pure) |
| PG-S149-001 policy entry | `.factory/policies.yaml` | Configuration artifact |
| Demo-evidence scrub gate | `CLAUDE.md` or factory runbook | Documentation artifact |

## Purity Classification

All deliverables are outside the production Rust codebase:

| File | Classification | Reason |
|------|---------------|--------|
| `bin/compute-input-hash` | Effectful (I/O) | Reads filesystem, writes frontmatter |
| `bin/test_compute_input_hash.py` | Pure (test-only) | In-memory computation only |
| `.factory/policies.yaml` | Configuration artifact | No code, no side effects |
| `CLAUDE.md` | Documentation artifact | No code, no side effects |

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
   MATCH for all E-11 wave-TBD stories (STORY-091, STORY-121, STORY-143, STORY-147,
   STORY-150, STORY-155, STORY-157).
6. **Research-agent validation:** All three gap codifications require research-agent
   validation per DF-VALIDATION-001 before any GitHub issue is filed.

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
| `.factory/policies.yaml` | Modify | Add PG-S149-001 policy or amend DF-ADVERSARY-CHECKOUT-GUARD-001 |
| `CLAUDE.md` | Modify | Add demo-evidence path-scrub gate note |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~4 k |
| `bin/compute-input-hash` (~200 lines) | ~2.5 k |
| `bin/test_compute_input_hash.py` (~150 lines) | ~2 k |
| `.factory/policies.yaml` (relevant sections) | ~2 k |
| `CLAUDE.md` (relevant sections) | ~1 k |
| **Total** | **~11.5 k** |

Well within the 20–30% context-window threshold. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** All three gaps require research-agent validation before
  any GitHub issue is filed. See CLAUDE.md: "Deferred or open findings ... MUST be
  validated by the research agent (`vsdd-factory:research-agent`) before being filed
  as GitHub issues."
- Source process-gaps: PG-S149-001 (STATE.md D-395, wave-70-story-149, 2026-07-07);
  PG-W70-DEMO-SCRUB (F-W70P2-002, wave-70 Phase-2 gate, MEDIUM, fixed PR #376);
  PG-HASH-EMPTY-INPUTS (`bin/compute-input-hash` `_INPUTS_RE` line 99, empty `inputs:`
  not handled).
- S-7.02 disposition: creating this story at draft status codifies all three PG-*
  open items from wave-70 for S-7.02 wave-70 cycle-close purposes.
- No behavioral contract required: E-11 convention (epics.md E-11: "BCs: none
  authored yet — status: draft; pending PO authorship").
- input-hash note: `inputs: []` yields `d41d8cd` (MD5 of empty string, first 7 chars).
  The tool currently errors on this input; AC-157-003 fixes it. The stored `d41d8cd`
  documents the expected correct value; it cannot be tool-verified until AC-157-003
  ships. See scope item PG-HASH-EMPTY-INPUTS.
- Precedent: STORY-147 (PG-MUTANTS-JOBS-001, 2026-07-01) and STORY-155
  (PG-INDEX-DRIFT-001, 2026-07-05) — same E-11 pattern: cycle process-gap follow-up
  encoding lessons into project workflow/tooling/docs.
