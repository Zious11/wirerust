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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.013.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.014.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.015.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.016.md
input-hash: "d2026ba"
traces_to: .factory/stories/STORY-076.md
id: "HS-081"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.013
  - BC-2.11.014
  - BC-2.11.015
  - BC-2.11.016
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: MITRE Grouping Presents Tactics in Kill-Chain Order with Correct Sorting

## Scenario

When MITRE grouping mode is active, findings must be displayed grouped by attack phase in the
canonical MITRE ATT&CK kill-chain progression — not sorted alphabetically or by insertion order.

1. A set of findings is constructed spanning three MITRE tactics: one in Command and Control (T1071),
   one in Defense Evasion (T1036), and one with no technique (uncategorized). Multiple findings
   exist within the Defense Evasion tactic with different verdicts and confidence levels.
2. The tool is invoked with the MITRE grouping flag active.
3. The output shows tactic section headers.
4. Defense Evasion appears before Command and Control in the output (kill-chain ordering, not
   alphabetical: Defense Evasion is mid-chain; Command and Control is later).
5. Within the Defense Evasion section, findings are sorted: Likely/High verdict appears before
   Inconclusive/Medium.
6. The "Uncategorized" section appears last, after all named tactic sections.
7. The MITRE technique line for a known ID (T1036) includes the technique name separated by
   an em-dash character (U+2014), not an ASCII double-hyphen.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.013 | Postcondition 2: tactic headers in canonical order | Defense Evasion before Command and Control |
| BC-2.11.013 | Postcondition 4: Uncategorized always last | No-technique finding appears in last section |
| BC-2.11.014 | Postcondition 1: sort by verdict within bucket | Likely before Inconclusive in same tactic |
| BC-2.11.014 | Postcondition 2: sort by confidence within same verdict | High before Medium for same verdict |
| BC-2.11.015 | Postcondition 1: None technique maps to Uncategorized | Finding with no technique in last section |
| BC-2.11.016 | Postcondition 1: em-dash separates ID from name | T1036 — Masquerading format |

## Verification Approach

Invoke the tool with a set of findings spanning the described tactics. In the terminal output:

1. Find the positions of each tactic section header using line scanning.
2. Assert: Defense Evasion header position < Command and Control header position.
3. Assert: Uncategorized section is the final section (nothing follows its header until EOF).
4. Within the Defense Evasion section lines, assert that the Likely/High finding's line
   precedes the Inconclusive/Medium finding's line.
5. Assert the MITRE line for T1036 contains the Unicode em-dash `\u{2014}` (U+2014), not `--`.
6. Assert the no-technique finding appears under the Uncategorized header.

At the unit level, construct the findings programmatically and call `TerminalReporter::render`
with `show_mitre_grouping = true`, then scan the resulting string.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Tactic section order matches kill-chain; Uncategorized is last; sort within sections is correct.
- **Edge case handling** (weight: 0.2): Unknown technique IDs land in Uncategorized; findings with None technique land in Uncategorized.
- **Error quality** (weight: 0.1): No crash on edge cases; unknown IDs produce "(unknown)" label.
- **Performance** (weight: 0.1): Grouping logic completes in O(n log n) time; no unexpected slowness.
- **Data integrity** (weight: 0.15): Em-dash is the exact U+2014 codepoint; no ASCII substitution.

## Edge Conditions

- All findings in one tactic: only that tactic section plus possibly Uncategorized.
- Unknown technique ID "T9999": appears under Uncategorized with "(unknown)" label.
- When all findings have None technique: only an Uncategorized section is present.
- When `show_mitre_grouping = false` (default mode): no tactic headers, and the technique
  line reads `MITRE: T1036` with no em-dash and no name.

## Failure Guidance

"HOLDOUT LOW: HS-081 (satisfaction: 0.XX) -- MITRE grouping produced tactics in incorrect order (alphabetical or insertion order instead of kill-chain order), Uncategorized was not last, or sorting within a tactic bucket was wrong."
