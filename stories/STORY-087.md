---
document_type: story
story_id: STORY-087
epic_id: E-9
version: "1.2"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.004.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.005.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.007.md
input-hash: "1de3972"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-086]
blocks: [STORY-088, STORY-089]
behavioral_contracts:
  - BC-2.12.004
  - BC-2.12.005
  - BC-2.12.007
verification_properties: [VP-018]
priority: P0
cycle: v0.1.0-greenfield-spec
wave: 24
target_module: cli
subsystems: [SS-12]
estimated_days: 2
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-087`

# STORY-087: Output Format Flags and Reassembly Configuration Flags

## Narrative
- **As a** forensic analyst
- **I want to** control output format (`--output-format json|csv`, `--json`, `--csv`) and TCP reassembly parameters (`--reassemble`, `--no-reassemble`, `--reassembly-depth`, `--reassembly-memcap`, five threshold flags)
- **So that** the output channel and reassembly tuning are correctly encoded in the parsed CLI struct with proper defaults and mutual-exclusion enforcement

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.12.004 | --output-format json Parses to Some(OutputFormat::Json) |
| BC-2.12.005 | Reassembly CLI Flags: --reassemble/--no-reassemble, depth, memcap, and five anomaly-threshold flags |
| BC-2.12.007 | --reassemble and --no-reassemble are Mutually Exclusive (clap conflicts_with) |

## Acceptance Criteria

### AC-001 (traces to BC-2.12.004 postcondition 1)
`Cli::try_parse_from(["wirerust", "--output-format", "json", "summary", "x.pcap"])` yields `output_format = Some(OutputFormat::Json)`.
- **Test:** `test_output_format_json_flag()`

### AC-002 (traces to BC-2.12.004 postcondition 2)
`--output-format csv` yields `output_format = Some(OutputFormat::Csv)`.
- **Test:** `test_output_format_csv_flag()`

### AC-003 (traces to BC-2.12.004 postcondition 3)
When `--output-format` is absent, `output_format = None`.
- **Test:** `test_output_format_absent_is_none()`

### AC-004 (traces to BC-2.12.004 postcondition 4)
`--output-format xml` causes a clap parse error (unrecognized variant).
- **Test:** `test_output_format_invalid_value_rejected()`

### AC-005 (traces to BC-2.12.005 postcondition 3)
When `--reassembly-depth` is absent, `cli.reassembly_depth = 10` (default).
- **Test:** `test_reassembly_depth_default_is_10()`

### AC-006 (traces to BC-2.12.005 postcondition 4)
When `--reassembly-memcap` is absent, `cli.reassembly_memcap = 1024` (default).
- **Test:** `test_reassembly_memcap_default_is_1024()`

### AC-007 (traces to BC-2.12.005 postcondition 5)
Threshold override flags (`--overlap-threshold`, `--small-segment-threshold`, etc.) are `None` when absent and `Some(value)` when provided.
- **Test:** `test_reassembly_threshold_flags_default_none()`

### AC-008 (traces to BC-2.12.005 postcondition 6)
`--overlap-threshold 256` is rejected by clap (out of 0-255 range).
- **Test:** `test_overlap_threshold_out_of_range_rejected()`

### AC-009 (traces to BC-2.12.005 invariant 3)
`--small-segment-ignore-ports 23,513` produces `small_segment_ignore_ports = Some([23, 513])` (comma-delimited Vec<u16>).
- **Test:** `test_small_segment_ignore_ports_comma_delimited()`

### AC-010 (traces to BC-2.12.007 postcondition 1)
`Cli::try_parse_from` with both `--reassemble` AND `--no-reassemble` returns `Err` with `ArgumentConflict` error kind.
- **Test:** `test_reassemble_and_no_reassemble_conflict()`

### AC-011 (traces to BC-2.12.007 invariant 1)
The conflict is symmetric: `--no-reassemble --reassemble` (reversed order) also returns `Err`.
- **Test:** `test_reassemble_conflict_is_symmetric()`

### AC-012 (traces to BC-2.12.007 edge case EC-003)
`--reassemble` alone parses successfully; `cli.reassemble = true`.
- **Test:** `test_reassemble_alone_parses_ok()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Cli` struct output format fields | `src/cli.rs:47-49` | pure-core |
| Reassembly flags on `Cli` struct | `src/cli.rs:61-106` | pure-core |
| `OutputFormat` ValueEnum | `src/cli.rs` | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `--reassembly-depth 0` | Accepted (0 is a valid u64); `reassembly_depth = 0` |
| EC-002 | `--small-segment-max-bytes 0` | `small_segment_max_bytes = Some(0)` (disables detection) |
| EC-003 | `--overlap-threshold 255` (max) | `overlap_threshold = Some(255)`; accepted |
| EC-004 | `--output-format` and `--json` together | `--json` wins via `resolve_format` precedence (BC-2.12.016; not tested here — tested in STORY-089) |
| EC-005 | No reassembly flags at all | `reassemble = false`, `no_reassemble = false`, `depth = 10`, `memcap = 1024` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/cli.rs` reassembly fields | pure-core | Clap derive parse is pure; no I/O at parse time |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,800 |
| `src/cli.rs` (reassembly and output sections) | ~3,500 |
| `tests/cli_story_087_tests.rs` (new formalization tests) | ~2,500 |
| BC files (3 BCs) | ~4,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~14,300** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~7%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-012 (test-writer)
2. [ ] Verify Red Gate: all tests fail
3. [ ] Add `OutputFormat` ValueEnum with `Json` and `Csv` variants
4. [ ] Wire `output_format: Option<OutputFormat>` onto `Cli` struct
5. [ ] Add all reassembly flags to `Cli` with correct defaults and range constraints
6. [ ] Add `--small-segment-ignore-ports` as `Option<Vec<u16>>` with `value_delimiter = ','`
7. [ ] Declare `conflicts_with = "no_reassemble"` on `--reassemble`
8. [ ] Write edge-case tests for EC-001 through EC-005
9. [ ] Run `cargo test --all-targets` and `cargo clippy --all-targets -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-086 | `Cli` struct is the root; global flags use `global = true` | Clap derive only; no runtime logic in `cli.rs` | `--services` removal must stay gone; grep to verify |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `--reassemble` declares `conflicts_with = "no_reassemble"` (asymmetric; clap makes it bidirectional) | BC-2.12.007 invariant 1 | Test: both flags together returns Err(ArgumentConflict) |
| `OutputFormat` is a `ValueEnum`; only `json` and `csv` are valid | BC-2.12.004 invariant 1 | Test: invalid value causes parse error |
| All reassembly flags are `global = true` | BC-2.12.005 invariant 1 | Verify attribute in `src/cli.rs` |
| `overlap_threshold` clamp is 0-255 | BC-2.12.005 postcondition 6 | clap `value_parser = clap::value_parser!(u32).range(0..=255)` |
| `small_segment_threshold` clamp is 0-2048 | BC-2.12.005 postcondition 7 | clap range validator |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `clap` | `>= 4.0` (workspace) | `ValueEnum`, range-checked `value_parser`, `conflicts_with` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/cli.rs` | modify | Add `OutputFormat` enum, output format fields, all reassembly flags |
| `tests/cli_story_087_tests.rs` | modify | Add AC-001..AC-012 test functions |
