---
document_type: verification-property
level: L4
version: "2.1"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.10.005
bcs:
  - BC-2.10.005
  - BC-2.10.006
  - BC-2.10.007
  - BC-2.10.008
module: src/mitre.rs
proof_method: kani
feasibility: feasible
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "c1f1063d076a3effe4d5b650deffaf12cf5420804d78d62e9c35675b9f3fc0c1"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set."
  - "2026-06-09 (F2 issue #7, directives v2 + ADR-006): appended non-normative 'F4 Harness-Update Obligations' body section recording the catalog growth (SEEDED 15→21, EMITTED 6→13, recon T0846→T0888) and the Finding field-rename grep change (mitre_technique:Some → mitre_techniques:vec!). Lock fields (verification_lock, proof_completed_date, proof_file_hash, verified_at_commit), property statement, and source BCs are UNCHANGED — this is an F4 obligation pointer, authoritative copy in verification-delta.md §4."
  - "v2.1 (2026-06-10, issue #222): Updated format invariant in Sub-property A to explicitly accept ICS sub-technique IDs (T[0-9]{4}(\\.[0-9]{3})? — the optional .[0-9]{3} suffix was already present in code but not stated in this VP spec). Updated SEEDED_IDS array in harness skeleton: T0855→T1692.001, T0856→T1692.002 (MITRE ATT&CK-ICS v19.1 revocation, both IDs map to parent T1692 Unauthorized Message). Updated EMITTED_IDS similarly for T0855→T1692.001. Added Known Limitation note documenting that VP-007 is a closed-world consistency check and cannot detect external ATT&CK revocations; references issue #222 as the defect that escaped this gap."
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-007: MITRE Technique ID Format and Catalog Completeness

## Property Statement

The MITRE catalog in `mitre.rs` satisfies two sub-properties:

**Sub-property A (ID format):** Every technique ID string returned by `technique_name()`
and `technique_tactic()` matches the regex pattern `T[0-9]{4}(\.[0-9]{3})?`.
That is: the letter `T` followed by exactly four decimal digits, optionally followed
by a period and exactly three decimal digits.

This pattern covers three distinct ID shapes present in the catalog:
- **Enterprise technique:** `T` + 4 digits (e.g. `T1027`, `T1036`)
- **Enterprise sub-technique:** `T` + 4 digits + `.` + 3 digits (e.g. `T1071.001`, `T1499.002`, `T1505.003`)
- **ICS technique:** `T` + 4 digits starting with 0 (e.g. `T0836`, `T0814`, `T0888`)
- **ICS sub-technique:** `T` + 4 digits starting with 0 or 1 + `.` + 3 digits (e.g. `T1692.001`, `T1692.002`)

ICS sub-technique IDs use the same `T[0-9]{4}.[0-9]{3}` shape as Enterprise sub-techniques;
both are accepted by the same regex branch. No special-casing is required for ICS vs Enterprise
sub-techniques.

No other format is present in the static match.

**Sub-property B (emitter-catalog completeness):** Every technique ID string that
any analyzer places into `Finding.mitre_technique` is present in the static match
in `technique_info`, such that `technique_name(id)` returns `Some(...)` and
`technique_tactic(id)` returns `Some(...)` for that ID.

Corollary: Unknown IDs passed to `technique_name`/`technique_tactic` return `None`
and do NOT cause a panic (BC-2.10.006).

## Source Contract

- **Primary BC:** BC-2.10.005 -- technique_name Returns Some for Every Seeded ID (21 Total; post-F2 + issue #222 remap)
- **Postcondition:** `technique_name(id).is_some()` for all 21 seeded IDs
- **Invariant:** INV-9 (MITRE Technique ID Format, inv-01-core-invariants.md)
- **Related BC:** BC-2.10.006 -- technique_name Returns None for Unknown IDs
- **Related BC:** BC-2.10.007 -- technique_tactic Returns Correct Tactic for Every Seeded ID
- **Related BC:** BC-2.10.008 -- All Emitted Technique IDs Resolve in Lookup

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- enumerate all 21 seeded IDs explicitly | Complete coverage of the static match table |

The static match in `mitre.rs` is closed-world enumeration. Kani can enumerate all
21 known IDs (including ICS sub-technique IDs in `T[0-9]{4}.[0-9]{3}` form) plus
verify that the return type (Option) is handled correctly.

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // All 21 seeded IDs from mitre.rs technique_info (src/mitre.rs — post-F2 + issue #222 remap)
    // ICS sub-technique IDs T1692.001 and T1692.002 replace the revoked T0855 and T0856
    // per MITRE ATT&CK-ICS v19.1 (issue #222).
    const SEEDED_IDS: &[&str] = &[
        // Enterprise (11)
        "T1027",
        "T1036",
        "T1040",
        "T1046",
        "T1071",
        "T1071.001",
        "T1071.004",
        "T1083",
        "T1499.002",
        "T1505.003",
        "T1573",
        // ICS (10)
        "T0846",
        "T1692.001",  // was T0855 (revoked v19.1 → T1692 Unauthorized Message: Command Message)
        "T1692.002",  // was T0856 (revoked v19.1 → T1692 Unauthorized Message: Reporting Message)
        "T0885",
        "T0836",
        "T0814",
        "T0806",
        "T0835",
        "T0831",
        "T0888",
    ];

    #[kani::proof]
    fn verify_all_seeded_ids_resolve() {
        for id in SEEDED_IDS {
            let name = technique_name(id);
            assert!(name.is_some(),
                "seeded ID {} not found in technique_info", id);

            let tactic = technique_tactic(id);
            assert!(tactic.is_some(),
                "seeded ID {} has no tactic in technique_info", id);
        }
    }

    #[kani::proof]
    fn verify_unknown_id_returns_none_no_panic() {
        // An ID that is definitely not in the catalog
        let unknown = "TXXXX";
        let name = technique_name(unknown);
        assert!(name.is_none());
        let tactic = technique_tactic(unknown);
        assert!(tactic.is_none());
    }
}

// Compile-time check: all IDs emitted by analyzers are in seeded list.
// This is verified by a build-time test that scans analyzer source for
// hardcoded technique ID strings and checks each against the catalog.
#[cfg(test)]
mod catalog_completeness {
    use super::*;

    // IDs emitted by current analyzers (compile-time constant list)
    // Post-F2 (issue #7) + issue #222 remap: 13 total (6 Enterprise + 7 ICS)
    // T0855 replaced by T1692.001 (MITRE ATT&CK-ICS v19.1 revocation, issue #222)
    const EMITTED_IDS: &[&str] = &[
        // Enterprise (6)
        "T1027",      // TLS analyzer: SNI anomaly
        "T1036",      // Reassembly: conflicting overlap
        "T1046",      // HTTP: admin panel
        "T1083",      // HTTP: path traversal
        "T1499.002",  // HTTP: header flood
        "T1505.003",  // HTTP: web shell
        // ICS (7) — Modbus analyzer (post-F2)
        "T1692.001",  // was T0855: Modbus write FCs (issue #222 remap)
        "T0836",      // Modbus: Modify Parameter
        "T0814",      // Modbus: Denial of Service
        "T0806",      // Modbus: Brute Force I/O
        "T0835",      // Modbus: Manipulate I/O Image
        "T0831",      // Modbus: Manipulation of Control
        "T0888",      // Modbus recon path: Remote System Information Discovery
    ];

    #[test]
    fn all_emitted_ids_resolve_in_catalog() {
        for id in EMITTED_IDS {
            assert!(technique_name(id).is_some(),
                "emitted ID {} not in mitre catalog", id);
            assert!(technique_tactic(id).is_some(),
                "emitted ID {} has no tactic", id);
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Finite | 21 seeded IDs; closed enumeration |
| Proof complexity | Very low | Static match table; no loops or state |
| Tool support | High | `technique_name` is a pure function returning Option<&str> |
| Estimated proof time | < 10 seconds | Trivial Kani proof; simple unit test sufficient as fallback |

## Source Location

`src/mitre.rs:122-156` -- `technique_info` static match block.

The 21 currently-seeded IDs include T1040, T1071, T1071.001, T1071.004, T1573,
T0846, T1692.002, T0885 which are staged-but-never-emitted (O-04). Sub-property
B (emitter-catalog completeness) only tests the IDs that analyzers actually emit.

## Known Limitation: VP-007 Cannot Detect External ATT&CK Revocations

VP-007 is a **closed-world internal consistency** check. It verifies that:
- every seeded ID satisfies the `T[0-9]{4}(\.[0-9]{3})?` format invariant,
- every seeded ID resolves in `technique_info` (no missing arms), and
- every emitted ID is a subset of the seeded catalog (no unknown IDs on findings).

It has **no oracle for whether a seeded/emitted ID is still active in the external
ATT&CK standard.** An ID that has been revoked or renamed by MITRE will still pass
all VP-007 assertions as long as the internal catalog is self-consistent.

This gap was the root cause of **issue #222**: `T0855` (revoked under ATT&CK-ICS
v19.1, remapped to `T1692.001`) and `T0856` (remapped to `T1692.002`) passed all
VP-007 assertions in v0.3.0 and v0.4.0 while the product advertised v19.1
conformance. The defect was only surfaced by a manual external reconciliation
audit (`.factory/research/mitre-ics-v19-catalog-audit.md`).

**Scope boundary (normative):** VP-007 proves internal catalog self-consistency.
External ATT&CK currency (whether seeded IDs remain active in the pinned version)
is out of scope for this property and MUST be validated by periodic manual
reconciliation against the ATT&CK STIX bundle. A future hardening item could
automate this by diffing `SEEDED_TECHNIQUE_IDS` against the `ics-attack-19.1.json`
non-revoked set, but that mechanism is not part of VP-007.

## F4 Harness-Update Obligations (F2 issue #7 — directives v2 + ADR-006)

> **Non-normative appendix. This section records F4 harness/source obligations that keep the
> locked VP-007 proof green after the Modbus F2 revision. It does NOT edit the property statement,
> the lock fields (`verification_lock`, `proof_completed_date`, `proof_file_hash`,
> `verified_at_commit`), the source BCs, or any frontmatter count. Same pattern as the VP-004
> extension obligation recorded for Feature #100. The authoritative version of these obligations
> lives in `.factory/phase-f2-spec-evolution/verification-delta.md §4`; this appendix is a pointer
> co-located with the VP for traceability.**

The MITRE catalog grows in the Modbus F2 commit (directives v2 Decision 12). The VP-007 harness /
catalog-drift guard MUST be updated in the SAME F4 commit so the locked proof stays
`VERIFICATION:- SUCCESSFUL`:

| Quantity (mitre.rs) | Pre-F2 expected | Post-F2 expected (directives v2) |
|---------------------|-----------------|----------------------------------|
| `SEEDED_TECHNIQUE_ID_COUNT` / `SEEDED_TECHNIQUE_IDS.len()` | 15 | **21** (11 Enterprise + 10 ICS; +6 new ICS arms: T0836, T0814, T0806, T0835, T0831, T0888) |
| `EMITTED_IDS.len()` | 6 (Enterprise only) | **13** (6 Enterprise + 7 ICS: **T1692.001** [was T0855, remapped issue #222], T0836, T0814, T0806, T0835, T0831, **T0888**) |
| Recon-path emitted ID | n/a (no ICS emitted) | **T0888** "Remote System Information Discovery" (corrects the v1 T0846 misattribution; **T0846 stays SEEDED but is NOT Modbus-emitted**) |
| Emitted-ID grep pattern | `mitre_technique: Some` | `mitre_techniques: vec!` (ADR-006 Decision 13: `Finding` field rename `Option<String>` → `Vec<String>`) |

**POL-11 positive-coverage obligation (carried forward; updated issue #222):** the guard MUST assert the
runtime-computed counts `EMITTED_IDS.len() == 13` and `SEEDED_TECHNIQUE_ID_COUNT == 21`, and MUST
deliberately resolve ≥1 newly-added ICS ID **including T0888** AND **T1692.001** (the remapped
successor of revoked T0855) through `technique_name` + `technique_tactic` (assert `Some(..)` on
both), so the proof cannot pass false-green over an empty/no-op loop. Re-run `cargo kani` over the
VP-007 harnesses after the atomic catalog update; both `VERIFICATION:- SUCCESSFUL` and the
positive-coverage assertions MUST hold. The ICS sub-technique IDs `T1692.001` and `T1692.002` MUST
be present in `SEEDED_TECHNIQUE_IDS` and satisfy the `T[0-9]{4}(\.[0-9]{3})?` format check.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
| F4 harness-update obligation recorded (issue #7: SEEDED 15→21, EMITTED 6→13, recon T0888, field-rename grep) — lock fields unchanged | 2026-06-09 | formal-verifier |
| v2.1 spec update (issue #222): Sub-property A format rule made explicit for ICS sub-techniques; T0855→T1692.001, T0856→T1692.002 in harness skeleton SEEDED_IDS/EMITTED_IDS; Known Limitation section added | 2026-06-10 | architect |
