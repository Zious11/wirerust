---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.013.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.005.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.006.md
  - .factory/stories/STORY-135.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-112"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.013
  - BC-2.17.005
  - BC-2.17.006
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcap: TCP flow on port 44818 with ENIP SendRRData frame carrying CPF 0x00B2 item with CIP service byte 0x05 (Reset)."
---

# Holdout Scenario: CIP Reset Service on 0x00B2 Emits T0816 Device Restart/Shutdown

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

A crafted PCAP file is presented containing a single TCP flow on port 44818 carrying one
EtherNet/IP SendRRData frame. The frame's CPF payload contains a Unconnected Data Item
(type_id 0x00B2) whose CIP payload begins with service byte 0x05 (CIP Reset service,
request bit clear).

The CIP Reset service (0x05) is the standard mechanism to trigger a device software reset
(cold restart) on an EtherNet/IP-connected PLC, drive, or relay. It is distinct from CIP
Stop (0x07): Stop halts the user program but leaves the device running; Reset reboots the
device entirely.

### Case A — CIP Reset Request Emits T0816 Finding

1. The user runs: `wirerust analyze enip_cip_reset.pcap --enip --json`
2. The tool exits 0.
3. The JSON findings array contains exactly one finding with `"mitre_techniques": ["T0816"]`.
4. The finding category/summary is consistent with "device restart", "shutdown", "reset",
   or "T0816" — evaluator does not require the exact summary string but must confirm the
   finding clearly references a CIP Reset or device restart event.
5. No T0858 finding is emitted for this frame (Reset ≠ Stop).

### Case B — Second CIP Reset Frame Generates Second T0816 Finding (Per-Occurrence)

A second fixture is presented containing TWO CIP Reset request frames in the same TCP flow.
The evaluator confirms TWO T0816 findings are present in the JSON output (or, if the
MAX_FINDINGS cap has not been reached, the count matches the number of Reset frames seen).
CIP Reset detection has no one-shot guard — each occurrence emits one finding, up to
MAX_FINDINGS. This distinguishes Reset from ListIdentity (which has a one-shot guard).

### Case C — CIP Reset on 0x00B2, Response Byte (0x85) Does Not Fire

A fixture with CIP service byte 0x85 (0x05 | 0x80 = response bit set) must NOT produce a
T0816 finding. The evaluator confirms no T0816 finding for the response frame.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.013 | Postcondition 1 — exactly one T0816 Finding per CIP Reset request on 0x00B2 | Case A: one T0816 finding in JSON output |
| BC-2.17.013 | No one-shot guard — per-occurrence detection | Case B: two Reset frames → two T0816 findings |
| BC-2.17.013 | Precondition 2 — service & 0x80 == 0 (request only) | Case C: response byte 0x85 does not fire T0816 |
| BC-2.17.013 | Invariant — T0816 is Inhibit Response Function; distinct from T0858 | Case A: technique is "T0816", not "T0858" |
| BC-2.17.005 | CPF 0x00B2 item extraction | Case A: item-walk finds 0x00B2 and passes CIP data to CIP header parse |
| BC-2.17.006 | CIP service byte 0x05 extracted and classified | Case A: service=0x05 classified as Reset |

<!-- HIDDEN TRACEABILITY: BC-2.17.013 Precondition 3 (0x00B2 only); BC-2.17.013 Postcondition 2 (no one-shot guard) -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_cip_reset.pcap` — TCP flow dst port 44818, one ENIP SendRRData frame with CPF
   0x00B2 item, CIP service byte = 0x05 (Reset request).
2. `enip_cip_reset_two.pcap` — same flow with two consecutive Reset request frames.
3. `enip_cip_reset_response.pcap` — same structure but CIP service = 0x85 (Reset response).

## Verification Approach

```bash
wirerust analyze enip_cip_reset.pcap --enip --json
# Expect: exit 0; one finding with "T0816"; no T0858 finding.

wirerust analyze enip_cip_reset_two.pcap --enip --json
# Expect: exit 0; two findings with "T0816" (per-occurrence, no one-shot guard).

wirerust analyze enip_cip_reset_response.pcap --enip --json
# Expect: exit 0; no T0816 finding (response bit set → not a request).
```

## Evaluation Rubric

- **T0816 emission** (weight: 0.45): Case A produces exactly one T0816 finding. Missing or
  wrong technique = FAIL.
- **Per-occurrence (no one-shot)** (weight: 0.25): Case B produces two T0816 findings.
  If only one finding appears, the one-shot guard is incorrectly applied to Reset.
- **Request/response gate** (weight: 0.15): Case C produces no T0816 finding.
- **Technique purity** (weight: 0.15): T0816 appears; T0858 does not appear for Reset.

## Failure Guidance

"HOLDOUT FAIL: HS-112 — CIP Reset detection missing or conflated with Stop. If T0858 appears
instead of T0816, classify_cip_service is mapping service=0x05 to the wrong class. If Case B
produces only one T0816 (not two), an incorrect one-shot guard has been applied to Reset —
Reset detection must be per-occurrence with no one-shot guard. See BC-2.17.013."
