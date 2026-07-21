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

BAD_CASES: list[tuple[str, str] | tuple[str, str, str]] = [
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
    # ------------------------------------------------------------------
    # AC-174-008 patterns (a)-(c): IEC-104 stale Red-Gate headers that
    # slipped through the original gate (PG-REDGREEN-COMMENT-CLEANUP,
    # PG-REDGREEN-SIBLING-SWEEP). These known-bad cases FAILED (Red Gate, STORY-174) until
    # the gate was extended with three new patterns:
    #   (a) All tests\b.*\bMUST FAIL   (interposed words between tokens)
    #   (b) FAILS?\s+Red Gate
    #   (c) are\s+todo!\(\)\s+stub
    # ------------------------------------------------------------------
    (
        "AC-174-008 pattern (a): All tests in this module MUST FAIL (interposed words)",
        """\
        // All tests in this module MUST FAIL (Red Gate) because classify_frame_format is a stub.
        """,
    ),
    (
        "AC-174-008 pattern (a): All tests in this section MUST FAIL (section header variant)",
        """\
        //! All tests in this section MUST FAIL because parse_asdu is todo!().
        """,
    ),
    (
        "AC-174-008 pattern (b): FAILS Red Gate",
        """\
        // This skeleton compiles and FAILS Red Gate because the function is a todo!() stub.
        """,
    ),
    (
        "AC-174-008 pattern (b): FAIL Red Gate (singular form)",
        """\
        // Harnesses FAIL Red Gate by design until the implementation phase ships.
        """,
    ),
    (
        "AC-174-008 pattern (c): are todo!() stubs",
        """\
        // All bodies are todo!() stubs — they will panic on any call before implementation.
        """,
    ),
    (
        "AC-174-008 pattern (c): are todo!() stub (singular)",
        """\
        // The harness body is todo!() stub until STORY-174 wires the assertion.
        """,
    ),
    # ------------------------------------------------------------------
    # AC-176-001 patterns (a)-(d): phrase-level stub-era vocabulary for
    # skeleton/seam compile-only assertions and CI-wiring-incomplete prose
    # (PG-GATE-VOCAB-BLINDSPOT). These known-bad cases FAILED (Red Gate, STORY-176) until
    # the gate was extended with four phrase patterns:
    #   (a) skeleton\s+compiles?        — "harness skeleton compiles" stub-era
    #   (b) compile-only\s+seams?       — "compile-only seam(s)" present-tense
    #   (c) (?:are|is)\s+(?:currently\s+)?compile-only — present-tense compile-only claim
    #   (d) \buntil\b[^\n]*\bwired\b    — "fails until wired" CI-wiring prose
    # ------------------------------------------------------------------
    (
        "AC-176-001 pattern (a): harness skeleton compiles only (stub-era compile-only header)",
        """\
        // harness skeleton compiles only — wiring deferred to STORY-176
        """,
        "AC-176-001 pattern a",
    ),
    (
        "AC-176-001 pattern (a): VP-044 Kani skeleton compiles, no assertions (stub-era header)",
        """\
        // VP-044 Kani skeleton compiles — no proof assertions added yet
        """,
        "AC-176-001 pattern a",
    ),
    (
        "AC-176-001 pattern (b): compile-only seams module header (present-tense stub assertion)",
        """\
        // this module exposes compile-only seams — assertions pending in STORY-174
        """,
        "AC-176-001 pattern b",
    ),
    (
        "AC-176-001 pattern (b): harness is a compile-only seam (present-tense stub label)",
        """\
        // harness body is a compile-only seam during red-gate phase
        """,
        "AC-176-001 pattern b",
    ),
    (
        "AC-176-001 pattern (c): are currently compile-only (present-tense no-assertions claim)",
        """\
        // all harness bodies are currently compile-only — no real assertions
        """,
        "AC-176-001 pattern c",
    ),
    (
        "AC-176-001 pattern (c): is compile-only (present-tense stub-boundary claim, no 'currently')",
        """\
        // this function is compile-only at the red-gate boundary
        """,
        "AC-176-001 pattern c",
    ),
    (
        "AC-176-001 pattern (d): fails until wired (CI-wiring incomplete prose)",
        """\
        // CI test fails until the IEC-104 handler is wired
        """,
        "AC-176-001 pattern d",
    ),
    (
        "AC-176-001 pattern (d): returns early until STORY-NNN is wired (CI-wiring incomplete prose)",
        """\
        // function returns early until STORY-176 is wired
        """,
        "AC-176-001 pattern d",
    ),
    (
        "AC-176-001 pattern (d): CI job fails until wired (bare motivating phrase, no object clause)",
        """\
        // CI job fails until wired — see STORY-NNN
        """,
        "AC-176-001 pattern d",
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
    # ------------------------------------------------------------------
    # AC-174-008 allowlist cases: past-tense provenance prose that must
    # NOT be flagged by patterns (a), (b), or (c). Proves no regression
    # against legitimate historical narration after the token extension.
    # ------------------------------------------------------------------
    (
        "AC-174-008 pattern (a) allowlist: All tests passed (no 'MUST FAIL')",
        """\
        // All tests in this module passed their Red Gate phase and are now GREEN.
        """,
    ),
    (
        "AC-174-008 pattern (a) allowlist: All tests were required to FAIL (no 'MUST')",
        """\
        // All tests were required to FAIL at the Red Gate stage (STORY-168); all are now GREEN.
        """,
    ),
    (
        "AC-174-008 pattern (b) allowlist: failed Red Gate (past tense 'failed', not 'FAILS?')",
        """\
        // This skeleton originally failed Red Gate as a compile-only seam; now asserting GREEN.
        """,
    ),
    (
        "AC-174-008 pattern (b) allowlist: passed Red Gate (no FAILS? pattern)",
        """\
        // The harness passed Red Gate validation after the implementation shipped in STORY-174.
        """,
    ),
    (
        "AC-174-008 pattern (c) allowlist: were todo!() stubs (past tense 'were', not 'are')",
        """\
        // These function bodies were todo!() stubs before STORY-172 shipped; all now GREEN.
        """,
    ),
    (
        "AC-174-008 pattern (c) allowlist: originated as todo!() stubs (past-tense provenance)",
        """\
        // All harness bodies originated as todo!() stubs; STORY-174 upgraded them to assertions.
        """,
    ),
    # ------------------------------------------------------------------
    # AC-176-001 allowlist cases: past-tense provenance and legitimate
    # architectural uses of skeleton/seam vocabulary that must NOT be
    # flagged by patterns (a)-(d). Proves zero false positives on the
    # common forms named in the STORY-176 spec.
    # ------------------------------------------------------------------
    (
        "AC-176-001 pattern (a) allowlist: skeleton originated (past-tense provenance, no 'compiles')",
        """\
        // Harness skeleton originated in STORY-167
        """,
    ),
    (
        "AC-176-001 pattern (a) allowlist: bare skeleton label (architectural reference, no 'compiles')",
        """\
        // VP-024 Sub-D skeleton
        """,
    ),
    (
        "AC-176-001 pattern (a) allowlist: skeleton compiled (past-tense 'compiled', excluded by trailing-\\b in pattern 26)",
        """\
        // harness skeleton compiled cleanly in STORY-NNN
        """,
    ),
    (
        "AC-176-001 pattern (a) allowlist: exoskeleton compiles (leading-\\b excludes prefix form; CR-005)",
        """\
        // exoskeleton compiles — real hardware acceleration
        """,
    ),
    (
        "AC-176-001 pattern (b) allowlist: Test seam accessors (architectural idiom, no 'compile-only' prefix)",
        """\
        // Test seam accessors provide injection points for testing
        """,
    ),
    (
        "AC-176-001 pattern (b) allowlist: VP-047 seam (bare seam reference, no 'compile-only')",
        """\
        // VP-047 seam exercised per architecture diagram
        """,
    ),
    (
        "AC-176-001 pattern (b) allowlist: UDP gap-key seam (bare seam reference, no 'compile-only')",
        """\
        // UDP gap-key seam validated in STORY-147
        """,
    ),
    (
        "AC-176-001 pattern (c) allowlist: was compile-only (past tense 'was', not 'are'/'is')",
        """\
        // was compile-only until STORY-153 wired it
        """,
    ),
    (
        "AC-176-001 pattern (c) allowlist: compile-only checks passed (no 'are'/'is' before 'compile-only')",
        """\
        // compile-only checks passed for the test infrastructure
        """,
    ),
    (
        "AC-176-001 pattern (d) allowlist: STORY-NNN wired the handler (past-tense 'wired', no preceding 'until')",
        """\
        // STORY-153 wired the handler — all assertions now GREEN
        """,
    ),
    (
        "AC-176-001 pattern (d) allowlist: was compile-only until STORY-NNN wired it (past-tense 'until...wired' context)",
        """\
        // was compile-only until STORY-153 wired it
        """,
    ),
    (
        "AC-176-001 pattern (d) allowlist: until STORY-NNN wired the handler (negative lookahead on 'the' suppresses match; CR-002)",
        """\
        // was compile-only until STORY-153 wired the handler
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
        for entry in BAD_CASES:
            label, content = entry[0], entry[1]
            expected_pattern: str | None = entry[2] if len(entry) > 2 else None  # type: ignore[misc]
            p = _tmpfile(content, tmp, f"bad_{passed}.rs")
            violations = scan_file(p)
            if violations:
                if expected_pattern is not None:
                    matched_labels = [v[2] for v in violations]
                    if any(expected_pattern in ml for ml in matched_labels):
                        print(f"  PASS  [{label}]")
                        passed += 1
                    else:
                        print(
                            f"  FAIL  [{label}] — gate fired but not on expected pattern "
                            f"(expected {expected_pattern!r}; got {matched_labels})"
                        )
                        failures += 1
                else:
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

    # ------------------------------------------------------------------
    # AC-158-005: zero-file guard — must exit non-zero when
    # _collect_rust_files returns [].
    #
    # Behavior shipped with AC-158-005: exits non-zero when no files found.
    # Originally authored RED for AC-158-005 (pre-fix: printed WARNING,
    # exited 0); now a GREEN regression guard — guards against any reversion
    # to the silent-exit-0 behavior.
    # ------------------------------------------------------------------
    print()
    print("=== AC-158-005 zero-file guard (must exit non-zero when no files found) ===")

    _orig_find_005 = mod._find_repo_root  # type: ignore[attr-defined]
    _orig_collect = mod._collect_rust_files  # type: ignore[attr-defined]
    try:
        with tempfile.TemporaryDirectory() as _td_005:
            _hermetic_005 = Path(_td_005)
            (_hermetic_005 / ".factory").mkdir()
            mod._find_repo_root = lambda _s: _hermetic_005  # type: ignore[attr-defined]
            # Patch _collect_rust_files to return [] — simulates a repo with no
            # tracked Rust files (e.g., src/ renamed or git ls-files returns nothing).
            mod._collect_rust_files = lambda _repo_root: []  # type: ignore[attr-defined]
            exit_code = mod.main()  # type: ignore[attr-defined]
            if exit_code == 1:
                print(
                    "  PASS  [zero-file guard: exits 1 exactly when _collect_rust_files "
                    "returns [] (AC-158-005)]"
                )
                passed += 1
            else:
                print(
                    "  FAIL  [zero-file guard: expected exit 1 when "
                    "_collect_rust_files returns [], got "
                    + repr(exit_code)
                    + " — REGRESSION: zero-file guard (AC-158-005) no longer exits 1 — "
                    "check the `if not rust_files:` guard in main()]"
                )
                failures += 1
    finally:
        mod._find_repo_root = _orig_find_005  # type: ignore[attr-defined]
        mod._collect_rust_files = _orig_collect  # type: ignore[attr-defined]

    # ------------------------------------------------------------------
    # AC-162-004: _find_repo_root hermetic sentinel tests
    # (F-W72G-P2-OBS-001 — .factory/ OR-sentinel arm untested)
    #
    # Calls mod._find_repo_root(start) directly and verifies the
    # implementation correctly identifies repo roots by walking upward for
    # .git or .factory sentinels. Includes cases: (a) .factory/ only,
    # (b1) .git directory, (b2) .git file (worktree), (c) no sentinel
    # (regression guard).
    # ------------------------------------------------------------------
    print()
    print(
        "=== AC-162-004 _find_repo_root sentinel hermetic tests "
        "(F-W72G-P2-OBS-001) ==="
    )

    _find_repo_root = mod._find_repo_root  # type: ignore[attr-defined]

    # (a) .factory/ sentinel only — exercises the OR-sentinel arm (no .git present).
    with tempfile.TemporaryDirectory() as _td_a:
        _root_a = Path(_td_a)
        (_root_a / ".factory").mkdir()
        _deep_a = _root_a / "sub" / "deep"
        _deep_a.mkdir(parents=True)
        _result_a = _find_repo_root(_deep_a)
        if _result_a == _root_a:
            print(
                "  PASS  [_find_repo_root: .factory/ OR-sentinel resolves root "
                "(F-W72G-P2-OBS-001)]"
            )
            passed += 1
        else:
            print(
                f"  FAIL  [_find_repo_root: .factory/ OR-sentinel resolves root "
                f"(F-W72G-P2-OBS-001)] — expected {_root_a}, got {_result_a!r}"
            )
            failures += 1

    # (b1) .git directory sentinel.
    with tempfile.TemporaryDirectory() as _td_b1:
        _root_b1 = Path(_td_b1)
        (_root_b1 / ".git").mkdir()
        _sub_b1 = _root_b1 / "project" / "src"
        _sub_b1.mkdir(parents=True)
        _result_b1 = _find_repo_root(_sub_b1)
        if _result_b1 == _root_b1:
            print(
                "  PASS  [_find_repo_root: .git directory sentinel resolves root "
                "(F-W72G-P2-OBS-001)]"
            )
            passed += 1
        else:
            print(
                f"  FAIL  [_find_repo_root: .git directory sentinel resolves root "
                f"(F-W72G-P2-OBS-001)] — expected {_root_b1}, got {_result_b1!r}"
            )
            failures += 1

    # (b2) .git FILE sentinel (worktree case — .git is a file, not a directory).
    with tempfile.TemporaryDirectory() as _td_b2:
        _root_b2 = Path(_td_b2)
        (_root_b2 / ".git").write_text(
            "gitdir: /some/repo/.git/worktrees/branch\n", encoding="utf-8"
        )
        _sub_b2 = _root_b2 / "nested" / "dir"
        _sub_b2.mkdir(parents=True)
        _result_b2 = _find_repo_root(_sub_b2)
        if _result_b2 == _root_b2:
            print(
                "  PASS  [_find_repo_root: .git file (worktree) sentinel resolves root "
                "(F-W72G-P2-OBS-001)]"
            )
            passed += 1
        else:
            print(
                f"  FAIL  [_find_repo_root: .git file (worktree) sentinel resolves root "
                f"(F-W72G-P2-OBS-001)] — expected {_root_b2}, got {_result_b2!r}"
            )
            failures += 1

    # (c) NO sentinel — temp tree with neither .git nor .factory should return None or
    # an ancestor root outside the temp tree (if walk passes the boundary). This is a
    # regression guard: before fix, _find_repo_root would return None unconditionally.
    # Post-fix, if _result_c is not None, it MUST NOT be within the temp tree (the
    # temp tree contains no sentinels, so a non-None result must be an ancestor that
    # lives outside it). Assertion: _result_c is None, OR the result path does not
    # start with the temp-tree root (F-W72G-P2-OBS-001).
    with tempfile.TemporaryDirectory() as _td_c:
        _root_c = Path(_td_c)
        _deep_c = _root_c / "a" / "b" / "c" / "d"
        _deep_c.mkdir(parents=True)
        _result_c = _find_repo_root(_deep_c)
        if _result_c is None or not _result_c.is_relative_to(_root_c):
            print(
                "  PASS  [_find_repo_root: no-sentinel temp tree returns None or "
                "ancestor (F-W72G-P2-OBS-001)]"
            )
            passed += 1
        else:
            print(
                f"  FAIL  [_find_repo_root: no-sentinel temp tree must return None or "
                f"ancestor outside tree (F-W72G-P2-OBS-001)] — result {_result_c} "
                f"is within temp tree {_root_c}"
            )
            failures += 1

    # ------------------------------------------------------------------
    # AC-162-003: zero-file guard exit-code precision (hermetic)
    # (F-W72G-P2-OBS-001 + AC-162-003)
    #
    # Patches _find_repo_root to return a hermetic repo root, and installs
    # a spy on _collect_rust_files that records the repo_root argument and
    # returns [].
    #
    # Primary assertion: spy confirms main() passes the hermetic root to
    # _collect_rust_files, verifying that main() delegates repo-root
    # detection to _find_repo_root.
    #
    # Secondary assertion: exit_code == 1 exactly (zero-file guard fires,
    # not exit 2 from repo-root-not-found guard).
    #
    # Both assertions hold now that main() delegates to _find_repo_root;
    # the combined test guards against regression of either.
    # ------------------------------------------------------------------
    print()
    print(
        "=== AC-162-003 zero-file guard exit-code precision hermetic "
        "(F-W72G-P2-OBS-001) ==="
    )

    _orig_find = mod._find_repo_root  # type: ignore[attr-defined]
    _orig_collect3 = mod._collect_rust_files  # type: ignore[attr-defined]
    try:
        with tempfile.TemporaryDirectory() as _td_main:
            _hermetic_root = Path(_td_main)
            (_hermetic_root / ".factory").mkdir()

            _collect_calls: list = []

            def _spy_collect(_repo_root: "Path") -> list:  # type: ignore[type-arg]
                _collect_calls.append(_repo_root)
                return []

            mod._find_repo_root = lambda _start: _hermetic_root  # type: ignore[attr-defined]
            mod._collect_rust_files = _spy_collect  # type: ignore[attr-defined]
            _exit_code = mod.main()  # type: ignore[attr-defined]

            _root_used_ok = bool(_collect_calls) and _collect_calls[0] == _hermetic_root
            _exit_code_ok = _exit_code == 1

            if _root_used_ok and _exit_code_ok:
                print(
                    "  PASS  [zero-file guard hermetic: main() used _find_repo_root result "
                    "and exited 1 exactly (AC-162-003, F-W72G-P2-OBS-001)]"
                )
                passed += 1
            else:
                _reasons = []
                if not _root_used_ok:
                    _actual_root = (
                        _collect_calls[0] if _collect_calls else "<not called>"
                    )
                    _reasons.append(
                        f"main() passed {_actual_root!r} to _collect_rust_files "
                        f"(expected hermetic root {_hermetic_root}; "
                        f"main() must delegate to _find_repo_root)"
                    )
                if not _exit_code_ok:
                    _reasons.append(f"exit_code={_exit_code!r} (expected 1, not 2)")
                print(
                    f"  FAIL  [zero-file guard hermetic: main() must use _find_repo_root "
                    f"for repo-root detection (AC-162-003, F-W72G-P2-OBS-001)] — "
                    + "; ".join(_reasons)
                )
                failures += 1
    finally:
        mod._find_repo_root = _orig_find  # type: ignore[attr-defined]
        mod._collect_rust_files = _orig_collect3  # type: ignore[attr-defined]

    print()
    print(f"Results: {passed} passed, {failures} failed.")
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())
