#!/usr/bin/env python3
"""Self-test for AC-176-003: .gitignore mutants.out* glob.

Verifies that .gitignore covers the standard cargo-mutants output directories:
  - mutants.out/           (default cargo mutants run output)
  - mutants.out.j4-invalid/ (cargo mutants invalid-mutant output)

These are NOT covered by the existing mutants-f6*/ glob.

Run: python3 bin/test_gitignore_mutants_glob.py

Red Gate: FAILS until .gitignore is updated with mutants.out*/ (AC-176-003).
"""

import subprocess
import sys
from pathlib import Path


def _find_repo_root() -> Path:
    """Walk upward from this script's location looking for .git or .factory/."""
    candidate = Path(__file__).resolve().parent
    for _ in range(6):
        if (candidate / ".git").exists() or (candidate / ".factory").is_dir():
            return candidate
        candidate = candidate.parent
    raise RuntimeError(
        "Could not locate repo root (no .git or .factory/ sentinel found)"
    )


def check_ignored(repo_root: Path, path: str) -> bool:
    """Return True if `git check-ignore -q <path>` exits 0 (path is ignored)."""
    result = subprocess.run(
        ["git", "check-ignore", "-q", path],
        cwd=repo_root,
        capture_output=True,
    )
    return result.returncode == 0


def run_tests() -> int:
    repo_root = _find_repo_root()

    failures = 0
    passed = 0

    print("=== AC-176-003: .gitignore mutants.out* glob ===")

    # mutants.out/ — default cargo mutants output directory
    path_1 = "mutants.out/"
    if check_ignored(repo_root, path_1):
        print(f"  PASS  [{path_1!r} is ignored by .gitignore]")
        passed += 1
    else:
        print(
            f"  FAIL  [{path_1!r} is NOT ignored by .gitignore] — "
            f"add 'mutants.out*/' glob under cargo-mutants section in .gitignore (AC-176-003)"
        )
        failures += 1

    # mutants.out.j4-invalid/ — cargo mutants output for invalid mutants
    path_2 = "mutants.out.j4-invalid/"
    if check_ignored(repo_root, path_2):
        print(f"  PASS  [{path_2!r} is ignored by .gitignore]")
        passed += 1
    else:
        print(
            f"  FAIL  [{path_2!r} is NOT ignored by .gitignore] — "
            f"add 'mutants.out*/' glob under cargo-mutants section in .gitignore (AC-176-003)"
        )
        failures += 1

    print()
    print(f"Results: {passed} passed, {failures} failed.")
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())
