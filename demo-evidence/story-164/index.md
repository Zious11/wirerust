# STORY-164 Demo Evidence Index

**Story:** STORY-164 v1.10 — Wave-73 cycle-closing: status-vocabulary legend, citation
preflight validator, changelog-gate content assertion, guidance-doc reference row,
BREAKING-change holdout-sweep obligation  
**Date:** 2026-07-11  
**Method:** Terminal captures (CLI/governance-tooling story; no VHS required per house
precedent for bin/ tooling stories)  
**Scrub gate:** PG-W70-DEMO-SCRUB — PASS (see below)

---

## Evidence Files

| File | AC | Summary |
|------|----|---------|
| [AC-164-001.md](AC-164-001.md) | AC-164-001 | STORY-INDEX status-vocabulary legend: synonym note, loci agreement rule _(delivery: six statuses at lines 124–145; current: seven statuses at lines 128–150 — superseded added v3.48, sed range shifted; see AC-164-001.md currency note)_ |
| [AC-164-002.md](AC-164-002.md) | AC-164-002 | `bin/validate-citations` live runs: PASS run (4 real anchors), all 8 failure classes demonstrated (FILE NOT FOUND, LINE OUT OF RANGE, INVALID RANGE, INVALID LINE, MALFORMED, OUTSIDE REPO, NOT A FILE, UNREADABLE), exit codes 0/1/2 verified, self-test T01–T22 all green |
| [AC-164-003.md](AC-164-003.md) | AC-164-003 | `bin/changelog-gate-check` behavioral demos: real content → PASS exit 0, whitespace-only → FAIL exit 1, header-only → FAIL exit 1; ci.yml delegation line at line 509 |
| [AC-164-004.md](AC-164-004.md) | AC-164-004 | CLAUDE.md Project References row for `docs-writer-dispatch-guidance.md` at line 249 |
| [AC-164-005.md](AC-164-005.md) | AC-164-005 | CLAUDE.md row for `breaking-change-delivery-protocol.md` at line 250; protocol document head-30 and verification grep |

---

## Per-AC Status Summary

| AC | Verification | Status |
|----|-------------|--------|
| AC-164-001 | `sed -n '124,145p'` shows legend with synonym note and loci agreement rule _(delivery: six statuses; current: seven — superseded added v3.48; sed range now 128–150; see AC-164-001.md)_ | PASS |
| AC-164-002 | 8 failure classes + exit codes 0/1/2 + self-test T01–T22 all green | PASS |
| AC-164-003 | 3 diff scenarios (PASS/FAIL/FAIL) + ci.yml delegation line confirmed | PASS |
| AC-164-004 | grep confirms row at CLAUDE.md line 249 | PASS |
| AC-164-005 | grep confirms row at CLAUDE.md line 250; protocol doc content verified | PASS |

---

## Scrub Gate (PG-W70-DEMO-SCRUB)

Gate command run against this directory per the gate document:
`grep -rE` with patterns for absolute home paths.

Result: **zero results from evidence content — PASS**

All absolute host paths have been replaced with `$REPO_ROOT` or relative forms.
No absolute home-directory paths appear as actual path references in any evidence file.
The only matches from the gate grep are this metadata description itself, which is
expected (the gate pattern appears in the gate description text — not as a real path).
