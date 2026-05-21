---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-017.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.019.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.022.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-017.md
id: "HS-032"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.018
  - BC-2.04.019
  - BC-2.04.022
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: TCP Segment Splicing Evasion Is Detected with T1036 Finding

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

An adversary attempts to evade network analysis by sending two conflicting
TCP segments at the same sequence number range — a classic IDS evasion technique.
The first segment carries benign bytes; the second carries different bytes at the
same offsets, hoping one IDS sees the first and another sees the second.

1. A pcap contains a TCP flow where one segment at byte offset 100-149 carries
   bytes spelling "AAAA..." and a second segment at byte offset 100-149 (same
   range, same direction) carries different bytes spelling "BBBB...". Both
   segments appear in the capture.
2. The user runs: `wirerust analyze <spliced-pcap> --format json`
3. The tool emits at least one finding with category "Anomaly", verdict "Likely",
   confidence "High", and a MITRE technique reference to T1036.
4. The summary of the finding contains the flow key (source/destination IP and
   port identifiers).
5. The original bytes ("AAAA...") are preserved — the first-seen bytes win.
   The conflicting bytes ("BBBB...") are NOT used in the reassembled stream.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.018 | postcondition 2: Anomaly/Likely/High finding with T1036 emitted on conflict | Evasion detection finding is present |
| BC-2.04.018 | postcondition 3: original bytes preserved (first-wins) | BBBB never accepted into stream |
| BC-2.04.019 | postcondition 1: overlap threshold alert fires when cumulative overlaps exceed threshold | Many conflicts trigger a cumulative alert too |
| BC-2.04.022 | postcondition 1: cumulative threshold latch fires at most once per direction | No alert flooding from repeated conflicts |

## Verification Approach

### Known-problematic corpus

**Corpus source:** Craft a minimal pcap using Python's `scapy` library or a
pre-built IDS test set (e.g., the Snort/Suricata test pcap collections that
exercise TCP evasion). The key property: same flow key, same direction, same
byte range, different bytes.

```bash
wirerust analyze <evasion-pcap> --format json
```

In the JSON findings array, verify:
- At least one finding where `category == "Anomaly"` and `verdict == "Likely"`
  and `confidence == "High"`.
- The finding's `mitre_technique` field contains "T1036".
- The finding's `summary` contains the IP-port pair of the flow.

Verify the cumulative overlap alert (if more than the threshold number of
overlapping segments exist): one additional finding per direction with
confidence "Medium" and technique "T1036".

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): T1036 finding emitted for each
  conflicting overlap event; original bytes preserved.
- **Edge case handling** (weight: 0.2): Many conflicts do not cause alert flooding
  beyond the one-shot cumulative latch.
- **Error quality** (weight: 0.1): No crash on adversarial input.
- **Performance** (weight: 0.1): Does not degrade severely under heavy overlap load.
- **Data integrity** (weight: 0.1): First-wins policy consistently enforced;
  conflicting bytes never appear in the reassembled stream.

## Edge Conditions

- Conflict when the findings cap (10,000) is already full: latch is still set
  even if the finding is dropped.
- Both directions independently produce conflicts: each direction's per-direction
  latch fires independently.
- A single conflicting segment among many normal segments: exactly one T1036 finding
  is emitted for that one event; normal segments produce no findings.

## Failure Guidance

"HOLDOUT LOW: HS-032 (satisfaction: 0.XX) — TCP segment-splicing evasion was
not detected; no T1036 finding was emitted for a flow with conflicting byte
ranges at the same sequence offsets."
