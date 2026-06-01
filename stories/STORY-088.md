---
document_type: story
story_id: STORY-088
epic_id: E-9
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.008.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.009.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.010.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.011.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.012.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.013.md
input-hash: "9e10d74"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-086, STORY-087]
blocks: [STORY-089, STORY-090]
behavioral_contracts:
  - BC-2.12.008
  - BC-2.12.009
  - BC-2.12.010
  - BC-2.12.011
  - BC-2.12.012
  - BC-2.12.013
verification_properties: [VP-018]
priority: P0
cycle: v0.1.0-greenfield-spec
wave: 25
target_module: main
subsystems: [SS-12]
estimated_days: 3
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-088`

# STORY-088: run_analyze Orchestration — Analyzer Enablement, Reassembly Logic, Target Expansion, Progress Bar

## Narrative
- **As a** forensic analyst
- **I want** `run_analyze` to correctly wire `--all` into individual analyzer enables, enforce the `--no-reassemble` override with a warning, expand directory targets to sorted `.pcap` files, surface a per-file progress bar, and produce a clear error for non-existent targets
- **So that** the analysis pipeline starts with the correct set of analyzers regardless of which flag combination the operator used

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.12.008 | --all Enables dns/http/tls Together |
| BC-2.12.009 | needs_reassembly Logic; --no-reassemble Forces Off with Warning |
| BC-2.12.010 | NO_COLOR Env Var Disables Color |
| BC-2.12.011 | Directory Target Expands to *.pcap Sorted; *.pcapng Excluded |
| BC-2.12.012 | Non-Existent Target Yields bail! with Target Not Found |
| BC-2.12.013 | Per-Target Progress Bar on stderr via indicatif |

## Acceptance Criteria

### AC-001 (traces to BC-2.12.008 postcondition 1)
When `Commands::Analyze` has `all = true`, `run_analyze` computes `enable_dns = true`, `enable_http = true`, `enable_tls = true` via `dns || all`, `http || all`, `tls || all` respectively.
- **Test:** `test_all_flag_enables_all_three_analyzers()`

### AC-002 (traces to BC-2.12.008 invariant 3)
`--mitre` is NOT included in the `--all` expansion; when `all = true` and `mitre = false`, mitre-grouped rendering is not activated.
- **Test:** `test_all_does_not_imply_mitre()`

### AC-003 (traces to BC-2.12.009 postcondition 1; contributes to VP-018)
`needs_reassembly = cli.reassemble || enable_http || enable_tls` is computed correctly before reassembler construction. This is the `needs_reassembly` half of the CLI reassemble/no-reassemble mutual-exclusion property (VP-018): BC-2.12.007 governs the parse-time conflict; BC-2.12.009 governs the runtime override behavior. STORY-088 owns the runtime half.
- **Test:** `test_needs_reassembly_formula()`

### AC-004 (traces to BC-2.12.009 postcondition 5)
When `skip_reassembly = true` AND `enable_http || enable_tls`, a warning is printed to stderr matching: `"Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. Stream analysis will be skipped."`.
- **Test:** `test_no_reassemble_with_http_emits_warning()`

### AC-005 (traces to BC-2.12.009 postcondition 4)
When `skip_reassembly = true`, `http_analyzer` and `tls_analyzer` are not constructed (`None`).
- **Test:** `test_no_reassemble_skips_http_and_tls_constructors()`

### AC-006 (traces to BC-2.12.009 postcondition 6)
`dns_analyzer` is always constructed independently of reassembly (`None` reassembler does not block DNS).
- **Test:** `test_dns_analyzer_constructed_without_reassembly()`

### AC-007 (traces to BC-2.12.010 postcondition 1)
When `NO_COLOR` environment variable is set (any value, including empty string), `use_color = false` regardless of `--no-color` flag.
- **Test:** `test_no_color_env_var_disables_color()` (assert_cmd per-subprocess `.env()` injection — no `serial_test` required)

### AC-008 (traces to BC-2.12.010 postcondition 2)
When `NO_COLOR` is absent and `--no-color` is also absent, `use_color = true`.
- **Test:** `test_use_color_true_when_no_flags_set()` (assert_cmd per-subprocess `.env()` injection — no `serial_test` required)

### AC-009 (traces to BC-2.12.011 postcondition 1)
`resolve_targets` on a directory returns a sorted `Vec<PathBuf>` of only `.pcap` files; `.pcapng`, `.txt`, and other extensions are excluded.
- **Test:** `test_resolve_targets_directory_pcap_only_sorted()`

### AC-010 (traces to BC-2.12.011 invariant 1)
`resolve_targets` on a directory excludes files with uppercase extensions such as `.PCAP`; extension matching is case-sensitive (`ext == "pcap"` verbatim, not case-folded).
- **Test:** `test_resolve_targets_case_sensitive_extension_exclusion()`

### AC-011 (traces to BC-2.12.011 invariant 3)
Directory expansion is NOT recursive; subdirectories within the target directory are skipped.
- **Test:** `test_resolve_targets_not_recursive()`

### AC-012 (traces to BC-2.12.012 postcondition 1)
`resolve_targets` on a non-existent path returns `Err` whose message matches `"Target not found: <path>"`.
- **Test:** `test_resolve_targets_nonexistent_path_error()`

### AC-013 (traces to BC-2.12.013 postcondition 3)
The progress bar appears on stderr (not stdout) and is finished-and-cleared after each file's packet loop (`pb.finish_and_clear()` called). (Note: `indicatif` suppresses all rendering when stderr is a non-TTY pipe, which is always the case under assert_cmd. The formalization test therefore verifies the observable stdout-cleanliness guarantee — that no progress artifacts bleed into stdout — rather than directly observing stderr progress rendering. BC-2.12.013 stderr/finish_and_clear behavior is LOW-confidence from a behavioral-test perspective; the guarantee is structural, enforced by code review of the `finish_and_clear()` call site, not by subprocess observation.)
- **Test:** `test_progress_bar_does_not_appear_in_output()` (structural check: stdout does not contain ANSI progress bar bytes)

### AC-014 (traces to BC-2.12.013 invariant 4)
`run_summary` has NO progress bar. (Note: same TTY-limitation applies as AC-013 — indicatif suppresses rendering on non-TTY stderr, so the test verifies stdout cleanliness. The absence of a progress bar in `run_summary` is additionally confirmed by code-structure review.)
- **Test:** `test_run_summary_has_no_progress_bar()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `run_analyze` | `src/main.rs:54-210` | effectful-shell (I/O: file reads, stderr writes) |
| `resolve_targets` | `src/main.rs:340-360` | effectful-shell (filesystem reads) |
| `use_color` computation | `src/main.rs:43` | effectful-shell (env var read) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Directory with zero `.pcap` files | `resolve_targets` returns `Ok(vec![])` |
| EC-002 | Directory with `.PCAP` (uppercase extension) | Excluded (case-sensitive `== "pcap"`); promoted to AC-010 (BC-2.12.011 invariant 1) |
| EC-003 | `--no-reassemble` without `--http`/`--tls` | No warning emitted; reassembler simply not built |
| EC-004 | `NO_COLOR=""` (empty value) | `use_color = false` (any set value, including empty, counts) |
| EC-005 | Two pcap files in directory: `b.pcap`, `a.pcap` | Returned in order `[a.pcap, b.pcap]` (sorted) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `resolve_targets` | effectful-shell | Reads filesystem metadata |
| `use_color` computation | effectful-shell | Reads process environment |
| `run_analyze` | effectful-shell | Reads pcap files, writes progress to stderr |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| `src/main.rs` (run_analyze, resolve_targets) | ~8,000 |
| `tests/main_story_088_tests.rs` | ~3,000 |
| BC files (6 BCs — BC-2.12.008..013) | ~8,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~24,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~12%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-014 in `tests/main_story_088_tests.rs` (test-writer)
2. [ ] Verify Red Gate: all tests fail
3. [ ] Implement `--all` OR-expansion in `run_analyze` (main.rs:57-59)
4. [ ] Implement `needs_reassembly`/`skip_reassembly` logic with stderr warning (VP-018 runtime half)
5. [ ] Implement `resolve_targets` function (directory expansion + non-existent bail)
6. [ ] Implement `use_color` env-var check (tests use assert_cmd per-subprocess `.env()` injection — no `#[serial]` needed)
7. [ ] Implement progress bar creation and `finish_and_clear` pattern
8. [ ] Write edge-case tests for EC-001 through EC-005 in `tests/main_story_088_tests.rs`
9. [ ] Verify purity boundaries: `resolve_targets` and progress bar are effectful-shell only
10. [ ] Run `cargo test --all-targets`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-086 | `Cli` struct fields established; subcommands typed | `Commands::Analyze { all, dns, http, tls, mitre, targets }` | Keep `--services` absent |
| STORY-087 | Reassembly flags on `Cli`; `cli.reassemble`, `cli.no_reassemble` | `conflicts_with` declared on `--reassemble` | `cli.reassembly_depth` default is 10, `memcap` default is 1024 |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `resolve_targets` must NOT recurse into subdirectories | BC-2.12.011 invariant 3 | Test: subdir inside target dir is skipped |
| Extension check is `ext == "pcap"` (case-sensitive) | BC-2.12.011 invariant 1 | Test: `.PCAP` excluded |
| Warning message is exact hardcoded string | BC-2.12.009 invariant 1 | `assert!(stderr_output.contains("Warning: --http/--tls require TCP reassembly..."))` |
| `pb.finish_and_clear()` always called after inner loop | BC-2.12.013 invariant 2 | Structural review; progress bar template hardcoded |
| `NO_COLOR` env tests use assert_cmd per-subprocess `.env()` injection | BC-2.12.010, testing hygiene | No global env mutation; `#[serial]` not required |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `indicatif` | workspace version | Progress bar (`ProgressBar`, `ProgressStyle`) |
| `anyhow` | workspace version | `bail!` macro for `resolve_targets` error |
| `assert_cmd` | dev-dependency | Subprocess behavioral tests; `.env()` injects per-process env for `NO_COLOR` tests — `serial_test` is NOT required because each subprocess gets an isolated env |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/main.rs` | modify | `run_analyze` (analyzer enablement, reassembly logic, progress bar), `resolve_targets` |
| `tests/main_story_088_tests.rs` | create | AC-001..AC-014 test functions; assert_cmd subprocess tests for env-var and output checks |

Note: `tests/cli_tests.rs` is a pre-existing test file for other stories and is referenced as read-context only; it is NOT the artifact produced by this story. `Cargo.toml` does not require `serial_test` — assert_cmd subprocess isolation makes it unnecessary.
