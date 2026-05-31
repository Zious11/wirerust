# Red Gate Log — STORY-089

**Timestamp:** 2026-05-31
**Agent:** test-writer
**Story:** STORY-089 — Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing
**Wave:** 26
**Implementation Strategy:** brownfield-formalization

## Phase 1: Architecture Verification

Confirmed implementation anchors against src/main.rs (no drift):

| Anchor | BC | Actual location | Matches story spec? |
|--------|----|-----------------|---------------------|
| Decode-error handler with eprintln guard | BC-2.12.014 | src/main.rs:166-173 | YES |
| `summary.skipped_packets = total_decode_errors` | BC-2.12.014 pc3 | src/main.rs:183 | YES |
| Same pattern in `run_summary` | BC-2.12.014 | src/main.rs:266-276, 278 | YES |
| `unclassified_flows` injection into reassembly detail | BC-2.12.015 | src/main.rs:204-208 | YES |
| `resolve_format` pure function with precedence | BC-2.12.016 | src/main.rs:312-320 | YES |
| `write_output` file/stdout routing with anyhow context | BC-2.12.017 | src/main.rs:327-338 | YES |
| Error context strings exact | BC-2.12.017 inv4 | src/main.rs:330, 332 | YES |

All BC postconditions satisfied by existing implementation.

## Phase 2: Binary Behavior Verification

Before writing tests, the binary was run against fixtures to confirm exact output
markers. Key observations:

### Decode Errors (BC-2.12.014)
- `dns-remoteshell.pcap`: 58 total packets, 73 fail `decode_packet` ("No IP layer found").
  Produces EXACTLY 1 warning line on stderr. `skipped_packets = 73` in JSON output.
- `http-ooo.pcap`: 16 clean packets, 0 decode errors, `skipped_packets = 0`.
- Warning text confirmed: `"Warning: failed to decode packet (No IP layer found). Further errors counted silently."`

### Unclassified Flows (BC-2.12.015)
- `analyze http-ooo.pcap --http --json`: `"unclassified_flows": 0` present in TCP Reassembly detail.
- `analyze http-ooo.pcap --dns --no-reassemble --json`: `"unclassified_flows"` key ABSENT.
- `analyze http-ooo.pcap --dns --json` (no reassembler even with --dns alone): key ABSENT.

### Format Resolution (BC-2.12.016)
- `--json --output-format csv`: output starts with `{` (JSON wins, not CSV).
- `--csv`: output starts with `category,verdict,confidence,...` CSV header.
- No flags: output starts with `WIRERUST TRIAGE REPORT` (terminal format).
- `--output-format json`: output starts with `{`.
- `--output-format csv`: output starts with `category,`.

### Write Output (BC-2.12.017)
- `--json <path>`: file written; stdout empty.
- `--json` (no path): stdout contains JSON.
- No flags: stdout contains terminal table.
- `--json /nonexistent/dir/out.json`: failure with "Error: Failed to write JSON output to ...".
- `--csv /nonexistent/dir/out.csv`: failure with "Error: Failed to write CSV output to ...".

## Phase 3: Fixture Decision

No new fixture required. `dns-remoteshell.pcap` naturally produces decode errors
(73 non-IP packets that fail `decode_packet`). This covers:
- AC-001 (first warning), AC-002 (subsequent silent), AC-003 (skipped_packets=73),
- AC-004 (at-most-once), EC-002 (all/many errors), EC-005 (one warning, correct count).

`http-ooo.pcap` (0 decode errors) covers EC-001 (zero errors, no warning, skipped=0).

## Phase 4: Source Change Decision

resolve_format extraction decision: NOT extracted to lib.rs.

Justification: Behavioral tests for AC-007/008/009 are mutation-resistant because they
assert the actual output FORMAT produced (JSON '{' prefix, CSV header line, terminal table
header). An adversarial mutation swapping Json/Csv/Terminal would flip the observed format
and fail the assertion. This provides equivalent mutation-resistance to a direct unit test
on `resolve_format(cli)`, without requiring any src changes.

This is consistent with the STORY-088 precedent: binary-private functions are tested
purely behaviorally via assert_cmd subprocess invocation.

## Phase 5: Red Gate Execution

Test file created: `tests/main_story_089_tests.rs`
Module: `mod story_089` (DF-TEST-NAMESPACE-001)
Test count: 17 (12 ACs + 5 ECs)

All test bodies set to `assert!(false, "RED GATE STUB: test not yet verified against implementation")`.

```
running 17 tests
test story_089::test_EC_002_unclassified_flows_zero_still_present_in_detail ... FAILED
test story_089::test_EC_001_zero_decode_errors_no_warning_and_skipped_zero ... FAILED
test story_089::test_decode_error_warning_printed_at_most_once ... FAILED
test story_089::test_EC_003_json_flag_without_path_writes_to_stdout ... FAILED
test story_089::test_EC_004_json_wins_over_output_format_csv ... FAILED
test story_089::test_EC_005_all_packets_fail_one_warning_skipped_count_accurate ... FAILED
test story_089::test_resolve_format_csv_flag ... FAILED
test story_089::test_first_decode_error_warning_printed ... FAILED
test story_089::test_resolve_format_json_flag_wins_over_output_format ... FAILED
test story_089::test_skipped_packets_equals_total_decode_errors ... FAILED
test story_089::test_subsequent_decode_errors_silent ... FAILED
test story_089::test_unclassified_flows_absent_without_reassembler ... FAILED
test story_089::test_resolve_format_falls_back_to_output_format ... FAILED
test story_089::test_write_output_default_to_stdout ... FAILED
test story_089::test_unclassified_flows_injected_into_reassembly_summary ... FAILED
test story_089::test_write_output_file_write_error_has_context ... FAILED
test story_089::test_write_output_json_with_path_writes_to_file ... FAILED

test result: FAILED. 0 passed; 17 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Red Gate: VERIFIED — all 17 stubs failed as required.**

## src/ Change Verification

```
git diff --stat develop -- src/
(empty — no changes to src/)
```

**Confirmed: ZERO src/ changes. brownfield-formalization mode.**

## Housekeeping Note

Story FSR cites `tests/cli_tests.rs` and `tests/reporter_tests.rs` as target files.
Per instructions, the actual test file is `tests/main_story_089_tests.rs` (dedicated
namespace per DF-TEST-NAMESPACE-001). This clears the STORY-089 half of deferred
F-FSR-088-089. The factory-artifact story file FSR discrepancy is noted for the
orchestrator to route — NOT modified from this worktree.
