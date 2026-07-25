---
document_type: story
story_id: STORY-183
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-25T00:00:00Z
phase: f7
level: feature
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
inputs: []
input-hash: "d41d8cd"
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
- **So that** Python files in `bin/` containing stale RED-phase documentation are never silently
  skipped by the gate (PG-W84-010), and the class of stale phrases discovered by the adversary
  during STORY-180 pass-1 (PG-W85-003, 9 stale sites) can no longer escape the gate undetected

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; bin/ tooling change only)_

## Background

### PG-W84-010 — bin/check-green-doc-tense scans only *.rs files

`bin/check-green-doc-tense` (Python) collects files via `_collect_rust_files()` (line ~466),
which runs `git ls-files -- tests/*.rs src/**/*.rs` and filters with `line.endswith(".rs")`.
The scan explicitly excludes all `bin/*.py` files. During STORY-176 adversarial pass-1
(wave-84), the adversary caught stale RED-phase prose in `bin/test_check_green_doc_tense.py`
that had passed the gate (finding F-S176P1-003). The root cause was confirmed in
`df-validation-2026-07-25.md §PG-W84-010` (LOCAL-CARRY-FORWARD, HIGH confidence): the
module docstring of `_collect_rust_files()` explicitly states the scan is Rust-only.

### PG-W85-003 — Missing "Expected RED:" and "currently falls through" pattern classes

`bin/check-green-doc-tense` `_VIOLATION_PATTERNS` (line 217) contains 29 patterns
(patterns 1–29, with 26–29 added by STORY-176). The patterns list does NOT include:
- `Expected RED:` — a heading pattern indicating the test was authored as a RED-gate test
  but whose doc still describes the pre-pass state ("Expected RED: <stale state description>")
- `currently falls through` — a body phrase from implementation notes indicating code was
  in a non-final path state during story authorship

During STORY-180 per-story adversarial pass-1, the adversary found 9 stale sites of these
classes that the gate had not flagged (D-506, 2026-07-24). The gate exited 0 with no flag.

### DF-VALIDATION coupling ruling

`df-validation-2026-07-25.md §Cross-Finding Observations, point 2` (coupling ruling):
"PG-W84-010 (scan glob) and PG-W85-003 (pattern set) both target `bin/check-green-doc-tense`
and should be one story. Delivering the glob extension alone would extend the scan to
`bin/*.py` without the patterns that actually live there." STORY-183 combines both fixes.

### CHANGELOG obligation

`bin/` changes trigger AC-158-001 / PG-W71-CHANGELOG. A `[Unreleased]` CHANGELOG entry
is required for this story at delivery time (enforced by the `changelog-gate` CI job).

### CI wiring obligation

STORY-183 extends the EXISTING `bin/test_check_green_doc_tense.py` (not creating a new
file). L-W84-003 / AC-165-001 requires CI wiring only for NEW `bin/test_*.py` files.
No CI wiring change is required.

## Acceptance Criteria

### AC-183-001 (traces to PG-W84-010 — scan glob extension to bin/*.py)

`bin/check-green-doc-tense` is extended to include `bin/*.py` files in its prose scan.

- Given `_collect_rust_files()` at line ~466 currently runs `git ls-files -- tests/*.rs src/**/*.rs`
  and filters with `line.endswith(".rs")`, excluding all Python files including `bin/*.py`
- When the function is refactored (or a new unified `_collect_files()` function is added) to
  additionally run `git ls-files -- bin/*.py` and include entries ending with `.py`
- And the overall function name is updated (or an alias added) to reflect the broader scope
  (e.g., `_collect_source_files()` or with a `# type: (*.rs + bin/*.py)` comment)
- And the per-file comment detection logic in `_is_comment_line()` (line ~460) is extended
  to also detect Python comment lines (`stripped.startswith("#")`) in addition to Rust line
  comments (`stripped.startswith("//")`) — so that patterns do NOT fire on Python inline
  comments in `bin/*.py` files (which are not prose violations)

- Then `git ls-files -- bin/*.py | xargs bin/check-green-doc-tense` exits non-zero when
  any of the new patterns (AC-183-003, AC-183-004) appear in `bin/*.py` prose, and exits
  zero when the files contain no stale phrases

Verification:
```bash
# Confirm the script now processes bin/*.py
# Create a temp .py with a violation, confirm non-zero exit
echo '# Expected RED: something stale' > /tmp/test_violation.py
bin/check-green-doc-tense /tmp/test_violation.py
echo "Exit code: $?"  # Must be non-zero (1)
rm /tmp/test_violation.py

# Confirm clean py file exits 0
echo '# This is a green doc comment' > /tmp/test_clean.py
bin/check-green-doc-tense /tmp/test_clean.py
echo "Exit code: $?"  # Must be 0
rm /tmp/test_clean.py
```

### AC-183-002 (traces to PG-W84-010 — Python comment-line awareness)

The comment-line detection in `bin/check-green-doc-tense` correctly identifies Python
`#`-prefixed comment lines so that violation patterns do NOT fire on commented-out test
fixtures inside `bin/test_check_green_doc_tense.py`.

- Given `_is_comment_line()` currently only checks `stripped.startswith("//")` (Rust)
- When a Python file is being processed and a line starts with `#` (after stripping
  leading whitespace)
- Then `_is_comment_line()` returns True for that line, and no violation is reported even
  if the line content would otherwise match a `_VIOLATION_PATTERNS` entry

This ensures the self-test file `bin/test_check_green_doc_tense.py` can contain known-bad
fixture strings inside Python comment lines without self-reporting violations.

**Zero-false-positive requirement:** Running `bin/check-green-doc-tense` over the existing
`bin/` directory (after AC-183-001..AC-183-004 are delivered) MUST exit 0 — the gate must
not flag its own known-bad test fixture strings embedded in Python comment/string literals.

Verification:
```bash
# After delivery, full bin/ scan must exit 0
git ls-files -- bin/*.py | xargs -I{} bin/check-green-doc-tense {}
echo "Exit code: $?"  # Must be 0
# Also confirm via the self-test
python3 bin/test_check_green_doc_tense.py
echo "Exit code: $?"  # Must be 0
```

### AC-183-003 (traces to PG-W85-003 — "Expected RED:" heading pattern)

A new violation pattern is added to `_VIOLATION_PATTERNS` in `bin/check-green-doc-tense`
for the `Expected RED:` heading class.

- Given that no entry for `Expected RED` appears in `_VIOLATION_PATTERNS` (confirmed by
  `df-validation-2026-07-25.md §PG-W85-003` grep: "Neither `Expected RED` nor `falls through`
  appears in `_VIOLATION_PATTERNS`")
- When the new pattern is added (after the existing pattern 29 block):
  ```python
  # Pattern 30 (PG-W85-003): "Expected RED:" heading — stale pre-pass state in test doc
  (r'Expected RED:', 'expected-red-heading'),
  ```
  (The colon is the discriminator — it marks the heading form, distinguishing it from
  references like "this was expected RED before the fix")
- And the corresponding known-bad fixture is added to `bin/test_check_green_doc_tense.py`
  BAD_CASES list:
  ```python
  ("expected-red-heading violation", "## Expected RED: all assertions fail", "expected-red-heading"),
  ```
- And the corresponding known-good fixture is added to GOOD_CASES list:
  ```python
  ("expected-red-heading clean — past tense", "This was expected to fail (RED phase) before implementation"),
  ```

- Then `bin/check-green-doc-tense` detects `Expected RED:` in prose and exits non-zero,
  while not flagging past-tense references that do not use the colon form

**Zero-false-positive requirement:** The pattern must not flag:
- `# Expected RED: ...` (Python comment line — filtered by AC-183-002 comment detection)
- `// Expected RED: ...` (Rust comment line — filtered by existing comment detection)
- `expected_red` (snake_case identifier — different token; no colon)

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must pass (0 exit), including new expected-red-heading known-bad case
```

### AC-183-004 (traces to PG-W85-003 — "currently falls through" body-phrase pattern)

A new violation pattern is added to `_VIOLATION_PATTERNS` in `bin/check-green-doc-tense`
for the `currently falls through` body-phrase class.

- Given no entry for `currently falls through` appears in `_VIOLATION_PATTERNS` (confirmed
  by `df-validation-2026-07-25.md §PG-W85-003` grep)
- When the new pattern is added (after pattern 30):
  ```python
  # Pattern 31 (PG-W85-003): "currently falls through" — stale in-progress path description
  (r'currently falls through', 'currently-falls-through'),
  ```
  (The word "currently" is the discriminator, marking present-tense active stale prose;
  past-tense forms like "fell through before" are not caught — correct behavior)
- And the corresponding known-bad fixture is added to BAD_CASES:
  ```python
  ("currently-falls-through violation", "the packet currently falls through the dispatcher", "currently-falls-through"),
  ```
- And the corresponding known-good fixture is added to GOOD_CASES:
  ```python
  ("currently-falls-through clean — past tense", "before the fix, the packet fell through the dispatcher"),
  ```

- Then `bin/check-green-doc-tense` detects `currently falls through` in prose and exits
  non-zero, while not flagging past-tense or otherwise clean phrasings

**Zero-false-positive requirement:** The pattern must not flag:
- `# currently falls through` (Python comment — filtered by AC-183-002)
- `// currently falls through` (Rust comment — filtered by existing check)
- `currently_falls_through_handler` (snake_case identifier — not a prose sentence)

Verification:
```bash
python3 bin/test_check_green_doc_tense.py
# Must pass (0 exit), including new currently-falls-through known-bad case
```

### AC-183-005 (traces to AC-158-001 / PG-W71-CHANGELOG — CHANGELOG obligation for bin/ changes)

A `[Unreleased]` CHANGELOG entry is added for the bin/ changes in STORY-183.

- Given `bin/check-green-doc-tense` and `bin/test_check_green_doc_tense.py` are in the
  `bin/` directory, which is in the AC-158-001 changelog-gate trigger set
- When the implementing PR is opened, the PR MUST include a new item under `[Unreleased]`
  in `CHANGELOG.md`:
  ```
  - Extend `bin/check-green-doc-tense` scan glob to include `bin/*.py` prose (PG-W84-010),
    add Python comment-line detection, and add patterns for `Expected RED:` heading class
    and `currently falls through` body-phrase class (PG-W85-003, STORY-183)
  ```
- And the CI `changelog-gate` job passes on the PR

Verification:
```bash
# CI changelog-gate job must pass (automated)
# Manual verify: grep for [Unreleased] entry in CHANGELOG.md
grep -A 5 "Unreleased" CHANGELOG.md | grep -q "check-green-doc-tense" && echo PASS || echo FAIL
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Scan glob extension + Python comment detection | `bin/check-green-doc-tense` (amend) | develop |
| Pattern 30: `Expected RED:` + pattern 31: `currently falls through` | `bin/check-green-doc-tense` (amend `_VIOLATION_PATTERNS`) | develop |
| Self-test: new BAD_CASES + GOOD_CASES for patterns 30–31 | `bin/test_check_green_doc_tense.py` (amend) | develop |
| CHANGELOG entry | `CHANGELOG.md` (amend `[Unreleased]`) | develop |

**No `src/` changes, no `Cargo.toml` changes, no CI workflow changes.**
CHANGELOG obligation: YES — `bin/` changes trigger AC-158-001.
No new `bin/test_*.py` file — L-W84-003 / AC-165-001 CI-wiring obligation does NOT apply.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Rust comment `// Expected RED: stale heading` | NOT flagged — `_is_comment_line()` returns True for `//`-prefix (existing behavior) |
| EC-002 | Python comment `# Expected RED: stale heading` | NOT flagged — `_is_comment_line()` returns True for `#`-prefix (AC-183-002) |
| EC-003 | `# currently falls through` in test-fixture string literal in `bin/test_check_green_doc_tense.py` | NOT self-flagged — the fixture is inside a Python string, not prose; AC-183-002 comment detection + the gate's own skip-self logic must handle this cleanly |
| EC-004 | `_VIOLATION_PATTERNS` entry for `expected_red` or `expectedRed` (no colon) | Does NOT match pattern 30 (regex requires `Expected RED:` with capital E, space, and colon) — camelCase/snake_case identifiers are not flagged |
| EC-005 | `bin/test_check_green_doc_tense.py` self-test contains known-bad strings | Gate must exit 0 on a full `git ls-files -- bin/*.py` scan — the self-test file embeds known-bad strings as Python string literals, which are prose (not comment lines). The gate's existing safe-context mechanism (or new Python string-literal detection) must exclude these from flagging. |
| EC-006 | `_collect_source_files()` / unified function includes a `.py` file outside `bin/` | AC-183-001 specifies `git ls-files -- bin/*.py` — only `bin/` Python files are included, not `src/` or `tests/` Python files (no such files currently exist in this codebase) |
| EC-007 | `bin/` contains a `.py` file with ONLY `#`-comment lines matching a pattern | Gate exits 0 — all matching lines are comment lines, filtered by AC-183-002. This is the mechanism that allows `bin/test_check_green_doc_tense.py` BAD_CASES to live in a Python file scanned by the gate. |

**Note on EC-005:** The existing gate handles this in Rust files by relying on Rust test
functions being inside `#[cfg(test)]` blocks OR by comment detection. For Python, the
BAD_CASES strings are inside Python string literals (not comment lines) within a function.
Implementer must verify that `bin/check-green-doc-tense` handles Python string-literal
context (e.g., strings inside `BAD_CASES = [...]`) without false-positives. If the gate
DOES flag its own test fixtures, a `.check-green-doc-tense-ignore` pragma or a skip-file
mechanism may be required — this is an implementation decision for the implementer to
resolve, with zero-false-positive as the hard requirement.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `bin/check-green-doc-tense` | effectful-shell | Shell script (Python): performs filesystem I/O (git subprocess, file reads), consumes stdin/stdout. The `_VIOLATION_PATTERNS` list and `_collect_source_files()` are pure-data / pure-function additions but live inside an effectful script. |
| `bin/test_check_green_doc_tense.py` | effectful-shell | Test runner: spawns subprocesses, reads files, asserts on exit codes. No pure-core classification applies. |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| Story spec (this file) | ~4.0 k |
| `bin/check-green-doc-tense` (full script, ~600 lines) | ~5.0 k |
| `bin/test_check_green_doc_tense.py` (full file, ~700 lines) | ~5.5 k |
| `CHANGELOG.md` (recent unreleased section only) | ~0.5 k |
| Tool outputs overhead | ~1.0 k |
| **Total** | **~16.0 k** |
| Agent context window | 200 k (Sonnet) |
| **Budget usage** | **~8%** |

Well within context window. No story split required.

## Tasks

1. **Read `bin/check-green-doc-tense` fully** — understand `_collect_rust_files()` (line ~466),
   `_is_comment_line()` (line ~460), and `_VIOLATION_PATTERNS` (line 217, patterns 1–29).
   Read `bin/test_check_green_doc_tense.py` fully — understand BAD_CASES and GOOD_CASES
   format and the safe-context mechanism for embedded violation strings.

2. **Extend `_collect_source_files()` / scan glob (AC-183-001):** Add `git ls-files -- bin/*.py`
   alongside the existing Rust globs. Update the filter to accept `.py` endings.

3. **Extend `_is_comment_line()` for Python (AC-183-002):** Add `stripped.startswith("#")`
   branch. Verify against EC-001..EC-007 edge cases.

4. **Add patterns 30 and 31 to `_VIOLATION_PATTERNS` (AC-183-003, AC-183-004):**
   After existing pattern 29, add the two new entries with PG-W reference comments.

5. **Extend `bin/test_check_green_doc_tense.py` (AC-183-003, AC-183-004):** Add one
   known-bad and one known-good fixture per new pattern (4 new test cases total). Confirm
   the existing test runner picks them up and passes.

6. **Zero-false-positive gate check (AC-183-002):** Run
   `git ls-files -- bin/*.py | xargs -I{} bin/check-green-doc-tense {}` and confirm exit 0
   after all changes. This validates that the self-test's embedded known-bad fixtures do NOT
   cause false-positive flags.

7. **Run full self-test (AC-183-003, AC-183-004):** `python3 bin/test_check_green_doc_tense.py`
   must exit 0.

8. **Add CHANGELOG `[Unreleased]` entry (AC-183-005):** One-line entry describing the
   scan-glob + pattern extensions, referencing PG-W84-010, PG-W85-003, STORY-183.

9. **Develop PR:** All changes (`bin/check-green-doc-tense`, `bin/test_check_green_doc_tense.py`,
   `CHANGELOG.md`) in a single develop PR. CI `changelog-gate` must pass.

## Previous Story Intelligence

- **STORY-176 (wave-84, patterns 26–29):** Added skeleton/seam/compile-only/until-wired
  patterns (26–29) to `_VIOLATION_PATTERNS`. The wave-84 adversary found F-S176P1-003
  (stale Python doc missed by the gate) — PG-W84-010 directly tracks this gap. STORY-183
  adds the next two pattern classes (30–31) using the same pattern-addition methodology
  established in STORY-176. Read STORY-176's Tasks section before implementing for the
  exact BAD_CASES / GOOD_CASES format.
- **STORY-180 (wave-85, BC-2.19.029/030):** Adversarial pass-1 found 9 stale sites with
  `Expected RED:` and `currently falls through` phrasing that the gate had not caught
  (D-506, PG-W85-003). STORY-183 closes this gap directly.
- **Patterns 1–29 baseline:** AC-183-003 adds pattern 30 and AC-183-004 adds pattern 31.
  The existing 29-entry list in `_VIOLATION_PATTERNS` must not be modified — append only.

## Architecture Compliance Rules

- **Append-only `_VIOLATION_PATTERNS`:** Never remove or reorder existing patterns.
  New patterns are appended after the last existing pattern (currently 29). Numbering
  in comments must be sequential (30, 31, ...).
- **`bin/` changes require CHANGELOG (AC-158-001):** Any PR touching `bin/` files MUST
  include an `[Unreleased]` CHANGELOG entry. The `changelog-gate` CI job enforces this.
- **L-W84-003 / AC-165-001 CI wiring:** Not triggered — STORY-183 extends an existing
  `bin/test_*.py` file, not creating a new one. No `.github/workflows/ci.yml` amendment
  needed.
- **Action SHA-pin policy:** STORY-183 makes no `ci.yml` changes; no new SHA pins required.
- **Zero-false-positive hard requirement:** The gate MUST exit 0 on a full `bin/*.py` scan
  after delivery. If self-test embedded fixtures cause false positives, the implementer must
  resolve them (skip pragma or safe-context detection) before closing the story.

## Library & Framework Requirements

| Dependency | Version | Source |
|------------|---------|--------|
| Python | 3.10+ | `bin/` scripts use modern type syntax (CLAUDE.md) |
| No new Python packages | — | All changes use Python stdlib only |
| No new Cargo.toml deps | — | No Rust changes in this story |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `bin/check-green-doc-tense` | Modify | Extend `_collect_source_files()` glob, extend `_is_comment_line()`, add patterns 30–31 to `_VIOLATION_PATTERNS` |
| `bin/test_check_green_doc_tense.py` | Modify | Add 4 new fixtures (2 BAD_CASES + 2 GOOD_CASES) for patterns 30–31 |
| `CHANGELOG.md` | Modify | Add `[Unreleased]` entry for PG-W84-010 + PG-W85-003 (AC-183-005) |

**Forbidden modifications:** `src/**/*`, `Cargo.toml`, `.github/workflows/ci.yml`,
`tests/**/*`

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-25 | story-writer | Initial authorship — wave-86 STORY-CREATION BURST (D-516); combined PG-W84-010 + PG-W85-003 per DF-VALIDATION-001 coupling ruling; 3 pts; E-11; wave 86; grounded against bin/check-green-doc-tense line 217 (_VIOLATION_PATTERNS), line ~460 (_is_comment_line), line ~466 (_collect_rust_files), and bin/test_check_green_doc_tense.py BAD_CASES/GOOD_CASES format. |
