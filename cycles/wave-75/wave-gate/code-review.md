---
pass: wave-gate
wave: 75
story: STORY-165
reviewer: vsdd-factory:code-reviewer (Sonnet 4.6)
date: 2026-07-13
diff_range: d6e3be8..fa646ed
pr: 398
verdict: PASS
---

# Wave-75 Gate Code Review

## Scope

PR #398 — STORY-165 squash merge (ci: add bin-selftest job + register STORY-165 governance mandates in CLAUDE.md). Files reviewed:

- `.github/workflows/ci.yml` (+18 lines: new `bin-selftest` job at lines 464-480)
- `CLAUDE.md` (+2 Project References table rows at lines 251-252; +1 wording qualifier)

Review axes per gate instructions:
1. ci.yml `bin-selftest` job: YAML quality, job placement/ordering, naming, timeout, permissions minimality, checkout pin correctness, step naming, comment quality, consistency with sibling gate jobs, interaction risk with the 12 other jobs.
2. CLAUDE.md rows: table conformance, description accuracy vs the actual mandate docs (`.factory/maintenance/pr-description-row-verify-mandate.md` and `.factory/maintenance/delivery-doc-currency-protocol.md`), row ordering.
3. Maintainability flags: style drift, missed simplification, doc clarity.

## Context

Per-story convergence ran 9 passes (clean streak P7/P8/P9). Wave gate is at 2/3 clean. PR-level review was APPROVE with 0 findings. This is the fresh-eyes gate-level sweep.

## Verdict

**PASS.** No BLOCKING, MAJOR, or MINOR findings. One NIT (hardcoded test counts in step names and comment). The bin-selftest job is structurally correct, minimal, and consistent with sibling gate jobs. The two CLAUDE.md rows are accurate against their target documents.

---

## Findings

### NIT-1: Hardcoded test counts in comment and step names will silently stale

- **Severity:** NIT
- **Category:** maintainability
- **Location:** `.github/workflows/ci.yml:466-467, 477, 479`

**Description:** The job block comment names specific suite sizes (`bin/test_validate_citations.py (22 tests)` and `bin/test_changelog_gate_content.py (10 tests)`), and both `step: name:` fields repeat those counts. These are purely informational labels — CI correctness is unaffected when counts are wrong — but they will silently become stale the next time either suite grows. Future maintainers reading CI logs or the comment will see incorrect counts with no CI signal to prompt a fix.

**Evidence:** The reference structural pattern (`green-doc-tense-gate`, lines 459-462) uses count-free step names:
```yaml
      - name: Self-test the gate script
        run: python3 bin/test_check_green_doc_tense.py
      - name: Scan for stale RED-phase comment headers in test files
        run: python3 bin/check-green-doc-tense
```
No parenthetical count appears anywhere in that job.

The new job at lines 477 and 479 reads:
```yaml
      - name: Run bin/test_validate_citations.py (22 tests)
        run: python3 bin/test_validate_citations.py
      - name: Run bin/test_changelog_gate_content.py (10 tests)
        run: python3 bin/test_changelog_gate_content.py
```

**Proposed fix:** Remove the parenthetical counts from both step `name:` fields and from lines 465-466 of the comment. If counts are considered useful for human scanning, replace with a comment that they must be kept in sync, or derive the count dynamically (though that adds complexity not warranted here). Simplest fix: drop counts and let the script name speak for itself.

---

## Observations (not findings, no disposition required)

**OBS-1: ci.yml Project References row describes only 4 of 13 CI jobs (pre-existing drift)**

`CLAUDE.md:245` reads `CI pipeline (test, clippy, fmt, semantic PR)`. This wave adds job 13 (`bin-selftest`), but the row was already stale — jobs `fuzz-build`, `audit`, `deny`, `trust-boundary`, `help-provenance-gate`, `action-pin-gate`, `green-doc-tense-gate`, and `changelog-gate` have been added since that description was written and are not listed. The description was never a comprehensive enumeration; it names only the original Rust-quality core. This wave's ci.yml touch was a natural opportunity to scope the description explicitly (e.g., "CI pipeline — core Rust quality gates + bin/ governance jobs"), but omitting that update does not introduce a new problem. Recorded here for the next maintainer to resolve in a housekeeping pass.

**OBS-2: bin-selftest uses a different self-test pattern than green-doc-tense-gate**

`green-doc-tense-gate` embeds its self-test as step 1 within the gate job itself ("self-test before the gate runs, so a broken gate script surfaces immediately"). `bin-selftest` is an independent parallel job running standalone test suites. The two patterns are not inconsistent — the inline-self-test pattern is appropriate when a test file guards the gate's own correctness; the standalone-job pattern is appropriate when test files exercise scripts that have no corresponding CI scan step (the case here: `validate-citations` and `changelog-gate-check` have no standalone scan jobs that need an inline guard). Both patterns are intentional and correct for their respective use cases.

---

## CI Wiring Verification

The `bin-selftest` job is structurally sound:

| Check | Result |
|-------|--------|
| `runs-on: ubuntu-latest` | Consistent with all other gate jobs |
| `timeout-minutes: 5` | Matches `trust-boundary`, `help-provenance-gate`, `action-pin-gate`, `green-doc-tense-gate`, `changelog-gate` |
| `permissions: contents: read` | Minimal; matches `green-doc-tense-gate` and `changelog-gate` patterns |
| Checkout SHA `9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7.0.0` | 40-char SHA; matches every other job in the file exactly; passes `action-pin-gate` |
| No `continue-on-error:` | Correct — these are deterministic internal tests, not external advisory scans |
| No `if:` conditional | Correct — tests must pass on all branches and push events |
| No `needs:` dependency | Correct — independent parallel gate, same pattern as all other gate jobs |
| No `shell: bash` override | Correct — `shell: bash` is used only for multi-line scripts with `set -euo pipefail`; plain `python3 script.py` uses the default shell appropriately |
| Job ID `bin-selftest` | Lowercase, hyphen-separated; consistent with `trust-boundary`, `action-pin-gate`, `green-doc-tense-gate`, `changelog-gate` |
| Interaction risk with 12 other jobs | None — no shared mutable state, no Rust cache dependency, no toolchain install |
| action-pin-gate compliance | Passes: the sole `uses:` line is a 40-char SHA, not a mutable tag |

CLAUDE.md row accuracy:

| Row | Target file | Description accuracy |
|-----|-------------|----------------------|
| `pr-description-row-verify-mandate.md` | File exists at `.factory/maintenance/pr-description-row-verify-mandate.md`; verified contents | "row-verify ≥3 per-test entries" matches Mandate §1 ("at least three randomly-selected entries"); "cross-check claimed counts against actual CI output" matches Mandate §2. Accurate. |
| `delivery-doc-currency-protocol.md` | File exists at `.factory/maintenance/delivery-doc-currency-protocol.md`; verified contents | "status loci, tense audit, demo-evidence currency notes" correctly enumerates Steps 1, 2, 3 of the protocol; "before first adversarial pass of the wave gate" matches the Scope Trigger. Accurate. |

Row ordering: both new rows appended at the bottom of the maintenance-docs group, consistent with the chronological-addition ordering of prior rows (PG-W70, DF-MERGE-AUTH, PG-RA, PG-W72 precede PG-W74 entries). Consistent.

---

## ROUTE-W74-DEFERRED Carry-Forward

Wave-74 code review (`.factory/cycles/wave-74/wave-gate/code-review.md`) recorded the following deferred findings against `bin/` scripts. Wave-75 did **not** touch `bin/`; these items are not re-filed here and carry forward to the next bin-touch PR.

| ID (wave-74) | Severity | File | Description | Disposition |
|--------------|----------|------|-------------|-------------|
| MINOR-1 | MINOR | `bin/test_validate_citations.py` | `_run()` helper is dead code with a design mismatch (separate temp dirs for citations file vs. WIRERUST_REPO_ROOT) | Deferred — batch with next housekeeping pass |
| MINOR-2 | MINOR | `bin/validate-citations` | `parse_line()` docstring omits the regex-mismatch `None` return path | Accept-deferred — one-line docstring fix; batch with next housekeeping pass |
| NIT-1 | NIT | `bin/test_validate_citations.py` | `os`, `stat`, `tempfile` imported inline in test bodies instead of at module top | Accept-deferred — cosmetic; batch with next housekeeping pass |
| NIT-2 (accepted) | NIT | `bin/changelog-gate-check` | `^+##` filter allows bare `+#` top-level heading lines through — accepted by design in story-level review | No action |
| NIT-3 (accepted) | NIT | `bin/validate-citations` | `n_valid` name slightly misleading — accepted by design in story-level review | No action |
| NIT-4 | NIT | `bin/test_validate_citations.py` | Unnecessary f-string in T21 (no interpolation placeholders) | Accept-deferred — cosmetic; batch with next housekeeping pass |

---

## Finding Disposition Table

*(Per AC-158-006 / PG-W71-CODEREVIEW-ARTIFACT: disposition cells marked PROPOSED pending orchestrator/human ratification.)*

| ID | Severity | File | Description | Proposed Disposition |
|----|----------|------|-------------|----------------------|
| NIT-1 | NIT | `.github/workflows/ci.yml:466-467, 477, 479` | Hardcoded test counts `(22 tests)` / `(10 tests)` in comment and step names will silently stale when suites grow | DEFERRED (human-ratified 2026-07-13) — cosmetic only; batch with next bin-touch housekeeping pass; joins ROUTE-W74-DEFERRED |

---

## Summary

No blocking findings. Wave-75 (STORY-165) delivers a lean, correctly-structured CI wiring job (`bin-selftest`) and two accurate CLAUDE.md governance reference rows. The one NIT (hardcoded test counts in step names and comment) is a maintainability concern with no runtime impact. Gate can close.

**Gate status: CLOSED — PASS**
