---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.012.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.023.md
  - .factory/stories/STORY-135.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-113"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.012
  - BC-2.17.023
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcaps: (A) 51 CIP SetAttributeSingle requests in 1 TCP flow within one second, (B) exactly 50 requests — no finding expected. Timestamps must be within 1-second window. See scenario body."
---

# Holdout Scenario: CIP Write-Burst T0836 — Strict Greater-Than Threshold (51st Fires, 50th Does Not)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The write-burst detector for EtherNet/IP fires when the count of CIP write-class service
requests (SetAttributesAll/SetAttributeList/SetAttributeSingle) in a 1-second window
STRICTLY EXCEEDS the threshold (default 50). This means:
- 50 writes in 1 second: NO finding (50 is not > 50).
- 51 writes in 1 second: ONE finding is emitted on the 51st write (51 > 50).
- 52+ writes: NO additional finding per window (one-shot guard).

This strict-greater-than semantics is a boundary condition that evaluators must test precisely.

### Case A — 51 Writes in 1 Second Emits Exactly One T0836 Finding (Default Threshold)

1. A crafted PCAP is presented containing one TCP flow on port 44818.
2. The flow carries 51 consecutive ENIP SendRRData frames, each embedding one CIP
   SetAttributeSingle (service=0x10) request in a 0x00B2 Unconnected Data Item.
3. All 51 frames have timestamps within a 1-second window (e.g., t=0.000s to t=0.990s).
4. The user runs: `wirerust analyze enip_write_burst_51.pcap --enip --json`
5. The tool exits 0.
6. The evaluator confirms: EXACTLY ONE finding with `"mitre_techniques": ["T0836"]` in
   the JSON findings array. No additional T0836 findings (one-shot guard prevents re-emission
   within the same window).
7. The finding summary/description must reference "write" or "attribute" or "parameter
   modification" or "T0836" — evaluator does not require exact wording.

### Case B — Exactly 50 Writes in 1 Second: No T0836 Finding (Strict > Gate)

1. A second crafted PCAP is presented: same structure but exactly 50 CIP write frames within
   1 second.
2. The user runs: `wirerust analyze enip_write_burst_50.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: **ZERO** T0836 findings. The 50th write does NOT trigger T0836
   because `50 > 50` is false. This is the boundary negative control.

### Case C — Tunable Threshold via --enip-write-burst-threshold

1. A third PCAP is presented: 6 CIP write frames within 1 second.
2. The user runs: `wirerust analyze enip_write_burst_6.pcap --enip --enip-write-burst-threshold 5 --json`
3. With threshold=5, the 6th write exceeds 5 (6 > 5), so exactly ONE T0836 finding is emitted.
4. The user also runs: `wirerust analyze enip_write_burst_6.pcap --enip --json`
   (default threshold=50 — 6 writes does not exceed 50, so no T0836 finding).
5. This confirms the CLI flag correctly sets the detection threshold.

### Case D — 52 Writes: Still Only One T0836 Finding (One-Shot Guard)

For completeness, if the evaluator has a fixture with 52 writes in 1 second, there must
still be EXACTLY ONE T0836 finding (not two). The one-shot guard (`write_burst_emitted = true`)
prevents re-emission within the same window after the first finding.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.012 | Postcondition 5 — finding emitted when count > threshold AND guard false | Case A: 51 > 50 → one T0836 finding |
| BC-2.17.012 | Postcondition 5 — strict greater-than: 50 does NOT fire | Case B: 50 is NOT > 50 → zero findings |
| BC-2.17.012 | Postcondition 5 — one-shot guard prevents duplicate within window | Case D: 52 writes → still one finding |
| BC-2.17.012 | Precondition 1 — SetAttributeSingle (0x10) is a write-class service | Cases A/B/C: service=0x10 increments write counter |
| BC-2.17.023 | --enip-write-burst-threshold flag configures threshold | Case C: threshold=5 causes 6th write to fire |

<!-- HIDDEN TRACEABILITY: BC-2.17.012 Postcondition 1-4 (counter increment, aggregate counter, window start) -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_write_burst_51.pcap` — TCP flow dst port 44818; 51 ENIP SendRRData frames, each
   with CPF 0x00B2 item, CIP service=0x10 (SetAttributeSingle, request); all frames with
   timestamps t=0.000s to t=0.990s (within 1 second).
2. `enip_write_burst_50.pcap` — Same but exactly 50 frames.
3. `enip_write_burst_6.pcap` — Same but exactly 6 frames within 1 second.

Note: In all fixtures, CIP service=0x10 (SetAttributeSingle) with bit 7 clear confirms
request mode. Minimal path data (e.g., 4 bytes) is sufficient.

## Verification Approach

```bash
wirerust analyze enip_write_burst_51.pcap --enip --json
# Expect: exit 0; exactly 1 finding with "T0836".

wirerust analyze enip_write_burst_50.pcap --enip --json
# Expect: exit 0; ZERO findings with "T0836". (Critical boundary check.)

wirerust analyze enip_write_burst_6.pcap --enip --enip-write-burst-threshold 5 --json
# Expect: exit 0; exactly 1 finding with "T0836".

wirerust analyze enip_write_burst_6.pcap --enip --json
# Expect: exit 0; ZERO findings (6 does not exceed default threshold 50).
```

## Evaluation Rubric

- **Strict-greater-than boundary (51 fires)** (weight: 0.30): Case A: exactly one T0836.
- **Strict-greater-than boundary (50 does not fire)** (weight: 0.30): Case B: zero T0836.
  This is the most important negative control — off-by-one here is a high-severity bug.
- **One-shot guard** (weight: 0.15): Case D: 52 writes → one T0836, not two.
- **Threshold configurability** (weight: 0.25): Case C: --enip-write-burst-threshold 5
  changes threshold; 6 writes fires at threshold 5 but not at default 50.

## Failure Guidance

"HOLDOUT FAIL: HS-113 — write-burst threshold boundary error. If Case B (50 writes) emits
T0836, the threshold check uses >= instead of > (off-by-one: threshold exceeded at count==50,
should require count==51). Fix: use `count > threshold`, not `count >= threshold`. If Case A
(51 writes) does NOT emit T0836, the window start-timestamp or counter-increment logic is
incorrect. See BC-2.17.012 Postcondition 5. If Case C CLI override has no effect, verify
--enip-write-burst-threshold is wired to EnipAnalyzer.enip_write_burst_threshold. See
BC-2.17.023."
