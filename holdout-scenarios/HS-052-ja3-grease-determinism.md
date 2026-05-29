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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.006.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.007.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.008.md
input-hash: "6e52bc5"
traces_to: .factory/stories/STORY-051.md
id: "HS-052"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.006
  - BC-2.07.007
  - BC-2.07.008
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: JA3 Fingerprint Matches Known-Good Reference Value

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains a single TLS ClientHello from a Chrome browser (TLS 1.3 ClientHello with GREASE values in the cipher list and extensions).
2. The analyst runs wirerust and obtains the `ja3_hashes` map from the TLS analyzer summary.
3. The JA3 hash emitted must match the value that other established JA3 tools (Wireshark JA3 plugin, ja3.zone lookup) produce for the same ClientHello — GREASE values filtered, remaining ciphers and extensions in original order, MD5 of the assembled 5-field string.
4. A second pcap contains a ClientHello with an identical cipher list EXCEPT for GREASE value `0x3a3a` inserted at position 0. Both pcaps must produce the same JA3 hash.
5. The JA3S hash from a corresponding ServerHello has exactly 3 comma-separated fields (version, cipher, extensions).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.006 | postcondition 1; invariant 1 | GREASE values filtered by bitmask before JA3 assembly |
| BC-2.07.007 | postcondition 1-8; invariant 2-4 | 5-field format; decimal encoding; MD5 hex; order-sensitive |
| BC-2.07.008 | postcondition 1-6; invariant 1-2 | JA3S 3-field format; single cipher value; GREASE filtered |

## Verification Approach

1. Compute the expected JA3 hash for a known Chrome 120 ClientHello using a reference implementation (ja3.zone or Wireshark plugin).
2. Run wirerust on a pcap of that ClientHello.
3. Assert `analyzers[TLS].detail.ja3_hashes` contains a key matching the expected 32-character lowercase hex hash.
4. Run wirerust on the same ClientHello with a GREASE cipher prepended.
5. Assert the same JA3 hash appears — confirming GREASE filtering.
6. Inspect JA3S output and verify exactly 2 commas in the stored string components (when decoded from the MD5 input — verifiable by inspection of the source-level string before hashing).

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): JA3 hash matches reference tool output for the same ClientHello.
- **Edge case handling** (weight: 0.3): GREASE insertion does not change the hash; the hash is order-sensitive so cipher reordering must change it.
- **Error quality** (weight: 0.1): No error output for valid TLS.
- **Data integrity** (weight: 0.1): Hash is exactly 32 lowercase hex characters.

## Edge Conditions

- GREASE value at position 0 of cipher list — must be transparent to the fingerprint.
- All-GREASE cipher list — JA3 cipher field must be empty string.
- JA3S for ServerHello with no extensions — JA3S extension field is empty string.

## Failure Guidance

"HOLDOUT LOW: HS-052 (satisfaction: 0.XX) -- JA3 hash did not match reference value or GREASE filtering did not produce a stable fingerprint; verify bitmask `(val & 0x0F0F) == 0x0A0A` and decimal field encoding."
