---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-069.md
  - .factory/stories/STORY-013.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-024"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-7"
behavioral_contracts:
  - BC-2.09.001
  - BC-2.04.018
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Source IP Field — Present for Reassembly Findings, Absent for HTTP/TLS Findings

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A forensic analyst runs wirerust on a pcap that triggers both a TCP reassembly anomaly
   (conflicting overlap) and an HTTP anomaly (path traversal).
2. The JSON output for the reassembly finding INCLUDES a `source_ip` key with the IP address
   of the packet that triggered the anomaly.
3. The JSON output for the HTTP finding does NOT include a `source_ip` key (it is absent,
   not null).
4. This source IP field lets the analyst immediately identify the attacker's IP from the
   reassembly finding without consulting the full packet dump.
5. The source IP is the actual IP from the packet (e.g., "10.0.0.1" or an IPv6 address),
   not a placeholder or a default value.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.09.001 | Invariant 2 — reassembly anomaly findings have source_ip: Some(packet.src_ip); HTTP and TLS findings have source_ip: None | Steps 2-3: source_ip presence/absence per finding type |
| BC-2.04.018 | Postcondition 1 — conflicting overlap emits Anomaly/Likely/High finding with MITRE T1036 | Step 2: the reassembly finding that carries source_ip |

## Verification Approach

```
wirerust analyze --output-format json combined_anomalies.pcap | jq '
  .findings | group_by(.category) | map({
    key: .[0].category,
    has_source_ip: (map(has("source_ip")) | any)
  })'
```

Check that findings from TCP reassembly (category "Anomaly" with MITRE T1036) have
`has_source_ip: true`, while HTTP findings (e.g., path traversal, category "Reconnaissance")
have `has_source_ip: false`.

Verify the source_ip value is a valid IP address string:
```
wirerust analyze --output-format json combined_anomalies.pcap | jq '
  [.findings[] | select(has("source_ip")) | .source_ip]'
```

All values should be valid dotted-decimal IPv4 or colon-separated IPv6 address strings.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): source_ip present for reassembly findings;
  absent for HTTP/TLS findings — no exceptions.
- **Data integrity** (weight: 0.3): source_ip values are valid IP address strings matching
  the actual packet source in the pcap.
- **Edge case handling** (weight: 0.1): A reassembly finding where the triggering packet
  has an IPv6 source shows an IPv6 source_ip string (not dotted-decimal).
- **Error quality** (weight: 0.1): source_ip is never null in JSON — it is either present
  with a value or entirely absent.

## Edge Conditions

- A reassembly finding triggered by a packet from an IPv6 host: source_ip should be an
  IPv6 address string.
- Multiple reassembly findings from the same source IP: each finding independently carries
  the source_ip.
- A flow where the "anomalous" packet comes from the server side (responder): source_ip
  should be the server's IP, not always the client's.

## Failure Guidance

"HOLDOUT LOW: HS-024 (satisfaction: 0.XX) — source_ip field appears in HTTP/TLS findings
where it should be absent, or is missing from reassembly findings where it should be present."
