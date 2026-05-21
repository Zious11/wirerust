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
  - .factory/stories/STORY-069.md
  - .factory/stories/STORY-070.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.002.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md
  - .factory/specs/behavioral-contracts/ss-10/BC-2.10.005.md
input-hash: "a3ed987"
traces_to: .factory/specs/prd.md
id: "HS-017"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-7"
behavioral_contracts:
  - BC-2.01.002
  - BC-2.09.001
  - BC-2.09.006
  - BC-2.10.005
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: E-1 to E-7 Cross-Subsystem — Packet Ingestion Feeds Finding Construction

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A security analyst runs wirerust end-to-end on a pcap that contains a TCP stream with
   detectable anomalies — the pipeline flows from packet ingestion (E-1) through packet
   decoding (E-1/E-2 boundary) into finding emission (E-7).
2. The JSON output has a well-formed `findings` array where each element has the required
   fields (category, verdict, confidence, summary) plus optional fields only when non-None.
3. For findings with MITRE technique IDs, the technique ID resolves to a known name in the
   catalog — confirming the E-7 MITRE mapping (STORY-071) integrates correctly with findings
   emitted by E-2 reassembly.
4. The `timestamp` field is absent from ALL findings in the JSON output (not null, absent).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.002 | Postcondition 1 — packets loaded in file order; drives finding production pipeline | Step 1: ingestion feeding the full pipeline |
| BC-2.09.001 | Postcondition 1 — Finding constructed with required and optional fields | Step 2: finding structure correctness |
| BC-2.09.006 | Postcondition 1 — None fields (timestamp) omitted from JSON via skip_serializing_if | Step 4: timestamp absent from all JSON findings |
| BC-2.10.005 | Postcondition 1 — technique_name resolves all emitted technique IDs | Step 3: E-7 MITRE catalog serves E-2 findings |

## Verification Approach

```
wirerust analyze --output-format json --mitre evasion_traffic.pcap
```

Check JSON:
1. `findings` array is non-empty.
2. Each finding object lacks a `timestamp` key (use `jq 'has("timestamp")'` on each element).
3. Each finding object with a `mitre_technique_id` has a corresponding human-readable
   technique name visible in `--mitre` terminal mode.
4. No null values appear for any key — only absent keys for None fields.

Cross-check with terminal output:
```
wirerust analyze --mitre evasion_traffic.pcap
```

Verify that the MITRE tactic grouping references the same technique names as the catalog.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Findings are well-formed; required fields
  present; None fields absent from JSON.
- **Data integrity** (weight: 0.3): MITRE technique IDs in findings resolve to names in
  the catalog; no finding has an unresolvable technique ID in normal operation.
- **Edge case handling** (weight: 0.15): A capture producing 0 findings still returns a
  valid JSON structure with an empty `findings` array (not null or absent).
- **Error quality** (weight: 0.1): Pipeline errors in packet decoding are counted as
  skipped_packets, not swallowed silently.

## Edge Conditions

- A finding with `source_ip: Some(...)` from a reassembly anomaly must show `source_ip`
  in JSON; a finding from HTTP with `source_ip: None` must not show `source_ip` in JSON.
- MITRE technique IDs in JSON use the canonical string format (e.g., "T1036", not 1036).

## Failure Guidance

"HOLDOUT LOW: HS-017 (satisfaction: 0.XX) — cross-subsystem pipeline from packet ingestion
to finding construction produces malformed JSON, shows null timestamp, or MITRE technique
IDs do not resolve."
