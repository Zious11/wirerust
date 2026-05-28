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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.009.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.010.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.011.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.012.md
input-hash: "08c9d58"
traces_to: .factory/stories/STORY-051.md
id: "HS-074"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.011
  - BC-2.07.012
  - BC-2.07.009
  - BC-2.07.010
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Known-Problematic SSL 3.0 pcap Generates Expected Deprecated-Protocol Findings

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap containing a real SSL 3.0 handshake (Wireshark sample `ssl.cap` or any pcap from a penetration testing corpus containing POODLE attack demonstrations) is analyzed by wirerust.
2. The pcap contains at least one ClientHello with version 0x0300 (SSL 3.0) and one ServerHello that also negotiates 0x0300.
3. The analyst runs wirerust on this pcap.
4. Expected findings: at least one deprecated-protocol finding for the ClientHello with `direction == "ClientToServer"` and at least one for the ServerHello with `direction == "ServerToClient"`. Both findings have `summary` containing "RFC 7568".
5. If the SSL 3.0 session uses weak ciphers (as common in older SSL implementations), additional weak-cipher findings may appear — this is expected and correct behavior.
6. The tool does not crash or panic on this older-format pcap.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.011 | postcondition 1-2; invariant 1-3 | Deprecated client protocol (<= SSLv3) emits Anomaly/Likely/High finding with RFC 7568 reference |
| BC-2.07.012 | postcondition 1-2; invariant 1-2 | Deprecated server protocol emits finding with ServerToClient direction |
| BC-2.07.009 | postcondition 1-2 | Weak client cipher (if present in SSL 3.0 session) emits Anomaly/Likely/High finding |
| BC-2.07.010 | postcondition 1-2 | Weak server cipher (if RC4 or similar selected) emits Anomaly/Likely/Medium finding |

## Verification Approach

corpus_source: Wireshark SSL sample capture `ssl.cap` (https://wiki.wireshark.org/SampleCaptures#SSL_with_decryption_keys) or equivalent POODLE demonstration pcap containing SSL 3.0 ClientHello/ServerHello.
corpus_size: typically 10-50 packets for a single SSL 3.0 handshake
known_edge_cases: SSL 3.0 may use export-grade ciphers; ClientHello version 0x0300 triggers deprecated finding

Run wirerust on the SSL 3.0 pcap with JSON output.

1. Assert `findings` array is non-empty.
2. Assert at least 2 findings have `summary` containing "RFC 7568".
3. Assert one of those findings has `direction == "ClientToServer"` (client-side deprecated protocol).
4. Assert one of those findings has `direction == "ServerToClient"` (server-side deprecated protocol).
5. Assert all deprecated-protocol findings have `confidence == "High"` and `verdict == "Likely"`.
6. Assert `mitre_technique` is null for all cipher/protocol weakness findings.
7. Assert wirerust exits with status 0 (no crash on old SSL pcap).

false_negative_threshold: 0 — all SSL 3.0 sessions in the corpus must produce deprecated-protocol findings.

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Deprecated protocol findings present for SSL 3.0 ClientHello and ServerHello; correct confidence and direction.
- **Edge case handling** (weight: 0.25): Real-world SSL 3.0 traffic parsed without panicking; export/null ciphers also detected if present.
- **Error quality** (weight: 0.2): "RFC 7568" present in deprecated-protocol summaries; directions correctly differentiated.
- **Data integrity** (weight: 0.1): Tool exits cleanly; no panics on older pcap format.

## Edge Conditions

- SSL 3.0 version 0x0300 is the EXACT boundary — it triggers the `<= 0x0300` condition.
- TLS 1.0 (0x0301) would not trigger the deprecated finding, even in the same pcap (different flow).
- Some SSL 3.0 sessions use NULL ciphers or EXPORT ciphers — these generate additional weak-cipher findings.

## Failure Guidance

"HOLDOUT LOW: HS-074 (satisfaction: 0.XX) -- SSL 3.0 pcap did not generate expected deprecated-protocol findings; check that version 0x0300 triggers the finding, both ClientToServer and ServerToClient directions are detected, and RFC 7568 appears in the summary."
