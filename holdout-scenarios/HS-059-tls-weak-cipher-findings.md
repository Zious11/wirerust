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
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.030.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.036.md
input-hash: "08c9d58"
traces_to: .factory/stories/STORY-051.md
id: "HS-059"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-5"
behavioral_contracts:
  - BC-2.07.009
  - BC-2.07.010
  - BC-2.07.011
  - BC-2.07.012
  - BC-2.07.030
  - BC-2.07.036
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Weak Cipher and Deprecated Protocol Findings Are Confidence-Correct and Independent

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

1. A pcap contains three TLS sessions:
   - Session 1: Modern TLS 1.3 ClientHello (version 0x0303) + TLS 1.3 ServerHello with strong cipher — produces ZERO findings.
   - Session 2: SSL 3.0 ClientHello (version 0x0300) with TLS_NULL_WITH_NULL_NULL in cipher list + SSL 3.0 ServerHello selecting TLS_RSA_WITH_RC4_128_MD5.
   - Session 3: TLS 1.0 ClientHello (version 0x0301) with strong ciphers — produces ZERO findings (TLS 1.0 is above the deprecated threshold).
2. The analyst runs wirerust on this pcap.
3. Session 2 generates exactly 4 findings: (a) weak client cipher (High confidence), (b) deprecated client protocol SSL 3.0 (High confidence), (c) weak server cipher RC4 (Medium confidence), (d) deprecated server protocol SSL 3.0 (High confidence).
4. Session 1 and Session 3 generate zero findings.
5. All Session 2 findings have `mitre_technique` as null.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.07.009 | postcondition 1-2; invariant 1-2 | Weak client cipher emits one High-confidence finding; GREASE immune |
| BC-2.07.010 | postcondition 1-2; invariant 1 | RC4 server cipher emits one Medium-confidence finding |
| BC-2.07.011 | postcondition 1-2; invariant 1-3 | SSL 3.0 client triggers deprecated protocol finding with "RFC 7568" in summary |
| BC-2.07.012 | postcondition 1-2; invariant 1-2 | SSL 3.0 server triggers deprecated protocol finding with ServerToClient direction |
| BC-2.07.030 | postcondition 1-4 | Modern TLS 1.3 and TLS 1.0 sessions produce zero findings |
| BC-2.07.036 | postcondition 1-2; invariant 1-2 | cipher_name renders unknown IDs as "0xNNNN" lowercase hex |

## Verification Approach

Craft a pcap (or use crafted byte arrays in integration tests) with the three sessions. Run wirerust with JSON output.

1. Assert exactly 4 findings in the `findings` array.
2. For the weak-client-cipher finding: assert `verdict == "Likely"`, `confidence == "High"`, `direction == "ClientToServer"`, `mitre_technique == null`.
3. For the weak-server-cipher finding: assert `verdict == "Likely"`, `confidence == "Medium"`, `direction == "ServerToClient"`.
4. For each deprecated-protocol finding: assert `summary` contains "RFC 7568"; assert one has `direction == "ClientToServer"`, one has `direction == "ServerToClient"`.
5. Assert zero findings from sessions 1 and 3.
6. Assert `analyzers[TLS].detail.tls_versions` contains entries for version 768 (SSL 3.0 = 0x0300), 769 (TLS 1.0 = 0x0301), and 771 (TLS 1.3 ClientHello legacy = 0x0303).

## Evaluation Rubric

- **Functional correctness** (weight: 0.45): Exactly 4 findings from session 2; zero from sessions 1 and 3; correct confidence levels.
- **Edge case handling** (weight: 0.3): TLS 1.0 (0x0301) is above threshold and must NOT trigger deprecated-protocol finding; RC4 triggers server finding but not client finding (different predicate).
- **Error quality** (weight: 0.15): "RFC 7568" appears in deprecated-protocol summaries; cipher name in evidence is human-readable (not just hex).
- **Data integrity** (weight: 0.1): MITRE technique is null for all cipher/protocol weakness findings.

## Edge Conditions

- TLS 1.0 (version 0x0301) is ABOVE the `<= 0x0300` threshold — it must not be flagged.
- RC4 triggers `is_weak_server_cipher` but NOT `is_weak_cipher` (client side) — asymmetric predicate.
- One NULL cipher in a list with many strong ciphers still triggers the client weak-cipher finding.
- Both deprecated-protocol findings for SSL 3.0 must have distinct directions.

## Failure Guidance

"HOLDOUT LOW: HS-059 (satisfaction: 0.XX) -- TLS weak-cipher or deprecated-protocol findings were wrong count, wrong confidence, or TLS 1.0 was incorrectly flagged; check deprecated-protocol threshold and RC4 asymmetry."
