---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs: [stories/, behavioral-contracts/, prd.md]
input-hash: "[md5-pending]"
traces_to: ".factory/specs/prd.md"
id: "HS-069"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.019
  - BC-2.07.020
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Two Invalid UTF-8 SNI Byte Sequences With Same Lossy Form Produce Distinct sni_counts Keys

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains two TLS ClientHellos:
   - (A) SNI bytes: `\xc0\x80` — an overlong encoding of NUL (invalid UTF-8). from_utf8_lossy produces `U+FFFD U+FFFD`.
   - (B) SNI bytes: `\xed\xa0\x80` — a surrogate half (U+D800, invalid UTF-8). from_utf8_lossy also produces `U+FFFD U+FFFD`.
2. Both produce the same lossy string form (two U+FFFD replacement characters), but their raw byte sequences are distinct.
3. The analyst runs wirerust on this pcap.
4. The `top_snis` in the TLS summary (or equivalently the internal sni_counts map) must contain TWO distinct entries — one for `<non-utf8:c080>` and one for `<non-utf8:eda080>` — not a single merged entry.
5. Both ClientHellos produce a T1027 finding each (total: 2 findings from this pcap segment).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.019 | postcondition 1-3; invariant 1 | sni_counts key for non-UTF-8 is `<non-utf8:hex>` format; distinct byte sequences produce distinct keys |
| BC-2.07.020 | postcondition 1-4; invariant 1-2 | lossy string in finding summary; hex in evidence; no escaping at analyzer layer |

## Verification Approach

Craft a pcap with the two described ClientHellos. Run wirerust with JSON output.

1. Assert `findings` contains exactly 2 T1027 findings.
2. Inspect the `top_snis` field in `analyzers[TLS].detail.top_snis`. Assert it contains 2 distinct entries whose keys start with `<non-utf8:`.
3. Assert the two keys are `<non-utf8:c080>` and `<non-utf8:eda080>` (or the exact hex of the respective byte sequences).
4. Assert each finding's `evidence[0]` starts with `"hex: "` followed by the respective hex.
5. Assert each finding's `summary` contains U+FFFD characters (the lossy form), visible as the replacement diamond symbol in UTF-8 aware terminals.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Two distinct sni_counts keys for the two invalid byte sequences; each produces its own finding.
- **Edge case handling** (weight: 0.3): The hex-tagged key format disambiguates byte sequences that are identical in lossy string form.
- **Error quality** (weight: 0.1): finding.summary contains U+FFFD replacement characters from from_utf8_lossy; finding.evidence contains lossless hex.
- **Data integrity** (weight: 0.1): sni_counts entries are per distinct byte sequence, not per lossy string.

## Edge Conditions

- The key disambiguation test is specifically designed to catch implementations that naively use the lossy string as the sni_counts key — those would merge the two sequences into one entry.
- The hex in the key must be lowercase.
- Both sequences produce identical summaries (same U+FFFD content) but distinct evidence (different hex).

## Failure Guidance

"HOLDOUT LOW: HS-069 (satisfaction: 0.XX) -- Two invalid UTF-8 SNI byte sequences that produce the same from_utf8_lossy result were merged into one sni_counts entry; the key must use hex-tagged format `<non-utf8:hex>` to preserve per-byte-sequence uniqueness."
