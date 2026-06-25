---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.018.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.016.md
  - .factory/stories/STORY-137.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-117"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.018
  - BC-2.17.016
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcap with 3+ malformed ENIP frames (unknown command codes) in one TCP flow on port 44818. See byte layout in scenario body."
---

# Holdout Scenario: Malformed ENIP Frame Burst (>=3) Emits T0814 Structural Anomaly

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

A burst of malformed ENIP frames on port 44818 triggers the structural anomaly detection
(T0814). The malformed-frame counter (`malformed_in_window`) is incremented for each
structural-reject event: an ENIP header whose command is not in the known-valid set fails
the `is_valid_enip_frame` check and is rejected byte-by-byte. When `malformed_in_window`
reaches the MALFORMED_ANOMALY_THRESHOLD (= 3), a T0814 finding is emitted once per window
(one-shot guard via `malformed_anomaly_emitted`).

"Malformed" here means structural invalidity at the ENIP encapsulation layer — an unknown
or invalid command code causes the validity gate to reject the header.

### Case A — 3 Malformed Frames Triggers T0814 (Threshold = 3)

1. A crafted PCAP is presented: one TCP flow on port 44818; THREE ENIP frames, each with
   a 24-byte header carrying an unknown command code (e.g., command=0xFF00 — not in the
   known-valid set of 0x0063/0x0064/0x0065/0x0066/0x006F/0x0070). All three are rejected
   by the validity gate.
2. The user runs: `wirerust analyze enip_malformed_3.pcap --enip --json`
3. The tool exits 0 (no panic — malformed frames must not cause a crash).
4. The evaluator confirms: EXACTLY ONE finding with `"mitre_techniques": ["T0814"]`.
5. The finding summary references "structural anomaly", "malformed", "crash-probe", or
   similar. Evaluator does not require exact wording.

### Case B — 2 Malformed Frames: No T0814 Finding (Threshold Not Yet Reached)

1. A second crafted PCAP: same structure but only TWO malformed ENIP frames.
2. The user runs: `wirerust analyze enip_malformed_2.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: ZERO T0814 findings (2 is less than threshold 3; threshold
   requires `malformed_in_window >= 3`, i.e., reaching 3, not exceeding it).

Note: The threshold semantics for T0814 is "malformed_in_window >= 3" (fires ON the 3rd),
distinct from the strict-greater-than of write-burst (fires on the 51st) and error-burst
(fires on the 6th). Evaluator must verify exactly which count triggers the first firing.

### Case C — 4 Malformed Frames: Still Exactly One T0814 (One-Shot Guard)

1. A third PCAP: FOUR malformed frames in the same flow/window.
2. The evaluator confirms: EXACTLY ONE T0814 finding (the one-shot guard fires after the
   3rd malformed frame; the 4th does not produce an additional T0814 finding).

### Case D — Malformed Frames Do Not Cause Panic

A fourth PCAP presents adversarially crafted ENIP headers: frames with `length=0xFFFF`
(maximum declared length, 65535 bytes), but the TCP payload contains only 24 bytes. This
produces an oversized-frame detect (24 + 65535 > MAX_ENIP_CARRY_BYTES=600), which also
increments `malformed_in_window`. The evaluator confirms: no panic; the tool exits 0;
if 3+ such frames are presented, one T0814 finding appears.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.018 | Postcondition 3 — T0814 finding emitted when malformed_in_window reaches threshold (3) | Case A: 3 malformed frames → 1 T0814 |
| BC-2.17.018 | Precondition 4 — malformed_anomaly_emitted one-shot guard | Case C: 4 frames → still 1 T0814 |
| BC-2.17.018 | Precondition 2 — threshold = MALFORMED_ANOMALY_THRESHOLD = 3 | Case B: 2 frames < threshold → no T0814 |
| BC-2.17.016 | Frame-walk loop: unknown-command rejection increments malformed_in_window | Cases A/B/C: rejected headers counted |
| BC-2.17.016 | Oversized declared-frame skip also increments malformed_in_window | Case D: oversize path counted |

<!-- HIDDEN TRACEABILITY: BC-2.17.018 Precondition 1 (structural-reject paths: invalid command, oversize skip, carry overflow); BC-2.17.018 Note: threshold semantics = >= 3 (fires on 3rd), not strict > like write-burst -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_malformed_3.pcap` — TCP flow dst port 44818; 3 ENIP frames each with 24-byte
   header, command=0xFF00 (unknown), remaining header bytes arbitrary. No valid CPF follows.
2. `enip_malformed_2.pcap` — Same but 2 frames.
3. `enip_malformed_4.pcap` — Same but 4 frames.
4. `enip_oversize_3.pcap` — 3 ENIP frames each with 24-byte header where length=0xFFFF
   (65535) but only 24 bytes of TCP payload (oversized declared-frame path).

## Verification Approach

```bash
wirerust analyze enip_malformed_3.pcap --enip --json
# Expect: exit 0; exactly 1 T0814 finding; no panic.

wirerust analyze enip_malformed_2.pcap --enip --json
# Expect: exit 0; ZERO T0814 findings.

wirerust analyze enip_malformed_4.pcap --enip --json
# Expect: exit 0; exactly 1 T0814 finding (one-shot guard; not 2).

wirerust analyze enip_oversize_3.pcap --enip --json
# Expect: exit 0; at least 1 T0814 finding (oversize path also counts as malformed);
#         no panic on length=0xFFFF.
```

## Evaluation Rubric

- **T0814 fires at threshold 3** (weight: 0.35): Case A: exactly one T0814 for 3 malformed.
- **Threshold not met at 2** (weight: 0.25): Case B: zero T0814 for 2 malformed.
- **One-shot guard** (weight: 0.20): Case C: 4 malformed → one T0814, not two.
- **No panic safety** (weight: 0.20): Case D: oversize frames (length=0xFFFF) processed
  without panic; T0814 emitted after 3 such events.

## Failure Guidance

"HOLDOUT FAIL: HS-117 — T0814 structural anomaly missing, wrong threshold, or panic.
If Case A (3 frames) produces no T0814, either the malformed_in_window counter is not
reaching 3, or the is_valid_enip_frame check is not rejecting unknown command codes.
If Case B (2 frames) produces T0814, the threshold check fires at 2 instead of 3
(check: malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD, where threshold = 3). If
Case D panics on length=0xFFFF, the oversize-frame skip path is not bounds-checking
the cursor advance. See BC-2.17.018 and BC-2.17.016."
