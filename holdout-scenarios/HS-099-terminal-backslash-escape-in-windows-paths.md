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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.008.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.010.md
input-hash: "d2026ba"
traces_to: .factory/stories/STORY-076.md
id: "HS-099"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.007
  - BC-2.11.008
  - BC-2.11.010
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Backslash in Finding Summary Is Escaped to Double-Backslash in Terminal Output

## Scenario

A backslash (0x5C) in a finding's summary or evidence — which can appear in Windows-style
file paths or escape sequences in HTTP payloads — must be escaped to `\\` in the terminal
output. A raw single backslash must not pass through.

This matters because raw backslashes can confuse terminal display and change the meaning of
subsequent characters (e.g., `\n`, `\t`). The escape function must handle this.

1. A finding with `summary = "HTTP request to \\server\\share\\file"` (a UNC path with
   backslashes) is rendered by TerminalReporter.
2. Each single backslash in the original summary is escaped to `\\` (double backslash).
3. The output contains `"\\\\server\\\\share\\\\file"` (each original `\` becomes `\\`).

Additionally:
4. A finding with `summary = "normal ASCII text: abc 123 !@#$%^&*()"` (no backslash,
   no control bytes) passes through unchanged.
5. A finding with `evidence = ["path: C:\\Users\\attacker"]` has the backslashes escaped
   in the evidence cell too.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.007 | Postcondition 4: backslash escaped to `\\` | Items 1-3: backslash in UNC path |
| BC-2.11.008 | Postcondition 1: printable ASCII preserved | Item 4: clean summary unchanged |
| BC-2.11.010 | Postcondition 2: each evidence line escaped | Item 5: backslash in evidence |

## Verification Approach

Directly invoke `TerminalReporter::render` with:

**Test 1 — backslash in summary:**
Finding with `summary = "path: \\server\\share"` (two backslashes in original).
Expected in output: `"path: \\\\server\\\\share"` (each `\` doubled).
Assert: raw `\x5c\x73` (backslash-s, which would be ambiguous) does NOT appear as-is.
Assert: `\\` appears exactly twice more than in the input (four total in output vs two in input).

**Test 2 — clean ASCII:**
Finding with `summary = "abc !@#$%^&*()"`.
Assert: the summary appears in output character-for-character unchanged.

**Test 3 — backslash in evidence:**
Finding with `evidence = ["C:\\Users\\victim"]`.
Assert: the rendered evidence line shows `"C:\\\\Users\\\\victim"`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Each backslash becomes `\\`; clean ASCII passes through.
- **Edge case handling** (weight: 0.2): Backslash escaping in evidence lines (not just summary).
- **Error quality** (weight: 0.1): No panic on payloads with many backslashes.
- **Performance** (weight: 0.05): Linear time processing.
- **Data integrity** (weight: 0.15): Non-backslash printable ASCII (including `!@#$%^&*()`) is entirely unchanged.

## Edge Conditions

- A summary that is only backslashes: `"\\"` → `"\\\\"`.
- An empty summary: no crash, no output.
- Backslash immediately followed by `n`: produces `\\n` in output, not a literal newline (the backslash is escaped first, then the `n` is printable ASCII — result is two characters, not a newline).
- Backslash immediately followed by `x1b` (ESC): both are escaped independently (`\\` for the backslash and `\u{1b}` for the ESC — they remain adjacent in the output).

## Failure Guidance

"HOLDOUT LOW: HS-099 (satisfaction: 0.XX) -- Backslash characters in finding summaries or evidence were not doubled to `\\` in the terminal output, allowing potentially ambiguous escape sequences to appear in the terminal."
