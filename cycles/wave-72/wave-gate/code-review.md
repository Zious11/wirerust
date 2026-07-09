---
document_type: wave-gate-code-review
wave: wave-72
gate_event: integration-gate
reviewer: code-reviewer
timestamp: 2026-07-09T00:00:00Z
verdict: APPROVE-WITH-COMMENTS
pr_reviewed: "#391 (ci: harden action-pin-gate scan guard + wave-72 gate fixes)"
develop_sha_reviewed: 44f8c9ce57b1ebe7ea1d166628a2518ebf981997
finding_counts:
  minor: 5
  nit: 4
  total: 9
---

# Wave-72 Integration Gate — Code Review

**Gate:** wave-72 integration gate (D-414, 2026-07-09)
**Verdict:** APPROVE-WITH-COMMENTS (5 MINOR / 4 NIT; 0 BLOCKING)
**PR reviewed:** #391 "ci: harden action-pin-gate scan guard + wave-72 gate fixes"
**develop SHA reviewed:** `44f8c9ce57b1ebe7ea1d166628a2518ebf981997`

Per AC-158-006 (PG-W71-CODEREVIEW-ARTIFACT): this artifact enumerates every MINOR and NIT
finding from the gate-level code review together with its disposition.

---

## Code Review Findings

### MINOR Findings

| ID | Location | Summary | Disposition |
|----|----------|---------|-------------|
| CR-001 | `bin/lint-cycle-artifact` — `_find_repo_root` | Uses `sys.exit` inside a helper function (lint-cycle-artifact). `sys.exit` inside a library-style helper is an anti-pattern — it prevents callers from catching the error; `RuntimeError` (or a custom exception) is more composable and testable. | **FIXED — PR #391** (`RuntimeError` raised instead of `sys.exit` in `_find_repo_root`) |
| CR-002 | `bin/check-green-doc-tense` — git failure path | Raw traceback surfaced to the user on git subprocess failure. The git-failure code path should catch `subprocess.CalledProcessError`, format a clean error message, and exit 1 without exposing the raw traceback. | **FIXED — PR #391** (clean error path added) |
| CR-003 | `bin/compute-input-hash` / `bin/lint-cycle-artifact` — repo-root sentinel | The tools detect the repo root by searching for `.factory/`. If the `.factory/` directory is not present (e.g., on a `develop` checkout without the worktree mounted), the tools produce a confusing "repo root not found" error with no guidance. Adding `Cargo.toml` as a secondary sentinel makes the detection more robust and the error message more actionable. | **FIXED — PR #391** (`Cargo.toml` added as secondary sentinel) |
| CR-004 | `docs/adr/0012.md` — Decision 3a/3c | Decisions 3a and 3c duplicate each other in intent (both address the same "unclassified port" scenario with near-identical wording). This creates reader confusion about which decision is authoritative. One should be primary; the other should reference it. | **DEFERRED** — doc-debt; route to next maintenance sweep |
| CR-005 | `CHANGELOG.md` — BREAKING entry placement | The BREAKING change entry for JSON enum casing (introduced in wave-72) was placed under `### Added` rather than `### Changed (BREAKING)`. BREAKING changes have dedicated placement in Keep-a-Changelog convention and under the project's changelog discipline. | **FIXED — PR #391** (BREAKING entry moved from `### Added` to `### Changed (BREAKING)`) |

### NIT Findings

| ID | Location | Summary | Disposition |
|----|----------|---------|-------------|
| CR-006 | `bin/test_lint_cycle_artifact.py` — TC2 fixture | TC2 contains a duplicate assertion (`assert result.returncode == 0` appears twice consecutively) and a comment that describes the wrong test case (copy-paste from TC1). Both are cosmetic but reduce test-suite readability. | **DEFERRED** — maintenance |
| CR-007 | `bin/lint-cycle-artifact` — `_PARSE_ERRORS` tuple | `_PARSE_ERRORS` tuple is defined inside `main()` rather than at module level. Tuple literals defined inside function bodies are re-constructed on each call. At module level they are constructed once and reused. | **DEFERRED** — maintenance |
| CR-008 | `bin/lint-cycle-artifact` — SEC-001 guard idiom | The SEC-001 path-guard idiom (`if not path.resolve().is_relative_to(root)`) appears twice without a shared helper or cross-reference comment. Future maintainers may update one site without the other. | **DEFERRED** — maintenance |
| CR-009 | `bin/test_lint_cycle_artifact.py` — contains_key asserts | TC7 asserts `expected_key in story_bcs_set` after the set-equality check that already implies this. The redundant contains-key asserts add verbosity without additional coverage. | **DEFERRED** — maintenance |

---

## Adjacent Gate Findings and Dispositions

These findings were surfaced at the wave-72 integration gate outside the code review pass but
are recorded here for completeness per the gate artifact protocol.

| ID | Severity | Source | Summary | Disposition |
|----|----------|--------|---------|-------------|
| F-W72G-P1-001 | HIGH | Adversary Pass 1 | `action-pin-gate` CI job scanned 0 workflow files — scan-target existence guard missing, parallel to the trust-boundary gap fixed by STORY-158 AC-158-004. Process gap (PG-W71-CI-SCAN-GUARDS class). | **FIXED — PR #391** (existence guard added to action-pin-gate scan, positive-coverage count validated: 23 workflow action refs validated locally) |
| F-W72G-P1-002 | LOW | Adversary Pass 1 | `CLAUDE.md` placement of the new CHANGELOG obligation section (AC-158-002) is in the Git Workflow section rather than a dedicated Delivery section. Arguable whether a PR-time obligation belongs in Git Workflow. | **ACCEPTED** — orchestrator adjudication: governs PR-time artifacts; Git Workflow placement is defensible and consistent with adjacent obligations; no change required |
| SEC-W72-001 | LOW | Security review | STORY-159 demo tape files contained tilde-form home paths (`~/Documents/GITHUB/wirerust`) that bypassed the existing `/Users/` + `/home/` scrub gate (CWE-200, information disclosure). | **FIXED — PR #391** (5 tape files scrubbed; demo-evidence-scrub-gate.md extended to also reject `~/` paths per D-414 burst) |
| SEC-W72-002 | LOW | Security review | Carried LOW advisory from prior gate (security review of wave-72 delivery). DF-VALIDATION-001 pipeline pending. | **DEFERRED** — carried; DF-VALIDATION-001-gated before issue filing |
| SEC-W72-003 | LOW | Security review | Carried LOW advisory from prior gate (security review of wave-72 delivery). DF-VALIDATION-001 pipeline pending. | **DEFERRED** — carried; DF-VALIDATION-001-gated before issue filing |
| BLOCKING-01 | CONSISTENCY | Consistency audit | STORY-INDEX v3.31 body incomplete: STORY-161 catalog row status `draft` (should be `delivered`); v3.31 changelog header entry absent. | **FIXED — D-414 burst** (STORY-161 status draft→delivered; v3.31 header added; wave-72 progress row updated) |
| ADVISORY-01 | CONSISTENCY | Consistency audit | Wave-72 Wave Delivery Progress table row showed "DELIVERED & CLOSED" before integration gate close — label should reflect gate-in-progress state. | **FIXED — D-414 burst** (row updated to "DELIVERY COMPLETE (D-413) — integration gate IN PROGRESS") |
| ADVISORY-02 | CONSISTENCY | Consistency audit | Historical provenance note in STORY-INDEX. | **ACCEPTED** — historical; no change required |

---

## Summary

- **0 BLOCKING findings** at gate close (F-W72G-P1-001 HIGH fixed before gate close via PR #391).
- **5 MINOR findings:** CR-001/002/003/005 FIXED (PR #391); CR-004 DEFERRED to maintenance sweep.
- **4 NIT findings:** CR-006/007/008/009 all DEFERRED to maintenance.
- **Gate verdict:** APPROVE-WITH-COMMENTS — gate proceeds to adversary Pass 2.
