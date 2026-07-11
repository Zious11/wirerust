#!/usr/bin/env python3
"""
Self-test for AC-164-003: changelog-gate content assertion in .github/workflows/ci.yml.

The changelog-gate CI job currently performs only a presence check (CHANGELOG.md
appears in the diff). AC-164-003 mandates adding a content assertion immediately
after the presence check that verifies at least one non-blank, non-header content
line was added to CHANGELOG.md, preventing a whitespace-only touch from satisfying
the gate.

These tests verify that the mandated content assertion is present in ci.yml.
They FAIL against the current (pre-implementation) ci.yml and will PASS once
AC-164-003 is delivered.

Run: python3 bin/test_changelog_gate_content.py
"""

import sys
from pathlib import Path


# ---------------------------------------------------------------------------
# Locate ci.yml relative to this script (bin/ → repo root → .github/workflows/)
# ---------------------------------------------------------------------------

def find_ci_yml() -> Path:
    """Walk up from bin/ to find .github/workflows/ci.yml."""
    script_dir = Path(__file__).resolve().parent
    for candidate in [script_dir, *script_dir.parents]:
        ci = candidate / ".github" / "workflows" / "ci.yml"
        if ci.exists():
            return ci
    raise FileNotFoundError(
        "Cannot find .github/workflows/ci.yml relative to bin/. "
        "Run from the repo root or a worktree branch."
    )


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

def test_content_lines_variable_present() -> None:
    """
    AC-164-003(a): The CONTENT_LINES variable must be defined in the changelog-gate
    run: block (the bash variable that counts non-blank, non-header added lines).
    FAILS against the current presence-only ci.yml.
    """
    ci_yml = find_ci_yml()
    text = ci_yml.read_text(encoding="utf-8")
    assert "CONTENT_LINES" in text, (
        "AC-164-003 FAIL: 'CONTENT_LINES' not found in .github/workflows/ci.yml.\n"
        "The changelog-gate content assertion (AC-164-003) has not been implemented.\n"
        "Expected: a CONTENT_LINES bash variable counting non-blank, non-header added lines."
    )
    print(f"  [PASS] CONTENT_LINES variable present in ci.yml")


def test_changelog_diff_variable_present() -> None:
    """
    AC-164-003(a): CHANGELOG_DIFF=$(git diff origin/develop...HEAD -- CHANGELOG.md)
    must be present in the changelog-gate run: block.
    FAILS against the current presence-only ci.yml.
    """
    ci_yml = find_ci_yml()
    text = ci_yml.read_text(encoding="utf-8")
    assert "CHANGELOG_DIFF" in text, (
        "AC-164-003 FAIL: 'CHANGELOG_DIFF' not found in .github/workflows/ci.yml.\n"
        "The changelog-gate content assertion (AC-164-003) has not been implemented.\n"
        "Expected: CHANGELOG_DIFF=$(git diff origin/develop...HEAD -- CHANGELOG.md)"
    )
    print(f"  [PASS] CHANGELOG_DIFF variable present in ci.yml")


def test_whitespace_only_message_present() -> None:
    """
    AC-164-003(a): The FAIL message for a whitespace-only touch must be present,
    per the exact wording specified in the AC:
      "whitespace-only touch does not satisfy AC-158-001 / PG-W71-CHANGELOG"
    FAILS against the current presence-only ci.yml.
    """
    ci_yml = find_ci_yml()
    text = ci_yml.read_text(encoding="utf-8")
    assert "whitespace-only" in text, (
        "AC-164-003 FAIL: 'whitespace-only' not found in .github/workflows/ci.yml.\n"
        "The changelog-gate FAIL message (AC-164-003) has not been implemented.\n"
        "Expected the message to include 'whitespace-only touch does not satisfy AC-158-001'."
    )
    print(f"  [PASS] whitespace-only FAIL message present in ci.yml")


def test_content_line_pass_message_present() -> None:
    """
    AC-164-003(a): The PASS message reporting the number of content lines must be
    present: "PASS: CHANGELOG.md updated with ${CONTENT_LINES} content line(s)."
    FAILS against the current presence-only ci.yml.
    """
    ci_yml = find_ci_yml()
    text = ci_yml.read_text(encoding="utf-8")
    assert "content line" in text, (
        "AC-164-003 FAIL: 'content line' not found in .github/workflows/ci.yml.\n"
        "The PASS echo message (AC-164-003) has not been implemented.\n"
        "Expected: echo 'PASS: CHANGELOG.md updated with ${CONTENT_LINES} content line(s).'"
    )
    print(f"  [PASS] content line PASS message present in ci.yml")


def test_grep_filter_chain_present() -> None:
    """
    AC-164-003(a): The grep filter chain that strips blank lines and ## headers from
    the diff must be present. Checks for the section-header filter 'grep -v '^+##''.
    FAILS against the current presence-only ci.yml.
    """
    ci_yml = find_ci_yml()
    text = ci_yml.read_text(encoding="utf-8")
    # The AC specifies: grep -v '^+##'  to exclude section headers
    assert "'^+##'" in text or "'^+##'" in text or "^+##" in text, (
        "AC-164-003 FAIL: section-header filter (^+##) not found in .github/workflows/ci.yml.\n"
        "The grep filter chain (AC-164-003) has not been implemented.\n"
        "Expected: grep -v '^+##' to strip section headers from the content line count."
    )
    print(f"  [PASS] grep section-header filter (^+##) present in ci.yml")


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def main() -> None:
    tests = [
        test_content_lines_variable_present,
        test_changelog_diff_variable_present,
        test_whitespace_only_message_present,
        test_content_line_pass_message_present,
        test_grep_filter_chain_present,
    ]
    passed = 0
    failed = 0
    for t in tests:
        print(f"\n{t.__name__}:")
        try:
            t()
            passed += 1
        except Exception as exc:
            print(f"  [FAIL] {exc}")
            failed += 1

    print(f"\n{'='*60}")
    print(f"Results: {passed} passed, {failed} failed")
    if failed:
        sys.exit(1)
    print("All tests passed.")


if __name__ == "__main__":
    main()
