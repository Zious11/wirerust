---
document_type: story
story_id: STORY-089
epic_id: E-9
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.014.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.015.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.016.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.017.md
input-hash: ""
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-086, STORY-087, STORY-088]
blocks: [STORY-090]
behavioral_contracts:
  - BC-2.12.014
  - BC-2.12.015
  - BC-2.12.016
  - BC-2.12.017
verification_properties: []
priority: P0
cycle: v1.0.0-brownfield
wave: 26
target_module: main
subsystems: [SS-12]
estimated_days: 2
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — full TDD Iron Law enforced.

> **Execute:** `/vsdd-factory:deliver-story STORY-089`

# STORY-089: Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing

## Narrative
- **As a** forensic analyst
- **I want** decode errors to be counted without aborting the run (first error warned once, rest silent), unclassified flow counts from the dispatcher to appear in the reassembly summary, output format to be resolved with `--json`/`--csv` taking precedence over `--output-format`, and the final output routed to a file or stdout based on the flag
- **So that** partial captures produce results, the full summary is complete, and output lands in the expected place

## Acceptance Criteria

### AC-001 (traces to BC-2.12.014 postcondition 1)
The first decode error in a packet loop prints exactly one warning to stderr: `"Warning: failed to decode packet ({e}). Further errors counted silently."`.
- **Test:** `test_first_decode_error_warning_printed()`

### AC-002 (traces to BC-2.12.014 postcondition 2)
Subsequent decode errors (2nd, 3rd, ...) are counted silently; no additional stderr output is emitted.
- **Test:** `test_subsequent_decode_errors_silent()`

### AC-003 (traces to BC-2.12.014 postcondition 3)
After the packet loop, `summary.skipped_packets` equals `total_decode_errors` (the total count of all failed decodes).
- **Test:** `test_skipped_packets_equals_total_decode_errors()`

### AC-004 (traces to BC-2.12.014 invariant 2)
The warning is printed at most ONCE per `run_analyze`/`run_summary` invocation, regardless of how many decode errors occur.
- **Test:** `test_decode_error_warning_printed_at_most_once()`

### AC-005 (traces to BC-2.12.015 postcondition 1)
When a reassembler was constructed, after `finalize()`, `dispatcher.unclassified_flows()` is injected into the reassembly `AnalysisSummary.detail` as `"unclassified_flows": <count>`.
- **Test:** `test_unclassified_flows_injected_into_reassembly_summary()`

### AC-006 (traces to BC-2.12.015 invariant 1)
When no reassembler was constructed, `"unclassified_flows"` is NOT present in any summary detail map.
- **Test:** `test_unclassified_flows_absent_without_reassembler()`

### AC-007 (traces to BC-2.12.016 postcondition 1)
`resolve_format(cli)` returns `Some(OutputFormat::Json)` when `cli.json.is_some()`, regardless of `--output-format` setting.
- **Test:** `test_resolve_format_json_flag_wins_over_output_format()`

### AC-008 (traces to BC-2.12.016 postcondition 2)
`resolve_format(cli)` returns `Some(OutputFormat::Csv)` when `cli.csv.is_some()` and `cli.json.is_none()`.
- **Test:** `test_resolve_format_csv_flag()`

### AC-009 (traces to BC-2.12.016 postcondition 3)
`resolve_format(cli)` returns `cli.output_format` (which may be `None`) when neither `--json` nor `--csv` is present.
- **Test:** `test_resolve_format_falls_back_to_output_format()`

### AC-010 (traces to BC-2.12.017 postcondition 1)
When `cli.json = Some(Some(path))`, `write_output` writes the string to the file at `path` via `std::fs::write`.
- **Test:** `test_write_output_json_with_path_writes_to_file()`

### AC-011 (traces to BC-2.12.017 postcondition 3)
When neither `--json <FILE>` nor `--csv <FILE>` is given, `write_output` prints to stdout via `println!`.
- **Test:** `test_write_output_default_to_stdout()`

### AC-012 (traces to BC-2.12.017 invariant 4)
File write errors are wrapped with anyhow context message `"Failed to write JSON output to <path>"` or `"Failed to write CSV output to <path>"`.
- **Test:** `test_write_output_file_write_error_has_context()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Decode-error handler | `src/main.rs:166-173` | effectful-shell (stderr write) |
| `unclassified_flows` injection | `src/main.rs:204-208` | pure-core (map insert) |
| `resolve_format` | `src/main.rs:304-320` | pure-core |
| `write_output` | `src/main.rs:322-338` | effectful-shell (file/stdout write) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Zero decode errors | `skipped_packets = 0`; no warning |
| EC-002 | All packets fail decode | `skipped_packets = N`; exactly one warning |
| EC-003 | Reassembler present, `unclassified_flows() = 0` | `"unclassified_flows": 0` in detail (zero is present, not absent) |
| EC-004 | `--json Some(None)` (flag given, no file path) | `write_output` prints to stdout |
| EC-005 | `--json` and `--output-format csv` both given | `resolve_format` returns `Some(Json)` (`--json` wins) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `resolve_format` | pure-core | Pure function of CLI struct fields; no I/O |
| `write_output` | effectful-shell | Writes to file or stdout |
| Decode-error counting | effectful-shell | `eprintln!` to stderr on first error |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| `src/main.rs` (relevant sections) | ~6,000 |
| `tests/cli_tests.rs`, `tests/reporter_tests.rs` | ~3,000 |
| BC files (4 BCs) | ~5,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~18,500** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~9%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-012 (test-writer)
2. [ ] Verify Red Gate: all tests fail
3. [ ] Implement decode-error handler with first-error warning and `total_decode_errors` counter
4. [ ] Implement `summary.skipped_packets = total_decode_errors` after packet loop
5. [ ] Implement `unclassified_flows` injection after `finalize()` when reassembler is Some
6. [ ] Implement `resolve_format` pure function with precedence: `--json` > `--csv` > `--output-format`
7. [ ] Implement `write_output` routing to file (for `Some(Some(path))`) or stdout
8. [ ] Write edge-case tests for EC-001 through EC-005
9. [ ] Verify purity: `resolve_format` is pure; `write_output` is effectful-shell
10. [ ] Run `cargo test --all-targets` and `cargo clippy -- -D warnings`

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-086 | `Commands` struct fields established | `cli.json: Option<Option<PathBuf>>` — nested Option for optional file path | `--json Some(None)` means flag given without path (print to stdout) |
| STORY-087 | `OutputFormat` ValueEnum established | `cli.output_format: Option<OutputFormat>` | `--json` and `--csv` are mutually exclusive via clap |
| STORY-088 | `run_analyze` structure known; packet loop pattern | `total_decode_errors` counter pattern | Warning must fire at most once per invocation |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Decode error warning is exactly the hardcoded string | BC-2.12.014 invariant 1 | `assert!(stderr.contains("Warning: failed to decode packet"))` |
| `total_decode_errors == 0` guards the first-error print | BC-2.12.014 invariant 1 | Code inspection; test that second error produces no extra output |
| `resolve_format` gives `--json`/`--csv` higher precedence than `--output-format` | BC-2.12.016 invariant 3 | Test: `--json` with `--output-format csv` returns `Some(Json)` |
| `write_output` error context strings are exact | BC-2.12.017 invariant 4 | `assert!(err.to_string().contains("Failed to write JSON output to"))` |
| `unclassified_flows` injection only when reassembler is Some | BC-2.12.015 invariant 1 | Test: no reassembler → key absent from detail map |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `anyhow` | workspace version | `bail!`, error context chaining for `write_output` |
| `serde_json` | workspace version | `serde_json::json!(count)` for unclassified_flows value |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/main.rs` | modify | Decode-error handler, `unclassified_flows` injection, `resolve_format`, `write_output` |
| `tests/cli_tests.rs` | modify | AC-007..AC-012 tests for `resolve_format` and `write_output` |
| `tests/reporter_tests.rs` | modify | AC-001..AC-006 tests for decode-error and dispatcher stats |
