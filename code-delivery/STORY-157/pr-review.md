# PR Review: STORY-157 — input-hash empty-inputs + inline-comment + hook-divergence docs

**Verdict:** APPROVE

**Reviewer:** pr-reviewer (Opus 4.7, fresh context)
**PR:** #380
**Feature branch:** `feature/STORY-157-process-gap-codifications` @ 12bb7f4
**Target:** `develop`
**Story:** STORY-157 v1.7, wave 71, E-11, 5 pts
**Reviewed:** 2026-07-08

---

## Summary

Verified all 6 develop-tree ACs (003/004/005/006/009/010) against the diff.
Tests pass 9/9 locally, `--scan` reports MATCH=110 STALE=0 as promised —
the first fully-clean canonical hash scan in project history.

No blocking findings. Recommend merge once CI is green.

---

## AC Coverage Verification

| AC | Deliverable | Verification |
|----|-------------|--------------|
| AC-157-003 | `inputs: []` → `d41d8cd`, exit 0 | `_INPUTS_INLINE_EMPTY_RE = r"^inputs:\s*\[\s*\]\s*$"` short-circuits `parse_inputs`; `compute_hash` returns `hashlib.md5(b"").hexdigest()[:7]` (derived, not hardcoded). test 7 (`test_empty_inputs_inline_compact`) passes. |
| AC-157-004 | Empty multiline block → `d41d8cd`, exit 0 | Existing `_INPUTS_RE`'s `(...)*` quantifier already matches `inputs:\n` alone with zero item lines; the fix removes the `if not paths: raise SystemExit(...)` guard so `[]` is returned and the empty short-circuit fires. test 8 (`test_empty_inputs_multiline_block`) passes. |
| AC-157-005 | ≥1 test per empty-inputs variant | Tests 7 and 8 both explicitly catch `SystemExit` and re-raise as `AssertionError` with AC attribution — proper regression-guard framing. |
| AC-157-006 | `--scan` MATCH=110 STALE=0, no ERRORs | Verified locally at HEAD 12bb7f4: `MATCH=110 STALE=0`, no error entries. |
| AC-157-009 | CLAUDE.md `PG-HASH-HOOK-DIVERGENCE` | All three required elements present at CLAUDE.md:174-201: (a) canonical algorithm named (`bin/compute-input-hash`), (b) divergence noted (bash `$(cat)` strips trailing newlines), (c) advisory-only rule with concrete STORY-156 Python=`ce96d86` hook=`7b7dc6b` evidence. Also documents STORY-150 (`c5acbe4`/`26416e1`) and STORY-157 (`357bca5`/`4a47ab6`) divergence pairs. |
| AC-157-010 | Strip inline `# comment` suffixes, test 9 | `path.find(" #")` + slice + strip in `parse_inputs`; guarded by empty-path check (`if path:`) so a comment-only line doesn't produce a phantom entry. Test 9 (`test_inline_comment_stripped_from_path`) verifies commented path hashes identically to clean path. |

Factory-half ACs (001/002/007/008) are on `factory-artifacts@8271307` and correctly excluded from this diff.

---

## Diff Coherence & Discipline

- All 22 files scoped to STORY-157 (once diffed against `origin/develop` rather than stale local `develop`). No unrelated Rust changes leaked in — the STORY-150 tls/drain-loop and STORY-156 ARP files that appeared in `develop..feature` diffs were already merged to `origin/develop` via PRs #378/#379.
- 8 commits, all conventional-format with `(STORY-157)` scope. Red-gate baseline `a31bba7` precedes green-gate commits `5daf63a`/`3cccb46`/`ba358fd`, matching the PR description.
- CLAUDE.md Python 3.10+ floor call-out (F-157-P4-OBS-003) added to both the tool's docstring and the CLAUDE.md canonical section — consistent.
- Demo evidence: 19 artifacts + `evidence-report.md` present under `docs/demo-evidence/STORY-157/`, both success and error paths recorded (AC-157-010 has both `-error-path-baseline` and `-inline-comment-success`).
- Diff size: ~500 lines total, well within the 500-line flag threshold for pure implementation (the extra volume is demo evidence, which is reasonable).

---

## Code Correctness

- Empty-inputs short-circuit lives in `compute_hash`, not scattered — single choke point, easy to audit.
- `hashlib.md5(b"").hexdigest()[:7]` derivation is self-documenting; no magic constant `d41d8cd` in the source.
- Comment-stripping uses ` #` (space-hash) which is the YAML inline-comment convention — does not affect literal `#` in a path (e.g., no false-stripping of `foo#bar.md`).
- The `if path:` guard after strip prevents an entry consisting only of a comment (`  - # RETIRED`) from adding an empty path to the list.
- No new dependencies. Pure stdlib.

---

## Test Quality

- Both new empty-inputs tests use `try/except SystemExit → AssertionError` — this is textbook regression-guard framing: if a future refactor reintroduces the SystemExit, the test fails with a clear AC-attributed message instead of a confusing test-runner error.
- Test 9 hashes both a clean and a commented variant of the same story and asserts equality — the correct property (idempotence under comment stripping) rather than a brittle pinned hash.
- All 9 self-tests pass; 164/164 cargo tests pass (unchanged, expected — no Rust surface touched).

---

## Observations (non-blocking)

- **SEC-001 (LOW, accepted upstream):** Comment stripping enables `  - ../../etc/shadow  # RETIRED` to resolve outside repo. Documented as accepted LOW because the tool is internal, requires factory-artifacts write access, returns only a 7-char hash, and has no CI privilege-escalation path. Follow-up mitigation (validate `..`/absolute paths post-strip) is appropriate for a maintenance sweep, not this PR.
- **NITPICK:** `_INPUTS_INLINE_EMPTY_RE` doesn't match `inputs: [] # trailing comment` (extra content after `]`). Not in the AC set and unusual in practice.
- **NITPICK:** `path.find(" #")` treats the first ` #` as the comment start; a path containing literal ` #` (e.g., `foo #hashtag/bar.md`) would be mis-truncated. Standard YAML inline-comment convention; no such paths exist in the repo.

---

## Verified Gates

- Adversarial convergence: CONVERGED 3/3 streak (passes 4-6 CLEAN per BC-5.39.001)
- Security review: APPROVE (0 CRITICAL, 0 HIGH; 2 LOW accepted with rationale)
- Self-tests: 9/9 pass (verified locally on this reviewer's machine)
- `--scan`: MATCH=110 STALE=0 (verified locally on this reviewer's machine)
- Demo evidence: 19 artifacts + `evidence-report.md` present, both success and error paths recorded
- `depends_on: []` — no upstream PRs to wait for
- Path-scrub gate: zero `/Users/` or `/home/` in demo evidence per PR description

---

## Finding Table

| # | Severity | Category | Finding | Suggestion |
|---|----------|----------|---------|------------|
| — | — | — | No blocking, high, or medium findings. | — |
| N1 | NITPICK | correctness | `_INPUTS_INLINE_EMPTY_RE` won't match `inputs: [] # comment`. | Optionally extend regex to allow trailing comment; not required by any AC. |
| N2 | NITPICK | correctness | Comment strip splits on first ` #`, mis-truncating paths that legitimately contain ` #`. | YAML inline-comment convention is standard; leave as-is unless such paths appear. |
| L1 | LOW (accepted) | security | SEC-001: comment strip enables path-traversal-in-hash-only path. | Follow-up maintenance task; already documented and accepted. |

**Verdict: APPROVE** — merge once CI is green.
