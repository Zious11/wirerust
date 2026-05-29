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
  - "v1.3: correct EC-004 to document tls-parser 0.12 SSL2 ServerHello parse-rejection (F-S054-P6-001); resolves cross-BC contradiction with BC-2.07.012 — 2026-05-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.002: Parse Complete TLS ServerHello: JA3S Fingerprint Computed

## Description

When a complete TLS ServerHello record arrives on the server direction of a flow,
`TlsAnalyzer` extracts the negotiated protocol version, selected cipher suite, and
extensions. It computes the JA3S MD5 fingerprint from `version,cipher,extensions`,
stores it in `ja3s_counts`, and tracks the cipher name in `cipher_counts`. If the
negotiated cipher is weak, an Anomaly/Likely/Medium finding is emitted. If the version
is SSL 3.0 (0x0300) or lower and reachable under tls-parser, an Anomaly/Likely/High
finding is emitted (see postcondition 6 and BC-2.07.012 for tls-parser 0.12 reachability
constraints). The flow's `server_hello_seen` flag is set to true.

**tls-parser 0.12 reachability constraint (F-S054-P1-002):** A ServerHello record with
version `0x0200` (SSL 2.0) or lower is rejected at the tls-parser record layer before
`handle_server_hello` is invoked — `parse_errors` is incremented and no finding is produced.
Only `0x0300` (SSL 3.0) is a reachable deprecated-version trigger under tls-parser 0.12.
See EC-004 and BC-2.07.012 EC-004.

## Preconditions

1. `TlsAnalyzer::on_data` has been called with bytes for the server direction.
2. The accumulated server-direction buffer contains a complete TLS Handshake record
   (`record_type == 0x16`) with a complete ServerHello message.
3. `payload_len <= MAX_RECORD_PAYLOAD` (18,432 bytes).
4. The flow's `server_hello_seen` is currently false (first ServerHello only; once
   both hellos are seen the flow is done and subsequent data is ignored).

## Postconditions

1. `flow.server_hello_seen` is set to `true`.
2. The ServerHello `version` field (u16) is inserted/incremented in `version_counts`.
3. A JA3S MD5 hex string (32 lowercase hex chars) is computed and inserted/incremented
   in `ja3s_counts` (bounded by `MAX_MAP_ENTRIES`).
4. The cipher name (from `cipher_name(sh.cipher)`) is inserted/incremented in
   `cipher_counts` (bounded by `MAX_MAP_ENTRIES`).
5. If `is_weak_server_cipher(sh.cipher)` is true, one `Anomaly/Likely/Medium` finding
   is pushed to `all_findings` (see BC-2.07.010).
6. If `version <= 0x0300` AND the record was accepted by tls-parser (i.e., `handle_server_hello`
   was reached), one `Anomaly/Likely/High` finding is pushed to `all_findings` (see
   BC-2.07.012). Under tls-parser 0.12 this is only reachable for `version == 0x0300`
   (SSL 3.0); a ServerHello with version `0x0200` or lower is rejected at the record layer
   before this handler is invoked — no finding is produced (see EC-004).

## Invariants

1. JA3S is computed solely from `(version, selected_cipher, extension_ids)`; GREASE
   extension IDs are filtered (same `is_grease_u16` mask as JA3).
2. Unknown cipher IDs render as `0xNNNN` lowercase hex (see BC-2.07.036).
3. `version_counts` receives the ServerHello version independently of any prior
   ClientHello version count.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ServerHello with no extensions (sh.ext = None) | JA3S computed with empty extensions field |
| EC-002 | ServerHello with extensions that fail parse_tls_extensions | parse_errors++; JA3S computed with empty ext field |
| EC-003 | ServerHello cipher = TLS_NULL_WITH_NULL_NULL (0x0000) | is_weak_server_cipher returns true; Anomaly/Likely/Medium finding emitted |
| EC-004 | ServerHello version = 0x0200 (SSL 2.0) — PARSE-REJECTION under tls-parser 0.12 | tls-parser rejects the record at the record layer before `handle_server_hello` is reached; `parse_errors` is incremented; `version_counts[0x0200]` remains 0; `ja3s_counts` is not updated; NO deprecated-protocol finding is produced. Pinned by `test_BC_2_07_002_ec004_ssl2_version_parse_behavior_pinned` (asserts parse_errors==1, version_counts[0x0200]==0, no finding). This mirrors BC-2.07.012 EC-004. If tls-parser is upgraded to accept SSL 2.0 ServerHello records, this behavior transitions to a positive Anomaly/Likely/High finding — see BC-2.07.012 EC-004 upgrade guard. |
| EC-005 | ServerHello version = 0x0301 (TLS 1.0) | No deprecated-protocol finding; version counted only |
| EC-006 | ServerHello when `ja3s_counts` at MAX_MAP_ENTRIES with a new hash | New hash silently dropped |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ServerHello with TLS 1.2 (0x0303), strong cipher TLS_AES_128_GCM_SHA256 | server_hello_seen=true; ja3s_counts has entry; cipher_counts has entry; no findings | happy-path |
| ServerHello with TLS_RSA_EXPORT_WITH_RC4_40_MD5 cipher | One Anomaly/Likely/Medium finding; cipher in evidence | error |
| ServerHello with version 0x0300 (SSL 3.0) | One Anomaly/Likely/High finding with "SSL 3.0" in summary | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | JA3S is 32 lowercase hex chars | proptest: compute_ja3s_is_deterministic_and_hex |
| — | Weak server cipher produces Anomaly/Likely/Medium finding | unit: test_weak_cipher_finding_server |
| — | server_hello_seen set after processing | unit: test_parse_server_hello |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- ServerHello parsing and JA3S fingerprinting is a core TLS analysis capability |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:542-604, C-13) |
| Stories | STORY-053 |
| Origin BC | BC-TLS-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.001 -- related to (ClientHello counterpart)
- BC-2.07.003 -- composes with (both-hellos-done short-circuit)
- BC-2.07.008 -- composes with (JA3S string format)
- BC-2.07.010 -- composes with (weak server cipher detection)
- BC-2.07.012 -- composes with (deprecated server version detection)

## Architecture Anchors

- `src/analyzer/tls.rs:542-604` -- `handle_server_hello` implementation
- `src/analyzer/tls.rs:563` -- JA3S computation
- `src/analyzer/tls.rs:566-568` -- cipher tracking
- `tests/tls_analyzer_tests.rs` -- test_parse_server_hello, test_weak_cipher_finding_server

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:542-604` (`handle_server_hello`) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: TlsServerHelloContents struct from tls_parser
- **guard clause**: version <= 0x0300 deprecation check; is_weak_server_cipher guard

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates version_counts, ja3s_counts, cipher_counts, all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
