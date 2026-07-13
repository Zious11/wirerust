# AC-162-003 — Zero-File Guard Exit-Code Precision

**Story:** STORY-162  
**AC:** AC-162-003 (zero-file guard exit code must be exactly 1, not merely non-zero)  
**Source file:** `bin/test_check_green_doc_tense.py`  
**Finding:** F-W72G-P2-OBS-001

---

## Test Suite Execution

Command:
```
python3 bin/test_check_green_doc_tense.py
```

### AC-162-003 Section Output

```
=== AC-162-003 zero-file guard exit-code precision hermetic (F-W72G-P2-OBS-001) ===
  PASS  [zero-file guard hermetic: main() used _find_repo_root result and exited 1 exactly (AC-162-003, F-W72G-P2-OBS-001)]
```

### Full Test Summary

```
Results: 60 passed, 0 failed.
```

---

## Success Path Demo — Tool Runs Normally (Exit 0)

The tool is run from a temporary directory. Because the tool resolves repo root via
its own script path (`Path(__file__).resolve()`), it walks upward from the script
location in the worktree and finds the repo root. The tool scans the worktree's
tracked Rust files and exits 0 (clean pass):

Command:
```
cd "$(mktemp -d)" && python3 <repo>/.worktrees/STORY-162/bin/check-green-doc-tense; echo "exit=$?"
```

Output:
```
PASS: no stale RED-phase comment headers found (110 files scanned).
exit=0
```

Result: exit=0 — success path confirmed.

---

## Hermetic Test Design Notes

The AC-162-003 test in `bin/test_check_green_doc_tense.py`:
- Uses `tempfile.TemporaryDirectory()` to create a controlled directory tree with a
  `.factory/` sentinel so `_find_repo_root` reliably returns a non-None value.
- Monkey-patches `_collect_rust_files` on the importlib-loaded module to return `[]`
  (empty list), forcing the zero-file guard to fire.
- Calls `mod.main()` directly and asserts `exit_code == 1` (exact value).
- This distinguishes exit 1 (zero-file guard, lines 370-376 pre-refactor) from exit 2
  (repo-root-not-found), which the prior AC-158-005 test (`exit_code != 0`) did not.

---

## Result

| AC | Criterion | Verdict |
|----|-----------|---------|
| AC-162-003 | New hermetic test asserts `exit_code == 1` exactly; labeled F-W72G-P2-OBS-001; 60/60 pass | PASS |
