# Evidence Report — STORY-183

**Story:** STORY-183: check-green-doc-tense: bin/*.py Prose Coverage + TIER-1
Behavioral-Absence Token Coverage
**Epic:** E-11 (Tooling and Self-Improvement)
**Wave:** 86
**Date:** 2026-09-05
**Branch:** feature/STORY-183-green-doc-tense-py-surface
**Product type:** Python CLI tooling (`bin/check-green-doc-tense` gate +
`bin/test_check_green_doc_tense.py` self-test runner) — evidence captured as raw
terminal output transcripts (matching the STORY-181/STORY-182 precedent for this
repo's `bin/` tooling stories), not VHS/GIF, since the deliverable is a Python CLI
gate's stdout/exit-code behavior rather than an interactive terminal product.

---

## Full Suite Status (context)

Implementation was reported complete and green prior to this recording session.
This session captures per-acceptance-criterion evidence only; it does not modify
any tool, test, or CI source file. The only files added by this session live under
`docs/demo-evidence/STORY-183/`.

---

## Coverage Map

| AC | Description | Evidence File(s) | Verdict |
|----|-------------|-------------------|---------|
| AC-183-001 | `bin/*.py` glob added to `_collect_source_files()` (renamed from `_collect_rust_files`); scanned-file count includes `.py` files | `AC-183-001-and-008-live-scan-pass.txt`, `AC-183-001-rename-grep.txt` | PASS |
| AC-183-002 | Language-scoped comment detection: `#` is a comment prefix only for `.py` files; Rust `#[attr]` lines are never scan-eligible | `AC-183-002-language-scoped-comment-excerpt.txt` | PASS |
| AC-183-003 | Pattern 30 (`Expected RED:` heading) added, `.rs`/`.py` BAD cases + allowlist GOOD case | `AC-183-003-004-007-selftest-full-run.txt` | PASS |
| AC-183-004 | Pattern 31 (`currently fall(s)`) added, `.rs`/`.py` BAD cases + past-tense/TIER-2 GOOD cases | `AC-183-003-004-007-selftest-full-run.txt` | PASS |
| AC-183-005 | `[Unreleased]` CHANGELOG entry covering PG-W84-010 + PG-W85-003 + STORY-183; `changelog-gate` passes | `AC-183-005-changelog-gate.txt`, `AC-183-005-changelog-excerpt.txt` | PASS |
| AC-183-006 | Positive `.py` coverage proof (RED/flag path): a violating `bin/*.py` file IS flagged (exit 1, Pattern named); the same content in a `.rs` file is NOT flagged (suffix-scoped negative guard) | `AC-183-006-red-path-demo.sh`, `AC-183-006-red-path-output.txt` | PASS |
| AC-183-007 | Patterns 32–37 (six remaining TIER-1 tokens) added, `.rs`/`.py` BAD cases where prescribed, TIER-2 zero-FP GOOD cases | `AC-183-003-004-007-selftest-full-run.txt` | PASS |
| AC-183-008 | Zero-false-positive live-codebase sweep: `python3 bin/check-green-doc-tense` exits 0 across all 130 tracked source files (TIER-1 covered, TIER-2 correctly excluded) | `AC-183-001-and-008-live-scan-pass.txt` | PASS |
| AC-183-009 | `bin/test_lint_cycle_artifact.py` passes locally (21/21); the 3 stale RED-GATE-era phrases scrubbed from that file are now absent | `AC-183-009-lint-cycle-artifact-run.txt`, `AC-183-009-stale-phrase-grep.txt` | PASS |

---

## AC-183-001 Summary (rename + `bin/*.py` glob)

`python3 bin/check-green-doc-tense` prints:

```
PASS: no stale RED-phase comment headers found (130 files scanned).
```

130 files scanned now includes `bin/*.py` (the merged invocation is
`git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py`). The rename from
`_collect_rust_files` to `_collect_source_files` is confirmed by
`git grep -n _collect_source_files bin/check-green-doc-tense`, which shows both
the function definition (line 574) and its call site inside `main()` (line 669);
no `_collect_rust_files` references remain in the tool itself.

## AC-183-002 Summary (language-scoped comment detection)

Self-test excerpt confirms both directions of the suffix-scoping:

- `.py` form BAD cases for Patterns 30/32/33 (`# ...` Python comment lines) ARE
  flagged — e.g. `PASS [Pattern 30: 'Expected RED:' heading violation (.py form —
  Python comment)]`.
- The negative guard — a `#`-prefixed line placed in a `.rs` file — is NOT
  flagged: `PASS [Suffix-scoping negative guard: '# Expected RED:' in .rs file
  NOT flagged (# is not a Rust comment)]`. This is the AC-183-006/F-009 proof that
  `.py` eligibility is suffix-scoped, not global — a Rust `#[attr]` line is never
  scan-eligible.

## AC-183-003 / AC-183-004 / AC-183-007 Summary (Patterns 30–37 full self-test run)

`python3 bin/test_check_green_doc_tense.py` full run result:

```
Results: 125 passed, 0 failed.
```

All 12 new BAD_CASES (Patterns 30–37, `.rs` and `.py` forms where prescribed) are
flagged as expected; all corresponding GOOD_CASES (allowlists, TIER-2 zero-FP
cases, and the `is expected to` efficacy case) pass clean. The full transcript
includes every one of the 125 individual `PASS` lines plus the four F-010
hermetic end-to-end checks (`exit 1 on violation`, `not empty-collection exit`,
`output names violating.py`, `collect finds exactly 1 source file`) and the two
AC-183-001 non-hermetic collector checks.

## AC-183-005 Summary (CHANGELOG obligation)

```
$ git diff origin/develop...HEAD -- CHANGELOG.md | bin/changelog-gate-check
PASS: CHANGELOG.md updated with 12 content line(s).
exit: 0
```

The `[Unreleased]` entry (12 added content lines) documents the `bin/*.py` glob
extension, the language-scoped `#`-comment eligibility, the 8 new TIER-1 patterns
(30–37) with their token list, and the `_collect_rust_files` →
`_collect_source_files` rename — see `AC-183-005-changelog-excerpt.txt` for the
full added-lines excerpt.

## AC-183-006 Summary (RED / flag path — positive `.py` coverage proof)

Beyond the self-test runner's own hermetic e2e checks (see AC-183-003/004/007
summary above), this session additionally exercised the flag path standalone,
outside the self-test harness, to demonstrate the mechanism directly:

`AC-183-006-red-path-demo.sh` builds two throwaway `git init` repos under
`mktemp -d` (never inside the wirerust tree), copies `bin/check-green-doc-tense`
into each, and:

1. Writes `# currently asserts the implementation is complete` to `bin/violating.py`
   in the first throwaway repo, `git add`s it, then runs the gate. Result:
   ```
   FAIL [bin/violating.py:1]: Pattern 32 (PG-W85-003): 'currently asserts' — RED-phase present-tense claim (AC-183-007)
        # currently asserts the implementation is complete

   Found 1 stale RED-phase comment(s) in tracked source files (DF-GREEN-DOC-TENSE-SWEEP).
   ...
   --- exit code: 1 ---
   ```
   Exit code **1**, with **Pattern 32** and the violating file (`bin/violating.py`)
   both named in the output — the flag/RED path.

2. Writes the identical `#`-prefixed line to `src/placeholder.rs` in a second
   throwaway repo and runs the gate again. Result:
   ```
   PASS: no stale RED-phase comment headers found (1 files scanned).
   --- exit code: 0 ---
   ```
   Exit code **0** — confirming the suffix-scoped negative guard: the same text
   is inert in a `.rs` file.

Full transcript: `AC-183-006-red-path-output.txt`. Script: `AC-183-006-red-path-demo.sh`.

**Worktree cleanliness after the RED-path demo:** `git status --porcelain` in the
STORY-183 worktree, run immediately after the demo, reported no new or modified
tracked files and no stray demo artifacts — only the pre-existing untracked
`bin/__pycache__/` build byproduct (present before this recording session began)
remained. The throwaway repos were created under `mktemp -d` outside the wirerust
tree and removed via `trap ... EXIT` / explicit `rm -rf` at the end of the script;
no violating `.py` file was ever written into, or committed to, this repository.

## AC-183-008 Summary (zero-false-positive live sweep)

```
$ python3 bin/check-green-doc-tense
PASS: no stale RED-phase comment headers found (130 files scanned).
exit: 0
```

All 8 new TIER-1 patterns (30–37) are live-swept across the full 130-file tracked
source set with zero false positives — the 10 live `falls through to` sites and
all `is expected to` / `No wildcard arm` / `not yet implemented` / `currently
fails` TIER-2 sites remain unflagged, consistent with the self-test's TIER-2
GOOD_CASEs (see AC-183-003/004/007 summary).

## AC-183-009 Summary (lint-cycle-artifact self-test + stale-phrase scrub)

```
Results: 21 passed, 0 failed
All tests passed.
```

The 4-line scrub (module docstring line 3, line 5–6, and line 125 in
`bin/test_lint_cycle_artifact.py`) removed the three stale RED-GATE-era phrases
that became newly scan-eligible once this story added `bin/*.py` to the gate's
scan glob (the literal `RED GATE` token, the stale `TC1–TC8` count, and the
present-tense `MUST FAIL until created` claim). Post-scrub grep for those
patterns returns no matches — see `AC-183-009-stale-phrase-grep.txt`.

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

The mandatory gate command (per
`.factory/maintenance/demo-evidence-scrub-gate.md`) was run against all captured
raw output before writing these evidence files, matching for absolute macOS- or
Linux-style home-directory paths and tilde-form home references.

Result: **zero matches** — no absolute host paths or tilde-form home references
were present in any captured `python3`, `git`, or `grep` output. The one file
that legitimately needed a repo-root reference — the RED-path demo script,
`AC-183-006-red-path-demo.sh` — was written to self-locate via
`git rev-parse --show-toplevel` rather than hardcoding an absolute worktree path,
per the same portability discipline VHS tapes use in this repo (worktrees under
`.worktrees/STORY-NNN/` are removed after merge).

The gate was re-run against the final committed `docs/demo-evidence/STORY-183/`
directory after writing all files: zero results (gate PASS). (This sentence
intentionally avoids reproducing the gate's own regex literal, which would
otherwise self-trigger the gate.)
