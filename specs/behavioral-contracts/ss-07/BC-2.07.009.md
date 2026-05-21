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

# BC-2.07.009: Weak Client Cipher in ClientHello Emits Anomaly/Likely/High Finding

## Description

During `handle_client_hello`, after computing JA3, the cipher list is scanned for weak
ciphers (NULL, ANON, EXPORT cipher names). If any weak ciphers are found, a single
`Anomaly/Likely/High` finding is pushed to `all_findings` with the names of all weak
ciphers in the `evidence` vec. The `direction` is `Some(Direction::ClientToServer)`.
There is no MITRE technique ID on this finding.

Note: The `evidence` vec contains one string per weak cipher name, so its length is
data-dependent (O-06 domain debt). See BC-INDEX note and cap-07 §weak-cipher evidence
cardinality.

## Preconditions

1. A complete TLS ClientHello is being processed by `handle_client_hello`.
2. At least one cipher in `ch.ciphers` satisfies `is_weak_cipher(id) == true`.
3. `is_weak_cipher` returns true when the cipher name (from `TlsCipherSuite::from_id`)
   contains "NULL", "ANON", or "EXPORT" (case-insensitive uppercase comparison).

## Postconditions

1. One finding is pushed to `all_findings` with:
   - category: Anomaly
   - verdict: Likely
   - confidence: High
   - summary: "ClientHello offers weak cipher suites (NULL/anonymous/export)"
   - evidence: Vec<String> with one entry per weak cipher name (NOT hex; readable names)
   - mitre_technique: None
   - source_ip: None
   - timestamp: None
   - direction: Some(Direction::ClientToServer)
2. Exactly ONE finding per ClientHello, regardless of how many weak ciphers exist.
3. Strong ciphers in the same list do not suppress the finding.

## Invariants

1. GREASE-valued cipher IDs never trigger a weak-cipher finding because
   `TlsCipherSuite::from_id(id.0)` returns `None` for GREASE values (they are not
   in the cipher suite database), and `is_weak_cipher` returns `false` for `None`.
   There is NO explicit GREASE pre-filter on the weak-cipher scan; the scan operates
   on the raw `ch.ciphers` list (lines 497-502). GREASE immunity is a consequence of
   the `None`-returns-false branch, not an explicit filter.
2. If `TlsCipherSuite::from_id(id.0)` returns None (unknown cipher), `is_weak_cipher`
   returns false -- unknown ciphers do NOT trigger the finding.
3. The evidence vec has data-dependent cardinality (O-06). There is no per-cipher cap
   in the current implementation (issue #102). Upper bound is approximately 9,216 names.
4. Raw cipher names are stored in evidence (ADR 0003 / INV-4).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single weak cipher TLS_NULL_WITH_NULL_NULL | One finding; evidence = ["TLS_NULL_WITH_NULL_NULL"] |
| EC-002 | Multiple weak ciphers (NULL + ANON + EXPORT) | One finding; evidence has all weak cipher names |
| EC-003 | Cipher list has only strong ciphers | No finding |
| EC-004 | Unknown cipher ID (not in tls_parser db) | Not counted as weak; no finding from it |
| EC-005 | Weak cipher alongside strong ciphers | One finding; only the weak cipher names in evidence |
| EC-006 | Weak cipher when all_findings is large (stress test) | Finding still pushed; TlsAnalyzer.all_findings is unbounded (no MAX_FINDINGS cap at this layer) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello with TLS_RSA_EXPORT_WITH_RC4_40_MD5 cipher | Finding(Anomaly/Likely/High, no MITRE, direction=ClientToServer) | happy-path |
| ClientHello with only AES-256-GCM ciphers | No finding | happy-path |
| ClientHello with NULL_WITH_NULL_NULL + ANON_WITH_NULL + strong cipher | One finding; evidence has 2 entries | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Weak client cipher produces exactly one finding | unit: test_weak_cipher_finding_client |
| — | Strong ciphers produce no finding | unit: test_normal_handshake_no_findings |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- weak client cipher detection is one of the 7 TLS anomaly findings described in cap-07 |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation -- cipher names stored as raw strings) |
| Architecture Module | SS-07 (analyzer/tls.rs:497-517, C-13) |
| Stories | STORY-054 |
| Origin BC | BC-TLS-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.010 -- related to (server cipher counterpart -- different verdict field: Medium)
- BC-2.07.001 -- depends on (triggered during ClientHello handling)

## Architecture Anchors

- `src/analyzer/tls.rs:56-64` -- `is_weak_cipher` function (NULL/ANON/EXPORT check)
- `src/analyzer/tls.rs:497-517` -- weak cipher collection and finding push in handle_client_hello
- `tests/tls_analyzer_tests.rs` -- test_weak_cipher_finding_client

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:497-517` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if !weak.is_empty() { self.all_findings.push(...) }`
- **assertion**: test_weak_cipher_finding_client; tls_integration_tests test_ssl30_pcap_generates_findings

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
