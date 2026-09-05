---
document_type: story
story_id: STORY-183
epic_id: E-11
version: "2.13"
status: ready
producer: story-writer
timestamp: 2026-07-25T00:00:00Z
phase: f7
level: feature
cycle: wave-086
points: 5
priority: P2
depends_on: []
blocks: []
# BC status: E-11 convention — governance-only story; no BCs authored
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: bin/
subsystems: []
estimated_days: 2
wave: "86"
traces_to:
  - .factory/cycles/wave-084/lessons.md
  - .factory/cycles/wave-085/lessons.md
  - .factory/cycles/wave-085/STORY-180/convergence-report.md
  - .factory/planning/df-validation-2026-07-25.md
  - bin/check-green-doc-tense
  - bin/test_check_green_doc_tense.py
  - CHANGELOG.md
  - .github/workflows/ci.yml
  - bin/test_lint_cycle_artifact.py
inputs:
  - .factory/cycles/wave-084/lessons.md
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/df-validation-2026-07-25.md
input-hash: "9c9b12f"
# NOTE: Stored value is the canonical Python hash (9c9b12f); the bash hook reports a divergent value (5598136) — advisory only per PG-HASH-HOOK-DIVERGENCE.
---

# STORY-183: check-green-doc-tense: bin/*.py Prose Coverage + TIER-1 Behavioral-Absence Token Coverage

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** ready
**Wave:** 86
**Points:** 5
**Priority:** P2

## Narrative

- **As a** CI gate operator reviewing step-4 (green-doc-tense) gate results
- **I want** `bin/check-green-doc-tense` to scan `bin/*.py` files as well as `*.rs` files,
  and to implement all TIER-1 behavioral-absence tokens from DF-GREEN-DOC-TENSE-SWEEP v6
  (F-W86S-P2-006 / F-W86S-P3-001 / F-W86S-P4-004 rulings) that are absent from the current
  `_VIOLATION_PATTERNS` set (pattern numbering is owned by the tool's registry; the policy
  names tokens by literal text, never by number)
- **So that** Python files in `bin/` containing stale RED-phase documentation are never
  silently skipped by the gate (PG-W84-010), and the TIER-1 phrasing classes discovered during
  STORY-180 pass-1 (PG-W85-003, 9 stale sites, primarily `currently asserts`) can no longer
  escape the gate undetected

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; bin/ tooling change only)_

## Background

### Governing Policy

This story is governed by **DF-GREEN-DOC-TENSE-SWEEP v6** (`policies.yaml`, updated
2026-07-25 per F-W86S-P3-001 ruling and further updated by F-W86S-P4-004 ruling). The v6
two-tier enforcement model (F-W86S-P2-006 / F-W86S-P3-001 / F-W86S-P4-004 rulings) is the
canonical authority for all pattern decisions in this story. Any conflict between this
story's acceptance criteria and DF-GREEN-DOC-TENSE-SWEEP v6 resolves in favor of the policy.

**Number-agnostic policy (F-W86S-P4-004, v6):** Pattern numbering is owned by
`bin/check-green-doc-tense`'s `_VIOLATION_PATTERNS` registry. The policy names tokens by
literal text, never by number. No claim in this story that "the policy prescribes pattern
number N" is authoritative — only the tool registry is.

### PG-W84-010 — bin/check-green-doc-tense scans only *.rs files

`bin/check-green-doc-tense` collects files via `_collect_rust_files()` (line ~466), which
runs `git ls-files -- tests/*.rs src/**/*.rs` and filters with `line.endswith(".rs")`. The
scan explicitly excludes all `bin/*.py` files. During STORY-176 adversarial pass-1 (wave-84),
the adversary caught stale RED-phase prose in `bin/test_check_green_doc_tense.py` that had
passed the gate (finding F-S176P1-003). The root cause was confirmed in
`df-validation-2026-07-25.md §PG-W84-010`.

### PG-W85-003 — Missing TIER-1 behavioral-absence phrase classes

`bin/check-green-doc-tense` `_VIOLATION_PATTERNS` (line 217) contains **28 tuples**, labeled
Pattern 1 through Pattern 29 in the module docstring token list. **Ground truth (`.factory/cycles/wave-085/STORY-180/convergence-report.md` lines 63-66, STORY-180 F-180-P1-003):** the 9 D-506
stale sites used `currently asserts` and `is expected to` — NOT `Expected RED:` /
`currently falls through`. The lesson summary AND the PG-W85-003 paragraph at
`.factory/cycles/wave-085/STORY-180/convergence-report.md` lines 68-70 both carried broader (incorrect) labels. Lines 63-66 are
the primary citation for the specific D-506 token set; the citation is non-exhaustive and
the zero-live-hit greps are the mechanical backstop for confirming token absence.

The following TIER-1 tokens (zero-FP automatable, 0 live legitimate uses each) are absent
from the existing 28 tuples (confirmed by grep of `bin/check-green-doc-tense`):
- `currently asserts` — primary D-506 class; 9 stale sites (`.factory/cycles/wave-085/STORY-180/convergence-report.md` line 64)
- `falls to the wildcard` — 0 live legitimate uses
- `currently fall` — covers "currently falls through", "currently fall through", etc.
- `doesn't exist yet` / `does not exist yet`
- `currently has NO`
- `currently satisfied by`
- `will be GREEN currently`

**TIER-2 tokens (context-dependent; MUST NOT be added to tool per F-W86S-P2-006 / F-W86S-P3-001 PO rulings):**
- `is expected to` — **secondary D-506 phrasing class** (`.factory/cycles/wave-085/STORY-180/convergence-report.md` line 64); 6+ live
  legitimate uses (e.g., "this test is expected to PASS on the current codebase"). The tool
  MUST NOT flag this token. Manual/adversarial sweep owns it.
- `falls through to` — 10 live legitimate uses accurately describing match-arm fallthrough
  (corrected from v3's "12" per DF-GREEN-DOC-TENSE-SWEEP v4 F-W86S-P3-017 grep, 2026-07-25).
- `no .* arm` — [MOVED FROM TIER-1, v3→v4, F-W86S-P3-001] 3 live legitimate uses documenting
  exhaustive-match design. Adversary checks context; tool MUST NOT flag.
- `not yet implemented` — [MOVED FROM TIER-1, v3→v4, F-W86S-P3-001] 17 live uses (historical
  error-string quotes, Red Gate provenance, assertion messages). Adversary checks context; tool
  MUST NOT flag.
- `currently FAILS` / `currently fails` — [MOVED FROM TIER-1, v3→v4, F-W86S-P3-001] 1 live
  legitimate RED-guard use (D-078 D11 fix not shipped). Tool MUST NOT flag.

Additionally, `Expected RED:` (0 live uses) is added as Pattern 30 as a mandated
zero-FP guard (per AC-183-003), even though it was NOT in the D-506 sites.

### Scan semantics: `_is_comment_line()` returning True means ELIGIBLE TO BE FLAGGED

`scan_file()` (line ~502) iterates every line and calls `_is_comment_line()`. When
`_is_comment_line()` returns True, the line IS eligible to be scanned for violations. When it
returns False, the line is SKIPPED (not scanned). The entire BAD_CASES corpus in
`test_check_green_doc_tense.py` consists of `//`-comment lines that return True from
`_is_comment_line()` and ARE flagged. For Python files, `#`-prefixed lines are the comment
lines and MUST return True from `_is_comment_line()` so they ARE scanned for violations.
This is the mechanism that makes `.py` scanning useful — the opposite of an exemption.

### String-literal false-positive mechanics

When `bin/test_check_green_doc_tense.py` is scanned, multi-line string literals whose
content lines start with `//` (e.g., the BAD_CASES fixture strings) will be flagged by
existing patterns because `_is_comment_line()` only checks whether the STRIPPED line
begins with `//` — it does NOT check whether that line is inside a string literal.

**The correct fix:** convert multi-line BAD_CASES string fixtures to single-line string form.
A single-line string `"// harness skeleton compiles only\n"` appears in the Python source as:
```
    "// harness skeleton compiles only\n",
```
After stripping: `"// harness skeleton compiles only\n",` — this begins with `"`, NOT `//`,
so `_is_comment_line()` returns False and the line is not scanned. The temp `.rs` file
receives the full `// harness skeleton compiles only\n` content and IS flagged correctly.

A genuine `.py` scan of the self-test file hits **40 `//`-line fixtures + 2 `#` lines at
258/261** = **42 violations** that must be eliminated before the gate exits 0. The 2 `#`
lines must be reworded (Task 4). The 40 `//`-line violations must be fixed by converting
each multi-line string fixture to single-line form (Task 5).

**Scope of `bin/*.py` glob:** The `git ls-files -- bin/*.py` glob covers Python files
with `.py` extension. The `.py` file set is exactly **6 `test_*.py` files**
(`bin/test_check_green_doc_tense.py`, `bin/test_compute_input_hash.py`,
`bin/test_changelog_gate_content.py`, `bin/test_gitignore_mutants_glob.py`,
`bin/test_validate_citations.py`, `bin/test_lint_cycle_artifact.py`).
`bin/compute-input-hash` and `bin/changelog-gate-check` have **no `.py` extension** and
are NOT matched by the glob. Extension-less Python executables in `bin/` (e.g.,
`bin/check-green-doc-tense` itself, `bin/fetch-e2e-pcaps`) are **out of scope** for this
story — they would require shebang-based detection. The follow-up gap is noted for a
separate story if needed. **In-source `# Pattern NN:` comments** in `bin/check-green-doc-tense`
must use non-flagging wording (not containing the literal flagged phrase) so they do not
trigger the gate if the tool is later included in its own scan.

### DF-VALIDATION coupling ruling

`df-validation-2026-07-25.md §Cross-Finding Observations, point 2`: "PG-W84-010 (scan glob)
and PG-W85-003 (pattern set) both target `bin/check-green-doc-tense` and should be one story."
STORY-183 combines both fixes.

### CHANGELOG obligation

`bin/` changes trigger AC-158-001 / PG-W71-CHANGELOG. A `[Unreleased]` CHANGELOG entry is
required for this story at delivery time (enforced by the `changelog-gate` CI job).

### CI wiring obligation

STORY-183 extends the EXISTING `bin/test_check_green_doc_tense.py` (not creating a new file).
L-W84-003 / AC-165-001 requires CI wiring only for NEW `bin/test_*.py` files. No CI wiring
change is required for the self-test file. The ci.yml comment lines :434 and :442 + step-name line :462 (scope descriptions "tests/*.rs and src/**/*.rs" and "in test files") are stale prose that must be updated — these are non-functional edits only; the FUNCTIONAL job steps remain unchanged.

## Acceptance Criteria

### AC-183-001 (traces to PG-W84-010 — scan glob extension to bin/*.py)

`bin/check-green-doc-tense` is extended to include `bin/*.py` files in its prose scan.

- Given `_collect_rust_files()` at line ~466 currently runs `git ls-files -- tests/*.rs src/**/*.rs`
  and filters with `line.endswith(".rs")`, excluding all Python files including `bin/*.py`;
  additionally, under git's default pathspec matching (without `:(glob)` magic), `src/*.rs`
  crosses directory separators and covers all `.rs` files under `src/`, subsuming
  `src/**/*.rs`; `src/**/*.rs` is retained in the merged invocation for explicit-intent
  redundancy; git ls-files de-duplicates index entries so no double-count (F-W86S-P12-006)
- When the function is renamed from `_collect_rust_files` to `_collect_source_files` and
  extended to use a SINGLE merged invocation `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py`
  and include entries ending with `.py` (for Python); under git's default pathspec matching,
  `src/*.rs` crosses directory separators and covers all of src/, subsuming `src/**/*.rs`;
  `src/**/*.rs` is retained for explicit-intent redundancy; git ls-files de-duplicates index
  entries so the merged invocation cannot double-count (F-W86S-P12-006)
- And the rename is propagated to 13 `_collect_rust_files` sites + 1 `rust_files` prose site at :721 (14 total) in `bin/test_check_green_doc_tense.py`
  (6 functional monkey-patch sites at approximately :699/:707/:726/:859/:872/:905;
  7 `_collect_rust_files` prose sites at approximately :688,:705,:711,:718,:839,:843,:891;
  plus :721 — a `rust_files` prose site, failure-message string citing `if not rust_files:` guard — update to `source_files`)
- And the `main()` function (line ~549) is updated to call the new/renamed collector function
- And the pass-message at line ~591 is updated: rename the local variable from `rust_files` to `source_files` only (variable rename only — no "both file types" prose change required)
- And a self-test check using the runner's PASS/FAIL convention prints
  `  PASS  [_collect_source_files: test_check_green_doc_tense.py found]` and increments
  `passed` when `_collect_source_files(repo_root)` returns a path list (list[Path]) containing an entry
  whose `.name` attribute equals `test_check_green_doc_tense.py`; prints
  `  FAIL  [_collect_source_files: test_check_green_doc_tense.py found]` and increments
  `failures` otherwise (comparison by `.name` avoids repo-root-prefix fragility — do NOT
  compare to a repo-relative string like `"bin/test_check_green_doc_tense.py"`).
  **`repo_root` derivation:** use `mod._find_repo_root(Path(mod.__file__).resolve().parent)`
  where `mod` is the imported `check-green-doc-tense` module. Note: the PRE-EXISTING
  monkey-patch sections AC-158-005 (bin/test_check_green_doc_tense.py:698-726, patch at :704)
  and AC-162-003 (:858-905, patch at :871) patch `_find_repo_root` to return hermetic
  tempdirs — if the `_collect_source_files(repo_root)` assertions added by this story are
  placed inside those blocks, they resolve to a hermetic tempdir → empty file set → spurious
  FAIL. Insert the assertions immediately after the `finally` block ending at :905 and BEFORE
  the `print()` at :907 — this ensures the checks execute and feed the passed/failures counters
  and the Results: line (outside all monkey-patch finally blocks). The Task 9
  hermetic test runs a fresh subprocess and cannot see parent-process monkey patches; its
  placement relative to the finally blocks does not affect it.
- And a self-test check using the runner's PASS/FAIL convention prints
  `  PASS  [_collect_source_files: src/*.rs file found]` and increments `passed` when
  `_collect_source_files(repo_root)` contains at least one entry satisfying
  `p.parent.name == "src" and p.suffix == ".rs"` — expressed as
  `any(p.parent.name == "src" and p.suffix == ".rs" for p in files)` (class assertion;
  `mitre.rs` is the illustrative example but any top-level `src/*.rs` file satisfies it);
  prints `  FAIL  [_collect_source_files: src/*.rs file found]` and increments `failures`
  otherwise. This check verifies that top-level `src/*.rs` files appear in the scanned set;
  `src/**/*.rs` requires a literal `/` after the star segment and NEVER matches top-level
  files like `src/mitre.rs` (10 top-level files); `src/*.rs` (star crosses /) covers all
  `.rs` files under `src/` and strictly SUBSUMES `src/**/*.rs` — `src/*.rs` is
  LOAD-BEARING and MUST NOT be dropped (F-W86S-P12-006)

- Then `python3 bin/check-green-doc-tense` (bare invocation — no file arguments; main() uses
  git ls-files internally) exits non-zero when any `bin/*.py` file contains a pattern match
  in a `#`-prefixed comment line, and exits 0 when no such violations exist
- And the scanned-file count in the pass-message includes `.py` files (e.g., "N files scanned"
  where N includes at least one `.py` entry; pre-story baseline Rust-only count:
  `git ls-files -- tests/*.rs src/**/*.rs | wc -l`; post-story count (after this story adds
  `src/*.rs` and `bin/*.py` globs): `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py | wc -l`;
  the `.name == "test_check_green_doc_tense.py"` check from AC-183-001's When clause above already entails `.suffix == ".py"` membership — not a brittle exact-count check)

Verification:
```bash
set -euo pipefail
# The gate is verified via its self-test runner — no file arguments exist:
python3 bin/test_check_green_doc_tense.py
# Must pass (exit 0), including new .py fixture cases added in AC-183-003/004/007

# Manual smoke: after delivery, confirm the gate scans bin/*.py
python3 bin/check-green-doc-tense
# Must exit 0 (zero violations in bin/*.py after all scrubs and rewords in this story)

# Positive .py coverage proof: self-test runner exercises bad_N.py files
# (via runner extension in Task 7) — confirmed by ".py form" PASS lines in output

# ci.yml scope-update verification (F-003 process-gap):
test "$(grep -c 'in test files' .github/workflows/ci.yml)" -eq 0
grep -qF 'src/*.rs' .github/workflows/ci.yml
grep -qF 'bin/*.py' .github/workflows/ci.yml
```

### AC-183-002 (traces to PG-W84-010 — Python comment-line scan eligibility)

The comment-line detection in `bin/check-green-doc-tense` is extended so that Python
`#`-prefixed lines ARE ELIGIBLE TO BE SCANNED (and flagged) for violations — the same
semantics as `//`-prefixed Rust lines.

- Given `_is_comment_line()` currently only returns True for `stripped.startswith("//")`,
  meaning ONLY `//`-prefixed lines are scanned for violations
- When `_is_comment_line()` is extended with a `suffix` parameter so that `#` prefix is
  treated as a comment ONLY for `.py` files (prevents ~3625 Rust attribute lines from
  becoming scan-eligible — see F-008, EC-012):
  ```python
  def _is_comment_line(stripped: str, suffix: str = "") -> bool:
      """Return True if the line is a comment — eligible to be scanned for violations.

      Rust: lines starting with // (including /// outer doc and //! inner doc)
      Python: lines starting with # (any Python comment)

      The `#` prefix is treated as a comment ONLY for .py files to avoid false-positives
      from Rust attribute lines (#[test], #[cfg], #[should_panic]). Rust source has
      approximately 3625 such `#[...]` attribute lines — adding them to the scan unscoped
      would produce massive false-positive noise.

      Returning True means the line IS scanned; returning False means the line is skipped.
      """
      if stripped.startswith("//"):
          return True
      if stripped.startswith("#") and suffix == ".py":
          return True
      return False
  ```
- And `scan_file()` passes `path.suffix` to `_is_comment_line()` for every line check
- Then `scan_file()` processes `#`-prefixed lines in `.py` files through all violation
  patterns, flagging any that match; Rust `#[attr]` lines are NOT scan-eligible

**Implementer obligation — zero-false-positive for `bin/test_check_green_doc_tense.py`:**
After adding `bin/*.py` to the scan, the self-test file hits 42 violations (see Background
§String-literal false-positive mechanics). All 42 must be eliminated before `python3
bin/check-green-doc-tense` exits 0. See Tasks 4 and 5 for the concrete remediation steps.

Verification:
```bash
set -euo pipefail
# After all scrubs and rewording, full bin/*.py scan must exit 0:
python3 bin/check-green-doc-tense
# Must exit 0 (non-zero exit causes set -e to abort — no echo needed)

# Self-test must pass:
python3 bin/test_check_green_doc_tense.py
# Must exit 0
```

### AC-183-003 (traces to PG-W85-003 — "Expected RED:" heading pattern, Pattern 30)

A new violation pattern is added to `_VIOLATION_PATTERNS` as the **29th tuple** (labeled
"Pattern 30") in `bin/check-green-doc-tense`.

- Given that no entry for `Expected RED` appears in `_VIOLATION_PATTERNS` (confirmed by grep)
- When the new tuple is appended after the existing Pattern 29 entry:
  ```python
  # -----------------------------------------------------------------------
  # Patterns 30+: added to implement TIER-1 behavioral-absence tokens
  # from DF-GREEN-DOC-TENSE-SWEEP v6 (F-W86S-P2-006/P3-001/P4-004 rulings, 2026-07-25)
  # absent from the existing _VIOLATION_PATTERNS set (STORY-183, wave-86).
  # -----------------------------------------------------------------------
  (
      # Pattern 30: stale pre-pass RED heading with colon (test doc phase marker).
      # The colon is the discriminator (marks heading form); bare form without colon is OK.
      # 0 live legitimate uses (as of wave-86). NOT in the D-506 9 stale sites.
      # Allowlist: bare form without colon; past-tense provenance ("was expected to fail").
      "Pattern 30 (PG-W85-003): 'Expected RED:' heading — stale pre-pass state (AC-183-003)",
      re.compile(r"Expected RED:", re.IGNORECASE),
  ),
  ```
- And the corresponding known-bad `.rs` fixture is added to BAD_CASES in
  `bin/test_check_green_doc_tense.py`:
  ```python
  (
      "Pattern 30: 'Expected RED:' heading violation (.rs form)",
      "// Expected RED: all assertions fail before implementation\n",
      "Pattern 30",
  ),
  ```
- And the corresponding known-bad `.py` fixture is added to BAD_CASES (4-tuple with `.py`
  extension — requires runner extension in Task 7):
  ```python
  (
      "Pattern 30: 'Expected RED:' heading violation (.py form — Python comment)",
      "# Expected RED: all assertions fail before implementation\n",
      "Pattern 30",
      ".py",
  ),
  ```
- And the corresponding known-good fixture is added to GOOD_CASES:
  ```python
  (
      "Pattern 30 allowlist: past-tense — was expected to fail (RED phase), no colon",
      "// This test was expected to fail (RED phase) before the implementation shipped.\n",
  ),
  ```

- Then `python3 bin/test_check_green_doc_tense.py` exits 0 with both Pattern 30 bad cases
  flagged and the allowlist case clean

**Zero-false-positive requirement:**
- `// Expected RED:` (Rust comment) — IS flagged ✓
- `# Expected RED:` (Python comment) — IS flagged ✓ (via AC-183-002 extension)
- `// was expected to fail (RED phase)` — NOT flagged (no colon; matches allowlist)
- `expected_red` (snake_case identifier in non-comment line) — NOT flagged (not a comment line)

**Runner extension required:** BAD_CASES currently uses 2-element or 3-element tuples written
to `bad_N.rs` files. Pattern 30's `.py` BAD case requires a 4-element tuple with `.py`
extension. The runner must be extended to use `entry[3]` as the file extension when present:
```python
ext = entry[3] if len(entry) > 3 else ".rs"
p = _tmpfile(content, tmp, f"bad_{passed}{ext}")
```

- And `python3 bin/check-green-doc-tense` MUST exit non-zero when any scanned source file
  contains a line matching Pattern 30 — this is the gate semantics per AC-183-001; the
  BAD_CASE mechanism above exercises this path via the test runner

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must exit 0; both Pattern 30 BAD cases must appear in output as PASS
# (The gate MUST exit non-zero when encountering a Pattern 30 match — exercised by BAD_CASEs)
```

### AC-183-004 (traces to PG-W85-003 — "currently fall" body-phrase pattern, Pattern 31)

A new violation pattern is added to `_VIOLATION_PATTERNS` as the **30th tuple** (labeled
"Pattern 31") in `bin/check-green-doc-tense`.

- Given no entry for `currently fall` appears in `_VIOLATION_PATTERNS` (confirmed by grep)
- When the new tuple is appended after Pattern 30:
  ```python
  (
      # Pattern 31: stale present-tense dispatch description (in-progress fallthrough variants).
      # Word "currently" is the discriminator; `falls?\b` matches both fall and falls (verb inflection).
      # 0 live legitimate uses of the present-tense "currently + falls?" form.
      # Allowlist: past-tense "fell through to"; bare "falls through to" without the modifier
      # (TIER-2 per DF-GREEN-DOC-TENSE-SWEEP v6 F-W86S-P2-006/P3-001/P4-004 rulings — MUST NOT be flagged).
      "Pattern 31 (PG-W85-003): currently fall(s) — stale in-progress dispatch description (AC-183-004)",
      re.compile(r"currently\s+falls?\b", re.IGNORECASE),
  ),
  ```
- And the corresponding known-bad `.rs` fixture is added to BAD_CASES:
  ```python
  (
      "Pattern 31: currently falls through violation (.rs form)",
      "// the packet currently falls through the dispatcher to the wildcard arm\n",
      "Pattern 31",
  ),
  ```
- And the corresponding known-bad `.py` fixture is added to BAD_CASES (4-tuple):
  ```python
  (
      "Pattern 31: currently falls through violation (.py form — Python comment)",
      "# the packet currently falls through the dispatcher to the wildcard arm\n",
      "Pattern 31",
      ".py",
  ),
  ```
- And the corresponding known-good fixtures are added to GOOD_CASES:
  ```python
  (
      "Pattern 31 allowlist: past tense — fell through before the fix",
      "// Before the fix, the packet fell through the dispatcher to the wildcard arm.\n",
  ),
  (
      # Per DF-GREEN-DOC-TENSE-SWEEP v6 (F-W86S-P2-006/P3-001/P4-004 rulings, 2026-07-25),
      # `falls through to` is a TIER-2 context-dependent token that bin/check-green-doc-tense
      # MUST NOT flag; asserting the tool does not flag it is correct behavior, not a defect.
      "Pattern 31 zero-FP: falls through to (TIER-2 — MUST NOT be flagged per F-W86S-P2-006 PO ruling)",
      "// The lax path falls through to the wildcard arm when no analyzer matches.\n",
  ),
  ```

- Then `python3 bin/test_check_green_doc_tense.py` exits 0 with both Pattern 31 bad cases
  flagged and both allowlist/zero-FP cases clean

**Zero-false-positive requirement:**
- `// the packet currently falls through` — IS flagged ✓
- `# the packet currently falls through` — IS flagged ✓ (via AC-183-002)
- `// before the fix, the packet fell through` — NOT flagged (past tense "fell") ✓
- `// falls through to the wildcard arm` — NOT flagged (no "currently" prefix; TIER-2 by policy) ✓

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must exit 0; both Pattern 31 BAD cases must appear in output as PASS
```

### AC-183-005 (traces to AC-158-001 / PG-W71-CHANGELOG — CHANGELOG obligation for bin/ changes)

A `[Unreleased]` CHANGELOG entry is added for the bin/ changes in STORY-183.

- Given `bin/check-green-doc-tense` and `bin/test_check_green_doc_tense.py` are in the
  `bin/` directory, which is in the AC-158-001 changelog-gate trigger set
- When the implementing PR is opened, the PR MUST include a new item under `[Unreleased]`
  in `CHANGELOG.md` covering PG-W84-010 + PG-W85-003 + STORY-183
- And the CI `changelog-gate` job passes on the PR

Verification:
```bash
set -euo pipefail
# Use the true CI-equivalent gate invocation (three-dot diff, matches ci.yml:533):
git diff origin/develop...HEAD -- CHANGELOG.md | bin/changelog-gate-check
# Must exit 0 — gate verifies that ADDED non-heading content lines exist under [Unreleased]
# in the diff (not section placement; a section header with no content lines would fail)
# NOTE: two-dot `git diff origin/develop -- CHANGELOG.md` counts uncommitted edits and
# diverges from CI behaviour — always use three-dot form for gate verification.
```

### AC-183-006 (traces to PG-W84-010 — positive .py coverage proof via suffix-scoped mechanism)

A deliberate test proves that a `.py` file with a stale `#`-comment IS flagged by the gate
(positive coverage: the gate is not merely inert on Python files). After the F-008 fix, the
`.py` extension in 4-tuple BAD cases is genuinely load-bearing: the runner writes `bad_N.py`,
and `_is_comment_line(stripped, suffix=".py")` returns True for `#` lines ONLY because the
suffix is `.py`. A `.rs` file with the same `#` content would NOT be flagged.

- Given `_is_comment_line(stripped, suffix)` returns True for `#`-prefixed lines only when
  `suffix == ".py"` (AC-183-002 language-scoped mechanism, F-008)
- When the self-test runner's BAD cases include 4-tuple entries with `".py"` extension
  (see AC-183-003, AC-183-004) and the runner writes them as `bad_N.py`
- Then `scan_file(bad_N.py)` returns a non-empty violations list, confirming patterns fire on
  `#`-prefixed Python comment lines (suffix `.py` makes them scan-eligible)
- And a `bad_N.rs` file containing the same `# Expected RED:` line would NOT be flagged
  (suffix `.rs` — `_is_comment_line` returns False for `#` prefix on non-.py files)
- And a GOOD_CASE is added to `bin/test_check_green_doc_tense.py` exercising this negative
  guard explicitly:
  ```python
  (
      # Suffix-scoping negative guard (F-009): `#`-prefixed line in a .rs file is NOT
      # scan-eligible; `_is_comment_line(stripped, suffix=".rs")` returns False for `#` prefix.
      # Proves `.py` eligibility is suffix-scoped, not global.
      "Suffix-scoping negative guard: '# Expected RED:' in .rs file NOT flagged (# is not a Rust comment)",
      "# Expected RED: all assertions fail before implementation\n",
  ),
  ```
  The runner writes this content to `good_{n}.rs`; the gate scans it as a `.rs` file and
  MUST NOT flag it (suffix `.rs` → `_is_comment_line` returns False for `#` prefix)

**End-to-end assertion (wiring AC-183-001 and AC-183-006):**
The self-test also verifies that `main()` discovers `bin/*.py` via `_collect_source_files()`
and that the file count returned includes at least one `.py` file. This is already asserted
in AC-183-001 (`_collect_source_files(repo_root)` returns a `list[Path]` containing an entry whose `.name` equals `test_check_green_doc_tense.py` — compare by `.name`, never by repo-relative string). The gate's live run (`python3 bin/check-green-doc-tense`)
exercises the full pipeline: collect → scan with suffix-scoped comment detection → report.

Verification:
```bash
set -euo pipefail
python3 bin/test_check_green_doc_tense.py
# In output: ".py form" PASS entries confirm suffix-scoped positive .py coverage

python3 bin/check-green-doc-tense
# Must exit 0 — end-to-end: collects bin/*.py, scans with suffix-scoped _is_comment_line
```

### AC-183-007 (traces to PG-W85-003 — Patterns 32–37: remaining TIER-1 behavioral-absence tokens)

Six additional violation patterns are added to `_VIOLATION_PATTERNS` as the **31st–36th
tuples** (labeled Patterns 32–37) in `bin/check-green-doc-tense`, implementing all remaining
TIER-1 tokens from DF-GREEN-DOC-TENSE-SWEEP v6 absent from the current 28-tuple set.
Patterns originally proposed as 34 (`no .* arm`), 36 (`not yet implemented`), and 40
(`currently FAILS`) are TIER-2 per DF-GREEN-DOC-TENSE-SWEEP v6 F-W86S-P3-001 ruling and are
NOT added; their exclusion is validated by GOOD_CASEs below.

- Given none of the following TIER-1 tokens appear in the existing 28 tuples (confirmed by
  `grep` of `bin/check-green-doc-tense` returning no output for each)
- When the following six tuples are appended after Pattern 31:
  ```python
  (
      # Pattern 32: present-tense assertion-state claim (D-506 primary, 9 stale sites,
      # STORY-180 F-180-P1-003, `.factory/cycles/wave-085/STORY-180/convergence-report.md` line 64). 0 live legitimate uses.
      "Pattern 32 (PG-W85-003): 'currently asserts' — RED-phase present-tense claim (AC-183-007)",
      re.compile(r"currently\s+asserts?\b", re.IGNORECASE),
  ),
  (
      # Pattern 33: wildcard-arm fallthrough phrase; TIER-1 per v6; 0 live legitimate uses.
      # The TIER-1 form omits the intervening word present in TIER-2.
      # Discriminator: TIER-2 inserts "through" between the verb and the wildcard destination
      # ("falls through to" — 10 live uses); the TIER-1 form lacks that intermediary word.
      # Pattern fires only on the contiguous four-token phrase (verb + `to the wildcard`
      # with no intervening word); the TIER-2 form `falls through to` is not matched
      # because the interposed `through` breaks the verb→`to` adjacency.
      "Pattern 33 (PG-W85-003): 'falls to the wildcard' — RED-phase wildcard-arm fallthrough (AC-183-007)",
      re.compile(r"falls\s+to\s+the\s+wildcard", re.IGNORECASE),
  ),
  (
      # Pattern 34: negative-capability claim (not-yet-extant feature); 0 live legitimate uses.
      # Renumbered from Pattern 35 (v3); prior v3 Pattern 34 moved to TIER-2 in v4 (F-W86S-P3-001).
      "Pattern 34 (PG-W85-003): 'does not / doesn't exist yet' — negative-capability claim (AC-183-007)",
      re.compile(r"does\s+not\s+exist\s+yet|doesn['’]?t\s+exist\s+yet", re.IGNORECASE),
  ),
  (
      # Pattern 35: present-tense absence claim; 0 live legitimate uses.
      # Renumbered from Pattern 37 (v3).
      # Known theoretical FP: the pattern also matches a hyphenated no-prefix token
      # immediately after the phrase, because \b fires at the hyphen (e.g. a "no-op"
      # continuation). 0 live hits.
      "Pattern 35 (PG-W85-003): 'currently has NO' — present-tense absence claim (AC-183-007)",
      re.compile(r"currently\s+has\s+no\b", re.IGNORECASE),
  ),
  (
      # Pattern 36: passive stub-status phrase; 0 live legitimate uses.
      # Renumbered from Pattern 38 (v3).
      "Pattern 36 (PG-W85-003): 'currently satisfied by' — passive stub-status phrase (AC-183-007)",
      re.compile(r"currently\s+satisfied\s+by\b", re.IGNORECASE),
  ),
  (
      # Pattern 37: conditional present-RED tense claim; 0 live legitimate uses.
      # Renumbered from Pattern 39 (v3); retained per F-W86S-P4-024 adjudication.
      "Pattern 37 (PG-W85-003): 'will be GREEN currently' — conditional present-RED claim (AC-183-007)",
      re.compile(r"will\s+be\s+GREEN\s+currently", re.IGNORECASE),
  ),
  ```
- And each new pattern has at least one BAD_CASE (`.rs` form) and one GOOD_CASE in
  `bin/test_check_green_doc_tense.py`; Patterns 32/33 additionally have `.py` BAD_CASES.
  The prescribed fixture strings (mechanically verifiable) are:
  ```python
  # Pattern 32 BAD (.rs):
  ("Pattern 32: currently asserts violation (.rs form)",
   "// the iec104 decoder currently asserts a valid ASDU header\n",
   "Pattern 32"),
  # Pattern 32 BAD (.py):
  ("Pattern 32: currently asserts violation (.py form)",
   "# the iec104 decoder currently asserts a valid ASDU header\n",
   "Pattern 32", ".py"),
  # Pattern 32 GOOD (allowlist — present-tense assertion-state phrase absent):
  ("Pattern 32 allowlist: past tense — the decoder verified the header",
   "// the decoder verifies that each ASDU header is valid after processing\n"),
  # Pattern 33 BAD (.rs):
  ("Pattern 33: falls to the wildcard violation (.rs form)",
   "// the unrecognized packet falls to the wildcard arm for logging\n",
   "Pattern 33"),
  # Pattern 33 BAD (.py):
  ("Pattern 33: falls to the wildcard violation (.py form)",
   "# the unrecognized packet falls to the wildcard arm for logging\n",
   "Pattern 33", ".py"),
  # Pattern 33 GOOD (allowlist — wildcard-arm fallthrough phrase absent):
  ("Pattern 33 allowlist: different routing — forwarded to default handler",
   "// the unrecognized packet is forwarded to the default handler branch\n"),
  # Pattern 34 BAD (.rs):
  ("Pattern 34: does not exist yet violation (.rs form)",
   "// this error-handling path does not exist yet in the decoder\n",
   "Pattern 34"),
  # Pattern 34 GOOD (negative-capability phrase absent):
  ("Pattern 34 allowlist: past tense — path was added",
   "// this code path was added in response to the missing handler report\n"),
  # Pattern 35 BAD (.rs):
  ("Pattern 35: currently has NO violation (.rs form)",
   "// the TLS dissector currently has no SNI extraction logic\n",
   "Pattern 35"),
  # Pattern 35 GOOD (allowlist — present-tense absence claim absent):
  ("Pattern 35 allowlist: different phrasing — lacks",
   "// the TLS dissector lacks SNI extraction for DTLS traffic\n"),
  # Pattern 36 BAD (.rs):
  ("Pattern 36: currently satisfied by violation (.rs form)",
   "// the invariant is currently satisfied by the no-op stub placeholder\n",
   "Pattern 36"),
  # Pattern 36 GOOD (allowlist — passive stub-status phrase absent):
  ("Pattern 36 allowlist: production implementation",
   "// the invariant is enforced by the production TCP-state machine\n"),
  # Pattern 37 BAD (.rs):
  ("Pattern 37: will be GREEN currently violation (.rs form)",
   "// this gate will be GREEN currently because the check is bypassed\n",
   "Pattern 37"),
  # Pattern 37 GOOD (allowlist — conditional present-RED tense claim absent):
  ("Pattern 37 allowlist: future tense without currently",
   "// this gate will be GREEN after the implementation ships\n"),
  ```

**First-match-wins / break-on-first constraint:** The `_VIOLATION_PATTERNS` list is evaluated
in sequential order for each line; the FIRST matching pattern wins and subsequent patterns are
not checked for that line (break-on-first semantics). Pattern priority is determined by list
position (Pattern 30 before 31 before 32, etc.), preserved by the append-only policy. A BAD
fixture that happens to match two patterns (e.g., a line with both "currently asserts" and
"Expected RED:") will be reported under the LOWER-numbered pattern that appears first in the
list. GOOD_CASE fixtures must not match ANY pattern — pattern order does not affect their
clean-pass requirement.

- And the three TIER-2-confirmed tokens have GOOD_CASEs asserting NOT-flagged, each with a
  TIER-2 citation comment per DF-GREEN-DOC-TENSE-SWEEP v6 F-W86S-P3-001 ruling:
  ```python
  (
      # DF-GREEN-DOC-TENSE-SWEEP v6 TIER-2: 'no .* arm' FALSIFIED as TIER-1 (F-W86S-P3-001).
      # 3 live legitimate uses document exhaustive-match design. Tool MUST NOT flag.
      "TIER-2 zero-FP: 'No wildcard arm' NOT flagged (moved TIER-2 v4 per F-W86S-P3-001)",
      "// No wildcard arm: compiler enforces exhaustiveness.\n",
  ),
  (
      # DF-GREEN-DOC-TENSE-SWEEP v6 TIER-2: 'not yet implemented' FALSIFIED (F-W86S-P3-001).
      # 17 live uses (historical error-string quotes, RED-gate messages). Tool MUST NOT flag.
      "TIER-2 zero-FP: 'not yet implemented' in error string NOT flagged (TIER-2 v4 F-W86S-P3-001)",
      '// Err("ARP extraction not yet implemented")\n',
  ),
  (
      # DF-GREEN-DOC-TENSE-SWEEP v6 TIER-2: 'currently fails' FALSIFIED (F-W86S-P3-001).
      # 1 live legitimate RED-guard use (D-078 D11 fix not shipped). Tool MUST NOT flag.
      "TIER-2 zero-FP: 'currently fails' NOT flagged (moved TIER-2 v4 per F-W86S-P3-001)",
      "// which currently fails.\n",
  ),
  ```
- And the module docstring token list is updated to document Patterns 30–37 (28 existing +
  8 new = 36 total tuples; note: the docstring token list has 37 items / 36 tuples —
  item 5 shares tuple 4)

- Then `python3 bin/test_check_green_doc_tense.py` exits 0 with all Pattern 32–37 BAD cases
  flagged and all three TIER-2 GOOD_CASEs clean (tool does NOT flag them)

**No-literal-phrase sweep obligation (F-001 — 5th recurrence prevention):** The Task-4/Task-6
rule (no literal flagged phrase in any in-source `#` comment) applies to EVERY `#` comment
this story prescribes for `bin/test_check_green_doc_tense.py`. No annotation line in the
fixture block above may match any of the 36 patterns (8 new Patterns 30–37 + 28 existing
Patterns 1–29). The fixture-block annotations have been worded to DESCRIBE the allowlist
condition without quoting the literal phrase (e.g., "allowlist — present-tense assertion-state
phrase absent" instead of `no "currently asserts"`). Implementer obligation: verify the
prescribed Python block for every TIER-1 token — zero `#` annotation lines may match.

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must exit 0; Pattern 32–37 BAD cases must appear in output as PASS;
# three TIER-2 GOOD_CASEs must confirm tool does NOT flag them
```

### AC-183-008 (traces to PG-W85-003 — efficacy: TIER-1 D-506 phrase-level phrasing covered; TIER-2 correctly excluded)

The final pattern set (Patterns 30–37) covers the TIER-1 phrasing from the 9 D-506 stale
sites and produces zero false positives on live TIER-2 sites, per DF-GREEN-DOC-TENSE-SWEEP v6
(F-W86S-P2-006 / F-W86S-P3-001 rulings, 2026-07-25).

**Scope note (P16-006):** Patterns 30–37 are phrase-level patterns only; bare tokens
(`RED GATE:`, `todo!()`) are NOT covered and are accepted residuals — see Notes §Story scope
clarification.

- Given the 9 D-506 stale sites (STORY-180 F-180-P1-003, `.factory/cycles/wave-085/STORY-180/convergence-report.md` lines 63-66) used:
  - `currently asserts` (TIER-1) — covered by Pattern 32 ✓
  - `is expected to` (TIER-2) — correctly NOT flagged by the tool per F-W86S-P2-006 ruling ✓
- When Patterns 30–37 are delivered and GOOD_CASES cover:
  - `"// The lax path falls through to the wildcard arm."` (TIER-2 zero-FP, Pattern 31 GOOD case)
  - `"// Before the fix, the packet fell through the dispatcher."` (past tense, not flagged)
  - `"// was expected to fail (RED phase) before the implementation shipped."` (allowlisted)
  - `"// this test is expected to PASS on the current codebase"` (TIER-2 'is expected to' — tool
    must NOT flag this; GOOD_CASE confirms the tool correctly skips it)
  - `"// No wildcard arm: compiler enforces exhaustiveness."` (TIER-2 'no .* arm' — NOT flagged)
  - `'// Err("ARP extraction not yet implemented")'` (TIER-2 'not yet implemented' — NOT flagged)
  - `"// which currently fails."` (TIER-2 'currently fails' — NOT flagged)
- Then `python3 bin/test_check_green_doc_tense.py` passes all GOOD_CASES related to patterns
  30–37

**TIER-2 non-flagging requirement (F-W86S-P2-006 / F-W86S-P3-001 rulings):**
The tool MUST NOT flag `is expected to`. Add a GOOD_CASE to confirm:
```python
(
    # DF-GREEN-DOC-TENSE-SWEEP v6 TIER-2: 'is expected to' has 6+ live legitimate uses.
    # Tool MUST NOT flag it — manual/adversarial sweep only. NOT a tool defect.
    "Efficacy: 'is expected to' NOT flagged (TIER-2 per F-W86S-P2-006 / F-W86S-P3-001 rulings)",
    "// this test is expected to PASS on the current codebase\n",
),
```

**Zero-FP regression spot-check:**
The following 10 live sites use "falls through to" WITHOUT "currently" and MUST NOT be flagged
(as of e8841d76):
- `src/analyzer/tls.rs:930`
- `tests/bc_2_16_d078_lax_malformed_tests.rs:18,90,216`
- `tests/bc_2_16_d078b_lax_some_arm_tests.rs:335`
- `tests/main_story_089_tests.rs:890`
- `tests/dnp3_f6_story140_group_a_survivors.rs:812,850`
- `tests/bc_f6_mutation_gap_tests.rs:791,793`

The Pattern 31 zero-FP GOOD_CASE and the gate's live run (`python3 bin/check-green-doc-tense`)
serve as the combined regression guard.

Verification:
```bash
set -euo pipefail
# Run self-test — all Pattern 30-37 GOOD_CASES must pass:
python3 bin/test_check_green_doc_tense.py

# Zero-FP regression check on live codebase (must exit 0):
python3 bin/check-green-doc-tense
# If this exits 0, the 10 live 'falls through to' sites plus all 'is expected to'
# sites are clean — zero false positives from Patterns 30-37.
```

### AC-183-009 (process-gap PG-W84-012 extended at D-525 — bin/test_lint_cycle_artifact.py MUST pass locally)

`python3 bin/test_lint_cycle_artifact.py` MUST exit 0 (all self-tests pass) at delivery time.
CI wiring for this selftest file is tracked under PG-W84-012 AS EXTENDED at D-525 per
F-W86S-P9-012, which covers all three: (a) required-status-check registration for the
`bin-selftest` job, (b) adding `bin/test_lint_cycle_artifact.py` as a step in that job,
AND (c) adding `bin/test_compute_input_hash.py` as a step in that job.
The selftest runs locally only until parts (b) and (c) land — do NOT add a CI wiring task
for this story; that remains the PG-W84-012 (D-525) ops task pending devops-engineer
dispatch and human authorization.

- Given `bin/test_lint_cycle_artifact.py` is modified by Task 13 (4-line scrub at lines :3, :5,
  :6, :125) as part of this story
- When the implementer runs `python3 bin/test_lint_cycle_artifact.py` locally after the Task 13
  scrub
- Then the command MUST exit 0 with the same pass count as before the scrub (the scrub is a
  pure text rewording — no test logic changes; the TC count remains 21)
- And the following mechanical grep predicates ALL print 0 — these fail exactly when
  Task 13 is skipped:
  ```bash
  set -euo pipefail
  test "$(grep -c "RED GATE version" bin/test_lint_cycle_artifact.py)" -eq 0
  test "$(grep -c "MUST FAIL until bin/lint-cycle-artifact" bin/test_lint_cycle_artifact.py)" -eq 0
  test "$(grep -c "TC1–TC8" bin/test_lint_cycle_artifact.py)" -eq 0
  ```
  (The en-dash in `TC1–TC8` must match the literal character in the file; use the exact
  phrase from the source. These three checks fail if and only if Task 13's 4-line scrub
  was not applied — lines :3, :5, :6, and :125 of `bin/test_lint_cycle_artifact.py`.)
- And CI wiring for this file is explicitly NOT added in this story's PR (see PG-W84-012
  extended at D-525 for the ops-task tracking all three: (a) `bin-selftest`
  required-status-check registration, (b) adding `bin/test_lint_cycle_artifact.py` as a
  step, and (c) adding `bin/test_compute_input_hash.py` as a step)

Verification:
```bash
set -euo pipefail
python3 bin/test_lint_cycle_artifact.py
# Must exit 0; pass count must be 21 (TC1–TC21 unchanged by the scrub)

test "$(grep -c "RED GATE version" bin/test_lint_cycle_artifact.py)" -eq 0
test "$(grep -c "MUST FAIL until bin/lint-cycle-artifact" bin/test_lint_cycle_artifact.py)" -eq 0
test "$(grep -c "TC1–TC8" bin/test_lint_cycle_artifact.py)" -eq 0
# All three gate on 0 — fail exactly when Task 13 scrub was skipped

# CI wiring: NOT wired in this story — deferred to PG-W84-012 (D-525) ops task
# covers (a) required-status-check registration + (b) bin/test_lint_cycle_artifact.py
#   step + (c) bin/test_compute_input_hash.py step
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Scan glob extension + language-scoped Python comment-line scan eligibility | `bin/check-green-doc-tense` (amend `_collect_rust_files` → `_collect_source_files`, `_is_comment_line(stripped, suffix)`, `main`) | develop |
| Patterns 30–37 (28 tuples → 36 tuples) | `bin/check-green-doc-tense` (amend `_VIOLATION_PATTERNS`) | develop |
| Runner `.py` extension support + 12 new BAD_CASES + 14 new GOOD_CASES (26 total) | `bin/test_check_green_doc_tense.py` (amend) | develop |
| Stale-comment scrub (lines 258/261) | `bin/test_check_green_doc_tense.py` (amend — reword two #-prefixed lines) | develop |
| String-literal false-positive resolution (~40 multi-line fixtures → single-line) | `bin/test_check_green_doc_tense.py` (amend) | develop |
| CHANGELOG entry | `CHANGELOG.md` (amend `[Unreleased]`) | develop |
| Stale scope comment update | `.github/workflows/ci.yml` (amend comment lines :434 and :442 + step-name line :462; non-functional edits only) | develop |
| Confirmed-stale docstring/comment scrub (Task 13 — 4 lines: :3, :5, :6, :125) | `bin/test_lint_cycle_artifact.py` (amend) | develop |

**No `src/` changes, no `Cargo.toml` changes, no functional CI workflow changes.**
CHANGELOG obligation: YES — `bin/` changes trigger AC-158-001.
No new `bin/test_*.py` file — L-W84-003 / AC-165-001 CI-wiring obligation does NOT apply.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Rust comment `// Expected RED: stale heading` | IS scanned and IS flagged — `_is_comment_line()` returns True for `//` prefix (existing behavior); Pattern 30 matches |
| EC-002 | Python comment `# Expected RED: stale heading` | IS scanned and IS flagged — `_is_comment_line()` returns True for `#` prefix (AC-183-002); Pattern 30 matches |
| EC-003 | Python non-comment line containing `Expected RED:` (e.g., string literal `"// Expected RED: ...\n"`) | NOT flagged — after stripping, line begins with `"`, not `//` or `#`; `_is_comment_line()` returns False; scan skips it |
| EC-004 | `expected_red` (snake_case identifier) in a non-comment line | NOT flagged — non-comment line is skipped |
| EC-005 | `bin/test_check_green_doc_tense.py` existing multi-line string literals contain `//` lines with violation patterns | These produce 40 false-positive violations when the gate scans the file. Resolution: convert each such fixture to single-line string format (Task 5). Zero-FP after conversion is a HARD requirement. |
| EC-006 | `bin/` contains a `.py` file with ONLY `#`-comment lines matching a pattern | IS flagged — `#`-prefixed lines are ELIGIBLE to be flagged (AC-183-002); desired behavior |
| EC-007 | `// currently falls through` in Rust comment line | IS scanned and IS flagged — `_is_comment_line()` returns True; Pattern 31 matches |
| EC-008 | `// falls through to the wildcard arm` (no "currently") | NOT flagged — Pattern 31 requires "currently" prefix; TIER-2 token by policy |
| EC-009 | `// this test is expected to PASS` | NOT flagged — `is expected to` is TIER-2 (F-W86S-P2-006 ruling); tool must not flag it |
| EC-010 | Extension-less Python executables in `bin/` (e.g., `bin/check-green-doc-tense`); and `.py` files outside `bin/` (`tests/fixtures/mk_modbus_large_pcap.py`, `tests/fixtures/mk_modbus_pcap.py`, `fuzz/seed_corpus.py`) | OUT OF SCOPE for this story — `git ls-files -- bin/*.py` glob only covers `bin/*.py` files; extension-less executables require shebang-based detection (deferred); `.py` files outside `bin/` are outside PG-W84-010's target surface; the residual surface is recorded as a follow-up candidate — see STATE.md drift row DRIFT-py-surface-outside-bin |
| EC-011 | Python docstring (string literal) containing stale RED-phase prose | NOT scanned — docstrings are string literals, not `#`-prefixed comment lines; `_is_comment_line()` returns False and they are skipped. **DF-GREEN-DOC-TENSE-SWEEP v6 known-residual class:** confirmed-stale scrub targets: `bin/test_lint_cycle_artifact.py:3` ('— RED GATE version.'), `:5` (stale TC count 'TC1–TC8'), `:6` ('All tests MUST FAIL until bin/lint-cycle-artifact is created.'), and `:125` ('`# Test cases (TC1–TC8)`' — `#`-comment line, newly scan-eligible under this story) — all four scrubbed as part of this story in Task 13; DRIFT-docstring-scan row is recorded by state-manager (already present in STATE.md; no action in this story's PR). NOT-stale verdicts (no action needed): `bin/test_gitignore_mutants_glob.py:12` and `bin/test_validate_citations.py:645,647`. PG-W84-010 claim is scoped to comment-line prose only; docstring scanning deferred. |
| EC-012 | Rust source line `#[test]` / `#[cfg]` / `#[should_panic]` attribute | NOT scanned — `_is_comment_line(stripped, suffix)` returns False for `#` prefix when suffix is not `.py` (F-008 language-scoped fix). Rust `#` is an attribute delimiter, not a comment. Approximately 3625 Rust attribute lines are excluded from scan-eligibility by this scoping. |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `bin/check-green-doc-tense` | effectful-shell | Shell script (Python): git subprocess, file I/O, stdout/stderr. Pure-data additions (`_VIOLATION_PATTERNS`, `_is_comment_line` extension) live inside an effectful script. |
| `bin/test_check_green_doc_tense.py` | effectful-shell | Test runner: spawns subprocesses, reads/writes temp files, asserts on exit codes and scan results. |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| Story spec (this file) | ~7.0 k |
| `bin/check-green-doc-tense` (full script, ~690-700 lines after additions) | ~5.5 k |
| `bin/test_check_green_doc_tense.py` (full file, ~1000-1050 lines after additions) | ~7.5 k |
| `.github/workflows/ci.yml` (comment-only change, surrounding lines) | ~0.5 k |
| `CHANGELOG.md` (recent unreleased section only) | ~0.5 k |
| Tool outputs overhead | ~1.0 k |
| **Total** | **~22.0 k** |
| Agent context window | 200 k (Sonnet) |
| **Budget usage** | **~11%** |

Well within context window. No story split required.

## Tasks

1. **Read both target files fully** — `bin/check-green-doc-tense` (understand
   `_collect_rust_files()` line ~466, `_is_comment_line()` line ~460, `_VIOLATION_PATTERNS`
   line 217 — 28 tuples, labels 1–29, last entry is Pattern 29 "until … wired"). Read
   `bin/test_check_green_doc_tense.py` fully — understand BAD_CASES (2-/3-tuple format,
   runner writes `bad_N.rs` temp files), GOOD_CASES, and all existing test runner sections.
   File is 913 lines. Confirm 28 tuples and absence of all TIER-1 tokens via grep.

2. **Rename and extend `_collect_source_files()` glob (AC-183-001):** Rename
   `_collect_rust_files` to `_collect_source_files` (mandatory — no "(or annotate)"
   alternative). Replace the existing Rust-only glob with a SINGLE merged invocation
   `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py` (under git's default pathspec
   matching, `src/*.rs` crosses directory separators and covers all of src/, subsuming
   `src/**/*.rs`; `src/**/*.rs` is retained for explicit-intent redundancy; git ls-files
   de-duplicates index entries so the merged invocation cannot double-count —
   F-W86S-P12-006). Accept `.py` endings.
   Update `main()` call site. Update the pass-message at ~591: rename the local variable
   from `rust_files` to `source_files` (variable rename only).
   Add a self-test check using the runner's PASS/FAIL convention confirming
   `_collect_source_files(repo_root)` returns a path list (list[Path]) containing an entry whose `.name`
   attribute equals `test_check_green_doc_tense.py`; prints `  PASS  [_collect_source_files: test_check_green_doc_tense.py found]` and increments `passed` on success, prints
   `  FAIL  [_collect_source_files: test_check_green_doc_tense.py found]` and increments
   `failures` on failure (comparison by `.name` avoids repo-root-prefix fragility — do NOT
   compare to a repo-relative string like `"bin/test_check_green_doc_tense.py"`).
   **`repo_root` derivation:** use `mod._find_repo_root(Path(mod.__file__).resolve().parent)`
   (where `mod` is the imported module). Using the script's own file location (not `Path.cwd()`)
   ensures correct repo root resolution regardless of the caller's working directory. The hermetic
   test functions (Task 9) use subprocess isolation — they do NOT monkey-patch `_find_repo_root`;
   monkey-patching of `_find_repo_root` is pre-existing in AC-158-005 (`:698-726`) and
   AC-162-003 (`:858-905`) and is NOT introduced by Task 9.
   Add a self-test check using the runner's PASS/FAIL convention confirming
   `_collect_source_files(repo_root)` contains at least one entry satisfying
   `p.parent.name == "src" and p.suffix == ".rs"` — expressed as
   `any(p.parent.name == "src" and p.suffix == ".rs" for p in files)` (class assertion;
   avoids mitre.rs-literal fragility; `mitre.rs` is illustrative only); prints
   `  PASS  [_collect_source_files: src/*.rs file found]` and increments `passed` on success,
   prints `  FAIL  [_collect_source_files: src/*.rs file found]` and increments `failures` on
   failure. This check cannot pass with only `src/**/*.rs` in the glob — see AC-183-001
   F-W86S-P9-009.
   **Propagate rename inside `bin/check-green-doc-tense` itself:**
   - Function docstring at lines :466-474 (baseline develop@e8841d76, pre-edit — re-locate
     by content after each insertion; and module docstring at line :4) — update scope
     descriptions to enumerate the full pathspec: "tests/*.rs, src/**/*.rs, src/*.rs, and
     bin/*.py" (under git's default pathspec, `src/*.rs` crosses directory separators and
     covers all of src/; `src/**/*.rs` is retained for explicit-intent redundancy; git
     ls-files de-duplicates — all four globs listed for clarity — F-W86S-P12-006)
   - Local variable `rust_files` at :556 (baseline develop@e8841d76, pre-edit) → rename to
     `source_files` throughout the function
   - Error string at :558-560 (baseline develop@e8841d76, pre-edit; `"no tracked Rust files found"`) →
     update to `"no tracked source files found"` (covers both `.rs` and `.py`)
   **Propagate rename to 13 `_collect_rust_files` sites + 1 `rust_files` prose site at :721 (14 total) in `bin/test_check_green_doc_tense.py`:**
   - 6 functional monkey-patch sites (approximately :699/:707/:726 zero-file guard;
     :859/:872/:905 spy) — these MUST be updated or tests will fail
   - 7 `_collect_rust_files` prose/comment sites (approximately :688,:705,:711,:718,:839,:843,:891)
   - Plus 1 `rust_files`-only prose site at :721 (failure-message string citing
     `if not rust_files:` guard in `main()` — update to `source_files`)
   **Do NOT update either CHANGELOG.md locus affected by this rename** — both are SHIPPED
   HISTORICAL entries preserved per DF-SIBLING-SWEEP-001:
   - `:741` — cites `_collect_rust_files` by name (documents what shipped in STORY-162/176);
     must remain as-is for audit provenance.
   - `:851` — cites the pre-rename error-string prose "no tracked Rust files are found"
     (documents the AC-158-005 zero-file guard as shipped); must remain as-is.
   Similarly, historical references in delivered story specs (STORY-158, STORY-162) are
   preserved as shipped spec provenance — do NOT sweep `.factory/stories/` history.

3. **Extend `_is_comment_line(stripped, suffix)` (AC-183-002, F-008):** Add a `suffix`
   parameter (default `""`). Return True for `#`-prefixed lines ONLY when `suffix == ".py"`.
   Update every call site in `scan_file()` to pass `path.suffix`. This prevents ~3625 Rust
   attribute lines (`#[test]`, `#[cfg]`, `#[should_panic]`) from becoming scan-eligible (EC-012).
   Verify AC-183-002 semantics: returning True makes the line ELIGIBLE TO BE FLAGGED.

4. **Scrub stale `#`-comments in `bin/test_check_green_doc_tense.py` (F-006 / AC-183-002):**
   Two Python `#`-prefixed comment lines at approximately lines ~258 and ~261 contain the
   literal flagged phrases `"harness skeleton compiles"` and `"fails until wired"` inside
   quoted examples. The regex patterns match inside quotes equally — the gate will flag these
   lines regardless of quoting. **Safety criterion (not "no literal phrase" — that is
   insufficient for regex patterns like 29 and 23):** the rewritten line MUST NOT *match*
   any of the 36 patterns. Verify mechanically after rewording with
   `python3 bin/check-green-doc-tense`.
   - Line ~258: reword entirely to eliminate `harness skeleton compiles`, e.g.:
     `#   (a) \bskeleton\s+compiles?\b  — compile-only stub-era assertion (pattern 26)`
   - Line ~261: reword entirely to eliminate `fails until wired`, e.g.:
     `#   (d) \buntil\b[^\n]*\bwired\b   — CI-wiring-incomplete prose (pattern 29)`
   The prescribed replacements are safe by a specific, deliberate mechanism: writing the
   pattern in regex-literal form places the escape character `\b` immediately before the
   trigger word, so the text reads `…buntil` / `…bwired` and Pattern 29's `\b` assertion
   cannot fire (the preceding `b` is a word character). **Do NOT "clean up" these lines by
   removing the `\b` escapes** — `#  (d) until … wired` DOES match Pattern 29 and will
   fail the gate. Sibling line `:213` is already safe by the same `\b`-escape mechanism.
   Sibling lines `:259` and `:260` are safe for a different reason — Patterns 27/28 require
   an `exposes|is a|are` verb immediately before `compile-only`, which is absent — so they
   need no `\b` guard. All three are deliberately left unchanged.

5. **Resolve string-literal false positives in `bin/test_check_green_doc_tense.py`
   (AC-183-002 zero-FP obligation):** The BAD_CASES section contains multi-line string
   literals whose indented content lines start with `//` after stripping. A genuine `.py`
   scan of the self-test file produces 40 such violations. The correct fix:
   **Convert each multi-line string fixture to single-line form** so the Python source line
   begins with `"`, not `//`. Example:
   ```python
   # BEFORE (multi-line — triggers false positive when gate scans the .py file):
   """\
   // harness skeleton compiles only — wiring deferred to STORY-176
   """,
   # AFTER (single-line — Python source line begins with '"'; not flagged):
   "// harness skeleton compiles only — wiring deferred to STORY-176\n",
   ```
   The temp `.rs` file receives the full `// harness skeleton compiles only\n` string and IS
   flagged correctly by the gate — the test is unaffected. Apply this conversion to all ~40
   multi-line fixtures in BAD_CASES. The `bin/test_check_green_doc_tense.py` self-test file
   MUST remain in the scan set (no skip-file pragma — that would invert the PG-W84-010
   self-application requirement).
   **GOOD_CASES are deliberately NOT converted** — a GOOD_CASE whose content matches any
   pattern is a test-design error the self-test already catches via the GOOD_CASE assertion.
   Residual: ~45 `//`-prefixed source lines remain in the scanned file from GOOD_CASES multi-line
   fixtures and must be re-checked whenever a new pattern is appended.
   **Quote-escaping note (F-018):** Two fixtures in BAD_CASES (approximately at lines :91
   and :97) contain a literal `"` character inside the string. (:402 is a GOOD_CASE
   non-comment line and is NOT in BAD_CASES.) Converting these to
   double-quoted single-line strings requires escaping the inner `"` as `\"`. To avoid
   escaping, use single-quoted strings instead: `'// this has a "quoted" word\n'`.
   After conversion, `python3 bin/check-green-doc-tense` MUST exit 0.

6. **Add Patterns 30–31 to `_VIOLATION_PATTERNS` (AC-183-003, AC-183-004):** After the
   Pattern 29 entry, add the section comment and two new tuples. Verify numbering: the 28th
   tuple is Pattern 29, the new 29th tuple is Pattern 30, the new 30th tuple is Pattern 31.
   In-source `# Pattern NN:` comments MUST NOT contain the literal flagged phrase (quoting does not prevent regex match — see Task 4).

7. **Extend `bin/test_check_green_doc_tense.py` runner for `.py` extension (AC-183-003/004/006):**
   In the BAD_CASES loop, use `entry[3]` as the file extension when present:
   ```python
   ext = entry[3] if len(entry) > 3 else ".rs"
   p = _tmpfile(content, tmp, f"bad_{passed}{ext}")
   ```
   Add the 4 new BAD_CASES (2 for Pattern 30 — `.rs` and `.py`; 2 for Pattern 31 — `.rs`
   and `.py`). Add 3 new GOOD_CASES (Pattern 30 allowlist, Pattern 31 allowlist, Pattern 31
   zero-FP for bare "falls through to" with verbatim PO ruling).
   **Type annotation note (F-020):** The `BAD_CASES` list type annotation at approximately
   line :51 currently covers 2- or 3-element tuples. After adding 4-element tuples (with `.py`
   extension), update the annotation to include the 4-tuple variant. If the type becomes
   unwieldy, add `# type: ignore` on the BAD_CASES assignment line rather than introducing
   a complex Union type.

8a. **Add self-test BAD_CASES and GOOD_CASES for Patterns 32–37 (AC-183-007 — pre-RED step):**
    Add the 8 BAD_CASES prescribed in AC-183-007: Pattern 32 (`.rs` and `.py` forms),
    Pattern 33 (`.rs` and `.py` forms), Pattern 34 (`.rs`), Pattern 35 (`.rs`), Pattern 36
    (`.rs`), Pattern 37 (`.rs`). Add the 6 GOOD_CASEs (one per pattern). Add the three
    TIER-2 zero-FP GOOD_CASEs (for `no .* arm`, `not yet implemented`, `currently fails`)
    asserting the tool does NOT flag them (per F-W86S-P3-001 ruling). Add the efficacy
    GOOD_CASE for `is expected to` (AC-183-008). Add the suffix-scoping negative-guard
    GOOD_CASE (AC-183-006): the case where a `# Expected RED:` line written to a `.rs` temp
    file is NOT flagged. DO NOT append any tuples to `_VIOLATION_PATTERNS` in this step —
    that is Task 8b. **Run self-test after this step:** the self-test MUST FAIL (RED) because
    no pattern yet matches the new BAD_CASES. If it passes, the BAD_CASES are wrong or the
    patterns already exist.

8b. **Append Patterns 32–37 tuples to `_VIOLATION_PATTERNS` (AC-183-007 — GREEN step, run
    together with Task 6):** Append the 6 new tuples as detailed in AC-183-007 (Patterns
    32–37; prior-v3 candidates `no .* arm`, `not yet implemented`, `currently FAILS` are
    TIER-2 per F-W86S-P3-001 ruling and are NOT added). Run self-test after this step (and
    Task 6) — MUST PASS (GREEN), clearing all new BAD_CASES. In-source `# Pattern NN:`
    comments MUST NOT contain the literal flagged phrase (quoting does not prevent regex
    match — see Task 4).

9. **E2E positive-coverage self-test (F-010):** Add a test function to
   `bin/test_check_green_doc_tense.py` that verifies the full collect→scan→exit pipeline
   in a hermetic environment. Required new imports (explicit deliverables): `subprocess`
   and `shutil` — add both at the top of the test file (neither is currently present).
   Note: `tempfile` is already imported at function scope at approximately :640 — do NOT
   add it as a top-level import; the function-local binding at `:640` shadows the
   module-level name, not the reverse (a top-level import is shadowed inside any function
   that rebinds the same name locally).
   **Placement constraint (load-bearing for non-hermetic assertions):** the two
   non-hermetic `_collect_source_files(repo_root)` assertions (from Task 2) MUST be placed
   OUTSIDE (after) the PRE-EXISTING monkey-patch sections at :698-726 (AC-158-005) and
   :858-905 (AC-162-003), which patch `_find_repo_root` to return hermetic tempdirs.
   Placing those assertions inside a monkey-patch block causes `_collect_source_files` to
   resolve to a hermetic tempdir → empty file set → spurious FAIL.
   The Task 9 hermetic test runs a fresh subprocess and cannot see parent-process monkey
   patches; placement of the hermetic test relative to the finally blocks does NOT affect
   the subprocess. Insert both (hermetic test and non-hermetic assertions) immediately after
   the `finally` block ending at :905 and BEFORE the `print()` at :907, so the checks
   execute and feed the passed/failures counters and the Results: line.
   - Create a temporary directory `<tmp>` and run `git init` inside it to make it a valid
     git repo (`_find_repo_root` walks upward from the **script's own location** to find the
     root — there is NO `WIRERUST_REPO_ROOT` env override; do NOT use it)
   - Copy `bin/check-green-doc-tense` into `<tmp>/bin/check-green-doc-tense` (creates the
     `<tmp>/bin/` directory) so that `_find_repo_root` resolves to `<tmp>` when the copy
     is invoked
   - `git -C <tmp> add bin/check-green-doc-tense` (optional — has no effect on
     `_collect_source_files`; the suffix-filter for `.py` means this extension-less file is
     never collected; only `bin/violating.py` must be indexed)
   - Create `<tmp>/bin/violating.py` with a **single-line** `#`-prefixed comment line
     containing a TIER-1 phrase (e.g., `"# currently asserts the implementation is complete\n"`).
     **Must be single-line**: after AC-183-002 extends the scan to `bin/*.py`, a multi-line
     fixture in this file would be a `#`-prefixed line itself eligible for scanning, causing
     Pattern 32 to self-flag on `# currently asserts` — keeping it single-line avoids this.
   - `git -C <tmp> add bin/violating.py` (cwd=`<tmp>`) — `git ls-files` is index-only; an
     untracked file is invisible to `_collect_source_files()` and the harness would exit 0
     vacuously
   - Run the copy in the temp repo with explicit stream capture:
     ```python
     proc = subprocess.run([sys.executable, str(tmp/"bin"/"check-green-doc-tense")],
                           capture_output=True, text=True)
     combined = proc.stdout + proc.stderr
     ```
     (`capture_output=True, text=True` is required — without it, stdout and stderr go to
     the terminal and `proc.stdout`/`proc.stderr` are `None`.)
   - If the process exits 1 (violation detected): print
     `  PASS  [hermetic-e2e: exit 1 on violation]` and increment `passed`;
     otherwise print `  FAIL  [hermetic-e2e: exit 1 on violation]` and increment `failures`
   - If `combined` does NOT contain `"no tracked source files found"` (verifies exit-1
     was a genuine violation, not the empty-collection guard in `main()` at :557-563 —
     both paths exit 1, so this negative assertion distinguishes them; note that
     `bin/check-green-doc-tense:558-562` writes this message to `file=sys.stderr`, so it
     appears in `proc.stderr` and therefore in `combined`): print
     `  PASS  [hermetic-e2e: not empty-collection exit]` and increment `passed`;
     otherwise print `  FAIL  [hermetic-e2e: not empty-collection exit]` and increment
     `failures`
   - If `combined` contains a FAIL line naming `bin/violating.py` and the pattern label
     (e.g., `"FAIL [bin/violating.py:1]: Pattern 32 ..."`): print
     `  PASS  [hermetic-e2e: output names violating.py]` and increment `passed`;
     otherwise print `  FAIL  [hermetic-e2e: output names violating.py]` and increment
     `failures`
   - If `len(_collect_source_files(tmp)) == 1` (the temp repo has exactly one tracked
     source file — `bin/violating.py`; `bin/check-green-doc-tense` has no `.py` suffix
     so it is not collected): print
     `  PASS  [hermetic-e2e: collect finds exactly 1 source file]` and increment `passed`;
     otherwise print `  FAIL  [hermetic-e2e: collect finds exactly 1 source file]` and
     increment `failures`
   This validates the full integration path: `_collect_source_files()` discovers the `.py`
   file, `_is_comment_line()` returns True for the `#` line, and the pattern fires. This is
   the df-validation self-application smoke test confirming PG-W84-010 is exercised end-to-end.

10. **Sibling-prose sweep (F-009):**
    - `bin/check-green-doc-tense` line :4 (module docstring headline sentence): rewrite
      the FULL headline sentence — not only the glob list. Current text:
      `"Scans tracked test files (tests/*.rs and src/**/*.rs cfg(test) modules) for"`
      Required rewrite: `"Scans tracked source files (tests/*.rs, src/**/*.rs, src/*.rs, and bin/*.py) for"`
      The noun phrase "test files" changes to "source files"; the parenthetical
      "(tests/*.rs and src/**/*.rs cfg(test) modules)" changes to enumerate all four
      globs — the "cfg(test) modules" parenthetical is FALSE post-story (it falsely implies
      only test-annotated modules are scanned); it MUST be removed, not merely amended.
      (Under git's default pathspec, `src/*.rs` crosses directory separators and covers
      all of src/; `src/**/*.rs` is retained for explicit-intent redundancy; git
      ls-files de-duplicates — all four globs listed for clarity — F-W86S-P12-006.)
    - `bin/check-green-doc-tense` line :467 (`_collect_source_files` docstring first line):
      current text: `"Collect test-scope Rust files:"` — rewrite to
      `"Collect scanned source files:"`. Also update the bullet points within the docstring
      to enumerate src/*.rs and bin/*.py alongside the existing globs.
    - `bin/check-green-doc-tense` line :472 (docstring exclusion rationale): current text:
      `"so newly-added test files do not cause false failures"` — rewrite to
      `"so newly-added source files do not cause false failures"`.
    - `bin/check-green-doc-tense` lines :26-30 (TOKEN LIST preamble — contains the
      comment-marker claim, NOT glob text): the current text states anchoring is to lines
      whose non-whitespace content starts with `//` or `//!`. After AC-183-002, anchors
      are `//` (all files) and `#` (`.py` files, suffix-scoped). Update lines :28-29 to
      state: "matches `//` comment lines in all scanned files; `#` comment lines in
      `.py` files."
    - `bin/check-green-doc-tense` lines :31-85 (TOKEN LIST body): update to document
      Patterns 30–37 (8 new tokens added by this story).
    - `bin/check-green-doc-tense` ALLOWLIST heading at ~:90: no glob update required
      here — the glob scope lives in the function body and docstring, not in the
      ALLOWLIST heading.
    - Lines :87-88 (baseline develop@e8841d76, pre-edit — re-locate by content after each
      insertion) and line :97: The current :87-88 text falsely claims anchoring is "to
      comment lines (leading `//` or `//!`) ... to avoid false-positives ... inside string
      literals." After this story: anchors are `//` (all files) and `#` (`.py` files,
      suffix-scoped); string-literal false-positives DO occur — the 42-site remediation
      (Tasks 4 and 5) exists precisely because of them. Rewrite :87-88 to: (a) state that
      `_is_comment_line` now accepts a `suffix` parameter and returns True for `//` on all
      files and `#` on `.py` files only; (b) note that string-literal comment-shaped lines
      (e.g., BAD_CASES fixtures with `//` content lines) ARE flagged by design — see EC-005
      and Task 5 for the remediation protocol (F-007 / F-W86S-P12-007). Separately, for
      line :97 ("Pattern-specific allowlist notes (patterns 12-29):"): if allowlist notes
      for patterns 30-37 are added to that section, update the range label to include them;
      otherwise add a one-line inline note that Patterns 30-37 carry inline `# Allowlist:`
      comments in their tuple comments, so the "12-29" heading remains literally accurate.
    - Lines :212-215 (module-level pattern-registry comment block introducing
      `_VIOLATION_PATTERNS` at :217, baseline develop@e8841d76, pre-edit — re-locate by
      content): the current comment describes which lines the patterns match against using
      `//` or `//!` only. After AC-183-002 extends comment-line eligibility to `#` for
      `.py` files, update this block to state that patterns match `//` comment lines in
      all scanned files and `#` comment lines in `.py` files. (Note: `_is_comment_line()`
      at :~460 has its own one-line docstring at :461 that is rewritten wholesale by
      AC-183-002's block — no contradiction with Task 1's `:~460` reference.)
    - Lines :577 (baseline develop@e8841d76, pre-edit; "in test files" failure summary) and
      :581–585 (baseline develop@e8841d76, pre-edit; explanatory prose) — re-locate by
      content after each insertion: update scope descriptions from "test files" to
      "source files" to include `bin/*.py`.
    - `.github/workflows/ci.yml` stale prose sites (COMMENT-ONLY changes; functional job
      steps unchanged):
      - Line ~442: update scope comment to enumerate the full pathspec delivered:
        "tests/*.rs, src/**/*.rs, src/*.rs, and bin/*.py"
      - Line :434: job comment says "in test files" — stale once `bin/*.py` and `src/*.rs`
        are in scope; update to "in tracked source files (tests/*.rs, src/**/*.rs, src/*.rs,
        and bin/*.py)"
      - Line :462: step name "Scan for stale RED-phase comment headers in test files"
        — stale; update to "Scan for stale RED-phase comment headers in source files"
      - Line :436: "during strict TDD, test files receive module-level or section-level
        comments..." — **adjudicated historical problem-origin narrative**, deliberately
        preserved; this sentence describes where the anti-pattern was first observed (test
        files during TDD), not the current gate scope (stated at :442). Do NOT update this
        line (F-W86S-P21-009 adjudication). The AC predicate `grep -c 'in test files'` does
        not match `:436`'s "test files receive" phrase, so the predicate is satisfied without
        modifying :436.

11. **Add CHANGELOG `[Unreleased]` entry (AC-183-005):** One-line entry describing the
    scan-glob + language-scoped comment-detection + TIER-1 behavioral-absence pattern
    extensions (Patterns 30–37), referencing PG-W84-010, PG-W85-003, STORY-183.

12. **Run full self-test and zero-FP gate check (AC-183-001/002/006):**
    ```bash
    set -euo pipefail
    python3 bin/test_check_green_doc_tense.py  # must exit 0
    python3 bin/check-green-doc-tense           # must exit 0 (no new violations)
    ```

13. **Scrub confirmed-stale docstring/comment sites in `bin/test_lint_cycle_artifact.py` (F-002/EC-011):**
    Four lines in `bin/test_lint_cycle_artifact.py` contain stale phrasing and must be
    scrubbed as part of this delivery:
    - Line 3 (`'— RED GATE version.'`): string literal/docstring — reword to past-tense
      provenance, dropping the literal "RED GATE" token, e.g.:
      `'— Delivered pre-implementation; all tests were red until bin/lint-cycle-artifact landed (STORY-158).'`
    - Line 5 (stale TC count — `'TC1–TC8 implement all eight test cases'` or similar):
      string literal/docstring — the file now contains TC1–TC21 (21 self-tests); update
      the count to reflect the actual test count, e.g.:
      `'TC1–TC21 implement all 21 self-tests.'`
    - Line 6 (`'All tests MUST FAIL until bin/lint-cycle-artifact is created.'`): string
      literal/docstring — reword to past-tense, e.g.:
      `'Tests were written to fail before bin/lint-cycle-artifact was created.'`
    - Line 125 (`# Test cases (TC1–TC8)` or similar): `#`-comment line, **newly
      scan-eligible** after Task 2's glob extension — the stale TC count (TC1–TC8) must
      be updated to match the actual 21 test cases, e.g.:
      `# Test cases (TC1–TC21)`
    After rewording all four lines, verify: `python3 bin/test_lint_cycle_artifact.py` must
    exit 0 with the same pass count as before the scrub. DRIFT-docstring-scan row is
    recorded by state-manager (already present in STATE.md; no action in this story's PR).
    NOT-stale verdicts (no action needed): `bin/test_gitignore_mutants_glob.py:12` and
    `bin/test_validate_citations.py:645,647` — confirmed NOT-stale per v5 classification.

14. **Develop PR:** `bin/check-green-doc-tense`, `bin/test_check_green_doc_tense.py`,
    `bin/test_lint_cycle_artifact.py`, `CHANGELOG.md`, and `.github/workflows/ci.yml`
    (comment-only) in a single develop PR. CI `changelog-gate` must pass.

## Previous Story Intelligence

- **STORY-176 (wave-84, patterns 26–29):** Added skeleton/seam/compile-only/until-wired
  patterns. The wave-84 adversary found stale Python doc missed by the gate (F-S176P1-003) —
  PG-W84-010 tracks this gap. STORY-183 adds patterns 30–37 using the same append-only
  methodology. Read STORY-176 Tasks before implementing for BAD_CASES / GOOD_CASES format.
- **STORY-180 (wave-85):** Adversarial pass-1 found 9 stale sites with `currently asserts`
  and `is expected to` phrasing (D-506, PG-W85-003, `.factory/cycles/wave-085/STORY-180/convergence-report.md` lines 63-66). STORY-183
  adds Pattern 32 (`currently asserts` — TIER-1) as the primary fix. `is expected to` is
  TIER-2 and correctly excluded from the tool per F-W86S-P2-006 PO ruling.
- **Patterns 1–29 baseline:** 28 tuples, labeled Pattern 1 through Pattern 29. Pattern 29 is
  the "until … wired" entry (last in the `# Patterns 26-29` block). Append-only; never reorder
  or remove existing entries. New entries become tuples 29–36 (Patterns 30–37).

## Architecture Compliance Rules

- **Append-only `_VIOLATION_PATTERNS`:** Never remove or reorder existing patterns. New
  patterns are appended as the 29th–36th tuples with sequential label numbers (30–37).
- **`bin/` changes require CHANGELOG (AC-158-001):** Any PR touching `bin/` files MUST include
  an `[Unreleased]` CHANGELOG entry. The `changelog-gate` CI job enforces this.
- **L-W84-003 / AC-165-001 CI wiring:** Not triggered — STORY-183 extends an existing
  `bin/test_*.py` file. No new `.github/workflows/ci.yml` job steps required.
- **Action SHA-pin policy:** STORY-183 makes no functional `ci.yml` changes (non-functional edits only (lines :434, :442 comment lines + :462 step-name line)); no new SHA pins required. The ACTION-PIN-GATE job is unaffected.
- **Zero-false-positive hard requirement:** `python3 bin/check-green-doc-tense` MUST exit 0
  after delivery. The implementer resolves string-literal FPs by converting multi-line
  fixtures to single-line form (Task 5). No skip-file pragma is permitted on the self-test
  file — it must remain in the scan set (PG-W84-010 self-application requirement).
- **`_is_comment_line(stripped, suffix)` semantics:** INCLUSIVE for its eligible set (returning
  True = ELIGIBLE TO BE FLAGGED). Language-scoped: `#` prefix is comment-eligible ONLY for
  `.py` files; Rust `#[attr]` lines are excluded (F-008). Extending suffix scoping makes Python
  comment lines scannable while preventing ~3625 Rust attribute false positives.
- **TIER-2 exclusion is policy, not a defect:** Asserting the tool does NOT flag `is expected
  to`, `falls through to`, `no .* arm`, `not yet implemented`, or `currently fails` is correct
  behavior per DF-GREEN-DOC-TENSE-SWEEP v6 (F-W86S-P2-006 / F-W86S-P3-001 rulings). No
  adversarial finding can overturn these PO rulings.

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Python | 3.10+ | `bin/` scripts use modern type syntax (CLAUDE.md) |
| No new Python packages | — | All changes use Python stdlib only |
| No new Cargo.toml deps | — | No Rust changes in this story |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `bin/check-green-doc-tense` | Modify | Rename `_collect_rust_files`→`_collect_source_files`, extend with `bin/*.py` glob; extend language-scoped `_is_comment_line(stripped, suffix)`; add Patterns 30–37 to `_VIOLATION_PATTERNS`; rewrite :4 headline sentence to "Scans tracked source files (tests/*.rs, src/**/*.rs, src/*.rs, and bin/*.py) for" (removes false "cfg(test) modules" parenthetical); update :467 docstring first line to "Collect scanned source files:" (with src/*.rs and bin/*.py bullets); update :472 "newly-added test files" to "newly-added source files"; update token list |
| `bin/test_check_green_doc_tense.py` | Modify | Runner `.py` extension support; 12 new BAD_CASES + 14 new GOOD_CASES (26 total); scrub lines ~258/~261; convert ~40 multi-line fixtures to single-line form |
| `CHANGELOG.md` | Modify | Add `[Unreleased]` entry for PG-W84-010 + PG-W85-003 (AC-183-005) |
| `.github/workflows/ci.yml` | Modify (comment lines :434 and :442 + step-name line :462 only) | Update scope descriptions to include `bin/*.py`; no functional job changes |
| `bin/test_lint_cycle_artifact.py` | Modify (Task 13 — 4 lines only) | Scrub confirmed-stale phrasing at lines :3, :5, :6, :125 |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`
**Note:** `tests/**/*` files must NOT be modified by this story. Reading/grepping test files
for the efficacy zero-FP check (AC-183-008) is read-only and permitted.

## Notes

- **DF-VALIDATION-001 status:** PG-W84-010 and PG-W85-003 are LOCAL-CARRY-FORWARD per
  `df-validation-2026-07-25.md §PG-W84-010` and `§PG-W85-003` (HIGH confidence). No upstream
  filing required. PG-W84-010 + PG-W85-003 LOCAL-CARRY-FORWARD dispositions per
  DF-VALIDATION-001 (mirroring STORY-182's Notes pattern).
- **No behavioral contract required:** E-11 convention.
- **tdd_mode: strict — automated RED procedure for this story:** `tdd_mode: strict` is satisfied
  by a genuine automated RED: add the 12 new BAD_CASES (Tasks 7 and 8a) BEFORE adding the 8
  new pattern tuples (Tasks 6 and 8b). Run `python3 bin/test_check_green_doc_tense.py` at that
  intermediate state — the self-test FAILS (RED) because BAD_CASES contain TIER-1 phrases
  that no yet-added pattern matches. Then add all 8 pattern tuples (Tasks 6 + 8b) and re-run —
  the self-test passes (GREEN). Required task ordering: Task 1 (baseline confirm) →
  Task 3 (suffix-aware `_is_comment_line` extension — required BEFORE RED, because 4 of
  the 12 BAD_CASES are `.py` 4-tuples that cannot clear until the `suffix` param exists;
  without Task 3 those cases never flag even after Tasks 6+8b add the patterns, making the
  GREEN checkpoint unreachable) → Tasks 7/8a (BAD/GOOD cases — creates RED) → run self-test
  (observe FAIL) → Tasks 6 + 8b (all 8 pattern tuples — clears RED) → run self-test (GREEN) →
  remaining Tasks 2/4/5/9–13.
- **Points rationale (5 pts):** Expanded from 3 pts (v1.0/v1.1) due to: (a) 8 new patterns
  (vs 2 originally; 11 proposed in v3, 3 falsified to TIER-2 in v4), (b) ~40 multi-line
  fixture conversions to single-line form, (c) 20+ new BAD/GOOD self-test cases. The
  40-fixture restructure is the primary scope driver. Point count unchanged from v1.2.
- **Develop PR:** All ACs can be batched in a single develop PR. CHANGELOG entry required
  (`bin/` changes trigger AC-158-001). No new CI step added.
  **Sibling story (STORY-182) also touches `.github/workflows/ci.yml`** in disjoint
  regions: STORY-183 edits comment lines :434/:442 + step-name line :462 only (non-functional);
  STORY-182 edits the test job ~:40-47 adding an additive `cargo test` run step.
  Merge order is irrelevant for conflict purposes — the two edit regions do not overlap — but
  line anchors are order-dependent: the ci.yml comment line anchors in this story
  (:434/:442/:462) are baseline develop@e8841d76, pre-STORY-182 — re-locate by content
  match if STORY-182 merged first. A rebase is required if both PRs are in flight
  simultaneously to avoid merge conflicts from adjacent line changes.
- **Story scope clarification (F-007, F-W86S-P3-001 + P16-006):** Retitled from "Full TIER-1 Token
  Coverage" to "TIER-1 Behavioral-Absence Token Coverage". Residual TIER-1 tokens NOT covered
  by this story:
  - §(d)/§(a) bare-word markers with phrase-level coverage only: `scaffold`, `uncalled`,
    `stub`, `skeleton` — phrase patterns (e.g., Pattern 25) cover the actionable form; bare
    words have ~91 legitimate uses and are excluded per the policy's phrase-level design
  - Bare `MUST FAIL` — TIER-2 per v6; tool catches specific phrases (Patterns 23-24) only
  - Three tokens falsified and moved to TIER-2 in v4 (F-W86S-P3-001): `no .* arm`,
    `not yet implemented`, `currently FAILS` — see Background §PG-W85-003 TIER-2 section
  - Pending-TIER-1 follow-up: `unimplemented!()` — 0 live hits per v4 grep (2026-07-25);
    pending addition after this story ships (track as follow-up)
  - Bare `Red Gate` / `RED GATE` / `todo!()` tokens: NOT covered — existing tuples are
    phrase-level by design (require context words); 32 live `RED GATE:` section headings
    across 10 test files are accepted residual (they are section labels, not behavioral-absence
    claims); phrase-level coverage is the deliberate FP-safety tradeoff; future tightening
    is a separate story.
- **Bare `RED` markers re-tiered TIER-2 per DF-GREEN-DOC-TENSE-SWEEP v6:** Pattern 30
  (`Expected RED:` heading with colon) is retained TIER-1; bare standalone `RED` markers
  without semantic phrase context are TIER-2 per the v6 ruling (F-W86S-P6-009/010).
- **Deferred scrub obligation (F-W86S-P6-009/010 + P16-003, v6 ruling):** Live stale sites deferred to next maintenance sweep: `iec104_analyzer_tests.rs:6271` and `modbus_detection_tests.rs:2472/:2480` — owner: next maintenance sweep. Additionally, `tests/iec104_analyzer_tests.rs:6948-6953` ("currently these fall through the `_` catch-all") — reword prescription: past-tense "before STORY-180, these fell through the `_` catch-all"; site absent from the deferred-scrub list because Pattern 31 (`currently\s+falls?\b`) has a contiguity blind spot: the interposed word "these" defeats the regex without triggering a match. **Contiguity limitation (P16-003):** Patterns 31 and 32 are defeated by interposed words between "currently" and the verb; widening is deferred to avoid FP risk against the 10 live `falls through to` TIER-2 sites; sibling patterns in `bin/check-green-doc-tense` at :325/:376/:455 that tolerate interposed words via broader intermediate-word matching are the eventual model for future tightening. Deferred site flows to the same DRIFT-stale-red-scrub vehicle (state-manager records at D-533).
- **FP budget for `src/*.rs` glob (F-P11-012, F-W86S-P12-006):** Under git's default pathspec
  matching, `src/*.rs` crosses directory separators and covers all `.rs` files under `src/`
  (not only top-level files); `src/**/*.rs` is retained for explicit-intent redundancy; git
  ls-files de-duplicates so no double-count. The top-level `src/*.rs` files were spot-checked
  as the primary newly-explicit entries: grep of `^\s*//` comment lines against all 36 TIER-1
  patterns → 0 matches as of commit `e8841d76`. No new false positives.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.13 | 2026-09-04 | story-writer | WAVE-86 PASS-24 REMEDIATION — F-W86S-P24-001 MED (Task 10 :4 bullet "Current text" quote corrected: removed spurious bold markdown (`**test files**`, `**cfg(test) modules**`) that did not appear in the live source at `bin/check-green-doc-tense:4` — verified live source reads plain "Scans tracked test files (tests/*.rs and src/**/*.rs cfg(test) modules) for" with no markdown bold, the only `**` on the line being the glob `src/**/*.rs`; Task 10's "Required rewrite" quote also de-bolded from `**source files**` to plain "source files", reconciling it with the FSR row at `:1265` which already prescribed the plain form; the duplicate parenthetical quote two lines below the "Current text" line was likewise de-bolded for consistency). ADDENDUM (2026-09-04, wave-86 human story-approval gate D-544, no version bump): `level: maintenance` → `level: feature` (E-11 convention alignment matching STORY-147/166/176). METADATA-ONLY classification change — `level` is not adversary-reviewed spec content and is not part of the `input-hash` (inputs: unchanged, hash unchanged at 9c9b12f). Adversarial convergence 3/3 (passes 25/26/27, BC-5.39.001 SATISFIED at v2.13) is PRESERVED — no re-convergence required. |
| 2.12 | 2026-07-28 | story-writer | WAVE-86 PASS-22 REMEDIATION — F-W86S-P22-004 LOW (AC-183-006 end-to-end assertion paraphrase at `:519-523`: "returns a set containing `bin/test_check_green_doc_tense.py`" contradicted AC-183-001 on two counts: (1) type: AC-183-001 and Task 2 both say "path list (list[Path])", not "set"; source confirms `_collect_source_files` returns `list[Path]`; (2) comparison form: AC-183-001 explicitly says compare by `.name`, never by repo-relative string; paraphrase described the forbidden repo-relative string form; fixed to "`_collect_source_files(repo_root)` returns a `list[Path]` containing an entry whose `.name` equals `test_check_green_doc_tense.py` — compare by `.name`, never by repo-relative string"); N-2 NIT (AC-183-001 final Then-clause: standalone `any(p.suffix == ".py" for p in files)` class assertion replaced with explanatory note that the `.name == "test_check_green_doc_tense.py"` check from the When clause already entails `.suffix == ".py"` membership — Task 2 prescribes exactly two checks, not three; the third was redundant); AUDIT-5 hardening (AC-183-005 Verification fence: `set -euo pipefail` added — already fail-closed: `bin/changelog-gate-check` exits 1 on empty stdin AND on no-plus-lines; hardened for consistency, NOT fixing a live false-GREEN per D-530; AUDIT5 now 0). |
| 2.11 | 2026-07-28 | story-writer | WAVE-86 PASS-21 REMEDIATION — F-W86S-P21-001 MED (Pattern 33 comment block reworded: eliminated the contiguous four-token phrase `falls to the wildcard` which Pattern 33's own regex `falls\s+to\s+the\s+wildcard` would match, causing a self-referential-flag hazard; replacement states "fires only on the contiguous four-token phrase (verb + `to the wildcard` with no intervening word)" without placing the flagged phrase contiguously; restores compliance with the comment-safety discipline established at v2.4 F-W86S-P14-004); F-W86S-P21-009 LOW (Task 10 ci.yml sweep: `:436` "test files receive..." adjudicated as historical problem-origin narrative — describes where the anti-pattern was first observed, not the current gate scope (which is at :442); adjudication note added to Task 10 to prevent re-raising; AC predicate `grep -c 'in test files'` correctly does not match `:436`'s "test files receive" phrase); F-W86S-P21-010 NIT (Task 5 GOOD_CASES residual count: ~46 → ~45 — hand-verified by adversary: 45 `//`-prefixed lines in the GOOD_CASES range [332,632]). |
| 2.10 | 2026-07-27 | story-writer | WAVE-86 PASS-20 REMEDIATION — F-001 MED (Task 2 `repo_root` derivation paragraph: wrong attribution ("hermetic test functions monkey-patch `_find_repo_root`") replaced — Task 9 uses subprocess isolation, NOT monkey-patching; monkey-patching of `_find_repo_root` is pre-existing in AC-158-005 (`:698-726`) and AC-162-003 (`:858-905`) and is NOT introduced by Task 9); F-003 MED (AC-183-001 Verification: ci.yml scope-update predicates added — `test "$(grep -c 'in test files' .github/workflows/ci.yml)" -eq 0`, `grep -qF 'src/*.rs'`, `grep -qF 'bin/*.py'`); F-005 MED (AC-183-002 Verification: non-gating `echo "Exit code: $?"` forms replaced with `set -euo pipefail` at fence head — `set -e` causes non-zero exit to abort without needing echo); F-006 MED (seven missing `set -euo pipefail` first lines added: AC-183-001 Verification, AC-183-002 Verification, AC-183-006 Verification, AC-183-008 Verification, AC-183-009 grep block, AC-183-009 Verification, Task 12 bash block); F-009 MED (Task 9 hermetic fixture: single-line mandate added for `#`-prefixed content — multi-line would make the fixture's own `# currently asserts` line scan-eligible via Pattern 32 after AC-183-002 extension; mechanism explained); F-011 LOW (Task 4 sibling-line safety explanation corrected: `:213` safe by `\b`-escape mechanism; `:259`/`:260` safe by a DIFFERENT reason — Patterns 27/28 require `exposes\|is a\|are` verb before `compile-only`, which is absent; all three deliberately left unchanged); F-012 LOW (Task 5: explicit GOOD_CASES deliberate-design note added — GOOD_CASES are NOT converted; deliberate choice because a matching GOOD_CASE is a test-design error; residual of ~46 `//`-prefixed scan-eligible lines noted); F-014 LOW (AC-183-007 Pattern 33 comment: "Pattern fires when 'through' is absent between the verb and the destination phrase" overstates — replaced with "Pattern fires only on the contiguous phrase `falls to the wildcard`; TIER-2 form `falls through to` not matched because interposed `through` breaks `falls\s+to` adjacency"). |
| 2.9 | 2026-07-27 | story-writer | WAVE-86 PASS-19 REMEDIATION — F-005 MED (Task 9 subprocess capture made explicit: `capture_output=True, text=True` required in subprocess.run call; `combined = proc.stdout + proc.stderr` added; all output assertions changed from checking "output" to checking `combined`; added note that bin/check-green-doc-tense:558-562 writes "no tracked source files found" to `file=sys.stderr` so it appears in proc.stderr/combined; 4th positively-discriminating assertion added: `len(_collect_source_files(tmp)) == 1`); F-006 MED (Task 10 :4 bullet expanded: now prescribes rewriting the FULL headline sentence — not only the glob list; "test files" noun phrase → "source files"; "(tests/*.rs and src/**/*.rs cfg(test) modules)" → "(tests/*.rs, src/**/*.rs, src/*.rs, and bin/*.py)" with explicit note that "cfg(test) modules" parenthetical is FALSE post-story and MUST be removed; two new bullets added: :467 docstring first line "Collect test-scope Rust files:" → "Collect scanned source files:" + src/*.rs and bin/*.py bullets; :472 "newly-added test files" → "newly-added source files"; FSR Notes for bin/check-green-doc-tense updated to enumerate :4/:467/:472 loci); F-007 MED (Task 4 safety criterion rewritten from phrase-level to match-level rule: MUST NOT match any of 36 patterns; mechanism documented — writing pattern in regex-literal form places \b before trigger word so Pattern 29 \b assertion cannot fire; Do NOT remove \b escapes warning added; sibling lines :213/:259 already safe by same mechanism noted); F-008 MED (Task 9 tempfile import rationale corrected: "top-level import would shadow function-scope import" is backwards — the function-local binding at :640 shadows the module-level name, not the reverse); F-009 MED (Task 2 CHANGELOG preservation note extended: both CHANGELOG.md loci covered — :741 cites _collect_rust_files by name (STORY-162/176 provenance); :851 cites pre-rename error-string prose "no tracked Rust files are found" (AC-158-005 zero-file guard as shipped); both must remain as-is per DF-SIBLING-SWEEP-001); F-010 MED (Task 2 "8 prose/comment sites" corrected to "7 _collect_rust_files prose/comment sites (:688,:705,:711,:718,:839,:843,:891) plus 1 rust_files-only prose site at :721 (failure-message string)"). |
| 2.8 | 2026-07-27 | story-writer | WAVE-86 PASS-18 REMEDIATION — P18-001 MED (Task 10 :212-215 bullet relabeled from "_is_comment_line() docstring" to "module-level pattern-registry comment block introducing _VIOLATION_PATTERNS at :217"; prescription updated from suffix-aware _is_comment_line docstring rewrite to updating the comment block to state patterns match "// in all scanned files; # in .py files"; added clarification that _is_comment_line() at :~460 has its own docstring at :461 rewritten wholesale by AC-183-002 — no contradiction with Task 1's :~460 reference); P18-002 MED (Task 10 bullet 1 split into 4 bullets: (i) :4 only for glob-scope text update; (ii) :26-30 for comment-marker claim fix — "lines whose non-whitespace content starts with // or //!" → "matches // comment lines in all scanned files; # comment lines in .py files"; (iii) :31-85 for token list update; (iv) :90 ALLOWLIST heading — no glob update required, noted explicitly); P18-007 LOW (Task 9 negative assertion added: process output must NOT contain "no tracked source files found" to distinguish genuine violation exit-1 from empty-collection guard exit-1 at main() :557-563); P18-009 LOW (token budget: ~950-960 lines corrected to ~1000-1050 lines; baseline 914 + net delta ≈ 1000-1050). |
| 2.7 | 2026-07-26 | story-writer | WAVE-86 PASS-17 REMEDIATION — P17-006 MED (Task 10 sweep: new bullet added for lines :212-215 `_is_comment_line()` docstring — current "starts with `//` or `//!`" claim is false after AC-183-002; rewrite with suffix-aware description); P17-007a MED (AC-183-001 :228 placement instruction: "Place the assertions after :905" → "Insert the assertions immediately after the `finally` block ending at :905 and BEFORE the `print()` at :907 — ensures checks feed passed/failures counters and Results: line"); P17-007b MED (Task 9 :1017-1019 placement instruction corrected to match: "Insert both immediately after the `finally` block ending at :905 and BEFORE the `print()` at :907"); P17-011 MED (Task 9 hermetic assert/exit checks respecified in runner convention: `  PASS  [hermetic-e2e: exit 1 on violation]` / `  FAIL  [...]` with counters for both exit-code and output-contents checks); P17-012 LOW (AC-183-001 baseline count: split into pre-story `git ls-files -- tests/*.rs src/**/*.rs \| wc -l` and post-story `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py \| wc -l` — prior single command incorrectly included `src/*.rs` in the pre-story baseline). |
| 2.6 | 2026-07-26 | story-writer | WAVE-86 PASS-16 REMEDIATION — F-W86S-P16-003 MED (Notes §Deferred scrub obligation extended with P16-003 contiguity blind spot: `tests/iec104_analyzer_tests.rs:6948-6953` "currently these fall through the `_` catch-all" added; reword prescription (past-tense: "before STORY-180, these fell through the `_` catch-all"); contiguity limitation of Patterns 31/32 documented — interposed words defeat `currently\s+falls?\b`/`currently\s+asserts?\b`; widening deferred to avoid FP risk against 10 TIER-2 `falls through to` sites; sibling patterns at :325/:376/:455 cited as eventual model; D-533 vehicle noted); F-W86S-P16-004 MED (Pattern 35 FP-note comment at :562-564 rewrote to eliminate contiguous "currently has no" — replaced with "the pattern also matches a hyphenated no-prefix token immediately after the phrase, because \b fires at the hyphen (e.g. a 'no-op' continuation). 0 live hits."; verified no `# `-line in replacement matches any of 36 patterns); F-W86S-P16-005 MED (Task 2 :872 wrapped single-space `PASS [_collect_source_files:` fixed to `  PASS  [_collect_source_files: test_check_green_doc_tense.py found]` — collapsed onto one line; full story `PASS [`/`FAIL [` sweep: 0 remaining single-space forms); F-W86S-P16-006 MED (Notes §Story scope clarification: new bullet added for bare `Red Gate`/`RED GATE`/`todo!()` tokens — NOT covered, phrase-level design deliberate, 32 live `RED GATE:` headings across 10 test files are accepted residual section labels; AC-183-008 heading/body amended to scope as "phrase-level" only with P16-006 note referencing §Story scope clarification); P16-009 LOW (Task 9 :1023-1024: `git -C <tmp> add bin/check-green-doc-tense` annotated as optional — extension-less file never collected by suffix-filter; only bin/violating.py must be indexed); NIT-1 ("path set" → "path list (list[Path])" at both loci :217/:871 via replace_all). |
| 2.5 | 2026-07-26 | story-writer | WAVE-86 PASS-15 REMEDIATION — F-W86S-P15-005 MED (two loci corrected: (a) AC-183-001 :222-226 repo_root note rewritten — Task 9 subprocess cannot see parent patches; monkey-patchers are PRE-EXISTING AC-158-005 (:698-726, patch :704) and AC-162-003 (:858-905, patch :871); placement load-bearing only for in-process _collect_source_files assertions — hermetic tempdir → empty set → spurious FAIL if placed inside monkey-patch block; (b) Task 9 :998-1003 placement constraint rewritten — non-hermetic assertions must be outside :698-726/:858-905; subprocess cannot see parent patches; subprocess placement relative to finally blocks has no effect); F-W86S-P15-006 LOW (PASS/FAIL strings at :215-233 and :863-880 corrected to runner convention: `  PASS  [label]` / `  FAIL  [label]` — 2-space indent, 2 spaces after PASS/FAIL; 4 occurrences updated via replace_all); F-W86S-P15-007 LOW (Task 9 imports :1000-1001: subprocess and shutil are new top-level imports; tempfile already at function scope :640 — top-level add shadows; note added); F-W86S-P15-011 LOW (Pattern 35 comment: known theoretical FP added — `currently\s+has\s+no\b` matches "currently has no-op" at \b/hyphen boundary; 0 live hits wave-86); NIT-3 (Task 1 :851 line count: wc -l confirms 913 — value already correct, no change). |
| 2.4 | 2026-07-26 | story-writer | WAVE-86 PASS-14 REMEDIATION — F-W86S-P14-001 MED (Task 8 split into 8a/8b: 8a covers BAD_CASES+GOOD_CASES as pre-RED step consumed with Task 7; 8b appends 6 Patterns 32-37 tuples as GREEN step run with Task 6; Notes ordering updated: "Tasks 7/8a (BAD/GOOD cases — RED) → Tasks 6+8b (all 8 pattern tuples — GREEN)"; "Tasks 7 and 8" → "Tasks 7 and 8a"; "Task 6" → "Tasks 6 + 8b" in GREEN clause; GREEN checkpoint unreachable note updated to cite Tasks 6+8b); F-W86S-P14-004 LOW (Pattern 33 comment :539-540 reworded: eliminated contiguous "falls to the wildcard" across line wrap — new 5-line comment describes discriminator without placing "falls" adjacent to "to the wildcard" on any single line); F-W86S-P14-005 LOW (two _collect_source_files self-test assertions at :215-233 and Task 2 :858-870 respecified in runner's PASS/FAIL convention — each check prints PASS/FAIL with a label and increments passed/failures counters; no bare asserts); NIT-1 (P14-007: Task 10 bullets 2+3 merged into single bullet — :87-88 rewrite content combined, :97 range-label instruction kept distinct at end); NIT-2 (P14-008: Notes :1178 "TIER-2 per v4" → "TIER-2 per v6"). |
| 2.3 | 2026-07-26 | story-writer | WAVE-86 PASS-13 REMEDIATION — F-W86S-P13-001 HIGH (AC-183-001 :228-230: false "both globs cover same files" claim corrected — `src/**/*.rs` requires literal `/` after star segment, NEVER matches top-level src/*.rs (10 files); `src/*.rs` (star crosses /) covers all src/ and strictly SUBSUMES `src/**/*.rs`; `src/*.rs` is LOAD-BEARING and MUST NOT be dropped; consistent with six already-correct loci); F-W86S-P13-003 MED (Pattern 33 comment :534: wrong discriminator "bare form without to the wildcard" replaced with correct one: intervening word "through" breaks `falls\s+to\s+the\s+wildcard` pattern); F-W86S-P13-004 MED (AC-183-009: two inverted-gate loci fixed — grep -c exits 1 when count is 0, replaced with `test "$(grep -c ...)" -eq 0` gating form at both :743-748 and :762-765; "return 0" → "print 0"); F-W86S-P13-005 MED (merge-order note: "irrelevant" scoped to conflict purposes; ci.yml line anchors labeled "baseline develop@e8841d76, pre-STORY-182 — re-locate by content match if STORY-182 merged first"); F-W86S-P13-006 MED (convergence-report.md path-qualified at all 7 loci to `.factory/cycles/wave-085/STORY-180/convergence-report.md`; file added to traces_to); F-W86S-P13-009 LOW (Task 9: required imports subprocess/shutil/tempfile stated as explicit deliverables; placement constraint stated — hermetic section MUST be outside monkey-patch `finally` blocks; placement is load-bearing); F-W86S-P13-010 LOW (Pattern 34 regex: `doesn'?t` → `doesn['']?t` typographic-apostrophe variant); F-W86S-P13-011 LOW (Task 10: bin/check-green-doc-tense:97 added to sweep list with range-update instruction); F-W86S-P13-012 LOW (Task 2 and Task 10 tool-file anchors :87-88/:466-474/:556/:558-560/:577 labeled "baseline develop@e8841d76 (pre-edit) — re-locate by content after each insertion"); F-W86S-P13-013 LOW (Pattern 34 GOOD annotation :593: no-literal-phrase discipline — `no "exist yet"` → "negative-capability phrase absent"); F-W86S-P13-014 LOW (AC-183-008: all 10 "falls through to" sites enumerated, not just 4); NIT-1 (docstring: "// or //! (inner doc)" → "// (including /// outer doc and //! inner doc)"); NIT-2 (token budget: ~620 lines → ~690-700); NIT-3 (:393: "catches both singular and plural" → "matches both fall and falls (verb inflection)"); process-gap (EC-010 extended: .py files outside bin/ added as also OUT OF SCOPE; DRIFT-py-surface-outside-bin noted). |
| 2.2 | 2026-07-26 | story-writer | WAVE-86 PASS-12 REMEDIATION — F-W86S-P12-001 HIGH (AC-183-007 fixture-block #-annotations at 5 loci reworded to DESCRIBE without quoting literal flagged phrases: Pattern 32 GOOD → "allowlist — present-tense assertion-state phrase absent"; Pattern 33 GOOD → "allowlist — wildcard-arm fallthrough phrase absent"; Pattern 35 GOOD → "allowlist — present-tense absence claim absent"; Pattern 36 GOOD → "allowlist — passive stub-status phrase absent"; Pattern 37 GOOD → "allowlist — conditional present-RED tense claim absent"; sweep clause added to AC-183-007 mandating no-literal-phrase discipline for every # comment prescribed for bin/test_check_green_doc_tense.py); F-W86S-P12-003 MED (tdd_mode RED ordering: Task 3 inserted into pre-RED segment — "Task 1 → Task 3 → Tasks 7/8 → RED → Task 6 → GREEN → remaining Tasks 2/4/5/9-13"; rationale: 4 of 12 BAD_CASES are .py 4-tuples that cannot clear until Task 3 suffix param exists); F-W86S-P12-005 MED (AC-183-009 Then-clause: 3 mechanical grep predicates added — grep -c "RED GATE version", "MUST FAIL until bin/lint-cycle-artifact", "TC1–TC8" each MUST be 0; stated as failing exactly when Task 13 is skipped); F-W86S-P12-006 LOW (git pathspec semantics corrected at 5 loci: AC-183-001 Given clause, AC-183-001 When clause, AC-183-001 assertion note, Task 2 invocation rationale, Task 2 docstring scope — false claim that src/**/*.rs has a top-level blind spot replaced with correct semantics: src/*.rs under git wildmatch crosses directory separators and covers all of src/, src/**/*.rs retained for explicit-intent redundancy, git ls-files de-duplicates; Notes FP budget note updated); F-W86S-P12-007 LOW (Task 10: instruction added to rewrite bin/check-green-doc-tense:87-88 to describe suffix-aware comment anchoring // + # and note string-literal false-positives are flagged by design per EC-005/Task 5); F-W86S-P12-010 LOW (AC-183-009: "covers both (a)/(b)" → "covers all three: (a) required-status-check registration, (b) bin/test_lint_cycle_artifact.py step, (c) bin/test_compute_input_hash.py step"; "until part (b) lands" → "until parts (b) and (c) land"; Verification comment updated); NIT-1 (Task 2 :466-473 → :466-474); NIT-2 (Task 10 "lines 2–4 scope text" → "line 4 scope text"); NIT-3 (Background §PG-W85-003 "sole authoritative source" softened to "primary citation; citation is non-exhaustive; zero-live-hit greps are the mechanical backstop"); F-W86S-P12-009 LOW (Notes §Develop PR: sibling story note added — STORY-182 also touches ci.yml in disjoint regions; rebase required if both PRs in flight). |
| 2.1 | 2026-07-26 | story-writer | WAVE-86 PASS-11 REMEDIATION — F-P11-005 MED (AC-183-001 + Task 2: collapsed two-invocation pattern to single merged `git ls-files -- tests/*.rs src/**/*.rs src/*.rs bin/*.py`; explicit no-dedup note; Task 2 docstring scope updated to cite single invocation); NIT-fix 2 (AC-183-001 + Task 2: pass-message claim corrected to variable-rename-only); F-P11-014 LOW (AC-183-001: "N > previous Rust-only count" replaced with baseline-derivation command + class assertion `any(p.suffix == ".py" for p in files)`); F-P11-006 MED (AC-183-009: PG-W84-012 attribution extended to cite D-525 per F-W86S-P9-012, covering both (a) required-status-check registration and (b) bin/test_lint_cycle_artifact.py step; selftest-runs-locally-until-(b)-lands note added; Then clause + verification comment updated); NIT-fix 1 (ACR: "comment-only updates" → "non-functional edits only (lines :434, :442 comment lines + :462 step-name line)"); F-P11-007 MED (tdd_mode note rewritten: prescribes real automated RED via Tasks 7/8 BAD_CASES-first before Task 6 patterns; inverted-order manual-RED text removed); F-P11-012 LOW (Notes: zero-FP budget note added — 10 src/*.rs files audited, 0 matches against 36 patterns as of e8841d76); F-P11-013 LOW (Task 9: `git add <tmp>/...` → `git -C <tmp> add ...` with cwd note, both occurrences). |
| 2.0 | 2026-07-26 | story-writer | WAVE-86 PASS-10 REMEDIATION — F-P10-002 MED (src/*.rs glob widening not propagated to three scope-prose sweep instructions: Task 2 :808-812 docstring scope updated to "tests/*.rs, src/**/*.rs, src/*.rs, and bin/*.py"; Task 10 :931-934 scope declarations updated to include src/*.rs; Task 10 :939-943 ci.yml comment instructions updated to enumerate src/*.rs); F-P10-011 LOW (AC-183-001 :211-220 + Task 2 :800-807 self-test assertions clarified: repo_root derivation stated explicitly as `mod._find_repo_root(Path(mod.__file__).resolve().parent)` with hermetic-section monkey-patch note; mitre.rs-literal assertion replaced with class assertion `any(p.parent.name == "src" and p.suffix == ".rs" for p in files)` keeping mitre.rs as illustrative prose example); F-P10-010 LOW (tdd_mode: strict E-11 template note added in Notes). |
| 1.9 | 2026-07-26 | story-writer | WAVE-86 PASS-9 REMEDIATION — F-P9-009 MED (AC-183-001 + Task 2: pathspec `src/**/*.rs` extended with `src/*.rs` to cover top-level src/*.rs files that git wildmatch `**` skips; self-test assertion added verifying a top-level src file is in the scanned set; points unchanged at 5); F-P9-010 MED (AC-183-007: explicit fixture strings added for all 8 BAD_CASE entries lacking them (Patterns 32 .rs/.py, 33 .rs/.py, 34 .rs, 35 .rs, 36 .rs, 37 .rs) and all 6 GOOD_CASE entries lacking them (Patterns 32-37); first-match-wins break-on-first pattern-priority constraint documented); F-P9-012 LOW (AC-183-009 added: `python3 bin/test_lint_cycle_artifact.py` MUST pass locally at delivery; CI wiring deferred to PG-W84-012 ops task); NIT-03 (AC-183-003 verification: added explicit `MUST exit non-zero` assertion for gate behavior when encountering a Pattern 30 match — aligns modal verb with AC-183-001 language). |
| 1.8 | 2026-07-26 | story-writer | WAVE-86 PASS-8 REMEDIATION — F-009 MED (bin/test_lint_cycle_artifact.py row added to Architecture Mapping table and FSR table: "Modify (Task 13 — 4 lines only)"); F-010 MED (Task 13 updated: scrub list extended to :3/:5/:6/:125; "Three lines"→"Four lines"; line :125 is a #-comment line newly scan-eligible under this story); F-011 LOW (EC-011 :655 confirmed-stale scrub targets updated from :3/:6 to :3/:5/:6/:125; :125 noted as #-comment line); F-012 LOW (Task 2 adjudication note extended: "Similarly, historical references in delivered story specs (STORY-158, STORY-162) are preserved as shipped spec provenance — do NOT sweep .factory/stories/ history."). |
| 1.7 | 2026-07-26 | story-writer | WAVE-86 PASS-7 REMEDIATION — F-002 HIGH (17 governing cites DF-GREEN-DOC-TENSE-SWEEP v5→v6 via replace_all; 3 standalone: "The v5 two-tier"→v6, "(F-W86S-P4-004, v5)"→v6, "TIER-1 per v5"→v6; grep confirms zero residual governing cites; historical provenance at :856 "per v5 classification" preserved); F-003 HIGH (Task 6: deleted "quoted phrases prevent flagging" claim; replaced with "In-source `# Pattern NN:` comments MUST NOT contain the literal flagged phrase (quoting does not prevent regex match — see Task 4)"); F-004 MED (Task 13 :847/:848 TC1–TC17→TC1–TC21 both occurrences, "and 21 self-tests"→"(21 self-tests)"); F-005 MED (AC-183-001 :201 + Task 2 heading :701: "13 references"→"13 `_collect_rust_files` sites + 1 `rust_files` prose site at :721 (14 total)"; :721 added to prose list with type clarification); F-006 MED (4 ci.yml scope loci :187/:635/:882/:911: "comment line ~442 only"→"comment lines :434 and :442 + step-name line :462 (non-functional edits only)"); F-010 LOW (Task 2 :693: "path set containing `bin/test_check_green_doc_tense.py`"→.name-based comparison with fragility note); F-012 LOW (AC-183-005 :431-432: two-dot→three-dot `origin/develop...HEAD -- CHANGELOG.md`; two-dot divergence noted); F-013 LOW (Task 10 :821: added ":577 ('in test files' failure summary) and :581–585 (explanatory prose)" to bin/check-green-doc-tense scope-prose sweep); F-014 LOW (EC-011 :655 + Task 13 :856: "Log advisory STATE.md drift row DRIFT-docstring-scan"→"DRIFT-docstring-scan row is recorded by state-manager (already present in STATE.md; no action in this story's PR)"). |
| 1.6 | 2026-07-25 | story-writer | WAVE-86 PASS-6 REMEDIATION — F-005 MED (AC-183-001: removed "and 1 reference in CHANGELOG.md (see Task 2)" — historical entry preserved per DF-SIBLING-SWEEP-001, outside the 13 test-file refs); F-006 MED (Task 9: removed "and create a minimal commit so the index is non-empty" — git add only; no commit required for git ls-files); F-007 MED (Task 9 FAIL format: FAIL [bin/violating.py:1]: Pattern 32 — not "FAIL  bin/violating.py: Pattern 32"); F-008 MED (Task 13 TC count: TC1–21 (21 self-tests)); F-009+F-010 PO PROPAGATION (Notes: bare RED markers re-tiered TIER-2 per DF-GREEN-DOC-TENSE-SWEEP v6; Pattern 30 retained TIER-1; deferred scrub obligation F-W86S-P6-009/010 added — iec104_analyzer_tests.rs:6271 + modbus_detection_tests.rs:2472/:2480; owner: next maintenance sweep); F-011 MED (Task 2: add :721 prose site — failure-message citing if not rust_files: guard); F-012 MED (input-hash NOTE: "Stored value is the canonical Python hash (9c9b12f); bash hook reports 5598136 — advisory only per PG-HASH-HOOK-DIVERGENCE"); F-013 MED (traces_to: add CHANGELOG.md + .github/workflows/ci.yml + bin/test_lint_cycle_artifact.py); F-017 LOW (Task 10 ci.yml sweep: :434 job comment + :462 step name added as stale prose sites); F-018 LOW (Task 5 quote-escaping: "Two fixtures in BAD_CASES (:91, :97)" — :402 is GOOD_CASE); F-019 LOW (Task 13: "state-manager records DRIFT-docstring-scan"); F-020 NIT (AC-183-007: 37 items / 36 tuples — item 5 shares tuple 4). |
| 1.5 | 2026-07-25 | story-writer | WAVE-86 PASS-5 REMEDIATION — F-002 HIGH (Task 9 hermetic harness fix: copy script into `<tmp>/bin/check-green-doc-tense` inside the `git init`ed temp repo so `_find_repo_root` walks from the script's location upward to `<tmp>`; no env override, no functional tool change); F-003 HIGH (Task 9: `git add bin/violating.py` required so git ls-files index sees the file; assert literal violation-output prefix — FAIL line naming bin/violating.py + pattern label — not just exit code); F-004 MED (Task 2 CHANGELOG:741 rename: per DF-SIBLING-SWEEP-001 preserve historical entry; parenthetical annotation optional only; do NOT alter historical name); F-005 MED (Background .py file list: `bin/test_changelog_gate_check.py` → `bin/test_changelog_gate_content.py`); F-006 MED (Task 6: deleted false "quoted phrases prevent flagging" claim; replaced with correct rule — in-source Pattern NN comments must NOT contain the literal flagged phrase, since regex matches inside quotes equally); F-007 MED (EC-011: "Task 12" → "Task 13" for docstring scrub target); F-008 MED (v4 → v5 at 9 governing sites: AC-183-007 intro :482/:484, AC-183-007 TIER-2 GOOD_CASEs :534/:540/:546, AC-183-008 :568/:590, Architecture Compliance Rules :850; historical provenance cites at :505/:536/:542/:548/:882-896/:112 unchanged); F-009 MED (P4-002 → P4-004 ruling ID at :287/:362/:391 — number-agnostic registry ruling not docstring-scope ruling); F-010 MED (Task 13: add line 5 scrub — `bin/test_lint_cycle_artifact.py:5` stale count "TC1–TC8 implement all eight test cases" reworded; file now has TC1..TC17 / 21 self-tests); F-017 MED (Task 8: suffix-scoping negative-guard GOOD_CASE from AC-183-006 assigned — runner writes to `good_{n}.rs`, gate scans as .rs file and MUST NOT flag it); F-019 LOW (GOOD_CASES arithmetic: 13→14; totals 25→26 in Architecture Mapping and FSR; 14 = P30 allowlist + P31 allowlist + P31 zero-FP + suffix-scoping negative guard + 6×P32-37 + 3 TIER-2 + is-expected-to efficacy); F-020 LOW (Task 1 test file line count: 914→913 lines); F-021 LOW (rename sweep extended to tool's own surfaces: `_collect` docstring :466-473, `rust_files` local :556, "no tracked Rust files found" error string :558-560 — all become inaccurate with .py in scope); F-022 LOW (:121-122 "may be added as Pattern 30" → "is added as Pattern 30" — AC-183-003 mandates it); F-025 LOW (Task 13 replacement text: drops literal "RED GATE" token from example — uses past-tense provenance phrasing); F-027 NIT (AC-183-001 set-membership assertion: compare resolved absolute paths or basenames, not repo-relative strings — collector returns list[Path] absolute). |
| 1.4 | 2026-07-25 | story-writer | WAVE-86 PASS-4 REMEDIATION — F-002 HIGH (new Task added: scrub confirmed-stale docstring sites bin/test_lint_cycle_artifact.py:3/:6 — reword RED-GATE phrasing to past-tense provenance or remove; verify python3 bin/test_lint_cycle_artifact.py exits 0/21 passed after scrub; EC-011 updated: policy v5 known-residual class cited, STATE.md drift row DRIFT-docstring-scan noted, NOT-stale verdicts recorded for test_gitignore_mutants_glob.py:12 + test_validate_citations.py:645,647); F-004 HIGH (policy now number-agnostic per v5 F-W86S-P4-004 ruling: v4→v5 throughout, all claims that policy prescribes pattern numbers removed, pattern numbering stated as owned by _VIOLATION_PATTERNS tool registry, policy names tokens by literal text only); F-005 MED (stale v3 citation residue at ~:276-277/:353/:382 updated to cite v5 or version-agnostically with F-W86S-P2-006/P3-001/P4-002 rulings); F-006 MED (in-source comment self-flag prescriptions corrected: quoted phrases do NOT prevent regex match; all Pattern NN: inline comments reworded to exclude the literal flagged phrase; e.g. Pattern 30: 'stale pre-pass RED-heading with colon', Pattern 32: 'present-tense assertion-state claim', etc.); F-007 MED (bin/*.py glob membership corrected: bin/compute-input-hash and bin/changelog-gate-check are NOT .py files and removed from examples; .py set is exactly 6 test_*.py files); F-008 MED (rename-site arithmetic corrected: 6 FUNCTIONAL :699/:707/:726/:859/:872/:905 + 7 PROSE :688/:705/:711/:718/:839/:843/:891 = 13 total; fabricated :899 cite removed; "4 functional / 8 prose" corrected to "6 functional / 7 prose"); F-009 MED (suffix-scoping NEGATIVE guard GOOD_CASE added: `# Expected RED:` in .rs file written as good_{n}.rs NOT flagged — proves # eligibility is .py-scoped only); F-010 MED [process-gap] (E2E positive-coverage self-test task added: hermetic temp git repo containing bin/violating.py with TIER-1 phrase → collect→scan→exit path → assert exit 1 + violation reported; df-validation self-application smoke row); F-017 LOW (mis-anchored citation corrected: tests/main_story_089_tests.rs:648→:890); F-018 LOW (FSR + Architecture Mapping case-count corrected: 12 new BAD_CASES + 13 new GOOD_CASES = 25 total); F-019 LOW (AC-183-005 annotation reworded: changelog-gate-check verifies ADDED non-heading content lines in the diff, not section placement); F-020 LOW (authority claim scoped: convergence-report.md lines 63-66 are authoritative for specific D-506 tokens; PG-W85-003 paragraph at :68-70 and lesson summary both carried the broader/incorrect labels). |
| 1.3 | 2026-07-25 | story-writer | WAVE-86 PASS-3 REMEDIATION — F-007 CRIT (retitle: "Full TIER-1 Token Coverage"→"TIER-1 Behavioral-Absence Token Coverage"; residual exclusions documented in Notes; title swept to STORY-INDEX); PO v4 PROPAGATION: F-007/F-W86S-P3-001 (Patterns 34/36/40 FALSIFIED→TIER-2; AC-183-007 redesigned: 9 patterns→6 patterns 32-37 with renumbering; 3 TIER-2 GOOD_CASEs added asserting NOT-flagged; AC-183-008 efficacy scope 30-40→30-37; 28+11=39→28+8=36 total; Background PG-W85-003 updated); F-002 HIGH (AC-183-001 + Task 2: rename mandate only—no "(or annotate)" alternative; all 13 self-test sites enumerated: 4 functional +:8 prose +1 CHANGELOG); F-003 HIGH (AC-183-005 verification: `bin/changelog-gate-check CHANGELOG.md`→`git diff origin/develop -- CHANGELOG.md \| bin/changelog-gate-check`); F-004 HIGH (EC-011 added: docstrings are invisible; 4 known-residual sites cited; PG-W84-010 scope narrowed to comment-line prose); F-008 MED (AC-183-002: _is_comment_line() now language-scoped with suffix param; `#` prefix = comment only for .py; EC-012 added: Rust attribute lines excluded); F-017 LOW (falls through to count 12→10 per v4 grep); F-018 LOW (Task 5: quote-escaping note for 3 fixtures with literal `"`); F-019 LOW (Task 9: mislabel fixed—lines 87-88 = comment-anchoring note, ALLOWLIST heading at :90); F-020 NIT (Task 7: BAD_CASES type annotation 4-tuple variant + # type: ignore noted); F-021 MED (AC-183-006 re-anchored on suffix-scoped mechanism + e2e assertion added); Architecture Compliance v3→v4; FSR Patterns 30-40→30-37; Notes: residual exclusions + unimplemented!() follow-up. |
| 1.2 | 2026-07-25 | story-writer | WAVE-86 PASS-2 REMEDIATION — F-001 CRIT (Pattern redesign: ground truth corrected to convergence-report lines 63-66: 9 D-506 sites used `currently asserts` (TIER-1→Pattern 32) and `is expected to` (TIER-2→manual sweep only); all missing TIER-1 tokens from DF-GREEN-DOC-TENSE-SWEEP v3 added as Patterns 32-40; AC-183-007 new; old AC-183-007 → AC-183-008 rewritten); F-002 HIGH (AC-183-001 adds positive .py coverage assertion for `_collect_source_files()`); F-005 HIGH (Task 5 Option B STRUCK — self-test file must remain in scan set per PG-W84-010; single-line format is the correct fix); F-006 MED (Pattern 31 GOOD_CASE updated with verbatim F-W86S-P2-006 PO ruling; governing policy reference added to Background); F-011 MED (Task 5 corrected: scope 40 //‐lines + 2 # lines = 42 violations; single-line string format is the correct mechanic; points 3→5); F-015 MED (extension-less bin/ executables documented as out-of-scope with rationale; in-source Pattern NN comment phrasing note); F-018 LOW (token budget updated: test file ~950-960 lines after additions); F-020 LOW (Notes section added with DF-VALIDATION-001 disposition); F-022 LOW (tests/**/* removed from Forbidden; read-only access note added); F-023 NIT (AC-183-005 verification uses `bin/changelog-gate-check` citation). |
| 1.1 | 2026-07-25 | story-writer | WAVE-86 PASS-1 REMEDIATION — F-001 CRIT (bare invocations no CLI file args); F-002 HIGH (_is_comment_line() True=ELIGIBLE corrected + .py extension); F-003 HIGH (positive .py coverage AC-183-006 new); F-004 HIGH (BAD_CASES redesign .rs/.py cases + runner ext tuple); F-005 MED (pattern tuple shape corrected); F-006 MED (scrub lines 258/261 task); F-007 MED (AC-183-006 added); F-008 MED (pattern count 28-to-29 corrected); F-009 MED (ci.yml sibling-prose sweep); F-010 MED (AC-183-007 efficacy+zero-FP); F-020 MED (inputs: set); F-023 LOW (level:maintenance). |
| 1.0 | 2026-07-25 | story-writer | Initial authorship — wave-86 STORY-CREATION BURST (D-516). |
