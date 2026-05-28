---
document_type: holdout-scenario
level: ops
version: "1.0"
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

## Scenario

1. A user runs wirerust analyze on a capture that produces findings with diverse
   verdict and confidence combinations: Likely/High, Likely/Medium, Likely/Low,
   Inconclusive/Medium, Inconclusive/Low, and Unlikely/Low.
2. The terminal output shows each finding as a one-liner with brackets around the category,
   uppercase verdict token, uppercase confidence in parentheses, and the summary text after
   an em-dash separator.
3. The format is consistent across all findings — no finding uses lowercase tokens or
   omits the category bracket.
4. A forensic analyst reading the output can immediately distinguish verdict levels by
   the token (LIKELY vs. INCONCLUSIVE vs. UNLIKELY) without consulting documentation.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.09.002 | Postcondition 1 — Display format is "[Category] VERDICT (CONFIDENCE) — summary" | Step 2: exact format of the one-liner |
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
- Separator between confidence and summary is an em-dash (`—` U+2014), not a hyphen

If no suitable multi-finding pcap is available, check whether the Finding Display
implementation correctly handles all three Verdict variants and all three Confidence
variants by examining test output.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): All six verdict/confidence combinations render
  in the correct format without exception.
- **Data integrity** (weight: 0.3): Em-dash separator is used, not ASCII hyphen; uppercase
  is strict (no "Likely" mixed-case in output).
- **Edge case handling** (weight: 0.1): A finding with an empty summary still renders
  without panicking.
- **Error quality** (weight: 0.1): N/A for this scenario.

## Edge Conditions

- A finding with a summary string that itself contains `—` (em-dash) should still render
  the separator correctly.
- All three Verdict variants should appear in some realistic capture to confirm all code
  paths execute.
- The display format must be stable across different terminal widths — no line-wrapping
  logic should alter token casing.

## Failure Guidance

"HOLDOUT LOW: HS-006 (satisfaction: 0.XX) — finding display format uses wrong casing,
wrong separator, or omits required fields in terminal output."
