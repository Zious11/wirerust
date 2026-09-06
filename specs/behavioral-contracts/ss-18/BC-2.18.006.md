---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-18
capability: CAP-18
lifecycle_status: active
introduced: feature-s7comm
modified: []
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

# BC-2.18.006: Port-102 Four-Way `Support` Assignment (S7comm=Supported, S7comm-plus=DetectionOnly, MMS/ICCP=KnownUnsupported) — Static-Catalog-Only Fix, Dynamic Gap Classifier Explicitly Deferred to F4

## Description

The four `KNOWN_PROTOCOLS` entries sharing canonical port TCP/102 — S7comm,
S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 — receive concrete `Support` (BC-2.18.005)
assignments per ADR-014 Decision 3 (RATIFIED) and Decision 6:

| Entry | `Support` value | Rationale |
|---|---|---|
| S7comm | `Supported` | Full classic-S7comm dissection (ADR-014 Decisions 1/2; SS-21 BCs authored in feature-s7comm F2 part B) |
| S7comm-plus | `DetectionOnly` | Framing-level classification + unencrypted session-setup metadata only, per ADR-014 Decision 6 ("observed, not dissected") |
| IEC 61850 MMS | `KnownUnsupported` | Out of scope this cycle (ADR-014 Decision 10) |
| ICCP/TASE.2 | `KnownUnsupported` | Out of scope this cycle (ADR-014 Decision 10) |

This BC also carries the **explicit, testable non-goal** named in the F2 authoring
scope: this cycle's `Support`-enum fix resolves only the *static* catalog partition
(`supported_protocols()`/`unsupported_protocols()`, BC-2.18.003/004). It does **not**
fix `main.rs::lookup_protocol_state` — the *dynamic* coverage-gap tri-state classifier
— which has no per-flow protocol identity to key on and will continue to misreport a
port-102 gap-flow as attributable to whichever port-102 entry it encounters first by
declaration order, regardless of whether the underlying traffic is genuine S7comm,
S7comm-plus, MMS, or ICCP. This defect is correctly and explicitly deferred to a future
F4 cycle (ADR-014 Decision 2 disambiguation table / Decision 10 consequence), pending
the analyzer's parsed COTP `protocol_id` (SS-20/SS-21) becoming available to the gap
classifier.

## Related BCs

- BC-2.18.005 — depends on (this BC's four assignments are concrete instances of the
  `Support` enum's exhaustiveness requirement)
- BC-2.18.003 — composes with (these four assignments are inputs to
  `supported_protocols()`/`unsupported_protocols()`'s per-entry filter)
- BC-2.18.004 — composes with (the partition invariant holds across all four entries
  regardless of which of the three `Support` values each carries)

## Preconditions

1. `KNOWN_PROTOCOLS` contains (or, upon this feature landing, will contain) exactly
   four entries with `canonical_ports: &[102]`: S7comm, S7comm-plus, IEC 61850 MMS,
   ICCP/TASE.2.
2. Each entry's `support:` field (BC-2.18.005) is set per the table above.

## Postconditions

1. `KNOWN_PROTOCOLS.iter().find(|p| p.name == "S7comm").unwrap().support == Support::Supported`.
2. `KNOWN_PROTOCOLS.iter().find(|p| p.name == "S7comm-plus").unwrap().support == Support::DetectionOnly`.
3. `KNOWN_PROTOCOLS.iter().find(|p| p.name == "IEC 61850 MMS").unwrap().support == Support::KnownUnsupported`.
4. `KNOWN_PROTOCOLS.iter().find(|p| p.name == "ICCP/TASE.2").unwrap().support == Support::KnownUnsupported`.
5. Per BC-2.18.003's amended derivation, `supported_protocols()` includes S7comm and
   excludes all three of S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2.
6. Per BC-2.18.003's amended derivation, `unsupported_protocols()` includes **all
   three** of S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 — S7comm-plus is retained in
   the unsupported set precisely because `unsupported_protocols()` filters on
   `support != Support::Supported`, not `support == Support::KnownUnsupported`. Using
   the latter would silently drop S7comm-plus from `unsupported_protocols()` entirely,
   breaking the two-set partition invariant (BC-2.18.004) that `supported_protocols() ⊎
   unsupported_protocols() == KNOWN_PROTOCOLS` must hold.
7. **NON-GOAL (explicit, testable via regression guard):** `main.rs::lookup_protocol_state`'s
   dynamic coverage-gap classification behavior for `(Tcp, 102)` gap-flows is
   **unchanged** by this feature. It continues to attribute an unclassified port-102
   gap flow to whichever port-102 `KNOWN_PROTOCOLS` entry it matches first by
   declaration order — a pre-existing, documented limitation (ADR-012's original
   port-102 caveat) that this cycle's static-catalog fix does not and cannot resolve,
   because `lookup_protocol_state` has no access to the analyzer's parsed COTP
   `protocol_id` at the point it runs.

## Invariants

1. **Static fix, dynamic gap unaffected**: this is the single most important
   distinction this BC encodes. `Support` is a compile-time, per-`KnownProtocol`-entry
   property; `lookup_protocol_state` operates on a raw `(TransportProto, u16)` port
   pair with no protocol identity at all. No catalog-model option — `bool`, exclusion
   list, or `Support` enum — can fix the dynamic classifier, because the fix requires
   information (the analyzer's parsed protocol-ID) that does not exist at the point
   `lookup_protocol_state` runs (ADR-014 Decision 3 critical caveat).
2. **`!= Supported` vs. `== KnownUnsupported` is load-bearing, not stylistic**:
   `unsupported_protocols()` MUST be implemented as the complement of
   `supported_protocols()` (filter `support != Support::Supported`), never as a direct
   filter on `support == Support::KnownUnsupported`. The latter formulation is a subtle
   but critical defect that silently excludes every `DetectionOnly` entry from both
   sets, breaking VP-041's partition invariant. This BC's Postcondition 6 is the
   canonical regression check for this exact defect class.
3. **Four-way collision is unchanged in count, changed in resolution granularity**: the
   port-102 collision documented in ADR-012 (and echoed in `cap-18-protocol-coverage-catalog.md`'s
   existing caveat text) still involves the same four protocols; what changes is that
   the *static* catalog can now express that one of the four (S7comm) is dissected,
   one (S7comm-plus) is partially observed, and two (MMS, ICCP) are neither — where
   previously the static catalog could only say "none of the four are supported."
4. **F4 trigger condition, stated for forward reference**: the eventual dynamic-gap-classifier
   fix (out of scope here) will require `lookup_protocol_state` to accept or derive a
   protocol identity from the analyzer layer (SS-20/SS-21's parsed `protocol_id`) rather
   than deriving attribution from the port number alone — this is the F4 trigger
   condition named in ADR-014 Decision 3's critical caveat and Decision 10's
   consequence, recorded here for the F4 cycle's benefit.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--coverage-gaps` is run against a capture containing genuine S7comm-plus traffic on port 102, with no `--s7comm` or equivalent flag wiring the S7comm-plus DetectionOnly finding path | `lookup_protocol_state` reports the gap-flow using its pre-existing (unfixed) first-declaration-order attribution — potentially attributing it to "S7comm" even though the actual traffic is S7comm-plus. This is the documented, unfixed dynamic-gap behavior (Postcondition 7), not a regression introduced by this feature |
| EC-002 | The `protocols` CLI subcommand (`--all`/`--supported`/`--unsupported`) is run after this feature lands | S7comm appears under `--supported`; S7comm-plus, MMS, and ICCP/TASE.2 all appear under `--unsupported` — this is the *static* surface, which IS correctly resolved by this feature (contrast with EC-001's dynamic surface) |
| EC-003 | A future adversarial or maintenance-sweep pass greps for `== Support::KnownUnsupported` inside `unsupported_protocols()`'s implementation | Zero matches expected — the implementation must use `!= Support::Supported`; any match is a HIGH-severity regression finding per this BC's Invariant 2 |
| EC-004 | S7comm-plus's `Support::DetectionOnly` value interacting with the `--supported`/`--unsupported` CLI filter flags (BC-2.18.001/002) | S7comm-plus is never listed under `--supported`; it IS listed under `--unsupported` (per Postcondition 6) — there is no third CLI filter option for `DetectionOnly` specifically in this cycle |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|---------|
| `supported_protocols()` after this feature lands | Includes S7comm; excludes S7comm-plus, IEC 61850 MMS, ICCP/TASE.2 | happy-path: static supported-set correctness |
| `unsupported_protocols()` after this feature lands | Includes S7comm-plus, IEC 61850 MMS, ICCP/TASE.2 (all three); excludes S7comm | happy-path: static unsupported-set correctness, including DetectionOnly retention |
| `supported_protocols().len() + unsupported_protocols().len()` | Unchanged partition-completeness relative to `KNOWN_PROTOCOLS.len()` (BC-2.18.004) — adding S7comm to `KNOWN_PROTOCOLS` and marking it `Supported` increases both `KNOWN_PROTOCOLS.len()` and `supported_protocols().len()` by 1, preserving the invariant | partition: counting invariant preserved across the four-entry group |
| `main.rs::lookup_protocol_state` behavior for `(Tcp, 102)` gap-flows, before vs. after this feature | Byte-for-byte unchanged — this is the explicit non-goal regression guard (Postcondition 7) | non-goal regression guard |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| VP-041 (existing, amended scope) — the partition/oracle harnesses must pass with S7comm-plus correctly retained in `unsupported_protocols()`, exercising the `!= Supported` vs `== KnownUnsupported` distinction as a concrete non-vacuous case | proptest (existing VP-041 harnesses, amended — not a new VP; amendment finalized in the F2 INTEGRATE sub-burst) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — the port-102 collision is the catalog's own documented flagship caveat (see cap-18-protocol-coverage-catalog.md "Key caveats" section); this BC is the concrete per-entry resolution of that caveat for the four affected entries |
| L2 Domain Invariants | None directly (pure-core; no domain-level invariant from brownfield spec applies) |
| Architecture Module | SS-18 (`src/protocols.rs` C-26, `KNOWN_PROTOCOLS` port-102 entries); SS-20/SS-21 (the eventual F4 consumer of the analyzer's `protocol_id`, out of scope here); `main.rs::lookup_protocol_state` (explicitly NOT modified by this BC) |
| ADR | ADR-014 Decisions 3 (RATIFIED), 6, 10; interacts with ADR-012 (original port-102 caveat and `lookup_protocol_state` design) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure catalog data-model contract; no finding emission) |

## Architecture Anchors

- `src/protocols.rs` — the four port-102 `KnownProtocol` literals (S7comm, S7comm-plus,
  IEC 61850 MMS, ICCP/TASE.2), each with an explicit `support:` field per the table
  above
- `src/main.rs::lookup_protocol_state` — explicitly UNCHANGED by this BC; doc-comment
  should note the F4-deferred limitation per ADR-014 Decision 3's critical caveat
- `.factory/specs/domain/capabilities/cap-18-protocol-coverage-catalog.md` — existing
  "Key caveats" section documenting the port-102 four-way collision (to be refreshed by
  the F2 INTEGRATE sub-burst to reflect the static-fix/dynamic-gap split this BC
  establishes)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 3` —
  "Critical caveat — this does not fully solve port 102" subsection (full non-goal
  rationale)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 10` —
  `PORT_102_NOTE`/`collision_note` F4 consequence

## Story Anchor

STORY-193

## VP Anchors

- VP-041 (existing, amended scope — not a new VP allocation) — must exercise
  S7comm-plus's `DetectionOnly` retention in `unsupported_protocols()` as a concrete
  proptest case; amendment finalized in the F2 INTEGRATE sub-burst

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | read-only (`&'static` compile-time constants) |
| **Deterministic** | yes |
| **Thread safety** | yes |
| **Overall classification** | pure — compile-time data model; the NON-GOAL clause (Postcondition 7) is a statement about a separate, unchanged runtime function (`lookup_protocol_state`), not about this BC's own pure-core scope |
