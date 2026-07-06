---
artifact: verification-property
vp_id: VP-030
title: "pcapng Multi-IDB Linktype Agreement Totality (WHITELISTED domain)"
status: verified
phase: P1
tool: proptest
subsystem: SS-01
module: "reader.rs"
producer: architect
timestamp: 2026-06-19T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-009-pcapng-reader-design.md
feature_cycle: feature-pcapng-reader
source_bc: BC-2.01.018
bcs:
  - BC-2.01.018
verification_lock: true
verified_at_commit: "1ca30a3"
verified_prs: "#293, #294"
---

# VP-030: pcapng Multi-IDB Linktype Agreement Totality (WHITELISTED domain)

## Property Statement

The multi-IDB linktype agreement check in `src/reader.rs` satisfies the following property
over the **WHITELISTED DataLink domain only**. Non-whitelisted DataLink values short-circuit
to `E-INP-001` before the agreement check is reached; they are out of VP-030's scope.

For all sequences of WHITELISTED DataLink IDB values:

1. **All-equal → Ok:** If all IDBs in the sequence have the same WHITELISTED DataLink
   value, the reader returns `Ok` (no linktype conflict).
2. **First-differing → Err(E-INP-011):** If the first IDB with a DataLink value differing
   from the initial IDB is encountered, the reader returns `Err(E-INP-011)` on that IDB
   (not a subsequent one — the error is triggered at the first disagreement).
3. **Comparison unit is DataLink (not raw u16):** The linktype agreement check compares
   `DataLink` enum values (after whitelisted-value parsing), not raw `u16` values from
   the IDB body. This prevents false positives from numeric coincidences.

**Domain restriction rationale (ADR-009 rev 7 / H-3):** The original VP-030 domain
(`any sequence of IDB linktype u16 values`) included non-whitelisted values that are
unreachable by the agreement check (they hit `E-INP-001` first). The restated domain
eliminates those unreachable paths and makes the property non-vacuous.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.01.018 | Multi-IDB linktype agreement: all-equal WHITELISTED IDBs → Ok; first-differing → Err(E-INP-011); comparison unit is DataLink | All harnesses |

## Proof Harnesses

Three proptest harnesses:

```rust
proptest! {
    // Sub-1: All IDBs with the same WHITELISTED linktype → Ok.
    fn proptest_VP_030_all_equal_whitelisted_idbs_ok(...) { ... }

    // Sub-2: First IDB differing from the initial WHITELISTED linktype → Err(E-INP-011).
    fn proptest_VP_030_first_differing_whitelisted_idb_errs_e_inp_011(...) { ... }

    // Sub-3: Comparison unit is DataLink enum, not raw u16.
    fn proptest_VP_030_comparison_unit_is_datalink(...) { ... }
}
```

All three harnesses confirmed at F6 lock @ develop 1ca30a3 (PRs #293 + #294).

## Feasibility Assessment

**Assessment: FEASIBLE (completed — proptest suite confirmed at F6 lock).**

The multi-IDB agreement check is a sequence-comparison property over a finite set of
WHITELISTED DataLink values. proptest is the natural tool for sequence-over-domain
properties; the domain restriction to WHITELISTED values keeps the strategy space small
and non-vacuous.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-030 designed (ADR-009 rev 4); domain restated in rev 7 (H-3): WHITELISTED only | draft |
| F4 (TDD implementation) | 3 proptest harnesses authored | draft |
| F6 (formal hardening) | All 3 harnesses confirmed in CI; 0 failures at F6 lock | draft → verified |

Lock: `status: verified`, `verification_lock: true` set by state-manager after F6 confirmation
@ develop 1ca30a3 (PRs #293 + #294).
