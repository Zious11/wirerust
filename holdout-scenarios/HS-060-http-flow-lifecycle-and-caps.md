---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs: [stories/, behavioral-contracts/, prd.md]
input-hash: "[md5-pending]"
traces_to: ".factory/specs/prd.md"
id: "HS-060"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-4"
behavioral_contracts:
  - BC-2.06.019
  - BC-2.06.021
  - BC-2.06.022
  - BC-2.06.024
  - BC-2.06.025
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: HTTP Flow Close Resets Per-Flow State Without Affecting Aggregate Counters

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains an HTTP flow that is poisoned (request direction accumulates 3 parse errors) and then closed (TCP RST or FIN).
2. After the close, the same 5-tuple (same source/destination IP and port pair) reappears in the pcap with a valid HTTP request.
3. The analyst runs wirerust on this pcap.
4. The second HTTP request on the same 5-tuple succeeds and is counted in the method map — the poisoning from the first flow does not carry over.
5. The aggregate `non_http_flows` counter reflects the poisoned first flow (value 1) but does not double-count on the second flow (which sends valid HTTP).
6. A separate assertion: the HTTP buffer cap of 65,536 bytes is enforced — sending 100 KB to a single direction does not cause a panic or corrupt state.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.06.019 | postcondition 1-4; invariant 2 | on_flow_close drops per-flow state; same FlowKey reopens fresh |
| BC-2.06.021 | postcondition 1-3; invariant 1-2 | Cross-flow isolation: first flow's poison does not affect second flow |
| BC-2.06.022 | postcondition 1-4; invariant 1-3 | Per-direction buffer cap 65,536; bytes past cap silently dropped; no panic |
| BC-2.06.024 | postcondition 1-4; invariant 2-3 | Map cardinality cap: new unique keys silently dropped at 50,000; existing keys still increment |
| BC-2.06.025 | postcondition 1-3; invariant 1-3 | uris Vec capped at 10,000; no panic; subsequent URIs dropped |

## Verification Approach

Test the flow-lifecycle behavior against a pcap or synthetic test vectors.

1. Process a poisoned flow (3 binary blobs on request direction). Assert `non_http_flows == 1`.
2. Call flow_close for that FlowKey. Assert the flow entry is removed from internal state.
3. Send a valid HTTP GET on the same FlowKey. Assert `methods["GET"] == 1` (fresh state; poison not carried over).
4. Assert `non_http_flows` is still 1 (not 2 — the second flow sends valid HTTP and is not poisoned).
5. For buffer cap: send 65,537 bytes to a single direction. Assert no panic and `client_buf.len() == 65,536`.
6. For uris cap: send 10,001 distinct-URI requests. Assert `recent_uris` in summary shows exactly 20 (first 20); no panic.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Flow close resets per-flow state; aggregate counters preserved; same FlowKey reopens fresh.
- **Edge case handling** (weight: 0.3): Buffer cap does not panic on 100 KB input; URI cap drops silently; map cap allows existing keys to increment.
- **Error quality** (weight: 0.15): No panic or undefined behavior at any cap boundary.
- **Data integrity** (weight: 0.1): on_flow_close ignores close reason (it is a no-op parameter).

## Edge Conditions

- Same 5-tuple appearing twice in a pcap (flow reuse after close) — second flow must start with clean state.
- Buffer at exactly 65,535: next 1-byte on_data appends 1 byte, not 0 bytes.
- uris Vec at 9,999: next request appends (len becomes 10,000); at 10,000 the next request drops the URI.
- Map at 50,000 keys: existing keys still increment (contains_key short-circuit).

## Failure Guidance

"HOLDOUT LOW: HS-060 (satisfaction: 0.XX) -- HTTP flow lifecycle or cap behavior incorrect; check that on_flow_close resets per-flow state, that reopened flows start fresh, and that buffer/map/uris caps do not panic."
