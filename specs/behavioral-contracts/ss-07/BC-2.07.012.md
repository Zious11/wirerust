---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.012: Deprecated Server Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding

## Description

During `handle_server_hello`, after tracking the version, if `sh.version.0 <= 0x0300`
(SSL 2.0 or SSL 3.0), a finding is pushed with `Anomaly/Likely/High` and direction
`ServerToClient`. This is the server-side complement to BC-2.07.011. The summary text
says "negotiated" instead of "uses" to reflect that the server finalizes the version
selection.

## Preconditions

1. A complete TLS ServerHello is being processed by `handle_server_hello`.
2. `sh.version.0 <= 0x0300`.

## Postconditions

1. One finding is pushed to `all_findings` with:
   - category: Anomaly
   - verdict: Likely
   - confidence: High
   - summary: "ServerHello negotiated deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)"
   - evidence: ["Version: 0x{version:04x} ({version_name})"]
   - mitre_technique: None
   - source_ip: None
   - timestamp: None
   - direction: Some(Direction::ServerToClient)
2. `version_name` mapping is identical to BC-2.07.011.

## Invariants

1. TLS 1.0 (0x0301) does NOT trigger the finding.
2. The server-side finding uses direction `ServerToClient`; client-side uses
   `ClientToServer`. These are distinguishable in JSON output.
3. Can co-occur with the client-side deprecated-version finding if both hellos are SSL 3.0.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ServerHello version = 0x0300 | Finding(Anomaly/Likely/High, "SSL 3.0", direction=ServerToClient) |
| EC-002 | ServerHello version = 0x0301 | No finding; boundary just above threshold |
| EC-003 | ClientHello SSL 3.0 + ServerHello SSL 3.0 | Two findings: one ClientToServer, one ServerToClient |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ServerHello version=0x0300 | Finding(Anomaly/Likely/High, direction=ServerToClient, "SSL 3.0") | happy-path |
| ServerHello version=0x0303 | No deprecated-protocol finding | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Deprecated server version produces Anomaly/Likely/High finding | unit (no independent test; refactor-risk noted in R4 MEDIUM verdict) |
| — | Direction is ServerToClient | unit assertion on finding.direction field |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- deprecated server protocol detection is one of the 7 TLS anomaly findings described in cap-07 |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:584-604, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-012 (pass-3 ingestion corpus, MEDIUM confidence -- no independent server-side test) |

## Related BCs

- BC-2.07.011 -- related to (client-side deprecated protocol counterpart)
- BC-2.07.002 -- depends on (triggered during ServerHello handling)

## Architecture Anchors

- `src/analyzer/tls.rs:584-604` -- deprecated server version check and finding push

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:584-604` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if version <= 0x0300 { ... }` at tls.rs:584
- **documentation**: no independent unit test for server-side path; shares code structure with BC-2.07.011

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, version_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
