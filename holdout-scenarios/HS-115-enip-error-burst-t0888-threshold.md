---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.014.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.008.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.026.md
  - .factory/stories/STORY-134.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-115"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.014
  - BC-2.17.008
  - BC-2.17.026
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcaps: (A) 6 CIP error response frames within 10 seconds (general_status != 0), (B) exactly 5 — no finding. CIP error responses are detected at the 0x00B2 response layer."
---

# Holdout Scenario: CIP Error Burst T0888 — Strict Greater-Than Threshold (6th Fires, 5th Does Not)

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The CIP error-burst detector (T0888 Pattern B) fires when the count of CIP error responses
(service byte with response-bit set AND general_status != 0, per BC-2.17.008) within a
10-second window STRICTLY EXCEEDS the default threshold of 5. This means:
- 5 error responses in 10 seconds: NO T0888 Pattern B finding.
- 6 error responses in 10 seconds: EXACTLY ONE T0888 Pattern B finding (on the 6th).
- One-shot guard (`error_rate_emitted`) prevents additional findings per window.

This is the "error burst as service probe detection" — an attacker probing which CIP services
are supported generates error responses for unsupported services.

A CIP error response is identified by: CIP service byte with bit 7 set (response flag) AND
general_status field (byte 2 of the CIP response data on 0x00B2) != 0x00.

### Case A — 6 CIP Error Responses in 10 Seconds Emits One T0888 Pattern B Finding

1. A crafted PCAP is presented: one TCP flow on port 44818; 6 ENIP SendRRData frames, each
   carrying a CPF 0x00B2 item containing a CIP response with service byte = 0x8E (service
   0x0E | response-bit 0x80, GetAttributeSingle response), and general_status = 0x08
   (path destination unknown — a common "unsupported service" error).
2. All 6 frames have timestamps within a 10-second window.
3. The user runs: `wirerust analyze enip_error_burst_6.pcap --enip --json`
4. The tool exits 0.
5. The evaluator confirms: EXACTLY ONE finding with `"mitre_techniques": ["T0888"]` whose
   summary or description references "error burst" or "error rate" or "probe" or "T0888".
   The finding must NOT be a Pattern A finding (Identity Object read — those require a
   request not a response).

### Case B — Exactly 5 CIP Error Responses in 10 Seconds: No T0888 Pattern B Finding

1. A second crafted PCAP: same structure but exactly 5 error response frames in 10 seconds.
2. The user runs: `wirerust analyze enip_error_burst_5.pcap --enip --json`
3. The tool exits 0.
4. The evaluator confirms: ZERO T0888 Pattern B findings. The 5th error does not fire
   (5 is NOT > 5). This is the boundary negative control (strict greater-than).

### Case C — --enip-error-burst-threshold Override

1. A third PCAP: 3 CIP error response frames in 10 seconds.
2. The user runs: `wirerust analyze enip_error_burst_3.pcap --enip --enip-error-burst-threshold 2 --json`
3. With threshold=2: 3 > 2 → ONE T0888 Pattern B finding.
4. The user also runs: `wirerust analyze enip_error_burst_3.pcap --enip --json`
   (default threshold=5: 3 is NOT > 5 → zero findings).
5. This confirms the CLI flag --enip-error-burst-threshold correctly sets the sensitivity.

### Case D — Success Response (general_status=0) Does Not Increment Error Counter

A fourth fixture: 6 CIP response frames with general_status = 0x00 (success). These are
successful responses, not errors. The evaluator confirms ZERO T0888 Pattern B findings
(success responses do not count toward the error burst threshold).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.014 | Pattern B Precondition 1 — error count > threshold fires | Case A: 6 > 5 → T0888 |
| BC-2.17.014 | Pattern B Precondition 1 — strict greater-than; 5 does not fire | Case B: 5 is NOT > 5 → no finding |
| BC-2.17.014 | Pattern B Precondition 2 — error_rate_emitted one-shot guard | Cases A: guard prevents duplicate |
| BC-2.17.008 | CIP error response detection — service byte bit 7 set AND general_status != 0 | Cases A/D: response-bit + non-zero status = error |
| BC-2.17.008 | general_status = 0 does not count as error | Case D: success responses do not increment counter |
| BC-2.17.026 | --enip-error-burst-threshold flag configures threshold | Case C: threshold=2 fires at 3 errors |

<!-- HIDDEN TRACEABILITY: BC-2.17.014 Pattern B Postcondition 2 (one T0888 per window); BC-2.17.008 extraction of general_status from 0x00B2 response -->

## Fixture Creation Obligation

**F4 must create:**
1. `enip_error_burst_6.pcap` — TCP flow dst port 44818; 6 ENIP SendRRData frames each with
   CPF 0x00B2 item, CIP service=0x8E (GetAttributeSingle response, bit 7 set), general_status
   byte = 0x08 (error, non-zero); timestamps t=0.0s to t=9.0s (within 10-second window).
2. `enip_error_burst_5.pcap` — Same but 5 frames.
3. `enip_error_burst_3.pcap` — Same but 3 frames.
4. `enip_error_burst_success.pcap` — 6 frames with general_status=0x00 (success, not errors).

## Verification Approach

```bash
wirerust analyze enip_error_burst_6.pcap --enip --json
# Expect: exit 0; exactly 1 T0888 finding (Pattern B).

wirerust analyze enip_error_burst_5.pcap --enip --json
# Expect: exit 0; ZERO T0888 Pattern B findings. (Critical boundary check.)

wirerust analyze enip_error_burst_3.pcap --enip --enip-error-burst-threshold 2 --json
# Expect: exit 0; exactly 1 T0888 finding.

wirerust analyze enip_error_burst_3.pcap --enip --json
# Expect: exit 0; ZERO T0888 findings (3 does not exceed default threshold 5).

wirerust analyze enip_error_burst_success.pcap --enip --json
# Expect: exit 0; ZERO T0888 Pattern B findings (success responses not counted).
```

## Evaluation Rubric

- **Strict-greater-than (6 fires)** (weight: 0.30): Case A: one T0888 Pattern B.
- **Strict-greater-than (5 does not fire)** (weight: 0.30): Case B: zero T0888 Pattern B.
  Off-by-one here means the threshold is wrong.
- **Success responses excluded** (weight: 0.20): Case D: zero T0888 Pattern B when
  general_status=0 (success responses are not errors).
- **CLI threshold override** (weight: 0.20): Case C: threshold=2 causes 3 errors to fire.

## Failure Guidance

"HOLDOUT FAIL: HS-115 — error-burst threshold off-by-one or success responses counted.
If Case B (5 errors) emits T0888, the comparison uses >= instead of > (off-by-one). Fix:
`error_count > threshold`, not `error_count >= threshold`. If Case D (success responses)
emits T0888 Pattern B, the error filter (general_status != 0) is not applied in BC-2.17.008.
See BC-2.17.014 Pattern B and BC-2.17.008. If --enip-error-burst-threshold has no effect
(Case C), verify BC-2.17.026 wiring to EnipAnalyzer.enip_error_burst_threshold."
