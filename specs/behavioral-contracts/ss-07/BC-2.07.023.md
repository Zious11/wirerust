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

# BC-2.07.023: Empty SNI Hostname Bytes Counted Under "" Key; No Finding

## Description

When a ClientHello's SNI ServerNameList contains one entry with zero-length hostname
bytes (the entry exists, but `hostname == b""`), `extract_sni` classifies the empty
byte slice. `str::from_utf8(b"")` succeeds with `Ok("")`; `"".is_ascii()` is true;
`contains_c0_or_del("")` is false (no bytes to check). Therefore arm 1 fires:
`SniValue::Ascii("")`. The empty string is counted in `sni_counts` under the key `""`.
No finding is emitted.

## Preconditions

1. A ClientHello has an SNI extension with one entry where `hostname.len() == 0`.
2. `str::from_utf8(b"")` returns `Ok("")`.

## Postconditions

1. `extract_sni` returns `Some(SniValue::Ascii(""))`.
2. `sni_counts[""]` is incremented.
3. No finding is pushed.
4. This is a degenerate RFC violation (RFC 6066 requires non-empty hostname) but
   not flagged as anomalous by the current implementation.

## Invariants

1. Empty string satisfies arm 1 conditions vacuously (no bytes to fail any check).
2. The sni_counts key for empty SNI is `""` (empty string), distinct from the
   `<non-utf8:...>` key format used for arm 4.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First entry is empty, second entry is non-empty | Only first entry processed (BC-2.07.024); empty counted; no finding |
| EC-002 | Empty bytes when sni_counts at MAX_MAP_ENTRIES and "" not yet seen | Key silently dropped |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello with SNI entry where hostname = b"" | sni_counts[""] = 1; no finding | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Empty hostname bytes counted under "" key with no finding | unit: test_sni_with_empty_hostname_bytes |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- degenerate empty SNI is an SNI edge case in TLS analysis |
| L2 Domain Invariants | INV-5 (SNI 4-way classification -- arm 1 applies vacuously to empty string) |
| Architecture Module | SS-07 (analyzer/tls.rs:251-252, C-13) |
| Stories | STORY-057 |
| Origin BC | BC-TLS-023 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.022 -- related to (empty list vs. empty hostname bytes)
- BC-2.07.013 -- composes with (arm 1 handles this)

## Architecture Anchors

- `src/analyzer/tls.rs:251-252` -- arm 1 match clause handles empty string vacuously
- `tests/tls_analyzer_tests.rs` -- test_sni_with_empty_hostname_bytes

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:251-252` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_sni_with_empty_hostname_bytes

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
