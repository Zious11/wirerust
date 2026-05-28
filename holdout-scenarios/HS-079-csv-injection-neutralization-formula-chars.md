---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-076.md
  - .factory/stories/STORY-077.md
  - .factory/stories/STORY-078.md
  - .factory/stories/STORY-079.md
  - .factory/stories/STORY-080.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.020.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.021.md
input-hash: "bfce575"
traces_to: .factory/stories/STORY-076.md
id: "HS-079"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.021
  - BC-2.11.020
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: CSV Output Neutralizes Formula-Injection Characters in Every Column

## Scenario

An attacker injects formula-trigger characters into packet payloads that surface as finding
fields. When the CSV reporter writes these findings, no cell should start with an unguarded
`=`, `+`, `-`, or `@` that a spreadsheet application would interpret as a formula.

1. A set of findings is prepared where:
   - The `summary` field starts with `=HYPERLINK("http://evil.example","Click here")`
   - The `category` field starts with `+1`
   - The `evidence` first element starts with `-1` (a formula that subtracts)
2. The tool is invoked with CSV output format: `wirerust analyze <crafted.pcap> --csv`
3. The CSV output is parsed as RFC 4180.
4. Every cell that originally started with a trigger character now starts with `'` (single quote).
5. The rest of the cell value after the `'` prefix is the original value unchanged.
6. The column count in every row remains exactly 9.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.021 | Postcondition 1: trigger chars get `'` prefix | =, +, - in cells must become '=, '+, '- |
| BC-2.11.021 | Invariant 1: applied to ALL nine columns without exception | All columns are checked, not just summary |
| BC-2.11.020 | Invariant 1: column count always exactly 9 | Neutralization must not change column count |

## Verification Approach

Option 1 (integration): Craft a pcap that produces findings with formula-trigger summaries
(e.g., an HTTP request with path `=HYPERLINK(...)`). Run `wirerust analyze <pcap> --csv`,
capture stdout, parse as CSV, and assert:
- Every cell starting with `=`, `+`, `-`, `@`, TAB, or CR has been prefixed with `'`.
- No cell starting with those characters exists in ANY column.

Option 2 (unit-level): Directly invoke `CsvReporter::render` with a `Finding` whose
`summary = "=cmd"`, `category = "+1"`. Parse the output CSV and assert prefix behavior.

Additional checks:
- A cell containing `"a=cmd"` (trigger at position 2) must NOT be prefixed — position-1-only rule.
- A cell that is an empty string must remain empty (no prefix added).
- A cell starting with `'` already must remain unchanged.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): All six trigger characters are neutralized in every column.
- **Edge case handling** (weight: 0.25): Position-2+ trigger chars are not affected; empty strings are not affected.
- **Error quality** (weight: 0.1): Valid RFC 4180 output is produced; no malformed CSV.
- **Performance** (weight: 0.05): Completes within normal run time.
- **Data integrity** (weight: 0.1): Column count is 9 in every row after neutralization.

## Edge Conditions

- TAB (U+0009) and CR (U+000D) as the first byte of a cell are also trigger characters.
- A `summary` that is empty should produce an empty cell, not a `'` prefix.
- A `summary` starting with a single-quote `'` is NOT a trigger and must not receive a double-prefix.
- After neutralization, the csv crate applies RFC 4180 quoting; the `'` prefix must survive as literal content.

## Failure Guidance

"HOLDOUT LOW: HS-079 (satisfaction: 0.XX) -- CSV output contained unguarded formula-trigger characters (=, +, -, @, TAB, CR) at the start of one or more cells, leaving the output vulnerable to CSV injection in spreadsheet applications."
