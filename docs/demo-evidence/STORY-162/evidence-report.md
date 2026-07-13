# Evidence Report — STORY-162

**Story:** STORY-162: Wave-72 cycle-closing: LMR-003 template-conformance exemption + check-green-doc-tense main() guard self-tests  
**Wave:** 73  
**Date:** 2026-07-10  
**Branch:** feature/STORY-162-cgdt-guard-tests

---

## Coverage Map

| AC | Description | Evidence File | Verdict |
|----|-------------|---------------|---------|
| AC-162-001 | VP-INDEX LMR-003 amendment (definition + allowlist extension + VP-024 v2.5 precedent) | `AC-001-002-vp-index-lmr003-amendment.md` | PASS |
| AC-162-002 | VP-INDEX version bumped to `"2.40"` | `AC-001-002-vp-index-lmr003-amendment.md` | PASS |
| AC-162-003 | Zero-file guard exits exactly 1; hermetic test labeled F-W72G-P2-OBS-001 | `AC-003-zero-file-exit-code-precision.md` | PASS |
| AC-162-004 | `.factory/` OR-sentinel tested hermetcially in four labeled PASS cases | `AC-004-factory-or-sentinel-hermetic.md` | PASS |
| AC-162-005 | PR title uses `docs:` semantic prefix | `AC-005-pr-title-docs-prefix.md` | AT PR TIME |

---

## Evidence Transcripts

### AC-162-001/002 — VP-INDEX grep + version check

```
$ grep "^version:" .factory/specs/verification-properties/VP-INDEX.md
version: "2.40"

$ grep -n "template-conformance\|inputs:\|input-hash:" \
    .factory/specs/verification-properties/VP-INDEX.md
8:modified: "2026-07-10: [v2.40] LMR-003 template-conformance provenance fields amendment
   (STORY-162 AC-162-001/002, F-S161P1-001) ..."
[... allowlist rows and definition paragraph present — see AC-001-002 transcript ...]
```

### AC-162-003 — Test suite (60/60 pass)

```
$ python3 bin/test_check_green_doc_tense.py
...
=== AC-162-003 zero-file guard exit-code precision hermetic (F-W72G-P2-OBS-001) ===
  PASS  [zero-file guard hermetic: main() used _find_repo_root result and exited 1 exactly (AC-162-003, F-W72G-P2-OBS-001)]

Results: 60 passed, 0 failed.
```

Success-path demo (tool run from temp dir):
```
$ cd "$(mktemp -d)" && python3 <repo>/.worktrees/STORY-162/bin/check-green-doc-tense; echo "exit=$?"
PASS: no stale RED-phase comment headers found (110 files scanned).
exit=0
```

### AC-162-004 — Four sentinel PASS lines

```
$ python3 bin/test_check_green_doc_tense.py
...
=== AC-162-004 _find_repo_root sentinel hermetic tests (F-W72G-P2-OBS-001) ===
  PASS  [_find_repo_root: .factory/ OR-sentinel resolves root (F-W72G-P2-OBS-001)]
  PASS  [_find_repo_root: .git directory sentinel resolves root (F-W72G-P2-OBS-001)]
  PASS  [_find_repo_root: .git file (worktree) sentinel resolves root (F-W72G-P2-OBS-001)]
  PASS  [_find_repo_root: no-sentinel temp tree returns None or ancestor (F-W72G-P2-OBS-001)]

Results: 60 passed, 0 failed.
```

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths; no
occurrences of absolute user home paths were present in the committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-10).

---

## Recording Method

This is a governance + Python-tooling story (no UI, no Rust binary changes). Evidence
is captured as CLI transcript markdown files (command + full output). VHS recordings
are not applicable — the product deliverables are documentation (VP-INDEX.md) and
Python test additions, not interactive CLI tools.

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-002-vp-index-lmr003-amendment.md` | AC-162-001, AC-162-002 |
| `AC-003-zero-file-exit-code-precision.md` | AC-162-003 |
| `AC-004-factory-or-sentinel-hermetic.md` | AC-162-004 |
| `AC-005-pr-title-docs-prefix.md` | AC-162-005 |
| `evidence-report.md` | Index (this file) |
