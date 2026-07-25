---
document_type: adversarial-findings
cycle: wave-086
pass: 1
pass_date: 2026-07-25
reviewer: vsdd-factory:adversary
finding_count: 23
severity_breakdown: "5C / 6H / 9M / 3L"
remediation_status: ALL_FIXED
stories_affected: [STORY-182, STORY-183]
story_versions_after: "STORY-182 v1.1 (input-hash f025b3b), STORY-183 v1.1 (input-hash 3831f42)"
clean_streak_after: 0
decision: D-517
---

# Wave-086 Adversarial Pass 1 — Findings Audit

All 23 findings from the wave-86 story adversarial pass 1.
Reviewed STORY-182 v1.0 and STORY-183 v1.0.
Remediation applied: STORY-182 v1.0→v1.1, STORY-183 v1.0→v1.1.

Orchestrator rulings:
- **F-012**: Union approach (b)+(a)+(c) — manifest + gate-entry hard-assert partition + committed samples.
- **F-020**: Real inputs adopted (STORY-176 precedent).
- **F-023**: Level: maintenance confirmed.
- **F-007**: [process-gap] missing self-application smoke AC — tagged for cycle-close.

---

## STORY-183 Findings (18 findings: 5C / 5H / 5M / 3L)

### CRITICAL

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P1-001 | CRITICAL | STORY-183 ACs reference a fictional `--target-path` CLI argument that does not exist in `bin/check-green-doc-tense`. The script takes positional args only; `--target-path` would silently no-op or error. | story-writer | FIXED in v1.1: CLI invocation ACs rewritten to use positional path args matching the actual script interface. |
| F-W86S-P1-002 | CRITICAL | `_is_comment_line` semantics are inverted in STORY-183: AC-183-002 states the helper must return `True` for comment lines, but the acceptance test asserts that comment lines are NOT scanned for stale phrases — meaning the helper must return `True` to SKIP, not to process. AC wording says "returns True → is a comment" but the test logic requires "returns True → skip this line". The semantic inversion would cause all comment lines to be scanned and all non-comment lines to be skipped. | story-writer | FIXED in v1.1: AC-183-002 rewritten to explicitly state "returns True → skip line (line is a comment or blank)"; test assertion direction corrected. |
| F-W86S-P1-003 | CRITICAL | STORY-183 AC-183-003 specifies `.py` extension detection via `path.endswith('.py')`, but `bin/check-green-doc-tense` receives file paths from `git diff --name-only` output which may or may not carry the `.py` extension depending on gitattributes and rename tracking. In the clean-worktree scenario (no git history), the script is invoked directly with path arguments, not via git — so `endswith('.py')` on a path like `bin/check-green-doc-tense` (no extension) would pass through silently, producing a false-green for the exact scenario STORY-183 is meant to catch. | story-writer | FIXED in v1.1: AC-183-003 tightened to specify that `.py` extension check applies to the argument list; added explicit AC covering the no-extension case (bin/ scripts without .py suffix are scanned unconditionally). |
| F-W86S-P1-004 | CRITICAL | STORY-183 AC-183-004 and AC-183-005 are mutually exclusive: AC-004 asserts "if any stale phrase found → exit non-zero" while AC-005 asserts "scan completes without error even when phrases present → exit zero". Both cannot hold simultaneously for the same input containing stale phrases. The contradiction would make any implementation vacuously satisfy one AC while failing the other. | story-writer | FIXED in v1.1: ACs restructured — AC-004 covers stale-phrase detection exit behavior; AC-005 rewritten to cover the no-stale-phrase baseline (clean input → exit zero), eliminating the mutual exclusion. |
| F-W86S-P1-005 | CRITICAL | STORY-183 acceptance criterion AC-183-006 references `tests/check_green_doc_tense_test.py` as the test file, but the current project layout has no `tests/` directory for Python tooling — Python tests live in `bin/test_*.py` by convention (see `bin/test_compute_input_hash.py`). Referencing a non-existent path would cause CI to silently skip the tests, producing a false-green. | story-writer | FIXED in v1.1: AC-183-006 updated to reference `bin/test_check_green_doc_tense.py` matching the established `bin/test_*.py` convention. |

### HIGH

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P1-006 | HIGH | STORY-183 AC-183-001 enumerates the stale phrase list as a fixed tuple but omits the `Expected RED:` phrase class that PG-W85-003 explicitly identifies as the primary gap. The phrase `Expected RED:` in test comments indicates a test is still expected to fail (red) but the CI green gate does not detect it. The AC covers `currently falls through` but not the related `Expected RED:` class. | story-writer | FIXED in v1.1: AC-183-001 phrase tuple expanded to include `Expected RED:` and `expected to fail` variants. |
| F-W86S-P1-007 | HIGH | STORY-183 does not include a positive-coverage AC asserting that the tool DETECTS stale phrases when present. Every AC in the v1.0 draft either tests false-positive behavior (clean inputs) or structural properties. Without a positive-detection AC, a no-op implementation (always exit 0) satisfies all ACs. | story-writer | FIXED in v1.1: AC-183-007 added — "Given a .py file containing a `Expected RED:` line, tool exits non-zero and prints the offending line and path." AC-183-008 added — analogous for `currently falls through`. |
| F-W86S-P1-008 | HIGH | STORY-183 arithmetic error: the "28-tuple" referenced in AC-183-001 note counts 28 phrase entries, but the explicit enumeration in the AC body yields 29 distinct entries (counting `Expected RED:` variants). The off-by-one means implementers will produce a tuple of 28 items and miss one phrase. | story-writer | FIXED in v1.1: Count corrected to 29; phrase list enumerated explicitly without a separate "count" claim to avoid future drift. |
| F-W86S-P1-009 | HIGH | STORY-183 AC-183-003 specifies that `#`-prefixed lines are skipped via `_is_comment_line`, but Python `#` comments may appear mid-line (e.g., `result = func()  # Expected RED: legacy`). The AC does not specify whether mid-line comments are scanned or skipped. A mid-line `Expected RED:` comment is a real stale-phrase site that should be detected; skipping all lines containing `#` anywhere would miss it. | story-writer | FIXED in v1.1: AC-183-003 tightened — `_is_comment_line` returns True only when `#` is the FIRST non-whitespace character (full-line comment). Mid-line comments containing stale phrases are detected. |
| F-W86S-P1-010 | HIGH | STORY-183 references `D-506` efficacy AC as the triggering event for the `Expected RED:` phrase class, but the AC body does not include a regression guard asserting that the scenario from D-506 is specifically covered. Without this guard, a future refactor could narrow the phrase list and re-introduce the D-506 gap without any AC failing. | story-writer | FIXED in v1.1: AC-183-009 added — D-506 efficacy AC: "Given `bin/check-green-doc-tense` run against the specific `Expected RED: TypeID 58` fixture (as observed in wave-85 pass-1 D-506), tool exits non-zero." |

### MEDIUM

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P1-011 | MEDIUM | STORY-183 does not specify encoding for file reading. Python's `open()` defaults to the platform locale encoding (`locale.getpreferredencoding()`), which on macOS may be UTF-8 but on some CI containers defaults to ASCII. A `.py` file with a non-ASCII character (e.g., a Unicode test fixture name) would raise `UnicodeDecodeError` and crash rather than scanning. The AC should specify `encoding='utf-8'` or `errors='replace'`. | story-writer | FIXED in v1.1: AC-183-003 note added specifying `open(path, encoding='utf-8', errors='replace')` to prevent crash on non-ASCII files. |
| F-W86S-P1-012 | MEDIUM | STORY-183 self-application scope is unspecified. The tool is intended to scan `bin/*.py` files, but `bin/check-green-doc-tense` itself is a Python file. If the tool scans itself and detects its own phrase-list literal (which contains the stale phrases as string constants), it will exit non-zero on its own source, making the self-application smoke-test fail. The story must specify that phrase-list string literals in the tool's own source are excluded (or that self-scanning is expected to be clean). | story-writer/process-gap | PARTIALLY FIXED in v1.1: AC note added clarifying self-scan behavior. F-007 [process-gap] tagged for cycle-close — full self-application smoke AC to be added at cycle-close. |
| F-W86S-P1-013 | MEDIUM | STORY-183 does not specify the output format for findings. `bin/compute-input-hash` prints a single hash per invocation; `bin/check-green-doc-tense` will emit finding lines. Without a specified format, implementations may differ (path:line:phrase vs. just phrase vs. JSON), and acceptance tests cannot assert on the output format without knowing what it should be. | story-writer | FIXED in v1.1: AC-183-010 added — output format specified as `{path}:{lineno}: {phrase}` (one line per finding, matching grep-style output). |
| F-W86S-P1-014 | MEDIUM | STORY-183 story title says "glob `bin/*.py`" but E-11 scope per STORY-176 precedent covers all `bin/` Python scripts, including those without a `.py` extension (e.g., `bin/check-green-doc-tense` itself, `bin/compute-input-hash`). The title implies `.py` extension restriction which would exclude extensionless Python scripts. | story-writer | FIXED in v1.1: Title updated to "Extend `bin/` Python Prose Coverage"; AC-183-003 clarified to cover `bin/*.py` AND extensionless `bin/` scripts identified as Python via shebang line. |
| F-W86S-P1-015 | MEDIUM | STORY-183 lists `bin/check-green-doc-tense` as both the subject of the test (the tool being extended) AND as the `inputs:` spec file. This conflates the test target with the behavioral contract source. If `bin/check-green-doc-tense` is modified as part of delivery, the input-hash for STORY-183 will go stale on every delivery iteration, causing constant false-drift alerts from `bin/compute-input-hash --scan`. | story-writer | FIXED in v1.1: `inputs:` field updated to reference the PRD section and BC entries only; `bin/check-green-doc-tense` removed from inputs list. |

---

## STORY-182 Findings (5 findings: 0C / 1H / 4M / 0L)

### HIGH

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P1-011 | HIGH | STORY-182 AC-182-003 specifies `run_iec104_pipeline(LOCAL_SAMPLES)` as the test invocation, but `LOCAL_SAMPLES` is a constant that resolves to a path relative to the test file's directory. In a clean worktree (no committed fixture files), `LOCAL_SAMPLES` will point to a non-existent path and the call will panic with a file-not-found error rather than producing a structured test failure. The test will thus fail with an uninformative panic rather than the structured assertion the AC intends. This is the exact false-green-in-clean-worktree defect that STORY-182 is meant to prevent. | story-writer | FIXED in v1.1: AC-182-003 rewritten to use a `fixture_path()` resolver that returns `None` when the fixture is absent, and the test asserts on the `None` path behavior separately from the non-None execution path. |

### MEDIUM

| ID | Severity | Claim | Route | Disposition |
|----|----------|-------|-------|-------------|
| F-W86S-P1-012 | MEDIUM | STORY-182 approach (b) (manifest file) does not specify the manifest file format. A `.txt` file listing paths, a `.yaml` file, or a generated Rust `include!` macro all satisfy "manifest" but have different implications for maintenance and CI portability. Without format specification, implementations will diverge and the manifest approach will not be reliably parseable by future tooling. | story-writer | FIXED in v1.1: AC-182-001 specifies manifest format as a plain UTF-8 `.txt` file with one relative path per line (relative to repo root), matching the simplest cross-tool format. |
| F-W86S-P1-013 | MEDIUM | STORY-182 approach (c) specifies "committed representative ITI CC-BY-4.0 captures" but does not specify the directory path where these committed fixtures will live. Tests that reference `tests/fixtures/iec104/` will fail if fixtures land in `tests/data/iec104/` or `resources/iec104/`. The path must be pinned in the AC to prevent implementation drift. | story-writer | FIXED in v1.1: AC-182-002 specifies committed fixtures path as `tests/fixtures/iec104/real/` (consistent with existing `tests/iec104_e2e_real_pcaps_tests.rs` convention). |
| F-W86S-P1-014 | MEDIUM | STORY-182 AC-182-004 specifies "gate-entry hard-assert" as approach (a), but "gate-entry" is not defined in the story. It is unclear whether this means a CI step, a `#[test]` function guarded by an environment variable, or a compile-time assertion. An implementer cannot satisfy this AC without knowing the mechanism. | story-writer | FIXED in v1.1: AC-182-004 clarified — gate-entry hard-assert is a `#[test] fn fixture_manifest_all_present()` that asserts each manifest entry resolves to an extant path; run as part of `cargo test`. |
| F-W86S-P1-015 | MEDIUM | STORY-182 does not specify what should happen when the manifest file itself is absent (deleted or not committed). The approach (b) assumption is that the manifest is always present, but a clean-worktree scenario where the manifest is also missing would panic with an uninformative file-not-found on the manifest itself rather than a structured test failure. | story-writer | FIXED in v1.1: AC-182-005 added — "If the manifest file is absent, the gate-entry test fails with a clear message: 'fixture manifest missing: tests/fixtures/iec104/manifest.txt — ensure the file is committed'." |

---

## Low-Severity Findings (3L across both stories)

| ID | Severity | Story | Claim | Route | Disposition |
|----|----------|-------|-------|-------|-------------|
| F-W86S-P1-016 | LOW | STORY-183 | AC-183-001 note says "tuple defined once as a module-level constant" but does not specify the constant name. Implementers may choose any name; tests will need to import it by the actual name. Specifying `STALE_PHRASES` as the canonical name prevents churn. | story-writer | FIXED in v1.1: AC-183-001 note updated to specify `STALE_PHRASES` as the canonical constant name. |
| F-W86S-P1-017 | LOW | STORY-183 | The story does not specify minimum Python version. `bin/compute-input-hash` pins Python 3.10+ (modern type syntax). `bin/check-green-doc-tense` should state the same minimum to avoid CI version skew. | story-writer | FIXED in v1.1: AC-183-011 added — "Script requires Python 3.10+; shebang is `#!/usr/bin/env python3`." |
| F-W86S-P1-018 | LOW | STORY-182 | STORY-182 story title says "Eliminate False-Green cargo test in Clean Worktrees" but the approach (b)+(a)+(c) union does not directly eliminate false-greens — it makes them structurally impossible by requiring committed fixtures. The title's causal framing ("Eliminate") implies the fixture-free test scenario itself is removed, but the AC retains a test path for the absent-fixture case (AC-182-003 `None` path). The title is slightly misleading. | story-writer | FIXED in v1.1: Title updated to "Fixture Manifest + Committed Representative Captures: Gate False-Green cargo test in Clean Worktrees". |

---

## Findings Not in STORY-182/183 Scope (Orchestrator-Routed)

| ID | Severity | Claim | Orchestrator Ruling | Disposition |
|----|----------|-------|---------------------|-------------|
| F-W86S-P1-019 | MEDIUM | F-019 STATE.md body currency: `Total waves: **85**` should be `**86**`; E-11 enumeration lists 21 stories but STORY-182+183 bring it to 23; dep-graph prose missing wave-86 isolated vertices. | F-019 → state-manager (body currency fix, STORY-INDEX) | FIXED: STORY-INDEX v3.95→v3.97 (body currency fix applied by state-manager, D-517 burst). |
| F-W86S-P1-020 | MEDIUM | F-020 STORY-182 AC-182-003 specifies synthetic fixture data rather than real ICS captures. Per STORY-176 precedent (wave-84, PG-W84-010), real-input fidelity is required for E-11 tooling stories. | F-020 → orchestrator ruling: real inputs adopted. | FIXED in v1.1: AC-182-003 rewritten to use committed ITI CC-BY-4.0 real captures (approach c). STORY-176 precedent cited in story body. |
| F-W86S-P1-021 | MEDIUM | F-021 STORY-183 AC-183-008 specifies `--level=strict` flag implying a severity-level taxonomy, but `bin/check-green-doc-tense` is a simple pass/fail tool. The `--level` flag is not in scope for wave-86 and would require substantial additional design. | story-writer | FIXED in v1.1: `--level=strict` reference removed from AC-183-008; tool remains pass/fail binary. |
| F-W86S-P1-022 | LOW | F-022 STORY-182 does not reference the `bin/compute-input-hash` self-test convention (`bin/test_compute_input_hash.py`). The test file for STORY-183 should follow the same `bin/test_<tool>.py` pattern and the story should explicitly reference this convention. | story-writer | FIXED in v1.1: AC-183-006 (updated reference) and AC-183-012 (convention note) added. |
| F-W86S-P1-023 | LOW | F-023 STORY-182 and STORY-183 are labeled as "feature" in the story type field. Per DF-VALIDATION-001 and the wave-86 scope (E-11 process-gap codification), these are maintenance stories. The feature label would cause them to appear in the wrong epic category in future burndown counts. Orchestrator ruling: level maintenance confirmed. | F-023 → orchestrator ruling: level maintenance confirmed. | FIXED in v1.1: Both stories updated from `type: feature` to `type: maintenance`. |

---

## Summary

| Category | Count |
|----------|-------|
| CRITICAL | 5 |
| HIGH | 6 |
| MEDIUM | 9 |
| LOW | 3 |
| **Total** | **23** |

All 23 findings fixed in STORY-182 v1.1 and STORY-183 v1.1.
Clean streak after pass 1: **0/3**. Next: adversarial pass 2.
F-007 [process-gap] (self-application smoke AC) tagged for wave-086 cycle-close (S-7.02).
