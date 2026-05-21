---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-015.md
  - .factory/stories/STORY-016.md
  - .factory/stories/STORY-031.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.006.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.035.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.001.md
input-hash: "9dc8bb4"
traces_to: .factory/stories/STORY-031.md
id: "HS-046"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.006
  - BC-2.04.035
  - BC-2.05.001
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Real-World Known-Good PCAP Produces Low False-Positive Rate

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

Run wirerust against a well-known, publicly available pcap from a clean
production network environment. The tool should produce minimal false-positive
T1036 findings (normal retransmissions must not be flagged) and correctly
identify the protocol mix in the capture.

### Corpus source

**Corpus:** Wireshark sample captures collection — specifically the
`http_with_junk_after_headers.pcap` or `http-chunked-gzip.pcap` from
https://wiki.wireshark.org/SampleCaptures

This corpus is:
- Well-maintained and widely used for network analysis tool testing
- Known to contain normal HTTP traffic with standard retransmissions
- Publicly available and reproducible

**Corpus size:** Approximately 50-200 TCP flows, ~10,000-100,000 packets.

### Scenario Steps

1. The user runs: `wirerust analyze <wireshark-sample-pcap> --output-format json`
2. The tool completes with exit code 0.
3. The JSON output contains HTTP-level findings and statistics (requests
   counted, methods seen, status codes).
4. The T1036/ConflictingOverlap findings count is at or near zero (normal
   HTTP traffic does not use conflicting TCP overlaps).
5. The `bytes_reassembled` value is plausible for the pcap's total payload.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.006 | postcondition 1-2: direction tagging correct | HTTP requests and responses correctly differentiated |
| BC-2.04.035 | postcondition 1-2: identical retransmissions not flagged as conflicts | Clean TCP retransmissions produce zero T1036 findings |
| BC-2.05.001 | postcondition 1: TLS content signature routes correctly | Any HTTPS flows in the corpus are correctly identified |

## Verification Approach

```bash
wirerust analyze ~/sample.pcap --output-format json > output.json
cat output.json | jq '.findings | map(select(.mitre_technique == "T1036")) | length'
```

**Expected result:**
- T1036 findings: 0 (or very close to 0; normal network traffic does not
  contain IDS evasion attacks).
- Exit code: 0.
- `bytes_reassembled` > 0 (some data was reassembled).
- HTTP stats: at least 1 request counted.

**False positive threshold:** At most 1 T1036 finding per 10,000 flows
(network captures from clean environments occasionally have legitimate
retransmission anomalies from hardware bugs, but not from evasion).

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Tool runs to completion; HTTP stats present.
- **Edge case handling** (weight: 0.2): Normal retransmissions don't trigger T1036.
- **Error quality** (weight: 0.2): No crash or panic on production traffic.
- **Performance** (weight: 0.1): Completes in reasonable time (under 60 seconds
  for a typical sample pcap).
- **Data integrity** (weight: 0.1): `bytes_reassembled` is plausible; no obvious
  double-counting.

## Edge Conditions

- Pcap may contain IPv6 traffic: should not crash (IPv6 flows with no HTTP/TLS
  data become unclassified).
- Pcap may contain UDP, ICMP: these are not TCP and should be counted in
  skipped_packets, not reassembled.
- Pcap may contain partial connections: incomplete 3-way handshakes produce
  mid-stream-join flows; these work correctly.

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | https://wiki.wireshark.org/SampleCaptures — specifically HTTP sample pcaps |
| corpus_size | ~50-200 flows, ~10,000-100,000 packets |
| known_edge_cases | Some retransmissions; occasional TCP window scaling; possible out-of-order packets |
| false_positive_threshold | 0 T1036/ConflictingOverlap findings for clean retransmission traffic |
| false_negative_threshold | HTTP flows must be classified; `http_requests > 0` required |

## Failure Guidance

"HOLDOUT LOW: HS-046 (satisfaction: 0.XX) — production-traffic false positive
rate exceeded threshold; T1036 findings were emitted for normal TCP
retransmissions in a known-good HTTP pcap, or the tool crashed on standard
network traffic."
