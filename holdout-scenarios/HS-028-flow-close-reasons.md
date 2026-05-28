---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-019.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.010.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.011.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.013.md
input-hash: "decf4d6"
traces_to: .factory/stories/STORY-019.md
id: "HS-028"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.010
  - BC-2.04.011
  - BC-2.04.013
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Flow Close Semantics — RST Skips Payload, FIN Delivers Payload First

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A forensic analyst is examining a pcap that contains two TCP connections:
one that ends with a RST packet carrying a payload, and one that ends with
a FIN packet also carrying a payload.

1. **RST connection:** A TCP flow exchanges data, then one side sends a RST
   packet with a 50-byte payload appended. The tool processes this pcap.
   The RST payload should NOT be delivered to the protocol analyzer — RST
   terminates the flow immediately, discarding any accompanying payload.

2. **FIN connection:** A second TCP flow exchanges data and ends with a FIN
   packet that carries a 50-byte payload. The tool processes this pcap.
   The FIN payload SHOULD be delivered to the protocol analyzer before the
   flow is considered closed. The close event fires after the payload delivery.

3. The user runs: `wirerust analyze <rst-and-fin-pcap> --output-format json`

4. The tool completes with exit code 0. The output reflects the differing
   semantics: RST truncates immediately (payload lost), FIN includes the
   last payload.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.010 | postcondition 6 and invariant 3: RST payload not processed | RST-carrying-payload scenario |
| BC-2.04.011 | invariant 2: FIN payload processed before flow close | FIN-carrying-payload scenario |
| BC-2.04.013 | postcondition 1-2: expire_flows correctly identifies idle flows | Idle timeout flows are cleaned up correctly |

## Verification Approach

Run against a pcap with RST and FIN flows. Alternatively, construct two small
synthetic pcaps and run the tool on each:

```bash
wirerust analyze <rst-payload-pcap> --output-format json
wirerust analyze <fin-payload-pcap> --output-format json
```

For the RST pcap: verify the `bytes_reassembled` count does NOT include the
50 bytes from the RST packet.

For the FIN pcap: verify the `bytes_reassembled` count DOES include the 50
bytes from the FIN packet, and any protocol-level analysis (e.g., HTTP)
correctly sees that final payload.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): RST payload is excluded;
  FIN payload is included before close.
- **Edge case handling** (weight: 0.3): RST can arrive from any flow state
  (New, SynSent, Established, Closing) and must always terminate the flow.
- **Error quality** (weight: 0.1): No crash; graceful handling of both cases.
- **Performance** (weight: 0.05): Normal throughput.
- **Data integrity** (weight: 0.05): Statistics accurately reflect which bytes
  were actually reassembled.

## Edge Conditions

- RST on a brand-new flow (before handshake completes): flow removed, no data.
- Simultaneous RST and FIN bits in the same packet: RST semantics win.
- FIN retransmit (same FIN seen twice from same direction): second FIN should
  still lead to closure after both sides have FINed.
- Idle timeout: a flow that saw no traffic for longer than the timeout threshold
  should be cleaned up in a finalize pass.

## Failure Guidance

"HOLDOUT LOW: HS-028 (satisfaction: 0.XX) — TCP RST/FIN close semantics are
incorrect; RST payload was delivered or FIN payload was discarded before close."
