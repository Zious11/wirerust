---
story_id: STORY-164
pr: "397"
reviewer: vsdd-factory:security-reviewer
date: 2026-07-11
verdict: PASS_WITH_FINDINGS
github_review_id: "4678796876"
---

# Security Review — PR #397 (STORY-164, wave-74)

**Reviewer:** vsdd-factory:security-reviewer
**Date:** 2026-07-11
**Files reviewed:** `bin/validate-citations`, `bin/changelog-gate-check`, `.github/workflows/ci.yml` diff
**Focus areas:** CWE-22 path traversal (parity with #392), command injection, unquoted expansions, supply-chain (SHA-pin policy)

---

## Summary

No CRITICAL or HIGH findings. The CWE-22 containment in `validate-citations` is correctly implemented using `.resolve()` + `.is_relative_to()`, which is the idiomatic Python 3.9+ approach and handles both absolute-path injection and `../` traversal (including via symlinks). The `changelog-gate-check` bash script is safe: all variable expansions are double-quoted, `set -euo pipefail` is present, and the grep chain is correctly guarded with `|| true`. The ci.yml change adds no new `uses:` actions, so the SHA-pin policy is unaffected.

Two LOW findings and two INFO findings are documented below.

---

## Findings Table

| ID | Severity | CWE | File / Location | Summary | Disposition |
|----|----------|-----|-----------------|---------|-------------|
| SEC-001 | LOW | CWE-610 | `bin/validate-citations` lines 74–76 | `WIRERUST_REPO_ROOT=/` env override bypasses `is_relative_to()` containment — every path becomes valid | Non-blocking; design intentional for test use; document trust boundary |
| SEC-002 | LOW | CWE-367 | `bin/validate-citations` lines 176–198 | TOCTOU between `is_file()` check and `count_lines()` open — symlink swap window | Non-blocking; very low exploitability for a local dev tool |
| SEC-003 | INFO | CWE-22 | `bin/compute-input-hash` | Existing path traversal gap in `compute-input-hash` not fixed here; deferred to GitHub #392 | Accepted / tracked in #392 |
| SEC-004 | INFO | CWE-829 | `.github/workflows/ci.yml:509` | `bin/changelog-gate-check` is CI-executable mutable via any PR (same risk accepted for all bin/ scripts) | Accepted / consistent with existing pattern |

---

## Detailed Findings

### SEC-001: WIRERUST_REPO_ROOT override bypasses path-containment guard

**Severity:** LOW — CWE-610 (Externally Controlled Reference to a Resource in Another Sphere)

`bin/validate-citations` lines 74–76 accept `WIRERUST_REPO_ROOT` as-is. Setting `WIRERUST_REPO_ROOT=/` makes `resolved_path.is_relative_to(resolved_root)` trivially True for every path, nullifying the CWE-22 containment advertised in the docstring.

Attack vector requires control of the process environment. Information disclosed is "file has N lines" (no content), so actual impact is low. Design is intentional (enables test isolation without a real `.factory/`).

Proposed mitigation (non-blocking): add a comment noting callers who set `WIRERUST_REPO_ROOT` accept responsibility for path containment.

### SEC-002: TOCTOU between is_file() and count_lines() open

**Severity:** LOW — CWE-367 (Time-of-Check Time-of-Use Race Condition)

`is_file()` check at line 182 and `open("rb")` inside `count_lines()` at line 128 are not atomic. A symlink could be swapped in the window. Exploitability is very low (requires local filesystem write access + precise race timing against a millisecond-range tool).

Proposed mitigation (non-blocking): open the file first and let `OSError` handle both existence and accessibility atomically. Low priority for a local dev CI tool.

### SEC-003: CWE-22 parity — compute-input-hash deferred (INFO)

**Severity:** INFO — CWE-22 (Path Traversal)

`bin/compute-input-hash` has no `is_relative_to()` containment. CHANGELOG correctly notes this as partial parity. Tracked in GitHub #392 as deferred risk. Not a regression from this PR.

### SEC-004: bin/changelog-gate-check mutable via PRs (INFO)

**Severity:** INFO — CWE-829 (Inclusion of Functionality from Untrusted Control Sphere)

ci.yml invokes `bin/changelog-gate-check` as a bare path. A PR modifying that script would have the modified version execute in CI. This is the standard risk for all bin/ scripts in the repo; the posture is consistent.

---

## Positive Findings

- CWE-22 containment via `.resolve()` + `.is_relative_to()` is idiomatic and handles absolute paths, `../` traversal, and symlink chasing correctly.
- Non-UTF-8 input is explicitly hardened on both stdin and file-argument paths.
- Bash script `|| true` pipefail guard is the correct fix; all variable expansions are double-quoted.
- ci.yml change adds no new `uses:` actions; SHA-pin policy unaffected.

---

## Verdict

**PASS** — No CRITICAL or HIGH findings. All LOW and INFO findings are either accepted-by-design or deferred to the existing #392 tracking issue. PR is safe to merge from a security perspective.
