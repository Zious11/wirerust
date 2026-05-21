---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-021.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.012.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-021.md
id: "HS-037"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.012
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: End-of-PCAP Finalizes All Open Flows Without Duplication

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

At the end of a pcap, some TCP flows may not have been cleanly closed (no RST
or FIN seen). The tool must finalize these flows, flushing any remaining buffered
data to the protocol analyzers and producing a clean, complete output.

1. A pcap contains 5 TCP flows that transfer data but have no RST or FIN at
   the end — the capture ended before the connections closed naturally. The
   flows have residual bytes buffered in the reassembly engine.
2. The user runs: `wirerust analyze <open-flows-pcap> --output-format json`
3. The tool completes with exit code 0.
4. All 5 flows' data is delivered to the protocol analyzers before the output
   is produced — there are no "phantom" bytes that get silently dropped.
5. The finalize step runs exactly once — a second invocation (if it could be
   triggered) would be a complete no-op.
6. Each flow's remaining buffered bytes appear in `bytes_reassembled` in the
   statistics.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.012 | postcondition 1-2: finalize closes all remaining flows; self.flows is empty after | All open flows flushed at end of capture |
| BC-2.04.012 | postcondition 5 and invariant 1: second finalize call is complete no-op | Idempotency prevents double-counting |

## Verification Approach

```bash
wirerust analyze <open-flows-pcap> --output-format json
```

Verify:
- Exit code is 0.
- Any protocol-level analysis (HTTP, TLS) from the truncated flows appears in output.
- `bytes_reassembled` is non-zero and accounts for data from all 5 flows.
- If running the tool twice on the same pcap, the output is identical (deterministic
  finalization with no ordering artifacts).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): All open flows produce their final
  protocol-level findings even without RST/FIN closure.
- **Edge case handling** (weight: 0.2): A pcap with zero open flows at end
  (all closed cleanly) also works correctly.
- **Error quality** (weight: 0.1): No warnings or errors about "unflushed data."
- **Performance** (weight: 0.1): Finalization does not significantly extend runtime.
- **Data integrity** (weight: 0.1): Statistics reflect all flows (open and closed).

## Edge Conditions

- A pcap with no TCP flows at all: finalize is a no-op; output is empty or
  just metadata.
- A flow that had all its data delivered (empty buffer at finalize): finalize
  closes it cleanly with nothing to flush.
- A flow with out-of-order buffered segments that never had their gap filled:
  finalize delivers the buffered portions up to the gap, then closes.

## Failure Guidance

"HOLDOUT LOW: HS-037 (satisfaction: 0.XX) — the tool dropped buffered data
from TCP flows that were open at end-of-capture; protocol-level findings
were missing for flows without RST/FIN closure."
