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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.017.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.019.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.020.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.021.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.037.md
input-hash: "08c9d58"
traces_to: .factory/stories/STORY-051.md
id: "HS-057"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.017
  - BC-2.07.019
  - BC-2.07.020
  - BC-2.07.021
  - BC-2.07.037
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Non-ASCII UTF-8 and Invalid UTF-8 SNI Bytes Produce T1027 Findings With Raw Byte Preservation

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains three TLS ClientHellos with adversarial SNI values:
   - (A) Cyrillic characters: `мир.рф` encoded as valid UTF-8 multi-byte sequences
   - (B) Invalid UTF-8 bytes: `\xff\xfe` (cannot decode as UTF-8)
   - (C) Mixed: a sequence that is valid UTF-8 but contains BOTH a C0 byte (0x01) AND a non-ASCII code point (é = 0xC3 0xA9)
2. The analyst runs wirerust on this pcap.
3. All three ClientHellos produce an anomaly finding with MITRE technique T1027.
4. Finding (A) and (C): summary mentions "non-ASCII characters" — NOT "control bytes". Finding (B): summary mentions "non-UTF-8 bytes".
5. Finding (B) has a `sni_counts` key in the format `<non-utf8:XXXX>` where XXXX is the hex encoding of the raw bytes.
6. Finding (C) routes to arm 3 (non-ASCII UTF-8) rather than arm 2 (ASCII with control), because the non-ASCII code point causes is_ascii() to return false before the control-byte check is evaluated.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.017 | postcondition 1-3; invariant 1 | Non-ASCII UTF-8 SNI (A) emits T1027 finding with raw UTF-8 hostname in summary |
| BC-2.07.019 | postcondition 1-3; invariant 1 | Non-UTF-8 bytes (B) emit T1027 finding; sni_counts key is hex-tagged |
| BC-2.07.020 | postcondition 1-4; invariant 1-2 | Raw bytes preserved in evidence; lossy string in summary for arm 4 |
| BC-2.07.021 | postcondition 1-3 | Raw decoded UTF-8 in summary for arm 3; no Debug escaping |
| BC-2.07.037 | postcondition 1-4; invariant 1-2 | Mixed C0+non-ASCII routes to arm 3, not arm 2; summary says "non-ASCII" not "control" |

## Verification Approach

Craft a pcap with three ClientHellos. Run wirerust with JSON output.

1. Assert exactly 3 T1027 findings in `findings` array.
2. For finding (A): assert `summary` contains the raw Cyrillic characters (e.g., "мир") as readable UTF-8 — not `\u{...}` escape sequences.
3. For finding (B): assert `evidence[0]` starts with `"hex: "` followed by `"fffe"` or the actual hex of the invalid bytes. Assert the sni_counts key (visible via the top_snis field) is in `<non-utf8:XXXX>` format.
4. For finding (C): assert `summary` contains "non-ASCII" and NOT "control bytes" — the arm 3 message dominates.
5. Assert all three findings have `mitre_technique == "T1027"`.
6. Assert all three findings have `direction == "ClientToServer"`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): All three hostnames detected; correct T1027 MITRE code; correct direction.
- **Edge case handling** (weight: 0.35): Arm 3 wins over arm 2 for mixed C0+non-ASCII input; Cyrillic appears verbatim not Debug-escaped; hex-tagged key for invalid UTF-8.
- **Error quality** (weight: 0.15): Evidence field contains hex starting with "hex: "; summary does not use {:?} formatting.
- **Data integrity** (weight: 0.1): sni_counts entries present for all three hostnames.

## Edge Conditions

- The arm ordering is critical: arm 2 (ASCII+C0) must not fire when the string also has non-ASCII code points. The `is_ascii()` check is evaluated before `contains_c0_or_del`.
- Non-UTF-8 bytes: the sni_counts key must be the hex-tagged format, not the lossy string form, because two different invalid byte sequences can produce the same lossy U+FFFD form.
- Finding summary for non-UTF-8 (arm 4): must use `from_utf8_lossy` with U+FFFD replacement chars, not the raw invalid bytes.

## Failure Guidance

"HOLDOUT LOW: HS-057 (satisfaction: 0.XX) -- SNI arm 3/4 detection failed; arm ordering may be wrong (arm 2 firing for mixed C0+non-ASCII), raw bytes escaped instead of preserved, or hex-tagged sni_counts key not used for invalid UTF-8."
