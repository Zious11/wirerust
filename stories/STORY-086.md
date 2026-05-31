---
document_type: story
story_id: STORY-086
epic_id: E-9
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.001.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.002.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.003.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.006.md
input-hash: "4a6449b"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-080]
blocks: [STORY-087, STORY-088, STORY-089, STORY-090, STORY-096]
behavioral_contracts:
  - BC-2.12.001
  - BC-2.12.002
  - BC-2.12.003
  - BC-2.12.006
verification_properties: []
priority: P0
cycle: v0.1.0-greenfield-spec
wave: 23
target_module: cli
subsystems: [SS-12]
estimated_days: 2
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced (todo!() stubs + Red Gate density check required).

> **Execute:** `/vsdd-factory:deliver-story STORY-086`

# STORY-086: CLI Subcommand Parsing — analyze, summary, --no-color, Multiple Targets

## Narrative
- **As a** forensic analyst
- **I want to** invoke `wirerust analyze` or `wirerust summary` with one or more positional target paths and global flags like `--no-color`
- **So that** the CLI surface is correctly structured, required arguments are enforced by clap, and the flag state is accurately captured in the parsed struct

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.12.001 | analyze Subcommand Parses Positional Targets and All Flags |
| BC-2.12.002 | summary Subcommand Parses Targets and --hosts Flag |
| BC-2.12.003 | Global Flag --no-color Parsed and Stored |
| BC-2.12.006 | Multiple Positional Targets Accepted in analyze |

## Acceptance Criteria

### AC-001 (traces to BC-2.12.001 postcondition 1)
`Cli::try_parse_from(["wirerust", "analyze", "cap.pcap"])` returns `Ok` with `cli.command` matching `Commands::Analyze { targets: [cap.pcap], dns: false, http: false, tls: false, mitre: false, all: false, .. }`.
- **Test:** `test_analyze_subcommand_basic_parse()`

### AC-002 (traces to BC-2.12.001 postcondition 3)
When `--dns`, `--http`, `--tls`, `--mitre`, or `--all` are present, their corresponding struct fields are `true`; absent flags remain `false`.
- **Test:** `test_analyze_individual_protocol_flags()`

### AC-003 (traces to BC-2.12.001 invariant 1)
`Cli::try_parse_from(["wirerust", "analyze"])` (no targets) returns `Err`; clap surfaces a required-argument-missing error.
- **Test:** `test_analyze_requires_at_least_one_target()`

### AC-004 (traces to BC-2.12.001 invariant 3)
`--mitre` is a separate flag that sets `mitre = true` but does NOT enable any analyzer; `dns`, `http`, `tls` remain `false` when only `--mitre` is passed.
- **Test:** `test_mitre_flag_does_not_imply_analyzers()`

### AC-005 (traces to BC-2.12.002 postcondition 1)
`Cli::try_parse_from(["wirerust", "summary", "cap.pcap"])` returns `Ok` with `Commands::Summary { targets: [cap.pcap], hosts: false }`.
- **Test:** `test_summary_subcommand_basic_parse()`

### AC-006 (traces to BC-2.12.002 postcondition 3)
`--hosts` flag sets `hosts = true`; absent flag leaves `hosts = false`.
- **Test:** `test_summary_hosts_flag()`

### AC-007 (traces to BC-2.12.002 invariant 4)
`--services` (removed flag) is rejected by clap with `UnknownArgument`.
- **Test:** `test_summary_services_flag_removed()`

### AC-008 (traces to BC-2.12.003 postcondition 1)
`--no-color` sets `cli.no_color = true` whether placed before or after the subcommand (global flag semantics).
- **Test:** `test_no_color_flag_global_placement()`

### AC-009 (traces to BC-2.12.003 invariant 2)
`cli.no_color` is a plain `bool` (never `Option<bool>`); when absent it is `false`.
- **Test:** `test_no_color_flag_default_false()`

### AC-010 (traces to BC-2.12.006 postcondition 1)
`Cli::try_parse_from(["wirerust", "analyze", "a.pcap", "b.pcap", "c.pcap"])` produces `targets = [a.pcap, b.pcap, c.pcap]` in command-line order; duplicates are preserved.
- **Test:** `test_multiple_targets_preserve_order_and_duplicates()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Cli` struct | `src/cli.rs` | pure-core (clap parse is pure) |
| `Commands::Analyze` variant | `src/cli.rs:113-139` | pure-core |
| `Commands::Summary` variant | `src/cli.rs:141-155` | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `--all` flag with individual protocol flags | `all = true`; individual flags also true if provided |
| EC-002 | `--mitre` alone | `mitre = true`, `all = false`, `dns/http/tls = false` |
| EC-003 | `--hosts` on analyze subcommand | Clap error — `--hosts` is only on `summary` |
| EC-004 | `--services` passed to `summary` | Clap `UnknownArgument` error |
| EC-005 | Duplicate targets `a.pcap a.pcap` | `targets = [a.pcap, a.pcap]` (no dedup at parse time) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/cli.rs` | pure-core | Clap derive parsing is a pure transformation of argv to a typed struct |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| `src/cli.rs` (source reference) | ~4,000 |
| `tests/cli_story_086_tests.rs` (new formalization tests) | ~3,000 |
| BC files (4 BCs) | ~5,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~15,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~8%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all 10 tests fail against existing stubs
3. [ ] Implement `Commands::Analyze` and `Commands::Summary` clap structs to pass tests
4. [ ] Add `--no-color` global flag to `Cli` struct
5. [ ] Verify `targets` is `Vec<PathBuf>` with `required = true` in both subcommands
6. [ ] Confirm `--services` is absent (no declaration)
7. [ ] Write edge-case tests for EC-001 through EC-005
8. [ ] Verify purity boundaries: all assertions in `cli.rs` are pure clap parse
9. [ ] Run `cargo test --all-targets` and `cargo clippy --all-targets -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-9 | — | — | — |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| CLI parsing is pure; no I/O in `cli.rs` | ADR-based purity boundary | `cargo clippy`; grep for `std::fs` or `std::io` in `cli.rs` |
| `--services` must be absent (removed by PR #74) | BC-2.12.002 invariant 4, LESSON-P1.04 | `grep -n 'services' src/cli.rs` returns nothing |
| `targets` uses `required = true` in clap derive | BC-2.12.001 invariant 1 | Test: zero-target parse returns Err |
| `--no-color` is `global = true` | BC-2.12.003 invariant 1 | Test: flag placed before subcommand still sets field |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `clap` | `>= 4.0` (workspace version) | CLI argument parsing via derive macros |
| `serde` | workspace version | (indirect; not used in cli.rs directly) |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/cli.rs` | modify (formalize tests exist) | Clap struct definitions for `Cli`, `Commands`, `OutputFormat` |
| `tests/cli_story_086_tests.rs` | modify | Add AC-001..AC-010 test functions and edge-case tests |
