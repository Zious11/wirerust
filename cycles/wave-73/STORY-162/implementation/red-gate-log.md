---
document_type: red-gate-log
level: ops
version: "1.0"
status: verified
producer: implementer
timestamp: 2026-07-10T00:00:00Z
phase: 3
inputs: []
input-hash: "d41d8cd"
traces_to: "STORY-162"
stub_architect_agent: "n/a (Python tool — no stubs required)"
stub_compile_verified: true
test_writer_agent: "orchestrator"
red_gate_verified: true
---

# Red Gate Log: wave-73 STORY-162

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|-------------|-----------------|------|
| STORY-162 | 4 new hermetic main()-guard tests | Yes — 4 fail, 55 pass | VERIFIED RED |

## Stubs Created

### STORY-162: LMR-003 template-conformance exemption + check-green-doc-tense main() guard self-tests

No compiled stubs needed. Story targets a Python tooling script (`bin/check-green-doc-tense`)
and its self-test suite (`bin/test_check_green_doc_tense.py`). The test file was committed at
2aa0617 with 4 failing tests; the corresponding `_find_repo_root` helper was not yet extracted
from `bin/check-green-doc-tense` (stub state: inline code in `main()` not exposed as testable
helper).

Stub commit: `fa40d1c` — `py_compile` clean, 55/55 pre-existing tests pass, stub functionality
not yet exposed (four new tests reference `mod._find_repo_root` which does not yet exist).

Test commit: `2aa0617` — 4 new tests added, all 4 fail (Red Gate verified).

## Red Gate Verification

### STORY-162 — AC-162-003 (zero-file guard exit-code precision)

- `[zero-file guard hermetic: main() must use _find_repo_root for repo-root detection (AC-162-003, F-W72G-P2-OBS-001)]`
  — FAIL (expected) — main() passed the live worktree path to `_collect_rust_files` instead
    of the hermetic temp root; `_find_repo_root` not yet extracted.

### STORY-162 — AC-162-004 (.factory/ OR-sentinel hermetic tests)

- `[_find_repo_root: .factory/ OR-sentinel resolves root (F-W72G-P2-OBS-001)]`
  — FAIL (expected) — `mod._find_repo_root` does not yet exist as a module-level attribute.

- `[_find_repo_root: .git directory sentinel resolves root (F-W72G-P2-OBS-001)]`
  — FAIL (expected) — same root cause.

- `[_find_repo_root: .git file (worktree) sentinel resolves root (F-W72G-P2-OBS-001)]`
  — FAIL (expected) — same root cause.

All four failures are assertion-style failures (expected values vs. None or wrong path);
no crashes or import errors. The pre-existing 55 tests all continue to pass.

## Regression Check

| Existing Tests | Status |
|---------------|--------|
| 55 pre-existing tests (25 BAD + 29 GOOD + 1 AC-158-005) | all pass |

## Hand-Off to Implementer

- Stories ready for implementation: STORY-162
- Implementation guidance:
  1. Extract `_find_repo_root(start: Path) -> Path | None` from `main()` in
     `bin/check-green-doc-tense` (walk upward from `start`, check `.git` file/dir
     or `.factory/` dir, return first match or None).
  2. Wire `main()` to call `_find_repo_root(script_path.parent)` and preserve
     exact exit-code semantics (root not found → exit 2; no files → exit 1).
  3. Verify: `python3 bin/test_check_green_doc_tense.py` → 59 passed, 0 failed.
  4. Commit: `feat(STORY-162): extract _find_repo_root helper and delegate main() root detection`
  5. Add CHANGELOG entry: `docs(STORY-162): add CHANGELOG entry`
  6. Amend VP-INDEX (factory side, uncommitted): bump version 2.39→2.40, add LMR-003
     template-conformance exemption (Option A) with two allowlist rows for `inputs:`
     and `input-hash:`.

Verifier: orchestrator ran suite independently 2026-07-10.

---

## Addendum (2026-07-10, post-Green)

A 5th test — the no-sentinel regression guard for `_find_repo_root` (AC-162-004(c)) — was
added at commit b94da37 after Green (deferred from the RED gate because it was vacuous
against the always-None stub); its assertion was corrected at d519df8 (F-S162P1-002).
Final suite: 60 passed, 0 failed (55 pre-existing + 5 new).
