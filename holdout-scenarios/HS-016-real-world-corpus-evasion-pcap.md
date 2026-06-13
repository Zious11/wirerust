---
document_type: holdout-scenario
level: ops
version: "1.2"
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
input-hash: "af54ab6"
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

> **Rubric note (v1.1 — Phase-4 adjudication):** The raw-conflicting-bytes-in-evidence
> expectation has been removed from the rubric. None of the cited BCs require the
> conflicting-overlap finding's `evidence` field to contain raw conflicting bytes:
> - BC-2.04.018 PC2 enumerates the finding's fields (category, verdict, confidence, mitre,
>   summary=FlowKey, direction=None) and does NOT mention the `evidence` field at all.
> - BC-2.04.037 is scoped to `insert_segment` return value and buffer state; it does not
>   specify the `evidence` contents of the downstream finding.
> - BC-2.09.005 is a "do not escape attacker-controlled bytes" contract — it does not mandate
>   that the overlap finding populate `evidence` with raw bytes.
> The current implementation (`evidence = ["Retransmitted segment contains different data"]`)
> is BC-compliant. See holdout-finding-triage-2026-06-01.md §Finding 2 for full analysis.
>
> FUTURE ENHANCEMENT NOTE: Forensic parity with the TLS SNI path (which emits raw hex in
> evidence) would improve overlap finding utility. If BC-2.04.018 is amended to require raw
> conflicting bytes in `evidence`, this rubric item may be reinstated. Until such an amendment,
> a description string is compliant and this item does not count against satisfaction.

## Scenario

1. A malware researcher runs wirerust on the Wireshark sample file `Ethereal-snort-dmz.pcap`
   or similar captures from the NETRESEC publicly available pcap repository that contain
   known TCP anomalies (retransmissions with conflicting data, overlapping segments, or
   fragmentation evasion patterns).
2. wirerust correctly detects and emits findings for the known anomalies — the false
   negative rate for TCP stream manipulation is low.
3. The findings include MITRE technique IDs appropriate to the detected evasion technique
   (T1036 for masquerading/conflicting retransmissions).
4. The `evidence` field is present and non-empty (a description is acceptable; raw bytes are
   not required by any current BC).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.018 | Postcondition 1 — conflicting overlap emits Anomaly/Likely/High finding with MITRE T1036 | Step 2: evasion detected and finding emitted with correct fields |
| BC-2.04.037 | Postcondition 1 — same-range conflicting overlap returns ConflictingOverlap; original bytes preserved in buffer | Step 2: first-wins policy correctly identifies conflict (buffer preservation is the insert_segment invariant; the evidence string in the finding is separate) |
| BC-2.09.005 | Postcondition 1 — Finding.summary and evidence carry attacker-controlled bytes WITHOUT terminal-escape transformation | Step 4: evidence string, if it contains attacker-controlled content, must not be escape-sanitized. (Note: the current fixed description string "Retransmitted segment contains different data" is hardcoded, not attacker-controlled, so BC-2.09.005 neither governs nor is violated by it.) |

## Verification Approach

Download from NETRESEC publicly available PCAPs or Wireshark captures:
- `Capture-File-15.pcap` from honeynet.org forensic challenges (contains TCP evasion patterns)
- Or generate a synthetic pcap with conflicting TCP retransmissions using Scapy:
  ```python
  # Send SYN, SYN+ACK, ACK, DATA-seg1="AAAA", RETRANSMIT-seg1="BBBB" (same seq, different bytes)
  ```

```
wirerust analyze --output-format json tcp_evasion.pcap | jq '.findings[] | select(.mitre_techniques // [] | index("T1036"))'
```

Expect: at least one finding with category=Anomaly, verdict=Likely, confidence=High, and
mitre_techniques containing "T1036". The `evidence` field must be present and non-empty; its specific
content (raw bytes vs. description string) is NOT checked by this scenario.

| Field | Description |
|-------|-------------|
| corpus_source | Synthetic pcap with deliberate TCP conflicting retransmission, or Honeynet challenge PCAPs |
| corpus_size | Small targeted pcap: ~20 packets |
| known_edge_cases | Conflicting retransmission: same sequence number, different payload bytes |
| false_positive_threshold | N/A — anomaly is deliberately injected |
| false_negative_threshold | 0% — T1036 conflict must be detected |

## Evaluation Rubric

- **Functional correctness** (weight: 0.7): T1036 finding is emitted for conflicting TCP
  overlap; finding has correct verdict (Likely), confidence (High), category (Anomaly), and
  mitre_techniques containing "T1036".
- **Edge case handling** (weight: 0.2): Multiple conflicting segments in one flow produce
  findings (up to threshold), not just the first one. Exact-duplicate retransmissions do NOT
  emit a T1036 finding (only genuinely different bytes trigger the conflict finding).
- **Error quality** (weight: 0.1): No panic or crash on the evasion pcap.

NOTE: The "Data integrity" rubric item (raw byte sequences in evidence) has been removed
(v1.1). No cited BC requires raw bytes in the overlap finding's evidence field. The current
compliant behavior is a fixed descriptive string. If a future BC amendment adds this
requirement, reinstate the item here.

## Edge Conditions

- A retransmission that is an EXACT duplicate (same bytes) must NOT emit a T1036 finding;
  only genuinely different bytes trigger the conflict finding.
- TCP timestamp options or extended headers in the retransmit must not confuse the
  payload-level comparison.

## Failure Guidance

"HOLDOUT LOW: HS-016 (satisfaction: 0.XX) — conflicting TCP retransmission not detected,
or T1036 finding not emitted with correct verdict/confidence/category."
