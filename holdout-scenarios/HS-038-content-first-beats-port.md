---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-031.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.001.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.003.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-031.md
id: "HS-038"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-3"
behavioral_contracts:
  - BC-2.05.001
  - BC-2.05.002
  - BC-2.05.003
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: TLS on Non-Standard Port Is Detected by Content, Not Port

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A common evasion technique for encrypted traffic is to run TLS on port 80
(the standard HTTP port) to blend in with HTTP traffic. Similarly, an HTTP
server might run on port 443 (the standard HTTPS port). The tool must classify
these flows by their content, not by their port numbers.

1. **TLS on port 80:** A pcap contains a TCP flow on port 80 whose first bytes
   are `0x16 0x03` (a TLS ClientHello record header). The tool must route this
   to TLS analysis, not HTTP analysis.
2. **HTTP on port 443:** A second pcap contains a TCP flow on port 443 whose
   first bytes are `GET /index HTTP/1.1` (a plaintext HTTP request). The tool
   must route this to HTTP analysis, not TLS analysis.
3. **Unknown content, unknown port:** A third pcap contains a flow on port 9999
   with binary content that is neither TLS nor HTTP. The tool must not crash;
   it should note the flow as unclassified.
4. The user runs: `wirerust analyze <port-mismatch-pcap> --format json`
5. The tool completes with exit code 0 in all cases.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.001 | postcondition 1: 0x16/0x03 prefix routes to TLS regardless of port | TLS-on-80 classified as TLS |
| BC-2.05.001 | invariant 2: content-first over port | TLS wins over HTTP port heuristic |
| BC-2.05.002 | postcondition 1: HTTP method prefix routes to HTTP; case-sensitive; trailing space required | HTTP-on-443 classified as HTTP |
| BC-2.05.003 | postcondition 1-2: port fallback only when both content checks fail | Binary unknown content on port 9999 produces None |

## Verification Approach

Three pcap runs:

```bash
wirerust analyze <tls-on-80.pcap> --format json
wirerust analyze <http-on-443.pcap> --format json
wirerust analyze <unknown-port-9999.pcap> --format json
```

For TLS on port 80: verify TLS-level findings or JA3 fingerprints appear
in the output. No HTTP findings should appear for this flow.

For HTTP on port 443: verify HTTP method/status statistics appear in the
output. No TLS/JA3 findings should appear for this flow.

For unknown content on port 9999: verify no crash; the flow may appear in
unclassified_flows statistics.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): TLS-on-80 produces TLS output;
  HTTP-on-443 produces HTTP output.
- **Edge case handling** (weight: 0.2): Unknown content on unknown port produces
  a clean no-op, not a crash.
- **Error quality** (weight: 0.1): No error messages for expected edge cases.
- **Performance** (weight: 0.1): Normal runtime.
- **Data integrity** (weight: 0.1): No cross-contamination between TLS and HTTP
  analysis paths.

## Edge Conditions

- `data.len() < 5` for the TLS check: must fall through to HTTP check or port
  fallback without incorrect classification.
- `0x16 0x03` bytes but NOT valid TLS: still routed to TLS (loose gate); the
  TLS analyzer handles the parse error.
- `b"GET"` without trailing space: does NOT match HTTP; falls to port fallback.
- Uppercase `GET ` vs lowercase `get `: case-sensitive; only uppercase matches.

## Failure Guidance

"HOLDOUT LOW: HS-038 (satisfaction: 0.XX) — content-first classification
failed; TLS on port 80 was misclassified as HTTP or HTTP on port 443 was
misclassified as TLS, indicating port-based heuristics overrode content inspection."
