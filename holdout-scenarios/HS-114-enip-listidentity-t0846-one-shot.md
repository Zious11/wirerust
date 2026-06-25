---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.010.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.004.md
  - .factory/stories/STORY-134.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-114"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.010
  - BC-2.17.004
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcap: TCP flow on port 44818 with multiple ENIP ListIdentity (command=0x0063) frames. No CIP payload required — detection is at ENIP command layer."
---

# Holdout Scenario: ListIdentity Command Emits T0846 Once Per Flow (One-Shot Guard)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

EtherNet/IP ListIdentity (command code 0x0063) is the network-wide device enumeration
broadcast. The analyzer emits T0846 ("Remote System Discovery") per-flow with a one-shot
guard: the FIRST ListIdentity frame on a flow produces one T0846 finding; subsequent
ListIdentity frames on the same flow do NOT produce additional findings. This prevents a
single scan campaign from flooding the findings list with near-identical T0846 entries.

### Case A — Single ListIdentity Frame Emits Exactly One T0846 Finding

1. A crafted PCAP is presented: one TCP flow on port 44818; one ENIP frame with
   command=0x0063 (ListIdentity), length=0, session_handle=0 (ListIdentity does not
   require a registered session), status=0, sender_context all-zero, options=0.
2. The user runs: `wirerust analyze enip_list_identity_one.pcap --enip --json`
3. The tool exits 0.
4. The JSON findings array contains EXACTLY ONE finding with `"mitre_techniques": ["T0846"]`.
5. The finding evidence or summary references "ListIdentity" or "discovery" or "enumeration"
   — evaluator does not require exact wording.

### Case B — Multiple ListIdentity Frames on Same Flow: Still Only One T0846 Finding

1. A second crafted PCAP is presented: one TCP flow on port 44818; FIVE ENIP frames all
   with command=0x0063 (ListIdentity).
2. The user runs: `wirerust analyze enip_list_identity_five.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: EXACTLY ONE T0846 finding (the one-shot guard fires after the
   first ListIdentity; the remaining four frames do not produce additional T0846 findings).
   This is the critical one-shot guard verification.

### Case C — ListIdentity Detection Is At ENIP Command Layer (No CIP Required)

ListIdentity detection occurs at the ENIP encapsulation-command classification layer
(before CPF item walk or CIP parse). The fixture in Case A uses command=0x0063 with
length=0 (no payload). The evaluator confirms that no parse error is emitted for an
empty-payload ListIdentity (zero-length payload is normal for this command type).

### Case D — ListIdentity Distinct from T0888 (Identity Object Read)

A fourth fixture is presented with a GetAttributeSingle (CIP service=0x0E, path class=0x01
Identity Object) in a 0x00B2 item. This produces a T0888 finding (not T0846). The evaluator
confirms: the ListIdentity scenario produces T0846; the CIP Identity-Object read produces
T0888; these are distinct technique IDs from distinct detection layers (ENIP command layer
vs. CIP service layer).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.010 | Postcondition 2 — first ListIdentity emits T0846, sets one-shot guard | Case A: one frame → one T0846 |
| BC-2.17.010 | Postcondition 3 — subsequent ListIdentity frames do not emit additional findings | Case B: five frames → one T0846 (guard active) |
| BC-2.17.010 | Invariant 1 — per-flow one-shot guard | Case B: guard prevents duplicate T0846 |
| BC-2.17.010 | Invariant 4 — ListIdentity detected at ENIP command layer, no CIP needed | Case C: zero-length payload accepted |
| BC-2.17.010 | Invariant 5 — T0846 distinct from T0888 | Case D: different techniques for different detections |
| BC-2.17.004 | classify_enip_command returns ListIdentity for command=0x0063 | Cases A/B: command classifier identifies 0x0063 |

<!-- HIDDEN TRACEABILITY: BC-2.17.010 EC-002 (multiple frames same flow → one-shot guard); EC-004 (session_handle=0 normal for ListIdentity) -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_list_identity_one.pcap` — TCP flow dst port 44818; one ENIP header with
   command=0x0063, length=0, session=0, status=0, context=0x8, options=0. Total: 24 bytes.
2. `enip_list_identity_five.pcap` — Same flow with five such frames consecutively.
3. `enip_identity_read_0x0e.pcap` — TCP flow dst port 44818 with ENIP SendRRData +
   CPF 0x00B2 + CIP service=0x0E (GetAttributeSingle) targeting class=0x01 (Identity Object).

Note: A ListIdentity frame with length=0 means no CPF payload after the 24-byte header,
which is protocol-correct for a ListIdentity request (the response from devices would be
sent separately). This zero-payload form is the normal wire representation.

## Verification Approach

```bash
wirerust analyze enip_list_identity_one.pcap --enip --json
# Expect: exit 0; exactly 1 finding with "T0846".

wirerust analyze enip_list_identity_five.pcap --enip --json
# Expect: exit 0; exactly 1 finding with "T0846" (NOT 5 findings — one-shot guard active).

wirerust analyze enip_identity_read_0x0e.pcap --enip --json
# Expect: exit 0; 1 finding with "T0888" (NOT T0846 — different detection layer).
```

## Evaluation Rubric

- **T0846 single-emission** (weight: 0.35): Case A produces exactly one T0846.
- **One-shot guard** (weight: 0.35): Case B produces exactly one T0846 despite five frames.
  If five T0846 findings appear, the one-shot guard is missing — HIGH severity.
- **Zero-payload ListIdentity accepted** (weight: 0.15): Case C produces T0846 with no
  parse error for empty payload.
- **T0846/T0888 distinction** (weight: 0.15): Case D confirms the two detection layers
  produce different techniques.

## Failure Guidance

"HOLDOUT FAIL: HS-114 — ListIdentity one-shot guard missing or T0846/T0888 conflated.
If Case B produces 5 T0846 findings instead of 1, the per-flow list_identity_emitted guard
is not implemented. Each flow must track the guard and set it after the first ListIdentity
T0846 finding. See BC-2.17.010 Postcondition 3 and Invariant 1. If Case D produces T0846
instead of T0888 for a CIP Identity Object read, the detection layers are conflated — T0846
is for ENIP-layer command=0x0063 (ListIdentity); T0888 is for CIP-layer GetAttribute to
class 0x01."
