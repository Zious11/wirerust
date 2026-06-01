---
document_type: holdout-scenario
level: ops
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-069.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.002.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.003.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.004.md
input-hash: "5cabe5c"
traces_to: .factory/specs/prd.md
id: "HS-006"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-7"
behavioral_contracts:
  - BC-2.09.002
  - BC-2.09.003
  - BC-2.09.004
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Finding One-Liner Format — All Verdict and Confidence Combinations

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

> **Rubric note (v1.1 — Phase-4 adjudication):** The em-dash separator requirement has been
> removed from the terminal output check. BC-2.09.002 governs `Finding::fmt::Display`
> (src/findings.rs), which uses em-dash U+2014 and already complies. The terminal reporter
> (src/reporter/terminal.rs) is a separate layer (ADR-0003) and has no BC-mandated separator
> character. The terminal reporter's ASCII hyphen `-` is **not a defect**: piped CLI output
> convention (clig.dev, Snort/Suricata/Zeek practice) favors ASCII hyphen for grep/cut/awk
> compatibility. The legitimate checks are: bracket format, uppercase verdict token, uppercase
> confidence token. See holdout-finding-triage-2026-06-01.md §Finding 1 for full analysis.

## Scenario

1. A user runs wirerust analyze on a capture that produces findings with diverse
   verdict and confidence combinations: Likely/High, Likely/Medium, Likely/Low,
   Inconclusive/Medium, Inconclusive/Low, and Unlikely/Low.
2. The terminal output shows each finding as a one-liner with brackets around the category,
   uppercase verdict token, and uppercase confidence in parentheses followed by the summary.
3. The format is consistent across all findings — no finding uses lowercase tokens or
   omits the category bracket.
4. A forensic analyst reading the output can immediately distinguish verdict levels by
   the token (LIKELY vs. INCONCLUSIVE vs. UNLIKELY) without consulting documentation.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.09.002 | Postcondition 1 — Display format is "[Category] VERDICT (CONFIDENCE) — summary" | Step 2: BC-2.09.002 governs Finding::Display (findings.rs) — already em-dash compliant. The terminal reporter rendering path is a separate layer; separator is not BC-mandated for terminal output. |
| BC-2.09.003 | Postcondition 1-3 — Likely/Unlikely/Inconclusive render as uppercase | Step 3: no lowercase verdict tokens appear |
| BC-2.09.004 | Postcondition 1-3 — High/Medium/Low render as uppercase | Step 3: no lowercase confidence tokens appear |

## Verification Approach

Run wirerust on a pcap that exercises multiple analyzers to produce diverse findings:

```
wirerust analyze mixed_findings.pcap
```

Inspect terminal output. For each finding line, verify:
- Starts with `[` followed by a category name, then `]`
- Verdict is one of: `LIKELY`, `UNLIKELY`, `INCONCLUSIVE` (exact uppercase)
- Confidence is one of: `HIGH`, `MEDIUM`, `LOW` (exact uppercase, in parentheses)

NOTE: The separator character between the confidence token and the summary is NOT
checked here. The terminal reporter is not BC-bound to a specific separator. ASCII
hyphen (`-`) and em-dash (`—`) are both acceptable for terminal output. Only
`Finding::Display` (used for logging/debug, not terminal rendering) is bound to
em-dash by BC-2.09.002.

If no suitable multi-finding pcap is available, check whether the Finding Display
implementation correctly handles all three Verdict variants and all three Confidence
variants by examining test output.

## Evaluation Rubric

- **Functional correctness** (weight: 0.6): All six verdict/confidence combinations render
  in the correct format — brackets present, verdict uppercase, confidence uppercase in
  parentheses.
- **Data integrity** (weight: 0.2): Uppercase is strict (no "Likely" mixed-case in output).
  Separator character is NOT checked (not BC-mandated for terminal output).
- **Edge case handling** (weight: 0.1): A finding with an empty summary still renders
  without panicking.
- **Error quality** (weight: 0.1): N/A for this scenario.

## Edge Conditions

- A finding with a summary string that itself contains `—` (em-dash) should still render
  without panicking or corrupting the output structure.
- All three Verdict variants should appear in some realistic capture to confirm all code
  paths execute.
- The display format must be stable across different terminal widths — no line-wrapping
  logic should alter token casing.

## Failure Guidance

"HOLDOUT LOW: HS-006 (satisfaction: 0.XX) — finding display format uses wrong casing or
omits required bracket/token fields in terminal output."
