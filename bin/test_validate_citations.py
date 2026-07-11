#!/usr/bin/env python3
"""
Self-test for bin/validate-citations (AC-164-002).

Tests verify the delivered behavior of the tool against real file fixtures.
Each test covers a distinct validation scenario as documented in the tool's
ALGORITHM section (see bin/validate-citations).

Run: python3 bin/test_validate_citations.py

Test coverage (AC-164-002(d) + edge cases):
  T01  Valid file:line citation passes (exit 0, stdout "PASS: 1 citations verified")
  T02  Valid file:line-range citation passes (exit 0)
  T03  Nonexistent file is rejected with FILE NOT FOUND (exit 1)
  T04  Out-of-range single line is rejected with LINE OUT OF RANGE (exit 1)
  T05  Out-of-range range endpoint (second endpoint) is rejected (exit 1)
  T06  Comment lines and blank lines are ignored (exit 0)
  T07  Empty input (no citations) → "PASS: 0 citations verified", exit 0
  T08  EC-002: start > end range → "INVALID RANGE", exit 1
  T09  Exit code 2 on bad argument (usage error)
  T10  Multiple valid citations all pass (exit 0, count matches)
  T11  Mixed valid + invalid → correct failure count and exit 1
  T12  F-S164P1-002: non-blank, non-comment, unparseable line → MALFORMED, exit 1
  T13  F-S164P1-004: line number 0 → INVALID LINE, exit 1
  T14  F-S164P1-004: range start 0 → INVALID LINE, exit 1
  T15  F-S164P2-002: malformed-only input → FAIL: 1 of 1 (denominator includes MALFORMED)
  T16  F-S164P2-003: absolute path → OUTSIDE REPO, exit 1 (CWE-22)
  T17  F-S164P2-003: parent-escape path → OUTSIDE REPO, exit 1 (CWE-22)
  T18  F-S164P2-004: non-UTF-8 citations file → exit 2, no traceback
"""

import subprocess
import sys
import tempfile
from pathlib import Path

TOOL = Path(__file__).resolve().parent / "validate-citations"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _run(citations_content: str, extra_args: list[str] | None = None) -> tuple[int, str, str]:
    """
    Write citations_content to a temp file, invoke the tool with that file,
    and return (returncode, stdout, stderr).
    """
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".txt", delete=False, encoding="utf-8"
    ) as f:
        f.write(citations_content)
        citations_path = f.name

    with tempfile.TemporaryDirectory() as tmp:
        env_override: dict[str, str] = {"WIRERUST_REPO_ROOT": tmp}
        import os
        env = {**os.environ, **env_override}

        cmd = [sys.executable, str(TOOL), citations_path] + (extra_args or [])
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            env=env,
        )

    # Clean up
    Path(citations_path).unlink(missing_ok=True)

    return result.returncode, result.stdout, result.stderr


def _run_with_real_files(
    citations_content: str,
    real_files: dict[str, bytes],
) -> tuple[int, str, str]:
    """
    Create real_files inside a temp directory (which is used as the repo root),
    write citations_content pointing to those files, and invoke the tool.

    real_files: {relative_path: file_contents_bytes}
    """
    import os

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        # Create the files inside the temp dir
        for rel_path, content in real_files.items():
            full_path = tmp_path / rel_path
            full_path.parent.mkdir(parents=True, exist_ok=True)
            full_path.write_bytes(content)

        # Write the citations file (paths are relative to repo root = tmp_path)
        citations_file = tmp_path / "citations.txt"
        citations_file.write_text(citations_content, encoding="utf-8")

        env = {**os.environ, "WIRERUST_REPO_ROOT": str(tmp_path)}
        cmd = [sys.executable, str(TOOL), str(citations_file)]
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            env=env,
        )

    return result.returncode, result.stdout, result.stderr


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

def test_T01_valid_line_citation_passes() -> None:
    """T01: A valid file:line citation (within bounds) exits 0 with PASS message."""
    # File has 5 lines; cite line 3
    file_content = b"line1\nline2\nline3\nline4\nline5\n"
    citations = "doc/file.md:3\n"
    rc, out, err = _run_with_real_files(citations, {"doc/file.md": file_content})
    assert rc == 0, f"T01: expected exit 0, got {rc}\nstdout={out!r}\nstderr={err!r}"
    assert "PASS" in out, f"T01: expected 'PASS' in stdout, got {out!r}"
    assert "1" in out, f"T01: expected count '1' in stdout, got {out!r}"
    print(f"  [PASS] T01 valid single-line citation: exit={rc}, out={out.strip()!r}")


def test_T02_valid_range_citation_passes() -> None:
    """T02: A valid file:line-range citation (both endpoints in bounds) exits 0."""
    file_content = b"a\nb\nc\nd\ne\nf\n"  # 6 lines
    citations = "doc/spec.md:2-5\n"
    rc, out, err = _run_with_real_files(citations, {"doc/spec.md": file_content})
    assert rc == 0, f"T02: expected exit 0, got {rc}\nstdout={out!r}\nstderr={err!r}"
    assert "PASS" in out, f"T02: expected 'PASS' in stdout, got {out!r}"
    print(f"  [PASS] T02 valid range citation: exit={rc}, out={out.strip()!r}")


def test_T03_nonexistent_file_rejected() -> None:
    """T03: Citing a file that does not exist exits 1 and prints FILE NOT FOUND."""
    # The repo root has no files; we cite a nonexistent file
    citations = "ghost/missing.md:1\n"
    rc, out, err = _run_with_real_files(citations, {})
    assert rc == 1, f"T03: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "FILE NOT FOUND" in combined or "not found" in combined.lower(), (
        f"T03: expected FILE NOT FOUND in output, got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T03 nonexistent file rejected: exit={rc}")


def test_T04_out_of_range_single_line_rejected() -> None:
    """T04: Citing a line number beyond the file's line count exits 1 with LINE OUT OF RANGE."""
    file_content = b"only\nthree\nlines\n"  # 3 lines
    citations = "notes.md:10\n"  # line 10 doesn't exist
    rc, out, err = _run_with_real_files(citations, {"notes.md": file_content})
    assert rc == 1, f"T04: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "LINE OUT OF RANGE" in combined or "out of range" in combined.lower(), (
        f"T04: expected LINE OUT OF RANGE in output, got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T04 out-of-range single line rejected: exit={rc}")


def test_T05_out_of_range_range_endpoint_rejected() -> None:
    """T05: A range whose end endpoint exceeds file length exits 1 with LINE OUT OF RANGE."""
    file_content = b"line1\nline2\nline3\n"  # 3 lines
    citations = "doc.md:2-9\n"  # endpoint 9 > 3
    rc, out, err = _run_with_real_files(citations, {"doc.md": file_content})
    assert rc == 1, f"T05: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "LINE OUT OF RANGE" in combined or "out of range" in combined.lower(), (
        f"T05: expected LINE OUT OF RANGE in output, got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T05 out-of-range range endpoint rejected: exit={rc}")


def test_T06_comments_and_blanks_ignored() -> None:
    """T06: Comment lines (#...) and blank lines are silently ignored; valid refs still pass."""
    file_content = b"alpha\nbeta\ngamma\n"  # 3 lines
    citations = (
        "# This is a comment — ignored\n"
        "\n"
        "  \n"
        "target.md:2\n"
        "# another comment\n"
    )
    rc, out, err = _run_with_real_files(citations, {"target.md": file_content})
    assert rc == 0, (
        f"T06: expected exit 0 (only real citation is valid), got {rc}\n"
        f"stdout={out!r}\nstderr={err!r}"
    )
    assert "PASS" in out, f"T06: expected PASS in stdout, got {out!r}"
    print(f"  [PASS] T06 comments and blank lines ignored: exit={rc}, out={out.strip()!r}")


def test_T07_empty_input_passes() -> None:
    """T07: An input file with no citations (empty / all comments) exits 0 with count 0."""
    citations = "# nothing here\n\n"
    rc, out, err = _run_with_real_files(citations, {})
    assert rc == 0, f"T07: expected exit 0, got {rc}\nstdout={out!r}\nstderr={err!r}"
    assert "PASS" in out, f"T07: expected PASS in stdout, got {out!r}"
    assert "0" in out, f"T07: expected '0 citations' in stdout, got {out!r}"
    print(f"  [PASS] T07 empty input: exit={rc}, out={out.strip()!r}")


def test_T08_invalid_range_start_gt_end() -> None:
    """T08 (EC-002): A range where start > end exits 1 with INVALID RANGE."""
    file_content = b"a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np\nq\nr\ns\nt\n"  # 20 lines
    citations = "big.md:15-5\n"  # start (15) > end (5)
    rc, out, err = _run_with_real_files(citations, {"big.md": file_content})
    assert rc == 1, f"T08: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "INVALID RANGE" in combined or "start > end" in combined.lower(), (
        f"T08: expected INVALID RANGE in output, got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T08 invalid range (start > end) rejected: exit={rc}")


def test_T09_bad_argument_exits_2() -> None:
    """T09: Passing a nonexistent citations file as argument exits 2 (usage error)."""
    import os
    with tempfile.TemporaryDirectory() as tmp:
        env = {**os.environ, "WIRERUST_REPO_ROOT": tmp}
        nonexistent = "/absolutely/does/not/exist/citations.txt"
        result = subprocess.run(
            [sys.executable, str(TOOL), nonexistent],
            capture_output=True,
            text=True,
            env=env,
        )
    assert result.returncode == 2, (
        f"T09: expected exit 2 for nonexistent citations file, got {result.returncode}\n"
        f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
    )
    print(f"  [PASS] T09 nonexistent citations file → exit 2: exit={result.returncode}")


def test_T10_multiple_valid_citations_count() -> None:
    """T10: Multiple valid citations all pass and the count in the PASS message is correct."""
    file_a = b"line1\nline2\nline3\n"
    file_b = b"alpha\nbeta\ngamma\ndelta\n"
    citations = (
        "# Three valid citations\n"
        "a.md:1\n"
        "a.md:1-3\n"
        "b.md:4\n"
    )
    rc, out, err = _run_with_real_files(citations, {"a.md": file_a, "b.md": file_b})
    assert rc == 0, f"T10: expected exit 0, got {rc}\nstdout={out!r}\nstderr={err!r}"
    assert "3" in out, f"T10: expected count 3 in stdout, got {out!r}"
    assert "PASS" in out, f"T10: expected PASS in stdout, got {out!r}"
    print(f"  [PASS] T10 multiple valid citations: exit={rc}, out={out.strip()!r}")


def test_T11_mixed_valid_and_invalid() -> None:
    """T11: Mixed valid + invalid citations exit 1 with correct failure count."""
    file_content = b"only\ntwo\nlines\n"  # 3 lines
    citations = (
        "real.md:2\n"       # valid
        "real.md:99\n"      # out of range
        "ghost.md:1\n"      # file not found
    )
    rc, out, err = _run_with_real_files(citations, {"real.md": file_content})
    assert rc == 1, f"T11: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "FAIL" in combined, f"T11: expected FAIL in output, got stdout={out!r} stderr={err!r}"
    # Expect "2 of 3" invalid
    assert "2" in combined, (
        f"T11: expected failure count '2' in output, got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T11 mixed valid+invalid → FAIL with count: exit={rc}")


def test_T12_malformed_line_reported() -> None:
    """T12 (F-S164P1-002): A non-blank, non-comment, unparseable line exits 1 with MALFORMED."""
    # 'src/decoder.rs 196-210' uses a space instead of colon — fails the citation regex
    citations = "src/decoder.rs 196-210\n"
    rc, out, err = _run_with_real_files(citations, {})
    assert rc == 1, (
        f"T12: expected exit 1 for malformed citation (space instead of colon), "
        f"got {rc}\nstdout={out!r}\nstderr={err!r}"
    )
    combined = out + err
    assert "MALFORMED" in combined, (
        f"T12: expected MALFORMED in output for unparseable line, "
        f"got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T12 malformed citation reported: exit={rc}")


def test_T13_zero_line_number_rejected() -> None:
    """T13 (F-S164P1-004): A citation with line number 0 exits 1 with INVALID LINE."""
    file_content = b"line1\nline2\nline3\n"
    citations = "notes.md:0\n"
    rc, out, err = _run_with_real_files(citations, {"notes.md": file_content})
    assert rc == 1, (
        f"T13: expected exit 1 for line number 0, "
        f"got {rc}\nstdout={out!r}\nstderr={err!r}"
    )
    combined = out + err
    assert "INVALID LINE" in combined, (
        f"T13: expected INVALID LINE in output for line 0, "
        f"got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T13 zero line number rejected: exit={rc}")


def test_T14_zero_range_start_rejected() -> None:
    """T14 (F-S164P1-004): A range citation with start=0 exits 1 with INVALID LINE."""
    # Use a 10-line file so the range 0-5 would pass bounds checks if line-number
    # validation were absent — confirming the INVALID LINE check fires first.
    file_content = b"".join(f"line{i}\n".encode() for i in range(1, 11))
    citations = "notes.md:0-5\n"
    rc, out, err = _run_with_real_files(citations, {"notes.md": file_content})
    assert rc == 1, (
        f"T14: expected exit 1 for range start=0, "
        f"got {rc}\nstdout={out!r}\nstderr={err!r}"
    )
    combined = out + err
    assert "INVALID LINE" in combined, (
        f"T14: expected INVALID LINE in output for range start 0, "
        f"got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T14 zero range start rejected: exit={rc}")


def test_T15_malformed_counts_in_fail_denominator() -> None:
    """T15 (F-S164P2-002): MALFORMED lines count toward the FAIL message denominator.

    A malformed-only input must give 'FAIL: 1 of 1 citations invalid', not
    'FAIL: 1 of 0' (which occurs when MALFORMED lines increment failures but
    not the total line count used as N).
    """
    # Space instead of colon — unparseable by the citation regex
    citations = "src/decoder.rs 196-210\n"
    rc, out, err = _run_with_real_files(citations, {})
    assert rc == 1, f"T15: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "1 of 1" in combined, (
        f"T15: expected 'FAIL: 1 of 1' (malformed counted in denominator), "
        f"got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T15 malformed counted in FAIL denominator: exit={rc}")


def test_T16_absolute_path_rejected() -> None:
    """T16 (F-S164P2-003): Absolute path citations exit 1 with OUTSIDE REPO (CWE-22).

    pathlib's / operator discards the left side when the right side is absolute,
    so repo_root / '/etc/passwd' resolves to /etc/passwd — outside the repo root.
    The tool must detect and reject this rather than silently reading the file.
    """
    citations = "/etc/passwd:1\n"
    rc, out, err = _run_with_real_files(citations, {})
    assert rc == 1, (
        f"T16: expected exit 1 for absolute path, got {rc}\n"
        f"stdout={out!r}\nstderr={err!r}"
    )
    combined = out + err
    assert "OUTSIDE REPO" in combined, (
        f"T16: expected OUTSIDE REPO for absolute path citation, "
        f"got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T16 absolute path rejected as OUTSIDE REPO: exit={rc}")


def test_T17_parent_escape_rejected() -> None:
    """T17 (F-S164P2-003): Parent-directory escape citations exit 1 with OUTSIDE REPO (CWE-22).

    A path like '../../etc/passwd' resolves to a location above the repo root,
    which must be rejected regardless of whether that file exists.
    """
    citations = "../../etc/passwd:1\n"
    rc, out, err = _run_with_real_files(citations, {})
    assert rc == 1, (
        f"T17: expected exit 1 for parent-escape path, got {rc}\n"
        f"stdout={out!r}\nstderr={err!r}"
    )
    combined = out + err
    assert "OUTSIDE REPO" in combined, (
        f"T17: expected OUTSIDE REPO for parent-escape citation, "
        f"got stdout={out!r} stderr={err!r}"
    )
    print(f"  [PASS] T17 parent-escape path rejected as OUTSIDE REPO: exit={rc}")


def test_T18_non_utf8_citations_file_exits_2() -> None:
    """T18 (F-S164P2-004): Non-UTF-8 citations file exits 2 with error message, not traceback."""
    import os
    import tempfile

    # Write bytes that are not valid UTF-8
    invalid_bytes = b"\xff\xfeThis is not valid UTF-8\n"
    with tempfile.NamedTemporaryFile(delete=False, suffix=".txt") as f:
        f.write(invalid_bytes)
        citations_path = f.name

    try:
        with tempfile.TemporaryDirectory() as tmp:
            env = {**os.environ, "WIRERUST_REPO_ROOT": tmp}
            result = subprocess.run(
                [sys.executable, str(TOOL), citations_path],
                capture_output=True,
                text=True,
                env=env,
            )
    finally:
        Path(citations_path).unlink(missing_ok=True)

    assert result.returncode == 2, (
        f"T18: expected exit 2 for non-UTF-8 citations file, got {result.returncode}\n"
        f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
    )
    combined = result.stdout + result.stderr
    assert "UnicodeDecodeError" not in combined, (
        f"T18: expected no raw traceback, but UnicodeDecodeError appears in output: "
        f"{combined!r}"
    )
    print(f"  [PASS] T18 non-UTF-8 citations file → exit 2: exit={result.returncode}")


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def main() -> None:
    tests = [
        test_T01_valid_line_citation_passes,
        test_T02_valid_range_citation_passes,
        test_T03_nonexistent_file_rejected,
        test_T04_out_of_range_single_line_rejected,
        test_T05_out_of_range_range_endpoint_rejected,
        test_T06_comments_and_blanks_ignored,
        test_T07_empty_input_passes,
        test_T08_invalid_range_start_gt_end,
        test_T09_bad_argument_exits_2,
        test_T10_multiple_valid_citations_count,
        test_T11_mixed_valid_and_invalid,
        test_T12_malformed_line_reported,
        test_T13_zero_line_number_rejected,
        test_T14_zero_range_start_rejected,
        test_T15_malformed_counts_in_fail_denominator,
        test_T16_absolute_path_rejected,
        test_T17_parent_escape_rejected,
        test_T18_non_utf8_citations_file_exits_2,
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
