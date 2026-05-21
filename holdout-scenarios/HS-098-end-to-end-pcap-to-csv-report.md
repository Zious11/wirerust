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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.022.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.023.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.024.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-076.md
id: "HS-098"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.020
  - BC-2.11.021
  - BC-2.11.022
  - BC-2.11.023
  - BC-2.11.024
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: End-to-End pcap → CSV Output Is Parseable and Injection-Safe (Real-World Corpus)

## Scenario

A SIEM integration engineer imports wirerust CSV output into a data pipeline. The CSV must
be machine-parseable without any special handling, and must be safe to open in a spreadsheet
without formula-injection risk — even when processing a pcap with attacker-controlled content
in the payload.

**Known-good corpus:** Wireshark `http.cap` (see HS-090) — well-maintained, widely used,
no attacker content.

1. Run: `wirerust analyze --all <http.cap> --csv`
2. Exit code is 0.
3. Stdout is valid RFC 4180 CSV parseable by any standard library (e.g., Python's csv module).
4. The first line is exactly the 9-column header.
5. Every data row (if any) has exactly 9 columns as parsed by RFC 4180.
6. No cell in any row starts with `=`, `+`, `-`, `@`, TAB, or CR without a `'` prefix.
7. All optional fields that are None appear as empty strings, not "null" or "N/A".
8. The output can be imported into a spreadsheet without triggering formula execution.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.020 | Postcondition 1, Invariant 1: header and 9 columns | Items 4-5 |
| BC-2.11.021 | Postcondition 1: injection neutralization | Item 6: no unguarded trigger chars |
| BC-2.11.022 | Postcondition 1: evidence join | Multi-evidence in one cell |
| BC-2.11.023 | Postcondition 1: one row per finding | Item 5 |
| BC-2.11.024 | Invariant 4: no sentinel values | Item 7 |

### Real-World Corpus Metadata

| Field | Description |
|-------|-------------|
| corpus_source | Wireshark Sample Captures: https://wiki.wireshark.org/SampleCaptures (http.cap) |
| corpus_size | ~50 KB, ~43 packets |
| known_edge_cases | Standard HTTP traffic; optional fields may be None for some findings |
| false_positive_threshold | N/A (CSV format test, not detection accuracy) |
| false_negative_threshold | N/A |

## Verification Approach

1. Download `http.cap` from Wireshark sample captures.
2. Run: `wirerust analyze --all http.cap --csv`
3. Assert exit code == 0.
4. Parse stdout with a standard CSV library (Python's `csv.reader` or Rust's `csv::Reader`).
5. Assert first row == `["category","verdict","confidence","summary","evidence","mitre_technique","source_ip","direction","timestamp"]`.
6. For each subsequent row, assert `len(row) == 9`.
7. For each cell in each row, assert the cell does not start with `=`, `+`, `-`, or `@` (without a `'` prefix).
8. For each row, check columns 6-9 for optional fields: assert values are either non-empty or exactly `""`.
9. Assert no cell contains the string `"null"`, `"N/A"`, or `"-"`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): CSV is RFC 4180 parseable; header is exact; all rows have 9 columns.
- **Edge case handling** (weight: 0.2): Optional fields are empty strings; injection characters are neutralized.
- **Error quality** (weight: 0.1): No malformed CSV; exit code 0.
- **Performance** (weight: 0.15): Runs in < 30 seconds on http.cap.
- **Data integrity** (weight: 0.15): No sentinel values for None; CSV is machine-importable without formula risk.

## Edge Conditions

- If http.cap produces zero findings, output is header-only; that is valid and acceptable.
- The evidence field for multi-evidence findings must contain `"; "` as separator, not other delimiters.
- Direction values must be CamelCase Debug format, parseable by downstream tools.

## Failure Guidance

"HOLDOUT LOW: HS-098 (satisfaction: 0.XX) -- CSV output from real-world corpus was not RFC 4180 parseable, contained wrong column count, had unguarded formula-injection characters, or used sentinel values for None fields."
