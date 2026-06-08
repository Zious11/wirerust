---
document_type: story
story_id: STORY-091
epic_id: E-11
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 5
inputs: []
input-hash: "d41d8cd"
# BC status: pending PO authorship — behavioral_contracts is empty; story must remain
# status: draft until PO authors and anchors canonical BC-S.SS.NNN contracts for this story.
traces_to:
  - .factory/STATE.md
points: 5
depends_on: []
blocks: []
behavioral_contracts: []
verification_properties: []
priority: P1
cycle: v0.1.0-greenfield-spec
wave: ~
target_module: bin/validate-anchors
subsystems: []
estimated_days: 2
tdd_mode: strict
nfr:
  - NFR-SEC-006
  - NFR-SEC-007
  - NFR-MNT-005
implementation_strategy: greenfield-tooling
dispositions:
  - PROCESS-GAP-P5-001
---

# STORY-091: Anchor-Validation Tooling — bin/validate-anchors

> **Disposition target:** PROCESS-GAP-P5-001
>
> Phase-5 adversarial refinement surfaced source-line-anchor drift across four
> dimensions (BC source anchors, BC secondary anchors, consuming VP/invariant/
> supplement/entity docs, story bodies) over 11 passes. An exhaustive sweep
> corrected 83 stale citations out of 1305 (Pass 8). Root cause: sweeps were
> reactive — no automated tool detected anchor drift before adversarial passes.
> This story builds the durable preventive tool.

## Narrative

- **As a** spec maintainer or PO running a phase gate
- **I want** a Python CLI tool `bin/validate-anchors` that scans every
  `.factory/specs/**` artifact for `src|tests|fuzz/<path>.rs:NNN[-MMM]`
  citations and verifies each cited line number against the current source tree
- **So that** anchor drift is caught mechanically — at gate entry rather than
  by adversarial-review re-discovery — and the number of anchor-related findings
  per adversarial pass decays toward zero across all spec-corpus dimensions

## Behavioral Contracts

_No BCs authored yet. Status must remain `draft` until PO anchors canonical
BC-S.SS.NNN contracts covering the tool's behavior (exit codes, table format,
match/stale semantics, `--scan` mode, self-test expectations). See frontmatter
comment above._

## Acceptance Criteria

### AC-001 — Regex detection of anchor citations
`validate-anchors` discovers all occurrences of the pattern
`(?:src|tests|fuzz)/[A-Za-z0-9_/.-]+\.rs:(\d+)(?:-(\d+))?` in any Markdown or
YAML file under `.factory/specs/**`.
- **Test:** `test_anchor_regex_detects_all_citation_forms()`
- _(traces to: pending BC authorship)_

### AC-002 — MATCH verdict for valid anchor
For a citation `src/foo.rs:42`, the tool opens `src/foo.rs`, confirms line 42
exists (file has ≥ 42 lines) and is not blank, and reports `MATCH`.
- **Test:** `test_match_verdict_on_valid_single_line_anchor()`
- _(traces to: pending BC authorship)_

### AC-003 — MATCH verdict for valid range anchor
For a citation `src/foo.rs:42-55`, the tool confirms that line 42 through 55
all exist (file has ≥ 55 lines) and reports `MATCH`.
- **Test:** `test_match_verdict_on_valid_range_anchor()`
- _(traces to: pending BC authorship)_

### AC-004 — STALE verdict for out-of-bounds line number
For a citation `src/foo.rs:9999` where `foo.rs` has fewer than 9999 lines,
the tool reports `STALE` (line is beyond EOF).
- **Test:** `test_stale_verdict_on_out_of_bounds_line()`
- _(traces to: pending BC authorship)_

### AC-005 — MISSING verdict for file not found
For a citation referencing a path that does not exist in the repo, the tool
reports `MISSING` (file absent) — distinct from `STALE` (file present, line gone).
- **Test:** `test_missing_verdict_on_absent_file()`
- _(traces to: pending BC authorship)_

### AC-006 — --scan mode table output
`validate-anchors --scan` iterates all `.factory/specs/**/*.md` and
`.factory/specs/**/*.yaml` files, prints a per-citation row:
`<STATUS>  <spec-file>:<line-in-spec>  ->  <cited-file>:<cited-line>`, and
a summary line `MATCH=N STALE=M MISSING=K`, then exits non-zero if any STALE
or MISSING citations are found.
- **Test:** `test_scan_mode_table_format_and_exit_code()`
- _(traces to: pending BC authorship)_

### AC-007 — Exit 0 on all-MATCH corpus
`validate-anchors --scan` exits 0 when the full corpus scan returns no STALE
or MISSING rows (clean corpus state).
- **Test:** `test_scan_exits_zero_on_all_match()`
- _(traces to: pending BC authorship)_

### AC-008 — Exit 1 on at least one STALE or MISSING
`validate-anchors --scan` exits 1 when at least one citation is STALE or
MISSING, regardless of how many MATCH rows exist.
- **Test:** `test_scan_exits_one_on_any_stale()`
- _(traces to: pending BC authorship)_

### AC-009 — Repo root resolution identical to compute-input-hash
The tool resolves the repo root using the same algorithm as `compute-input-hash`:
`WIRERUST_REPO_ROOT` env override → walk up from script path for `.factory/` →
walk up from `cwd` for `.factory/`. Source-file paths in citations are resolved
relative to the repo root.
- **Test:** `test_repo_root_resolution_matches_compute_input_hash()`
- _(traces to: pending BC authorship)_

### AC-010 — Self-test via companion test script
A companion script `bin/test_validate_anchors.py` exercises the regex engine,
all verdict types (MATCH / STALE / MISSING), the summary line format, and the
exit-code contract using synthetic fixture files — no live `.factory/specs/`
corpus required. Running `python3 bin/test_validate_anchors.py` must exit 0.
- **Test:** `test_self_test_script_passes()`
- _(traces to: pending BC authorship)_

### AC-011 — Governance policy: consistency audit after fix-bursts (secondary AC)
A written governance note (added to `.factory/policies.yaml` or equivalent
canonical policy file) codifies that:
(a) a spec-coherence consistency audit (consistency-validator full-corpus sweep,
6 dimensions) SHOULD be run after any fix-burst that shifts code lines, and
(b) the audit SHOULD be run after any BC H1/title enrichment pass,
so that anchor + title drift is caught proactively rather than one-finding-per-
adversarial-pass (the failure mode documented in PROCESS-GAP-P5-001).
The policy entry references PROCESS-GAP-P5-001 and this story ID.
- **Validation:** Policy entry is present and references `PROCESS-GAP-P5-001`
  and `STORY-091`.
- _(traces to: pending BC authorship; governance — not a code test)_

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `validate-anchors` CLI | `bin/validate-anchors` (Python 3, stdlib-only) | Effectful (filesystem reads) |
| Anchor regex engine | Internal function `find_anchors(text)` | Pure |
| Line-count verifier | Internal function `verify_anchor(repo_root, cited_path, line, end_line)` | Effectful (file open) |
| Self-test harness | `bin/test_validate_anchors.py` | Effectful (subprocess + tempdir) |
| Governance policy entry | `.factory/policies.yaml` | configuration (no code) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Citation inside a YAML `|` block scalar | Regex still matches; tool does not skip multi-line values |
| EC-002 | Citation in a Markdown code fence (` ```rust `) | Tool detects and reports it; no filtering of code-fence context (conservative — better to report than miss) |
| EC-003 | Citation with a line range where start > end (e.g., `src/foo.rs:55-42`) | Report `INVALID-RANGE` row; do not crash |
| EC-004 | Same citation appears in multiple spec files | Each occurrence is reported as its own row; shared STALE anchor is caught N times |
| EC-005 | Spec file is empty or contains no citations | No rows emitted for that file; does not crash |
| EC-006 | Source file exists but has zero non-blank lines at the cited line (blank line) | Report `MATCH` — blank line means the line exists; drift detection is about line-number validity, not content |
| EC-007 | `WIRERUST_REPO_ROOT` is set but the path does not contain `.factory/` | Tool exits with a clear error: `WIRERUST_REPO_ROOT does not contain .factory/` |
| EC-008 | Citation to `fuzz/` path (e.g., `fuzz/fuzz_targets/foo.rs:15`) | Processed identically to `src/` and `tests/` citations |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `bin/validate-anchors` (main) | Effectful | Reads filesystem; iterates spec tree |
| `find_anchors(text)` | Pure | String pattern match on in-memory text |
| `verify_anchor(...)` | Effectful | Opens source file to count lines |
| `bin/test_validate_anchors.py` | Effectful | Creates temp dirs, runs subprocess |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| `bin/compute-input-hash` (design reference) | ~3,000 |
| `bin/test_compute_input_hash.py` (test pattern reference) | ~1,500 |
| `.factory/policies.yaml` (governance insertion target) | ~2,000 |
| `bin/validate-anchors` (new file, to be written) | ~4,000 |
| `bin/test_validate_anchors.py` (new file, to be written) | ~3,000 |
| BC files (0 BCs — pending authorship) | 0 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~18,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~9%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests in `bin/test_validate_anchors.py` for AC-001 through
       AC-010 using synthetic fixture files in a temp directory (test-writer step)
2. [ ] Verify Red Gate: `python3 bin/test_validate_anchors.py` fails (tool absent)
3. [ ] Implement `bin/validate-anchors` with:
       - Argparse: `--scan`, `--help`, and a single-file positional mode
       - `find_anchors(text)` regex engine covering `src|tests|fuzz` prefix
       - `verify_anchor(repo_root, path, start_line, end_line)` returning
         `MATCH | STALE | MISSING | INVALID-RANGE`
       - Per-citation row format: `<STATUS>  <spec-file>:<row>  ->  <cited-file>:<lines>`
       - Summary line: `MATCH=N STALE=M MISSING=K`
       - Exit 0 all MATCH; exit 1 any STALE or MISSING
       - Repo root resolution identical to `compute-input-hash` algorithm
4. [ ] Make the tool executable: `chmod +x bin/validate-anchors`
5. [ ] Verify Green Gate: `python3 bin/test_validate_anchors.py` passes
6. [ ] Add governance policy entry to `.factory/policies.yaml` covering AC-011:
       - ID: `ANCHOR-VALIDATION-001`
       - Policy: consistency audit SHOULD run after any fix-burst shifting code
         lines and after any BC H1/title enrichment
       - References: `PROCESS-GAP-P5-001`, `STORY-091`
7. [ ] Add docstring to `bin/validate-anchors` mirroring `compute-input-hash`
       style (usage, algorithm, repo root resolution, self-test invocation)
8. [ ] Update `CLAUDE.md` "Project References" table and any phase-gate
       runbook section to reference `bin/validate-anchors --scan`
9. [ ] Run `python3 bin/test_validate_anchors.py` (final green verification)

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-11 | — | — | — |

The primary design reference is the existing `bin/compute-input-hash` tool, which
establishes:
- Python 3 stdlib-only (no third-party dependencies)
- Repo root resolution via `.factory/` directory detection
- `--scan` mode with tabular MATCH/STALE output
- Self-test companion script in `bin/`
- Docstring as the authoritative algorithm spec

`validate-anchors` should follow the same conventions so both tools are
immediately recognizable as part of the same tooling family.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Python 3 stdlib-only — no `pip install` required | Design parity with `compute-input-hash` | No `import` of non-stdlib modules; CI does not run pip |
| Regex must cover all three source-tree roots: `src/`, `tests/`, `fuzz/` | EC-008; PROCESS-GAP-P5-001 root cause (all dims) | `test_anchor_regex_detects_fuzz_prefix()` in self-test |
| Exit code 0 = all MATCH; exit code 1 = any STALE or MISSING | AC-007, AC-008 | Self-test asserts subprocess exit code |
| Repo root resolution algorithm is identical to `compute-input-hash` | AC-009 | Code comment in both tools cross-references each other |
| Tool must NOT modify any `.factory/specs/**` file — read-only | Safety rule | No write calls inside the scanner; a `--fix` mode is explicitly out of scope for v1 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Python 3 | ≥ 3.8 (matches `compute-input-hash` requirement) | Runtime; stdlib only |
| `re` | stdlib | Anchor citation regex |
| `pathlib.Path` | stdlib | File iteration and path manipulation |
| `argparse` | stdlib | CLI flags (`--scan`, `--help`) |
| `sys` | stdlib | Exit code control |
| `os` | stdlib | `WIRERUST_REPO_ROOT` env var lookup |
| `subprocess` | stdlib (test script only) | Invoke tool under test in `test_validate_anchors.py` |
| `tempfile` | stdlib (test script only) | Fixture file creation in self-test |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `bin/validate-anchors` | **create** | Main anchor-validation CLI tool (Python 3, executable) |
| `bin/test_validate_anchors.py` | **create** | Self-test companion script |
| `.factory/policies.yaml` | **modify** | Add `ANCHOR-VALIDATION-001` governance policy entry |
| `CLAUDE.md` | **modify** | Add `bin/validate-anchors` to "Project References" table and note phase-gate usage |
