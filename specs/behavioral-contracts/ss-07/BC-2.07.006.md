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

# BC-2.07.006: JA3 Computation Filters GREASE Values per RFC 8701

## Description

`compute_ja3` and `compute_ja3s` filter GREASE values from cipher suites, extension
type IDs, and named groups before constructing the JA3/JA3S string. The filter uses the
bitmask test `(val & 0x0F0F) == 0x0A0A`. This is deliberately broader than RFC 8701's
strict 16-value definition: the bitmask accepts 256 values of the form `0x_A_A` (where
_ is any hex nibble), including the 16 canonical GREASE values and 240 non-canonical
ones. In practice this is harmless because IANA has assigned no real cipher/extension ID
with that low-nibble pattern outside the 16 GREASE values.

## Preconditions

1. `compute_ja3` is called during `handle_client_hello` with a cipher list and
   extension list from a parsed ClientHello.
2. The cipher list or extension type list contains one or more GREASE values.

## Postconditions

1. All values `v` where `(v & 0x0F0F) == 0x0A0A` are excluded from the JA3 string's
   cipher, extension, curve, and point-format fields.
2. The resulting JA3 hash is identical to the hash that would be produced from a
   ClientHello that never included the GREASE values.
3. Non-GREASE values are preserved in their original order.

## Invariants

1. The GREASE filter is applied to: cipher IDs, extension type IDs, named group IDs.
   EC point format bytes are NOT filtered (they are single bytes, not u16 values).
2. The bitmask is applied to all three u16 fields using the same `is_grease_u16`
   function.
3. Inserting a canonical GREASE value into any position of the cipher list does not
   change the resulting JA3 hash.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cipher list is all GREASE (e.g. [0x0a0a, 0x1a1a]) | JA3 cipher field is empty string after filtering |
| EC-002 | Non-canonical GREASE-pattern value 0x0a1a | Filtered by bitmask (0x0a1a & 0x0F0F == 0x0a0a); treated as GREASE |
| EC-003 | All 16 canonical RFC 8701 GREASE values | All filtered; same JA3 as empty cipher list |
| EC-004 | GREASE in extensions but not ciphers | Only extension field affected; cipher field unchanged |
| EC-005 | No GREASE values present | Filtering is a no-op; all values preserved |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Cipher list [0x0a0a, 0x002f] (GREASE + TLS_RSA_WITH_AES_128_CBC_SHA) | JA3 same as cipher list [0x002f] only | happy-path |
| Cipher list [0x0a0a] only | JA3 cipher field = "" | edge-case |
| Cipher list [0x002f, 0x0035] with no GREASE | JA3 cipher field = "47-53" (decimal) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 16 canonical GREASE values are recognized | unit: is_grease_u16_matches_all_canonical_grease_values |
| VP-TBD | Inserting GREASE at any position does not change JA3 hash | proptest: compute_ja3_is_grease_invariant |
| VP-TBD | Bitmask matches exactly (val & 0x0F0F) == 0x0A0A | proptest: is_grease_u16_matches_nibble_bitmask_contract |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per capabilities.md §CAP-07 -- GREASE filtering is required for accurate JA3 fingerprinting, a core TLS analysis output |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation -- JA3 string uses numeric IDs, not display names) |
| Architecture Module | SS-07 (analyzer/tls.rs:50-52, 100-143, C-16) |
| Stories | S-TBD |
| Origin BC | BC-TLS-006 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.007 -- composes with (JA3 string format uses GREASE-filtered fields)
- BC-2.07.001 -- depends on (GREASE filtering is applied during ClientHello handling)

## Architecture Anchors

- `src/analyzer/tls.rs:50-52` -- `is_grease_u16` function
- `src/analyzer/tls.rs:100-112` -- cipher and extension GREASE filtering in compute_ja3
- `src/analyzer/tls.rs:155-170` -- extension GREASE filtering in compute_ja3s
- `tests/tls_analyzer_tests.rs` -- test_ja3_grease_filtering

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:50-52` (is_grease_u16), `src/analyzer/tls.rs:100-143` (compute_ja3) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `.filter(|c| !is_grease_u16(c.0))` in cipher iteration
- **assertion**: test_ja3_grease_filtering; proptest compute_ja3_is_grease_invariant

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (pure function) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (pure function) |
| **Overall classification** | pure |
