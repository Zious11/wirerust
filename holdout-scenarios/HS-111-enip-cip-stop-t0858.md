---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.011.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.005.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.006.md
  - .factory/stories/STORY-135.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-111"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.011
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
fixture_note: "Requires crafted pcap: TCP flow on port 44818 delivering an ENIP SendRRData frame with CPF 0x00B2 Unconnected Data Item carrying CIP service byte 0x07 (Stop). See byte layout in scenario body."
---

# Holdout Scenario: CIP Stop Service on 0x00B2 Emits T0858 Change Operating Mode

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

A crafted PCAP file is presented containing a single TCP flow on port 44818. The flow carries
one EtherNet/IP SendRRData frame embedding a CIP Stop service request in a CPF Unconnected
Data Item (type_id 0x00B2).

**Byte layout (ENIP + CPF + CIP, approximate — exact fixture construction obligation for F4):**

```
ENIP 24-byte header:
  6F 00             -- command = 0x006F (SendRRData), LE
  18 00             -- length = 24 bytes payload (CPF wrapper + CIP stub), LE
  01 00 00 00       -- session_handle = 1, LE
  00 00 00 00       -- status = 0 (success)
  00 00 00 00 00 00 00 00  -- sender_context (8 bytes, all zero)
  00 00 00 00       -- options = 0

CPF (24 bytes):
  02 00             -- item count = 2
  00 00 00 00       -- item[0]: type_id=0x0000 (Null), length=0
  B2 00             -- item[1]: type_id=0x00B2 (Unconnected Data Item), LE
  0C 00             -- item[1] length = 12 bytes, LE
  
CIP payload (12 bytes within item[1] data):
  07               -- CIP service = 0x07 (Stop), request bit clear (bit 7 = 0)
  02               -- path size = 2 words (4 bytes path)
  20 01            -- logical segment: class=0x01 (Identity Object)
  24 01            -- logical segment: instance=0x01
  00 00 00 00 00 00 00  -- remaining pad bytes (CIP request data, if any)
```

Note: The exact byte-for-byte fixture is an F4 obligation. The evaluator uses the observable
CLI/JSON behavior (finding emitted or not, technique ID, finding category) as the acceptance gate.

### Case A — CIP Stop on 0x00B2 Emits T0858 Finding

1. The user runs: `wirerust analyze enip_cip_stop_0x00b2.pcap --enip --json`
2. The tool exits 0.
3. The evaluator inspects the JSON findings array. EXACTLY one finding with
   `"mitre_techniques": ["T0858"]` must be present.
4. The finding must have:
   - technique tag `T0858` (not T0814, not T0857, not T0858 + anything else)
   - category consistent with Execution/ICS-Execution context
   - summary containing language consistent with "CIP Stop" or "controller" or
     "mode change" or "run→stop" or similar — evaluator does NOT require the exact
     string but must confirm the finding is clearly about a CIP Stop event
5. No panic, no crash.

### Case B — CIP Stop on 0x00B1 Does NOT Fire in v0.11.0 (Scope Gate)

This is covered by HS-119 (0x00B1 deferral negative scenario). This scenario focuses on the
positive case.

### Case C — CIP Stop Response Byte (0x87) Does Not Emit Finding

A second fixture is presented where the CIP service byte is 0x87 (0x07 with response bit set,
bit 7 = 1). The response frame must NOT produce a T0858 finding. The evaluator confirms no
T0858 finding appears for a response-bit-set service byte.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.011 | Postcondition 1 — exactly one T0858 Finding emitted per CIP Stop request on 0x00B2 | Case A: one T0858 finding present in JSON |
| BC-2.17.011 | Precondition 2 — service & 0x80 == 0 (request only) | Case C: 0x87 (response bit set) does not emit T0858 |
| BC-2.17.011 | Invariant 1 — T0858 is the correct v19.1 technique (not T0814, not T0857) | Case A: technique field = "T0858" exactly |
| BC-2.17.005 | CPF item-walk identifies 0x00B2 item | Case A: item type_id 0x00B2 detected and CIP payload extracted |
| BC-2.17.006 | parse_cip_header extracts service code 0x07 | Case A: service = 0x07 identified as Stop class |

<!-- HIDDEN TRACEABILITY: BC-2.17.011 Precondition 3 (0x00B2 only) verified by HS-119 negative case -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_cip_stop_0x00b2.pcap` — TCP flow dst port 44818, one ENIP SendRRData frame with
   CPF 0x00B2 item carrying CIP service=0x07 (Stop request, bit 7 clear).
2. `enip_cip_stop_response.pcap` — same structure but CIP service byte = 0x87 (Stop
   response, bit 7 set). Used for Case C negative control.

## Verification Approach

```bash
wirerust analyze enip_cip_stop_0x00b2.pcap --enip --json
# Expect: exit 0; findings array contains exactly one entry with "T0858";
#         no T0814 structural-anomaly finding for this well-formed frame.

wirerust analyze enip_cip_stop_response.pcap --enip --json
# Expect: exit 0; no T0858 finding (response frame ignored for Stop detection).
```

## Evaluation Rubric

- **T0858 emission correctness** (weight: 0.50): Case A produces exactly one finding with
  technique "T0858". Missing finding = FAIL. Wrong technique = FAIL.
- **Request/response gate** (weight: 0.20): Case C (0x87) produces no T0858 finding.
- **No panic safety** (weight: 0.15): Neither case panics or crashes.
- **Technique accuracy** (weight: 0.15): Finding technique is "T0858" exactly; revoked
  IDs T0857 or T0814 must not appear as substitutes for this detection.

## Failure Guidance

"HOLDOUT FAIL: HS-111 — CIP Stop detection missing or incorrect technique. If no T0858
finding is present, the CPF item-walk may not be reaching the 0x00B2 CIP payload, or
classify_cip_service is not returning CipServiceClass::Stop for service=0x07. If the
technique is T0814 or T0857, the MITRE mapping is wrong (T0858 is required per
ics-attack-19.1). If Case C fires T0858 on the response byte 0x87, the request/response
filter (service & 0x80 == 0) is missing."
