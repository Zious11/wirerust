#!/usr/bin/env python3
"""
Self-test for compute-input-hash.

NOTE: This is the NEW canonical algorithm (re-baseline). There is no legacy
hash to reproduce — the old algorithm was never written down and its
implementation is lost. These tests pin the NEW algorithm's output.

Tests:
  (a) Determinism: same inputs → same hash on repeated calls.
  (b) Known-fixture: two temp files with known content → pinned 7-char hash.
  (c) CRLF/LF normalization: CRLF and LF inputs produce the SAME hash.
"""

import hashlib
import sys
import tempfile
from pathlib import Path

# ---------------------------------------------------------------------------
# Make the tool importable without installing it
# ---------------------------------------------------------------------------
BIN_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(BIN_DIR))

# We import internals directly to test them without subprocess overhead.
# The script isn't a module, so we exec() it into a namespace.
_tool_ns: dict = {}
with open(BIN_DIR / "compute-input-hash") as _f:
    exec(compile(_f.read(), "compute-input-hash", "exec"), _tool_ns)  # noqa: S102

normalize_line_endings = _tool_ns["normalize_line_endings"]
compute_hash = _tool_ns["compute_hash"]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def make_story_md(tmp_dir: Path, input_paths: list[str], name: str = "STORY-TEST") -> Path:
    """Create a minimal story .md with an inputs: block pointing at input_paths."""
    inputs_lines = "".join(f"  - {p}\n" for p in input_paths)
    content = (
        "---\n"
        "document_type: story\n"
        f"story_id: \"{name}\"\n"
        "inputs:\n"
        f"{inputs_lines}"
        "input-hash: \"0000000\"\n"
        "---\n"
        "\n"
        "# Test story body\n"
    )
    story_path = tmp_dir / f"{name}.md"
    story_path.write_text(content, encoding="utf-8")
    return story_path


# ---------------------------------------------------------------------------
# (b) Known-fixture test — compute the canonical hash once and pin it here.
#
# File A content (LF): "hello\nworld\n"   bytes: b"hello\nworld\n"
# File B content (LF): "foo bar\n"         bytes: b"foo bar\n"
# Concatenated (no separator): b"hello\nworld\nfoo bar\n"
# MD5 hexdigest of that:
import hashlib as _hl
_FIXTURE_CONCAT = b"hello\nworld\nfoo bar\n"
_PINNED_FULL_MD5 = _hl.md5(_FIXTURE_CONCAT).hexdigest()
_PINNED_HASH = _PINNED_FULL_MD5[:7]
# Pinned value is computed by this algorithm itself (re-baseline, no legacy).
# ---------------------------------------------------------------------------


def test_determinism() -> None:
    """(a) Same inputs → same hash on two independent calls."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        file_a = tmp_dir / "a.md"
        file_b = tmp_dir / "b.md"
        file_a.write_bytes(b"hello\nworld\n")
        file_b.write_bytes(b"foo bar\n")

        # Paths must be relative to repo_root (tmp_dir is our fake repo root).
        rel_a = "a.md"
        rel_b = "b.md"
        story = make_story_md(tmp_dir, [rel_a, rel_b])

        h1 = compute_hash(story, tmp_dir)
        h2 = compute_hash(story, tmp_dir)
        assert h1 == h2, f"Non-deterministic: {h1} != {h2}"
        print(f"  [PASS] determinism: {h1} == {h2}")


def test_known_fixture() -> None:
    """(b) Known fixture: two files with specific content → pinned hash."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        file_a = tmp_dir / "a.md"
        file_b = tmp_dir / "b.md"
        file_a.write_bytes(b"hello\nworld\n")   # LF
        file_b.write_bytes(b"foo bar\n")         # LF

        story = make_story_md(tmp_dir, ["a.md", "b.md"])
        h = compute_hash(story, tmp_dir)

        assert h == _PINNED_HASH, (
            f"Hash mismatch: got {h!r}, expected {_PINNED_HASH!r}\n"
            f"  (full MD5: {_PINNED_FULL_MD5}, concatenated bytes: {_FIXTURE_CONCAT!r})"
        )
        print(f"  [PASS] known fixture: hash={h!r} matches pinned={_PINNED_HASH!r}")
        print(f"         (full MD5={_PINNED_FULL_MD5}, "
              f"concat={_FIXTURE_CONCAT!r})")


def test_crlf_normalization() -> None:
    """(c) CRLF and LF inputs produce the SAME hash."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)

        # LF version
        lf_dir = tmp_dir / "lf"
        lf_dir.mkdir()
        (lf_dir / "a.md").write_bytes(b"hello\nworld\n")
        (lf_dir / "b.md").write_bytes(b"foo bar\n")
        story_lf = make_story_md(lf_dir, ["a.md", "b.md"])

        # CRLF version (same logical content)
        crlf_dir = tmp_dir / "crlf"
        crlf_dir.mkdir()
        (crlf_dir / "a.md").write_bytes(b"hello\r\nworld\r\n")
        (crlf_dir / "b.md").write_bytes(b"foo bar\r\n")
        story_crlf = make_story_md(crlf_dir, ["a.md", "b.md"])

        h_lf = compute_hash(story_lf, lf_dir)
        h_crlf = compute_hash(story_crlf, crlf_dir)

        assert h_lf == h_crlf, (
            f"CRLF normalization failed: LF hash={h_lf!r} != CRLF hash={h_crlf!r}"
        )
        print(f"  [PASS] CRLF normalization: LF hash={h_lf!r} == CRLF hash={h_crlf!r}")


def test_lone_cr_normalization() -> None:
    """Lone CR (\\r) is normalized to LF — same hash as LF version."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)

        lf_dir = tmp_dir / "lf"
        lf_dir.mkdir()
        (lf_dir / "a.md").write_bytes(b"hello\nworld\n")
        story_lf = make_story_md(lf_dir, ["a.md"])

        cr_dir = tmp_dir / "cr"
        cr_dir.mkdir()
        (cr_dir / "a.md").write_bytes(b"hello\rworld\r")
        story_cr = make_story_md(cr_dir, ["a.md"])

        h_lf = compute_hash(story_lf, lf_dir)
        h_cr = compute_hash(story_cr, cr_dir)

        assert h_lf == h_cr, (
            f"Lone-CR normalization failed: LF={h_lf!r} != CR={h_cr!r}"
        )
        print(f"  [PASS] lone-CR normalization: CR hash={h_cr!r} == LF hash={h_lf!r}")


def test_declaration_order_matters() -> None:
    """Input order matters: swapping inputs produces a DIFFERENT hash."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        (tmp_dir / "a.md").write_bytes(b"alpha")
        (tmp_dir / "b.md").write_bytes(b"beta")

        story_ab = make_story_md(tmp_dir, ["a.md", "b.md"], name="STORY-AB")
        story_ba = make_story_md(tmp_dir, ["b.md", "a.md"], name="STORY-BA")

        h_ab = compute_hash(story_ab, tmp_dir)
        h_ba = compute_hash(story_ba, tmp_dir)

        assert h_ab != h_ba, (
            f"Order-sensitivity failed: both hashes are {h_ab!r} "
            "(swapping inputs should change the hash)"
        )
        print(f"  [PASS] declaration order matters: AB={h_ab!r} != BA={h_ba!r}")


def test_missing_input_raises() -> None:
    """Missing input file raises SystemExit with a clear message (not a silent skip)."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        story = make_story_md(tmp_dir, ["nonexistent.md"])
        try:
            compute_hash(story, tmp_dir)
            raise AssertionError("Expected SystemExit for missing input file")
        except SystemExit as exc:
            msg = str(exc)
            assert "nonexistent.md" in msg or "missing" in msg.lower(), (
                f"Error message doesn't mention the missing file: {msg!r}"
            )
            print(f"  [PASS] missing input raises SystemExit: {msg[:80]!r}...")


# ---------------------------------------------------------------------------
# Red Gate tests — AC-157-003, AC-157-004, AC-157-010
# These tests MUST FAIL until the corresponding fixes land in compute-input-hash.
# Each catches the current SystemExit and converts it to AssertionError so the
# harness runner (which catches Exception) can report [FAIL] cleanly.
# ---------------------------------------------------------------------------

def test_empty_inputs_inline_compact() -> None:
    """AC-157-003: inputs: [] (inline compact YAML) must return d41d8cd, not SystemExit."""
    EMPTY_HASH = hashlib.md5(b"").hexdigest()[:7]  # d41d8cd
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        content = (
            "---\n"
            "document_type: story\n"
            "story_id: \"STORY-EMPTY-INLINE\"\n"
            "inputs: []\n"
            "input-hash: \"0000000\"\n"
            "---\n"
            "\n"
            "# Test story body\n"
        )
        story_path = tmp_dir / "STORY-EMPTY-INLINE.md"
        story_path.write_text(content, encoding="utf-8")
        try:
            h = compute_hash(story_path, tmp_dir)
        except SystemExit as exc:
            raise AssertionError(
                f"AC-157-003: compute_hash raised SystemExit({exc!r}) for inputs: [] "
                f"— expected hash {EMPTY_HASH!r}. "
                "parse_inputs must short-circuit on inline compact empty inputs."
            ) from None
        assert h == EMPTY_HASH, (
            f"AC-157-003: got {h!r}, expected {EMPTY_HASH!r} for inline compact inputs: []"
        )
        print(f"  [PASS] empty inputs inline compact: hash={h!r} == {EMPTY_HASH!r}")


def test_empty_inputs_multiline_block() -> None:
    """AC-157-004: empty multiline inputs block must return d41d8cd, not SystemExit."""
    EMPTY_HASH = hashlib.md5(b"").hexdigest()[:7]  # d41d8cd
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        content = (
            "---\n"
            "document_type: story\n"
            "story_id: \"STORY-EMPTY-BLOCK\"\n"
            "inputs:\n"
            "input-hash: \"0000000\"\n"
            "---\n"
            "\n"
            "# Test story body\n"
        )
        story_path = tmp_dir / "STORY-EMPTY-BLOCK.md"
        story_path.write_text(content, encoding="utf-8")
        try:
            h = compute_hash(story_path, tmp_dir)
        except SystemExit as exc:
            raise AssertionError(
                f"AC-157-004: compute_hash raised SystemExit({exc!r}) for empty "
                f"multiline inputs block — expected hash {EMPTY_HASH!r}. "
                "parse_inputs must short-circuit on empty multiline block."
            ) from None
        assert h == EMPTY_HASH, (
            f"AC-157-004: got {h!r}, expected {EMPTY_HASH!r} for empty multiline "
            "inputs block"
        )
        print(f"  [PASS] empty inputs multiline block: hash={h!r} == {EMPTY_HASH!r}")


def test_inline_comment_stripped_from_path() -> None:
    """AC-157-010: '  - path  # comment' must strip the comment before file resolution."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        (tmp_dir / "spec.md").write_bytes(b"spec content\n")

        # Reference: same file, no comment in the inputs entry
        story_clean = make_story_md(tmp_dir, ["spec.md"], name="STORY-NO-COMMENT")
        h_ref = compute_hash(story_clean, tmp_dir)

        # Story with inline comment suffix on the input path
        content_with_comment = (
            "---\n"
            "document_type: story\n"
            "story_id: \"STORY-WITH-COMMENT\"\n"
            "inputs:\n"
            "  - spec.md  # RETIRED 2026-06-19: superseded\n"
            "input-hash: \"0000000\"\n"
            "---\n"
            "\n"
            "# Test story body\n"
        )
        story_commented = tmp_dir / "STORY-WITH-COMMENT.md"
        story_commented.write_text(content_with_comment, encoding="utf-8")

        try:
            h_comment = compute_hash(story_commented, tmp_dir)
        except SystemExit as exc:
            raise AssertionError(
                f"AC-157-010: compute_hash raised SystemExit({exc!r}) for path with "
                "inline comment — expected comment stripped and base path resolved. "
                "parse_inputs must strip ' # ...' suffixes from each path entry."
            ) from None
        assert h_comment == h_ref, (
            f"AC-157-010: got {h_comment!r}, expected {h_ref!r} "
            "(comment-stripped path must hash identically to path without comment)"
        )
        print(f"  [PASS] inline comment stripped: {h_comment!r} == ref {h_ref!r}")


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def main() -> None:
    tests = [
        test_determinism,
        test_known_fixture,
        test_crlf_normalization,
        test_lone_cr_normalization,
        test_declaration_order_matters,
        test_missing_input_raises,
        test_empty_inputs_inline_compact,
        test_empty_inputs_multiline_block,
        test_inline_comment_stripped_from_path,
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

    print(f"\n{'='*50}")
    print(f"Results: {passed} passed, {failed} failed")
    if failed:
        sys.exit(1)
    print("All tests passed.")


if __name__ == "__main__":
    main()
