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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.006.md
input-hash: "bfce575"
traces_to: .factory/stories/STORY-076.md
id: "HS-078"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.006
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Skipped-Packets Warning Appears Iff Decode Errors Were Encountered

## Scenario

The terminal reporter must conditionally display a "Skipped:" line only when decode errors
actually occurred — not on every run. A clean pcap run should produce no mention of skipped
packets.

**Part A — clean run:**
1. The tool is run against a well-formed pcap file with no decode errors.
2. The resulting terminal output does NOT contain the string "Skipped:" anywhere.
3. The output still contains the normal report header and any findings.

**Part B — decode-error run:**
1. The tool is run against a pcap that triggers at least five decode errors (e.g., a pcap
   with truncated or malformed packets).
2. The resulting terminal output contains a line with "Skipped:" followed by the count
   of failed packets.
3. The count in the "Skipped:" line matches the actual number of decode errors.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.006 | Postcondition 2: No "Skipped:" line when skipped_packets = 0 | Part A: clean pcap must not produce the line |
| BC-2.11.006 | Postcondition 1: "Skipped: N packets (decode errors)" present when N > 0 | Part B: error pcap must produce the line with count |

## Verification Approach

**Part A:**
Run `wirerust analyze <clean.pcap>` and capture stdout.
Assert: `stdout` does not contain `"Skipped:"`.

**Part B:**
Run `wirerust analyze <malformed.pcap>` and capture stdout.
Assert: `stdout` contains `"Skipped:"`.
Assert: the number after "Skipped:" is a positive integer greater than zero.
Assert: the line includes the text `"decode errors"` or similar contextual phrasing.

A synthetic malformed pcap can be constructed by truncating packet data below the declared
capture length. Alternatively, test at the reporter layer by constructing a Summary with
`skipped_packets = 5` and asserting the output contains `"Skipped: 5"`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): The "Skipped:" line appears exactly when skipped_packets > 0 and is absent when skipped_packets = 0.
- **Edge case handling** (weight: 0.2): Zero vs. nonzero is the key boundary; skipped_packets = 1 should still show the line.
- **Error quality** (weight: 0.15): The message is human-readable and includes the count.
- **Performance** (weight: 0.05): Completes in reasonable time.
- **Data integrity** (weight: 0.1): The rest of the report output is unaffected by the presence or absence of the skipped line.

## Edge Conditions

- `skipped_packets = 0`: absolutely no "Skipped:" output — not even a "0 packets skipped" line.
- `skipped_packets = 1`: the singular case; the line should still appear.
- `skipped_packets = u64::MAX` (theoretical maximum): the line should appear with the large number; no overflow or panic.

## Failure Guidance

"HOLDOUT LOW: HS-078 (satisfaction: 0.XX) -- The skipped-packets line either appeared in a clean run (false positive) or was absent in an error run (false negative), breaking the conditional display contract."
