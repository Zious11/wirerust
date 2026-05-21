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
  - .factory/stories/STORY-018.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.019.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.041.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-017.md
id: "HS-047"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.018
  - BC-2.04.019
  - BC-2.04.041
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Real-World Known-Problematic PCAP Detects TCP Evasion Signatures

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

Run wirerust against a publicly available pcap that is known to contain TCP
IDS-evasion payloads or other reassembly anomalies. The tool should detect
the known anomalies and emit the appropriate findings without crashing.

### Corpus source

**Corpus:** Wireshark sample captures — `tcp-ethereal-file1.trace` or
pcap samples from the Snort/Suricata test suites that include TCP evasion
techniques (available from https://snort.org/downloads/additional or
from https://github.com/nids-testbed repositories).

Alternatively, use the Packetloss.is public pcap archives which include
traffic with various TCP anomalies (retransmissions with modified bytes,
segment interleaving, etc.).

**Corpus size:** ~10-100 flows, crafted to exercise specific anomalies.

### Scenario Steps

1. The user runs: `wirerust analyze <evasion-corpus-pcap> --format json`
2. The tool completes with exit code 0.
3. The JSON findings array contains at least one finding with `mitre_technique == "T1036"`.
4. No crash, panic, or memory exhaustion occurs even for adversarial inputs.
5. The finding summary contains the flow key (IP addresses and ports) of the
   offending flow.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.018 | postcondition 2: ConflictingOverlap produces T1036/Likely/High finding | Known evasion pcap triggers detection |
| BC-2.04.019 | postcondition 1: cumulative overlap threshold alert | Multiple overlaps trigger cumulative alert |
| BC-2.04.041 | postcondition 1-5: depth truncation for extremely deep flows | Deep flows in adversarial corpus bounded |

## Verification Approach

```bash
wirerust analyze <evasion-pcap> --format json > evasion_output.json
cat evasion_output.json | jq '.findings | map(select(.mitre_technique == "T1036")) | length'
```

**Expected result:**
- At least 1 T1036 finding (the known evasion is detected).
- No crash.
- Exit code 0.

**False negative threshold:** 0 missed T1036 events for a pcap specifically
crafted with conflicting TCP segment overlaps. Every ConflictingOverlap event
in the pcap must produce a finding (up to the MAX_FINDINGS cap).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Known evasion signatures produce
  T1036 findings; no false negatives for confirmed evasion traffic.
- **Edge case handling** (weight: 0.2): Tool remains stable under adversarial
  inputs; no crash, OOM, or hang.
- **Error quality** (weight: 0.1): Clean exit; no unexpected errors.
- **Performance** (weight: 0.1): Completes in reasonable time for adversarial pcap.
- **Data integrity** (weight: 0.1): Finding content is accurate (correct IP/port
  in summary; correct confidence level).

## Edge Conditions

- Adversarial pcap with 10,000+ conflicting overlaps: findings capped at 10,000
  normal + 1 segment-limit summary; `dropped_findings > 0`.
- Pcap with depth-exhaustion attack: truncation finding emitted; no OOM.
- Pcap combining evasion with normal traffic: only the evasion flows get T1036
  findings; normal flows remain clean.

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Snort/Suricata test pcap suites or crafted adversarial pcap with TCP overlap injections |
| corpus_size | ~10-100 flows; adversarial crafting ensures known anomalies |
| known_edge_cases | Conflicting byte overlaps; segment splicing; depth exhaustion |
| false_positive_threshold | N/A (adversarial corpus; all T1036 findings are expected) |
| false_negative_threshold | 0 missed T1036 events per ConflictingOverlap in pcap |

## Failure Guidance

"HOLDOUT LOW: HS-047 (satisfaction: 0.XX) — known TCP evasion signatures were
not detected; zero T1036 findings were produced for a pcap specifically containing
conflicting TCP segment overlaps, indicating a false-negative in evasion detection."
