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

# BC-2.06.025: uris List Capped at MAX_URIS=10000

## Description

The `uris: Vec<String>` list is bounded by `MAX_URIS = 10,000`. Before appending a parsed
URI, the analyzer checks `self.uris.len() < MAX_URIS`. If the list is already at capacity,
the URI is silently dropped. The list is NOT a deduplicating set; the same URI can appear
multiple times. The cap prevents unbounded memory growth from high-volume HTTP traffic.

## Preconditions

1. `self.uris.len() == MAX_URIS (10,000)`.
2. A new request is parsed with a URI to append.

## Postconditions

1. The URI is NOT appended to `self.uris` (`len()` stays at 10,000).
2. No error or counter increment occurs for the dropped URI.
3. Other counters (methods, hosts, etc.) are still updated for the request.

## Invariants

1. `MAX_URIS = 10,000` (http.rs:23).
2. Guard: `if self.uris.len() < MAX_URIS { self.uris.push(...) }` (http.rs:391-393).
3. The cap applies to the Vec length; URIs dropped at cap are permanently lost.
4. The `summarize()` method returns only the FIRST 20 URIs from this list (BC-2.06.023).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | uris at 9999; new request | URI appended (len=10000) |
| EC-002 | uris at 10000; new request | URI NOT appended; len stays 10000 |
| EC-003 | uris at cap; same URI repeated | URI still dropped (no contains() check for uris) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 10001 distinct requests | uris.len() == 10000; last URI not present | edge-case |
| 10000 requests; then valid GET | uris unchanged at 10000; method still counted | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | uris capped at MAX_URIS | MEDIUM -- no direct test; inferred from guard code |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- uris Vec cap is a memory-bounding mechanism for HTTP analysis statistics |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:391-393, C-14) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-025 (pass-3 ingestion corpus, MEDIUM confidence -- no direct test) |

## Related BCs

- BC-2.06.024 -- composes with (analogous cap on HashMap entries)

## Architecture Anchors

- `src/analyzer/http.rs:391-393` -- uris push guard

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:391-393` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.uris.len() < MAX_URIS { self.uris.push(parsed.uri.clone()); }`
- **inferred**: no direct test; code confirmed in source

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | conditionally mutates uris Vec |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
