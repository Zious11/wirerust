---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-066.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.001.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.002.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.003.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.004.md
input-hash: "7d67a54"
traces_to: .factory/specs/prd.md
id: "HS-011"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-6"
behavioral_contracts:
  - BC-2.08.001
  - BC-2.08.002
  - BC-2.08.003
  - BC-2.08.004
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: DNS — Query/Response Counting Without Emitting Any Findings

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A SOC operator processes a pcap containing 50 DNS queries and 50 DNS responses mixed
   with HTTP and TCP traffic, using `wirerust analyze --dns --output-format json`.
2. The JSON summary contains a DNS statistics section showing approximately 50 queries
   and 50 responses (exact counts depend on the pcap content).
3. The findings array contains NO entries with any DNS-related category or tag — DNS is
   statistics-only.
4. An unusual DNS scenario: a very high volume of queries with no corresponding responses
   (potential DNS flood). The tool still emits no findings — it counts queries and responses
   faithfully but makes no anomaly determination.
5. DNS traffic on port 53 TCP (not just UDP) is also counted correctly.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.08.001 | Postcondition 1 — port 53 TCP or UDP matches DnsAnalyzer | Steps 1, 5: port-53 dispatch regardless of transport |
| BC-2.08.002 | Postcondition 1-2 — QR-bit dispatch: response_count if bit set, query_count otherwise | Steps 2, 4: correct QR-bit counting |
| BC-2.08.003 | Postcondition 1 — summarize emits AnalysisSummary with dns_queries and dns_responses | Step 2: summary statistics structure |
| BC-2.08.004 | Postcondition 1 — DnsAnalyzer NEVER emits findings | Steps 3, 4: zero findings guarantee |

## Verification Approach

```
wirerust analyze --dns --output-format json dns_heavy.pcap | jq '{dns: .analyzers.dns, findings_count: (.findings | length)}'
```

Expect output like:
```json
{
  "dns": {"dns_queries": 50, "dns_responses": 50},
  "findings_count": 0
}
```

For TCP DNS on port 53:
```
wirerust analyze --dns --output-format json dns_over_tcp.pcap
```

Confirm `dns_queries` and `dns_responses` are non-zero.

For the flood scenario: run on a pcap with 1000 DNS queries and 0 responses.
Confirm `findings` array is still empty.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Query and response counts match the pcap
  content; findings array is empty regardless of DNS traffic pattern.
- **Data integrity** (weight: 0.3): QR-bit classification is accurate — response bit set
  correctly categorizes the packet as response, not query.
- **Edge case handling** (weight: 0.15): DNS over TCP (port 53 TCP) counted identically
  to DNS over UDP.
- **Performance** (weight: 0.1): Tool completes normally on a high-volume DNS pcap (1000+
  DNS packets) without excessive memory use.

## Edge Conditions

- A packet with src_port=53 AND dst_port=53 (loopback DNS server): both ports match;
  behavior is defined by which port the dispatch rule matches first.
- A UDP packet that is NOT DNS (port 54 source, 54 destination) should not be counted
  as DNS even if the bytes happen to look like a DNS message.
- The QR bit is bit 15 of the flags field in the DNS header (offset 2 from payload start
  after a 2-byte ID). Malformed DNS payloads that are too short should not crash the tool.

## Failure Guidance

"HOLDOUT LOW: HS-011 (satisfaction: 0.XX) — DNS analyzer emits findings, miscounts
queries/responses, or fails to process TCP port-53 traffic."
