# PR #428 Review — fix/w84g-changelog-ac176-003 → develop

**Verdict: APPROVE**
**Covered SHA:** `ec82788949fadca02162987c20a9bea79b458628`

## Context

Wave-84 gate fix for finding **F-W84G-P1-001**: STORY-176 (#427) shipped
AC-176-003 without an `[Unreleased]` CHANGELOG entry, and the content-blind
`changelog-gate` CI job passed anyway (the PR also touched other `bin/`
files, so the trigger was satisfied without the AC-176-003 bullet being
present). This PR adds the missing bullet.

**Scope:** single file, `CHANGELOG.md`, +10 lines. Documentation-only.

## Checklist

| # | Item | Result |
|---|------|--------|
| 1 | Diff coherence | PASS — single-file docs change matching the fix purpose |
| 2 | Description accuracy | PASS — see per-claim verification below |
| 3 | Test coverage | N/A — docs-only; no code changed |
| 4 | Demo evidence | N/A — CHANGELOG-only fix |
| 5 | Commit quality | PASS — `docs(wave-84): ... (F-W84G-P1-001)`, conventional + finding ref |
| 6 | Diff size | PASS — 10 lines |
| 7 | Missing changes | PASS — nothing else required for this fix |
| 8 | Dependency status | PASS — STORY-176 code already merged (#427, 595cdba8) |

## Per-claim factual verification

All claims in the new CHANGELOG bullet were cross-checked against the actual
delivered artifacts:

- "`.gitignore` gains `mutants.out*/` under the cargo-mutants section" —
  verified: `.gitignore:12`, under the `# cargo-mutants output directories` comment.
- "covering `mutants.out/` and `mutants.out.j4-invalid/`" — verified: these are
  the two paths asserted in `bin/test_gitignore_mutants_glob.py`.
- "complements the existing `mutants-f6*/` glob" — verified: `.gitignore:13`.
- "asserts both dirs are git-ignored via 2 `git check-ignore` assertions" —
  verified: exactly two `check_ignored()` checks in the self-test.
- "wired into CI's `bin-selftest` job" — verified: `.github/workflows/ci.yml:473`
  defines `bin-selftest`; lines 485-486 run the self-test.
- "was green on merge" — consistent: code merged in `595cdba8` (#427).

Placement verified: `CHANGELOG.md:49`, inside `## [Unreleased]` → `### Added`,
after the check-green-doc-tense entry and before the STORY-166 entry.

## Findings

### [NIT] Unbalanced backtick in the entry header (CHANGELOG.md:49)

| Field | Value |
|-------|-------|
| Severity | nit |
| Category | description |
| Finding | The header opens an inline-code backtick before `.gitignore` but never closes it within the header. Sibling entries use balanced backticks (e.g. STORY-166's `bin/validate-citations`), and this entry's own body correctly writes `.gitignore` with a closing backtick. |
| Suggestion | Add the closing backtick after `.gitignore`. Non-blocking cosmetic fix. |

## Verdict rationale

No blocking findings. The entry is factually accurate against every delivered
artifact, correctly placed, and closes F-W84G-P1-001. The single NIT is a
cosmetic markdown issue. This PR touches only `CHANGELOG.md`, so it does not
itself trip the changelog-gate trigger set (`src/`, `Cargo.toml`, `bin/`).

**APPROVE** — covered_sha `ec82788949fadca02162987c20a9bea79b458628`.
