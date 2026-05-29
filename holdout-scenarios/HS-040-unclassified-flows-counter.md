---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-033.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.007.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.009.md
input-hash: "90ffb3e"
traces_to: .factory/stories/STORY-033.md
id: "HS-040"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-3"
behavioral_contracts:
  - BC-2.05.007
  - BC-2.05.009
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Unclassified Flow Counter Accurately Reflects Coverage Gaps

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A forensic analyst analyzing a pcap wants to know how many TCP flows the
tool could not classify into a known protocol. Unclassified flows represent
a coverage gap — they received data but neither TLS nor HTTP fingerprints
were detected. The tool should report an accurate count of such flows.

1. A pcap contains 5 TCP flows:
   - 2 flows with valid HTTP traffic (correctly classified and analyzed)
   - 2 flows with valid TLS traffic (correctly classified and analyzed)
   - 1 flow on port 8888 with MQTT-like binary content (not TLS, not HTTP)
2. The user runs: `wirerust analyze <mixed-pcap> --output-format json`
3. The JSON output's statistics show `unclassified_flows: 1`.
4. The 4 classified flows receive full HTTP/TLS analysis.
5. The MQTT flow does not crash the tool and is silently counted as unclassified.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.007 | postcondition 1-2: unclassified_flows increments only at on_flow_close for None-routed flows | MQTT flow counted exactly once on close |
| BC-2.05.007 | invariant 1: classified flows (Http/Tls) do NOT increment unclassified_flows | HTTP and TLS flows not double-counted |
| BC-2.05.009 | postcondition 1-2: routes.remove and classification_attempts.remove both called unconditionally | Memory not leaked for the MQTT flow after close |
| BC-2.05.009 | postcondition 3-4: close forwarded to correct analyzer or unclassified counter | MQTT flow goes to unclassified counter; HTTP/TLS go to their analyzers |

## Verification Approach

```bash
wirerust analyze <mixed-pcap> --output-format json
```

Inspect the JSON output:
- `unclassified_flows` should be 1.
- HTTP statistics should reflect the 2 HTTP flows (2+ requests counted).
- TLS statistics should reflect the 2 TLS flows (handshakes counted, JA3 present).
- No crash or error for the MQTT flow.

Optionally run with a pcap containing only unclassified flows and verify
the counter matches the flow count.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): `unclassified_flows` count
  matches the actual number of unclassifiable flows.
- **Edge case handling** (weight: 0.2): Flows with no data sent (handshake-only,
  no payload) may also be counted as unclassified — this is a known limitation
  and should not crash.
- **Error quality** (weight: 0.1): No panic for unknown protocol flows.
- **Performance** (weight: 0.1): Normal throughput.
- **Data integrity** (weight: 0.1): Classified flow statistics accurate;
  unclassified counter does not overlap with classified counts.

## Edge Conditions

- A flow that was cached as permanent-None (retry budget exhausted) is also
  counted as unclassified on close.
- A dispatcher with no analyzers configured: `unclassified_flows` is NOT
  incremented (guard condition).
- A flow closed before any `on_data` calls: counted as unclassified if
  analyzers are configured.

## Failure Guidance

"HOLDOUT LOW: HS-040 (satisfaction: 0.XX) — the unclassified_flows counter
does not accurately reflect the number of flows that could not be classified;
either overcounting (classified flows counted) or undercounting (some None-routed
flows missed) was detected."
