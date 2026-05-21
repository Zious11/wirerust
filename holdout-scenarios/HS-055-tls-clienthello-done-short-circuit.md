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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.003.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.032.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.034.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-051.md
id: "HS-055"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.001
  - BC-2.07.003
  - BC-2.07.034
  - BC-2.07.032
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: TLS Analyzer Counts Handshakes Once and Ignores Post-Handshake Data

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains a complete TLS 1.3 session: ClientHello, ServerHello, followed by many megabytes of TLS application data records (record type 0x17) for the rest of the connection.
2. The analyst runs wirerust on this pcap.
3. The TLS analyzer summary shows `packets_analyzed == 1` (one handshake pair counted, not one per application-data record).
4. The `ja3_hashes` map contains exactly one entry from the ClientHello.
5. The `ja3s_hashes` map contains exactly one entry from the ServerHello.
6. The `parse_errors` counter is 0.
7. No findings are emitted from the application data phase.
8. A separate but related check: if the same pcap is re-processed after the handshake completes, the counters remain stable (idempotent after done).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.001 | postcondition 1-8 | ClientHello correctly populates handshake_seen, version_counts, ja3_counts, sni_counts |
| BC-2.07.003 | postcondition 1-5; invariant 1-2 | After both hellos, subsequent bytes return immediately without state mutation |
| BC-2.07.034 | postcondition 1-3 | on_data short-circuit is the first gate; 1 MB burst of app data leaves all counters unchanged |
| BC-2.07.032 | postcondition 1-3 | TLS 1.3 legacy_version 0x0303 is recorded; no deprecated-protocol finding emitted |

## Verification Approach

Use a pcap of a real TLS 1.3 HTTPS session (e.g., a curl request to httpbin.org captured with tcpdump). Run wirerust.

1. Assert `analyzers[TLS].packets_analyzed == 1`.
2. Assert `analyzers[TLS].detail.ja3_hashes` has exactly one key.
3. Assert `analyzers[TLS].detail.ja3s_hashes` has exactly one key.
4. Assert `analyzers[TLS].detail.parse_errors == 0`.
5. Assert `analyzers[TLS].detail.truncated_records == 0`.
6. Assert `findings` contains no TLS-sourced findings (no deprecated protocol, no weak cipher for a modern TLS 1.3 session).
7. Assert `analyzers[TLS].detail.tls_versions` contains key `"771"` (decimal for 0x0303).

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): packets_analyzed=1; single JA3 and JA3S entry; no parse_errors; version 771 counted.
- **Edge case handling** (weight: 0.3): Application data records do not increment handshakes_seen or emit findings.
- **Error quality** (weight: 0.15): No false-positive findings for normal modern TLS.
- **Data integrity** (weight: 0.1): All 7 required TLS summary keys present in BTreeMap.

## Edge Conditions

- Large application data phase (megabytes) — the done-check must fire at entry rather than parsing all records.
- TLS 1.3 uses legacy_version 0x0303 in ClientHello — version counter must be 771, not some TLS 1.3-specific value.
- post-handshake application data record type 0x17 — must be consumed silently and not counted as parse_errors.

## Failure Guidance

"HOLDOUT LOW: HS-055 (satisfaction: 0.XX) -- TLS analyzer did not short-circuit after handshake, or packet count exceeded 1, or application data was misinterpreted as a parse error."
