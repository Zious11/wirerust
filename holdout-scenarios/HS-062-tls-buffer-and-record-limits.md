---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-051.md
  - .factory/stories/STORY-052.md
  - .factory/stories/STORY-053.md
  - .factory/stories/STORY-054.md
  - .factory/stories/STORY-055.md
  - .factory/stories/STORY-056.md
  - .factory/stories/STORY-057.md
  - .factory/stories/STORY-058.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.004.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.005.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.029.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.033.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.035.md
input-hash: "6e52bc5"
traces_to: .factory/stories/STORY-051.md
id: "HS-062"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.004
  - BC-2.07.005
  - BC-2.07.029
  - BC-2.07.033
  - BC-2.07.035
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: TLS Oversized Records and Buffer Cap Enforced Without Panic

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A crafted network session contains:
   - A valid TLS ClientHello (handshake record, type 0x16, payload = 200 bytes).
   - A TLS record claiming a payload of 20,000 bytes (above the 18,432 byte limit).
   - A ChangeCipherSpec record (type 0x14, non-handshake).
   - An Alert record (type 0x15, non-handshake).
   - A valid TLS ServerHello (handshake, type 0x16).
2. The analyst runs wirerust on a pcap of this session.
3. Expected outcomes:
   - `handshakes_seen == 1` (ClientHello only; ServerHello arrives AFTER the oversized record clears the buffer, so it may not be parsed — but the scenario tests that the ClientHello WAS counted).
   - `parse_errors >= 1` and `truncated_records >= 1` (from the oversized record).
   - `truncated_records` and `parse_errors` were both incremented exactly once for the oversized record.
   - No panic occurred.
   - ChangeCipherSpec and Alert records were consumed silently (non-handshake skip path).
   - `parse_errors` was NOT incremented for the non-handshake records.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.004 | postcondition 1-4; invariant 1-2 | Oversized record increments parse_errors AND truncated_records together; buffer cleared |
| BC-2.07.005 | postcondition 1-4; invariant 1-3 | Per-direction buffer capped at 65,536; silent drop beyond cap; no counter increment |
| BC-2.07.029 | postcondition 1-5; invariant 1-2 | Bad TLS record body increments parse_errors only; no panic; flow preserved |
| BC-2.07.033 | postcondition 1-4; invariant 1-2 | Non-handshake records (0x14, 0x15) consumed silently; loop continues |
| BC-2.07.035 | postcondition 1-4; invariant 1-2 | on_flow_close drops per-flow TlsFlowState; aggregate counters unaffected |

## Verification Approach

Use a crafted pcap or synthetic byte stream. Run wirerust.

1. Assert `analyzers[TLS].packets_analyzed >= 1` (ClientHello was processed before the oversized record).
2. Assert `analyzers[TLS].detail.truncated_records >= 1`.
3. Assert `analyzers[TLS].detail.parse_errors >= 1`.
4. Verify that `parse_errors == truncated_records` for the oversized record (they are always incremented together).
5. Assert no finding in `findings` array sourced from the non-handshake record types.
6. Assert `analyzers[TLS].detail.parse_errors` was NOT incremented for the ChangeCipherSpec or Alert records.
7. Confirm the tool exits with status 0 (no panic/crash).

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): parse_errors and truncated_records both incremented for oversized record; non-handshake records silently consumed.
- **Edge case handling** (weight: 0.3): No panic at oversized record or buffer cap; both counters incremented atomically.
- **Error quality** (weight: 0.15): parse_errors not incremented for non-handshake record types.
- **Data integrity** (weight: 0.1): truncated_records key always present in summary even when 0.

## Edge Conditions

- Oversized record boundary: payload_len == 18,432 is accepted; payload_len == 18,433 triggers both counters.
- Non-handshake records of unknown type (e.g., 0x18) must also be consumed silently, not counted as errors.
- Buffer cap of 65,536 must not panic when more data arrives after the cap is reached.

## Failure Guidance

"HOLDOUT LOW: HS-062 (satisfaction: 0.XX) -- TLS buffer/record limits were not enforced; check that oversized records increment both parse_errors and truncated_records together, non-handshake records are silently skipped, and no panic occurs."
