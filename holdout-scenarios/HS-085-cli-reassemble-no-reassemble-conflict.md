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
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.004.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.005.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.007.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-086.md
id: "HS-085"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.007
  - BC-2.12.004
  - BC-2.12.005
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: --reassemble and --no-reassemble Together Are Rejected; Output Format Flags Work Independently

## Scenario

Two contradictory reassembly flags must be rejected at parse time — before any pcap analysis
begins. This ensures that ambiguous configuration is never silently resolved.

**Part A — flag conflict:**
1. The tool is invoked with `wirerust --reassemble --no-reassemble analyze test.pcap`.
2. The tool exits immediately with a non-zero exit code.
3. The error output mentions conflicting arguments or mutually exclusive flags.
4. No analysis is performed; no pcap is opened.
5. The same applies to `wirerust --no-reassemble --reassemble analyze test.pcap` (order reversal).

**Part B — output format flags:**
1. `wirerust --output-format json analyze <pcap>` produces valid JSON output (not terminal).
2. `wirerust --output-format csv analyze <pcap>` produces CSV output starting with the header row.
3. `wirerust --output-format invalid_value analyze <pcap>` is rejected at parse time.
4. When no format flag is given, output is in terminal (human-readable) format.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.007 | Postcondition 1: ArgumentConflict error when both flags given | Part A: both orderings fail |
| BC-2.12.007 | Invariant 1: conflict is symmetric | Part A: reversed order also fails |
| BC-2.12.004 | Postcondition 1-4: json/csv/invalid format values | Part B: format selection |
| BC-2.12.005 | Postcondition 5: threshold flags default None | Context for Part B: defaults not affected |

## Verification Approach

**Part A:**
Run `wirerust --reassemble --no-reassemble analyze test.pcap` in a subprocess.
Assert exit code != 0.
Assert stderr mentions conflict or mutual exclusion.
Also test reversed order: `wirerust --no-reassemble --reassemble analyze test.pcap`.
Assert same non-zero exit.

**Part B:**
Run each format variant:
- `--output-format json`: capture stdout; assert it parses as valid JSON; assert it has a "findings" key.
- `--output-format csv`: capture stdout; assert first line matches the 9-column CSV header.
- `--output-format xml`: assert non-zero exit code and stderr error.
- No flag: assert output contains "WIRERUST TRIAGE REPORT" (terminal header).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Both parts behave exactly as specified.
- **Edge case handling** (weight: 0.2): Symmetric conflict rejection; both orderings fail.
- **Error quality** (weight: 0.2): Conflict error is informative; invalid format value produces a helpful error.
- **Performance** (weight: 0.05): Parse-time rejection is immediate.
- **Data integrity** (weight: 0.05): JSON output is valid; CSV header is correct.

## Edge Conditions

- `--reassemble` alone: succeeds and activates reassembly.
- `--no-reassemble` alone: succeeds and deactivates reassembly.
- Neither flag: reassembly may or may not activate depending on which analyzers are selected.
- `--output-format` with both `--json` flag: JSON flag takes precedence (higher priority).

## Failure Guidance

"HOLDOUT LOW: HS-085 (satisfaction: 0.XX) -- The conflicting --reassemble/--no-reassemble combination was not rejected at parse time, or an invalid --output-format value was silently ignored instead of producing a parse error."
