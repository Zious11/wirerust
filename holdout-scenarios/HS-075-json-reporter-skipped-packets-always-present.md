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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.001.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.002.md
input-hash: "bfce575"
traces_to: .factory/stories/STORY-076.md
id: "HS-075"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.001
  - BC-2.11.002
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: JSON Reporter Includes skipped_packets Key Even When Zero and Output Is Parseable by jq

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. An analyst runs wirerust with `--output-format json` on a small pcap containing only a few clean HTTP requests — no malformed packets, no skipped packets.
2. The resulting JSON output is piped through `jq .summary.skipped_packets` by a downstream script.
3. The jq command must return `0` (the integer zero) — not `null`, not an error, and not nothing (empty output).
4. Additionally, the same downstream script queries `jq '.findings | length'` to get the finding count, and `jq '.analyzers | map(.analyzer_name) | sort'` to get a sorted list of analyzer names. All three queries must succeed without error.
5. The analyst also verifies that the JSON can be processed by `python3 -m json.tool` without error — confirming it is syntactically valid RFC 8259 JSON.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | postcondition 2-6; invariant 3 | Exactly 3 top-level keys; findings is array; analyzers is array; pretty-printed; stable schema |
| BC-2.11.002 | postcondition 2; invariant 1 | skipped_packets key always present in summary object, even when value is 0 |

## Verification Approach

Run wirerust on a minimal but valid pcap (e.g., a single HTTP request-response pair). Apply JSON output format.

1. Run: `wirerust analyze --output-format json <pcap> | jq .summary.skipped_packets`
   Assert output is `0` (integer zero, not null, not empty).
2. Run: `wirerust analyze --output-format json <pcap> | jq '.findings | length'`
   Assert output is a non-negative integer.
3. Run: `wirerust analyze --output-format json <pcap> | python3 -m json.tool > /dev/null`
   Assert exit code is 0 (valid JSON).
4. Run: `wirerust analyze --output-format json <pcap> | jq 'keys'`
   Assert output is `["analyzers","findings","summary"]` (exactly 3 keys).
5. Assert output is pretty-printed (contains newlines and indentation, not a single line).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): skipped_packets present as 0 integer; exactly 3 top-level keys; JSON valid per RFC 8259.
- **Edge case handling** (weight: 0.25): jq and python3 json.tool both process the output without error; no extra or missing top-level keys.
- **Error quality** (weight: 0.15): Pretty-printed output (indented, one key per line) for human readability.
- **Data integrity** (weight: 0.1): findings is an array (even when empty); analyzers is an array (even when empty).

## Edge Conditions

- `skipped_packets` must be a JSON number (0), not the JSON string `"0"`, and not absent.
- The summary object must contain exactly the keys specified in BC-2.11.001 postcondition 3: `total_packets`, `total_bytes`, `skipped_packets`, `unique_hosts`, `protocols`, `services`.
- A strict JSON parser (e.g., jq) must not produce any warnings about non-conformant input.

## Failure Guidance

"HOLDOUT LOW: HS-075 (satisfaction: 0.XX) -- JSON reporter output was malformed; skipped_packets was absent or null when it should be 0, or the output was not valid JSON per RFC 8259; check that serde_json::to_string_pretty is used and skipped_packets has no skip_serializing_if guard."
