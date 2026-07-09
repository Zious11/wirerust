#!/usr/bin/env python3
"""
Self-test for bin/lint-cycle-artifact — RED GATE version.

TC1–TC8 implement all eight test cases from STORY-158 Task 3 / AC-158-003.
All tests MUST FAIL until bin/lint-cycle-artifact is created.

Hermetic fixtures: each TC constructs its own fixture tree under
tempfile.TemporaryDirectory() and passes the root via WIRERUST_REPO_ROOT.
No live .factory/ files are referenced (CI-safe on develop checkouts).

Evaluation-order DAG respected in fixture design:
  (1) frontmatter presence → (6) path/story_id identity →
  (2) empty-bcs short-circuit [exit 0] → (3) BC existence on disk →
  (7) story ownership

Run: python3 bin/test_lint_cycle_artifact.py
"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path

BIN_DIR = Path(__file__).resolve().parent
TOOL = BIN_DIR / "lint-cycle-artifact"

# Exact error messages from AC-158-003 (ASCII -- not em-dash; v1.13 canonical form)
_ERR_MISSING_FRONTMATTER = (
    "ERROR: artifact lacks required frontmatter (story_id: and bcs: fields) "
    "-- see current cycle-artifact template (STORY-158)"
)
_ERR_INVALID_PATH = (
    "ERROR: artifact path does not match expected "
    ".factory/cycles/<wave>/STORY-NNN/<artifact> pattern "
    "-- cannot derive expected story_id"
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def run_tool(artifact_path: Path, repo_root: Path) -> subprocess.CompletedProcess:
    """Run lint-cycle-artifact <artifact-path> with WIRERUST_REPO_ROOT set."""
    env = os.environ.copy()
    env["WIRERUST_REPO_ROOT"] = str(repo_root)
    return subprocess.run(
        [sys.executable, str(TOOL), str(artifact_path)],
        capture_output=True,
        text=True,
        env=env,
    )


def make_artifact(tmp_dir: Path, rel_path: str, content: str) -> Path:
    """Write an artifact file at tmp_dir/rel_path, creating intermediate dirs."""
    artifact_path = tmp_dir / rel_path
    artifact_path.parent.mkdir(parents=True, exist_ok=True)
    artifact_path.write_text(content, encoding="utf-8")
    return artifact_path


def make_story(
    tmp_dir: Path,
    story_id: str,
    behavioral_contracts: list[str],
) -> Path:
    """Write a minimal parent story file under .factory/stories/."""
    if behavioral_contracts:
        bcs_block = "behavioral_contracts:\n" + "".join(
            f"  - {bc}\n" for bc in behavioral_contracts
        )
    else:
        bcs_block = "behavioral_contracts: []\n"
    content = (
        "---\n"
        f"story_id: {story_id}\n"
        f"{bcs_block}"
        "---\n"
        "\n"
        f"# {story_id} stub\n"
    )
    story_path = tmp_dir / ".factory" / "stories" / f"{story_id}.md"
    story_path.parent.mkdir(parents=True, exist_ok=True)
    story_path.write_text(content, encoding="utf-8")
    return story_path


def make_bc_file(tmp_dir: Path, bc_id: str) -> Path:
    """
    Write a stub BC file at the canonical on-disk path.

    BC-S.SS.NNN → .factory/specs/behavioral-contracts/ss-SS/BC-S.SS.NNN.md
    e.g. BC-2.11.036 → .factory/specs/behavioral-contracts/ss-11/BC-2.11.036.md
    """
    # Extract subsection: "BC-2.11.036" → body="2.11.036" → parts=["2","11","036"] → ss="11"
    body = bc_id.split("-", 1)[1]
    parts = body.split(".")
    ss = parts[1]
    bc_path = (
        tmp_dir / ".factory" / "specs" / "behavioral-contracts"
        / f"ss-{ss}" / f"{bc_id}.md"
    )
    bc_path.parent.mkdir(parents=True, exist_ok=True)
    bc_path.write_text(f"# {bc_id}\n\nStub BC file for hermetic test fixture.\n", encoding="utf-8")
    return bc_path


# ---------------------------------------------------------------------------
# Test cases (TC1–TC8)
# ---------------------------------------------------------------------------


def test_tc1_missing_frontmatter() -> None:
    """
    TC1: artifact with no YAML frontmatter block → exit non-zero, exact rule-1 error message.

    Fixture: artifact at a well-formed path but no --- block at all.
    Tests evaluation-order rule (1): missing frontmatter fires before any other rule.
    AC-158-003 rule (1) error string uses ASCII -- (v1.13, F-W72-P11-L01 / F-W72-P12-001).
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            "# FINDINGS\n\nNo frontmatter here.\n",
        )
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC1: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert _ERR_MISSING_FRONTMATTER in combined, (
            f"TC1: exact rule-1 error message not found in output.\n"
            f"Expected: {_ERR_MISSING_FRONTMATTER!r}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC1: missing frontmatter → exit {result.returncode}, "
            "exact rule-1 error message present"
        )


def test_tc2_empty_bcs_correct_path() -> None:
    """
    TC2: valid story_id + bcs: [] at correct path → exit 0.

    Fixture: artifact at .factory/cycles/wave-72/STORY-158/artifact.md with
    story_id: STORY-158 and bcs: []. No parent story file created.
    Tests rule (6) passes first (path matches), then rule (2) short-circuits before
    rule (7) — parent-story existence is NOT required for empty-bcs artifacts.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/artifact.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs: []\n"
                "---\n"
                "\n"
                "# Artifact body\n"
            ),
        )
        # Deliberately no parent story file — rule (2) short-circuits before rule (7)
        result = run_tool(artifact, tmp_dir)
        assert result.returncode == 0, (
            f"TC2: expected exit 0, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        # F-S158-P2-004: rule (2) must emit a PASS message (not silent exit 0)
        assert "PASS:" in result.stdout, (
            f"TC2: expected PASS message on stdout (F-S158-P2-004 — rule 2 empty-bcs "
            f"short-circuit must not exit silently).\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        assert "empty bcs" in result.stdout, (
            f"TC2: PASS message must reference 'empty bcs' to be self-documenting.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(f"  [PASS] TC2: empty bcs: at correct path → exit 0, PASS message emitted")


def test_tc3_unresolvable_bc_id() -> None:
    """
    TC3: bcs: contains a fabricated BC ID (no on-disk file) → exit non-zero, ID listed.

    Fixture: artifact at correct path with story_id: STORY-158 and bcs: [BC-9.99.999].
    No BC file on disk. No parent story file needed (rule 3 fires before rule 7).
    Tests rule (3): every BC ID in bcs: must resolve to an on-disk file.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs:\n"
                "  - BC-9.99.999\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        # No BC file created → BC-9.99.999 is fabricated
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC3: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert "BC-9.99.999" in combined, (
            f"TC3: expected unresolvable ID 'BC-9.99.999' listed in output.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(f"  [PASS] TC3: unresolvable BC ID → exit {result.returncode}, ID listed")


def test_tc4_prose_bc_id_not_flagged() -> None:
    """
    TC4: BC ID in body prose only (bcs: []) → exit 0; prose is not checked.

    Fixture: artifact at correct path with story_id: STORY-158 and bcs: [].
    A fabricated BC ID (BC-9.99.999) appears only in body prose — not in bcs: field.
    No BC file on disk. Tests rule (4): only bcs: frontmatter is linted, not body prose.
    Rule (2) short-circuits on bcs: [] before any prose scan could occur.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/artifact.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs: []\n"
                "---\n"
                "\n"
                "# Findings\n"
                "\n"
                "See BC-9.99.999 for details — referenced in prose only, not in bcs:.\n"
            ),
        )
        # No BC file created, but the ID is only in body prose, not in bcs:
        result = run_tool(artifact, tmp_dir)
        assert result.returncode == 0, (
            f"TC4: expected exit 0 (prose BC IDs not checked), "
            f"got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(f"  [PASS] TC4: prose-only BC ID → exit 0 (not flagged)")


def test_tc5_missing_bcs_key() -> None:
    """
    TC5: frontmatter present with story_id: but missing bcs: key entirely → rule (1) HARD FAIL.

    Fixture: artifact with YAML frontmatter that has story_id: but no bcs: key.
    Tests that a missing bcs: key triggers rule (1) (same error as missing frontmatter).
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "title: Some findings\n"
                "# bcs: key deliberately absent\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC5: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert _ERR_MISSING_FRONTMATTER in combined, (
            f"TC5: expected exact rule-1 error message not found.\n"
            f"Expected: {_ERR_MISSING_FRONTMATTER!r}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(f"  [PASS] TC5: missing bcs: key → exit {result.returncode}, rule-1 error")


def test_tc6_story_id_directory_mismatch() -> None:
    """
    TC6: artifact at wave-72/STORY-158/FINDINGS.md with story_id: STORY-999 → rule (6) HARD FAIL.

    Fixture: artifact at .factory/cycles/wave-72/STORY-158/FINDINGS.md but declares
    story_id: STORY-999. Tests rule (6) branch-(b): declared story_id does not match
    directory-derived value. Error must name both declared (STORY-999) and derived (STORY-158).
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-999\n"
                "bcs: []\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC6: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert "STORY-999" in combined, (
            f"TC6: expected declared value 'STORY-999' in error output.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        assert "STORY-158" in combined, (
            f"TC6: expected directory-derived 'STORY-158' in error output.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC6: story_id mismatch → exit {result.returncode}, "
            "both STORY-999 and STORY-158 named"
        )


def test_tc7_borrowed_bc_id() -> None:
    """
    TC7: bcs: [BC-2.11.036] where parent story STORY-158 has behavioral_contracts: [] →
    rule (7) HARD FAIL listing BC-2.11.036 as unowned (borrowed from a different story).

    Fixture:
      - Artifact at correct path with story_id: STORY-158 (rule 6 passes)
      - BC-2.11.036 file exists on disk (rule 3 passes)
      - .factory/stories/STORY-158.md has behavioral_contracts: [] (rule 7 fires)

    Tests rule (7): an artifact may NOT claim BCs that the parent story does not own.
    Path: .factory/specs/behavioral-contracts/ss-11/BC-2.11.036.md
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs:\n"
                "  - BC-2.11.036\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        make_bc_file(tmp_dir, "BC-2.11.036")
        make_story(tmp_dir, "STORY-158", behavioral_contracts=[])

        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC7: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert "BC-2.11.036" in combined, (
            f"TC7: expected unowned ID 'BC-2.11.036' listed in error output.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(f"  [PASS] TC7: borrowed BC ID → exit {result.returncode}, BC-2.11.036 listed")


def test_tc8_no_wave_intermediate() -> None:
    """
    TC8: artifact at .factory/cycles/STORY-158/x.md (no wave-NNN intermediate) →
    rule (6) branch-(a) invalid-path HARD FAIL.

    Fixture: artifact at .factory/cycles/STORY-158/x.md — the path has no wave-[0-9]+
    directory between .factory/cycles/ and STORY-158/, violating the required pattern.
    Tests rule (6) branch-(a): wave-NNN intermediate is required; its absence triggers
    the invalid-path error before any story_id derivation occurs.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/STORY-158/x.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs: []\n"
                "---\n"
                "\n"
                "# x\n"
            ),
        )
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC8: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert _ERR_INVALID_PATH in combined, (
            f"TC8: expected exact invalid-path error message not found.\n"
            f"Expected: {_ERR_INVALID_PATH!r}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC8: no wave-NNN intermediate → exit {result.returncode}, "
            "exact invalid-path error"
        )


def test_tc9_comment_interleaved_bcs() -> None:
    """
    TC9 (F-S158-P1-005): bcs: block list with an interleaved YAML comment line.

    The post-comment BC ID (BC-9.99.999) MUST be included in the parsed bcs: list
    and reach rule (3). Before the fix, the block-list parser broke on the comment
    line, silently dropping BC-9.99.999 — a false-PASS vector (escaped rules 3+7).

    Fixture:
      - Artifact at correct path with story_id: STORY-158
      - bcs: block list: BC-2.11.036, # comment, BC-9.99.999
      - BC-2.11.036 exists on disk (rule 3 passes for it)
      - BC-9.99.999 NOT on disk → unresolvable
    Expected: exit 1, BC-9.99.999 listed (proves it was not dropped by the parser).
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs:\n"
                "  - BC-2.11.036\n"
                "  # this BC exists on disk; the next one does not\n"
                "  - BC-9.99.999\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        make_bc_file(tmp_dir, "BC-2.11.036")
        # BC-9.99.999 deliberately NOT created → unresolvable
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC9: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert "BC-9.99.999" in combined, (
            f"TC9: BC-9.99.999 (post-comment item) must appear in error output — "
            f"parser must not drop items after comment lines.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC9: comment-interleaved bcs: → exit {result.returncode}, "
            "post-comment BC-9.99.999 listed (not dropped)"
        )


def test_tc10_inline_comment_suffix_stripped() -> None:
    """
    TC10 (F-S158-P1-002): story_id: and bcs: values carry inline comment suffixes.

    story_id: STORY-158  # note → must parse as STORY-158 (suffix stripped).
    - BC-2.11.036  # ok  → must parse as BC-2.11.036 (suffix stripped).

    Parity with bin/compute-input-hash's documented convention (CLAUDE.md
    "Inline comment suffixes").

    Fixture:
      - Artifact with story_id: STORY-158  # note and block bcs: [BC-2.11.036  # ok]
      - BC-2.11.036 exists on disk
      - Parent story STORY-158 owns BC-2.11.036
    Expected: exit 0 (inline comment suffixes stripped, all rules pass).
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-158  # note\n"
                "bcs:\n"
                "  - BC-2.11.036  # ok\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        make_bc_file(tmp_dir, "BC-2.11.036")
        make_story(tmp_dir, "STORY-158", behavioral_contracts=["BC-2.11.036"])
        result = run_tool(artifact, tmp_dir)
        assert result.returncode == 0, (
            f"TC10: expected exit 0 (inline comment suffixes stripped), "
            f"got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(f"  [PASS] TC10: inline comment suffixes stripped → exit 0")


def test_tc11_no_factory_cycles_ancestor() -> None:
    """
    TC11 (F-S158-P1-001): wave-NNN/STORY-NNN path without .factory/cycles/ ancestor.

    Path: <tmp>/wave-72/STORY-158/x.md — has wave-NNN parent for STORY-NNN
    but lacks the required .factory/cycles/ ancestor components.

    Rule (6) branch-(a) must fire: .factory/cycles/ is required above wave-NNN.
    Without this check, /tmp/wave-72/STORY-158/x.md would have silently passed
    rule (6) (wave-NNN parent found) and exited 0 on empty bcs:.

    Distinguishes from TC8 (.factory/cycles/STORY-NNN — missing wave-NNN):
    TC11 has wave-NNN but no .factory/cycles/ above it.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        # Artifact at wave-72/STORY-158/x.md — no .factory/cycles/ prefix
        artifact = make_artifact(
            tmp_dir,
            "wave-72/STORY-158/x.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs: []\n"
                "---\n"
                "\n"
                "# x\n"
            ),
        )
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC11: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert _ERR_INVALID_PATH in combined, (
            f"TC11: expected exact invalid-path error message not found.\n"
            f"Expected: {_ERR_INVALID_PATH!r}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC11: no .factory/cycles/ ancestor → exit {result.returncode}, "
            "exact invalid-path error"
        )


def test_tc12_blank_line_interleaved_bcs() -> None:
    """
    TC12 (F-S158-P2-001): A16b fixture — bcs: block list with an interleaved blank line.

    YAML treats blank lines within a block sequence as insignificant (they do not
    terminate the sequence). Before the structural fix, the blank line caused a break,
    silently dropping BC-99.99.999 — a false-PASS vector identical to the comment-line
    vector fixed in F-S158-P1-005. The structural fix kills both by unifying blank and
    comment lines into a single skip-and-continue case.

    Fixture (A16b): BC-2.11.036 (exists), blank line, BC-99.99.999 (unresolvable).
    Expected: exit 1, BC-99.99.999 listed (proves post-blank item was not dropped).
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "bcs:\n"
                "  - BC-2.11.036\n"
                "\n"
                "  - BC-99.99.999\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        make_bc_file(tmp_dir, "BC-2.11.036")
        # BC-99.99.999 deliberately NOT created → unresolvable
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC12: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert "BC-99.99.999" in combined, (
            f"TC12: BC-99.99.999 (post-blank item) must appear in error output — "
            f"parser must not drop items after blank lines.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC12: blank-line-interleaved bcs: → exit {result.returncode}, "
            "post-blank BC-99.99.999 listed (not dropped)"
        )


def test_tc13_non_utf8_artifact() -> None:
    """
    TC13 (F-S158-P2-002): artifact file contains non-UTF-8 bytes.

    UnicodeDecodeError inherits ValueError, not OSError — it escaped the original
    `except OSError` guard and produced a raw Python traceback on stderr.
    Expected: exit non-zero with a controlled ERROR message; no raw traceback.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact_path = (
            tmp_dir / ".factory" / "cycles" / "wave-72" / "STORY-158" / "FINDINGS.md"
        )
        artifact_path.parent.mkdir(parents=True, exist_ok=True)
        # Write bytes that are invalid UTF-8 (0xFF 0xFE are not valid UTF-8 sequences)
        artifact_path.write_bytes(
            b"---\nstory_id: STORY-158\nbcs: []\n---\n\xff\xfe invalid utf-8 here"
        )
        result = run_tool(artifact_path, tmp_dir)
        assert result.returncode != 0, (
            f"TC13: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert "ERROR:" in combined, (
            f"TC13: expected controlled ERROR message in output.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        assert "Traceback" not in combined, (
            f"TC13: raw Python traceback must not appear in output.\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC13: non-UTF-8 artifact → exit {result.returncode}, "
            "controlled ERROR message (no traceback)"
        )


def test_tc14_duplicate_story_id_key() -> None:
    """
    TC14 (F-S158-P2-003): frontmatter contains duplicate story_id: keys.

    YAML processors last-win on duplicate keys. An identity tool must reject
    ambiguous identity declarations rather than silently picking one value.

    Expected: exit non-zero with exact
    "ERROR: duplicate story_id: key in frontmatter -- ambiguous identity declaration".
    """
    _ERR_DUPLICATE = (
        "ERROR: duplicate story_id: key in frontmatter "
        "-- ambiguous identity declaration"
    )
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        artifact = make_artifact(
            tmp_dir,
            ".factory/cycles/wave-72/STORY-158/FINDINGS.md",
            (
                "---\n"
                "story_id: STORY-158\n"
                "story_id: STORY-999\n"
                "bcs: []\n"
                "---\n"
                "\n"
                "# Findings\n"
            ),
        )
        result = run_tool(artifact, tmp_dir)
        assert result.returncode != 0, (
            f"TC14: expected exit non-zero, got returncode={result.returncode}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        combined = result.stdout + result.stderr
        assert _ERR_DUPLICATE in combined, (
            f"TC14: expected exact duplicate-key error message not found.\n"
            f"Expected: {_ERR_DUPLICATE!r}\n"
            f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
        )
        print(
            f"  [PASS] TC14: duplicate story_id: key → exit {result.returncode}, "
            "exact duplicate-key error"
        )


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------


def main() -> None:
    tests = [
        test_tc1_missing_frontmatter,
        test_tc2_empty_bcs_correct_path,
        test_tc3_unresolvable_bc_id,
        test_tc4_prose_bc_id_not_flagged,
        test_tc5_missing_bcs_key,
        test_tc6_story_id_directory_mismatch,
        test_tc7_borrowed_bc_id,
        test_tc8_no_wave_intermediate,
        test_tc9_comment_interleaved_bcs,
        test_tc10_inline_comment_suffix_stripped,
        test_tc11_no_factory_cycles_ancestor,
        test_tc12_blank_line_interleaved_bcs,
        test_tc13_non_utf8_artifact,
        test_tc14_duplicate_story_id_key,
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
