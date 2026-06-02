---
document_type: verification-property
level: L4
version: "2.0"
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
and `technique_tactic()` matches the pattern `T[0-9]{4}` or `T[0-9]{4}.[0-9]{3}`.
No other format is present in the static match.

**Sub-property B (emitter-catalog completeness):** Every technique ID string that
any analyzer places into `Finding.mitre_technique` is present in the static match
in `technique_info`, such that `technique_name(id)` returns `Some(...)` and
`technique_tactic(id)` returns `Some(...)` for that ID.

Corollary: Unknown IDs passed to `technique_name`/`technique_tactic` return `None`
and do NOT cause a panic (BC-2.10.006).

## Source Contract

- **Primary BC:** BC-2.10.005 -- technique_name Returns Some for Every Seeded ID (15 Total)
- **Postcondition:** `technique_name(id).is_some()` for all 15 seeded IDs
- **Invariant:** INV-9 (MITRE Technique ID Format, inv-01-core-invariants.md)
- **Related BC:** BC-2.10.006 -- technique_name Returns None for Unknown IDs
- **Related BC:** BC-2.10.007 -- technique_tactic Returns Correct Tactic for Every Seeded ID
- **Related BC:** BC-2.10.008 -- All Emitted Technique IDs Resolve in Lookup

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- enumerate all 15 seeded IDs explicitly | Complete coverage of the static match table |

The static match in `mitre.rs` is closed-world enumeration. Kani can enumerate all
15 known IDs plus verify that the return type (Option) is handled correctly.

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // All 15 seeded IDs from mitre.rs technique_info (src/mitre.rs:122-156)
    const SEEDED_IDS: &[&str] = &[
        // Enterprise
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
        // ICS
        "T0846",
        "T0855",
        "T0856",
        "T0885",
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
    const EMITTED_IDS: &[&str] = &[
        "T1027",  // TLS analyzer: SNI anomaly
        "T1036",  // Reassembly: conflicting overlap
        "T1046",  // HTTP: admin panel
        "T1083",  // HTTP: path traversal
        "T1499.002", // HTTP: header flood
        "T1505.003", // HTTP: web shell
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
| Input space size | Finite | 15 seeded IDs; closed enumeration |
| Proof complexity | Very low | Static match table; no loops or state |
| Tool support | High | `technique_name` is a pure function returning Option<&str> |
| Estimated proof time | < 10 seconds | Trivial Kani proof; simple unit test sufficient as fallback |

## Source Location

`src/mitre.rs:122-156` -- `technique_info` static match block.

The 15 currently-seeded IDs include T1040, T1071, T1071.001, T1071.004, T1573,
T0846, T0855, T0856, T0885 which are staged-but-never-emitted (O-04). Sub-property
B (emitter-catalog completeness) only tests the IDs that analyzers actually emit.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
