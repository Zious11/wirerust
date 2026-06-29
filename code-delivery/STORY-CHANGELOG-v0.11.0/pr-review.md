# PR #340 Review — docs(changelog): complete v0.11.0 ENIP section

**Verdict:** APPROVE

**Scope:** Documentation-only. +111 lines to `CHANGELOG.md` (single file). No source, tests, build, or CI files touched.

## Summary

This PR backfills the missing `### Added` and `### Changed` sections of the `[0.11.0] - 2026-06-29` release (the headline ENIP/CIP analyzer) and adds two ENIP-specific `### Fixed` entries above the pre-existing Modbus/DNP3 EC-X1/EC-X2 bullets. All cited PR numbers, the umbrella issue, the merge-commit SHA, and the release date were independently verified against `gh pr`/`gh issue`/`git log`. Existing Fixed bullets and the footer compare-link block are byte-identical to `develop`. No blocking issues found.

## Checklist Results

| # | Criterion | Result |
|---|-----------|--------|
| 1 | Keep a Changelog section ordering (Added → Changed → Fixed) | PASS — final order on branch is Added (L12) → Changed (L95) → Fixed (L110) inside `[0.11.0]`. |
| 2 | Cited PR numbers correspond to merged PRs | PASS — every individually-bracketed PR (#317, #318, #319, #320, #321, #323, #324, #326, #327, #328, #329, #330, #331, #332, #333) verified MERGED via `gh pr view`. Issue #316 verified as the umbrella CLOSED issue ("feat(enip): add EtherNet/IP + CIP ICS analyzer (SS-17)"). Commit `b9b2e93` verified in `git log` ("ci(test): add green-doc-tense gate…"). |
| 3 | Existing Fixed bullets preserved unchanged | PASS — `git diff develop...HEAD` shows zero `-` lines (additions only). The four pre-existing Modbus/DNP3 EC-X1/EC-X2 Fixed bullets are untouched. |
| 4 | Footer compare-link syntax intact | PASS — footer is unchanged on the PR branch; `[0.11.0]: …compare/v0.10.0...v0.11.0` and `[Unreleased]: …compare/v0.11.0...HEAD` still present and well-formed. |
| 5 | Past/present tense, no aspirational markers | PASS — the only matches for "will/planned/future" appear inside a quoted description of the `green-doc-tense` CI gate itself (listing the markers the gate forbids), not as forward-looking claims about wirerust behavior. All new prose is in present/past tense ("now analyzes", "parses", "classifies", "added", "fired", "promoted", "discharged"). |
| 6 | v0.11.0 date accurate | PASS — `2026-06-29` matches today's date and matches the existing header on `develop`. |
| 7 | No other version sections modified | PASS — diff hunks are entirely within the `[0.11.0]` block (lines 9–127 of the new file). Sections for `[Unreleased]`, `[0.10.0]`, `[0.9.4]`, etc. and the footer link table are untouched. |

## Independent Verification Performed

- `git diff develop...docs/changelog-v0.11.0-complete --stat` → `CHANGELOG.md | 111 +++` (single-file, additive only).
- `gh pr view <N>` for every cited PR (#317–#324, #326–#334) → all MERGED with titles matching the changelog narrative (e.g. PR #328 title "fix(enip): resolve source_ip to client via port-44818 heuristic [#316]" matches the Fixed bullet about per-direction source-IP attribution).
- `gh issue view 316` → CLOSED, title "feat(enip): add EtherNet/IP + CIP ICS analyzer (SS-17)" — confirms the umbrella feature reference.
- `git show b9b2e93` → present in history, matches the green-doc-tense-gate description.
- Footer compare-link block on PR branch byte-identical to `develop`.

## Findings

### Non-blocking observations (NIT — optional, do not block merge)

1. **`PRs #317–#334` range wording.** The header bullet says "PRs #317–#334"; PR #325 inside that numeric range is an *open* Dependabot PR ("chore(deps): bump softprops/action-gh-release") unrelated to ENIP. The range expression is reasonable shorthand and every PR that is actually attributed in a bracket citation is real and merged, so this is not misleading in practice — but if you want strict precision, "PRs #317–#324, #326–#334" would be exact. Optional, do not block.

2. **`b9b2e93` is a merge commit, not the squash commit.** The Changed entry tags `[PR #321, b9b2e93]`; `b9b2e93` is actually the merge commit on `develop` (Merge: 7f040de 8bcf0e9). That's fine for traceability — readers can navigate from either the PR number or the SHA — but if the project convention is to cite the squashed commit on `develop`, double-check this matches the convention used elsewhere in the changelog. Optional.

3. **Heavy detail density.** The Added block is unusually long for a CHANGELOG (≈80 lines for one feature). This is a stylistic call — for a headline release with 13 stories, the level of detail is defensible, and it follows the precedent set by earlier high-detail entries in this file. No action needed.

### Blocking issues

None.

## Recommendation

Approve and merge. This is a high-quality, fact-checked changelog backfill that closes the documentation gap left when v0.11.0 was tagged without the ENIP section. Every claim is independently verifiable against merged PRs and the commit graph.
