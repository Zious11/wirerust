---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.3 (2026-05-28): W15 Pass-1 remediation — anchor line ranges verified (F-W15S051-P1-006); STORY-051 BC-prefixed companion tests added to Architecture Anchors test list (covers test rename + 2 new tests from Round 1 commit 920891e)."
  - "v1.4: PATCH — Pass-19 B-10 anchor fix: format string tls.rs:171→:172; Md5::digest tls.rs:172→:173 (off-by-one). No functional postcondition change. — 2026-06-13"
  - "v1.5: PG-ARP-F2-007 ss-07 full re-anchor — compute_ja3s doc+fn 153-173→154-174; ext GREASE filter 157-169→158-170 — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.008: JA3S String Format: version,cipher,extensions; MD5 Hex

## Description

`compute_ja3s` constructs the JA3S server fingerprint string from three comma-separated
fields: `version,selected_cipher_id,extension_type_ids`. GREASE extension IDs are
filtered using the same `is_grease_u16` mask. The MD5 hex digest of this string is the
JA3S fingerprint. Unlike JA3, JA3S has only 3 fields (no curves or point formats).
The function returns only the MD5 hex string, not the underlying JA3S string.

## Preconditions

1. `compute_ja3s` is called from `handle_server_hello` with `(version: u16, cipher: TlsCipherSuiteID, extensions: &[TlsExtension])`.

## Postconditions

1. The JA3S string has exactly 2 commas (3 fields).
2. Field 1: decimal `version`.
3. Field 2: decimal `cipher.0` (the selected cipher suite numeric ID).
4. Field 3: GREASE-filtered extension type IDs joined by `-`. Empty string if none.
5. The MD5 hex digest is 32 lowercase hex characters.
6. The hash is deterministic: same inputs always produce the same 32-char string.

## Invariants

1. The cipher field is a SINGLE value (server selects ONE cipher), not a list.
2. GREASE filtering applies only to extension IDs, not to the cipher field.
3. The JA3S string itself is not stored or returned; only the MD5 hash is returned.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No extensions | JA3S string = "version,cipher,"; trailing comma for empty ext field |
| EC-002 | All extensions are GREASE | JA3S extension field = "" |
| EC-003 | version = 0x0303 (TLS 1.2), cipher = 0x002f | JA3S = "771,47," |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| version=771, cipher=0x002f, extensions=[] | Hash is MD5 of "771,47,"; 32 lowercase hex chars | happy-path |
| Same inputs called twice | Identical hash (deterministic) | happy-path |
| version=0, cipher=0, extensions=[] | Hash is MD5 of "0,0,"; not empty | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-013 | JA3S hash is deterministic (same inputs, same output) | proptest: compute_ja3s_is_deterministic_and_hex |
| VP-013 | JA3S hash is 32 lowercase hex chars | proptest: compute_ja3s_is_deterministic_and_hex |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- JA3S is the server-side TLS fingerprint output of TLS analysis |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:154-174, C-13) |
| Stories | STORY-051 |
| Origin BC | BC-TLS-008 (pass-3 ingestion corpus, MEDIUM confidence) |

## Related BCs

- BC-2.07.007 -- related to (JA3 is the client-side counterpart)
- BC-2.07.002 -- depends on (JA3S computed during ServerHello handling)
- BC-2.07.006 -- composes with (GREASE filtering applies to extensions)

## Architecture Anchors

- `src/analyzer/tls.rs:154-174` -- `compute_ja3s` function body (doc-block + fn)
- `src/analyzer/tls.rs:172` -- format string `format!("{},{},{}", version, cipher.0, ext_ids)` and `Md5::digest` at 173
- `src/analyzer/tls.rs:158-170` -- extension GREASE filtering in compute_ja3s
- `tests/tls_analyzer_tests.rs` -- test_BC_2_07_008_ja3s_has_exactly_two_commas_three_fields
- `tests/tls_analyzer_tests.rs` -- test_BC_2_07_008_ja3s_grease_extension_filtered_from_ext_field
- `tests/tls_analyzer_tests.rs` -- test_BC_2_07_008_ja3s_hash_is_32_lowercase_hex_and_deterministic
- `tests/tls_analyzer_tests.rs` -- test_BC_2_07_008_ja3s_cipher_field_is_single_value_not_filtered
- `tests/tls_analyzer_tests.rs` -- test_BC_2_07_008_ja3s_grease_extension_filtered_but_grease_cipher_preserved
- `tests/tls_analyzer_tests.rs` -- test_BC_2_07_008_ja3s_all_grease_extensions_produce_empty_ext_field

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:154-174` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **inferred**: format string at tls.rs:172; proptest covers determinism and length

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (pure function) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (pure function) |
| **Overall classification** | pure |
