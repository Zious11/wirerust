---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.015.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.006.md
  - .factory/stories/STORY-136.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-116"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.015
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
fixture_note: "Requires crafted pcaps: ENIP SendRRData + CPF 0x00B2 + CIP service=0x54 (ForwardOpen) and service=0x4E (ForwardClose). Both CIP Connection Manager services are always carried in 0x00B2 items — not a scope restriction."
---

# Holdout Scenario: ForwardOpen/ForwardClose on 0x00B2 Emits Connection-Lifecycle Finding with Empty MITRE Technique Set

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

CIP ForwardOpen (service=0x54), LargeForwardOpen (service=0x5B), and ForwardClose (service=0x4E)
are Connection Manager services that establish and tear down CIP connections. In v0.11.0, these
are detected and a Finding is emitted — but with an EMPTY MITRE technique array (`mitre_techniques:
[]`). No MITRE technique is assigned in this release because the appropriate mapping is ambiguous
(T1692.001 is a possible future assignment but not confirmed). The finding is still emitted so
analysts can observe connection-lifecycle anomalies in the findings list.

These services MUST be carried in CPF 0x00B2 (Unconnected Data Item) items by CIP protocol
design — this is not a v0.11.0 scope restriction. A ForwardOpen in a 0x00B1 item would be
a protocol violation in real traffic.

### Case A — ForwardOpen (0x54) on 0x00B2 Emits Finding with Empty mitre_techniques

1. A crafted PCAP is presented: TCP flow on port 44818; ENIP SendRRData frame; CPF 0x00B2
   item; CIP service byte = 0x54 (ForwardOpen, request bit clear).
2. The user runs: `wirerust analyze enip_forward_open.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms:
   - At least one finding is present for this flow (a connection-lifecycle anomaly finding).
   - The finding's `mitre_techniques` array is EMPTY (`[]`, not `["T1692.001"]` or any
     other technique). In v0.11.0, no MITRE technique is assigned to ForwardOpen/ForwardClose.
   - No T0858, T0816, T0836, T0846, T0888, or T0814 technique appears for this finding.

### Case B — ForwardClose (0x4E) on 0x00B2 Emits Finding with Empty mitre_techniques

1. A second crafted PCAP: same structure but CIP service byte = 0x4E (ForwardClose, request).
2. The user runs: `wirerust analyze enip_forward_close.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: at least one finding present; `mitre_techniques` is empty.

### Case C — LargeForwardOpen (0x5B) on 0x00B2 Also Detected

1. A third PCAP: CIP service byte = 0x5B (LargeForwardOpen, request).
2. The evaluator confirms: finding present; `mitre_techniques` empty.

### Case D — Response Bytes (0xD4, 0xCE, 0xDB) Do Not Emit Findings

ForwardOpen response = 0x54 | 0x80 = 0xD4. ForwardClose response = 0x4E | 0x80 = 0xCE.
LargeForwardOpen response = 0x5B | 0x80 = 0xDB. Responses must NOT emit connection-lifecycle
findings (request filter: service & 0x80 == 0). The evaluator presents fixtures with these
response bytes and confirms zero connection-lifecycle findings.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.015 | Postcondition 1 — finding emitted for ForwardOpen/ForwardClose/LargeForwardOpen | Cases A/B/C: finding present in JSON |
| BC-2.17.015 | Postcondition 1 — mitre_techniques: vec![] (empty array in v0.11.0) | Cases A/B/C: techniques array is empty |
| BC-2.17.015 | Precondition 2 — request bit clear (service & 0x80 == 0) | Case D: response bytes do not emit findings |
| BC-2.17.015 | Precondition 3 — MUST be 0x00B2 (protocol requirement, not scope restriction) | Cases A/B/C: 0x00B2 carrier correctly handled |
| BC-2.17.006 | CIP header parse extracts service=0x54/0x4E/0x5B | Cases A/B/C: service bytes extracted and classified |

<!-- HIDDEN TRACEABILITY: BC-2.17.015 Note on T1692.001 future assignment (v0.12.0 candidate); ADR-010 Decision 5 in-scope, Decision 7 MITRE gap -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_forward_open.pcap` — TCP flow dst port 44818; ENIP SendRRData + CPF 0x00B2;
   CIP service=0x54 (ForwardOpen request). Minimal CIP request data (path_size=2, 4-byte path).
2. `enip_forward_close.pcap` — Same but CIP service=0x4E (ForwardClose request).
3. `enip_large_forward_open.pcap` — Same but CIP service=0x5B (LargeForwardOpen request).
4. `enip_forward_response.pcap` — CIP service=0xD4 (ForwardOpen response, bit 7 set).

## Verification Approach

```bash
wirerust analyze enip_forward_open.pcap --enip --json
# Expect: exit 0; at least 1 finding; that finding's "mitre_techniques" == [] (empty array).

wirerust analyze enip_forward_close.pcap --enip --json
# Expect: exit 0; at least 1 finding; "mitre_techniques" == [].

wirerust analyze enip_large_forward_open.pcap --enip --json
# Expect: exit 0; at least 1 finding; "mitre_techniques" == [].

wirerust analyze enip_forward_response.pcap --enip --json
# Expect: exit 0; ZERO connection-lifecycle findings for this frame.
```

## Evaluation Rubric

- **Finding emitted** (weight: 0.35): Cases A/B/C produce at least one finding each.
  If no finding appears, the ForwardOpen/ForwardClose classification is missing or the
  detection path is not wired.
- **Empty MITRE array** (weight: 0.35): The finding's `mitre_techniques` field is `[]`.
  If T1692.001 or any other technique appears, the v0.11.0 scope boundary is violated
  (T1692.001 is a future assignment, not assigned in this release).
- **Request filter** (weight: 0.20): Case D produces no finding for response bytes.
- **No false techniques** (weight: 0.10): T0858/T0816/T0836/T0846/T0888/T0814 must not
  appear for ForwardOpen/ForwardClose findings.

## Failure Guidance

"HOLDOUT FAIL: HS-116 — ForwardOpen/ForwardClose detection missing or wrong MITRE tag.
If no finding is emitted, classify_cip_service is not returning ForwardOpen/ForwardClose
for service=0x54/0x4E/0x5B. If mitre_techniques contains T1692.001 or another technique,
the v0.11.0 empty-technique decision (ADR-010 Decision 7 MITRE gap) was not honored. The
mitre_techniques array MUST be empty in v0.11.0 — assign T1692.001 in v0.12.0. See
BC-2.17.015."
