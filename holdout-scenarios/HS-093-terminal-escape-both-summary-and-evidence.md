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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.007.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.011.md
input-hash: "d2026ba"
traces_to: .factory/stories/STORY-076.md
id: "HS-093"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.010
  - BC-2.11.011
  - BC-2.11.007
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Escape Applied Independently to Summary, Each Evidence Line, and Analyzer Detail Values

## Scenario

The escape function must be called on THREE separate data locations: the finding summary,
each individual evidence line, and each analyzer summary detail value. A single call site
applying escape only to the summary would miss injection vectors in evidence and analyzer details.

1. A finding is constructed where:
   - `summary = "normal summary"` (clean)
   - `evidence = ["clean evidence", "evil\x1b[1;31m red-text injection", "clean after"]`
   - An analyzer summary detail has a value containing U+009B (CSI).

2. The terminal reporter renders this finding.

3. The summary `"normal summary"` passes through unchanged.

4. The evidence item `"clean evidence"` passes through unchanged.

5. The evidence item containing ESC (0x1B) is escaped: the raw ESC byte does NOT appear;
   `\u{1b}` appears instead.

6. The evidence item `"clean after"` passes through unchanged.

7. The analyzer summary detail value containing U+009B appears as `\u{9b}` in the output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.010 | Postcondition 1: summary is escaped | Point 3 — summary passes clean |
| BC-2.11.010 | Postcondition 2: EACH evidence entry escaped independently | Points 4-6 — per-entry escaping |
| BC-2.11.011 | Postcondition 1: analyzer detail values escaped | Point 7 — detail value escaping |
| BC-2.11.007 | Postcondition 1: ESC byte escaped | Point 5 — ESC byte in evidence |

## Verification Approach

Directly invoke `TerminalReporter::render` with:
- A `Finding` whose `summary` is clean and whose `evidence` has three items (one with ESC byte).
- An `AnalysisSummary` whose `detail` map has one entry with a U+009B value.

Scan the resulting output string:
1. Assert raw `\x1b` (ESC, 0x1B) does NOT appear anywhere.
2. Assert `\u{1b}` DOES appear (the escaped form).
3. Assert `"clean evidence"` appears unchanged.
4. Assert `"clean after"` appears unchanged.
5. Assert raw `\xc2\x9b` (UTF-8 for U+009B) does NOT appear.
6. Assert `\u{9b}` DOES appear (the escaped form of the CSI codepoint).
7. Assert `"normal summary"` appears unchanged.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): All three injection vectors are independently escaped; clean content passes through.
- **Edge case handling** (weight: 0.2): Each evidence item is checked independently; a control byte at item index 1 does not affect items at index 0 or 2.
- **Error quality** (weight: 0.1): No panic; the output is a valid UTF-8 string after escaping.
- **Performance** (weight: 0.05): Escaping operates in linear time.
- **Data integrity** (weight: 0.15): The escape function is applied at exactly three call sites (summary, per-evidence, per-detail); no call site is missing.

## Edge Conditions

- A finding with an empty evidence Vec: no crash; no evidence output lines.
- A finding with 100 evidence items, one of which has an ESC byte: only that one item is affected.
- An analyzer summary with an empty detail map: no crash; detail section absent or empty.
- Multiple control bytes in a single evidence item: all are escaped (the function iterates over characters).

## Failure Guidance

"HOLDOUT LOW: HS-093 (satisfaction: 0.XX) -- Escape was not applied to all three locations: one or more evidence lines, or the analyzer detail values, passed raw control bytes through to the terminal output."
