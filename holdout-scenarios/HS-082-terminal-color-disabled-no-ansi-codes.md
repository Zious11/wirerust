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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.017.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.018.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.019.md
input-hash: "d2026ba"
traces_to: .factory/stories/STORY-076.md
id: "HS-082"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.018
  - BC-2.11.019
  - BC-2.11.017
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: --no-color Strips All ANSI Escape Codes; Section Order Is Correct

## Scenario

When the analyst passes `--no-color`, the terminal output must contain no ANSI escape
sequences for any verdict or confidence level. The section ordering must also be verified
as correct regardless of color mode.

**Part A — no-color output:**
1. The tool is invoked with `wirerust analyze <pcap> --no-color` where the pcap produces
   findings with Likely/High, Inconclusive/Medium, and Unlikely/Low verdicts.
2. The raw bytes of stdout contain no ESC `[` sequences (ANSI CSI opener).
3. The actual finding text (verdict strings, category, summary) still appears in output.

**Part B — section order:**
1. The tool is invoked against any pcap that produces findings and DNS statistics.
2. The terminal output sections appear in the order: header, PROTOCOLS, optionally SERVICES,
   FINDINGS (only if findings exist), then ANALYZER sections last.
3. If findings is empty, the FINDINGS section is absent entirely.
4. The WIRERUST TRIAGE REPORT header is always first.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.018 | Postcondition 5: no ANSI codes when use_color = false | Raw scan for ESC[ in --no-color output |
| BC-2.11.019 | Postcondition 1: header is first | Part B section ordering |
| BC-2.11.019 | Postcondition 4: FINDINGS absent when empty | Part B absent section |
| BC-2.11.019 | Postcondition 5: ANALYZER sections last | Part B ordering |
| BC-2.11.017 | Postcondition 1: default mode emits bare MITRE ID only | Default mode check |

## Verification Approach

**Part A:**
Run `wirerust analyze <test.pcap> --no-color` and capture raw stdout bytes.
Assert: no byte sequence `\x1b[` (ESC followed by `[`) appears anywhere in the output.
Optionally verify the same output contains recognizable text for the verdict labels
(e.g., "Likely", "Inconclusive") to confirm the output is not simply empty.

**Part B:**
Run `wirerust analyze <pcap-with-findings>` (no --no-color; default color enabled).
Assert: the string "WIRERUST TRIAGE REPORT" appears before any "FINDINGS" or section headers.
Assert: if findings exist, "FINDINGS" section appears somewhere in output.
Run `wirerust analyze <pcap-with-no-findings>`.
Assert: "FINDINGS" is absent from the output.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): No ANSI codes with --no-color; correct section order.
- **Edge case handling** (weight: 0.2): Empty findings causes FINDINGS section to be fully absent.
- **Error quality** (weight: 0.1): Tool exits cleanly in all scenarios.
- **Performance** (weight: 0.1): No slowdown from color-stripping logic.
- **Data integrity** (weight: 0.2): All finding data still appears in --no-color mode; color removal does not strip content.

## Edge Conditions

- `--no-color` placed before the subcommand (global flag): must be honored.
- The `NO_COLOR` environment variable (if set) also disables color; both methods should produce no ANSI codes.
- Empty SERVICES map: the SERVICES section is absent.
- Default mode (no `--mitre`): technique line shows `MITRE: T1036` with no em-dash, no name.

## Failure Guidance

"HOLDOUT LOW: HS-082 (satisfaction: 0.XX) -- ANSI escape codes appeared in --no-color output, or sections appeared in the wrong order (e.g., ANALYZER before FINDINGS, or header not first)."
