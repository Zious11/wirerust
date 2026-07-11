# AC-162-004 — .factory/ OR-Sentinel Hermetic Tests

**Story:** STORY-162  
**AC:** AC-162-004 (.factory/ OR-sentinel for repo-root detection, hermetically tested)  
**Source file:** `bin/test_check_green_doc_tense.py`  
**Finding:** F-W72G-P2-OBS-001

---

## Test Suite Execution

Command:
```
python3 bin/test_check_green_doc_tense.py
```

### AC-162-004 Section Output (four PASS lines)

```
=== AC-162-004 _find_repo_root sentinel hermetic tests (F-W72G-P2-OBS-001) ===
  PASS  [_find_repo_root: .factory/ OR-sentinel resolves root (F-W72G-P2-OBS-001)]
  PASS  [_find_repo_root: .git directory sentinel resolves root (F-W72G-P2-OBS-001)]
  PASS  [_find_repo_root: .git file (worktree) sentinel resolves root (F-W72G-P2-OBS-001)]
  PASS  [_find_repo_root: no-sentinel temp tree returns None or ancestor (F-W72G-P2-OBS-001)]
```

### Full Test Summary

```
Results: 60 passed, 0 failed.
```

---

## Test Design Notes

The AC-162-004 tests cover the `_find_repo_root` helper that was extracted from
`bin/check-green-doc-tense` (AC-162-004(c) option 1 — helper extraction). Four test cases:

1. **`.factory/` OR-sentinel** — creates a `tempfile.TemporaryDirectory()` with only a
   `.factory/` subdirectory (no `.git`). Asserts `_find_repo_root(start)` returns the
   temp root. This is the primary AC-162-004 target: verifying the `.factory/` arm works
   independently of `.git`.

2. **`.git` directory sentinel** — creates a temp tree with a `.git/` directory. Asserts
   `_find_repo_root(start)` returns the temp root via the `.git` dir arm.

3. **`.git` file (worktree) sentinel** — creates a temp tree with a `.git` file (as
   created by `git worktree add`). Asserts `_find_repo_root(start)` returns the temp root
   via the `.git` file arm.

4. **No-sentinel tree returns None or ancestor** — creates a temp tree with no `.git` or
   `.factory/` at any level. Asserts `_find_repo_root(start)` returns `None` (or the
   filesystem ancestor — the test guards the no-sentinel contract).

All four tests are labeled with `F-W72G-P2-OBS-001` per AC-162-004 requirements. All use
`tempfile.TemporaryDirectory()` and do not rely on the live `.factory/` or `.git` of the
develop checkout (CI-safe).

---

## Result

| AC | Criterion | Verdict |
|----|-----------|---------|
| AC-162-004 | `.factory/` OR-sentinel tested hermetcially; `.git` dir/file arms also covered; no-sentinel guard verified; 4 PASS lines labeled F-W72G-P2-OBS-001 | PASS |
