---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.1: F-F2P2-001/002 Pass-2 remediation — VP-041 anti-drift semantics corrected; second VP-041 harness added. 2026-07-01"
  - "v1.2: F-F2P5-001 Pass-5 remediation — SUPPORTED_PORTS semantics reframed per ADR-012 canonical wording (not a pure classify() mirror); DNS/53 decode-loop path documented; Architecture Anchor doc-comment obligation updated verbatim; EC-005 clarified. 2026-07-01"
  - "v1.3: F-F2P7-004 Pass-7 remediation — partition harness non-vacuity mislabeling corrected: proptest_vp041_partition_invariant holds trivially by the complement derivation (unsupported = KNOWN \\ supported); proptest_vp041_oracle_cross_check is the non-vacuous guard. VP table partition row, Architecture Anchors, VP Anchors updated. 2026-07-01"
  - "v1.4: BC-INDEX v2.23 feature-iec104 amendment — SUPPORTED_PORTS adds port 2404 (IEC-104); supported entries count 7→8; port 2404 reflected in Precondition 3, Invariant 1, EC-005, Canonical Test Vectors. NEW-GAP-002: inputs and input-hash frontmatter added. 2026-07-13"
  - "v1.5: F7-L3 — unsupported_protocols() canonical test vector count corrected: ~23 → ~22 (IEC-104 moved from unsupported to supported in v1.4; unsupported count decrements by 1). 2026-07-14"
  - "v1.6: feature-s7comm F2 part A (ADR-014 Decision 3, RATIFIED option (d)) — derivation mechanism replaced: supported_protocols()/unsupported_protocols() now filter on the new Support enum field (BC-2.18.005) instead of the SUPPORTED_PORTS port-intersection-plus-ARP-special-case. unsupported_protocols() MUST filter support != Support::Supported, NOT support == Support::KnownUnsupported, to correctly retain DetectionOnly entries (S7comm-plus) in the unsupported set (BC-2.18.006). Title, Description, Preconditions, Postconditions, Invariants, Edge Cases, Canonical Test Vectors, VP table, and Architecture Anchors updated. SUPPORTED_PORTS is retired as the derivation mechanism for these two functions (it may persist elsewhere, e.g. as informational/legacy documentation, but is no longer load-bearing for this BC — a full removal decision is deferred to the F2 INTEGRATE sub-burst / architect). H1 title change requires BC-INDEX title-column propagation, deferred to the F2 INTEGRATE sub-burst per this cycle's explicit no-index-edit constraint. 2026-09-06"
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

# BC-2.18.003: `supported_protocols()` Returns Exactly the `Support::Supported`-Tagged Entries; `unsupported_protocols()` Returns the Complement (`!= Supported`)

## Description

`supported_protocols()` is a pure-core function in `src/protocols.rs` that returns the
subset of `KNOWN_PROTOCOLS` whose `support` field (the `Support` enum, BC-2.18.005) is
exactly `Support::Supported`. `unsupported_protocols()` returns the complement: every
entry `p` such that `p.support != Support::Supported` — critically, this is **not** the
same as filtering on `p.support == Support::KnownUnsupported`, because that formulation
would silently exclude every `Support::DetectionOnly` entry (S7comm-plus, this cycle;
see BC-2.18.006) from both sets, breaking the two-set partition invariant BC-2.18.004
formalizes. Both functions remain pure and deterministic; they have no I/O, no mutable
state, and no runtime dependencies.

**Superseded derivation (pre-v1.6, retained here for historical/migration context):**
prior to ADR-014 Decision 3's ratified `Support` enum, `supported_protocols()` was
computed from `canonical_ports ∩ SUPPORTED_PORTS` (a compile-time port-list constant)
plus a hand-coded ARP name special case (`|| p.name == "ARP"`), because ARP has
`canonical_ports: &[]` and could not be detected by port intersection at all. That
derivation is retired for this BC's scope as of v1.6: every entry's supported status,
including ARP's, is now expressed directly and exclusively via its `support:` field —
no special-casing is needed anywhere in this function.

This contract guards against drift between what the catalog reports as "supported" and
what `supported_protocols()`/`unsupported_protocols()` compute from the `support`
field. VP-041 (amended scope, F2 INTEGRATE sub-burst) guards the consistency between
these two functions and the `Support` value declared on each `KNOWN_PROTOCOLS` literal.

## Related BCs

- BC-2.18.004 — composes with (partition invariant: supported ∪ unsupported == KNOWN_PROTOCOLS)
- BC-2.18.005 — depends on (the `Support` enum and `KnownProtocol.support` field this BC's derivation filters on)
- BC-2.18.006 — composes with (the port-102 four-way assignment is the concrete case that makes the `!= Supported` vs `== KnownUnsupported` distinction load-bearing)
- BC-2.18.001 — depends on (the `--supported` / `--unsupported` filter flags in terminal output call these functions)
- BC-2.18.002 — depends on (same for JSON output)

## Preconditions

1. `supported_protocols()` is called as a pure function — no mutable state, no I/O.
2. `KNOWN_PROTOCOLS` is a non-empty static array (compile-time constant).
3. Every entry in `KNOWN_PROTOCOLS` has an explicit `support: Support` field (BC-2.18.005) — one of `Support::Supported`, `Support::KnownUnsupported`, or `Support::DetectionOnly`.

## Postconditions

1. `supported_protocols()` returns all entries `p` in `KNOWN_PROTOCOLS` such that
   `p.support == Support::Supported`.
2. `unsupported_protocols()` returns all entries `p` in `KNOWN_PROTOCOLS` such that
   `p.support != Support::Supported` — this is the complement of `supported_protocols()`,
   and includes both `Support::KnownUnsupported` AND `Support::DetectionOnly` entries.
   It is **not** equivalent to filtering on `p.support == Support::KnownUnsupported`,
   which would incorrectly drop every `DetectionOnly` entry from the result.
3. The ARP entry is in `supported_protocols()` because its `support` field is declared
   `Support::Supported` directly — no port-intersection special case is needed or
   present (ARP's `canonical_ports: &[]` no longer has any bearing on this
   determination, unlike the pre-v1.6 derivation).
4. Every entry in `KNOWN_PROTOCOLS` appears in exactly one of the two result sets
   (partition invariant, formalized in BC-2.18.004).

## Invariants

1. `supported_protocols()`/`unsupported_protocols()` derive exclusively from the
   compile-time `support: Support` field declared on each `KNOWN_PROTOCOLS` literal
   (BC-2.18.005) — there is no port-based, name-based, or list-based special-casing of
   any kind in either function's implementation as of v1.6.
2. `supported_protocols()` is pure and referentially transparent — the same call always
   returns the same result (given the same compile-time constants).
3. `unsupported_protocols()` MUST be implemented as `p.support != Support::Supported`,
   **never** `p.support == Support::KnownUnsupported`. This distinction is load-bearing
   (BC-2.18.006 Invariant 2): the latter formulation silently excludes
   `Support::DetectionOnly` entries from both result sets, breaking the partition
   invariant (BC-2.18.004).
4. `unsupported_protocols()` MUST NOT be a separate hand-maintained list; it must be
   derived as the complement of `supported_protocols()` within `KNOWN_PROTOCOLS`.
5. The sets are stable across the same binary build. They cannot change at runtime.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ARP entry's `support` field is `Support::Supported` (declared directly, no port involved) | ARP appears in `supported_protocols()`; no special-case code path exists anymore |
| EC-002 | TLS has two ports (443 and 8443) and `support: Support::Supported` | TLS appears once in `supported_protocols()`; not duplicated (single struct literal, single `support` field) |
| EC-003 | BACnet/IP's `support` field is `Support::KnownUnsupported` | BACnet/IP appears in `unsupported_protocols()` |
| EC-004 | GOOSE's `support` field is `Support::KnownUnsupported` (and `canonical_ports: &[]`, `port_detectable: false`) | GOOSE appears in `unsupported_protocols()`; port-detectability has no bearing on this function |
| EC-005 | IEC-104's `support` field is `Support::Supported` (feature-iec104; carried forward unchanged by this v1.6 amendment) | IEC-104 entry is in `supported_protocols()`; supported count remains 8 |
| EC-006 | `unsupported_protocols()` is called — result is `KNOWN_PROTOCOLS` minus the supported set | Exact complement; no manual list |
| EC-007 | Port-102 entries: S7comm's `support` is `Support::Supported`; S7comm-plus's is `Support::DetectionOnly`; IEC 61850 MMS's and ICCP/TASE.2's are both `Support::KnownUnsupported` (feature-s7comm, BC-2.18.006) | S7comm is in `supported_protocols()`; S7comm-plus, MMS, and ICCP/TASE.2 are **all three** in `unsupported_protocols()` — S7comm-plus's `DetectionOnly` status does NOT exempt it from the unsupported set (see Invariant 3) |
| EC-008 | `unsupported_protocols()` implemented (incorrectly, as a regression) with `p.support == Support::KnownUnsupported` | S7comm-plus (`DetectionOnly`) would be silently absent from BOTH `supported_protocols()` and `unsupported_protocols()`, breaking BC-2.18.004's partition invariant — this is the canonical regression this BC's Invariant 3 exists to prevent |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `supported_protocols()` | Returns 9 entries: Modbus/TCP, DNP3, EtherNet/IP+CIP, IEC-104, TLS, ARP, DNS, HTTP, S7comm (all with `support: Support::Supported`) | happy-path |
| `unsupported_protocols()` | Returns all other entries (~22, including S7comm-plus, IEC 61850 MMS, ICCP/TASE.2, BACnet/IP, GOOSE, etc.) | happy-path |
| ARP in `supported_protocols()` result | `p.name == "ARP"` present, via `support == Support::Supported`, no special case | ARP-via-Support-enum |
| S7comm-plus in `unsupported_protocols()` result (NOT via `== KnownUnsupported`) | `p.name == "S7comm-plus"` present, via `support != Support::Supported` (its actual value is `Support::DetectionOnly`) | DetectionOnly-retention regression guard |
| `supported_protocols().len() + unsupported_protocols().len()` | == `KNOWN_PROTOCOLS.len()` | partition |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-041 (amended scope, F2 INTEGRATE sub-burst) | Oracle cross-check (`proptest_vp041_oracle_cross_check`, amended): for each entry in KNOWN_PROTOCOLS, `entry ∈ supported_protocols() ⟺ entry.support == Support::Supported`. Oracle is computed INDEPENDENTLY — it does NOT call `supported_protocols()` or `unsupported_protocols()` (non-vacuous). Guards `supported_protocols()`-vs-`support`-field consistency. | proptest: `proptest_vp041_oracle_cross_check` (harness body updated by the F2 INTEGRATE sub-burst) |
| VP-041 (amended scope) | Partition/disjointness (`proptest_vp041_partition_invariant`): `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS` and `supported_protocols() ∩ unsupported_protocols() == ∅`, exercised with at least one `Support::DetectionOnly` entry (S7comm-plus) present to guard against the `== KnownUnsupported` regression. | proptest: `proptest_vp041_partition_invariant` |
| — | S7comm-plus (`DetectionOnly`) is present in `unsupported_protocols()`, not absent from both sets | unit: `test_BC_2_18_003_detection_only_retained_in_unsupported` (new, F3) |
| — | BACnet/IP (`KnownUnsupported`) is in `unsupported_protocols()` | unit: `test_BC_2_18_003_bacnet_unsupported` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — `supported_protocols()` and `unsupported_protocols()` are the canonical pure-core functions that partition the Protocol Coverage Catalog into what wirerust dissects versus what it knows about but does not dissect |
| L2 Domain Invariants | None directly (pure-core; no domain-level invariants from brownfield spec apply) |
| Architecture Module | SS-18 (src/protocols.rs C-26); `Support` enum; `KnownProtocol.support` field; `supported_protocols()` and `unsupported_protocols()` functions |
| ADR | ADR-014 Decision 3 (RATIFIED option (d), 2026-09-06 — supersedes ADR-012 Decision 5's SUPPORTED_PORTS-intersection derivation for these two functions) |
| Stories | STORY-151 (F3 feature-protocol-coverage — original implementation); (TBD — F3 story-writer assigns the feature-s7comm amendment story) |

## Architecture Anchors

- `src/protocols.rs` — `pub enum Support { Supported, KnownUnsupported, DetectionOnly }` and `KnownProtocol.support: Support` field (BC-2.18.005)
- `src/protocols.rs` — `pub fn supported_protocols() -> Vec<&'static KnownProtocol>` — returns entries with `support == Support::Supported`
- `src/protocols.rs` — `pub fn unsupported_protocols() -> Vec<&'static KnownProtocol>` — returns entries with `support != Support::Supported` (the complement; MUST NOT be `== Support::KnownUnsupported`)
- `src/protocols.rs` — `pub fn all_protocols() -> &'static [KnownProtocol]` — returns full `KNOWN_PROTOCOLS` slice (unchanged)
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_oracle_cross_check` (amended oracle: `entry ∈ supported_protocols() ⟺ entry.support == Support::Supported`; oracle computed independently, does NOT call `supported_protocols()` or `unsupported_protocols()` — non-vacuous; amendment finalized in the F2 INTEGRATE sub-burst)
- `tests/protocols_tests.rs` — VP-041 proptest harness `proptest_vp041_partition_invariant` (verifies `supported ∪ unsupported == KNOWN_PROTOCOLS` and `supported ∩ unsupported == ∅`, exercised with a `DetectionOnly` entry present; amendment finalized in the F2 INTEGRATE sub-burst)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 3` — full ratified `Support` enum definition and derivation rewrite

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-041 (amended scope, F2 INTEGRATE sub-burst — not a new VP) — `proptest_vp041_oracle_cross_check`: per-entry canonical membership predicate; guards `supported_protocols()`-vs-`support`-field consistency; oracle computed independently (non-vacuous — does NOT call `supported_protocols()`)
- VP-041 (amended scope) — `proptest_vp041_partition_invariant`: partition/disjointness of `supported_protocols()` and `unsupported_protocols()` over `KNOWN_PROTOCOLS`, exercised with a `Support::DetectionOnly` entry present

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | read-only (`KNOWN_PROTOCOLS` and `SUPPORTED_PORTS` are `&'static` compile-time constants) |
| **Deterministic** | yes (same binary always produces same result) |
| **Thread safety** | yes (no mutable state) |
| **Overall classification** | pure |
