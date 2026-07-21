# PR #429 Review — fix(wave-84): gate code-review tooling-quality fixes

**Reviewer:** pr-reviewer (fresh-context, cognitive diversity)
**PR:** #429 — `fix/w84g-tooling-quality` → `develop`
**covered_sha:** `700c5424ab32f63af747e95e8da5a85f2e5f8b6f`
**Date:** 2026-07-20

## Verdict: APPROVE

All 5 findings (CR-002, CR-005, CR-006, SEC-003, OBS-001) are correctly
resolved. Every claim was independently verified against the diff, the regex
behavior, and live self-test runs. No blocking findings.

## Verification performed

### CR-005 — pattern 26 leading `\b` (`\bskeleton\s+compiles?\b`)
Confirmed correct narrowing. `exoskeleton compiles` → no match (fixed);
`harness skeleton compiles` → still matches. The `\b` correctly fails to fire
between `o` and `s` inside "exoskeleton". Pure narrowing, no ReDoS surface.

### CR-006 — pattern 28 leading `\b` (`\b(?:are|is)\s+(?:currently\s+)?compile-only`)
Confirmed pure narrowing. Mid-word `hare compile-only` → no match (fixed);
`bodies are compile-only` → still matches. Zero-width assertion.

### CR-002 — 2 new GOOD test cases
- `exoskeleton compiles` genuinely exercises the new pattern-26 leading-`\b`
  (without the change it would have matched).
- `until STORY-153 wired the handler` is suppressed by pattern 29's
  `(?!\s+(?:it|the|a|that|this|them)\b)` negative lookahead — verified passing.

### SEC-003 — `timeout=30` handler
Correct. `subprocess.run(..., timeout=30)` with `TimeoutExpired` re-raised as
`AssertionError`. List-form invocation (no shell injection). CWE-400 closed.

### OBS-001 — CHANGELOG backtick fix
Repaired line has 6 balanced backticks (`` `.gitignore` ``, `` `mutants.out*/` ``,
`` `bin/test_gitignore_mutants_glob.py` ``). Rendering corrected.

### bin/ CHANGELOG obligation
New `### Fixed` sub-section present in `[Unreleased]`. changelog-gate satisfied.
`### Fixed`-before-`### Changed` ordering matches repo precedent in `[0.13.0]`.

### Live test evidence (run on PR branch HEAD)
- `bin/test_check_green_doc_tense.py`: **93 passed, 0 failed** (matches claimed 91→93).
- `bin/test_gitignore_mutants_glob.py`: **2 passed, 0 failed**.

## Checklist (8/8)
Diff coherence OK · description accuracy OK (matches diff exactly) · test
coverage OK · demo evidence N/A (correctly justified, tooling-only) ·
commit/title semantic OK · diff size trivial (~69 lines) OK · no missing
changes OK · deps merged (PR #427, #428) OK.

## Findings

| Severity | Category | Finding | Suggestion |
|----------|----------|---------|------------|
| suggestion | test-coverage | CR-006 (pattern 28 leading-`\b`) ships without a dedicated GOOD test case; the `exoskeleton` case only covers pattern 26. | Add a case like `// hare compile-only` to lock in the mid-word exclusion for pattern 28. Low priority — the change is a provably safe narrowing. |
| nit | description | The second new test case is labeled "pattern (d) … CR-002" but exercises pattern 29's pre-existing negative lookahead (unchanged in this PR). | Harmless; it's a valid regression guard. Label implies new-behavior coverage. |

Neither finding blocks merge.
