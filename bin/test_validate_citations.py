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
  T19  F-S164P3-003: unreadable citations file (chmod 000) → exit 2, no traceback
        (skipped with note when os.access reports readable, e.g. running as root)
  T20  F-S164P6-001: non-UTF-8 bytes on stdin → exit 2, no traceback
  T21  F-S164P8-001: citation to an existing directory → NOT A FILE, exit 1, no traceback
  T22  F-S164P8-001: unreadable cited target (chmod 000) → UNREADABLE, exit 1, no traceback
        (skipped with note when os.access reports readable, e.g. running as root)
  T23  AC-166-001(a)-(b): path:line:anchor with anchor present on the cited
        line → exit 0 (also folds in EC-002 regex-special-char anchor)
  T24  AC-166-001(c): path:line:anchor with anchor absent from the cited
        line → SYMBOL NOT AT LINE failure class, exit 1
  T25  AC-166-001(d): bare path:line citation on the same fixture still
        passes — backward-compatibility control
  T26  EC-003: range citation `path:start-end:anchor` asserts the anchor
        against the START line only -- anchor-on-start-but-not-later passes,
        anchor-absent-on-start-but-present-on-a-later-line fails
  T27  AC-166-001(c): SYMBOL NOT AT LINE message truncates the found
        line-text to <=80 chars (plain text[:80] slice, no ellipsis)
"""

import os
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

TOOL = Path(__file__).resolve().parent / "validate-citations"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
#
# ROUTE-W74 MINOR-1: the previous _run() helper (single-file-only invocation,
# with a temp citations file separate from the WIRERUST_REPO_ROOT temp dir)
# was dead code -- every test uses _run_with_real_files() instead, which
# creates cited files inside the same temp dir used as the repo root. Removed
# rather than documented: _run_with_real_files() supersedes it entirely and
# no test exercises the separate-temp-dir shape.


def _run_with_real_files(
    citations_content: str,
    real_files: dict[str, bytes],
) -> tuple[int, str, str]:
    """
    Create real_files inside a temp directory (which is used as the repo root),
    write citations_content pointing to those files, and invoke the tool.

    real_files: {relative_path: file_contents_bytes}
    """
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


def test_T19_unreadable_citations_file_exits_2() -> None:
    """T19 (F-S164P3-003): Unreadable citations file (chmod 000) exits 2, not traceback.

    If the process runs as root (where chmod 000 is still readable), the test
    skips with a printed note rather than giving a false pass or false fail.
    """
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".txt", delete=False, encoding="utf-8"
    ) as f:
        f.write("doc/file.md:1\n")
        citations_path = f.name

    try:
        os.chmod(citations_path, 0o000)

        if os.access(citations_path, os.R_OK):
            # Running as root — chmod 000 does not prevent reads; skip to avoid
            # a false pass (tool would succeed) or false fail (wrong assertion).
            print(
                f"  [SKIP] T19 skipped: process can read chmod-000 file "
                f"(running as root or equivalent); test not applicable."
            )
            return

        with tempfile.TemporaryDirectory() as tmp:
            env = {**os.environ, "WIRERUST_REPO_ROOT": tmp}
            result = subprocess.run(
                [sys.executable, str(TOOL), citations_path],
                capture_output=True,
                text=True,
                env=env,
            )
    finally:
        # Restore read permission so the temp file cleanup can proceed.
        try:
            os.chmod(citations_path, stat.S_IRUSR | stat.S_IWUSR)
        except OSError:
            pass
        Path(citations_path).unlink(missing_ok=True)

    assert result.returncode == 2, (
        f"T19: expected exit 2 for unreadable citations file, got {result.returncode}\n"
        f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
    )
    combined = result.stdout + result.stderr
    assert "PermissionError" not in combined and "Traceback" not in combined, (
        f"T19: expected no raw traceback, but got: {combined!r}"
    )
    print(f"  [PASS] T19 unreadable file → exit 2: exit={result.returncode}")


def test_T20_non_utf8_stdin_exits_2() -> None:
    """T20 (F-S164P6-001): Non-UTF-8 bytes on stdin must produce exit 2, not traceback.

    The stdin branch previously called sys.stdin.read() bare; a text-stream
    decode error raised UnicodeDecodeError → traceback + exit 1. The fix
    reads sys.stdin.buffer and decodes explicitly, so the same error path as
    the file-argument branch applies (stderr message + exit 2).
    """
    invalid_utf8 = b"\xff\xfe invalid bytes"

    with tempfile.TemporaryDirectory() as tmp:
        env = {**os.environ, "WIRERUST_REPO_ROOT": tmp}
        result = subprocess.run(
            [sys.executable, str(TOOL)],
            input=invalid_utf8,
            capture_output=True,
            env=env,
        )

    assert result.returncode == 2, (
        f"T20: expected exit 2 for non-UTF-8 stdin, got {result.returncode}\n"
        f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
    )
    combined = result.stdout + result.stderr
    assert b"Traceback" not in combined, (
        f"T20: expected no raw traceback, but got: {combined!r}"
    )
    print(f"  [PASS] T20 non-UTF-8 stdin → exit 2: exit={result.returncode}")


def test_T21_directory_target_not_a_file() -> None:
    """T21 (F-S164P8-001): A citation to an existing directory → NOT A FILE, exit 1.

    A directory passes abs_path.exists() but is_file() returns False; the
    unguarded count_lines() call would raise IsADirectoryError. The fix adds
    an is_file() check that produces a clean NOT A FILE diagnostic instead.
    """
    with tempfile.TemporaryDirectory() as tmp:
        # Create a subdirectory inside the repo root to cite.
        subdir = Path(tmp) / "docs"
        subdir.mkdir()
        # Write citations file citing the directory as if it were a file.
        citations = "docs:1\n"
        env = {**os.environ, "WIRERUST_REPO_ROOT": tmp}
        result = subprocess.run(
            [sys.executable, str(TOOL), "-"],
            input=citations,
            capture_output=True,
            text=True,
            env=env,
        )

    assert result.returncode == 1, (
        f"T21: expected exit 1 for directory target, got {result.returncode}\n"
        f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
    )
    combined = result.stdout + result.stderr
    assert "NOT A FILE" in combined, (
        f"T21: expected 'NOT A FILE' in output, got: {combined!r}"
    )
    assert "Traceback" not in combined, (
        f"T21: expected no raw traceback, but got: {combined!r}"
    )
    print(f"  [PASS] T21 directory target → NOT A FILE, exit 1: exit={result.returncode}")


def test_T22_unreadable_target_file() -> None:
    """T22 (F-S164P8-001): Unreadable cited target file (chmod 000) → UNREADABLE, exit 1.

    If the process runs as root (where chmod 000 is still readable), the test
    skips with a printed note rather than giving a false pass or false fail.
    """
    with tempfile.TemporaryDirectory() as tmp:
        # Create a real file in the temp root, then lock it down.
        target = Path(tmp) / "locked.txt"
        target.write_text("line one\n", encoding="utf-8")
        target.chmod(0o000)

        if os.access(str(target), os.R_OK):
            target.chmod(stat.S_IRUSR | stat.S_IWUSR)
            print(
                "  [SKIP] T22 skipped: process can read chmod-000 file "
                "(running as root or equivalent); test not applicable."
            )
            return

        try:
            citations = "locked.txt:1\n"
            env = {**os.environ, "WIRERUST_REPO_ROOT": tmp}
            result = subprocess.run(
                [sys.executable, str(TOOL), "-"],
                input=citations,
                capture_output=True,
                text=True,
                env=env,
            )
        finally:
            try:
                target.chmod(stat.S_IRUSR | stat.S_IWUSR)
            except OSError:
                pass

    assert result.returncode == 1, (
        f"T22: expected exit 1 for unreadable target, got {result.returncode}\n"
        f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
    )
    combined = result.stdout + result.stderr
    assert "UNREADABLE" in combined, (
        f"T22: expected 'UNREADABLE' in output, got: {combined!r}"
    )
    assert "Traceback" not in combined, (
        f"T22: expected no raw traceback, but got: {combined!r}"
    )
    print(f"  [PASS] T22 unreadable target → UNREADABLE, exit 1: exit={result.returncode}")


def test_T23_anchor_present_passes() -> None:
    """T23 (AC-166-001(a)-(b)): a path:line:anchor citation where the anchor
    IS present on the cited line exits 0 (PASS).

    Also folds in EC-002 (anchor field containing regex-special characters,
    e.g. 'arr[0]') as a second citation against the same fixture file: the
    tool MUST re.escape() the anchor before pattern-matching so '[' and ']'
    are treated as literal characters rather than a regex character class.
    """
    file_content = (
        b"# header comment\n"
        b"def some_function():\n"
        b"    return 1\n"
        b"\n"
        b"value = arr[0] + 1  # uses arr[0]\n"
    )
    citations = (
        "mod.py:2:some_function\n"  # def <anchor> present at line 2
        "mod.py:5:arr[0]\n"  # EC-002: regex-special anchor, substring match at line 5
    )
    rc, out, err = _run_with_real_files(citations, {"mod.py": file_content})
    assert rc == 0, f"T23: expected exit 0, got {rc}\nstdout={out!r}\nstderr={err!r}"
    assert "PASS" in out, f"T23: expected PASS in stdout, got {out!r}"
    assert "2" in out, f"T23: expected count 2 in stdout, got {out!r}"
    print(
        "  [PASS] T23 anchor present (+ EC-002 regex-special anchor): "
        f"exit={rc}, out={out.strip()!r}"
    )


def test_T24_anchor_absent_symbol_not_at_line() -> None:
    """T24 (AC-166-001(c)): a path:line:anchor citation where the anchor is
    NOT present on the cited line exits 1 with the SYMBOL NOT AT LINE
    failure class, per the exact message shape in AC-166-001(c):
    'SYMBOL NOT AT LINE: path:line (expected anchor '<anchor>', found '<line-text>')'.
    """
    file_content = (
        b"# header comment\n"
        b"def some_function():\n"
        b"    return 1\n"
    )
    citations = "mod.py:2:nonexistent_function\n"
    rc, out, err = _run_with_real_files(citations, {"mod.py": file_content})
    assert rc == 1, f"T24: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "SYMBOL NOT AT LINE:" in combined, (
        f"T24: expected 'SYMBOL NOT AT LINE:' failure-class prefix in output, "
        f"got stdout={out!r} stderr={err!r}"
    )
    assert "mod.py:2" in combined, (
        f"T24: expected cited path:line 'mod.py:2' in message, got {combined!r}"
    )
    assert "nonexistent_function" in combined, (
        f"T24: expected expected-anchor 'nonexistent_function' in message, got {combined!r}"
    )
    assert "def some_function():" in combined, (
        f"T24: expected found-line-text 'def some_function():' in message, got {combined!r}"
    )
    print(f"  [PASS] T24 anchor absent -> SYMBOL NOT AT LINE, exit 1: exit={rc}")


def test_T25_bare_citation_still_passes() -> None:
    """T25 (AC-166-001(d)): a bare path:line citation (no anchor field) on
    the same fixture file still exits 0 -- backward-compatibility control.

    NOTE: this is a control, not a Red Gate probe. Bare path:line citations
    are handled by pre-existing (pre-AC-166-001) code and served as the
    backward-compat control during the Red Gate run (passed both before and
    after the anchor-grammar extension landed), confirming the anchor-grammar
    work did not regress pre-existing behavior, per AC-166-001(d).
    """
    file_content = (
        b"# header comment\n"
        b"def some_function():\n"
        b"    return 1\n"
    )
    citations = "mod.py:2\n"
    rc, out, err = _run_with_real_files(citations, {"mod.py": file_content})
    assert rc == 0, f"T25: expected exit 0, got {rc}\nstdout={out!r}\nstderr={err!r}"
    assert "PASS" in out, f"T25: expected PASS in stdout, got {out!r}"
    print(f"  [PASS] T25 bare citation backward-compat control: exit={rc}, out={out.strip()!r}")


def test_T26_range_citation_anchor_asserts_start_line_only() -> None:
    """T26 (EC-003): for a range citation `path:start-end:anchor`, the anchor
    assertion applies ONLY to the range's start line -- the end line is
    bounds-checked only, never anchor-checked. This test proves both
    directions of that semantic:
      (a) anchor present on the start line, absent from a later line in the
          same range -> PASS (exit 0), because only the start line matters.
      (b) anchor absent from the start line, but present on a LATER line
          within the range -> FAIL (exit 1, SYMBOL NOT AT LINE), proving a
          later-line match does NOT satisfy the assertion.
    """
    file_content = (
        b"# header comment\n"       # line 1
        b"def some_function():\n"   # line 2 -- anchor present here
        b"    return 1\n"           # line 3 -- anchor absent here
        b"def some_function():\n"   # line 4 -- anchor present here (later line)
        b"    pass\n"                # line 5
    )

    # (a) start line (2) has the anchor; later line (3) in the range does not.
    citations_pass = "mod.py:2-3:some_function\n"
    rc, out, err = _run_with_real_files(citations_pass, {"mod.py": file_content})
    assert rc == 0, (
        f"T26a: expected exit 0 (anchor present on start line satisfies the "
        f"range assertion regardless of later lines), got {rc}\n"
        f"stdout={out!r}\nstderr={err!r}"
    )
    assert "PASS" in out, f"T26a: expected PASS in stdout, got {out!r}"

    # (b) start line (3) lacks the anchor; a later line (4) in the range has it.
    citations_fail = "mod.py:3-4:some_function\n"
    rc2, out2, err2 = _run_with_real_files(citations_fail, {"mod.py": file_content})
    assert rc2 == 1, (
        f"T26b: expected exit 1 (a match on a later line does not satisfy "
        f"the start-line-only anchor assertion), got {rc2}\n"
        f"stdout={out2!r}\nstderr={err2!r}"
    )
    combined2 = out2 + err2
    assert "SYMBOL NOT AT LINE" in combined2, (
        f"T26b: expected SYMBOL NOT AT LINE in output, got {combined2!r}"
    )
    assert "mod.py:3" in combined2, (
        f"T26b: expected the failure to cite the start line 'mod.py:3' "
        f"(not the end line), got {combined2!r}"
    )
    print(
        "  [PASS] T26 range anchor asserts start-line-only: "
        f"a_exit={rc}, b_exit={rc2}"
    )


def test_T27_symbol_failure_message_truncates_long_line() -> None:
    """T27 (AC-166-001(c) truncation): the cited line's text embedded in a
    SYMBOL NOT AT LINE failure message is truncated to at most 80 chars.

    Per bin/validate-citations (see `_truncate_for_message`:
    `return text[:limit]`), truncation is a plain slice with NO ellipsis
    or other marker appended -- this test asserts that exact observed
    behavior rather than assuming an ellipsis convention.
    """
    long_line = "x" * 150  # far exceeds the 80-char truncation limit
    file_content = (
        b"# header comment\n"          # line 1
        + (long_line.encode() + b"\n")  # line 2 -- long line, anchor absent
        + b"    pass\n"                 # line 3
    )
    citations = "mod.py:2:nonexistent_symbol\n"
    rc, out, err = _run_with_real_files(citations, {"mod.py": file_content})
    assert rc == 1, f"T27: expected exit 1, got {rc}\nstdout={out!r}\nstderr={err!r}"
    combined = out + err
    assert "SYMBOL NOT AT LINE" in combined, (
        f"T27: expected SYMBOL NOT AT LINE in output, got {combined!r}"
    )

    # Extract the found '<line-text>' portion of the message.
    marker = "found '"
    start_idx = combined.index(marker) + len(marker)
    end_idx = combined.index("')", start_idx)
    found_text = combined[start_idx:end_idx]

    assert len(found_text) <= 80, (
        f"T27: expected truncated line text to be <=80 chars, got "
        f"{len(found_text)} chars: {found_text!r}"
    )
    assert found_text == "x" * 80, (
        f"T27: expected exact plain-slice truncation ('x'*80, no ellipsis, "
        f"per _truncate_for_message's text[:limit] implementation), "
        f"got {found_text!r}"
    )
    print(
        "  [PASS] T27 long-line SYMBOL NOT AT LINE message truncated to "
        f"<=80 chars: exit={rc}, found_len={len(found_text)}"
    )


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
        test_T19_unreadable_citations_file_exits_2,
        test_T20_non_utf8_stdin_exits_2,
        test_T21_directory_target_not_a_file,
        test_T22_unreadable_target_file,
        test_T23_anchor_present_passes,
        test_T24_anchor_absent_symbol_not_at_line,
        test_T25_bare_citation_still_passes,
        test_T26_range_citation_anchor_asserts_start_line_only,
        test_T27_symbol_failure_message_truncates_long_line,
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
