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

# BC-2.07.007: JA3 String Format: version,ciphers,...; MD5 Hex

## Description

`compute_ja3` constructs a JA3 string by joining five fields with commas in the order:
`version,cipher_list,extension_list,curve_list,point_format_list`. Each list uses
hyphens to join the decimal numeric values. GREASE values are excluded. The JA3
fingerprint is the MD5 hex digest of this string (lowercase, 32 chars). The function
returns `(md5_hex, ja3_string)` as a pair.

## Preconditions

1. `compute_ja3` is called with `(version: u16, ciphers: &[TlsCipherSuiteID], extensions: &[TlsExtension])`.
2. GREASE values have already been filtered (see BC-2.07.006).

## Postconditions

1. The JA3 string has exactly 4 commas (5 fields).
2. The first field is the decimal representation of `version` (e.g., `771` for
   TLS 1.2 / 0x0303).
3. The cipher field is the decimal IDs of non-GREASE ciphers joined by `-`.
   If all ciphers are GREASE or none exist, the cipher field is `""`.
4. The extension field is the decimal type IDs of non-GREASE extensions joined by `-`.
5. The curves field is the decimal group IDs of non-GREASE named groups joined by `-`.
6. The point-format field is the decimal bytes from EcPointFormats joined by `-`.
7. The MD5 hash is computed over the UTF-8 bytes of the JA3 string.
8. The returned hash is 32 lowercase hex characters.

## Invariants

1. Field order is fixed: version, ciphers, extensions, curves, point-formats.
   Changing any field changes the hash.
2. The hash is order-sensitive: `[A, B]` and `[B, A]` produce different hashes.
3. Decimal encoding of cipher/extension IDs, not hex or names.
4. The JA3 string is not stored or emitted; only the MD5 hash is surfaced to callers.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No extensions, no curves, no point formats | JA3 string = "771,,,,"; 4 trailing commas for empty fields |
| EC-002 | version = 0 | JA3 string starts with "0," |
| EC-003 | Single cipher 0x002f | Cipher field = "47" (decimal) |
| EC-004 | Two ciphers [0x002f, 0x0035] | Cipher field = "47-53" |
| EC-005 | Same ciphers in different order | Different hashes (order-sensitive) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| version=771, ciphers=[0x002f], extensions=[], curves=[], pf=[] | ja3_str="771,47,,,"; hash is MD5 of that string | happy-path |
| version=771, empty all | ja3_str="771,,,,"; hash is MD5 of "771,,,," | edge-case |
| ciphers in two orders [A,B] vs [B,A] | Two different hashes | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | JA3 string has exactly 4 commas | proptest: compute_ja3_has_five_fields_and_hex_hash |
| VP-TBD | Hash is 32 lowercase hex chars | proptest: compute_ja3_has_five_fields_and_hex_hash |
| VP-TBD | First field is the version | proptest: compute_ja3_has_five_fields_and_hex_hash checks starts_with version |
| VP-TBD | Order-sensitivity: [A,B] != [B,A] | proptest: compute_ja3_is_order_sensitive |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- JA3 string format is the algorithm defining the fingerprint output |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:94-151, C-13) |
| Stories | S-TBD |
| Origin BC | BC-TLS-007 (pass-3 ingestion corpus, MEDIUM confidence) |

## Related BCs

- BC-2.07.006 -- depends on (GREASE filtering applied before string construction)
- BC-2.07.008 -- related to (JA3S uses similar format for server-side)
- BC-2.07.001 -- depends on (JA3 computed during ClientHello handling)

## Architecture Anchors

- `src/analyzer/tls.rs:94-151` -- `compute_ja3` function body
- `src/analyzer/tls.rs:146-150` -- string assembly and MD5 computation

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:94-151` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **inferred**: format string `format!("{version},{cipher_str},{ext_ids},{curves_str},{pf_str}")` at tls.rs:148
- **assertion**: proptest tests verify 5-field format, hex output, version prefix

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (pure function) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (pure function) |
| **Overall classification** | pure |
