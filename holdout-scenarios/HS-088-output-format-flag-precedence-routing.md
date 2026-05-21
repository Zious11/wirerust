---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs: [stories/, behavioral-contracts/, prd.md]
input-hash: "[md5-pending]"
traces_to: ""
id: "HS-088"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.016
  - BC-2.12.017
  - BC-2.12.014
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: --json Flag Wins Over --output-format; Output Routes to File or Stdout Correctly

## Scenario

The format resolution logic has a documented precedence: the explicit `--json`/`--csv` flags
override the `--output-format` flag. The routing logic then sends output to a file or stdout.
An analyst who uses `--json --output-format csv` must get JSON, not CSV.

**Part A — precedence:**
1. The tool is invoked with `wirerust --json --output-format csv analyze <pcap>`.
2. The stdout output is valid JSON (begins with `{`).
3. The output is NOT in CSV format (no nine-column header row).

**Part B — file routing:**
1. The tool is invoked with `wirerust --json /tmp/wirerust_out.json analyze <pcap>`.
2. The file `/tmp/wirerust_out.json` is created containing valid JSON.
3. Stdout produces no JSON output (or minimal progress info only).
4. The file contents parse successfully as JSON with a "findings" key.

**Part C — decode error counting:**
1. The tool is run against a pcap file with known malformed packets.
2. Exactly one warning appears on stderr mentioning a decode failure; subsequent errors are silent.
3. The summary in the output shows `skipped_packets` equal to the total number of decode errors.
4. The run does not abort — it completes and produces a report for the non-malformed packets.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.016 | Postcondition 1: --json wins over --output-format | Part A: JSON despite --output-format csv |
| BC-2.12.017 | Postcondition 1: file path → write to file | Part B: output in file not stdout |
| BC-2.12.014 | Postcondition 1: first decode error warns once | Part C: exactly one warning |
| BC-2.12.014 | Postcondition 3: skipped_packets = total decode errors | Part C: count in output |
| BC-2.12.014 | Invariant 2: warning printed at most once per invocation | Part C: no repeated warnings |

## Verification Approach

**Part A:**
Run `wirerust --json --output-format csv analyze <pcap>` (or use clap's option to combine both).
Capture stdout. Assert: the output begins with `{` and is valid JSON. Assert: no CSV header
line (`category,verdict,...`) appears.

**Part B:**
Run `wirerust --json /tmp/test_out.json analyze <pcap>`. Assert: exit code 0. Assert: file
`/tmp/test_out.json` exists. Read it and assert it parses as valid JSON with a `"findings"` key.

**Part C:**
Use a crafted pcap with at least 3 intentionally malformed packets. Run `wirerust analyze <pcap>`.
Capture stderr. Assert: exactly one line on stderr contains "failed to decode packet" or similar.
Assert: the terminal or JSON output contains `"skipped_packets": 3` (or the actual count).
Assert: the output also contains findings from the valid packets.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): All three parts match the described behavior.
- **Edge case handling** (weight: 0.2): Precedence is correct; file routing does not duplicate to stdout.
- **Error quality** (weight: 0.2): Decode error warning is informative; file-write errors include path context.
- **Performance** (weight: 0.05): No slowdown from error counting.
- **Data integrity** (weight: 0.1): JSON output is valid and complete even when some packets were skipped.

## Edge Conditions

- `--json` with no file path: routes to stdout (flag given but no path argument).
- `--json path` where path directory doesn't exist: exits with error mentioning "Failed to write JSON output to <path>".
- Zero decode errors: `skipped_packets = 0`; no warning; terminal output shows no "Skipped:" line.
- All packets malformed: `skipped_packets = N`; exactly one warning; empty findings list.

## Failure Guidance

"HOLDOUT LOW: HS-088 (satisfaction: 0.XX) -- Output format precedence was wrong (--output-format overrode --json), file routing failed to write to the specified path, or decode error warning appeared multiple times."
