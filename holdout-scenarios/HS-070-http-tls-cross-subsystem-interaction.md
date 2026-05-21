---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-041.md
  - .factory/stories/STORY-042.md
  - .factory/stories/STORY-043.md
  - .factory/stories/STORY-044.md
  - .factory/stories/STORY-045.md
  - .factory/stories/STORY-046.md
  - .factory/stories/STORY-051.md
  - .factory/stories/STORY-052.md
  - .factory/stories/STORY-053.md
  - .factory/stories/STORY-054.md
  - .factory/stories/STORY-055.md
  - .factory/stories/STORY-056.md
  - .factory/stories/STORY-057.md
  - .factory/stories/STORY-058.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.001.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.013.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.001.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.030.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-041.md
id: "HS-070"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.001
  - BC-2.06.013
  - BC-2.07.001
  - BC-2.07.030
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: HTTP and TLS Analyzers Operate Independently on Same pcap Without Cross-Contamination

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains: (a) a plain HTTP/1.1 flow on port 80 with normal traffic, and (b) a TLS 1.3 flow on port 443.
2. The HTTP analyzer receives the port-80 traffic and parses it normally. The TLS analyzer receives the port-443 traffic via content-first dispatch. Neither analyzer sees the other flow's bytes.
3. The HTTP analyzer on the TLS flow: the TLS flow's ClientHello bytes look like binary garbage to an HTTP parser. After 3 consecutive failures, the HTTP analyzer marks this as a non-HTTP flow.
4. Wait — if the content-first dispatcher correctly routes the TLS flow to the TLS analyzer only, the HTTP analyzer should NEVER receive TLS bytes. This scenario tests that the routing is correct: the `non_http_flows` counter is 0, not 1.
5. The analyst runs wirerust on this pcap.
6. Expected: HTTP summary shows only port-80 traffic. TLS summary shows only port-443 traffic. `non_http_flows == 0`. TLS-sourced findings do not appear in the HTTP section.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.001 | postcondition 1-7 | HTTP analyzer correctly processes plain HTTP flow |
| BC-2.06.013 | postcondition 1-5 | If TLS bytes somehow reach HTTP analyzer, they increment parse_errors; this BC is the failure mode being avoided |
| BC-2.07.001 | postcondition 1-8 | TLS analyzer correctly processes TLS flow via content-first dispatch |
| BC-2.07.030 | postcondition 1-4 | Modern TLS flow produces zero TLS findings |

## Verification Approach

Use a pcap with one HTTP (port 80) and one HTTPS (port 443) flow. Run wirerust.

1. Assert `findings` array contains zero findings from either flow (both are well-formed traffic).
2. Assert `analyzers[HTTP].detail.non_http_flows == "0"` — TLS traffic was not dispatched to HTTP analyzer.
3. Assert `analyzers[HTTP].detail.parse_errors == "0"` — no TLS bytes leaked to HTTP parser.
4. Assert `analyzers[TLS].packets_analyzed >= 1` — TLS handshake was parsed.
5. Assert two distinct analyzer entries in the `analyzers` array: one named "HTTP" and one named "TLS".
6. Assert HTTP `transactions` reflects only the port-80 HTTP responses.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): HTTP and TLS analyzers process disjoint traffic; non_http_flows=0 because TLS traffic was routed correctly.
- **Edge case handling** (weight: 0.3): This scenario validates the content-first dispatch boundary — TLS bytes must never reach the HTTP parser in a correctly configured pipeline.
- **Error quality** (weight: 0.1): No cross-subsystem findings; zero parse_errors in both analyzers.
- **Data integrity** (weight: 0.1): Two distinct named analyzers in the output; HTTP and TLS stats are independent.

## Edge Conditions

- If content-first dispatch routes incorrectly (e.g., falls back to port-based dispatch), TLS ClientHello bytes would reach the HTTP analyzer, produce parse errors, and eventually poison the direction. This is the failure mode being tested.
- Port 443 with content-first dispatch should see the TLS signature (`\x16\x03`) and route to TLS analyzer without HTTP ever seeing those bytes.

## Failure Guidance

"HOLDOUT LOW: HS-070 (satisfaction: 0.XX) -- HTTP and TLS analyzers were not correctly isolated; TLS bytes leaked to HTTP analyzer (non_http_flows > 0) or HTTP bytes leaked to TLS analyzer (wrong handshakes_seen count)."
