---
pass: 1
previous_review: null
wave: 73
story: STORY-162
reviewer: vsdd-factory:code-reviewer
date: 2026-07-10
diff: "git diff f1e0c36..b5e1e15"
---

# Wave-73 Gate Code Review

**Scope:** STORY-162 squash commit — `bin/check-green-doc-tense` `_find_repo_root`
extraction + `bin/test_check_green_doc_tense.py` +5 hermetic tests + `CHANGELOG.md`
entry.

**Verdict: APPROVE-WITH-COMMENTS**

The change is sound. The extraction is a clean refactor: `_find_repo_root` is a well-
documented helper with a formal contract (precondition / postcondition / invariant),
and `main()` correctly delegates to it. The five new tests cover the four previously-
untested code paths. Two MINOR/NIT issues are noted below; none block merge.

---

## Part B — Findings

### CR-001: AC-158-005 regression guard is non-hermetic after the refactor
- **Severity:** MINOR
- **Category:** test-robustness (code-quality)
- **Location:** `bin/test_check_green_doc_tense.py:451-452`
- **BC Reference:** AC-158-005
- **Description:** The pre-existing AC-158-005 test patches `_collect_rust_files` to
  return `[]` but does NOT patch `_find_repo_root`. After the STORY-162 refactor,
  `main()` now calls `_find_repo_root(script_path.parent)` first. When the test suite
  is run from a detached directory or a CI sandbox where no `.git` / `.factory/`
  ancestor is within 6 hops of `bin/`, `_find_repo_root` returns `None`, `main()`
  returns `2` (root-not-found guard), and `exit_code != 0` evaluates True — so the
  test passes for the wrong reason, masking a silent regression of the zero-file guard.
  The newly-added AC-162-003 test IS hermetic (it patches both helpers), but
  AC-158-005 still carries this fragility.
- **Evidence:** `mod._collect_rust_files = lambda _repo_root: []` (line 451) with no
  corresponding patch to `mod._find_repo_root`. Test assertion is `exit_code != 0`
  (line 453), which passes for both exit-1 (intended) and exit-2 (root-not-found,
  masking the real guard).
- **Proposed Fix:** Tighten the AC-158-005 assertion from `exit_code != 0` to
  `exit_code == 1`, and add a hermetic `_find_repo_root` patch identical to
  AC-162-003's (patch to return a known temp root so root-discovery cannot fail):
  ```python
  with tempfile.TemporaryDirectory() as _td:
      _hermetic = Path(_td)
      (_hermetic / ".factory").mkdir()
      mod._find_repo_root = lambda _s: _hermetic
      mod._collect_rust_files = lambda _r: []
      exit_code = mod.main()
      # ... restore both; assert exit_code == 1
  ```

---

### CR-002: Docstring says "6 levels" but `range(6)` checks start + 5 ancestors (pre-existing off-by-one in comment)
- **Severity:** NIT
- **Category:** maintainability
- **Location:** `bin/check-green-doc-tense:366`
- **BC Reference:** n/a
- **Description:** The docstring line "Walk upward up to 6 levels from *start*" and the
  inline comment `# at most 6 levels up` are ambiguous. `range(6)` iterates 6 times,
  checking `start`, `start.parent`, …, `start.parent^5` — that is the start directory
  plus 5 levels up (6 candidates total). If "6 levels" means "6 ancestors above start",
  the loop would need `range(7)`. The behavior was inherited from the pre-refactor code
  and is not a new regression, but the docstring makes the contract appear to promise
  more search depth than the implementation delivers.
- **Evidence:**
  ```python
  for _ in range(6):  # at most 6 levels up   ← 6 iterations, not 6 ancestors above start
      if (candidate / ".git").exists() or (candidate / ".factory").is_dir():
          return candidate
      candidate = candidate.parent
  ```
- **Proposed Fix:** Either change the loop to `range(7)` to check 6 true ancestors above
  `start`, or update the docstring to read "Walk upward through at most 6 candidates
  (start inclusive)" so the count matches `range(6)`.

---

### CR-003: No-sentinel test (c) uses string-prefix matching for path containment instead of `Path.is_relative_to()`
- **Severity:** NIT
- **Category:** code-quality
- **Location:** `bin/test_check_green_doc_tense.py:562`
- **BC Reference:** AC-162-004
- **Description:** The condition `not str(_result_c).startswith(str(_root_c))` uses
  raw string prefix matching to check that `_result_c` is not within the temp tree.
  String-prefix matching is not equivalent to filesystem hierarchy containment: a path
  `/tmp/foobar` would be incorrectly treated as starting-with `/tmp/foo`, causing a
  false negative if `_root_c` happened to be a string prefix of an unrelated path.
  `Path.is_relative_to()` (available since Python 3.9; project targets 3.10+) performs
  correct hierarchy-aware containment.
- **Evidence:**
  ```python
  if _result_c is None or not str(_result_c).startswith(str(_root_c)):
  ```
  Compare: `Path("/tmp/foobar").is_relative_to(Path("/tmp/foo"))` → `False` (correct);
  `str("/tmp/foobar").startswith(str("/tmp/foo"))` → `True` (misleading).
- **Proposed Fix:**
  ```python
  if _result_c is None or not _result_c.is_relative_to(_root_c):
  ```

---

## Finding Disposition Table

(For orchestrator ratification — prefixed `proposed:`)

| ID | Severity | Summary | Disposition |
|----|----------|---------|---------------------|
| CR-001 | MINOR | AC-158-005 regression guard non-hermetic after refactor; could pass with exit-2 instead of exit-1 | DEFERRED (human, 2026-07-11, next maintenance sweep) — AC-162-003 provides hermetic coverage already |
| CR-002 | NIT | Docstring "6 levels" ambiguity vs. `range(6)` (pre-existing off-by-one in comment) | DEFERRED (human, 2026-07-11, next maintenance sweep) — fix docstring wording in follow-on cleanup |
| CR-003 | NIT | Test (c) uses `str.startswith` instead of `Path.is_relative_to()` for containment check | DEFERRED (human, 2026-07-11, next maintenance sweep) — fix in follow-on cleanup pass |

---

## Convergence Verdict

`findings remain -- iterate`

No CRITICAL or HIGH findings. Two NITs and one MINOR are logged above. The MINOR (CR-001)
is the only item with any risk surface, and it is mitigated in practice by the hermetic
AC-162-003 test added in the same commit. All three findings are suitable for a follow-on
cleanup pass rather than blocking this gate.
