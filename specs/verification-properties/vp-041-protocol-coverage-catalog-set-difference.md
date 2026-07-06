---
artifact: verification-property
vp_id: VP-041
title: "Protocol Coverage Catalog Set-Difference Correctness — Oracle Cross-Check + Partition Invariant"
status: draft
phase: P1
tool: proptest
subsystem: SS-18
module: "src/protocols.rs"
producer: architect
timestamp: 2026-07-01T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
feature_cycle: feature-protocol-coverage
source_bc: BC-2.18.003
bcs:
  - BC-2.18.003
  - BC-2.18.004
verification_lock: false
---

# VP-041: Protocol Coverage Catalog Set-Difference Correctness — Oracle Cross-Check + Partition Invariant

## Property Statement

The pure-core functions `supported_protocols()` and `unsupported_protocols()` in
`src/protocols.rs` satisfy two complementary properties over the static `KNOWN_PROTOCOLS`
compile-time array:

### Sub-1: Oracle Cross-Check (non-vacuous)

For each entry in `KNOWN_PROTOCOLS`, independently compute whether it should be in the
supported set using an oracle that does NOT call `supported_protocols()` or
`unsupported_protocols()`:

```
oracle_supported = entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p))
                   || entry.name == "ARP"
```

Then assert that `supported_protocols()` returns **exactly** the entries where
`oracle_supported == true` (set-equality by name).

This harness is **non-vacuous and falsifiable** (DF-KANI-NONVACUITY-001): it can detect a
broken `supported_protocols()` that returns too many, too few, or wrong entries. The
partition/disjoint invariants alone cannot detect such breakage because they hold trivially
whenever `unsupported = KNOWN \\ supported` by definition.

### Sub-2: Partition Invariant

For all entries in `KNOWN_PROTOCOLS`:
- `supported_protocols() ∪ unsupported_protocols() = KNOWN_PROTOCOLS` (partition completeness)
- `supported_protocols() ∩ unsupported_protocols() = ∅` (disjoint — no entry in both sets)

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.18.003 | `supported_protocols()` returns exactly the protocols with a port in `SUPPORTED_PORTS` or name "ARP"; `KNOWN_PROTOCOLS = supported ∪ unsupported` (partition completeness) | Sub-1 (oracle) + Sub-2 (partition) |
| BC-2.18.004 | `unsupported_protocols()` is the complement of `supported_protocols()` in `KNOWN_PROTOCOLS`; `supported ∩ unsupported = ∅` | Sub-2 (disjoint) |

## Purity Classification

`supported_protocols()` and `unsupported_protocols()` are **pure-core** functions:
- No I/O; no global mutable state; no heap allocation beyond the returned slices
- Both operate on the `KNOWN_PROTOCOLS` compile-time constant array
- Outputs are deterministic and reproducible across all invocations

## Proof Harnesses

```rust
#[cfg(test)]
mod vp041_protocol_coverage_catalog_set_difference {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// VP-041 Sub-1: Oracle cross-check.
        /// For each KNOWN_PROTOCOLS entry, independently determine oracle_supported
        /// and assert supported_protocols() returns exactly those entries.
        /// The oracle does NOT call supported_protocols() or unsupported_protocols().
        #[test]
        fn proptest_vp041_oracle_cross_check(_seed: u32) {
            let supported = supported_protocols();
            let supported_names: std::collections::HashSet<&str> =
                supported.iter().map(|e| e.name).collect();

            for entry in KNOWN_PROTOCOLS {
                let oracle_supported = entry.canonical_ports.iter()
                    .any(|p| SUPPORTED_PORTS.contains(p))
                    || entry.name == "ARP";

                if oracle_supported {
                    prop_assert!(
                        supported_names.contains(entry.name),
                        "oracle says {} is supported but supported_protocols() excludes it",
                        entry.name
                    );
                } else {
                    prop_assert!(
                        !supported_names.contains(entry.name),
                        "oracle says {} is unsupported but supported_protocols() includes it",
                        entry.name
                    );
                }
            }
        }

        /// VP-041 Sub-2: Partition invariant.
        /// supported ∪ unsupported = KNOWN_PROTOCOLS AND supported ∩ unsupported = ∅.
        #[test]
        fn proptest_vp041_partition_invariant(_seed: u32) {
            let supported = supported_protocols();
            let unsupported = unsupported_protocols();

            let supported_names: std::collections::HashSet<&str> =
                supported.iter().map(|e| e.name).collect();
            let unsupported_names: std::collections::HashSet<&str> =
                unsupported.iter().map(|e| e.name).collect();

            // Partition completeness: every KNOWN_PROTOCOLS entry is in exactly one set.
            for entry in KNOWN_PROTOCOLS {
                let in_supported = supported_names.contains(entry.name);
                let in_unsupported = unsupported_names.contains(entry.name);
                prop_assert!(
                    in_supported ^ in_unsupported,
                    "entry {} must be in exactly one of supported/unsupported",
                    entry.name
                );
            }

            // Disjoint: intersection is empty.
            let intersection: Vec<&&str> = supported_names.iter()
                .filter(|n| unsupported_names.contains(*n))
                .collect();
            prop_assert!(
                intersection.is_empty(),
                "supported and unsupported must be disjoint; found overlap: {:?}",
                intersection
            );
        }
    }
}
```

## Feasibility Assessment

**Assessment: FEASIBLE.**

Both target functions are pure-core with no loops over symbolic inputs — they operate
on the static `KNOWN_PROTOCOLS` compile-time array. proptest provides the appropriate
framework for set-equality assertions. The oracle cross-check harness is non-vacuous
by design (it can distinguish correct from incorrect implementations). Default proptest
case count (100) is sufficient because the property is structural, not numeric.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-041 added (feature-protocol-coverage design layer); reframed F-F2P1-008 to add non-vacuous oracle harness | draft |
| F4 (TDD implementation) | 2 proptest harnesses authored for protocols.rs | draft → active |
| F6 (formal hardening) | proptest suite confirmed in CI; status active → verified | active → verified |

Lock gate: `status: verified` and `verification_lock: true` set by state-manager after
F6 confirmation. Mirrors VP-033/VP-035/VP-037 lock pattern.
