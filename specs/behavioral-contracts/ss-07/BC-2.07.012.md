---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.3: tls-parser-0.12 reachability correction (F-S054-P1-002) — 2026-05-29: SSL 2.0 and sub-0x0200 ServerHello version_name arms are defensive/unreachable under tls-parser 0.12; pin test documented"
  - "v1.4: reconcile pin-test rename (ec004_ec005) + correct BC-2.07.011 EC cross-ref (EC-001/EC-003, not EC-006/EC-007) — F-S054-P3-001 — 2026-05-29"
  - "v1.5: mitre_technique: None → mitre_techniques: vec![] in Postconditions (ARP-F2 P14 B6) — 2026-06-13"
  - "v1.6: PG-ARP-F2-007 ss-07 full re-anchor — deprecated server version 584-604→630-650; version_name match 586-590→631-635 — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.012: Deprecated Server Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding

## Description

During `handle_server_hello`, after tracking the version, if `sh.version.0 <= 0x0300`,
a finding is pushed with `Anomaly/Likely/High` and direction `ServerToClient`. This is
the server-side complement to BC-2.07.011. The summary text says "negotiated" instead
of "uses" to reflect that the server finalizes the version selection.

**tls-parser 0.12 reachability constraint (F-S054-P1-002):** Under tls-parser 0.12, a
ServerHello record with version field `0x0200` (SSL 2.0) or any sub-0x0200 value is
rejected at the record-layer parser before `handle_server_hello` is reached. Therefore,
the only reachable deprecated-version arm under current tls-parser is `0x0300` (SSL 3.0).
The `0x0200` ("SSL 2.0") and sub-0x0200 catchall ("Unknown legacy SSL") arms in the
`version_name` match are defensive code, correct in logic, but not wire-triggerable. See
EC-004 and EC-005. Contrast with the client side (BC-2.07.011): tls-parser accepts SSL 2.0
and sub-0x0200 ClientHello records, so those arms ARE reachable client-side.

## Preconditions

1. A complete TLS ServerHello has been parsed by `handle_server_hello` (i.e., tls-parser
   successfully decoded the record at the record layer).
2. `sh.version.0 <= 0x0300`.
3. Under tls-parser 0.12, precondition 2 is only satisfiable via `sh.version.0 == 0x0300`
   (SSL 3.0). ServerHellos with version `0x0200` or lower are rejected by tls-parser
   before this handler is invoked (parse_errors incremented; handler not reached).

## Postconditions

1. One finding is pushed to `all_findings` with:
   - category: Anomaly
   - verdict: Likely
   - confidence: High
   - summary: "ServerHello negotiated deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)"
   - evidence: ["Version: 0x{version:04x} ({version_name})"]
   - mitre_techniques: vec![]
   - source_ip: None
   - timestamp: None
   - direction: Some(Direction::ServerToClient)
2. `version_name` mapping: `0x0200` -> "SSL 2.0", `0x0300` -> "SSL 3.0", anything else ->
   "Unknown legacy SSL". The `0x0200` and catchall arms are defensive; under tls-parser 0.12
   they are unreachable via ServerHello wire input (see EC-004, EC-005).

## Invariants

1. TLS 1.0 (0x0301) does NOT trigger the finding.
2. The server-side finding uses direction `ServerToClient`; client-side uses
   `ClientToServer`. These are distinguishable in JSON output.
3. Can co-occur with the client-side deprecated-version finding if both hellos are SSL 3.0.
4. Under tls-parser 0.12: a ServerHello with version `0x0200` or lower is rejected at the
   record layer. `handle_server_hello` is never invoked, so `version_counts` does not
   record the version, `ja3s_counts` is not updated, and no deprecated-protocol finding
   with direction `ServerToClient` is produced. This mirrors the SSL 2.0 ClientHello
   ServerHello rejection already documented in BC-2.07.002 EC-004.

## Edge Cases

| ID | Description | Expected Behavior | Reachable (tls-parser 0.12) |
|----|-------------|-------------------|-----------------------------|
| EC-001 | ServerHello version = 0x0300 (SSL 3.0) | Finding(Anomaly/Likely/High, "SSL 3.0", direction=ServerToClient) | Yes |
| EC-002 | ServerHello version = 0x0301 (TLS 1.0) | No finding; boundary just above threshold | Yes |
| EC-003 | ClientHello SSL 3.0 + ServerHello SSL 3.0 | Two findings: one ClientToServer, one ServerToClient | Yes |
| EC-004 | ServerHello version = 0x0200 (SSL 2.0) — DEFENSIVE/UNREACHABLE | tls-parser rejects record at record layer; parse_errors++; handle_server_hello not reached; no finding produced; version_name "SSL 2.0" arm is correct but wire-untriggerable under tls-parser 0.12. Pinned by test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin. | No (parser rejection) |
| EC-005 | ServerHello version < 0x0200 (e.g., 0x0100) — DEFENSIVE/UNREACHABLE catchall | Same as EC-004: tls-parser rejects at record layer; version_name "Unknown legacy SSL" catchall arm is correct but wire-untriggerable under tls-parser 0.12. Also pinned by test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin (covers both 0x0200 and sub-0x0200 via single assertion). | No (parser rejection) |

> **Upgrade guard (EC-004 / EC-005):** If tls-parser is upgraded to a version that
> accepts SSL 2.0 ServerHello records, EC-004 and EC-005 transition from "parser
> rejection" to "positive finding" behavior. The production code at `tls.rs:631-635`
> already handles both arms correctly. In that case, convert
> `test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin` from a pin test
> to an assertion test matching the BC-2.07.011 EC-001/EC-003 client-side pattern
> (= STORY-054 EC-006/EC-007).

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ServerHello version=0x0300 | Finding(Anomaly/Likely/High, direction=ServerToClient, "SSL 3.0") | happy-path |
| ServerHello version=0x0303 | No deprecated-protocol finding | happy-path |
| ServerHello version=0x0200 (tls-parser 0.12) | parse_errors=1; handle_server_hello not reached; no ServerToClient finding; version_counts[0x0200]=0 | pin (EC-004) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SSL 3.0 ServerHello produces Anomaly/Likely/High finding, direction=ServerToClient | unit: test_server_ssl30_deprecated_finding |
| — | Direction is ServerToClient | unit assertion on finding.direction field |
| — | ServerHello version=0x0200 is rejected by tls-parser 0.12 before handle_server_hello (EC-004 pin) | unit: test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin — asserts parse_errors=1, version_counts[0x0200]=0, ja3s_counts empty, no ServerToClient deprecated-protocol finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- deprecated server protocol detection is one of the 7 TLS anomaly findings described in cap-07 |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:630-650, C-13) |
| Stories | STORY-054 |
| Origin BC | BC-TLS-012 (pass-3 ingestion corpus, MEDIUM confidence -- no independent server-side test at ingestion; EC-004/EC-005 reachability corrected by F-S054-P1-002 probe) |

## Related BCs

- BC-2.07.011 -- related to (client-side deprecated protocol counterpart; client-side 0x0200 IS reachable, contrast with server-side EC-004)
- BC-2.07.002 -- depends on (triggered during ServerHello handling; BC-2.07.002 EC-004 documents analogous SSL 2.0 ServerHello rejection by tls-parser)

## Architecture Anchors

- `src/analyzer/tls.rs:630-650` -- deprecated server version check and finding push
- `src/analyzer/tls.rs:631-635` -- version_name match arms (0x0200 and catchall are defensive/unreachable under tls-parser 0.12)
- `tests/tls_analyzer_tests.rs` -- test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin (EC-004/EC-005 pin; asserts parse_errors=1 when ServerHello version=0x0200)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:630-650` |
| **Confidence** | high (EC-001/EC-002/EC-003); pin-documented (EC-004/EC-005) |
| **Extraction Date** | 2026-05-20 |
| **Reachability Probe Date** | 2026-05-29 (F-S054-P1-002) |

## Evidence Types Used

- **guard clause**: `if version <= 0x0300 { ... }` at tls.rs:630
- **pin test**: test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin — empirically confirms 0x0200 ServerHello rejected at record layer under tls-parser 0.12
- **documentation**: EC-004/EC-005 server-side arms are defensive; client-side equivalents (BC-2.07.011 EC-001/EC-003 = STORY-054 EC-006/EC-007) ARE reachable

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, version_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
