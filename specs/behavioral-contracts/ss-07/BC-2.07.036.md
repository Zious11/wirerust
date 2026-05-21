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

# BC-2.07.036: Unknown Cipher IDs Render as Hex 0xNNNN Lowercase

## Description

`cipher_name(id)` is called to produce a human-readable cipher name for evidence
strings and cipher_counts map keys. For cipher IDs recognized by the `tls_parser`
crate (`TlsCipherSuite::from_id(id.0)` returns `Some`), the name is the canonical
IANA string (e.g., "TLS_AES_256_GCM_SHA384"). For unrecognized IDs (returns `None`),
the fallback format is `"0x{id:04x}"` -- lowercase hex with leading zero-padding to 4
hex digits.

## Preconditions

1. `cipher_name(id: TlsCipherSuiteID)` is called with a cipher ID.
2. `TlsCipherSuite::from_id(id.0)` returns `None` (unrecognized ID).

## Postconditions

1. Returns `format!("0x{:04x}", id.0)` -- a 6-character lowercase hex string for
   values fitting in 4 hex digits (e.g., `"0x1234"`).
2. The returned string is used as the `cipher_counts` map key and in finding evidence.

## Invariants

1. The format is `0x` prefix followed by exactly 4 lowercase hex digits (zero-padded).
2. For known cipher IDs, the name string has no `0x` prefix and uses IANA naming
   (e.g., "TLS_NULL_WITH_NULL_NULL", not "0x0000").
3. All u16 values (0x0000-0xFFFF) are covered: 0x0000 -> "0x0000", 0xFFFF -> "0xffff".

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ID = 0x0000 (TLS_NULL, recognized) | Returns "TLS_NULL_WITH_NULL_NULL" (from_id returns Some) |
| EC-002 | ID = 0x1234 (unrecognized) | Returns "0x1234" |
| EC-003 | ID = 0x000A (known, e.g., TLS_RSA_WITH_3DES_EDE_CBC_SHA) | Returns IANA name |
| EC-004 | ID = 0xFFFF (unrecognized) | Returns "0xffff" (lowercase) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| id = 0x1234 (unrecognized) | "0x1234" | happy-path |
| id = 0x0035 (TLS_RSA_WITH_AES_256_CBC_SHA, recognized) | "TLS_RSA_WITH_AES_256_CBC_SHA" | happy-path |
| id = 0xAAAA (unrecognized) | "0xaaaa" (lowercase) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Unknown cipher IDs render as 0xNNNN lowercase hex | unit (no dedicated test; format verified from code) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- cipher name rendering is used in TLS analysis evidence strings and summary output |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:77-83, C-13) |
| Stories | STORY-054 |
| Origin BC | BC-TLS-036 (pass-3 ingestion corpus, MEDIUM confidence -- no direct test) |

## Related BCs

- BC-2.07.010 -- composes with (server weak-cipher evidence uses cipher_name)
- BC-2.07.031 -- composes with (cipher_counts keys use cipher_name output)

## Architecture Anchors

- `src/analyzer/tls.rs:77-83` -- `cipher_name` function with hex fallback

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:77-83` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **inferred**: `format!("0x{:04x}", id.0)` at tls.rs:82; no direct test

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (pure function) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (pure function) |
| **Overall classification** | pure |
