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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.003.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.005.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-076.md
id: "HS-073"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.003
  - BC-2.11.005
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: JSON Reporter Treats C0 and C1 Bytes Differently in the Same Finding

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A TLS analyzer emits a finding for a ClientHello whose SNI contains both a C0 control byte (ESC, 0x1B) and a C1 control byte (U+009B, encoded as UTF-8 bytes 0xC2 0x9B). The finding summary is a string containing both characters.
2. The analyst runs wirerust with `--output-format json` on a pcap containing this ClientHello.
3. The JSON output's `findings` array contains the anomaly finding.
4. In the raw JSON bytes of the finding's `summary` field:
   - The ESC byte (0x1B, a C0 byte) appears as the six-character escape sequence `` (backslash-u-0-0-1-b), conforming to RFC 8259.
   - The C1 codepoint U+009B appears as the raw two-byte UTF-8 sequence 0xC2 0x9B — it is NOT escaped to ``.
5. A developer who reads the JSON with a strict UTF-8-aware tool sees the C1 byte preserved as raw UTF-8; a developer who processes the JSON with a standard JSON parser that decodes `\uNNNN` sequences will correctly decode the ESC.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.003 | postcondition 1-2, 4 | C0 bytes (0x00-0x1F) are escaped as per RFC 8259; DEL (0x7F) is NOT escaped |
| BC-2.11.005 | postcondition 1; invariant 2 | C1 codepoints (U+0080-U+009F) pass through as raw UTF-8 bytes; contrast with C0 escaping |

## Verification Approach

Run wirerust on a pcap with the described SNI. Inspect the raw bytes of the JSON output file.

1. Assert the `findings` array contains the SNI anomaly finding.
2. Locate the `summary` field in the raw JSON bytes. Assert that the bytes `5c 75 30 30 31 62` (representing ``) appear in the summary (ESC escaped).
3. Assert that the bytes `c2 9b` (raw UTF-8 encoding of U+009B) appear in the summary (C1 NOT escaped).
4. Assert `json.parse()` succeeds with a standard JSON parser (the output is valid JSON).
5. Assert that after JSON parsing, the decoded `summary` string contains the original ESC byte (0x1B) and the original U+009B codepoint.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): ESC escaped as ``; U+009B passed through as raw 0xC2 0x9B bytes.
- **Edge case handling** (weight: 0.3): Same string contains both types of control characters; they must be treated differently based on the RFC 8259 rules for C0 vs. non-C0.
- **Error quality** (weight: 0.1): The JSON remains valid and parseable despite the mix.
- **Data integrity** (weight: 0.1): Round-trip through JSON parser recovers original bytes for both character types.

## Edge Conditions

- serde_json escapes exactly the C0 range (0x00-0x1F) per RFC 8259. DEL (0x7F) and C1 (U+0080-U+009F as UTF-8) are not in this range and pass through.
- The C1 bytes 0xC2 0x9B are a valid two-byte UTF-8 encoding. serde_json preserves valid UTF-8 bytes as-is for codepoints outside the C0 range.
- This asymmetry is intentional and matches the RFC 8259 requirement.

## Failure Guidance

"HOLDOUT LOW: HS-073 (satisfaction: 0.XX) -- JSON reporter incorrectly treated C0 and C1 bytes the same way; ESC must be `` while U+009B must be raw 0xC2 0x9B bytes; check that serde_json::to_string_pretty is used without custom serialization."
