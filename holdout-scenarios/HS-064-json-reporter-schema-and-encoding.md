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
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.003.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.004.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.005.md
input-hash: "bfce575"
traces_to: .factory/stories/STORY-076.md
id: "HS-064"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.001
  - BC-2.11.002
  - BC-2.11.003
  - BC-2.11.004
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

# Holdout Scenario: JSON Reporter Output Matches Stable Schema and Encodes Forensic Bytes Correctly

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains mixed HTTP and TLS traffic, including an HTTP request with an ESC byte (0x1B) in a URI, and a TLS ClientHello with a Cyrillic SNI hostname.
2. The analyst runs wirerust with `--output-format json` on this pcap.
3. The output is a pretty-printed JSON document (indented, one key per line).
4. The analyst parses the JSON and verifies:
   - Exactly 3 top-level keys: `"summary"`, `"findings"`, `"analyzers"`.
   - The `"summary"` object contains `"skipped_packets"` set to 0 (the key is present, not absent, even when zero).
   - A finding with ESC in its `summary` field has the ESC byte represented as `` (six characters: backslash, u, 0, 0, 1, b) in the JSON text — NOT as a raw 0x1B byte.
   - A finding with Cyrillic characters in its `summary` field contains the Cyrillic UTF-8 bytes directly readable — NOT as `М` escape sequences.
   - DEL (0x7F) in a finding summary passes through as a raw 0x7F byte (serde_json does not escape 0x7F).
5. The JSON is valid per RFC 8259 and can be parsed by any standard JSON parser without error.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | postcondition 2-6 | Exactly 3 top-level keys; findings array; summary subkeys; pretty-printed |
| BC-2.11.002 | postcondition 2-3; invariant 1 | skipped_packets present even when 0 |
| BC-2.11.003 | postcondition 1-2, 4 | C0 ESC escaped as ; DEL not escaped; round-trip preserves bytes |
| BC-2.11.004 | postcondition 1 | Cyrillic readable in JSON (raw UTF-8, no \u escaping) |
| BC-2.11.005 | postcondition 1; invariant 2 | C1 (U+009B) passes through as raw UTF-8 bytes; distinct from C0 treatment |

## Verification Approach

Run wirerust with JSON output on the mixed HTTP+TLS pcap. Inspect the raw JSON bytes.

1. Parse the JSON and assert `Object.keys(json) == ["analyzers", "findings", "summary"]` (or equivalent).
2. Assert `json.summary.skipped_packets == 0` (key present, value 0).
3. Locate the finding with ESC in summary. Assert the raw JSON bytes at that position are `` (six bytes: 5c 75 30 30 31 62), not 0x1B.
4. Locate the finding with Cyrillic SNI. Assert the raw JSON bytes contain the raw UTF-8 encoding of the Cyrillic characters, not `\u` escape sequences.
5. Assert the JSON is parseable by `jq` or `python3 -m json.tool` without error.
6. Assert output uses pretty-printing (newlines and indentation present).

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Exactly 3 top-level keys; skipped_packets always present; C0 escaped per RFC 8259.
- **Edge case handling** (weight: 0.3): Cyrillic appears as readable UTF-8 (not \u-escaped); DEL passes through raw; C1 passes through raw.
- **Error quality** (weight: 0.2): JSON is valid and parseable by external tools; pretty-printed for human readability.
- **Data integrity** (weight: 0.1): Round-trip C0 byte: serialize then deserialize recovers original byte value.

## Edge Conditions

- serde_json escapes C0 (0x00-0x1F) via `\uNNNN` but does NOT escape C1 (0x80-0x9F as UTF-8 multi-byte) or DEL (0x7F) — these pass through.
- `escape_for_terminal` must NOT be called by JsonReporter — that function is terminal-reporter-only per ADR 0003.
- skipped_packets=0 must appear as a key-value pair, not be absent from the JSON (no `skip_serializing_if` guard on this field).

## Failure Guidance

"HOLDOUT LOW: HS-064 (satisfaction: 0.XX) -- JSON reporter output was malformed; check that top-level keys are exactly 3, skipped_packets is always present, C0 bytes are \u-escaped, and Cyrillic appears as readable UTF-8 not escape sequences."
