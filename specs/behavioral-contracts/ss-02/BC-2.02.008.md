---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/decoder.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-02
capability: CAP-02
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

# BC-2.02.008: Reject Unsupported Link Types in decode_packet

## Description

`decode_packet` has its own unsupported-link-type check separate from the reader-level gate
(BC-2.01.001). If called with any `DataLink` variant not in {ETHERNET, RAW, IPV4, IPV6,
LINUX_SLL}, it returns `Err(anyhow!("Unsupported link type: {other:?}"))` immediately.
This provides defense-in-depth if the decoder is called directly, bypassing the reader gate.

## Preconditions

1. `decode_packet` is called with `datalink` set to a variant outside the accepted set.
2. `data` may be any byte slice.

## Postconditions

1. Returns `Err(anyhow::Error)` immediately without attempting to parse `data`.
2. The error message contains "Unsupported link type:" and the debug representation of the variant.
3. No panic occurs.
4. `data` is not read.

## Invariants

1. The unsupported-type check fires before any byte access -- the `other =>` arm is the last
   match arm and returns immediately.
2. This check is duplicated in `lax_parse` for the same set of unsupported variants.
3. The error prefix "Unsupported link type:" is the distinguishing string; callers may use
   it to differentiate from "Parse error:".

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DataLink::IEEE802_11 (wireless) | Err("Unsupported link type: IEEE802_11") |
| EC-002 | Any numeric link type not in whitelist | Err with debug variant name |
| EC-003 | Called after reader rejects the same type | Defense-in-depth; same error produced |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| DataLink::IEEE802_11 with any bytes | Err containing "Unsupported link type:" | error |
| Arbitrary DataLink variant not in whitelist | Err (no panic) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All non-whitelisted DataLink variants produce Err | proptest: enumerate pcap-file DataLink variants |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 |
| Capability Anchor Justification | CAP-02 ("Link-type gating") per capabilities.md §CAP-02 -- the decode-layer link-type gate is the defense-in-depth complement to the reader-layer gate |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-02 (decoder.rs, C-5) |
| Stories | S-TBD |
| Origin BC | BC-DEC-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.01.001 -- composes with (reader-level link-type gate is the primary; this is secondary)
- BC-2.02.007 -- related to (both produce Err without panic; different trigger conditions)

## Architecture Anchors

- `src/decoder.rs:136` -- `other => return Err(anyhow!("Unsupported link type: {other:?}"))`
- `src/decoder.rs:204` -- same guard in lax_parse

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/decoder.rs:136` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: wildcard match arm returning Err

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed.
