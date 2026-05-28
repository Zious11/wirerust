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
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.001.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.012.md
  - .factory/specs/behavioral-contracts/ss-06/BC-2.06.023.md
input-hash: "6151176"
traces_to: .factory/stories/STORY-041.md
id: "HS-067"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.001
  - BC-2.06.004
  - BC-2.06.012
  - BC-2.06.023
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Known-Good HTTP Traffic Corpus Produces Zero False-Positive Findings

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap of clean, well-formed HTTP/1.1 traffic from the Wireshark sample captures collection (specifically `http.cap` or `http-wireshark-list.pcap` from the Wireshark wiki) is analyzed.
2. The traffic consists of standard browser-style HTTP GET requests to legitimate hostnames, with standard methods (GET, POST), well-formed Host headers, standard URI lengths, and standard User-Agents.
3. The analyst runs wirerust on this pcap.
4. Expected: zero HTTP-sourced findings (no path traversal, no web-shell, no admin-panel, no unusual method, no missing-host, no long-URI, no empty-UA findings). The `transactions` counter reflects the number of HTTP responses. The `methods` map has only standard method keys (GET, HEAD, POST). The `parse_errors` counter is 0.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.001 | postcondition 1-7 | Standard requests parsed correctly; all stats accumulated |
| BC-2.06.004 | postcondition 1-4; invariant 1 | Responses counted in transactions; requests do not inflate transactions |
| BC-2.06.012 | postcondition 1-3; invariant 1 | Well-formed requests produce zero findings |
| BC-2.06.023 | postcondition 1-3; invariant 1-4 | HTTP summary is complete; methods only contain standard values |

## Verification Approach

corpus_source: Wireshark sample captures — `http.cap` (https://wiki.wireshark.org/SampleCaptures)
corpus_size: ~4,400 packets, ~730 KB
known_edge_cases: Mix of HTTP/1.0 and HTTP/1.1; some responses without content-length; chunked encoding (not parsed at header level)

Run wirerust on `http.cap` with JSON output.

1. Assert `findings` array contains 0 HTTP-sourced findings (from analyzer HTTP).
2. Assert `analyzers[HTTP].detail.parse_errors == 0`.
3. Assert `analyzers[HTTP].detail.non_http_flows == "0"`.
4. Assert `analyzers[HTTP].detail.methods` keys are all standard HTTP methods.
5. Assert `analyzers[HTTP].detail.transactions` is a positive integer matching the number of HTTP responses in the capture.
6. Assert wirerust exits with status 0.

false_positive_threshold: 0 (zero tolerance for false positives on known-good traffic)

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Zero findings for known-good traffic; parse_errors=0; non_http_flows=0.
- **Edge case handling** (weight: 0.2): HTTP/1.0 requests without Host header do not trigger missing-Host finding (HTTP/1.0 exempt).
- **Error quality** (weight: 0.2): Tool exits cleanly; no panics on real-world traffic.
- **Data integrity** (weight: 0.1): methods map contains only expected HTTP methods; transactions is positive.

## Edge Conditions

- HTTP/1.0 requests without Host header are present in the corpus — the Host-check exemption for version==0 must work.
- The corpus may contain pipelined responses — these should be counted correctly in transactions.

## Failure Guidance

"HOLDOUT LOW: HS-067 (satisfaction: 0.XX) -- HTTP analyzer produced false-positive findings on known-good traffic corpus; check URI detection patterns, Host exemption for HTTP/1.0, or parse error handling."
