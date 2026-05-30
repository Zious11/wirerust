---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: add VP back-ref for client version_name reachability test (F-S054-P3-003) — 2026-05-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.011: Deprecated Client Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding

## Description

During `handle_client_hello`, after processing SNI and JA3, the ClientHello version is
tested against the deprecated-protocol threshold (`version <= 0x0300`). SSL 2.0 (0x0200)
and SSL 3.0 (0x0300) are both deprecated by RFC 7568. Any version at or below 0x0300
triggers a finding naming the specific version. The finding direction is ClientToServer.
There is no MITRE technique ID on this finding.

## Preconditions

1. A complete TLS ClientHello is being processed by `handle_client_hello`.
2. `ch.version.0 <= 0x0300` (i.e., the negotiated version is SSLv3 or earlier).

## Postconditions

1. One finding is pushed to `all_findings` with:
   - category: Anomaly
   - verdict: Likely
   - confidence: High
   - summary: "ClientHello uses deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)"
   - evidence: ["Version: 0x{version:04x} ({version_name})"]
   - mitre_technique: None
   - source_ip: None
   - timestamp: None
   - direction: Some(Direction::ClientToServer)
2. The version_name is mapped: 0x0200 -> "SSL 2.0", 0x0300 -> "SSL 3.0",
   anything else -> "Unknown legacy SSL".
3. `handshakes_seen` is already incremented at the top of `handle_client_hello`.

## Invariants

1. TLS 1.0 (0x0301) does NOT trigger the finding (threshold is strictly <= 0x0300).
2. The summary text always contains "RFC 7568" as a normative reference.
3. This finding is independent of any weak-cipher finding: both can fire in the same
   ClientHello if it offers SSL 3.0 AND a weak cipher.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | version = 0x0200 (SSL 2.0) | Finding with "SSL 2.0" in summary |
| EC-002 | version = 0x0300 (SSL 3.0) | Finding with "SSL 3.0" in summary |
| EC-003 | version = 0x0100 (below SSL 2.0, unknown) | Finding with "Unknown legacy SSL" |
| EC-004 | version = 0x0301 (TLS 1.0) | No finding; threshold boundary just above |
| EC-005 | SSL 3.0 ClientHello with weak cipher | Two findings: deprecated-protocol AND weak-cipher |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello with version 0x0300 | Finding(Anomaly/Likely/High, "SSL 3.0", direction=ClientToServer) | happy-path |
| ClientHello with version 0x0303 (TLS 1.2) | No deprecated-protocol finding | happy-path |
| ClientHello with version 0x0200 | Finding with "SSL 2.0" in summary | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SSL 3.0 ClientHello produces Anomaly/Likely/High finding | integration: test_ssl30_pcap_generates_findings |
| — | TLS 1.0 ClientHello does not produce deprecated-protocol finding | unit: threshold boundary test |
| — | EC-001 (0x0200 → "SSL 2.0") and EC-003 (0x0100 → "Unknown legacy SSL") version_name arms are reachable client-side; correct finding produced | unit: test_BC_2_07_011_client_deprecated_version_name_ssl2_and_legacy |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- deprecated client protocol detection is one of the 7 TLS anomaly findings described in cap-07 |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:519-539, C-13) |
| Stories | STORY-054 |
| Origin BC | BC-TLS-011 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.012 -- related to (server-side deprecated protocol counterpart)
- BC-2.07.001 -- depends on (triggered during ClientHello handling)

## Architecture Anchors

- `src/analyzer/tls.rs:519-539` -- deprecated client version check and finding push
- `tests/tls_integration_tests.rs` -- test_ssl30_pcap_generates_findings

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:519-539` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if version <= 0x0300 { ... }` at tls.rs:519
- **assertion**: test_ssl30_pcap_generates_findings (integration test)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
