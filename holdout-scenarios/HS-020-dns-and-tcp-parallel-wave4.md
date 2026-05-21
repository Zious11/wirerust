---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-011.md
  - .factory/stories/STORY-066.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.003.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.001.md
  - .factory/specs/behavioral-contracts/ss-08/BC-2.08.004.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-020"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-6"
behavioral_contracts:
  - BC-2.04.003
  - BC-2.08.001
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

# Holdout Scenario: Cross-Subsystem Wave 4 — DNS Statistics Alongside TCP Reassembly

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains interleaved DNS queries (UDP port 53), DNS over TCP (TCP port 53),
   and a concurrent HTTP session being reassembled.
2. wirerust processes all traffic in a single pass: DNS packets are counted by the DNS
   analyzer, TCP traffic is reassembled by the TCP reassembly engine, and HTTP content
   is passed to the HTTP analyzer.
3. The DNS analyzer counts queries and responses but emits zero findings.
4. The TCP reassembly engine handles the TCP connections independently of the DNS UDP
   traffic — DNS UDP packets do not interfere with TCP flow state.
5. The final JSON output contains both DNS statistics and TCP/HTTP findings (if any)
   in a single consistent structure.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.003 | Postcondition 1 — FlowKey canonicalization for TCP flows | Step 4: TCP flows are isolated from UDP DNS traffic |
| BC-2.08.001 | Postcondition 1 — port 53 dispatch catches both TCP and UDP | Steps 1-2: DNS captured regardless of transport |
| BC-2.08.004 | Postcondition 1 — DnsAnalyzer NEVER emits findings | Step 3: no DNS findings in output alongside HTTP findings |

## Verification Approach

```
wirerust analyze --dns --http --output-format json mixed_dns_http.pcap | jq '{
  dns_queries: .analyzers.dns.dns_queries,
  dns_responses: .analyzers.dns.dns_responses,
  findings_count: (.findings | length),
  has_dns_findings: ([.findings[] | select(.category | ascii_downcase | contains("dns"))] | length > 0)
}'
```

Expect:
- dns_queries > 0
- dns_responses > 0 (or >= 0 if pcap has only queries)
- has_dns_findings == false (no DNS category findings)
- findings_count >= 0 (may have HTTP findings from HTTP traffic)

Confirm that TCP port-53 DNS traffic is counted in DNS stats AND that the same port-53
TCP packets are NOT routed to the TCP reassembly engine as if they were HTTP connections.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): DNS stats populated; TCP HTTP flows reassembled;
  both coexist in one analysis run without interference.
- **Data integrity** (weight: 0.3): DNS UDP and DNS TCP both counted; TCP port-53 not
  accidentally processed by HTTP analyzer.
- **Edge case handling** (weight: 0.15): A DNS over TCP session that also has enough bytes
  to trigger HTTP method prefix detection should be routed by DNS port, not HTTP content
  (since DNS is a packet-level bypass of the stream dispatcher per spec).
- **Error quality** (weight: 0.1): Mixed-protocol pcap produces no parse errors from
  protocol type confusion.

## Edge Conditions

- DNS TCP port 53 packets routed to DNS analyzer; they should NOT also appear in the TCP
  reassembly flow table as TCP connections to be analyzed by HTTP/TLS.
- Very high volume DNS (1000+ packets) with concurrent HTTP should not degrade HTTP finding
  accuracy.

## Failure Guidance

"HOLDOUT LOW: HS-020 (satisfaction: 0.XX) — DNS statistics and TCP reassembly interfere
with each other in mixed-protocol captures, or DNS findings are emitted."
