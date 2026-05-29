---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-051.md
  - .factory/stories/STORY-052.md
  - .factory/stories/STORY-053.md
  - .factory/stories/STORY-054.md
  - .factory/stories/STORY-055.md
  - .factory/stories/STORY-056.md
  - .factory/stories/STORY-057.md
  - .factory/stories/STORY-058.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.031.md
input-hash: "6e52bc5"
traces_to: .factory/stories/STORY-051.md
id: "HS-066"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.031
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: TLS Analyzer Summarize Output Has All Required Keys With Correct Semantics

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains TLS traffic with: 3 distinct ClientHello handshakes with distinct SNIs, 3 corresponding ServerHellos with distinct JA3S hashes, 25 distinct SNIs (to test top_snis truncation), TLS 1.2 and TLS 1.3 sessions mixed, and 1 oversized record (to populate truncated_records).
2. The analyst runs wirerust on this pcap with JSON output.
3. The analyst inspects the TLS analyzer entry in the `analyzers` array.
4. Expected: `packets_analyzed` equals the number of handshake pairs seen (not packets). The `detail` BTreeMap has exactly 7 keys in alphabetical order. `top_snis` contains at most 20 entries sorted by count descending. `tls_versions` keys are decimal strings (not hex). `parse_errors` and `truncated_records` are both always present, even when 0. `cipher_suites`, `ja3_hashes`, and `ja3s_hashes` are present and reflect the actual handshake data.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.031 | postcondition 1-9; invariant 1-4 | Complete 7-key detail map; alphabetical BTreeMap ordering; top_snis capped at 20; tls_versions in decimal strings; parse_errors and truncated_records always present |

## Verification Approach

Run wirerust on the TLS-heavy pcap. Parse JSON output.

1. Navigate to `analyzers[].detail` where `analyzer_name == "TLS"`.
2. Assert `Object.keys(detail).sort()` equals `["cipher_suites","ja3_hashes","ja3s_hashes","parse_errors","tls_versions","top_snis","truncated_records"]` (7 keys, alphabetical).
3. Assert `detail.top_snis` is an array with at most 20 entries, sorted by count descending.
4. Assert `detail.tls_versions` keys are decimal strings like `"771"` (not `"0x0303"`).
5. Assert `detail.parse_errors` is present as a JSON number.
6. Assert `detail.truncated_records` is present as a JSON number (may be 0 for clean traffic, >= 1 when oversized record present).
7. Assert `packets_analyzed` equals the number of complete handshakes seen (not total TLS records).

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Exactly 7 keys; packets_analyzed counts handshakes; tls_versions in decimal; top_snis sorted and capped.
- **Edge case handling** (weight: 0.3): 25 SNIs → top_snis has exactly 20; truncated_records key present even when 0.
- **Error quality** (weight: 0.2): BTreeMap ensures alphabetical ordering in serialized JSON output.
- **Data integrity** (weight: 0.1): summarize() is read-only; can be called multiple times with identical results.

## Edge Conditions

- 25 distinct SNIs — top_snis must be truncated to exactly 20 (sorted by count descending, .take(20)).
- Version 0x0303 must appear as `"771"` (decimal conversion), not `"0x0303"`.
- `truncated_records` and `parse_errors` must always be present as JSON numbers, even when both are 0 (after a perfectly valid pcap).

## Failure Guidance

"HOLDOUT LOW: HS-066 (satisfaction: 0.XX) -- TLS summarize output was incomplete or incorrect; verify exactly 7 detail keys, tls_versions in decimal, top_snis truncated to 20, and truncated_records always present."
