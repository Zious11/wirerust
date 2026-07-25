---
document_type: story
story_id: STORY-183
epic_id: E-11
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-07-25T00:00:00Z
phase: f7
level: maintenance
cycle: wave-086
points: 3
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
estimated_days: 1
wave: "86"
traces_to:
  - .factory/cycles/wave-084/lessons.md
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/df-validation-2026-07-25.md
  - bin/check-green-doc-tense
  - bin/test_check_green_doc_tense.py
inputs:
  - .factory/cycles/wave-084/lessons.md
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/df-validation-2026-07-25.md
input-hash: "3831f42"
---

# STORY-183: check-green-doc-tense: bin/*.py Prose Coverage + Expected-RED Phrase-Class Extension

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** 86
**Points:** 3
**Priority:** P2

## Narrative

- **As a** CI gate operator reviewing step-4 (green-doc-tense) gate results
- **I want** `bin/check-green-doc-tense` to scan `bin/*.py` prose as well as `*.rs` files,
  and to detect `Expected RED:` headings and `currently falls through` body-phrase stale prose
- **So that** Python files in `bin/` containing stale RED-phase documentation are never
  silently skipped by the gate (PG-W84-010), and the class of stale phrases discovered during
  STORY-180 pass-1 (PG-W85-003, 9 stale sites) can no longer escape the gate undetected

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; bin/ tooling change only)_

## Background

### PG-W84-010 — bin/check-green-doc-tense scans only *.rs files

`bin/check-green-doc-tense` collects files via `_collect_rust_files()` (line ~466), which
runs `git ls-files -- tests/*.rs src/**/*.rs` and filters with `line.endswith(".rs")`. The
scan explicitly excludes all `bin/*.py` files. During STORY-176 adversarial pass-1 (wave-84),
the adversary caught stale RED-phase prose in `bin/test_check_green_doc_tense.py` that had
passed the gate (finding F-S176P1-003). The root cause was confirmed in
`df-validation-2026-07-25.md §PG-W84-010`.

### PG-W85-003 — Missing "Expected RED:" and "currently falls through" pattern classes

`bin/check-green-doc-tense` `_VIOLATION_PATTERNS` (line 217) contains **28 tuples**, labeled
Pattern 1 through Pattern 29 in the module docstring token list. The patterns list does NOT
include:
- `Expected RED:` — a heading pattern indicating the test was authored as a RED-gate test
  but whose doc still describes the pre-pass state
- `currently falls through` / `currently fall through` — a body phrase from implementation
  notes indicating code was in a non-final path state during story authorship

During STORY-180 per-story adversarial pass-1, the adversary found 9 stale sites of these
classes that the gate had not flagged (D-506, 2026-07-24). The gate exited 0 with no flag.

### Scan semantics: `_is_comment_line()` returning True means ELIGIBLE TO BE FLAGGED

`scan_file()` (line ~502) iterates every line and calls `_is_comment_line()`. When
`_is_comment_line()` returns True, the line IS eligible to be scanned for violations. When it
returns False, the line is SKIPPED (not scanned). The entire BAD_CASES corpus in
`test_check_green_doc_tense.py` consists of `//`-comment lines that return True from
`_is_comment_line()` and ARE flagged. For Python files, `#`-prefixed lines are the comment
lines and MUST return True from `_is_comment_line()` so they ARE scanned for violations.
This is the mechanism that makes `.py` scanning useful — the opposite of an exemption.

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
change is required for the self-test file. The ci.yml comment at line ~442 (scope description
"tests/*.rs and src/**/*.rs") is stale prose that must be updated — the FUNCTIONAL job steps
(lines 461/463) remain unchanged.

## Acceptance Criteria

### AC-183-001 (traces to PG-W84-010 — scan glob extension to bin/*.py)

`bin/check-green-doc-tense` is extended to include `bin/*.py` files in its prose scan.

- Given `_collect_rust_files()` at line ~466 currently runs `git ls-files -- tests/*.rs src/**/*.rs`
  and filters with `line.endswith(".rs")`, excluding all Python files including `bin/*.py`
- When the function is refactored (or a new unified `_collect_source_files()` function is added)
  to additionally run `git ls-files -- bin/*.py` and include entries ending with `.py`
- And the overall function name is updated (or an alias added) to reflect the broader scope
  (e.g., `_collect_source_files()` or annotated `# scope: *.rs + bin/*.py`)
- And the `main()` function (line ~549) is updated to call the new/renamed collector function
- And the pass-message at line ~591 is updated to reflect both file types in the scanned count

- Then `python3 bin/check-green-doc-tense` (bare invocation — no file arguments; main() uses
  git ls-files internally) exits non-zero when any `bin/*.py` file contains a pattern match
  in a `#`-prefixed comment line, and exits 0 when no such violations exist

Verification:
```bash
# The gate is verified via its self-test runner — no file arguments exist:
python3 bin/test_check_green_doc_tense.py
# Must pass (exit 0), including new .py fixture cases added in AC-183-003/004/006

# Manual smoke: after delivery, confirm the gate scans bin/*.py
python3 bin/check-green-doc-tense
# Must exit 0 (zero violations in bin/*.py after all scrubs and rewords in this story)
```

### AC-183-002 (traces to PG-W84-010 — Python comment-line scan eligibility)

The comment-line detection in `bin/check-green-doc-tense` is extended so that Python
`#`-prefixed lines ARE ELIGIBLE TO BE SCANNED (and flagged) for violations — the same
semantics as `//`-prefixed Rust lines.

- Given `_is_comment_line()` currently only returns True for `stripped.startswith("//")`,
  meaning ONLY `//`-prefixed lines are scanned for violations
- When `_is_comment_line()` is extended to also return True for `stripped.startswith("#")`:
  ```python
  def _is_comment_line(stripped: str) -> bool:
      """Return True if the line is a comment — eligible to be scanned for violations.

      Rust: lines starting with // or //! (inner doc)
      Python: lines starting with # (any Python comment)

      Returning True means the line IS scanned; returning False means the line is skipped.
      """
      return stripped.startswith("//") or stripped.startswith("#")
  ```
- Then `scan_file()` processes `#`-prefixed lines in `.py` files through all violation
  patterns, flagging any that match — exactly as it processes `//`-prefixed lines in `.rs` files

**Implementer obligation — zero-false-positive for `bin/test_check_green_doc_tense.py`:**
The self-test file contains both Python `#`-comments describing patterns AND multi-line
string literals whose indented content begins with `//` (e.g., the BAD_CASES fixtures).
After adding `bin/*.py` to the scan:
1. `#`-prefixed Python comments that contain violation-phrase text will be flagged — the
   implementer MUST scrub those specific lines (see Task 4 for lines 258/261).
2. String literal lines whose stripped form starts with `//` will be flagged by the existing
   `//`-detection, same as before this story — the implementer must resolve these (see Task 5
   for the string-literal false-positive remediation: use string concatenation, skip pragmas,
   or a dedicated file-exclusion mechanism consistent with zero-FP requirement).

Verification:
```bash
# After all scrubs and rewording, full bin/*.py scan must exit 0:
python3 bin/check-green-doc-tense
echo "Exit code: $?"  # Must be 0

# Self-test must pass:
python3 bin/test_check_green_doc_tense.py
echo "Exit code: $?"  # Must be 0
```

### AC-183-003 (traces to PG-W85-003 — "Expected RED:" heading pattern, Pattern 30)

A new violation pattern is added to `_VIOLATION_PATTERNS` as the **29th tuple** (labeled
"Pattern 30") in `bin/check-green-doc-tense`.

- Given that no entry for `Expected RED` appears in `_VIOLATION_PATTERNS` (confirmed by
  `df-validation-2026-07-25.md §PG-W85-003`)
- When the new tuple is appended after the existing Pattern 29 entry:
  ```python
  # -----------------------------------------------------------------------
  # Patterns 30-31: added to catch stale "Expected RED:" and "currently
  # falls through" phrasing classes found in D-506 (PG-W85-003, STORY-183).
  # -----------------------------------------------------------------------
  (
      # Pattern 30: "Expected RED:" heading — stale pre-pass state in test doc.
      # The colon is the discriminator: it marks the heading form "Expected RED: <desc>",
      # distinguishing it from contextual references like "this was expected RED before fix".
      # Allowlist: bare "expected RED" without colon; "was expected to fail (RED phase)".
      "Pattern 30 (PG-W85-003): Expected RED: heading — stale pre-pass state in test doc (AC-183-003)",
      re.compile(r"Expected RED:", re.IGNORECASE),
  ),
  ```
- And the corresponding known-bad `.rs` fixture is added to BAD_CASES in
  `bin/test_check_green_doc_tense.py`:
  ```python
  (
      "Pattern 30: Expected RED: heading violation (.rs form)",
      "// Expected RED: all assertions fail before implementation\n",
      "Pattern 30",
  ),
  ```
- And the corresponding known-bad `.py` fixture is added to BAD_CASES (4-tuple with `.py`
  extension — requires runner extension in Task 3):
  ```python
  (
      "Pattern 30: Expected RED: heading violation (.py form — Python comment)",
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

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must exit 0; both Pattern 30 BAD cases must appear in output as PASS
```

### AC-183-004 (traces to PG-W85-003 — "currently falls through" body-phrase pattern, Pattern 31)

A new violation pattern is added to `_VIOLATION_PATTERNS` as the **30th tuple** (labeled
"Pattern 31") in `bin/check-green-doc-tense`.

- Given no entry for `currently falls through` appears in `_VIOLATION_PATTERNS` (confirmed by
  `df-validation-2026-07-25.md §PG-W85-003`)
- When the new tuple is appended after Pattern 30:
  ```python
  (
      # Pattern 31: "currently falls through" / "currently fall through" — stale in-progress
      # path description asserting present-tense incomplete dispatch. The word "currently"
      # is the discriminator — past-tense forms ("fell through before the fix") are allowlisted.
      # The `falls?` handles both "currently falls through" and "currently fall through".
      # Allowlist: "fell through to" (past tense); "falls through to" without "currently".
      "Pattern 31 (PG-W85-003): currently falls? through — stale in-progress path description (AC-183-004)",
      re.compile(r"currently\s+falls?\s+through", re.IGNORECASE),
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
- And the corresponding known-good fixture is added to GOOD_CASES:
  ```python
  (
      "Pattern 31 allowlist: past tense — fell through before the fix",
      "// Before the fix, the packet fell through the dispatcher to the wildcard arm.\n",
  ),
  ```
- And a zero-FP known-good for the 12 live legitimate "falls through to" sites is added:
  ```python
  (
      "Pattern 31 zero-FP: falls through to (no 'currently' prefix — legitimate present-tense path narration)",
      "// The lax path falls through to the wildcard arm when no analyzer matches.\n",
  ),
  ```

- Then `python3 bin/test_check_green_doc_tense.py` exits 0 with both Pattern 31 bad cases
  flagged and both allowlist/zero-FP cases clean

**Zero-false-positive requirement:**
- `// the packet currently falls through` — IS flagged ✓
- `# the packet currently falls through` — IS flagged ✓ (via AC-183-002)
- `// before the fix, the packet fell through` — NOT flagged (past tense "fell") ✓
- `// falls through to the wildcard arm` — NOT flagged (no "currently" prefix) ✓

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
grep -A 5 "Unreleased" CHANGELOG.md | grep -q "check-green-doc-tense" && echo PASS || echo FAIL
```

### AC-183-006 (traces to PG-W84-010 — positive .py coverage proof)

A deliberate test proves that a `.py` file with a stale `#`-comment IS flagged by the gate
(positive coverage: the gate is not merely inert on Python files).

- Given `scan_file()` in `bin/check-green-doc-tense` is called with a Path to a temp `.py`
  file containing `# Expected RED: deliberate stale heading\n`
- When the self-test runner's BAD cases include the 4-tuple `(".py" form, see AC-183-003)` and
  the runner writes it as `bad_N.py` (via the runner extension in Task 3)
- Then `scan_file(bad_N.py)` returns a non-empty violations list, confirming patterns fire on
  `#`-prefixed Python comment lines

This AC is satisfied by the `.py` 4-tuple entries added in AC-183-003 and AC-183-004 once the
runner extension is in place.

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# In output: both ".py form" PASS entries confirm positive .py coverage
```

### AC-183-007 (traces to PG-W85-003 — efficacy check against D-506 sites + zero-FP on live sites)

Patterns 30 and 31 cover the actual phrasing classes found in the 9 D-506 stale sites, and
produce zero false positives on the 12 live legitimate "falls through" sites.

- Given the 9 D-506 stale sites discovered by the wave-85 adversary all used one of two
  phrasing forms: `Expected RED:` headings (flagged by Pattern 30) or `currently falls through`
  / `currently fall through` body phrases (flagged by Pattern 31)
- When both patterns are delivered and the zero-FP GOOD_CASES covers:
  - `"// The lax path falls through to the wildcard arm."` (legitimate present-tense path
    narration without "currently" — one representative of the 12 live sites)
  - `"// Before the fix, the packet fell through to the dispatcher."` (past tense)
  - `"// was expected to fail (RED phase) before the implementation shipped."` (no colon form)
- Then `python3 bin/test_check_green_doc_tense.py` passes all GOOD_CASES related to patterns
  30 and 31

**Live legitimate "falls through to" reference sites (zero-FP regression guard):**
The following sites in the current codebase use "falls through to" WITHOUT "currently" and
MUST NOT be flagged. The Pattern 31 zero-FP GOOD_CASE (`"falls through to" without "currently"`)
serves as the regression guard:
- `tests/bc_2_16_d078_lax_malformed_tests.rs:18,90`
- `tests/main_story_089_tests.rs:648`
- `src/analyzer/tls.rs:930`
- And up to 8 additional live sites — enumerated with `grep -rn "falls through to"
  tests/ src/ | grep -v "currently"` at implementation time

Verification:
```bash
# Run self-test — all Pattern 30/31 GOOD_CASES must pass:
python3 bin/test_check_green_doc_tense.py

# Zero-FP regression spot-check on known live sites (must exit 0):
python3 bin/check-green-doc-tense
# The gate scans tests/ and src/ — if these run clean the zero-FP check passes
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Scan glob extension + Python comment-line scan eligibility | `bin/check-green-doc-tense` (amend `_collect_rust_files` → `_collect_source_files`, `_is_comment_line`, `main`) | develop |
| Pattern 30 (Expected RED:) + Pattern 31 (currently falls through) | `bin/check-green-doc-tense` (amend `_VIOLATION_PATTERNS` — 28 tuples → 30 tuples) | develop |
| Runner extension for .py fixture extension + new BAD/GOOD cases | `bin/test_check_green_doc_tense.py` (amend) | develop |
| Stale-comment scrub (lines 258/261) | `bin/test_check_green_doc_tense.py` (amend — reword two #-prefixed lines) | develop |
| CHANGELOG entry | `CHANGELOG.md` (amend `[Unreleased]`) | develop |
| Stale scope comment update | `.github/workflows/ci.yml` (amend comment line ~442 only; no functional change) | develop |

**No `src/` changes, no `Cargo.toml` changes, no functional CI workflow changes.**
CHANGELOG obligation: YES — `bin/` changes trigger AC-158-001.
No new `bin/test_*.py` file — L-W84-003 / AC-165-001 CI-wiring obligation does NOT apply.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Rust comment `// Expected RED: stale heading` | IS scanned and IS flagged — `_is_comment_line()` returns True for `//` prefix (existing behavior); Pattern 30 matches |
| EC-002 | Python comment `# Expected RED: stale heading` | IS scanned and IS flagged — `_is_comment_line()` returns True for `#` prefix (AC-183-002); Pattern 30 matches |
| EC-003 | Python non-comment line containing `Expected RED:` (e.g., string literal) | NOT flagged — `_is_comment_line()` returns False for non-`//`/non-`#` lines; scan skips it |
| EC-004 | `expected_red` (snake_case identifier) in a non-comment line | NOT flagged — non-comment line is skipped |
| EC-005 | `bin/test_check_green_doc_tense.py` BAD_CASES string literals contain `//` lines with violation patterns | Implementer must prevent self-flagging (string-literal context) via string concatenation, skip pragma, or equivalent — zero-FP is a HARD requirement |
| EC-006 | `bin/` contains a `.py` file with ONLY `#`-comment lines matching a pattern | IS flagged — `#`-prefixed lines are ELIGIBLE to be flagged (AC-183-002); this is the desired behavior |
| EC-007 | `// currently falls through` in Rust comment line | IS scanned and IS flagged — `_is_comment_line()` returns True; Pattern 31 matches |
| EC-008 | `// falls through to the wildcard arm` (no "currently") | NOT flagged — Pattern 31 requires "currently" prefix |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `bin/check-green-doc-tense` | effectful-shell | Shell script (Python): git subprocess, file I/O, stdout/stderr. Pure-data additions (`_VIOLATION_PATTERNS`, `_is_comment_line` extension) live inside an effectful script. |
| `bin/test_check_green_doc_tense.py` | effectful-shell | Test runner: spawns subprocesses, reads/writes temp files, asserts on exit codes and scan results. |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| Story spec (this file) | ~5.5 k |
| `bin/check-green-doc-tense` (full script, ~600 lines) | ~5.0 k |
| `bin/test_check_green_doc_tense.py` (full file, ~800 lines after additions) | ~6.0 k |
| `.github/workflows/ci.yml` (comment-only change, surrounding lines) | ~0.5 k |
| `CHANGELOG.md` (recent unreleased section only) | ~0.5 k |
| Tool outputs overhead | ~1.0 k |
| **Total** | **~18.5 k** |
| Agent context window | 200 k (Sonnet) |
| **Budget usage** | **~9%** |

Well within context window. No story split required.

## Tasks

1. **Read both target files fully** — `bin/check-green-doc-tense` (understand
   `_collect_rust_files()` line ~466, `_is_comment_line()` line ~460, `_VIOLATION_PATTERNS`
   line 217 — 28 tuples, labels 1–29, last entry is Pattern 29 "until … wired"). Read
   `bin/test_check_green_doc_tense.py` fully — understand BAD_CASES (2-/3-tuple format,
   runner writes `bad_N.rs` temp files), GOOD_CASES, and all existing test runner sections.

2. **Extend `_collect_source_files()` glob (AC-183-001):** Add `git ls-files -- bin/*.py`
   alongside the existing Rust globs. Accept `.py` endings. Update `main()` call site.
   Update the pass-message at ~591 to reflect both file types. Rename function from
   `_collect_rust_files` to `_collect_source_files` (or annotate with `# scope: *.rs + bin/*.py`).

3. **Extend `_is_comment_line()` (AC-183-002):** Add `stripped.startswith("#")` return arm.
   Verify AC-183-002 semantics: returning True makes the line ELIGIBLE TO BE FLAGGED.

4. **Scrub stale `#`-comments in `bin/test_check_green_doc_tense.py` (F-006 / AC-183-002):**
   Two Python `#`-prefixed comment lines at approximately:
   - Line ~258: `#   (a) skeleton\s+compiles?        — "harness skeleton compiles" stub-era`
     → Reword to remove the `"harness skeleton compiles"` quoted example, e.g.:
     `#   (a) \bskeleton\s+compiles?\b  — compile-only stub-era assertion (pattern 26)`
   - Line ~261: `#   (d) \buntil\b[^\n]*\bwired\b    — "fails until wired" CI-wiring prose`
     → Reword to remove the `"fails until wired"` quoted example, e.g.:
     `#   (d) \buntil\b[^\n]*\bwired\b   — CI-wiring-incomplete prose (pattern 29)`
   After rewording, confirm neither line triggers patterns 26 or 29 by running
   `python3 bin/test_check_green_doc_tense.py` — it must still pass fully.

5. **Resolve string-literal false positives in `bin/test_check_green_doc_tense.py` (AC-183-002
   zero-FP obligation):** The BAD_CASES section contains multi-line string literals whose
   indented lines start with `//`, e.g.:
   ```python
   """\
   // harness skeleton compiles only — wiring deferred to STORY-176
   """,
   ```
   When the gate scans `test_check_green_doc_tense.py`, these `//` lines will be flagged by
   existing patterns. The implementer MUST choose a resolution:
   - **Option A (preferred):** Convert each such BAD_CASES fixture content to use Python string
     concatenation so the literal `//`-comment text does not appear as a contiguous line in the
     Python source, e.g.: `"// harness ske" + "leton compiles only — wiring deferred to STORY-176\n"`.
     The temp `.rs` file receives the reassembled full string and still triggers the gate.
   - **Option B:** Add a per-file skip mechanism (e.g., `# check-green-doc-tense: skip-file`
     pragma at the top of `test_check_green_doc_tense.py`) recognized by `_collect_source_files()`.
     This is acceptable ONLY for the self-test file and must be documented with rationale.
   The zero-FP requirement (gate exits 0 on full `bin/*.py` scan) is NON-NEGOTIABLE.

6. **Add patterns 30 and 31 to `_VIOLATION_PATTERNS` (AC-183-003, AC-183-004):** After the
   Pattern 29 entry, add the section comment and two new tuples. Verify numbering: the 28th
   tuple is Pattern 29, the new 29th tuple is Pattern 30, the new 30th tuple is Pattern 31.

7. **Extend `bin/test_check_green_doc_tense.py` runner for `.py` extension (AC-183-003/004/006):**
   In the BAD_CASES loop, use `entry[3]` as the file extension when present:
   ```python
   ext = entry[3] if len(entry) > 3 else ".rs"
   p = _tmpfile(content, tmp, f"bad_{passed}{ext}")
   ```
   Add the 4 new BAD_CASES (2 for Pattern 30 — `.rs` and `.py`; 2 for Pattern 31 — `.rs`
   and `.py`). Add 3 new GOOD_CASES (Pattern 30 allowlist, Pattern 31 allowlist, Pattern 31
   zero-FP for bare "falls through to").

8. **Efficacy verification (AC-183-007):** After adding GOOD_CASES, run
   `python3 bin/check-green-doc-tense` on the repo to confirm all existing test/src files still
   pass clean (zero new false positives from patterns 30/31 on the 12+ live "falls through to"
   sites). Run `grep -rn "falls through to" tests/ src/` to enumerate all live sites; verify
   none start with "currently".

9. **Sibling-prose sweep (F-009):**
   - `bin/check-green-doc-tense` module docstring (lines 2–4 scope text, lines 26–30 TOKEN LIST
     preamble, lines 87–88 ALLOWLIST preamble): update scope declarations from "tests/*.rs and
     src/**/*.rs" to include "bin/*.py".
   - `.github/workflows/ci.yml` comment at line ~442: update
     `"bin/check-green-doc-tense scans tracked tests/*.rs and src/**/*.rs"` to include
     `"and bin/*.py"`. This is a COMMENT-ONLY change; the functional job steps are unchanged.

10. **Add CHANGELOG `[Unreleased]` entry (AC-183-005):** One-line entry describing the
    scan-glob + comment-detection + pattern extensions, referencing PG-W84-010, PG-W85-003,
    STORY-183.

11. **Run full self-test and zero-FP gate check (AC-183-001/002/006):**
    ```bash
    python3 bin/test_check_green_doc_tense.py  # must exit 0
    python3 bin/check-green-doc-tense           # must exit 0 (no new violations)
    ```

12. **Develop PR:** `bin/check-green-doc-tense`, `bin/test_check_green_doc_tense.py`,
    `CHANGELOG.md`, and `.github/workflows/ci.yml` (comment-only) in a single develop PR.
    CI `changelog-gate` must pass.

## Previous Story Intelligence

- **STORY-176 (wave-84, patterns 26–29):** Added skeleton/seam/compile-only/until-wired
  patterns. The wave-84 adversary found stale Python doc missed by the gate (F-S176P1-003) —
  PG-W84-010 tracks this gap. STORY-183 adds patterns 30–31 using the same append-only
  methodology. Read STORY-176 Tasks before implementing for BAD_CASES / GOOD_CASES format.
- **STORY-180 (wave-85):** Adversarial pass-1 found 9 stale sites with "Expected RED:" and
  "currently falls through" phrasing (D-506, PG-W85-003). STORY-183 closes this gap.
- **Patterns 1–29 baseline:** 28 tuples, labeled Pattern 1 through Pattern 29. Pattern 29 is
  the "until … wired" entry (last in the `# Patterns 26-29` block). Append-only; never reorder
  or remove existing entries. New entries become the 29th and 30th tuples (Pattern 30, Pattern 31).

## Architecture Compliance Rules

- **Append-only `_VIOLATION_PATTERNS`:** Never remove or reorder existing patterns. New
  patterns are appended as the 29th and 30th tuples with sequential label numbers (30, 31).
- **`bin/` changes require CHANGELOG (AC-158-001):** Any PR touching `bin/` files MUST include
  an `[Unreleased]` CHANGELOG entry. The `changelog-gate` CI job enforces this.
- **L-W84-003 / AC-165-001 CI wiring:** Not triggered — STORY-183 extends an existing
  `bin/test_*.py` file. No new `.github/workflows/ci.yml` job steps required.
- **Action SHA-pin policy:** STORY-183 makes no functional `ci.yml` changes (comment-only
  update to line ~442); no new SHA pins required. The ACTION-PIN-GATE job is unaffected.
- **Zero-false-positive hard requirement:** `python3 bin/check-green-doc-tense` MUST exit 0
  after delivery. The implementer has full latitude on string-literal false-positive resolution
  strategy (Task 5) provided zero-FP is achieved.
- **`_is_comment_line()` semantics:** INCLUSIVE (returning True = ELIGIBLE TO BE FLAGGED).
  Extending it for `#` makes Python comment lines scannable, not exempt.

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Python | 3.10+ | `bin/` scripts use modern type syntax (CLAUDE.md) |
| No new Python packages | — | All changes use Python stdlib only |
| No new Cargo.toml deps | — | No Rust changes in this story |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `bin/check-green-doc-tense` | Modify | Extend `_collect_source_files()`, extend `_is_comment_line()`, add Pattern 30/31 to `_VIOLATION_PATTERNS`, update module docstring scope text |
| `bin/test_check_green_doc_tense.py` | Modify | Runner `.py` extension support; 4 new BAD_CASES + 3 new GOOD_CASES; scrub lines ~258/~261; resolve string-literal FP |
| `CHANGELOG.md` | Modify | Add `[Unreleased]` entry for PG-W84-010 + PG-W85-003 (AC-183-005) |
| `.github/workflows/ci.yml` | Modify (comment line ~442 only) | Update scope comment to include `bin/*.py`; no functional job changes |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`, `tests/**/*`

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-07-25 | story-writer | WAVE-86 PASS-1 REMEDIATION — F-001 CRIT (rewrite all verification to bare python3 invocations, no file-arg CLI); F-002 CRIT (AC-183-002 complete rewrite: `_is_comment_line` True = ELIGIBLE TO BE FLAGGED, not exempted; Python `#`-lines must be scanned); F-003 CRIT (add AC-183-006 positive .py coverage proof); F-004 CRIT (runner extension for .py fixture extension; .rs BAD case uses `// Expected RED:`, .py BAD case uses `# Expected RED:`; EC-001/002/007 corrected); F-005 HIGH (pattern tuple shape corrected to (label, re.compile(...))); F-006 HIGH (Task 4: scrub lines 258/261 `#`-comments; Task 5: string-literal FP resolution); F-007 HIGH (AC-183-006 positive .py coverage proof); F-008 MED (pattern count corrected to 28 tuples, labels to Pattern 29; new entries are 29th/30th tuples labeled 30/31); F-009 MED (sibling-prose sweep tasks; ci.yml removed from Forbidden-modifications for comment-only update, Task 9); F-010 MED (AC-183-007 efficacy check; zero-FP GOOD_CASE for bare "falls through to"; Pattern 31 covers `falls?` for both "falls" and "fall" forms); F-020 MED (inputs: set to [wave-084/lessons.md, wave-085/lessons.md, df-validation-2026-07-25.md]); F-023 LOW (level: feature→maintenance). |
