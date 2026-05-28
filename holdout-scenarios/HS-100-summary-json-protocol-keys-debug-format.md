---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-086.md
  - .factory/stories/STORY-087.md
  - .factory/stories/STORY-088.md
  - .factory/stories/STORY-089.md
  - .factory/stories/STORY-090.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.018.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.019.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.021.md
input-hash: "db27506"
traces_to: .factory/stories/STORY-086.md
id: "HS-100"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.021
  - BC-2.12.018
  - BC-2.12.019
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: JSON Summary Uses Debug-Format Protocol Keys (CamelCase, Not Uppercase)

## Scenario

The JSON output's `protocols` map uses Rust Debug-format variant names for protocol keys.
This means `"Tcp"` not `"TCP"`, `"Udp"` not `"UDP"`, and `"Icmp"` not `"ICMP"`. A downstream
tool that parses the JSON and expects uppercase keys will break.

1. The tool is run against a pcap containing TCP, UDP, and ICMP packets.
2. The JSON output is parsed.
3. The `summary.protocols` object contains keys `"Tcp"`, `"Udp"`, and `"Icmp"` (CamelCase).
4. The keys `"TCP"`, `"UDP"`, `"ICMP"` (uppercase) do NOT appear.
5. The values (packet counts) for each key are positive integers reflecting the actual counts.

Additionally, the `summary.services` map uses uppercase service name strings
(e.g., `"HTTP"`, `"DNS"`, `"HTTPS"`) because these are derived from the port-to-name
table, not from Rust enum Debug format. The case convention differs between `protocols`
and `services`.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.021 | Postcondition 7: protocol keys use Debug format | Items 1-5: CamelCase keys not uppercase |
| BC-2.12.021 | Invariant 2: `{k:?}` Debug format for protocol keys | Explicit format contract |
| BC-2.12.018 | Postcondition 4: protocol map updated by ingest | Protocol counts accumulate correctly |
| BC-2.12.019 | Postcondition 1: service hints from port | Services map uses port-based names |

## Verification Approach

1. Run `wirerust analyze --json <pcap-with-tcp-udp-icmp>`.
2. Parse the JSON output.
3. Assert: `json["summary"]["protocols"]` has key `"Tcp"` (not `"TCP"`).
4. Assert: `json["summary"]["protocols"]` has key `"Udp"` (not `"UDP"`), if UDP traffic present.
5. Assert: `json["summary"]["protocols"]` does NOT contain key `"TCP"` or `"UDP"` or `"ICMP"`.
6. Assert: the values associated with each protocol key are positive integers matching packet counts.

For services:
7. Assert: `json["summary"]["services"]` (if present) uses uppercase names like `"HTTP"`, `"DNS"`.
8. Assert: the services map does NOT use CamelCase like `"Http"` or `"Dns"`.

At the unit level, construct a `Summary`, ingest packets of known protocols, and verify the
`protocol_counts()` accessor returns the correct keys.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Protocol keys are CamelCase Debug format; service keys are uppercase; counts are correct.
- **Edge case handling** (weight: 0.2): Protocol::Other is also in Debug format if present; no special-casing.
- **Error quality** (weight: 0.1): JSON is valid; all counts are non-negative integers.
- **Performance** (weight: 0.05): Protocol accumulation is O(1) per packet via HashMap.
- **Data integrity** (weight: 0.15): Protocol and service counts match actual traffic; no over-counting.

## Edge Conditions

- A pcap with only TCP traffic: only `"Tcp"` appears in protocols map (no `"Udp"` or `"Icmp"`).
- A pcap with `Protocol::Other`: appears as `"Other"` (Debug format) in the protocols map.
- Empty protocols map (no packets decoded): `"protocols": {}` — not `"protocols": null`.
- Port not in the service table: service key absent from services map (not `""` or null).

## Failure Guidance

"HOLDOUT LOW: HS-100 (satisfaction: 0.XX) -- JSON summary protocol keys used uppercase format ('TCP', 'UDP') instead of Rust Debug CamelCase ('Tcp', 'Udp'), breaking downstream parsers that rely on the documented format."
