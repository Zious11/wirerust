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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.013.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.014.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.015.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.016.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.018.md
input-hash: "6e52bc5"
traces_to: .factory/stories/STORY-051.md
id: "HS-056"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.013
  - BC-2.07.014
  - BC-2.07.015
  - BC-2.07.016
  - BC-2.07.018
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: SNI Control-Byte Obfuscation Detected With Exact Boundary Semantics

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains four TLS ClientHellos, each with a distinct SNI:
   - (A) `example.com` (clean ASCII)
   - (B) `evil.com` with a 0x1B ESC byte embedded (e.g., bytes `evil\x1bcom.net`)
   - (C) `xn--caf-dma.example` (valid Punycode A-label, pure ASCII)
   - (D) `test host.com` (contains 0x20 space, which is NOT a C0 byte)
2. The analyst runs wirerust on this pcap.
3. Only SNI (B) produces an anomaly finding; the other three produce no findings.
4. The finding for SNI (B) has MITRE technique T1027 and includes a hex-encoded representation of the raw hostname bytes in the evidence field.
5. The finding for SNI (B) is exactly one finding — not one per ESC byte even if the SNI contained multiple control bytes.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.013 | postcondition 1-3; invariant 1 | Clean ASCII SNI (A, C, D) produces no finding |
| BC-2.07.014 | postcondition 1-4; invariant 4 | C0 byte in SNI (B) emits one T1027 finding; raw bytes in evidence |
| BC-2.07.015 | postcondition 1-3; invariant 1 | One finding per hostname regardless of control byte count |
| BC-2.07.016 | postcondition 1-4; invariant 1 | 0x1F triggers; 0x20 does NOT trigger; boundary is exact |
| BC-2.07.018 | postcondition 1-3 | Punycode A-label treated as clean ASCII arm 1 |

## Verification Approach

Craft a pcap with four TLS ClientHello records. Run wirerust with JSON output.

1. Assert `findings` array has exactly 1 SNI-related finding.
2. Assert that finding has `mitre_technique == "T1027"`.
3. Assert the finding's `evidence` array contains a string starting with `"hex: "` followed by lowercase hex.
4. Assert `sni_counts` in `analyzers[TLS].detail.top_snis` includes entries for all four SNIs.
5. Assert the space-containing SNI (D) has no finding (0x20 is NOT a C0 byte).
6. Assert the Punycode SNI (C) has no finding.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Exactly one finding for ESC-containing SNI; zero findings for all others.
- **Edge case handling** (weight: 0.35): Space (0x20) is below 0x21 but above 0x1F — it is the exact boundary and must NOT trigger; Punycode A-label must pass through as clean ASCII.
- **Error quality** (weight: 0.1): Evidence field contains hex representation; summary contains raw hostname (not Debug-escaped).
- **Data integrity** (weight: 0.1): sni_counts incremented for all four SNIs including the anomalous one.

## Edge Conditions

- The space character (0x20) is the most likely false-positive source — the predicate is strictly `< 0x20`.
- Punycode A-labels (e.g., `xn--...`) are pure ASCII and must not be flagged despite looking unusual.
- Multiple C0 bytes in one SNI must produce exactly one finding, not N findings.

## Failure Guidance

"HOLDOUT LOW: HS-056 (satisfaction: 0.XX) -- SNI control-byte detection produced wrong finding count; check the 0x1F/0x20 boundary, Punycode handling, or the one-finding-per-hostname invariant."
