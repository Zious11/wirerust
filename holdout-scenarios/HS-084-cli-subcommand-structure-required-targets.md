---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-086.md
  - .factory/stories/STORY-087.md
  - .factory/stories/STORY-088.md
  - .factory/stories/STORY-089.md
  - .factory/stories/STORY-090.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.001.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.002.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.003.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.006.md
input-hash: "db27506"
traces_to: .factory/stories/STORY-086.md
id: "HS-084"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.001
  - BC-2.12.002
  - BC-2.12.003
  - BC-2.12.006
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: CLI Subcommand Parsing Enforces Required Targets and Correct Flag Semantics

## Scenario

An analyst who mis-types or omits required arguments must receive an immediate, clear error
from the CLI — not a silent no-op or misleading behavior.

**Part A — missing target:**
1. The tool is invoked as `wirerust analyze` with no target file.
2. The tool exits with a non-zero exit code.
3. The error output indicates a missing required argument (not an unrelated error).

**Part B — multiple targets preserved:**
1. The tool is invoked with three pcap files: `wirerust analyze a.pcap b.pcap c.pcap`.
2. All three files are processed (or if they don't exist, three separate "not found" errors appear for each).
3. The order of targets matches the command-line order.

**Part C — --no-color global placement:**
1. The tool is invoked with `wirerust --no-color analyze <pcap>`.
2. The flag placed before the subcommand is honored; output contains no ANSI codes.
3. The tool also accepts `wirerust analyze --no-color <pcap>` (after subcommand).

**Part D — --mitre flag scope:**
1. The tool is invoked with `wirerust analyze --mitre <pcap>` (no --dns, --http, --tls).
2. Protocol analyzers are NOT implied by `--mitre` alone; the run proceeds as a packet-count only run.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.001 | Invariant 1: at least one target required | Part A: zero-target invocation fails |
| BC-2.12.006 | Postcondition 1: multiple targets in order | Part B: three targets preserved in command-line order |
| BC-2.12.003 | Postcondition 1: --no-color works in both placements | Part C: global flag semantics |
| BC-2.12.001 | Invariant 3: --mitre does not imply analyzers | Part D: mitre flag scope is presentation-only |

## Verification Approach

**Part A:**
Run `wirerust analyze` (no file) in a subprocess. Assert exit code != 0. Assert stderr
contains clap error text mentioning a missing argument.

**Part B:**
Create three temporary pcap files (or use known-good ones). Run `wirerust analyze a.pcap b.pcap c.pcap`.
Observe the output or errors for all three files being referenced in order.

**Part C:**
Run `wirerust --no-color analyze <test.pcap>` and `wirerust analyze --no-color <test.pcap>`.
In both cases, scan stdout for `\x1b[` and assert it is absent.

**Part D:**
Run `wirerust analyze --mitre <test.pcap>` against a pcap known to have HTTP traffic.
Assert: no HTTP analyzer output appears (no HTTP section in the report), since `--http` was not given.
Assert: the run completes without error.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Each sub-part behaves as specified.
- **Edge case handling** (weight: 0.2): Global flag placement works in both positions; mitre scope is correctly limited.
- **Error quality** (weight: 0.2): Zero-target error is informative; exit code is non-zero.
- **Performance** (weight: 0.05): Immediate parse-time rejection for zero-target; no blocking.
- **Data integrity** (weight: 0.05): Multiple targets processed in declared order.

## Edge Conditions

- Duplicate targets (`a.pcap a.pcap`): both are processed (no deduplication at parse time).
- `--hosts` flag on `analyze` subcommand: clap rejects it with a flag-not-found error.
- `wirerust analyze` with no subcommand: shows help, exits non-zero.

## Failure Guidance

"HOLDOUT LOW: HS-084 (satisfaction: 0.XX) -- The CLI did not enforce required targets (accepted zero-target invocation silently), --no-color was not honored in global position, or --mitre incorrectly implied protocol analyzers."
