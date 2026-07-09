# PR Review — #391 (F-W72G-P1-001)

**Verdict:** APPROVE
**Reviewer:** vsdd-factory:pr-reviewer (Opus 4.7)
**Branch:** `ci/w72-gate-fix-action-pin-guard` → `develop`
**Scope:** 10 files changed (+90/-32); CI/tooling/docs-only. No product Rust source modified.

---

## Summary

CI/tooling/docs-only fix PR closing wave-72 integration-gate findings:

- **F-W72G-P1-001 (HIGH):** `action-pin-gate` scan-target existence guard + positive-coverage assertion
- **SEC-W72-001 (LOW):** STORY-159 tape absolute-host-path scrub (5 tapes + evidence-report)
- **CR-001/002/003 (MINOR):** `bin/check-green-doc-tense` and `bin/lint-cycle-artifact` hardening
- **CR-005 (MINOR):** CHANGELOG restructure — split BREAKING enum casing (`### Changed (BREAKING)`) from additive `schema_version` (`### Added`)

All 8 review checklist items pass. **No blocking findings.**

---

## Detailed Verification

### 1. CI guards (`.github/workflows/ci.yml` action-pin-gate) — sound

- Directory-existence guard (`test -d .github/workflows/`) added before the scan; fails loudly on rename/removal. Matches the SEC-001 trust-boundary / help-provenance-gate pattern (PG-W71-CI-SCAN-GUARDS).
- Zero-file guard: `find .github/workflows/ -maxdepth 1 -name "*.yml" | wc -l` → fail if `-eq 0`. Shell logic correct.
- `VALIDATED` counter is incremented **after** both the local-ref skip (`ref_full` starts with `./`) and the allowlist skip (`continue` on match), so it reflects "remote, non-allowlisted refs actually validated." Final assertion `[ "${VALIDATED}" -eq 0 ]` → FAIL correctly detects a stripped/empty scan target that would otherwise trivially PASS.
- PASS line now emits the count: `"PASS: N remote action ref(s) validated, 0 mutable."` — informative, human-verifiable.

### 2. Tape path scrub — complete (5/5)

- All five STORY-159 `.tape` files scrubbed: `~/Documents/GITHUB/wirerust/.worktrees/STORY-159` → `<REPO-ROOT>/.worktrees/STORY-159`, consistent scrub-marker form.
- `evidence-report.md` narrative updated to describe the new form and explicitly note wave-72 scrub provenance.
- Binary `.gif`/`.webm` artifacts intentionally not re-rendered (historical evidence) — documented in the CHANGELOG entry, appropriate scope decision.
- Follow-up on scrub-gate doc `~/` tilde-expansion pattern extension (lives on factory-artifacts) is explicitly called out in the CHANGELOG — not swallowed.

### 3. CHANGELOG restructuring — correct per Keep a Changelog intent

- `### Added` split into `### Changed (BREAKING)` (verdict/confidence/category lowercase enum casing) and a separate `### Added` (`schema_version` envelope field). A breaking value-shape change is not purely additive, so this is the semantically correct classification.
- Cross-reference updated: `"the schema_version envelope field (see \`### Added\` below)"` — accurate anchor.
- Numbered list correctly renumbered from items 1–5 (with schema_version at #2) to items 1–4 in the Changed (BREAKING) section after schema_version was extracted to Added.
- Wave-72 entry (PG-W71-CI-SCAN-GUARDS + SEC-W72-001) includes correct provenance IDs and CWE-200 tag.

### 4. Bin tool changes — correct error propagation, no swallow

- `bin/check-green-doc-tense`: `subprocess.run(..., check=True)` now wrapped in `try/except CalledProcessError` → prints diagnostic to stderr and `sys.exit(1)`. Error surfaces with context; no swallow.
- Repo-root heuristic changed from `.git or Cargo.toml` to `.git or .factory/` — correct for worktree-mounted execution where the working directory may not contain `Cargo.toml` but always contains `.factory/`. `(candidate / ".git").exists()` matches both directory (main checkout) and file (worktree gitdir pointer).
- `bin/lint-cycle-artifact`: `_find_repo_root` no longer calls `sys.exit(2)` directly — raises `RuntimeError`, caught by `main()` and translated to `return 2`. Correct refactor for testability; exit code preserved.

### 5. VALIDATED counter regression analysis

False-failure risk is bounded: the counter fails only if every `uses:` in the tree is either a local `./` ref or on the `dtolnay/rust-toolchain@{stable,nightly}` allowlist. Current workflow tree contains many remote SHA-pinned refs, so `VALIDATED > 0` always. If a future refactor legitimately removed all remote refs, the FAIL message explicitly instructs to "update or disable this gate explicitly" — that is the intended semantic, not a false failure.

---

## Review Checklist

| # | Item | Result |
|---|---|---|
| 1 | Diff coherence — all changes scoped to F-W72G-P1-001 | PASS |
| 2 | Description accuracy — PR title matches diff scope | PASS |
| 3 | Test coverage — CI/docs/bin only; no product code | N/A (fix PR) |
| 4 | Demo evidence — tape scrub applied; evidence-report updated | PASS |
| 5 | Commit quality — 3 conventional commits, story ID present | PASS |
| 6 | Diff size — 90/32 across 10 files, small | PASS |
| 7 | Missing changes — CHANGELOG follow-up flagged as known | PASS |
| 8 | Dependency status — no upstream deps | N/A |

---

## Non-Blocking Observations (no changes requested)

- **nit / coverage:** The zero-file guard matches only `*.yml`; `*.yaml` (also valid GitHub Actions extension) is not counted. Not an issue for this repo (all workflows are `.yml`), but a possible future edge case if a `.yaml`-only workflow tree ever ships. Deferred.
- **nit / style:** `### Changed (BREAKING)` is a local heading annotation rather than strict Keep a Changelog vocabulary. Semantically correct; downstream tooling that greps for exact `### Changed` still matches. No change requested.
- **nit / error handling:** `bin/check-green-doc-tense` does not catch `FileNotFoundError` for a missing `git` binary. Acceptable — that is an environment failure that should crash noisily.

---

## Findings Table

| Severity | Category | Finding | Suggestion |
|---|---|---|---|
| — | — | No blocking findings | — |
| nit | coverage | `find … -name "*.yml"` misses `.yaml` extension | Extend to `\( -name "*.yml" -o -name "*.yaml" \)` if a `.yaml`-only workflow ever lands. Deferred. |
| nit | style | `### Changed (BREAKING)` is a local annotation | Strict Keep a Changelog uses plain `### Changed`; current form is more informative and acceptable. |
| nit | robustness | Missing `git` binary would crash with `FileNotFoundError` in `bin/check-green-doc-tense` | Acceptable — environment failure should crash noisily. No change. |

---

**Recommendation: APPROVE and merge when CI is green.**
