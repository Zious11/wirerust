---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-020.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.014.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-020.md
id: "HS-044"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.014
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Memory Accounting Stays Consistent Across Insert, Flush, and Close

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

After processing a complex pcap with many flows — some of which are evicted,
some closed cleanly, some timed out — the tool's internal memory accounting
should remain accurate. A memory accounting bug could manifest as `total_memory`
gradually drifting from the actual sum of buffered bytes across all active flows.

1. A pcap contains 20 concurrent TCP flows with interleaved traffic, some of
   which are evicted by the memory cap, some closed by RST, some by FIN, and
   some timed out at end-of-capture.
2. The user runs: `wirerust analyze <complex-flows-pcap> --output-format json`
3. The tool completes with exit code 0.
4. No internal consistency assertion fails (no debug-mode panic from
   `memory_used()` mismatch).
5. After finalize, `total_memory` reaches 0 (all flows have been closed and
   their memory accounted for).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.014 | postcondition 1: total_memory increments on insert | Memory tracked upward correctly |
| BC-2.04.014 | postcondition 2: total_memory decrements on flush | Flushed bytes removed from accounting |
| BC-2.04.014 | postcondition 3: total_memory decrements on close by flow's memory_used() | All buffered bytes accounted for at close |
| BC-2.04.014 | postcondition 4 and invariant 2: total_memory == sum of flow.memory_used() at all times | No drift between global and per-flow accounting |

## Verification Approach

```bash
wirerust analyze <complex-flows-pcap> --output-format json
```

The main observable here is that the tool completes without panic. In debug
builds, the `memory_used()` assertion fires if `total_memory` diverges from
the sum of per-flow `buffered_bytes`. Run the debug build:

```bash
cargo test --no-run && target/debug/wirerust analyze <complex-flows-pcap>
```

In release mode: verify no `total_memory` overflow (which would manifest as
an unexpected memory allocation failure) and that the tool's RSS memory does
not grow unboundedly during processing.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Tool completes without panic or OOM.
- **Edge case handling** (weight: 0.3): Memory accounting holds even when eviction,
  RST, FIN, and timeout close all occur in the same run.
- **Error quality** (weight: 0.2): No debug assertion failures; no memory leak.
- **Performance** (weight: 0.05): Normal throughput.
- **Data integrity** (weight: 0.05): After finalize, no active flows remain;
  `total_memory` logically returns to 0.

## Edge Conditions

- Zero-length segments: `total_memory` unchanged (early return before accounting).
- Duplicate retransmissions: `total_memory` unchanged (no new bytes stored).
- Flow eviction: `total_memory` decremented by evicted flow's `memory_used()`.
- All flows evicted but `total_memory` still over memcap: engine continues
  processing; `total_memory` stays elevated until more flows are evicted.

## Failure Guidance

"HOLDOUT LOW: HS-044 (satisfaction: 0.XX) — memory accounting inconsistency
detected; `total_memory` drifted from the actual sum of per-flow buffered bytes,
indicating an insert/flush/close path does not update the aggregate correctly."
