---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-032.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.005.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.006.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-032.md
id: "HS-039"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-3"
behavioral_contracts:
  - BC-2.05.005
  - BC-2.05.006
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Classification Cache Is Immutable and Retry Budget Eventual

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A long-running TCP flow begins with several data chunks that have
ambiguous content (SSH-style binary that is neither TLS nor HTTP), then
later in the flow sends a recognizable HTTP GET request. The dispatcher
must eventually classify it as HTTP and stick with that classification,
even if later TLS bytes arrive.

1. A pcap contains one TCP flow that sends 4 chunks of SSH-like binary
   data (no match), followed by a chunk starting with `GET /api HTTP/1.1`.
   The flow continues with more data including some bytes that happen to
   start with `0x16 0x03` (which look like TLS).
2. The user runs: `wirerust analyze <late-classify-pcap> --output-format json`
3. The tool correctly classifies the flow as HTTP when the GET request arrives
   (within the retry budget).
4. After HTTP classification, the flow stays classified as HTTP — later TLS-like
   bytes do NOT reclassify it as TLS.
5. HTTP-level analysis (method, status codes, headers) is correctly performed
   on this flow's content.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.005 | postcondition 4: cached Http is immutable; cannot be reclassified as Tls | Late TLS bytes don't override HTTP classification |
| BC-2.05.005 | invariant 1: Http/Tls cached on first non-None result | HTTP cached after GET chunk arrives |
| BC-2.05.006 | postcondition Phase A: None not cached before retry cap; each chunk re-runs classify | First 4 SSH-like chunks re-run classify without caching None |
| BC-2.05.006 | postcondition Phase B: permanent None after cap if no match | If flow never matched, None cached after `max_classification_attempts` |

## Verification Approach

```bash
wirerust analyze <late-classify-pcap> --output-format json
```

Verify:
- HTTP statistics appear for the flow (at minimum: `requests` count > 0).
- No TLS findings for this flow (TLS bytes after HTTP classification are
  forwarded to the HTTP analyzer, not the TLS analyzer).
- Exit code is 0.

For the permanent-None path: use a pcap where a flow on port 9999 sends
only SSH-like binary forever with max_classification_attempts exhausted.
Verify the flow's `unclassified_flows` counter increments on close.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Late HTTP classification works;
  subsequent TLS bytes do not reclassify the flow.
- **Edge case handling** (weight: 0.2): Retry budget exhaustion produces a
  permanent None classification; flow is counted as unclassified.
- **Error quality** (weight: 0.1): No crash from any classification state.
- **Performance** (weight: 0.1): Retry logic doesn't cause N-squared overhead
  for long unclassified flows.
- **Data integrity** (weight: 0.1): Classification state and cache are consistent
  for the flow's lifetime.

## Edge Conditions

- `max_classification_attempts = 0`: every flow immediately gets permanent None.
- Same flow key reused after close: new flow starts with a clean cache.
- Flow with HTTP cached; analyzer is None: data forwarded to a None analyzer
  silently (no panic).

## Failure Guidance

"HOLDOUT LOW: HS-039 (satisfaction: 0.XX) — classification cache mutation
allowed a cached HTTP flow to be reclassified as TLS by later data chunks,
or late HTTP classification failed within the retry budget."
