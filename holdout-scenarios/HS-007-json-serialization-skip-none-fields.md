---
document_type: holdout-scenario
level: ops
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-069.md
  - .factory/stories/STORY-070.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.005.md
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md
input-hash: "5cabe5c"
traces_to: .factory/specs/prd.md
id: "HS-007"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-7"
behavioral_contracts:
  - BC-2.09.001
  - BC-2.09.005
  - BC-2.09.006
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: JSON Finding Serialization — None Fields Omitted, Raw Bytes Preserved

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user runs `wirerust analyze --output-format json` on a capture that contains detectable
   HTTP and TLS anomalies.
2. The JSON output contains a `findings` array where each object has required fields
   (category, verdict, confidence, summary) always present.
3. Optional/conditional fields are OMITTED from the JSON when absent — the keys do not appear
   at all, not as JSON null. Specifically:
   - `mitre_techniques` (Vec<String>): key absent when the vec is empty
     (`skip_serializing_if = "Vec::is_empty"`); present as a JSON array when non-empty.
   - `source_ip` (Option<IpAddr>): key absent when None.
   - `timestamp` (Option<DateTime<Utc>>): key absent when None; present as RFC 3339 string when Some.
   - `direction` (Option<Direction>): key absent when None.
   There is no `mitre_technique_id` field and no `mitre_tactic` field in the Finding schema —
   those never existed.
4. The `summary` field in the JSON preserves raw bytes from the network data, including
   any non-ASCII or control byte sequences, without being cleaned up, escaped via terminal
   escaping, or truncated.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.09.001 | Postcondition 1 — Finding constructed with correct required and optional fields | Step 2: all required fields present in JSON |
| BC-2.09.005 | Postcondition 1 — summary and evidence carry RAW post-from_utf8_lossy bytes | Step 4: raw byte preservation in JSON output |
| BC-2.09.006 | Postcondition 1 — None Option / empty Vec fields omitted via skip_serializing_if | Step 3: absent keys for None source_ip/direction/timestamp; absent mitre_techniques key when vec empty; present when non-None/non-empty |

## Verification Approach

```
wirerust analyze --output-format json capture_with_anomalies.pcap | jq '.findings[0]'
```

Verify:
- Keys `category`, `verdict`, `confidence`, `summary` always present in every finding object.
- Key `mitre_techniques` is ABSENT when the vec is empty (e.g., cipher/protocol weakness
  findings); PRESENT as a JSON array (e.g., `["T1036"]`) when non-empty. No scalar
  `mitre_technique_id` or `mitre_tactic` key ever appears.
- Key `timestamp`: ABSENT for findings where timestamp is None (e.g., the segment-limit
  summary finding, BC-2.04.054); PRESENT as RFC 3339 string for flow-data-path findings
  that populate timestamp (O-01 is closed; timestamp is wired at the majority of emission sites).
  Use `jq 'has("timestamp")'` on individual findings to confirm per-finding presence/absence.
- For HTTP findings: key `source_ip` does NOT appear.
- For reassembly findings: key `source_ip` DOES appear with an IP address value.
- `summary` field: if the capture has non-ASCII SNI or URI data, that content appears
  verbatim in the `summary` field, not backslash-escaped in terminal style.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Required fields always present; optional fields
  absent when None, present when Some.
- **Data integrity** (weight: 0.35): Raw bytes in `summary` pass through to JSON without
  terminal-style escaping; JSON uses standard RFC 8259 escaping for C0 bytes, not backslash
  hex sequences.
- **Edge case handling** (weight: 0.1): A finding where all optional fields are Some produces
  a JSON object with all fields present.
- **Error quality** (weight: 0.1): JSON is valid according to RFC 8259 regardless of byte content.

## Edge Conditions

- A finding with a non-empty `mitre_techniques` vec (e.g., `["T1036"]`) should show the
  `mitre_techniques` key as a JSON array. A finding with `mitre_techniques = vec![]` must
  NOT show the `mitre_techniques` key at all. There is no `mitre_technique_id` or
  `mitre_tactic` key in the Finding schema.
- A finding where evidence is an empty vec should omit the evidence key or emit an empty
  array — confirm behavior matches the skip_serializing_if policy.
- Timestamp is populated at the majority of flow-data-path emission sites (O-01 is CLOSED
  as of STORY-097/098/099). The sole by-design timestamp=None case is the segment-limit
  summary finding (BC-2.04.054). A finding with timestamp=Some emits a `timestamp` key
  containing an RFC 3339 string; a finding with timestamp=None omits the key entirely.

## Failure Guidance

"HOLDOUT LOW: HS-007 (satisfaction: 0.XX) — JSON output includes null fields that should
be absent, or raw bytes in summary are modified by terminal-escape logic."
