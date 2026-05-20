---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.010: Weak Server Cipher Selected Emits Anomaly/Likely/Medium Finding

## Description

During `handle_server_hello`, after computing JA3S and tracking cipher counts, the
selected cipher (`sh.cipher`) is tested with `is_weak_server_cipher`. This function
extends the client-cipher weak set with RC4 ciphers. If the server has negotiated a weak
cipher, a single `Anomaly/Likely/Medium` finding is pushed. The confidence is Medium
(not High as for the client-side finding) because the server makes the final selection.

## Preconditions

1. A complete TLS ServerHello is being processed by `handle_server_hello`.
2. `is_weak_server_cipher(sh.cipher)` returns true.
3. `is_weak_server_cipher` = `is_weak_cipher(id)` (NULL/ANON/EXPORT) OR cipher name
   contains "RC4".

## Postconditions

1. One finding is pushed to `all_findings` with:
   - category: Anomaly
   - verdict: Likely
   - confidence: Medium
   - summary: format "ServerHello selected weak cipher suite ({name})"
   - evidence: ["Selected cipher: {name} (0x{id:04x})"]
   - mitre_technique: None
   - source_ip: None
   - timestamp: None
   - direction: Some(Direction::ServerToClient)
2. Exactly ONE finding per ServerHello.
3. The cipher name in summary and evidence is the human-readable name from
   `TlsCipherSuite::from_id`, or the `0xNNNN` hex fallback for unknown IDs
   (see BC-2.07.036).

## Invariants

1. `is_weak_server_cipher` is a strict superset of `is_weak_cipher` (adds RC4).
2. The finding direction is `ServerToClient` (not ClientToServer).
3. The confidence is Medium for server selection -- even though the server chose it,
   the client offered it, so both bear responsibility.
4. Evidence is exactly one string: cipher name plus hex ID.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Server selects TLS_RSA_WITH_RC4_128_MD5 | Finding(Anomaly/Likely/Medium, direction=ServerToClient) |
| EC-002 | Server selects TLS_NULL_WITH_NULL_NULL | Finding emitted (is_weak_cipher also covers this) |
| EC-003 | Server selects AES-256-GCM | No finding |
| EC-004 | Unknown cipher ID (0xFFFF) | is_weak_server_cipher returns false (from_id returns None); no finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ServerHello selecting TLS_RSA_EXPORT_WITH_RC4_40_MD5 | Finding(Anomaly/Likely/Medium, direction=ServerToClient) | happy-path |
| ServerHello selecting TLS_AES_256_GCM_SHA384 | No finding | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Weak server cipher produces Anomaly/Likely/Medium finding | unit: test_weak_cipher_finding_server |
| VP-TBD | Direction is ServerToClient | unit: test_weak_cipher_finding_server asserts direction |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- weak server cipher selection is one of the 7 TLS anomaly findings described in cap-07 |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:570-582, C-16) |
| Stories | S-TBD |
| Origin BC | BC-TLS-010 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.009 -- related to (client cipher counterpart -- Anomaly/Likely/High)
- BC-2.07.002 -- depends on (triggered during ServerHello handling)
- BC-2.07.036 -- composes with (cipher_name hex fallback for unknown IDs)

## Architecture Anchors

- `src/analyzer/tls.rs:66-75` -- `is_weak_server_cipher` (adds RC4 check)
- `src/analyzer/tls.rs:570-582` -- server weak-cipher finding push
- `tests/tls_analyzer_tests.rs` -- test_weak_cipher_finding_server

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:570-582` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if is_weak_server_cipher(sh.cipher) { self.all_findings.push(...) }`
- **assertion**: test_weak_cipher_finding_server

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
