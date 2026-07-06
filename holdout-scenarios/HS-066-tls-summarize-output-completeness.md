---
document_type: holdout-scenario
level: ops
version: "1.1"
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
input-hash: "0639a9b"
traces_to: .factory/stories/STORY-051.md
id: "HS-066"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.031
lifecycle_status: active
modified:
  - "v1.1 (maint-2026-07-06): stale→active — relaxed 7-key assertion to 10-key assertion per BC-2.07.031 v1.5 (dropped_map_entries) + BC-2.07.039 (handshake_reassembly_overflows) + BC-2.07.043 (buffer_saturation_drops); updated postcondition range 1-9→1-10 (FIX-C holdout repair)"
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
4. Expected: `packets_analyzed` equals the number of handshake pairs seen (not packets). The `detail`
   BTreeMap has exactly 10 keys in alphabetical order: `buffer_saturation_drops`, `cipher_suites`,
   `dropped_map_entries`, `handshake_reassembly_overflows`, `ja3_hashes`, `ja3s_hashes`,
   `parse_errors`, `tls_versions`, `top_snis`, `truncated_records`. `top_snis` contains at most 20
   entries sorted by count descending. `tls_versions` keys are decimal strings (not hex).
   `parse_errors`, `truncated_records`, `dropped_map_entries`, `handshake_reassembly_overflows`, and
   `buffer_saturation_drops` are all always present even when 0. `cipher_suites`, `ja3_hashes`, and
   `ja3s_hashes` are present and reflect the actual handshake data.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.031 | postcondition 1-10; invariant 1-5 | Complete 10-key detail map; alphabetical BTreeMap ordering; top_snis capped at 20; tls_versions in decimal strings; parse_errors, truncated_records, dropped_map_entries, handshake_reassembly_overflows, and buffer_saturation_drops always present |

## Verification Approach

Run wirerust on the TLS-heavy pcap. Parse JSON output.

1. Navigate to `analyzers[].detail` where `analyzer_name == "TLS"`.
2. Assert `Object.keys(detail).sort()` equals `["buffer_saturation_drops","cipher_suites","dropped_map_entries","handshake_reassembly_overflows","ja3_hashes","ja3s_hashes","parse_errors","tls_versions","top_snis","truncated_records"]` (10 keys, alphabetical). All three observability counters must be present even when 0.
3. Assert `detail.top_snis` is an array with at most 20 entries, sorted by count descending.
4. Assert `detail.tls_versions` keys are decimal strings like `"771"` (not `"0x0303"`).
5. Assert `detail.parse_errors` is present as a JSON number.
6. Assert `detail.truncated_records` is present as a JSON number (may be 0 for clean traffic, >= 1 when oversized record present).
7. Assert `packets_analyzed` equals the number of complete handshakes seen (not total TLS records).

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Exactly 10 keys (including `dropped_map_entries`, `handshake_reassembly_overflows`, `buffer_saturation_drops` always present); packets_analyzed counts handshakes; tls_versions in decimal; top_snis sorted and capped.
- **Edge case handling** (weight: 0.3): 25 SNIs → top_snis has exactly 20; truncated_records key present even when 0.
- **Error quality** (weight: 0.2): BTreeMap ensures alphabetical ordering in serialized JSON output.
- **Data integrity** (weight: 0.1): summarize() is read-only; can be called multiple times with identical results.

## Edge Conditions

- 25 distinct SNIs — top_snis must be truncated to exactly 20 (sorted by count descending, .take(20)).
- Version 0x0303 must appear as `"771"` (decimal conversion), not `"0x0303"`.
- `truncated_records` and `parse_errors` must always be present as JSON numbers, even when both are 0 (after a perfectly valid pcap).

## Failure Guidance

"HOLDOUT LOW: HS-066 (satisfaction: 0.XX) -- TLS summarize output was incomplete or incorrect; verify exactly 10 detail keys (including dropped_map_entries, handshake_reassembly_overflows, buffer_saturation_drops), tls_versions in decimal, top_snis truncated to 20, and all observability counters always present."
