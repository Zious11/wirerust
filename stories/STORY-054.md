---
document_type: story
story_id: "STORY-054"
epic_id: "E-5"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.009.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.010.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.011.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.012.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.030.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.036.md
input-hash: "89cab55"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-052, STORY-053]
blocks: []
behavioral_contracts:
  - BC-2.07.009
  - BC-2.07.010
  - BC-2.07.011
  - BC-2.07.012
  - BC-2.07.030
  - BC-2.07.036
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 18
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-054`

# STORY-054: Cipher and Protocol Weakness Findings — Weak Ciphers, Deprecated SSL Versions, and Baseline Zero-Finding

## Narrative
- **As a** malware researcher
- **I want** the TLS analyzer to emit structured `Anomaly/Likely/High` and `Anomaly/Likely/Medium` findings when a ClientHello or ServerHello offers or selects a weak cipher (NULL/ANON/EXPORT/RC4) or uses a deprecated SSL version (≤ 0x0300)
- **So that** I can identify TLS misconfigurations and evasion-via-weak-crypto patterns while trusting that normal modern TLS handshakes produce zero findings (no false positives)

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.009 | Weak Client Cipher in ClientHello Emits Anomaly/Likely/High Finding |
| BC-2.07.010 | Weak Server Cipher Selected Emits Anomaly/Likely/Medium Finding |
| BC-2.07.011 | Deprecated Client Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding |
| BC-2.07.012 | Deprecated Server Protocol (<=SSLv3) Emits Anomaly/Likely/High Finding |
| BC-2.07.030 | Normal Handshake with Strong Cipher Produces Zero Findings |
| BC-2.07.036 | Unknown Cipher IDs Render as Hex 0xNNNN Lowercase |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.009 postcondition 1-2)
When `handle_client_hello` detects at least one cipher in `ch.ciphers` satisfying `is_weak_cipher` (cipher name contains "NULL", "ANON", or "EXPORT"), exactly ONE `Finding` is pushed to `all_findings` with: `category = Anomaly`, `verdict = Likely`, `confidence = High`, `summary = "ClientHello offers weak cipher suites (NULL/anonymous/export)"`, `evidence = Vec<String>` (one entry per weak cipher name), `mitre_technique = None`, `direction = Some(Direction::ClientToServer)`.
- **Test:** `test_weak_cipher_finding_client`

### AC-002 (traces to BC-2.07.009 postcondition 2)
Exactly ONE finding per ClientHello is emitted regardless of how many weak ciphers are in the list. Multiple weak ciphers produce one finding, not multiple.
- **Test:** `test_weak_cipher_finding_client` (send multiple weak ciphers)

### AC-003 (traces to BC-2.07.009 invariant 1-2)
GREASE-valued cipher IDs never trigger a weak-cipher finding: `TlsCipherSuite::from_id(id.0)` returns `None` for GREASE values, and `is_weak_cipher` returns `false` for `None`. Unknown cipher IDs also do not trigger the finding.
- **Test:** `test_normal_handshake_no_findings` (send GREASE cipher; assert no finding)

### AC-004 (traces to BC-2.07.010 postcondition 1-2)
When `handle_server_hello` detects `is_weak_server_cipher(sh.cipher)` (cipher name contains "NULL", "ANON", "EXPORT", or "RC4"), exactly ONE `Finding` is pushed with: `category = Anomaly`, `verdict = Likely`, `confidence = Medium`, `summary = "ServerHello selected weak cipher suite ({name})"`, `evidence = ["Selected cipher: {name} (0x{id:04x})"]`, `mitre_technique = None`, `direction = Some(Direction::ServerToClient)`.
- **Test:** `test_weak_cipher_finding_server`

### AC-005 (traces to BC-2.07.010 invariant 1)
`is_weak_server_cipher` is a strict superset of `is_weak_cipher` — it additionally flags RC4 ciphers (cipher name contains "RC4"). TLS_RSA_WITH_RC4_128_MD5 triggers the server-side finding but would NOT trigger the client-side `is_weak_cipher` check (which does not include RC4).
- **Test:** `test_weak_cipher_finding_server` (use RC4 cipher; assert Medium confidence)

### AC-006 (traces to BC-2.07.011 postcondition 1-2)
When `handle_client_hello` detects `ch.version.0 <= 0x0300`, exactly ONE `Finding` is pushed with: `category = Anomaly`, `verdict = Likely`, `confidence = High`, `summary = "ClientHello uses deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)"`, `evidence = ["Version: 0x{version:04x} ({version_name})"]`, `mitre_technique = None`, `direction = Some(Direction::ClientToServer)`. Version name mapping: 0x0200 -> "SSL 2.0", 0x0300 -> "SSL 3.0", else -> "Unknown legacy SSL".
- **Test:** `test_ssl30_pcap_generates_findings` (integration)

### AC-007 (traces to BC-2.07.011 invariant 1-2)
TLS 1.0 (0x0301) does NOT trigger the deprecated-protocol finding (threshold is strictly `<= 0x0300`). The summary always contains the string "RFC 7568" as a normative reference.
- **Test:** Unit test with version 0x0301; assert no deprecated-protocol finding

### AC-008 (traces to BC-2.07.011 invariant 3)
Both the deprecated-protocol finding AND the weak-cipher finding can fire from the same ClientHello if it offers SSL 3.0 AND a weak cipher. Both findings appear independently in `all_findings`.
- **Test:** Unit test with SSL 3.0 ClientHello containing a weak cipher; assert `all_findings.len() >= 2`

### AC-009 (traces to BC-2.07.012 postcondition 1-2)
When `handle_server_hello` detects `sh.version.0 <= 0x0300`, exactly ONE `Finding` is pushed with: `category = Anomaly`, `verdict = Likely`, `confidence = High`, `summary = "ServerHello negotiated deprecated protocol ({version_name}, RFC 7568 prohibits SSLv3)"`, `evidence = ["Version: 0x{version:04x} ({version_name})"]`, `mitre_technique = None`, `direction = Some(Direction::ServerToClient)`.
- **Test:** Unit test for `handle_server_hello` with version 0x0300

### AC-010 (traces to BC-2.07.012 invariant 1-2)
TLS 1.0 (0x0301) does NOT trigger the server-side deprecated-protocol finding. When both ClientHello AND ServerHello have SSL 3.0, two separate findings are emitted: one with `ClientToServer` direction and one with `ServerToClient` direction.
- **Test:** Unit test with SSL 3.0 ClientHello + SSL 3.0 ServerHello; assert two deprecated-protocol findings with distinct directions

### AC-011 (traces to BC-2.07.030 postcondition 1-4)
A TLS handshake with clean ASCII SNI, version > 0x0300, and no weak ciphers on either side produces zero findings. After both hellos: `all_findings.len() == 0`, `handshakes_seen == 1`, all count maps have exactly one entry each, `parse_errors == 0`.
- **Test:** `test_normal_handshake_no_findings`

### AC-012 (traces to BC-2.07.036 postcondition 1-2)
`cipher_name(id)` returns `format!("0x{:04x}", id.0)` for unrecognized cipher IDs (where `TlsCipherSuite::from_id(id.0)` returns `None`). The output is a 6-character lowercase string with `"0x"` prefix and 4 hex digits (e.g., `"0x1234"`, `"0xffff"`).
- **Test:** Unit test for `cipher_name` with an unrecognized ID (e.g., 0x1234)

### AC-013 (traces to BC-2.07.036 invariant 1-2)
For recognized cipher IDs, `cipher_name` returns the IANA canonical name string (e.g., `"TLS_AES_256_GCM_SHA384"`) without a `"0x"` prefix. For ID 0xFFFF (unrecognized), `cipher_name` returns `"0xffff"` (lowercase).
- **Test:** Unit test for `cipher_name` with a recognized ID and with 0xFFFF

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `is_weak_cipher` | src/analyzer/tls.rs:56-64 | pure-core |
| `is_weak_server_cipher` | src/analyzer/tls.rs:66-75 | pure-core |
| `cipher_name` | src/analyzer/tls.rs:77-83 | pure-core |
| Weak-cipher scan in `handle_client_hello` | src/analyzer/tls.rs:497-517 | effectful-shell |
| Server cipher check in `handle_server_hello` | src/analyzer/tls.rs:570-582 | effectful-shell |
| Deprecated version check (client) | src/analyzer/tls.rs:519-539 | effectful-shell |
| Deprecated version check (server) | src/analyzer/tls.rs:584-604 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Client cipher list has only strong ciphers | No weak-cipher finding |
| EC-002 | Client cipher list has NULL + ANON + strong cipher | One finding; evidence has 2 weak cipher names |
| EC-003 | Unknown cipher ID 0xFFFF in client cipher list | `from_id` returns None; `is_weak_cipher` returns false; no finding |
| EC-004 | Server selects TLS_RSA_WITH_RC4_128_MD5 | Anomaly/Likely/Medium finding (RC4 triggers `is_weak_server_cipher`) |
| EC-005 | Server selects 0xFFFF (unknown cipher) | `is_weak_server_cipher` returns false; no finding; cipher_counts key = "0xffff" |
| EC-006 | ClientHello version = 0x0200 (SSL 2.0) | Finding summary contains "SSL 2.0" |
| EC-007 | ClientHello version = 0x0100 (below SSL 2.0) | Finding summary contains "Unknown legacy SSL" |
| EC-008 | ServerHello version = 0x0301 (TLS 1.0) | No deprecated-protocol finding (threshold strictly <= 0x0300) |
| EC-009 | cipher_name(0xAAAA) | Returns "0xaaaa" (lowercase, not "0xAAAA") |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (is_weak_cipher, is_weak_server_cipher, cipher_name) | pure-core | No I/O, no global state; pure predicate/formatting functions |
| src/analyzer/tls.rs (weak-cipher scan, deprecated-version check in handle_*) | effectful-shell | Mutates all_findings via push |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~4,000 |
| Referenced code (tls.rs lines 56-83, 497-604) | ~4,500 |
| Test files (tls_analyzer_tests.rs cipher/version finding tests) | ~4,000 |
| BC files (6 BCs) | ~7,000 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~21,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~11%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-013 (test-writer)
2. [ ] Verify Red Gate: all AC tests fail before implementation
3. [ ] Implement `is_weak_cipher`: returns true if `TlsCipherSuite::from_id(id.0).map_or(false, |c| c.name().to_uppercase().contains("NULL") || ... "ANON" || ... "EXPORT")`
4. [ ] Implement `is_weak_server_cipher`: extends `is_weak_cipher` to also check for "RC4" in cipher name
5. [ ] Implement `cipher_name`: returns IANA name if `from_id` returns Some, else `format!("0x{:04x}", id.0)`
6. [ ] Implement weak-cipher scan in `handle_client_hello`: collect all weak cipher names from `ch.ciphers`; if non-empty, push one Anomaly/Likely/High finding
7. [ ] Implement deprecated-version check in `handle_client_hello`: if `ch.version.0 <= 0x0300`, push Anomaly/Likely/High finding with version-name mapping
8. [ ] Implement weak-cipher check in `handle_server_hello`: if `is_weak_server_cipher(sh.cipher)`, push Anomaly/Likely/Medium finding
9. [ ] Implement deprecated-version check in `handle_server_hello`: if `sh.version.0 <= 0x0300`, push Anomaly/Likely/High finding with ServerToClient direction
10. [ ] Write `test_normal_handshake_no_findings` asserting zero findings for modern TLS
11. [ ] Run all tests; verify all pass
12. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-051 | `is_grease_u16` filters GREASE values by bitmask; GREASE IDs have `from_id` return None | GREASE immunity in weak-cipher check comes from `None`-returns-false branch, NOT an explicit GREASE pre-filter | The raw `ch.ciphers` list is scanned for weak ciphers WITHOUT filtering GREASE first — GREASE immunity is a side-effect of `from_id` returning None |
| STORY-052 | `handle_client_hello` orchestrates all per-ClientHello analysis including cipher scan | Evidence vec in weak-cipher finding has O(weak_ciphers_count) cardinality — no per-cipher cap | Weak cipher evidence stores cipher NAMES (readable strings), not hex IDs |
| STORY-053 | `handle_server_hello` orchestrates all per-ServerHello analysis including weak-cipher and version checks | Direction field distinguishes client-side (ClientToServer) from server-side (ServerToClient) findings | `confidence = Medium` for server weak-cipher (not High) — server made the final selection |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Weak-client-cipher finding: `confidence = High`; weak-server-cipher finding: `confidence = Medium` | BC-2.07.009 postcondition 1 vs BC-2.07.010 postcondition 1 | Unit tests: AC-001 (High), AC-004 (Medium) |
| `direction = ClientToServer` for client findings; `direction = ServerToClient` for server findings | BC-2.07.009/011 postcondition vs BC-2.07.010/012 postcondition | Unit tests assert direction field for each finding type |
| `mitre_technique = None` for all cipher/protocol weakness findings | BC-2.07.009/010/011/012 postcondition 1 | Code review: confirm no MITRE ID set in these finding push sites |
| Deprecated-protocol threshold is strictly `<= 0x0300`; TLS 1.0 (0x0301) must NOT trigger | BC-2.07.011 invariant 1 | Unit test: AC-007 with version 0x0301 |
| cipher_name hex format is `"0x{:04x}"` (lowercase, 0-padded to 4 digits) | BC-2.07.036 postcondition 1 | Unit test: AC-012, AC-013 |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `TlsCipherSuiteID`, `TlsCipherSuite::from_id` for cipher name lookup |
| Rust std | 2024 edition (stable) | `str::contains`, `format!`, `Vec::push` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | modify | `is_weak_cipher` (56-64), `is_weak_server_cipher` (66-75), `cipher_name` (77-83), weak-cipher scan in `handle_client_hello` (497-517), deprecated-version checks (519-539, 584-604), server weak-cipher (570-582) |
| tests/tls_analyzer_tests.rs | modify | `test_weak_cipher_finding_client`, `test_weak_cipher_finding_server`, `test_normal_handshake_no_findings`, version boundary tests |
| tests/tls_integration_tests.rs | modify | `test_ssl30_pcap_generates_findings` (AC-006) |
