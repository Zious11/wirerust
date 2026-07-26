---
document_type: story
story_id: STORY-183
epic_id: E-11
version: "1.9"
status: draft
producer: story-writer
timestamp: 2026-07-25T00:00:00Z
phase: f7
level: maintenance
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
**Status:** draft
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
Pattern 1 through Pattern 29 in the module docstring token list. **Ground truth (convergence-report.md lines 63-66, STORY-180 F-180-P1-003):** the 9 D-506
stale sites used `currently asserts` and `is expected to` — NOT `Expected RED:` /
`currently falls through`. The lesson summary AND the PG-W85-003 paragraph at
convergence-report.md lines 68-70 both carried broader (incorrect) labels. Lines 63-66 are
the sole authoritative source for the specific D-506 token set.

The following TIER-1 tokens (zero-FP automatable, 0 live legitimate uses each) are absent
from the existing 28 tuples (confirmed by grep of `bin/check-green-doc-tense`):
- `currently asserts` — primary D-506 class; 9 stale sites (convergence report line 64)
- `falls to the wildcard` — 0 live legitimate uses
- `currently fall` — covers "currently falls through", "currently fall through", etc.
- `doesn't exist yet` / `does not exist yet`
- `currently has NO`
- `currently satisfied by`
- `will be GREEN currently`

**TIER-2 tokens (context-dependent; MUST NOT be added to tool per F-W86S-P2-006 / F-W86S-P3-001 PO rulings):**
- `is expected to` — **secondary D-506 phrasing class** (convergence report line 64); 6+ live
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
  additionally `src/**/*.rs` NEVER matches top-level `src/*.rs` files (git wildmatch `**`
  requires an intermediate directory component — `src/mitre.rs`, `src/lib.rs`, etc. are
  silently unscanned; F-W86S-P9-009, wave-86)
- When the function is renamed from `_collect_rust_files` to `_collect_source_files` and
  extended to additionally run `git ls-files -- bin/*.py src/*.rs` and include entries ending
  with `.py` (for Python) while the existing Rust glob is corrected to
  `git ls-files -- tests/*.rs src/**/*.rs src/*.rs` (adding `src/*.rs` to cover top-level
  src files that `src/**/*.rs` misses)
- And the rename is propagated to 13 `_collect_rust_files` sites + 1 `rust_files` prose site at :721 (14 total) in `bin/test_check_green_doc_tense.py`
  (6 functional monkey-patch sites at approximately :699/:707/:726/:859/:872/:905;
  7 `_collect_rust_files` prose sites at approximately :688,:705,:711,:718,:839,:843,:891;
  plus :721 — a `rust_files` prose site, failure-message string citing `if not rust_files:` guard — update to `source_files`)
- And the `main()` function (line ~549) is updated to call the new/renamed collector function
- And the pass-message at line ~591 is updated to reflect both file types in the scanned count
- And a self-test assertion verifies `_collect_source_files(repo_root)` returns a path set
  containing an entry whose `.name` attribute equals `test_check_green_doc_tense.py`
  (collector returns `Path` objects with absolute paths; comparison by `.name` avoids
  repo-root-prefix fragility — do NOT compare to a repo-relative string like
  `"bin/test_check_green_doc_tense.py"`)
- And a self-test assertion verifies `_collect_source_files(repo_root)` returns a path set
  containing at least one entry whose `.name` attribute equals a known top-level `src/*.rs`
  file (e.g., `mitre.rs`) — this assertion CANNOT pass unless `src/*.rs` is in the glob,
  because `src/**/*.rs` alone would never match `src/mitre.rs` (git wildmatch blind spot,
  F-W86S-P9-009)

- Then `python3 bin/check-green-doc-tense` (bare invocation — no file arguments; main() uses
  git ls-files internally) exits non-zero when any `bin/*.py` file contains a pattern match
  in a `#`-prefixed comment line, and exits 0 when no such violations exist
- And the scanned-file count in the pass-message includes `.py` files (e.g., "N files scanned"
  where N > the previous Rust-only count)

Verification:
```bash
# The gate is verified via its self-test runner — no file arguments exist:
python3 bin/test_check_green_doc_tense.py
# Must pass (exit 0), including new .py fixture cases added in AC-183-003/004/007

# Manual smoke: after delivery, confirm the gate scans bin/*.py
python3 bin/check-green-doc-tense
# Must exit 0 (zero violations in bin/*.py after all scrubs and rewords in this story)

# Positive .py coverage proof: self-test runner exercises bad_N.py files
# (via runner extension in Task 7) — confirmed by ".py form" PASS lines in output
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

      Rust: lines starting with // or //! (inner doc)
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
      # Word "currently" is the discriminator; `falls?\b` catches both singular and plural.
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
in AC-183-001 (`_collect_source_files(repo_root)` returns a set containing
`bin/test_check_green_doc_tense.py`). The gate's live run (`python3 bin/check-green-doc-tense`)
exercises the full pipeline: collect → scan with suffix-scoped comment detection → report.

Verification:
```bash
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
      # STORY-180 F-180-P1-003, convergence-report line 64). 0 live legitimate uses.
      "Pattern 32 (PG-W85-003): 'currently asserts' — RED-phase present-tense claim (AC-183-007)",
      re.compile(r"currently\s+asserts?\b", re.IGNORECASE),
  ),
  (
      # Pattern 33: wildcard-arm fallthrough phrase; TIER-1 per v6; 0 live legitimate uses.
      # NOT the same as the TIER-2 "falls through to" token (bare form without "to the wildcard").
      "Pattern 33 (PG-W85-003): 'falls to the wildcard' — RED-phase wildcard-arm fallthrough (AC-183-007)",
      re.compile(r"falls\s+to\s+the\s+wildcard", re.IGNORECASE),
  ),
  (
      # Pattern 34: negative-capability claim (not-yet-extant feature); 0 live legitimate uses.
      # Renumbered from Pattern 35 (v3); prior v3 Pattern 34 moved to TIER-2 in v4 (F-W86S-P3-001).
      "Pattern 34 (PG-W85-003): 'does not / doesn't exist yet' — negative-capability claim (AC-183-007)",
      re.compile(r"does\s+not\s+exist\s+yet|doesn'?t\s+exist\s+yet", re.IGNORECASE),
  ),
  (
      # Pattern 35: present-tense absence claim; 0 live legitimate uses.
      # Renumbered from Pattern 37 (v3).
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
  # Pattern 32 GOOD (no "currently asserts"):
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
  # Pattern 33 GOOD (no "falls to the wildcard"):
  ("Pattern 33 allowlist: different routing — forwarded to default handler",
   "// the unrecognized packet is forwarded to the default handler branch\n"),
  # Pattern 34 BAD (.rs):
  ("Pattern 34: does not exist yet violation (.rs form)",
   "// this error-handling path does not exist yet in the decoder\n",
   "Pattern 34"),
  # Pattern 34 GOOD (no "exist yet"):
  ("Pattern 34 allowlist: past tense — path was added",
   "// this code path was added in response to the missing handler report\n"),
  # Pattern 35 BAD (.rs):
  ("Pattern 35: currently has NO violation (.rs form)",
   "// the TLS dissector currently has no SNI extraction logic\n",
   "Pattern 35"),
  # Pattern 35 GOOD (no "currently has no"):
  ("Pattern 35 allowlist: different phrasing — lacks",
   "// the TLS dissector lacks SNI extraction for DTLS traffic\n"),
  # Pattern 36 BAD (.rs):
  ("Pattern 36: currently satisfied by violation (.rs form)",
   "// the invariant is currently satisfied by the no-op stub placeholder\n",
   "Pattern 36"),
  # Pattern 36 GOOD (no "currently satisfied by"):
  ("Pattern 36 allowlist: production implementation",
   "// the invariant is enforced by the production TCP-state machine\n"),
  # Pattern 37 BAD (.rs):
  ("Pattern 37: will be GREEN currently violation (.rs form)",
   "// this gate will be GREEN currently because the check is bypassed\n",
   "Pattern 37"),
  # Pattern 37 GOOD (no "will be GREEN currently"):
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

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must exit 0; Pattern 32–37 BAD cases must appear in output as PASS;
# three TIER-2 GOOD_CASEs must confirm tool does NOT flag them
```

### AC-183-008 (traces to PG-W85-003 — efficacy: TIER-1 D-506 phrasing covered; TIER-2 correctly excluded)

The final pattern set (Patterns 30–37) covers the TIER-1 phrasing from the 9 D-506 stale
sites and produces zero false positives on live TIER-2 sites, per DF-GREEN-DOC-TENSE-SWEEP v6
(F-W86S-P2-006 / F-W86S-P3-001 rulings, 2026-07-25).

- Given the 9 D-506 stale sites (STORY-180 F-180-P1-003, convergence report lines 63-66) used:
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
The following live sites use "falls through to" WITHOUT "currently" and MUST NOT be flagged:
- `tests/bc_2_16_d078_lax_malformed_tests.rs:18,90`
- `tests/main_story_089_tests.rs:890`
- `src/analyzer/tls.rs:930`

The Pattern 31 zero-FP GOOD_CASE and the gate's live run (`python3 bin/check-green-doc-tense`)
serve as the combined regression guard.

Verification:
```bash
# Run self-test — all Pattern 30-37 GOOD_CASES must pass:
python3 bin/test_check_green_doc_tense.py

# Zero-FP regression check on live codebase (must exit 0):
python3 bin/check-green-doc-tense
# If this exits 0, the 10 live 'falls through to' sites plus all 'is expected to'
# sites are clean — zero false positives from Patterns 30-37.
```

### AC-183-009 (process-gap PG-W84-012 — bin/test_lint_cycle_artifact.py MUST pass locally)

`python3 bin/test_lint_cycle_artifact.py` MUST exit 0 (all self-tests pass) at delivery time.
CI wiring for this selftest file is tracked under PG-W84-012 (pending devops-engineer dispatch
and human authorization) — do NOT add a CI wiring task for this story; that remains the
PG-W84-012 ops task.

- Given `bin/test_lint_cycle_artifact.py` is modified by Task 13 (4-line scrub at lines :3, :5,
  :6, :125) as part of this story
- When the implementer runs `python3 bin/test_lint_cycle_artifact.py` locally after the Task 13
  scrub
- Then the command MUST exit 0 with the same pass count as before the scrub (the scrub is a
  pure text rewording — no test logic changes; the TC count remains 21)
- And CI wiring for this file is explicitly NOT added in this story's PR (see PG-W84-012 for
  the ops-task tracking the `bin-selftest` required-status-check wiring)

Verification:
```bash
python3 bin/test_lint_cycle_artifact.py
# Must exit 0; pass count must be 21 (TC1–TC21 unchanged by the scrub)
# CI wiring: NOT wired in this story — deferred to PG-W84-012 ops task
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
| EC-010 | Extension-less Python executables in `bin/` (e.g., `bin/check-green-doc-tense`) | OUT OF SCOPE for this story — `git ls-files -- bin/*.py` glob only covers `.py` files; shebang detection deferred |
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
| `bin/check-green-doc-tense` (full script, ~620 lines after additions) | ~5.5 k |
| `bin/test_check_green_doc_tense.py` (full file, ~950-960 lines after additions) | ~7.5 k |
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
   alternative). Add `git ls-files -- bin/*.py` alongside the existing Rust globs AND add
   `src/*.rs` to the Rust glob invocation (correcting the `src/**/*.rs` blind spot for
   top-level src files — F-W86S-P9-009). The corrected Rust glob becomes
   `git ls-files -- tests/*.rs src/**/*.rs src/*.rs`. Accept `.py` endings.
   Update `main()` call site. Update the pass-message at ~591 to reflect both file types.
   Add a self-test assertion confirming `_collect_source_files(repo_root)` returns a path set
   containing an entry whose `.name` attribute equals `test_check_green_doc_tense.py`
   (comparison by `.name` avoids repo-root-prefix fragility — do NOT compare to a
   repo-relative string like `"bin/test_check_green_doc_tense.py"`).
   Add a self-test assertion confirming `_collect_source_files(repo_root)` returns a path set
   containing at least one entry whose `.name` equals a known top-level src file such as
   `mitre.rs` (asserts `src/*.rs` is working; this assertion cannot pass with only
   `src/**/*.rs` in the glob — see AC-183-001 F-W86S-P9-009 note).
   **Propagate rename inside `bin/check-green-doc-tense` itself:**
   - Function docstring at lines :466-473 — update scope description to include `bin/*.py`
   - Local variable `rust_files` at :556 → rename to `source_files` throughout the function
   - Error string at :558-560 (`"no tracked Rust files found"`) → update to
     `"no tracked source files found"` (covers both `.rs` and `.py`)
   **Propagate rename to 13 `_collect_rust_files` sites + 1 `rust_files` prose site at :721 (14 total) in `bin/test_check_green_doc_tense.py`:**
   - 6 functional monkey-patch sites (approximately :699/:707/:726 zero-file guard;
     :859/:872/:905 spy) — these MUST be updated or tests will fail
   - 8 prose/comment sites (approximately :688,:705,:711,:718,:721,:839,:843,:891)
     (:721 — failure-message string citing `if not rust_files:` guard in `main()`;
     references `rust_files` by name — update to `source_files`)
   **Do NOT update** `CHANGELOG.md` line ~741: that entry is a SHIPPED HISTORICAL changelog
   entry (preserved per DF-SIBLING-SWEEP-001); the `_collect_rust_files` name in that entry
   documents what shipped in STORY-176 and must remain as-is for audit provenance. A
   parenthetical annotation like "(renamed _collect_source_files in STORY-183)" may be added
   inline but is NOT required. Similarly, historical references in delivered story specs
   (STORY-158, STORY-162) are preserved as shipped spec provenance — do NOT sweep
   `.factory/stories/` history.

3. **Extend `_is_comment_line(stripped, suffix)` (AC-183-002, F-008):** Add a `suffix`
   parameter (default `""`). Return True for `#`-prefixed lines ONLY when `suffix == ".py"`.
   Update every call site in `scan_file()` to pass `path.suffix`. This prevents ~3625 Rust
   attribute lines (`#[test]`, `#[cfg]`, `#[should_panic]`) from becoming scan-eligible (EC-012).
   Verify AC-183-002 semantics: returning True makes the line ELIGIBLE TO BE FLAGGED.

4. **Scrub stale `#`-comments in `bin/test_check_green_doc_tense.py` (F-006 / AC-183-002):**
   Two Python `#`-prefixed comment lines at approximately lines ~258 and ~261 contain the
   literal flagged phrases `"harness skeleton compiles"` and `"fails until wired"` inside
   quoted examples. The regex patterns match inside quotes equally — the gate will flag these
   lines regardless of quoting. The comments MUST NOT contain the literal flagged phrase at all
   (removing quotes is insufficient):
   - Line ~258: reword entirely to eliminate `harness skeleton compiles`, e.g.:
     `#   (a) \bskeleton\s+compiles?\b  — compile-only stub-era assertion (pattern 26)`
   - Line ~261: reword entirely to eliminate `fails until wired`, e.g.:
     `#   (d) \buntil\b[^\n]*\bwired\b   — CI-wiring-incomplete prose (pattern 29)`
   After rewording, confirm neither line triggers patterns 26 or 29 by running
   `python3 bin/test_check_green_doc_tense.py` — it must still pass fully.

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

8. **Add Patterns 32–37 and self-test cases (AC-183-007):** Append 6 new tuples to
   `_VIOLATION_PATTERNS` as detailed in AC-183-007 (Patterns 32–37; Patterns 34/36/40 from
   v3 are TIER-2 and are NOT added). For each pattern add at minimum one BAD_CASE `.rs` form
   and one GOOD_CASE. For Patterns 32/33 also add `.py` BAD_CASES.
   Add the three TIER-2 zero-FP GOOD_CASEs (for `no .* arm`, `not yet implemented`,
   `currently fails`) asserting the tool does NOT flag them (per F-W86S-P3-001 ruling).
   Add the efficacy GOOD_CASE for `is expected to` (AC-183-008) confirming the tool does
   NOT flag it.
   **Also add the suffix-scoping negative-guard GOOD_CASE (AC-183-006):** the case where a
   `# Expected RED:` line written to a `.rs` temp file is NOT flagged (suffix `.rs` →
   `_is_comment_line` returns False for `#` prefix). This confirms `.py` scan eligibility
   is suffix-scoped, not global.

9. **E2E positive-coverage self-test (F-010):** Add a test function to
   `bin/test_check_green_doc_tense.py` that verifies the full collect→scan→exit pipeline
   in a hermetic environment:
   - Create a temporary directory `<tmp>` and run `git init` inside it to make it a valid
     git repo (`_find_repo_root` walks upward from the **script's own location** to find the
     root — there is NO `WIRERUST_REPO_ROOT` env override; do NOT use it)
   - Copy `bin/check-green-doc-tense` into `<tmp>/bin/check-green-doc-tense` (creates the
     `<tmp>/bin/` directory) so that `_find_repo_root` resolves to `<tmp>` when the copy
     is invoked
   - `git add <tmp>/bin/check-green-doc-tense` (git ls-files reads the index after add;
     no commit required)
   - Create `<tmp>/bin/violating.py` with a `#`-prefixed comment line containing a TIER-1
     phrase (e.g., `"# currently asserts the implementation is complete\n"`)
   - `git add <tmp>/bin/violating.py` — `git ls-files` is index-only; an untracked file is
     invisible to `_collect_source_files()` and the harness would exit 0 vacuously
   - Run `python3 <tmp>/bin/check-green-doc-tense` (the copy in the temp repo, not the
     live script from the working tree)
   - Assert the process exits 1 (violation detected)
   - Assert the output contains a FAIL line naming `bin/violating.py` and the pattern
     label (e.g., `"FAIL [bin/violating.py:1]: Pattern 32 ..."`); check the literal
     violation output prefix, not just the exit code
   This validates the full integration path: `_collect_source_files()` discovers the `.py`
   file, `_is_comment_line()` returns True for the `#` line, and the pattern fires. This is
   the df-validation self-application smoke test confirming PG-W84-010 is exercised end-to-end.

10. **Sibling-prose sweep (F-009):**
    - `bin/check-green-doc-tense` module docstring (lines 2–4 scope text, lines 26–30 TOKEN LIST
      preamble, lines 87–88 comment-anchoring note, ALLOWLIST heading at ~:90): update scope
      declarations from "tests/*.rs and src/**/*.rs" to include "bin/*.py". Update token list
      to document Patterns 30–37.
    - Lines :577 ("in test files" failure summary) and :581–585 (explanatory prose): update
      scope descriptions from "test files" to "source files" to include `bin/*.py`.
    - `.github/workflows/ci.yml` stale prose sites (COMMENT-ONLY changes; functional job
      steps unchanged):
      - Line ~442: update scope comment to include `"and bin/*.py"`
      - Line :434: job comment says "in test files" — stale once `bin/*.py` is in scope;
        update to "in tracked source files (*.rs and bin/*.py)"
      - Line :462: step name "Scan for stale RED-phase comment headers in test files"
        — stale; update to "Scan for stale RED-phase comment headers in source files"

11. **Add CHANGELOG `[Unreleased]` entry (AC-183-005):** One-line entry describing the
    scan-glob + language-scoped comment-detection + TIER-1 behavioral-absence pattern
    extensions (Patterns 30–37), referencing PG-W84-010, PG-W85-003, STORY-183.

12. **Run full self-test and zero-FP gate check (AC-183-001/002/006):**
    ```bash
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
  and `is expected to` phrasing (D-506, PG-W85-003, convergence report lines 63-66). STORY-183
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
- **Action SHA-pin policy:** STORY-183 makes no functional `ci.yml` changes (comment-only updates to lines :434, :442, and :462 step-name only); no new SHA pins required. The ACTION-PIN-GATE job is unaffected.
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
| `bin/check-green-doc-tense` | Modify | Rename `_collect_rust_files`→`_collect_source_files`, extend with `bin/*.py` glob; extend language-scoped `_is_comment_line(stripped, suffix)`; add Patterns 30–37 to `_VIOLATION_PATTERNS`; update module docstring scope text and token list |
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
- **Points rationale (5 pts):** Expanded from 3 pts (v1.0/v1.1) due to: (a) 8 new patterns
  (vs 2 originally; 11 proposed in v3, 3 falsified to TIER-2 in v4), (b) ~40 multi-line
  fixture conversions to single-line form, (c) 20+ new BAD/GOOD self-test cases. The
  40-fixture restructure is the primary scope driver. Point count unchanged from v1.2.
- **Develop PR:** All ACs can be batched in a single develop PR. CHANGELOG entry required
  (`bin/` changes trigger AC-158-001). No new CI step added.
- **Story scope clarification (F-007, F-W86S-P3-001):** Retitled from "Full TIER-1 Token
  Coverage" to "TIER-1 Behavioral-Absence Token Coverage". Residual TIER-1 tokens NOT covered
  by this story:
  - §(d)/§(a) bare-word markers with phrase-level coverage only: `scaffold`, `uncalled`,
    `stub`, `skeleton` — phrase patterns (e.g., Pattern 25) cover the actionable form; bare
    words have ~91 legitimate uses and are excluded per the policy's phrase-level design
  - Bare `MUST FAIL` — TIER-2 per v4; tool catches specific phrases (Patterns 23-24) only
  - Three tokens falsified and moved to TIER-2 in v4 (F-W86S-P3-001): `no .* arm`,
    `not yet implemented`, `currently FAILS` — see Background §PG-W85-003 TIER-2 section
  - Pending-TIER-1 follow-up: `unimplemented!()` — 0 live hits per v4 grep (2026-07-25);
    pending addition after this story ships (track as follow-up)
- **Bare `RED` markers re-tiered TIER-2 per DF-GREEN-DOC-TENSE-SWEEP v6:** Pattern 30
  (`Expected RED:` heading with colon) is retained TIER-1; bare standalone `RED` markers
  without semantic phrase context are TIER-2 per the v6 ruling (F-W86S-P6-009/010).
- **Deferred scrub obligation (F-W86S-P6-009/010, v6 ruling):** Two live stale sites
  adjudicated at wave-86 pass-6: `iec104_analyzer_tests.rs:6271` and
  `modbus_detection_tests.rs:2472/:2480` — owner: next maintenance sweep.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
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
