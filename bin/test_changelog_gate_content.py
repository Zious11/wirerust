#!/usr/bin/env python3
"""
Self-test for AC-164-003: changelog-gate content assertion
(bin/changelog-gate-check, invoked from .github/workflows/ci.yml).

The changelog-gate CI job validates CHANGELOG.md additions via
bin/changelog-gate-check. The script counts non-blank, non-section-header
added lines in the diff; a whitespace-only touch does not satisfy the gate.

String-presence tests (T01–T05) verify that the key bash constructs are
present in bin/changelog-gate-check. Behavioral tests (B01–B05) execute the
gate logic against crafted diff fixtures, verify correct exit codes and
output messages, and guard the exec bit.

Run: python3 bin/test_changelog_gate_content.py
"""

import subprocess
import sys
import textwrap
from pathlib import Path


# ---------------------------------------------------------------------------
# Locate bin/changelog-gate-check relative to this script
# ---------------------------------------------------------------------------

def find_changelog_gate_check() -> Path:
    """Walk up from bin/ to find bin/changelog-gate-check."""
    script_dir = Path(__file__).resolve().parent
    check = script_dir / "changelog-gate-check"
    if check.exists():
        return check
    raise FileNotFoundError(
        "Cannot find bin/changelog-gate-check relative to bin/. "
        "Run from the repo root or a worktree branch."
    )


# ---------------------------------------------------------------------------
# String-presence tests (T01–T05): verify key bash constructs exist in
# bin/changelog-gate-check (AC-164-003).
# ---------------------------------------------------------------------------

def test_content_lines_variable_present() -> None:
    """
    AC-164-003(a): CONTENT_LINES must be defined in bin/changelog-gate-check
    (the bash variable that counts non-blank, non-header added lines).
    """
    check = find_changelog_gate_check()
    text = check.read_text(encoding="utf-8")
    assert "CONTENT_LINES" in text, (
        "AC-164-003 FAIL: 'CONTENT_LINES' not found in bin/changelog-gate-check.\n"
        "Expected: a CONTENT_LINES bash variable counting non-blank, non-header added lines."
    )
    print(f"  [PASS] CONTENT_LINES variable present in changelog-gate-check")


def test_changelog_diff_variable_present() -> None:
    """
    AC-164-003(a): CHANGELOG_DIFF must be present in bin/changelog-gate-check.
    """
    check = find_changelog_gate_check()
    text = check.read_text(encoding="utf-8")
    assert "CHANGELOG_DIFF" in text, (
        "AC-164-003 FAIL: 'CHANGELOG_DIFF' not found in bin/changelog-gate-check.\n"
        "Expected: CHANGELOG_DIFF=$(cat) or equivalent."
    )
    print(f"  [PASS] CHANGELOG_DIFF variable present in changelog-gate-check")


def test_whitespace_only_message_present() -> None:
    """
    AC-164-003(a): The FAIL message for a whitespace-only touch must be present,
    per the exact wording: "whitespace-only touch does not satisfy AC-158-001 / PG-W71-CHANGELOG"
    """
    check = find_changelog_gate_check()
    text = check.read_text(encoding="utf-8")
    assert "whitespace-only" in text, (
        "AC-164-003 FAIL: 'whitespace-only' not found in bin/changelog-gate-check.\n"
        "Expected the message to include 'whitespace-only touch does not satisfy AC-158-001'."
    )
    print(f"  [PASS] whitespace-only FAIL message present in changelog-gate-check")


def test_content_line_pass_message_present() -> None:
    """
    AC-164-003(a): The PASS message reporting the number of content lines must be
    present: "PASS: CHANGELOG.md updated with ${CONTENT_LINES} content line(s)."
    """
    check = find_changelog_gate_check()
    text = check.read_text(encoding="utf-8")
    assert "content line" in text, (
        "AC-164-003 FAIL: 'content line' not found in bin/changelog-gate-check.\n"
        "Expected: echo 'PASS: CHANGELOG.md updated with ${CONTENT_LINES} content line(s).'"
    )
    print(f"  [PASS] content line PASS message present in changelog-gate-check")


def test_grep_filter_chain_present() -> None:
    """
    AC-164-003(a): The grep filter chain that strips blank lines and ## headers from
    the diff must be present. Checks for the section-header filter '^+##'.
    """
    check = find_changelog_gate_check()
    text = check.read_text(encoding="utf-8")
    assert "^+##" in text, (
        "AC-164-003 FAIL: section-header filter (^+##) not found in bin/changelog-gate-check.\n"
        "Expected: grep -v '^+##' to strip section headers from the content line count."
    )
    print(f"  [PASS] grep section-header filter (^+##) present in changelog-gate-check")


# ---------------------------------------------------------------------------
# Behavioral tests (B01–B05): execute the gate logic against crafted diff
# fixtures, verify exit codes / output messages, and guard the exec bit
# (F-S164P1-001 companion; B05 added for F-S164P2-001).
# ---------------------------------------------------------------------------

def _run_gate(diff_text: str) -> tuple[int, str]:
    """Run bin/changelog-gate-check with diff_text piped to stdin.

    Returns (returncode, combined stdout+stderr).
    """
    gate = find_changelog_gate_check()
    result = subprocess.run(
        ["bash", str(gate)],
        input=diff_text,
        capture_output=True,
        text=True,
    )
    return result.returncode, result.stdout + result.stderr


def test_B01_real_content_line_pass() -> None:
    """B01: A diff with a real content line exits 0 (PASS path)."""
    diff = textwrap.dedent("""\
        --- a/CHANGELOG.md
        +++ b/CHANGELOG.md
        @@ -8,6 +8,7 @@
         ## [Unreleased]

        +- Added the new widget feature.

         ### Added
    """)
    rc, out = _run_gate(diff)
    assert rc == 0, f"B01: expected exit 0, got {rc}\nout={out!r}"
    assert "PASS" in out, f"B01: expected PASS in output, got {out!r}"
    print(f"  [PASS] B01 real content line → exit 0 PASS: exit={rc}, out={out.strip()!r}")


def test_B02_blank_only_touch_fail() -> None:
    """B02: A diff with only blank line additions exits 1 with whitespace-only message."""
    diff = textwrap.dedent("""\
        --- a/CHANGELOG.md
        +++ b/CHANGELOG.md
        @@ -8,6 +8,9 @@
         ## [Unreleased]
        +
        +
        +
         ### Added
    """)
    rc, out = _run_gate(diff)
    assert rc == 1, f"B02: expected exit 1, got {rc}\nout={out!r}"
    assert "whitespace-only" in out, (
        f"B02: expected 'whitespace-only' in output for blank-only diff, got {out!r}"
    )
    print(f"  [PASS] B02 blank-only touch → exit 1 with whitespace-only: exit={rc}")


def test_B03_section_header_only_add_fail() -> None:
    """B03: A diff with only section header additions exits 1 (headers do not count as content)."""
    diff = textwrap.dedent("""\
        --- a/CHANGELOG.md
        +++ b/CHANGELOG.md
        @@ -8,6 +8,9 @@
         ## [Unreleased]
        +## New Section
        +### Subsection
        +#### Sub-subsection
         ### Added
    """)
    rc, out = _run_gate(diff)
    assert rc == 1, f"B03: expected exit 1, got {rc}\nout={out!r}"
    assert "FAIL" in out, (
        f"B03: expected 'FAIL' in output for header-only diff, got {out!r}"
    )
    print(f"  [PASS] B03 section-header-only add → exit 1 FAIL: exit={rc}")


def test_B04_deletions_only_fail() -> None:
    """B04: A diff with only deletions (no content additions) exits 1."""
    diff = textwrap.dedent("""\
        --- a/CHANGELOG.md
        +++ b/CHANGELOG.md
        @@ -8,7 +8,6 @@
         ## [Unreleased]
        -old entry removed

         ### Added
    """)
    rc, out = _run_gate(diff)
    assert rc == 1, f"B04: expected exit 1, got {rc}\nout={out!r}"
    assert "FAIL" in out, (
        f"B04: expected 'FAIL' in output for deletions-only diff, got {out!r}"
    )
    print(f"  [PASS] B04 deletions-only → exit 1 FAIL: exit={rc}")


def test_B05_exec_bit_direct_invocation() -> None:
    """B05 (F-S164P2-001): Script is invocable via direct path without 'bash' prefix.

    ci.yml invokes bin/changelog-gate-check as a bare path; if the exec bit is
    missing, CI fails with exit 126 while bash-prefixed tests stay green. This
    test detects that gap by calling the script directly (requires 100755 mode
    and a valid shebang).
    """
    gate = find_changelog_gate_check()
    diff = textwrap.dedent("""\
        --- a/CHANGELOG.md
        +++ b/CHANGELOG.md
        @@ -1,3 +1,4 @@
        +- Real content addition.
    """)
    result = subprocess.run(
        [str(gate)],  # direct path — no "bash" prefix; requires exec bit + shebang
        input=diff,
        capture_output=True,
        text=True,
    )
    combined = result.stdout + result.stderr
    assert result.returncode == 0, (
        f"B05: direct invocation failed (exit {result.returncode}) — "
        f"exec bit may be missing (run: git ls-files -s bin/changelog-gate-check, "
        f"should show 100755) or shebang is invalid.\nout={combined!r}"
    )
    assert "PASS" in combined, f"B05: expected PASS in output, got {combined!r}"
    print(f"  [PASS] B05 direct invocation (no bash prefix): exit={result.returncode}")


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
        test_B01_real_content_line_pass,
        test_B02_blank_only_touch_fail,
        test_B03_section_header_only_add_fail,
        test_B04_deletions_only_fail,
        test_B05_exec_bit_direct_invocation,
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
