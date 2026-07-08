# PR Review — #381 (F-W71-P1-001 wave-71 Unreleased CHANGELOG entries)

**Verdict:** APPROVE
**Reviewer:** pr-reviewer (Opus 4.7, fresh-context / cognitive-diversity)
**Base:** `develop`
**Head:** `docs/w71-unreleased-changelog`
**Scope:** docs-only, `CHANGELOG.md` +20 lines
**Date:** 2026-07-08

---

## Summary

This PR closes wave-71 gate finding **F-W71-P1-001 (process-gap, MEDIUM)** by
adding the three missing `[Unreleased]` CHANGELOG entries for the wave-71
merged PRs (#378 STORY-156, #379 STORY-150, #380 STORY-157). It matches the
wave-70 precedent (commit 87035da / PR #377). The diff is minimal, docs-only,
+20 lines to `CHANGELOG.md`, and introduces no code, tests, dependencies, or
runtime surface changes.

I verified every claim in the three new entries against the source PR bodies
and against the CLAUDE.md content on this branch. All statements are factually
accurate; placement under `Changed` / `Fixed` / `Tests / Internal` follows
Keep a Changelog conventions and the wave-70 precedent.

---

## Findings

**None (blocking / suggestion / nit).**

Per pr-reviewer rules, when no issues are found I must explain what was
verified rather than rubber-stamp. Verification detail below.

---

## Verification Detail

### Entry 1 — STORY-150 / PR #379 (Changed section)

Claim vs. source (PR #379 body, title
`refactor(tls): unify drain-loop dispatch arms (TLS-DRAIN-DUP-001) + Kani VP-039 + mutation re-run (STORY-150)`):

| Claim in CHANGELOG | Source in PR #379 | Verdict |
|---|---|---|
| "TLS handshake drain-loop DRY unification in `process_handshake_carry`" | PR title + "unifies the duplicated C2S and S2C dispatch arms in TlsAnalyzer::process_handshake_carry" | ACCURATE |
| "Single `msg_bytes` extraction and single `parse_tls_message_handshake` call site" | ADR: "consolidate msg_bytes extraction and parse_tls_message_handshake into one shared call site" | ACCURATE |
| "direction-guarded dispatch arms ... defense-in-depth refactor" | F-150-P1-003 (LOW): "Missing direction-guard defense-in-depth" resolved by commit e367f5e; `kani_vp039_direction_guard` (12 checks SUCCESSFUL) | ACCURATE |
| "Kani VP-039 3/3 proofs re-verified" | Test Evidence: "Kani VP-039 harnesses 3/3 SUCCESSFUL (75/12/12 checks)" | ACCURATE |
| "zero new mutation survivors" | Test Evidence: "42/45 caught (93.3%); 3 pre-existing survivors in compute_ja3 ... Zero new survivors" | ACCURATE |

Placement under `Changed` is consistent with the wave-70 precedent for
STORY-149 TLS carry-path restructure (CHANGELOG.md:21–27).

### Entry 2 — STORY-157 / PR #380 (Fixed section)

Claim vs. source (PR #380 body, title
`fix(tooling): input-hash empty-inputs + inline-comment handling + hook-divergence docs (STORY-157)`):

| Claim in CHANGELOG | Source | Verdict |
|---|---|---|
| "`inputs: []` (empty inputs list) now derives hash `d41d8cd` (MD5 of empty bytes) instead of raising an error" | AC-157-003 + ADR: "return [] and short-circuit compute_hash to hashlib.md5(b'').hexdigest()[:7]" | ACCURATE |
| "inline ` # comment` suffixes are stripped from input path entries before file resolution" | AC-157-010 + ADR: "Strip everything from ` #` onward from each input path entry before file resolution" | ACCURATE |
| "CLAUDE.md documents the canonical-tool/hook divergence (PG-HASH-HOOK-DIVERGENCE)" | CLAUDE.md:176 `### Known Tool Divergences (PG-HASH-HOOK-DIVERGENCE)` and CLAUDE.md:202 references confirmed on branch | ACCURATE |
| "edge cases, and Python 3.10+ floor" | CLAUDE.md:100–101 `requires Python 3.10+ — the tool uses modern type syntax` | ACCURATE |

Placement under `Fixed` matches conventional-commit `fix` type.

### Entry 3 — STORY-156 / PR #378 (Tests / Internal section)

Claim vs. source (PR #378 body, title
`test(STORY-156): ARP findings unbounded-cap documentation + regression test (BC-2.16.016)`):

| Claim in CHANGELOG | Source | Verdict |
|---|---|---|
| "BC-2.16.016 ARP unbounded-findings coverage" | PR scope: BC-2.16.016 v1.1 ARP findings unbounded-cap | ACCURATE |
| "Standalone `summarize()` no-`dropped_findings` regression pin closes the coverage gap for BC-2.16.016 unbounded-findings behavior" | AC-004 test `test_BC_2_16_016_summarize_has_no_dropped_findings_key` (commit 7e4fe6d) | ACCURATE |
| "docstring anchor corrected" | Docstring citation fix at commit a61950f | ACCURATE |
| "CLI `--arp` `long_help` unbounded-findings documentation coverage pinned" | AC-001 test `test_BC_2_16_016_cli_help_documents_arp_findings_unbounded` (pre-existing, eca21e9; confirmed as pinning coverage) | ACCURATE |

Placement under a new `Tests / Internal` subsection is a reasonable
extension of the v0.11.5 `Docs / Internal` precedent (CHANGELOG.md:71),
appropriately re-labeled for a test/docstring-only PR.

---

## Checklist (pr-reviewer 8-item)

1. **Diff Coherence** — PASS. All +20 lines relate to F-W71-P1-001 remediation.
2. **Description Accuracy** — PASS. PR body correctly describes the three added entries and cites the wave-70 precedent (commit 87035da / PR #377).
3. **Test Coverage** — N/A. Docs-only change; no source under test.
4. **Demo Evidence** — N/A explicitly per PR body. Correct for a CHANGELOG text edit.
5. **Commit Quality** — PASS. Semantic PR title `docs: wave-71 unreleased changelog entries (F-W71-P1-001)` uses allowed `docs` type.
6. **Diff Size** — PASS. +20 lines, well under any threshold.
7. **Missing Changes** — PASS. Three wave-71 merged PRs (#378, #379, #380) all covered; no missing story entries.
8. **Dependency Status** — PASS. All three upstream PRs already merged to develop.

---

## Verdict

**APPROVE.** No blocking, suggestion, or nit findings. The three entries are
factually accurate against the source PRs, placement follows Keep a Changelog
conventions and the wave-70 precedent, and the diff is scoped to exactly the
gate-remediation objective.
