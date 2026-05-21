---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-001.md
  - .factory/stories/STORY-005.md
  - .factory/stories/STORY-011.md
  - .factory/stories/STORY-012.md
  - .factory/stories/STORY-066.md
  - .factory/stories/STORY-069.md
  - .factory/stories/STORY-071.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.002.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.028.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.003.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.003.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-023"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.002
  - BC-2.04.028
  - BC-2.08.003
  - BC-2.10.003
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Waves 1-5 Full Integration — PCAP -> Decode -> Reassembly -> DNS -> MITRE

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A SOC operator runs wirerust on a realistic network capture containing:
   - TCP connections with a full three-way handshake
   - DNS queries and responses (UDP port 53)
   - At least one TCP stream with detectable content
2. The JSON output structure has three top-level keys: `summary`, `findings`, `analyzers`.
3. The `analyzers` key contains sub-objects for each enabled analyzer:
   - `reassembly` with TCP reassembly statistics
   - `dns` with query/response counts
4. The `summary` key includes total_packets, total_bytes, and skipped_packets.
5. Any TCP-layer findings include MITRE tactic grouping when viewed in terminal mode.
6. The whole pipeline completes in a reasonable time (under 30 seconds for a 1MB pcap).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.002 | Postcondition 1 — all packets loaded in order as the pipeline input | Step 1: ingestion feeds everything |
| BC-2.04.028 | Postcondition 1 — summarize returns AnalysisSummary with reassembly stats | Step 3: reassembly analyzer present in output |
| BC-2.08.003 | Postcondition 1 — summarize emits AnalysisSummary with dns_queries and dns_responses | Step 3: DNS analyzer present in output |
| BC-2.10.003 | Postcondition 1 — all_tactics_in_report_order for grouping in terminal output | Step 5: MITRE grouping in terminal mode |

## Verification Approach

```
wirerust analyze --dns --output-format json realistic_traffic.pcap | jq '{
  has_summary: (has("summary")),
  has_findings: (has("findings")),
  has_analyzers: (has("analyzers")),
  has_reassembly: (.analyzers | has("reassembly")),
  has_dns: (.analyzers | has("dns")),
  total_packets: .summary.total_packets,
  dns_queries: .analyzers.dns.dns_queries
}'
```

All boolean fields should be `true`. total_packets > 0. dns_queries >= 0.

Terminal mode:
```
wirerust analyze --dns --mitre realistic_traffic.pcap
```

Verify no crash; output is organized with tactic headers.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): JSON structure has correct top-level keys;
  all enabled analyzers appear in `analyzers` sub-object.
- **Data integrity** (weight: 0.3): Packet counts, byte counts, DNS counts, and reassembly
  stats are internally consistent (e.g., bytes_reassembled <= total_bytes).
- **Performance** (weight: 0.2): Pipeline completes within 30 seconds for a 1MB pcap on
  typical hardware.
- **Edge case handling** (weight: 0.1): Running without --dns flag: dns analyzer absent
  from analyzers; running with --dns: present.

## Edge Conditions

- If a pcap has no DNS traffic: dns_queries and dns_responses should both be 0 (not absent).
- If a pcap has no TCP traffic: bytes_reassembled should be 0 and flows_total should be 0.
- The `analyzers` key must always be present even if empty (no analyzers enabled).

## Failure Guidance

"HOLDOUT LOW: HS-023 (satisfaction: 0.XX) — the integrated pipeline from waves 1-5 produces
incorrect JSON structure, missing analyzer sub-objects, or wrong summary statistics."
