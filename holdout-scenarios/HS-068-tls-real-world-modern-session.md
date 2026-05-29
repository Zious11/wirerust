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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.001.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.002.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.030.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.031.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.034.md
input-hash: "6e52bc5"
traces_to: .factory/stories/STORY-051.md
id: "HS-068"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.001
  - BC-2.07.002
  - BC-2.07.030
  - BC-2.07.031
  - BC-2.07.034
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Known-Good TLS 1.3 Traffic Produces Zero Findings and Correct JA3 Fingerprints

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap of modern HTTPS traffic captured from a Chrome browser connecting to google.com or a similar major TLS 1.3 endpoint is analyzed.
2. The traffic contains TLS 1.3 ClientHellos (with GREASE values), ServerHellos, followed by TLS application data records.
3. The analyst runs wirerust on this pcap.
4. Expected: zero TLS-sourced findings (no weak ciphers, no deprecated protocols, no SNI anomalies). The `handshakes_seen` counter is >= 1. The `ja3_hashes` map contains at least one 32-character lowercase hex key. The `top_snis` field contains recognizable domain names (e.g., `google.com`). All TLS 1.3 ClientHellos show version 771 (0x0303) in `tls_versions` as the decimal key.
5. No panic or tool crash occurs.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.001 | postcondition 1-8 | ClientHello correctly parsed; handshakes_seen, version_counts, ja3_counts, sni_counts populated |
| BC-2.07.002 | postcondition 1-4 | ServerHello parsed; server_hello_seen set; JA3S computed; cipher_counts populated |
| BC-2.07.030 | postcondition 1-4 | Modern TLS handshake with strong cipher produces zero findings |
| BC-2.07.031 | postcondition 1-9 | Summary output complete with all 7 detail keys |
| BC-2.07.034 | postcondition 1-3 | Application data records do not re-enter processing after done |

## Verification Approach

corpus_source: TLS 1.3 session captured with tcpdump from Chrome connecting to google.com, or a Wireshark sample capture with TLS 1.3 (e.g., `tls12.pcapng` from Wireshark wiki for comparison, or any modern browser HTTPS capture).
corpus_size: typically 200-500 packets for a single HTTPS session
known_edge_cases: GREASE values in cipher list; large ClientHello due to many extensions

Run wirerust on the pcap with JSON output.

1. Assert `findings` array contains zero TLS-sourced findings.
2. Assert `analyzers[TLS].packets_analyzed >= 1`.
3. Assert each key in `analyzers[TLS].detail.ja3_hashes` is exactly 32 lowercase hex characters.
4. Assert `analyzers[TLS].detail.tls_versions` contains key `"771"`.
5. Assert `analyzers[TLS].detail.top_snis` is non-empty and contains recognizable domain names.
6. Assert `analyzers[TLS].detail.parse_errors == 0`.
7. Assert wirerust exits with status 0.

false_positive_threshold: 0 (zero tolerance for findings on modern browser TLS)
false_negative_threshold: N/A (this is a known-good corpus; no findings expected)

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Zero findings; ja3_hashes present and correctly formatted; tls_versions in decimal.
- **Edge case handling** (weight: 0.2): GREASE values in Chrome ClientHello filtered out before JA3 computation.
- **Error quality** (weight: 0.2): No parse_errors; no truncated_records; clean exit.
- **Data integrity** (weight: 0.1): top_snis contains readable domain names; all 7 detail keys present.

## Edge Conditions

- Chrome ClientHello typically includes 10-15 GREASE values — all must be filtered for JA3.
- Application data records (post-handshake) must not increment handshakes_seen.
- TLS 1.3 session ticket messages and encrypted extensions are non-handshake records that must be silently skipped.

## Failure Guidance

"HOLDOUT LOW: HS-068 (satisfaction: 0.XX) -- TLS analyzer produced false-positive findings or wrong fingerprints on known-good modern HTTPS traffic; verify GREASE filtering, JA3 format (32 hex chars), and done-check short-circuit."
