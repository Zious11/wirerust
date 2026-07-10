# Demo Evidence Report — STORY-157

**Story:** Wave-70 process-gap codifications: adversary attestation preamble,
demo-evidence scrub gate, input-hash empty-inputs handling, merge-authorization
procedure  
**Story ID:** STORY-157  
**Branch:** feature/STORY-157-process-gap-codifications  
**HEAD at evidence generation:** 70d99ad  
**Recorded:** 2026-07-08  
**Tool:** VHS 0.11.0 / Font: Menlo  

---

## Coverage Summary

| AC | Description | Scope | Demo Artifact | Status |
|----|-------------|-------|--------------|--------|
| AC-157-001 | Adversary attestation preamble policy (DF-ADVERSARY-CHECKOUT-GUARD-002) | Factory-half | `.factory/policies.yaml` citation | Cited below |
| AC-157-002 | Demo-evidence path-scrub gate in CLAUDE.md | Factory-half (+ scrub gate run) | Scrub gate live run (dogfooding) | Cited below |
| AC-157-003 | `inputs: []` inline compact → `d41d8cd` | Develop-tree | `AC-157-003-005-self-test.*`, `AC-157-003-004-live-demo.*` | Recorded |
| AC-157-004 | Empty multiline inputs block → `d41d8cd` | Develop-tree | `AC-157-003-005-self-test.*`, `AC-157-003-004-live-demo.*` | Recorded |
| AC-157-005 | Self-test coverage: empty-inputs test cases | Develop-tree | `AC-157-003-005-self-test.*` | Recorded |
| AC-157-006 | `--scan` completes MATCH=110 STALE=0 | Develop-tree | `AC-157-006-scan-gate.*` | Recorded |
| AC-157-007 | Merge-authorization policy (DF-MERGE-AUTH-CLASSIFIER-001) | Factory-half | `.factory/policies.yaml` citation | Cited below |
| AC-157-008 | PR-manager step-8 guidance update | Factory-half | `.factory/maintenance/pr-manager-merge-auth-guidance.md` citation | Cited below |
| AC-157-009 | CLAUDE.md documents PG-HASH-HOOK-DIVERGENCE | Develop-tree | `AC-157-009-hook-divergence.*` | Recorded |
| AC-157-010 | Inline comment stripping in inputs parser (success + error path) | Develop-tree | `AC-157-010-inline-comment-success.*`, `AC-157-010-error-path-baseline.*` | Recorded |

---

## Recorded Demos (Develop-Tree ACs)

### AC-157-003, AC-157-004, AC-157-005 — Empty Inputs Handling

**Self-test suite (9/9 pass, incl. AC-157-003/004/005 regression-guard cases):**

| File | Size |
|------|------|
| `AC-157-003-005-self-test.gif` | 129 KB |
| `AC-157-003-005-self-test.webm` | 137 KB |
| `AC-157-003-005-self-test.tape` | VHS script |

`python3 bin/test_compute_input_hash.py` runs 9 tests and reports
`9 passed, 0 failed — All tests passed.`  
Tests 7 (`test_empty_inputs_inline_compact`) and 8 (`test_empty_inputs_multiline_block`)
are the regression guards for AC-157-003 and AC-157-004 respectively.
Test 9 (`test_inline_comment_stripped_from_path`) guards AC-157-010.

**Live demo (both empty-inputs variants produce `d41d8cd`):**

| File | Size |
|------|------|
| `AC-157-003-004-live-demo.gif` | 76 KB |
| `AC-157-003-004-live-demo.webm` | 77 KB |
| `AC-157-003-004-live-demo.tape` | VHS script |

Fixture: `/tmp/story157-demo/story-empty-inline.md` (`inputs: []`)
and `/tmp/story157-demo/story-empty-block.md` (empty multiline block).
Both emit `d41d8cd` — MD5 of empty bytes, confirming the short-circuit
in `compute_hash` is reached. Error path (pre-fix SystemExit on both forms)
is documented in the test's `AssertionError` message in the self-test tape.

---

### AC-157-006 — Scan Gate: MATCH=110 STALE=0

| File | Size |
|------|------|
| `AC-157-006-scan-gate.gif` | 131 KB |
| `AC-157-006-scan-gate.webm` | 133 KB |
| `AC-157-006-scan-gate.tape` | VHS script |

`python3 bin/compute-input-hash --scan 2>&1 | tail -25` run from the
STORY-157 worktree. Tool auto-resolves repo root (`.factory/` lookup from
script parent directories). Tail captures the last 25 lines including the
summary line `MATCH=110 STALE=0`. This confirms all E-11 stories with
`inputs: []` and `input-hash: d41d8cd` (STORY-091, 121, 143, 147, 148, 149,
150, 155) now report MATCH instead of ERROR.

---

### AC-157-009 — Known Tool Divergences in CLAUDE.md

| File | Size |
|------|------|
| `AC-157-009-hook-divergence.gif` | 195 KB |
| `AC-157-009-hook-divergence.webm` | 209 KB |
| `AC-157-009-hook-divergence.tape` | VHS script |

`grep -A 28 'Known Tool Divergences' CLAUDE.md` captures the full
`### Known Tool Divergences (PG-HASH-HOOK-DIVERGENCE)` section
(CLAUDE.md lines 176–202), demonstrating all three required elements:
(a) names `bin/compute-input-hash` (Python) as the canonical algorithm;
(b) notes the plugin hook's divergent bash computation via `$(cat file)`;
(c) states hook errors are advisory-only with concrete evidence:
STORY-156: Python=`ce96d86`, hook=`7b7dc6b`.

---

### AC-157-010 — Inline Comment Stripping (Success + Error Path)

**Success path (fixed tool):**

| File | Size |
|------|------|
| `AC-157-010-inline-comment-success.gif` | 113 KB |
| `AC-157-010-inline-comment-success.webm` | 125 KB |
| `AC-157-010-inline-comment-success.tape` | VHS script |

Fixture: `/tmp/story157-demo/story-inline-comment.md` with inputs entry
`  - spec.md  # RETIRED 2026-06-19: superseded`. Fixed tool emits `0e0bbc2`,
which matches the hash for the clean path `spec.md` without comment.
Both hashes are shown side-by-side to confirm identity.

**Error path (develop-baseline, before fix):**

| File | Size |
|------|------|
| `AC-157-010-error-path-baseline.gif` | 104 KB |
| `AC-157-010-error-path-baseline.webm` | 113 KB |
| `AC-157-010-error-path-baseline.tape` | VHS script |

Baseline tool extracted via `git show origin/develop:bin/compute-input-hash`
to `/tmp/input-hash-baseline.py`. Running it on the same fixture fails with:
```
Input file missing: /private/tmp/story157-demo/spec.md  # RETIRED 2026-06-19: superseded
  Referenced by: /private/tmp/story157-demo/story-inline-comment.md
  Relative path: spec.md  # RETIRED 2026-06-19: superseded
```
The comment text is treated as part of the file path, causing `SystemExit`.

---

## Factory-Half AC Citations (not demoed — artifact references)

### AC-157-001 — DF-ADVERSARY-CHECKOUT-GUARD-002 Policy Entry

**Artifact:** `.factory/policies.yaml`  
**Entry:** `DF-ADVERSARY-CHECKOUT-GUARD-002`  
**Scope:** Adversary dispatch two-phase structural enforcement; VOID instruction
for attestation-lacking reports. Added per AC-157-001 in the factory-artifacts
branch.

### AC-157-002 — Demo-Evidence Path-Scrub Gate (dogfooding)

**Artifact:** `CLAUDE.md` (demo-evidence path-scrub gate subsection) +
`.factory/maintenance/demo-evidence-scrub-gate.md`  
**Dogfooding:** This evidence set was scrubbed per AC-157-002's own gate before
committing. Gate result: the path-scrub grep (absolute-path pattern) against
`docs/demo-evidence/` returned zero matches.

PASS — zero absolute host paths in any committed evidence file.

### AC-157-007 — DF-MERGE-AUTH-CLASSIFIER-001 Policy Entry

**Artifact:** `.factory/policies.yaml`  
**Entry:** `DF-MERGE-AUTH-CLASSIFIER-001`  
**Scope:** Merge-authorization classifier: conditions for wave-level clause (b)
sufficiency vs. conditions requiring fresh per-PR human authorization. Added per
AC-157-007 in the factory-artifacts branch.

### AC-157-008 — PR-Manager Step-8 Guidance Update

**Artifact:** `.factory/maintenance/pr-manager-merge-auth-guidance.md`  
**Scope:** Step-8 guidance updated to reference `DF-MERGE-AUTH-CLASSIFIER-001`
by ID and summarize the authorization classifier. Clarifies that when step-8
encounters a condition requiring fresh per-PR authorization, it must surface
this to the human rather than defaulting to autonomous merge.

---

## Red-Gate Evidence Citation

**Log:** `.factory/cycles/wave-71/STORY-157/implementation/red-gate-log.md`  
**Commit:** fb500d3 (red gate baseline at commit 021990e; 3 fail-as-expected tests)  
**Summary:** 3 new tests (AC-157-003, 004, 010) failed as expected at red gate;
6 baseline tests passed. Red gate verdict: PASSED.

No source code or tests were re-broken to produce this evidence report.
Implementation commits (green gate) are reflected in the test tape (9/9 pass).

---

## Path-Scrub Gate (PG-W70-DEMO-SCRUB — AC-157-002 dogfooding)

This story codified the scrub gate (AC-157-002). Evidence was scrubbed per its
own rule before committing. The gate command (absolute-path grep pattern against
`docs/demo-evidence/`) was run from the worktree root. Result: **zero matches**
— scrub gate PASS.

---

*Evidence generated by vsdd-factory:demo-recorder. All .tape files are the VHS
source scripts; all .gif/.webm files are the VHS-generated recordings.*
