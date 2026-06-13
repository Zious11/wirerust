---
document_type: holdout-scenario
level: ops
version: "1.1"
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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.022.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.023.md
input-hash: "bfce575"
traces_to: .factory/stories/STORY-076.md
id: "HS-080"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.020
  - BC-2.11.022
  - BC-2.11.023
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: CSV Output Has Exactly Nine Columns and Correct Header in All Conditions

## Scenario

The CSV reporter must produce a stable, fixed-schema output that downstream parsers can
rely on without schema discovery. This scenario verifies the column count and header
stability under multiple conditions.

1. Run the tool with `--csv` output against a pcap that produces:
   - Zero findings
   - Exactly one finding with all optional fields present
   - Multiple findings where some have empty evidence, some have multi-line evidence
2. For each run, parse the raw CSV output.
3. The first line is always the exact header:
   `category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp`
4. Every data row (if any) has exactly 9 comma-separated fields as parsed by an RFC 4180 parser.
5. A field value containing a comma is double-quoted by the csv library — the column count
   as parsed by RFC 4180 is still 9.
6. Summary and AnalysisSummary inputs are not reflected anywhere in the CSV output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.020 | Postcondition 1: first line is exact header | Header row must be verbatim and first |
| BC-2.11.020 | Invariant 1: column count always 9 | Even with quoted commas, field count is 9 |
| BC-2.11.022 | Postcondition 1: evidence joined with "; " | Multi-evidence appears in one cell |
| BC-2.11.023 | Postcondition 1: one row per finding | Row count = 1 + findings.len() |
| BC-2.11.023 | Postcondition 2/3: summary and analyzer_summaries not in output | No extra rows or columns from these |

## Verification Approach

Run `wirerust analyze <pcap> --csv` and capture stdout.

1. **Zero-finding case:** Run against an empty or non-matching pcap.
   - Assert: exactly 1 line, which is the header.
   - Assert: no data rows.

2. **Multi-finding case:** Run against a pcap with 3 findings, at least one with
   evidence containing 2 items.
   - Assert: line count = 4 (1 header + 3 data rows).
   - Assert: parse each data row with a CSV parser; field count = 9.
   - Assert: evidence column of the multi-evidence finding contains `"; "` as separator.

3. **Comma-in-field case:** Use a finding whose summary contains a comma.
   - Assert: RFC 4180 parser still returns 9 fields for that row.

4. Check the exact header string byte-for-byte against the expected value.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Header is exact; every row has 9 fields; row count = 1 + findings.len().
- **Edge case handling** (weight: 0.25): Comma-in-field is handled; empty findings produce header-only; multi-evidence joins correctly.
- **Error quality** (weight: 0.1): No malformed CSV; output is parseable by standard CSV libraries.
- **Performance** (weight: 0.1): Output produced within normal run time.
- **Data integrity** (weight: 0.1): Summary statistics and analyzer details do not appear as extra rows or columns.

## Edge Conditions

- A finding with an empty evidence Vec produces an empty evidence cell, not a missing field.
- A finding with a single evidence entry produces the entry with no separator.
- A finding with evidence containing `"; "` in an item does not produce extra splits when re-parsed.
- The header line uses comma as delimiter with no spaces around commas.

## Failure Guidance

"HOLDOUT LOW: HS-080 (satisfaction: 0.XX) -- The CSV output had incorrect column count, wrong header, or extra rows from Summary/AnalysisSummary data, breaking downstream parser compatibility."
