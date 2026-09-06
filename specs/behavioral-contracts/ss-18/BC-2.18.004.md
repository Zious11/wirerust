---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-18
capability: CAP-18
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified:
  - "v1.1: F-F2P2-002 Pass-2 remediation — second VP-041 harness proptest_vp041_partition_invariant added; non-vacuity clarification. 2026-07-01"
  - "v1.2: F-F2P7-004 Pass-7 remediation — partition harness non-vacuity mislabeling corrected: proptest_vp041_partition_invariant holds trivially by the complement derivation (unsupported = KNOWN \\ supported); proptest_vp041_oracle_cross_check is the non-vacuous guard. Invariant 4, VP table partition row, Architecture Anchors, VP Anchors updated. 2026-07-01"
  - "v1.3: BC-INDEX v2.23 feature-iec104 amendment — EC-003/EC-007 IEC-104 examples added. NEW-GAP-002: inputs and input-hash frontmatter added. 2026-07-13"
  - "v1.4: feature-s7comm F2 part A (ADR-014 Decision 3, RATIFIED option (d)) — the partition is now expressed over the Support enum field (BC-2.18.005) rather than SUPPORTED_PORTS; Invariants 1-3 rewritten to reference support instead of SUPPORTED_PORTS; EC-004/005/007 updated to reflect Support-enum assignments including S7comm-plus's DetectionOnly retention in the unsupported set (BC-2.18.006); VP table and Architecture Anchors updated; counting invariant now includes S7comm as a 9th supported entry. 2026-09-06"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/ss-18-protocol-coverage-catalog.md
  - docs/adr/0012-protocols-catalog-and-coverage-gaps.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
input-hash: "f156347"
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
- BC-2.18.005 — depends on (the `Support` enum this partition is now expressed over)
- BC-2.18.006 — composes with (S7comm-plus's `DetectionOnly` retention in the unsupported set is the concrete case that exercises this BC's partition-preservation invariant)
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

1. The partition is STATIC — it depends only on the compile-time `support: Support`
   field declared on each `KNOWN_PROTOCOLS` literal (BC-2.18.005). It cannot change at
   runtime.
2. Adding a new entry to `KNOWN_PROTOCOLS` requires an explicit `support:` value
   (compile error otherwise, BC-2.18.005) — there is no way to add an entry that
   "accidentally" lands in the wrong set by omission, unlike the pre-v1.4 SUPPORTED_PORTS
   derivation where a forgotten port-list update silently left a new entry unsupported.
3. A `Support::DetectionOnly` entry (e.g. S7comm-plus, BC-2.18.006) belongs to
   `unsupported_protocols()`, not to a third, separate set — the partition remains
   exactly two sets. `DetectionOnly` is a refinement within the "not supported" half of
   the partition, not an independent third partition member.
4. VP-041 (amended scope, F2 INTEGRATE sub-burst) uses TWO harnesses:
   `proptest_vp041_oracle_cross_check` (per-entry oracle cross-check — guards
   `supported_protocols()`-vs-`support`-field consistency; oracle computed
   INDEPENDENTLY without calling `supported_protocols()` or `unsupported_protocols()`
   — non-vacuous) and `proptest_vp041_partition_invariant` (partition/disjointness —
   verifies union completeness and empty intersection of the two function outputs,
   exercised with a `DetectionOnly` entry present). Both harnesses MUST pass.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Catalog at minimum size (1 entry) | Union = {entry}, disjoint trivially satisfied; partition holds |
| EC-002 | New ICS protocol added to KNOWN_PROTOCOLS with `support: Support::KnownUnsupported` | New entry appears in unsupported set; partition still valid; no drift in supported set |
| EC-003 | New supported protocol added: IEC-104 KNOWN_PROTOCOLS entry with `support: Support::Supported` (feature-iec104, carried forward unchanged by this v1.4 amendment) | IEC-104 entry appears in supported set; partition valid; counting invariant holds |
| EC-004 | ARP entry's `support` field is `Support::Supported`, declared directly (no port-based special case exists as of v1.4) | Partition still valid; no phantom entry, no special-case code path |
| EC-005 | Port-102 entries: S7comm (`Support::Supported`), S7comm-plus (`Support::DetectionOnly`), IEC 61850 MMS (`Support::KnownUnsupported`), ICCP/TASE.2 (`Support::KnownUnsupported`) — feature-s7comm, BC-2.18.006 | S7comm in supported set; S7comm-plus, MMS, and ICCP/TASE.2 all three in unsupported set (DetectionOnly is retained in unsupported, not dropped from both sets); union still complete; disjoint holds |
| EC-006 | All entries are unsupported (hypothetical, all `Support::KnownUnsupported` or `Support::DetectionOnly`) | `supported_protocols()` returns empty Vec; `unsupported_protocols()` returns full KNOWN_PROTOCOLS slice; counting invariant holds (0 + N == N) |
| EC-007 | IEC-104 entry with `support: Support::Supported` (feature-iec104, carried forward) | IEC-104 remains in the supported set; counting invariant holds; partition remains valid |
| EC-008 | A hypothetical (incorrect) implementation of `unsupported_protocols()` using `support == Support::KnownUnsupported` instead of `support != Support::Supported` | S7comm-plus (`DetectionOnly`) would be absent from BOTH sets — a phantom omission that violates Postcondition 1 (union completeness); this is the canonical regression this BC's partition invariant guards against (see BC-2.18.003 Invariant 3) |

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
| VP-041 (amended scope, F2 INTEGRATE sub-burst) | oracle | Oracle cross-check (`proptest_vp041_oracle_cross_check`, amended): for each entry in KNOWN_PROTOCOLS, membership in `supported_protocols()` holds if and only if that entry's `support` field equals `Support::Supported`. Oracle computed INDEPENDENTLY — does NOT call `supported_protocols()` or `unsupported_protocols()` (non-vacuous). Guards `supported_protocols()`-vs-`support`-field consistency. | proptest: `proptest_vp041_oracle_cross_check` |
| VP-041 (amended scope) | partition | Partition/disjointness (`proptest_vp041_partition_invariant`): the union of `supported_protocols()` and `unsupported_protocols()` equals `KNOWN_PROTOCOLS`, and their intersection is empty. Verifies union-completeness and disjointness of the two function outputs, exercised with a `Support::DetectionOnly` entry present to guard the retention property (BC-2.18.006). `proptest_vp041_oracle_cross_check` provides the non-vacuous guard. | proptest: `proptest_vp041_partition_invariant` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — this BC formalizes the partition invariant that is the foundational correctness property of the Protocol Coverage Catalog: the two coverage sets are jointly exhaustive and mutually exclusive over KNOWN_PROTOCOLS |
| L2 Domain Invariants | None directly (pure-core invariant; no domain-level brownfield invariants apply) |
| Architecture Module | SS-18 (src/protocols.rs C-26); `Support` enum; `KnownProtocol.support` field |
| ADR | ADR-014 Decision 3 (RATIFIED option (d), 2026-09-06 — supersedes ADR-012 Decision 5's SUPPORTED_PORTS-based derivation for the partition's underlying functions) |
| Stories | STORY-151 (F3 feature-protocol-coverage — original implementation); (TBD — F3 story-writer assigns the feature-s7comm amendment story) |

## Architecture Anchors

- `src/protocols.rs` — `KNOWN_PROTOCOLS`, `Support` enum, `KnownProtocol.support` field, `supported_protocols()`, `unsupported_protocols()` — these are the complete scope of this invariant as of v1.4
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_oracle_cross_check` (amended oracle keyed on `entry.support == Support::Supported`; oracle computed independently, does NOT call `supported_protocols()` or `unsupported_protocols()` — non-vacuous; guards `supported_protocols()`-vs-`support`-field consistency; amendment finalized in the F2 INTEGRATE sub-burst)
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_partition_invariant` (verifies union completeness and empty intersection, exercised with a `DetectionOnly` entry present; amendment finalized in the F2 INTEGRATE sub-burst)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 3` — full ratified `Support` enum definition and partition-preservation rationale for `DetectionOnly`

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-041 (amended scope, F2 INTEGRATE sub-burst — not a new VP) — `proptest_vp041_oracle_cross_check` (per-entry canonical membership predicate; guards `supported_protocols()`-vs-`support`-field consistency; oracle computed independently — non-vacuous)
- VP-041 (amended scope) — `proptest_vp041_partition_invariant` (partition/disjointness over `KNOWN_PROTOCOLS`, exercised with a `Support::DetectionOnly` entry present; non-vacuous guard is `proptest_vp041_oracle_cross_check`)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | read-only (`&'static` compile-time constants) |
| **Deterministic** | yes |
| **Thread safety** | yes |
| **Overall classification** | pure |
