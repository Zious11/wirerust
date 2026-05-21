---
document_type: story
story_id: STORY-096
epic_id: E-10
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-13/BC-2.13.001.md
  - .factory/specs/behavioral-contracts/ss-13/BC-2.13.002.md
  - .factory/specs/behavioral-contracts/ss-13/BC-2.13.003.md
  - .factory/specs/behavioral-contracts/ss-13/BC-2.13.004.md
input-hash: ""
traces_to: .factory/specs/prd.md
points: 3
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.13.001
  - BC-2.13.002
  - BC-2.13.003
  - BC-2.13.004
verification_properties: []
priority: P1
cycle: v1.0.0-brownfield
wave: 24
target_module: cli
subsystems: [SS-13]
estimated_days: 1
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-096`

# STORY-096: Absent Behavior Contracts — Removed Flags Rejected by clap

## Narrative
- **As a** forensic analyst who may be familiar with older wirerust versions
- **I want** `--threats`, `--beacon`, `--filter`, and `--verbose` to be actively rejected by clap with an unknown-argument error
- **So that** I receive immediate, clear feedback when typing an obsolete flag rather than silent misuse of a removed feature

## Acceptance Criteria

### AC-001 (traces to BC-2.13.001 postcondition 1)
`Cli::try_parse_from(["wirerust", "analyze", "--threats", "test.pcap"])` returns `Err` with an error kind indicating an unknown or unexpected argument.
- **Test:** `test_threats_flag_rejected_by_clap()`

### AC-002 (traces to BC-2.13.001 invariant 1)
Neither `--threats` nor any `threats`-related field appears in `Cli`, `Commands::Analyze`, or any other subcommand in `src/cli.rs`.
- **Test:** `test_threats_field_absent_from_cli()` (grep-based: `grep -n 'threats' src/cli.rs` returns nothing)

### AC-003 (traces to BC-2.13.002 postcondition 1)
`Cli::try_parse_from(["wirerust", "analyze", "--beacon", "test.pcap"])` returns `Err` with an unknown-argument error.
- **Test:** `test_beacon_flag_rejected_by_clap()`

### AC-004 (traces to BC-2.13.002 invariant 2)
No `C2BeaconAnalyzer` or equivalent struct exists in `src/`.
- **Test:** `test_beacon_analyzer_absent_from_src()` (grep-based: `grep -rn 'beacon\|Beacon' src/` finds no analyzer)

### AC-005 (traces to BC-2.13.003 postcondition 1)
`Cli::try_parse_from(["wirerust", "analyze", "--filter", "tcp", "test.pcap"])` returns `Err` with an unknown-argument error.
- **Test:** `test_filter_flag_rejected_by_clap()`

### AC-006 (traces to BC-2.13.003 invariant 2)
No BPF expression evaluation exists in `src/`; all packets from an accepted pcap are processed without pre-filtering.
- **Test:** `test_bpf_filter_absent_from_src()` (structural: no BPF library in Cargo.toml)

### AC-007 (traces to BC-2.13.004 postcondition 1)
`Cli::try_parse_from(["wirerust", "analyze", "--verbose", "test.pcap"])` returns `Err` with an unknown-argument error.
- **Test:** `test_verbose_flag_rejected_by_clap()`

### AC-008 (traces to BC-2.13.004 postcondition 1)
`Cli::try_parse_from(["wirerust", "analyze", "-v", "test.pcap"])` also returns `Err` (short form also not declared).
- **Test:** `test_verbose_short_flag_rejected_by_clap()`

### AC-009 (traces to BC-2.13.004 invariant 1)
No `--verbose` or `-v` declaration exists in the CLI surface; no log-level control exists.
- **Test:** `test_verbose_field_absent_from_cli()` (grep-based: `grep -n 'verbose' src/cli.rs` returns nothing relevant)

### AC-010 (traces to BC-2.13.001 postcondition 3 / BC-2.13.002 postcondition 3)
A valid invocation without any removed flag (e.g., `wirerust analyze test.pcap`) parses successfully — removed flags do not affect valid parses.
- **Test:** `test_valid_invocation_unaffected_by_absent_flags()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| CLI absent-flag enforcement | `src/cli.rs` (flag absence = clap rejects at parse time) | pure-core (clap parse) |
| LESSON-P1.04 comment | `src/cli.rs:24-32` | documentation |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `--threats` placed before subcommand | Also rejected (clap treats unknown global args as errors) |
| EC-002 | `--beacon` combined with valid flags | Error fires before any analysis |
| EC-003 | `--filter "tcp port 80"` with a space-separated expression | Error fires on `--filter`; clap does not parse the rest |
| EC-004 | Valid invocation: `wirerust analyze --http test.pcap` | Parses successfully; none of the four removed flags affect this |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/cli.rs` | pure-core | Absence of declarations is enforced by clap at parse time; pure |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,000 |
| `src/cli.rs` (reviewed for absent fields) | ~3,500 |
| `tests/cli_tests.rs` | ~2,000 |
| BC files (4 BCs) | ~4,500 |
| Tool outputs overhead | ~500 |
| **Total** | **~12,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~6%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all tests fail (they currently pass because the flags are indeed absent — so the test-writer must write assertions that CONFIRM absence and rejection; the Red Gate is established by writing the assertion as a `todo!()` stub first)
3. [ ] Verify `--threats` is absent from `src/cli.rs` (no declaration)
4. [ ] Verify `--beacon` is absent (no `C2BeaconAnalyzer` or related struct)
5. [ ] Verify `--filter` is absent (no BPF library in `Cargo.toml`)
6. [ ] Verify `--verbose`/`-v` is absent
7. [ ] Add LESSON-P1.04 comment to `src/cli.rs` documenting all four removed flags if not already present
8. [ ] Write grep-based tests (code-level assertions) for field absence
9. [ ] Write edge-case tests for EC-001 through EC-004
10. [ ] Run `cargo test --all-targets`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-10 | — | — | These 4 BCs are "absent behavior" contracts (SS-13); the story proves ABSENCE, not presence. Tests assert `try_parse_from` returns Err AND that no struct field exists. |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `--threats` is not declared in `Cli` or any subcommand | BC-2.13.001 invariant 1 | `grep -n 'threats' src/cli.rs` returns no declaration |
| `--beacon` is not declared; no `C2BeaconAnalyzer` exists | BC-2.13.002 invariant 2 | `grep -rn 'beacon\|Beacon' src/` finds no analyzer struct |
| `--filter` is not declared; no BPF dependency in Cargo.toml | BC-2.13.003 invariant 2 | `grep -n 'bpf\|filter' Cargo.toml` finds no BPF library |
| `--verbose` and `-v` are not declared | BC-2.13.004 invariant 1 | `grep -n 'verbose' src/cli.rs` returns nothing |
| LESSON-P1.04 comment documents the removal of these flags (PR #74) | LESSON-P1.04 | Present in `src/cli.rs:24-32` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `clap` | `>= 4.0` (workspace) | Unknown argument rejection happens automatically when flags are not declared |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/cli.rs` | verify (no modifications required) | Confirm four removed flags are absent; LESSON-P1.04 comment exists |
| `tests/cli_tests.rs` | modify | Add AC-001..AC-010 test functions proving rejection and absence |
