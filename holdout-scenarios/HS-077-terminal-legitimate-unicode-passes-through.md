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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.008.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.009.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-076.md
id: "HS-077"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.008
  - BC-2.11.009
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Legitimate Unicode (Cyrillic, Emoji, NBSP) Survives Terminal Output Unchanged

## Scenario

A finding contains a summary with a mix of: printable ASCII, Cyrillic characters,
emoji, and a Non-Breaking Space (U+00A0 — the first codepoint ABOVE the C1 escape range).
The tool must display all of these unchanged while still escaping control bytes.

1. A finding is prepared with summary:
   `"Suspicious host: сервер.ru cafe\u{2615} path\u{00a0}ok"` (Cyrillic, coffee-cup emoji,
   Non-Breaking Space).
2. The terminal reporter renders this finding.
3. The Cyrillic characters `сервер.ru` appear unchanged in the output.
4. The coffee-cup emoji U+2615 appears unchanged.
5. The Non-Breaking Space (U+00A0) appears unchanged — it is NOT escaped to `\u{a0}`.
6. The ASCII portions `"Suspicious host: "` and `" path"` and `" ok"` appear unchanged.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.008 | Postcondition 2: non-ASCII UTF-8 at U+00A0 and above passes through | Cyrillic and emoji at U+00A0+ must not be escaped |
| BC-2.11.009 | Postcondition 2: U+00A0 is NOT escaped | The boundary: U+009F escapes, U+00A0 does not |

## Verification Approach

Directly invoke `TerminalReporter::render` with a `Finding` whose summary contains:
- The Cyrillic string `"сервер"` (U+0441 U+0435 U+0440 U+0432 U+0435 U+0440)
- The emoji U+2615 (HOT BEVERAGE)
- U+00A0 (NON-BREAKING SPACE)

Assert the rendered output:
- Contains the exact Cyrillic bytes for `"сервер"`.
- Contains the emoji codepoint U+2615.
- Contains U+00A0 as-is (two-byte UTF-8 sequence `\xc2\xa0`).
- Does NOT contain `\u{a0}` (escaped form of U+00A0) — it must be unescaped.
- Does NOT contain `\u{2615}` or any escape sequence for the emoji.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): All three legitimate Unicode characters appear verbatim in the output.
- **Edge case handling** (weight: 0.3): U+00A0 is specifically confirmed as unescaped; contrast with U+009F being escaped.
- **Error quality** (weight: 0.1): No panic or encoding error.
- **Performance** (weight: 0.05): Completes immediately.
- **Data integrity** (weight: 0.05): ASCII surrounding the Unicode characters is intact.

## Edge Conditions

- U+00A0 is the critical boundary: it must NOT be escaped even though U+009F (one below the range top) IS escaped.
- A finding with ONLY Cyrillic in the summary and no control characters should pass through completely unchanged.
- An empty summary (`""`) should produce no crash and render as an empty line.

## Failure Guidance

"HOLDOUT LOW: HS-077 (satisfaction: 0.XX) -- The terminal reporter over-escaped legitimate Unicode characters (Cyrillic, emoji, or NBSP), making non-ASCII content unreadable in the output."
