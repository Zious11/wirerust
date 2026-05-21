---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-013.md
  - .factory/stories/STORY-014.md
  - .factory/stories/STORY-069.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.037.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.005.md
input-hash: "bfffc90"
traces_to: .factory/specs/prd.md
id: "HS-016"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.018
  - BC-2.04.037
  - BC-2.09.005
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Real-World Corpus — Known-Problematic PCAP with TCP Evasion Patterns

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A malware researcher runs wirerust on the Wireshark sample file `Ethereal-snort-dmz.pcap`
   or similar captures from the NETRESEC publicly available pcap repository that contain
   known TCP anomalies (retransmissions with conflicting data, overlapping segments, or
   fragmentation evasion patterns).
2. wirerust correctly detects and emits findings for the known anomalies — the false
   negative rate for TCP stream manipulation is low.
3. Each evasion-related finding carries raw forensic data in its evidence field — the actual
   bytes involved in the conflict are preserved verbatim.
4. The findings include MITRE technique IDs appropriate to the detected evasion technique
   (T1036 for masquerading/conflicting retransmissions).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.018 | Postcondition 1 — conflicting overlap emits Anomaly/Likely/High finding with MITRE T1036 | Step 2: evasion detected and finding emitted |
| BC-2.04.037 | Postcondition 1 — same-range conflicting overlap returns ConflictingOverlap; original bytes preserved | Step 3: first-wins policy preserves original data |
| BC-2.09.005 | Postcondition 1 — Finding.summary and evidence carry raw bytes | Step 3: forensic data in evidence field |

## Verification Approach

Download from NETRESEC publicly available PCAPs or Wireshark captures:
- `Capture-File-15.pcap` from honeynet.org forensic challenges (contains TCP evasion patterns)
- Or generate a synthetic pcap with conflicting TCP retransmissions using Scapy:
  ```python
  # Send SYN, SYN+ACK, ACK, DATA-seg1="AAAA", RETRANSMIT-seg1="BBBB" (same seq, different bytes)
  ```

```
wirerust analyze --output-format json tcp_evasion.pcap | jq '.findings[] | select(.mitre_technique_id == "T1036")'
```

Expect: at least one finding with category=Anomaly, verdict=Likely, confidence=High, and
mitre_technique_id=T1036. The `evidence` field should contain the differing byte content
without terminal-escape transformation.

| Field | Description |
|-------|-------------|
| corpus_source | Synthetic pcap with deliberate TCP conflicting retransmission, or Honeynet challenge PCAPs |
| corpus_size | Small targeted pcap: ~20 packets |
| known_edge_cases | Conflicting retransmission: same sequence number, different payload bytes |
| false_positive_threshold | N/A — anomaly is deliberately injected |
| false_negative_threshold | 0% — T1036 conflict must be detected |

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): T1036 finding is emitted for conflicting TCP
  overlap; finding has correct verdict/confidence/category.
- **Data integrity** (weight: 0.3): Evidence field in the finding contains the raw byte
  sequences involved in the conflict, not a sanitized description.
- **Edge case handling** (weight: 0.1): Multiple conflicting segments in one flow produce
  findings (up to threshold), not just the first one.
- **Error quality** (weight: 0.1): No panic or crash on the evasion pcap.

## Edge Conditions

- A retransmission that is an EXACT duplicate (same bytes) must NOT emit a T1036 finding;
  only genuinely different bytes trigger the conflict finding.
- TCP timestamp options or extended headers in the retransmit must not confuse the
  payload-level comparison.

## Failure Guidance

"HOLDOUT LOW: HS-016 (satisfaction: 0.XX) — conflicting TCP retransmission not detected,
T1036 finding not emitted, or evidence field contains sanitized rather than raw bytes."
