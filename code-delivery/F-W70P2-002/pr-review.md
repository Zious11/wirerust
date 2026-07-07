# PR #376 Review — docs: scrub absolute host paths from committed demo evidence (F-W70P2-002)

**Verdict:** APPROVE
**Reviewer:** pr-reviewer (Opus 4.7, fresh context, information-asymmetry wall)
**Branch:** `fix/demo-evidence-path-scrub` → `develop`
**Diff stat:** 193 files changed, 196 insertions(+), 196 deletions(-)

## Summary

Mechanical path-scrub across `docs/demo-evidence/**` only. Replaces
`/Users/zious/Documents/GITHUB/wirerust` → `<REPO-ROOT>` and `/Users/zious`
(in `$PATH` exports) → `<HOME>`. No Rust source, no binaries, no CI config,
no test predicates touched.

## Verification Results

| Check | Result | Evidence |
|-------|--------|----------|
| Diff stat matches PR description (193 files, 196 additions, 196 deletions) | PASS | `git diff --stat` confirms 193 files, 196/196 |
| Zero absolute paths remain in head-branch files | PASS | `git ls-tree fix/demo-evidence-path-scrub -- docs/demo-evidence/` + grep returns empty |
| No `Users/zious` on any `+` line (0 matches) | PASS | `grep -c "^\+.*Users/zious"` = 0 |
| 196 `-` lines match 196 `+` lines with `<REPO-ROOT>` / `<HOME>` | PASS | `grep -c` on both sides = 196 |
| No other-form absolute paths (`/home/*`, `/root/*`, `/private/var/*`, `/tmp/[a-z]/…`) introduced | PASS | grep of full diff — no hits |
| No file emptied or truncated | PASS | Minimum line count across changed files = 21; all files retain full content |
| Every `-` line has an exactly-matching `+` line after path normalization (no non-path content altered) | PASS | Awk pair-check after normalizing all path tokens to a common marker — zero unbalanced pairs |
| `.sh` helper (STORY-144/show-tls-result.sh): only `WDIR=` literal substituted | PASS | Manual diff inspection |
| `.txt` transcripts (STORY-149): only cargo compile output paths substituted; timing, assertions, bench identifiers untouched | PASS | Manual diff inspection |

## Findings

### Blocking

None.

### Non-Blocking / Suggestion

1. **suggestion / process-gap** — Follow-up: `.factory` branch (factory-artifacts
   worktree) needs its own path-scrub pass. Already flagged by the author in the
   PR body under Follow-ups. Out of scope for this PR.

2. **suggestion / process-gap** — Follow-up: `demo-recorder` VHS tape template
   needs a path-scrub gate so future recordings don't reintroduce the leak class.
   Already flagged by the author in the PR body under Follow-ups.

### Nit

1. **nit / documentation** — Two STORY-144 files (tapes and `show-tls-result.sh`)
   embed `.factory/demo-evidence/fix-tls-clienthello-frag/…` paths under
   `<REPO-ROOT>`. These still resolve correctly for any consumer substituting
   `<REPO-ROOT>` to their clone root, so no action required. Called out only in
   case the author wants a more explicit `<REPO-ROOT>/.factory/…` note in a
   README for the STORY-144 evidence bundle.

## PR Description Adequacy

Adequate. Description includes:

- Summary
- Origin (F-W70P2-002 wave-70 Phase-2 gate)
- Replacement policy (both tokens)
- File-type inventory (.tape, .txt, .sh)
- Per-directory breakdown matching the 193 files
- Verification grep gate
- Explicit rationale for skipping security review and demo evidence
- Risk assessment
- Rollback path (`git revert` trivially reverts all 196 substitutions)

Semantic-PR title conforms to `docs:` type per repo convention
(`amannn/action-semantic-pull-request` gate).

## Rationale for Approval

This is a mechanical string-substitution across `docs/demo-evidence/**` only.
Pair-balance analysis confirms every `-` line is paired with a `+` line that
differs solely in the path token — no timing, assertion text, command flags,
or bench output was altered. All 196 in-repo occurrences of the leak class
are removed with no residue. No Rust source, no CI configuration, no test
predicates touched, so `cargo test / clippy / fmt` gates are unaffected.

The two acknowledged follow-ups (factory-artifacts branch, demo-recorder
template gate) are correctly scoped out of this PR.

## Recommended Next Step

Merge after CI green and human merge authorization per the PR's own pre-merge
checklist.
