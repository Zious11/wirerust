---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/http.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-06
capability: CAP-06
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

# BC-2.06.024: Per-Map Cardinality Cap: New Keys Dropped Past MAX_MAP_ENTRIES

## Description

The `methods`, `hosts`, and `user_agents` HashMaps are individually capped at
`MAX_MAP_ENTRIES = 50,000` keys. When a new key would be inserted but the map has already
reached the cap, the insert is silently skipped. Existing keys always increment regardless of
the cap. The `status_codes` map uses u16 keys (max 65535 distinct values) and has a practical
cap from the key type, but no explicit MAX_MAP_ENTRIES guard. The `uris` Vec is separately
bounded by MAX_URIS (BC-2.06.025).

## Preconditions

1. One of the guarded maps has reached exactly `MAX_MAP_ENTRIES = 50,000` distinct keys.
2. A new request or response is parsed with a novel key for that map.

## Postconditions

1. The new key is NOT added to the map (the `map.len() < MAX_MAP_ENTRIES || map.contains_key(key)` guard fails for new keys when at cap).
2. The counter for that new value is silently dropped.
3. Existing keys continue to increment normally.
4. No error is emitted; no counter is incremented for the drop.

## Invariants

1. `MAX_MAP_ENTRIES = 50,000` (http.rs:24).
2. Guard pattern: `if self.methods.len() < MAX_MAP_ENTRIES || self.methods.contains_key(&parsed.method)` (http.rs:375-378).
3. The guard allows EXISTING keys to increment even at cap (the `contains_key` short-circuit).
4. The cap applies independently per map; a host map at cap does not prevent new method keys.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Map at 50000 keys; new unique method arrives | Method NOT inserted; no panic |
| EC-002 | Map at 50000 keys; existing method arrives | Existing method count incremented normally |
| EC-003 | Map at 49999 keys; new method | New method inserted (cap not yet hit) |
| EC-004 | status_codes at practical u16 limit | No guard; type-level cap applies |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 50001 unique methods | methods.len() == 50000; last unique not present | edge-case |
| 50000 unique methods + 1 repeat of first | first method count incremented; len still 50000 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | New keys dropped past MAX_MAP_ENTRIES | MEDIUM -- no direct test in corpus; inferred from guard code |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- map cardinality cap is a memory-bounding mechanism for HTTP analysis statistics |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:375-389, C-12) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-024 (pass-3 ingestion corpus, MEDIUM confidence -- no direct test) |

## Related BCs

- BC-2.06.025 -- composes with (uris Vec has a separate cap; both are bounded-resource mechanisms)

## Architecture Anchors

- `src/analyzer/http.rs:375-389` -- map entry guards with MAX_MAP_ENTRIES check

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:375-389` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.methods.len() < MAX_MAP_ENTRIES || self.methods.contains_key(...)` (inferred)
- **inferred**: no direct test; code pattern confirmed in source

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | conditionally mutates methods, hosts, user_agents maps |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
