# [STORY-164] Citation preflight validator + changelog-gate content assertion

**Epic:** E-11 — Tooling and Self-Improvement
**Mode:** maintenance (wave-74, E-11 governance-only)
**Convergence:** CONVERGED after 8 adversarial passes (streak P6/P7/P8 = PASS_NITPICK_ONLY x3)

![Tests](https://img.shields.io/badge/tests-32%2F32-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-N%2FA%20(Python%20tooling)-blue)
![Mutation](https://img.shields.io/badge/mutation-N%2FA%20(governance%20story)-blue)
![Holdout](https://img.shields.io/badge/holdout-N%2FA%20(E--11%20governance)-blue)

This PR codifies four wave-73 process gaps as durable tooling artifacts. (1) `bin/validate-citations`
(Python 3, stdlib-only, 22 self-tests) mechanically validates `file:line-range` anchor citations
before dispatch — closing the fabricated-citation class surfaced at CRITICAL severity by finding
F-S163P1-001. (2) `bin/changelog-gate-check` is extracted from ci.yml and gains a content assertion
that rejects whitespace-only or header-only CHANGELOG edits, closing the presence-only gap noted in
PG-W73-CHANGELOG-GATE-CONTENT. (3) CLAUDE.md Project References table gains two rows:
`docs-writer-dispatch-guidance.md` (peer of the existing `pr-manager-merge-auth-guidance.md` row)
and `breaking-change-delivery-protocol.md` (new maintenance artifact). Factory-side deliverables
(STORY-INDEX status-vocabulary legend, breaking-change holdout-sweep protocol) are committed on the
`factory-artifacts` branch and are NOT part of this develop PR diff.

---

## Architecture Changes

```mermaid
graph TD
    CIyml[".github/workflows/ci.yml\n(before: inline grep content check)"]
    CGC["bin/changelog-gate-check\n(new: extracted bash helper)"]
    VC["bin/validate-citations\n(new: Python 3 citation preflight)"]
    TESTS_VC["bin/test_validate_citations.py\n(22 self-tests, T01–T22)"]
    TESTS_CGC["bin/test_changelog_gate_content.py\n(10 self-tests, B01–B10)"]
    CLAUDE["CLAUDE.md\n(+2 Project References rows)"]

    CITML -->|"delegates to"| CGC
    TESTS_CGC -->|"exercises"| CGC
    TESTS_VC -->|"exercises"| VC

    style CGC fill:#90EE90
    style VC fill:#90EE90
    style TESTS_VC fill:#90EE90
    style TESTS_CGC fill:#90EE90
    style CLAUDE fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: Extract bin/changelog-gate-check + add bin/validate-citations (AC-164-002/003)

**Context:** The changelog-gate CI job checked only for CHANGELOG.md presence in the diff (a
whitespace-only touch satisfied it). There was no tool to validate `file:line-range` anchor
citations before adversarial review; F-S163P1-001 caught fabricated citations at CRITICAL
severity only after dispatch.

**Decision:** Extract the changelog-gate content check into a standalone bash helper
`bin/changelog-gate-check` (testable independently of CI). Add a new Python 3 tool
`bin/validate-citations` following the `bin/compute-input-hash` structural pattern (stdlib only,
Python 3.10+ type syntax, self-test suite in the companion test file).

**Rationale:** Extraction of `changelog-gate-check` mirrors the existing `bin/` tooling pattern
and enables the 10-test suite (B01–B10) to exercise content vs. presence behavior hermetically.
A Python tool for citation validation allows structured output, 8 discrete failure classes, and
three exit codes (0=PASS, 1=citation errors, 2=input/config errors) without external dependencies.

**Alternatives Considered:**
1. Inline the content assertion directly in ci.yml — rejected because it cannot be unit-tested
   without a real PR diff.
2. Shell script for citation validation — rejected because parsing file:line-range with reliable
   error classes is error-prone in bash; Python 3.10+ pathlib gives idiomatic path containment.

**Consequences:**
- `bin/changelog-gate-check` is now independently testable and exercised by CI via this PR's
  own diff (the content assertion verifies itself).
- `bin/validate-citations` closes the fabricated-citation class mechanically with CWE-22
  containment via `.resolve()` + `.is_relative_to()`.
- Two LOW security findings (SEC-001 WIRERUST_REPO_ROOT bypass, SEC-002 TOCTOU) are
  accepted-by-design for a local dev tool; both documented in `security-review.md`.

</details>

---

## Story Dependencies

```mermaid
graph LR
    STORY164["STORY-164\n🟡 this PR"]

    STORY164 --> NONE["(no downstream blockers\nin depends_on)"]

    style STORY164 fill:#FFD700
    style NONE fill:#E0E0E0
```

STORY-164 has `depends_on: []` — no upstream PRs must merge before this one.
No downstream stories are currently blocked on this PR.

---

## Spec Traceability

```mermaid
flowchart LR
    PG1["PG-W73-CITATION-VALIDATOR\n(F-S163P1-001 CRITICAL)"]
    PG2["PG-W73-CHANGELOG-GATE-CONTENT\n(carried from STORY-162 P5)"]
    PG3["Wave-73 consistency audit\n(docs-writer guidance missing from CLAUDE.md)"]
    PG4["PG-W73-STATUS-VOCAB\n(F-W73G-P3-001)"]

    AC002["AC-164-002\nbin/validate-citations"]
    AC003["AC-164-003\nbin/changelog-gate-check + ci.yml assertion"]
    AC004["AC-164-004\nCLAUDE.md docs-writer row"]
    AC005["AC-164-005\nCLAUDE.md breaking-change row"]
    AC001["AC-164-001\nSTORY-INDEX legend"]

    T_VC["T01–T22: validate-citations\n22 self-tests"]
    T_CGC["B01–B10: changelog-gate-check\n10 self-tests"]
    S_VC["bin/validate-citations"]
    S_CGC["bin/changelog-gate-check"]
    S_CI[".github/workflows/ci.yml"]
    S_CLAUDE["CLAUDE.md (2 rows)"]
    FA["factory-artifacts branch\nSTORY-INDEX legend\nbreaking-change-protocol"]

    PG1 --> AC002
    PG2 --> AC003
    PG3 --> AC004
    PG3 --> AC005
    PG4 --> AC001

    AC002 --> T_VC --> S_VC
    AC003 --> T_CGC --> S_CGC
    AC003 --> S_CI
    AC004 --> S_CLAUDE
    AC005 --> S_CLAUDE
    AC001 --> FA
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Unit tests (validate-citations) | 22/22 pass | 100% | PASS |
| Unit tests (changelog-gate-check) | 10/10 pass | 100% | PASS |
| Coverage | N/A (Python tooling; no Rust source changed) | N/A | N/A |
| Mutation kill rate | N/A (governance story) | N/A | N/A |
| Holdout satisfaction | N/A (E-11 governance-only) | N/A | N/A |

### Test Flow

```mermaid
graph LR
    VC["22 Tests\n(bin/test_validate_citations.py)"]
    CGC["10 Tests\n(bin/test_changelog_gate_content.py)"]

    VC --> Pass1["PASS (22/22)"]
    CGC --> Pass2["PASS (10/10)"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 32 added (22 validate-citations T01–T22, 10 changelog-gate B01–B10), 0 modified |
| **Total suite** | cargo test --all-targets: full suite green |
| **Coverage delta** | N/A — Python tooling, no Rust source |
| **Mutation kill rate** | N/A — governance story |
| **Regressions** | 0 |

<details>
<summary><strong>Detailed Test Results</strong></summary>

### validate-citations (T01–T22)

| Test | Failure class | Result |
|------|--------------|--------|
| T01 missing citations table | input error | PASS |
| T02 no anchors | PASS path | PASS |
| T03 valid single anchor | PASS path | PASS |
| T04 valid multi-anchor | PASS path | PASS |
| T05 file not found | FILE NOT FOUND | PASS |
| T06 line out of range | LINE OUT OF RANGE | PASS |
| T07 invalid range (start > end) | INVALID RANGE | PASS |
| T08 invalid line (0 or negative) | INVALID LINE | PASS |
| T09 malformed anchor | MALFORMED | PASS |
| T10 outside repo | OUTSIDE REPO | PASS |
| T11 not a file (directory) | NOT A FILE | PASS |
| T12 unreadable target | UNREADABLE | PASS |
| T13–T22 | edge cases / exit codes 0/1/2 | PASS (10/10) |

### changelog-gate-check (B01–B10)

| Test | Scenario | Result |
|------|----------|--------|
| B01 real Unreleased content | PASS exit 0 | PASS |
| B02 whitespace-only addition | FAIL exit 1 | PASS |
| B03 header-only addition | FAIL exit 1 | PASS |
| B04–B10 | edge cases + direct invocation | PASS (7/7) |

</details>

---

## Demo Evidence

Evidence files committed on the feature branch at `.factory/demo-evidence/story-164/`.
Demo-evidence path-scrub gate (PG-W70-DEMO-SCRUB) passed — zero absolute host paths in
any evidence file (verified 2026-07-11).

| AC | Evidence File | Verdict |
|----|---------------|---------|
| AC-164-001 | `AC-164-001.md` — STORY-INDEX legend at lines 124–145 | PASS (factory-artifacts) |
| AC-164-002 | `AC-164-002.md` — 8 failure classes + exit codes 0/1/2 + T01–T22 green | PASS |
| AC-164-003 | `AC-164-003.md` — 3 diff scenarios (PASS/FAIL/FAIL) + ci.yml delegation at line 509 | PASS |
| AC-164-004 | `AC-164-004.md` — CLAUDE.md row at line 249 | PASS |
| AC-164-005 | `AC-164-005.md` — CLAUDE.md row at line 250; protocol doc verified | PASS |

**Recording method:** CLI transcript markdown files. VHS not applicable — deliverables are
Python/bash tooling and documentation, not interactive terminal UI (house precedent for bin/ stories).

---

## Holdout Evaluation

N/A — E-11 governance-only story; no behavioral contracts authored; no Rust source changed;
holdout evaluation not applicable per E-11 convention.

---

## Adversarial Review

| Pass | Findings | Critical | High | Status |
|------|----------|----------|------|--------|
| P1 | 6 | 0 | 0 | Fixed (F-S164P1-001/002/004/005/006) |
| P2 | 4 | 0 | 0 | Fixed (F-S164P2-001/002/003/004) |
| P3 | 3 | 0 | 0 | Fixed (F-S164P3-001/002/003) |
| P4 | 2n | 0 | 0 | Nits fixed |
| P5 | 2 | 0 | 0 | Fixed (F-S164P5-002 CHANGELOG parity) |
| P6 | 1n | 0 | 0 | PASS_NITPICK_ONLY (streak begins) |
| P7 | 2n | 0 | 0 | PASS_NITPICK_ONLY |
| P8 | 1n | 0 | 0 | PASS_NITPICK_ONLY (streak 3/3) |

**Convergence:** CONVERGED after 8 passes. Streak P6/P7/P8 = PASS_NITPICK_ONLY x3.
Finding trajectory: 6→4→3→2n→2→1n→2n→1n. Zero open HIGH or CRITICAL findings.
DF-CONVERGENCE-BEFORE-MERGE-001 satisfied.

<details>
<summary><strong>Accepted-by-Design Dispositions</strong></summary>

Two items accepted-by-design, recorded in STORY-164 v1.10 Notes:

- **stdin non-UTF-8 → exit 2 (not 1):** Intentional distinction between "bad input format"
  (exit 2) and "citation validation errors" (exit 1). The exit-code contract is documented in
  the tool's ALGORITHM docstring.
- **T20 docstring label "unreadable target":** Matches the test name and failure class verbatim.
  The label is correct and needs no change.

</details>

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0"]
    Medium["Medium: 0"]
    Low["Low: 2"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#87CEEB
```

<details>
<summary><strong>Security Scan Details</strong></summary>

### Findings (from security-review.md)

| ID | Severity | CWE | Finding | Disposition |
|----|----------|-----|---------|-------------|
| SEC-001 | LOW | CWE-610 | `WIRERUST_REPO_ROOT=/` bypasses `is_relative_to()` containment | Accepted / design-intentional for test isolation |
| SEC-002 | LOW | CWE-367 | TOCTOU between `is_file()` check and `count_lines()` open | Accepted / very low exploitability for local dev tool |
| SEC-003 | INFO | CWE-22 | `compute-input-hash` path traversal gap (pre-existing, deferred) | Accepted / tracked in GitHub #392 |
| SEC-004 | INFO | CWE-829 | `bin/changelog-gate-check` mutable via PRs (same as all bin/ scripts) | Accepted / consistent pattern |

### Positive Findings

- CWE-22 containment via `.resolve()` + `.is_relative_to()` is correct and handles absolute
  paths, `../` traversal, and symlink chasing.
- Non-UTF-8 input hardened on both stdin and file-argument paths.
- Bash `|| true` pipefail guard correct; all variable expansions double-quoted.
- ci.yml change adds no new `uses:` actions; SHA-pin policy unaffected.

### Dependency Audit

- No Rust source modified — `cargo audit` surface unchanged.
- Python scripts use stdlib only (no third-party deps introduced).

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** `bin/validate-citations` and `bin/changelog-gate-check` (Python/bash tooling, not in Rust binary); `ci.yml` changelog-gate step; `CLAUDE.md` reference table
- **User impact:** None if failure occurs — tooling scripts only, not production code
- **Data impact:** None — no data storage, no persistent state changed by this PR
- **Risk Level:** LOW

### Performance Impact

| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| `changelog-gate` CI step | presence-only grep | extracted helper + content check | ~0ms | OK |
| Rust binary | unchanged | unchanged | 0 | OK |
| Test suite (Rust) | green | green | 0 | OK |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert <MERGE_COMMIT_SHA>
git push origin develop
```

This PR makes no schema changes, no behavioral changes to production Rust code, and the
ci.yml change is additive (content assertion on top of the existing presence check).
Rollback is trivially safe.

**Verification after rollback:**
- `python3 bin/test_validate_citations.py` — should report 0 tests (tool no longer exists)
- `python3 bin/test_changelog_gate_content.py` — should report 0 tests
- `cargo test --all-targets` — should remain green

</details>

### Feature Flags

| Flag | Controls | Default |
|------|----------|---------|
| (none) | N/A | N/A |

---

## Companion Governance Changes (Not in This PR)

The following deliverables are committed on the `factory-artifacts` branch and are NOT part of
this develop PR diff:

- **AC-164-001:** STORY-INDEX status-vocabulary legend — six statuses with precise semantics,
  synonym note (delivered/merged/completed equivalence), loci agreement rule; placed after
  `## Index Table` heading before the table itself.
- **AC-164-005 companion:** `breaking-change-delivery-protocol.md` in `.factory/maintenance/`
  codifying the BREAKING-change holdout-sweep obligation identified in wave-73.

---

## Traceability

| Process Gap | Story AC | Test / Artifact | Status |
|-------------|---------|-----------------|--------|
| PG-W73-CITATION-VALIDATOR (F-S163P1-001 CRITICAL) | AC-164-002 | T01–T22 (22/22) | PASS |
| PG-W73-CHANGELOG-GATE-CONTENT | AC-164-003 | B01–B10 (10/10) + ci.yml | PASS |
| Wave-73 docs-writer guidance discoverability | AC-164-004 | CLAUDE.md line 249 | PASS |
| Breaking-change holdout-sweep obligation | AC-164-005 | CLAUDE.md line 250 | PASS |
| PG-W73-STATUS-VOCAB (F-W73G-P3-001) | AC-164-001 | factory-artifacts STORY-INDEX legend | PASS (companion) |
| AC-158-001 (changelog gate) | — | CHANGELOG.md `[Unreleased]` entry | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
F-S163P1-001 CRITICAL -> AC-164-002 -> T01–T22 -> bin/validate-citations (CWE-22 contained)
PG-W73-CHANGELOG-GATE-CONTENT -> AC-164-003 -> B01–B10 -> bin/changelog-gate-check + ci.yml:509
Wave-73 consistency audit -> AC-164-004 -> CLAUDE.md line 249 (docs-writer row)
Wave-73 consistency audit -> AC-164-005 -> CLAUDE.md line 250 (breaking-change row)
F-W73G-P3-001 -> AC-164-001 -> STORY-INDEX legend -> factory-artifacts branch
AC-158-001 -> CHANGELOG.md [Unreleased] entry -> exercised by ci.yml changelog-gate job
```

</details>

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: maintenance
factory-version: "1.0.0-rc.22"
pipeline-stages:
  spec-crystallization: completed (STORY-164.md v1.10)
  story-decomposition: completed (E-11 governance pattern)
  tdd-implementation: completed
  holdout-evaluation: "N/A — E-11 governance convention"
  adversarial-review: completed (8 passes, CONVERGED)
  formal-verification: "N/A — Python tooling + documentation only"
  convergence: achieved
convergence-metrics:
  adversarial-passes: 8
  streak-classification: "PASS_NITPICK_ONLY x3 (P6/P7/P8)"
  finding-trajectory: "6→4→3→2n→2→1n→2n→1n"
  open-high-critical: 0
  test-pass: "32/32 (22 + 10)"
models-used:
  builder: claude-sonnet-4-6
  adversary: claude-sonnet-4-6
  orchestrator: claude-sonnet-4-6
wave: "74"
story-points: 4
generated-at: "2026-07-11"
```

</details>

---

## Pre-Merge Checklist

- [x] All CI status checks passing (12/12 green)
- [x] No critical/high security findings unresolved (0 across 8 adversarial passes + security review)
- [x] Adversarial convergence satisfied (DF-CONVERGENCE-BEFORE-MERGE-001, streak 3/3)
- [x] Demo evidence present and scrub-gate passed (PG-W70-DEMO-SCRUB)
- [x] CHANGELOG [Unreleased] entry present (AC-158-001 changelog gate)
- [x] No production Rust source modified (E-11 governance-only)
- [x] Rollback procedure validated (trivial revert)
- [x] pr-reviewer APPROVE posted (review comment 4678783752, D-425 self-PR waiver documented)
- [x] security-reviewer PASS (no CRITICAL/HIGH; 2 LOW accepted-by-design)
- [ ] Human merge authorization (DF-MERGE-AUTH-CLASSIFIER-001 / D-425 interim path — orchestrator executes after gate report)
