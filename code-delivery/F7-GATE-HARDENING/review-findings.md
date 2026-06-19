# Review Findings — ci/f7-gate-hardening (help-provenance-gate hardening)

**PR:** #275 — https://github.com/Zious11/wirerust/pull/275
**Branch:** ci/f7-gate-hardening
**Base:** develop
**Delivery type:** Fix PR — closes SEC-001/SEC-002 from PR #274 security review

---

## Source Findings (from PR #274 security review)

| ID | Severity | CWE | Description | Status |
|----|----------|-----|-------------|--------|
| SEC-001 | LOW | CWE-390 | `grep … \|\| true` silently passes on missing `src/cli.rs` (file-not-found swallowed) | FIXED |
| SEC-002 | LOW | CWE-697 | Broad regex matches standards-body IDs (RFC, ISO, CVE, IEC, ANSI, NIST, IEEE) — potential false-positive | FIXED |

---

## Fix Applied

- **SEC-001:** Added `test -f src/cli.rs` guard before grep. If file is absent, gate exits 1 with a clear human-readable message directing maintainers to update the gate scope.
- **SEC-002:** Added `grep -vE '\b(RFC|ISO|CVE|IEC|ANSI|NIST|IEEE)-'` second-pass filter. Strips standards-body matches before pass/fail decision. Factory IDs (BC, STORY, LESSON, VP, ADR, EC, AC, TD, PG) are disjoint from this exclusion list.

---

## Convergence Tracking

| Cycle | Agent | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|-------|----------|----------|-------|-----------|---------|
| 0 (source) | PR #274 security-reviewer | 2 | 0 | — | 2 | REQUEST_CHANGES |
| 1 | security-reviewer (this PR) | TBD | TBD | TBD | TBD | TBD |
| 1 | pr-reviewer (this PR) | TBD | TBD | TBD | TBD | TBD |

---

## CI Status (Run 27847850421)

| Job | Status | Duration |
|-----|--------|----------|
| Semantic PR | PASS | 3s |
| Test | PASS | 45s |
| Clippy | PASS | 24s |
| Format | PASS | 8s |
| Fuzz build | PASS | 1m11s |
| Audit | PASS | 15s |
| Deny | PASS | 24s |
| Trust-boundary (test-seam gate) | PASS | 7s |
| **Help-provenance gate** | **PASS** | **5s** |
| Action pin gate | PASS | 6s |

All 10 checks PASS. CI green. Help-provenance-gate PASS confirmed.

---

_Last updated: 2026-06-19 — Step 6 complete (CI all green); awaiting review verdicts (Step 4/5)_
