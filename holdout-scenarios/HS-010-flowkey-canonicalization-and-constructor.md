---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-011.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.001.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.003.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.049.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
id: "HS-010"
category: "integration-boundaries"
must_pass: "true"
priority: "must-pass"
epic_id: "E-2"
behavioral_contracts:
  - BC-2.04.001
  - BC-2.04.003
  - BC-2.04.049
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: FlowKey Symmetry — Bidirectional Packets Merge into One Flow

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A user has a pcap of a normal HTTP session: client 10.0.0.1:54321 connects to
   server 10.0.0.2:80. The capture contains packets going in both directions.
2. wirerust processes the capture and shows a single flow entry for this connection,
   not two separate flows (one per direction).
3. The analyst sees the same flow key regardless of which packet direction they observe,
   confirming that A->B and B->A are treated as the same connection.
4. A second user intentionally tries to configure wirerust with obviously broken settings
   (e.g., via crafted config that would set depth to 0). The tool panics with a clear
   error message at startup — not silently during analysis.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.04.001 | Postcondition 1 — TcpReassembler::new panics on zero-valued config fields | Step 4: loud failure at construction, not silent mid-analysis |
| BC-2.04.003 | Postcondition 1 — FlowKey::new is commutative: A->B and B->A produce identical key | Steps 2-3: bidirectional packets share one flow entry |
| BC-2.04.049 | Postcondition 1 — FlowKey::Display uses U+2192 arrow, not ASCII -> | Steps 2-3: flow key display format |

## Verification Approach

```
wirerust analyze --output-format json bidirectional_http.pcap
```

Inspect the JSON summary: there should be one flow entry in reassembly stats for each
distinct TCP connection (not two per connection). The reassembly detail map should show
a consistent flow count.

For configuration validation: wirerust currently reads config from CLI flags; test the
`--depth 0` flag to confirm the tool panics or errors at startup, not mid-file.

```
wirerust analyze --depth 0 bidirectional_http.pcap
```

Expect: immediate panic or error before processing packets.

If terminal output shows flow keys, verify they contain the Unicode right-arrow (→)
rather than ASCII `->`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Bidirectional TCP traffic produces one flow
  per connection, not two; config validation panics at construction time.
- **Data integrity** (weight: 0.3): Flow count in reassembly stats equals number of distinct
  TCP connections, not number of packet directions.
- **Edge case handling** (weight: 0.15): Zero-config fields trigger panic before any file
  I/O begins.
- **Error quality** (weight: 0.1): Panic message identifies which config field is invalid.

## Edge Conditions

- If client and server happen to use IPs that compare equal (loopback traffic), the
  canonicalization must still work (same IP, different ports).
- A connection where client port > server port (e.g., 54321 > 80) must canonicalize the
  same way as server port > client port.
- IPv6 bidirectional flows must also canonicalize correctly (IPv6 comparison is byte-by-byte).

## Failure Guidance

"HOLDOUT LOW: HS-010 (satisfaction: 0.XX) — bidirectional TCP traffic creates two flow
entries instead of one, or zero-config fields don't trigger panic at construction time."
