---
document_type: story
story_id: STORY-183
epic_id: E-11
version: "1.2"
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
inputs:
  - .factory/cycles/wave-084/lessons.md
  - .factory/cycles/wave-085/lessons.md
  - .factory/planning/df-validation-2026-07-25.md
input-hash: "3831f42"
---

# STORY-183: check-green-doc-tense: bin/*.py Prose Coverage + Full TIER-1 Token Coverage

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** 86
**Points:** 5
**Priority:** P2

## Narrative

- **As a** CI gate operator reviewing step-4 (green-doc-tense) gate results
- **I want** `bin/check-green-doc-tense` to scan `bin/*.py` files as well as `*.rs` files,
  and to implement all TIER-1 behavioral-absence tokens from DF-GREEN-DOC-TENSE-SWEEP v3
  (as revised by F-W86S-P2-006 PO ruling 2026-07-25) that are absent from the current
  28-tuple `_VIOLATION_PATTERNS` set
- **So that** Python files in `bin/` containing stale RED-phase documentation are never
  silently skipped by the gate (PG-W84-010), and the TIER-1 phrasing classes discovered during
  STORY-180 pass-1 (PG-W85-003, 9 stale sites, primarily `currently asserts`) can no longer
  escape the gate undetected

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; bin/ tooling change only)_

## Background

### Governing Policy

This story is governed by **DF-GREEN-DOC-TENSE-SWEEP v3** (`policies.yaml`, last-extended
2026-07-25). The v3 two-tier enforcement model (F-W86S-P2-006 ruling) is the canonical
authority for all pattern decisions in this story. Any conflict between this story's
acceptance criteria and DF-GREEN-DOC-TENSE-SWEEP v3 resolves in favor of the policy.

### PG-W84-010 — bin/check-green-doc-tense scans only *.rs files

`bin/check-green-doc-tense` collects files via `_collect_rust_files()` (line ~466), which
runs `git ls-files -- tests/*.rs src/**/*.rs` and filters with `line.endswith(".rs")`. The
scan explicitly excludes all `bin/*.py` files. During STORY-176 adversarial pass-1 (wave-84),
the adversary caught stale RED-phase prose in `bin/test_check_green_doc_tense.py` that had
passed the gate (finding F-S176P1-003). The root cause was confirmed in
`df-validation-2026-07-25.md §PG-W84-010`.

### PG-W85-003 — Missing TIER-1 behavioral-absence phrase classes

`bin/check-green-doc-tense` `_VIOLATION_PATTERNS` (line 217) contains **28 tuples**, labeled
Pattern 1 through Pattern 29 in the module docstring token list. **Ground truth (convergence
report lines 63-66, STORY-180 F-180-P1-003):** the 9 D-506 stale sites used `currently
asserts` and `is expected to` — NOT `Expected RED:` / `currently falls through` (the lesson
summary was incorrect; the convergence report is authoritative).

The following TIER-1 tokens (zero-FP automatable, 0 live legitimate uses each) are absent
from the existing 28 tuples (confirmed by grep of `bin/check-green-doc-tense`):
- `currently asserts` — primary D-506 class; 9 stale sites (convergence report line 64)
- `falls to the wildcard` — 0 live legitimate uses
- `currently fall` — covers "currently falls through", "currently fall through", etc.
- `no .* arm` — "no <X> arm" match-arm absence
- `doesn't exist yet` / `does not exist yet`
- `not yet implemented`
- `currently has NO`
- `currently satisfied by`
- `will be GREEN currently`
- `currently FAILS`

**TIER-2 tokens (context-dependent; MUST NOT be added to tool per F-W86S-P2-006 PO ruling):**
- `is expected to` — **secondary D-506 phrasing class** (convergence report line 64); 6+ live
  legitimate uses (e.g., "this test is expected to PASS on the current codebase"). The tool
  MUST NOT flag this token. Manual/adversarial sweep owns it.
- `falls through to` — 12 live legitimate uses accurately describing match-arm fallthrough.

Additionally, `Expected RED:` (0 live uses) may be added as Pattern 30 as an extra
zero-FP guard, even though it was NOT in the D-506 sites.

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
with `.py` extension (e.g., `bin/test_check_green_doc_tense.py`, `bin/compute-input-hash`,
`bin/changelog-gate-check`). Extension-less Python executables in `bin/` (e.g.,
`bin/check-green-doc-tense` itself, `bin/fetch-e2e-pcaps`) are **out of scope** for this
story — they would require shebang-based detection. The follow-up gap is noted for a
separate story if needed. **In-source `# Pattern NN:` comments** in `bin/check-green-doc-tense`
must use non-flagging wording (e.g., `# Pattern 30: 'Expected RED:' heading` with quotes
around the pattern phrase) so they do not trigger the gate if the tool is later included in
its own scan.

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
- And a self-test assertion verifies `_collect_source_files(repo_root)` returns a path set
  that includes `bin/test_check_green_doc_tense.py` as a concrete tracked `.py` file

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
  # Patterns 30-40: added to implement all TIER-1 behavioral-absence tokens
  # from DF-GREEN-DOC-TENSE-SWEEP v3 (F-W86S-P2-006 ruling 2026-07-25)
  # absent from the existing 28-tuple set (STORY-183, wave-86).
  # -----------------------------------------------------------------------
  (
      # Pattern 30: 'Expected RED:' heading — stale pre-pass state in test doc.
      # The colon is the discriminator: "Expected RED: <desc>" marks a heading form.
      # 0 live legitimate uses (as of wave-86). NOT in the D-506 9 stale sites.
      # Allowlist: bare "expected RED" without colon; "was expected to fail (RED phase)".
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

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must exit 0; both Pattern 30 BAD cases must appear in output as PASS
```

### AC-183-004 (traces to PG-W85-003 — "currently fall" body-phrase pattern, Pattern 31)

A new violation pattern is added to `_VIOLATION_PATTERNS` as the **30th tuple** (labeled
"Pattern 31") in `bin/check-green-doc-tense`.

- Given no entry for `currently fall` appears in `_VIOLATION_PATTERNS` (confirmed by grep)
- When the new tuple is appended after Pattern 30:
  ```python
  (
      # Pattern 31: "currently fall(s)" — stale in-progress dispatch description. Covers
      # "currently falls through", "currently fall through", "currently falls past", etc.
      # The word "currently" is the discriminator; `falls?\b` catches both "falls" and "fall".
      # 0 live legitimate uses of "currently fall*" form.
      # Allowlist: "fell through to" (past tense); "falls through to" without "currently"
      # (TIER-2 per DF-GREEN-DOC-TENSE-SWEEP v3 F-W86S-P2-006 ruling — MUST NOT be flagged).
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
      # Per DF-GREEN-DOC-TENSE-SWEEP v3 (F-W86S-P2-006 ruling, 2026-07-25),
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
# Use the authoritative gate rather than a grep:
bin/changelog-gate-check CHANGELOG.md
# Must exit 0 (entry present under [Unreleased])
```

### AC-183-006 (traces to PG-W84-010 — positive .py coverage proof)

A deliberate test proves that a `.py` file with a stale `#`-comment IS flagged by the gate
(positive coverage: the gate is not merely inert on Python files).

- Given `scan_file()` in `bin/check-green-doc-tense` is called with a Path to a temp `.py`
  file containing `# Expected RED: deliberate stale heading\n`
- When the self-test runner's BAD cases include the 4-tuple `(".py" form, see AC-183-003)` and
  the runner writes it as `bad_N.py` (via the runner extension in Task 7)
- Then `scan_file(bad_N.py)` returns a non-empty violations list, confirming patterns fire on
  `#`-prefixed Python comment lines

This AC is satisfied by the `.py` 4-tuple entries added in AC-183-003 and AC-183-004 once the
runner extension is in place.

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# In output: both ".py form" PASS entries confirm positive .py coverage
```

### AC-183-007 (traces to PG-W85-003 — Patterns 32–40: remaining TIER-1 behavioral-absence tokens)

Nine additional violation patterns are added to `_VIOLATION_PATTERNS` as the **31st–39th
tuples** (labeled Patterns 32–40) in `bin/check-green-doc-tense`, implementing all remaining
TIER-1 tokens from DF-GREEN-DOC-TENSE-SWEEP v3 absent from the current 28-tuple set.

- Given none of the following TIER-1 tokens appear in the existing 28 tuples (confirmed by
  `grep` of `bin/check-green-doc-tense` returning no output for each)
- When the following nine tuples are appended after Pattern 31:
  ```python
  (
      # Pattern 32: 'currently asserts' — primary D-506 phrasing class (9 stale sites,
      # STORY-180 F-180-P1-003, convergence-report line 64). 0 live legitimate uses.
      "Pattern 32 (PG-W85-003): 'currently asserts' — RED-phase present-tense claim (AC-183-007)",
      re.compile(r"currently\s+asserts?\b", re.IGNORECASE),
  ),
  (
      # Pattern 33: 'falls to the wildcard' — wildcard-arm fallthrough; 0 live uses.
      # TIER-1 per DF-GREEN-DOC-TENSE-SWEEP v3. NOT the same as 'falls through to' (TIER-2).
      "Pattern 33 (PG-W85-003): 'falls to the wildcard' — RED-phase wildcard-arm fallthrough (AC-183-007)",
      re.compile(r"falls\s+to\s+the\s+wildcard", re.IGNORECASE),
  ),
  (
      # Pattern 34: 'no .* arm' — behavioral-absence phrase asserting a match arm is absent.
      # Catches 'no X arm', 'no dispatch arm', 'no handler arm', etc.
      # The \w+ requires at least one word token between 'no' and 'arm'.
      "Pattern 34 (PG-W85-003): 'no [X] arm' — present-tense match-arm absence claim (AC-183-007)",
      re.compile(r"\bno\b\s+\w+\s+arm\b", re.IGNORECASE),
  ),
  (
      # Pattern 35: 'does not exist yet' / 'doesn't exist yet' — negative-capability phrase.
      "Pattern 35 (PG-W85-003): 'does not / doesn't exist yet' — negative-capability claim (AC-183-007)",
      re.compile(r"does\s+not\s+exist\s+yet|doesn'?t\s+exist\s+yet", re.IGNORECASE),
  ),
  (
      # Pattern 36: 'not yet implemented' — classic stub marker surviving into GREEN.
      "Pattern 36 (PG-W85-003): 'not yet implemented' — stub-era capability absence (AC-183-007)",
      re.compile(r"\bnot\s+yet\s+implemented\b", re.IGNORECASE),
  ),
  (
      # Pattern 37: 'currently has NO' — present-tense absence claim.
      "Pattern 37 (PG-W85-003): 'currently has NO' — present-tense absence claim (AC-183-007)",
      re.compile(r"currently\s+has\s+no\b", re.IGNORECASE),
  ),
  (
      # Pattern 38: 'currently satisfied by' — passive implementation-status phrase.
      "Pattern 38 (PG-W85-003): 'currently satisfied by' — passive stub-status phrase (AC-183-007)",
      re.compile(r"currently\s+satisfied\s+by\b", re.IGNORECASE),
  ),
  (
      # Pattern 39: 'will be GREEN currently' — conditional tense claim asserting present RED.
      "Pattern 39 (PG-W85-003): 'will be GREEN currently' — conditional present-RED claim (AC-183-007)",
      re.compile(r"will\s+be\s+GREEN\s+currently", re.IGNORECASE),
  ),
  (
      # Pattern 40: 'currently FAILS' — present-false claim on a passing test.
      # IGNORECASE captures both 'currently FAILS' and 'currently fails'.
      # Past-tense 'failed', 'was failing', 'used to fail' are not matched.
      "Pattern 40 (PG-W85-003): 'currently FAILS' — present-false claim on passing test (AC-183-007)",
      re.compile(r"currently\s+fails?\b", re.IGNORECASE),
  ),
  ```
- And each new pattern has at least one BAD_CASE (`.rs` form) and one GOOD_CASE in
  `bin/test_check_green_doc_tense.py`; Patterns 32/33/34 additionally have `.py` BAD_CASES
- And the module docstring token list is updated to document Patterns 30–40 (28 existing +
  11 new = 39 total tuples)

- Then `python3 bin/test_check_green_doc_tense.py` exits 0 with all Pattern 32–40 BAD cases
  flagged and all GOOD cases clean

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must exit 0; Pattern 32–40 BAD cases must appear in output as PASS
```

### AC-183-008 (traces to PG-W85-003 — efficacy: TIER-1 D-506 phrasing covered; TIER-2 correctly excluded)

The final pattern set covers the TIER-1 phrasing from the 9 D-506 stale sites and produces
zero false positives on live TIER-2 sites.

- Given the 9 D-506 stale sites (STORY-180 F-180-P1-003, convergence report lines 63-66) used:
  - `currently asserts` (TIER-1) — covered by Pattern 32 ✓
  - `is expected to` (TIER-2) — correctly NOT flagged by the tool per F-W86S-P2-006 ruling ✓
- When both patterns are delivered and GOOD_CASES cover:
  - `"// The lax path falls through to the wildcard arm."` (TIER-2 zero-FP, Pattern 31 GOOD case)
  - `"// Before the fix, the packet fell through the dispatcher."` (past tense, not flagged)
  - `"// was expected to fail (RED phase) before the implementation shipped."` (allowlisted)
  - `"// this test is expected to PASS on the current codebase"` (TIER-2 'is expected to' — tool
    must NOT flag this; GOOD_CASE confirms the tool correctly skips it)
- Then `python3 bin/test_check_green_doc_tense.py` passes all GOOD_CASES related to patterns
  30–40

**TIER-2 non-flagging requirement (F-W86S-P2-006 ruling):**
The tool MUST NOT flag `is expected to`. Add a GOOD_CASE to confirm:
```python
(
    # DF-GREEN-DOC-TENSE-SWEEP v3 TIER-2: 'is expected to' has 6+ live legitimate uses.
    # Tool MUST NOT flag it — manual/adversarial sweep only. NOT a tool defect.
    "Efficacy: 'is expected to' NOT flagged (TIER-2 per F-W86S-P2-006 ruling)",
    "// this test is expected to PASS on the current codebase\n",
),
```

**Zero-FP regression spot-check:**
The following live sites use "falls through to" WITHOUT "currently" and MUST NOT be flagged:
- `tests/bc_2_16_d078_lax_malformed_tests.rs:18,90`
- `tests/main_story_089_tests.rs:648`
- `src/analyzer/tls.rs:930`

The Pattern 31 zero-FP GOOD_CASE and the gate's live run (`python3 bin/check-green-doc-tense`)
serve as the combined regression guard.

Verification:
```bash
# Run self-test — all Pattern 30-40 GOOD_CASES must pass:
python3 bin/test_check_green_doc_tense.py

# Zero-FP regression check on live codebase (must exit 0):
python3 bin/check-green-doc-tense
# If this exits 0, the 12 live 'falls through to' sites plus all 'is expected to'
# sites are clean — zero false positives from Patterns 30-40.
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Scan glob extension + Python comment-line scan eligibility | `bin/check-green-doc-tense` (amend `_collect_rust_files` → `_collect_source_files`, `_is_comment_line`, `main`) | develop |
| Patterns 30–40 (28 tuples → 39 tuples) | `bin/check-green-doc-tense` (amend `_VIOLATION_PATTERNS`) | develop |
| Runner `.py` extension support + 20+ new BAD/GOOD cases | `bin/test_check_green_doc_tense.py` (amend) | develop |
| Stale-comment scrub (lines 258/261) | `bin/test_check_green_doc_tense.py` (amend — reword two #-prefixed lines) | develop |
| String-literal false-positive resolution (~40 multi-line fixtures → single-line) | `bin/test_check_green_doc_tense.py` (amend) | develop |
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
| EC-003 | Python non-comment line containing `Expected RED:` (e.g., string literal `"// Expected RED: ...\n"`) | NOT flagged — after stripping, line begins with `"`, not `//` or `#`; `_is_comment_line()` returns False; scan skips it |
| EC-004 | `expected_red` (snake_case identifier) in a non-comment line | NOT flagged — non-comment line is skipped |
| EC-005 | `bin/test_check_green_doc_tense.py` existing multi-line string literals contain `//` lines with violation patterns | These produce 40 false-positive violations when the gate scans the file. Resolution: convert each such fixture to single-line string format (Task 5). Zero-FP after conversion is a HARD requirement. |
| EC-006 | `bin/` contains a `.py` file with ONLY `#`-comment lines matching a pattern | IS flagged — `#`-prefixed lines are ELIGIBLE to be flagged (AC-183-002); desired behavior |
| EC-007 | `// currently falls through` in Rust comment line | IS scanned and IS flagged — `_is_comment_line()` returns True; Pattern 31 matches |
| EC-008 | `// falls through to the wildcard arm` (no "currently") | NOT flagged — Pattern 31 requires "currently" prefix; TIER-2 token by policy |
| EC-009 | `// this test is expected to PASS` | NOT flagged — `is expected to` is TIER-2 (F-W86S-P2-006 ruling); tool must not flag it |
| EC-010 | Extension-less Python executables in `bin/` (e.g., `bin/check-green-doc-tense`) | OUT OF SCOPE for this story — `git ls-files -- bin/*.py` glob only covers `.py` files; shebang detection deferred |

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
   File is 914 lines. Confirm 28 tuples and absence of all TIER-1 tokens via grep.

2. **Extend `_collect_source_files()` glob (AC-183-001):** Add `git ls-files -- bin/*.py`
   alongside the existing Rust globs. Accept `.py` endings. Update `main()` call site.
   Update the pass-message at ~591 to reflect both file types. Rename function from
   `_collect_rust_files` to `_collect_source_files` (or annotate with `# scope: *.rs + bin/*.py`).
   Add a self-test assertion confirming `_collect_source_files(repo_root)` returns a path
   set containing `bin/test_check_green_doc_tense.py`.

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
   After conversion, `python3 bin/check-green-doc-tense` MUST exit 0.

6. **Add Patterns 30–31 to `_VIOLATION_PATTERNS` (AC-183-003, AC-183-004):** After the
   Pattern 29 entry, add the section comment and two new tuples. Verify numbering: the 28th
   tuple is Pattern 29, the new 29th tuple is Pattern 30, the new 30th tuple is Pattern 31.
   Ensure in-source `# Pattern NN:` comments use quoted phrases (e.g., `# Pattern 30: 'Expected RED:'`)
   so they do not flag themselves if later included in the scan.

7. **Extend `bin/test_check_green_doc_tense.py` runner for `.py` extension (AC-183-003/004/006):**
   In the BAD_CASES loop, use `entry[3]` as the file extension when present:
   ```python
   ext = entry[3] if len(entry) > 3 else ".rs"
   p = _tmpfile(content, tmp, f"bad_{passed}{ext}")
   ```
   Add the 4 new BAD_CASES (2 for Pattern 30 — `.rs` and `.py`; 2 for Pattern 31 — `.rs`
   and `.py`). Add 3 new GOOD_CASES (Pattern 30 allowlist, Pattern 31 allowlist, Pattern 31
   zero-FP for bare "falls through to" with verbatim PO ruling).

8. **Add Patterns 32–40 and self-test cases (AC-183-007):** Append 9 new tuples to
   `_VIOLATION_PATTERNS` as detailed in AC-183-007. For each pattern add at minimum one
   BAD_CASE `.rs` form and one GOOD_CASE. For Patterns 32–34 also add `.py` BAD_CASES.
   Add the efficacy GOOD_CASE for `is expected to` (AC-183-008) confirming the tool does
   NOT flag it.

9. **Sibling-prose sweep (F-009):**
   - `bin/check-green-doc-tense` module docstring (lines 2–4 scope text, lines 26–30 TOKEN LIST
     preamble, lines 87–88 ALLOWLIST preamble): update scope declarations from "tests/*.rs and
     src/**/*.rs" to include "bin/*.py". Update token list to document Patterns 30–40.
   - `.github/workflows/ci.yml` comment at line ~442: update
     `"bin/check-green-doc-tense scans tracked tests/*.rs and src/**/*.rs"` to include
     `"and bin/*.py"`. This is a COMMENT-ONLY change; the functional job steps are unchanged.

10. **Add CHANGELOG `[Unreleased]` entry (AC-183-005):** One-line entry describing the
    scan-glob + comment-detection + full TIER-1 pattern extensions (Patterns 30–40),
    referencing PG-W84-010, PG-W85-003, STORY-183.

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
  PG-W84-010 tracks this gap. STORY-183 adds patterns 30–40 using the same append-only
  methodology. Read STORY-176 Tasks before implementing for BAD_CASES / GOOD_CASES format.
- **STORY-180 (wave-85):** Adversarial pass-1 found 9 stale sites with `currently asserts`
  and `is expected to` phrasing (D-506, PG-W85-003, convergence report lines 63-66). STORY-183
  adds Pattern 32 (`currently asserts` — TIER-1) as the primary fix. `is expected to` is
  TIER-2 and correctly excluded from the tool per F-W86S-P2-006 PO ruling.
- **Patterns 1–29 baseline:** 28 tuples, labeled Pattern 1 through Pattern 29. Pattern 29 is
  the "until … wired" entry (last in the `# Patterns 26-29` block). Append-only; never reorder
  or remove existing entries. New entries become tuples 29–39 (Patterns 30–40).

## Architecture Compliance Rules

- **Append-only `_VIOLATION_PATTERNS`:** Never remove or reorder existing patterns. New
  patterns are appended as the 29th–39th tuples with sequential label numbers (30–40).
- **`bin/` changes require CHANGELOG (AC-158-001):** Any PR touching `bin/` files MUST include
  an `[Unreleased]` CHANGELOG entry. The `changelog-gate` CI job enforces this.
- **L-W84-003 / AC-165-001 CI wiring:** Not triggered — STORY-183 extends an existing
  `bin/test_*.py` file. No new `.github/workflows/ci.yml` job steps required.
- **Action SHA-pin policy:** STORY-183 makes no functional `ci.yml` changes (comment-only
  update to line ~442); no new SHA pins required. The ACTION-PIN-GATE job is unaffected.
- **Zero-false-positive hard requirement:** `python3 bin/check-green-doc-tense` MUST exit 0
  after delivery. The implementer resolves string-literal FPs by converting multi-line
  fixtures to single-line form (Task 5). No skip-file pragma is permitted on the self-test
  file — it must remain in the scan set (PG-W84-010 self-application requirement).
- **`_is_comment_line()` semantics:** INCLUSIVE (returning True = ELIGIBLE TO BE FLAGGED).
  Extending it for `#` makes Python comment lines scannable, not exempt.
- **TIER-2 exclusion is policy, not a defect:** Asserting the tool does NOT flag `is expected
  to` or `falls through to` is correct behavior per DF-GREEN-DOC-TENSE-SWEEP v3
  (F-W86S-P2-006 ruling). No adversarial finding can overturn this PO ruling.

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Python | 3.10+ | `bin/` scripts use modern type syntax (CLAUDE.md) |
| No new Python packages | — | All changes use Python stdlib only |
| No new Cargo.toml deps | — | No Rust changes in this story |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `bin/check-green-doc-tense` | Modify | Extend `_collect_source_files()`, extend `_is_comment_line()`, add Patterns 30–40 to `_VIOLATION_PATTERNS`, update module docstring scope text and token list |
| `bin/test_check_green_doc_tense.py` | Modify | Runner `.py` extension support; 20+ new BAD_CASES + 10+ new GOOD_CASES; scrub lines ~258/~261; convert ~40 multi-line fixtures to single-line form |
| `CHANGELOG.md` | Modify | Add `[Unreleased]` entry for PG-W84-010 + PG-W85-003 (AC-183-005) |
| `.github/workflows/ci.yml` | Modify (comment line ~442 only) | Update scope comment to include `bin/*.py`; no functional job changes |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`
**Note:** `tests/**/*` files must NOT be modified by this story. Reading/grepping test files
for the efficacy zero-FP check (AC-183-008) is read-only and permitted.

## Notes

- **DF-VALIDATION-001 status:** PG-W84-010 and PG-W85-003 are LOCAL-CARRY-FORWARD per
  `df-validation-2026-07-25.md §PG-W84-010` and `§PG-W85-003` (HIGH confidence). No upstream
  filing required. PG-W84-010 + PG-W85-003 LOCAL-CARRY-FORWARD dispositions per
  DF-VALIDATION-001 (mirroring STORY-182's Notes pattern).
- **No behavioral contract required:** E-11 convention.
- **Points rationale (5 pts):** Expanded from 3 pts (v1.0/v1.1) due to: (a) 11 new patterns
  (vs 2 originally), (b) ~40 multi-line fixture conversions to single-line form, (c) 20+ new
  BAD/GOOD self-test cases. The 40-fixture restructure is the primary scope driver.
- **Develop PR:** All ACs can be batched in a single develop PR. CHANGELOG entry required
  (`bin/` changes trigger AC-158-001). No new CI step added.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-07-25 | story-writer | WAVE-86 PASS-2 REMEDIATION — F-001 CRIT (Pattern redesign: ground truth corrected to convergence-report lines 63-66: 9 D-506 sites used `currently asserts` (TIER-1→Pattern 32) and `is expected to` (TIER-2→manual sweep only); all missing TIER-1 tokens from DF-GREEN-DOC-TENSE-SWEEP v3 added as Patterns 32-40; AC-183-007 new; old AC-183-007 → AC-183-008 rewritten); F-002 HIGH (AC-183-001 adds positive .py coverage assertion for `_collect_source_files()`); F-005 HIGH (Task 5 Option B STRUCK — self-test file must remain in scan set per PG-W84-010; single-line format is the correct fix); F-006 MED (Pattern 31 GOOD_CASE updated with verbatim F-W86S-P2-006 PO ruling; governing policy reference added to Background); F-011 MED (Task 5 corrected: scope 40 //‐lines + 2 # lines = 42 violations; single-line string format is the correct mechanic; points 3→5); F-015 MED (extension-less bin/ executables documented as out-of-scope with rationale; in-source Pattern NN comment phrasing note); F-018 LOW (token budget updated: test file ~950-960 lines after additions); F-020 LOW (Notes section added with DF-VALIDATION-001 disposition); F-022 LOW (tests/**/* removed from Forbidden; read-only access note added); F-023 NIT (AC-183-005 verification uses `bin/changelog-gate-check` citation). |
| 1.1 | 2026-07-25 | story-writer | WAVE-86 PASS-1 REMEDIATION — F-001 CRIT (bare invocations no CLI file args); F-002 HIGH (_is_comment_line() True=ELIGIBLE corrected + .py extension); F-003 HIGH (positive .py coverage AC-183-006 new); F-004 HIGH (BAD_CASES redesign .rs/.py cases + runner ext tuple); F-005 MED (pattern tuple shape corrected); F-006 MED (scrub lines 258/261 task); F-007 MED (AC-183-006 added); F-008 MED (pattern count 28-to-29 corrected); F-009 MED (ci.yml sibling-prose sweep); F-010 MED (AC-183-007 efficacy+zero-FP); F-020 MED (inputs: set); F-023 LOW (level:maintenance). |
| 1.0 | 2026-07-25 | story-writer | Initial authorship — wave-86 STORY-CREATION BURST (D-516). |
