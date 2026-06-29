#!/usr/bin/env python3
"""Self-test for bin/check-green-doc-tense.

Verifies that the gate script:
  1. Flags every known-bad (stale RED-phase) pattern.
  2. Does NOT flag any known-good (legitimate past-tense / contextual) pattern.
  3. Does NOT flag non-comment lines that happen to contain token text.
  4. Exits 0 on a clean fixture, exits 1 on a violating fixture.

Run: python3 bin/test_check_green_doc_tense.py
"""

import importlib.machinery
import importlib.util
import sys
import textwrap
import types
from pathlib import Path

# ---------------------------------------------------------------------------
# Load the module under test
# ---------------------------------------------------------------------------

_SCRIPT = Path(__file__).parent / "check-green-doc-tense"

loader = importlib.machinery.SourceFileLoader("check_green_doc_tense", str(_SCRIPT))
spec = importlib.util.spec_from_loader("check_green_doc_tense", loader)
assert spec is not None
mod: types.ModuleType = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(mod)  # type: ignore[union-attr]

scan_file = mod.scan_file  # type: ignore[attr-defined]
_is_comment = mod._is_comment_line  # type: ignore[attr-defined]

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _tmpfile(content: str, tmp_path: Path, name: str = "fixture.rs") -> Path:
    p = tmp_path / name
    p.write_text(textwrap.dedent(content), encoding="utf-8")
    return p


# ---------------------------------------------------------------------------
# Known-BAD fixtures — each line must be flagged
# ---------------------------------------------------------------------------

BAD_CASES: list[tuple[str, str]] = [
    (
        "module-level MUST FAIL header",
        """\
        //! All tests MUST FAIL (todo!() panic) before implementation — Red Gate per BC-5.38.001.
        """,
    ),
    (
        "ALL tests must fail header (mixed case)",
        """\
        //! RED GATE: ALL tests must fail (todo!() panics) before implementation begins.
        """,
    ),
    (
        "designed to FAIL header",
        """\
        //! All tests in this file are designed to FAIL (Red Gate) until the implementation.
        """,
    ),
    (
        "RED GATE: ... tests must fail (section header)",
        """\
        // RED GATE: all tests must fail (todo!() panics) before implementation begins.
        """,
    ),
    (
        "All stubs panic to satisfy the Red Gate",
        """\
        // All stubs panic to satisfy the Red Gate: every test must FAIL before implementation.
        """,
    ),
    (
        "All test bodies panic — Red Gate",
        """\
        // All test bodies panic — Red Gate (Part A stubs).
        """,
    ),
    (
        "PART A: stub-only bodies — panic",
        """\
        // PART A: stub-only bodies — panic!("STORY-019 stub — Red Gate").
        """,
    ),
    (
        "stub-only bodies — panic — Red Gate",
        """\
        // stub-only bodies — panic!("stub") — Red Gate.
        """,
    ),
    (
        "Every test body panics; all must FAIL before",
        """\
        // Every test body panics; all must FAIL before implementation begins.
        """,
    ),
    (
        "All stubs MUST fail before",
        """\
        // All stubs MUST fail before Part B fills real assertions.
        """,
    ),
    # ------------------------------------------------------------------
    # Patterns 12-17: feature-enip stale RED phrasings (F-135-002)
    # ------------------------------------------------------------------
    (
        "RED — stubs only (em-dash variant)",
        """\
        // STORY-135 command detection tests (RED — stubs only; todo!() enforces Red Gate).
        """,
    ),
    (
        "RED -- stubs only (double-hyphen variant)",
        """\
        // STORY-135 command detection tests (RED -- stubs only; todo!() enforces Red Gate).
        """,
    ),
    (
        "All tests are RED until",
        """\
        // All tests are RED until STORY-135 detection logic is implemented.
        """,
    ),
    (
        "RED (STORY-135 stub) per-test docstring",
        """\
        /// RED (STORY-135 stub): process_pdu reaches todo!() for Stop detection.
        """,
    ),
    (
        "RED (STORY-134 stub) per-test docstring",
        """\
        /// RED (STORY-134 stub): todo!() hit on first write.
        """,
    ),
    (
        "todo!() until STORY-NNN implements",
        """\
        // Red Gate: all tests exercise `process_pdu`, which is `todo!()` until STORY-134 implements detection.
        """,
    ),
    (
        "will panic at … until the implementation lands",
        """\
        // Each test will panic at `process_pdu` until the implementation lands.
        """,
    ),
    (
        "test will panic … until … implements",
        """\
        // Each test will panic at the stub until STORY-134 implements detection.
        """,
    ),
    (
        "Each test will panic at … until (recon-style wrapped header)",
        """\
        // Each test will panic at `process_pdu` until the
        """,
    ),
    # ------------------------------------------------------------------
    # Patterns 19-22: stale GREEN-BY-DESIGN todo!() references (F-135-P3-001)
    # ------------------------------------------------------------------
    (
        "before reaching todo!() Stop-detection block (pattern 19+22)",
        """\
        /// gate fires in the CPF loop before reaching the todo!() Stop-detection block.
        """,
    ),
    (
        "before reaching todo!() Reset block (pattern 19)",
        """\
        /// GREEN-BY-DESIGN: type_id != 0x00B2 gate fires before reaching todo!() Reset block.
        """,
    ),
    (
        "no todo!() is reached — lowercase (pattern 20)",
        """\
        /// only — no todo!() is reached.
        """,
    ),
    (
        "No todo!() is reached — sentence-initial uppercase (pattern 20)",
        """\
        /// No todo!() is reached because the function returns at line 1 of its body.
        """,
    ),
    (
        "before any todo!() block (pattern 21)",
        """\
        /// GREEN-BY-DESIGN: the is_non_enip early-return fires before any todo!() block.
        """,
    ),
    (
        "todo!() Stop-detection block standalone (pattern 22)",
        """\
        /// The test exercises the path before the todo!() Stop-detection block runs.
        """,
    ),
]

# ---------------------------------------------------------------------------
# Known-GOOD fixtures — must NOT be flagged
# ---------------------------------------------------------------------------

GOOD_CASES: list[tuple[str, str]] = [
    (
        "past-tense: passed their Red Gate phase",
        """\
        //! These tests passed their Red Gate phase (all failed before implementation) and are now GREEN.
        """,
    ),
    (
        "past-tense: originally written as a Red Gate",
        """\
        //! Originally written as a Red Gate suite; all tests pass in the GREEN state.
        """,
    ),
    (
        "historical: Red Gate for AC-011",
        """\
        // Red Gate: this test was the genuine Red Gate for AC-011 (harness absent).
        """,
    ),
    (
        "doc-comment: Red Gate assertion (describes test nature)",
        """\
        /// **Red Gate assertion**: after running `wirerust analyze --help`, this test
        """,
    ),
    (
        "statistical description: making this a reliable Red Gate",
        """\
        /// 1/7! = 1/5040 ≈ 0.02%, making this a reliable Red Gate.
        """,
    ),
    (
        "statistical description: deterministic Red Gate in all practical senses",
        """\
        /// this test a deterministic Red Gate in all practical senses.
        """,
    ),
    (
        "inline test assertion: strict must fail with SliceError",
        """\
        // Strict must fail with SliceError::Len.
        """,
    ),
    (
        "inline test assertion: attempt to override initiator — must fail",
        """\
        flow.set_initiator(ip_server, 80); // attempt to override initiator — must fail
        """,
    ),
    (
        "provenance section header: RED-phase:",
        """\
        /// RED-phase: before the SPB arm existed, SPB fell through to the wildcard `_` arm.
        """,
    ),
    (
        "past-tense originated prose",
        """\
        // Originally written as Red Gate stubs (STORY-019); all assertions now GREEN.
        """,
    ),
    (
        "past-tense originated prose (Part A)",
        """\
        // Originated as Red Gate stubs (Part A); all assertions now GREEN.
        """,
    ),
    (
        "non-comment line containing a token (string literal)",
        """\
        let msg = "All tests MUST FAIL before implementation";
        """,
    ),
    (
        "past-tense: Tests originated as Red Gate stubs",
        """\
        //! Tests originated as Red Gate stubs (todo!() panics) before implementation; all now GREEN.
        """,
    ),
    (
        "past-tense: Red Gate commit reference",
        """\
        // On the stub (4e22ef9), this PANICS (Red Gate — test must fail).
        """,
    ),
    # ------------------------------------------------------------------
    # Allowlist cases for patterns 12-17 (must NOT be flagged)
    # ------------------------------------------------------------------
    (
        "past-tense: originated as Red-Gate stubs (not 'stubs only')",
        """\
        // These tests originated as Red-Gate stubs (STORY-135); all assertions now GREEN.
        """,
    ),
    (
        "past-tense: (was RED) parenthetical reference",
        """\
        // Implementation (was RED) is now complete; all tests pass.
        """,
    ),
    (
        "past-tense: tests were RED until (past tense 'were')",
        """\
        // Tests were RED until STORY-135 shipped; all 15 now pass.
        """,
    ),
    (
        "past-tense: tests passed their RED phase",
        """\
        //! Tests passed their RED phase (STORY-135 stub); all assertions now GREEN.
        """,
    ),
    (
        "past-tense: GREEN provenance referencing STORY-NNN stub origin",
        """\
        //! Originated as STORY-135 stub; implementation shipped in STORY-135.
        """,
    ),
    (
        "past-tense: STORY-NNN stub (GREEN) label",
        """\
        /// STORY-135 stub (GREEN): all detections implemented and passing.
        """,
    ),
    (
        "past-tense: todo!() was replaced",
        """\
        // The todo!() was replaced in STORY-135 when detection logic landed.
        """,
    ),
    (
        "past-tense: would panic (conditional, not current-state)",
        """\
        // Each test would panic if the implementation were missing, but STORY-135 is complete.
        """,
    ),
    (
        "past-tense: tests panicked before implementation",
        """\
        // Tests panicked before implementation; all now pass with real assertions.
        """,
    ),
    (
        "past-tense: Each test would have panicked (conditional past, not current-state)",
        """\
        // Each test would have panicked at process_pdu if STORY-134 had not shipped.
        """,
    ),
    (
        "past-tense: Each test panicked before STORY-134 (past tense 'panicked')",
        """\
        // Each test panicked before STORY-134; all 20 now pass.
        """,
    ),
    # ------------------------------------------------------------------
    # Allowlist cases for patterns 19-22 (must NOT be flagged)
    # ------------------------------------------------------------------
    (
        "past-tense: the todo!() was replaced (pattern 19/20 allowlist)",
        """\
        // The todo!() was replaced in STORY-135 when detection logic landed.
        """,
    ),
    (
        "past-tense: originated as todo!() stubs (pattern 19/22 allowlist)",
        """\
        //! Tests originated as todo!() stubs before STORY-135 implemented detection; all now GREEN.
        """,
    ),
    (
        "past-tense: the todo!() Stop-detection block was replaced (pattern 22 allowlist)",
        """\
        // The todo!() Stop-detection block was replaced by the T0858 detection logic in STORY-135.
        """,
    ),
    (
        "past-tense: todo!() was replaced — before reaching phrasing (pattern 19 allowlist)",
        """\
        // Before reaching the now-implemented Stop-detection block, the gate short-circuits.
        """,
    ),
]

# ---------------------------------------------------------------------------
# Test runner
# ---------------------------------------------------------------------------


def run_tests() -> int:
    import tempfile

    failures = 0
    passed = 0

    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)

        print("=== BAD cases (must be flagged) ===")
        for label, content in BAD_CASES:
            p = _tmpfile(content, tmp, f"bad_{passed}.rs")
            violations = scan_file(p)
            if violations:
                print(f"  PASS  [{label}]")
                passed += 1
            else:
                print(f"  FAIL  [{label}] — gate did NOT flag expected violation")
                failures += 1

        print()
        print("=== GOOD cases (must NOT be flagged) ===")
        for label, content in GOOD_CASES:
            p = _tmpfile(content, tmp, f"good_{passed}.rs")
            violations = scan_file(p)
            if not violations:
                print(f"  PASS  [{label}]")
                passed += 1
            else:
                detail = "; ".join(v[2] for v in violations)
                print(f"  FAIL  [{label}] — gate incorrectly flagged: {detail}")
                failures += 1

    print()
    print(f"Results: {passed} passed, {failures} failed.")
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())
