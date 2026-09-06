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

# BC-2.18.005: `Support` Enum on `KnownProtocol` — Exhaustive, Compile-Time-Enforced Per-Entry Assignment

## Description

`KnownProtocol` gains a new field, `pub support: Support`, backed by an exhaustively
matched three-variant enum (ADR-014 Decision 3, RATIFIED option (d)):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// A full dissector exists for this protocol.
    Supported,
    /// No dissector exists for this protocol at all.
    KnownUnsupported,
    /// Framing-level classification / observation only — never promoted to a full
    /// dissector.
    DetectionOnly,
}
```

This replaces the previous derivation of "is this protocol supported" from
`canonical_ports ∩ SUPPORTED_PORTS` (plus a hand-coded ARP name exception) with an
explicit, per-entry, compile-time-declared field. Because Rust struct-expression rules
require every `KnownProtocol` literal to supply every field, omitting `support:` on any
of the ~30 catalog entries is a **compile error**, not a silent behavior — this is the
"safe positive polarity" ADR-014 Decision 3 identifies as decisive over the
originally-recommended name-keyed exclusion-list alternative (option b), which had an
unsafe default-allow polarity for any new same-port catalog entry. `DetectionOnly` is a
genuine third state — a refinement of "not fully supported" that a `bool` field cannot
express — required because S7comm-plus (BC-2.18.006) needs it.

## Related BCs

- BC-2.18.003 — depends on (this BC's `support` field is what BC-2.18.003's amended
  `supported_protocols()`/`unsupported_protocols()` derivation filters on)
- BC-2.18.004 — depends on (the partition invariant continues to hold over the new
  derivation)
- BC-2.18.006 — composes with (the four port-102 per-entry assignments are concrete
  instances of this BC's exhaustiveness requirement)

## Preconditions

1. `KNOWN_PROTOCOLS: &[KnownProtocol]` is the existing ~30-entry static catalog array
   in `src/protocols.rs`.
2. Every `KnownProtocol` struct literal in `KNOWN_PROTOCOLS` is a Rust struct
   expression (not `..Default::default()` or similar field-skipping construct).

## Postconditions

1. `KnownProtocol` has a new field `pub support: Support`.
2. `Support` has exactly three variants: `Supported`, `KnownUnsupported`,
   `DetectionOnly` — derived (`Debug, Clone, Copy, PartialEq, Eq`), no `Default` impl
   (a default would reintroduce the unsafe-omission failure mode this design exists to
   prevent).
3. Every one of the ~30 `KnownProtocol` literals in `KNOWN_PROTOCOLS` supplies an
   explicit `support:` value; omitting the field for any entry fails to compile.
4. For the 26 pre-existing entries whose supported/unsupported status is unchanged by
   this feature (Modbus, DNP3, ENIP, IEC-104, TLS, DNS, HTTP, ARP → `Supported`; every
   other pre-existing entry → `KnownUnsupported`), the assigned `Support` value exactly
   mirrors what `canonical_ports ∩ SUPPORTED_PORTS` (plus the ARP special case) computed
   for that entry immediately before this feature — no behavioral change for these 26
   entries, only a change in how the fact is expressed.
5. `protocols.rs` remains a documented pure-core leaf: `Support` and its use on
   `KnownProtocol` introduce no dependency on `dispatcher.rs` or any other module
   (ADR-014 Decision 3, rejecting option (c)'s `dispatch_target: Option<&'static str>`
   alternative for exactly this reason).

## Invariants

1. **No `Default` derive on `Support`**: a default value would let a new catalog entry
   silently compile with an unintended support state (e.g. silently defaulting to
   `Supported`), reintroducing the unsafe polarity this design rejects. Every entry's
   `support:` value must be an explicit, deliberate choice in the source.
2. **Three variants are exhaustive for this cycle's needs**: `Supported` /
   `KnownUnsupported` / `DetectionOnly` are the only states any current or
   near-term-planned catalog entry requires (per ADR-014 Decision 3's validation
   brief). Extending the enum in the future (a hypothetical fourth state) is not
   precluded, but is out of scope here.
3. **`Support` reuses ADR-0012 Decision 2's vocabulary, applied to a new axis**: the
   *dynamic* coverage-gap tri-state (`known-supported`/`known-unsupported`/`unknown`,
   ADR-012 Decision 2) and this *static* catalog-partition tri-state
   (`Supported`/`KnownUnsupported`/`DetectionOnly`) share vocabulary lineage
   (Suricata-derived) but are **not the same enum** and are not interchangeable —
   `Support` has no `unknown` variant (every catalog entry has a known, declared
   support state at compile time; "unknown" is meaningful only for the dynamic,
   runtime coverage-gap classifier, which operates on ports it has never seen a
   catalog entry for at all).
4. **Compile-time enforcement is the entire correctness mechanism**: unlike VP-041's
   proptest oracle (which runs at test time), the "every entry has an explicit support
   value" property requires no runtime proof — it is enforced unconditionally by the
   Rust compiler on every build.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A developer adds a new `KnownProtocol` literal to `KNOWN_PROTOCOLS` without a `support:` field | Compile error — "missing field `support`" |
| EC-002 | A developer attempts `KnownProtocol { support: Support::default(), .. }` | Compile error — `Support` has no `Default` impl |
| EC-003 | An entry's support state genuinely changes in a future cycle (e.g. a future MMS cycle promotes IEC 61850 MMS to `Supported`) | A one-line change to that entry's existing `support:` field — no separate exclusion list to keep in sync (contrast with the rejected option (b)) |
| EC-004 | ARP, which previously required a hand-coded `\|\| p.name == "ARP"` special case in `supported_protocols()` because it has `canonical_ports: &[]` | Now expressed directly as `support: Support::Supported` on the ARP literal — no special-casing needed anywhere in the derivation function (BC-2.18.003) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|---------|
| `KNOWN_PROTOCOLS.len()` compared against the count of literals with an explicit `support:` field (via `cargo check` / compilation success) | Equal — compilation succeeds iff every literal supplies the field | happy-path: compile-time exhaustiveness |
| `Support::Supported == Support::Supported` | `true` (derives `PartialEq, Eq`) | happy-path: equality |
| `Support::DetectionOnly != Support::KnownUnsupported` | `true` | happy-path: distinctness (load-bearing for BC-2.18.003's `!= Supported` vs `== KnownUnsupported` distinction) |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| VP-041 (existing, amended scope) — the oracle cross-check is re-anchored from the `SUPPORTED_PORTS` intersection to the `entry.support` field; see BC-2.18.003 for the full amended harness description | proptest (existing VP-041 harnesses, amended — not a new VP; VP amendment is part of the F2 INTEGRATE sub-burst) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — the `Support` enum is the compile-time-enforced data model that makes the Protocol Coverage Catalog's supported/unsupported partition an explicit per-entry fact rather than a derived port-intersection heuristic |
| L2 Domain Invariants | None directly (pure-core; no domain-level invariant from brownfield spec applies) |
| Architecture Module | SS-18 (`src/protocols.rs` C-26); `Support` enum, `KnownProtocol.support` field |
| ADR | ADR-014 Decision 3 (RATIFIED option (d), human 2026-09-06, per `.factory/cycles/feature-s7comm/f2-port102-model-validation.md`); supersedes ADR-014's own original option (b) recommendation; interacts with ADR-012 Decision 2 (vocabulary lineage) and Decision 5 (superseded derivation) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none — pure data-model contract; no finding emission) |

## Architecture Anchors

- `src/protocols.rs` — `pub enum Support { Supported, KnownUnsupported, DetectionOnly }` (new, `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, no `Default`)
- `src/protocols.rs` — `pub struct KnownProtocol { .., pub support: Support }` (new field)
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 3` — full RATIFIED enum definition, safe-positive-polarity rationale, prior-art comparison (Wireshark/Suricata/Zeek)
- `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` — SS-18 catalog data model shard (to be updated by architect with the `Support` enum in the F2 INTEGRATE sub-burst)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-041 (existing, amended scope — not a new VP allocation) — oracle cross-check
  re-anchored from `SUPPORTED_PORTS` intersection to `entry.support`; amendment
  finalized in the F2 INTEGRATE sub-burst

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | read-only (`&'static` compile-time constant) |
| **Deterministic** | yes |
| **Thread safety** | yes |
| **Overall classification** | pure — compile-time data model |
