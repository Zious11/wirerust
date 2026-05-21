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
id: "HS-092"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.021
  - BC-2.11.022
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

# Holdout Scenario: CSV Evidence Join Then Injection-Neutralization Combined Edge Case

## Scenario

The order of operations matters: evidence items are first joined into one string with `"; "`
as the separator, and THEN the resulting string passes through injection neutralization. This
means that if the FIRST evidence item starts with `=`, the entire joined string starts with
`=` and receives the `'` prefix — even though the trigger character comes from a single item.

1. A finding has evidence = `["=MALICIOUS_FORMULA", "benign_info"]`.
2. After joining: `"=MALICIOUS_FORMULA; benign_info"`.
3. This joined string starts with `=`, so it is neutralized to `"'=MALICIOUS_FORMULA; benign_info"`.
4. The evidence cell in the CSV contains the neutralized joined string.

Additionally, a second combination:
5. A finding has evidence = `["benign_first", "=formula_in_second_item"]`.
6. After joining: `"benign_first; =formula_in_second_item"`.
7. This joined string starts with `b` (not a trigger), so it is NOT neutralized.
8. The `=` inside position 2+ is not a trigger character.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.022 | Postcondition 4: evidence join THEN neutralize | First item triggers prefix on joined string |
| BC-2.11.021 | Postcondition 1: trigger chars get prefix | `=` at position 0 of joined string is neutralized |
| BC-2.11.021 | Invariant 2: only position 0 inspected | `=` at position 2+ does not trigger prefix |
| BC-2.11.020 | Invariant 1: column count stays 9 | Prefix addition does not change column count |

## Verification Approach

Directly invoke `CsvReporter::render` with two findings:

**Finding A:**
- `evidence = ["=FORMULA", "normal"]`
- Expected CSV evidence cell: `'=FORMULA; normal` (note: `'` prefix added because joined starts with `=`)

**Finding B:**
- `evidence = ["normal_first", "=FORMULA"]`
- Expected CSV evidence cell: `normal_first; =FORMULA` (no prefix — joined string starts with `n`)

Parse the output CSV and verify:
1. Finding A's evidence column contains `'=FORMULA; normal` (with `'` prefix).
2. Finding B's evidence column contains `normal_first; =FORMULA` (without `'` prefix).
3. Both rows have exactly 9 columns.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Both findings produce the correct evidence cell value (with and without prefix).
- **Edge case handling** (weight: 0.3): The "position-only-0-is-inspected" rule is correctly applied after the join, not per-item.
- **Error quality** (weight: 0.1): Valid RFC 4180 CSV output.
- **Performance** (weight: 0.05): Completes immediately.
- **Data integrity** (weight: 0.05): Column count remains 9 in both rows.

## Edge Conditions

- All evidence items trigger-prefixed but only the JOINED string's first character determines the prefix.
- Evidence = `["=a", "=b"]` → joined = `"=a; =b"` → neutralized to `"'=a; =b"` (one prefix, not two).
- Evidence = `[]` (empty) → joined = `""` → no prefix (empty string rule).
- Evidence = `[""]` (one empty item) → joined = `""` → no prefix.

## Failure Guidance

"HOLDOUT LOW: HS-092 (satisfaction: 0.XX) -- CSV injection neutralization was applied per-evidence-item before joining (incorrect order) or was not applied at all after joining, allowing formula characters to reach the CSV cell unguarded."
