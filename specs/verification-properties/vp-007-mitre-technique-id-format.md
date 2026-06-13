---
document_type: verification-property
level: L4
version: "2.4"
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
  - "v2.2 (2026-06-10, Pass-2 remediation, issue #8 DNP3 F2): Updated catalog counts from 21/13 to 23/15. SEEDED_IDS: added T1691.001 and T0827 (ICS section now 12 entries); ICS comment updated 10→12. EMITTED_IDS: added T1691.001 and T0827 (now 15: 6 Enterprise + 9 ICS); comment updated. F4 Harness-Update Obligations table: Post-F2 expected SEEDED 21→23, EMITTED 13→15; positive-coverage obligation assertions updated to == 23 / == 15. Added Re-verification Obligation note: verification_lock must be broken and VP-007 re-proven in F6 against the post-F4 catalog containing T1691.001+T0827 (CC-002). Lock fields (verification_lock, proof_completed_date, proof_file_hash, verified_at_commit) and property statement are UNCHANGED — the lock itself is not broken until F6 re-run."
  - "v2.3 (2026-06-13, corpus-wide consistency audit remediation IR-1): F4 Harness-Update Obligations table extended with Post-ARP column (issue #9, STORY-114): SEEDED 23→25 (12 Enterprise + 13 ICS; +T0830 ICS LateralMovement, +T1557.002 Enterprise CredentialAccess); EMITTED 15→17 (7E+10I; +T0830+T1557.002). POL-11 positive-coverage obligation updated to assert ==25/==17. Re-verification Obligation section CC-003 added (ARP F2 issue #9, STORY-114). Lock fields and property statement UNCHANGED — lock broken + re-proven at STORY-114 F6 per CC-003."
  - "v2.4 (2026-06-13, Pass-12 corpus debt cleanup F-C-P12-001): Source Location line anchor corrected: 'src/mitre.rs:122-156' → 'src/mitre.rs:128-182'. Verified against live src/mitre.rs: pub fn technique_info at line 128; let info = match id { at line 129; _ => return None at line 179; closing }; of match at line 180; Some(info) at line 181; closing } of function at line 182. The prior range 122-156 was pre-F2 stale (technique_info was shorter before F2 Modbus/DNP3 arms were added). No proof-lock, property statement, or BC change."
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

- **Primary BC:** BC-2.10.005 -- technique_name Returns Some for Every Seeded ID (23 Total; post-F2 issue #8 DNP3 + issue #222 remap)
- **Postcondition:** `technique_name(id).is_some()` for all 23 seeded IDs
- **Invariant:** INV-9 (MITRE Technique ID Format, inv-01-core-invariants.md)
- **Related BC:** BC-2.10.006 -- technique_name Returns None for Unknown IDs
- **Related BC:** BC-2.10.007 -- technique_tactic Returns Correct Tactic for Every Seeded ID
- **Related BC:** BC-2.10.008 -- All Emitted Technique IDs Resolve in Lookup

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- enumerate all 23 seeded IDs explicitly | Complete coverage of the static match table |

The static match in `mitre.rs` is closed-world enumeration. Kani can enumerate all
23 known IDs (including ICS sub-technique IDs in `T[0-9]{4}.[0-9]{3}` form) plus
verify that the return type (Option) is handled correctly.

## Proof Harness Skeleton

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // All 23 seeded IDs from mitre.rs technique_info (src/mitre.rs — post-F2 issue #8 DNP3 + issue #222 remap)
    // ICS sub-technique IDs T1692.001 and T1692.002 replace the revoked T0855 and T0856
    // per MITRE ATT&CK-ICS v19.1 (issue #222).
    // T1691.001 and T0827 added in F2 issue #8 (DNP3 TCP analyzer).
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
        // ICS (12)
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
        "T1691.001",  // DNP3: Block OT Message: Command Message (IcsInhibitResponseFunction) — issue #8
        "T0827",      // DNP3: Loss of Control (IcsImpact) — issue #8
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
    // Post-F2 issue #8 (DNP3) + issue #7 (Modbus) + issue #222 remap: 15 total (6 Enterprise + 9 ICS)
    // T0855 replaced by T1692.001 (MITRE ATT&CK-ICS v19.1 revocation, issue #222)
    // T1691.001 and T0827 added by DNP3 analyzer (issue #8)
    const EMITTED_IDS: &[&str] = &[
        // Enterprise (6)
        "T1027",      // TLS analyzer: SNI anomaly
        "T1036",      // Reassembly: conflicting overlap
        "T1046",      // HTTP: admin panel
        "T1083",      // HTTP: path traversal
        "T1499.002",  // HTTP: header flood
        "T1505.003",  // HTTP: web shell
        // ICS (9) — Modbus analyzer (post-F2 issue #7) + DNP3 analyzer (issue #8)
        "T1692.001",  // was T0855: Modbus write FCs + DNP3 unauthorized control (issue #222 remap)
        "T0836",      // Modbus + DNP3: Modify Parameter
        "T0814",      // Modbus + DNP3: Denial of Service
        "T0806",      // Modbus: Brute Force I/O
        "T0835",      // Modbus: Manipulate I/O Image
        "T0831",      // Modbus: Manipulation of Control
        "T0888",      // Modbus recon path: Remote System Information Discovery
        "T1691.001",  // DNP3: Block OT Message: Command Message (request/response inference) — issue #8
        "T0827",      // DNP3: Loss of Control (derived/correlated Impact finding) — issue #8
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
| Input space size | Finite | 23 seeded IDs; closed enumeration |
| Proof complexity | Very low | Static match table; no loops or state |
| Tool support | High | `technique_name` is a pure function returning Option<&str> |
| Estimated proof time | < 10 seconds | Trivial Kani proof; simple unit test sufficient as fallback |

## Source Location

`src/mitre.rs:128-182` -- `technique_info` static match block (let info = match id at 129; _ => return None at 179).

The 23 currently-seeded IDs include T1040, T1071, T1071.001, T1071.004, T1573,
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

| Quantity (mitre.rs) | Pre-F2 expected | Post-F2 expected (Modbus directives v2) | Post-DNP3 expected (issue #8) | Post-ARP expected (issue #9, STORY-114) |
|---------------------|-----------------|------------------------------------------|-------------------------------|------------------------------------------|
| `SEEDED_TECHNIQUE_ID_COUNT` / `SEEDED_TECHNIQUE_IDS.len()` | 15 | **21** (11 Enterprise + 10 ICS; +6 new ICS arms: T0836, T0814, T0806, T0835, T0831, T0888) | **23** (11 Enterprise + 12 ICS; +T1691.001 + T0827) | **25** (12 Enterprise + 13 ICS; +T1557.002 Enterprise CredentialAccess + T0830 ICS LateralMovement) |
| `EMITTED_IDS.len()` | 6 (Enterprise only) | **13** (6 Enterprise + 7 ICS: **T1692.001** [was T0855, remapped issue #222], T0836, T0814, T0806, T0835, T0831, **T0888**) | **15** (6 Enterprise + 9 ICS; +T1691.001 + T0827) | **17** (7 Enterprise + 10 ICS; +T1557.002 Enterprise, +T0830 ICS — both emitted by ArpAnalyzer D1 spoof/D2 GARP/D12 mismatch) |
| Recon-path emitted ID | n/a (no ICS emitted) | **T0888** "Remote System Information Discovery" (corrects the v1 T0846 misattribution; **T0846 stays SEEDED but is NOT Modbus-emitted**) | (unchanged) | (unchanged) |
| Emitted-ID grep pattern | `mitre_technique: Some` | `mitre_techniques: vec!` (ADR-006 Decision 13: `Finding` field rename `Option<String>` → `Vec<String>`) | (unchanged) | (unchanged) |

**POL-11 positive-coverage obligation (carried forward; updated issue #9 ARP F2 / STORY-114):** the guard MUST assert the
runtime-computed counts `EMITTED_IDS.len() == 17` and `SEEDED_TECHNIQUE_ID_COUNT == 25`, and MUST
deliberately resolve ≥1 newly-added ICS ID **including T0888**, **T1692.001** (the remapped
successor of revoked T0855), **T1691.001**, **T0827**, **T0830**, and **T1557.002** through
`technique_name` + `technique_tactic` (assert `Some(..)` on both), so the proof cannot pass
false-green over an empty/no-op loop.
Re-run `cargo kani` over the VP-007 harnesses after the atomic catalog update; both
`VERIFICATION:- SUCCESSFUL` and the positive-coverage assertions MUST hold. The ICS sub-technique
IDs `T1692.001`, `T1692.002`, and `T1691.001` MUST be present in `SEEDED_TECHNIQUE_IDS` and satisfy
the `T[0-9]{4}(\.[0-9]{3})?` format check. T1557.002 (Enterprise sub-technique) satisfies the
same `T[0-9]{4}\.[0-9]{3}` branch. The seeded 12E+13I split is normative per BC-2.10.005
(T1557.002 is Enterprise, T0830 is ICS).

## Re-verification Obligation (CC-002 — DNP3 F2, issue #8)

> **VP-007 carries `verification_lock: true`. Per VP-lock discipline CC-002, the lock MUST be
> broken and VP-007 re-proven after any structural change that invalidates the existing proof.**

The proof enumerates `SEEDED_IDS` and `EMITTED_IDS` explicitly. Adding T1691.001 and T0827 to
both arrays changes the enumeration such that:

- `verify_all_seeded_ids_resolve` now iterates 23 IDs (was 21); the proof is structurally
  identical but the catalog must contain the two new `technique_info` arms or it fails.
- `all_emitted_ids_resolve_in_catalog` now checks 15 IDs (was 13); same structural form.

**Obligation:** When the DNP3 F4 implementation story delivers the `technique_info` arms for
T1691.001 and T0827 (per ADR-007 Decision 5 §9.1 of the architecture delta), the F4/F6
formal-verifier MUST:

1. Break the lock: set `verification_lock: false` in this VP's frontmatter.
2. Re-run `cargo kani` against the updated `src/mitre.rs` containing T1691.001 + T0827 arms,
   the updated `SEEDED_TECHNIQUE_IDS` (23 entries), and the updated `EMITTED_IDS` (15 entries).
3. Confirm `VERIFICATION:- SUCCESSFUL` for all VP-007 harnesses (`verify_all_seeded_ids_resolve`,
   `verify_unknown_id_returns_none_no_panic`, `all_emitted_ids_resolve_in_catalog`).
4. Confirm the POL-11 positive-coverage assertions pass: `SEEDED_TECHNIQUE_ID_COUNT == 23`,
   `EMITTED_IDS.len() == 15`.
5. Re-lock: set `verification_lock: true`, update `proof_completed_date`, `proof_file_hash`,
   and `verified_at_commit` to the F4/F6 run values.

**This is not optional.** A locked VP-007 with a 21-entry `SEEDED_IDS` array but a 23-entry
production catalog is a false-green locked proof — it no longer characterizes the live code.
The re-verification obligation was triggered by the catalog growth (T1691.001 + T0827) introduced
in F2 issue #8 (DNP3 TCP analyzer).

## Re-verification Obligation (CC-003 — ARP F2, issue #9, STORY-114)

> **VP-007 carries `verification_lock: true`. The lock MUST be broken and VP-007 re-proven after
> STORY-114 delivers the T0830 and T1557.002 `technique_info` arms. This obligation is
> authoritative; the corresponding obligation pointer is in
> `.factory/specs/architecture/arp-architecture-delta.md §5`.**

The proof enumerates `SEEDED_IDS` and `EMITTED_IDS` explicitly. Adding T0830 and T1557.002 to
both arrays changes the enumeration such that:

- `verify_all_seeded_ids_resolve` now iterates 25 IDs (was 23 after CC-002); the proof is
  structurally identical but the catalog must contain the two new `technique_info` arms or it
  fails.
- `all_emitted_ids_resolve_in_catalog` now checks 17 IDs (was 15 after CC-002); same structural
  form.

The seeded split is **12 Enterprise + 13 ICS = 25 total** (T1557.002 is Enterprise
CredentialAccess; T0830 is ICS LateralMovement). This is consistent with BC-2.10.005
post-STORY-114 postcondition. Do NOT use the split 11E+14I.

**Obligation:** When STORY-114 F4 implementation delivers the `technique_info` arms for T0830
and T1557.002 (per arp-architecture-delta.md §5 five-part atomic update), the F4/F6
formal-verifier MUST:

1. Break the lock: set `verification_lock: false` in this VP's frontmatter.
2. Re-run `cargo kani` against the updated `src/mitre.rs` containing T0830 + T1557.002 arms,
   the updated `SEEDED_TECHNIQUE_IDS` (25 entries), and the updated `EMITTED_IDS` (17 entries).
3. Confirm `VERIFICATION:- SUCCESSFUL` for all VP-007 harnesses (`verify_all_seeded_ids_resolve`,
   `verify_unknown_id_returns_none_no_panic`, `all_emitted_ids_resolve_in_catalog`).
4. Confirm the POL-11 positive-coverage assertions pass: `SEEDED_TECHNIQUE_ID_COUNT == 25`,
   `EMITTED_IDS.len() == 17`.
5. Re-lock: set `verification_lock: true`, update `proof_completed_date`, `proof_file_hash`,
   and `verified_at_commit` to the F4/F6 run values.

**Note on obligation sequencing:** CC-002 (DNP3) must be discharged in STORY-109 F6 before
CC-003 (ARP/STORY-114) is triggered. CC-003 is a NEW obligation layered on top of CC-002;
the STORY-114 F6 re-run verifies the combined 25-entry catalog (post-CC-002 23-entry catalog
plus the two ARP additions).

**This is not optional.** A locked VP-007 with a 23-entry `SEEDED_IDS` array but a 25-entry
production catalog is a false-green locked proof — it no longer characterizes the live code.
The re-verification obligation was triggered by the catalog growth (T0830 + T1557.002)
introduced in F2 issue #9 (ARP security analyzer, STORY-114).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
| F4 harness-update obligation recorded (issue #7: SEEDED 15→21, EMITTED 6→13, recon T0888, field-rename grep) — lock fields unchanged | 2026-06-09 | formal-verifier |
| v2.1 spec update (issue #222): Sub-property A format rule made explicit for ICS sub-techniques; T0855→T1692.001, T0856→T1692.002 in harness skeleton SEEDED_IDS/EMITTED_IDS; Known Limitation section added | 2026-06-10 | architect |
| v2.2 Pass-2 remediation (issue #8 DNP3 F2): SEEDED count 21→23 (T1691.001 + T0827 added, ICS now 12); EMITTED count 13→15 (9 ICS); F4 Obligations table updated; POL-11 assertions updated to ==23/==15; Re-verification Obligation (CC-002) section added; lock fields UNCHANGED — re-lock deferred to F6 re-run | 2026-06-10 | architect |
| v2.3 corpus-wide consistency audit remediation (IR-1): F4 Obligations table Post-ARP column added (STORY-114: SEEDED 23→25, 12E+13I; EMITTED 15→17, 7E+10I; T0830 ICS + T1557.002 Enterprise); POL-11 assertions updated to ==25/==17; Re-verification Obligation (CC-003) section added mirroring CC-002 pattern; lock fields UNCHANGED — re-lock deferred to STORY-114 F6 | 2026-06-13 | architect |
