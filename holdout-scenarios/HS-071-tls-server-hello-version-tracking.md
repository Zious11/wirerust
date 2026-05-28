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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.002.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.003.md
input-hash: "08c9d58"
traces_to: .factory/stories/STORY-051.md
id: "HS-071"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.002
  - BC-2.07.003
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: ServerHello Version Tracked Independently From ClientHello Version

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains a TLS session where the ClientHello advertises version 0x0301 (TLS 1.0) in its legacy_version field, but the ServerHello negotiates version 0x0303 (TLS 1.2) in its version field. This can happen in version downgrade negotiation scenarios.
2. The analyst runs wirerust on this pcap.
3. The `tls_versions` map in the TLS summary must contain BOTH version 769 (decimal for 0x0301) AND version 771 (decimal for 0x0303) — one from the ClientHello and one from the ServerHello.
4. The JA3 hash was computed from the ClientHello fields (version 769 as the first field).
5. The JA3S hash was computed from the ServerHello fields (version 771 as the first field).
6. `handshakes_seen == 1` (one ClientHello processed).
7. No deprecated-protocol finding is emitted — both 0x0301 and 0x0303 are above the `<= 0x0300` threshold.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.002 | postcondition 2-3; invariant 3 | ServerHello version tracked independently from ClientHello in shared version_counts map |
| BC-2.07.003 | postcondition 1-5; invariant 1-2 | After both hellos seen, subsequent records silently skipped; done() returns true |

## Verification Approach

Craft a pcap with a ClientHello version 0x0301 and ServerHello version 0x0303. Run wirerust with JSON output.

1. Assert `analyzers[TLS].detail.tls_versions` contains BOTH `"769"` and `"771"`.
2. Assert `analyzers[TLS].detail.tls_versions["769"] == 1` (one ClientHello with version 769).
3. Assert `analyzers[TLS].detail.tls_versions["771"] == 1` (one ServerHello with version 771).
4. Assert `analyzers[TLS].detail.ja3_hashes` contains a key starting with the expected 32-char hex derived from version 769.
5. Assert `analyzers[TLS].detail.ja3s_hashes` contains a key derived from version 771.
6. Assert `findings` contains zero deprecated-protocol findings (0x0301 and 0x0303 are both above the threshold).
7. Assert `analyzers[TLS].packets_analyzed == 1`.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Both versions in tls_versions; correct JA3 from ClientHello version; correct JA3S from ServerHello version.
- **Edge case handling** (weight: 0.3): Version 0x0301 (TLS 1.0) is above the deprecated-protocol threshold (> 0x0300) so must NOT trigger a finding.
- **Error quality** (weight: 0.1): handshakes_seen correctly reflects one complete handshake pair.
- **Data integrity** (weight: 0.1): version_counts is shared between ClientHello and ServerHello processing — both contribute to the same map.

## Edge Conditions

- The same version_counts map is used by both handle_client_hello and handle_server_hello — a version seen in both hellos would have count 2 in the map.
- TLS 1.0 (0x0301) is specifically above the deprecated threshold — this is a common gotcha (the threshold is `<= 0x0300`, not `<= 0x0301`).

## Failure Guidance

"HOLDOUT LOW: HS-071 (satisfaction: 0.XX) -- TLS version tracking was incorrect; client and server versions should both appear in tls_versions; TLS 1.0 (0x0301) should not trigger a deprecated-protocol finding."
