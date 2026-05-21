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
id: "HS-086"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-10"
behavioral_contracts:
  - BC-2.13.001
  - BC-2.13.002
  - BC-2.13.003
  - BC-2.13.004
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Obsolete Flags --threats, --beacon, --filter, --verbose Are Actively Rejected

## Scenario

An analyst who is familiar with an older version of wirerust may attempt to use flags that
were removed in PR #74. Rather than silently ignoring these flags or doing something unexpected,
the tool must immediately and clearly reject them with an error.

1. The tool is invoked with `wirerust analyze --threats test.pcap`.
   - Exit code is non-zero.
   - No analysis is performed.
   - Error output indicates an unknown or unrecognized argument.

2. The tool is invoked with `wirerust analyze --beacon test.pcap`.
   - Same rejection behavior.

3. The tool is invoked with `wirerust analyze --filter tcp test.pcap`.
   - Same rejection behavior.

4. The tool is invoked with `wirerust analyze --verbose test.pcap`.
   - Same rejection behavior.

5. The tool is invoked with `wirerust analyze -v test.pcap` (short form of --verbose).
   - Same rejection behavior.

6. A valid invocation `wirerust analyze --http test.pcap` completes normally, confirming that
   the removal of the obsolete flags does not affect valid invocations.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.13.001 | Postcondition 1: --threats rejected | Item 1 above |
| BC-2.13.002 | Postcondition 1: --beacon rejected | Item 2 above |
| BC-2.13.003 | Postcondition 1: --filter rejected | Item 3 above |
| BC-2.13.004 | Postcondition 1: --verbose rejected | Item 4 above |
| BC-2.13.004 | Postcondition 1: -v rejected | Item 5 (short form) |
| BC-2.13.001 | Postcondition 3: valid invocation unaffected | Item 6 above |

## Verification Approach

For each removed flag, run the tool in a subprocess and capture exit code and stderr:

```
# Test each removed flag
for flag in "--threats" "--beacon" "--verbose" "-v"; do
    wirerust analyze $flag test.pcap
    assert exit_code != 0
    assert stderr contains "unexpected argument" or "unrecognized"
done

# Test --filter specially (it takes an argument)
wirerust analyze --filter tcp test.pcap
assert exit_code != 0

# Test valid invocation
wirerust analyze --http <real_or_nonexistent>.pcap
# Should not fail due to the removed flags; may fail on file-not-found separately
```

The error messages do not need to be verbatim; what matters is non-zero exit and error output
mentioning the unexpected flag name.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): All five removed flags (including -v) produce non-zero exit and error output.
- **Edge case handling** (weight: 0.2): Flags placed before or after the subcommand are both rejected.
- **Error quality** (weight: 0.15): Error mentions the specific unknown argument; does not crash with a panic.
- **Performance** (weight: 0.05): Immediate parse-time rejection.
- **Data integrity** (weight: 0.1): Valid invocations (`--http`) are completely unaffected by the removal.

## Edge Conditions

- `--threats` placed before the subcommand: `wirerust --threats analyze test.pcap` — also rejected.
- `--beacon` combined with valid flags like `--http`: still rejected immediately.
- `--filter` with a space-separated expression: clap rejects at `--filter` before parsing the expression.
- These flags must not silently pass through as free positional arguments.

## Failure Guidance

"HOLDOUT LOW: HS-086 (satisfaction: 0.XX) -- One or more removed flags (--threats, --beacon, --filter, --verbose, -v) were not rejected by the CLI — they were silently ignored or produced unexpected behavior instead of a parse error."
