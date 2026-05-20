---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/mitre.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-10
capability: CAP-10
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

# BC-2.10.008: All Emitted Technique IDs Resolve in Lookup

## Description

Every technique ID that any analyzer or reassembly engine currently emits as
`Finding.mitre_technique = Some("TXXXX")` must resolve to `Some(...)` when passed to
`technique_name` or `technique_tactic`. The 6 currently-emitted IDs are: T1027, T1036,
T1046, T1083, T1499.002, T1505.003. No emitted ID may return None from the lookup --
that would cause the terminal reporter to display `<id> (unknown)` for a Finding produced
by the current analyzers.

## Preconditions

1. `technique_name` or `technique_tactic` is called with one of the 6 emitted IDs.

## Postconditions

1. All 6 currently-emitted IDs return `Some(...)`: T1027, T1036, T1046, T1083, T1499.002, T1505.003.
2. None of the 6 emitted IDs returns None.

## Invariants

1. The emitted set (6 IDs) is a strict subset of the catalogued set (15 IDs).
2. The invariant is enforced by convention: when an analyzer adds a new emission site, the
   developer must add the ID to `technique_info` first (or simultaneously).
3. The authoritative list of emitted IDs is `grep -rn 'mitre_technique: Some' src/` per mitre.rs comment.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | T1027 (TLS SNI control-byte) | Some("Obfuscated Files or Information") |
| EC-002 | T1036 (TCP conflicting overlap) | Some("Masquerading") |
| EC-003 | T1046 (HTTP admin panel) | Some("Network Service Discovery") |
| EC-004 | T1083 (HTTP path traversal) | Some("File and Directory Discovery") |
| EC-005 | T1499.002 (HTTP too-many-headers) | Some("Service Exhaustion Flood") |
| EC-006 | T1505.003 (HTTP web shell) | Some("Web Shell") |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| technique_name("T1027") | Some("Obfuscated Files or Information") | happy-path |
| technique_name("T1505.003") | Some("Web Shell") | happy-path |
| technique_name("T1499.002") | Some("Service Exhaustion Flood") | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 6 emitted IDs resolve in technique_name | unit: test each emitted ID |
| VP-TBD | No new emission site uses an ID not in technique_info | manual: code review of analyzer PRs |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 |
| Capability Anchor Justification | CAP-10 ("MITRE ATT&CK mapping") per capabilities.md §CAP-10 -- emitted-ID resolution is the end-to-end correctness property of the MITRE mapping capability |
| L2 Domain Invariants | INV-9 (MITRE technique ID format) |
| Architecture Module | SS-10 (mitre.rs, C-16) |
| Stories | S-TBD |
| Origin BC | BC-MIT-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.10.005 -- composes with (catalog lookup is the mechanism)
- BC-2.10.007 -- composes with (tactic resolution also applies to emitted IDs)

## Architecture Anchors

- `src/mitre.rs:123-154` -- technique_info match table covering all 6 emitted IDs
- Emitted sites: `src/analyzer/tls.rs:443,463,483` (T1027 x3), `src/analyzer/http.rs:198,228,244,423,482` (T1083, T1505.003, T1046, T1499.002 x2), `src/reassembly/mod.rs:442` (T1036)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/mitre.rs:123-154` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: technique_name_resolves_every_seeded_id covers the emitted subset
- **documentation**: mitre.rs comment "grep -rn 'mitre_technique: Some' src/"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed. This invariant should ideally be enforced by a CI check (e.g., a
test that greps for mitre_technique: Some patterns and asserts each ID resolves). That
test does not currently exist; it is a test-gap from the domain-debt O-04 area.
