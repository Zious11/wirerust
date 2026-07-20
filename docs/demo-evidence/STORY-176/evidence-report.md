# Demo Evidence Report — STORY-176

**Story:** STORY-176 — Feature-IEC104 Cycle-Close: Local Gate + Tooling Hygiene Sweeps
**Wave:** 84
**Branch:** feature/STORY-176-cycle-close-hygiene
**Recorded:** 2026-07-20
**Scrub gate (PG-W70-DEMO-SCRUB):** PASS — zero absolute host paths in evidence files

---

## Coverage Map

| AC | Path(s) Covered | Recording(s) | Result |
|----|-----------------|--------------|--------|
| AC-176-001 (gate phrase patterns) | Success path: `python3 bin/check-green-doc-tense` → PASS (114 files) | AC-176-001-gate-success.{gif,webm,tape} | PASS |
| AC-176-001 (gate phrase patterns) | Self-test: `python3 bin/test_check_green_doc_tense.py` → 91 passed, 0 failed | AC-176-001-gate-selftest.{gif,webm,tape} | PASS |
| AC-176-001 (gate phrase patterns) | Negative path: create temp file with `skeleton compiles` comment → gate FLAGS it (pattern a) → delete temp file → gate returns to PASS | AC-176-001-gate-negative.{gif,webm,tape} | PASS |
| AC-176-002 (re-baseline note) | `grep -n "re-baseline" ../../.factory/maintenance/delivery-doc-currency-protocol.md` → 3 non-empty lines | AC-176-002-rebaseline-note.{gif,webm,tape} | PASS |
| AC-176-003 (.gitignore glob) | `.gitignore` has `mutants.out*/` → `git status` shows NO-UNTRACKED-MUTANTS-DIRS after mkdir/rmdir → `test_gitignore_mutants_glob.py` → 2 passed, 0 failed → CI wiring confirmed | AC-176-003-gitignore-glob.{gif,webm,tape} | PASS |

---

## Artifacts

### AC-176-001 — Green-doc-tense gate phrase-pattern extension

**Success path** — `bin/check-green-doc-tense` exits 0 on the current tree (114 files scanned):
- `AC-176-001-gate-success.tape` — VHS script
- `AC-176-001-gate-success.gif` — recording
- `AC-176-001-gate-success.webm` — recording

**Self-test** — `bin/test_check_green_doc_tense.py` exits 0 (91 passed, 0 failed):
- `AC-176-001-gate-selftest.tape` — VHS script
- `AC-176-001-gate-selftest.gif` — recording
- `AC-176-001-gate-selftest.webm` — recording

**Negative path** — gate detects `skeleton compiles` stub-era pattern, then returns to PASS after cleanup:
- `AC-176-001-gate-negative.tape` — VHS script
- `AC-176-001-gate-negative.gif` — recording
- `AC-176-001-gate-negative.webm` — recording

Negative-path demo sequence:
1. Create `tests/demo_stub_prose_tests.rs` containing `// harness skeleton compiles only -- wiring deferred`
2. `git add` to make git ls-files track it
3. `python3 bin/check-green-doc-tense` exits 1 with `FAIL [tests/demo_stub_prose_tests.rs:1]: skeleton compiles? (stub-era compile-only assertion; AC-176-001 pattern a)`
4. `git restore --staged && rm` to delete and unstage the temp file
5. `git status --porcelain | grep demo_stub` emits nothing (CLEAN)
6. `python3 bin/check-green-doc-tense` exits 0: `PASS: no stale RED-phase comment headers found (114 files scanned).`

### AC-176-002 — Delivery-doc input-hash re-baseline reminder

- `AC-176-002-rebaseline-note.tape` — VHS script
- `AC-176-002-rebaseline-note.gif` — recording
- `AC-176-002-rebaseline-note.webm` — recording

`grep -n "re-baseline" ../../.factory/maintenance/delivery-doc-currency-protocol.md` returns 3 lines:
- Line 178: `**Post-delivery re-baseline step:**`
- Line 192: `**The re-baseline is NOT optional.**`
- Line 200: reference to observed on STORY-164/165 (re-baselined 2026-07-18)

Note: AC-176-002 is a factory-artifacts deliverable (`.factory/maintenance/delivery-doc-currency-protocol.md` lives on the factory-artifacts branch). The grep is run from the worktree using the relative path `../../.factory/maintenance/delivery-doc-currency-protocol.md`.

### AC-176-003 — .gitignore mutants.out* glob

- `AC-176-003-gitignore-glob.tape` — VHS script
- `AC-176-003-gitignore-glob.gif` — recording
- `AC-176-003-gitignore-glob.webm` — recording

Demo sequence:
1. `grep -n "mutants.out" .gitignore` → line 12: `mutants.out*/`
2. `mkdir -p mutants.out mutants.out.j4-invalid` then `git status --porcelain | grep mutants || echo 'NO-UNTRACKED-MUTANTS-DIRS'` → `NO-UNTRACKED-MUTANTS-DIRS` (directories are ignored by .gitignore)
3. `rmdir mutants.out mutants.out.j4-invalid` (cleanup)
4. `python3 bin/test_gitignore_mutants_glob.py` → `Results: 2 passed, 0 failed`
5. `grep -n "test_gitignore_mutants_glob" .github/workflows/ci.yml` → lines 485-486 (bin-selftest job step)

---

## Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Command run before commit (PG-W70-DEMO-SCRUB gate — checks for absolute host paths):
```
grep -rE '<USERDIR>/|<HOMEDIR>/|<TILDE>/' docs/demo-evidence/STORY-176/
# (patterns: user-home, system-home, tilde-home paths)
```
Result: **zero matches** — PASS.

All absolute host paths in the VHS tape hidden-setup sections were replaced with `<repo>/`
after recording completed (the setup `cd` command is hidden from the recorded frames; the
visible terminal output and recorded video contain no host-identifying strings).

---

## Worktree Cleanliness

`git status --porcelain` before commit: `?? bin/__pycache__/` only — `bin/__pycache__/`
is excluded from the commit (not staged). The `tests/demo_stub_prose_tests.rs` temp file
used in the AC-176-001 negative-path demo was deleted and unstaged within the same VHS tape
session (confirmed by `CLEAN: demo_stub_prose_tests.rs removed` in the recording).
