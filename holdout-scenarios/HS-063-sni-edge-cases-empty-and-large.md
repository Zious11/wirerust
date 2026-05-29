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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.022.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.023.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.024.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.025.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.026.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.027.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.028.md
input-hash: "6e52bc5"
traces_to: .factory/stories/STORY-051.md
id: "HS-063"
category: "edge-case-combinations"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.022
  - BC-2.07.023
  - BC-2.07.024
  - BC-2.07.025
  - BC-2.07.026
  - BC-2.07.027
  - BC-2.07.028
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: SNI Edge Cases — Empty List, Multi-Name, Large SNI, and Count-Cap Decoupling

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains five TLS ClientHellos with unusual SNI configurations:
   - (A) SNI extension present but ServerNameList is empty (no hostname entries).
   - (B) SNI extension with two entries: first is `example.com`, second is `evil\x01.com`. Only the first should be processed.
   - (C) SNI extension with one entry where NameType byte is 1 (non-standard) but hostname bytes are `legit.test` (clean ASCII).
   - (D) SNI extension with a hostname of exactly 16,384 bytes of repeating 'a' characters (under the MAX_RECORD_PAYLOAD limit).
   - (E) A ClientHello where the `sni_counts` map has been pre-filled to capacity (50,000 entries) and a new anomalous non-UTF-8 SNI arrives.
2. The analyst runs wirerust on this pcap.
3. (A) produces no finding and no sni_counts entry.
4. (B) produces no finding (first entry is clean ASCII) and sni_counts has `example.com` counted once; the second entry's C0 byte is never inspected.
5. (C) produces no finding; NameType is discarded; the clean ASCII hostname is counted in sni_counts.
6. (D) parses successfully; no truncated_records increment; sni_counts has a 16 KB key; handshakes_seen incremented.
7. (E) produces an anomaly finding (T1027) even though the sni_counts map is at capacity and cannot insert the new key.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.022 | postcondition 1-4; invariant 1-2 | Empty ServerNameList: no count, no finding; same as no-SNI |
| BC-2.07.023 | postcondition 1-4; invariant 1-2 | Empty hostname bytes counted under "" key; no finding |
| BC-2.07.024 | postcondition 1-4; invariant 1-2 | Only first ServerName entry processed; second ignored |
| BC-2.07.025 | postcondition 1-3; invariant 1-3 | Non-zero NameType discarded; hostname processed normally |
| BC-2.07.026 | postcondition 1-3 | Trailing bytes in SNI extension tolerated silently |
| BC-2.07.027 | postcondition 1-5; invariant 1-2 | 16 KB SNI accepted without truncated_records increment |
| BC-2.07.028 | postcondition 1-4; invariant 1-2 | Count cap does not suppress finding emission; decoupled |

## Verification Approach

Craft a pcap with the five described ClientHellos. Run wirerust with JSON output.

1. Assert (A): sni_counts in summary does NOT contain an entry from the empty-list ClientHello.
2. Assert (B): top_snis contains `example.com`; no T1027 finding for `evil\x01.com` (it was never inspected).
3. Assert (C): top_snis contains `legit.test`; no finding; no NameType-related error.
4. Assert (D): `truncated_records == 0`; `handshakes_seen >= 4` (A, B, C, D all increment it); no parse error for 16 KB SNI.
5. Assert (E): `findings` contains a T1027 finding from the anomalous SNI; `sni_counts.len()` remains at 50,000 (the finding fires despite count drop).
6. Assert no panics at any point.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Empty list produces no count/finding; multi-name only processes first; NameType discarded; 16 KB accepted; count-cap does not suppress findings.
- **Edge case handling** (weight: 0.35): Specifically that the second entry in (B) is never inspected and its C0 byte causes no finding; that (E) finding fires even when sni_counts is full.
- **Error quality** (weight: 0.15): No parse_errors for valid but unusual SNI structures; truncated_records stays at 0 for 16 KB SNI.
- **Data integrity** (weight: 0.1): handshakes_seen increments for all valid ClientHellos including those with empty SNI lists.

## Edge Conditions

- ClientHello with empty SNI ServerNameList — this is structurally valid TLS; no error should occur.
- NameType=1 (non-zero) is an unusual encoding but still valid for the current implementation's intent to extract hostname bytes.
- Count cap decoupling: the finding emission path and the sni_counts insertion path are sequential, not conditional on each other.

## Failure Guidance

"HOLDOUT LOW: HS-063 (satisfaction: 0.XX) -- SNI edge-case handling failed; verify that multi-name list only processes first entry, count cap does not suppress findings, empty list produces no count/finding, and 16 KB SNI does not trigger truncated_records."
