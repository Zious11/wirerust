---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-18
capability: CAP-18
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified:
  - "v1.1: F-F2P2-002 Pass-2 remediation — second VP-041 harness proptest_vp041_partition_invariant added; non-vacuity clarification. 2026-07-01"
  - "v1.2: F-F2P7-004 Pass-7 remediation — partition harness non-vacuity mislabeling corrected: proptest_vp041_partition_invariant holds trivially by the complement derivation (unsupported = KNOWN \\ supported); proptest_vp041_oracle_cross_check is the non-vacuous guard. Invariant 4, VP table partition row, Architecture Anchors, VP Anchors updated. 2026-07-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.18.004: Catalog Partition Invariant — Supported ∪ Unsupported == KNOWN_PROTOCOLS and Disjoint

## Description

The `KNOWN_PROTOCOLS` static array is partitioned into exactly two disjoint sets:
`supported_protocols()` and `unsupported_protocols()`. Their union equals `KNOWN_PROTOCOLS`
in its entirety, and their intersection is empty. No entry is in both sets; no entry is
absent from both sets. This partition invariant is the foundational correctness property
of the coverage catalog and is guarded by VP-041 (proptest). It is the invariant that
makes the `--supported` / `--unsupported` CLI filter flags semantically meaningful and
their outputs jointly exhaustive.

## Related BCs

- BC-2.18.003 — composes with (defines the two functions whose correctness this BC formalizes as an invariant)
- BC-2.18.001 — depends on (terminal output relies on this invariant to guarantee complete coverage of KNOWN_PROTOCOLS under any filter combination)
- BC-2.18.002 — depends on (JSON output relies on the same invariant)

## Preconditions

1. `KNOWN_PROTOCOLS` is a non-empty static array (compile-time constant).
2. `supported_protocols()` and `unsupported_protocols()` are called on the same binary build (same compile-time state).

## Postconditions

1. **Union completeness:** `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS` — every entry in `KNOWN_PROTOCOLS` appears in exactly one of the two result sets.
2. **Disjoint:** `supported_protocols() ∩ unsupported_protocols() == ∅` — no entry appears in both sets.
3. **Counting invariant:** `supported_protocols().len() + unsupported_protocols().len() == KNOWN_PROTOCOLS.len()`.
4. **Entry completeness:** For any entry `p` in `KNOWN_PROTOCOLS`, `p` appears in `supported_protocols()` if and only if it does NOT appear in `unsupported_protocols()`.
5. **No phantom entries:** Neither `supported_protocols()` nor `unsupported_protocols()` contains any entry that is not in `KNOWN_PROTOCOLS`.

## Invariants

1. The partition is STATIC — it depends only on compile-time constants (`KNOWN_PROTOCOLS`, `SUPPORTED_PORTS`). It cannot change at runtime.
2. Adding a new entry to `KNOWN_PROTOCOLS` without updating `SUPPORTED_PORTS` will cause the new entry to appear in `unsupported_protocols()`. This is the intended drift-detection behavior.
3. Adding a new port to `SUPPORTED_PORTS` without a corresponding `KNOWN_PROTOCOLS` entry with that port does NOT change the partition (the new port has no matching entry to move).
4. VP-041 uses TWO harnesses: `proptest_vp041_oracle_cross_check` (per-entry oracle cross-check — guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency; oracle computed INDEPENDENTLY without calling `supported_protocols()` or `unsupported_protocols()` — non-vacuous) and `proptest_vp041_partition_invariant` (partition/disjointness — verifies union completeness and empty intersection of the two function outputs; holds trivially by the complement derivation (`unsupported = KNOWN \ supported`); `proptest_vp041_oracle_cross_check` provides the non-vacuous guard). Both harnesses MUST pass.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Catalog at minimum size (1 entry) | Union = {entry}, disjoint trivially satisfied; partition holds |
| EC-002 | New ICS protocol added to KNOWN_PROTOCOLS without corresponding SUPPORTED_PORTS entry | New entry appears in unsupported set; partition still valid; no drift in supported set |
| EC-003 | New supported protocol added: KNOWN_PROTOCOLS entry added AND SUPPORTED_PORTS updated | New entry appears in supported set; partition valid; counting invariant holds |
| EC-004 | ARP entry is only entry with `canonical_ports: &[]` in supported set | Partition still valid; ARP special case does not create a phantom entry |
| EC-005 | Port-102 entries (four entries all with `canonical_ports: &[102]`) | All four in unsupported set; union still complete; disjoint holds since 102 is not in SUPPORTED_PORTS |
| EC-006 | All entries are unsupported (hypothetical) | `supported_protocols()` returns empty Vec; `unsupported_protocols()` returns full KNOWN_PROTOCOLS slice; counting invariant holds (0 + N == N) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `supported_protocols().len() + unsupported_protocols().len()` | == `all_protocols().len()` (currently ~30) | counting-invariant |
| Every entry `p` in `KNOWN_PROTOCOLS` | `p` is in exactly one of the two sets | partition-completeness |
| No entry appears in both sets | `supported ∩ unsupported == ∅` | disjoint |
| `supported_protocols()` result contains no entry with `p.name` not in `KNOWN_PROTOCOLS` | No phantom entries | no-phantom |
| `unsupported_protocols()` result contains no entry with `p.name` not in `KNOWN_PROTOCOLS` | No phantom entries | no-phantom |

## Verification Properties

| VP-NNN | Sub | Property | Proof Method |
|--------|-----|----------|-------------|
| VP-041 | oracle | Oracle cross-check (`proptest_vp041_oracle_cross_check`): for each entry in KNOWN_PROTOCOLS, `entry ∈ supported_protocols() ⟺ entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) \|\| entry.name=="ARP"`. Oracle computed INDEPENDENTLY — does NOT call `supported_protocols()` or `unsupported_protocols()` (non-vacuous). Guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency. | proptest: `proptest_vp041_oracle_cross_check` |
| VP-041 | partition | Partition/disjointness (`proptest_vp041_partition_invariant`): `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS` and `supported_protocols() ∩ unsupported_protocols() == ∅`. Verifies union-completeness and disjointness of the two function outputs; holds trivially by the complement derivation (`unsupported = KNOWN \ supported`). `proptest_vp041_oracle_cross_check` provides the non-vacuous guard. | proptest: `proptest_vp041_partition_invariant` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — this BC formalizes the partition invariant that is the foundational correctness property of the Protocol Coverage Catalog: the two coverage sets are jointly exhaustive and mutually exclusive over KNOWN_PROTOCOLS |
| L2 Domain Invariants | None directly (pure-core invariant; no domain-level brownfield invariants apply) |
| Architecture Module | SS-18 (src/protocols.rs C-26) |
| ADR | ADR-012 Decision 5 (SUPPORTED_PORTS compile-time mirror; VP-041 guards set-difference property) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/protocols.rs` — `KNOWN_PROTOCOLS`, `SUPPORTED_PORTS`, `supported_protocols()`, `unsupported_protocols()` — these four items are the complete scope of this invariant
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_oracle_cross_check` (oracle: `entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) || entry.name=="ARP"`; oracle computed independently, does NOT call `supported_protocols()` or `unsupported_protocols()` — non-vacuous; guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency)
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_partition_invariant` (verifies `supported ∪ unsupported == KNOWN_PROTOCOLS` and `supported ∩ unsupported == ∅`; holds trivially by the complement derivation (`unsupported = KNOWN \ supported`); non-vacuous guard is `proptest_vp041_oracle_cross_check`)

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-041 — `proptest_vp041_oracle_cross_check` (per-entry canonical membership predicate; guards `supported_protocols()`-vs-`SUPPORTED_PORTS` consistency; oracle computed independently — non-vacuous)
- VP-041 — `proptest_vp041_partition_invariant` (partition/disjointness: `supported ∪ unsupported == KNOWN_PROTOCOLS`; `supported ∩ unsupported == ∅`; holds trivially by the complement derivation (`unsupported = KNOWN \ supported`); non-vacuous guard is `proptest_vp041_oracle_cross_check`)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | read-only (`&'static` compile-time constants) |
| **Deterministic** | yes |
| **Thread safety** | yes |
| **Overall classification** | pure |
